// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `timeline` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------- model

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct KeyInfo {
    pub frame: f32,
    pub value: f32,
    pub ease: Ease,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TrackInfo {
    /// Channel index in `Project::channels` (equals the position in `TimelineModel::tracks`).
    pub ch: usize,
    pub target: String,
    pub prop: Prop,
    pub label: String,
    /// `keys`, `expr`, `noise` or `driven`.
    pub source: &'static str,
    /// Key channels that are not skin-collapse helpers; only these accept key edits.
    pub editable: bool,
    pub locked: bool,
    pub shy: bool,
    pub solo: bool,
    pub color: Option<String>,
    pub mode: Mode,
    pub weight: f32,
    pub extrapolate: Option<Extrapolate>,
    /// Index into `TimelineModel::groups`.
    pub group: usize,
    pub keys: Vec<KeyInfo>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GroupInfo {
    pub id: usize,
    pub label: String,
    /// `bone`, `layer`, `camera`, `ik`, `constraint` or `sub` (the kind headers are depth 0).
    pub kind: &'static str,
    pub depth: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    /// Channels that hang directly off this group.
    pub tracks: Vec<usize>,
    /// Sorted union of the key frames of this group and everything below it (summary diamonds).
    pub frames: Vec<f32>,
    /// Every editable track below this group is locked (and there is at least one).
    pub locked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "row", content = "index", rename_all = "snake_case")]
pub enum Row {
    Group(usize),
    Track(usize),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimelineModel {
    pub frames: u32,
    pub fps: f32,
    pub groups: Vec<GroupInfo>,
    pub tracks: Vec<TrackInfo>,
    /// Display order, depth first: a group row, its own tracks, then its child groups.
    pub rows: Vec<Row>,
}

pub(super) fn kind_name(k: TargetKind) -> &'static str {
    match k {
        TargetKind::Bone => "bone",
        TargetKind::Layer => "layer",
        TargetKind::Camera => "camera",
        TargetKind::Ik => "ik",
        TargetKind::Constraint => "constraint",
        TargetKind::Sub => "sub",
    }
}

pub(super) fn kind_label(k: TargetKind) -> &'static str {
    match k {
        TargetKind::Bone => "Bones",
        TargetKind::Layer => "Layers",
        TargetKind::Camera => "Camera",
        TargetKind::Ik => "IK chains",
        TargetKind::Constraint => "Constraints",
        TargetKind::Sub => "Deformers and paths",
    }
}

pub(super) const KIND_ORDER: [TargetKind; 6] = [TargetKind::Bone, TargetKind::Layer, TargetKind::Camera, TargetKind::Ik, TargetKind::Constraint, TargetKind::Sub];

pub(super) fn prop_name(p: Prop) -> String {
    serde_json::to_value(p).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_else(|| format!("{p:?}"))
}

pub(super) fn is_keyed(c: &Channel) -> bool {
    matches!(c.source, Source::Keys { .. }) && !crate::skins::is_collapse_channel(c)
}

/// (group label, group path) chain from the kind header down to the group that owns the channel.
pub(super) fn chain_of(p: &Project, c: &Channel, kind: TargetKind) -> Vec<(String, String)> {
    match kind {
        TargetKind::Bone => {
            let mut chain = vec![c.target.clone()];
            let mut cur = p.bones.iter().find(|b| b.name == c.target);
            let mut guard = 0;
            while let Some(parent) = cur.and_then(|b| b.parent.as_ref()) {
                chain.push(parent.clone());
                cur = p.bones.iter().find(|b| &b.name == parent);
                guard += 1;
                if guard > p.bones.len() {
                    break;
                }
            }
            chain.reverse();
            chain.into_iter().map(|n| (n.clone(), n)).collect()
        }
        TargetKind::Layer => {
            let mut out = Vec::new();
            let mut acc = String::new();
            for seg in c.target.split('/') {
                if !acc.is_empty() {
                    acc.push('/');
                }
                acc.push_str(seg);
                out.push((seg.to_string(), acc.clone()));
            }
            out
        }
        TargetKind::Camera => Vec::new(),
        TargetKind::Ik | TargetKind::Constraint => vec![(c.target.clone(), c.target.clone())],
        TargetKind::Sub => {
            let parts: Vec<&str> = c.target.split("::").collect();
            let upto = if parts.len() >= 2 { parts.len() - 1 } else { parts.len() };
            let mut out = Vec::new();
            for i in 0..upto {
                out.push((parts[i].to_string(), parts[..=i].join("::")));
            }
            out
        }
    }
}

pub(super) fn meta_for(p: &Project, target: &str, prop: Prop) -> (bool, bool, bool, Option<String>) {
    let (mut lock, mut shy, mut solo) = (false, false, false);
    let mut color: Option<(bool, String)> = None; // (specific to the prop, colour)
    for m in &p.channel_meta {
        if m.target != target || m.prop.is_some_and(|q| q != prop) {
            continue;
        }
        lock |= m.lock;
        shy |= m.shy;
        solo |= m.solo;
        if let Some(c) = &m.color {
            let specific = m.prop.is_some();
            if color.as_ref().is_none_or(|(s, _)| specific || !*s) {
                color = Some((specific, c.clone()));
            }
        }
    }
    (lock || chanmeta::layer_locked(p, target, prop), shy, solo, color.map(|c| c.1))
}

/// Build the timeline model. Deterministic: order follows the kind order, then channel order.
pub fn build(p: &Project) -> TimelineModel {
    let mut groups: Vec<GroupInfo> = Vec::new();
    let mut tracks: Vec<TrackInfo> = Vec::new();
    let mut index: BTreeMap<(usize, String), usize> = BTreeMap::new();
    let mut root_of: BTreeMap<usize, usize> = BTreeMap::new();
    // channel -> group, filled kind by kind but stored in channel order afterwards
    let mut owner: Vec<usize> = vec![0; p.channels.len()];
    for (ki, kind) in KIND_ORDER.iter().enumerate() {
        for (ci, c) in p.channels.iter().enumerate() {
            if c.prop.target_kind() != *kind {
                continue;
            }
            let root = *root_of.entry(ki).or_insert_with(|| {
                groups.push(GroupInfo { id: groups.len(), label: kind_label(*kind).into(), kind: kind_name(*kind), depth: 0, parent: None, children: vec![], tracks: vec![], frames: vec![], locked: false });
                groups.len() - 1
            });
            let mut parent = root;
            for (label, path) in chain_of(p, c, *kind) {
                let id = match index.get(&(ki, path.clone())) {
                    Some(&i) => i,
                    None => {
                        let id = groups.len();
                        groups.push(GroupInfo { id, label, kind: kind_name(*kind), depth: groups[parent].depth + 1, parent: Some(parent), children: vec![], tracks: vec![], frames: vec![], locked: false });
                        groups[parent].children.push(id);
                        index.insert((ki, path), id);
                        id
                    }
                };
                parent = id;
            }
            groups[parent].tracks.push(ci);
            owner[ci] = parent;
        }
    }
    for (ci, c) in p.channels.iter().enumerate() {
        let (lock, shy, solo, color) = meta_for(p, &c.target, c.prop);
        let (source, keys, extrap) = match &c.source {
            Source::Keys { track } => ("keys", track.keys.iter().map(|k| KeyInfo { frame: k.frame, value: k.value, ease: k.ease }).collect(), Some(track.extrapolate)),
            Source::Expr { .. } => ("expr", vec![], None),
            Source::Noise { .. } => ("noise", vec![], None),
            Source::Driven { .. } => ("driven", vec![], None),
        };
        let mut label = prop_name(c.prop);
        if c.prop.target_kind() == TargetKind::Sub {
            if let Some(last) = c.target.rsplit("::").next().filter(|_| c.target.contains("::")) {
                label = format!("{last} {label}");
            }
        }
        tracks.push(TrackInfo {
            ch: ci,
            target: c.target.clone(),
            prop: c.prop,
            label,
            source,
            editable: is_keyed(c),
            locked: lock,
            shy,
            solo,
            color,
            mode: c.mode,
            weight: c.weight,
            extrapolate: extrap,
            group: owner[ci],
            keys,
        });
    }
    // summaries bottom-up (children always have larger ids than their parents)
    for gi in (0..groups.len()).rev() {
        let mut fr: Vec<f32> = groups[gi].tracks.iter().flat_map(|&t| tracks[t].keys.iter().map(|k| k.frame)).collect();
        for &c in &groups[gi].children.clone() {
            fr.extend(groups[c].frames.iter().copied());
        }
        fr.sort_by(|a, b| a.total_cmp(b));
        fr.dedup();
        groups[gi].frames = fr;
        let mut editable = 0;
        let mut locked = 0;
        let mut stack = vec![gi];
        while let Some(x) = stack.pop() {
            for &t in &groups[x].tracks {
                if tracks[t].editable {
                    editable += 1;
                    if tracks[t].locked {
                        locked += 1;
                    }
                }
            }
            stack.extend(groups[x].children.iter().copied());
        }
        groups[gi].locked = editable > 0 && editable == locked;
    }
    let mut rows = Vec::new();
    fn walk(g: usize, groups: &[GroupInfo], rows: &mut Vec<Row>) {
        rows.push(Row::Group(g));
        for &t in &groups[g].tracks {
            rows.push(Row::Track(t));
        }
        for &c in &groups[g].children {
            walk(c, groups, rows);
        }
    }
    for g in groups.iter().filter(|g| g.parent.is_none()) {
        walk(g.id, &groups, &mut rows);
    }
    TimelineModel { frames: p.frames, fps: p.fps, groups, tracks, rows }
}

// ---------------------------------------------------------------- selection

/// One key: channel index plus its exact frame.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyRef {
    pub ch: usize,
    pub frame: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelMode {
    #[default]
    Replace,
    Add,
    Toggle,
    Subtract,
}

impl SelMode {
    pub fn parse(s: &str) -> Option<SelMode> {
        serde_json::from_value(serde_json::Value::String(s.into())).ok()
    }
}

/// Sort by (channel, frame) and drop duplicates.
pub fn normalize(sel: &mut Vec<KeyRef>) {
    sel.sort_by(|a, b| a.ch.cmp(&b.ch).then(a.frame.total_cmp(&b.frame)));
    sel.dedup_by(|b, a| a.ch == b.ch && a.frame == b.frame);
}

/// Combine the current selection with `hits` according to `mode`.
pub fn select_apply(current: &[KeyRef], hits: &[KeyRef], mode: SelMode) -> Vec<KeyRef> {
    let has = |v: &[KeyRef], k: &KeyRef| v.iter().any(|x| x.ch == k.ch && x.frame == k.frame);
    let mut out: Vec<KeyRef> = match mode {
        SelMode::Replace => hits.to_vec(),
        SelMode::Add => current.iter().chain(hits).copied().collect(),
        SelMode::Subtract => current.iter().filter(|k| !has(hits, k)).copied().collect(),
        SelMode::Toggle => current.iter().filter(|k| !has(hits, k)).chain(hits.iter().filter(|k| !has(current, k))).copied().collect(),
    };
    normalize(&mut out);
    out
}

pub(super) fn track_keys(m: &TimelineModel, t: usize, f0: f32, f1: f32, include_locked: bool, out: &mut Vec<KeyRef>) {
    let tr = &m.tracks[t];
    if !tr.editable || (tr.locked && !include_locked) {
        return;
    }
    out.extend(tr.keys.iter().filter(|k| k.frame >= f0 && k.frame <= f1).map(|k| KeyRef { ch: t, frame: k.frame }));
}

pub(super) fn descendants(m: &TimelineModel, g: usize, out: &mut Vec<usize>) {
    out.extend(m.groups[g].tracks.iter().copied());
    for &c in &m.groups[g].children {
        descendants(m, c, out);
    }
}

/// Keys inside `[f0, f1]` on the display rows `row0..=row1` (either order). A group row stands for
/// every track below it. Locked tracks are skipped unless `include_locked`.
pub fn select_box(m: &TimelineModel, f0: f32, f1: f32, row0: usize, row1: usize, include_locked: bool) -> Vec<KeyRef> {
    let (f0, f1) = (f0.min(f1), f0.max(f1));
    let (r0, r1) = (row0.min(row1), row0.max(row1));
    let mut chans: Vec<usize> = Vec::new();
    for r in m.rows.iter().skip(r0).take(r1.saturating_sub(r0) + 1) {
        match *r {
            Row::Track(t) => chans.push(t),
            Row::Group(g) => descendants(m, g, &mut chans),
        }
    }
    chans.sort_unstable();
    chans.dedup();
    let mut out = Vec::new();
    for t in chans {
        track_keys(m, t, f0, f1, include_locked, &mut out);
    }
    normalize(&mut out);
    out
}

/// Every key of one channel.
pub fn select_channel(m: &TimelineModel, ch: usize, include_locked: bool) -> Vec<KeyRef> {
    let mut out = Vec::new();
    if ch < m.tracks.len() {
        track_keys(m, ch, f32::NEG_INFINITY, f32::INFINITY, include_locked, &mut out);
    }
    out
}

/// Every key below a group.
pub fn select_group(m: &TimelineModel, group: usize, include_locked: bool) -> Vec<KeyRef> {
    let mut out = Vec::new();
    if group < m.groups.len() {
        let mut chans = Vec::new();
        descendants(m, group, &mut chans);
        for t in chans {
            track_keys(m, t, f32::NEG_INFINITY, f32::INFINITY, include_locked, &mut out);
        }
    }
    normalize(&mut out);
    out
}

/// Every key of every track.
pub fn select_all(m: &TimelineModel, include_locked: bool) -> Vec<KeyRef> {
    let mut out = Vec::new();
    for t in 0..m.tracks.len() {
        track_keys(m, t, f32::NEG_INFINITY, f32::INFINITY, include_locked, &mut out);
    }
    out
}

/// Drop references that no longer point at a key (after an edit elsewhere).
pub fn prune(m: &TimelineModel, sel: &[KeyRef]) -> Vec<KeyRef> {
    sel.iter().filter(|r| m.tracks.get(r.ch).is_some_and(|t| t.keys.iter().any(|k| k.frame == r.frame))).copied().collect()
}

// ---------------------------------------------------------------- options, plan, clip

/// What to do when an edit puts a key on a frame that already holds one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Collision {
    /// The incoming key overwrites the key already there.
    #[default]
    Replace,
    /// The key already there stays; the incoming key is dropped (frames are merged).
    Merge,
    /// Refuse the whole edit.
    Refuse,
}

impl Collision {
    pub fn parse(s: &str) -> Option<Collision> {
        serde_json::from_value(serde_json::Value::String(s.into())).ok()
    }
}

/// Allowed time range for edited keys.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bounds {
    /// `0 ..= project.frames` (the closing key of a loop sits on `frames`).
    #[default]
    Project,
    Free,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditOpts {
    /// Round moved / created key frames to whole frames.
    pub snap: bool,
    pub collision: Collision,
    pub bounds: Bounds,
}

impl Default for EditOpts {
    fn default() -> Self {
        EditOpts { snap: true, collision: Collision::Replace, bounds: Bounds::Project }
    }
}

/// Keys copied out of a project, frames relative to the earliest copied key.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub tracks: Vec<ClipTrack>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClipTrack {
    pub target: String,
    pub prop: Prop,
    pub keys: Vec<Key>,
}

impl Clip {
    /// Point every clip track that targets `from` at `to` instead (e.g. left arm -> right arm).
    pub fn retarget(&mut self, from: &str, to: &str) -> usize {
        let mut n = 0;
        for t in &mut self.tracks {
            if t.target == from {
                t.target = to.into();
                n += 1;
            }
        }
        n
    }
    pub fn key_count(&self) -> usize {
        self.tracks.iter().map(|t| t.keys.len()).sum()
    }
}

/// Destination override for [`TlOp::Paste`]: clip track `(from_target, from_prop)` lands on `(to_target, to_prop)`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Remap {
    pub from_target: String,
    pub from_prop: Prop,
    pub to_target: String,
    pub to_prop: Prop,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct EditPlan {
    /// The edit (a `Batch`); `None` when nothing changes (copy, zero move).
    pub op: Option<Op>,
    /// Where the edited keys are afterwards (indices valid after the op is applied).
    pub selection: Vec<KeyRef>,
    pub changed: usize,
    pub notes: Vec<String>,
    pub clip: Option<Clip>,
}

// ---------------------------------------------------------------- working copy

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Loc {
    Existing(usize),
    Fresh(usize),
}
