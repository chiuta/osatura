// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Certified curve distances (part of `lod`; see the module docs in `mod.rs`).
use super::*;

// Certified curve distances

pub(super) fn dist_pt_seg(p: P, a: P, b: P) -> f64 {
    let ab = sub(b, a);
    let l2 = dot(ab, ab);
    let t = if l2 < 1e-300 { 0.0 } else { (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0) };
    dist(p, add(a, mul(ab, t)))
}

pub(super) fn dist_to_poly(p: P, poly: &[P]) -> f64 {
    match poly.len() {
        0 => f64::INFINITY,
        1 => dist(p, poly[0]),
        _ => poly.windows(2).map(|w| dist_pt_seg(p, w[0], w[1])).fold(f64::INFINITY, f64::min),
    }
}

pub(super) fn cubic_at(c: &Cubic, t: f64) -> P {
    let u = 1.0 - t;
    let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        b0 * c[0][0] + b1 * c[1][0] + b2 * c[2][0] + b3 * c[3][0],
        b0 * c[0][1] + b1 * c[1][1] + b2 * c[2][1] + b3 * c[3][1],
    ]
}

pub(super) fn cubic_d1(c: &Cubic, t: f64) -> P {
    let u = 1.0 - t;
    let a = mul(sub(c[1], c[0]), 3.0 * u * u);
    let b = mul(sub(c[2], c[1]), 6.0 * u * t);
    let d = mul(sub(c[3], c[2]), 3.0 * t * t);
    add(add(a, b), d)
}

pub(super) fn cubic_d2(c: &Cubic, t: f64) -> P {
    let u = 1.0 - t;
    let a = mul(add(sub(c[2], mul(c[1], 2.0)), c[0]), 6.0 * u);
    let b = mul(add(sub(c[3], mul(c[2], 2.0)), c[1]), 6.0 * t);
    add(a, b)
}

pub(super) fn split_cubic(c: &Cubic) -> (Cubic, Cubic) {
    let m = |a: P, b: P| mul(add(a, b), 0.5);
    let (p01, p12, p23) = (m(c[0], c[1]), m(c[1], c[2]), m(c[2], c[3]));
    let (p012, p123) = (m(p01, p12), m(p12, p23));
    let mid = m(p012, p123);
    ([c[0], p01, p012, mid], [mid, p123, p23, c[3]])
}

/// Distance bound of a cubic from its chord segment: the curve lies in the convex hull of its
/// control points and the distance to a segment is convex, so the maximum is at a control point.
pub(super) fn hull_flat(c: &Cubic) -> f64 {
    dist_pt_seg(c[1], c[0], c[3]).max(dist_pt_seg(c[2], c[0], c[3]))
}

/// Append the flattening of `c` (without its first point) to `out`; `hmax` receives the largest
/// control-polygon flatness of any emitted piece, i.e. a rigorous bound of the distance between
/// the curve and the polyline.
pub(super) fn flatten_cubic(c: &Cubic, tol: f64, out: &mut Vec<P>, hmax: &mut f64, depth: u32) {
    let h = hull_flat(c);
    if h <= tol || depth >= 22 {
        if h > *hmax {
            *hmax = h;
        }
        out.push(c[3]);
    } else {
        let (a, b) = split_cubic(c);
        flatten_cubic(&a, tol, out, hmax, depth + 1);
        flatten_cubic(&b, tol, out, hmax, depth + 1);
    }
}

/// Polyline through the curve plus the certified flattening bound.
pub fn flatten_poly(c: &[[f64; 2]; 4], tol: f64) -> (Vec<[f64; 2]>, f64) {
    let mut v = vec![c[0]];
    let mut h = 0.0;
    flatten_cubic(c, tol, &mut v, &mut h, 0);
    (v, h)
}

/// Nearest-distance index for a polyline: uniform grid of segment lists (exact queries).
pub(super) struct PolyIdx<'a> {
    pub(super) pts: &'a [P],
    pub(super) x0: f64,
    pub(super) y0: f64,
    pub(super) cell: f64,
    pub(super) nx: usize,
    pub(super) ny: usize,
    pub(super) cells: Vec<Vec<u32>>,
}

impl<'a> PolyIdx<'a> {
    pub(super) fn new(pts: &'a [P]) -> PolyIdx<'a> {
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for q in pts {
            x0 = x0.min(q[0]);
            y0 = y0.min(q[1]);
            x1 = x1.max(q[0]);
            y1 = y1.max(q[1]);
        }
        let m = pts.len().saturating_sub(1).max(1);
        let diag = (x1 - x0).hypot(y1 - y0).max(1e-9);
        let cell = (diag / (m as f64).sqrt().max(1.0)).max(diag / 512.0).max(1e-9);
        let nx = (((x1 - x0) / cell).floor() as usize + 1).min(1024);
        let ny = (((y1 - y0) / cell).floor() as usize + 1).min(1024);
        let mut idx = PolyIdx { pts, x0, y0, cell, nx, ny, cells: vec![Vec::new(); nx * ny] };
        for (i, w) in pts.windows(2).enumerate() {
            let (cx0, cx1) = (idx.cx(w[0][0].min(w[1][0])), idx.cx(w[0][0].max(w[1][0])));
            let (cy0, cy1) = (idx.cy(w[0][1].min(w[1][1])), idx.cy(w[0][1].max(w[1][1])));
            for cy in cy0..=cy1 {
                for cx in cx0..=cx1 {
                    idx.cells[cy * nx + cx].push(i as u32);
                }
            }
        }
        idx
    }
    pub(super) fn cx(&self, x: f64) -> usize {
        (((x - self.x0) / self.cell).floor().max(0.0) as usize).min(self.nx - 1)
    }
    pub(super) fn cy(&self, y: f64) -> usize {
        (((y - self.y0) / self.cell).floor().max(0.0) as usize).min(self.ny - 1)
    }
    pub(super) fn nearest(&self, p: P) -> (f64, usize) {
        if self.pts.len() < 2 {
            return (self.pts.first().map_or(f64::INFINITY, |q| dist(p, *q)), 0);
        }
        let (cx, cy) = (self.cx(p[0]) as i64, self.cy(p[1]) as i64);
        let mut best = (f64::INFINITY, 0usize);
        let maxr = self.nx.max(self.ny) as i64;
        for r in 0..=maxr {
            for yy in (cy - r).max(0)..=(cy + r).min(self.ny as i64 - 1) {
                for xx in (cx - r).max(0)..=(cx + r).min(self.nx as i64 - 1) {
                    if (yy - cy).abs() != r && (xx - cx).abs() != r {
                        continue;
                    }
                    for &si in &self.cells[yy as usize * self.nx + xx as usize] {
                        let w = &self.pts[si as usize..si as usize + 2];
                        let d = dist_pt_seg(p, w[0], w[1]);
                        if d < best.0 || (d == best.0 && (si as usize) < best.1) {
                            best = (d, si as usize);
                        }
                    }
                }
            }
            if best.0 <= (r as f64) * self.cell {
                break;
            }
        }
        best
    }
}

/// Upper bound of the directed Hausdorff distance `sup_{x in a} dist(x, b)` for polylines, never
/// more than `gap` above the true value (branch and bound on the 1-Lipschitz distance function).
pub fn directed_hausdorff_upper(a: &[[f64; 2]], b: &[[f64; 2]], gap: f64) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let idx = PolyIdx::new(b);
    let dtp = |p: P| -> (f64, usize) { idx.nearest(p) };
    let seg_d = |p: P, s: usize| -> f64 {
        if b.len() < 2 {
            dist(p, b[0])
        } else {
            dist_pt_seg(p, b[s], b[s + 1])
        }
    };
    if a.len() == 1 {
        return dtp(a[0]).0;
    }
    let gap = gap.max(1e-12);
    let mut lb = 0.0f64;
    let mut res = 0.0f64;
    // (p, q, d(p), d(q), nearest segment of p, nearest segment of q, depth)
    let mut stack: Vec<(P, P, f64, f64, usize, usize, u32)> = Vec::new();
    for w in a.windows(2) {
        let (d0, s0) = dtp(w[0]);
        let (d1, s1) = dtp(w[1]);
        lb = lb.max(d0).max(d1);
        stack.push((w[0], w[1], d0, d1, s0, s1, 0));
        while let Some((p, q, da, db, sa, sb, depth)) = stack.pop() {
            let len = dist(p, q);
            // upper bounds of sup over [p, q]: Lipschitz, and convexity of the distance to one fixed segment
            let mut ub = (da + db + len) * 0.5;
            for s in [sa, sb] {
                ub = ub.min(seg_d(p, s).max(seg_d(q, s)));
            }
            if ub <= lb + gap || depth >= 40 || len < 1e-14 {
                res = res.max(ub.min(da.max(db) + len));
                continue;
            }
            let m = mul(add(p, q), 0.5);
            let (dm, sm) = dtp(m);
            lb = lb.max(dm);
            stack.push((m, q, dm, db, sm, sb, depth + 1));
            stack.push((p, m, da, dm, sa, sm, depth + 1));
        }
    }
    res.max(lb)
}

/// Certified-up-to-`gap` symmetric Hausdorff distance bound of two polylines.
pub fn hausdorff_upper(a: &[[f64; 2]], b: &[[f64; 2]], gap: f64) -> f64 {
    directed_hausdorff_upper(a, b, gap).max(directed_hausdorff_upper(b, a, gap))
}

pub(super) fn resample(poly: &[P], n: usize) -> Vec<P> {
    if poly.len() < 2 || n < 2 {
        return poly.to_vec();
    }
    let mut cum = vec![0.0f64];
    for w in poly.windows(2) {
        cum.push(cum.last().unwrap() + dist(w[0], w[1]));
    }
    let total = *cum.last().unwrap();
    if total < 1e-15 {
        return vec![poly[0], poly[poly.len() - 1]];
    }
    let mut out = Vec::with_capacity(n);
    let mut j = 0;
    for i in 0..n {
        let s = total * i as f64 / (n - 1) as f64;
        while j + 2 < cum.len() && cum[j + 1] < s {
            j += 1;
        }
        let seg = (cum[j + 1] - cum[j]).max(1e-300);
        let t = ((s - cum[j]) / seg).clamp(0.0, 1.0);
        out.push(add(poly[j], mul(sub(poly[j + 1], poly[j]), t)));
    }
    out
}

/// Discrete Frechet distance of two point sequences (monotone matching).
pub fn discrete_frechet(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let (n, m) = (a.len(), b.len());
    let mut ca = vec![0.0f64; n * m];
    for i in 0..n {
        for j in 0..m {
            let d = dist(a[i], b[j]);
            ca[i * m + j] = match (i, j) {
                (0, 0) => d,
                (0, _) => ca[j - 1].max(d),
                (_, 0) => ca[(i - 1) * m].max(d),
                _ => ca[(i - 1) * m + j].min(ca[(i - 1) * m + j - 1]).min(ca[i * m + j - 1]).max(d),
            };
        }
    }
    ca[n * m - 1]
}
