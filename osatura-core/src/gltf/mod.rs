// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! glTF 2.0 binary (.glb) export: the bone hierarchy with sampled animation, plus the
//! frame-0 vector shapes as triangle meshes. Only core glTF 2.0 features are used.
//!
//! What is exported, stated plainly:
//! * Bones and their animation are exact (sampled every frame, linear interpolation, shear dropped).
//! * Fills become flat triangle meshes (ear clipping). Shapes are *rigid*: a layer with a `bone`
//!   follows that bone node; other shapes stay put. Shapes whose anchors all carry bone weights are
//!   skinned meshes (JOINTS_0/WEIGHTS_0, 4 influences per vertex, weights interpolated along each
//!   segment). Layer transform animation (non-bone) and opacity animation are not exported.
//! * Strokes are triangulated into their own meshes (width, width profile, butt/round/square caps,
//!   miter/round/bevel joins with a miter limit of 4, dashes). Stroke vertices inherit the bone
//!   weights of the path point they were offset from. Translucent strokes can look darker where
//!   the generated triangles overlap (sharp inner corners, round joins), because glTF blends each
//!   triangle on its own.
//! * Gradients are exact for linear and radial paints: glTF has no gradient, so a small generated
//!   PNG (256x1 for linear, 256x256 for radial) is used as a base-colour texture with TEXCOORD_0
//!   computed from the layer-space position, so colours interpolate correctly across large
//!   triangles. Stops are interpolated in sRGB, straight alpha. Spread mode is pad (clamp).
//! * Layer masks (own or inherited from a group) are baked by clipping the triangles against the
//!   mask layer's triangulated fill, exactly, but only when the mask source is a vector layer with
//!   opaque fills, no strokes and full opacity, and only for the frame-0 mask shape (an animated
//!   mask does not move in the file). Clipped layers are exported rigid, not skinned.
//! * Image layers become a textured quad or the deformation mesh (embedded PNG, skinned when the
//!   mesh has complete weights). Masks on meshed images are not baked.
//! * Path morphs become glTF morph targets with animated weights, but only after a per-frame check
//!   that `base + sum(weight * delta)` reproduces the engine's geometry (same topology, fixed
//!   subdivision per segment) and the triangulation stays valid; otherwise the path stays a static
//!   frame-0 shape and the reason is returned in the report.
//!
//! The stage's y axis points down, so y is negated (glTF is y-up).
use crate::engine::{Frame, NodeKind};
use crate::hdr::srgb_to_linear;
use crate::math::{Affine2, Vec2};
use crate::path::{profile_at, VPath};
use crate::project::{Layer, LayerKind};
use crate::style::{Cap, Join, Paint, Rgba, StrokeStyle};
use crate::Engine;
use serde_json::{json, Value};
use std::collections::HashMap;

mod geometry;
mod masks;
mod export;

pub use geometry::*;
use masks::*;
pub use export::*;

#[cfg(test)]
mod tests;
