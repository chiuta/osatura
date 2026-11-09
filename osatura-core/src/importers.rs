// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Importers for open, text-based formats: SVG (subset), BVH motion capture, and mouth-cue
//! files (Papagayo/Moho-switch style `frame PHONEME` lists and Rhubarb-style TSV).
//!
//! Provenance note: written from the public W3C SVG 1.1 path grammar, the published BVH
//! description, and the plain-text layouts of the cue files. Only the 2-D subset useful for
//! rigging is supported; unsupported features are reported as warnings, never guessed.

use crate::math::Vec2;
use crate::path::{Anchor, VPath};
use crate::perf::record;
use crate::project::{Channel, Prop};
use std::collections::HashMap;

// =============================================================================================
// SVG
// =============================================================================================

const MAX_SVG_BYTES: usize = 8 * 1024 * 1024;
const MAX_ANCHORS: usize = 200_000;

#[derive(Clone, Debug, Default)]
pub struct SvgImport {
    pub width: f32,
    pub height: f32,
    pub paths: Vec<VPath>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
struct M {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl M {
    const ID: M = M { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };
    /// `self ∘ o`: apply `o` first, then `self` (SVG nesting order).
    fn then(self, o: M) -> M {
        M {
            a: self.a * o.a + self.c * o.b,
            b: self.b * o.a + self.d * o.b,
            c: self.a * o.c + self.c * o.d,
            d: self.b * o.c + self.d * o.d,
            e: self.a * o.e + self.c * o.f + self.e,
            f: self.b * o.e + self.d * o.f + self.f,
        }
    }
    fn apply(&self, p: Vec2) -> Vec2 {
        Vec2::new(self.a * p.x + self.c * p.y + self.e, self.b * p.x + self.d * p.y + self.f)
    }
}

fn parse_numbers(s: &str) -> Vec<f32> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if let Some((v, n)) = scan_number(&b[i..]) {
            out.push(v);
            i += n;
        } else {
            i += 1;
        }
    }
    out
}

/// Parse one SVG number at the start of `b`; returns (value, bytes consumed).
fn scan_number(b: &[u8]) -> Option<(f32, usize)> {
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let ds = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let mut digits = i - ds;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        digits += i - fs;
    }
    if digits == 0 {
        return None;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let es = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > es {
            i = j;
        }
    }
    std::str::from_utf8(&b[..i]).ok()?.parse::<f32>().ok().map(|v| (v, i))
}

fn parse_transform(s: &str) -> Result<M, String> {
    let mut m = M::ID;
    let mut rest = s.trim();
    while !rest.is_empty() {
        let open = rest.find('(').ok_or("bad transform")?;
        let close = rest.find(')').ok_or("bad transform")?;
        let name = rest[..open].trim();
        let v = parse_numbers(&rest[open + 1..close]);
        let t = match (name, v.len()) {
            ("translate", 1) => M { e: v[0], ..M::ID },
            ("translate", 2) => M { e: v[0], f: v[1], ..M::ID },
            ("scale", 1) => M { a: v[0], d: v[0], ..M::ID },
            ("scale", 2) => M { a: v[0], d: v[1], ..M::ID },
            ("rotate", 1) | ("rotate", 3) => {
                let (s_, c_) = v[0].to_radians().sin_cos();
                let r = M { a: c_, b: s_, c: -s_, d: c_, e: 0.0, f: 0.0 };
                if v.len() == 3 {
                    M { e: v[1], f: v[2], ..M::ID }.then(r).then(M { e: -v[1], f: -v[2], ..M::ID })
                } else {
                    r
                }
            }
            ("skewX", 1) => M { c: v[0].to_radians().tan(), ..M::ID },
            ("skewY", 1) => M { b: v[0].to_radians().tan(), ..M::ID },
            ("matrix", 6) => M { a: v[0], b: v[1], c: v[2], d: v[3], e: v[4], f: v[5] },
            _ => return Err(format!("unsupported transform '{name}'")),
        };
        m = m.then(t);
        rest = rest[close + 1..].trim_start_matches([' ', ',', '\n', '\t', '\r']);
    }
    Ok(m)
}

fn named_color(n: &str) -> Option<&'static str> {
    Some(match n {
        "black" => "#000000",
        "white" => "#ffffff",
        "red" => "#ff0000",
        "green" => "#008000",
        "lime" => "#00ff00",
        "blue" => "#0000ff",
        "yellow" => "#ffff00",
        "gray" | "grey" => "#808080",
        "orange" => "#ffa500",
        "purple" => "#800080",
        "cyan" | "aqua" => "#00ffff",
        "magenta" | "fuchsia" => "#ff00ff",
        _ => return None,
    })
}

/// `None` = no paint; `Some(Err)` = unsupported paint.
fn parse_paint(v: &str) -> Option<Result<String, String>> {
    let v = v.trim();
    if v == "none" || v == "transparent" {
        return None;
    }
    if let Some(h) = v.strip_prefix('#') {
        return Some(match h.len() {
            3 => Ok(format!("#{}", h.chars().flat_map(|c| [c, c]).collect::<String>())),
            6 if h.chars().all(|c| c.is_ascii_hexdigit()) => Ok(format!("#{h}")),
            _ => Err(format!("unsupported colour '{v}'")),
        });
    }
    if let Some(c) = named_color(&v.to_ascii_lowercase()) {
        return Some(Ok(c.to_string()));
    }
    Some(Err(format!("unsupported paint '{v}'")))
}

#[derive(Clone, Debug)]
struct Style {
    fill: Option<String>,
    stroke: Option<String>,
    stroke_width: f32,
}

fn parse_attrs(tag: &str) -> HashMap<String, String> {
    let b = tag.as_bytes();
    let mut i = 0;
    // skip element name
    while i < b.len() && !b[i].is_ascii_whitespace() {
        i += 1;
    }
    let mut out = HashMap::new();
    while i < b.len() {
        while i < b.len() && (b[i].is_ascii_whitespace() || b[i] == b'/') {
            i += 1;
        }
        let ns = i;
        while i < b.len() && b[i] != b'=' && !b[i].is_ascii_whitespace() && b[i] != b'/' {
            i += 1;
        }
        let name = &tag[ns..i];
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= b.len() || b[i] != b'=' {
            continue;
        }
        i += 1;
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= b.len() || (b[i] != b'"' && b[i] != b'\'') {
            continue;
        }
        let q = b[i];
        let vs = i + 1;
        let mut j = vs;
        while j < b.len() && b[j] != q {
            j += 1;
        }
        let v = tag[vs..j.min(tag.len())]
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        if !name.is_empty() {
            out.insert(name.to_string(), v);
        }
        i = j + 1;
    }
    // inline CSS declarations override presentation attributes
    if let Some(st) = out.get("style").cloned() {
        for decl in st.split(';') {
            if let Some((k, v)) = decl.split_once(':') {
                out.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    out
}

fn style_from(attrs: &HashMap<String, String>, parent: &Style, warnings: &mut Vec<String>) -> Style {
    let mut s = parent.clone();
    let mut paint = |key: &str, slot: &mut Option<String>| {
        if let Some(v) = attrs.get(key) {
            match parse_paint(v) {
                None => *slot = None,
                Some(Ok(c)) => *slot = Some(c),
                Some(Err(e)) => {
                    warnings.push(format!("{e}; using grey"));
                    *slot = Some("#808080".into());
                }
            }
        }
    };
    paint("fill", &mut s.fill);
    paint("stroke", &mut s.stroke);
    if let Some(w) = attrs.get("stroke-width").and_then(|w| parse_numbers(w).first().copied()) {
        s.stroke_width = w;
    }
    s
}

struct PathBuilder {
    subpaths: Vec<(Vec<Anchor>, bool)>,
    cur: Vec<Anchor>,
    pos: Vec2,
    start: Vec2,
}

impl PathBuilder {
    fn new() -> Self {
        PathBuilder { subpaths: vec![], cur: vec![], pos: Vec2::ZERO, start: Vec2::ZERO }
    }
    fn move_to(&mut self, p: Vec2) {
        self.flush(false);
        self.cur.push(Anchor::corner(p));
        self.pos = p;
        self.start = p;
    }
    fn ensure_started(&mut self) {
        if self.cur.is_empty() {
            let p = self.pos;
            self.cur.push(Anchor::corner(p));
            self.start = p;
        }
    }
    fn line_to(&mut self, p: Vec2) {
        self.ensure_started();
        self.cur.push(Anchor::corner(p));
        self.pos = p;
    }
    fn cubic_to(&mut self, h1: Vec2, h2: Vec2, p: Vec2) {
        self.ensure_started();
        if let Some(l) = self.cur.last_mut() {
            l.handle_out = Some(h1);
        }
        let mut a = Anchor::corner(p);
        a.handle_in = Some(h2);
        self.cur.push(a);
        self.pos = p;
    }
    fn close(&mut self) {
        self.flush(true);
        self.pos = self.start;
    }
    fn flush(&mut self, closed: bool) {
        if self.cur.len() >= 2 || (closed && self.cur.len() == 1) {
            let mut a = std::mem::take(&mut self.cur);
            if closed && a.len() >= 2 && a[0].pos.distance(a[a.len() - 1].pos) < 1e-4 {
                let last = a.pop().unwrap();
                a[0].handle_in = last.handle_in;
            }
            self.subpaths.push((a, closed));
        } else {
            self.cur.clear();
        }
    }
}

/// SVG arc → cubic Béziers (W3C implementation notes, endpoint → centre parameterisation).
fn arc_to(pb: &mut PathBuilder, rx: f32, ry: f32, rot_deg: f32, large: bool, sweep: bool, p: Vec2) {
    let p0 = pb.pos;
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx < 1e-6 || ry < 1e-6 || p0.distance(p) < 1e-6 {
        pb.line_to(p);
        return;
    }
    let (sin, cos) = rot_deg.to_radians().sin_cos();
    let dx = (p0.x - p.x) * 0.5;
    let dy = (p0.y - p.y) * 0.5;
    let x1 = cos * dx + sin * dy;
    let y1 = -sin * dx + cos * dy;
    let lam = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lam > 1.0 {
        let s = lam.sqrt();
        rx *= s;
        ry *= s;
    }
    let num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut k = (num / den).max(0.0).sqrt();
    if large == sweep {
        k = -k;
    }
    let cxp = k * rx * y1 / ry;
    let cyp = -k * ry * x1 / rx;
    let cx = cos * cxp - sin * cyp + (p0.x + p.x) * 0.5;
    let cy = sin * cxp + cos * cyp + (p0.y + p.y) * 0.5;
    let ang = |ux: f32, uy: f32, vx: f32, vy: f32| (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
    let th1 = ang(1.0, 0.0, (x1 - cxp) / rx, (y1 - cyp) / ry);
    let mut dth = ang((x1 - cxp) / rx, (y1 - cyp) / ry, (-x1 - cxp) / rx, (-y1 - cyp) / ry);
    if !sweep && dth > 0.0 {
        dth -= std::f32::consts::TAU;
    } else if sweep && dth < 0.0 {
        dth += std::f32::consts::TAU;
    }
    let segs = (dth.abs() / (std::f32::consts::FRAC_PI_2)).ceil().max(1.0) as usize;
    let step = dth / segs as f32;
    let t = 4.0 / 3.0 * (step / 4.0).tan();
    let pt = |a: f32| Vec2::new(cx + rx * a.cos() * cos - ry * a.sin() * sin, cy + rx * a.cos() * sin + ry * a.sin() * cos);
    let dpt = |a: f32| Vec2::new(-rx * a.sin() * cos - ry * a.cos() * sin, -rx * a.sin() * sin + ry * a.cos() * cos);
    for i in 0..segs {
        let a0 = th1 + step * i as f32;
        let a1 = a0 + step;
        let (s0, e0) = (pt(a0), pt(a1));
        let (d0, d1) = (dpt(a0), dpt(a1));
        let end = if i + 1 == segs { p } else { e0 };
        pb.cubic_to(Vec2::new(s0.x + t * d0.x, s0.y + t * d0.y), Vec2::new(e0.x - t * d1.x, e0.y - t * d1.y), end);
    }
}

/// Parse an SVG path `d` string into sub-paths (absolute coordinates).
pub fn parse_path_data(d: &str) -> Result<Vec<(Vec<Anchor>, bool)>, String> {
    let b = d.as_bytes();
    let mut i = 0;
    let mut pb = PathBuilder::new();
    let mut cmd = b' ';
    let mut last_ctrl: Option<Vec2> = None; // for S/T reflection
    let mut last_cmd_kind = b' ';
    let mut budget = 0usize;
    let skip_ws = |i: &mut usize| {
        while *i < b.len() && (b[*i].is_ascii_whitespace() || b[*i] == b',') {
            *i += 1;
        }
    };
    loop {
        skip_ws(&mut i);
        if i >= b.len() {
            break;
        }
        if b[i].is_ascii_alphabetic() {
            cmd = b[i];
            i += 1;
            if cmd == b'z' || cmd == b'Z' {
                pb.close();
                last_ctrl = None;
                last_cmd_kind = b'z';
                continue;
            }
        } else if cmd == b' ' || cmd == b'z' || cmd == b'Z' {
            return Err("path data must start with a command".into());
        }
        budget += 1;
        if budget > MAX_ANCHORS {
            return Err("path has too many segments".into());
        }
        let rel = cmd.is_ascii_lowercase();
        let base = if rel { pb.pos } else { Vec2::ZERO };
        let num = |i: &mut usize| -> Result<f32, String> {
            skip_ws(i);
            let (v, n) = scan_number(&b[*i..]).ok_or_else(|| format!("expected a number at byte {}", *i))?;
            *i += n;
            Ok(v)
        };
        let flag = |i: &mut usize| -> Result<bool, String> {
            while *i < b.len() && (b[*i].is_ascii_whitespace() || b[*i] == b',') {
                *i += 1;
            }
            match b.get(*i) {
                Some(b'0') => {
                    *i += 1;
                    Ok(false)
                }
                Some(b'1') => {
                    *i += 1;
                    Ok(true)
                }
                _ => Err("expected arc flag 0 or 1".into()),
            }
        };
        match cmd.to_ascii_uppercase() {
            b'M' => {
                let p = Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?);
                pb.move_to(p);
                cmd = if rel { b'l' } else { b'L' }; // implicit lineto
                last_ctrl = None;
                last_cmd_kind = b'M';
            }
            b'L' => {
                let p = Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?);
                pb.line_to(p);
                last_ctrl = None;
                last_cmd_kind = b'L';
            }
            b'H' => {
                let x = num(&mut i)?;
                let p = Vec2::new(if rel { pb.pos.x + x } else { x }, pb.pos.y);
                pb.line_to(p);
                last_ctrl = None;
                last_cmd_kind = b'L';
            }
            b'V' => {
                let y = num(&mut i)?;
                let p = Vec2::new(pb.pos.x, if rel { pb.pos.y + y } else { y });
                pb.line_to(p);
                last_ctrl = None;
                last_cmd_kind = b'L';
            }
            b'C' => {
                let h1 = Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?);
                let h2 = Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?);
                let p = Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?);
                pb.cubic_to(h1, h2, p);
                last_ctrl = Some(h2);
                last_cmd_kind = b'C';
            }
            b'S' => {
                let h1 = match (last_cmd_kind, last_ctrl) {
                    (b'C', Some(c)) => Vec2::new(2.0 * pb.pos.x - c.x, 2.0 * pb.pos.y - c.y),
                    _ => pb.pos,
                };
                let h2 = Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?);
                let p = Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?);
                pb.cubic_to(h1, h2, p);
                last_ctrl = Some(h2);
                last_cmd_kind = b'C';
            }
            b'Q' | b'T' => {
                let p0 = pb.pos;
                let q = if cmd.eq_ignore_ascii_case(&b'Q') {
                    Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?)
                } else {
                    match (last_cmd_kind, last_ctrl) {
                        (b'Q', Some(c)) => Vec2::new(2.0 * p0.x - c.x, 2.0 * p0.y - c.y),
                        _ => p0,
                    }
                };
                let p = Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?);
                let h1 = Vec2::new(p0.x + 2.0 / 3.0 * (q.x - p0.x), p0.y + 2.0 / 3.0 * (q.y - p0.y));
                let h2 = Vec2::new(p.x + 2.0 / 3.0 * (q.x - p.x), p.y + 2.0 / 3.0 * (q.y - p.y));
                pb.cubic_to(h1, h2, p);
                last_ctrl = Some(q);
                last_cmd_kind = b'Q';
            }
            b'A' => {
                let rx = num(&mut i)?;
                let ry = num(&mut i)?;
                let rot = num(&mut i)?;
                let large = flag(&mut i)?;
                let sweep = flag(&mut i)?;
                let p = Vec2::new(base.x + num(&mut i)?, base.y + num(&mut i)?);
                arc_to(&mut pb, rx, ry, rot, large, sweep, p);
                last_ctrl = None;
                last_cmd_kind = b'A';
            }
            other => return Err(format!("unknown path command '{}'", other as char)),
        }
    }
    pb.flush(false);
    Ok(pb.subpaths)
}

fn transform_anchors(a: &mut [Anchor], m: &M) {
    for x in a {
        x.map_points(&mut |p| m.apply(p));
    }
}

fn length_attr(attrs: &HashMap<String, String>, k: &str, default: f32) -> f32 {
    attrs.get(k).and_then(|v| parse_numbers(v).first().copied()).unwrap_or(default)
}

/// Import an SVG document (subset): paths, rect, circle, ellipse, line, polyline, polygon,
/// nested `<g>` with transforms, solid fill/stroke. Gradients, filters, text, clipping and
/// `<use>` are reported as warnings and skipped.
pub fn import_svg(text: &str) -> Result<SvgImport, String> {
    if text.len() > MAX_SVG_BYTES {
        return Err("SVG is larger than 8 MB".into());
    }
    let mut out = SvgImport::default();
    let mut stack: Vec<(M, Style, bool)> = vec![(M::ID, Style { fill: Some("#000000".into()), stroke: None, stroke_width: 1.0 }, false)];
    let mut skip_depth: Vec<String> = Vec::new();
    let mut anchors_total = 0usize;
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut count = 0usize;
    while i < bytes.len() {
        let Some(off) = text[i..].find('<') else { break };
        let start = i + off;
        if text[start..].starts_with("<!--") {
            i = text[start..].find("-->").map(|e| start + e + 3).ok_or("unterminated comment")?;
            continue;
        }
        if text[start..].starts_with("<![CDATA[") {
            i = text[start..].find("]]>").map(|e| start + e + 3).ok_or("unterminated CDATA")?;
            continue;
        }
        if text[start..].starts_with("<?") || text[start..].starts_with("<!") {
            i = text[start..].find('>').map(|e| start + e + 1).ok_or("unterminated declaration")?;
            continue;
        }
        // find the end of the tag, honouring quotes
        let mut j = start + 1;
        let mut q = 0u8;
        while j < bytes.len() {
            let c = bytes[j];
            if q != 0 {
                if c == q {
                    q = 0;
                }
            } else if c == b'"' || c == b'\'' {
                q = c;
            } else if c == b'>' {
                break;
            }
            j += 1;
        }
        if j >= bytes.len() {
            return Err("unterminated tag".into());
        }
        let inner = &text[start + 1..j];
        i = j + 1;
        count += 1;
        if count > 200_000 {
            return Err("SVG has too many elements".into());
        }
        if let Some(close) = inner.strip_prefix('/') {
            let name = close.trim();
            if skip_depth.last().map(|s| s == name).unwrap_or(false) {
                skip_depth.pop();
            } else if skip_depth.is_empty() && (name == "g" || name == "svg") && stack.len() > 1 {
                stack.pop();
            }
            continue;
        }
        let self_closing = inner.ends_with('/');
        let name_end = inner.find(|c: char| c.is_whitespace() || c == '/').unwrap_or(inner.len());
        let name = &inner[..name_end];
        if !skip_depth.is_empty() {
            if !self_closing && skip_depth.last().map(|s| s == name).unwrap_or(false) {
                skip_depth.push(name.to_string());
            }
            continue;
        }
        let attrs = parse_attrs(inner);
        let (pm, pstyle, _) = stack.last().cloned().unwrap();
        match name {
            "defs" | "clipPath" | "mask" | "filter" | "style" | "script" | "text" | "symbol" | "linearGradient" | "radialGradient" | "pattern" | "metadata" | "title" | "desc" => {
                if matches!(name, "clipPath" | "mask" | "filter" | "text" | "pattern" | "linearGradient" | "radialGradient") {
                    out.warnings.push(format!("<{name}> is not supported and was skipped"));
                }
                if !self_closing {
                    skip_depth.push(name.to_string());
                }
                continue;
            }
            "svg" | "g" | "a" => {
                if name == "svg" && stack.len() == 1 && out.width == 0.0 {
                    if let Some(vb) = attrs.get("viewBox") {
                        let v = parse_numbers(vb);
                        if v.len() == 4 {
                            out.width = v[2];
                            out.height = v[3];
                        }
                    }
                    if out.width == 0.0 {
                        out.width = length_attr(&attrs, "width", 0.0);
                        out.height = length_attr(&attrs, "height", 0.0);
                    }
                }
                let t = match attrs.get("transform") {
                    Some(t) => parse_transform(t).unwrap_or_else(|e| {
                        out.warnings.push(e);
                        M::ID
                    }),
                    None => M::ID,
                };
                let st = style_from(&attrs, &pstyle, &mut out.warnings);
                if !self_closing {
                    stack.push((pm.then(t), st, true));
                }
                continue;
            }
            _ => {}
        }
        let t = match attrs.get("transform") {
            Some(t) => parse_transform(t).unwrap_or_else(|e| {
                out.warnings.push(e);
                M::ID
            }),
            None => M::ID,
        };
        let m = pm.then(t);
        let st = style_from(&attrs, &pstyle, &mut out.warnings);
        let id = attrs.get("id").cloned().unwrap_or_else(|| format!("{name}{}", out.paths.len()));
        let mut shapes: Vec<(Vec<Anchor>, bool)> = Vec::new();
        match name {
            "path" => match attrs.get("d").map(|d| parse_path_data(d)) {
                Some(Ok(sp)) => shapes = sp,
                Some(Err(e)) => out.warnings.push(format!("path '{id}': {e}")),
                None => {}
            },
            "rect" => {
                let (x, y, w, h) = (length_attr(&attrs, "x", 0.0), length_attr(&attrs, "y", 0.0), length_attr(&attrs, "width", 0.0), length_attr(&attrs, "height", 0.0));
                if attrs.contains_key("rx") || attrs.contains_key("ry") {
                    out.warnings.push(format!("rect '{id}': rounded corners ignored"));
                }
                if w > 0.0 && h > 0.0 {
                    let pts = [Vec2::new(x, y), Vec2::new(x + w, y), Vec2::new(x + w, y + h), Vec2::new(x, y + h)];
                    shapes.push((pts.iter().map(|&p| Anchor::corner(p)).collect(), true));
                }
            }
            "circle" | "ellipse" => {
                let (cx, cy) = (length_attr(&attrs, "cx", 0.0), length_attr(&attrs, "cy", 0.0));
                let (rx, ry) = if name == "circle" {
                    let r = length_attr(&attrs, "r", 0.0);
                    (r, r)
                } else {
                    (length_attr(&attrs, "rx", 0.0), length_attr(&attrs, "ry", 0.0))
                };
                if rx > 0.0 && ry > 0.0 {
                    shapes.push((VPath::ellipse("e", Vec2::new(cx, cy), rx, ry).anchors, true));
                }
            }
            "line" => {
                let (a, b) = (
                    Vec2::new(length_attr(&attrs, "x1", 0.0), length_attr(&attrs, "y1", 0.0)),
                    Vec2::new(length_attr(&attrs, "x2", 0.0), length_attr(&attrs, "y2", 0.0)),
                );
                shapes.push((vec![Anchor::corner(a), Anchor::corner(b)], false));
            }
            "polyline" | "polygon" => {
                let n = attrs.get("points").map(|p| parse_numbers(p)).unwrap_or_default();
                let pts: Vec<Anchor> = n.chunks_exact(2).map(|c| Anchor::corner(Vec2::new(c[0], c[1]))).collect();
                if pts.len() >= 2 {
                    shapes.push((pts, name == "polygon"));
                }
            }
            "use" | "image" | "foreignObject" => out.warnings.push(format!("<{name}> is not supported and was skipped")),
            _ => {}
        }
        for (k, (mut a, closed)) in shapes.into_iter().enumerate() {
            anchors_total += a.len();
            if anchors_total > MAX_ANCHORS {
                return Err("SVG has too many anchors".into());
            }
            transform_anchors(&mut a, &m);
            let pname = if k == 0 { id.clone() } else { format!("{id}_{k}") };
            let mut p = VPath::new(&pname, closed, a);
            if closed || st.fill.is_some() {
                if let Some(f) = &st.fill {
                    p = p.with_fill(f);
                }
            }
            if let Some(s) = &st.stroke {
                // stroke width scales with the transform's mean scale
                let sc = ((m.a * m.d - m.b * m.c).abs()).sqrt();
                p = p.with_stroke(s, st.stroke_width * sc);
            }
            if p.fill.is_none() && p.stroke.is_none() {
                continue; // invisible
            }
            out.paths.push(p);
        }
        if !self_closing && name == "a" {
            stack.push((pm, pstyle, true));
        }
    }
    // unique names
    let mut seen: HashMap<String, usize> = HashMap::new();
    for p in &mut out.paths {
        let n = seen.entry(p.name.clone()).or_insert(0);
        if *n > 0 {
            p.name = format!("{}_{}", p.name, n);
        }
        *n += 1;
    }
    if out.paths.is_empty() {
        return Err("no drawable shapes found".into());
    }
    Ok(out)
}

// =============================================================================================
// BVH
// =============================================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct BvhJoint {
    pub name: String,
    pub parent: Option<usize>,
    pub offset: [f32; 3],
    pub channels: Vec<String>,
    /// Index of this joint's first value in each motion row.
    pub first: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bvh {
    pub joints: Vec<BvhJoint>,
    pub frame_time: f32,
    pub rows: Vec<Vec<f32>>,
}

impl Bvh {
    pub fn joint(&self, name: &str) -> Option<usize> {
        self.joints.iter().position(|j| j.name == name)
    }

    /// Value of `channel` (e.g. `"Zrotation"`) for `joint` at time `t` seconds (linear interpolation).
    pub fn sample(&self, joint: usize, channel: &str, t: f32) -> Option<f32> {
        let j = self.joints.get(joint)?;
        let ci = j.channels.iter().position(|c| c.eq_ignore_ascii_case(channel))?;
        let idx = j.first + ci;
        let pos = (t / self.frame_time).clamp(0.0, (self.rows.len().saturating_sub(1)) as f32);
        let (a, b) = (pos.floor() as usize, (pos.ceil() as usize).min(self.rows.len() - 1));
        let f = pos - pos.floor();
        Some(self.rows[a][idx] * (1.0 - f) + self.rows[b][idx] * f)
    }
}

pub fn parse_bvh(text: &str) -> Result<Bvh, String> {
    if text.len() > 64 * 1024 * 1024 {
        return Err("BVH is larger than 64 MB".into());
    }
    let mut toks = text.split_whitespace().peekable();
    if toks.next() != Some("HIERARCHY") {
        return Err("BVH must start with HIERARCHY".into());
    }
    let mut joints: Vec<BvhJoint> = Vec::new();
    let mut stack: Vec<Option<usize>> = Vec::new(); // parent for the next joint; None for End Site scopes
    let mut next_val = 0usize;
    let mut pending: Option<Option<usize>> = None; // parent of the joint whose '{' we are waiting for
    let mut in_end = 0usize;
    loop {
        let t = toks.next().ok_or("BVH ended inside HIERARCHY")?;
        match t {
            "ROOT" | "JOINT" => {
                let name = toks.next().ok_or("joint without a name")?.to_string();
                let parent = if t == "ROOT" { None } else { stack.last().copied().flatten() };
                if joints.len() >= 1000 {
                    return Err("BVH has more than 1000 joints".into());
                }
                joints.push(BvhJoint { name, parent, offset: [0.0; 3], channels: vec![], first: 0 });
                pending = Some(Some(joints.len() - 1));
            }
            "End" => {
                toks.next(); // "Site"
                pending = Some(None);
                in_end += 1;
            }
            "{" => stack.push(pending.take().ok_or("unexpected '{'")?),
            "}" => {
                stack.pop().ok_or("unbalanced '}'")?;
                if stack.is_empty() {
                    // possible next ROOT (multiple roots are allowed); continue until MOTION
                }
            }
            "OFFSET" => {
                let v: Vec<f32> = (0..3).map(|_| toks.next().and_then(|x| x.parse().ok()).ok_or("bad OFFSET")).collect::<Result<_, _>>()?;
                if let Some(Some(j)) = stack.last() {
                    joints[*j].offset = [v[0], v[1], v[2]];
                }
            }
            "CHANNELS" => {
                let n: usize = toks.next().and_then(|x| x.parse().ok()).ok_or("bad CHANNELS count")?;
                if n > 6 {
                    return Err("a joint cannot have more than 6 channels".into());
                }
                let names: Vec<String> = (0..n).map(|_| toks.next().map(String::from).ok_or("missing channel name")).collect::<Result<_, _>>()?;
                if let Some(Some(j)) = stack.last() {
                    joints[*j].first = next_val;
                    joints[*j].channels = names;
                    next_val += n;
                }
            }
            "MOTION" => break,
            other => return Err(format!("unexpected token '{other}' in HIERARCHY")),
        }
    }
    let _ = in_end;
    if joints.is_empty() {
        return Err("BVH has no joints".into());
    }
    if toks.next() != Some("Frames:") {
        return Err("expected 'Frames:'".into());
    }
    let nframes: usize = toks.next().and_then(|x| x.parse().ok()).ok_or("bad frame count")?;
    if toks.next() != Some("Frame") || toks.next() != Some("Time:") {
        return Err("expected 'Frame Time:'".into());
    }
    let frame_time: f32 = toks.next().and_then(|x| x.parse().ok()).ok_or("bad frame time")?;
    if (frame_time.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || nframes == 0 || nframes > 200_000 {
        return Err("frame count or frame time out of range".into());
    }
    let mut rows = Vec::with_capacity(nframes);
    for r in 0..nframes {
        let row: Vec<f32> = (0..next_val)
            .map(|_| toks.next().and_then(|x| x.parse().ok()).ok_or_else(|| format!("motion data ends early (row {r})")))
            .collect::<Result<_, _>>()?;
        rows.push(row);
    }
    Ok(Bvh { joints, frame_time, rows })
}

/// Turn one rotation axis of BVH joints into bone-angle channels (2-D projection).
/// `pairs` = (joint name, bone name); `axis` e.g. `"Zrotation"`. Angles are relative to the
/// first motion frame, so the rig keeps its own rest pose.
pub fn bvh_to_channels(bvh: &Bvh, pairs: &[(String, String)], axis: &str, fps: f32, frames: u32, tol: f32) -> Result<Vec<Channel>, String> {
    let mut out = Vec::new();
    for (joint, bone) in pairs {
        let ji = bvh.joint(joint).ok_or_else(|| format!("BVH has no joint '{joint}'"))?;
        let first = bvh.sample(ji, axis, 0.0).ok_or_else(|| format!("joint '{joint}' has no channel '{axis}'"))?;
        let samples: Vec<f32> = (0..frames).map(|f| bvh.sample(ji, axis, f as f32 / fps).unwrap_or(first) - first).collect();
        out.push(record(bone, Prop::BoneAngle, &samples, tol));
    }
    Ok(out)
}

// =============================================================================================
// Mouth cues
// =============================================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct MouthCue {
    /// Frame number (Moho-switch style files) — fractional when converted from seconds.
    pub frame: f32,
    pub shape: String,
}

/// Parse `MohoSwitch1` files (`frame PHONEME` lines) or Rhubarb TSV (`seconds<TAB>shape`).
pub fn parse_mouth_cues(text: &str, fps: f32) -> Result<Vec<MouthCue>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("MohoSwitch") {
            continue;
        }
        let mut it = line.split_whitespace();
        let (a, b) = (it.next(), it.next());
        let (Some(a), Some(b)) = (a, b) else { return Err(format!("line {}: expected two columns", n + 1)) };
        let v: f32 = a.parse().map_err(|_| format!("line {}: bad time/frame '{a}'", n + 1))?;
        // Rhubarb prints seconds with a decimal point; Moho switch files use whole frames.
        let frame = if a.contains('.') { v * fps } else { v };
        out.push(MouthCue { frame, shape: b.to_string() });
    }
    if out.is_empty() {
        return Err("no mouth cues found".into());
    }
    out.sort_by(|x, y| x.frame.total_cmp(&y.frame));
    Ok(out)
}

/// Map a Papagayo/Preston-Blair phoneme or Rhubarb shape letter to OSATURA's five mouth
/// classes: 0 rest, 1 small, 2 open, 3 round, 4 wide.
pub fn mouth_class(shape: &str) -> usize {
    match shape {
        "rest" | "MBP" | "X" | "A" => 0,
        "etc" | "FV" | "L" | "B" | "G" | "H" => 1,
        "AI" | "D" => 2,
        "O" | "U" | "WQ" | "F" => 3,
        "E" | "C" => 4, // Papagayo "E" = wide; Rhubarb's rounded "E" is handled in `mouth_channel`
        _ => 1,
    }
}

/// Switch-layer exposure channel from cues. `mapping[class]` = child index of the switch layer.
pub fn mouth_channel(target: &str, cues: &[MouthCue], mapping: &[usize; 5], rhubarb: bool) -> Channel {
    let mut ex: Vec<(f32, usize)> = Vec::new();
    for c in cues {
        let class = if rhubarb && c.shape == "E" { 3 } else { mouth_class(&c.shape) };
        let idx = mapping[class];
        if ex.last().map(|l| l.1) != Some(idx) {
            ex.push((c.frame.round(), idx));
        }
    }
    Channel::exposure(target, &ex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_flags_parse_compactly() {
        assert_eq!(parse_numbers("1.5.5-2e1,3"), vec![1.5, 0.5, -20.0, 3.0]);
        let sp = parse_path_data("M0 0a5 5 0 1 0 10 0").unwrap();
        assert_eq!(sp.len(), 1);
        let sp2 = parse_path_data("M0 0a5 5 0 1010 0").unwrap(); // flags run into the number
        assert_eq!(sp2[0].0.len(), sp[0].0.len());
    }

    #[test]
    fn path_commands_relative_and_absolute() {
        let sp = parse_path_data("M10 10 l10 0 v10 h-10 z").unwrap();
        assert_eq!(sp.len(), 1);
        let (a, closed) = &sp[0];
        assert!(closed);
        let pts: Vec<(f32, f32)> = a.iter().map(|x| (x.pos.x, x.pos.y)).collect();
        assert_eq!(pts, vec![(10.0, 10.0), (20.0, 10.0), (20.0, 20.0), (10.0, 20.0)]);
        // implicit lineto after M, multiple subpaths
        let sp = parse_path_data("M0 0 5 5 M20 20 L30 30").unwrap();
        assert_eq!(sp.len(), 2);
        assert_eq!(sp[0].0.len(), 2);
    }

    #[test]
    fn curves_set_handles_and_arcs_end_exactly() {
        let sp = parse_path_data("M0 0 C 0 10 10 10 10 0 S 20 -10 20 0 Q 25 10 30 0 T 40 0").unwrap();
        let a = &sp[0].0;
        assert_eq!(a[0].handle_out, Some(Vec2::new(0.0, 10.0)));
        assert_eq!(a[1].handle_in, Some(Vec2::new(10.0, 10.0)));
        // S reflects the previous second control point (10,10) about (10,0) → (10,-10)
        assert_eq!(a[1].handle_out, Some(Vec2::new(10.0, -10.0)));
        let arc = parse_path_data("M0 0 A 10 10 0 0 1 20 0").unwrap();
        let last = arc[0].0.last().unwrap();
        assert!((last.pos.x - 20.0).abs() < 1e-4 && last.pos.y.abs() < 1e-4);
        assert!(arc[0].0.len() >= 3, "a half circle needs two cubic segments");
    }

    #[test]
    fn bad_path_data_is_an_error() {
        assert!(parse_path_data("10 10").is_err());
        assert!(parse_path_data("M 1").is_err());
        assert!(parse_path_data("M0 0 X 1").is_err());
        assert!(parse_path_data("M0 0 A 1 1 0 2 0 5 5").is_err());
    }

    const SVG: &str = r##"<?xml version="1.0"?>
    <!-- test --><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50">
      <defs><linearGradient id="g"><stop offset="0"/></linearGradient></defs>
      <g transform="translate(10,5)" fill="#f00">
        <rect id="box" x="0" y="0" width="20" height="10"/>
        <g transform="scale(2)"><circle cx="5" cy="5" r="2" fill="blue" stroke="#000" stroke-width="1"/></g>
      </g>
      <path id="p" d="M0 0 L5 0 L5 5 Z" fill="url(#g)"/>
      <polyline points="0,0 4,4 8,0" fill="none" stroke="green"/>
      <text x="1" y="1">hi</text>
    </svg>"##;

    #[test]
    fn svg_import_groups_transforms_styles_and_warnings() {
        let r = import_svg(SVG).unwrap();
        assert_eq!((r.width, r.height), (100.0, 50.0));
        let names: Vec<_> = r.paths.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["box", "circle1", "p", "polyline3"]);
        let boxp = &r.paths[0];
        assert_eq!(boxp.anchors[0].pos, Vec2::new(10.0, 5.0), "group translate applied");
        assert_eq!(boxp.anchors[2].pos, Vec2::new(30.0, 15.0));
        let circ = &r.paths[1];
        let (mn, mx) = circ.bounds().unwrap();
        assert!((mx.x - mn.x - 8.0).abs() < 0.2, "radius 2 scaled by 2 ⇒ diameter 8, got {}", mx.x - mn.x);
        assert!((mn.x - (10.0 + 6.0)).abs() < 0.2);
        assert!(circ.stroke.is_some() && circ.fill.is_some());
        assert!(r.paths[3].fill.is_none() && r.paths[3].stroke.is_some());
        assert!(r.warnings.iter().any(|w| w.contains("url(#g)")), "{:?}", r.warnings);
        assert!(r.warnings.iter().any(|w| w.contains("<text>")), "{:?}", r.warnings);
    }

    #[test]
    fn svg_import_rejects_garbage_and_huge_input() {
        assert!(import_svg("<svg><rect/></svg>").is_err(), "nothing drawable");
        assert!(import_svg("<svg><path d='M0 0 L1 1'").is_err(), "unterminated tag");
        assert!(import_svg(&"x".repeat(MAX_SVG_BYTES + 1)).is_err());
    }

    #[test]
    fn transforms_compose_in_order() {
        let m = parse_transform("translate(10 0) rotate(90)").unwrap();
        let p = m.apply(Vec2::new(1.0, 0.0));
        assert!((p.x - 10.0).abs() < 1e-5 && (p.y - 1.0).abs() < 1e-5);
        let m = parse_transform("rotate(90 5 5)").unwrap();
        let q = m.apply(Vec2::new(10.0, 5.0));
        assert!((q.x - 5.0).abs() < 1e-4 && (q.y - 10.0).abs() < 1e-4);
        assert!(parse_transform("wiggle(3)").is_err());
    }

    const BVH: &str = "HIERARCHY
ROOT Hips
{
  OFFSET 0 0 0
  CHANNELS 3 Xposition Yposition Zrotation
  JOINT Arm
  {
    OFFSET 10 0 0
    CHANNELS 1 Zrotation
    End Site
    {
      OFFSET 5 0 0
    }
  }
}
MOTION
Frames: 3
Frame Time: 0.5
0 0 0 10
0 0 0 40
0 0 0 70
";

    #[test]
    fn bvh_parses_and_samples() {
        let b = parse_bvh(BVH).unwrap();
        assert_eq!(b.joints.len(), 2);
        assert_eq!(b.joints[1].parent, Some(0));
        assert_eq!(b.joints[1].first, 3);
        assert_eq!(b.rows.len(), 3);
        let arm = b.joint("Arm").unwrap();
        assert_eq!(b.sample(arm, "Zrotation", 0.0), Some(10.0));
        assert_eq!(b.sample(arm, "Zrotation", 0.25), Some(25.0));
        assert_eq!(b.sample(arm, "Zrotation", 99.0), Some(70.0), "clamped");
        assert_eq!(b.sample(arm, "Xrotation", 0.0), None);
    }

    #[test]
    fn bvh_channels_are_relative_to_first_frame() {
        let b = parse_bvh(BVH).unwrap();
        let ch = bvh_to_channels(&b, &[("Arm".into(), "arm_up".into())], "Zrotation", 4.0, 5, 0.01).unwrap();
        assert_eq!(ch.len(), 1);
        if let crate::project::Source::Keys { track } = &ch[0].source {
            assert_eq!(track.eval(0.0), Some(0.0));
            assert!((track.eval(4.0).unwrap() - 60.0).abs() < 0.5);
        } else {
            panic!("keys expected");
        }
        assert!(bvh_to_channels(&b, &[("Nope".into(), "x".into())], "Zrotation", 24.0, 3, 0.1).is_err());
    }

    #[test]
    fn bvh_errors() {
        assert!(parse_bvh("nonsense").is_err());
        assert!(parse_bvh(&BVH.replace("Frames: 3", "Frames: 9")).is_err(), "data ends early");
        assert!(parse_bvh(&BVH.replace("Frame Time: 0.5", "Frame Time: 0")).is_err());
        assert!(parse_bvh(&BVH.replace("CHANNELS 1", "CHANNELS 9")).is_err());
    }

    #[test]
    fn mouth_cues_moho_and_rhubarb() {
        let moho = "MohoSwitch1\n1 rest\n5 AI\n9 MBP\n12 O\n";
        let cues = parse_mouth_cues(moho, 24.0).unwrap();
        assert_eq!(cues.len(), 4);
        let ch = mouth_channel("mouth", &cues, &[0, 1, 2, 3, 4], false);
        if let crate::project::Source::Keys { track } = &ch.source {
            assert_eq!(track.eval(6.0), Some(2.0));
            assert_eq!(track.eval(13.0), Some(3.0));
        }
        let rb = "0.00\tX\n0.50\tD\n1.00\tE\n";
        let cues = parse_mouth_cues(rb, 24.0).unwrap();
        assert_eq!(cues[1].frame, 12.0);
        let ch = mouth_channel("mouth", &cues, &[0, 1, 2, 3, 4], true);
        if let crate::project::Source::Keys { track } = &ch.source {
            assert_eq!(track.eval(25.0), Some(3.0), "Rhubarb E is the rounded shape");
        }
        assert!(parse_mouth_cues("", 24.0).is_err());
        assert!(parse_mouth_cues("5\n", 24.0).is_err());
        assert!(parse_mouth_cues("abc rest\n", 24.0).is_err());
    }
}
