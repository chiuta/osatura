// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `gltf` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// mask baking: exact clipping of triangles against a triangulated mask shape
// ---------------------------------------------------------------------------------------------

pub(super) const CLIP_EPS: f32 = 1e-6;

pub(super) fn poly_area(p: &[Vec2]) -> f32 {
    (0..p.len()).map(|i| cr(p[i], p[(i + 1) % p.len()])).sum::<f32>() * 0.5
}

/// Sutherland-Hodgman against one half plane: keep the left of a->b (or the right).
pub(super) fn clip_half(poly: &[Vec2], a: Vec2, b: Vec2, keep_left: bool) -> Vec<Vec2> {
    let s = |p: Vec2| if keep_left { cr(b - a, p - a) } else { -cr(b - a, p - a) };
    let mut out = Vec::with_capacity(poly.len() + 2);
    for i in 0..poly.len() {
        let (c, n) = (poly[i], poly[(i + 1) % poly.len()]);
        let (sc, sn) = (s(c), s(n));
        if sc >= 0.0 {
            out.push(c);
        }
        if (sc >= 0.0) != (sn >= 0.0) {
            let t = sc / (sc - sn);
            out.push(c.lerp(n, t));
        }
    }
    out
}

pub(super) fn clip_convex(poly: &[Vec2], tri: &[Vec2; 3]) -> Vec<Vec2> {
    let mut cur = poly.to_vec();
    for e in 0..3 {
        cur = clip_half(&cur, tri[e], tri[(e + 1) % 3], true);
        if cur.len() < 3 {
            return vec![];
        }
    }
    cur
}

/// `poly` minus the convex triangle `tri`, as disjoint convex pieces.
pub(super) fn subtract_convex(poly: &[Vec2], tri: &[Vec2; 3]) -> Vec<Vec<Vec2>> {
    let mut pieces = Vec::new();
    let mut cur = poly.to_vec();
    for e in 0..3 {
        let outside = clip_half(&cur, tri[e], tri[(e + 1) % 3], false);
        if outside.len() >= 3 && poly_area(&outside).abs() > CLIP_EPS {
            pieces.push(outside);
        }
        cur = clip_half(&cur, tri[e], tri[(e + 1) % 3], true);
        if cur.len() < 3 {
            break;
        }
    }
    pieces
}

pub(super) fn bbox(p: &[Vec2]) -> (Vec2, Vec2) {
    let mut lo = Vec2::new(f32::MAX, f32::MAX);
    let mut hi = Vec2::new(f32::MIN, f32::MIN);
    for q in p {
        lo = Vec2::new(lo.x.min(q.x), lo.y.min(q.y));
        hi = Vec2::new(hi.x.max(q.x), hi.y.max(q.y));
    }
    (lo, hi)
}

/// Clip a triangle mesh against masks (`(counter-clockwise triangles, inverted)`), exactly.
/// `None` if the result would exceed `limit` pieces.
pub(super) fn clip_mesh(pos: &[Vec2], tris: &[[u32; 3]], masks: &[(Vec<[Vec2; 3]>, bool)], limit: usize) -> Option<(Vec<Vec2>, Vec<[u32; 3]>)> {
    let mut cur: Vec<Vec<Vec2>> = tris
        .iter()
        .map(|t| {
            let mut v = vec![pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]];
            if poly_area(&v) < 0.0 {
                v.reverse();
            }
            v
        })
        .filter(|v| poly_area(v).abs() > 0.0)
        .collect();
    for (mt, invert) in masks {
        let mbox: Vec<(Vec2, Vec2)> = mt.iter().map(|t| bbox(t)).collect();
        let mut next: Vec<Vec<Vec2>> = Vec::new();
        for poly in cur {
            let mut rem = vec![poly];
            let mut inter: Vec<Vec<Vec2>> = Vec::new();
            for (m, mb) in mt.iter().zip(&mbox) {
                let mut nr = Vec::new();
                for r in rem {
                    let rb = bbox(&r);
                    if rb.1.x < mb.0.x || rb.0.x > mb.1.x || rb.1.y < mb.0.y || rb.0.y > mb.1.y {
                        nr.push(r);
                        continue;
                    }
                    let i = clip_convex(&r, m);
                    if i.len() >= 3 && poly_area(&i).abs() > CLIP_EPS {
                        inter.push(i);
                    }
                    nr.extend(subtract_convex(&r, m));
                }
                rem = nr;
                if rem.is_empty() {
                    break;
                }
            }
            next.extend(if *invert { rem } else { inter });
            if next.len() > limit {
                return None;
            }
        }
        cur = next;
    }
    let (mut p, mut t) = (Vec::new(), Vec::new());
    for poly in cur {
        let base = p.len() as u32;
        p.extend_from_slice(&poly);
        for k in 1..poly.len() - 1 {
            t.push([base, base + k as u32, base + k as u32 + 1]);
        }
    }
    Some((p, t))
}

// ---------------------------------------------------------------------------------------------
// gradients as textures
// ---------------------------------------------------------------------------------------------

pub(super) const GRAD_N: usize = 256;

#[derive(Clone, Copy)]
pub(super) enum GradMap {
    Linear { from: Vec2, to: Vec2 },
    Radial { center: Vec2, radius: f32 },
}

pub(super) fn grad_map(p: &Paint) -> Option<GradMap> {
    match p {
        Paint::Linear { from, to, stops } if stops.len() >= 2 && from.distance(*to) > 1e-6 => Some(GradMap::Linear { from: *from, to: *to }),
        Paint::Radial { center, radius, stops } if stops.len() >= 2 && *radius > 1e-3 => Some(GradMap::Radial { center: *center, radius: *radius }),
        _ => None,
    }
}

pub(super) fn grad_uv(m: &GradMap, p: Vec2) -> [f32; 2] {
    let n = GRAD_N as f32;
    match m {
        GradMap::Linear { from, to } => {
            let d = *to - *from;
            let u = (p - *from).dot(d) / d.dot(d);
            [(0.5 + u * (n - 1.0)) / n, 0.5]
        }
        GradMap::Radial { center, radius } => {
            let s = (p - *center) * (1.0 / radius);
            [(0.5 + (s.x + 1.0) * 0.5 * (n - 1.0)) / n, (0.5 + (s.y + 1.0) * 0.5 * (n - 1.0)) / n]
        }
    }
}

pub(super) fn color_at(stops: &[crate::style::Stop], t: f32) -> Rgba {
    let mut s: Vec<_> = stops.to_vec();
    s.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    if t <= s[0].offset {
        return s[0].color;
    }
    for w in s.windows(2) {
        if t <= w[1].offset {
            let span = w[1].offset - w[0].offset;
            let u = if span <= 1e-9 { 1.0 } else { (t - w[0].offset) / span };
            return w[0].color.lerp(w[1].color, u);
        }
    }
    s[s.len() - 1].color
}

pub(super) fn encode_rgba_png(w: u32, h: u32, rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header().expect("png header");
        wr.write_image_data(rgba).expect("png data");
    }
    out
}

/// Gradient lookup texture. Texel centres hold the colour at `k/(N-1)` (linear) or at the
/// matching point of the unit square (radial), so bilinear sampling reproduces the paint.
pub(super) fn gradient_png(p: &Paint) -> Vec<u8> {
    let n = GRAD_N;
    match p {
        Paint::Linear { stops, .. } => {
            let mut px = Vec::with_capacity(n * 4);
            for k in 0..n {
                px.extend_from_slice(&color_at(stops, k as f32 / (n - 1) as f32).to_u8());
            }
            encode_rgba_png(n as u32, 1, &px)
        }
        Paint::Radial { stops, .. } => {
            let mut px = Vec::with_capacity(n * n * 4);
            for j in 0..n {
                for i in 0..n {
                    let (sx, sy) = (-1.0 + 2.0 * i as f32 / (n - 1) as f32, -1.0 + 2.0 * j as f32 / (n - 1) as f32);
                    px.extend_from_slice(&color_at(stops, sx.hypot(sy)).to_u8());
                }
            }
            encode_rgba_png(n as u32, n as u32, &px)
        }
        Paint::Solid(c) => encode_rgba_png(1, 1, &c.to_u8()),
    }
}

pub(super) fn stops_translucent(p: &Paint) -> bool {
    match p {
        Paint::Solid(c) => c.a < 0.999,
        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => stops.iter().any(|s| s.color.a < 0.999),
    }
}

// ---------------------------------------------------------------------------------------------
// morph targets: fixed subdivision so topology never changes
// ---------------------------------------------------------------------------------------------

pub(super) fn seg_pts(p: &VPath, i: usize) -> [Vec2; 4] {
    let a = &p.anchors[i];
    let b = &p.anchors[(i + 1) % p.anchors.len()];
    [a.pos, a.hout(), b.hin(), b.pos]
}

pub(super) fn seg_count_for(p: &VPath, i: usize, tol: f32) -> usize {
    let [p0, c1, c2, p1] = seg_pts(p, i);
    let ctrl = p0.distance(c1) + c1.distance(c2) + c2.distance(p1);
    ((ctrl / tol.max(1e-3)).sqrt() * 0.6).ceil().clamp(1.0, 48.0) as usize
}

pub(super) fn flatten_fixed(p: &VPath, ns: &[usize]) -> Vec<Vec2> {
    let mut out = Vec::new();
    let Some(first) = p.anchors.first() else { return out };
    out.push(first.pos);
    for (i, &n) in ns.iter().enumerate().take(p.segment_count()) {
        let [p0, c1, c2, p1] = seg_pts(p, i);
        for k in 1..=n {
            let t = k as f32 / n as f32;
            let m = 1.0 - t;
            out.push(p0 * (m * m * m) + c1 * (3.0 * m * m * t) + c2 * (3.0 * m * t * t) + p1 * (t * t * t));
        }
    }
    if p.closed && out.len() > 1 {
        out.pop();
    }
    out
}

pub(super) struct MorphData {
    pub(super) names: Vec<String>,
    pub(super) deltas: Vec<Vec<Vec2>>,
    pub(super) w0: Vec<f32>,
    /// frame-major: `frames * names.len()` weights
    pub(super) frames: Vec<f32>,
}

pub(super) type MorphCache = Option<Vec<(Frame, Vec<Vec<Vec<f32>>>)>>;

/// Try to express `src`'s morphs as glTF targets. Ok = (base points, triangles, data).
pub(super) type Tri = [u32; 3];

pub(super) fn try_morph(engine: &Engine, cache: &mut MorphCache, nid: usize, pi: usize, src: &VPath) -> Result<(Vec<Vec2>, Vec<Tri>, MorphData), String> {
    let nf = engine.project.frames as usize;
    if nf > 1500 || src.anchors.len() < 3 {
        return Err("too many frames (or too few anchors) to verify morph targets".into());
    }
    let morph_paths: Vec<VPath> = src.morphs.iter().map(|m| VPath { anchors: m.anchors.clone(), morphs: vec![], ..src.clone() }).collect();
    let ns: Vec<usize> = (0..src.segment_count())
        .map(|i| {
            let mut n = seg_count_for(src, i, 0.5);
            for mp in &morph_paths {
                n = n.max(seg_count_for(mp, i, 0.5));
            }
            n
        })
        .collect();
    let base = flatten_fixed(src, &ns);
    let deltas: Vec<Vec<Vec2>> = morph_paths.iter().map(|mp| flatten_fixed(mp, &ns).iter().zip(&base).map(|(a, b)| *a - *b).collect()).collect();
    let cache = cache.get_or_insert_with(|| (0..nf).map(|f| (engine.evaluate(f as f32), engine.morph_weights(f as f32))).collect());
    let nt = src.morphs.len();
    let mut frames = Vec::with_capacity(nf * nt);
    let mut tris: Vec<[u32; 3]> = Vec::new();
    for (f, (fr, mw)) in cache.iter().enumerate() {
        let w = mw.get(nid).and_then(|x| x.get(pi)).cloned().unwrap_or_default();
        if w.len() != nt {
            return Err("morph weights unavailable".into());
        }
        let actual = flatten_fixed(&fr.nodes[nid].paths[pi], &ns);
        if actual.len() != base.len() {
            return Err(format!("topology differs at frame {f}"));
        }
        let recon: Vec<Vec2> = (0..base.len())
            .map(|v| {
                let mut p = base[v];
                for (k, d) in deltas.iter().enumerate() {
                    p += d[v] * w[k];
                }
                p
            })
            .collect();
        let err = recon.iter().zip(&actual).map(|(a, b)| a.distance(*b)).fold(0.0, f32::max);
        if err > 0.05 {
            return Err(format!("geometry at frame {f} is not base + weighted deltas (error {err:.3}); the layer is also deformed by skinning, a deformer, wobble or point offsets"));
        }
        if f == 0 {
            tris = triangulate(&actual).into_iter().map(|t| [t[0] as u32, t[1] as u32, t[2] as u32]).collect();
            if tris.is_empty() {
                return Err("fill cannot be triangulated".into());
            }
        }
        for t in &tris {
            let (a, b, c) = (recon[t[0] as usize], recon[t[1] as usize], recon[t[2] as usize]);
            if cr(b - a, c - a) < -0.1 {
                return Err(format!("the frame-0 triangulation folds over at frame {f}"));
            }
        }
        frames.extend_from_slice(&w);
    }
    if frames.iter().all(|w| w.abs() < 1e-6) {
        return Err("morph weights are zero on every frame".into());
    }
    let w0 = frames[..nt].to_vec();
    Ok((base, tris, MorphData { names: src.morphs.iter().map(|m| m.name.clone()).collect(), deltas, w0, frames }))
}
