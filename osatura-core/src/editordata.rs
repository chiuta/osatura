// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Checks of the project parts that rendering never reads: skins, comps, markers, events, shots, channel
//! metadata, bone visibility ranges, the clip stack and LOD records, plus values the renderers silently
//! clamp (camera zoom, layer opacity, `ik` mix, constraint weight).
//!
//! Backward compatibility rule: a saved project that loaded and rendered before must still load. So this module
//! reports **errors** only for numbers that cannot exist in a saved file (NaN or infinity: JSON has no such value,
//! a project built through the API could carry one) and **warnings** for everything else: unknown layer, bone, ik or
//! constraint names, duplicates, negative or inverted time ranges, zero or negative lengths and out-of-range values
//! that the engine clamps. `Engine::new` adds the errors to its own; `osatura check` and `lint` print the warnings.

use crate::project::{LayerKind, Project, Source, TargetKind};
use std::collections::HashSet;

/// Result of [`check`]: `errors` make `Engine::new` fail, `warnings` never do.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub errors: Vec<String>,
    /// `(where, message)`, e.g. `("skin outfit/red", "layer 'x' does not exist")`.
    pub warnings: Vec<(String, String)>,
}

fn layer_paths(layers: &[crate::project::Layer], prefix: &str, out: &mut Vec<(String, f32, bool)>) {
    for l in layers {
        let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
        out.push((path.clone(), l.opacity, matches!(l.kind, LayerKind::Selector { .. })));
        layer_paths(l.children(), &path, out);
    }
}

fn switch_children(layers: &[crate::project::Layer], path: &str) -> Option<usize> {
    let (head, rest) = match path.split_once('/') {
        Some((h, r)) => (h, Some(r)),
        None => (path, None),
    };
    let l = layers.iter().find(|l| l.name == head)?;
    match rest {
        Some(r) => switch_children(l.children(), r),
        None => match &l.kind {
            LayerKind::Selector { layers, .. } => Some(layers.len()),
            _ => None,
        },
    }
}

/// All checks. Cheap (linear in the size of the editor data).
pub fn check(p: &Project) -> Report {
    let mut r = Report::default();
    let mut paths: Vec<(String, f32, bool)> = Vec::new();
    layer_paths(&p.layers, "", &mut paths);
    let layer_set: HashSet<&str> = paths.iter().map(|x| x.0.as_str()).collect();
    let bone_set: HashSet<&str> = p.bones.iter().map(|b| b.name.as_str()).collect();
    let ik_set: HashSet<&str> = p.ik.iter().map(|c| c.name.as_str()).collect();
    let con_set: HashSet<&str> = p.constraints.iter().map(|c| c.name.as_str()).collect();
    let frames = p.frames as f32;
    let warn = |r: &mut Report, at: String, msg: String| r.warnings.push((at, msg));

    // values the renderers clamp silently
    if (p.camera.zoom.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !p.camera.zoom.is_finite() {
        warn(&mut r, "camera".into(), format!("zoom is {}; the renderers clamp a zoom <= 0 to 0.01, so the picture is not what the file says", p.camera.zoom));
    }
    for (path, op, _) in &paths {
        if !op.is_finite() {
            r.errors.push(format!("layer '{path}': opacity is not a finite number"));
        } else if !(0.0..=1.0).contains(op) {
            warn(&mut r, format!("layer {path}"), format!("opacity {op} is outside 0..1; rendering clamps it to {}", op.clamp(0.0, 1.0)));
        }
    }
    for c in &p.ik {
        if c.mix.is_finite() && !(0.0..=1.0).contains(&c.mix) {
            warn(&mut r, format!("ik {}", c.name), format!("mix {} is outside 0..1; it is clamped", c.mix));
        }
    }
    for c in &p.constraints {
        if c.weight.is_finite() && !(0.0..=1.0).contains(&c.weight) {
            warn(&mut r, format!("constraint {}", c.name), format!("weight {} is outside 0..1; it is clamped", c.weight));
        }
    }
    for c in &p.channels {
        if let Source::Keys { track } = &c.source {
            let (lo, hi, what) = match c.prop {
                crate::project::Prop::LayerOpacity => (0.0, 1.0, "layer_opacity"),
                crate::project::Prop::CameraZoom => (f32::MIN_POSITIVE, f32::MAX, "camera_zoom"),
                _ => continue,
            };
            if let Some(k) = track.keys.iter().find(|k| k.value.is_finite() && (k.value < lo || k.value > hi)) {
                let msg = if c.prop == crate::project::Prop::CameraZoom {
                    format!("{what} key at frame {} is {}; a zoom <= 0 is clamped to 0.01", k.frame, k.value)
                } else {
                    format!("{what} key at frame {} is {}; the value is clamped to 0..1 when rendering", k.frame, k.value)
                };
                warn(&mut r, format!("{}:{what}", c.target), msg);
            }
        }
    }

    // time values: finite (error), then range sanity (warning)
    let time = |r: &mut Report, at: String, what: &str, v: f32| -> bool {
        if !v.is_finite() {
            r.errors.push(format!("{at}: {what} is not a finite number"));
            false
        } else {
            true
        }
    };

    // markers
    let mut seen = HashSet::new();
    for (i, m) in p.markers.iter().enumerate() {
        let at = format!("marker #{i} '{}'", m.name);
        let ok_f = time(&mut r, at.clone(), "frame", m.frame);
        let ok_e = m.end.map(|e| time(&mut r, at.clone(), "end", e)).unwrap_or(true);
        if ok_f && ok_e {
            if m.frame < 0.0 || m.frame > frames {
                warn(&mut r, at.clone(), format!("frame {} lies outside the project (0..{})", m.frame, p.frames));
            }
            if let Some(e) = m.end {
                if e < m.frame {
                    warn(&mut r, at.clone(), format!("end {e} is before the start {}", m.frame));
                } else if e == m.frame {
                    warn(&mut r, at.clone(), "region of zero length (end equals frame); use a point marker".into());
                }
            }
        }
        if m.name.trim().is_empty() {
            warn(&mut r, at.clone(), "marker without a name".into());
        }
        if !seen.insert(m.name.as_str()) && !m.name.trim().is_empty() {
            warn(&mut r, at, "duplicate marker name".into());
        }
    }

    // events (several events may share a name: they are cues at different frames)
    for (i, e) in p.events.iter().enumerate() {
        let at = format!("event #{i} '{}'", e.name);
        if time(&mut r, at.clone(), "frame", e.frame) && (e.frame < 0.0 || e.frame > frames) {
            warn(&mut r, at.clone(), format!("frame {} lies outside the project (0..{}); the cue never fires", e.frame, p.frames));
        }
        if e.name.trim().is_empty() {
            warn(&mut r, at, "event without a name".into());
        }
    }

    // comps
    let mut seen = HashSet::new();
    for c in &p.comps {
        let at = format!("comp '{}'", c.name);
        if c.name.trim().is_empty() {
            warn(&mut r, at.clone(), "comp without a name".into());
        }
        if !seen.insert(c.name.as_str()) {
            warn(&mut r, at.clone(), "duplicate comp name; only the first one is found by name".into());
        }
        for (path, st) in &c.layers {
            if !st.opacity.is_finite() {
                r.errors.push(format!("{at}: opacity of '{path}' is not a finite number"));
            } else if !(0.0..=1.0).contains(&st.opacity) {
                warn(&mut r, at.clone(), format!("opacity {} of '{path}' is outside 0..1", st.opacity));
            }
            if !layer_set.contains(path.as_str()) {
                warn(&mut r, at.clone(), format!("layer '{path}' does not exist (the state is ignored when the comp is applied)"));
            } else if let Some(sel) = st.selected {
                match switch_children(&p.layers, path) {
                    Some(n) if sel >= n => warn(&mut r, at.clone(), format!("selected child {sel} of '{path}' does not exist (it has {n})")),
                    None => warn(&mut r, at.clone(), format!("'{path}' is not a switch layer but the comp selects a child")),
                    _ => {}
                }
            }
        }
    }

    // shots
    let mut seen = HashSet::new();
    for s in &p.shots {
        let at = format!("shot '{}'", s.name);
        for (what, v) in [("x", s.x), ("y", s.y), ("zoom", s.zoom), ("rotation", s.rotation)] {
            time(&mut r, at.clone(), what, v);
        }
        if s.zoom.is_finite() && s.zoom <= 0.0 {
            warn(&mut r, at.clone(), format!("zoom {} is not positive; camera-seq refuses it and a camera zoom <= 0 is clamped to 0.01", s.zoom));
        }
        if s.name.trim().is_empty() {
            warn(&mut r, at.clone(), "shot without a name".into());
        }
        if !seen.insert(s.name.as_str()) {
            warn(&mut r, at, "duplicate shot name".into());
        }
    }

    // channel metadata
    let mut seen = HashSet::new();
    for m in &p.channel_meta {
        let at = format!("channel_meta {}:{}", m.target, m.prop.map(|x| format!("{x:?}")).unwrap_or_else(|| "*".into()));
        if !p.channels.iter().any(|c| c.target == m.target && m.prop.is_none_or(|pr| c.prop == pr)) {
            warn(&mut r, at.clone(), "no channel has this target/property, so the flags never apply".into());
        }
        if let Some(col) = &m.color {
            if crate::style::Rgba::parse(col).is_none() {
                warn(&mut r, at.clone(), format!("colour '{col}' cannot be parsed"));
            }
        }
        if !seen.insert((m.target.as_str(), m.prop)) {
            warn(&mut r, at, "duplicate entry for the same target and property".into());
        }
    }

    // bone visibility ranges
    let mut seen = HashSet::new();
    for v in &p.bone_vis {
        let at = format!("bone_vis {}", v.bone);
        let a = time(&mut r, at.clone(), "from", v.from);
        let b = time(&mut r, at.clone(), "to", v.to);
        if !bone_set.contains(v.bone.as_str()) {
            warn(&mut r, at.clone(), "unknown bone".into());
        }
        if a && b {
            if v.to < v.from {
                warn(&mut r, at.clone(), format!("range {}..{} is inverted, the bone is never shown", v.from, v.to));
            } else if v.to < 0.0 || v.from > frames {
                warn(&mut r, at.clone(), format!("range {}..{} lies outside the project (0..{})", v.from, v.to, p.frames));
            }
        }
        if !seen.insert(v.bone.as_str()) {
            warn(&mut r, at, "more than one range for the same bone (only the first applies)".into());
        }
    }

    // skins
    let mut seen = HashSet::new();
    for s in &p.skins {
        let at = format!("skin {}/{}", s.group, s.name);
        if s.group.trim().is_empty() || s.name.trim().is_empty() {
            warn(&mut r, at.clone(), "skin needs a group and a name".into());
        }
        if !seen.insert((s.group.as_str(), s.name.as_str())) {
            warn(&mut r, at.clone(), "duplicate skin (same group and name)".into());
        }
        if s.layers.is_empty() {
            warn(&mut r, at.clone(), "skin without layers".into());
        }
        for l in s.layers.iter().filter(|l| !layer_set.contains(l.as_str())) {
            warn(&mut r, at.clone(), format!("layer '{l}' does not exist"));
        }
        for c in s.constraints.iter().filter(|c| !con_set.contains(c.as_str())) {
            warn(&mut r, at.clone(), format!("constraint '{c}' does not exist"));
        }
        for c in s.iks.iter().filter(|c| !ik_set.contains(c.as_str())) {
            warn(&mut r, at.clone(), format!("ik chain '{c}' does not exist"));
        }
        for b in s.bones.iter().filter(|b| !bone_set.contains(b.as_str())) {
            warn(&mut r, at.clone(), format!("bone '{b}' does not exist"));
        }
    }
    if let Err(m) = crate::skins::validate(p) {
        warn(&mut r, "skins".into(), m);
    }

    // clip stack
    if let Err(m) = crate::clipstack::validate(p) {
        warn(&mut r, "clips".into(), m);
    }
    for c in &p.clips {
        for t in &c.take.tracks {
            let target = c.rename.iter().find(|(from, _)| *from == t.target).map(|(_, to)| to.as_str()).unwrap_or(t.target.as_str());
            let known = match t.prop.target_kind() {
                TargetKind::Bone => bone_set.contains(target),
                TargetKind::Layer => layer_set.contains(target),
                TargetKind::Ik => ik_set.contains(target),
                TargetKind::Constraint => con_set.contains(target),
                TargetKind::Camera | TargetKind::Sub => true,
            };
            if !known {
                warn(&mut r, format!("clip {}", c.name), format!("take track targets '{target}' ({:?}), which does not exist; bake would fail", t.prop));
            }
        }
    }

    // LOD records
    let mut seen = HashSet::new();
    for l in &p.lods {
        let at = format!("lods level {}", l.level);
        for (what, v) in [("tolerance_px", l.tolerance_px), ("scale", l.scale), ("declared_px", l.declared_px), ("bound_stage", l.bound_stage), ("path_px", l.path_px), ("mesh_px", l.mesh_px), ("keys_px", l.keys_px), ("cull_px", l.cull_px)] {
            if !v.is_finite() {
                r.errors.push(format!("{at}: {what} is not a finite number"));
            } else if v < 0.0 {
                warn(&mut r, at.clone(), format!("{what} is negative ({v})"));
            }
        }
        if l.scale.is_finite() && l.scale <= 0.0 {
            warn(&mut r, at.clone(), "scale must be positive".into());
        }
        if !seen.insert(l.level) {
            warn(&mut r, at, "two records for the same level".into());
        }
    }
    r
}

/// The warnings in one line each, `where: message`.
pub fn warning_lines(p: &Project) -> Vec<String> {
    check(p).warnings.into_iter().map(|(a, m)| format!("{a}: {m}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comps::{Comp, Event, LayerState};

    fn base() -> Project {
        crate::demo::project()
    }

    #[test]
    fn demo_and_clean_projects_have_no_findings() {
        let r = check(&base());
        assert!(r.errors.is_empty() && r.warnings.is_empty(), "{r:?}");
    }

    #[test]
    fn camera_zoom_and_opacity_warn_but_still_load_and_clamp() {
        let mut p = base();
        p.camera.zoom = 0.0;
        p.layers[0].opacity = 1.5;
        let r = check(&p);
        assert!(r.errors.is_empty());
        assert!(r.warnings.iter().any(|(a, m)| a == "camera" && m.contains("clamp")), "{r:?}");
        assert!(r.warnings.iter().any(|(_, m)| m.contains("opacity 1.5")), "{r:?}");
        // still loads: previously valid files keep loading
        assert!(crate::Engine::new(p).is_ok());
    }

    #[test]
    fn unknown_references_are_warnings_and_non_finite_numbers_are_errors() {
        let mut p = base();
        p.skins.push(crate::skins::Skin { name: "red".into(), group: "outfit".into(), layers: vec!["nope".into()], constraints: vec!["nc".into()], iks: vec!["ni".into()], bones: vec!["nb".into()] });
        p.comps.push(Comp { name: "c".into(), layers: [("ghost".to_string(), LayerState { visible: true, opacity: 2.0, selected: None })].into() });
        p.events.push(Event { name: "late".into(), frame: 9999.0, data: serde_json::Value::Null });
        p.events.push(Event { name: "early".into(), frame: -3.0, data: serde_json::Value::Null });
        p.markers.push(crate::keyops::Marker { name: "m".into(), frame: 5.0, end: Some(2.0) });
        p.markers.push(crate::keyops::Marker { name: "m".into(), frame: 5.0, end: Some(5.0) });
        p.shots.push(crate::camshots::Shot { name: "z".into(), x: 0.0, y: 0.0, zoom: 0.0, rotation: 0.0 });
        p.bone_vis.push(crate::bonevis::BoneVis { bone: "ghost".into(), from: 5.0, to: 1.0 });
        p.channel_meta.push(crate::chanmeta::ChannelMeta { target: "ghost".into(), prop: None, color: Some("not a colour".into()), ..Default::default() });
        let r = check(&p);
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        let text: String = r.warnings.iter().map(|(a, m)| format!("{a}: {m}\n")).collect();
        for needle in [
            "layer 'nope' does not exist", "constraint 'nc'", "ik chain 'ni'", "bone 'nb'", "layer 'ghost' does not exist", "outside the project",
            "end 2 is before", "zero length", "duplicate marker", "zoom 0", "is inverted", "unknown bone", "no channel has this target", "cannot be parsed",
        ] {
            assert!(text.contains(needle), "missing {needle:?} in\n{text}");
        }
        assert!(crate::Engine::new(p.clone()).is_ok(), "warnings never reject a project");
        // NaN cannot exist in a saved file, so it is an error
        p.events[0].frame = f32::NAN;
        p.markers[0].frame = f32::INFINITY;
        p.shots[0].x = f32::NAN;
        let errs = crate::Engine::new(p).err().expect("non-finite editor data must be rejected");
        let all = errs.join("\n");
        assert!(all.contains("event #0") && all.contains("marker #0") && all.contains("shot 'z'"), "{all}");
    }

    #[test]
    fn clip_and_lod_problems_are_reported() {
        let mut p = base();
        let take = crate::motionlib::Take { name: "t".into(), length: 0.0, tracks: vec![crate::motionlib::TakeTrack { target: "ghost_bone".into(), prop: crate::project::Prop::BoneAngle, keys: vec![] }] };
        p.clips.push(crate::clipstack::Clip { name: "c".into(), take, start: 0.0, speed: 1.0, loops: 1, mode: Default::default(), weight: 1.0, weight_keys: Default::default(), include: vec![], exclude: vec![], track_weights: vec![], fade_in: 0.0, fade_out: 0.0, rename: vec![] });
        let text = warning_lines(&p).join("\n");
        assert!(text.contains("take length must be positive") && text.contains("ghost_bone"), "{text}");
        assert!(crate::Engine::new(p).is_ok());
    }
}
