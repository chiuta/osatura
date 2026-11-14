// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Layer comps (named visibility/opacity/switch states) and timeline events (named cues).

use crate::project::{Layer, LayerKind, Project};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerState {
    pub visible: bool,
    pub opacity: f32,
    /// Selected child of a switch layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Comp {
    pub name: String,
    /// Layer path (`/`-joined) → state.
    pub layers: BTreeMap<String, LayerState>,
}

/// A named cue at a frame, with an optional payload for the host application
/// (for example `{"sound": "step.wav"}`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub frame: f32,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub data: serde_json::Value,
}

fn capture_layers(ls: &[Layer], prefix: &str, out: &mut BTreeMap<String, LayerState>) {
    for l in ls {
        let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
        let selected = match &l.kind {
            LayerKind::Selector { selected, .. } => Some(*selected),
            _ => None,
        };
        out.insert(path.clone(), LayerState { visible: l.visible, opacity: l.opacity, selected });
        capture_layers(l.children(), &path, out);
    }
}

/// Record the current state of every layer under `name` (replacing a comp of that name).
pub fn capture(p: &mut Project, name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("a comp needs a name".into());
    }
    let mut layers = BTreeMap::new();
    capture_layers(&p.layers, "", &mut layers);
    p.comps.retain(|c| c.name != name);
    p.comps.push(Comp { name: name.to_string(), layers });
    Ok(())
}

/// Apply a comp. Layers that no longer exist are skipped and reported.
pub fn apply(p: &mut Project, name: &str) -> Result<Vec<String>, String> {
    let comp = p.comps.iter().find(|c| c.name == name).cloned().ok_or_else(|| format!("no comp named '{name}'"))?;
    let mut missing = Vec::new();
    for (path, st) in &comp.layers {
        match p.find_layer_mut(path) {
            Some(l) => {
                l.visible = st.visible;
                l.opacity = st.opacity;
                if let (LayerKind::Selector { selected, layers }, Some(s)) = (&mut l.kind, st.selected) {
                    *selected = s.min(layers.len().saturating_sub(1));
                }
            }
            None => missing.push(path.clone()),
        }
    }
    Ok(missing)
}

/// Events active on `frame` (within half a frame).
pub fn events_at(events: &[Event], frame: f32) -> Vec<&Event> {
    events.iter().filter(|e| (e.frame - frame).abs() < 0.5).collect()
}

/// Events in `[from, to)` in time order — what a player fires while crossing that range.
pub fn events_between(events: &[Event], from: f32, to: f32) -> Vec<&Event> {
    let mut v: Vec<&Event> = events.iter().filter(|e| e.frame >= from && e.frame < to).collect();
    v.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    #[test]
    fn capture_change_apply_restores() {
        let mut p = demo::project();
        capture(&mut p, "day").unwrap();
        let (v0, o0) = (p.layers[0].visible, p.layers[0].opacity);
        p.layers[0].visible = !v0;
        p.layers[0].opacity = 0.1;
        capture(&mut p, "odd").unwrap();
        assert_eq!(p.comps.len(), 2);
        let missing = apply(&mut p, "day").unwrap();
        assert!(missing.is_empty());
        assert_eq!((p.layers[0].visible, p.layers[0].opacity), (v0, o0));
        apply(&mut p, "odd").unwrap();
        assert_eq!(p.layers[0].opacity, 0.1);
        assert!(apply(&mut p, "nope").is_err());
        assert!(capture(&mut p, " ").is_err());
    }

    #[test]
    fn recapture_replaces_and_missing_layers_are_reported() {
        let mut p = demo::project();
        capture(&mut p, "a").unwrap();
        capture(&mut p, "a").unwrap();
        assert_eq!(p.comps.len(), 1);
        let gone = p.layers.remove(0).name;
        let missing = apply(&mut p, "a").unwrap();
        assert!(missing.contains(&gone));
    }

    #[test]
    fn switch_selection_is_part_of_a_comp() {
        let mut p = demo::project();
        capture(&mut p, "base").unwrap();
        let mouth = p.find_layer_mut("mouth").expect("demo has a mouth switch");
        if let LayerKind::Selector { selected, .. } = &mut mouth.kind {
            *selected = 2;
        }
        apply(&mut p, "base").unwrap();
        if let LayerKind::Selector { selected, .. } = &p.find_layer("mouth").unwrap().kind {
            assert_eq!(*selected, 0);
        }
    }

    #[test]
    fn events_are_found_by_frame_and_range() {
        let ev = vec![
            Event { name: "step".into(), frame: 6.0, data: serde_json::json!({"sound": "step.wav"}) },
            Event { name: "step".into(), frame: 18.0, data: serde_json::Value::Null },
            Event { name: "end".into(), frame: 47.0, data: serde_json::Value::Null },
        ];
        assert_eq!(events_at(&ev, 6.0).len(), 1);
        assert_eq!(events_between(&ev, 0.0, 20.0).len(), 2);
        assert_eq!(events_between(&ev, 7.0, 47.0).len(), 1, "range end is exclusive");
        let j = serde_json::to_string(&ev[1]).unwrap();
        assert!(!j.contains("data"), "null payload omitted");
    }
}
