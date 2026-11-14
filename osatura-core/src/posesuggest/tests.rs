// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `posesuggest`.

use super::*;
use crate::anim::Ease;
use crate::math::Vec2;
use crate::project::Bone;

fn arm() -> Project {
    let mut p = Project::new("arm", 400.0, 300.0, 24.0, 40);
    p.bones = vec![
        Bone::new("u", None, 80.0, Vec2::new(150.0, 120.0), 10.0),
        Bone::new("l", Some("u"), 70.0, Vec2::new(80.0, 0.0), -5.0),
        Bone::new("h", Some("l"), 30.0, Vec2::new(70.0, 0.0), 0.0).with_angle_limits(-60.0, 60.0),
    ];
    p
}

/// Reach cycle on the arm: u and l swing together, h follows l.
fn arm_with_cycle() -> Project {
    let mut p = arm();
    let mut tu = Track::new();
    let mut tl = Track::new();
    let mut th = Track::new();
    for i in 0..=12 {
        let f = i as f32 * 3.0;
        let ph = std::f32::consts::TAU * i as f32 / 12.0;
        tu.set_key(f, 40.0 * ph.sin(), Ease::Linear);
        tl.set_key(f, 30.0 * (ph + 0.8).sin() + 20.0, Ease::Linear);
        th.set_key(f, 0.5 * 30.0 * (ph + 0.8).sin(), Ease::Linear);
    }
    p.channels = vec![Channel::keys("u", Prop::BoneAngle, tu), Channel::keys("l", Prop::BoneAngle, tl), Channel::keys("h", Prop::BoneAngle, th)];
    p
}

fn engine_tip(p: &Project, angles: &[f32], bone: usize) -> Vec2 {
    let mut q = p.clone();
    q.channels = p.bones.iter().zip(angles).map(|(b, a)| Channel::keys(&b.name, Prop::BoneAngle, Track::with_keys(&[(0.0, *a, Ease::Linear)]))).collect();
    let e = crate::Engine::new(q).unwrap();
    e.evaluate(0.0).world[bone].apply(Vec2::new(p.bones[bone].length, 0.0))
}

#[test]
fn forward_kinematics_matches_the_engine() {
    let p = arm();
    for angles in [[0.0f32, 0.0, 0.0], [30.0, -45.0, 20.0], [-80.0, 100.0, 59.0], [10.0, 10.0, -200.0]] {
        for bone in 0..3 {
            let want = engine_tip(&p, &angles, bone);
            let got = fk_tip(&p, &angles, bone);
            assert!((want.x - got[0]).abs() < 1e-2 && (want.y - got[1]).abs() < 1e-2, "{angles:?} bone {bone}: {want:?} vs {got:?}");
        }
    }
}

#[test]
fn database_from_project_and_library() {
    let p = arm_with_cycle();
    let db = build_db(Some(&p), None, None).unwrap();
    assert_eq!(db.bones, vec!["u", "l", "h"]);
    assert_eq!(db.poses.len(), 13);
    assert_eq!(db.sequences.len(), 1);
    assert_eq!(db.meta[3], PoseMeta { seq: 0, idx: 3, frame: 9.0 });
    assert!((db.poses[3][0] - 40.0).abs() < 1e-3, "key value at frame 9 (a quarter cycle)");
    assert_eq!(db.successor(0), Some(1));
    assert_eq!(db.successor(12), None);
    db.validate().unwrap();
    assert_eq!(PoseDb::from_json(&db.to_json()).unwrap(), db);
    // stepped sampling
    let dense = build_db(Some(&p), None, Some(1.0)).unwrap();
    assert_eq!(dense.poses.len(), 37);
    // library takes become sequences; missing bones are 0 and extra bones are appended
    let mut lib = Library::default();
    let mut q = arm_with_cycle();
    q.channels.push(Channel::keys("tail", Prop::BoneAngle, Track::with_keys(&[(0.0, 5.0, Ease::Linear), (6.0, 15.0, Ease::Linear)])));
    crate::motionlib::record(&mut lib, &q, "wag", 0.0, 6.0, &["tail".to_string()]).unwrap();
    let both = build_db(Some(&p), Some(&lib), None).unwrap();
    assert_eq!(both.bones, vec!["u", "l", "h", "tail"]);
    assert_eq!(both.sequences.len(), 2);
    assert_eq!(both.sequences[1].name, "wag");
    assert_eq!(both.poses[0][3], 0.0);
    assert_eq!(both.poses.last().unwrap()[3], 15.0);
    assert_eq!(both.poses.last().unwrap()[0], 0.0);
}

#[test]
fn database_errors_are_reported() {
    let p = arm();
    assert!(build_db(Some(&p), None, None).unwrap_err().contains("no keyed bone rotations"));
    assert!(build_db(None, None, None).is_err());
    let q = arm_with_cycle();
    assert!(build_db(Some(&q), None, Some(0.0)).is_err());
    assert!(build_db(Some(&q), None, Some(f32::NAN)).is_err());
    let mut one = arm();
    one.channels = vec![Channel::keys("u", Prop::BoneAngle, Track::with_keys(&[(0.0, 1.0, Ease::Linear)]))];
    assert!(build_db(Some(&one), None, None).unwrap_err().contains("at least two poses"));
    let mut db = build_db(Some(&q), None, None).unwrap();
    db.poses[0].pop();
    assert!(db.validate().is_err());
    assert!(PoseDb::from_json("{").is_err());
    let mut bad = build_db(Some(&q), None, None).unwrap();
    bad.format = "x".into();
    assert!(bad.validate().is_err());
}

#[test]
fn jacobi_and_power_iteration_agree_with_a_known_matrix() {
    let a = vec![vec![2.0, 1.0, 0.0], vec![1.0, 2.0, 0.0], vec![0.0, 0.0, 5.0]];
    let (vals, vecs) = jacobi_eigen(a.clone());
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| a.total_cmp(b));
    assert!((sorted[0] - 1.0).abs() < 1e-9 && (sorted[1] - 3.0).abs() < 1e-9 && (sorted[2] - 5.0).abs() < 1e-9, "{sorted:?}");
    // A v = lambda v for every pair
    for c in 0..3 {
        for r in 0..3 {
            let av: f64 = (0..3).map(|k| a[r][k] * vecs[k][c]).sum();
            assert!((av - vals[c] * vecs[r][c]).abs() < 1e-9);
        }
    }
    let (l, v) = power_iteration(&a, 200, 3);
    assert!((l - 5.0).abs() < 1e-6 && v[2].abs() > 0.999);
}

#[test]
fn pca_recovers_known_axes_and_variances() {
    // 6-D data spread along two known orthonormal axes with standard deviations 10 and 3, plus tiny noise
    let u1 = [0.5f32, 0.5, 0.5, 0.5, 0.0, 0.0];
    let u2 = [0.5f32, -0.5, 0.5, -0.5, 0.0, 0.0];
    let mut rng = Rng::new(11);
    let mut gauss = || {
        let (a, b) = (rng.unit().max(1e-6), rng.unit());
        (-2.0 * a.ln()).sqrt() * (std::f32::consts::TAU * b).cos()
    };
    let poses: Vec<Vec<f32>> = (0..600)
        .map(|_| {
            let (a, b) = (10.0 * gauss(), 3.0 * gauss());
            (0..6).map(|j| 7.0 + a * u1[j] + b * u2[j] + 0.01 * gauss()).collect()
        })
        .collect();
    let sp = PoseSpace::fit(&poses, 0.99, 6).unwrap();
    assert_eq!(sp.k(), 2, "two axes carry 99 % of the variance");
    let cos = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>().abs();
    assert!(cos(&sp.comps[0], &u1) > 0.999, "first axis {:?}", sp.comps[0]);
    assert!(cos(&sp.comps[1], &u2) > 0.995, "second axis {:?}", sp.comps[1]);
    assert!((sp.eig[0].sqrt() - 10.0).abs() < 0.8 && (sp.eig[1].sqrt() - 3.0).abs() < 0.3, "{:?}", sp.eig);
    assert!(sp.explained[1] > 0.99 && sp.explained[0] < sp.explained[1]);
    assert!((sp.mean[0] - 7.0).abs() < 1.5);
    // projection and reconstruction round trip for points on the plane
    let x = poses[5].clone();
    let back = sp.reconstruct(&sp.project(&x));
    assert!(rms(&x, &back) < 0.1, "{}", rms(&x, &back));
    // orthonormal axes
    assert!(sp.comps[0].iter().zip(&sp.comps[1]).map(|(a, b)| a * b).sum::<f32>().abs() < 1e-4);
    assert!((sp.comps[0].iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 1e-4);
    // errors
    assert!(PoseSpace::fit(&poses[..1], 0.9, 3).is_err());
    assert!(PoseSpace::fit(&poses, 0.0, 3).is_err());
    assert!(PoseSpace::fit(&poses, 0.9, 0).is_err());
    assert!(PoseSpace::fit(&vec![vec![1.0, 2.0]; 5], 0.9, 2).unwrap_err().contains("identical"));
}

fn circle_db() -> PoseDb {
    // poses on a circle of radius 30 in the plane of bones b0, b1 (a curved one-dimensional manifold), 4 bones
    let mut db = PoseDb { format: db_format(), version: 1, bones: (0..4).map(|i| format!("b{i}")).collect(), poses: vec![], meta: vec![], sequences: vec![SeqInfo { name: "circle".into(), source: "library".into(), poses: 36 }] };
    for i in 0..36 {
        let t = std::f32::consts::TAU * i as f32 / 36.0;
        db.poses.push(vec![30.0 * t.cos(), 30.0 * t.sin(), 0.0, 0.0]);
        db.meta.push(PoseMeta { seq: 0, idx: i, frame: i as f32 * 2.0 });
    }
    db
}

#[test]
fn between_stays_closer_to_the_manifold_than_a_linear_blend() {
    let sg = Suggester::with_pca(circle_db(), 0.999, 4).unwrap();
    let a = sg.db.poses[0].clone();
    let b = sg.db.poses[12].clone(); // 120 degrees apart: the chord midpoint is 15 inside the circle
    let c = sg.between(&a, &b, 0.5, 0.8, 6).unwrap();
    let get = |m: &str| c.iter().find(|x| x.method == m).unwrap();
    let off = |x: &Candidate| (x.pose[0].hypot(x.pose[1]) - 30.0).abs();
    assert!((off(get("linear")) - 15.0).abs() < 0.2, "linear midpoint is 15 off the circle, got {}", off(get("linear")));
    assert!((off(get("pca")) - 15.0).abs() < 0.5, "a PCA blend of a curved manifold is no better");
    assert!(off(get("manifold")) < 5.0, "kernel pull: {}", off(get("manifold")));
    assert!(off(get("library")) < 0.1, "a library pose lies on the circle");
    // the ranking prefers on-manifold candidates
    assert!(c[0].method == "library" || c[0].method == "manifold", "best {:?}", c.iter().map(|x| (&x.method, x.score)).collect::<Vec<_>>());
    assert!(get("linear").score < get("manifold").score);
    assert!(c.windows(2).all(|w| w[0].score >= w[1].score));
    // the library pose is roughly half way: around 60 degrees, i.e. index 6
    let lp = get("library");
    let ang = lp.pose[1].atan2(lp.pose[0]).to_degrees();
    assert!((ang - 60.0).abs() < 12.0, "library between-pose at {ang} degrees");
    // endpoints: t = 0 gives A back for the blends
    let c0 = sg.between(&a, &b, 0.0, 0.0, 6).unwrap();
    assert!(rms(&c0.iter().find(|x| x.method == "linear").unwrap().pose, &a) < 1e-4);
    assert!(rms(&c0.iter().find(|x| x.method == "pca").unwrap().pose, &a) < 0.1);
}

#[test]
fn between_on_a_straight_manifold_keeps_the_linear_blend() {
    // poses on a straight line: nothing to fix
    let mut db = circle_db();
    for (i, p) in db.poses.iter_mut().enumerate() {
        *p = vec![i as f32, 2.0 * i as f32, -(i as f32), 0.5 * i as f32];
    }
    let sg = Suggester::new(db).unwrap();
    let (a, b) = (sg.db.poses[4].clone(), sg.db.poses[14].clone());
    let c = sg.between(&a, &b, 0.5, 0.5, 4).unwrap();
    let lin = c.iter().find(|x| x.method == "linear").unwrap();
    let man = c.iter().find(|x| x.method == "manifold").unwrap();
    assert!(rms(&lin.pose, &sg.db.poses[9]) < 1e-3);
    assert!(rms(&man.pose, &sg.db.poses[9]) < 1.5, "{}", rms(&man.pose, &sg.db.poses[9]));
    assert!(lin.plausibility > 0.9);
}

#[test]
fn between_rejects_bad_input() {
    let sg = Suggester::with_pca(circle_db(), 0.99, 4).unwrap();
    let a = sg.db.poses[0].clone();
    assert!(sg.between(&a[..3], &a, 0.5, 0.5, 4).is_err());
    assert!(sg.between(&a, &a, 1.5, 0.5, 4).is_err());
    assert!(sg.between(&a, &a, 0.5, -0.1, 4).is_err());
    assert!(sg.between(&a, &a, 0.5, 0.5, 0).is_err());
    let mut nan = a.clone();
    nan[0] = f32::NAN;
    assert!(sg.between(&nan, &a, 0.5, 0.5, 4).is_err());
    assert!(sg.between(&a, &a, f32::NAN, 0.5, 4).is_err());
}

#[test]
fn completion_is_exact_on_a_linear_pose_space() {
    // poses = mean + a*u1 + b*u2 exactly; fixing two angles determines the rest
    let u1 = [0.6f32, 0.8, 0.0, 0.0, 0.0];
    let u2 = [0.0f32, 0.0, 0.6, 0.8, 0.0];
    let mut rng = Rng::new(5);
    let mut db = PoseDb { format: db_format(), version: 1, bones: (0..5).map(|i| format!("b{i}")).collect(), poses: vec![], meta: vec![], sequences: vec![SeqInfo { name: "s".into(), source: "library".into(), poses: 80 }] };
    for i in 0..80 {
        let (a, b) = (30.0 * (rng.unit() - 0.5), 20.0 * (rng.unit() - 0.5));
        db.poses.push((0..5).map(|j| 4.0 + a * u1[j] + b * u2[j]).collect());
        db.meta.push(PoseMeta { seq: 0, idx: i, frame: i as f32 });
    }
    let sg = Suggester::with_pca(db, 0.999, 5).unwrap();
    assert_eq!(sg.space.k(), 2);
    let truth = vec![4.0 + 10.0 * 0.6 + 5.0 * 0.0, 4.0 + 10.0 * 0.8, 4.0 + 5.0 * 0.6, 4.0 + 5.0 * 0.8, 4.0];
    let cons = vec![
        Constraint::Angle { bone: "b0".into(), value: truth[0], weight: 1.0 },
        Constraint::Angle { bone: "b3".into(), value: truth[3], weight: 1.0 },
    ];
    let c = sg.complete(None, &cons, &[0.0; 5], None, None, &[], 0.001, 3, None).unwrap();
    let best = c.iter().find(|x| x.method == "pca-ls").unwrap();
    assert!(rms(&best.pose, &truth) < 0.1, "recovered {:?} vs {:?}", best.pose, truth);
    assert!(best.constraint_error.unwrap() < 0.1);
    // the nearest-library baseline cannot be that exact
    let nn = c.iter().find(|x| x.method == "library-nn").unwrap();
    assert!(nn.constraint_error.unwrap() > best.constraint_error.unwrap());
    assert_eq!(c[0].method, "pca-ls", "ranked first");
    assert!(c.windows(2).all(|w| w[0].score >= w[1].score));
}

#[test]
fn completion_with_a_reach_target_hits_it_and_respects_fixed_bones() {
    let p = arm_with_cycle();
    let sg = Suggester::new(build_db(Some(&p), None, Some(1.0)).unwrap()).unwrap();
    // the tip of the hand bone in a pose that is in the library
    let truth = pose_at(&p, &sg.db.bones, 7.0);
    let ang = |x: &[f32]| p.bones.iter().map(|b| x[sg.db.bones.iter().position(|n| *n == b.name).unwrap()]).collect::<Vec<f32>>();
    let target = engine_tip(&p, &ang(&truth), 2);
    let cons = vec![Constraint::Reach { bone: "h".into(), at: [target.x, target.y], weight: 1.0 }];
    let cur = pose_at(&p, &sg.db.bones, 0.0);
    let c = sg.complete(Some(&p), &cons, &cur, Some(&ang(&cur)), None, &[], 0.1, 5, None).unwrap();
    let best = &c[0];
    let reached = engine_tip(&p, &ang(&best.pose), 2);
    assert!(reached.distance(target) < 3.0, "reached {reached:?}, wanted {target:?} (error {:?})", best.constraint_error);
    assert!(best.plausibility > 0.3, "the result stays near the library: {}", best.plausibility);
    // fixing the upper arm to its current value: the free bones still reach as well as they can
    let free = vec!["l".to_string(), "h".to_string()];
    let c2 = sg.complete(Some(&p), &cons, &cur, Some(&ang(&cur)), Some(&free), &[], 0.1, 3, None).unwrap();
    assert!((c2[0].pose[0] - cur[0]).abs() < 0.2, "u stays at {} (got {})", cur[0], c2[0].pose[0]);
    // a locked bone is held too, and constraining it is refused
    let locked = vec!["l".to_string()];
    let c3 = sg.complete(Some(&p), &cons, &cur, Some(&ang(&cur)), None, &locked, 0.1, 3, None).unwrap();
    assert!((c3[0].pose[1] - cur[1]).abs() < 0.2);
    let bad = vec![Constraint::Angle { bone: "l".into(), value: 5.0, weight: 1.0 }];
    assert!(sg.complete(Some(&p), &bad, &cur, None, None, &locked, 0.1, 3, None).unwrap_err().contains("locked"));
    // angle limit of the hand (+-60) is honoured
    let far = vec![Constraint::Angle { bone: "h".into(), value: 170.0, weight: 1.0 }];
    let c4 = sg.complete(Some(&p), &far, &cur, Some(&ang(&cur)), None, &[], 0.1, 1, None).unwrap();
    assert!(c4[0].pose[2] <= 60.0 + 1e-3);
}

#[test]
fn completion_validates_its_input() {
    let sg = Suggester::with_pca(circle_db(), 0.99, 4).unwrap();
    let cur = vec![0.0; 4];
    let a = |b: &str, v: f32, w: f32| vec![Constraint::Angle { bone: b.into(), value: v, weight: w }];
    assert!(sg.complete(None, &[], &cur, None, None, &[], 1.0, 3, None).is_err(), "no constraints");
    assert!(sg.complete(None, &a("zzz", 1.0, 1.0), &cur, None, None, &[], 1.0, 3, None).unwrap_err().contains("not in the pose database"));
    assert!(sg.complete(None, &a("b0", f32::NAN, 1.0), &cur, None, None, &[], 1.0, 3, None).is_err());
    assert!(sg.complete(None, &a("b0", 1.0, 0.0), &cur, None, None, &[], 1.0, 3, None).is_err());
    assert!(sg.complete(None, &a("b0", 1.0, 1.0), &cur[..2], None, None, &[], 1.0, 3, None).is_err());
    assert!(sg.complete(None, &a("b0", 1.0, 1.0), &cur, None, None, &[], -1.0, 3, None).is_err());
    assert!(sg.complete(None, &a("b0", 1.0, 1.0), &cur, None, None, &[], 1.0, 0, None).is_err());
    let reach = vec![Constraint::Reach { bone: "b0".into(), at: [1.0, 1.0], weight: 1.0 }];
    assert!(sg.complete(None, &reach, &cur, None, None, &[], 1.0, 3, None).unwrap_err().contains("needs the project"));
    let p = arm_with_cycle();
    let bad = vec![Constraint::Reach { bone: "nope".into(), at: [1.0, 1.0], weight: 1.0 }];
    let sg2 = Suggester::new(build_db(Some(&p), None, None).unwrap()).unwrap();
    assert!(sg2.complete(Some(&p), &bad, &[0.0; 3], None, None, &[], 1.0, 3, None).is_err());
}

#[test]
fn next_pose_continues_the_motion_of_the_library() {
    let p = arm_with_cycle();
    let sg = Suggester::new(build_db(Some(&p), None, None).unwrap()).unwrap();
    // standing exactly on library pose 3: the best successor is pose 4 and the key goes 3 frames later
    let cur = sg.db.poses[3].clone();
    let c = sg.next(&cur, 20.0, 4, 5).unwrap();
    let truth = &sg.db.poses[4];
    let best = c.iter().map(|x| rms(&x.pose, truth)).fold(f32::MAX, f32::min);
    assert!(best < 3.0, "closest candidate is {best} deg from the true successor");
    let mean = c.iter().find(|x| x.method == "markov-mean").unwrap();
    assert!(rms(&mean.pose, truth) < 6.0, "{}", rms(&mean.pose, truth));
    assert!(rms(&mean.pose, &cur) > 3.0, "it moves");
    assert!((mean.frame.unwrap() - 23.0).abs() < 0.6, "gap of 3 frames: {:?}", mean.frame);
    assert!(c.windows(2).all(|w| w[0].score >= w[1].score));
    assert!(c.len() <= 5);
    // a pose far from everything gets a warning
    let far = vec![500.0, -500.0, 300.0];
    let c = sg.next(&far, 0.0, 4, 3).unwrap();
    assert!(c[0].notes.iter().any(|n| n.contains("library may not cover")));
    // errors
    assert!(sg.next(&cur[..2], 0.0, 4, 3).is_err());
    assert!(sg.next(&cur, 0.0, 0, 3).is_err());
    assert!(sg.next(&cur, 0.0, 4, 0).is_err());
    // a library of single poses has no successors
    let mut one = circle_db();
    one.sequences = (0..36).map(|i| SeqInfo { name: format!("s{i}"), source: "library".into(), poses: 1 }).collect();
    for (i, m) in one.meta.iter_mut().enumerate() {
        m.seq = i;
        m.idx = 0;
    }
    let sg = Suggester::with_pca(one, 0.99, 3).unwrap();
    assert!(sg.next(&[0.0; 4], 0.0, 4, 3).unwrap_err().contains("no consecutive"));
}

#[test]
fn applying_is_one_undo_step_and_respects_locks() {
    let mut p = arm_with_cycle();
    p.frames = 20;
    let vals = vec![BoneValue { bone: "u".into(), angle: 12.5 }, BoneValue { bone: "l".into(), angle: -7.0 }];
    let mut ed = crate::edit::Editor::new(p.clone()).unwrap();
    let before = ed.doc().clone();
    let (op, skipped) = apply_op(ed.project(), &vals, 50.0, Ease::Smooth).unwrap();
    assert!(skipped.is_empty());
    ed.apply(op).unwrap();
    assert_eq!(ed.project().frames, 51, "the timeline grows to hold the key");
    let u = ed.project().channels.iter().find(|c| c.target == "u").unwrap();
    if let Source::Keys { track } = &u.source {
        assert_eq!(track.eval(50.0), Some(12.5));
        assert_eq!(track.keys.len(), 14);
    }
    assert!(ed.undo());
    assert_eq!(*ed.doc(), before, "one undo restores everything");
    assert!(ed.redo());
    // new channel for a bone without keys, plus a lock on another
    let mut q = arm();
    q.frames = 30;
    q.channel_meta.push(crate::chanmeta::ChannelMeta { target: "l".into(), prop: Some(Prop::BoneAngle), lock: true, ..Default::default() });
    q.channels.push(Channel::keys("l", Prop::BoneAngle, Track::with_keys(&[(0.0, 1.0, Ease::Linear)])));
    let (op, skipped) = apply_op(&q, &vals, 10.0, Ease::Linear).unwrap();
    assert_eq!(skipped, vec!["l".to_string()]);
    let mut ed = crate::edit::Editor::new(q.clone()).unwrap();
    ed.apply(op).unwrap();
    assert_eq!(ed.project().channels.len(), 2, "u got a channel, locked l did not change");
    let l = ed.project().channels.iter().find(|c| c.target == "l").unwrap();
    if let Source::Keys { track } = &l.source {
        assert_eq!(track.keys.len(), 1);
    }
    // everything locked: an error, nothing to apply
    let only_l = vec![BoneValue { bone: "l".into(), angle: 3.0 }];
    assert!(apply_op(&q, &only_l, 5.0, Ease::Linear).unwrap_err().contains("locked"));
    // bad input
    assert!(apply_op(&q, &[BoneValue { bone: "zz".into(), angle: 1.0 }], 5.0, Ease::Linear).is_err());
    assert!(apply_op(&q, &vals, -1.0, Ease::Linear).is_err());
    assert!(apply_op(&q, &vals, f32::NAN, Ease::Linear).is_err());
    assert!(apply_op(&q, &[BoneValue { bone: "u".into(), angle: f32::INFINITY }], 1.0, Ease::Linear).is_err());
}

#[test]
fn request_layer_end_to_end_with_locks() {
    let mut p = arm_with_cycle();
    p.channel_meta.push(crate::chanmeta::ChannelMeta { target: "h".into(), prop: Some(Prop::BoneAngle), lock: true, ..Default::default() });
    let sg = Suggester::new(build_db(Some(&p), None, None).unwrap()).unwrap();
    let req = |json: &str| serde_json::from_str::<Request>(json).unwrap();
    let s = suggest(&p, &sg, &req(r#"{"kind":"between","a":0,"b":6,"t":0.5,"top":3}"#), None).unwrap();
    assert_eq!(s.kind, "between");
    assert!(s.candidates.len() <= 3 && !s.candidates.is_empty());
    assert_eq!(s.locked_skipped, vec!["h".to_string()]);
    assert!(s.candidates.iter().all(|c| c.values.iter().all(|v| v.bone != "h")));
    assert!((s.candidates[0].frame.unwrap() - 3.0).abs() < 1e-5);
    assert!(s.notes.iter().any(|n| n.contains("locked")));
    let s = suggest(&p, &sg, &req(r#"{"kind":"complete","frame":3,"constraints":[{"kind":"angle","bone":"u","value":20.0}]}"#), None).unwrap();
    assert!(s.candidates[0].constraint_error.unwrap() < 2.0);
    let s = suggest(&p, &sg, &req(r#"{"kind":"next","frame":6,"top":2}"#), None).unwrap();
    assert!(s.candidates.len() <= 2);
    assert!(s.candidates[0].frame.unwrap() > 6.0);
    // a free list limits what is written
    let s = suggest(&p, &sg, &req(r#"{"kind":"complete","frame":3,"free":["u","l"],"constraints":[{"kind":"angle","bone":"u","value":20.0}]}"#), None).unwrap();
    assert!(s.candidates[0].values.iter().all(|v| v.bone == "u" || v.bone == "l"));
    // errors
    assert!(suggest(&p, &sg, &req(r#"{"kind":"dance"}"#), None).is_err());
    assert!(suggest(&p, &sg, &req(r#"{"kind":"between","a":0}"#), None).unwrap_err().contains("b is required"));
    assert!(suggest(&p, &sg, &req(r#"{"kind":"next","frame":-2}"#), None).is_err());
    assert!(suggest(&p, &sg, &req(r#"{"kind":"complete","frame":0,"constraints":[]}"#), None).is_err());
    // a library bone the project lacks is reported
    let mut q = arm();
    q.bones.pop();
    q.channels = arm_with_cycle().channels.into_iter().take(2).collect();
    let s = suggest(&q, &sg, &req(r#"{"kind":"next","frame":0}"#), None).unwrap();
    assert!(s.notes.iter().any(|n| n.contains("not in this project")));
}

#[test]
fn mlp_completion_trains_deterministically_and_learns_the_toy_mapping() {
    // poses of 4 bones lying on a line: b = [t, 2t, -t, 0.5 t]; any one observed bone fixes the rest
    let mut db = circle_db();
    db.poses = (0..60).map(|i| { let t = -30.0 + i as f32; vec![t, 2.0 * t, -t, 0.5 * t] }).collect();
    db.meta = (0..60).map(|i| PoseMeta { seq: 0, idx: i, frame: i as f32 }).collect();
    db.sequences = vec![SeqInfo { name: "line".into(), source: "library".into(), poses: 60 }];
    let o = TrainOpts { hidden: vec![24], epochs: 150, lr: 0.02, momentum: 0.9, batch: 8, seed: 2 };
    let (m, hist) = train_completion(&db, &o).unwrap();
    let (m2, _) = train_completion(&db, &o).unwrap();
    assert_eq!(m, m2, "deterministic");
    assert!(hist.last().unwrap() < &(hist[0] * 0.2), "loss {} -> {}", hist[0], hist.last().unwrap());
    let y = predict_completion(&m, &db.bones, &[Some(12.0), None, None, None]).unwrap();
    assert_eq!(y[0], 12.0, "observed values pass through");
    let want = [12.0, 24.0, -12.0, 6.0];
    assert!(rms(&y, &want) < 4.0, "predicted {y:?}");
    // far better than the mean pose
    let mean_err = rms(&[0.0, 0.0, 0.0, 0.0], &[0.0, 24.0, -12.0, 6.0]);
    assert!(rms(&y, &want) * 2.0 < mean_err);
    // bad models
    let mut other = m.clone();
    other.bones[0] = "zzz".into();
    assert!(predict_completion(&other, &db.bones, &[Some(1.0), None, None, None]).unwrap_err().contains("trained for bones"));
    let mut thin = m.clone();
    thin.layers[0].n_in = 3;
    thin.layers[0].w.truncate(24 * 3);
    assert!(predict_completion(&thin, &db.bones, &[Some(1.0), None, None, None]).is_err());
    // through the suggester as a candidate
    let sg = Suggester::with_pca(db.clone(), 0.99, 4).unwrap();
    let cons = vec![Constraint::Angle { bone: "b0".into(), value: 12.0, weight: 1.0 }];
    let c = sg.complete(None, &cons, &[0.0; 4], None, None, &[], 1.0, 2, Some(&m)).unwrap();
    assert!(c.iter().any(|x| x.method == "mlp"));
    // training errors
    let mut tiny = db.clone();
    tiny.poses.truncate(3);
    tiny.meta.truncate(3);
    assert!(train_completion(&tiny, &o).is_err());
    assert!(train_completion(&db, &TrainOpts { epochs: 0, ..o.clone() }).is_err());
}

#[test]
fn suggester_scales_and_is_deterministic() {
    let db = synthetic_db(3, 12, 10, 8);
    let a = Suggester::new(db.clone()).unwrap();
    let b = Suggester::new(db).unwrap();
    assert_eq!(a.space.comps, b.space.comps);
    assert_eq!(a.spacing, b.spacing);
    assert!(a.space.k() >= 2 && a.space.k() <= 12);
    let c1 = a.between(&a.db.poses[0], &a.db.poses[2], 0.5, 0.5, 6).unwrap();
    let c2 = b.between(&b.db.poses[0], &b.db.poses[2], 0.5, 0.5, 6).unwrap();
    assert_eq!(c1.iter().map(|c| c.pose.clone()).collect::<Vec<_>>(), c2.iter().map(|c| c.pose.clone()).collect::<Vec<_>>());
}

#[test]
fn benchmark_numbers_make_sense() {
    let quick = TrainOpts { hidden: vec![16], epochs: 20, lr: 0.02, momentum: 0.9, batch: 16, seed: 1 };
    let b = benchmark(1, &quick).unwrap();
    let get = |t: &str, m: &str| b.iter().find(|l| l.task == t && l.method == m).unwrap().rms_deg;
    // any learned method must beat pasting values into the mean pose, otherwise the library taught nothing
    assert!(get("completion", "PCA least squares") < get("completion", "mean pose") * 0.5);
    assert!(get("completion", "nearest library pose") < get("completion", "mean pose"));
    assert!(get("next", "Markov mean") < get("next", "stay in place"));
    assert!(b.iter().all(|l| l.rms_deg.is_finite()));
}
