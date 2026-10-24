// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Auto-trace: bitmap (PNG / JPEG) to vector layers, entirely local and deterministic.
//!
//! Classical, textbook methods only; there is no learned model, no pretrained data and no network:
//!
//! * colour quantisation: weighted k-means (Lloyd iterations, k-means++ seeding from a seeded
//!   xorshift generator) or median cut, over a 5-bit-per-channel histogram; images that already
//!   have at most `colors` distinct colours keep them exactly;
//! * clean-up of the label map: 3x3 majority filter and removal of connected components below
//!   `min_area` pixels (merged into the neighbour they share the most border with);
//! * 4-connected component labelling and border following on the pixel cracks. This is the
//!   border-following idea of Suzuki and Abe (1985), "Topological structural analysis of digitized
//!   binary images by border following", done on cracks between pixels instead of on pixel centres,
//!   so outer borders (positive signed area) and hole borders (negative) fall out of the orientation
//!   and the polygon area is exactly the pixel area;
//! * corner detection by the turning angle over a +-k sample window, smoothing of the open runs between
//!   corners, then least-squares cubic Bezier fitting with Newton-Raphson re-parameterisation and
//!   recursive splitting at the worst point after Schneider (1990), "An Algorithm for Automatically
//!   Fitting Digitized Curves" (Graphics Gems);
//! * optional polygon output with Douglas-Peucker (1973) or Visvalingam-Whyatt (1993) simplification;
//! * line-art mode: luminance threshold (Otsu 1979 when none is given), Zhang-Suen (1984) thinning,
//!   skeleton graph walk with spur pruning, stroke width from a 3-4 chamfer distance transform.
//!
//! Holes: a region whose hole is transparent in the source gets that hole as a cut-out (even-odd fill
//! in the SVG, a Boolean difference layer in a project). A hole that is filled by other traced regions
//! is *not* cut; the smaller region is stacked on top (areas sort the z-order), which avoids hairline
//! seams between neighbours.
//!
//! Honest limits: output quality is that of a quantise-and-contour tracer. Photographs and soft
//! gradients become flat colour bands, thin anti-aliased lines can be lost to the despeckle step, and
//! the transparent/stacked hole rule is a heuristic (majority of the hole border).
use crate::math::Vec2;
use crate::path::{Anchor, VPath};
use crate::project::{Layer, Project};
use crate::style::{Cap, Join, Paint, Rgba, StrokeStyle};
use std::collections::{BTreeMap, HashSet};

mod options;
mod label_map;
mod geometry;
mod chain_fitting;
mod line_mode;
mod project;

pub use options::*;
pub use label_map::*;
pub use geometry::*;
pub use chain_fitting::*;
pub use line_mode::*;
pub use project::*;

#[cfg(test)]
mod tests;
