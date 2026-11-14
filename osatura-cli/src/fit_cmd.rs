// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! `osatura export-fit` and `osatura size-report` (see `osatura_core::sizefit`).

use osatura_core::sizefit::{self, FitOptions, Format, Measure, Search};
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

/// `export-fit <project> <out> --max-size 500KB [--format ..] [--min-ssim X] [--measure raw|gzip] [--frames N]
/// [--search ladder|descent|grid] [--report out.json] [--pareto out.csv]`.
/// Returns Err (exit 1) when the budget cannot be met; the smallest export found is still written.
pub fn export_fit(
    path: &str,
    out: &str,
    opts: &HashMap<String, String>,
    flags: &[String],
) -> Result<(), String> {
    let budget = sizefit::parse_size(
        opts.get("--max-size")
            .ok_or("export-fit needs --max-size (for example 500KB)")?,
    )?;
    let fmt = match opts.get("--format") {
        Some(f) => Format::parse(f)?,
        None => Format::from_path(out).ok_or("cannot tell the format from the output name; pass --format lottie|html|gif|apng|webm|project")?,
    };
    let mut o = FitOptions::new(fmt, budget);
    if let Some(m) = opts.get("--measure") {
        o.measure = Measure::parse(m)?;
    }
    if let Some(v) = opts.get("--min-ssim") {
        o.min_ssim = Some(
            v.parse()
                .map_err(|_| format!("--min-ssim needs a number, got '{v}'"))?,
        );
    }
    if let Some(v) = opts.get("--frames") {
        o.sample_frames = v
            .parse()
            .ok()
            .filter(|n: &usize| (1..=240).contains(n))
            .ok_or("--frames needs a whole number between 1 and 240")?;
    }
    if let Some(v) = opts.get("--search") {
        o.search = Search::parse(v)?;
    }
    if opts.contains_key("--pareto") && o.search == Search::Descent {
        o.search = Search::Grid;
    }
    if let Some(v) = opts.get("--max-seconds") {
        o.max_seconds = v
            .parse()
            .ok()
            .filter(|s: &f64| s.is_finite() && *s >= 0.0)
            .ok_or("--max-seconds needs a number >= 0 (0 = unlimited)")?;
    }
    o.audit = flags.iter().any(|f| f == "--audit");
    o.baseline = !flags.iter().any(|f| f == "--no-baseline");
    let p = load(path)?;
    let r = sizefit::fit(&p, o)?;
    write(out, &r.bytes)?;
    if let Some(rp) = opts.get("--report") {
        write(
            rp,
            serde_json::to_string_pretty(&r.to_json())
                .unwrap()
                .as_bytes(),
        )?;
    }
    if let Some(cp) = opts.get("--pareto") {
        write(cp, r.pareto_csv().as_bytes())?;
    }
    let c = &r.chosen;
    println!(
        "{} -> {out}: {} bytes raw, {} gzip (budget {} {}); SSIM mean {:.4}, worst {:.4} at frame {}, max CIEDE2000 {:.2}; {} candidates built",
        fmt.name(), c.raw, c.gzip, budget, r.options.measure.name(), r.quality.ssim_mean, r.quality.ssim_min, r.quality.worst_frame, r.quality.de_max, r.evaluations
    );
    if let Some((b, g)) = r.baseline {
        println!("existing exporter at defaults: {b} bytes raw, {g} gzip");
    }
    if !r.dead_layers.is_empty() {
        println!("dropped never-visible layers: {}", r.dead_layers.join(", "));
    }
    if r.time_limited {
        println!("time budget exhausted ({} s, checked inside every candidate): the search stopped early and the best result so far is kept (exit code 0 if it fits the size budget, 1 if not; raise --max-seconds or pass 0 for the full search; a single ffmpeg or quantiser call is not interruptible)", r.options.max_seconds);
    }
    if !r.floors.is_empty() {
        println!("knobs at their floor: {}", r.floors.join(", "));
    }
    if r.met {
        Ok(())
    } else {
        Err(format!(
            "budget not met: {}",
            r.reason.clone().unwrap_or_default()
        ))
    }
}

/// `size-report <project> [--tol T] [--json]`.
pub fn size_report(
    path: &str,
    opts: &HashMap<String, String>,
    flags: &[String],
) -> Result<(), String> {
    let tol: f32 = match opts.get("--tol") {
        Some(v) => v
            .parse()
            .map_err(|_| format!("--tol needs a number, got '{v}'"))?,
        None => 0.05,
    };
    let e = load(path)?;
    osatura_core::export::check_budget(&e)?;
    let r = sizefit::size_report(&e, tol)?;
    if flags.iter().any(|f| f == "--json") {
        println!("{}", serde_json::to_string_pretty(&r).unwrap());
    } else {
        print!("{}", sizefit::size_report_text(&r, 12));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use osatura_core::anim::{Ease, Track};
    use osatura_core::math::Vec2;
    use osatura_core::path::VPath;
    use osatura_core::project::{Channel, Layer, Prop};

    fn setup(tag: &str) -> (osatura_core::fsutil::ScratchDir, String) {
        let d = osatura_core::fsutil::ScratchDir::new(&format!("osatura-fit-{tag}"));
        let mut p = Project::new("fit", 64.0, 48.0, 12.0, 12);
        p.background = Some(osatura_core::style::Rgba::hex("#ffffff"));
        p.layers = vec![
            Layer::vector(
                "sq",
                vec![VPath::rect("r", 6.0, 8.0, 14.0, 14.0).with_fill("#cc2222")],
            ),
            Layer::vector(
                "ball",
                vec![VPath::circle("c", Vec2::new(40.0, 24.0), 7.0).with_fill("#2255cc")],
            ),
        ];
        p.channels = vec![Channel::keys(
            "sq",
            Prop::LayerX,
            Track::with_keys(&[(0.0, 0.0, Ease::Linear), (11.0, 30.0, Ease::Linear)]),
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
    fn export_fit_writes_the_file_and_the_report() {
        let (d, p) = setup("ok");
        let out = d.join("o.json").to_string_lossy().into_owned();
        let rep = d.join("r.json").to_string_lossy().into_owned();
        let csv = d.join("p.csv").to_string_lossy().into_owned();
        export_fit(
            &p,
            &out,
            &opts(&[
                ("--max-size", "60KB"),
                ("--report", &rep),
                ("--frames", "2"),
            ]),
            &["--no-baseline".to_string()],
        )
        .unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
        assert!(v["layers"].is_array() && v["fr"] == 12.0);
        let r: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&rep).unwrap()).unwrap();
        assert_eq!(r["schema"], "osatura.sizefit/1");
        assert_eq!(r["met"], true);
        assert!(r["chosen"]["raw_bytes"].as_u64().unwrap() <= 60_000);
        assert_eq!(
            r["chosen"]["raw_bytes"].as_u64().unwrap(),
            fs::metadata(&out).unwrap().len()
        );
        // --pareto switches to the grid and writes the CSV
        let po = d.join("o2.json").to_string_lossy().into_owned();
        export_fit(
            &p,
            &po,
            &opts(&[
                ("--max-size", "20KB"),
                ("--format", "project"),
                ("--pareto", &csv),
                ("--frames", "2"),
            ]),
            &["--no-baseline".to_string()],
        )
        .unwrap();
        let c = fs::read_to_string(&csv).unwrap();
        assert!(c.starts_with("position,decimals,") && c.lines().count() > 5);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn export_fit_reports_an_unmet_budget_and_still_writes_the_smallest_file() {
        let (d, p) = setup("unmet");
        let out = d.join("o.html").to_string_lossy().into_owned();
        let e = export_fit(
            &p,
            &out,
            &opts(&[("--max-size", "50"), ("--frames", "2")]),
            &["--no-baseline".to_string()],
        )
        .unwrap_err();
        assert!(e.contains("budget not met") && e.contains("floor"), "{e}");
        assert!(fs::metadata(&out).unwrap().len() > 50);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn export_fit_validates_its_arguments() {
        let (d, p) = setup("args");
        let out = d.join("o.json").to_string_lossy().into_owned();
        let none = ["--no-baseline".to_string()];
        assert!(export_fit(&p, &out, &opts(&[]), &none)
            .unwrap_err()
            .contains("--max-size"));
        assert!(export_fit(&p, &out, &opts(&[("--max-size", "lots")]), &none).is_err());
        assert!(export_fit(
            &p,
            &out,
            &opts(&[("--max-size", "5KB"), ("--measure", "zip")]),
            &none
        )
        .is_err());
        assert!(export_fit(
            &p,
            &out,
            &opts(&[("--max-size", "5KB"), ("--format", "svg")]),
            &none
        )
        .unwrap_err()
        .contains("not supported"));
        assert!(export_fit(
            &p,
            &out,
            &opts(&[("--max-size", "5KB"), ("--min-ssim", "high")]),
            &none
        )
        .is_err());
        assert!(export_fit(
            &p,
            &out,
            &opts(&[("--max-size", "5KB"), ("--frames", "0")]),
            &none
        )
        .is_err());
        assert!(export_fit(
            &p,
            &d.join("o.xyz").to_string_lossy(),
            &opts(&[("--max-size", "5KB")]),
            &none
        )
        .unwrap_err()
        .contains("format"));
        assert!(export_fit(
            "/nonexistent/p.json",
            &out,
            &opts(&[("--max-size", "5KB")]),
            &none
        )
        .is_err());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn size_report_runs_in_text_and_json_modes() {
        let (d, p) = setup("rep");
        size_report(&p, &opts(&[]), &[]).unwrap();
        size_report(&p, &opts(&[("--tol", "0.1")]), &["--json".to_string()]).unwrap();
        assert!(size_report(&p, &opts(&[("--tol", "x")]), &[]).is_err());
        let _ = fs::remove_dir_all(&d);
    }
}
