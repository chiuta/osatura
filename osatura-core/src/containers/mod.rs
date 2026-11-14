// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Layout containers and scroll regions for responsive scenes.
//!
//! A [`Container`] arranges an ordered list of members (layers, nested containers, spacers) along
//! a main axis (row or column) inside a rectangle. The distribution maths is the usual
//! one-dimensional "flexible box" arithmetic, written here from scratch with this project's own
//! schema:
//!
//! * every member has a *basis* size (explicit `width`/`height`, else the bounding box of the
//!   layer's geometry, else the intrinsic size of a nested container) clamped by optional
//!   min / max;
//! * lines are filled greedily when `wrap` is on (otherwise there is one line);
//! * free space on a line is shared by `grow` weights, a shortfall is taken back by
//!   `shrink * basis` weights, and members that hit a min / max are frozen and the rest is
//!   re-shared (a loop that terminates after at most one pass per member);
//! * what is left over is placed by `justify` (start, center, end, space_between, space_around);
//! * across the axis a member is placed by `align` (start, center, end, stretch), per member
//!   overridable.
//!
//! The container rectangle comes from a [`RectRule`]: fixed, pinned to canvas edges with margins
//! and optional size, an aspect-ratio fit / fill of the canvas, or "hug" (the content's own
//! size). Nested containers take their rectangle from their parent's slot.
//!
//! The result is, per member layer, a translation (and, for members with `resize`, a scale
//! factor) that the engine adds to the layer's animated transform on every evaluation - a
//! non-keyed, deterministic pass, like glue in [`crate::links`]. Sizes are measured on the *rest*
//! pose (static transforms), so animation of position / rotation / scale is layered on top of
//! the slot without re-flowing.
//!
//! A [`Scroll`] puts a content container behind a viewport rectangle: the content is shifted by
//! a clamped offset, optionally with a deterministic rubber-band overscroll, and the viewport
//! can clip the content through the ordinary layer-mask mechanism (a rectangle layer named by
//! `clip_layer`, which is fitted to the viewport). [`ScrollRuntime`] simulates dragging, release,
//! momentum, snap points and the elastic return.
//!
//! Limits: sizes and slots live in the parent space of the members (stage coordinates for
//! top-level layers; camera and parallax are not considered); stretch needs an unrotated,
//! unskewed, measurable layer; measurement uses flattened outlines of vector layers and groups
//! of them (image, emitter, liquid and boolean layers need explicit `width` and `height`, and
//! then the box origin is the layer's position).
use crate::layout::{content_bounds, layer_bounds};
use crate::math::Vec2;
use crate::project::{Layer, MaskRef, Project};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod layout;
mod resolved;
mod rectangles;
mod responsive;

pub use layout::*;
pub use resolved::*;
pub use rectangles::*;
pub use responsive::*;

#[cfg(test)]
mod tests;
