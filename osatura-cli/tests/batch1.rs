// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! CLI tests of the functions added in clean-room batch 1 (docs/clean-room/specs).
use std::process::Command;

fn osatura(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_osatura")).args(args).output().expect("run osatura")
}

#[test]
fn schema_goes_to_stdout_or_a_file_and_is_deterministic() {
    let a = osatura(&["schema"]);
    assert!(a.status.success());
    let b = osatura(&["schema"]);
    assert_eq!(a.stdout, b.stdout);
    let v: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(v["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert!(v["properties"]["frames"].is_object());
    assert!(v.get("additionalProperties").is_none());
    let strict = osatura(&["schema", "--strict"]);
    let v: serde_json::Value = serde_json::from_slice(&strict.stdout).unwrap();
    assert_eq!(v["additionalProperties"], false);
    let dir = std::env::temp_dir().join(format!("osatura-schema-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join("s.json");
    let w = osatura(&["schema", out.to_str().unwrap()]);
    assert!(w.status.success());
    assert_eq!(std::fs::read(&out).unwrap(), a.stdout);
    std::fs::remove_dir_all(&dir).ok();
}

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("osatura-batch1-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn render_shutter_blurs_a_moving_demo_and_rejects_bad_use() {
    let proj = tmp("demo.json");
    assert!(osatura(&["demo", proj.to_str().unwrap()]).status.success());
    let (plain, zero, blur, again) = (tmp("plain.png"), tmp("zero.png"), tmp("blur.png"), tmp("again.png"));
    let p = proj.to_str().unwrap();
    assert!(osatura(&["render", p, "6", plain.to_str().unwrap()]).status.success());
    // angle 0: identical to the plain render
    assert!(osatura(&["render", p, "6", zero.to_str().unwrap(), "--shutter", "0"]).status.success());
    assert_eq!(std::fs::read(&plain).unwrap(), std::fs::read(&zero).unwrap());
    let args = ["render", p, "6", blur.to_str().unwrap(), "--shutter", "360", "--samples", "4"];
    assert!(osatura(&args).status.success());
    assert_ne!(std::fs::read(&plain).unwrap(), std::fs::read(&blur).unwrap());
    // reproducible
    assert!(osatura(&["render", p, "6", again.to_str().unwrap(), "--shutter", "360", "--samples", "4"]).status.success());
    assert_eq!(std::fs::read(&blur).unwrap(), std::fs::read(&again).unwrap());
    // errors: angle out of range, samples without shutter, svg output, zero samples
    let bad = |extra: &[&str], out: &str| {
        let mut v = vec!["render", p, "6", out];
        v.extend_from_slice(extra);
        let o = osatura(&v);
        assert!(!o.status.success(), "{extra:?}");
        String::from_utf8_lossy(&o.stderr).to_string()
    };
    assert!(bad(&["--shutter", "400"], blur.to_str().unwrap()).contains("[0, 360]"));
    assert!(bad(&["--samples", "4"], blur.to_str().unwrap()).contains("--shutter"));
    assert!(bad(&["--shutter", "90"], tmp("x.svg").to_str().unwrap()).contains(".png"));
    assert!(bad(&["--shutter", "90", "--samples", "0"], blur.to_str().unwrap()).contains("samples"));
    std::fs::remove_dir_all(proj.parent().unwrap()).ok();
}

fn keys_of(project: &std::path::Path, target: &str) -> Vec<(f64, f64, String)> {
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(project).unwrap()).unwrap();
    let c = v["channels"].as_array().unwrap().iter().find(|c| c["target"] == target && c["prop"] == "bone_angle").unwrap();
    c["track"]["keys"].as_array().unwrap().iter().map(|k| (k["frame"].as_f64().unwrap(), k["value"].as_f64().unwrap(), k["ease"]["type"].as_str().unwrap_or("linear").to_string())).collect()
}

#[test]
fn keys_curve_bakes_a_stepped_curve_into_the_selected_segment_only() {
    let proj = tmp("curve-demo.json");
    let p = proj.to_str().unwrap();
    assert!(osatura(&["demo", p]).status.success());
    let out = tmp("curve-out.json");
    let o = osatura(&["keys", "curve", p, out.to_str().unwrap(), "--curve", "steps:4", "--target", "arm_up", "--prop", "bone_angle", "--from", "0", "--to", "0"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let (before, after) = (keys_of(&proj, "arm_up"), keys_of(&out, "arm_up"));
    // the first segment (0..12, 0 -> -25) became four held plateaus, the rest is unchanged
    assert_eq!(after.len(), before.len() + 3);
    let head: Vec<(f64, f64, &str)> = after[..4].iter().map(|k| (k.0, k.1, k.2.as_str())).collect();
    assert_eq!(head, vec![(0.0, 0.0, "step"), (3.0, -6.25, "step"), (6.0, -12.5, "step"), (9.0, -18.75, "step")]);
    assert_eq!(after[4..], before[1..]);
    // other channels are not touched
    assert_eq!(keys_of(&proj, "hand"), keys_of(&out, "hand"));
    // refusals say why and write nothing
    let gone = tmp("curve-none.json");
    let o = osatura(&["keys", "curve", p, gone.to_str().unwrap(), "--curve", "steps:4:jump-start"]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("not 0"));
    assert!(!gone.exists());
    let o = osatura(&["keys", "curve", p, gone.to_str().unwrap()]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("--curve"));
}
