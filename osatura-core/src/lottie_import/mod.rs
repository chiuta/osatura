// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Lottie (Bodymovin JSON) import, a deliberate subset: shape layers with paths (`sh`, static),
//! rectangles, ellipses, solid and gradient fills (also with animated end points / centre / radius, imported as `grad_dx`/`grad_dy` channels), strokes and groups with static transforms;
//! layer transforms (static or keyframed position / scale / rotation / opacity), blend modes and a
//! solid background layer. Everything else is skipped and reported in `warnings`.
//!
//! Also imported: image layers with embedded PNG / JPEG data URIs, null layers as parents, layer
//! `parent` links (the parent's transform is baked into the child, see `bake_parented`), layer
//! masks (add / subtract / intersect with opacity, inversion and keyed shapes, built from hidden
//! source layers and layer masks) and luma mattes (approximated). Layer effects (`ef`): gaussian
//! blur, drop shadow, tint and fill become per-layer effects (`Layer::effects`, see `layerfx`) on any
//! layer kind, in any composition, with animated numbers as channels (keys and easing as for
//! transforms; a shadow's direction / distance are sampled per frame into x / y offsets). Static
//! tint, fill, tritone and levels on shape layers are baked into the colours instead (exact, also for
//! gradients); tritone and levels are not imported on precomp / image layers. Effect colours cannot
//! be animated (first value); all other effects are reported as warnings.
//!
//! Keyframe easing: hold keys become steps, bezier handles (`o`/`i`) become cubic-bezier easing,
//! keys without handles are linear. Layer in/out ranges become a multiplying opacity step track. Curved position paths (`to`/`ti`) are baked per frame.
use crate::anim::{Ease, Track};
use crate::layerfx::LayerEffect;
use crate::math::{Affine2, Vec2};
use crate::path::{Anchor, VPath};
use crate::project::{compose_transform, Channel, Layer, Mode, Project, Prop};
use crate::style::{Blend, Paint, Rgba, Stop, StrokeStyle};
use serde_json::Value;

mod base;
mod layers;
mod effects;

pub use base::*;
use layers::*;
pub use effects::*;

#[cfg(test)]
mod tests;
