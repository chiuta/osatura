// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `ci` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

pub const MANIFEST_SCHEMA: &str = "osatura-ci/1";

pub const REPORT_SCHEMA: &str = "osatura-ci-report/1";

pub const DEFAULT_MANIFEST: &str = "osatura-ci.json";

pub(super) const MAX_FRAMES_PER_CHECK: usize = 2000;

// ---- manifest ----------------------------------------------------------------------------------

/// Optional thresholds; unset fields fall back to the manifest `defaults`, then to built-ins.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tol {
    /// `de2000` (default) or `de76`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<String>,
    /// A pixel differs when its colour difference exceeds this (default 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_de: Option<f64>,
    /// Largest colour difference allowed among counted pixels (default 8.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_de: Option<f64>,
    /// Smallest allowed mean SSIM (default 0.99).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_ssim: Option<f64>,
    /// Largest allowed fraction of counted pixels (default 0.001).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_diff_fraction: Option<f64>,
    /// Anti-aliasing tolerance: differing groups of at most this many edge pixels are forgiven (default 2, 0 = off).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aa_isolated: Option<usize>,
    /// Anti-aliasing tolerance: edge pixels with a difference up to this are forgiven (default 0 = off).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aa_edge_de: Option<f64>,
    /// Largest allowed pHash Hamming distance (default: not checked).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_phash_distance: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct Resolved {
    pub metric: DeMetric,
    pub pixel_de: f64,
    pub max_de: f64,
    pub min_ssim: f64,
    pub max_diff_fraction: f64,
    pub aa_isolated: usize,
    pub aa_edge_de: f64,
    pub max_phash_distance: Option<u32>,
}

impl Tol {
    pub(super) fn validate(&self, what: &str) -> Result<(), String> {
        if let Some(m) = &self.metric {
            DeMetric::parse(m).map_err(|e| format!("{what}: {e}"))?;
        }
        let nonneg = |v: Option<f64>, n: &str| -> Result<(), String> {
            match v {
                Some(x) if !(x.is_finite() && x >= 0.0) => Err(format!("{what}: {n} must be a number >= 0")),
                _ => Ok(()),
            }
        };
        nonneg(self.pixel_de, "pixel_de")?;
        nonneg(self.max_de, "max_de")?;
        nonneg(self.aa_edge_de, "aa_edge_de")?;
        if let Some(s) = self.min_ssim {
            if !(s.is_finite() && (-1.0..=1.0).contains(&s)) {
                return Err(format!("{what}: min_ssim must be in -1..1"));
            }
        }
        if let Some(f) = self.max_diff_fraction {
            if !(f.is_finite() && (0.0..=1.0).contains(&f)) {
                return Err(format!("{what}: max_diff_fraction must be in 0..1"));
            }
        }
        if matches!(self.max_phash_distance, Some(d) if d > 64) {
            return Err(format!("{what}: max_phash_distance must be 0..64"));
        }
        if matches!(self.aa_isolated, Some(n) if n > 64) {
            return Err(format!("{what}: aa_isolated must be 0..64"));
        }
        Ok(())
    }

    pub fn resolve(&self, base: &Tol) -> Resolved {
        let metric = self.metric.as_deref().or(base.metric.as_deref()).and_then(|m| DeMetric::parse(m).ok()).unwrap_or(DeMetric::De2000);
        Resolved {
            metric,
            pixel_de: self.pixel_de.or(base.pixel_de).unwrap_or(1.0),
            max_de: self.max_de.or(base.max_de).unwrap_or(8.0),
            min_ssim: self.min_ssim.or(base.min_ssim).unwrap_or(0.99),
            max_diff_fraction: self.max_diff_fraction.or(base.max_diff_fraction).unwrap_or(0.001),
            aa_isolated: self.aa_isolated.or(base.aa_isolated).unwrap_or(2),
            aa_edge_de: self.aa_edge_de.or(base.aa_edge_de).unwrap_or(0.0),
            max_phash_distance: self.max_phash_distance.or(base.max_phash_distance),
        }
    }
}

pub(super) fn one() -> f32 {
    1.0
}

pub(super) fn half() -> f32 {
    0.5
}

pub(super) fn raster_name() -> String {
    "raster".into()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderCheck {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Explicit frame numbers. Without `frames` and `step`: first, middle and last frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<Vec<u32>>,
    /// Every n-th frame starting at 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,
    /// Output pixels per stage unit (a small value is a proxy render).
    #[serde(default = "one")]
    pub scale: f32,
    /// Canvas size `WxH` (layout containers re-solve), applied before `scale`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// `raster`, `float` or `svg`.
    #[serde(default = "raster_name")]
    pub renderer: String,
    #[serde(default)]
    pub thresholds: Tol,
    /// Rectangles `[x, y, w, h]` in stage units that are not compared.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<[f32; 4]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidateCheck {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Lowest linter severity that fails: `error` (default), `warn`, `info` or `off`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FuzzCheck {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub samples: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<f32>,
    /// Largest allowed failure rate, 0..1 (default 0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_fail_rate: Option<f32>,
    /// `false` = report the failure rate as information and never fail (`init` writes this when the
    /// project already fails most poses, where a threshold could not catch a regression). Default true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LimitsCheck {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_layers: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_bones: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_channels: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_frames: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_canvas_pixels: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_bytes: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportSizeCheck {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// `gif`, `apng`, `lottie`, `html` or `svg-frames` (sum of all frames as SVG text).
    pub format: String,
    pub max_bytes: u64,
    #[serde(default = "half")]
    pub scale: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeterminismCheck {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,
    #[serde(default = "half")]
    pub scale: f32,
    /// Default: raster, float and svg.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub renderers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopCheck {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default = "half")]
    pub scale: f32,
    /// The seam (last -> first) may be at most this many times the mean frame-to-frame change (default 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_seam_ratio: Option<f64>,
    /// Optional absolute ceiling for the seam, mean absolute channel difference in percent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_seam_mad: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerfCheck {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default = "one")]
    pub scale: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,
    pub budget_ms: f64,
    /// Relative slack above the budget before it counts as over (default 0.5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<f64>,
    /// Fail when over budget (default false: informational).
    #[serde(default)]
    pub enforce: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Check {
    Render(RenderCheck),
    Validate(ValidateCheck),
    Fuzz(FuzzCheck),
    Limits(LimitsCheck),
    ExportSize(ExportSizeCheck),
    Determinism(DeterminismCheck),
    Loop(LoopCheck),
    Perf(PerfCheck),
}

impl Check {
    pub fn name(&self) -> &str {
        match self {
            Check::Render(c) => &c.name,
            Check::Validate(c) => &c.name,
            Check::Fuzz(c) => &c.name,
            Check::Limits(c) => &c.name,
            Check::ExportSize(c) => &c.name,
            Check::Determinism(c) => &c.name,
            Check::Loop(c) => &c.name,
            Check::Perf(c) => &c.name,
        }
    }
    pub fn kind(&self) -> &'static str {
        match self {
            Check::Render(_) => "render",
            Check::Validate(_) => "validate",
            Check::Fuzz(_) => "fuzz",
            Check::Limits(_) => "limits",
            Check::ExportSize(_) => "export-size",
            Check::Determinism(_) => "determinism",
            Check::Loop(_) => "loop",
            Check::Perf(_) => "perf",
        }
    }
    pub(super) fn project<'a>(&'a self, default: &'a Option<String>) -> Option<&'a str> {
        let own = match self {
            Check::Render(c) => &c.project,
            Check::Validate(c) => &c.project,
            Check::Fuzz(c) => &c.project,
            Check::Limits(c) => &c.project,
            Check::ExportSize(c) => &c.project,
            Check::Determinism(c) => &c.project,
            Check::Loop(c) => &c.project,
            Check::Perf(c) => &c.project,
        };
        own.as_deref().or(default.as_deref())
    }
}

pub(super) fn d_goldens() -> String {
    "goldens".into()
}

pub(super) fn d_schema() -> String {
    MANIFEST_SCHEMA.into()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(default = "d_schema")]
    pub schema: String,
    /// Default project (path relative to the manifest) for checks without their own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Golden directory relative to the manifest (default `goldens`).
    #[serde(default = "d_goldens")]
    pub goldens: String,
    /// Thresholds applied to every render check unless the check overrides them.
    #[serde(default)]
    pub defaults: Tol,
    pub checks: Vec<Check>,
}

pub(super) fn safe_name(n: &str) -> bool {
    !n.is_empty() && n.len() <= 64 && n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) && !n.starts_with('.')
}

pub(super) fn validate_scale(s: f32, what: &str) -> Result<(), String> {
    if !(s.is_finite() && s > 0.0 && s <= 16.0) {
        return Err(format!("{what}: scale must be in (0, 16]"));
    }
    Ok(())
}

pub(super) fn validate_frames(frames: &Option<Vec<u32>>, step: &Option<u32>, what: &str) -> Result<(), String> {
    if frames.is_some() && step.is_some() {
        return Err(format!("{what}: give either 'frames' or 'step', not both"));
    }
    if matches!(frames, Some(f) if f.is_empty()) {
        return Err(format!("{what}: 'frames' is empty"));
    }
    if matches!(step, Some(0)) {
        return Err(format!("{what}: 'step' must be at least 1"));
    }
    Ok(())
}

/// Paths in a manifest are relative to the manifest's directory: not empty, not absolute, no `..`, no NUL.
pub fn manifest_path_ok(what: &str, p: &str) -> Result<(), String> {
    if p.is_empty() || p.contains('\0') || p.starts_with('/') || p.starts_with('\\') || Path::new(p).is_absolute() || p.as_bytes().get(1) == Some(&b':') || p.split(['/', '\\']).any(|c| c == "..") {
        return Err(format!("{what} must be a relative path inside the manifest directory (no '..', no absolute path), got '{p}'"));
    }
    Ok(())
}

impl Manifest {
    /// Parse and validate. All problems found in the manifest itself are reported together.
    pub fn parse(text: &str) -> Result<Manifest, String> {
        let m: Manifest = serde_json::from_str(text).map_err(|e| format!("invalid manifest: {e}"))?;
        m.validate()?;
        Ok(m)
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut errs: Vec<String> = Vec::new();
        if self.schema != MANIFEST_SCHEMA {
            errs.push(format!("schema must be '{MANIFEST_SCHEMA}', got '{}'", self.schema));
        }
        if let Err(e) = manifest_path_ok("goldens", &self.goldens) {
            errs.push(e);
        }
        if let Some(p) = &self.project {
            if let Err(e) = manifest_path_ok("project", p) {
                errs.push(e);
            }
        }
        if let Err(e) = self.defaults.validate("defaults") {
            errs.push(e);
        }
        if self.checks.is_empty() {
            errs.push("checks is empty".into());
        }
        let mut seen = std::collections::HashSet::new();
        for c in &self.checks {
            let n = c.name();
            let what = format!("check '{n}'");
            if !safe_name(n) {
                errs.push(format!("check name '{n}' must be 1-64 characters of letters, digits, '-', '_' or '.' and not start with '.'"));
                continue;
            }
            if !seen.insert(n.to_string()) {
                errs.push(format!("duplicate check name '{n}'"));
            }
            match c.project(&self.project) {
                None => errs.push(format!("{what}: no project (set 'project' on the check or in the manifest)")),
                Some(p) => {
                    if let Err(e) = manifest_path_ok(&format!("{what}: project"), p) {
                        errs.push(e);
                    }
                }
            }
            let r: Result<(), String> = (|| {
                match c {
                    Check::Render(r) => {
                        validate_scale(r.scale, &what)?;
                        validate_frames(&r.frames, &r.step, &what)?;
                        r.thresholds.validate(&what)?;
                        if !["raster", "float", "svg"].contains(&r.renderer.as_str()) {
                            return Err(format!("{what}: unknown renderer '{}' (raster, float, svg)", r.renderer));
                        }
                        if let Some(s) = &r.size {
                            crate::containers::parse_size(s).map_err(|e| format!("{what}: {e}"))?;
                        }
                        for g in &r.ignore {
                            if !g.iter().all(|v| v.is_finite()) || g[2] <= 0.0 || g[3] <= 0.0 {
                                return Err(format!("{what}: ignore rectangles are [x, y, w, h] with w, h > 0"));
                            }
                        }
                        if r.renderer == "svg" && !r.ignore.is_empty() {
                            return Err(format!("{what}: ignore regions need a pixel renderer (raster or float)"));
                        }
                    }
                    Check::Validate(v) => {
                        if let Some(l) = &v.lint {
                            if !["error", "warn", "info", "off"].contains(&l.as_str()) {
                                return Err(format!("{what}: lint must be error, warn, info or off"));
                            }
                        }
                    }
                    Check::Fuzz(f) => {
                        if matches!(f.samples, Some(0)) || matches!(f.samples, Some(n) if n > 100_000) {
                            return Err(format!("{what}: samples must be 1..100000"));
                        }
                        if matches!(f.max_fail_rate, Some(r) if !(0.0..=1.0).contains(&r)) {
                            return Err(format!("{what}: max_fail_rate must be in 0..1"));
                        }
                        if matches!(f.frame, Some(x) if !x.is_finite() || x < 0.0) {
                            return Err(format!("{what}: frame must be >= 0"));
                        }
                    }
                    Check::Limits(l) => {
                        if l.max_layers.is_none() && l.max_bones.is_none() && l.max_channels.is_none() && l.max_frames.is_none() && l.max_canvas_pixels.is_none() && l.max_file_bytes.is_none() {
                            return Err(format!("{what}: set at least one limit"));
                        }
                    }
                    Check::ExportSize(e) => {
                        validate_scale(e.scale, &what)?;
                        if !["gif", "apng", "lottie", "html", "svg-frames"].contains(&e.format.as_str()) {
                            return Err(format!("{what}: unknown format '{}' (gif, apng, lottie, html, svg-frames)", e.format));
                        }
                        if e.max_bytes == 0 {
                            return Err(format!("{what}: max_bytes must be at least 1"));
                        }
                    }
                    Check::Determinism(d) => {
                        validate_scale(d.scale, &what)?;
                        validate_frames(&d.frames, &d.step, &what)?;
                        for r in &d.renderers {
                            if !["raster", "float", "svg"].contains(&r.as_str()) {
                                return Err(format!("{what}: unknown renderer '{r}' (raster, float, svg)"));
                            }
                        }
                    }
                    Check::Loop(l) => {
                        validate_scale(l.scale, &what)?;
                        if matches!(l.max_seam_ratio, Some(r) if !(r.is_finite() && r >= 0.0)) || matches!(l.max_seam_mad, Some(r) if !(r.is_finite() && r >= 0.0)) {
                            return Err(format!("{what}: seam limits must be numbers >= 0"));
                        }
                    }
                    Check::Perf(p) => {
                        validate_scale(p.scale, &what)?;
                        validate_frames(&p.frames, &p.step, &what)?;
                        if !(p.budget_ms.is_finite() && p.budget_ms > 0.0) {
                            return Err(format!("{what}: budget_ms must be a number > 0"));
                        }
                        if matches!(p.tolerance, Some(t) if !(t.is_finite() && t >= 0.0)) {
                            return Err(format!("{what}: tolerance must be a number >= 0"));
                        }
                    }
                }
                Ok(())
            })();
            if let Err(e) = r {
                errs.push(e);
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(format!("invalid manifest:\n  - {}", errs.join("\n  - ")))
        }
    }
}
