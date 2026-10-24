// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Animation CI (I-06): golden-frame regression with perceptual diff, structural checks, budgets
//! and reports (JSON, JUnit XML, self-contained HTML).
//!
//! A manifest (`osatura-ci.json`) lists named checks:
//!
//! * `render`      frames are rendered and compared with stored goldens (PNG, content-hash named,
//!                 index in `goldens/index.json`) using [`crate::percep`]: per-pixel colour
//!                 difference, SSIM, differing-pixel fraction, anti-aliasing tolerance, ignore
//!                 regions, optional pHash distance. Renderers: `raster` (tiny-skia, 8 bit),
//!                 `float` (the f32 rasteriser, quantised to 8 bit), `svg` (exact text compare).
//! * `validate`    the project compiles (`Engine::with_asset_dir`) and the linter reports nothing
//!                 at or above a severity.
//! * `fuzz`        the I-03 pose fuzzer stays under a failure rate.
//! * `limits`      layer / bone / channel / frame / canvas / file-size ceilings.
//! * `export-size` byte budget of an export (gif, apng, lottie, html, svg-frames).
//! * `determinism` the same project rendered twice from freshly loaded engines is byte-identical.
//! * `loop`        seam between the last and the first frame compared with typical frame steps.
//! * `perf`        ms/frame ceiling; informational unless `enforce` is true (machine dependent).
//!
//! Exit codes (see [`CiReport::exit_code`]): 0 all pass, 1 some check failed, 2 a check or the
//! manifest could not be evaluated.
use crate::engine::Engine;
use crate::percep::{self, CompareOptions, DeMetric, Img, Region};
use crate::project::Project;
use crate::raster::{render_rgba, RasterOptions};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use crate::clock::Stamp;

mod manifest;
mod results;
mod checks;

pub use manifest::*;
pub use results::*;
pub use checks::*;

#[cfg(test)]
mod tests;
