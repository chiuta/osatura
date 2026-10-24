// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `machine` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------------------------

pub(super) struct RtCond {
    pub(super) input: usize,
    pub(super) op: Cmp,
    pub(super) value: f64,
}

pub(super) struct RtTrans {
    pub(super) from: Option<usize>,
    pub(super) to: usize,
    pub(super) conds: Vec<RtCond>,
    pub(super) exit: Option<f32>,
    /// Effective crossfade, seconds.
    pub(super) dur: f64,
    pub(super) ease: Ease,
}

pub(super) enum RtKind {
    Leaf(Leaf),
    Blend { input: usize, thresholds: Vec<f32>, children: Vec<Leaf> },
}

pub(super) struct RtState {
    pub(super) name: String,
    pub(super) mode: PlayMode,
    pub(super) speed: f64,
    pub(super) kind: RtKind,
}

#[derive(Clone, Copy)]
pub(super) struct Play {
    pub(super) state: usize,
    /// Position in loops of the state's length (0 at entry).
    pub(super) pos: f64,
}

pub(super) enum FadeFrom {
    Live(Play),
    Snap(Pose),
}

pub(super) struct Fade {
    pub(super) from: FadeFrom,
    pub(super) elapsed: f64,
    pub(super) dur: f64,
    pub(super) ease: Ease,
}

pub(super) struct LayerRt {
    pub(super) name: String,
    pub(super) states: Vec<RtState>,
    pub(super) trans: Vec<RtTrans>,
    pub(super) cur: Play,
    pub(super) fade: Option<Fade>,
}

/// A live machine: feed inputs, advance time, read the pose / frame.
pub struct MachineRuntime {
    pub(super) machine_name: String,
    pub(super) base: Project,
    pub(super) asset_dir: std::path::PathBuf,
    pub(super) fps: f32,
    pub(super) base_frame: f32,
    pub(super) in_names: Vec<String>,
    pub(super) in_kind: Vec<InputKind>,
    pub(super) in_hold: Vec<bool>,
    pub(super) values: Vec<f64>,
    pub(super) layers: Vec<LayerRt>,
    pub(super) time: f64,
    pub(super) events: Vec<MachineEvent>,
    pub(super) bindings: crate::binding::BindingRuntime,
}

pub(super) fn phase_of(mode: PlayMode, pos: f64) -> f64 {
    match mode {
        PlayMode::Loop => pos - pos.floor(),
        PlayMode::Once => pos.clamp(0.0, 1.0),
        PlayMode::PingPong => pos.rem_euclid(2.0) / 2.0,
    }
}

/// Position within the source (0..1) that is sampled.
pub(super) fn sample_phase(mode: PlayMode, pos: f64) -> f32 {
    match mode {
        PlayMode::Loop => (pos - pos.floor()) as f32,
        PlayMode::Once => pos.clamp(0.0, 1.0) as f32,
        PlayMode::PingPong => {
            let m = pos.rem_euclid(2.0);
            (if m <= 1.0 { m } else { 2.0 - m }) as f32
        }
    }
}

impl MachineRuntime {
    /// Build the runtime of the named machine (asset directory: current directory).
    pub fn new(project: &Project, machine: &str) -> Result<MachineRuntime, String> {
        MachineRuntime::with_asset_dir(project, machine, std::path::Path::new("."))
    }

    pub fn with_asset_dir(project: &Project, machine: &str, asset_dir: &std::path::Path) -> Result<MachineRuntime, String> {
        let m = project.machines.iter().find(|m| m.name == machine).ok_or_else(|| format!("unknown machine '{machine}'"))?;
        let mut errs = validate_machine(project, m);
        errs.extend(crate::binding::validate(project));
        if !errs.is_empty() {
            return Err(errs.join("; "));
        }
        let in_ix: HashMap<&str, usize> = m.inputs.iter().enumerate().map(|(i, x)| (x.name.as_str(), i)).collect();
        let mut layers = Vec::new();
        for l in &m.layers {
            let sx: HashMap<&str, usize> = l.states.iter().enumerate().map(|(i, s)| (s.name.as_str(), i)).collect();
            let mut states = Vec::new();
            for s in &l.states {
                let kind = match &s.source {
                    StateSource::Blend { input, children } => RtKind::Blend {
                        input: in_ix[input.as_str()],
                        thresholds: children.iter().map(|c| c.threshold).collect(),
                        children: children.iter().map(|c| resolve_leaf(project, &c.source, &l.mask)).collect::<Result<_, _>>()?,
                    },
                    other => RtKind::Leaf(resolve_leaf(project, other, &l.mask)?),
                };
                states.push(RtState { name: s.name.clone(), mode: s.mode, speed: s.speed as f64, kind });
            }
            let trans = l
                .transitions
                .iter()
                .map(|t| {
                    let to = sx[t.to.as_str()];
                    RtTrans {
                        from: if t.from == "*" { None } else { Some(sx[t.from.as_str()]) },
                        to,
                        conds: t.conditions.iter().map(|c| RtCond { input: in_ix[c.input.as_str()], op: c.op, value: c.value }).collect(),
                        exit: t.exit_time,
                        dur: t.duration.unwrap_or(l.states[to].blend_in) as f64,
                        ease: t.ease,
                    }
                })
                .collect();
            layers.push(LayerRt { name: l.name.clone(), states, trans, cur: Play { state: sx[l.entry.as_str()], pos: 0.0 }, fade: None });
        }
        let mut base = project.clone();
        base.machines.clear();
        base.bindings.clear();
        let mut rt = MachineRuntime {
            machine_name: m.name.clone(),
            base,
            asset_dir: asset_dir.to_path_buf(),
            fps: project.fps,
            base_frame: 0.0,
            in_names: m.inputs.iter().map(|i| i.name.clone()).collect(),
            in_kind: m.inputs.iter().map(|i| i.kind).collect(),
            in_hold: m.inputs.iter().map(|i| i.hold).collect(),
            values: m.inputs.iter().map(|i| if i.kind == InputKind::Trigger { 0.0 } else { i.default }).collect(),
            layers,
            time: 0.0,
            events: Vec::new(),
            bindings: crate::binding::BindingRuntime::new(&project.bindings)?,
        };
        for l in 0..rt.layers.len() {
            let name = rt.layers[l].states[rt.layers[l].cur.state].name.clone();
            rt.events.push(MachineEvent { time: 0.0, layer: rt.layers[l].name.clone(), kind: EventKind::Entered, state: name });
        }
        Ok(rt)
    }

    pub fn machine_name(&self) -> &str {
        &self.machine_name
    }
    pub fn time(&self) -> f64 {
        self.time
    }
    /// Project frame at which the project's own channels are evaluated (default 0).
    pub fn set_base_frame(&mut self, f: f32) {
        self.base_frame = f;
    }

    pub(super) fn input_ix(&self, name: &str) -> Result<usize, String> {
        self.in_names.iter().position(|n| n == name).ok_or_else(|| format!("machine '{}' has no input '{name}'", self.machine_name))
    }

    pub fn set_bool(&mut self, name: &str, v: bool) -> Result<(), String> {
        let i = self.input_ix(name)?;
        if self.in_kind[i] != InputKind::Bool {
            return Err(format!("input '{name}' is not a bool"));
        }
        self.values[i] = if v { 1.0 } else { 0.0 };
        Ok(())
    }

    pub fn set_number(&mut self, name: &str, v: f64) -> Result<(), String> {
        let i = self.input_ix(name)?;
        if self.in_kind[i] != InputKind::Number {
            return Err(format!("input '{name}' is not a number"));
        }
        if !v.is_finite() {
            return Err(format!("input '{name}': value must be finite"));
        }
        self.values[i] = v;
        Ok(())
    }

    pub fn fire(&mut self, name: &str) -> Result<(), String> {
        let i = self.input_ix(name)?;
        if self.in_kind[i] != InputKind::Trigger {
            return Err(format!("input '{name}' is not a trigger"));
        }
        self.values[i] = 1.0;
        Ok(())
    }

    /// Set an input of any kind from a number (bool: `!= 0`; trigger: fires when `!= 0`).
    pub fn set_input(&mut self, name: &str, v: f64) -> Result<(), String> {
        match self.in_kind[self.input_ix(name)?] {
            InputKind::Bool => self.set_bool(name, v != 0.0),
            InputKind::Number => self.set_number(name, v),
            InputKind::Trigger => {
                if v != 0.0 {
                    self.fire(name)
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn input_value(&self, name: &str) -> Option<f64> {
        self.in_names.iter().position(|n| n == name).map(|i| self.values[i])
    }

    /// Publish an external value for the project's data bindings (applied on the next `step`).
    pub fn set_value(&mut self, path: &str, v: f64) -> Result<(), String> {
        self.bindings.set_value(path, v)
    }

    /// Name of the current state of a layer.
    pub fn state_of(&self, layer: &str) -> Option<&str> {
        self.layers.iter().find(|l| l.name == layer).map(|l| l.states[l.cur.state].name.as_str())
    }

    /// `(layer, state)` for every layer.
    pub fn states(&self) -> Vec<(String, String)> {
        self.layers.iter().map(|l| (l.name.clone(), l.states[l.cur.state].name.clone())).collect()
    }

    /// Whether the layer is currently crossfading.
    pub fn is_fading(&self, layer: &str) -> bool {
        self.layers.iter().find(|l| l.name == layer).is_some_and(|l| l.fade.is_some())
    }

    /// Events not yet taken, oldest first.
    pub fn events(&self) -> &[MachineEvent] {
        &self.events
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Events since the last call (entered / exited), oldest first.
    pub fn take_events(&mut self) -> Vec<MachineEvent> {
        std::mem::take(&mut self.events)
    }

    // ---- sampling ----

    pub(super) fn leaf_pose(leaf: &Leaf, phase: f32) -> Pose {
        let mut pose = Pose::default();
        let local = leaf.offset + phase * leaf.len;
        for (t, p, track, w) in &leaf.tracks {
            if let Some(v) = track.eval(local) {
                if v.is_finite() {
                    pose.add(t, *p, v, *w);
                }
            }
        }
        pose
    }

    pub(super) fn weights_of(&self, thresholds: &[f32], input: usize) -> Vec<f32> {
        blend_weights(thresholds, self.values[input] as f32)
    }

    pub(super) fn state_len(&self, st: &RtState) -> f64 {
        match &st.kind {
            RtKind::Leaf(l) => l.len as f64,
            RtKind::Blend { input, thresholds, children } => {
                let w = self.weights_of(thresholds, *input);
                children.iter().zip(&w).map(|(c, w)| c.len as f64 * *w as f64).sum()
            }
        }
    }

    pub(super) fn play_pose(&self, l: &LayerRt, play: Play) -> Pose {
        let st = &l.states[play.state];
        let phase = sample_phase(st.mode, play.pos);
        match &st.kind {
            RtKind::Leaf(leaf) => MachineRuntime::leaf_pose(leaf, phase),
            RtKind::Blend { input, thresholds, children } => {
                let w = self.weights_of(thresholds, *input);
                let poses: Vec<Pose> = children.iter().zip(&w).map(|(c, w)| if *w > 0.0 { MachineRuntime::leaf_pose(c, phase) } else { Pose::default() }).collect();
                let parts: Vec<(&Pose, f32)> = poses.iter().zip(&w).map(|(p, w)| (p, *w)).collect();
                Pose::mix(&parts)
            }
        }
    }

    pub(super) fn layer_pose(&self, li: usize) -> Pose {
        let l = &self.layers[li];
        let cur = self.play_pose(l, l.cur);
        match &l.fade {
            None => cur,
            Some(f) => {
                let from = match &f.from {
                    FadeFrom::Live(p) => self.play_pose(l, *p),
                    FadeFrom::Snap(s) => s.clone(),
                };
                let u = if f.dur > 0.0 { f.ease.apply((f.elapsed / f.dur) as f32).clamp(0.0, 1.0) } else { 1.0 };
                Pose::mix(&[(&from, 1.0 - u), (&cur, u)])
            }
        }
    }

    /// Pose of all layers together.
    pub fn pose(&self) -> Pose {
        let mut out = Pose::default();
        for li in 0..self.layers.len() {
            for (t, p, v, w) in self.layer_pose(li).items {
                out.add(&t, p, v, w);
            }
        }
        out
    }

    /// Channels the runtime adds to the project: machine pose, then data-binding properties.
    pub fn pose_channels(&self) -> Vec<Channel> {
        let mut c = self.pose().channels();
        c.extend(self.bindings.channels());
        c
    }

    /// Project (without machines / bindings) with the current pose channels appended.
    pub fn project_now(&self) -> Project {
        let mut p = self.base.clone();
        p.channels.extend(self.pose_channels());
        p
    }

    /// Engine for the current state; evaluate it at [`MachineRuntime::base_frame`] (or render with
    /// the SVG / raster renderers).
    pub fn engine(&self) -> Result<Engine, String> {
        Engine::with_asset_dir(self.project_now(), &self.asset_dir).map_err(|e| e.join("; "))
    }

    /// Like [`MachineRuntime::engine`] but `edit` may change the project copy first (used by the
    /// facade to apply a canvas size and scroll offsets without touching the machine state).
    pub fn engine_with(&self, edit: impl FnOnce(&mut Project)) -> Result<Engine, String> {
        let mut p = self.project_now();
        edit(&mut p);
        Engine::with_asset_dir(p, &self.asset_dir).map_err(|e| e.join("; "))
    }

    pub fn base_frame(&self) -> f32 {
        self.base_frame
    }

    /// Evaluated frame at the machine's current local times.
    pub fn frame(&self) -> Result<Frame, String> {
        Ok(self.engine()?.evaluate(self.base_frame))
    }

    // ---- stepping ----

    /// Advance by `dt` seconds (0 <= dt <= 10). Bindings are applied first, then time passes in
    /// substeps of at most 1/60 s with transitions checked after each.
    pub fn step(&mut self, dt: f64) -> Result<(), String> {
        if !(dt.is_finite() && (0.0..=MAX_STEP).contains(&dt)) {
            return Err(format!("dt must be within 0..={MAX_STEP} seconds"));
        }
        self.bindings.update(dt);
        for (m, input, v, edge) in self.bindings.input_outputs() {
            if m != self.machine_name {
                continue;
            }
            if let Ok(i) = self.input_ix(&input) {
                match self.in_kind[i] {
                    InputKind::Bool => self.values[i] = if v != 0.0 { 1.0 } else { 0.0 },
                    InputKind::Number => self.values[i] = v,
                    InputKind::Trigger => {
                        if edge {
                            self.values[i] = 1.0;
                        }
                    }
                }
            }
        }
        let n = if dt == 0.0 { 1 } else { (dt / SUBSTEP).ceil() as usize };
        let h = dt / n as f64;
        for _ in 0..n {
            self.substep(h);
        }
        for i in 0..self.values.len() {
            if self.in_kind[i] == InputKind::Trigger && !self.in_hold[i] {
                self.values[i] = 0.0;
            }
        }
        Ok(())
    }

    pub(super) fn advance(&self, l: &LayerRt, play: &mut Play, h: f64) {
        let st = &l.states[play.state];
        let len = self.state_len(st);
        if len > 0.0 {
            play.pos += h * st.speed * self.fps as f64 / len;
        }
    }

    pub(super) fn substep(&mut self, h: f64) {
        self.time += h;
        for li in 0..self.layers.len() {
            // advance time
            let mut cur = self.layers[li].cur;
            self.advance(&self.layers[li], &mut cur, h);
            let mut fade_done = false;
            let mut live = None;
            if let Some(f) = &self.layers[li].fade {
                if let FadeFrom::Live(p) = f.from {
                    let mut p = p;
                    self.advance(&self.layers[li], &mut p, h);
                    live = Some(p);
                }
                fade_done = f.elapsed + h >= f.dur;
            }
            let l = &mut self.layers[li];
            l.cur = cur;
            if let Some(f) = &mut l.fade {
                f.elapsed += h;
                if let Some(p) = live {
                    f.from = FadeFrom::Live(p);
                }
            }
            if fade_done {
                l.fade = None;
            }
            // transitions (zero-duration ones may chain)
            for _ in 0..MAX_HOPS {
                let Some(ti) = self.find_transition(li) else { break };
                let instant = self.fire_transition(li, ti);
                if !instant {
                    break;
                }
            }
        }
    }

    pub(super) fn find_transition(&self, li: usize) -> Option<usize> {
        let l = &self.layers[li];
        let st = &l.states[l.cur.state];
        let phase = phase_of(st.mode, l.cur.pos);
        'next: for (i, t) in l.trans.iter().enumerate() {
            match t.from {
                Some(f) if f != l.cur.state => continue,
                None if t.to == l.cur.state => continue,
                _ => {}
            }
            if let Some(x) = t.exit {
                if phase < x as f64 - 1e-9 {
                    continue;
                }
            }
            for c in &t.conds {
                if !c.op.test(self.values[c.input], c.value) {
                    continue 'next;
                }
            }
            return Some(i);
        }
        None
    }

    /// Returns whether the transition was instantaneous.
    pub(super) fn fire_transition(&mut self, li: usize, ti: usize) -> bool {
        let snap = if self.layers[li].fade.is_some() { Some(self.layer_pose(li)) } else { None };
        let consumed: Vec<usize> = self.layers[li].trans[ti].conds.iter().map(|c| c.input).filter(|&i| self.in_kind[i] == InputKind::Trigger).collect();
        for i in consumed {
            self.values[i] = 0.0;
        }
        let time = self.time;
        let l = &mut self.layers[li];
        let (to, dur, ease) = (l.trans[ti].to, l.trans[ti].dur, l.trans[ti].ease);
        let old = l.cur;
        self.events.push(MachineEvent { time, layer: l.name.clone(), kind: EventKind::Exited, state: l.states[old.state].name.clone() });
        self.events.push(MachineEvent { time, layer: l.name.clone(), kind: EventKind::Entered, state: l.states[to].name.clone() });
        l.cur = Play { state: to, pos: 0.0 };
        if dur > 0.0 {
            l.fade = Some(Fade { from: snap.map_or(FadeFrom::Live(old), FadeFrom::Snap), elapsed: 0.0, dur, ease });
            false
        } else {
            l.fade = None;
            true
        }
    }
}
