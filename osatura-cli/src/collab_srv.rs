// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! HTTP side of the collaboration session (R-111): routes under `/api/collab/`, the server-sent-events
//! stream, the optional shared token. The merge logic is in `osatura_core::collabhub` / `collab`.
//!
//! Security, honestly: plain HTTP (no TLS), no user accounts. With `--collab-token T` (at least 16
//! characters) every request must carry the token, as the `X-Collab-Token` header or in the `osatura_token`
//! cookie that `POST /api/login` sets (HttpOnly, SameSite=Strict); the token is never accepted in the URL, so it
//! does not end up in logs, history or Referer headers. It is compared in constant time (as SHA-256 digests),
//! and failed attempts are slowed down per client address (from the fifth in a row the server answers 429 to that
//! address for 2, 4, ... 30 s; a wrong cookie counts like a wrong header). The right token is always accepted, also
//! from a locked address, so a stale cookie or somebody else's guesses never keep the owner out: the price is that
//! the back-off slows guessing but cannot stop an attacker who tries, which is why the token must be long and random
//! (at least 16 characters, `openssl rand -hex 16`). Behind a reverse proxy every client has the proxy's address.
//! Without a token anybody who can reach the port can read and change the project and write the save path;
//! the default bind is 127.0.0.1 and `studio.rs` refuses other addresses without a token. A participant id is
//! only a label: joining returns a random 128-bit secret and every call made as that participant must send it
//! (`X-Collab-Secret`), so knowing or guessing the numeric id is not enough to act as someone.
//!
//! Persistence (`--collab-log PATH`): the session's checkpoint and every accepted envelope are written to
//! a line-oriented log (format in `osatura_core::collablog`). Appends are one `write_all` per entry
//! followed by one `fdatasync` per request (so an edit is on disk before its HTTP answer is sent; start
//! with `--collab-nosync` to leave flushing to the operating system: faster, but a power cut can lose
//! the last edits). A new session, and every compaction, rewrites the file atomically (temporary file,
//! `fsync`, `rename`, directory `fsync`). A crash while appending leaves at worst one cut-off last line,
//! which is dropped on the next start. A log that is damaged elsewhere, or that belongs to a different
//! project, stops the server with an error instead of being overwritten. Participants and undo history are
//! not persisted: after a restart the pages join again (new ids) and receive the resumed state. If the log
//! cannot be written the server says so on stderr, keeps serving from memory and stops logging.

use crate::studio::{err, state};
use osatura_core::collab::{Envelope, Presence};
use osatura_core::collabauth::{self, Bucket, TokenError};
use osatura_core::collablog;
use osatura_core::collabhub::{Event, Hub, Mode, SubmitError, LOCAL_ID};
use osatura_core::edit::{Editor, Op};
use osatura_core::pick;
use serde_json::{json, Value};
use std::io::Write;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use crate::http::{Method, Resp};

/// Log length of the running session as seen by `state()` (-1 = no session).
pub static COLLAB_INDEX: AtomicI64 = AtomicI64::new(-1);

const MAX_STREAMS: usize = 64;

/// Per-request identity the page sends on every call.
#[derive(Default, Clone)]
pub struct Ctx {
    pub user: Option<u32>,
    pub base: Option<usize>,
    /// The participant secret sent with the request (`X-Collab-Secret`).
    pub secret: Option<String>,
}

/// The open log file: where, the chain hash of its last line and the sync policy.
struct LogFile {
    path: std::path::PathBuf,
    file: Option<std::fs::File>,
    prev: String,
    sync: bool,
    failed: bool,
    /// Digest of the document the session first started from (kept across compactions).
    origin: String,
}

impl LogFile {
    /// A fresh temporary name next to the log (random, so it cannot be guessed and pre-created).
    fn tmp_path(&self) -> std::path::PathBuf {
        let mut n = self.path.as_os_str().to_owned();
        n.push(format!(".tmp-{}", osatura_core::fsutil::random_hex(6)));
        n.into()
    }

    /// Replace the file by a single checkpoint line for `hub` (atomically) and reopen it for appending.
    fn rewrite(&mut self, hub: &Hub) -> Result<(), String> {
        let (line, h) = collablog::encode_header(hub, &self.origin);
        let tmp = self.tmp_path();
        let io = |what: &str, e: std::io::Error| format!("collaboration log {}: {what}: {e}", self.path.display());
        {
            // exclusive creation, readable by the owner only (the log holds the whole project)
            let mut f = osatura_core::fsutil::create_private_file(&tmp).map_err(|e| io("cannot create temporary file", e))?;
            f.write_all(line.as_bytes()).map_err(|e| io("write failed", e))?;
            f.sync_all().map_err(|e| io("fsync failed", e))?;
        }
        self.file = None;
        std::fs::rename(&tmp, &self.path).map_err(|e| io("rename failed", e))?;
        if let Some(dir) = self.path.parent().filter(|d| !d.as_os_str().is_empty()).or(Some(std::path::Path::new("."))) {
            if let Ok(d) = std::fs::File::open(dir) {
                let _ = d.sync_all(); // best effort: makes the rename itself durable
            }
        }
        self.file = Some(std::fs::OpenOptions::new().append(true).open(&self.path).map_err(|e| io("cannot reopen for appending", e))?);
        self.prev = h;
        self.failed = false;
        Ok(())
    }

    fn append(&mut self, evs: &[Event]) -> Result<(), String> {
        let Some(f) = self.file.as_mut() else { return Ok(()) };
        let mut buf = String::new();
        let mut prev = self.prev.clone();
        for e in evs {
            if let Event::Ops { index, by, envelope } = e {
                let (l, h) = collablog::encode_entry(&prev, *index, *by, envelope);
                buf.push_str(&l);
                prev = h;
            }
        }
        if buf.is_empty() {
            return Ok(());
        }
        let r = f.write_all(buf.as_bytes()).and_then(|_| if self.sync { f.sync_data() } else { f.flush() });
        r.map_err(|e| format!("collaboration log {}: append failed: {e}", self.path.display()))?;
        self.prev = prev;
        Ok(())
    }
}

/// Cached selection bounds: (layer paths, frame, project revision, bounds).
type BoundsEntry = (Vec<String>, i64, u64, Option<[f32; 4]>);

pub struct Collab {
    pub hub: Hub,
    pub token: Option<String>,
    subs: Vec<mpsc::Sender<String>>,
    t0: Instant,
    log: Option<LogFile>,
    /// The hub was rebuilt from the log and no participant has joined it yet.
    resumed: bool,
    /// Last computed selection bounds per participant: (layer paths, frame, project revision) -> bounds.
    bounds_cache: std::collections::BTreeMap<u32, BoundsEntry>,
    /// Participant secrets (random 128 bit, hex), by participant id.
    secrets: std::collections::BTreeMap<u32, String>,
    /// Failed token attempts in a row, per client address, and until when that client is answered 429.
    clients: std::collections::HashMap<String, Attempts>,
    /// Server secret for per-participant tokens (`osatura_core::collabauth`), set with `--collab-secret-file`.
    secret: Option<String>,
    /// Participant names whose tokens are refused although they verify (`--collab-revoke`, `POST /api/collab/revoke`).
    revoked: Vec<String>,
    /// `--collab-host`: names of the per-participant tokens that may save the project, compact the session log and
    /// revoke tokens. `None` = not restricted (the default, as before).
    hosts: Option<Vec<String>>,
    /// `(requests per second, burst)` allowed per client address; `None` = no limit.
    rate: Option<(f64, f64)>,
    buckets: std::collections::HashMap<String, Bucket>,
    /// Participant name of the last request that authenticated with a per-participant token.
    authed: Option<String>,
    /// Why the last refused credential was refused when that is worth telling (expired, revoked).
    auth_note: Option<String>,
    /// Test hook: the Unix time used for token expiry instead of the system clock.
    unix_now: Option<u64>,
}

#[derive(Clone, Copy)]
struct Attempts {
    fails: u32,
    blocked_until: Option<Instant>,
    last: Instant,
}

/// Most client addresses remembered; beyond that the idlest entry is dropped.
const MAX_CLIENTS: usize = 512;

/// Most participant names that can be revoked while the server runs.
const MAX_REVOKED: usize = 1000;

/// Constant-time comparison: both sides are hashed first, so neither the content nor the length leaks.
fn ct_eq(a: &str, b: &str) -> bool {
    let (a, b) = (osatura_core::vcs::sha256_hex(a.as_bytes()), osatura_core::vcs::sha256_hex(b.as_bytes()));
    let mut d = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        d |= x ^ y;
    }
    d == 0
}

/// Shortest accepted `--collab-token`.
pub const MIN_TOKEN_LEN: usize = 16;

/// A token must be at least [`MIN_TOKEN_LEN`] characters of `A-Za-z0-9._~+/=-` (it travels in a cookie).
pub fn check_token(t: &str) -> Result<(), String> {
    if t.chars().count() < MIN_TOKEN_LEN {
        return Err(format!("--collab-token must be at least {MIN_TOKEN_LEN} characters (e.g. `openssl rand -hex 16`)"));
    }
    if !t.bytes().all(|c| c.is_ascii_alphanumeric() || b"._~+/=-".contains(&c)) {
        return Err("--collab-token may only contain letters, digits and . _ ~ + / = -".into());
    }
    Ok(())
}

/// Reads the server secret from a file (first line, trimmed). A group- or world-readable file is refused on Unix:
/// the secret mints tokens for everybody.
pub fn read_secret_file(path: &str) -> Result<String, String> {
    osatura_core::fsutil::refuse_symlink(std::path::Path::new(path))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let m = std::fs::metadata(path).map_err(|e| format!("cannot read secret file {path}: {e}"))?;
        if m.permissions().mode() & 0o077 != 0 {
            return Err(format!("secret file {path} is readable by other users (mode {:o}); run: chmod 600 {path}", m.permissions().mode() & 0o777));
        }
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read secret file {path}: {e}"))?;
    let secret = text.lines().next().unwrap_or("").trim().to_string();
    osatura_core::collabauth::check_secret(&secret)?;
    Ok(secret)
}

/// The secret for `collab-token`: `--secret-file F`, else the environment variable `OSATURA_COLLAB_SECRET`.
pub fn secret_from(file: Option<&str>) -> Result<String, String> {
    match file {
        Some(f) => read_secret_file(f),
        None => {
            let v = std::env::var("OSATURA_COLLAB_SECRET").map_err(|_| "give --secret-file F (or set OSATURA_COLLAB_SECRET); the secret is never accepted on the command line, where other users can see it".to_string())?;
            osatura_core::collabauth::check_secret(&v)?;
            Ok(v.trim().to_string())
        }
    }
}

fn submit_err(e: SubmitError) -> Value {
    json!({"ok": false, "errors": e.errors, "resync": e.resync})
}

fn qp<'a>(query: &'a str, k: &str) -> Option<&'a str> {
    query.split('&').find_map(|kv| kv.strip_prefix(k).and_then(|r| r.strip_prefix('=')))
}

fn replica_of(v: &Value) -> Result<u32, Value> {
    v["replica"].as_u64().filter(|x| *x > 0 && *x < u32::MAX as u64).map(|x| x as u32).ok_or_else(|| err(vec!["missing or bad 'replica'".into()]))
}

impl Collab {
    pub fn new(doc: &Value, token: Option<String>) -> Collab {
        Collab {
            hub: Hub::new(doc),
            token,
            subs: vec![],
            t0: Instant::now(),
            log: None,
            resumed: false,
            bounds_cache: Default::default(),
            secrets: Default::default(),
            clients: Default::default(),
            secret: None,
            revoked: Vec::new(),
            hosts: None,
            rate: None,
            buckets: Default::default(),
            authed: None,
            auth_note: None,
            unix_now: None,
        }
    }

    /// Accept per-participant tokens (`osatura collab-token`) made with `secret`; `revoked` names are refused.
    pub fn with_secret(mut self, secret: &str, revoked: Vec<String>) -> Result<Collab, String> {
        collabauth::check_secret(secret)?;
        for n in &revoked {
            collabauth::check_name(n).map_err(|e| format!("--collab-revoke: {e}"))?;
        }
        self.secret = Some(secret.trim().to_string());
        self.revoked = revoked;
        Ok(self)
    }

    /// Only these participant names (tokens made with `--collab-secret-file`) may save the project, compact the session log
    /// and revoke tokens; everybody else with a valid token can still edit. Needs the secret (the names come from the tokens).
    pub fn with_hosts(mut self, hosts: Vec<String>) -> Result<Collab, String> {
        if self.secret.is_none() {
            return Err("--collab-host needs --collab-secret-file (the host names come from per-participant tokens)".into());
        }
        if hosts.is_empty() {
            return Err("--collab-host needs at least one participant name".into());
        }
        for n in &hosts {
            collabauth::check_name(n).map_err(|e| format!("--collab-host: {e}"))?;
        }
        self.hosts = Some(hosts);
        Ok(self)
    }

    /// May the request that was just authenticated save the project and compact the session log? Everyone may unless
    /// `--collab-host` is given; with it only the named per-participant tokens may. The shared `--collab-token` and a
    /// server that needs no credential at all are not restricted (they have no names).
    pub fn is_host(&self) -> bool {
        match (&self.authed, &self.hosts) {
            (Some(name), Some(hosts)) => hosts.iter().any(|h| h == name),
            _ => true,
        }
    }

    /// May the request that was just authenticated revoke tokens? Stricter than `is_host`: with per-participant tokens
    /// only the named hosts may (without `--collab-host` nobody may: restart with `--collab-revoke` instead).
    pub fn can_administer(&self) -> bool {
        match (&self.authed, &self.hosts) {
            (None, _) => true,
            (Some(name), Some(hosts)) => hosts.iter().any(|h| h == name),
            (Some(_), None) => false,
        }
    }

    /// Refuse the tokens of `names` from now on and remove the participants who joined under those names (an event
    /// stream that is already open ends when its client disconnects; its next request is refused). Returns the names that
    /// were not revoked before.
    pub fn revoke_names(&mut self, names: &[String]) -> Result<Vec<String>, String> {
        if self.secret.is_none() {
            return Err("no per-participant tokens are in use (start the studio with --collab-secret-file)".into());
        }
        if names.is_empty() || names.len() > 64 {
            return Err("give 1 to 64 participant names".into());
        }
        for n in names {
            collabauth::check_name(n)?;
        }
        let mut added = Vec::new();
        for n in names {
            if !self.revoked.contains(n) {
                if self.revoked.len() >= MAX_REVOKED {
                    return Err(format!("too many revoked names ({MAX_REVOKED}): change the secret instead, which revokes every token"));
                }
                self.revoked.push(n.clone());
                added.push(n.clone());
            }
        }
        for u in self.hub.users_view() {
            if let (Some(id), Some(name)) = (u["replica"].as_u64(), u["name"].as_str()) {
                if names.iter().any(|n| n == name) {
                    self.secrets.remove(&(id as u32));
                    self.hub.leave(id as u32);
                }
            }
        }
        Ok(added)
    }

    /// Limit every client address to `per_sec` requests per second (bursts of twice that); 0 switches the limit off.
    pub fn with_rate(mut self, per_sec: f64) -> Collab {
        self.rate = if per_sec > 0.0 && per_sec.is_finite() { Some((per_sec, (per_sec * 2.0).max(2.0))) } else { None };
        self
    }

    fn unix_now(&self) -> u64 {
        self.unix_now.unwrap_or_else(osatura_core::clock::unix_seconds)
    }

    /// Does the server require a credential at all?
    pub fn auth_required(&self) -> bool {
        self.token.is_some() || self.secret.is_some()
    }

    /// Why the last refused credential was refused ("token expired ...", "token revoked"), when that is worth saying.
    pub fn auth_note(&self) -> Option<&str> {
        self.auth_note.as_deref()
    }

    /// Request-rate limit: `Ok` or `Err(seconds to wait)`. Applies per client address, after authentication.
    pub fn check_rate(&mut self, client: &str) -> Result<(), u64> {
        let Some((rate, burst)) = self.rate else { return Ok(()) };
        let now = self.now();
        if !self.buckets.contains_key(client) && self.buckets.len() >= MAX_CLIENTS {
            if let Some(oldest) = self.buckets.iter().min_by_key(|(_, b)| b.last_ms()).map(|(k, _)| k.clone()) {
                self.buckets.remove(&oldest);
            }
        }
        let b = self.buckets.entry(client.to_string()).or_insert_with(|| Bucket::new(burst, now));
        if b.take(rate, burst, now) {
            Ok(())
        } else {
            Err(((1.0 / rate).ceil() as u64).max(1))
        }
    }

    /// Is `given` an accepted credential: the shared token, or a per-participant token that verifies, is not
    /// revoked and has not expired? Sets `authed` / `auth_note`.
    fn credential_ok(&mut self, given: &str) -> bool {
        if let Some(tok) = &self.token {
            if ct_eq(given, tok) {
                self.authed = None;
                return true;
            }
        }
        let Some(sec) = self.secret.clone() else { return false };
        match collabauth::verify_token(&sec, given, self.unix_now()) {
            Ok(c) if self.revoked.contains(&c.name) => {
                self.auth_note = Some(format!("the token of '{}' was revoked", c.name));
                false
            }
            Ok(c) => {
                self.authed = Some(c.name);
                true
            }
            Err(TokenError::Expired { expires }) => {
                self.auth_note = Some(format!("the token expired (valid until Unix time {expires}); ask the host for a new one"));
                false
            }
            Err(_) => false,
        }
    }

    /// Attach the persistent log at `path`. An existing log is replayed: the hub resumes with its state, clock
    /// and index and the editor takes over the logged document. Errors (damaged log, a log of another project,
    /// a document that no longer validates) are returned for the caller to print; the file is never modified then.
    pub fn open_log(&mut self, ed: &mut Editor, path: &str, sync: bool) -> Result<String, String> {
        let p = std::path::PathBuf::from(path);
        osatura_core::fsutil::refuse_symlink(&p)?;
        let mut lf = LogFile { path: p.clone(), file: None, prev: String::new(), sync, failed: false, origin: String::new() };
        let existing = match std::fs::read(&p) {
            Ok(b) if b.is_empty() => None,
            Ok(b) => Some(String::from_utf8(b).map_err(|_| format!("collaboration log {path} is not valid UTF-8 text: damaged or not a log"))?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(format!("cannot read collaboration log {path}: {e}")),
        };
        let msg = match existing {
            None => {
                self.log = Some(lf);
                format!("collaboration log {path}: new (written when the first participant joins)")
            }
            Some(text) => {
                let l = collablog::load(&text).map_err(|e| format!("{e} (file {path}; fix or remove it, it was left untouched)"))?;
                let project = collablog::digest(&Hub::new(ed.doc()).doc_json());
                if project != l.origin && project != l.current {
                    return Err(format!("collaboration log {path} belongs to a different project state than the one loaded (neither the session's starting document nor its last state); start the studio with the matching project file, or move the log away. The log was left untouched."));
                }
                ed.replace_doc(l.hub.doc_json()).map_err(|e| format!("collaboration log {path}: the logged project no longer validates: {}", e.join("; ")))?;
                if l.torn_tail {
                    // Truncate through a plain write handle: on Windows an append-only handle may not resize the file.
                    let t = std::fs::OpenOptions::new().write(true).open(&p).map_err(|e| format!("cannot open {path} to drop the cut-off last line: {e}"))?;
                    t.set_len(l.valid_len as u64).map_err(|e| format!("cannot drop the cut-off last line of {path}: {e}"))?;
                    let _ = t.sync_all();
                }
                let f = std::fs::OpenOptions::new().append(true).open(&p).map_err(|e| format!("cannot open {path} for appending: {e}"))?;
                lf.file = Some(f);
                lf.prev = l.last_hash.clone();
                lf.origin = l.origin.clone();
                let info = format!("collaboration log {path}: resumed at index {} ({} envelopes after the checkpoint{})", l.hub.index(), l.entries, if l.torn_tail { "; a cut-off last line was dropped" } else { "" });
                self.hub = l.hub;
                self.resumed = true;
                self.log = Some(lf);
                self.publish_index();
                info
            }
        };
        Ok(msg)
    }

    fn log_checkpoint(&mut self) {
        let hub = &self.hub;
        if let Some(l) = self.log.as_mut() {
            if let Err(e) = l.rewrite(hub) {
                eprintln!("{e}; persistence is off for the rest of this run");
                l.file = None;
                l.failed = true;
            }
        }
    }

    pub fn now(&self) -> u64 {
        self.t0.elapsed().as_millis() as u64
    }

    /// True while at least one participant has joined (edits are then merged through the hub).
    pub fn active(&self) -> bool {
        self.hub.joined_count() > 0
    }

    /// Bounds of `sel` at `frame` for participant `id`, recomputed only when the selection, the frame (whole
    /// frames) or the project changed since the last call.
    fn bounds_for(&mut self, ed: &Editor, id: u32, sel: &[String], frame: Option<f32>) -> Option<[f32; 4]> {
        if sel.is_empty() {
            self.bounds_cache.remove(&id);
            return None;
        }
        let f = frame.filter(|f| f.is_finite()).unwrap_or(0.0);
        let key = (sel.to_vec(), f.round() as i64, ed.revision());
        if let Some((s, fr, rev, b)) = self.bounds_cache.get(&id) {
            if (s, *fr, *rev) == (&key.0, key.1, key.2) {
                return *b;
            }
        }
        let b = pick::layers_bounds(ed.engine(), key.1 as f32, sel);
        self.bounds_cache.insert(id, (key.0, key.1, key.2, b));
        b
    }

    /// After the project changed: selection boxes of everybody who has something selected follow the layers.
    pub fn refresh_bounds(&mut self, ed: &Editor) {
        let now = self.now();
        for p in self.hub.presence.all() {
            if p.selection.is_empty() {
                continue;
            }
            let b = self.bounds_for(ed, p.replica, &p.selection, p.frame);
            if b != p.bounds {
                let mut q = p.clone();
                q.bounds = b;
                let _ = self.hub.set_presence(p.replica, q, now);
            }
        }
        self.flush();
    }

    /// True while a log is attached and writing.
    pub fn logging(&self) -> bool {
        self.log.as_ref().map(|l| l.file.is_some()).unwrap_or(false)
    }

    pub fn publish_index(&self) {
        COLLAB_INDEX.store(if self.active() { self.hub.index() as i64 } else { -1 }, Ordering::SeqCst);
    }

    /// Send the hub's pending events to every open event stream.
    pub fn flush(&mut self) {
        self.publish_index();
        let evs = self.hub.take_events();
        if evs.is_empty() {
            return;
        }
        if let Some(l) = self.log.as_mut() {
            if let Err(e) = l.append(&evs) {
                eprintln!("{e}; persistence is off for the rest of this run");
                l.file = None;
                l.failed = true;
            }
        }
        let frames: Vec<String> = evs.iter().map(Event::to_sse).collect();
        self.subs.retain(|tx| frames.iter().all(|f| tx.send(f.clone()).is_ok()));
    }

    fn note_failure(&mut self, client: &str) {
        let now = Instant::now();
        if !self.clients.contains_key(client) && self.clients.len() >= MAX_CLIENTS {
            // forget the address that was idle longest
            if let Some(oldest) = self.clients.iter().min_by_key(|(_, a)| a.last).map(|(k, _)| k.clone()) {
                self.clients.remove(&oldest);
            }
        }
        let a = self.clients.entry(client.to_string()).or_insert(Attempts { fails: 0, blocked_until: None, last: now });
        a.fails = a.fails.saturating_add(1);
        a.last = now;
        if a.fails >= 5 {
            let secs = (1u64 << (a.fails - 4).min(5)).min(30);
            a.blocked_until = Some(now + Duration::from_secs(secs));
        }
    }

    fn blocked(&self, client: &str) -> bool {
        self.clients.get(client).and_then(|a| a.blocked_until).map(|t| Instant::now() < t).unwrap_or(false)
    }

    /// The Set-Cookie value must be one token: no control characters, `;`, `,`, space, quotes or backslash.
    fn token_cookie_safe(t: &str) -> bool {
        !t.is_empty() && t.bytes().all(|c| c.is_ascii_alphanumeric() || b"._~+/=-".contains(&c))
    }

    /// Check the credentials of a request from `client` (its address). `Err(401)` = missing or wrong,
    /// `Err(429)` = that client failed too often lately and sent a wrong credential again. A correct token is
    /// always accepted. A wrong header or a wrong cookie counts as a failed attempt; no credential at all does not.
    pub fn check_auth(&mut self, header: Option<&str>, cookie: Option<&str>, client: &str) -> Result<(), u16> {
        if !self.auth_required() {
            return Ok(());
        }
        self.authed = None;
        self.auth_note = None;
        let from_cookie = cookie.and_then(|c| c.split(';').filter_map(|p| p.trim().strip_prefix("osatura_token=")).next());
        let given: Vec<&str> = [header, from_cookie].into_iter().flatten().collect();
        for t in &given {
            if self.credential_ok(t) {
                self.auth_note = None;
                self.clients.remove(client);
                return Ok(());
            }
        }
        if given.is_empty() {
            return Err(if self.blocked(client) { 429 } else { 401 });
        }
        let was_blocked = self.blocked(client);
        // an expired or revoked token is a stale credential, not a guess: it does not feed the back-off
        if self.auth_note.is_none() {
            self.note_failure(client);
        }
        Err(if was_blocked { 429 } else { 401 })
    }

    /// `POST /api/login {"token": "..."}`: on success sets the HttpOnly, SameSite=Strict cookie (plus `Secure` when the
    /// request came in over HTTPS, see `Policy::is_https`).
    pub fn login(&mut self, body: &[u8], client: &str, https: bool) -> Resp {
        let json = |v: Value| Resp::from_string(v.to_string()).with_header(("Content-Type".into(), "application/json".into()));
        if !self.auth_required() {
            return json(json!({"ok": true, "token_required": false}));
        }
        let given = serde_json::from_slice::<Value>(body).ok().and_then(|v| v["token"].as_str().map(String::from)).unwrap_or_default();
        self.authed = None;
        self.auth_note = None;
        if self.credential_ok(&given) {
            if !Self::token_cookie_safe(&given) {
                // cannot happen with a token checked by `check_token`; never put such a value in a header
                return json(err(vec!["the configured token cannot be used in a cookie".into()])).with_status_code(500);
            }
            self.clients.remove(client);
            // a per-participant token's cookie lives no longer than the token
            let max_age = match (&self.authed, collabauth::verify_token(self.secret.as_deref().unwrap_or(""), &given, self.unix_now())) {
                (Some(_), Ok(c)) => format!("; Max-Age={}", c.expires.saturating_sub(self.unix_now())),
                _ => String::new(),
            };
            json(json!({"ok": true, "name": self.authed})).with_header(("Set-Cookie".into(), format!("osatura_token={given}; Path=/; HttpOnly; SameSite=Strict{max_age}{}", if https { "; Secure" } else { "" })))
        } else if let Some(note) = self.auth_note.clone() {
            json(err(vec![note])).with_status_code(401)
        } else {
            let was_blocked = self.blocked(client);
            self.note_failure(client);
            json(err(vec![if was_blocked { "too many failed attempts: wait a moment".into() } else { "wrong token".into() }])).with_status_code(if was_blocked { 429 } else { 401 })
        }
    }

    /// May this request act as the participant it names? Naming nobody, or a participant that is not in the
    /// session, is allowed (it acts as the host); naming a joined participant needs that participant's secret.
    pub fn verify_ctx(&self, ctx: &Ctx) -> bool {
        match ctx.user {
            Some(u) if self.hub.has_user(u) => match (self.secrets.get(&u), &ctx.secret) {
                (Some(s), Some(given)) => ct_eq(s, given),
                _ => false,
            },
            _ => true,
        }
    }

    fn require_secret(&self, id: u32, ctx: &Ctx) -> Result<(), Value> {
        let c = Ctx { user: Some(id), base: None, secret: ctx.secret.clone() };
        if self.verify_ctx(&c) {
            Ok(())
        } else {
            Err(err(vec![format!("participant {id}: the participant secret is missing or wrong (send X-Collab-Secret)")]))
        }
    }

    pub fn who(&self, ctx: &Ctx) -> u32 {
        ctx.user.filter(|u| self.hub.has_user(*u)).unwrap_or(LOCAL_ID)
    }

    /// After a request changed the editor by its own means (nudge, key, gizmo, ...): record the difference as
    /// the requesting participant's edit, so every other participant receives it.
    pub fn sync_editor(&mut self, ed: &Editor, ctx: &Ctx) {
        if !self.active() {
            return;
        }
        let who = self.who(ctx);
        let now = self.now();
        if let Err(e) = self.hub.submit_doc(who, ed.doc(), &mut |_| Ok(()), now) {
            eprintln!("collab: could not record an edit: {}", e.errors.join("; "));
        }
        self.flush();
    }

    /// `/api/op` while a session is active: the edit is interpreted in the sender's view (`ctx.base`).
    pub fn op(&mut self, ed: &mut Editor, ctx: &Ctx, op: Op) -> Result<Value, Value> {
        let who = self.who(ctx);
        let now = self.now();
        self.hub.submit_ops(who, ctx.base, &[op], &mut |j| ed.replace_doc(j.clone()), now).map_err(submit_err)?;
        self.flush();
        Ok(state(ed))
    }

    pub fn undo_redo(&mut self, ed: &mut Editor, ctx: &Ctx, undo: bool) -> Result<Value, Value> {
        let who = self.who(ctx);
        let now = self.now();
        self.hub.undo_redo(who, undo, &mut |j| ed.replace_doc(j.clone()), now).map_err(submit_err)?;
        self.flush();
        Ok(state(ed))
    }

    fn with_info(&self, mut v: Value, who: Option<u32>) -> Value {
        if let (Value::Object(m), Value::Object(s)) = (&mut v, self.hub.status(who)) {
            for (k, x) in s {
                m.insert(format!("collab_{k}"), x);
            }
        }
        v
    }

    /// Handle `/api/collab/*`. `None` means the path is not a collaboration route.
    pub fn route(&mut self, ed: &mut Editor, method: &Method, path: &str, query: &str, body: &str, ctx: &Ctx) -> Option<Result<Value, Value>> {
        let now = self.now();
        if !path.starts_with("/api/collab/") {
            return None;
        }
        self.hub.expire(now);
        let parse = |s: &str| -> Result<Value, Value> { serde_json::from_str(s).map_err(|e| err(vec![format!("bad JSON: {e}")])) };
        let r: Result<Value, Value> = (|| match (method, path) {
            (Method::Post, "/api/collab/join") => {
                let v = parse(body)?;
                // with per-participant tokens the name comes from the token: nobody can join under another name
                let authed = self.authed.clone();
                let name = authed.as_deref().unwrap_or_else(|| v["name"].as_str().unwrap_or(""));
                let mode = match v["mode"].as_str().unwrap_or("thin") {
                    "thin" => Mode::Thin,
                    "replica" => Mode::Replica,
                    m => return Err(err(vec![format!("bad mode '{m}' (thin, replica)")])),
                };
                if self.resumed {
                    // the hub was restored from the log: bring in what the host edited since (if anything)
                    self.resumed = false;
                    self.hub.submit_doc(LOCAL_ID, ed.doc(), &mut |_| Ok(()), now).map_err(submit_err)?;
                } else if !self.active() {
                    // nobody is in: start the session from the editor's current document
                    self.hub = Hub::new(ed.doc());
                    if let Some(l) = self.log.as_mut() {
                        l.origin = collablog::digest(&self.hub.doc_json());
                    }
                    self.log_checkpoint();
                }
                let info = self.hub.join(name, mode, now).map_err(|e| err(vec![e]))?;
                let secret = osatura_core::fsutil::random_hex(16);
                self.secrets.retain(|id, _| self.hub.has_user(*id));
                self.secrets.insert(info.replica, secret.clone());
                let mut out = json!({"ok": true, "replica": info.replica, "secret": secret, "colour": info.colour, "name": info.name, "index": info.index, "mode": info.mode,
                    "clock": self.hub.clock(), "users": self.hub.users_view(), "presence": self.hub.presence.all()});
                if mode == Mode::Replica {
                    out["snapshot"] = serde_json::to_value(self.hub.snapshot()).unwrap_or(Value::Null);
                }
                out["doc"] = ed.doc().clone();
                Ok(out)
            }
            (Method::Post, "/api/collab/leave") => {
                let v = parse(body)?;
                let id = replica_of(&v)?;
                self.require_secret(id, ctx)?;
                self.secrets.remove(&id);
                Ok(json!({"ok": true, "left": self.hub.leave(id)}))
            }
            (Method::Get, "/api/collab/state") => {
                let who: Option<u32> = qp(query, "replica").and_then(|x| x.parse().ok());
                // asking as a participant counts as activity for it, but only with its secret
                let who = who.filter(|w| self.require_secret(*w, ctx).is_ok());
                if let Some(w) = who {
                    self.hub.touch(w, now);
                }
                let mut v = self.with_info(state(ed), who);
                v["collab_active"] = json!(self.active());
                if qp(query, "doc") == Some("1") {
                    v["doc"] = ed.doc().clone();
                }
                if qp(query, "snapshot") == Some("1") {
                    v["snapshot"] = serde_json::to_value(self.hub.snapshot()).unwrap_or(Value::Null);
                }
                Ok(v)
            }
            (Method::Get, "/api/collab/log") => {
                let since: usize = qp(query, "since").and_then(|x| x.parse().ok()).unwrap_or(0);
                let items = self.hub.log_since(since).map_err(|e| err(vec![e]))?;
                let items: Vec<Value> = items.into_iter().map(|(i, e)| json!({"index": i + 1, "by": e.replica, "envelope": e})).collect();
                Ok(json!({"ok": true, "index": self.hub.index(), "entries": items}))
            }
            (Method::Post, "/api/collab/ops") => {
                let v = parse(body)?;
                let id = replica_of(&v)?;
                if !self.hub.has_user(id) {
                    return Err(err(vec![format!("unknown participant {id}: join first")]));
                }
                self.require_secret(id, ctx)?;
                let sub = if v.get("envelopes").is_some() {
                    let envs: Vec<Envelope> = serde_json::from_value(v["envelopes"].clone()).map_err(|e| err(vec![format!("bad envelopes: {e}")]))?;
                    self.hub.submit_envelopes(id, envs, &mut |j| ed.replace_doc(j.clone()), now)
                } else {
                    let ops: Vec<Op> = if v.get("ops").is_some() {
                        serde_json::from_value(v["ops"].clone()).map_err(|e| err(vec![format!("bad ops: {e}")]))?
                    } else {
                        vec![serde_json::from_value(v["op"].clone()).map_err(|e| err(vec![format!("bad op: {e}")]))?]
                    };
                    if ops.is_empty() {
                        return Err(err(vec!["no ops".into()]));
                    }
                    let base = match &v["base"] {
                        Value::Null => None,
                        b => Some(b.as_u64().ok_or_else(|| err(vec!["bad 'base'".into()]))? as usize),
                    };
                    self.hub.submit_ops(id, base, &ops, &mut |j| ed.replace_doc(j.clone()), now)
                };
                let s = sub.map_err(submit_err)?;
                self.flush();
                let mut out = state(ed);
                out["applied"] = json!(s.envelope.is_some());
                out["index"] = json!(s.index);
                Ok(out)
            }
            (Method::Post, "/api/collab/undo") | (Method::Post, "/api/collab/redo") => {
                let v = parse(body)?;
                let id = replica_of(&v)?;
                self.require_secret(id, ctx)?;
                let c = Ctx { user: Some(id), base: None, secret: ctx.secret.clone() };
                let mut out = self.undo_redo(ed, &c, path.ends_with("undo"))?;
                out["collab_can_undo"] = json!(self.hub.can_undo(id));
                out["collab_can_redo"] = json!(self.hub.can_redo(id));
                Ok(out)
            }
            (Method::Post, "/api/collab/presence") => {
                let v = parse(body)?;
                let id = replica_of(&v)?;
                self.require_secret(id, ctx)?;
                let mut p = Presence::new(id, "");
                p.n = v["n"].as_u64().unwrap_or(0);
                p.frame = v["frame"].as_f64().map(|x| x as f32);
                if let Some(c) = v["cursor"].as_array().filter(|c| c.len() == 2) {
                    if let (Some(x), Some(y)) = (c[0].as_f64(), c[1].as_f64()) {
                        p.cursor = Some([x as f32, y as f32]);
                    }
                }
                if let Some(a) = v["selection"].as_array() {
                    p.selection = a.iter().filter_map(|s| s.as_str().map(String::from)).take(64).collect();
                }
                p.bounds = self.bounds_for(ed, id, &p.selection, p.frame);
                self.hub.set_presence(id, p, now).map_err(|e| err(vec![e]))?;
                self.flush();
                Ok(json!({"ok": true}))
            }
            (Method::Post, "/api/collab/revoke") => {
                // `{"names": ["ana", "bob"]}`; the caller was checked by the studio (`Collab::can_administer`)
                let v = parse(body)?;
                let names: Vec<String> = v["names"].as_array().map(|a| a.iter().filter_map(|n| n.as_str().map(String::from)).collect()).unwrap_or_default();
                let added = self.revoke_names(&names).map_err(|e| err(vec![e]))?;
                Ok(json!({"ok": true, "revoked": added, "revoked_total": self.revoked.len()}))
            }
            (Method::Post, "/api/collab/compact") => {
                let mut out = self.hub.compact();
                self.log_checkpoint();
                out["logged"] = json!(self.logging());
                out["ok"] = json!(true);
                Ok(out)
            }
            _ => Err(err(vec!["unknown collaboration route".into()])),
        })();
        self.flush();
        Some(r)
    }

    /// Open a server-sent-events stream: the receiver yields the frames to write (replays the log from `since`, then
    /// users, presence and every later event). At most [`MAX_STREAMS`] streams at a time.
    pub fn connect_sse(&mut self, query: &str, last_event_id: Option<&str>) -> Result<mpsc::Receiver<String>, Resp> {
        // streams whose client went away are noticed by a failed send
        self.subs.retain(|tx| tx.send(": ping\n\n".to_string()).is_ok());
        if self.subs.len() >= MAX_STREAMS {
            return Err(Resp::from_string("too many event streams").with_status_code(503));
        }
        let since = qp(query, "since").or(last_event_id).and_then(|x| x.parse::<usize>().ok()).unwrap_or_else(|| self.hub.index());
        let (tx, rx) = mpsc::channel::<String>();
        match self.hub.catch_up(since) {
            Ok(evs) => {
                for e in evs {
                    let _ = tx.send(e.to_sse());
                }
            }
            Err(_) => {
                let _ = tx.send("event: resync\ndata: {}\n\n".into());
            }
        }
        self.subs.push(tx);
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use osatura_core::demo;

    fn editor() -> Editor {
        Editor::new(demo::project()).unwrap()
    }

    fn call(c: &mut Collab, ed: &mut Editor, m: Method, p: &str, q: &str, b: Value) -> Result<Value, Value> {
        // calls made as a participant carry its secret, like the page does
        let secret = b["replica"].as_u64().and_then(|r| c.secrets.get(&(r as u32)).cloned());
        c.route(ed, &m, p, q, &b.to_string(), &Ctx { secret, ..Ctx::default() }).expect("collab route")
    }

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    fn secret_collab() -> Collab {
        let mut c = Collab::new(&json!({}), None).with_secret(SECRET, vec![]).unwrap();
        c.unix_now = Some(1_000_000);
        c
    }

    /// Wave 7 (L-18): per-participant tokens replace the shared bearer.
    #[test]
    fn participant_tokens_are_checked_for_signature_expiry_and_revocation() {
        let ana = osatura_core::collabauth::mint_token(SECRET, "ana", 1_000_000, 600).unwrap();
        let mut c = secret_collab();
        assert!(c.auth_required());
        assert_eq!(c.check_auth(None, None, "a"), Err(401), "a credential is required");
        assert_eq!(c.check_auth(Some(&ana), None, "a"), Ok(()));
        assert_eq!(c.authed.as_deref(), Some("ana"));
        // via the cookie, as the login page sets it
        assert_eq!(c.check_auth(None, Some(&format!("x=1; osatura_token={ana}")), "a"), Ok(()));
        // the secret itself and a token from another secret are no credentials
        assert_eq!(c.check_auth(Some(SECRET), None, "a"), Err(401));
        let other = osatura_core::collabauth::mint_token("ffffffffffffffffffffffffffffffff", "ana", 1_000_000, 600).unwrap();
        assert_eq!(c.check_auth(Some(&other), None, "a"), Err(401));
        assert_eq!(c.authed, None);
        // expiry: refused with a note, and a stale token does not feed the guessing back-off
        c.unix_now = Some(1_000_601);
        for _ in 0..10 {
            assert_eq!(c.check_auth(Some(&ana), None, "stale"), Err(401));
        }
        assert!(c.auth_note().unwrap().contains("expired"), "{:?}", c.auth_note());
        c.unix_now = Some(1_000_600);
        assert_eq!(c.check_auth(Some(&ana), None, "stale"), Ok(()), "never locked out by its own expired attempts");
        // revocation by name
        let mut r = Collab::new(&json!({}), None).with_secret(SECRET, vec!["ana".into()]).unwrap();
        r.unix_now = Some(1_000_000);
        assert_eq!(r.check_auth(Some(&ana), None, "a"), Err(401));
        assert!(r.auth_note().unwrap().contains("revoked"));
        let bo = osatura_core::collabauth::mint_token(SECRET, "bo", 1_000_000, 600).unwrap();
        assert_eq!(r.check_auth(Some(&bo), None, "a"), Ok(()));
        // guessing still backs off
        for _ in 0..5 {
            let _ = r.check_auth(Some("ap1.ana.9999999999.0000000000000000000000000000000000000000000000000000000000000000"), None, "g");
        }
        assert_eq!(r.check_auth(Some("ap1.x.1.2"), None, "g"), Err(429));
        assert!(Collab::new(&json!({}), None).with_secret("short", vec![]).is_err());
        assert!(Collab::new(&json!({}), None).with_secret(SECRET, vec!["bad name".into()]).is_err());
    }

    #[test]
    fn shared_token_and_participant_tokens_can_be_combined() {
        let mut c = Collab::new(&json!({}), Some("s3cret-s3cret-s3cret".into())).with_secret(SECRET, vec![]).unwrap();
        c.unix_now = Some(5);
        let t = osatura_core::collabauth::mint_token(SECRET, "cy", 5, 60).unwrap();
        assert_eq!(c.check_auth(Some("s3cret-s3cret-s3cret"), None, "a"), Ok(()));
        assert_eq!(c.authed, None, "the shared token names nobody");
        assert_eq!(c.check_auth(Some(&t), None, "a"), Ok(()));
        assert_eq!(c.authed.as_deref(), Some("cy"));
        assert_eq!(c.check_auth(Some("nope"), None, "a"), Err(401));
    }

    /// `--collab-host`: saving, compacting and revoking are for the named tokens only; editing is for everybody.
    #[test]
    fn only_the_named_hosts_may_save_compact_and_revoke() {
        let tok = |n: &str| osatura_core::collabauth::mint_token(SECRET, n, 1_000_000, 600).unwrap();
        let mut c = secret_collab().with_hosts(vec!["alexio".into()]).unwrap();
        c.unix_now = Some(1_000_000);
        assert_eq!(c.check_auth(Some(&tok("alexio")), None, "a"), Ok(()));
        assert!(c.is_host() && c.can_administer());
        assert_eq!(c.check_auth(Some(&tok("ana")), None, "a"), Ok(()), "a participant may still edit");
        assert!(!c.is_host() && !c.can_administer(), "but not save, compact or revoke");

        // without --collab-host nothing changes for save/compact, but revoking over HTTP stays closed to token holders
        let mut open = secret_collab();
        assert_eq!(open.check_auth(Some(&tok("ana")), None, "a"), Ok(()));
        assert!(open.is_host(), "not restricted by default");
        assert!(!open.can_administer(), "revoking needs a named host");

        // the shared token and a server with no credential have no names: they are not restricted
        let mut both = Collab::new(&json!({}), Some("s3cret-s3cret-s3cret".into())).with_secret(SECRET, vec![]).unwrap().with_hosts(vec!["alexio".into()]).unwrap();
        both.unix_now = Some(1_000_000);
        assert_eq!(both.check_auth(Some("s3cret-s3cret-s3cret"), None, "a"), Ok(()));
        assert!(both.is_host() && both.can_administer(), "the shared token is the admin key");
        let plain = Collab::new(&json!({}), None);
        assert!(plain.is_host() && plain.can_administer());

        // hosts need the secret, at least one valid name
        assert!(Collab::new(&json!({}), None).with_hosts(vec!["alexio".into()]).is_err());
        assert!(secret_collab().with_hosts(vec![]).is_err());
        assert!(secret_collab().with_hosts(vec!["a b".into()]).is_err());
    }

    #[test]
    fn revoking_a_name_refuses_its_token_at_once_and_removes_the_participant() {
        let ana = osatura_core::collabauth::mint_token(SECRET, "ana", 1_000_000, 600).unwrap();
        let bob = osatura_core::collabauth::mint_token(SECRET, "bob", 1_000_000, 600).unwrap();
        let mut c = secret_collab();
        let mut ed = editor();
        assert_eq!(c.check_auth(Some(&ana), None, "a"), Ok(()));
        let j = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"mode": "thin"})).unwrap();
        assert_eq!(j["name"], "ana", "joined under the token's name");
        assert!(c.hub.users_view().iter().any(|u| u["name"] == "ana"));

        let added = c.revoke_names(&["ana".into(), "ana".into(), "cy".into()]).unwrap();
        assert_eq!(added, vec!["ana".to_string(), "cy".to_string()], "each name once");
        assert!(!c.hub.users_view().iter().any(|u| u["name"] == "ana"), "the revoked participant is gone");
        assert_eq!(c.check_auth(Some(&ana), None, "a"), Err(401), "the very next request is refused");
        assert!(c.auth_note().unwrap().contains("revoked"));
        assert_eq!(c.check_auth(Some(&bob), None, "a"), Ok(()), "other tokens still work");
        // nothing to revoke without per-participant tokens; bad arguments are refused
        assert!(Collab::new(&json!({}), Some("s3cret-s3cret-s3cret".into())).revoke_names(&["ana".into()]).is_err());
        assert!(c.revoke_names(&[]).is_err());
        assert!(c.revoke_names(&["a b".into()]).is_err());
        assert!(c.revoke_names(&(0..65).map(|i| format!("n{i}")).collect::<Vec<_>>()).is_err());
        // the HTTP route does the same
        let r = call(&mut c, &mut ed, Method::Post, "/api/collab/revoke", "", json!({"names": ["bob"]})).unwrap();
        assert_eq!((r["ok"].as_bool(), r["revoked"].clone()), (Some(true), json!(["bob"])));
        assert_eq!(c.check_auth(Some(&bob), None, "a"), Err(401));
    }

    #[test]
    fn login_with_a_participant_token_sets_a_cookie_that_expires_with_it_and_join_uses_the_token_name() {
        let mut c = secret_collab();
        let t = osatura_core::collabauth::mint_token(SECRET, "ana", 1_000_000, 600).unwrap();
        let r = c.login(format!(r#"{{"token":"{t}"}}"#).as_bytes(), "a", false);
        assert_eq!(r.status, 200);
        let cookie = r.headers.iter().find(|(k, _)| k == "Set-Cookie").map(|(_, v)| v.clone()).unwrap();
        assert!(cookie.starts_with(&format!("osatura_token={t};")) && cookie.contains("HttpOnly") && cookie.contains("Max-Age=600"), "{cookie}");
        assert_eq!(c.login(br#"{"token":"wrong"}"#, "a", false).status, 401);
        c.unix_now = Some(2_000_000);
        let r = c.login(format!(r#"{{"token":"{t}"}}"#).as_bytes(), "a", false);
        assert_eq!(r.status, 401);
        assert!(String::from_utf8_lossy(&r.body).contains("expired"));
        // joining under the token's name, whatever the page asks for
        c.unix_now = Some(1_000_100);
        assert_eq!(c.check_auth(Some(&t), None, "a"), Ok(()));
        let mut ed = Editor::new(osatura_core::Project::new("p", 10.0, 10.0, 24.0, 4)).unwrap();
        let j = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "mallory"})).unwrap();
        assert_eq!(j["name"], "ana", "{j}");
    }

    #[test]
    fn requests_are_rate_limited_per_client_address() {
        let mut c = Collab::new(&json!({}), None).with_rate(10.0);
        // burst of 20, then refused
        let ok = (0..40).filter(|_| c.check_rate("10.0.0.1").is_ok()).count();
        assert!((20..=22).contains(&ok), "burst {ok}");
        assert_eq!(c.check_rate("10.0.0.1"), Err(1), "wait one second at most");
        assert!(c.check_rate("10.0.0.2").is_ok(), "another address has its own bucket");
        // off by default and with 0
        let mut d = Collab::new(&json!({}), None);
        assert!((0..1000).all(|_| d.check_rate("x").is_ok()));
        assert!((0..1000).all(|_| Collab::new(&json!({}), None).with_rate(0.0).check_rate("x").is_ok()));
        // the table of addresses is bounded
        let mut e = Collab::new(&json!({}), None).with_rate(1.0);
        for i in 0..(MAX_CLIENTS + 50) {
            let _ = e.check_rate(&format!("addr{i}"));
        }
        assert!(e.buckets.len() <= MAX_CLIENTS);
    }

    #[test]
    fn secret_files_must_be_private_and_long_enough() {
        let dir = osatura_core::fsutil::ScratchDir::new("osatura-secret");
        let f = dir.join("s");
        std::fs::write(&f, format!("{SECRET}\nsecond line ignored\n")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
            assert!(read_secret_file(f.to_str().unwrap()).unwrap_err().contains("chmod 600"));
            std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert_eq!(read_secret_file(f.to_str().unwrap()).unwrap(), SECRET);
        std::fs::write(dir.join("short"), "abc\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dir.join("short"), std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert!(read_secret_file(dir.join("short").to_str().unwrap()).unwrap_err().contains("at least 16"));
        assert!(read_secret_file(dir.join("missing").to_str().unwrap()).is_err());
    }

    #[test]
    fn token_is_required_and_compared_in_full() {
        let mut c = Collab::new(&json!({}), Some("s3cret-s3cret-s3cret".into()));
        assert_eq!(c.check_auth(None, None, "a"), Err(401));
        assert_eq!(c.check_auth(Some("s3cret-s3cret-s3cre"), None, "a"), Err(401), "a prefix is not the token");
        assert_eq!(c.check_auth(Some("s3cret-s3cret-s3cretx"), Some("a=b"), "a"), Err(401));
        assert_eq!(c.check_auth(Some("s3cret-s3cret-s3cret"), None, "a"), Ok(()));
        assert_eq!(c.check_auth(None, Some("x=1; osatura_token=s3cret-s3cret-s3cret; y=2"), "a"), Ok(()));
        assert_eq!(c.check_auth(None, Some("osatura_token=nope"), "a"), Err(401));
        assert_eq!(Collab::new(&json!({}), None).check_auth(None, None, "a"), Ok(()));
    }

    #[test]
    fn short_or_odd_tokens_are_refused() {
        assert!(check_token("short").unwrap_err().contains("at least 16"));
        assert!(check_token("fifteen-chars-x").is_err());
        assert!(check_token("sixteen-chars-ok!").unwrap_err().contains("only contain"));
        assert!(check_token("0123456789abcdef").is_ok());
    }

    #[test]
    fn the_token_is_never_read_from_the_url_and_login_sets_an_http_only_cookie() {
        let mut c = Collab::new(&json!({}), Some("0123456789abcdef".into()));
        // the old `?token=` way is not an accepted credential any more (check_auth does not even see the query)
        assert_eq!(c.check_auth(None, None, "a"), Err(401));
        let r = c.login(br#"{"token":"0123456789abcdef"}"#, "a", false);
        assert_eq!(r.status, 200);
        let cookie = r.headers.iter().find(|(k, _)| k == "Set-Cookie").map(|(_, v)| v.clone()).unwrap();
        assert_eq!(cookie, "osatura_token=0123456789abcdef; Path=/; HttpOnly; SameSite=Strict");
        // over HTTPS (a TLS proxy in front, --public-origin https://...) the cookie is Secure as well
        let r = c.login(br#"{"token":"0123456789abcdef"}"#, "a", true);
        let cookie = r.headers.iter().find(|(k, _)| k == "Set-Cookie").map(|(_, v)| v.clone()).unwrap();
        assert_eq!(cookie, "osatura_token=0123456789abcdef; Path=/; HttpOnly; SameSite=Strict; Secure");
        assert_eq!(c.login(br#"{"token":"wrong"}"#, "a", false).status, 401);
        assert_eq!(c.login(b"not json", "a", false).status, 401);
    }

    #[test]
    fn repeated_wrong_tokens_are_slowed_down_per_client_and_the_right_token_always_works() {
        const T: &str = "0123456789abcdef";
        let mut c = Collab::new(&json!({}), Some(T.into()));
        for _ in 0..4 {
            assert_eq!(c.check_auth(Some("guess"), None, "10.0.0.1"), Err(401));
        }
        // the fifth failure of that address starts a back-off: more wrong credentials get 429
        assert_eq!(c.check_auth(Some("guess"), None, "10.0.0.1"), Err(401));
        assert_eq!(c.check_auth(Some("guess"), None, "10.0.0.1"), Err(429));
        assert_eq!(c.login(br#"{"token":"wrong"}"#, "10.0.0.1", false).status, 429);
        // but the right token is accepted at once, from the locked address too, and from every other address
        assert_eq!(c.check_auth(Some(T), None, "10.0.0.2"), Ok(()));
        assert_eq!(c.login(format!(r#"{{"token":"{T}"}}"#).as_bytes(), "10.0.0.1", false).status, 200);
        assert_eq!(c.check_auth(Some(T), None, "10.0.0.1"), Ok(()), "old behaviour: 429 for 30 s");
        // success cleared the address
        assert_eq!(c.check_auth(Some("guess"), None, "10.0.0.1"), Err(401));
        // a wrong COOKIE counts as a failed attempt as well (it used to be a free way to guess)
        let mut d = Collab::new(&json!({}), Some(T.into()));
        for _ in 0..5 {
            assert_eq!(d.check_auth(None, Some("osatura_token=old"), "10.0.0.3"), Err(401));
        }
        assert_eq!(d.check_auth(None, Some("osatura_token=old"), "10.0.0.3"), Err(429));
        assert_eq!(d.check_auth(None, Some("osatura_token=old"), "10.0.0.4"), Err(401), "other addresses are not locked");
        // no credential at all (the first page load) is no attempt
        let mut e = Collab::new(&json!({}), Some(T.into()));
        for _ in 0..20 {
            assert_eq!(e.check_auth(None, None, "10.0.0.5"), Err(401));
        }
        assert_eq!(e.check_auth(Some(T), None, "10.0.0.5"), Ok(()));
        // the table of addresses is bounded
        for i in 0..(MAX_CLIENTS + 50) {
            let _ = e.check_auth(Some("x"), None, &format!("addr{i}"));
        }
        assert!(e.clients.len() <= MAX_CLIENTS);
    }

    /// A token that could split a Set-Cookie header (CR, LF, `;`, space, quotes) never gets that far.
    #[test]
    fn tokens_that_could_break_the_cookie_header_are_refused() {
        for bad in ["0123456789abcdef\r\nSet-Cookie: x=1", "0123456789abcdef;Path=/", "0123456789abcdef\n", "0123456789abc def", "0123456789abcdef\"", "0123456789abcdef,x"] {
            assert!(check_token(bad).is_err(), "{bad:?}");
            // belt and braces: even if such a token reached the server, login would not echo it into a header
            let mut c = Collab::new(&json!({}), Some(bad.into()));
            let r = c.login(serde_json::to_vec(&json!({"token": bad})).unwrap().as_slice(), "a", false);
            assert_eq!(r.status, 500, "{bad:?}");
            assert!(r.headers.iter().all(|(k, _)| !k.eq_ignore_ascii_case("set-cookie")));
        }
    }

    #[test]
    fn a_participant_id_alone_cannot_act_as_that_participant() {
        let mut ed = editor();
        let mut c = Collab::new(ed.doc(), None);
        let a = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "ana"})).unwrap();
        let (ia, secret) = (a["replica"].as_u64().unwrap(), a["secret"].as_str().unwrap().to_string());
        assert_eq!(secret.len(), 32, "128 random bits as hex");
        let b = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "bo"})).unwrap();
        assert_ne!(b["secret"], a["secret"]);
        // an attacker who knows only the id (it is public in the users list)
        let none = Ctx::default();
        let wrong = Ctx { secret: Some("00000000000000000000000000000000".into()), ..Ctx::default() };
        for ctx in [&none, &wrong] {
            for (p, body) in [
                ("/api/collab/ops", json!({"replica": ia, "ops": [{"op": "set", "path": "/fps", "value": 31}]})),
                ("/api/collab/undo", json!({"replica": ia})),
                ("/api/collab/presence", json!({"replica": ia, "n": 1})),
                ("/api/collab/leave", json!({"replica": ia})),
            ] {
                let r = c.route(&mut ed, &Method::Post, p, "", &body.to_string(), ctx).unwrap();
                assert!(r.is_err() && r.unwrap_err()["errors"][0].as_str().unwrap().contains("secret"), "{p}");
            }
        }
        assert!(c.hub.has_user(ia as u32), "still joined");
        assert_ne!(ed.project().fps, 31.0);
        // the page-level check used for /api/op and friends
        assert!(!c.verify_ctx(&Ctx { user: Some(ia as u32), ..Ctx::default() }));
        assert!(!c.verify_ctx(&Ctx { user: Some(ia as u32), secret: Some("x".into()), base: None }));
        assert!(c.verify_ctx(&Ctx { user: Some(ia as u32), secret: Some(secret.clone()), base: None }));
        assert!(c.verify_ctx(&Ctx::default()), "no participant named: the host");
        // and with the secret it works
        let ok = c.route(&mut ed, &Method::Post, "/api/collab/leave", "", &json!({"replica": ia}).to_string(), &Ctx { secret: Some(secret), ..Ctx::default() }).unwrap();
        assert_eq!(ok.unwrap()["left"], json!(true));
    }

    #[test]
    fn event_streams_are_capped_and_dead_ones_are_dropped() {
        let ed = editor();
        let mut c = Collab::new(ed.doc(), None);
        let live: Vec<_> = (0..MAX_STREAMS).map(|_| c.connect_sse("", None).unwrap()).collect();
        assert_eq!(c.connect_sse("", None).unwrap_err().status, 503);
        drop(live);
        assert!(c.connect_sse("", None).is_ok(), "streams whose connection ended make room");
    }

    #[test]
    fn join_edit_in_a_stale_view_leave_through_the_routes() {
        let mut ed = editor();
        let mut c = Collab::new(ed.doc(), None);
        assert!(!c.active());
        let a = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "ana"})).unwrap();
        let b = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "bo"})).unwrap();
        let (ia, ib) = (a["replica"].as_u64().unwrap(), b["replica"].as_u64().unwrap());
        assert!(c.active() && ia != ib && a["colour"].as_str().unwrap().starts_with('#'));
        let n0 = ed.project().layers.len();
        // ana inserts a copy of layer 0 at the front, bo (same base) toggles what he sees as layer 0
        let l0 = ed.doc()["layers"][0].clone();
        let mut copy = l0.clone();
        copy["name"] = json!("front_copy");
        call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!({"replica": ia, "base": 0, "ops": [{"op": "insert", "path": "/layers/0", "value": copy}]})).unwrap();
        assert_eq!(ed.project().layers.len(), n0 + 1);
        let name0 = l0["name"].as_str().unwrap().to_string();
        let vis = !l0["visible"].as_bool().unwrap_or(true);
        let r = call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!({"replica": ib, "base": 0, "ops": [{"op": "set", "path": "/layers/0/visible", "value": vis}]})).unwrap();
        assert_eq!(r["applied"], json!(true));
        assert_eq!(ed.doc()["layers"][0]["name"], "front_copy");
        assert_eq!(ed.doc()["layers"][1]["name"], json!(name0));
        assert_eq!(ed.doc()["layers"][1]["visible"], json!(vis), "followed the element, not the index");
        // an op that would make the project invalid is refused whole
        let bad = call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!({"replica": ib, "ops": [{"op": "set", "path": "/frames", "value": -5}]}));
        assert!(bad.is_err());
        assert!(call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!({"replica": 999, "ops": [{"op": "set", "path": "/frames", "value": 5}]})).is_err());
        assert!(call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!({"replica": ib, "ops": []})).is_err());
        // per-user undo: ana undoes her insert only
        let u = call(&mut c, &mut ed, Method::Post, "/api/collab/undo", "", json!({"replica": ia})).unwrap();
        assert_eq!(u["collab_can_redo"], json!(true));
        assert_eq!(ed.project().layers.len(), n0);
        assert_eq!(ed.doc()["layers"][0]["visible"], json!(vis), "bo's change survived ana's undo");
        // state, log, presence, compaction
        let s = call(&mut c, &mut ed, Method::Get, "/api/collab/state", &format!("replica={ia}&doc=1&snapshot=1"), json!(null)).unwrap();
        assert!(s["doc"].is_object() && s["snapshot"].is_object() && s["collab_users"].as_array().unwrap().len() == 2);
        let l = call(&mut c, &mut ed, Method::Get, "/api/collab/log", "since=1", json!(null)).unwrap();
        assert_eq!(l["entries"].as_array().unwrap().len() as u64, l["index"].as_u64().unwrap() - 1);
        call(&mut c, &mut ed, Method::Post, "/api/collab/presence", "", json!({"replica": ia, "n": 1, "frame": 3, "cursor": [10, 20], "selection": ["arm"]})).unwrap();
        assert_eq!(c.hub.presence.get(ia as u32).unwrap().cursor, Some([10.0, 20.0]));
        assert!(call(&mut c, &mut ed, Method::Post, "/api/collab/presence", "", json!({"replica": 4242})).is_err());
        let k = call(&mut c, &mut ed, Method::Post, "/api/collab/compact", "", json!({})).unwrap();
        assert_eq!(k["ok"], json!(true));
        assert!(call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"mode": "weird"})).is_err());
        assert!(call(&mut c, &mut ed, Method::Get, "/api/collab/nothing", "", json!(null)).is_err());
        assert!(call(&mut c, &mut ed, Method::Post, "/api/collab/leave", "", json!({"replica": ia})).unwrap()["left"] == json!(true));
        assert!(c.route(&mut ed, &Method::Get, "/api/state", "", "", &Ctx::default()).is_none());
        assert!(call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!("x")).is_err());
    }

    #[test]
    fn editor_side_edits_are_recorded_for_everybody() {
        let mut ed = editor();
        let mut c = Collab::new(ed.doc(), None);
        call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "ana"})).unwrap();
        let i0 = c.hub.index();
        ed.apply(Op::Set { path: "/fps".into(), value: json!(30) }).unwrap();
        c.sync_editor(&ed, &Ctx::default());
        assert_eq!(c.hub.index(), i0 + 1);
        assert_eq!(c.hub.doc_json()["fps"], json!(30));
        c.sync_editor(&ed, &Ctx::default());
        assert_eq!(c.hub.index(), i0 + 1, "nothing changed, nothing recorded");
    }

    fn tmp_log(tag: &str) -> (osatura_core::fsutil::ScratchDir, String) {
        let d = osatura_core::fsutil::ScratchDir::new(&format!("osatura-collab-{tag}"));
        let p = d.join("collab.log").to_string_lossy().into_owned();
        (d, p)
    }

    #[cfg(unix)]
    #[test]
    fn the_log_file_is_readable_by_its_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let (_scratch, path) = tmp_log("mode");
        let mut ed = editor();
        let mut c = Collab::new(ed.doc(), None);
        c.open_log(&mut ed, &path, true).unwrap();
        let a = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "ana"})).unwrap()["replica"].as_u64().unwrap();
        call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!({"replica": a, "ops": [{"op": "set", "path": "/fps", "value": 31}]})).unwrap();
        call(&mut c, &mut ed, Method::Post, "/api/collab/compact", "", json!({})).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "the log holds the whole project");
        let dir = std::path::Path::new(&path).parent().unwrap();
        let leftovers = std::fs::read_dir(dir).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().contains(".tmp-")).count();
        assert_eq!(leftovers, 0, "no temporary file stays behind");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_session_survives_a_restart_with_the_same_state_clock_and_index() {
        let (_scratch, path) = tmp_log("restart");
        let mut ed = editor();
        let start = ed.doc().clone();
        let mut c = Collab::new(ed.doc(), None);
        assert!(c.open_log(&mut ed, &path, true).unwrap().contains("new"));
        let a = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "ana"})).unwrap()["replica"].as_u64().unwrap();
        let b = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "bo"})).unwrap()["replica"].as_u64().unwrap();
        call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!({"replica": a, "ops": [{"op": "set", "path": "/fps", "value": 31}]})).unwrap();
        call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!({"replica": b, "ops": [{"op": "set", "path": "/layers/0/visible", "value": false}]})).unwrap();
        call(&mut c, &mut ed, Method::Post, "/api/collab/undo", "", json!({"replica": b})).unwrap();
        ed.apply(Op::Set { path: "/fps".into(), value: json!(32) }).unwrap();
        c.sync_editor(&ed, &Ctx::default());
        let (doc, clock, lamport, index) = (ed.doc().clone(), c.hub.clock().clone(), c.hub.lamport(), c.hub.index());
        assert_eq!(index, 4);
        drop(c);
        // restart: the studio loads the project file as it was at the start (or as last saved)
        let mut ed2 = Editor::new(demo::project()).unwrap();
        assert_eq!(ed2.doc(), &start);
        let mut c2 = Collab::new(ed2.doc(), None);
        let msg = c2.open_log(&mut ed2, &path, true).unwrap();
        assert!(msg.contains("resumed at index 4"), "{msg}");
        assert_eq!(ed2.doc(), &doc, "the editor took over the logged document");
        assert_eq!((c2.hub.clock().clone(), c2.hub.lamport(), c2.hub.index()), (clock, lamport, index));
        assert!(!c2.active());
        // the host edited while nobody was in: the join brings it into the log
        ed2.apply(Op::Set { path: "/fps".into(), value: json!(33) }).unwrap();
        let j = call(&mut c2, &mut ed2, Method::Post, "/api/collab/join", "", json!({"name": "cy"})).unwrap();
        let cy = j["replica"].as_u64().unwrap();
        assert!(cy > b, "ids are not reused: {cy} vs {b}");
        assert_eq!(c2.hub.index(), index + 1);
        call(&mut c2, &mut ed2, Method::Post, "/api/collab/ops", "", json!({"replica": cy, "ops": [{"op": "set", "path": "/frames", "value": 40}]})).unwrap();
        drop(c2);
        // a second restart sees everything, including what was appended after the first
        let mut ed3 = Editor::new(demo::project()).unwrap();
        let mut c3 = Collab::new(ed3.doc(), None);
        c3.open_log(&mut ed3, &path, true).unwrap();
        assert_eq!((c3.hub.index(), ed3.doc()["fps"].clone(), ed3.doc()["frames"].clone()), (index + 2, json!(33), json!(40)));
        // compaction rewrites the file as one checkpoint; it still resumes
        let j = call(&mut c3, &mut ed3, Method::Post, "/api/collab/join", "", json!({"name": "di"})).unwrap();
        assert!(j["ok"] == json!(true));
        let k = call(&mut c3, &mut ed3, Method::Post, "/api/collab/compact", "", json!({})).unwrap();
        assert_eq!(k["logged"], json!(true));
        assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 1);
        let (idx, d3) = (c3.hub.index(), ed3.doc().clone());
        drop(c3);
        let mut ed4 = Editor::new(demo::project()).unwrap();
        let mut c4 = Collab::new(ed4.doc(), None);
        c4.open_log(&mut ed4, &path, false).unwrap();
        assert_eq!((c4.hub.index(), ed4.doc().clone()), (idx, d3));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_damaged_or_foreign_log_stops_the_start_and_is_left_untouched() {
        let (_scratch, path) = tmp_log("bad");
        let mut ed = editor();
        let mut c = Collab::new(ed.doc(), None);
        c.open_log(&mut ed, &path, true).unwrap();
        let a = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "ana"})).unwrap()["replica"].as_u64().unwrap();
        for v in [31, 32, 33] {
            call(&mut c, &mut ed, Method::Post, "/api/collab/ops", "", json!({"replica": a, "ops": [{"op": "set", "path": "/fps", "value": v}]})).unwrap();
        }
        drop(c);
        let good = std::fs::read_to_string(&path).unwrap();
        assert_eq!(good.lines().count(), 4);
        // another project: its document is neither the origin nor the last state
        let mut other = editor();
        other.apply(Op::Set { path: "/width".into(), value: json!(123) }).unwrap();
        let mut c = Collab::new(other.doc(), None);
        let e = c.open_log(&mut other, &path, true).unwrap_err();
        assert!(e.contains("different project") && e.contains("untouched"), "{e}");
        // damage in the middle
        let broken = good.replacen("\"fps\"", "\"fpz\"", 1);
        assert_ne!(broken, good);
        std::fs::write(&path, &broken).unwrap();
        let mut ed = editor();
        let mut c = Collab::new(ed.doc(), None);
        let e = c.open_log(&mut ed, &path, true).unwrap_err();
        assert!(e.contains("damaged at line") && e.contains("checksum"), "{e}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), broken, "never rewritten");
        // not a log at all
        std::fs::write(&path, "just some text\n").unwrap();
        assert!(c.open_log(&mut ed, &path, true).unwrap_err().contains("line 1"));
        // a cut-off last line is dropped and the file is repaired for appending
        std::fs::write(&path, &good[..good.len() - 15]).unwrap();
        let mut ed = editor();
        let mut c = Collab::new(ed.doc(), None);
        assert!(c.open_log(&mut ed, &path, true).unwrap().contains("cut-off"));
        assert_eq!(c.hub.index(), 2);
        assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 3);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn selection_bounds_are_computed_by_the_server_and_follow_the_layers() {
        let mut ed = editor();
        let mut c = Collab::new(ed.doc(), None);
        let a = call(&mut c, &mut ed, Method::Post, "/api/collab/join", "", json!({"name": "ana"})).unwrap()["replica"].as_u64().unwrap();
        let name = "ground".to_string();
        // the client tries to claim its own bounds: ignored
        call(&mut c, &mut ed, Method::Post, "/api/collab/presence", "", json!({"replica": a, "n": 1, "frame": 0, "selection": [name], "bounds": [1, 2, 3, 4]})).unwrap();
        let b0 = c.hub.presence.get(a as u32).unwrap().bounds.expect("bounds of a vector layer");
        assert!(b0[2] > b0[0] && b0[3] > b0[1] && b0 != [1.0, 2.0, 3.0, 4.0], "{b0:?}");
        let direct = pick::layers_bounds(ed.engine(), 0.0, std::slice::from_ref(&name)).unwrap();
        assert_eq!(b0, direct);
        // nothing selected, or only unknown names: no box
        call(&mut c, &mut ed, Method::Post, "/api/collab/presence", "", json!({"replica": a, "n": 2, "selection": ["no/such/layer"]})).unwrap();
        assert_eq!(c.hub.presence.get(a as u32).unwrap().bounds, None);
        call(&mut c, &mut ed, Method::Post, "/api/collab/presence", "", json!({"replica": a, "n": 3, "frame": 0, "selection": [name]})).unwrap();
        c.hub.take_events();
        // somebody moves the layer: the box follows without the owner sending anything
        let idx = ed.project().layers.iter().position(|l| l.name == name).unwrap();
        ed.apply(Op::Set { path: format!("/layers/{idx}/transform/position"), value: json!([100.0, 0.0]) }).unwrap();
        c.refresh_bounds(&ed);
        let b1 = c.hub.presence.get(a as u32).unwrap().bounds.unwrap();
        assert!((b1[0] - b0[0] - 100.0).abs() < 0.5 && (b1[2] - b0[2] - 100.0).abs() < 0.5, "{b0:?} -> {b1:?}");
        assert_eq!(b1, pick::layers_bounds(ed.engine(), 0.0, std::slice::from_ref(&name)).unwrap());
        // unchanged project: nothing new is published
        c.refresh_bounds(&ed);
        assert!(c.hub.take_events().iter().filter(|e| matches!(e, Event::Presence(_))).count() <= 1);
        assert!(pick::layers_bounds(ed.engine(), f32::NAN, &[name]).is_none());
    }
}
