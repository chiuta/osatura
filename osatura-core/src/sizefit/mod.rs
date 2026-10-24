// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Size-targeted export (roadmap I-10): "make this export fit in N bytes, as good as it can be".
//!
//! Own design, textbook parts only:
//!
//! * **Quality ladder.** Every format has a few *knobs* (axes), each an ordered list of values from the
//!   highest quality to the lowest (number precision, path simplification tolerance, key reduction
//!   tolerance, frame decimation, resolution scale, palette size, dithering, lossy frame deltas,
//!   VP9 CRF). The axes are merged into one *chain* of rungs, each rung one step lower than the last
//!   (balanced by normalised position, so no knob is exhausted before the others have moved).
//! * **Search.** Size is expected to fall along the chain, so the best fitting rung is found by
//!   bisection on the *measured* byte count of the produced file. The probes are checked for
//!   monotonicity; if they are not monotone the search falls back to a linear scan. A coordinate
//!   descent (raise one knob one step, lower another one step) then polishes the answer, and an
//!   exhaustive grid is available (`Search::Grid`) for the exact optimum on the grid.
//! * **Quality.** SSIM (`percep`, Wang et al. 2004 with a Gaussian window) between the render of the
//!   original project and a render of the *reduced* export, on `N` sampled frames. GIF, APNG and WebM
//!   are decoded from the produced bytes. Lottie is read back from the produced JSON by a small
//!   reader (the Lottie importer skips animated paths, so it cannot be used) and drawn with the
//!   engine's rasteriser. HTML pages cannot be rasterised here (no SVG rasteriser exists in the
//!   workspace), so the quality of an HTML rung is *modelled*: the same reductions are applied to the
//!   evaluated frame data and that is rasterised. The project format is re-parsed and rendered by
//!   the engine.
//! * **Budget.** The budgeted quantity is the raw file size or its gzip (level 9) size plus a 1 % margin
//!   and 32 bytes ([`gzip_budget_len`], so system `gzip -9` / `-6` output also fits); both raw and plain
//!   gzip sizes are always reported.
//!
//! Clean-room notes: median-cut colour quantisation (Heckbert 1982), Floyd-Steinberg error diffusion
//! (1976), Ramer-Douglas-Peucker / Schneider least-squares Bezier fitting for the reductions,
//! bisection and coordinate descent for the search. Nothing here copies another tool's algorithm
//! or file layout.

use crate::anim::Track;
use crate::engine::{Engine, Frame, NodeKind};
use crate::math::{Affine2, Vec2};
use crate::path::{Anchor, VPath};
use crate::percep::{self, Img};
use crate::project::{Project, Source};
use crate::raster::{pixmap_from_frame, render_rgba, to_straight_rgba, RasterOptions};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{Read, Write};

pub const SCHEMA: &str = "osatura.sizefit/1";


mod settings;
mod util;
mod paths;
mod reduce;
mod quant;
mod webm;
mod series;
mod lottie;
mod output;
mod fitter;
mod report;

pub use settings::*;
pub use util::*;
pub use paths::*;
pub use reduce::*;
pub use quant::*;
pub use webm::*;
use series::*;
pub use lottie::*;
pub use output::*;
pub use fitter::*;
pub use report::*;

#[cfg(test)]
mod tests;
