// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Static analysis and quality tools for rigs and animation:
//! a rig **linter**, **motion contracts** ("unit tests for animation"), a **seamless-loop
//! solver**, and a **principles coach** that suggests timing improvements.
//!
//! All four are original designs built on generic, textbook ideas (static checks, assertions,
//! cubic-Hermite end-conditions, animation-principles heuristics).

use crate::anim::{Ease, Key};
use crate::engine::Engine;
pub use crate::looper::LoopMode;
use crate::math::Vec2;
use crate::project::{Channel, Project, Prop, Source};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ================================================================================================
// Linter
// ================================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Finding {
    pub severity: Severity,
    pub code: &'static str,
    pub target: String,
    pub message: String,
}

fn find(sev: Severity, code: &'static str, target: &str, message: String) -> Finding {
    Finding { severity: sev, code, target: target.to_string(), message }
}

fn keyed(c: &Channel) -> Option<&crate::anim::Track> {
    match &c.source {
        Source::Keys { track } => Some(track),
        _ => None,
    }
}

/// Gradients whose stops are not in ascending offset order (every renderer sorts them, stable, so the
/// picture is the same everywhere, but the file is probably not what its author meant) and
/// gradients without extent (painted as their last stop). `(layer path, message)`.
pub fn gradient_warnings(project: &Project) -> Vec<(String, String)> {
    use crate::project::LayerKind;
    use crate::style::Paint;
    fn check(name: &str, what: &str, p: &Paint, out: &mut Vec<(String, String)>) {
        let stops = match p {
            Paint::Solid(_) => return,
            Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => stops,
        };
        if stops.windows(2).any(|w| w[1].offset < w[0].offset) {
            out.push((name.to_string(), format!("{what} gradient stops are not in ascending offset order; they are sorted (stable) when drawn")));
        }
        if p.degenerate_color().is_some() {
            out.push((name.to_string(), format!("{what} gradient has no extent (zero-length axis or radius 0); it paints its last stop everywhere")));
        }
    }
    fn walk(prefix: &str, layers: &[crate::project::Layer], out: &mut Vec<(String, String)>) {
        for l in layers {
            let name = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
            match &l.kind {
                LayerKind::Vector { paths } | LayerKind::Boolean { paths, .. } => {
                    for p in paths {
                        if let Some(f) = &p.fill {
                            check(&name, "fill", f, out);
                        }
                        if let Some(s) = &p.stroke {
                            check(&name, "stroke", &s.paint, out);
                        }
                    }
                }
                LayerKind::Metaball { fill, stroke, .. } => {
                    if let Some(f) = fill {
                        check(&name, "fill", f, out);
                    }
                    if let Some(s) = stroke {
                        check(&name, "stroke", &s.paint, out);
                    }
                }
                _ => {}
            }
            walk(&name, l.children(), out);
        }
    }
    let mut out = Vec::new();
    walk("", &project.layers, &mut out);
    out
}

/// Keyed `layer_switch` values that select a child that does not exist. The engine does not hide the switch then: it clamps
/// the rounded value into `0..count-1` (negative shows the first child, `>= count` the last one; a switch without children
/// shows nothing), and the message says so. `(channel, message)`.
pub fn switch_key_warnings(project: &Project) -> Vec<(String, String)> {
    fn count(layers: &[crate::project::Layer], path: &str, name: &str) -> Option<usize> {
        for l in layers {
            let p = if path.is_empty() { l.name.clone() } else { format!("{path}/{}", l.name) };
            if p == name {
                return match &l.kind {
                    crate::project::LayerKind::Selector { layers, .. } => Some(layers.len()),
                    _ => None,
                };
            }
            if let Some(n) = count(l.children(), &p, name) {
                return Some(n);
            }
        }
        None
    }
    let mut out = Vec::new();
    for c in &project.channels {
        if c.prop != Prop::LayerSwitch {
            continue;
        }
        let (Some(t), Some(n)) = (keyed(c), count(&project.layers, "", &c.target)) else { continue };
        if let Some(k) = t.keys.iter().find(|k| k.value.is_finite() && (k.value.round() < 0.0 || k.value.round() >= n as f32)) {
            let shown = if n == 0 {
                "the switch has no children, so nothing is shown".to_string()
            } else if k.value.round() < 0.0 {
                "the engine clamps it to child 0 (the first child is shown)".to_string()
            } else {
                format!("the engine clamps it to child {} (the last child is shown)", n - 1)
            };
            out.push((format!("{}:LayerSwitch", c.target), format!("key at frame {} selects child {} but '{}' has {n} children (valid: 0..{}); {shown}; fix the key if that is not intended", k.frame, k.value.round(), c.target, n.saturating_sub(1))));
        }
    }
    out
}

pub fn lint(project: &Project, asset_dir: Option<&Path>) -> Vec<Finding> {
    let mut out = Vec::new();
    for (chan, msg) in switch_key_warnings(project) {
        out.push(find(Severity::Warn, "L012", &chan, msg));
    }
    for (layer, msg) in gradient_warnings(project) {
        out.push(find(Severity::Warn, "L011", &layer, msg));
    }
    // L013: references, ranges and clamped values in the parts rendering never reads (see `editordata`)
    for (at, msg) in crate::editordata::check(project).warnings {
        out.push(find(Severity::Warn, "L013", &at, msg));
    }
    let engine = match asset_dir {
        Some(d) => Engine::with_asset_dir(project.clone(), d),
        None => Engine::new(project.clone()),
    };
    let engine = match engine {
        Ok(e) => Some(e),
        Err(errs) => {
            for e in errs {
                out.push(find(Severity::Error, "L000", "", format!("project does not validate: {e}")));
            }
            None
        }
    };

    for c in &project.channels {
        let name = format!("{}:{:?}", c.target, c.prop);
        let Some(t) = keyed(c) else { continue };
        if t.keys.iter().any(|k| !k.value.is_finite() || !k.frame.is_finite()) {
            out.push(find(Severity::Error, "L010", &name, "contains a non-finite key".into()));
            continue;
        }
        // L001: big angle jumps between smooth keys
        if c.prop == Prop::BoneAngle {
            for w in t.keys.windows(2) {
                if (w[1].value - w[0].value).abs() > 180.0 && w[0].ease != Ease::Step {
                    out.push(find(
                        Severity::Warn,
                        "L001",
                        &name,
                        format!("rotates {:.0}° between frames {} and {}; interpolation takes the long way — add an in-between key", w[1].value - w[0].value, w[0].frame, w[1].frame),
                    ));
                }
            }
            // L002: keys beyond the bone's limits
            if let Some(b) = project.bones.iter().find(|b| b.name == c.target) {
                if let Some([lo, hi]) = b.angle_limits {
                    if let Some(k) = t.keys.iter().find(|k| k.value < lo - 1e-3 || k.value > hi + 1e-3) {
                        out.push(find(Severity::Warn, "L002", &name, format!("key at frame {} ({:.1}°) is outside the limits [{lo}, {hi}]; it will be clamped", k.frame, k.value)));
                    }
                }
            }
        }
        // L005: constant channel
        if t.keys.len() >= 2 && t.keys.iter().all(|k| (k.value - t.keys[0].value).abs() < 1e-6) {
            out.push(find(Severity::Info, "L005", &name, "all keys have the same value; the channel never changes anything".into()));
        }
        // L007: keys outside the timeline
        if let Some(k) = t.keys.iter().find(|k| k.frame < 0.0 || k.frame > project.frames as f32) {
            out.push(find(Severity::Info, "L007", &name, format!("key at frame {} lies outside the 0..{} timeline", k.frame, project.frames)));
        }
    }

    // L006: duplicate replace channels
    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    for c in &project.channels {
        if c.mode == crate::project::Mode::Replace && c.weight >= 0.999 {
            *seen.entry((c.target.clone(), format!("{:?}", c.prop))).or_insert(0) += 1;
        }
    }
    let mut dups: Vec<_> = seen.into_iter().filter(|(_, n)| *n > 1).collect();
    dups.sort();
    for ((t, p), n) in dups {
        out.push(find(Severity::Warn, "L006", &format!("{t}:{p}"), format!("{n} full-weight Replace channels drive the same value; only the last one has any effect")));
    }

    // springs
    for s in &project.springs {
        if s.damping.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
            out.push(find(Severity::Error, "L004", &s.bone, "spring damping must be > 0".into()));
        } else if s.damping < 0.05 {
            out.push(find(Severity::Warn, "L004", &s.bone, format!("damping {} will ring for a very long time", s.damping)));
        }
        if s.frequency > project.fps * 0.25 {
            out.push(find(Severity::Warn, "L004", &s.bone, format!("frequency {} Hz is above a quarter of the frame rate; the baked simulation may alias", s.frequency)));
        }
    }

    // IK
    for ik in &project.ik {
        if (ik.bend.abs() - 1.0).abs() > 1e-3 && ik.bones.len() == 2 {
            out.push(find(Severity::Warn, "L008", &ik.name, format!("bend should be +1 or −1, got {}", ik.bend)));
        }
    }
    if let Some(e) = &engine {
        let reach: HashMap<&str, f32> = project
            .ik
            .iter()
            .filter_map(|c| c.bones.iter().map(|b| project.bones.iter().find(|x| &x.name == b).map(|x| x.length)).sum::<Option<f32>>().map(|r| (c.name.as_str(), r)))
            .collect();
        let step = (project.frames / 12).max(1);
        'chains: for ik in &project.ik {
            let (Some(root), Some(tgt)) = (
                project.bones.iter().position(|b| Some(&b.name) == ik.bones.first()),
                project.bones.iter().position(|b| b.name == ik.target),
            ) else {
                continue;
            };
            let mut f = 0;
            while f < project.frames {
                let fr = e.evaluate(f as f32);
                let d = fr.world[root].apply(Vec2::ZERO).distance(fr.world[tgt].apply(Vec2::ZERO));
                if d > reach[ik.name.as_str()] * 1.001 && ik.mix > 0.0 {
                    out.push(find(Severity::Info, "L003", &ik.name, format!("target '{}' is out of reach at frame {f} ({d:.0} px > {:.0} px); the chain will be fully stretched", ik.target, reach[ik.name.as_str()])));
                    continue 'chains;
                }
                f += step;
            }
        }
    }
    out.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.code.cmp(b.code)).then(a.target.cmp(&b.target)));
    out
}

// ================================================================================================
// Motion contracts
// ================================================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Contract {
    /// The pose at the end of the timeline equals the pose at frame 0 (all bone tips, ±`tol` px).
    LoopSeamless { tol: f32 },
    /// A bone tip stays within `[min, max]` on an axis (`"x"` or `"y"`), optionally only in `[from, to]`.
    TipRange {
        bone: String,
        axis: String,
        #[serde(default)]
        min: Option<f32>,
        #[serde(default)]
        max: Option<f32>,
        #[serde(default)]
        from: Option<f32>,
        #[serde(default)]
        to: Option<f32>,
    },
    /// A bone tip never moves faster than `px_per_frame` between consecutive frames.
    MaxSpeed { bone: String, px_per_frame: f32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Violation {
    pub contract: usize,
    pub frame: f32,
    pub message: String,
}

fn tip(e: &Engine, frame: f32, bone: usize) -> Vec2 {
    let f = e.evaluate(frame);
    f.world[bone].apply(Vec2::new(e.project.bones[bone].length, 0.0))
}

pub fn check_contracts(e: &Engine, contracts: &[Contract]) -> Result<Vec<Violation>, String> {
    let p = &e.project;
    let bone_idx = |n: &str| p.bones.iter().position(|b| b.name == n).ok_or_else(|| format!("contract refers to unknown bone '{n}'"));
    let mut out = Vec::new();
    for (i, c) in contracts.iter().enumerate() {
        match c {
            Contract::LoopSeamless { tol } => {
                let (a, b) = (e.evaluate(0.0), e.evaluate(p.frames as f32));
                for (bi, bone) in p.bones.iter().enumerate() {
                    let pa = a.world[bi].apply(Vec2::new(bone.length, 0.0));
                    let pb = b.world[bi].apply(Vec2::new(bone.length, 0.0));
                    let d = pa.distance(pb);
                    if d > *tol {
                        out.push(Violation { contract: i, frame: p.frames as f32, message: format!("bone '{}' ends {d:.1} px away from where it started (tolerance {tol})", bone.name) });
                    }
                }
            }
            Contract::TipRange { bone, axis, min, max, from, to } => {
                let bi = bone_idx(bone)?;
                if axis != "x" && axis != "y" {
                    return Err(format!("contract {i}: axis must be \"x\" or \"y\""));
                }
                let (f0, f1) = (from.unwrap_or(0.0), to.unwrap_or(p.frames as f32 - 1.0));
                let mut f = f0.ceil();
                while f <= f1 {
                    let t = tip(e, f, bi);
                    let v = if axis == "x" { t.x } else { t.y };
                    if min.is_some_and(|m| v < m - 1e-3) || max.is_some_and(|m| v > m + 1e-3) {
                        out.push(Violation { contract: i, frame: f, message: format!("tip of '{bone}' has {axis} = {v:.1}, outside [{}, {}]", fmt_opt(*min), fmt_opt(*max)) });
                        break; // one report per contract keeps output readable
                    }
                    f += 1.0;
                }
            }
            Contract::MaxSpeed { bone, px_per_frame } => {
                let bi = bone_idx(bone)?;
                let mut prev = tip(e, 0.0, bi);
                for f in 1..p.frames {
                    let t = tip(e, f as f32, bi);
                    let d = t.distance(prev);
                    if d > *px_per_frame {
                        out.push(Violation { contract: i, frame: f as f32, message: format!("tip of '{bone}' moves {d:.1} px/frame (limit {px_per_frame})") });
                        break;
                    }
                    prev = t;
                }
            }
        }
    }
    Ok(out)
}

fn fmt_opt(v: Option<f32>) -> String {
    v.map_or("−∞/∞".into(), |x| format!("{x}"))
}

// ================================================================================================
// Seamless-loop solver
// ================================================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct LoopReport {
    pub channels_adjusted: usize,
    /// Largest |last − first| key value over all keyed channels, before and after.
    pub seam_before: f32,
    pub seam_after: f32,
}

fn seam(project: &Project) -> f32 {
    project.channels.iter().filter_map(keyed).filter(|t| t.keys.len() >= 2).map(|t| (t.keys.last().unwrap().value - t.keys[0].value).abs()).fold(0.0, f32::max)
}

/// Make every keyed channel end where it begins, in position *and* (approximately) velocity,
/// by adding a cubic-Hermite correction over the last `window` frames of each track.
/// Keys before the window are untouched; the correction has zero value and zero slope at the
/// window start, so the animation inside is only gently bent.
///
/// Seam semantics (C5): by default ([`LoopMode::Exclusive`]) the loop period is the project length
/// `N`: a closing key sitting on the last played frame `N-1` is moved to frame `N` (the start of the
/// next cycle) and the track cycles, so frame `N-1` is one step before frame 0 instead of a duplicate
/// of it. [`LoopMode::Inclusive`] keeps the closing key on its frame (frame `N-1` == frame 0).
pub fn solve_loop(project: &mut Project, window: f32) -> Result<LoopReport, String> {
    solve_loop_mode(project, window, LoopMode::Exclusive)
}

pub fn solve_loop_mode(project: &mut Project, window: f32, mode: LoopMode) -> Result<LoopReport, String> {
    if !matches!(window.partial_cmp(&2.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) {
        return Err("window must be at least 2 frames".into());
    }
    let before = seam(project);
    let mut adjusted = 0;
    let n_frames = project.frames as f32;
    for c in &mut project.channels {
        let Source::Keys { track } = &mut c.source else { continue };
        let n = track.keys.len();
        if n < 3 {
            continue;
        }
        if mode == LoopMode::Exclusive && (track.keys[n - 1].frame - (n_frames - 1.0)).abs() < 1e-3 {
            track.keys[n - 1].frame = n_frames;
        }
        let (f_first, f_last) = (track.keys[0].frame, track.keys[n - 1].frame);
        let w = window.min(f_last - f_first);
        if w.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
            continue;
        }
        if mode == LoopMode::Exclusive && (f_last - f_first - n_frames).abs() < 1e-3 {
            track.extrapolate = crate::anim::Extrapolate::Loop;
        }
        let dv = track.keys[0].value - track.keys[n - 1].value;
        let slope_start = (track.keys[1].value - track.keys[0].value) / (track.keys[1].frame - track.keys[0].frame).max(1e-6);
        let slope_end = (track.keys[n - 1].value - track.keys[n - 2].value) / (track.keys[n - 1].frame - track.keys[n - 2].frame).max(1e-6);
        let ds = slope_start - slope_end;
        if dv.abs() < 1e-6 && ds.abs() < 1e-6 {
            continue;
        }
        let w0 = f_last - w;
        for k in track.keys.iter_mut().filter(|k| k.frame > w0) {
            let u = ((k.frame - w0) / w).clamp(0.0, 1.0);
            // c(0)=0, c'(0)=0, c(1)=dv, c'(1)=ds (per frame)
            k.value += dv * (3.0 * u * u - 2.0 * u * u * u) + ds * w * (u * u * u - u * u);
        }
        // the final key must close the loop exactly
        let first = track.keys[0].value;
        track.keys[n - 1].value = first;
        adjusted += 1;
    }
    Ok(LoopReport { channels_adjusted: adjusted, seam_before: before, seam_after: seam(project) })
}

// ================================================================================================
// Principles coach
// ================================================================================================

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Advice {
    pub code: &'static str,
    pub target: String,
    pub frame: f32,
    pub message: String,
}

fn moving_start(keys: &[Key]) -> Option<f32> {
    keys.windows(2).find(|w| (w[1].value - w[0].value).abs() > 1e-4).map(|w| w[0].frame)
}

/// Non-destructive timing review. Heuristics only: every item is a suggestion, never an edit.
pub fn coach(project: &Project) -> Vec<Advice> {
    let mut out = Vec::new();
    let mut starts: HashMap<i64, Vec<String>> = HashMap::new();
    for c in &project.channels {
        let Some(t) = keyed(c) else { continue };
        let name = format!("{}:{:?}", c.target, c.prop);
        let k = &t.keys;
        if k.len() < 2 {
            continue;
        }
        // C001 robotic constant velocity
        if k.len() >= 3 {
            let slopes: Vec<f32> = k.windows(2).map(|w| (w[1].value - w[0].value) / (w[1].frame - w[0].frame).max(1e-6)).collect();
            let all_linear = k[..k.len() - 1].iter().all(|x| x.ease == Ease::Linear);
            let s0 = slopes[0];
            if all_linear && s0.abs() > 1e-4 && slopes.iter().all(|s| (s - s0).abs() <= 0.05 * s0.abs()) {
                out.push(Advice { code: "C001", target: name.clone(), frame: k[0].frame, message: "constant velocity throughout — looks mechanical; add ease in/out or an extra key".into() });
            }
        }
        // C002 / C005 big moves
        let big = match c.prop {
            Prop::BoneAngle | Prop::LayerRotation => 25.0,
            Prop::BoneOffsetX | Prop::BoneOffsetY | Prop::LayerX | Prop::LayerY => 40.0,
            _ => f32::INFINITY,
        };
        for (i, w) in k.windows(2).enumerate() {
            let d = (w[1].value - w[0].value).abs();
            let frames = (w[1].frame - w[0].frame).max(1e-6);
            if d >= big && frames < 2.0 && w[0].ease != Ease::Step {
                out.push(Advice { code: "C005", target: name.clone(), frame: w[0].frame, message: format!("jumps {d:.0} in {frames:.0} frame(s) — reads as a pop; add in-betweens or use a hold") });
                break;
            }
            let has_overshoot = k.get(i + 2).is_some_and(|n| (n.value - w[1].value) * (w[1].value - w[0].value) < 0.0);
            let eased_with_character = matches!(w[0].ease, Ease::Back { .. } | Ease::Spring { .. } | Ease::Bounce);
            let has_anticipation = i > 0 && (w[0].value - k[i - 1].value) * (w[1].value - w[0].value) < 0.0;
            if d >= big * 1.5 && !has_overshoot && !eased_with_character && !has_anticipation {
                out.push(Advice { code: "C002", target: name.clone(), frame: w[0].frame, message: format!("large move ({d:.0}) with no anticipation or overshoot — try a Back/Spring ease or a small counter-move before it") });
                break;
            }
        }
        if let Some(s) = moving_start(k) {
            starts.entry(s.round() as i64).or_default().push(c.target.clone());
        }
    }
    // C003 simultaneous starts
    let mut groups: Vec<_> = starts.into_iter().collect();
    groups.sort();
    for (f, mut targets) in groups {
        targets.sort();
        targets.dedup();
        if targets.len() >= 4 {
            out.push(Advice { code: "C003", target: targets.join(", "), frame: f as f32, message: format!("{} parts start moving on frame {f}; offset them by a few frames (overlap) for a more natural feel", targets.len()) });
        }
    }
    out.sort_by(|a, b| a.code.cmp(b.code).then(a.target.cmp(&b.target)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::Track;
    use crate::demo;
    use crate::project::{Bone, IkChain, Spring};

    fn arm_project() -> Project {
        let mut p = Project::new("t", 200.0, 200.0, 24.0, 48);
        p.bones = vec![
            Bone::new("root", None, 10.0, Vec2::new(100.0, 100.0), 0.0),
            Bone::new("upper", Some("root"), 30.0, Vec2::ZERO, 0.0),
            Bone::new("lower", Some("upper"), 30.0, Vec2::new(30.0, 0.0), 0.0),
            Bone::new("goal", Some("root"), 5.0, Vec2::new(50.0, 0.0), 0.0),
        ];
        p
    }

    #[test]
    fn linter_flags_each_problem_once() {
        let mut p = arm_project();
        p.bones[1].angle_limits = Some([-45.0, 45.0]);
        p.channels = vec![
            Channel::keys("upper", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 250.0, Ease::Linear)])),
            Channel::keys("lower", Prop::BoneAngle, Track::with_keys(&[(0.0, 5.0, Ease::Linear), (10.0, 5.0, Ease::Linear)])),
            Channel::keys("lower", Prop::BoneAngle, Track::with_keys(&[(0.0, 1.0, Ease::Linear), (10.0, 2.0, Ease::Linear)])),
        ];
        p.springs = vec![Spring { bone: "lower".into(), frequency: 20.0, damping: 0.01, warmup: true }];
        p.ik = vec![IkChain { name: "arm".into(), bones: vec!["upper".into(), "lower".into()], target: "goal".into(), bend: 0.3, mix: 1.0, iterations: 8, soft: 0.0, stretch: 1.0 }];
        p.bones[3].rest_offset = Vec2::new(200.0, 0.0); // far beyond the 60 px reach
        let f = lint(&p, None);
        let codes: Vec<&str> = f.iter().map(|x| x.code).collect();
        for want in ["L001", "L002", "L004", "L005", "L006", "L008", "L003"] {
            assert!(codes.contains(&want), "missing {want} in {codes:?}");
        }
        assert!(f.windows(2).all(|w| w[0].severity >= w[1].severity), "sorted by severity");
    }

    #[test]
    fn linter_reports_invalid_projects_and_accepts_the_demo() {
        let mut bad = arm_project();
        bad.channels = vec![Channel::keys("ghost", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear)]))];
        assert!(lint(&bad, None).iter().any(|f| f.code == "L000" && f.severity == Severity::Error));
        let f = lint(&demo::project(), None);
        assert!(!f.iter().any(|x| x.severity == Severity::Error), "{f:?}");
    }

    #[test]
    fn contracts_pass_and_fail_precisely() {
        let mut p = arm_project();
        p.channels = vec![Channel::keys("upper", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (24.0, 60.0, Ease::Linear), (47.0, 0.0, Ease::Linear)]))];
        let e = Engine::new(p).unwrap();
        let ok = check_contracts(&e, &[Contract::LoopSeamless { tol: 1.0 }, Contract::TipRange { bone: "lower".into(), axis: "y".into(), min: Some(0.0), max: Some(500.0), from: None, to: None }]).unwrap();
        assert!(ok.is_empty(), "{ok:?}");
        let bad = check_contracts(
            &e,
            &[
                Contract::TipRange { bone: "lower".into(), axis: "y".into(), min: None, max: Some(100.0), from: Some(0.0), to: Some(47.0) },
                Contract::MaxSpeed { bone: "lower".into(), px_per_frame: 0.5 },
            ],
        )
        .unwrap();
        assert_eq!(bad.len(), 2, "{bad:?}");
        assert!(check_contracts(&e, &[Contract::MaxSpeed { bone: "nope".into(), px_per_frame: 1.0 }]).is_err());
        assert!(check_contracts(&e, &[Contract::TipRange { bone: "lower".into(), axis: "z".into(), min: None, max: None, from: None, to: None }]).is_err());
    }

    #[test]
    fn seamless_contract_detects_pops_and_solver_fixes_them() {
        let mut p = arm_project();
        p.frames = 49;
        p.channels = vec![Channel::keys("upper", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (16.0, 30.0, Ease::Linear), (32.0, 50.0, Ease::Linear), (48.0, 40.0, Ease::Linear)]))];
        let e = Engine::new(p.clone()).unwrap();
        assert!(!check_contracts(&e, &[Contract::LoopSeamless { tol: 1.0 }]).unwrap().is_empty());
        let r = solve_loop(&mut p, 24.0).unwrap();
        assert_eq!(r.channels_adjusted, 1);
        assert!(r.seam_before > 30.0 && r.seam_after < 1e-4, "{r:?}");
        let Source::Keys { track } = &p.channels[0].source else { panic!() };
        assert_eq!(track.keys.first().unwrap().value, track.keys.last().unwrap().value);
        assert_eq!(track.keys[0].value, 0.0);
        assert_eq!(track.keys[1].value, 30.0, "keys before the window are untouched");
        assert!(solve_loop(&mut p, 1.0).is_err());
    }

    /// C5: the solved loop has period N: the closing key moves to frame N, so frame N-1 is not a
    /// copy of frame 0 and the wrap step is an ordinary step; Inclusive keeps the old behaviour.
    #[test]
    fn solved_loop_has_no_duplicated_seam_frame() {
        let mk = || {
            let mut p = arm_project();
            p.frames = 49;
            p.channels = vec![Channel::keys("upper", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (16.0, 30.0, Ease::Linear), (32.0, 50.0, Ease::Linear), (48.0, 40.0, Ease::Linear)]))];
            p
        };
        let angles = |p: &Project| -> Vec<f32> {
            let e = Engine::new(p.clone()).unwrap();
            (0..p.frames).map(|f| e.evaluate(f as f32).world[1].rotation()).collect()
        };
        let mut p = mk();
        solve_loop(&mut p, 24.0).unwrap();
        let Source::Keys { track } = &p.channels[0].source else { panic!() };
        assert_eq!(track.keys.last().unwrap().frame, 49.0);
        let a = angles(&p);
        let max_step = a.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f32::max);
        let wrap = (a[0] - a[a.len() - 1]).abs();
        assert!(wrap > 1e-3, "seam frame duplicated");
        assert!(wrap <= max_step * 1.01, "wrap {wrap} vs step {max_step}");
        let e = Engine::new(p.clone()).unwrap();
        assert!(check_contracts(&e, &[Contract::LoopSeamless { tol: 0.5 }]).unwrap().is_empty());
        // legacy
        let mut q = mk();
        solve_loop_mode(&mut q, 24.0, LoopMode::Inclusive).unwrap();
        let b = angles(&q);
        assert!((b[0] - b[b.len() - 1]).abs() < 1e-3);
    }

    /// C12d: `check` / lint flag a switch key that selects a missing child.
    #[test]
    fn out_of_range_switch_keys_are_flagged() {
        use crate::project::Layer;
        let mut p = arm_project();
        let kids: Vec<Layer> = (0..3).map(|i| Layer::vector(&format!("m{i}"), vec![crate::path::VPath::rect("r", 0.0, 0.0, 5.0, 5.0)])).collect();
        p.layers = vec![Layer::switch("mouth", kids)];
        p.channels = vec![Channel::keys("mouth", Prop::LayerSwitch, Track::with_keys(&[(0.0, 0.0, Ease::Step), (5.0, 2.0, Ease::Step)]))];
        assert!(switch_key_warnings(&p).is_empty());
        p.channels = vec![Channel::keys("mouth", Prop::LayerSwitch, Track::with_keys(&[(0.0, 0.0, Ease::Step), (5.0, 3.0, Ease::Step)]))];
        let w = switch_key_warnings(&p);
        assert_eq!(w.len(), 1);
        assert!(w[0].1.contains("selects child 3") && w[0].1.contains("3 children"), "{w:?}");
        assert!(w[0].1.contains("clamps it to child 2") && !w[0].1.contains("nothing is shown"), "message must describe the engine's clamping: {w:?}");
        assert!(lint(&p, None).iter().any(|f| f.code == "L012"));
        p.channels = vec![Channel::keys("mouth", Prop::LayerSwitch, Track::with_keys(&[(0.0, -1.0, Ease::Step)]))];
        let w = switch_key_warnings(&p);
        assert_eq!(w.len(), 1);
        assert!(w[0].1.contains("clamps it to child 0"), "{w:?}");
        // and the engine really does what the message says (negative -> child 0, >= count -> last child)
        let shown = |p: &Project, f: f32| {
            let fr = crate::Engine::new(p.clone()).unwrap().evaluate(f);
            let sw = fr.nodes.iter().find(|n| n.kind == crate::engine::NodeKind::Switch).unwrap();
            assert_eq!(sw.children.len(), 1);
            fr.nodes[sw.children[0]].name.clone()
        };
        assert_eq!(shown(&p, 0.0), "mouth/m0");
        p.channels = vec![Channel::keys("mouth", Prop::LayerSwitch, Track::with_keys(&[(0.0, 7.0, Ease::Step)]))];
        assert_eq!(shown(&p, 0.0), "mouth/m2");
    }

    #[test]
    fn loop_solver_matches_velocity_approximately() {
        let mut p = arm_project();
        // a ramp that ends with a steep slope while it starts flat
        p.channels = vec![Channel::keys("upper", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 0.0, Ease::Linear), (30.0, 20.0, Ease::Linear), (40.0, 60.0, Ease::Linear)]))];
        solve_loop(&mut p, 20.0).unwrap();
        let Source::Keys { track } = &p.channels[0].source else { panic!() };
        let n = track.keys.len();
        let end_slope = (track.keys[n - 1].value - track.keys[n - 2].value) / (track.keys[n - 1].frame - track.keys[n - 2].frame);
        let start_slope = (track.keys[1].value - track.keys[0].value) / (track.keys[1].frame - track.keys[0].frame);
        // the original end slope was 4/frame; after solving, it must be much closer to the start slope (0)
        assert!((end_slope - start_slope).abs() < 2.0, "end {end_slope} vs start {start_slope}");
    }

    #[test]
    fn coach_finds_the_classic_problems() {
        let mut p = arm_project();
        let mut ch = vec![
            // robotic
            Channel::keys("upper", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 10.0, Ease::Linear), (20.0, 20.0, Ease::Linear)])),
            // pop
            Channel::keys("lower", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (1.0, 90.0, Ease::Linear)])),
            // large move, no character
            Channel::keys("goal", Prop::BoneOffsetX, Track::with_keys(&[(0.0, 0.0, Ease::EaseInOut), (30.0, 100.0, Ease::EaseInOut)])),
        ];
        // four channels starting together
        for b in ["root", "upper", "lower", "goal"] {
            ch.push(Channel::keys(b, Prop::BoneScale, Track::with_keys(&[(5.0, 1.0, Ease::EaseInOut), (20.0, 1.2, Ease::EaseInOut)])));
        }
        p.channels = ch;
        let codes: Vec<_> = coach(&p).into_iter().map(|a| a.code).collect();
        for want in ["C001", "C002", "C003", "C005"] {
            assert!(codes.contains(&want), "missing {want} in {codes:?}");
        }
        // a well-animated move gets no advice
        p.channels = vec![Channel::keys("upper", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::EaseInOut), (4.0, -8.0, Ease::Back { overshoot: 1.2 }), (20.0, 40.0, Ease::EaseInOut)]))];
        assert!(coach(&p).is_empty(), "{:?}", coach(&p));
    }
}
