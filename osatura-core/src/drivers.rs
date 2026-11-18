// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Pose drivers: one control value blends between stored poses.
//!
//! * [`slider_driver`] — a 1-D slider between N poses (e.g. head turn −1 … +1, expression 0…1).
//! * [`joystick_driver`] — a 2-D control from up to four axis poses (left, right, up, down);
//!   diagonals are the sum of the two axes.
//! * [`view_swap`] — pick a child of a switch layer from a driver value (front / ¾ / side by yaw).
//!
//! Each driver is an ordinary controller bone plus ordinary driven channels, so the result is
//! plain, editable project data: animate the controller's offset to animate the driver.

use crate::anim::{Ease, Track};
use crate::math::Vec2;
use crate::project::{Bone, Channel, LayerKind, Mode, Project, Prop, Source};
use std::collections::BTreeMap;

/// One stored pose: absolute channel values `(target, property, value)` reached at `at`.
#[derive(Clone, Debug, PartialEq)]
pub struct PoseEntry {
    pub at: f32,
    pub values: Vec<(String, Prop, f32)>,
}

fn controller_name(p: &Project, base: &str) -> Result<String, String> {
    let n = format!("ctrl_{base}");
    if p.bones.iter().any(|b| b.name == n) {
        return Err(format!("a bone named '{n}' already exists"));
    }
    Ok(n)
}

fn add_controller(p: &mut Project, name: &str) {
    p.bones.push(Bone::new(name, None, 10.0, Vec2::ZERO, 0.0));
}

/// Build the per-(target,prop) curves: pose values by slider position; poses that don't set
/// a pair contribute 0 there.
/// `(layer, prop name)` -> `(prop, (frame, value) samples)`.
type CurveMap = BTreeMap<(String, String), (Prop, Vec<(f32, f32)>)>;

fn curves(poses: &[PoseEntry]) -> CurveMap {
    let mut keys: CurveMap = BTreeMap::new();
    for pose in poses {
        for (t, pr, _) in &pose.values {
            keys.entry((t.clone(), format!("{pr:?}"))).or_insert((*pr, Vec::new()));
        }
    }
    for ((t, pn), (pr, list)) in keys.iter_mut() {
        for pose in poses {
            let v = pose.values.iter().find(|(tt, pp, _)| tt == t && format!("{pp:?}") == *pn).map_or(0.0, |x| x.2);
            list.push((pose.at, v));
        }
        let _ = pr;
    }
    keys
}

fn check_poses(poses: &[PoseEntry], min: usize) -> Result<Vec<PoseEntry>, String> {
    if poses.len() < min {
        return Err(format!("need at least {min} poses"));
    }
    let mut v = poses.to_vec();
    v.sort_by(|a, b| a.at.total_cmp(&b.at));
    if v.windows(2).any(|w| w[0].at == w[1].at) || v.iter().any(|x| !x.at.is_finite()) {
        return Err("pose positions must be finite and distinct".into());
    }
    Ok(v)
}

/// Add controller bone `ctrl_<name>` and one additive driven channel per affected value.
/// The slider is the controller's `BoneOffsetX`. Returns the controller bone's name.
pub fn slider_driver(p: &mut Project, name: &str, poses: &[PoseEntry]) -> Result<String, String> {
    let poses = check_poses(poses, 2)?;
    let ctrl = controller_name(p, name)?;
    add_controller(p, &ctrl);
    for ((target, _), (prop, list)) in curves(&poses) {
        let frames: Vec<(f32, f32, Ease)> = list.iter().map(|&(a, v)| (a, v, Ease::Linear)).collect();
        p.channels.push(Channel {
            target,
            prop,
            source: Source::Driven { input: ctrl.clone(), input_prop: Prop::BoneOffsetX, curve: Track::with_keys(&frames) },
            mode: Mode::Add,
            weight: 1.0,
        });
    }
    Ok(ctrl)
}

#[derive(Clone, Debug, Default)]
pub struct JoystickPoses {
    pub left: Vec<(String, Prop, f32)>,
    pub right: Vec<(String, Prop, f32)>,
    pub up: Vec<(String, Prop, f32)>,
    pub down: Vec<(String, Prop, f32)>,
}

/// 2-D controller: `BoneOffsetX` ∈ [−1, 1] blends left/right poses, `BoneOffsetY` up/down
/// (up = −1 following the stage's downward y axis, down = +1). Axes are additive.
pub fn joystick_driver(p: &mut Project, name: &str, j: &JoystickPoses) -> Result<String, String> {
    if [&j.left, &j.right, &j.up, &j.down].iter().all(|v| v.is_empty()) {
        return Err("a joystick needs at least one axis pose".into());
    }
    let ctrl = controller_name(p, name)?;
    add_controller(p, &ctrl);
    let mut axis = |lo: &Vec<(String, Prop, f32)>, hi: &Vec<(String, Prop, f32)>, input_prop: Prop| {
        let poses = vec![
            PoseEntry { at: -1.0, values: lo.clone() },
            PoseEntry { at: 0.0, values: vec![] },
            PoseEntry { at: 1.0, values: hi.clone() },
        ];
        for ((target, _), (prop, list)) in curves(&poses) {
            let frames: Vec<(f32, f32, Ease)> = list.iter().map(|&(a, v)| (a, v, Ease::Linear)).collect();
            p.channels.push(Channel {
                target,
                prop,
                source: Source::Driven { input: ctrl.clone(), input_prop, curve: Track::with_keys(&frames) },
                mode: Mode::Add,
                weight: 1.0,
            });
        }
    };
    axis(&j.left, &j.right, Prop::BoneOffsetX);
    axis(&j.up, &j.down, Prop::BoneOffsetY);
    Ok(ctrl)
}

/// Switch-layer child by driver value: `ranges` = (from_value, child_index), e.g. yaw angle
/// `[(-180, 0), (-60, 1), (-20, 2), (20, 1), (60, 0)]`. The driver is any bone property.
pub fn view_swap(p: &mut Project, switch_layer: &str, driver_bone: &str, driver_prop: Prop, ranges: &[(f32, usize)]) -> Result<(), String> {
    if ranges.is_empty() {
        return Err("need at least one range".into());
    }
    let layer = p.find_layer(switch_layer).ok_or_else(|| format!("no layer '{switch_layer}'"))?;
    let LayerKind::Selector { layers, .. } = &layer.kind else { return Err(format!("'{switch_layer}' is not a switch layer")) };
    if let Some(&(_, c)) = ranges.iter().find(|&&(_, c)| c >= layers.len()) {
        return Err(format!("child index {c} is out of range (the layer has {})", layers.len()));
    }
    let mut r = ranges.to_vec();
    r.sort_by(|a, b| a.0.total_cmp(&b.0));
    if r.windows(2).any(|w| w[0].0 == w[1].0) {
        return Err("range starts must be distinct".into());
    }
    let frames: Vec<(f32, f32, Ease)> = r.iter().map(|&(v, c)| (v, c as f32, Ease::Step)).collect();
    p.channels.push(Channel {
        target: switch_layer.to_string(),
        prop: Prop::LayerSwitch,
        source: Source::Driven { input: driver_bone.to_string(), input_prop: driver_prop, curve: Track::with_keys(&frames) },
        mode: Mode::Replace,
        weight: 1.0,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::Ease;
    use crate::path::VPath;
    use crate::project::Layer;
    use crate::Engine;

    fn base() -> Project {
        let mut p = Project::new("d", 200.0, 200.0, 24.0, 10);
        p.bones = vec![Bone::new("head", None, 20.0, Vec2::new(100.0, 100.0), 0.0), Bone::new("jaw", Some("head"), 10.0, Vec2::new(10.0, 0.0), 0.0)];
        p
    }

    fn angle(p: &Project, bone: &str) -> f32 {
        let e = Engine::new(p.clone()).unwrap();
        let f = e.evaluate(0.0);
        let i = p.bones.iter().position(|b| b.name == bone).unwrap();
        f.world[i].rotation().to_degrees()
    }

    fn set_slider(p: &mut Project, ctrl: &str, v: f32) {
        p.channels.insert(0, Channel::keys(ctrl, Prop::BoneOffsetX, Track::with_keys(&[(0.0, v, Ease::Linear)])));
    }

    #[test]
    fn slider_blends_between_poses() {
        let mut p = base();
        let poses = vec![
            PoseEntry { at: -1.0, values: vec![("head".into(), Prop::BoneAngle, -30.0)] },
            PoseEntry { at: 0.0, values: vec![] },
            PoseEntry { at: 1.0, values: vec![("head".into(), Prop::BoneAngle, 30.0), ("jaw".into(), Prop::BoneAngle, 20.0)] },
        ];
        let ctrl = slider_driver(&mut p, "turn", &poses).unwrap();
        assert_eq!(ctrl, "ctrl_turn");
        assert!(slider_driver(&mut p, "turn", &poses).is_err(), "name clash");
        for (v, head, jaw) in [(-1.0, -30.0, 0.0), (0.0, 0.0, 0.0), (0.5, 15.0, 10.0), (1.0, 30.0, 20.0)] {
            let mut q = p.clone();
            set_slider(&mut q, &ctrl, v);
            assert!((angle(&q, "head") - head).abs() < 0.1, "v={v}");
            assert!((angle(&q, "jaw") - (head + jaw)).abs() < 0.1, "v={v}: jaw is a child of head");
        }
    }

    #[test]
    fn slider_input_validation() {
        let mut p = base();
        assert!(slider_driver(&mut p, "x", &[PoseEntry { at: 0.0, values: vec![] }]).is_err());
        let dup = vec![PoseEntry { at: 0.0, values: vec![] }, PoseEntry { at: 0.0, values: vec![] }];
        assert!(slider_driver(&mut p, "x", &dup).is_err());
        assert_eq!(p.bones.len(), 2, "failed calls leave the project untouched");
    }

    #[test]
    fn joystick_axes_add_up() {
        let mut p = base();
        let j = JoystickPoses {
            left: vec![("head".into(), Prop::BoneAngle, -20.0)],
            right: vec![("head".into(), Prop::BoneAngle, 20.0)],
            up: vec![("jaw".into(), Prop::BoneAngle, -10.0)],
            down: vec![("jaw".into(), Prop::BoneAngle, 10.0)],
        };
        let ctrl = joystick_driver(&mut p, "look", &j).unwrap();
        let mut q = p.clone();
        q.channels.insert(0, Channel::keys(&ctrl, Prop::BoneOffsetX, Track::with_keys(&[(0.0, 1.0, Ease::Linear)])));
        q.channels.insert(1, Channel::keys(&ctrl, Prop::BoneOffsetY, Track::with_keys(&[(0.0, -1.0, Ease::Linear)])));
        assert!((angle(&q, "head") - 20.0).abs() < 0.1);
        assert!((angle(&q, "jaw") - (20.0 - 10.0)).abs() < 0.1, "diagonal = both axes");
        assert!(joystick_driver(&mut p, "empty", &JoystickPoses::default()).is_err());
    }

    #[test]
    fn view_swap_selects_children_by_angle() {
        let mut p = base();
        let kids: Vec<Layer> = (0..3).map(|i| Layer::vector(&format!("v{i}"), vec![VPath::rect("r", 0.0, 0.0, 10.0 + i as f32, 10.0).with_fill("#336699")])).collect();
        p.layers = vec![Layer::switch("view", kids)];
        view_swap(&mut p, "view", "head", Prop::BoneAngle, &[(-180.0, 2), (-30.0, 1), (-10.0, 0), (10.0, 1), (30.0, 2)]).unwrap();
        let pick = |deg: f32| {
            let mut q = p.clone();
            q.channels.insert(0, Channel::keys("head", Prop::BoneAngle, Track::with_keys(&[(0.0, deg, Ease::Linear)])));
            let e = Engine::new(q).unwrap();
            e.evaluate(0.0).nodes.iter().filter(|n| format!("{n:?}").contains("v0")).count()
        };
        // only structural checks here: the engine accepts it and the channel is well-formed
        let _ = pick(0.0);
        let Source::Driven { curve, .. } = &p.channels.last().unwrap().source else { panic!() };
        assert_eq!(curve.eval(-5.0), Some(0.0));
        assert_eq!(curve.eval(20.0), Some(1.0));
        assert_eq!(curve.eval(45.0), Some(2.0));
        assert_eq!(curve.eval(-100.0), Some(2.0));
        assert!(view_swap(&mut p, "view", "head", Prop::BoneAngle, &[(0.0, 9)]).is_err(), "child out of range");
        assert!(view_swap(&mut p, "nope", "head", Prop::BoneAngle, &[(0.0, 0)]).is_err());
        assert!(view_swap(&mut p, "view", "ghost", Prop::BoneAngle, &[(0.0, 0)]).is_ok(), "bad driver is caught at compile time instead");
        assert!(Engine::new(p).is_err());
    }
}
