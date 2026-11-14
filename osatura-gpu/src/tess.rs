// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! CPU tessellation for the GPU renderer (no GPU types here, so it is unit-tested on any machine).
//!
//! * Fills: a triangle fan from the first vertex. Overlaps and self-intersections are resolved on the GPU by the
//!   stencil buffer (stencil-then-cover), which is the textbook way to fill arbitrary non-convex polygons
//!   without ear clipping or a monotone partition.
//! * Strokes: every segment becomes a quad; joins (miter with limit / bevel / round) and caps (butt / square /
//!   round) are small triangle fans. All triangles are unioned by the stencil pass, so overlaps never double-blend.
//! * Dashes: arc-length splitting of the polyline into open pieces.
//!
//! Stroking happens in the layer's local space and the triangles are then mapped through the local->pixel
//! affine, so a non-uniform scale gives an elliptical pen exactly like the CPU rasteriser.

use osatura_core::math::{Affine2, Vec2};
use osatura_core::style::{Cap, Join};

/// Flat list of triangle vertices (3 per triangle), in target pixels.
pub type Tris = Vec<[f32; 2]>;

/// Stroke geometry request.
#[derive(Clone, Copy, Debug)]
pub struct StrokeSpec {
    pub width: f32,
    pub cap: Cap,
    pub join: Join,
    /// SVG-style miter limit (ratio of miter length to stroke width); the CPU rasteriser uses 4.
    pub miter_limit: f32,
}

struct Sink<'a> {
    out: &'a mut Tris,
    m: &'a Affine2,
    /// Pixels per local unit (for choosing arc subdivision).
    px_scale: f32,
}

impl Sink<'_> {
    fn tri(&mut self, a: Vec2, b: Vec2, c: Vec2) {
        for p in [a, b, c] {
            let q = self.m.apply(p);
            self.out.push([q.x, q.y]);
        }
    }
    fn quad(&mut self, a: Vec2, b: Vec2, c: Vec2, d: Vec2) {
        self.tri(a, b, c);
        self.tri(a, c, d);
    }
    /// Fan around `c` from angle `a0` sweeping `sweep` radians (radius `r` in local units).
    fn arc(&mut self, c: Vec2, r: f32, a0: f32, sweep: f32) {
        let n = arc_steps(r * self.px_scale, sweep.abs());
        let mut prev = c + Vec2::from_angle(a0) * r;
        for k in 1..=n {
            let a = a0 + sweep * k as f32 / n as f32;
            let cur = c + Vec2::from_angle(a) * r;
            self.tri(c, prev, cur);
            prev = cur;
        }
    }
}

/// Number of chords needed to approximate an arc of `angle` radians at `radius_px` within 0.05 px.
pub fn arc_steps(radius_px: f32, angle: f32) -> usize {
    let tol = 0.05f32;
    let r = radius_px.max(1e-3);
    let da = 2.0 * (1.0 - tol / r).clamp(-1.0, 1.0).acos();
    ((angle / da.max(1e-3)).ceil() as usize).clamp(1, 256)
}

/// Triangle fan over a closed polygon already in pixel space.
pub fn fill_fan(poly: &[Vec2]) -> Tris {
    let mut out = Tris::new();
    if poly.len() < 3 {
        return out;
    }
    for i in 1..poly.len() - 1 {
        for p in [poly[0], poly[i], poly[i + 1]] {
            out.push([p.x, p.y]);
        }
    }
    out
}

/// Map a local-space polygon through `m`.
pub fn map_poly(poly: &[Vec2], m: &Affine2) -> Vec<Vec2> {
    poly.iter().map(|p| m.apply(*p)).collect()
}

fn px_scale(m: &Affine2) -> f32 {
    m.a.hypot(m.b).max(m.c.hypot(m.d)).max(1e-6)
}

fn perp(d: Vec2) -> Vec2 {
    Vec2::new(-d.y, d.x)
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

/// Remove consecutive duplicates (and the closing duplicate of a closed ring).
fn dedupe(pts: &[Vec2], closed: bool) -> Vec<Vec2> {
    let mut v: Vec<Vec2> = Vec::with_capacity(pts.len());
    for p in pts {
        if !p.x.is_finite() || !p.y.is_finite() {
            continue;
        }
        if v.last().is_none_or(|l| l.distance(*p) > 1e-6) {
            v.push(*p);
        }
    }
    if closed && v.len() > 1 && v[0].distance(v[v.len() - 1]) <= 1e-6 {
        v.pop();
    }
    v
}

fn join_at(s: &mut Sink, p: Vec2, d0: Vec2, d1: Vec2, hw: f32, join: Join, limit: f32) {
    let cr = cross(d0, d1);
    let dt = d0.dot(d1);
    if cr.abs() < 1e-6 && dt > 0.0 {
        return; // collinear
    }
    // The outer side is opposite to the turn direction (left normal = perp(d)).
    let side = if cr > 0.0 { -1.0 } else { 1.0 };
    let v0 = perp(d0) * (hw * side);
    let v1 = perp(d1) * (hw * side);
    let (a, b) = (p + v0, p + v1);
    match join {
        Join::Bevel => s.tri(p, a, b),
        Join::Miter => {
            let cos_half = ((1.0 + dt) * 0.5).max(0.0).sqrt();
            if cos_half > 1e-4 && 1.0 / cos_half <= limit {
                let tip = p + (v0 + v1) * (1.0 / (1.0 + dt));
                s.tri(p, a, tip);
                s.tri(p, tip, b);
            } else {
                s.tri(p, a, b);
            }
        }
        Join::Round => {
            let a0 = v0.y.atan2(v0.x);
            let mut sweep = v1.y.atan2(v1.x) - a0;
            while sweep > std::f32::consts::PI {
                sweep -= std::f32::consts::TAU;
            }
            while sweep < -std::f32::consts::PI {
                sweep += std::f32::consts::TAU;
            }
            s.arc(p, hw, a0, sweep);
        }
    }
}

fn cap_at(s: &mut Sink, p: Vec2, d: Vec2, hw: f32, cap: Cap) {
    // `d` points away from the line end.
    match cap {
        Cap::Butt => {}
        Cap::Square => {
            let n = perp(d) * hw;
            s.quad(p + n, p - n, p - n + d * hw, p + n + d * hw);
        }
        Cap::Round => {
            let a0 = perp(d).y.atan2(perp(d).x);
            s.arc(p, hw, a0, -std::f32::consts::PI);
        }
    }
}

/// Triangles of a constant-width stroke of the local-space polyline `pts`, mapped through `m` (local -> pixels).
pub fn stroke_tris(pts: &[Vec2], closed: bool, spec: &StrokeSpec, m: &Affine2) -> Tris {
    let mut out = Tris::new();
    let hw = spec.width * 0.5;
    if (hw.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !hw.is_finite() {
        return out;
    }
    let v = dedupe(pts, closed);
    let mut s = Sink { out: &mut out, m, px_scale: px_scale(m) };
    match v.len() {
        0 => {}
        1 => match spec.cap {
            Cap::Butt => {}
            Cap::Round => s.arc(v[0], hw, 0.0, std::f32::consts::TAU),
            Cap::Square => {
                let (x, y) = (Vec2::new(hw, 0.0), Vec2::new(0.0, hw));
                s.quad(v[0] - x - y, v[0] + x - y, v[0] + x + y, v[0] - x + y);
            }
        },
        n => {
            let segs = if closed { n } else { n - 1 };
            let dir = |i: usize| (v[(i + 1) % n] - v[i]).normalized();
            for i in 0..segs {
                let (p0, p1, d) = (v[i], v[(i + 1) % n], dir(i));
                let nn = perp(d) * hw;
                s.quad(p0 + nn, p0 - nn, p1 - nn, p1 + nn);
            }
            let lim = spec.miter_limit.max(1.0);
            if closed {
                // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
                #[allow(clippy::needless_range_loop)]
                for i in 0..n {
                    join_at(&mut s, v[i], dir((i + n - 1) % n), dir(i), hw, spec.join, lim);
                }
            } else {
                // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
                #[allow(clippy::needless_range_loop)]
                for i in 1..n - 1 {
                    join_at(&mut s, v[i], dir(i - 1), dir(i), hw, spec.join, lim);
                }
                cap_at(&mut s, v[0], -dir(0), hw, spec.cap);
                cap_at(&mut s, v[n - 1], dir(n - 2), hw, spec.cap);
            }
        }
    }
    out
}

/// Upper bound on dash pieces per path (a guard against absurd patterns, not a quality limit).
pub const MAX_DASHES: usize = 20_000;

/// A valid dash pattern: an even count (>= 2) of finite, positive lengths.
pub fn dash_valid(d: &[f32]) -> bool {
    d.len() >= 2 && d.len().is_multiple_of(2) && d.iter().all(|x| x.is_finite() && *x > 0.0)
}

/// Split a polyline into the "on" pieces of a dash pattern. A closed ring is treated as an open polyline that
/// returns to its start. Errors if the pattern would produce more than [`MAX_DASHES`] pieces.
pub fn dash(pts: &[Vec2], closed: bool, pattern: &[f32], offset: f32) -> Result<Vec<Vec<Vec2>>, String> {
    let mut path = dedupe(pts, closed);
    if closed && path.len() > 1 {
        path.push(path[0]);
    }
    let total: f32 = pattern.iter().sum();
    let mut pieces: Vec<Vec<Vec2>> = Vec::new();
    if path.len() < 2 || !dash_valid(pattern) {
        return Ok(pieces);
    }
    // Position inside the pattern at distance 0.
    let mut o = offset.rem_euclid(total);
    let mut idx = 0usize;
    while o >= pattern[idx] {
        o -= pattern[idx];
        idx = (idx + 1) % pattern.len();
    }
    let mut left = pattern[idx] - o; // length remaining in the current entry
    let mut cur: Vec<Vec2> = if idx.is_multiple_of(2) { vec![path[0]] } else { Vec::new() };
    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = a.distance(b);
        let mut t = 0.0f32;
        while seg - t > left {
            t += left;
            let p = a.lerp(b, t / seg);
            if idx.is_multiple_of(2) {
                cur.push(p);
                pieces.push(std::mem::take(&mut cur));
                if pieces.len() > MAX_DASHES {
                    return Err(format!("dash pattern produces more than {MAX_DASHES} pieces"));
                }
            } else {
                cur = vec![p];
            }
            idx = (idx + 1) % pattern.len();
            left = pattern[idx];
        }
        left -= seg - t;
        if idx.is_multiple_of(2) {
            cur.push(b);
        }
    }
    if idx.is_multiple_of(2) && cur.len() >= 2 {
        pieces.push(cur);
    }
    Ok(pieces)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area_by_sampling(tris: &Tris, lo: f32, hi: f32, step: f32) -> f32 {
        let inside = |p: [f32; 2], t: &[[f32; 2]]| {
            let s = |a: [f32; 2], b: [f32; 2]| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
            let (d1, d2, d3) = (s(t[0], t[1]), s(t[1], t[2]), s(t[2], t[0]));
            (d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0) || (d1 <= 0.0 && d2 <= 0.0 && d3 <= 0.0)
        };
        let mut count = 0usize;
        let n = ((hi - lo) / step) as usize;
        for iy in 0..n {
            for ix in 0..n {
                let p = [lo + (ix as f32 + 0.5) * step, lo + (iy as f32 + 0.5) * step];
                if tris.chunks(3).any(|t| inside(p, t)) {
                    count += 1;
                }
            }
        }
        count as f32 * step * step
    }

    fn spec(cap: Cap, join: Join) -> StrokeSpec {
        StrokeSpec { width: 2.0, cap, join, miter_limit: 4.0 }
    }

    #[test]
    fn fan_of_square_is_two_triangles() {
        let sq = [Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0), Vec2::new(4.0, 4.0), Vec2::new(0.0, 4.0)];
        let t = fill_fan(&sq);
        assert_eq!(t.len(), 6);
        assert!((area_by_sampling(&t, -1.0, 5.0, 0.05) - 16.0).abs() < 0.3);
        assert!(fill_fan(&sq[..2]).is_empty());
    }

    #[test]
    fn butt_line_area_is_length_times_width() {
        let t = stroke_tris(&[Vec2::new(1.0, 5.0), Vec2::new(11.0, 5.0)], false, &spec(Cap::Butt, Join::Miter), &Affine2::IDENTITY);
        assert!((area_by_sampling(&t, 0.0, 12.0, 0.05) - 20.0).abs() < 0.4);
    }

    #[test]
    fn square_and_round_caps_extend_the_line() {
        let line = [Vec2::new(2.0, 5.0), Vec2::new(10.0, 5.0)];
        let sq = stroke_tris(&line, false, &spec(Cap::Square, Join::Miter), &Affine2::IDENTITY);
        assert!((area_by_sampling(&sq, 0.0, 12.0, 0.05) - 20.0).abs() < 0.5, "8 + 2*1 long, 2 wide");
        let rd = stroke_tris(&line, false, &spec(Cap::Round, Join::Miter), &Affine2::IDENTITY);
        let want = 16.0 + std::f32::consts::PI;
        assert!((area_by_sampling(&rd, 0.0, 12.0, 0.05) - want).abs() < 0.4);
    }

    #[test]
    fn miter_join_fills_the_corner_and_bevel_does_not() {
        let l = [Vec2::new(1.0, 1.0), Vec2::new(8.0, 1.0), Vec2::new(8.0, 8.0)];
        let miter = area_by_sampling(&stroke_tris(&l, false, &spec(Cap::Butt, Join::Miter), &Affine2::IDENTITY), 0.0, 10.0, 0.05);
        let bevel = area_by_sampling(&stroke_tris(&l, false, &spec(Cap::Butt, Join::Bevel), &Affine2::IDENTITY), 0.0, 10.0, 0.05);
        let round = area_by_sampling(&stroke_tris(&l, false, &spec(Cap::Butt, Join::Round), &Affine2::IDENTITY), 0.0, 10.0, 0.05);
        // Two 7x2 bars overlap in a 1x1 block (centre half-width 1): 14 + 14 - 1 + outer corner.
        assert!((miter - 28.0).abs() < 0.5, "miter {miter}");
        assert!((bevel - 27.5).abs() < 0.5, "bevel {bevel}");
        assert!(round > bevel && round < miter, "round {round}");
    }

    #[test]
    fn miter_limit_falls_back_to_bevel_on_sharp_angles() {
        let l = [Vec2::new(1.0, 1.0), Vec2::new(9.0, 1.2), Vec2::new(1.0, 1.4)];
        let a = stroke_tris(&l, false, &spec(Cap::Butt, Join::Miter), &Affine2::IDENTITY);
        let far = a.iter().map(|p| p[0]).fold(0.0f32, f32::max);
        assert!(far < 10.5, "no long spike beyond the limit, got x={far}");
    }

    #[test]
    fn closed_stroke_has_no_gap_at_the_start() {
        let sq = [Vec2::new(2.0, 2.0), Vec2::new(8.0, 2.0), Vec2::new(8.0, 8.0), Vec2::new(2.0, 8.0)];
        let t = stroke_tris(&sq, true, &spec(Cap::Butt, Join::Miter), &Affine2::IDENTITY);
        // Ring between 1..9 and 3..7: 64 - 16 = 48.
        assert!((area_by_sampling(&t, 0.0, 10.0, 0.05) - 48.0).abs() < 0.6);
    }

    #[test]
    fn transform_maps_vertices_and_degenerate_input_is_empty() {
        let m = Affine2::translate(Vec2::new(100.0, 0.0));
        let t = stroke_tris(&[Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0)], false, &spec(Cap::Butt, Join::Miter), &m);
        assert!(t.iter().all(|p| p[0] >= 100.0 - 1e-3));
        let none = StrokeSpec { width: 0.0, ..spec(Cap::Butt, Join::Miter) };
        assert!(stroke_tris(&[Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0)], false, &none, &m).is_empty());
        assert!(stroke_tris(&[], false, &spec(Cap::Round, Join::Round), &m).is_empty());
        assert!(stroke_tris(&[Vec2::new(f32::NAN, 0.0), Vec2::new(1.0, 1.0)], false, &spec(Cap::Butt, Join::Miter), &m).is_empty(), "non-finite points are dropped");
    }

    #[test]
    fn dot_stroke_for_a_single_point() {
        let t = stroke_tris(&[Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0)], false, &spec(Cap::Round, Join::Round), &Affine2::IDENTITY);
        assert!((area_by_sampling(&t, 0.0, 10.0, 0.05) - std::f32::consts::PI).abs() < 0.2);
        let b = stroke_tris(&[Vec2::new(5.0, 5.0)], false, &spec(Cap::Butt, Join::Round), &Affine2::IDENTITY);
        assert!(b.is_empty());
    }

    #[test]
    fn dashes_split_by_arc_length() {
        let pieces = dash(&[Vec2::new(0.0, 0.0), Vec2::new(40.0, 0.0)], false, &[10.0, 10.0], 0.0).unwrap();
        assert_eq!(pieces.len(), 2);
        assert_eq!((pieces[0][0].x, pieces[0][1].x), (0.0, 10.0));
        assert_eq!((pieces[1][0].x, pieces[1][1].x), (20.0, 30.0));
        // An offset of 5 shifts the pattern: first piece is 5 long.
        let shifted = dash(&[Vec2::new(0.0, 0.0), Vec2::new(40.0, 0.0)], false, &[10.0, 10.0], 5.0).unwrap();
        assert_eq!((shifted[0][0].x, shifted[0].last().unwrap().x), (0.0, 5.0));
        assert_eq!(shifted[1][0].x, 15.0);
        // Negative offsets wrap.
        let neg = dash(&[Vec2::new(0.0, 0.0), Vec2::new(40.0, 0.0)], false, &[10.0, 10.0], -15.0).unwrap();
        assert_eq!(neg[0][0].x, 0.0);
    }

    #[test]
    fn dashes_follow_corners_and_reject_bad_patterns() {
        let l = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)];
        let pieces = dash(&l, false, &[15.0, 100.0], 0.0).unwrap();
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].len(), 3, "the dash bends around the corner");
        assert!(dash(&l, false, &[0.0, 1.0], 0.0).unwrap().is_empty());
        assert!(dash(&l, false, &[1.0], 0.0).unwrap().is_empty());
        let err = dash(&[Vec2::new(0.0, 0.0), Vec2::new(1.0e6, 0.0)], false, &[1.0, 1.0], 0.0);
        assert!(err.is_err());
    }

    #[test]
    fn arc_steps_grow_with_radius() {
        assert!(arc_steps(100.0, std::f32::consts::PI) > arc_steps(2.0, std::f32::consts::PI));
        assert!(arc_steps(0.0, 1.0) >= 1);
    }
}
