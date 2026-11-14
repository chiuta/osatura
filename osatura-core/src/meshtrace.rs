// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Alpha-silhouette mesh tracing: a free-form triangle mesh that follows an image's outline.
//!
//! Clean-room design note (textbook algorithms only):
//! 1. *Outline*: `trace::trace_alpha` (marching squares + Ramer–Douglas–Peucker, holes by
//!    containment depth). Every edge is subdivided to at most the boundary edge size.
//! 2. *Interior points*: variable-radius Poisson-disc sampling (Bridson's active-list scheme,
//!    deterministic splitmix64 generator). The target spacing `h(p)` grows linearly with the
//!    distance to the outline (`grading`) and shrinks again around hot-spots (user points or
//!    bone samples).
//! 3. *Triangulation*: Bowyer–Watson Delaunay with triangle adjacency and walking point
//!    location. Outline edges missing from the result are recovered by splitting them at their
//!    midpoints (conforming Delaunay). Triangles whose centroid is outside the even-odd region
//!    of the outline (outside the shape, or inside a hole) are dropped.
//! 4. *Vertex-flow smoothing*: Laplacian (`lambda`) or Taubin (`lambda|mu`, no shrinkage)
//!    smoothing of the interior vertices; the outline is fixed unless asked for. A move that
//!    would flip a triangle is shortened or skipped, so the mesh never inverts.
//!
//! Nothing here panics on degenerate input; problems are reported as `Err(String)` or counted in
//! `TracedMesh::unresolved_edges` (outline edges that could not be made part of the mesh; the
//! triangles stay valid, only the fit to the silhouette is then locally coarser).

use crate::math::Vec2;
use crate::project::{FreeMesh, ImageSource, LayerKind, MeshSpec, Project};
use std::collections::{HashMap, HashSet};

/// A region of interest: the mesh is refined around it (positions in image pixels).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hotspot {
    pub pos: Vec2,
    pub radius: f32,
}

#[derive(Clone, Debug)]
pub struct TraceOptions {
    /// Alpha value (0..255) at or above which a pixel belongs to the silhouette.
    pub threshold: u8,
    /// Douglas–Peucker tolerance of the outline in pixels.
    pub tolerance: f32,
    /// Outlines smaller than this (pixels²) are ignored.
    pub min_area: f32,
    /// Largest interior edge length in pixels.
    pub density: f32,
    /// Edge length at the outline and in hot-spots; 0 = `density / 4`.
    pub edge_size: f32,
    /// Growth of the edge length per pixel of distance from the outline / hot-spot.
    pub grading: f32,
    pub hotspots: Vec<Hotspot>,
    /// Edge length inside a hot-spot; 0 = `edge_size`.
    pub hotspot_size: f32,
    pub smooth: SmoothOptions,
}

impl Default for TraceOptions {
    fn default() -> Self {
        TraceOptions {
            threshold: 128,
            tolerance: 1.5,
            min_area: 12.0,
            density: 24.0,
            edge_size: 0.0,
            grading: 0.5,
            hotspots: Vec::new(),
            hotspot_size: 0.0,
            smooth: SmoothOptions::default(),
        }
    }
}

/// Vertex-flow smoothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmoothOptions {
    /// Number of (lambda, mu) passes; 0 = off.
    pub iterations: usize,
    /// Step toward the neighbour mean, in (0, 1].
    pub lambda: f32,
    /// Taubin shrink compensation (the second, inflating pass of each iteration).
    pub taubin: bool,
    /// Also move outline vertices (along the outline only).
    pub boundary: bool,
}

impl Default for SmoothOptions {
    fn default() -> Self {
        SmoothOptions {
            iterations: 0,
            lambda: 0.5,
            taubin: true,
            boundary: false,
        }
    }
}

/// A traced triangle mesh (positions in image pixels, counter-clockwise: positive cross product).
#[derive(Clone, Debug, PartialEq)]
pub struct TracedMesh {
    pub vertices: Vec<Vec2>,
    pub triangles: Vec<[u32; 3]>,
    /// True for vertices on an outline (outer or hole).
    pub boundary: Vec<bool>,
    /// Outline edges the triangulation could not recover (normally 0).
    pub unresolved_edges: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeshStats {
    pub vertices: usize,
    pub triangles: usize,
    pub boundary_vertices: usize,
    pub area: f32,
    /// Triangle quality `4·sqrt(3)·A / Σ l²` (1 = equilateral).
    pub min_quality: f32,
    pub mean_quality: f32,
    pub min_angle_deg: f32,
    pub mean_boundary_edge: f32,
    pub mean_interior_edge: f32,
    pub unresolved_edges: usize,
}

const MAX_POINTS: usize = 60_000;

// ---------------------------------------------------------------------------------------------
// small geometry helpers (f64)
// ---------------------------------------------------------------------------------------------

type P = [f64; 2];

fn orient(a: P, b: P, c: P) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// > 0 when `d` is strictly inside the circumcircle of the counter-clockwise triangle `abc`.
fn incircle(a: P, b: P, c: P, d: P) -> f64 {
    let (ax, ay) = (a[0] - d[0], a[1] - d[1]);
    let (bx, by) = (b[0] - d[0], b[1] - d[1]);
    let (cx, cy) = (c[0] - d[0], c[1] - d[1]);
    let (a2, b2, c2) = (ax * ax + ay * ay, bx * bx + by * by, cx * cx + cy * cy);
    ax * (by * c2 - b2 * cy) - ay * (bx * c2 - b2 * cx) + a2 * (bx * cy - by * cx)
}

fn dist_seg(p: P, a: P, b: P) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((p[0] - a[0] - t * dx).powi(2) + (p[1] - a[1] - t * dy).powi(2)).sqrt()
}

fn p64(v: Vec2) -> P {
    [v.x as f64, v.y as f64]
}

/// Even-odd point-in-region test over a set of rings, accelerated by horizontal buckets.
struct Region {
    edges: Vec<(P, P)>,
    y0: f64,
    bh: f64,
    buckets: Vec<Vec<u32>>,
}

impl Region {
    fn new(rings: &[Vec<P>]) -> Region {
        let mut edges = Vec::new();
        for r in rings {
            for i in 0..r.len() {
                edges.push((r[i], r[(i + 1) % r.len()]));
            }
        }
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for e in &edges {
            lo = lo.min(e.0[1]).min(e.1[1]);
            hi = hi.max(e.0[1]).max(e.1[1]);
        }
        if edges.is_empty() {
            return Region {
                edges,
                y0: 0.0,
                bh: 1.0,
                buckets: Vec::new(),
            };
        }
        let nb = (edges.len() / 2).clamp(1, 4096);
        let bh = ((hi - lo) / nb as f64).max(1e-6);
        let mut buckets = vec![Vec::new(); nb + 1];
        for (i, e) in edges.iter().enumerate() {
            let a = (((e.0[1].min(e.1[1]) - lo) / bh) as usize).min(nb);
            let b = (((e.0[1].max(e.1[1]) - lo) / bh) as usize).min(nb);
            for bucket in &mut buckets[a..=b] {
                bucket.push(i as u32);
            }
        }
        Region {
            edges,
            y0: lo,
            bh,
            buckets,
        }
    }

    fn inside(&self, p: P) -> bool {
        if self.buckets.is_empty()
            || p[1] < self.y0
            || !matches!(
                p[1].partial_cmp(&(self.y0 + self.bh * (self.buckets.len() - 1) as f64 + self.bh)),
                Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
            )
        {
            return false;
        }
        let b = (((p[1] - self.y0) / self.bh) as usize).min(self.buckets.len() - 1);
        let mut inside = false;
        for &ei in &self.buckets[b] {
            let (a, c) = self.edges[ei as usize];
            if (a[1] > p[1]) != (c[1] > p[1])
                && p[0] < (c[0] - a[0]) * (p[1] - a[1]) / (c[1] - a[1]) + a[0]
            {
                inside = !inside;
            }
        }
        inside
    }
}

/// Distance to the outline, by cell buckets of the outline segments.
struct SegGrid {
    cell: f64,
    x0: f64,
    y0: f64,
    nx: usize,
    ny: usize,
    cells: Vec<Vec<u32>>,
    segs: Vec<(P, P)>,
}

impl SegGrid {
    fn new(rings: &[Vec<P>], cell: f64) -> SegGrid {
        let mut segs = Vec::new();
        for r in rings {
            for i in 0..r.len() {
                segs.push((r[i], r[(i + 1) % r.len()]));
            }
        }
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for s in &segs {
            for q in [s.0, s.1] {
                x0 = x0.min(q[0]);
                y0 = y0.min(q[1]);
                x1 = x1.max(q[0]);
                y1 = y1.max(q[1]);
            }
        }
        let cell = cell.max(1e-3);
        let nx = (((x1 - x0) / cell) as usize + 1).min(4096);
        let ny = (((y1 - y0) / cell) as usize + 1).min(4096);
        let cell = cell.max((x1 - x0) / nx as f64).max((y1 - y0) / ny as f64);
        let mut cells = vec![Vec::new(); nx * ny];
        for (i, s) in segs.iter().enumerate() {
            let cx = |x: f64| (((x - x0) / cell) as usize).min(nx - 1);
            let cy = |y: f64| (((y - y0) / cell) as usize).min(ny - 1);
            for j in cy(s.0[1].min(s.1[1]))..=cy(s.0[1].max(s.1[1])) {
                for k in cx(s.0[0].min(s.1[0]))..=cx(s.0[0].max(s.1[0])) {
                    cells[j * nx + k].push(i as u32);
                }
            }
        }
        SegGrid {
            cell,
            x0,
            y0,
            nx,
            ny,
            cells,
            segs,
        }
    }

    /// Distance to the nearest outline segment, searched within `reach` pixels (else `reach`).
    fn dist(&self, p: P, reach: f64) -> f64 {
        let r = (reach / self.cell).ceil() as i64 + 1;
        let cx = ((p[0] - self.x0) / self.cell).floor() as i64;
        let cy = ((p[1] - self.y0) / self.cell).floor() as i64;
        let mut best = reach;
        for j in (cy - r).max(0)..=(cy + r).min(self.ny as i64 - 1) {
            for k in (cx - r).max(0)..=(cx + r).min(self.nx as i64 - 1) {
                for &si in &self.cells[j as usize * self.nx + k as usize] {
                    let s = self.segs[si as usize];
                    best = best.min(dist_seg(p, s.0, s.1));
                }
            }
        }
        best
    }
}

/// Target edge length as a function of position.
struct SizeField<'a> {
    h_b: f64,
    h_max: f64,
    g: f64,
    hot: &'a [Hotspot],
    h_hot: f64,
    reach: f64,
    outline: SegGrid,
}

impl SizeField<'_> {
    fn at(&self, p: P) -> f64 {
        let d = self.outline.dist(p, self.reach);
        let mut h = (self.h_b + self.g * d).min(self.h_max);
        for s in self.hot {
            let dh =
                ((p[0] - s.pos.x as f64).hypot(p[1] - s.pos.y as f64) - s.radius as f64).max(0.0);
            h = h.min((self.h_hot + self.g * dh).min(self.h_max));
        }
        h
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

// ---------------------------------------------------------------------------------------------
// Bowyer–Watson Delaunay triangulation with adjacency
// ---------------------------------------------------------------------------------------------

const NONE: usize = usize::MAX;

#[derive(Clone, Copy)]
struct Tri {
    v: [usize; 3],
    /// `n[i]`: the triangle across the edge opposite `v[i]`.
    n: [usize; 3],
    alive: bool,
}

struct Delaunay {
    pts: Vec<P>,
    tris: Vec<Tri>,
    last: usize,
    n_super: usize,
}

impl Delaunay {
    /// `bbox`: (min, max) of the points to be inserted.
    fn new(lo: P, hi: P) -> Delaunay {
        let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
        let r = ((hi[0] - lo[0]).max(hi[1] - lo[1]).max(1.0)) * 20.0;
        let pts = vec![
            [cx - 2.0 * r, cy - r],
            [cx + 2.0 * r, cy - r],
            [cx, cy + 2.0 * r],
        ];
        let t = Tri {
            v: [0, 1, 2],
            n: [NONE; 3],
            alive: true,
        };
        Delaunay {
            pts,
            tris: vec![t],
            last: 0,
            n_super: 3,
        }
    }

    fn locate(&self, p: P) -> Option<usize> {
        let mut t = if self.tris[self.last].alive {
            self.last
        } else {
            self.tris.iter().position(|t| t.alive)?
        };
        for _ in 0..self.tris.len() * 3 + 16 {
            let tr = &self.tris[t];
            let mut moved = false;
            for i in 0..3 {
                let (a, b) = (self.pts[tr.v[(i + 1) % 3]], self.pts[tr.v[(i + 2) % 3]]);
                if orient(a, b, p) < 0.0 {
                    if tr.n[i] == NONE {
                        return None;
                    }
                    t = tr.n[i];
                    moved = true;
                    break;
                }
            }
            if !moved {
                return Some(t);
            }
        }
        // fall back to a scan (walk cycled on a degenerate configuration)
        (0..self.tris.len()).find(|&i| {
            let tr = &self.tris[i];
            tr.alive
                && (0..3).all(|k| {
                    orient(self.pts[tr.v[(k + 1) % 3]], self.pts[tr.v[(k + 2) % 3]], p) >= 0.0
                })
        })
    }

    /// Insert a point; returns its index (the existing one for a duplicate), or `None` when the
    /// insertion would be numerically unsafe (the point is then skipped).
    fn insert(&mut self, p: P) -> Option<usize> {
        if !p[0].is_finite() || !p[1].is_finite() {
            return None;
        }
        let t0 = self.locate(p)?;
        for &vi in &self.tris[t0].v {
            let q = self.pts[vi];
            if (q[0] - p[0]).abs() < 1e-7 && (q[1] - p[1]).abs() < 1e-7 {
                return if vi >= self.n_super { Some(vi) } else { None };
            }
        }
        // cavity
        let mut bad = vec![t0];
        let mut seen: HashSet<usize> = HashSet::new();
        seen.insert(t0);
        let mut k = 0;
        while k < bad.len() {
            let t = bad[k];
            k += 1;
            for i in 0..3 {
                let nb = self.tris[t].n[i];
                if nb != NONE && !seen.contains(&nb) {
                    seen.insert(nb);
                    let v = self.tris[nb].v;
                    if incircle(self.pts[v[0]], self.pts[v[1]], self.pts[v[2]], p) > 1e-9 {
                        bad.push(nb);
                    }
                }
            }
        }
        let bad_set: HashSet<usize> = bad.iter().copied().collect();
        // boundary edges (a, b, outer), counter-clockwise around the cavity
        let mut edges: Vec<(usize, usize, usize)> = Vec::new();
        for &t in &bad {
            let tr = self.tris[t];
            for i in 0..3 {
                if tr.n[i] == NONE || !bad_set.contains(&tr.n[i]) {
                    edges.push((tr.v[(i + 1) % 3], tr.v[(i + 2) % 3], tr.n[i]));
                }
            }
        }
        // star-shaped and simple?
        let mut starts: HashMap<usize, usize> = HashMap::new();
        for (ei, e) in edges.iter().enumerate() {
            if orient(self.pts[e.0], self.pts[e.1], p) <= 1e-12 || starts.insert(e.0, ei).is_some()
            {
                return None;
            }
        }
        let pi = self.pts.len();
        self.pts.push(p);
        for &t in &bad {
            self.tris[t].alive = false;
        }
        let base = self.tris.len();
        for e in &edges {
            self.tris.push(Tri {
                v: [e.0, e.1, pi],
                n: [NONE, NONE, e.2],
                alive: true,
            });
        }
        for (ei, e) in edges.iter().enumerate() {
            let me = base + ei;
            // edge (b, p) is shared with the new triangle that starts at b
            if let Some(&o) = starts.get(&e.1) {
                self.tris[me].n[0] = base + o;
            }
            // edge (p, a) is shared with the new triangle that ends at a
            if let Some(o) = edges.iter().position(|x| x.1 == e.0) {
                self.tris[me].n[1] = base + o;
            }
            if e.2 != NONE {
                let ot = self.tris[e.2];
                for j in 0..3 {
                    if ot.v[(j + 1) % 3] == e.1 && ot.v[(j + 2) % 3] == e.0 {
                        self.tris[e.2].n[j] = me;
                    }
                }
            }
        }
        self.last = base;
        Some(pi)
    }

    fn edge_set(&self) -> HashSet<(usize, usize)> {
        let mut s = HashSet::new();
        for t in self.tris.iter().filter(|t| t.alive) {
            for i in 0..3 {
                let (a, b) = (t.v[i], t.v[(i + 1) % 3]);
                s.insert((a.min(b), a.max(b)));
            }
        }
        s
    }
}

// ---------------------------------------------------------------------------------------------
// the tracer
// ---------------------------------------------------------------------------------------------

fn finite_pos(x: f32) -> bool {
    x.is_finite() && x > 0.0
}

/// Trace an `w × h` alpha mask into a triangle mesh (positions in pixels).
pub fn trace_mesh(
    alpha: &[u8],
    w: usize,
    h: usize,
    o: &TraceOptions,
) -> Result<TracedMesh, String> {
    if w == 0 || h == 0 || alpha.len() < w * h {
        return Err("alpha mask is empty or smaller than width x height".into());
    }
    if w.saturating_mul(h) > 64_000_000 {
        return Err("image is too large to trace (max 64 megapixels)".into());
    }
    if !finite_pos(o.density) || o.density < 1.0 {
        return Err("density must be a finite number >= 1".into());
    }
    if !(o.edge_size == 0.0 || finite_pos(o.edge_size))
        || !(o.hotspot_size == 0.0 || finite_pos(o.hotspot_size))
    {
        return Err("edge size and hot-spot size must be positive".into());
    }
    if !o.tolerance.is_finite() || o.tolerance < 0.0 || !o.min_area.is_finite() {
        return Err("tolerance must be a finite number >= 0".into());
    }
    if !o.grading.is_finite() || o.grading < 0.01 || o.grading > 100.0 {
        return Err("grading must be within 0.01..100".into());
    }
    let s = o.smooth;
    if !s.lambda.is_finite() || s.lambda <= 0.0 || s.lambda > 1.0 {
        return Err("smoothing lambda must be in (0, 1]".into());
    }
    if s.iterations > 1000 {
        return Err("at most 1000 smoothing iterations".into());
    }
    for hs in &o.hotspots {
        if !hs.pos.x.is_finite()
            || !hs.pos.y.is_finite()
            || !hs.radius.is_finite()
            || hs.radius < 0.0
        {
            return Err("hot-spots need finite positions and a radius >= 0".into());
        }
    }
    let h_b = if o.edge_size > 0.0 {
        o.edge_size as f64
    } else {
        (o.density as f64 / 4.0).max(1.0)
    };
    let h_max = (o.density as f64).max(h_b);
    let h_hot = if o.hotspot_size > 0.0 {
        o.hotspot_size as f64
    } else {
        h_b
    };

    let contours = crate::trace::trace_alpha(alpha, w, h, o.threshold, o.tolerance, o.min_area);
    if contours.is_empty() {
        return Err("no opaque region found (nothing at or above the alpha threshold, or smaller than min area)".into());
    }

    // outline rings, subdivided to the boundary edge size
    let mut pts: Vec<P> = Vec::new();
    let mut rings: Vec<Vec<P>> = Vec::new();
    let mut constraints: Vec<(usize, usize)> = Vec::new();
    for c in &contours {
        let mut ring: Vec<P> = Vec::new();
        for i in 0..c.pts.len() {
            let (a, b) = (p64(c.pts[i]), p64(c.pts[(i + 1) % c.pts.len()]));
            let l = (b[0] - a[0]).hypot(b[1] - a[1]);
            let n = ((l / h_b).ceil() as usize).max(1);
            if n > MAX_POINTS {
                return Err("outline is too long for this edge size; raise edge size".into());
            }
            for k in 0..n {
                let t = k as f64 / n as f64;
                ring.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]);
            }
            if pts.len() + ring.len() > MAX_POINTS {
                return Err(format!(
                    "more than {MAX_POINTS} outline vertices; raise edge size or tolerance"
                ));
            }
        }
        // drop consecutive duplicates
        ring.dedup_by(|a, b| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9);
        if ring.len() > 1
            && (ring[0][0] - ring[ring.len() - 1][0]).abs() < 1e-9
            && (ring[0][1] - ring[ring.len() - 1][1]).abs() < 1e-9
        {
            ring.pop();
        }
        if ring.len() < 3 {
            continue;
        }
        let base = pts.len();
        for i in 0..ring.len() {
            constraints.push((base + i, base + (i + 1) % ring.len()));
        }
        pts.extend_from_slice(&ring);
        rings.push(ring);
    }
    if rings.is_empty() {
        return Err("no usable outline".into());
    }
    let n_outline = pts.len();
    let region = Region::new(&rings);

    // interior points: variable-radius Poisson-disc sampling
    let reach = (h_max - h_b) / o.grading as f64 + 1.0;
    let size = SizeField {
        h_b,
        h_max,
        g: o.grading as f64,
        hot: &o.hotspots,
        h_hot,
        reach,
        outline: SegGrid::new(&rings, h_max.max(4.0)),
    };
    let cell = (0.85 * h_max).max(1e-3);
    let mut hash: HashMap<(i64, i64), Vec<u32>> = HashMap::new();
    let key = |p: P| ((p[0] / cell).floor() as i64, (p[1] / cell).floor() as i64);
    let mut hs: Vec<f64> = Vec::with_capacity(n_outline);
    for (i, p) in pts.iter().enumerate() {
        hash.entry(key(*p)).or_default().push(i as u32);
        hs.push(h_b);
    }
    let mut active: Vec<usize> = (0..n_outline).collect();
    let mut rng = Rng(0xA5A5_1234_5678_9ABC);
    while !active.is_empty() {
        let ai = (rng.next() % active.len() as u64) as usize;
        let src = active[ai];
        let (sp, sh) = (pts[src], hs[src]);
        let mut found = false;
        for _ in 0..20 {
            let ang = rng.unit() * std::f64::consts::TAU;
            let rad = sh * 0.85 * (1.0 + rng.unit());
            let c = [sp[0] + ang.cos() * rad, sp[1] + ang.sin() * rad];
            if c[0] < 0.0 || c[1] < 0.0 || c[0] > w as f64 || c[1] > h as f64 || !region.inside(c) {
                continue;
            }
            let hc = size.at(c);
            let (kx, ky) = key(c);
            let mut ok = true;
            'chk: for dy in -1..=1 {
                for dx in -1..=1 {
                    if let Some(v) = hash.get(&(kx + dx, ky + dy)) {
                        for &q in v {
                            let q = q as usize;
                            let need = 0.85 * 0.5 * (hc + hs[q]);
                            if (pts[q][0] - c[0]).powi(2) + (pts[q][1] - c[1]).powi(2) < need * need
                            {
                                ok = false;
                                break 'chk;
                            }
                        }
                    }
                }
            }
            if !ok {
                continue;
            }
            if pts.len() >= MAX_POINTS {
                return Err(format!("more than {MAX_POINTS} vertices; raise density"));
            }
            let id = pts.len();
            pts.push(c);
            hs.push(hc);
            hash.entry((kx, ky)).or_default().push(id as u32);
            active.push(id);
            found = true;
            break;
        }
        if !found {
            active.swap_remove(ai);
        }
    }

    // Delaunay triangulation
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in &pts {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let mut dt = Delaunay::new(lo, hi);
    let mut vid: Vec<Option<usize>> = Vec::with_capacity(pts.len()); // our point -> dt vertex
    for p in &pts {
        vid.push(dt.insert(*p));
    }
    // constraint edges in dt vertex ids; vertices on an outline
    let mut cons: Vec<(usize, usize)> = constraints
        .iter()
        .filter_map(|&(a, b)| Some((vid[a]?, vid[b]?)).filter(|(x, y)| x != y))
        .collect();
    let mut on_outline: HashSet<usize> = cons.iter().flat_map(|c| [c.0, c.1]).collect();
    let mut unresolved = constraints.len() - cons.len();
    // conforming recovery: split missing outline edges at their midpoints
    for _ in 0..40 {
        let present = dt.edge_set();
        let mut next = Vec::with_capacity(cons.len());
        let mut changed = false;
        for &(a, b) in &cons {
            if present.contains(&(a.min(b), a.max(b))) {
                next.push((a, b));
                continue;
            }
            let (pa, pb) = (dt.pts[a], dt.pts[b]);
            if (pb[0] - pa[0]).hypot(pb[1] - pa[1]) < 2e-3 || dt.pts.len() > MAX_POINTS * 2 {
                next.push((a, b));
                continue;
            }
            match dt.insert([(pa[0] + pb[0]) * 0.5, (pa[1] + pb[1]) * 0.5]) {
                Some(m) if m != a && m != b => {
                    on_outline.insert(m);
                    next.push((a, m));
                    next.push((m, b));
                    changed = true;
                }
                _ => next.push((a, b)),
            }
        }
        cons = next;
        if !changed {
            break;
        }
    }
    let present = dt.edge_set();
    unresolved += cons
        .iter()
        .filter(|&&(a, b)| !present.contains(&(a.min(b), a.max(b))))
        .count();

    // keep the triangles inside the silhouette
    let mut remap: HashMap<usize, u32> = HashMap::new();
    let mut vertices: Vec<Vec2> = Vec::new();
    let mut boundary: Vec<bool> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    for t in dt
        .tris
        .iter()
        .filter(|t| t.alive && t.v.iter().all(|&v| v >= dt.n_super))
    {
        let (a, b, c) = (dt.pts[t.v[0]], dt.pts[t.v[1]], dt.pts[t.v[2]]);
        if orient(a, b, c) < 1e-9 {
            continue;
        }
        let cen = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
        if !region.inside(cen) {
            continue;
        }
        let mut ix = [0u32; 3];
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for k in 0..3 {
            let v = t.v[k];
            ix[k] = *remap.entry(v).or_insert_with(|| {
                vertices.push(Vec2::new(dt.pts[v][0] as f32, dt.pts[v][1] as f32));
                boundary.push(on_outline.contains(&v));
                (vertices.len() - 1) as u32
            });
        }
        triangles.push(ix);
    }
    if triangles.is_empty() {
        return Err(
            "triangulation produced no triangles (shape too thin for this edge size?)".into(),
        );
    }
    let mut mesh = TracedMesh {
        vertices,
        triangles,
        boundary,
        unresolved_edges: unresolved,
    };
    if s.iterations > 0 {
        smooth_mesh(&mut mesh, &s);
    }
    Ok(mesh)
}

// ---------------------------------------------------------------------------------------------
// vertex-flow smoothing
// ---------------------------------------------------------------------------------------------

fn tri_area(v: &[Vec2], t: [u32; 3]) -> f32 {
    let (a, b, c) = (v[t[0] as usize], v[t[1] as usize], v[t[2] as usize]);
    ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)) * 0.5
}

/// Laplacian / Taubin smoothing of the mesh vertices in place. Triangle orientation is
/// preserved: a vertex move that would flip or collapse an incident triangle is halved up to
/// four times and otherwise skipped.
pub fn smooth_mesh(m: &mut TracedMesh, o: &SmoothOptions) {
    let n = m.vertices.len();
    if n == 0
        || o.iterations == 0
        || (o.lambda.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater))
    {
        return;
    }
    // adjacency; boundary edges are those used by exactly one triangle
    let mut edge_use: HashMap<(u32, u32), u32> = HashMap::new();
    let mut inc: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (ti, t) in m.triangles.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            *edge_use.entry((a.min(b), a.max(b))).or_default() += 1;
            inc[a as usize].push(ti as u32);
        }
    }
    let mut nbr: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut bnbr: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut keys: Vec<(&(u32, u32), &u32)> = edge_use.iter().collect();
    keys.sort();
    for (&(a, b), &c) in keys {
        nbr[a as usize].push(b);
        nbr[b as usize].push(a);
        if c == 1 {
            bnbr[a as usize].push(b);
            bnbr[b as usize].push(a);
        }
    }
    let is_b: Vec<bool> = (0..n).map(|i| !bnbr[i].is_empty()).collect();
    let mu = if o.taubin {
        // pass-band 0.1: mu = 1 / (kpb - 1/lambda)
        1.0 / (0.1 - 1.0 / o.lambda)
    } else {
        0.0
    };
    let pass = |m: &mut TracedMesh, step: f32| {
        let old = m.vertices.clone();
        for i in 0..n {
            let ring = if is_b[i] {
                if !o.boundary || bnbr[i].len() != 2 {
                    continue;
                }
                &bnbr[i]
            } else {
                &nbr[i]
            };
            if ring.is_empty() {
                continue;
            }
            let mut mean = Vec2::ZERO;
            for &j in ring {
                mean += old[j as usize];
            }
            mean = mean * (1.0 / ring.len() as f32);
            let delta = (mean - old[i]) * step;
            let before: Vec<f32> = inc[i]
                .iter()
                .map(|&t| tri_area(&m.vertices, m.triangles[t as usize]))
                .collect();
            let cur = m.vertices[i];
            let mut f = 1.0f32;
            for _ in 0..5 {
                m.vertices[i] = cur + delta * f;
                let ok = inc[i].iter().zip(&before).all(|(&t, &b0)| {
                    let a = tri_area(&m.vertices, m.triangles[t as usize]);
                    a > 0.05 * b0.min(a.max(b0)) && a > 1e-6
                });
                if ok {
                    break;
                }
                f *= 0.5;
                m.vertices[i] = cur;
            }
        }
    };
    for _ in 0..o.iterations {
        pass(m, o.lambda);
        if o.taubin {
            pass(m, mu);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// statistics
// ---------------------------------------------------------------------------------------------

/// Quality figures of a mesh.
pub fn mesh_stats(m: &TracedMesh) -> MeshStats {
    let (mut min_q, mut sum_q, mut area, mut min_ang) = (f32::MAX, 0.0f32, 0.0f32, f32::MAX);
    for t in &m.triangles {
        let (a, b, c) = (
            m.vertices[t[0] as usize],
            m.vertices[t[1] as usize],
            m.vertices[t[2] as usize],
        );
        let ar = tri_area(&m.vertices, *t);
        let (la, lb, lc) = (b.distance(c), a.distance(c), a.distance(b));
        let q = if la * la + lb * lb + lc * lc > 0.0 {
            4.0 * 3f32.sqrt() * ar / (la * la + lb * lb + lc * lc)
        } else {
            0.0
        };
        min_q = min_q.min(q);
        sum_q += q;
        area += ar;
        // smallest angle through the law of cosines
        for (x, y, z) in [(la, lb, lc), (lb, lc, la), (lc, la, lb)] {
            if y > 0.0 && z > 0.0 {
                let cos = ((y * y + z * z - x * x) / (2.0 * y * z)).clamp(-1.0, 1.0);
                min_ang = min_ang.min(cos.acos().to_degrees());
            }
        }
    }
    let mut edges: HashSet<(u32, u32)> = HashSet::new();
    for t in &m.triangles {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            edges.insert((a.min(b), a.max(b)));
        }
    }
    let (mut sb, mut nb, mut si, mut ni) = (0.0f32, 0usize, 0.0f32, 0usize);
    for &(a, b) in &edges {
        let l = m.vertices[a as usize].distance(m.vertices[b as usize]);
        if m.boundary[a as usize] && m.boundary[b as usize] {
            sb += l;
            nb += 1;
        } else if !m.boundary[a as usize] && !m.boundary[b as usize] {
            si += l;
            ni += 1;
        }
    }
    let nt = m.triangles.len().max(1) as f32;
    MeshStats {
        vertices: m.vertices.len(),
        triangles: m.triangles.len(),
        boundary_vertices: m.boundary.iter().filter(|&&b| b).count(),
        area,
        min_quality: if m.triangles.is_empty() { 0.0 } else { min_q },
        mean_quality: sum_q / nt,
        min_angle_deg: if min_ang == f32::MAX { 0.0 } else { min_ang },
        mean_boundary_edge: if nb > 0 { sb / nb as f32 } else { 0.0 },
        mean_interior_edge: if ni > 0 { si / ni as f32 } else { 0.0 },
        unresolved_edges: m.unresolved_edges,
    }
}

// ---------------------------------------------------------------------------------------------
// project integration
// ---------------------------------------------------------------------------------------------

/// Decode an image source into `(alpha, width, height)`.
pub fn image_alpha(
    src: &ImageSource,
    asset_dir: &std::path::Path,
) -> Result<(Vec<u8>, usize, usize), String> {
    let png = match (&src.png_base64, &src.file) {
        (Some(b), _) => crate::base64::decode(b)?,
        (None, Some(f)) => {
            crate::assets::read(asset_dir, f, crate::assets::MAX_IMAGE_BYTES, "image")?
        }
        (None, None) => return Err("image has neither 'png_base64' nor 'file'".into()),
    };
    let pm = crate::assets::decode_png_checked(&png)?;
    let alpha = pm.data().chunks_exact(4).map(|p| p[3]).collect();
    Ok((alpha, pm.width() as usize, pm.height() as usize))
}

/// Hot-spots along every bone of the rest pose (layer-space coordinates), `spacing` apart.
pub fn bone_hotspots(p: &Project, radius: f32) -> Result<Vec<Hotspot>, String> {
    if !radius.is_finite() || radius < 0.0 {
        return Err("bone hot-spot radius must be >= 0".into());
    }
    let rest = p.rest_world()?;
    let mut out = Vec::new();
    for (b, w) in p.bones.iter().zip(&rest) {
        let (a, e) = (w.apply(Vec2::ZERO), w.apply(Vec2::new(b.length, 0.0)));
        let n = ((a.distance(e) / radius.max(1.0)).ceil() as usize).clamp(1, 64);
        for k in 0..=n {
            out.push(Hotspot {
                pos: a + (e - a) * (k as f32 / n as f32),
                radius,
            });
        }
    }
    Ok(out)
}

/// Trace the alpha of image layer `layer` and give it a free-form mesh. Hot-spots in `o` are in
/// layer space (the space of bone positions); they are converted to image pixels here. When the
/// project has bones and `skin` is true the new mesh is skinned by diffusion weights
/// (`Project::auto_skin_diffuse`).
pub fn trace_layer_mesh(
    p: &mut Project,
    layer: &str,
    o: &TraceOptions,
    asset_dir: &std::path::Path,
    skin: bool,
    influences: usize,
) -> Result<MeshStats, String> {
    let (alpha, w, h, origin, size) = {
        let l = p
            .find_layer_mut(layer)
            .ok_or_else(|| format!("no layer at '{layer}'"))?;
        let LayerKind::Image {
            source,
            origin,
            size,
            ..
        } = &l.kind
        else {
            return Err(format!("layer '{layer}' is not an image layer"));
        };
        let (alpha, w, h) = image_alpha(source, asset_dir)?;
        (
            alpha,
            w,
            h,
            *origin,
            size.unwrap_or(Vec2::new(w as f32, h as f32)),
        )
    };
    if !finite_pos(size.x) || !finite_pos(size.y) {
        return Err(format!("layer '{layer}' has an invalid size"));
    }
    let (sx, sy) = (w as f32 / size.x, h as f32 / size.y);
    let mut opts = o.clone();
    for hs in &mut opts.hotspots {
        hs.pos = Vec2::new((hs.pos.x - origin.x) * sx, (hs.pos.y - origin.y) * sy);
        hs.radius *= sx.max(sy);
    }
    let mesh = trace_mesh(&alpha, w, h, &opts)?;
    let stats = mesh_stats(&mesh);
    let free = FreeMesh {
        vertices: mesh
            .vertices
            .iter()
            .map(|v| Vec2::new(origin.x + v.x / sx, origin.y + v.y / sy))
            .collect(),
        triangles: mesh.triangles.clone(),
        uvs: mesh.vertices.clone(),
    };
    let l = p
        .find_layer_mut(layer)
        .ok_or_else(|| format!("no layer at '{layer}'"))?;
    if let LayerKind::Image {
        size: sz, mesh: m, ..
    } = &mut l.kind
    {
        sz.get_or_insert(size);
        *m = Some(MeshSpec {
            cols: 0,
            rows: 0,
            weights: Vec::new(),
            free: Some(free),
        });
    }
    if skin && !p.bones.is_empty() {
        p.auto_skin_diffuse(
            layer,
            influences.max(1),
            crate::weights::DiffusionOptions::default(),
        )?;
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Filled disc (optionally with a hole) in a `w × h` mask.
    fn disc(w: usize, h: usize, c: (f32, f32), r: f32, hole: f32) -> Vec<u8> {
        let mut a = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                let d = ((x as f32 + 0.5 - c.0).powi(2) + (y as f32 + 0.5 - c.1).powi(2)).sqrt();
                if d <= r && d >= hole {
                    a[y * w + x] = 255;
                }
            }
        }
        a
    }

    fn opaque(a: &[u8]) -> usize {
        a.iter().filter(|&&v| v >= 128).count()
    }

    fn opts() -> TraceOptions {
        TraceOptions {
            density: 16.0,
            edge_size: 4.0,
            tolerance: 1.0,
            ..Default::default()
        }
    }

    fn mask_at(a: &[u8], w: usize, h: usize, p: Vec2, grow: i32) -> bool {
        for dy in -grow..=grow {
            for dx in -grow..=grow {
                let (x, y) = (p.x.floor() as i32 + dx, p.y.floor() as i32 + dy);
                if x >= 0
                    && y >= 0
                    && (x as usize) < w
                    && (y as usize) < h
                    && a[y as usize * w + x as usize] >= 128
                {
                    return true;
                }
            }
        }
        false
    }

    fn assert_valid(m: &TracedMesh) {
        for t in &m.triangles {
            assert!(t.iter().all(|&i| (i as usize) < m.vertices.len()));
            assert!(
                tri_area(&m.vertices, *t) > 1e-4,
                "degenerate or inverted triangle {t:?}"
            );
        }
        assert_eq!(m.boundary.len(), m.vertices.len());
    }

    #[test]
    fn disc_area_matches_silhouette_and_nothing_sticks_out() {
        let (w, h) = (160, 160);
        let a = disc(w, h, (80.0, 80.0), 60.0, 0.0);
        let m = trace_mesh(&a, w, h, &opts()).unwrap();
        assert_valid(&m);
        assert_eq!(m.unresolved_edges, 0);
        let st = mesh_stats(&m);
        let want = opaque(&a) as f32;
        assert!(
            (st.area - want).abs() / want < 0.03,
            "mesh area {} vs silhouette {want}",
            st.area
        );
        for t in &m.triangles {
            let c =
                (m.vertices[t[0] as usize] + m.vertices[t[1] as usize] + m.vertices[t[2] as usize])
                    * (1.0 / 3.0);
            assert!(
                mask_at(&a, w, h, c, 1),
                "triangle centroid {c:?} outside the silhouette"
            );
        }
        // every triangle edge is inside the (slightly grown) silhouette too
        for v in &m.vertices {
            assert!(((v.x - 80.0).powi(2) + (v.y - 80.0).powi(2)).sqrt() < 61.5);
        }
    }

    #[test]
    fn hole_is_respected() {
        let (w, h) = (160, 160);
        let a = disc(w, h, (80.0, 80.0), 65.0, 30.0);
        let m = trace_mesh(&a, w, h, &opts()).unwrap();
        assert_valid(&m);
        assert_eq!(m.unresolved_edges, 0);
        let want = opaque(&a) as f32;
        let st = mesh_stats(&m);
        assert!(
            (st.area - want).abs() / want < 0.04,
            "area {} vs {want}",
            st.area
        );
        for t in &m.triangles {
            let c =
                (m.vertices[t[0] as usize] + m.vertices[t[1] as usize] + m.vertices[t[2] as usize])
                    * (1.0 / 3.0);
            let d = ((c.x - 80.0).powi(2) + (c.y - 80.0).powi(2)).sqrt();
            assert!(d > 28.0, "triangle centroid inside the hole (r={d})");
            assert!(d < 66.5);
        }
        // the hole outline has vertices too
        assert!(m
            .vertices
            .iter()
            .zip(&m.boundary)
            .any(|(v, &b)| b && ((v.x - 80.0).powi(2) + (v.y - 80.0).powi(2)).sqrt() < 32.0));
    }

    #[test]
    fn two_separate_islands_and_an_island_in_a_hole() {
        let (w, h) = (200, 100);
        let mut a = disc(w, h, (45.0, 50.0), 35.0, 0.0);
        let b = disc(w, h, (150.0, 50.0), 40.0, 25.0);
        let c = disc(w, h, (150.0, 50.0), 12.0, 0.0);
        for i in 0..a.len() {
            a[i] = a[i].max(b[i]).max(c[i]);
        }
        let m = trace_mesh(&a, w, h, &opts()).unwrap();
        assert_valid(&m);
        let want = opaque(&a) as f32;
        assert!((mesh_stats(&m).area - want).abs() / want < 0.05);
        // the island inside the ring is meshed, the gap around it (r in 14..23) is not
        let centre_tris = m.triangles.iter().filter(|t| {
            let c =
                (m.vertices[t[0] as usize] + m.vertices[t[1] as usize] + m.vertices[t[2] as usize])
                    * (1.0 / 3.0);
            ((c.x - 150.0).powi(2) + (c.y - 50.0).powi(2)).sqrt() < 10.0
        });
        assert!(centre_tris.count() > 0);
        for t in &m.triangles {
            let c =
                (m.vertices[t[0] as usize] + m.vertices[t[1] as usize] + m.vertices[t[2] as usize])
                    * (1.0 / 3.0);
            let d = ((c.x - 150.0).powi(2) + (c.y - 50.0).powi(2)).sqrt();
            assert!(!(15.0..23.0).contains(&d), "triangle in the gap at r={d}");
        }
    }

    #[test]
    fn density_is_higher_near_the_boundary_and_in_hotspots() {
        let (w, h) = (200, 200);
        let a = disc(w, h, (100.0, 100.0), 90.0, 0.0);
        let o = TraceOptions {
            density: 30.0,
            edge_size: 4.0,
            grading: 0.4,
            ..Default::default()
        };
        let m = trace_mesh(&a, w, h, &o).unwrap();
        let st = mesh_stats(&m);
        assert!(
            st.mean_interior_edge > 1.8 * st.mean_boundary_edge,
            "interior {} vs boundary {}",
            st.mean_interior_edge,
            st.mean_boundary_edge
        );
        // vertex count per area: outer ring (r 80..90) vs core (r < 40)
        let dens = |lo: f32, hi: f32| {
            let n = m
                .vertices
                .iter()
                .filter(|v| (lo..hi).contains(&((v.x - 100.0).hypot(v.y - 100.0))))
                .count() as f32;
            n / (std::f32::consts::PI * (hi * hi - lo * lo))
        };
        assert!(
            dens(78.0, 90.0) > 4.0 * dens(0.0, 40.0),
            "{} vs {}",
            dens(78.0, 90.0),
            dens(0.0, 40.0)
        );

        // hot-spot in the middle refines its neighbourhood
        let o2 = TraceOptions {
            hotspots: vec![Hotspot {
                pos: Vec2::new(100.0, 100.0),
                radius: 12.0,
            }],
            ..o.clone()
        };
        let m2 = trace_mesh(&a, w, h, &o2).unwrap();
        let near = |m: &TracedMesh| {
            m.vertices
                .iter()
                .filter(|v| (v.x - 100.0).hypot(v.y - 100.0) < 20.0)
                .count()
        };
        assert!(
            near(&m2) > 3 * near(&m).max(1),
            "hotspot {} vs plain {}",
            near(&m2),
            near(&m)
        );
        assert_valid(&m2);
    }

    #[test]
    fn smoothing_improves_quality_keeps_boundary_and_orientation() {
        let (w, h) = (160, 160);
        let a = disc(w, h, (80.0, 80.0), 70.0, 20.0);
        let raw = trace_mesh(&a, w, h, &opts()).unwrap();
        let mut sm = raw.clone();
        smooth_mesh(
            &mut sm,
            &SmoothOptions {
                iterations: 15,
                lambda: 0.6,
                taubin: true,
                boundary: false,
            },
        );
        assert_valid(&sm);
        let (q0, q1) = (mesh_stats(&raw), mesh_stats(&sm));
        assert!(
            q1.mean_quality > q0.mean_quality,
            "mean quality {} -> {}",
            q0.mean_quality,
            q1.mean_quality
        );
        assert!(
            q1.min_quality >= q0.min_quality * 0.95,
            "min quality {} -> {}",
            q0.min_quality,
            q1.min_quality
        );
        for i in 0..raw.vertices.len() {
            if raw.boundary[i] {
                assert_eq!(raw.vertices[i], sm.vertices[i], "boundary vertex moved");
            }
        }
        assert_eq!(raw.triangles, sm.triangles);
        // through the options as well, and with boundary smoothing the outline moves but stays valid
        let o = TraceOptions {
            smooth: SmoothOptions {
                iterations: 5,
                lambda: 0.5,
                taubin: true,
                boundary: true,
            },
            ..opts()
        };
        let mb = trace_mesh(&a, w, h, &o).unwrap();
        assert_valid(&mb);
        assert!((mesh_stats(&mb).area - opaque(&a) as f32).abs() / (opaque(&a) as f32) < 0.06);
        // plain Laplacian (no Taubin) shrinks less than it would collapse; still valid
        let mut lap = raw.clone();
        smooth_mesh(
            &mut lap,
            &SmoothOptions {
                iterations: 5,
                lambda: 1.0,
                taubin: false,
                boundary: false,
            },
        );
        assert_valid(&lap);
    }

    #[test]
    fn deterministic() {
        let a = disc(120, 120, (60.0, 60.0), 50.0, 15.0);
        let o = TraceOptions {
            smooth: SmoothOptions {
                iterations: 3,
                ..Default::default()
            },
            ..opts()
        };
        let m1 = trace_mesh(&a, 120, 120, &o).unwrap();
        let m2 = trace_mesh(&a, 120, 120, &o).unwrap();
        assert_eq!(m1, m2);
    }

    #[test]
    fn concave_shape_and_thin_features_do_not_break() {
        // L shape with a thin 2px spur
        let (w, h) = (100, 100);
        let mut a = vec![0u8; w * h];
        for y in 10..90 {
            for x in 10..90 {
                if x < 35 || y > 65 {
                    a[y * w + x] = 255;
                }
            }
        }
        for x in 35..80 {
            a[40 * w + x] = 255;
            a[41 * w + x] = 255;
        }
        let m = trace_mesh(&a, w, h, &opts()).unwrap();
        assert_valid(&m);
        let want = opaque(&a) as f32;
        assert!(
            (mesh_stats(&m).area - want).abs() / want < 0.08,
            "{} vs {want}",
            mesh_stats(&m).area
        );
        // a single opaque pixel and a 1-pixel line: either a mesh or a clean error, never a panic
        let mut dot = vec![0u8; 20 * 20];
        dot[5 * 20 + 5] = 255;
        let _ = trace_mesh(
            &dot,
            20,
            20,
            &TraceOptions {
                min_area: 0.0,
                ..opts()
            },
        );
        let mut line = vec![0u8; 50 * 10];
        for x in 5..45 {
            line[5 * 50 + x] = 255;
        }
        let _ = trace_mesh(
            &line,
            50,
            10,
            &TraceOptions {
                min_area: 0.0,
                ..opts()
            },
        );
        // image touching its borders
        let full = vec![255u8; 30 * 30];
        let m = trace_mesh(
            &full,
            30,
            30,
            &TraceOptions {
                density: 10.0,
                edge_size: 5.0,
                ..Default::default()
            },
        )
        .unwrap();
        assert_valid(&m);
        assert!(
            (mesh_stats(&m).area - 900.0).abs() < 40.0,
            "{}",
            mesh_stats(&m).area
        );
    }

    #[test]
    fn bad_input_is_an_error() {
        let a = disc(40, 40, (20.0, 20.0), 15.0, 0.0);
        assert!(trace_mesh(&[], 0, 0, &opts()).is_err());
        assert!(trace_mesh(&a[..10], 40, 40, &opts()).is_err());
        assert!(trace_mesh(&vec![0u8; 1600], 40, 40, &opts())
            .unwrap_err()
            .contains("no opaque"));
        for bad in [f32::NAN, 0.0, -3.0, f32::INFINITY, 0.5] {
            assert!(
                trace_mesh(
                    &a,
                    40,
                    40,
                    &TraceOptions {
                        density: bad,
                        ..opts()
                    }
                )
                .is_err(),
                "density {bad}"
            );
        }
        assert!(trace_mesh(
            &a,
            40,
            40,
            &TraceOptions {
                edge_size: -1.0,
                ..opts()
            }
        )
        .is_err());
        assert!(trace_mesh(
            &a,
            40,
            40,
            &TraceOptions {
                tolerance: f32::NAN,
                ..opts()
            }
        )
        .is_err());
        assert!(trace_mesh(
            &a,
            40,
            40,
            &TraceOptions {
                grading: 0.0,
                ..opts()
            }
        )
        .is_err());
        let bad_s = SmoothOptions {
            iterations: 1,
            lambda: 0.0,
            ..Default::default()
        };
        assert!(trace_mesh(
            &a,
            40,
            40,
            &TraceOptions {
                smooth: bad_s,
                ..opts()
            }
        )
        .is_err());
        let hs = Hotspot {
            pos: Vec2::new(f32::NAN, 0.0),
            radius: 1.0,
        };
        assert!(trace_mesh(
            &a,
            40,
            40,
            &TraceOptions {
                hotspots: vec![hs],
                ..opts()
            }
        )
        .is_err());
        // so many vertices that the cap triggers
        let big = disc(400, 400, (200.0, 200.0), 190.0, 0.0);
        assert!(trace_mesh(
            &big,
            400,
            400,
            &TraceOptions {
                density: 1.0,
                edge_size: 1.0,
                ..Default::default()
            }
        )
        .is_err());
        // smoothing an empty mesh is a no-op
        let mut e = TracedMesh {
            vertices: vec![],
            triangles: vec![],
            boundary: vec![],
            unresolved_edges: 0,
        };
        smooth_mesh(
            &mut e,
            &SmoothOptions {
                iterations: 3,
                ..Default::default()
            },
        );
        assert_eq!(mesh_stats(&e).triangles, 0);
    }

    #[test]
    fn delaunay_is_valid_for_degenerate_points() {
        let mut dt = Delaunay::new([0.0, 0.0], [10.0, 10.0]);
        // collinear, duplicate and grid points (cocircular)
        let mut ids = Vec::new();
        for i in 0..5 {
            ids.push(dt.insert([i as f64, 0.0]));
        }
        assert_eq!(
            dt.insert([2.0, 0.0]),
            ids[2],
            "duplicate returns the existing vertex"
        );
        assert_eq!(dt.insert([f64::NAN, 0.0]), None);
        for y in 1..5 {
            for x in 0..5 {
                dt.insert([x as f64, y as f64]);
            }
        }
        let alive: Vec<_> = dt.tris.iter().filter(|t| t.alive).collect();
        assert!(!alive.is_empty());
        for t in &alive {
            assert!(
                orient(dt.pts[t.v[0]], dt.pts[t.v[1]], dt.pts[t.v[2]]) > 0.0,
                "inverted triangle"
            );
        }
        // adjacency is symmetric
        for (i, t) in dt.tris.iter().enumerate().filter(|(_, t)| t.alive) {
            for k in 0..3 {
                if t.n[k] != NONE {
                    assert!(dt.tris[t.n[k]].alive && dt.tris[t.n[k]].n.contains(&i));
                }
            }
        }
    }

    // ---- project / engine integration ----

    fn png_of(a: &[u8], w: usize, h: usize) -> Vec<u8> {
        let mut pm = tiny_skia::Pixmap::new(w as u32, h as u32).unwrap();
        for (i, px) in pm.pixels_mut().iter_mut().enumerate() {
            let v = a[i];
            *px = tiny_skia::PremultipliedColorU8::from_rgba(v / 2, v / 3, v / 4, v).unwrap();
        }
        pm.encode_png().unwrap()
    }

    fn rigged() -> Project {
        use crate::project::Layer;
        let mut p = Project::new("t", 200.0, 200.0, 24.0, 24);
        let a = disc(160, 160, (80.0, 80.0), 70.0, 22.0);
        p.layers.push(Layer::image_from_png(
            "pic",
            &png_of(&a, 160, 160),
            Vec2::new(20.0, 20.0),
        ));
        p.bones.push(crate::project::Bone::new(
            "root",
            None,
            80.0,
            Vec2::new(20.0, 100.0),
            0.0,
        ));
        p.bones.push(crate::project::Bone::new(
            "tip",
            Some("root"),
            80.0,
            Vec2::new(80.0, 0.0),
            0.0,
        ));
        p
    }

    fn pic_mesh(f: &crate::engine::Frame) -> crate::engine::MeshOut {
        f.nodes
            .iter()
            .find(|n| n.name == "pic")
            .and_then(|n| n.image.as_ref())
            .and_then(|i| i.mesh.clone())
            .expect("mesh")
    }

    #[test]
    fn layer_integration_free_mesh_deforms_with_bones_and_round_trips() {
        let mut p = rigged();
        let st =
            trace_layer_mesh(&mut p, "pic", &opts(), std::path::Path::new("."), true, 4).unwrap();
        assert!(st.triangles > 50 && st.min_quality > 0.0);
        let LayerKind::Image {
            mesh: Some(m),
            size,
            ..
        } = &p.layers[0].kind
        else {
            panic!()
        };
        let f = m.free.as_ref().unwrap();
        assert_eq!(
            m.weights.len(),
            f.vertices.len(),
            "one weight list per free vertex"
        );
        assert!(m.weights.iter().all(|w| !w.is_empty()));
        assert_eq!(*size, Some(Vec2::new(160.0, 160.0)));
        // layer space = origin + pixels
        assert!(f.vertices.iter().all(|v| v.x >= 20.0 && v.x <= 180.0));
        assert_eq!(f.uvs.len(), f.vertices.len());
        // JSON round trip, and an old project (no `free`) still loads
        let json = serde_json::to_string(&p).unwrap();
        let back: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
        let old: MeshSpec = serde_json::from_str(r#"{"cols":3,"rows":2}"#).unwrap();
        assert!(old.free.is_none() && old.weights.is_empty());
        assert!(!serde_json::to_string(&old).unwrap().contains("free"));

        // engine: rest frame reproduces the rest mesh; posing a bone moves the vertices
        let e = crate::engine::Engine::new(p.clone()).unwrap();
        let rest = e.evaluate(0.0);
        let mesh0 = pic_mesh(&rest);
        assert_eq!(mesh0.positions.len(), f.vertices.len());
        assert_eq!(mesh0.tris.len(), f.triangles.len());
        for (a, b) in mesh0.positions.iter().zip(&f.vertices) {
            assert!(a.distance(*b) < 0.05);
        }
        let mut q = p.clone();
        let tr = crate::anim::Track::with_keys(&[
            (0.0, 0.0, crate::anim::Ease::Linear),
            (12.0, 60.0, crate::anim::Ease::Linear),
        ]);
        q.channels.push(crate::project::Channel::keys(
            "tip",
            crate::project::Prop::BoneAngle,
            tr,
        ));
        let e2 = crate::engine::Engine::new(q).unwrap();
        let posed = e2.evaluate(12.0);
        let mesh1 = pic_mesh(&posed);
        let moved = mesh1
            .positions
            .iter()
            .zip(&f.vertices)
            .filter(|(a, b)| a.distance(**b) > 5.0)
            .count();
        assert!(moved > f.vertices.len() / 5, "only {moved} vertices moved");
        assert!(mesh1
            .positions
            .iter()
            .all(|v| v.x.is_finite() && v.y.is_finite()));
    }

    #[test]
    fn layer_errors() {
        let mut p = rigged();
        assert!(
            trace_layer_mesh(&mut p, "nope", &opts(), std::path::Path::new("."), false, 4)
                .unwrap_err()
                .contains("no layer")
        );
        p.layers.push(crate::project::Layer::group("g", vec![]));
        assert!(
            trace_layer_mesh(&mut p, "g", &opts(), std::path::Path::new("."), false, 4)
                .unwrap_err()
                .contains("not an image")
        );
        let mut q = rigged();
        if let LayerKind::Image { source, .. } = &mut q.layers[0].kind {
            *source = ImageSource {
                file: Some("missing.png".into()),
                png_base64: None,
            };
        }
        assert!(
            trace_layer_mesh(&mut q, "pic", &opts(), std::path::Path::new("."), false, 4).is_err()
        );
        // the engine rejects broken free meshes with errors, not panics
        let mut r = rigged();
        trace_layer_mesh(&mut r, "pic", &opts(), std::path::Path::new("."), false, 4).unwrap();
        let mutate = |f: &dyn Fn(&mut FreeMesh)| {
            let mut x = r.clone();
            if let LayerKind::Image { mesh: Some(m), .. } = &mut x.layers[0].kind {
                f(m.free.as_mut().unwrap());
            }
            crate::engine::Engine::new(x).err()
        };
        assert!(mutate(&|f| f.triangles.push([0, 1, 99999]))
            .unwrap()
            .join(";")
            .contains("missing vertex"));
        assert!(mutate(&|f| f.vertices[0] = Vec2::new(f32::NAN, 0.0)).is_some());
        assert!(mutate(&|f| f.uvs.truncate(3)).is_some());
        assert!(mutate(&|f| {
            f.triangles.clear();
        })
        .is_some());
        assert!(mutate(&|_| {}).is_none());
        // wrong weight count
        let mut x = r.clone();
        if let LayerKind::Image { mesh: Some(m), .. } = &mut x.layers[0].kind {
            m.weights = vec![vec![]; 3];
        }
        assert!(crate::engine::Engine::new(x).is_err());
    }

    #[test]
    fn bone_hotspots_follow_the_rest_pose() {
        let p = rigged();
        let hs = bone_hotspots(&p, 10.0).unwrap();
        assert!(hs.len() > 10);
        assert!(hs
            .iter()
            .any(|h| h.pos.distance(Vec2::new(20.0, 100.0)) < 1e-3));
        assert!(hs
            .iter()
            .any(|h| h.pos.distance(Vec2::new(180.0, 100.0)) < 1e-3));
        assert!(bone_hotspots(&p, -1.0).is_err());
    }
}
