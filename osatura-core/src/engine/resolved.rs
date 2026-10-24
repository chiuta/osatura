// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `engine` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// Resolved (index-based) structures
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct IkResolved {
    pub(super) bones: Vec<usize>,
    pub(super) target: usize,
    pub(super) bend: f32,
    pub(super) iterations: u32,
    pub(super) soft: f32,
    pub(super) stretch: f32,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum CKind {
    Aim { target: usize, offset: f32 },
    CopyRotation { source: usize },
    Pin { target: usize },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ConstraintResolved {
    pub(super) bone: usize,
    pub(super) kind: CKind,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SpringResolved {
    pub(super) bone: usize,
    pub(super) frequency: f32,
    pub(super) damping: f32,
    pub(super) warmup: bool,
}

pub(super) struct ChanResolved {
    pub(super) prop: Prop,
    /// Bone / node / ik / constraint index, 0 (camera), or a path-slot index.
    pub(super) slot: usize,
    /// Index into `project.channels`.
    pub(super) chan: usize,
    pub(super) expr: Option<Expr>,
    /// Driven keys: the input value to read.
    pub(super) input: Option<(Prop, usize)>,
}

pub(super) struct NodeInfo {
    pub(super) path: String,
    pub(super) parent: Option<usize>,
    pub(super) children: Vec<usize>,
    /// Index path into `project.layers` / `children()`.
    pub(super) idx_path: Vec<usize>,
    pub(super) bone: Option<usize>,
    pub(super) mask: Option<(usize, bool)>,
    /// Top-level ancestor (its `parallax` governs the whole subtree).
    pub(super) root: usize,
}

/// Geometry bookkeeping for one vector layer.
#[derive(Default)]
pub(super) struct NodeGeo {
    /// per path: slot of each morph's weight
    pub(super) morph_slots: Vec<Vec<usize>>,
    /// per path, per anchor: `[dx slot, dy slot]`
    pub(super) point_slots: Vec<Vec<[usize; 2]>>,
    /// per path: `[fill.from, fill.to, stroke.from, stroke.to]` gradient handle slots `[dx, dy]` (only for gradient paints)
    pub(super) grad_slots: Vec<[Option<[usize; 2]>; 4]>,
    pub(super) deformers: Vec<DefInfo>,
}

pub(super) enum DefInfo {
    Lattice { cols: usize, rows: usize, rect: [f32; 4], slots: Vec<[usize; 2]> },
    Cloth { cols: usize, rows: usize, rect: [f32; 4], slots: Vec<[usize; 2]> },
    Spine { rest_pts: Vec<Vec2>, rest_poly: Polyline, slots: Vec<[usize; 2]>, radius: Option<f32> },
    Pin { solver: crate::pinwarp::PinSolver, slots: Vec<[usize; 2]> },
    /// `mesh_w`: mean-value weights of every rest mesh vertex (empty list = outside the cage),
    /// cached here when the cage is the layer's first deformer on an image layer.
    Cage { solver: crate::cage::CageSolver, slots: Vec<[usize; 2]>, mesh_w: Vec<Vec<f64>> },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LayerProps {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) rot: f32,
    pub(super) sx: f32,
    pub(super) sy: f32,
    pub(super) skew: f32,
    pub(super) opacity: f32,
    pub(super) switch: f32,
    pub(super) wobble: f32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CamProps {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) zoom: f32,
    pub(super) rot: f32,
    pub(super) focus: f32,
    pub(super) blur: f32,
}

#[derive(Clone)]
pub(super) struct State {
    pub(super) bones: Vec<BonePose>,
    pub(super) layers: Vec<LayerProps>,
    pub(super) cam: CamProps,
    pub(super) ik_mix: Vec<f32>,
    pub(super) cweight: Vec<f32>,
    pub(super) pvals: Vec<f32>,
}

impl State {
    /// Read a value in channel units (degrees for angles).
    pub(super) fn read(&self, prop: Prop, slot: usize) -> f32 {
        match prop {
            Prop::BoneAngle => self.bones[slot].angle.to_degrees(),
            Prop::BoneScale => self.bones[slot].scale,
            Prop::BoneOffsetX => self.bones[slot].offset.x,
            Prop::BoneOffsetY => self.bones[slot].offset.y,
            Prop::LayerOpacity => self.layers[slot].opacity,
            Prop::LayerX => self.layers[slot].x,
            Prop::LayerY => self.layers[slot].y,
            Prop::LayerRotation => self.layers[slot].rot,
            Prop::LayerScaleX => self.layers[slot].sx,
            Prop::LayerScaleY => self.layers[slot].sy,
            Prop::LayerSkew => self.layers[slot].skew,
            Prop::LayerSwitch => self.layers[slot].switch,
            Prop::LayerWobble => self.layers[slot].wobble,
            Prop::CameraX => self.cam.x,
            Prop::CameraY => self.cam.y,
            Prop::CameraZoom => self.cam.zoom,
            Prop::CameraRotation => self.cam.rot,
            Prop::CameraFocus => self.cam.focus,
            Prop::CameraBlur => self.cam.blur,
            Prop::IkMix => self.ik_mix[slot],
            Prop::ConstraintWeight => self.cweight[slot],
            Prop::PathMorph
            | Prop::PointDx
            | Prop::PointDy
            | Prop::LatticeDx
            | Prop::LatticeDy
            | Prop::SpineDx
            | Prop::SpineDy
            | Prop::PinDx
            | Prop::PinDy
            | Prop::CageDx
            | Prop::CageDy
            | Prop::FxBlur
            | Prop::FxDx
            | Prop::FxDy
            | Prop::FxSoft
            | Prop::FxAmount
            | Prop::GradDx
            | Prop::GradDy => self.pvals[slot],
        }
    }

    pub(super) fn write(&mut self, prop: Prop, slot: usize, v: f32) {
        match prop {
            Prop::BoneAngle => self.bones[slot].angle = v.to_radians(),
            Prop::BoneScale => self.bones[slot].scale = v,
            Prop::BoneOffsetX => self.bones[slot].offset.x = v,
            Prop::BoneOffsetY => self.bones[slot].offset.y = v,
            Prop::LayerOpacity => self.layers[slot].opacity = v.clamp(0.0, 1.0),
            Prop::LayerX => self.layers[slot].x = v,
            Prop::LayerY => self.layers[slot].y = v,
            Prop::LayerRotation => self.layers[slot].rot = v,
            Prop::LayerScaleX => self.layers[slot].sx = v,
            Prop::LayerScaleY => self.layers[slot].sy = v,
            Prop::LayerSkew => self.layers[slot].skew = v,
            Prop::LayerSwitch => self.layers[slot].switch = v,
            Prop::LayerWobble => self.layers[slot].wobble = v,
            Prop::CameraX => self.cam.x = v,
            Prop::CameraY => self.cam.y = v,
            Prop::CameraZoom => self.cam.zoom = v.max(0.01),
            Prop::CameraRotation => self.cam.rot = v,
            Prop::CameraFocus => self.cam.focus = v,
            Prop::CameraBlur => self.cam.blur = v.max(0.0),
            Prop::IkMix => self.ik_mix[slot] = v.clamp(0.0, 1.0),
            Prop::ConstraintWeight => self.cweight[slot] = v.clamp(0.0, 1.0),
            Prop::PathMorph
            | Prop::PointDx
            | Prop::PointDy
            | Prop::LatticeDx
            | Prop::LatticeDy
            | Prop::SpineDx
            | Prop::SpineDy
            | Prop::PinDx
            | Prop::PinDy
            | Prop::CageDx
            | Prop::CageDy
            | Prop::FxBlur
            | Prop::FxDx
            | Prop::FxDy
            | Prop::FxSoft
            | Prop::FxAmount
            | Prop::GradDx
            | Prop::GradDy => self.pvals[slot] = v,
        }
    }
}

/// Moves the handles of a gradient: linear `from` / `to` by their offsets; radial centre by `from`, radius by `to.x`.
pub(super) fn shift_gradient(p: &mut crate::style::Paint, from: Vec2, to: Vec2) {
    match p {
        crate::style::Paint::Linear { from: f, to: t, .. } => {
            *f += from;
            *t += to;
        }
        crate::style::Paint::Radial { center, radius, .. } => {
            *center += from;
            *radius = (*radius + to.x).max(0.0);
        }
        crate::style::Paint::Solid(_) => {}
    }
}

// ---------------------------------------------------------------------------------------------
// Public output types
// ---------------------------------------------------------------------------------------------

/// A decoded image plus its original PNG bytes (for SVG embedding).
pub struct ImageData {
    pub pixmap: sk::Pixmap,
    pub png: Vec<u8>,
}

impl std::fmt::Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImageData({}×{})", self.pixmap.width(), self.pixmap.height())
    }
}

/// A deformed triangle mesh (positions in layer space, UVs in image pixels).
#[derive(Clone, Debug)]
pub struct MeshOut {
    pub positions: Vec<Vec2>,
    pub uvs: Vec<Vec2>,
    pub tris: Vec<[u32; 3]>,
}

#[derive(Clone, Debug)]
pub struct ImageDraw {
    pub data: Arc<ImageData>,
    pub origin: Vec2,
    pub size: Vec2,
    pub mesh: Option<MeshOut>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Vector,
    Group,
    Switch,
    Image,
}

/// One layer, posed and ready to draw.
#[derive(Clone, Debug)]
pub struct RenderNode {
    pub id: usize,
    /// `/`-joined layer path.
    pub name: String,
    pub kind: NodeKind,
    /// Layer space → screen (includes camera). Geometry in `paths` is in layer space.
    pub world: Affine2,
    pub opacity: f32,
    pub blend: Blend,
    pub visible: bool,
    /// `(mask node id, inverted)`.
    pub mask: Option<(usize, bool)>,
    /// Deformed geometry (vector layers).
    pub paths: Vec<VPath>,
    /// Image layers.
    pub image: Option<ImageDraw>,
    /// Children to draw, bottom to top. For switch layers only the selected child.
    pub children: Vec<usize>,
    /// Enabled layer effects with their values at this frame (see `layerfx`); empty for most layers.
    pub effects: Vec<crate::layerfx::LayerEffect>,
}

/// Everything needed to draw one frame.
pub struct Frame {
    /// Bone world transforms (stage space).
    pub world: Vec<Affine2>,
    /// Camera matrix at parallax 1 (stage → screen); use it to draw stage-space overlays.
    pub camera: Affine2,
    pub nodes: Vec<RenderNode>,
    pub roots: Vec<usize>,
    /// Set when a computed number was not finite (a value overflowed `f32`, for instance a deep chain of large
    /// scales). The affected nodes are hidden and their transforms reset so nothing downstream sees NaN or
    /// infinity; this says where it happened. Exporters that can report errors refuse such a frame
    /// ([`Frame::check_finite`]).
    pub nonfinite: Option<String>,
}

impl Frame {
    /// `Err("non-finite value in <place>")` if evaluation had to repair a non-finite number.
    pub fn check_finite(&self) -> Result<(), String> {
        match &self.nonfinite {
            Some(w) => Err(format!("non-finite value in {w}")),
            None => Ok(()),
        }
    }
}

pub(super) fn aff_finite(m: &Affine2) -> bool {
    [m.a, m.b, m.c, m.d, m.tx, m.ty].iter().all(|v| v.is_finite())
}

/// Largest absolute value of any floating point number in a project (coordinates, scales, offsets, key values, ...).
pub const MAX_MAGNITUDE: f64 = 1.0e7;

pub(super) fn find_big(v: &serde_json::Value, path: &mut String) -> bool {
    match v {
        serde_json::Value::Number(n) => n.is_f64() && n.as_f64().is_some_and(|x| !matches!(x.abs().partial_cmp(&MAX_MAGNITUDE), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal))),
        serde_json::Value::Array(a) => a.iter().enumerate().any(|(i, x)| {
            let keep = path.len();
            path.push_str(&format!("/{i}"));
            if find_big(x, path) {
                return true;
            }
            path.truncate(keep);
            false
        }),
        serde_json::Value::Object(m) => m.iter().any(|(k, x)| {
            let keep = path.len();
            path.push_str(&format!("/{k}"));
            if find_big(x, path) {
                return true;
            }
            path.truncate(keep);
            false
        }),
        _ => false,
    }
}

/// Every floating point number in the project must lie within +-[`MAX_MAGNITUDE`] (so products of a few of them
/// still fit in `f32`); returns the JSON pointer of the first one that does not.
pub fn check_magnitudes(project: &Project) -> Result<(), String> {
    let v = serde_json::to_value(project).map_err(|e| e.to_string())?;
    check_json_magnitudes(&v)
}

/// [`check_magnitudes`] on JSON text that is already parsed. Reading a file goes through this, because a literal
/// such as `1e300` becomes infinity in an `f32` field and would then serialise as `null`, out of sight.
pub fn check_json_magnitudes(v: &serde_json::Value) -> Result<(), String> {
    let mut path = String::new();
    if find_big(v, &mut path) {
        let mut cur = v;
        for part in path.split('/').skip(1) {
            cur = match cur {
                serde_json::Value::Array(a) => &a[part.parse::<usize>().unwrap_or(0)],
                serde_json::Value::Object(m) => &m[part],
                x => x,
            };
        }
        return Err(format!("value {cur} at {path} is outside the supported range of +-{MAX_MAGNITUDE:e} (coordinates, scales, offsets and key values must stay below it)"));
    }
    Ok(())
}
