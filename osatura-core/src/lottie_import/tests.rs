// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `lottie_import`.
#![allow(unused_imports)]

#[test]
fn files_with_millions_of_values_are_refused_up_front() {
    let kf = r#"{"t":0,"s":[0]}"#;
    let many = vec![kf; 1_500_000].join(",");
    let doc = format!(r#"{{"w":10,"h":10,"fr":24,"ip":0,"op":10,"layers":[{{"ty":4,"ks":{{"o":{{"a":1,"k":[{many}]}}}}}}]}}"#);
    let err = import_lottie(&doc).err().expect("must be refused");
    assert!(err.contains("more than the limit"), "{err}");
    assert!(import_lottie(r#"{"w":10,"h":10,"fr":24,"ip":0,"op":10,"layers":[]}"#).is_ok());
    let err = import_lottie(r#"{"w":1e30,"h":10,"fr":24,"ip":0,"op":10,"layers":[]}"#).err().expect("1e30 stage");
    assert!(err.contains("supported range"), "{err}");
}

use super::*;
use crate::project::Source;
use crate::Engine;
use serde_json::json;

fn doc() -> String {
    json!({
        "v": "5.7.0", "fr": 24, "ip": 0, "op": 25, "w": 200, "h": 100, "nm": "t", "assets": [],
        "layers": [
            {"ty": 4, "nm": "top", "ind": 1, "ip": 0, "op": 25, "bm": 1,
             "ks": {"o": {"a": 0, "k": 50}, "p": {"a": 1, "k": [{"t": 0, "s": [10, 20, 0], "h": 0}, {"t": 24, "s": [110, 20, 0]}]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}},
             "shapes": [{"ty": "gr", "it": [
                {"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [40, 20]}, "r": {"a": 0, "k": 0}},
                {"ty": "el", "p": {"a": 0, "k": [60, 20]}, "s": {"a": 0, "k": [20, 20]}},
                {"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}},
                {"ty": "st", "c": {"a": 0, "k": [0, 0, 1, 1]}, "o": {"a": 0, "k": 100}, "w": {"a": 0, "k": 3}},
                {"ty": "tr", "p": {"a": 0, "k": [0, 0]}, "a": {"a": 0, "k": [0, 0]}, "s": {"a": 0, "k": [100, 100]}, "r": {"a": 0, "k": 0}, "o": {"a": 0, "k": 100}}]}]},
            {"ty": 4, "nm": "bottom", "ind": 2, "ks": {},
             "shapes": [{"ty": "sh", "ks": {"a": 0, "k": {"i": [[0, 0], [0, 0], [0, 0]], "o": [[0, 0], [0, 0], [0, 0]], "v": [[0, 0], [50, 0], [25, 40]], "c": true}}},
                        {"ty": "fl", "c": {"a": 0, "k": [0, 1, 0, 1]}, "o": {"a": 0, "k": 100}}]},
            {"ty": 1, "nm": "bg", "sc": "#112233", "sw": 200, "sh": 100, "ks": {}},
            {"ty": 0, "nm": "precomp", "ks": {}}
        ]})
    .to_string()
}

#[test]
fn imports_shapes_styles_and_animation() {
    let r = import_lottie(&doc()).unwrap();
    let p = &r.project;
    assert_eq!((p.width, p.height, p.fps, p.frames), (200.0, 100.0, 24.0, 25));
    assert_eq!(p.layers.len(), 2);
    assert_eq!(p.layers[0].name, "bottom", "Lottie lists the top layer first; we draw bottom first");
    assert_eq!(p.layers[1].blend, Blend::Multiply);
    assert!((p.layers[1].opacity - 0.5).abs() < 1e-6);
    assert!(p.background.is_some());
    let crate::project::LayerKind::Vector { paths } = &p.layers[1].kind else { panic!() };
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().all(|x| x.fill.is_some() && x.stroke.is_some()));
    assert!(paths[1].anchors.iter().any(|a| a.handle_out.is_some()), "ellipse has curve handles");
    let e = Engine::new(p.clone()).unwrap();
    let x = |f: f32| e.evaluate(f).nodes.iter().find(|n| n.name == "top").unwrap().world.tx;
    assert!((x(0.0) - 10.0).abs() < 1e-3 && (x(24.0) - 110.0).abs() < 1e-3 && (x(12.0) - 60.0).abs() < 1e-3);
    assert!(r.warnings.iter().any(|w| w.contains("precomp layer")), "{:?}", r.warnings);
    assert!(r.warnings.iter().any(|w| w.contains("without easing")));
}

#[test]
fn roundtrips_with_our_own_exporter() {
    let e = Engine::new(crate::demo::project()).unwrap();
    let j = crate::export::lottie_json(&e).to_string();
    let back = import_lottie(&j).unwrap();
    assert_eq!(back.project.frames, e.project.frames);
    assert!(!back.project.layers.is_empty());
    Engine::new(back.project).unwrap();
}

/// Export -> import -> render: the frames agree (static paths, animated paths, transforms).
#[test]
fn roundtrip_pictures_match_for_the_demo() {
    let e = Engine::new(crate::demo::project()).unwrap();
    let j = crate::export::lottie_json(&e).to_string();
    let back = import_lottie(&j).unwrap();
    assert!(!back.warnings.iter().any(|w| w.contains("path")), "{:?}", back.warnings);
    let re = Engine::new(back.project).unwrap();
    for f in [0.0, 6.0, 13.0, 25.0] {
        let (a, b) = (render_img(&e, f), render_img(&re, f));
        let ssim = crate::percep::ssim(&a, &b).unwrap();
        eprintln!("demo frame {f}: ssim {ssim}");
        assert!(ssim > 0.97, "frame {f}: ssim {ssim}");
    }
}

/// A path that morphs (point offsets) and a moving layer survive export -> import.
#[test]
fn roundtrip_animated_path_and_transform() {
    use crate::project::Channel;
    let mut p = Project::new("ap", 80.0, 60.0, 12.0, 13);
    p.background = None;
    let tri = crate::path::VPath::new("tri", true, vec![
        crate::path::Anchor::corner(Vec2::new(10.0, 40.0)),
        crate::path::Anchor::corner(Vec2::new(30.0, 40.0)),
        crate::path::Anchor::corner(Vec2::new(20.0, 10.0)),
    ]).with_fill("#cc2244");
    let still = Layer::vector("still", vec![crate::path::VPath::rect("r", 50.0, 5.0, 20.0, 10.0).with_fill("#2255cc")]);
    p.layers = vec![Layer::vector("tri", vec![tri]), still];
    p.channels = vec![
        Channel::keys("tri::tri::2", Prop::PointDy, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, -8.0, Ease::Linear)])),
        Channel::keys("tri::tri::2", Prop::PointDx, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, 15.0, Ease::Linear)])),
        Channel::keys("still", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, 10.0, Ease::Linear)])),
    ];
    let e = Engine::new(p).unwrap();
    let j = crate::export::lottie_json(&e).to_string();
    let back = import_lottie(&j).unwrap();
    assert!(back.warnings.iter().all(|w| !w.contains("skipped")), "{:?}", back.warnings);
    let re = Engine::new(back.project).unwrap();
    for f in [0.0, 3.0, 6.0, 12.0] {
        let (a, b) = (render_img(&e, f), render_img(&re, f));
        let mse = crate::percep::mse(&a, &b).unwrap();
        assert!(mse < 12.0, "frame {f}: mse {mse}");
    }
    // the triangle's tip really moved in the re-imported file
    let (a0, a12) = (render_img(&re, 0.0), render_img(&re, 12.0));
    assert!(crate::percep::mse(&a0, &a12).unwrap() > 50.0);
}

/// Keyframes that change the number of points cannot be morphed: first shape + a warning.
#[test]
fn path_keys_with_other_topology_warn_and_use_the_first_shape() {
    let sh = |pts: &[[f64; 2]]| json!({"v": pts, "i": pts.iter().map(|_| [0, 0]).collect::<Vec<_>>(), "o": pts.iter().map(|_| [0, 0]).collect::<Vec<_>>(), "c": true});
    let ks = json!({"a": 1, "k": [
        {"t": 0, "s": [sh(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]])], "h": 1},
        {"t": 5, "s": [sh(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])], "h": 1}]});
    let shapes = json!([{"ty": "sh", "ks": ks}, {"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}}]);
    let r = run(lay("a", 1, stat_ks([0.0, 0.0]), shapes, json!({})));
    assert!(r.warnings.iter().any(|w| w.contains("number of points")), "{:?}", r.warnings);
    match &r.project.layers[0].kind {
        crate::project::LayerKind::Vector { paths } => assert_eq!(paths[0].anchors.len(), 3),
        _ => panic!(),
    }
}

/// Eased keys and holds carry over to the morph weights.
#[test]
fn animated_path_keys_keep_hold_and_interpolate_linearly() {
    let sq = |x: f64| json!({"v": [[x, 0.0], [x + 10.0, 0.0], [x + 10.0, 10.0], [x, 10.0]], "i": [[0,0],[0,0],[0,0],[0,0]], "o": [[0,0],[0,0],[0,0],[0,0]], "c": true});
    let ks = json!({"a": 1, "k": [
        {"t": 0, "s": [sq(0.0)], "i": {"x": [1], "y": [1]}, "o": {"x": [0], "y": [0]}},
        {"t": 10, "s": [sq(20.0)], "h": 1},
        {"t": 20, "s": [sq(20.0)]}]});
    let shapes = json!([{"ty": "sh", "ks": ks}, {"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}}]);
    let r = run(lay("a", 1, stat_ks([0.0, 0.0]), shapes, json!({})));
    let e = Engine::new(r.project).unwrap();
    let x0 = |f: f32| { let fr = e.evaluate(f); fr.nodes.iter().find_map(|n| n.paths.first().map(|p| p.anchors[0].pos.x)).unwrap() };
    assert!((x0(0.0) - 0.0).abs() < 1e-3 && (x0(5.0) - 10.0).abs() < 1e-3 && (x0(10.0) - 20.0).abs() < 1e-3, "{} {} {}", x0(0.0), x0(5.0), x0(10.0));
    assert!((x0(15.0) - 20.0).abs() < 1e-3);
}

#[test]
fn rejects_garbage_and_limits() {
    assert!(import_lottie("not json").is_err());
    assert!(import_lottie("{}").is_err());
    assert!(import_lottie(&json!({"fr": 24, "ip": 0, "op": 10, "w": 0, "h": 10, "layers": []}).to_string()).is_err());
    assert!(import_lottie(&json!({"fr": 24, "ip": 0, "op": 1e9, "w": 10, "h": 10, "layers": []}).to_string()).is_err());
    let r = import_lottie(&json!({"fr": 24, "ip": 0, "op": 10, "w": 10, "h": 10, "layers": [{"ty": 4, "nm": "a", "ks": {}, "shapes": [{"ty": "sh", "ks": {"a": 0, "k": {"v": "x"}}}]}]}).to_string()).unwrap();
    assert!(r.warnings.iter().any(|w| w.contains("malformed")));
}

#[test]
fn bezier_handles_become_easing() {
    let d = json!({
        "v": "5.7.0", "fr": 24, "ip": 0, "op": 25, "w": 100, "h": 100, "nm": "e", "assets": [],
        "layers": [{"ty": 4, "nm": "l", "ind": 1, "ip": 0, "op": 25,
            "ks": {"o": {"a": 1, "k": [
                {"t": 0, "s": [0], "o": {"x": [0.9], "y": [0.0]}, "i": {"x": [0.1], "y": [1.0]}},
                {"t": 24, "s": [100]}]}, "p": {"a": 0, "k": [0, 0, 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}},
            "shapes": [{"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}]}]
    });
    let imp = import_lottie(&d.to_string()).unwrap();
    assert!(!imp.warnings.iter().any(|w| w.contains("without easing")), "{:?}", imp.warnings);
    let ch = imp.project.channels.iter().find(|c| c.prop == Prop::LayerOpacity).unwrap();
    let Source::Keys { track } = &ch.source else { panic!() };
    assert!(matches!(track.keys[0].ease, Ease::CubicBezier { .. }));
    let mid = track.eval(6.0).unwrap();
    assert!(mid < 0.2, "strong ease-in: slow start, got {mid}");
    // diagonal handles are plain linear
    let lin = d.to_string().replace("\"x\":[0.9]", "\"x\":[0.3]").replace("\"y\":[0.0]", "\"y\":[0.3]").replace("\"x\":[0.1]", "\"x\":[0.7]").replace("\"y\":[1.0]", "\"y\":[0.7]");
    let imp = import_lottie(&lin).unwrap();
    let ch = imp.project.channels.iter().find(|c| c.prop == Prop::LayerOpacity).unwrap();
    let Source::Keys { track } = &ch.source else { panic!() };
    assert_eq!(track.keys[0].ease, Ease::Linear);
}

#[test]
fn curved_position_path_is_baked() {
    let d = json!({
        "v": "5.7.0", "fr": 24, "ip": 0, "op": 25, "w": 200, "h": 100, "nm": "c", "assets": [],
        "layers": [{"ty": 4, "nm": "l", "ind": 1, "ip": 0, "op": 25,
            "ks": {"o": {"a": 0, "k": 100}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0},
                "p": {"a": 1, "k": [
                    {"t": 0, "s": [0, 0, 0], "to": [0, 50, 0], "ti": [0, 50, 0], "o": {"x": [0.3], "y": [0.3]}, "i": {"x": [0.7], "y": [0.7]}},
                    {"t": 24, "s": [100, 0, 0]}]}},
            "shapes": [{"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}]}]
    });
    let imp = import_lottie(&d.to_string()).unwrap();
    assert!(!imp.warnings.iter().any(|w| w.contains("motion paths")), "{:?}", imp.warnings);
    let get = |prop: Prop| {
        let ch = imp.project.channels.iter().find(|c| c.prop == prop).unwrap();
        let Source::Keys { track } = &ch.source else { panic!() };
        track.clone()
    };
    let (x, y) = (get(Prop::LayerX), get(Prop::LayerY));
    assert!((x.eval(12.0).unwrap() - 50.0).abs() < 0.5 && (y.eval(12.0).unwrap() - 37.5).abs() < 0.5, "{:?} {:?}", x.eval(12.0), y.eval(12.0));
    assert!(y.eval(0.0).unwrap().abs() < 1e-3 && y.eval(24.0).unwrap().abs() < 1e-3 && (x.eval(24.0).unwrap() - 100.0).abs() < 1e-3);
    assert_eq!(x.keys.len(), 25, "one key per frame, no duplicates");
}

#[test]
fn layer_in_out_range_hides_outside() {
    let d = json!({
        "v": "5.7.0", "fr": 24, "ip": 0, "op": 30, "w": 100, "h": 100, "nm": "r", "assets": [],
        "layers": [{"ty": 4, "nm": "l", "ind": 1, "ip": 10, "op": 20,
            "ks": {"o": {"a": 0, "k": 100}, "p": {"a": 0, "k": [0, 0, 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}},
            "shapes": [{"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}]}]
    });
    let imp = import_lottie(&d.to_string()).unwrap();
    assert!(!imp.warnings.iter().any(|w| w.contains("in/out")), "{:?}", imp.warnings);
    let e = Engine::new(imp.project).unwrap();
    let op = |f: f32| e.evaluate(f).nodes.iter().find(|n| n.name == "l").map(|n| n.opacity).unwrap_or(0.0);
    assert_eq!((op(5.0), op(10.0), op(19.0), op(20.0), op(29.0)), (0.0, 1.0, 1.0, 0.0, 0.0));
}

#[test]
fn precomp_becomes_a_group_with_time_mapping() {
    let d = json!({
        "v": "5.7.0", "fr": 24, "ip": 0, "op": 50, "w": 200, "h": 100, "nm": "pc",
        "assets": [{"id": "comp_0", "layers": [
            {"ty": 4, "nm": "dot", "ind": 1, "ip": 0, "op": 50,
             "ks": {"o": {"a": 0, "k": 100}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0},
                    "p": {"a": 1, "k": [{"t": 0, "s": [0, 0, 0]}, {"t": 10, "s": [100, 0, 0]}]}},
             "shapes": [{"ty": "rc", "p": {"a": 0, "k": [0, 0]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}]}]}],
        "layers": [{"ty": 0, "nm": "pre", "refId": "comp_0", "ind": 1, "ip": 0, "op": 50, "st": 20, "sr": 2,
            "ks": {"o": {"a": 0, "k": 100}, "p": {"a": 0, "k": [5, 7, 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}}}]
    });
    let imp = import_lottie(&d.to_string()).unwrap();
    assert!(!imp.warnings.iter().any(|w| w.contains("not imported") || w.contains("skipped")), "{:?}", imp.warnings);
    let e = Engine::new(imp.project.clone()).unwrap();
    let x = |f: f32| e.evaluate(f).nodes.iter().find(|n| n.name == "pre/dot").unwrap().world.tx;
    // comp time 0..10 plays over project frames 20..40 (start 20, stretch 2), plus the group offset of 5
    assert!((x(10.0) - 5.0).abs() < 1e-3, "before the start: first value");
    assert!((x(30.0) - 55.0).abs() < 1e-3, "halfway through: {}", x(30.0));
    assert!((x(45.0) - 105.0).abs() < 1e-3, "after the end: last value");
    assert!(imp.project.channels.iter().filter(|c| c.target == "pre/dot").count() > 0);
}

#[test]
fn alpha_mattes_become_layer_masks() {
    let rect = |x: f64| json!([{"ty": "rc", "p": {"a": 0, "k": [x, 20]}, "s": {"a": 0, "k": [20, 20]}, "r": {"a": 0, "k": 0}}, {"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}}]);
    let d = json!({
        "v": "5.7.0", "fr": 24, "ip": 0, "op": 5, "w": 100, "h": 100, "nm": "m", "assets": [],
        "layers": [
            {"ty": 4, "nm": "matte", "ind": 1, "td": 1, "ks": {}, "shapes": rect(20.0)},
            {"ty": 4, "nm": "art", "ind": 2, "tt": 2, "ks": {}, "shapes": rect(20.0)}]
    });
    let imp = import_lottie(&d.to_string()).unwrap();
    assert!(!imp.warnings.iter().any(|w| w.contains("matte") || w.contains("'tt'") || w.contains("'td'")), "{:?}", imp.warnings);
    let art = imp.project.layers.iter().find(|l| l.name == "art").unwrap();
    let m = art.mask.as_ref().unwrap();
    assert_eq!((m.layer.as_str(), m.invert), ("matte", true));
    assert!(!imp.project.layers.iter().find(|l| l.name == "matte").unwrap().visible);
    Engine::new(imp.project).unwrap();
    // round trip through our own export: the matte pair comes back as one masked layer
    let mut p = Project::new("rt", 100.0, 100.0, 24.0, 2);
    let mut a = Layer::vector("art", vec![crate::path::VPath::rect("r", 0.0, 0.0, 30.0, 30.0).with_fill("#f00")]);
    a.mask = Some(crate::project::MaskRef { layer: "hole".into(), invert: false });
    let mut h = Layer::vector("hole", vec![crate::path::VPath::rect("r", 5.0, 5.0, 10.0, 10.0).with_fill("#fff")]);
    h.visible = false;
    p.layers = vec![h, a];
    let lot = crate::export::lottie_json(&Engine::new(p).unwrap()).to_string();
    let back = import_lottie(&lot).unwrap();
    assert!(back.project.layers.iter().any(|l| l.mask.is_some()), "{:?}", back.warnings);
}

// ---- parenting, masks, luma mattes, images, effects ----

fn base_doc(layers: Value, assets: Value) -> String {
    json!({"v": "5.7.0", "fr": 10, "ip": 0, "op": 11, "w": 100, "h": 100, "nm": "x", "assets": assets, "layers": layers}).to_string()
}

fn rect_shapes(x: f64, y: f64, w: f64, h: f64, rgb: [f64; 3]) -> Value {
    json!([{"ty": "rc", "p": {"a": 0, "k": [x + w / 2.0, y + h / 2.0]}, "s": {"a": 0, "k": [w, h]}, "r": {"a": 0, "k": 0}},
           {"ty": "fl", "c": {"a": 0, "k": [rgb[0], rgb[1], rgb[2], 1]}, "o": {"a": 0, "k": 100}}])
}

fn lay(nm: &str, ind: i64, ks: Value, shapes: Value, extra: Value) -> Value {
    let mut base = json!({"ty": 4, "nm": nm, "ind": ind, "ip": 0, "op": 11, "ks": ks, "shapes": shapes});
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

fn stat_ks(p: [f64; 2]) -> Value {
    json!({"o": {"a": 0, "k": 100}, "p": {"a": 0, "k": [p[0], p[1], 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}})
}

fn rect_shape_json(x0: f64, y0: f64, x1: f64, y1: f64) -> Value {
    json!({"i": [[0, 0], [0, 0], [0, 0], [0, 0]], "o": [[0, 0], [0, 0], [0, 0], [0, 0]], "v": [[x0, y0], [x1, y0], [x1, y1], [x0, y1]], "c": true})
}

fn mask(mode: &str, inv: bool, op: f64, x0: f64, y0: f64, x1: f64, y1: f64) -> Value {
    json!({"mode": mode, "inv": inv, "nm": "m", "o": {"a": 0, "k": op}, "x": {"a": 0, "k": 0}, "pt": {"a": 0, "k": rect_shape_json(x0, y0, x1, y1)}})
}

fn alpha_at(e: &Engine, frame: f32, x: usize, y: usize) -> u8 {
    let (w, _, d) = crate::raster::render_rgba(e, frame, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: true });
    d[(y * w as usize + x) * 4 + 3]
}

fn masked(masks: Value) -> Engine {
    let l = lay("art", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({"masksProperties": masks}));
    let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    Engine::new(imp.project).unwrap()
}

#[test]
fn static_parent_is_baked_and_opacity_is_not_inherited() {
    let mut ks_null = stat_ks([10.0, 5.0]);
    ks_null["s"] = json!({"a": 0, "k": [200, 200, 100]});
    ks_null["o"] = json!({"a": 0, "k": 10});
    let nl = json!({"ty": 3, "nm": "ctl", "ind": 1, "ip": 0, "op": 11, "ks": ks_null});
    let mut ks = stat_ks([3.0, 4.0]);
    ks["o"] = json!({"a": 0, "k": 50});
    let child = lay("child", 2, ks, rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 1}));
    let imp = import_lottie(&base_doc(json!([child, nl]), json!([]))).unwrap();
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    assert_eq!(imp.project.layers.len(), 1, "the null layer is not a drawn layer");
    assert!(imp.project.channels.is_empty(), "static chain: no channels");
    let e = Engine::new(imp.project).unwrap();
    let f = e.evaluate(0.0);
    let n = f.nodes.iter().find(|n| n.name == "child").unwrap();
    assert!((n.world.tx - 16.0).abs() < 1e-4 && (n.world.ty - 13.0).abs() < 1e-4 && (n.world.a - 2.0).abs() < 1e-4);
    assert!((n.opacity - 0.5).abs() < 1e-5, "Lottie parenting inherits the transform only");
}

#[test]
fn animated_parent_chain_matches_the_reference_matrix() {
    // grandparent rotates, parent has non-uniform scale and is itself a drawn layer, child is rotated about an anchor
    let rot = json!({"a": 1, "k": [{"t": 0, "s": [0], "o": {"x": [0.2], "y": [0.0]}, "i": {"x": [0.8], "y": [1.0]}}, {"t": 10, "s": [90]}]});
    let mut gks = stat_ks([50.0, 50.0]);
    gks["r"] = rot.clone();
    let mut pks = stat_ks([20.0, 0.0]);
    pks["s"] = json!({"a": 0, "k": [100, 50, 100]});
    let mut cks = stat_ks([5.0, 7.0]);
    cks["r"] = json!({"a": 0, "k": 30});
    cks["a"] = json!({"a": 0, "k": [2, 1, 0]});
    let g = json!({"ty": 3, "nm": "g", "ind": 1, "ip": 0, "op": 11, "ks": gks});
    let p = lay("p", 2, pks, rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 1.0, 0.0]), json!({"parent": 1}));
    let c = lay("c", 3, cks, rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 2}));
    let imp = import_lottie(&base_doc(json!([c, p, g]), json!([]))).unwrap();
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    assert_eq!(imp.project.layers.len(), 2, "draw order c above p is kept");
    assert_eq!(imp.project.layers[1].name, "c");
    let e = Engine::new(imp.project).unwrap();
    // reference: ease through the same cubic bezier
    let ease = Ease::CubicBezier { x1: 0.2, y1: 0.0, x2: 0.8, y2: 1.0 };
    for f in [0.0f32, 1.0, 4.0, 5.0, 9.0, 10.0] {
        let angle = 90.0 * ease.apply(f / 10.0);
        let m = compose_transform(Vec2::new(50.0, 50.0), angle, Vec2::new(1.0, 1.0), 0.0, Vec2::ZERO)
            .mul(compose_transform(Vec2::new(20.0, 0.0), 0.0, Vec2::new(1.0, 0.5), 0.0, Vec2::ZERO))
            .mul(compose_transform(Vec2::new(5.0, 7.0), 30.0, Vec2::new(1.0, 1.0), 0.0, Vec2::new(2.0, 1.0)));
        let w = e.evaluate(f).nodes.iter().find(|n| n.name == "c").unwrap().world;
        for (got, want) in [(w.a, m.a), (w.b, m.b), (w.c, m.c), (w.d, m.d), (w.tx, m.tx), (w.ty, m.ty)] {
            assert!((got - want).abs() < 2e-3, "frame {f}: {w:?} vs {m:?}");
        }
    }
}

#[test]
fn parent_links_that_cannot_be_followed_are_reported() {
    let a = lay("a", 1, stat_ks([1.0, 2.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 2}));
    let b = lay("b", 2, stat_ks([3.0, 4.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 1}));
    let lost = lay("lost", 3, stat_ks([3.0, 4.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 99}));
    let imp = import_lottie(&base_doc(json!([a, b, lost]), json!([]))).unwrap();
    assert_eq!(imp.project.layers.len(), 3);
    assert!(imp.warnings.iter().any(|w| w.contains("cycle")), "{:?}", imp.warnings);
    assert!(imp.warnings.iter().any(|w| w.contains("parent layer 99 not found")));
    Engine::new(imp.project).unwrap();
}

#[test]
fn mask_modes_add_subtract_intersect_invert_opacity() {
    let e = masked(json!([mask("a", false, 100.0, 10.0, 10.0, 30.0, 30.0)]));
    assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (255, 0), "add");
    let e = masked(json!([mask("s", false, 100.0, 10.0, 10.0, 30.0, 30.0)]));
    assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (0, 255), "subtract");
    let e = masked(json!([mask("a", true, 100.0, 10.0, 10.0, 30.0, 30.0)]));
    assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (0, 255), "inverted add");
    let e = masked(json!([mask("a", false, 100.0, 0.0, 0.0, 30.0, 30.0), mask("i", false, 100.0, 10.0, 10.0, 40.0, 40.0)]));
    assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 5, 5), alpha_at(&e, 0.0, 35, 35)), (255, 0, 0), "intersect");
    let e = masked(json!([mask("a", false, 100.0, 0.0, 0.0, 40.0, 40.0), mask("s", false, 100.0, 10.0, 10.0, 30.0, 30.0)]));
    assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (0, 255), "ring: add then subtract");
    let e = masked(json!([mask("a", false, 100.0, 0.0, 0.0, 20.0, 40.0), mask("a", false, 100.0, 20.0, 0.0, 40.0, 40.0)]));
    assert_eq!((alpha_at(&e, 0.0, 10, 20), alpha_at(&e, 0.0, 30, 20)), (255, 255), "two adds are a union");
    let e = masked(json!([mask("a", false, 50.0, 10.0, 10.0, 30.0, 30.0)]));
    let a = alpha_at(&e, 0.0, 20, 20);
    assert!((a as i32 - 128).abs() <= 2, "mask opacity: {a}");
    let e = masked(json!([mask("n", false, 100.0, 10.0, 10.0, 30.0, 30.0)]));
    assert_eq!(alpha_at(&e, 0.0, 3, 3), 255, "mode none draws nothing");
}

#[test]
fn keyed_mask_path_morphs_and_follows_the_layer() {
    let keyed = json!({"mode": "a", "inv": false, "nm": "k", "o": {"a": 0, "k": 100}, "x": {"a": 0, "k": 0},
        "pt": {"a": 1, "k": [{"t": 0, "s": [rect_shape_json(10.0, 10.0, 30.0, 30.0)]}, {"t": 10, "s": [rect_shape_json(0.0, 0.0, 40.0, 40.0)]}]}});
    let e = masked(json!([keyed.clone()]));
    assert_eq!((alpha_at(&e, 0.0, 7, 20), alpha_at(&e, 5.0, 7, 20), alpha_at(&e, 5.0, 3, 20), alpha_at(&e, 10.0, 2, 20)), (0, 255, 0, 255));
    // the layer is moved: masks live in layer space
    let l = lay("art", 1, stat_ks([20.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({"masksProperties": [mask("a", false, 100.0, 10.0, 10.0, 30.0, 30.0)]}));
    let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
    let e = Engine::new(imp.project).unwrap();
    assert_eq!((alpha_at(&e, 0.0, 35, 20), alpha_at(&e, 0.0, 15, 20)), (255, 0));
}

#[test]
fn masks_work_on_precomps_and_with_mattes() {
    let inner = lay("dot", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({}));
    let pre = json!({"ty": 0, "nm": "pre", "refId": "c0", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]),
        "masksProperties": [mask("a", false, 100.0, 10.0, 10.0, 30.0, 30.0), mask("s", false, 100.0, 18.0, 18.0, 22.0, 22.0)]});
    let imp = import_lottie(&base_doc(json!([pre]), json!([{"id": "c0", "layers": [inner]}]))).unwrap();
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    let e = Engine::new(imp.project).unwrap();
    assert_eq!((alpha_at(&e, 0.0, 12, 12), alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (255, 0, 0));
    // a masked layer as matte source: masks are dropped with a warning
    let src = lay("src", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 1.0, 1.0]), json!({"td": 1, "masksProperties": [mask("a", false, 100.0, 10.0, 10.0, 30.0, 30.0)]}));
    let art = lay("art", 2, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({"tt": 1}));
    let imp = import_lottie(&base_doc(json!([src, art]), json!([]))).unwrap();
    assert!(imp.warnings.iter().any(|w| w.contains("track-matte source")), "{:?}", imp.warnings);
    Engine::new(imp.project).unwrap();
}

#[test]
fn luma_mattes_use_luminance() {
    let shapes = json!([{"ty": "gr", "it": rect_shapes(0.0, 0.0, 20.0, 40.0, [1.0, 1.0, 1.0])}, {"ty": "gr", "it": rect_shapes(20.0, 0.0, 20.0, 40.0, [0.2, 0.2, 0.2])}]);
    let build = |tt: i64| {
        let src = lay("src", 1, stat_ks([0.0, 0.0]), shapes.clone(), json!({"td": 1}));
        let art = lay("art", 2, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({"tt": tt}));
        let imp = import_lottie(&base_doc(json!([src, art]), json!([]))).unwrap();
        assert!(imp.warnings.iter().any(|w| w.contains("luma matte approximated")), "{:?}", imp.warnings);
        assert!(!imp.warnings.iter().any(|w| w.contains("not imported")));
        Engine::new(imp.project).unwrap()
    };
    let e = build(3);
    let (a, b) = (alpha_at(&e, 0.0, 10, 20), alpha_at(&e, 0.0, 30, 20));
    assert!(a >= 254 && (b as i32 - 51).abs() <= 2, "{a} {b}");
    let e = build(4);
    let (a, b, outside) = (alpha_at(&e, 0.0, 10, 20), alpha_at(&e, 0.0, 30, 20), alpha_at(&e, 0.0, 60, 20));
    assert!(a <= 1 && (b as i32 - 204).abs() <= 2 && outside == 0, "{a} {b} {outside}");
}

fn png_uri(rgba: [u8; 4]) -> String {
    let px: Vec<u8> = (0..16).flat_map(|_| rgba).collect();
    format!("data:image/png;base64,{}", crate::base64::encode(&crate::open_formats::encode_rgba_png(4, 4, &px).unwrap()))
}

#[test]
fn embedded_images_become_image_layers() {
    let asset = |id: &str, p: String| json!({"id": id, "w": 20, "h": 20, "u": "", "p": p, "e": 1});
    let il = |nm: &str, r: &str| json!({"ty": 2, "nm": nm, "refId": r, "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([10.0, 10.0])});
    let imp = import_lottie(&base_doc(json!([il("pic", "i0")]), json!([asset("i0", png_uri([0, 0, 255, 255]))]))).unwrap();
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    let crate::project::LayerKind::Image { size, .. } = &imp.project.layers[0].kind else { panic!() };
    assert_eq!(*size, Some(Vec2::new(20.0, 20.0)), "asset w/h is the display size");
    let e = Engine::new(imp.project).unwrap();
    assert_eq!((alpha_at(&e, 0.0, 15, 15), alpha_at(&e, 0.0, 5, 5)), (255, 0));
    // JPEG is re-encoded to PNG
    let jpg = "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAIBAQEBAQIBAQECAgICAgQDAgICAgUEBAMEBgUGBgYFBgYGBwkIBgcJBwYGCAsICQoKCgoKBggLDAsKDAkKCgr/2wBDAQICAgICAgUDAwUKBwYHCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgr/wAARCAAIAAgDASIAAhEBAxEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwD5Hooor+Xz/eA//9k=";
    let imp = import_lottie(&base_doc(json!([il("jpg", "j")]), json!([asset("j", format!("data:image/jpeg;base64,{jpg}"))]))).unwrap();
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    let e = Engine::new(imp.project).unwrap();
    let (w, _, d) = crate::raster::render_rgba(&e, 0.0, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: true });
    let o = (15 * w as usize + 15) * 4;
    assert!(d[o] > 200 && d[o + 1] < 60 && d[o + 3] == 255, "{:?}", &d[o..o + 4]);
}

#[test]
fn unusable_images_are_skipped_with_a_reason() {
    let il = |nm: &str, r: &str| json!({"ty": 2, "nm": nm, "refId": r, "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0])});
    let assets = json!([{"id": "ext", "w": 10, "h": 10, "u": "images/", "p": "a.png", "e": 0},
        {"id": "bad", "w": 10, "h": 10, "u": "", "p": "data:image/png;base64,@@@@", "e": 1},
        {"id": "gif", "w": 10, "h": 10, "u": "", "p": "data:image/gif;base64,R0lGOA==", "e": 1}]);
    let imp = import_lottie(&base_doc(json!([il("a", "ext"), il("b", "bad"), il("c", "gif"), il("d", "none")]), assets)).unwrap();
    assert!(imp.project.layers.is_empty());
    let w = imp.warnings.join("\n");
    assert!(w.contains("external file 'images/a.png'") && w.contains("invalid base64") && w.contains("image/gif") && w.contains("asset not found"), "{w}");
}

#[test]
fn effects_are_reported_by_name_and_controls_are_not() {
    let ef = json!([{"ty": 29, "nm": "Gaussian Blur", "en": 1}, {"ty": 25, "en": 1}, {"ty": 0, "nm": "Slider Control"}, {"ty": 21, "en": 0}]);
    let l = lay("fx", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"ef": ef}));
    let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
    let fx: Vec<&String> = imp.warnings.iter().filter(|w| w.contains("effect")).collect();
    assert_eq!(fx.len(), 2, "{:?}", imp.warnings);
    assert!(fx.iter().any(|w| w.contains("Gaussian Blur")) && fx.iter().any(|w| w.contains("Drop Shadow")));
    assert_eq!(imp.project.layers.len(), 1);
}

// ---- layer effects ----
fn efp(ty: i64, v: Value) -> Value {
    json!({"ty": ty, "nm": "p", "v": {"a": 0, "k": v}})
}
fn effect(ty: i64, params: Vec<Value>) -> Value {
    json!({"ty": ty, "nm": "e", "en": 1, "ef": params})
}
fn fx_layer(shapes: Value, ef: Vec<Value>) -> Value {
    lay("fx", 1, stat_ks([0.0, 0.0]), shapes, json!({"ef": ef}))
}
fn first_fill(imp: &Imported) -> Rgba {
    let crate::project::LayerKind::Vector { paths } = &imp.project.layers[0].kind else { panic!() };
    let Some(Paint::Solid(c)) = &paths[0].fill else { panic!() };
    *c
}
fn close(a: Rgba, r: f32, g: f32, b: f32) {
    assert!((a.r - r).abs() < 1e-3 && (a.g - g).abs() < 1e-3 && (a.b - b).abs() < 1e-3, "{a:?} vs {r} {g} {b}");
}
fn run(l: Value) -> Imported {
    import_lottie(&base_doc(json!([l]), json!([]))).unwrap()
}

#[test]
fn fill_effect_replaces_colour() {
    let e = effect(21, vec![efp(7, json!(0)), efp(7, json!(0)), efp(2, json!([0, 1, 0])), efp(7, json!(0)), efp(0, json!(0)), efp(0, json!(0)), efp(0, json!(1))]);
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![e.clone()]));
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    close(first_fill(&imp), 0.0, 1.0, 0.0);
    // half opacity (0..1 scale and 0..100 scale agree)
    for half in [0.5, 50.0] {
        let mut e2 = e.clone();
        e2["ef"][6] = efp(0, json!(half));
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![e2]))), 0.5, 0.5, 0.0);
    }
    // the engine renders the new colour
    let e = Engine::new(imp.project).unwrap();
    let (_, _, d) = crate::raster::render_rgba(&e, 0.0, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: true });
    assert!(d[1] > 250 && d[0] < 5 || d.chunks(4).any(|p| p[1] > 250 && p[0] < 5 && p[3] == 255));
}

#[test]
fn tint_maps_luminance_between_two_colours() {
    let e = effect(20, vec![efp(2, json!([0, 0, 1])), efp(2, json!([1, 1, 0])), efp(0, json!(100))]);
    // white stays on the "white" colour, black on the "black" colour
    close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 1.0, 1.0]), vec![e.clone()]))), 1.0, 1.0, 0.0);
    close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 0.0, 0.0]), vec![e.clone()]))), 0.0, 0.0, 1.0);
    // amount 0 leaves the colour alone; 50 mixes halfway
    let mut e0 = e.clone();
    e0["ef"][2] = efp(0, json!(0));
    close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![e0]))), 1.0, 0.0, 0.0);
    let mut eh = e;
    eh["ef"][2] = efp(0, json!(50));
    let l = luma(Rgba::new(1.0, 0.0, 0.0, 1.0));
    let t = Rgba::new(0.0, 0.0, 1.0, 1.0).lerp(Rgba::new(1.0, 1.0, 0.0, 1.0), l);
    close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![eh]))), 0.5 + 0.5 * t.r, 0.5 * t.g, 0.5 * t.b);
}

#[test]
fn tritone_and_levels() {
    let tri = effect(23, vec![efp(2, json!([1, 1, 1])), efp(2, json!([1, 0, 0])), efp(2, json!([0, 0, 0])), efp(0, json!(0))]);
    close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 1.0, 1.0]), vec![tri.clone()]))), 1.0, 1.0, 1.0);
    close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 0.0, 0.0]), vec![tri.clone()]))), 0.0, 0.0, 0.0);
    let g = 0.5 * 0.7152 / 0.5; // luma of (0,0.5/.7152..) == 0.5 -> midtone colour
    close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 0.5 / 0.7152, 0.0]), vec![tri.clone()]))), 1.0, 0.0, 0.0);
    let _ = g;
    // blend 100 keeps the original
    let mut keep = tri;
    keep["ef"][3] = efp(0, json!(100));
    close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.2, 0.4, 0.6]), vec![keep]))), 0.2, 0.4, 0.6);
    // levels: input 51..204 of 255 stretches 0.2..0.8 to 0..1
    let lv = effect(24, vec![efp(7, json!(1)), efp(0, json!(51)), efp(0, json!(204)), efp(0, json!(1)), efp(0, json!(0)), efp(0, json!(255))]);
    close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.5, 0.2, 0.8]), vec![lv.clone()]))), 0.5, 0.0, 1.0);
    let mut bad = lv;
    bad["ef"][2] = efp(0, json!(10));
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.5, 0.2, 0.8]), vec![bad]));
    close(first_fill(&imp), 0.5, 0.2, 0.8);
    assert!(imp.warnings.iter().any(|w| w.contains("Levels") && w.contains("invalid")), "{:?}", imp.warnings);
}

#[test]
fn effects_apply_in_order_and_reach_gradients_and_strokes() {
    let shapes = json!([{"ty": "rc", "p": {"a": 0, "k": [5, 5]}, "s": {"a": 0, "k": [4, 4]}, "r": {"a": 0, "k": 0}},
        {"ty": "gf", "t": 1, "s": {"a": 0, "k": [0, 0]}, "e": {"a": 0, "k": [10, 0]}, "g": {"p": 2, "k": {"a": 0, "k": [0, 1, 0, 0, 1, 1, 1, 1]}}, "o": {"a": 0, "k": 100}},
        {"ty": "st", "c": {"a": 0, "k": [1, 1, 1, 1]}, "o": {"a": 0, "k": 100}, "w": {"a": 0, "k": 1}}]);
    let fill = |c: [f64; 3]| effect(21, vec![efp(7, json!(0)), efp(7, json!(0)), efp(2, json!(c)), efp(7, json!(0)), efp(0, json!(0)), efp(0, json!(0)), efp(0, json!(1))]);
    let inv_levels = effect(24, vec![efp(7, json!(1)), efp(0, json!(0)), efp(0, json!(255)), efp(0, json!(1)), efp(0, json!(255)), efp(0, json!(0))]);
    let imp = run(fx_layer(shapes, vec![fill([1.0, 0.0, 0.0]), inv_levels]));
    let crate::project::LayerKind::Vector { paths } = &imp.project.layers[0].kind else { panic!() };
    // red filled, then inverted by the second effect -> cyan, also in the stroke
    let Some(Paint::Linear { stops, .. }) = &paths[0].fill else { panic!("{:?}", paths[0].fill) };
    assert_eq!(stops.len(), 2);
    close(stops[0].color, 0.0, 1.0, 1.0);
    close(stops[1].color, 0.0, 1.0, 1.0);
    let Paint::Solid(s) = &paths[0].stroke.as_ref().unwrap().paint else { panic!() };
    close(*s, 0.0, 1.0, 1.0);
}

fn anim_p(ty: i64, keys: Value) -> Value {
    json!({"ty": ty, "nm": "p", "v": {"a": 1, "k": keys}})
}
fn blur_ef(r: Value) -> Value {
    effect(29, vec![r, efp(7, json!(1)), efp(7, json!(0))])
}
fn shadow_ef(color: [f64; 3], op: Value, dir: Value, dist: Value, soft: Value) -> Value {
    effect(25, vec![efp(2, json!(color)), op, dir, dist, soft, efp(7, json!(0))])
}
fn rgba_img(imp: &Imported, frame: f32) -> crate::percep::Img {
    let e = Engine::new(imp.project.clone()).unwrap();
    let (w, h, d) = crate::raster::render_rgba(&e, frame, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: true });
    crate::percep::Img::new(w as usize, h as usize, d).unwrap()
}

#[test]
fn blur_and_shadow_become_layer_effects() {
    // direction 90 degrees = right (clockwise from up), distance 5, softness 3, opacity 127.5/255
    let sh = shadow_ef([0.0, 0.0, 0.0], efp(0, json!(127.5)), efp(0, json!(90)), efp(0, json!(5)), efp(0, json!(3)));
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![blur_ef(efp(0, json!(8))), sh]));
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    assert!(imp.post_fx.is_empty());
    let fx = &imp.project.layers[0].effects;
    assert_eq!(fx.len(), 2);
    assert_eq!(fx[0], LayerEffect::blur(8.0));
    let crate::layerfx::EffectKind::DropShadow { dx, dy, blur, color, opacity } = &fx[1].kind else { panic!() };
    assert!((dx - 5.0).abs() < 1e-4 && dy.abs() < 1e-4 && (*blur - 3.0).abs() < 1e-6 && *color == Rgba::BLACK && (opacity - 0.5).abs() < 1e-6);
    // direction 135 degrees points down and right
    let sh2 = shadow_ef([0.0, 0.0, 0.0], efp(0, json!(255)), efp(0, json!(135)), efp(0, json!(10)), efp(0, json!(0)));
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![sh2]));
    let crate::layerfx::EffectKind::DropShadow { dx, dy, .. } = &imp.project.layers[0].effects[0].kind else { panic!() };
    assert!(*dx > 7.0 && *dy > 7.0, "{dx} {dy}");
    // it renders: the halo is there
    let i = rgba_img(&imp, 0.0);
    assert!(i.px(10, 10)[3] > 0, "{:?}", i.px(10, 10));
}

fn two_layer_doc(fx_on_a: Vec<Value>) -> Imported {
    let a = lay("a", 2, stat_ks([0.0, 0.0]), rect_shapes(4.0, 14.0, 10.0, 10.0, [1.0, 0.0, 0.0]), json!({"ef": fx_on_a}));
    let b = lay("b", 1, stat_ks([0.0, 0.0]), rect_shapes(44.0, 14.0, 10.0, 10.0, [0.0, 0.0, 1.0]), json!({}));
    import_lottie(&base_doc(json!([a, b]), json!([]))).unwrap()
}

#[test]
fn effects_work_in_multi_layer_compositions_and_stay_isolated() {
    let with = two_layer_doc(vec![blur_ef(efp(0, json!(5))), shadow_ef([0.0, 0.0, 0.0], efp(0, json!(255)), efp(0, json!(135)), efp(0, json!(4)), efp(0, json!(2)))]);
    assert!(with.warnings.is_empty(), "{:?}", with.warnings);
    assert!(with.post_fx.is_empty());
    let without = two_layer_doc(vec![]);
    assert_eq!(with.project.layers.len(), 2);
    // Lottie draws the bottom of the list first: `b` is layer 0, `a` layer 1
    let (b, a) = (&with.project.layers[0], &with.project.layers[1]);
    assert!(b.effects.is_empty() && a.effects.len() == 2, "{:?}", with.project.layers.iter().map(|l| (&l.name, l.effects.len())).collect::<Vec<_>>());
    let (iw, io) = (rgba_img(&with, 0.0), rgba_img(&without, 0.0));
    assert_ne!(iw, io);
    for y in 0..iw.h {
        for x in 30..iw.w {
            assert_eq!(iw.px(x, y), io.px(x, y), "({x},{y})");
        }
    }
    assert_eq!(iw.px(44, 19), [0, 0, 255, 255]);
}

#[test]
fn animated_effect_values_become_channels() {
    // blur 0 -> 10 over 10 frames, linear; drop shadow distance 0 -> 10 at 90 degrees, opacity 255 -> 0
    let blur = blur_ef(anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [10]}])));
    let sh = shadow_ef([0.0, 0.0, 0.0], anim_p(0, json!([{"t": 0, "s": [255]}, {"t": 10, "s": [0]}])), efp(0, json!(90)), anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [10]}])), efp(0, json!(0)));
    let tint = effect(20, vec![efp(2, json!([0, 0, 0])), efp(2, json!([1, 1, 1])), anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [100]}]))]);
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![blur, sh, tint]));
    assert!(imp.warnings.iter().all(|w| !w.contains("first value")), "{:?}", imp.warnings);
    // animated tint stays a layer effect (cannot be baked), static colours of the shape are untouched
    close(first_fill(&imp), 1.0, 0.0, 0.0);
    assert_eq!(imp.project.layers[0].effects.len(), 3);
    let e = Engine::new(imp.project.clone()).unwrap();
    for (f, r, dx, op, amt) in [(0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32), (5.0, 5.0, 5.0, 0.5, 0.5), (10.0, 10.0, 10.0, 0.0, 1.0)] {
        let fx = &e.evaluate(f).nodes[0].effects;
        assert!((fx[0].get(Prop::FxBlur).unwrap() - r).abs() < 1e-3, "frame {f}");
        assert!((fx[1].get(Prop::FxDx).unwrap() - dx).abs() < 1e-3 && fx[1].get(Prop::FxDy).unwrap().abs() < 1e-3, "frame {f}: {:?}", fx[1]);
        assert!((fx[1].get(Prop::FxAmount).unwrap() - op).abs() < 1e-3);
        assert!((fx[2].get(Prop::FxAmount).unwrap() - amt).abs() < 1e-3);
    }
    assert_ne!(rgba_img(&imp, 0.0), rgba_img(&imp, 8.0));
}

#[test]
fn hold_and_bezier_easing_carry_over_to_effect_channels() {
    let hold = blur_ef(anim_p(0, json!([{"t": 0, "s": [2], "h": 1}, {"t": 10, "s": [20]}])));
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![hold]));
    let e = Engine::new(imp.project).unwrap();
    assert_eq!(e.evaluate(9.0).nodes[0].effects[0].get(Prop::FxBlur), Some(2.0));
    assert_eq!(e.evaluate(10.0).nodes[0].effects[0].get(Prop::FxBlur), Some(20.0));
    let ease = blur_ef(anim_p(0, json!([{"t": 0, "s": [0], "o": {"x": [0.9], "y": [0.0]}, "i": {"x": [0.9], "y": [0.0]}}, {"t": 10, "s": [20]}])));
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![ease]));
    let e = Engine::new(imp.project).unwrap();
    let mid = e.evaluate(5.0).nodes[0].effects[0].get(Prop::FxBlur).unwrap();
    assert!(mid < 5.0, "ease-in stays low at the middle: {mid}");
}

#[test]
fn shadow_direction_animation_follows_the_arc() {
    // direction 0 -> 180 degrees at distance 10: halfway the shadow points right (dx = 10), not at the straight-line midpoint (dx = 0)
    let sh = shadow_ef([0.0, 0.0, 0.0], efp(0, json!(255)), anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [180]}])), efp(0, json!(10)), efp(0, json!(0)));
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![sh]));
    let e = Engine::new(imp.project).unwrap();
    let fx = &e.evaluate(5.0).nodes[0].effects[0];
    assert!((fx.get(Prop::FxDx).unwrap() - 10.0).abs() < 0.2 && fx.get(Prop::FxDy).unwrap().abs() < 0.2, "{fx:?}");
    let fx = &e.evaluate(0.0).nodes[0].effects[0];
    assert!(fx.get(Prop::FxDx).unwrap().abs() < 1e-3 && (fx.get(Prop::FxDy).unwrap() + 10.0).abs() < 1e-3);
}

#[test]
fn colour_effects_reach_precomp_and_image_layers() {
    let green = effect(21, vec![efp(7, json!(0)), efp(7, json!(0)), efp(2, json!([0, 1, 0])), efp(7, json!(0)), efp(0, json!(0)), efp(0, json!(0)), efp(0, json!(1))]);
    let pre = json!({"ty": 0, "nm": "pc", "refId": "c", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]), "ef": [green.clone()]});
    let assets = json!([{"id": "c", "layers": [lay("in", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({}))]}]);
    let imp = import_lottie(&base_doc(json!([pre]), assets.clone())).unwrap();
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    assert_eq!(imp.project.layers[0].effects, vec![LayerEffect::fill(Rgba::new(0.0, 1.0, 0.0, 1.0), 1.0)]);
    let i = rgba_img(&imp, 0.0);
    assert_eq!(i.px(2, 2), [0, 255, 0, 255]);
    // tint on a precomp, with animated amount
    let tint = effect(20, vec![efp(2, json!([0, 0, 1])), efp(2, json!([1, 1, 0])), anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [100]}]))]);
    let pre = json!({"ty": 0, "nm": "pc", "refId": "c", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]), "ef": [tint]});
    let imp = import_lottie(&base_doc(json!([pre]), assets.clone())).unwrap();
    assert_eq!(rgba_img(&imp, 0.0).px(2, 2), [255, 0, 0, 255]);
    let end = rgba_img(&imp, 10.0).px(2, 2);
    assert!(end[0].abs_diff(54) <= 1 && end[1].abs_diff(54) <= 1 && end[2].abs_diff(201) <= 1, "red has luminance .21 -> between blue and yellow: {end:?}");
    // an image layer
    let img = json!({"ty": 2, "nm": "im", "refId": "i", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]), "ef": [green.clone()]});
    let assets = json!([{"id": "i", "w": 2, "h": 2, "p": png_uri([255, 0, 0, 255]), "e": 1}]);
    let imp = import_lottie(&base_doc(json!([img]), assets)).unwrap();
    assert_eq!(imp.project.layers[0].effects.len(), 1, "{:?}", imp.warnings);
    let p = rgba_img(&imp, 0.0).px(0, 0);
    assert!(p[1] > 250 && p[0] < 5, "{p:?}");
    // tritone / levels have no layer form: named, not applied
    let tri = effect(23, vec![efp(2, json!([1, 1, 1])), efp(2, json!([0.5, 0.5, 0.5])), efp(2, json!([0, 0, 0])), efp(0, json!(0))]);
    let pre = json!({"ty": 0, "nm": "pc", "refId": "c", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]), "ef": [tri]});
    let assets = json!([{"id": "c", "layers": [lay("in", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({}))]}]);
    let imp = import_lottie(&base_doc(json!([pre]), assets)).unwrap();
    assert!(imp.project.layers[0].effects.is_empty());
    assert!(imp.warnings.iter().any(|w| w.contains("Tritone") && w.contains("shape layers")), "{:?}", imp.warnings);
}

#[test]
fn effects_inside_precomps_use_prefixed_channel_targets() {
    let blur = blur_ef(anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [6]}])));
    let inner = lay("in", 1, stat_ks([0.0, 0.0]), rect_shapes(10.0, 10.0, 10.0, 10.0, [1.0, 0.0, 0.0]), json!({"ef": [blur]}));
    let pre = json!({"ty": 0, "nm": "pc", "refId": "c", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0])});
    let imp = import_lottie(&base_doc(json!([pre]), json!([{"id": "c", "layers": [inner]}]))).unwrap();
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    assert!(imp.project.channels.iter().any(|c| c.prop == Prop::FxBlur && c.target.ends_with("/in::fx0")), "{:?}", imp.project.channels.iter().map(|c| &c.target).collect::<Vec<_>>());
    let e = Engine::new(imp.project.clone()).unwrap();
    let n = e.evaluate(10.0).nodes.iter().find(|n| n.name.ends_with("/in")).unwrap().effects.clone();
    assert_eq!(n[0].get(Prop::FxBlur), Some(6.0));
    // the precomp group itself is not blurred
    assert!(e.evaluate(10.0).nodes[0].effects.is_empty());
}

#[test]
fn effects_survive_masks_and_two_effect_orders() {
    let mk = || vec![mask("a", false, 100.0, 5.0, 5.0, 15.0, 15.0)];
    let l = lay("m", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 30.0, 30.0, [1.0, 0.0, 0.0]), json!({"masksProperties": mk(), "ef": [blur_ef(efp(0, json!(4)))]}));
    let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
    // the effect sits on the outer (masked) layer, after the mask
    let top = &imp.project.layers.last().unwrap();
    assert_eq!(top.effects.len(), 1, "{:?}", imp.warnings);
    Engine::new(imp.project).unwrap();
}

#[test]
fn disabled_unsupported_and_malformed_effects_never_crash() {
    let green = effect(21, vec![efp(7, json!(0)), efp(7, json!(0)), efp(2, json!([0, 1, 0]))]);
    let mut off = green.clone();
    off["en"] = json!(0);
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![off]));
    close(first_fill(&imp), 1.0, 0.0, 0.0);
    assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
    // disabled blur / shadow / tint produce no effects at all
    let mut offs = vec![blur_ef(efp(0, json!(5))), shadow_ef([0.0; 3], efp(0, json!(255)), efp(0, json!(0)), efp(0, json!(3)), efp(0, json!(1))), effect(20, vec![efp(2, json!([0, 0, 0])), efp(2, json!([1, 1, 1])), efp(0, json!(100))])];
    offs.iter_mut().for_each(|e| e["en"] = json!(0));
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), offs));
    assert!(imp.project.layers[0].effects.is_empty() && imp.warnings.is_empty(), "{:?}", imp.warnings);
    // unsupported ones are named
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![effect(30, vec![]), json!({"ty": 99, "nm": "Weird Glow", "en": 1}), effect(26, vec![])]));
    let w = imp.warnings.join("\n");
    assert!(w.contains("Twirl") && w.contains("Weird Glow") && w.contains("Radial Wipe"), "{w}");
    // one-dimensional blur is not supported and says so
    let hb = effect(29, vec![efp(0, json!(8)), efp(7, json!(2))]);
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![hb]));
    assert!(imp.project.layers[0].effects.is_empty() && imp.warnings.iter().any(|w| w.contains("both dimensions")), "{:?}", imp.warnings);
    // malformed shapes of the ef array and of its values
    for bad in [json!("x"), json!(5), json!({"a": 1}), json!([1, "a", null, [], {"ty": "z"}]),
                json!([{"ty": 21, "en": 1, "ef": "no"}, {"ty": 21, "en": 1, "ef": [null, {"v": 3}, {"v": {"a": 0, "k": "red"}}]}]),
                json!([{"ty": 29, "en": 1, "ef": [{"v": {"a": 0, "k": [1e300]}}]}, {"ty": 25, "ef": [{"v": {"a": 1, "k": []}}]}]),
                json!([{"ty": 29, "en": 1, "ef": [{"v": {"a": 0, "k": -5}}]}, {"ty": 20, "en": 1, "ef": [{"v": {"a": 0, "k": [1]}}]}]),
                json!([{"ty": 29, "en": 1, "ef": [{"v": {"a": 1, "k": [{"t": 0}, {"t": "x", "s": [1]}, 7]}}]}]),
                json!([{"ty": 25, "en": 1, "ef": [{"v": {"a": 0, "k": [0, 0, 0]}}, {"v": {"a": 0, "k": 255}}, {"v": {"a": 1, "k": [{"t": 0, "s": [1e30]}, {"t": 1e30, "s": [5]}]}}, {"v": {"a": 0, "k": 3}}, {"v": {"a": 0, "k": 1}}]}]),
                json!((0..200).map(|_| blur_ef(efp(0, json!(1)))).collect::<Vec<_>>())] {
        let l = lay("m", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"ef": bad}));
        let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
        assert_eq!(imp.project.layers.len(), 1);
        close(first_fill(&imp), 1.0, 0.0, 0.0);
        Engine::new(imp.project.clone()).expect("whatever was imported must be a valid project");
        assert!(imp.project.layers[0].effects.len() <= crate::layerfx::MAX_EFFECTS);
    }
}

#[test]
fn animated_colours_are_reported_as_first_value() {
    let anim = json!({"ty": 2, "nm": "c", "v": {"a": 1, "k": [{"t": 0, "s": [1, 0, 0]}, {"t": 10, "s": [0, 0, 1]}]}});
    let e = effect(20, vec![anim, efp(2, json!([1, 1, 1])), efp(0, json!(100))]);
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 0.0, 0.0]), vec![e]));
    assert!(imp.warnings.iter().any(|w| w.contains("Tint") && w.contains("first value")), "{:?}", imp.warnings);
    close(first_fill(&imp), 1.0, 0.0, 0.0);
}

#[test]
fn static_colour_effects_stay_baked_on_shape_layers_but_follow_blur_order() {
    // alone: baked (no layer effect); next to a blur: kept as layer effects in `ef` order
    let tint = || effect(20, vec![efp(2, json!([0, 0, 1])), efp(2, json!([1, 1, 0])), efp(0, json!(100))]);
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 1.0, 1.0]), vec![tint()]));
    assert!(imp.project.layers[0].effects.is_empty());
    let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 1.0, 1.0]), vec![tint(), blur_ef(efp(0, json!(3)))]));
    close(first_fill(&imp), 1.0, 1.0, 1.0);
    let k: Vec<_> = imp.project.layers[0].effects.iter().map(|e| std::mem::discriminant(&e.kind)).collect();
    assert_eq!(k, vec![std::mem::discriminant(&LayerEffect::tint(Rgba::BLACK, Rgba::WHITE, 1.0).kind), std::mem::discriminant(&LayerEffect::blur(1.0).kind)]);
}

// ---- exporter round trip ----
fn effects_at(e: &Engine, frame: f32, name: &str) -> Vec<LayerEffect> {
    e.evaluate(frame).nodes.iter().find(|n| n.name == name).map(|n| n.effects.clone()).unwrap_or_default()
}

fn near(a: &[LayerEffect], b: &[LayerEffect]) -> bool {
    use crate::layerfx::EffectKind as K;
    let f = |x: f32, y: f32| (x - y).abs() < 0.02;
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| match (&x.kind, &y.kind) {
        (K::Blur { radius: p }, K::Blur { radius: q }) => f(*p, *q),
        (K::DropShadow { dx, dy, blur, color, opacity }, K::DropShadow { dx: dx2, dy: dy2, blur: b2, color: c2, opacity: o2 }) => f(*dx, *dx2) && f(*dy, *dy2) && f(*blur, *b2) && f(*opacity, *o2) && color == c2,
        (K::Tint { black, white, amount }, K::Tint { black: b2, white: w2, amount: a2 }) => black == b2 && white == w2 && f(*amount, *a2),
        (K::Fill { color, opacity }, K::Fill { color: c2, opacity: o2 }) => color == c2 && f(*opacity, *o2),
        _ => false,
    })
}

fn render_img(e: &Engine, frame: f32) -> crate::percep::Img {
    let (w, h, d) = crate::raster::render_rgba(e, frame, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: false });
    crate::percep::Img::new(w as usize, h as usize, d).unwrap()
}

#[test]
fn exporter_writes_layer_effects_and_reimport_gives_them_back() {
    use crate::project::Channel;
    let mut p = Project::new("rt", 60.0, 40.0, 10.0, 11);
    let mut a = Layer::vector("a", vec![crate::path::VPath::rect("r", 10.0, 10.0, 20.0, 15.0).with_fill("#ff0000")]);
    a.effects = vec![
        LayerEffect::blur(3.0),
        LayerEffect::drop_shadow(4.0, -3.0, 2.0, Rgba::new(0.0, 0.0, 0.5, 1.0), 0.6),
        LayerEffect::tint(Rgba::new(0.0, 0.0, 0.2, 1.0), Rgba::new(1.0, 1.0, 0.5, 1.0), 0.5),
    ];
    let mut b = Layer::vector("b", vec![crate::path::VPath::rect("r", 40.0, 10.0, 10.0, 10.0).with_fill("#00ff00")]);
    b.effects = vec![LayerEffect::fill(Rgba::new(0.0, 0.0, 1.0, 1.0), 0.5)];
    let mut off = LayerEffect::blur(9.0);
    off.enabled = false;
    b.effects.push(off);
    p.layers = vec![a, b];
    // animated: blur radius and shadow offset
    p.channels = vec![
        Channel::keys("a::fx0", Prop::FxBlur, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 6.0, Ease::Linear)])),
        Channel::keys("a::fx1", Prop::FxDx, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 8.0, Ease::Linear)])),
        Channel::keys("a::fx2", Prop::FxAmount, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 1.0, Ease::Linear)])),
    ];
    let orig = Engine::new(p).unwrap();
    let json = crate::export::lottie_json(&orig).to_string();
    let back = import_lottie(&json).unwrap();
    assert!(back.warnings.iter().all(|w| !w.contains("effect")), "{:?}", back.warnings);
    let re = Engine::new(back.project.clone()).unwrap();
    // the disabled effect is not written
    assert!(!json.contains("\"en\":0"));
    for f in 0..11 {
        let f = f as f32;
        // layer "b" has a plain colour effect: on a shape layer with geometry the importer bakes it into the fill
        // (its picture is compared below), "a" keeps its effects as effects
        let (x, y) = (effects_at(&orig, f, "a"), effects_at(&re, f, "a"));
        assert!(near(&x, &y), "a frame {f}: {x:?} vs {y:?}");
    }
    for f in [0.0, 5.0, 10.0] {
        let (ia, ib) = (render_img(&orig, f), render_img(&re, f));
        let d = crate::percep::mse(&ia, &ib).unwrap();
        assert!(d < 40.0, "frame {f}: re-imported picture differs (mse {d})");
    }
    // export is deterministic
    assert_eq!(json, crate::export::lottie_json(&orig).to_string());
}

#[test]
fn exporter_moves_single_leaf_group_effects_and_skips_multi_leaf_groups() {
    let mut one = Layer::group("g1", vec![Layer::vector("a", vec![crate::path::VPath::rect("r", 5.0, 5.0, 10.0, 10.0).with_fill("#f00")])]);
    one.effects = vec![LayerEffect::blur(2.0)];
    let mut two = Layer::group("g2", vec![Layer::vector("b", vec![crate::path::VPath::rect("r", 30.0, 5.0, 10.0, 10.0).with_fill("#00f")]), Layer::vector("c", vec![crate::path::VPath::rect("r", 45.0, 5.0, 10.0, 10.0).with_fill("#0f0")])]);
    two.effects = vec![LayerEffect::blur(2.0)];
    let mut p = Project::new("g", 60.0, 40.0, 10.0, 2);
    p.layers = vec![one, two];
    let j = crate::export::lottie_json(&Engine::new(p).unwrap());
    let layers = j["layers"].as_array().unwrap();
    let with_ef: Vec<&str> = layers.iter().filter(|l| l.get("ef").is_some()).map(|l| l["nm"].as_str().unwrap()).collect();
    assert_eq!(with_ef, vec!["g1/a"], "{with_ef:?}");
}

#[test]
fn exporter_without_effects_writes_no_ef() {
    let e = Engine::new(crate::demo::project()).unwrap();
    assert!(!crate::export::lottie_json(&e).to_string().contains("\"ef\""));
}

// ---- animated gradients (wave 4, M4) ---------------------------------------------------------

fn gf_json(t: i64, s: Value, e: Value) -> Value {
    json!({"ty": "gf", "t": t, "s": s, "e": e, "o": {"a": 0, "k": 100}, "g": {"p": 2, "k": {"a": 0, "k": [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]}}})
}

fn fill_of(e: &Engine, f: f32) -> Paint {
    let fr = e.evaluate(f);
    fr.nodes.iter().find_map(|n| n.paths.first().and_then(|p| p.fill.clone())).unwrap()
}

/// The centre and the radius of an animated radial gradient follow the Lottie keys on every frame; they used to
/// stay at the first key (a warning said so).
#[test]
fn animated_radial_gradient_geometry_is_imported() {
    let s = json!({"a": 1, "k": [{"t": 0, "s": [10.0, 10.0], "i": {"x": [1], "y": [1]}, "o": {"x": [0], "y": [0]}}, {"t": 10, "s": [30.0, 20.0]}]});
    let e = json!({"a": 1, "k": [{"t": 0, "s": [20.0, 10.0], "i": {"x": [1], "y": [1]}, "o": {"x": [0], "y": [0]}}, {"t": 10, "s": [30.0, 50.0]}]});
    let shapes = json!([{"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [40, 40]}, "r": {"a": 0, "k": 0}}, gf_json(2, s, e)]);
    let r = run(lay("g", 1, stat_ks([0.0, 0.0]), shapes, json!({})));
    assert!(!r.warnings.iter().any(|w| w.contains("gradient")), "{:?}", r.warnings);
    let en = Engine::new(r.project).unwrap();
    for (f, c, rad) in [(0.0f32, (10.0f32, 10.0f32), 10.0f32), (5.0, (20.0, 15.0), (5.0f32 * 5.0 + 15.0 * 15.0).sqrt()), (10.0, (30.0, 20.0), 30.0)] {
        match fill_of(&en, f) {
            Paint::Radial { center, radius, .. } => {
                assert!((center.x - c.0).abs() < 1e-3 && (center.y - c.1).abs() < 1e-3, "frame {f}: centre {center:?}");
                assert!((radius - rad).abs() < 0.05, "frame {f}: radius {radius} vs {rad}");
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn animated_linear_gradient_end_points_and_group_transforms() {
    let s = json!({"a": 0, "k": [0.0, 0.0]});
    let e = json!({"a": 1, "k": [{"t": 0, "s": [10.0, 0.0], "i": {"x": [1], "y": [1]}, "o": {"x": [0], "y": [0]}}, {"t": 10, "s": [10.0, 20.0]}]});
    // the group moves by (100, 0) and scales by 2: gradient points live in the group's space
    let tr = json!({"ty": "tr", "p": {"a": 0, "k": [100, 0]}, "a": {"a": 0, "k": [0, 0]}, "s": {"a": 0, "k": [200, 200]}, "r": {"a": 0, "k": 0}, "o": {"a": 0, "k": 100}});
    let shapes = json!([{"ty": "gr", "it": [{"ty": "rc", "p": {"a": 0, "k": [5, 5]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}, gf_json(1, s, e), tr]}]);
    let r = run(lay("g", 1, stat_ks([0.0, 0.0]), shapes, json!({})));
    let en = Engine::new(r.project).unwrap();
    for (f, to) in [(0.0f32, (120.0f32, 0.0f32)), (10.0, (120.0, 40.0))] {
        match fill_of(&en, f) {
            Paint::Linear { from, to: t, .. } => {
                assert!((from.x - 100.0).abs() < 1e-3 && from.y.abs() < 1e-3, "{from:?}");
                assert!((t.x - to.0).abs() < 1e-3 && (t.y - to.1).abs() < 1e-3, "frame {f}: {t:?}");
            }
            other => panic!("{other:?}"),
        }
    }
}

/// Osatura -> Lottie -> Osatura: a path whose radial gradient moves (and grows) keeps looking the same on every
/// frame. With the gradient frozen at the first key (the old import) the pictures differ by a lot.
#[test]
fn animated_gradient_survives_the_round_trip() {
    use crate::project::Channel;
    let mut p = Project::new("grad", 80.0, 60.0, 12.0, 13);
    p.background = Some(Rgba::hex("#ffffff"));
    let path = crate::path::VPath::rect("blob", 5.0, 5.0, 70.0, 50.0).with_paint(Paint::radial(Vec2::new(20.0, 30.0), 12.0, &[(0.0, "#ff0000"), (1.0, "#0000ff")]));
    p.layers = vec![Layer::vector("blush", vec![path])];
    p.channels = vec![
        Channel::keys("blush::blob::fill.from", Prop::GradDx, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, 40.0, Ease::Linear)])),
        Channel::keys("blush::blob::fill.from", Prop::GradDy, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, -10.0, Ease::Linear)])),
        Channel::keys("blush::blob::fill.to", Prop::GradDx, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, 18.0, Ease::Linear)])),
    ];
    let e = Engine::new(p.clone()).unwrap();
    let j = crate::export::lottie_json(&e).to_string();
    let back = import_lottie(&j).unwrap();
    assert!(!back.warnings.iter().any(|w| w.contains("gradient")), "{:?}", back.warnings);
    assert!(back.project.channels.iter().any(|c| c.prop == Prop::GradDx), "gradient channels were created");
    let re = Engine::new(back.project.clone()).unwrap();
    let mut frozen = back.project;
    frozen.channels.retain(|c| !matches!(c.prop, Prop::GradDx | Prop::GradDy));
    let fe = Engine::new(frozen).unwrap();
    let mut worst_frozen = 0.0f64;
    for f in 0..=12 {
        let f = f as f32;
        let (a, b, c) = (render_img(&e, f), render_img(&re, f), render_img(&fe, f));
        let mse = crate::percep::mse(&a, &b).unwrap();
        assert!(mse < 4.0, "frame {f}: mse {mse}");
        worst_frozen = worst_frozen.max(crate::percep::mse(&a, &c).unwrap());
    }
    assert!(worst_frozen > 400.0, "the test must be able to tell a frozen gradient from a moving one ({worst_frozen})");
}

/// The reviewer's case: the demo's bone-driven radial "blush" must be where it was on every frame after a
/// round trip (it used to vanish on ~40 of 48 frames because the gradient stayed at the first key). The gradient is
/// compared in stage coordinates, frame by frame.
#[test]
fn demo_round_trip_keeps_the_blush_on_every_frame() {
    let e = Engine::new(crate::demo::project()).unwrap();
    let back = import_lottie(&crate::export::lottie_json(&e).to_string()).unwrap();
    let re = Engine::new(back.project).unwrap();
    let stage = |en: &Engine, f: f32| -> Option<(Vec2, f32)> {
        let fr = en.evaluate(f);
        fr.nodes.iter().filter(|n| n.name.ends_with("blush")).find_map(|n| {
            n.paths.first().and_then(|p| match &p.fill {
                Some(Paint::Radial { center, radius, .. }) => Some((n.world.apply(*center), radius * (n.world.a * n.world.d - n.world.b * n.world.c).abs().sqrt())),
                _ => None,
            })
        })
    };
    let mut moved = 0.0f32;
    let first = stage(&e, 0.0).expect("blush in the original");
    for f in 0..e.project.frames {
        let (a, b) = (stage(&e, f as f32).unwrap(), stage(&re, f as f32).expect("blush after the round trip"));
        assert!(a.0.distance(b.0) < 1.0, "frame {f}: centre {:?} vs {:?}", a.0, b.0);
        assert!((a.1 - b.1).abs() < 0.1 * a.1.max(1.0), "frame {f}: radius {} vs {}", a.1, b.1);
        moved = moved.max(a.0.distance(first.0));
    }
    assert!(moved > 5.0, "the blush moves in the demo ({moved}), so this test can tell a frozen gradient");
}

/// One shape layer holding `n` identical "sh" items in a group, all with the default name.
fn many_shapes_doc(n: usize) -> String {
    let sh = json!({"ty": "sh", "ks": {"a": 0, "k": rect_shape_json(0.0, 0.0, 4.0, 4.0)}});
    let mut items: Vec<Value> = (0..n).map(|_| sh.clone()).collect();
    items.push(json!({"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}}));
    base_doc(json!([lay("many", 1, stat_ks([0.0, 0.0]), json!([{"ty": "gr", "it": items}]), json!({}))]), json!([])).to_string()
}

/// Importing grows about linearly with the number of shapes (the duplicate-name search used to be quadratic: 2000
/// shapes took 1.1 s, 4000 took 5.6 s).
#[test]
fn importing_thousands_of_shapes_is_not_quadratic() {
    let time = |n: usize| {
        let doc = many_shapes_doc(n);
        let t = crate::clock::Stamp::now();
        let imp = import_lottie(&doc).unwrap();
        let el = t.elapsed().as_secs_f64();
        let count = match &imp.project.layers[0].kind {
            crate::project::LayerKind::Vector { paths } => paths.len(),
            other => panic!("{other:?}"),
        };
        assert_eq!(count, n);
        el
    };
    let (a, b) = (time(1500), time(6000));
    eprintln!("1500 shapes {a:.3} s, 6000 shapes {b:.3} s, ratio {:.1}", b / a);
    // linear would be 4x; quadratic 16x. The margin leaves room for timing noise.
    assert!(b < 10.0 * a.max(0.02), "1500 shapes took {a:.3} s, 6000 took {b:.3} s");
    // unique, stable names
    let imp = import_lottie(&many_shapes_doc(5)).unwrap();
    if let crate::project::LayerKind::Vector { paths } = &imp.project.layers[0].kind {
        let names: Vec<&str> = paths.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["path", "path_2", "path_3", "path_4", "path_5"]);
    }
}
