// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `ci` (see the module docs in `mod.rs`).
use super::*;

// ---- results -----------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Fail,
    Error,
    /// Informational result (never fails the run).
    Info,
    /// A golden was written by `--update`.
    Updated,
    /// A failing frame was accepted by `bless`.
    Blessed,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Error => "error",
            Status::Info => "info",
            Status::Updated => "updated",
            Status::Blessed => "blessed",
        }
    }
    pub(super) fn rank(self) -> u8 {
        match self {
            Status::Error => 5,
            Status::Fail => 4,
            Status::Blessed => 3,
            Status::Updated => 2,
            Status::Info => 1,
            Status::Pass => 0,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Metrics {
    pub ssim: f64,
    pub ssim_rgb: f64,
    pub mse: f64,
    pub psnr: f64,
    pub max_de: f64,
    pub max_de_raw: f64,
    pub mean_de: f64,
    pub diff_pixels: usize,
    pub forgiven_pixels: usize,
    pub ignored_pixels: usize,
    pub diff_fraction: f64,
    pub phash_distance: u32,
}

/// PNG bytes kept in memory for the report and the artifacts directory.
#[derive(Clone, Debug, Default)]
pub struct FrameImages {
    pub golden: Option<Vec<u8>>,
    pub actual: Option<Vec<u8>>,
    pub heat: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FrameResult {
    pub frame: u32,
    pub status: Status,
    pub message: String,
    /// `true` when the frame matched the golden byte for byte.
    pub exact: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub golden: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Metrics>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<Region>,
    #[serde(skip_serializing_if = "is_zero")]
    pub regions_total: usize,
    #[serde(skip)]
    pub images: FrameImages,
}

pub(super) fn is_zero(n: &usize) -> bool {
    *n == 0
}

#[derive(Clone, Debug, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub kind: String,
    pub status: Status,
    pub message: String,
    pub duration_ms: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<FrameResult>,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub details: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct Summary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub info: usize,
    pub updated: usize,
    pub blessed: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct CiReport {
    pub schema: String,
    pub manifest: String,
    pub mode: String,
    pub duration_ms: f64,
    pub summary: Summary,
    pub checks: Vec<CheckResult>,
}

impl CiReport {
    /// 0 all good, 1 at least one check failed, 2 at least one check could not be evaluated.
    pub fn exit_code(&self) -> i32 {
        if self.summary.errors > 0 {
            2
        } else if self.summary.failed > 0 {
            1
        } else {
            0
        }
    }

    pub(super) fn summarize(checks: &[CheckResult]) -> Summary {
        let n = |s: Status| checks.iter().filter(|c| c.status == s).count();
        Summary { total: checks.len(), passed: n(Status::Pass), failed: n(Status::Fail), errors: n(Status::Error), info: n(Status::Info), updated: n(Status::Updated), blessed: n(Status::Blessed) }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

// ---- golden store ------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoldenRef {
    pub hash: String,
    pub renderer: String,
    #[serde(default)]
    pub w: usize,
    #[serde(default)]
    pub h: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GoldenIndex {
    #[serde(default)]
    pub schema: String,
    /// check name -> zero-padded frame -> golden
    #[serde(default)]
    pub entries: BTreeMap<String, BTreeMap<String, GoldenRef>>,
}

pub(super) const INDEX_SCHEMA: &str = "osatura-goldens/1";

pub struct GoldenStore {
    pub(super) dir: PathBuf,
    pub index: GoldenIndex,
    pub(super) dirty: bool,
}

pub(super) fn fkey(frame: u32) -> String {
    format!("{frame:05}")
}

pub(super) fn golden_name_ok(hash: &str) -> Result<(), String> {
    if hash.len() < 24 || hash.len() > 64 || !hash.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
        return Err(format!("invalid golden hash {hash:?} (need 24-64 lowercase hex digits)"));
    }
    Ok(())
}

impl GoldenStore {
    pub fn open(dir: &Path) -> Result<GoldenStore, String> {
        let p = dir.join("index.json");
        let index = if p.exists() {
            let t = std::fs::read_to_string(&p).map_err(|e| format!("cannot read {}: {e}", p.display()))?;
            let i: GoldenIndex = serde_json::from_str(&t).map_err(|e| format!("invalid golden index {}: {e}", p.display()))?;
            if i.schema != INDEX_SCHEMA {
                return Err(format!("golden index {} has schema '{}', expected '{INDEX_SCHEMA}'", p.display(), i.schema));
            }
            for (check, frames) in &i.entries {
                for (frame, r) in frames {
                    golden_name_ok(&r.hash).map_err(|e| format!("golden index {}: check '{check}' frame {frame}: {e}", p.display()))?;
                }
            }
            i
        } else {
            GoldenIndex { schema: INDEX_SCHEMA.into(), entries: BTreeMap::new() }
        };
        Ok(GoldenStore { dir: dir.to_path_buf(), index, dirty: false })
    }

    pub fn get(&self, check: &str, frame: u32) -> Option<&GoldenRef> {
        self.index.entries.get(check).and_then(|m| m.get(&fkey(frame)))
    }

    /// The file of a golden. The hash comes from a file in the repository (the index), so it is checked:
    /// 24 to 64 lowercase hex digits, nothing that could name another path (`../x`, `/etc/passwd`).
    pub(super) fn file(&self, r: &GoldenRef) -> Result<PathBuf, String> {
        golden_name_ok(&r.hash)?;
        Ok(self.dir.join(format!("{}.{}", r.hash, if r.renderer == "svg" { "svg" } else { "png" })))
    }

    /// Load a golden image, checking that the file still has the pixels its name promises.
    pub fn load_img(&self, r: &GoldenRef) -> Result<Img, String> {
        let p = self.file(r)?;
        let bytes = crate::assets::read_path(&p, crate::assets::MAX_IMAGE_BYTES, "golden")?;
        let img = Img::from_png(&bytes).map_err(|e| format!("golden {}: {e}", p.display()))?;
        if !img.content_hash().starts_with(&r.hash) {
            return Err(format!("golden {} was modified: its pixels do not match its name", p.display()));
        }
        Ok(img)
    }

    pub fn load_text(&self, r: &GoldenRef) -> Result<String, String> {
        let p = self.file(r)?;
        let t = String::from_utf8(crate::assets::read_path(&p, crate::assets::MAX_DATA_BYTES, "golden")?).map_err(|_| format!("golden {} is not UTF-8 text", p.display()))?;
        if !crate::vcs::sha256_hex(t.as_bytes()).starts_with(&r.hash) {
            return Err(format!("golden {} was modified: its text does not match its name", p.display()));
        }
        Ok(t)
    }

    pub(super) fn put(&mut self, check: &str, frame: u32, r: GoldenRef, bytes: &[u8]) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("cannot create {}: {e}", self.dir.display()))?;
        let p = self.file(&r)?;
        if !p.exists() {
            crate::fsutil::write_atomic(&p, bytes)?;
        }
        self.index.entries.entry(check.to_string()).or_default().insert(fkey(frame), r);
        self.dirty = true;
        Ok(())
    }

    pub fn put_img(&mut self, check: &str, frame: u32, renderer: &str, img: &Img) -> Result<String, String> {
        let hash = img.content_hash()[..24].to_string();
        self.put(check, frame, GoldenRef { hash: hash.clone(), renderer: renderer.into(), w: img.w, h: img.h }, &img.to_png()?)?;
        Ok(hash)
    }

    pub fn put_text(&mut self, check: &str, frame: u32, text: &str) -> Result<String, String> {
        let hash = crate::vcs::sha256_hex(text.as_bytes())[..24].to_string();
        self.put(check, frame, GoldenRef { hash: hash.clone(), renderer: "svg".into(), w: 0, h: 0 }, text.as_bytes())?;
        Ok(hash)
    }

    /// Drop index entries of checks that are no longer in the manifest or frames that are no longer listed.
    pub(super) fn retain(&mut self, check: &str, frames: &[u32]) {
        if let Some(m) = self.index.entries.get_mut(check) {
            let keep: std::collections::HashSet<String> = frames.iter().map(|&f| fkey(f)).collect();
            let before = m.len();
            m.retain(|k, _| keep.contains(k));
            if m.len() != before {
                self.dirty = true;
            }
        }
    }

    pub fn save(&mut self) -> Result<(), String> {
        if self.dirty {
            std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
            let p = self.dir.join("index.json");
            crate::fsutil::write_atomic(&p, serde_json::to_string_pretty(&self.index).unwrap().as_bytes())?;
            self.dirty = false;
        }
        Ok(())
    }

    /// Delete golden files that no index entry refers to. Returns the removed file names.
    pub fn prune(&self) -> Result<Vec<String>, String> {
        let used: std::collections::HashSet<PathBuf> = self.index.entries.values().flat_map(|m| m.values()).filter_map(|r| self.file(r).ok()).collect();
        let mut gone = Vec::new();
        let rd = match std::fs::read_dir(&self.dir) {
            Ok(r) => r,
            Err(_) => return Ok(gone),
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            let hashed = name.len() > 25 && name.as_bytes()[24] == b'.' && name[..24].chars().all(|c| c.is_ascii_hexdigit()) && (name.ends_with(".png") || name.ends_with(".svg"));
            if hashed && !used.contains(&p) {
                std::fs::remove_file(&p).map_err(|e| format!("cannot remove {}: {e}", p.display()))?;
                gone.push(name);
            }
        }
        gone.sort();
        Ok(gone)
    }
}

// ---- running -----------------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct RunOptions {
    /// Rewrite every golden of the selected render checks from the current render.
    pub update: bool,
    /// Accept only failing or missing goldens; passing ones are left untouched.
    pub bless: bool,
    /// Delete golden files nothing refers to.
    pub prune: bool,
    /// Run only checks with these names (empty = all).
    pub only: Vec<String>,
    /// Write failing frames (golden, actual, diff, side by side) as PNG files below this directory.
    pub artifacts: Option<PathBuf>,
}

pub(super) fn load_engine(base: &Path, project: &str, size: Option<&str>) -> Result<(Engine, u64), String> {
    let path = crate::assets::resolve(base, project)?;
    load_engine_at(&path, size)
}

/// Load the project at `path` (already resolved, or named by the user on the command line).
pub(super) fn load_engine_at(path: &Path, size: Option<&str>) -> Result<(Engine, u64), String> {
    let path = path.to_path_buf();
    let text = String::from_utf8(crate::assets::read_path(&path, crate::assets::MAX_DATA_BYTES, "project")?).map_err(|_| format!("project {} is not UTF-8", path.display()))?;
    let mut p = Project::from_json(&text).map_err(|e| format!("invalid project JSON in {}: {e}", path.display()))?;
    if let Some(s) = size {
        let (w, h) = crate::containers::parse_size(s)?;
        p = crate::containers::responsive(&p, w, h)?;
    }
    let dir = path.parent().map(|d| d.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let e = Engine::with_asset_dir(p, &dir).map_err(|errs| format!("{} is not a valid project: {}", path.display(), errs.join("; ")))?;
    Ok((e, text.len() as u64))
}

pub(super) fn pick_frames(frames: &Option<Vec<u32>>, step: &Option<u32>, total: u32) -> Result<Vec<u32>, String> {
    let total = total.max(1);
    let mut v: Vec<u32> = if let Some(f) = frames {
        if let Some(bad) = f.iter().find(|&&x| x >= total) {
            return Err(format!("frame {bad} is outside the project's 0..{} frames", total - 1));
        }
        f.clone()
    } else if let Some(s) = step {
        (0..total).step_by(*s as usize).collect()
    } else {
        vec![0, total / 2, total - 1]
    };
    v.sort_unstable();
    v.dedup();
    if v.len() > MAX_FRAMES_PER_CHECK {
        return Err(format!("{} frames selected, at most {MAX_FRAMES_PER_CHECK} per check", v.len()));
    }
    Ok(v)
}

pub(super) fn render_img(e: &Engine, renderer: &str, frame: u32, scale: f32) -> Result<Img, String> {
    let o = RasterOptions { scale, show_bones: false, transparent: false };
    match renderer {
        "raster" => {
            let (w, h, px) = render_rgba(e, frame as f32, &o);
            Img::new(w as usize, h as usize, px)
        }
        "float" => {
            let f = crate::fraster::render_float(e, frame as f32, &o);
            Img::new(f.width as usize, f.height as usize, f.to_straight_u8())
        }
        other => Err(format!("renderer '{other}' has no pixel output")),
    }
}

pub(super) fn render_svg(e: &Engine, frame: u32) -> String {
    crate::svg::render_frame(e, frame as f32, &crate::svg::RenderOptions::default())
}

pub(super) fn finish(name: &str, kind: &str, status: Status, message: String, t0: Stamp, frames: Vec<FrameResult>, details: Value) -> CheckResult {
    CheckResult { name: name.into(), kind: kind.into(), status, message, duration_ms: t0.elapsed().as_secs_f64() * 1000.0, frames, details }
}

pub(super) fn errored(name: &str, kind: &str, t0: Stamp, msg: String) -> CheckResult {
    finish(name, kind, Status::Error, msg, t0, vec![], Value::Null)
}

pub(super) fn compare_opts(t: &Resolved, ignore_px: Vec<[i64; 4]>) -> CompareOptions {
    CompareOptions { metric: t.metric, pixel_de: t.pixel_de, aa_isolated: t.aa_isolated, aa_edge_de: t.aa_edge_de, ignore: ignore_px, ..CompareOptions::default() }
}

pub(super) fn metrics_of(d: &percep::Diff) -> Metrics {
    Metrics {
        ssim: d.ssim,
        ssim_rgb: d.ssim_rgb,
        mse: d.mse,
        psnr: d.psnr,
        max_de: d.max_de,
        max_de_raw: d.max_de_raw,
        mean_de: d.mean_de,
        diff_pixels: d.diff_pixels,
        forgiven_pixels: d.forgiven_pixels,
        ignored_pixels: d.ignored_pixels,
        diff_fraction: d.diff_fraction,
        phash_distance: d.phash_distance,
    }
}

pub(super) fn frame_ok(frame: u32, status: Status, message: &str, golden: Option<String>) -> FrameResult {
    FrameResult { frame, status, message: message.into(), exact: false, golden, metrics: None, regions: vec![], regions_total: 0, images: FrameImages::default() }
}

/// The reasons (empty = within all thresholds) why a comparison fails the thresholds.
pub fn judge(d: &percep::Diff, t: &Resolved) -> Vec<String> {
    let mut why = Vec::new();
    if d.ssim < t.min_ssim {
        why.push(format!("SSIM {:.4} < {:.4}", d.ssim, t.min_ssim));
    }
    if d.ssim_rgb < t.min_ssim {
        why.push(format!("channel SSIM (min of R, G, B) {:.4} < {:.4}", d.ssim_rgb, t.min_ssim));
    }
    if d.max_de > t.max_de {
        why.push(format!("max {} {:.2} > {:.2}", t.metric.name(), d.max_de, t.max_de));
    }
    if d.diff_fraction > t.max_diff_fraction {
        why.push(format!("{:.4}% of pixels differ ({} px) > {:.4}%", d.diff_fraction * 100.0, d.diff_pixels, t.max_diff_fraction * 100.0));
    }
    if let Some(m) = t.max_phash_distance {
        if d.phash_distance > m {
            why.push(format!("pHash distance {} > {m}", d.phash_distance));
        }
    }
    why
}

pub(super) fn safe_write(dir: &Path, name: &str, bytes: &[u8]) {
    if std::fs::create_dir_all(dir).is_ok() {
        // atomic and symlink-refusing like every other output; a refused artifact is reported, not dropped silently
        if let Err(e) = crate::fsutil::write_atomic(&dir.join(name), bytes) {
            eprintln!("warning: artifact not written: {e}");
        }
    }
}

pub(super) fn run_render(c: &RenderCheck, base: &Path, project: &str, defaults: &Tol, store: &mut GoldenStore, o: &RunOptions) -> CheckResult {
    let t0 = Stamp::now();
    let (e, _) = match load_engine(base, project, c.size.as_deref()) {
        Ok(x) => x,
        Err(m) => return errored(&c.name, "render", t0, m),
    };
    let frames = match pick_frames(&c.frames, &c.step, e.project.frames) {
        Ok(f) => f,
        Err(m) => return errored(&c.name, "render", t0, m),
    };
    let th = c.thresholds.resolve(defaults);
    let writing = o.update || o.bless;
    if o.update {
        store.retain(&c.name, &frames);
    }
    let mut out = Vec::new();
    for &f in &frames {
        let fr = if c.renderer == "svg" { svg_frame(&e, &c.name, f, store, o) } else { pixel_frame(&e, c, f, &th, store, o) };
        out.push(match fr {
            Ok(r) => r,
            Err(m) => frame_ok(f, Status::Error, &m, None),
        });
        if let (Some(dir), true) = (&o.artifacts, out.last().map(|r| matches!(r.status, Status::Fail | Status::Blessed)).unwrap_or(false)) {
            let r = out.last().unwrap();
            let d = dir.join(&c.name);
            for (suffix, b) in [("golden", &r.images.golden), ("actual", &r.images.actual), ("diff", &r.images.heat)] {
                if let Some(b) = b {
                    safe_write(&d, &format!("f{:05}-{suffix}.png", f), b);
                }
            }
        }
    }
    let worst = out.iter().map(|r| r.status).max_by_key(|s| s.rank()).unwrap_or(Status::Pass);
    let bad = out.iter().filter(|r| r.status == Status::Fail).count();
    let msg = match worst {
        Status::Error => format!("{} frame(s) could not be evaluated", out.iter().filter(|r| r.status == Status::Error).count()),
        Status::Fail => format!("{bad} of {} frame(s) differ from the golden", out.len()),
        Status::Blessed => format!("{} golden(s) accepted", out.iter().filter(|r| r.status == Status::Blessed).count()),
        Status::Updated => format!("{} golden(s) written", out.iter().filter(|r| r.status == Status::Updated).count()),
        _ => format!("{} frame(s) match", out.len()),
    };
    let _ = writing;
    finish(&c.name, "render", worst, msg, t0, out, json!({"renderer": c.renderer, "scale": c.scale, "metric": th.metric.name(), "thresholds": {
        "pixel_de": th.pixel_de, "max_de": th.max_de, "min_ssim": th.min_ssim, "max_diff_fraction": th.max_diff_fraction,
        "aa_isolated": th.aa_isolated, "aa_edge_de": th.aa_edge_de, "max_phash_distance": th.max_phash_distance}}))
}
