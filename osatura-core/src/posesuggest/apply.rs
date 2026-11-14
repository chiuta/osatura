// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `posesuggest` (see the module docs in `mod.rs`).
use super::*;

/// Gaussian elimination with partial pivoting; `None` when singular.
pub(super) fn solve_linear(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for c in 0..n {
        let piv = (c..n).max_by(|&i, &j| a[i][c].abs().total_cmp(&a[j][c].abs()))?;
        if a[piv][c].abs() < 1e-14 {
            return None;
        }
        a.swap(c, piv);
        b.swap(c, piv);
        for r in c + 1..n {
            let f = a[r][c] / a[c][c];
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for k in c..n {
                a[r][k] -= f * a[c][k];
            }
            b[r] -= f * b[c];
        }
    }
    let mut x = vec![0.0; n];
    for r in (0..n).rev() {
        let s: f64 = (r + 1..n).map(|k| a[r][k] * x[k]).sum();
        x[r] = (b[r] - s) / a[r][r];
    }
    Some(x)
}

// ---------------------------------------------------------------------------------------------
// applying a candidate
// ---------------------------------------------------------------------------------------------

/// One undoable batch that keys the candidate's rotations at `frame`. Locked channels are skipped (and returned);
/// an error if nothing can be written.
pub fn apply_op(project: &Project, values: &[BoneValue], frame: f32, ease: Ease) -> Result<(Op, Vec<String>), String> {
    if !frame.is_finite() || frame < 0.0 {
        return Err("frame must be a finite number >= 0".into());
    }
    let mut ops: Vec<Op> = Vec::new();
    let mut skipped = Vec::new();
    // work on a copy so several values for one channel (never expected) stay consistent
    let mut channels: Vec<Channel> = project.channels.clone();
    let mut touched: Vec<usize> = Vec::new();
    let mut added: Vec<Channel> = Vec::new();
    for v in values {
        if !v.angle.is_finite() {
            return Err(format!("angle of '{}' is not finite", v.bone));
        }
        if !project.bones.iter().any(|b| b.name == v.bone) {
            return Err(format!("unknown bone '{}'", v.bone));
        }
        if crate::chanmeta::is_locked(project, &v.bone, Prop::BoneAngle) {
            skipped.push(v.bone.clone());
            continue;
        }
        let pos = channels.iter().position(|c| c.target == v.bone && c.prop == Prop::BoneAngle && matches!(c.source, Source::Keys { .. }) && !crate::skins::is_collapse_channel(c));
        match pos {
            Some(i) => {
                if let Source::Keys { track } = &mut channels[i].source {
                    track.set_key(frame, v.angle, ease);
                }
                if !touched.contains(&i) {
                    touched.push(i);
                }
            }
            None => added.push(Channel::keys(&v.bone, Prop::BoneAngle, Track::with_keys(&[(frame, v.angle, ease)]))),
        }
    }
    if touched.is_empty() && added.is_empty() {
        return Err("nothing to apply: every proposed channel is locked".into());
    }
    let need = frame.ceil() as u32 + 1;
    if need > project.frames {
        ops.push(Op::Set { path: "/frames".into(), value: serde_json::json!(need) });
    }
    for i in touched {
        ops.push(Op::Set { path: format!("/channels/{i}"), value: serde_json::to_value(&channels[i]).map_err(|e| e.to_string())? });
    }
    for c in added {
        ops.push(Op::Insert { path: "/channels/-".into(), value: serde_json::to_value(&c).map_err(|e| e.to_string())? });
    }
    Ok((Op::Batch { ops }, skipped))
}

// ---------------------------------------------------------------------------------------------
// request / response layer shared by the CLI and the studio
// ---------------------------------------------------------------------------------------------

pub(super) fn d_top() -> usize {
    5
}

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    /// "between", "complete" or "next".
    pub kind: String,
    /// between: first / second frame and the fraction; complete / next: the frame the pose belongs at.
    #[serde(default)]
    pub a: Option<f32>,
    #[serde(default)]
    pub b: Option<f32>,
    #[serde(default)]
    pub t: Option<f32>,
    #[serde(default)]
    pub frame: Option<f32>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    #[serde(default)]
    pub free: Option<Vec<String>>,
    #[serde(default = "d_top")]
    pub top: usize,
    #[serde(default)]
    pub snap: Option<f32>,
    #[serde(default)]
    pub prior: Option<f32>,
    #[serde(default)]
    pub k: Option<usize>,
}

/// Run a request against a project. The caller builds the [`Suggester`] (from the project, a library or a saved database).
pub fn suggest(project: &Project, sg: &Suggester, req: &Request, model: Option<&Mlp>) -> Result<Suggestions, String> {
    let bones = &sg.db.bones;
    let locked: Vec<String> = bones.iter().filter(|b| crate::chanmeta::is_locked(project, b, Prop::BoneAngle)).cloned().collect();
    let mut notes: Vec<String> = Vec::new();
    let missing: Vec<&String> = bones.iter().filter(|b| !project.bones.iter().any(|x| x.name == **b)).collect();
    if !missing.is_empty() {
        notes.push(format!("{} library bone(s) are not in this project and are ignored: {}", missing.len(), missing.iter().take(6).map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
    }
    let frame_ok = |f: Option<f32>, what: &str| -> Result<f32, String> {
        let f = f.ok_or_else(|| format!("{what} is required"))?;
        if !f.is_finite() || f < 0.0 {
            return Err(format!("{what} must be a finite frame >= 0"));
        }
        Ok(f)
    };
    let (kind, mut cands, key_frame) = match req.kind.as_str() {
        "between" => {
            let (fa, fb) = (frame_ok(req.a, "a")?, frame_ok(req.b, "b")?);
            let t = req.t.unwrap_or(0.5);
            let c = sg.between(&pose_at(project, bones, fa), &pose_at(project, bones, fb), t, req.snap.unwrap_or(0.5), req.k.unwrap_or(6))?;
            ("between", c, fa + (fb - fa) * t)
        }
        "complete" => {
            let f = frame_ok(req.frame, "frame")?;
            let cur = pose_at(project, bones, f);
            let base: Vec<f32> = project.bones.iter().map(|b| pose_at(project, std::slice::from_ref(&b.name), f)[0]).collect();
            let c = sg.complete(Some(project), &req.constraints, &cur, Some(&base), req.free.as_deref(), &locked, req.prior.unwrap_or(1.0), req.top.max(1), model)?;
            ("complete", c, f)
        }
        "next" => {
            let f = frame_ok(req.frame, "frame")?;
            let c = sg.next(&pose_at(project, bones, f), f, req.k.unwrap_or(8), req.top.max(1))?;
            ("next", c, f)
        }
        k => return Err(format!("unknown suggestion kind '{k}' (between, complete, next)")),
    };
    cands.truncate(req.top.max(1));
    for c in cands.iter_mut() {
        if c.frame.is_none() {
            c.frame = Some(key_frame);
        }
    }
    let locked_skipped = sg.bind(&mut cands, project, req.free.as_deref());
    if !locked_skipped.is_empty() {
        notes.push(format!("locked channels are left alone: {}", locked_skipped.join(", ")));
    }
    Ok(Suggestions {
        kind: kind.into(),
        candidates: cands,
        notes,
        locked_skipped,
        bones: bones.clone(),
        poses_in_library: sg.db.poses.len(),
        pca_axes: sg.space.k(),
        pca_explained: sg.space.explained.last().copied().unwrap_or(0.0),
    })
}

// ---------------------------------------------------------------------------------------------
// the small-network hook
// ---------------------------------------------------------------------------------------------

/// Predict a full pose from observed bone values with a pose-completion [`Mlp`] (see `tinymlp` for the contract).
/// The model's bone list must equal `bones`.
pub fn predict_completion(m: &Mlp, bones: &[String], observed: &[Option<f32>]) -> Result<Vec<f32>, String> {
    let d = bones.len();
    if m.bones != bones {
        return Err(format!("the model was trained for bones [{}], the library has [{}]", m.bones.join(", "), bones.join(", ")));
    }
    if m.n_in() != 2 * d || m.n_out() != d {
        return Err(format!("the model has {} inputs / {} outputs, a completion model for {d} bones needs {} / {d}", m.n_in(), m.n_out(), 2 * d));
    }
    let mean = |j: usize| m.mean.get(j).copied().unwrap_or(0.0);
    let scale = |j: usize| m.scale.get(j).copied().unwrap_or(1.0);
    let mut x = vec![0.0f32; 2 * d];
    for j in 0..d {
        if let Some(v) = observed[j] {
            x[j] = (v - mean(j)) / scale(j);
            x[d + j] = 1.0;
        }
    }
    let y = m.forward(&x);
    Ok((0..d).map(|j| observed[j].unwrap_or(y[j] * scale(j) + mean(j))).collect())
}

/// Train a pose-completion network on the database: random subsets of bones are observed and the network
/// must restore the whole pose (masked-input regression, MSE weighted 1.0 on hidden bones and 0.2 on observed ones).
pub fn train_completion(db: &PoseDb, o: &TrainOpts) -> Result<(Mlp, Vec<f32>), String> {
    db.validate()?;
    let d = db.dim();
    if !(2..=64).contains(&d) {
        return Err("a completion model needs 2 to 64 bones".into());
    }
    if db.poses.len() < 4 {
        return Err("training needs at least 4 poses".into());
    }
    let n = db.poses.len();
    let mean: Vec<f32> = (0..d).map(|j| db.poses.iter().map(|p| p[j]).sum::<f32>() / n as f32).collect();
    let scale: Vec<f32> = (0..d).map(|j| (db.poses.iter().map(|p| (p[j] - mean[j]).powi(2)).sum::<f32>() / n as f32).sqrt().max(1.0)).collect();
    let poses = &db.poses;
    let (mean_c, scale_c) = (mean.clone(), scale.clone());
    let mut sample = move |rng: &mut Rng| {
        let p = &poses[rng.below(n)];
        let frac = 0.1 + 0.6 * rng.unit();
        let mut mask: Vec<bool> = (0..d).map(|_| rng.unit() < frac).collect();
        if mask.iter().all(|&m| m) {
            mask[rng.below(d)] = false;
        }
        if !mask.iter().any(|&m| m) {
            mask[rng.below(d)] = true;
        }
        let mut x = vec![0.0f32; 2 * d];
        let mut t = vec![0.0f32; d];
        let mut w = vec![0.0f32; d];
        for j in 0..d {
            let nv = (p[j] - mean_c[j]) / scale_c[j];
            t[j] = nv;
            if mask[j] {
                x[j] = nv;
                x[d + j] = 1.0;
                w[j] = 0.2;
            } else {
                w[j] = 1.0;
            }
        }
        (x, t, w)
    };
    let (layers, hist) = tinymlp::train(2 * d, d, o, n.max(32), &mut sample)?;
    let m = Mlp { format: "osatura-mlp".into(), version: 1, bones: db.bones.clone(), mean, scale, layers: layers.into_iter().collect::<Vec<Dense>>() };
    m.validate()?;
    Ok((m, hist))
}
