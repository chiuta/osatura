// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Pose suggestions learned from the project's own poses.
//!
//! Everything here is classical and small-data. No pretrained weights, no network, no deep learning: a
//! suggestion is only as good as the pose library it was learned from (a few dozen poses give a few
//! principal axes and nothing more).
//!
//! * **Pose database** ([`PoseDb`]): vectors of bone-rotation deltas (degrees) sampled at the key frames of
//!   the project's keyed `bone_angle` channels and of every take of a motion library (BVH retargets baked
//!   into a project or recorded into a library therefore count too). Each sequence keeps its order, so the
//!   database also knows which pose followed which.
//! * **Pose space** ([`PoseSpace`]): principal component analysis (covariance matrix, cyclic Jacobi
//!   eigenvalue iteration; a power iteration is provided as an independent check).
//! * **Between** ([`Suggester::between`]): an in-between pose for two key poses that stays on the pose
//!   manifold: linear blend in angle space (reference), blend in PCA space, the PCA blend pulled toward a
//!   Gaussian-kernel average of its nearest library poses, and the library pose that makes the smallest
//!   detour between the two.
//! * **Completion** ([`Suggester::complete`]): given a few fixed angles and/or tip targets (stage
//!   coordinates, evaluated with a forward-kinematics model checked against the engine), the most plausible
//!   full pose: regularised least squares in PCA space (Tikhonov / MAP with the PCA eigenvalues as prior
//!   variances), Gauss-Newton with Levenberg-Marquardt damping from several starts. For angle-only
//!   constraints the problem is linear and solved exactly. A nearest-library-pose baseline and, if a
//!   user-supplied [`Mlp`] is given, a network candidate are returned beside it.
//! * **Next** ([`Suggester::next`]): a kernel-smoothed first-order Markov / nearest-successor model: the
//!   library poses nearest to the current one vote with Gaussian weights for their successors' motion.
//!
//! Scores rank candidates and are heuristics, not probabilities: plausibility is `1 / (1 + max(0, d - s) / s)`
//! with `d` the RMS-degree distance to the nearest library pose and `s` the library's typical nearest-neighbour
//! spacing (a pose within the library's own sampling resolution counts as on the manifold).
//!
//! Applying a candidate ([`apply_op`]) returns one [`Op::Batch`], so a single undo reverts it; channels
//! that are locked ([`crate::chanmeta`]) are never written and are reported.
use crate::anim::{Ease, Track};
use crate::edit::Op;
use crate::motionlib::Library;
use crate::project::{Channel, Mode, Project, Prop, Source};
use crate::tinymlp::{self, Dense, Mlp, Rng, TrainOpts};
use serde::{Deserialize, Serialize};

mod database;
mod kinematics;
mod apply;
mod synthetic;

pub use database::*;
pub use kinematics::*;
pub use apply::*;
pub use synthetic::*;

#[cfg(test)]
mod tests;
