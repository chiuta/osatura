// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `api` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// Animation CI (write operations), 1.3
// ---------------------------------------------------------------------------------------------

/// What [`ci_run_mode`] does with the goldens (recorded reference frames).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CiMode {
    /// Compare only; nothing is written (same as [`ci_run`]).
    Verify,
    /// Re-record every golden of the render checks from the current render.
    Update,
    /// Accept only the failing or missing goldens; passing ones stay untouched.
    Bless,
}

/// Result of [`ci_init`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CiInit {
    /// The manifest file that was written.
    pub manifest: String,
    /// `(kind, name)` of every generated check.
    pub checks: Vec<(String, String)>,
    /// The first run that recorded the goldens, `None` when `write_goldens` was false.
    pub recorded: Option<CiOutcome>,
}

/// Run an animation-CI manifest in the given [`CiMode`]. Like [`ci_run`], failing checks are
/// reported in the outcome, not as an error; an unreadable or invalid manifest is
/// [`ApiError::InvalidArgument`].
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-cirun-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # std::fs::write(dir.join("p.json"), Animation::demo().to_json()).unwrap();
/// # let manifest = dir.join("ci.json");
/// # std::fs::write(&manifest, r#"{"schema":"osatura-ci/1","project":"p.json","goldens":"goldens","checks":[{"kind":"render","name":"f","frames":[0],"scale":0.25}]}"#).unwrap();
/// let m = manifest.to_str().unwrap();
/// assert_eq!(ci_run_mode(m, CiMode::Verify)?.exit_code, 1);   // no golden recorded yet
/// let rec = ci_run_mode(m, CiMode::Update)?;
/// assert_eq!(rec.exit_code, 0, "{}", rec.json);
/// assert_eq!(ci_run_mode(m, CiMode::Verify)?.passed, 1);
/// assert!(ci_run_mode("/nonexistent-osatura.json", CiMode::Update).is_err());
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn ci_run_mode(manifest_path: &str, mode: CiMode) -> Result<CiOutcome, ApiError> {
    guard(|| {
        let o = crate::ci::RunOptions { update: mode == CiMode::Update, bless: mode == CiMode::Bless, ..Default::default() };
        let r = crate::ci::run(std::path::Path::new(manifest_path), &o).map_err(ApiError::InvalidArgument)?;
        Ok(ci_outcome(&r))
    })
}

/// Accept the current render of every failing or missing golden (shorthand for
/// [`ci_run_mode`] with [`CiMode::Bless`]).
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-cibless-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # std::fs::write(dir.join("p.json"), Animation::demo().to_json()).unwrap();
/// # let manifest = dir.join("ci.json");
/// # std::fs::write(&manifest, r#"{"schema":"osatura-ci/1","project":"p.json","goldens":"goldens","checks":[{"kind":"render","name":"f","frames":[0],"scale":0.25}]}"#).unwrap();
/// let m = manifest.to_str().unwrap();
/// let o = ci_bless(m)?;                       // missing golden is accepted
/// assert_eq!(o.exit_code, 0, "{}", o.json);
/// assert_eq!(ci_run(m)?.passed, 1);
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn ci_bless(manifest_path: &str) -> Result<CiOutcome, ApiError> {
    ci_run_mode(manifest_path, CiMode::Bless)
}

/// Create a manifest with default checks for `project_file`, calibrated on the project as it is
/// now. The manifest goes to `manifest` (default: `osatura-ci.json` next to the project, `ci::DEFAULT_MANIFEST`); an existing
/// file is never overwritten. With `write_goldens` the reference frames are recorded right away.
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-ciinit-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let init = ci_init(file.to_str().unwrap(), None, true)?;
/// assert!(init.checks.iter().any(|(kind, _)| kind == "validate"));
/// assert_eq!(init.recorded.as_ref().map(|r| r.errors), Some(0));
/// assert!(ci_init(file.to_str().unwrap(), None, false).is_err());   // manifest exists
/// assert_eq!(ci_run(&init.manifest)?.errors, 0);
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn ci_init(project_file: &str, manifest: Option<&str>, write_goldens: bool) -> Result<CiInit, ApiError> {
    guard(|| {
        let proj = std::path::Path::new(project_file);
        let out = match manifest {
            Some(m) => std::path::PathBuf::from(m),
            None => proj.parent().filter(|d| !d.as_os_str().is_empty()).map(|d| d.join(crate::ci::DEFAULT_MANIFEST)).unwrap_or_else(|| std::path::PathBuf::from(crate::ci::DEFAULT_MANIFEST)),
        };
        if out.exists() {
            return Err(ApiError::InvalidArgument(format!("{} already exists", out.display())));
        }
        let m = crate::ci::init_manifest(proj, &out).map_err(ApiError::InvalidArgument)?;
        let text = serde_json::to_string_pretty(&m).map_err(|e| ApiError::Internal(e.to_string()))?;
        crate::fsutil::write_atomic(&out, text.as_bytes()).map_err(|e| ApiError::InvalidArgument(format!("cannot write {e}")))?;
        let checks = m.checks.iter().map(|c| (c.kind().to_string(), c.name().to_string())).collect();
        let recorded = if write_goldens {
            let r = crate::ci::run(&out, &crate::ci::RunOptions { update: true, ..Default::default() }).map_err(ApiError::InvalidArgument)?;
            Some(ci_outcome(&r))
        } else {
            None
        };
        Ok(CiInit { manifest: out.to_string_lossy().to_string(), checks, recorded })
    })
}

/// A running state machine of an [`Animation`] (interactive playback). Create it with
/// [`Animation::player`], feed it inputs by name, advance time and render. The animation itself is
/// not changed; the player works on a copy of the project.
pub struct Player {
    pub(super) rt: crate::machine::MachineRuntime,
    /// Copy of the project used to solve layouts and to build level-of-detail levels.
    pub(super) proj: Project,
    /// Number of channels the project itself has (pose channels follow them).
    pub(super) base_channels: usize,
    pub(super) canvas: (f32, f32),
    pub(super) scrolls: Vec<(String, crate::containers::ScrollRuntime)>,
    /// Level-of-detail levels (built on first use, then cached) and the one in use.
    pub(super) lod_levels: Option<Vec<crate::lod::Lod>>,
    pub(super) lod_pick: usize,
    pub(super) lod_screen: Option<(f32, f32, f32)>,
}

/// A loaded, validated animation plus its undo history.
pub struct Animation {
    pub(super) editor: Editor,
}

pub(super) fn guard<T>(f: impl FnOnce() -> Result<T, ApiError>) -> Result<T, ApiError> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(_) => Err(ApiError::Internal("panic inside osatura-core".into())),
    }
}

pub(super) fn check_frame(a: &Animation, frame: f32) -> Result<f32, ApiError> {
    if !frame.is_finite() {
        return Err(ApiError::InvalidArgument("frame must be a finite number".into()));
    }
    Ok(frame.clamp(0.0, a.editor.project().frames.saturating_sub(1) as f32))
}

impl Animation {
    /// Parse and validate a project from JSON text.
    pub fn from_json(json: &str) -> Result<Animation, ApiError> {
        if json.len() > MAX_JSON_BYTES {
            return Err(ApiError::InvalidJson("project is larger than MAX_JSON_BYTES".into()));
        }
        guard(|| {
            let project = Project::from_json(json).map_err(|e| ApiError::InvalidJson(e.to_string()))?;
            Self::from_project(project)
        })
    }

    pub(super) fn from_project(project: Project) -> Result<Animation, ApiError> {
        let editor = Editor::new(project).map_err(ApiError::InvalidProject)?;
        Ok(Animation { editor })
    }

    /// Check a project JSON text without keeping it.
    pub fn validate_json(json: &str) -> Result<(), ApiError> {
        Animation::from_json(json).map(|_| ())
    }

    /// A small built-in example animation (useful for tests and first steps).
    pub fn demo() -> Animation {
        Animation { editor: Editor::new(crate::demo::project()).expect("built-in demo project is valid") }
    }

    /// The current project as pretty-printed JSON.
    pub fn to_json(&self) -> String {
        self.editor.project().to_json_pretty()
    }

    pub fn info(&self) -> AnimationInfo {
        let p = self.editor.project();
        AnimationInfo { name: p.name.clone(), width: p.width, height: p.height, fps: p.fps, frames: p.frames }
    }

    pub fn frame_count(&self) -> u32 {
        self.editor.project().frames
    }

    pub fn fps(&self) -> f32 {
        self.editor.project().fps
    }

    /// Bone names in rig order (the order of [`Animation::bone_world`]).
    pub fn bone_names(&self) -> Vec<String> {
        self.editor.project().bones.iter().map(|b| b.name.clone()).collect()
    }

    /// Layer paths (`group/child`), depth first.
    pub fn layer_names(&self) -> Vec<String> {
        self.editor.project().layer_paths()
    }

    /// World matrices of all bones at `frame` (clamped to the timeline), six floats per bone:
    /// `a b c d tx ty`. Fails only for a non-finite frame.
    pub fn bone_world(&self, frame: f32) -> Result<Vec<[f32; 6]>, ApiError> {
        let f = check_frame(self, frame)?;
        guard(|| Ok(self.editor.engine().evaluate(f).world.iter().map(|m| [m.a, m.b, m.c, m.d, m.tx, m.ty]).collect()))
    }

    /// Names of the runtime state machines stored in the project.
    pub fn machine_names(&self) -> Vec<String> {
        self.editor.project().machines.iter().map(|m| m.name.clone()).collect()
    }

    /// Start the named state machine at time 0 (states at their entry, inputs at their defaults).
    pub fn player(&self, machine: &str) -> Result<Player, ApiError> {
        guard(|| {
            let p = self.editor.project();
            let rt = crate::machine::MachineRuntime::new(p, machine).map_err(ApiError::InvalidArgument)?;
            let mut pl = Player {
                rt,
                proj: p.clone(),
                base_channels: p.channels.len(),
                canvas: (p.width, p.height),
                scrolls: Vec::new(),
                lod_levels: None,
                lod_pick: 0,
                lod_screen: None,
            };
            pl.init_scrolls()?;
            Ok(pl)
        })
    }

    pub(super) fn raster_opts(&self, s: &RenderSettings) -> Result<RasterOptions, ApiError> {
        if !s.scale.is_finite() || !(0.05..=4.0).contains(&s.scale) {
            return Err(ApiError::InvalidArgument("scale must be within 0.05..=4".into()));
        }
        let p = self.editor.project();
        let (w, h) = ((p.width * s.scale).round(), (p.height * s.scale).round());
        if w > MAX_RENDER_SIDE as f32 || h > MAX_RENDER_SIDE as f32 {
            return Err(ApiError::InvalidArgument(format!("output larger than {MAX_RENDER_SIDE} pixels per side")));
        }
        Ok(RasterOptions { scale: s.scale, show_bones: s.show_bones, transparent: s.transparent })
    }

    /// Render one frame to straight-alpha RGBA8.
    pub fn render_rgba(&self, frame: f32, settings: &RenderSettings) -> Result<Image, ApiError> {
        let f = check_frame(self, frame)?;
        let o = self.raster_opts(settings)?;
        guard(|| {
            let (width, height, pixels) = render_rgba(self.editor.engine(), f, &o);
            Ok(Image { width, height, pixels })
        })
    }

    /// Render one frame to a PNG file in memory.
    pub fn render_png(&self, frame: f32, settings: &RenderSettings) -> Result<Vec<u8>, ApiError> {
        let f = check_frame(self, frame)?;
        let o = self.raster_opts(settings)?;
        guard(|| render_png(self.editor.engine(), f, &o).map_err(ApiError::Export))
    }

    /// Render one frame to an SVG document.
    pub fn render_svg(&self, frame: f32, show_bones: bool) -> Result<String, ApiError> {
        let f = check_frame(self, frame)?;
        guard(|| Ok(render_frame(self.editor.engine(), f, &RenderOptions { show_bones })))
    }

    /// Encode the whole animation. `scale` applies to GIF only (within 0.05..=4).
    pub fn export(&self, format: ExportFormat, scale: f32) -> Result<Vec<u8>, ApiError> {
        let e = self.editor.engine();
        match format {
            ExportFormat::Gif => {
                let o = self.raster_opts(&RenderSettings { scale, ..RenderSettings::default() })?;
                guard(|| crate::export::gif_bytes(e, o.scale).map_err(ApiError::Export))
            }
            ExportFormat::Glb => guard(|| crate::gltf::export_glb(e).map_err(ApiError::Export)),
            ExportFormat::Lottie => guard(|| Ok(crate::export::lottie_json(e).to_string().into_bytes())),
            ExportFormat::Html => guard(|| Ok(crate::export::html_player(e).into_bytes())),
            ExportFormat::Apng => {
                let o = self.raster_opts(&RenderSettings { scale, transparent: true, ..RenderSettings::default() })?;
                guard(|| {
                    crate::raster::check_sequence(&e.project, o.scale).map_err(ApiError::Export)?;
                    let (mut w, mut h) = (0usize, 0usize);
                    let mut frames = Vec::with_capacity(e.project.frames as usize);
                    for f in 0..e.project.frames {
                        let (fw, fh, px) = render_rgba(e, f as f32, &o);
                        (w, h) = (fw as usize, fh as usize);
                        frames.push(px);
                    }
                    crate::sizefit::apng_encode_rgba(w, h, &frames, e.project.fps, 1).map_err(ApiError::Export)
                })
            }
        }
    }

    /// Set (or add) a keyframe on `target.prop`. `prop` is the snake_case property name used in
    /// project JSON, for example `"bone_angle"`. Validated, undoable, respects channel locks.
    pub fn set_key(&mut self, target: &str, prop: &str, frame: f32, value: f32, interpolation: Interpolation) -> Result<(), ApiError> {
        let (prop, frame) = self.key_args(prop, frame, value)?;
        guard(|| self.editor.set_key(target, prop, frame, value, interpolation.ease()).map_err(ApiError::EditRejected))
    }

    /// Delete the key at `frame` on `target.prop`; returns whether a key was removed.
    pub fn delete_key(&mut self, target: &str, prop: &str, frame: f32) -> Result<bool, ApiError> {
        let (prop, frame) = self.key_args(prop, frame, 0.0)?;
        guard(|| self.editor.delete_key(target, prop, frame).map_err(ApiError::EditRejected))
    }

    // ----- (1.4) editing basics: create layers and bones -----

    /// Append a filled rectangle layer. With `parent` = the name of a top-level group layer the
    /// layer goes into that group instead. Validated and undoable; `fill` is `#rgb`, `#rrggbb` or
    /// `#rrggbbaa`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_rect_layer(&mut self, name: &str, x: f32, y: f32, w: f32, h: f32, fill: &str, parent: Option<&str>) -> Result<(), ApiError> {
        if ![x, y, w, h].iter().all(|v| v.is_finite()) || w <= 0.0 || h <= 0.0 {
            return Err(ApiError::InvalidArgument("x, y must be finite and w, h positive finite numbers".into()));
        }
        let path = crate::path::VPath::rect("shape", x, y, w, h);
        self.add_vector_layer(name, path, fill, parent)
    }

    /// Append a filled ellipse layer (see [`Animation::add_rect_layer`]).
    #[allow(clippy::too_many_arguments)]
    pub fn add_ellipse_layer(&mut self, name: &str, cx: f32, cy: f32, rx: f32, ry: f32, fill: &str, parent: Option<&str>) -> Result<(), ApiError> {
        if ![cx, cy, rx, ry].iter().all(|v| v.is_finite()) || rx <= 0.0 || ry <= 0.0 {
            return Err(ApiError::InvalidArgument("cx, cy must be finite and rx, ry positive finite numbers".into()));
        }
        let path = crate::path::VPath::ellipse("shape", crate::math::Vec2::new(cx, cy), rx, ry);
        self.add_vector_layer(name, path, fill, parent)
    }

    /// Append an empty group layer (children inherit its transform). Add children with the
    /// `parent` argument of the other `add_*_layer` methods.
    pub fn add_group_layer(&mut self, name: &str) -> Result<(), ApiError> {
        self.insert_layer(crate::project::Layer::group(name, Vec::new()), None)
    }

    pub(super) fn add_vector_layer(&mut self, name: &str, path: crate::path::VPath, fill: &str, parent: Option<&str>) -> Result<(), ApiError> {
        if crate::style::Rgba::parse(fill).is_none() {
            return Err(ApiError::InvalidArgument(format!("bad colour '{fill}' (use #rgb, #rrggbb or #rrggbbaa)")));
        }
        self.insert_layer(crate::project::Layer::vector(name, vec![path.with_fill(fill)]), parent)
    }

    pub(super) fn insert_layer(&mut self, layer: crate::project::Layer, parent: Option<&str>) -> Result<(), ApiError> {
        if layer.name.trim().is_empty() {
            return Err(ApiError::InvalidArgument("layer name must not be empty".into()));
        }
        if layer.name.contains('/') {
            return Err(ApiError::InvalidArgument("layer name must not contain '/'".into()));
        }
        if self.layer_names().iter().any(|n| n == &layer.name || n.rsplit('/').next() == Some(layer.name.as_str())) {
            return Err(ApiError::InvalidArgument(format!("a layer named '{}' already exists", layer.name)));
        }
        let at = match parent {
            None => "/layers/-".to_string(),
            Some(g) => {
                let i = self
                    .editor
                    .project()
                    .layers
                    .iter()
                    .position(|l| l.name == g && matches!(l.kind, crate::project::LayerKind::Group { .. }))
                    .ok_or_else(|| ApiError::InvalidArgument(format!("no top-level group layer named '{g}'")))?;
                format!("/layers/{i}/layers/-")
            }
        };
        let value = serde_json::to_value(&layer).map_err(|e| ApiError::Internal(e.to_string()))?;
        guard(|| self.editor.apply(crate::edit::Op::Insert { path: at, value }).map_err(ApiError::EditRejected))
    }

    /// Append a bone. `parent` must already exist (parents precede children). `x`, `y` is the head
    /// position in the parent's space (world space for a root); `angle` is the rest rotation in
    /// degrees. Validated and undoable.
    pub fn add_bone(&mut self, name: &str, parent: Option<&str>, length: f32, x: f32, y: f32, angle: f32) -> Result<(), ApiError> {
        if ![length, x, y, angle].iter().all(|v| v.is_finite()) || length <= 0.0 {
            return Err(ApiError::InvalidArgument("length must be positive and all numbers finite".into()));
        }
        if name.trim().is_empty() {
            return Err(ApiError::InvalidArgument("bone name must not be empty".into()));
        }
        if self.bone_names().iter().any(|n| n == name) {
            return Err(ApiError::InvalidArgument(format!("a bone named '{name}' already exists")));
        }
        if let Some(p) = parent {
            if !self.bone_names().iter().any(|n| n == p) {
                return Err(ApiError::InvalidArgument(format!("unknown parent bone '{p}'")));
            }
        }
        let bone = crate::project::Bone::new(name, parent, length, crate::math::Vec2::new(x, y), angle);
        let value = serde_json::to_value(&bone).map_err(|e| ApiError::Internal(e.to_string()))?;
        guard(|| self.editor.apply(crate::edit::Op::Insert { path: "/bones/-".into(), value }).map_err(ApiError::EditRejected))
    }

    // ----- (1.4) physics bake and EXR -----

    /// Simulate a bone chain (each bone the child of the previous, root first) with a physics
    /// preset (`"hair"`, `"tail"`, `"rope"`, `"antenna"`, `"cape"`) and write the result as angle
    /// channels, as one undo step. With `overwrite = false`, bones that already have hand-made
    /// angle keys keep them (they are listed in [`PhysicsBake::skipped`]).
    pub fn bake_physics(&mut self, chain: &[&str], preset: &str, overwrite: bool) -> Result<PhysicsBake, ApiError> {
        let opts = crate::physics::preset(preset).ok_or_else(|| ApiError::InvalidArgument(format!("unknown preset '{preset}' (hair, tail, rope, antenna, cape)")))?;
        if chain.is_empty() {
            return Err(ApiError::InvalidArgument("the chain must name at least one bone".into()));
        }
        let names: Vec<String> = chain.iter().map(|s| s.to_string()).collect();
        guard(|| {
            let mut p = self.editor.project().clone();
            let sim = crate::physics::simulate_chain(&p, &names, &opts).map_err(ApiError::InvalidArgument)?;
            let (added, skipped) = crate::physics::merge_channels(&mut p, sim, overwrite);
            if added > 0 {
                let value = serde_json::to_value(&p.channels).map_err(|e| ApiError::Internal(e.to_string()))?;
                self.editor.apply(crate::edit::Op::Set { path: "/channels".into(), value }).map_err(ApiError::EditRejected)?;
            }
            Ok(PhysicsBake { added, skipped })
        })
    }

    /// Render one frame to an OpenEXR file in memory: linear light, premultiplied alpha, rendered
    /// by the float compositor. `float32 = false` stores HALF, `true` stores FLOAT.
    pub fn render_exr(&self, frame: f32, settings: &RenderSettings, float32: bool) -> Result<Vec<u8>, ApiError> {
        let f = check_frame(self, frame)?;
        let o = self.raster_opts(settings)?;
        guard(|| {
            let img = crate::fraster::render_float(self.editor.engine(), f, &o);
            let prec = if float32 { crate::hdr::Precision::Float } else { crate::hdr::Precision::Half };
            crate::hdr::encode_exr_buf(img.width, img.height, &img.rgba, prec).map_err(ApiError::Export)
        })
    }

    pub(super) fn key_args(&self, prop: &str, frame: f32, value: f32) -> Result<(Prop, f32), ApiError> {
        let p = Prop::parse(prop).ok_or_else(|| ApiError::InvalidArgument(format!("unknown property '{prop}'")))?;
        if !frame.is_finite() || !value.is_finite() {
            return Err(ApiError::InvalidArgument("frame and value must be finite numbers".into()));
        }
        Ok((p, frame))
    }

    /// Derive level-of-detail levels with a declared error bound: level 1 stays within
    /// `tolerance_px` output pixels at `scale` pixels per stage unit, each further level within
    /// twice the previous one; `levels` is 1..=12. Level 0 is the original. Deterministic; the
    /// bound covers timeline playback (see the `Player::set_screen_size` note for machines).
    ///
    /// ```
    /// use osatura_core::prelude::*;
    /// # fn main() -> Result<(), ApiError> {
    /// let anim = Animation::demo();
    /// let levels = anim.lod_levels(0.5, 1.0, 3)?;
    /// assert_eq!(levels.len(), 4); // the original plus three
    /// assert_eq!(levels[0].declared_px, 0.0);
    /// assert!(levels[3].declared_px >= levels[1].declared_px);
    /// let coarse = Animation::from_json(&levels[3].json)?; // every level is a normal project
    /// assert_eq!(coarse.frame_count(), anim.frame_count());
    /// # Ok(()) }
    /// ```
    pub fn lod_levels(&self, tolerance_px: f64, scale: f64, levels: usize) -> Result<Vec<LodLevel>, ApiError> {
        let o = crate::lod::LodOptions::new(tolerance_px, scale, levels);
        o.check().map_err(ApiError::InvalidArgument)?;
        guard(|| {
            let v = crate::lod::build(self.editor.project(), &o).map_err(ApiError::InvalidArgument)?;
            Ok(v.into_iter()
                .map(|l| LodLevel {
                    level: l.record.level,
                    declared_px: l.record.declared_px,
                    bound_stage: l.record.bound_stage,
                    path_anchors: l.record.stats.path_anchors,
                    keys: l.record.stats.keys,
                    layers: l.record.stats.layers,
                    json: l.project.to_json_pretty(),
                })
                .collect())
        })
    }

    /// Find the smallest export of `format` within `max_bytes` (1 KB..=1 GB) by lowering the
    /// quality knobs of that format step by step. When the budget cannot be met the smallest
    /// result is returned with `met == false`.
    pub fn fit_to_size(&self, format: FitFormat, max_bytes: usize) -> Result<SizeFit, ApiError> {
        if !(1000..=1_000_000_000).contains(&max_bytes) {
            return Err(ApiError::InvalidArgument("max_bytes must be within 1000..=1000000000".into()));
        }
        let f = match format {
            FitFormat::Lottie => crate::sizefit::Format::Lottie,
            FitFormat::Html => crate::sizefit::Format::Html,
            FitFormat::Gif => crate::sizefit::Format::Gif,
            FitFormat::Apng => crate::sizefit::Format::Apng,
            FitFormat::Project => crate::sizefit::Format::Project,
        };
        guard(|| {
            let r = crate::sizefit::fit(self.editor.project(), crate::sizefit::FitOptions::new(f, max_bytes)).map_err(ApiError::Export)?;
            Ok(SizeFit { met: r.met, size: r.bytes.len(), ssim_min: r.quality.ssim_min, reason: r.reason, bytes: r.bytes })
        })
    }

    /// Retarget a BVH motion-capture clip (text) onto this rig as keyed channels, in one undoable
    /// edit. Bones are matched by name and topology; locked channels are skipped and reported.
    pub fn retarget_bvh(&mut self, bvh: &str, settings: &MocapSettings) -> Result<MocapReport, ApiError> {
        guard(|| {
            let b = crate::mocap::parse_bvh(bvh).map_err(ApiError::InvalidArgument)?;
            let p = self.editor.project().clone();
            let m = crate::mocap::match_bones(&b, &p, None).map_err(ApiError::InvalidArgument)?;
            let o = crate::mocap::RetargetOptions { in_place: settings.in_place, mirror: settings.mirror, smooth: settings.smooth, ..Default::default() };
            let r = crate::mocap::retarget(&p, &b, &m, &o).map_err(ApiError::InvalidArgument)?;
            let op = crate::mocap::bake_op(&p, &r).map_err(ApiError::Internal)?;
            self.editor.apply(op).map_err(ApiError::EditRejected)?;
            Ok(MocapReport {
                matched: m.matches.iter().map(|x| (x.bvh_name.clone(), x.rig_name.clone())).collect(),
                unmatched_joints: m.unmatched_bvh.clone(),
                unmatched_bones: m.unmatched_rig.clone(),
                frames: r.out_frames,
                bones_used: r.bones_used,
                blocked: r.blocked.clone(),
                notes: r.notes.clone(),
            })
        })
    }

    /// Run a pose fuzz campaign (`samples` within 1..=100000 random poses from `seed`) and report
    /// where the rig breaks. The animation is only read.
    ///
    /// ```
    /// use osatura_core::prelude::*;
    /// # fn main() -> Result<(), ApiError> {
    /// let r = Animation::demo().fuzz(20, 1)?;
    /// assert_eq!(r.samples, 20);
    /// assert!(r.json.contains("failure_rate"));
    /// assert!(Animation::demo().fuzz(0, 1).is_err());
    /// # Ok(()) }
    /// ```
    pub fn fuzz(&self, samples: usize, seed: u64) -> Result<FuzzSummary, ApiError> {
        if !(1..=100_000).contains(&samples) {
            return Err(ApiError::InvalidArgument("samples must be within 1..=100000".into()));
        }
        guard(|| {
            let o = crate::fuzz::FuzzOptions { samples, seed, ..Default::default() };
            let r = crate::fuzz::run(self.editor.project(), None, &o).map_err(ApiError::InvalidArgument)?;
            Ok(FuzzSummary { samples: r.summary.samples, failures: r.summary.failures, failure_rate: r.summary.failure_rate, max_severity: r.summary.max_severity, json: r.to_json() })
        })
    }

    /// Build an animation from a bitmap: the image is traced into filled vector layers (at most
    /// `colors`, 2..=64) on a stage of the image size.
    pub fn from_image(image: &[u8], colors: usize, name: &str) -> Result<Animation, ApiError> {
        guard(|| {
            let res = trace_bytes(image, colors)?;
            Self::from_project(crate::autotrace::to_project(&res, name))
        })
    }

    /// Undo the last edit; false when there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        self.editor.undo()
    }

    /// Redo the last undone edit; false when there is nothing to redo.
    pub fn redo(&mut self) -> bool {
        self.editor.redo()
    }
}

impl Player {
    /// Set a bool input.
    pub fn set_bool(&mut self, name: &str, value: bool) -> Result<(), ApiError> {
        self.rt.set_bool(name, value).map_err(ApiError::InvalidArgument)
    }

    /// Set a number input (finite).
    pub fn set_number(&mut self, name: &str, value: f64) -> Result<(), ApiError> {
        self.rt.set_number(name, value).map_err(ApiError::InvalidArgument)
    }

    /// Fire a trigger input; it is consumed by the transition that tests it.
    pub fn fire(&mut self, name: &str) -> Result<(), ApiError> {
        self.rt.fire(name).map_err(ApiError::InvalidArgument)
    }

    /// Publish an external value (data binding path such as `player.health`).
    /// A scroll region whose `input` is this path takes the value as its offset in pixels along
    /// its main axis (as if dragged there; the scroll is then at rest).
    pub fn set_value(&mut self, path: &str, value: f64) -> Result<(), ApiError> {
        self.rt.set_value(path, value).map_err(ApiError::InvalidArgument)?;
        let mut hits: Vec<(String, [f32; 2])> = Vec::new();
        for s in self.proj.scrolls.iter().filter(|s| s.input.as_deref() == Some(path)) {
            let mut raw = [0.0f32; 2];
            match s.axis {
                crate::containers::ScrollAxis::X => raw[0] = value as f32,
                _ => raw[1] = value as f32,
            }
            hits.push((s.id.clone(), raw));
        }
        for (id, raw) in hits {
            if let Some((_, r)) = self.scrolls.iter_mut().find(|(i, _)| *i == id) {
                r.drag_to(raw);
            }
        }
        Ok(())
    }

    /// Advance the machine (and any scroll regions that are springing back or flinging) by `dt`
    /// seconds (0..=10).
    pub fn advance(&mut self, dt: f64) -> Result<(), ApiError> {
        guard(|| {
            self.rt.step(dt).map_err(ApiError::InvalidArgument)?;
            for (_, r) in &mut self.scrolls {
                r.step(dt as f32);
            }
            Ok(())
        })
    }

    pub(super) fn init_scrolls(&mut self) -> Result<(), ApiError> {
        self.scrolls.clear();
        if self.proj.scrolls.is_empty() {
            return Ok(());
        }
        let sol = crate::containers::solve(&self.proj, self.canvas.0, self.canvas.1).map_err(ApiError::InvalidArgument)?;
        for sc in &self.proj.scrolls {
            let max = sol.scrolls.iter().find(|i| i.id == sc.id).map_or([0.0; 2], |i| i.max);
            self.scrolls.push((sc.id.clone(), crate::containers::ScrollRuntime::new(sc, max)));
        }
        Ok(())
    }

    /// Lay the project out for another canvas size (stage units), as a responsive window resize:
    /// containers re-solve, scroll limits follow, and renders get that size. Fails for a size that
    /// is not finite, not positive or larger than the engine allows.
    pub fn set_canvas_size(&mut self, width: f32, height: f32) -> Result<(), ApiError> {
        crate::containers::responsive(&self.proj, width, height).map_err(ApiError::InvalidArgument)?;
        let old = self.canvas;
        self.canvas = (width, height);
        if let Err(e) = self.refresh_scroll_limits() {
            self.canvas = old;
            return Err(e);
        }
        Ok(())
    }

    pub(super) fn refresh_scroll_limits(&mut self) -> Result<(), ApiError> {
        if self.scrolls.is_empty() {
            return Ok(());
        }
        let sol = crate::containers::solve(&self.proj, self.canvas.0, self.canvas.1).map_err(ApiError::InvalidArgument)?;
        for (id, r) in &mut self.scrolls {
            if let Some(i) = sol.scrolls.iter().find(|i| i.id == *id) {
                r.set_max(i.max);
            }
        }
        Ok(())
    }

    /// Current canvas size in stage units (the project's own size until [`Player::set_canvas_size`]).
    pub fn canvas_size(&self) -> (f32, f32) {
        self.canvas
    }

    /// Ids of the project's scroll regions.
    pub fn scroll_ids(&self) -> Vec<String> {
        self.scrolls.iter().map(|(i, _)| i.clone()).collect()
    }

    pub(super) fn scroll_mut(&mut self, id: &str) -> Result<&mut crate::containers::ScrollRuntime, ApiError> {
        self.scrolls.iter_mut().find(|(i, _)| i == id).map(|(_, r)| r).ok_or_else(|| ApiError::InvalidArgument(format!("unknown scroll '{id}'")))
    }

    /// Finger / pointer drag: the raw offset in pixels the user asks for (content moved up / left
    /// by that much). Past the ends the displayed offset is rubber-banded or clamped.
    pub fn scroll_drag(&mut self, id: &str, x: f32, y: f32) -> Result<(), ApiError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(ApiError::InvalidArgument("scroll offset must be finite".into()));
        }
        self.scroll_mut(id)?.drag_to([x, y]);
        Ok(())
    }

    /// Let go with a release velocity in pixels per second; [`Player::advance`] then lets the
    /// region fling and settle (snap points, spring back).
    pub fn scroll_release(&mut self, id: &str, vx: f32, vy: f32) -> Result<(), ApiError> {
        if !vx.is_finite() || !vy.is_finite() {
            return Err(ApiError::InvalidArgument("release velocity must be finite".into()));
        }
        self.scroll_mut(id)?.release([vx, vy]);
        Ok(())
    }

    /// Displayed offset of a scroll region in pixels, or `None` for an unknown id.
    pub fn scroll_offset(&self, id: &str) -> Option<[f32; 2]> {
        self.scrolls.iter().find(|(i, _)| i == id).map(|(_, r)| r.pos)
    }

    /// Largest useful offset of a scroll region at the current canvas size.
    pub fn scroll_max(&self, id: &str) -> Option<[f32; 2]> {
        self.scrolls.iter().find(|(i, _)| i == id).map(|(_, r)| r.max())
    }

    /// Whether a scroll region has come to rest.
    pub fn scroll_settled(&self, id: &str) -> Option<bool> {
        self.scrolls.iter().find(|(i, _)| i == id).map(|(_, r)| r.is_settled())
    }

    /// Tell the player how large it is shown on screen so that it can pick a cheaper level of
    /// detail: the coarsest level whose declared error bound, at that screen scale, is at most
    /// `max_error_px` pixels. The levels are built on first use (deterministically) and cached.
    /// Only geometry and timeline keys are reduced; state-machine poses are not modelled by the
    /// bound (see `lod::LIMITS`). With `max_error_px` of 0 only levels whose declared bound is 0 (lossless ones) qualify.
    pub fn set_screen_size(&mut self, width_px: f32, height_px: f32, max_error_px: f32) -> Result<(), ApiError> {
        if !(width_px.is_finite() && height_px.is_finite() && width_px > 0.0 && height_px > 0.0) {
            return Err(ApiError::InvalidArgument("screen size must be finite and > 0".into()));
        }
        if !max_error_px.is_finite() || max_error_px < 0.0 {
            return Err(ApiError::InvalidArgument("max_error_px must be finite and >= 0".into()));
        }
        guard(|| {
            if self.lod_levels.is_none() {
                let lv = crate::lod::build(&self.proj, &crate::lod::LodOptions::new(LOD_TOLERANCE, 1.0, LOD_LEVELS)).map_err(ApiError::InvalidArgument)?;
                self.lod_levels = Some(lv);
            }
            let scale = crate::lod::fit_scale(&Project { width: self.canvas.0, height: self.canvas.1, ..self.proj.clone() }, width_px as f64, height_px as f64);
            let recs: Vec<_> = self.lod_levels.as_ref().map(|l| l.iter().map(|x| x.record.clone()).collect()).unwrap_or_default();
            self.lod_pick = crate::lod::pick_level(&recs, scale, max_error_px as f64);
            self.lod_screen = Some((width_px, height_px, max_error_px));
            Ok(())
        })
    }

    /// Forget the screen size: render the original again.
    pub fn clear_screen_size(&mut self) {
        self.lod_screen = None;
        self.lod_pick = 0;
    }

    /// Level of detail in use (0 = the original).
    pub fn lod_level(&self) -> usize {
        if self.lod_screen.is_some() {
            self.lod_pick
        } else {
            0
        }
    }

    /// Declared error bound in pixels of the level in use at the screen size given to
    /// [`Player::set_screen_size`]; 0 for the original or when no screen size is set.
    pub fn lod_bound_px(&self) -> f32 {
        let (Some((w, h, _)), Some(lv)) = (self.lod_screen, self.lod_levels.as_ref()) else { return 0.0 };
        let scale = crate::lod::fit_scale(&Project { width: self.canvas.0, height: self.canvas.1, ..self.proj.clone() }, w as f64, h as f64);
        lv.get(self.lod_pick).map_or(0.0, |l| (l.record.bound_stage * scale) as f32)
    }

    /// The project edits that make the playback engine: level of detail, canvas size, scroll offsets.
    pub(super) fn shape(&self, p: &mut Project, dropped: &[usize]) {
        if self.lod_level() > 0 {
            if let Some(l) = self.lod_levels.as_ref().and_then(|v| v.get(self.lod_pick)) {
                let pose: Vec<crate::project::Channel> = p.channels.split_off(self.base_channels.min(p.channels.len()));
                let mut q = l.project.clone();
                q.machines.clear();
                q.bindings.clear();
                q.channels.extend(pose);
                let mut i = 0usize;
                q.channels.retain(|_| {
                    i += 1;
                    !dropped.contains(&(i - 1))
                });
                *p = q;
            }
        }
        p.width = self.canvas.0;
        p.height = self.canvas.1;
        for s in &mut p.scrolls {
            if let Some((_, r)) = self.scrolls.iter().find(|(i, _)| *i == s.id) {
                for a in 0..2 {
                    s.offset[a] = crate::containers::raw_offset_for(r.pos[a], r.max()[a], s.elastic.as_ref());
                }
            }
        }
    }

    pub(super) fn build_engine(&self) -> Result<crate::engine::Engine, ApiError> {
        let mut dropped: Vec<usize> = Vec::new();
        loop {
            match self.rt.engine_with(|p| self.shape(p, &dropped)) {
                Ok(e) => return Ok(e),
                Err(m) => {
                    // a reduced level may have removed a layer that a pose channel animates: drop those channels
                    let floor = self.lod_levels.as_ref().and_then(|v| v.get(self.lod_pick)).map_or(usize::MAX, |l| l.project.channels.len());
                    let mut fresh = Vec::new();
                    if self.lod_level() > 0 {
                        for part in m.split("channel ").skip(1) {
                            if let Some(n) = part.split(':').next().and_then(|d| d.trim().parse::<usize>().ok()) {
                                if n >= floor && part.contains("unknown target") {
                                    // n counts the channels that are still present; map it back to the original position
                                    let mut left = n;
                                    let mut orig = 0usize;
                                    loop {
                                        if !dropped.contains(&orig) {
                                            if left == 0 {
                                                break;
                                            }
                                            left -= 1;
                                        }
                                        orig += 1;
                                    }
                                    if !fresh.contains(&orig) {
                                        fresh.push(orig);
                                    }
                                }
                            }
                        }
                    }
                    if fresh.is_empty() {
                        return Err(ApiError::InvalidProject(vec![m]));
                    }
                    // indices are positions in the shaped project; they stay valid because we drop highest first
                    dropped.extend(fresh);
                }
            }
        }
    }

    /// Current state of a layer.
    pub fn state(&self, layer: &str) -> Option<String> {
        self.rt.state_of(layer).map(str::to_string)
    }

    /// Events (state entered / exited) not yet taken, as a JSON array, without consuming them.
    pub fn events_json(&self) -> String {
        serde_json::to_string(self.rt.events()).unwrap_or_else(|_| "[]".into())
    }

    /// Discard the pending events.
    pub fn clear_events(&mut self) {
        self.rt.clear_events();
    }

    /// Events (state entered / exited) since the last call, as a JSON array.
    pub fn take_events_json(&mut self) -> String {
        serde_json::to_string(&self.rt.take_events()).unwrap_or_else(|_| "[]".into())
    }

    /// Render the current pose to straight-alpha RGBA8.
    pub fn render_rgba(&self, settings: &RenderSettings) -> Result<Image, ApiError> {
        if !settings.scale.is_finite() || !(0.05..=4.0).contains(&settings.scale) {
            return Err(ApiError::InvalidArgument("scale must be within 0.05..=4".into()));
        }
        if (self.canvas.0 * settings.scale).round() > MAX_RENDER_SIDE as f32 || (self.canvas.1 * settings.scale).round() > MAX_RENDER_SIDE as f32 {
            return Err(ApiError::InvalidArgument(format!("output larger than {MAX_RENDER_SIDE} pixels per side")));
        }
        guard(|| {
            let e = self.build_engine()?;
            let o = RasterOptions { scale: settings.scale, show_bones: settings.show_bones, transparent: settings.transparent };
            let (width, height, pixels) = render_rgba(&e, self.rt.base_frame(), &o);
            Ok(Image { width, height, pixels })
        })
    }

    /// Render the current pose to an SVG document.
    pub fn render_svg(&self, show_bones: bool) -> Result<String, ApiError> {
        guard(|| {
            let e = self.build_engine()?;
            Ok(render_frame(&e, self.rt.base_frame(), &RenderOptions { show_bones }))
        })
    }

    /// World matrices of all bones in the current pose (six floats per bone).
    pub fn bone_world(&self) -> Result<Vec<[f32; 6]>, ApiError> {
        guard(|| {
            let e = self.build_engine()?;
            Ok(e.evaluate(self.rt.base_frame()).world.iter().map(|m| [m.a, m.b, m.c, m.d, m.tx, m.ty]).collect())
        })
    }
}
