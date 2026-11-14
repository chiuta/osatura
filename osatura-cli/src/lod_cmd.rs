// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! `osatura lod` and `osatura lod-pick` (see `osatura_core::lod`).

use osatura_core::lod::{self, Lod, LodOptions, VerifyOptions, VerifyReport};
use osatura_core::Project;
use std::collections::HashMap;
#[cfg(test)]
use std::fs;

fn load(path: &str) -> Result<Project, String> {
    crate::load_project(path)
}

fn write(path: &str, bytes: &[u8]) -> Result<(), String> {
    osatura_core::fsutil::write_atomic_mkdir(std::path::Path::new(path), bytes)
}

fn parse_wh(s: &str, what: &str) -> Result<(f64, f64), String> {
    let (w, h) = s
        .split_once('x')
        .ok_or_else(|| format!("{what} needs WxH, got '{s}'"))?;
    let w: f64 = w.parse().map_err(|_| format!("{what}: bad width '{w}'"))?;
    let h: f64 = h.parse().map_err(|_| format!("{what}: bad height '{h}'"))?;
    if !(w > 0.0 && h > 0.0) {
        return Err(format!("{what} must be positive"));
    }
    Ok((w, h))
}

fn num(opts: &HashMap<String, String>, key: &str, default: f64) -> Result<f64, String> {
    match opts.get(key) {
        None => Ok(default),
        Some(v) => v
            .parse()
            .map_err(|_| format!("{key} needs a number, got '{v}'")),
    }
}

fn options(
    p: &Project,
    scale: f32,
    opts: &HashMap<String, String>,
    default_tol: f64,
) -> Result<LodOptions, String> {
    let s = match opts.get("--size") {
        Some(sz) => {
            let (w, h) = parse_wh(sz, "--size")?;
            lod::fit_scale(p, w, h)
        }
        None => scale as f64,
    };
    let levels = num(opts, "--levels", 4.0)?;
    if levels.fract() != 0.0 || levels < 1.0 {
        return Err("--levels needs a whole number of at least 1".into());
    }
    let mut o = LodOptions::new(
        num(opts, "--tolerance-px", default_tol)?,
        s,
        levels as usize,
    );
    o.ratio = num(opts, "--ratio", 2.0)?;
    o.check()?;
    Ok(o)
}

/// `lod <project> --tolerance-px 0.5 --scale S|--size WxH --levels 4 [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]`.
/// Exit 1 if `--verify` finds a measured deviation above the declared bound.
pub fn lod(
    path: &str,
    scale: f32,
    opts: &HashMap<String, String>,
    flags: &[String],
) -> Result<(), String> {
    let p = load(path)?;
    osatura_core::export::check_budget(&p)?;
    let o = options(&p, scale, opts, 0.5)?;
    let levels = lod::build(&p, &o)?;
    let mut reports: Vec<VerifyReport> = Vec::new();
    let mut violation = false;
    if flags.iter().any(|f| f == "--verify") {
        let mut vo = VerifyOptions::new(o.scale);
        if let Some(n) = opts.get("--frames") {
            vo.frames = n
                .parse()
                .ok()
                .filter(|n: &usize| (1..=1000).contains(n))
                .ok_or("--frames needs a whole number between 1 and 1000")?;
        }
        for l in levels.iter().skip(1) {
            let r = lod::verify(&p, l, &vo)?;
            if !r.holds {
                violation = true;
            }
            reports.push(r);
        }
    }
    let dir = opts
        .get("--out-dir")
        .cloned()
        .unwrap_or_else(|| ".".to_string());
    let stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string();
    for l in levels.iter().skip(1) {
        let out = format!("{dir}/{stem}_L{}.json", l.record.level);
        write(&out, l.project.to_json_pretty().as_bytes())?;
    }
    print!("{}", lod::table(&levels, &reports));
    println!("scale {} px per unit; level k tolerance = {} px x {}^(k-1); files {dir}/{stem}_L1..L{}.json", o.scale, o.tolerance_px, o.ratio, o.levels);
    for l in levels.iter().skip(1) {
        for n in &l.record.notes {
            println!("  L{}: {n}", l.record.level);
        }
    }
    if let Some(rp) = opts.get("--report") {
        write(
            rp,
            serde_json::to_string_pretty(&lod::report_json(&levels, &reports))
                .unwrap()
                .as_bytes(),
        )?;
    }
    if violation {
        return Err("verification found a measured deviation above the declared bound (this is a bug in lod)".into());
    }
    Ok(())
}

/// `lod-pick <project> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--ratio 2] [--out chosen.json]`.
pub fn pick(path: &str, opts: &HashMap<String, String>) -> Result<(), String> {
    let p = load(path)?;
    let (w, h) = parse_wh(
        opts.get("--screen-px")
            .ok_or("lod-pick needs --screen-px WxH")?,
        "--screen-px",
    )?;
    let max_px = num(opts, "--max-px", 0.5)?;
    if !matches!(
        max_px.partial_cmp(&0.0),
        Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
    ) {
        return Err("--max-px must not be negative".into());
    }
    let screen = lod::fit_scale(&p, w, h);
    let mut o = options(&p, screen as f32, &HashMap::new(), 0.25)?;
    // reference tolerance, count and ratio from the command line, scale = the screen scale
    let tol = num(opts, "--tolerance-px", 0.25)?;
    let levels = num(opts, "--levels", 4.0)?;
    o.tolerance_px = tol;
    o.levels = levels as usize;
    o.ratio = num(opts, "--ratio", 2.0)?;
    o.scale = screen;
    o.check()?;
    let levels: Vec<Lod> = lod::build(&p, &o)?;
    let recs: Vec<_> = levels.iter().map(|l| l.record.clone()).collect();
    let k = lod::pick_level(&recs, screen, max_px);
    print!("{}", lod::table(&levels, &[]));
    let r = &levels[k].record;
    println!("screen {w}x{h} -> scale {screen:.4} px/unit; budget {max_px} px -> level L{k} (declared {:.4} px, {} anchors, {} mesh vertices, {} keys)", r.declared_px, r.stats.path_anchors, r.stats.mesh_vertices, r.stats.keys);
    if let Some(out) = opts.get("--out") {
        write(out, levels[k].project.to_json_pretty().as_bytes())?;
        println!("wrote {out}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use osatura_core::math::Vec2;
    use osatura_core::path::VPath;
    use osatura_core::project::Layer;

    fn setup(tag: &str) -> (osatura_core::fsutil::ScratchDir, String) {
        let d = osatura_core::fsutil::ScratchDir::new(&format!("osatura-lod-{tag}"));
        let mut p = Project::new("lod", 200.0, 200.0, 12.0, 6);
        let pts: Vec<Vec2> = (0..60)
            .map(|i| {
                let a = i as f32 / 60.0 * std::f32::consts::TAU;
                Vec2::new(100.0 + 50.0 * a.cos(), 100.0 + 50.0 * a.sin())
            })
            .collect();
        p.layers = vec![Layer::vector(
            "blob",
            vec![VPath::smooth_through("c", &pts, true, 1.0).with_fill("#2255cc")],
        )];
        let path = d.join("p.json");
        fs::write(&path, p.to_json_pretty()).unwrap();
        (d, path.to_string_lossy().into_owned())
    }

    fn opts(kv: &[(&str, &str)]) -> HashMap<String, String> {
        kv.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn lod_writes_levels_and_a_report_and_verifies() {
        let (d, path) = setup("run");
        let out = d.join("out");
        let rep = d.join("r.json");
        lod(
            &path,
            1.0,
            &opts(&[
                ("--tolerance-px", "1"),
                ("--levels", "3"),
                ("--out-dir", out.to_str().unwrap()),
                ("--report", rep.to_str().unwrap()),
                ("--frames", "4"),
            ]),
            &["--verify".to_string()],
        )
        .unwrap();
        for k in 1..=3 {
            let p = load(out.join(format!("p_L{k}.json")).to_str().unwrap()).unwrap();
            assert_eq!(p.lods.len(), 1);
            assert_eq!(p.lods[0].level, k);
        }
        let r: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&rep).unwrap()).unwrap();
        assert_eq!(r["levels"].as_array().unwrap().len(), 4);
        assert_eq!(r["verify"].as_array().unwrap().len(), 3);
        assert!(r["verify"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["holds"] == true));
        // deterministic files
        let first = fs::read(out.join("p_L2.json")).unwrap();
        lod(
            &path,
            1.0,
            &opts(&[
                ("--tolerance-px", "1"),
                ("--levels", "3"),
                ("--out-dir", out.to_str().unwrap()),
            ]),
            &[],
        )
        .unwrap();
        assert_eq!(first, fs::read(out.join("p_L2.json")).unwrap());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn lod_rejects_bad_arguments() {
        let (d, path) = setup("bad");
        assert!(lod(&path, 1.0, &opts(&[("--tolerance-px", "0")]), &[]).is_err());
        assert!(lod(&path, 1.0, &opts(&[("--levels", "2.5")]), &[]).is_err());
        assert!(lod(&path, 1.0, &opts(&[("--size", "wide")]), &[]).is_err());
        assert!(lod("/nonexistent/p.json", 1.0, &opts(&[]), &[]).is_err());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn lod_pick_chooses_by_screen_size_and_writes_the_level() {
        let (d, path) = setup("pick");
        let chosen = d.join("chosen.json");
        pick(
            &path,
            &opts(&[
                ("--screen-px", "400x400"),
                ("--max-px", "5"),
                ("--out", chosen.to_str().unwrap()),
            ]),
        )
        .unwrap();
        let p = load(chosen.to_str().unwrap()).unwrap();
        assert_eq!(p.lods.len(), 1);
        assert!(
            p.lods[0].level >= 1,
            "a 5 px budget should select a reduced level"
        );
        assert!(p.lods[0].declared_px <= 5.0 + 1e-9);
        // a zero budget keeps the original
        pick(
            &path,
            &opts(&[
                ("--screen-px", "400x400"),
                ("--max-px", "0"),
                ("--out", chosen.to_str().unwrap()),
            ]),
        )
        .unwrap();
        assert!(load(chosen.to_str().unwrap()).unwrap().lods.is_empty());
        assert!(pick(&path, &opts(&[])).is_err());
        assert!(pick(&path, &opts(&[("--screen-px", "0x10")])).is_err());
        let _ = fs::remove_dir_all(d);
    }
}
