// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Regression tests for the doc/behaviour mismatches found while documenting the project: every output goes through the
//! atomic, symlink-refusing writer; `golden check` is exact; strict flags; the `export` atlas name; `render` frame range;
//! `fuzz` exit semantics; `take-apply` tokens; `studio` option checks, `/favicon.ico` and the cookie; `mirror` and
//! `script` warnings; the editor-data warnings of `check`. Each test fails on the behaviour it replaced.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_osatura");

struct Out {
    ok: bool,
    stdout: String,
    stderr: String,
}

impl Out {
    fn all(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

#[cfg_attr(not(unix), allow(dead_code))]
fn have_ffmpeg() -> bool {
    Command::new("ffmpeg").arg("-version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn run_in(dir: &Path, args: &[&str]) -> Out {
    let o = Command::new(BIN).current_dir(dir).args(args).output().expect("cannot start osatura");
    Out { ok: o.status.success(), stdout: String::from_utf8_lossy(&o.stdout).into_owned(), stderr: String::from_utf8_lossy(&o.stderr).into_owned() }
}

/// Like `run_in`, but a command that is still running after `secs` (a server that should have refused to start) is killed
/// and reported as a failure of the test.
fn run_limited(dir: &Path, args: &[&str], secs: u64) -> Out {
    let mut c = Command::new(BIN).current_dir(dir).args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("cannot start osatura");
    let t0 = Instant::now();
    loop {
        if let Some(st) = c.try_wait().unwrap() {
            let (mut so, mut se) = (String::new(), String::new());
            c.stdout.take().unwrap().read_to_string(&mut so).unwrap();
            c.stderr.take().unwrap().read_to_string(&mut se).unwrap();
            return Out { ok: st.success(), stdout: so, stderr: se };
        }
        if t0.elapsed() > Duration::from_secs(secs) {
            let _ = c.kill();
            let _ = c.wait();
            panic!("{args:?} was still running after {secs} s (it should have exited)");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// A tiny two-bone project that every command handles in milliseconds.
fn tiny_json() -> String {
    serde_json::json!({
        "format": 1, "name": "tiny", "width": 32, "height": 24, "fps": 12, "frames": 4,
        "bones": [
            {"name": "arm_up", "length": 10, "rest_offset": [8, 12], "rest_angle": 0},
            {"name": "arm_low", "parent": "arm_up", "length": 10, "rest_offset": [10, 0], "rest_angle": 0}
        ],
        "layers": [{"name": "box", "type": "vector", "paths": [{"name": "r", "closed": true, "anchors": [
            {"pos": [2, 2], "weights": [{"bone": "arm_up", "weight": 1}]}, {"pos": [20, 2], "weights": [{"bone": "arm_up", "weight": 1}]},
            {"pos": [20, 20], "weights": [{"bone": "arm_up", "weight": 1}]}, {"pos": [2, 20], "weights": [{"bone": "arm_up", "weight": 1}]}], "fill": "#336699"}]}],
        "channels": [
            {"target": "arm_up", "prop": "bone_angle", "source": "keys", "track": {"keys": [{"frame": 0, "value": 0, "ease": {"type": "linear"}}, {"frame": 3, "value": 20, "ease": {"type": "linear"}}]}},
            {"target": "arm_low", "prop": "bone_angle", "source": "keys", "track": {"keys": [{"frame": 0, "value": 0, "ease": {"type": "linear"}}, {"frame": 3, "value": -30, "ease": {"type": "linear"}}]}}
        ]
    })
    .to_string()
}

fn setup(tag: &str) -> osatura_core::fsutil::ScratchDir {
    let d = osatura_core::fsutil::ScratchDir::new(&format!("osatura-docfix-{tag}"));
    fs::write(d.join("p.json"), tiny_json()).unwrap();
    fs::write(d.join("target.txt"), "precious").unwrap();
    d
}

#[cfg_attr(not(unix), allow(dead_code))]
#[cfg(unix)]
fn link(d: &Path, rel: &str) {
    let p = d.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    std::os::unix::fs::symlink(d.join("target.txt"), &p).unwrap();
}

/// Item 1: every command that writes a file the user names refuses a symbolic link in its place, leaves the link's
/// target untouched, and says why. (Unix only: needs symlinks.)
#[cfg(unix)]
#[test]
fn every_user_addressed_output_refuses_a_symlink() {
    // (label, setup commands that run first, link to plant, command; the planted link is the output)
    type Case = (&'static str, Vec<Vec<&'static str>>, &'static str, Vec<&'static str>);
    let cases: Vec<Case> = vec![
        ("fuzz --report", vec![], "link", vec!["fuzz", "p.json", "--samples", "3", "--report", "link"]),
        ("fuzz --heatmap", vec![], "link", vec!["fuzz", "p.json", "--samples", "3", "--heatmap", "link"]),
        ("fuzz-grid", vec![], "link", vec!["fuzz-grid", "p.json", "--x", "bone:arm_up:angle", "--y", "bone:arm_low:angle", "--res", "4", "link"]),
        ("export-fit output", vec![], "link", vec!["export-fit", "p.json", "link", "--max-size", "50KB", "--format", "project"]),
        ("export-fit --report", vec![], "link", vec!["export-fit", "p.json", "o.json", "--max-size", "50KB", "--format", "project", "--report", "link"]),
        ("lod --report", vec![], "link", vec!["lod", "p.json", "--tolerance-px", "0.5", "--levels", "2", "--report", "link"]),
        ("lod --out-dir file", vec![], "out/p_L1.json", vec!["lod", "p.json", "--tolerance-px", "0.5", "--levels", "2", "--out-dir", "out"]),
        ("ci run --junit", vec![], "link", vec!["ci", "run", "cim.json", "--update", "--junit", "link"]),
        ("ci run --html", vec![], "link", vec!["ci", "run", "cim.json", "--update", "--html", "link"]),
        ("ci run --json", vec![], "link", vec!["ci", "run", "cim.json", "--update", "--json", "link"]),
        ("face-synth files", vec![], "fsd/params.json", vec!["face-synth", "fsd", "--frames", "2", "--size", "32"]),
        ("face-calibrate", vec![vec!["face-synth", "fs", "--frames", "40", "--size", "32"]], "link", vec!["face-calibrate", "fs/truth.ndjson", "link"]),
        ("face-detect --overlay", vec![vec!["face-synth", "fs", "--frames", "2", "--size", "32"]], "link", vec!["face-detect", "fs/000000.png", "--overlay", "link"]),
        ("pose-db build", vec![], "link", vec!["pose-db", "build", "link", "--project", "p.json"]),
        ("pose-db train-mlp", vec![vec!["pose-db", "build", "db.json", "--project", "p.json", "--step", "1"]], "link", vec!["pose-db", "train-mlp", "db.json", "link", "--epochs", "1"]),
        ("export .mp4 (ffmpeg writes it)", vec![], "x.mp4", vec!["export", "p.json", "x.mp4"]),
        ("export .webm (ffmpeg writes it)", vec![], "x.webm", vec!["export", "p.json", "x.webm"]),
        ("dof-export .mp4", vec![], "x.mp4", vec!["dof-export", "p.json", "x.mp4"]),
        ("export .png atlas", vec![], "sheet.atlas.json", vec!["export", "p.json", "sheet.png"]),
        ("export .gif", vec![], "x.gif", vec!["export", "p.json", "x.gif"]),
        ("export-oca", vec![], "oca/tiny.oca", vec!["export-oca", "p.json", "oca"]),
        ("render", vec![], "x.png", vec!["render", "p.json", "0", "x.png"]),
        ("golden write", vec![], "link", vec!["golden", "write", "p.json", "link"]),
        ("import-lottie effects file", vec![vec!["export", "p.json", "lot.json"]], "link", vec!["import-lottie", "lot.json", "imp.json", "link"]),
        ("studio --collab-log", vec![], "link", vec!["studio", "p.json", "--port", "0", "--ui-config", "ui.json", "--collab-log", "link"]),
    ];
    for (label, pre, plant, args) in cases {
        if (plant.ends_with(".mp4") || plant.ends_with(".webm")) && !have_ffmpeg() {
            eprintln!("SKIP {label}: ffmpeg is not on PATH");
            continue;
        }
        let d = setup("sym");
        fs::write(d.join("cim.json"), r#"{"schema":"osatura-ci/1","project":"p.json","goldens":"goldens","checks":[{"kind":"validate","name":"v"},{"kind":"render","name":"f","frames":[0,2]}]}"#).unwrap();
        for c in &pre {
            let o = run_in(&d, c);
            assert!(o.ok, "{label}: setup {c:?} failed: {}", o.all());
        }
        link(&d, plant);
        let o = run_limited(&d, &args, 60);
        assert!(!o.ok, "{label}: a symlinked output must be refused, but the command succeeded:\n{}", o.all());
        assert!(o.all().contains("symbolic link"), "{label}: the error must say why:\n{}", o.all());
        assert_eq!(fs::read_to_string(d.join("target.txt")).unwrap(), "precious", "{label}: the link's target was overwritten");
        assert!(fs::symlink_metadata(d.join(plant)).unwrap().file_type().is_symlink(), "{label}: the link itself was replaced");
    }
}

/// The golden store's index is written atomically too: a symlinked `goldens/index.json` is refused.
#[cfg(unix)]
#[test]
fn a_symlinked_golden_index_is_refused() {
    let d = setup("goldenidx");
    fs::write(d.join("cim.json"), r#"{"schema":"osatura-ci/1","project":"p.json","goldens":"goldens","checks":[{"kind":"render","name":"f","frames":[0]}]}"#).unwrap();
    assert!(run_in(&d, &["ci", "run", "cim.json", "--update"]).ok);
    let idx = fs::read_to_string(d.join("goldens/index.json")).unwrap();
    fs::write(d.join("idx_copy.json"), &idx).unwrap();
    fs::remove_file(d.join("goldens/index.json")).unwrap();
    std::os::unix::fs::symlink(d.join("idx_copy.json"), d.join("goldens/index.json")).unwrap();
    // a new frame needs a new golden, so the index has to be saved
    fs::write(d.join("cim.json"), r#"{"schema":"osatura-ci/1","project":"p.json","goldens":"goldens","checks":[{"kind":"render","name":"f","frames":[0,3]}]}"#).unwrap();
    let o = run_in(&d, &["ci", "run", "cim.json", "--update"]);
    assert!(!o.ok && o.all().contains("symbolic link"), "{}", o.all());
    assert_eq!(fs::read_to_string(d.join("idx_copy.json")).unwrap(), idx, "the link's target was not touched");
}

/// `--follow-symlinks` still works: the target is written, atomically, and the link stays a link.
#[cfg(unix)]
#[test]
fn follow_symlinks_opts_in_for_the_rerouted_writers() {
    for (label, args) in [
        ("fuzz --report", vec!["fuzz", "p.json", "--samples", "3", "--report", "link", "--follow-symlinks"]),
        ("lod --report", vec!["lod", "p.json", "--tolerance-px", "0.5", "--levels", "2", "--report", "link", "--follow-symlinks"]),
        ("ci run --json", vec!["ci", "run", "cim.json", "--update", "--json", "link", "--follow-symlinks"]),
    ] {
        let d = setup("follow");
        fs::write(d.join("cim.json"), r#"{"schema":"osatura-ci/1","project":"p.json","goldens":"goldens","checks":[{"kind":"validate","name":"v"}]}"#).unwrap();
        link(&d, "link");
        let o = run_in(&d, &args);
        assert!(o.ok, "{label}: {}", o.all());
        let t = fs::read_to_string(d.join("target.txt")).unwrap();
        assert_ne!(t, "precious", "{label}: the target should have been written");
        assert!(fs::symlink_metadata(d.join("link")).unwrap().file_type().is_symlink(), "{label}: still a link");
    }
}

/// Item 1 (policy): commands that write several files create their folders; `fuzz --report` does not and says so.
#[test]
fn parent_folder_policy_is_as_documented() {
    let d = setup("mkdir");
    let o = run_in(&d, &["lod", "p.json", "--tolerance-px", "0.5", "--levels", "2", "--out-dir", "deep/er", "--report", "rep/lod.json"]);
    assert!(o.ok, "{}", o.all());
    assert!(d.join("deep/er/p_L1.json").exists() && d.join("rep/lod.json").exists(), "lod creates its folders");
    let o = run_in(&d, &["fuzz", "p.json", "--samples", "3", "--report", "missing/dir/r.json"]);
    assert!(!o.ok && o.all().contains("missing/dir"), "fuzz does not create folders: {}", o.all());
}

/// Item 2: `golden check` is exact by default and says how many cells / how much differs.
#[test]
fn golden_check_is_exact_by_default_and_tol_loosens_it() {
    let d = setup("golden");
    assert!(run_in(&d, &["golden", "write", "p.json", "g.json"]).ok);
    let o = run_in(&d, &["golden", "check", "p.json", "g.json"]);
    assert!(o.ok && o.stdout.contains("match exactly"), "{}", o.all());
    // 50 more degrees on the skinned bone's last key
    let mut v: serde_json::Value = serde_json::from_str(&tiny_json()).unwrap();
    v["channels"][0]["track"]["keys"][1]["value"] = serde_json::json!(70);
    fs::write(d.join("q.json"), v.to_string()).unwrap();
    let o = run_in(&d, &["golden", "check", "q.json", "g.json"]);
    assert!(!o.ok, "a 50 degree change must fail: {}", o.all());
    assert!(o.stdout.contains("differs") && o.stdout.contains("thumbnail cells") && o.stdout.contains("max channel difference"), "{}", o.all());
    assert!(o.stderr.contains("exact comparison"), "{}", o.all());
    // a tiny colour change: fails exactly, passes under a loose tolerance
    let mut v: serde_json::Value = serde_json::from_str(&tiny_json()).unwrap();
    v["layers"][0]["paths"][0]["fill"] = serde_json::json!("#336698");
    fs::write(d.join("c.json"), v.to_string()).unwrap();
    assert!(!run_in(&d, &["golden", "check", "c.json", "g.json"]).ok, "one colour step must fail the exact check");
    let o = run_in(&d, &["golden", "check", "c.json", "g.json", "--tol", "0.01"]);
    assert!(o.ok && o.stdout.contains("tolerance"), "{}", o.all());
    assert!(!run_in(&d, &["golden", "check", "c.json", "g.json", "--tol", "-1"]).ok);
}

/// Item 7 and 8: unknown flags are refused by commands whose usage lists no option; `--json` belongs to its commands.
#[test]
fn stray_flags_are_refused_everywhere() {
    let d = setup("flags");
    for args in [vec!["check", "p.json"], vec!["stats", "p.json"], vec!["lint", "p.json"], vec!["coach", "p.json"], vec!["machine", "list", "p.json"]] {
        assert!(run_in(&d, &args).ok, "{args:?} without a stray flag must work");
        let mut v = args.clone();
        v.push("--foo");
        let o = run_in(&d, &v);
        assert!(!o.ok && o.all().contains("unknown option --foo"), "{v:?}: {}", o.all());
    }
    let o = run_in(&d, &["stats", "p.json", "--json"]);
    assert!(!o.ok && o.all().contains("unknown option --json"), "stats prints JSON always; the flag changed nothing and is gone: {}", o.all());
    let o = run_in(&d, &["stats", "p.json"]);
    assert!(serde_json::from_str::<serde_json::Value>(&o.stdout).is_ok(), "stats output is JSON: {}", o.stdout);
    assert!(run_in(&d, &["timeline", "p.json", "--json"]).ok, "--json stays valid where it is documented");
}

/// Item 7: `import-lottie` never overwrites its input.
#[test]
fn import_lottie_refuses_to_overwrite_its_input() {
    let d = setup("lottie");
    assert!(run_in(&d, &["export", "p.json", "lot.json"]).ok);
    let before = fs::read_to_string(d.join("lot.json")).unwrap();
    let o = run_in(&d, &["import-lottie", "lot.json", "lot.json"]);
    assert!(!o.ok && o.all().contains("refusing to overwrite the input"), "{}", o.all());
    assert_eq!(fs::read_to_string(d.join("lot.json")).unwrap(), before);
    let o = run_in(&d, &["import-lottie", "lot.json", "imp.json", "lot.json"]);
    assert!(!o.ok && o.all().contains("refusing to overwrite the input"), "effects file may not be the input either: {}", o.all());
    assert!(run_in(&d, &["import-lottie", "lot.json", "imp.json", "fx.json"]).ok);
}

/// Item 6: the sprite-sheet layout is `<name>.atlas.json`, so it cannot overwrite a Lottie file of the same stem; the
/// unknown-format message lists every extension.
#[test]
fn export_png_atlas_has_its_own_name_and_the_format_error_lists_everything() {
    let d = setup("atlas");
    assert!(run_in(&d, &["export", "p.json", "sheet.json"]).ok);
    let lottie = fs::read_to_string(d.join("sheet.json")).unwrap();
    let o = run_in(&d, &["export", "p.json", "sheet.png"]);
    assert!(o.ok, "{}", o.all());
    assert_eq!(fs::read_to_string(d.join("sheet.json")).unwrap(), lottie, "the Lottie file of the same stem is untouched");
    let atlas: serde_json::Value = serde_json::from_str(&fs::read_to_string(d.join("sheet.atlas.json")).unwrap()).unwrap();
    assert!(atlas.is_object());
    assert!(o.stdout.contains("sheet.atlas.json"));
    let o = run_in(&d, &["export", "p.json", "x.bmp"]);
    assert!(!o.ok);
    for ext in [".gif", ".html", ".json", ".lottie", ".mp4", ".apng", ".webm", ".png", ".glb"] {
        assert!(o.all().contains(ext), "unknown-format message must list {ext}: {}", o.all());
    }
    let o = run_in(&d, &["export", "p.json", "noext"]);
    assert!(!o.ok && o.all().contains("no file extension"), "{}", o.all());
}

/// Item 8: `fuzz` is a report unless a gate is asked for.
#[test]
fn fuzz_exits_zero_as_a_report_and_one_as_a_gate() {
    // a rig that fails often: a bone with an angle limit the fuzzer breaks is not needed, the demo's IK is enough
    let d = osatura_core::fsutil::ScratchDir::new("osatura-docfix-fuzz");
    assert!(run_in(&d, &["demo", "demo.json"]).ok);
    let o = run_in(&d, &["fuzz", "demo.json", "--samples", "40", "--strategy", "lhs"]);
    assert!(o.ok, "report only: exit 0 whatever the failure rate: {}", o.all());
    assert!(o.stdout.contains("failures") && o.stdout.contains("report only"), "{}", o.stdout);
    let o = run_in(&d, &["fuzz", "demo.json", "--samples", "40", "--strategy", "lhs", "--gate"]);
    assert!(!o.ok && o.all().contains("exceeds --max-fail-rate 0.00%"), "{}", o.all());
    let o = run_in(&d, &["fuzz", "demo.json", "--samples", "40", "--strategy", "lhs", "--max-fail-rate", "1"]);
    assert!(o.ok, "a ceiling of 100% never trips: {}", o.all());
}

/// Item 9: `render` of a frame outside the animation is an error; `--clamp` opts into the nearest frame.
#[test]
fn render_frame_out_of_range_is_an_error_unless_clamped() {
    let d = setup("frame");
    let o = run_in(&d, &["render", "p.json", "4", "o.png"]);
    assert!(!o.ok && o.all().contains("outside the animation") && o.all().contains("0..3") && o.all().contains("--clamp"), "{}", o.all());
    assert!(!d.join("o.png").exists(), "nothing is written on the error");
    assert!(!run_in(&d, &["render", "p.json", "-1", "o.png"]).ok);
    assert!(!run_in(&d, &["render", "p.json", "NaN", "o.png"]).ok);
    assert!(run_in(&d, &["render", "p.json", "3", "last.png"]).ok, "the last frame is fine");
    let o = run_in(&d, &["render", "p.json", "99", "c.png", "--clamp"]);
    assert!(o.ok && o.stderr.contains("renders frame 3"), "{}", o.all());
    assert_eq!(fs::read(d.join("c.png")).unwrap(), fs::read(d.join("last.png")).unwrap(), "clamped to the last frame");
}

/// Item 10: take-apply names a missing library and refuses stray words.
#[test]
fn take_apply_is_strict_about_library_and_tokens() {
    let d = setup("take");
    let o = run_in(&d, &["take-apply", "p.json", "o.json", "nolib.json", "t", "0"]);
    assert!(!o.ok && o.all().contains("nolib.json"), "the error must name the library file: {}", o.all());
    assert!(run_in(&d, &["take-record", "p.json", "lib.json", "t", "0", "3"]).ok);
    assert!(run_in(&d, &["take-apply", "p.json", "o.json", "lib.json", "t", "0"]).ok);
    assert!(run_in(&d, &["take-apply", "p.json", "o.json", "lib.json", "t", "0", "-", "2"]).ok, "- means no renames, then the speed");
    let o = run_in(&d, &["take-apply", "p.json", "o2.json", "lib.json", "t", "0", "speed=1"]);
    assert!(!o.ok && o.all().contains("speed=1"), "{}", o.all());
    assert!(!d.join("o2.json").exists());
    let o = run_in(&d, &["take-apply", "p.json", "o2.json", "lib.json", "t", "0", "garbage"]);
    assert!(!o.ok && o.all().contains("not a rename"), "{}", o.all());
    let o = run_in(&d, &["take-apply", "p.json", "o2.json", "lib.json", "t", "0", "arm_up=arm_low"]);
    assert!(o.ok, "a real rename works: {}", o.all());
    let o = run_in(&d, &["take-apply", "p.json", "o2.json", "lib.json", "t", "0", "-", "fast"]);
    assert!(!o.ok && o.all().contains("bad speed 'fast'"), "{}", o.all());
}

/// Item 10: --collab-nosync needs --collab-log.
#[test]
fn collab_nosync_without_a_log_is_an_error() {
    let d = setup("nosync");
    let o = run_limited(&d, &["studio", "p.json", "--port", "0", "--collab-nosync"], 20);
    assert!(!o.ok && o.all().contains("--collab-log"), "{}", o.all());
}

/// Item 10: mirror and script say when they did nothing.
#[test]
fn mirror_and_script_warn_when_there_is_nothing_to_do() {
    let d = setup("mirror");
    let o = run_in(&d, &["mirror", "p.json", "m.json"]);
    assert!(o.ok, "exit 0: {}", o.all());
    assert!(o.stderr.contains("no channel target has a left/right name") && o.stderr.contains("only the sign"), "{}", o.stderr);
    assert!(o.stdout.contains("0 of 2 channel target(s) swapped"), "{}", o.stdout);
    // a sided project swaps without a warning
    let mut v: serde_json::Value = serde_json::from_str(&tiny_json()).unwrap();
    v["bones"] = serde_json::json!([
        {"name": "arm_L", "length": 10, "rest_offset": [8, 12], "rest_angle": 0},
        {"name": "arm_R", "length": 10, "rest_offset": [24, 12], "rest_angle": 0}
    ]);
    v["layers"] = serde_json::json!([]);
    v["channels"] = serde_json::json!([v["channels"][0].clone(), v["channels"][1].clone()]);
    v["channels"][0]["target"] = serde_json::json!("arm_L");
    v["channels"][1]["target"] = serde_json::json!("arm_R");
    fs::write(d.join("s.json"), v.to_string()).unwrap();
    let o = run_in(&d, &["mirror", "s.json", "m2.json"]);
    assert!(o.ok && !o.stderr.contains("warning"), "{}", o.all());
    assert!(o.stdout.contains("2 of 2 channel target(s) swapped"), "{}", o.stdout);
    // no channels at all
    let mut v: serde_json::Value = serde_json::from_str(&tiny_json()).unwrap();
    v["channels"] = serde_json::json!([]);
    fs::write(d.join("e.json"), v.to_string()).unwrap();
    let o = run_in(&d, &["mirror", "e.json", "m3.json"]);
    assert!(o.ok && o.stderr.contains("no channels"), "{}", o.all());
    // script with no requests (only if this build has scripting)
    fs::write(d.join("nothing.rhai"), "let n = frames();").unwrap();
    let o = run_in(&d, &["script", "p.json", "s_out.json", "nothing.rhai"]);
    if o.ok {
        assert!(o.stderr.contains("0 requests were applied"), "{}", o.all());
    }
}

/// Item 10: face-bake reports the keys of the channels it created, and tells when the landmarks outlast the project.
#[test]
fn face_bake_counts_the_keys_it_made_and_reports_cut_frames() {
    let d = setup("facebake");
    assert!(run_in(&d, &["face-synth", "fs", "--frames", "60", "--size", "32", "--fps", "30"]).ok || run_in(&d, &["face-synth", "fs", "--frames", "60", "--size", "32"]).ok);
    // map the synthetic mouth opening onto a bone angle
    fs::write(
        d.join("map.json"),
        r#"{"params":[{"source":"face","id":"mouth_open","target":"arm_low","prop":"bone_angle","in_min":0,"in_max":1,"out_min":0,"out_max":40}]}"#,
    )
    .unwrap();
    let o = run_in(&d, &["face-bake", "p.json", "fs/truth.ndjson", "map.json", "baked.json", "--calibrate-frames", "0", "--fps", "30", "--allow-empty"]);
    assert!(o.ok, "{}", o.all());
    assert!(o.stdout.contains("+1 channels (") && o.stdout.contains("keys in them)"), "{}", o.stdout);
    assert!(!o.stdout.contains("0 keys set,"), "the old message said '0 keys set' for a keyed channel: {}", o.stdout);
    assert!(o.stderr.contains("longer than the project"), "60 landmark frames at 30 fps do not fit 4 frames at 12 fps: {}", o.stderr);
}

/// Item 10: mocap-retarget says when it extends the project, and --keep-length keeps the length.
#[test]
fn mocap_retarget_reports_a_length_change_and_honours_keep_length() {
    let d = setup("mocap");
    // a two-joint BVH, 40 frames at 12 fps
    let mut bvh = String::from("HIERARCHY\nROOT arm_up\n{\n OFFSET 0 0 0\n CHANNELS 3 Zrotation Xrotation Yrotation\n JOINT arm_low\n {\n  OFFSET 10 0 0\n  CHANNELS 3 Zrotation Xrotation Yrotation\n  End Site\n  {\n   OFFSET 10 0 0\n  }\n }\n}\nMOTION\nFrames: 40\nFrame Time: 0.083333\n");
    for i in 0..40 {
        bvh.push_str(&format!("{} 0 0 {} 0 0\n", i as f32, -(i as f32)));
    }
    fs::write(d.join("m.bvh"), bvh).unwrap();
    fs::write(d.join("map.json"), r#"{"arm_up": "arm_up", "arm_low": "arm_low"}"#).unwrap();
    let o = run_in(&d, &["mocap-retarget", "p.json", "m.bvh", "ext.json", "--map", "map.json"]);
    assert!(o.ok, "{}", o.all());
    assert!(o.stdout.contains("the project length was extended from 4 to 39 frames"), "{}", o.stdout);
    let ext: serde_json::Value = serde_json::from_str(&fs::read_to_string(d.join("ext.json")).unwrap()).unwrap();
    assert_eq!(ext["frames"], 39);
    let o = run_in(&d, &["mocap-retarget", "p.json", "m.bvh", "kept.json", "--map", "map.json", "--keep-length"]);
    assert!(o.ok, "{}", o.all());
    let kept: serde_json::Value = serde_json::from_str(&fs::read_to_string(d.join("kept.json")).unwrap()).unwrap();
    assert_eq!(kept["frames"], 4, "--keep-length keeps the project length");
    assert!(o.stdout.contains("--keep-length") && !o.stdout.contains("was extended"), "{}", o.stdout);
    let max = kept["channels"].as_array().unwrap().iter().flat_map(|c| c["track"]["keys"].as_array().into_iter().flatten()).filter_map(|k| k["frame"].as_f64()).fold(0.0, f64::max);
    assert!(max <= 3.0, "no key beyond the last frame: {max}");
}

/// Items 3-5: the editor-data warnings of `check`; the fixtures under out/ still load without a new error.
#[test]
fn check_warns_about_editor_data_and_old_fixtures_still_load() {
    let d = setup("editor");
    let mut v: serde_json::Value = serde_json::from_str(&tiny_json()).unwrap();
    v["camera"] = serde_json::json!({"zoom": 0.0});
    v["layers"][0]["opacity"] = serde_json::json!(1.5);
    v["skins"] = serde_json::json!([{"name": "red", "group": "outfit", "layers": ["ghost"]}]);
    v["events"] = serde_json::json!([{"name": "late", "frame": 99}]);
    fs::write(d.join("w.json"), v.to_string()).unwrap();
    let o = run_in(&d, &["check", "w.json"]);
    assert!(o.ok, "warnings never reject a project: {}", o.all());
    for needle in ["warning: camera: zoom is 0", "opacity 1.5 is outside 0..1", "layer 'ghost' does not exist", "event #0 'late'"] {
        assert!(o.stdout.contains(needle), "missing {needle:?} in:\n{}", o.stdout);
    }
    let o = run_in(&d, &["lint", "w.json"]);
    assert!(o.stdout.contains("L013"), "lint reports the same findings: {}", o.all());
    // every project in out/ (when present) keeps loading, without warnings from the new checks
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("out");
    let mut checked = 0;
    if let Ok(rd) = fs::read_dir(&root) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("json") && !p.to_string_lossy().ends_with(".lottie.json") {
                let text = fs::read_to_string(&p).unwrap_or_default();
                if !text.contains("\"layers\"") || !text.contains("\"frames\"") {
                    continue;
                }
                let o = run_in(&d, &["check", p.to_str().unwrap()]);
                assert!(o.ok, "{} no longer loads: {}", p.display(), o.all());
                assert!(!o.stdout.contains("warning"), "{}: {}", p.display(), o.stdout);
                checked += 1;
            }
        }
    }
    eprintln!("checked {checked} fixture(s) under out/");
}

/// Item 11: the studio answers /favicon.ico (also behind a token), and the cookie is Secure only over HTTPS.
#[test]
fn studio_serves_a_favicon_and_marks_the_cookie_secure_over_https() {
    let d = setup("studio");
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let mut child = Command::new(BIN)
        .current_dir(&d)
        .args(["studio", "p.json", "--port", &port.to_string(), "--ui-config", "ui.json", "--collab-token", "0123456789abcdef-token", "--public-origin", "https://studio.example.org"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    struct Kill<'a>(&'a mut std::process::Child);
    impl Drop for Kill<'_> {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let _k = Kill(&mut child);
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let req = |method: &str, path: &str, extra: &str, body: &str| -> String {
        let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        write!(s, "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n{extra}Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        let mut out = String::new();
        let _ = s.read_to_string(&mut out);
        out
    };
    let f = req("GET", "/favicon.ico", "", "");
    assert!(f.starts_with("HTTP/1.1 200") && f.contains("image/svg+xml") && f.contains("<svg"), "{f}");
    let login = |extra: &str| req("POST", "/api/login", &format!("Content-Type: application/json\r\n{extra}"), r#"{"token":"0123456789abcdef-token"}"#);
    let plain = login("");
    assert!(plain.contains("HttpOnly") && plain.contains("SameSite=Strict") && !plain.contains("Secure"), "{plain}");
    let tls = login("X-Forwarded-Proto: https\r\n");
    assert!(tls.contains("HttpOnly") && tls.contains("SameSite=Strict") && tls.contains("; Secure"), "{tls}");
    let page = req("GET", "/", "Cookie: osatura_token=0123456789abcdef-token\r\n", "");
    assert!(page.contains("synthetic (stylised) faces only") && !page.contains("contains <b>no face detector</b>"), "the face panel text names the synthetic detector");
    assert!(page.contains("rel=\"icon\""), "the page links its icon");
}

// SPDX-License-Identifier: AGPL-3.0-or-later
#[test]
fn vcs_accepts_json_flag() {
    let exe = env!("CARGO_BIN_EXE_osatura");
    let d = std::env::temp_dir().join(format!("arm_vcsjson_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let p = d.join("p.json");
    let o = std::process::Command::new(exe).args(["demo", p.to_str().unwrap()]).output().unwrap();
    let _ = o;
    assert!(p.exists());
    std::process::Command::new(exe).args(["vcs", "init", p.to_str().unwrap()]).output().unwrap();
    let o = std::process::Command::new(exe).args(["vcs", "status", p.to_str().unwrap(), "--json"]).output().unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!err.contains("unknown option"), "{err}");
    let _ = std::fs::remove_dir_all(&d);
}

/// Acceptance matrix for the per-command options `--json`, `--quiet` and `--allow-empty`: every command (and sub-form)
/// that documents one accepts it (it is never reported as an unknown option; the command may still fail later on its
/// own inputs, which the dummy files here are not meant to satisfy), and a sample of commands, sibling sub-forms and
/// sibling forms that do not document it refuse it. The refusal happens before any file is read.
#[test]
fn json_quiet_allow_empty_acceptance_matrix() {
    let d = setup("matrix");
    let accepted: &[&[&str]] = &[
        // --json
        &["vcs", "log", "--json"],
        &["vcs", "status", "--json"],
        &["vcs", "diff", "--json"],
        &["vcs", "show", "--json"],
        &["ui", "commands", "--json"],
        &["ui", "shortcuts", "--json"],
        &["face-detect", "nofile.png", "--json"],
        &["pose-suggest", "between", "p.json", "-", "--a", "0", "--b", "1", "--json"],
        &["pick", "p.json", "--frame", "0", "--x", "1", "--y", "1", "--json"],
        &["select-box", "p.json", "--frame", "0", "--rect", "0,0,5,5", "--json"],
        &["timeline", "p.json", "--json"],
        &["trail", "p.json", "--target", "bone:arm_up", "--json"],
        &["size-report", "p.json", "--json"],
        &["collab-sim", "p.json", "--clients", "2", "--ops", "2", "--json"],
        &["ci", "run", "--json", "out.json"],
        // --quiet
        &["ci", "run", "--quiet"],
        // --allow-empty
        &["physics", "p.json", "o.json", "arm_up", "--allow-empty"],
        &["live-map", "p.json", "ev.json", "map.json", "o.json", "--allow-empty"],
        &["face-bake", "p.json", "lm.ndjson", "map.json", "o.json", "--allow-empty"],
    ];
    for args in accepted {
        let o = run_in(&d, args);
        assert!(!o.all().contains("unknown option"), "{args:?} must accept its documented option: {}", o.all());
    }
    let refused: &[&[&str]] = &[
        // commands that do not document the option
        &["render", "p.json", "0", "o.png", "--json"],
        &["check", "p.json", "--quiet"],
        &["stats", "p.json", "--allow-empty"],
        &["lint", "p.json", "--json"],
        &["export", "p.json", "o.gif", "--json"],
        &["frames", "p.json", "outdir", "--quiet"],
        &["fuzz", "p.json", "--quiet"],
        &["golden", "write", "p.json", "g.json", "--json"],
        &["golden", "check", "p.json", "g.json", "--quiet"],
        // sibling sub-forms of commands that do document it
        &["ci", "diff", "a.png", "b.png", "--quiet"],
        &["ci", "init", "p.json", "--quiet"],
        &["ci", "bless", "--json", "x.json"],
        &["ci", "bless", "--allow-empty"],
        &["trail", "p.json", "arm_up", "t.svg", "--json"],
        &["ui", "layout", "presets", "--json"],
        &["ui", "search", "move", "--json"],
        &["vcs", "commit", "-m", "x", "--json"],
        &["vcs", "checkout", "main", "--json"],
        &["vcs", "gc", "--json"],
        &["pose-suggest", "next", "p.json", "-", "--frame", "1", "--json"],
        &["pose-suggest", "complete", "p.json", "-", "--frame", "1", "--json"],
        &["pose-db", "info", "db.json", "--json"],
        &["physics", "p.json", "o.json", "arm_up", "--quiet"],
        &["live-map", "p.json", "ev.json", "map.json", "o.json", "--json"],
        &["face-bake", "p.json", "lm.ndjson", "map.json", "o.json", "--json"],
        &["face-detect", "nofile.png", "--quiet"],
    ];
    for args in refused {
        let o = run_in(&d, args);
        assert!(!o.ok && o.all().contains("unknown option"), "{args:?} must refuse the option: {}", o.all());
    }
}

/// `golden write --scale` above 1 is an error (it was clamped silently); `golden check` has no `--scale`.
#[test]
fn golden_scale_above_one_is_refused_and_check_has_no_scale() {
    let d = setup("goldenscale");
    let o = run_in(&d, &["golden", "write", "p.json", "g.json", "--scale", "2"]);
    assert!(!o.ok && o.all().contains("above 1") && !d.join("g.json").exists(), "{}", o.all());
    assert!(run_in(&d, &["golden", "write", "p.json", "g.json", "--scale", "0.5"]).ok);
    assert!(run_in(&d, &["golden", "write", "p.json", "g1.json", "--scale", "1"]).ok);
    let o = run_in(&d, &["golden", "check", "p.json", "g.json", "--scale", "0.5"]);
    assert!(!o.ok && o.all().contains("unknown option --scale for 'golden check'"), "{}", o.all());
    let o = run_in(&d, &["golden", "check", "p.json", "g.json"]);
    assert!(o.ok && o.stdout.contains("match exactly"), "{}", o.all());
    let o = run_in(&d, &["golden", "write", "p.json", "g2.json", "--tol", "0.1"]);
    assert!(!o.ok && o.all().contains("unknown option --tol for 'golden write'"), "{}", o.all());
}
