// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Verification: measure the actual deviation (part of `lod`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Verification: measure the actual deviation

/// Parameters of [`verify`].
#[derive(Clone, Debug, PartialEq)]
pub struct VerifyOptions {
    /// Output pixels per stage unit.
    pub scale: f64,
    /// Evenly spaced sample frames (key frames and half frames are always added).
    pub frames: usize,
    /// Also render both levels and compute pixel statistics on `pixel_frames` frames.
    pub pixels: bool,
    pub pixel_frames: usize,
}

impl VerifyOptions {
    pub fn new(scale: f64) -> VerifyOptions {
        VerifyOptions { scale, frames: 24, pixels: true, pixel_frames: 6 }
    }
}

/// Result of [`verify`]. All pixel numbers are at `scale`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VerifyReport {
    pub level: usize,
    pub declared_px: f64,
    /// Largest screen-space deviation of evaluated geometry (Hausdorff for paths, deformed mesh
    /// surface for images, culled layers charged with their diameter).
    pub measured_px: f64,
    pub measured_paths_px: f64,
    pub measured_mesh_px: f64,
    pub measured_cull_px: f64,
    pub worst_frame: f32,
    pub worst_layer: String,
    pub frames_checked: usize,
    /// `measured_px <= declared_px` (plus a numeric slack of 1e-6 px). `false` is a bug.
    pub holds: bool,
    /// declared / measured, `None` when nothing deviates.
    pub conservatism: Option<f64>,
    pub pixel_frames: usize,
    pub pixel_max_abs: u32,
    pub pixel_max_de2000: f64,
    /// Windowed (9x9) estimate of the mean edge displacement from the L1 image difference over the
    /// total variation of the reference; a lower bound of the local maximum, reported for orientation.
    pub pixel_shift_est_px: f64,
}

impl VerifyReport {
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

pub(super) fn verify_frames(orig: &Project, lod: &Project, n: usize) -> Vec<f32> {
    let total = orig.frames.max(1) as f32;
    let mut v: Vec<f32> = Vec::new();
    for p in [orig, lod] {
        for c in &p.channels {
            if let Source::Keys { track } = &c.source {
                v.extend(track.keys.iter().map(|k| k.frame).filter(|f| *f >= 0.0 && *f < total));
            }
        }
    }
    let n = n.max(1);
    for i in 0..n {
        let f = (i as f32 * total / n as f32).floor();
        v.push(f);
        v.push((f + 0.5).min(total - 1e-3));
    }
    v.push(total - 1.0);
    v.sort_by(|a, b| a.total_cmp(b));
    v.dedup();
    v
}

pub(super) fn screen_outline(path: &VPath, w: &A, scale: f64) -> Vec<P> {
    let mut anchors = path.anchors.clone();
    for a in &mut anchors {
        a.map_points(&mut |q| {
            let r = a_apply(w, p_of(q));
            Vec2::new((r[0] * scale) as f32, (r[1] * scale) as f32)
        });
    }
    outline_polyline(&anchors, path.closed, 0.02)
}

/// L0 vertex indices + weights, and the L1 vertex indices + weights at the same spot.
pub(super) type ProbePoint = ([usize; 3], [f64; 3], [usize; 3], [f64; 3]);

pub(super) struct MeshProbe {
    /// Sample points: (L0 vertex indices with weights) and where they lie in the L1 mesh.
    pub(super) pts: Vec<ProbePoint>,
}

pub(super) fn locate(rest: &[P], tris: &[[usize; 3]], p: P) -> Option<([usize; 3], [f64; 3])> {
    let mut best: Option<(f64, [usize; 3], [f64; 3])> = None;
    for t in tris {
        let tp = [rest[t[0]], rest[t[1]], rest[t[2]]];
        let l = bary(p, tp);
        let m = l.iter().cloned().fold(f64::MAX, f64::min);
        if best.as_ref().is_none_or(|b| m > b.0) {
            best = Some((m, *t, l));
        }
        if m >= -1e-9 {
            break;
        }
    }
    best.map(|b| (b.1, b.2))
}

/// Measure the deviation of `lod` from `orig` (the L0 project) by evaluating both on sampled frames.
pub fn verify(orig: &Project, lod: &Lod, o: &VerifyOptions) -> Result<VerifyReport, String> {
    if o.scale.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return Err("scale must be positive".into());
    }
    let e0 = Engine::new(orig.clone()).map_err(|e| e.join("; "))?;
    let e1 = Engine::new(lod.project.clone()).map_err(|e| e.join("; "))?;
    let frames = verify_frames(orig, &lod.project, o.frames);
    let (mut m_paths, mut m_mesh, mut m_cull) = (0.0f64, 0.0f64, 0.0f64);
    let mut worst = (0.0f64, 0.0f32, String::new());
    let mut probes: BTreeMap<String, MeshProbe> = BTreeMap::new();
    for &t in &frames {
        let f0 = e0.evaluate(t);
        let f1 = e1.evaluate(t);
        for nd in &f0.nodes {
            let w0 = a_of(&nd.world);
            let mut local_worst = 0.0f64;
            let Some(i1) = e1.node_index(&nd.name) else {
                // removed layer: charged with its screen diameter while it was drawn
                if nd.visible && nd.opacity > 0.0 {
                    let pts = node_points(nd, &w0);
                    if !pts.is_empty() {
                        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
                        for q in &pts {
                            x0 = x0.min(q[0]);
                            y0 = y0.min(q[1]);
                            x1 = x1.max(q[0]);
                            y1 = y1.max(q[1]);
                        }
                        let hw = nd.paths.iter().filter_map(|pp| pp.stroke.as_ref().map(|s| s.width as f64 * 0.5)).fold(0.0, f64::max) * lin_norm(&w0);
                        let d = ((x1 - x0) + 2.0 * hw).hypot((y1 - y0) + 2.0 * hw) * o.scale;
                        m_cull = m_cull.max(d);
                        if d > worst.0 {
                            worst = (d, t, nd.name.clone());
                        }
                    }
                }
                continue;
            };
            let n1 = &f1.nodes[i1];
            let w1 = a_of(&n1.world);
            if nd.paths.len() == n1.paths.len() {
                for (pa, pb) in nd.paths.iter().zip(&n1.paths) {
                    let (a, b) = (screen_outline(pa, &w0, o.scale), screen_outline(pb, &w1, o.scale));
                    let d = hausdorff_upper(&a, &b, 0.002);
                    local_worst = local_worst.max(d);
                    m_paths = m_paths.max(d);
                }
            }
            if let (Some(a), Some(b)) = (&nd.image, &n1.image) {
                // rectangle corners
                let (oa, sa) = (p_of(a.origin), p_of(a.size));
                let (ob, sb) = (p_of(b.origin), p_of(b.size));
                for c in [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
                    let pa = a_apply(&w0, [oa[0] + sa[0] * c[0], oa[1] + sa[1] * c[1]]);
                    let pb = a_apply(&w1, [ob[0] + sb[0] * c[0], ob[1] + sb[1] * c[1]]);
                    local_worst = local_worst.max(dist(pa, pb) * o.scale);
                }
                if let (Some(ma), Some(mb)) = (&a.mesh, &b.mesh) {
                    let probe = probes.entry(nd.name.clone()).or_insert_with(|| {
                        let rest_of = |p: &Project, img: &crate::engine::ImageDraw| -> Vec<P> {
                            match p.find_layer(&nd.name).map(|l| &l.kind) {
                                Some(LayerKind::Image { mesh: Some(m), origin, .. }) => m.rest_vertices(*origin, img.size).iter().map(|v| p_of(*v)).collect(),
                                _ => vec![],
                            }
                        };
                        let (ra, rb) = (rest_of(orig, a), rest_of(&lod.project, b));
                        let tb: Vec<[usize; 3]> = mb.tris.iter().map(|t| [t[0] as usize, t[1] as usize, t[2] as usize]).collect();
                        let mut pts = Vec::new();
                        if ra.len() == ma.positions.len() && rb.len() == mb.positions.len() {
                            for t in &ma.tris {
                                let t = [t[0] as usize, t[1] as usize, t[2] as usize];
                                let cs: [[f64; 3]; 7] = [
                                    [1.0, 0.0, 0.0],
                                    [0.0, 1.0, 0.0],
                                    [0.0, 0.0, 1.0],
                                    [0.5, 0.5, 0.0],
                                    [0.0, 0.5, 0.5],
                                    [0.5, 0.0, 0.5],
                                    [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0],
                                ];
                                for l in cs {
                                    let p = add(add(mul(ra[t[0]], l[0]), mul(ra[t[1]], l[1])), mul(ra[t[2]], l[2]));
                                    if let Some((tt, ll)) = locate(&rb, &tb, p) {
                                        pts.push((t, l, tt, ll));
                                    }
                                }
                            }
                        }
                        MeshProbe { pts }
                    });
                    for (ta, la, tb_, lb) in &probe.pts {
                        let ya = (0..3).fold([0.0, 0.0], |acc, j| add(acc, mul(p_of(ma.positions[ta[j]]), la[j])));
                        let yb = (0..3).fold([0.0, 0.0], |acc, j| add(acc, mul(p_of(mb.positions[tb_[j]]), lb[j])));
                        let d = dist(a_apply(&w0, ya), a_apply(&w1, yb)) * o.scale;
                        m_mesh = m_mesh.max(d);
                        local_worst = local_worst.max(d);
                    }
                }
            }
            if local_worst > worst.0 {
                worst = (local_worst, t, nd.name.clone());
            }
        }
    }
    let measured = m_paths.max(m_mesh).max(m_cull).max(worst.0);
    let (mut pmax, mut pde, mut pshift, mut pframes) = (0u32, 0.0f64, 0.0f64, 0usize);
    if o.pixels {
        let ro = crate::raster::RasterOptions { scale: o.scale as f32, show_bones: false, transparent: false };
        let mut pf: Vec<f32> = (0..o.pixel_frames.max(1)).map(|i| (i as f32 * orig.frames.max(1) as f32 / o.pixel_frames.max(1) as f32).floor()).collect();
        pf.push(worst.1);
        pf.sort_by(|a, b| a.total_cmp(b));
        pf.dedup();
        for t in pf {
            let (w, h, a) = crate::raster::render_rgba(&e0, t, &ro);
            let (_, _, b) = crate::raster::render_rgba(&e1, t, &ro);
            let (w, h) = (w as usize, h as usize);
            pframes += 1;
            for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
                let d = (0..4).map(|k| (pa[k] as i32 - pb[k] as i32).unsigned_abs()).max().unwrap_or(0);
                if d > 0 {
                    pmax = pmax.max(d);
                    pde = pde.max(crate::percep::pixel_de([pa[0], pa[1], pa[2], pa[3]], [pb[0], pb[1], pb[2], pb[3]], crate::percep::DeMetric::De2000));
                }
            }
            pshift = pshift.max(shift_estimate(&a, &b, w, h));
        }
    }
    Ok(VerifyReport {
        level: lod.record.level,
        declared_px: lod.record.declared_px,
        measured_px: measured,
        measured_paths_px: m_paths,
        measured_mesh_px: m_mesh,
        measured_cull_px: m_cull,
        worst_frame: worst.1,
        worst_layer: worst.2,
        frames_checked: frames.len(),
        holds: measured <= lod.record.declared_px + 1e-6,
        conservatism: if measured > 1e-9 { Some(lod.record.declared_px / measured) } else { None },
        pixel_frames: pframes,
        pixel_max_abs: pmax,
        pixel_max_de2000: pde,
        pixel_shift_est_px: pshift,
    })
}

/// Mean edge displacement estimate: windowed (9x9) L1 difference over the total variation of the reference.
pub(super) fn shift_estimate(a: &[u8], b: &[u8], w: usize, h: usize) -> f64 {
    if w < 3 || h < 3 {
        return 0.0;
    }
    let lum = |im: &[u8], i: usize| (0.299 * im[i * 4] as f64 + 0.587 * im[i * 4 + 1] as f64 + 0.114 * im[i * 4 + 2] as f64) / 255.0;
    let mut diff = vec![0.0f64; w * h];
    let mut tv = vec![0.0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            diff[i] = (lum(a, i) - lum(b, i)).abs();
            let dx = if x + 1 < w { lum(a, i + 1) - lum(a, i) } else { 0.0 };
            let dy = if y + 1 < h { lum(a, i + w) - lum(a, i) } else { 0.0 };
            tv[i] = dx.hypot(dy);
        }
    }
    let integral = |v: &[f64]| -> Vec<f64> {
        let mut s = vec![0.0; (w + 1) * (h + 1)];
        for y in 0..h {
            let mut row = 0.0;
            for x in 0..w {
                row += v[y * w + x];
                s[(y + 1) * (w + 1) + x + 1] = s[y * (w + 1) + x + 1] + row;
            }
        }
        s
    };
    let (sd, st) = (integral(&diff), integral(&tv));
    let rect = |s: &[f64], x0: usize, y0: usize, x1: usize, y1: usize| s[y1 * (w + 1) + x1] - s[y0 * (w + 1) + x1] - s[y1 * (w + 1) + x0] + s[y0 * (w + 1) + x0];
    let r = 4usize;
    let mut best = 0.0f64;
    for y in 0..h {
        for x in 0..w {
            let (x0, y0, x1, y1) = (x.saturating_sub(r), y.saturating_sub(r), (x + r + 1).min(w), (y + r + 1).min(h));
            let t = rect(&st, x0, y0, x1, y1);
            if t >= 1.0 {
                best = best.max(rect(&sd, x0, y0, x1, y1) / t);
            }
        }
    }
    best
}

/// Plain-text table of levels (and measurements, if given).
pub fn table(levels: &[Lod], verify: &[VerifyReport]) -> String {
    let mut s = String::new();
    s.push_str("level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  path  mesh  keys  cull  measured_px  ratio\n");
    for l in levels {
        let r = &l.record;
        let v = verify.iter().find(|v| v.level == r.level);
        s.push_str(&format!(
            "L{:<4} {:>7.3} {:>8} {:>7} {:>5} {:>7} {:>8} {:>12.4} {:>5.3} {:>5.3} {:>5.3} {:>5.3} {:>12} {:>6}\n",
            r.level,
            r.tolerance_px,
            r.stats.path_anchors,
            r.stats.mesh_vertices,
            r.stats.keys,
            r.stats.layers,
            r.stats.bytes,
            r.declared_px,
            r.path_px,
            r.mesh_px,
            r.keys_px,
            r.cull_px,
            v.map_or("-".to_string(), |v| format!("{:.4}", v.measured_px)),
            v.and_then(|v| v.conservatism).map_or("-".to_string(), |c| format!("{c:.1}x")),
        ));
    }
    s
}

/// JSON report of a build (and verification).
pub fn report_json(levels: &[Lod], verify: &[VerifyReport]) -> Value {
    json!({
        "levels": levels.iter().map(|l| serde_json::to_value(&l.record).unwrap_or(Value::Null)).collect::<Vec<_>>(),
        "verify": verify.iter().map(|v| v.to_json()).collect::<Vec<_>>(),
        "assumptions": ASSUMPTIONS,
        "limits": LIMITS,
    })
}
