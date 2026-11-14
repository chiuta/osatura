// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! CLI regression tests for the reviewer findings: dof vs render, dof-export video, lod with containers, mapping-file
//! typos, strict flags, sound validation, collab-sim default mode, lottie round trip. Each fails on the old behaviour.

use std::fs;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_osatura");

fn run(args: &[&str]) -> (bool, String) {
    let o = Command::new(BIN)
        .args(args)
        .output()
        .expect("cannot start osatura");
    (
        o.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        ),
    )
}

fn demo(dir: &std::path::Path) -> String {
    let p = dir.join("demo.json").to_string_lossy().into_owned();
    let (ok, t) = run(&["demo", &p]);
    assert!(ok, "{t}");
    p
}

#[test]
fn dof_with_zero_strength_equals_render() {
    let s = osatura_core::fsutil::ScratchDir::new("osatura-fix-dof");
    let p = demo(&s);
    let (a, b) = (
        s.join("a.png").to_string_lossy().into_owned(),
        s.join("b.png").to_string_lossy().into_owned(),
    );
    assert!(run(&["render", &p, "10", &a]).0);
    let (ok, t) = run(&["dof", &p, "10", &b, "--strength", "0"]);
    assert!(ok, "{t}");
    let (ia, ib) = (image_px(&a), image_px(&b));
    assert_eq!(ia.0, ib.0);
    let worst =
        ia.1.iter()
            .zip(&ib.1)
            .map(|(x, y)| (*x as i32 - *y as i32).abs())
            .max()
            .unwrap();
    assert!(worst <= 1, "dof(strength 0) differs from render by {worst}");
}

fn image_px(path: &str) -> ((u32, u32), Vec<u8>) {
    png_decode(path)
}

fn png_decode(path: &str) -> ((u32, u32), Vec<u8>) {
    let f = fs::File::open(path).unwrap();
    let dec = png::Decoder::new(std::io::BufReader::new(f));
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let i = r.next_frame(&mut buf).unwrap();
    buf.truncate(i.buffer_size());
    ((i.width, i.height), buf)
}

#[test]
fn dof_export_video_is_real_or_an_error() {
    let s = osatura_core::fsutil::ScratchDir::new("osatura-fix-dofv");
    let p = demo(&s);
    let out = s.join("o.mp4").to_string_lossy().into_owned();
    let (ok, t) = run(&[
        "dof-export",
        &p,
        &out,
        "--from",
        "0",
        "--to",
        "5",
        "--scale",
        "0.5",
    ]);
    if ok {
        let len = fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
        assert!(len > 100, "success reported but {out} is {len} bytes: {t}");
        let head = fs::read(&out).unwrap();
        assert!(
            head.len() > 12 && &head[4..8] == b"ftyp",
            "not an MP4 container"
        );
    } else {
        assert!(
            !std::path::Path::new(&out).exists() || fs::metadata(&out).unwrap().len() == 0,
            "{t}"
        );
        assert!(!t.to_lowercase().contains("wrote"), "{t}");
    }
}

#[test]
fn unknown_flags_are_refused_with_a_hint() {
    let s = osatura_core::fsutil::ScratchDir::new("osatura-fix-flags");
    let p = demo(&s);
    let out = s.join("o.png").to_string_lossy().into_owned();
    let (ok, t) = run(&["render", &p, "0", &out, "--bone"]);
    assert!(!ok && t.contains("did you mean --bones"), "{t}");
}

#[test]
fn mapping_file_typos_are_errors_and_empty_results_fail() {
    let s = osatura_core::fsutil::ScratchDir::new("osatura-fix-map");
    let p = demo(&s);
    let ev = s.join("ev.json");
    fs::write(&ev, r#"{"events":[]}"#).unwrap();
    let map = s.join("m.json");
    fs::write(
        &map,
        r#"{"params":[{"sorce":"mouse","id":"x"}],"triggers":[]}"#,
    )
    .unwrap();
    let out = s.join("o.json").to_string_lossy().into_owned();
    let (ok, t) = run(&[
        "live-map",
        &p,
        ev.to_str().unwrap(),
        map.to_str().unwrap(),
        &out,
    ]);
    assert!(!ok, "typo accepted: {t}");
    assert!(
        !std::path::Path::new(&out).exists(),
        "output written despite error"
    );
}

#[test]
fn sound_add_rejects_a_missing_file() {
    let s = osatura_core::fsutil::ScratchDir::new("osatura-fix-snd");
    let p = demo(&s);
    let out = s.join("o.json").to_string_lossy().into_owned();
    let (ok, t) = run(&[
        "sound",
        "add",
        &p,
        &out,
        "boom",
        &s.join("nope.wav").to_string_lossy(),
    ]);
    assert!(!ok, "{t}");
    assert!(!std::path::Path::new(&out).exists());
}

#[test]
fn collab_sim_default_mode_is_project_aware() {
    let s = osatura_core::fsutil::ScratchDir::new("osatura-fix-collab");
    let p = demo(&s);
    let (_, t) = run(&[
        "collab-sim",
        &p,
        "--clients",
        "3",
        "--ops",
        "30",
        "--seed",
        "1",
        "--json",
    ]);
    let v: serde_json::Value =
        serde_json::from_str(t.trim()).unwrap_or_else(|e| panic!("{e}: {t}"));
    assert_eq!(v["ok"], true, "{t}");
}

#[test]
fn lod_survives_container_members_and_lottie_round_trips() {
    let s = osatura_core::fsutil::ScratchDir::new("osatura-fix-lod");
    let p = demo(&s);
    let dir = s.join("lod").to_string_lossy().into_owned();
    let (ok, t) = run(&[
        "lod",
        &p,
        "--tolerance-px",
        "50",
        "--levels",
        "3",
        "--out-dir",
        &dir,
    ]);
    assert!(ok, "{t}");
    let lot = s.join("l.json").to_string_lossy().into_owned();
    let (ok, t) = run(&["export", &p, &lot]);
    assert!(ok, "{t}");
    let back = s.join("back.json").to_string_lossy().into_owned();
    let (ok, t) = run(&["import-lottie", &lot, &back]);
    assert!(ok, "{t}");
    let (ok, t) = run(&["check", &back]);
    assert!(ok, "{t}");
}
