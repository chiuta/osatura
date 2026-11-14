// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `fuzz` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// snapshots of an evaluated frame
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeoKind {
    Mesh,
    Path,
    Quad,
}

/// Deformed geometry of one node (layer space) plus the data needed to judge it.
#[derive(Clone, Debug)]
pub(super) struct Snap {
    pub(super) name: String,
    pub(super) kind: GeoKind,
    pub(super) pts: Vec<Vec2>,
    pub(super) tris: Vec<[usize; 3]>,
    pub(super) edges: Vec<[usize; 2]>,
    /// Flattened outlines with, per point, the anchor index it belongs to (paths only).
    pub(super) outlines: Vec<(Vec<Vec2>, Vec<usize>, bool)>,
    /// Closed polygon on `pts` for the orientation test (paths with fill only; empty otherwise).
    pub(super) loops: Vec<(usize, usize)>,
    pub(super) world: Affine2,
}

pub(super) fn snapshot(frame: &Frame) -> Vec<Snap> {
    let mut out = Vec::new();
    for n in &frame.nodes {
        if !n.visible || n.opacity <= 0.0 {
            continue;
        }
        match n.kind {
            NodeKind::Vector => {
                for (pi, p) in n.paths.iter().enumerate() {
                    let pts: Vec<Vec2> = p.anchors.iter().map(|a| a.pos).collect();
                    if pts.is_empty() {
                        continue;
                    }
                    let k = pts.len();
                    let mut edges = Vec::new();
                    for i in 0..k.saturating_sub(1) {
                        edges.push([i, i + 1]);
                    }
                    let mut loops = vec![];
                    if p.closed && k > 2 {
                        edges.push([k - 1, 0]);
                        loops.push((0, k));
                    }
                    let fl = p.flatten_t(1.0);
                    let outline = (fl.iter().map(|f| f.0).collect::<Vec<_>>(), fl.iter().map(|f| f.1.min(k - 1)).collect::<Vec<_>>(), p.closed);
                    out.push(Snap { name: format!("{}#{}", n.name, if p.name.is_empty() { pi.to_string() } else { p.name.clone() }), kind: GeoKind::Path, pts, tris: vec![], edges, outlines: vec![outline], loops, world: n.world });
                }
            }
            NodeKind::Image => {
                if let Some(img) = &n.image {
                    match &img.mesh {
                        Some(m) => {
                            let tris = m.tris.iter().map(|t| [t[0] as usize, t[1] as usize, t[2] as usize]).collect();
                            out.push(Snap { name: format!("{}#mesh", n.name), kind: GeoKind::Mesh, pts: m.positions.clone(), tris, edges: vec![], outlines: vec![], loops: vec![], world: n.world });
                        }
                        None => {
                            let (o, s) = (img.origin, img.size);
                            let pts = vec![o, Vec2::new(o.x + s.x, o.y), o + s, Vec2::new(o.x, o.y + s.y)];
                            out.push(Snap { name: format!("{}#quad", n.name), kind: GeoKind::Quad, pts, tris: vec![[0, 1, 2], [0, 2, 3]], edges: vec![], outlines: vec![], loops: vec![], world: n.world });
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

pub(super) fn bbox(pts: &[Vec2], m: &Affine2) -> Option<(Vec2, Vec2)> {
    let mut it = pts.iter().map(|p| m.apply(*p));
    let f = it.next()?;
    let (mut lo, mut hi) = (f, f);
    for p in it {
        lo = Vec2::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Vec2::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    Some((lo, hi))
}

// ---------------------------------------------------------------------------------------------
// per-sample evaluation
// ---------------------------------------------------------------------------------------------

/// Normalised metrics (1 = at the threshold).
#[derive(Clone, Debug, Default, Serialize)]
pub struct Norm {
    pub fold: f32,
    pub stretch: f32,
    pub shear: f32,
    pub area: f32,
    pub spike: f32,
    pub self_intersection: f32,
    pub ik_unreachable: f32,
    pub limit: f32,
    pub off_canvas: f32,
    pub non_finite: f32,
    pub panic: f32,
}

impl Norm {
    pub fn items(&self) -> [(&'static str, f32); 11] {
        [
            ("fold", self.fold),
            ("stretch", self.stretch),
            ("shear", self.shear),
            ("area", self.area),
            ("spike", self.spike),
            ("self_intersection", self.self_intersection),
            ("ik_unreachable", self.ik_unreachable),
            ("limit", self.limit),
            ("off_canvas", self.off_canvas),
            ("non_finite", self.non_finite),
            ("panic", self.panic),
        ]
    }
}

/// Raw metric values of one sample.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SampleMetrics {
    pub min_det: f32,
    pub max_area_ratio: f32,
    pub flipped_triangles: usize,
    pub max_edge_stretch: f32,
    pub max_shear: f32,
    pub max_spike: f32,
    pub new_self_intersections: usize,
    pub ik_error_fraction: f32,
    pub ik_worst_chain: Option<String>,
    pub limit_violations: usize,
    pub layers_off_canvas: usize,
    pub layers_exploded: usize,
    pub non_finite: bool,
    pub panicked: bool,
    pub norm: Norm,
}

/// Verdict for one sampled pose.
#[derive(Clone, Debug, Serialize)]
pub struct SampleResult {
    pub index: usize,
    pub values: Vec<f32>,
    pub severity: f32,
    pub failed: bool,
    /// Normalised metrics at or above 1, worst first.
    pub kinds: Vec<String>,
    pub metrics: SampleMetrics,
}

pub(super) struct EvalOut {
    pub(super) result: SampleResult,
    /// Normalised stress per vertex for each rest geometry (same order as the fuzzer's `geos`).
    pub(super) stress: Vec<Vec<f32>>,
    /// Worst self-intersecting anchor per path, merged into `stress` already.
    #[allow(dead_code)]
    pub(super) _pad: (),
}

pub(super) struct Geo {
    pub(super) snap: Snap,
    pub(super) rest_self: Vec<usize>,
    pub(super) rest_area: Vec<f32>,
    pub(super) rest_inside: bool,
}

/// A prepared project: probe engine, rest geometry and thresholds. Reusable for many samples.
pub struct Fuzzer {
    pub params: Vec<Param>,
    pub thresholds: Thresholds,
    pub frame: f32,
    /// Failure kinds that are measured but never count (e.g. `ik_unreachable`, `limit`).
    pub ignore: Vec<String>,
    pub(super) engine: Engine,
    pub(super) geos: Vec<Geo>,
    pub(super) canvas: (f32, f32),
    /// Bone world positions at rest, for the overlay: `(name, head, tip)` in stage space.
    pub(super) rest_bones: Vec<(String, Vec2, Vec2)>,
    pub(super) evals: usize,
}

pub(super) fn engine_for(p: Project, asset_dir: Option<&Path>) -> Result<Engine, String> {
    let r = match asset_dir {
        Some(d) => Engine::with_asset_dir(p, d),
        None => Engine::new(p),
    };
    r.map_err(|e| format!("project does not compile: {}", e.join("; ")))
}

impl Fuzzer {
    pub fn new(project: &Project, asset_dir: Option<&Path>, popts: &ParamOptions, th: Thresholds, frame: f32) -> Result<Fuzzer, String> {
        let params = enumerate_params(project, popts);
        if params.is_empty() {
            return Err("the rig has no parameters to fuzz".into());
        }
        Fuzzer::with_params(project, asset_dir, params, th, frame)
    }

    pub fn with_params(project: &Project, asset_dir: Option<&Path>, params: Vec<Param>, th: Thresholds, frame: f32) -> Result<Fuzzer, String> {
        if !frame.is_finite() {
            return Err("frame must be a number".into());
        }
        // rest reference: no animation at all
        let mut rest_p = project.clone();
        rest_p.channels.clear();
        rest_p.clip_baked.clear();
        rest_p.channel_meta.clear();
        let rest_engine = engine_for(rest_p, asset_dir)?;
        let rest_frame = rest_engine.evaluate(frame);
        let snaps = snapshot(&rest_frame);
        let (w, h) = (project.width, project.height);
        let geos: Vec<Geo> = snaps
            .into_iter()
            .map(|s| {
                let rest_self = s.outlines.iter().map(|(p, _, c)| self_intersections(p, *c, 1000).len()).collect();
                let rest_area = s.loops.iter().map(|&(a, b)| signed_area(&s.pts[a..b])).collect();
                let rest_inside = bbox(&s.pts, &s.world).is_some_and(|(lo, hi)| hi.x >= 0.0 && lo.x <= w && hi.y >= 0.0 && lo.y <= h);
                Geo { snap: s, rest_self, rest_area, rest_inside }
            })
            .collect();
        let rest_bones = project
            .bones
            .iter()
            .zip(&rest_frame.world)
            .map(|(b, m)| (b.name.clone(), m.apply(Vec2::ZERO), m.apply(Vec2::new(b.length, 0.0))))
            .collect();
        // probe engine
        let mut pp = project.clone();
        pp.channel_meta.clear();
        pp.channels.retain(|c| !params.iter().any(|q| q.target == c.target && q.prop == c.prop));
        let mut probes: Vec<Channel> = params
            .iter()
            .map(|q| Channel::keys(&q.target, q.prop, Track::with_keys(&[(0.0, q.rest, Ease::Step)])).with_mode(Mode::Replace))
            .collect();
        probes.append(&mut pp.channels);
        pp.channels = probes;
        let engine = engine_for(pp, asset_dir)?;
        Ok(Fuzzer { params, thresholds: th, frame, ignore: vec![], engine, geos, canvas: (w, h), rest_bones, evals: 0 })
    }

    pub fn rest_values(&self) -> Vec<f32> {
        self.params.iter().map(|p| p.rest).collect()
    }

    /// Number of samples evaluated so far.
    pub fn evaluations(&self) -> usize {
        self.evals
    }

    pub fn eval(&mut self, index: usize, values: &[f32]) -> SampleResult {
        self.eval_full(index, values).result
    }

    pub(super) fn eval_full(&mut self, index: usize, values: &[f32]) -> EvalOut {
        self.evals += 1;
        let n_geo = self.geos.len();
        let mut norm = Norm::default();
        let mut mt = SampleMetrics { min_det: 1.0, max_area_ratio: 1.0, max_edge_stretch: 1.0, ..Default::default() };
        let mut stress: Vec<Vec<f32>> = self.geos.iter().map(|g| vec![0.0; g.snap.pts.len()]).collect();
        // set probes
        for (i, v) in values.iter().enumerate().take(self.params.len()) {
            if let Source::Keys { track } = &mut self.engine.project.channels[i].source {
                track.keys[0].value = *v;
            }
        }
        mt.non_finite = values.iter().any(|v| !v.is_finite());
        for (q, v) in self.params.iter().zip(values) {
            if let Some([a, b]) = q.limit {
                if *v < a - 1e-3 || *v > b + 1e-3 {
                    mt.limit_violations += 1;
                }
            }
        }
        norm.limit = if mt.limit_violations > 0 { 1.0 } else { 0.0 };
        if mt.non_finite {
            norm.non_finite = CAP;
            return self.finish(index, values, mt, norm, stress);
        }
        let frame_no = self.frame;
        let eng = &self.engine;
        let fr = match catch_unwind(AssertUnwindSafe(|| eng.evaluate(frame_no))) {
            Ok(f) => f,
            Err(_) => {
                mt.panicked = true;
                norm.panic = CAP;
                return self.finish(index, values, mt, norm, stress);
            }
        };
        if fr.world.iter().any(|m| ![m.a, m.b, m.c, m.d, m.tx, m.ty].iter().all(|x| x.is_finite())) {
            mt.non_finite = true;
        }
        // IK
        let p = &self.engine.project;
        for c in &p.ik {
            let mix = self.params.iter().zip(values).find(|(q, _)| q.prop == Prop::IkMix && q.target == c.name).map_or(c.mix, |(_, v)| *v);
            if mix < 0.05 {
                continue;
            }
            let ix = |n: &str| p.bones.iter().position(|b| b.name == n);
            let (Some(first), Some(last), Some(tg)) = (c.bones.first().and_then(|n| ix(n)), c.bones.last().and_then(|n| ix(n)), ix(&c.target)) else { continue };
            if first >= fr.world.len() || last >= fr.world.len() || tg >= fr.world.len() {
                continue;
            }
            let tip = fr.world[last].apply(Vec2::new(p.bones[last].length, 0.0));
            let root = fr.world[first].apply(Vec2::ZERO);
            let target = fr.world[tg].apply(Vec2::ZERO);
            let mut reach = 0.0;
            for n in &c.bones {
                if let Some(bi) = ix(n) {
                    reach += (fr.world[bi].apply(Vec2::new(p.bones[bi].length, 0.0)) - fr.world[bi].apply(Vec2::ZERO)).length();
                }
            }
            let err = tip.distance(target);
            let frac = if reach > 1e-3 { err / reach } else { err };
            let _ = root;
            if !frac.is_finite() {
                mt.non_finite = true;
            } else if frac > mt.ik_error_fraction {
                mt.ik_error_fraction = frac;
                mt.ik_worst_chain = Some(c.name.clone());
            }
        }
        norm.ik_unreachable = (mt.ik_error_fraction / self.thresholds.ik_error).min(CAP);
        // geometry
        let snaps = snapshot(&fr);
        let th = self.thresholds.clone();
        let (cw, chh) = self.canvas;
        let diag = (cw * cw + chh * chh).sqrt();
        let mut by_name: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for (i, g) in self.geos.iter().enumerate() {
            by_name.entry(g.snap.name.as_str()).or_insert(i);
        }
        let (mut min_det, mut max_det) = (f32::INFINITY, f32::NEG_INFINITY);
        for s in &snaps {
            let Some(&gi) = by_name.get(s.name.as_str()) else { continue };
            let g = &self.geos[gi];
            if g.snap.pts.len() != s.pts.len() {
                continue; // topology changed (generated geometry): not comparable
            }
            if s.pts.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
                mt.non_finite = true;
                continue;
            }
            if let Some((lo, hi)) = bbox(&s.pts, &s.world) {
                let mx = th.canvas_margin * cw.max(chh);
                if g.rest_inside && (hi.x < -mx || lo.x > cw + mx || hi.y < -mx || lo.y > chh + mx) {
                    mt.layers_off_canvas += 1;
                }
                if (hi - lo).length() > th.max_extent * diag {
                    mt.layers_exploded += 1;
                }
            }
            let mm = mesh_metrics(&g.snap.pts, &s.pts, &g.snap.tris, &g.snap.edges, &th);
            if mm.triangles > 0 {
                min_det = min_det.min(mm.min_det);
                max_det = max_det.max(mm.max_det);
            }
            mt.flipped_triangles += mm.flips;
            mt.max_edge_stretch = mt.max_edge_stretch.max(mm.max_stretch);
            mt.max_shear = mt.max_shear.max(mm.max_shear);
            mt.max_spike = mt.max_spike.max(mm.max_spike);
            let st = &mut stress[gi];
            for (a, b) in st.iter_mut().zip(&mm.stress) {
                *a = a.max(*b);
            }
            // paths: outline crossings and orientation flips
            for (k, (poly, anchor_of, closed)) in s.outlines.iter().enumerate() {
                let rest_n = g.rest_self.get(k).copied().unwrap_or(0);
                let hits = self_intersections(poly, *closed, 1000);
                if hits.len() > rest_n {
                    mt.new_self_intersections += hits.len() - rest_n;
                    for (i, j) in &hits {
                        for sg in [*i, *j] {
                            if let Some(&a) = anchor_of.get(sg) {
                                if a < st.len() {
                                    st[a] = st[a].max(1.0 + 0.1 * (hits.len() - rest_n).min(50) as f32);
                                }
                            }
                        }
                    }
                }
            }
            for (k, &(a, b)) in g.snap.loops.iter().enumerate() {
                let (ra, da) = (g.rest_area[k], signed_area(&s.pts[a..b]));
                if ra.abs() > 1e-6 {
                    let ratio = da / ra;
                    min_det = min_det.min(ratio);
                    max_det = max_det.max(ratio);
                    if ratio <= th.min_det {
                        mt.flipped_triangles += 1;
                        for v in st[a..b].iter_mut() {
                            *v = v.max(1.0 + (th.min_det - ratio));
                        }
                    }
                }
            }
        }
        if min_det.is_finite() {
            mt.min_det = min_det;
            mt.max_area_ratio = max_det.max(1.0);
        }
        norm.fold = if mt.min_det <= th.min_det { 1.0 + (th.min_det - mt.min_det) } else { ((1.0 - mt.min_det) / (1.0 - th.min_det)).max(0.0) }.min(CAP);
        norm.stretch = ((mt.max_edge_stretch - 1.0) / (th.max_stretch - 1.0)).min(CAP);
        norm.shear = (mt.max_shear / th.max_shear).min(CAP);
        // not `clamp`: `f32::max/min` map a NaN ratio (0/0 area threshold) to a bound, `clamp` would keep NaN
        #[allow(clippy::manual_clamp)]
        {
            norm.area = ((mt.max_area_ratio - 1.0) / (th.max_area - 1.0)).max(0.0).min(CAP);
        }
        norm.spike = (mt.max_spike / th.max_spike).min(CAP);
        norm.self_intersection = if mt.new_self_intersections > 0 { 1.0 + 0.1 * mt.new_self_intersections.min(50) as f32 } else { 0.0 };
        norm.off_canvas = if mt.layers_off_canvas + mt.layers_exploded > 0 { 1.0 } else { 0.0 };
        if mt.non_finite {
            norm.non_finite = CAP;
        }
        let _ = n_geo;
        self.finish(index, values, mt, norm, stress)
    }

    pub(super) fn finish(&mut self, index: usize, values: &[f32], mut mt: SampleMetrics, norm: Norm, stress: Vec<Vec<f32>>) -> EvalOut {
        let mut items: Vec<(&'static str, f32)> = norm.items().to_vec();
        items.retain(|x| !self.ignore.iter().any(|i| i == x.0));
        let severity = items.iter().map(|x| x.1).fold(0.0f32, f32::max);
        items.retain(|x| x.1 >= 1.0);
        items.sort_by(|a, b| b.1.total_cmp(&a.1));
        let kinds = items.iter().map(|x| x.0.to_string()).collect();
        mt.norm = norm;
        EvalOut { result: SampleResult { index, values: values.to_vec(), severity, failed: severity >= 1.0, kinds, metrics: mt }, stress, _pad: () }
    }
}
