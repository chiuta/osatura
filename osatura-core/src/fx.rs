// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Procedural effects: liquid metaballs (marching squares), polygon booleans,
//! and closed-form deterministic particles.

use crate::math::Vec2;
use crate::noise::hash01;
use crate::path::VPath;
use crate::style::{Paint, Rgba, StrokeStyle};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ------------------------------------------------------------------------------------------
// Boolean operations
// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoolOp {
    Union,
    Intersect,
    Difference,
    Xor,
}

fn to_pts(c: &[[f32; 2]]) -> Vec<Vec2> {
    c.iter().map(|p| Vec2::new(p[0], p[1])).collect()
}

fn dist2(a: Vec2, b: Vec2) -> f32 {
    let d = a - b;
    d.dot(d)
}

/// Join a hole into its outline with a zero-width bridge so one closed polygon can
/// represent a shape with holes under non-zero winding fill (holes come out wound
/// opposite to the outline).
fn bridge_holes(outer: Vec<Vec2>, holes: &[Vec<Vec2>]) -> Vec<Vec2> {
    let mut poly = outer;
    for h in holes {
        if h.len() < 3 || poly.is_empty() {
            continue;
        }
        let (mut bi, mut bj, mut bd) = (0, 0, f32::MAX);
        for (i, &p) in poly.iter().enumerate() {
            for (j, &q) in h.iter().enumerate() {
                let d = dist2(p, q);
                if d < bd {
                    (bi, bj, bd) = (i, j, d);
                }
            }
        }
        let mut out = Vec::with_capacity(poly.len() + h.len() + 2);
        out.extend_from_slice(&poly[..=bi]);
        for k in 0..=h.len() {
            out.push(h[(bj + k) % h.len()]);
        }
        out.extend_from_slice(&poly[bi..]);
        poly = out;
    }
    poly
}

/// Boolean operation on two sets of polygons (each polygon a closed point list).
/// Returns one polygon per resulting shape; holes are bridged into their outline.
pub fn boolean(subject: &[Vec<Vec2>], clip: &[Vec<Vec2>], op: BoolOp) -> Vec<Vec<Vec2>> {
    let conv = |ps: &[Vec<Vec2>]| -> Vec<Vec<[f32; 2]>> {
        ps.iter().map(|p| p.iter().map(|v| [v.x, v.y]).collect()).collect()
    };
    let (s, c) = (conv(subject), conv(clip));
    let rule = match op {
        BoolOp::Union => OverlayRule::Union,
        BoolOp::Intersect => OverlayRule::Intersect,
        BoolOp::Difference => OverlayRule::Difference,
        BoolOp::Xor => OverlayRule::Xor,
    };
    let shapes = s.overlay(&c, rule, FillRule::NonZero);
    shapes
        .iter()
        .filter_map(|shape| {
            let outer = to_pts(shape.first()?);
            let holes: Vec<Vec<Vec2>> = shape.iter().skip(1).map(|h| to_pts(h)).collect();
            Some(bridge_holes(outer, &holes))
        })
        .collect()
}

/// Boolean on whole paths: `paths[0]` is the subject, the rest are combined (union) into the clip.
/// Result polygons take the subject's paint.
pub fn boolean_paths(paths: &[VPath], op: BoolOp, tol: f32) -> Vec<VPath> {
    let Some(first) = paths.first() else { return Vec::new() };
    let subj = vec![first.flatten(tol)];
    let clip: Vec<Vec<Vec2>> = paths[1..].iter().map(|p| p.flatten(tol)).collect();
    boolean(&subj, &clip, op)
        .into_iter()
        .enumerate()
        .map(|(i, pts)| {
            let mut v = VPath::polygon(&format!("{}{}", if first.name.is_empty() { "b" } else { &first.name }, i), &pts);
            v.fill = first.fill.clone();
            v.stroke = first.stroke.clone();
            v
        })
        .collect()
}

// ------------------------------------------------------------------------------------------
// Metaballs / liquid
// ------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Blob {
    /// Layer space, or the bone's local space when `bone` is set.
    pub pos: Vec2,
    pub radius: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bone: Option<String>,
}

fn field(blobs: &[(Vec2, f32)], p: Vec2) -> f32 {
    let mut f = 0.0;
    for &(c, r) in blobs {
        let d2 = dist2(p, c).max(1e-4);
        f += r * r / d2;
    }
    f
}

/// Iso-contours of the metaball field Σ r²/d² = `threshold` via marching squares.
/// `cell` is the grid size in layer units. Returns closed polylines.
pub fn metaball_contours(blobs: &[(Vec2, f32)], threshold: f32, cell: f32) -> Vec<Vec<Vec2>> {
    if blobs.is_empty() || (threshold.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || (cell.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) {
        return Vec::new();
    }
    // Field falls below threshold beyond r/sqrt(t); pad so contours close inside the grid.
    let pad = blobs.iter().map(|b| b.1).fold(0.0f32, f32::max) * (1.0 / threshold.sqrt() + 0.5) + cell * 2.0;
    let (mut lo, mut hi) = (Vec2::new(f32::MAX, f32::MAX), Vec2::new(f32::MIN, f32::MIN));
    for &(c, _) in blobs {
        lo = Vec2::new(lo.x.min(c.x), lo.y.min(c.y));
        hi = Vec2::new(hi.x.max(c.x), hi.y.max(c.y));
    }
    lo = lo - Vec2::new(pad, pad);
    hi += Vec2::new(pad, pad);
    let nx = (((hi.x - lo.x) / cell).ceil() as usize).clamp(1, 600);
    let ny = (((hi.y - lo.y) / cell).ceil() as usize).clamp(1, 600);
    let stride = nx + 1;
    let at = |i: usize, j: usize| lo + Vec2::new(i as f32 * cell, j as f32 * cell);
    let mut vals = vec![0.0f32; stride * (ny + 1)];
    for j in 0..=ny {
        for i in 0..=nx {
            vals[j * stride + i] = field(blobs, at(i, j));
        }
    }
    // Edge keys: (i, j, 0) horizontal edge (i,j)-(i+1,j); (i, j, 1) vertical edge (i,j)-(i,j+1).
    type Key = (usize, usize, u8);
    let mut adj: HashMap<Key, Vec<Key>> = HashMap::new();
    let mut link = |a: Key, b: Key| {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    };
    for j in 0..ny {
        for i in 0..nx {
            let v = [vals[j * stride + i], vals[j * stride + i + 1], vals[(j + 1) * stride + i + 1], vals[(j + 1) * stride + i]];
            let mut case = 0u8;
            for (k, x) in v.iter().enumerate() {
                if *x >= threshold {
                    case |= 1 << k;
                }
            }
            let e = [(i, j, 0u8), (i + 1, j, 1), (i, j + 1, 0), (i, j, 1)]; // top, right, bottom, left
            let centre = (v[0] + v[1] + v[2] + v[3]) * 0.25 >= threshold;
            let segs: &[(usize, usize)] = match case {
                1 | 14 => &[(3, 0)],
                2 | 13 => &[(0, 1)],
                3 | 12 => &[(3, 1)],
                4 | 11 => &[(1, 2)],
                5 => {
                    if centre { &[(0, 1), (2, 3)] } else { &[(3, 0), (1, 2)] }
                }
                6 | 9 => &[(0, 2)],
                7 | 8 => &[(3, 2)],
                10 => {
                    if centre { &[(3, 0), (1, 2)] } else { &[(0, 1), (2, 3)] }
                }
                _ => &[],
            };
            for &(a, b) in segs {
                link(e[a], e[b]);
            }
        }
    }
    let pos_of = |k: Key| -> Vec2 {
        let (i, j, dir) = k;
        let (a, b) = if dir == 0 { ((i, j), (i + 1, j)) } else { ((i, j), (i, j + 1)) };
        let (va, vb) = (vals[a.1 * stride + a.0], vals[b.1 * stride + b.0]);
        let t = if (vb - va).abs() < 1e-9 { 0.5 } else { ((threshold - va) / (vb - va)).clamp(0.0, 1.0) };
        at(a.0, a.1).lerp(at(b.0, b.1), t)
    };
    let mut keys: Vec<Key> = adj.keys().copied().collect();
    keys.sort(); // deterministic order
    let mut visited: std::collections::HashSet<Key> = std::collections::HashSet::new();
    let mut out = Vec::new();
    for start in keys {
        if visited.contains(&start) {
            continue;
        }
        let mut contour = vec![pos_of(start)];
        visited.insert(start);
        let mut prev = start;
        let mut cur = match adj[&start].first() {
            Some(&n) => n,
            None => continue,
        };
        while cur != start && !visited.contains(&cur) {
            visited.insert(cur);
            contour.push(pos_of(cur));
            let next = adj[&cur].iter().copied().find(|&n| n != prev).unwrap_or(start);
            prev = cur;
            cur = next;
        }
        if contour.len() >= 3 {
            out.push(contour);
        }
    }
    out
}

/// Liquid layer geometry: smooth closed paths around the metaball iso-contour.
pub fn liquid_paths(blobs: &[(Vec2, f32)], threshold: f32, cell: f32, fill: &Option<Paint>, stroke: &Option<StrokeStyle>) -> Vec<VPath> {
    metaball_contours(blobs, threshold, cell)
        .into_iter()
        .enumerate()
        .map(|(i, pts)| {
            let mut p = VPath::smooth_through(&format!("liquid{i}"), &pts, true, 1.0);
            p.fill = fill.clone();
            p.stroke = stroke.clone();
            p
        })
        .collect()
}

// ------------------------------------------------------------------------------------------
// Particles (closed form: position is a pure function of time, so any frame can be evaluated)
// ------------------------------------------------------------------------------------------

fn d_rate() -> f32 {
    20.0
}
fn d_life() -> f32 {
    1.0
}
fn d_speed() -> f32 {
    60.0
}
fn d_spread() -> f32 {
    30.0
}
fn d_dir() -> f32 {
    -90.0
}
fn d_size() -> f32 {
    3.0
}
fn d_max() -> usize {
    2000
}
fn d_c0() -> Rgba {
    Rgba::hex("#ffffff")
}
fn d_c1() -> Rgba {
    Rgba::hex("#ffffff").with_alpha(0.0)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmitterSpec {
    /// Where particles are born (layer space, or bone-local when `bone` is set).
    #[serde(default)]
    pub pos: Vec2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bone: Option<String>,
    /// Particles per second.
    #[serde(default = "d_rate")]
    pub rate: f32,
    /// Seconds a particle lives.
    #[serde(default = "d_life")]
    pub lifetime: f32,
    /// ±fraction of lifetime (0..1).
    #[serde(default)]
    pub lifetime_jitter: f32,
    #[serde(default = "d_speed")]
    pub speed: f32,
    #[serde(default)]
    pub speed_jitter: f32,
    /// Emission direction in degrees (0 = +x, −90 = up on screen).
    #[serde(default = "d_dir")]
    pub direction: f32,
    /// Full cone width in degrees.
    #[serde(default = "d_spread")]
    pub spread: f32,
    #[serde(default)]
    pub gravity: Vec2,
    #[serde(default = "d_size")]
    pub size_start: f32,
    #[serde(default = "d_size")]
    pub size_end: f32,
    #[serde(default = "d_c0")]
    pub color_start: Rgba,
    #[serde(default = "d_c1")]
    pub color_end: Rgba,
    #[serde(default)]
    pub seed: u32,
    /// Emission starts at this time (seconds).
    #[serde(default)]
    pub start: f32,
    /// Emission stops after this many seconds (None = forever).
    #[serde(default)]
    pub duration: Option<f32>,
    /// Hard cap on live particles (safety).
    #[serde(default = "d_max")]
    pub max_particles: usize,
}

impl Default for EmitterSpec {
    fn default() -> Self {
        serde_json::from_str("{}").expect("all fields have defaults")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    pub pos: Vec2,
    /// 0 at birth → 1 at death.
    pub life: f32,
    pub size: f32,
    pub color: Rgba,
}

impl EmitterSpec {
    pub fn validate(&self) -> Result<(), String> {
        if !(self.rate > 0.0 && self.rate <= 2000.0) {
            return Err("rate must be in (0, 2000]".into());
        }
        if self.lifetime.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
            return Err("lifetime must be > 0".into());
        }
        if !(0.0..=1.0).contains(&self.lifetime_jitter) || !(0.0..=1.0).contains(&self.speed_jitter) {
            return Err("jitter values must be in 0..1".into());
        }
        Ok(())
    }

    /// All live particles at `time` seconds, emitter at `origin`.
    pub fn particles(&self, time: f32, origin: Vec2) -> Vec<Particle> {
        let t = time - self.start;
        if t < 0.0 {
            return Vec::new();
        }
        let max_life = self.lifetime * (1.0 + self.lifetime_jitter);
        let last = {
            let n = (t * self.rate).floor() as i64;
            match self.duration {
                Some(d) => n.min((d * self.rate).ceil() as i64 - 1),
                None => n,
            }
        };
        let first = (((t - max_life) * self.rate).floor() as i64).max(0);
        let mut out = Vec::new();
        for i in first..=last {
            if out.len() >= self.max_particles {
                break;
            }
            let id = i as i32;
            let h = |k: u32| hash01(id, self.seed.wrapping_add(k.wrapping_mul(0x9E37_79B9)));
            let birth = i as f32 / self.rate;
            let age = t - birth;
            let life = self.lifetime * (1.0 + self.lifetime_jitter * (h(1) * 2.0 - 1.0));
            if age < 0.0 || age >= life {
                continue;
            }
            let ang = (self.direction + self.spread * (h(2) - 0.5)).to_radians();
            let speed = self.speed * (1.0 + self.speed_jitter * (h(3) * 2.0 - 1.0));
            let vel = Vec2::from_angle(ang) * speed;
            let pos = origin + vel * age + self.gravity * (0.5 * age * age);
            let u = age / life;
            out.push(Particle {
                pos,
                life: u,
                size: self.size_start + (self.size_end - self.size_start) * u,
                color: self.color_start.lerp(self.color_end, u),
            });
        }
        out
    }
}

/// Particles as small filled circles (one path each).
pub fn particle_paths(ps: &[Particle]) -> Vec<VPath> {
    ps.iter()
        .enumerate()
        .filter(|(_, p)| p.size > 0.01 && p.color.a > 0.002)
        .map(|(i, p)| {
            let mut c = VPath::circle(&format!("p{i}"), p.pos, p.size);
            c.fill = Some(Paint::Solid(p.color));
            c
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x: f32, y: f32, s: f32) -> Vec<Vec2> {
        vec![Vec2::new(x, y), Vec2::new(x + s, y), Vec2::new(x + s, y + s), Vec2::new(x, y + s)]
    }
    fn area(p: &[Vec2]) -> f32 {
        let mut a = 0.0;
        for i in 0..p.len() {
            let (u, v) = (p[i], p[(i + 1) % p.len()]);
            a += u.x * v.y - v.x * u.y;
        }
        a * 0.5
    }

    #[test]
    fn boolean_areas() {
        let a = vec![square(0.0, 0.0, 10.0)];
        let b = vec![square(5.0, 5.0, 10.0)];
        let ar = |op| boolean(&a, &b, op).iter().map(|p| area(p).abs()).sum::<f32>();
        assert!((ar(BoolOp::Union) - 175.0).abs() < 1e-2);
        assert!((ar(BoolOp::Intersect) - 25.0).abs() < 1e-2);
        assert!((ar(BoolOp::Difference) - 75.0).abs() < 1e-2);
        assert!((ar(BoolOp::Xor) - 150.0).abs() < 1e-2);
    }

    #[test]
    fn difference_with_hole_is_bridged_into_one_polygon() {
        let a = vec![square(0.0, 0.0, 30.0)];
        let b = vec![square(10.0, 10.0, 10.0)];
        let r = boolean(&a, &b, BoolOp::Difference);
        assert_eq!(r.len(), 1);
        // signed (winding) area of the bridged polygon = outer − hole
        assert!((area(&r[0]).abs() - 800.0).abs() < 1e-1, "{}", area(&r[0]));
    }

    #[test]
    fn single_blob_contour_is_a_circle_of_radius_r() {
        // field r²/d² = 1 at d = r
        let cs = metaball_contours(&[(Vec2::new(50.0, 50.0), 20.0)], 1.0, 2.0);
        assert_eq!(cs.len(), 1);
        for p in &cs[0] {
            assert!((p.distance(Vec2::new(50.0, 50.0)) - 20.0).abs() < 0.6, "{p:?}");
        }
    }

    #[test]
    fn blobs_merge_when_close_and_split_when_far() {
        let n = |d: f32| metaball_contours(&[(Vec2::new(50.0, 50.0), 15.0), (Vec2::new(50.0 + d, 50.0), 15.0)], 1.0, 2.0).len();
        assert_eq!(n(20.0), 1, "overlapping blobs fuse into one liquid body");
        assert_eq!(n(80.0), 2, "distant blobs are separate");
    }

    #[test]
    fn liquid_bridge_exists_between_near_blobs() {
        // gap between the circles (distance 36 > 2r = 30) is still filled by the field
        let cs = metaball_contours(&[(Vec2::new(0.0, 0.0), 15.0), (Vec2::new(36.0, 0.0), 15.0)], 1.0, 1.0);
        assert_eq!(cs.len(), 1, "metaball neck forms before the circles touch");
    }

    #[test]
    fn particles_are_pure_function_of_time() {
        let e = EmitterSpec { gravity: Vec2::new(0.0, 100.0), lifetime_jitter: 0.3, speed_jitter: 0.3, ..Default::default() };
        let a = e.particles(2.37, Vec2::ZERO);
        let _ = e.particles(0.5, Vec2::ZERO);
        let b = e.particles(2.37, Vec2::ZERO);
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }

    #[test]
    fn particle_count_matches_rate_times_lifetime() {
        let e = EmitterSpec { rate: 30.0, lifetime: 1.0, ..Default::default() };
        let n = e.particles(5.0, Vec2::ZERO).len() as f32;
        assert!((n - 30.0).abs() <= 1.0, "{n}");
        assert!(e.particles(-0.1, Vec2::ZERO).is_empty());
    }

    #[test]
    fn emitter_stops_after_duration() {
        let e = EmitterSpec { duration: Some(0.5), lifetime: 0.5, ..Default::default() };
        assert!(!e.particles(0.6, Vec2::ZERO).is_empty());
        assert!(e.particles(1.2, Vec2::ZERO).is_empty());
    }

    #[test]
    fn particles_follow_ballistics() {
        let e = EmitterSpec { rate: 1.0, lifetime: 10.0, speed: 10.0, direction: 0.0, spread: 0.0, gravity: Vec2::new(0.0, 8.0), ..Default::default() };
        let p = e.particles(2.0, Vec2::new(5.0, 5.0));
        let first = p[0]; // born at t=0, age 2
        assert!((first.pos.x - 25.0).abs() < 1e-3 && (first.pos.y - 21.0).abs() < 1e-3, "{:?}", first.pos);
    }

    #[test]
    fn spec_parses_from_minimal_json() {
        let e: EmitterSpec = serde_json::from_str(r#"{"rate": 5}"#).unwrap();
        assert_eq!(e.rate, 5.0);
        assert!(e.validate().is_ok());
    }
}
