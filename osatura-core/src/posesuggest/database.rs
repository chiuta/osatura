// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `posesuggest` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// database
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PoseMeta {
    pub seq: usize,
    /// Position inside the sequence (0-based).
    pub idx: usize,
    /// Frame in the source (project or take).
    pub frame: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeqInfo {
    pub name: String,
    /// "project" or "library".
    pub source: String,
    pub poses: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PoseDb {
    #[serde(default = "db_format")]
    pub format: String,
    #[serde(default = "one")]
    pub version: u32,
    pub bones: Vec<String>,
    pub poses: Vec<Vec<f32>>,
    pub meta: Vec<PoseMeta>,
    pub sequences: Vec<SeqInfo>,
}

pub(super) fn db_format() -> String {
    "osatura-posedb".into()
}

pub(super) fn one() -> u32 {
    1
}

impl PoseDb {
    pub fn dim(&self) -> usize {
        self.bones.len()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format != "osatura-posedb" || self.version != 1 {
            return Err(format!("not an osatura-posedb version 1 file (format '{}', version {})", self.format, self.version));
        }
        if self.bones.is_empty() {
            return Err("pose database has no bones".into());
        }
        if self.poses.len() != self.meta.len() {
            return Err("poses and meta differ in length".into());
        }
        for (i, p) in self.poses.iter().enumerate() {
            if p.len() != self.bones.len() || p.iter().any(|v| !v.is_finite()) {
                return Err(format!("pose {i} has the wrong size or a non-finite value"));
            }
        }
        for m in &self.meta {
            if m.seq >= self.sequences.len() {
                return Err("pose refers to a missing sequence".into());
            }
        }
        Ok(())
    }

    pub fn from_json(s: &str) -> Result<PoseDb, String> {
        let db: PoseDb = serde_json::from_str(s).map_err(|e| format!("bad pose database: {e}"))?;
        db.validate()?;
        Ok(db)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("pose db serialises")
    }

    /// Index of the pose that follows `i` in its sequence.
    pub fn successor(&self, i: usize) -> Option<usize> {
        let m = &self.meta[i];
        let j = i + 1;
        (j < self.poses.len() && self.meta[j].seq == m.seq && self.meta[j].idx == m.idx + 1).then_some(j)
    }
}

pub(super) type NamedTracks = Vec<(String, Track)>;

pub(super) fn project_tracks(p: &Project) -> NamedTracks {
    p.channels
        .iter()
        .filter(|c| c.prop == Prop::BoneAngle && c.mode == Mode::Replace && !crate::skins::is_collapse_channel(c))
        .filter_map(|c| match &c.source {
            Source::Keys { track } if !track.keys.is_empty() => Some((c.target.clone(), track.clone())),
            _ => None,
        })
        .collect()
}

pub(super) fn take_tracks(t: &crate::motionlib::Take) -> NamedTracks {
    t.tracks
        .iter()
        .filter(|tr| tr.prop == Prop::BoneAngle && !tr.keys.is_empty())
        .map(|tr| (tr.target.clone(), Track { keys: tr.keys.clone(), extrapolate: Default::default() }))
        .collect()
}

/// Build the database from a project's keyed bone rotations and/or a motion library.
/// `step`: sample every `step` frames instead of at the key frames. Bones a sequence does not animate are 0.
pub fn build_db(project: Option<&Project>, lib: Option<&Library>, step: Option<f32>) -> Result<PoseDb, String> {
    if let Some(s) = step {
        if (s.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !s.is_finite() {
            return Err("step must be a positive number of frames".into());
        }
    }
    let mut seqs: Vec<(String, &'static str, NamedTracks)> = Vec::new();
    if let Some(p) = project {
        let t = project_tracks(p);
        if !t.is_empty() {
            seqs.push(("project".into(), "project", t));
        }
    }
    if let Some(l) = lib {
        for take in &l.takes {
            let t = take_tracks(take);
            if !t.is_empty() {
                seqs.push((take.name.clone(), "library", t));
            }
        }
    }
    if seqs.is_empty() {
        return Err("no keyed bone rotations found (bone_angle channels in the project or takes in the library)".into());
    }
    let mut bones: Vec<String> = Vec::new();
    // project bones first, in their order, so database columns follow the rig
    if let Some(p) = project {
        for b in &p.bones {
            if seqs.iter().any(|s| s.2.iter().any(|(t, _)| *t == b.name)) && !bones.contains(&b.name) {
                bones.push(b.name.clone());
            }
        }
    }
    for s in &seqs {
        for (t, _) in &s.2 {
            if !bones.contains(t) {
                bones.push(t.clone());
            }
        }
    }
    let mut db = PoseDb { format: db_format(), version: 1, bones, poses: vec![], meta: vec![], sequences: vec![] };
    for (name, source, tracks) in &seqs {
        let mut frames: Vec<f32> = Vec::new();
        match step {
            None => {
                for (_, t) in tracks {
                    frames.extend(t.keys.iter().map(|k| k.frame));
                }
                frames.sort_by(|a, b| a.total_cmp(b));
                frames.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
            }
            Some(s) => {
                let lo = tracks.iter().filter_map(|(_, t)| t.first_frame()).fold(f32::MAX, f32::min);
                let hi = tracks.iter().filter_map(|(_, t)| t.last_frame()).fold(f32::MIN, f32::max);
                let mut f = lo;
                while f < hi {
                    frames.push(f);
                    f += s;
                    if frames.len() > 100_000 {
                        return Err("step produces more than 100000 poses".into());
                    }
                }
                frames.push(hi);
            }
        }
        if frames.iter().any(|f| !f.is_finite()) {
            return Err("non-finite key frame".into());
        }
        let seq = db.sequences.len();
        for (idx, &f) in frames.iter().enumerate() {
            let pose: Vec<f32> = db.bones.iter().map(|b| tracks.iter().find(|(t, _)| t == b).and_then(|(_, tr)| tr.eval(f)).unwrap_or(0.0)).collect();
            if pose.iter().any(|v| !v.is_finite()) {
                return Err("non-finite angle in the source".into());
            }
            db.poses.push(pose);
            db.meta.push(PoseMeta { seq, idx, frame: f });
        }
        db.sequences.push(SeqInfo { name: name.clone(), source: (*source).into(), poses: frames.len() });
    }
    if db.poses.len() < 2 {
        return Err("a pose database needs at least two poses".into());
    }
    Ok(db)
}

/// The project's current pose (bone-rotation deltas) of the database bones at `frame`: keyed Replace channels,
/// 0 for bones without one (expressions, noise and additive channels are not evaluated).
pub fn pose_at(project: &Project, bones: &[String], frame: f32) -> Vec<f32> {
    bones
        .iter()
        .map(|b| {
            project
                .channels
                .iter()
                .find(|c| c.target == *b && c.prop == Prop::BoneAngle && c.mode == Mode::Replace && matches!(c.source, Source::Keys { .. }) && !crate::skins::is_collapse_channel(c))
                .and_then(|c| match &c.source {
                    Source::Keys { track } => track.eval(frame),
                    _ => None,
                })
                .unwrap_or(0.0)
        })
        .collect()
}

pub fn rms(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() {
        return 0.0;
    }
    (a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum::<f32>() / a.len() as f32).sqrt()
}

// ---------------------------------------------------------------------------------------------
// PCA
// ---------------------------------------------------------------------------------------------

/// Cyclic Jacobi eigenvalue algorithm for a symmetric matrix. Returns eigenvalues (unsorted) and the
/// eigenvectors as columns of `v` (`v[row][col]`).
pub fn jacobi_eigen(mut a: Vec<Vec<f64>>) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = a.len();
    let mut v = vec![vec![0.0; n]; n];
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        v[i][i] = 1.0;
    }
    for _ in 0..80 {
        let off: f64 = (0..n).flat_map(|i| (0..n).filter(move |&j| j != i).map(move |j| (i, j))).map(|(i, j)| a[i][j] * a[i][j]).sum();
        let diag: f64 = (0..n).map(|i| a[i][i] * a[i][i]).sum::<f64>().max(1e-300);
        if off <= 1e-24 * diag {
            break;
        }
        for p in 0..n {
            for q in p + 1..n {
                if a[p][q].abs() < 1e-300 {
                    continue;
                }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
                #[allow(clippy::needless_range_loop)]
                for k in 0..n {
                    let (akp, akq) = (a[k][p], a[k][q]);
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
                #[allow(clippy::needless_range_loop)]
                for k in 0..n {
                    let (apk, aqk) = (a[p][k], a[q][k]);
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
                // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
                #[allow(clippy::needless_range_loop)]
                for k in 0..n {
                    let (vkp, vkq) = (v[k][p], v[k][q]);
                    v[k][p] = c * vkp - s * vkq;
                    v[k][q] = s * vkp + c * vkq;
                }
            }
        }
    }
    ((0..n).map(|i| a[i][i]).collect(), v)
}

/// Power iteration for the dominant eigenpair of a symmetric positive semi-definite matrix.
pub fn power_iteration(a: &[Vec<f64>], iters: usize, seed: u64) -> (f64, Vec<f64>) {
    let n = a.len();
    let mut rng = Rng::new(seed);
    let mut x: Vec<f64> = (0..n).map(|_| rng.unit() as f64 + 0.1).collect();
    let mut lambda = 0.0;
    for _ in 0..iters {
        let y: Vec<f64> = (0..n).map(|i| (0..n).map(|j| a[i][j] * x[j]).sum()).collect();
        let norm = y.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm < 1e-300 {
            return (0.0, x);
        }
        lambda = norm;
        x = y.iter().map(|v| v / norm).collect();
    }
    (lambda, x)
}

#[derive(Clone, Debug, Serialize)]
pub struct PoseSpace {
    pub mean: Vec<f32>,
    /// `k` orthonormal axes of length `d`, strongest first.
    pub comps: Vec<Vec<f32>>,
    /// Variance (deg squared) along each axis.
    pub eig: Vec<f32>,
    pub total_var: f32,
    /// Cumulative explained variance fraction per axis.
    pub explained: Vec<f32>,
}

impl PoseSpace {
    /// Keep the fewest axes explaining `var_keep` of the variance (at most `max_k`, at least 1).
    pub fn fit(poses: &[Vec<f32>], var_keep: f32, max_k: usize) -> Result<PoseSpace, String> {
        if poses.len() < 2 {
            return Err("PCA needs at least two poses".into());
        }
        if !(var_keep > 0.0 && var_keep <= 1.0) || max_k == 0 {
            return Err("variance fraction must be in (0, 1] and max components >= 1".into());
        }
        let d = poses[0].len();
        let n = poses.len();
        let mean: Vec<f64> = (0..d).map(|j| poses.iter().map(|p| p[j] as f64).sum::<f64>() / n as f64).collect();
        let mut cov = vec![vec![0.0f64; d]; d];
        for p in poses {
            for i in 0..d {
                let di = p[i] as f64 - mean[i];
                for j in i..d {
                    cov[i][j] += di * (p[j] as f64 - mean[j]);
                }
            }
        }
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for i in 0..d {
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for j in i..d {
                cov[i][j] /= (n - 1) as f64;
                cov[j][i] = cov[i][j];
            }
        }
        let total: f64 = (0..d).map(|i| cov[i][i]).sum();
        if total < 1e-9 {
            return Err("all poses are identical; there is nothing to learn".into());
        }
        let (vals, vecs) = jacobi_eigen(cov);
        let mut order: Vec<usize> = (0..d).collect();
        order.sort_by(|&a, &b| vals[b].total_cmp(&vals[a]).then(a.cmp(&b)));
        let (mut comps, mut eig, mut explained) = (Vec::new(), Vec::new(), Vec::new());
        let mut acc = 0.0;
        for &c in &order {
            if comps.len() >= max_k || (!comps.is_empty() && (acc / total >= var_keep as f64 || vals[c] < 1e-9 * total)) {
                break;
            }
            let mut axis: Vec<f64> = (0..d).map(|r| vecs[r][c]).collect();
            // sign convention: largest-magnitude entry positive (deterministic)
            let big = axis.iter().cloned().fold(0.0f64, |m, v| if v.abs() > m.abs() { v } else { m });
            if big < 0.0 {
                axis.iter_mut().for_each(|v| *v = -*v);
            }
            acc += vals[c].max(0.0);
            comps.push(axis.iter().map(|&v| v as f32).collect());
            eig.push(vals[c].max(1e-12) as f32);
            explained.push((acc / total) as f32);
        }
        Ok(PoseSpace { mean: mean.iter().map(|&v| v as f32).collect(), comps, eig, total_var: total as f32, explained })
    }

    pub fn k(&self) -> usize {
        self.comps.len()
    }
    pub fn project(&self, x: &[f32]) -> Vec<f32> {
        self.comps.iter().map(|c| c.iter().zip(x).zip(&self.mean).map(|((a, v), m)| a * (v - m)).sum()).collect()
    }
    pub fn reconstruct(&self, z: &[f32]) -> Vec<f32> {
        let mut x = self.mean.clone();
        for (c, &zi) in self.comps.iter().zip(z) {
            for (xj, cj) in x.iter_mut().zip(c) {
                *xj += cj * zi;
            }
        }
        x
    }
}
