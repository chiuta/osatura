// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Regression tests for the wave-4 review: asset paths on write (H1), LOD bound with layout containers (H2),
//! per-command option table (H3), `--key=value`, sub-command `--help`, input caps and friends. Each of them
//! fails on the behaviour that was reported.

use osatura_core::fsutil::ScratchDir;
use std::fs;
use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_osatura");

fn run_in(dir: &Path, args: &[&str]) -> (bool, String) {
    let o = Command::new(BIN).current_dir(dir).args(args).output().expect("cannot start osatura");
    (o.status.success(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn write_png(path: &Path) {
    let f = fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), 4, 4);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().unwrap();
    w.write_image_data(&[200u8; 4 * 4 * 4]).unwrap();
}

/// `<dir>/pr/p.json` with an image layer whose file `img.png` sits next to it.
fn image_project(dir: &Path) {
    fs::create_dir_all(dir.join("pr")).unwrap();
    write_png(&dir.join("pr/img.png"));
    let mut p = osatura_core::Project::new("img", 64.0, 64.0, 24.0, 12);
    p.layers.push(osatura_core::project::Layer::image(
        "pic",
        osatura_core::project::ImageSource { file: Some("img.png".into()), png_base64: None },
        osatura_core::math::Vec2::ZERO,
    ));
    fs::write(dir.join("pr/p.json"), p.to_json_pretty()).unwrap();
}

/// H1: the output of a command is validated against the INPUT project's directory, not the current one.
#[test]
fn h1_mirror_from_the_parent_directory_keeps_working() {
    let s = ScratchDir::new("osatura-w4-h1");
    image_project(&s);
    let (ok, t) = run_in(&s, &["mirror", "pr/p.json", "pr/o.json"]);
    assert!(ok, "old behaviour: 'output would be invalid: ... img.png' -- {t}");
    assert!(run_in(&s, &["check", "pr/o.json"]).0);
}

#[test]
fn h1_output_in_a_parent_directory_gets_rewritten_paths() {
    let s = ScratchDir::new("osatura-w4-h1b");
    image_project(&s);
    let (ok, t) = run_in(&s, &["mirror", "pr/p.json", "o.json"]);
    assert!(ok, "{t}");
    let text = fs::read_to_string(s.join("o.json")).unwrap();
    assert!(text.contains("pr/img.png"), "asset path not rewritten: {text}");
    let (ok, t) = run_in(&s, &["check", "o.json"]);
    assert!(ok, "rewritten output must not dangle: {t}");
}

#[test]
fn h1_output_in_a_sibling_directory_warns() {
    let s = ScratchDir::new("osatura-w4-h1c");
    image_project(&s);
    fs::create_dir_all(s.join("other")).unwrap();
    let (ok, t) = run_in(&s.join("pr"), &["mirror", "p.json", "../other/o.json"]);
    assert!(ok, "{t}");
    assert!(t.contains("warning") && t.contains("img.png") && t.contains("will not find it"), "no warning about the dangling asset: {t}");
}

/// A tiny project (no bones: `ci init` is fast on it) at `<dir>/d.json`.
fn tiny(dir: &Path) -> String {
    let mut p = osatura_core::Project::new("tiny", 32.0, 32.0, 24.0, 4);
    p.layers.push(osatura_core::project::Layer::vector("sq", vec![osatura_core::path::VPath::rect("r", 2.0, 2.0, 20.0, 20.0).with_fill("#336699")]));
    let path = dir.join("d.json");
    fs::write(&path, p.to_json_pretty()).unwrap();
    path.to_string_lossy().into_owned()
}

/// H3: a boolean `--prune` of `ci run` in last position, and a value `--prune N` for autotrace, from one table.
#[test]
fn h3_ci_run_prune_as_last_argument() {
    let s = ScratchDir::new("osatura-w4-h3");
    tiny(&s);
    let (ok, t) = run_in(&s, &["ci", "init", "d.json"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["ci", "run", "--prune"]);
    assert!(!t.contains("needs a value"), "old behaviour: --prune needs a value -- {t}");
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["ci", "run", "--prune", "--quiet"]);
    assert!(ok, "{t}");
}

#[test]
fn ci_accepts_global_flags_and_eq_form_after_the_subcommand() {
    let s = ScratchDir::new("osatura-w4-ci");
    tiny(&s);
    assert!(run_in(&s, &["ci", "init", "d.json"]).0);
    let (ok, t) = run_in(&s, &["ci", "run", "--allow-external-assets"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["ci", "run", "--junit=j.xml"]);
    assert!(ok, "{t}");
    assert!(s.join("j.xml").exists());
    let (ok, t) = run_in(&s, &["ci", "run", "--prune=yes"]);
    assert!(!ok && t.contains("--prune is a flag and takes no value"), "{t}");
}

#[test]
fn eq_form_and_subcommand_help() {
    let s = ScratchDir::new("osatura-w4-eq");
    let p = s.join("d.json").to_string_lossy().into_owned();
    assert!(run_in(&s, &["demo", &p]).0);
    let (ok, t) = run_in(&s, &["render", "d.json", "3", "o.png", "--scale=0.5"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["render", "d.json", "3", "o.png", "--bones=1"]);
    assert!(!ok && t.contains("--bones is a flag"), "{t}");
    let (ok, t) = run_in(&s, &["render", "--help"]);
    assert!(ok && t.starts_with("USAGE:") && t.contains("osatura render"), "{t}");
    let (ok, t) = run_in(&s, &["render", "--version"]);
    assert!(ok && t.starts_with("osatura "), "{t}");
}

// ---- M5: output into a directory that cannot hold a temporary file, symbolic links ---------------------------

#[cfg(unix)]
fn is_root() -> bool {
    Command::new("id").arg("-u").output().map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0").unwrap_or(false)
}

/// The binary run as an unprivileged user (`nobody` through setpriv when the tests run as root); `None` when that is
/// not possible here (then the test says so and passes vacuously, it never pretends).
#[cfg(unix)]
fn unprivileged(dir: &Path, args: &[&str]) -> Option<(bool, String)> {
    let o = if is_root() {
        if Command::new("setpriv").arg("--help").output().is_err() {
            eprintln!("setpriv missing: unprivileged test skipped");
            return None;
        }
        Command::new("setpriv").args(["--reuid=65534", "--regid=65534", "--clear-groups", BIN]).args(args).current_dir(dir).output().ok()?
    } else {
        Command::new(BIN).args(args).current_dir(dir).output().ok()?
    };
    Some((o.status.success(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))))
}

#[cfg(unix)]
fn chmod(p: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(p, fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(unix)]
#[test]
fn m5_write_into_a_read_only_directory_falls_back_to_overwriting_the_writable_file() {
    let s = ScratchDir::new("osatura-w4-m5");
    chmod(&s, 0o755);
    let input = tiny(&s);
    let out = s.join("out.json");
    fs::write(&out, "old").unwrap();
    chmod(&out, 0o666);
    chmod(&s, 0o555); // nobody can create files here, but may write out.json
    let r = unprivileged(&s, &["mirror", &input, &out.to_string_lossy()]);
    chmod(&s, 0o755);
    let Some((ok, t)) = r else { return };
    assert!(ok, "old behaviour: 'cannot create temporary file: Permission denied' -- {t}");
    assert!(fs::read_to_string(&out).unwrap().contains("\"name\""), "file was rewritten in place");
    // a file that is not writable either still fails with a clear message
    chmod(&out, 0o444);
    chmod(&s, 0o555);
    let r = unprivileged(&s, &["mirror", &input, &out.to_string_lossy()]);
    chmod(&s, 0o755);
    let (ok, t) = r.unwrap();
    assert!(!ok && t.contains("not writable"), "{t}");
}

#[cfg(unix)]
#[test]
fn m5_symbolic_link_outputs_are_refused_with_the_opt_in_named() {
    let s = ScratchDir::new("osatura-w4-m5l");
    let input = tiny(&s);
    let real = s.join("real.json");
    fs::write(&real, "old").unwrap();
    std::os::unix::fs::symlink(&real, s.join("link.json")).unwrap();
    let (ok, t) = run_in(&s, &["mirror", &input, "link.json"]);
    assert!(!ok && t.contains("--follow-symlinks") && t.contains("real.json"), "{t}");
    assert_eq!(fs::read_to_string(&real).unwrap(), "old");
    let (ok, t) = run_in(&s, &["mirror", &input, "link.json", "--follow-symlinks"]);
    assert!(ok, "{t}");
    assert!(fs::symlink_metadata(s.join("link.json")).unwrap().file_type().is_symlink(), "the link itself is kept");
    assert!(fs::read_to_string(&real).unwrap().contains("\"name\""));
}

/// H1 also holds for the other commands that read a project from another directory.
#[test]
fn h1_other_commands_resolve_assets_against_the_input_directory() {
    let s = ScratchDir::new("osatura-w4-h1d");
    image_project(&s);
    for args in [
        vec!["size-report", "pr/p.json"],
        vec!["export-oca", "pr/p.json", "oca"],
        vec!["lod", "pr/p.json", "--tolerance-px", "1", "--levels", "2", "--out-dir", "lodo"],
    ] {
        let (ok, t) = run_in(&s, &args);
        assert!(ok && !t.contains("img.png"), "old behaviour: asset looked up in the current directory: {args:?} -- {t}");
    }
}

#[cfg(unix)] // uses /dev/zero
#[test]
fn input_size_cap_and_special_files_are_refused() {
    let s = ScratchDir::new("osatura-w4-cap");
    let d = tiny(&s);
    let (ok, t) = run_in(&s, &["--max-input-bytes", "20", "check", &d]);
    assert!(!ok && t.contains("max-input-bytes"), "{t}");
    let (ok, t) = run_in(&s, &["check", "/dev/zero"]);
    assert!(!ok && t.contains("regular file"), "{t}");
    assert!(run_in(&s, &["check", &d]).0);
}

#[test]
fn dof_export_checks_the_output_directory_before_rendering() {
    let s = ScratchDir::new("osatura-w4-dof");
    let d = tiny(&s);
    let t0 = std::time::Instant::now();
    let (ok, t) = run_in(&s, &["dof-export", &d, "no/such/dir/o.gif"]);
    assert!(!ok && t.contains("no/such/dir"), "{t}");
    assert!(t0.elapsed().as_secs_f64() < 5.0);
}

#[test]
fn pick_tolerance_must_be_a_finite_non_negative_number() {
    let s = ScratchDir::new("osatura-w4-pick");
    let d = tiny(&s);
    for bad in ["-1", "NaN"] {
        let (ok, t) = run_in(&s, &["pick", &d, "--frame", "0", "--x", "5", "--y", "5", "--tol", bad]);
        assert!(!ok && t.contains("--tol"), "{bad}: {t}");
    }
    assert!(run_in(&s, &["pick", &d, "--frame", "0", "--x", "5", "--y", "5", "--tol", "2"]).0);
}

#[test]
fn fuzz_rejects_unknown_bones_and_frames_out_of_range() {
    let s = ScratchDir::new("osatura-w4-fuzz");
    let p = s.join("d.json").to_string_lossy().into_owned();
    assert!(run_in(&s, &["demo", &p]).0);
    let (ok, t) = run_in(&s, &["fuzz", "d.json", "--samples", "2", "--only", "no_such_bone"]);
    assert!(!ok && t.contains("no_such_bone"), "{t}");
    let (ok, t) = run_in(&s, &["fuzz", "d.json", "--samples", "2", "--frame", "99999"]);
    assert!(!ok && t.contains("frame"), "{t}");
}

/// M1: behind a TLS proxy the browser sends `Origin: https://<public host>`. Read the real port printed for `--port 0`.
#[test]
fn m1_studio_accepts_https_origin_of_an_allowed_host_only() {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpStream;
    let s = ScratchDir::new("osatura-w4-m1");
    let d = tiny(&s);
    let mut child = Command::new(BIN)
        .current_dir(&s)
        .args(["studio", &d, "--port", "0", "--allow-host", "studio.example.org"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut port = 0u16;
    for _ in 0..20 {
        let mut l = String::new();
        if out.read_line(&mut l).unwrap_or(0) == 0 {
            break;
        }
        if let Some(i) = l.find("127.0.0.1:") {
            port = l[i + 10..].trim().split(|c: char| !c.is_ascii_digit()).next().unwrap().parse().unwrap_or(0);
            if port != 0 {
                break;
            }
        }
    }
    assert!(port != 0, "--port 0 must print the real port");
    let ask = |origin: &str| -> String {
        let mut c = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let body = "{}";
        write!(c, "POST /api/ui/config HTTP/1.1\r\nHost: studio.example.org\r\nOrigin: {origin}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        let mut r = String::new();
        c.read_to_string(&mut r).ok();
        r.lines().next().unwrap_or("").to_string()
    };
    let https = ask("https://studio.example.org");
    let other = ask("https://evil.example.com");
    let _ = child.kill();
    let _ = child.wait();
    assert!(!https.contains("403"), "old behaviour: https origin refused -- {https}");
    assert!(other.contains("403"), "{other}");
}

#[test]
fn lint_reports_an_unusable_sound_track() {
    let s = ScratchDir::new("osatura-w4-lint");
    let mut p = osatura_core::Project::new("snd", 32.0, 32.0, 24.0, 4);
    let mut v: serde_json::Value = serde_json::from_str(&p.to_json_pretty()).unwrap();
    v["sounds"] = serde_json::json!([{"name": "boom", "file": "missing.wav"}]);
    p = serde_json::from_value(v).unwrap();
    fs::write(s.join("d.json"), p.to_json_pretty()).unwrap();
    let (ok, t) = run_in(&s, &["lint", "d.json"]);
    assert!(!ok && t.contains("sound-unusable"), "{t}");
}
