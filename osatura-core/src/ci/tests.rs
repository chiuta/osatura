// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `ci`.

use super::*;
use crate::demo;

fn tmpdir(tag: &str) -> crate::fsutil::ScratchDir {
    crate::fsutil::ScratchDir::new(&format!("osatura-ci-{tag}"))
}

#[test]
fn golden_hash_from_the_index_cannot_name_another_path() {
    let d = tmpdir("hashname");
    for bad in ["../../etc/passwd", "/etc/passwd", "", "abc", "ABCDEF0123456789ABCDEF01", &"a".repeat(70), "0123456789abcdef0123456/", "0123456789abcdef01234567/../x"] {
        assert!(golden_name_ok(bad).is_err(), "{bad:?} must be refused");
    }
    assert!(golden_name_ok(&"ab12".repeat(6)).is_ok());
    // through the index file: opening a store whose index names a path outside the directory fails
    std::fs::write(
        d.join("index.json"),
        format!(r#"{{"schema":"{INDEX_SCHEMA}","entries":{{"c":{{"0":{{"hash":"../../../etc/passwd","renderer":"raster","w":1,"h":1}}}}}}}}"#),
    )
    .unwrap();
    let err = GoldenStore::open(&d).err().expect("must be refused");
    assert!(err.contains("golden index"), "{err}");
    // and a direct reference is refused before any file is touched
    let store = GoldenStore { dir: d.clone(), index: GoldenIndex::default(), dirty: false };
    let r = GoldenRef { hash: "../../../etc/passwd".into(), renderer: "raster".into(), w: 1, h: 1 };
    assert!(store.load_img(&r).is_err());
    assert!(store.load_text(&r).is_err());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn manifest_paths_must_stay_inside_the_manifest_directory() {
    for bad in ["/etc/passwd", "../x.json", "a/../../x", "a\\..\\b", "", "C:\\x", "a\0b"] {
        assert!(manifest_path_ok("project", bad).is_err(), "{bad:?}");
    }
    for ok in ["demo.json", "sub/dir/p.json", "./p.json"] {
        assert!(manifest_path_ok("project", ok).is_ok(), "{ok:?}");
    }
}

fn write_demo(dir: &Path) -> PathBuf {
    let p = dir.join("demo.json");
    // 8 frames instead of 48 keeps the tests fast in debug builds
    let mut pr = demo::project();
    pr.frames = 8;
    std::fs::write(&p, serde_json::to_string(&pr).unwrap()).unwrap();
    p
}

fn small_manifest(dir: &Path) -> PathBuf {
    let m = json!({"schema": MANIFEST_SCHEMA, "project": "demo.json", "checks": [
        {"kind": "validate", "name": "validate"},
        {"kind": "render", "name": "frames", "frames": [0, 6], "scale": 0.1},
        {"kind": "render", "name": "svg", "frames": [0], "renderer": "svg"},
        {"kind": "determinism", "name": "det", "frames": [0, 5], "scale": 0.1},
        {"kind": "limits", "name": "limits", "max_layers": 500, "max_bones": 100},
        {"kind": "export-size", "name": "gif", "format": "gif", "max_bytes": 5000000, "scale": 0.1},
        {"kind": "perf", "name": "speed", "budget_ms": 1000.0, "scale": 0.1, "frames": [0]}
    ]});
    let p = dir.join(DEFAULT_MANIFEST);
    std::fs::write(&p, serde_json::to_string_pretty(&m).unwrap()).unwrap();
    p
}

#[test]
fn manifest_validation_reports_every_problem() {
    let bad = |v: Value| Manifest::parse(&v.to_string()).unwrap_err();
    assert!(bad(json!({"checks": []})).contains("checks is empty"));
    let e = bad(json!({"project": "p.json", "checks": [
        {"kind": "render", "name": "a b"},
        {"kind": "render", "name": "x", "scale": 0.0},
        {"kind": "render", "name": "x", "frames": [1], "step": 2},
        {"kind": "render", "name": "y", "thresholds": {"min_ssim": 2.0}},
        {"kind": "render", "name": "z", "renderer": "gpu"},
        {"kind": "render", "name": "w", "thresholds": {"metric": "lab"}},
        {"kind": "render", "name": "v", "ignore": [[0, 0, 0, 5]]},
        {"kind": "limits", "name": "l"},
        {"kind": "export-size", "name": "e", "format": "mp4", "max_bytes": 5},
        {"kind": "perf", "name": "p", "budget_ms": -1.0}
    ]}));
    for want in ["must be 1-64 characters", "scale must be in (0, 16]", "duplicate check name 'x'", "either 'frames' or 'step'", "min_ssim must be in -1..1", "unknown renderer 'gpu'", "unknown colour metric", "ignore rectangles", "at least one limit", "unknown format 'mp4'", "budget_ms must be"] {
        assert!(e.contains(want), "missing '{want}' in\n{e}");
    }
    assert!(bad(json!({"checks": [{"kind": "render", "name": "a"}]})).contains("no project"));
    assert!(bad(json!({"schema": "other/9", "project": "p", "checks": [{"kind": "validate", "name": "a"}]})).contains("schema must be"));
    assert!(bad(json!({"goldens": "../x", "project": "p", "checks": [{"kind": "validate", "name": "a"}]})).contains("goldens must be a relative path"));
    // unknown fields and kinds are errors, not silently ignored
    assert!(Manifest::parse(r#"{"project":"p","checks":[{"kind":"render","name":"a","scael":1}]}"#).is_err());
    assert!(Manifest::parse(r#"{"project":"p","checks":[{"kind":"teleport","name":"a"}]}"#).is_err());
    assert!(Manifest::parse("{not json").unwrap_err().starts_with("invalid manifest"));
    let ok = Manifest::parse(r#"{"project":"p.json","checks":[{"kind":"render","name":"a"}]}"#).unwrap();
    assert_eq!(ok.goldens, "goldens");
    assert_eq!(ok.schema, MANIFEST_SCHEMA);
    let again = Manifest::parse(&serde_json::to_string(&ok).unwrap()).unwrap();
    assert_eq!(ok, again, "manifest round-trips");
}

#[test]
fn golden_update_then_compare_round_trip() {
    let dir = tmpdir("roundtrip");
    write_demo(&dir);
    let mp = small_manifest(&dir);
    // no goldens yet: render checks fail, nothing is written
    let r0 = run(&mp, &RunOptions::default()).unwrap();
    assert_eq!(r0.exit_code(), 1);
    assert!(r0.checks.iter().find(|c| c.name == "frames").unwrap().frames[0].message.contains("no golden"));
    assert!(!dir.join("goldens").exists());
    // update writes content-hash named files and an index
    let r1 = run(&mp, &RunOptions { update: true, ..Default::default() }).unwrap();
    assert_eq!(r1.exit_code(), 0, "{}", r1.to_json());
    assert_eq!(r1.mode, "update");
    let idx: GoldenIndex = serde_json::from_str(&std::fs::read_to_string(dir.join("goldens/index.json")).unwrap()).unwrap();
    assert_eq!(idx.entries["frames"].len(), 2);
    let g = &idx.entries["frames"]["00006"];
    assert_eq!(g.hash.len(), 24);
    assert!(dir.join("goldens").join(format!("{}.png", g.hash)).exists());
    assert!(dir.join("goldens").join(format!("{}.svg", idx.entries["svg"]["00000"].hash)).exists());
    let before = std::fs::read(dir.join("goldens/index.json")).unwrap();
    // verify passes, exactly
    let r2 = run(&mp, &RunOptions::default()).unwrap();
    assert_eq!(r2.exit_code(), 0, "{}", r2.to_json());
    assert!(r2.checks.iter().find(|c| c.name == "frames").unwrap().frames.iter().all(|f| f.exact));
    // a second update changes nothing on disk
    let r3 = run(&mp, &RunOptions { update: true, ..Default::default() }).unwrap();
    assert_eq!(r3.summary.updated, 0);
    assert_eq!(std::fs::read(dir.join("goldens/index.json")).unwrap(), before);
    // --only
    let r4 = run(&mp, &RunOptions { only: vec!["frames".into()], ..Default::default() }).unwrap();
    assert_eq!(r4.checks.len(), 1);
    assert!(run(&mp, &RunOptions { only: vec!["nope".into()], ..Default::default() }).unwrap_err().contains("no check named 'nope'"));
    assert!(run(&mp, &RunOptions { update: true, bless: true, ..Default::default() }).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

fn move_a_layer(dir: &Path) {
    let p = dir.join("demo.json");
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let layer = v["layers"].as_array_mut().unwrap().iter_mut().find(|l| l["name"] == "sun").expect("demo has a sun layer");
    let pos = layer["transform"]["position"].as_array().unwrap().clone();
    layer["transform"]["position"] = json!([pos[0].as_f64().unwrap() + 60.0, pos[1].as_f64().unwrap() + 30.0]);
    std::fs::write(&p, serde_json::to_string(&v).unwrap()).unwrap();
}

#[test]
fn a_changed_project_fails_with_regions_and_bless_accepts_it() {
    let dir = tmpdir("regress");
    write_demo(&dir);
    let mp = small_manifest(&dir);
    run(&mp, &RunOptions { update: true, ..Default::default() }).unwrap();
    move_a_layer(&dir);
    let arts = dir.join("art");
    let r = run(&mp, &RunOptions { artifacts: Some(arts.clone()), ..Default::default() }).unwrap();
    assert_eq!(r.exit_code(), 1, "{}", r.to_json());
    let c = r.checks.iter().find(|c| c.name == "frames").unwrap();
    assert_eq!(c.status, Status::Fail);
    let f = c.frames.iter().find(|f| f.status == Status::Fail).unwrap();
    let m = f.metrics.as_ref().unwrap();
    assert!(m.diff_pixels > 0 && m.ssim < 1.0 && m.max_de > 1.0, "{m:?}");
    assert!(!f.regions.is_empty());
    assert!(f.images.golden.is_some() && f.images.actual.is_some() && f.images.heat.is_some());
    assert!(arts.join("frames").join(format!("f{:05}-diff.png", f.frame)).exists());
    // the other checks are unaffected by the visual change
    assert_eq!(r.checks.iter().find(|c| c.name == "det").unwrap().status, Status::Pass);
    // bless accepts exactly the failing frames and then verify passes
    let b = run(&mp, &RunOptions { bless: true, ..Default::default() }).unwrap();
    assert_eq!(b.exit_code(), 0);
    assert!(b.summary.blessed >= 1);
    assert!(run(&mp, &RunOptions::default()).unwrap().exit_code() == 0);
    // prune removes the now unreferenced old goldens
    let n_before = std::fs::read_dir(dir.join("goldens")).unwrap().count();
    run(&mp, &RunOptions { prune: true, ..Default::default() }).unwrap();
    let n_after = std::fs::read_dir(dir.join("goldens")).unwrap().count();
    assert!(n_after < n_before, "{n_after} < {n_before}");
    assert!(run(&mp, &RunOptions::default()).unwrap().exit_code() == 0, "prune kept every referenced golden");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tampered_goldens_and_broken_inputs_are_detected() {
    let dir = tmpdir("tamper");
    write_demo(&dir);
    let mp = small_manifest(&dir);
    run(&mp, &RunOptions { update: true, ..Default::default() }).unwrap();
    // overwrite one golden with other pixels: its name no longer matches
    let idx: GoldenIndex = serde_json::from_str(&std::fs::read_to_string(dir.join("goldens/index.json")).unwrap()).unwrap();
    let g = &idx.entries["frames"]["00006"];
    let other = Img::solid(10, 10, [1, 2, 3, 255]);
    std::fs::write(dir.join("goldens").join(format!("{}.png", g.hash)), other.to_png().unwrap()).unwrap();
    // make the frame differ so the golden is actually read
    move_a_layer(&dir);
    let r = run(&mp, &RunOptions::default()).unwrap();
    let f = r.checks.iter().find(|c| c.name == "frames").unwrap().frames.iter().find(|f| f.frame == 6).unwrap();
    assert_eq!(f.status, Status::Error);
    assert!(f.message.contains("was modified"), "{}", f.message);
    assert_eq!(r.exit_code(), 2);
    // missing project: the check errors (exit 2), the run continues
    std::fs::remove_file(dir.join("demo.json")).unwrap();
    let r = run(&mp, &RunOptions::default()).unwrap();
    assert_eq!(r.exit_code(), 2);
    assert!(r.checks.iter().all(|c| c.status == Status::Error || c.status == Status::Fail));
    // missing manifest and a corrupt golden index are hard errors
    assert!(run(&dir.join("none.json"), &RunOptions::default()).unwrap_err().contains("cannot read manifest"));
    std::fs::write(dir.join("goldens/index.json"), "{\"schema\":\"x\"}").unwrap();
    assert!(run(&mp, &RunOptions::default()).unwrap_err().contains("schema"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn frames_outside_the_project_are_an_error_not_a_panic() {
    let dir = tmpdir("range");
    write_demo(&dir);
    let m = json!({"project": "demo.json", "checks": [{"kind": "render", "name": "far", "frames": [9999], "scale": 0.1}]});
    let mp = dir.join(DEFAULT_MANIFEST);
    std::fs::write(&mp, m.to_string()).unwrap();
    let r = run(&mp, &RunOptions::default()).unwrap();
    assert_eq!(r.exit_code(), 2);
    assert!(r.checks[0].message.contains("outside the project"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn size_change_is_reported_as_a_failure() {
    let dir = tmpdir("size");
    write_demo(&dir);
    let mk = |scale: f32| {
        let m = json!({"project": "demo.json", "checks": [{"kind": "render", "name": "s", "frames": [0], "scale": scale}]});
        let p = dir.join(DEFAULT_MANIFEST);
        std::fs::write(&p, m.to_string()).unwrap();
        p
    };
    run(&mk(0.1), &RunOptions { update: true, ..Default::default() }).unwrap();
    let r = run(&mk(0.2), &RunOptions::default()).unwrap();
    assert_eq!(r.exit_code(), 1);
    assert!(r.checks[0].frames[0].message.contains("size changed"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn thresholds_decide_between_pass_and_fail() {
    let dir = tmpdir("thresh");
    write_demo(&dir);
    let mk = |t: Value| {
        let m = json!({"project": "demo.json", "checks": [{"kind": "render", "name": "s", "frames": [0], "scale": 0.1, "thresholds": t}]});
        let p = dir.join(DEFAULT_MANIFEST);
        std::fs::write(&p, m.to_string()).unwrap();
        p
    };
    run(&mk(json!({})), &RunOptions { update: true, ..Default::default() }).unwrap();
    // move the sun by a few stage units: a small local change
    let p = dir.join("demo.json");
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let sun = v["layers"].as_array_mut().unwrap().iter_mut().find(|l| l["name"] == "sun").unwrap();
    let pos = sun["transform"]["position"].as_array().unwrap().clone();
    sun["transform"]["position"] = json!([pos[0].as_f64().unwrap() + 3.0, pos[1].as_f64().unwrap()]);
    std::fs::write(&p, v.to_string()).unwrap();
    let strict = run(&mk(json!({"pixel_de": 0.0, "max_de": 0.0})), &RunOptions::default()).unwrap();
    assert_eq!(strict.exit_code(), 1);
    let lax = run(&mk(json!({"pixel_de": 50.0, "max_de": 50.0, "min_ssim": 0.5, "max_diff_fraction": 1.0})), &RunOptions::default()).unwrap();
    assert_eq!(lax.exit_code(), 0, "{}", lax.to_json());
    assert!(!lax.checks[0].frames[0].exact);
    assert!(lax.checks[0].frames[0].metrics.is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ignore_regions_hide_a_change_in_a_render_check() {
    let dir = tmpdir("mask");
    write_demo(&dir);
    let mk = |ignore: Value| {
        let m = json!({"project": "demo.json", "checks": [{"kind": "render", "name": "s", "frames": [0], "scale": 0.5, "ignore": ignore}]});
        let p = dir.join(DEFAULT_MANIFEST);
        std::fs::write(&p, m.to_string()).unwrap();
        p
    };
    let e = crate::engine::Engine::new(demo::project()).unwrap();
    let (w, h) = (e.project.width, e.project.height);
    run(&mk(json!([])), &RunOptions { update: true, ..Default::default() }).unwrap();
    move_a_layer(&dir);
    assert_eq!(run(&mk(json!([])), &RunOptions::default()).unwrap().exit_code(), 1);
    let r = run(&mk(json!([[0.0, 0.0, w, h]])), &RunOptions::default()).unwrap();
    assert_eq!(r.exit_code(), 0, "{}", r.to_json());
    assert_eq!(r.checks[0].frames[0].metrics.as_ref().unwrap().diff_pixels, 0);
    assert!(r.checks[0].frames[0].metrics.as_ref().unwrap().ignored_pixels > 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn structural_checks_pass_and_fail_for_the_right_reasons() {
    let dir = tmpdir("struct");
    write_demo(&dir);
    let m = json!({"project": "demo.json", "checks": [
        {"kind": "limits", "name": "tight", "max_layers": 1},
        {"kind": "limits", "name": "roomy", "max_layers": 1000, "max_file_bytes": 10000000},
        {"kind": "export-size", "name": "tiny", "format": "gif", "max_bytes": 10, "scale": 0.1},
        {"kind": "export-size", "name": "lottie", "format": "lottie", "max_bytes": 10000000},
        {"kind": "export-size", "name": "html", "format": "html", "max_bytes": 10000000},
        {"kind": "export-size", "name": "apng", "format": "apng", "max_bytes": 10000000, "scale": 0.1},
        {"kind": "export-size", "name": "svgs", "format": "svg-frames", "max_bytes": 10000000},
        {"kind": "perf", "name": "slow-info", "budget_ms": 0.0001, "tolerance": 0.0, "scale": 0.1, "frames": [0]},
        {"kind": "perf", "name": "slow-enforced", "budget_ms": 0.0001, "tolerance": 0.0, "scale": 0.1, "frames": [0], "enforce": true},
        {"kind": "perf", "name": "fast-enforced", "budget_ms": 100000.0, "scale": 0.1, "frames": [0], "enforce": true},
        {"kind": "validate", "name": "valid", "lint": "off"},
        {"kind": "fuzz", "name": "fuzz", "samples": 20, "max_fail_rate": 1.0}
    ]});
    let mp = dir.join(DEFAULT_MANIFEST);
    std::fs::write(&mp, m.to_string()).unwrap();
    let r = run(&mp, &RunOptions::default()).unwrap();
    let st = |n: &str| r.checks.iter().find(|c| c.name == n).unwrap().status;
    assert_eq!(st("tight"), Status::Fail);
    assert!(r.checks.iter().find(|c| c.name == "tight").unwrap().message.contains("layers"));
    assert_eq!(st("roomy"), Status::Pass);
    assert_eq!(st("tiny"), Status::Fail);
    for n in ["lottie", "html", "apng", "svgs", "fast-enforced", "valid"] {
        assert_eq!(st(n), Status::Pass, "{n}: {}", r.checks.iter().find(|c| c.name == n).unwrap().message);
    }
    assert_eq!(st("fuzz"), Status::Info, "a 100 % fuzz gate cannot fail: information, not a pass");
    assert_eq!(st("slow-info"), Status::Info, "performance is informational unless enforced");
    assert_eq!(st("slow-enforced"), Status::Fail);
    assert_eq!(r.exit_code(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fuzz_check_fails_when_the_rate_is_exceeded() {
    let dir = tmpdir("fuzz");
    // a project whose fuzzing is expected to produce some failures is hard to build; instead use rate 0 on a
    // project and an impossible negative-free bound: run twice with max_fail_rate 0 and 1 and compare semantics
    write_demo(&dir);
    let mk = |rate: f32| {
        let m = json!({"project": "demo.json", "checks": [{"kind": "fuzz", "name": "f", "samples": 30, "max_fail_rate": rate}]});
        let p = dir.join(DEFAULT_MANIFEST);
        std::fs::write(&p, m.to_string()).unwrap();
        p
    };
    // C12b: a gate at 100 % can never fail, so it is reported as information, never as a pass
    let loose = run(&mk(1.0), &RunOptions::default()).unwrap();
    assert_eq!(loose.checks[0].status, Status::Info, "{}", loose.checks[0].message);
    assert!(loose.checks[0].message.contains("cannot fail"));
    assert_eq!(loose.exit_code(), 0);
    let rate = loose.checks[0].details["failure_rate"].as_f64().unwrap();
    let tight = run(&mk(0.0), &RunOptions::default()).unwrap();
    assert_eq!(tight.checks[0].status, if rate > 0.0 { Status::Fail } else { Status::Pass });
    let _ = std::fs::remove_dir_all(&dir);
}

/// Documented manifest semantics of a `fuzz` check (docs/CI.md): it GATES by default (`gate` unset = true) with
/// `max_fail_rate` defaulting to 0; only `"gate": false` or `max_fail_rate >= 1` turn it into information.
#[test]
fn fuzz_check_gates_by_default_and_gate_false_only_informs() {
    let dir = tmpdir("fuzzgate");
    write_demo(&dir);
    let run_with = |extra: serde_json::Value| {
        let mut c = json!({"kind": "fuzz", "name": "f", "samples": 30});
        for (k, v) in extra.as_object().unwrap() {
            c[k] = v.clone();
        }
        let p = dir.join(DEFAULT_MANIFEST);
        std::fs::write(&p, json!({"project": "demo.json", "checks": [c]}).to_string()).unwrap();
        run(&p, &RunOptions::default()).unwrap()
    };
    // neither `gate` nor `max_fail_rate` given: a gate with the default rate 0
    let dflt = run_with(json!({}));
    let rate = dflt.checks[0].details["failure_rate"].as_f64().unwrap();
    assert_eq!(dflt.checks[0].details["max_fail_rate"].as_f64(), Some(0.0));
    if rate > 0.0 {
        assert_eq!(dflt.checks[0].status, Status::Fail, "{}", dflt.checks[0].message);
        assert_eq!(dflt.exit_code(), 1);
    } else {
        assert_eq!(dflt.checks[0].status, Status::Pass);
    }
    // `gate: true` is the same as the default
    assert_eq!(run_with(json!({"gate": true})).checks[0].status, dflt.checks[0].status);
    // `gate: false`: reported, never failing, never a pass
    let off = run_with(json!({"gate": false}));
    assert_eq!(off.checks[0].status, Status::Info);
    assert!(off.checks[0].message.contains("gate = false"), "{}", off.checks[0].message);
    assert_eq!(off.exit_code(), 0);
    // a rate bound at or above the measured failure rate passes
    if rate < 0.98 {
        let ok = run_with(json!({"max_fail_rate": rate + 0.01}));
        assert_eq!(ok.checks[0].status, Status::Pass, "{}", ok.checks[0].message);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// C12b: `init` on a project that already fails most poses writes an information-only fuzz check.
#[test]
fn init_writes_a_non_gating_fuzz_check_when_most_poses_fail() {
    let dir = tmpdir("initfuzz");
    write_demo(&dir);
    let m = init_manifest(&dir.join("demo.json"), &dir.join(DEFAULT_MANIFEST)).unwrap();
    let f = m.checks.iter().find_map(|c| if let Check::Fuzz(f) = c { Some(f.clone()) } else { None }).expect("fuzz check");
    let e = Engine::new(demo::project()).unwrap();
    let rate = crate::fuzz::run(&e.project, None, &crate::fuzz::FuzzOptions { samples: 100, seed: 1, ..Default::default() }).unwrap().summary.failure_rate;
    if rate > 0.5 {
        assert_eq!(f.gate, Some(false), "rate {rate}");
    } else {
        assert!(f.max_fail_rate.unwrap() < 1.0 && f.gate.is_none(), "rate {rate}");
    }
    // whichever it is, the written manifest can never end in a PASS that cannot fail
    std::fs::write(dir.join(DEFAULT_MANIFEST), serde_json::to_string(&m).unwrap()).unwrap();
    let r = run(&dir.join(DEFAULT_MANIFEST), &RunOptions { only: vec!["pose-fuzz".into()], ..Default::default() }).unwrap();
    let st = r.checks.iter().find(|c| c.name == "pose-fuzz").unwrap().status;
    assert!(st == Status::Info || st == Status::Pass);
    if f.max_fail_rate.unwrap() >= 1.0 {
        assert_eq!(st, Status::Info);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn loop_check_separates_a_loop_from_a_one_way_motion() {
    let dir = tmpdir("loop");
    let mut p = demo::project();
    // a static scene always loops
    p.channels.clear();
    let path = dir.join("static.json");
    std::fs::write(&path, serde_json::to_string(&p).unwrap()).unwrap();
    let m = json!({"project": "static.json", "checks": [{"kind": "loop", "name": "l"}]});
    let mp = dir.join(DEFAULT_MANIFEST);
    std::fs::write(&mp, m.to_string()).unwrap();
    assert_eq!(run(&mp, &RunOptions::default()).unwrap().checks[0].status, Status::Pass);
    // statistics on the demo: the seam is a number, the check is deterministic
    let e = Engine::new(demo::project()).unwrap();
    let a = loop_stats(&e, 0.1).unwrap();
    let b = loop_stats(&e, 0.1).unwrap();
    assert_eq!(a, b);
    assert!(a.0 >= 0.0 && a.1 >= 0.0 && a.2 >= a.1);
    // a one-frame project cannot loop-check
    let mut one = demo::project();
    one.frames = 1;
    assert!(loop_stats(&Engine::new(one).unwrap(), 0.1).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn init_creates_a_manifest_that_passes_on_the_project_it_was_made_from() {
    let dir = tmpdir("init");
    let proj = write_demo(&dir);
    // a short, small copy keeps this test fast in debug builds
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&proj).unwrap()).unwrap();
    v["frames"] = json!(8);
    v["width"] = json!(160.0);
    v["height"] = json!(120.0);
    std::fs::write(&proj, v.to_string()).unwrap();
    let mp = dir.join(DEFAULT_MANIFEST);
    let m = init_manifest(&proj, &mp).unwrap();
    assert_eq!(m.project.as_deref(), Some("demo.json"));
    let kinds: Vec<&str> = m.checks.iter().map(|c| c.kind()).collect();
    for k in ["validate", "render", "determinism", "fuzz", "limits", "export-size", "perf"] {
        assert!(kinds.contains(&k), "init should add a {k} check: {kinds:?}");
    }
    std::fs::write(&mp, serde_json::to_string_pretty(&m).unwrap()).unwrap();
    run(&mp, &RunOptions { update: true, ..Default::default() }).unwrap();
    let r = run(&mp, &RunOptions::default()).unwrap();
    assert_eq!(r.exit_code(), 0, "{}", r.to_json());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn runs_are_deterministic_apart_from_timings() {
    let dir = tmpdir("repeat");
    write_demo(&dir);
    let mp = small_manifest(&dir);
    run(&mp, &RunOptions { update: true, ..Default::default() }).unwrap();
    let strip = |r: &CiReport| -> Value {
        let mut v = serde_json::to_value(r).unwrap();
        v["duration_ms"] = json!(0);
        for c in v["checks"].as_array_mut().unwrap() {
            c["duration_ms"] = json!(0);
            if c["kind"] == "perf" {
                c["details"] = json!(null);
                c["message"] = json!("");
            }
        }
        v
    };
    let a = run(&mp, &RunOptions::default()).unwrap();
    let b = run(&mp, &RunOptions::default()).unwrap();
    assert_eq!(strip(&a), strip(&b));
    let _ = std::fs::remove_dir_all(&dir);
}

fn failing_report() -> (crate::fsutil::ScratchDir, CiReport) {
    let dir = tmpdir("report");
    write_demo(&dir);
    let mp = small_manifest(&dir);
    run(&mp, &RunOptions { update: true, ..Default::default() }).unwrap();
    move_a_layer(&dir);
    let r = run(&mp, &RunOptions::default()).unwrap();
    (dir, r)
}

#[test]
fn junit_xml_is_well_formed_and_counts_match() {
    let (dir, mut r) = failing_report();
    // add awkward characters to the messages
    r.checks[0].message = "a <b> & \"c\" 'd' \u{1}".into();
    let x = junit_xml(&r);
    assert!(x.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    // minimal well-formedness check with a tiny tag stack
    let mut stack: Vec<String> = Vec::new();
    let mut i = 0;
    let b = x.as_bytes();
    while i < b.len() {
        if b[i] == b'<' {
            let j = x[i..].find('>').unwrap() + i;
            let tag = &x[i + 1..j];
            if tag.starts_with('?') {
            } else if let Some(name) = tag.strip_prefix('/') {
                assert_eq!(stack.pop().as_deref(), Some(name), "closing tag mismatch");
            } else if !tag.ends_with('/') {
                stack.push(tag.split_whitespace().next().unwrap().to_string());
            }
            i = j;
        } else if b[i] == b'&' {
            let j = x[i..].find(';').expect("entity must end") + i;
            assert!(["&amp", "&lt", "&gt", "&quot", "&apos"].contains(&&x[i..j]), "bad entity {}", &x[i..=j]);
        }
        i += 1;
    }
    assert!(stack.is_empty());
    assert!(!x.contains('\u{1}'));
    assert_eq!(x.matches("<testcase ").count(), r.checks.len());
    assert_eq!(x.matches("<failure ").count(), r.summary.failed);
    assert!(x.contains("<testsuites name=\"osatura-ci\"") && x.contains(&format!("failures=\"{}\"", r.summary.failed)));
    assert!(x.contains("frame 0") || x.contains("frame 6"), "failure text names the frame");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn html_report_is_self_contained_and_shows_failures() {
    let (dir, r) = failing_report();
    let h = html_report(&r);
    assert!(h.starts_with("<!doctype html>"));
    assert!(h.contains("data:image/png;base64,"), "images are inlined");
    assert!(h.matches("data:image/png;base64,").count() >= 3, "golden, current and difference");
    for bad in ["src=\"http", "href=\"http", "url(http", "<link ", "<script src", "@import"] {
        assert!(!h.contains(bad), "external reference '{bad}'");
    }
    assert!(h.contains("Osatura CI report") && h.contains("exit code 1"));
    assert!(h.contains("frames") && h.contains("class=\"b fail\""));
    assert!(h.contains("<table") && h.contains("Region"));
    assert!(h.len() < 16_000_000);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn exit_codes_follow_the_worst_result() {
    let mk = |s: &[Status]| {
        let checks: Vec<CheckResult> = s.iter().enumerate().map(|(i, &st)| CheckResult { name: format!("c{i}"), kind: "limits".into(), status: st, message: String::new(), duration_ms: 0.0, frames: vec![], details: Value::Null }).collect();
        CiReport { schema: REPORT_SCHEMA.into(), manifest: "m".into(), mode: "verify".into(), duration_ms: 0.0, summary: CiReport::summarize(&checks), checks }
    };
    assert_eq!(mk(&[Status::Pass, Status::Info, Status::Updated, Status::Blessed]).exit_code(), 0);
    assert_eq!(mk(&[Status::Pass, Status::Fail]).exit_code(), 1);
    assert_eq!(mk(&[Status::Fail, Status::Error]).exit_code(), 2);
    assert_eq!(mk(&[]).exit_code(), 0);
    let j: Value = serde_json::from_str(&mk(&[Status::Fail]).to_json()).unwrap();
    assert_eq!(j["schema"], REPORT_SCHEMA);
    assert_eq!(j["summary"]["failed"], 1);
}

#[test]
fn float_renderer_goldens_work_like_raster_ones() {
    let dir = tmpdir("float");
    write_demo(&dir);
    let m = json!({"project": "demo.json", "checks": [{"kind": "render", "name": "fl", "frames": [3], "scale": 0.1, "renderer": "float"}]});
    let mp = dir.join(DEFAULT_MANIFEST);
    std::fs::write(&mp, m.to_string()).unwrap();
    run(&mp, &RunOptions { update: true, ..Default::default() }).unwrap();
    assert_eq!(run(&mp, &RunOptions::default()).unwrap().exit_code(), 0);
    let _ = std::fs::remove_dir_all(&dir);
}
