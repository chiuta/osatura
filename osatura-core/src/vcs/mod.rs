// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Version control for animation projects with semantic merge ("OsaGit", I-01).
//!
//! Own design; no code, format or command implementation of any other version-control system is
//! used (and nothing here shells out to one). Textbook ideas only:
//! * content-addressed store and Merkle DAG (R. Merkle 1979/1987): an object is named by the SHA-256
//!   (FIPS 180-4, implemented below and tested against the published vectors) of its bytes; a tree
//!   names its parts by hash and a commit names its tree and parents by hash, so one hash fixes
//!   the whole history below it;
//! * lowest common ancestors on the commit DAG for the merge base;
//! * the semantic diff and three-way merge of [`crate::semdiff`] (diff3, Khanna/Kunal/Pierce 2007).
//!
//! # Repository layout (`<project dir>/.osatura-vcs/`)
//! * `objects/ab/cdef...` one file per object: header `ARMOBJ1 <kind> <payload length>\n` then the
//!   payload; kinds `blob` (canonical JSON of one project part), `tree` (`{"parts": {name: hash}}`),
//!   `commit` (`{"tree","parents","author","time","message"}`);
//! * `refs/heads/<branch>`, `refs/tags/<tag>`: a 64-hex hash and a newline;
//! * `HEAD`: `ref: refs/heads/<branch>` or a bare hash (detached);
//! * `config`: JSON `{"project": "<file>", "author": "<name>"}`;
//! * `MERGE_STATE`: JSON, present while a merge / cherry-pick / revert has unresolved conflicts or
//!   is waiting for `commit`.
//!
//! # Trees
//! The project document is split into semantic parts so that unchanged parts are stored once:
//! `meta` (all top-level scalars), `obj/<key>` (top-level objects such as `camera`),
//! `order/<key>` + `arr/<key>/<id>` (each element of a top-level array with identities: every layer,
//! bone, channel, skin, sound ... is one part, the order list names them), `val/<key>` (arrays
//! without identities). Parts are canonical JSON (sorted keys, integral floats as integers).
//!
//! # NOT done
//! No remote, push or pull; no packing or compression; binary assets referenced by a project
//! (images, audio) are not stored, only their path strings; merge is semantic only for the JSON
//! structure (see `semdiff`), so a renamed layer is a remove plus an add; a criss-cross history
//! (several best common ancestors) uses the newest one instead of a recursive virtual base; the
//! store does no locking, so two simultaneous writers on one repository are not supported.
use crate::semdiff::{self, Change, Choice, Conflict};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

mod repo;
mod repo_ops;

pub use repo::*;

#[cfg(test)]
mod tests;
