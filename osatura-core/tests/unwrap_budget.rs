// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Ratchet for `.unwrap()` in production code. Counts the lines that call `.unwrap()` in the library and CLI sources, before the
//! first `#[cfg(test)]` of each file and not in test, fuzz or demo modules. The number may only go down: after removing an
//! `unwrap`, lower BUDGET; a new `unwrap` fails this test (use `?`, `ok_or_else`, `total_cmp` or `expect` with the invariant).
//! Sorting floats with `partial_cmp(..).unwrap()` is never acceptable: a NaN from imported data would panic (use `total_cmp`).
use std::{fs, path::{Path, PathBuf}};

const BUDGET: usize = 58;

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() { rs_files(&p, out); } else if p.extension().is_some_and(|x| x == "rs") { out.push(p); }
    }
}

#[test]
fn production_unwraps_do_not_grow_and_no_float_sort_can_panic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    let mut files = vec![];
    for c in ["osatura-core", "osatura-cli", "osatura-capi", "osatura-wasm", "osatura-gpu", "osatura-gui"] {
        let d = root.join(c).join("src");
        if d.is_dir() { rs_files(&d, &mut files); }
    }
    let (mut n, mut nan_sorts) = (0usize, vec![]);
    for f in files {
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        if name.ends_with("_tests.rs") || name == "fuzz.rs" || name == "demo.rs" { continue; }
        let s = fs::read_to_string(&f).unwrap();
        let body = s.split("#[cfg(test)]").next().unwrap();
        for (i, l) in body.lines().enumerate() {
            let t = l.trim_start();
            if t.starts_with("//") { continue; }
            if l.contains(".unwrap()") { n += 1; }
            if l.contains("partial_cmp") && l.contains(".unwrap()") { nan_sorts.push(format!("{}:{}", f.display(), i + 1)); }
        }
    }
    assert!(nan_sorts.is_empty(), "partial_cmp(..).unwrap() panics on NaN; use total_cmp: {nan_sorts:?}");
    assert!(n <= BUDGET, "production .unwrap() count is {n}, budget {BUDGET}: do not add new ones");
    assert!(n >= BUDGET - 5, "production .unwrap() count fell to {n}: lower BUDGET in this test to {n}");
}
