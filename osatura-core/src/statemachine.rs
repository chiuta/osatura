// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Deterministic animation state machine.
//!
//! Clean-room design: instead of a runtime object that is mutated by calls (and so depends on
//! call order and timing), the machine is a *pure function of an input event log*:
//! `simulate(machine, events, frames)` always yields the same per-frame blend description,
//! and `bake` turns it into ordinary keyed channels. Inputs are numbers; events are named
//! pulses that exist for exactly one frame. Everything can therefore be tested, diffed, cached
//! and exported without a special runtime.

use crate::anim::{Ease, Track};
use crate::perf::simplify_samples;
use crate::project::{Channel, Mode, Prop, Source};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

/// A state is a named clip of keyed channels, `length` frames long.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub name: String,
    pub length: f32,
    #[serde(default = "yes")]
    pub looped: bool,
    pub channels: Vec<Channel>,
}

fn yes() -> bool {
    true
}

/// When a transition may fire.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Cond {
    /// The named event arrives on this frame.
    Event { name: String },
    Above { input: String, value: f32 },
    Below { input: String, value: f32 },
    Equals { input: String, value: f32 },
    /// A non-looping state has played to its end.
    Finished,
    /// At least this many frames have passed in the state.
    After { frames: f32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// State name or `"*"` for any state.
    pub from: String,
    pub to: String,
    pub when: Cond,
    /// Crossfade length in frames.
    #[serde(default = "def_fade")]
    pub fade: f32,
    #[serde(default = "def_ease")]
    pub ease: Ease,
}

fn def_fade() -> f32 {
    6.0
}
fn def_ease() -> Ease {
    Ease::Smooth
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateMachine {
    pub initial: String,
    pub states: Vec<Clip>,
    #[serde(default)]
    pub transitions: Vec<Transition>,
}

/// An input change or event at a frame. Events use `value` as the new input value too, so
/// `{"frame": 12, "name": "speed", "value": 0.8}` both sets the input and pulses the event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputEvent {
    pub frame: u32,
    pub name: String,
    #[serde(default = "one")]
    pub value: f32,
}

fn one() -> f32 {
    1.0
}

/// What is playing on one frame.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameState {
    pub frame: u32,
    pub state: usize,
    /// Local time inside `state` (before looping).
    pub local: f32,
    /// Fade target: (state, local time, eased blend 0..1).
    pub fade: Option<(usize, f32, f32)>,
}

impl StateMachine {
    pub fn validate(&self) -> Result<(), String> {
        let names: HashSet<&str> = self.states.iter().map(|s| s.name.as_str()).collect();
        if names.len() != self.states.len() {
            return Err("duplicate state names".into());
        }
        if !names.contains(self.initial.as_str()) {
            return Err(format!("initial state '{}' does not exist", self.initial));
        }
        for s in &self.states {
            if s.length.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
                return Err(format!("state '{}' needs a positive length", s.name));
            }
        }
        for t in &self.transitions {
            if t.from != "*" && !names.contains(t.from.as_str()) {
                return Err(format!("transition from unknown state '{}'", t.from));
            }
            if !names.contains(t.to.as_str()) {
                return Err(format!("transition to unknown state '{}'", t.to));
            }
            if !matches!(t.fade.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) {
                return Err("transition fade must be >= 0".into());
            }
        }
        Ok(())
    }

    fn index(&self, name: &str) -> usize {
        self.states.iter().position(|s| s.name == name).unwrap()
    }
}

struct Fade {
    to: usize,
    start: u32,
    len: f32,
    ease: Ease,
}

/// Run the machine for `frames` frames against an input event log.
pub fn simulate(sm: &StateMachine, events: &[InputEvent], frames: u32) -> Result<Vec<FrameState>, String> {
    sm.validate()?;
    let mut by_frame: BTreeMap<u32, Vec<&InputEvent>> = BTreeMap::new();
    for e in events {
        by_frame.entry(e.frame).or_default().push(e);
    }
    let mut inputs: HashMap<String, f32> = HashMap::new();
    let mut cur = sm.index(&sm.initial);
    let mut entered = 0u32;
    let mut fade: Option<Fade> = None;
    let mut out = Vec::with_capacity(frames as usize);
    for f in 0..frames {
        let mut fired: HashSet<&str> = HashSet::new();
        if let Some(evs) = by_frame.get(&f) {
            for e in evs {
                inputs.insert(e.name.clone(), e.value);
                fired.insert(e.name.as_str());
            }
        }
        // finish a completed fade
        if let Some(fd) = &fade {
            if (f - fd.start) as f32 >= fd.len {
                cur = fd.to;
                entered = fd.start;
                fade = None;
            }
        }
        // look for a transition (only when not fading)
        if fade.is_none() {
            let local = (f - entered) as f32;
            let clip = &sm.states[cur];
            for t in &sm.transitions {
                if t.from != "*" && sm.index(&t.from) != cur {
                    continue;
                }
                let to = sm.index(&t.to);
                if t.from == "*" && to == cur {
                    continue;
                }
                let ok = match &t.when {
                    Cond::Event { name } => fired.contains(name.as_str()),
                    Cond::Above { input, value } => inputs.get(input).is_some_and(|v| v > value),
                    Cond::Below { input, value } => inputs.get(input).is_some_and(|v| v < value),
                    Cond::Equals { input, value } => inputs.get(input).is_some_and(|v| (v - value).abs() < 1e-6),
                    Cond::Finished => !clip.looped && local >= clip.length,
                    Cond::After { frames } => local >= *frames,
                };
                if ok {
                    if t.fade <= 0.0 {
                        cur = to;
                        entered = f;
                    } else {
                        fade = Some(Fade { to, start: f, len: t.fade, ease: t.ease });
                    }
                    break;
                }
            }
        }
        let fs = FrameState {
            frame: f,
            state: cur,
            local: (f - entered) as f32,
            fade: fade.as_ref().map(|fd| {
                let t = ((f - fd.start) as f32 / fd.len).clamp(0.0, 1.0);
                (fd.to, (f - fd.start) as f32, fd.ease.apply(t))
            }),
        };
        out.push(fs);
    }
    Ok(out)
}

fn clip_time(c: &Clip, local: f32) -> f32 {
    if c.looped {
        local.rem_euclid(c.length)
    } else {
        local.min(c.length)
    }
}

fn clip_tracks(c: &Clip) -> HashMap<(String, Prop), (&Track, Mode)> {
    let mut m = HashMap::new();
    for ch in &c.channels {
        if let Source::Keys { track } = &ch.source {
            m.insert((ch.target.clone(), ch.prop), (track, ch.mode));
        }
    }
    m
}

/// Bake the simulation into keyed channels (one per animated `(target, prop)`), reduced to
/// within `tol` of the exact per-frame blend. Only keyed channels take part.
pub fn bake(sm: &StateMachine, timeline: &[FrameState], tol: f32) -> Vec<Channel> {
    let tracks: Vec<_> = sm.states.iter().map(clip_tracks).collect();
    let mut keys: Vec<(String, Prop)> = tracks.iter().flat_map(|m| m.keys().cloned()).collect::<HashSet<_>>().into_iter().collect();
    keys.sort_by(|a, b| (a.0.as_str(), format!("{:?}", a.1)).cmp(&(b.0.as_str(), format!("{:?}", b.1))));
    keys.into_iter()
        .map(|(target, prop)| {
            let mut samples = Vec::new();
            let mut mode = Mode::Replace;
            for fs in timeline {
                let sample = |si: usize, local: f32| -> Option<f32> {
                    tracks[si].get(&(target.clone(), prop)).and_then(|(t, _)| t.eval(clip_time(&sm.states[si], local)))
                };
                if let Some((t, m)) = tracks[fs.state].get(&(target.clone(), prop)) {
                    let _ = t;
                    mode = *m;
                }
                let a = sample(fs.state, fs.local);
                let v = match fs.fade {
                    None => a,
                    Some((to, lb, w)) => match (a, sample(to, lb)) {
                        (Some(x), Some(y)) => Some(x + (y - x) * w),
                        (Some(x), None) => Some(x),
                        (None, Some(y)) => Some(y),
                        (None, None) => None,
                    },
                };
                if let Some(v) = v {
                    samples.push((fs.frame as f32, v));
                }
            }
            let mut ch = Channel::keys(&target, prop, simplify_samples(&samples, tol));
            ch.mode = mode;
            ch
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(name: &str, value: f32, looped: bool, len: f32) -> Clip {
        Clip { name: name.into(), length: len, looped, channels: vec![Channel::keys("b", Prop::BoneAngle, Track::with_keys(&[(0.0, value, Ease::Linear), (len, value, Ease::Linear)]))] }
    }

    fn machine() -> StateMachine {
        StateMachine {
            initial: "idle".into(),
            states: vec![clip("idle", 0.0, true, 24.0), clip("jump", 90.0, false, 12.0), clip("land", 10.0, false, 6.0)],
            transitions: vec![
                Transition { from: "idle".into(), to: "jump".into(), when: Cond::Event { name: "jump".into() }, fade: 4.0, ease: Ease::Linear },
                Transition { from: "jump".into(), to: "land".into(), when: Cond::Finished, fade: 2.0, ease: Ease::Linear },
                Transition { from: "land".into(), to: "idle".into(), when: Cond::After { frames: 6.0 }, fade: 0.0, ease: Ease::Linear },
            ],
        }
    }

    fn ev(frame: u32, name: &str) -> InputEvent {
        InputEvent { frame, name: name.into(), value: 1.0 }
    }

    #[test]
    fn deterministic_and_follows_the_event_log() {
        let sm = machine();
        let a = simulate(&sm, &[ev(10, "jump")], 60).unwrap();
        let b = simulate(&sm, &[ev(10, "jump")], 60).unwrap();
        assert_eq!(a, b);
        assert_eq!(a[9].state, 0, "idle before the event");
        assert!(a[10].fade.is_some(), "fade begins on the event frame");
        assert_eq!(a[14].state, 1, "jump after a 4-frame fade");
        // jump (12 frames) finishes → land → idle again
        assert!(a.iter().any(|f| f.state == 2));
        assert_eq!(a[59].state, 0);
    }

    #[test]
    fn no_events_stays_in_initial_state() {
        let a = simulate(&machine(), &[], 30).unwrap();
        assert!(a.iter().all(|f| f.state == 0 && f.fade.is_none()));
        assert_eq!(a[25].local, 25.0, "local time counts up (looping is applied when sampling)");
    }

    #[test]
    fn input_conditions_and_any_state() {
        let mut sm = machine();
        sm.transitions = vec![Transition { from: "*".into(), to: "jump".into(), when: Cond::Above { input: "speed".into(), value: 0.5 }, fade: 0.0, ease: Ease::Linear }];
        let a = simulate(&sm, &[InputEvent { frame: 5, name: "speed".into(), value: 0.9 }], 10).unwrap();
        assert_eq!(a[4].state, 0);
        assert_eq!(a[5].state, 1);
        assert_eq!(a[9].state, 1, "'*' never re-enters the state it is already in");
    }

    #[test]
    fn bake_matches_the_blend_and_is_reduced() {
        let sm = machine();
        let tl = simulate(&sm, &[ev(10, "jump")], 40).unwrap();
        let ch = bake(&sm, &tl, 0.01);
        assert_eq!(ch.len(), 1);
        let Source::Keys { track } = &ch[0].source else { panic!() };
        assert!(track.eval(5.0).unwrap().abs() < 1e-3, "idle value");
        let mid = track.eval(12.0).unwrap();
        assert!((mid - 45.0).abs() < 0.5, "half-way through a linear 4-frame fade to 90: {mid}");
        assert!((track.eval(20.0).unwrap() - 90.0).abs() < 0.5);
        assert!(track.keys.len() < 40, "reduced from per-frame samples: {}", track.keys.len());
    }

    #[test]
    fn validation_errors() {
        let mut sm = machine();
        sm.initial = "nope".into();
        assert!(simulate(&sm, &[], 5).is_err());
        let mut sm = machine();
        sm.transitions[0].to = "ghost".into();
        assert!(sm.validate().unwrap_err().contains("ghost"));
        let mut sm = machine();
        sm.states[0].length = 0.0;
        assert!(sm.validate().is_err());
    }

    #[test]
    fn json_round_trip() {
        let sm = machine();
        let back: StateMachine = serde_json::from_str(&serde_json::to_string(&sm).unwrap()).unwrap();
        assert_eq!(sm, back);
    }
}
