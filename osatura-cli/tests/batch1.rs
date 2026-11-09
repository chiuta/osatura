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
