// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `machine` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// Scripted simulation (used by `machine-sim`, the C ABI and tests)
// ---------------------------------------------------------------------------------------------

/// One timed batch of inputs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScriptEvent {
    /// Seconds from the start.
    pub t: f64,
    /// Input name -> value (bool: 0/1 or true/false; number).
    #[serde(default)]
    pub set: std::collections::BTreeMap<String, serde_json::Value>,
    /// Triggers to fire.
    #[serde(default)]
    pub trigger: Vec<String>,
    /// External value path -> value for data bindings.
    #[serde(default)]
    pub value: std::collections::BTreeMap<String, f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Script {
    /// Machine name (default: the project's only machine).
    #[serde(default)]
    pub machine: Option<String>,
    /// Total length in seconds (default: last event + 1 s).
    #[serde(default)]
    pub duration: Option<f64>,
    pub events: Vec<ScriptEvent>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Tick {
    pub tick: usize,
    pub time: f64,
    pub states: Vec<(String, String)>,
    pub fading: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SimLog {
    pub machine: String,
    pub fps: f32,
    pub ticks: Vec<Tick>,
    pub events: Vec<MachineEvent>,
}

pub const MAX_SIM_TICKS: usize = 1_000_000;

/// Run a script at a fixed rate: tick 0 applies the events with `t <= 0` and steps by 0, tick k
/// applies events with `(k-1)/fps < t <= k/fps` and steps by `1/fps`. `per_tick` sees the runtime
/// after every tick (render frames there).
pub fn simulate(
    project: &Project,
    script: &Script,
    fps: f32,
    asset_dir: &std::path::Path,
    mut per_tick: impl FnMut(usize, &MachineRuntime) -> Result<(), String>,
) -> Result<SimLog, String> {
    if !(fps.is_finite() && fps > 0.0 && fps <= 1000.0) {
        return Err("fps must be within (0, 1000]".into());
    }
    let name = match &script.machine {
        Some(n) => n.clone(),
        None => match project.machines.as_slice() {
            [m] => m.name.clone(),
            [] => return Err("project has no machines".into()),
            _ => return Err("project has several machines: name one in the script".into()),
        },
    };
    let mut rt = MachineRuntime::with_asset_dir(project, &name, asset_dir)?;
    let mut evs = script.events.clone();
    if evs.iter().any(|e| !e.t.is_finite() || e.t < 0.0) {
        return Err("script event times must be finite and >= 0".into());
    }
    evs.sort_by(|a, b| a.t.total_cmp(&b.t));
    let last = evs.last().map_or(0.0, |e| e.t);
    let duration = script.duration.unwrap_or(last + 1.0);
    if !(duration.is_finite() && duration >= 0.0) {
        return Err("duration must be finite and >= 0".into());
    }
    let n = (duration * fps as f64).floor() as usize + 1;
    if n > MAX_SIM_TICKS {
        return Err(format!("simulation would need more than {MAX_SIM_TICKS} ticks"));
    }
    let dt = 1.0 / fps as f64;
    let mut next = 0;
    let mut cur = 0.0f64;
    let mut log = SimLog { machine: name, fps, ticks: vec![], events: vec![] };
    for k in 0..n {
        let t_end = k as f64 * dt;
        while next < evs.len() && evs[next].t <= t_end + 1e-9 {
            let e = &evs[next];
            // Advance to the event's own time first, so an input never acts before it was given
            // (it used to be applied at the start of the tick that contains it).
            let te = e.t.clamp(cur, t_end.max(cur));
            if te > cur {
                rt.step(te - cur)?;
                log.events.extend(rt.take_events());
                cur = te;
            }
            for (n, v) in &e.set {
                let x = match v {
                    serde_json::Value::Bool(b) => *b as u8 as f64,
                    serde_json::Value::Number(x) => x.as_f64().unwrap_or(f64::NAN),
                    _ => return Err(format!("script input '{n}': value must be a number or boolean")),
                };
                rt.set_input(n, x)?;
            }
            for n in &e.trigger {
                rt.fire(n)?;
            }
            for (p, v) in &e.value {
                rt.set_value(p, *v)?;
            }
            next += 1;
        }
        rt.step((t_end - cur).max(0.0))?;
        cur = cur.max(t_end);
        log.events.extend(rt.take_events());
        let fading = rt.layers.iter().filter(|l| l.fade.is_some()).map(|l| l.name.clone()).collect();
        log.ticks.push(Tick { tick: k, time: t_end, states: rt.states(), fading });
        per_tick(k, &rt)?;
    }
    Ok(log)
}

// ---------------------------------------------------------------------------------------------
// Editing helpers
// ---------------------------------------------------------------------------------------------

/// Add a machine (replacing nothing): the project must stay valid.
pub fn add(p: &mut Project, m: Machine) -> Result<(), String> {
    if p.machines.iter().any(|x| x.name == m.name) {
        return Err(format!("machine '{}' already exists", m.name));
    }
    let mut trial = p.clone();
    trial.machines.push(m.clone());
    let errs = validate(&trial);
    if !errs.is_empty() {
        return Err(errs.join("\n"));
    }
    p.machines.push(m);
    Ok(())
}

pub fn remove(p: &mut Project, name: &str) -> Result<(), String> {
    let i = p.machines.iter().position(|m| m.name == name).ok_or_else(|| format!("no machine '{name}'"))?;
    if let Some(b) = p.bindings.iter().find(|b| matches!(&b.target, crate::binding::Target::Input { machine, .. } if machine == name)) {
        return Err(format!("binding '{}' still drives machine '{name}'", b.name));
    }
    p.machines.remove(i);
    Ok(())
}

pub fn add_binding(p: &mut Project, b: crate::binding::Binding) -> Result<(), String> {
    if p.bindings.iter().any(|x| x.name == b.name) {
        return Err(format!("binding '{}' already exists", b.name));
    }
    let mut trial = p.clone();
    trial.bindings.push(b.clone());
    let errs = crate::binding::validate(&trial);
    if !errs.is_empty() {
        return Err(errs.join("\n"));
    }
    let t = trial_errors(&trial, std::path::Path::new("."));
    if !t.is_empty() {
        return Err(t.join("\n"));
    }
    p.bindings.push(b);
    Ok(())
}

pub fn remove_binding(p: &mut Project, name: &str) -> Result<(), String> {
    let i = p.bindings.iter().position(|b| b.name == name).ok_or_else(|| format!("no binding '{name}'"))?;
    p.bindings.remove(i);
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------------
