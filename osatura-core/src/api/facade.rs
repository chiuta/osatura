// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `api` (see the module docs in `mod.rs`).
use super::*;

/// Version of this facade (semver, see the module docs). Independent of the crate version.
pub const API_VERSION: &str = "1.4.0";

/// Major component of [`API_VERSION`]; bumped only on incompatible changes.
pub const API_VERSION_MAJOR: u32 = 1;

/// Largest accepted output side in pixels (guards against absurd allocations).
pub const MAX_RENDER_SIDE: u32 = 16384;

/// Largest accepted project JSON in bytes.
pub const MAX_JSON_BYTES: usize = 256 * 1024 * 1024;

/// Level-of-detail grid used by `Player::set_screen_size`: level 1 is bounded by this many stage
/// units, each further level by twice the previous one.
pub(super) const LOD_TOLERANCE: f64 = 0.125;

pub(super) const LOD_LEVELS: usize = 8;

/// Every error the facade can return.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ApiError {
    /// The text is not valid JSON or does not describe a project (or is too large / too new).
    InvalidJson(String),
    /// The project parsed but failed validation; one message per problem.
    InvalidProject(Vec<String>),
    /// A caller argument was rejected (non-finite number, unknown property, scale out of range...).
    InvalidArgument(String),
    /// An edit was rejected (for example a locked channel); the project is unchanged.
    EditRejected(Vec<String>),
    /// An encoder failed or the format is not supported for this project.
    Export(String),
    /// An unexpected internal failure (a caught panic). Please report it as a bug.
    Internal(String),
    /// (1.3) The project file has uncommitted changes and the operation would overwrite them.
    DirtyTree(String),
    /// (1.3) A merge, cherry-pick or revert is still in progress: resolve it, commit or abort first.
    MergeInProgress(String),
    /// (1.3) Conflicts are still unresolved; one pointer per conflict (see [`vcs_conflicts`]).
    Conflicts(Vec<String>),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::InvalidJson(m) => write!(f, "invalid project JSON: {m}"),
            ApiError::InvalidProject(v) => write!(f, "invalid project: {}", v.join("; ")),
            ApiError::InvalidArgument(m) => write!(f, "invalid argument: {m}"),
            ApiError::EditRejected(v) => write!(f, "edit rejected: {}", v.join("; ")),
            ApiError::Export(m) => write!(f, "export failed: {m}"),
            ApiError::Internal(m) => write!(f, "internal error: {m}"),
            ApiError::DirtyTree(m) => write!(f, "uncommitted changes: {m}"),
            ApiError::MergeInProgress(m) => write!(f, "merge in progress: {m}"),
            ApiError::Conflicts(v) => write!(f, "unresolved conflicts: {}", v.join(", ")),
        }
    }
}

impl std::error::Error for ApiError {}

/// Interpolation of the segment that starts at a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Interpolation {
    Linear,
    /// Hold the value until the next key.
    Step,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// Catmull-Rom spline through neighbouring keys.
    Smooth,
}

impl Interpolation {
    pub(super) fn ease(self) -> Ease {
        match self {
            Interpolation::Linear => Ease::Linear,
            Interpolation::Step => Ease::Step,
            Interpolation::EaseIn => Ease::EaseIn,
            Interpolation::EaseOut => Ease::EaseOut,
            Interpolation::EaseInOut => Ease::EaseInOut,
            Interpolation::Smooth => Ease::Smooth,
        }
    }
}

/// Basic facts about a loaded animation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct AnimationInfo {
    pub name: String,
    /// Stage size in stage units.
    pub width: f32,
    pub height: f32,
    pub fps: f32,
    /// Number of frames; valid frame positions run `0..frames`.
    pub frames: u32,
}

/// Options for raster rendering.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RenderSettings {
    /// Output pixels per stage unit, within 0.05..=4.
    pub scale: f32,
    /// Draw the skeleton overlay.
    pub show_bones: bool,
    /// Leave the background transparent.
    pub transparent: bool,
}

impl RenderSettings {
    /// Settings with the given scale (other options off). Use this and the `with_*` methods:
    /// the struct is `#[non_exhaustive]`, so it cannot be built with a struct literal.
    pub fn new(scale: f32) -> RenderSettings {
        RenderSettings { scale, show_bones: false, transparent: false }
    }
    pub fn with_bones(mut self, show_bones: bool) -> RenderSettings {
        self.show_bones = show_bones;
        self
    }
    pub fn with_transparent(mut self, transparent: bool) -> RenderSettings {
        self.transparent = transparent;
        self
    }
}

impl Default for RenderSettings {
    fn default() -> Self {
        RenderSettings { scale: 1.0, show_bones: false, transparent: false }
    }
}

/// A rendered picture: straight-alpha RGBA8, row-major, top row first.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Whole-animation export targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExportFormat {
    /// Looping GIF of the whole timeline.
    Gif,
    /// Binary glTF 2.0.
    Glb,
    /// Lottie-style JSON document.
    Lottie,
    /// One self-contained HTML player page.
    Html,
    /// (1.4) Looping truecolour APNG of the whole timeline (`scale` within 0.05..=4).
    Apng,
}

// ---------------------------------------------------------------------------------------------
// Tools (1.2): level of detail, size-targeted export, mocap retarget, tracing, fuzzing, CI, history
// ---------------------------------------------------------------------------------------------

/// One derived level of detail (see [`Animation::lod_levels`]).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct LodLevel {
    /// 0 is the original, 1.. are the reduced levels.
    pub level: usize,
    /// Declared error bound in output pixels at the render scale the levels were built for.
    pub declared_px: f64,
    /// The same bound in stage units; multiply by a display scale to get pixels there.
    pub bound_stage: f64,
    /// Path anchors, layers and keys that remain.
    pub path_anchors: usize,
    pub keys: usize,
    pub layers: usize,
    /// The level as project JSON (load it with [`Animation::from_json`]).
    pub json: String,
}

impl LodLevel {
    /// Declared bound in pixels when shown at `screen_scale` output pixels per stage unit.
    pub fn bound_at(&self, screen_scale: f64) -> f64 {
        self.bound_stage * screen_scale
    }
}

/// Target format of [`Animation::fit_to_size`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FitFormat {
    Lottie,
    Html,
    Gif,
    Apng,
    /// OSATURA project JSON (keys are reduced).
    Project,
}

/// Result of [`Animation::fit_to_size`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SizeFit {
    /// Whether the budget was met (when not, `bytes` is the smallest file found).
    pub met: bool,
    pub size: usize,
    pub bytes: Vec<u8>,
    /// Lowest structural similarity (0..1) of the sampled frames against the original render.
    pub ssim_min: f64,
    /// Why the budget could not be met, if it could not.
    pub reason: Option<String>,
}

/// Options for [`Animation::retarget_bvh`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct MocapSettings {
    /// Drop horizontal root travel (keep the bob).
    pub in_place: bool,
    /// Flip the figure left-right.
    pub mirror: bool,
    /// Moving-average radius in output frames (0 = off).
    pub smooth: usize,
}

impl MocapSettings {
    pub fn new() -> MocapSettings {
        MocapSettings { in_place: false, mirror: false, smooth: 0 }
    }
    pub fn with_in_place(mut self, in_place: bool) -> MocapSettings {
        self.in_place = in_place;
        self
    }
    pub fn with_mirror(mut self, mirror: bool) -> MocapSettings {
        self.mirror = mirror;
        self
    }
    pub fn with_smooth(mut self, smooth: usize) -> MocapSettings {
        self.smooth = smooth;
        self
    }
}

impl Default for MocapSettings {
    fn default() -> Self {
        MocapSettings::new()
    }
}

/// What [`Animation::retarget_bvh`] did.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct MocapReport {
    /// `(BVH joint, rig bone)` pairs that were matched.
    pub matched: Vec<(String, String)>,
    pub unmatched_joints: Vec<String>,
    pub unmatched_bones: Vec<String>,
    /// Frames of animation written and bones driven.
    pub frames: usize,
    pub bones_used: usize,
    /// `target.prop` entries not written because the channel is locked.
    pub blocked: Vec<String>,
    pub notes: Vec<String>,
}

/// Summary of [`Animation::fuzz`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct FuzzSummary {
    pub samples: usize,
    pub failures: usize,
    /// 0..1
    pub failure_rate: f32,
    pub max_severity: f32,
    /// The complete report as JSON.
    pub json: String,
}

/// One commit of a project history (see [`vcs_log`]).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct VcsCommit {
    pub id: String,
    pub author: String,
    /// Seconds since the Unix epoch.
    pub time: u64,
    pub message: String,
}

/// Result of [`Animation::bake_physics`] (1.4).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct PhysicsBake {
    /// Channels written.
    pub added: usize,
    /// Bones whose existing keyed angles were kept (`overwrite = false`) or whose channel is locked.
    pub skipped: Vec<String>,
}

/// One branch (see [`vcs_branches`], 1.4).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct VcsBranch {
    pub name: String,
    /// Commit id the branch points to.
    pub commit: String,
    /// Whether it is the checked-out branch.
    pub current: bool,
}

/// One semantic change between two states of a project (see [`vcs_diff`], 1.4).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct VcsChange {
    /// `"Added"`, `"Removed"`, `"Changed"`, `"Moved"` or `"Retimed"`.
    pub kind: String,
    pub path: String,
    pub description: String,
}

/// Working-tree state (see [`vcs_status`], 1.4).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct VcsStatus {
    pub branch: Option<String>,
    pub head: Option<String>,
    /// Uncommitted changes against HEAD (everything when there is no commit yet).
    pub changes: Vec<VcsChange>,
    /// `"merge"`, `"cherry-pick"` or `"revert"` when one is unfinished.
    pub merge: Option<String>,
    /// Paths of unresolved conflicts.
    pub conflicts: Vec<String>,
}

/// One commit with what it changed (see [`vcs_show`], 1.4).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct VcsShow {
    pub commit: VcsCommit,
    pub changes: Vec<VcsChange>,
}

/// Result of [`vcs_gc`] (1.4).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct VcsGc {
    pub kept: usize,
    pub removed: usize,
}

/// Result of [`ci_run`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CiOutcome {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    /// Checks that could not be evaluated.
    pub errors: usize,
    /// 0 all good, 1 a check failed, 2 a check could not be evaluated.
    pub exit_code: i32,
    /// The complete report as JSON.
    pub json: String,
    /// (1.3) Checks whose goldens were rewritten (`CiMode::Update`).
    pub updated: usize,
    /// (1.3) Checks whose failing or missing goldens were accepted (`CiMode::Bless`).
    pub blessed: usize,
}

/// Trace a bitmap (PNG or JPEG bytes) into an SVG document of filled vector regions with at most
/// `colors` (2..=64) colours.
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// let png = Animation::demo().render_png(0.0, &RenderSettings::new(0.25))?;
/// let svg = trace_image_svg(&png, 6)?;
/// assert!(svg.starts_with("<svg") || svg.contains("<svg"));
/// assert!(trace_image_svg(b"not an image", 6).is_err());
/// # Ok(()) }
/// ```
pub fn trace_image_svg(image: &[u8], colors: usize) -> Result<String, ApiError> {
    guard(|| {
        let res = trace_bytes(image, colors)?;
        Ok(crate::autotrace::to_svg(&res))
    })
}

pub(super) fn trace_bytes(image: &[u8], colors: usize) -> Result<crate::autotrace::TraceResult, ApiError> {
    if !(2..=64).contains(&colors) {
        return Err(ApiError::InvalidArgument("colors must be within 2..=64".into()));
    }
    let img = crate::autotrace::decode_image(image).map_err(ApiError::InvalidArgument)?;
    let o = crate::autotrace::TraceOptions { colors, ..Default::default() };
    o.validate().map_err(ApiError::InvalidArgument)?;
    crate::autotrace::trace(&img, &o).map_err(ApiError::InvalidArgument)
}

/// The commits of a project's version history, newest first (read-only). `path` is the project
/// file or a directory inside its repository; `limit` is within 1..=10000. An empty history gives
/// an empty list.
pub fn vcs_log(path: &str, limit: usize) -> Result<Vec<VcsCommit>, ApiError> {
    if !(1..=10_000).contains(&limit) {
        return Err(ApiError::InvalidArgument("limit must be within 1..=10000".into()));
    }
    guard(|| {
        let repo = crate::vcs::Repo::open(std::path::Path::new(path)).map_err(ApiError::InvalidArgument)?;
        if repo.head_commit().map_err(ApiError::InvalidArgument)?.is_none() {
            return Ok(Vec::new());
        }
        let log = repo.log("HEAD", limit).map_err(ApiError::InvalidArgument)?;
        Ok(log.into_iter().map(|(id, c)| VcsCommit { id, author: c.author, time: c.time, message: c.message }).collect())
    })
}

/// Run an animation-CI manifest in verify mode (nothing is re-recorded). Relative paths in the
/// manifest resolve next to it. A failing check is not an error: look at [`CiOutcome::exit_code`].
pub fn ci_run(manifest_path: &str) -> Result<CiOutcome, ApiError> {
    guard(|| {
        let r = crate::ci::run(std::path::Path::new(manifest_path), &crate::ci::RunOptions::default()).map_err(ApiError::InvalidArgument)?;
        Ok(ci_outcome(&r))
    })
}

pub(super) fn ci_outcome(r: &crate::ci::CiReport) -> CiOutcome {
    CiOutcome {
        total: r.summary.total,
        passed: r.summary.passed,
        failed: r.summary.failed,
        errors: r.summary.errors,
        exit_code: r.exit_code(),
        json: r.to_json(),
        updated: r.summary.updated,
        blessed: r.summary.blessed,
    }
}
