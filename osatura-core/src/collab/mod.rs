// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Real-time multi-user editing: a convergent replica of the project JSON tree (R-111).
//!
//! No networking and no clock in here: everything is a pure function of the envelopes a replica has
//! seen, so it can be tested with random delivery orders. Own design from textbook ideas, no code
//! from any other project.
//!
//! Sources of the techniques (all published, none proprietary):
//! * L. Lamport, "Time, Clocks, and the Ordering of Events in a Distributed System", CACM 21(7), 1978:
//!   logical clocks and the happened-before order (used for the `lamport` stamp of every envelope).
//! * C. Fidge 1988 / F. Mattern 1989: vector clocks (`VClock`, causal delivery, duplicate detection).
//! * M. Shapiro, N. Preguica, C. Baquero, M. Zawirski, "Conflict-free Replicated Data Types", SSS 2011
//!   (and INRIA RR-7687, "A comprehensive study of Convergent and Commutative Replicated Data Types"):
//!   last-writer-wins register, tombstones, operation-based CRDT with causal delivery.
//! * H.-G. Roh et al., "Replicated abstract data types: building blocks for collaborative
//!   applications", J. Parallel Distrib. Comput. 71(3), 2011: the replicated growable array (RGA) rule
//!   for placing a concurrent insert into a sequence.
//!
//! # Data model
//! The project JSON is a tree of objects, arrays and scalars. Every object and array gets a stable
//! identity (`Id`), every array element too. An edit is expressed on identities, never on indices:
//! * object key  = LWW register (`Reg`): the write with the larger `Stamp (lamport, replica, index)` wins.
//!   A key delete is a write of `Absent` (delete and set are ordered by stamp, neither "wins").
//! * array       = RGA sequence of elements with tombstones; an insert names the element it goes
//!   after (`None` = head), concurrent inserts after the same element are ordered by `(lamport, id)`
//!   descending; a remove only sets a tombstone, so it REMOVES WINS over any concurrent edit
//!   inside the element (the edit is applied to the hidden element and is invisible), and an
//!   element is never resurrected (an undo of a remove inserts a fresh copy).
//! * array element value = LWW register.
//!
//! A local edit (the existing `Op::{Set, Insert, Remove, Batch}` on RFC 6901 pointers) is translated
//! against the replica's current tree: the pointer's indices become element identities (so a
//! stale index from a user's view is "transformed" by construction), and a `Set` of a whole
//! object/array is turned into a minimal structural diff (per key; arrays by longest common
//! subsequence) so two users editing different keys of the same channel both survive. If the
//! old and new values are unrelated (different scalars, different type, objects with different
//! `id`/`name`/`target`/`prop`) the register is replaced as a whole (LWW).
//!
//! # Delivery
//! `Envelope { replica, seq, lamport, deps, ops }`: `seq` counts that replica's envelopes from 1,
//! `deps` is the sender's vector clock when it created the envelope. `Replica::receive` holds back an
//! envelope until its predecessors from the same replica and all `deps` are applied (causal
//! delivery), drops duplicates by `seq` (idempotent), and applies the rest. Given causal delivery, every
//! replica that has applied the same set of envelopes holds a byte-identical `Doc` (tested with
//! seeded random delivery orders).
//!
//! # Invalid concurrent edits
//! An op that names an unknown identity or the wrong container kind is dropped on application (counted
//! in `dropped`), never a panic. Edits to a removed element or to a node that was replaced by an
//! LWW overwrite are applied to the (now invisible) node, so all replicas agree and the edit simply does
//! not show. Whether the merged JSON is a valid OSATURA project is NOT part of this module (a
//! server decides; see `collabhub`).
//!
//! # Per-user undo
//! `Replica::undo` reverses only this replica's own most recent edit, as new envelopes: an insert is
//! undone by removing that element (if still there), a remove by inserting a copy next to its old left
//! neighbour, a register write by writing the previous value back, but only if the register still holds
//! this edit's write (if someone wrote after, theirs stays: the undo of that part is skipped).
//!
//! # Compaction
//! `Replica::compact` drops log entries every peer has, and, only when all peers have exactly this
//! replica's clock (quiescent, nothing in flight), purges tombstones and unreachable nodes.
use crate::edit::Op;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

mod identities;
mod translation;
mod replica;
mod presence;

pub use identities::*;
use translation::*;
pub use replica::*;
pub use presence::*;

#[cfg(test)]
mod tests;
