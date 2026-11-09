// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Collision of two 2D capsules (a segment with a radius) and self-collision of a chain of capsules.
//!
//! Basis: elementary geometry (the distance between two segments, and a capsule as the set of points within `r` of a segment), as in
//! the standard computational-geometry literature (C. Ericson, *Real-Time Collision Detection*, 2005, section on the closest points of two
//! segments; D. Eberly's note on the distance between segments). Neither source was read for this implementation: everything below is
//! derived from the geometry and checked by the arithmetic tests of docs/clean-room/specs/10-coliziune-capsula-capsula.md.
//!
//! Two capsules collide when the minimum distance `d` between their segments is strictly below `ra + rb`. The penetration depth is
//! `ra + rb - d`, the normal points from A to B. When the segments touch (`d == 0`) the normal is the left normal of A's direction
//! (A's direction rotated by +90 degrees), or `(1, 0)` when A is a point. Used by `physics::simulate_strip` when
//! `PhysicsOptions::self_collision` is set (CLI `physics ... selfcollide=R`), see docs/clean-room/specs/INTREBARI.md.

/// A 2D point or vector.
pub type P2 = [f64; 2];

/// A segment `a`-`b` thickened by `radius`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capsule {
    pub a: P2,
    pub b: P2,
    pub radius: f64,
}

impl Capsule {
    pub fn new(a: P2, b: P2, radius: f64) -> Capsule {
        Capsule { a, b, radius }
    }
    fn check(&self) -> Result<(), String> {
        if !self.a.iter().chain(&self.b).all(|v| v.is_finite()) {
            return Err("capsule coordinates must be finite".into());
        }
        if !self.radius.is_finite() || self.radius <= 0.0 {
            return Err(format!("capsule radius must be a positive finite number, got {}", self.radius));
        }
        Ok(())
    }
}

/// Result of a collision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    /// Minimum distance between the two segments.
    pub distance: f64,
    /// `ra + rb - distance` (> 0).
    pub depth: f64,
    /// Unit normal from A towards B.
    pub normal: P2,
    /// Closest point on A's segment and on B's segment.
    pub point_a: P2,
    pub point_b: P2,
}

fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn at(p: P2, d: P2, t: f64) -> P2 {
    [p[0] + d[0] * t, p[1] + d[1] * t]
}

/// Closest points of the segments `p0-p1` and `q0-q1`: the parameters `(s, t)` in `[0, 1]` and the points. For parallel segments the
/// pair nearest to the start of the first segment is returned. Zero-length segments are points.
pub fn closest_points(p0: P2, p1: P2, q0: P2, q1: P2) -> (f64, f64, P2, P2) {
    let d1 = sub(p1, p0);
    let d2 = sub(q1, q0);
    let r = sub(p0, q0);
    let (a, e) = (dot(d1, d1), dot(d2, d2));
    let f = dot(d2, r);
    let (s, t);
    if a == 0.0 && e == 0.0 {
        (s, t) = (0.0, 0.0);
    } else if a == 0.0 {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = dot(d1, r);
        if e == 0.0 {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = dot(d1, d2);
            let denom = a * e - b * b;
            let mut s0 = if denom > 1e-12 * a * e { ((b * f - c * e) / denom).clamp(0.0, 1.0) } else { 0.0 };
            let mut t0 = (b * s0 + f) / e;
            if t0 < 0.0 {
                t0 = 0.0;
                s0 = (-c / a).clamp(0.0, 1.0);
            } else if t0 > 1.0 {
                t0 = 1.0;
                s0 = ((b - c) / a).clamp(0.0, 1.0);
            }
            (s, t) = (s0, t0);
        }
    }
    (s, t, at(p0, d1, s), at(q0, d2, t))
}

/// Tests two capsules. `Ok(None)` when they do not overlap (tangent capsules do not collide); `Err` for non-finite coordinates or a radius
/// that is not positive.
pub fn collide(a: &Capsule, b: &Capsule) -> Result<Option<Contact>, String> {
    a.check()?;
    b.check()?;
    let (_, _, pa, pb) = closest_points(a.a, a.b, b.a, b.b);
    let diff = sub(pb, pa);
    let distance = dot(diff, diff).sqrt();
    let sum = a.radius + b.radius;
    if distance >= sum {
        return Ok(None);
    }
    let normal = if distance > 0.0 {
        [diff[0] / distance, diff[1] / distance]
    } else {
        let d = sub(a.b, a.a);
        let len = dot(d, d).sqrt();
        if len > 0.0 {
            [-d[1] / len, d[0] / len]
        } else {
            [1.0, 0.0]
        }
    };
    Ok(Some(Contact { distance, depth: sum - distance, normal, point_a: pa, point_b: pb }))
}

/// Position corrections `(for A, for B)` that separate a contact: A moves by `-n * depth * wa / (wa + wb)`, B by `+n * depth * wb / (wa + wb)`
/// (`wa`, `wb` are inverse masses, 0 = fixed). No correction when both are 0. Negative or non-finite weights are an error.
pub fn corrections(c: &Contact, wa: f64, wb: f64) -> Result<(P2, P2), String> {
    if !(wa.is_finite() && wb.is_finite()) || wa < 0.0 || wb < 0.0 {
        return Err("inverse masses must be finite and >= 0".into());
    }
    let total = wa + wb;
    if total == 0.0 {
        return Ok(([0.0, 0.0], [0.0, 0.0]));
    }
    let (ka, kb) = (c.depth * wa / total, c.depth * wb / total);
    Ok(([-c.normal[0] * ka, -c.normal[1] * ka], [c.normal[0] * kb, c.normal[1] * kb]))
}

/// The capsules of a chain of points: segment `i` runs from `points[i]` to `points[i + 1]`.
pub fn chain_capsules(points: &[P2], radius: f64) -> Vec<Capsule> {
    points.windows(2).map(|w| Capsule::new(w[0], w[1], radius)).collect()
}

/// Colliding pairs `(i, j, contact)` with `i < j` of a chain, skipping neighbours (`j - i <= 1`).
pub fn self_collisions(chain: &[Capsule]) -> Result<Vec<(usize, usize, Contact)>, String> {
    let mut out = Vec::new();
    for i in 0..chain.len() {
        for j in (i + 2)..chain.len() {
            if let Some(c) = collide(&chain[i], &chain[j])? {
                out.push((i, j, c));
            }
        }
    }
    Ok(out)
}

/// Position corrections for every capsule of a chain from its self-collisions, accumulated and not yet applied (so the result does not
/// depend on the order of the pairs). `weights` are the inverse masses per capsule.
pub fn chain_corrections(chain: &[Capsule], weights: &[f64]) -> Result<Vec<P2>, String> {
    if weights.len() != chain.len() {
        return Err("one inverse mass per capsule is needed".into());
    }
    let mut acc = vec![[0.0, 0.0]; chain.len()];
    for (i, j, c) in self_collisions(chain)? {
        let (ca, cb) = corrections(&c, weights[i], weights[j])?;
        acc[i][0] += ca[0];
        acc[i][1] += ca[1];
        acc[j][0] += cb[0];
        acc[j][1] += cb[1];
    }
    Ok(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} vs {b}");
    }
    fn near2(a: P2, b: P2) {
        near(a[0], b[0]);
        near(a[1], b[1]);
    }
    fn cap(a: P2, b: P2, r: f64) -> Capsule {
        Capsule::new(a, b, r)
    }
    fn base() -> Capsule {
        cap([0.0, 0.0], [10.0, 0.0], 1.0)
    }

    #[test]
    fn far_apart() {
        assert!(collide(&base(), &cap([5.0, 3.0], [5.0, 10.0], 1.0)).unwrap().is_none());
    }

    #[test]
    fn overlap_depth_normal_and_points() {
        let c = collide(&base(), &cap([5.0, 1.5], [5.0, 10.0], 1.0)).unwrap().unwrap();
        near(c.distance, 1.5);
        near(c.depth, 0.5);
        near2(c.normal, [0.0, 1.0]);
        near2(c.point_a, [5.0, 0.0]);
        near2(c.point_b, [5.0, 1.5]);
    }

    #[test]
    fn parallel_segments() {
        let c = collide(&cap([0.0, 0.0], [10.0, 0.0], 1.5), &cap([3.0, 2.0], [7.0, 2.0], 1.5)).unwrap().unwrap();
        near(c.distance, 2.0);
        near(c.depth, 1.0);
        near2(c.normal, [0.0, 1.0]);
    }

    #[test]
    fn crossing_segments_use_the_left_normal() {
        let c = collide(&base(), &cap([5.0, -5.0], [5.0, 5.0], 1.0)).unwrap().unwrap();
        near(c.distance, 0.0);
        near(c.depth, 2.0);
        near2(c.normal, [0.0, 1.0]);
    }

    #[test]
    fn end_regions() {
        let c = collide(&base(), &cap([11.5, 0.0], [20.0, 0.0], 1.0)).unwrap().unwrap();
        near(c.depth, 0.5);
        near2(c.normal, [1.0, 0.0]);
        let c = collide(&base(), &cap([11.0, 5.0], [11.0, -5.0], 1.0)).unwrap().unwrap();
        near2(c.point_a, [10.0, 0.0]);
        near2(c.point_b, [11.0, 0.0]);
        near(c.depth, 1.0);
        near2(c.normal, [1.0, 0.0]);
    }

    #[test]
    fn circles_and_degenerate_segments() {
        let c = collide(&cap([0.0, 0.0], [0.0, 0.0], 1.0), &cap([1.5, 0.0], [1.5, 0.0], 1.0)).unwrap().unwrap();
        near(c.depth, 0.5);
        near2(c.normal, [1.0, 0.0]);
        // a point against a segment
        let c = collide(&base(), &cap([5.0, 1.0], [5.0, 1.0], 1.5)).unwrap().unwrap();
        near(c.distance, 1.0);
        // coincident points: normal (1, 0)
        let c = collide(&cap([2.0, 2.0], [2.0, 2.0], 1.0), &cap([2.0, 2.0], [2.0, 2.0], 1.0)).unwrap().unwrap();
        near2(c.normal, [1.0, 0.0]);
        near(c.depth, 2.0);
        // collinear overlapping segments
        let c = collide(&base(), &cap([5.0, 0.0], [15.0, 0.0], 1.0)).unwrap().unwrap();
        near(c.distance, 0.0);
        near2(c.normal, [0.0, 1.0]);
    }

    #[test]
    fn tangent_is_not_a_collision() {
        assert!(collide(&base(), &cap([5.0, 2.0], [5.0, 10.0], 1.0)).unwrap().is_none());
    }

    #[test]
    fn corrections_by_inverse_mass() {
        let c = collide(&base(), &cap([5.0, 1.5], [5.0, 10.0], 1.0)).unwrap().unwrap();
        let (a, b) = corrections(&c, 1.0, 1.0).unwrap();
        near2(a, [0.0, -0.25]);
        near2(b, [0.0, 0.25]);
        let (a, b) = corrections(&c, 1.0, 0.0).unwrap();
        near2(a, [0.0, -0.5]);
        near2(b, [0.0, 0.0]);
        let (a, b) = corrections(&c, 0.0, 0.0).unwrap();
        assert_eq!((a, b), ([0.0, 0.0], [0.0, 0.0]));
        assert!(corrections(&c, -1.0, 1.0).is_err());
    }

    fn chain(r: f64) -> Vec<Capsule> {
        vec![cap([0.0, 0.0], [10.0, 0.0], r), cap([10.0, 0.0], [10.0, 1.0], r), cap([10.0, 1.0], [0.0, 1.0], r), cap([0.0, 1.0], [0.0, 0.5], r)]
    }

    #[test]
    fn chain_self_collision_small_radius() {
        let hits = self_collisions(&chain(0.4)).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!((hits[0].0, hits[0].1), (0, 3));
        near(hits[0].2.distance, 0.5);
        near(hits[0].2.depth, 0.3);
    }

    #[test]
    fn chain_self_collision_large_radius() {
        let hits = self_collisions(&chain(0.6)).unwrap();
        let pairs: Vec<(usize, usize)> = hits.iter().map(|h| (h.0, h.1)).collect();
        assert_eq!(pairs, vec![(0, 2), (0, 3)]);
        near(hits[0].2.depth, 0.2);
        near(hits[1].2.depth, 0.7);
        // neighbours (0,1), (1,2), (2,3) are skipped although their ends coincide; (1,3) is 10 apart
        assert!(!pairs.contains(&(1, 3)));
    }

    #[test]
    fn accumulated_corrections_do_not_depend_on_pair_order() {
        let c = chain(0.6);
        let w = [1.0; 4];
        let fwd = chain_corrections(&c, &w).unwrap();
        // the same chain with the capsule list reversed gives the mirrored result
        let rev: Vec<Capsule> = c.iter().rev().map(|k| cap(k.b, k.a, k.radius)).collect();
        let back = chain_corrections(&rev, &w).unwrap();
        for i in 0..4 {
            near2(fwd[i], back[3 - i]);
        }
        assert!(chain_corrections(&c, &w[..3]).is_err());
        assert_eq!(chain_capsules(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], 0.5).len(), 2);
    }

    #[test]
    fn errors() {
        assert!(collide(&cap([0.0, 0.0], [1.0, 0.0], 0.0), &base()).is_err());
        assert!(collide(&base(), &cap([0.0, 0.0], [1.0, 0.0], -1.0)).is_err());
        assert!(collide(&base(), &cap([f64::NAN, 0.0], [1.0, 0.0], 1.0)).is_err());
        assert!(collide(&base(), &cap([0.0, 0.0], [f64::INFINITY, 0.0], 1.0)).is_err());
        assert!(collide(&base(), &cap([0.0, 0.0], [1.0, 0.0], f64::NAN)).is_err());
    }
}
