// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
use crate::expr::{self, Ctx, Expr};
use crate::math::{Affine2, Vec2};
use crate::noise::{fbm1, noise1};
use crate::path::{Polyline, VPath};
use crate::project::{
    compose_transform, ConstraintKind, Deformer, ImageSource, Layer, LayerKind, Mode, Project, Prop, Source,
    TargetKind, Wobble,
};
use crate::rig::{fk, resolve_parents, soft_distance, solve_fabrik, solve_two_bone_scaled, wrap_pi, BonePose, SpringState};
use crate::style::Blend;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tiny_skia as sk;
#[cfg(test)]
mod reference_tests {
    use super::*;
    use crate::path::VPath;
    use crate::project::Layer;

    #[test]
    fn reference_layers_are_hidden_unless_the_editor_asks() {
        let mut p = Project::new("r", 50.0, 50.0, 24.0, 2);
        let mut l = Layer::vector("sketch", vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#ff0000")]);
        l.reference = true;
        p.layers = vec![l];
        let mut e = Engine::new(p.clone()).unwrap();
        assert!(!e.evaluate(0.0).nodes[0].visible);
        e.show_reference = true;
        assert!(e.evaluate(0.0).nodes[0].visible);
        let j = p.to_json_pretty();
        assert!(j.contains("\"reference\": true"));
        assert!(!Project::new("x", 1.0, 1.0, 24.0, 1).to_json_pretty().contains("reference"), "default is not serialised");
    }
}

mod resolved;
mod engine_impl;
mod methods_b;
mod methods_c;

pub use resolved::*;
pub use engine_impl::*;

#[cfg(test)]
mod tests;
