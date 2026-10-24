// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Formats, measure, settings, axes and the quality ladder (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Formats, measure, settings, axes

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

pub(super) fn ax(name: &'static str, values: &[f64], set: fn(&mut Settings, f64)) -> Axis {
    Axis { name, values: values.to_vec(), set, in_chain: true }
}

pub(super) fn ax_free(name: &'static str, values: &[f64], set: fn(&mut Settings, f64)) -> Axis {
    Axis { name, values: values.to_vec(), set, in_chain: false }
}

pub(super) const SCALES: [f64; 8] = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3];

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
