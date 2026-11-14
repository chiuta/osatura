// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! OSATURA optional GPU renderer.
//!
//! * [`tess`]: CPU tessellation (fan fills for stencil-then-cover, stroke triangles with joins/caps, dashes). No GPU needed.
//! * [`compare`]: image-difference metrics used by the CPU-vs-GPU parity tests. No GPU needed.
//! * [`gpu`]: the wgpu back end (offscreen RGBA8, WGSL shaders in `src/wgsl`).
//!
//! Not part of the default workspace build; see ROADMAP section 9 (R-110) for scope and limits.

pub mod compare;
pub mod gpu;
pub mod tess;

pub use gpu::{
    render, render_frame, AdapterChoice, AdapterSummary, GpuContext, GpuError, GpuImage,
    GpuOptions, Timing,
};
