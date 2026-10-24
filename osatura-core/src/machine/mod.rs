// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Runtime state machines for interactive playback.
//!
//! (Not to be confused with [`crate::statemachine`], which turns an input *event log* into baked
//! channels offline. This module is the live counterpart: a host feeds inputs and a time step and
//! reads back the evaluated frame.)
//!
//! Own design, textbook ingredients only (finite-state machine, 1-D blend tree, one easing
//! crossfade, normalised-weight averaging that equals the clip stack's replace-blend maths):
//!
//! * **Inputs** are named `bool`, `number` or `trigger` values. A trigger is a one-shot: it stays
//!   set until a transition that tests it fires (it is then consumed) or, unless `hold` is set,
//!   until the end of the `step` call after it was fired.
//! * **States** play a source: a clip of `Project.clips`, a frame `segment` of the project's own
//!   keyed channels, a marker `region`, or a 1-D `blend` of two or more of those driven by a number
//!   input. Playback is `loop`, `once` or `ping_pong` at a `speed`; `blend_in` is the default
//!   crossfade length into the state.
//! * **Transitions** go from a state (or `"*"` = any state) to a state when all conditions hold
//!   (`input op value`, AND), optionally only once the state has played to `exit_time` (0..1 of its
//!   length). The first matching transition in declaration order wins. They crossfade with
//!   `duration` seconds and an easing curve; an interrupted crossfade is frozen into a snapshot
//!   and fades on from there.
//! * **Layers** run in parallel inside one machine; each owns a disjoint set of
//!   `(target, property)` pairs (checked by validation) selected by an optional glob `mask`.
//!
//! Blending: every state yields a *pose* (target, property -> value, coverage weight). Poses are
//! mixed with `v = sum(w_i * c_i * v_i) / sum(w_i * c_i)` and coverage `sum(w_i * c_i)`, which is
//! what stacking replace-mode clips gives. The result is written as constant replace-mode channels
//! appended to the project; the engine evaluates them as usual ([`MachineRuntime::engine`]).
//!
//! Determinism: the runtime is a pure function of the call sequence (`set_*`, `fire`, `step(dt)`).
//! `step` subdivides large `dt` into pieces of at most 1/60 s, so results do not depend on how a
//! host chunks time beyond that resolution.
use crate::anim::{Ease, Track};
use crate::clipstack::glob;
use crate::engine::{Engine, Frame};
use crate::project::{Channel, Mode, Project, Prop, Source};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod model;
mod poses;
mod runtime;
mod simulation;

pub use model::*;
pub use poses::*;
pub use runtime::*;
pub use simulation::*;

#[cfg(test)]
mod tests;
