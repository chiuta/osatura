// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Web studio: a small local HTTP server with a browser UI on top of `Editor`.

use osatura_core::anim::Ease;
use osatura_core::edit::{Editor, Op};
use osatura_core::export;
use osatura_core::gizmo::{self, DragRequest};
use osatura_core::math::Vec2;
use osatura_core::pick::{self, MarqueeMode, PickOptions, Region, SelectMode, Selection, Target};
use osatura_core::faceparam::{self, Calibration, FaceConfig, FaceTracker, Layout, Raw};
use osatura_core::liveinput::{self, LiveSession, Mapping, Recording};
use osatura_core::motionlib::Library;
use osatura_core::overlays::{self, TrailTarget};
use osatura_core::posesuggest;
use osatura_core::tinymlp::Mlp;
use osatura_core::project::Prop;
use osatura_core::engine::Engine;
use osatura_core::scrub::{self, CacheKey, FrameCache, LodProxy};
use osatura_core::timeline::{self, KeyRef, SelMode, TlOp};
use osatura_core::svg::{render_frame, RenderOptions};
use serde_json::{json, Value};
use crate::collab_srv::{Collab, Ctx};
use crate::uiconfig::UiStore;
use crate::http::{HttpReq, Limits, Method, Reply, Resp};

const PAGE: &str = include_str!("studio.html");

fn hdr(k: &str, v: &str) -> (String, String) {
    (k.to_string(), v.to_string())
}

fn json_resp(v: Value) -> Resp {
    Resp::from_string(v.to_string()).with_header(hdr("Content-Type", "application/json"))
}

pub(crate) fn state(ed: &Editor) -> Value {
    let p = ed.project();
    let mut v = json!({
        "ok": true, "can_undo": ed.can_undo(), "can_redo": ed.can_redo(),
        "rev": ed.revision(), "frames": p.frames, "fps": p.fps, "width": p.width, "height": p.height,
        "bones": p.bones.iter().map(|b| json!({"name": b.name, "rest_angle": b.rest_angle})).collect::<Vec<_>>(),
        "channels": serde_json::to_value(&p.channels).unwrap_or(Value::Null),
    });
    let ci = crate::collab_srv::COLLAB_INDEX.load(std::sync::atomic::Ordering::SeqCst);
    if ci >= 0 {
        v["collab_index"] = json!(ci);
    }
    v
}


fn num_field(v: &Value, k: &str) -> Result<f32, Value> {
    v[k].as_f64().filter(|x| x.is_finite()).map(|x| x as f32).ok_or_else(|| err(vec![format!("missing or non-finite '{k}'")]))
}

fn targets_of(v: &Value) -> Result<Vec<Target>, Value> {
    if v.is_null() {
        return Ok(Vec::new());
    }
    let items: Vec<Target> = serde_json::from_value(v.clone()).map_err(|e| err(vec![format!("bad targets: {e}")]))?;
    if items.len() > 10_000 {
        return Err(err(vec!["too many targets".into()]));
    }
    Ok(items)
}

fn pick_options(v: &Value) -> Result<PickOptions, Value> {
    if v["options"].is_null() {
        return Ok(PickOptions::default());
    }
    serde_json::from_value(v["options"].clone()).map_err(|e| err(vec![format!("bad options: {e}")]))
}

/// Selection plus what the page needs to draw its gizmo.
fn selection_view(ed: &Editor, frame: f32, sel: &Selection) -> Value {
    let fr = ed.engine().evaluate(frame);
    let g = gizmo::gizmo_view(ed.engine(), &fr, sel);
    let b = pick::selection_bounds(ed.engine(), &fr, sel);
    // every control point of the selected layers, so the page can draw them in point mode
    let mut all: Vec<Value> = Vec::new();
    for path in sel.layers() {
        for (kind, pos) in pick::layer_points(ed.engine(), &fr, path) {
            if all.len() < 4000 {
                all.push(json!({"layer": path, "point": kind, "pos": [pos.x, pos.y]}));
            }
        }
    }
    json!({"ok": true, "selection": sel.items, "bounds": b.map(|(lo, hi)| json!([lo.x, lo.y, hi.x, hi.y])), "gizmo": g, "points": all})
}


fn qparam<'a>(query: &'a str, k: &str) -> Option<&'a str> {
    query.split('&').find_map(|kv| kv.strip_prefix(&format!("{k}=")))
}

/// How a request wants the reduced-detail copy used: screen size of the picture in pixels and the budget.
#[derive(Clone, Copy, Debug)]
struct LodAsk {
    screen_scale: f64,
    max_px: f64,
}

/// What a picture came from, for the response headers.
struct Drawn {
    bytes: std::sync::Arc<Vec<u8>>,
    ctype: &'static str,
    hit: bool,
    ms: f64,
    /// Level of detail used (0 = the project itself) and its declared deviation in screen pixels.
    lod: u8,
    lod_px: f64,
}

/// One rendered picture through the frame cache.
/// `kind`: 0 = SVG vectors (level 0 = the full-quality picture), 1 = low-resolution bitmap inside an SVG, 2 = raw PNG.
/// With `lod` and a proxy `level` > 0 the picture is drawn from the reduced-detail copy `LodProxy` picks for the screen.
// internal cache helper threading its request context; kept flat.
#[allow(clippy::too_many_arguments)]
fn cached_frame(ed: &Editor, cache: &mut FrameCache, lodc: &mut LodProxy, frame: f32, level: u8, kind: u8, bones: bool, lod: Option<LodAsk>) -> Result<Drawn, Value> {
    scrub::check_level(level).map_err(|e| err(vec![e]))?;
    if !frame.is_finite() || frame < 0.0 || frame > ed.project().frames.saturating_sub(1) as f32 {
        return Err(err(vec![format!("frame {frame} out of range")]));
    }
    let ctype = if kind == 2 { "image/png" } else { "image/svg+xml" };
    let t0 = std::time::Instant::now();
    // the level of detail is part of what the picture is, so it is decided before the cache is asked
    let (lk, lpx) = match lod.filter(|_| level > 0) {
        Some(a) => {
            // a bitmap proxy is drawn with `proxy_scale(level)` pixels per unit: that, not the screen, limits what can be seen
            let sc = if kind == 0 { a.screen_scale } else { a.screen_scale.min(scrub::proxy_scale(level) as f64) };
            lodc.pick(ed.project(), ed.revision(), sc, a.max_px)
        }
        None => (0, 0.0),
    };
    let key = CacheKey::new(frame, level, bones, kind).with_lod(lk);
    if let Some(b) = cache.get(ed.revision(), key) {
        return Ok(Drawn { bytes: b, ctype, hit: true, ms: t0.elapsed().as_secs_f64() * 1000.0, lod: lk, lod_px: lpx });
    }
    let t1 = std::time::Instant::now();
    let e: &Engine = if lk > 0 { lodc.engine(lk, ed.asset_dir()).map_err(|x| err(vec![x]))? } else { ed.engine() };
    let bytes: Vec<u8> = match (kind, level) {
        (0, 0) => render_frame(e, frame, &RenderOptions { show_bones: bones }).into_bytes(),
        (0, l) => scrub::proxy_svg_vector(e, frame, l, bones).map_err(|x| err(vec![x]))?.into_bytes(),
        (1, l) => scrub::proxy_svg(e, frame, l, bones).map_err(|x| err(vec![x]))?.into_bytes(),
        (2, l) => scrub::render_proxy_png(e, frame, l, 1.0, bones).map_err(|x| err(vec![x]))?,
        _ => return Err(err(vec!["bad kind".into()])),
    };
    let ms = t1.elapsed().as_secs_f64() * 1000.0;
    Ok(Drawn { bytes: cache.put(ed.revision(), key, bytes), ctype, hit: false, ms, lod: lk, lod_px: lpx })
}

fn overlay_view(ed: &Editor, ui: &UiStore, frame: Option<u32>, selection: &[String]) -> Result<Value, Value> {
    let st = ui.overlays();
    let mut v = json!({"ok": true, "settings": serde_json::to_value(&st).unwrap_or(Value::Null), "frames": ed.project().frames});
    if let Some(f) = frame {
        let g = overlays::ghosts_for(ed.engine(), f, &st.onion, selection).map_err(|e| err(vec![e]))?;
        v["ghosts"] = serde_json::to_value(&g).unwrap_or(Value::Null);
        v["svg"] = if st.onion.enabled { json!(overlays::onion_svg(ed.engine(), f, &st.onion, selection).map_err(|e| err(vec![e]))?) } else { Value::Null };
    }
    Ok(v)
}

fn live_status(live: &LiveSession) -> Value {
    json!({"ok": true, "recording": live.is_recording(), "recorded": live.recorded_len(), "mapping": serde_json::to_value(live.mapping()).unwrap_or(Value::Null)})
}

pub(crate) fn err(errors: Vec<String>) -> Value {
    json!({"ok": false, "errors": errors})
}

/// Webcam face pipeline state: landmarks arrive from the page's landmark provider, geometry, calibration
/// and smoothing run here (the same code as `face-bake`), the resulting events go through the live session.
struct FaceState {
    tracker: FaceTracker,
    collecting: Option<Vec<Raw>>,
    /// Own synthetic-trained detector for /api/face/detect (loaded on first use) and its box tracker.
    model: Option<osatura_core::facedet::FaceModel>,
    boxes: osatura_core::facedet::BoxTracker,
}

fn live_push_resp(live: &mut LiveSession, events: &[liveinput::InputEvent], warnings: Vec<String>) -> Result<Value, Value> {
    let fired = live.push(events).map_err(|e| err(vec![e]))?;
    let values: Vec<Value> = live.preview().into_iter().map(|(t, pr, v)| json!({"target": t, "prop": pr, "value": v})).collect();
    let fired: Vec<Value> = fired.iter().map(|f| json!({"time": f.time, "action": serde_json::to_value(&f.action).unwrap_or(Value::Null)})).collect();
    Ok(json!({"ok": true, "values": values, "fired": fired, "warnings": warnings, "recorded": live.recorded_len()}))
}

fn pct_decode(s: &str) -> String {
    let b = s.as_bytes();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => match ((b[i + 1] as char).to_digit(16), (b[i + 2] as char).to_digit(16)) {
                (Some(h), Some(l)) => {
                    out.push((h * 16 + l) as u8);
                    i += 2;
                }
                _ => out.push(b'%'),
            },
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Which requests the studio talks to at all. A page on another web site can make the visitor's browser send
/// requests to `127.0.0.1:PORT`; these rules decide which of them are answered.
///  * `Host` must be one of this server's own names (`127.0.0.1:PORT`, `localhost:PORT`, `[::1]:PORT`, the
///    `--bind` address, `--allow-host` names). A DNS-rebinding page reaches the server under its own host name
///    and is refused (421).
///  * Behind a TLS reverse proxy the page is loaded from `https://<public name>`: names given with `--allow-host`
///    and the host of `--public-origin URL` are accepted as `Host` (with or without the port) and as
///    `https://<name>` / `http://<name>` origins. Any other origin stays refused.
///  * `Origin`, when present, must be this server's own origin, and `Sec-Fetch-Site`, when present, must be
///    `same-origin` or `none`: a request made by another web page is refused (403). Command-line clients send
///    neither header.
///  * A request body must be declared `application/json` (415): the "simple" content types a web page can send
///    without asking permission first (text/plain, forms) are not accepted.
pub struct Policy {
    hosts: Vec<String>,
    /// Public names (`--allow-host`, `--public-origin`) without the server's port: accepted as `Host` and, with
    /// `https://` or `http://`, as `Origin` (a TLS proxy forwards the public name and the page's origin is https).
    public_names: Vec<String>,
    /// Hosts of `--public-origin https://...`: requests addressed to them reached the studio through TLS, so the
    /// login cookie gets the `Secure` attribute.
    https_names: Vec<String>,
    ip_port: Option<u16>,
    /// Where the corresponding source is published (`--source-url`); `None` = the default offer text (AGPL-3.0 section 13).
    source_url: Option<String>,
}

impl Policy {
    pub fn new(bind: &str, port: u16, loopback: bool, extra: &[String]) -> Policy {
        let mut hosts: Vec<String> = vec![format!("127.0.0.1:{port}"), format!("localhost:{port}"), format!("[::1]:{port}"), bind.to_ascii_lowercase()];
        let mut public_names = Vec::new();
        for h in extra {
            let h = h.trim().to_ascii_lowercase();
            if !h.is_empty() {
                public_names.push(h.clone());
                hosts.push(if h.contains(':') && !h.ends_with(']') { h } else { format!("{h}:{port}") });
            }
        }
        // on a deliberate non-local bind the server is reached by its address: any IP literal on its port is its own name
        Policy { hosts, public_names, https_names: Vec::new(), ip_port: if loopback { None } else { Some(port) }, source_url: None }
    }

    /// Add the host of a `--public-origin` URL (`https://studio.example.org[:port]`) to the public names.
    pub fn with_public_origin(mut self, origin: Option<&str>) -> Policy {
        if let Some(o) = origin {
            if let Ok(h) = public_origin_host(o) {
                if o.starts_with("https://") {
                    self.https_names.push(h.clone());
                }
                self.hosts.push(h.clone());
                self.public_names.push(h);
            }
        }
        self
    }

    /// `Origin` header value accepted? `http://` origins follow the `Host` rules; `https://` only for public names.
    fn origin_ok(&self, origin: &str) -> bool {
        let o = origin.trim().to_ascii_lowercase();
        if let Some(rest) = o.strip_prefix("http://") {
            return self.host_ok(rest) || self.public_names.iter().any(|n| n == rest);
        }
        if let Some(rest) = o.strip_prefix("https://") {
            return self.public_names.iter().any(|n| n == rest);
        }
        false
    }

    /// Did this request reach the studio over HTTPS? True for a `Host` that is the host of an `https://` `--public-origin`,
    /// for an `https://` `Origin` and for `X-Forwarded-Proto: https` from the TLS proxy. Used only to add `Secure` to the
    /// login cookie (a header a client forges can only make its own cookie stricter).
    pub fn is_https(&self, req: &HttpReq) -> bool {
        let host = req.header("Host").map(|h| h.trim().to_ascii_lowercase()).unwrap_or_default();
        self.https_names.contains(&host)
            || req.header("Origin").is_some_and(|o| o.trim().to_ascii_lowercase().starts_with("https://"))
            || req.header("X-Forwarded-Proto").is_some_and(|v| v.trim().eq_ignore_ascii_case("https"))
    }

    /// Set the `--source-url` shown on `/source`.
    pub fn with_source_url(mut self, url: Option<String>) -> Policy {
        self.source_url = url;
        self
    }

    fn host_ok(&self, host: &str) -> bool {
        let h = host.trim().to_ascii_lowercase();
        if self.hosts.contains(&h) || self.public_names.contains(&h) {
            return true;
        }
        match (self.ip_port, h.rsplit_once(':')) {
            (Some(port), Some((name, p))) if p.parse::<u16>().ok() == Some(port) => name.trim_matches(|c| c == '[' || c == ']').parse::<std::net::IpAddr>().is_ok(),
            _ => false,
        }
    }

    pub fn check(&self, req: &HttpReq) -> Result<(), Resp> {
        let deny = |code: u16, msg: &str| Err(Resp::from_string(msg.to_string()).with_status_code(code));
        let Some(host) = req.header("Host") else { return deny(421, "a Host header is required") };
        if !self.host_ok(host) {
            return deny(421, "unexpected Host header: this studio only answers requests addressed to itself (DNS-rebinding protection)");
        }
        let path = req.url.split('?').next().unwrap_or("");
        let page = req.method == Method::Get && path == "/";
        if let Some(o) = req.header("Origin") {
            if !self.origin_ok(o) {
                return deny(403, "cross-origin request refused");
            }
        }
        if !page {
            if let Some(site) = req.header("Sec-Fetch-Site") {
                if site != "same-origin" && site != "none" {
                    return deny(403, "cross-site request refused");
                }
            }
        }
        if req.method == Method::Post && !req.body.is_empty() {
            let ct = req.header("Content-Type").unwrap_or("").split(';').next().unwrap_or("").trim().to_ascii_lowercase();
            if ct != "application/json" {
                return deny(415, "POST bodies must be sent as Content-Type: application/json");
            }
        }
        Ok(())
    }
}

/// `--public-origin https://studio.example.org[:port]`: the host (and port) browsers use to reach the studio through
/// a TLS reverse proxy. A bare origin only: scheme, host, optional port, nothing else.
pub fn public_origin_host(u: &str) -> Result<String, String> {
    let rest = u.strip_prefix("https://").or_else(|| u.strip_prefix("http://")).ok_or("--public-origin must be an http:// or https:// URL such as https://studio.example.org")?;
    let ok = !rest.is_empty() && rest.len() <= 255 && !rest.contains(['/', '?', '#', '@', '\\']) && !rest.chars().any(|c| c.is_whitespace() || c.is_control());
    if ok { Ok(rest.to_ascii_lowercase()) } else { Err("--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org".into()) }
}

/// Funding links (a removable courtesy, never required).
pub const SUPPORT_LINE: &str = "Support the project: https://buymeacoffee.com/echo.of.the.strings - https://patreon.com/c/alexio_tf";

/// Validate `--source-url`: an absolute http(s) URL without whitespace, quotes or angle brackets.
pub fn check_source_url(u: &str) -> Result<(), String> {
    let ok = (u.starts_with("https://") || u.starts_with("http://")) && u.len() <= 500 && u.len() > 9 && !u.chars().any(|c| c.is_whitespace() || c.is_control() || matches!(c, '"' | '\'' | '<' | '>' | '\\'));
    if ok { Ok(()) } else { Err("--source-url must be an http:// or https:// URL without spaces or quotes".into()) }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The source address built into official release binaries: `release.yml` sets `OSATURA_SOURCE_URL` at compile time to
/// the commit the binary is built from (`<server>/<repository>/tree/<sha>`; a fork building with its own workflow points
/// to itself). A plain `cargo build` has none, so a modified build does not claim somebody else's source. An explicit
/// `--source-url` always wins; an unusable value is ignored.
pub fn default_source_url() -> Option<&'static str> {
    option_env!("OSATURA_SOURCE_URL").filter(|u| !u.is_empty() && check_source_url(u).is_ok())
}

/// Plain-text source offer (CLI `--version` and the body of `/source`).
pub fn source_offer_text(url: Option<&str>) -> String {
    let where_ = match url {
        Some(u) => format!("The corresponding source code of this exact version is available at: {u}"),
        None => "The operator of this server has not configured a source URL (this build has none built in). You are entitled to the complete corresponding source of the version you are using: ask the person or organisation that runs this server or that gave you this program, or the maintainer (Alexio, alexio@trom.tf), and it will be provided. (The operator can set the address with `osatura studio --source-url URL`.)".to_string(),
    };
    format!("OSATURA studio (osatura-cli) is free software under the GNU Affero General Public License, version 3 or (at your option) any later version. Copyright (c) 2026 Alexandru-Ionut Chiuta (Alexio). It comes with NO WARRANTY. If you modify it and let others interact with it over a network, section 13 of the AGPL requires you to offer them the corresponding source of your version. {where_} The engine libraries it links (osatura-core and others) are MPL-2.0 and keep that licence file by file; see COPYRIGHT. {SUPPORT_LINE}")
}

fn source_page(url: Option<&str>) -> String {
    let link = match url {
        Some(u) => format!("<p><a href=\"{0}\" rel=\"noopener\">{0}</a></p>", esc(u)),
        None => String::new(),
    };
    format!("<!doctype html><meta charset=utf-8><meta name=viewport content=\"width=device-width,initial-scale=1\"><title>OSATURA source</title><style>body{{font:16px/1.5 system-ui;margin:2em auto;max-width:42em;padding:0 16px}}</style><h1>OSATURA studio: source code</h1><p>{}</p>{link}<p><a href=\"/\">Back to the studio</a></p>", esc(&source_offer_text(url)))
}

/// The studio icon: two bones joined at a joint. Served for `GET /favicon.ico`.
pub const FAVICON_SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 32 32\"><rect width=\"32\" height=\"32\" rx=\"6\" fill=\"#20242b\"/><path d=\"M7 24 L15 14 L25 9\" stroke=\"#f2b98f\" stroke-width=\"3\" stroke-linecap=\"round\" fill=\"none\"/><circle cx=\"15\" cy=\"14\" r=\"3\" fill=\"#f6f1e7\"/></svg>";

const LOGIN_PAGE: &str = "<!doctype html><meta charset=utf-8><title>OSATURA studio</title><style>body{font:16px system-ui;margin:3em auto;max-width:24em}input,button{font:inherit;padding:.4em;margin:.3em 0;width:100%;box-sizing:border-box}</style><h1>OSATURA studio</h1><p>This studio needs its access token.</p><form id=f><input id=t type=password autocomplete=current-password placeholder=token autofocus><button>Open</button></form><p id=m></p><script>f.onsubmit=async e=>{e.preventDefault();const r=await fetch('/api/login',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({token:t.value})});if(r.ok)location.reload();else m.textContent=r.status==429?'Too many attempts, wait a moment.':'Wrong token.'}</script>";

#[allow(clippy::too_many_arguments)]
fn handle(ed: &mut Editor, live: &mut LiveSession, face: &mut FaceState, ui: &mut UiStore, cache: &mut FrameCache, lodc: &mut LodProxy, collab: &mut Collab, save_path: &str, policy: &Policy, req: HttpReq) -> Reply {
    if let Err(r) = policy.check(&req) {
        return Reply::Full(r);
    }
    let url = req.url.clone();
    let (path, query) = url.split_once('?').unwrap_or((&url, ""));
    let method = req.method;
    let header_of = |name: &str| req.header(name).map(String::from);
    // AGPL-3.0 section 13: the source offer must reach every user, so it is answered before (and without) the token
    if method == Method::Get && path == "/source" {
        return Reply::Full(Resp::from_string(source_page(policy.source_url.as_deref())).with_header(hdr("Content-Type", "text/html; charset=utf-8")));
    }
    // browsers ask for /favicon.ico on their own: answer it (a tiny SVG, no data in it) instead of a 404 / 401 in the console
    if method == Method::Get && path == "/favicon.ico" {
        return Reply::Full(Resp::from_string(FAVICON_SVG).with_header(hdr("Content-Type", "image/svg+xml")).with_header(hdr("Cache-Control", "max-age=86400")));
    }
    if method == Method::Post && path == "/api/login" {
        return Reply::Full(collab.login(&req.body, &req.peer, policy.is_https(&req)));
    }
    if let Err(code) = collab.check_auth(header_of("X-Collab-Token").as_deref(), header_of("Cookie").as_deref(), &req.peer) {
        let r = if code == 429 {
            Resp::from_string("too many failed attempts: wait a moment and try again").with_status_code(429)
        } else if let Some(note) = collab.auth_note().filter(|_| !(method == Method::Get && path == "/")) {
            Resp::from_string(note.to_string()).with_status_code(401)
        } else if method == Method::Get && path == "/" {
            Resp::from_string(LOGIN_PAGE).with_header(hdr("Content-Type", "text/html; charset=utf-8")).with_status_code(401)
        } else {
            Resp::from_string("a token is required: sign in on the page, or send the X-Collab-Token header").with_status_code(401)
        };
        return Reply::Full(r);
    }
    if let Err(wait) = collab.check_rate(&req.peer) {
        return Reply::Full(Resp::from_string(format!("too many requests from this address: wait {wait} s")).with_status_code(429).with_header(hdr("Retry-After", &wait.to_string())));
    }
    if method == Method::Get && path == "/api/collab/events" {
        let last = header_of("Last-Event-ID");
        return match collab.connect_sse(query, last.as_deref()) {
            Ok(rx) => Reply::Stream(rx),
            Err(r) => Reply::Full(r),
        };
    }
    let ctx = Ctx {
        user: header_of("X-Collab-Replica").and_then(|v| v.trim().parse().ok()),
        base: header_of("X-Collab-Base").and_then(|v| v.trim().parse().ok()),
        secret: header_of("X-Collab-Secret"),
    };
    // a participant id is only a name; acting as that participant needs the secret handed out when it joined
    if !collab.verify_ctx(&ctx) {
        return Reply::Full(json_resp(err(vec!["X-Collab-Replica needs the matching X-Collab-Secret".into()])).with_status_code(403));
    }
    let rev0 = ed.revision();
    let body = match std::str::from_utf8(&req.body) {
        Ok(b) => b.to_string(),
        Err(_) => return Reply::Full(Resp::from_string("request body is not UTF-8").with_status_code(400)),
    };
    let parse = |s: &str| -> Result<Value, Value> { serde_json::from_str(s).map_err(|e| err(vec![format!("bad JSON: {e}")])) };

    let result: Result<Resp, Value> = (|| {
        if let Some(r) = collab.route(ed, &method, path, query, &body, &ctx) {
            return r.map(json_resp);
        }
        match (&method, path) {
            (Method::Get, "/") => Ok(Resp::from_string(PAGE).with_header(hdr("Content-Type", "text/html; charset=utf-8"))),
            (Method::Get, "/api/state") => Ok(json_resp(state(ed))),
            (Method::Get, "/api/project") => Ok(json_resp(ed.doc().clone())),
            // Read-only version-control views (I-01): history and semantic diff of the repository next to the project file.
            (Method::Get, "/api/vcs/log") | (Method::Get, "/api/vcs/diff") => {
                let dir = std::path::Path::new(save_path).parent().filter(|b| !b.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
                let repo = osatura_core::vcs::Repo::open(dir).map_err(|e| err(vec![e]))?;
                let dec = |s: &str| s.replace("%7E", "~").replace("%7e", "~");
                if path == "/api/vcs/log" {
                    let limit: usize = qparam(query, "limit").unwrap_or("50").parse().map_err(|_| err(vec!["bad limit".into()]))?;
                    let rev = dec(qparam(query, "rev").unwrap_or("HEAD"));
                    let log = repo.log(&rev, limit.min(1000)).map_err(|e| err(vec![e]))?;
                    let items: Vec<Value> = log.iter().map(|(h, c)| json!({"hash": h, "author": c.author, "time": c.time, "message": c.message, "parents": c.parents})).collect();
                    Ok(json_resp(json!({"ok": true, "branch": repo.current_branch().ok().flatten(), "commits": items})))
                } else {
                    let a = dec(qparam(query, "a").unwrap_or("HEAD"));
                    let b = qparam(query, "b").map(dec);
                    let ch = repo.diff(Some(&a), b.as_deref()).map_err(|e| err(vec![e]))?;
                    Ok(json_resp(json!({"ok": true, "changes": ch})))
                }
            }
            (Method::Get, p) if p.starts_with("/frame/") && p.ends_with(".svg") => {
                let n: f32 = p["/frame/".len()..p.len() - 4].parse().map_err(|_| err(vec!["bad frame".into()]))?;
                let d = cached_frame(ed, cache, lodc, n, 0, 0, query.contains("bones=1"), None)?;
                Ok(Resp::from_data(d.bytes.as_ref().clone())
                    .with_header(hdr("Content-Type", d.ctype))
                    .with_header(hdr("X-Cache", if d.hit { "hit" } else { "miss" }))
                    .with_header(hdr("X-Render-Ms", &format!("{:.3}", d.ms)))
                    .with_header(hdr("Cache-Control", "no-store")))
            }
            // One frame at a chosen quality: proxy=0 full, 1..4 scrub proxies; kind=vector (default) | raster | png.
            (Method::Get, "/api/frame") => {
                let n: f32 = qparam(query, "frame").unwrap_or("0").parse().map_err(|_| err(vec!["bad frame".into()]))?;
                let level: u8 = qparam(query, "proxy").unwrap_or("0").parse().map_err(|_| err(vec!["bad proxy level".into()]))?;
                let kind = match qparam(query, "kind").unwrap_or("vector") {
                    "vector" | "svg" => 0u8,
                    "raster" => 1,
                    "png" => 2,
                    k => return Err(err(vec![format!("bad kind '{k}' (vector, raster, png)")])),
                };
                let kind = if level == 0 && kind == 1 { 0 } else { kind };
                // lod=1&screen=WxH&lodpx=P: draw from the reduced-detail copy that stays within P screen pixels on a W x H screen
                let lod = match qparam(query, "lod") {
                    Some("1") => {
                        let (w, h) = qparam(query, "screen").and_then(|s| s.split_once('x')).and_then(|(w, h)| Some((w.parse::<f64>().ok()?, h.parse::<f64>().ok()?))).ok_or_else(|| err(vec!["lod=1 needs screen=WxH".into()]))?;
                        if !(w.is_finite() && h.is_finite() && w >= 1.0 && h >= 1.0 && w <= 65536.0 && h <= 65536.0) {
                            return Err(err(vec!["screen must be WxH in pixels, 1..65536".into()]));
                        }
                        let px: f64 = qparam(query, "lodpx").unwrap_or("1").parse().map_err(|_| err(vec!["bad lodpx".into()]))?;
                        if !(px.is_finite() && (0.0..=16.0).contains(&px)) {
                            return Err(err(vec!["lodpx must be between 0 and 16".into()]));
                        }
                        let p = ed.project();
                        Some(LodAsk { screen_scale: (w / p.width as f64).min(h / p.height as f64), max_px: px })
                    }
                    Some("0") | None => None,
                    Some(x) => return Err(err(vec![format!("bad lod '{x}' (0 or 1)")])),
                };
                let d = cached_frame(ed, cache, lodc, n, level, kind, query.contains("bones=1"), lod)?;
                Ok(Resp::from_data(d.bytes.as_ref().clone())
                    .with_header(hdr("Content-Type", d.ctype))
                    .with_header(hdr("X-Cache", if d.hit { "hit" } else { "miss" }))
                    .with_header(hdr("X-Render-Ms", &format!("{:.3}", d.ms)))
                    .with_header(hdr("X-Lod-Level", &d.lod.to_string()))
                    .with_header(hdr("X-Lod-Bound-Px", &format!("{:.4}", d.lod_px)))
                    .with_header(hdr("X-Proxy-Level", &level.to_string()))
                    .with_header(hdr("Cache-Control", "no-store")))
            }
            (Method::Get, "/api/cache") => {
                lodc.poll(ed.revision());
                Ok(json_resp(json!({"ok": true, "revision": ed.revision(), "entries": cache.len(), "bytes": cache.bytes(), "hits": cache.hits, "misses": cache.misses, "invalidations": cache.invalidations,
                "lod": {"builds": lodc.builds, "invalidations": lodc.invalidations, "engines": lodc.engines_made, "last_build_ms": lodc.last_build_ms, "unavailable": lodc.unavailable, "levels": lodc.records().iter().map(|r| json!({"level": r.level, "bound_stage": r.bound_stage, "anchors": r.stats.path_anchors, "keys": r.stats.keys})).collect::<Vec<_>>()}})))
            }
            // Onion-skin / trail / proxy settings (stored with the other UI settings) and the ghost picture for a frame.
            (Method::Get, "/api/overlays") => {
                let frame = qparam(query, "frame").map(|s| s.parse::<u32>().map_err(|_| err(vec!["bad frame".into()]))).transpose()?;
                let sel: Vec<String> = qparam(query, "sel").map(pct_decode).unwrap_or_default().split(',').filter(|x| !x.is_empty()).map(String::from).collect();
                Ok(json_resp(overlay_view(ed, ui, frame, &sel)?))
            }
            (Method::Post, "/api/overlays") => {
                let v = parse(&body)?;
                if v["reset"].as_bool() == Some(true) {
                    ui.update(&json!({"reset": "overlays"})).map_err(err)?;
                } else if !v["settings"].is_null() {
                    ui.update(&json!({"overlays": v["settings"].clone()})).map_err(err)?;
                }
                let frame = v["frame"].as_u64().map(|f| f as u32);
                let sel: Vec<String> = v["selection"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                Ok(json_resp(overlay_view(ed, ui, frame, &sel)?))
            }
            // Motion trails for targets (layer pivot / bone / control point) around a frame, with optional tick hit-testing.
            (Method::Post, "/api/trails") => {
                let v = parse(&body)?;
                let st = ui.overlays().trails;
                let targets: Vec<TrailTarget> = serde_json::from_value(v["targets"].clone()).map_err(|e| err(vec![format!("bad targets: {e}")]))?;
                let last = ed.project().frames.max(1) - 1;
                let (from, to) = match (v["from"].as_u64(), v["to"].as_u64()) {
                    (Some(f), Some(t)) => (f as u32, t as u32),
                    _ => {
                        let cur = v["frame"].as_u64().ok_or_else(|| err(vec!["give from+to or frame".into()]))? as u32;
                        let b = v["before"].as_u64().map(|x| x as u32).unwrap_or(st.before);
                        let a = v["after"].as_u64().map(|x| x as u32).unwrap_or(st.after);
                        overlays::trail_range(last + 1, cur.min(last), b, a)
                    }
                };
                let smooth = v["smooth"].as_u64().map(|x| x as u32).unwrap_or(st.smooth);
                let tr = overlays::trails(ed.engine(), &targets, from, to, smooth).map_err(|e| err(vec![e]))?;
                let mut out = json!({"ok": true, "from": from, "to": to, "trails": tr.iter().map(overlays::trail_json).collect::<Vec<_>>()});
                if v["svg"].as_bool().unwrap_or(false) {
                    out["svg"] = json!(overlays::trails_svg(ed.engine(), &tr, v["frame"].as_u64().map(|f| f as u32)));
                }
                if v["hit"].is_object() {
                    let h = &v["hit"];
                    let (x, y, r) = (num_field(h, "x")?, num_field(h, "y")?, num_field(h, "radius")?);
                    out["hit"] = match overlays::hit_tick(&tr, Vec2::new(x, y), r) {
                        Some(t) => serde_json::to_value(&t).unwrap_or(Value::Null),
                        None => Value::Null,
                    };
                    out["path_hit"] = match overlays::hit_path(&tr, Vec2::new(x, y), r) {
                        Some((ti, f, d)) => json!({"trail": ti, "frame": f, "dist": d}),
                        None => Value::Null,
                    };
                }
                Ok(json_resp(out))
            }
            // Waveform of every sound track after trim/fades/gain (clip-local peaks), for the timeline.
            (Method::Get, "/api/sounds") => {
                let base = std::path::Path::new(save_path).parent().filter(|b| !b.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
                let p = ed.project();
                let total = (p.frames as f32 / p.fps).max(0.001);
                let sounds: Vec<Value> = (0..p.sounds.len())
                    .map(|i| {
                        let s = &p.sounds[i];
                        let head = json!({"name": s.name, "start": s.start / p.fps, "gain": s.gain});
                        // enough buckets for about one per two pixels of a 1200 px timeline
                        let est = (1200.0 * s.trim_out.map(|o| o - s.trim_in).unwrap_or(total) / total / 2.0).clamp(16.0, 800.0) as usize;
                        match crate::soundio::clip_peaks(p, base, i, est) {
                            Ok((dur, peaks)) => {
                                let mut h = head;
                                h["duration"] = json!(dur);
                                h["peaks"] = json!(peaks.iter().map(|(a, b)| [(a * 1000.0).round() / 1000.0, (b * 1000.0).round() / 1000.0]).collect::<Vec<_>>());
                                h
                            }
                            Err(e) => {
                                let mut h = head;
                                h["error"] = json!(e);
                                h
                            }
                        }
                    })
                    .collect();
                Ok(json_resp(json!({"ok": true, "total": total, "sounds": sounds})))
            }
            (Method::Get, "/export.gif") => {
                let b = export::gif_bytes(ed.engine(), 1.0).map_err(|e| err(vec![e]))?;
                Ok(Resp::from_data(b).with_header(hdr("Content-Type", "image/gif")).with_header(hdr("Content-Disposition", "attachment; filename=\"animation.gif\"")))
            }
            (Method::Get, "/export.html") => {
                let base = std::path::Path::new(save_path).parent().filter(|b| !b.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
                let au = if ed.project().sounds.is_empty() { None } else { crate::soundio::mix_bytes(ed.project(), base, "mp3").ok().filter(|(_, b)| b.len() <= export::HTML_AUDIO_MAX) };
                Ok(Resp::from_string(export::html_player_with_audio(ed.engine(), au.as_ref().map(|(m, b)| (*m, b.as_slice()))))
                .with_header(hdr("Content-Type", "text/html; charset=utf-8"))
                .with_header(hdr("Content-Disposition", "attachment; filename=\"player.html\"")))
            }
            (Method::Get, "/export.html.nosound") => Ok(Resp::from_string(export::html_player(ed.engine()))
                .with_header(hdr("Content-Type", "text/html; charset=utf-8"))
                .with_header(hdr("Content-Disposition", "attachment; filename=\"player.html\""))),
            (Method::Get, "/export.lottie.json") => Ok(Resp::from_string(export::lottie_json(ed.engine()).to_string())
                .with_header(hdr("Content-Type", "application/json"))
                .with_header(hdr("Content-Disposition", "attachment; filename=\"animation.json\""))),
            (Method::Post, "/api/op") => {
                let op: Op = serde_json::from_value(parse(&body)?).map_err(|e| err(vec![format!("bad op: {e}")]))?;
                if collab.active() {
                    // edits are interpreted in the sender's view and merged (index paths follow their elements)
                    return collab.op(ed, &ctx, op).map(json_resp);
                }
                ed.apply(op).map_err(err)?;
                Ok(json_resp(state(ed)))
            }
            (Method::Post, "/api/undo") => {
                if collab.active() {
                    // in a session undo only reverses the sender's own edits
                    return collab.undo_redo(ed, &ctx, true).map(json_resp);
                }
                ed.undo();
                Ok(json_resp(state(ed)))
            }
            (Method::Post, "/api/redo") => {
                if collab.active() {
                    return collab.undo_redo(ed, &ctx, false).map(json_resp);
                }
                ed.redo();
                Ok(json_resp(state(ed)))
            }
            (Method::Post, "/api/key") | (Method::Post, "/api/delete_key") => {
                let v = parse(&body)?;
                let target = v["target"].as_str().ok_or_else(|| err(vec!["missing target".into()]))?.to_string();
                let prop: Prop = serde_json::from_value(v["prop"].clone()).map_err(|e| err(vec![format!("bad prop: {e}")]))?;
                let frame = v["frame"].as_f64().ok_or_else(|| err(vec!["missing frame".into()]))? as f32;
                if path == "/api/key" {
                    let value = v["value"].as_f64().ok_or_else(|| err(vec!["missing value".into()]))? as f32;
                    let ease: Ease = if v["ease"].is_null() { Ease::Linear } else { serde_json::from_value(v["ease"].clone()).map_err(|e| err(vec![format!("bad ease: {e}")]))? };
                    ed.set_key(&target, prop, frame, value, ease).map_err(err)?;
                } else {
                    ed.delete_key(&target, prop, frame).map_err(err)?;
                }
                Ok(json_resp(state(ed)))
            }
            // ---- pose suggestions: classical PCA / kernel methods learned from the project's own keyed poses ----
            (Method::Post, "/api/suggest") => {
                let v = parse(&body)?;
                let req: posesuggest::Request = serde_json::from_value(v.clone()).map_err(|e| err(vec![format!("bad request: {e}")]))?;
                let lib: Option<Library> = if v["library"].is_null() { None } else { Some(serde_json::from_value(v["library"].clone()).map_err(|e| err(vec![format!("bad library: {e}")]))?) };
                let model: Option<Mlp> = if v["model"].is_null() { None } else { Some(Mlp::from_json(&v["model"].to_string()).map_err(|e| err(vec![e]))?) };
                let project = ed.project().clone();
                let db = posesuggest::build_db(Some(&project), lib.as_ref(), None).map_err(|e| err(vec![e]))?;
                let sg = posesuggest::Suggester::new(db).map_err(|e| err(vec![e]))?;
                let sug = posesuggest::suggest(&project, &sg, &req, model.as_ref()).map_err(|e| err(vec![e]))?;
                match v["apply"].as_u64() {
                    None => Ok(json_resp(json!({"ok": true, "suggestions": sug}))),
                    Some(i) => {
                        let c = sug.candidates.get(i as usize).ok_or_else(|| err(vec![format!("there is no candidate {i}")]))?;
                        let (op, _skipped) = posesuggest::apply_op(&project, &c.values, c.frame.unwrap_or(0.0), Ease::Smooth).map_err(|e| err(vec![e]))?;
                        let st = if collab.active() { collab.op(ed, &ctx, op)? } else { ed.apply(op).map_err(err)?; state(ed) };
                        Ok(json_resp(json!({"ok": true, "suggestions": sug, "state": st})))
                    }
                }
            }
            // ---- live input (MIDI / mouse / gamepad / keys captured by the page) ----
            (Method::Get, "/api/live") => Ok(json_resp(live_status(live))),
            (Method::Post, "/api/live/mapping") => {
                let v = parse(&body)?;
                let m: Mapping = serde_json::from_value(v["mapping"].clone()).map_err(|e| err(vec![format!("bad mapping: {e}")]))?;
                let lib = if v["library"].is_null() { None } else { Some(serde_json::from_value::<Library>(v["library"].clone()).map_err(|e| err(vec![format!("bad library: {e}")]))?) };
                live.set_mapping(ed.project(), m, lib).map_err(|e| err(vec![e]))?;
                Ok(json_resp(live_status(live)))
            }
            (Method::Post, "/api/live/push") => {
                let rec: Recording = serde_json::from_value(parse(&body)?).map_err(|e| err(vec![format!("bad input: {e}")]))?;
                let (events, warnings) = rec.into_events().map_err(|e| err(vec![e]))?;
                Ok(json_resp(live_push_resp(live, &events, warnings)?))
            }
            // ---- webcam face landmarks (the page supplies a landmark provider; no detector is built in) ----
            (Method::Post, "/api/face/config") => {
                let v = parse(&body)?;
                let cfg: FaceConfig = if v["config"].is_null() { FaceConfig::default() } else { serde_json::from_value(v["config"].clone()).map_err(|e| err(vec![format!("bad config: {e}")]))? };
                let cal = if v["calibration"].is_null() { Calibration::nominal() } else { serde_json::from_value(v["calibration"].clone()).map_err(|e| err(vec![format!("bad calibration: {e}")]))? };
                face.tracker = FaceTracker::new(cfg, cal).map_err(|e| err(vec![e]))?;
                face.collecting = None;
                Ok(json_resp(json!({"ok": true})))
            }
            (Method::Post, "/api/face/calibrate") => {
                let v = parse(&body)?;
                if v["on"].as_bool().ok_or_else(|| err(vec!["missing on".into()]))? {
                    face.collecting = Some(Vec::new());
                    Ok(json_resp(json!({"ok": true})))
                } else {
                    let rows = face.collecting.take().ok_or_else(|| err(vec!["not calibrating".into()]))?;
                    let cal = Calibration::from_rows(&rows, &rows).map_err(|e| err(vec![e]))?;
                    face.tracker.set_calibration(cal.clone()).map_err(|e| err(vec![e]))?;
                    Ok(json_resp(json!({"ok": true, "frames": rows.len(), "calibration": serde_json::to_value(&cal).unwrap_or(Value::Null)})))
                }
            }
            (Method::Post, "/api/face/push") => {
                let v = parse(&body)?;
                let frames = v["frames"].as_array().ok_or_else(|| err(vec!["missing frames".into()]))?;
                if frames.len() > 10_000 {
                    return Err(err(vec!["more than 10000 frames in one push".into()]));
                }
                let layout = if v["layout"].is_null() { None } else { Some(Layout::from_json(&v["layout"]).map_err(|e| err(vec![e]))?) };
                let size = match v["size"].as_array() {
                    Some(a) if a.len() == 2 && a[0].as_f64().unwrap_or(0.0) > 0.0 && a[1].as_f64().unwrap_or(0.0) > 0.0 => Some((a[0].as_f64().unwrap() as f32, a[1].as_f64().unwrap() as f32)),
                    None => None,
                    _ => return Err(err(vec!["size must be [w,h] with positive numbers".into()])),
                };
                let fr = faceparam::parse_frames(frames, layout, size, 30.0).map_err(|e| err(vec![e]))?;
                let mut events = Vec::new();
                for f in &fr {
                    if let Some(fc) = &f.face {
                        if let (Some(rows), Some(r)) = (&mut face.collecting, faceparam::measure(fc, &face.tracker.cfg.geometry)) {
                            rows.push(r);
                        }
                        events.extend(face.tracker.process(f.t, fc).map_err(|e| err(vec![e]))?);
                    }
                }
                Ok(json_resp(live_push_resp(live, &events, vec![])?))
            }
            // Server-side detection of one frame (base64 PNG/JPEG, data URI accepted) with the own detector.
            // It was trained on synthetic faces only: stylised / cartoon faces, NOT photographs or webcams.
            (Method::Post, "/api/face/detect") => {
                let v = parse(&body)?;
                let b64 = v["image"].as_str().ok_or_else(|| err(vec!["missing image (base64 PNG or JPEG)".into()]))?;
                let b64 = b64.split_once(",").map(|(_, r)| r).unwrap_or(b64);
                let bytes = osatura_core::base64::decode(b64.trim()).map_err(|e| err(vec![format!("bad base64: {e}")]))?;
                let img = osatura_core::autotrace::decode_image(&bytes).map_err(|e| err(vec![e]))?;
                let mut g = osatura_core::facegen::Gray::from_rgba(img.w, img.h, &img.rgba).map_err(|e| err(vec![e]))?;
                let f = g.w.max(g.h) as f32 / 256.0;
                if f > 1.0 {
                    g = g.scaled_down(f);
                }
                if face.model.is_none() {
                    face.model = Some(osatura_core::facedet::FaceModel::builtin().map_err(|e| err(vec![e]))?);
                }
                let m = face.model.as_ref().unwrap();
                let opts = osatura_core::facedet::DetectOpts { max_faces: 1, ..Default::default() };
                let found = if v["track"].as_bool().unwrap_or(true) { face.boxes.step(m, &g, &opts) } else { m.detect(&g, &opts).into_iter().next() };
                let (w, h) = (g.w as f32, g.h as f32);
                let pts: Option<Vec<Value>> = found.as_ref().map(|r| r.pts.iter().map(|p| json!([p.x / w, p.y / h])).collect());
                let mut resp = json!({"ok": true, "found": found.is_some(), "layout": "osatura23", "size": [img.w, img.h], "points": pts,
                    "box": found.as_ref().map(|r| json!({"cx": r.det.cx / w, "cy": r.det.cy / h, "side": r.det.side / w, "score": r.det.score})),
                    "note": "detector trained on synthetic faces only"});
                if v["push"].as_bool().unwrap_or(false) {
                    if let Some(r) = &found {
                        let mut fc = [Vec2::new(0.0, 0.0); 23];
                        for (i, p) in r.pts.iter().enumerate().take(23) {
                            fc[i] = Vec2::new(p.x, p.y) * (1.0 / w.max(1.0));
                        }
                        let t = v["t"].as_f64().unwrap_or(0.0) as f32;
                        let _ = h;
                        let measured = faceparam::measure(&fc, &face.tracker.cfg.geometry);
                        if let (Some(rows), Some(raw)) = (&mut face.collecting, measured) {
                            rows.push(raw);
                        }
                        let events = face.tracker.process(t, &fc).map_err(|e| err(vec![e]))?;
                        resp["live"] = live_push_resp(live, &events, vec![])?;
                    }
                }
                Ok(json_resp(resp))
            }
            // One frame of the synthetic head (osatura_core::facegen::anim_spec) rendered by the engine,
            // with its exact landmarks; the studio's "Synthetic face (detector)" mode feeds it to /api/face/detect.
            (Method::Post, "/api/face/synth") => {
                let v = parse(&body)?;
                let t = v["t"].as_f64().unwrap_or(0.0) as f32;
                let size = v["size"].as_u64().unwrap_or(128).clamp(32, 512) as usize;
                let seed = v["seed"].as_u64().unwrap_or(7);
                let spec = osatura_core::facegen::anim_spec(seed, t, size as f32);
                let g = osatura_core::facegen::render_spec(&spec, size, size, Some(seed + 1), &osatura_core::facegen::Post::NONE, 0).map_err(|e| err(vec![e]))?;
                let png = g.to_png().map_err(|e| err(vec![e]))?;
                let pts: Vec<Value> = spec.landmarks().iter().map(|p| json!([p.x / size as f32, p.y / size as f32])).collect();
                Ok(json_resp(json!({"ok": true, "image": format!("data:image/png;base64,{}", osatura_core::base64::encode(&png)), "layout": "osatura23", "size": [size, size], "points": pts})))
            }
            (Method::Post, "/api/live/record") => {
                let v = parse(&body)?;
                if v["on"].as_bool().ok_or_else(|| err(vec!["missing on".into()]))? {
                    live.start_recording();
                    Ok(json_resp(live_status(live)))
                } else {
                    let events = live.stop_recording().ok_or_else(|| err(vec!["not recording".into()]))?;
                    let tol = v["tol"].as_f64().unwrap_or(0.01) as f32;
                    let mut q = ed.project().clone();
                    let rep = liveinput::bake(&mut q, &events, live.mapping(), live.library(), tol).map_err(|e| err(vec![e]))?;
                    let ops = vec![
                        Op::Set { path: "/channels".into(), value: serde_json::to_value(&q.channels).map_err(|e| err(vec![e.to_string()]))? },
                        Op::Set { path: "/events".into(), value: serde_json::to_value(&q.events).map_err(|e| err(vec![e.to_string()]))? },
                    ];
                    ed.apply(Op::Batch { ops }).map_err(err)?;
                    let mut st = state(ed);
                    st["live"] = json!({"events": events.len(), "channels": rep.channels, "keys_set": rep.keys_set, "param_keys": rep.param_keys, "cut_frames": rep.cut_frames, "takes": rep.takes, "project_events": rep.events, "skipped": rep.skipped, "notes": rep.notes});
                    Ok(json_resp(st))
                }
            }
            // ---- canvas selection and transform gizmos (stateless: the page holds the selection) ----
            (Method::Get, "/api/layers") => {
                let f = ed.engine().evaluate(0.0);
                let rows: Vec<Value> = (0..ed.engine().node_count())
                    .map(|id| {
                        let n = &f.nodes[id];
                        let ix = ed.engine().node_idx_path(id);
                        let mut ptr = format!("/layers/{}", ix[0]);
                        for i in &ix[1..] {
                            ptr.push_str(&format!("/layers/{i}"));
                        }
                        let reference = ed.project().find_layer(&n.name).map(|l| l.reference).unwrap_or(false);
                        // `locked` = this layer's own flag, `locked_eff` also counts the groups above it
                        let locked = ed.project().find_layer(&n.name).map(|l| l.locked).unwrap_or(false);
                        let locked_eff = ed.project().is_layer_locked(&n.name);
                        json!({"path": n.name, "depth": ix.len() - 1, "kind": format!("{:?}", n.kind).to_lowercase(), "visible": ed.project().find_layer(&n.name).map(|l| l.visible).unwrap_or(true), "reference": reference, "locked": locked, "locked_eff": locked_eff, "ptr": ptr})
                    })
                    .collect();
                Ok(json_resp(json!({"ok": true, "layers": rows})))
            }
            (Method::Post, "/api/pick") => {
                let v = parse(&body)?;
                let frame = num_field(&v, "frame")?;
                let p = Vec2::new(num_field(&v, "x")?, num_field(&v, "y")?);
                let o = pick_options(&v)?;
                let fr = ed.engine().evaluate(frame);
                let hits = pick::pick_all(ed.engine(), &fr, p, &o);
                Ok(json_resp(json!({"ok": true, "hit": hits.first(), "hits": hits})))
            }
            (Method::Post, "/api/marquee") => {
                let v = parse(&body)?;
                let frame = num_field(&v, "frame")?;
                let o = pick_options(&v)?;
                let mode = match v["mode"].as_str() {
                    None => MarqueeMode::Intersect,
                    Some(m) => MarqueeMode::parse(m).ok_or_else(|| err(vec![format!("bad mode '{m}' (bbox, contain, intersect)")]))?,
                };
                let region = if let Some(r) = v["rect"].as_array() {
                    let n: Vec<f32> = r.iter().filter_map(|x| x.as_f64()).map(|x| x as f32).collect();
                    if n.len() != 4 || n.iter().any(|x| !x.is_finite()) {
                        return Err(err(vec!["rect must be [x0, y0, x1, y1]".into()]));
                    }
                    Region::rect(Vec2::new(n[0], n[1]), Vec2::new(n[2], n[3]))
                } else if v["lasso"].is_array() {
                    let pts: Vec<Vec2> = serde_json::from_value(v["lasso"].clone()).map_err(|e| err(vec![format!("bad lasso: {e}")]))?;
                    if pts.len() < 3 || pts.len() > 5000 || pts.iter().any(|q| !q.x.is_finite() || !q.y.is_finite()) {
                        return Err(err(vec!["lasso needs 3..5000 finite points".into()]));
                    }
                    Region::Lasso(pts)
                } else {
                    return Err(err(vec!["give rect or lasso".into()]));
                };
                let fr = ed.engine().evaluate(frame);
                let targets = pick::marquee(ed.engine(), &fr, &region, mode, &o);
                Ok(json_resp(json!({"ok": true, "targets": targets})))
            }
            (Method::Post, "/api/select") => {
                let v = parse(&body)?;
                let frame = num_field(&v, "frame").unwrap_or(0.0);
                let mut sel = Selection::from_items(targets_of(&v["selection"])?);
                let hits = targets_of(&v["targets"])?;
                let mode = match v["mode"].as_str() {
                    None => SelectMode::Replace,
                    Some(m) => SelectMode::parse(m).ok_or_else(|| err(vec![format!("bad mode '{m}' (replace, add, toggle, subtract)")]))?,
                };
                sel.prune(ed.engine());
                let hits: Vec<Target> = hits.into_iter().filter(|t| match t {
                    Target::Layer { path } => ed.engine().node_index(path).is_some(),
                    Target::Bone { name } => ed.engine().bone_index(name).is_some(),
                    Target::Point { layer, .. } => ed.engine().node_index(layer).is_some(),
                }).collect();
                sel.apply(&hits, mode);
                Ok(json_resp(selection_view(ed, frame, &sel)))
            }
            (Method::Post, "/api/gizmo") => {
                let v = parse(&body)?;
                let req: DragRequest = serde_json::from_value(v.clone()).map_err(|e| err(vec![format!("bad gizmo request: {e}")]))?;
                if v["commit"].as_bool().unwrap_or(false) {
                    let out = gizmo::commit(ed, &req).map_err(err)?;
                    let mut st = state(ed);
                    st["drag"] = serde_json::to_value(&out).unwrap_or(Value::Null);
                    let mut sel = Selection::from_items(req.selection.clone());
                    sel.prune(ed.engine());
                    st["selection"] = selection_view(ed, req.frame, &sel);
                    Ok(json_resp(st))
                } else {
                    let out = gizmo::preview(ed, &req).map_err(|e| err(vec![e]))?;
                    Ok(json_resp(json!({"ok": true, "drag": out})))
                }
            }
            (Method::Post, "/api/nudge") => {
                let v = parse(&body)?;
                let frame = num_field(&v, "frame").unwrap_or(0.0);
                let sel = targets_of(&v["selection"])?;
                let d = v["delta"].as_array().filter(|a| a.len() == 2).and_then(|a| Some(Vec2::new(a[0].as_f64()? as f32, a[1].as_f64()? as f32))).ok_or_else(|| err(vec!["delta must be [dx, dy]".into()]))?;
                if !d.x.is_finite() || !d.y.is_finite() {
                    return Err(err(vec!["delta must be finite".into()]));
                }
                let locked: Vec<String> = serde_json::from_value(v["locked"].clone()).unwrap_or_default();
                let out = gizmo::nudge(ed, frame, &sel, d, v["auto_key"].as_bool().unwrap_or(false), &locked).map_err(err)?;
                let mut st = state(ed);
                st["drag"] = serde_json::to_value(&out).unwrap_or(Value::Null);
                let mut s2 = Selection::from_items(sel);
                s2.prune(ed.engine());
                st["selection"] = selection_view(ed, frame, &s2);
                Ok(json_resp(st))
            }
            (Method::Post, "/api/delete") => {
                let v = parse(&body)?;
                let frame = num_field(&v, "frame").unwrap_or(0.0);
                let sel = targets_of(&v["selection"])?;
                gizmo::delete(ed, &sel).map_err(err)?;
                let mut st = state(ed);
                let mut s2 = Selection::from_items(sel);
                s2.prune(ed.engine());
                st["selection"] = selection_view(ed, frame, &s2);
                Ok(json_resp(st))
            }
            // ---- timeline / dope sheet / graph editor (stateless: the page holds the key selection) ----
            (Method::Get, "/api/timeline") => {
                let mut v = serde_json::to_value(timeline::build(ed.project())).map_err(|e| err(vec![e.to_string()]))?;
                v["ok"] = json!(true);
                Ok(json_resp(v))
            }
            (Method::Post, "/api/timeline/select") => {
                let v = parse(&body)?;
                let m = timeline::build(ed.project());
                let cur: Vec<KeyRef> = if v["selection"].is_null() { Vec::new() } else { serde_json::from_value(v["selection"].clone()).map_err(|e| err(vec![format!("bad selection: {e}")]))? };
                let mode = match v["mode"].as_str() {
                    None => SelMode::Replace,
                    Some(s) => SelMode::parse(s).ok_or_else(|| err(vec![format!("bad mode '{s}' (replace, add, toggle, subtract)")]))?,
                };
                let incl = v["include_locked"].as_bool().unwrap_or(false);
                let idx = |k: &str| v[k].as_u64().map(|x| x as usize);
                let hits: Vec<KeyRef> = if v["all"].as_bool().unwrap_or(false) {
                    timeline::select_all(&m, incl)
                } else if let Some(c) = idx("channel") {
                    timeline::select_channel(&m, c, incl)
                } else if let Some(g) = idx("group") {
                    timeline::select_group(&m, g, incl)
                } else if v["box"].is_object() {
                    let b = &v["box"];
                    let n = |k: &str| b[k].as_f64().filter(|x| x.is_finite()).map(|x| x as f32).ok_or_else(|| err(vec![format!("box.{k} missing or not finite")]));
                    let r = |k: &str| b[k].as_u64().map(|x| x as usize).ok_or_else(|| err(vec![format!("box.{k} missing")]));
                    timeline::select_box(&m, n("f0")?, n("f1")?, r("row0")?, r("row1")?, incl)
                } else {
                    Vec::new()
                };
                let sel = timeline::select_apply(&timeline::prune(&m, &cur), &hits, mode);
                Ok(json_resp(json!({"ok": true, "selection": sel})))
            }
            (Method::Post, "/api/timeline/op") => {
                let v = parse(&body)?;
                let op: TlOp = serde_json::from_value(v["op"].clone()).map_err(|e| err(vec![format!("bad timeline op: {e}")]))?;
                if v["preview"].as_bool().unwrap_or(false) {
                    let pl = timeline::plan(ed.project(), &op).map_err(|e| err(vec![e]))?;
                    return Ok(json_resp(json!({"ok": true, "preview": true, "plan": pl})));
                }
                let pl = timeline::apply(ed, &op).map_err(err)?;
                let mut st = state(ed);
                st["plan"] = serde_json::to_value(&pl).unwrap_or(Value::Null);
                Ok(json_resp(st))
            }
            (Method::Get, "/api/graph") => {
                let q = |k: &str| query.split('&').find_map(|kv| kv.strip_prefix(&format!("{k}=")));
                let chans: Vec<usize> = q("ch").unwrap_or("").split(',').filter(|s| !s.is_empty()).map(|s| s.parse::<usize>().map_err(|_| err(vec![format!("bad channel '{s}'")]))).collect::<Result<_, _>>()?;
                let f0: f32 = q("f0").map(|s| s.parse().map_err(|_| err(vec!["bad f0".into()]))).transpose()?.unwrap_or(0.0);
                let f1: f32 = q("f1").map(|s| s.parse().map_err(|_| err(vec!["bad f1".into()]))).transpose()?.unwrap_or(ed.project().frames as f32);
                let n: usize = q("n").map(|s| s.parse().map_err(|_| err(vec!["bad n".into()]))).transpose()?.unwrap_or(200);
                let gv = timeline::graph_view(ed.project(), &chans, f0, f1, n.clamp(2, 2000)).map_err(|e| err(vec![e]))?;
                let mut j = serde_json::to_value(&gv).map_err(|e| err(vec![e.to_string()]))?;
                j["ok"] = json!(true);
                Ok(json_resp(j))
            }
            // ---- UI configuration: layout, shortcuts, recents (stored in the user config file, not in the project) ----
            (Method::Get, "/api/ui") => Ok(json_resp(ui.view())),
            (Method::Post, "/api/ui") => {
                ui.update(&parse(&body)?).map_err(err)?;
                Ok(json_resp(ui.view()))
            }
            (Method::Get, "/api/ui/search") => {
                let q = query.split('&').find_map(|kv| kv.strip_prefix("q=")).map(pct_decode).unwrap_or_default();
                let limit: usize = query.split('&').find_map(|kv| kv.strip_prefix("limit=")).and_then(|s| s.parse().ok()).unwrap_or(50);
                Ok(json_resp(ui.search_view(&q, limit.clamp(1, 200))))
            }
            (Method::Post, "/api/save") => {
                // atomic (temporary file + rename) and never through a symbolic link
                osatura_core::fsutil::write_atomic(std::path::Path::new(save_path), serde_json::to_string_pretty(ed.doc()).unwrap_or_default().as_bytes())
                    .map_err(|e| err(vec![format!("cannot save: {e}")]))?;
                Ok(json_resp(json!({"ok": true, "saved": save_path})))
            }
            _ => Ok(Resp::from_string("not found").with_status_code(404)),
        }
    })();
    if ed.revision() != rev0 && collab.active() {
        collab.refresh_bounds(ed);
    }
    // an edit the editor made by its own means (nudge, key, gizmo, ...) is recorded for the other participants
    if ed.revision() != rev0 && collab.active() && !matches!(path, "/api/op" | "/api/undo" | "/api/redo") && !path.starts_with("/api/collab/") {
        collab.sync_editor(ed, &ctx);
    }
    Reply::Full(result.unwrap_or_else(|e| json_resp(e).with_status_code(400)))
}

/// The credentials and limits of a collaboration session (see `collab_srv` and docs/SECURITY.md).
#[derive(Default, Clone)]
pub struct Security {
    /// `--collab-token`: one shared bearer value.
    pub token: Option<String>,
    /// `--collab-secret-file`: the secret that per-participant tokens are made from (`osatura collab-token`).
    pub secret: Option<String>,
    /// `--collab-revoke`: participant names whose tokens are refused.
    pub revoke: Vec<String>,
    /// `--collab-rate`: requests per second per client address; `None` = the default (100 when a credential is
    /// required, no limit otherwise), `Some(0.0)` = off.
    pub rate: Option<f64>,
}

/// Warnings about the chosen bind address and credentials (printed at start; a pure function so that it is tested).
pub fn bind_warnings(addr: &str, loopback: bool, sec: &Security) -> Vec<String> {
    let host = addr.rsplit_once(':').map(|(h, _)| h).unwrap_or(addr).trim_matches(|c| c == '[' || c == ']');
    let mut w = Vec::new();
    if loopback {
        return w;
    }
    let wildcard = host == "0.0.0.0" || host == "::" || host.is_empty();
    if wildcard {
        w.push(format!("WARNING: {addr} listens on EVERY network interface of this machine; bind a single address (--bind 192.0.2.10) or, better, keep 127.0.0.1 and put a TLS reverse proxy in front (docs/SECURITY.md)"));
    }
    w.push(format!("WARNING: listening on {addr} over plain HTTP. Tokens and the whole project cross the network in clear text and anything on the path can read them; use an SSH tunnel or a TLS reverse proxy (then name its public host with --allow-host or --public-origin, so that pages loaded from https://<that host> are accepted). Recipes for nginx and Caddy: docs/SECURITY.md."));
    if sec.secret.is_none() {
        w.push("WARNING: one shared --collab-token means everybody who knows it is the same user and it never expires; prefer per-participant tokens: --collab-secret-file F and `osatura collab-token NAME --secret-file F --ttl SECONDS`".into());
    }
    w
}

/// Serve until killed. `addr` like `127.0.0.1:8080`.
// public server entry point with a stable signature.
#[allow(clippy::too_many_arguments)]
pub fn serve(mut ed: Editor, save_path: &str, addr: &str, ui_path: Option<std::path::PathBuf>, sec: Security, collab_log: Option<(String, bool)>, allow_hosts: &[String], source_url: Option<String>, public_origin: Option<String>) -> Result<(), String> {
    let host = addr.rsplit_once(':').map(|(h, _)| h).unwrap_or(addr).trim_matches(|c| c == '[' || c == ']');
    let loopback = host == "localhost" || host.parse::<std::net::IpAddr>().map(|i| i.is_loopback()).unwrap_or(false);
    if !loopback && sec.token.is_none() && sec.secret.is_none() {
        return Err(format!("refusing to listen on {addr}: the studio has no accounts and no TLS, so a non-local address needs --collab-token or --collab-secret-file (and still sends everything in clear text)"));
    }
    let listener = std::net::TcpListener::bind(addr).map_err(|e| format!("cannot listen on {addr}: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let policy = Policy::new(addr, port, loopback, allow_hosts).with_public_origin(public_origin.as_deref()).with_source_url(source_url);
    // port 0 asks the system for a free port: say which one it is
    let shown = if addr.ends_with(":0") { format!("{}:{port}", addr.trim_end_matches(":0")) } else { addr.to_string() };
    let addr = shown.as_str();
    println!("AGPL-3.0 source offer: http://{addr}/source");
    println!("OSATURA studio on http://{addr}  (Ctrl-C to stop; saves go to {save_path})");
    let mut ui = UiStore::open(ui_path);
    match (&ui.path, &ui.warning) {
        (_, Some(w)) => println!("UI settings: {w}"),
        (Some(p), None) => println!("UI settings (layout, shortcuts) in {}", p.display()),
        (None, None) => println!("UI settings are kept for this session only (no config path)"),
    }
    for w in bind_warnings(addr, loopback, &sec) {
        println!("{w}");
    }
    if !allow_hosts.is_empty() || public_origin.is_some() {
        let names: Vec<String> = allow_hosts.iter().cloned().chain(public_origin.iter().cloned()).collect();
        println!("behind a proxy: Host and http/https Origin headers naming {} are accepted; every other origin is refused (403)", names.join(", "));
    }
    if sec.token.is_some() || sec.secret.is_some() {
        println!("collaboration token required: open http://{addr}/ and enter it once (it is then kept in an HttpOnly cookie); API clients send it as the X-Collab-Token header");
    }
    let rate = sec.rate.unwrap_or(if sec.token.is_some() || sec.secret.is_some() { 100.0 } else { 0.0 });
    let mut collab = Collab::new(ed.doc(), sec.token.clone()).with_rate(rate);
    if let Some(s) = &sec.secret {
        collab = collab.with_secret(s, sec.revoke.clone())?;
        println!("per-participant tokens: made with `osatura collab-token NAME --secret-file F`; each is bound to one name and expires{}", if sec.revoke.is_empty() { String::new() } else { format!("; revoked: {}", sec.revoke.join(", ")) });
    }
    if rate > 0.0 {
        println!("rate limit: {rate} requests per second per client address (bursts of {}); --collab-rate 0 turns it off", rate * 2.0);
    }
    if let Some((path, sync)) = &collab_log {
        println!("{}", collab.open_log(&mut ed, path, *sync)?);
    }
    let mut live = LiveSession::new();
    let mut lodc = LodProxy::background();
    // 256 pictures or 192 MB, whichever is reached first; emptied on every edit (see `FrameCache`)
    let mut cache = FrameCache::new(256, 192 << 20);
    let mut face = FaceState { tracker: FaceTracker::new(FaceConfig::default(), Calibration::nominal())?, collecting: None, model: None, boxes: Default::default() };
    crate::http::serve_loop(listener, Limits::default(), |req| handle(&mut ed, &mut live, &mut face, &mut ui, &mut cache, &mut lodc, &mut collab, save_path, &policy, req));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_source_url_is_a_valid_url_or_absent() {
        if let Some(u) = default_source_url() {
            assert!(check_source_url(u).is_ok(), "{u}");
        }
        // an explicit address is what the offer states, and the page links to it
        let t = source_offer_text(Some("https://example.org/src/tree/abc"));
        assert!(t.contains("available at: https://example.org/src/tree/abc"), "{t}");
        assert!(source_page(Some("https://example.org/src/tree/abc")).contains("href=\"https://example.org/src/tree/abc\""));
        assert!(source_offer_text(None).contains("has not configured a source URL"));
    }

    #[test]
    fn bind_warnings_cover_plain_http_wildcards_and_shared_tokens() {
        let shared = Security { token: Some("x".repeat(16)), ..Default::default() };
        assert!(bind_warnings("127.0.0.1:8080", true, &shared).is_empty(), "loopback needs no warning");
        let w = bind_warnings("0.0.0.0:8080", false, &shared);
        assert!(w.iter().any(|m| m.contains("EVERY network interface")), "{w:?}");
        assert!(w.iter().any(|m| m.contains("plain HTTP") && m.contains("nginx and Caddy")));
        assert!(w.iter().any(|m| m.contains("per-participant tokens")), "a shared token is discouraged");
        let w = bind_warnings("[::]:8080", false, &shared);
        assert!(w.iter().any(|m| m.contains("EVERY network interface")), "{w:?}");
        let per_user = Security { secret: Some("s".repeat(16)), ..Default::default() };
        let w = bind_warnings("192.0.2.10:8080", false, &per_user);
        assert!(w.len() == 1 && w[0].contains("plain HTTP"), "a single address with per-participant tokens: only the TLS warning: {w:?}");
    }

    fn req(method: Method, url: &str, headers: &[(&str, &str)], body: &str) -> HttpReq {
        HttpReq { method, url: url.into(), headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), body: body.as_bytes().to_vec(), peer: String::new() }
    }

    fn local() -> Policy {
        Policy::new("127.0.0.1:8080", 8080, true, &[])
    }

    fn code(p: &Policy, r: &HttpReq) -> u16 {
        p.check(r).map(|_| 200).unwrap_or_else(|e| e.status)
    }

    #[test]
    fn only_requests_addressed_to_the_studio_itself_are_answered() {
        let p = local();
        for h in ["127.0.0.1:8080", "localhost:8080", "LOCALHOST:8080", "[::1]:8080"] {
            assert_eq!(code(&p, &req(Method::Get, "/api/state", &[("Host", h)], "")), 200, "{h}");
        }
        // DNS rebinding: the page is served under the attacker's name
        for h in ["evil.example:8080", "evil.example", "127.0.0.1:9999", "127.0.0.1", "localhost.evil.example:8080", "192.168.1.5:8080"] {
            assert_eq!(code(&p, &req(Method::Get, "/api/project", &[("Host", h)], "")), 421, "{h}");
        }
        assert_eq!(code(&p, &req(Method::Get, "/api/project", &[], "")), 421, "no Host header");
    }

    #[test]
    fn a_non_local_bind_accepts_its_address_ip_literals_and_listed_names_only() {
        let p = Policy::new("0.0.0.0:8080", 8080, false, &["studio.lan".into()]);
        for h in ["192.168.1.5:8080", "10.0.0.2:8080", "[fe80::1]:8080", "studio.lan:8080", "0.0.0.0:8080"] {
            assert_eq!(code(&p, &req(Method::Get, "/", &[("Host", h)], "")), 200, "{h}");
        }
        for h in ["evil.example:8080", "192.168.1.5:9", "other.lan:8080"] {
            assert_eq!(code(&p, &req(Method::Get, "/", &[("Host", h)], "")), 421, "{h}");
        }
    }

    #[test]
    fn requests_made_by_other_web_pages_are_refused() {
        let p = local();
        let h = ("Host", "127.0.0.1:8080");
        let post = |extra: &[(&str, &str)], body: &str| {
            let mut hs = vec![h, ("Content-Type", "application/json")];
            hs.extend_from_slice(extra);
            code(&p, &req(Method::Post, "/api/save", &hs, body))
        };
        assert_eq!(post(&[("Origin", "http://127.0.0.1:8080"), ("Sec-Fetch-Site", "same-origin")], "{}"), 200);
        assert_eq!(post(&[], "{}"), 200, "command-line clients send neither header");
        assert_eq!(post(&[("Origin", "http://evil.example")], "{}"), 403);
        assert_eq!(post(&[("Origin", "null")], "{}"), 403);
        assert_eq!(post(&[("Origin", "http://localhost:9999")], "{}"), 403, "another local app on another port");
        assert_eq!(post(&[("Sec-Fetch-Site", "cross-site")], "{}"), 403);
        assert_eq!(post(&[("Sec-Fetch-Site", "same-site")], "{}"), 403, "same-site includes other ports of localhost");
        // reads: a cross-site page cannot fetch the API either, but may navigate to the page itself
        assert_eq!(code(&p, &req(Method::Get, "/api/project", &[h, ("Sec-Fetch-Site", "cross-site")], "")), 403);
        assert_eq!(code(&p, &req(Method::Get, "/", &[h, ("Sec-Fetch-Site", "cross-site")], "")), 200);
    }

    /// M1: behind a TLS reverse proxy the page's Origin is https://<public name>; it used to be refused (403) for
    /// every https origin even with --allow-host.
    #[test]
    fn https_origins_of_allowed_public_hosts_are_accepted_and_others_are_not() {
        let post = |p: &Policy, host: &str, origin: &str| code(p, &req(Method::Post, "/api/save", &[("Host", host), ("Content-Type", "application/json"), ("Origin", origin)], "{}"));
        let plain = local();
        assert_eq!(post(&plain, "127.0.0.1:8080", "https://127.0.0.1:8080"), 403, "no public host configured: https stays refused");
        let p = Policy::new("127.0.0.1:8080", 8080, true, &["studio.example.org".into()]);
        for host in ["studio.example.org", "studio.example.org:8080"] {
            assert_eq!(post(&p, host, "https://studio.example.org"), 200, "{host}");
        }
        assert_eq!(post(&p, "studio.example.org", "http://studio.example.org"), 200);
        assert_eq!(post(&p, "studio.example.org", "https://evil.example"), 403);
        assert_eq!(post(&p, "studio.example.org", "https://studio.example.org.evil.example"), 403);
        assert_eq!(post(&p, "studio.example.org", "https://evil.example/studio.example.org"), 403);
        assert_eq!(post(&p, "studio.example.org", "null"), 403);
        assert_eq!(post(&p, "evil.example", "https://studio.example.org"), 421, "the Host rule still applies");
        // --public-origin
        let q = Policy::new("127.0.0.1:8080", 8080, true, &[]).with_public_origin(Some("https://Studio.Example.org:8443"));
        assert_eq!(post(&q, "studio.example.org:8443", "https://studio.example.org:8443"), 200);
        assert_eq!(post(&q, "studio.example.org:8443", "https://studio.example.org"), 403);
        assert!(public_origin_host("https://a.example/path").is_err());
        assert!(public_origin_host("ftp://a.example").is_err());
        assert_eq!(public_origin_host("https://A.example:444").unwrap(), "a.example:444");
    }

    /// The login cookie is `Secure` exactly when the request came in over HTTPS.
    #[test]
    fn https_requests_are_recognised_for_the_secure_cookie() {
        let q = Policy::new("127.0.0.1:8080", 8080, true, &[]).with_public_origin(Some("https://studio.example.org"));
        assert!(q.is_https(&req(Method::Post, "/api/login", &[("Host", "studio.example.org")], "")));
        assert!(q.is_https(&req(Method::Post, "/api/login", &[("Host", "127.0.0.1:8080"), ("X-Forwarded-Proto", "https")], "")));
        assert!(q.is_https(&req(Method::Post, "/api/login", &[("Host", "127.0.0.1:8080"), ("Origin", "https://studio.example.org")], "")));
        assert!(!q.is_https(&req(Method::Post, "/api/login", &[("Host", "127.0.0.1:8080")], "")), "plain http on localhost: no Secure (the cookie would be dropped)");
        // an http:// public origin never makes the cookie Secure
        let h = Policy::new("127.0.0.1:8080", 8080, true, &[]).with_public_origin(Some("http://studio.lan"));
        assert!(!h.is_https(&req(Method::Post, "/api/login", &[("Host", "studio.lan")], "")));
    }

    #[test]
    fn post_bodies_must_be_json() {
        let p = local();
        let h = ("Host", "127.0.0.1:8080");
        for ct in ["text/plain", "application/x-www-form-urlencoded", "multipart/form-data; boundary=x", ""] {
            let hs: Vec<(&str, &str)> = if ct.is_empty() { vec![h] } else { vec![h, ("Content-Type", ct)] };
            assert_eq!(code(&p, &req(Method::Post, "/api/op", &hs, "{}")), 415, "{ct:?}");
        }
        assert_eq!(code(&p, &req(Method::Post, "/api/op", &[h, ("Content-Type", "application/json; charset=utf-8")], "{}")), 200);
        assert_eq!(code(&p, &req(Method::Post, "/api/undo", &[h], "")), 200, "bodyless POSTs need no type (the origin checks cover them)");
    }
}
