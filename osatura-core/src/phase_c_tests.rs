// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Phase C: liquid layers, boolean layers, emitters inside the engine.
use crate::engine::Engine;
use crate::fx::*;
use crate::math::Vec2;
use crate::path::VPath;
use crate::project::*;
use crate::raster::{render_pixmap, RasterOptions};
use crate::style::Paint;

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}
fn blob(x: f32, y: f32, r: f32) -> Blob {
    Blob { pos: v(x, y), radius: r, bone: None }
}

#[test]
fn liquid_layer_fuses_blobs_and_round_trips_json() {
    let mut p = Project::new("t", 200.0, 200.0, 24.0, 10);
    p.layers = vec![Layer::liquid("goo", vec![blob(70.0, 100.0, 20.0), blob(100.0, 100.0, 20.0)], Paint::solid("#3366ff"))];
    let json = p.to_json_pretty();
    let p2 = Project::from_json(&json).unwrap();
    assert_eq!(p, p2);
    let e = Engine::new(p2).unwrap();
    let f = e.evaluate(0.0);
    assert_eq!(f.nodes[0].paths.len(), 1, "two touching blobs → one body");
    let pm = render_pixmap(&e, 0.0, &RasterOptions { scale: 1.0, show_bones: false, transparent: false });
    assert_eq!(pm.pixel(85, 100).unwrap().alpha(), 255, "filled between the centres");
    assert_eq!(pm.pixel(10, 10).unwrap().alpha(), 0);
}

#[test]
fn bone_bound_blobs_move_and_merge_over_time() {
    use crate::anim::{Ease, Track};
    let mut p = Project::new("t", 300.0, 200.0, 24.0, 10);
    p.bones = vec![Bone::new("root", None, 10.0, v(0.0, 0.0), 0.0), Bone::new("drop", None, 10.0, v(220.0, 100.0), 0.0)];
    p.layers = vec![Layer::liquid(
        "goo",
        vec![blob(60.0, 100.0, 18.0), Blob { pos: Vec2::ZERO, radius: 18.0, bone: Some("drop".into()) }],
        Paint::solid("#00aa88"),
    )];
    p.channels = vec![Channel::keys("drop", Prop::BoneOffsetX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (9.0, -140.0, Ease::Linear)]))];
    let e = Engine::new(p).unwrap();
    assert_eq!(e.evaluate(0.0).nodes[0].paths.len(), 2, "far apart");
    assert_eq!(e.evaluate(9.0).nodes[0].paths.len(), 1, "fused");
}

#[test]
fn boolean_layer_cuts_a_hole() {
    let mut p = Project::new("t", 100.0, 100.0, 24.0, 2);
    let subject = VPath::rect("s", 10.0, 10.0, 80.0, 80.0).with_fill("#ff0000");
    let cutter = VPath::rect("c", 30.0, 30.0, 40.0, 40.0);
    p.layers = vec![Layer::boolean("frame", BoolOp::Difference, vec![subject, cutter])];
    let e = Engine::new(p).unwrap();
    let pm = render_pixmap(&e, 0.0, &RasterOptions { scale: 1.0, show_bones: false, transparent: false });
    assert_eq!(pm.pixel(15, 50).unwrap().alpha(), 255, "ring is filled");
    assert_eq!(pm.pixel(50, 50).unwrap().alpha(), 0, "hole is empty");
}

#[test]
fn emitter_layer_is_deterministic_and_time_dependent() {
    let mut p = Project::new("t", 200.0, 200.0, 24.0, 48);
    let spec = EmitterSpec { pos: v(100.0, 180.0), rate: 40.0, lifetime: 1.0, speed: 80.0, gravity: v(0.0, 60.0), seed: 3, ..Default::default() };
    p.layers = vec![Layer::emitter("fx", spec)];
    let e = Engine::new(p).unwrap();
    let n = |f: f32| e.evaluate(f).nodes[0].paths.len();
    assert_eq!(n(30.0), n(30.0));
    assert!(n(0.0) <= 1 && n(30.0) > 20);
    let a = e.evaluate(30.0).nodes[0].paths[0].anchors[0].pos;
    let _ = e.evaluate(2.0);
    assert_eq!(a, e.evaluate(30.0).nodes[0].paths[0].anchors[0].pos);
}

#[test]
fn validation_covers_effect_layers() {
    let mut p = Project::new("t", 100.0, 100.0, 24.0, 2);
    p.layers = vec![
        Layer::liquid("l", vec![Blob { pos: Vec2::ZERO, radius: -1.0, bone: Some("ghost".into()) }], Paint::solid("#000")),
        Layer::boolean("b", BoolOp::Union, vec![VPath::rect("a", 0.0, 0.0, 1.0, 1.0)]),
        Layer::emitter("e", EmitterSpec { rate: 0.0, ..Default::default() }),
    ];
    let all = match Engine::new(p) {
        Err(e) => e.join("\n"),
        Ok(_) => panic!("must fail"),
    };
    assert!(all.contains("blob radius") && all.contains("ghost") && all.contains("at least two paths") && all.contains("rate"), "{all}");
}
