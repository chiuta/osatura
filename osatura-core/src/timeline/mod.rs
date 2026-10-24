// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Timeline / dope-sheet model and key-editing operations.
//!
//! * [`build`] turns a project into a [`TimelineModel`]: one track per animation channel
//!   (`tracks[i].ch == i`), grouped hierarchically (kind -> bone parent chain / layer path /
//!   deformer path), with the channel metadata (lock, shy, solo, colour) from `chanmeta`.
//! * Keys are addressed by [`KeyRef`] = (channel index, exact frame). Selection helpers are pure
//!   functions on `Vec<KeyRef>` (box over rows and time, toggle, by channel / group).
//! * Every edit is planned by [`plan`] from a [`TlOp`] and returns an [`EditPlan`] holding one
//!   `Op::Batch` for [`crate::edit::Editor::apply`], so it is one undo step and is validated by the
//!   editor like any other edit. Planning never mutates the project; any error (locked channel,
//!   frame outside the project, collision under `Refuse`, stale key) refuses the whole op.
//!
//! Clean-room note: generic dope-sheet maths (translate / scale about a pivot / clip copy and
//! paste with channel remap / ripple insert and delete / loop closing). The data model and the
//! operation set are this project's own; nothing follows any product's behaviour, naming or
//! shortcuts.
use crate::anim::{Ease, Extrapolate, Key, Track};
use crate::chanmeta;
use crate::edit::Op;
use crate::project::{Channel, Mode, Project, Prop, Source, TargetKind};
use crate::timeline_graph::{self as g, HandleMode, HandlePos, Side, TangentKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod model;
mod keys;
mod edits;

pub use model::*;
pub use keys::*;
pub use edits::*;

#[cfg(test)]
mod tests;
