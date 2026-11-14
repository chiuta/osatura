// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Pose fuzzing and stress heat-maps (own design, textbook methods only).
//!
//! Goal: find, automatically, the poses and parameter combinations where a rig breaks, and show where.
//!
//! * **Parameters** ([`enumerate_params`]): every bone angle / scale / offset (within its limits, or a
//!   default range, optionally widened by a margin so limit handling is exercised), IK mix, constraint
//!   weights and deformer control-point offsets. Controller bones of drivers are ordinary bone offsets, and the
//!   range of a bone property that feeds a driven-key curve is taken from the curve's key range.
//! * **Sampling** ([`generate`]): uniform random, Latin hypercube, Halton (radical-inverse low-discrepancy
//!   sequence, first primes as bases), per-axis sweeps, extreme corners and PCA pose-space sampling
//!   (poses drawn along the principal axes of a pose database, see `posesuggest`). All are deterministic
//!   for a given seed (SplitMix64 generator).
//! * **Applying a sample**: the project is cloned once; one constant "probe" channel per parameter is
//!   prepended (channels that animated the same `(target, prop)` are dropped from the clone) and its value is
//!   rewritten in place for every sample. The saved project is never touched.
//! * **Metrics** ([`mesh_metrics`], [`Fuzzer::eval`]): per triangle the deformation gradient `F` relative to
//!   the rest pose (signed area ratio `det F` for fold detection, singular values for stretch and shear),
//!   edge length ratios, per-vertex spikes (growth of the Laplacian offset relative to rest), outline
//!   self-intersections (segment sweep, only counted beyond those already in the rest pose), IK targets
//!   that stay unreachable, parameters asked beyond their limits, non-finite numbers, layers that leave the
//!   canvas or explode. Every metric is divided by its threshold, so `1.0` always means "at the failure
//!   threshold"; the severity of a sample is the largest normalised metric.
//! * **Analysis**: per-parameter marginal failure rates, pairwise interaction scores, delta-debugging
//!   ([`ddmin`]) plus bisection ([`shrink_value`]) to a minimal failing pose, per-vertex worst-case stress
//!   aggregated over all samples, an SVG overlay, per-bone bars, a parameter-pair grid.
//!
//! Not covered (see ROADMAP): cloth penetration, spring/cloth bakes are not re-simulated per sample,
//! only one frame is probed per run, and a panic inside the engine is caught and reported but the default
//! panic message still goes to stderr.
use crate::anim::{Ease, Track};
use crate::engine::{Engine, Frame, NodeKind};
use crate::math::{Affine2, Vec2};
use crate::posesuggest::{PoseDb, PoseSpace};
use crate::project::{Channel, Deformer, Mode, Project, Prop, Source};
use serde::Serialize;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use crate::clock::Stamp;

mod random;
mod sampling;
mod snapshots;
mod delta_debugging;
mod grid;

pub use random::*;
pub use sampling::*;
pub use snapshots::*;
pub use delta_debugging::*;
pub use grid::*;

#[cfg(test)]
mod tests;
