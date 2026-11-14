// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Canvas transform gizmos: the maths behind drag handles and the edit plan they produce.
//!
//! A drag is described by a handle, the pointer position when it started and the pointer offset
//! since. [`drag_matrix`] turns that into one affine map `D` in canvas space (plus snapping
//! guides); [`plan`] converts `D` into property edits for every selected layer / bone / control
//! point, and [`commit`] applies them as **one** [`Op::Batch`], so one drag is one undo step.
//!
//! * translate: axis constraint (`constrain`), snap of the selection box's edges and centre lines
//!   to a grid, to other layers' bounds and centres and to the canvas, within a pixel tolerance;
//! * rotate: around the box centre (or a given pivot), `constrain` snaps the angle to 15 degrees;
//! * scale: 8 handles on a box that follows a single layer's own axes; edges scale one axis,
//!   corners both; `constrain` keeps proportions, `center` scales around the centre; dragging
//!   across the opposite edge flips (negative scale);
//! * bone ends: the head moves the bone, the tip rotates (and stretches unless `constrain`);
//! * control points: anchors, bezier handles, deformer points and free-mesh vertices.
//!
//! Auto-key: a layer / point property that already has a keyed channel is keyed at the current
//! frame; with `auto_key` any edited property is. Otherwise a layer edit changes the static
//! transform. Bone edits always key the pose (the static value would be the rest pose).
//!
//! Limits: the key value assumes the keyed channel is the replace-mode source of the property and
//! that any other contributing channel runs after it; snapping is done in canvas (camera-applied)
//! space; scale does not snap; handles of skinned or wobbling paths are edited in rest space.
use crate::anim::Ease;
use crate::edit::{Editor, Op};
use crate::engine::{Engine, Frame};
use crate::math::{Affine2, Vec2};
use crate::pick::{self, box_frame, bone_ends, BoxFrame, PointKind, Selection, Target};
use crate::project::{Channel, Deformer, LayerKind, Prop, Source, Transform2D};
use serde::{Deserialize, Serialize};
use serde_json::json;

mod handles;
mod drag_maths;
mod delete;

pub use handles::*;
pub use drag_maths::*;
pub use delete::*;

#[cfg(test)]
mod tests;
