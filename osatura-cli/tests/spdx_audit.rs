// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Licence consistency audit: every source file in a workspace crate must carry an SPDX line equal to the
//! `license` of that crate's Cargo.toml (osatura-cli = AGPL-3.0-or-later, all others = MPL-2.0), and the
//! top-level `scripts/` must be MPL-2.0. Fails on a missing or mismatching identifier. Not legal advice.
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if name == "target" || name == "node_modules" || name.starts_with('.') {
                continue;
            }
            walk(&p, out);
        } else if matches!(
            p.extension().and_then(|x| x.to_str()),
            Some("rs" | "js" | "mjs" | "sh")
        ) {
            out.push(p);
        }
    }
}

fn spdx_of(text: &str) -> Option<String> {
    text.lines().take(6).find_map(|l| {
        // REUSE-IgnoreStart
        l.split_once("SPDX-License-Identifier:")
            .map(|(_, r)| r.trim().trim_end_matches("*/").trim().to_string())
        // REUSE-IgnoreEnd
    })
}

fn cargo_license(manifest: &Path) -> String {
    let t = fs::read_to_string(manifest).unwrap();
    for l in t.lines() {
        let l = l.trim();
        if let Some(r) = l.strip_prefix("license") {
            if let Some((_, v)) = r.split_once('=') {
                return v.trim().trim_matches('"').to_string();
            }
        }
    }
    panic!("no license field in {}", manifest.display());
}

#[test]
fn every_source_file_matches_its_crate_licence() {
    let r = root();
    let mut bad = Vec::new();
    let mut checked = 0;
    for krate in [
        "osatura-core",
        "osatura-cli",
        "osatura-capi",
        "osatura-wasm",
        "osatura-gpu",
    ] {
        let lic = cargo_license(&r.join(krate).join("Cargo.toml"));
        let want = if krate == "osatura-cli" {
            "AGPL-3.0-or-later"
        } else {
            "MPL-2.0"
        };
        assert_eq!(lic, want, "{krate}: Cargo license field");
        let mut files = Vec::new();
        walk(&r.join(krate), &mut files);
        for f in files {
            checked += 1;
            let text = fs::read_to_string(&f).unwrap();
            match spdx_of(&text) {
                Some(s) if s == lic => {}
                Some(s) => bad.push(format!("{}: SPDX {s} but crate licence {lic}", f.display())),
                None => bad.push(format!("{}: no SPDX line", f.display())),
            }
        }
    }
    let mut files = Vec::new();
    walk(&r.join("scripts"), &mut files);
    for f in files {
        checked += 1;
        let text = fs::read_to_string(&f).unwrap();
        if spdx_of(&text).as_deref() != Some("MPL-2.0") {
            bad.push(format!("{}: scripts/ must be MPL-2.0", f.display()));
        }
    }
    assert!(checked > 100, "suspiciously few files checked: {checked}");
    assert!(bad.is_empty(), "licence mismatches:\n{}", bad.join("\n"));
}

#[test]
fn reuse_toml_covers_non_source_files() {
    let r = root();
    let t = fs::read_to_string(r.join("REUSE.toml")).expect("REUSE.toml missing");
    for needle in [
        "studio.html",
        "osatura.h",
        "smoke.c",
        "wgsl",
        "face23_synth.bin",
        "api_surface.txt",
        "THIRD_PARTY_NOTICES.md",
    ] {
        assert!(t.contains(needle), "REUSE.toml does not mention {needle}");
    }
}
