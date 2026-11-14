// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `fuzz` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// delta debugging
// ---------------------------------------------------------------------------------------------

/// Zeller's `ddmin`: a 1-minimal subset of `items` for which `fails` still returns true (`fails(items)` must be
/// true). Subsets are tried first, then complements, with the granularity doubling when nothing shrinks.
pub fn ddmin<T: Clone>(items: &[T], fails: &mut dyn FnMut(&[T]) -> bool) -> Vec<T> {
    let mut c: Vec<T> = items.to_vec();
    if c.len() == 1 && fails(&[]) {
        return vec![];
    }
    let mut n = 2usize;
    while c.len() >= 2 {
        let len = c.len();
        let mut reduced = false;
        // `n` is reassigned only immediately before `break`, so the range bound is never observed again
        #[allow(clippy::mut_range_bound)]
        for i in 0..n {
            let (a, b) = (i * len / n, (i + 1) * len / n);
            if a == b {
                continue;
            }
            let sub = c[a..b].to_vec();
            if sub.len() < len && fails(&sub) {
                c = sub;
                n = 2;
                reduced = true;
                break;
            }
        }
        if !reduced {
            // as above: `n` changes only right before `break`
            #[allow(clippy::mut_range_bound)]
            for i in 0..n {
                let (a, b) = (i * len / n, (i + 1) * len / n);
                if a == b {
                    continue;
                }
                let comp: Vec<T> = c[..a].iter().chain(c[b..].iter()).cloned().collect();
                if !comp.is_empty() && fails(&comp) {
                    c = comp;
                    n = (n - 1).max(2);
                    reduced = true;
                    break;
                }
            }
        }
        if !reduced {
            if n >= len {
                break;
            }
            n = (2 * n).min(len);
        }
    }
    c
}

/// Bisect the blend factor `t` between `rest` (passing) and `v` (failing) and return the smallest failing
/// value found (to within `tol` of the range). Assumes failure grows monotonically with `t`.
pub fn shrink_value(rest: f32, v: f32, tol: f32, fails: &mut dyn FnMut(f32) -> bool) -> f32 {
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    let span = (v - rest).abs().max(1e-12);
    for _ in 0..40 {
        if (hi - lo) * span <= tol {
            break;
        }
        let mid = 0.5 * (lo + hi);
        if fails(rest + mid * (v - rest)) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    rest + hi * (v - rest)
}

#[derive(Clone, Debug, Serialize)]
pub struct MinimalParam {
    pub id: String,
    pub value: f32,
    pub rest: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Minimal {
    pub kind: String,
    pub severity: f32,
    pub params: Vec<MinimalParam>,
    pub evaluations: usize,
    /// Samples at the minimal pose still fail (re-checked).
    pub verified: bool,
}

/// Shrink a failing pose: first to a 1-minimal set of parameters moved away from rest (ddmin), then each of
/// those towards rest by bisection. Failure means "the chosen kind (default: the sample's primary kind) is at or above its threshold".
pub fn minimize(f: &mut Fuzzer, values: &[f32], want: Option<&str>) -> Result<Minimal, String> {
    let start = f.evaluations();
    let rest = f.rest_values();
    let orig = f.eval(0, values);
    if !orig.failed {
        return Err("the pose to minimise does not fail".into());
    }
    let kind = match want {
        Some(k) if orig.kinds.iter().any(|x| x == k) => k.to_string(),
        Some(k) => return Err(format!("the pose does not fail with kind '{k}' (it fails with: {})", orig.kinds.join(", "))),
        None => orig.kinds.first().cloned().unwrap_or_default(),
    };
    let moved: Vec<usize> = (0..values.len()).filter(|&i| (values[i] - rest[i]).abs() > 1e-6).collect();
    let with = |set: &[usize], vals: &[f32]| -> Vec<f32> {
        let mut v = rest.clone();
        for &i in set {
            v[i] = vals[i];
        }
        v
    };
    let fails_kind = |f: &mut Fuzzer, v: &[f32]| -> bool {
        let r = f.eval(0, v);
        r.failed && r.kinds.contains(&kind)
    };
    if fails_kind(f, &rest) {
        return Ok(Minimal { kind, severity: 0.0, params: vec![], evaluations: f.evaluations() - start, verified: true });
    }
    let mut set = {
        let mut pred = |s: &[usize]| fails_kind(f, &with(s, values));
        ddmin(&moved, &mut pred)
    };
    set.sort();
    let mut cur = with(&set, values);
    for _pass in 0..2 {
        let mut changed = false;
        for &i in &set {
            let before = cur[i];
            let tol = 1e-3 * (f.params[i].hi - f.params[i].lo).abs().max(1e-6);
            let mut probe = cur.clone();
            let nv = {
                let mut pred = |x: f32| {
                    probe[i] = x;
                    fails_kind(f, &probe)
                };
                shrink_value(rest[i], before, tol, &mut pred)
            };
            if (nv - before).abs() > 1e-9 {
                cur[i] = nv;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let fin = f.eval(0, &cur);
    let verified = fin.failed;
    if !verified {
        cur = with(&set, values); // bisection assumption broke: fall back to the ddmin result
    }
    let fin = f.eval(0, &cur);
    Ok(Minimal {
        kind,
        severity: fin.severity,
        params: set.iter().map(|&i| MinimalParam { id: f.params[i].id.clone(), value: cur[i], rest: rest[i] }).collect(),
        evaluations: f.evaluations() - start,
        verified: fin.failed,
    })
}

// ---------------------------------------------------------------------------------------------
// the run: sampling, aggregation, clustering
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct FuzzOptions {
    pub samples: usize,
    pub seed: u64,
    pub strategy: Strategy,
    pub frame: f32,
    pub params: ParamOptions,
    pub thresholds: Thresholds,
    pub minimize: bool,
    /// Pose database for the pca strategy (default: the project's own keyed rotations).
    pub db: Option<PoseDb>,
    /// How many worst samples the report lists.
    pub keep_worst: usize,
    /// Failure kinds to ignore (still measured in the metrics).
    pub ignore: Vec<String>,
}

impl Default for FuzzOptions {
    fn default() -> Self {
        FuzzOptions { samples: 200, seed: 1, strategy: Strategy::Random, frame: 0.0, params: ParamOptions::default(), thresholds: Thresholds::default(), minimize: false, db: None, keep_worst: 20, ignore: vec![] }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Bin {
    pub lo: f32,
    pub hi: f32,
    pub n: usize,
    pub failures: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Marginal {
    pub id: String,
    pub bone: Option<String>,
    pub bins: Vec<Bin>,
    /// Failure rate in the outer two bins / the inner bins.
    pub rate_outer: f32,
    pub rate_inner: f32,
    /// Highest minus lowest bin failure rate (bins with at least 3 samples).
    pub spread: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PairScore {
    pub a: String,
    pub b: String,
    /// Worst cell failure rate (3x3 tertile cells, >= 5 samples) minus the larger of the two marginal rates of that cell.
    pub synergy: f32,
    pub cell_rate: f32,
    pub cell_samples: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeatGeo {
    pub name: String,
    pub kind: GeoKind,
    /// Rest-pose vertices in screen space.
    pub pts: Vec<[f32; 2]>,
    pub tris: Vec<[usize; 3]>,
    pub edges: Vec<[usize; 2]>,
    /// Worst normalised stress per vertex over all samples (1 = at a threshold).
    pub max_stress: Vec<f32>,
    /// Fraction of samples in which the vertex was at or above threshold.
    pub fail_fraction: Vec<f32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeatData {
    pub width: f32,
    pub height: f32,
    pub geos: Vec<HeatGeo>,
    pub bones: Vec<(String, [f32; 2], [f32; 2])>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Summary {
    pub samples: usize,
    pub failures: usize,
    pub failure_rate: f32,
    pub by_kind: BTreeMap<String, usize>,
    pub max_severity: f32,
    pub seconds: f32,
    pub ms_per_sample: f32,
    /// Severity with every parameter at rest (a rig that already fails there fails everywhere).
    pub baseline_severity: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct BoneBar {
    pub name: String,
    pub rate: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct FuzzReport {
    pub schema: String,
    pub version: u32,
    pub project: String,
    pub strategy: String,
    pub seed: u64,
    pub frame: f32,
    pub thresholds: Thresholds,
    pub params: Vec<Param>,
    pub summary: Summary,
    pub marginals: Vec<Marginal>,
    pub per_bone: Vec<BoneBar>,
    pub pairs: Vec<PairScore>,
    pub worst: Vec<SampleResult>,
    pub minimal: Option<Minimal>,
    pub heat: HeatData,
}

impl FuzzReport {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }
}

/// Run a fuzz campaign. The project is only read.
pub fn run(project: &Project, asset_dir: Option<&Path>, o: &FuzzOptions) -> Result<FuzzReport, String> {
    let mut f = Fuzzer::new(project, asset_dir, &o.params, o.thresholds.clone(), o.frame)?;
    for k in &o.ignore {
        if !Norm::default().items().iter().any(|i| i.0 == k) {
            return Err(format!("unknown failure kind '{k}' to ignore (fold, stretch, shear, area, spike, self_intersection, ik_unreachable, limit, off_canvas, non_finite, panic)"));
        }
    }
    f.ignore = o.ignore.clone();
    let pca = if o.strategy == Strategy::Pca {
        let db = match &o.db {
            Some(d) => d.clone(),
            None => crate::posesuggest::build_db(Some(project), None, None).map_err(|e| format!("pca strategy: {e}"))?,
        };
        Some(PcaSampler::new(&db, &f.params)?)
    } else {
        None
    };
    let pts = generate(&f.params, o.strategy, o.samples, o.seed, pca.as_ref())?;
    let rest = f.rest_values();
    let baseline = f.eval(0, &rest).severity;
    let t0 = Stamp::now();
    let mut results: Vec<SampleResult> = Vec::with_capacity(pts.len());
    let mut maxs: Vec<Vec<f32>> = f.geos.iter().map(|g| vec![0.0; g.snap.pts.len()]).collect();
    let mut fails: Vec<Vec<f32>> = maxs.clone();
    for (i, v) in pts.iter().enumerate() {
        let out = f.eval_full(i, v);
        for (gi, st) in out.stress.iter().enumerate() {
            for (k, s) in st.iter().enumerate() {
                maxs[gi][k] = maxs[gi][k].max(*s);
                if *s >= 1.0 {
                    fails[gi][k] += 1.0;
                }
            }
        }
        results.push(out.result);
    }
    let secs = t0.elapsed().as_secs_f32();
    let n = results.len();
    let failures = results.iter().filter(|r| r.failed).count();
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    for r in results.iter().filter(|r| r.failed) {
        if let Some(k) = r.kinds.first() {
            *by_kind.entry(k.clone()).or_insert(0) += 1;
        }
    }
    let summary = Summary {
        samples: n,
        failures,
        failure_rate: failures as f32 / n.max(1) as f32,
        by_kind,
        max_severity: results.iter().map(|r| r.severity).fold(0.0, f32::max),
        seconds: secs,
        ms_per_sample: 1000.0 * secs / n.max(1) as f32,
        baseline_severity: baseline,
    };
    let (marginals, per_bone) = marginals(&f.params, &results, 5);
    let pairs = pair_scores(&f.params, &results, &marginals, 10, 10);
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| results[b].severity.total_cmp(&results[a].severity).then(a.cmp(&b)));
    let worst: Vec<SampleResult> = order.iter().take(o.keep_worst).map(|&i| results[i].clone()).collect();
    let minimal = if o.minimize {
        match order.first().map(|&i| &results[i]) {
            Some(r) if r.failed => Some(minimize(&mut f, &r.values, None)?),
            _ => None,
        }
    } else {
        None
    };
    let heat = HeatData {
        width: project.width,
        height: project.height,
        geos: f
            .geos
            .iter()
            .enumerate()
            .map(|(gi, g)| HeatGeo {
                name: g.snap.name.clone(),
                kind: g.snap.kind,
                pts: g.snap.pts.iter().map(|p| {
                    let q = g.snap.world.apply(*p);
                    [q.x, q.y]
                }).collect(),
                tris: g.snap.tris.clone(),
                edges: g.snap.edges.clone(),
                max_stress: maxs[gi].clone(),
                fail_fraction: fails[gi].iter().map(|c| c / n.max(1) as f32).collect(),
            })
            .collect(),
        bones: f.rest_bones.iter().map(|(nm, a, b)| (nm.clone(), [a.x, a.y], [b.x, b.y])).collect(),
    };
    Ok(FuzzReport {
        schema: "osatura-fuzz-report".into(),
        version: 1,
        project: project.name.clone(),
        strategy: o.strategy.name().into(),
        seed: o.seed,
        frame: o.frame,
        thresholds: o.thresholds.clone(),
        params: f.params.clone(),
        summary,
        marginals,
        per_bone,
        pairs,
        worst,
        minimal,
        heat,
    })
}

/// Per-parameter failure rate in `nbins` equal bins of the parameter range, plus per-bone bars (the worst
/// increase of the failure rate at the extremes of its range - outer 40% minus inner 60% - among the bone's parameters; this isolates
/// the parameters that cause failures from those that merely co-occur with them).
pub fn marginals(params: &[Param], results: &[SampleResult], nbins: usize) -> (Vec<Marginal>, Vec<BoneBar>) {
    let nbins = nbins.max(3);
    let mut out = Vec::new();
    for (k, p) in params.iter().enumerate() {
        let mut bins: Vec<Bin> = (0..nbins)
            .map(|b| Bin { lo: p.lo + (p.hi - p.lo) * b as f32 / nbins as f32, hi: p.lo + (p.hi - p.lo) * (b + 1) as f32 / nbins as f32, n: 0, failures: 0 })
            .collect();
        for r in results {
            let b = bin_of(r.values[k], p.lo, p.hi, nbins);
            bins[b].n += 1;
            if r.failed {
                bins[b].failures += 1;
            }
        }
        let rate = |idx: &[usize]| {
            let (n, f) = idx.iter().fold((0, 0), |(n, f), &i| (n + bins[i].n, f + bins[i].failures));
            if n == 0 { 0.0 } else { f as f32 / n as f32 }
        };
        let outer = [0, nbins - 1];
        let inner: Vec<usize> = (1..nbins - 1).collect();
        let rates: Vec<f32> = bins.iter().filter(|b| b.n >= 3).map(|b| b.failures as f32 / b.n as f32).collect();
        let spread = if rates.is_empty() { 0.0 } else { rates.iter().cloned().fold(0.0, f32::max) - rates.iter().cloned().fold(1.0, f32::min) };
        let bone = p.id.strip_prefix("bone:").and_then(|s| s.rsplit_once(':')).map(|x| x.0.to_string());
        out.push(Marginal { id: p.id.clone(), bone, rate_outer: rate(&outer), rate_inner: rate(&inner), spread, bins });
    }
    let mut bars: BTreeMap<String, f32> = BTreeMap::new();
    for m in &out {
        let name = m.bone.clone().unwrap_or_else(|| m.id.clone());
        let e = bars.entry(name).or_insert(0.0);
        *e = e.max((m.rate_outer - m.rate_inner).max(0.0));
    }
    let mut bars: Vec<BoneBar> = bars.into_iter().map(|(name, rate)| BoneBar { name, rate }).collect();
    bars.sort_by(|a, b| b.rate.total_cmp(&a.rate).then(a.name.cmp(&b.name)));
    (out, bars)
}

pub(super) fn bin_of(v: f32, lo: f32, hi: f32, n: usize) -> usize {
    (((v - lo) / (hi - lo).max(1e-12) * n as f32).floor().max(0.0) as usize).min(n - 1)
}

/// Pairwise interaction: for the `top_k` most influential parameters, how much worse than either single-parameter
/// bin is the worst tertile cell of the pair?
pub fn pair_scores(params: &[Param], results: &[SampleResult], marg: &[Marginal], top_k: usize, keep: usize) -> Vec<PairScore> {
    let mut idx: Vec<usize> = (0..params.len()).collect();
    idx.sort_by(|&a, &b| marg[b].spread.total_cmp(&marg[a].spread).then(a.cmp(&b)));
    idx.truncate(top_k);
    let mut out = Vec::new();
    let rate_in = |k: usize, t: usize| -> f32 {
        let p = &params[k];
        let (mut n, mut f) = (0usize, 0usize);
        for r in results {
            if bin_of(r.values[k], p.lo, p.hi, 3) == t {
                n += 1;
                f += r.failed as usize;
            }
        }
        if n == 0 { 0.0 } else { f as f32 / n as f32 }
    };
    for (x, &a) in idx.iter().enumerate() {
        for &b in &idx[x + 1..] {
            let mut cells = [[(0usize, 0usize); 3]; 3];
            for r in results {
                let (i, j) = (bin_of(r.values[a], params[a].lo, params[a].hi, 3), bin_of(r.values[b], params[b].lo, params[b].hi, 3));
                cells[i][j].0 += 1;
                cells[i][j].1 += r.failed as usize;
            }
            let mut best: Option<(f32, f32, usize)> = None;
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for i in 0..3 {
                // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
                #[allow(clippy::needless_range_loop)]
                for j in 0..3 {
                    let (n, f) = cells[i][j];
                    if n < 5 {
                        continue;
                    }
                    let rate = f as f32 / n as f32;
                    let syn = rate - rate_in(a, i).max(rate_in(b, j));
                    if best.is_none_or(|(s, _, _)| syn > s) {
                        best = Some((syn, rate, n));
                    }
                }
            }
            if let Some((s, rate, n)) = best {
                out.push(PairScore { a: params[a].id.clone(), b: params[b].id.clone(), synergy: s, cell_rate: rate, cell_samples: n });
            }
        }
    }
    out.sort_by(|p, q| q.synergy.total_cmp(&p.synergy).then(p.a.cmp(&q.a)).then(p.b.cmp(&q.b)));
    out.truncate(keep);
    out
}
