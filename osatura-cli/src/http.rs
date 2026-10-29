// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! A deliberately small HTTP/1.1 server for the local studio (std only).
//!
//! Why not a library: the studio needs hard limits that the previous server library did not offer.
//! Every connection gets its own thread, but only a bounded number of them (`Limits::max_conns`; the rest are
//! answered `503` at once). Reading the request head and the body each has a deadline (`408` on expiry), a
//! `Content-Length` above the limit is answered `413` before any body byte is read, chunked bodies are refused,
//! and a stalled client therefore cannot hold up any other client: the connection threads only do I/O, the
//! requests themselves are handed to the single handler thread (which owns the editor) through a channel.
//! Every response is `Connection: close` (no keep-alive state to attack) and carries the security headers
//! (`Content-Security-Policy`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

/// A parsed request. Header names keep their spelling; look them up with [`HttpReq::header`].
pub struct HttpReq {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    /// IP address of the TCP peer (no port); empty when unknown. Behind a reverse proxy this is the proxy's address.
    pub peer: String,
}

impl HttpReq {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
}

/// A complete response (the body is in memory).
#[derive(Debug)]
pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Resp {
    pub fn from_string(s: impl Into<String>) -> Resp {
        Resp { status: 200, headers: vec![("Content-Type".into(), "text/plain; charset=utf-8".into())], body: s.into().into_bytes() }
    }
    pub fn from_data(b: Vec<u8>) -> Resp {
        Resp { status: 200, headers: vec![("Content-Type".into(), "application/octet-stream".into())], body: b }
    }
    /// Sets a header, replacing an earlier one of the same name.
    pub fn with_header(mut self, h: (String, String)) -> Resp {
        self.headers.retain(|(n, _)| !n.eq_ignore_ascii_case(&h.0));
        self.headers.push(h);
        self
    }
    pub fn with_status_code(mut self, c: u16) -> Resp {
        self.status = c;
        self
    }
}

/// What the handler answers: a full response, or an event stream fed from a channel.
pub enum Reply {
    Full(Resp),
    Stream(mpsc::Receiver<String>),
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub head_deadline: Duration,
    pub body_deadline: Duration,
    pub write_timeout: Duration,
    pub max_head: usize,
    pub max_body: usize,
    pub max_conns: usize,
    /// Most simultaneous connections from ONE remote address (0 = no limit): a client that opens connections and never
    /// finishes a request cannot use up the whole `max_conns` and lock everybody else out before authentication.
    pub max_conns_per_peer: usize,
    /// Also apply `max_conns_per_peer` to loopback addresses. Off by default: a reverse proxy on the same machine
    /// connects from 127.0.0.1 on behalf of every visitor and must limit its own clients (docs/SECURITY.md section 12).
    pub limit_loopback_peers: bool,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits { head_deadline: Duration::from_secs(10), body_deadline: Duration::from_secs(20), write_timeout: Duration::from_secs(10), max_head: 16 << 10, max_body: 32 << 20, max_conns: 128, max_conns_per_peer: 16, limit_loopback_peers: false }
    }
}

pub const CSP: &str = "default-src 'self'; img-src 'self' data: blob:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'";

fn reason(c: u16) -> &'static str {
    match c {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        411 => "Length Required",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        421 => "Misdirected Request",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Status",
    }
}

fn security_headers() -> [(&'static str, &'static str); 4] {
    [("Content-Security-Policy", CSP), ("X-Content-Type-Options", "nosniff"), ("X-Frame-Options", "DENY"), ("Referrer-Policy", "no-referrer")]
}

/// The bytes of a response head + body (what is written to the socket).
pub fn encode(r: &Resp) -> Vec<u8> {
    let mut out = format!("HTTP/1.1 {} {}\r\n", r.status, reason(r.status)).into_bytes();
    let have = |n: &str| r.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(n));
    for (k, v) in &r.headers {
        // header values never contain line breaks (they would split the response)
        if k.contains(['\r', '\n']) || v.contains(['\r', '\n']) {
            continue;
        }
        out.extend_from_slice(format!("{k}: {v}\r\n").as_bytes());
    }
    for (k, v) in security_headers() {
        if !have(k) {
            out.extend_from_slice(format!("{k}: {v}\r\n").as_bytes());
        }
    }
    if !have("cache-control") {
        out.extend_from_slice(b"Cache-Control: no-store\r\n");
    }
    out.extend_from_slice(format!("Content-Length: {}\r\nConnection: close\r\n\r\n", r.body.len()).as_bytes());
    out.extend_from_slice(&r.body);
    out
}

type Fail = (u16, &'static str);

fn find_head_end(b: &[u8]) -> Option<usize> {
    b.windows(4).position(|w| w == b"\r\n\r\n")
}

/// Parse a request head (without the blank line).
/// A `(name, value)` request header.
pub type Header = (String, String);

pub fn parse_head(head: &[u8]) -> Result<(Method, String, Vec<Header>), Fail> {
    let text = std::str::from_utf8(head).map_err(|_| (400, "request head is not UTF-8"))?;
    let mut lines = text.split("\r\n");
    let line = lines.next().unwrap_or("");
    let mut it = line.split(' ');
    let (m, u, v) = (it.next().unwrap_or(""), it.next().unwrap_or(""), it.next().unwrap_or(""));
    if it.next().is_some() || m.is_empty() || u.is_empty() {
        return Err((400, "bad request line"));
    }
    if v != "HTTP/1.1" && v != "HTTP/1.0" {
        return Err((400, "unsupported HTTP version"));
    }
    let method = match m {
        "GET" => Method::Get,
        "POST" => Method::Post,
        _ => return Err((405, "only GET and POST are served")),
    };
    if !u.starts_with('/') || u.bytes().any(|c| c < 0x21 || c == 0x7f) {
        return Err((400, "bad request target"));
    }
    let mut headers = Vec::new();
    for l in lines {
        let Some((k, val)) = l.split_once(':') else { return Err((400, "bad header line")) };
        if k.is_empty() || !k.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_!#$%&'*+.^`|~".contains(&c)) {
            return Err((400, "bad header name"));
        }
        if headers.len() >= 100 {
            return Err((431, "too many headers"));
        }
        // RFC 9110: no control characters in a field value (a stray CR/NUL could be read differently by a proxy in front)
        if val.bytes().any(|c| (c < 0x20 && c != b'\t') || c == 0x7f) {
            return Err((400, "control character in a header value"));
        }
        headers.push((k.to_string(), val.trim().to_string()));
    }
    Ok((method, u.to_string(), headers))
}

fn set_left(s: &TcpStream, deadline: Instant) -> Result<(), Fail> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return Err((408, "request timed out"));
    }
    s.set_read_timeout(Some(left)).map_err(|_| (500, "socket error"))
}

fn timed_out(e: &std::io::Error) -> bool {
    matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)
}

/// Read one request from `s` under the deadlines and size limits of `lim`.
pub fn read_request(s: &mut TcpStream, lim: &Limits) -> Result<HttpReq, Fail> {
    let t0 = Instant::now();
    let mut buf: Vec<u8> = Vec::with_capacity(2048);
    let mut chunk = [0u8; 4096];
    let end = loop {
        if let Some(e) = find_head_end(&buf) {
            break e;
        }
        if buf.len() > lim.max_head {
            return Err((431, "request head too large"));
        }
        set_left(s, t0 + lim.head_deadline)?;
        match s.read(&mut chunk) {
            Ok(0) => return Err((400, "connection closed in the request head")),
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(e) if timed_out(&e) => return Err((408, "request head timed out")),
            Err(_) => return Err((400, "read error")),
        }
    };
    if end > lim.max_head {
        return Err((431, "request head too large"));
    }
    let (method, url, headers) = parse_head(&buf[..end])?;
    let mut body: Vec<u8> = buf[end + 4..].to_vec();
    let get = |n: &str| -> Vec<&str> { headers.iter().filter(|(k, _)| k.eq_ignore_ascii_case(n)).map(|(_, v)| v.as_str()).collect() };
    if get("host").len() > 1 {
        // two Host headers: proxies and this server could disagree on which one is meant (host-confusion attacks)
        return Err((400, "duplicate Host header"));
    }
    if !get("transfer-encoding").is_empty() {
        return Err((411, "chunked bodies are not accepted: send Content-Length"));
    }
    let cls = get("content-length");
    let len: usize = match cls.as_slice() {
        [] => 0,
        [one] => one.parse().map_err(|_| (400, "bad Content-Length"))?,
        many if many.iter().all(|x| x == &many[0]) => many[0].parse().map_err(|_| (400, "bad Content-Length"))?,
        _ => return Err((400, "conflicting Content-Length headers")),
    };
    if len > lim.max_body {
        return Err((413, "request body too large"));
    }
    if method == Method::Get && len > 0 {
        return Err((400, "a GET request must not have a body"));
    }
    if get("expect").iter().any(|v| v.eq_ignore_ascii_case("100-continue")) && body.len() < len {
        let _ = s.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
    }
    let bt = Instant::now() + lim.body_deadline;
    while body.len() < len {
        set_left(s, bt)?;
        let want = (len - body.len()).min(chunk.len());
        match s.read(&mut chunk[..want]) {
            Ok(0) => return Err((400, "connection closed in the request body")),
            Ok(n) => body.extend_from_slice(&chunk[..n]),
            Err(e) if timed_out(&e) => return Err((408, "request body timed out")),
            Err(_) => return Err((400, "read error")),
        }
    }
    body.truncate(len);
    let peer = s.peer_addr().map(|a| a.ip().to_string()).unwrap_or_default();
    Ok(HttpReq { method, url, headers, body, peer })
}

fn write_all_timeout(s: &mut TcpStream, b: &[u8], lim: &Limits) -> bool {
    let _ = s.set_write_timeout(Some(lim.write_timeout));
    s.write_all(b).and_then(|_| s.flush()).is_ok()
}

fn finish(mut s: TcpStream, unread: bool) {
    let _ = s.shutdown(Shutdown::Write);
    if unread {
        // let the client see the answer instead of a reset: swallow what it still sends, briefly
        let t0 = Instant::now();
        let mut sink = [0u8; 8192];
        let mut total = 0usize;
        while t0.elapsed() < Duration::from_millis(500) && total < (1 << 20) {
            let _ = s.set_read_timeout(Some(Duration::from_millis(200)));
            match s.read(&mut sink) {
                Ok(0) | Err(_) => break,
                Ok(n) => total += n,
            }
        }
    }
}

struct Job {
    req: HttpReq,
    tx: mpsc::Sender<Reply>,
}

fn serve_conn(mut s: TcpStream, lim: Limits, jobs: mpsc::Sender<Job>) {
    let _ = s.set_nodelay(true);
    let req = match read_request(&mut s, &lim) {
        Ok(r) => r,
        Err((code, msg)) => {
            write_all_timeout(&mut s, &encode(&Resp::from_string(msg).with_status_code(code)), &lim);
            finish(s, true);
            return;
        }
    };
    let (tx, rx) = mpsc::channel();
    if jobs.send(Job { req, tx }).is_err() {
        return;
    }
    match rx.recv() {
        Ok(Reply::Full(r)) => {
            write_all_timeout(&mut s, &encode(&r), &lim);
            finish(s, false);
        }
        Ok(Reply::Stream(events)) => {
            let mut head = String::from("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\nX-Accel-Buffering: no\r\n");
            for (k, v) in security_headers() {
                head.push_str(&format!("{k}: {v}\r\n"));
            }
            head.push_str("\r\nretry: 2000\n\n");
            if !write_all_timeout(&mut s, head.as_bytes(), &lim) {
                return;
            }
            loop {
                let frame = match events.recv_timeout(Duration::from_secs(10)) {
                    Ok(f) => f,
                    Err(mpsc::RecvTimeoutError::Timeout) => ": keep-alive\n\n".to_string(),
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                };
                if !write_all_timeout(&mut s, frame.as_bytes(), &lim) {
                    break;
                }
            }
        }
        Err(_) => {}
    }
}

/// Open connections per remote address (only for the addresses `max_conns_per_peer` applies to).
#[derive(Default)]
struct PeerCounts(Mutex<HashMap<std::net::IpAddr, usize>>);

impl PeerCounts {
    /// Count one more connection from `ip`; false (and nothing counted) when it already has `max`.
    fn acquire(&self, ip: std::net::IpAddr, max: usize) -> bool {
        let mut g = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let c = g.entry(ip).or_insert(0);
        if *c >= max {
            return false;
        }
        *c += 1;
        true
    }

    fn release(&self, ip: std::net::IpAddr) {
        let mut g = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(c) = g.get_mut(&ip) {
            *c = c.saturating_sub(1);
            if *c == 0 {
                g.remove(&ip);
            }
        }
    }
}

/// One accepted connection's claim on the global and the per-address counters; released when it ends.
struct Slot {
    total: Arc<AtomicUsize>,
    peer: Option<(Arc<PeerCounts>, std::net::IpAddr)>,
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.total.fetch_sub(1, Ordering::SeqCst);
        if let Some((counts, ip)) = &self.peer {
            counts.release(*ip);
        }
    }
}

/// Accept connections on `listener` and answer each request with `handler` (called on the calling thread, one
/// request at a time). Returns when the listener fails.
pub fn serve_loop(listener: TcpListener, lim: Limits, mut handler: impl FnMut(HttpReq) -> Reply) {
    let (jobs_tx, jobs_rx) = mpsc::channel::<Job>();
    let live = Arc::new(AtomicUsize::new(0));
    let l2 = live.clone();
    let peers = Arc::new(PeerCounts::default());
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut s) = conn else { continue };
            // the per-address cap first: one address must not be able to fill the global cap
            let per_peer = s.peer_addr().ok().map(|a| a.ip()).filter(|ip| lim.max_conns_per_peer > 0 && (lim.limit_loopback_peers || !ip.is_loopback()));
            if let Some(ip) = per_peer {
                if !peers.acquire(ip, lim.max_conns_per_peer) {
                    write_all_timeout(&mut s, &encode(&Resp::from_string("too many connections from this address").with_status_code(503)), &lim);
                    continue;
                }
            }
            let slot = Slot { total: l2.clone(), peer: per_peer.map(|ip| (peers.clone(), ip)) };
            if l2.fetch_add(1, Ordering::SeqCst) >= lim.max_conns {
                // (dropping the slot gives back both counters)
                drop(slot);
                write_all_timeout(&mut s, &encode(&Resp::from_string("too many connections").with_status_code(503)), &lim);
                continue;
            }
            let jt = jobs_tx.clone();
            if std::thread::Builder::new().spawn(move || {
                let _slot = slot;
                serve_conn(s, lim, jt);
            }).is_err() {
                // (the slot moved into the closure is dropped with it)
            }
        }
    });
    for job in jobs_rx {
        let r = handler(job.req);
        let _ = job.tx.send(r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lim() -> Limits {
        Limits { head_deadline: Duration::from_millis(400), body_deadline: Duration::from_millis(400), write_timeout: Duration::from_secs(2), max_head: 4096, max_body: 1024, max_conns: 6, max_conns_per_peer: 0, limit_loopback_peers: false }
    }

    fn start(l: Limits) -> std::net::SocketAddr {
        let li = TcpListener::bind("127.0.0.1:0").unwrap();
        let a = li.local_addr().unwrap();
        std::thread::spawn(move || {
            serve_loop(li, l, |r| {
                let body = format!("{:?} {} {}", r.method, r.url, r.body.len());
                Reply::Full(Resp::from_string(body))
            })
        });
        a
    }

    fn roundtrip(a: std::net::SocketAddr, raw: &[u8]) -> String {
        let mut s = TcpStream::connect(a).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        s.write_all(raw).unwrap();
        let mut out = Vec::new();
        let _ = s.read_to_end(&mut out);
        String::from_utf8_lossy(&out).into_owned()
    }

    #[test]
    fn answers_with_security_headers_and_parses_the_body() {
        let a = start(lim());
        let r = roundtrip(a, b"POST /x?y=1 HTTP/1.1\r\nHost: h\r\nContent-Length: 3\r\n\r\nabc");
        assert!(r.starts_with("HTTP/1.1 200 OK"), "{r}");
        assert!(r.ends_with("Post /x?y=1 3"));
        for h in ["Content-Security-Policy: default-src 'self'", "X-Content-Type-Options: nosniff", "X-Frame-Options: DENY", "Referrer-Policy: no-referrer"] {
            assert!(r.contains(h), "missing {h} in {r}");
        }
    }

    #[test]
    fn oversized_body_is_refused_before_it_is_read() {
        let a = start(lim());
        // 413 comes although no body byte was sent at all
        let r = roundtrip(a, b"POST /x HTTP/1.1\r\nContent-Length: 999999999\r\n\r\n");
        assert!(r.starts_with("HTTP/1.1 413"), "{r}");
        assert!(roundtrip(a, b"POST /x HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n").starts_with("HTTP/1.1 411"));
        assert!(roundtrip(a, b"POST /x HTTP/1.1\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\nab").starts_with("HTTP/1.1 400"));
        assert!(roundtrip(a, b"PUT /x HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 405"));
        assert!(roundtrip(a, b"GET nope HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 400"));
        let big = format!("GET /x HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(9000));
        assert!(roundtrip(a, big.as_bytes()).starts_with("HTTP/1.1 431"));
    }

    #[test]
    fn two_host_headers_are_refused_and_the_peer_address_is_known() {
        let a = start(lim());
        let r = roundtrip(a, b"GET /x HTTP/1.1\r\nHost: a.example\r\nHost: b.example\r\n\r\n");
        assert!(r.starts_with("HTTP/1.1 400") && r.contains("duplicate Host"), "{r}");
        let r = roundtrip(a, b"GET /x HTTP/1.1\r\nhost: a.example\r\nHOST: a.example\r\n\r\n");
        assert!(r.starts_with("HTTP/1.1 400"), "case-insensitive: {r}");
        assert!(roundtrip(a, b"GET /x HTTP/1.1\r\nHost: a.example\r\n\r\n").starts_with("HTTP/1.1 200"));
        // the handler sees the TCP peer
        let li = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = li.local_addr().unwrap();
        std::thread::spawn(move || serve_loop(li, lim(), |r| Reply::Full(Resp::from_string(format!("peer={}", r.peer)))));
        assert!(roundtrip(addr, b"GET /x HTTP/1.1\r\n\r\n").ends_with("peer=127.0.0.1"));
    }

    #[test]
    fn control_characters_in_header_values_are_refused() {
        let a = start(lim());
        for bad in [&b"GET /x HTTP/1.1\r\nHost: a.example\r\nX: a\x00b\r\n\r\n"[..], b"GET /x HTTP/1.1\r\nHost: a.example\r\nX: a\rb\r\n\r\n", b"GET /x HTTP/1.1\r\nHost: a.example\r\nX: a\x7fb\r\n\r\n"] {
            assert!(roundtrip(a, bad).starts_with("HTTP/1.1 400"), "{:?}", String::from_utf8_lossy(bad));
        }
        // a horizontal tab inside a value is legal
        assert!(roundtrip(a, b"GET /x HTTP/1.1\r\nHost: a.example\r\nX: a\tb\r\n\r\n").starts_with("HTTP/1.1 200"));
    }

    #[test]
    fn a_stalled_client_times_out_and_blocks_nobody() {
        let a = start(lim());
        // a POST that promises 100 bytes and sends 2, and a head that never ends
        let mut slow_body = TcpStream::connect(a).unwrap();
        slow_body.write_all(b"POST /x HTTP/1.1\r\nContent-Length: 100\r\n\r\nab").unwrap();
        let mut slow_head = TcpStream::connect(a).unwrap();
        slow_head.write_all(b"GET /x HTTP/1.1\r\nHost: a").unwrap();
        // while both stall, other clients are served at once
        let t0 = Instant::now();
        for _ in 0..5 {
            assert!(roundtrip(a, b"GET /ok HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 200"));
        }
        assert!(t0.elapsed() < Duration::from_millis(300), "served while others stall: {:?}", t0.elapsed());
        for s in [&mut slow_body, &mut slow_head] {
            s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut out = String::new();
            let _ = s.read_to_string(&mut out);
            assert!(out.starts_with("HTTP/1.1 408"), "{out}");
        }
    }

    #[test]
    fn one_address_cannot_use_up_the_connection_cap() {
        // 3 per address (loopback counted for the test), 6 in all: a flood from one address leaves room for another
        let a = start(Limits { max_conns_per_peer: 3, limit_loopback_peers: true, ..lim() });
        let holders: Vec<TcpStream> = (0..3).map(|_| {
            let mut s = TcpStream::connect(a).unwrap();
            s.write_all(b"GET /x HTTP/1.1\r\n").unwrap();
            s
        }).collect();
        std::thread::sleep(Duration::from_millis(100));
        let r = roundtrip(a, b"GET /ok HTTP/1.1\r\n\r\n");
        assert!(r.starts_with("HTTP/1.1 503") && r.contains("from this address"), "{r}");
        // the slots come back when the stalled connections end (408 after the head deadline), and nothing leaks
        std::thread::sleep(Duration::from_millis(700));
        drop(holders);
        // slot release is not instant on a loaded runner (seen on Windows CI): allow a few seconds, not one guess
        let mut r = String::new();
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(100));
            r = roundtrip(a, b"GET /ok HTTP/1.1\r\n\r\n");
            if r.starts_with("HTTP/1.1 200") {
                break;
            }
        }
        assert!(r.starts_with("HTTP/1.1 200"), "{r}");
    }

    #[test]
    fn loopback_peers_are_not_counted_by_default() {
        // a reverse proxy on this machine connects from 127.0.0.1 for everybody: no per-address cap there
        let a = start(Limits { max_conns_per_peer: 2, ..lim() });
        let holders: Vec<TcpStream> = (0..4).map(|_| {
            let mut s = TcpStream::connect(a).unwrap();
            s.write_all(b"GET /x HTTP/1.1\r\n").unwrap();
            s
        }).collect();
        std::thread::sleep(Duration::from_millis(100));
        assert!(roundtrip(a, b"GET /ok HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 200"));
        drop(holders);
    }

    #[test]
    fn peer_counts_acquire_and_release() {
        let c = PeerCounts::default();
        let ip: std::net::IpAddr = "192.0.2.7".parse().unwrap();
        let other: std::net::IpAddr = "192.0.2.8".parse().unwrap();
        assert!(c.acquire(ip, 2) && c.acquire(ip, 2));
        assert!(!c.acquire(ip, 2), "the third is refused");
        assert!(c.acquire(other, 2), "another address has its own count");
        c.release(ip);
        assert!(c.acquire(ip, 2), "a released slot can be taken again");
        c.release(ip);
        c.release(ip);
        c.release(ip); // releasing more than was taken never underflows
        assert!(c.0.lock().unwrap().get(&ip).is_none(), "an address with no connection is forgotten");
    }

    #[test]
    fn the_number_of_connections_is_capped() {
        let a = start(lim());
        let holders: Vec<TcpStream> = (0..6).map(|_| {
            let mut s = TcpStream::connect(a).unwrap();
            s.write_all(b"GET /x HTTP/1.1\r\n").unwrap();
            s
        }).collect();
        std::thread::sleep(Duration::from_millis(100));
        let r = roundtrip(a, b"GET /ok HTTP/1.1\r\n\r\n");
        assert!(r.starts_with("HTTP/1.1 503"), "{r}");
        drop(holders);
    }
}
