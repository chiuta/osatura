// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `api`.
#![allow(unused_imports)]

use super::*;

#[test]
fn load_errors_are_typed() {
    assert!(matches!(Animation::from_json(""), Err(ApiError::InvalidJson(_))));
    assert!(matches!(Animation::from_json("[1,2]"), Err(ApiError::InvalidJson(_))));
    assert!(matches!(Animation::from_json("{\"format\":9999,\"name\":\"x\"}"), Err(ApiError::InvalidJson(_))));
    let mut p = crate::demo::project();
    p.fps = -1.0;
    let bad = p.to_json_pretty();
    match Animation::from_json(&bad) {
        Err(ApiError::InvalidProject(v)) => assert!(!v.is_empty()),
        Err(e) => panic!("wrong error {e}"),
        Ok(_) => panic!("accepted a bad fps"),
    }
    assert!(Animation::validate_json(&Animation::demo().to_json()).is_ok());
}

#[test]
fn queries_match_the_project() {
    let a = Animation::demo();
    let i = a.info();
    assert_eq!(i.frames, a.frame_count());
    assert_eq!(i.fps, a.fps());
    assert!(!a.bone_names().is_empty() && !a.layer_names().is_empty());
    let w = a.bone_world(1e9).unwrap(); // clamped to the last frame
    assert_eq!(w.len(), a.bone_names().len());
    assert!(a.bone_world(f32::INFINITY).is_err());
}

#[test]
fn render_validates_and_works() {
    let a = Animation::demo();
    let s = RenderSettings { scale: 0.5, ..RenderSettings::default() };
    let img = a.render_rgba(0.0, &s).unwrap();
    assert_eq!(img.pixels.len(), img.width as usize * img.height as usize * 4);
    for bad in [0.0, -1.0, 5.0, f32::NAN] {
        let s = RenderSettings { scale: bad, ..RenderSettings::default() };
        assert!(matches!(a.render_rgba(0.0, &s), Err(ApiError::InvalidArgument(_))), "scale {bad}");
        assert!(a.render_png(0.0, &s).is_err());
    }
    assert!(a.render_svg(f32::NAN, false).is_err());
    assert!(a.render_svg(0.0, true).unwrap().contains("<svg"));
    assert_eq!(&a.render_png(2.0, &s_default()).unwrap()[1..4], b"PNG");
}

fn s_default() -> RenderSettings {
    RenderSettings { scale: 0.25, ..RenderSettings::default() }
}

#[test]
fn edits_validate_undo_and_lock() {
    let mut a = Animation::demo();
    let bone = a.bone_names()[0].clone();
    let before = a.to_json();
    a.set_key(&bone, "bone_angle", 3.0, 12.0, Interpolation::Smooth).unwrap();
    assert_ne!(a.to_json(), before);
    assert!(a.delete_key(&bone, "bone_angle", 3.0).unwrap());
    assert!(!a.delete_key(&bone, "bone_angle", 77.0).unwrap());
    assert!(a.undo() && a.undo());
    assert_eq!(a.to_json(), before);
    assert!(!a.undo());
    assert!(a.redo());
    assert!(matches!(a.set_key(&bone, "no_such_prop", 0.0, 1.0, Interpolation::Linear), Err(ApiError::InvalidArgument(_))));
    assert!(a.set_key(&bone, "bone_angle", f32::NAN, 1.0, Interpolation::Linear).is_err());
    assert!(a.set_key(&bone, "bone_angle", 1.0, f32::INFINITY, Interpolation::Linear).is_err());
    // unknown target is rejected by validation, project unchanged
    let j = a.to_json();
    assert!(a.set_key("no_such_bone", "bone_angle", 1.0, 1.0, Interpolation::Linear).is_err());
    assert_eq!(a.to_json(), j);
}

#[test]
fn exports_produce_bytes() {
    let a = Animation::demo();
    assert!(a.export(ExportFormat::Gif, 0.25).unwrap().starts_with(b"GIF8"));
    assert!(a.export(ExportFormat::Glb, 1.0).unwrap().starts_with(b"glTF"));
    assert!(a.export(ExportFormat::Lottie, 1.0).unwrap().starts_with(b"{"));
    assert!(a.export(ExportFormat::Html, 1.0).unwrap().len() > 100);
    assert!(a.export(ExportFormat::Gif, 99.0).is_err());
}

/// Demo project plus two clips and a machine "m" (state a -> b when `go`).
fn machine_project() -> Project {
    let mut p = crate::demo::project();
    let bone = p.bones[0].name.clone();
    let mk = |n: &str, v: f32| {
        let tr = crate::motionlib::TakeTrack { target: bone.clone(), prop: Prop::BoneAngle, keys: vec![crate::anim::Key { frame: 0.0, value: v, ease: Ease::Linear }, crate::anim::Key { frame: 10.0, value: v, ease: Ease::Linear }] };
        crate::clipstack::Clip::new(n, crate::motionlib::Take { name: n.into(), length: 10.0, tracks: vec![tr] })
    };
    p.clips = vec![mk("a", 0.0), mk("b", 30.0)];
    let st = |n: &str| crate::machine::State { name: n.into(), source: crate::machine::StateSource::Clip { clip: n.into() }, mode: Default::default(), speed: 1.0, blend_in: 0.0 };
    p.machines.push(crate::machine::Machine {
        name: "m".into(),
        inputs: vec![crate::machine::Input { name: "go".into(), kind: crate::machine::InputKind::Bool, default: 0.0, hold: false }],
        layers: vec![crate::machine::Layer {
            name: "l".into(),
            mask: vec![],
            entry: "a".into(),
            states: vec![st("a"), st("b")],
            transitions: vec![crate::machine::Transition {
                from: "a".into(),
                to: "b".into(),
                conditions: vec![crate::machine::Condition { input: "go".into(), op: Default::default(), value: 1.0 }],
                exit_time: None,
                duration: Some(0.0),
                ease: Ease::Linear,
            }],
        }],
    });
    p
}

#[test]
fn player_runs_a_machine() {
    let p = machine_project();
    let a = Animation::from_json(&p.to_json_pretty()).unwrap();
    assert_eq!(a.machine_names(), ["m"]);
    assert!(a.player("nope").is_err());
    let mut pl = a.player("m").unwrap();
    let before = pl.bone_world().unwrap()[0];
    pl.advance(0.1).unwrap();
    assert_eq!(pl.state("l").as_deref(), Some("a"));
    pl.set_bool("go", true).unwrap();
    pl.advance(0.1).unwrap();
    assert_eq!(pl.state("l").as_deref(), Some("b"));
    assert_ne!(pl.bone_world().unwrap()[0], before);
    assert!(pl.take_events_json().contains("\"state\":\"b\""));
    assert!(pl.render_svg(false).unwrap().starts_with("<svg"));
    assert_eq!(pl.render_rgba(&RenderSettings::new(0.25)).unwrap().pixels.len() % 4, 0);
    assert!(pl.set_bool("zzz", true).is_err() && pl.set_number("go", 1.0).is_err() && pl.fire("go").is_err());
    assert!(pl.advance(f64::NAN).is_err() && pl.set_value("bad path", 1.0).is_err());
    assert!(pl.render_rgba(&RenderSettings::new(9.0)).is_err());
}

/// machine project + a column of ten 100x20 rows inside a 100x50 scroll region.
fn scroll_player_project() -> Project {
    use crate::containers::{Container, Dir, Member, RectRule, Scroll};
    use crate::path::VPath;
    use crate::project::Layer;
    let mut p = machine_project();
    for i in 0..10 {
        p.layers.push(Layer::vector(&format!("row{i}"), vec![VPath::rect("r", 0.0, 0.0, 100.0, 20.0).with_fill("#336699")]));
    }
    let mut c = Container::new("col", Dir::Column, (0..10).map(|i| Member::layer(&format!("row{i}"))).collect());
    c.rect = RectRule::default();
    p.containers.push(c);
    let mut s = Scroll::new("list", "col");
    s.viewport = RectRule::Fixed { x: 0.0, y: 0.0, w: 100.0, h: 50.0 };
    s.snap_y = vec![0.0, 40.0, 80.0, 120.0, 150.0];
    s.input = Some("ui.scroll".into());
    p.scrolls.push(s);
    p
}

fn row_ys(pl: &Player) -> Vec<f32> {
    let e = pl.build_engine().unwrap();
    (0..10).map(|i| e.layer_local_at(pl.rt.base_frame(), e.node_index(&format!("row{i}")).unwrap()).position.y).collect()
}

#[test]
fn player_scrolls_drags_and_flings() {
    let a = Animation::from_json(&scroll_player_project().to_json_pretty()).unwrap();
    let mut pl = a.player("m").unwrap();
    assert_eq!(pl.scroll_ids(), ["list"]);
    assert!((pl.scroll_max("list").unwrap()[1] - 150.0).abs() < 1e-3);
    assert_eq!(pl.scroll_offset("list"), Some([0.0, 0.0]));
    assert!(pl.scroll_offset("nope").is_none() && pl.scroll_drag("nope", 0.0, 0.0).is_err());
    assert!(pl.scroll_drag("list", f32::NAN, 0.0).is_err() && pl.scroll_release("list", 0.0, f32::INFINITY).is_err());
    let y0 = row_ys(&pl);
    pl.scroll_drag("list", 0.0, 70.0).unwrap();
    let y1 = row_ys(&pl);
    assert!((y0[0] - y1[0] - 70.0).abs() < 1e-3 && (y0[3] - y1[3] - 70.0).abs() < 1e-3, "{y0:?} {y1:?}");
    // past the end the offset clamps
    pl.scroll_drag("list", 0.0, 900.0).unwrap();
    assert!((pl.scroll_offset("list").unwrap()[1] - 150.0).abs() < 1e-3);
    // release with a fling lands on a snap point once advanced
    pl.scroll_drag("list", 0.0, 30.0).unwrap();
    pl.scroll_release("list", 0.0, 100.0).unwrap();
    assert_eq!(pl.scroll_settled("list"), Some(false));
    for _ in 0..300 {
        pl.advance(1.0 / 60.0).unwrap();
    }
    assert_eq!(pl.scroll_settled("list"), Some(true));
    assert_eq!(pl.scroll_offset("list"), Some([0.0, 40.0]));
    // the linked input path drives the offset
    pl.set_value("ui.scroll", 100.0).unwrap();
    assert_eq!(pl.scroll_offset("list"), Some([0.0, 100.0]));
    assert!(pl.set_value("ui.scroll", f64::NAN).is_err());
    // the machine keeps working alongside
    pl.set_bool("go", true).unwrap();
    pl.advance(0.1).unwrap();
    assert_eq!(pl.state("l").as_deref(), Some("b"));
    // determinism: same calls, same offsets
    let run = || {
        let mut q = a.player("m").unwrap();
        q.scroll_drag("list", 0.0, 55.0).unwrap();
        q.scroll_release("list", 0.0, -400.0).unwrap();
        for _ in 0..90 {
            q.advance(1.0 / 60.0).unwrap();
        }
        q.scroll_offset("list")
    };
    assert_eq!(run(), run());
}

#[test]
fn player_canvas_size_relayouts_and_validates() {
    let mut p = scroll_player_project();
    // the viewport follows the canvas height (top 0, bottom 0), 100 wide
    p.scrolls[0].viewport = crate::containers::RectRule::Pin { left: Some(0.0), top: Some(0.0), right: None, bottom: Some(0.0), width: Some(100.0), height: None };
    let a = Animation::from_json(&p.to_json_pretty()).unwrap();
    let mut pl = a.player("m").unwrap();
    let (w0, h0) = pl.canvas_size();
    assert_eq!((w0, h0), (a.info().width, a.info().height));
    for bad in [(0.0, 10.0), (-1.0, 10.0), (f32::NAN, 10.0), (10.0, f32::INFINITY), (1e9, 10.0)] {
        assert!(pl.set_canvas_size(bad.0, bad.1).is_err(), "{bad:?}");
    }
    assert_eq!(pl.canvas_size(), (w0, h0));
    pl.set_canvas_size(w0, 120.0).unwrap();
    assert!((pl.scroll_max("list").unwrap()[1] - 80.0).abs() < 1e-3, "{:?}", pl.scroll_max("list"));
    // the position is re-clamped when the limit shrinks, and renders get the new size
    pl.scroll_drag("list", 0.0, 80.0).unwrap();
    pl.set_canvas_size(w0 / 2.0, 150.0).unwrap();
    assert!((pl.scroll_max("list").unwrap()[1] - 50.0).abs() < 1e-3);
    assert!((pl.scroll_offset("list").unwrap()[1] - 50.0).abs() < 1e-3);
    let img = pl.render_rgba(&RenderSettings::new(1.0)).unwrap();
    assert_eq!((img.width, img.height), ((w0 / 2.0).round() as u32, 150));
    assert!(pl.render_svg(false).unwrap().starts_with("<svg"));
    assert!(pl.render_rgba(&RenderSettings::new(4.0)).is_ok());
    // back to the original size
    pl.set_canvas_size(w0, h0).unwrap();
    assert_eq!(pl.canvas_size(), (w0, h0));
}

#[test]
fn player_picks_a_level_of_detail_within_the_requested_bound() {
    use crate::math::Vec2;
    use crate::path::VPath;
    use crate::project::Layer;
    let mut p = machine_project();
    let pts: Vec<Vec2> = (0..96)
        .map(|i| {
            let t = i as f32 / 96.0 * std::f32::consts::TAU;
            let r = 40.0 + 0.3 * (t * 5.0).sin();
            Vec2::new(100.0 + r * t.cos(), 100.0 + r * t.sin())
        })
        .collect();
    p.layers.push(Layer::vector("blob", vec![VPath::smooth_through("c", &pts, true, 1.0).with_fill("#336699")]));
    let a = Animation::from_json(&p.to_json_pretty()).unwrap();
    let mut pl = a.player("m").unwrap();
    assert_eq!(pl.lod_level(), 0);
    assert_eq!(pl.lod_bound_px(), 0.0);
    let full = pl.render_rgba(&RenderSettings::new(1.0)).unwrap();
    assert!(pl.set_screen_size(0.0, 10.0, 1.0).is_err() && pl.set_screen_size(10.0, 10.0, -1.0).is_err() && pl.set_screen_size(f32::NAN, 10.0, 1.0).is_err());
    // exact request: nothing may be reduced
    let (w, h) = (a.info().width, a.info().height);
    pl.set_screen_size(w, h, 0.0).unwrap();
    assert_eq!(pl.lod_level(), 0);
    // a generous request picks a coarser level whose declared bound stays within it
    for max_px in [0.25f32, 1.0, 4.0] {
        pl.set_screen_size(w, h, max_px).unwrap();
        assert!(pl.lod_bound_px() <= max_px + 1e-6, "bound {} > {max_px}", pl.lod_bound_px());
        let img = pl.render_rgba(&RenderSettings::new(1.0)).unwrap();
        assert_eq!((img.width, img.height), (full.width, full.height));
    }
    pl.set_screen_size(w, h, 4.0).unwrap();
    assert!(pl.lod_level() > 0, "a 4 px budget must allow a reduced level");
    let coarse = pl.lod_level();
    // a smaller screen tolerates at least as coarse a level
    pl.set_screen_size(w / 4.0, h / 4.0, 4.0).unwrap();
    assert!(pl.lod_level() >= coarse);
    // levels are cached: the second call does not rebuild
    let ptr = pl.lod_levels.as_ref().unwrap().as_ptr();
    pl.set_screen_size(w, h, 1.0).unwrap();
    assert_eq!(ptr, pl.lod_levels.as_ref().unwrap().as_ptr());
    // the machine still drives the pose on a reduced level
    pl.set_screen_size(w, h, 4.0).unwrap();
    let before = pl.bone_world().unwrap()[0];
    pl.set_bool("go", true).unwrap();
    pl.advance(0.1).unwrap();
    assert_ne!(pl.bone_world().unwrap()[0], before);
    pl.clear_screen_size();
    assert_eq!(pl.lod_level(), 0);
}

#[test]
fn lod_levels_are_valid_projects_with_a_declared_bound() {
    let a = Animation::demo();
    let lv = a.lod_levels(1.0, 1.0, 3).unwrap();
    assert_eq!(lv.len(), 4);
    assert_eq!(lv[0].level, 0);
    for (i, l) in lv.iter().enumerate() {
        assert_eq!(l.level, i);
        assert!(Animation::from_json(&l.json).is_ok());
        if i > 0 {
            assert!(l.declared_px <= 1.0 * 2f64.powi(i as i32 - 1) + 1e-9);
            assert!((l.bound_at(2.0) - l.bound_stage * 2.0).abs() < 1e-12);
            assert!(l.layers <= lv[0].layers && l.keys <= lv[0].keys);
        }
    }
    for bad in [(0.0, 1.0, 2usize), (-1.0, 1.0, 2), (1.0, 0.0, 2), (1.0, 1.0, 0), (1.0, 1.0, 99), (f64::NAN, 1.0, 2)] {
        assert!(matches!(a.lod_levels(bad.0, bad.1, bad.2), Err(ApiError::InvalidArgument(_))), "{bad:?}");
    }
    assert_eq!(a.lod_levels(1.0, 1.0, 3).unwrap(), lv, "deterministic");
}

#[test]
fn fit_to_size_meets_a_generous_budget_and_reports_an_impossible_one() {
    use crate::path::VPath;
    use crate::project::Layer;
    // a small animation keeps this fast in debug builds
    let mut p = Project::new("tiny", 32.0, 32.0, 12.0, 6);
    for i in 0..4 {
        p.layers.push(Layer::vector(&format!("l{i}"), vec![VPath::rect("r", 2.0 + i as f32 * 6.0, 4.0, 8.0, 8.0).with_fill("#336699")]));
    }
    let a = Animation::from_json(&p.to_json_pretty()).unwrap();
    let r = a.fit_to_size(FitFormat::Lottie, 10_000_000).unwrap();
    assert!(r.met && r.size == r.bytes.len() && r.size <= 10_000_000 && r.ssim_min > 0.5, "{r:?}");
    let tiny = a.fit_to_size(FitFormat::Project, 1000).unwrap();
    assert!(tiny.size > 0 && (tiny.met == (tiny.size <= 1000)));
    assert!(a.fit_to_size(FitFormat::Html, 10).is_err() && a.fit_to_size(FitFormat::Gif, usize::MAX).is_err());
}

fn mini_bvh() -> String {
    let mut s = String::from("HIERARCHY\nROOT Hips\n{\n OFFSET 0 100 0\n CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation\n");
    s.push_str(" JOINT Spine\n {\n  OFFSET 0 12 0\n  CHANNELS 3 Zrotation Xrotation Yrotation\n  JOINT Head\n  {\n   OFFSET 0 12 0\n   CHANNELS 3 Zrotation Xrotation Yrotation\n   End Site\n   {\n    OFFSET 0 8 0\n   }\n  }\n }\n}\nMOTION\nFrames: 4\nFrame Time: 0.0333333\n");
    for f in 0..4 {
        s.push_str(&format!("0 100 0 0 0 0 {} 0 0 0 0 0\n", f * 10));
    }
    s
}

#[test]
fn retarget_bvh_writes_channels_in_one_undoable_edit() {
    use crate::math::Vec2;
    use crate::project::Bone;
    let mut p = Project::new("rig", 400.0, 400.0, 30.0, 10);
    p.bones.push(Bone::new("hips", None, 20.0, Vec2::new(200.0, 250.0), -90.0));
    p.bones.push(Bone::new("spine", Some("hips"), 40.0, Vec2::new(20.0, 0.0), 0.0));
    p.bones.push(Bone::new("head", Some("spine"), 30.0, Vec2::new(40.0, 0.0), 0.0));
    let mut a = Animation::from_json(&p.to_json_pretty()).unwrap();
    let before = a.to_json();
    let r = a.retarget_bvh(&mini_bvh(), &MocapSettings::new().with_smooth(1)).unwrap();
    assert!(r.frames >= 3, "{r:?}");
    assert!(r.matched.iter().any(|(j, b)| j == "Spine" && b == "spine"), "{r:?}");
    assert_ne!(a.to_json(), before);
    assert!(a.undo());
    assert_eq!(a.to_json(), before);
    assert!(a.retarget_bvh("not a bvh", &MocapSettings::default()).is_err());
    assert_eq!(a.to_json(), before);
    // a rig without matching bones is an error, not a panic
    let mut d = Animation::demo();
    let _ = d.retarget_bvh(&mini_bvh(), &MocapSettings::new().with_in_place(true).with_mirror(true));
}

#[test]
fn tracing_and_fuzzing_validate_input() {
    let png = Animation::demo().render_png(0.0, &RenderSettings::new(0.25)).unwrap();
    let a = Animation::from_image(&png, 6, "traced").unwrap();
    assert!(!a.layer_names().is_empty());
    assert!(a.render_rgba(0.0, &RenderSettings::new(0.5)).is_ok());
    for bad in [0usize, 1, 65] {
        assert!(matches!(Animation::from_image(&png, bad, "x"), Err(ApiError::InvalidArgument(_))));
        assert!(trace_image_svg(&png, bad).is_err());
    }
    assert!(Animation::from_image(b"", 4, "x").is_err() && Animation::from_image(&png[..20], 4, "x").is_err());
    let f = Animation::demo().fuzz(30, 7).unwrap();
    assert_eq!(f.samples, 30);
    assert!(f.failure_rate >= 0.0 && f.failure_rate <= 1.0 && f.failures <= 30);
    let g = Animation::demo().fuzz(30, 7).unwrap();
    assert_eq!((g.failures, g.max_severity), (f.failures, f.max_severity), "same seed, same result");
    assert!(Animation::demo().fuzz(100_001, 1).is_err());
}

fn scratch(tag: &str) -> crate::fsutil::ScratchDir {
    crate::fsutil::ScratchDir::new(&format!("osatura-api-{tag}"))
}

#[test]
fn vcs_log_reads_history_and_ci_run_reports() {
    let d = scratch("vcs");
    let f = d.join("p.json");
    std::fs::write(&f, Animation::demo().to_json()).unwrap();
    let mut repo = crate::vcs::Repo::init(&f, Some("ana")).unwrap();
    repo.fixed_time = Some(1_000);
    assert!(vcs_log(f.to_str().unwrap(), 10).unwrap().is_empty(), "no commits yet");
    repo.commit(Some("first"), None).unwrap();
    let mut a = Animation::from_json(&std::fs::read_to_string(&f).unwrap()).unwrap();
    let bone = a.bone_names()[0].clone();
    a.set_key(&bone, "bone_angle", 5.0, 33.0, Interpolation::Linear).unwrap();
    std::fs::write(&f, a.to_json()).unwrap();
    repo.fixed_time = Some(2_000);
    repo.commit(Some("second"), None).unwrap();
    let log = vcs_log(d.to_str().unwrap(), 10).unwrap();
    assert_eq!(log.iter().map(|c| c.message.as_str()).collect::<Vec<_>>(), ["second", "first"]);
    assert_eq!(log[0].author, "ana");
    assert_eq!(vcs_log(f.to_str().unwrap(), 1).unwrap().len(), 1);
    assert!(vcs_log(f.to_str().unwrap(), 0).is_err());
    assert!(vcs_log(d.join("nowhere").to_str().unwrap(), 5).is_err());
    // CI: a manifest with one structural check
    let m = d.join("ci.json");
    std::fs::write(&m, r#"{"schema":"osatura-ci/1","project":"p.json","goldens":"goldens","checks":[{"kind":"validate","name":"v"}]}"#).unwrap();
    match ci_run(m.to_str().unwrap()) {
        Ok(o) => {
            assert_eq!((o.total, o.passed, o.failed, o.errors, o.exit_code), (1, 1, 0, 0, 0), "{}", o.json);
            assert!(o.json.contains("\"v\""));
        }
        Err(e) => panic!("{e}"),
    }
    assert!(ci_run(d.join("missing.json").to_str().unwrap()).is_err());
    std::fs::write(&m, "{ nope").unwrap();
    assert!(ci_run(m.to_str().unwrap()).is_err());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn vcs_write_roundtrip_with_conflicts_and_typed_errors() {
    let d = scratch("vcswrite");
    let f = d.join("p.json");
    std::fs::write(&f, Animation::demo().to_json()).unwrap();
    let p = f.to_str().unwrap();
    let edit = |angle: f32, frame: f32| {
        let mut a = Animation::from_json(&std::fs::read_to_string(&f).unwrap()).unwrap();
        let bone = a.bone_names()[0].clone();
        a.set_key(&bone, "bone_angle", frame, angle, Interpolation::Linear).unwrap();
        std::fs::write(&f, a.to_json()).unwrap();
    };
    assert!(vcs_commit(p, Some("x"), None).is_err(), "no repository yet");
    vcs_init(p, Some("ana")).unwrap();
    assert!(vcs_init(p, None).is_err());
    let base = vcs_commit(p, Some("base"), None).unwrap();
    assert!(vcs_commit(p, Some("same"), None).is_err());
    assert_eq!(vcs_tag(p, "v1", None).unwrap(), base);
    assert!(vcs_tag(p, "bad name", None).is_err());
    vcs_branch(p, "other", Some("v1")).unwrap();
    edit(20.0, 5.0);
    // dirty tree: typed error for checkout, merge, cherry-pick, revert
    assert!(matches!(vcs_checkout(p, "other", false), Err(ApiError::DirtyTree(_))));
    assert!(matches!(vcs_merge(p, "other", false, None, None), Err(ApiError::DirtyTree(_))));
    assert!(matches!(vcs_cherry_pick(p, &base, None), Err(ApiError::DirtyTree(_))));
    assert!(matches!(vcs_revert(p, &base, None), Err(ApiError::DirtyTree(_))));
    vcs_commit(p, Some("main edit"), None).unwrap();
    vcs_checkout(p, "other", false).unwrap();
    edit(40.0, 5.0);
    vcs_commit(p, Some("other edit"), None).unwrap();
    vcs_checkout(p, "main", false).unwrap();
    let conflicts = match vcs_merge(p, "other", false, None, None).unwrap() {
        VcsMergeOutcome::Conflicted(c) => c,
        o => panic!("{o:?}"),
    };
    assert!(!conflicts.is_empty() && conflicts.iter().all(|c| !c.path.is_empty() && !c.kind.is_empty()));
    assert!(conflicts.iter().any(|c| c.ours.is_some() && c.theirs.is_some()));
    assert_eq!(vcs_conflicts(p).unwrap(), conflicts);
    // while merging: typed errors
    assert!(matches!(vcs_commit(p, None, None), Err(ApiError::Conflicts(v)) if v.len() == conflicts.len()));
    assert!(matches!(vcs_checkout(p, "other", true), Err(ApiError::MergeInProgress(_))));
    assert!(matches!(vcs_merge(p, "other", false, None, None), Err(ApiError::MergeInProgress(_))));
    assert!(matches!(vcs_revert(p, &base, None), Err(ApiError::MergeInProgress(_))));
    assert!(vcs_resolve(p, "/nowhere", &VcsChoice::Ours).is_err());
    assert!(matches!(vcs_resolve(p, &conflicts[0].path, &VcsChoice::Value("{ nope".into())), Err(ApiError::InvalidJson(_))));
    // abort restores the committed state
    vcs_abort(p).unwrap();
    assert!(vcs_conflicts(p).unwrap().is_empty());
    assert!(vcs_abort(p).is_err());
    // merge again and resolve each side
    let VcsMergeOutcome::Conflicted(c2) = vcs_merge(p, "other", false, None, None).unwrap() else { panic!() };
    let mut left = c2.len();
    for c in &c2 {
        left = vcs_resolve(p, &c.path, &VcsChoice::Ours).unwrap();
    }
    assert_eq!(left, 0);
    let merged = vcs_commit(p, None, None).unwrap();
    assert_eq!(vcs_log(p, 1).unwrap()[0].id, merged);
    assert_eq!(vcs_merge(p, "other", false, None, None).unwrap(), VcsMergeOutcome::UpToDate);
    // revert the merge's first-parent change, then cherry-pick it back via the original commit
    let log = vcs_log(p, 10).unwrap();
    let main_edit = log.iter().find(|c| c.message == "main edit").unwrap().id.clone();
    assert!(matches!(vcs_revert(p, &main_edit, None).unwrap(), VcsMergeOutcome::Merged(_) | VcsMergeOutcome::Conflicted(_)));
    let _ = vcs_abort(p);
    // fast-forward
    vcs_branch(p, "ahead", None).unwrap();
    vcs_checkout(p, "ahead", false).unwrap();
    edit(1.0, 9.0);
    let ahead = vcs_commit(p, Some("ahead"), None).unwrap();
    vcs_checkout(p, "main", false).unwrap();
    assert_eq!(vcs_merge(p, "ahead", false, None, None).unwrap(), VcsMergeOutcome::FastForward(ahead));
    assert!(vcs_merge(p, "no-such-rev", false, None, None).is_err());
    assert!(vcs_branch(p, "x", Some("no-such-rev")).is_err());
    let _ = std::fs::remove_dir_all(&d);
    assert!(!d.exists());
}

#[test]
fn ci_modes_init_and_bless() {
    let d = scratch("ciwrite");
    let f = d.join("p.json");
    std::fs::write(&f, Animation::demo().to_json()).unwrap();
    let m = d.join("ci.json");
    std::fs::write(&m, r#"{"schema":"osatura-ci/1","project":"p.json","goldens":"goldens","checks":[{"kind":"render","name":"f","frames":[0,3],"scale":0.25}]}"#).unwrap();
    let ms = m.to_str().unwrap();
    let v = ci_run_mode(ms, CiMode::Verify).unwrap();
    assert_eq!((v.failed, v.exit_code), (1, 1), "{}", v.json);
    let u = ci_run_mode(ms, CiMode::Update).unwrap();
    assert_eq!((u.exit_code, u.updated), (0, 1), "{}", u.json);
    assert_eq!(ci_run(ms).unwrap().passed, 1);
    // change the animation: verify fails, bless accepts, verify passes
    let mut a = Animation::from_json(&std::fs::read_to_string(&f).unwrap()).unwrap();
    let bone = a.bone_names()[0].clone();
    for fr in 0..4 {
        a.set_key(&bone, "bone_angle", fr as f32, 80.0 + fr as f32 * 10.0, Interpolation::Linear).unwrap();
    }
    std::fs::write(&f, a.to_json()).unwrap();
    assert_eq!(ci_run(ms).unwrap().exit_code, 1);
    let b = ci_bless(ms).unwrap();
    assert_eq!((b.exit_code, b.blessed), (0, 1), "{}", b.json);
    assert_eq!(ci_run(ms).unwrap().exit_code, 0);
    // init: default manifest, no overwrite, optional goldens
    let f2 = d.join("q.json");
    std::fs::write(&f2, Animation::demo().to_json()).unwrap();
    let i = ci_init(f2.to_str().unwrap(), Some(d.join("q-ci.json").to_str().unwrap()), false).unwrap();
    assert!(i.recorded.is_none() && i.checks.len() >= 4 && std::path::Path::new(&i.manifest).exists());
    assert!(ci_init(f2.to_str().unwrap(), Some(&i.manifest), false).is_err());
    assert!(ci_init(d.join("none.json").to_str().unwrap(), Some(d.join("z.json").to_str().unwrap()), false).is_err());
    assert!(!d.join("z.json").exists());
    assert!(ci_run_mode(d.join("missing.json").to_str().unwrap(), CiMode::Bless).is_err());
    let _ = std::fs::remove_dir_all(&d);
    assert!(!d.exists());
}

#[test]
fn new_error_variants_display() {
    assert_eq!(ApiError::DirtyTree("x".into()).to_string(), "uncommitted changes: x");
    assert_eq!(ApiError::MergeInProgress("y".into()).to_string(), "merge in progress: y");
    assert_eq!(ApiError::Conflicts(vec!["a".into(), "b".into()]).to_string(), "unresolved conflicts: a, b");
}

#[test]
fn error_display_and_trait() {
    let e: Box<dyn std::error::Error> = Box::new(ApiError::InvalidProject(vec!["a".into(), "b".into()]));
    assert_eq!(e.to_string(), "invalid project: a; b");
    assert!(API_VERSION.starts_with(&format!("{API_VERSION_MAJOR}.")));
}

// ----- 1.4 additions -----

#[test]
fn create_layers_and_bones_are_validated_and_undoable() {
    let mut a = Animation::demo();
    let n0 = a.layer_names().len();
    a.add_group_layer("grp").unwrap();
    a.add_rect_layer("box", 1.0, 2.0, 30.0, 20.0, "#336699", Some("grp")).unwrap();
    a.add_ellipse_layer("dot", 50.0, 50.0, 5.0, 8.0, "#f00", None).unwrap();
    assert_eq!(a.layer_names().len(), n0 + 3);
    assert!(a.layer_names().iter().any(|n| n == "grp/box"));
    assert!(a.render_rgba(0.0, &RenderSettings::new(0.2)).is_ok());
    // round-trips through JSON
    assert!(Animation::from_json(&a.to_json()).is_ok());
    for bad in [
        a.add_rect_layer("box", 0.0, 0.0, 1.0, 1.0, "#fff", None),          // duplicate name
        a.add_rect_layer("x/y", 0.0, 0.0, 1.0, 1.0, "#fff", None),          // slash
        a.add_rect_layer("", 0.0, 0.0, 1.0, 1.0, "#fff", None),             // empty
        a.add_rect_layer("n1", 0.0, 0.0, -1.0, 1.0, "#fff", None),          // size
        a.add_rect_layer("n2", f32::NAN, 0.0, 1.0, 1.0, "#fff", None),      // NaN
        a.add_rect_layer("n3", 0.0, 0.0, 1.0, 1.0, "red-ish", None),        // colour
        a.add_rect_layer("n4", 0.0, 0.0, 1.0, 1.0, "#fff", Some("dot")),    // parent is not a group
        a.add_ellipse_layer("n5", 0.0, 0.0, 0.0, 1.0, "#fff", None),
    ] {
        assert!(matches!(bad, Err(ApiError::InvalidArgument(_))), "{bad:?}");
    }
    assert_eq!(a.layer_names().len(), n0 + 3, "failed adds change nothing");
    assert!(a.undo() && a.undo() && a.undo());
    assert_eq!(a.layer_names().len(), n0);

    let b0 = a.bone_names().len();
    let root = a.bone_names()[0].clone();
    a.add_bone("tip", Some(&root), 12.0, 3.0, 0.0, 15.0).unwrap();
    assert_eq!(a.bone_names().len(), b0 + 1);
    a.set_key("tip", "bone_angle", 4.0, 10.0, Interpolation::Linear).unwrap();
    assert!(a.add_bone("tip", None, 1.0, 0.0, 0.0, 0.0).is_err());
    assert!(a.add_bone("t2", Some("nope"), 1.0, 0.0, 0.0, 0.0).is_err());
    assert!(a.add_bone("t3", None, 0.0, 0.0, 0.0, 0.0).is_err());
    assert!(a.add_bone("t4", None, 1.0, f32::INFINITY, 0.0, 0.0).is_err());
}

#[test]
fn apng_and_exr_exports() {
    let a = Animation::demo();
    let apng = a.export(ExportFormat::Apng, 0.25).unwrap();
    assert_eq!(&apng[1..4], b"PNG");
    assert!(apng.windows(4).any(|w| w == b"acTL"), "animation control chunk");
    let (_, _, frames) = crate::sizefit::apng_decode(&apng).unwrap();
    assert_eq!(frames.len(), a.frame_count() as usize);
    assert!(a.export(ExportFormat::Apng, 0.0).is_err());
    let half = a.render_exr(0.0, &RenderSettings::new(0.25), false).unwrap();
    let full = a.render_exr(0.0, &RenderSettings::new(0.25), true).unwrap();
    assert_eq!(&half[0..4], &[0x76, 0x2f, 0x31, 0x01], "EXR magic");
    assert_eq!(&full[0..4], &[0x76, 0x2f, 0x31, 0x01]);
    assert!(full.len() > half.len());
    assert!(a.render_exr(f32::NAN, &RenderSettings::default(), false).is_err());
    assert!(a.render_exr(0.0, &RenderSettings::new(99.0), false).is_err());
}

#[test]
fn physics_bake_adds_channels_once_and_is_undoable() {
    let mut a = Animation::demo();
    a.add_bone("h1", None, 20.0, 10.0, 10.0, 90.0).unwrap();
    a.add_bone("h2", Some("h1"), 20.0, 20.0, 0.0, 0.0).unwrap();
    let before = a.to_json();
    let r = a.bake_physics(&["h1", "h2"], "hair", false).unwrap();
    assert_eq!(r.added, 2);
    assert!(a.to_json() != before);
    let again = a.bake_physics(&["h1", "h2"], "hair", false).unwrap();
    assert_eq!(again.added, 0, "existing keys win");
    assert_eq!(again.skipped.len(), 2);
    assert_eq!(a.bake_physics(&["h1", "h2"], "cape", true).unwrap().added, 2);
    assert!(a.undo() && a.undo());
    assert_eq!(a.to_json(), before);
    assert!(a.bake_physics(&["h1"], "lava", false).is_err());
    assert!(a.bake_physics(&[], "hair", false).is_err());
    assert!(a.bake_physics(&["ghost"], "hair", false).is_err());
}

#[test]
fn vcs_branch_list_delete_status_diff_show_gc_fsck() {
    let dir = std::env::temp_dir().join(format!("osatura-api14-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("p.json");
    std::fs::write(&file, Animation::demo().to_json()).unwrap();
    let path = file.to_str().unwrap();
    vcs_init(path, Some("ana")).unwrap();
    assert!(vcs_branches(path).unwrap().is_empty() || vcs_branches(path).unwrap().iter().all(|b| b.current));
    let c1 = vcs_commit(path, Some("first"), None).unwrap();
    let st = vcs_status(path).unwrap();
    assert_eq!(st.head.as_deref(), Some(c1.as_str()));
    assert!(st.changes.is_empty() && st.merge.is_none() && st.conflicts.is_empty());

    // edit the working file
    let mut a = Animation::from_json(&std::fs::read_to_string(&file).unwrap()).unwrap();
    a.add_rect_layer("extra", 0.0, 0.0, 5.0, 5.0, "#0f0", None).unwrap();
    std::fs::write(&file, a.to_json()).unwrap();
    let st = vcs_status(path).unwrap();
    assert!(st.changes.iter().any(|c| c.kind == "Added" && c.path.contains("extra") || c.description.contains("extra")), "{:?}", st.changes);
    let d = vcs_diff(path, Some("HEAD"), None).unwrap();
    assert_eq!(d, st.changes);
    assert!(vcs_diff(path, Some("HEAD"), Some("HEAD")).unwrap().is_empty());
    let c2 = vcs_commit(path, Some("second"), None).unwrap();
    let shown = vcs_show(path, &c2).unwrap();
    assert_eq!(shown.commit.id, c2);
    assert_eq!(shown.commit.message, "second");
    assert!(!shown.changes.is_empty());
    assert!(!vcs_show(path, &c1).unwrap().changes.is_empty(), "a root commit shows everything");
    assert!(vcs_show(path, "nope").is_err());

    vcs_branch(path, "feature", Some(&c1)).unwrap();
    let bs = vcs_branches(path).unwrap();
    assert!(bs.iter().any(|b| b.name == "feature" && !b.current && b.commit == c1));
    let cur = bs.iter().find(|b| b.current).unwrap().name.clone();
    assert!(vcs_branch_delete(path, &cur).is_err(), "cannot delete the checked-out branch");
    assert!(vcs_branch_delete(path, "ghost").is_err());
    vcs_branch_delete(path, "feature").unwrap();
    assert!(!vcs_branches(path).unwrap().iter().any(|b| b.name == "feature"));

    assert!(vcs_fsck(path).unwrap().is_empty());
    let g = vcs_gc(path).unwrap();
    assert!(g.kept > 0);
    assert!(vcs_fsck(path).unwrap().is_empty());
    assert!(vcs_status("/nonexistent-osatura-dir/p.json").is_err());
    assert!(vcs_gc("/nonexistent-osatura-dir/p.json").is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

/// Surface snapshot: the public items of this file must equal `api_surface.txt`.
/// Regenerate deliberately with `OSATURA_BLESS=1 cargo test -p osatura-core api::`.
#[test]
fn api_surface_snapshot() {
    // the facade is split over several files: read them in module order (mod.rs head, then the parts)
    let src = [include_str!("mod.rs"), include_str!("facade.rs"), include_str!("vcs.rs"), include_str!("ci_write.rs")].join("\n");
    let mut items: Vec<String> = Vec::new();
    let mut ctx = String::new();
    let mut in_enum = false;
    for line in src.lines() {
        if line.starts_with("#[cfg(test)]") || (line.starts_with("pub use ") && line.ends_with("::*;")) {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('#') && !line.starts_with("//") && !line.is_empty() && !line.starts_with('}') {
            in_enum = line.starts_with("pub enum ");
            ctx = if line.starts_with("impl") || line.starts_with("pub struct") || line.starts_with("pub enum") {
                line.trim_end_matches('{').trim().to_string()
            } else {
                String::new()
            };
        }
        let t = line.trim_end();
        let top = !line.starts_with(' ');
        if line.trim_end() == "#[non_exhaustive]" {
            items.push(line.trim_end().to_string());
        }
        let item = if top && t.starts_with("pub ") {
            Some(t.trim_end_matches('{').trim_end_matches(';').trim().to_string())
        } else if !top && line.starts_with("    ") && !line.starts_with("     ") {
            let b = t.trim_start();
            if b.starts_with("pub ") {
                Some(format!("{ctx} :: {}", b.trim_end_matches('{').trim_end_matches(',').trim_end_matches(';').trim()))
            } else if in_enum && b.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                Some(format!("{ctx} :: {}", b.trim_end_matches(',')))
            } else {
                None
            }
        } else {
            None
        };
        if let Some(i) = item {
            if !i.contains("pub use") || top {
                items.push(i);
            }
        }
    }
    let got = items.join("\n") + "\n";
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/api_surface.txt");
    if std::env::var("OSATURA_BLESS").is_ok() {
        std::fs::write(path, &got).unwrap();
    }
    let want = include_str!("../api_surface.txt");
    assert_eq!(got, want, "public API surface changed: if intended, bump API_VERSION and rerun with OSATURA_BLESS=1");
}
