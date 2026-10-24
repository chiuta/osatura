// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Motion capture (BVH) import and retargeting onto a 2D rig.
//!
//! * [`parse_bvh`]: parser written from the public BVH description (HIERARCHY with ROOT / JOINT /
//!   End Site, OFFSET, CHANNELS in any order, MOTION with `Frames:` / `Frame Time:` and one row of
//!   values per frame). Tolerant of CRLF, tabs, repeated blanks, braces glued to words and keyword
//!   case; errors carry the line number.
//! * Forward kinematics in 3D: every channel is an elementary right-handed rotation (degrees) or
//!   translation and the local transform is `T(offset) * C1 * C2 * ...` in the listed order, so
//!   `Zrotation Xrotation Yrotation` means `Rz * Rx * Ry`.
//! * [`View`] projection to the stage plane (front = XY, side = ZY, top = XZ, or a yaw about the
//!   vertical axis) and per-bone 2D angles.
//! * [`match_bones`]: automatic mapping of BVH joints to rig bones (normalised names plus synonym
//!   tables, then hierarchy topology and limb-length sanity as fallback) with a confidence table
//!   and a manual override map.
//! * [`retarget`]: rest-relative 2D angle deltas applied to the rig's bone angle channels, scaled
//!   root translation, resampling to the project frame rate, smoothing and optional foot lock.
//!   The result is a list of appended keyed channels (non-destructive, channel locks respected),
//!   which [`bake`] adds directly or [`bake_op`] expresses as one undoable editor operation.
//!
//! Limits: a 2D projection discards depth (a bone pointing at the camera has no angle and holds its
//! last one), twist about a bone's own axis is ignored, there is no finger retargeting beyond what
//! the rig names, and the rig bone keeps its own length (only angles and root travel are
//! transferred).

use crate::anim::{Ease, Track};
use crate::edit::Op;
use crate::project::{Channel, Project, Prop};
use std::collections::{BTreeMap, BTreeSet};

pub type V3 = [f64; 3];

const MAX_JOINTS: usize = 4096;
const MAX_DEPTH: usize = 512;
const MAX_FRAMES: usize = 1_000_000;
const MAX_VALUES: usize = 200_000_000;


mod bvh;
mod kinematics;
mod projection;
mod names;
mod skeleton;
mod matching;
mod retarget;

pub use bvh::*;
pub use kinematics::*;
pub use projection::*;
pub use names::*;
pub use skeleton::*;
pub use matching::*;
pub use retarget::*;

#[cfg(test)]
mod tests;
