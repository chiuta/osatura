// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `posesuggest` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// synthetic library and honest benchmark
// ---------------------------------------------------------------------------------------------

/// A deterministic gait-like library: `seqs` cycles of `per` poses over `bones` bones; each cycle has its own
/// amplitude, a phase offset and a little noise. The poses lie near a curved two-parameter surface.
pub fn synthetic_db(seed: u64, seqs: usize, per: usize, bones: usize) -> PoseDb {
    let mut rng = Rng::new(seed);
    let names: Vec<String> = (0..bones).map(|i| format!("b{i}")).collect();
    let amp: Vec<f32> = (0..bones).map(|_| 10.0 + 30.0 * rng.unit()).collect();
    let psi: Vec<f32> = (0..bones).map(|_| std::f32::consts::TAU * rng.unit()).collect();
    let off: Vec<f32> = (0..bones).map(|_| 10.0 * (rng.unit() - 0.5)).collect();
    let mut db = PoseDb { format: db_format(), version: 1, bones: names, poses: vec![], meta: vec![], sequences: vec![] };
    for s in 0..seqs {
        let k = 0.5 + 1.0 * rng.unit();
        let phase0 = std::f32::consts::TAU * rng.unit();
        for i in 0..per {
            let phi = phase0 + std::f32::consts::TAU * i as f32 / per as f32;
            let pose: Vec<f32> = (0..bones)
                .map(|j| {
                    let noise = (rng.unit() - 0.5) * 1.0;
                    k * amp[j] * (phi + psi[j]).sin() + off[j] + 0.15 * k * k * amp[j] * (2.0 * phi + psi[j]).sin() + noise
                })
                .collect();
            db.poses.push(pose);
            db.meta.push(PoseMeta { seq: s, idx: i, frame: (i * 4) as f32 });
        }
        db.sequences.push(SeqInfo { name: format!("cycle{s}"), source: "library".into(), poses: per });
    }
    db
}

/// The network configuration the CLI benchmark reports.
pub fn default_bench_opts() -> TrainOpts {
    TrainOpts { hidden: vec![64, 64], epochs: 1500, lr: 0.02, momentum: 0.9, batch: 16, seed: 1 }
}

#[derive(Clone, Debug, Serialize)]
pub struct BenchLine {
    pub task: String,
    pub method: String,
    /// Mean RMS error in degrees against the held-out ground truth (lower is better).
    pub rms_deg: f32,
}

pub(super) fn split_db(db: &PoseDb, hold: impl Fn(usize) -> bool) -> (PoseDb, PoseDb) {
    let mk = |keep: &dyn Fn(usize) -> bool| {
        let mut out = PoseDb { format: db_format(), version: 1, bones: db.bones.clone(), poses: vec![], meta: vec![], sequences: vec![] };
        for (s, info) in db.sequences.iter().enumerate() {
            if !keep(s) {
                continue;
            }
            let ns = out.sequences.len();
            out.sequences.push(info.clone());
            for (i, m) in db.meta.iter().enumerate().filter(|(_, m)| m.seq == s) {
                out.poses.push(db.poses[i].clone());
                out.meta.push(PoseMeta { seq: ns, idx: m.idx, frame: m.frame });
            }
        }
        out
    };
    (mk(&|s| !hold(s)), mk(&|s| hold(s)))
}

/// Compare the methods on held-out sequences of the synthetic library. Honest by construction: whatever wins, wins.
pub fn benchmark(seed: u64, mlp_opts: &TrainOpts) -> Result<Vec<BenchLine>, String> {
    let all = synthetic_db(seed, 30, 16, 10);
    let (train, test) = split_db(&all, |s| s % 5 == 0);
    let sg = Suggester::new(train.clone())?;
    let d = train.dim();
    let (mlp, _hist) = train_completion(&train, mlp_opts)?;
    let mut rng = Rng::new(seed ^ 0xBEEF);
    let mean_pose = sg.space.mean.clone();
    let mut sums: Vec<(&str, &str, f64, usize)> = vec![
        ("completion", "mean pose", 0.0, 0),
        ("completion", "nearest library pose", 0.0, 0),
        ("completion", "PCA least squares", 0.0, 0),
        ("completion", "MLP", 0.0, 0),
    ];
    for p in &test.poses {
        for _ in 0..3 {
            let mut obs = vec![false; d];
            let mut left = 3;
            while left > 0 {
                let j = rng.below(d);
                if !obs[j] {
                    obs[j] = true;
                    left -= 1;
                }
            }
            let hidden: Vec<usize> = (0..d).filter(|&j| !obs[j]).collect();
            let err = |x: &[f32]| (hidden.iter().map(|&j| (x[j] - p[j]).powi(2)).sum::<f32>() / hidden.len() as f32).sqrt() as f64;
            let cons: Vec<Constraint> = (0..d).filter(|&j| obs[j]).map(|j| Constraint::Angle { bone: sg.db.bones[j].clone(), value: p[j], weight: 1.0 }).collect();
            // mean pose with the observed values pasted in
            let mut m = mean_pose.clone();
            for j in 0..d {
                if obs[j] {
                    m[j] = p[j];
                }
            }
            // nearest library pose by the observed bones only
            let nn = train.poses.iter().min_by(|a, b| {
                let da: f32 = (0..d).filter(|&j| obs[j]).map(|j| (a[j] - p[j]).powi(2)).sum();
                let db_: f32 = (0..d).filter(|&j| obs[j]).map(|j| (b[j] - p[j]).powi(2)).sum();
                da.total_cmp(&db_)
            });
            let cur = vec![0.0; d];
            let cands = sg.complete(None, &cons, &cur, None, None, &[], 1.0, 1, None)?;
            let pca = cands.iter().find(|c| c.method == "pca-ls").ok_or("no PCA candidate")?;
            let observed: Vec<Option<f32>> = (0..d).map(|j| obs[j].then_some(p[j])).collect();
            let net = predict_completion(&mlp, &sg.db.bones, &observed)?;
            let vals = [err(&m), err(nn.expect("non-empty")), err(&pca.pose), err(&net)];
            for (s, v) in sums.iter_mut().zip(vals) {
                s.2 += v;
                s.3 += 1;
            }
        }
    }
    let mut out: Vec<BenchLine> = sums.iter().map(|s| BenchLine { task: s.0.into(), method: s.1.into(), rms_deg: (s.2 / s.3 as f64) as f32 }).collect();
    // between: predict the middle pose of three consecutive test poses
    let mut bsum = [0.0f64; 4];
    let mut bn = 0usize;
    for i in 0..test.poses.len() {
        if i + 2 < test.poses.len() && test.meta[i].seq == test.meta[i + 2].seq {
            let truth = &test.poses[i + 1];
            let c = sg.between(&test.poses[i], &test.poses[i + 2], 0.5, 0.5, 6)?;
            for (slot, m) in ["linear", "pca", "manifold", "library"].iter().enumerate() {
                if let Some(x) = c.iter().find(|x| x.method == *m) {
                    bsum[slot] += rms(&x.pose, truth) as f64;
                }
            }
            bn += 1;
        }
    }
    for (slot, m) in ["linear blend", "PCA blend", "PCA blend pulled to the library", "nearest between library pose"].iter().enumerate() {
        out.push(BenchLine { task: "between".into(), method: (*m).into(), rms_deg: (bsum[slot] / bn.max(1) as f64) as f32 });
    }
    // next: predict the following pose
    let (mut hold, mut mean_e, mut best_e, mut nn) = (0.0f64, 0.0f64, 0.0f64, 0usize);
    for i in 0..test.poses.len() {
        if let Some(j) = test.successor(i) {
            let truth = &test.poses[j];
            hold += rms(&test.poses[i], truth) as f64;
            let c = sg.next(&test.poses[i], test.meta[i].frame, 8, 8)?;
            mean_e += rms(&c.iter().find(|x| x.method == "markov-mean").ok_or("no mean")?.pose, truth) as f64;
            best_e += c.iter().map(|x| rms(&x.pose, truth)).fold(f32::MAX, f32::min) as f64;
            nn += 1;
        }
    }
    let nn = nn.max(1) as f64;
    out.push(BenchLine { task: "next".into(), method: "stay in place".into(), rms_deg: (hold / nn) as f32 });
    out.push(BenchLine { task: "next".into(), method: "Markov mean".into(), rms_deg: (mean_e / nn) as f32 });
    out.push(BenchLine { task: "next".into(), method: "best of the candidates (oracle choice)".into(), rms_deg: (best_e / nn) as f32 });
    Ok(out)
}
