// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! CLI round trip of the own face detector: face-synth -> face-track -> face-bake, face-detect, face-eval
//! option checks. Synthetic faces only; nothing here says anything about photographs or webcams.

use std::fs;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_osatura");

fn run(args: &[&str]) -> (bool, String) {
    let o = Command::new(BIN).args(args).output().expect("cannot start osatura");
    (o.status.success(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

#[test]
fn synth_track_bake_round_trip_and_error_paths() {
    let scratch = osatura_core::fsutil::ScratchDir::new("osatura-face-cli");
    let dir = scratch.to_path_buf();
    let d = |n: &str| dir.join(n).to_string_lossy().into_owned();
    let (ok, t) = run(&["face-synth", &d("frames"), "--frames", "8", "--size", "96", "--fps", "15"]);
    assert!(ok, "{t}");
    assert!(dir.join("frames/000007.png").exists() && dir.join("frames/truth.ndjson").exists());
    let (ok, t) = run(&["face-track", &d("frames"), &d("trk.ndjson"), "--fps", "15"]);
    assert!(ok, "{t}");
    let text = fs::read_to_string(d("trk.ndjson")).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 9);
    assert!(lines[0].contains("osatura-landmarks") && lines[0].contains("osatura23"));
    let found = lines[1..].iter().filter(|l| !l.contains("\"p\":null")).count();
    assert!(found >= 7, "face found in {found} of 8 frames");
    // the output is consumable by face-bake
    let (ok, t) = run(&["demo", &d("demo.json")]);
    assert!(ok, "{t}");
    fs::write(d("map.json"), r#"{"params":[{"source":"face","id":"mouth_open","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":60}],"triggers":[]}"#).unwrap();
    let (ok, t) = run(&["face-bake", &d("demo.json"), &d("trk.ndjson"), &d("map.json"), &d("baked.json"), "--calibrate-frames", "4", "--fps", "15"]);
    assert!(ok, "{t}");
    assert!(t.contains("+1 channels"), "{t}");
    // single-image detection as JSON
    let (ok, t) = run(&["face-detect", &d("frames/000003.png"), "--json"]);
    assert!(ok, "{t}");
    let v: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(v["faces"].as_array().unwrap().len(), 1);
    assert_eq!(v["faces"][0]["points"].as_array().unwrap().len(), 23);
    // errors
    assert!(!run(&["face-detect", &d("missing.png")]).0);
    assert!(!run(&["face-track", &d("nothing"), &d("o.ndjson")]).0);
    assert!(!run(&["face-train"]).0);
    assert!(!run(&["face-train", "--samples", "3", "--out", &d("m.bin")]).0);
    assert!(!run(&["face-detect", &d("frames/000003.png"), "--model", &d("trk.ndjson")]).0);
    let _ = fs::remove_dir_all(&dir);
}
