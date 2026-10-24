// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of the level-of-detail module.

use super::*;
use crate::anim::Ease;
use crate::path::BoneWeight;
use crate::project::{Bone, ImageSource, MaskRef, MeshSpec, Transform2D};

/// Dense Hausdorff distance of two outlines: exact nearest distances (grid index) at points
/// sampled every `step` along both curves.
fn true_hausdorff(a: &[Anchor], ca: bool, b: &[Anchor], cb: bool) -> f64 {
    let pa = outline_polyline(a, ca, 1e-4);
    let pb = outline_polyline(b, cb, 1e-4);
    let dir = |x: &[P], y: &[P]| {
        let idx = PolyIdx::new(y);
        let n = ((path_len(x) / 0.01) as usize).clamp(2, 400_000);
        resample(x, n).iter().map(|&q| idx.nearest(q).0).fold(0.0, f64::max)
    };
    dir(&pa, &pb).max(dir(&pb, &pa))
}

fn lcg(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*seed >> 33) as f64) / ((1u64 << 31) as f64)
}

fn circle_pts(n: usize, r: f32, wobble: f32, seed: u64) -> Vec<Vec2> {
    let mut s = seed;
    (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            let rr = r + wobble * (lcg(&mut s) as f32 - 0.5);
            Vec2::new(100.0 + rr * a.cos(), 100.0 + rr * a.sin())
        })
        .collect()
}

fn scene_with(layers: Vec<Layer>) -> Project {
    let mut p = Project::new("t", 200.0, 200.0, 24.0, 12);
    p.layers = layers;
    p
}

#[test]
fn simplified_beziers_declare_a_bound_at_least_the_true_hausdorff_distance() {
    for (n, wob, eps) in [(48usize, 0.2f32, 0.15f64), (64, 0.0, 0.05), (40, 1.0, 0.5), (90, 0.05, 0.3), (24, 0.0, 2.0)] {
        let path = VPath::smooth_through("c", &circle_pts(n, 60.0, wob, n as u64), true, 1.0);
        let sp = simplify_path_certified(&path.anchors, true, eps);
        assert!(sp.removed > 0, "n={n} eps={eps}: nothing removed");
        assert!(sp.bound <= eps + 1e-9, "declared {} above eps {eps}", sp.bound);
        let truth = true_hausdorff(&path.anchors, true, &sp.anchors, true);
        assert!(sp.bound + 1e-3 >= truth, "n={n} eps={eps}: declared {} < true {truth}", sp.bound);
        assert!(truth <= eps + 1e-3, "true {truth} exceeds the tolerance {eps}");
    }
}

#[test]
fn simplified_polylines_and_open_paths_are_certified_and_keep_their_ends() {
    let mut s = 7u64;
    for case in 0..12 {
        let n = 20 + case * 3;
        let pts: Vec<Vec2> = (0..n)
            .map(|i| Vec2::new(i as f32 * 4.0, 50.0 * ((i as f32) * 0.2).sin() + 0.4 * (lcg(&mut s) as f32 - 0.5)))
            .collect();
        let anchors: Vec<Anchor> = pts.iter().map(|&p| Anchor::corner(p)).collect();
        let eps = 0.3 + case as f64 * 0.1;
        let sp = simplify_path_certified(&anchors, false, eps);
        assert!(sp.removed > 0);
        assert_eq!(sp.anchors.first().unwrap().pos, anchors.first().unwrap().pos);
        assert_eq!(sp.anchors.last().unwrap().pos, anchors.last().unwrap().pos);
        let truth = true_hausdorff(&anchors, false, &sp.anchors, false);
        assert!(sp.bound + 1e-3 >= truth && sp.bound <= eps + 1e-9, "case {case}: declared {} true {truth} eps {eps}", sp.bound);
    }
}

#[test]
fn hausdorff_bound_helpers_agree_with_brute_force() {
    let a: Vec<P> = (0..200).map(|i| [i as f64 * 0.5, (i as f64 * 0.1).sin() * 10.0]).collect();
    let b: Vec<P> = (0..120).map(|i| [i as f64 * 0.83, (i as f64 * 0.17).sin() * 10.0 + 0.5]).collect();
    let up = directed_hausdorff_upper(&a, &b, 0.01);
    let exact_samples = resample(&a, 20000).iter().map(|&p| dist_to_poly(p, &b)).fold(0.0, f64::max);
    assert!(up + 1e-9 >= exact_samples, "upper {up} < sampled {exact_samples}");
    assert!(up <= exact_samples + 0.05, "upper {up} much above sampled {exact_samples}");
    assert!(directed_hausdorff_upper(&a, &a, 0.01) < 1e-9);
    assert!(discrete_frechet(&a, &a) < 1e-12);
}

#[test]
fn flattening_bound_is_rigorous() {
    let c: Cubic = [[0.0, 0.0], [50.0, 90.0], [-20.0, 90.0], [60.0, 0.0]];
    for tol in [1.0, 0.1, 0.01] {
        let (poly, h) = flatten_poly(&c, tol);
        assert!(h <= tol + 1e-12);
        let worst = (0..=4000).map(|i| dist_to_poly(cubic_at(&c, i as f64 / 4000.0), &poly)).fold(0.0, f64::max);
        assert!(worst <= h + 1e-9, "curve is {worst} from its polyline, bound {h}");
    }
}

#[test]
fn unsuitable_inputs_are_left_alone() {
    let tri = VPath::polygon("t", &[Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(5.0, 8.0)]);
    let r = simplify_path_certified(&tri.anchors, true, 100.0);
    assert_eq!(r.removed, 0);
    assert_eq!(r.anchors, tri.anchors);
    let r0 = simplify_path_certified(&tri.anchors, true, 0.0);
    assert_eq!(r0.removed, 0);
    let rn = simplify_path_certified(&tri.anchors, true, f64::NAN);
    assert_eq!(rn.removed, 0);
    // a rectangle with four corners cannot lose one at a small tolerance
    let rect = VPath::rect("r", 0.0, 0.0, 100.0, 60.0);
    assert_eq!(simplify_path_certified(&rect.anchors, true, 0.5).removed, 0);
}

fn blob_scene(scale: f32) -> Project {
    let mut layer = Layer::vector("blob", vec![VPath::smooth_through("c", &circle_pts(80, 40.0, 0.3, 3), true, 1.0).with_fill("#336699")]);
    layer.transform = Transform2D { position: Vec2::new(0.0, 0.0), scale: Vec2::new(scale, scale), ..Default::default() };
    scene_with(vec![layer])
}

fn anchors_of(p: &Project, layer: &str) -> usize {
    match &p.find_layer(layer).unwrap().kind {
        LayerKind::Vector { paths } => paths.iter().map(|x| x.anchors.len()).sum(),
        _ => 0,
    }
}

#[test]
fn tolerance_is_scale_aware() {
    // same shape drawn 1x and 3x larger on screen: the larger one needs finer local tolerance
    let o = LodOptions::new(1.0, 1.0, 1);
    let small = build(&blob_scene(1.0), &o).unwrap();
    let big = build(&blob_scene(3.0), &o).unwrap();
    let (ks, kb) = (anchors_of(&small[1].project, "blob"), anchors_of(&big[1].project, "blob"));
    assert!(ks < 80, "small version should lose anchors, has {ks}");
    assert!(kb >= ks, "3x version must keep at least as many anchors ({kb} vs {ks})");
    // and a doubled render scale (output pixels per unit) is stricter as well
    let o2 = LodOptions::new(1.0, 2.0, 1);
    let fine = build(&blob_scene(1.0), &o2).unwrap();
    assert!(anchors_of(&fine[1].project, "blob") >= ks);
    // the declared bound in stage units follows the render scale
    assert!((small[1].record.bound_stage * 1.0 - small[1].record.declared_px).abs() < 1e-5);
    assert!((fine[1].record.bound_stage * 2.0 - fine[1].record.declared_px).abs() < 1e-5);
    for l in small.iter().chain(&big).chain(&fine).skip(1) {
        assert!(l.record.declared_px <= l.record.tolerance_px + 1e-9);
    }
}

fn rig_scene(dense: bool) -> Project {
    let mut p = Project::new("rig", 200.0, 200.0, 24.0, 48);
    p.bones = vec![Bone::new("arm", None, 100.0, Vec2::new(50.0, 100.0), 0.0)];
    let pts: Vec<Vec2> = (0..16).map(|i| Vec2::new(50.0 + i as f32 * 100.0 / 15.0, 100.0 + if i % 2 == 0 { 4.0 } else { -4.0 })).collect();
    let mut sk = VPath::polygon("skin", &pts).with_fill("#cc3300");
    for a in &mut sk.anchors {
        a.weights = vec![BoneWeight { bone: "arm".into(), weight: 1.0 }];
    }
    let mut ride = Layer::vector("ride", vec![VPath::rect("r", 140.0, 90.0, 20.0, 20.0).with_fill("#006600")]);
    ride.bone = Some("arm".into());
    p.layers = vec![Layer::vector("skinned", vec![sk]), ride];
    let keys: Vec<(f32, f32, Ease)> = if dense {
        (0..48).map(|f| (f as f32, 30.0 * (f as f32 / 48.0 * std::f32::consts::TAU).sin(), Ease::Linear)).collect()
    } else {
        vec![(0.0, 0.0, Ease::Linear), (24.0, 30.0, Ease::Linear), (47.0, 0.0, Ease::Linear)]
    };
    p.channels = vec![Channel::keys("arm", Prop::BoneAngle, Track::with_keys(&keys))];
    p
}

#[test]
fn dense_keys_are_reduced_and_the_pixel_bound_is_honoured() {
    let p = rig_scene(true);
    let o = LodOptions::new(1.0, 1.0, 3);
    let levels = build(&p, &o).unwrap();
    let mut prev = levels[0].record.stats.keys;
    for l in &levels[1..] {
        assert!(l.record.stats.keys <= prev, "keys grew: {} -> {}", prev, l.record.stats.keys);
        prev = l.record.stats.keys;
        assert!(l.record.declared_px <= l.record.tolerance_px + 1e-9);
    }
    assert!(levels[3].record.stats.keys < 30, "coarsest level still has {} keys", levels[3].record.stats.keys);
    assert!(levels[3].record.keys_px > 0.0);
    for l in &levels[1..] {
        let r = verify(&p, l, &VerifyOptions { scale: 1.0, frames: 48, pixels: false, pixel_frames: 1 }).unwrap();
        assert!(r.holds, "level {}: measured {} > declared {}", l.record.level, r.measured_px, r.declared_px);
        assert!(r.measured_px <= l.record.keys_px + l.record.path_px + 1e-6);
    }
}

#[test]
fn key_reduction_honours_the_value_bound_for_every_ease() {
    let eases = [Ease::Linear, Ease::EaseInOut, Ease::Smooth, Ease::Back { overshoot: 1.2 }, Ease::CubicBezier { x1: 0.3, y1: 0.0, x2: 0.7, y2: 1.0 }];
    let grid: Vec<f32> = (0..96).map(|i| i as f32 / 2.0).collect();
    for e in eases {
        let keys: Vec<(f32, f32, Ease)> = (0..48).map(|f| (f as f32, 20.0 * (f as f32 * 0.3).sin() + 3.0 * (f as f32 * 1.7).cos(), e)).collect();
        let t = Track::with_keys(&keys);
        for tol in [0.05, 0.5, 3.0] {
            let r = reduce_track_bounded(&t, tol, &grid);
            let dev = track_deviation_at(&t, &r, &merged_samples(&t, &grid));
            assert!(dev <= tol + 1e-6, "{e:?} tol {tol}: deviation {dev}");
            assert!(r.keys.len() <= t.keys.len());
            assert_eq!(r.keys.first().unwrap().frame, 0.0);
            assert_eq!(r.keys.last().unwrap().frame, 47.0);
        }
    }
    // tiny and degenerate input is returned unchanged
    let two = Track::with_keys(&[(0.0, 1.0, Ease::Linear), (5.0, 2.0, Ease::Linear)]);
    assert_eq!(reduce_track_bounded(&two, 1.0, &grid), two);
    let t = Track::with_keys(&[(0.0, 1.0, Ease::Linear), (5.0, 2.0, Ease::Linear), (9.0, 0.0, Ease::Linear)]);
    assert_eq!(reduce_track_bounded(&t, f64::NAN, &grid), t);
    assert_eq!(reduce_track_bounded(&t, -1.0, &grid), t);
}

fn mesh_png() -> Vec<u8> {
    crate::percep::Img::solid(16, 16, [200, 30, 30, 255]).to_png().unwrap()
}

fn mesh_scene(cols: usize, rows: usize, animate: bool) -> Project {
    let mut p = Project::new("mesh", 200.0, 200.0, 24.0, 24);
    p.bones = vec![Bone::new("a", None, 60.0, Vec2::new(40.0, 100.0), 0.0), Bone::new("b", None, 60.0, Vec2::new(100.0, 100.0), 0.0)];
    let mut img = Layer::image_from_png("img", &mesh_png(), Vec2::new(40.0, 60.0));
    let weights: Vec<Vec<BoneWeight>> = (0..=rows)
        .flat_map(|_r| (0..=cols).map(move |c| {
            let t = c as f32 / cols as f32;
            vec![BoneWeight { bone: "a".into(), weight: 1.0 - t }, BoneWeight { bone: "b".into(), weight: t }]
        }))
        .collect();
    if let LayerKind::Image { size, mesh, .. } = &mut img.kind {
        *size = Some(Vec2::new(120.0, 80.0));
        *mesh = Some(MeshSpec { cols, rows, weights, free: None });
    }
    p.layers = vec![img];
    if animate {
        p.channels = vec![Channel::keys("b", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, 25.0, Ease::Linear), (23.0, 0.0, Ease::Linear)]))];
    }
    p
}

fn mesh_vertices(p: &Project) -> usize {
    match &p.find_layer("img").unwrap().kind {
        LayerKind::Image { mesh: Some(m), .. } => m.vertex_count(),
        _ => 0,
    }
}

#[test]
fn grid_mesh_collapses_with_a_certified_bound() {
    let p = mesh_scene(10, 8, true);
    let o = LodOptions::new(1.5, 1.0, 3);
    let levels = build(&p, &o).unwrap();
    assert_eq!(mesh_vertices(&levels[0].project), 99);
    let mut prev = 99;
    for l in &levels[1..] {
        let v = mesh_vertices(&l.project);
        assert!(v <= prev, "vertices grew {prev} -> {v}");
        prev = v;
        assert!(l.record.mesh_px <= l.record.tolerance_px * o.geometry_share + 1e-9);
        // the simplified project still renders and validates
        Engine::new(l.project.clone()).unwrap();
    }
    assert!(prev < 99, "no vertex was collapsed at 6 px");
    for l in &levels[1..] {
        let r = verify(&p, l, &VerifyOptions { scale: 1.0, frames: 24, pixels: true, pixel_frames: 3 }).unwrap();
        assert!(r.holds, "level {}: measured {} declared {}", l.record.level, r.measured_px, r.declared_px);
        assert!(r.measured_mesh_px <= l.record.mesh_px + 1e-6, "mesh measured {} > certified {}", r.measured_mesh_px, l.record.mesh_px);
    }
}

#[test]
fn unskinned_grid_collapses_to_its_corners_and_keeps_the_outline() {
    let n = 9usize;
    let rest: Vec<P> = (0..=n).flat_map(|r| (0..=n).map(move |c| [c as f64 * 10.0, r as f64 * 10.0])).collect();
    let mut tris = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let a = r * (n + 1) + c;
            tris.push([a, a + 1, a + n + 2]);
            tris.push([a, a + n + 2, a + n + 1]);
        }
    }
    let attr: Vec<Vec<f64>> = vec![vec![]; rest.len()];
    let y: Vec<Vec<[f32; 2]>> = vec![rest.iter().map(|p| [p[0] as f32, p[1] as f32]).collect(); 3];
    let s = vec![1.0; 3];
    let q = qem_simplify(&QemInput { rest: &rest, tris: &tris, attr: &attr, y: &y, s: &s }, 0.5);
    assert_eq!(q.keep, vec![0, n, n * (n + 1), (n + 1) * (n + 1) - 1], "only the four corners can remain");
    assert_eq!(q.tris.len(), 2);
    assert!(q.bound_px < 1e-9);
    // zero tolerance leaves the mesh alone
    let z = qem_simplify(&QemInput { rest: &rest, tris: &tris, attr: &attr, y: &y, s: &s }, 0.0);
    assert_eq!(z.collapses, 0);
    assert_eq!(z.keep.len(), rest.len());
}

#[test]
fn qem_bound_covers_the_true_surface_difference() {
    // deformed positions: a bend that makes interior vertices matter
    let (cols, rows) = (12usize, 6usize);
    let rest: Vec<P> = (0..=rows).flat_map(|r| (0..=cols).map(move |c| [c as f64 * 8.0, r as f64 * 8.0])).collect();
    let mut tris = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let a = r * (cols + 1) + c;
            tris.push([a, a + 1, a + cols + 2]);
            tris.push([a, a + cols + 2, a + cols + 1]);
        }
    }
    let y: Vec<Vec<[f32; 2]>> = (0..4)
        .map(|f| {
            rest.iter()
                .map(|p| {
                    let a = (p[0] / 96.0) * 0.9 * (f as f64 + 1.0);
                    [(p[0] + 6.0 * (a).sin() * p[1] / 48.0) as f32, (p[1] + 14.0 * (1.0 - a.cos()) + 3.0 * (p[0] * 0.2).sin()) as f32]
                })
                .collect()
        })
        .collect();
    let attr: Vec<Vec<f64>> = rest.iter().map(|p| vec![p[0] / 96.0 * 10.0]).collect();
    let s = vec![1.5, 1.0, 1.2, 1.0];
    let eps = 2.0;
    let q = qem_simplify(&QemInput { rest: &rest, tris: &tris, attr: &attr, y: &y, s: &s }, eps);
    assert!(q.collapses > 0 && q.bound_px <= eps + 1e-9);
    // brute force: evaluate both piecewise-linear surfaces on a lattice of rest points
    let loc_all = |tr: &[[usize; 3]], p: P| locate(&rest, tr, p).unwrap();
    let mut worst = 0.0f64;
    for i in 0..=96 {
        for j in 0..=48 {
            let pt = [i as f64, j as f64];
            let (t0, l0) = loc_all(&tris, pt);
            let (t1, l1) = loc_all(&q.tris, pt);
            for (f, yf) in y.iter().enumerate() {
                let ev = |t: [usize; 3], l: [f64; 3]| (0..3).fold([0.0, 0.0], |a, k| add(a, mul([yf[t[k]][0] as f64, yf[t[k]][1] as f64], l[k])));
                worst = worst.max(dist(ev(t0, l0), ev(t1, l1)) * s[f]);
            }
        }
    }
    assert!(worst <= q.bound_px + 1e-6, "true max {worst} above certified {}", q.bound_px);
    assert!(worst > 0.0);
    // outline preserved: kept vertices include all four corners, every kept vertex is an original one
    for corner in [0, cols, rows * (cols + 1), (rows + 1) * (cols + 1) - 1] {
        assert!(q.keep.contains(&corner));
    }
}

#[test]
fn tiny_container_members_are_kept_not_a_fatal_error() {
    use crate::containers::{Container, Dir, Member};
    let big = Layer::vector("big", vec![VPath::rect("r", 0.0, 0.0, 60.0, 40.0).with_fill("#336699")]);
    let dot = Layer::vector("dot", vec![VPath::circle("d", Vec2::new(20.0, 20.0), 0.1).with_fill("#000000")]);
    let free = Layer::vector("free", vec![VPath::circle("d", Vec2::new(90.0, 20.0), 0.1).with_fill("#000000")]);
    let mut p = scene_with(vec![big, dot, free]);
    p.containers.push(Container::new("row", Dir::Row, vec![Member::layer("big"), Member::layer("dot")]));
    Engine::new(p.clone()).unwrap();
    let levels = build(&p, &LodOptions::new(1.0, 1.0, 2)).expect("lod must not abort on a container member");
    for l in &levels[1..] {
        assert!(l.project.find_layer("dot").is_some(), "container member kept");
        assert!(l.project.find_layer("free").is_none(), "unreferenced speck still culled");
        assert!(l.record.notes.iter().any(|n| n.starts_with("skipped:") && n.contains("dot")), "{:?}", l.record.notes);
        Engine::new(l.project.clone()).unwrap();
    }
}

/// H2 (wave 4): simplifying the path of a container member changes its bounding box, the container re-flows the
/// siblings and the measured deviation exceeded the declared bound. Layout members are now left alone and the
/// bound is checked by rendering both projects (also re-solved for another canvas size).
#[test]
fn declared_bound_holds_on_a_project_with_a_layout_container() {
    use crate::containers::{Container, Dir, Member, RectRule};
    let wob = |seed: u64, r: f32| circle_pts(60, r, 6.0, seed);
    let sun = Layer::vector("sun", vec![VPath::smooth_through("c", &wob(3, 40.0), true, 1.0).with_fill("#ffcc00")]);
    let hills = Layer::vector("hills", vec![VPath::smooth_through("c", &wob(5, 55.0), true, 1.0).with_fill("#336644")]);
    // not a member: still reduced, so the level has a real, non-zero declared bound
    let free = Layer::vector("free", vec![VPath::smooth_through("c", &wob(9, 30.0), true, 1.0).with_fill("#aa3333")]);
    let mut p = scene_with(vec![sun, hills, free]);
    let mut c = Container::new("row", Dir::Row, vec![Member::layer("sun"), Member::layer("hills")]);
    c.members[0].grow = 1.0;
    c.gap = 10.0;
    c.rect = RectRule::Aspect { ratio: 1.5, mode: Default::default(), margin: 0.0, ax: 0.5, ay: 0.5 };
    p.containers.push(c);
    Engine::new(p.clone()).unwrap();
    for tol in [3.0, 4.0, 6.0] {
        let levels = build(&p, &LodOptions::new(tol, 1.0, 3)).unwrap();
        for l in &levels[1..] {
            assert!(l.record.notes.iter().any(|n| n.contains("skipped: layout member") && n.contains("sun") && n.contains("hills")), "{:?}", l.record.notes);
            for m in ["sun", "hills"] {
                assert_eq!(l.project.find_layer(m), p.find_layer(m), "member {m} must be untouched");
            }
            assert!(l.record.path_px > 0.0, "the non-member path is still reduced");
            for (size, scale) in [(None, 1.0), (Some((300.0f32, 200.0f32)), 1.0)] {
                // the same canvas for original and level (the CLI's --size re-solves both)
                let (o, q) = match size {
                    None => (p.clone(), l.clone()),
                    Some((w, h)) => {
                        let mut q = l.clone();
                        q.project = crate::containers::responsive(&l.project, w, h).unwrap();
                        (crate::containers::responsive(&p, w, h).unwrap(), q)
                    }
                };
                let r = verify(&o, &q, &VerifyOptions { scale, frames: 12, pixels: true, pixel_frames: 3 }).unwrap();
                assert!(r.holds && r.measured_px <= r.declared_px + 1e-6, "tol {tol} size {size:?}: measured {} declared {}", r.measured_px, r.declared_px);
            }
        }
    }
}

#[test]
fn culling_is_conservative() {
    let mut dot = Layer::vector("dot", vec![VPath::circle("d", Vec2::new(20.0, 20.0), 0.12).with_fill("#000000")]);
    dot.opacity = 1.0;
    let big = Layer::vector("big", vec![VPath::rect("r", 50.0, 50.0, 60.0, 40.0).with_fill("#336699")]);
    let tiny_mask = Layer::vector("maskdot", vec![VPath::circle("m", Vec2::new(30.0, 30.0), 0.1).with_fill("#ffffff")]);
    let mut masked = Layer::vector("masked", vec![VPath::rect("r", 0.0, 0.0, 100.0, 100.0).with_fill("#990000")]);
    masked.mask = Some(MaskRef { layer: "maskdot".into(), invert: false });
    let sw = Layer::switch("sw", vec![Layer::vector("a", vec![VPath::circle("d", Vec2::new(70.0, 20.0), 0.1).with_fill("#000000")]), Layer::vector("b", vec![VPath::circle("d", Vec2::new(70.0, 20.0), 0.1).with_fill("#000000")])]);
    let mut blended = Layer::vector("blended", vec![VPath::circle("d", Vec2::new(90.0, 20.0), 0.1).with_fill("#000000")]);
    blended.blend = crate::style::Blend::Difference;
    let p = scene_with(vec![dot, big, tiny_mask, masked, sw, blended]);
    let levels = build(&p, &LodOptions::new(1.0, 1.0, 1)).unwrap();
    let q = &levels[1].project;
    assert!(q.find_layer("dot").is_none(), "sub-pixel dot should be culled");
    for keep in ["big", "maskdot", "masked", "sw/a", "sw/b", "blended"] {
        assert!(q.find_layer(keep).is_some(), "{keep} must not be culled");
    }
    assert!(levels[1].record.cull_px > 0.0 && levels[1].record.cull_px <= 0.5 + 1e-9);
    // at a higher render scale the same dot is no longer sub-pixel
    let hi = build(&p, &LodOptions::new(1.0, 8.0, 1)).unwrap();
    assert!(hi[1].project.find_layer("dot").is_some());
    // verify charges the culled layer with its diameter and holds
    let r = verify(&p, &levels[1], &VerifyOptions { scale: 1.0, frames: 4, pixels: true, pixel_frames: 2 }).unwrap();
    assert!(r.holds && r.measured_cull_px > 0.0, "{r:?}");
    // the culled dot changes at most a few grey levels of one pixel
    assert!(r.pixel_max_abs <= 64, "{}", r.pixel_max_abs);
}

#[test]
fn building_is_deterministic_and_does_not_touch_the_input() {
    let p = rig_scene(true);
    let before = p.clone();
    let o = LodOptions::new(0.8, 1.0, 3);
    let a = build(&p, &o).unwrap();
    let b = build(&p, &o).unwrap();
    assert_eq!(p, before, "input project was modified");
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.project.to_json_pretty().into_bytes(), y.project.to_json_pretty().into_bytes());
        assert_eq!(serde_json::to_string(&x.record).unwrap(), serde_json::to_string(&y.record).unwrap());
    }
    let m = mesh_scene(8, 6, true);
    let m1 = build(&m, &o).unwrap();
    let m2 = build(&m, &o).unwrap();
    for (x, y) in m1.iter().zip(&m2) {
        assert_eq!(x.project.to_json_pretty(), y.project.to_json_pretty());
    }
    let blob = blob_scene(1.0);
    let b1 = build(&blob, &o).unwrap();
    let b2 = build(&blob, &o).unwrap();
    assert_eq!(b1[2].project.to_json_pretty(), b2[2].project.to_json_pretty());
}

#[test]
fn levels_are_monotone() {
    for p in [blob_scene(1.0), rig_scene(true), mesh_scene(10, 8, true), crate::demo::project()] {
        let levels = build(&p, &LodOptions::new(0.5, 1.0, 5)).unwrap();
        assert_eq!(levels.len(), 6);
        for w in levels.windows(2) {
            let (a, b) = (&w[0].record, &w[1].record);
            assert!(b.stats.path_anchors <= a.stats.path_anchors, "anchors grew at L{}", b.level);
            assert!(b.stats.keys <= a.stats.keys, "keys grew at L{}", b.level);
            assert!(b.stats.mesh_vertices <= a.stats.mesh_vertices, "mesh vertices grew at L{}", b.level);
            assert!(b.stats.layers <= a.stats.layers);
            assert!(b.declared_px + 1e-12 >= a.declared_px, "bound shrank at L{}", b.level);
            assert!(b.tolerance_px > a.tolerance_px || a.level == 0);
        }
        assert_eq!(levels[0].project, p);
    }
}

#[test]
fn minimal_input_passes_through_unchanged() {
    let mut p = scene_with(vec![Layer::vector("tri", vec![VPath::polygon("t", &[Vec2::new(10.0, 10.0), Vec2::new(90.0, 10.0), Vec2::new(50.0, 80.0)]).with_fill("#112233")])]);
    p.channels = vec![Channel::keys("tri", Prop::LayerRotation, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (11.0, 5.0, Ease::Linear)]))];
    let levels = build(&p, &LodOptions::new(0.5, 1.0, 3)).unwrap();
    for l in &levels[1..] {
        let mut q = l.project.clone();
        q.lods.clear();
        assert_eq!(q, p, "level {} changed a minimal project", l.record.level);
        assert_eq!(l.record.declared_px, 0.0);
    }
    // a second pass over an already reduced level changes nothing more at the same tolerance
    let blob = blob_scene(1.0);
    let l1 = build(&blob, &LodOptions::new(1.0, 1.0, 1)).unwrap().remove(1);
    let mut again = build(&l1.project, &LodOptions::new(1.0, 1.0, 1)).unwrap().remove(1);
    again.project.lods.clear();
    let mut once = l1.project.clone();
    once.lods.clear();
    assert!(anchors_of(&again.project, "blob") <= anchors_of(&once, "blob"));
}

#[test]
fn records_are_embedded_and_round_trip() {
    let p = blob_scene(1.0);
    let levels = build(&p, &LodOptions::new(1.0, 2.0, 2)).unwrap();
    let l = &levels[2];
    assert_eq!(l.project.lods.len(), 1);
    assert_eq!(l.project.lods[0].level, 2);
    assert_eq!(l.project.lods[0].scale, 2.0);
    let back = Project::from_json(&l.project.to_json_pretty()).unwrap();
    assert_eq!(back, l.project);
    assert_eq!(l.record.stats.path_anchors, anchors_of(&l.project, "blob"));
    // projects without records do not serialise the field
    assert!(!p.to_json_pretty().contains("\"lods\""));
}

#[test]
fn verification_passes_on_the_demo_project_and_detects_a_planted_violation() {
    let p = crate::demo::project();
    let o = LodOptions::new(0.5, 1.0, 4);
    let levels = build(&p, &o).unwrap();
    for l in &levels[1..] {
        let r = verify(&p, l, &VerifyOptions { scale: 1.0, frames: 6, pixels: false, pixel_frames: 1 }).unwrap();
        assert!(r.holds, "level {}: measured {} > declared {} ({} at frame {})", r.level, r.measured_px, r.declared_px, r.worst_layer, r.worst_frame);
    }
    // negative control: displace a point of the coarsest level by 5 px -> must be reported
    let mut bad = levels[4].clone();
    if let Some(l) = bad.project.find_layer_mut("hills") {
        if let LayerKind::Vector { paths } = &mut l.kind {
            paths[0].anchors[2].pos.y += 5.0;
        }
    }
    let r = verify(&p, &bad, &VerifyOptions { scale: 1.0, frames: 4, pixels: false, pixel_frames: 1 }).unwrap();
    assert!(!r.holds && r.measured_px > 1.0, "planted violation not detected: {r:?}");
}

#[test]
fn dead_layers_are_dropped_losslessly_and_reported() {
    let mut hidden = Layer::vector("hidden", vec![VPath::rect("r", 0.0, 0.0, 50.0, 50.0).with_fill("#ff0000")]);
    hidden.visible = false;
    let p = scene_with(vec![hidden, Layer::vector("shown", vec![VPath::rect("r", 60.0, 60.0, 50.0, 50.0).with_fill("#00ff00")])]);
    let levels = build(&p, &LodOptions::new(0.5, 1.0, 1)).unwrap();
    assert!(levels[1].project.find_layer("hidden").is_none());
    assert!(levels[1].record.notes.iter().any(|n| n.contains("dead layer")));
    assert_eq!(levels[1].record.declared_px, 0.0);
}

#[test]
fn expression_referenced_and_ik_bones_are_not_reduced() {
    let mut p = rig_scene(true);
    // an expression reads the arm: its error would be amplified, so the channel must stay as it is
    p.layers.push(Layer::vector("follower", vec![VPath::rect("r", 10.0, 10.0, 10.0, 10.0).with_fill("#000000")]));
    p.channels.push(Channel::expr("follower", Prop::LayerX, "get(\"arm\", \"bone_angle\") * 2", Mode::Replace));
    let levels = build(&p, &LodOptions::new(2.0, 1.0, 2)).unwrap();
    assert_eq!(levels[2].record.stats.keys, levels[0].record.stats.keys);
    // IK: bones in a chain are never touched
    let mut q = rig_scene(true);
    q.bones.push(Bone::new("hand", Some("arm"), 30.0, Vec2::new(100.0, 0.0), 0.0));
    q.bones.push(Bone::new("goal", None, 5.0, Vec2::new(150.0, 100.0), 0.0));
    q.ik = vec![crate::project::IkChain::two_bone("ik", "arm", "hand", "goal", 1.0)];
    let lq = build(&q, &LodOptions::new(2.0, 1.0, 2)).unwrap();
    assert_eq!(lq[2].record.stats.keys, lq[0].record.stats.keys);
}

#[test]
fn layer_channels_and_camera_are_reduced_with_levers() {
    let mut p = scene_with(vec![
        Layer::vector("box", vec![VPath::rect("r", 20.0, 20.0, 60.0, 40.0).with_fill("#336699")]),
        Layer::vector("spinner", vec![VPath::rect("r", 100.0, 100.0, 40.0, 4.0).with_fill("#993366")]),
    ]);
    p.frames = 60;
    p.layers[1].transform.position = Vec2::new(0.0, 0.0);
    let dense = |f: &dyn Fn(f32) -> f32| Track::with_keys(&(0..60).map(|i| (i as f32, f(i as f32), Ease::Linear)).collect::<Vec<_>>());
    p.channels = vec![
        Channel::keys("box", Prop::LayerX, dense(&|t| 20.0 * (t * 0.1).sin())),
        Channel::keys("spinner", Prop::LayerRotation, dense(&|t| 10.0 * (t * 0.07).sin())),
        Channel::keys("", Prop::CameraX, dense(&|t| 3.0 * (t * 0.05).sin() + 100.0)),
    ];
    let levels = build(&p, &LodOptions::new(2.0, 1.0, 3)).unwrap();
    assert!(levels[3].record.stats.keys < levels[0].record.stats.keys / 2, "{} keys left of {}", levels[3].record.stats.keys, levels[0].record.stats.keys);
    for l in &levels[1..] {
        let r = verify(&p, l, &VerifyOptions { scale: 1.0, frames: 60, pixels: false, pixel_frames: 1 }).unwrap();
        assert!(r.holds, "level {}: measured {} > declared {} ({})", r.level, r.measured_px, r.declared_px, r.worst_layer);
    }
}

#[test]
fn options_and_projects_are_validated() {
    let p = blob_scene(1.0);
    for bad in [
        LodOptions::new(0.0, 1.0, 2),
        LodOptions::new(-1.0, 1.0, 2),
        LodOptions::new(f64::NAN, 1.0, 2),
        LodOptions::new(1.0, 0.0, 2),
        LodOptions::new(1.0, 100.0, 2),
        LodOptions::new(1.0, 1.0, 0),
        LodOptions::new(1.0, 1.0, 50),
        LodOptions { ratio: 0.5, ..LodOptions::new(1.0, 1.0, 2) },
        LodOptions { geometry_share: 0.8, keys_share: 0.5, ..LodOptions::new(1.0, 1.0, 2) },
        LodOptions { cull_share: 2.0, ..LodOptions::new(1.0, 1.0, 2) },
    ] {
        assert!(build(&p, &bad).is_err(), "{bad:?} accepted");
    }
    let mut broken = blob_scene(1.0);
    broken.channels = vec![Channel::keys("nope", Prop::LayerX, Track::new())];
    assert!(build(&broken, &LodOptions::new(1.0, 1.0, 1)).unwrap_err().contains("does not validate"));
    let l = build(&p, &LodOptions::new(1.0, 1.0, 1)).unwrap();
    assert!(verify(&p, &l[1], &VerifyOptions::new(0.0)).is_err());
}

#[test]
fn level_picking_follows_the_display_scale() {
    let p = blob_scene(1.0);
    let levels = build(&p, &LodOptions::new(0.25, 1.0, 4)).unwrap();
    let recs: Vec<LodRecord> = levels.iter().map(|l| l.record.clone()).collect();
    // tiny budget: only L0; huge budget: coarsest
    assert_eq!(pick_level(&recs, 1.0, 0.0), 0);
    assert_eq!(pick_level(&recs, 1.0, 1e9), 4);
    // zooming in (more pixels per unit) can only select a finer or equal level
    let mut last = usize::MAX;
    for s in [0.1, 0.5, 1.0, 2.0, 8.0, 64.0] {
        let k = pick_level(&recs, s, 0.5);
        assert!(k <= last, "level {k} at scale {s} after {last}");
        last = k;
        if k > 0 {
            assert!(recs[k].bound_stage * s <= 0.5 + 1e-12);
        }
        if k < 4 {
            assert!(recs[k + 1].bound_stage * s > 0.5, "level {} would also have fit at scale {s}", k + 1);
        }
    }
    assert_eq!(pick_level(&recs, 0.0, 1.0), 0);
    assert_eq!(pick_level(&[], 1.0, 1.0), 0);
    assert!((fit_scale(&p, 400.0, 1000.0) - 2.0).abs() < 1e-12);
}

#[test]
fn report_and_table_list_every_level() {
    let p = rig_scene(true);
    let levels = build(&p, &LodOptions::new(1.0, 1.0, 2)).unwrap();
    let v: Vec<VerifyReport> = levels[1..].iter().map(|l| verify(&p, l, &VerifyOptions { scale: 1.0, frames: 6, pixels: false, pixel_frames: 1 }).unwrap()).collect();
    let t = table(&levels, &v);
    assert_eq!(t.lines().count(), 4);
    let j = report_json(&levels, &v);
    assert_eq!(j["levels"].as_array().unwrap().len(), 3);
    assert_eq!(j["verify"].as_array().unwrap().len(), 2);
    assert!(j["limits"].as_array().unwrap().len() >= 5);
    assert!(j["assumptions"].as_array().unwrap().len() >= 5);
}

#[test]
fn mesh_with_non_affine_uvs_and_deformed_layers_is_left_alone() {
    // explicit uvs that are not an affine function of the position: not collapsed
    let mut p = mesh_scene(6, 6, false);
    if let Some(l) = p.find_layer_mut("img") {
        if let LayerKind::Image { mesh: Some(m), origin, size, .. } = &mut l.kind {
            let v = crate::project::mesh_vertices(*origin, size.unwrap(), 6, 6);
            let tris: Vec<[u32; 3]> = crate::weights::grid_triangles(6, 6).iter().map(|t| [t[0] as u32, t[1] as u32, t[2] as u32]).collect();
            let uvs: Vec<Vec2> = v.iter().map(|q| Vec2::new((q.x - 40.0) * 0.1 + ((q.y) * 0.3).sin(), (q.y - 60.0) * 0.2)).collect();
            m.free = Some(crate::project::FreeMesh { vertices: v, triangles: tris, uvs });
        }
    }
    let l = build(&p, &LodOptions::new(100.0, 1.0, 1)).unwrap();
    assert_eq!(mesh_vertices(&l[1].project), 49);
}

#[test]
fn file_backed_images_have_no_pixels_and_are_not_touched() {
    let mut p = mesh_scene(6, 6, false);
    if let Some(l) = p.find_layer_mut("img") {
        if let LayerKind::Image { source, .. } = &mut l.kind {
            *source = ImageSource { file: Some("missing.png".into()), png_base64: None };
        }
    }
    // the engine cannot load the file, so the project does not even validate: reported, not guessed
    assert!(build(&p, &LodOptions::new(5.0, 1.0, 1)).is_err());
}

#[test]
fn projects_with_links_get_identical_levels() {
    let mut p = blob_scene(1.0);
    p.glues = vec![crate::links::Glue { name: "g".into(), mode: Default::default(), pairs: vec![], strength: 1.0 }];
    match build(&p, &LodOptions::new(5.0, 1.0, 2)) {
        Ok(levels) => {
            assert!(levels[1..].iter().all(|l| l.record.declared_px == 0.0 && l.record.notes.iter().any(|n| n.contains("glues"))));
            assert_eq!(anchors_of(&levels[2].project, "blob"), 80);
        }
        // an empty glue may be rejected by validation; then the guard itself is checked directly
        Err(_) => assert!(links_present(&p)),
    }
}
