// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! End-to-end tests of `texture` and `texture-add`: the files written, the exact bytes for the same options, integer seeds,
//! refusals with a message, and the project that `texture-add` produces.

use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_osatura");

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("osatura-texture-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(args: &[&str]) -> (bool, String) {
    let o = Command::new(BIN).args(args).output().unwrap();
    (o.status.success(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn s(p: &std::path::Path) -> &str {
    p.to_str().unwrap()
}

fn png_size(b: &[u8]) -> (u32, u32) {
    assert_eq!(&b[..8], &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a], "not a PNG");
    (u32::from_be_bytes(b[16..20].try_into().unwrap()), u32::from_be_bytes(b[20..24].try_into().unwrap()))
}

#[test]
fn texture_writes_every_requested_map_and_is_repeatable() {
    let d = dir("maps");
    let (a, n, al, h) = (d.join("a.png"), d.join("n.png"), d.join("al.png"), d.join("h.png"));
    let (ok, out) = run(&["texture", "granite", s(&a), "--size", "64", "--normal-out", s(&n), "--albedo-out", s(&al), "--height-out", s(&h)]);
    assert!(ok, "{out}");
    for f in [&a, &n, &al, &h] {
        assert_eq!(png_size(&std::fs::read(f).unwrap()), (64, 64), "{f:?}");
    }
    let b = d.join("b.png");
    assert!(run(&["texture", "granite", s(&b), "--size", "64"]).0);
    assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap(), "same options, same image");
}

#[test]
fn large_seeds_are_exact_and_list_names_every_preset() {
    let d = dir("seed");
    let (x, y) = (d.join("x.png"), d.join("y.png"));
    assert!(run(&["texture", "rock", s(&x), "--size", "32", "--seed", "16777216"]).0);
    assert!(run(&["texture", "rock", s(&y), "--size", "32", "--seed", "16777217"]).0);
    assert_ne!(std::fs::read(&x).unwrap(), std::fs::read(&y).unwrap(), "seed 2^24 + 1 must not round to 2^24");
    let (ok, list) = run(&["texture", "list"]);
    assert!(ok);
    for p in ["rock", "lava", "marble", "mandelbrot", "burning-ship", "crag (kind)"] {
        assert!(list.contains(p), "{p} missing from: {list}");
    }
}

#[test]
fn bad_requests_are_refused_with_a_reason_and_write_nothing() {
    let d = dir("bad");
    let o = d.join("o.png");
    for (args, needle) in [
        (vec!["texture", "nope", s(&o)], "unknown texture"),
        (vec!["texture", "rock", s(&o), "--size", "5000"], "size must be"),
        (vec!["texture", "rock", s(&o), "--size", "abc"], "whole number"),
        (vec!["texture", "rock", s(&o), "--seed", "-1"], "whole number"),
        (vec!["texture", "rock", s(&o), "--size", "64", "--normal-out", s(&o)], "named twice"),
        (vec!["texture", "mandelbrot", s(&o), "--size", "4096", "--ssaa", "2", "--zoom", "1000000000000"], "too many iterations"),
        (vec!["texture", "rock", s(&o), "--size", "64", "--relief", "nan"], "finite"),
    ] {
        let (ok, out) = run(&args);
        assert!(!ok && out.contains(needle), "{args:?} -> {out}");
        assert!(!o.exists(), "{args:?} left a file behind");
    }
}

#[test]
fn texture_add_embeds_an_image_layer_with_its_normal_map() {
    let d = dir("add");
    let (p, q) = (d.join("p.json"), d.join("q.json"));
    assert!(run(&["demo", s(&p)]).0);
    let (ok, out) = run(&["texture-add", s(&p), s(&q), "backdrop", "marble", "--size", "64", "--x", "10", "--y", "20"]);
    assert!(ok, "{out}");
    let (ok, out) = run(&["check", s(&q)]);
    assert!(ok, "{out}");
    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&q).unwrap()).unwrap();
    let layers = j["layers"].as_array().unwrap();
    assert_eq!(layers[0]["name"], "backdrop", "behind everything by default");
    assert!(layers[0]["normal_map"].is_object(), "the normal map travels with the layer");
    // in front, without a normal map
    let r = d.join("r.json");
    let (ok, out) = run(&["texture-add", s(&q), s(&r), "overlay", "ice", "--size", "64", "--front", "--no-normal"]);
    assert!(ok, "{out}");
    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&r).unwrap()).unwrap();
    let last = j["layers"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["name"], "overlay");
    assert!(last["normal_map"].is_null());
    // a name that exists is refused and the output is not written
    let t = d.join("t.json");
    let (ok, out) = run(&["texture-add", s(&r), s(&t), "overlay", "rock", "--size", "64"]);
    assert!(!ok && out.contains("already exists") && !t.exists(), "{out}");
}

#[test]
fn the_texture_effect_changes_the_render_and_unknown_presets_fail_check() {
    let d = dir("fx");
    let p = d.join("p.json");
    assert!(run(&["demo", s(&p)]).0);
    let mut j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let (plain, textured, bad) = (d.join("plain.png"), d.join("textured.png"), d.join("bad.json"));
    assert!(run(&["render", s(&p), "0", s(&plain)]).0);
    j["layers"][1]["effects"] = serde_json::json!([{"type": "texture", "preset": "granite", "tile": 96, "amount": 1}]);
    let q = d.join("q.json");
    std::fs::write(&q, j.to_string()).unwrap();
    let (ok, out) = run(&["check", s(&q)]);
    assert!(ok, "{out}");
    assert!(run(&["render", s(&q), "0", s(&textured)]).0);
    assert_ne!(std::fs::read(&plain).unwrap(), std::fs::read(&textured).unwrap());
    j["layers"][1]["effects"] = serde_json::json!([{"type": "texture", "preset": "julia"}]);
    std::fs::write(&bad, j.to_string()).unwrap();
    assert!(!run(&["check", s(&bad)]).0, "an escape-time preset cannot tile, so the effect refuses it");
}
