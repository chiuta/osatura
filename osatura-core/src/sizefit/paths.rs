// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Path simplification: greedy anchor removal with a least-squares Bezier refit (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Path simplification: greedy anchor removal with a least-squares Bezier refit

/// A path vertex with *relative* tangents (the Lottie convention).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PNode {
    pub p: [f64; 2],
    pub i: [f64; 2],
    pub o: [f64; 2],
}

pub(super) fn add2(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}
pub(super) fn sub2(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
pub(super) fn dot2(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
pub(super) fn len2(a: [f64; 2]) -> f64 {
    dot2(a, a).sqrt()
}

pub(super) fn cubic_at(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        b0 * c[0][0] + b1 * c[1][0] + b2 * c[2][0] + b3 * c[3][0],
        b0 * c[0][1] + b1 * c[1][1] + b2 * c[2][1] + b3 * c[3][1],
    ]
}

pub(super) fn seg_ctrl(a: &PNode, b: &PNode) -> [[f64; 2]; 4] {
    [a.p, add2(a.p, a.o), add2(b.p, b.i), b.p]
}

pub(super) fn dist_pt_seg(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = sub2(b, a);
    let l2 = dot2(ab, ab);
    let t = if l2 < 1e-18 { 0.0 } else { (dot2(sub2(p, a), ab) / l2).clamp(0.0, 1.0) };
    len2(sub2(p, [a[0] + ab[0] * t, a[1] + ab[1] * t]))
}

pub(super) fn dist_pt_poly(p: [f64; 2], poly: &[[f64; 2]]) -> f64 {
    poly.windows(2).map(|w| dist_pt_seg(p, w[0], w[1])).fold(f64::INFINITY, f64::min)
}

pub(super) fn unit_or(v: [f64; 2], fallbacks: &[[f64; 2]]) -> [f64; 2] {
    for c in std::iter::once(&v).chain(fallbacks) {
        let l = len2(*c);
        if l > 1e-9 {
            return [c[0] / l, c[1] / l];
        }
    }
    [0.0, 0.0]
}

pub(super) const FIT_K: usize = 8;

/// Replace `a - k - b` by one cubic from `a` to `b`: returns the new outgoing tangent of `a`, the new
/// incoming tangent of `b` (both relative) and the symmetric sampled Hausdorff distance between the
/// original two segments and the replacement (stage units).
pub(super) fn merge_fit(a: &PNode, k: &PNode, b: &PNode) -> ([f64; 2], [f64; 2], f64) {
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
