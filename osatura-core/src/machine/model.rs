// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `machine` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// Data model (serialised in Project.machines)
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    Bool,
    Number,
    Trigger,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub name: String,
    pub kind: InputKind,
    /// Initial value (bool: 0/1).
    #[serde(default)]
    pub default: f64,
    /// Triggers only: stay set until consumed instead of lasting one `step`.
    #[serde(default)]
    pub hold: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cmp {
    #[default]
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Cmp {
    pub(super) fn test(self, a: f64, b: f64) -> bool {
        match self {
            Cmp::Eq => a == b,
            Cmp::Ne => a != b,
            Cmp::Lt => a < b,
            Cmp::Le => a <= b,
            Cmp::Gt => a > b,
            Cmp::Ge => a >= b,
        }
    }
}

pub(super) fn one64() -> f64 {
    1.0
}

pub(super) fn one32() -> f32 {
    1.0
}

/// `input op value`. A bool is 0/1 and a trigger is 1 while set, so `{input: "jump"}` alone
/// (op `eq`, value 1) means "the trigger is set" / "the flag is true".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub input: String,
    #[serde(default)]
    pub op: Cmp,
    #[serde(default = "one64")]
    pub value: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayMode {
    #[default]
    Loop,
    Once,
    PingPong,
}

/// What a state plays.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StateSource {
    /// A clip of `Project.clips` (its embedded take, masks, rename and weights).
    Clip { clip: String },
    /// Frames `start..end` of the project's keyed replace-mode channels.
    Segment { start: f32, end: f32 },
    /// A region marker (`Project.markers` entry with an `end`).
    Region { marker: String },
    /// 1-D blend tree: children (leaf sources) mixed by a number input.
    Blend { input: String, children: Vec<BlendChild> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlendChild {
    /// Input value at which this child has full weight.
    pub threshold: f32,
    pub source: StateSource,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub name: String,
    pub source: StateSource,
    #[serde(default)]
    pub mode: PlayMode,
    #[serde(default = "one32")]
    pub speed: f32,
    /// Default crossfade (seconds) into this state when a transition gives no `duration`.
    #[serde(default)]
    pub blend_in: f32,
}

/// Unknown keys are an error here (not just a warning): a typo such as `conditions` for `when` would otherwise
/// silently turn a guarded transition into one that always fires.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    /// State name, or `"*"` for any state (never matches a transition into the current state).
    pub from: String,
    pub to: String,
    #[serde(default, rename = "when", skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
    /// Only once the state has played this fraction (0..1) of its length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_time: Option<f32>,
    /// Crossfade seconds; defaults to the target state's `blend_in`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
    #[serde(default)]
    pub ease: Ease,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    /// Glob patterns of targets this layer drives (empty = all).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mask: Vec<String>,
    /// Name of the state entered at start.
    pub entry: String,
    pub states: Vec<State>,
    #[serde(default)]
    pub transitions: Vec<Transition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Machine {
    pub name: String,
    #[serde(default)]
    pub inputs: Vec<Input>,
    pub layers: Vec<Layer>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Entered,
    Exited,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineEvent {
    /// Runtime time in seconds.
    pub time: f64,
    pub layer: String,
    pub kind: EventKind,
    pub state: String,
}

pub const MAX_STEP: f64 = 10.0;

pub(super) const SUBSTEP: f64 = 1.0 / 60.0;

pub(super) const MAX_HOPS: usize = 8;

// ---------------------------------------------------------------------------------------------
// Resolving sources
// ---------------------------------------------------------------------------------------------

pub(super) type TrackSet = Vec<(String, Prop, Track, f32)>;

pub(super) struct Leaf {
    /// Length in frames.
    pub(super) len: f32,
    /// Added to the local frame when sampling (segments are absolute).
    pub(super) offset: f32,
    pub(super) tracks: TrackSet,
}

pub(super) fn mask_ok(mask: &[String], target: &str) -> bool {
    mask.is_empty() || mask.iter().any(|g| glob(g, target))
}

pub(super) fn resolve_leaf(p: &Project, src: &StateSource, mask: &[String]) -> Result<Leaf, String> {
    match src {
        StateSource::Clip { clip } => {
            let c = p.clips.iter().find(|c| &c.name == clip).ok_or_else(|| format!("unknown clip '{clip}'"))?;
            if !(c.take.length.is_finite() && c.take.length > 0.0) {
                return Err(format!("clip '{clip}' has no positive length"));
            }
            let mut tracks = Vec::new();
            for tt in &c.take.tracks {
                let target = c.rename.iter().find(|(f, _)| f == &tt.target).map_or(tt.target.as_str(), |(_, t)| t.as_str()).to_string();
                if tt.keys.is_empty() || !c.passes_mask(&target) || !mask_ok(mask, &target) {
                    continue;
                }
                let mut w = c.weight;
                if let Some(r) = c.track_weights.iter().find(|r| glob(&r.pattern, &target) && r.prop.is_none_or(|pp| pp == tt.prop)) {
                    w *= r.weight;
                }
                let t = Track::with_keys(&tt.keys.iter().map(|k| (k.frame, k.value, k.ease)).collect::<Vec<_>>());
                tracks.push((target, tt.prop, t, w));
            }
            Ok(Leaf { len: c.take.length, offset: 0.0, tracks })
        }
        StateSource::Segment { start, end } => {
            if !(start.is_finite() && end.is_finite() && *start >= 0.0 && end > start) {
                return Err(format!("segment {start}..{end} needs 0 <= start < end"));
            }
            if *end > p.frames as f32 {
                return Err(format!("segment {start}..{end} lies beyond the {} project frames", p.frames));
            }
            Ok(Leaf { len: end - start, offset: *start, tracks: segment_tracks(p, mask) })
        }
        StateSource::Region { marker } => {
            let m = p.markers.iter().find(|m| &m.name == marker).ok_or_else(|| format!("unknown marker '{marker}'"))?;
            let end = m.end.ok_or_else(|| format!("marker '{marker}' is not a region (no end)"))?;
            resolve_leaf(p, &StateSource::Segment { start: m.frame, end }, mask)
        }
        StateSource::Blend { .. } => Err("a blend tree cannot be nested".into()),
    }
}

/// Keyed replace-mode channels of the project (later channels override earlier ones).
pub(super) fn segment_tracks(p: &Project, mask: &[String]) -> TrackSet {
    let mut out: TrackSet = Vec::new();
    for c in &p.channels {
        if let (Source::Keys { track }, Mode::Replace) = (&c.source, c.mode) {
            if track.keys.is_empty() || !mask_ok(mask, &c.target) {
                continue;
            }
            let item = (c.target.clone(), c.prop, track.clone(), c.weight.clamp(0.0, 1.0));
            match out.iter().position(|o| o.0 == c.target && o.1 == c.prop) {
                Some(i) => out[i] = item,
                None => out.push(item),
            }
        }
    }
    out
}
