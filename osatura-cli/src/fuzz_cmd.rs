// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! `osatura fuzz` and `osatura fuzz-grid`: pose fuzzing and stress heat-maps (see `osatura_core::fuzz`).

use osatura_core::fuzz::{self, FuzzOptions, ParamOptions, Strategy};
use osatura_core::posesuggest::PoseDb;
use osatura_core::Project;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn num<T: std::str::FromStr>(opts: &HashMap<String, String>, k: &str, d: T) -> Result<T, String> {
    match opts.get(k) {
        None => Ok(d),
        Some(s) => s.parse::<T>().map_err(|_| format!("{k} needs a number, got '{s}'")),
    }
}

fn load(path: &str) -> Result<Project, String> {
    crate::load_project(path)
}

fn param_options(opts: &HashMap<String, String>, flags: &[String]) -> Result<ParamOptions, String> {
    let mut po = ParamOptions { limit_margin: num(opts, "--margin", 0.0f32)?, ..ParamOptions::default() };
    if let Some(s) = opts.get("--props") {
        po.bone_props = s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
        for k in &po.bone_props {
            if !["angle", "scale", "offx", "offy"].contains(&k.as_str()) {
                return Err(format!("--props: unknown kind '{k}' (angle, scale, offx, offy)"));
            }
        }
    }
    if let Some(s) = opts.get("--only") {
        po.only_bones = s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
    }
    if flags.iter().any(|f| f == "--no-deformers") {
        po.deformers = false;
    }
    Ok(po)
}

/// `--only` must name bones (or IK targets) of the project and `--frame` must be a frame of its timeline; a typo used
/// to produce an empty or meaningless run.
fn check_selection(project: &Project, po: &ParamOptions, frame: f32) -> Result<(), String> {
    for n in &po.only_bones {
        if !project.bones.iter().any(|b| &b.name == n) && !project.ik.iter().any(|c| &c.target == n) {
            let names: Vec<&str> = project.bones.iter().map(|b| b.name.as_str()).take(12).collect();
            return Err(format!("--only: no bone named '{n}' (bones: {}{})", names.join(", "), if project.bones.len() > 12 { ", ..." } else { "" }));
        }
    }
    let last = project.frames.saturating_sub(1) as f32;
    if !frame.is_finite() || frame < 0.0 || frame > last {
        return Err(format!("--frame {frame} is outside the animation (0..={last})"));
    }
    Ok(())
}

pub fn fuzz(path: &str, opts: &HashMap<String, String>, flags: &[String]) -> Result<(), String> {
    let project = load(path)?;
    let dir = Path::new(path).parent().filter(|p| !p.as_os_str().is_empty()).map(|p| p.to_path_buf()).unwrap_or_else(|| ".".into());
    let mut o = FuzzOptions { samples: num(opts, "--samples", 200usize)?, ..FuzzOptions::default() };
    o.seed = num(opts, "--seed", 1u64)?;
    o.frame = num(opts, "--frame", 0.0f32)?;
    o.strategy = Strategy::parse(opts.get("--strategy").map(String::as_str).unwrap_or("random"))?;
    if let Some(s) = opts.get("--ignore") {
        o.ignore = s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
    }
    o.minimize = flags.iter().any(|f| f == "--minimize");
    o.params = param_options(opts, flags)?;
    check_selection(&project, &o.params, o.frame)?;
    if let Some(db) = opts.get("--db") {
        o.db = Some(PoseDb::from_json(&fs::read_to_string(db).map_err(|e| format!("cannot read {db}: {e}"))?)?);
    }
    let max_rate: f32 = num(opts, "--max-fail-rate", 0.0f32)?;
    // A gate is asked for with --gate (ceiling 0 unless --max-fail-rate says otherwise) or by naming a ceiling. Without
    // either, fuzz is a report: it prints the summary and exits 0 however many samples failed (random poses of a rig with
    // IK fail often, which is information, not an error). `ci run`'s fuzz check is a gate of its own and is not affected.
    let gate = flags.iter().any(|f| f == "--gate") || opts.contains_key("--max-fail-rate");
    if !(0.0..=1.0).contains(&max_rate) {
        return Err("--max-fail-rate needs a fraction between 0 and 1".into());
    }
    let rep = fuzz::run(&project, Some(&dir), &o)?;
    if let Some(r) = opts.get("--report") {
        write_out(r, rep.to_json().as_bytes())?;
        println!("wrote {r}");
    }
    if let Some(h) = opts.get("--heatmap") {
        write_out(h, fuzz::heatmap_svg(&rep).as_bytes())?;
        println!("wrote {h}");
    }
    let s = &rep.summary;
    println!(
        "fuzz '{}': {} samples ({}, seed {}, frame {}), {} parameters, {} failures ({:.1}%), max severity {:.2}, baseline {:.2}, {:.2} ms/sample",
        rep.project, s.samples, rep.strategy, rep.seed, rep.frame, rep.params.len(), s.failures, 100.0 * s.failure_rate, s.max_severity, s.baseline_severity, s.ms_per_sample
    );
    for (k, n) in &s.by_kind {
        println!("  {k}: {n}");
    }
    for b in rep.per_bone.iter().take(5).filter(|b| b.rate > 0.0) {
        println!("  extreme-range failure excess  {:<24} {:.0}%", b.name, 100.0 * b.rate);
    }
    if let Some(p) = rep.pairs.first().filter(|p| p.synergy > 0.0) {
        println!("  strongest interaction: {} + {} (cell failure rate {:.0}% over {} samples)", p.a, p.b, 100.0 * p.cell_rate, p.cell_samples);
    }
    if let Some(m) = &rep.minimal {
        println!("  minimal reproducer ({}, severity {:.2}, {} evaluations{}):", m.kind, m.severity, m.evaluations, if m.verified { "" } else { ", NOT verified" });
        for q in &m.params {
            println!("    {} = {:.3}   (rest {:.3})", q.id, q.value, q.rest);
        }
    }
    if gate && s.failure_rate > max_rate {
        return Err(format!("fuzz: failure rate {:.2}% exceeds --max-fail-rate {:.2}%", 100.0 * s.failure_rate, 100.0 * max_rate));
    }
    if !gate && s.failures > 0 {
        println!("(report only: exit 0; add --gate, or --max-fail-rate R, to fail above a failure rate)");
    }
    Ok(())
}

/// Atomic, symlink-refusing output (parent directories are not created: the directory must exist).
fn write_out(path: &str, bytes: &[u8]) -> Result<(), String> {
    osatura_core::fsutil::write_atomic(Path::new(path), bytes)
}

pub fn grid(path: &str, out: &str, opts: &HashMap<String, String>, flags: &[String]) -> Result<(), String> {
    let project = load(path)?;
    let dir = Path::new(path).parent().filter(|p| !p.as_os_str().is_empty()).map(|p| p.to_path_buf()).unwrap_or_else(|| ".".into());
    let x = opts.get("--x").ok_or("fuzz-grid needs --x PARAM (for example bone:arm_up:angle)")?;
    let y = opts.get("--y").ok_or("fuzz-grid needs --y PARAM")?;
    let res: usize = num(opts, "--res", 32usize)?;
    let (po, frame) = (param_options(opts, flags)?, num(opts, "--frame", 0.0f32)?);
    check_selection(&project, &po, frame)?;
    let g = fuzz::grid(&project, Some(&dir), &po, fuzz::Thresholds::default(), frame, x, y, res)?;
    let lower = out.to_ascii_lowercase();
    if lower.ends_with(".json") {
        write_out(out, serde_json::to_string_pretty(&g).map_err(|e| e.to_string())?.as_bytes())?;
    } else if lower.ends_with(".png") {
        let cell = (512 / g.res).max(4) as u32;
        let side = cell * g.res as u32;
        let mut pm = tiny_skia::Pixmap::new(side, side).ok_or("grid too large")?;
        for j in 0..g.res {
            for i in 0..g.res {
                let c = fuzz::stress_rgb(g.severity[j * g.res + i]);
                let mut paint = tiny_skia::Paint::default();
                paint.set_color_rgba8(c[0], c[1], c[2], 255);
                let r = tiny_skia::Rect::from_xywh((i as u32 * cell) as f32, (j as u32 * cell) as f32, cell as f32, cell as f32).ok_or("bad rect")?;
                pm.fill_rect(r, &paint, tiny_skia::Transform::identity(), None);
            }
        }
        write_out(out, &pm.encode_png().map_err(|e| e.to_string())?)?;
    } else {
        write_out(out, fuzz::grid_svg(&g).as_bytes())?;
    }
    println!("wrote {out} ({}x{} cells, {:.1}% failing; x = {}, y = {})", g.res, g.res, 100.0 * g.failure_rate, g.x.id, g.y.id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(kv: &[(&str, &str)]) -> HashMap<String, String> {
        kv.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn fuzz_and_grid_commands_write_outputs_and_gate_on_failure_rate() {
        let scratch = osatura_core::fsutil::ScratchDir::new("armfuzz_cli");
        let d = scratch.to_path_buf();
        let f = d.join("p.json");
        let json = serde_json::to_string(&osatura_core::demo::project()).unwrap();
        fs::write(&f, &json).unwrap();
        let (fp, rep, svg) = (f.to_str().unwrap(), d.join("r.json"), d.join("h.svg"));
        let o = opts(&[("--samples", "60"), ("--strategy", "lhs"), ("--report", rep.to_str().unwrap()), ("--heatmap", svg.to_str().unwrap()), ("--max-fail-rate", "1")]);
        fuzz(fp, &o, &["--minimize".to_string()]).unwrap();
        let j: serde_json::Value = serde_json::from_str(&fs::read_to_string(&rep).unwrap()).unwrap();
        assert_eq!(j["schema"], "osatura-fuzz-report");
        assert!(fs::read_to_string(&svg).unwrap().starts_with("<svg"));
        // report by default: the demo rig fails somewhere in its full range, and the exit is still 0
        fuzz(fp, &opts(&[("--samples", "60")]), &[]).unwrap();
        // gate: --gate allows no failures, an explicit ceiling gates too
        let e = fuzz(fp, &opts(&[("--samples", "60")]), &["--gate".to_string()]).unwrap_err();
        assert!(e.contains("exceeds --max-fail-rate 0.00%"), "{e}");
        let e = fuzz(fp, &opts(&[("--samples", "60"), ("--max-fail-rate", "0")]), &[]).unwrap_err();
        assert!(e.contains("exceeds --max-fail-rate"), "{e}");
        fuzz(fp, &opts(&[("--samples", "60")]), &["--gate".to_string()]).unwrap_err();
        // a ceiling of 1 (100 %) never trips, with or without --gate
        fuzz(fp, &opts(&[("--samples", "60"), ("--max-fail-rate", "1")]), &["--gate".to_string()]).unwrap();
        // argument errors
        assert!(fuzz(fp, &opts(&[("--strategy", "nope")]), &[]).is_err());
        assert!(fuzz(fp, &opts(&[("--samples", "x")]), &[]).is_err());
        assert!(fuzz(fp, &opts(&[("--props", "angel")]), &[]).is_err());
        assert!(fuzz(fp, &opts(&[("--max-fail-rate", "2")]), &[]).is_err());
        // wave 4: unknown --only names and frames outside the animation are errors that name the culprit
        let e = fuzz(fp, &opts(&[("--only", "arm_up,no_such_bone")]), &[]).unwrap_err();
        assert!(e.contains("no_such_bone"), "{e}");
        let e = fuzz(fp, &opts(&[("--frame", "9999")]), &[]).unwrap_err();
        assert!(e.contains("outside the animation"), "{e}");
        assert!(fuzz(fp, &opts(&[("--frame", "-1")]), &[]).is_err());
        assert!(fuzz(fp, &opts(&[("--frame", "NaN")]), &[]).is_err());
        assert!(fuzz("/nonexistent/p.json", &opts(&[]), &[]).is_err());
        // grid: svg, png, json
        for name in ["g.svg", "g.png", "g.json"] {
            let out = d.join(name);
            grid(fp, out.to_str().unwrap(), &opts(&[("--x", "bone:arm_up:angle"), ("--y", "bone:arm_low:angle"), ("--res", "6")]), &[]).unwrap();
            assert!(fs::metadata(&out).unwrap().len() > 100);
        }
        assert!(fs::read(d.join("g.png")).unwrap().starts_with(&[0x89, b'P', b'N', b'G']));
        assert!(grid(fp, d.join("x.svg").to_str().unwrap(), &opts(&[("--y", "bone:arm_low:angle")]), &[]).is_err());
        assert_eq!(fs::read_to_string(&f).unwrap(), json); // input untouched
        let _ = fs::remove_dir_all(&d);
    }
}
