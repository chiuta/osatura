// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `gizmo`.
#![allow(unused_imports)]

use super::*;
use crate::anim::Track;
use crate::path::{Anchor, VPath};
use crate::project::{Bone, IkChain, Layer, Project};

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}
fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}
fn rect(name: &str, x: f32, y: f32, w: f32, h: f32) -> Layer {
    Layer::vector(name, vec![VPath::rect("r", x, y, w, h).with_fill("#c33")])
}
fn editor(layers: Vec<Layer>) -> Editor {
    let mut p = Project::new("t", 400.0, 300.0, 24.0, 11);
    p.layers = layers;
    Editor::new(p).unwrap()
}
fn req(sel: &[Target], handle: Handle, start: Vec2, delta: Vec2) -> DragRequest {
    DragRequest {
        frame: 0.0,
        selection: sel.to_vec(),
        handle,
        start,
        delta,
        modifiers: Modifiers { no_snap: true, ..Default::default() },
        snap: SnapOptions::default(),
        auto_key: false,
        locked: vec![],
        pivot: None,
    }
}
fn l(name: &str) -> Target {
    Target::layer(name)
}
fn pts(ed: &Editor, t: &Target) -> Vec<Vec2> {
    pick::target_points(ed.engine(), &ed.engine().evaluate(0.0), t)
}
/// After a commit the selection's geometry must equal `d` applied to its old geometry.
fn assert_moved_by(before: &[Vec2], after: &[Vec2], d: &Affine2) {
    assert_eq!(before.len(), after.len());
    for (b, a) in before.iter().zip(after) {
        assert!(d.apply(*b).distance(*a) < 5e-3, "{:?} -> {:?}, expected {:?}", b, a, d.apply(*b));
    }
}

#[test]
fn translate_commit_numbers_and_single_undo() {
    let mut ed = editor(vec![rect("a", 0.0, 0.0, 100.0, 50.0)]);
    ed.apply(Op::Set { path: "/layers/0/transform/position".into(), value: json!([50.0, 50.0]) }).unwrap();
    let before = ed.doc().clone();
    let r = commit(&mut ed, &req(&[l("a")], Handle::Move, v(60.0, 60.0), v(10.0, 5.0))).unwrap();
    assert!(r.changed && r.summary.contains("move 10.0, 5.0"), "{}", r.summary);
    let p = &ed.project().layers[0].transform.position;
    assert!(near(p.x, 60.0) && near(p.y, 55.0), "{p:?}");
    assert!(ed.undo());
    assert_eq!(ed.doc(), &before, "undo restores the exact document");
    assert!(!ed.can_undo() || ed.doc() == &before);
    // one history entry per drag: undo once more must have nothing of ours to undo
    let mut ed2 = editor(vec![rect("a", 0.0, 0.0, 10.0, 10.0)]);
    commit(&mut ed2, &req(&[l("a")], Handle::Move, v(5.0, 5.0), v(3.0, 4.0))).unwrap();
    assert!(ed2.undo() && !ed2.can_undo());
}

#[test]
fn axis_constraint_picks_the_dominant_axis() {
    let mut ed = editor(vec![rect("a", 0.0, 0.0, 10.0, 10.0)]);
    let mut r = req(&[l("a")], Handle::Move, v(5.0, 5.0), v(30.0, 7.0));
    r.modifiers.constrain = true;
    commit(&mut ed, &r).unwrap();
    let p = ed.project().layers[0].transform.position;
    assert!(near(p.x, 30.0) && near(p.y, 0.0), "{p:?}");
    let mut r = req(&[l("a")], Handle::Move, v(5.0, 5.0), v(2.0, -40.0));
    r.modifiers.constrain = true;
    commit(&mut ed, &r).unwrap();
    let p = ed.project().layers[0].transform.position;
    assert!(near(p.x, 30.0) && near(p.y, -40.0), "{p:?}");
}

#[test]
fn snapping_to_object_bounds_canvas_and_grid_with_tolerance() {
    // a: 0..40 square; b: wide target whose left edge is at x = 200
    let ed = editor(vec![rect("a", 0.0, 0.0, 40.0, 40.0), rect("b", 200.0, 100.0, 80.0, 80.0)]);
    let mut r = req(&[l("a")], Handle::Move, v(20.0, 20.0), v(157.0, 0.0));
    r.modifiers.no_snap = false;
    // right edge of a would be at 197 -> snaps to b's left edge 200 (diff 3 <= 6)
    let p = preview(&ed, &r).unwrap();
    assert!(near(p.matrix[4], 160.0), "{:?}", p.matrix);
    assert!(p.guides.iter().any(|g| g.axis == "x" && near(g.at, 200.0) && g.kind == "object"), "{:?}", p.guides);
    // out of tolerance: no snap
    let r2 = DragRequest { delta: v(150.0, 0.0), ..r.clone() };
    let p2 = preview(&ed, &r2).unwrap();
    assert!(near(p2.matrix[4], 150.0) && p2.guides.iter().all(|g| g.axis != "x"), "{:?} {:?}", p2.matrix, p2.guides);
    // canvas edge: right edge at 400 (canvas width) within tolerance
    let r3 = DragRequest { delta: v(357.0, 0.0), ..r.clone() };
    let p3 = preview(&ed, &r3).unwrap();
    assert!(near(p3.matrix[4], 360.0) && p3.guides.iter().any(|g| g.kind == "canvas"), "{:?}", p3);
    // grid only
    let mut r4 = r.clone();
    r4.snap = SnapOptions { grid: Some(25.0), objects: false, canvas: false, tolerance: 6.0 };
    r4.delta = v(21.0, 0.0);
    let p4 = preview(&ed, &r4).unwrap();
    assert!(near(p4.matrix[4], 25.0), "left edge 21 snaps to grid 25: {:?}", p4.matrix);
    assert!(p4.guides.iter().any(|g| g.kind == "grid"));
    // modifier turns snapping off
    let mut r5 = r.clone();
    r5.modifiers.no_snap = true;
    assert!(near(preview(&ed, &r5).unwrap().matrix[4], 157.0));
}

#[test]
fn rotate_around_centre_numbers_and_angle_snap() {
    let mut ed = editor(vec![rect("a", 0.0, 0.0, 100.0, 100.0)]);
    // grab the right-middle of the centre (50,50), swing it to straight down: +90 degrees
    let r = req(&[l("a")], Handle::Rotate, v(100.0, 50.0), v(-50.0, 50.0));
    let pr = preview(&ed, &r).unwrap();
    assert!(pr.summary.contains("90.0"), "{}", pr.summary);
    let before = pts(&ed, &l("a"));
    commit(&mut ed, &r).unwrap();
    let t = &ed.project().layers[0].transform;
    assert!(near(t.rotation, 90.0), "{}", t.rotation);
    assert!(near(t.position.x, 100.0) && near(t.position.y, 0.0), "centre stays put: {:?}", t.position);
    let after = pts(&ed, &l("a"));
    assert_moved_by(&before, &after, &Affine2::translate(v(50.0, 50.0)).mul(Affine2::rotate(std::f32::consts::FRAC_PI_2)).mul(Affine2::translate(v(-50.0, -50.0))));
    // 15 degree snap: a 40 degree swing becomes 45
    let ed2 = editor(vec![rect("a", 0.0, 0.0, 100.0, 100.0)]);
    let a = 40f32.to_radians();
    let mut r2 = req(&[l("a")], Handle::Rotate, v(100.0, 50.0), v(50.0 * a.cos() - 50.0, 50.0 * a.sin()));
    r2.modifiers.constrain = true;
    assert!(preview(&ed2, &r2).unwrap().summary.contains("45.0"));
    // pointer exactly on the pivot is rejected
    assert!(preview(&ed2, &req(&[l("a")], Handle::Rotate, v(50.0, 50.0), v(1.0, 1.0))).is_err());
}

#[test]
fn scale_corner_edge_centre_uniform_and_flip() {
    let se = Handle::Scale { at: ScaleHandle::Se };
    let mut ed = editor(vec![rect("a", 0.0, 0.0, 100.0, 100.0)]);
    // corner, pivot at the opposite corner: doubles
    commit(&mut ed, &req(&[l("a")], se, v(100.0, 100.0), v(100.0, 100.0))).unwrap();
    let t = ed.project().layers[0].transform.clone();
    assert!(near(t.scale.x, 2.0) && near(t.scale.y, 2.0) && near(t.position.x, 0.0), "{t:?}");
    ed.undo();
    // around the centre (alt)
    let mut r = req(&[l("a")], se, v(100.0, 100.0), v(50.0, 50.0));
    r.modifiers.center = true;
    commit(&mut ed, &r).unwrap();
    let t = ed.project().layers[0].transform.clone();
    assert!(near(t.scale.x, 2.0) && near(t.position.x, -50.0) && near(t.position.y, -50.0), "{t:?}");
    ed.undo();
    // east edge scales x only
    commit(&mut ed, &req(&[l("a")], Handle::Scale { at: ScaleHandle::E }, v(100.0, 50.0), v(50.0, 20.0))).unwrap();
    let t = ed.project().layers[0].transform.clone();
    assert!(near(t.scale.x, 1.5) && near(t.scale.y, 1.0), "{t:?}");
    ed.undo();
    // edge + constrain = proportional about the opposite edge's centre
    let mut r = req(&[l("a")], Handle::Scale { at: ScaleHandle::E }, v(100.0, 50.0), v(100.0, 0.0));
    r.modifiers.constrain = true;
    commit(&mut ed, &r).unwrap();
    let t = ed.project().layers[0].transform.clone();
    assert!(near(t.scale.x, 2.0) && near(t.scale.y, 2.0) && near(t.position.y, -50.0), "{t:?}");
    ed.undo();
    // corner + constrain keeps proportions from the diagonal projection
    let mut r = req(&[l("a")], se, v(100.0, 100.0), v(100.0, 0.0));
    r.modifiers.constrain = true;
    commit(&mut ed, &r).unwrap();
    let t = ed.project().layers[0].transform.clone();
    assert!(near(t.scale.x, t.scale.y) && t.scale.x > 1.0, "{t:?}");
    ed.undo();
    // dragging the east edge across the west edge flips x
    let before = pts(&ed, &l("a"));
    let r = req(&[l("a")], Handle::Scale { at: ScaleHandle::E }, v(100.0, 50.0), v(-200.0, 0.0));
    let d = drag_matrix(box_frame(ed.engine(), &ed.engine().evaluate(0.0), &Selection::from_items(vec![l("a")]), 1.0).as_ref(), None, r.handle, r.start, r.delta, &r.modifiers, &r.snap, &SnapTargets::default(), None).unwrap();
    commit(&mut ed, &r).unwrap();
    let t = ed.project().layers[0].transform.clone();
    assert!(near(t.scale.x, -1.0) && near(t.scale.y, 1.0) && near(t.rotation, 0.0), "flip shows as a negative scale: {t:?}");
    assert_moved_by(&before, &pts(&ed, &l("a")), &d.matrix);
    // the west edge stayed where it was
    assert!(near(t.position.x, 0.0));
    // a handle that starts on its pivot is an error, not a NaN
    let fresh = editor(vec![rect("a", 0.0, 0.0, 100.0, 100.0)]);
    let bad = req(&[l("a")], Handle::Scale { at: ScaleHandle::Nw }, v(100.0, 100.0), v(1.0, 1.0));
    assert!(preview(&fresh, &bad).is_err());
}

#[test]
fn scale_inside_a_rotated_group_follows_the_layer_axes() {
    let mut g = Layer::group("g", vec![rect("k", 0.0, 0.0, 100.0, 40.0)]);
    g.transform.rotation = 90.0;
    g.transform.position = v(200.0, 50.0);
    let mut ed = editor(vec![g]);
    let f = ed.engine().evaluate(0.0);
    let bx = box_frame(ed.engine(), &f, &Selection::from_items(vec![l("g/k")]), 1.0).unwrap();
    // east handle of the oriented box sits at the right end of the rotated layer
    let e = bx.point(1.0, 0.5);
    assert!(e.distance(v(180.0, 150.0)) < 1e-3, "{e:?}");
    let before = pts(&ed, &l("g/k"));
    let r = req(&[l("g/k")], Handle::Scale { at: ScaleHandle::E }, e, v(0.0, 100.0));
    let dm = preview(&ed, &r).unwrap().matrix;
    commit(&mut ed, &r).unwrap();
    let t = &ed.project().layers[0].layers_ref()[0].transform;
    assert!(near(t.scale.x, 2.0) && near(t.scale.y, 1.0), "{t:?} {dm:?}");
    let d = Affine2 { a: dm[0], b: dm[1], c: dm[2], d: dm[3], tx: dm[4], ty: dm[5] };
    assert_moved_by(&before, &pts(&ed, &l("g/k")), &d);
}

trait LayersRef {
    fn layers_ref(&self) -> &[Layer];
}
impl LayersRef for Layer {
    fn layers_ref(&self) -> &[Layer] {
        self.children()
    }
}

#[test]
fn move_in_nested_rotated_scaled_parent_moves_by_exactly_the_pointer_delta() {
    let mut inner = rect("k", 0.0, 0.0, 30.0, 30.0);
    inner.transform.rotation = 20.0;
    inner.transform.skew = 10.0;
    let mut g = Layer::group("g", vec![inner]);
    g.transform.rotation = 37.0;
    g.transform.scale = v(2.0, 0.5);
    g.transform.position = v(150.0, 120.0);
    let mut ed = editor(vec![g]);
    let before = pts(&ed, &l("g/k"));
    commit(&mut ed, &req(&[l("g/k")], Handle::Move, v(0.0, 0.0), v(13.0, -7.0))).unwrap();
    assert_moved_by(&before, &pts(&ed, &l("g/k")), &Affine2::translate(v(13.0, -7.0)));
    // rotation of the same layer about the canvas pivot: rigid in canvas space
    let before = pts(&ed, &l("g/k"));
    let r = req(&[l("g/k")], Handle::Rotate, v(300.0, 0.0), v(0.0, 100.0));
    let dm = preview(&ed, &r).unwrap().matrix;
    commit(&mut ed, &r).unwrap();
    let d = Affine2 { a: dm[0], b: dm[1], c: dm[2], d: dm[3], tx: dm[4], ty: dm[5] };
    assert_moved_by(&before, &pts(&ed, &l("g/k")), &d);
}

#[test]
fn group_and_child_both_selected_moves_once() {
    let g = Layer::group("g", vec![rect("k", 0.0, 0.0, 10.0, 10.0)]);
    let mut ed = editor(vec![g]);
    let before = pts(&ed, &l("g/k"));
    commit(&mut ed, &req(&[l("g"), l("g/k")], Handle::Move, v(0.0, 0.0), v(20.0, 0.0))).unwrap();
    assert_moved_by(&before, &pts(&ed, &l("g/k")), &Affine2::translate(v(20.0, 0.0)));
}

#[test]
fn auto_key_behaviour() {
    let mut p = Project::new("t", 400.0, 300.0, 24.0, 11);
    p.layers = vec![rect("a", 0.0, 0.0, 10.0, 10.0), rect("b", 0.0, 0.0, 10.0, 10.0)];
    p.channels = vec![Channel::keys("a", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 100.0, Ease::Linear)]))];
    let mut ed = Editor::new(p).unwrap();
    // animated property: keyed at the current frame even with auto-key off; other props stay static
    let mut r = req(&[l("a")], Handle::Move, v(5.0, 5.0), v(10.0, 3.0));
    r.frame = 5.0;
    let out = commit(&mut ed, &r).unwrap();
    let ch = &ed.project().channels[0];
    let Source::Keys { track } = &ch.source else { panic!() };
    assert_eq!(track.keys.len(), 3);
    assert!(near(track.eval(5.0).unwrap(), 60.0), "{:?}", track.keys);
    assert!(near(track.eval(0.0).unwrap(), 0.0) && near(track.eval(10.0).unwrap(), 100.0), "other keys untouched");
    assert!(near(ed.engine().evaluate(5.0).nodes[0].world.tx, 60.0));
    assert!(out.edits.iter().any(|e| e.prop == "layer_x" && e.mode == "key"));
    assert!(out.edits.iter().any(|e| e.prop == "layer_y" && e.mode == "static"), "{:?}", out.edits);
    assert!(near(ed.project().layers[0].transform.position.y, 3.0));
    assert_eq!(ed.project().channels.len(), 1, "no channel created for y");
    // not animated, auto-key off: static edit, no channels
    let n = ed.project().channels.len();
    commit(&mut ed, &req(&[l("b")], Handle::Move, v(1.0, 1.0), v(5.0, 0.0))).unwrap();
    assert_eq!(ed.project().channels.len(), n);
    assert!(near(ed.project().layers[1].transform.position.x, 5.0));
    // not animated, auto-key on: channel created, static value untouched
    let mut r = req(&[l("b")], Handle::Move, v(1.0, 1.0), v(0.0, 8.0));
    r.auto_key = true;
    r.frame = 3.0;
    commit(&mut ed, &r).unwrap();
    assert_eq!(ed.project().channels.len(), n + 1);
    let c = ed.project().channels.last().unwrap();
    assert!(c.target == "b" && c.prop == Prop::LayerY);
    let Source::Keys { track } = &c.source else { panic!() };
    assert!(near(track.eval(3.0).unwrap(), 8.0) && track.keys[0].frame == 3.0);
    assert!(near(ed.project().layers[1].transform.position.y, 0.0));
    // undo removes the channel again
    assert!(ed.undo());
    assert_eq!(ed.project().channels.len(), n);
}

#[test]
fn locked_channel_and_locked_layer_refuse_the_whole_drag() {
    let mut p = Project::new("t", 400.0, 300.0, 24.0, 11);
    p.layers = vec![rect("a", 0.0, 0.0, 10.0, 10.0), rect("b", 0.0, 0.0, 10.0, 10.0)];
    p.channels = vec![Channel::keys("a", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 100.0, Ease::Linear)]))];
    crate::chanmeta::set_meta(&mut p, "a", Some(Prop::LayerX), &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
    let mut ed = Editor::new(p).unwrap();
    let before = ed.doc().clone();
    let e = commit(&mut ed, &req(&[l("b"), l("a")], Handle::Move, v(1.0, 1.0), v(5.0, 5.0))).unwrap_err();
    assert!(e.join(" ").contains("locked"), "{e:?}");
    assert_eq!(ed.doc(), &before, "nothing applied, not even the unlocked layer");
    assert!(!ed.can_undo());
    let mut r = req(&[l("b")], Handle::Move, v(1.0, 1.0), v(5.0, 5.0));
    r.locked = vec!["b".into()];
    assert!(commit(&mut ed, &r).unwrap_err().join(" ").contains("locked"));
}

#[test]
fn zero_drag_leaves_no_history_and_bad_input_is_an_error() {
    let mut ed = editor(vec![rect("a", 0.0, 0.0, 10.0, 10.0)]);
    let out = commit(&mut ed, &req(&[l("a")], Handle::Move, v(1.0, 1.0), v(0.0, 0.0))).unwrap();
    assert!(!out.changed && !ed.can_undo());
    assert!(commit(&mut ed, &req(&[], Handle::Move, v(0.0, 0.0), v(1.0, 1.0))).is_err());
    assert!(commit(&mut ed, &req(&[l("nope")], Handle::Move, v(0.0, 0.0), v(1.0, 1.0))).is_err());
    assert!(commit(&mut ed, &req(&[l("a")], Handle::Move, v(0.0, 0.0), v(f32::NAN, 1.0))).is_err());
    assert!(commit(&mut ed, &req(&[l("a")], Handle::BoneTip, v(0.0, 0.0), v(1.0, 1.0))).is_err());
    // a line has no extent along one axis: the box is padded, scaling its thin axis is finite
    let line = Layer::vector("ln", vec![VPath::new("p", false, vec![Anchor::corner(v(0.0, 50.0)), Anchor::corner(v(100.0, 50.0))]).with_stroke("#000", 2.0)]);
    let ed2 = editor(vec![line]);
    let pr = preview(&ed2, &req(&[l("ln")], Handle::Scale { at: ScaleHandle::E }, v(100.0, 50.0), v(100.0, 0.0))).unwrap();
    assert!(pr.matrix.iter().all(|x| x.is_finite()));
}

#[test]
fn decompose_round_trips_and_prefers_negative_scale_for_mirrors() {
    let t0 = Transform2D { position: v(30.0, -12.0), rotation: 33.0, scale: v(1.5, 0.7), skew: 12.0, anchor: v(5.0, 9.0) };
    let t1 = decompose(&t0.matrix(), t0.anchor, 33.0).unwrap();
    assert!(near(t1.rotation, 33.0) && near(t1.scale.x, 1.5) && near(t1.scale.y, 0.7) && near(t1.skew, 12.0) && t1.position.distance(t0.position) < 1e-3, "{t1:?}");
    // mirror in x
    let m = Affine2::scale_xy(-1.0, 1.0);
    let t = decompose(&m, Vec2::ZERO, 0.0).unwrap();
    assert!(near(t.scale.x, -1.0) && near(t.scale.y, 1.0) && near(t.rotation, 0.0), "{t:?}");
    // continuity across a full turn
    let big = Transform2D { rotation: 725.0, ..Default::default() };
    let t = decompose(&big.matrix(), Vec2::ZERO, 720.0).unwrap();
    assert!(near(t.rotation, 725.0), "{}", t.rotation);
    assert!(decompose(&Affine2::scale_xy(0.0, 1.0), Vec2::ZERO, 0.0).is_err());
}

fn bone_project() -> Editor {
    let mut p = Project::new("t", 400.0, 300.0, 24.0, 11);
    p.bones = vec![Bone::new("b", None, 100.0, v(50.0, 50.0), 0.0), Bone::new("c", Some("b"), 50.0, v(100.0, 0.0), 0.0)];
    Editor::new(p).unwrap()
}

#[test]
fn bone_tip_rotates_and_stretches_head_moves_and_keys_are_one_undo() {
    let mut ed = bone_project();
    let before = ed.doc().clone();
    // tip at (150,50); pull it to (150,150): 45 degrees, length 100 -> 141.42
    let r = req(&[Target::bone("b")], Handle::BoneTip, v(150.0, 50.0), v(0.0, 100.0));
    let out = commit(&mut ed, &r).unwrap();
    assert!(out.edits.iter().any(|e| e.prop == "bone_angle" && near(e.to, 45.0)), "{:?}", out.edits);
    assert!(out.edits.iter().any(|e| e.prop == "bone_scale" && near(e.to, 2f32.sqrt())));
    let f = ed.engine().evaluate(0.0);
    let (h, t) = bone_ends(ed.engine(), &f, "b").unwrap();
    assert!(h.distance(v(50.0, 50.0)) < 1e-2 && t.distance(v(150.0, 150.0)) < 1e-2, "{h:?} {t:?}");
    assert!(ed.undo() && ed.doc() == &before && !ed.can_undo());
    // constrain: rotate only, snapped to 15 degree steps, no stretch key
    let mut r = req(&[Target::bone("b")], Handle::BoneTip, v(150.0, 50.0), v(0.0, 60.0));
    r.modifiers.constrain = true;
    let out = commit(&mut ed, &r).unwrap();
    assert!(out.edits.iter().all(|e| e.prop != "bone_scale"), "{:?}", out.edits);
    let ang = out.edits.iter().find(|e| e.prop == "bone_angle").unwrap().to;
    assert!(near(ang, 30.0), "atan(60/100)=31 deg snaps to 30: {ang}");
    ed.undo();
    // head moves the whole bone: offsets keyed, angle untouched
    let r = req(&[Target::bone("b")], Handle::BoneHead, v(50.0, 50.0), v(10.0, -5.0));
    let out = commit(&mut ed, &r).unwrap();
    assert!(out.edits.iter().all(|e| e.prop != "bone_angle"), "{:?}", out.edits);
    let f = ed.engine().evaluate(0.0);
    let (h, _) = bone_ends(ed.engine(), &f, "b").unwrap();
    assert!(h.distance(v(60.0, 45.0)) < 1e-2, "{h:?}");
    // the child bone follows (it is attached to the parent's tip)
    let (ch, _) = bone_ends(ed.engine(), &f, "c").unwrap();
    assert!(ch.distance(v(160.0, 45.0)) < 1e-2, "{ch:?}");
}

#[test]
fn child_bone_in_a_rotated_parent_and_ik_refusal() {
    let mut ed = bone_project();
    ed.set_key("b", Prop::BoneAngle, 0.0, 90.0, Ease::Linear).unwrap();
    // b now points down; c hangs from its tip at (50,150), pointing down
    let f = ed.engine().evaluate(0.0);
    let (ch, ct) = bone_ends(ed.engine(), &f, "c").unwrap();
    assert!(ch.distance(v(50.0, 150.0)) < 1e-2 && ct.distance(v(50.0, 200.0)) < 1e-2, "{ch:?} {ct:?}");
    let r = req(&[Target::bone("c")], Handle::BoneTip, ct, v(50.0, 0.0)); // swing tip to (100,200)
    commit(&mut ed, &r).unwrap();
    let f = ed.engine().evaluate(0.0);
    let (_, ct2) = bone_ends(ed.engine(), &f, "c").unwrap();
    assert!(ct2.distance(v(100.0, 200.0)) < 1e-2, "{ct2:?}");

    let mut p = ed.project().clone();
    p.ik = vec![IkChain::two_bone("arm", "b", "c", "t", 1.0)];
    p.bones.push(Bone::new("t", None, 5.0, v(200.0, 100.0), 0.0));
    let mut ed2 = Editor::new(p).unwrap();
    let e = commit(&mut ed2, &req(&[Target::bone("b")], Handle::BoneHead, v(50.0, 50.0), v(5.0, 5.0))).unwrap_err();
    assert!(e.join(" ").contains("IK"), "{e:?}");
    // the IK target itself is free to move
    commit(&mut ed2, &req(&[Target::bone("t")], Handle::BoneHead, v(200.0, 100.0), v(5.0, 5.0))).unwrap();
}

#[test]
fn point_drags_static_keyed_and_deformer_points() {
    let mut layer = rect("a", 10.0, 10.0, 100.0, 100.0);
    layer.deformers.push(Deformer::Cage { name: "cg".into(), rest: vec![v(0.0, 0.0), v(200.0, 0.0), v(200.0, 200.0), v(0.0, 200.0)], offsets: vec![] });
    let mut ed = editor(vec![layer]);
    let pt = |k: PointKind| Target::Point { layer: "a".into(), point: k };
    // static anchor move (handles, if any, travel with it)
    let a0 = pt(PointKind::Anchor { path: 0, index: 0 });
    commit(&mut ed, &req(std::slice::from_ref(&a0), Handle::Move, v(10.0, 10.0), v(7.0, 3.0))).unwrap();
    let LayerKind::Vector { paths } = &ed.project().layers[0].kind else { panic!() };
    assert!(paths[0].anchors[0].pos.distance(v(17.0, 13.0)) < 1e-3);
    assert!(ed.undo());
    // auto-key: PointDx/PointDy channels created at the frame
    let mut r = req(std::slice::from_ref(&a0), Handle::Move, v(10.0, 10.0), v(7.0, 3.0));
    r.auto_key = true;
    r.frame = 4.0;
    commit(&mut ed, &r).unwrap();
    assert!(ed.project().channels.iter().any(|c| c.prop == Prop::PointDx && c.target == "a::r::0"));
    let f = ed.engine().evaluate(4.0);
    assert!(f.nodes[0].paths[0].anchors[0].pos.distance(v(17.0, 13.0)) < 1e-3);
    ed.undo();
    // cage point: static offsets array is created in full
    let c1 = pt(PointKind::Cage { deformer: 0, index: 1 });
    commit(&mut ed, &req(std::slice::from_ref(&c1), Handle::Move, v(200.0, 0.0), v(0.0, 12.0))).unwrap();
    let Deformer::Cage { offsets, .. } = &ed.project().layers[0].deformers[0] else { panic!() };
    assert_eq!(offsets.len(), 4);
    assert!(offsets[1].distance(v(0.0, 12.0)) < 1e-3 && offsets[0].length() < 1e-6);
    ed.undo();
    let mut r = req(&[c1], Handle::Move, v(200.0, 0.0), v(0.0, 12.0));
    r.auto_key = true;
    commit(&mut ed, &r).unwrap();
    assert!(ed.project().channels.iter().any(|c| c.prop == Prop::CageDy && c.target == "a::cg::1"));
    // points cannot be mixed with layers, nor rotated
    assert!(commit(&mut ed, &req(&[a0.clone(), l("a")], Handle::Move, v(0.0, 0.0), v(1.0, 1.0))).is_err());
    assert!(commit(&mut ed, &req(&[a0], Handle::Rotate, v(0.0, 0.0), v(1.0, 1.0))).is_err());
}

#[test]
fn nudge_and_delete_are_single_undo_steps() {
    let mut p = Project::new("t", 400.0, 300.0, 24.0, 11);
    p.layers = vec![rect("a", 0.0, 0.0, 10.0, 10.0), rect("b", 20.0, 0.0, 10.0, 10.0), rect("c", 40.0, 0.0, 10.0, 10.0)];
    p.channels = vec![Channel::keys("b", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 5.0, Ease::Linear)])), Channel::keys("c", Prop::LayerY, Track::with_keys(&[(0.0, 0.0, Ease::Linear)]))];
    let mut ed = Editor::new(p).unwrap();
    let before = ed.doc().clone();
    nudge(&mut ed, 0.0, &[l("a")], v(1.0, 0.0), false, &[]).unwrap();
    assert!(near(ed.project().layers[0].transform.position.x, 1.0));
    assert!(ed.undo() && ed.doc() == &before);
    delete(&mut ed, &[l("b"), l("a")]).unwrap();
    assert_eq!(ed.project().layers.len(), 1);
    assert_eq!(ed.project().layers[0].name, "c");
    assert_eq!(ed.project().channels.len(), 1, "channels of deleted layers go too");
    assert!(ed.undo() && ed.doc() == &before, "delete is one step");
    assert!(delete(&mut ed, &[]).is_err());
    assert!(delete(&mut ed, &[Target::bone("x")]).is_err());
}

#[test]
fn deleting_a_layer_that_is_still_referenced_is_rejected_and_rolled_back() {
    let mut p = Project::new("t", 400.0, 300.0, 24.0, 11);
    let mut m = rect("m", 0.0, 0.0, 10.0, 10.0);
    m.mask = Some(crate::project::MaskRef { layer: "a".into(), invert: false });
    p.layers = vec![rect("a", 0.0, 0.0, 10.0, 10.0), m];
    let mut ed = Editor::new(p).unwrap();
    let before = ed.doc().clone();
    assert!(delete(&mut ed, &[l("a")]).is_err());
    assert_eq!(ed.doc(), &before);
}

#[test]
fn handle_hit_testing_priorities_and_cursors() {
    let bx = BoxFrame::from_aabb(v(0.0, 0.0), v(100.0, 100.0));
    assert_eq!(hit_handle(&bx, v(100.0, 100.0), 6.0), Some(Handle::Scale { at: ScaleHandle::Se }));
    assert_eq!(hit_handle(&bx, v(50.0, 1.0), 6.0), Some(Handle::Scale { at: ScaleHandle::N }));
    assert_eq!(hit_handle(&bx, v(50.0, 50.0), 6.0), Some(Handle::Move));
    assert_eq!(hit_handle(&bx, v(50.0, -ROTATE_OFFSET), 6.0), Some(Handle::Rotate));
    assert_eq!(hit_handle(&bx, v(150.0, 50.0), 6.0), None);
    // tiny box: corners win over edges
    let tiny = BoxFrame::from_aabb(v(0.0, 0.0), v(4.0, 4.0));
    assert_eq!(hit_handle(&tiny, v(0.0, 0.0), 6.0), Some(Handle::Scale { at: ScaleHandle::Nw }));
    let hs = box_handles(&bx);
    let cur = |h: ScaleHandle| hs.iter().find(|x| x.handle == Handle::Scale { at: h }).unwrap().cursor.clone();
    assert_eq!((cur(ScaleHandle::N).as_str(), cur(ScaleHandle::E).as_str(), cur(ScaleHandle::Se).as_str(), cur(ScaleHandle::Ne).as_str()), ("ns-resize", "ew-resize", "nwse-resize", "nesw-resize"));
    assert_eq!(hs.len(), 9);
    // a rotated box rotates the cursor classes
    let rot = BoxFrame { origin: v(0.0, 0.0), u: v(0.0, 100.0), v: v(-100.0, 0.0) };
    let hs = box_handles(&rot);
    assert_eq!(hs.iter().find(|x| x.handle == Handle::Scale { at: ScaleHandle::E }).unwrap().cursor, "ns-resize");
}

#[test]
fn gizmo_view_kinds() {
    let ed = editor(vec![rect("a", 0.0, 0.0, 10.0, 10.0)]);
    let f = ed.engine().evaluate(0.0);
    assert_eq!(gizmo_view(ed.engine(), &f, &Selection::new()).kind, "none");
    let g = gizmo_view(ed.engine(), &f, &Selection::from_items(vec![l("a")]));
    assert_eq!((g.kind.as_str(), g.handles.len()), ("box", 9));
    let g = gizmo_view(ed.engine(), &f, &Selection::from_items(vec![Target::Point { layer: "a".into(), point: PointKind::Anchor { path: 0, index: 0 } }]));
    assert_eq!((g.kind.as_str(), g.points.len()), ("points", 1));
    let b = bone_project();
    let fb = b.engine().evaluate(0.0);
    let g = gizmo_view(b.engine(), &fb, &Selection::from_items(vec![Target::bone("b")]));
    assert_eq!((g.kind.as_str(), g.handles.len()), ("bone", 2));
}

#[test]
fn preview_does_not_touch_the_project_and_is_deterministic() {
    let mut ed = editor(vec![rect("a", 0.0, 0.0, 100.0, 100.0), rect("b", 150.0, 0.0, 50.0, 50.0)]);
    let before = ed.doc().clone();
    let mut r = req(&[l("a")], Handle::Move, v(10.0, 10.0), v(47.0, 3.0));
    r.modifiers.no_snap = false;
    let p1 = serde_json::to_string(&preview(&ed, &r).unwrap()).unwrap();
    let p2 = serde_json::to_string(&preview(&ed, &r).unwrap()).unwrap();
    assert_eq!(p1, p2);
    assert_eq!(ed.doc(), &before);
    assert!(!ed.can_undo());
    let pv = preview(&ed, &r).unwrap();
    assert!(!pv.outline.is_empty() && pv.box_after.is_some() && pv.changed);
    let c1 = commit(&mut ed, &r).unwrap();
    assert_eq!(serde_json::to_string(&c1).unwrap(), p1, "the commit does what the preview showed");
}

#[test]
fn requests_deserialize_from_api_json() {
    let j = json!({"frame": 2, "selection": [{"kind":"layer","path":"a"}], "handle": {"type":"scale","at":"se"}, "start": [1,2], "delta": [3,4], "modifiers": {"constrain": true}});
    let r: DragRequest = serde_json::from_value(j).unwrap();
    assert_eq!(r.handle, Handle::Scale { at: ScaleHandle::Se });
    assert!(r.modifiers.constrain && !r.modifiers.center && !r.auto_key && r.snap.objects);
    let j = json!({"frame": 0, "selection": [], "handle": {"type":"nonsense"}, "start": [0,0], "delta": [0,0]});
    assert!(serde_json::from_value::<DragRequest>(j).is_err());
}

#[test]
fn a_locked_layer_refuses_drags_nudges_point_edits_and_deletes_until_unlocked() {
    let mut ed = editor(vec![rect("a", 0.0, 0.0, 10.0, 10.0), rect("b", 50.0, 0.0, 10.0, 10.0)]);
    ed.apply(Op::Set { path: "/layers/0/locked".into(), value: serde_json::json!(true) }).unwrap();
    let before = ed.doc().clone();
    let e = commit(&mut ed, &req(&[l("a"), l("b")], Handle::Move, v(1.0, 1.0), v(5.0, 5.0))).unwrap_err();
    assert!(e.join(" ").contains("'a' is locked"), "{e:?}");
    assert!(nudge(&mut ed, 0.0, &[l("a")], v(1.0, 0.0), false, &[]).unwrap_err().join(" ").contains("locked"));
    assert!(delete(&mut ed, &[l("a")]).unwrap_err().join(" ").contains("locked"));
    let pt = Target::Point { layer: "a".into(), point: crate::pick::PointKind::Anchor { path: 0, index: 0 } };
    assert!(nudge(&mut ed, 0.0, &[pt], v(1.0, 0.0), false, &[]).unwrap_err().join(" ").contains("locked"));
    assert_eq!(ed.doc(), &before);
    // the unlocked neighbour is fine, and unlocking (a plain Op, as the panel does) allows the edit again
    nudge(&mut ed, 0.0, &[l("b")], v(1.0, 0.0), false, &[]).unwrap();
    ed.apply(Op::Set { path: "/layers/0/locked".into(), value: serde_json::json!(false) }).unwrap();
    nudge(&mut ed, 0.0, &[l("a")], v(1.0, 0.0), false, &[]).unwrap();
    delete(&mut ed, &[l("a")]).unwrap();
    // a locked group protects its child
    let mut ed2 = editor(vec![Layer::group("g", vec![rect("c", 0.0, 0.0, 10.0, 10.0)])]);
    ed2.apply(Op::Set { path: "/layers/0/locked".into(), value: serde_json::json!(true) }).unwrap();
    assert!(nudge(&mut ed2, 0.0, &[l("g/c")], v(1.0, 0.0), false, &[]).is_err());
}
