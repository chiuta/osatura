// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `autotrace` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// geometry: f64 points, Schneider fitting
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct P {
    pub(super) x: f64,
    pub(super) y: f64,
}

pub(super) fn p(x: f64, y: f64) -> P {
    P { x, y }
}

impl std::ops::Add for P {
    type Output = P;
    fn add(self, o: P) -> P {
        p(self.x + o.x, self.y + o.y)
    }
}

impl std::ops::Sub for P {
    type Output = P;
    fn sub(self, o: P) -> P {
        p(self.x - o.x, self.y - o.y)
    }
}

impl std::ops::Mul<f64> for P {
    type Output = P;
    fn mul(self, k: f64) -> P {
        p(self.x * k, self.y * k)
    }
}

impl P {
    pub(super) fn dot(self, o: P) -> f64 {
        self.x * o.x + self.y * o.y
    }
    pub(super) fn len(self) -> f64 {
        self.x.hypot(self.y)
    }
    pub(super) fn norm(self) -> P {
        let l = self.len();
        if l < 1e-12 {
            p(0.0, 0.0)
        } else {
            self * (1.0 / l)
        }
    }
    pub(super) fn v2(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

/// Cubic Bezier segment.
#[derive(Clone, Copy, Debug)]
pub struct Bez {
    pub(super) p: [P; 4],
    pub(super) straight: bool,
}

pub(super) fn bez_at(b: &[P; 4], t: f64) -> P {
    let m = 1.0 - t;
    b[0] * (m * m * m) + b[1] * (3.0 * m * m * t) + b[2] * (3.0 * m * t * t) + b[3] * (t * t * t)
}

pub(super) fn bez_d1(b: &[P; 4], t: f64) -> P {
    let m = 1.0 - t;
    (b[1] - b[0]) * (3.0 * m * m) + (b[2] - b[1]) * (6.0 * m * t) + (b[3] - b[2]) * (3.0 * t * t)
}

pub(super) fn bez_d2(b: &[P; 4], t: f64) -> P {
    let m = 1.0 - t;
    (b[2] - b[1] * 2.0 + b[0]) * (6.0 * m) + (b[3] - b[2] * 2.0 + b[1]) * (6.0 * t)
}

pub(super) fn chord_params(d: &[P]) -> Vec<f64> {
    let mut u = vec![0.0; d.len()];
    for i in 1..d.len() {
        u[i] = u[i - 1] + (d[i] - d[i - 1]).len();
    }
    let total = u[d.len() - 1];
    if total > 0.0 {
        for v in u.iter_mut() {
            *v /= total;
        }
    }
    u
}

pub(super) fn generate_bezier(d: &[P], u: &[f64], t1: P, t2: P) -> [P; 4] {
    let n = d.len();
    let (d0, d3) = (d[0], d[n - 1]);
    let (mut c00, mut c01, mut c11, mut x0, mut x1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..n {
        let t = u[i];
        let m = 1.0 - t;
        let (b0, b1, b2, b3) = (m * m * m, 3.0 * m * m * t, 3.0 * m * t * t, t * t * t);
        let a1 = t1 * b1;
        let a2 = t2 * b2;
        c00 += a1.dot(a1);
        c01 += a1.dot(a2);
        c11 += a2.dot(a2);
        let tmp = d[i] - (d0 * (b0 + b1) + d3 * (b2 + b3));
        x0 += a1.dot(tmp);
        x1 += a2.dot(tmp);
    }
    let det = c00 * c11 - c01 * c01;
    let (al, ar) = if det.abs() > 1e-12 { ((x0 * c11 - x1 * c01) / det, (c00 * x1 - c01 * x0) / det) } else { (0.0, 0.0) };
    let seg = (d3 - d0).len();
    let eps = 1e-6 * seg;
    if al < eps || ar < eps || !al.is_finite() || !ar.is_finite() {
        let dist = seg / 3.0;
        return [d0, d0 + t1 * dist, d3 + t2 * dist, d3];
    }
    [d0, d0 + t1 * al, d3 + t2 * ar, d3]
}

pub(super) fn reparam(b: &[P; 4], d: &[P], u: &[f64]) -> Vec<f64> {
    d.iter()
        .zip(u)
        .map(|(&pt, &t)| {
            let q = bez_at(b, t);
            let q1 = bez_d1(b, t);
            let q2 = bez_d2(b, t);
            let num = (q - pt).dot(q1);
            let den = q1.dot(q1) + (q - pt).dot(q2);
            if den.abs() < 1e-12 {
                t
            } else {
                (t - num / den).clamp(0.0, 1.0)
            }
        })
        .collect()
}

pub(super) fn max_err(b: &[P; 4], d: &[P], u: &[f64]) -> (f64, usize) {
    let mut worst = (0.0, d.len() / 2);
    for i in 1..d.len() - 1 {
        let e = (bez_at(b, u[i]) - d[i]).dot(bez_at(b, u[i]) - d[i]);
        if e >= worst.0 {
            worst = (e, i);
        }
    }
    worst
}

/// Schneider's recursive fit. `t1` points into the curve at the start, `t2` points into the curve at the end.
pub(super) fn fit_cubic(d: &[P], t1: P, t2: P, err2: f64, out: &mut Vec<Bez>) {
    let n = d.len();
    if n == 2 {
        let dist = (d[1] - d[0]).len() / 3.0;
        out.push(Bez { p: [d[0], d[0] + t1 * dist, d[1] + t2 * dist, d[1]], straight: false });
        return;
    }
    let mut u = chord_params(d);
    let mut b = generate_bezier(d, &u, t1, t2);
    let (mut e, mut split) = max_err(&b, d, &u);
    if e < err2 {
        out.push(Bez { p: b, straight: false });
        return;
    }
    // Newton-Raphson re-parameterisation (Schneider tries it when the error is within a small factor of the
    // tolerance; here the gate is wider and the loop stops when it no longer improves)
    if e < err2 * 400.0 {
        for _ in 0..12 {
            let prev = e;
            u = reparam(&b, d, &u);
            b = generate_bezier(d, &u, t1, t2);
            let r = max_err(&b, d, &u);
            e = r.0;
            split = r.1;
            if e < err2 {
                out.push(Bez { p: b, straight: false });
                return;
            }
            if e > prev * 0.99 {
                break;
            }
        }
    }
    let split = split.clamp(1, n - 2);
    let centre = (d[split - 1] - d[split + 1]).norm();
    let centre = if centre.len() < 0.5 { (d[n - 1] - d[0]).norm() * -1.0 } else { centre };
    fit_cubic(&d[..=split], t1, centre, err2, out);
    fit_cubic(&d[split..], centre * -1.0, t2, err2, out);
}

pub(super) fn dist_to_chord(a: P, b: P, q: P) -> f64 {
    let ab = b - a;
    let l = ab.len();
    if l < 1e-12 {
        return (q - a).len();
    }
    ((q - a).x * ab.y - (q - a).y * ab.x).abs() / l
}

// ---------------------------------------------------------------------------------------------
// polygon simplification
// ---------------------------------------------------------------------------------------------

/// Douglas-Peucker (1973) on an open polyline; the end points are always kept.
pub fn douglas_peucker(pts: &[Vec2], tol: f32) -> Vec<Vec2> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![false; pts.len()];
    keep[0] = true;
    keep[pts.len() - 1] = true;
    let mut stack = vec![(0usize, pts.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let mut worst = (0.0f64, a);
        for i in a + 1..b {
            let d = dist_to_chord(p(pts[a].x as f64, pts[a].y as f64), p(pts[b].x as f64, pts[b].y as f64), p(pts[i].x as f64, pts[i].y as f64));
            if d > worst.0 {
                worst = (d, i);
            }
        }
        if worst.0 > tol as f64 {
            keep[worst.1] = true;
            stack.push((a, worst.1));
            stack.push((worst.1, b));
        }
    }
    pts.iter().zip(&keep).filter(|(_, k)| **k).map(|(q, _)| *q).collect()
}

/// Closed-polygon Douglas-Peucker: split at the two points that are farthest apart.
pub fn douglas_peucker_closed(pts: &[Vec2], tol: f32) -> Vec<Vec2> {
    let n = pts.len();
    if n < 4 {
        return pts.to_vec();
    }
    let mut far = (0.0f32, 1usize);
    for i in 1..n {
        let d = pts[0].distance(pts[i]);
        if d > far.0 {
            far = (d, i);
        }
    }
    let mut a: Vec<Vec2> = pts[..=far.1].to_vec();
    let mut b: Vec<Vec2> = pts[far.1..].to_vec();
    b.push(pts[0]);
    a = douglas_peucker(&a, tol);
    b = douglas_peucker(&b, tol);
    a.pop();
    b.pop();
    a.extend(b);
    a
}

/// Visvalingam-Whyatt (1993) on a closed polygon: repeatedly drop the vertex whose triangle with its
/// neighbours has the smallest area until every remaining triangle has area >= `min_area`.
pub fn visvalingam_closed(pts: &[Vec2], min_area: f32) -> Vec<Vec2> {
    let mut v: Vec<Vec2> = pts.to_vec();
    let tri = |a: Vec2, b: Vec2, c: Vec2| ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs() * 0.5;
    while v.len() > 3 {
        let n = v.len();
        let mut best = (f32::MAX, 0);
        for i in 0..n {
            let a = tri(v[(i + n - 1) % n], v[i], v[(i + 1) % n]);
            if a < best.0 {
                best = (a, i);
            }
        }
        if best.0 >= min_area {
            break;
        }
        v.remove(best.1);
    }
    v
}
