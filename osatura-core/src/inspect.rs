// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Project inspection: statistics, semantic diff, and golden-frame regression hashes.
//!
//! Golden files store an exact FNV-1a hash per frame *and* a 16×16 thumbnail so results can
//! still be accepted within a tolerance on another CPU/platform where floating-point rounding
//! differs by a pixel.

use crate::engine::Engine;
use crate::project::{Layer, Project, Source};
use crate::raster::{render_rgba, RasterOptions};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Stats {
    pub name: String,
    pub size: (f32, f32),
    pub fps: f32,
    pub frames: u32,
    pub seconds: f32,
    pub bones: usize,
    pub ik_chains: usize,
    pub constraints: usize,
    pub springs: usize,
    pub layers: usize,
    pub channels: usize,
    pub keyed_channels: usize,
    pub keys: usize,
    pub markers: usize,
}

fn count_layers(ls: &[Layer]) -> usize {
    ls.iter().map(|l| 1 + count_layers(l.children())).sum()
}

pub fn stats(p: &Project) -> Stats {
    let mut keyed = 0;
    let mut keys = 0;
    for c in &p.channels {
        if let Source::Keys { track } = &c.source {
            keyed += 1;
            keys += track.keys.len();
        }
    }
    Stats {
        name: p.name.clone(),
        size: (p.width, p.height),
        fps: p.fps,
        frames: p.frames,
        seconds: p.frames as f32 / p.fps.max(1e-6),
        bones: p.bones.len(),
        ik_chains: p.ik.len(),
        constraints: p.constraints.len(),
        springs: p.springs.len(),
        layers: count_layers(&p.layers),
        channels: p.channels.len(),
        keyed_channels: keyed,
        keys,
        markers: p.markers.len(),
    }
}

fn collect_layers(ls: &[Layer], prefix: &str, out: &mut BTreeMap<String, Value>) {
    for l in ls {
        let path = if prefix.is_empty() {
            l.name.clone()
        } else {
            format!("{prefix}/{}", l.name)
        };
        let mut v = serde_json::to_value(l).unwrap_or(Value::Null);
        // children are compared as their own entries
        if let Value::Object(m) = &mut v {
            m.remove("children");
        }
        out.insert(path.clone(), v);
        collect_layers(l.children(), &path, out);
    }
}

fn named<T: Serialize>(items: &[T], key: impl Fn(&T) -> String) -> BTreeMap<String, Value> {
    items
        .iter()
        .map(|i| (key(i), serde_json::to_value(i).unwrap_or(Value::Null)))
        .collect()
}

fn diff_maps(
    kind: &str,
    a: &BTreeMap<String, Value>,
    b: &BTreeMap<String, Value>,
    out: &mut Vec<String>,
) {
    for (k, va) in a {
        match b.get(k) {
            None => out.push(format!("- {kind} '{k}' removed")),
            Some(vb) if vb != va => {
                out.push(format!("~ {kind} '{k}' changed{}", describe_change(va, vb)))
            }
            _ => {}
        }
    }
    for k in b.keys().filter(|k| !a.contains_key(*k)) {
        out.push(format!("+ {kind} '{k}' added"));
    }
}

fn describe_change(a: &Value, b: &Value) -> String {
    let (Value::Object(x), Value::Object(y)) = (a, b) else {
        return String::new();
    };
    let mut fields: Vec<&str> = x
        .keys()
        .chain(y.keys())
        .map(String::as_str)
        .filter(|k| x.get(*k) != y.get(*k))
        .collect();
    fields.sort();
    fields.dedup();
    if fields.is_empty() {
        String::new()
    } else {
        format!(" ({})", fields.join(", "))
    }
}

/// Human-readable semantic differences between two projects (`+` added, `-` removed, `~` changed).
pub fn diff(a: &Project, b: &Project) -> Vec<String> {
    let mut out = Vec::new();
    let mut setting = |name: &str, x: String, y: String| {
        if x != y {
            out.push(format!("~ {name}: {x} → {y}"));
        }
    };
    setting("name", a.name.clone(), b.name.clone());
    setting(
        "size",
        format!("{}×{}", a.width, a.height),
        format!("{}×{}", b.width, b.height),
    );
    setting("fps", a.fps.to_string(), b.fps.to_string());
    setting("frames", a.frames.to_string(), b.frames.to_string());
    setting(
        "camera",
        serde_json::to_string(&a.camera).unwrap_or_default(),
        serde_json::to_string(&b.camera).unwrap_or_default(),
    );
    setting(
        "background",
        format!("{:?}", a.background),
        format!("{:?}", b.background),
    );
    diff_maps(
        "bone",
        &named(&a.bones, |x| x.name.clone()),
        &named(&b.bones, |x| x.name.clone()),
        &mut out,
    );
    diff_maps(
        "ik",
        &named(&a.ik, |x| x.name.clone()),
        &named(&b.ik, |x| x.name.clone()),
        &mut out,
    );
    diff_maps(
        "constraint",
        &named(&a.constraints, |x| x.name.clone()),
        &named(&b.constraints, |x| x.name.clone()),
        &mut out,
    );
    diff_maps(
        "spring",
        &named(&a.springs, |x| x.bone.clone()),
        &named(&b.springs, |x| x.bone.clone()),
        &mut out,
    );
    let (mut la, mut lb) = (BTreeMap::new(), BTreeMap::new());
    collect_layers(&a.layers, "", &mut la);
    collect_layers(&b.layers, "", &mut lb);
    diff_maps("layer", &la, &lb, &mut out);
    let ck = |c: &crate::project::Channel| format!("{}:{:?}", c.target, c.prop);
    diff_maps(
        "channel",
        &named(&a.channels, ck),
        &named(&b.channels, ck),
        &mut out,
    );
    diff_maps(
        "marker",
        &named(&a.markers, |m| m.name.clone()),
        &named(&b.markers, |m| m.name.clone()),
        &mut out,
    );
    out
}

// ---- golden frames ---------------------------------------------------------------------------

const THUMB: usize = 16;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoldenFrame {
    pub hash: String,
    /// 16×16 RGBA box average, row-major.
    pub thumb: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Golden {
    pub project: String,
    pub scale: f32,
    pub frames: Vec<GoldenFrame>,
}

pub fn fnv1a(data: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn thumbnail(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut sum = vec![0u32; THUMB * THUMB * 4];
    let mut cnt = vec![0u32; THUMB * THUMB];
    for y in 0..h {
        for x in 0..w {
            let cell = (y * THUMB / h.max(1)).min(THUMB - 1) * THUMB
                + (x * THUMB / w.max(1)).min(THUMB - 1);
            cnt[cell] += 1;
            for c in 0..4 {
                sum[cell * 4 + c] += rgba[(y * w + x) * 4 + c] as u32;
            }
        }
    }
    (0..THUMB * THUMB * 4)
        .map(|i| (sum[i] / cnt[i / 4].max(1)) as u8)
        .collect()
}

fn thumb_distance(a: &[u8], b: &[u8]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 1.0;
    }
    a.iter()
        .zip(b)
        .map(|(&x, &y)| (x as f32 - y as f32).abs())
        .sum::<f32>()
        / (a.len() as f32 * 255.0)
}

pub fn golden_record(engine: &Engine, scale: f32) -> Golden {
    let opts = RasterOptions {
        scale,
        show_bones: false,
        transparent: false,
    };
    let frames = (0..engine.project.frames)
        .map(|f| {
            let (w, h, px) = render_rgba(engine, f as f32, &opts);
            GoldenFrame {
                hash: format!("{:016x}", fnv1a(&px)),
                thumb: thumbnail(&px, w as usize, h as usize),
            }
        })
        .collect();
    Golden {
        project: engine.project.name.clone(),
        scale,
        frames,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GoldenMismatch {
    pub frame: usize,
    /// 0..1 mean absolute thumbnail difference.
    pub distance: f32,
    /// Thumbnail cells (of 256) whose average colour differs. The golden file keeps only a 16x16 thumbnail and a hash of the
    /// full-resolution pixels, so this is a lower bound: a change below the thumbnail's resolution can leave it at 0 while
    /// the hash still differs.
    pub cells_differing: usize,
    /// Largest per-channel difference (0..255) between corresponding thumbnail values.
    pub max_diff: u8,
    /// The frame exists on only one side (the project and the golden have different lengths).
    pub missing: bool,
}

/// Frames that differ from the golden file. The comparison is exact by default: with `tolerance <= 0` any difference
/// of the rendered pixels (a different hash) is a mismatch, which is right for the deterministic CPU rasteriser.
/// With `tolerance > 0` a frame whose hash differs still passes when the mean absolute difference of the 16x16
/// thumbnails is within `tolerance` (a coarse check that small details can slip through).
pub fn golden_check(engine: &Engine, golden: &Golden, tolerance: f32) -> Vec<GoldenMismatch> {
    let now = golden_record(engine, golden.scale);
    let mut bad = Vec::new();
    for i in 0..now.frames.len().max(golden.frames.len()) {
        match (now.frames.get(i), golden.frames.get(i)) {
            (Some(a), Some(b)) => {
                if a.hash != b.hash {
                    let d = thumb_distance(&a.thumb, &b.thumb);
                    if tolerance <= 0.0 || d > tolerance {
                        let n = a.thumb.len().min(b.thumb.len());
                        let cells = (0..n / 4)
                            .filter(|&c| a.thumb[c * 4..c * 4 + 4] != b.thumb[c * 4..c * 4 + 4])
                            .count();
                        let max_diff = a
                            .thumb
                            .iter()
                            .zip(&b.thumb)
                            .map(|(&x, &y)| x.abs_diff(y))
                            .max()
                            .unwrap_or(0);
                        bad.push(GoldenMismatch {
                            frame: i,
                            distance: d,
                            cells_differing: cells,
                            max_diff,
                            missing: false,
                        });
                    }
                }
            }
            _ => bad.push(GoldenMismatch {
                frame: i,
                distance: 1.0,
                cells_differing: 0,
                max_diff: 255,
                missing: true,
            }),
        }
    }
    bad
}

/// Side-by-side review image for one frame: `before | after | difference`, where the third
/// panel is a heat map (black = identical, red→yellow = larger change, scaled by `gain`).
pub fn compare_frame(
    a: &Engine,
    b: &Engine,
    frame: f32,
    scale: f32,
    gain: f32,
) -> Result<(u32, u32, Vec<u8>), String> {
    let opts = RasterOptions {
        scale,
        show_bones: false,
        transparent: false,
    };
    let (wa, ha, pa) = render_rgba(a, frame, &opts);
    let (wb, hb, pb) = render_rgba(b, frame, &opts);
    if (wa, ha) != (wb, hb) {
        return Err(format!(
            "frame sizes differ ({wa}×{ha} vs {wb}×{hb}); compare projects of the same size"
        ));
    }
    let (w, h) = (wa as usize, ha as usize);
    let mut out = vec![0u8; w * 3 * h * 4];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            let d = (0..3)
                .map(|c| (pa[i + c] as f32 - pb[i + c] as f32).abs())
                .fold((pa[i + 3] as f32 - pb[i + 3] as f32).abs(), f32::max)
                / 255.0;
            let t = (d * gain).clamp(0.0, 1.0);
            let heat = [(t * 2.0).min(1.0), (t * 2.0 - 1.0).max(0.0), 0.0];
            for (panel, px) in [
                (0, [pa[i], pa[i + 1], pa[i + 2], pa[i + 3]]),
                (1, [pb[i], pb[i + 1], pb[i + 2], pb[i + 3]]),
                (
                    2,
                    [(heat[0] * 255.0) as u8, (heat[1] * 255.0) as u8, 0, 255],
                ),
            ] {
                let o = (y * w * 3 + panel * w + x) * 4;
                out[o..o + 4].copy_from_slice(&px);
            }
        }
    }
    Ok((w as u32 * 3, h as u32, out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    #[test]
    fn stats_count_things() {
        let s = stats(&demo::project());
        assert!(s.bones >= 6 && s.layers > 0 && s.keys > 0 && s.keyed_channels > 0);
        assert!((s.seconds - s.frames as f32 / s.fps).abs() < 1e-4);
    }

    #[test]
    fn diff_of_identical_projects_is_empty() {
        let p = demo::project();
        assert!(diff(&p, &p.clone()).is_empty());
    }

    #[test]
    fn diff_reports_add_remove_change() {
        let a = demo::project();
        let mut b = a.clone();
        b.bones.pop();
        b.fps = 30.0;
        b.bones[0].length += 1.0;
        b.channels.remove(0);
        b.layers[0].opacity = 0.5;
        let d = diff(&a, &b).join("\n");
        assert!(d.contains("~ fps: 24 → 30"), "{d}");
        assert!(d.contains("- bone 'foot_ctrl' removed"), "{d}");
        assert!(d.contains("~ bone 'arm_up' changed (length)"), "{d}");
        assert!(d.contains("- channel"), "{d}");
        assert!(d.contains("~ layer") && d.contains("opacity"), "{d}");
    }

    #[test]
    fn golden_round_trip_and_regression_detection() {
        let mut p = demo::project();
        p.frames = 3;
        let e = Engine::new(p.clone()).unwrap();
        let g = golden_record(&e, 0.25);
        assert_eq!(g.frames.len(), 3);
        assert!(
            golden_check(&e, &g, 0.0).is_empty(),
            "same project matches exactly"
        );
        // serialises
        let g2: Golden = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        assert_eq!(g, g2);
        // change the artwork noticeably
        let mut q = p.clone();
        for l in &mut q.layers {
            l.visible = false;
        }
        q.background = Some(crate::style::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });
        let bad = golden_check(&Engine::new(q).unwrap(), &g, 0.01);
        assert_eq!(bad.len(), 3, "every frame differs");
        assert!(bad[0].distance > 0.05);
        assert!(bad[0].cells_differing > 0 && bad[0].max_diff > 0 && !bad[0].missing);
    }

    /// The default (tolerance 0) is exact: a 50-degree arm change and a background colour change are both caught,
    /// also when the thumbnails barely move; only an explicit tolerance lets small differences pass.
    #[test]
    fn golden_check_is_exact_at_tolerance_zero() {
        let mut p = demo::project();
        p.frames = 4;
        let g = golden_record(&Engine::new(p.clone()).unwrap(), 0.25);
        // background colour change (the demo's sky layer covers the whole background, so look at a project where it shows)
        let mut open = p.clone();
        for l in &mut open.layers {
            l.visible = false;
        }
        let g_open = golden_record(&Engine::new(open.clone()).unwrap(), 0.25);
        let mut q = open.clone();
        q.background = Some(crate::style::Rgba {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        });
        let bad = golden_check(&Engine::new(q).unwrap(), &g_open, 0.0);
        assert!(
            !bad.is_empty(),
            "a background colour change must fail the exact check"
        );
        // a big rotation of one bone
        let bone = p
            .bones
            .iter()
            .find(|b| b.parent.is_some())
            .map(|b| b.name.clone())
            .unwrap();
        let mut q = p.clone();
        for b in &mut q.bones {
            if b.name == bone {
                b.rest_angle += 50.0;
            }
        }
        let bad = golden_check(&Engine::new(q.clone()).unwrap(), &g, 0.0);
        assert!(
            !bad.is_empty(),
            "a 50 degree change must fail the exact check"
        );
        // a hash mismatch that the thumbnail cannot see still fails at 0 and passes under a tolerance
        let mut tweaked = g.clone();
        tweaked.frames[0].hash = "0000000000000000".into();
        let bad = golden_check(&Engine::new(p.clone()).unwrap(), &tweaked, 0.0);
        assert_eq!(bad.len(), 1);
        assert_eq!(
            (bad[0].frame, bad[0].cells_differing, bad[0].max_diff),
            (0, 0, 0)
        );
        assert!(golden_check(&Engine::new(p).unwrap(), &tweaked, 0.01).is_empty());
    }

    #[test]
    fn compare_sheet_has_three_panels_and_flags_changes() {
        let mut p = demo::project();
        p.frames = 2;
        let a = Engine::new(p.clone()).unwrap();
        let (w, h, same) = compare_frame(&a, &a, 0.0, 0.2, 4.0).unwrap();
        let panel = (w / 3) as usize;
        let diff_px = |img: &[u8]| {
            img.chunks_exact(4 * panel * 3)
                .flat_map(|row| {
                    row[panel * 2 * 4..]
                        .chunks_exact(4)
                        .map(|c| c[0] as u32 + c[1] as u32)
                        .collect::<Vec<_>>()
                })
                .sum::<u32>()
        };
        assert_eq!(
            diff_px(&same),
            0,
            "identical projects ⇒ black difference panel"
        );
        assert!(h > 0);
        let mut q = p.clone();
        for l in &mut q.layers {
            l.visible = false;
        }
        let (_, _, changed) = compare_frame(&a, &Engine::new(q).unwrap(), 0.0, 0.2, 4.0).unwrap();
        assert!(diff_px(&changed) > 1000);
        let mut big = p.clone();
        big.width *= 2.0;
        assert!(compare_frame(&a, &Engine::new(big).unwrap(), 0.0, 0.2, 4.0).is_err());
    }

    #[test]
    fn fnv_is_stable() {
        assert_eq!(fnv1a(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a(b"a"), 0xaf63dc4c8601ec8c);
    }
}
