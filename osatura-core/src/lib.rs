// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! OSATURA core — an independently implemented 2D vector rigging and animation engine.
//!
//! Everything here is implemented from scratch from publicly known techniques
//! (cubic Béziers, Catmull-Rom smoothing, forward kinematics, analytic two-bone
//! IK, linear-blend skinning, keyframe tracks with cubic-bezier easing).
//! It does not read or write any proprietary file format and shares no copied code
//! or assets with any existing animation product.
//!
//! Pipeline: [`Project`] (plain data, JSON) → [`Engine::new`] (validate + resolve
//! names) → [`Engine::evaluate`] (frame → posed, deformed geometry) →
//! [`svg::render_frame`].
//!
//! # Stability: what is and is not promised
//!
//! **Only [`api`] and [`prelude`] are the supported public API.** They follow the semver policy
//! documented in [`api`] (`API_VERSION`), return typed [`api::ApiError`] values, never panic on bad
//! input, and their exact item list is pinned by a snapshot test (`src/api_surface.txt`).
//!
//! **Everything else is unstable**: all other modules (`engine`, `project`, `edit`, `svg`, ...) and
//! the root-level re-exports (`Engine`, `Project`, `Track`, ...) are internal building blocks shared
//! with the CLI, studio and C ABI. They are `#[doc(hidden)]`, still `pub` so those programs keep
//! working, and may change or disappear in any release without notice. Their `Err(String)` values
//! are human-readable text, not a contract.
//!
//! Start with [`prelude`]:
//!
//! ```
//! use osatura_core::prelude::*;
//! let anim = Animation::demo();
//! assert_eq!(anim.render_rgba(0.0, &RenderSettings::default()).unwrap().pixels.len() % 4, 0);
//! ```

pub mod api;

/// The supported API in one import: `use osatura_core::prelude::*;`.
pub mod prelude {
    pub use crate::api::{
        ci_bless, ci_init, ci_run, ci_run_mode, trace_image_svg, vcs_abort, vcs_branch, vcs_branch_delete, vcs_branches, vcs_checkout, vcs_cherry_pick, vcs_commit, vcs_conflicts,
        vcs_diff, vcs_fsck, vcs_gc, vcs_init, vcs_log, vcs_merge, vcs_resolve, vcs_revert, vcs_show, vcs_status, vcs_tag, Animation, AnimationInfo, ApiError, CiInit, CiMode, CiOutcome,
        ExportFormat, FitFormat, FuzzSummary, Image, Interpolation, LodLevel, MocapReport, MocapSettings, PhysicsBake, Player, RenderSettings, SizeFit,
        VcsBranch, VcsChange, VcsChoice, VcsCommit, VcsConflict, VcsGc, VcsMergeOutcome, VcsShow, VcsStatus, API_VERSION, API_VERSION_MAJOR,
    };
}

pub mod clock;
#[doc(hidden)]
pub mod collabauth;
#[doc(hidden)]
pub mod agent;
#[doc(hidden)]
pub mod assets;
#[doc(hidden)]
pub mod editordata;
pub mod fsutil;
#[doc(hidden)]
pub mod anim;
#[doc(hidden)]
pub mod anim_tools;
#[doc(hidden)]
pub mod audio;
#[doc(hidden)]
pub mod base64;
#[doc(hidden)]
pub mod bonevis;
#[doc(hidden)]
pub mod camshots;
#[doc(hidden)]
pub mod chanmeta;
#[doc(hidden)]
pub mod collab;
pub mod collabhub;
pub mod collablog;
pub mod comps;
#[doc(hidden)]
pub mod demo;
#[doc(hidden)]
pub mod audiomap;
#[doc(hidden)]
pub mod autorig;
#[doc(hidden)]
pub mod depth;
#[doc(hidden)]
pub mod drivers;
#[doc(hidden)]
pub mod edit;
#[doc(hidden)]
pub mod engine;
#[doc(hidden)]
pub mod links;
pub mod containers;
#[doc(hidden)]
pub mod warp;
#[doc(hidden)]
pub mod explain;
#[doc(hidden)]
pub mod facedet;
#[doc(hidden)]
pub mod facegen;
#[doc(hidden)]
pub mod faceparam;
#[doc(hidden)]
pub mod mocap;
#[doc(hidden)]
pub mod export;
#[doc(hidden)]
pub mod expr;
#[doc(hidden)]
pub mod fx;
mod ik_modes_tests;
#[doc(hidden)]
pub mod eyeclip;
#[doc(hidden)]
pub mod footlock;
#[doc(hidden)]
pub mod looper;
#[doc(hidden)]
pub mod stretch;
#[doc(hidden)]
pub mod dof;
#[doc(hidden)]
pub mod animimport;
#[doc(hidden)]
pub mod soundtrack;
#[doc(hidden)]
pub mod formatschema;
#[doc(hidden)]
pub mod segease;
#[doc(hidden)]
pub mod stereopan;
#[doc(hidden)]
pub mod speech;
#[doc(hidden)]
pub mod gltf;
pub mod fraster;
#[doc(hidden)]
pub mod hdr;
#[doc(hidden)]
pub mod importers;
#[doc(hidden)]
pub mod inspect;
#[doc(hidden)]
pub mod keyops;
#[doc(hidden)]
pub mod layout;
#[doc(hidden)]
pub mod lint;
#[doc(hidden)]
pub mod liveinput;
#[doc(hidden)]
pub mod math;
#[doc(hidden)]
pub mod midi;
#[doc(hidden)]
pub mod noise;
#[doc(hidden)]
pub mod open_formats;
#[doc(hidden)]
pub mod overlays;
#[doc(hidden)]
pub mod scrub;
#[cfg(test)]
mod phase_b_tests;
#[cfg(test)]
mod phase_c_tests;
#[doc(hidden)]
pub mod path;
#[doc(hidden)]
pub mod perf;
#[doc(hidden)]
pub mod pinwarp;
#[doc(hidden)]
pub mod layerfx;
pub mod postfx;
#[doc(hidden)]
pub mod lottie_export;
pub mod lottie_import;
#[doc(hidden)]
pub mod magnet;
#[doc(hidden)]
pub mod motionlib;
pub mod clipstack;
#[doc(hidden)]
pub mod binding;
#[doc(hidden)]
pub mod machine;
#[doc(hidden)]
pub mod parentswitch;
#[doc(hidden)]
pub mod pathfollow;
#[doc(hidden)]
pub mod physics;
#[doc(hidden)]
pub mod cage;
pub mod cloth;
pub mod meshtrace;
#[doc(hidden)]
pub mod posematch;
#[doc(hidden)]
pub mod project;
#[doc(hidden)]
pub mod raster;
#[doc(hidden)]
pub mod rigedit;
mod robustness;
#[doc(hidden)]
pub mod procgen;
#[doc(hidden)]
pub mod rig;
#[doc(hidden)]
pub mod safety;
#[doc(hidden)]
pub mod percep;
#[doc(hidden)]
pub mod ci;
pub mod sizefit;
pub mod lod;
#[doc(hidden)]
#[cfg(feature = "scripting")]
pub mod script;
pub mod scriptops;
#[doc(hidden)]
pub mod seqimport;
#[doc(hidden)]
pub mod skins;
#[doc(hidden)]
pub mod snapshots;
pub mod semdiff;
pub mod vcs;
#[doc(hidden)]
pub mod springfit;
#[doc(hidden)]
pub mod statemachine;
#[doc(hidden)]
pub mod pick;
pub mod autotrace;
pub mod tinymlp;
pub mod posesuggest;
pub mod fuzz;
pub mod gizmo;
pub mod style;
#[doc(hidden)]
pub mod svg;
#[doc(hidden)]
pub mod timeline;
pub mod timeline_graph;
pub mod trace;
pub mod unknownfields;
#[doc(hidden)]
pub mod vocab;
#[doc(hidden)]
pub mod weights;

#[doc(hidden)]
pub use anim::{Ease, Extrapolate, Key, Track};
#[doc(hidden)]
pub use engine::{Engine, Frame, NodeKind, RenderNode};
#[doc(hidden)]
pub use math::{Affine2, Vec2};
#[doc(hidden)]
pub use path::{Anchor, BoneWeight, VPath};
#[doc(hidden)]
pub use project::{Bone, Camera, Channel, IkChain, Layer, LayerKind, MaskRef, Mode, Project, Prop, Source, Transform2D};
#[doc(hidden)]
pub use style::{Blend, Paint, Rgba, StrokeStyle};
