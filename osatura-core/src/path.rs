// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
use crate::math::{Affine2, Vec2};
use crate::style::{Cap, Paint, StrokeStyle};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// Influence of one bone on one anchor. Weights are normalised at compile time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoneWeight {
    pub bone: String,
    pub weight: f32,
}

/// A Bézier anchor. Handles are absolute positions; `None` means "on the anchor"
/// (a sharp corner on that side).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Anchor {
    pub pos: Vec2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle_in: Option<Vec2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle_out: Option<Vec2>,
    /// Empty = not skinned (static).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weights: Vec<BoneWeight>,
}

impl Anchor {
    pub fn corner(pos: Vec2) -> Self {
        Anchor {
            pos,
            handle_in: None,
            handle_out: None,
            weights: Vec::new(),
        }
    }
    pub fn hin(&self) -> Vec2 {
        self.handle_in.unwrap_or(self.pos)
    }
    pub fn hout(&self) -> Vec2 {
        self.handle_out.unwrap_or(self.pos)
    }
    /// Apply `f` to the position and both handles.
    pub fn map_points(&mut self, f: &mut dyn FnMut(Vec2) -> Vec2) {
        self.pos = f(self.pos);
        self.handle_in = self.handle_in.map(&mut *f);
        self.handle_out = self.handle_out.map(&mut *f);
    }
}

/// An alternative shape for a path (same anchor count), blended in additively by
/// a `path_morph` channel: `base + Σ weightᵢ · (morphᵢ − base)`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Morph {
    pub name: String,
    pub anchors: Vec<Anchor>,
}

/// A cubic-Bézier vector path with paint attributes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VPath {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub closed: bool,
    pub anchors: Vec<Anchor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Paint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<StrokeStyle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphs: Vec<Morph>,
}

impl VPath {
    pub fn new(name: &str, closed: bool, anchors: Vec<Anchor>) -> Self {
        VPath {
            name: name.to_string(),
            closed,
            anchors,
            fill: None,
            stroke: None,
            morphs: Vec::new(),
        }
    }

    /// Add a morph target built by mapping each anchor of the base shape.
    pub fn with_morph(mut self, name: &str, f: impl Fn(&Anchor) -> Anchor) -> Self {
        let anchors = self.anchors.iter().map(f).collect();
        self.morphs.push(Morph {
            name: name.to_string(),
            anchors,
        });
        self
    }

    pub fn with_fill(mut self, hex: &str) -> Self {
        self.fill = Some(Paint::solid(hex));
        self
    }
    pub fn with_paint(mut self, p: Paint) -> Self {
        self.fill = Some(p);
        self
    }
    pub fn with_stroke(mut self, hex: &str, w: f32) -> Self {
        self.stroke = Some(StrokeStyle::solid(hex, w));
        self
    }
    pub fn with_stroke_style(mut self, s: StrokeStyle) -> Self {
        self.stroke = Some(s);
        self
    }

    /// Smooth curve through `pts` using Catmull-Rom → Bézier conversion.
    /// `tension` 1.0 is standard Catmull-Rom; 0.0 gives straight segments.
    pub fn smooth_through(name: &str, pts: &[Vec2], closed: bool, tension: f32) -> Self {
        let n = pts.len();
        let at = |i: isize| -> Vec2 {
            if closed {
                pts[i.rem_euclid(n as isize) as usize]
            } else {
                pts[i.clamp(0, n as isize - 1) as usize]
            }
        };
        let k = tension / 6.0;
        let anchors = (0..n as isize)
            .map(|i| {
                let t = (at(i + 1) - at(i - 1)) * k;
                let p = at(i);
                let (hin, hout) = if !closed && i == 0 {
                    (None, Some(p + t))
                } else if !closed && i == n as isize - 1 {
                    (Some(p - t), None)
                } else {
                    (Some(p - t), Some(p + t))
                };
                Anchor {
                    pos: p,
                    handle_in: hin,
                    handle_out: hout,
                    weights: Vec::new(),
                }
            })
            .collect();
        VPath::new(name, closed, anchors)
    }

    pub fn polygon(name: &str, pts: &[Vec2]) -> Self {
        VPath::new(name, true, pts.iter().map(|&p| Anchor::corner(p)).collect())
    }

    pub fn rect(name: &str, x: f32, y: f32, w: f32, h: f32) -> Self {
        VPath::polygon(
            name,
            &[
                Vec2::new(x, y),
                Vec2::new(x + w, y),
                Vec2::new(x + w, y + h),
                Vec2::new(x, y + h),
            ],
        )
    }

    /// Approximate circle with four cubic segments.
    pub fn circle(name: &str, center: Vec2, r: f32) -> Self {
        VPath::ellipse(name, center, r, r)
    }

    pub fn ellipse(name: &str, center: Vec2, rx: f32, ry: f32) -> Self {
        const K: f32 = 0.552_284_8;
        let c = center;
        let mk = |pos: Vec2, tin: Vec2, tout: Vec2| Anchor {
            pos,
            handle_in: Some(pos + Vec2::new(tin.x * rx * K, tin.y * ry * K)),
            handle_out: Some(pos + Vec2::new(tout.x * rx * K, tout.y * ry * K)),
            weights: Vec::new(),
        };
        VPath::new(
            name,
            true,
            vec![
                mk(
                    c + Vec2::new(rx, 0.0),
                    Vec2::new(0.0, -1.0),
                    Vec2::new(0.0, 1.0),
                ),
                mk(
                    c + Vec2::new(0.0, ry),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(-1.0, 0.0),
                ),
                mk(
                    c + Vec2::new(-rx, 0.0),
                    Vec2::new(0.0, 1.0),
                    Vec2::new(0.0, -1.0),
                ),
                mk(
                    c + Vec2::new(0.0, -ry),
                    Vec2::new(-1.0, 0.0),
                    Vec2::new(1.0, 0.0),
                ),
            ],
        )
    }

    /// Number of Bézier segments.
    pub fn segment_count(&self) -> usize {
        let n = self.anchors.len();
        if n < 2 {
            0
        } else if self.closed {
            n
        } else {
            n - 1
        }
    }

    fn segment(&self, i: usize) -> [Vec2; 4] {
        let a = &self.anchors[i];
        let b = &self.anchors[(i + 1) % self.anchors.len()];
        [a.pos, a.hout(), b.hin(), b.pos]
    }

    /// Polyline approximation. Closed paths do not repeat the first point.
    pub fn flatten(&self, tolerance: f32) -> Vec<Vec2> {
        self.flatten_t(tolerance)
            .into_iter()
            .map(|(p, _, _)| p)
            .collect()
    }

    /// Like [`flatten`](Self::flatten), but each point also says where it came from: the index of
    /// its segment and the curve parameter `t` (`0..=1`) on that segment. Point 0 is
    /// `(anchor 0, segment 0, t = 0)`.
    pub fn flatten_t(&self, tolerance: f32) -> Vec<(Vec2, usize, f32)> {
        let mut out = Vec::new();
        let Some(first) = self.anchors.first() else {
            return out;
        };
        out.push((first.pos, 0, 0.0));
        for i in 0..self.segment_count() {
            let [p0, c1, c2, p1] = self.segment(i);
            let ctrl = p0.distance(c1) + c1.distance(c2) + c2.distance(p1);
            let n = ((ctrl / tolerance.max(1e-3)).sqrt() * 0.6)
                .ceil()
                .clamp(1.0, 48.0) as usize;
            for k in 1..=n {
                let t = k as f32 / n as f32;
                let m = 1.0 - t;
                let pt = p0 * (m * m * m)
                    + c1 * (3.0 * m * m * t)
                    + c2 * (3.0 * m * t * t)
                    + p1 * (t * t * t);
                out.push((pt, i, t));
            }
        }
        if self.closed && out.len() > 1 {
            out.pop(); // last point coincides with the first
        }
        out
    }

    pub fn polyline(&self, tolerance: f32) -> Polyline {
        Polyline::new(self.flatten(tolerance), self.closed)
    }

    /// Conservative bounding box of anchors and handles: `(min, max)`.
    pub fn bounds(&self) -> Option<(Vec2, Vec2)> {
        let mut it = self.anchors.iter().flat_map(|a| [a.pos, a.hin(), a.hout()]);
        let first = it.next()?;
        let (mut lo, mut hi) = (first, first);
        for p in it {
            lo = Vec2::new(lo.x.min(p.x), lo.y.min(p.y));
            hi = Vec2::new(hi.x.max(p.x), hi.y.max(p.y));
        }
        Some((lo, hi))
    }

    pub fn transformed(&self, m: &Affine2) -> VPath {
        let mut out = self.clone();
        for a in &mut out.anchors {
            a.map_points(&mut |p| m.apply(p));
        }
        out
    }

    /// Interpolate geometry toward `other` (same anchor count). Paint stays `self`'s.
    pub fn lerp(&self, other: &VPath, t: f32) -> Option<VPath> {
        if self.anchors.len() != other.anchors.len() {
            return None;
        }
        let mut out = self.clone();
        for (a, b) in out.anchors.iter_mut().zip(&other.anchors) {
            a.pos = a.pos.lerp(b.pos, t);
            a.handle_in = match (a.handle_in, b.handle_in) {
                (None, None) => None,
                _ => Some(a.hin().lerp(b.hin(), t)),
            };
            a.handle_out = match (a.handle_out, b.handle_out) {
                (None, None) => None,
                _ => Some(a.hout().lerp(b.hout(), t)),
            };
        }
        Some(out)
    }

    /// SVG path data (`d` attribute).
    pub fn to_svg_d(&self) -> String {
        let mut d = String::new();
        let Some(first) = self.anchors.first() else {
            return d;
        };
        let _ = write!(d, "M{} {}", n2(first.pos.x), n2(first.pos.y));
        for i in 0..self.segment_count() {
            let [_, c1, c2, p1] = self.segment(i);
            let _ = write!(
                d,
                " C{} {} {} {} {} {}",
                n2(c1.x),
                n2(c1.y),
                n2(c2.x),
                n2(c2.y),
                n2(p1.x),
                n2(p1.y)
            );
        }
        if self.closed && self.anchors.len() > 1 {
            d.push_str(" Z");
        }
        d
    }
}

/// Two-decimal number without a negative zero ("-0.00").
pub(crate) fn n2(v: f32) -> String {
    let s = format!("{v:.2}");
    if s == "-0.00" {
        "0.00".to_string()
    } else {
        s
    }
}

/// A polyline with arc-length parameterisation.
#[derive(Clone, Debug)]
pub struct Polyline {
    pub pts: Vec<Vec2>,
    pub closed: bool,
    cum: Vec<f32>,
}

impl Polyline {
    pub fn new(pts: Vec<Vec2>, closed: bool) -> Self {
        let mut cum = Vec::with_capacity(pts.len() + 1);
        cum.push(0.0);
        let segs = if closed {
            pts.len()
        } else {
            pts.len().saturating_sub(1)
        };
        for i in 0..segs {
            let d = pts[i].distance(pts[(i + 1) % pts.len()]);
            cum.push(cum[i] + d);
        }
        Polyline { pts, closed, cum }
    }

    pub fn length(&self) -> f32 {
        *self.cum.last().unwrap_or(&0.0)
    }

    pub fn segment_count(&self) -> usize {
        self.cum.len().saturating_sub(1)
    }

    /// Point and unit tangent at arc length `dist` (clamped; wrapped if closed).
    pub fn sample(&self, dist: f32) -> (Vec2, Vec2) {
        let n = self.segment_count();
        if n == 0 {
            return (
                self.pts.first().copied().unwrap_or(Vec2::ZERO),
                Vec2::new(1.0, 0.0),
            );
        }
        let len = self.length();
        let d = if self.closed && len > 0.0 {
            dist.rem_euclid(len)
        } else {
            dist.clamp(0.0, len)
        };
        let i = (self.cum.partition_point(|&c| c <= d).saturating_sub(1)).min(n - 1);
        let (a, b) = (self.pts[i], self.pts[(i + 1) % self.pts.len()]);
        let seg = self.cum[i + 1] - self.cum[i];
        let t = if seg > 1e-9 {
            (d - self.cum[i]) / seg
        } else {
            0.0
        };
        (a.lerp(b, t), (b - a).normalized())
    }

    /// Closest point on the polyline: `(arc length, signed lateral offset)`,
    /// offset positive on the left of the direction of travel (normal = (-ty, tx)).
    pub fn project(&self, p: Vec2) -> (f32, f32) {
        let n = self.segment_count();
        if n == 0 {
            return (0.0, 0.0);
        }
        let mut best = (f32::MAX, 0.0, 0.0);
        for i in 0..n {
            let (a, b) = (self.pts[i], self.pts[(i + 1) % self.pts.len()]);
            let ab = b - a;
            let l2 = ab.dot(ab);
            let t = if l2 < 1e-12 {
                0.0
            } else {
                ((p - a).dot(ab) / l2).clamp(0.0, 1.0)
            };
            let q = a + ab * t;
            let d = p.distance(q);
            if d < best.0 {
                let dir = ab.normalized();
                let normal = Vec2::new(-dir.y, dir.x);
                let side = (p - q).dot(normal);
                best = (d, self.cum[i] + t * l2.sqrt(), side);
            }
        }
        (best.1, best.2)
    }
}

/// Result of expanding a stroke with variable width into fillable polygons.
pub struct Outline {
    pub polys: Vec<Vec<Vec2>>,
    /// `true` for closed paths (outer + inner ring need even-odd fill).
    pub even_odd: bool,
}

pub(crate) fn profile_at(profile: &[[f32; 2]], t: f32) -> f32 {
    match profile {
        [] => 1.0,
        [one] => one[1],
        _ => {
            if t <= profile[0][0] {
                return profile[0][1];
            }
            for w in profile.windows(2) {
                if t <= w[1][0] {
                    let span = w[1][0] - w[0][0];
                    let u = if span <= 0.0 {
                        1.0
                    } else {
                        (t - w[0][0]) / span
                    };
                    return w[0][1] + (w[1][1] - w[0][1]) * u;
                }
            }
            profile[profile.len() - 1][1]
        }
    }
}

/// Expand `poly` into a polygon whose half-width follows `style.width * profile(t)`.
pub fn width_outline(poly: &Polyline, style: &StrokeStyle) -> Option<Outline> {
    let pts = &poly.pts;
    let n = pts.len();
    if n < 2 {
        return None;
    }
    let total = poly.length().max(1e-6);
    let seg_dir = |i: usize| (pts[(i + 1) % n] - pts[i]).normalized();
    let mut left = Vec::with_capacity(n);
    let mut right = Vec::with_capacity(n);
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        let dir_in = if i > 0 || poly.closed {
            Some(seg_dir((i + n - 1) % n))
        } else {
            None
        };
        let dir_out = if i + 1 < n || poly.closed {
            Some(seg_dir(i))
        } else {
            None
        };
        let (d0, d1) = match (dir_in, dir_out) {
            (Some(a), Some(b)) => (a, b),
            (Some(a), None) => (a, a),
            (None, Some(b)) => (b, b),
            (None, None) => return None,
        };
        let nrm = |d: Vec2| Vec2::new(-d.y, d.x);
        let mut m = (nrm(d0) + nrm(d1)).normalized();
        if m == Vec2::ZERO {
            m = nrm(d1);
        }
        let cos_half = m.dot(nrm(d1)).max(0.5); // miter limit: at most 2×
        let t = poly.cum[i] / total;
        let hw = 0.5 * style.width * profile_at(&style.width_profile, t) / cos_half;
        left.push(pts[i] + m * hw);
        right.push(pts[i] - m * hw);
    }
    if poly.closed {
        return Some(Outline {
            polys: vec![left, right],
            even_odd: true,
        });
    }
    // Open: left forward, cap, right backward, cap.
    let mut poly_pts: Vec<Vec2> = Vec::new();
    poly_pts.extend(&left);
    let cap_pts = |center: Vec2, dir: Vec2, r: f32, out: &mut Vec<Vec2>| match style.cap {
        Cap::Butt => {}
        Cap::Square => {
            let nrm = Vec2::new(-dir.y, dir.x);
            out.push(center + nrm * r + dir * r);
            out.push(center - nrm * r + dir * r);
        }
        Cap::Round => {
            let base = dir.y.atan2(dir.x);
            for k in 1..8 {
                let a = base + std::f32::consts::FRAC_PI_2 - std::f32::consts::PI * k as f32 / 8.0;
                out.push(center + Vec2::from_angle(a) * r);
            }
        }
    };
    let end_r = left[n - 1].distance(pts[n - 1]);
    cap_pts(pts[n - 1], seg_dir(n - 2), end_r, &mut poly_pts);
    poly_pts.extend(right.iter().rev());
    let start_r = left[0].distance(pts[0]);
    cap_pts(pts[0], -seg_dir(0), start_r, &mut poly_pts);
    Some(Outline {
        polys: vec![poly_pts],
        even_odd: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_path_d() {
        let p = VPath::polygon(
            "tri",
            &[
                Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(0.0, 10.0),
            ],
        );
        let d = p.to_svg_d();
        assert!(d.starts_with("M0.00 0.00 C"));
        assert!(d.ends_with(" Z"));
        assert_eq!(d.matches('C').count(), 3);
    }

    #[test]
    fn smooth_closed_has_handles_everywhere() {
        let pts = [
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let p = VPath::smooth_through("s", &pts, true, 1.0);
        assert!(p
            .anchors
            .iter()
            .all(|a| a.handle_in.is_some() && a.handle_out.is_some()));
    }

    #[test]
    fn circle_flatten_has_right_radius_and_length() {
        let c = VPath::circle("c", Vec2::new(5.0, 5.0), 10.0);
        assert_eq!(c.to_svg_d().matches('C').count(), 4);
        let poly = c.polyline(0.1);
        for p in &poly.pts {
            assert!((p.distance(Vec2::new(5.0, 5.0)) - 10.0).abs() < 0.1);
        }
        assert!((poly.length() - std::f32::consts::TAU * 10.0).abs() < 0.5);
    }

    #[test]
    fn polyline_sample_and_project_are_inverse() {
        let pl = Polyline::new(
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(10.0, 10.0),
            ],
            false,
        );
        assert_eq!(pl.length(), 20.0);
        let (p, t) = pl.sample(15.0);
        assert!(p.distance(Vec2::new(10.0, 5.0)) < 1e-4);
        assert!(t.distance(Vec2::new(0.0, 1.0)) < 1e-4);
        let (d, side) = pl.project(Vec2::new(5.0, -3.0));
        assert!((d - 5.0).abs() < 1e-4);
        assert!(
            (side - -3.0).abs() < 1e-4,
            "left of +x travel is -y in screen space: {side}"
        );
    }

    #[test]
    fn bounds_and_transform_and_lerp() {
        let r = VPath::rect("r", 0.0, 0.0, 10.0, 20.0);
        let (lo, hi) = r.bounds().unwrap();
        assert_eq!((lo, hi), (Vec2::new(0.0, 0.0), Vec2::new(10.0, 20.0)));
        let moved = r.transformed(&Affine2::translate(Vec2::new(5.0, 5.0)));
        assert_eq!(moved.bounds().unwrap().0, Vec2::new(5.0, 5.0));
        let mid = r.lerp(&moved, 0.5).unwrap();
        assert_eq!(mid.bounds().unwrap().0, Vec2::new(2.5, 2.5));
        assert!(r.lerp(&VPath::circle("c", Vec2::ZERO, 1.0), 0.5).is_some());
        assert!(r.lerp(&VPath::polygon("t", &[Vec2::ZERO]), 0.5).is_none());
    }

    #[test]
    fn variable_width_outline_tapers() {
        let pl = Polyline::new(
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(50.0, 0.0),
                Vec2::new(100.0, 0.0),
            ],
            false,
        );
        let mut st = StrokeStyle::solid("#000", 10.0);
        st.cap = Cap::Butt;
        st.width_profile = vec![[0.0, 1.0], [1.0, 0.0]];
        let o = width_outline(&pl, &st).unwrap();
        assert!(!o.even_odd);
        let ys: Vec<f32> = o.polys[0].iter().map(|p| p.y.abs()).collect();
        assert!((ys[0] - 5.0).abs() < 1e-3, "starts at full half-width");
        assert!(ys.iter().cloned().fold(0.0, f32::max) <= 5.0 + 1e-3);
        assert!(ys[2] < 1e-3, "tapers to a point");
    }

    #[test]
    fn closed_width_outline_has_two_rings() {
        let c = VPath::circle("c", Vec2::ZERO, 20.0);
        let st = StrokeStyle::solid("#000", 4.0);
        let o = width_outline(&c.polyline(0.2), &st).unwrap();
        assert!(o.even_odd && o.polys.len() == 2);
        let mut r = [o.polys[0][0].length(), o.polys[1][0].length()];
        r.sort_by(f32::total_cmp);
        assert!(
            (r[0] - 18.0).abs() < 0.2 && (r[1] - 22.0).abs() < 0.2,
            "{r:?}"
        );
    }
}
