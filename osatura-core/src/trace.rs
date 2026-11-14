// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Vector tracing of bitmaps: alpha mask → closed polygons → simplified vector paths.
//!
//! Clean-room design note (no ML, no third-party tracer): marching squares over the alpha
//! channel with linear edge interpolation, segment chaining by shared grid edges, and
//! Ramer–Douglas–Peucker simplification. Holes are detected by containment depth.

use crate::fx::BoolOp;
use crate::math::Vec2;
use crate::path::VPath;
use crate::project::Layer;
use std::collections::HashMap;

/// A traced outline.
#[derive(Clone, Debug)]
pub struct Contour {
    pub pts: Vec<Vec2>,
    /// True when the contour is a hole inside another contour.
    pub hole: bool,
}

/// Extract iso-contours of `alpha` (`w × h`, 0..255) at `threshold` (0..255).
/// Coordinates are in pixels; pixel `(x, y)` has its centre at `(x + 0.5, y + 0.5)`.
pub fn contours(alpha: &[u8], w: usize, h: usize, threshold: u8) -> Vec<Vec<Vec2>> {
    if w == 0 || h == 0 || alpha.len() < w * h {
        return Vec::new();
    }
    // padded grid so every contour closes
    let (gw, gh) = (w + 2, h + 2);
    let val = |i: usize, j: usize| -> f32 {
        if i == 0 || j == 0 || i > w || j > h {
            0.0
        } else {
            alpha[(j - 1) * w + (i - 1)] as f32
        }
    };
    let thr = threshold as f32 - 0.5;
    type Key = (usize, usize, u8); // (i, j, 0 = horizontal edge to i+1, 1 = vertical edge to j+1)
    let mut adj: HashMap<Key, Vec<Key>> = HashMap::new();
    let mut link = |a: Key, b: Key| {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    };
    for j in 0..gh - 1 {
        for i in 0..gw - 1 {
            let v = [val(i, j), val(i + 1, j), val(i + 1, j + 1), val(i, j + 1)];
            let mut case = 0u8;
            for (k, x) in v.iter().enumerate() {
                if *x > thr {
                    case |= 1 << k;
                }
            }
            let top = (i, j, 0u8);
            let right = (i + 1, j, 1u8);
            let bottom = (i, j + 1, 0u8);
            let left = (i, j, 1u8);
            let centre = (v[0] + v[1] + v[2] + v[3]) * 0.25 > thr;
            match case {
                0 | 15 => {}
                1 | 14 => link(left, top),
                2 | 13 => link(top, right),
                3 | 12 => link(left, right),
                4 | 11 => link(right, bottom),
                6 | 9 => link(top, bottom),
                7 | 8 => link(left, bottom),
                5 => {
                    if centre {
                        link(top, right);
                        link(left, bottom);
                    } else {
                        link(left, top);
                        link(right, bottom);
                    }
                }
                10 => {
                    if centre {
                        link(left, top);
                        link(right, bottom);
                    } else {
                        link(top, right);
                        link(left, bottom);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    let point = |k: Key| -> Vec2 {
        let (i, j, o) = k;
        let (a, b, pa, pb) = if o == 0 {
            (val(i, j), val(i + 1, j), (i as f32, j as f32), (i as f32 + 1.0, j as f32))
        } else {
            (val(i, j), val(i, j + 1), (i as f32, j as f32), (i as f32, j as f32 + 1.0))
        };
        let t = if (b - a).abs() < 1e-6 { 0.5 } else { ((thr - a) / (b - a)).clamp(0.0, 1.0) };
        // grid node (i, j) is pixel (i-1, j-1) centre → pixel-space (i - 0.5, j - 0.5)
        Vec2::new(pa.0 + (pb.0 - pa.0) * t - 0.5, pa.1 + (pb.1 - pa.1) * t - 0.5)
    };
    let mut keys: Vec<Key> = adj.keys().copied().collect();
    keys.sort();
    let mut seen: HashMap<Key, bool> = HashMap::new();
    let mut out = Vec::new();
    for start in keys {
        if seen.contains_key(&start) {
            continue;
        }
        let mut poly = Vec::new();
        let (mut prev, mut cur) = (None::<Key>, start);
        loop {
            seen.insert(cur, true);
            poly.push(point(cur));
            let next = adj[&cur].iter().copied().find(|n| Some(*n) != prev && !seen.contains_key(n)).or_else(|| {
                adj[&cur].iter().copied().find(|n| *n == start && prev != Some(*n))
            });
            match next {
                Some(n) if n != start => {
                    prev = Some(cur);
                    cur = n;
                }
                _ => break,
            }
        }
        if poly.len() >= 3 {
            out.push(poly);
        }
    }
    out
}

fn area(p: &[Vec2]) -> f32 {
    let mut s = 0.0;
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        s += a.x * b.y - b.x * a.y;
    }
    s * 0.5
}

fn point_in(p: Vec2, poly: &[Vec2]) -> bool {
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn rdp(pts: &[Vec2], tol: f32, out: &mut Vec<Vec2>) {
    if pts.len() < 3 {
        out.extend_from_slice(&pts[..pts.len().saturating_sub(1)]);
        return;
    }
    let (a, b) = (pts[0], pts[pts.len() - 1]);
    let mut best = (0usize, 0.0f32);
    for (i, &p) in pts.iter().enumerate().skip(1).take(pts.len() - 2) {
        let d = crate::rig::dist_to_segment(p, a, b);
        if d > best.1 {
            best = (i, d);
        }
    }
    if best.1 > tol {
        rdp(&pts[..=best.0], tol, out);
        rdp(&pts[best.0..], tol, out);
    } else {
        out.push(a);
    }
}

/// Ramer–Douglas–Peucker on a closed polygon (split at the two most distant vertices).
pub fn simplify_closed(pts: &[Vec2], tol: f32) -> Vec<Vec2> {
    if pts.len() < 4 || tol <= 0.0 {
        return pts.to_vec();
    }
    let a = 0;
    let b = (0..pts.len()).max_by(|&x, &y| pts[a].distance(pts[x]).total_cmp(&pts[a].distance(pts[y]))).unwrap_or(0);
    let (lo, hi) = (a.min(b), a.max(b));
    let first: Vec<Vec2> = pts[lo..=hi].to_vec();
    let mut second: Vec<Vec2> = pts[hi..].to_vec();
    second.extend_from_slice(&pts[..=lo]);
    let mut out = Vec::new();
    rdp(&first, tol, &mut out);
    rdp(&second, tol, &mut out);
    if out.len() < 3 {
        pts.to_vec()
    } else {
        out
    }
}

/// Trace an alpha mask into classified, simplified contours (largest first). Contours with
/// less than `min_area` pixels² are dropped.
pub fn trace_alpha(alpha: &[u8], w: usize, h: usize, threshold: u8, tolerance: f32, min_area: f32) -> Vec<Contour> {
    let mut polys: Vec<Vec<Vec2>> = contours(alpha, w, h, threshold)
        .into_iter()
        .map(|c| simplify_closed(&c, tolerance))
        .filter(|c| c.len() >= 3 && area(c).abs() >= min_area)
        .collect();
    polys.sort_by(|a, b| area(b).abs().total_cmp(&area(a).abs()));
    let depth: Vec<usize> = (0..polys.len())
        .map(|i| (0..polys.len()).filter(|&j| j != i && area(&polys[j]).abs() > area(&polys[i]).abs() && point_in(polys[i][0], &polys[j])).count())
        .collect();
    polys.into_iter().zip(depth).map(|(pts, d)| Contour { pts, hole: d % 2 == 1 }).collect()
}

/// Decode a PNG and trace its alpha.
pub fn trace_png(png: &[u8], threshold: u8, tolerance: f32, min_area: f32) -> Result<Vec<Contour>, String> {
    let pm = crate::assets::decode_png_checked(png)?;
    let (w, h) = (pm.width() as usize, pm.height() as usize);
    let alpha: Vec<u8> = pm.data().chunks_exact(4).map(|p| p[3]).collect();
    Ok(trace_alpha(&alpha, w, h, threshold, tolerance, min_area))
}

/// Build a vector layer from traced contours: each outer contour (with its holes) becomes a
/// Boolean-difference layer so holes render correctly with a plain fill.
pub fn contours_to_layer(name: &str, contours: &[Contour], fill: &str, offset: Vec2) -> Layer {
    let mv = |c: &Contour, n: String| VPath::polygon(&n, &c.pts.iter().map(|&p| p + offset).collect::<Vec<_>>());
    let mut children = Vec::new();
    for (i, c) in contours.iter().enumerate().filter(|(_, c)| !c.hole) {
        let holes: Vec<&Contour> = contours.iter().filter(|h| h.hole && point_in(h.pts[0], &c.pts)).collect();
        let mut paths = vec![mv(c, format!("outer{i}"))];
        for (k, h) in holes.iter().enumerate() {
            paths.push(mv(h, format!("hole{i}_{k}")));
        }
        if paths.len() == 1 {
            // no holes: a plain filled vector layer (Boolean needs at least two paths)
            let p0 = paths.remove(0).with_fill(fill);
            children.push(Layer::vector(&format!("shape{i}"), vec![p0]));
            continue;
        }
        let mut l = Layer::boolean(&format!("shape{i}"), BoolOp::Difference, paths);
        if let crate::project::LayerKind::Boolean { paths, .. } = &mut l.kind {
            for p in paths.iter_mut() {
                *p = p.clone().with_fill(fill);
            }
        }
        children.push(l);
    }
    Layer::group(name, children)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disc(w: usize, h: usize, cx: f32, cy: f32, r: f32, inner: Option<f32>) -> Vec<u8> {
        let mut a = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                if d <= r && inner.is_none_or(|i| d > i) {
                    a[y * w + x] = 255;
                }
            }
        }
        a
    }

    #[test]
    fn traces_a_disc_with_right_area() {
        let a = disc(40, 40, 20.0, 20.0, 12.0, None);
        let c = trace_alpha(&a, 40, 40, 128, 0.3, 4.0);
        assert_eq!(c.len(), 1);
        assert!(!c[0].hole);
        let ar = area(&c[0].pts).abs();
        let exact = std::f32::consts::PI * 144.0;
        assert!((ar - exact).abs() / exact < 0.06, "area {ar} vs {exact}");
    }

    #[test]
    fn ring_gives_outer_and_hole() {
        let a = disc(60, 60, 30.0, 30.0, 20.0, Some(9.0));
        let c = trace_alpha(&a, 60, 60, 128, 0.3, 4.0);
        assert_eq!(c.len(), 2, "outer + hole");
        assert!(!c[0].hole && c[1].hole);
        assert!(area(&c[0].pts).abs() > area(&c[1].pts).abs());
    }

    #[test]
    fn simplification_reduces_points_within_tolerance() {
        let a = disc(60, 60, 30.0, 30.0, 25.0, None);
        let fine = trace_alpha(&a, 60, 60, 128, 0.0, 4.0);
        let coarse = trace_alpha(&a, 60, 60, 128, 1.0, 4.0);
        assert!(coarse[0].pts.len() * 3 < fine[0].pts.len(), "{} vs {}", coarse[0].pts.len(), fine[0].pts.len());
        let (af, ac) = (area(&fine[0].pts).abs(), area(&coarse[0].pts).abs());
        assert!((af - ac).abs() / af < 0.05);
    }

    #[test]
    fn two_blobs_and_min_area_filter() {
        let mut a = disc(80, 40, 20.0, 20.0, 10.0, None);
        let b = disc(80, 40, 60.0, 20.0, 3.0, None);
        for i in 0..a.len() {
            a[i] = a[i].max(b[i]);
        }
        assert_eq!(trace_alpha(&a, 80, 40, 128, 0.3, 1.0).len(), 2);
        assert_eq!(trace_alpha(&a, 80, 40, 128, 0.3, 100.0).len(), 1, "small blob dropped");
        assert!(trace_alpha(&[], 0, 0, 128, 0.3, 1.0).is_empty());
        assert!(trace_alpha(&[0u8; 100], 10, 10, 128, 0.3, 1.0).is_empty());
    }

    #[test]
    fn ring_becomes_layer_that_validates_and_renders_a_hole() {
        let a = disc(60, 60, 30.0, 30.0, 20.0, Some(9.0));
        let c = trace_alpha(&a, 60, 60, 128, 0.3, 4.0);
        let layer = contours_to_layer("ring", &c, "#ff0000", Vec2::ZERO);
        let mut p = crate::project::Project::new("t", 60.0, 60.0, 24.0, 1);
        p.layers = vec![layer];
        let eng = crate::Engine::new(p).expect("valid");
        let svg = crate::svg::render_frame(&eng, 0.0, &Default::default());
        assert!(svg.contains("<path"), "has geometry");
    }
}
