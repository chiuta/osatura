// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `timeline`.
#![allow(unused_imports)]

use super::*;
use crate::edit::Editor;
use crate::math::Vec2;
use crate::project::Bone;

fn lin(k: &[(f32, f32)]) -> Track {
    Track::with_keys(&k.iter().map(|&(f, v)| (f, v, Ease::Linear)).collect::<Vec<_>>())
}

/// Rig: root -> arm bones with channels, a layer, a camera.
fn proj() -> Project {
    let mut p = Project::new("t", 100.0, 100.0, 24.0, 48);
    p.bones = vec![
        Bone::new("root", None, 10.0, Vec2::ZERO, 0.0),
        Bone::new("arm", Some("root"), 10.0, Vec2::ZERO, 0.0),
        Bone::new("arm_r", Some("root"), 10.0, Vec2::ZERO, 0.0),
    ];
    p.channels = vec![
        Channel::keys("arm", Prop::BoneAngle, lin(&[(0.0, 0.0), (12.0, 30.0), (24.0, 60.0)])),
        Channel::keys("arm", Prop::BoneScale, lin(&[(0.0, 1.0), (24.0, 2.0)])),
        Channel::keys("root", Prop::BoneOffsetX, lin(&[(0.0, 0.0), (48.0, 10.0)])),
        Channel::expr("arm_r", Prop::BoneAngle, "0", Mode::Add),
        Channel::keys("", Prop::CameraZoom, lin(&[(0.0, 1.0), (48.0, 1.5)])),
    ];
    p
}

fn kr(ch: usize, frame: f32) -> KeyRef {
    KeyRef { ch, frame }
}

fn keys_of_ch(p: &Project, ch: usize) -> Vec<(f32, f32)> {
    keys_of(&p.channels[ch]).unwrap().keys.iter().map(|k| (k.frame, k.value)).collect()
}

fn run(e: &mut Editor, op: TlOp) -> EditPlan {
    apply(e, &op).unwrap()
}

#[test]
fn model_groups_hierarchically_with_metadata() {
    let mut p = proj();
    chanmeta::set_meta(&mut p, "arm", Some(Prop::BoneAngle), &chanmeta::Change { lock: Some(true), color: Some("#ff8800".into()), ..Default::default() }).unwrap();
    chanmeta::set_meta(&mut p, "root", None, &chanmeta::Change { shy: Some(true), ..Default::default() }).unwrap();
    let m = build(&p);
    assert_eq!(m.tracks.len(), 5);
    assert!(m.tracks.iter().enumerate().all(|(i, t)| t.ch == i));
    let labels: Vec<_> = m.groups.iter().map(|g| (g.label.as_str(), g.depth, g.parent)).collect();
    // Bones > root > arm / arm_r ; Camera
    assert_eq!(labels[0], ("Bones", 0, None));
    assert!(labels.contains(&("root", 1, Some(0))));
    assert!(labels.iter().any(|l| l.0 == "arm" && l.1 == 2));
    assert!(labels.iter().any(|l| l.0 == "arm_r" && l.1 == 2));
    assert!(labels.iter().any(|l| l.0 == "Camera" && l.1 == 0));
    let arm = &m.tracks[0];
    assert!(arm.locked && arm.editable && arm.color.as_deref() == Some("#ff8800"));
    assert!(!m.tracks[1].locked, "lock is per prop");
    assert!(m.tracks[2].shy, "target-wide shy");
    assert_eq!(m.tracks[3].source, "expr");
    assert!(!m.tracks[3].editable && m.tracks[3].keys.is_empty());
    assert_eq!(m.tracks[0].keys.len(), 3);
    assert_eq!(m.tracks[0].label, "bone_angle");
    // group summary: the Bones header sees every bone key frame
    assert_eq!(m.groups[0].frames, vec![0.0, 12.0, 24.0, 48.0]);
    // depth-first rows: header, root's own track, then children
    assert_eq!(m.rows[0], Row::Group(0));
    assert_eq!(m.rows.len(), m.groups.len() + m.tracks.len());
    let ri = m.rows.iter().position(|r| *r == Row::Track(2)).unwrap();
    let ai = m.rows.iter().position(|r| *r == Row::Track(0)).unwrap();
    assert!(ri < ai, "parent's tracks come before the child bone's");
    // deterministic
    assert_eq!(serde_json::to_string(&build(&p)).unwrap(), serde_json::to_string(&build(&p)).unwrap());
}

#[test]
fn model_group_lock_summary_and_layer_paths() {
    let mut p = proj();
    chanmeta::set_meta(&mut p, "arm", None, &chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
    let m = build(&p);
    let arm_g = m.groups.iter().find(|g| g.label == "arm").unwrap();
    assert!(arm_g.locked);
    assert!(!m.groups.iter().find(|g| g.label == "Bones").unwrap().locked);
    // layer path "grp/leaf" nests
    let mut q = Project::new("l", 10.0, 10.0, 24.0, 10);
    q.channels = vec![Channel::keys("grp/leaf", Prop::LayerX, lin(&[(0.0, 0.0), (5.0, 1.0)])), Channel::keys("grp", Prop::LayerOpacity, lin(&[(0.0, 1.0)]))];
    let m = build(&q);
    let leaf = m.groups.iter().find(|g| g.label == "leaf").unwrap();
    assert_eq!(m.groups[leaf.parent.unwrap()].label, "grp");
    assert_eq!(m.tracks[1].group, leaf.parent.unwrap());
    // sub targets: layer::deformer::idx
    let mut s = Project::new("s", 10.0, 10.0, 24.0, 10);
    s.channels = vec![Channel::keys("L::lat::1,2", Prop::LatticeDx, lin(&[(0.0, 0.0)]))];
    let m = build(&s);
    assert_eq!(m.tracks[0].label, "1,2 lattice_dx");
    assert_eq!(m.groups[m.tracks[0].group].label, "lat");
}

#[test]
fn selection_box_toggle_channel_and_prune() {
    let p = proj();
    let m = build(&p);
    let ri = m.rows.iter().position(|r| *r == Row::Track(0)).unwrap();
    let b = select_box(&m, 10.0, 30.0, ri, ri, false);
    assert_eq!(b, vec![kr(0, 12.0), kr(0, 24.0)]);
    // reversed arguments, group row selects descendants (bones header covers channels 0,1,2)
    let all = select_box(&m, 100.0, -5.0, 0, ri, false);
    assert!(all.contains(&kr(0, 0.0)) && all.contains(&kr(1, 24.0)) && all.contains(&kr(2, 48.0)) && !all.iter().any(|k| k.ch == 4));
    assert_eq!(select_channel(&m, 1, false), vec![kr(1, 0.0), kr(1, 24.0)]);
    assert_eq!(select_all(&m, false).len(), 3 + 2 + 2 + 2);
    let g = m.tracks[0].group;
    assert_eq!(select_group(&m, g, false).len(), 5, "arm: three angle keys and two scale keys");
    assert_eq!(select_group(&m, m.tracks[2].group, false).len(), 7, "root includes its child bones");
    // modes
    let a = vec![kr(0, 0.0), kr(0, 12.0)];
    let h = vec![kr(0, 12.0), kr(1, 0.0)];
    assert_eq!(select_apply(&a, &h, SelMode::Replace), vec![kr(0, 12.0), kr(1, 0.0)]);
    assert_eq!(select_apply(&a, &h, SelMode::Add), vec![kr(0, 0.0), kr(0, 12.0), kr(1, 0.0)]);
    assert_eq!(select_apply(&a, &h, SelMode::Toggle), vec![kr(0, 0.0), kr(1, 0.0)]);
    assert_eq!(select_apply(&a, &h, SelMode::Subtract), vec![kr(0, 0.0)]);
    assert_eq!(SelMode::parse("toggle"), Some(SelMode::Toggle));
    assert_eq!(prune(&m, &[kr(0, 12.0), kr(0, 13.0), kr(9, 0.0)]), vec![kr(0, 12.0)]);
    // locked channels are skipped by default
    let mut q = proj();
    chanmeta::set_meta(&mut q, "arm", Some(Prop::BoneAngle), &chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
    let mq = build(&q);
    assert!(select_channel(&mq, 0, false).is_empty());
    assert_eq!(select_channel(&mq, 0, true).len(), 3);
}

#[test]
fn move_keys_exact_snap_bounds_and_undo() {
    let mut e = Editor::new(proj()).unwrap();
    let before = e.doc().clone();
    let pl = run(&mut e, TlOp::MoveKeys { keys: vec![kr(0, 12.0), kr(0, 24.0)], dt: 3.4, dv: 5.0, opts: EditOpts::default() });
    assert_eq!(keys_of_ch(e.project(), 0), vec![(0.0, 0.0), (15.0, 35.0), (27.0, 65.0)], "3.4 snaps to 3");
    assert_eq!(pl.selection, vec![kr(0, 15.0), kr(0, 27.0)]);
    assert_eq!(pl.changed, 2);
    assert!(e.undo());
    assert_eq!(e.doc(), &before, "undo restores the document exactly");
    // no snap keeps fractions
    let o = EditOpts { snap: false, ..Default::default() };
    run(&mut e, TlOp::MoveKeys { keys: vec![kr(0, 12.0)], dt: 0.5, dv: 0.0, opts: o });
    assert_eq!(keys_of_ch(e.project(), 0)[1], (12.5, 30.0));
    e.undo();
    // value only: frames are not snapped
    let o = EditOpts { snap: true, ..Default::default() };
    let pl = move_keys(e.project(), &[kr(0, 12.0)], 0.0, 1.0, &o).unwrap();
    assert_eq!(pl.selection, vec![kr(0, 12.0)]);
    // bounds
    let err = move_keys(e.project(), &[kr(0, 24.0)], 30.0, 0.0, &o).unwrap_err();
    assert!(err.contains("outside"), "{err}");
    assert!(move_keys(e.project(), &[kr(0, 0.0)], -1.0, 0.0, &o).is_err());
    let free = EditOpts { bounds: Bounds::Free, ..Default::default() };
    assert!(move_keys(e.project(), &[kr(0, 0.0)], -1.0, 0.0, &free).is_ok());
    // zero move is a no-op without an op
    let z = move_keys(e.project(), &[kr(0, 0.0)], 0.0, 0.0, &o).unwrap();
    assert!(z.op.is_none());
    // validation of inputs
    assert!(move_keys(e.project(), &[kr(0, 0.0)], f32::NAN, 0.0, &o).is_err());
    assert!(move_keys(e.project(), &[], 1.0, 0.0, &o).unwrap_err().contains("no keys"));
    assert!(move_keys(e.project(), &[kr(0, 7.0)], 1.0, 0.0, &o).unwrap_err().contains("stale"));
    assert!(move_keys(e.project(), &[kr(99, 0.0)], 1.0, 0.0, &o).is_err());
    assert!(move_keys(e.project(), &[kr(3, 0.0)], 1.0, 0.0, &o).unwrap_err().contains("not an editable keyed"), "expression channel");
}

#[test]
fn collision_modes_replace_merge_refuse() {
    let p = proj();
    let sel = [kr(0, 0.0)];
    // move key 0 onto key 12
    let rep = move_keys(&p, &sel, 12.0, 0.0, &EditOpts::default()).unwrap();
    let mut e = Editor::new(p.clone()).unwrap();
    e.apply(rep.op.unwrap()).unwrap();
    assert_eq!(keys_of_ch(e.project(), 0), vec![(12.0, 0.0), (24.0, 60.0)], "moved key replaced the one at 12");
    let merge = move_keys(&p, &sel, 12.0, 0.0, &EditOpts { collision: Collision::Merge, ..Default::default() }).unwrap();
    let mut e = Editor::new(p.clone()).unwrap();
    e.apply(merge.op.clone().unwrap()).unwrap();
    assert_eq!(keys_of_ch(e.project(), 0), vec![(12.0, 30.0), (24.0, 60.0)], "existing key kept, incoming dropped");
    assert!(merge.notes.iter().any(|n| n.contains("kept it")));
    assert!(merge.selection.is_empty());
    let r = move_keys(&p, &sel, 12.0, 0.0, &EditOpts { collision: Collision::Refuse, ..Default::default() }).unwrap_err();
    assert!(r.contains("already exists"), "{r}");
    // two moved keys landing on one frame: scale 0.01 about 0 with snap collapses 0,12 -> 0,0
    let sc = scale_keys(&p, &[kr(0, 0.0), kr(0, 12.0)], 0.0, 0.01, &EditOpts::default()).unwrap();
    assert!(sc.notes.iter().any(|n| n.contains("earlier one was kept")));
    assert!(scale_keys(&p, &[kr(0, 0.0), kr(0, 12.0)], 0.0, 0.01, &EditOpts { collision: Collision::Refuse, ..Default::default() }).is_err());
}

#[test]
fn scale_about_pivot_is_exact() {
    let mut e = Editor::new(proj()).unwrap();
    run(&mut e, TlOp::ScaleKeys { keys: vec![kr(0, 12.0), kr(0, 24.0)], pivot: 12.0, factor: 0.5, opts: EditOpts::default() });
    assert_eq!(keys_of_ch(e.project(), 0), vec![(0.0, 0.0), (12.0, 30.0), (18.0, 60.0)]);
    assert!(scale_keys(e.project(), &[kr(0, 0.0)], 0.0, 0.0, &EditOpts::default()).is_err());
    assert!(scale_keys(e.project(), &[kr(0, 0.0)], 0.0, f32::INFINITY, &EditOpts::default()).is_err());
    assert!(scale_keys(e.project(), &[kr(0, 0.0)], f32::NAN, 2.0, &EditOpts::default()).is_err());
    // scaling past the end is refused with project bounds
    assert!(scale_keys(e.project(), &[kr(0, 18.0)], 0.0, 10.0, &EditOpts::default()).is_err());
}

#[test]
fn copy_paste_with_remap_and_new_channel() {
    let mut e = Editor::new(proj()).unwrap();
    let before = e.doc().clone();
    let clip = copy_keys(e.project(), &[kr(0, 12.0), kr(0, 24.0), kr(1, 24.0)]).unwrap();
    assert_eq!(clip.tracks.len(), 2);
    assert_eq!(clip.tracks[0].keys.iter().map(|k| k.frame).collect::<Vec<_>>(), vec![0.0, 12.0]);
    assert_eq!(clip.tracks[1].keys[0].frame, 12.0, "relative to the earliest copied key");
    // identity paste at 30 lands inside the same channel
    let pl = run(&mut e, TlOp::Paste { clip: clip.clone(), at: 30.0, remap: vec![], opts: EditOpts::default() });
    assert_eq!(keys_of_ch(e.project(), 0), vec![(0.0, 0.0), (12.0, 30.0), (24.0, 60.0), (30.0, 30.0), (42.0, 60.0)]);
    assert_eq!(pl.changed, 3);
    e.undo();
    assert_eq!(e.doc(), &before);
    // remap arm.angle -> arm_r.angle (no keyed channel there: a new channel is created, selection points at it)
    let rm = Remap { from_target: "arm".into(), from_prop: Prop::BoneAngle, to_target: "arm_r".into(), to_prop: Prop::BoneAngle };
    let n = e.project().channels.len();
    let pl = run(&mut e, TlOp::Paste { clip: clip.clone(), at: 0.0, remap: vec![rm], opts: EditOpts::default() });
    assert_eq!(e.project().channels.len(), n + 1);
    let c = &e.project().channels[n];
    assert_eq!((c.target.as_str(), c.prop), ("arm_r", Prop::BoneAngle));
    assert_eq!(keys_of_ch(e.project(), n), vec![(0.0, 30.0), (12.0, 60.0)]);
    assert!(pl.selection.contains(&kr(n, 0.0)) && pl.selection.contains(&kr(n, 12.0)) && pl.selection.contains(&kr(1, 12.0)));
    e.undo();
    assert_eq!(e.project().channels.len(), n);
    // retarget helper
    let mut c2 = clip.clone();
    assert_eq!(c2.retarget("arm", "arm_r"), 2);
    // errors
    let bad = Remap { from_target: "nope".into(), from_prop: Prop::BoneAngle, to_target: "arm".into(), to_prop: Prop::BoneAngle };
    assert!(paste_clip(e.project(), &clip, 0.0, &[bad], &EditOpts::default()).is_err());
    let kind = Remap { from_target: "arm".into(), from_prop: Prop::BoneAngle, to_target: "arm".into(), to_prop: Prop::LayerX };
    assert!(paste_clip(e.project(), &clip, 0.0, &[kind], &EditOpts::default()).unwrap_err().contains("different kinds"));
    assert!(paste_clip(e.project(), &Clip { tracks: vec![] }, 0.0, &[], &EditOpts::default()).is_err());
    assert!(paste_clip(e.project(), &clip, 40.0, &[], &EditOpts::default()).is_err(), "would run past the project end");
    let mut nan = clip.clone();
    nan.tracks[0].keys[0].value = f32::NAN;
    assert!(paste_clip(e.project(), &nan, 0.0, &[], &EditOpts::default()).is_err());
    // merge mode on paste keeps existing keys
    let m = paste_clip(e.project(), &clip, 12.0, &[], &EditOpts { collision: Collision::Merge, ..Default::default() }).unwrap();
    assert_eq!(m.changed, 0);
    assert!(m.op.is_none());
}

#[test]
fn duplicate_delete_ease_value_and_add_key() {
    let mut e = Editor::new(proj()).unwrap();
    let before = e.doc().clone();
    run(&mut e, TlOp::Duplicate { keys: vec![kr(0, 0.0), kr(0, 12.0)], offset: 30.0, opts: EditOpts::default() });
    assert_eq!(keys_of_ch(e.project(), 0), vec![(0.0, 0.0), (12.0, 30.0), (24.0, 60.0), (30.0, 0.0), (42.0, 30.0)]);
    e.undo();
    assert!(duplicate_keys(e.project(), &[kr(0, 0.0)], 0.2, &EditOpts::default()).unwrap_err().contains("zero"));
    // delete: partial and whole channel (channel is removed, indices shift, undo restores)
    run(&mut e, TlOp::Delete { keys: vec![kr(0, 12.0)] });
    assert_eq!(keys_of_ch(e.project(), 0), vec![(0.0, 0.0), (24.0, 60.0)]);
    e.undo();
    let n = e.project().channels.len();
    let pl = run(&mut e, TlOp::Delete { keys: vec![kr(1, 0.0), kr(1, 24.0), kr(2, 0.0)] });
    assert_eq!(e.project().channels.len(), n - 1);
    assert!(e.project().channels.iter().all(|c| !(c.target == "arm" && c.prop == Prop::BoneScale)));
    assert_eq!(pl.changed, 3);
    assert!(pl.selection.is_empty());
    e.undo();
    assert_eq!(e.doc(), &before);
    // ease
    let bz = Ease::CubicBezier { x1: 0.2, y1: 0.0, x2: 0.8, y2: 1.0 };
    run(&mut e, TlOp::SetEase { keys: vec![kr(0, 0.0), kr(0, 12.0)], ease: bz });
    let t = keys_of(&e.project().channels[0]).unwrap();
    assert_eq!((t.keys[0].ease, t.keys[1].ease, t.keys[2].ease), (bz, bz, Ease::Linear));
    let bad = Ease::CubicBezier { x1: 2.0, y1: 0.0, x2: 0.8, y2: 1.0 };
    assert!(set_ease(e.project(), &[kr(0, 0.0)], bad).is_err());
    // value
    run(&mut e, TlOp::SetValue { keys: vec![kr(0, 0.0), kr(0, 24.0)], value: 7.0 });
    assert_eq!(keys_of_ch(e.project(), 0), vec![(0.0, 7.0), (12.0, 30.0), (24.0, 7.0)]);
    assert!(set_value(e.project(), &[kr(0, 0.0)], f32::NAN).is_err());
    // add key: value defaults to the evaluated curve
    let pl = run(&mut e, TlOp::AddKey { target: "root".into(), prop: Prop::BoneOffsetX, frame: 24.0, value: None, ease: None, opts: EditOpts::default() });
    assert_eq!(keys_of_ch(e.project(), 2)[1], (24.0, 5.0));
    assert_eq!(pl.selection, vec![kr(2, 24.0)]);
    // and a brand-new channel needs an explicit value
    assert!(add_key(e.project(), "arm_r", Prop::BoneScale, 3.0, None, Ease::Linear, &EditOpts::default()).is_err());
    let pl = add_key(e.project(), "arm_r", Prop::BoneScale, 3.2, Some(2.0), Ease::Linear, &EditOpts::default()).unwrap();
    assert_eq!(pl.selection, vec![kr(e.project().channels.len(), 3.0)]);
    assert!(add_key(e.project(), "arm", Prop::BoneAngle, 99.0, Some(1.0), Ease::Linear, &EditOpts::default()).is_err());
}

#[test]
fn locked_channels_are_refused_atomically() {
    let mut p = proj();
    chanmeta::set_meta(&mut p, "arm", Some(Prop::BoneScale), &chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
    let mut e = Editor::new(p).unwrap();
    let before = e.doc().clone();
    // channel 0 is free, channel 1 is locked: nothing may change
    let sel = [kr(0, 12.0), kr(1, 0.0)];
    for op in [
        TlOp::MoveKeys { keys: sel.to_vec(), dt: 1.0, dv: 0.0, opts: EditOpts::default() },
        TlOp::ScaleKeys { keys: sel.to_vec(), pivot: 0.0, factor: 2.0, opts: EditOpts::default() },
        TlOp::Delete { keys: sel.to_vec() },
        TlOp::SetEase { keys: sel.to_vec(), ease: Ease::Step },
        TlOp::SetValue { keys: sel.to_vec(), value: 1.0 },
        TlOp::Duplicate { keys: sel.to_vec(), offset: 5.0, opts: EditOpts::default() },
        TlOp::Smooth { keys: sel.to_vec(), kind: TangentKind::Monotone },
        TlOp::Offset { channels: vec![1], dt: 1.0, opts: EditOpts::default() },
        TlOp::Extrapolate { channels: vec![1], extrapolate: Extrapolate::Loop },
        TlOp::MakeLoop { channels: vec![1], end: None, extrapolate: None, opts: EditOpts::default() },
        TlOp::AddKey { target: "arm".into(), prop: Prop::BoneScale, frame: 3.0, value: Some(1.0), ease: None, opts: EditOpts::default() },
        TlOp::Handle { ch: 1, frame: 0.0, side: Side::Out, t: 1.0, v: 1.0, mode: HandleMode::Free },
        TlOp::Ripple { at: 0.0, count: 2.0, mode: RetimeMode::Insert, scope: Some(vec![1]), adjust_frames: false, opts: EditOpts::default() },
    ] {
        let err = apply(&mut e, &op).unwrap_err().join(" ");
        assert!(err.contains("locked"), "{op:?}: {err}");
        assert_eq!(e.doc(), &before);
    }
    // pasting onto a locked destination is refused too
    let clip = copy_keys(e.project(), &[kr(0, 12.0)]).unwrap();
    let rm = Remap { from_target: "arm".into(), from_prop: Prop::BoneAngle, to_target: "arm".into(), to_prop: Prop::BoneScale };
    assert!(paste_clip(e.project(), &clip, 0.0, &[rm], &EditOpts::default()).unwrap_err().contains("locked"));
    assert!(!e.can_undo(), "refused edits leave no history");
}

#[test]
fn ripple_insert_delete_scope_and_frames() {
    let mut p = proj();
    chanmeta::set_meta(&mut p, "root", Some(Prop::BoneOffsetX), &chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
    let mut e = Editor::new(p).unwrap();
    let before = e.doc().clone();
    // insert 6 frames at 12 (all unlocked channels; locked root.x skipped, reported)
    let pl = run(&mut e, TlOp::Ripple { at: 12.0, count: 6.0, mode: RetimeMode::Insert, scope: None, adjust_frames: true, opts: EditOpts::default() });
    assert_eq!(keys_of_ch(e.project(), 0), vec![(0.0, 0.0), (18.0, 30.0), (30.0, 60.0)]);
    assert_eq!(keys_of_ch(e.project(), 1), vec![(0.0, 1.0), (30.0, 2.0)]);
    assert_eq!(keys_of_ch(e.project(), 2), vec![(0.0, 0.0), (48.0, 10.0)], "locked channel untouched");
    assert!(pl.notes.iter().any(|n| n.contains("skipped locked")));
    assert_eq!(e.project().frames, 54);
    assert_eq!(keys_of_ch(e.project(), 4)[1].0, 54.0);
    e.undo();
    assert_eq!(e.doc(), &before);
    // without adjust_frames the camera key at 48 would be pushed past the end
    let o = EditOpts::default();
    let err = ripple(e.project(), 12.0, 6.0, RetimeMode::Insert, None, false, &o);
    assert!(err.unwrap_err().contains("outside"));
    // free bounds allow it
    let free = EditOpts { bounds: Bounds::Free, ..Default::default() };
    let pl = ripple(e.project(), 12.0, 6.0, RetimeMode::Insert, None, false, &free).unwrap();
    e.apply(pl.op.unwrap()).unwrap();
    assert_eq!(e.project().frames, 48);
    assert_eq!(keys_of_ch(e.project(), 4)[1].0, 54.0);
    e.undo();
    // delete frames 12..24: key at 12 removed, key at 24 moves to 12, 48 -> 36
    run(&mut e, TlOp::Ripple { at: 12.0, count: 12.0, mode: RetimeMode::Delete, scope: None, adjust_frames: true, opts: EditOpts::default() });
    assert_eq!(keys_of_ch(e.project(), 0), vec![(0.0, 0.0), (12.0, 60.0)]);
    assert_eq!(e.project().frames, 36);
    assert_eq!(keys_of_ch(e.project(), 4)[1].0, 36.0);
    e.undo();
    // errors
    assert!(ripple(e.project(), 0.0, 0.0, RetimeMode::Insert, None, false, &o).is_err());
    assert!(ripple(e.project(), 0.0, 48.0, RetimeMode::Delete, None, true, &o).unwrap_err().contains("no frames"));
    assert!(ripple(e.project(), f32::NAN, 2.0, RetimeMode::Insert, None, false, &o).is_err());
    // explicit scope: only listed channels
    run(&mut e, TlOp::Ripple { at: 0.0, count: 2.0, mode: RetimeMode::Insert, scope: Some(vec![1]), adjust_frames: false, opts: EditOpts::default() });
    assert_eq!(keys_of_ch(e.project(), 1), vec![(2.0, 1.0), (26.0, 2.0)]);
    assert_eq!(keys_of_ch(e.project(), 0)[0].0, 0.0);
}

#[test]
fn offset_extrapolate_and_loop_helpers() {
    let mut e = Editor::new(proj()).unwrap();
    run(&mut e, TlOp::Offset { channels: vec![0], dt: 4.0, opts: EditOpts::default() });
    assert_eq!(keys_of_ch(e.project(), 0).iter().map(|k| k.0).collect::<Vec<_>>(), vec![4.0, 16.0, 28.0]);
    assert!(offset_channels(e.project(), &[0], 30.0, &EditOpts::default()).is_err());
    assert!(offset_channels(e.project(), &[], 1.0, &EditOpts::default()).is_err());
    e.undo();
    run(&mut e, TlOp::Extrapolate { channels: vec![0, 1], extrapolate: Extrapolate::PingPong });
    assert_eq!(keys_of(&e.project().channels[1]).unwrap().extrapolate, Extrapolate::PingPong);
    // make_loop closes the cycle with the first key's value
    let pl = run(&mut e, TlOp::MakeLoop { channels: vec![0], end: Some(48.0), extrapolate: None, opts: EditOpts::default() });
    let t = keys_of(&e.project().channels[0]).unwrap();
    assert_eq!(t.keys.last().map(|k| (k.frame, k.value)), Some((48.0, 0.0)));
    assert_eq!(t.extrapolate, Extrapolate::Loop);
    assert_eq!(pl.selection, vec![kr(0, 48.0)]);
    assert!((t.eval(50.0).unwrap() - t.eval(2.0).unwrap()).abs() < 1e-4, "looped");
    assert!(make_loop(e.project(), &[0], Some(-1.0), Extrapolate::Loop, &EditOpts::default()).is_err());
    assert!(make_loop(e.project(), &[0], Some(10.0), Extrapolate::Loop, &EditOpts::default()).is_err(), "before the last key");
    let one = Project { channels: vec![Channel::keys("arm", Prop::BoneAngle, lin(&[(0.0, 1.0)]))], ..proj() };
    assert!(make_loop(&one, &[0], None, Extrapolate::Loop, &EditOpts::default()).unwrap_err().contains("two keys"));
}

#[test]
fn handle_and_smooth_ops_go_through_the_editor() {
    let mut e = Editor::new(proj()).unwrap();
    let before = e.doc().clone();
    run(&mut e, TlOp::Handle { ch: 0, frame: 0.0, side: Side::Out, t: 6.0, v: 0.0, mode: HandleMode::Free });
    match keys_of(&e.project().channels[0]).unwrap().keys[0].ease {
        Ease::CubicBezier { x1, y1, .. } => assert!((x1 - 0.5).abs() < 1e-6 && y1.abs() < 1e-6),
        x => panic!("{x:?}"),
    }
    e.undo();
    assert_eq!(e.doc(), &before);
    assert!(move_handle(e.project(), 0, 5.0, Side::Out, HandlePos { t: 1.0, v: 1.0 }, HandleMode::Free).is_err());
    assert!(move_handle(e.project(), 0, 24.0, Side::Out, HandlePos { t: 1.0, v: 1.0 }, HandleMode::Free).is_err());
    run(&mut e, TlOp::Smooth { keys: vec![kr(0, 0.0), kr(0, 12.0)], kind: TangentKind::Monotone });
    let t = keys_of(&e.project().channels[0]).unwrap();
    assert!(matches!(t.keys[0].ease, Ease::CubicBezier { .. }) && matches!(t.keys[1].ease, Ease::CubicBezier { .. }));
    assert!(matches!(t.keys[2].ease, Ease::Linear));
    // collinear data stays a straight line under monotone smoothing
    assert!((t.eval(6.0).unwrap() - 15.0).abs() < 0.05);
}

#[test]
fn flags_op_sets_lock_and_undoes() {
    let mut e = Editor::new(proj()).unwrap();
    let before = e.doc().clone();
    let pl = run(&mut e, TlOp::Flags { target: "arm".into(), prop: Some(Prop::BoneAngle), lock: Some(true), shy: None, solo: None, color: Some("#00ff00".into()) });
    assert_eq!(pl.changed, 1);
    assert!(chanmeta::is_locked(e.project(), "arm", Prop::BoneAngle));
    assert!(build(e.project()).tracks[0].locked);
    assert!(apply(&mut e, &TlOp::Delete { keys: vec![kr(0, 0.0)] }).is_err());
    e.undo();
    assert_eq!(e.doc(), &before);
    assert!(set_flags(e.project(), "ghost", None, &chanmeta::Change { lock: Some(true), ..Default::default() }).is_err());
    assert!(set_flags(e.project(), "arm", None, &chanmeta::Change { color: Some("not a colour".into()), ..Default::default() }).is_err());
}

#[test]
fn ops_parse_from_json_and_copy_is_not_an_edit() {
    let j = r#"{"op":"move_keys","keys":[{"ch":0,"frame":12}],"dt":2}"#;
    let op: TlOp = serde_json::from_str(j).unwrap();
    assert_eq!(op, TlOp::MoveKeys { keys: vec![kr(0, 12.0)], dt: 2.0, dv: 0.0, opts: EditOpts::default() });
    let j = r#"{"op":"move_keys","keys":[{"ch":0,"frame":12}],"dt":2,"opts":{"collision":"refuse"}}"#;
    match serde_json::from_str::<TlOp>(j).unwrap() {
        TlOp::MoveKeys { opts, .. } => assert!(opts.snap && opts.collision == Collision::Refuse),
        o => panic!("{o:?}"),
    }
    assert!(serde_json::from_str::<TlOp>(r#"{"op":"explode"}"#).is_err());
    let mut e = Editor::new(proj()).unwrap();
    let pl = run(&mut e, TlOp::Copy { keys: vec![kr(0, 0.0)] });
    assert!(pl.op.is_none() && pl.clip.is_some());
    assert!(!e.can_undo());
    // clips round-trip through JSON
    let c = pl.clip.unwrap();
    let back: Clip = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(c, back);
    assert_eq!(Collision::parse("merge"), Some(Collision::Merge));
}

#[test]
fn operations_are_deterministic() {
    let p = proj();
    let ops = [
        TlOp::MoveKeys { keys: vec![kr(0, 12.0), kr(2, 0.0), kr(1, 24.0)], dt: 5.0, dv: 1.0, opts: EditOpts::default() },
        TlOp::Delete { keys: vec![kr(4, 0.0), kr(0, 24.0)] },
        TlOp::Ripple { at: 3.0, count: 2.0, mode: RetimeMode::Insert, scope: None, adjust_frames: true, opts: EditOpts::default() },
    ];
    for op in &ops {
        let a = serde_json::to_string(&plan(&p, op).unwrap()).unwrap();
        let b = serde_json::to_string(&plan(&p, op).unwrap()).unwrap();
        assert_eq!(a, b);
    }
    // selection order in the request does not matter
    let a = plan(&p, &TlOp::Delete { keys: vec![kr(0, 12.0), kr(0, 0.0)] }).unwrap();
    let b = plan(&p, &TlOp::Delete { keys: vec![kr(0, 0.0), kr(0, 12.0), kr(0, 0.0)] }).unwrap();
    assert_eq!(serde_json::to_string(&a.op).unwrap(), serde_json::to_string(&b.op).unwrap());
}

#[test]
fn graph_view_samples_keys_and_handles() {
    let p = proj();
    let v = graph_view(&p, &[0, 3, 2], 0.0, 48.0, 49).unwrap();
    assert_eq!(v.channels.len(), 2);
    assert_eq!(v.skipped.len(), 1);
    assert!(v.range[0] < 0.0 && v.range[1] > 60.0, "{:?}", v.range);
    assert_eq!(v.channels[0].keys.len(), 3);
    assert!(v.channels[0].keys[0].inn.is_none() && v.channels[0].keys[0].out.is_some());
    assert!(v.channels[0].samples.len() >= 49);
    assert!(graph_view(&p, &[0], 5.0, 5.0, 10).is_err());
    assert!(graph_view(&p, &[77], 0.0, 1.0, 10).is_err());
    let j = serde_json::to_value(&v).unwrap();
    assert!(j["channels"][0]["keys"][1]["in"].is_object());
    assert_eq!(serde_json::to_string(&v).unwrap(), serde_json::to_string(&graph_view(&p, &[0, 3, 2], 0.0, 48.0, 49).unwrap()).unwrap());
    assert_eq!(graph_view(&p, &[], 0.0, 1.0, 4).unwrap().range, [-1.0, 1.0]);
}

#[test]
fn demo_project_builds_and_every_edit_keeps_it_valid() {
    let p = crate::demo::project();
    let m = build(&p);
    assert_eq!(m.tracks.len(), p.channels.len());
    assert!(m.tracks.iter().any(|t| t.source == "expr") && m.tracks.iter().any(|t| t.editable));
    let all = select_all(&m, false);
    assert!(!all.is_empty());
    let mut e = Editor::new(p).unwrap();
    let before = e.doc().clone();
    run(&mut e, TlOp::MoveKeys { keys: all.clone(), dt: 0.0, dv: 1.0, opts: EditOpts::default() });
    e.undo();
    run(&mut e, TlOp::Smooth { keys: all.clone(), kind: TangentKind::Monotone });
    e.undo();
    run(&mut e, TlOp::Ripple { at: 10.0, count: 4.0, mode: RetimeMode::Delete, scope: None, adjust_frames: true, opts: EditOpts::default() });
    e.undo();
    assert_eq!(e.doc(), &before);
}

#[test]
fn channels_of_a_locked_layer_are_locked_in_the_model_and_refuse_edits() {
    let mut p = Project::new("t", 100.0, 100.0, 24.0, 48);
    p.layers = vec![crate::project::Layer::group("g", vec![crate::project::Layer::vector("v", vec![])]), crate::project::Layer::vector("w", vec![])];
    p.channels = vec![
        Channel::keys("g/v", Prop::LayerX, lin(&[(0.0, 0.0), (12.0, 30.0)])),
        Channel::keys("w", Prop::LayerX, lin(&[(0.0, 0.0), (12.0, 30.0)])),
        Channel::keys("", Prop::CameraZoom, lin(&[(0.0, 1.0)])),
    ];
    p.find_layer_mut("g").unwrap().locked = true;
    let m = build(&p);
    assert_eq!(m.tracks.iter().map(|t| t.locked).collect::<Vec<_>>(), vec![true, false, false], "group lock reaches the child layer only");
    assert!(chanmeta::layer_locked(&p, "g/v::lat::0,0", Prop::LatticeDx) && !chanmeta::layer_locked(&p, "w::lat::0,0", Prop::LatticeDx) && !chanmeta::layer_locked(&p, "g", Prop::BoneAngle));
    assert!(chanmeta::assert_unlocked(&p, "g/v", Prop::LayerX).unwrap_err().contains("layer 'g/v' is locked"));
    assert!(chanmeta::assert_unlocked(&p, "w", Prop::LayerX).is_ok());
    let mut e = Editor::new(p).unwrap();
    let before = e.doc().clone();
    let err = apply(&mut e, &TlOp::MoveKeys { keys: vec![kr(0, 12.0)], dt: 2.0, dv: 0.0, opts: Default::default() }).unwrap_err();
    assert!(err.join(" ").contains("locked"), "{err:?}");
    assert_eq!(e.doc(), &before);
    run(&mut e, TlOp::MoveKeys { keys: vec![kr(1, 12.0)], dt: 2.0, dv: 0.0, opts: Default::default() });
    // selecting by box skips the locked tracks unless asked
    let m = build(e.project());
    assert!(select_all(&m, false).iter().all(|k| k.ch != 0));
    assert!(select_all(&m, true).iter().any(|k| k.ch == 0));
    // unlock through a plain Op and the same edit works
    e.apply(Op::Set { path: "/layers/0/locked".into(), value: serde_json::json!(false) }).unwrap();
    run(&mut e, TlOp::MoveKeys { keys: vec![kr(0, 12.0)], dt: 2.0, dv: 0.0, opts: Default::default() });
}
