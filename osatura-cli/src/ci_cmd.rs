// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! `osatura ci ...`: animation CI (see `osatura_core::ci` and docs/CI.md).
//! Exit codes: 0 all pass, 1 a check failed (or the images differ), 2 something could not be evaluated.

use osatura_core::ci::{self, RunOptions, Status};
use osatura_core::percep::{self, CompareOptions, DeMetric, Img};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

/// Exit code requested by a `ci` command; read by `main` when the command returns `Err`.
pub static EXIT: AtomicU8 = AtomicU8::new(0);

const VALUE: &[&str] = &["--junit", "--html", "--json", "--only", "--artifacts", "--out", "--metric", "--tol", "--ignore", "--side", "--overlay", "--flicker", "--pixel-de", "--aa", "--manifest"];
const FLAGS: &[&str] = &["--update", "--bless", "--prune", "--no-goldens", "--quiet"];
/// Global flags every command accepts, also after the subcommand (`ci run --allow-external-assets`).
const GLOBAL: &[&str] = &["--allow-external-assets", "--allow-empty", "--follow-symlinks"];

/// Does `ci` take a value after this option?
pub fn is_value_opt(o: &str) -> bool {
    VALUE.contains(&o)
}

/// Is this a boolean option of `ci`?
pub fn is_flag(o: &str) -> bool {
    FLAGS.contains(&o) || GLOBAL.contains(&o)
}

struct P {
    pos: Vec<String>,
    opts: HashMap<String, String>,
    flags: Vec<String>,
}

fn parse(raw: &[String]) -> Result<P, String> {
    let mut p = P { pos: vec![], opts: HashMap::new(), flags: vec![] };
    let mut i = 0;
    while i < raw.len() {
        let a = raw[i].as_str();
        if VALUE.contains(&a) {
            i += 1;
            let v = raw.get(i).ok_or_else(|| format!("{a} needs a value"))?;
            p.opts.insert(a.to_string(), v.clone());
        } else if FLAGS.contains(&a) || GLOBAL.contains(&a) {
            p.flags.push(a.to_string());
        } else if a.starts_with("--") {
            return Err(format!("unknown option {a} for ci (see: osatura ci)"));
        } else {
            p.pos.push(a.to_string());
        }
        i += 1;
    }
    Ok(p)
}

fn fail(code: u8, msg: String) -> Result<(), String> {
    EXIT.store(code, Ordering::SeqCst);
    Err(msg)
}

fn write(path: &str, body: &[u8]) -> Result<(), String> {
    osatura_core::fsutil::write_atomic_mkdir(Path::new(path), body)
}

pub fn run(raw: &[String]) -> Result<(), String> {
    let after: Vec<String> = raw.iter().skip_while(|a| a.as_str() != "ci").skip(1).cloned().collect();
    let p = match parse(&after) {
        Ok(p) => p,
        Err(e) => return fail(2, e),
    };
    let r = match p.pos.first().map(String::as_str) {
        Some("init") => init(&p),
        Some("run") => run_manifest(&p, false),
        Some("bless") => run_manifest(&p, true),
        Some("diff") => diff(&p),
        _ => Err("usage: osatura ci init <project> | run [manifest] | bless [manifest] | diff a.png b.png (see: osatura --help)".into()),
    };
    match r {
        Ok(()) => Ok(()),
        Err(e) => {
            if EXIT.load(Ordering::SeqCst) == 0 {
                EXIT.store(2, Ordering::SeqCst);
            }
            Err(e)
        }
    }
}

fn init(p: &P) -> Result<(), String> {
    let project = p.pos.get(1).ok_or("ci init needs a project file")?;
    let out = p.opts.get("--out").or(p.opts.get("--manifest")).cloned().unwrap_or_else(|| {
        Path::new(project).parent().filter(|d| !d.as_os_str().is_empty()).map(|d| d.join(ci::DEFAULT_MANIFEST)).unwrap_or_else(|| PathBuf::from(ci::DEFAULT_MANIFEST)).to_string_lossy().to_string()
    });
    if Path::new(&out).exists() {
        return Err(format!("{out} already exists; remove it or pass --out"));
    }
    let m = ci::init_manifest(Path::new(project), Path::new(&out))?;
    write(&out, serde_json::to_string_pretty(&m).unwrap().as_bytes())?;
    println!("wrote {out} with {} checks:", m.checks.len());
    for c in &m.checks {
        println!("  {:<12} {}", c.kind(), c.name());
    }
    if let Some(ci::Check::Fuzz(f)) = m.checks.iter().find(|c| matches!(c, ci::Check::Fuzz(_))) {
        println!("note: pose-fuzz max_fail_rate {:.2} was calibrated to the project as it is now (default full-range sampling)", f.max_fail_rate.unwrap_or(0.0));
    }
    if p.flags.iter().any(|f| f == "--no-goldens") {
        println!("goldens not written: run `osatura ci run {out} --update`");
        return Ok(());
    }
    let rep = ci::run(Path::new(&out), &RunOptions { update: true, ..Default::default() })?;
    let n: usize = rep.checks.iter().map(|c| c.frames.iter().filter(|f| f.status == Status::Updated).count()).sum();
    println!("wrote {n} golden frame(s) below {}", Path::new(&out).parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new(".")).join("goldens").display());
    if rep.exit_code() != 0 {
        return fail(rep.exit_code() as u8, "some checks fail on the project as it is; see `osatura ci run`".into());
    }
    Ok(())
}

fn run_manifest(p: &P, bless: bool) -> Result<(), String> {
    let manifest = p.pos.get(1).cloned().or(p.opts.get("--manifest").cloned()).unwrap_or_else(|| ci::DEFAULT_MANIFEST.to_string());
    let o = RunOptions {
        update: p.flags.iter().any(|f| f == "--update"),
        bless: bless || p.flags.iter().any(|f| f == "--bless"),
        prune: p.flags.iter().any(|f| f == "--prune"),
        only: p.opts.get("--only").map(|s| s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()).unwrap_or_default(),
        artifacts: p.opts.get("--artifacts").map(PathBuf::from),
    };
    let rep = ci::run(Path::new(&manifest), &o)?;
    if !p.flags.iter().any(|f| f == "--quiet") {
        for c in &rep.checks {
            println!("[{:<7}] {:<12} {:<18} {}", c.status.name().to_uppercase(), c.kind, c.name, c.message);
            for f in c.frames.iter().filter(|f| matches!(f.status, Status::Fail | Status::Error | Status::Blessed)) {
                println!("            frame {:>4}: {}", f.frame, f.message);
                for g in f.regions.iter().take(3) {
                    println!("                      region x={} y={} {}x{} px={} max_dE={:.1}", g.x, g.y, g.w, g.h, g.pixels, g.max_de);
                }
            }
        }
    }
    let s = &rep.summary;
    println!("{} checks: {} passed, {} failed, {} errors, {} info, {} updated, {} blessed ({:.1} s)", s.total, s.passed, s.failed, s.errors, s.info, s.updated, s.blessed, rep.duration_ms / 1000.0);
    if let Some(f) = p.opts.get("--junit") {
        write(f, ci::junit_xml(&rep).as_bytes())?;
    }
    if let Some(f) = p.opts.get("--html") {
        write(f, ci::html_report(&rep).as_bytes())?;
    }
    if let Some(f) = p.opts.get("--json") {
        write(f, rep.to_json().as_bytes())?;
    }
    match rep.exit_code() {
        0 => Ok(()),
        c => fail(c as u8, format!("osatura ci: {}", if c == 1 { "checks failed" } else { "checks could not be evaluated" })),
    }
}

fn read_png(path: &str) -> Result<Img, String> {
    let b = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    Img::from_png(&b).map_err(|e| format!("{path}: {e}"))
}

fn diff(p: &P) -> Result<(), String> {
    let (a, b) = match (p.pos.get(1), p.pos.get(2)) {
        (Some(a), Some(b)) => (a, b),
        _ => return Err("ci diff needs two PNG files".into()),
    };
    let metric = p.opts.get("--metric").map(String::as_str).unwrap_or("de2000");
    let (ia, ib) = (read_png(a)?, read_png(b)?);
    if (ia.w, ia.h) != (ib.w, ib.h) {
        return fail(1, format!("sizes differ: {}x{} vs {}x{}", ia.w, ia.h, ib.w, ib.h));
    }
    let num = |k: &str| -> Result<Option<f64>, String> { p.opts.get(k).map(|v| v.parse::<f64>().map_err(|_| format!("{k} needs a number, got '{v}'"))).transpose() };
    let de = match metric {
        "ssim" | "mse" => DeMetric::De2000,
        m => DeMetric::parse(m)?,
    };
    let mut ignore = vec![];
    if let Some(s) = p.opts.get("--ignore") {
        for r in s.split(';').filter(|x| !x.trim().is_empty()) {
            let v: Vec<i64> = r.split(',').map(|x| x.trim().parse::<i64>().map_err(|_| format!("--ignore needs x,y,w,h in pixels (several separated by ';'), got '{r}'"))).collect::<Result<_, _>>()?;
            if v.len() != 4 {
                return Err(format!("--ignore needs x,y,w,h, got '{r}'"));
            }
            ignore.push([v[0], v[1], v[2], v[3]]);
        }
    }
    let aa = match p.opts.get("--aa") {
        Some(v) => v.parse::<usize>().map_err(|_| "--aa needs a whole number (largest forgiven pixel group, 0 = off)")?,
        None => 2,
    };
    let o = CompareOptions { metric: de, pixel_de: num("--pixel-de")?.unwrap_or(1.0), aa_isolated: aa, ignore, ..CompareOptions::default() };
    let d = percep::compare(&ia, &ib, &o)?;
    println!("size        {}x{}", d.w, d.h);
    println!("SSIM        {:.5}   (luma)   channels min {:.5}   (R, G, B)", d.ssim, d.ssim_rgb);
    println!("MSE         {:.4}   PSNR {:.2} dB", d.mse, d.psnr);
    println!("{:<11} max {:.3}   mean {:.4}   (largest over all differing pixels: {:.3})", de.name(), d.max_de, d.mean_de, d.max_de_raw);
    println!("pixels      {} differ ({:.4}%), {} forgiven as anti-aliasing, {} ignored", d.diff_pixels, d.diff_fraction * 100.0, d.forgiven_pixels, d.ignored_pixels);
    println!("pHash       distance {}", d.phash_distance);
    println!("regions     {}", d.regions_total);
    for r in d.regions.iter().take(10) {
        println!("  x={} y={} {}x{}  {} px  max dE {:.2}", r.x, r.y, r.w, r.h, r.pixels, r.max_de);
    }
    if let Some(f) = p.opts.get("--out") {
        write(f, &percep::heatmap(&ia, &d).to_png()?)?;
        println!("wrote {f}");
    }
    if let Some(f) = p.opts.get("--side") {
        let hm = percep::heatmap(&ia, &d);
        write(f, &percep::side_by_side(&[&ia, &ib, &hm]).to_png()?)?;
        println!("wrote {f}");
    }
    if let Some(f) = p.opts.get("--overlay") {
        write(f, &percep::overlay(&ia, &ib)?.to_png()?)?;
        println!("wrote {f}");
    }
    if let Some(f) = p.opts.get("--flicker") {
        write(f, &percep::flicker_gif(&ia, &ib)?)?;
        println!("wrote {f}");
    }
    let (differs, why) = match metric {
        "ssim" => {
            // luma SSIM is blind to a hue change at equal luminance, so the per-channel SSIM gates too
            let t = num("--tol")?.unwrap_or(0.99);
            let mut why = Vec::new();
            if d.ssim < t {
                why.push(format!("luma SSIM {:.5} < {t}", d.ssim));
            }
            if d.ssim_rgb < t {
                why.push(format!("channel SSIM (min of R, G, B) {:.5} < {t}", d.ssim_rgb));
            }
            (!why.is_empty(), why.join("; "))
        }
        "mse" => {
            let t = num("--tol")?.unwrap_or(0.0);
            (d.mse > t, format!("MSE {:.4} > {t}", d.mse))
        }
        _ => {
            let t = num("--tol")?.unwrap_or(2.0);
            (d.max_de > t, format!("max {} {:.3} > {t}", de.name(), d.max_de))
        }
    };
    if differs {
        return fail(1, format!("images differ: {why}"));
    }
    println!("images match ({metric})");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static SERIAL: Mutex<()> = Mutex::new(());

    fn args(s: &str) -> Vec<String> {
        std::iter::once("ci").chain(s.split_whitespace()).map(String::from).collect()
    }

    fn tmp(tag: &str) -> osatura_core::fsutil::ScratchDir {
        osatura_core::fsutil::ScratchDir::new(&format!("osatura-cicmd-{tag}"))
    }

    fn exec(s: &str) -> (Result<(), String>, u8) {
        EXIT.store(0, Ordering::SeqCst);
        let r = run(&args(s));
        (r, EXIT.load(Ordering::SeqCst))
    }

    #[test]
    fn option_parsing_rejects_unknown_and_incomplete_options() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (r, code) = exec("run --bogus");
        assert!(r.unwrap_err().contains("unknown option --bogus"));
        assert_eq!(code, 2);
        let (r, code) = exec("run --junit");
        assert!(r.unwrap_err().contains("needs a value"));
        assert_eq!(code, 2);
        let (r, code) = exec("nothing");
        assert!(r.unwrap_err().starts_with("usage:"));
        assert_eq!(code, 2);
    }

    #[test]
    fn diff_exit_codes_and_outputs() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let d = tmp("diff");
        let a = Img::solid(32, 32, [255, 255, 255, 255]);
        let mut b = a.clone();
        for y in 8..24 {
            for x in 8..24 {
                b.set(x, y, [200, 0, 0, 255]);
            }
        }
        let (pa, pb, pc) = (d.join("a.png"), d.join("b.png"), d.join("c.png"));
        std::fs::write(&pa, a.to_png().unwrap()).unwrap();
        std::fs::write(&pb, b.to_png().unwrap()).unwrap();
        std::fs::write(&pc, Img::solid(16, 16, [0; 4]).to_png().unwrap()).unwrap();
        let s = |p: &PathBuf| p.to_string_lossy().to_string();
        let (r, code) = exec(&format!("diff {} {}", s(&pa), s(&pa)));
        assert!(r.is_ok() && code == 0);
        let heat = d.join("heat.png");
        let (r, code) = exec(&format!("diff {} {} --out {} --side {} --overlay {} --flicker {}", s(&pa), s(&pb), s(&heat), s(&d.join("s.png")), s(&d.join("o.png")), s(&d.join("f.gif"))));
        assert!(r.unwrap_err().contains("images differ"));
        assert_eq!(code, 1);
        for f in ["heat.png", "s.png", "o.png", "f.gif"] {
            assert!(d.join(f).exists(), "{f}");
        }
        let h = Img::from_png(&std::fs::read(&heat).unwrap()).unwrap();
        assert_eq!((h.w, h.h), (32, 32));
        // each metric reaches the same verdict on this clear difference, and an ignore mask removes it
        for m in ["ssim", "de2000", "de76", "mse"] {
            let (r, code) = exec(&format!("diff {} {} --metric {m}", s(&pa), s(&pb)));
            assert!(r.is_err() && code == 1, "{m}");
        }
        let (r, _) = exec(&format!("diff {} {} --ignore 0,0,32,32", s(&pa), s(&pb)));
        assert!(r.is_ok());
        // different sizes fail (1), unreadable input is an error (2)
        let (r, code) = exec(&format!("diff {} {}", s(&pa), s(&pc)));
        assert!(r.unwrap_err().contains("sizes differ") && code == 1);
        let (r, code) = exec(&format!("diff {} {}", s(&pa), s(&d.join("missing.png"))));
        assert!(r.is_err() && code == 2);
        let (r, code) = exec(&format!("diff {} {} --metric lab", s(&pa), s(&pb)));
        assert!(r.is_err() && code == 2);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// C9: red vs the green of equal luma is not a match under the ssim metric (luma SSIM alone said 1.0).
    #[test]
    fn ssim_verdict_sees_an_equiluminant_hue_change() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let d = tmp("hue");
        let (pa, pb) = (d.join("red.png"), d.join("green.png"));
        std::fs::write(&pa, Img::solid(32, 32, [255, 0, 0, 255]).to_png().unwrap()).unwrap();
        std::fs::write(&pb, Img::solid(32, 32, [0, 130, 0, 255]).to_png().unwrap()).unwrap();
        let s = |p: &PathBuf| p.to_string_lossy().to_string();
        let (r, code) = exec(&format!("diff {} {} --metric ssim", s(&pa), s(&pb)));
        let e = r.unwrap_err();
        assert!(code == 1 && e.contains("channel SSIM"), "{e}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn init_run_bless_and_reports_end_to_end() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let d = tmp("e2e");
        // one layer and no rig keeps init (which renders, fuzzes and exports) fast in debug builds
        let mut p = osatura_core::demo::project();
        p.frames = 4;
        p.layers.retain(|l| l.name == "sun");
        p.channels.clear();
        p.bones.clear();
        p.ik.clear();
        p.constraints.clear();
        p.springs.clear();
        let proj = d.join("p.json");
        std::fs::write(&proj, serde_json::to_string(&p).unwrap()).unwrap();
        let man = d.join("osatura-ci.json");
        let s = |p: &PathBuf| p.to_string_lossy().to_string();
        let (r, code) = exec(&format!("init {} --out {}", s(&proj), s(&man)));
        assert!(r.is_ok() && code == 0, "{r:?}");
        assert!(d.join("goldens/index.json").exists());
        let (r, _) = exec(&format!("init {}", s(&proj)));
        assert!(r.is_ok() || r.unwrap_err().contains("already exists"));
        // restrict to cheap checks so the test is quick in debug builds
        let (r, code) = exec(&format!("run {} --only validate,limits,frames --junit {} --html {} --json {}", s(&man), s(&d.join("j.xml")), s(&d.join("r.html")), s(&d.join("r.json"))));
        assert!(r.is_ok() && code == 0, "{r:?}");
        let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(d.join("r.json")).unwrap()).unwrap();
        assert_eq!(j["summary"]["failed"], 0);
        assert!(std::fs::read_to_string(d.join("j.xml")).unwrap().contains("<testsuites"));
        assert!(std::fs::read_to_string(d.join("r.html")).unwrap().contains("Osatura CI report"));
        // change the project: exit 1; bless: exit 0; unknown --only: exit 2
        let mut q = p.clone();
        q.layers.iter_mut().find(|l| l.name == "sun").unwrap().transform.position.x += 50.0;
        std::fs::write(&proj, serde_json::to_string(&q).unwrap()).unwrap();
        let (r, code) = exec(&format!("run {} --only frames", s(&man)));
        assert!(r.is_err() && code == 1);
        let (r, code) = exec(&format!("bless {} --only frames", s(&man)));
        assert!(r.is_ok() && code == 0);
        let (r, code) = exec(&format!("run {} --only frames", s(&man)));
        assert!(r.is_ok() && code == 0);
        let (r, code) = exec(&format!("run {} --only nope", s(&man)));
        assert!(r.is_err() && code == 2);
        let _ = std::fs::remove_dir_all(&d);
    }
}
