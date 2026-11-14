// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Deterministic level of detail (LOD) with a declared error bound (ROADMAP I-11).
//!
//! Given a [`Project`] and an error budget in *output pixels at a stated render scale* ("at most
//! 0.5 px at 1920x1080"), [`build`] derives levels L1..Ln (L0 is the original). Each level is an
//! ordinary [`Project`] generated on demand; the only thing stored in a level is a small
//! [`LodRecord`] (`Project::lods`) that says what was done and which bound was declared.
//!
//! What a level does, and what is *proved* about it (the assumptions are in [`ASSUMPTIONS`]):
//!
//! 1. **Path geometry** (unskinned, undeformed vector layers). Anchors are removed greedily
//!    (Visvalingam-Whyatt order: cheapest removal first) and the removed span is replaced by one
//!    cubic fitted to the original span (Schneider-style least squares plus Newton
//!    re-parametrisation, textbook). The cost of a removal is a *certified upper bound* of the
//!    Hausdorff distance between the original span and the replacement: both curves are flattened
//!    with a control-polygon bound (a cubic lies in the convex hull of its control points, so the
//!    distance to its chord is at most that of the two inner control points) and the directed
//!    Hausdorff distance of the two polylines is bounded by branch and bound with the 1-Lipschitz
//!    property of the distance function. Spans are always compared with the *original* path (no
//!    accumulation); the path bound is the maximum over its final spans. Local units are converted
//!    to pixels with the largest singular value of the layer's world matrix over the sampled
//!    animation (plus a safety margin), which is why the bound holds for all sampled frames.
//! 2. **Animation keys** (Ramer-Douglas-Peucker style key reduction with an L-infinity value
//!    bound, verified on a sub-frame grid). The value tolerance of a channel is the pixel budget
//!    divided by a *lever arm* (pixels per unit of that channel: bone angle x distance of the skinned
//!    geometry from the bone head x scale, layer translation x parent scale, ...) and by the number
//!    of reduced channels that can move the same layer, so the pixel errors add up to at most the budget.
//! 3. **Image meshes**: half-edge collapse ordered by the generalised quadric error metric
//!    (Garland and Heckbert 1997/98, bone weights as extra attributes). Every accepted collapse is
//!    *certified* against the original mesh: the deformed surfaces are piecewise linear, so the
//!    deviation is a convex function on each cell of the overlay of the two triangulations and its
//!    maximum is attained at the cell corners (found by exact convex clipping). The maximum is
//!    taken over all sampled frames.
//! 4. **Culling**: layers whose screen extent never exceeds a small diameter are removed; the
//!    removal is charged with that diameter (convention: deleting a shape = collapsing it to a point
//!    inside it). Dead layers (invisible at every frame) are dropped losslessly (`sizefit`).
//! 5. Effects, blur, post effects, depth of field, cloth/IK/spring solved geometry, deformers and
//!    skinned paths are **not** touched (no bound can be documented); see [`LIMITS`].
//!
//! Everything is deterministic: no randomness, no hash-map iteration order is observable, ties are
//! broken by index. [`verify`] renders / evaluates L0 and Ln on sampled and key frames and measures
//! the actual deviation (screen-space Hausdorff distance of the evaluated geometry, plus pixel
//! statistics) so that declared and measured bounds can be compared.

use crate::anim::Track;
use crate::engine::Engine;
use crate::math::{Affine2, Vec2};
use crate::path::{Anchor, VPath};
use crate::project::{Channel, Layer, LayerKind, Mode, Project, Prop, Source};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

/// Assumptions under which the declared bound is claimed to hold.
pub const ASSUMPTIONS: &[&str] = &[
    "bounds are for timeline playback at frames inside 0..frames; motion is sampled at SUB samples per frame plus all key frames, scale/lever maxima get a 2% margin, so excursions shorter than 1/SUB frame are not covered",
    "pixel values are stage units times the stated render scale (raster scale); the bound scales linearly with the scale, see LodRecord::bound_stage",
    "path bound = Hausdorff distance of the stroke/fill outline curve (round joins only for strokes); fill-rule topology is guarded by a discrete Frechet check, not proved",
    "only channels with a documented Jacobian are reduced: bone angle / offset of bones that no IK chain, constraint or spring touches, layer x / y / rotation, camera x / y; none may be read by an expression or driven key",
    "key errors and geometry errors are added (triangle inequality), culled layers are charged with their screen diameter",
    "mesh deviation is measured on the deformed vertices of the sampled frames (piecewise linear surface)",
];

/// What is deliberately left alone because no bound can be stated.
pub const LIMITS: &[&str] = &[
    "skinned vector paths, path morphs, per-point offsets, lattice/spine/pin/cage/cloth deformers and wobble are not simplified",
    "IK chains, constraints, springs, expressions, driven keys, noise, state machines and data bindings are not reduced or modelled",
    "opacity, scale, skew, switch, camera zoom/rotation/focus/blur channels are not reduced (no pixel Jacobian is derived)",
    "liquid, emitter and boolean layers, masks, blend modes, gradients, strokes with dashes / width profiles / miter or bevel joins are left unchanged",
    "post effects, depth of field and any filter are never downgraded",
    "projects with mesh locks or glues are not geometry-reduced",
    "culling is a per-shape criterion: many tiny shapes that together form a texture are removed independently",
];

/// Samples per frame used for the motion analysis.
pub const SUB: usize = 2;
const MARGIN: f64 = 1.02;

type P = [f64; 2];
type Cubic = [P; 4];

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}
fn add(a: P, b: P) -> P {
    [a[0] + b[0], a[1] + b[1]]
}
fn mul(a: P, s: f64) -> P {
    [a[0] * s, a[1] * s]
}
fn dot(a: P, b: P) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn norm(a: P) -> f64 {
    dot(a, a).sqrt()
}
fn dist(a: P, b: P) -> f64 {
    norm(sub(a, b))
}
fn r32(x: f64) -> f64 {
    x as f32 as f64
}
fn p_of(v: Vec2) -> P {
    [v.x as f64, v.y as f64]
}


mod distance;
mod simplify;
mod options;
mod affine;
mod structure;
mod motion;
mod keys;
mod mesh;
mod build;
mod verify;

pub use distance::*;
pub use simplify::*;
pub use options::*;
use affine::*;
pub use structure::*;
pub use motion::*;
pub use keys::*;
pub use mesh::*;
pub use build::*;
pub use verify::*;

#[cfg(test)]
mod tests;
