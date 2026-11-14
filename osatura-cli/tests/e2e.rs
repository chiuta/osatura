// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Full end-to-end test: one demo project is taken through the main pipeline using only the CLI
//! binary, in a temporary directory. Every stage asserts that the command succeeded and checks a
//! measurable property of its result (channel counts, rendered pixels, file signatures, simulated
//! state log, report contents, exit codes).
//!
//! Run: `cargo test -p osatura-cli --test e2e -- --nocapture` (prints a per-stage timing table),
//! or `scripts/e2e.sh` (release build, same test). GPU rendering is only attempted to confirm that
//! a build without the `gpu` feature reports it cleanly (it is skipped, not failed).

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const BIN: &str = env!("CARGO_BIN_EXE_osatura");

struct Env {
    dir: PathBuf,
    timings: Vec<(String, f64)>,
}

struct Out {
    ok: bool,
    code: Option<i32>,
    text: String,
}

impl Env {
    fn new() -> Env {
        let dir = std::env::temp_dir().join(format!("osatura-e2e-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)));
        fs::create_dir_all(&dir).unwrap();
        Env { dir, timings: vec![] }
    }
    fn p(&self, name: &str) -> String {
        self.dir.join(name).to_string_lossy().into_owned()
    }
    /// Run the binary in the temp dir; stdout and stderr are returned together.
    fn run_in(&self, cwd: &Path, args: &[&str]) -> Out {
        let o = Command::new(BIN).args(args).current_dir(cwd).output().expect("cannot start osatura");
        let mut text = String::from_utf8_lossy(&o.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&o.stderr));
        Out { ok: o.status.success(), code: o.status.code(), text }
    }
    fn run(&self, args: &[&str]) -> Out {
        self.run_in(&self.dir, args)
    }
    /// Run and require success.
    fn ok(&self, args: &[&str]) -> String {
        let o = self.run(args);
        assert!(o.ok, "`osatura {}` failed (exit {:?}):\n{}", args.join(" "), o.code, o.text);
        o.text
    }
    fn json(&self, name: &str) -> Value {
        serde_json::from_str(&fs::read_to_string(self.p(name)).unwrap_or_else(|e| panic!("{name}: {e}"))).unwrap_or_else(|e| panic!("{name}: {e}"))
    }
    fn put(&self, name: &str, v: &Value) {
        fs::write(self.p(name), serde_json::to_string_pretty(v).unwrap()).unwrap();
    }
    fn stage(&mut self, name: &str, f: impl FnOnce(&Env)) {
        let t = Instant::now();
        f(self);
        let s = t.elapsed().as_secs_f64();
        eprintln!("[e2e] {name:<28} {s:7.2} s");
        self.timings.push((name.to_string(), s));
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn channels(v: &Value) -> usize {
    v["channels"].as_array().map_or(0, |c| c.len())
}

fn anchors(v: &Value) -> usize {
    match v {
        Value::Object(m) => m.iter().map(|(k, x)| if k == "anchors" { x.as_array().map_or(0, |a| a.len()) } else { anchors(x) }).sum(),
        Value::Array(a) => a.iter().map(anchors).sum(),
        _ => 0,
    }
}

fn read_png(path: &str) -> (u32, u32, png::ColorType, png::BitDepth, Vec<u8>) {
    let f = fs::File::open(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut r = png::Decoder::new(std::io::BufReader::new(f)).read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().expect("png size")];
    let i = r.next_frame(&mut buf).unwrap();
    buf.truncate(i.buffer_size());
    (i.width, i.height, i.color_type, i.bit_depth, buf)
}

fn write_png_gray_rgb(path: &str, w: u32, h: u32, f: impl Fn(u32, u32) -> u8) {
    let file = fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut wr = enc.write_header().unwrap();
    let mut data = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let v = f(x, y);
            data.extend_from_slice(&[v, v, v]);
        }
    }
    wr.write_image_data(&data).unwrap();
}

/// 16-bit mono WAV: a 150 Hz tone with a 900 Hz overtone, amplitude-modulated at 3 Hz (syllable-like).
fn wav_bytes(seconds: f32, rate: u32) -> Vec<u8> {
    let n = (seconds * rate as f32) as usize;
    let mut pcm = Vec::with_capacity(n * 2);
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let env = 0.5 * (1.0 + (2.0 * std::f32::consts::PI * 3.0 * t).sin());
        let s = env * (8000.0 * (2.0 * std::f32::consts::PI * 150.0 * t).sin() + 3000.0 * (2.0 * std::f32::consts::PI * 900.0 * t).sin());
        pcm.extend_from_slice(&(s as i16).to_le_bytes());
    }
    let mut w = Vec::new();
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&rate.to_le_bytes());
    w.extend_from_slice(&(rate * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    w.extend_from_slice(&pcm);
    w
}

const BVH: &str = "HIERARCHY
ROOT hips
{
  OFFSET 0 0 0
  CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
  JOINT spine
  {
    OFFSET 0 10 0
    CHANNELS 3 Zrotation Xrotation Yrotation
    End Site
    {
      OFFSET 0 10 0
    }
  }
}
MOTION
Frames: 4
Frame Time: 0.0416667
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 10 0 0
0 0 0 0 0 0 20 0 0
0 0 0 0 0 0 10 0 0
";

fn strip_timing(v: &mut Value) {
    match v {
        Value::Object(m) => {
            m.remove("seconds");
            m.remove("ms_per_sample");
            m.values_mut().for_each(strip_timing);
        }
        Value::Array(a) => a.iter_mut().for_each(strip_timing),
        _ => {}
    }
}

fn differing_pixels(a: &[u8], b: &[u8], bytes_per_px: usize) -> usize {
    assert_eq!(a.len(), b.len());
    a.chunks(bytes_per_px).zip(b.chunks(bytes_per_px)).filter(|(x, y)| x != y).count()
}

#[test]
fn full_pipeline_end_to_end() {
    let mut e = Env::new();
    let total = Instant::now();

    // 1. create ------------------------------------------------------------------------------
    e.stage("create demo", |e| {
        e.ok(&["demo", &e.p("p0.json")]);
        let s: Value = serde_json::from_str(&e.ok(&["stats", &e.p("p0.json")])).unwrap();
        assert_eq!((s["bones"].as_u64(), s["layers"].as_u64(), s["frames"].as_u64()), (Some(6), Some(14), Some(48)));
        assert_eq!(channels(&e.json("p0.json")), 14);
    });

    // 2. animate + rig -------------------------------------------------------------------------
    e.stage("animate (sway) + autorig", |e| {
        e.ok(&["idle", "sway", &e.p("p0.json"), &e.p("p1.json"), "--target", "arm_up", "--prop", "bone_angle", "--amp", "3"]);
        assert_eq!(channels(&e.json("p1.json")), 15, "sway adds exactly one channel");
        e.ok(&["autorig", &e.p("p1.json"), &e.p("p2.json")]);
        let lint = e.run(&["lint", &e.p("p2.json")]);
        assert!(lint.ok, "lint reports errors:\n{}", lint.text);
        assert!(lint.text.contains("0 error(s)"), "{}", lint.text);
    });

    // 3. skin ----------------------------------------------------------------------------------
    e.stage("skin add + apply", |e| {
        e.ok(&["skin", &e.p("p2.json"), &e.p("p3.json"), "add", "mouth", "smile", "mouth/closed,mouth/open"]);
        e.ok(&["skin", &e.p("p3.json"), &e.p("p4.json"), "apply", "mouth", "smile"]);
        let skins = e.json("p4.json")["skins"].clone();
        assert_eq!(skins[0]["name"], "smile");
        assert_eq!(skins[0]["layers"], json!(["mouth/closed", "mouth/open"]));
        // an unknown layer is refused with an error, not a panic
        let bad = e.run(&["skin", &e.p("p2.json"), &e.p("bad.json"), "add", "mouth", "x", "mouth/nope"]);
        assert!(!bad.ok && !bad.text.contains("panicked"), "{}", bad.text);
    });

    // 4. physics bake ----------------------------------------------------------------------------
    e.stage("physics bake", |e| {
        let before = channels(&e.json("p4.json"));
        let out = e.ok(&["physics", &e.p("p4.json"), &e.p("p5.json"), "arm_low,hand", "gravity=900", "merge=replace"]);
        assert!(out.contains("baked 2 bones"), "{out}");
        assert!(channels(&e.json("p5.json")) >= before, "baking must not remove channels");
        let a = e.p("ph_a.svg");
        let b = e.p("ph_b.svg");
        e.ok(&["render", &e.p("p4.json"), "30", &a]);
        e.ok(&["render", &e.p("p5.json"), "30", &b]);
        assert_ne!(fs::read(&a).unwrap(), fs::read(&b).unwrap(), "the baked simulation must change frame 30");
    });

    // 5. state machine + data binding --------------------------------------------------------------
    e.stage("machine + binding + sim", |e| {
        e.put("mach.json", &json!({"name": "m", "inputs": [{"name": "go", "kind": "bool"}], "layers": [{"name": "base", "entry": "idle",
            "states": [{"name": "idle", "source": {"kind": "segment", "start": 0, "end": 12}}, {"name": "act", "source": {"kind": "segment", "start": 12, "end": 36}}],
            "transitions": [{"from": "idle", "to": "act", "when": [{"input": "go"}], "duration": 0.1}]}]}));
        e.ok(&["machine", "add", &e.p("p5.json"), &e.p("p6.json"), &e.p("mach.json")]);
        assert!(e.ok(&["machine", "validate", &e.p("p6.json")]).contains("ok"));
        e.ok(&["bind", "add", &e.p("p6.json"), &e.p("p7.json"), "goBind", "hud.go", "@m:go"]);
        e.put("vals.json", &json!({"hud.go": 1}));
        assert!(e.ok(&["bind-eval", &e.p("p7.json"), &e.p("vals.json")]).contains("goBind = 1"));
        e.put("sc.json", &json!({"duration": 1.5, "events": [{"t": 0.5, "value": {"hud.go": 1}}]}));
        e.ok(&["machine-sim", &e.p("p7.json"), &e.p("sc.json"), &e.p("sim.json")]);
        let sim = e.json("sim.json");
        let ticks = sim["ticks"].as_array().unwrap();
        assert_eq!(ticks[0]["states"][0][1], "idle");
        assert_eq!(ticks[ticks.len() - 1]["states"][0][1], "act");
        let left = sim["events"].as_array().unwrap().iter().find(|v| v["kind"] == "exited" && v["state"] == "idle").unwrap();
        // regression: the input given at 0.5 s used to take effect at the start of its tick (0.472 s)
        assert!(left["time"].as_f64().unwrap() >= 0.5 - 1e-6, "state left at {} before the input at 0.5", left["time"]);
        // refused: an unknown value path
        e.put("sc_bad.json", &json!({"events": [{"t": 0, "set": {"ghost": 1}}]}));
        assert!(!e.run(&["machine-sim", &e.p("p7.json"), &e.p("sc_bad.json"), &e.p("x.json")]).ok);
    });

    // 6. layout containers ---------------------------------------------------------------------------
    e.stage("layout containers", |e| {
        e.put("cont.json", &json!({"id": "row1", "axis": "row", "gap": 10, "padding": [10, 10, 10, 10],
            "rect": {"kind": "pin", "left": 10, "right": 10, "top": 10, "height": 100},
            "members": [{"layer": "sun"}, {"layer": "blush", "grow": 1, "resize": true}]}));
        e.ok(&["container", "add", &e.p("p7.json"), &e.p("p8.json"), &e.p("cont.json")]);
        let wide = e.ok(&["container", "solve", &e.p("p8.json"), "--size", "800x480"]);
        let narrow = e.ok(&["container", "solve", &e.p("p8.json"), "--size", "400x300"]);
        assert!(wide.contains("container row1: x 10.00 y 10.00 w 780.00"), "{wide}");
        assert!(narrow.contains("container row1: x 10.00 y 10.00 w 380.00"), "{narrow}");
        assert_ne!(wide.lines().find(|l| l.starts_with("layer blush")), narrow.lines().find(|l| l.starts_with("layer blush")), "the growing member must follow the canvas");
        e.ok(&["layout-preview", &e.p("p8.json"), &e.p("lp.svg"), "--size", "400x300"]);
        assert!(fs::read_to_string(e.p("lp.svg")).unwrap().contains("<svg"));
        // a missing layer is refused
        e.put("cont_bad.json", &json!({"id": "bad", "axis": "row", "members": [{"layer": "no_such_layer"}]}));
        assert!(!e.run(&["container", "add", &e.p("p8.json"), &e.p("x.json"), &e.p("cont_bad.json")]).ok);
    });

    // 7. layer effects (plain project JSON; no CLI command edits them) ---------------------------------
    e.stage("layer effects", |e| {
        let mut p = e.json("p8.json");
        let sun = p["layers"].as_array_mut().unwrap().iter_mut().find(|l| l["name"] == "sun").unwrap();
        sun["effects"] = json!([{"type": "blur", "radius": 4}, {"type": "drop_shadow", "dx": 6, "dy": 6, "blur": 3, "color": "#000000", "opacity": 0.5}]);
        e.put("p9.json", &p);
        e.ok(&["render", &e.p("p9.json"), "10", &e.p("fx.svg")]);
        let svg = fs::read_to_string(e.p("fx.svg")).unwrap();
        assert!(svg.contains("<filter") && svg.contains("feGaussianBlur"), "effects must reach the SVG");
        e.ok(&["render", &e.p("p8.json"), "10", &e.p("nofx.png")]);
        e.ok(&["render", &e.p("p9.json"), "10", &e.p("fx.png")]);
        let (w0, h0, ct0, bd0, a) = read_png(&e.p("nofx.png"));
        let (_, _, _, _, b) = read_png(&e.p("fx.png"));
        let bpp = ct0.samples() * if bd0 == png::BitDepth::Sixteen { 2 } else { 1 };
        let diff = differing_pixels(&a, &b, bpp);
        assert!(diff > 50, "blur and shadow must change pixels (changed {diff})");
        assert!(diff < (w0 * h0) as usize / 3, "only the sun's neighbourhood may change (changed {diff} of {})", w0 * h0);
        // an out-of-range effect is a validation error
        let mut bad = e.json("p9.json");
        bad["layers"][0]["effects"] = json!([{"type": "blur", "radius": 1.0e9}]);
        e.put("fx_bad.json", &bad);
        let o = e.run(&["render", &e.p("fx_bad.json"), "0", &e.p("x.svg")]);
        assert!(!o.ok && !o.text.contains("panicked"), "{}", o.text);
    });

    // 8. lip-sync with a generated WAV ----------------------------------------------------------------
    e.stage("lip-sync (generated WAV)", |e| {
        fs::write(e.p("speech.wav"), wav_bytes(2.0, 16000)).unwrap();
        fs::write(e.p("speech.txt"), "hello there").unwrap();
        let before = channels(&e.json("p9.json"));
        let out = e.ok(&["lipsync-text", &e.p("p9.json"), &e.p("speech.wav"), &e.p("speech.txt"), "mouth", &e.p("p10.json"), "--map", "0,1,2,1,0"]);
        assert!(out.contains("2 words"), "{out}");
        assert!(channels(&e.json("p10.json")) >= before);
        let keys = |f: &str| -> Vec<(f64, f64)> {
            let p = e.json(f);
            let c = p["channels"].as_array().unwrap().iter().find(|c| c["target"] == "mouth" && c["prop"] == "layer_switch").expect("mouth layer_switch channel").clone();
            c["track"]["keys"].as_array().unwrap().iter().map(|k| (k["frame"].as_f64().unwrap(), k["value"].as_f64().unwrap())).collect()
        };
        let (old, new) = (keys("p9.json"), keys("p10.json"));
        assert_ne!(old, new, "lip-sync must rewrite the mouth switch keys");
        assert!(new.len() >= 4 && new.iter().all(|&(f, v)| (0.0..=48.0 * 2.0).contains(&f) && [0.0, 1.0, 2.0].contains(&v)), "keys {new:?}");
        assert!(new.iter().map(|k| k.1 as i32).collect::<std::collections::BTreeSet<_>>().len() >= 2, "more than one mouth shape is used");
        // a mapping index outside the switch is refused
        assert!(!e.run(&["lipsync-text", &e.p("p9.json"), &e.p("speech.wav"), &e.p("speech.txt"), "mouth", &e.p("x.json"), "--map", "0,1,2,3,4"]).ok);
    });

    // 9. mocap retarget with a generated BVH ------------------------------------------------------------
    e.stage("mocap retarget (generated BVH)", |e| {
        fs::write(e.p("m.bvh"), BVH).unwrap();
        let info = e.ok(&["mocap-info", &e.p("m.bvh")]);
        assert!(info.contains("4 frames") && info.contains("2 joints"), "{info}");
        e.put("bvhmap.json", &json!({"hips": "thigh", "spine": "shin"}));
        let before = channels(&e.json("p10.json"));
        let out = e.ok(&["mocap-retarget", &e.p("p10.json"), &e.p("m.bvh"), &e.p("p11.json"), "--map", &e.p("bvhmap.json")]);
        assert!(out.contains("4 frames"), "{out}");
        let after = e.json("p11.json");
        // "hips" keeps its hips role (root travel is transferred too), so there are the two angle channels plus the root offsets
        let angle_ch = after["channels"].as_array().unwrap().iter().filter(|c| (c["target"] == "shin" || c["target"] == "thigh") && c["prop"] == "bone_angle").count();
        assert_eq!(angle_ch, 2, "one angle channel per mapped bone");
        assert!(channels(&after) >= before + 2, "mocap channels were added");
        let peak = after["channels"].as_array().unwrap().iter().filter(|c| c["target"] == "shin" || c["target"] == "thigh").flat_map(|c| c["track"]["keys"].as_array().cloned().unwrap_or_default()).filter_map(|k| k["value"].as_f64()).fold(0.0f64, |m, v| m.max(v.abs()));
        assert!(peak > 5.0, "the 20 degree swing must reach the keys (peak {peak})");
        // malformed BVH is an error, not a panic
        fs::write(e.p("bad.bvh"), "HIERARCHY\nROOT x\n").unwrap();
        let o = e.run(&["mocap-info", &e.p("bad.bvh")]);
        assert!(!o.ok && !o.text.contains("panicked"));
    });

    // 10. autotrace of a generated image ------------------------------------------------------------------
    e.stage("autotrace (generated PNG)", |e| {
        write_png_gray_rgb(&e.p("shape.png"), 64, 64, |x, y| {
            let (dx, dy) = (x as i32 - 32, y as i32 - 32);
            if dx * dx + dy * dy < 300 || (8 < x && x < 20 && 8 < y && y < 20) { 0 } else { 255 }
        });
        let out = e.ok(&["autotrace", &e.p("shape.png"), &e.p("trace.json")]);
        let ssim: f64 = out.split("SSIM ").nth(1).and_then(|s| s.split_whitespace().next()).and_then(|s| s.parse().ok()).unwrap_or_else(|| panic!("no SSIM in: {out}"));
        assert!(ssim > 0.9, "traced shape must resemble the source (SSIM {ssim})");
        let t = e.json("trace.json");
        let n = anchors(&t);
        assert!((6..=60).contains(&n), "a disc and a square need few anchors (got {n})");
        e.ok(&["render", &e.p("trace.json"), "0", &e.p("trace.png")]);
        let (w, h, _, _, _) = read_png(&e.p("trace.png"));
        assert_eq!((w, h), (64, 64));
        // not an image
        fs::write(e.p("junk.png"), b"not a png").unwrap();
        assert!(!e.run(&["autotrace", &e.p("junk.png"), &e.p("x.json")]).ok);
    });

    // The heavy stages (lod, export-fit, ci) are 10-20x slower in a debug build. There they work on a
    // 12-frame cut of the project (and export-fit and ci on the 64x64 traced image) so that plain `cargo test` stays
    // practical; a release build (scripts/e2e.sh) runs them on the full 48-frame 640x480 project.
    let release = !cfg!(debug_assertions);
    let heavy = if release { "p11.json" } else { "p11_short.json" };
    let fit_src = if release { "p11.json" } else { "trace.json" };
    if !release {
        let mut p = e.json("p11.json");
        p["frames"] = json!(12);
        p.as_object_mut().unwrap().remove("machines"); // its segments reach past frame 12
        p.as_object_mut().unwrap().remove("bindings");
        e.put("p11_short.json", &p);
    }

    // 11. level of detail -------------------------------------------------------------------------------------
    e.stage("lod", |e| {
        let dir = e.p("lod");
        let heavy_path = e.p(heavy);
        let mut args = vec!["lod", heavy_path.as_str(), "--tolerance-px", "0.5", "--levels", "3", "--out-dir", dir.as_str(), "--frames", "4"];
        if release {
            args.push("--verify"); // renders every level against the original: slow in a debug build
        }
        let out = e.ok(&args);
        assert!(out.contains("L3"), "{out}");
        let full = anchors(&e.json(heavy));
        let counts: Vec<usize> = (1..=3).map(|i| anchors(&e.json(&format!("lod/{}_L{i}.json", heavy.trim_end_matches(".json"))))).collect();
        assert!(counts.windows(2).all(|w| w[1] <= w[0]) && counts[2] < full, "anchor counts must fall with the level: full {full}, levels {counts:?}");
    });

    // 12. export-fit: lottie / html / gif ---------------------------------------------------------------------
    e.stage("export-fit lottie + html", |e| {
        let out = e.ok(&["export-fit", &e.p(fit_src), &e.p("o.lottie.json"), "--max-size", "600KB", "--format", "lottie", "--frames", "8"]);
        assert!(out.contains("SSIM mean"), "{out}");
        let l = fs::metadata(e.p("o.lottie.json")).unwrap().len();
        assert!(l <= 600_000, "lottie is {l} bytes");
        let lot = e.json("o.lottie.json");
        assert!(lot["layers"].as_array().is_some_and(|a| !a.is_empty()) && lot["fr"].as_f64() == Some(24.0), "not a Lottie document");
        e.ok(&["export-fit", &e.p(fit_src), &e.p("o.html"), "--max-size", "600KB", "--format", "html", "--frames", "8"]);
        let html = fs::read_to_string(e.p("o.html")).unwrap();
        assert!(html.contains("<html") && html.contains("<svg") && fs::metadata(e.p("o.html")).unwrap().len() <= 600_000);
        // an impossible budget exits non-zero but still writes the smallest export
        let tiny = e.run(&["export-fit", &e.p(fit_src), &e.p("o_tiny.html"), "--max-size", "1KB", "--format", "html", "--frames", "4"]);
        assert!(!tiny.ok, "a 1 KB budget cannot be met");
    });
    e.stage("export-fit gif (small image)", |e| {
        let out = e.ok(&["export-fit", &e.p("trace.json"), &e.p("o.gif"), "--max-size", "200KB", "--format", "gif", "--frames", "6", "--search", "ladder"]);
        assert!(out.contains("gif ->"), "{out}");
        let g = fs::read(e.p("o.gif")).unwrap();
        assert!(g.starts_with(b"GIF89a") && g.len() <= 200_000, "gif signature / size ({} bytes)", g.len());
        assert_eq!(u16::from_le_bytes([g[6], g[7]]), 64, "GIF width");
    });

    // 13. CI: init, run, regression ----------------------------------------------------------------------------
    e.stage("ci init + run + regression", |e| {
        let dir = e.dir.join("ci");
        fs::create_dir_all(&dir).unwrap();
        fs::copy(e.p(if release { "p11.json" } else { "trace.json" }), dir.join("p.json")).unwrap();
        let init = e.run_in(&dir, &["ci", "init", "p.json"]);
        assert!(init.ok, "{}", init.text);
        assert!(dir.join("osatura-ci.json").exists() && dir.join("goldens").is_dir());
        let run = e.run_in(&dir, &["ci", "run", "--json", "r.json"]);
        assert!(run.ok, "ci run on an untouched project must pass:\n{}", run.text);
        let rep: Value = serde_json::from_str(&fs::read_to_string(dir.join("r.json")).unwrap()).unwrap();
        assert_eq!(rep["summary"]["failed"], 0);
        assert!(rep["summary"]["passed"].as_u64().unwrap() >= 5);
        // change what the picture looks like: CI must go red with exit code 1
        let mut p: Value = serde_json::from_str(&fs::read_to_string(dir.join("p.json")).unwrap()).unwrap();
        p["layers"][0]["visible"] = json!(false); // the sky in the demo, the traced group in the small image
        fs::write(dir.join("p.json"), serde_json::to_string(&p).unwrap()).unwrap();
        let bad = e.run_in(&dir, &["ci", "run", "--quiet"]);
        assert_eq!(bad.code, Some(1), "a visual regression must exit 1:\n{}", bad.text);
        // bless accepts the new look
        assert!(e.run_in(&dir, &["ci", "bless"]).ok);
        assert!(e.run_in(&dir, &["ci", "run", "--quiet"]).ok, "after bless the run is green again");
    });

    // 14. version control: commit, branch, clean merge, conflict + resolve ---------------------------------------
    e.stage("vcs init/commit/branch/merge", |e| {
        let d = e.dir.join("vc");
        fs::create_dir_all(&d).unwrap();
        fs::copy(e.p("p11.json"), d.join("p.json")).unwrap();
        let v = |args: &[&str]| -> String {
            let o = e.run_in(&d, &[&["vcs"], args].concat());
            assert!(o.ok, "vcs {} failed:\n{}", args.join(" "), o.text);
            o.text
        };
        v(&["init", "p.json"]);
        v(&["commit", "-m", "base", "--author", "e2e"]);
        v(&["branch", "feat"]);
        v(&["checkout", "feat"]);
        e.ok(&["marker", &d.join("p.json").to_string_lossy(), &d.join("p.json").to_string_lossy(), "intro", "3"]);
        v(&["commit", "-m", "marker on feat"]);
        v(&["checkout", "main"]);
        e.ok(&["event", &d.join("p.json").to_string_lossy(), &d.join("p.json").to_string_lossy(), "boom", "10"]);
        v(&["commit", "-m", "event on main"]);
        let m = v(&["merge", "feat"]);
        assert!(m.contains("merge committed"), "{m}");
        let merged: Value = serde_json::from_str(&fs::read_to_string(d.join("p.json")).unwrap()).unwrap();
        assert!(merged["markers"].to_string().contains("intro") && merged.to_string().contains("boom"), "both sides must survive the merge");
        // conflict: both branches place marker x on different frames
        v(&["branch", "c1"]);
        v(&["checkout", "c1"]);
        e.ok(&["marker", &d.join("p.json").to_string_lossy(), &d.join("p.json").to_string_lossy(), "x", "5"]);
        v(&["commit", "-m", "c1"]);
        v(&["checkout", "main"]);
        e.ok(&["marker", &d.join("p.json").to_string_lossy(), &d.join("p.json").to_string_lossy(), "x", "7"]);
        v(&["commit", "-m", "c2"]);
        let c = e.run_in(&d, &["vcs", "merge", "c1"]);
        assert!(!c.ok && c.text.contains("CONFLICT /markers/x/frame"), "{}", c.text);
        v(&["resolve", "/markers/x/frame", "--theirs"]);
        v(&["commit", "-m", "resolved"]);
        let after: Value = serde_json::from_str(&fs::read_to_string(d.join("p.json")).unwrap()).unwrap();
        let x = after["markers"].as_array().unwrap().iter().find(|m| m["name"] == "x").unwrap();
        assert_eq!(x["frame"], 5, "--theirs picks the c1 value");
        assert!(v(&["fsck"]).contains("consistent"));
        assert!(v(&["log"]).contains("resolved"));
    });

    // 15. fuzz --------------------------------------------------------------------------------------------------------
    e.stage("fuzz (deterministic)", |e| {
        let run = |name: &str| {
            e.ok(&["fuzz", &e.p("p11.json"), "--samples", "60", "--seed", "7", "--max-fail-rate", "1", "--report", &e.p(name)]);
            let mut r = e.json(name);
            strip_timing(&mut r);
            r
        };
        let (a, b) = (run("fz1.json"), run("fz2.json"));
        assert_eq!(a, b, "same seed must give the same report (timings aside)");
        assert_eq!(a["summary"]["samples"], 60);
        assert!(a["summary"]["failures"].as_u64().unwrap() <= 60);
        // a failure rate above the allowed maximum exits non-zero (this rig fails most extreme poses)
        let strict = e.run(&["fuzz", &e.p("p11.json"), "--samples", "60", "--seed", "7", "--max-fail-rate", "0"]);
        if a["summary"]["failures"].as_u64().unwrap() > 0 {
            assert!(!strict.ok, "max-fail-rate 0 must fail when failures exist");
        }
    });

    // 16. renders: svg / raster / float, GPU skipped gracefully ----------------------------------------------------------
    e.stage("render svg/raster/float/gpu", |e| {
        e.ok(&["render", &e.p("p11.json"), "12", &e.p("r.svg")]);
        let svg = fs::read_to_string(e.p("r.svg")).unwrap();
        assert!(svg.starts_with("<svg") || svg.contains("<svg"));
        e.ok(&["render", &e.p("p11.json"), "12", &e.p("r.png")]);
        let (w, h, _, _, px) = read_png(&e.p("r.png"));
        assert_eq!((w, h), (640, 480));
        assert!(px.iter().any(|&b| b != px[0]), "image must not be a single colour");
        // different frames differ (the project is animated)
        e.ok(&["render", &e.p("p11.json"), "30", &e.p("r30.png")]);
        assert_ne!(px, read_png(&e.p("r30.png")).4);
        // float renderer: 16-bit PNG and EXR
        e.ok(&["render-hdr", &e.p("p11.json"), "12", &e.p("h.png")]);
        e.ok(&["render-hdr", &e.p("p11.json"), "12", &e.p("h8.png"), "--8bit"]);
        let (_, _, _, bd, f) = read_png(&e.p("h.png"));
        let (_, _, _, _, l) = read_png(&e.p("h8.png"));
        assert_eq!(bd, png::BitDepth::Sixteen);
        let words = |b: &[u8]| -> Vec<f64> { b.chunks(2).map(|c| u16::from_be_bytes([c[0], c[1]]) as f64 / 257.0).collect() };
        let (fa, la) = (words(&f), words(&l));
        let mean: f64 = fa.iter().zip(&la).map(|(a, b)| (a - b).abs()).sum::<f64>() / fa.len() as f64;
        assert!(mean < 1.0, "float and 8-bit renderers must agree (mean abs diff {mean:.3} of 255)");
        e.ok(&["render-hdr", &e.p("p11.json"), "12", &e.p("h.exr")]);
        assert_eq!(&fs::read(e.p("h.exr")).unwrap()[..4], &[0x76, 0x2f, 0x31, 0x01], "OpenEXR magic");
        // GPU: only in builds with the `gpu` feature; otherwise a clear message, never a crash
        let g = e.run(&["render-gpu", &e.p("p11.json"), &e.p("g.png")]);
        if g.ok {
            assert!(Path::new(&e.p("g.png")).exists());
            eprintln!("[e2e] gpu: rendered");
        } else {
            assert!(g.text.contains("gpu feature") || g.text.contains("adapter"), "unexpected render-gpu failure: {}", g.text);
            assert!(!g.text.contains("panicked"));
            eprintln!("[e2e] gpu: skipped ({})", g.text.lines().next().unwrap_or(""));
        }
    });

    let secs = total.elapsed().as_secs_f64();
    eprintln!("[e2e] total {secs:.1} s ({} stages)", e.timings.len());
    if let Ok(p) = std::env::var("OSATURA_E2E_TIMINGS") {
        let mut s = String::new();
        for (n, t) in &e.timings {
            s.push_str(&format!("{n:<32} {t:7.2} s\n"));
        }
        s.push_str(&format!("{:<32} {secs:7.2} s\n", "total"));
        let _ = fs::write(p, s);
    }
}

/// A project with a layout container, layer effects, a state machine and a locked layer, cut down to 24 frames (the
/// machine's segments are trimmed so it stays valid): lod, export-fit and ci init + run must all work on it.
#[test]
fn container_layerfx_machine_locked_project_through_lod_fit_and_ci() {
    let e = Env::new();
    e.ok(&["demo", &e.p("demo.json")]);
    let mut p = e.json("demo.json");
    p["frames"] = json!(24);
    p["channels"] = json!(p["channels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let mut c = c.clone();
            if let Some(k) = c["keys"].as_array_mut() {
                k.retain(|k| k["frame"].as_f64().unwrap_or(0.0) < 24.0);
            }
            c
        })
        .filter(|c| c["keys"].as_array().map(|k| !k.is_empty()).unwrap_or(true))
        .collect::<Vec<_>>());
    let layers = p["layers"].as_array_mut().unwrap();
    for l in layers.iter_mut() {
        match l["name"].as_str().unwrap_or("") {
            "ground" => l["locked"] = json!(true),
            "sun" => l["effects"] = json!([{"enabled": true, "type": "blur", "radius": 4.0}, {"enabled": true, "type": "drop_shadow", "dx": 6.0, "dy": 6.0, "blur": 3.0, "color": "#000000", "opacity": 0.5}]),
            _ => {}
        }
    }
    p["containers"] = json!([{"id": "row1", "members": [{"layer": "sun", "grow": 0.0, "shrink": 1.0}, {"layer": "blush", "grow": 1.0, "shrink": 1.0, "resize": true}],
        "axis": "row", "gap": 10.0, "padding": [10.0, 10.0, 10.0, 10.0], "justify": "start", "align": "start", "wrap": "none",
        "rect": {"kind": "pin", "left": 10.0, "top": 10.0, "right": 10.0, "height": 100.0}}]);
    p["machines"] = json!([{"name": "m", "inputs": [{"name": "go", "kind": "bool", "default": 0.0, "hold": false}], "layers": [{"name": "base", "entry": "idle",
        "states": [{"name": "idle", "source": {"kind": "segment", "start": 0.0, "end": 12.0}, "mode": "loop", "speed": 1.0, "blend_in": 0.0},
                   {"name": "act", "source": {"kind": "segment", "start": 12.0, "end": 23.0}, "mode": "loop", "speed": 1.0, "blend_in": 0.0}],
        "transitions": [{"from": "idle", "to": "act", "when": [{"input": "go", "op": "eq", "value": 1.0}], "duration": 0.1, "ease": {"type": "linear"}}]}]}]);
    e.put("c.json", &p);
    e.ok(&["check", &e.p("c.json")]);
    // lod: the container members must be kept or skipped, never a fatal error
    let out = e.ok(&["lod", &e.p("c.json"), "--tolerance-px", "50", "--levels", "3", "--out-dir", &e.p("lod"), "--frames", "6"]);
    assert!(!out.to_lowercase().contains("error"), "{out}");
    assert!(e.dir.join("lod").is_dir());
    // export-fit
    e.ok(&["export-fit", &e.p("c.json"), &e.p("c.html"), "--max-size", "600KB", "--format", "html", "--frames", "4"]);
    assert!(fs::metadata(e.p("c.html")).unwrap().len() > 100);
    // ci init + run
    let dir = e.dir.join("ci2");
    fs::create_dir_all(&dir).unwrap();
    fs::copy(e.p("c.json"), dir.join("p.json")).unwrap();
    let init = e.run_in(&dir, &["ci", "init", "p.json"]);
    assert!(init.ok, "{}", init.text);
    let run = e.run_in(&dir, &["ci", "run", "--quiet"]);
    assert!(run.ok, "ci run on the container project:\n{}", run.text);
}
