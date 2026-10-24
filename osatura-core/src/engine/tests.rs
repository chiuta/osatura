// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `engine`.
#![allow(unused_imports)]

use super::*;
use crate::anim::{Ease, Track};
use crate::path::{Anchor, BoneWeight};
use crate::project::{Bone, Channel, IkChain, MaskRef, Transform2D};

fn bone(name: &str, parent: Option<&str>, len: f32, off: Vec2, rest: f32) -> Bone {
    Bone::new(name, parent, len, off, rest)
}

fn empty_project() -> Project {
    Project::new("t", 100.0, 100.0, 24.0, 10)
}

fn tip_of(e: &Engine, f: &Frame, name: &str) -> Vec2 {
    let i = e.project.bones.iter().position(|b| b.name == name).unwrap();
    f.world[i].apply(Vec2::new(e.project.bones[i].length, 0.0))
}

fn arm_with_target(tx: f32, ty: f32) -> Project {
    let mut p = empty_project();
    p.bones = vec![
        bone("a", None, 100.0, Vec2::ZERO, 0.0),
        bone("b", Some("a"), 100.0, Vec2::new(100.0, 0.0), 0.0),
        bone("t", None, 5.0, Vec2::new(tx, ty), 0.0),
    ];
    p.ik = vec![IkChain::two_bone("arm", "a", "b", "t", 1.0)];
    p
}

fn two(a: f32, b: f32) -> Track {
    Track::with_keys(&[(0.0, a, Ease::Linear), (10.0, b, Ease::Linear)])
}

// ---- rigging (carried over) ----

#[test]
fn ik_reaches_reachable_target() {
    let e = Engine::new(arm_with_target(120.0, 80.0)).unwrap();
    let f = e.evaluate(0.0);
    let tip = tip_of(&e, &f, "b");
    assert!(tip.distance(Vec2::new(120.0, 80.0)) < 0.05, "tip at {tip:?}");
    let joint = f.world[1].apply(Vec2::ZERO);
    assert!((joint.length() - 100.0).abs() < 0.05);
}

#[test]
fn ik_clamps_unreachable_target_in_a_straight_line() {
    let e = Engine::new(arm_with_target(0.0, 500.0)).unwrap();
    let f = e.evaluate(0.0);
    let tip = tip_of(&e, &f, "b");
    assert!(tip.distance(Vec2::new(0.0, 200.0)) < 0.5, "tip at {tip:?}");
}

#[test]
fn ik_bend_flips_the_joint_side() {
    let mut p = arm_with_target(120.0, 0.0);
    p.ik[0].bend = -1.0;
    let down = Engine::new(p.clone()).unwrap().evaluate(0.0).world[1].apply(Vec2::ZERO).y;
    p.ik[0].bend = 1.0;
    let up = Engine::new(p).unwrap().evaluate(0.0).world[1].apply(Vec2::ZERO).y;
    assert!(down * up < 0.0, "elbows on opposite sides: {down} vs {up}");
}

#[test]
fn full_weight_anchor_follows_its_bone() {
    let mut p = empty_project();
    p.bones = vec![bone("a", None, 50.0, Vec2::new(10.0, 10.0), 0.0)];
    let mut anchor = Anchor::corner(Vec2::new(60.0, 10.0));
    anchor.weights = vec![BoneWeight { bone: "a".into(), weight: 3.0 }];
    p.layers = vec![Layer::vector("L", vec![VPath::new("p", false, vec![anchor])])];
    p.channels = vec![Channel::keys("a", Prop::BoneAngle, two(0.0, 90.0))];
    let e = Engine::new(p).unwrap();
    let rest = e.evaluate(0.0).nodes[0].paths[0].anchors[0].pos;
    assert!(rest.distance(Vec2::new(60.0, 10.0)) < 1e-3);
    let turned = e.evaluate(10.0).nodes[0].paths[0].anchors[0].pos;
    assert!(turned.distance(Vec2::new(10.0, 60.0)) < 1e-3, "{turned:?}");
}

#[test]
fn auto_skin_binds_to_the_nearest_bone_and_recurses_into_groups() {
    let mut p = empty_project();
    p.bones = vec![
        bone("left", None, 40.0, Vec2::new(0.0, 0.0), 0.0),
        bone("right", None, 40.0, Vec2::new(200.0, 0.0), 0.0),
    ];
    p.layers = vec![Layer::group(
        "G",
        vec![Layer::vector("L", vec![VPath::new("p", false, vec![Anchor::corner(Vec2::new(20.0, 2.0))])])],
    )];
    p.auto_skin("G", 2).unwrap();
    let LayerKind::Group { layers } = &p.layers[0].kind else { panic!() };
    let LayerKind::Vector { paths } = &layers[0].kind else { panic!() };
    let w = &paths[0].anchors[0].weights;
    assert_eq!(w.len(), 2);
    assert_eq!(w[0].bone, "left");
    assert!(w[0].weight > 0.99);
    assert!((w.iter().map(|x| x.weight).sum::<f32>() - 1.0).abs() < 1e-5);
}

// ---- validation ----

#[test]
fn validation_reports_all_problems() {
    let mut p = arm_with_target(0.0, 0.0);
    p.ik[0].target = "nope".into();
    p.channels = vec![Channel::keys("ghost", Prop::BoneAngle, Track::new())];
    let errs = Engine::new(p).err().unwrap();
    assert!(errs.len() >= 2, "{errs:?}");
}

#[test]
fn child_listed_before_parent_is_rejected() {
    let mut p = empty_project();
    p.bones = vec![bone("c", Some("p"), 1.0, Vec2::ZERO, 0.0), bone("p", None, 1.0, Vec2::ZERO, 0.0)];
    assert!(Engine::new(p).is_err());
}

#[test]
fn bad_layer_references_are_rejected() {
    let mut p = empty_project();
    let mut a = Layer::vector("a", vec![]);
    a.mask = Some(MaskRef { layer: "missing".into(), invert: false });
    let mut b = Layer::vector("b", vec![]);
    b.bone = Some("nobone".into());
    p.layers = vec![a, b, Layer::vector("b", vec![])];
    let errs = Engine::new(p).err().unwrap();
    assert!(errs.iter().any(|e| e.contains("unknown mask")));
    assert!(errs.iter().any(|e| e.contains("unknown bone")));
    assert!(errs.iter().any(|e| e.contains("duplicate layer")));
}

#[test]
fn mask_cycles_and_self_masks_are_rejected() {
    let mut p = empty_project();
    let mut a = Layer::vector("a", vec![]);
    a.mask = Some(MaskRef { layer: "b".into(), invert: false });
    let mut b = Layer::vector("b", vec![]);
    b.mask = Some(MaskRef { layer: "a".into(), invert: false });
    let mut c = Layer::vector("c", vec![]);
    c.mask = Some(MaskRef { layer: "c".into(), invert: false });
    p.layers = vec![a, b, c];
    let errs = Engine::new(p).err().unwrap();
    assert!(errs.iter().any(|e| e.contains("cycle")), "{errs:?}");
    assert!(errs.iter().any(|e| e.contains("itself")), "{errs:?}");
}

#[test]
fn layer_switch_on_a_non_switch_layer_is_rejected() {
    let mut p = empty_project();
    p.layers = vec![Layer::vector("v", vec![])];
    p.channels = vec![Channel::keys("v", Prop::LayerSwitch, Track::new())];
    assert!(Engine::new(p).is_err());
}

#[test]
fn bad_expressions_and_get_references_are_rejected() {
    let mut p = arm_with_target(0.0, 0.0);
    p.channels = vec![
        Channel::expr("a", Prop::BoneAngle, "1 +", Mode::Replace),
        Channel::expr("a", Prop::BoneAngle, "get(\"zzz\", \"bone_angle\")", Mode::Replace),
        Channel::expr("a", Prop::BoneAngle, "get(\"a\", \"nonsense\")", Mode::Replace),
    ];
    let errs = Engine::new(p).err().unwrap();
    assert_eq!(errs.len(), 3, "{errs:?}");
}

// ---- layers, transforms, camera ----

fn rect_layer(name: &str) -> Layer {
    Layer::vector(name, vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0)])
}

#[test]
fn group_transform_is_inherited_by_children() {
    let mut p = empty_project();
    let mut g = Layer::group("g", vec![rect_layer("c")]);
    g.transform = Transform2D { position: Vec2::new(30.0, 0.0), scale: Vec2::new(2.0, 2.0), ..Default::default() };
    p.layers = vec![g];
    let e = Engine::new(p).unwrap();
    let f = e.evaluate(0.0);
    let child = &f.nodes[e.node_index("g/c").unwrap()];
    assert!(child.world.apply(Vec2::new(5.0, 5.0)).distance(Vec2::new(40.0, 10.0)) < 1e-4);
}

#[test]
fn layer_channels_animate_transform_and_opacity() {
    let mut p = empty_project();
    p.layers = vec![rect_layer("r")];
    p.channels = vec![
        Channel::keys("r", Prop::LayerX, two(0.0, 50.0)),
        Channel::keys("r", Prop::LayerRotation, two(0.0, 90.0)),
        Channel::keys("r", Prop::LayerOpacity, two(1.0, 0.0)),
    ];
    let e = Engine::new(p).unwrap();
    let f = e.evaluate(5.0);
    let n = &f.nodes[0];
    assert!((n.opacity - 0.5).abs() < 1e-5);
    // rotate 45° then move +25 in x: (1,0) -> (cos45, sin45) + (25,0)
    let w = n.world.apply(Vec2::new(1.0, 0.0));
    assert!(w.distance(Vec2::new(25.0 + std::f32::consts::FRAC_1_SQRT_2, std::f32::consts::FRAC_1_SQRT_2)) < 1e-3, "{w:?}");
}

#[test]
fn bone_attached_layer_follows_its_bone() {
    let mut p = empty_project();
    p.bones = vec![bone("a", None, 50.0, Vec2::new(10.0, 10.0), 0.0)];
    let mut hat = rect_layer("hat");
    hat.bone = Some("a".into());
    p.layers = vec![hat];
    p.channels = vec![Channel::keys("a", Prop::BoneAngle, two(0.0, 90.0))];
    let e = Engine::new(p).unwrap();
    let at0 = e.evaluate(0.0).nodes[0].world.apply(Vec2::new(60.0, 10.0));
    assert!(at0.distance(Vec2::new(60.0, 10.0)) < 1e-3);
    let at10 = e.evaluate(10.0).nodes[0].world.apply(Vec2::new(60.0, 10.0));
    assert!(at10.distance(Vec2::new(10.0, 60.0)) < 1e-3, "{at10:?}");
}

/// L-71 (fixed in wave 7): a bone-attached layer inside a transformed group composes `skin * parent_stage * local`.
/// The skin matrix (bone world * inverse rest) is a stage-space motion, so a group scaled 2 no longer doubles the
/// distance the child travels; at rest nothing changes.
#[test]
fn bone_attach_under_a_transformed_group_composes_bone_after_the_group() {
    let build = |attach: bool| {
        let mut p = empty_project();
        p.bones = vec![bone("a", None, 50.0, Vec2::new(10.0, 10.0), 0.0)];
        let mut child = rect_layer("c");
        if attach {
            child.bone = Some("a".into());
        }
        let mut g = Layer::group("g", vec![child]);
        g.transform.position = Vec2::new(100.0, 0.0);
        g.transform.scale = Vec2::new(2.0, 2.0);
        p.layers = vec![g];
        p.channels = vec![Channel::keys("a", Prop::BoneAngle, two(0.0, 90.0))];
        Engine::new(p).unwrap()
    };
    let (with, without) = (build(true), build(false));
    let q = Vec2::new(60.0, 10.0);
    let node = |e: &Engine, f: f32| e.evaluate(f).nodes.iter().find(|n| n.name == "g/c").unwrap().world;
    // at rest: identical (the bone adds nothing)
    assert!(node(&with, 0.0).apply(q).distance(node(&without, 0.0).apply(q)) < 1e-4);
    // posed: the skin matrix is applied to the child's stage position, once
    let skin = with.evaluate(10.0).world[0].mul(with.inv_rest[0]);
    let group = node(&without, 10.0);
    let expected = skin.apply(group.apply(q));
    assert!(node(&with, 10.0).apply(q).distance(expected) < 1e-3, "world = skin * parent_stage * local");
    // the travel equals the travel of the same point attached with an identity group: not doubled
    let travel = node(&with, 10.0).apply(q).distance(node(&with, 0.0).apply(q));
    let direct = skin.apply(node(&with, 0.0).apply(q)).distance(node(&with, 0.0).apply(q));
    assert!((travel - direct).abs() < 1e-3, "travel {travel} vs bone motion {direct}");
}

/// A layer without a bone follows its bone-attached ancestor; a layer's own bone replaces the ancestor's.
#[test]
fn bone_attach_is_inherited_and_a_child_bone_replaces_the_parents() {
    let mut p = empty_project();
    p.bones = vec![bone("a", None, 50.0, Vec2::new(10.0, 10.0), 0.0), bone("b", None, 50.0, Vec2::new(10.0, 80.0), 0.0)];
    let mut inner = Layer::group("inner", vec![rect_layer("x")]);
    inner.bone = Some("b".into());
    let plain = rect_layer("y");
    let mut outer = Layer::group("outer", vec![inner, plain]);
    outer.bone = Some("a".into());
    p.layers = vec![outer];
    p.channels = vec![Channel::keys("a", Prop::BoneAngle, two(0.0, 90.0))];
    let e = Engine::new(p).unwrap();
    let w = |name: &str, f: f32| e.evaluate(f).nodes.iter().find(|n| n.name == name).unwrap().world;
    let q = Vec2::new(60.0, 10.0);
    let skin_a = e.evaluate(10.0).world[0].mul(e.inv_rest[0]);
    // y has no bone of its own: follows a
    assert!(w("outer/y", 10.0).apply(q).distance(skin_a.apply(w("outer/y", 0.0).apply(q))) < 1e-3);
    // x sits under inner (bone b, which does not move): it does not also take a's motion
    assert!(w("outer/inner/x", 10.0).apply(q).distance(w("outer/inner/x", 0.0).apply(q)) < 1e-3);
}

#[test]
fn switch_layer_selects_one_child_with_step_keys() {
    let mut p = empty_project();
    p.layers = vec![Layer::switch("mouth", vec![rect_layer("a"), rect_layer("b"), rect_layer("c")])];
    p.channels = vec![Channel::keys(
        "mouth",
        Prop::LayerSwitch,
        Track::with_keys(&[(0.0, 0.0, Ease::Step), (4.0, 2.0, Ease::Step), (8.0, 1.0, Ease::Step)]),
    )];
    let e = Engine::new(p).unwrap();
    let shown = |f: f32| {
        let fr = e.evaluate(f);
        let ch = fr.nodes[0].children.clone();
        assert_eq!(ch.len(), 1);
        fr.nodes[ch[0]].name.clone()
    };
    assert_eq!(shown(0.0), "mouth/a");
    assert_eq!(shown(3.9), "mouth/a");
    assert_eq!(shown(4.0), "mouth/c");
    assert_eq!(shown(9.0), "mouth/b");
}

#[test]
fn camera_pan_zoom_and_parallax() {
    let mut p = empty_project(); // 100×100 stage
    let mut far = rect_layer("far");
    far.parallax = 0.5;
    let mut hud = rect_layer("hud");
    hud.parallax = 0.0;
    p.layers = vec![far, rect_layer("near"), hud];
    p.channels = vec![Channel::keys("", Prop::CameraX, two(50.0, 70.0))];
    let e = Engine::new(p).unwrap();
    let f = e.evaluate(10.0); // camera moved +20 in x
    let sx = |name: &str| f.nodes[e.node_index(name).unwrap()].world.apply(Vec2::ZERO).x;
    assert!((sx("near") - -20.0).abs() < 1e-4, "near scrolls fully");
    assert!((sx("far") - -10.0).abs() < 1e-4, "far scrolls at half speed");
    assert!((sx("hud") - 0.0).abs() < 1e-4, "hud is pinned to the screen");

    let mut p2 = empty_project();
    p2.layers = vec![rect_layer("r")];
    p2.camera.zoom = 2.0;
    let e2 = Engine::new(p2).unwrap();
    // zoom about the stage centre: stage centre stays put, origin goes to -50
    let w = &e2.evaluate(0.0).nodes[0].world;
    assert!(w.apply(Vec2::new(50.0, 50.0)).distance(Vec2::new(50.0, 50.0)) < 1e-4);
    assert!(w.apply(Vec2::ZERO).distance(Vec2::new(-50.0, -50.0)) < 1e-4);
}

// ---- procedural channels ----

#[test]
fn expression_channels_see_time_and_other_values() {
    let mut p = arm_with_target(120.0, 0.0);
    p.ik.clear();
    p.channels = vec![
        Channel::keys("a", Prop::BoneAngle, two(0.0, 40.0)),
        // b mirrors a at half strength, driven by an expression reading a's current angle
        Channel::expr("b", Prop::BoneAngle, "get(\"a\", \"bone_angle\") * -0.5", Mode::Replace),
        Channel::expr("t", Prop::BoneOffsetY, "10 * sin(pi * t)", Mode::Replace),
    ];
    let e = Engine::new(p).unwrap();
    let f = e.evaluate(10.0);
    assert!((f.world[1].rotation().to_degrees() - (40.0 - 20.0)).abs() < 1e-3);
    let f5 = e.evaluate(1.0); // 10 frames → t = 0.1 → sin(0.1π)·10
    assert!((f5.world[2].apply(Vec2::ZERO).y - 10.0 * (0.1f32 * std::f32::consts::PI).sin()).abs() < 1e-3);
}

#[test]
fn noise_adds_on_top_of_keys_and_is_deterministic() {
    let mut p = arm_with_target(120.0, 0.0);
    p.ik.clear();
    p.channels = vec![
        Channel::keys("a", Prop::BoneAngle, two(0.0, 0.0)),
        Channel::noise("a", Prop::BoneAngle, 10.0, 3.0, 5),
    ];
    let e = Engine::new(p.clone()).unwrap();
    let ang = |f: f32| e.evaluate(f).world[0].rotation().to_degrees();
    assert!((0..10).any(|f| ang(f as f32).abs() > 0.5), "noise moves the bone");
    assert!((0..10).all(|f| ang(f as f32).abs() <= 10.0 + 1e-3));
    assert_eq!(ang(3.0), ang(3.0));
    assert_eq!(ang(3.0), Engine::new(p).unwrap().evaluate(3.0).world[0].rotation().to_degrees());
}

#[test]
fn driven_keys_map_one_value_to_another() {
    let mut p = arm_with_target(120.0, 0.0);
    p.ik.clear();
    p.layers = vec![rect_layer("bulge")];
    // elbow bend (b's angle, 0..90°) drives the bulge layer's x-scale 1 → 2
    p.channels = vec![
        Channel::keys("b", Prop::BoneAngle, two(0.0, 90.0)),
        Channel::driven(
            "bulge",
            Prop::LayerScaleX,
            "b",
            Prop::BoneAngle,
            Track::with_keys(&[(0.0, 1.0, Ease::Linear), (90.0, 2.0, Ease::Linear)]),
        ),
    ];
    let e = Engine::new(p).unwrap();
    let sx = |f: f32| e.evaluate(f).nodes[0].world.apply(Vec2::new(1.0, 0.0)).x;
    assert!((sx(0.0) - 1.0).abs() < 1e-4);
    assert!((sx(5.0) - 1.5).abs() < 1e-3);
    assert!((sx(10.0) - 2.0).abs() < 1e-4);
}

#[test]
fn channel_modes_and_weight() {
    let mut p = arm_with_target(120.0, 0.0);
    p.ik.clear();
    p.channels = vec![
        Channel::keys("a", Prop::BoneScale, two(2.0, 2.0)),
        Channel::expr("a", Prop::BoneScale, "3", Mode::Multiply),
        Channel::expr("a", Prop::BoneOffsetX, "10", Mode::Replace).with_weight(0.5),
        Channel::expr("a", Prop::BoneOffsetX, "4", Mode::Add),
    ];
    let e = Engine::new(p).unwrap();
    let f = e.evaluate(0.0);
    assert!((f.world[0].x_scale() - 6.0).abs() < 1e-4, "2 × 3");
    assert!((f.world[0].apply(Vec2::ZERO).x - 9.0).abs() < 1e-4, "0→5 (50% toward 10) then +4");
}

#[test]
fn evaluate_is_a_pure_function_of_frame() {
    let mut p = arm_with_target(120.0, 40.0);
    p.channels = vec![Channel::noise("t", Prop::BoneOffsetX, 20.0, 2.0, 1)];
    let e = Engine::new(p).unwrap();
    let a = e.evaluate(7.5).world[1];
    let _ = e.evaluate(2.0);
    let b = e.evaluate(7.5).world[1];
    assert_eq!(a, b);
}

// ---- large finite values (R1) ----

fn one_anchor_project() -> Project {
    let mut p = empty_project();
    p.layers = vec![Layer::vector("L", vec![VPath::new("p", false, vec![Anchor::corner(Vec2::new(5.0, 5.0))])])];
    p
}

#[test]
fn channel_expressions_with_absurd_nesting_are_rejected_not_fatal() {
    let mut p = arm_with_target(0.0, 0.0);
    p.channels = vec![
        Channel::expr("a", Prop::BoneAngle, &format!("{}1{}", "(".repeat(20_000), ")".repeat(20_000)), Mode::Replace),
        Channel::expr("a", Prop::BoneAngle, &format!("{}1", "-".repeat(200_000)), Mode::Replace),
    ];
    let errs = Engine::new(p).err().unwrap();
    assert_eq!(errs.len(), 2, "{errs:?}");
}

#[test]
fn huge_but_finite_numbers_are_refused_with_the_place() {
    // each used to give exit 0 with NaN / inf / null in the output
    let mut p = empty_project();
    p.camera.zoom = 3.0e38;
    let err = Engine::new(p).err().unwrap().join("; ");
    assert!(err.contains("/camera/zoom") && err.contains("outside the supported range"), "{err}");

    let mut p = one_anchor_project();
    p.layers[0].transform.scale = Vec2::new(3.0e38, 1.0);
    assert!(Engine::new(p).is_err());

    let mut p = empty_project();
    p.bones = vec![bone("a", None, 10.0, Vec2::new(3.0e38, 0.0), 0.0)];
    assert!(Engine::new(p).is_err());

    let mut p = empty_project();
    p.layers = vec![Layer::vector("L", vec![VPath::new("p", false, vec![Anchor::corner(Vec2::new(3.0e38, 0.0))])])];
    let err = Engine::new(p).err().unwrap().join("; ");
    assert!(err.contains("anchors"), "{err}");

    let mut p = one_anchor_project();
    p.layers[0].transform.position = Vec2::new(3.4e38, 0.0);
    assert!(Engine::new(p).is_err());
}

#[test]
fn huge_literals_in_json_are_refused_at_load_not_turned_into_infinity() {
    // 1e300 does not fit an f32: serde makes it infinity, which then serialises as null and slips past any later scan
    let base = serde_json::to_value(one_anchor_project()).unwrap();
    let cases: [(&str, serde_json::Value); 4] = [
        ("/camera/zoom", serde_json::json!(1e300)),
        ("/layers/0/transform/scale", serde_json::json!([1e300, 1.0])),
        ("/layers/0/paths/0/anchors/0/pos", serde_json::json!([1e300, 0.0])),
        ("/bones", serde_json::json!([{"name": "a", "parent": null, "length": 5.0, "rest_offset": [1e308, 0.0], "rest_angle": 0.0}])),
    ];
    for (ptr, val) in cases {
        let mut v = base.clone();
        let target = if ptr == "/bones" { Some(&mut v) } else { None };
        match target {
            Some(t) => t["bones"] = val,
            None => {
                // layers/0/paths is only valid for vector layers: set through the pointer
                *v.pointer_mut(ptr).unwrap_or_else(|| panic!("{ptr} missing in {base}")) = val;
            }
        }
        let err = Project::from_json(&v.to_string()).err().unwrap_or_else(|| panic!("{ptr} was accepted")).to_string();
        assert!(err.contains("outside the supported range"), "{ptr}: {err}");
    }
    assert!(Project::from_json(&serde_json::to_string(&base).unwrap()).is_ok());
}

#[test]
fn ordinary_large_numbers_still_load() {
    let mut p = one_anchor_project();
    p.layers[0].transform.position = Vec2::new(9.0e6, -9.0e6);
    p.camera.zoom = 100.0;
    assert!(Engine::new(p).is_ok());
}

#[test]
fn overflow_through_nested_scales_is_reported_not_exported_as_nan() {
    // every number is within range, the product of six nested scales is not an f32
    let mut inner = Layer::vector("L", vec![VPath::new("p", false, vec![Anchor::corner(Vec2::new(5.0, 5.0))])]);
    inner.transform.scale = Vec2::new(1e7, 1e7);
    let mut cur = inner;
    for i in 0..6 {
        let mut g = Layer::group(&format!("g{i}"), vec![cur]);
        g.transform.scale = Vec2::new(1e7, 1e7);
        cur = g;
    }
    let mut p = empty_project();
    p.layers = vec![cur];
    let e = Engine::new(p).unwrap();
    let f = e.evaluate(0.0);
    assert!(f.nonfinite.is_some());
    for n in &f.nodes {
        let m = n.world;
        assert!([m.a, m.b, m.c, m.d, m.tx, m.ty].iter().all(|v| v.is_finite()), "repaired frames carry no NaN");
    }
    let err = e.evaluate_checked(0.0).err().unwrap();
    assert!(err.starts_with("non-finite value in ") && err.contains("at frame 0"), "{err}");
    assert!(e.check_all_frames_finite().unwrap_err().starts_with("non-finite value in "));
    // a normal project passes
    assert!(Engine::new(one_anchor_project()).unwrap().check_all_frames_finite().is_ok());
}
