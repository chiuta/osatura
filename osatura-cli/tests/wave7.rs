// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Wave 7 regression tests: per-sub-command option validation for `export`, `keys`, `idle`, `comp`, `snapshot`,
//! the behaviour fixes (`follow --chain`, `make-loop`, soft IK) and the colour-space option.

use osatura_core::fsutil::ScratchDir;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const BIN: &str = env!("CARGO_BIN_EXE_osatura");

fn run_in(dir: &Path, args: &[&str]) -> (bool, String) {
    let o = Command::new(BIN).current_dir(dir).args(args).output().expect("cannot start osatura");
    (o.status.success(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

/// The demo project at `<dir>/d.json`.
fn demo(dir: &Path) {
    let (ok, t) = run_in(dir, &["demo", "d.json"]);
    assert!(ok, "{t}");
}

#[test]
fn a_export_refuses_options_that_the_format_does_not_use() {
    let s = ScratchDir::new("osatura-w7-export");
    demo(&s);
    for (out, opt, val) in [("o.json", "--scale", "0.5"), ("o.html", "--report", "r.json"), ("o.glb", "--scale", "2"), ("o.apng", "--columns", "2"), ("o.mp4", "--codec", "av1"), ("o.gif", "--codec", "vp9")] {
        let (ok, t) = run_in(&s, &["export", "d.json", out, opt, val]);
        assert!(!ok && t.contains(&format!("unknown option {opt} for 'export .{}'", &out[2..])), "{out} {opt}: {t}");
        assert!(!s.join(out).exists(), "{out} must not be written when the option is refused");
    }
    // accepted combinations still work
    let (ok, t) = run_in(&s, &["export", "d.json", "o.gif", "--scale", "0.25", "--report", "r.json"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["export", "d.json", "o.json", "--report", "r2.json"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["export", "d.json", "O.APNG", "--scale", "0.25"]);
    assert!(ok, "{t}");
    // an unknown extension reports the format, not an option error
    let (ok, t) = run_in(&s, &["export", "d.json", "o.xyz", "--scale", "2"]);
    assert!(!ok && t.contains("unknown export format"), "{t}");
}

#[test]
fn a_keys_idle_comp_snapshot_have_one_option_set_per_sub_command() {
    let s = ScratchDir::new("osatura-w7-subs");
    demo(&s);
    // keys: required option per operation, foreign options refused
    let (ok, t) = run_in(&s, &["keys", "shift", "d.json", "o.json"]);
    assert!(!ok && t.contains("keys shift needs --by"), "{t}");
    let (ok, t) = run_in(&s, &["keys", "scale", "d.json", "o.json"]);
    assert!(!ok && t.contains("keys scale needs --factor"), "{t}");
    let (ok, t) = run_in(&s, &["keys", "shift", "d.json", "o.json", "--by", "1", "--factor", "2"]);
    assert!(!ok && t.contains("unknown option --factor for 'keys shift'") && t.contains("keys scale"), "{t}");
    let (ok, t) = run_in(&s, &["keys", "bogus", "d.json", "o.json"]);
    assert!(!ok && t.contains("unknown keys operation"), "{t}");
    let (ok, t) = run_in(&s, &["keys", "shift", "d.json", "o.json", "--by", "1"]);
    assert!(ok, "{t}");
    // idle
    let (ok, t) = run_in(&s, &["idle", "breath", "d.json", "o.json", "--target", "torso", "--prop", "bone_scale", "--seed", "2"]);
    assert!(!ok && t.contains("unknown option --seed for 'idle breath'"), "{t}");
    // comp: list takes no output file; capture / apply need a name
    let (ok, t) = run_in(&s, &["comp", "list", "d.json"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["comp", "capture", "d.json", "c.json"]);
    assert!(!ok && t.contains("needs <project.json> <out.json> <name>"), "{t}");
    let (ok, t) = run_in(&s, &["comp", "capture", "d.json", "c.json", "pose1"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["comp", "list", "c.json"]);
    assert!(ok && t.contains("pose1"), "{t}");
    let (ok, t) = run_in(&s, &["comp", "list", "c.json", "c.json"]);
    assert!(ok && t.contains("pose1"), "legacy 4-word form: {t}");
    let (ok, t) = run_in(&s, &["comp", "bogus", "d.json", "c.json"]);
    assert!(!ok && t.contains("unknown comp action"), "{t}");
    // snapshot
    let (ok, t) = run_in(&s, &["snapshot", "save", "d.json", "snaps", "--keep", "3"]);
    assert!(ok && t.contains("saved revision 1"), "{t}");
    let (ok, t) = run_in(&s, &["snapshot", "list", "d.json", "snaps", "--keep", "3"]);
    assert!(!ok && t.contains("unknown option --keep for 'snapshot list'") && t.contains("snapshot save"), "{t}");
    let (ok, t) = run_in(&s, &["snapshot", "save", "d.json", "snaps", "--number", "1"]);
    assert!(!ok && t.contains("snapshot restore"), "{t}");
    let (ok, t) = run_in(&s, &["snapshot", "list", "d.json", "snaps", "extra.json"]);
    assert!(!ok && t.contains("needs <project.json> <dir> and nothing else"), "{t}");
    let (ok, t) = run_in(&s, &["snapshot", "restore", "d.json", "snaps"]);
    assert!(!ok && t.contains("needs <project.json> <dir> <out.json>"), "{t}");
    let (ok, t) = run_in(&s, &["snapshot", "restore", "d.json", "snaps", "r.json", "--number", "1"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["snapshot", "recover", "d.json", "snaps", "r2.json", "--number", "1"]);
    assert!(!ok && t.contains("--number"), "{t}");
    let (ok, t) = run_in(&s, &["snapshot", "recover", "d.json", "snaps", "r2.json"]);
    assert!(ok, "{t}");
}

#[test]
fn b_follow_never_writes_keys_past_the_last_frame_and_make_loop_explains_the_flat_channel() {
    let s = ScratchDir::new("osatura-w7-follow");
    demo(&s);
    let text = std::fs::read_to_string(s.join("d.json")).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let frames = v["frames"].as_u64().unwrap() as f64;
    // a keyed source channel that ends on the last frame
    let ch = v["channels"].as_array().unwrap().iter().find(|c| c["source"] == "keys" && c["prop"] == "bone_angle").cloned().unwrap();
    let (target, bones) = (ch["target"].as_str().unwrap().to_string(), v["bones"].as_array().unwrap().iter().map(|b| b["name"].as_str().unwrap().to_string()).filter(|n| n != ch["target"].as_str().unwrap()).take(2).collect::<Vec<_>>());
    let last_key = ch["track"]["keys"].as_array().unwrap().last().unwrap()["frame"].as_f64().unwrap();
    assert!(last_key <= frames);
    let chain = bones.join(",");
    let (ok, t) = run_in(&s, &["follow", "d.json", "f.json", "--target", &target, "--prop", "bone_angle", "--chain", &chain, "--delay", "20"]);
    assert!(ok, "{t}");
    let f: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(s.join("f.json")).unwrap()).unwrap();
    for c in f["channels"].as_array().unwrap() {
        if let Some(keys) = c["track"]["keys"].as_array() {
            assert!(keys.iter().all(|k| k["frame"].as_f64().unwrap() <= frames), "key past the last frame in {c}");
        }
    }
    let (ok, t) = run_in(&s, &["lint", "f.json"]);
    assert!(!t.contains("L007") || !t.contains("outside the 0.."), "lint still reports keys outside the timeline: {t} {ok}");
    // make-loop on a walking rig: the flat channel is explained
    v["channels"].as_array_mut().unwrap().push(serde_json::json!({"target": bones[0], "prop": "bone_offset_x", "source": "keys", "mode": "replace", "weight": 1.0,
        "track": {"keys": [{"frame": 0, "value": 0, "ease": {"type": "linear"}}, {"frame": frames, "value": 50, "ease": {"type": "linear"}}]}}));
    std::fs::write(s.join("w.json"), v.to_string()).unwrap();
    let (ok, t) = run_in(&s, &["make-loop", "w.json", "l.json", &bones[0], "bone_offset_x"]);
    assert!(ok && t.contains("is now constant") && t.contains("removes on purpose"), "{t}");
}

#[test]
fn d_color_space_command_switches_the_png_to_linear_light() {
    let s = ScratchDir::new("osatura-w7-linear");
    let mut p = osatura_core::Project::new("half", 8.0, 8.0, 24.0, 1);
    p.background = osatura_core::style::Rgba::parse("#ffffff");
    let mut top = osatura_core::project::Layer::vector("t", vec![osatura_core::path::VPath::rect("r", 0.0, 0.0, 8.0, 8.0).with_fill("#000000")]);
    top.opacity = 0.5;
    p.layers = vec![top];
    std::fs::write(s.join("p.json"), p.to_json_pretty()).unwrap();
    let px = |name: &str| -> u8 {
        let d = png::Decoder::new(std::io::Cursor::new(std::fs::read(s.join(name)).unwrap()));
        let mut r = d.read_info().unwrap();
        let mut buf = vec![0; r.output_buffer_size().unwrap()];
        let info = r.next_frame(&mut buf).unwrap();
        buf[(4 * info.width as usize + 4) * info.color_type.samples()]
    };
    let (ok, t) = run_in(&s, &["render", "p.json", "0", "srgb.png"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["color-space", "p.json", "lin.json", "linear"]);
    assert!(ok && t.contains("color_space: linear"), "{t}");
    assert!(std::fs::read_to_string(s.join("lin.json")).unwrap().contains("\"color_space\": \"linear\""));
    let (ok, t) = run_in(&s, &["render", "lin.json", "0", "lin.png"]);
    assert!(ok, "{t}");
    assert!((px("srgb.png") as i32 - 128).abs() <= 1, "default: {}", px("srgb.png"));
    assert!((px("lin.png") as i32 - 188).abs() <= 1, "linear light: {}", px("lin.png"));
    // back to srgb drops the key again
    let (ok, _) = run_in(&s, &["color-space", "lin.json", "back.json", "srgb"]);
    assert!(ok && !std::fs::read_to_string(s.join("back.json")).unwrap().contains("color_space"));
    let (ok, t) = run_in(&s, &["color-space", "p.json", "x.json", "p3"]);
    assert!(!ok && t.contains("srgb or linear"), "{t}");
    assert!(!s.join("x.json").exists());
}

// ---- E: collaboration hardening, against a real server process ----

struct Kill(Child);
impl Drop for Kill {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn request(port: u16, method: &str, path: &str, token: Option<&str>, body: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let tok = token.map(|t| format!("X-Collab-Token: {t}\r\n")).unwrap_or_default();
    write!(s, "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{tok}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    out
}

fn start_studio(dir: &Path, port: u16, extra: &[&str]) -> Kill {
    let mut args = vec!["studio".to_string(), "d.json".into(), "--port".into(), port.to_string(), "--ui-config".into(), "ui.json".into()];
    args.extend(extra.iter().map(|s| s.to_string()));
    let child = Command::new(BIN).current_dir(dir).args(&args).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Kill(child)
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

const SECRET: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn secret_file(dir: &Path) {
    let f = dir.join("collab.secret");
    std::fs::write(&f, format!("{SECRET}\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

#[test]
fn e_collab_token_command_mints_verifiable_bounded_tokens() {
    let s = ScratchDir::new("osatura-w7-mint");
    secret_file(&s);
    let o = Command::new(BIN).current_dir(&s).args(["collab-token", "ana", "--secret-file", "collab.secret", "--ttl", "3600"]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let token = String::from_utf8_lossy(&o.stdout).trim().to_string();
    let claims = osatura_core::collabauth::verify_token(SECRET, &token, osatura_core::clock::unix_seconds()).unwrap();
    assert_eq!(claims.name, "ana");
    let left = claims.expires - osatura_core::clock::unix_seconds();
    assert!((3590..=3600).contains(&left), "{left}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("valid until Unix time"));
    let bad: Vec<(Vec<&str>, &str)> = vec![
        (vec!["collab-token", "a b", "--secret-file", "collab.secret"], "participant name"),
        (vec!["collab-token", "ana", "--secret-file", "collab.secret", "--ttl", "0"], "--ttl needs"),
        (vec!["collab-token", "ana", "--secret-file", "collab.secret", "--ttl", "99999999"], "30 days"),
        (vec!["collab-token", "ana", "--secret-file", "missing.secret"], "cannot read"),
        (vec!["collab-token", "ana", "--secret", SECRET], "unknown option --secret"),
    ];
    for (args, want) in bad {
        let (ok, t) = run_in(&s, &args);
        assert!(!ok && t.contains(want), "{args:?}: {t}");
    }
    // without --secret-file the secret may come from the environment, never from an argument
    let o = Command::new(BIN).current_dir(&s).env("OSATURA_COLLAB_SECRET", SECRET).args(["collab-token", "bo"]).output().unwrap();
    assert!(o.status.success());
    let o = Command::new(BIN).current_dir(&s).env_remove("OSATURA_COLLAB_SECRET").args(["collab-token", "bo"]).output().unwrap();
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("never accepted on the command line"));
}

#[test]
fn e_studio_with_a_secret_file_accepts_only_valid_participant_tokens_and_rate_limits() {
    let s = ScratchDir::new("osatura-w7-studio");
    demo(&s);
    secret_file(&s);
    // a non-local bind is refused without any credential, and allowed with a secret file (we only check the refusal)
    let (ok, t) = run_in(&s, &["studio", "d.json", "--bind", "0.0.0.0", "--port", "0"]);
    assert!(!ok && t.contains("--collab-secret-file"), "{t}");
    // --collab-revoke needs the secret file
    let (ok, t) = run_in(&s, &["studio", "d.json", "--collab-revoke", "ana", "--port", "0"]);
    assert!(!ok && t.contains("needs --collab-secret-file"), "{t}");
    let (ok, t) = run_in(&s, &["studio", "d.json", "--collab-rate", "fast", "--port", "0"]);
    assert!(!ok && t.contains("--collab-rate needs a number"), "{t}");
    let now = osatura_core::clock::unix_seconds();
    let good = osatura_core::collabauth::mint_token(SECRET, "ana", now, 600).unwrap();
    let revoked = osatura_core::collabauth::mint_token(SECRET, "eve", now, 600).unwrap();
    let expired = osatura_core::collabauth::mint_token(SECRET, "old", now - 100, 50).unwrap();
    let port = free_port();
    let _k = start_studio(&s, port, &["--collab-secret-file", "collab.secret", "--collab-revoke", "eve", "--collab-rate", "5"]);
    assert!(request(port, "GET", "/api/state", None, "").starts_with("HTTP/1.1 401"));
    assert!(request(port, "GET", "/api/state", Some(SECRET), "").starts_with("HTTP/1.1 401"), "the secret is not a credential");
    let r = request(port, "GET", "/api/state", Some(&good), "");
    assert!(r.starts_with("HTTP/1.1 200"), "{r}");
    let r = request(port, "GET", "/api/state", Some(&revoked), "");
    assert!(r.starts_with("HTTP/1.1 401") && r.contains("revoked"), "{r}");
    // the participant joins under the name in the token
    let r = request(port, "POST", "/api/collab/join", Some(&good), "{\"name\":\"mallory\"}");
    assert!(r.contains("\"name\":\"ana\""), "{r}");
    let r = request(port, "GET", "/api/state", Some(&expired), "");
    assert!(r.starts_with("HTTP/1.1 401") && r.contains("expired"), "{r}");
    // 5 requests per second, bursts of 10: a quick series meets 429 with Retry-After
    let mut limited = None;
    for _ in 0..40 {
        let r = request(port, "GET", "/api/state", Some(&good), "");
        if r.starts_with("HTTP/1.1 429") {
            limited = Some(r);
            break;
        }
    }
    let r = limited.expect("rate limit never answered 429");
    assert!(r.contains("Retry-After: 1") && r.contains("too many requests"), "{r}");
}

#[test]
fn e_studio_with_named_hosts_keeps_save_compact_and_revoke_for_them() {
    let s = ScratchDir::new("osatura-w7-hosts");
    demo(&s);
    secret_file(&s);
    // the names come from per-participant tokens: --collab-host needs the secret file and a name
    let (ok, t) = run_in(&s, &["studio", "d.json", "--collab-host", "alexio", "--port", "0"]);
    assert!(!ok && t.contains("--collab-host") && t.contains("--collab-secret-file"), "{t}");
    let now = osatura_core::clock::unix_seconds();
    let host = osatura_core::collabauth::mint_token(SECRET, "alexio", now, 600).unwrap();
    let ana = osatura_core::collabauth::mint_token(SECRET, "ana", now, 600).unwrap();
    let port = free_port();
    let _k = start_studio(&s, port, &["--collab-secret-file", "collab.secret", "--collab-host", "alexio", "--collab-rate", "0"]);
    // an ordinary participant can read and edit ...
    assert!(request(port, "GET", "/api/state", Some(&ana), "").starts_with("HTTP/1.1 200"));
    // ... but save, compact and revoke are refused (403) and nothing is written
    for (path, body) in [("/api/save", "{}"), ("/api/collab/compact", "{}"), ("/api/collab/revoke", "{\"names\":[\"alexio\"]}")] {
        let r = request(port, "POST", path, Some(&ana), body);
        assert!(r.starts_with("HTTP/1.1 403") && r.contains("--collab-host"), "{path}: {r}");
    }
    let before = std::fs::read_to_string(s.join("d.json")).unwrap();
    // the host may
    let r = request(port, "POST", "/api/save", Some(&host), "{}");
    assert!(r.starts_with("HTTP/1.1 200") && r.contains("\"ok\":true"), "{r}");
    assert!(std::fs::metadata(s.join("d.json")).is_ok() && !before.is_empty());
    let r = request(port, "POST", "/api/collab/compact", Some(&host), "{}");
    assert!(r.starts_with("HTTP/1.1 200"), "{r}");
    // revoking takes effect at once for the next request of that name, and the host's own token keeps working
    let r = request(port, "POST", "/api/collab/revoke", Some(&host), "{\"names\":[\"ana\"]}");
    assert!(r.starts_with("HTTP/1.1 200") && r.contains("\"revoked\":[\"ana\"]"), "{r}");
    let r = request(port, "GET", "/api/state", Some(&ana), "");
    assert!(r.starts_with("HTTP/1.1 401") && r.contains("revoked"), "{r}");
    assert!(request(port, "GET", "/api/state", Some(&host), "").starts_with("HTTP/1.1 200"));
}

#[test]
fn h_frame_range_is_checked_by_dof_render_hdr_and_fx() {
    let s = ScratchDir::new("osatura-w7-frames");
    demo(&s);
    std::fs::write(s.join("chain.json"), "[]").unwrap();
    let cases: [(&[&str], &str); 3] = [
        (&["dof", "d.json", "999", "o.png", "--scale", "0.1"], "o.png"),
        (&["render-hdr", "d.json", "999", "o.exr", "--scale", "0.1"], "o.exr"),
        (&["fx", "d.json", "999", "chain.json", "f.png", "--scale", "0.1"], "f.png"),
    ];
    for (args, out) in cases {
        let (ok, t) = run_in(&s, args);
        assert!(!ok && t.contains("outside the animation") && t.contains("--clamp"), "{args:?}: {t}");
        assert!(!s.join(out).exists(), "{out} must not be written for a refused frame");
        let mut with = args.to_vec();
        with.push("--clamp");
        let (ok, t) = run_in(&s, &with);
        assert!(ok && t.contains("warning: frame 999 is outside"), "{with:?}: {t}");
        assert!(s.join(out).exists(), "{out}");
    }
    // in-range frames need no flag, and --clamp stays refused where the form does not list it
    let (ok, t) = run_in(&s, &["dof", "d.json", "3", "o2.png", "--scale", "0.1"]);
    assert!(ok, "{t}");
    let (ok, t) = run_in(&s, &["dof-export", "d.json", "x.gif", "--clamp"]);
    assert!(!ok && t.contains("unknown option --clamp"), "{t}");
}
