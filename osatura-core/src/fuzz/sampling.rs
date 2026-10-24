// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `fuzz` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// sampling strategies
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    Random,
    Lhs,
    Halton,
    Sweep,
    Corners,
    Pca,
}

impl Strategy {
    pub fn parse(s: &str) -> Result<Strategy, String> {
        Ok(match s {
            "random" => Strategy::Random,
            "lhs" => Strategy::Lhs,
            "halton" => Strategy::Halton,
            "sweep" => Strategy::Sweep,
            "corners" => Strategy::Corners,
            "pca" => Strategy::Pca,
            _ => return Err(format!("unknown strategy '{s}' (random, lhs, halton, sweep, corners, pca)")),
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Strategy::Random => "random",
            Strategy::Lhs => "lhs",
            Strategy::Halton => "halton",
            Strategy::Sweep => "sweep",
            Strategy::Corners => "corners",
            Strategy::Pca => "pca",
        }
    }
}

/// Latin hypercube in the unit cube: every dimension is cut into `n` equal strata and each stratum is hit exactly once.
pub fn latin_hypercube(n: usize, dim: usize, rng: &mut Rng) -> Vec<Vec<f32>> {
    let mut pts = vec![vec![0.0f32; dim]; n];
    for d in 0..dim {
        let mut perm: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            perm.swap(i, rng.below(i + 1));
        }
        for (i, p) in pts.iter_mut().enumerate() {
            p[d] = (perm[i] as f32 + rng.unit()) / n as f32;
        }
    }
    pts
}

/// A pose space fitted on a pose database, restricted to the bones that are parameters of the rig.
pub struct PcaSampler {
    pub(super) space: PoseSpace,
    /// For each database column: index of the matching `bone:NAME:angle` parameter.
    pub(super) cols: Vec<usize>,
}

impl PcaSampler {
    pub fn new(db: &PoseDb, params: &[Param]) -> Result<PcaSampler, String> {
        db.validate()?;
        let mut cols = Vec::new();
        let mut src = Vec::new();
        for (j, b) in db.bones.iter().enumerate() {
            if let Some(pi) = params.iter().position(|p| p.id == format!("bone:{b}:angle")) {
                cols.push(pi);
                src.push(j);
            }
        }
        if cols.is_empty() {
            return Err("the pose database shares no bone with the rig's angle parameters".into());
        }
        let poses: Vec<Vec<f32>> = db.poses.iter().map(|p| src.iter().map(|&j| p[j]).collect()).collect();
        let space = PoseSpace::fit(&poses, 0.95, 8)?;
        Ok(PcaSampler { space, cols })
    }
    pub fn components(&self) -> usize {
        self.space.k()
    }
}

/// Most samples one fuzz run may ask for.
pub const MAX_SAMPLES: usize = 1_000_000;

/// Most numbers (samples x parameters) one run may generate at once.
pub const MAX_SAMPLE_VALUES: usize = 50_000_000;

/// Parameter vectors for a strategy. `n` is the requested number of samples (sweep and corners may return
/// a different count, see their notes). Deterministic for a given `seed`.
pub fn generate(params: &[Param], strategy: Strategy, n: usize, seed: u64, pca: Option<&PcaSampler>) -> Result<Vec<Vec<f32>>, String> {
    let d = params.len();
    if d == 0 {
        return Err("the rig has no parameters to fuzz".into());
    }
    if n == 0 {
        return Err("--samples must be at least 1".into());
    }
    if n > MAX_SAMPLES {
        return Err(format!("--samples {n} is more than the limit of {MAX_SAMPLES}"));
    }
    if n.saturating_mul(d) > MAX_SAMPLE_VALUES {
        return Err(format!("--samples {n} x {d} parameters = {} numbers, more than the limit of {MAX_SAMPLE_VALUES}; use fewer samples", n.saturating_mul(d)));
    }
    let mut rng = Rng::new(seed);
    let map = |u: &[f32]| -> Vec<f32> { params.iter().zip(u).map(|(p, &u)| p.lo + u * (p.hi - p.lo)).collect() };
    Ok(match strategy {
        Strategy::Random => (0..n).map(|_| map(&(0..d).map(|_| rng.unit()).collect::<Vec<_>>())).collect(),
        Strategy::Lhs => latin_hypercube(n, d, &mut rng).iter().map(|u| map(u)).collect(),
        Strategy::Halton => (1..=n as u64).map(|i| map(&halton_point(i, d))).collect(),
        Strategy::Sweep => {
            // one axis at a time, others at rest; both ends of the range included
            let m = (n / d).max(2);
            let mut out = Vec::new();
            for (k, p) in params.iter().enumerate() {
                for s in 0..m {
                    let mut v: Vec<f32> = params.iter().map(|q| q.rest).collect();
                    v[k] = p.lo + (p.hi - p.lo) * s as f32 / (m - 1) as f32;
                    out.push(v);
                }
            }
            out
        }
        Strategy::Corners => {
            // all 2^d corners when they fit into n, else n random corners
            if d <= 20 && (1usize << d) <= n {
                (0..1usize << d).map(|m| params.iter().enumerate().map(|(k, p)| if m >> k & 1 == 1 { p.hi } else { p.lo }).collect()).collect()
            } else {
                (0..n).map(|_| params.iter().map(|p| if rng.next_u64() & 1 == 1 { p.hi } else { p.lo }).collect()).collect()
            }
        }
        Strategy::Pca => {
            let s = pca.ok_or("the pca strategy needs a pose database (--db) or keyed bone rotations in the project")?;
            let k = s.space.k();
            (0..n)
                .map(|_| {
                    let z: Vec<f32> = (0..k).map(|i| (rng.gauss() * 2.0).clamp(-3.0, 3.0) * s.space.eig[i].sqrt()).collect();
                    let x = s.space.reconstruct(&z);
                    let mut v: Vec<f32> = params.iter().map(|q| q.rest).collect();
                    for (c, &pi) in s.cols.iter().enumerate() {
                        v[pi] = x[c].clamp(params[pi].lo, params[pi].hi);
                    }
                    v
                })
                .collect()
        }
    })
}

// ---------------------------------------------------------------------------------------------
// geometry metrics
// ---------------------------------------------------------------------------------------------

/// Failure thresholds. A metric at its threshold has normalised value 1.
#[derive(Clone, Debug, Serialize)]
pub struct Thresholds {
    /// A triangle with `det F <= min_det` counts as folded (0 = flipped or collapsed).
    pub min_det: f32,
    /// Largest allowed edge length ratio vs rest (growth or shrink).
    pub max_stretch: f32,
    /// Largest allowed `smax/smin - 1` of the deformation gradient (non-uniform stretch / shear).
    pub max_shear: f32,
    /// Largest allowed area growth `det F`.
    pub max_area: f32,
    /// Largest allowed spike (growth of the Laplacian offset in local edge lengths).
    pub max_spike: f32,
    /// IK end-effector error as a fraction of chain reach.
    pub ik_error: f32,
    /// A layer is lost when its box is farther than this many canvas sizes outside the canvas.
    pub canvas_margin: f32,
    /// A layer is exploded when its box is larger than this many canvas diagonals.
    pub max_extent: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Thresholds { min_det: 0.0, max_stretch: 3.0, max_shear: 2.0, max_area: 6.0, max_spike: 1.0, ik_error: 0.03, canvas_margin: 0.0, max_extent: 20.0 }
    }
}

pub(super) const CAP: f32 = 100.0;

/// Aggregated result of [`mesh_metrics`].
#[derive(Clone, Debug, Serialize)]
pub struct MeshMetrics {
    pub triangles: usize,
    pub min_det: f32,
    pub max_det: f32,
    pub flips: usize,
    /// Largest edge length ratio vs rest (>= 1; growth or shrink, whichever is larger).
    pub max_stretch: f32,
    /// Largest `smax/smin - 1`.
    pub max_shear: f32,
    pub max_spike: f32,
    /// Normalised per-vertex stress (max over incident elements and spike); 1 = at a threshold.
    #[serde(skip)]
    pub stress: Vec<f32>,
}

pub(super) fn det2(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

/// Deformation-gradient metrics of a triangle mesh against its rest shape. `edges` may add extra edges
/// (open or closed paths use them instead of triangles). Triangles whose rest area is ~0 are skipped.
pub fn mesh_metrics(rest: &[Vec2], def: &[Vec2], tris: &[[usize; 3]], edges: &[[usize; 2]], th: &Thresholds) -> MeshMetrics {
    let n = rest.len().min(def.len());
    let mut stress = vec![0.0f32; n];
    let mut m = MeshMetrics { triangles: 0, min_det: f32::INFINITY, max_det: f32::NEG_INFINITY, flips: 0, max_stretch: 1.0, max_shear: 0.0, max_spike: 0.0, stress: vec![] };
    let mut all_edges: Vec<[usize; 2]> = edges.to_vec();
    for t in tris {
        all_edges.extend([[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]]);
    }
    let mean_rest = {
        let (mut s, mut c) = (0.0f32, 0usize);
        for e in &all_edges {
            if e[0] < n && e[1] < n {
                s += rest[e[0]].distance(rest[e[1]]);
                c += 1;
            }
        }
        if c == 0 { 0.0 } else { s / c as f32 }
    };
    for t in tris {
        if t.iter().any(|&i| i >= n) {
            continue;
        }
        let (r0, r1, r2) = (rest[t[0]], rest[t[1]], rest[t[2]]);
        let (e1, e2) = (r1 - r0, r2 - r0);
        let a_rest = det2(e1, e2);
        if a_rest.abs() < 1e-9 * (mean_rest * mean_rest).max(1e-12) {
            continue;
        }
        let (d1, d2) = (def[t[1]] - def[t[0]], def[t[2]] - def[t[0]]);
        let det = det2(d1, d2) / a_rest;
        // F = [d1 d2] [e1 e2]^-1
        let inv = [[e2.y / a_rest, -e2.x / a_rest], [-e1.y / a_rest, e1.x / a_rest]];
        let f = [[d1.x * inv[0][0] + d2.x * inv[1][0], d1.x * inv[0][1] + d2.x * inv[1][1]], [d1.y * inv[0][0] + d2.y * inv[1][0], d1.y * inv[0][1] + d2.y * inv[1][1]]];
        let s1 = f[0][0] * f[0][0] + f[0][1] * f[0][1] + f[1][0] * f[1][0] + f[1][1] * f[1][1];
        let disc = (s1 * s1 - 4.0 * det * det).max(0.0).sqrt();
        let smax = ((s1 + disc) * 0.5).sqrt();
        let smin = if smax > 1e-12 { det.abs() / smax } else { 0.0 };
        let shear = if det <= th.min_det { 0.0 } else if smin > 1e-9 { smax / smin - 1.0 } else { 1e3 };
        m.triangles += 1;
        m.min_det = m.min_det.min(det);
        m.max_det = m.max_det.max(det);
        if det <= th.min_det {
            m.flips += 1;
        }
        m.max_shear = m.max_shear.max(shear);
        let n_fold = if det <= th.min_det { 1.0 + (th.min_det - det) } else { (1.0 - det) / (1.0 - th.min_det) }.max(0.0);
        let n_area = if det > 1.0 { (det - 1.0) / (th.max_area - 1.0) } else { 0.0 };
        let n_shear = shear / th.max_shear;
        let s = n_fold.max(n_area).max(n_shear).min(CAP);
        for &i in t {
            stress[i] = stress[i].max(s);
        }
    }
    for e in &all_edges {
        if e[0] >= n || e[1] >= n {
            continue;
        }
        let l0 = rest[e[0]].distance(rest[e[1]]);
        if l0 < 1e-6 {
            continue;
        }
        let l = def[e[0]].distance(def[e[1]]);
        let ratio = if l < 1e-9 { 1e3 } else { (l / l0).max(l0 / l) };
        m.max_stretch = m.max_stretch.max(ratio);
        let s = ((ratio - 1.0) / (th.max_stretch - 1.0)).min(CAP);
        for &i in e {
            stress[i] = stress[i].max(s);
        }
    }
    // spikes: growth of the Laplacian offset, measured in local edge lengths (similarity invariant)
    let mut nbr: Vec<Vec<usize>> = vec![vec![]; n];
    for e in &all_edges {
        if e[0] < n && e[1] < n && e[0] != e[1] {
            if !nbr[e[0]].contains(&e[1]) {
                nbr[e[0]].push(e[1]);
            }
            if !nbr[e[1]].contains(&e[0]) {
                nbr[e[1]].push(e[0]);
            }
        }
    }
    for i in 0..n {
        if nbr[i].len() < 2 {
            continue;
        }
        let k = nbr[i].len() as f32;
        let (mut cr, mut cd, mut lr, mut ld) = (Vec2::ZERO, Vec2::ZERO, 0.0f32, 0.0f32);
        for &j in &nbr[i] {
            cr += rest[j];
            cd += def[j];
            lr += rest[i].distance(rest[j]);
            ld += def[i].distance(def[j]);
        }
        let (lap_r, lap_d) = ((cr * (1.0 / k) - rest[i]).length(), (cd * (1.0 / k) - def[i]).length());
        let (er, ed) = (lr / k, ld / k);
        if er < 1e-6 {
            continue;
        }
        let spike = (lap_d / ed.max(1e-9) - lap_r / er).max(0.0);
        m.max_spike = m.max_spike.max(spike);
        stress[i] = stress[i].max((spike / th.max_spike).min(CAP));
    }
    if m.triangles == 0 {
        m.min_det = 1.0;
        m.max_det = 1.0;
    }
    m.stress = stress;
    m
}

pub(super) fn orient(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    det2(b - a, c - a)
}

/// Do segments `ab` and `cd` cross properly (interiors intersect, not merely touch)?
pub fn segments_cross(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let (o1, o2, o3, o4) = (orient(a, b, c), orient(a, b, d), orient(c, d, a), orient(c, d, b));
    o1 * o2 < 0.0 && o3 * o4 < 0.0
}

/// Pairs of non-adjacent segments of a polyline that cross each other (x-sweep over bounding boxes; at most `cap` pairs).
pub fn self_intersections(pts: &[Vec2], closed: bool, cap: usize) -> Vec<(usize, usize)> {
    let n = pts.len();
    let segs = if closed { n } else { n.saturating_sub(1) };
    if segs < 3 {
        return vec![];
    }
    let seg = |i: usize| (pts[i], pts[(i + 1) % n]);
    let mut order: Vec<usize> = (0..segs).collect();
    let xmin = |i: usize| {
        let (a, b) = seg(i);
        a.x.min(b.x)
    };
    let xmax = |i: usize| {
        let (a, b) = seg(i);
        a.x.max(b.x)
    };
    order.sort_by(|&a, &b| xmin(a).total_cmp(&xmin(b)));
    let mut out = Vec::new();
    for (oi, &i) in order.iter().enumerate() {
        let (a, b) = seg(i);
        for &j in &order[oi + 1..] {
            if xmin(j) > xmax(i) {
                break;
            }
            let adjacent = (i + 1) % segs == j || (j + 1) % segs == i;
            if !closed && (i + 1 == j || j + 1 == i) || closed && adjacent {
                continue;
            }
            let (c, d) = seg(j);
            if a.y.max(b.y) < c.y.min(d.y) || c.y.max(d.y) < a.y.min(b.y) {
                continue;
            }
            if segments_cross(a, b, c, d) {
                out.push((i.min(j), i.max(j)));
                if out.len() >= cap {
                    return out;
                }
            }
        }
    }
    out
}

/// Signed area of a closed polygon (shoelace).
pub fn signed_area(pts: &[Vec2]) -> f32 {
    let n = pts.len();
    (0..n).map(|i| det2(pts[i], pts[(i + 1) % n])).sum::<f32>() * 0.5
}
