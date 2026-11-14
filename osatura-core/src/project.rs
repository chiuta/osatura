// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
use crate::anim::{Ease, Track};
use crate::math::{Affine2, Vec2};
use crate::path::{BoneWeight, VPath};
use crate::rig::{dist_to_segment, fk, resolve_parents, BonePose};
use crate::fx::{Blob, BoolOp, EmitterSpec};
use crate::style::{Blend, Paint, Rgba, StrokeStyle};
use serde::{Deserialize, Serialize};

/// A bone. Local space: origin at the bone's head, +x along the bone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bone {
    pub name: String,
    /// Parent bone name; parents must be listed before their children.
    #[serde(default)]
    pub parent: Option<String>,
    pub length: f32,
    /// Position of the head in the parent's space (roots: world position).
    #[serde(default)]
    pub rest_offset: Vec2,
    /// Rest rotation relative to the parent, in **degrees**.
    #[serde(default)]
    pub rest_angle: f32,
    /// `[min, max]` clamp for the animated rotation delta, degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle_limits: Option<[f32; 2]>,
    /// `[min, max]` clamp for the animated scale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_limits: Option<[f32; 2]>,
}

impl Bone {
    pub fn new(name: &str, parent: Option<&str>, length: f32, offset: Vec2, rest_angle: f32) -> Bone {
        Bone {
            name: name.into(),
            parent: parent.map(String::from),
            length,
            rest_offset: offset,
            rest_angle,
            angle_limits: None,
            scale_limits: None,
        }
    }
    pub fn with_angle_limits(mut self, min: f32, max: f32) -> Self {
        self.angle_limits = Some([min, max]);
        self
    }
}

/// Inverse kinematics for a chain of bones (`bones[0]` = root … last = tip; each
/// bone must be a direct child of the previous one). Exactly two bones use an
/// analytic solver with a bend direction; longer chains use FABRIK. The end of the
/// tip bone is pulled toward the head of the `target` bone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IkChain {
    pub name: String,
    pub bones: Vec<String>,
    pub target: String,
    /// +1 or −1: which way a two-bone chain folds.
    #[serde(default = "one_f")]
    pub bend: f32,
    /// Initial FK↔IK blend, 0 = pure FK pose, 1 = pure IK (animate with `ik_mix`).
    #[serde(default = "one_f")]
    pub mix: f32,
    /// FABRIK iterations for chains longer than two bones: a floor. The solver exits early once the tip is within 0.01 px of the target and otherwise continues up to 256 sweeps (`rig::FABRIK_MAX_ITERS`).
    #[serde(default = "def_iters")]
    pub iterations: u32,
    /// "Soft" IK: fraction of the chain's reach (0..0.9) over which the extension eases off
    /// instead of snapping straight. 0 = off. Ignored when `stretch` > 1.
    #[serde(default)]
    pub soft: f32,
    /// Maximum stretch factor (1..4). Above 1 the chain lengthens (the root bone's scale grows,
    /// children inherit it) to reach targets beyond its natural length. 1 = off.
    #[serde(default = "one_f")]
    pub stretch: f32,
}

fn def_threshold() -> f32 {
    1.0
}
fn def_cell() -> f32 {
    3.0
}

fn def_iters() -> u32 {
    16
}

impl IkChain {
    pub fn two_bone(name: &str, root: &str, tip: &str, target: &str, bend: f32) -> IkChain {
        IkChain {
            name: name.into(),
            bones: vec![root.into(), tip.into()],
            target: target.into(),
            bend,
            mix: 1.0,
            iterations: 16,
            soft: 0.0,
            stretch: 1.0,
        }
    }
    pub fn chain(name: &str, bones: &[&str], target: &str) -> IkChain {
        IkChain {
            name: name.into(),
            bones: bones.iter().map(|s| s.to_string()).collect(),
            target: target.into(),
            bend: 1.0,
            mix: 1.0,
            iterations: 16,
            soft: 0.0,
            stretch: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConstraintKind {
    /// Rotate the bone so its +x axis points at the target bone's head (+ `offset` degrees).
    Aim {
        target: String,
        #[serde(default)]
        offset: f32,
    },
    /// Match the world rotation of another bone.
    CopyRotation { source: String },
    /// Slide the bone's head to the target bone's head ("pin").
    Pin { target: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub name: String,
    pub bone: String,
    #[serde(flatten)]
    pub kind: ConstraintKind,
    /// Influence 0..1 (animate with `constraint_weight`).
    #[serde(default = "one_f")]
    pub weight: f32,
}

/// Secondary motion: the bone's world rotation follows its animated rotation
/// through a damped spring (lag, overshoot, drag). Simulated deterministically
/// and baked, so any frame can be evaluated in any order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spring {
    pub bone: String,
    /// Natural frequency in Hz (higher = stiffer).
    #[serde(default = "def_freq")]
    pub frequency: f32,
    /// Damping ratio: < 1 rings and overshoots, 1 = critically damped.
    #[serde(default = "def_damp")]
    pub damping: f32,
    /// Pre-roll one pass of the timeline so looping animations start in steady state.
    #[serde(default = "yes")]
    pub warmup: bool,
}

fn def_freq() -> f32 {
    3.0
}
fn def_damp() -> f32 {
    0.35
}

pub(crate) fn one_f() -> f32 {
    1.0
}
pub(crate) fn yes() -> bool {
    true
}
fn one_vec() -> Vec2 {
    Vec2::new(1.0, 1.0)
}

/// Layer transform: `T(position) · R(rotation) · Shear(skew) · S(scale) · T(−anchor)`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    #[serde(default)]
    pub position: Vec2,
    /// Degrees.
    #[serde(default)]
    pub rotation: f32,
    #[serde(default = "one_vec")]
    pub scale: Vec2,
    /// Horizontal shear in degrees.
    #[serde(default)]
    pub skew: f32,
    /// Pivot, in the layer's own coordinates.
    #[serde(default)]
    pub anchor: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Transform2D { position: Vec2::ZERO, rotation: 0.0, scale: one_vec(), skew: 0.0, anchor: Vec2::ZERO }
    }
}

impl Transform2D {
    pub fn matrix(&self) -> Affine2 {
        compose_transform(self.position, self.rotation, self.scale, self.skew, self.anchor)
    }
}

pub fn compose_transform(pos: Vec2, rot_deg: f32, scale: Vec2, skew_deg: f32, anchor: Vec2) -> Affine2 {
    Affine2::translate(pos)
        .mul(Affine2::rotate(rot_deg.to_radians()))
        .mul(Affine2::shear_x(skew_deg.to_radians()))
        .mul(Affine2::scale_xy(scale.x, scale.y))
        .mul(Affine2::translate(-anchor))
}

/// Mask a layer by the alpha of another layer (addressed by its `/`-joined path).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaskRef {
    pub layer: String,
    #[serde(default)]
    pub invert: bool,
}

/// Hand-drawn "boil": points jitter with coherent noise that re-seeds `rate` times per second.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wobble {
    /// Maximum displacement in layer units (animate with `layer_wobble`).
    pub amp: f32,
    /// Spatial frequency: noise cycles per layer unit.
    #[serde(default = "def_wscale")]
    pub scale: f32,
    /// How many different "boil" poses per second.
    #[serde(default = "def_rate")]
    pub rate: f32,
    #[serde(default)]
    pub seed: u32,
}

fn def_wscale() -> f32 {
    0.03
}
fn def_rate() -> f32 {
    8.0
}

/// Shape deformers applied (in rest space, before skinning) to a vector or image layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Deformer {
    /// Free-form deformation: a Bézier control lattice over `rect = [x, y, w, h]`.
    /// Control point `(c, r)` is addressed as `"layer::name::c,r"` by `lattice_dx/dy`.
    Lattice {
        name: String,
        cols: usize,
        rows: usize,
        rect: [f32; 4],
        /// Static offsets, row-major `(cols+1)·(rows+1)`; empty = all zero.
        #[serde(default)]
        offsets: Vec<Vec2>,
    },
    /// Bend geometry along a curve ("curver"): anchors project onto the rest curve
    /// and are re-attached to the deformed one. Spine point `i` is `"layer::name::i"`
    /// for `spine_dx/dy`.
    Spine {
        name: String,
        rest: Vec<Vec2>,
        #[serde(default)]
        offsets: Vec<Vec2>,
        /// Only affect geometry within this distance of the curve (soft edge); `None` = everything.
        #[serde(default)]
        radius: Option<f32>,
    },
    /// Pin warp: drag pins, the artwork follows smoothly within `radius` of each pin
    /// (compact radial-basis interpolation, see `pinwarp`). Pin `i` is `"layer::name::i"`
    /// for `pin_dx/dy`.
    Pin {
        name: String,
        pins: Vec<Vec2>,
        #[serde(default)]
        offsets: Vec<Vec2>,
        radius: f32,
    },
    /// Per-vertex displacement grid over `rect = [x, y, w, h]`: bilinear, so every grid vertex
    /// moves exactly by its own offset (a lattice is a smooth Bézier approximation instead).
    /// Vertex `(c, r)` is addressed as `"layer::name::c,r"` by `lattice_dx/dy`. Written by the
    /// cloth baker (`cloth.rs`); on an image layer give the mesh the same `cols x rows`.
    Cloth {
        name: String,
        cols: usize,
        rows: usize,
        rect: [f32; 4],
        /// Static offsets, row-major `(cols+1)·(rows+1)`; empty = all zero.
        #[serde(default)]
        offsets: Vec<Vec2>,
    },
    /// Cage deformer (mean-value coordinates, `cage.rs`): content inside the closed polygon
    /// `rest` follows its vertices; content outside is left unchanged. Vertex `i` is
    /// `"layer::name::i"` for `cage_dx/dy`.
    Cage {
        name: String,
        rest: Vec<Vec2>,
        #[serde(default)]
        offsets: Vec<Vec2>,
    },
}

impl Deformer {
    pub fn name(&self) -> &str {
        match self {
            Deformer::Lattice { name, .. } | Deformer::Spine { name, .. } | Deformer::Pin { name, .. } | Deformer::Cloth { name, .. } | Deformer::Cage { name, .. } => name,
        }
    }
}

/// Where an image's pixels come from. Prefer `png_base64` for self-contained projects.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub png_base64: Option<String>,
}

/// A grid mesh over an image for bone-driven deformation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MeshSpec {
    #[serde(default)]
    pub cols: usize,
    #[serde(default)]
    pub rows: usize,
    /// Per vertex (row-major, `(cols+1)·(rows+1)`; with `free`: one list per free vertex);
    /// empty = unskinned (use `auto_skin_mesh`).
    #[serde(default)]
    pub weights: Vec<Vec<BoneWeight>>,
    /// Free-form triangle mesh (e.g. from `meshtrace`). When present, `cols`/`rows` are ignored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free: Option<FreeMesh>,
}

/// A free-form triangle mesh over an image layer (vertices in layer space).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FreeMesh {
    /// Rest positions in layer space.
    pub vertices: Vec<Vec2>,
    /// Triangles as vertex indices (counter-clockwise on screen, as the grid mesh).
    pub triangles: Vec<[u32; 3]>,
    /// Texture coordinates in image pixels; empty = derived from the positions
    /// (`(v - origin) / size * pixel size`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uvs: Vec<Vec2>,
}

impl MeshSpec {
    /// Rest vertices of this mesh over an image at `origin` with displayed `size`.
    pub fn rest_vertices(&self, origin: Vec2, size: Vec2) -> Vec<Vec2> {
        match &self.free {
            Some(f) => f.vertices.clone(),
            None => mesh_vertices(origin, size, self.cols, self.rows),
        }
    }
    /// Triangles (as `usize` indices) of this mesh.
    pub fn tri_indices(&self) -> Vec<[usize; 3]> {
        match &self.free {
            Some(f) => f.triangles.iter().map(|t| [t[0] as usize, t[1] as usize, t[2] as usize]).collect(),
            None => crate::weights::grid_triangles(self.cols, self.rows),
        }
    }
    /// Number of vertices (`weights` must match).
    pub fn vertex_count(&self) -> usize {
        match &self.free {
            Some(f) => f.vertices.len(),
            None => (self.cols.max(1) + 1) * (self.rows.max(1) + 1),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LayerKind {
    /// Bézier shapes.
    Vector { paths: Vec<VPath> },
    /// Container; children inherit its transform.
    Group { layers: Vec<Layer> },
    /// Shows exactly one child (frame-by-frame drawings, symbol/mouth swaps).
    /// Animate `layer_switch` with step keys to pick the child.
    #[serde(rename = "switch", alias = "selector")]
    Selector {
        layers: Vec<Layer>,
        #[serde(default)]
        selected: usize,
    },
    /// Metaball liquid: soft blobs that fuse when they approach each other.
    #[serde(rename = "liquid", alias = "metaball")]
    Metaball {
        blobs: Vec<Blob>,
        /// Iso-level of the field Σ r²/d² (1.0 = each blob is a circle of its radius).
        #[serde(default = "def_threshold")]
        threshold: f32,
        /// Marching-squares grid size in layer units (smaller = smoother, slower).
        #[serde(default = "def_cell")]
        cell: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<Paint>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<StrokeStyle>,
    },
    /// Polygon boolean: `paths[0]` combined with the union of the remaining paths.
    Boolean { op: BoolOp, paths: Vec<VPath> },
    /// Deterministic particle emitter.
    Emitter { spec: EmitterSpec },
    /// A PNG, optionally with a deformation mesh.
    Image {
        source: ImageSource,
        /// Top-left corner in layer space.
        #[serde(default)]
        origin: Vec2,
        /// Displayed size; `None` = the image's pixel size.
        #[serde(default)]
        size: Option<Vec2>,
        #[serde(default)]
        mesh: Option<MeshSpec>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    #[serde(flatten)]
    pub kind: LayerKind,
    #[serde(default)]
    pub transform: Transform2D,
    #[serde(default = "one_f")]
    pub opacity: f32,
    #[serde(default = "yes")]
    pub visible: bool,
    #[serde(default)]
    pub blend: Blend,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<MaskRef>,
    /// Ride a bone rigidly (follows its motion relative to the rest pose).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bone: Option<String>,
    /// Multiplane parallax for top-level layers: 0 = fixed to the screen,
    /// 1 = moves with the camera, <1 = far away, >1 = close up.
    #[serde(default = "one_f")]
    pub parallax: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deformers: Vec<Deformer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wobble: Option<Wobble>,
    /// Tracing aid: shown only where the engine's `show_reference` is on (editor), never in exports.
    #[serde(default, skip_serializing_if = "is_false")]
    pub reference: bool,
    /// Editing lock (R-100): canvas picking and boxes skip the layer and its descendants, gizmo drags, nudges,
    /// deletes and timeline edits of its channels refuse. Rendering and evaluation ignore it, and plain
    /// `Op`s (the property panel, scripts, undo) still work, which is how a layer is unlocked again.
    #[serde(default, skip_serializing_if = "is_false")]
    pub locked: bool,
    /// Tangent-space normal map used by the `relight` post effect for the pixels this layer (and
    /// its children without a map of their own) covers, instead of the global map or the bevel.
    /// See `postfx::layer_normal_field`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normal_map: Option<crate::postfx::NormalMap>,
    /// Per-layer effects (blur, drop shadow, tint, fill), applied in order to this layer (and its children)
    /// in an isolated pass before its mask, opacity and blend mode. See `layerfx`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<crate::layerfx::LayerEffect>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Layer {
    fn base(name: &str, kind: LayerKind) -> Layer {
        Layer {
            name: name.into(),
            kind,
            transform: Transform2D::default(),
            opacity: 1.0,
            visible: true,
            blend: Blend::Normal,
            mask: None,
            bone: None,
            parallax: 1.0,
            deformers: Vec::new(),
            wobble: None,
            reference: false,
            locked: false,
            normal_map: None,
            effects: Vec::new(),
        }
    }
    pub fn vector(name: &str, paths: Vec<VPath>) -> Layer {
        Layer::base(name, LayerKind::Vector { paths })
    }
    pub fn group(name: &str, layers: Vec<Layer>) -> Layer {
        Layer::base(name, LayerKind::Group { layers })
    }
    pub fn switch(name: &str, layers: Vec<Layer>) -> Layer {
        Layer::base(name, LayerKind::Selector { layers, selected: 0 })
    }
    pub fn liquid(name: &str, blobs: Vec<Blob>, fill: Paint) -> Layer {
        Layer::base(name, LayerKind::Metaball { blobs, threshold: 1.0, cell: 3.0, fill: Some(fill), stroke: None })
    }
    pub fn boolean(name: &str, op: BoolOp, paths: Vec<VPath>) -> Layer {
        Layer::base(name, LayerKind::Boolean { op, paths })
    }
    pub fn emitter(name: &str, spec: EmitterSpec) -> Layer {
        Layer::base(name, LayerKind::Emitter { spec })
    }
    pub fn image(name: &str, source: ImageSource, origin: Vec2) -> Layer {
        Layer::base(name, LayerKind::Image { source, origin, size: None, mesh: None })
    }
    /// Embed PNG bytes (self-contained).
    pub fn image_from_png(name: &str, png: &[u8], origin: Vec2) -> Layer {
        Layer::image(name, ImageSource { file: None, png_base64: Some(crate::base64::encode(png)) }, origin)
    }

    /// Frame-by-frame animation: a switch layer holding one child per drawing
    /// (`name/d0`, `name/d1`, …). Pair it with [`Channel::exposure`].
    pub fn frame_by_frame(name: &str, drawings: Vec<Vec<VPath>>) -> Layer {
        Layer::switch(
            name,
            drawings.into_iter().enumerate().map(|(i, paths)| Layer::vector(&format!("d{i}"), paths)).collect(),
        )
    }

    pub fn children(&self) -> &[Layer] {
        match &self.kind {
            LayerKind::Group { layers } | LayerKind::Selector { layers, .. } => layers,
            _ => &[],
        }
    }
    pub fn children_mut(&mut self) -> Option<&mut Vec<Layer>> {
        match &mut self.kind {
            LayerKind::Group { layers } | LayerKind::Selector { layers, .. } => Some(layers),
            _ => None,
        }
    }
}

/// What an animation channel drives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Prop {
    /// Bone rotation delta, degrees.
    BoneAngle,
    BoneScale,
    BoneOffsetX,
    BoneOffsetY,
    /// Layer opacity 0..1 (ignored for hidden layers).
    LayerOpacity,
    LayerX,
    LayerY,
    /// Degrees.
    LayerRotation,
    LayerScaleX,
    LayerScaleY,
    /// Degrees.
    LayerSkew,
    /// Selected child index of a switch layer (rounded).
    LayerSwitch,
    /// Wobble amplitude of a layer.
    LayerWobble,
    CameraX,
    CameraY,
    CameraZoom,
    /// Degrees.
    CameraRotation,
    /// Depth-of-field focus: the layer `parallax` that is sharp (used by `dof`).
    CameraFocus,
    /// Depth-of-field strength: blur radius in output pixels per unit of parallax distance.
    CameraBlur,
    /// FK↔IK blend of an IK chain (target = chain name).
    IkMix,
    /// Constraint influence (target = constraint name).
    ConstraintWeight,
    /// Morph weight; target `"layer::path::morph"`.
    PathMorph,
    /// Anchor offset; target `"layer::path::anchorIndex"`.
    PointDx,
    PointDy,
    /// Lattice control point offset; target `"layer::deformer::col,row"`.
    LatticeDx,
    LatticeDy,
    /// Spine control point offset; target `"layer::deformer::pointIndex"`.
    SpineDx,
    SpineDy,
    /// Pin-warp pin offset; target `"layer::deformer::pinIndex"`.
    PinDx,
    PinDy,
    /// Cage vertex offset; target `"layer::deformer::vertexIndex"`.
    CageDx,
    CageDy,
    /// Layer effect numbers; target `"layer::fxN"` (see `layerfx`). Blur radius.
    FxBlur,
    /// Drop shadow offset x / y (stage units).
    FxDx,
    FxDy,
    /// Drop shadow softness (blur radius).
    FxSoft,
    /// Main 0..1 amount: shadow opacity, tint amount or fill opacity.
    FxAmount,
    /// Gradient handle offset of a vector path; target `"layer::path::fill.from"`, `fill.to`, `stroke.from` or
    /// `stroke.to`. Linear gradients: the start / end point moves by (dx, dy). Radial gradients: `.from` moves
    /// the centre, `.to` changes the radius by `dx` (`dy` is ignored). Only paths whose fill / stroke is a gradient
    /// have these targets.
    GradDx,
    GradDy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    Bone,
    Layer,
    Camera,
    Ik,
    Constraint,
    /// Path / deformer sub-targets (`layer::name::sub`).
    Sub,
}

impl Prop {
    pub fn target_kind(self) -> TargetKind {
        use Prop::*;
        match self {
            BoneAngle | BoneScale | BoneOffsetX | BoneOffsetY => TargetKind::Bone,
            LayerOpacity | LayerX | LayerY | LayerRotation | LayerScaleX | LayerScaleY | LayerSkew
            | LayerSwitch | LayerWobble => TargetKind::Layer,
            CameraX | CameraY | CameraZoom | CameraRotation | CameraFocus | CameraBlur => TargetKind::Camera,
            IkMix => TargetKind::Ik,
            ConstraintWeight => TargetKind::Constraint,
            PathMorph | PointDx | PointDy | LatticeDx | LatticeDy | SpineDx | SpineDy | PinDx | PinDy | CageDx | CageDy | FxBlur | FxDx | FxDy | FxSoft | FxAmount | GradDx | GradDy => TargetKind::Sub,
        }
    }

    pub fn parse(s: &str) -> Option<Prop> {
        serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
    }
}

/// How a channel's value combines with what is already there.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Replace,
    /// Add on top (procedural noise over hand-keyed animation, additive poses…).
    Add,
    Multiply,
}

/// Where a channel's value comes from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum Source {
    /// Hand-keyed curve.
    Keys { track: Track },
    /// Expression (see `expr` module), evaluated per frame.
    Expr { expr: String },
    /// Fractal value noise in `[-amp, amp]`; `freq` in cycles per second.
    Noise {
        amp: f32,
        freq: f32,
        #[serde(default)]
        seed: u32,
        #[serde(default = "one_u")]
        octaves: u32,
    },
    /// Driven key: the curve maps another value (`input`/`input_prop`) to this one.
    /// The curve's "frames" are input values (e.g. elbow angle → bulge amount).
    Driven { input: String, input_prop: Prop, curve: Track },
}

fn one_u() -> u32 {
    1
}

/// One animated value. Channels run in list order; later ones see earlier results
/// (via `get(...)` in expressions and driven keys).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    /// Bone / layer / IK / constraint name, or a `layer::name::sub` path (ignored for camera props).
    #[serde(default)]
    pub target: String,
    pub prop: Prop,
    #[serde(flatten)]
    pub source: Source,
    #[serde(default)]
    pub mode: Mode,
    /// Influence 0..1 (blend toward the channel's result).
    #[serde(default = "one_f")]
    pub weight: f32,
}

impl Channel {
    pub fn keys(target: &str, prop: Prop, track: Track) -> Channel {
        Channel { target: target.into(), prop, source: Source::Keys { track }, mode: Mode::Replace, weight: 1.0 }
    }
    pub fn expr(target: &str, prop: Prop, expr: &str, mode: Mode) -> Channel {
        Channel { target: target.into(), prop, source: Source::Expr { expr: expr.into() }, mode, weight: 1.0 }
    }
    pub fn noise(target: &str, prop: Prop, amp: f32, freq: f32, seed: u32) -> Channel {
        Channel {
            target: target.into(),
            prop,
            source: Source::Noise { amp, freq, seed, octaves: 2 },
            mode: Mode::Add,
            weight: 1.0,
        }
    }
    pub fn driven(target: &str, prop: Prop, input: &str, input_prop: Prop, curve: Track) -> Channel {
        Channel {
            target: target.into(),
            prop,
            source: Source::Driven { input: input.into(), input_prop, curve },
            mode: Mode::Replace,
            weight: 1.0,
        }
    }
    /// Frame-by-frame exposure sheet: `(frame, drawing index)` pairs held with step interpolation.
    pub fn exposure(target: &str, exposures: &[(f32, usize)]) -> Channel {
        let keys: Vec<(f32, f32, Ease)> = exposures.iter().map(|&(f, i)| (f, i as f32, Ease::Step)).collect();
        Channel::keys(target, Prop::LayerSwitch, Track::with_keys(&keys))
    }
    pub fn with_mode(mut self, m: Mode) -> Self {
        self.mode = m;
        self
    }
    pub fn with_weight(mut self, w: f32) -> Self {
        self.weight = w;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Point of the stage shown at the screen centre; `None` = stage centre.
    #[serde(default)]
    pub position: Option<Vec2>,
    #[serde(default = "one_f")]
    pub zoom: f32,
    /// Degrees.
    #[serde(default)]
    pub rotation: f32,
    /// Depth of field: the layer `parallax` depth that is in focus (animate with `camera_focus`).
    #[serde(default = "one_f", skip_serializing_if = "is_default_focus")]
    pub focus: f32,
    /// Depth of field: blur radius (output pixels) per unit of parallax distance (`camera_blur`).
    #[serde(default = "default_dof_blur", skip_serializing_if = "is_default_blur")]
    pub blur: f32,
}

fn default_dof_blur() -> f32 {
    12.0
}
fn is_default_focus(v: &f32) -> bool {
    *v == 1.0
}
fn is_default_blur(v: &f32) -> bool {
    *v == 12.0
}

impl Default for Camera {
    fn default() -> Self {
        Camera { position: None, zoom: 1.0, rotation: 0.0, focus: 1.0, blur: 12.0 }
    }
}

/// The space the raster (`render`, GIF, PNG, video frames) and float (`render-hdr`) compositors blend in.
///
/// * `Srgb` (default): colours are blended as stored, i.e. in sRGB-encoded space, which is what most 2-D tools do and
///   what every golden of this project was recorded with.
/// * `Linear`: every input colour (fills, strokes, gradient stops, the background, image pixels) is converted to linear
///   light, all blending, opacity, masks and blur happen there in f32, and the result is converted back to sRGB once at
///   the end. Gradients interpolate in linear light, a 50 % black over white is the sRGB value 188 (not 128), and
///   half-transparent edges and blurs stay physically plausible. SVG output, Lottie and the GPU renderer are not affected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorSpace {
    #[default]
    Srgb,
    Linear,
}

impl ColorSpace {
    pub fn is_default(&self) -> bool {
        *self == ColorSpace::Srgb
    }

    /// `srgb` or `linear` (case-insensitive).
    pub fn parse(s: &str) -> Result<ColorSpace, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "srgb" => Ok(ColorSpace::Srgb),
            "linear" => Ok(ColorSpace::Linear),
            other => Err(format!("unknown colour space '{other}' (srgb or linear)")),
        }
    }
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// File-format version; files without it are treated as version 1.
    #[serde(default = "format_version")]
    pub format: u32,
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub fps: f32,
    /// Number of frames (frame indices run `0..frames`).
    pub frames: u32,
    #[serde(default)]
    pub background: Option<Rgba>,
    /// Space in which the raster and float renderers composite (default: sRGB-encoded values, the historical
    /// behaviour). `linear` blends in linear light; see [`ColorSpace`].
    #[serde(default, skip_serializing_if = "ColorSpace::is_default")]
    pub color_space: ColorSpace,
    #[serde(default)]
    pub camera: Camera,
    #[serde(default)]
    pub bones: Vec<Bone>,
    #[serde(default)]
    pub ik: Vec<IkChain>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    #[serde(default)]
    pub springs: Vec<Spring>,
    #[serde(default)]
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub channels: Vec<Channel>,
    /// Timeline markers and regions (metadata only; never rendered).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<crate::keyops::Marker>,
    /// Named layer states ("comps").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comps: Vec<crate::comps::Comp>,
    /// Named cues for the host application (sound, gameplay hooks…).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<crate::comps::Event>,
    /// Bone overlay visibility ranges (editor aid).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bone_vis: Vec<crate::bonevis::BoneVis>,
    /// Audio tracks (references only) mixed into video exports.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sounds: Vec<crate::soundtrack::SoundTrack>,
    /// Skin sets: named groups of layers shown together (outfits, expressions, damage states).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skins: Vec<crate::skins::Skin>,
    /// Per-channel colour tag, shy/solo/lock flags (editor metadata).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub channel_meta: Vec<crate::chanmeta::ChannelMeta>,
    /// Saved camera positions ("shots").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shots: Vec<crate::camshots::Shot>,
    /// Layered clip stack (takes placed on the timeline); see [`crate::clipstack`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clips: Vec<crate::clipstack::Clip>,
    /// Channels generated by the last clip bake (exact snapshot; re-baking removes only these).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clip_baked: Vec<Channel>,
    /// Mesh-to-mesh locks: vertices of one layer follow the surface of another (`links.rs`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mesh_locks: Vec<crate::links::MeshLock>,
    /// Glue: vertices of two layers/paths that move together (`links.rs`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glues: Vec<crate::links::Glue>,
    /// Runtime state machines for interactive playback; see [`crate::machine`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machines: Vec<crate::machine::Machine>,
    /// Data bindings: external values drive properties / machine inputs; see [`crate::binding`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<crate::binding::Binding>,
    /// Layout containers (row / column distribution with a canvas-relative rectangle); see [`crate::containers`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub containers: Vec<crate::containers::Container>,
    /// Scroll regions over a content container; see [`crate::containers`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scrolls: Vec<crate::containers::Scroll>,
    /// Record of the level of detail this project was derived at; see [`crate::lod`]. Empty for authored projects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lods: Vec<crate::lod::LodRecord>,
}

/// Current project file-format version. Bump when a change is not backward compatible and add a
/// migration step to [`Project::from_json`].
pub const FORMAT_VERSION: u32 = 1;

fn format_version() -> u32 {
    FORMAT_VERSION
}

/// A bone as a `(head, tail)` segment.
type Segment = (Vec2, Vec2);

impl Project {
    pub fn new(name: &str, width: f32, height: f32, fps: f32, frames: u32) -> Project {
        Project {
            format: FORMAT_VERSION,
            name: name.into(),
            width,
            height,
            fps,
            frames,
            background: None,
            color_space: ColorSpace::Srgb,
            camera: Camera::default(),
            bones: vec![],
            ik: vec![],
            constraints: vec![],
            springs: vec![],
            layers: vec![],
            channels: vec![],
            markers: vec![],
            comps: vec![],
            shots: vec![],
            clips: vec![],
            clip_baked: vec![],
            skins: vec![],
            bone_vis: vec![],
            sounds: vec![],
            channel_meta: vec![],
            events: vec![],
            mesh_locks: vec![],
            glues: vec![],
            machines: vec![],
            bindings: vec![],
            containers: vec![],
            scrolls: vec![],
            lods: vec![],
        }
    }

    pub fn from_json(s: &str) -> Result<Project, serde_json::Error> {
        use serde::de::Error;
        let v: serde_json::Value = serde_json::from_str(s)?;
        if let Some(f) = v.get("format").and_then(|f| f.as_u64()) {
            if f > FORMAT_VERSION as u64 {
                return Err(serde_json::Error::custom(format!(
                    "project format {f} is newer than this build supports ({FORMAT_VERSION}); update OSATURA"
                )));
            }
        }
        crate::engine::check_json_magnitudes(&v).map_err(serde_json::Error::custom)?;
        serde_json::from_value(v)
    }
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("project serialises")
    }

    /// Find a layer by its `/`-joined path.
    pub fn find_layer(&self, path: &str) -> Option<&Layer> {
        let mut layers: &[Layer] = &self.layers;
        let mut found: Option<&Layer> = None;
        for part in path.split('/') {
            let l = layers.iter().find(|l| l.name == part)?;
            layers = l.children();
            found = Some(l);
        }
        found
    }

    /// True when the layer at `path` or any group above it has its `locked` flag set. Unknown paths are not locked.
    pub fn is_layer_locked(&self, path: &str) -> bool {
        let mut layers: &[Layer] = &self.layers;
        for part in path.split('/') {
            let Some(l) = layers.iter().find(|l| l.name == part) else { return false };
            if l.locked {
                return true;
            }
            layers = l.children();
        }
        false
    }

    pub fn find_layer_mut(&mut self, path: &str) -> Option<&mut Layer> {
        let mut layers: &mut Vec<Layer> = &mut self.layers;
        let parts: Vec<&str> = path.split('/').collect();
        for (i, part) in parts.iter().enumerate() {
            let idx = layers.iter().position(|l| l.name == *part)?;
            if i == parts.len() - 1 {
                return Some(&mut layers[idx]);
            }
            layers = layers[idx].children_mut()?;
        }
        None
    }

    /// All layer paths in preorder.
    pub fn layer_paths(&self) -> Vec<String> {
        fn walk(ls: &[Layer], prefix: &str, out: &mut Vec<String>) {
            for l in ls {
                let p = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
                out.push(p.clone());
                walk(l.children(), &p, out);
            }
        }
        let mut out = Vec::new();
        walk(&self.layers, "", &mut out);
        out
    }

    /// World transforms of all bones in the rest pose.
    pub fn rest_world(&self) -> Result<Vec<Affine2>, String> {
        let parent = resolve_parents(&self.bones)?;
        let poses = vec![BonePose::default(); self.bones.len()];
        Ok(fk(&self.bones, &parent, &poses))
    }

    fn bone_segments(&self) -> Result<(Vec<Segment>, Vec<String>), String> {
        let rest = self.rest_world()?;
        let segs = self
            .bones
            .iter()
            .zip(&rest)
            .map(|(b, w)| (w.apply(Vec2::ZERO), w.apply(Vec2::new(b.length, 0.0))))
            .collect();
        Ok((segs, self.bones.iter().map(|b| b.name.clone()).collect()))
    }

    /// Bind every anchor of every path in `layer` (and its descendants) to its
    /// nearest bones (inverse-distance weights, at most `max_influences` per
    /// anchor), measured in the rest pose. Image layers with a mesh are skinned too.
    pub fn auto_skin(&mut self, layer: &str, max_influences: usize) -> Result<(), String> {
        let (segs, names) = self.bone_segments()?;
        let layer = self.find_layer_mut(layer).ok_or_else(|| format!("no layer at '{layer}'"))?;
        skin_layer(layer, &segs, &names, max_influences.max(1));
        Ok(())
    }
}

impl Project {
    /// Skin the mesh of an image layer by harmonic diffusion on the mesh graph
    /// (see `weights`): smooth, partition of unity, no leaking across gaps.
    /// Needs the layer's `mesh` and displayed `size`.
    pub fn auto_skin_diffuse(&mut self, layer: &str, max_influences: usize, opts: crate::weights::DiffusionOptions) -> Result<(), String> {
        let (segs, names) = self.bone_segments()?;
        let l = self.find_layer_mut(layer).ok_or_else(|| format!("no layer at '{layer}'"))?;
        let LayerKind::Image { origin, size, mesh, .. } = &mut l.kind else {
            return Err(format!("layer '{layer}' is not an image layer"));
        };
        let m = mesh.as_mut().ok_or_else(|| format!("layer '{layer}' has no mesh"))?;
        let sz = size.ok_or_else(|| format!("layer '{layer}' needs an explicit size"))?;
        let verts = m.rest_vertices(*origin, sz);
        let tris = m.tri_indices();
        let dense = crate::weights::diffusion_weights(&verts, &tris, &segs, opts);
        m.weights = crate::weights::to_bone_weights(&dense, &names, max_influences.max(1));
        Ok(())
    }
}

/// Inverse-distance weights of `p` against the bone segments (top `k`, normalised).
fn weights_for(p: Vec2, segs: &[(Vec2, Vec2)], names: &[String], k: usize) -> Vec<BoneWeight> {
    let mut cand: Vec<(usize, f32)> =
        segs.iter().enumerate().map(|(i, &(a, b))| (i, dist_to_segment(p, a, b))).collect();
    cand.sort_by(|x, y| x.1.total_cmp(&y.1));
    cand.truncate(k);
    let ws: Vec<f32> = cand.iter().map(|&(_, d)| 1.0 / ((d + 6.0) * (d + 6.0))).collect();
    let sum: f32 = ws.iter().sum();
    cand.iter().zip(&ws).map(|(&(i, _), &w)| BoneWeight { bone: names[i].clone(), weight: w / sum }).collect()
}

fn skin_layer(l: &mut Layer, segs: &[(Vec2, Vec2)], names: &[String], k: usize) {
    match &mut l.kind {
        LayerKind::Vector { paths } => {
            for path in paths {
                for anchor in &mut path.anchors {
                    anchor.weights = weights_for(anchor.pos, segs, names, k);
                }
            }
        }
        LayerKind::Group { layers } | LayerKind::Selector { layers, .. } => {
            for c in layers {
                skin_layer(c, segs, names, k);
            }
        }
        LayerKind::Image { origin, size, mesh, .. } => {
            if let Some(m) = mesh {
                // Vertex positions need the displayed size; without it (pixel size unknown
                // here) fall back to a 1:1 pixel grid only when `size` is given.
                let Some(sz) = *size else { return };
                m.weights = m.rest_vertices(*origin, sz)
                    .into_iter()
                    .map(|p| weights_for(p, segs, names, k))
                    .collect();
            }
        }
        LayerKind::Metaball { .. } | LayerKind::Boolean { .. } | LayerKind::Emitter { .. } => {}
    }
}

/// Row-major grid vertices of a `cols × rows` mesh over the rect `origin .. origin+size`.
pub fn mesh_vertices(origin: Vec2, size: Vec2, cols: usize, rows: usize) -> Vec<Vec2> {
    let (cols, rows) = (cols.max(1), rows.max(1));
    let mut v = Vec::with_capacity((cols + 1) * (rows + 1));
    for r in 0..=rows {
        for c in 0..=cols {
            v.push(origin + Vec2::new(size.x * c as f32 / cols as f32, size.y * r as f32 / rows as f32));
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_and_liquid_keep_their_file_names_and_the_new_names_are_read_aliases() {
        let sw = Layer::switch("s", vec![]);
        let lq = Layer::liquid("l", vec![], Paint::solid("#ffffff"));
        let (a, b) = (serde_json::to_value(&sw).unwrap(), serde_json::to_value(&lq).unwrap());
        assert_eq!(a["type"], "switch");
        assert_eq!(b["type"], "liquid");
        for (alias, want) in [("selector", &sw), ("metaball", &lq)] {
            let mut v = if alias == "selector" { a.clone() } else { b.clone() };
            v["type"] = alias.into();
            let back: Layer = serde_json::from_value(v).unwrap();
            assert_eq!(&back, want, "{alias}");
        }
    }

    #[test]
    fn layer_json_is_flat_and_round_trips() {
        let l = Layer::group("g", vec![Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 1.0, 1.0)])]);
        let json = serde_json::to_string(&l).unwrap();
        assert!(json.contains("\"type\":\"group\""), "{json}");
        let back: Layer = serde_json::from_str(&json).unwrap();
        assert_eq!(l, back);
    }

    #[test]
    fn channel_json_round_trips_for_every_source() {
        let chans = vec![
            Channel::keys("a", Prop::BoneAngle, Track::with_keys(&[(0.0, 1.0, Ease::Linear)])),
            Channel::expr("a", Prop::BoneScale, "1 + 0.1 * sin(time)", Mode::Replace),
            Channel::noise("a", Prop::BoneOffsetX, 3.0, 2.0, 9),
            Channel::driven("a", Prop::BoneOffsetY, "b", Prop::BoneAngle, Track::new()),
            Channel::exposure("m", &[(0.0, 0), (4.0, 2)]),
        ];
        let json = serde_json::to_string(&chans).unwrap();
        let back: Vec<Channel> = serde_json::from_str(&json).unwrap();
        assert_eq!(chans, back);
        assert!(json.contains("\"source\":\"noise\""));
    }

    #[test]
    fn rig_extras_round_trip() {
        let mut p = Project::new("t", 10.0, 10.0, 24.0, 1);
        p.bones = vec![Bone::new("a", None, 1.0, Vec2::ZERO, 0.0).with_angle_limits(-10.0, 90.0)];
        p.ik = vec![IkChain::chain("tail", &["a", "b", "c"], "t")];
        p.constraints = vec![
            Constraint { name: "aim".into(), bone: "a".into(), kind: ConstraintKind::Aim { target: "t".into(), offset: 5.0 }, weight: 0.5 },
            Constraint { name: "cp".into(), bone: "a".into(), kind: ConstraintKind::CopyRotation { source: "b".into() }, weight: 1.0 },
            Constraint { name: "pin".into(), bone: "a".into(), kind: ConstraintKind::Pin { target: "t".into() }, weight: 1.0 },
        ];
        p.springs = vec![Spring { bone: "a".into(), frequency: 2.0, damping: 0.4, warmup: true }];
        let mut l = Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 1.0, 1.0).with_morph("m", |a| a.clone())]);
        l.deformers = vec![
            Deformer::Lattice { name: "lat".into(), cols: 2, rows: 2, rect: [0.0, 0.0, 1.0, 1.0], offsets: vec![] },
            Deformer::Spine { name: "sp".into(), rest: vec![Vec2::ZERO, Vec2::new(1.0, 0.0)], offsets: vec![], radius: Some(5.0) },
        ];
        l.wobble = Some(Wobble { amp: 1.0, scale: 0.05, rate: 8.0, seed: 3 });
        p.layers = vec![l, Layer::image_from_png("img", &[1, 2, 3], Vec2::ZERO)];
        let back = Project::from_json(&p.to_json_pretty()).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn find_layer_by_path() {
        let mut p = Project::new("t", 10.0, 10.0, 24.0, 1);
        p.layers = vec![Layer::group("g", vec![Layer::vector("v", vec![])])];
        assert!(p.find_layer("g/v").is_some());
        assert!(p.find_layer("g/x").is_none());
        assert!(p.find_layer_mut("g/v").is_some());
        assert_eq!(p.layer_paths(), vec!["g".to_string(), "g/v".to_string()]);
    }

    #[test]
    fn transform_matrix_order() {
        let t = Transform2D { position: Vec2::new(10.0, 0.0), scale: Vec2::new(2.0, 2.0), anchor: Vec2::new(1.0, 0.0), ..Default::default() };
        let m = t.matrix();
        assert!(m.apply(Vec2::new(1.0, 0.0)).distance(Vec2::new(10.0, 0.0)) < 1e-5);
        assert!(m.apply(Vec2::new(2.0, 0.0)).distance(Vec2::new(12.0, 0.0)) < 1e-5);
    }

    #[test]
    fn prop_parse_and_kinds() {
        assert_eq!(Prop::parse("bone_angle"), Some(Prop::BoneAngle));
        assert_eq!(Prop::parse("camera_zoom"), Some(Prop::CameraZoom));
        assert_eq!(Prop::parse("lattice_dx"), Some(Prop::LatticeDx));
        assert_eq!(Prop::parse("nope"), None);
        assert_eq!(Prop::PathMorph.target_kind(), TargetKind::Sub);
        assert_eq!(Prop::IkMix.target_kind(), TargetKind::Ik);
    }

    #[test]
    fn frame_by_frame_builds_a_switch_of_drawings() {
        let l = Layer::frame_by_frame("walk", vec![vec![], vec![], vec![]]);
        assert_eq!(l.children().len(), 3);
        assert_eq!(l.children()[2].name, "d2");
    }

    #[test]
    fn mesh_vertices_grid() {
        let v = mesh_vertices(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0), 2, 1);
        assert_eq!(v.len(), 6);
        assert_eq!(v[0], Vec2::new(10.0, 20.0));
        assert_eq!(v[5], Vec2::new(110.0, 70.0));
    }

    #[test]
    fn layer_lock_flag_is_saved_only_when_set_and_old_projects_load() {
        let mut l = Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 1.0, 1.0)]);
        assert!(!serde_json::to_string(&l).unwrap().contains("locked"), "default is not written");
        let old: Layer = serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
        assert!(!old.locked);
        l.locked = true;
        let j = serde_json::to_string(&l).unwrap();
        assert!(j.contains("\"locked\":true"), "{j}");
        assert!(serde_json::from_str::<Layer>(&j).unwrap().locked);
        let explicit: Layer = serde_json::from_str(&j.replace("true", "false")).unwrap();
        assert!(!explicit.locked);
    }

    #[test]
    fn a_locked_group_locks_everything_below_it() {
        let mut p = Project::new("t", 10.0, 10.0, 24.0, 5);
        p.layers = vec![Layer::group("g", vec![Layer::vector("c", vec![]), Layer::group("h", vec![Layer::vector("d", vec![])])]), Layer::vector("o", vec![])];
        assert!(!p.is_layer_locked("g/c") && !p.is_layer_locked("nope"));
        p.find_layer_mut("g").unwrap().locked = true;
        assert!(p.is_layer_locked("g") && p.is_layer_locked("g/c") && p.is_layer_locked("g/h/d"));
        assert!(!p.is_layer_locked("o"));
        p.find_layer_mut("g").unwrap().locked = false;
        p.find_layer_mut("g/h").unwrap().locked = true;
        assert!(!p.is_layer_locked("g/c") && p.is_layer_locked("g/h/d"));
    }
}
