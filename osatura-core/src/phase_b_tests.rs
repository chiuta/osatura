// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Phase B: N-chain IK, FK/IK mix, limits, constraints, springs, deformers, morphs, wobble, images.
use crate::anim::{Ease, Track};
use crate::engine::Engine;
use crate::math::Vec2;
use crate::path::VPath;
use crate::project::*;
use crate::raster::{render_pixmap, RasterOptions};

fn proj() -> Project {
    Project::new("t", 200.0, 200.0, 24.0, 48)
}
fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}
fn near(a: Vec2, b: Vec2, tol: f32) -> bool {
    a.distance(b) <= tol
}
fn chain3(tx: f32, ty: f32) -> Project {
    let mut p = proj();
    p.bones = vec![
        Bone::new("a", None, 50.0, Vec2::ZERO, 0.0),
        Bone::new("b", Some("a"), 50.0, v(50.0, 0.0), 0.0),
        Bone::new("c", Some("b"), 50.0, v(50.0, 0.0), 0.0),
        Bone::new("t", None, 5.0, v(tx, ty), 0.0),
    ];
    p.ik = vec![IkChain::chain("arm", &["a", "b", "c"], "t")];
    p
}
fn tip(e: &Engine, f: f32, bone: &str) -> Vec2 {
    let fr = e.evaluate(f);
    let i = e.bone_index(bone).unwrap();
    fr.world[i].apply(v(e.project.bones[i].length, 0.0))
}

#[test]
fn three_bone_fabrik_reaches_target() {
    let e = Engine::new(chain3(60.0, 70.0)).unwrap();
    assert!(
        near(tip(&e, 0.0, "c"), v(60.0, 70.0), 0.5),
        "{:?}",
        tip(&e, 0.0, "c")
    );
}

#[test]
fn ik_mix_zero_keeps_fk_pose() {
    let mut p = chain3(60.0, 70.0);
    p.ik[0].mix = 0.0;
    let e = Engine::new(p).unwrap();
    assert!(near(tip(&e, 0.0, "c"), v(150.0, 0.0), 1e-3));
}

#[test]
fn ik_mix_blends_between_fk_and_ik() {
    let mut p = chain3(60.0, 70.0);
    p.ik[0].mix = 0.5;
    let e = Engine::new(p).unwrap();
    let t = tip(&e, 0.0, "c");
    assert!(
        t.distance(v(150.0, 0.0)) > 5.0 && t.distance(v(60.0, 70.0)) > 5.0,
        "{t:?}"
    );
}

#[test]
fn unreachable_target_straightens_chain() {
    let e = Engine::new(chain3(0.0, 500.0)).unwrap();
    assert!(near(tip(&e, 0.0, "c"), v(0.0, 150.0), 0.5));
}

#[test]
fn angle_limits_clamp_channels() {
    let mut p = proj();
    p.bones = vec![Bone::new("a", None, 50.0, Vec2::ZERO, 0.0).with_angle_limits(-45.0, 45.0)];
    p.channels = vec![Channel::keys(
        "a",
        Prop::BoneAngle,
        Track::with_keys(&[(0.0, 170.0, Ease::Linear)]),
    )];
    let e = Engine::new(p).unwrap();
    let r = e.evaluate(0.0).world[0].rotation();
    assert!((r - 45f32.to_radians()).abs() < 1e-4, "{r}");
}

#[test]
fn aim_constraint_points_at_target() {
    let mut p = proj();
    p.bones = vec![
        Bone::new("h", None, 10.0, Vec2::ZERO, 0.0),
        Bone::new("t", None, 1.0, v(0.0, 100.0), 0.0),
    ];
    p.constraints = vec![Constraint {
        name: "aim".into(),
        bone: "h".into(),
        kind: ConstraintKind::Aim {
            target: "t".into(),
            offset: 0.0,
        },
        weight: 1.0,
    }];
    let e = Engine::new(p).unwrap();
    let r = e.evaluate(0.0).world[0].rotation();
    assert!((r - std::f32::consts::FRAC_PI_2).abs() < 1e-3, "{r}");
}

#[test]
fn constraint_weight_half_is_halfway() {
    let mut p = proj();
    p.bones = vec![
        Bone::new("h", None, 10.0, Vec2::ZERO, 0.0),
        Bone::new("t", None, 1.0, v(0.0, 100.0), 0.0),
    ];
    p.constraints = vec![Constraint {
        name: "aim".into(),
        bone: "h".into(),
        kind: ConstraintKind::Aim {
            target: "t".into(),
            offset: 0.0,
        },
        weight: 0.5,
    }];
    let e = Engine::new(p).unwrap();
    let r = e.evaluate(0.0).world[0].rotation();
    assert!((r - std::f32::consts::FRAC_PI_4).abs() < 1e-3, "{r}");
}

#[test]
fn copy_rotation_and_pin() {
    let mut p = proj();
    p.bones = vec![
        Bone::new("src", None, 10.0, v(100.0, 100.0), 30.0),
        Bone::new("dst", None, 10.0, Vec2::ZERO, 0.0),
        Bone::new("pinned", None, 10.0, Vec2::ZERO, 0.0),
    ];
    p.constraints = vec![
        Constraint {
            name: "cr".into(),
            bone: "dst".into(),
            kind: ConstraintKind::CopyRotation {
                source: "src".into(),
            },
            weight: 1.0,
        },
        Constraint {
            name: "pin".into(),
            bone: "pinned".into(),
            kind: ConstraintKind::Pin {
                target: "src".into(),
            },
            weight: 1.0,
        },
    ];
    let e = Engine::new(p).unwrap();
    let f = e.evaluate(0.0);
    assert!((f.world[1].rotation() - 30f32.to_radians()).abs() < 1e-3);
    assert!(near(f.world[2].apply(Vec2::ZERO), v(100.0, 100.0), 1e-3));
}

fn spring_project(damping: f32) -> Project {
    let mut p = proj();
    p.bones = vec![
        Bone::new("a", None, 50.0, Vec2::ZERO, 0.0),
        Bone::new("tail", Some("a"), 30.0, v(50.0, 0.0), 0.0),
    ];
    p.channels = vec![Channel::keys(
        "a",
        Prop::BoneAngle,
        Track::with_keys(&[
            (0.0, 0.0, Ease::Linear),
            (10.0, 0.0, Ease::Linear),
            (14.0, 60.0, Ease::Linear),
        ]),
    )];
    p.springs = vec![Spring {
        bone: "tail".into(),
        frequency: 3.0,
        damping,
        warmup: false,
    }];
    p
}

#[test]
fn spring_lags_overshoots_and_settles() {
    let e = Engine::new(spring_project(0.2)).unwrap();
    let rel = |f: f32| {
        let w = e.evaluate(f).world;
        (w[1].rotation() - w[0].rotation()).to_degrees()
    };
    assert!(rel(5.0).abs() < 1e-3, "at rest before the move");
    let lag = (11..=13).map(|f| rel(f as f32)).fold(f32::MAX, f32::min);
    assert!(lag < -1.0, "tail lags behind the parent's swing: {lag}");
    let swing: Vec<f32> = (14..48).map(|f| rel(f as f32)).collect();
    assert!(swing.iter().any(|x| *x > 1.0), "rebounds past rest");
    assert!(rel(47.0).abs() < rel(14.0).abs(), "decays");
}

#[test]
fn springs_are_random_access_deterministic() {
    let e = Engine::new(spring_project(0.3)).unwrap();
    let a = e.evaluate(20.5).world[1];
    let _ = e.evaluate(3.0);
    let b = e.evaluate(20.5).world[1];
    assert_eq!(a, b);
}

fn square_layer() -> Layer {
    Layer::vector("sq", vec![VPath::rect("r", 0.0, 0.0, 100.0, 100.0)])
}

#[test]
fn uniform_lattice_offset_translates() {
    let mut p = proj();
    let mut l = square_layer();
    l.deformers = vec![Deformer::Lattice {
        name: "lat".into(),
        cols: 2,
        rows: 2,
        rect: [0.0, 0.0, 100.0, 100.0],
        offsets: vec![v(10.0, -5.0); 9],
    }];
    p.layers = vec![l];
    let e = Engine::new(p).unwrap();
    let f = e.evaluate(0.0);
    let a = &f.nodes[0].paths[0].anchors;
    assert!(
        near(a[0].pos, v(10.0, -5.0), 1e-3) && near(a[2].pos, v(110.0, 95.0), 1e-3),
        "{:?}",
        a[0].pos
    );
}

#[test]
fn lattice_control_point_is_animatable() {
    let mut p = proj();
    let mut l = square_layer();
    l.deformers = vec![Deformer::Lattice {
        name: "lat".into(),
        cols: 1,
        rows: 1,
        rect: [0.0, 0.0, 100.0, 100.0],
        offsets: vec![],
    }];
    p.layers = vec![l];
    p.channels = vec![Channel::keys(
        "sq::lat::1,1",
        Prop::LatticeDx,
        Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 30.0, Ease::Linear)]),
    )];
    let e = Engine::new(p).unwrap();
    let corner = |f: f32| e.evaluate(f).nodes[0].paths[0].anchors[2].pos;
    assert!(near(corner(0.0), v(100.0, 100.0), 1e-3));
    assert!(near(corner(10.0), v(130.0, 100.0), 1e-3));
}

#[test]
fn spine_translation_moves_shape() {
    let mut p = proj();
    let mut l = Layer::vector("tail", vec![VPath::rect("r", 0.0, -5.0, 100.0, 10.0)]);
    l.deformers = vec![Deformer::Spine {
        name: "sp".into(),
        rest: vec![v(0.0, 0.0), v(50.0, 0.0), v(100.0, 0.0)],
        offsets: vec![v(0.0, 20.0); 3],
        radius: None,
    }];
    p.layers = vec![l];
    let e = Engine::new(p).unwrap();
    let a = &e.evaluate(0.0).nodes[0].paths[0].anchors;
    assert!(
        near(a[0].pos, v(0.0, 15.0), 0.6) && near(a[2].pos, v(100.0, 25.0), 0.6),
        "{:?} {:?}",
        a[0].pos,
        a[2].pos
    );
}

#[test]
fn spine_bend_curves_shape() {
    let mut p = proj();
    let mut l = Layer::vector("tail", vec![VPath::rect("r", 0.0, -5.0, 100.0, 10.0)]);
    l.deformers = vec![Deformer::Spine {
        name: "sp".into(),
        rest: vec![v(0.0, 0.0), v(50.0, 0.0), v(100.0, 0.0)],
        offsets: vec![v(0.0, 0.0), v(0.0, -20.0), v(0.0, 0.0)],
        radius: None,
    }];
    p.layers = vec![l];
    let e = Engine::new(p).unwrap();
    let a = &e.evaluate(0.0).nodes[0].paths[0].anchors;
    // anchors above the middle of the strip are lifted; the ends stay put
    assert!(a[0].pos.y > -6.0 && a[0].pos.y < -4.0, "{:?}", a[0].pos);
}

#[test]
fn morph_blends_shapes() {
    let mut p = proj();
    let base = VPath::rect("p", 0.0, 0.0, 10.0, 10.0).with_morph("wide", |a| {
        let mut b = a.clone();
        b.pos.x *= 3.0;
        b
    });
    p.layers = vec![Layer::vector("s", vec![base])];
    p.channels = vec![Channel::keys(
        "s::p::wide",
        Prop::PathMorph,
        Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 1.0, Ease::Linear)]),
    )];
    let e = Engine::new(p).unwrap();
    let x = |f: f32| e.evaluate(f).nodes[0].paths[0].anchors[1].pos.x;
    assert!(
        (x(0.0) - 10.0).abs() < 1e-3
            && (x(10.0) - 30.0).abs() < 1e-3
            && (x(5.0) - 20.0).abs() < 1e-3
    );
}

#[test]
fn point_offsets_are_animatable() {
    let mut p = proj();
    p.layers = vec![Layer::vector(
        "s",
        vec![VPath::rect("p", 0.0, 0.0, 10.0, 10.0)],
    )];
    p.channels = vec![Channel::keys(
        "s::p::0",
        Prop::PointDy,
        Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 7.0, Ease::Linear)]),
    )];
    let e = Engine::new(p).unwrap();
    assert!((e.evaluate(10.0).nodes[0].paths[0].anchors[0].pos.y - 7.0).abs() < 1e-3);
}

#[test]
fn wobble_is_deterministic_bounded_and_boils() {
    let mut p = proj();
    let mut l = square_layer();
    l.wobble = Some(Wobble {
        amp: 3.0,
        scale: 0.05,
        rate: 8.0,
        seed: 4,
    });
    p.layers = vec![l];
    let e = Engine::new(p).unwrap();
    let pos = |f: f32| e.evaluate(f).nodes[0].paths[0].anchors[0].pos;
    assert_eq!(pos(5.0), pos(5.0));
    assert!(pos(0.0).distance(v(0.0, 0.0)) <= 3.0 * 1.5);
    assert!(pos(0.0) != pos(6.0), "different boil ticks differ");
    assert_eq!(pos(0.0), pos(0.5), "same boil tick holds");
}

fn png(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut pm = tiny_skia::Pixmap::new(w, h).unwrap();
    pm.fill(tiny_skia::Color::from_rgba8(
        rgba[0], rgba[1], rgba[2], rgba[3],
    ));
    pm.encode_png().unwrap()
}

#[test]
fn image_layer_renders_raster_and_svg() {
    let mut p = proj();
    p.layers = vec![Layer::image_from_png(
        "img",
        &png(20, 20, [255, 0, 0, 255]),
        v(50.0, 50.0),
    )];
    let e = Engine::new(p).unwrap();
    let pm = render_pixmap(
        &e,
        0.0,
        &RasterOptions {
            scale: 1.0,
            show_bones: false,
            transparent: false,
        },
    );
    let px = pm.pixel(60, 60).unwrap();
    assert!(px.red() > 200 && px.alpha() == 255);
    assert_eq!(pm.pixel(10, 10).unwrap().alpha(), 0);
    let svg = crate::svg::render_frame(&e, 0.0, &crate::svg::RenderOptions { show_bones: false });
    assert!(svg.contains("<image") && svg.contains("data:image/png;base64"));
}

#[test]
fn image_mesh_follows_bone_weights() {
    let mut p = proj();
    p.bones = vec![Bone::new("b", None, 10.0, v(0.0, 0.0), 0.0)];
    p.channels = vec![Channel::keys(
        "b",
        Prop::BoneOffsetX,
        Track::with_keys(&[(0.0, 0.0, Ease::Linear), (1.0, 100.0, Ease::Linear)]),
    )];
    let mut l = Layer::image_from_png("img", &png(20, 20, [0, 0, 255, 255]), v(10.0, 10.0));
    if let LayerKind::Image { size, mesh, .. } = &mut l.kind {
        *size = Some(v(20.0, 20.0));
        *mesh = Some(MeshSpec {
            cols: 2,
            rows: 2,
            weights: vec![],
            ..Default::default()
        });
    }
    p.layers = vec![l];
    p.auto_skin("img", 1).unwrap();
    let e = Engine::new(p).unwrap();
    let pm = render_pixmap(
        &e,
        1.0,
        &RasterOptions {
            scale: 1.0,
            show_bones: false,
            transparent: false,
        },
    );
    assert_eq!(
        pm.pixel(120, 20).unwrap().alpha(),
        255,
        "mesh image moved with its bone"
    );
    assert_eq!(
        pm.pixel(15, 15).unwrap().alpha(),
        0,
        "and left its old place"
    );
}

#[test]
fn validation_reports_new_errors() {
    let mut p = chain3(0.0, 0.0);
    p.ik.push(IkChain::chain("bad", &["a"], "t"));
    p.springs = vec![Spring {
        bone: "nope".into(),
        frequency: 3.0,
        damping: 0.3,
        warmup: false,
    }];
    p.constraints = vec![Constraint {
        name: "c".into(),
        bone: "a".into(),
        kind: ConstraintKind::Aim {
            target: "a".into(),
            offset: 0.0,
        },
        weight: 1.0,
    }];
    let errs = match Engine::new(p) {
        Err(e) => e,
        Ok(_) => panic!("must fail"),
    };
    let all = errs.join("\n");
    assert!(
        all.contains("at least two bones")
            && all.contains("unknown bone 'nope'")
            && all.contains("cannot constrain itself"),
        "{all}"
    );
}

/// C11: with the default `iterations` (16) the tip reaches random reachable targets to < 0.05 px
/// (a fixed 16 sweeps used to stop up to ~2.5 px short), for 4- and 6-bone chains.
#[test]
fn fabrik_reaches_random_targets_within_a_twentieth_of_a_pixel() {
    let mut seed = 12345u64;
    let mut rnd = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 33) as f32) / (1u64 << 31) as f32
    };
    for nb in [4usize, 6] {
        let names: Vec<String> = (0..nb).map(|i| format!("b{i}")).collect();
        let mut worst = 0.0f32;
        for case in 0..60 {
            let lens: Vec<f32> = (0..nb).map(|_| 20.0 + 25.0 * rnd()).collect();
            let reach: f32 = lens.iter().sum();
            let (rr, ang) = (reach * (0.2 + 0.78 * rnd()), std::f32::consts::TAU * rnd());
            let (tx, ty) = (rr * ang.cos(), rr * ang.sin());
            let mut p = proj();
            p.bones = names
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    Bone::new(
                        n,
                        if i == 0 {
                            None
                        } else {
                            Some(names[i - 1].as_str())
                        },
                        lens[i],
                        if i == 0 {
                            Vec2::ZERO
                        } else {
                            v(lens[i - 1], 0.0)
                        },
                        0.0,
                    )
                })
                .collect();
            p.bones.push(Bone::new("t", None, 5.0, v(tx, ty), 0.0));
            let refs: Vec<&str> = names.iter().map(String::as_str).collect();
            p.ik = vec![IkChain::chain("arm", &refs, "t")];
            assert_eq!(p.ik[0].iterations, 16, "the default");
            let e = Engine::new(p).unwrap();
            let err = tip(&e, 0.0, &names[nb - 1]).distance(v(tx, ty));
            worst = worst.max(err);
            assert!(
                err < 0.05,
                "{nb} bones, case {case}: tip {err} px from the target"
            );
        }
        assert!(worst < 0.05);
    }
}
