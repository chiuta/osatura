// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Retargeting (part of `mocap`; see the module docs in `mod.rs`).
use super::*;

// Retargeting

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scale {
    /// Rig leg length / BVH leg length.
    Auto,
    /// Rig units per BVH unit.
    Fixed(f64),
}

#[derive(Clone, Debug)]
pub struct RetargetOptions {
    pub view: View,
    pub scale: Scale,
    /// Output frame rate; `None` = project fps.
    pub fps: Option<f64>,
    /// Moving-average radius in output frames (0 = off).
    pub smooth: usize,
    /// First / last BVH frame (inclusive).
    pub start: Option<usize>,
    pub end: Option<usize>,
    /// BVH frame whose pose is the rest pose; `None` = the zero-rotation hierarchy pose.
    pub rest_frame: Option<usize>,
    /// Flip the figure left-right on the stage.
    pub mirror: bool,
    pub root_motion: bool,
    /// Drop horizontal root travel (keep the bob).
    pub in_place: bool,
    pub weight: f32,
    /// Matches below this confidence are ignored.
    pub min_confidence: f32,
    /// Root controller bone and stage ground y for [`crate::footlock`].
    pub foot_lock: Option<(String, f32)>,
}

impl Default for RetargetOptions {
    fn default() -> Self {
        RetargetOptions { view: View::Front, scale: Scale::Auto, fps: None, smooth: 0, start: None, end: None, rest_frame: None, mirror: false, root_motion: true, in_place: false, weight: 1.0, min_confidence: 0.4, foot_lock: None }
    }
}

#[derive(Clone, Debug)]
pub struct RetargetResult {
    pub channels: Vec<Channel>,
    /// `target.prop` of channels not written because they are locked.
    pub blocked: Vec<String>,
    /// Frames of animation produced.
    pub out_frames: usize,
    /// Project length needed (>= out_frames); `bake` extends the project to this.
    pub project_frames: u32,
    pub scale: f64,
    pub bones_used: usize,
    /// Source frames where some bone pointed along the view axis and held its last angle.
    pub degenerate: usize,
    pub notes: Vec<String>,
}

pub(super) fn norm360(a: f64) -> f64 {
    let mut x = a % 360.0;
    if x > 180.0 {
        x -= 360.0;
    } else if x <= -180.0 {
        x += 360.0;
    }
    x
}

pub(super) fn lerp_at(v: &[f64], x: f64) -> f64 {
    let i0 = (x.floor().max(0.0) as usize).min(v.len() - 1);
    let i1 = (i0 + 1).min(v.len() - 1);
    let t = (x - i0 as f64).clamp(0.0, 1.0);
    v[i0] + (v[i1] - v[i0]) * t
}

pub(super) fn smooth_series(v: &mut [f64], radius: usize) {
    let n = v.len();
    if radius == 0 || n < 3 {
        return;
    }
    let src = v.to_vec();
    for i in 0..n {
        let r = radius.min(i).min(n - 1 - i);
        let sum: f64 = src[i - r..=i + r].iter().sum();
        v[i] = sum / (2 * r + 1) as f64;
    }
}

pub fn retarget(p: &Project, bvh: &Bvh, mr: &MatchResult, o: &RetargetOptions) -> Result<RetargetResult, String> {
    if bvh.frames == 0 {
        return Err("BVH has no frames".into());
    }
    let fps = o.fps.unwrap_or(p.fps as f64);
    if !(fps.is_finite() && fps > 0.0 && fps <= 1000.0) {
        return Err("fps must be in (0, 1000]".into());
    }
    if o.smooth > 1000 {
        return Err("smooth radius must be at most 1000".into());
    }
    if !(o.weight.is_finite() && (0.0..=1.0).contains(&o.weight)) {
        return Err("weight must be in 0..1".into());
    }
    if let Scale::Fixed(s) = o.scale {
        if !(s.is_finite() && s > 0.0) {
            return Err("scale must be positive".into());
        }
    }
    let s0 = o.start.unwrap_or(0);
    let e0 = o.end.unwrap_or(bvh.frames - 1);
    if s0 > e0 || e0 >= bvh.frames {
        return Err(format!("frame range {s0}..{e0} does not fit the {} BVH frames", bvh.frames));
    }
    if let Some(rf) = o.rest_frame {
        if rf >= bvh.frames {
            return Err(format!("rest frame {rf} is outside the {} BVH frames", bvh.frames));
        }
    }
    let used: Vec<&BoneMatch> = mr.matches.iter().filter(|m| m.confidence >= o.min_confidence && m.rig < p.bones.len() && m.bvh < bvh.joints.len()).collect();
    if used.is_empty() {
        return Err("no bone matches to retarget (all below the confidence threshold?)".into());
    }
    let mut notes = Vec::new();
    let nsel = e0 - s0 + 1;
    let rest_pose = bvh.pose(o.rest_frame);
    let rest_ang: Vec<Option<f64>> = used.iter().map(|m| bvh.tip_point(&rest_pose, m.tip).and_then(|t| angle_2d(rest_pose.pos[m.bvh], t, o.view, o.mirror))).collect();
    for (m, r) in used.iter().zip(&rest_ang) {
        if r.is_none() {
            notes.push(format!("{}: the rest pose bone points along the view axis; this bone stays at its rest angle", m.rig_name));
        }
    }
    // per-frame deltas
    let mut series: Vec<Vec<f64>> = vec![Vec::with_capacity(nsel); used.len()];
    let mut prev: Vec<Option<f64>> = vec![None; used.len()];
    let hips_match = used.iter().find(|m| m.slot == "hips");
    let mut hip_xy: Vec<[f64; 2]> = Vec::with_capacity(nsel);
    let mut degenerate = 0usize;
    let proj = |pt: V3| {
        let mut q = o.view.project(pt);
        if o.mirror {
            q[0] = -q[0];
        }
        q
    };
    for f in s0..=e0 {
        let pose = bvh.pose(Some(f));
        for (i, m) in used.iter().enumerate() {
            let ang = bvh.tip_point(&pose, m.tip).and_then(|t| angle_2d(pose.pos[m.bvh], t, o.view, o.mirror));
            let d = match (ang, rest_ang[i]) {
                (Some(a), Some(r)) => {
                    let mut d = norm360(a - r);
                    if let Some(pv) = prev[i] {
                        while d - pv > 180.0 {
                            d -= 360.0;
                        }
                        while d - pv < -180.0 {
                            d += 360.0;
                        }
                    }
                    prev[i] = Some(d);
                    d
                }
                (None, Some(_)) => {
                    degenerate += 1;
                    prev[i].unwrap_or(0.0)
                }
                _ => 0.0,
            };
            series[i].push(d);
        }
        if let Some(h) = hips_match {
            hip_xy.push(proj(pose.pos[h.bvh]));
        }
    }
    if degenerate > 0 {
        notes.push(format!("{degenerate} bone-frames pointed along the view axis and held their last angle (2D projection loses depth)"));
    }
    let hip_ref = match (hips_match, o.rest_frame) {
        (Some(h), Some(_)) => Some(proj(rest_pose.pos[h.bvh])),
        (Some(_), None) => hip_xy.first().copied(),
        _ => None,
    };

    // scale
    let scale = match o.scale {
        Scale::Fixed(s) => s,
        Scale::Auto => {
            let mut ratios = Vec::new();
            for (a, b) in [("thigh", "shin"), ("upperarm", "forearm")] {
                for side in ["l", "r"] {
                    let (ka, kb) = (format!("{a}.{side}"), format!("{b}.{side}"));
                    if let (Some(ma), Some(mb)) = (used.iter().find(|m| m.slot == ka), used.iter().find(|m| m.slot == kb)) {
                        let tip_b = bvh.tip_point(&rest_pose, mb.tip);
                        let bl = dist3(rest_pose.pos[ma.bvh], rest_pose.pos[mb.bvh]) + tip_b.map_or(0.0, |t| dist3(rest_pose.pos[mb.bvh], t));
                        let rl = (p.bones[ma.rig].length + p.bones[mb.rig].length) as f64;
                        if bl > 1e-9 && rl > 1e-9 {
                            ratios.push(rl / bl);
                        }
                    }
                }
                if !ratios.is_empty() {
                    break; // legs preferred; arms only as a fallback
                }
            }
            match median(&mut ratios) {
                Some(s) => s,
                None => {
                    notes.push("no matched limb to derive the root-travel scale from; using 1.0 (pass --scale)".into());
                    1.0
                }
            }
        }
    };

    // resample to output frames
    let span = (e0 - s0) as f64;
    let out_n = if e0 == s0 { 1 } else { (span * bvh.frame_time * fps + 1e-6).floor() as usize + 1 };
    if out_n > 1_000_000 {
        return Err("resulting animation would be longer than 1,000,000 frames".into());
    }
    let step = 1.0 / (fps * bvh.frame_time);
    let pos_of = |k: usize| (k as f64 * step).min(span);
    let mut dl: Vec<Vec<f64>> = series.iter().map(|s| (0..out_n).map(|k| lerp_at(s, pos_of(k))).collect()).collect();
    for d in dl.iter_mut() {
        smooth_series(d, o.smooth);
    }
    let (mut hx, mut hy) = (vec![0.0; out_n], vec![0.0; out_n]);
    if let (Some(r), true) = (hip_ref, hips_match.is_some()) {
        let xs: Vec<f64> = hip_xy.iter().map(|q| q[0] - r[0]).collect();
        let ys: Vec<f64> = hip_xy.iter().map(|q| q[1] - r[1]).collect();
        for k in 0..out_n {
            hx[k] = lerp_at(&xs, pos_of(k)) * scale;
            hy[k] = lerp_at(&ys, pos_of(k)) * scale;
        }
        smooth_series(&mut hx, o.smooth);
        smooth_series(&mut hy, o.smooth);
        if o.in_place {
            hx.iter_mut().for_each(|v| *v = 0.0);
        }
    }

    // rig-space angles
    let rw = crate::rigedit::rest_world(p)?;
    let nbn = p.bones.len();
    let parent: Vec<Option<usize>> = p.bones.iter().map(|b| b.parent.as_deref().and_then(|n| p.bones.iter().position(|x| x.name == n))).collect();
    let mut mi_of: Vec<Option<usize>> = vec![None; nbn];
    for (i, m) in used.iter().enumerate() {
        mi_of[m.rig] = Some(i);
    }
    let mut local: Vec<Vec<f32>> = vec![Vec::with_capacity(out_n); nbn];
    let mut prev_local: Vec<Option<f64>> = vec![None; nbn];
    let mut root_off: Vec<[f32; 2]> = Vec::with_capacity(out_n);
    let hips_rig = hips_match.map(|h| h.rig);
    for k in 0..out_n {
        let mut wa = vec![0.0f64; nbn];
        for b in 0..nbn {
            let pw = parent[b].map_or(0.0, |q| wa[q]);
            let rest_local = p.bones[b].rest_angle as f64;
            let d = match mi_of[b] {
                Some(mi) => {
                    let mut d = norm360(rw[b].1 as f64 + dl[mi][k] - pw - rest_local);
                    if let Some(pv) = prev_local[b] {
                        while d - pv > 180.0 {
                            d -= 360.0;
                        }
                        while d - pv < -180.0 {
                            d += 360.0;
                        }
                    }
                    prev_local[b] = Some(d);
                    d
                }
                None => 0.0,
            };
            if mi_of[b].is_some() {
                local[b].push(d as f32);
            }
            wa[b] = pw + rest_local + d;
        }
        if let (true, Some(hb)) = (o.root_motion, hips_rig) {
            let pw = parent[hb].map_or(0.0, |q| wa[q]).to_radians();
            let (s, c) = pw.sin_cos();
            let (dx, dy) = (hx[k], hy[k]);
            root_off.push([(dx * c + dy * s) as f32, (-dx * s + dy * c) as f32]);
        }
    }

    // channels
    let key = |vals: &[f32]| -> Track {
        let ks: Vec<(f32, f32, Ease)> = vals.iter().enumerate().map(|(k, &v)| (k as f32, v, Ease::Linear)).collect();
        Track::with_keys(&ks)
    };
    let mut chans: Vec<Channel> = Vec::new();
    for b in 0..nbn {
        if mi_of[b].is_some() {
            chans.push(Channel::keys(&p.bones[b].name, Prop::BoneAngle, key(&local[b])).with_weight(o.weight));
        }
    }
    if let (true, Some(hb)) = (o.root_motion, hips_rig) {
        let xs: Vec<f32> = root_off.iter().map(|v| v[0]).collect();
        let ys: Vec<f32> = root_off.iter().map(|v| v[1]).collect();
        chans.push(Channel::keys(&p.bones[hb].name, Prop::BoneOffsetX, key(&xs)).with_weight(o.weight));
        chans.push(Channel::keys(&p.bones[hb].name, Prop::BoneOffsetY, key(&ys)).with_weight(o.weight));
    } else if o.root_motion && hips_rig.is_none() {
        notes.push("no hips match: root travel not transferred".into());
    }
    let bones_used = used.len();
    let (mut chans, blocked) = crate::chanmeta::split_unlocked(p, chans);
    if !blocked.is_empty() {
        notes.push(format!("locked channels skipped: {}", blocked.join(", ")));
    }
    let project_frames = p.frames.max(out_n as u32);
    if project_frames > p.frames {
        notes.push(format!("project length extended from {} to {} frames", p.frames, project_frames));
    }
    if let Some((bone, ground)) = &o.foot_lock {
        let mut tmp = p.clone();
        let base = tmp.channels.len();
        tmp.channels.extend(chans.iter().cloned());
        tmp.frames = project_frames;
        let lo = crate::footlock::LockOptions { ground_y: *ground, ..Default::default() };
        let contacts = crate::footlock::lock_feet(&mut tmp, bone, &lo)?;
        let extra: Vec<Channel> = tmp.channels[base + chans.len()..].to_vec();
        notes.push(format!("foot lock on '{bone}': {} contact(s) pinned", contacts.len()));
        chans.extend(extra);
    }
    Ok(RetargetResult { channels: chans, blocked, out_frames: out_n, project_frames, scale, bones_used, degenerate, notes })
}

/// Append the retargeted channels (and extend the project length if the clip is longer).
pub fn bake(p: &mut Project, r: &RetargetResult) {
    p.channels.extend(r.channels.iter().cloned());
    p.frames = p.frames.max(r.project_frames);
}

/// The same change as one editor operation, so a single undo removes the whole retarget.
pub fn bake_op(p: &Project, r: &RetargetResult) -> Result<Op, String> {
    let mut ops = Vec::new();
    if r.project_frames > p.frames {
        ops.push(Op::Set { path: "/frames".into(), value: serde_json::json!(r.project_frames) });
    }
    for c in &r.channels {
        ops.push(Op::Insert { path: "/channels/-".into(), value: serde_json::to_value(c).map_err(|e| e.to_string())? });
    }
    Ok(Op::Batch { ops })
}
