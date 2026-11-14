// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `posesuggest` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// forward kinematics (checked against the engine in the tests)
// ---------------------------------------------------------------------------------------------

/// Stage position of the tip of `bone` when every project bone is rotated by `angles[i]` degrees from its rest
/// angle (angle limits applied; scale and offset channels are not modelled).
pub fn fk_tip(project: &Project, angles: &[f32], bone: usize) -> [f32; 2] {
    let n = project.bones.len();
    let mut world: Vec<(f64, f64, f64)> = Vec::with_capacity(n); // x, y, rotation (radians)
    let index = |name: &str| project.bones.iter().position(|b| b.name == name);
    for (i, b) in project.bones.iter().enumerate() {
        let mut delta = angles.get(i).copied().unwrap_or(0.0);
        if let Some([lo, hi]) = b.angle_limits {
            delta = delta.clamp(lo, hi);
        }
        let (px, py, pr) = match b.parent.as_deref().and_then(index) {
            Some(pi) if pi < world.len() => world[pi],
            _ => (0.0, 0.0, 0.0),
        };
        let (ox, oy) = (b.rest_offset.x as f64, b.rest_offset.y as f64);
        let (s, c) = pr.sin_cos();
        let pos = (px + c * ox - s * oy, py + s * ox + c * oy);
        let rot = pr + ((b.rest_angle + delta) as f64).to_radians();
        world.push((pos.0, pos.1, rot));
    }
    let (x, y, r) = world[bone];
    let (s, c) = r.sin_cos();
    let len = project.bones[bone].length as f64;
    [(x + c * len) as f32, (y + s * len) as f32]
}

// ---------------------------------------------------------------------------------------------
// suggestions
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BoneValue {
    pub bone: String,
    pub angle: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Candidate {
    pub method: String,
    pub label: String,
    pub score: f32,
    pub plausibility: f32,
    /// Bone rotations to write (after `bind`: project bones only, locked and non-free bones removed).
    pub values: Vec<BoneValue>,
    /// RMS of the constraint residuals (degrees for angles, pixels for tip targets), completion only.
    pub constraint_error: Option<f32>,
    /// Frame the pose belongs at (next-pose: current frame plus the typical gap).
    pub frame: Option<f32>,
    pub notes: Vec<String>,
    #[serde(skip)]
    pub pose: Vec<f32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Suggestions {
    pub kind: String,
    pub candidates: Vec<Candidate>,
    pub notes: Vec<String>,
    /// Channels that were left out because they are locked.
    pub locked_skipped: Vec<String>,
    pub bones: Vec<String>,
    pub poses_in_library: usize,
    pub pca_axes: usize,
    pub pca_explained: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Constraint {
    /// Hold a bone's rotation (degrees) at `value`.
    Angle { bone: String, value: f32, #[serde(default = "w1")] weight: f32 },
    /// Put the tip of `bone` at stage position `at`.
    Reach { bone: String, at: [f32; 2], #[serde(default = "w_reach")] weight: f32 },
}

pub(super) fn w1() -> f32 {
    1.0
}

pub(super) fn w_reach() -> f32 {
    0.1
}

pub struct Suggester {
    pub db: PoseDb,
    pub space: PoseSpace,
    /// Median nearest-neighbour distance between library poses (RMS degrees, at least 0.5).
    pub spacing: f32,
    /// Median distance between consecutive poses of a sequence (RMS degrees, at least 1).
    pub step: f32,
    pub(super) zs: Vec<Vec<f32>>,
}

pub(super) fn median(v: &mut [f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

impl Suggester {
    pub fn new(db: PoseDb) -> Result<Suggester, String> {
        Suggester::with_pca(db, 0.95, 12)
    }

    pub fn with_pca(db: PoseDb, var_keep: f32, max_k: usize) -> Result<Suggester, String> {
        db.validate()?;
        let space = PoseSpace::fit(&db.poses, var_keep, max_k)?;
        let zs: Vec<Vec<f32>> = db.poses.iter().map(|p| space.project(p)).collect();
        let mut nn = Vec::new();
        // large libraries: estimate the spacing from at most 1500 evenly spread query poses
        let stride = (db.poses.len() / 1500).max(1);
        for i in (0..db.poses.len()).step_by(stride) {
            let mut best = f32::MAX;
            for j in 0..db.poses.len() {
                if i != j {
                    let d = rms(&db.poses[i], &db.poses[j]);
                    if d > 1e-4 {
                        best = best.min(d);
                    }
                }
            }
            if best < f32::MAX {
                nn.push(best);
            }
        }
        let spacing = median(&mut nn).max(0.5);
        let mut steps = Vec::new();
        for i in 0..db.poses.len() {
            if let Some(j) = db.successor(i) {
                steps.push(rms(&db.poses[i], &db.poses[j]));
            }
        }
        let step = median(&mut steps).max(1.0);
        Ok(Suggester { db, space, spacing, step, zs })
    }

    pub(super) fn check_pose(&self, x: &[f32], what: &str) -> Result<(), String> {
        if x.len() != self.db.dim() {
            return Err(format!("{what}: expected {} values, got {}", self.db.dim(), x.len()));
        }
        if x.iter().any(|v| !v.is_finite()) {
            return Err(format!("{what}: non-finite value"));
        }
        Ok(())
    }

    /// Library poses nearest to `x` (RMS degrees), closest first.
    pub fn nearest(&self, x: &[f32], n: usize) -> Vec<(usize, f32)> {
        let mut v: Vec<(usize, f32)> = self.db.poses.iter().enumerate().map(|(i, p)| (i, rms(x, p))).collect();
        v.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        v.truncate(n);
        v
    }

    pub fn plausibility(&self, x: &[f32]) -> f32 {
        let d = self.nearest(x, 1).first().map_or(f32::MAX, |p| p.1);
        1.0 / (1.0 + (d - self.spacing).max(0.0) / self.spacing)
    }

    pub(super) fn cand(&self, method: &str, label: &str, pose: Vec<f32>, score: f32, plaus: f32) -> Candidate {
        Candidate { method: method.into(), label: label.into(), score, plausibility: plaus, values: vec![], constraint_error: None, frame: None, notes: vec![], pose }
    }

    // ---- between -------------------------------------------------------------------------

    /// In-between poses at fraction `t` of the way from `a` to `b`. `snap` (0..1) is how far the PCA blend is
    /// pulled toward the kernel average of its `k` nearest library poses.
    pub fn between(&self, a: &[f32], b: &[f32], t: f32, snap: f32, k: usize) -> Result<Vec<Candidate>, String> {
        self.check_pose(a, "pose A")?;
        self.check_pose(b, "pose B")?;
        if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&snap) || k == 0 {
            return Err("t and snap must be in [0, 1] and k >= 1".into());
        }
        let dab = rms(a, b);
        let score_of = |x: &[f32], this: &Suggester| -> (f32, f32) {
            let detour = (rms(a, x) + rms(x, b) - dab).max(0.0) / dab.max(this.spacing);
            let plaus = this.plausibility(x);
            (plaus / (1.0 + detour), plaus)
        };
        let lin: Vec<f32> = a.iter().zip(b).map(|(x, y)| x * (1.0 - t) + y * t).collect();
        let (za, zb) = (self.space.project(a), self.space.project(b));
        let zt: Vec<f32> = za.iter().zip(&zb).map(|(x, y)| x * (1.0 - t) + y * t).collect();
        let pca = self.space.reconstruct(&zt);
        // kernel pull toward the library, neighbours measured in PCA coordinates
        let mut nb: Vec<(usize, f32)> = self.zs.iter().enumerate().map(|(i, z)| (i, z.iter().zip(&zt).map(|(p, q)| (p - q) * (p - q)).sum::<f32>().sqrt())).collect();
        nb.sort_by(|x, y| x.1.total_cmp(&y.1).then(x.0.cmp(&y.0)));
        nb.truncate(k.min(self.db.poses.len()));
        let h = (nb.iter().map(|p| p.1).sum::<f32>() / nb.len() as f32).max(1e-3);
        let ws: Vec<f32> = nb.iter().map(|p| (-(p.1 / h).powi(2) * 0.5).exp()).collect();
        let wsum: f32 = ws.iter().sum::<f32>().max(1e-12);
        let mut kernel = vec![0.0f32; a.len()];
        for ((i, _), w) in nb.iter().zip(&ws) {
            for (kj, pj) in kernel.iter_mut().zip(&self.db.poses[*i]) {
                *kj += w * pj / wsum;
            }
        }
        let manifold: Vec<f32> = pca.iter().zip(&kernel).map(|(p, q)| p * (1.0 - snap) + q * snap).collect();
        // the library pose with the smallest detour near the requested fraction
        let mut best: Option<(f32, usize)> = None;
        for (i, p) in self.db.poses.iter().enumerate() {
            let (da, db_) = (rms(a, p), rms(p, b));
            if da < 1e-3 || db_ < 1e-3 {
                continue;
            }
            let cost = (da + db_ - dab) + (da - t * (da + db_)).abs();
            if best.is_none_or(|bb| cost < bb.0) {
                best = Some((cost, i));
            }
        }
        let mut out = Vec::new();
        for (m, l, p) in [("linear", "linear blend of the angles", lin), ("pca", "blend in PCA space", pca), ("manifold", "PCA blend pulled toward the library", manifold)] {
            let (s, pl) = score_of(&p, self);
            out.push(self.cand(m, l, p, s, pl));
        }
        if let Some((_, i)) = best {
            let p = self.db.poses[i].clone();
            let (s, pl) = score_of(&p, self);
            let mut c = self.cand("library", &format!("library pose {} of '{}'", self.db.meta[i].idx, self.db.sequences[self.db.meta[i].seq].name), p, s, pl);
            c.notes.push("an existing pose from the library, not an interpolation".into());
            out.push(c);
        }
        out.sort_by(|x, y| y.score.total_cmp(&x.score));
        Ok(out)
    }

    // ---- completion ----------------------------------------------------------------------

    /// Most plausible full poses satisfying the constraints. `current` supplies the values of fixed bones (all
    /// bones not in `free`, plus `locked`) and the pose of project bones the database does not know.
    #[allow(clippy::too_many_arguments)]
    pub fn complete(
        &self,
        project: Option<&Project>,
        constraints: &[Constraint],
        current: &[f32],
        base_angles: Option<&[f32]>,
        free: Option<&[String]>,
        locked: &[String],
        prior: f32,
        top: usize,
        model: Option<&Mlp>,
    ) -> Result<Vec<Candidate>, String> {
        self.check_pose(current, "current pose")?;
        if !matches!(prior.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) || !prior.is_finite() || top == 0 {
            return Err("prior must be >= 0 and top >= 1".into());
        }
        let d = self.db.dim();
        let ix = |b: &str| self.db.bones.iter().position(|x| x == b);
        let mut fixed = vec![false; d];
        for l in locked {
            if let Some(i) = ix(l) {
                fixed[i] = true;
            }
        }
        if let Some(f) = free {
            for (i, b) in self.db.bones.iter().enumerate() {
                if !f.contains(b) {
                    fixed[i] = true;
                }
            }
        }
        // resolve constraints
        enum Rc {
            A(usize, f32, f32),
            R(usize, [f32; 2], f32), // project bone index, target, weight
        }
        let mut rcs: Vec<Rc> = Vec::new();
        let mut reach_dep: Vec<usize> = Vec::new();
        for c in constraints {
            match c {
                Constraint::Angle { bone, value, weight } => {
                    let i = ix(bone).ok_or_else(|| format!("bone '{bone}' is not in the pose database"))?;
                    if !value.is_finite() || ((*weight).partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !weight.is_finite() {
                        return Err("constraint values must be finite and weights positive".into());
                    }
                    if locked.contains(bone) {
                        return Err(format!("bone '{bone}' is locked and cannot be constrained"));
                    }
                    fixed[i] = false;
                    rcs.push(Rc::A(i, *value, *weight));
                }
                Constraint::Reach { bone, at, weight } => {
                    let p = project.ok_or("a reach constraint needs the project (for its bone lengths)")?;
                    let pi = p.bones.iter().position(|b| b.name == *bone).ok_or_else(|| format!("unknown bone '{bone}'"))?;
                    if at.iter().any(|v| !v.is_finite()) || ((*weight).partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !weight.is_finite() {
                        return Err("constraint values must be finite and weights positive".into());
                    }
                    reach_dep.push(pi);
                    rcs.push(Rc::R(pi, *at, *weight));
                }
            }
        }
        if rcs.is_empty() {
            return Err("give at least one constraint (a fixed angle or a reach target)".into());
        }
        let project_ix: Vec<Option<usize>> = match project {
            Some(p) => self.db.bones.iter().map(|b| p.bones.iter().position(|x| x.name == *b)).collect(),
            None => vec![None; d],
        };
        let limits: Vec<Option<[f32; 2]>> = match project {
            Some(p) => project_ix.iter().map(|i| i.and_then(|i| p.bones[i].angle_limits)).collect(),
            None => vec![None; d],
        };
        let clamp_pose = |x: &mut Vec<f32>| {
            for (v, l) in x.iter_mut().zip(&limits) {
                if let Some([lo, hi]) = l {
                    *v = v.clamp(*lo, *hi);
                }
            }
        };
        let tip_of = |x: &[f32], pi: usize| -> [f32; 2] {
            let p = project.expect("checked");
            let mut ang: Vec<f32> = base_angles.map(|b| b.to_vec()).unwrap_or_else(|| vec![0.0; p.bones.len()]);
            ang.resize(p.bones.len(), 0.0);
            for (j, pj) in project_ix.iter().enumerate() {
                if let Some(pj) = pj {
                    ang[*pj] = x[j];
                }
            }
            fk_tip(p, &ang, pi)
        };
        const W_FIXED: f32 = 1.0e4;
        let k = self.space.k();
        let residuals = |z: &[f32]| -> Vec<f32> {
            let mut x = self.space.reconstruct(z);
            clamp_pose(&mut x);
            let mut r = Vec::new();
            for c in &rcs {
                match c {
                    Rc::A(i, v, w) => r.push(w.sqrt() * (x[*i] - v)),
                    Rc::R(pi, at, w) => {
                        let t = tip_of(&x, *pi);
                        r.push(w.sqrt() * (t[0] - at[0]));
                        r.push(w.sqrt() * (t[1] - at[1]));
                    }
                }
            }
            for i in 0..d {
                if fixed[i] {
                    r.push(W_FIXED.sqrt() * (x[i] - current[i]));
                }
            }
            for (j, zj) in z.iter().enumerate() {
                r.push((prior / self.space.eig[j]).sqrt() * zj);
            }
            r
        };
        let cost = |z: &[f32]| residuals(z).iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>();
        let linear = reach_dep.is_empty();
        let solve = |z0: Vec<f32>| -> Vec<f32> {
            let mut z = z0;
            let mut lam = 1e-2f64;
            let mut c0 = cost(&z);
            for _ in 0..if linear { 8 } else { 60 } {
                let r0 = residuals(&z);
                let m = r0.len();
                // Jacobian by central differences
                let mut jac = vec![vec![0.0f64; k]; m];
                for j in 0..k {
                    let h = 0.02f32;
                    let (mut zp, mut zm) = (z.clone(), z.clone());
                    zp[j] += h;
                    zm[j] -= h;
                    let (rp, rm) = (residuals(&zp), residuals(&zm));
                    for i in 0..m {
                        jac[i][j] = ((rp[i] - rm[i]) / (2.0 * h)) as f64;
                    }
                }
                let mut jtj = vec![vec![0.0f64; k]; k];
                let mut jtr = vec![0.0f64; k];
                for i in 0..m {
                    for a in 0..k {
                        jtr[a] += jac[i][a] * r0[i] as f64;
                        for b in 0..k {
                            jtj[a][b] += jac[i][a] * jac[i][b];
                        }
                    }
                }
                let mut improved = false;
                for _ in 0..12 {
                    let mut a = jtj.clone();
                    for q in 0..k {
                        a[q][q] += lam * (jtj[q][q] + 1e-9);
                    }
                    let Some(delta) = solve_linear(a, jtr.iter().map(|v| -v).collect()) else {
                        lam *= 10.0;
                        continue;
                    };
                    let zn: Vec<f32> = z.iter().zip(&delta).map(|(a, b)| a + *b as f32).collect();
                    let cn = cost(&zn);
                    if cn < c0 {
                        let step: f64 = delta.iter().map(|v| v * v).sum::<f64>().sqrt();
                        z = zn;
                        let gain = c0 - cn;
                        c0 = cn;
                        lam = (lam / 3.0).max(1e-9);
                        improved = true;
                        if step < 1e-4 || gain < 1e-9 {
                            return z;
                        }
                        break;
                    }
                    lam *= 4.0;
                }
                if !improved {
                    break;
                }
            }
            z
        };
        // starts: the mean pose and the library poses that already satisfy the constraints best
        let mut starts: Vec<Vec<f32>> = vec![vec![0.0; k]];
        let mut ranked: Vec<(f64, usize)> = self.zs.iter().enumerate().map(|(i, z)| (cost(z), i)).collect();
        ranked.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for (_, i) in ranked.iter().take(4) {
            starts.push(self.zs[*i].clone());
        }
        let mut sols: Vec<(f64, Vec<f32>)> = Vec::new();
        for s in starts {
            let z = solve(s);
            let c = cost(&z);
            let mut x = self.space.reconstruct(&z);
            clamp_pose(&mut x);
            if sols.iter().all(|(_, y)| rms(y, &x) > 0.5) {
                sols.push((c, x));
            }
        }
        sols.sort_by(|a, b| a.0.total_cmp(&b.0));
        let const_err = |x: &[f32]| -> f32 {
            let (mut s, mut n) = (0.0f32, 0usize);
            for c in &rcs {
                match c {
                    Rc::A(i, v, _) => {
                        s += (x[*i] - v).powi(2);
                        n += 1;
                    }
                    Rc::R(pi, at, _) => {
                        let t = tip_of(x, *pi);
                        s += (t[0] - at[0]).powi(2) + (t[1] - at[1]).powi(2);
                        n += 1;
                    }
                }
            }
            (s / n as f32).sqrt()
        };
        let mut out: Vec<Candidate> = Vec::new();
        for (n, (_, x)) in sols.iter().take(top).enumerate() {
            let e = const_err(x);
            let pl = self.plausibility(x);
            let mut c = self.cand("pca-ls", &format!("regularised least squares in PCA space (start {})", n + 1), x.clone(), pl / (1.0 + e / 5.0), pl);
            c.constraint_error = Some(e);
            out.push(c);
        }
        // baseline: the library pose that fits best
        let lib_best = (0..self.db.poses.len())
            .map(|i| {
                let mut x = self.db.poses[i].clone();
                for j in 0..d {
                    if fixed[j] {
                        x[j] = current[j];
                    }
                }
                clamp_pose(&mut x);
                (const_err(&x), x)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((e, x)) = lib_best {
            let pl = self.plausibility(&x);
            let mut c = self.cand("library-nn", "best-fitting library pose (nearest-neighbour baseline)", x, pl / (1.0 + e / 5.0), pl);
            c.constraint_error = Some(e);
            out.push(c);
        }
        if let Some(m) = model {
            if !reach_dep.is_empty() {
                // the network sees only angles
                if let Some(c) = out.first_mut() {
                    c.notes.push("the MLP was skipped: it handles fixed angles only, not reach targets".into());
                }
            } else {
                let mut obs: Vec<Option<f32>> = vec![None; d];
                for i in 0..d {
                    if fixed[i] {
                        obs[i] = Some(current[i]);
                    }
                }
                for c in &rcs {
                    if let Rc::A(i, v, _) = c {
                        obs[*i] = Some(*v);
                    }
                }
                let mut x = predict_completion(m, &self.db.bones, &obs)?;
                clamp_pose(&mut x);
                let e = const_err(&x);
                let pl = self.plausibility(&x);
                let mut c = self.cand("mlp", "user-supplied small network", x, pl / (1.0 + e / 5.0), pl);
                c.constraint_error = Some(e);
                out.push(c);
            }
        }
        out.sort_by(|a, b| b.score.total_cmp(&a.score));
        Ok(out)
    }

    // ---- next ----------------------------------------------------------------------------

    /// Candidates for the pose that follows `current`: the motion that followed the nearest library poses, applied
    /// to `current`. `frame` is the current frame; candidates carry `frame + gap` where `gap` is the weighted
    /// frame spacing of the voting successors.
    pub fn next(&self, current: &[f32], frame: f32, k: usize, top: usize) -> Result<Vec<Candidate>, String> {
        self.check_pose(current, "current pose")?;
        if k == 0 || top == 0 {
            return Err("k and top must be at least 1".into());
        }
        let mut voters: Vec<(usize, usize, f32)> = Vec::new(); // (pose, successor, distance)
        for (i, p) in self.db.poses.iter().enumerate() {
            if let Some(j) = self.db.successor(i) {
                voters.push((i, j, rms(current, p)));
            }
        }
        if voters.is_empty() {
            return Err("the library has no consecutive poses (every sequence is a single pose)".into());
        }
        voters.sort_by(|a, b| a.2.total_cmp(&b.2).then(a.0.cmp(&b.0)));
        voters.truncate(k);
        let h = (voters.iter().map(|v| v.2).sum::<f32>() / voters.len() as f32).max(self.spacing * 0.5);
        let ws: Vec<f32> = voters.iter().map(|v| (-(v.2 / h).powi(2) * 0.5).exp()).collect();
        let wsum: f32 = ws.iter().sum::<f32>().max(1e-12);
        let d = current.len();
        let delta_of = |(i, j, _): &(usize, usize, f32)| -> Vec<f32> { (0..d).map(|b| self.db.poses[*j][b] - self.db.poses[*i][b]).collect() };
        let gap_of = |(i, j, _): &(usize, usize, f32)| (self.db.meta[*j].frame - self.db.meta[*i].frame).max(0.0);
        let mut mean_delta = vec![0.0f32; d];
        let mut mean_gap = 0.0f32;
        for (v, w) in voters.iter().zip(&ws) {
            for (m, dv) in mean_delta.iter_mut().zip(delta_of(v)) {
                *m += w * dv / wsum;
            }
            mean_gap += w * gap_of(v) / wsum;
        }
        let spread = (voters.iter().zip(&ws).map(|(v, w)| w * rms(&delta_of(v), &mean_delta).powi(2)).sum::<f32>() / wsum).sqrt();
        let apply = |delta: &[f32]| -> Vec<f32> { current.iter().zip(delta).map(|(c, dv)| c + dv).collect() };
        let far = voters[0].2 / self.spacing;
        let mut out = Vec::new();
        let mp = apply(&mean_delta);
        let mpl = self.plausibility(&mp);
        let mut mc = self.cand("markov-mean", "kernel-weighted mean of the following motions", mp, (1.0 / (1.0 + spread / self.step)) * mpl, mpl);
        mc.frame = Some(frame + mean_gap);
        out.push(mc);
        // individual successors, near-duplicates merged
        let mut singles: Vec<(f32, Vec<f32>, f32, String)> = Vec::new();
        for (v, w) in voters.iter().zip(&ws) {
            let p = apply(&delta_of(v));
            let label = format!("what followed pose {} of '{}'", self.db.meta[v.0].idx, self.db.sequences[self.db.meta[v.0].seq].name);
            match singles.iter_mut().find(|s| rms(&s.1, &p) < 0.5 * self.step.min(self.spacing).max(0.25)) {
                Some(s) => s.0 += w / wsum,
                None => singles.push((w / wsum, p, gap_of(v), label)),
            }
        }
        for (share, p, gap, label) in singles {
            let pl = self.plausibility(&p);
            let mut c = self.cand("successor", &label, p, share * pl, pl);
            c.frame = Some(frame + gap);
            out.push(c);
        }
        if far > 3.0 {
            for c in out.iter_mut() {
                c.notes.push(format!("the current pose is {:.1} deg from the nearest library pose (typical spacing {:.1}); the library may not cover it", voters[0].2, self.spacing));
            }
        }
        out.sort_by(|a, b| b.score.total_cmp(&a.score));
        out.truncate(top);
        Ok(out)
    }

    // ---- binding to a project ------------------------------------------------------------

    /// Fill `values` of each candidate: project bones only, locked channels and bones outside `free` removed.
    /// Returns the names of locked channels that were left out.
    pub fn bind(&self, cands: &mut [Candidate], project: &Project, free: Option<&[String]>) -> Vec<String> {
        let mut skipped: Vec<String> = Vec::new();
        for c in cands.iter_mut() {
            c.values.clear();
            for (b, v) in self.db.bones.iter().zip(&c.pose) {
                if !project.bones.iter().any(|x| x.name == *b) || free.is_some_and(|f| !f.contains(b)) {
                    continue;
                }
                if crate::chanmeta::is_locked(project, b, Prop::BoneAngle) {
                    if !skipped.contains(b) {
                        skipped.push(b.clone());
                    }
                    continue;
                }
                c.values.push(BoneValue { bone: b.clone(), angle: *v });
            }
        }
        skipped
    }
}
