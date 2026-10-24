// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `fuzz`.
#![allow(unused_imports)]

use super::*;
use crate::project::{Bone, IkChain, Layer, LayerKind, MeshSpec};
use crate::path::{BoneWeight, VPath};

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// Two-bone rig with a one-cell image mesh: left edge rides bone "a", right edge rides bone "b".
/// Mesh spans x 360..440, y 180..220; the joint (head of b) is at (400, 200). The truth function below uses the same shape translated to (100, 100).
fn fold_rig() -> Project {
    let mut p = Project::new("fold", 600.0, 400.0, 24.0, 10);
    p.bones = vec![Bone::new("a", None, 100.0, v(300.0, 200.0), 0.0), Bone::new("b", Some("a"), 100.0, v(100.0, 0.0), 0.0)];
    let mut pm = tiny_skia::Pixmap::new(4, 4).unwrap();
    pm.fill(tiny_skia::Color::from_rgba8(200, 30, 30, 255));
    let mut l = Layer::image_from_png("quad", &pm.encode_png().unwrap(), v(360.0, 180.0));
    if let LayerKind::Image { size, mesh, .. } = &mut l.kind {
        *size = Some(v(80.0, 40.0));
        let w = |b: &str| vec![BoneWeight { bone: b.into(), weight: 1.0 }];
        *mesh = Some(MeshSpec { cols: 1, rows: 1, weights: vec![w("a"), w("b"), w("a"), w("b")], free: None });
    }
    p.layers = vec![l];
    p
}

fn opts_only(bone: &str, prop_suffix: &str) -> (ParamOptions, String) {
    (ParamOptions { only_bones: vec![bone.into()], deformers: false, ..Default::default() }, format!("bone:{bone}:{prop_suffix}"))
}

/// Independent ground truth: signed areas of the two triangles after rotating the right edge about the joint.
fn truth_folded(angle_deg: f32) -> bool {
    let (c, s) = (angle_deg.to_radians().cos(), angle_deg.to_radians().sin());
    let rot = |p: Vec2| v(100.0 + c * (p.x - 100.0) - s * (p.y - 100.0), 100.0 + s * (p.x - 100.0) + c * (p.y - 100.0));
    let (tl, bl) = (v(60.0, 80.0), v(60.0, 120.0));
    let (tr, br) = (rot(v(140.0, 80.0)), rot(v(140.0, 120.0)));
    let area = |a: Vec2, b: Vec2, c: Vec2| det2(b - a, c - a);
    // rest areas are positive for [tl,tr,br] and [tl,br,bl]
    area(tl, tr, br) <= 0.0 || area(tl, br, bl) <= 0.0
}

#[test]
fn fold_detected_beyond_threshold_and_not_before() {
    let p = fold_rig();
    let (po, id) = opts_only("b", "angle");
    let mut f = Fuzzer::new(&p, None, &po, Thresholds::default(), 0.0).unwrap();
    let k = f.params.iter().position(|q| q.id == id).unwrap();
    let mut first_fold = None;
    let mut checked = 0;
    for a in (0..=179).map(|a| a as f32) {
        let mut vals = f.rest_values();
        vals[k] = a;
        let r = f.eval(0, &vals);
        let folded = r.metrics.min_det <= 0.0;
        // away from the exact boundary the engine must agree with the independent calculation
        if !(truth_folded(a - 0.6) != truth_folded(a + 0.6)) {
            assert_eq!(folded, truth_folded(a), "angle {a}: min_det {}", r.metrics.min_det);
            checked += 1;
        }
        if folded && first_fold.is_none() {
            first_fold = Some(a);
        }
        if folded {
            assert!(r.failed && r.kinds.contains(&"fold".to_string()), "angle {a}");
        }
    }
    assert!(checked > 150);
    let ff = first_fold.expect("a fold must occur below 180 degrees");
    assert!(ff > 20.0 && ff < 170.0, "first fold at {ff}");
    // rest pose is clean
    let rv = f.rest_values();
    let r = f.eval(0, &rv);
    assert!(!r.failed, "{:?}", r.kinds);
    assert!((r.metrics.min_det - 1.0).abs() < 1e-4);
}

#[test]
fn project_is_not_modified() {
    let p = fold_rig();
    let before = p.clone();
    let o = FuzzOptions { samples: 30, strategy: Strategy::Lhs, minimize: true, ..Default::default() };
    let rep = run(&p, None, &o).unwrap();
    assert_eq!(p, before);
    assert!(rep.summary.failures > 0);
    let _ = grid(&p, None, &ParamOptions::default(), Thresholds::default(), 0.0, "bone:a:angle", "bone:b:angle", 4).unwrap();
    assert_eq!(p, before);
}

#[test]
fn stretch_metric_numbers() {
    let rest = [v(0.0, 0.0), v(10.0, 0.0), v(0.0, 10.0)];
    let th = Thresholds::default();
    // uniform scale 2: edges x2, det 4, no shear
    let def: Vec<Vec2> = rest.iter().map(|p| *p * 2.0).collect();
    let m = mesh_metrics(&rest, &def, &[[0, 1, 2]], &[], &th);
    assert!((m.max_stretch - 2.0).abs() < 1e-5);
    assert!((m.min_det - 4.0).abs() < 1e-4 && (m.max_det - 4.0).abs() < 1e-4);
    assert!(m.max_shear.abs() < 1e-4);
    assert_eq!(m.flips, 0);
    // x scaled by 3, y by 1: smax/smin - 1 = 2, longest edge ratio 3 (x edge), det 3
    let def: Vec<Vec2> = rest.iter().map(|p| v(p.x * 3.0, p.y)).collect();
    let m = mesh_metrics(&rest, &def, &[[0, 1, 2]], &[], &th);
    assert!((m.max_shear - 2.0).abs() < 1e-4, "{}", m.max_shear);
    assert!((m.max_stretch - 3.0).abs() < 1e-5);
    assert!((m.min_det - 3.0).abs() < 1e-4);
    // mirrored: det -1, flagged
    let def: Vec<Vec2> = rest.iter().map(|p| v(-p.x, p.y)).collect();
    let m = mesh_metrics(&rest, &def, &[[0, 1, 2]], &[], &th);
    assert_eq!(m.flips, 1);
    assert!((m.min_det + 1.0).abs() < 1e-5);
    assert!(m.stress.iter().all(|s| *s >= 1.0));
    // rotation by 90 degrees changes nothing
    let def: Vec<Vec2> = rest.iter().map(|p| v(-p.y, p.x)).collect();
    let m = mesh_metrics(&rest, &def, &[[0, 1, 2]], &[], &th);
    assert!((m.max_stretch - 1.0).abs() < 1e-5 && m.max_shear.abs() < 1e-4 && (m.min_det - 1.0).abs() < 1e-5);
    assert!(m.stress.iter().all(|s| *s < 1e-3));
    // shear x += 0.5 y: det 1, smax/smin - 1 from the shear matrix [[1,.5],[0,1]]
    let def: Vec<Vec2> = rest.iter().map(|p| v(p.x + 0.5 * p.y, p.y)).collect();
    let m = mesh_metrics(&rest, &def, &[[0, 1, 2]], &[], &th);
    let (s1, det) = (1.0 + 0.25 + 1.0, 1.0f32);
    let smax = ((s1 + (s1 * s1 - 4.0 * det * det).sqrt()) * 0.5).sqrt();
    assert!((m.max_shear - (smax * smax - 1.0)).abs() < 1e-4, "{} vs {}", m.max_shear, smax * smax - 1.0);
    // degenerate rest triangles are ignored
    let m = mesh_metrics(&[v(0.0, 0.0), v(1.0, 0.0), v(2.0, 0.0)], &[v(0.0, 0.0), v(1.0, 0.0), v(2.0, 5.0)], &[[0, 1, 2]], &[], &th);
    assert_eq!(m.triangles, 0);
}

#[test]
fn spike_is_similarity_invariant_and_detects_a_pulled_vertex() {
    let mut rest = vec![];
    for r in 0..5 {
        for c in 0..5 {
            rest.push(v(c as f32 * 10.0, r as f32 * 10.0));
        }
    }
    let tris = crate::weights::grid_triangles(4, 4);
    let th = Thresholds::default();
    let scaled: Vec<Vec2> = rest.iter().map(|p| v(p.x * 2.0 + 5.0, p.y * 2.0)).collect();
    assert!(mesh_metrics(&rest, &scaled, &tris, &[], &th).max_spike < 1e-3);
    let mut bumped = rest.clone();
    bumped[12] += v(0.0, 25.0);
    let m = mesh_metrics(&rest, &bumped, &tris, &[], &th);
    assert!(m.max_spike > 0.5, "spike {}", m.max_spike);
    assert!(m.stress[12] > 0.5);
}

#[test]
fn self_intersection_sweep_matches_brute_force() {
    // bow-tie: two crossing edges
    let bow = [v(0.0, 0.0), v(10.0, 10.0), v(10.0, 0.0), v(0.0, 10.0)];
    assert_eq!(self_intersections(&bow, true, 100).len(), 1);
    let sq = [v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0), v(0.0, 10.0)];
    assert!(self_intersections(&sq, true, 100).is_empty());
    let mut rng = Rng::new(7);
    for _ in 0..30 {
        let pts: Vec<Vec2> = (0..12).map(|_| v(rng.unit() * 20.0, rng.unit() * 20.0)).collect();
        let n = pts.len();
        let mut brute = 0;
        for i in 0..n {
            for j in i + 1..n {
                if j == i + 1 || (i == 0 && j == n - 1) {
                    continue;
                }
                if segments_cross(pts[i], pts[(i + 1) % n], pts[j], pts[(j + 1) % n]) {
                    brute += 1;
                }
            }
        }
        assert_eq!(self_intersections(&pts, true, 1000).len(), brute);
    }
    assert!((signed_area(&sq) - 100.0).abs() < 1e-4);
}

#[test]
fn nan_inputs_are_flagged() {
    let p = fold_rig();
    let po = ParamOptions { only_bones: vec!["b".into()], deformers: false, ..Default::default() };
    let mut f = Fuzzer::new(&p, None, &po, Thresholds::default(), 0.0).unwrap();
    let mut vals = f.rest_values();
    vals[0] = f32::NAN;
    let r = f.eval(0, &vals);
    assert!(r.failed && r.metrics.non_finite && r.kinds[0] == "non_finite");
    // non-finite geometry in the metrics themselves does not panic
    let rest = [v(0.0, 0.0), v(1.0, 0.0), v(0.0, 1.0)];
    let def = [v(f32::NAN, 0.0), v(1.0, 0.0), v(0.0, 1.0)];
    let _ = mesh_metrics(&rest, &def, &[[0, 1, 2]], &[], &Thresholds::default());
    // report JSON never contains NaN (not valid JSON)
    let rep = run(&p, None, &FuzzOptions { samples: 5, ..Default::default() }).unwrap();
    assert!(!rep.to_json().contains("NaN"));
}

#[test]
fn radical_inverse_and_halton_known_values() {
    assert_eq!(first_primes(6), vec![2, 3, 5, 7, 11, 13]);
    let b2: Vec<f64> = (1..=7).map(|i| radical_inverse(i, 2)).collect();
    assert_eq!(b2, vec![0.5, 0.25, 0.75, 0.125, 0.625, 0.375, 0.875]);
    let b3: Vec<f64> = (1..=5).map(|i| radical_inverse(i, 3)).collect();
    for (a, b) in b3.iter().zip([1.0 / 3.0, 2.0 / 3.0, 1.0 / 9.0, 4.0 / 9.0, 7.0 / 9.0]) {
        assert!((a - b).abs() < 1e-12);
    }
    let h = halton_point(4, 3);
    assert!((h[0] - 0.125).abs() < 1e-6 && (h[1] - 4.0 / 9.0).abs() < 1e-6 && (h[2] - 0.8).abs() < 1e-6);
}

fn unit_params(d: usize) -> Vec<Param> {
    (0..d).map(|i| Param { id: format!("bone:b{i}:angle"), target: format!("b{i}"), prop: Prop::BoneAngle, lo: 0.0, hi: 1.0, rest: 0.5, limit: None }).collect()
}

#[test]
fn strategies_are_deterministic_and_cover() {
    let ps = unit_params(4);
    for s in [Strategy::Random, Strategy::Lhs, Strategy::Halton, Strategy::Sweep, Strategy::Corners] {
        let a = generate(&ps, s, 64, 5, None).unwrap();
        let b = generate(&ps, s, 64, 5, None).unwrap();
        assert_eq!(a, b, "{s:?}");
        assert!(a.iter().all(|x| x.len() == 4 && x.iter().all(|y| (0.0..=1.0).contains(y))), "{s:?} out of range");
    }
    assert_ne!(generate(&ps, Strategy::Random, 8, 1, None).unwrap(), generate(&ps, Strategy::Random, 8, 2, None).unwrap());
    // LHS: exactly one point in every stratum of every dimension
    let n = 50;
    let l = generate(&ps, Strategy::Lhs, n, 3, None).unwrap();
    for d in 0..4 {
        let mut hit = vec![0; n];
        for x in &l {
            hit[((x[d] * n as f32) as usize).min(n - 1)] += 1;
        }
        assert!(hit.iter().all(|&c| c == 1), "dim {d}: {hit:?}");
    }
    // sweep: ends of each axis included, other axes at rest
    let sw = generate(&ps, Strategy::Sweep, 40, 0, None).unwrap();
    assert_eq!(sw.len(), 40);
    assert_eq!(sw[0], vec![0.0, 0.5, 0.5, 0.5]);
    assert_eq!(sw[9], vec![1.0, 0.5, 0.5, 0.5]);
    // corners: all 16 distinct corners for d = 4, n >= 16
    let c = generate(&ps, Strategy::Corners, 16, 0, None).unwrap();
    let mut keys: Vec<String> = c.iter().map(|x| format!("{x:?}")).collect();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), 16);
    // Halton equals the textbook points
    let h = generate(&ps, Strategy::Halton, 3, 0, None).unwrap();
    assert!((h[2][0] - 0.75).abs() < 1e-6 && (h[2][1] - 1.0 / 9.0).abs() < 1e-6);
    // error paths
    assert!(generate(&[], Strategy::Random, 4, 0, None).is_err());
    assert!(generate(&ps, Strategy::Random, 0, 0, None).is_err());
    assert!(generate(&ps, Strategy::Pca, 4, 0, None).unwrap_err().contains("pose database"));
    assert!(Strategy::parse("nope").is_err());
    // low discrepancy beats chance on coverage: every 4x4 cell of the first two Halton dims is hit by 64 points
    let h = generate(&unit_params(2), Strategy::Halton, 64, 0, None).unwrap();
    let mut cells = std::collections::HashSet::new();
    for x in &h {
        cells.insert(((x[0] * 4.0) as usize, (x[1] * 4.0) as usize));
    }
    assert_eq!(cells.len(), 16);
}

#[test]
fn pca_samples_follow_the_database() {
    let ps = vec![
        Param { id: "bone:a:angle".into(), target: "a".into(), prop: Prop::BoneAngle, lo: -90.0, hi: 90.0, rest: 0.0, limit: None },
        Param { id: "bone:b:angle".into(), target: "b".into(), prop: Prop::BoneAngle, lo: -90.0, hi: 90.0, rest: 0.0, limit: None },
    ];
    // poses on a line b = 2a
    let db = crate::posesuggest::synthetic_db(1, 2, 20, 2);
    let mut db = db;
    db.bones = vec!["a".into(), "b".into()];
    db.poses = (0..40).map(|i| vec![(i as f32 - 20.0) * 1.5, (i as f32 - 20.0) * 3.0]).collect();
    db.meta = (0..40).map(|i| crate::posesuggest::PoseMeta { seq: 0, idx: i, frame: i as f32 }).collect();
    db.sequences = vec![crate::posesuggest::SeqInfo { name: "s".into(), source: "project".into(), poses: 40 }];
    let s = PcaSampler::new(&db, &ps).unwrap();
    assert_eq!(s.components(), 1);
    let pts = generate(&ps, Strategy::Pca, 50, 9, Some(&s)).unwrap();
    for p in &pts {
        if p[0].abs() < 40.0 {
            assert!((p[1] - 2.0 * p[0]).abs() < 0.5, "{p:?}");
        }
    }
    assert!(PcaSampler::new(&db, &unit_params(2)).is_err());
}

#[test]
fn ddmin_finds_the_minimal_combination() {
    let items: Vec<usize> = (0..16).collect();
    let mut calls = 0;
    let mut pred = |s: &[usize]| {
        calls += 1;
        s.contains(&3) && s.contains(&11)
    };
    let m = ddmin(&items, &mut pred);
    assert_eq!(m, vec![3, 11]);
    assert!(calls < 80, "{calls}");
    // a single culprit
    let mut p1 = |s: &[usize]| s.contains(&7);
    assert_eq!(ddmin(&items, &mut p1), vec![7]);
    // everything needed
    let few = vec![1usize, 2, 3];
    let mut p2 = |s: &[usize]| s.len() == 3;
    assert_eq!(ddmin(&few, &mut p2), few);
    // bisection finds the boundary
    let b = shrink_value(0.0, 100.0, 0.01, &mut |x| x >= 37.5);
    assert!((b - 37.5).abs() < 0.02 && b >= 37.5, "{b}");
    let b = shrink_value(10.0, -90.0, 0.01, &mut |x| x <= -20.0);
    assert!((b + 20.0).abs() < 0.02 && b <= -20.0, "{b}");
}

/// Fold rig plus an unrelated bone with nothing to break: the minimiser must drop it.
#[test]
fn minimizer_reduces_to_the_failing_parameter() {
    let mut p = fold_rig();
    p.bones.push(Bone::new("noise", None, 20.0, v(550.0, 20.0), 0.0));
    let po = ParamOptions { deformers: false, ..Default::default() };
    let mut f = Fuzzer::new(&p, None, &po, Thresholds::default(), 0.0).unwrap();
    let kb = f.params.iter().position(|q| q.id == "bone:b:angle").unwrap();
    let mut vals = f.rest_values();
    for (i, q) in f.params.iter().enumerate() {
        vals[i] = q.rest + 0.4 * (q.hi - q.rest);
    }
    vals[kb] = 175.0;
    assert!(f.eval(0, &vals).failed);
    let m = minimize(&mut f, &vals, Some("fold")).unwrap();
    assert!(m.verified);
    assert_eq!(m.params.len(), 1, "{:?}", m.params);
    assert_eq!(m.params[0].id, "bone:b:angle");
    // the minimal angle sits at the true fold boundary
    let boundary = (0..180).map(|a| a as f32).find(|a| truth_folded(*a)).unwrap();
    assert!((m.params[0].value - boundary).abs() < 3.0, "{} vs {boundary}", m.params[0].value);
    assert_eq!(m.kind, "fold");
    // a passing pose cannot be minimised
    let rv = f.rest_values();
    assert!(minimize(&mut f, &rv, None).is_err());
}

#[test]
fn marginals_and_pairs_point_at_the_culprits() {
    let p = fold_rig();
    let o = FuzzOptions { samples: 120, strategy: Strategy::Halton, params: ParamOptions { deformers: false, bone_props: vec!["angle".into()], ..Default::default() }, ..Default::default() };
    let rep = run(&p, None, &o).unwrap();
    let top = rep.marginals.iter().max_by(|a, b| a.spread.total_cmp(&b.spread)).unwrap();
    assert_eq!(top.id, "bone:b:angle");
    let angle = rep.marginals.iter().find(|m| m.id == "bone:b:angle").unwrap();
    assert!(angle.rate_outer > angle.rate_inner);
    assert_eq!(rep.per_bone[0].name, "b");
    assert!(!rep.pairs.is_empty());
    // inner bins never fail for b's angle (fold needs a large bend)
    assert_eq!(angle.bins[2].failures, 0);
}

#[test]
fn only_and_props_restrict_ik_mix_too() {
    let mut p = Project::new("ik", 400.0, 300.0, 24.0, 4);
    p.bones = vec![
        Bone::new("up", None, 50.0, v(100.0, 150.0), 0.0),
        Bone::new("lo", Some("up"), 50.0, v(50.0, 0.0), 0.0),
        Bone::new("tgt", None, 10.0, v(180.0, 150.0), 0.0),
        Bone::new("arm_up", None, 10.0, v(10.0, 10.0), 0.0),
    ];
    p.ik = vec![IkChain::two_bone("leg", "up", "lo", "tgt", 1.0)];
    let ids = |po: &ParamOptions| enumerate_params(&p, po).into_iter().map(|q| q.id).collect::<Vec<_>>();
    assert!(ids(&ParamOptions::default()).contains(&"ik:leg:mix".to_string()));
    let only = ids(&ParamOptions { only_bones: vec!["arm_up".into()], bone_props: vec!["angle".into()], ..Default::default() });
    assert_eq!(only, vec!["bone:arm_up:angle".to_string()], "no IK mix, constraint or deformer parameter may be sampled");
    let only_bone = ids(&ParamOptions { only_bones: vec!["arm_up".into()], ..Default::default() });
    assert!(only_bone.iter().all(|i| i.starts_with("bone:arm_up:")), "{only_bone:?}");
    // an IK chain whose bones are selected keeps its mix when no property filter is given
    let chain = ids(&ParamOptions { only_bones: vec!["up".into()], ..Default::default() });
    assert!(chain.contains(&"ik:leg:mix".to_string()));
    assert!(!ids(&ParamOptions { only_bones: vec!["up".into()], bone_props: vec!["angle".into()], ..Default::default() }).contains(&"ik:leg:mix".to_string()));
}

#[test]
fn ik_unreachable_and_limits() {
    let mut p = Project::new("ik", 400.0, 300.0, 24.0, 4);
    p.bones = vec![
        Bone::new("up", None, 50.0, v(100.0, 150.0), 0.0).with_angle_limits(-45.0, 45.0),
        Bone::new("lo", Some("up"), 50.0, v(50.0, 0.0), 0.0),
        Bone::new("tgt", None, 10.0, v(180.0, 150.0), 0.0),
    ];
    p.ik = vec![IkChain::two_bone("leg", "up", "lo", "tgt", 1.0)];
    let po = ParamOptions { deformers: false, offset_range: Some(150.0), ..Default::default() };
    let mut f = Fuzzer::new(&p, None, &po, Thresholds::default(), 0.0).unwrap();
    let rv = f.rest_values();
    let r = f.eval(0, &rv);
    assert!(!r.failed, "{:?} {:?}", r.kinds, r.metrics.ik_error_fraction);
    let kx = f.params.iter().position(|q| q.id == "bone:tgt:offx").unwrap();
    let mut vals = f.rest_values();
    vals[kx] = 120.0; // target 220 px away, reach 100
    let r = f.eval(0, &vals);
    assert!(r.kinds.contains(&"ik_unreachable".to_string()), "{:?}", r.metrics);
    assert_eq!(r.metrics.ik_worst_chain.as_deref(), Some("leg"));
    // ik mix 0 turns the chain off, so the target no longer matters
    let km = f.params.iter().position(|q| q.id == "ik:leg:mix").unwrap();
    vals[km] = 0.0;
    assert!(!f.eval(0, &vals).kinds.contains(&"ik_unreachable".to_string()));
    // limits: inside is fine, a margin makes the sampler ask for more than the rig allows
    let ka = f.params.iter().position(|q| q.id == "bone:up:angle").unwrap();
    assert_eq!((f.params[ka].lo, f.params[ka].hi), (-45.0, 45.0));
    let mut v2 = f.rest_values();
    v2[ka] = 60.0;
    let r = f.eval(0, &v2);
    assert_eq!(r.metrics.limit_violations, 1);
    assert!(r.kinds.contains(&"limit".to_string()));
    let wide = ParamOptions { limit_margin: 30.0, deformers: false, ..Default::default() };
    let ps = enumerate_params(&p, &wide);
    let a = ps.iter().find(|q| q.id == "bone:up:angle").unwrap();
    assert_eq!((a.lo, a.hi, a.limit), (-75.0, 75.0, Some([-45.0, 45.0])));
}

#[test]
fn canvas_escape_and_param_enumeration() {
    let mut p = Project::new("c", 200.0, 200.0, 24.0, 4);
    p.bones = vec![Bone::new("root", None, 40.0, v(100.0, 100.0), 0.0), Bone::new("arm", Some("root"), 40.0, v(40.0, 0.0), 0.0)];
    let mut l = Layer::vector("blob", vec![VPath::rect("r", -10.0, -10.0, 20.0, 20.0)]);
    l.bone = Some("root".into());
    p.layers = vec![l];
    let po = ParamOptions { offset_range: Some(500.0), deformers: false, ..Default::default() };
    let mut f = Fuzzer::new(&p, None, &po, Thresholds::default(), 0.0).unwrap();
    let kx = f.params.iter().position(|q| q.id == "bone:root:offx").unwrap();
    let mut vals = f.rest_values();
    assert!(!f.eval(0, &vals).failed);
    vals[kx] = 450.0;
    let r = f.eval(0, &vals);
    assert!(r.kinds.contains(&"off_canvas".to_string()), "{:?}", r.metrics);
    // enumeration: 4 params per bone, stable order, ids unique
    let ps = enumerate_params(&p, &po);
    assert_eq!(ps.len(), 8);
    let mut ids: Vec<&String> = ps.iter().map(|q| &q.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 8);
    // a project without bones has nothing to fuzz
    let empty = Project::new("e", 10.0, 10.0, 24.0, 1);
    assert!(Fuzzer::new(&empty, None, &po, Thresholds::default(), 0.0).is_err());
}

#[test]
fn driven_outputs_are_not_probed_and_inputs_use_the_curve_range() {
    let mut p = Project::new("d", 200.0, 200.0, 24.0, 4);
    p.bones = vec![Bone::new("ctrl", None, 10.0, v(20.0, 20.0), 0.0), Bone::new("bulge", None, 10.0, v(100.0, 100.0), 0.0)];
    let curve = Track::with_keys(&[(0.0, 1.0, Ease::Linear), (10.0, 2.0, Ease::Linear)]);
    p.channels = vec![Channel::driven("bulge", Prop::BoneScale, "ctrl", Prop::BoneOffsetX, curve)];
    let ps = enumerate_params(&p, &ParamOptions { deformers: false, ..Default::default() });
    assert!(ps.iter().all(|q| q.id != "bone:bulge:scale"));
    let x = ps.iter().find(|q| q.id == "bone:ctrl:offx").unwrap();
    assert!((x.lo + 1.0).abs() < 1e-5 && (x.hi - 11.0).abs() < 1e-5, "{} {}", x.lo, x.hi);
    // the driver still works inside the fuzzer: ctrl offset 10 -> bulge scale 2 (check through the bone's world scale)
    let mut f = Fuzzer::new(&p, None, &ParamOptions { deformers: false, ..Default::default() }, Thresholds::default(), 0.0).unwrap();
    let kx = f.params.iter().position(|q| q.id == "bone:ctrl:offx").unwrap();
    let mut vals = f.rest_values();
    vals[kx] = 10.0;
    assert!(!f.eval(0, &vals).metrics.non_finite);
}

#[test]
fn report_schema_and_heatmap_svg() {
    let p = fold_rig();
    let o = FuzzOptions { samples: 60, strategy: Strategy::Random, minimize: true, params: ParamOptions { deformers: false, ..Default::default() }, ..Default::default() };
    let rep = run(&p, None, &o).unwrap();
    let j: serde_json::Value = serde_json::from_str(&rep.to_json()).unwrap();
    assert_eq!(j["schema"], "osatura-fuzz-report");
    assert_eq!(j["version"], 1);
    for k in ["project", "strategy", "seed", "frame", "thresholds", "params", "summary", "marginals", "per_bone", "pairs", "worst", "minimal", "heat"] {
        assert!(j.get(k).is_some(), "missing {k}");
    }
    let s = &j["summary"];
    assert_eq!(s["samples"], 60);
    for k in ["failures", "failure_rate", "by_kind", "max_severity", "seconds", "ms_per_sample", "baseline_severity"] {
        assert!(s.get(k).is_some(), "missing summary.{k}");
    }
    assert!(j["worst"].as_array().unwrap().len() <= 20);
    assert!(j["worst"][0]["severity"].as_f64().unwrap() >= j["worst"][1]["severity"].as_f64().unwrap());
    assert!(j["minimal"]["verified"].as_bool().unwrap());
    assert!(j["heat"]["geos"][0]["max_stress"].as_array().unwrap().len() == 4);
    // same seed, same report (apart from timings)
    let again = run(&p, None, &o).unwrap();
    assert_eq!(again.summary.failures, rep.summary.failures);
    assert_eq!(again.worst[0].values, rep.worst[0].values);
    // SVG is well-formed and carries the overlay + bars
    let svg = heatmap_svg(&rep);
    well_formed(&svg);
    assert!(svg.contains("<polygon") && svg.contains("Extra failure rate"));
    // worst-case stress is at the threshold or above somewhere, so a warm colour must appear
    assert!(rep.heat.geos[0].max_stress.iter().cloned().fold(0.0, f32::max) >= 1.0);
    let g = grid(&p, None, &ParamOptions { deformers: false, ..Default::default() }, Thresholds::default(), 0.0, "bone:b:angle", "bone:b:scale", 9).unwrap();
    assert_eq!(g.severity.len(), 81);
    let gs = grid_svg(&g);
    well_formed(&gs);
    assert_eq!(gs.matches("<rect").count(), 81 + 24 + 1);
    // errors
    assert!(grid(&p, None, &ParamOptions::default(), Thresholds::default(), 0.0, "bone:zz:angle", "bone:b:angle", 4).is_err());
    assert!(grid(&p, None, &ParamOptions::default(), Thresholds::default(), 0.0, "bone:b:angle", "bone:b:angle", 4).is_err());
    assert!(grid(&p, None, &ParamOptions::default(), Thresholds::default(), 0.0, "bone:a:angle", "bone:b:angle", 1).is_err());
}

#[test]
fn ignored_kinds_do_not_count() {
    let mut p = Project::new("ik", 400.0, 300.0, 24.0, 4);
    p.bones = vec![Bone::new("up", None, 50.0, v(100.0, 150.0), 0.0), Bone::new("lo", Some("up"), 50.0, v(50.0, 0.0), 0.0), Bone::new("tgt", None, 10.0, v(180.0, 150.0), 0.0)];
    p.ik = vec![IkChain::two_bone("leg", "up", "lo", "tgt", 1.0)];
    let mut o = FuzzOptions { samples: 80, strategy: Strategy::Halton, params: ParamOptions { deformers: false, offset_range: Some(200.0), bone_props: vec!["offx".into()], only_bones: vec!["tgt".into()], ..Default::default() }, ..Default::default() };
    let a = run(&p, None, &o).unwrap();
    assert!(a.summary.by_kind.contains_key("ik_unreachable"));
    o.ignore = vec!["ik_unreachable".into()];
    let b = run(&p, None, &o).unwrap();
    assert_eq!(b.summary.failures, 0);
    o.ignore = vec!["bogus".into()];
    assert!(run(&p, None, &o).unwrap_err().contains("unknown failure kind"));
}

#[test]
fn ramp_is_monotone_in_range_and_safe() {
    assert_eq!(ramp(0.0), [44, 79, 158]);
    assert_eq!(ramp(1.0), [125, 26, 12]);
    assert_eq!(ramp(f32::NAN), ramp(0.0));
    assert_eq!(ramp(-5.0), ramp(0.0));
    assert_eq!(ramp(9.0), ramp(1.0));
    assert_eq!(stress_color(0.0), "#2c4f9e");
    // below the threshold: blue side; at the threshold and above: warm side; monotone in between
    let warm = |c: [u8; 3]| c[0] > c[2];
    assert!(!warm(stress_rgb(0.99)) && warm(stress_rgb(1.0)) && warm(stress_rgb(10.0)));
    assert!(stress_t(0.5) < stress_t(0.9) && stress_t(0.9) < stress_t(1.0) && stress_t(1.0) < stress_t(2.0) && stress_t(2.0) < stress_t(3.9));
    assert_eq!(stress_t(f32::NAN), 0.0);
}

/// Minimal XML well-formedness check: balanced tags, quoted attributes, escaped text.
fn well_formed(s: &str) {
    let mut stack: Vec<String> = vec![];
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'<' {
            let end = s[i..].find('>').expect("unterminated tag") + i;
            let tag = &s[i + 1..end];
            assert!(!tag.contains('<'), "stray '<' in {tag}");
            if let Some(name) = tag.strip_prefix('/') {
                assert_eq!(stack.pop().as_deref(), Some(name.trim()), "mismatched close {name}");
            } else if !tag.starts_with('?') && !tag.starts_with('!') {
                let selfc = tag.ends_with('/');
                let name = tag.split_whitespace().next().unwrap().trim_end_matches('/').to_string();
                assert_eq!(tag.matches('"').count() % 2, 0, "odd quotes in {tag}");
                if !selfc {
                    stack.push(name);
                }
            }
            i = end + 1;
        } else {
            if b[i] == b'&' {
                let semi = s[i..].find(';').expect("bare &");
                assert!(["&amp;", "&lt;", "&gt;", "&quot;"].contains(&&s[i..i + semi + 1]), "bad entity {}", &s[i..i + semi + 1]);
            }
            i += 1;
        }
    }
    assert!(stack.is_empty(), "unclosed {stack:?}");
}

#[test]
fn svg_escapes_names() {
    let mut p = fold_rig();
    p.name = "a<b>&\"c".into();
    let rep = run(&p, None, &FuzzOptions { samples: 4, params: ParamOptions { deformers: false, ..Default::default() }, ..Default::default() }).unwrap();
    well_formed(&heatmap_svg(&rep));
}

#[test]
fn path_layers_get_edge_metrics_and_self_intersection() {
    // a ribbon outline that crosses itself once the tip bone bends far enough
    let mut p = Project::new("ribbon", 300.0, 200.0, 24.0, 4);
    p.bones = vec![Bone::new("a", None, 80.0, v(40.0, 100.0), 0.0), Bone::new("b", Some("a"), 80.0, v(80.0, 0.0), 0.0)];
    let pts = [v(40.0, 90.0), v(120.0, 90.0), v(200.0, 90.0), v(200.0, 110.0), v(120.0, 110.0), v(40.0, 110.0)];
    let mut path = VPath::polygon("ribbon", &pts);
    for (i, a) in path.anchors.iter_mut().enumerate() {
        let bone = if i == 1 || i == 4 { None } else if pts[i].x > 120.0 { Some("b") } else { Some("a") };
        if let Some(b) = bone {
            a.weights = vec![BoneWeight { bone: b.into(), weight: 1.0 }];
        }
    }
    p.layers = vec![Layer::vector("rib", vec![path])];
    let po = ParamOptions { only_bones: vec!["b".into()], deformers: false, ..Default::default() };
    let mut f = Fuzzer::new(&p, None, &po, Thresholds::default(), 0.0).unwrap();
    let k = f.params.iter().position(|q| q.id == "bone:b:angle").unwrap();
    let rv = f.rest_values();
    assert!(!f.eval(0, &rv).failed);
    let mut seen_cross = false;
    for a in (-180..=180).step_by(10) {
        let mut vals = f.rest_values();
        vals[k] = a as f32;
        let r = f.eval(0, &vals);
        assert!(!r.metrics.non_finite);
        seen_cross |= r.metrics.new_self_intersections > 0 || r.metrics.flipped_triangles > 0;
    }
    assert!(seen_cross, "a fully bent ribbon must fold or cross itself");
}
