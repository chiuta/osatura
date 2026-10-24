// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Small utilities: gzip sizes, rounding, resizing (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Small utilities

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

pub(super) fn num(x: f64) -> Value {
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
pub(super) fn fmt_dec(x: f64, d: u8) -> String {
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

pub(super) fn lerp(a: f64, b: f64, t: f64) -> f64 {
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
