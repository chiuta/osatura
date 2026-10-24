// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Path simplification with a certified Hausdorff bound (part of `lod`; see the module docs in `mod.rs`).
use super::*;

// Path simplification with a certified Hausdorff bound

pub(super) struct OrigSeg {
    pub(super) ctrl: Cubic,
    pub(super) poly: Vec<P>,
    pub(super) h: f64,
}

pub(super) fn unit_or(cands: &[P]) -> P {
    for c in cands {
        let l = norm(*c);
        if l > 1e-12 {
            return mul(*c, 1.0 / l);
        }
    }
    [0.0, 0.0]
}

/// Sampled points of a span of original segments (`K` per segment).
pub(super) fn span_samples(segs: &[&OrigSeg]) -> Vec<P> {
    const K: usize = 8;
    let mut v = Vec::with_capacity(segs.len() * K + 1);
    for s in segs {
        for j in 0..K {
            v.push(cubic_at(&s.ctrl, j as f64 / K as f64));
        }
    }
    v.push(segs.last().unwrap().ctrl[3]);
    v
}

/// Least-squares tangent lengths for end tangents `t1` (outgoing) and `t2` (pointing from the end
/// back along the curve), given parameters `u` of the samples.
pub(super) fn lsq_alpha(pts: &[P], u: &[f64], p0: P, p3: P, t1: P, t2: P) -> Option<(f64, f64)> {
    let chord = dist(p0, p3);
    let (mut c11, mut c12, mut c22, mut x1, mut x2) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (pt, &t) in pts.iter().zip(u) {
        let w = 1.0 - t;
        let (b0, b1, b2, b3) = (w * w * w, 3.0 * w * w * t, 3.0 * w * t * t, t * t * t);
        let a1 = mul(t1, b1);
        let a2 = mul(t2, b2);
        c11 += dot(a1, a1);
        c12 += dot(a1, a2);
        c22 += dot(a2, a2);
        let base = add(mul(p0, b0 + b1), mul(p3, b2 + b3));
        let r = sub(*pt, base);
        x1 += dot(a1, r);
        x2 += dot(a2, r);
    }
    let det = c11 * c22 - c12 * c12;
    if det.abs() <= 1e-12 * (c11 * c22).abs().max(1e-30) {
        return None;
    }
    let (s, e) = ((x1 * c22 - x2 * c12) / det, (c11 * x2 - c12 * x1) / det);
    if s.is_finite() && e.is_finite() && s > 1e-6 * chord && e > 1e-6 * chord && s < 8.0 * chord.max(1e-9) * 4.0 && e < 8.0 * chord.max(1e-9) * 4.0 {
        Some((s, e))
    } else {
        None
    }
}

pub(super) fn chord_params(pts: &[P]) -> Vec<f64> {
    let mut u = vec![0.0f64; pts.len()];
    for j in 1..pts.len() {
        u[j] = u[j - 1] + dist(pts[j], pts[j - 1]);
    }
    let total = u[pts.len() - 1];
    if total > 1e-300 {
        for x in &mut u {
            *x /= total;
        }
    }
    u
}

pub(super) fn mk_cubic(p0: P, p3: P, t1: P, t2: P, a1: f64, a2: f64) -> Cubic {
    [p0, [r32(p0[0] + t1[0] * a1), r32(p0[1] + t1[1] * a1)], [r32(p3[0] + t2[0] * a2), r32(p3[1] + t2[1] * a2)], p3]
}

/// Candidate replacement cubics for a span (always includes the straight line).
pub(super) fn candidates(segs: &[&OrigSeg]) -> Vec<Cubic> {
    let first = &segs[0].ctrl;
    let last = &segs[segs.len() - 1].ctrl;
    let (p0, p3) = (first[0], last[3]);
    let line: Cubic = [p0, p0, p3, p3];
    let mut out = vec![line];
    let chord = dist(p0, p3);
    if chord < 1e-12 {
        return out;
    }
    let t1 = unit_or(&[sub(first[1], first[0]), sub(first[2], first[0]), sub(first[3], first[0])]);
    let t2 = unit_or(&[sub(last[2], last[3]), sub(last[1], last[3]), sub(last[0], last[3])]);
    if norm(t1) == 0.0 || norm(t2) == 0.0 {
        return out;
    }
    let pts = span_samples(segs);
    let mut u = chord_params(&pts);
    let mut alphas = (chord / 3.0, chord / 3.0);
    if let Some(a) = lsq_alpha(&pts, &u, p0, p3, t1, t2) {
        alphas = a;
    }
    out.push(mk_cubic(p0, p3, t1, t2, chord / 3.0, chord / 3.0));
    // Newton re-parametrisation (Schneider)
    for _ in 0..3 {
        let c = mk_cubic(p0, p3, t1, t2, alphas.0, alphas.1);
        for (j, pt) in pts.iter().enumerate() {
            let d = sub(cubic_at(&c, u[j]), *pt);
            let d1 = cubic_d1(&c, u[j]);
            let d2 = cubic_d2(&c, u[j]);
            let den = dot(d1, d1) + dot(d, d2);
            if den.abs() > 1e-12 {
                u[j] = (u[j] - dot(d, d1) / den).clamp(0.0, 1.0);
            }
        }
        u[0] = 0.0;
        let last_i = u.len() - 1;
        u[last_i] = 1.0;
        match lsq_alpha(&pts, &u, p0, p3, t1, t2) {
            Some(a) => alphas = a,
            None => break,
        }
    }
    out.push(mk_cubic(p0, p3, t1, t2, alphas.0, alphas.1));
    out.push(mk_cubic(p0, p3, t1, t2, alphas.0 * 0.8, alphas.1 * 0.8));
    out.push(mk_cubic(p0, p3, t1, t2, alphas.0 * 1.25, alphas.1 * 1.25));
    out
}

pub(super) fn sampled_error(cand: &Cubic, pts: &[P]) -> f64 {
    let q: Vec<P> = (0..=24).map(|j| cubic_at(cand, j as f64 / 24.0)).collect();
    let e1 = pts.iter().map(|&p| dist_to_poly(p, &q)).fold(0.0, f64::max);
    let e2 = q.iter().map(|&p| dist_to_poly(p, pts)).fold(0.0, f64::max);
    e1.max(e2)
}

/// Result of [`simplify_path_certified`].
#[derive(Clone, Debug)]
pub struct SimplifiedPath {
    pub anchors: Vec<Anchor>,
    /// Certified upper bound (path units) of the Hausdorff distance to the original outline.
    pub bound: f64,
    pub removed: usize,
}

/// Certified cost (upper bound of the Hausdorff distance) of replacing the span of original
/// segments by the best candidate; `None` if no candidate stays within `limit` (or the Frechet
/// guard fails). Returns the cost and the cubic.
pub(super) fn certify_span(segs: &[&OrigSeg], limit: f64, flat: f64, gap: f64) -> Option<(f64, Cubic)> {
    let pts = span_samples(segs);
    let mut cands = candidates(segs);
    // rank by cheap sampled error (stable), certify the best two
    let mut ranked: Vec<(f64, usize)> = cands.iter().enumerate().map(|(i, c)| (sampled_error(c, &pts), i)).collect();
    ranked.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut orig_poly: Vec<P> = Vec::new();
    let mut h_o = 0.0f64;
    for (i, s) in segs.iter().enumerate() {
        if i == 0 {
            orig_poly.extend_from_slice(&s.poly);
        } else {
            orig_poly.extend_from_slice(&s.poly[1..]);
        }
        h_o = h_o.max(s.h);
    }
    let mut best: Option<(f64, Cubic)> = None;
    for &(se, i) in ranked.iter().take(2) {
        if se > limit * 1.5 + 4.0 * flat {
            continue;
        }
        let c = cands[i];
        let (q, h_c) = flatten_poly(&c, flat);
        let e = hausdorff_upper(&orig_poly, &q, gap) + h_o + h_c;
        if e <= limit && best.as_ref().is_none_or(|b| e < b.0) {
            // guard: monotone correspondence (catches winding / backtracking differences)
            let ra = resample(&orig_poly, 64);
            let rb = resample(&q, 64);
            let step = (path_len(&orig_poly).max(path_len(&q)) / 63.0).max(0.0);
            if discrete_frechet(&ra, &rb) <= limit + step {
                best = Some((e, c));
            }
        }
    }
    cands.clear();
    best
}

pub(super) fn path_len(p: &[P]) -> f64 {
    p.windows(2).map(|w| dist(w[0], w[1])).sum()
}

/// Remove anchors of one path while the certified Hausdorff distance to the original outline stays
/// within `eps` (path units). The result is deterministic; ends of open paths and the last three
/// anchors of a closed path are kept. Weights, morphs and strokes are the caller's business.
pub fn simplify_path_certified(anchors: &[Anchor], closed: bool, eps: f64) -> SimplifiedPath {
    let n = anchors.len();
    let unchanged = || SimplifiedPath { anchors: anchors.to_vec(), bound: 0.0, removed: 0 };
    let min_keep = if closed { 3 } else { 2 };
    if (eps.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !eps.is_finite() || n <= min_keep {
        return unchanged();
    }
    let flat = (eps * 0.04).max(1e-9);
    let gap = (eps * 0.02).max(1e-9);
    let pos: Vec<P> = anchors.iter().map(|a| p_of(a.pos)).collect();
    let hin: Vec<P> = anchors.iter().map(|a| p_of(a.hin())).collect();
    let hout: Vec<P> = anchors.iter().map(|a| p_of(a.hout())).collect();
    let ns = if closed { n } else { n - 1 };
    let segs: Vec<OrigSeg> = (0..ns)
        .map(|i| {
            let j = (i + 1) % n;
            let ctrl: Cubic = [pos[i], hout[i], hin[j], pos[j]];
            let (poly, h) = flatten_poly(&ctrl, flat);
            OrigSeg { ctrl, poly, h }
        })
        .collect();
    let mut alive: Vec<usize> = (0..n).collect();
    let mut cost: Vec<Option<Option<(f64, Cubic)>>> = vec![None; n];
    let mut cur_out = hout.clone();
    let mut cur_in = hin.clone();
    let mut mod_out = vec![false; n];
    let mut mod_in = vec![false; n];
    let mut span_err = vec![0.0f64; n];
    let mut removed = 0;
    while alive.len() > min_keep {
        let m = alive.len();
        let range = if closed { 0..m } else { 1..m - 1 };
        let mut best: Option<(f64, usize)> = None;
        for j in range {
            let id = alive[j];
            if cost[id].is_none() {
                let (pi, ni) = (alive[(j + m - 1) % m], alive[(j + 1) % m]);
                let count = (ni + n - pi) % n;
                let sp: Vec<&OrigSeg> = (0..count).map(|k| &segs[(pi + k) % n]).collect();
                cost[id] = Some(certify_span(&sp, eps, flat, gap));
            }
            if let Some(Some((c, _))) = &cost[id] {
                if best.is_none_or(|(bc, _)| *c < bc) {
                    best = Some((*c, j));
                }
            }
        }
        let Some((c, j)) = best else { break };
        let (pi, id, ni) = (alive[(j + m - 1) % m], alive[j], alive[(j + 1) % m]);
        let cub = cost[id].unwrap().unwrap().1;
        cur_out[pi] = cub[1];
        cur_in[ni] = cub[2];
        mod_out[pi] = true;
        mod_in[ni] = true;
        span_err[pi] = c;
        alive.remove(j);
        cost[id] = None;
        cost[pi] = None;
        cost[ni] = None;
        removed += 1;
    }
    if removed == 0 {
        return unchanged();
    }
    let bound = alive.iter().map(|&i| span_err[i]).fold(0.0, f64::max);
    let out = alive
        .iter()
        .map(|&i| {
            let a = &anchors[i];
            let h = |p: P| if p == pos[i] { None } else { Some(Vec2::new(p[0] as f32, p[1] as f32)) };
            Anchor {
                pos: a.pos,
                handle_in: if mod_in[i] { h(cur_in[i]) } else { a.handle_in },
                handle_out: if mod_out[i] { h(cur_out[i]) } else { a.handle_out },
                weights: Vec::new(),
            }
        })
        .collect();
    SimplifiedPath { anchors: out, bound, removed }
}

/// Hausdorff-distance bound by dense evaluation (used by tests and `verify`): maximum over both
/// directions of the distance between flattened outlines, in the units of the anchors.
pub fn outline_polyline(anchors: &[Anchor], closed: bool, tol: f64) -> Vec<[f64; 2]> {
    let n = anchors.len();
    if n == 0 {
        return vec![];
    }
    let mut v = vec![p_of(anchors[0].pos)];
    let ns = if closed { n } else { n.saturating_sub(1) };
    let mut h = 0.0;
    for i in 0..ns {
        let j = (i + 1) % n;
        let c: Cubic = [p_of(anchors[i].pos), p_of(anchors[i].hout()), p_of(anchors[j].hin()), p_of(anchors[j].pos)];
        flatten_cubic(&c, tol, &mut v, &mut h, 0);
    }
    v
}
