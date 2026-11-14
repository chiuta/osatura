// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! `face-train`, `face-eval`, `face-detect`, `face-track`, `face-synth` (see `osatura_core::facedet`).
//!
//! The detector is trained on procedural synthetic faces only. It is meant for stylised / cartoon
//! faces; it is not validated on photographs or webcams.

use osatura_core::autotrace::decode_image;
use osatura_core::facedet::{self, BoxTracker, DetectOpts, FaceModel, FaceResult, TrainOpts};
use osatura_core::facegen::{self, Gray};
use osatura_core::faceparam::NAMES;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

type Opts = HashMap<String, String>;

fn num(o: &Opts, k: &str, d: f64) -> Result<f64, String> {
    match o.get(k) {
        None => Ok(d),
        Some(v) => v.parse().map_err(|_| format!("{k} needs a number, got '{v}'")),
    }
}

fn write(path: &str, bytes: &[u8]) -> Result<(), String> {
    osatura_core::fsutil::write_atomic_mkdir(std::path::Path::new(path), bytes)
}

pub fn load_model(o: &Opts) -> Result<FaceModel, String> {
    match o.get("--model") {
        Some(p) => FaceModel::load(&fs::read(p).map_err(|e| format!("cannot read {p}: {e}"))?),
        None => FaceModel::builtin(),
    }
}

fn detect_opts(o: &Opts) -> Result<DetectOpts, String> {
    let mut d = DetectOpts { min_side: num(o, "--min-side", 0.0)? as f32, ..DetectOpts::default() };
    d.max_side = num(o, "--max-side", f64::INFINITY)? as f32;
    if let Some(t) = o.get("--threshold") {
        d.threshold = Some(t.parse().map_err(|_| format!("--threshold needs a number, got '{t}'"))?);
    }
    d.max_faces = num(o, "--max-faces", 8.0)? as usize;
    Ok(d)
}

fn load_gray(path: &Path, max_dim: f32) -> Result<(Gray, usize, usize), String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let img = decode_image(&bytes)?;
    let g = Gray::from_rgba(img.w, img.h, &img.rgba)?;
    let f = g.w.max(g.h) as f32 / max_dim;
    let (w, h) = (g.w, g.h);
    Ok((if max_dim > 0.0 && f > 1.0 { g.scaled_down(f) } else { g }, w, h))
}

pub fn train(o: &Opts) -> Result<(), String> {
    let out = o.get("--out").ok_or("face-train needs --out model.bin")?;
    let d = TrainOpts::default();
    for (k, max) in [("--samples", 1_000_000.0), ("--mine-rounds", 100.0), ("--scene", 4096.0)] {
        if num(o, k, 0.0)? > max {
            return Err(format!("{k} may be at most {max}"));
        }
    }
    let t = TrainOpts {
        samples: num(o, "--samples", d.samples as f64)? as usize,
        seed: num(o, "--seed", d.seed as f64)? as u64,
        mine_rounds: num(o, "--mine-rounds", d.mine_rounds as f64)? as usize,
        scene: num(o, "--scene", d.scene as f64)? as usize,
        reg_win: num(o, "--reg-win", d.reg_win as f64)? as usize,
        reg_cell: num(o, "--reg-cell", d.reg_cell as f64)? as usize,
        det_win: num(o, "--det-win", d.det_win as f64)? as usize,
        det_cell: num(o, "--det-cell", d.det_cell as f64)? as usize,
        ridge_lambda: num(o, "--ridge-lambda", d.ridge_lambda)?,
        stages: num(o, "--stages", d.stages as f64)? as usize,
        refine_win: num(o, "--refine-win", d.refine_win as f64)? as usize,
        stage_patch: num(o, "--stage-patch", d.stage_patch as f64)? as usize,
        stage_cell: num(o, "--stage-cell", d.stage_cell as f64)? as usize,
        stage_lambda: num(o, "--stage-lambda", d.stage_lambda)?,
        ..d
    };
    let t0 = std::time::Instant::now();
    let m = facedet::train(&t)?;
    let bytes = if out.ends_with(".json") { m.to_json().into_bytes() } else { m.to_bytes() };
    write(out, &bytes)?;
    println!("wrote {out} ({} bytes) in {:.1}s: {} samples, seed {}, mined negatives {}", bytes.len(), t0.elapsed().as_secs_f32(), t.samples, t.seed, m.meta["mined_negatives"]);
    println!("trained on synthetic faces only: expect stylised/cartoon faces to work, photographs and webcams are NOT validated");
    Ok(())
}

pub fn eval(o: &Opts, flags: &[String]) -> Result<(), String> {
    let m = load_model(o)?;
    let n = num(o, "--eval-n", 200.0)? as usize;
    let r = facedet::report(&m, n.clamp(10, 100_000), num(o, "--seed", 9.0)? as u64, num(o, "--scene", 128.0)? as usize)?;
    let text = serde_json::to_string_pretty(&r).unwrap_or_default();
    if let Some(p) = o.get("--report") {
        write(p, text.as_bytes())?;
    }
    if flags.iter().any(|f| f == "--json") || o.get("--report").is_none() {
        println!("{text}");
    } else {
        println!("wrote {}", o["--report"]);
    }
    Ok(())
}

fn result_json(r: &FaceResult) -> Value {
    json!({"box": {"cx": r.det.cx, "cy": r.det.cy, "side": r.det.side, "score": r.det.score},
           "points": r.pts.iter().map(|p| json!([p.x, p.y])).collect::<Vec<_>>()})
}

pub fn detect(path: &str, o: &Opts, flags: &[String]) -> Result<(), String> {
    let m = load_model(o)?;
    let (g, w, h) = load_gray(Path::new(path), num(o, "--max-dim", 0.0)? as f32)?;
    let k = w as f32 / g.w as f32; // back to the original pixel frame
    let faces: Vec<FaceResult> = m.detect(&g, &detect_opts(o)?);
    let scaled: Vec<FaceResult> = faces
        .iter()
        .map(|r| FaceResult { det: facedet::Detection { cx: r.det.cx * k, cy: r.det.cy * k, side: r.det.side * k, score: r.det.score }, pts: r.pts.iter().map(|p| *p * k).collect() })
        .collect();
    if let Some(ov) = o.get("--overlay") {
        let mut img = g.clone();
        for r in &faces {
            for p in &r.pts {
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (x, y) = (p.x.round() as isize + dx, p.y.round() as isize + dy);
                    if x >= 0 && y >= 0 && (x as usize) < img.w && (y as usize) < img.h {
                        img.px[y as usize * img.w + x as usize] = 1.0;
                    }
                }
            }
        }
        write(ov, &img.to_png()?)?;
    }
    if flags.iter().any(|f| f == "--json") {
        let v = json!({"image": path, "size": [w, h], "layout": "osatura23", "names": NAMES, "faces": scaled.iter().map(result_json).collect::<Vec<_>>()});
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    } else if scaled.is_empty() {
        println!("{path}: no face found (the model only knows synthetic / cartoon faces)");
    } else {
        for (i, r) in scaled.iter().enumerate() {
            println!("{path}: face {i}: box centre ({:.1}, {:.1}) side {:.1} score {:.2}", r.det.cx, r.det.cy, r.det.side, r.det.score);
        }
    }
    Ok(())
}

fn frame_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| matches!(p.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(), Some("png" | "jpg" | "jpeg")))
        .collect();
    v.sort();
    if v.is_empty() {
        return Err(format!("no .png/.jpg frames in {}", dir.display()));
    }
    Ok(v)
}

pub fn track(input: &str, out: &str, o: &Opts) -> Result<(), String> {
    let m = load_model(o)?;
    let fps = num(o, "--fps", 30.0)? as f32;
    if !(fps > 0.0 && fps.is_finite()) {
        return Err("--fps must be positive".into());
    }
    let max_dim = num(o, "--max-dim", 256.0)? as f32;
    let p = Path::new(input);
    let mut tmp: Option<osatura_core::fsutil::TempDir> = None;
    let files = if p.is_dir() {
        frame_files(p)?
    } else if p.is_file() {
        let guard = osatura_core::fsutil::private_temp_dir("osatura-face")?;
        let d = guard.path().to_path_buf();
        let st = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-protocol_whitelist", "file", "-i", &osatura_core::fsutil::file_arg(input), "-vf", &format!("fps={fps}"), "-start_number", "0"])
            .arg(d.join("%06d.png"))
            .status()
            .map_err(|e| format!("cannot run ffmpeg (needed for video input): {e}"))?;
        if !st.success() {
            return Err("ffmpeg could not decode the input".into());
        }
        tmp = Some(guard);
        frame_files(&d)?
    } else {
        return Err(format!("{input} is neither a frame directory nor a video file"));
    };
    let dopts = detect_opts(o)?;
    let mut tr = BoxTracker::default();
    let mut lines = Vec::new();
    let mut size = (0usize, 0usize);
    let mut found = 0usize;
    for (i, f) in files.iter().enumerate() {
        let (g, w, h) = load_gray(f, max_dim)?;
        if i == 0 {
            size = (w, h);
        } else if (w, h) != size {
            return Err(format!("{} has size {w}x{h}, the first frame is {}x{}", f.display(), size.0, size.1));
        }
        let t = i as f32 / fps;
        match tr.step(&m, &g, &dopts) {
            Some(r) => {
                found += 1;
                let pts: Vec<Value> = r.pts.iter().map(|q| json!([q.x / g.w as f32, q.y / g.h as f32])).collect();
                lines.push(json!({"t": t, "score": r.det.score, "p": pts}).to_string());
            }
            None => lines.push(json!({"t": t, "p": null}).to_string()),
        }
    }
    drop(tmp); // removes the private frame directory
    let head = json!({"format": "osatura-landmarks", "version": 1, "layout": "osatura23", "fps": fps, "size": [size.0, size.1], "source": "osatura facedet (synthetic-trained)"});
    let text = format!("{}\n{}\n", head, lines.join("\n"));
    write(out, text.as_bytes())?;
    println!("wrote {out}: {} frames, face found in {found}", files.len());
    if found * 2 < files.len() {
        println!("warning: the face was found in fewer than half of the frames; the model only knows synthetic / cartoon faces");
    }
    Ok(())
}

/// Synthetic head animation as a PNG sequence plus ground-truth landmarks and parameters.
pub fn synth(dir: &str, o: &Opts) -> Result<(), String> {
    let frames = num(o, "--frames", 90.0)? as usize;
    let size = num(o, "--size", 160.0)? as usize;
    let fps = num(o, "--fps", 30.0)? as f32;
    let seed = num(o, "--seed", 7.0)? as u64;
    if frames == 0 || frames > 100_000 || !(32..=1024).contains(&size) || (fps.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) {
        return Err("face-synth needs 1..100000 frames, a size of 32..1024 and a positive fps".into());
    }
    fs::create_dir_all(dir).map_err(|e| format!("cannot create {dir}: {e}"))?;
    let mut truth = vec![json!({"format": "osatura-landmarks", "version": 1, "layout": "osatura23", "fps": fps, "size": [size, size]}).to_string()];
    let mut params = Vec::new();
    for i in 0..frames {
        let t = i as f32 / fps;
        let s = facegen::anim_spec(seed, t, size as f32);
        let g = facegen::render_spec(&s, size, size, Some(seed + 1), &facegen::Post::NONE, i as u64)?;
        write(&format!("{dir}/{i:06}.png"), &g.to_png()?)?;
        let pts: Vec<Value> = s.landmarks().iter().map(|q| json!([q.x / size as f32, q.y / size as f32])).collect();
        truth.push(json!({"t": t, "p": pts}).to_string());
        params.push(json!({"t": t, "yaw": s.yaw, "pitch": s.pitch, "roll": s.roll, "eye_r_open": s.open_r, "eye_l_open": s.open_l, "mouth_open": s.mouth_open, "brow_raise": s.brow_raise}));
    }
    write(&format!("{dir}/truth.ndjson"), (truth.join("\n") + "\n").as_bytes())?;
    write(&format!("{dir}/params.json"), serde_json::to_string_pretty(&params).unwrap_or_default().as_bytes())?;
    println!("wrote {frames} frames ({size}x{size}), truth.ndjson and params.json to {dir}");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::*;

    /// Every `face-*` output (model, report, overlay, landmark stream, synthetic frames) goes through `write`: atomic,
    /// creates its folders, and refuses a symbolic link (the commands that need a trained model are too slow to run here).
    #[cfg(unix)]
    #[test]
    fn face_outputs_use_the_atomic_symlink_refusing_writer() {
        let d = osatura_core::fsutil::ScratchDir::new("osatura-face-write");
        let target = d.join("target.txt");
        fs::write(&target, "precious").unwrap();
        let link = d.join("model.bin");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let e = write(link.to_str().unwrap(), b"new").unwrap_err();
        assert!(e.contains("symbolic link"), "{e}");
        assert_eq!(fs::read_to_string(&target).unwrap(), "precious");
        // folders are created, and the file replaces an older one atomically
        let nested = d.join("a/b/report.json");
        write(nested.to_str().unwrap(), b"{}").unwrap();
        write(nested.to_str().unwrap(), b"{\"x\":1}").unwrap();
        assert_eq!(fs::read_to_string(&nested).unwrap(), "{\"x\":1}");
        assert!(fs::read_dir(d.join("a/b")).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().contains(".tmp-")), "no temporary file is left behind");
    }
}
