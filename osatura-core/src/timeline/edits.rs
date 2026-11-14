// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `timeline` (see the module docs in `mod.rs`).
use super::*;

/// Only `frames` matters to `check_frame`; avoid cloning the whole project.
pub(super) fn shallow(p: &Project) -> Project {
    Project::new("", p.width, p.height, p.fps, p.frames)
}

/// Shift whole channels in time.
pub fn offset_channels(p: &Project, chans: &[usize], dt: f32, o: &EditOpts) -> Result<EditPlan, String> {
    finite(dt, "dt")?;
    if chans.is_empty() {
        return Err("no channels given".into());
    }
    let dt = snapf(o, dt);
    let mut w = Work::new(p);
    let mut changed = 0;
    for &ch in chans {
        let t = w.existing(ch)?;
        for k in &mut t.keys {
            k.frame += dt;
            check_frame(p, o, k.frame, "offset")?;
            changed += 1;
        }
    }
    w.finish(Vec::new(), changed)
}

/// Set what channels do outside their key range.
pub fn set_extrapolate(p: &Project, chans: &[usize], e: Extrapolate) -> Result<EditPlan, String> {
    if chans.is_empty() {
        return Err("no channels given".into());
    }
    let mut w = Work::new(p);
    let mut changed = 0;
    for &ch in chans {
        w.existing(ch)?.extrapolate = e;
        changed += 1;
    }
    w.finish(Vec::new(), changed)
}

/// Make channels loop: optionally close the cycle with a key on `end` holding the first key's value,
/// then set the extrapolation (default `Loop`).
pub fn make_loop(p: &Project, chans: &[usize], end: Option<f32>, e: Extrapolate, o: &EditOpts) -> Result<EditPlan, String> {
    if chans.is_empty() {
        return Err("no channels given".into());
    }
    let mut w = Work::new(p);
    let mut out = Vec::new();
    let mut changed = 0;
    for &ch in chans {
        let t = w.existing(ch)?;
        if t.keys.len() < 2 {
            return Err(format!("channel {ch} needs at least two keys to loop"));
        }
        if let Some(end) = end {
            let end = snapf(o, end);
            check_frame(p, o, end, "loop end")?;
            let first = t.keys[0];
            if end <= first.frame {
                return Err(format!("loop end {end} must be after the first key ({})", first.frame));
            }
            if t.keys.last().is_some_and(|l| l.frame > end) {
                return Err(format!("loop end {end} is before the last key of channel {ch}"));
            }
            t.set_key(end, first.value, first.ease);
            out.push((Loc::Existing(ch), end));
        }
        t.extrapolate = e;
        changed += 1;
    }
    w.finish(out, changed)
}

/// Move one Bezier handle of the key at `frame` on channel `ch`.
pub fn move_handle(p: &Project, ch: usize, frame: f32, side: Side, pos: HandlePos, mode: HandleMode) -> Result<EditPlan, String> {
    keyed_track(p, ch)?;
    let mut w = Work::new(p);
    let t = w.existing(ch)?;
    let i = t.keys.iter().position(|k| k.frame == frame).ok_or_else(|| format!("no key at frame {frame} on channel {ch}"))?;
    g::set_handle(t, i, side, pos, mode)?;
    w.finish(vec![(Loc::Existing(ch), frame)], 1)
}

/// Auto-smooth the segments leaving the selected keys.
pub fn smooth_keys(p: &Project, sel: &[KeyRef], kind: TangentKind) -> Result<EditPlan, String> {
    let by = gather(p, sel)?;
    let mut w = Work::new(p);
    let mut out = Vec::new();
    let mut changed = 0;
    for (ch, frames) in &by {
        let t = w.existing(*ch)?;
        let starts: Vec<usize> = t.keys.iter().enumerate().filter(|(_, k)| frames.contains(&k.frame)).map(|(i, _)| i).collect();
        changed += g::smooth_segments(t, &starts, kind);
        out.extend(frames.iter().map(|&f| (Loc::Existing(*ch), f)));
    }
    w.finish(out, changed)
}

/// Channel flags (lock / shy / solo / colour) as an edit, through `chanmeta::set_meta`.
pub fn set_flags(p: &Project, target: &str, prop: Option<Prop>, c: &chanmeta::Change) -> Result<EditPlan, String> {
    let mut q = shallow(p);
    q.channels = p.channels.clone();
    q.channel_meta = p.channel_meta.clone();
    chanmeta::set_meta(&mut q, target, prop, c)?;
    let v = serde_json::to_value(&q.channel_meta).map_err(|e| e.to_string())?;
    Ok(EditPlan { op: Some(Op::Batch { ops: vec![Op::Set { path: "/channel_meta".into(), value: v }] }), changed: 1, ..Default::default() })
}

// ---------------------------------------------------------------- one JSON-able op enum

/// Everything the studio and the CLI can ask of the timeline, as JSON (`{"op": "move_keys", ...}`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum TlOp {
    MoveKeys { keys: Vec<KeyRef>, #[serde(default)] dt: f32, #[serde(default)] dv: f32, #[serde(default)] opts: EditOpts },
    ScaleKeys { keys: Vec<KeyRef>, pivot: f32, factor: f32, #[serde(default)] opts: EditOpts },
    Copy { keys: Vec<KeyRef> },
    Paste { clip: Clip, at: f32, #[serde(default)] remap: Vec<Remap>, #[serde(default)] opts: EditOpts },
    Duplicate { keys: Vec<KeyRef>, offset: f32, #[serde(default)] opts: EditOpts },
    Delete { keys: Vec<KeyRef> },
    SetEase { keys: Vec<KeyRef>, ease: Ease },
    SetValue { keys: Vec<KeyRef>, value: f32 },
    AddKey { target: String, prop: Prop, frame: f32, #[serde(default)] value: Option<f32>, #[serde(default)] ease: Option<Ease>, #[serde(default)] opts: EditOpts },
    Ripple { at: f32, count: f32, mode: RetimeMode, #[serde(default)] scope: Option<Vec<usize>>, #[serde(default)] adjust_frames: bool, #[serde(default)] opts: EditOpts },
    Offset { channels: Vec<usize>, dt: f32, #[serde(default)] opts: EditOpts },
    Extrapolate { channels: Vec<usize>, extrapolate: Extrapolate },
    MakeLoop { channels: Vec<usize>, #[serde(default)] end: Option<f32>, #[serde(default)] extrapolate: Option<Extrapolate>, #[serde(default)] opts: EditOpts },
    Handle { ch: usize, frame: f32, side: Side, t: f32, v: f32, #[serde(default)] mode: HandleMode },
    Smooth { keys: Vec<KeyRef>, kind: TangentKind },
    Flags { target: String, #[serde(default)] prop: Option<Prop>, #[serde(default)] lock: Option<bool>, #[serde(default)] shy: Option<bool>, #[serde(default)] solo: Option<bool>, #[serde(default)] color: Option<String> },
}

/// Plan any [`TlOp`] against the project.
pub fn plan(p: &Project, op: &TlOp) -> Result<EditPlan, String> {
    match op {
        TlOp::MoveKeys { keys, dt, dv, opts } => move_keys(p, keys, *dt, *dv, opts),
        TlOp::ScaleKeys { keys, pivot, factor, opts } => scale_keys(p, keys, *pivot, *factor, opts),
        TlOp::Copy { keys } => Ok(EditPlan { clip: Some(copy_keys(p, keys)?), selection: keys.clone(), ..Default::default() }),
        TlOp::Paste { clip, at, remap, opts } => paste_clip(p, clip, *at, remap, opts),
        TlOp::Duplicate { keys, offset, opts } => duplicate_keys(p, keys, *offset, opts),
        TlOp::Delete { keys } => delete_keys(p, keys),
        TlOp::SetEase { keys, ease } => set_ease(p, keys, *ease),
        TlOp::SetValue { keys, value } => set_value(p, keys, *value),
        TlOp::AddKey { target, prop, frame, value, ease, opts } => add_key(p, target, *prop, *frame, *value, ease.unwrap_or(Ease::Linear), opts),
        TlOp::Ripple { at, count, mode, scope, adjust_frames, opts } => ripple(p, *at, *count, *mode, scope.as_deref(), *adjust_frames, opts),
        TlOp::Offset { channels, dt, opts } => offset_channels(p, channels, *dt, opts),
        TlOp::Extrapolate { channels, extrapolate } => set_extrapolate(p, channels, *extrapolate),
        TlOp::MakeLoop { channels, end, extrapolate, opts } => make_loop(p, channels, *end, extrapolate.unwrap_or(Extrapolate::Loop), opts),
        TlOp::Handle { ch, frame, side, t, v, mode } => move_handle(p, *ch, *frame, *side, HandlePos { t: *t, v: *v }, *mode),
        TlOp::Smooth { keys, kind } => smooth_keys(p, keys, *kind),
        TlOp::Flags { target, prop, lock, shy, solo, color } => set_flags(p, target, *prop, &chanmeta::Change { color: color.clone(), shy: *shy, solo: *solo, lock: *lock }),
    }
}

/// Plan and apply in one call (one undo step). Returns the plan; a plan without an op is not applied.
pub fn apply(ed: &mut crate::edit::Editor, op: &TlOp) -> Result<EditPlan, Vec<String>> {
    let pl = plan(ed.project(), op).map_err(|e| vec![e])?;
    if let Some(o) = &pl.op {
        ed.apply(o.clone())?;
    }
    Ok(pl)
}

// ---------------------------------------------------------------- graph view

#[derive(Clone, Debug, Serialize)]
pub struct GraphKey {
    pub frame: f32,
    pub value: f32,
    pub ease: Ease,
    #[serde(rename = "in")]
    pub inn: Option<HandlePos>,
    pub out: Option<HandlePos>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphChannel {
    pub ch: usize,
    pub target: String,
    pub prop: Prop,
    pub color: Option<String>,
    pub locked: bool,
    pub samples: Vec<[f32; 2]>,
    pub keys: Vec<GraphKey>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphView {
    pub f0: f32,
    pub f1: f32,
    /// Auto-fitted `[lo, hi]` over all returned channels.
    pub range: [f32; 2],
    pub channels: Vec<GraphChannel>,
    pub skipped: Vec<String>,
}

/// Curves, keys and handles for the graph editor. Non-keyed channels are listed in `skipped`.
pub fn graph_view(p: &Project, chans: &[usize], f0: f32, f1: f32, samples: usize) -> Result<GraphView, String> {
    if (f1.partial_cmp(&f0) != Some(std::cmp::Ordering::Greater)) || !f0.is_finite() || !f1.is_finite() {
        return Err("graph range must be f0 < f1".into());
    }
    let m = build(p);
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    let mut out = Vec::new();
    let mut skipped = Vec::new();
    for &ch in chans.iter().take(64) {
        let c = p.channels.get(ch).ok_or_else(|| format!("no channel {ch}"))?;
        let Some(t) = keys_of(c).filter(|_| is_keyed(c)) else {
            skipped.push(format!("channel {ch} ({}.{}) has no editable curve", c.target, prop_name(c.prop)));
            continue;
        };
        if let Some((a, b)) = g::value_range(t, f0, f1, samples, 0.0) {
            lo = lo.min(a);
            hi = hi.max(b);
        }
        let hs = g::handles(t);
        out.push(GraphChannel {
            ch,
            target: c.target.clone(),
            prop: c.prop,
            color: m.tracks[ch].color.clone(),
            locked: m.tracks[ch].locked,
            samples: g::sample_track(t, f0, f1, samples).into_iter().map(|(a, b)| [a, b]).collect(),
            keys: t.keys.iter().zip(hs).map(|(k, h)| GraphKey { frame: k.frame, value: k.value, ease: k.ease, inn: h.inn, out: h.out }).collect(),
        });
    }
    let range = if lo.is_finite() && hi.is_finite() {
        let pad = if hi - lo > 1e-6 { (hi - lo) * 0.08 } else { 0.5 };
        [lo - pad, hi + pad]
    } else {
        [-1.0, 1.0]
    };
    Ok(GraphView { f0, f1, range, channels: out, skipped })
}

// ---------------------------------------------------------------- tests
