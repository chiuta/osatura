// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `timeline` (see the module docs in `mod.rs`).
use super::*;

pub(super) struct Work<'a> {
    pub(super) p: &'a Project,
    pub(super) edited: BTreeMap<usize, Track>,
    pub(super) fresh: Vec<(String, Prop, Track)>,
    pub(super) extra: Vec<Op>,
    pub(super) notes: Vec<String>,
}

pub(super) fn keys_of(c: &Channel) -> Option<&Track> {
    match &c.source {
        Source::Keys { track } => Some(track),
        _ => None,
    }
}

impl<'a> Work<'a> {
    pub(super) fn new(p: &'a Project) -> Self {
        Work { p, edited: BTreeMap::new(), fresh: Vec::new(), extra: Vec::new(), notes: Vec::new() }
    }

    pub(super) fn check(&self, ch: usize) -> Result<&'a Channel, String> {
        let c = self.p.channels.get(ch).ok_or_else(|| format!("no channel {ch}"))?;
        if keys_of(c).is_none() {
            return Err(format!("channel {ch} ({}.{}) is not a keyed channel", c.target, prop_name(c.prop)));
        }
        if crate::skins::is_collapse_channel(c) {
            return Err(format!("channel {ch} ({}) is a skin helper and cannot be edited here", c.target));
        }
        chanmeta::assert_unlocked(self.p, &c.target, c.prop)?;
        Ok(c)
    }

    pub(super) fn existing(&mut self, ch: usize) -> Result<&mut Track, String> {
        let c = self.check(ch)?;
        Ok(self.edited.entry(ch).or_insert_with(|| keys_of(c).cloned().unwrap_or_default()))
    }

    pub(super) fn locate(&mut self, target: &str, prop: Prop) -> Result<Loc, String> {
        chanmeta::assert_unlocked(self.p, target, prop)?;
        if let Some(i) = self.p.channels.iter().position(|c| c.target == target && c.prop == prop && is_keyed(c)) {
            self.existing(i)?;
            return Ok(Loc::Existing(i));
        }
        if let Some(j) = self.fresh.iter().position(|(t, pr, _)| t == target && *pr == prop) {
            return Ok(Loc::Fresh(j));
        }
        self.fresh.push((target.into(), prop, Track::new()));
        Ok(Loc::Fresh(self.fresh.len() - 1))
    }

    pub(super) fn track_at(&mut self, l: Loc) -> &mut Track {
        match l {
            Loc::Existing(i) => self.edited.get_mut(&i).expect("located before use"),
            Loc::Fresh(j) => &mut self.fresh[j].2,
        }
    }

    pub(super) fn finish(self, sel: Vec<(Loc, f32)>, changed: usize) -> Result<EditPlan, String> {
        let mut ops: Vec<Op> = Vec::new();
        let mut removed: Vec<usize> = Vec::new();
        for (i, t) in &self.edited {
            let orig = keys_of(&self.p.channels[*i]).expect("checked");
            if t.keys.is_empty() {
                removed.push(*i);
            } else if t != orig {
                ops.push(Op::Set { path: format!("/channels/{i}/track"), value: serde_json::to_value(t).map_err(|e| e.to_string())? });
            }
        }
        let mut fresh_pos: BTreeMap<usize, usize> = BTreeMap::new();
        for (j, (target, prop, t)) in self.fresh.iter().enumerate() {
            if t.keys.is_empty() {
                continue;
            }
            fresh_pos.insert(j, fresh_pos.len());
            let ch = Channel::keys(target, *prop, t.clone());
            ops.push(Op::Insert { path: "/channels/-".into(), value: serde_json::to_value(&ch).map_err(|e| e.to_string())? });
        }
        ops.extend(self.extra);
        removed.sort_unstable_by(|a, b| b.cmp(a));
        for i in &removed {
            ops.push(Op::Remove { path: format!("/channels/{i}") });
        }
        let n_after = self.p.channels.len() - removed.len();
        let mut selection: Vec<KeyRef> = sel
            .into_iter()
            .filter_map(|(l, f)| match l {
                Loc::Existing(i) if !removed.contains(&i) => Some(KeyRef { ch: i - removed.iter().filter(|&&r| r < i).count(), frame: f }),
                Loc::Existing(_) => None,
                Loc::Fresh(j) => fresh_pos.get(&j).map(|&k| KeyRef { ch: n_after + k, frame: f }),
            })
            .collect();
        normalize(&mut selection);
        Ok(EditPlan { op: if ops.is_empty() { None } else { Some(Op::Batch { ops }) }, selection, changed, notes: self.notes, clip: None })
    }
}

pub(super) fn keyed_track(p: &Project, ch: usize) -> Result<&Track, String> {
    let c = p.channels.get(ch).ok_or_else(|| format!("no channel {ch}"))?;
    if !is_keyed(c) {
        return Err(format!("channel {ch} ({}.{}) is not an editable keyed channel", c.target, prop_name(c.prop)));
    }
    Ok(keys_of(c).expect("keyed"))
}

/// Validate a selection against the project; group the frames per channel.
pub(super) fn gather(p: &Project, sel: &[KeyRef]) -> Result<BTreeMap<usize, Vec<f32>>, String> {
    if sel.is_empty() {
        return Err("no keys selected".into());
    }
    let mut by: BTreeMap<usize, Vec<f32>> = BTreeMap::new();
    for r in sel {
        let t = keyed_track(p, r.ch)?;
        if !r.frame.is_finite() || !t.keys.iter().any(|k| k.frame == r.frame) {
            return Err(format!("stale key reference (channel {}, frame {})", r.ch, r.frame));
        }
        let v = by.entry(r.ch).or_default();
        if !v.contains(&r.frame) {
            v.push(r.frame);
        }
    }
    for v in by.values_mut() {
        v.sort_by(|a, b| a.total_cmp(b));
    }
    Ok(by)
}

pub(super) fn take_keys(t: &mut Track, frames: &[f32]) -> Vec<Key> {
    let mut taken = Vec::new();
    t.keys.retain(|k| {
        if frames.contains(&k.frame) {
            taken.push(*k);
            false
        } else {
            true
        }
    });
    taken
}

pub(super) fn snapf(o: &EditOpts, f: f32) -> f32 {
    if o.snap {
        let r = f.round();
        if r == 0.0 {
            0.0
        } else {
            r
        }
    } else {
        f
    }
}

pub(super) fn check_frame(p: &Project, o: &EditOpts, f: f32, what: &str) -> Result<(), String> {
    if !f.is_finite() {
        return Err(format!("{what}: frame is not finite"));
    }
    if o.bounds == Bounds::Project && (f < 0.0 || f > p.frames as f32) {
        return Err(format!("{what}: frame {f} is outside the project range 0..={}", p.frames));
    }
    Ok(())
}

/// Put `incoming` keys into `t`; returns the frames that were actually written.
pub(super) fn place(t: &mut Track, mut incoming: Vec<Key>, c: Collision, notes: &mut Vec<String>) -> Result<Vec<f32>, String> {
    incoming.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    let mut uniq: Vec<Key> = Vec::new();
    for k in incoming {
        if uniq.last().is_some_and(|m| m.frame == k.frame) {
            if c == Collision::Refuse {
                return Err(format!("two keys would land on frame {}", k.frame));
            }
            notes.push(format!("two keys landed on frame {}: the earlier one was kept", k.frame));
            continue;
        }
        uniq.push(k);
    }
    let mut written = Vec::new();
    for k in uniq {
        match t.keys.binary_search_by(|x| x.frame.total_cmp(&k.frame)) {
            Ok(i) => match c {
                Collision::Replace => {
                    t.keys[i] = k;
                    written.push(k.frame);
                }
                Collision::Merge => notes.push(format!("frame {} already had a key: kept it, dropped the incoming one", k.frame)),
                Collision::Refuse => return Err(format!("a key already exists at frame {}", k.frame)),
            },
            Err(i) => {
                t.keys.insert(i, k);
                written.push(k.frame);
            }
        }
    }
    Ok(written)
}

pub(super) fn finite(v: f32, what: &str) -> Result<(), String> {
    if v.is_finite() {
        Ok(())
    } else {
        Err(format!("{what} must be a finite number"))
    }
}

pub(super) fn validate_ease(e: &Ease) -> Result<(), String> {
    match *e {
        Ease::CubicBezier { x1, y1, x2, y2 } => {
            if ![x1, y1, x2, y2].iter().all(|v| v.is_finite()) || !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
                return Err("cubic-bezier needs finite values with x1 and x2 in 0..1".into());
            }
        }
        Ease::Spring { bounces, decay } => finite(bounces + decay, "spring parameters")?,
        Ease::Back { overshoot } => finite(overshoot, "back overshoot")?,
        _ => {}
    }
    Ok(())
}

// ---------------------------------------------------------------- operations

/// Move the selected keys by `dt` frames (snapped when `opts.snap`) and `dv` in value.
pub fn move_keys(p: &Project, sel: &[KeyRef], dt: f32, dv: f32, o: &EditOpts) -> Result<EditPlan, String> {
    finite(dt, "dt")?;
    finite(dv, "dv")?;
    let by = gather(p, sel)?;
    let mut w = Work::new(p);
    let mut out = Vec::new();
    let mut changed = 0;
    for (ch, frames) in &by {
        let t = w.existing(*ch)?;
        let mut moved = take_keys(t, frames);
        for k in &mut moved {
            if dt != 0.0 {
                k.frame = snapf(o, k.frame + dt);
            }
            k.value += dv;
            finite(k.value, "moved value")?;
        }
        for k in &moved {
            check_frame(p, o, k.frame, "move")?;
        }
        let mut notes = Vec::new();
        let written = place(t, moved, o.collision, &mut notes)?;
        w.notes.extend(notes);
        changed += written.len();
        out.extend(written.into_iter().map(|f| (Loc::Existing(*ch), f)));
    }
    if dt == 0.0 && dv == 0.0 {
        return Ok(EditPlan { selection: sel.to_vec(), ..Default::default() });
    }
    w.finish(out, changed)
}

/// Scale the spacing of the selected keys about `pivot` (factor 2 = twice as slow).
pub fn scale_keys(p: &Project, sel: &[KeyRef], pivot: f32, factor: f32, o: &EditOpts) -> Result<EditPlan, String> {
    finite(pivot, "pivot")?;
    if (factor.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !factor.is_finite() {
        return Err("scale factor must be a positive finite number".into());
    }
    let by = gather(p, sel)?;
    let mut w = Work::new(p);
    let mut out = Vec::new();
    let mut changed = 0;
    for (ch, frames) in &by {
        let t = w.existing(*ch)?;
        let mut moved = take_keys(t, frames);
        for k in &mut moved {
            k.frame = snapf(o, pivot + (k.frame - pivot) * factor);
            check_frame(p, o, k.frame, "scale")?;
        }
        let mut notes = Vec::new();
        let written = place(t, moved, o.collision, &mut notes)?;
        w.notes.extend(notes);
        changed += written.len();
        out.extend(written.into_iter().map(|f| (Loc::Existing(*ch), f)));
    }
    w.finish(out, changed)
}

/// Copy the selected keys into a clip (not an edit).
pub fn copy_keys(p: &Project, sel: &[KeyRef]) -> Result<Clip, String> {
    let by = gather(p, sel)?;
    let origin = by.values().flat_map(|v| v.iter().copied()).fold(f32::INFINITY, f32::min);
    let mut tracks: Vec<ClipTrack> = Vec::new();
    for (ch, frames) in &by {
        let c = &p.channels[*ch];
        let t = keyed_track(p, *ch)?;
        let keys: Vec<Key> = t.keys.iter().filter(|k| frames.contains(&k.frame)).map(|k| Key { frame: k.frame - origin, ..*k }).collect();
        match tracks.iter_mut().find(|x| x.target == c.target && x.prop == c.prop) {
            Some(x) => {
                x.keys.extend(keys);
                x.keys.sort_by(|a, b| a.frame.total_cmp(&b.frame));
                x.keys.dedup_by(|b, a| a.frame == b.frame);
            }
            None => tracks.push(ClipTrack { target: c.target.clone(), prop: c.prop, keys }),
        }
    }
    Ok(Clip { tracks })
}

pub(super) fn validate_clip(c: &Clip) -> Result<(), String> {
    if c.tracks.is_empty() || c.key_count() == 0 {
        return Err("the clip is empty".into());
    }
    if c.key_count() > 100_000 {
        return Err("the clip is too large".into());
    }
    for t in &c.tracks {
        for k in &t.keys {
            if !k.frame.is_finite() || !k.value.is_finite() {
                return Err("the clip has a non-finite key".into());
            }
            validate_ease(&k.ease)?;
        }
    }
    Ok(())
}

/// Paste a clip with its first key at frame `at`. `remap` redirects clip tracks to other channels
/// (the property may change only within the same target kind).
pub fn paste_clip(p: &Project, clip: &Clip, at: f32, remap: &[Remap], o: &EditOpts) -> Result<EditPlan, String> {
    finite(at, "at")?;
    validate_clip(clip)?;
    for r in remap {
        if !clip.tracks.iter().any(|t| t.target == r.from_target && t.prop == r.from_prop) {
            return Err(format!("remap source {}.{} is not in the clip", r.from_target, prop_name(r.from_prop)));
        }
        if r.from_prop.target_kind() != r.to_prop.target_kind() {
            return Err(format!("cannot remap {} onto {}: different kinds of target", prop_name(r.from_prop), prop_name(r.to_prop)));
        }
    }
    let mut w = Work::new(p);
    let mut out = Vec::new();
    let mut changed = 0;
    for ct in &clip.tracks {
        let (target, prop) = match remap.iter().find(|r| r.from_target == ct.target && r.from_prop == ct.prop) {
            Some(r) => (r.to_target.clone(), r.to_prop),
            None => (ct.target.clone(), ct.prop),
        };
        let mut keys: Vec<Key> = ct.keys.iter().map(|k| Key { frame: snapf(o, at + k.frame), ..*k }).collect();
        for k in &keys {
            check_frame(p, o, k.frame, "paste")?;
        }
        keys.sort_by(|a, b| a.frame.total_cmp(&b.frame));
        let loc = w.locate(&target, prop)?;
        let mut notes = Vec::new();
        let written = place(w.track_at(loc), keys, o.collision, &mut notes)?;
        w.notes.extend(notes);
        changed += written.len();
        out.extend(written.into_iter().map(|f| (loc, f)));
    }
    w.finish(out, changed)
}

/// Duplicate the selected keys `offset` frames later (or earlier) on the same channels.
pub fn duplicate_keys(p: &Project, sel: &[KeyRef], offset: f32, o: &EditOpts) -> Result<EditPlan, String> {
    finite(offset, "offset")?;
    let off = snapf(o, offset);
    if off == 0.0 {
        return Err("duplicate offset must not be zero".into());
    }
    let by = gather(p, sel)?;
    let mut w = Work::new(p);
    let mut out = Vec::new();
    let mut changed = 0;
    for (ch, frames) in &by {
        let t = w.existing(*ch)?;
        let copies: Vec<Key> = t.keys.iter().filter(|k| frames.contains(&k.frame)).map(|k| Key { frame: snapf(o, k.frame + off), ..*k }).collect();
        for k in &copies {
            check_frame(p, o, k.frame, "duplicate")?;
        }
        // copies may land on keys of the selection itself: those are not moved, so they collide normally
        let mut notes = Vec::new();
        let written = place(t, copies, o.collision, &mut notes)?;
        w.notes.extend(notes);
        changed += written.len();
        out.extend(written.into_iter().map(|f| (Loc::Existing(*ch), f)));
    }
    w.finish(out, changed)
}

/// Delete the selected keys; a channel left without keys is removed (as `Editor::delete_key` does).
pub fn delete_keys(p: &Project, sel: &[KeyRef]) -> Result<EditPlan, String> {
    let by = gather(p, sel)?;
    let mut w = Work::new(p);
    let mut changed = 0;
    for (ch, frames) in &by {
        let t = w.existing(*ch)?;
        changed += take_keys(t, frames).len();
    }
    w.finish(Vec::new(), changed)
}

/// Set the interpolation of the segment leaving each selected key.
pub fn set_ease(p: &Project, sel: &[KeyRef], ease: Ease) -> Result<EditPlan, String> {
    validate_ease(&ease)?;
    let by = gather(p, sel)?;
    let mut w = Work::new(p);
    let mut out = Vec::new();
    let mut changed = 0;
    for (ch, frames) in &by {
        let t = w.existing(*ch)?;
        for k in t.keys.iter_mut().filter(|k| frames.contains(&k.frame)) {
            k.ease = ease;
            changed += 1;
        }
        out.extend(frames.iter().map(|&f| (Loc::Existing(*ch), f)));
    }
    w.finish(out, changed)
}

/// Give every selected key the same value.
pub fn set_value(p: &Project, sel: &[KeyRef], value: f32) -> Result<EditPlan, String> {
    finite(value, "value")?;
    let by = gather(p, sel)?;
    let mut w = Work::new(p);
    let mut out = Vec::new();
    let mut changed = 0;
    for (ch, frames) in &by {
        let t = w.existing(*ch)?;
        for k in t.keys.iter_mut().filter(|k| frames.contains(&k.frame)) {
            k.value = value;
            changed += 1;
        }
        out.extend(frames.iter().map(|&f| (Loc::Existing(*ch), f)));
    }
    w.finish(out, changed)
}

/// Add one key (value defaults to the channel's current curve value at that frame).
pub fn add_key(p: &Project, target: &str, prop: Prop, frame: f32, value: Option<f32>, ease: Ease, o: &EditOpts) -> Result<EditPlan, String> {
    validate_ease(&ease)?;
    let frame = snapf(o, frame);
    check_frame(p, o, frame, "add key")?;
    let mut w = Work::new(p);
    let loc = w.locate(target, prop)?;
    let v = match value {
        Some(v) => v,
        None => w.track_at(loc).eval(frame).ok_or_else(|| "a value is required for a channel that has no keys yet".to_string())?,
    };
    finite(v, "value")?;
    w.track_at(loc).set_key(frame, v, ease);
    w.finish(vec![(loc, frame)], 1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetimeMode {
    /// Open a gap of `count` frames at `at`; later keys move right.
    Insert,
    /// Remove frames `at .. at+count`: keys inside are deleted, later keys move left.
    Delete,
}

/// Ripple edit. `scope` lists channels; `None` means every unlocked keyed channel (locked ones are
/// skipped and reported in the notes). With `adjust_frames` the project length changes with it.
pub fn ripple(p: &Project, at: f32, count: f32, mode: RetimeMode, scope: Option<&[usize]>, adjust_frames: bool, o: &EditOpts) -> Result<EditPlan, String> {
    finite(at, "at")?;
    finite(count, "count")?;
    let (at, count) = (snapf(o, at), snapf(o, count));
    if count <= 0.0 {
        return Err("count must be positive".into());
    }
    let mut w = Work::new(p);
    let chans: Vec<usize> = match scope {
        Some(l) => l.to_vec(),
        None => {
            let mut v = Vec::new();
            for (i, c) in p.channels.iter().enumerate() {
                if !is_keyed(c) {
                    continue;
                }
                if chanmeta::is_locked(p, &c.target, c.prop) {
                    w.notes.push(format!("skipped locked channel {} {}", c.target, prop_name(c.prop)));
                } else {
                    v.push(i);
                }
            }
            v
        }
    };
    let new_frames = if adjust_frames {
        let nf = match mode {
            RetimeMode::Insert => p.frames as f32 + count,
            RetimeMode::Delete => p.frames as f32 - count,
        };
        if nf < 1.0 {
            return Err("the project would have no frames left".into());
        }
        w.extra.push(Op::Set { path: "/frames".into(), value: serde_json::json!(nf as u32) });
        nf as u32
    } else {
        p.frames
    };
    let po = Project { frames: new_frames, ..shallow(p) };
    let mut changed = 0;
    for ch in chans {
        let t = w.existing(ch)?;
        match mode {
            RetimeMode::Insert => {
                for k in t.keys.iter_mut().filter(|k| k.frame >= at) {
                    k.frame += count;
                    check_frame(&po, o, k.frame, "ripple insert")?;
                    changed += 1;
                }
            }
            RetimeMode::Delete => {
                let before = t.keys.len();
                t.keys.retain(|k| !(k.frame >= at && k.frame < at + count));
                changed += before - t.keys.len();
                for k in t.keys.iter_mut().filter(|k| k.frame >= at + count) {
                    k.frame -= count;
                    changed += 1;
                }
            }
        }
    }
    w.finish(Vec::new(), changed)
}
