// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Size-targeted export (roadmap I-10): "make this export fit in N bytes, as good as it can be".
//!
//! Own design, textbook parts only:
//!
//! * **Quality ladder.** Every format has a few *knobs* (axes), each an ordered list of values from the
//!   highest quality to the lowest (number precision, path simplification tolerance, key reduction
//!   tolerance, frame decimation, resolution scale, palette size, dithering, lossy frame deltas,
//!   VP9 CRF). The axes are merged into one *chain* of rungs, each rung one step lower than the last
//!   (balanced by normalised position, so no knob is exhausted before the others have moved).
//! * **Search.** Size is expected to fall along the chain, so the best fitting rung is found by
//!   bisection on the *measured* byte count of the produced file. The probes are checked for
//!   monotonicity; if they are not monotone the search falls back to a linear scan. A coordinate
//!   descent (raise one knob one step, lower another one step) then polishes the answer, and an
//!   exhaustive grid is available (`Search::Grid`) for the exact optimum on the grid.
//! * **Quality.** SSIM (`percep`, Wang et al. 2004 with a Gaussian window) between the render of the
//!   original project and a render of the *reduced* export, on `N` sampled frames. GIF, APNG and WebM
//!   are decoded from the produced bytes. Lottie is read back from the produced JSON by a small
//!   reader (the Lottie importer skips animated paths, so it cannot be used) and drawn with the
//!   engine's rasteriser. HTML pages cannot be rasterised here (no SVG rasteriser exists in the
//!   workspace), so the quality of an HTML rung is *modelled*: the same reductions are applied to the
//!   evaluated frame data and that is rasterised. The project format is re-parsed and rendered by
//!   the engine.
//! * **Budget.** The budgeted quantity is the raw file size or its gzip (level 9) size plus a 1 % margin
//!   and 32 bytes ([`gzip_budget_len`], so system `gzip -9` / `-6` output also fits); both raw and plain
//!   gzip sizes are always reported.
//!
//! Clean-room notes: median-cut colour quantisation (Heckbert 1982), Floyd-Steinberg error diffusion
//! (1976), Ramer-Douglas-Peucker / Schneider least-squares Bezier fitting for the reductions,
//! bisection and coordinate descent for the search. Nothing here copies another tool's algorithm
//! or file layout.

use crate::anim::Track;
use crate::engine::{Engine, Frame, NodeKind};
use crate::math::{Affine2, Vec2};
use crate::path::{Anchor, VPath};
use crate::percep::{self, Img};
use crate::project::{Project, Source};
use crate::raster::{pixmap_from_frame, render_rgba, to_straight_rgba, RasterOptions};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{Read, Write};

pub const SCHEMA: &str = "osatura.sizefit/1";

// ------------------------------------------------------------------------------------------
// Formats, measure, settings, axes
// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Format {
    Lottie,
    Html,
    Gif,
    Apng,
    Webm,
    /// OSATURA's own project JSON (where key reduction acts on the real keys).
    Project,
}

impl Format {
    pub fn parse(s: &str) -> Result<Format, String> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "lottie" => Format::Lottie,
            "html" | "htm" => Format::Html,
            "gif" => Format::Gif,
            "apng" => Format::Apng,
            "webm" => Format::Webm,
            "project" => Format::Project,
            "svg" => return Err("format 'svg' is not supported: there is no animated SVG exporter (use html, which embeds SVG frames)".into()),
            "gltf" | "glb" => return Err("format 'gltf' is not supported: a glTF export has no size ladder here (skinned geometry is not re-quantised)".into()),
            o => return Err(format!("unknown format '{o}' (lottie, html, gif, apng, webm, project)")),
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Format::Lottie => "lottie",
            Format::Html => "html",
            Format::Gif => "gif",
            Format::Apng => "apng",
            Format::Webm => "webm",
            Format::Project => "project",
        }
    }
    /// Guess from an output path (`.json` means Lottie, as for `export`).
    pub fn from_path(path: &str) -> Option<Format> {
        let ext = path.rsplit('.').next()?.to_ascii_lowercase();
        Some(match ext.as_str() {
            "json" => Format::Lottie,
            "html" | "htm" => Format::Html,
            "gif" => Format::Gif,
            "apng" => Format::Apng,
            "webm" => Format::Webm,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Measure {
    Raw,
    Gzip,
}

impl Measure {
    pub fn parse(s: &str) -> Result<Measure, String> {
        match s {
            "raw" => Ok(Measure::Raw),
            "gzip" => Ok(Measure::Gzip),
            o => Err(format!("unknown --measure '{o}' (raw or gzip)")),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Measure::Raw => "raw",
            Measure::Gzip => "gzip",
        }
    }
}

/// Parse "500KB", "1.5MB", "64000", "200 kib" (K = 1000, KiB = 1024).
pub fn parse_size(s: &str) -> Result<usize, String> {
    let t = s.trim().to_ascii_lowercase().replace(' ', "");
    let split = t.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(t.len());
    let (num, unit) = t.split_at(split);
    let n: f64 = num.parse().map_err(|_| format!("cannot read a size from '{s}' (try 500KB)"))?;
    let mult = match unit {
        "" | "b" => 1.0,
        "k" | "kb" => 1000.0,
        "m" | "mb" => 1e6,
        "g" | "gb" => 1e9,
        "kib" => 1024.0,
        "mib" => 1048576.0,
        o => return Err(format!("unknown size unit '{o}' in '{s}'")),
    };
    let v = n * mult;
    if !(v.is_finite() && (1.0..4e12).contains(&v)) {
        return Err(format!("size '{s}' is out of range"));
    }
    Ok(v.round() as usize)
}

/// Every reduction knob. Which fields matter depends on the format (see [`axes`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// Decimals kept in coordinates (6 = effectively lossless for f32 data).
    pub decimals: u8,
    /// Path simplification tolerance, stage units (0 = off).
    pub path_eps: f32,
    /// Key reduction tolerance in value units (0 = off).
    pub key_tol: f32,
    /// Remove layers that never draw anything (verified by rendering).
    pub drop_dead: bool,
    /// Keep one frame in `frame_step`.
    pub frame_step: u32,
    /// Resolution scale of raster formats.
    pub scale: f32,
    /// Palette size (GIF, APNG); 0 = truecolour APNG.
    pub colors: u16,
    pub dither: bool,
    /// Lossy GIF frame delta: pixels within this many levels of the previous frame are not redrawn.
    pub delta_tol: u8,
    /// VP9 constant-quality value (higher = smaller).
    pub crf: u8,
}

impl Settings {
    pub fn full() -> Settings {
        Settings { decimals: 6, path_eps: 0.0, key_tol: 0.0, drop_dead: true, frame_step: 1, scale: 1.0, colors: 256, dither: false, delta_tol: 0, crf: 24 }
    }
    pub fn to_json(&self) -> Value {
        json!({"decimals": self.decimals, "path_eps": self.path_eps, "key_tol": self.key_tol, "drop_dead": self.drop_dead,
            "frame_step": self.frame_step, "scale": self.scale, "colors": self.colors, "dither": self.dither,
            "delta_tol": self.delta_tol, "crf": self.crf})
    }
}

/// One knob: values from best quality to lowest quality.
pub struct Axis {
    pub name: &'static str,
    pub values: Vec<f64>,
    set: fn(&mut Settings, f64),
    /// False for a knob that is not a quality ladder (its values trade size against look, not in one
    /// direction): it stays at its first value on the chain and is only explored by descent and grid.
    pub in_chain: bool,
}

fn ax(name: &'static str, values: &[f64], set: fn(&mut Settings, f64)) -> Axis {
    Axis { name, values: values.to_vec(), set, in_chain: true }
}

fn ax_free(name: &'static str, values: &[f64], set: fn(&mut Settings, f64)) -> Axis {
    Axis { name, values: values.to_vec(), set, in_chain: false }
}

const SCALES: [f64; 8] = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3];

/// The knobs of a format, in tie-break order (least visible first).
pub fn axes(f: Format) -> Vec<Axis> {
    let dec = |v: &[f64]| ax("decimals", v, |s, x| s.decimals = x as u8);
    let eps = ax("path_eps", &[0.0, 0.05, 0.1, 0.2, 0.4, 0.8, 1.5], |s, x| s.path_eps = x as f32);
    let kt = |v: &[f64]| ax("key_tol", v, |s, x| s.key_tol = x as f32);
    let step = |v: &[f64]| ax("frame_step", v, |s, x| s.frame_step = x as u32);
    let scale = ax("scale", &SCALES, |s, x| s.scale = x as f32);
    match f {
        Format::Lottie => vec![
            dec(&[6.0, 4.0, 3.0, 2.0, 1.0, 0.0]),
            eps,
            kt(&[0.0, 0.02, 0.05, 0.1, 0.2, 0.4, 0.8, 1.5, 3.0]),
            step(&[1.0, 2.0, 3.0, 4.0, 6.0]),
        ],
        Format::Html => vec![dec(&[2.0, 1.0, 0.0]), eps, step(&[1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0])],
        Format::Gif => vec![
            ax("colors", &[256.0, 192.0, 128.0, 96.0, 64.0, 48.0, 32.0, 24.0, 16.0, 8.0], |s, x| s.colors = x as u16),
            ax_free("dither", &[0.0, 1.0], |s, x| s.dither = x > 0.5),
            ax("delta_tol", &[0.0, 2.0, 4.0, 8.0, 16.0], |s, x| s.delta_tol = x as u8),
            step(&[1.0, 2.0, 3.0, 4.0, 6.0]),
            scale,
        ],
        Format::Apng => vec![
            ax("colors", &[0.0, 256.0, 128.0, 64.0, 32.0, 16.0, 8.0], |s, x| s.colors = x as u16),
            ax_free("dither", &[0.0, 1.0], |s, x| s.dither = x > 0.5),
            step(&[1.0, 2.0, 3.0, 4.0, 6.0]),
            scale,
        ],
        Format::Webm => vec![
            ax("crf", &[24.0, 28.0, 32.0, 36.0, 40.0, 44.0, 48.0, 52.0, 56.0, 60.0, 63.0], |s, x| s.crf = x as u8),
            step(&[1.0, 2.0, 3.0, 4.0, 6.0]),
            scale,
        ],
        Format::Project => vec![
            dec(&[6.0, 5.0, 4.0, 3.0, 2.0, 1.0]),
            kt(&[0.0, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0]),
        ],
    }
}

/// Apply a position (one rung index per axis) to the full-quality settings.
pub fn settings_at(f: Format, pos: &[usize]) -> Settings {
    let mut s = Settings::full();
    for (a, &p) in axes(f).iter().zip(pos) {
        (a.set)(&mut s, a.values[p.min(a.values.len() - 1)]);
    }
    s
}

/// The chain of positions from the best rung to the floor: each step lowers exactly one axis by one rung,
/// steps ordered by the axis's normalised progress (ties: axis order).
pub fn chain(f: Format) -> Vec<Vec<usize>> {
    let ax = axes(f);
    let mut steps: Vec<(f64, usize)> = Vec::new();
    for (i, a) in ax.iter().enumerate().filter(|(_, a)| a.in_chain) {
        let n = a.values.len();
        for j in 1..n {
            steps.push((j as f64 / (n - 1) as f64, i));
        }
    }
    steps.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut pos = vec![0usize; ax.len()];
    let mut out = vec![pos.clone()];
    for (_, i) in steps {
        pos[i] += 1;
        out.push(pos.clone());
    }
    out
}

// ------------------------------------------------------------------------------------------
// Small utilities
// ------------------------------------------------------------------------------------------

pub fn gzip_bytes(data: &[u8]) -> Vec<u8> {
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    let _ = e.write_all(data);
    e.finish().unwrap_or_default()
}

pub fn gzip_len(data: &[u8]) -> usize {
    gzip_bytes(data).len()
}

/// The size the budget is checked against with `--measure gzip`: the larger of the zlib level-6 (the
/// default of gzip(1)) and level-9 gzip sizes, plus a 1 % safety margin and 32 bytes of header slack
/// (gzip(1) stores a file name and time stamp). The system gzip at `-9` then produces a file within
/// the budget. gzip(1) at its default `-6` (and any faster level) is a weaker compressor than zlib and
/// can come out 5-9 % larger than the level-9 size, which a 1 % margin does not cover: budget for the
/// level you serve with, or leave headroom. The plain level-9 size is what is reported as `gzip_bytes`.
pub fn gzip_budget_len(data: &[u8]) -> usize {
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(6));
    let _ = e.write_all(data);
    let l6 = e.finish().map(|v| v.len()).unwrap_or(0);
    let n = gzip_len(data).max(l6);
    n + n / 100 + 32
}

/// Round to `d` decimals; `d >= 9` keeps the value; never returns -0.
pub fn round_dec(x: f64, d: u8) -> f64 {
    if d >= 9 || !x.is_finite() {
        return x;
    }
    let m = 10f64.powi(d as i32);
    let r = (x * m).round() / m;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

fn num(x: f64) -> Value {
    if !x.is_finite() {
        return Value::from(0);
    }
    if x.fract() == 0.0 && x.abs() < 1e15 {
        Value::from(x as i64)
    } else {
        Value::from(x)
    }
}

/// Shortest decimal text of `x` rounded to `d` decimals (no trailing zeros, no "-0").
fn fmt_dec(x: f64, d: u8) -> String {
    let r = round_dec(x, d);
    let mut s = format!("{:.*}", d as usize, r);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".into();
    }
    s
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Bilinear resize of straight RGBA (premultiplied for the blend so edges stay clean).
pub fn resize_rgba(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u8> {
    if sw == dw && sh == dh {
        return src.to_vec();
    }
    let mut out = vec![0u8; dw * dh * 4];
    let (fx, fy) = (sw as f64 / dw as f64, sh as f64 / dh as f64);
    for y in 0..dh {
        let sy = ((y as f64 + 0.5) * fy - 0.5).clamp(0.0, (sh - 1) as f64);
        let (y0, ty) = (sy.floor() as usize, sy - sy.floor());
        let y1 = (y0 + 1).min(sh - 1);
        for x in 0..dw {
            let sx = ((x as f64 + 0.5) * fx - 0.5).clamp(0.0, (sw - 1) as f64);
            let (x0, tx) = (sx.floor() as usize, sx - sx.floor());
            let x1 = (x0 + 1).min(sw - 1);
            let mut acc = [0.0f64; 4];
            for (yy, wy) in [(y0, 1.0 - ty), (y1, ty)] {
                for (xx, wx) in [(x0, 1.0 - tx), (x1, tx)] {
                    let i = (yy * sw + xx) * 4;
                    let w = wx * wy;
                    let a = src[i + 3] as f64;
                    acc[0] += src[i] as f64 * a * w;
                    acc[1] += src[i + 1] as f64 * a * w;
                    acc[2] += src[i + 2] as f64 * a * w;
                    acc[3] += a * w;
                }
            }
            let o = (y * dw + x) * 4;
            if acc[3] > 0.0 {
                out[o] = (acc[0] / acc[3]).round().clamp(0.0, 255.0) as u8;
                out[o + 1] = (acc[1] / acc[3]).round().clamp(0.0, 255.0) as u8;
                out[o + 2] = (acc[2] / acc[3]).round().clamp(0.0, 255.0) as u8;
                out[o + 3] = acc[3].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

// ------------------------------------------------------------------------------------------
// Path simplification: greedy anchor removal with a least-squares Bezier refit
// ------------------------------------------------------------------------------------------

/// A path vertex with *relative* tangents (the Lottie convention).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PNode {
    pub p: [f64; 2],
    pub i: [f64; 2],
    pub o: [f64; 2],
}

fn add2(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}
fn sub2(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn dot2(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn len2(a: [f64; 2]) -> f64 {
    dot2(a, a).sqrt()
}

fn cubic_at(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        b0 * c[0][0] + b1 * c[1][0] + b2 * c[2][0] + b3 * c[3][0],
        b0 * c[0][1] + b1 * c[1][1] + b2 * c[2][1] + b3 * c[3][1],
    ]
}

fn seg_ctrl(a: &PNode, b: &PNode) -> [[f64; 2]; 4] {
    [a.p, add2(a.p, a.o), add2(b.p, b.i), b.p]
}

fn dist_pt_seg(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = sub2(b, a);
    let l2 = dot2(ab, ab);
    let t = if l2 < 1e-18 { 0.0 } else { (dot2(sub2(p, a), ab) / l2).clamp(0.0, 1.0) };
    len2(sub2(p, [a[0] + ab[0] * t, a[1] + ab[1] * t]))
}

fn dist_pt_poly(p: [f64; 2], poly: &[[f64; 2]]) -> f64 {
    poly.windows(2).map(|w| dist_pt_seg(p, w[0], w[1])).fold(f64::INFINITY, f64::min)
}

fn unit_or(v: [f64; 2], fallbacks: &[[f64; 2]]) -> [f64; 2] {
    for c in std::iter::once(&v).chain(fallbacks) {
        let l = len2(*c);
        if l > 1e-9 {
            return [c[0] / l, c[1] / l];
        }
    }
    [0.0, 0.0]
}

const FIT_K: usize = 8;

/// Replace `a - k - b` by one cubic from `a` to `b`: returns the new outgoing tangent of `a`, the new
/// incoming tangent of `b` (both relative) and the symmetric sampled Hausdorff distance between the
/// original two segments and the replacement (stage units).
fn merge_fit(a: &PNode, k: &PNode, b: &PNode) -> ([f64; 2], [f64; 2], f64) {
    let s1 = seg_ctrl(a, k);
    let s2 = seg_ctrl(k, b);
    let mut pts: Vec<[f64; 2]> = Vec::with_capacity(2 * FIT_K + 1);
    for j in 0..=FIT_K {
        pts.push(cubic_at(&s1, j as f64 / FIT_K as f64));
    }
    for j in 1..=FIT_K {
        pts.push(cubic_at(&s2, j as f64 / FIT_K as f64));
    }
    let zero = [0.0, 0.0];
    let straight = a.o == zero && k.i == zero && k.o == zero && b.i == zero;
    let (o_new, i_new) = if straight {
        (zero, zero)
    } else {
        // chord-length parameters
        let mut u = vec![0.0f64; pts.len()];
        for j in 1..pts.len() {
            u[j] = u[j - 1] + len2(sub2(pts[j], pts[j - 1]));
        }
        let total = u[pts.len() - 1];
        let chord = len2(sub2(b.p, a.p));
        if total < 1e-12 || chord < 1e-12 {
            (zero, zero)
        } else {
            let t1 = unit_or(sub2(s1[1], s1[0]), &[sub2(s1[2], s1[0]), sub2(s1[3], s1[0])]);
            let t2 = unit_or(sub2(s2[2], s2[3]), &[sub2(s2[1], s2[3]), sub2(s2[0], s2[3])]);
            let (mut c11, mut c12, mut c22, mut x1, mut x2) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for j in 0..pts.len() {
                let t = u[j] / total;
                let w = 1.0 - t;
                let (b0, b1, b2, b3) = (w * w * w, 3.0 * w * w * t, 3.0 * w * t * t, t * t * t);
                let a1 = [t1[0] * b1, t1[1] * b1];
                let a2 = [t2[0] * b2, t2[1] * b2];
                c11 += dot2(a1, a1);
                c12 += dot2(a1, a2);
                c22 += dot2(a2, a2);
                let base = [
                    a.p[0] * (b0 + b1) + b.p[0] * (b2 + b3),
                    a.p[1] * (b0 + b1) + b.p[1] * (b2 + b3),
                ];
                let r = sub2(pts[j], base);
                x1 += dot2(a1, r);
                x2 += dot2(a2, r);
            }
            let det = c11 * c22 - c12 * c12;
            let (mut al1, mut al2) = (chord / 3.0, chord / 3.0);
            if det.abs() > 1e-12 * (c11 * c22).abs().max(1e-30) {
                let (s, e) = ((x1 * c22 - x2 * c12) / det, (c11 * x2 - c12 * x1) / det);
                if s.is_finite() && e.is_finite() && s > 1e-6 * chord && e > 1e-6 * chord && s < 4.0 * total && e < 4.0 * total {
                    al1 = s;
                    al2 = e;
                }
            }
            ([t1[0] * al1, t1[1] * al1], [t2[0] * al2, t2[1] * al2])
        }
    };
    let na = PNode { o: o_new, ..*a };
    let nb = PNode { i: i_new, ..*b };
    let merged = seg_ctrl(&na, &nb);
    let mp: Vec<[f64; 2]> = (0..=24).map(|j| cubic_at(&merged, j as f64 / 24.0)).collect();
    let e1 = pts.iter().map(|&p| dist_pt_poly(p, &mp)).fold(0.0, f64::max);
    let e2 = mp.iter().map(|&p| dist_pt_poly(p, &pts)).fold(0.0, f64::max);
    (o_new, i_new, e1.max(e2))
}

/// Remove anchors from a set of equally shaped paths (`frames[f][vertex]`, one shared `closed` flag)
/// while the sampled geometric deviation stays within `eps` in *every* frame. All frames lose the same
/// vertices, so the topology stays consistent over time. Returns the number of removed vertices.
pub fn simplify_paths(frames: &mut [Vec<PNode>], closed: bool, eps: f64) -> usize {
    let Some(n) = frames.first().map(|f| f.len()) else { return 0 };
    if eps <= 0.0 || frames.iter().any(|f| f.len() != n) {
        return 0;
    }
    let min_keep = if closed { 3 } else { 2 };
    let mut alive: Vec<usize> = (0..n).collect();
    let mut cost: HashMap<usize, f64> = HashMap::new();
    let mut removed = 0;
    while alive.len() > min_keep {
        let m = alive.len();
        let range = if closed { 0..m } else { 1..m - 1 };
        let mut best: Option<(f64, usize)> = None;
        for j in range {
            let id = alive[j];
            let c = *cost.entry(id).or_insert_with(|| {
                let (pi, ni) = (alive[(j + m - 1) % m], alive[(j + 1) % m]);
                frames.iter().map(|f| merge_fit(&f[pi], &f[id], &f[ni]).2).fold(0.0, f64::max)
            });
            if best.is_none_or(|(bc, _)| c < bc) {
                best = Some((c, j));
            }
        }
        let Some((c, j)) = best else { break };
        if c > eps {
            break;
        }
        let (pi, id, ni) = (alive[(j + m - 1) % m], alive[j], alive[(j + 1) % m]);
        for f in frames.iter_mut() {
            let (o, i, _) = merge_fit(&f[pi], &f[id], &f[ni]);
            f[pi].o = o;
            f[ni].i = i;
        }
        alive.remove(j);
        cost.remove(&id);
        cost.remove(&pi);
        cost.remove(&ni);
        removed += 1;
    }
    if removed > 0 {
        let keep: std::collections::HashSet<usize> = alive.into_iter().collect();
        for f in frames.iter_mut() {
            let mut k = 0;
            f.retain(|_| {
                let r = keep.contains(&k);
                k += 1;
                r
            });
        }
    }
    removed
}

/// Engine path -> relative-tangent nodes (in the path's own space).
pub fn pnodes_of(path: &VPath) -> Vec<PNode> {
    path.anchors
        .iter()
        .map(|a| {
            let (hi, ho) = (a.hin(), a.hout());
            PNode {
                p: [a.pos.x as f64, a.pos.y as f64],
                i: [(hi.x - a.pos.x) as f64, (hi.y - a.pos.y) as f64],
                o: [(ho.x - a.pos.x) as f64, (ho.y - a.pos.y) as f64],
            }
        })
        .collect()
}

pub fn anchors_of(nodes: &[PNode]) -> Vec<Anchor> {
    nodes
        .iter()
        .map(|n| {
            let pos = Vec2::new(n.p[0] as f32, n.p[1] as f32);
            let h = |r: [f64; 2]| if r == [0.0, 0.0] { None } else { Some(Vec2::new((n.p[0] + r[0]) as f32, (n.p[1] + r[1]) as f32)) };
            Anchor { pos, handle_in: h(n.i), handle_out: h(n.o), weights: Vec::new() }
        })
        .collect()
}

/// Simplify one evaluated path with tolerance `eps` (path units). Paths with a width profile are left alone.
pub fn simplify_vpath(path: &mut VPath, eps: f64) -> usize {
    if eps <= 0.0 || path.anchors.len() < 3 || path.stroke.as_ref().is_some_and(|s| !s.width_profile.is_empty()) {
        return 0;
    }
    let mut fr = vec![pnodes_of(path)];
    let r = simplify_paths(&mut fr, path.closed, eps);
    if r > 0 {
        path.anchors = anchors_of(&fr[0]);
    }
    r
}

// ------------------------------------------------------------------------------------------
// Key reduction (project tracks, exact error bound in value units)
// ------------------------------------------------------------------------------------------

/// Frames at which a track is compared: every integer frame of the timeline and every key frame.
fn track_samples(t: &Track, frames: u32) -> Vec<f32> {
    let mut v: Vec<f32> = (0..frames.max(1)).map(|f| f as f32).collect();
    v.extend(t.keys.iter().map(|k| k.frame));
    v.sort_by(|a, b| a.total_cmp(b));
    v.dedup();
    v
}

/// Largest |reduced - original| over the integer frames of the timeline and all original key frames.
pub fn track_deviation(orig: &Track, red: &Track, frames: u32) -> f64 {
    track_samples(orig, frames)
        .into_iter()
        .map(|f| match (orig.eval(f), red.eval(f)) {
            (Some(a), Some(b)) => (a as f64 - b as f64).abs(),
            _ => 0.0,
        })
        .fold(0.0, f64::max)
}

/// Drop keys while `track_deviation <= tol`. First and last key are kept, easings of kept keys are
/// unchanged. Top-down Ramer-Douglas-Peucker on key indices followed by a repair loop that re-inserts
/// the removed key nearest to the worst sample until the bound holds for the *whole* track (it
/// terminates: at worst every key comes back and the deviation is 0).
pub fn reduce_track(t: &Track, tol: f32, frames: u32) -> Track {
    let n = t.keys.len();
    if n < 3 || tol < 0.0 || !tol.is_finite() {
        return t.clone();
    }
    let tol = tol as f64;
    let samples = track_samples(t, frames);
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    let sub = |lo: usize, hi: usize| Track { keys: vec![t.keys[lo], t.keys[hi]], extrapolate: t.extrapolate };
    let mut stack = vec![(0usize, n - 1)];
    while let Some((lo, hi)) = stack.pop() {
        if hi - lo < 2 {
            continue;
        }
        let cand = sub(lo, hi);
        let (f0, f1) = (t.keys[lo].frame, t.keys[hi].frame);
        // worst sample inside the span; split at the original key nearest to it
        let mut worst = (0.0f64, f0);
        for &f in samples.iter().filter(|&&f| f > f0 && f < f1) {
            let e = (t.eval(f).unwrap_or(0.0) as f64 - cand.eval(f).unwrap_or(0.0) as f64).abs();
            if e > worst.0 {
                worst = (e, f);
            }
        }
        if worst.0 <= tol {
            continue;
        }
        let split = (lo + 1..hi).min_by(|&a, &b| (t.keys[a].frame - worst.1).abs().total_cmp(&(t.keys[b].frame - worst.1).abs())).unwrap();
        keep[split] = true;
        stack.push((lo, split));
        stack.push((split, hi));
    }
    let build = |keep: &[bool]| Track { keys: t.keys.iter().zip(keep).filter(|(_, &k)| k).map(|(k, _)| *k).collect(), extrapolate: t.extrapolate };
    let mut red = build(&keep);
    loop {
        // find the worst sample of the whole reduced track
        let mut worst = (0.0f64, 0.0f32);
        for &f in &samples {
            let e = (t.eval(f).unwrap_or(0.0) as f64 - red.eval(f).unwrap_or(0.0) as f64).abs();
            if e > worst.0 {
                worst = (e, f);
            }
        }
        if worst.0 <= tol {
            return red;
        }
        let Some(add) = (0..n).filter(|&i| !keep[i]).min_by(|&a, &b| (t.keys[a].frame - worst.1).abs().total_cmp(&(t.keys[b].frame - worst.1).abs())) else {
            return red;
        };
        keep[add] = true;
        red = build(&keep);
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyReduction {
    pub channels: usize,
    pub keys_before: usize,
    pub keys_after: usize,
    pub max_deviation: f64,
}

/// Reduce every keyed channel of `p` in place.
pub fn reduce_project_keys(p: &mut Project, tol: f32) -> KeyReduction {
    let frames = p.frames;
    let mut r = KeyReduction::default();
    for c in &mut p.channels {
        if let Source::Keys { track } = &mut c.source {
            r.channels += 1;
            r.keys_before += track.keys.len();
            let red = reduce_track(track, tol, frames);
            r.max_deviation = r.max_deviation.max(track_deviation(track, &red, frames));
            r.keys_after += red.keys.len();
            *track = red;
        }
    }
    r
}

// ------------------------------------------------------------------------------------------
// Dead layers
// ------------------------------------------------------------------------------------------

fn mask_targets(ls: &[crate::project::Layer], out: &mut std::collections::HashSet<String>) {
    for l in ls {
        if let Some(m) = &l.mask {
            out.insert(m.layer.clone());
        }
        mask_targets(l.children(), out);
    }
}

/// Layers that never draw anything: hidden (or fully transparent, or under a hidden ancestor) in every
/// frame, not used as a mask source, not a child of a switch (its index would shift). Each removal is
/// accepted only if the project still validates, and the final project must render pixel-identical
/// frames at quarter resolution on up to 12 sample frames - otherwise nothing is removed.
pub fn drop_dead_layers(p: &Project) -> (Project, Vec<String>) {
    let unchanged = || (p.clone(), Vec::new());
    let Ok(e0) = Engine::new(p.clone()) else { return unchanged() };
    let frames: Vec<Frame> = (0..p.frames).map(|f| e0.evaluate(f as f32)).collect();
    let n = frames.first().map_or(0, |f| f.nodes.len());
    let mut alive = vec![false; n];
    for f in &frames {
        fn walk(f: &Frame, id: usize, op: f32, vis: bool, alive: &mut [bool]) {
            let nd = &f.nodes[id];
            let vis = vis && nd.visible;
            let op = op * nd.opacity;
            if vis && op > 0.0 {
                alive[id] = true;
            }
            if vis {
                for &c in &nd.children {
                    walk(f, c, op, vis, alive);
                }
            }
        }
        for &r in &f.roots {
            walk(f, r, 1.0, true, &mut alive);
        }
        for nd in &f.nodes {
            if let Some((m, _)) = nd.mask {
                alive[m] = true;
            }
        }
    }
    let mut masked = std::collections::HashSet::new();
    mask_targets(&p.layers, &mut masked);
    // nodes whose parent is a switch
    let mut under_switch = vec![false; n];
    for nd in &frames[0].nodes {
        if nd.kind == NodeKind::Switch {
            // every layer path starting with this node's path + "/" is a (grand)child; only direct children matter
            let prefix = format!("{}/", nd.name);
            for m in &frames[0].nodes {
                if m.name.starts_with(&prefix) && !m.name[prefix.len()..].contains('/') {
                    under_switch[m.id] = true;
                }
            }
        }
    }
    let mut cands: Vec<String> = Vec::new();
    for nd in &frames[0].nodes {
        if alive[nd.id] || under_switch[nd.id] || masked.contains(&nd.name) {
            continue;
        }
        // topmost dead node only
        let parent_dead = nd.name.rfind('/').is_some_and(|i| {
            let pn = &nd.name[..i];
            frames[0].nodes.iter().any(|m| m.name == pn && !alive[m.id])
        });
        if !parent_dead {
            cands.push(nd.name.clone());
        }
    }
    if cands.is_empty() {
        return unchanged();
    }
    let mut q = p.clone();
    let mut dropped = Vec::new();
    for c in cands {
        let mut t = q.clone();
        if crate::scriptops::delete_layer(&mut t, &c).is_ok() && Engine::new(t.clone()).is_ok() {
            q = t;
            dropped.push(c);
        }
    }
    if dropped.is_empty() {
        return unchanged();
    }
    let Ok(e1) = Engine::new(q.clone()) else { return unchanged() };
    let opts = RasterOptions { scale: 0.25, show_bones: false, transparent: false };
    let m = p.frames.max(1);
    let k = 12.min(m);
    for j in 0..k {
        let f = (j * m / k) as f32;
        if render_rgba(&e0, f, &opts).2 != render_rgba(&e1, f, &opts).2 {
            return unchanged();
        }
    }
    (q, dropped)
}

// ------------------------------------------------------------------------------------------
// Colour quantisation (median cut), dithering, indexed frame sequences
// ------------------------------------------------------------------------------------------

#[derive(Default, Clone)]
struct FastHash(u64);
impl std::hash::Hasher for FastHash {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = (self.0 ^ x as u64).wrapping_mul(0x100000001b3);
        }
    }
    fn write_u32(&mut self, x: u32) {
        self.0 = (x as u64).wrapping_mul(0x9E3779B97F4A7C15);
        self.0 ^= self.0 >> 29;
    }
}
type FastMap<K, V> = HashMap<K, V, std::hash::BuildHasherDefault<FastHash>>;

fn pack(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | b as u32
}

/// Median-cut palette (Heckbert 1982) of at most `max` colours from the opaque pixels of `frames`.
/// If there are no more distinct colours than `max` the palette is exact. Deterministic.
pub fn median_cut(frames: &[&[u8]], max: usize) -> Vec<[u8; 3]> {
    let max = max.max(1);
    let total: usize = frames.iter().map(|f| f.len() / 4).sum();
    let stride = (total / 3_000_000).max(1);
    let mut hist: FastMap<u32, u32> = FastMap::default();
    let mut k = 0usize;
    for f in frames {
        for p in f.chunks_exact(4) {
            k += 1;
            if k.is_multiple_of(stride) && p[3] >= 128 {
                *hist.entry(pack(p[0], p[1], p[2])).or_insert(0) += 1;
            }
        }
    }
    let mut items: Vec<(u32, u32)> = hist.into_iter().collect();
    items.sort_unstable();
    if items.len() <= max {
        return items.iter().map(|&(c, _)| [(c >> 16) as u8, (c >> 8) as u8, c as u8]).collect();
    }
    let ch = |c: u32, a: usize| ((c >> (16 - 8 * a)) & 255) as f64;
    // boxes are ranges of `items`
    struct B {
        lo: usize,
        hi: usize,
        sse: f64,
    }
    let sse_of = |items: &[(u32, u32)]| -> f64 {
        let mut w = 0.0;
        let mut m = [0.0; 3];
        for &(c, n) in items {
            w += n as f64;
            for (a, ma) in m.iter_mut().enumerate() {
                *ma += ch(c, a) * n as f64;
            }
        }
        if w == 0.0 {
            return 0.0;
        }
        let mean = [m[0] / w, m[1] / w, m[2] / w];
        items.iter().map(|&(c, n)| (0..3).map(|a| (ch(c, a) - mean[a]).powi(2)).sum::<f64>() * n as f64).sum()
    };
    let mut boxes = vec![B { lo: 0, hi: items.len(), sse: sse_of(&items) }];
    while boxes.len() < max {
        let Some((bi, _)) = boxes.iter().enumerate().filter(|(_, b)| b.hi - b.lo > 1).max_by(|a, b| a.1.sse.total_cmp(&b.1.sse).then(b.0.cmp(&a.0))) else { break };
        let (lo, hi) = (boxes[bi].lo, boxes[bi].hi);
        // longest axis
        let mut mn = [255.0f64; 3];
        let mut mx = [0.0f64; 3];
        for &(c, _) in &items[lo..hi] {
            for a in 0..3 {
                mn[a] = mn[a].min(ch(c, a));
                mx[a] = mx[a].max(ch(c, a));
            }
        }
        let axis = (0..3).max_by(|&a, &b| (mx[a] - mn[a]).total_cmp(&(mx[b] - mn[b])).then(b.cmp(&a))).unwrap();
        items[lo..hi].sort_by(|x, y| ch(x.0, axis).total_cmp(&ch(y.0, axis)).then(x.0.cmp(&y.0)));
        let pop: u64 = items[lo..hi].iter().map(|x| x.1 as u64).sum();
        let mut acc = 0u64;
        let mut cut = lo + 1;
        for (i, x) in items[lo..hi].iter().enumerate() {
            acc += x.1 as u64;
            if acc * 2 >= pop {
                cut = lo + i + 1;
                break;
            }
        }
        let cut = cut.clamp(lo + 1, hi - 1);
        let (s1, s2) = (sse_of(&items[lo..cut]), sse_of(&items[cut..hi]));
        boxes[bi] = B { lo, hi: cut, sse: s1 };
        boxes.push(B { lo: cut, hi, sse: s2 });
    }
    boxes.sort_by_key(|b| b.lo);
    boxes
        .iter()
        .map(|b| {
            let mut w = 0.0;
            let mut m = [0.0; 3];
            for &(c, n) in &items[b.lo..b.hi] {
                w += n as f64;
                for (a, ma) in m.iter_mut().enumerate() {
                    *ma += ch(c, a) * n as f64;
                }
            }
            [(m[0] / w).round() as u8, (m[1] / w).round() as u8, (m[2] / w).round() as u8]
        })
        .collect()
}

struct Mapper<'a> {
    pal: &'a [[u8; 3]],
    base: u8,
    cache: FastMap<u32, u8>,
}

impl<'a> Mapper<'a> {
    fn new(pal: &'a [[u8; 3]], base: u8) -> Self {
        Mapper { pal, base, cache: FastMap::default() }
    }
    fn nearest(&mut self, r: u8, g: u8, b: u8) -> u8 {
        let key = pack(r, g, b);
        if let Some(&i) = self.cache.get(&key) {
            return i;
        }
        let mut best = (u32::MAX, 0usize);
        for (i, c) in self.pal.iter().enumerate() {
            let d = (c[0] as i32 - r as i32).pow(2) as u32 * 3 + (c[1] as i32 - g as i32).pow(2) as u32 * 4 + (c[2] as i32 - b as i32).pow(2) as u32 * 2;
            if d < best.0 {
                best = (d, i);
            }
        }
        let idx = self.base + best.1 as u8;
        self.cache.insert(key, idx);
        idx
    }
}

/// Frames reduced to a shared palette. Index 0 is reserved (transparent / "unchanged") when
/// `transparent` is set; colours follow.
pub struct Indexed {
    pub w: usize,
    pub h: usize,
    /// Full palette including the reserved entry (black) at index 0 when `transparent`.
    pub palette: Vec<[u8; 3]>,
    pub transparent: bool,
    pub frames: Vec<Vec<u8>>,
}

/// Quantise `frames` (straight RGBA, all `w x h`) to at most `colors` palette entries in total.
/// Pixels with alpha < 128 map to the reserved index 0 (so `reserve` must be true if any exist).
pub fn quantize<F: AsRef<[u8]>>(frames: &[F], w: usize, h: usize, colors: usize, dither: bool, reserve: bool) -> Indexed {
    let colors = colors.clamp(2, 256);
    let ncol = colors - reserve as usize;
    let refs: Vec<&[u8]> = frames.iter().map(|f| f.as_ref()).collect();
    let pal = median_cut(&refs, ncol);
    let base = reserve as u8;
    let mut full: Vec<[u8; 3]> = Vec::new();
    if reserve {
        full.push([0, 0, 0]);
    }
    full.extend(&pal);
    let mut mapper = Mapper::new(&pal, base);
    let mut out = Vec::with_capacity(frames.len());
    for fr in frames {
        let fr = fr.as_ref();
        let mut idx = vec![0u8; w * h];
        if dither && !pal.is_empty() {
            // Floyd-Steinberg, left to right, two error rows
            let mut cur = vec![[0.0f32; 3]; w + 2];
            let mut nxt = vec![[0.0f32; 3]; w + 2];
            for y in 0..h {
                for e in nxt.iter_mut() {
                    *e = [0.0; 3];
                }
                for x in 0..w {
                    let o = (y * w + x) * 4;
                    if fr[o + 3] < 128 {
                        idx[y * w + x] = 0;
                        continue;
                    }
                    let mut v = [0u8; 3];
                    let mut want = [0.0f32; 3];
                    for a in 0..3 {
                        want[a] = (fr[o + a] as f32 + cur[x + 1][a]).clamp(0.0, 255.0);
                        v[a] = want[a].round() as u8;
                    }
                    let i = mapper.nearest(v[0], v[1], v[2]);
                    idx[y * w + x] = i;
                    let c = full[i as usize];
                    for a in 0..3 {
                        let err = want[a] - c[a] as f32;
                        cur[x + 2][a] += err * 7.0 / 16.0;
                        nxt[x][a] += err * 3.0 / 16.0;
                        nxt[x + 1][a] += err * 5.0 / 16.0;
                        nxt[x + 2][a] += err * 1.0 / 16.0;
                    }
                }
                std::mem::swap(&mut cur, &mut nxt);
            }
        } else {
            for (p, slot) in idx.iter_mut().enumerate().take(w * h) {
                let o = p * 4;
                *slot = if fr[o + 3] < 128 { 0 } else { mapper.nearest(fr[o], fr[o + 1], fr[o + 2]) };
            }
        }
        out.push(idx);
    }
    Indexed { w, h, palette: full, transparent: reserve, frames: out }
}

/// One output frame: a rectangle of indices.
pub struct Rect {
    pub left: usize,
    pub top: usize,
    pub w: usize,
    pub h: usize,
    pub idx: Vec<u8>,
}

/// Inter-frame deltas. Frame 0 is whole. Later frames hold only the bounding box of changed pixels;
/// unchanged pixels inside it carry index 0 (transparent). A pixel counts as unchanged if its palette
/// colour is within `tol` levels (per channel) of what the previous output left on screen, so the error
/// to the true indexed frame never exceeds `tol`. Requires `q.transparent`.
pub fn delta_rects(q: &Indexed, tol: u8) -> Vec<Rect> {
    let (w, h) = (q.w, q.h);
    let mut out = Vec::new();
    let mut shown: Vec<u8> = Vec::new();
    for (k, fr) in q.frames.iter().enumerate() {
        if k == 0 {
            out.push(Rect { left: 0, top: 0, w, h, idx: fr.clone() });
            shown = fr.clone();
            continue;
        }
        let same = |a: u8, b: u8| -> bool {
            if a == b {
                return true;
            }
            if tol == 0 || a == 0 || b == 0 {
                return false;
            }
            let (ca, cb) = (q.palette[a as usize], q.palette[b as usize]);
            (0..3).all(|c| (ca[c] as i32 - cb[c] as i32).abs() <= tol as i32)
        };
        let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
        for y in 0..h {
            for x in 0..w {
                if !same(fr[y * w + x], shown[y * w + x]) {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        if x1 <= x0 {
            out.push(Rect { left: 0, top: 0, w: 1, h: 1, idx: vec![0] });
            continue;
        }
        let (rw, rh) = (x1 - x0, y1 - y0);
        let mut idx = vec![0u8; rw * rh];
        for y in 0..rh {
            for x in 0..rw {
                let p = (y + y0) * w + x + x0;
                if !same(fr[p], shown[p]) {
                    idx[y * rw + x] = fr[p];
                    shown[p] = fr[p];
                }
            }
        }
        out.push(Rect { left: x0, top: y0, w: rw, h: rh, idx });
    }
    out
}

fn delay_cs(fps: f32, step: u32) -> u16 {
    ((100.0 * step as f32 / fps.max(0.001)).round() as u16).max(2)
}

/// Looping GIF with a global palette, optional deltas. `alpha` = frames contain transparent pixels
/// (then no deltas are used and every frame is redrawn on a cleared canvas).
pub fn gif_encode(q: &Indexed, fps: f32, step: u32, delta: bool, delta_tol: u8) -> Result<Vec<u8>, String> {
    if q.w > u16::MAX as usize || q.h > u16::MAX as usize {
        return Err("image too large for GIF".into());
    }
    let nent = q.palette.len().max(2).next_power_of_two();
    let mut pal = Vec::with_capacity(nent * 3);
    for i in 0..nent {
        let c = q.palette.get(i).copied().unwrap_or([0, 0, 0]);
        pal.extend_from_slice(&c);
    }
    let mut out = Vec::new();
    {
        let mut e = gif::Encoder::new(&mut out, q.w as u16, q.h as u16, &pal).map_err(|e| e.to_string())?;
        e.set_repeat(gif::Repeat::Infinite).map_err(|e| e.to_string())?;
        let delay = delay_cs(fps, step);
        let rects: Vec<Rect> = if delta && q.transparent && q.frames.len() > 1 {
            delta_rects(q, delta_tol)
        } else {
            q.frames.iter().map(|f| Rect { left: 0, top: 0, w: q.w, h: q.h, idx: f.clone() }).collect()
        };
        for (k, r) in rects.into_iter().enumerate() {
            let mut fr = gif::Frame {
                left: r.left as u16,
                top: r.top as u16,
                width: r.w as u16,
                height: r.h as u16,
                delay,
                buffer: std::borrow::Cow::Owned(r.idx),
                ..gif::Frame::default()
            };
            if q.transparent {
                fr.transparent = Some(0);
                // with deltas the canvas is kept; with real transparency each frame clears the previous one
                fr.dispose = if delta && q.frames.len() > 1 { gif::DisposalMethod::Keep } else { gif::DisposalMethod::Background };
            }
            let _ = k;
            e.write_frame(&fr).map_err(|e| e.to_string())?;
        }
    }
    Ok(out)
}

/// Decode a GIF to full composited RGBA frames (`Keep` / `Background` disposal; others treated as Keep).
pub fn gif_decode(bytes: &[u8]) -> Result<(usize, usize, Vec<Vec<u8>>), String> {
    let mut opts = gif::DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);
    let mut d = opts.read_info(bytes).map_err(|e| e.to_string())?;
    let (w, h) = (d.width() as usize, d.height() as usize);
    let mut canvas = vec![0u8; w * h * 4];
    let mut frames = Vec::new();
    let mut clear: Option<(usize, usize, usize, usize)> = None;
    while let Some(fr) = d.read_next_frame().map_err(|e| e.to_string())? {
        if let Some((l, t, rw, rh)) = clear.take() {
            for y in t..(t + rh).min(h) {
                for x in l..(l + rw).min(w) {
                    canvas[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
        let (l, t, rw, rh) = (fr.left as usize, fr.top as usize, fr.width as usize, fr.height as usize);
        for y in 0..rh {
            for x in 0..rw {
                let (cx, cy) = (l + x, t + y);
                if cx >= w || cy >= h {
                    continue;
                }
                let s = (y * rw + x) * 4;
                if fr.buffer[s + 3] > 0 {
                    let o = (cy * w + cx) * 4;
                    canvas[o..o + 4].copy_from_slice(&fr.buffer[s..s + 4]);
                }
            }
        }
        if fr.dispose == gif::DisposalMethod::Background {
            clear = Some((l, t, rw, rh));
        }
        frames.push(canvas.clone());
    }
    Ok((w, h, frames))
}

fn bit_depth(n: usize) -> (png::BitDepth, usize) {
    if n <= 2 {
        (png::BitDepth::One, 1)
    } else if n <= 4 {
        (png::BitDepth::Two, 2)
    } else if n <= 16 {
        (png::BitDepth::Four, 4)
    } else {
        (png::BitDepth::Eight, 8)
    }
}

fn pack_rows(idx: &[u8], w: usize, h: usize, bits: usize) -> Vec<u8> {
    if bits == 8 {
        return idx.to_vec();
    }
    let per = 8 / bits;
    let row_bytes = w.div_ceil(per);
    let mut out = vec![0u8; row_bytes * h];
    for y in 0..h {
        for x in 0..w {
            let v = idx[y * w + x] << (8 - bits * (x % per + 1));
            out[y * row_bytes + x / per] |= v;
        }
    }
    out
}

/// Looping APNG from an indexed sequence (palette + tRNS entry 0, bit depth by palette size, optional deltas).
pub fn apng_encode_indexed(q: &Indexed, fps: f32, step: u32, delta: bool, delta_tol: u8) -> Result<Vec<u8>, String> {
    let n = q.frames.len() as u32;
    if n == 0 {
        return Err("no frames".into());
    }
    let (depth, bits) = bit_depth(q.palette.len());
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, q.w as u32, q.h as u32);
        enc.set_color(png::ColorType::Indexed);
        enc.set_depth(depth);
        enc.set_compression(png::Compression::High);
        enc.set_filter(png::Filter::Adaptive);
        enc.set_palette(q.palette.iter().flat_map(|c| c.iter().copied()).collect::<Vec<u8>>());
        if q.transparent {
            enc.set_trns(vec![0u8]);
        }
        enc.set_animated(n, 0).map_err(|e| e.to_string())?;
        // not `clamp`: a NaN `fps` must land on the lower bound as `max` does, `clamp` would pass NaN through
        #[allow(clippy::manual_clamp)]
        enc.set_frame_delay((100 * step).min(65535) as u16, (fps * 100.0).round().max(1.0).min(65535.0) as u16).map_err(|e| e.to_string())?;
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        let rects: Vec<Rect> = if delta && q.transparent && q.frames.len() > 1 {
            delta_rects(q, delta_tol)
        } else {
            q.frames.iter().map(|f| Rect { left: 0, top: 0, w: q.w, h: q.h, idx: f.clone() }).collect()
        };
        for (k, r) in rects.iter().enumerate() {
            if k > 0 {
                wr.set_frame_dimension(r.w as u32, r.h as u32).map_err(|e| e.to_string())?;
                wr.set_frame_position(r.left as u32, r.top as u32).map_err(|e| e.to_string())?;
                let over = delta && q.transparent;
                wr.set_blend_op(if over { png::BlendOp::Over } else { png::BlendOp::Source }).map_err(|e| e.to_string())?;
                wr.set_dispose_op(png::DisposeOp::None).map_err(|e| e.to_string())?;
            }
            wr.write_image_data(&pack_rows(&r.idx, r.w, r.h, bits)).map_err(|e| e.to_string())?;
        }
        wr.finish().map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// Truecolour looping APNG at maximum deflate effort with adaptive filtering.
pub fn apng_encode_rgba<F: AsRef<[u8]>>(w: usize, h: usize, frames: &[F], fps: f32, step: u32) -> Result<Vec<u8>, String> {
    if frames.is_empty() {
        return Err("no frames".into());
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w as u32, h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::High);
        enc.set_filter(png::Filter::Adaptive);
        enc.set_animated(frames.len() as u32, 0).map_err(|e| e.to_string())?;
        // not `clamp`: a NaN `fps` must land on the lower bound as `max` does, `clamp` would pass NaN through
        #[allow(clippy::manual_clamp)]
        enc.set_frame_delay((100 * step).min(65535) as u16, (fps * 100.0).round().max(1.0).min(65535.0) as u16).map_err(|e| e.to_string())?;
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        for f in frames {
            wr.write_image_data(f.as_ref()).map_err(|e| e.to_string())?;
        }
        wr.finish().map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// Decode an APNG to full composited RGBA frames (blend `Source` / `Over`; dispose `None` / `Background` /
/// `Previous`).
pub fn apng_decode(bytes: &[u8]) -> Result<(usize, usize, Vec<Vec<u8>>), String> {
    crate::assets::check_png(bytes)?;
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
    dec.set_transformations(png::Transformations::EXPAND);
    let mut r = dec.read_info().map_err(|e| e.to_string())?;
    let (w, h) = (r.info().width as usize, r.info().height as usize);
    let n = r.info().animation_control.map_or(1, |a| a.num_frames as usize);
    let mut canvas = vec![0u8; w * h * 4];
    let mut frames = Vec::new();
    for _ in 0..n {
        let mut buf = vec![0u8; r.output_buffer_size().ok_or("png buffer size overflow")?];
        let oi = r.next_frame(&mut buf).map_err(|e| e.to_string())?;
        let fc = r.info().frame_control;
        let (fx, fy, fw, fh, blend, dispose) = match fc {
            Some(c) => (c.x_offset as usize, c.y_offset as usize, c.width as usize, c.height as usize, c.blend_op, c.dispose_op),
            None => (0, 0, w, h, png::BlendOp::Source, png::DisposeOp::None),
        };
        let px: Vec<u8> = match oi.color_type {
            png::ColorType::Rgba => buf[..fw * fh * 4].to_vec(),
            png::ColorType::Rgb => buf[..fw * fh * 3].chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
            png::ColorType::GrayscaleAlpha => buf[..fw * fh * 2].chunks_exact(2).flat_map(|c| [c[0], c[0], c[0], c[1]]).collect(),
            png::ColorType::Grayscale => buf[..fw * fh].iter().flat_map(|&c| [c, c, c, 255]).collect(),
            png::ColorType::Indexed => return Err("indexed output after expand".into()),
        };
        let before = canvas.clone();
        for y in 0..fh {
            for x in 0..fw {
                let (cx, cy) = (fx + x, fy + y);
                if cx >= w || cy >= h {
                    continue;
                }
                let s = (y * fw + x) * 4;
                let o = (cy * w + cx) * 4;
                match blend {
                    png::BlendOp::Source => canvas[o..o + 4].copy_from_slice(&px[s..s + 4]),
                    png::BlendOp::Over => {
                        let a = px[s + 3] as u32;
                        if a == 255 {
                            canvas[o..o + 4].copy_from_slice(&px[s..s + 4]);
                        } else if a > 0 {
                            let da = canvas[o + 3] as u32;
                            let oa = a + da * (255 - a) / 255;
                            for c in 0..3 {
                                canvas[o + c] = ((px[s + c] as u32 * a + canvas[o + c] as u32 * da * (255 - a) / 255) / oa.max(1)) as u8;
                            }
                            canvas[o + 3] = oa as u8;
                        }
                    }
                }
            }
        }
        frames.push(canvas.clone());
        match dispose {
            png::DisposeOp::None => {}
            png::DisposeOp::Background => {
                for y in fy..(fy + fh).min(h) {
                    for x in fx..(fx + fw).min(w) {
                        canvas[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[0, 0, 0, 0]);
                    }
                }
            }
            png::DisposeOp::Previous => canvas = before,
        }
    }
    Ok((w, h, frames))
}

// ------------------------------------------------------------------------------------------
// WebM through ffmpeg (VP9 with alpha, constant quality)
// ------------------------------------------------------------------------------------------


pub fn ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg").arg("-version").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().is_ok_and(|s| s.success())
}

/// VP9 (libvpx) WebM of straight-RGBA frames at `fps`, odd sizes padded to even.
pub fn webm_encode<F: AsRef<[u8]>>(w: usize, h: usize, frames: &[F], fps: f32, crf: u8) -> Result<Vec<u8>, String> {
    let dir = crate::fsutil::private_temp_dir("osatura-fit")?;
    let out = dir.join("out.webm");
    let mut child = std::process::Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error", "-protocol_whitelist", "file,pipe,fd", "-f", "rawvideo", "-pix_fmt", "rgba", "-s", &format!("{w}x{h}"), "-framerate", &format!("{fps}"), "-i", "-"])
        .args(["-vf", "pad=ceil(iw/2)*2:ceil(ih/2)*2:color=0x00000000", "-c:v", "libvpx-vp9", "-pix_fmt", "yuva420p", "-b:v", "0", "-crf", &crf.to_string()])
        .args(["-auto-alt-ref", "0", "-deadline", "good", "-cpu-used", "2", "-threads", "1"])
        .arg(&out)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run ffmpeg ({e}); install it or choose another format"))?;
    {
        let mut si = child.stdin.take().ok_or("ffmpeg stdin")?;
        for f in frames {
            if si.write_all(f.as_ref()).is_err() {
                break;
            }
        }
    }
    let o = child.wait_with_output().map_err(|e| e.to_string())?;
    if o.status.success() { std::fs::read(&out).map_err(|e| e.to_string()) } else { Err(format!("ffmpeg failed: {}", String::from_utf8_lossy(&o.stderr).trim())) }
}

/// Decode a WebM to RGBA frames of `w x h` (cropping any padding).
pub fn webm_decode(bytes: &[u8], w: usize, h: usize) -> Result<Vec<Vec<u8>>, String> {
    let dir = crate::fsutil::private_temp_dir("osatura-fit")?;
    let inp = dir.join("in.webm");
    std::fs::write(&inp, bytes).map_err(|e| e.to_string())?;
    let (pw, ph) = (w + w % 2, h + h % 2);
    let o = std::process::Command::new("ffmpeg")
        .args(["-loglevel", "error", "-protocol_whitelist", "file", "-c:v", "libvpx-vp9", "-i"])
        .arg(&inp)
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-fps_mode", "passthrough", "-"])
        .output()
        .map_err(|e| format!("cannot run ffmpeg ({e})"));
    let o = o?;
    if !o.status.success() {
        return Err(format!("ffmpeg decode failed: {}", String::from_utf8_lossy(&o.stderr).trim()));
    }
    let fsz = pw * ph * 4;
    if fsz == 0 || o.stdout.len() % fsz != 0 {
        return Err("unexpected decoded size".into());
    }
    Ok(o.stdout
        .chunks_exact(fsz)
        .map(|f| {
            let mut v = Vec::with_capacity(w * h * 4);
            for y in 0..h {
                v.extend_from_slice(&f[y * pw * 4..y * pw * 4 + w * 4]);
            }
            v
        })
        .collect())
}

#[allow(dead_code)]
fn _read_marker<R: Read>(_: R) {}

// ------------------------------------------------------------------------------------------
// Time series reduction shared by Lottie tracks (step decimation, key reduction, deduplication)
// ------------------------------------------------------------------------------------------

/// One sampled value of an animated track: a topology signature (closed flag, vertex count) and the
/// flattened numbers. Two samples can only be interpolated if their signatures are equal.
#[derive(Clone, Debug, PartialEq)]
struct Samp {
    sig: (bool, usize),
    v: Vec<f64>,
}

fn samp_of_shape(closed: bool, nodes: &[PNode], d: u8) -> Samp {
    let mut v = Vec::with_capacity(nodes.len() * 6);
    for n in nodes {
        v.extend(n.p.iter().map(|&x| round_dec(x, d)));
    }
    for n in nodes {
        v.extend(n.i.iter().map(|&x| round_dec(x, d)));
    }
    for n in nodes {
        v.extend(n.o.iter().map(|&x| round_dec(x, d)));
    }
    Samp { sig: (closed, nodes.len()), v }
}

fn shape_of_samp(s: &Samp) -> Value {
    let n = s.sig.1;
    let pair = |off: usize, k: usize| json!([num(s.v[off + 2 * k]), num(s.v[off + 2 * k + 1])]);
    let vs: Vec<Value> = (0..n).map(|k| pair(0, k)).collect();
    let is: Vec<Value> = (0..n).map(|k| pair(2 * n, k)).collect();
    let os: Vec<Value> = (0..n).map(|k| pair(4 * n, k)).collect();
    json!({"i": is, "o": os, "v": vs, "c": s.sig.0})
}

fn parse_shape(v: &Value) -> Option<(bool, Vec<PNode>)> {
    let arr = |k: &str| -> Option<Vec<[f64; 2]>> {
        v.get(k)?.as_array()?.iter().map(|p| { let a = p.as_array()?; Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?]) }).collect()
    };
    let (vv, ii, oo) = (arr("v")?, arr("i")?, arr("o")?);
    if vv.len() != ii.len() || vv.len() != oo.len() {
        return None;
    }
    let closed = v.get("c").and_then(Value::as_bool).unwrap_or(false);
    Some((closed, vv.iter().zip(&ii).zip(&oo).map(|((&p, &i), &o)| PNode { p, i, o }).collect()))
}

/// Choose the keyframes of a series. `step` keeps every `step`-th frame; with `linear` the kept frames are
/// reduced by Ramer-Douglas-Peucker so that linear interpolation between neighbours stays within `tol`
/// (max over all numbers, at the dropped frames); without it the track is held and only repeats are dropped.
/// Returns `(frame, hold_after)`.
fn select_keys(s: &[Samp], step: usize, tol: f64, linear: bool) -> Vec<(usize, bool)> {
    let n = s.len();
    let k: Vec<usize> = (0..n).step_by(step.max(1)).collect();
    if k.is_empty() {
        return vec![];
    }
    if !linear {
        let mut out: Vec<(usize, bool)> = Vec::new();
        for &f in &k {
            if out.last().is_none_or(|&(pf, _)| s[pf] != s[f]) {
                out.push((f, true));
            }
        }
        return out;
    }
    let mut keep = vec![false; k.len()];
    let mut run = 0usize;
    for i in 0..=k.len() {
        if i < k.len() && s[k[i]].sig == s[k[run]].sig {
            continue;
        }
        let (a0, b0) = (run, i - 1);
        keep[a0] = true;
        keep[b0] = true;
        let mut stack = vec![(a0, b0)];
        while let Some((a, b)) = stack.pop() {
            if b - a < 2 {
                continue;
            }
            let (fa, fb) = (k[a] as f64, k[b] as f64);
            let mut worst = (0.0f64, 0usize);
            for j in a + 1..b {
                let t = (k[j] as f64 - fa) / (fb - fa);
                let e = (0..s[k[j]].v.len()).map(|c| (s[k[j]].v[c] - lerp(s[k[a]].v[c], s[k[b]].v[c], t)).abs()).fold(0.0, f64::max);
                if e > worst.0 {
                    worst = (e, j);
                }
            }
            if worst.0 > tol {
                keep[worst.1] = true;
                stack.push((a, worst.1));
                stack.push((worst.1, b));
            }
        }
        run = i;
    }
    let kept: Vec<usize> = (0..k.len()).filter(|&i| keep[i]).collect();
    // drop keys that repeat their predecessor and (if any) their successor: a constant stretch
    let mut out: Vec<usize> = Vec::new();
    for (qi, &i) in kept.iter().enumerate() {
        let same_prev = out.last().is_some_and(|&p| s[k[p]] == s[k[i]]);
        let same_next = kept.get(qi + 1).is_none_or(|&nx| s[k[nx]] == s[k[i]]);
        if same_prev && same_next {
            continue;
        }
        out.push(i);
    }
    (0..out.len())
        .map(|q| {
            let hold = out.get(q + 1).is_none_or(|&nx| s[k[nx]].sig != s[k[out[q]]].sig);
            (k[out[q]], hold)
        })
        .collect()
}

fn emit_keys(s: &[Samp], keys: &[(usize, bool)], linear: bool, mk: &dyn Fn(&Samp) -> Value) -> Value {
    if keys.len() <= 1 {
        let f = keys.first().map_or(0, |k| k.0);
        return json!({"a": 0, "k": mk(&s[f])});
    }
    let arr: Vec<Value> = keys
        .iter()
        .enumerate()
        .map(|(q, &(f, hold))| {
            let mut o = json!({"t": f, "s": [mk(&s[f])]});
            if q + 1 < keys.len() {
                if hold || !linear {
                    o["h"] = json!(1);
                } else {
                    o["o"] = json!({"x": 0, "y": 0});
                    o["i"] = json!({"x": 1, "y": 1});
                }
            } else if !linear {
                o["h"] = json!(1);
            }
            o
        })
        .collect();
    json!({"a": 1, "k": arr})
}

/// Hold keyframes (exact at the kept frames) or, when `linear`, linear keyframes within `tol`, whichever
/// serialises smaller: linear keys carry easing handles, so they only pay off when they remove enough keys.
fn pick_encoding(s: &[Samp], step: usize, tol: f64, linear: bool, mk: &dyn Fn(&Samp) -> Value) -> Value {
    let hold = emit_keys(s, &select_keys(s, step, 0.0, false), false, mk);
    if !linear {
        return hold;
    }
    let lin = emit_keys(s, &select_keys(s, step, tol, true), true, mk);
    if lin.to_string().len() < hold.to_string().len() {
        lin
    } else {
        hold
    }
}

// ------------------------------------------------------------------------------------------
// Lottie: reduce a baked document, read it back
// ------------------------------------------------------------------------------------------

/// Hold-keyframe list of an animated track expanded to one value per frame (`n` frames).
fn expand_hold(k: &Value, n: usize) -> Option<Vec<Value>> {
    let arr = k.as_array()?;
    let mut keys: Vec<(usize, &Value)> = arr.iter().filter_map(|e| Some((e.get("t")?.as_f64()? as usize, e.get("s")?.as_array()?.first()?))).collect();
    keys.sort_by_key(|k| k.0);
    if keys.is_empty() {
        return None;
    }
    Some((0..n).map(|f| { let i = keys.partition_point(|k| k.0 <= f); keys[i.saturating_sub(1)].1.clone() }).collect())
}

fn round_doc(v: &mut Value, d: u8, cd: u8, colour: bool) {
    match v {
        Value::Number(x) => {
            if let Some(f) = x.as_f64() {
                if f.fract() != 0.0 {
                    *v = num(round_dec(f, if colour { cd } else { d }));
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|e| round_doc(e, d, cd, colour)),
        Value::Object(m) => {
            for (k, e) in m.iter_mut() {
                if matches!(k.as_str(), "fr" | "w" | "h" | "sw" | "sh" | "ip" | "op" | "t" | "s_") {
                    continue;
                }
                round_doc(e, d, cd, colour || k == "c" || k == "g");
            }
        }
        _ => {}
    }
}

/// Reduce a baked Lottie document ([`crate::export::lottie_json_mapped`]) with the lottie settings:
/// `decimals`, `path_eps` (spatial simplification over all frames, same vertices removed everywhere),
/// `key_tol` (0 = hold keyframes at every changed frame; > 0 = linear interpolation with the bound of
/// [`select_keys`]) and `frame_step`. Repeated frames are always merged and constant tracks become static.
pub fn lottie_reduce(base: &Value, s: &Settings) -> Value {
    let mut doc = base.clone();
    let n = doc.get("op").and_then(Value::as_f64).unwrap_or(1.0).max(1.0) as usize;
    let d = s.decimals;
    let linear = s.key_tol > 0.0;
    let tol = s.key_tol as f64;
    let step = s.frame_step.max(1) as usize;
    let Some(layers) = doc.get_mut("layers").and_then(Value::as_array_mut) else { return doc };
    for layer in layers.iter_mut() {
        // layer opacity
        if let Some(o) = layer.get_mut("ks").and_then(|k| k.get_mut("o")) {
            if o.get("a").and_then(Value::as_i64) == Some(1) {
                if let Some(vals) = o.get("k").and_then(|k| expand_hold(k, n)) {
                    let samp: Vec<Samp> = vals.iter().map(|v| Samp { sig: (false, 1), v: vec![round_dec(v.as_f64().unwrap_or(0.0), d)] }).collect();
                    *o = pick_encoding(&samp, step, tol, linear, &|sm| num(sm.v[0]));
                }
            }
        }
        // shape tracks
        let Some(groups) = layer.get_mut("shapes").and_then(Value::as_array_mut) else { continue };
        for g in groups.iter_mut() {
            let Some(items) = g.get_mut("it").and_then(Value::as_array_mut) else { continue };
            for it in items.iter_mut() {
                if it.get("ty").and_then(Value::as_str) != Some("sh") {
                    continue;
                }
                let Some(ks) = it.get_mut("ks") else { continue };
                if ks.get("a").and_then(Value::as_i64) != Some(1) {
                    continue;
                }
                let Some(vals) = ks.get("k").and_then(|k| expand_hold(k, n)) else { continue };
                let parsed: Vec<Option<(bool, Vec<PNode>)>> = vals.iter().map(parse_shape).collect();
                if parsed.iter().any(|p| p.is_none()) {
                    continue;
                }
                let mut shapes: Vec<(bool, Vec<PNode>)> = parsed.into_iter().map(|p| p.unwrap()).collect();
                // spatial simplification when the topology is the same in every frame
                if s.path_eps > 0.0 {
                    let (c0, n0) = (shapes[0].0, shapes[0].1.len());
                    if n0 >= 3 && shapes.iter().all(|(c, v)| *c == c0 && v.len() == n0) {
                        let mut fr: Vec<Vec<PNode>> = shapes.iter().map(|(_, v)| v.clone()).collect();
                        if simplify_paths(&mut fr, c0, s.path_eps as f64) > 0 {
                            shapes = fr.into_iter().map(|v| (c0, v)).collect();
                        }
                    }
                }
                let samp: Vec<Samp> = shapes.iter().map(|(c, v)| samp_of_shape(*c, v, d)).collect();
                *ks = pick_encoding(&samp, step, tol, linear, &shape_of_samp);
            }
        }
    }
    let cd = d.max(2);
    round_doc(&mut doc, d, cd, false);
    doc
}

/// Value of an animated or static track at integer frame `f` as the reader of the format would show it.
fn read_shape_at(ks: &Value, f: usize) -> Option<(bool, Vec<PNode>)> {
    if ks.get("a").and_then(Value::as_i64) != Some(1) {
        return parse_shape(ks.get("k")?);
    }
    let arr = ks.get("k")?.as_array()?;
    let mut keys: Vec<(usize, &Value)> = arr.iter().filter_map(|e| Some((e.get("t")?.as_f64()? as usize, e))).collect();
    keys.sort_by_key(|k| k.0);
    let i = keys.partition_point(|k| k.0 <= f).saturating_sub(1);
    let (t0, e0) = keys[i];
    let a = parse_shape(e0.get("s")?.as_array()?.first()?)?;
    let hold = e0.get("h").and_then(Value::as_i64) == Some(1);
    if hold || i + 1 >= keys.len() || f <= t0 {
        return Some(a);
    }
    let (t1, e1) = keys[i + 1];
    let b = parse_shape(e1.get("s")?.as_array()?.first()?)?;
    if a.0 != b.0 || a.1.len() != b.1.len() || t1 <= t0 {
        return Some(a);
    }
    let u = (f - t0) as f64 / (t1 - t0) as f64;
    let mix = |p: [f64; 2], q: [f64; 2]| [lerp(p[0], q[0], u), lerp(p[1], q[1], u)];
    Some((a.0, a.1.iter().zip(&b.1).map(|(x, y)| PNode { p: mix(x.p, y.p), i: mix(x.i, y.i), o: mix(x.o, y.o) }).collect()))
}

fn read_scalar_at(o: &Value, f: usize) -> f64 {
    if o.get("a").and_then(Value::as_i64) != Some(1) {
        return o.get("k").and_then(Value::as_f64).unwrap_or(100.0);
    }
    let Some(arr) = o.get("k").and_then(Value::as_array) else { return 100.0 };
    let mut keys: Vec<(usize, f64, bool)> = arr
        .iter()
        .filter_map(|e| Some((e.get("t")?.as_f64()? as usize, e.get("s")?.as_array()?.first()?.as_f64()?, e.get("h").and_then(Value::as_i64) == Some(1))))
        .collect();
    keys.sort_by_key(|a| a.0);
    if keys.is_empty() {
        return 100.0;
    }
    let i = keys.partition_point(|k| k.0 <= f).saturating_sub(1);
    let (t0, v0, hold) = keys[i];
    if hold || i + 1 >= keys.len() || f <= t0 {
        return v0;
    }
    let (t1, v1, _) = keys[i + 1];
    lerp(v0, v1, (f - t0) as f64 / (t1 - t0).max(1) as f64)
}

pub fn clone_frame(f: &Frame) -> Frame {
    Frame { world: f.world.clone(), camera: f.camera, nodes: f.nodes.clone(), roots: f.roots.clone(), nonfinite: f.nonfinite.clone() }
}

/// The picture a Lottie player would show at frame `f`, built from the evaluated engine frame `eval`
/// (structure, fills, strokes, masks) with the geometry and the layer opacity replaced by what is *read
/// from the produced document*. Baked vertices are in screen space, so layer transforms become identity;
/// group opacity is flattened into the leaves, as the exporter does.
pub fn lottie_model_frame(eval: &Frame, doc: &Value, map: &[crate::export::LottieLayerMap], f: usize) -> Frame {
    let mut fr = clone_frame(eval);
    // groups whose opacity lives in a precomp (a vector leaf below them has no top-level layer) keep it
    let mapped: std::collections::HashSet<usize> = map.iter().map(|m| m.node).collect();
    let mut keeps = vec![false; fr.nodes.len()];
    for id in 0..fr.nodes.len() {
        if fr.nodes[id].kind == NodeKind::Vector && !mapped.contains(&id) {
            let mut cur = Some(id);
            let mut guard = 0;
            // ancestors: nodes listing it as a child
            while let Some(c) = cur {
                guard += 1;
                if guard > 64 {
                    break;
                }
                cur = (0..fr.nodes.len()).find(|&n| fr.nodes[n].children.contains(&c));
                if let Some(a) = cur {
                    keeps[a] = true;
                }
            }
        }
    }
    for (i, nd) in fr.nodes.iter_mut().enumerate() {
        if nd.kind != NodeKind::Vector && keeps[i] {
            continue;
        }
        if nd.kind != NodeKind::Vector {
            nd.opacity = 1.0;
            nd.mask = None;
            nd.blend = crate::style::Blend::Normal;
        }
    }
    let layers = doc.get("layers").and_then(Value::as_array);
    for m in map.iter().filter(|m| !m.matte_copy) {
        let Some(layer) = layers.and_then(|l| l.get(m.index)) else { continue };
        let nd = &mut fr.nodes[m.node];
        nd.world = Affine2::IDENTITY;
        nd.visible = true;
        if let Some(o) = layer.get("ks").and_then(|k| k.get("o")) {
            nd.opacity = (read_scalar_at(o, f) / 100.0).clamp(0.0, 1.0) as f32;
        }
        for g in layer.get("shapes").and_then(Value::as_array).into_iter().flatten() {
            let pi: Option<usize> = g.get("nm").and_then(Value::as_str).and_then(|s| s.strip_prefix('g')).and_then(|s| s.parse().ok());
            let (Some(pi), Some(items)) = (pi, g.get("it").and_then(Value::as_array)) else { continue };
            let Some(sh) = items.iter().find(|i| i.get("ty").and_then(Value::as_str) == Some("sh")) else { continue };
            let Some(path) = nd.paths.get_mut(pi) else { continue };
            match sh.get("ks").and_then(|k| read_shape_at(k, f)) {
                Some((closed, nodes)) if nodes.len() >= 2 => {
                    path.anchors = anchors_of(&nodes);
                    path.closed = closed;
                }
                _ => path.anchors.clear(),
            }
        }
    }
    fr
}

// ------------------------------------------------------------------------------------------
// HTML player: number rounding in the SVG text, path simplification, frame dedup
// ------------------------------------------------------------------------------------------

fn round_nums(val: &str, d: u8, path_mode: bool) -> String {
    let b = val.as_bytes();
    let mut out = String::with_capacity(val.len());
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let starts = c.is_ascii_digit()
            || ((c == b'-' || c == b'+' || c == b'.') && b.get(i + 1).is_some_and(|n| n.is_ascii_digit() || (*n == b'.' && c != b'.')));
        let prev_ok = i == 0 || path_mode || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
        if starts && prev_ok {
            let mut j = i + 1;
            let mut dot = c == b'.';
            while j < b.len() && (b[j].is_ascii_digit() || (b[j] == b'.' && !dot)) {
                dot |= b[j] == b'.';
                j += 1;
            }
            match val[i..j].parse::<f64>() {
                Ok(x) => out.push_str(&fmt_dec(x, d)),
                Err(_) => out.push_str(&val[i..j]),
            }
            i = j;
        } else {
            let ch = val[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn round_attr(name: &str, val: &str, d: u8) -> String {
    if name == "id" || name == "class" || name == "xmlns" || name.ends_with("href") || val.contains('#') || val.contains("url(") || val.starts_with("data:") {
        return val.to_string();
    }
    match name {
        "d" => round_nums(val, d, true),
        "transform" if val.starts_with("matrix(") => {
            let inner = &val[7..val.len().saturating_sub(1)];
            let parts: Vec<&str> = inner.split_whitespace().collect();
            if parts.len() != 6 || !val.ends_with(')') {
                return val.to_string();
            }
            let f: Vec<String> = parts
                .iter()
                .enumerate()
                .map(|(k, p)| p.parse::<f64>().map(|x| fmt_dec(x, if k < 4 { d.saturating_add(3) } else { d })).unwrap_or_else(|_| p.to_string()))
                .collect();
            format!("matrix({})", f.join(" "))
        }
        "opacity" | "fill-opacity" | "stroke-opacity" | "stop-opacity" | "offset" => round_nums(val, d.max(2), false),
        _ => round_nums(val, d, false),
    }
}

/// Round every number in the attribute values of an SVG fragment to `d` decimals, print numbers in their shortest
/// form (no trailing zeros), drop default `opacity="1"` attributes and the newlines between elements.
/// Ids, class names, colours, `url(#..)` references and `data:` URIs are left alone.
pub fn svg_round(svg: &str, d: u8) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(p) = rest.find("=\"") {
        let name_start = rest[..p].rfind(|c: char| c.is_whitespace() || c == '<').map_or(0, |i| i + 1);
        out.push_str(&rest[..p + 2]);
        let after = &rest[p + 2..];
        let Some(end) = after.find('"') else {
            rest = after;
            break;
        };
        let name = &rest[name_start..p];
        out.push_str(&round_attr(name, &after[..end], d));
        out.push('"');
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out.replace(">\n", ">").replace(" opacity=\"1\"", "").replace(" fill-opacity=\"1\"", "")
}

/// Apply the rounding of [`svg_round`] to the evaluated frame data (positions, tangents, stroke widths,
/// translations to `d` decimals; linear parts of transforms to `d + 3`; opacities to `max(d, 2)`).
/// This is what the HTML quality model rasterises.
fn round_frame(f: &mut Frame, d: u8) {
    let r = |x: f32, k: u8| round_dec(x as f64, k) as f32;
    for nd in f.nodes.iter_mut() {
        nd.world.a = r(nd.world.a, d.saturating_add(3));
        nd.world.b = r(nd.world.b, d.saturating_add(3));
        nd.world.c = r(nd.world.c, d.saturating_add(3));
        nd.world.d = r(nd.world.d, d.saturating_add(3));
        nd.world.tx = r(nd.world.tx, d);
        nd.world.ty = r(nd.world.ty, d);
        nd.opacity = r(nd.opacity, d.max(2));
        for p in nd.paths.iter_mut() {
            for a in p.anchors.iter_mut() {
                a.map_points(&mut |v| Vec2::new(r(v.x, d), r(v.y, d)));
            }
            if let Some(st) = p.stroke.as_mut() {
                st.width = r(st.width, d);
                for x in st.dash.iter_mut() {
                    *x = r(*x, d);
                }
            }
        }
    }
}

fn simplify_frame_paths(f: &mut Frame, eps: f64) {
    if eps <= 0.0 {
        return;
    }
    for nd in f.nodes.iter_mut() {
        let sc = (nd.world.x_scale() as f64).max(1e-6);
        for p in nd.paths.iter_mut() {
            simplify_vpath(p, eps / sc);
        }
    }
}

/// The page: frames as inline SVG, deduplicated (`x` maps displayed frame -> unique frame), play / pause / scrubber.
pub fn html_page(title: &str, width: f32, fps: f32, unique: &[String], x: Option<&[usize]>) -> String {
    let data = serde_json::to_string(unique).unwrap_or_else(|_| "[]".into()).replace("</", "<\\/");
    let xs = match x {
        Some(x) => serde_json::to_string(x).unwrap_or_else(|_| "null".into()),
        None => "null".into(),
    };
    let nframes = x.map_or(unique.len(), |x| x.len());
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title>\
<style>body{{margin:0;font:14px system-ui,sans-serif;background:#14161c;color:#e8e8ee;display:flex;flex-direction:column;align-items:center;min-height:100vh}}\
#s{{margin:16px;max-width:96vw;background:#fff;line-height:0}}#s svg{{max-width:96vw;height:auto}}\
#b{{display:flex;gap:12px;align-items:center;width:min(96vw,{}px);margin-bottom:20px}}\
button{{background:#3a4156;color:#fff;border:0;border-radius:6px;padding:6px 14px;cursor:pointer}}input{{flex:1}}</style></head><body>\
<div id=\"s\"></div><div id=\"b\"><button id=\"p\">Pause</button><input id=\"l\" type=\"range\" min=\"0\" max=\"{}\" value=\"0\"><span id=\"n\">0</span></div>\
<script>const F={},X={},R={};let i=0,pl=1,la=performance.now(),ac=0;\
const s=document.getElementById('s'),l=document.getElementById('l'),n=document.getElementById('n'),p=document.getElementById('p'),N={};\
function show(k){{i=k;s.innerHTML=F[X?X[k]:k];l.value=k;n.textContent=k+' / '+(N-1)}}\
function tick(t){{if(pl){{ac+=(t-la)/1000*R;if(ac>=1){{const m=Math.floor(ac);ac-=m;show((i+m)%N)}}}}la=t;requestAnimationFrame(tick)}}\
p.onclick=()=>{{pl=!pl;p.textContent=pl?'Pause':'Play'}};l.oninput=()=>{{pl=0;p.textContent='Play';show(+l.value)}};show(0);requestAnimationFrame(tick);\
</script></body></html>\n",
        crate::svg::esc(title),
        width,
        nframes.saturating_sub(1),
        data,
        xs,
        fmt_dec(fps as f64, 4),
        nframes
    )
}

// ------------------------------------------------------------------------------------------
// Project JSON
// ------------------------------------------------------------------------------------------

fn round_value(v: &mut Value, d: u8) {
    match v {
        Value::Number(x) => {
            if let Some(f) = x.as_f64() {
                if f.fract() != 0.0 {
                    *v = num(round_dec(f, d));
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|e| round_value(e, d)),
        Value::Object(m) => {
            for (k, e) in m.iter_mut() {
                if matches!(k.as_str(), "fps" | "width" | "height") {
                    continue;
                }
                round_value(e, d);
            }
        }
        _ => {}
    }
}

/// Bytes of the project with keys reduced (`key_tol`) and numbers rounded (`decimals`).
pub fn project_bytes(pre: &Project, s: &Settings) -> Result<Vec<u8>, String> {
    let mut p = pre.clone();
    if s.key_tol > 0.0 {
        reduce_project_keys(&mut p, s.key_tol);
    }
    let mut v = serde_json::to_value(&p).map_err(|e| e.to_string())?;
    round_value(&mut v, s.decimals);
    serde_json::to_vec(&v).map_err(|e| e.to_string())
}

// ------------------------------------------------------------------------------------------
// The fitter
// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Search {
    /// Bisection over the chain only.
    Ladder,
    /// Chain bisection, then coordinate descent; an exhaustive grid is tried when constraints cannot be met.
    Descent,
    /// Exhaustive (thinned) grid over all axes.
    Grid,
}

impl Search {
    pub fn parse(s: &str) -> Result<Search, String> {
        match s {
            "ladder" => Ok(Search::Ladder),
            "descent" => Ok(Search::Descent),
            "grid" => Ok(Search::Grid),
            o => Err(format!("unknown --search '{o}' (ladder, descent or grid)")),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Search::Ladder => "ladder",
            Search::Descent => "descent",
            Search::Grid => "grid",
        }
    }
}

#[derive(Clone, Debug)]
pub struct FitOptions {
    pub format: Format,
    pub budget: usize,
    pub measure: Measure,
    /// Worst sampled frame must reach this SSIM.
    pub min_ssim: Option<f64>,
    /// Number of frames compared with the original render.
    pub sample_frames: usize,
    pub search: Search,
    /// Upper bound on produced candidates in the descent.
    pub max_evals: usize,
    /// Wall-clock budget of the whole search in seconds (default 60; 0 = unlimited). When it runs out the
    /// bisection result found so far is kept, the descent / adaptive / grid refinements stop, and the report
    /// says `time_limited`. A GIF candidate renders and encodes every frame (seconds each at 640x480x48), which
    /// is why the unbounded default search took minutes.
    pub max_seconds: f64,
    /// Largest grid enumerated (axes are thinned to fit).
    pub grid_max: usize,
    /// Evaluate every rung of the chain too and compare with the bisection (slow; used by tests).
    pub audit: bool,
    /// Also measure the project's existing exporter at its defaults.
    pub baseline: bool,
}

impl FitOptions {
    pub fn new(format: Format, budget: usize) -> FitOptions {
        FitOptions { format, budget, measure: Measure::Raw, min_ssim: None, sample_frames: 8, search: Search::Descent, max_evals: 120, max_seconds: 60.0, grid_max: 250, audit: false, baseline: true }
    }
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub pos: Vec<usize>,
    pub settings: Settings,
    pub valid: bool,
    pub error: Option<String>,
    pub raw: usize,
    pub gzip: usize,
    /// The measured size the budget applies to.
    pub budgeted: usize,
    pub ssim_mean: f64,
    pub ssim_min: f64,
    pub worst_frame: usize,
    pub per_frame: Vec<(usize, f64)>,
}

#[derive(Clone, Debug)]
pub struct Quality {
    pub ssim_mean: f64,
    pub ssim_min: f64,
    pub worst_frame: usize,
    /// (frame, ssim, max CIEDE2000)
    pub per_frame: Vec<(usize, f64, f64)>,
    pub de_max: f64,
}

#[derive(Clone, Debug)]
pub struct FitResult {
    pub options: FitOptions,
    pub met: bool,
    pub reason: Option<String>,
    pub chosen: Candidate,
    pub bytes: Vec<u8>,
    pub quality: Quality,
    /// Existing exporter at its defaults: (raw, gzip).
    pub baseline: Option<(usize, usize)>,
    pub floors: Vec<String>,
    pub evaluations: usize,
    /// The search stopped refining because `max_seconds` ran out.
    pub time_limited: bool,
    /// (chain rung, measured budgeted bytes) in probe order.
    pub probes: Vec<(usize, usize)>,
    pub chain_len: usize,
    /// "static" (fixed balanced chain) or "adaptive" (ordered by measured bytes saved per SSIM lost).
    pub chain_kind: &'static str,
    pub monotone: bool,
    pub fallback_linear: bool,
    pub audit_agrees: Option<bool>,
    pub dead_layers: Vec<String>,
    pub sample_frames: Vec<usize>,
    /// Every produced candidate (for the Pareto listing).
    pub all: Vec<Candidate>,
    pub axes: Vec<(String, Vec<f64>)>,
}

struct Ladder {
    idx: Option<usize>,
    probes: Vec<(usize, usize)>,
    monotone: bool,
    fallback: bool,
    audit_agrees: Option<bool>,
    len: usize,
}

/// Rasterised reference frames: `(width, height, per-frame RGBA)`.
type RasterEntry = (usize, usize, Vec<Option<Vec<u8>>>);
/// Quantiser cache key: (palette size, frame, quality, dither, transparent).
type QuantKey = (u32, usize, u16, bool, bool);

pub struct Fit<'a> {
    fmt: Format,
    o: FitOptions,
    axes: Vec<Axis>,
    n: usize,
    samples: Vec<usize>,
    pre_engine: Engine,
    pre_project: Project,
    pub dead: Vec<String>,
    refs: Vec<Img>,
    ref_engine: Engine,
    frames_cache: Vec<Option<Frame>>,
    lottie_base: Option<(Value, Vec<crate::export::LottieLayerMap>)>,
    raster_cache: HashMap<u32, RasterEntry>,
    cache: HashMap<Vec<usize>, Candidate>,
    quant_cache: Vec<(QuantKey, std::rc::Rc<Indexed>)>,
    order: Vec<Vec<usize>>,
    pub evals: usize,
    started: crate::clock::Stamp,
    time_limited: std::cell::Cell<bool>,
    /// while set, the time budget is not enforced (the final rebuild and the first fallback candidate must finish)
    grace: std::cell::Cell<bool>,
    src: &'a Project,
}

#[cfg(test)]
thread_local! {
    /// Test clock: the budget counts as spent once this many candidates were built (None = real clock).
    static TEST_EXPIRE_AFTER_EVALS: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

/// Error text of a candidate that was abandoned because `max_seconds` ran out.
const TIME_UP: &str = "time budget exhausted";

fn sample_list(n: usize, k: usize) -> Vec<usize> {
    let k = k.clamp(1, n.max(1));
    if k == 1 || n <= 1 {
        return vec![0];
    }
    let mut v: Vec<usize> = (0..k).map(|i| (i * (n - 1) + (k - 1) / 2) / (k - 1)).collect();
    v.dedup();
    v
}

impl<'a> Fit<'a> {
    pub fn new(src: &'a Project, o: FitOptions) -> Result<Fit<'a>, String> {
        let started = crate::clock::Stamp::now(); // the budget counts from here: reference renders are part of the run
        if o.budget == 0 {
            return Err("the size budget must be at least 1 byte".into());
        }
        if let Some(m) = o.min_ssim {
            if !(0.0..=1.0).contains(&m) {
                return Err("--min-ssim must be between 0 and 1".into());
            }
        }
        if o.format == Format::Webm && !ffmpeg_available() {
            return Err("webm needs ffmpeg on PATH (it is not available); choose gif or apng".into());
        }
        let ref_engine = Engine::new(src.clone()).map_err(|e| format!("project does not validate: {}", e.join("; ")))?;
        let n = src.frames.max(1) as usize;
        let (pre_project, dead) = if matches!(o.format, Format::Lottie | Format::Html | Format::Project) { drop_dead_layers(src) } else { (src.clone(), vec![]) };
        let pre_engine = Engine::new(pre_project.clone()).map_err(|e| format!("reduced project does not validate: {}", e.join("; ")))?;
        let samples = sample_list(n, o.sample_frames);
        let ro = RasterOptions { scale: 1.0, show_bones: false, transparent: false };
        let mut refs = Vec::new();
        for &f in &samples {
            let (w, h, d) = render_rgba(&ref_engine, f as f32, &ro);
            refs.push(Img::new(w as usize, h as usize, d)?);
        }
        let axes = axes(o.format);
        let order = chain(o.format);
        Ok(Fit { fmt: o.format, o, axes, n, samples, pre_engine, pre_project, dead, refs, ref_engine, frames_cache: (0..n).map(|_| None).collect(), lottie_base: None, raster_cache: HashMap::new(), cache: HashMap::new(), quant_cache: Vec::new(), order, evals: 0, started, time_limited: std::cell::Cell::new(false), grace: std::cell::Cell::new(false), src })
    }

    fn ensure_frame(&mut self, f: usize) {
        if self.frames_cache[f].is_none() {
            self.frames_cache[f] = Some(self.pre_engine.evaluate(f as f32));
        }
    }

    /// `Err(TIME_UP)` once the wall-clock budget is spent (and not in grace mode); remembered for the report.
    fn time_up(&self) -> Result<(), String> {
        if self.expired() {
            self.time_limited.set(true);
            return Err(TIME_UP.to_string());
        }
        Ok(())
    }

    /// The wall-clock budget is spent (never in grace mode or with `max_seconds` 0). Tests can replace the clock with
    /// a count of candidate builds (`TEST_EXPIRE_AFTER_EVALS`) to make the moment of expiry deterministic.
    fn expired(&self) -> bool {
        if self.grace.get() || self.o.max_seconds <= 0.0 {
            return false;
        }
        #[cfg(test)]
        {
            if let Some(n) = TEST_EXPIRE_AFTER_EVALS.with(|c| c.get()) {
                return self.evals >= n;
            }
        }
        self.started.elapsed().as_secs_f64() > self.o.max_seconds
    }

    fn fill_raster(&mut self, scale: f32, frames: &[usize]) -> Result<(), String> {
        let key = scale.to_bits();
        let n = self.n;
        self.raster_cache.entry(key).or_insert_with(|| (0, 0, (0..n).map(|_| None).collect()));
        let opts = RasterOptions { scale, show_bones: false, transparent: false };
        for &f in frames {
            if self.raster_cache[&key].2[f].is_none() {
                if self.expired() {
                    self.time_limited.set(true);
                    return Err(TIME_UP.to_string());
                }
                let (w, h, d) = render_rgba(&self.ref_engine, f as f32, &opts);
                let e = self.raster_cache.get_mut(&key).unwrap();
                e.0 = w as usize;
                e.1 = h as usize;
                e.2[f] = Some(d);
            }
        }
        if self.raster_cache.len() > 3 {
            let drop: Vec<u32> = self.raster_cache.keys().copied().filter(|&k| k != key).take(self.raster_cache.len() - 3).collect();
            for k in drop {
                self.raster_cache.remove(&k);
            }
        }
        Ok(())
    }

    fn sample_imgs_from_frames(&self, w: usize, h: usize, decoded: &[Vec<u8>], step: usize) -> Result<Vec<Img>, String> {
        let (rw, rh) = (self.refs[0].w, self.refs[0].h);
        let mut out = Vec::new();
        for &f in &self.samples {
            let i = (f / step).min(decoded.len().saturating_sub(1));
            let d = decoded.get(i).ok_or("decoder returned no frames")?;
            if d.len() != w * h * 4 {
                return Err("decoded frame has the wrong size".into());
            }
            out.push(Img::new(rw, rh, resize_rgba(d, w, h, rw, rh))?);
        }
        Ok(out)
    }

    /// Produce the export for `s` and the decoded sample frames at the reference size.
    fn produce(&mut self, s: &Settings) -> Result<(Vec<u8>, Vec<Img>), String> {
        let step = s.frame_step.max(1) as usize;
        let ro = RasterOptions { scale: 1.0, show_bones: false, transparent: false };
        match self.fmt {
            Format::Lottie => {
                if self.lottie_base.is_none() {
                    self.lottie_base = Some(crate::export::lottie_json_mapped(&self.pre_engine));
                }
                let (base, map) = self.lottie_base.clone().unwrap();
                self.time_up()?;
                let doc = lottie_reduce(&base, s);
                let bytes = serde_json::to_vec(&doc).map_err(|e| e.to_string())?;
                let parsed: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                let mut imgs = Vec::new();
                for k in 0..self.samples.len() {
                    self.time_up()?;
                    let f = self.samples[k];
                    self.ensure_frame(f);
                    let fr = lottie_model_frame(self.frames_cache[f].as_ref().unwrap(), &parsed, &map, f);
                    let pm = pixmap_from_frame(&self.pre_engine, &fr, f as f32, &ro);
                    imgs.push(Img::new(pm.width() as usize, pm.height() as usize, to_straight_rgba(&pm))?);
                }
                Ok((bytes, imgs))
            }
            Format::Html => {
                let disp: Vec<usize> = (0..self.n).step_by(step).collect();
                let mut sim: Vec<Frame> = Vec::with_capacity(disp.len());
                let mut strs: Vec<String> = Vec::with_capacity(disp.len());
                for &f in &disp {
                    self.time_up()?;
                    self.ensure_frame(f);
                    let mut fr = clone_frame(self.frames_cache[f].as_ref().unwrap());
                    simplify_frame_paths(&mut fr, s.path_eps as f64);
                    let svg = crate::svg::render_evaluated(&self.pre_engine, &fr, f as f32, &crate::svg::RenderOptions::default());
                    strs.push(svg_round(&svg, s.decimals));
                    sim.push(fr);
                }
                let mut uniq: Vec<String> = Vec::new();
                let mut seen: HashMap<String, usize> = HashMap::new();
                let mut x: Vec<usize> = Vec::with_capacity(strs.len());
                for st in strs {
                    let id = *seen.entry(st.clone()).or_insert_with(|| {
                        uniq.push(st);
                        uniq.len() - 1
                    });
                    x.push(id);
                }
                let identity = x.iter().enumerate().all(|(i, &v)| i == v);
                let p = &self.pre_project;
                let page = html_page(&p.name, p.width, p.fps / step as f32, &uniq, if identity { None } else { Some(&x) });
                let mut imgs = Vec::new();
                for &f in &self.samples {
                    self.time_up()?;
                    let m = f / step;
                    let mut fr = clone_frame(&sim[m]);
                    round_frame(&mut fr, s.decimals);
                    let pm = pixmap_from_frame(&self.pre_engine, &fr, (m * step) as f32, &ro);
                    imgs.push(Img::new(pm.width() as usize, pm.height() as usize, to_straight_rgba(&pm))?);
                }
                Ok((page.into_bytes(), imgs))
            }
            Format::Project => {
                let bytes = project_bytes(&self.pre_project, s)?;
                let p2: Project = serde_json::from_slice(&bytes).map_err(|e| format!("reduced project does not parse: {e}"))?;
                let e2 = Engine::new(p2).map_err(|e| format!("reduced project does not validate: {}", e.join("; ")))?;
                let mut imgs = Vec::new();
                for &f in &self.samples {
                    self.time_up()?;
                    let (w, h, d) = render_rgba(&e2, f as f32, &ro);
                    imgs.push(Img::new(w as usize, h as usize, d)?);
                }
                Ok((bytes, imgs))
            }
            Format::Gif | Format::Apng | Format::Webm => {
                let disp: Vec<usize> = (0..self.n).step_by(step).collect();
                self.fill_raster(s.scale, &disp)?;
                let (w, h, fps) = {
                    let c = &self.raster_cache[&s.scale.to_bits()];
                    (c.0, c.1, self.src.fps)
                };
                let c = &self.raster_cache[&s.scale.to_bits()];
                let refs: Vec<&[u8]> = disp.iter().map(|&f| c.2[f].as_deref().unwrap()).collect();
                let has_alpha = refs.iter().any(|f| f.chunks_exact(4).any(|p| p[3] < 128));
                let reserve = has_alpha || refs.len() > 1;
                self.time_up()?;
                let (bytes, decoded) = match self.fmt {
                    Format::Gif => {
                        let key = (s.scale.to_bits(), step, s.colors, s.dither, reserve);
                        let q = match self.quant_cache.iter().find(|(k, _)| *k == key) {
                            Some((_, q)) => q.clone(),
                            None => {
                                let q = std::rc::Rc::new(quantize(&refs, w, h, s.colors as usize, s.dither, reserve));
                                self.quant_cache.push((key, q.clone()));
                                if self.quant_cache.len() > 4 {
                                    self.quant_cache.remove(0);
                                }
                                q
                            }
                        };
                        self.time_up()?;
                        let b = gif_encode(&q, fps, step as u32, !has_alpha, s.delta_tol)?;
                        let (dw, dh, d) = gif_decode(&b)?;
                        if (dw, dh) != (w, h) {
                            return Err("decoded GIF has the wrong size".into());
                        }
                        (b, d)
                    }
                    Format::Apng => {
                        let b = if s.colors == 0 {
                            apng_encode_rgba(w, h, &refs, fps, step as u32)?
                        } else {
                            let key = (s.scale.to_bits(), step, s.colors, s.dither, reserve);
                            let q = match self.quant_cache.iter().find(|(k, _)| *k == key) {
                                Some((_, q)) => q.clone(),
                                None => {
                                    let q = std::rc::Rc::new(quantize(&refs, w, h, s.colors as usize, s.dither, reserve));
                                    self.quant_cache.push((key, q.clone()));
                                    if self.quant_cache.len() > 4 {
                                        self.quant_cache.remove(0);
                                    }
                                    q
                                }
                            };
                            apng_encode_indexed(&q, fps, step as u32, !has_alpha, 0)?
                        };
                        let (dw, dh, d) = apng_decode(&b)?;
                        if (dw, dh) != (w, h) {
                            return Err("decoded APNG has the wrong size".into());
                        }
                        (b, d)
                    }
                    _ => {
                        self.time_up()?;
                        let b = webm_encode(w, h, &refs, fps / step as f32, s.crf)?;
                        let d = webm_decode(&b, w, h)?;
                        (b, d)
                    }
                };
                let imgs = self.sample_imgs_from_frames(w, h, &decoded, 1)?;
                Ok((bytes, imgs))
            }
        }
    }

    /// Build candidate `pos` (cached).
    pub fn eval(&mut self, pos: &[usize]) -> Candidate {
        if let Some(c) = self.cache.get(pos) {
            return c.clone();
        }
        self.evals += 1;
        let settings = settings_at(self.fmt, pos);
        let c = match self.produce(&settings) {
            Ok((bytes, imgs)) => {
                let raw = bytes.len();
                let gz = gzip_len(&bytes);
                let gz_budget = gzip_budget_len(&bytes);
                let mut per = Vec::new();
                let mut err = None;
                for (k, img) in imgs.iter().enumerate() {
                    match percep::ssim(&self.refs[k], img) {
                        Ok(v) => per.push((self.samples[k], v)),
                        Err(e) => err = Some(e),
                    }
                }
                let worst = per.iter().cloned().fold((0usize, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });
                let mean = if per.is_empty() { 0.0 } else { per.iter().map(|p| p.1).sum::<f64>() / per.len() as f64 };
                Candidate {
                    pos: pos.to_vec(),
                    settings,
                    valid: err.is_none() && !per.is_empty(),
                    error: err,
                    raw,
                    gzip: gz,
                    budgeted: if self.o.measure == Measure::Raw { raw } else { gz_budget },
                    ssim_mean: mean,
                    ssim_min: if worst.1.is_finite() { worst.1 } else { 0.0 },
                    worst_frame: worst.0,
                    per_frame: per,
                }
            }
            Err(e) if e == TIME_UP => {
                // interrupted by the time budget: not a result, not remembered
                self.evals -= 1;
                return Candidate { pos: pos.to_vec(), settings, valid: false, error: Some(e), raw: 0, gzip: 0, budgeted: usize::MAX, ssim_mean: 0.0, ssim_min: 0.0, worst_frame: 0, per_frame: vec![] };
            }
            Err(e) => Candidate { pos: pos.to_vec(), settings, valid: false, error: Some(e), raw: 0, gzip: 0, budgeted: usize::MAX, ssim_mean: 0.0, ssim_min: 0.0, worst_frame: 0, per_frame: vec![] },
        };
        self.cache.insert(pos.to_vec(), c.clone());
        c
    }

    fn fits(&self, c: &Candidate) -> bool {
        c.valid && c.budgeted <= self.o.budget
    }
    fn good(&self, c: &Candidate) -> bool {
        self.fits(c) && self.o.min_ssim.is_none_or(|m| c.ssim_min >= m)
    }
    /// `a` strictly better than `b`: higher mean SSIM, then higher worst SSIM (differences below 1e-6 count as
    /// equal), then fewer reductions (smaller sum of rungs), then smaller, then lower position.
    fn better(a: &Candidate, b: &Candidate) -> bool {
        let eps = 1e-6;
        if (a.ssim_mean - b.ssim_mean).abs() > eps {
            return a.ssim_mean > b.ssim_mean;
        }
        if (a.ssim_min - b.ssim_min).abs() > eps {
            return a.ssim_min > b.ssim_min;
        }
        let (ra, rb): (usize, usize) = (a.pos.iter().sum(), b.pos.iter().sum());
        if ra != rb {
            return ra < rb;
        }
        if a.budgeted != b.budgeted {
            return a.budgeted < b.budgeted;
        }
        a.pos < b.pos
    }

    /// True once the wall-clock budget is spent (remembered for the report).
    fn out_of_time(&mut self) -> bool {
        if self.expired() {
            self.time_limited.set(true);
        }
        self.time_limited.get()
    }

    fn descend(&mut self, start: Vec<usize>) -> Vec<usize> {
        let lens: Vec<usize> = self.axes.iter().map(|a| a.values.len()).collect();
        let na = lens.len();
        let mut cur = start;
        let mut cur_c = self.eval(&cur);
        for _ in 0..60 {
            if self.evals >= self.o.max_evals || self.out_of_time() {
                break;
            }
            let mut best: Option<Candidate> = None;
            let consider = |me: &mut Self, q: Vec<usize>, best: &mut Option<Candidate>| {
                let c = me.eval(&q);
                if me.good(&c) && Self::better(&c, best.as_ref().unwrap_or(&cur_c)) {
                    *best = Some(c);
                }
            };
            for a in 0..na {
                if !self.axes[a].in_chain {
                    // a free knob: try each other value
                    for v in 0..lens[a] {
                        if v != cur[a] && self.evals < self.o.max_evals && !self.out_of_time() {
                            let mut q = cur.clone();
                            q[a] = v;
                            consider(self, q, &mut best);
                        }
                    }
                } else if cur[a] > 0 {
                    let mut q = cur.clone();
                    q[a] -= 1;
                    consider(self, q, &mut best);
                }
            }
            if best.is_none() {
                'trade: for a in 0..na {
                    if cur[a] == 0 || !self.axes[a].in_chain {
                        continue;
                    }
                    for b in 0..na {
                        if b != a && self.axes[b].in_chain && cur[b] + 1 < lens[b] {
                            if self.evals >= self.o.max_evals || self.out_of_time() {
                                break 'trade;
                            }
                            let mut q = cur.clone();
                            q[a] -= 1;
                            q[b] += 1;
                            consider(self, q, &mut best);
                        }
                    }
                }
            }
            match best {
                Some(c) => {
                    cur = c.pos.clone();
                    cur_c = c;
                }
                None => break,
            }
        }
        cur
    }

    /// All grid positions, axes thinned (endpoints kept) so that the product is at most `max`.
    pub fn grid_positions(&self, max: usize) -> Vec<Vec<usize>> {
        let mut idx: Vec<Vec<usize>> = self.axes.iter().map(|a| (0..a.values.len()).collect()).collect();
        while idx.iter().map(|v| v.len()).product::<usize>() > max {
            let (k, _) = idx.iter().enumerate().max_by(|a, b| a.1.len().cmp(&b.1.len()).then(b.0.cmp(&a.0))).unwrap();
            if idx[k].len() <= 2 {
                break;
            }
            let last = idx[k].len() - 1;
            let mut v: Vec<usize> = idx[k].iter().copied().enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, x)| x).collect();
            if v.last() != Some(&idx[k][last]) {
                v.push(idx[k][last]);
            }
            idx[k] = v;
        }
        let mut out = vec![vec![]];
        for v in &idx {
            let mut next = Vec::with_capacity(out.len() * v.len());
            for p in &out {
                for &x in v {
                    let mut q = p.clone();
                    q.push(x);
                    next.push(q);
                }
            }
            out = next;
        }
        out
    }

    /// Bisection over `chain` (rungs ordered from best to smallest) on measured sizes, with an empirical
    /// monotonicity check over everything probed and a linear-scan fallback.
    fn ladder(&mut self, chain: &[Vec<usize>], audit: bool) -> Ladder {
        let n = chain.len();
        let mut out = Ladder { idx: None, probes: Vec::new(), monotone: true, fallback: false, audit_agrees: None, len: n };
        let probe = |me: &mut Self, i: usize, probes: &mut Vec<(usize, usize)>| -> Candidate {
            let c = me.eval(&chain[i]);
            if !probes.iter().any(|p| p.0 == i) {
                probes.push((i, c.budgeted));
            }
            c
        };
        let c0 = probe(self, 0, &mut out.probes);
        if self.fits(&c0) {
            // fits at the top: nothing further down the chain can be better in quality
            out.idx = Some(0);
        } else {
            let cl = probe(self, n - 1, &mut out.probes);
            if self.fits(&cl) {
                let (mut lo, mut hi) = (0usize, n - 1);
                while hi - lo > 1 {
                    let mid = (lo + hi) / 2;
                    let c = probe(self, mid, &mut out.probes);
                    if self.fits(&c) {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                out.idx = Some(hi);
            } else {
                // a few extra probes to confirm the floor is the smallest
                for q in [n / 4, n / 2, 3 * n / 4] {
                    if q > 0 && q < n - 1 {
                        probe(self, q, &mut out.probes);
                    }
                }
            }
        }
        let monotone_of = |probes: &[(usize, usize)]| {
            let mut p = probes.to_vec();
            p.sort();
            p.windows(2).all(|w| w[1].1 <= w[0].1 || w[1].1 == usize::MAX && w[0].1 == usize::MAX)
        };
        out.monotone = monotone_of(&out.probes);
        if !out.monotone || audit {
            // linear scan of the chain: the lowest-index rung that fits
            for i in 0..n {
                probe(self, i, &mut out.probes);
            }
            // a rung the exhausted time budget kept from being built is unknown, which counts as not fitting
            let scan = (0..n).find(|&i| self.cache.get(&chain[i]).is_some_and(|c| self.fits(c)));
            if !out.monotone {
                out.fallback = true;
                out.idx = scan;
            } else {
                out.audit_agrees = Some(scan == out.idx);
                if scan != out.idx {
                    out.fallback = true;
                    out.idx = scan;
                }
            }
            out.monotone = monotone_of(&out.probes);
        }
        out
    }

    /// A chain ordered by measured efficiency. Each knob is scanned alone (others at their best rung); the
    /// chain then lowers, at every step, the knob whose next rung saves the most bytes per unit of SSIM lost.
    /// (Assumes the knobs act roughly independently; the descent afterwards does not.)
    fn adaptive_chain(&mut self) -> Vec<Vec<usize>> {
        if self.out_of_time() {
            return self.order.clone();
        }
        let na = self.axes.len();
        let chain_axes: Vec<usize> = (0..na).filter(|&a| self.axes[a].in_chain).collect();
        let mut scan: Vec<Vec<(f64, f64)>> = vec![Vec::new(); na];
        let base = self.eval(&vec![0usize; na]);
        for &a in &chain_axes {
            scan[a].push((base.budgeted as f64, base.ssim_mean));
            for r in 1..self.axes[a].values.len() {
                let mut q = vec![0usize; na];
                q[a] = r;
                let c = self.eval(&q);
                scan[a].push((if c.valid { c.budgeted as f64 } else { f64::INFINITY }, if c.valid { c.ssim_mean } else { 0.0 }));
            }
        }
        let mut pos = vec![0usize; na];
        let mut out = vec![pos.clone()];
        loop {
            let mut best: Option<(f64, usize)> = None;
            for &a in &chain_axes {
                let r = pos[a];
                if r + 1 >= scan[a].len() {
                    continue;
                }
                let ((s0, q0), (s1, q1)) = (scan[a][r], scan[a][r + 1]);
                let saved = if s1.is_infinite() || s0.is_infinite() { f64::NEG_INFINITY } else { s0 - s1 };
                let ratio = saved / (q0 - q1).max(1e-4);
                if best.is_none_or(|(b, _)| ratio > b) {
                    best = Some((ratio, a));
                }
            }
            let Some((_, a)) = best else { break };
            pos[a] += 1;
            out.push(pos.clone());
        }
        out
    }

    /// Run the search and assemble the result.
    pub fn run(mut self) -> Result<FitResult, String> {
        let budget = self.o.budget;
        let chosen: Option<Candidate>;
        let mut reason: Option<String> = None;
        let static_chain = self.order.clone();
        let mut primary = Ladder { idx: None, probes: Vec::new(), monotone: true, fallback: false, audit_agrees: None, len: static_chain.len() };
        let mut chain_kind = "static";
        let mut static_start: Option<Vec<usize>> = None;
        let mut adaptive_start: Option<Vec<usize>> = None;
        if self.o.search != Search::Grid {
            let audit = self.o.audit;
            primary = self.ladder(&static_chain, audit);
            if let Some(i) = primary.idx {
                let c = self.eval(&static_chain[i]);
                if self.good(&c) {
                    static_start = Some(static_chain[i].clone());
                }
            }
            let top_ok = primary.idx == Some(0) && static_start.is_some();
            if self.o.search == Search::Descent && !top_ok && primary.idx.is_some() && !self.out_of_time() {
                let ach = self.adaptive_chain();
                let lo = self.ladder(&ach, false);
                if let Some(i) = lo.idx {
                    let c = self.eval(&ach[i]);
                    if self.good(&c) {
                        adaptive_start = Some(ach[i].clone());
                    }
                }
                if lo.idx.is_some() {
                    primary = lo;
                    chain_kind = "adaptive";
                }
            }
        }
        let ladder_idx = primary.idx;
        let mut best_pos: Option<Vec<usize>> = static_start.clone().or(adaptive_start.clone());
        if self.o.search == Search::Descent {
            let mut scored: Vec<(Vec<usize>, Candidate)> = Vec::new();
            for q in [static_start.clone(), adaptive_start.clone()].into_iter().flatten() {
                if !scored.iter().any(|(p, _)| *p == q) {
                    let c = self.eval(&q);
                    scored.push((q, c));
                }
            }
            scored.sort_by(|a, b| if Self::better(&a.1, &b.1) { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater });
            let mut best: Option<Candidate> = None;
            for (q, _) in scored {
                let end = self.descend(q);
                let c = self.eval(&end);
                if self.good(&c) && best.as_ref().is_none_or(|b| Self::better(&c, b)) {
                    best = Some(c);
                }
            }
            best_pos = best.map(|c| c.pos).or(best_pos);
        }
        let (probes, monotone, fallback, audit_agrees, n) = (primary.probes.clone(), primary.monotone, primary.fallback, primary.audit_agrees, primary.len);
        // grid: asked for, or the only way left to satisfy the constraints
        let need_grid = self.o.search == Search::Grid || (self.o.search == Search::Descent && best_pos.is_none() && self.o.min_ssim.is_some() && ladder_idx.is_some());
        if need_grid {
            let mut gbest: Option<Candidate> = best_pos.as_ref().map(|p| self.eval(p));
            for p in self.grid_positions(self.o.grid_max) {
                if self.out_of_time() {
                    break;
                }
                let c = self.eval(&p);
                if self.good(&c) && gbest.as_ref().is_none_or(|b| Self::better(&c, b)) {
                    gbest = Some(c);
                }
            }
            best_pos = gbest.map(|c| c.pos);
        }
        if best_pos.is_none() && self.time_limited.get() {
            // the budget ran out before any candidate was finished: build the cheapest one (the chain floor) anyway,
            // so that there is a best-so-far result to return
            self.grace.set(true);
            if let Some(fp) = static_chain.last() {
                let c = self.eval(fp);
                if self.good(&c) {
                    best_pos = Some(fp.clone());
                }
            }
        }
        let met;
        if let Some(p) = best_pos {
            chosen = Some(self.eval(&p));
            met = true;
        } else {
            met = false;
            // best achievable: smallest valid size seen; if constraints were the problem, say so
            let all: Vec<Candidate> = self.cache.values().filter(|c| c.valid).cloned().collect();
            let smallest = all.iter().min_by(|a, b| a.budgeted.cmp(&b.budgeted).then(a.pos.cmp(&b.pos))).cloned();
            let fitting_any = all.iter().any(|c| c.budgeted <= budget);
            let time_note = if self.time_limited.get() { format!(" [time budget of {} s exhausted: the search was cut short, a longer --max-seconds may find a fit]", self.o.max_seconds) } else { String::new() };
            reason = Some(if fitting_any {
                format!("a candidate fits {budget} bytes but none reaches --min-ssim {:.4}", self.o.min_ssim.unwrap_or(0.0))
            } else {
                format!("the smallest export found is {} bytes ({}), above the budget of {budget} bytes: every knob is at its floor", smallest.as_ref().map_or(0, |c| c.budgeted), self.o.measure.name())
            });
            if self.o.search != Search::Grid {
                // an unmet size budget is only a statement about the chain floor and the probes
                if !fitting_any {
                    reason = reason.map(|r| format!("{r} (checked the chain floor and {} probes; sizes assumed monotone)", probes.len()));
                }
            }
            reason = reason.map(|r| format!("{r}{time_note}"));
            chosen = smallest.or_else(|| all.first().cloned());
            if chosen.is_none() {
                let errs: Vec<String> = self.cache.values().filter_map(|c| c.error.clone()).take(3).collect();
                return Err(format!("no candidate could be produced: {}", errs.join("; ")));
            }
        }
        let chosen = chosen.unwrap();
        // rebuild the chosen file (deterministic) and measure the full quality incl. CIEDE2000; this must finish
        self.grace.set(true);
        let (bytes, imgs) = self.produce(&chosen.settings)?;
        let mut per = Vec::new();
        let mut de_max: f64 = 0.0;
        for (k, img) in imgs.iter().enumerate() {
            let d = percep::compare(&self.refs[k], img, &percep::CompareOptions::default())?;
            de_max = de_max.max(d.max_de_raw);
            per.push((self.samples[k], d.ssim, d.max_de_raw));
        }
        let worst = per.iter().cloned().fold((0usize, f64::INFINITY, 0.0), |a, b| if b.1 < a.1 { b } else { a });
        let quality = Quality {
            ssim_mean: per.iter().map(|p| p.1).sum::<f64>() / per.len().max(1) as f64,
            ssim_min: worst.1,
            worst_frame: worst.0,
            per_frame: per,
            de_max,
        };
        let baseline = if self.o.baseline && !self.time_limited.get() { self.baseline().ok() } else { None };
        let last = self.axes.iter().map(|a| a.values.len() - 1).collect::<Vec<_>>();
        let floors: Vec<String> = self.axes.iter().enumerate().filter(|(i, a)| a.in_chain && chosen.pos[*i] == last[*i]).map(|(_, a)| a.name.to_string()).collect();
        let mut all: Vec<Candidate> = self.cache.values().cloned().collect();
        all.sort_by(|a, b| a.pos.cmp(&b.pos));
        Ok(FitResult {
            options: self.o.clone(),
            met,
            reason,
            chosen,
            bytes,
            quality,
            baseline,
            floors,
            evaluations: self.evals,
            time_limited: self.time_limited.get(),
            probes,
            chain_len: n,
            chain_kind,
            monotone,
            fallback_linear: fallback,
            audit_agrees,
            dead_layers: self.dead.clone(),
            sample_frames: self.samples.clone(),
            all,
            axes: self.axes.iter().map(|a| (a.name.to_string(), a.values.clone())).collect(),
        })
    }

    /// Size of the existing exporter at its default settings: (raw, gzip).
    fn baseline(&mut self) -> Result<(usize, usize), String> {
        let bytes: Vec<u8> = match self.fmt {
            Format::Lottie => crate::export::lottie_json(&self.ref_engine).to_string().into_bytes(),
            Format::Html => crate::export::html_player(&self.ref_engine).into_bytes(),
            Format::Gif => crate::export::gif_bytes(&self.ref_engine, 1.0)?,
            Format::Apng => crate::open_formats::apng_bytes(&self.ref_engine, 1.0)?,
            Format::Project => serde_json::to_vec(self.src).map_err(|e| e.to_string())?,
            Format::Webm => {
                let all: Vec<usize> = (0..self.n).collect();
                self.fill_raster(1.0, &all)?;
                let c = &self.raster_cache[&1f32.to_bits()];
                let refs: Vec<&[u8]> = all.iter().map(|&f| c.2[f].as_deref().unwrap()).collect();
                webm_encode(c.0, c.1, &refs, self.src.fps, 30)?
            }
        };
        Ok((bytes.len(), gzip_len(&bytes)))
    }
}

/// Fit `project` into the budget. See the module notes.
pub fn fit(project: &Project, o: FitOptions) -> Result<FitResult, String> {
    Fit::new(project, o)?.run()
}

// ------------------------------------------------------------------------------------------
// Reports
// ------------------------------------------------------------------------------------------

/// Non-dominated candidates (smaller budgeted size, higher mean SSIM), by index into `all`.
pub fn pareto_front(all: &[Candidate]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..all.len()).filter(|&i| all[i].valid).collect();
    idx.sort_by(|&a, &b| all[a].budgeted.cmp(&all[b].budgeted).then(all[b].ssim_mean.total_cmp(&all[a].ssim_mean)).then(all[a].pos.cmp(&all[b].pos)));
    let mut front = Vec::new();
    let mut best = f64::NEG_INFINITY;
    for i in idx {
        if all[i].ssim_mean > best + 1e-12 {
            best = all[i].ssim_mean;
            front.push(i);
        }
    }
    front
}

impl FitResult {
    pub fn to_json(&self) -> Value {
        let o = &self.options;
        let c = &self.chosen;
        json!({
            "schema": SCHEMA,
            "format": o.format.name(),
            "budget_bytes": o.budget,
            "measure": o.measure.name(),
            "met": self.met,
            "reason": self.reason,
            "search": {
                "strategy": o.search.name(),
                "evaluations": self.evaluations,
                "time_limited": self.time_limited,
                "max_seconds": o.max_seconds,
                "chain_length": self.chain_len,
                "chain_kind": self.chain_kind,
                "probes": self.probes.iter().map(|&(r, b)| json!({"rung": r, "bytes": if b == usize::MAX { Value::Null } else { json!(b) }})).collect::<Vec<_>>(),
                "monotone": self.monotone,
                "fallback_linear": self.fallback_linear,
                "audit_agrees": self.audit_agrees,
            },
            "chosen": {
                "position": c.pos,
                "settings": c.settings.to_json(),
                "raw_bytes": c.raw,
                "gzip_bytes": c.gzip,
                "budgeted_bytes": c.budgeted,
                "floors": self.floors,
            },
            "quality": {
                "metric": "ssim",
                "sample_frames": self.sample_frames,
                "ssim_mean": self.quality.ssim_mean,
                "ssim_min": self.quality.ssim_min,
                "worst_frame": self.quality.worst_frame,
                "max_ciede2000": self.quality.de_max,
                "min_ssim_required": o.min_ssim,
                "per_frame": self.quality.per_frame.iter().map(|&(f, s, d)| json!({"frame": f, "ssim": s, "max_ciede2000": d})).collect::<Vec<_>>(),
            },
            "baseline": self.baseline.map(|(r, g)| json!({"raw_bytes": r, "gzip_bytes": g, "note": "the existing exporter at its default settings"})),
            "dead_layers": self.dead_layers,
            "axes": self.axes.iter().map(|(n, v)| json!({"name": n, "values": v})).collect::<Vec<_>>(),
        })
    }

    /// CSV of every produced candidate with the Pareto front marked.
    pub fn pareto_csv(&self) -> String {
        let front: std::collections::HashSet<usize> = pareto_front(&self.all).into_iter().collect();
        let mut s = String::from("position,decimals,path_eps,key_tol,frame_step,scale,colors,dither,delta_tol,crf,raw_bytes,gzip_bytes,budgeted_bytes,ssim_mean,ssim_min,valid,fits,on_front,chosen\n");
        for (i, c) in self.all.iter().enumerate() {
            let t = &c.settings;
            s.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},{},{},{},{}\n",
                c.pos.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("-"),
                t.decimals,
                t.path_eps,
                t.key_tol,
                t.frame_step,
                t.scale,
                t.colors,
                t.dither as u8,
                t.delta_tol,
                t.crf,
                c.raw,
                c.gzip,
                if c.valid { c.budgeted.to_string() } else { String::new() },
                c.ssim_mean,
                c.ssim_min,
                c.valid as u8,
                (c.valid && c.budgeted <= self.options.budget) as u8,
                front.contains(&i) as u8,
                (c.pos == self.chosen.pos) as u8
            ));
        }
        s
    }
}

fn layer_bytes(l: &crate::project::Layer) -> usize {
    serde_json::to_vec(l).map_or(0, |v| v.len())
}

/// What contributes bytes: the project file per layer, image and channel; the baked Lottie per layer; the
/// HTML page per frame and per layer. `tol` is the key tolerance used for the "removable keys" estimate.
pub fn size_report(project: &Project, tol: f32) -> Result<Value, String> {
    let engine = Engine::new(project.clone()).map_err(|e| format!("project does not validate: {}", e.join("; ")))?;
    let pj = serde_json::to_vec(project).map_err(|e| e.to_string())?;
    // project file
    let mut layers: Vec<Value> = Vec::new();
    fn walk(ls: &[crate::project::Layer], prefix: &str, out: &mut Vec<Value>) {
        for l in ls {
            let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
            let img = match &l.kind {
                crate::project::LayerKind::Image { source, .. } => source.png_base64.as_ref().map_or(0, |b| b.len()),
                _ => 0,
            };
            out.push(json!({"layer": path, "json_bytes": layer_bytes(l), "embedded_image_bytes": img}));
            walk(l.children(), &path, out);
        }
    }
    walk(&project.layers, "", &mut layers);
    layers.sort_by(|a, b| b["json_bytes"].as_u64().cmp(&a["json_bytes"].as_u64()));
    let mut chans: Vec<Value> = Vec::new();
    for c in &project.channels {
        let bytes = serde_json::to_vec(c).map_or(0, |v| v.len());
        let (keys, removable) = match &c.source {
            Source::Keys { track } => (track.keys.len(), track.keys.len() - reduce_track(track, tol, project.frames).keys.len()),
            _ => (0, 0),
        };
        chans.push(json!({"channel": format!("{}.{:?}", c.target, c.prop), "json_bytes": bytes, "keys": keys, "removable_keys_at_tol": removable}));
    }
    chans.sort_by(|a, b| b["json_bytes"].as_u64().cmp(&a["json_bytes"].as_u64()));
    // baked Lottie
    let (doc, map) = crate::export::lottie_json_mapped(&engine);
    let lj = serde_json::to_vec(&doc).map_err(|e| e.to_string())?;
    let mut lottie_layers: Vec<Value> = doc
        .get("layers")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .enumerate()
                .map(|(i, l)| {
                    let mc = map.iter().find(|m| m.index == i).is_some_and(|m| m.matte_copy);
                    json!({"layer": l.get("nm").cloned().unwrap_or(Value::Null), "json_bytes": serde_json::to_vec(l).map_or(0, |v| v.len()), "matte_copy": mc})
                })
                .collect()
        })
        .unwrap_or_default();
    lottie_layers.sort_by(|a, b| b["json_bytes"].as_u64().cmp(&a["json_bytes"].as_u64()));
    // HTML
    let n = project.frames.max(1) as usize;
    let mut frame_bytes = Vec::with_capacity(n);
    let mut by_group: HashMap<String, usize> = HashMap::new();
    let mut defs = 0usize;
    for f in 0..n {
        let svg = crate::svg::render_frame(&engine, f as f32, &crate::svg::RenderOptions::default());
        frame_bytes.push(svg.len());
        if let (Some(a), Some(b)) = (svg.find("<defs>"), svg.find("</defs>")) {
            defs += b + 7 - a;
        }
        let mut parts = svg.split("<g id=\"");
        parts.next();
        for p in parts {
            if let Some(e) = p.find('"') {
                *by_group.entry(p[..e].to_string()).or_insert(0) += p.len().min(p.find("</g>").map_or(p.len(), |x| x + 4));
            }
        }
    }
    let html = crate::export::html_player(&engine);
    let mut groups: Vec<(String, usize)> = by_group.into_iter().collect();
    groups.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(json!({
        "project": {"json_bytes": pj.len(), "gzip_bytes": gzip_len(&pj), "layers": layers, "channels": chans, "key_tolerance": tol},
        "lottie": {"raw_bytes": lj.len(), "gzip_bytes": gzip_len(&lj), "layers": lottie_layers},
        "html": {"raw_bytes": html.len(), "gzip_bytes": gzip_len(html.as_bytes()), "frames": n,
                 "frame_bytes_min": frame_bytes.iter().min(), "frame_bytes_max": frame_bytes.iter().max(), "frame_bytes_mean": frame_bytes.iter().sum::<usize>() / n,
                 "defs_bytes_total": defs,
                 "layers": groups.iter().map(|(g, b)| json!({"layer": g, "svg_bytes_all_frames": b})).collect::<Vec<_>>()},
        "raster": {"frames": n, "width": project.width, "height": project.height,
                   "raw_rgba_bytes": (project.width * project.height) as usize * 4 * n,
                   "note": "GIF / APNG / WebM sizes depend on the knobs; use export-fit to measure"},
    }))
}

/// Human-readable table for [`size_report`] (top `top` rows of each list).
pub fn size_report_text(r: &Value, top: usize) -> String {
    let mut s = String::new();
    let u = |v: &Value, k: &str| v.get(k).and_then(Value::as_u64).unwrap_or(0);
    let p = &r["project"];
    s.push_str(&format!("project file: {} bytes ({} gzip)\n  by layer (JSON bytes; embedded image base64 in brackets):\n", u(p, "json_bytes"), u(p, "gzip_bytes")));
    for l in p["layers"].as_array().into_iter().flatten().take(top) {
        s.push_str(&format!("    {:>9}  {}{}\n", u(l, "json_bytes"), l["layer"].as_str().unwrap_or("?"), if u(l, "embedded_image_bytes") > 0 { format!("  [image {}]", u(l, "embedded_image_bytes")) } else { String::new() }));
    }
    s.push_str(&format!("  by channel (keys, removable at tol {}):\n", p["key_tolerance"]));
    for c in p["channels"].as_array().into_iter().flatten().take(top) {
        s.push_str(&format!("    {:>9}  {}  keys {} removable {}\n", u(c, "json_bytes"), c["channel"].as_str().unwrap_or("?"), u(c, "keys"), u(c, "removable_keys_at_tol")));
    }
    let l = &r["lottie"];
    s.push_str(&format!("baked Lottie: {} bytes ({} gzip), by layer:\n", u(l, "raw_bytes"), u(l, "gzip_bytes")));
    for x in l["layers"].as_array().into_iter().flatten().take(top) {
        s.push_str(&format!("    {:>9}  {}{}\n", u(x, "json_bytes"), x["layer"].as_str().unwrap_or("?"), if x["matte_copy"].as_bool() == Some(true) { "  (matte copy)" } else { "" }));
    }
    let h = &r["html"];
    s.push_str(&format!("HTML page: {} bytes ({} gzip), {} frames of {}..{} bytes (mean {}), gradient defs {} bytes in total; by layer over all frames:\n", u(h, "raw_bytes"), u(h, "gzip_bytes"), u(h, "frames"), u(h, "frame_bytes_min"), u(h, "frame_bytes_max"), u(h, "frame_bytes_mean"), u(h, "defs_bytes_total")));
    for x in h["layers"].as_array().into_iter().flatten().take(top) {
        s.push_str(&format!("    {:>9}  {}\n", u(x, "svg_bytes_all_frames"), x["layer"].as_str().unwrap_or("?")));
    }
    s.push_str(&format!("raster: {} frames of {}x{} = {} bytes uncompressed RGBA\n", u(&r["raster"], "frames"), r["raster"]["width"], r["raster"]["height"], u(&r["raster"], "raw_rgba_bytes")));
    s
}

#[cfg(test)]
mod tests {
    /// C12a: the gzip budget also holds for the system gzip at -9 and at its default -6.
    #[test]
    fn gzip_budget_holds_for_system_gzip() {
        use std::io::Write as _;
        use std::process::{Command, Stdio};
        if Command::new("gzip").arg("--version").output().is_err() {
            return; // no system gzip here
        }
        let e = crate::Engine::new(crate::demo::project()).unwrap();
        let lottie = crate::export::lottie_json(&e).to_string().into_bytes();
        let html = crate::export::html_player(&e).into_bytes();
        let project = serde_json::to_vec_pretty(&e.project).unwrap();
        for (name, data) in [("lottie", lottie), ("html", html), ("project", project)] {
            let budget = gzip_budget_len(&data);
            for level in ["-9", "-6", "-1"] {
                let mut c = Command::new("gzip").args([level, "-c"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
                let mut stdin = c.stdin.take().unwrap();
                let d = data.clone();
                let t = std::thread::spawn(move || {
                    let _ = stdin.write_all(&d);
                });
                let out = c.wait_with_output().unwrap();
                t.join().unwrap();
                match level {
                    "-9" => assert!(out.stdout.len() <= budget, "{name} gzip -9: {} > budget {budget}", out.stdout.len()),
                    // the default level is a weaker compressor: documented to be up to ~10 % larger
                    "-6" => assert!(out.stdout.len() * 100 <= budget * 110, "{name} gzip -6: {} vs budget {budget}", out.stdout.len()),
                    _ => {}
                }
            }
        }
    }

    use super::*;
    use crate::anim::{Ease, Extrapolate};
    use crate::project::{Channel, Layer, Prop};

    /// The demo rig, shrunk so the debug-build tests stay fast, 12 frames.
    fn small_demo() -> Project {
        let mut p = crate::containers::responsive(&crate::demo::project(), 96.0, 72.0).unwrap();
        p.frames = 8;
        p
    }

    /// Flat-colour scene: moving square on a background, a ball with a smooth bob.
    fn flat() -> Project {
        let mut p = Project::new("flat", 64.0, 48.0, 12.0, 16);
        p.background = Some(crate::style::Rgba::hex("#ffffff"));
        let sq = Layer::vector("sq", vec![VPath::rect("r", 6.0, 8.0, 14.0, 14.0).with_fill("#cc2222").with_stroke("#000000", 1.5)]);
        let ball = Layer::vector("ball", vec![VPath::circle("c", Vec2::new(40.0, 24.0), 7.0).with_fill("#2255cc")]);
        p.layers = vec![sq, ball];
        p.channels = vec![
            Channel::keys("sq", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (15.0, 30.0, Ease::Linear)])),
            Channel::keys("ball", Prop::LayerY, Track::with_keys(&[(0.0, 0.0, Ease::Smooth), (5.0, -10.0, Ease::Smooth), (10.0, 6.0, Ease::Smooth), (15.0, 0.0, Ease::Smooth)])),
        ];
        p
    }

    fn opts(fmt: Format, budget: usize) -> FitOptions {
        let mut o = FitOptions::new(fmt, budget);
        o.sample_frames = 2;
        o.baseline = false;
        o
    }

    fn chain_sizes(p: &Project, fmt: Format) -> Vec<usize> {
        use std::sync::{Mutex, OnceLock};
        type SizeCache = Mutex<HashMap<(Format, String, u32), Vec<usize>>>;
        static CACHE: OnceLock<SizeCache> = OnceLock::new();
        let key = (fmt, p.name.clone(), p.frames);
        if let Some(v) = CACHE.get_or_init(Default::default).lock().unwrap().get(&key) {
            return v.clone();
        }
        let mut f = Fit::new(p, opts(fmt, usize::MAX)).unwrap();
        let v: Vec<usize> = chain(fmt).iter().map(|pos| f.eval(pos).budgeted).collect();
        CACHE.get_or_init(Default::default).lock().unwrap().insert(key, v.clone());
        v
    }

    // ---- small utilities

    #[test]
    fn size_parsing() {
        assert_eq!(parse_size("500KB").unwrap(), 500_000);
        assert_eq!(parse_size("1.5 MB").unwrap(), 1_500_000);
        assert_eq!(parse_size("64000").unwrap(), 64_000);
        assert_eq!(parse_size("2kib").unwrap(), 2048);
        assert!(parse_size("fast").is_err() && parse_size("12 parsecs").is_err() && parse_size("0").is_err() && parse_size("").is_err());
    }

    #[test]
    fn precision_rounding_and_number_text() {
        assert_eq!(round_dec(1.2345678, 3), 1.235);
        assert_eq!(round_dec(-0.0004, 2), 0.0);
        assert!(round_dec(-0.0004, 2).is_sign_positive());
        assert_eq!(round_dec(5.5, 0), 6.0);
        assert_eq!(round_dec(0.1 + 0.2, 6), 0.3);
        assert_eq!(fmt_dec(12.30, 2), "12.3");
        assert_eq!(fmt_dec(60.0, 2), "60");
        assert_eq!(fmt_dec(-0.001, 2), "0");
        assert_eq!(fmt_dec(0.125, 2), "0.13");
        assert_eq!(num(3.0), json!(3));
        assert_eq!(num(3.25), json!(3.25));
    }

    #[test]
    fn svg_text_rounding_keeps_identifiers_colours_and_data() {
        let svg = "<svg viewBox=\"0 0 640 480\">\n<g id=\"g12\" opacity=\"1.000\">\n<path d=\"M60.00 412.567 C-0.004 3.14159 1.5-2.25 7 8\" fill=\"#123456\" stroke-width=\"3.456\" clip-path=\"url(#c1.5)\"/>\n<image href=\"data:image/png;base64,AB12.5+9\" transform=\"matrix(0.123456789 0 0 1.5 10.123456 20.5)\"/>\n</g>\n</svg>\n";
        let r = svg_round(svg, 1);
        assert!(r.contains("id=\"g12\"") && r.contains("fill=\"#123456\"") && r.contains("url(#c1.5)") && r.contains("AB12.5+9"));
        assert!(r.contains("d=\"M60 412.6 C0 3.1 1.5-2.3 7 8\""), "{r}");
        assert!(r.contains("stroke-width=\"3.5\""));
        assert!(!r.contains("opacity=\"1\"") && !r.contains('\n'), "{r}");
        assert!(r.contains("matrix(0.1235 0 0 1.5 10.1 20.5)"), "linear part keeps d+3 decimals: {r}");
        // idempotent
        assert_eq!(svg_round(&r, 1), r);
    }

    #[test]
    fn chain_is_a_proper_ladder() {
        for f in [Format::Lottie, Format::Html, Format::Gif, Format::Apng, Format::Webm, Format::Project] {
            let ch = chain(f);
            let ax = axes(f);
            assert_eq!(ch.len(), 1 + ax.iter().filter(|a| a.in_chain).map(|a| a.values.len() - 1).sum::<usize>());
            assert!(ch[0].iter().all(|&p| p == 0));
            assert_eq!(*ch.last().unwrap(), ax.iter().map(|a| if a.in_chain { a.values.len() - 1 } else { 0 }).collect::<Vec<_>>());
            for w in ch.windows(2) {
                let d: Vec<i64> = w[1].iter().zip(&w[0]).map(|(a, b)| *a as i64 - *b as i64).collect();
                assert_eq!(d.iter().filter(|&&x| x != 0).count(), 1, "each step moves one knob");
                assert_eq!(d.iter().sum::<i64>(), 1, "by exactly one rung");
            }
            // rung 0 reduces nothing
            let s0 = settings_at(f, &ch[0]);
            assert!(s0.path_eps == 0.0 && s0.key_tol == 0.0 && s0.frame_step == 1 && s0.scale == 1.0 && s0.delta_tol == 0 && s0.crf == 24, "{f:?}: {s0:?}");
            assert!(s0.decimals >= 2 && s0.drop_dead, "{f:?}: {s0:?}");
            // every axis list is strictly ordered from best to worst
            for a in ax.iter().filter(|a| a.in_chain && !(a.name == "colors" && a.values[0] == 0.0)) {
                assert!(a.values.len() >= 2, "{f:?} {}", a.name);
                let up = a.values[1] > a.values[0];
                assert!(a.values.windows(2).all(|w| (w[1] > w[0]) == up && w[1] != w[0]), "{f:?} {}", a.name);
            }
        }
        assert!(Format::parse("svg").unwrap_err().contains("not supported") && Format::parse("gltf").is_err() && Format::parse("mp3").is_err());
        assert_eq!(Format::from_path("a/b.GIF"), Some(Format::Gif));
        assert_eq!(Format::from_path("x.json"), Some(Format::Lottie));
        assert_eq!(Format::from_path("x.txt"), None);
    }

    // ---- geometry

    #[test]
    fn path_simplification_respects_its_tolerance() {
        // a circle with 64 anchors: smooth tangents, so many are redundant
        let n = 64;
        let r = 50.0;
        let nodes: Vec<PNode> = (0..n)
            .map(|k| {
                let a = k as f64 / n as f64 * std::f64::consts::TAU;
                let h = r * 4.0 / 3.0 * (std::f64::consts::TAU / n as f64 / 4.0).tan();
                PNode { p: [r * a.cos(), r * a.sin()], i: [h * a.sin(), -h * a.cos()], o: [-h * a.sin(), h * a.cos()] }
            })
            .collect();
        for eps in [0.05, 0.5, 2.0] {
            let mut fr = vec![nodes.clone()];
            let removed = simplify_paths(&mut fr, true, eps);
            assert_eq!(fr[0].len(), n - removed);
            assert!(fr[0].len() >= 3);
            if eps >= 0.5 {
                assert!(removed > 20, "eps {eps}: removed only {removed}");
            }
            // independent check: sampled distance of the new outline from the true circle <= eps + sampling slack
            let m = fr[0].len();
            let mut worst: f64 = 0.0;
            for k in 0..m {
                let c = seg_ctrl(&fr[0][k], &fr[0][(k + 1) % m]);
                for j in 0..=20 {
                    let q = cubic_at(&c, j as f64 / 20.0);
                    worst = worst.max((len2(q) - r).abs());
                }
            }
            assert!(worst <= eps * 1.5 + 0.05, "eps {eps}: deviation {worst}");
        }
        // off: unchanged
        let mut fr = vec![nodes.clone()];
        assert_eq!(simplify_paths(&mut fr, true, 0.0), 0);
        assert_eq!(fr[0], nodes);
    }

    #[test]
    fn collinear_vertices_go_corners_stay_and_open_ends_stay() {
        let c = |x: f64, y: f64| PNode { p: [x, y], i: [0.0, 0.0], o: [0.0, 0.0] };
        // open polyline with a collinear middle point and a real corner
        let mut fr = vec![vec![c(0.0, 0.0), c(5.0, 0.0), c(10.0, 0.0), c(10.0, 10.0)]];
        let removed = simplify_paths(&mut fr, false, 0.01);
        assert_eq!(removed, 1);
        assert_eq!(fr[0].iter().map(|n| n.p).collect::<Vec<_>>(), vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]);
        // a closed triangle keeps its 3 corners whatever the tolerance
        let mut tri = vec![vec![c(0.0, 0.0), c(10.0, 0.0), c(5.0, 8.0)]];
        assert_eq!(simplify_paths(&mut tri, true, 100.0), 0);
        // the same vertices go in every frame; a vertex that is needed in only one frame stays
        let mut two = vec![
            vec![c(0.0, 0.0), c(5.0, 0.0), c(10.0, 0.0), c(10.0, 10.0)],
            vec![c(0.0, 0.0), c(5.0, 3.0), c(10.0, 0.0), c(10.0, 10.0)],
        ];
        assert_eq!(simplify_paths(&mut two, false, 0.01), 0);
        assert_eq!(two[0].len(), 4);
        // mismatching topology: nothing happens
        let mut bad = vec![vec![c(0.0, 0.0), c(1.0, 0.0), c(2.0, 0.0)], vec![c(0.0, 0.0), c(2.0, 0.0)]];
        assert_eq!(simplify_paths(&mut bad, false, 1.0), 0);
    }

    // ---- key reduction

    fn lcg(seed: &mut u64) -> f32 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((*seed >> 33) as f32) / (1u64 << 31) as f32
    }

    #[test]
    fn key_reduction_bound_is_exact_for_every_ease_and_extrapolation() {
        let eases = [Ease::Linear, Ease::Step, Ease::EaseIn, Ease::EaseOut, Ease::EaseInOut, Ease::Smooth, Ease::CubicBezier { x1: 0.2, y1: 0.8, x2: 0.7, y2: 0.1 }, Ease::Back { overshoot: 1.2 }, Ease::Bounce];
        let extr = [Extrapolate::Hold, Extrapolate::Linear, Extrapolate::Loop, Extrapolate::PingPong, Extrapolate::LoopOffset];
        let mut seed = 7u64;
        let mut reduced_any = false;
        for trial in 0..60 {
            let n = 3 + (lcg(&mut seed) * 20.0) as usize;
            let mut t = Track::new();
            let mut f = (lcg(&mut seed) * 3.0).floor();
            for _ in 0..n {
                t.set_key(f, (lcg(&mut seed) - 0.5) * 100.0 * if trial % 3 == 0 { 0.01 } else { 1.0 }, eases[(lcg(&mut seed) * eases.len() as f32) as usize % eases.len()]);
                f += 1.0 + (lcg(&mut seed) * 4.0).floor() + if trial % 4 == 0 { 0.5 } else { 0.0 };
            }
            t.extrapolate = extr[trial % extr.len()];
            let frames = (f as u32 + 5).max(10);
            for tol in [0.0f32, 0.001, 0.05, 0.5, 5.0] {
                let r = reduce_track(&t, tol, frames);
                assert!(r.keys.len() <= t.keys.len() && r.keys.len() >= 2);
                assert_eq!(r.keys.first().unwrap().frame, t.keys.first().unwrap().frame);
                assert_eq!(r.keys.last().unwrap().frame, t.keys.last().unwrap().frame);
                // independent evaluation: every integer frame of the timeline and every original key frame
                let mut worst = 0.0f64;
                let mut probes: Vec<f32> = (0..frames).map(|x| x as f32).collect();
                probes.extend(t.keys.iter().map(|k| k.frame));
                for x in probes {
                    worst = worst.max((t.eval(x).unwrap() as f64 - r.eval(x).unwrap() as f64).abs());
                }
                assert!(worst <= tol as f64, "trial {trial} tol {tol}: deviation {worst}");
                assert_eq!(worst, track_deviation(&t, &r, frames));
                if tol == 0.0 {
                    assert_eq!(worst, 0.0);
                }
                reduced_any |= r.keys.len() < t.keys.len();
            }
        }
        assert!(reduced_any, "the reduction never removed a key");
        // a straight line collapses to its two ends
        let t = Track::with_keys(&[(0.0, 0.0, Ease::Linear), (1.0, 1.0, Ease::Linear), (2.0, 2.0, Ease::Linear), (3.0, 3.0, Ease::Linear)]);
        assert_eq!(reduce_track(&t, 1e-4, 4).keys.len(), 2);
        // a negative or NaN tolerance leaves the track alone
        assert_eq!(reduce_track(&t, f32::NAN, 4), t);
        assert_eq!(reduce_track(&t, -1.0, 4), t);
    }

    #[test]
    fn project_key_reduction_reports_and_bounds_every_channel() {
        let mut p = crate::demo::project();
        let before: usize = p.channels.iter().map(|c| if let Source::Keys { track } = &c.source { track.keys.len() } else { 0 }).sum();
        let orig = p.clone();
        let r = reduce_project_keys(&mut p, 0.5);
        assert_eq!(r.keys_before, before);
        assert!(r.keys_after <= r.keys_before && r.max_deviation <= 0.5);
        for (a, b) in orig.channels.iter().zip(&p.channels) {
            if let (Source::Keys { track: ta }, Source::Keys { track: tb }) = (&a.source, &b.source) {
                assert!(track_deviation(ta, tb, orig.frames) <= 0.5);
            }
        }
        let mut q = orig.clone();
        let z = reduce_project_keys(&mut q, 0.0);
        assert_eq!(z.max_deviation, 0.0);
        let mut e = orig.clone();
        let hi = reduce_project_keys(&mut e, 50.0);
        assert!(hi.keys_after < hi.keys_before, "a large tolerance must drop keys on the demo");
        Engine::new(e).expect("a reduced project still validates");
    }

    // ---- dead layers

    #[test]
    fn dead_layers_are_dropped_only_when_provably_invisible() {
        let mut p = flat();
        let mut hidden = Layer::vector("ghost", vec![VPath::rect("g", 1.0, 1.0, 5.0, 5.0).with_fill("#00ff00")]);
        hidden.visible = false;
        let mut clear = Layer::vector("clear", vec![VPath::rect("g", 30.0, 30.0, 5.0, 5.0).with_fill("#00ff00")]);
        clear.opacity = 0.0;
        // a hidden mask source must stay (it shapes another layer)
        let mut msrc = Layer::vector("msrc", vec![VPath::rect("m", 0.0, 0.0, 30.0, 30.0).with_fill("#ffffff")]);
        msrc.visible = false;
        let mut masked = Layer::vector("masked", vec![VPath::rect("g", 0.0, 0.0, 64.0, 48.0).with_fill("#ffaa00")]);
        masked.mask = Some(crate::project::MaskRef { layer: "msrc".into(), invert: false });
        p.layers.extend([hidden, clear, msrc, masked]);
        p.channels.push(Channel::keys("ghost", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (5.0, 9.0, Ease::Linear)])));
        let before = serde_json::to_string(&p).unwrap();
        let (q, dropped) = drop_dead_layers(&p);
        assert_eq!(serde_json::to_string(&p).unwrap(), before, "the input is not touched");
        assert_eq!(dropped, vec!["ghost".to_string(), "clear".to_string()]);
        assert!(q.find_layer("ghost").is_none() && q.find_layer("msrc").is_some() && q.find_layer("masked").is_some());
        assert!(!q.channels.iter().any(|c| c.target == "ghost"), "the dead layer's channels go with it");
        // pixel-identical picture
        let (e0, e1) = (Engine::new(p).unwrap(), Engine::new(q).unwrap());
        let o = RasterOptions { scale: 1.0, show_bones: false, transparent: false };
        for f in [0.0, 7.0, 15.0] {
            assert_eq!(render_rgba(&e0, f, &o).2, render_rgba(&e1, f, &o).2);
        }
        // nothing to drop: same project back
        let (r, d) = drop_dead_layers(&flat());
        assert!(d.is_empty() && serde_json::to_string(&r).unwrap() == serde_json::to_string(&flat()).unwrap());
    }

    // ---- quantiser, GIF, APNG

    fn test_frames(w: usize, h: usize, n: usize, colours: usize) -> Vec<Vec<u8>> {
        (0..n)
            .map(|k| {
                let mut v = vec![255u8; w * h * 4];
                for y in 0..h {
                    for x in 0..w {
                        let o = (y * w + x) * 4;
                        let band = ((x + y) / 3) % colours;
                        let moving = x >= k * 2 && x < k * 2 + 6 && (4..10).contains(&y);
                        let c = if moving { [250, 250, 20] } else { [(band * 40 % 256) as u8, (band * 97 % 256) as u8, (band * 23 % 256) as u8] };
                        v[o..o + 3].copy_from_slice(&c);
                    }
                }
                v
            })
            .collect()
    }

    #[test]
    fn median_cut_is_exact_for_few_colours_and_bounded_for_many() {
        let fr = test_frames(24, 16, 3, 5);
        let refs: Vec<&[u8]> = fr.iter().map(|f| f.as_slice()).collect();
        let pal = median_cut(&refs, 256);
        assert_eq!(pal.len(), 6, "5 band colours + the moving block");
        let rich: Vec<Vec<u8>> = (0..2).map(|k| (0..32 * 32).flat_map(|i| [(i * 7 + k) as u8, (i * 13) as u8, (i / 3) as u8, 255]).collect()).collect();
        let r2: Vec<&[u8]> = rich.iter().map(|f| f.as_slice()).collect();
        for max in [2, 16, 64] {
            let p = median_cut(&r2, max);
            assert!(!p.is_empty() && p.len() <= max);
            assert_eq!(p, median_cut(&r2, max), "deterministic");
        }
        // more colours never increases the quantisation error
        let err = |max: usize| {
            let q = quantize(&rich, 32, 32, max + 1, false, true);
            let mut e = 0u64;
            for (f, idx) in rich.iter().zip(&q.frames) {
                for (p, &i) in f.chunks_exact(4).zip(idx) {
                    let c = q.palette[i as usize];
                    e += (0..3).map(|a| (p[a] as i64 - c[a] as i64).pow(2) as u64).sum::<u64>();
                }
            }
            e
        };
        assert!(err(64) <= err(16) && err(16) <= err(4));
    }

    #[test]
    fn gif_roundtrip_is_exact_for_few_colours_and_deltas_are_lossless() {
        let fr = test_frames(40, 24, 6, 5);
        let q = quantize(&fr, 40, 24, 256, false, true);
        let plain = gif_encode(&q, 12.0, 1, false, 0).unwrap();
        let delta = gif_encode(&q, 12.0, 1, true, 0).unwrap();
        assert!(delta.len() < plain.len(), "deltas shrink a mostly static clip ({} vs {})", delta.len(), plain.len());
        let (w, h, a) = gif_decode(&plain).unwrap();
        let (_, _, b) = gif_decode(&delta).unwrap();
        assert_eq!((w, h, a.len()), (40, 24, 6));
        assert_eq!(a, b, "lossless delta decodes to the same pictures");
        for (orig, dec) in fr.iter().zip(&a) {
            assert_eq!(orig, dec, "few colours: the GIF holds the frames exactly");
        }
        // lossy delta: per-channel error to the indexed truth is bounded by the tolerance
        let q2 = quantize(&test_frames(40, 24, 6, 40), 40, 24, 64, false, true);
        let truth: Vec<Vec<u8>> = q2.frames.iter().map(|f| f.iter().flat_map(|&i| { let c = q2.palette[i as usize]; [c[0], c[1], c[2], 255] }).collect()).collect();
        let (_, _, lossy) = gif_decode(&gif_encode(&q2, 12.0, 1, true, 12).unwrap()).unwrap();
        let mut worst = 0i32;
        for (t, d) in truth.iter().zip(&lossy) {
            for (x, y) in t.chunks_exact(4).zip(d.chunks_exact(4)) {
                if x[0..3] != [0, 0, 0] || y[0..3] != [0, 0, 0] {
                    worst = worst.max((0..3).map(|c| (x[c] as i32 - y[c] as i32).abs()).max().unwrap());
                }
            }
        }
        assert!(worst <= 12, "lossy delta error {worst}");
        // real transparency: frames are drawn on a cleared canvas
        let mut t = test_frames(16, 16, 3, 4);
        for f in &mut t {
            for p in f.chunks_exact_mut(4).take(40) {
                p[3] = 0;
            }
        }
        let qt = quantize(&t, 16, 16, 256, false, true);
        let (_, _, dec) = gif_decode(&gif_encode(&qt, 12.0, 1, false, 0).unwrap()).unwrap();
        assert!(dec.iter().all(|f| f.chunks_exact(4).take(40).all(|p| p[3] == 0)));
        assert!(gif_decode(b"not a gif").is_err());
    }

    #[test]
    fn gif_delay_follows_step_and_fps() {
        assert_eq!(delay_cs(24.0, 1), 4);
        assert_eq!(delay_cs(24.0, 3), 13);
        assert_eq!(delay_cs(100.0, 1), 2, "GIF cannot go below 2 cs");
    }

    #[test]
    fn apng_roundtrip_indexed_and_truecolour() {
        let fr = test_frames(40, 24, 5, 5);
        let q = quantize(&fr, 40, 24, 256, false, true);
        assert!(q.palette.len() <= 8);
        for delta in [false, true] {
            let b = apng_encode_indexed(&q, 12.0, 1, delta, 0).unwrap();
            let (w, h, d) = apng_decode(&b).unwrap();
            assert_eq!((w, h, d.len()), (40, 24, 5));
            for (o, x) in fr.iter().zip(&d) {
                assert_eq!(o, x, "delta={delta}");
            }
        }
        let tc = apng_encode_rgba(40, 24, &fr, 12.0, 1).unwrap();
        let (_, _, d) = apng_decode(&tc).unwrap();
        assert_eq!(d, fr);
        // bit depths pack and unpack: 2, 3-4, 5-16 and more colours
        for ncol in [2usize, 4, 13, 40] {
            let f = test_frames(17, 9, 3, ncol);
            let q = quantize(&f, 17, 9, 256, false, true);
            let (_, _, d) = apng_decode(&apng_encode_indexed(&q, 12.0, 1, true, 0).unwrap()).unwrap();
            assert_eq!(d, f, "{ncol} colours");
        }
        assert!(apng_decode(b"nope").is_err());
        assert!(apng_encode_rgba(4, 4, &Vec::<Vec<u8>>::new(), 12.0, 1).is_err());
    }

    #[test]
    fn webm_encodes_and_decodes_when_ffmpeg_exists() {
        if !ffmpeg_available() {
            eprintln!("ffmpeg missing: webm test skipped");
            return;
        }
        let fr = test_frames(33, 21, 4, 5); // odd size: padded and cropped
        let b = webm_encode(33, 21, &fr, 12.0, 30).unwrap();
        assert_eq!(&b[..4], &[0x1a, 0x45, 0xdf, 0xa3], "EBML header");
        let d = webm_decode(&b, 33, 21).unwrap();
        assert_eq!(d.len(), 4);
        assert!(d.iter().all(|f| f.len() == 33 * 21 * 4));
        assert!(webm_decode(b"junk", 33, 21).is_err());
    }

    // ---- Lottie

    #[test]
    fn lottie_reduction_honours_precision_and_the_key_bound() {
        let p = small_demo();
        let e = Engine::new(p.clone()).unwrap();
        let (base, map) = crate::export::lottie_json_mapped(&e);
        let n = p.frames as usize;
        // the full rung is lossless to 1e-6 and strictly smaller than the raw exporter output
        let full = lottie_reduce(&base, &Settings::full());
        let (b0, b1) = (base.to_string().len(), full.to_string().len());
        assert!(b1 < b0, "lossless merging and number cleaning shrink the file ({b1} vs {b0})");
        let layers = base["layers"].as_array().unwrap();
        type Shapes = Vec<Vec<Vec<(bool, Vec<PNode>)>>>;
        let read_all = |doc: &Value| -> Shapes {
            doc["layers"].as_array().unwrap().iter().map(|l| {
                l.get("shapes").and_then(Value::as_array).into_iter().flatten().flat_map(|g| g["it"].as_array().unwrap().iter().filter(|i| i["ty"] == "sh")).map(|sh| (0..n).map(|f| read_shape_at(&sh["ks"], f).unwrap()).collect()).collect()
            }).collect()
        };
        let want = read_all(&base);
        let got = read_all(&full);
        let dev = |a: &Shapes, b: &Shapes| -> f64 {
            let mut worst: f64 = 0.0;
            for (la, lb) in a.iter().zip(b) {
                for (ta, tb) in la.iter().zip(lb) {
                    for (sa, sb) in ta.iter().zip(tb) {
                        assert_eq!((sa.0, sa.1.len()), (sb.0, sb.1.len()));
                        for (x, y) in sa.1.iter().zip(&sb.1) {
                            for c in 0..2 {
                                worst = worst.max((x.p[c] - y.p[c]).abs()).max((x.i[c] - y.i[c]).abs()).max((x.o[c] - y.o[c]).abs());
                            }
                        }
                    }
                }
            }
            worst
        };
        assert!(dev(&want, &got) <= 5.1e-7, "full rung deviation {}", dev(&want, &got));
        assert!(!layers.is_empty() && !map.is_empty());
        // precision: every non-integer number in the shape data has at most d decimals and moved by <= 0.5e-d
        for d in [0u8, 1, 2, 3] {
            let s = Settings { decimals: d, ..Settings::full() };
            let doc = lottie_reduce(&base, &s);
            let text = doc.to_string();
            let m = 10f64.powi(d as i32);
            let got = read_all(&doc);
            assert!(dev(&want, &got) <= 0.5 / m + 1e-9, "d={d}: {}", dev(&want, &got));
            for (_, v) in got.iter().flatten().flatten() {
                for nd in v {
                    for x in [nd.p[0], nd.p[1], nd.i[0], nd.i[1], nd.o[0], nd.o[1]] {
                        assert!((x * m - (x * m).round()).abs() < 1e-6, "d={d}: {x}");
                    }
                }
            }
            assert!(text.len() < b1 || d == 3, "d={d}");
        }
        // key reduction: linear keys within tol of the baked frames, fewer keyframes, valid JSON
        let count = |doc: &Value| doc.to_string().matches("\"t\":").count();
        let mut last = count(&full);
        for tol in [0.05f32, 0.5, 2.0] {
            let doc = lottie_reduce(&base, &Settings { key_tol: tol, ..Settings::full() });
            let got = read_all(&doc);
            let d = dev(&want, &got);
            assert!(d <= tol as f64 + 1e-5, "tol {tol}: deviation {d}");
            let c = count(&doc);
            assert!(c <= last, "tol {tol}: {c} keyframes vs {last}");
            last = c;
            let _: Value = serde_json::from_str(&doc.to_string()).unwrap();
        }
        assert!(last < count(&full), "a 2 unit tolerance must drop keyframes on the demo");
        // frame decimation: odd frames show the even frame before them (hold)
        let dec = lottie_reduce(&base, &Settings { frame_step: 2, ..Settings::full() });
        let got = read_all(&dec);
        for (la, lb) in want.iter().zip(&got) {
            for (ta, tb) in la.iter().zip(lb) {
                for f in 0..n {
                    let (a, b) = (&ta[f - f % 2], &tb[f]);
                    assert_eq!((a.0, a.1.len()), (b.0, b.1.len()));
                    for (x, y) in a.1.iter().zip(&b.1) {
                        for c in 0..2 {
                            assert!((x.p[c] - y.p[c]).abs() <= 5.1e-7 && (x.i[c] - y.i[c]).abs() <= 5.1e-7 && (x.o[c] - y.o[c]).abs() <= 5.1e-7, "frame {f}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn lottie_model_matches_the_engine_at_the_top_rung() {
        let p = flat();
        let o = opts(Format::Lottie, usize::MAX);
        let mut f = Fit::new(&p, o).unwrap();
        let c = f.eval(&chain(Format::Lottie)[0]);
        assert!(c.valid, "{:?}", c.error);
        assert!(c.ssim_min > 0.98, "baked Lottie read back vs the engine: {}", c.ssim_min);
        // constant tracks become static and layer opacity too
        let (doc, _) = crate::export::lottie_json_mapped(&Engine::new(p).unwrap());
        let red = lottie_reduce(&doc, &Settings::full());
        let kf = red["layers"][0]["shapes"][0]["it"][0]["ks"].clone();
        assert!(kf["a"] == 1 || kf["a"] == 0);
        assert!(red["layers"][0]["ks"]["o"]["a"] == 0, "constant opacity is static: {}", red["layers"][0]["ks"]["o"]);
    }

    // ---- HTML

    #[test]
    fn html_page_is_self_contained_dedupes_and_parses() {
        let p = flat();
        let o = opts(Format::Html, usize::MAX);
        let mut f = Fit::new(&p, o).unwrap();
        let (bytes, imgs) = f.produce(&Settings::full()).unwrap();
        let h = String::from_utf8(bytes).unwrap();
        assert!(h.starts_with("<!doctype html>") && h.contains("requestAnimationFrame") && h.contains("<svg"));
        assert!(!h.contains("src=\"http") && !h.contains("href=\"http"));
        assert_eq!(h.matches("</script>").count(), 1);
        assert_eq!(imgs.len(), 2);
        // a still project: one unique frame, an index list of the right length
        let mut still = flat();
        still.channels.clear();
        let mut f = Fit::new(&still, opts(Format::Html, usize::MAX)).unwrap();
        let (bytes, _) = f.produce(&Settings::full()).unwrap();
        let h = String::from_utf8(bytes).unwrap();
        assert_eq!(h.matches("<svg").count(), 1, "identical frames are stored once");
        assert!(h.contains(",X=[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]"), "{}", &h[h.find("const F").unwrap_or(0)..].chars().take(40).collect::<String>());
        // step halves the displayed frames
        let mut f = Fit::new(&p, opts(Format::Html, usize::MAX)).unwrap();
        let (bytes, _) = f.produce(&Settings { frame_step: 4, ..Settings::full() }).unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap().matches("<svg").count(), 4);
    }

    // ---- the search

    #[test]
    fn the_adaptive_chain_is_a_proper_chain_ordered_by_measured_efficiency() {
        let p = flat();
        let mut f = Fit::new(&p, opts(Format::Gif, 20_000)).unwrap();
        let ch = f.adaptive_chain();
        let ax = axes(Format::Gif);
        assert_eq!(ch.len(), 1 + ax.iter().filter(|a| a.in_chain).map(|a| a.values.len() - 1).sum::<usize>());
        assert!(ch[0].iter().all(|&x| x == 0));
        assert_eq!(*ch.last().unwrap(), ax.iter().map(|a| if a.in_chain { a.values.len() - 1 } else { 0 }).collect::<Vec<_>>());
        for w in ch.windows(2) {
            let d: Vec<i64> = w[1].iter().zip(&w[0]).map(|(a, b)| *a as i64 - *b as i64).collect();
            assert!(d.iter().filter(|&&x| x != 0).count() == 1 && d.iter().sum::<i64>() == 1);
        }
        // deterministic
        assert_eq!(ch, f.adaptive_chain());
        // and a descent-search report says which chain it bisected
        let r = fit(&p, opts(Format::Gif, 6_000)).unwrap();
        assert!(["static", "adaptive"].contains(&r.chain_kind) && r.to_json()["search"]["chain_kind"].is_string());
    }

    #[test]
    fn sizes_fall_along_the_ladders() {
        let p = small_demo();
        for fmt in [Format::Lottie, Format::Html, Format::Project] {
            let s = chain_sizes(&p, fmt);
            assert!(s.iter().all(|&b| b < usize::MAX), "{fmt:?}: every rung must build");
            let factor = if fmt == Format::Project { 1 } else { 2 };
            assert!(s.last().unwrap() * factor < s[0], "{fmt:?}: the floor is smaller than the top ({} vs {})", s.last().unwrap(), s[0]);
            // near-monotone: any increase is below 2 % of the previous rung (the search checks and falls back)
            for w in s.windows(2) {
                assert!(w[1] as f64 <= w[0] as f64 * 1.02, "{fmt:?}: size grew along the ladder: {s:?}");
            }
            let ups = s.windows(2).filter(|w| w[1] > w[0]).count();
            assert!(ups * 4 <= s.len(), "{fmt:?}: {ups} increases in {} rungs: {s:?}", s.len());
        }
        let g = flat();
        for fmt in [Format::Gif, Format::Apng] {
            let s = chain_sizes(&g, fmt);
            assert!(s.iter().all(|&b| b < usize::MAX), "{fmt:?}");
            assert!(s.last().unwrap() < &(s[0] / 2), "{fmt:?}: {s:?}");
            // monotone up to a small slack (palette changes can move LZW / deflate output slightly)
            for w in s.windows(2) {
                assert!(w[1] as f64 <= w[0] as f64 * 1.03 + 150.0, "{fmt:?}: {s:?}");
            }
        }
    }

    #[test]
    fn bisection_finds_the_same_rung_as_a_brute_force_scan() {
        let (mut bisected, mut scanned) = (0, 0);
        for (p, fmt) in [(small_demo(), Format::Lottie), (small_demo(), Format::Html), (small_demo(), Format::Project), (flat(), Format::Gif)] {
            let sizes = chain_sizes(&p, fmt);
            let mut sorted = sizes.clone();
            sorted.sort();
            for (bi, budget) in [sorted[sorted.len() / 4], sorted[sorted.len() * 3 / 4]].into_iter().enumerate() {
                let brute = sizes.iter().position(|&s| s <= budget).unwrap();
                let mut o = opts(fmt, budget);
                o.search = Search::Ladder;
                let plain = fit(&p, o.clone()).unwrap();
                assert_eq!(plain.chosen.pos, chain(fmt)[brute], "{fmt:?} budget {budget} (no audit)");
                o.audit = bi == 0;
                let r = fit(&p, o).unwrap();
                assert!(r.met, "{fmt:?} budget {budget}");
                assert_eq!(r.chosen.pos, chain(fmt)[brute], "{fmt:?} budget {budget}");
                assert!(bi != 0 || r.audit_agrees == Some(true) || r.fallback_linear);
                assert!(r.chosen.budgeted <= budget);
                assert_eq!(r.bytes.len(), r.chosen.raw);
                if plain.monotone && !plain.fallback_linear {
                    assert!(plain.probes.len() < sizes.len(), "bisection probes fewer rungs than the scan ({} of {})", plain.probes.len(), sizes.len());
                    bisected += 1;
                } else {
                    assert!(plain.fallback_linear, "a non-monotone chain must trigger the linear scan");
                    scanned += 1;
                }
            }
        }
        assert!(bisected > 0, "at least one chain must be searched by bisection (scanned {scanned})");
    }

    #[test]
    fn a_roomy_budget_gives_the_top_rung_and_descent_never_does_worse_than_the_ladder() {
        let p = small_demo();
        let big = fit(&p, opts(Format::Lottie, usize::MAX / 2)).unwrap();
        assert!(big.met && big.chosen.pos.iter().all(|&x| x == 0) && big.probes.len() == 1);
        let sizes = chain_sizes(&p, Format::Lottie);
        let budget = sizes[sizes.len() / 2];
        let mut lo = opts(Format::Lottie, budget);
        lo.search = Search::Ladder;
        let a = fit(&p, lo.clone()).unwrap();
        lo.search = Search::Descent;
        let b = fit(&p, lo.clone()).unwrap();
        assert!(b.chosen.budgeted <= budget && a.chosen.budgeted <= budget);
        assert!(b.chosen.ssim_mean >= a.chosen.ssim_mean - 1e-12, "descent {} vs ladder {}", b.chosen.ssim_mean, a.chosen.ssim_mean);
    }

    #[test]
    fn grid_is_at_least_as_good_as_descent_and_pareto_front_is_consistent() {
        let p = small_demo();
        let sizes = chain_sizes(&p, Format::Project);
        let budget = sizes[sizes.len() / 2];
        let mut o = opts(Format::Project, budget);
        o.search = Search::Descent;
        let d = fit(&p, o.clone()).unwrap();
        o.search = Search::Grid;
        let g = fit(&p, o).unwrap();
        assert!(g.met && g.chosen.budgeted <= budget);
        assert!(g.chosen.ssim_mean >= d.chosen.ssim_mean - 1e-12);
        let front = pareto_front(&g.all);
        assert!(!front.is_empty());
        for w in front.windows(2) {
            assert!(g.all[w[1]].budgeted >= g.all[w[0]].budgeted && g.all[w[1]].ssim_mean > g.all[w[0]].ssim_mean);
        }
        // nothing outside the front dominates a front member
        for &i in &front {
            assert!(!g.all.iter().any(|c| c.valid && c.budgeted < g.all[i].budgeted && c.ssim_mean > g.all[i].ssim_mean + 1e-12));
        }
        let csv = g.pareto_csv();
        assert_eq!(csv.lines().count(), g.all.len() + 1);
        assert!(csv.lines().next().unwrap().starts_with("position,decimals") && csv.contains(",1,"), "front rows are flagged");
        assert_eq!(csv.lines().filter(|l| l.ends_with(",1")).count(), 1, "exactly one chosen row");
    }

    #[test]
    fn an_impossible_budget_is_reported_with_the_best_achievable_size() {
        let p = small_demo();
        for fmt in [Format::Lottie, Format::Html] {
            let r = fit(&p, opts(fmt, 10)).unwrap();
            assert!(!r.met);
            assert!(r.reason.as_deref().unwrap().contains("floor"), "{:?}", r.reason);
            let floor = chain(fmt).last().unwrap().clone();
            assert_eq!(r.chosen.pos, floor, "the smallest export is the floor rung");
            assert_eq!(r.floors.len(), axes(fmt).iter().filter(|a| a.in_chain).count(), "every knob is at its floor");
            assert!(r.chosen.budgeted > 10 && r.bytes.len() == r.chosen.raw);
            let j = r.to_json();
            assert_eq!(j["met"], false);
            assert!(j["reason"].is_string());
        }
        assert!(fit(&p, FitOptions::new(Format::Lottie, 0)).is_err());
    }

    /// C12c: the search has a wall-clock budget (default 60 s). With the budget spent the descent does
    /// not run, the bisection result is kept and the report says so; 0 means unlimited.
    #[test]
    fn search_has_a_time_budget() {
        assert_eq!(FitOptions::new(Format::Gif, 1000).max_seconds, 60.0);
        let p = small_demo();
        let sizes = chain_sizes(&p, Format::Lottie);
        let budget = sizes[sizes.len() * 3 / 4];
        let mut o = opts(Format::Lottie, budget);
        o.max_seconds = 1e-9;
        let limited = fit(&p, o).unwrap();
        assert!(limited.time_limited && limited.met && limited.chosen.budgeted <= budget);
        assert_eq!(limited.to_json()["search"]["time_limited"], true);
        let mut o = opts(Format::Lottie, budget);
        o.max_seconds = 0.0;
        let full = fit(&p, o).unwrap();
        assert!(!full.time_limited);
        assert!(full.evaluations >= limited.evaluations, "{} vs {}", full.evaluations, limited.evaluations);
        assert!(full.quality.ssim_mean >= limited.quality.ssim_mean - 1e-9, "the unbounded search is at least as good");
    }

    /// M3 (wave 4): `max_seconds` used to be checked only between whole candidates, so a GIF search overshot it by
    /// minutes; and a budget that ran out part-way left rungs unbuilt which the linear scan then indexed directly
    /// (panic). The clock is replaced by a count of built candidates, so every moment of expiry is covered
    /// deterministically: never a panic, `time_limited` reported, best-so-far kept, and unbuilt rungs never "fit".
    #[test]
    fn an_expiring_budget_never_indexes_unbuilt_rungs() {
        let p = small_demo();
        let sizes = chain_sizes(&p, Format::Lottie);
        let budget = sizes[sizes.len() * 3 / 4];
        for search in [Search::Ladder, Search::Descent, Search::Grid] {
            for expire in 0..14usize {
                TEST_EXPIRE_AFTER_EVALS.with(|c| c.set(Some(expire)));
                let mut o = opts(Format::Lottie, budget);
                o.max_seconds = 1000.0; // only the test clock decides
                o.search = search;
                o.audit = expire % 2 == 0; // the audit forces the linear scan over the chain
                let r = fit(&p, o);
                TEST_EXPIRE_AFTER_EVALS.with(|c| c.set(None));
                match r {
                    Ok(r) => {
                        if r.met {
                            assert!(r.chosen.budgeted <= budget);
                        }
                        if r.evaluations > expire {
                            assert!(r.time_limited, "{search:?}/{expire}: the budget ran out but is not reported");
                            assert!(r.reason.as_deref().unwrap_or("").contains("time budget") || r.met, "{:?}", r.reason);
                        }
                    }
                    Err(e) => assert!(e.contains("budget"), "{search:?}/{expire}: {e}"),
                }
            }
        }
    }

    #[test]
    fn the_time_budget_is_enforced_inside_candidates() {
        let p = small_demo();
        // the real clock. A GIF search with a tiny budget stops within 1.5x the budget plus a fixed slack
        let budget_s = 0.2f64;
        let mut o = opts(Format::Gif, 1);
        o.max_seconds = budget_s;
        let t = crate::clock::Stamp::now();
        let r = fit(&p, o);
        let took = t.elapsed().as_secs_f64();
        assert!(took < budget_s * 1.5 + 3.0, "took {took} s for a {budget_s} s budget");
        if let (Ok(r), true) = (r, took > budget_s) {
            assert!(r.time_limited && r.reason.as_deref().unwrap_or("").contains("time budget"), "{:?}", r.reason);
        }
    }

    #[test]
    fn min_ssim_is_a_hard_constraint() {
        let p = small_demo();
        let sizes = chain_sizes(&p, Format::Lottie);
        let budget = sizes[sizes.len() * 3 / 4];
        let free = fit(&p, opts(Format::Lottie, budget)).unwrap();
        assert!(free.met);
        // ask for slightly more quality than the best fit can have
        let mut o = opts(Format::Lottie, budget);
        o.min_ssim = Some(1.0);
        o.search = Search::Ladder;
        let r = fit(&p, o).unwrap();
        assert!(!r.met && r.reason.as_deref().unwrap().contains("min-ssim"), "{:?}", r.reason);
        // an achievable floor is honoured
        let mut o = opts(Format::Lottie, budget);
        o.min_ssim = Some((free.quality.ssim_min - 0.01).max(0.0));
        let r = fit(&p, o.clone()).unwrap();
        assert!(r.met && r.quality.ssim_min >= o.min_ssim.unwrap() - 1e-9);
        let mut bad = opts(Format::Lottie, budget);
        bad.min_ssim = Some(1.5);
        assert!(fit(&p, bad).is_err());
    }

    #[test]
    fn gzip_measure_budgets_the_compressed_size() {
        let p = small_demo();
        let raw_top = chain_sizes(&p, Format::Html)[0];
        let mut o = opts(Format::Html, raw_top / 6);
        o.measure = Measure::Gzip;
        o.search = Search::Ladder;
        let r = fit(&p, o).unwrap();
        assert!(r.met, "{:?}", r.reason);
        assert!(r.chosen.budgeted <= raw_top / 6 && r.chosen.budgeted >= r.chosen.gzip + r.chosen.gzip / 100 + 32);
        assert!(r.chosen.raw > r.chosen.gzip, "the raw file is bigger than the budget while its gzip fits");
        assert_eq!(gzip_len(&r.bytes), r.chosen.gzip);
        assert_eq!(r.to_json()["measure"], "gzip");
    }

    #[test]
    fn the_search_is_deterministic_and_leaves_the_project_alone() {
        let p = small_demo();
        let before = serde_json::to_string(&p).unwrap();
        let sizes = chain_sizes(&p, Format::Lottie);
        let o = opts(Format::Lottie, sizes[sizes.len() / 2]);
        let a = fit(&p, o.clone()).unwrap();
        let b = fit(&p, o).unwrap();
        assert_eq!(a.bytes, b.bytes);
        assert_eq!(a.to_json(), b.to_json());
        assert_eq!(a.pareto_csv(), b.pareto_csv());
        assert_eq!(serde_json::to_string(&p).unwrap(), before, "source project untouched");
    }

    #[test]
    fn report_schema() {
        let p = flat();
        let mut o = opts(Format::Gif, 8000);
        o.baseline = true;
        o.search = Search::Ladder;
        let r = fit(&p, o).unwrap();
        let j = r.to_json();
        assert_eq!(j["schema"], SCHEMA);
        for k in ["format", "budget_bytes", "measure", "met", "reason", "search", "chosen", "quality", "baseline", "dead_layers", "axes"] {
            assert!(j.get(k).is_some(), "missing {k}");
        }
        for k in ["strategy", "evaluations", "chain_length", "probes", "monotone", "fallback_linear", "audit_agrees"] {
            assert!(j["search"].get(k).is_some(), "search.{k}");
        }
        for k in ["position", "settings", "raw_bytes", "gzip_bytes", "budgeted_bytes", "floors"] {
            assert!(j["chosen"].get(k).is_some(), "chosen.{k}");
        }
        for k in ["metric", "sample_frames", "ssim_mean", "ssim_min", "worst_frame", "max_ciede2000", "per_frame", "min_ssim_required"] {
            assert!(j["quality"].get(k).is_some(), "quality.{k}");
        }
        assert!(j["quality"]["per_frame"].as_array().unwrap().len() == r.sample_frames.len());
        assert!(j["baseline"]["raw_bytes"].as_u64().unwrap() > 0);
        assert_eq!(j["format"], "gif");
        assert!(j["quality"]["ssim_min"].as_f64().unwrap() <= j["quality"]["ssim_mean"].as_f64().unwrap() + 1e-12);
    }

    #[test]
    fn gif_and_apng_quality_falls_with_the_palette() {
        let p = small_demo();
        let mut f = Fit::new(&p, opts(Format::Gif, usize::MAX)).unwrap();
        let ch = chain(Format::Gif);
        let top = f.eval(&ch[0]);
        let low = f.eval(ch.last().unwrap());
        assert!(top.valid && low.valid);
        assert!(top.ssim_mean > low.ssim_mean && top.ssim_mean > 0.9, "top {} low {}", top.ssim_mean, low.ssim_mean);
        assert!(low.budgeted < top.budgeted);
    }

    #[test]
    fn webm_ladder_when_ffmpeg_exists() {
        if !ffmpeg_available() {
            return;
        }
        let p = flat();
        let mut f = Fit::new(&p, opts(Format::Webm, usize::MAX)).unwrap();
        let ch = chain(Format::Webm);
        let a = f.eval(&ch[0]);
        let b = f.eval(ch.last().unwrap());
        assert!(a.valid && b.valid, "{:?} {:?}", a.error, b.error);
        assert!(b.budgeted < a.budgeted && a.ssim_mean >= b.ssim_mean);
    }

    #[test]
    fn size_report_names_the_contributors() {
        let p = small_demo();
        let r = size_report(&p, 0.05).unwrap();
        assert!(r["project"]["json_bytes"].as_u64().unwrap() > 1000);
        assert!(!r["project"]["layers"].as_array().unwrap().is_empty() && !r["project"]["channels"].as_array().unwrap().is_empty());
        assert!(r["lottie"]["raw_bytes"].as_u64().unwrap() > 0 && r["html"]["frames"] == 8);
        assert!(!r["html"]["layers"].as_array().unwrap().is_empty());
        let lj = r["project"]["layers"].as_array().unwrap();
        for w in lj.windows(2) {
            assert!(w[0]["json_bytes"].as_u64() >= w[1]["json_bytes"].as_u64(), "sorted by size");
        }
        let t = size_report_text(&r, 3);
        assert!(t.contains("project file") && t.contains("baked Lottie") && t.contains("HTML page"));
    }

    #[test]
    fn invalid_options_are_errors_not_panics() {
        let p = flat();
        let mut o = FitOptions::new(Format::Gif, 1000);
        o.min_ssim = Some(-0.1);
        assert!(Fit::new(&p, o).is_err());
        assert!(Measure::parse("zip").is_err() && Search::parse("random").is_err());
        let mut bad = flat();
        bad.channels.push(Channel::keys("nobody", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear)])));
        assert!(Fit::new(&bad, FitOptions::new(Format::Lottie, 1000)).is_err());
        assert_eq!(sample_list(1, 8), vec![0]);
        assert_eq!(sample_list(10, 3), vec![0, 5, 9]);
        assert_eq!(sample_list(48, 1), vec![0]);
    }
}
