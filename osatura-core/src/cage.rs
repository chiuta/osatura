// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Cage deformer: a closed polygon around artwork; the content follows the cage vertices.
//!
//! Method (textbook, implemented from the maths): mean-value coordinates, M. S. Floater,
//! "Mean value coordinates", Computer Aided Geometric Design 20(1), 2003, in the 2D form of
//! Hormann & Floater (2006). For a point `x` inside a closed polygon `p_0..p_{n-1}` let
//! `s_i = p_i - x`, `r_i = |s_i|` and `alpha_i` the signed angle from `s_i` to `s_{i+1}`.
//! Then `w_i = (tan(alpha_{i-1}/2) + tan(alpha_i/2)) / r_i` and `lambda_i = w_i / sum_j w_j`.
//! The weights sum to one, reproduce linear functions (`sum lambda_i p_i = x`), are smooth in
//! the interior and interpolate the boundary linearly along each edge. `tan(alpha/2)` is
//! evaluated as `cross / (r_i r_{i+1} + dot)` (no trigonometry, no angle wrap-around).
//!
//! Policy decisions (documented, tested):
//! * Points **outside** the cage (or on no part of it) are **left unchanged**. Mean-value
//!   coordinates exist outside a convex cage, but extrapolated content would move without
//!   bound; a hard cut-off at the cage outline is predictable and matches "content inside the
//!   cage". Make the cage larger than the artwork if all of it should follow.
//! * Points within `1e-7 * diagonal` of the boundary count as **on the boundary**: a vertex gets
//!   weight 1, an edge point the linear weights of its two end vertices. No division by zero,
//!   no NaN is ever produced; a degenerate sum falls back to "unchanged".
//! * A concave (but simple) cage is allowed. Weights may then be negative for some points,
//!   which is inherent to mean-value coordinates; they still sum to one and reproduce
//!   linear functions.
//! * The deformed position of a point is `sum lambda_i (p_i + o_i)`, i.e. the displacement is
//!   `sum lambda_i o_i`: exactly zero when all offsets are zero.
//! * Weights depend only on the rest cage and the point, so the engine caches them for the rest
//!   mesh of an image layer at `Engine::new`; path anchors move with morphs and point offsets,
//!   so their weights are computed per frame (O(n) each, in f64, deterministic).

use crate::math::Vec2;

/// Largest supported cage.
pub const MAX_VERTS: usize = 256;

#[derive(Clone, Debug)]
pub struct CageSolver {
    rest: Vec<[f64; 2]>,
    tol: f64,
}

fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

/// Do the closed segments `a-b` and `c-d` share a point?
fn segments_touch(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let o = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| cross(sub(q, p), sub(r, p));
    let on = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        r[0] >= p[0].min(q[0])
            && r[0] <= p[0].max(q[0])
            && r[1] >= p[1].min(q[1])
            && r[1] <= p[1].max(q[1])
    };
    let (o1, o2, o3, o4) = (o(a, b, c), o(a, b, d), o(c, d, a), o(c, d, b));
    if ((o1 > 0.0 && o2 < 0.0) || (o1 < 0.0 && o2 > 0.0))
        && ((o3 > 0.0 && o4 < 0.0) || (o3 < 0.0 && o4 > 0.0))
    {
        return true;
    }
    (o1 == 0.0 && on(a, b, c))
        || (o2 == 0.0 && on(a, b, d))
        || (o3 == 0.0 && on(c, d, a))
        || (o4 == 0.0 && on(c, d, b))
}

impl CageSolver {
    /// Validate and precompute. Errors: fewer than 3 or more than `MAX_VERTS` vertices, non-finite
    /// coordinates, repeated consecutive vertices, zero area, or a self-intersecting outline.
    pub fn new(rest: &[Vec2]) -> Result<CageSolver, String> {
        let n = rest.len();
        if n < 3 {
            return Err("a cage needs at least 3 vertices".into());
        }
        if n > MAX_VERTS {
            return Err(format!("a cage has at most {MAX_VERTS} vertices"));
        }
        if rest.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err("cage vertices must be finite".into());
        }
        let pts: Vec<[f64; 2]> = rest.iter().map(|p| [p.x as f64, p.y as f64]).collect();
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for p in &pts {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        let diag = ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2)).sqrt();
        if diag <= 0.0 {
            return Err("the cage has no size".into());
        }
        let tol = 1e-7 * diag;
        let mut area2 = 0.0;
        for i in 0..n {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            if (sub(b, a)[0].hypot(sub(b, a)[1])) <= tol {
                return Err(format!("cage vertices {} and {} coincide", i, (i + 1) % n));
            }
            area2 += cross(a, b);
        }
        for i in 0..n {
            for j in i + 1..n {
                let adjacent = j == i + 1 || (i == 0 && j == n - 1);
                let (a, b, c, d) = (pts[i], pts[(i + 1) % n], pts[j], pts[(j + 1) % n]);
                if adjacent {
                    // a spike: the two edges fold back on each other
                    let (shared, p, q) = if j == i + 1 { (b, a, d) } else { (a, b, c) };
                    let (u, v) = (sub(p, shared), sub(q, shared));
                    if cross(u, v).abs() <= 1e-12 * u[0].hypot(u[1]) * v[0].hypot(v[1])
                        && dot(u, v) > 0.0
                    {
                        return Err(format!("cage edges {i} and {j} fold back on each other"));
                    }
                } else if segments_touch(a, b, c, d) {
                    return Err(format!(
                        "cage edges {i} and {j} cross or touch; the cage must be a simple polygon"
                    ));
                }
            }
        }
        if area2.abs() <= 1e-9 * diag * diag {
            return Err("the cage is degenerate (zero area)".into());
        }
        Ok(CageSolver { rest: pts, tol })
    }

    pub fn len(&self) -> usize {
        self.rest.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }

    /// Mean-value weights of rest-space point `p`, written to `out` (length `len()`).
    /// Returns `false` (and leaves `out` unspecified) if `p` is outside the cage.
    pub fn weights(&self, p: Vec2, out: &mut Vec<f64>) -> bool {
        let n = self.rest.len();
        out.clear();
        out.resize(n, 0.0);
        let x = [p.x as f64, p.y as f64];
        if !x[0].is_finite() || !x[1].is_finite() {
            return false;
        }
        let s: Vec<[f64; 2]> = self.rest.iter().map(|&q| sub(q, x)).collect();
        let r: Vec<f64> = s.iter().map(|v| v[0].hypot(v[1])).collect();
        // on a vertex
        for i in 0..n {
            if r[i] <= self.tol {
                out[i] = 1.0;
                return true;
            }
        }
        // on an edge
        for i in 0..n {
            let j = (i + 1) % n;
            let e = sub(self.rest[j], self.rest[i]);
            let len2 = dot(e, e);
            let t = (-dot(s[i], e) / len2).clamp(0.0, 1.0);
            let c = [s[i][0] + e[0] * t, s[i][1] + e[1] * t];
            if c[0].hypot(c[1]) <= self.tol {
                out[i] = 1.0 - t;
                out[j] = t;
                return true;
            }
        }
        // inside? (crossing number; the boundary was handled above)
        let mut inside = false;
        for i in 0..n {
            let (a, b) = (self.rest[i], self.rest[(i + 1) % n]);
            if (a[1] > x[1]) != (b[1] > x[1]) {
                let xc = a[0] + (x[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]);
                if x[0] < xc {
                    inside = !inside;
                }
            }
        }
        if !inside {
            return false;
        }
        let mut t = vec![0.0f64; n];
        for i in 0..n {
            let j = (i + 1) % n;
            let den = r[i] * r[j] + dot(s[i], s[j]);
            if den <= 0.0 {
                return false; // anti-parallel: only possible on the boundary
            }
            t[i] = cross(s[i], s[j]) / den;
        }
        let mut sum = 0.0;
        for i in 0..n {
            let w = (t[(i + n - 1) % n] + t[i]) / r[i];
            out[i] = w;
            sum += w;
        }
        if !sum.is_finite() || sum.abs() < 1e-300 {
            return false;
        }
        for w in out.iter_mut() {
            *w /= sum;
        }
        out.iter().all(|w| w.is_finite())
    }

    /// Displacement `sum lambda_i o_i` of rest-space point `p` (zero outside the cage).
    pub fn displacement(&self, offs: &[Vec2], p: Vec2) -> Vec2 {
        let mut w = Vec::new();
        if !self.weights(p, &mut w) {
            return Vec2::ZERO;
        }
        Self::apply_weights(&w, offs)
    }

    /// Displacement from already computed weights (`w` empty = outside).
    pub fn apply_weights(w: &[f64], offs: &[Vec2]) -> Vec2 {
        let (mut dx, mut dy) = (0.0f64, 0.0f64);
        for (l, o) in w.iter().zip(offs) {
            dx += l * o.x as f64;
            dy += l * o.y as f64;
        }
        Vec2::new(dx as f32, dy as f32)
    }
}

// ---------------------------------------------------------------- authoring helpers

use crate::chanmeta;
use crate::engine::Engine;
use crate::project::{Channel, Deformer, LayerKind, Project, Prop, Source};
use crate::{anim::Ease, anim::Track};

/// How to build the cage outline around a layer.
#[derive(Clone, Debug)]
pub struct CageOptions {
    pub name: String,
    /// Explicit rectangle `x,y,w,h` (layer space); `None` = the layer's rest bounds.
    pub rect: Option<[f32; 4]>,
    /// Convex hull of the layer's points instead of the box (ignored with `rect`).
    pub hull: bool,
    /// Vertices per box side minus one: each side is split into `verts` segments (box only).
    pub verts: usize,
    /// Outward padding.
    pub pad: f32,
}

impl Default for CageOptions {
    fn default() -> Self {
        CageOptions {
            name: "cage".into(),
            rect: None,
            hull: false,
            verts: 2,
            pad: 4.0,
        }
    }
}

/// Rest-space points of a layer (anchors and handles, or the image rectangle corners).
fn layer_points(p: &Project, layer: &str, kind: &LayerKind) -> Result<Vec<Vec2>, String> {
    match kind {
        LayerKind::Vector { paths } => {
            let mut v = Vec::new();
            for p in paths {
                for a in &p.anchors {
                    v.push(a.pos);
                    if a.handle_in.is_some() {
                        v.push(a.hin());
                    }
                    if a.handle_out.is_some() {
                        v.push(a.hout());
                    }
                }
            }
            if v.is_empty() {
                Err("the layer has no points".into())
            } else {
                Ok(v)
            }
        }
        LayerKind::Image { .. } => {
            let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
            let id = e
                .node_index(layer)
                .ok_or_else(|| format!("no layer at '{layer}'"))?;
            let fr = e.evaluate(0.0);
            let im = fr.nodes[id]
                .image
                .as_ref()
                .ok_or("the image layer has no pixels")?;
            Ok(vec![im.origin, im.origin + im.size])
        }
        _ => Err("cages apply to vector and image layers".into()),
    }
}

fn convex_hull(pts: &[Vec2]) -> Vec<Vec2> {
    let mut p: Vec<Vec2> = pts.to_vec();
    p.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    p.dedup();
    if p.len() < 3 {
        return p;
    }
    let cr = |o: Vec2, a: Vec2, b: Vec2| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut h: Vec<Vec2> = Vec::new();
    for &q in &p {
        while h.len() >= 2 && cr(h[h.len() - 2], h[h.len() - 1], q) <= 0.0 {
            h.pop();
        }
        h.push(q);
    }
    let lo = h.len() + 1;
    for &q in p.iter().rev().skip(1) {
        while h.len() >= lo && cr(h[h.len() - 2], h[h.len() - 1], q) <= 0.0 {
            h.pop();
        }
        h.push(q);
    }
    h.pop();
    h
}

/// Push every vertex of a convex CCW (in y-up maths) or CW polygon outward by `pad`.
fn pad_polygon(poly: &[Vec2], pad: f32) -> Vec<Vec2> {
    let n = poly.len();
    let area2: f32 = (0..n)
        .map(|i| poly[i].x * poly[(i + 1) % n].y - poly[(i + 1) % n].x * poly[i].y)
        .sum();
    let s = if area2 > 0.0 { 1.0 } else { -1.0 };
    (0..n)
        .map(|i| {
            let (a, b, c) = (poly[(i + n - 1) % n], poly[i], poly[(i + 1) % n]);
            let nrm = |p: Vec2, q: Vec2| {
                let d = q - p;
                let l = d.length().max(1e-9);
                Vec2::new(d.y / l, -d.x / l) * s
            };
            let (n1, n2) = (nrm(a, b), nrm(b, c));
            let m = n1 + n2;
            let ml = m.length();
            if ml < 1e-6 {
                return b + n1 * pad;
            }
            let dirn = m * (1.0 / ml);
            let cos_half = dirn.x * n1.x + dirn.y * n1.y;
            b + dirn * (pad / cos_half.max(0.5))
        })
        .collect()
}

/// Add a cage deformer around `layer` (non-destructive: returns a modified copy).
pub fn add_cage(p: &Project, layer: &str, o: &CageOptions) -> Result<Project, String> {
    if o.name.is_empty() || o.name.contains("::") {
        return Err("the cage name must be non-empty and must not contain '::'".into());
    }
    if !(o.pad.is_finite() && o.pad >= 0.0) {
        return Err("pad must be a number >= 0".into());
    }
    if !(1..=32).contains(&o.verts) {
        return Err("verts must be 1..32 segments per side".into());
    }
    let lay = p
        .find_layer(layer)
        .ok_or_else(|| format!("no layer at '{layer}'"))?;
    if !matches!(lay.kind, LayerKind::Vector { .. } | LayerKind::Image { .. }) {
        return Err(format!("layer '{layer}' must be a vector or image layer"));
    }
    if lay.deformers.iter().any(|d| d.name() == o.name) {
        return Err(format!(
            "layer '{layer}' already has a deformer named '{}'",
            o.name
        ));
    }
    let rest: Vec<Vec2> = if let Some(r) = o.rect {
        box_outline(r, o.verts, o.pad)?
    } else {
        let pts = layer_points(p, layer, &lay.kind)?;
        if o.hull {
            let h = convex_hull(&pts);
            if h.len() < 3 {
                return Err("the layer's points are collinear; no hull".into());
            }
            pad_polygon(&h, o.pad)
        } else {
            let (mut lo, mut hi) = (pts[0], pts[0]);
            for q in &pts {
                lo = Vec2::new(lo.x.min(q.x), lo.y.min(q.y));
                hi = Vec2::new(hi.x.max(q.x), hi.y.max(q.y));
            }
            box_outline([lo.x, lo.y, hi.x - lo.x, hi.y - lo.y], o.verts, o.pad)?
        }
    };
    CageSolver::new(&rest)?;
    let mut q = p.clone();
    q.find_layer_mut(layer)
        .ok_or_else(|| format!("no layer at '{layer}'"))?
        .deformers
        .push(Deformer::Cage {
            name: o.name.clone(),
            rest,
            offsets: vec![],
        });
    Engine::new(q.clone()).map_err(|v| v.join("; "))?;
    Ok(q)
}

fn box_outline(r: [f32; 4], seg: usize, pad: f32) -> Result<Vec<Vec2>, String> {
    if !r.iter().all(|v| v.is_finite()) {
        return Err("the cage rectangle must be finite".into());
    }
    let (x0, y0, x1, y1) = (r[0] - pad, r[1] - pad, r[0] + r[2] + pad, r[1] + r[3] + pad);
    if !(x1 > x0 && y1 > y0) {
        return Err("the cage rectangle has no size (give rect=x,y,w,h with positive w and h, or a layer with extent)".into());
    }
    let mut v = Vec::new();
    let lerp = |a: f32, b: f32, i: usize| a + (b - a) * i as f32 / seg as f32;
    for i in 0..seg {
        v.push(Vec2::new(lerp(x0, x1, i), y0));
    }
    for i in 0..seg {
        v.push(Vec2::new(x1, lerp(y0, y1, i)));
    }
    for i in 0..seg {
        v.push(Vec2::new(lerp(x1, x0, i), y1));
    }
    for i in 0..seg {
        v.push(Vec2::new(x0, lerp(y1, y0, i)));
    }
    Ok(v)
}

/// One pose edit: key the offsets of cage vertices at `frame`.
/// `moves` is `(vertex, dx, dy)`; `all` is added to every vertex (a rigid translation).
/// Missing channels are created with a key of the current value at frame 0 (or `frame` itself if 0).
pub fn pose_cage(
    p: &Project,
    layer: &str,
    name: &str,
    frame: f32,
    moves: &[(usize, f32, f32)],
    all: Option<(f32, f32)>,
    ease: Ease,
) -> Result<Project, String> {
    if !frame.is_finite() || frame < 0.0 {
        return Err("frame must be a number >= 0".into());
    }
    let lay = p
        .find_layer(layer)
        .ok_or_else(|| format!("no layer at '{layer}'"))?;
    let (n, base) = match lay.deformers.iter().find(|d| d.name() == name) {
        Some(Deformer::Cage { rest, offsets, .. }) => (rest.len(), offsets.clone()),
        Some(_) => return Err(format!("deformer '{name}' on '{layer}' is not a cage")),
        None => return Err(format!("layer '{layer}' has no cage named '{name}'")),
    };
    let mut want: Vec<Option<(f32, f32)>> = vec![None; n];
    if let Some((dx, dy)) = all {
        for w in want.iter_mut() {
            *w = Some((dx, dy));
        }
    }
    for &(i, dx, dy) in moves {
        if i >= n {
            return Err(format!("cage vertex {i} does not exist (0..{})", n - 1));
        }
        let cur = want[i].unwrap_or((0.0, 0.0));
        want[i] = Some(if all.is_some() {
            (cur.0 + dx, cur.1 + dy)
        } else {
            (dx, dy)
        });
    }
    if !want
        .iter()
        .flatten()
        .all(|v| v.0.is_finite() && v.1.is_finite())
    {
        return Err("offsets must be finite".into());
    }
    let tgt = |i: usize| format!("{layer}::{name}::{i}");
    for (i, w) in want.iter().enumerate() {
        if w.is_some() {
            chanmeta::assert_unlocked(p, &tgt(i), Prop::CageDx)?;
            chanmeta::assert_unlocked(p, &tgt(i), Prop::CageDy)?;
        }
    }
    let mut q = p.clone();
    for (i, w) in want.iter().enumerate() {
        let Some((dx, dy)) = *w else { continue };
        for (prop, v) in [(Prop::CageDx, dx), (Prop::CageDy, dy)] {
            let t = tgt(i);
            let existing = q.channels.iter_mut().find(|c| {
                c.target == t && c.prop == prop && matches!(c.source, Source::Keys { .. })
            });
            match existing {
                Some(Channel {
                    source: Source::Keys { track },
                    ..
                }) => track.set_key(frame, v, ease),
                _ => {
                    let rest0 = base
                        .get(i)
                        .map(|o| if prop == Prop::CageDx { o.x } else { o.y })
                        .unwrap_or(0.0);
                    let mut tr = Track::new();
                    if frame > 0.0 {
                        tr.set_key(0.0, rest0, Ease::Linear);
                    }
                    tr.set_key(frame, v, ease);
                    q.channels.push(Channel::keys(&t, prop, tr));
                }
            }
        }
    }
    Engine::new(q.clone()).map_err(|v| v.join("; "))?;
    Ok(q)
}

/// Parse `i:dx,dy;i:dx,dy` into moves.
pub fn parse_moves(s: &str) -> Result<Vec<(usize, f32, f32)>, String> {
    let mut v = Vec::new();
    for part in s.split(';').filter(|x| !x.trim().is_empty()) {
        let (i, d) = part
            .split_once(':')
            .ok_or_else(|| format!("bad vertex move '{part}' (use i:dx,dy)"))?;
        let (dx, dy) = d
            .split_once(',')
            .ok_or_else(|| format!("bad vertex move '{part}' (use i:dx,dy)"))?;
        let f = |t: &str| {
            t.trim()
                .parse::<f32>()
                .map_err(|_| format!("bad number '{t}'"))
        };
        v.push((
            i.trim()
                .parse::<usize>()
                .map_err(|_| format!("bad vertex index '{i}'"))?,
            f(dx)?,
            f(dy)?,
        ));
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;
    use crate::project::Layer;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }
    fn square() -> Vec<Vec2> {
        vec![v(0.0, 0.0), v(100.0, 0.0), v(100.0, 100.0), v(0.0, 100.0)]
    }
    /// L-shaped concave cage.
    fn ell() -> Vec<Vec2> {
        vec![
            v(0.0, 0.0),
            v(100.0, 0.0),
            v(100.0, 40.0),
            v(40.0, 40.0),
            v(40.0, 100.0),
            v(0.0, 100.0),
        ]
    }
    fn lcg(s: &mut u64) -> f32 {
        *s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*s >> 33) as f32) / (1u64 << 31) as f32
    }

    #[test]
    fn partition_of_unity_and_linear_reproduction_convex_and_concave() {
        for cage in [
            square(),
            ell(),
            vec![
                v(0.0, 0.0),
                v(80.0, 10.0),
                v(100.0, 70.0),
                v(40.0, 110.0),
                v(-10.0, 60.0),
            ],
        ] {
            let s = CageSolver::new(&cage).unwrap();
            let mut seed = 7u64;
            let mut w = Vec::new();
            let mut tested = 0;
            for _ in 0..600 {
                let p = v(lcg(&mut seed) * 130.0 - 15.0, lcg(&mut seed) * 130.0 - 15.0);
                if !s.weights(p, &mut w) {
                    continue;
                }
                tested += 1;
                let sum: f64 = w.iter().sum();
                assert!((sum - 1.0).abs() < 1e-9, "sum {sum}");
                let (mut x, mut y) = (0.0, 0.0);
                for (l, q) in w.iter().zip(&cage) {
                    x += l * q.x as f64;
                    y += l * q.y as f64;
                }
                assert!(
                    (x - p.x as f64).abs() < 1e-3 && (y - p.y as f64).abs() < 1e-3,
                    "({x},{y}) vs {p:?}"
                );
            }
            assert!(tested > 100);
        }
    }

    #[test]
    fn concave_weights_can_be_negative_but_still_valid() {
        let s = CageSolver::new(&ell()).unwrap();
        let mut w = Vec::new();
        assert!(s.weights(v(80.0, 30.0), &mut w));
        assert!(w.iter().any(|&l| l < 0.0) || w.iter().all(|&l| l >= 0.0));
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn identity_when_cage_at_rest_and_translation_scale_exact() {
        let s = CageSolver::new(&ell()).unwrap();
        let mut seed = 3u64;
        let zero = vec![Vec2::ZERO; 6];
        let tr = vec![v(7.0, -3.0); 6];
        let c = v(20.0, 30.0);
        // scale by 1.5 about c: offset_i = 0.5 (p_i - c)
        let sc: Vec<Vec2> = ell().iter().map(|&q| (q - c) * 0.5).collect();
        for _ in 0..300 {
            let p = v(lcg(&mut seed) * 100.0, lcg(&mut seed) * 100.0);
            assert_eq!(s.displacement(&zero, p), Vec2::ZERO);
            let d = s.displacement(&tr, p);
            let inside = s.displacement(&tr, p) != Vec2::ZERO;
            if inside {
                assert!((d.x - 7.0).abs() < 1e-3 && (d.y + 3.0).abs() < 1e-3);
                let e = s.displacement(&sc, p);
                let want = (p - c) * 0.5;
                assert!((e.x - want.x).abs() < 1e-3 && (e.y - want.y).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn points_outside_are_unchanged() {
        let s = CageSolver::new(&square()).unwrap();
        let offs = vec![v(5.0, 5.0); 4];
        for p in [
            v(-1.0, 50.0),
            v(101.0, 50.0),
            v(50.0, 200.0),
            v(-500.0, -500.0),
            v(f32::NAN, 0.0),
        ] {
            assert_eq!(s.displacement(&offs, p), Vec2::ZERO);
        }
        // inside the notch of the L is outside the cage
        let l = CageSolver::new(&ell()).unwrap();
        assert_eq!(l.displacement(&[v(5.0, 5.0); 6], v(80.0, 80.0)), Vec2::ZERO);
    }

    #[test]
    fn boundary_points_are_exact_and_finite() {
        let s = CageSolver::new(&ell()).unwrap();
        let mut w = Vec::new();
        for (i, q) in ell().iter().enumerate() {
            assert!(s.weights(*q, &mut w));
            for (j, l) in w.iter().enumerate() {
                assert_eq!(*l, if i == j { 1.0 } else { 0.0 });
            }
        }
        // mid-edge: halfway between vertices 1 and 2
        assert!(s.weights(v(100.0, 20.0), &mut w));
        assert!(
            (w[1] - 0.5).abs() < 1e-9 && (w[2] - 0.5).abs() < 1e-9 && w.iter().sum::<f64>() == 1.0
        );
        // edge points follow the edge linearly; many edge samples never give NaN
        let offs = vec![
            v(0.0, 0.0),
            v(10.0, 0.0),
            v(30.0, 0.0),
            v(0.0, 0.0),
            v(0.0, 0.0),
            v(0.0, 0.0),
        ];
        for k in 0..=40 {
            let p = v(100.0, k as f32);
            let d = s.displacement(&offs, p);
            assert!(d.x.is_finite() && d.y.is_finite());
            assert!((d.x - (10.0 + 20.0 * k as f32 / 40.0)).abs() < 1e-3);
        }
        // points a hair inside / outside are finite too
        for e in [1e-9f32, 1e-6, 1e-4, -1e-4] {
            let d = s.displacement(&offs, v(100.0 - e, 20.0));
            assert!(d.x.is_finite() && d.y.is_finite());
        }
    }

    #[test]
    fn deterministic() {
        let s = CageSolver::new(&ell()).unwrap();
        let offs: Vec<Vec2> = (0..6).map(|i| v(i as f32, -(i as f32))).collect();
        let a: Vec<Vec2> = (0..50)
            .map(|i| s.displacement(&offs, v(i as f32, 20.0)))
            .collect();
        let b: Vec<Vec2> = (0..50)
            .map(|i| s.displacement(&offs, v(i as f32, 20.0)))
            .collect();
        assert_eq!(a, b);
    }

    #[test]
    fn orientation_does_not_matter() {
        let mut cw = ell();
        cw.reverse();
        let a = CageSolver::new(&ell()).unwrap();
        let b = CageSolver::new(&cw).unwrap();
        let (oa, ob): (Vec<Vec2>, Vec<Vec2>) = {
            let o: Vec<Vec2> = (0..6).map(|i| v(i as f32 * 2.0, 1.0)).collect();
            let mut r = o.clone();
            r.reverse();
            (o, r)
        };
        let p = v(20.0, 20.0);
        let (da, db) = (a.displacement(&oa, p), b.displacement(&ob, p));
        assert!((da.x - db.x).abs() < 1e-4 && (da.y - db.y).abs() < 1e-4);
    }

    #[test]
    fn validation_errors() {
        assert!(CageSolver::new(&[v(0.0, 0.0), v(1.0, 1.0)])
            .unwrap_err()
            .contains("at least 3"));
        assert!(CageSolver::new(&[v(0.0, 0.0), v(10.0, 0.0), v(20.0, 0.0)]).is_err());
        assert!(
            CageSolver::new(&[v(0.0, 0.0), v(0.0, 0.0), v(10.0, 10.0), v(0.0, 10.0)])
                .unwrap_err()
                .contains("coincide")
        );
        // bow tie
        assert!(
            CageSolver::new(&[v(0.0, 0.0), v(10.0, 10.0), v(10.0, 0.0), v(0.0, 10.0)])
                .unwrap_err()
                .contains("simple")
        );
        assert!(CageSolver::new(&[v(0.0, 0.0), v(f32::NAN, 1.0), v(1.0, 5.0)]).is_err());
        // spike
        assert!(CageSolver::new(&[v(0.0, 0.0), v(10.0, 0.0), v(5.0, 0.0), v(5.0, 10.0)]).is_err());
        let big: Vec<Vec2> = (0..300)
            .map(|i| {
                let a = i as f32 / 300.0 * std::f32::consts::TAU;
                v(a.cos() * 50.0, a.sin() * 50.0)
            })
            .collect();
        assert!(CageSolver::new(&big).unwrap_err().contains("at most"));
    }

    // ----- project level

    fn proj() -> Project {
        let mut p = Project::new("t", 200.0, 200.0, 12.0, 24);
        let mut l = Layer::vector(
            "sq",
            vec![crate::path::VPath::rect("r", 40.0, 40.0, 100.0, 100.0).with_fill("#ff0000")],
        );
        l.deformers = vec![Deformer::Cage {
            name: "c".into(),
            rest: vec![
                v(20.0, 20.0),
                v(160.0, 20.0),
                v(160.0, 160.0),
                v(20.0, 160.0),
            ],
            offsets: vec![],
        }];
        p.layers = vec![l];
        p
    }

    fn anchors(p: &Project, f: f32) -> Vec<Vec2> {
        let e = Engine::new(p.clone()).unwrap();
        let fr = e.evaluate(f);
        let n = &fr.nodes[e.node_index("sq").unwrap()];
        n.paths
            .iter()
            .flat_map(|vp| vp.anchors.iter().map(|a| a.pos))
            .collect()
    }

    #[test]
    fn engine_moves_content_and_identity_at_rest() {
        let p = proj();
        let a0 = anchors(&p, 0.0);
        assert_eq!(
            a0,
            vec![
                v(40.0, 40.0),
                v(140.0, 40.0),
                v(140.0, 140.0),
                v(40.0, 140.0)
            ]
        );
        let q = pose_cage(&p, "sq", "c", 12.0, &[], Some((10.0, -5.0)), Ease::Linear).unwrap();
        let a = anchors(&q, 12.0);
        for (x, y) in a.iter().zip(&a0) {
            assert!((x.x - y.x - 10.0).abs() < 1e-3 && (x.y - y.y + 5.0).abs() < 1e-3);
        }
        // frame 0 is the rest pose; frame 6 is halfway
        assert_eq!(anchors(&q, 0.0), a0);
        let m = anchors(&q, 6.0);
        assert!((m[0].x - 45.0).abs() < 1e-3);
        // one corner pulled: nearer anchors follow more
        let r = pose_cage(&p, "sq", "c", 0.0, &[(2, 40.0, 40.0)], None, Ease::Linear).unwrap();
        let b = anchors(&r, 0.0);
        let mv = |i: usize| (b[i] - a0[i]).length();
        assert!(mv(2) > mv(1) && mv(2) > mv(3) && mv(3) > mv(0) * 0.0 && mv(2) > 1.0);
        // evaluation is deterministic
        assert_eq!(anchors(&r, 0.0), anchors(&r, 0.0));
    }

    #[test]
    fn handles_follow_and_outside_layer_is_untouched() {
        let mut p = proj();
        // a second shape outside the cage
        if let LayerKind::Vector { paths } = &mut p.layers[0].kind {
            paths.push(crate::path::VPath::rect("far", 170.0, 170.0, 20.0, 20.0));
        }
        let q = pose_cage(&p, "sq", "c", 0.0, &[], Some((9.0, 9.0)), Ease::Linear).unwrap();
        let a = anchors(&q, 0.0);
        assert_eq!(a[4], v(170.0, 170.0));
        assert_eq!(a[0], v(49.0, 49.0));
    }

    #[test]
    fn image_mesh_uses_cached_weights() {
        let mut p = proj();
        let mut pm = tiny_skia::Pixmap::new(4, 4).unwrap();
        pm.fill(tiny_skia::Color::from_rgba8(255, 0, 0, 255));
        let mut l = Layer::image_from_png("img", &pm.encode_png().unwrap(), v(40.0, 40.0));
        if let LayerKind::Image { size, .. } = &mut l.kind {
            *size = Some(v(100.0, 100.0));
        }
        l.deformers = p.layers[0].deformers.clone();
        p.layers = vec![l];
        let q = pose_cage(&p, "img", "c", 0.0, &[], Some((12.0, 0.0)), Ease::Linear).unwrap();
        let e = Engine::new(q).unwrap();
        let fr = e.evaluate(0.0);
        let m = fr.nodes[e.node_index("img").unwrap()]
            .image
            .as_ref()
            .unwrap()
            .mesh
            .as_ref()
            .unwrap();
        let rest = crate::project::mesh_vertices(v(40.0, 40.0), v(100.0, 100.0), 16, 16);
        for (a, b) in m.positions.iter().zip(&rest) {
            assert!((a.x - b.x - 12.0).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3);
        }
    }

    #[test]
    fn pose_errors_and_locks() {
        let p = proj();
        assert!(pose_cage(&p, "sq", "nope", 0.0, &[], Some((1.0, 1.0)), Ease::Linear).is_err());
        assert!(pose_cage(&p, "zz", "c", 0.0, &[], None, Ease::Linear).is_err());
        assert!(
            pose_cage(&p, "sq", "c", 0.0, &[(9, 1.0, 1.0)], None, Ease::Linear)
                .unwrap_err()
                .contains("does not exist")
        );
        assert!(pose_cage(&p, "sq", "c", -1.0, &[], None, Ease::Linear).is_err());
        let q = pose_cage(&p, "sq", "c", 5.0, &[(1, 3.0, 3.0)], None, Ease::Linear).unwrap();
        let mut locked = q.clone();
        chanmeta::set_meta(
            &mut locked,
            "sq::c::1",
            Some(Prop::CageDx),
            &chanmeta::Change {
                lock: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(pose_cage(
            &locked,
            "sq",
            "c",
            8.0,
            &[(1, 4.0, 4.0)],
            None,
            Ease::Linear
        )
        .unwrap_err()
        .contains("locked"));
        // re-keying updates the existing channel instead of adding another
        let r = pose_cage(&q, "sq", "c", 8.0, &[(1, 4.0, 4.0)], None, Ease::Linear).unwrap();
        assert_eq!(r.channels.len(), q.channels.len());
    }

    #[test]
    fn engine_rejects_bad_cages() {
        let mut p = proj();
        p.layers[0].deformers = vec![Deformer::Cage {
            name: "c".into(),
            rest: vec![v(0.0, 0.0), v(1.0, 1.0)],
            offsets: vec![],
        }];
        assert!(Engine::new(p.clone()).is_err());
        p.layers[0].deformers = vec![Deformer::Cage {
            name: "c".into(),
            rest: square(),
            offsets: vec![Vec2::ZERO; 3],
        }];
        assert!(Engine::new(p.clone())
            .err()
            .unwrap()
            .join(" ")
            .contains("offsets"));
    }

    #[test]
    fn add_cage_variants() {
        let mut p = proj();
        p.layers[0].deformers.clear();
        let q = add_cage(&p, "sq", &CageOptions::default()).unwrap();
        let Deformer::Cage { rest, .. } = &q.layers[0].deformers[0] else {
            panic!()
        };
        assert_eq!(rest.len(), 8);
        assert!(rest
            .iter()
            .all(|r| r.x >= 36.0 - 1e-4 && r.x <= 144.0 + 1e-4));
        let h = add_cage(
            &p,
            "sq",
            &CageOptions {
                hull: true,
                pad: 0.0,
                ..Default::default()
            },
        )
        .unwrap();
        let Deformer::Cage { rest, .. } = &h.layers[0].deformers[0] else {
            panic!()
        };
        assert_eq!(rest.len(), 4);
        let r = add_cage(
            &p,
            "sq",
            &CageOptions {
                rect: Some([0.0, 0.0, 50.0, 50.0]),
                verts: 1,
                pad: 0.0,
                ..Default::default()
            },
        )
        .unwrap();
        let Deformer::Cage { rest, .. } = &r.layers[0].deformers[0] else {
            panic!()
        };
        assert_eq!(rest.len(), 4);
        // errors
        assert!(add_cage(&p, "nope", &CageOptions::default()).is_err());
        assert!(add_cage(&q, "sq", &CageOptions::default())
            .unwrap_err()
            .contains("already"));
        assert!(add_cage(
            &p,
            "sq",
            &CageOptions {
                pad: -1.0,
                ..Default::default()
            }
        )
        .is_err());
        assert!(add_cage(
            &p,
            "sq",
            &CageOptions {
                verts: 0,
                ..Default::default()
            }
        )
        .is_err());
        assert!(add_cage(
            &p,
            "sq",
            &CageOptions {
                rect: Some([0.0, 0.0, 0.0, 5.0]),
                pad: 0.0,
                ..Default::default()
            }
        )
        .is_err());
        assert!(add_cage(
            &p,
            "sq",
            &CageOptions {
                name: "a::b".into(),
                ..Default::default()
            }
        )
        .is_err());
    }

    #[test]
    fn serde_roundtrip_and_old_projects() {
        let d: Deformer = serde_json::from_str(
            r#"{"type":"cage","name":"c","rest":[{"x":0,"y":0},{"x":10,"y":0},{"x":10,"y":10}]}"#,
        )
        .or_else(|_| {
            serde_json::from_str(r#"{"type":"cage","name":"c","rest":[[0,0],[10,0],[10,10]]}"#)
        })
        .unwrap();
        assert!(matches!(d, Deformer::Cage { ref offsets, .. } if offsets.is_empty()));
        let s = serde_json::to_string(&d).unwrap();
        assert_eq!(serde_json::from_str::<Deformer>(&s).unwrap(), d);
        assert_eq!(Prop::parse("cage_dx"), Some(Prop::CageDx));
        // a project without any cage still loads and renders
        let p = crate::demo::project();
        assert!(Engine::new(p).is_ok());
    }

    #[test]
    fn parse_moves_ok_and_bad() {
        assert_eq!(
            parse_moves("0:1,2;3:-4.5,0").unwrap(),
            vec![(0, 1.0, 2.0), (3, -4.5, 0.0)]
        );
        assert!(parse_moves("0-1,2").is_err());
        assert!(parse_moves("x:1,2").is_err());
        assert!(parse_moves("0:1").is_err());
    }
}
