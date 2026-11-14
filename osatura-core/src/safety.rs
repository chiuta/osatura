// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Output safety checks and a reduced-motion variant.
//!
//! Clean-room design note: the flash test follows the *public* WCAG 2.3.1 / "Three Flashes"
//! definitions (relative luminance, 10 % luminance step, darker state below 0.8, more than
//! three flashes per second over more than a quarter of a 10° field, approximated by a sliding
//! window of one ninth of the frame) — written from the published text, not from any
//! commercial analyser. It is an *engineering aid*, not a certification: broadcast or
//! regulated deliveries need an accredited tool.

use crate::engine::Engine;
use crate::math::Vec2;
use crate::project::{Project, Prop, Source};
use crate::raster::{render_rgba, RasterOptions};

#[derive(Clone, Debug, PartialEq)]
pub struct FlashFinding {
    /// Largest number of alternating transitions inside any one-second window (worst region).
    pub worst_transitions: usize,
    /// Frame at which the worst window ends.
    pub worst_frame: usize,
    /// More than 3 flashes (≥ 7 transitions) in a second.
    pub fails: bool,
    /// 5–6 transitions: close to the limit.
    pub warns: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SafetyReport {
    pub general: FlashFinding,
    pub red: FlashFinding,
}

impl SafetyReport {
    pub fn passes(&self) -> bool {
        !self.general.fails && !self.red.fails
    }
}

fn lin(c: u8) -> f32 {
    let x = c as f32 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

/// Box-downsample straight RGBA to at most `max_w` columns; returns (w, h, linear r,g,b).
fn reduce(rgba: &[u8], w: usize, h: usize, max_w: usize) -> (usize, usize, Vec<[f32; 3]>) {
    let k = w.div_ceil(max_w).max(1);
    let (nw, nh) = (w.div_ceil(k), h.div_ceil(k));
    let mut out = vec![[0.0f32; 3]; nw * nh];
    let mut cnt = vec![0.0f32; nw * nh];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            let a = rgba[i + 3] as f32 / 255.0;
            // composite over white so transparent areas do not read as black flashes
            let c = [lin(rgba[i]) * a + (1.0 - a), lin(rgba[i + 1]) * a + (1.0 - a), lin(rgba[i + 2]) * a + (1.0 - a)];
            let o = (y / k) * nw + x / k;
            for ch in 0..3 {
                out[o][ch] += c[ch];
            }
            cnt[o] += 1.0;
        }
    }
    for (o, c) in out.iter_mut().zip(&cnt) {
        for ch in o.iter_mut() {
            *ch /= c.max(1.0);
        }
    }
    (nw, nh, out)
}

fn luminance(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

fn saturated_red(c: [f32; 3]) -> bool {
    let s = c[0] + c[1] + c[2];
    s > 1e-6 && c[0] / s >= 0.8 && c[0] > 0.1
}

struct Integral {
    w: usize,
    data: Vec<u32>,
}

impl Integral {
    fn new(w: usize, h: usize, f: impl Fn(usize, usize) -> bool) -> Self {
        let mut data = vec![0u32; (w + 1) * (h + 1)];
        for y in 0..h {
            let mut row = 0;
            for x in 0..w {
                row += f(x, y) as u32;
                data[(y + 1) * (w + 1) + x + 1] = data[y * (w + 1) + x + 1] + row;
            }
        }
        Integral { w, data }
    }
    fn sum(&self, x0: usize, y0: usize, x1: usize, y1: usize) -> u32 {
        let s = self.w + 1;
        self.data[y1 * s + x1] + self.data[y0 * s + x0] - self.data[y0 * s + x1] - self.data[y1 * s + x0]
    }
}

/// Analyse straight-RGBA frames (all `w × h`) played at `fps`.
pub fn analyze_frames(frames: &[Vec<u8>], w: usize, h: usize, fps: f32) -> SafetyReport {
    let reduced: Vec<_> = frames.iter().map(|f| reduce(f, w, h, 96)).collect();
    let empty = FlashFinding { worst_transitions: 0, worst_frame: 0, fails: false, warns: false };
    if reduced.len() < 2 {
        return SafetyReport { general: empty.clone(), red: empty };
    }
    let (rw, rh) = (reduced[0].0, reduced[0].1);
    // sliding windows of 1/3 × 1/3 of the frame (≈ one 10° field at the reference geometry)
    let (ww, wh) = ((rw / 3).max(1), (rh / 3).max(1));
    let (sx, sy) = ((ww / 2).max(1), (wh / 2).max(1));
    let mut origins = Vec::new();
    let mut y = 0;
    while y + wh <= rh {
        let mut x = 0;
        while x + ww <= rw {
            origins.push((x, y));
            x += sx;
        }
        y += sy;
    }
    if origins.is_empty() {
        origins.push((0, 0));
    }
    let area = (ww * wh) as f32;
    let mut gen_ev: Vec<Vec<(usize, i8)>> = vec![Vec::new(); origins.len()];
    let mut red_ev: Vec<Vec<(usize, i8)>> = vec![Vec::new(); origins.len()];
    for t in 1..reduced.len() {
        let (a, b) = (&reduced[t - 1].2, &reduced[t].2);
        let inc = Integral::new(rw, rh, |x, y| {
            let (l0, l1) = (luminance(a[y * rw + x]), luminance(b[y * rw + x]));
            l1 - l0 >= 0.1 * l0.max(l1).max(1e-6) && l0 < 0.8
        });
        let dec = Integral::new(rw, rh, |x, y| {
            let (l0, l1) = (luminance(a[y * rw + x]), luminance(b[y * rw + x]));
            l0 - l1 >= 0.1 * l0.max(l1).max(1e-6) && l1 < 0.8
        });
        let rin = Integral::new(rw, rh, |x, y| !saturated_red(a[y * rw + x]) && saturated_red(b[y * rw + x]));
        let rout = Integral::new(rw, rh, |x, y| saturated_red(a[y * rw + x]) && !saturated_red(b[y * rw + x]));
        for (k, &(x, y)) in origins.iter().enumerate() {
            let q = |i: &Integral| i.sum(x, y, x + ww, y + wh) as f32 / area;
            let (pi, pd) = (q(&inc), q(&dec));
            if pi > 0.25 || pd > 0.25 {
                gen_ev[k].push((t, if pi >= pd { 1 } else { -1 }));
            }
            let (ri, ro) = (q(&rin), q(&rout));
            if ri > 0.25 || ro > 0.25 {
                red_ev[k].push((t, if ri >= ro { 1 } else { -1 }));
            }
        }
    }
    let win = fps.ceil().max(1.0) as usize;
    let judge = |ev: &[Vec<(usize, i8)>]| -> FlashFinding {
        let (mut worst, mut at) = (0usize, 0usize);
        for list in ev {
            for (idx, &(t, _)) in list.iter().enumerate() {
                // longest alternating chain ending here, within one second
                let mut n = 1;
                let mut j = idx;
                while j > 0 && t - list[j - 1].0 < win && list[j - 1].1 != list[j].1 {
                    n += 1;
                    j -= 1;
                }
                if n > worst {
                    worst = n;
                    at = t;
                }
            }
        }
        FlashFinding { worst_transitions: worst, worst_frame: at, fails: worst >= 7, warns: (5..7).contains(&worst) }
    };
    SafetyReport { general: judge(&gen_ev), red: judge(&red_ev) }
}

/// Render the whole timeline and analyse it.
pub fn analyze_engine(engine: &Engine, scale: f32) -> SafetyReport {
    let opts = RasterOptions { scale, show_bones: false, transparent: false };
    let mut frames = Vec::new();
    let (mut w, mut h) = (0, 0);
    for f in 0..engine.project.frames {
        let (fw, fh, d) = render_rgba(engine, f as f32, &opts);
        (w, h) = (fw as usize, fh as usize);
        frames.push(d);
    }
    analyze_frames(&frames, w, h, engine.project.fps)
}

/// Camera-motion summary per second (stage units are fractions of the stage width).
#[derive(Clone, Debug, PartialEq)]
pub struct MotionReport {
    pub max_pan_per_sec: f32,
    pub max_rotation_deg_per_sec: f32,
    pub max_zoom_ratio_per_sec: f32,
    /// Heuristic limits: pan > 0.6 widths/s, rotation > 45°/s or zoom > ×2/s.
    pub warnings: Vec<String>,
}

pub fn motion_report(engine: &Engine) -> MotionReport {
    let p = &engine.project;
    let centre = Vec2::new(p.width * 0.5, p.height * 0.5);
    let mut prev: Option<(Vec2, f32, f32)> = None;
    let (mut pan, mut rot, mut zoom) = (0.0f32, 0.0f32, 1.0f32);
    for f in 0..p.frames {
        let c = engine.evaluate(f as f32).camera;
        let cur = (c.apply(centre), c.rotation().to_degrees(), c.x_scale());
        if let Some(pr) = prev {
            pan = pan.max(cur.0.distance(pr.0) / p.width * p.fps);
            let mut dr = (cur.1 - pr.1).abs();
            if dr > 180.0 {
                dr = 360.0 - dr;
            }
            rot = rot.max(dr * p.fps);
            let r = (cur.2 / pr.2.max(1e-6)).max(pr.2 / cur.2.max(1e-6));
            zoom = zoom.max(r.powf(p.fps));
        }
        prev = Some(cur);
    }
    let mut warnings = Vec::new();
    if pan > 0.6 {
        warnings.push(format!("camera pans {pan:.2} stage widths per second"));
    }
    if rot > 45.0 {
        warnings.push(format!("camera rotates {rot:.0}° per second"));
    }
    if zoom > 2.0 {
        warnings.push(format!("camera zooms ×{zoom:.1} per second"));
    }
    MotionReport { max_pan_per_sec: pan, max_rotation_deg_per_sec: rot, max_zoom_ratio_per_sec: zoom, warnings }
}

/// A calmer copy of a project: keyed camera channels are scaled toward their first value by
/// `factor` (0 = locked camera, 1 = unchanged) and layer wobble ("boil") amplitude is scaled.
pub fn reduced_motion_twin(project: &Project, factor: f32) -> Project {
    let f = factor.clamp(0.0, 1.0);
    let mut p = project.clone();
    for ch in &mut p.channels {
        let cam = matches!(ch.prop, Prop::CameraX | Prop::CameraY | Prop::CameraRotation | Prop::CameraZoom);
        if cam || ch.prop == Prop::LayerWobble {
            if let Source::Keys { track } = &mut ch.source {
                let anchor = track.keys.first().map(|k| k.value).unwrap_or(0.0);
                for k in &mut track.keys {
                    k.value = anchor + (k.value - anchor) * f;
                }
            }
        }
    }
    fn walk(ls: &mut [crate::project::Layer], f: f32) {
        for l in ls {
            if let Some(w) = &mut l.wobble {
                w.amp *= f;
            }
            if let Some(c) = l.children_mut() {
                walk(c, f);
            }
        }
    }
    walk(&mut p.layers, f);
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::{Ease, Track};
    use crate::project::{Channel, Layer};
    use crate::path::VPath;

    fn solid(w: usize, h: usize, v: u8) -> Vec<u8> {
        [v, v, v, 255].repeat(w * h)
    }

    fn strobe(n: usize, period: usize) -> Vec<Vec<u8>> {
        (0..n).map(|t| solid(48, 36, if (t / period).is_multiple_of(2) { 0 } else { 255 })).collect()
    }

    #[test]
    fn fast_full_frame_strobe_fails() {
        // 24 fps, black/white every frame ⇒ 23 transitions per second
        let r = analyze_frames(&strobe(48, 1), 48, 36, 24.0);
        assert!(r.general.fails && !r.passes(), "{r:?}");
        assert!(r.general.worst_transitions >= 7);
    }

    #[test]
    fn slow_changes_pass() {
        // flips every 12 frames at 24 fps ⇒ 2 transitions per second
        let r = analyze_frames(&strobe(96, 12), 48, 36, 24.0);
        assert!(r.passes() && !r.general.warns, "{r:?}");
    }

    #[test]
    fn small_flashing_area_is_ignored() {
        // only a 4×4 pixel patch strobes — far below a quarter of the field
        let frames: Vec<Vec<u8>> = (0..48)
            .map(|t| {
                let mut f = solid(48, 36, 40);
                for y in 0..4 {
                    for x in 0..4 {
                        let i = (y * 48 + x) * 4;
                        let v = if t % 2 == 0 { 0 } else { 255 };
                        f[i] = v;
                        f[i + 1] = v;
                        f[i + 2] = v;
                    }
                }
                f
            })
            .collect();
        assert!(analyze_frames(&frames, 48, 36, 24.0).passes());
    }

    #[test]
    fn red_strobe_is_flagged_as_red_flash() {
        let frames: Vec<Vec<u8>> = (0..48).map(|t| if t % 2 == 0 { [255, 0, 0, 255].repeat(48 * 36) } else { [0, 120, 120, 255].repeat(48 * 36) }).collect();
        let r = analyze_frames(&frames, 48, 36, 24.0);
        assert!(r.red.fails, "{r:?}");
    }

    #[test]
    fn low_contrast_flicker_below_ten_percent_passes() {
        let frames: Vec<Vec<u8>> = (0..48).map(|t| solid(48, 36, if t % 2 == 0 { 100 } else { 104 })).collect();
        assert!(analyze_frames(&frames, 48, 36, 24.0).passes());
    }

    fn cam_project(shake: f32) -> Project {
        let mut p = Project::new("c", 200.0, 100.0, 24.0, 25);
        p.layers = vec![Layer::vector("box", vec![VPath::rect("r", 50.0, 20.0, 40.0, 40.0).with_fill("#336699")])];
        p.channels = vec![
            Channel::keys("", Prop::CameraX, Track::with_keys(&[(0.0, 100.0, Ease::Linear), (1.0, 100.0 + shake, Ease::Linear), (2.0, 100.0, Ease::Linear)]).looped(crate::anim::Extrapolate::Loop)),
        ];
        p
    }

    #[test]
    fn motion_report_and_twin() {
        let eng = Engine::new(cam_project(60.0)).unwrap();
        let m = motion_report(&eng);
        assert!(m.max_pan_per_sec > 0.6 && !m.warnings.is_empty(), "{m:?}");
        let calm = Engine::new(reduced_motion_twin(&cam_project(60.0), 0.05)).unwrap();
        let mc = motion_report(&calm);
        assert!(mc.max_pan_per_sec < m.max_pan_per_sec * 0.2, "{} vs {}", mc.max_pan_per_sec, m.max_pan_per_sec);
        assert!(mc.warnings.is_empty(), "{mc:?}");
        let locked = Engine::new(reduced_motion_twin(&cam_project(60.0), 0.0)).unwrap();
        assert!(motion_report(&locked).max_pan_per_sec < 1e-4);
    }

    #[test]
    fn engine_level_check_runs() {
        let mut p = Project::new("f", 60.0, 40.0, 24.0, 24);
        p.background = Some(crate::style::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
        p.layers = vec![Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 60.0, 40.0).with_fill("#ffffff")])];
        p.channels = vec![Channel::keys("v", Prop::LayerOpacity, Track::with_keys(&(0..24).map(|i| (i as f32, (i % 2) as f32, Ease::Step)).collect::<Vec<_>>()))];
        let r = analyze_engine(&Engine::new(p).unwrap(), 1.0);
        assert!(!r.passes(), "{r:?}");
    }
}
