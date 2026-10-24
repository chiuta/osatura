// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `ci` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

pub(super) fn svg_frame(e: &Engine, name: &str, f: u32, store: &mut GoldenStore, o: &RunOptions) -> Result<FrameResult, String> {
    let now = render_svg(e, f);
    let hash = crate::vcs::sha256_hex(now.as_bytes())[..24].to_string();
    let have = store.get(name, f).cloned();
    if o.update {
        let changed = have.as_ref().map(|g| g.hash != hash).unwrap_or(true);
        store.put_text(name, f, &now)?;
        let mut r = frame_ok(f, if changed { Status::Updated } else { Status::Pass }, if changed { "golden written" } else { "unchanged" }, Some(hash));
        r.exact = !changed;
        return Ok(r);
    }
    let Some(g) = have else {
        if o.bless {
            store.put_text(name, f, &now)?;
            return Ok(frame_ok(f, Status::Blessed, "new golden accepted", Some(hash)));
        }
        return Ok(frame_ok(f, Status::Fail, "no golden stored (run with --update or bless)", None));
    };
    if g.hash == hash {
        let mut r = frame_ok(f, Status::Pass, "identical", Some(hash));
        r.exact = true;
        return Ok(r);
    }
    let old = store.load_text(&g)?;
    let line = old.lines().zip(now.lines()).position(|(a, b)| a != b).map(|i| i + 1).unwrap_or(old.lines().count().min(now.lines().count()) + 1);
    let msg = format!("SVG text differs from line {line} (golden {} lines, now {} lines)", old.lines().count(), now.lines().count());
    if o.bless {
        store.put_text(name, f, &now)?;
        return Ok(frame_ok(f, Status::Blessed, &msg, Some(hash)));
    }
    Ok(frame_ok(f, Status::Fail, &msg, Some(g.hash)))
}

pub(super) fn pixel_frame(e: &Engine, c: &RenderCheck, f: u32, th: &Resolved, store: &mut GoldenStore, o: &RunOptions) -> Result<FrameResult, String> {
    let now = render_img(e, &c.renderer, f, c.scale)?;
    let hash = now.content_hash()[..24].to_string();
    let have = store.get(&c.name, f).cloned();
    if o.update {
        let changed = have.as_ref().map(|g| g.hash != hash).unwrap_or(true);
        store.put_img(&c.name, f, &c.renderer, &now)?;
        let mut r = frame_ok(f, if changed { Status::Updated } else { Status::Pass }, if changed { "golden written" } else { "unchanged" }, Some(hash));
        r.exact = !changed;
        return Ok(r);
    }
    let Some(g) = have else {
        if o.bless {
            store.put_img(&c.name, f, &c.renderer, &now)?;
            return Ok(frame_ok(f, Status::Blessed, "new golden accepted", Some(hash)));
        }
        return Ok(frame_ok(f, Status::Fail, "no golden stored (run with --update or bless)", None));
    };
    if g.hash == hash {
        let mut r = frame_ok(f, Status::Pass, "identical", Some(hash));
        r.exact = true;
        return Ok(r);
    }
    let gold = store.load_img(&g)?;
    let mut r = frame_ok(f, Status::Fail, "", Some(g.hash.clone()));
    r.images.golden = Some(gold.to_png()?);
    r.images.actual = Some(now.to_png()?);
    if (gold.w, gold.h) != (now.w, now.h) {
        r.message = format!("size changed: golden {}x{}, now {}x{}", gold.w, gold.h, now.w, now.h);
    } else {
        let ignore: Vec<[i64; 4]> = c
            .ignore
            .iter()
            .map(|g| {
                let (x0, y0) = ((g[0] * c.scale).floor() as i64, (g[1] * c.scale).floor() as i64);
                let (x1, y1) = (((g[0] + g[2]) * c.scale).ceil() as i64, ((g[1] + g[3]) * c.scale).ceil() as i64);
                [x0, y0, x1 - x0, y1 - y0]
            })
            .collect();
        let d = percep::compare(&gold, &now, &compare_opts(th, ignore))?;
        let why = judge(&d, th);
        r.metrics = Some(metrics_of(&d));
        r.regions = d.regions.clone();
        r.regions_total = d.regions_total;
        r.images.heat = Some(percep::heatmap(&gold, &d).to_png()?);
        if why.is_empty() {
            r.status = Status::Pass;
            r.message = if d.diff_pixels == 0 && d.forgiven_pixels > 0 { format!("within tolerance ({} anti-aliasing pixel(s) forgiven)", d.forgiven_pixels) } else { "within tolerance".into() };
            r.images = FrameImages::default();
            return Ok(r);
        }
        r.message = why.join("; ");
    }
    if o.bless {
        store.put_img(&c.name, f, &c.renderer, &now)?;
        r.status = Status::Blessed;
        r.message = format!("accepted: {}", r.message);
    }
    Ok(r)
}

pub(super) fn run_validate(c: &ValidateCheck, base: &Path, project: &str) -> CheckResult {
    let t0 = Stamp::now();
    let (e, _) = match load_engine(base, project, None) {
        Ok(x) => x,
        Err(m) => return finish(&c.name, "validate", Status::Fail, m, t0, vec![], Value::Null),
    };
    let level = c.lint.as_deref().unwrap_or("error");
    let findings = crate::lint::lint(&e.project, crate::assets::resolve(base, project).ok().as_deref().and_then(|p| p.parent()));
    let min = match level {
        "error" => Some(crate::lint::Severity::Error),
        "warn" => Some(crate::lint::Severity::Warn),
        "info" => Some(crate::lint::Severity::Info),
        _ => None,
    };
    let bad: Vec<&crate::lint::Finding> = findings.iter().filter(|f| min.map(|m| f.severity >= m).unwrap_or(false)).collect();
    let details = json!({"findings": findings.len(), "failing": bad.len(), "lint": level});
    if bad.is_empty() {
        finish(&c.name, "validate", Status::Pass, format!("project compiles; {} lint finding(s), none at '{level}' or above", findings.len()), t0, vec![], details)
    } else {
        let list: Vec<String> = bad.iter().take(5).map(|f| format!("{} [{}] {}", f.code, f.target, f.message)).collect();
        finish(&c.name, "validate", Status::Fail, format!("{} lint finding(s) at '{level}' or above: {}", bad.len(), list.join("; ")), t0, vec![], details)
    }
}

pub(super) fn run_fuzz(c: &FuzzCheck, base: &Path, project: &str) -> CheckResult {
    let t0 = Stamp::now();
    let path = match crate::assets::resolve(base, project) {
        Ok(p) => p,
        Err(e) => return errored(&c.name, "fuzz", t0, e),
    };
    let text = match crate::assets::read_path(&path, crate::assets::MAX_DATA_BYTES, "project").and_then(|b| String::from_utf8(b).map_err(|_| "project is not UTF-8".to_string())) {
        Ok(t) => t,
        Err(e) => return errored(&c.name, "fuzz", t0, e),
    };
    let p = match Project::from_json(&text) {
        Ok(p) => p,
        Err(e) => return errored(&c.name, "fuzz", t0, format!("invalid project JSON: {e}")),
    };
    let mut fo = crate::fuzz::FuzzOptions { samples: c.samples.unwrap_or(100), seed: c.seed.unwrap_or(1), ..Default::default() };
    fo.frame = c.frame.unwrap_or(0.0);
    let rep = match crate::fuzz::run(&p, path.parent(), &fo) {
        Ok(r) => r,
        Err(e) => return errored(&c.name, "fuzz", t0, e),
    };
    let max = c.max_fail_rate.unwrap_or(0.0);
    let s = &rep.summary;
    let details = json!({"samples": s.samples, "failures": s.failures, "failure_rate": s.failure_rate, "max_fail_rate": max, "by_kind": s.by_kind, "seed": fo.seed});
    // a gate at 100 % (or switched off) cannot fail: report it as information, never as a pass
    if c.gate == Some(false) || max >= 1.0 {
        let why = if c.gate == Some(false) { "non-gating (gate = false)" } else { "max_fail_rate is 100 %, so this cannot fail" };
        return finish(&c.name, "fuzz", Status::Info, format!("{} of {} pose samples fail ({:.2}%); not a gate: {why}", s.failures, s.samples, s.failure_rate * 100.0), t0, vec![], details);
    }
    if s.failure_rate > max {
        finish(&c.name, "fuzz", Status::Fail, format!("{} of {} pose samples fail ({:.2}% > {:.2}%)", s.failures, s.samples, s.failure_rate * 100.0, max * 100.0), t0, vec![], details)
    } else {
        finish(&c.name, "fuzz", Status::Pass, format!("{} of {} pose samples fail ({:.2}% <= {:.2}%)", s.failures, s.samples, s.failure_rate * 100.0, max * 100.0), t0, vec![], details)
    }
}

pub(super) fn run_limits(c: &LimitsCheck, base: &Path, project: &str) -> CheckResult {
    let t0 = Stamp::now();
    let (e, bytes) = match load_engine(base, project, None) {
        Ok(x) => x,
        Err(m) => return errored(&c.name, "limits", t0, m),
    };
    let p = &e.project;
    let mut over = Vec::new();
    let mut chk = |name: &str, have: u64, max: Option<u64>| {
        if let Some(m) = max {
            if have > m {
                over.push(format!("{name} {have} > {m}"));
            }
        }
    };
    chk("layers", e.node_count() as u64, c.max_layers.map(|v| v as u64));
    chk("bones", p.bones.len() as u64, c.max_bones.map(|v| v as u64));
    chk("channels", p.channels.len() as u64, c.max_channels.map(|v| v as u64));
    chk("frames", p.frames as u64, c.max_frames.map(|v| v as u64));
    chk("canvas pixels", (p.width * p.height) as u64, c.max_canvas_pixels);
    chk("file bytes", bytes, c.max_file_bytes);
    let details = json!({"layers": e.node_count(), "bones": p.bones.len(), "channels": p.channels.len(), "frames": p.frames, "canvas_pixels": (p.width * p.height) as u64, "file_bytes": bytes});
    if over.is_empty() {
        finish(&c.name, "limits", Status::Pass, "within all limits".into(), t0, vec![], details)
    } else {
        finish(&c.name, "limits", Status::Fail, over.join("; "), t0, vec![], details)
    }
}

pub(super) fn run_export_size(c: &ExportSizeCheck, base: &Path, project: &str) -> CheckResult {
    let t0 = Stamp::now();
    let (e, _) = match load_engine(base, project, None) {
        Ok(x) => x,
        Err(m) => return errored(&c.name, "export-size", t0, m),
    };
    let bytes: Result<u64, String> = match c.format.as_str() {
        "gif" => crate::export::gif_bytes(&e, c.scale).map(|b| b.len() as u64),
        "apng" => crate::open_formats::apng_bytes(&e, c.scale).map(|b| b.len() as u64),
        "lottie" => Ok(crate::export::lottie_json(&e).to_string().len() as u64),
        "html" => Ok(crate::export::html_player(&e).len() as u64),
        _ => Ok((0..e.project.frames).map(|f| render_svg(&e, f).len() as u64).sum()),
    };
    match bytes {
        Err(m) => errored(&c.name, "export-size", t0, m),
        Ok(b) => {
            let st = if b > c.max_bytes { Status::Fail } else { Status::Pass };
            finish(&c.name, "export-size", st, format!("{} export is {b} bytes (budget {})", c.format, c.max_bytes), t0, vec![], json!({"format": c.format, "bytes": b, "max_bytes": c.max_bytes}))
        }
    }
}

pub(super) fn digest(e: &Engine, renderer: &str, frame: u32, scale: f32) -> Result<String, String> {
    if renderer == "svg" {
        Ok(crate::vcs::sha256_hex(render_svg(e, frame).as_bytes()))
    } else {
        Ok(render_img(e, renderer, frame, scale)?.content_hash())
    }
}

pub(super) fn run_determinism(c: &DeterminismCheck, base: &Path, project: &str) -> CheckResult {
    let t0 = Stamp::now();
    let (e1, _) = match load_engine(base, project, None) {
        Ok(x) => x,
        Err(m) => return errored(&c.name, "determinism", t0, m),
    };
    let (e2, _) = match load_engine(base, project, None) {
        Ok(x) => x,
        Err(m) => return errored(&c.name, "determinism", t0, m),
    };
    let frames = match pick_frames(&c.frames, &c.step, e1.project.frames) {
        Ok(f) => f,
        Err(m) => return errored(&c.name, "determinism", t0, m),
    };
    let rends: Vec<String> = if c.renderers.is_empty() { vec!["raster".into(), "float".into(), "svg".into()] } else { c.renderers.clone() };
    let mut bad = Vec::new();
    let mut n = 0;
    for r in &rends {
        for &f in &frames {
            n += 1;
            // twice on the first engine (same instance reused) and once on the second (fresh load)
            let a = digest(&e1, r, f, c.scale);
            let b = digest(&e1, r, f, c.scale);
            let d = digest(&e2, r, f, c.scale);
            match (a, b, d) {
                (Ok(a), Ok(b), Ok(d)) => {
                    if a != b || a != d {
                        bad.push(format!("{r} frame {f}"));
                    }
                }
                (Err(m), _, _) | (_, Err(m), _) | (_, _, Err(m)) => return errored(&c.name, "determinism", t0, m),
            }
        }
    }
    let details = json!({"renders_compared": n, "renderers": rends, "frames": frames});
    if bad.is_empty() {
        finish(&c.name, "determinism", Status::Pass, format!("{n} render(s) repeated byte-identically"), t0, vec![], details)
    } else {
        finish(&c.name, "determinism", Status::Fail, format!("not reproducible: {}", bad.join(", ")), t0, vec![], details)
    }
}

/// Mean absolute difference of the four 8-bit channels, in percent of full scale.
pub(super) fn mad(a: &Img, b: &Img) -> f64 {
    a.rgba.iter().zip(&b.rgba).map(|(&x, &y)| (x as f64 - y as f64).abs()).sum::<f64>() / a.rgba.len() as f64 / 255.0 * 100.0
}

/// `(seam, mean step, max step)` of the animation as mean absolute differences in percent.
pub fn loop_stats(e: &Engine, scale: f32) -> Result<(f64, f64, f64), String> {
    let n = e.project.frames;
    if n < 2 {
        return Err("a loop check needs at least 2 frames".into());
    }
    if n as usize > 600 {
        return Err(format!("{n} frames: loop checks render every frame, at most 600"));
    }
    let imgs: Result<Vec<Img>, String> = (0..n).map(|f| render_img(e, "raster", f, scale)).collect();
    let imgs = imgs?;
    let steps: Vec<f64> = imgs.windows(2).map(|w| mad(&w[0], &w[1])).collect();
    let seam = mad(&imgs[imgs.len() - 1], &imgs[0]);
    Ok((seam, steps.iter().sum::<f64>() / steps.len() as f64, steps.iter().cloned().fold(0.0, f64::max)))
}

pub(super) fn run_loop(c: &LoopCheck, base: &Path, project: &str) -> CheckResult {
    let t0 = Stamp::now();
    let (e, _) = match load_engine(base, project, None) {
        Ok(x) => x,
        Err(m) => return errored(&c.name, "loop", t0, m),
    };
    let (seam, mean, max) = match loop_stats(&e, c.scale) {
        Ok(x) => x,
        Err(m) => return errored(&c.name, "loop", t0, m),
    };
    let ratio = c.max_seam_ratio.unwrap_or(2.0);
    let allowed = ratio * mean.max(0.01);
    let mut why = Vec::new();
    if seam > allowed {
        why.push(format!("seam {seam:.3}% > {ratio} x mean step {mean:.3}%"));
    }
    if matches!(c.max_seam_mad, Some(m) if seam > m) {
        why.push(format!("seam {seam:.3}% > absolute limit {:.3}%", c.max_seam_mad.unwrap()));
    }
    let details = json!({"seam_mad_pct": seam, "mean_step_mad_pct": mean, "max_step_mad_pct": max, "max_seam_ratio": ratio});
    if why.is_empty() {
        finish(&c.name, "loop", Status::Pass, format!("seam {seam:.3}% vs mean step {mean:.3}%"), t0, vec![], details)
    } else {
        finish(&c.name, "loop", Status::Fail, why.join("; "), t0, vec![], details)
    }
}

pub(super) fn run_perf(c: &PerfCheck, base: &Path, project: &str) -> CheckResult {
    let t0 = Stamp::now();
    let (e, _) = match load_engine(base, project, None) {
        Ok(x) => x,
        Err(m) => return errored(&c.name, "perf", t0, m),
    };
    let frames = match &c.frames {
        None if c.step.is_none() => {
            let n = e.project.frames.max(1);
            let k = n.min(12);
            (0..k).map(|i| i * n / k).collect()
        }
        _ => match pick_frames(&c.frames, &c.step, e.project.frames) {
            Ok(f) => f,
            Err(m) => return errored(&c.name, "perf", t0, m),
        },
    };
    let o = RasterOptions { scale: c.scale, show_bones: false, transparent: false };
    for &f in &frames {
        let _ = render_rgba(&e, f as f32, &o); // warm-up
    }
    let mut ms: Vec<f64> = frames
        .iter()
        .map(|&f| {
            let t = Stamp::now();
            let _ = render_rgba(&e, f as f32, &o);
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = ms[ms.len() / 2];
    let limit = c.budget_ms * (1.0 + c.tolerance.unwrap_or(0.5));
    let over = median > limit;
    let details = json!({"median_ms": median, "max_ms": ms[ms.len() - 1], "budget_ms": c.budget_ms, "limit_ms": limit, "frames_timed": ms.len(), "enforced": c.enforce});
    let msg = format!("median {median:.2} ms/frame, budget {} ms (+{:.0}% tolerance = {limit:.2} ms){}", c.budget_ms, c.tolerance.unwrap_or(0.5) * 100.0, if over && !c.enforce { "; over budget, informational only" } else { "" });
    let st = if over && c.enforce { Status::Fail } else if c.enforce { Status::Pass } else { Status::Info };
    finish(&c.name, "perf", st, msg, t0, vec![], details)
}

/// Run the manifest. `Err` means the manifest itself or the golden index is unusable (exit code 2).
pub fn run(manifest_path: &Path, o: &RunOptions) -> Result<CiReport, String> {
    let t0 = Stamp::now();
    let text = std::fs::read_to_string(manifest_path).map_err(|e| format!("cannot read manifest {}: {e}", manifest_path.display()))?;
    let m = Manifest::parse(&text)?;
    if o.update && o.bless {
        return Err("--update and --bless exclude each other".into());
    }
    for n in &o.only {
        if !m.checks.iter().any(|c| c.name() == n) {
            return Err(format!("--only: no check named '{n}' (have: {})", m.checks.iter().map(|c| c.name()).collect::<Vec<_>>().join(", ")));
        }
    }
    let base = manifest_path.parent().filter(|p| !p.as_os_str().is_empty()).map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let mut store = GoldenStore::open(&base.join(&m.goldens))?;
    let mut results = Vec::new();
    for c in &m.checks {
        if !o.only.is_empty() && !o.only.iter().any(|n| n == c.name()) {
            continue;
        }
        let project = c.project(&m.project).unwrap_or_default().to_string();
        results.push(match c {
            Check::Render(r) => run_render(r, &base, &project, &m.defaults, &mut store, o),
            Check::Validate(v) => run_validate(v, &base, &project),
            Check::Fuzz(f) => run_fuzz(f, &base, &project),
            Check::Limits(l) => run_limits(l, &base, &project),
            Check::ExportSize(x) => run_export_size(x, &base, &project),
            Check::Determinism(d) => run_determinism(d, &base, &project),
            Check::Loop(l) => run_loop(l, &base, &project),
            Check::Perf(p) => run_perf(p, &base, &project),
        });
    }
    store.save()?;
    if o.prune {
        store.prune()?;
    }
    let mode = if o.update { "update" } else if o.bless { "bless" } else { "verify" };
    Ok(CiReport {
        schema: REPORT_SCHEMA.into(),
        manifest: manifest_path.display().to_string(),
        mode: mode.into(),
        duration_ms: t0.elapsed().as_secs_f64() * 1000.0,
        summary: CiReport::summarize(&results),
        checks: results,
    })
}

// ---- init --------------------------------------------------------------------------------------

/// Build a manifest with sensible default checks for `project`, calibrated on the project as it is now
/// (limits get 50 % headroom, the fuzz and perf budgets are set from a measurement, the loop check is
/// added only when the project currently loops cleanly).
pub fn init_manifest(project_path: &Path, manifest_path: &Path) -> Result<Manifest, String> {
    let base = manifest_path.parent().filter(|p| !p.as_os_str().is_empty()).map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let rel = match (std::fs::canonicalize(project_path), std::fs::canonicalize(&base)) {
        (Ok(p), Ok(b)) => p.strip_prefix(&b).map(|r| r.to_path_buf()).unwrap_or(p),
        _ => project_path.to_path_buf(),
    };
    let rel_s = rel.to_string_lossy().replace('\\', "/");
    let (e, bytes) = load_engine_at(project_path, None)?;
    let p = &e.project;
    let frames = p.frames.max(1);
    let k = frames.min(6);
    let mut pick: Vec<u32> = (0..k).map(|i| if k == 1 { 0 } else { i * (frames - 1) / (k - 1) }).collect();
    pick.dedup();
    let maxdim = p.width.max(p.height).max(1.0);
    let scale = (480.0 / maxdim).min(1.0);
    let scale = (scale * 1000.0).round() / 1000.0;
    let mut checks = vec![Check::Validate(ValidateCheck { name: "validate".into(), project: None, lint: None })];
    checks.push(Check::Render(RenderCheck { name: "frames".into(), project: None, frames: Some(pick.clone()), step: None, scale, size: None, renderer: "raster".into(), thresholds: Tol::default(), ignore: vec![] }));
    let mid = pick.get(pick.len() / 2).copied().unwrap_or(0);
    checks.push(Check::Render(RenderCheck { name: "frames-float".into(), project: None, frames: Some(vec![0, mid]), step: None, scale, size: None, renderer: "float".into(), thresholds: Tol { max_de: Some(10.0), min_ssim: Some(0.98), ..Tol::default() }, ignore: vec![] }));
    checks.push(Check::Determinism(DeterminismCheck { name: "determinism".into(), project: None, frames: Some(vec![0, frames - 1]), step: None, scale: 0.5, renderers: vec![] }));
    // fuzz: measure the current failure rate and allow a small margin
    let fo = crate::fuzz::FuzzOptions { samples: 100, seed: 1, ..Default::default() };
    if let Ok(rep) = crate::fuzz::run(&e.project, Path::new(project_path).parent(), &fo) {
        let rate = rep.summary.failure_rate;
        let max = ((rate + 0.02) * 100.0).ceil() / 100.0;
        // when most poses already fail, no threshold can catch a regression: write an information-only check
        let gate = if rate > 0.5 || max >= 1.0 { Some(false) } else { None };
        checks.push(Check::Fuzz(FuzzCheck { name: "pose-fuzz".into(), project: None, samples: Some(100), seed: Some(1), frame: None, max_fail_rate: Some(max.min(1.0)), gate }));
    }
    let hr = |v: usize| ((v as f64 * 1.5).ceil() as usize).max(v + 1);
    checks.push(Check::Limits(LimitsCheck {
        name: "limits".into(),
        project: None,
        max_layers: Some(hr(e.node_count())),
        max_bones: Some(hr(p.bones.len())),
        max_channels: Some(hr(p.channels.len())),
        max_frames: Some(hr(p.frames as usize) as u32),
        max_canvas_pixels: None,
        max_file_bytes: Some(hr(bytes as usize) as u64),
    }));
    if let Ok(b) = crate::export::gif_bytes(&e, 0.5) {
        checks.push(Check::ExportSize(ExportSizeCheck { name: "gif-size".into(), project: None, format: "gif".into(), max_bytes: (b.len() as f64 * 1.5).ceil() as u64, scale: 0.5 }));
    }
    if (2..=600).contains(&frames) {
        if let Ok((seam, mean, _)) = loop_stats(&e, 0.5) {
            if seam <= 2.0 * mean.max(0.01) {
                checks.push(Check::Loop(LoopCheck { name: "loop-seam".into(), project: None, scale: 0.5, max_seam_ratio: None, max_seam_mad: None }));
            }
        }
    }
    // perf: median of up to 6 frames, budget 4x with a minimum of 5 ms
    let o = RasterOptions { scale, show_bones: false, transparent: false };
    let mut ms: Vec<f64> = pick
        .iter()
        .map(|&f| {
            let _ = render_rgba(&e, f as f32, &o);
            let t = Stamp::now();
            let _ = render_rgba(&e, f as f32, &o);
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let budget = (ms[ms.len() / 2] * 4.0).ceil().max(5.0);
    checks.push(Check::Perf(PerfCheck { name: "render-speed".into(), project: None, scale, frames: Some(pick), step: None, budget_ms: budget, tolerance: Some(0.5), enforce: false }));
    let m = Manifest { schema: MANIFEST_SCHEMA.into(), project: Some(rel_s), goldens: "goldens".into(), defaults: Tol::default(), checks };
    m.validate()?;
    Ok(m)
}

// ---- reports -----------------------------------------------------------------------------------

pub(super) fn xml_esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&apos;"),
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => o.push(c),
        }
    }
    o
}

/// JUnit XML: one `testcase` per check; failing frames are listed in the failure text.
pub fn junit_xml(r: &CiReport) -> String {
    let failures = r.checks.iter().filter(|c| c.status == Status::Fail).count();
    let errors = r.checks.iter().filter(|c| c.status == Status::Error).count();
    let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(&format!(
        "<testsuites name=\"osatura-ci\" tests=\"{}\" failures=\"{failures}\" errors=\"{errors}\" time=\"{:.3}\">\n",
        r.checks.len(),
        r.duration_ms / 1000.0
    ));
    s.push_str(&format!(
        "  <testsuite name=\"osatura-ci\" tests=\"{}\" failures=\"{failures}\" errors=\"{errors}\" skipped=\"0\" time=\"{:.3}\">\n",
        r.checks.len(),
        r.duration_ms / 1000.0
    ));
    for c in &r.checks {
        s.push_str(&format!("    <testcase classname=\"osatura.{}\" name=\"{}\" time=\"{:.3}\"", xml_esc(&c.kind), xml_esc(&c.name), c.duration_ms / 1000.0));
        let mut body = String::new();
        for f in c.frames.iter().filter(|f| matches!(f.status, Status::Fail | Status::Error)) {
            body.push_str(&format!("frame {}: {}\n", f.frame, f.message));
            for g in f.regions.iter().take(5) {
                body.push_str(&format!("  region x={} y={} w={} h={} pixels={} max_de={:.2}\n", g.x, g.y, g.w, g.h, g.pixels, g.max_de));
            }
        }
        match c.status {
            Status::Fail => s.push_str(&format!(">\n      <failure message=\"{}\" type=\"{}\">{}</failure>\n    </testcase>\n", xml_esc(&c.message), xml_esc(&c.kind), xml_esc(&body))),
            Status::Error => s.push_str(&format!(">\n      <error message=\"{}\" type=\"{}\">{}</error>\n    </testcase>\n", xml_esc(&c.message), xml_esc(&c.kind), xml_esc(&body))),
            _ => s.push_str(&format!(">\n      <system-out>{}</system-out>\n    </testcase>\n", xml_esc(&format!("{}: {}", c.status.name(), c.message)))),
        }
    }
    s.push_str("  </testsuite>\n</testsuites>\n");
    s
}

pub(super) fn h(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub(super) fn data_uri(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", crate::base64::encode(png))
}

/// One self-contained HTML page: summary, check table, and for every failing frame the golden,
/// the current render and the difference map as inlined PNG data URIs. No external resources.
pub fn html_report(r: &CiReport) -> String {
    let mut s = String::new();
    s.push_str("<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Osatura CI report</title>\n<style>\n");
    s.push_str(":root{--bg:#fff;--fg:#1c1f24;--mut:#5b6573;--line:#d9dee5;--card:#f6f8fa;--pass:#1a7f37;--fail:#cf222e;--err:#9a6700;--info:#0969da;--acc:#8250df}\n@media(prefers-color-scheme:dark){:root{--bg:#0d1117;--fg:#e6edf3;--mut:#9198a1;--line:#30363d;--card:#161b22;--pass:#3fb950;--fail:#ff7b72;--err:#d29922;--info:#58a6ff;--acc:#bc8cff}}\n");
    s.push_str("body{margin:0;padding:16px;background:var(--bg);color:var(--fg);font:14px/1.45 system-ui,-apple-system,Segoe UI,Roboto,sans-serif}main{max-width:1100px;margin:0 auto}h1{font-size:1.4rem;margin:.2em 0}h2{font-size:1.05rem;margin:1.4em 0 .5em}\n");
    s.push_str(".sum{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0}.sum div{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:8px 14px;min-width:84px}.sum b{display:block;font-size:1.4rem}\n");
    s.push_str("table{border-collapse:collapse;width:100%}th,td{text-align:left;padding:6px 8px;border-bottom:1px solid var(--line);vertical-align:top}th{color:var(--mut);font-weight:600}th.num,td.num{text-align:right;font-variant-numeric:tabular-nums}\n");
    s.push_str(".b{display:inline-block;padding:1px 8px;border-radius:99px;font-size:.8rem;font-weight:600;border:1px solid currentColor}.pass{color:var(--pass)}.fail{color:var(--fail)}.error{color:var(--err)}.info,.updated,.blessed{color:var(--info)}\n");
    s.push_str("details{background:var(--card);border:1px solid var(--line);border-radius:8px;margin:8px 0;padding:6px 12px}summary{cursor:pointer;font-weight:600;padding:4px 0}.mut{color:var(--mut)}code{font:12px ui-monospace,Menlo,monospace}\n");
    s.push_str(".imgs{display:flex;flex-wrap:wrap;gap:12px;margin:8px 0}figure{margin:0}figcaption{font-size:.8rem;color:var(--mut);margin-bottom:2px}.imgs img{max-width:100%;height:auto;border:1px solid var(--line);image-rendering:pixelated;background:repeating-conic-gradient(#ccc 0 25%,#fff 0 50%) 0 0/12px 12px}\n");
    s.push_str("button{font:inherit;border:1px solid var(--line);background:var(--bg);color:var(--fg);border-radius:6px;padding:2px 10px;cursor:pointer}\n</style></head><body><main>\n");
    s.push_str("<h1>Osatura CI report</h1>\n");
    s.push_str(&format!("<p class=\"mut\">manifest <code>{}</code> &middot; mode {} &middot; {:.2} s &middot; exit code {}</p>\n", h(&r.manifest), h(&r.mode), r.duration_ms / 1000.0, r.exit_code()));
    let sm = &r.summary;
    s.push_str(&format!(
        "<div class=\"sum\"><div><b>{}</b>checks</div><div class=\"pass\"><b>{}</b>passed</div><div class=\"fail\"><b>{}</b>failed</div><div class=\"error\"><b>{}</b>errors</div><div class=\"info\"><b>{}</b>info / updated / blessed</div></div>\n",
        sm.total,
        sm.passed,
        sm.failed,
        sm.errors,
        sm.info + sm.updated + sm.blessed
    ));
    s.push_str("<table><thead><tr><th>Check</th><th>Kind</th><th>Status</th><th>Result</th><th class=\"num\">ms</th></tr></thead><tbody>\n");
    for c in &r.checks {
        s.push_str(&format!(
            "<tr><td><a href=\"#c-{0}\">{0}</a></td><td>{1}</td><td><span class=\"b {2}\">{2}</span></td><td>{3}</td><td class=\"num\">{4:.0}</td></tr>\n",
            h(&c.name),
            h(&c.kind),
            c.status.name(),
            h(&c.message),
            c.duration_ms
        ));
    }
    s.push_str("</tbody></table>\n<h2>Details</h2>\n");
    let mut img_count = 0;
    for c in &r.checks {
        let open = if matches!(c.status, Status::Fail | Status::Error) { " open" } else { "" };
        s.push_str(&format!("<details id=\"c-{}\"{open}><summary><span class=\"b {}\">{}</span> {} <span class=\"mut\">({})</span></summary>\n<p>{}</p>\n", h(&c.name), c.status.name(), c.status.name(), h(&c.name), h(&c.kind), h(&c.message)));
        if !c.frames.is_empty() {
            s.push_str("<table><thead><tr><th>Frame</th><th>Status</th><th class=\"num\">SSIM</th><th class=\"num\">max dE</th><th class=\"num\">diff px</th><th class=\"num\">forgiven</th><th>Note</th></tr></thead><tbody>\n");
            for f in &c.frames {
                match &f.metrics {
                    Some(m) => s.push_str(&format!(
                        "<tr><td>{}</td><td><span class=\"b {}\">{}</span></td><td class=\"num\">{:.4}</td><td class=\"num\">{:.2}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td>{}</td></tr>\n",
                        f.frame, f.status.name(), f.status.name(), m.ssim, m.max_de, m.diff_pixels, m.forgiven_pixels, h(&f.message)
                    )),
                    None => s.push_str(&format!("<tr><td>{}</td><td><span class=\"b {}\">{}</span></td><td class=\"num\">{}</td><td class=\"num\"></td><td class=\"num\"></td><td class=\"num\"></td><td>{}</td></tr>\n", f.frame, f.status.name(), f.status.name(), if f.exact { "1" } else { "" }, h(&f.message))),
                }
            }
            s.push_str("</tbody></table>\n");
            for f in c.frames.iter().filter(|f| matches!(f.status, Status::Fail | Status::Blessed) && f.images.actual.is_some()) {
                if img_count >= 60 {
                    s.push_str("<p class=\"mut\">further images omitted (60 per report)</p>\n");
                    break;
                }
                img_count += 1;
                s.push_str(&format!("<details{open}><summary>frame {} &mdash; {}</summary>\n<div class=\"imgs\">", f.frame, h(&f.message)));
                let fig = |cap: &str, b: &Option<Vec<u8>>, id: &str| -> String {
                    b.as_ref().map(|b| format!("<figure><figcaption>{cap}</figcaption><img id=\"{id}\" alt=\"{cap}\" src=\"{}\"></figure>", data_uri(b))).unwrap_or_default()
                };
                let id = format!("fl-{}-{}", h(&c.name), f.frame);
                s.push_str(&fig("golden", &f.images.golden, &format!("{id}-g")));
                s.push_str(&fig("current", &f.images.actual, &format!("{id}-a")));
                s.push_str(&fig("difference", &f.images.heat, &format!("{id}-d")));
                s.push_str("</div>");
                if f.images.golden.is_some() {
                    s.push_str(&format!("<p><button type=\"button\" onclick=\"var g=document.getElementById('{id}-g'),a=document.getElementById('{id}-a'),t=g.src;g.src=a.src;a.src=t\">swap golden / current</button> <span class=\"mut\">click to flicker between the two</span></p>"));
                }
                if !f.regions.is_empty() {
                    s.push_str("<table><thead><tr><th>Region</th><th class=\"num\">x</th><th class=\"num\">y</th><th class=\"num\">w</th><th class=\"num\">h</th><th class=\"num\">pixels</th><th class=\"num\">max dE</th></tr></thead><tbody>");
                    for (i, g) in f.regions.iter().enumerate().take(10) {
                        s.push_str(&format!("<tr><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{:.2}</td></tr>", i + 1, g.x, g.y, g.w, g.h, g.pixels, g.max_de));
                    }
                    s.push_str("</tbody></table>");
                    if f.regions_total > 10 {
                        s.push_str(&format!("<p class=\"mut\">{} regions in total, 10 shown</p>", f.regions_total));
                    }
                }
                s.push_str("</details>\n");
            }
        }
        if !c.details.is_null() {
            s.push_str(&format!("<p class=\"mut\"><code>{}</code></p>\n", h(&c.details.to_string())));
        }
        s.push_str("</details>\n");
    }
    s.push_str("</main></body></html>\n");
    s
}

// ---- tests -------------------------------------------------------------------------------------
