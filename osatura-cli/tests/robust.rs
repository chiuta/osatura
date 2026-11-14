// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Robustness and security regressions that are only visible through the binary: huge numbers, generators that
//! must not leave invalid files, outputs that must not clobber the input, unknown fields, broken pipes, hostile
//! asset paths, oversized outputs and runaway sizes.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_osatura");

struct T {
    dir: PathBuf,
}

struct Out {
    ok: bool,
    code: Option<i32>,
    text: String,
}

impl T {
    fn new(tag: &str) -> T {
        let guard =
            osatura_core::fsutil::private_temp_dir(&format!("osatura-robust-{tag}")).unwrap();
        let dir = guard.path().to_path_buf();
        std::mem::forget(guard); // `T` removes the directory when dropped
        T { dir }
    }
    fn p(&self, n: &str) -> String {
        self.dir.join(n).to_string_lossy().into_owned()
    }
    fn run(&self, args: &[&str]) -> Out {
        let o = Command::new(BIN)
            .args(args)
            .current_dir(&self.dir)
            .output()
            .expect("cannot start osatura");
        let mut text = String::from_utf8_lossy(&o.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&o.stderr));
        Out {
            ok: o.status.success(),
            code: o.status.code(),
            text,
        }
    }
    fn demo(&self, name: &str) -> Value {
        assert!(self.run(&["demo", &self.p(name)]).ok);
        serde_json::from_str(&fs::read_to_string(self.p(name)).unwrap()).unwrap()
    }
    fn put(&self, name: &str, v: &Value) {
        fs::write(self.p(name), serde_json::to_string_pretty(v).unwrap()).unwrap();
    }
}

impl Drop for T {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn r1_huge_finite_numbers_are_refused_with_a_clear_error_and_exit_code() {
    let t = T::new("r1");
    let base = t.demo("d.json");
    let cases: Vec<(&str, Value)> = vec![
        ("/camera/zoom", json!(1e300)),
        ("/layers/1/transform/scale", json!([1e300, 1.0])),
        ("/bones/0/rest_offset", json!([1e308, 0.0])),
        ("/layers/0/paths/0/anchors/0/pos", json!([1e300, 0.0])),
        ("/layers/1/transform/position", json!([3.4e38, 0.0])),
    ];
    for (ptr, val) in cases {
        let mut v = base.clone();
        *v.pointer_mut(ptr).unwrap_or_else(|| panic!("{ptr}")) = val;
        t.put("big.json", &v);
        for cmd in [
            vec!["check", "big.json"],
            vec!["render", "big.json", "0", "o.svg"],
            vec!["export", "big.json", "o.json"],
        ] {
            let o = t.run(&cmd);
            assert!(
                !o.ok && o.code == Some(1),
                "{ptr} {cmd:?}: {} {:?}",
                o.text,
                o.code
            );
            assert!(
                o.text.contains("outside the supported range"),
                "{ptr}: {}",
                o.text
            );
        }
        assert!(
            !Path::new(&t.p("o.svg")).exists() && !Path::new(&t.p("o.json")).exists(),
            "nothing is written"
        );
    }
}

#[test]
fn r2_a_generator_never_leaves_an_invalid_project_behind() {
    let t = T::new("r2gen");
    t.demo("d.json");
    let o = t.run(&["marker", "d.json", "m.json", "big", "1e300"]);
    assert!(!o.ok, "{}", o.text);
    assert!(
        !Path::new(&t.p("m.json")).exists(),
        "the invalid result must not be written"
    );
    // an existing good output survives a failed generator run
    assert!(t.run(&["marker", "d.json", "m.json", "ok", "3"]).ok);
    let before = fs::read(t.p("m.json")).unwrap();
    let o = t.run(&["marker", "d.json", "m.json", "big", "1e300"]);
    assert!(!o.ok);
    assert_eq!(
        fs::read(t.p("m.json")).unwrap(),
        before,
        "atomic: old file untouched"
    );
    // the good one is valid for the other commands
    assert!(t.run(&["check", "m.json"]).ok);
}

#[test]
fn r2_write_json_reports_the_real_cause_of_an_invalid_output() {
    let t = T::new("r2cause");
    let mut v = t.demo("d.json");
    // make the input itself fine for parsing but give the generator a frame the project cannot hold
    v["frames"] = json!(10);
    t.put("small.json", &v);
    let o = t.run(&["event", "small.json", "e.json", "boom", "1e30"]);
    assert!(!o.ok, "{}", o.text);
    assert!(!Path::new(&t.p("e.json")).exists());
    assert!(
        o.text.contains("output would be invalid")
            || o.text.contains("outside")
            || o.text.contains("frame"),
        "{}",
        o.text
    );
}

#[test]
fn r2_outputs_never_overwrite_the_input_project() {
    let t = T::new("r2same");
    t.demo("d.json");
    let orig = fs::read(t.p("d.json")).unwrap();
    for cmd in [
        vec!["render", "d.json", "0", "d.json"],
        vec!["export", "d.json", "d.json"],
        vec!["render-hdr", "d.json", "0", "d.json"],
        vec!["golden", "write", "d.json", "d.json"],
        vec!["onion", "d.json", "d.json"],
    ] {
        let o = t.run(&cmd);
        assert!(!o.ok, "{cmd:?}: {}", o.text);
        assert_eq!(
            fs::read(t.p("d.json")).unwrap(),
            orig,
            "{cmd:?} destroyed the project"
        );
    }
    // also through a different spelling of the same file
    let o = t.run(&["render", "d.json", "0", "./d.json"]);
    assert!(!o.ok && o.text.contains("refusing"), "{}", o.text);
    // the png sheet writes <name>.atlas.json next to the sheet, so d.png no longer touches the project d.json
    let o = t.run(&["export", "d.json", "d.png"]);
    assert!(o.ok, "{}", o.text);
    assert_eq!(fs::read(t.p("d.json")).unwrap(), orig);
    assert!(Path::new(&t.p("d.atlas.json")).exists());
    // but a project that IS the sidecar name is still protected
    fs::copy(t.p("d.json"), t.p("s.atlas.json")).unwrap();
    let o = t.run(&["export", "s.atlas.json", "s.png"]);
    assert!(!o.ok && o.text.contains("refusing"), "{}", o.text);
    assert_eq!(fs::read(t.p("s.atlas.json")).unwrap(), orig);
    // a normal render still works
    assert!(t.run(&["render", "d.json", "0", "o.svg"]).ok);
}

#[test]
fn r2_symlinked_outputs_are_not_followed() {
    let t = T::new("r2link");
    t.demo("d.json");
    fs::write(t.p("victim.txt"), "precious").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(t.p("victim.txt"), t.p("o.svg")).unwrap();
        let o = t.run(&["render", "d.json", "0", "o.svg"]);
        assert!(!o.ok, "{}", o.text);
        assert_eq!(fs::read_to_string(t.p("victim.txt")).unwrap(), "precious");
    }
}

#[test]
fn r3_a_closed_pipe_is_not_a_panic() {
    let t = T::new("r3");
    t.demo("d.json");
    // lint prints one line per finding and the usage text is long: close stdout straight away
    for args in [
        vec!["help"],
        vec!["lint", "d.json"],
        vec!["check", "d.json"],
    ] {
        let mut c = Command::new(BIN)
            .args(&args)
            .current_dir(&t.dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        drop(c.stdout.take());
        let o = c.wait_with_output().unwrap();
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(
            !err.contains("panicked") && !err.contains("failed printing"),
            "{args:?}: {err}"
        );
        assert_ne!(o.status.code(), Some(101), "{args:?}");
    }
}

#[test]
fn r4_unknown_fields_are_warned_about_and_transition_typos_are_errors() {
    let t = T::new("r4");
    let mut v = t.demo("d.json");
    let ok = t.run(&["check", "d.json"]);
    assert!(ok.ok && !ok.text.contains("unknown field"), "{}", ok.text);
    v["layers"][0]["opacty"] = json!(0.5);
    v["layers"][0]["paths"][0]["fil"] = json!({"color": "#fff"});
    t.put("typo.json", &v);
    let o = t.run(&["check", "typo.json"]);
    assert!(o.ok, "an unknown field is a warning for check: {}", o.text);
    assert!(
        o.text.contains("unknown field layers[0].opacty")
            && o.text.contains("unknown field layers[0].paths[0].fil"),
        "{}",
        o.text
    );
    let l = t.run(&["lint", "typo.json"]);
    assert!(
        l.text.contains("unknown field layers[0].opacty"),
        "{}",
        l.text
    );

    // a machine transition with `conditions` instead of `when` used to load as an always-firing transition
    let mut m = t.demo("m.json");
    m["machines"] = json!([{
        "name": "m", "inputs": [{"name": "go", "kind": "bool"}],
        "layers": [{"name": "body", "entry": "a", "states": [], "transitions": [{"from": "a", "to": "b", "conditions": [{"input": "go"}]}]}]
    }]);
    t.put("mach.json", &m);
    let o = t.run(&["check", "mach.json"]);
    assert!(!o.ok && o.text.contains("conditions"), "{}", o.text);
}

#[test]
fn s3_runaway_sizes_are_refused_cleanly() {
    let t = T::new("s3");
    let mut v = t.demo("d.json");
    // fuzz --samples above the cap
    let o = t.run(&["fuzz", "d.json", "--samples", "1000000000"]);
    assert!(
        !o.ok && !o.text.contains("panicked") && o.code != Some(101),
        "{}",
        o.text
    );
    assert!(o.text.to_lowercase().contains("samples"), "{}", o.text);
    // a stage whose render would be gigantic
    v["width"] = json!(30_000.0);
    v["height"] = json!(30_000.0);
    t.put("wide.json", &v);
    let o = t.run(&["render", "wide.json", "0", "o.png"]);
    assert!(
        !o.ok && o.code != Some(101) && !o.text.contains("panicked"),
        "{}",
        o.text
    );
    assert!(
        o.text.contains("pixel") || o.text.contains("Mpixel") || o.text.contains("size"),
        "{}",
        o.text
    );
    assert!(!Path::new(&t.p("o.png")).exists());
}

#[test]
fn s4_deep_expressions_do_not_overflow_the_stack_through_the_cli() {
    let t = T::new("s4");
    let mut v = t.demo("d.json");
    let deep = format!("{}1{}", "(".repeat(20_000), ")".repeat(20_000));
    let minus = format!("{}1", "-".repeat(200_000));
    for (i, src) in [deep, minus].iter().enumerate() {
        let bone = v["bones"][0]["name"].as_str().unwrap().to_string();
        v["channels"] = json!([{"target": bone, "prop": "bone_angle", "source": {"expr": src}, "mode": "replace", "weight": 1.0}]);
        t.put(&format!("e{i}.json"), &v);
        let o = t.run(&["check", &format!("e{i}.json")]);
        assert!(!o.ok, "{}", &o.text[..o.text.len().min(300)]);
        assert!(
            o.code == Some(1) && !o.text.contains("overflow") && !o.text.contains("SIGABRT"),
            "exit {:?}",
            o.code
        );
    }
}

#[test]
fn s2_assets_stay_inside_the_project_directory() {
    let t = T::new("s2");
    let mut v = t.demo("d.json");
    // a tiny valid PNG next to the project and one outside it
    let png = {
        let o = t.run(&["render", "d.json", "0", "inside.png", "--scale", "0.1"]);
        assert!(o.ok, "{}", o.text);
        fs::read(t.p("inside.png")).unwrap()
    };
    let outer =
        std::env::temp_dir().join(format!("osatura-robust-outside-{}.png", std::process::id()));
    fs::write(&outer, &png).unwrap();
    let image_layer = |src: &str| {
        json!({"name": "img", "type": "image", "source": {"file": src}, "origin": [0.0, 0.0],
        "transform": {"position": [0.0, 0.0], "rotation": 0.0, "scale": [1.0, 1.0], "skew": 0.0, "anchor": [0.0, 0.0]}, "opacity": 1.0, "visible": true, "blend": "normal"})
    };
    let outside = outer.to_string_lossy().into_owned();
    let rel_up = format!("../{}", outer.file_name().unwrap().to_string_lossy());
    for (src, should_work) in [
        ("inside.png", true),
        (outside.as_str(), false),
        (rel_up.as_str(), false),
        ("/etc/passwd", false),
    ] {
        v["layers"] = json!([image_layer(src)]);
        v["channels"] = json!([]);
        t.put("img.json", &v);
        let o = t.run(&["check", "img.json"]);
        assert_eq!(o.ok, should_work, "{src}: {}", o.text);
        if !should_work {
            assert!(
                o.text.contains("outside")
                    || o.text.contains("inside the project")
                    || o.text.contains("not allowed")
                    || o.text.contains("relative"),
                "{src}: {}",
                o.text
            );
        }
    }
    // the escape hatch has to be asked for explicitly
    v["layers"] = json!([image_layer(&outside)]);
    t.put("img.json", &v);
    let o = t.run(&["check", "img.json", "--allow-external-assets"]);
    assert!(o.ok, "{}", o.text);
    let _ = fs::remove_file(outer);
}
