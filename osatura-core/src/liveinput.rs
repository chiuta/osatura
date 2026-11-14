// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Live input mapping: MIDI, mouse, gamepad and keyboard values drive parameters, and
//! buttons/keys fire trigger actions. Device independent and deterministic.
//!
//! * [`InputEvent`] — one normalised (0..1) value from some source at a time in seconds.
//!   Anything that can produce numbers can feed it: the studio page (Web MIDI, mouse, Gamepad
//!   API, keys), a `.mid` file ([`crate::midi::smf_to_events`]), or a JSON file.
//! * [`Mapping`] — a validated table of [`ParamMap`]s (source → `target.prop` with range,
//!   curve, dead zone, invert, smoothing) and [`TriggerMap`]s (threshold crossing → [`Action`]).
//! * [`bake`] — turns a recorded performance into ordinary keyed channels, events and takes
//!   (non-destructive, all-or-nothing, locks respected), exactly like the `drive` command does
//!   for per-frame CSV signals.
//! * [`LiveSession`] — the stateful side used by the studio: latest values, smoothed preview
//!   of the mapped parameters, edge detection, and a recording buffer.
//!
//! Clean-room: the mapping maths are the textbook range/curve/deadzone/one-pole-smoothing
//! chain already used by [`crate::perf::InputMap`], which is reused for the shaping.

use crate::anim::{Ease, Track};
use crate::comps::Event;
use crate::motionlib::Library;
use crate::perf::{Curve, InputMap};
use crate::project::{Channel, Mode, Project, Prop, Source};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    MidiCc,
    MidiNote,
    MidiPitchBend,
    MouseX,
    MouseY,
    MouseButton,
    GamepadAxis,
    GamepadButton,
    Key,
    /// Named face signal from [`crate::faceparam`] (ids such as `mouth_open`, `head_yaw`).
    Face,
}

/// One normalised input sample. `id` names the control within its source: MIDI `"<channel>:<number>"`
/// (channel 1..16), pitch bend `"<channel>"`, mouse button `"0"`/`"1"`/`"2"`, gamepad `"<pad>:<index>"`,
/// key = the DOM `code` such as `"KeyA"`. `time` is seconds since the performance started.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputEvent {
    pub source: InputSource,
    pub id: String,
    pub value: f32,
    pub time: f32,
}

/// Raw MIDI bytes captured with a time (seconds); decoded by [`crate::midi::decode`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawMidi {
    pub time: f32,
    pub bytes: Vec<u8>,
}

/// A recording file / studio push body: ready events plus raw MIDI bytes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Recording {
    #[serde(default)]
    pub events: Vec<InputEvent>,
    #[serde(default)]
    pub midi: Vec<RawMidi>,
}

impl Recording {
    /// All events (raw MIDI decoded; unmapped MIDI messages dropped), sorted by time.
    /// Returns warnings for undecodable MIDI instead of failing the whole recording.
    pub fn into_events(self) -> Result<(Vec<InputEvent>, Vec<String>), String> {
        let mut warns = Vec::new();
        let mut all = self.events;
        for m in &self.midi {
            if !m.time.is_finite() {
                return Err("MIDI time must be finite".into());
            }
            match crate::midi::decode(&m.bytes) {
                Ok(msg) => all.extend(crate::midi::to_event(&msg, m.time)),
                Err(e) => warns.push(format!("MIDI bytes skipped: {e}")),
            }
        }
        Ok((clean_events(&all)?, warns))
    }
}

pub const MAX_EVENTS: usize = 3_000_000;
const MAX_TIME: f32 = 7.0 * 86400.0;

/// Validate and normalise events: finite, time within 0..7 days, id 1..64 chars; values are
/// clamped to 0..1. Result is stably sorted by time.
pub fn clean_events(events: &[InputEvent]) -> Result<Vec<InputEvent>, String> {
    if events.len() > MAX_EVENTS {
        return Err(format!("more than {MAX_EVENTS} events"));
    }
    let mut v = Vec::with_capacity(events.len());
    for (i, e) in events.iter().enumerate() {
        if !e.time.is_finite() || e.time < 0.0 || e.time > MAX_TIME {
            return Err(format!(
                "event {i}: time must be a finite number of seconds within 0..{MAX_TIME}"
            ));
        }
        if !e.value.is_finite() {
            return Err(format!("event {i}: value must be finite"));
        }
        if e.id.is_empty() || e.id.len() > 64 {
            return Err(format!("event {i}: id must be 1..64 characters"));
        }
        v.push(InputEvent {
            value: e.value.clamp(0.0, 1.0),
            ..e.clone()
        });
    }
    v.sort_by(|a, b| a.time.total_cmp(&b.time));
    Ok(v)
}

/// `pattern` matches an event id when equal, `"*"`, or equal to the part after the first `:`
/// (so `"7"` matches MIDI controller 7 on any channel and `"1"` gamepad axis 1 on any pad).
pub fn id_matches(pattern: &str, id: &str) -> bool {
    pattern == "*" || pattern == id || id.split_once(':').is_some_and(|(_, rest)| rest == pattern)
}

fn one() -> f32 {
    1.0
}
fn half() -> f32 {
    0.5
}
fn lin() -> Curve {
    Curve::Linear
}

/// Continuous control → parameter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParamMap {
    pub source: InputSource,
    pub id: String,
    pub target: String,
    pub prop: Prop,
    #[serde(default)]
    pub in_min: f32,
    #[serde(default = "one")]
    pub in_max: f32,
    #[serde(default)]
    pub out_min: f32,
    #[serde(default = "one")]
    pub out_max: f32,
    /// Fraction of the input range (from `in_min`) treated as zero, 0..1.
    #[serde(default)]
    pub deadzone: f32,
    #[serde(default)]
    pub invert: bool,
    #[serde(default = "lin")]
    pub curve: Curve,
    /// One-pole smoothing per frame (bake) / per preview step, 0 = raw, towards 1 = heavier.
    #[serde(default)]
    pub smooth: f32,
    /// Add on top of existing animation instead of replacing it.
    #[serde(default)]
    pub additive: bool,
}

impl ParamMap {
    pub fn new(source: InputSource, id: &str, target: &str, prop: Prop) -> Self {
        ParamMap {
            source,
            id: id.into(),
            target: target.into(),
            prop,
            in_min: 0.0,
            in_max: 1.0,
            out_min: 0.0,
            out_max: 1.0,
            deadzone: 0.0,
            invert: false,
            curve: Curve::Linear,
            smooth: 0.0,
            additive: false,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let at = format!("{}.{:?}", self.target, self.prop);
        if self.id.is_empty() || self.target.is_empty() {
            return Err("a parameter map needs an id and a target".into());
        }
        if ![
            self.in_min,
            self.in_max,
            self.out_min,
            self.out_max,
            self.deadzone,
            self.smooth,
        ]
        .iter()
        .all(|v| v.is_finite())
        {
            return Err(format!("{at}: numbers must be finite"));
        }
        if self.in_min == self.in_max {
            return Err(format!("{at}: in_min and in_max must differ"));
        }
        if !(0.0..1.0).contains(&self.deadzone) || !(0.0..1.0).contains(&self.smooth) {
            return Err(format!(
                "{at}: deadzone and smooth must be within 0..1 (1 excluded)"
            ));
        }
        if let Curve::Pow { exp } = self.curve {
            if !exp.is_finite() || exp <= 0.0 || exp > 16.0 {
                return Err(format!("{at}: curve exponent must be within (0, 16]"));
            }
        }
        Ok(())
    }

    /// Range, dead zone, invert and curve, without smoothing.
    pub fn shape(&self, x: f32) -> f32 {
        let m = InputMap {
            input: String::new(),
            target: String::new(),
            prop: self.prop,
            in_min: self.in_min,
            in_max: self.in_max,
            out_min: self.out_min,
            out_max: self.out_max,
            deadzone: self.deadzone,
            invert: self.invert,
            curve: self.curve,
            smooth: 0.0,
        };
        m.apply(&[x])[0]
    }

    fn smoothed(&self, prev: Option<f32>, y: f32) -> f32 {
        match prev {
            Some(p) => p * self.smooth + y * (1.0 - self.smooth),
            None => y,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Edge {
    /// Value rises through the threshold (button down).
    #[default]
    Press,
    /// Value falls below the threshold (button up).
    Release,
    Both,
}

/// What a trigger does when it fires.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Live: moves the studio playhead. Baked: recorded as an event named `jump` with `data.to`
    /// (the time axis of a recording is not rewritten).
    JumpToFrame { frame: f32 },
    /// Paste a take from the library at the trigger's frame.
    PlayTake {
        take: String,
        #[serde(default = "one")]
        speed: f32,
    },
    /// Set a key on `target.prop` at the trigger's frame.
    SetKey {
        target: String,
        prop: Prop,
        value: f32,
        #[serde(default)]
        ease: Ease,
    },
    /// Add a project event at the trigger's frame.
    FireEvent {
        name: String,
        #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
        data: serde_json::Value,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerMap {
    pub source: InputSource,
    pub id: String,
    #[serde(default)]
    pub edge: Edge,
    #[serde(default = "half")]
    pub threshold: f32,
    pub action: Action,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    #[serde(default)]
    pub params: Vec<ParamMap>,
    #[serde(default)]
    pub triggers: Vec<TriggerMap>,
}

impl Mapping {
    pub fn from_json(s: &str) -> Result<Mapping, String> {
        serde_json::from_str(s).map_err(|e| format!("bad mapping: {e}"))
    }

    /// Check the table against a project (and a take library): numbers, locks, frames, takes.
    /// Target/property compatibility is checked when the baked project is compiled.
    pub fn validate(&self, p: &Project, lib: Option<&Library>) -> Result<(), String> {
        if self.params.len() + self.triggers.len() > 4096 {
            return Err("more than 4096 mappings".into());
        }
        for m in &self.params {
            m.validate()?;
            crate::chanmeta::assert_unlocked(p, &m.target, m.prop)?;
        }
        for (i, t) in self.triggers.iter().enumerate() {
            let at = format!("trigger {i}");
            if t.id.is_empty() {
                return Err(format!("{at}: id must not be empty"));
            }
            if !t.threshold.is_finite() || !(0.0..=1.0).contains(&t.threshold) {
                return Err(format!("{at}: threshold must be within 0..1"));
            }
            match &t.action {
                Action::JumpToFrame { frame } => {
                    if !frame.is_finite()
                        || *frame < 0.0
                        || *frame > p.frames.saturating_sub(1) as f32
                    {
                        return Err(format!(
                            "{at}: jump frame must be within 0..{}",
                            p.frames.saturating_sub(1)
                        ));
                    }
                }
                Action::PlayTake { take, speed } => {
                    if ((*speed).partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater))
                        || !speed.is_finite()
                    {
                        return Err(format!("{at}: take speed must be positive"));
                    }
                    let l = lib.ok_or_else(|| format!("{at}: play_take needs a take library"))?;
                    let tk = l
                        .find(take)
                        .ok_or_else(|| format!("{at}: no take named '{take}'"))?;
                    for tr in &tk.tracks {
                        crate::chanmeta::assert_unlocked(p, &tr.target, tr.prop)?;
                    }
                }
                Action::SetKey {
                    target,
                    prop,
                    value,
                    ..
                } => {
                    if !value.is_finite() || target.is_empty() {
                        return Err(format!("{at}: set_key needs a target and a finite value"));
                    }
                    crate::chanmeta::assert_unlocked(p, target, *prop)?;
                }
                Action::FireEvent { name, .. } => {
                    if name.trim().is_empty() {
                        return Err(format!("{at}: event name must not be empty"));
                    }
                }
            }
        }
        Ok(())
    }
}

/// A trigger that fired, at a time.
#[derive(Clone, Debug, PartialEq)]
pub struct Fired {
    pub time: f32,
    pub trigger: usize,
    pub action: Action,
}

/// Threshold-crossing detector state, one flag per (trigger, event id).
#[derive(Default)]
struct Edges(HashMap<(usize, String), bool>);

impl Edges {
    fn feed(&mut self, m: &Mapping, e: &InputEvent, out: &mut Vec<Fired>) {
        for (i, t) in m.triggers.iter().enumerate() {
            if t.source != e.source || !id_matches(&t.id, &e.id) {
                continue;
            }
            let now = e.value >= t.threshold && e.value > 0.0;
            let was = self.0.insert((i, e.id.clone()), now).unwrap_or(false);
            let fire = match t.edge {
                Edge::Press => now && !was,
                Edge::Release => !now && was,
                Edge::Both => now != was,
            };
            if fire {
                out.push(Fired {
                    time: e.time,
                    trigger: i,
                    action: t.action.clone(),
                });
            }
        }
    }
}

/// Triggers fired by a time-sorted event list.
pub fn fired_triggers(m: &Mapping, events: &[InputEvent]) -> Vec<Fired> {
    let (mut edges, mut out) = (Edges::default(), Vec::new());
    for e in events {
        edges.feed(m, e, &mut out);
    }
    out
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakeReport {
    pub channels: usize,
    /// Keys in the parameter channels that were created (`channels`): what the continuous mappings produced.
    pub param_keys: usize,
    /// Keys set by `set_key` triggers on top of that (a trigger key, not a parameter key).
    pub keys_set: usize,
    /// Frames of the recording that lie after the last project frame and were therefore not baked (0 = it fits).
    pub cut_frames: usize,
    pub takes: usize,
    pub events: usize,
    /// Triggers and events falling after the last project frame.
    pub skipped: usize,
    pub notes: Vec<String>,
}

fn frame_of(t: f32, fps: f32) -> f32 {
    (t * fps).round()
}

/// Bake a recorded performance into the project. One keyed channel per [`ParamMap`] that has
/// matching events (sample-and-hold per frame, then shaped, smoothed and reduced to the
/// fewest keys within `tol`); triggers become keys / takes / events. All-or-nothing: on any
/// error the project is left untouched. The result is compiled, so bad targets or properties are
/// reported here.
pub fn bake(
    p: &mut Project,
    events: &[InputEvent],
    m: &Mapping,
    lib: Option<&Library>,
    tol: f32,
) -> Result<BakeReport, String> {
    if !matches!(
        tol.partial_cmp(&0.0),
        Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
    ) || !tol.is_finite()
    {
        return Err("tolerance must be a non-negative number".into());
    }
    m.validate(p, lib)?;
    let ev = clean_events(events)?;
    let mut q = p.clone();
    let mut rep = BakeReport::default();
    let nf = q.frames.max(1) as usize;

    for (mi, pm) in m.params.iter().enumerate() {
        let mine: Vec<&InputEvent> = ev
            .iter()
            .filter(|e| e.source == pm.source && id_matches(&pm.id, &e.id))
            .collect();
        if mine.is_empty() {
            rep.notes.push(format!(
                "map {mi} ({}.{:?}): no matching events, nothing keyed",
                pm.target, pm.prop
            ));
            continue;
        }
        // sample and hold at every frame; before the first event the first value is held
        let (mut j, mut cur, mut prev) = (0usize, mine[0].value, None::<f32>);
        let mut samples = Vec::with_capacity(nf);
        for f in 0..nf {
            let tf = f as f32 / q.fps;
            while j < mine.len() && mine[j].time <= tf + 1e-6 {
                cur = mine[j].value;
                j += 1;
            }
            let y = pm.smoothed(prev, pm.shape(cur));
            prev = Some(y);
            samples.push((f as f32, y));
        }
        let last_t = mine.last().map(|e| e.time).unwrap_or(0.0);
        let beyond = (last_t * q.fps - (nf as f32 - 1.0)).ceil();
        if beyond > 0.0 {
            rep.cut_frames = rep.cut_frames.max(beyond as usize);
        }
        let mut ch = Channel::keys(
            &pm.target,
            pm.prop,
            crate::perf::simplify_samples(&samples, tol),
        );
        rep.param_keys += match &ch.source {
            Source::Keys { track } => track.keys.len(),
            _ => 0,
        };
        if pm.additive {
            ch.mode = Mode::Add;
        }
        q.channels.push(ch);
        rep.channels += 1;
    }

    let last = nf as f32 - 1.0;
    for fr in fired_triggers(m, &ev) {
        let frame = frame_of(fr.time, q.fps);
        if frame > last {
            rep.skipped += 1;
            continue;
        }
        match fr.action {
            Action::JumpToFrame { frame: to } => {
                q.events.push(Event {
                    name: "jump".into(),
                    frame,
                    data: serde_json::json!({ "to": to }),
                });
                rep.events += 1;
            }
            Action::FireEvent { name, data } => {
                q.events.push(Event { name, frame, data });
                rep.events += 1;
            }
            Action::SetKey {
                target,
                prop,
                value,
                ease,
            } => {
                match q.channels.iter_mut().find(|c| {
                    c.target == target
                        && c.prop == prop
                        && c.mode == Mode::Replace
                        && matches!(c.source, Source::Keys { .. })
                }) {
                    Some(c) => {
                        if let Source::Keys { track } = &mut c.source {
                            track.set_key(frame, value, ease);
                        }
                    }
                    None => q.channels.push(Channel::keys(
                        &target,
                        prop,
                        Track::with_keys(&[(frame, value, ease)]),
                    )),
                }
                rep.keys_set += 1;
            }
            Action::PlayTake { take, speed } => {
                let l = lib.ok_or("play_take needs a take library")?;
                let tk = l
                    .find(&take)
                    .ok_or_else(|| format!("no take named '{take}'"))?;
                crate::motionlib::apply(&mut q, tk, frame, &[], speed)?;
                rep.takes += 1;
            }
        }
    }
    q.events.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    crate::engine::Engine::new(q.clone())
        .map_err(|e| format!("the mapped project is not valid: {}", e.join("; ")))?;
    *p = q;
    Ok(rep)
}

/// Parsed recordings for the CLI: a JSON array of events, or a [`Recording`] object.
pub fn parse_recording_json(text: &str) -> Result<(Vec<InputEvent>, Vec<String>), String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("bad JSON: {e}"))?;
    let rec = if v.is_array() {
        Recording {
            events: serde_json::from_value(v).map_err(|e| format!("bad events: {e}"))?,
            midi: vec![],
        }
    } else {
        serde_json::from_value(v).map_err(|e| format!("bad recording: {e}"))?
    };
    rec.into_events()
}

// ---- live session (studio) ------------------------------------------------------------------

/// Stateful live side: remembers the newest value of every control, previews the mapped
/// parameter values (with smoothing), detects trigger edges and optionally records.
#[derive(Default)]
pub struct LiveSession {
    mapping: Mapping,
    library: Option<Library>,
    latest: HashMap<(InputSource, String), (u64, f32)>,
    seq: u64,
    smoothed: Vec<Option<f32>>,
    edges: Edges,
    recording: Option<Vec<InputEvent>>,
}

impl LiveSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Install a validated mapping (the previous one stays when validation fails).
    pub fn set_mapping(
        &mut self,
        p: &Project,
        m: Mapping,
        lib: Option<Library>,
    ) -> Result<(), String> {
        m.validate(p, lib.as_ref())?;
        self.smoothed = vec![None; m.params.len()];
        self.edges = Edges::default();
        self.mapping = m;
        self.library = lib;
        Ok(())
    }

    pub fn mapping(&self) -> &Mapping {
        &self.mapping
    }

    pub fn library(&self) -> Option<&Library> {
        self.library.as_ref()
    }

    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    pub fn recorded_len(&self) -> usize {
        self.recording.as_ref().map_or(0, |r| r.len())
    }

    pub fn start_recording(&mut self) {
        self.recording = Some(Vec::new());
    }

    /// Stop and return the recorded events (time-sorted), if recording was on.
    pub fn stop_recording(&mut self) -> Option<Vec<InputEvent>> {
        let mut r = self.recording.take()?;
        r.sort_by(|a, b| a.time.total_cmp(&b.time));
        Some(r)
    }

    /// Feed events (already cleaned by [`clean_events`] or [`Recording::into_events`]).
    /// Returns the triggers that fired. While recording the events are kept.
    pub fn push(&mut self, events: &[InputEvent]) -> Result<Vec<Fired>, String> {
        let events = clean_events(events)?;
        if let Some(r) = &mut self.recording {
            if r.len() + events.len() > MAX_EVENTS {
                return Err("recording buffer is full".into());
            }
            r.extend(events.iter().cloned());
        }
        let mut fired = Vec::new();
        for e in &events {
            self.seq += 1;
            self.latest
                .insert((e.source, e.id.clone()), (self.seq, e.value));
            self.edges.feed(&self.mapping, e, &mut fired);
        }
        Ok(fired)
    }

    /// Current mapped value of every parameter map that has seen input: `(target, prop, value)`.
    /// Smoothing advances one step per call.
    pub fn preview(&mut self) -> Vec<(String, Prop, f32)> {
        let mut out = Vec::new();
        for (i, pm) in self.mapping.params.iter().enumerate() {
            let newest = self
                .latest
                .iter()
                .filter(|((s, id), _)| *s == pm.source && id_matches(&pm.id, id))
                .map(|(_, v)| *v)
                .max_by_key(|v| v.0);
            if let Some((_, x)) = newest {
                let y = pm.smoothed(self.smoothed[i], pm.shape(x));
                self.smoothed[i] = Some(y);
                out.push((pm.target.clone(), pm.prop, y));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::Ease;
    use crate::math::Vec2;
    use crate::project::Bone;

    fn proj() -> Project {
        let mut p = Project::new("l", 200.0, 200.0, 10.0, 21);
        p.bones = vec![Bone::new("arm", None, 20.0, Vec2::new(100.0, 100.0), 0.0)];
        p
    }

    fn ev(source: InputSource, id: &str, value: f32, time: f32) -> InputEvent {
        InputEvent {
            source,
            id: id.into(),
            value,
            time,
        }
    }

    fn keyed(p: &Project, i: usize) -> &Track {
        let Source::Keys { track } = &p.channels[i].source else {
            panic!("not keyed")
        };
        track
    }

    #[test]
    fn mapping_math_range_curve_deadzone_invert() {
        let mut m = ParamMap::new(InputSource::MidiCc, "1", "arm", Prop::BoneAngle);
        m.out_min = -90.0;
        m.out_max = 90.0;
        assert!(
            (m.shape(0.0) + 90.0).abs() < 1e-4
                && (m.shape(0.5)).abs() < 1e-4
                && (m.shape(1.0) - 90.0).abs() < 1e-4
        );
        m.invert = true;
        assert!((m.shape(0.0) - 90.0).abs() < 1e-4);
        m.invert = false;
        m.deadzone = 0.2;
        assert!(
            (m.shape(0.1) + 90.0).abs() < 1e-4,
            "inside the dead zone = out_min"
        );
        assert!(
            (m.shape(0.6) - (-90.0 + 180.0 * 0.5)).abs() < 1e-3,
            "rest rescaled"
        );
        m.deadzone = 0.0;
        m.curve = Curve::Pow { exp: 2.0 };
        assert!((m.shape(0.5) - (-90.0 + 180.0 * 0.25)).abs() < 1e-3);
        m.curve = Curve::Linear;
        m.in_min = 0.25;
        m.in_max = 0.75;
        assert!(
            (m.shape(0.0) + 90.0).abs() < 1e-4 && (m.shape(1.0) - 90.0).abs() < 1e-4,
            "input clamped to the range"
        );
        assert!((m.smoothed(Some(10.0), 20.0) - 20.0).abs() < 1e-6);
        m.smooth = 0.5;
        assert!((m.smoothed(Some(10.0), 20.0) - 15.0).abs() < 1e-6);
    }

    #[test]
    fn validation_rejects_bad_numbers() {
        let ok = ParamMap::new(InputSource::Key, "KeyA", "arm", Prop::BoneAngle);
        assert!(ok.validate().is_ok());
        for f in [
            |m: &mut ParamMap| m.in_max = 0.0,
            |m: &mut ParamMap| m.out_max = f32::NAN,
            |m: &mut ParamMap| m.deadzone = 1.0,
            |m: &mut ParamMap| m.smooth = -0.1,
            |m: &mut ParamMap| m.curve = Curve::Pow { exp: 0.0 },
            |m: &mut ParamMap| m.curve = Curve::Pow { exp: f32::INFINITY },
            |m: &mut ParamMap| m.id.clear(),
        ] {
            let mut m = ok.clone();
            f(&mut m);
            assert!(m.validate().is_err());
        }
        let p = proj();
        let bad_trig = |a: Action, th: f32| {
            Mapping {
                params: vec![],
                triggers: vec![TriggerMap {
                    source: InputSource::Key,
                    id: "Space".into(),
                    edge: Edge::Press,
                    threshold: th,
                    action: a,
                }],
            }
            .validate(&p, None)
        };
        assert!(bad_trig(Action::JumpToFrame { frame: 999.0 }, 0.5).is_err());
        assert!(bad_trig(Action::JumpToFrame { frame: f32::NAN }, 0.5).is_err());
        assert!(bad_trig(Action::JumpToFrame { frame: 3.0 }, 2.0).is_err());
        assert!(
            bad_trig(
                Action::PlayTake {
                    take: "x".into(),
                    speed: 1.0
                },
                0.5
            )
            .is_err(),
            "no library"
        );
        assert!(bad_trig(
            Action::FireEvent {
                name: " ".into(),
                data: serde_json::Value::Null
            },
            0.5
        )
        .is_err());
        assert!(bad_trig(
            Action::SetKey {
                target: "arm".into(),
                prop: Prop::BoneAngle,
                value: f32::INFINITY,
                ease: Ease::Linear
            },
            0.5
        )
        .is_err());
        assert!(bad_trig(Action::JumpToFrame { frame: 3.0 }, 0.5).is_ok());
    }

    #[test]
    fn id_patterns() {
        assert!(id_matches("7", "3:7") && id_matches("3:7", "3:7") && id_matches("*", "x"));
        assert!(!id_matches("7", "3:17") && !id_matches("1:7", "3:7") && !id_matches("7", "7:3"));
    }

    #[test]
    fn events_are_cleaned_and_sorted() {
        let v = clean_events(&[
            ev(InputSource::MouseX, "x", 2.0, 1.0),
            ev(InputSource::MouseX, "x", -1.0, 0.5),
        ])
        .unwrap();
        assert_eq!((v[0].time, v[0].value, v[1].value), (0.5, 0.0, 1.0));
        assert!(clean_events(&[ev(InputSource::MouseX, "x", f32::NAN, 0.0)]).is_err());
        assert!(clean_events(&[ev(InputSource::MouseX, "x", 0.0, -1.0)]).is_err());
        assert!(clean_events(&[ev(InputSource::MouseX, "x", 0.0, f32::INFINITY)]).is_err());
        assert!(clean_events(&[ev(InputSource::MouseX, "", 0.0, 0.0)]).is_err());
    }

    #[test]
    fn recorded_events_become_keys() {
        let mut p = proj();
        // fps 10: a fader moves 0 -> 1 between t=0.5 s and t=1.5 s
        let events: Vec<InputEvent> = (0..=10)
            .map(|i| {
                ev(
                    InputSource::MidiCc,
                    "1:1",
                    i as f32 / 10.0,
                    0.5 + i as f32 * 0.1,
                )
            })
            .collect();
        let mut pm = ParamMap::new(InputSource::MidiCc, "1", "arm", Prop::BoneAngle);
        pm.out_min = 0.0;
        pm.out_max = 100.0;
        let m = Mapping {
            params: vec![pm],
            triggers: vec![],
        };
        let r = bake(&mut p, &events, &m, None, 0.01).unwrap();
        assert_eq!(r.channels, 1);
        let t = keyed(&p, 0);
        assert!(t.keys.len() <= 6, "reduced: {}", t.keys.len());
        assert!(
            (t.eval(0.0).unwrap()).abs() < 1e-3,
            "first value held before the first event"
        );
        assert!((t.eval(5.0).unwrap()).abs() < 1e-3);
        assert!(
            (t.eval(10.0).unwrap() - 50.0).abs() < 0.5,
            "{}",
            t.eval(10.0).unwrap()
        );
        assert!((t.eval(15.0).unwrap() - 100.0).abs() < 1e-3);
        assert!(
            (t.eval(20.0).unwrap() - 100.0).abs() < 1e-3,
            "held to the end"
        );
        // deterministic
        let mut p2 = proj();
        bake(&mut p2, &events, &m, None, 0.01).unwrap();
        assert_eq!(p.to_json_pretty(), p2.to_json_pretty());
    }

    #[test]
    fn smoothing_lags_and_additive_mode_is_kept() {
        let mut p = proj();
        let events = vec![
            ev(InputSource::MouseX, "x", 0.0, 0.0),
            ev(InputSource::MouseX, "x", 1.0, 0.5),
        ];
        let mut pm = ParamMap::new(InputSource::MouseX, "x", "arm", Prop::BoneAngle);
        pm.smooth = 0.8;
        pm.additive = true;
        bake(
            &mut p,
            &events,
            &Mapping {
                params: vec![pm],
                triggers: vec![],
            },
            None,
            0.0,
        )
        .unwrap();
        assert_eq!(p.channels[0].mode, Mode::Add);
        let t = keyed(&p, 0);
        let (a, b) = (t.eval(5.0).unwrap(), t.eval(20.0).unwrap());
        assert!(a > 0.0 && a < 0.5 && b > a && b < 1.0, "{a} {b}");
    }

    #[test]
    fn triggers_edge_detection_and_actions() {
        let mut p = proj();
        let mut lib = Library::default();
        // a take recorded from a tiny animation
        let mut src = proj();
        src.channels.push(Channel::keys(
            "arm",
            Prop::BoneAngle,
            Track::with_keys(&[(0.0, 0.0, Ease::Linear), (4.0, 40.0, Ease::Linear)]),
        ));
        crate::motionlib::record(&mut lib, &src, "wave", 0.0, 4.0, &[]).unwrap();
        let trig = |id: &str, edge, a| TriggerMap {
            source: InputSource::Key,
            id: id.into(),
            edge,
            threshold: 0.5,
            action: a,
        };
        let m = Mapping {
            params: vec![],
            triggers: vec![
                trig(
                    "KeyA",
                    Edge::Press,
                    Action::SetKey {
                        target: "arm".into(),
                        prop: Prop::BoneAngle,
                        value: 33.0,
                        ease: Ease::Step,
                    },
                ),
                trig(
                    "KeyB",
                    Edge::Both,
                    Action::FireEvent {
                        name: "flash".into(),
                        data: serde_json::json!({"n": 1}),
                    },
                ),
                trig("KeyC", Edge::Release, Action::JumpToFrame { frame: 2.0 }),
                trig(
                    "KeyD",
                    Edge::Press,
                    Action::PlayTake {
                        take: "wave".into(),
                        speed: 1.0,
                    },
                ),
            ],
        };
        let events = vec![
            ev(InputSource::Key, "KeyA", 1.0, 0.3), // frame 3
            ev(InputSource::Key, "KeyA", 1.0, 0.4), // key repeat: no second fire
            ev(InputSource::Key, "KeyA", 0.0, 0.5),
            ev(InputSource::Key, "KeyB", 1.0, 0.6), // frame 6 press
            ev(InputSource::Key, "KeyB", 0.0, 0.8), // frame 8 release
            ev(InputSource::Key, "KeyC", 1.0, 0.9),
            ev(InputSource::Key, "KeyC", 0.0, 1.0), // frame 10
            ev(InputSource::Key, "KeyD", 1.0, 1.2), // frame 12
            ev(InputSource::Key, "KeyD", 1.0, 99.0), // beyond the project: skipped silently, not a repeat press
            ev(InputSource::Key, "Other", 1.0, 0.1),
        ];
        let fired = fired_triggers(&m, &clean_events(&events).unwrap());
        assert_eq!(fired.len(), 5, "{fired:?}");
        let r = bake(&mut p, &events, &m, Some(&lib), 0.01).unwrap();
        assert_eq!((r.keys_set, r.events, r.takes, r.skipped), (1, 3, 1, 0));
        let flash: Vec<f32> = p
            .events
            .iter()
            .filter(|e| e.name == "flash")
            .map(|e| e.frame)
            .collect();
        assert_eq!(flash, vec![6.0, 8.0]);
        let jump = p.events.iter().find(|e| e.name == "jump").unwrap();
        assert_eq!((jump.frame, jump.data["to"].as_f64()), (10.0, Some(2.0)));
        // the SetKey made a channel with a key at frame 3, and the take pasted onto it at frame 12
        let t = keyed(&p, 0);
        assert!(t.keys.iter().any(|k| k.frame == 3.0 && k.value == 33.0));
        assert!(
            t.keys.iter().any(|k| k.frame == 12.0)
                && t.keys
                    .iter()
                    .any(|k| k.frame == 16.0 && (k.value - 40.0).abs() < 1e-4)
        );
    }

    #[test]
    fn failed_bake_leaves_project_untouched() {
        let mut p = proj();
        let before = p.to_json_pretty();
        let events = vec![ev(InputSource::MidiCc, "1:1", 0.5, 0.0)];
        let bad_target = Mapping {
            params: vec![ParamMap::new(
                InputSource::MidiCc,
                "1",
                "ghost",
                Prop::BoneAngle,
            )],
            triggers: vec![],
        };
        assert!(bake(&mut p, &events, &bad_target, None, 0.01).is_err());
        let mut nan = ParamMap::new(InputSource::MidiCc, "1", "arm", Prop::BoneAngle);
        nan.out_max = f32::NAN;
        assert!(bake(
            &mut p,
            &events,
            &Mapping {
                params: vec![nan],
                triggers: vec![]
            },
            None,
            0.01
        )
        .is_err());
        assert!(bake(
            &mut p,
            &[ev(InputSource::MidiCc, "1:1", f32::NAN, 0.0)],
            &Mapping::default(),
            None,
            0.01
        )
        .is_err());
        assert!(bake(&mut p, &events, &Mapping::default(), None, -1.0).is_err());
        let late = Mapping {
            params: vec![],
            triggers: vec![TriggerMap {
                source: InputSource::Key,
                id: "A".into(),
                edge: Edge::Press,
                threshold: 0.5,
                action: Action::PlayTake {
                    take: "none".into(),
                    speed: 1.0,
                },
            }],
        };
        assert!(
            bake(&mut p, &events, &late, Some(&Library::default()), 0.01).is_err(),
            "unknown take"
        );
        assert_eq!(p.to_json_pretty(), before);
        // no matching events: ok, noted, nothing added
        let m = Mapping {
            params: vec![ParamMap::new(
                InputSource::MidiCc,
                "9",
                "arm",
                Prop::BoneAngle,
            )],
            triggers: vec![],
        };
        let r = bake(&mut p, &events, &m, None, 0.01).unwrap();
        assert_eq!((r.channels, r.notes.len()), (0, 1));
    }

    #[test]
    fn locked_channels_are_refused() {
        let mut p = proj();
        p.channels.push(Channel::keys(
            "arm",
            Prop::BoneAngle,
            Track::with_keys(&[(0.0, 0.0, Ease::Linear)]),
        ));
        crate::chanmeta::set_meta(
            &mut p,
            "arm",
            Some(Prop::BoneAngle),
            &crate::chanmeta::Change {
                lock: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        let before = p.to_json_pretty();
        let events = vec![ev(InputSource::MidiCc, "1:1", 0.5, 0.0)];
        let m = Mapping {
            params: vec![ParamMap::new(
                InputSource::MidiCc,
                "1",
                "arm",
                Prop::BoneAngle,
            )],
            triggers: vec![],
        };
        let e = bake(&mut p, &events, &m, None, 0.01).unwrap_err();
        assert!(e.to_lowercase().contains("lock"), "{e}");
        let m = Mapping {
            params: vec![],
            triggers: vec![TriggerMap {
                source: InputSource::Key,
                id: "A".into(),
                edge: Edge::Press,
                threshold: 0.5,
                action: Action::SetKey {
                    target: "arm".into(),
                    prop: Prop::BoneAngle,
                    value: 1.0,
                    ease: Ease::Linear,
                },
            }],
        };
        assert!(bake(&mut p, &events, &m, None, 0.01).is_err());
        assert_eq!(p.to_json_pretty(), before);
    }

    #[test]
    fn midi_file_is_an_offline_live_source() {
        // CC1 sweeps 0 -> 127 over one second at 120 bpm, built as a format 0 file
        let mut t = vec![0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, 0x00, 0xB0, 1, 0];
        t.extend([0x83, 0x60, 0xB0, 1, 127]); // delta 480 ticks (VLQ 0x83 0x60) = 1 beat = 0.5 s at division 480
        t.extend([0x00, 0xFF, 0x2F, 0x00]);
        let mut f = b"MThd".to_vec();
        f.extend(6u32.to_be_bytes());
        f.extend([0, 0, 0, 1, 0x01, 0xE0]);
        f.extend(b"MTrk");
        f.extend((t.len() as u32).to_be_bytes());
        f.extend(&t);
        let ev = crate::midi::smf_to_events(&f).unwrap();
        assert_eq!(ev.len(), 2);
        assert!((ev[1].time - 0.5).abs() < 1e-6);
        let mut p = proj();
        let mut pm = ParamMap::new(InputSource::MidiCc, "1", "arm", Prop::BoneAngle);
        pm.out_max = 60.0;
        bake(
            &mut p,
            &ev,
            &Mapping {
                params: vec![pm],
                triggers: vec![],
            },
            None,
            0.01,
        )
        .unwrap();
        let t = keyed(&p, 0);
        assert!(t.eval(0.0).unwrap().abs() < 1e-3 && (t.eval(5.0).unwrap() - 60.0).abs() < 1e-3);
    }

    #[test]
    fn recording_json_forms() {
        let (e, w) =
            parse_recording_json(r#"[{"source":"mouse_x","id":"x","value":0.5,"time":1.0}]"#)
                .unwrap();
        assert_eq!((e.len(), w.len()), (1, 0));
        let (e, w) = parse_recording_json(r#"{"midi":[{"time":0.5,"bytes":[176,1,64]},{"time":0.6,"bytes":[176,1]}],"events":[{"source":"key","id":"A","value":1,"time":0.1}]}"#).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(w.len(), 1);
        assert_eq!(e[0].source, InputSource::Key, "sorted by time");
        assert!(parse_recording_json("{").is_err());
        assert!(
            parse_recording_json(r#"[{"source":"nope","id":"x","value":0,"time":0}]"#).is_err()
        );
    }

    #[test]
    fn live_session_previews_smooths_fires_and_records() {
        let p = proj();
        let mut s = LiveSession::new();
        let mut pm = ParamMap::new(InputSource::MouseX, "x", "arm", Prop::BoneAngle);
        pm.out_max = 90.0;
        pm.smooth = 0.5;
        let m = Mapping {
            params: vec![pm],
            triggers: vec![TriggerMap {
                source: InputSource::MouseButton,
                id: "0".into(),
                edge: Edge::Press,
                threshold: 0.5,
                action: Action::JumpToFrame { frame: 4.0 },
            }],
        };
        s.set_mapping(&p, m.clone(), None).unwrap();
        assert!(s.preview().is_empty(), "no input yet");
        s.start_recording();
        let fired = s
            .push(&[
                ev(InputSource::MouseX, "x", 1.0, 0.0),
                ev(InputSource::MouseButton, "0", 1.0, 0.1),
            ])
            .unwrap();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].action, Action::JumpToFrame { frame: 4.0 });
        let a = s.preview();
        assert_eq!(a[0].2, 90.0, "first step starts at the value");
        s.push(&[ev(InputSource::MouseX, "x", 0.0, 0.2)]).unwrap();
        let b = s.preview();
        assert!((b[0].2 - 45.0).abs() < 1e-4, "{b:?}");
        assert_eq!(s.recorded_len(), 3);
        let rec = s.stop_recording().unwrap();
        assert_eq!(rec.len(), 3);
        assert!(!s.is_recording() && s.stop_recording().is_none());
        // the recording bakes to keys
        let mut q = proj();
        let r = bake(&mut q, &rec, &m, None, 0.01).unwrap();
        assert_eq!((r.channels, r.events), (1, 1));
        // a bad mapping is refused and the old one stays
        let mut bad = m.clone();
        bad.params[0].in_max = 0.0;
        assert!(s.set_mapping(&p, bad, None).is_err());
        assert_eq!(s.mapping().params.len(), 1);
        assert!(s
            .push(&[ev(InputSource::MouseX, "x", f32::NAN, 0.0)])
            .is_err());
    }
}
