// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! The supported, stable public API of `osatura-core`.
//!
//! Only this module and [`crate::prelude`] are covered by the stability promise below. Every other
//! module of the crate (and the root-level re-exports) is an *internal, unstable* implementation
//! detail used by the CLI, the studio and the C ABI; it may change in any release and is hidden
//! from the generated documentation.
//!
//! # Stability policy
//!
//! [`API_VERSION`] follows semantic versioning for this facade only:
//!
//! * **patch** - bug fixes, no signature changes;
//! * **minor** - additions only: new functions, methods, enum variants and struct fields. Public
//!   enums and structs of the facade are `#[non_exhaustive]`, so downstream code must use a
//!   wildcard match arm and cannot build them with a struct literal (use the constructors);
//! * **major** - anything removed or changed incompatibly. The checked-in `api_surface.txt` lists
//!   every public item of this file and a test fails whenever the list changes, so each change to
//!   the surface is a deliberate, reviewed edit.
//!
//! Errors are values: every fallible function returns [`ApiError`], bad input never panics, and a
//! panic inside the engine is caught and reported as [`ApiError::Internal`].
//!
//! # Example
//!
//! ```
//! use osatura_core::prelude::*;
//!
//! # fn main() -> Result<(), ApiError> {
//! // A project is plain JSON. `Animation::demo()` returns a small built-in one.
//! let json = Animation::demo().to_json();
//! let mut anim = Animation::from_json(&json)?;
//! let info = anim.info();
//! assert!(info.frames > 0 && info.fps > 0.0);
//!
//! // Query the rig.
//! let first_bone = anim.bone_names()[0].clone();
//! let pose = anim.bone_world(0.0)?; // 6 floats per bone: a b c d tx ty
//! assert_eq!(pose.len(), anim.bone_names().len());
//!
//! // Edit through the undoable editor (validated; locks are respected).
//! anim.set_key(&first_bone, "bone_angle", 4.0, 30.0, Interpolation::EaseInOut)?;
//! assert!(anim.undo());
//!
//! // Render.
//! let image = anim.render_rgba(0.0, &RenderSettings::default())?;
//! assert_eq!(image.pixels.len(), image.width as usize * image.height as usize * 4);
//! let svg = anim.render_svg(0.0, false)?;
//! assert!(svg.starts_with("<svg"));
//! let png = anim.render_png(0.0, &RenderSettings::default())?;
//! assert_eq!(&png[1..4], b"PNG");
//!
//! // Bad input is an error value, never a panic.
//! assert!(matches!(Animation::from_json("{ nope"), Err(ApiError::InvalidJson(_))));
//! assert!(anim.render_rgba(f32::NAN, &RenderSettings::default()).is_err());
//! # Ok(()) }
//! ```
use crate::anim::Ease;
use crate::edit::Editor;
use crate::project::{Prop, Project};
use crate::raster::{render_png, render_rgba, RasterOptions};
use crate::svg::{render_frame, RenderOptions};
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};

mod facade;
mod vcs;
mod ci_write;

pub use facade::*;
pub use vcs::*;
pub use ci_write::*;

#[cfg(test)]
mod tests;
