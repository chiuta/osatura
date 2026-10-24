// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! L-02: `wasm32-unknown-unknown` cannot be built in the authoring environment (the target cannot be installed), so the
//! next best check runs here: the std calls that PANIC on that target (`Instant::now`, `SystemTime::now`,
//! `process::id`, `thread::spawn`) may appear only in `src/clock.rs` (which wraps them for the host clock) and in
//! `#[cfg(test)]` code. A new use outside `clock.rs` fails this test.

use std::fs;
use std::path::Path;

fn production_part(text: &str) -> &str {
    // everything before the first `#[cfg(test)]` line is production code in this code base
    match text.find("\n#[cfg(test)]") {
        Some(i) => &text[..i],
        None => text,
    }
}

/// Every `.rs` file under `dir`, including the module directories (`sizefit/`, `engine/`, ...).
fn rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

#[test]
fn panicking_std_calls_are_only_used_through_the_clock_module() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden = ["Instant::now", "SystemTime::now", "process::id()", "thread::spawn(", "std::time::Instant", "std::time::SystemTime", "UNIX_EPOCH"];
    let mut hits = Vec::new();
    let mut files = Vec::new();
    rs_files(&src, &mut files);
    for p in files {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // clock.rs is the one allowed user; tests.rs and *_tests.rs run on the host, not on wasm
        if name == "clock.rs" || name == "tests.rs" || name.ends_with("_tests.rs") {
            continue;
        }
        let text = fs::read_to_string(&p).unwrap();
        for (n, line) in production_part(&text).lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue; // comments and doc comments may mention them
            }
            for f in forbidden {
                if line.contains(f) {
                    hits.push(format!("{}:{}: {}", p.strip_prefix(&src).unwrap_or(&p).display(), n + 1, line.trim()));
                }
            }
        }
    }
    assert!(hits.is_empty(), "use crate::clock instead (these panic on wasm32-unknown-unknown):\n{}", hits.join("\n"));
}

/// The wasm module exports the clock set-up and the JavaScript wrapper feeds it (a host that never does gets durations of 0).
#[test]
fn the_wasm_module_and_wrapper_provide_the_host_clock() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("osatura-wasm");
    let lib = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    assert!(lib.contains("pub extern \"C\" fn arm_set_clock") && lib.contains("set_host_clock"), "arm_set_clock export missing");
    let js = fs::read_to_string(root.join("js/osatura.js")).unwrap();
    assert!(js.contains("arm_set_clock") && js.contains("performance.now") && js.contains("Date.now"), "the wrapper does not feed the clock");
}
