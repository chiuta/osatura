// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Data binding: named external values drive properties of the project.
//!
//! A host (a game, an app, a test script) publishes plain numbers under dotted paths such as
//! `player.health`. A [`Binding`] maps one such path through an optional expression, a range
//! remap with clamp and an optional easing curve, and exponential smoothing onto a *target*:
//!
//! * a property (`layer_opacity`, `bone_angle`, a parameter, `layer_switch` for switch layers,
//!   any other numeric [`Prop`]), written as an ordinary constant channel, or
//! * a number / bool / trigger input of a state machine ([`crate::machine`]).
//!
//! The binding is one-way (value -> property). All maths are textbook: affine range remap,
//! one-pole low-pass `y += (x - y) * (1 - exp(-dt / tau))`, and the engine's own expression
//! language ([`crate::expr`]) with the bound value available as the variable `value`.
//! Everything is deterministic: the output depends only on the sequence of
//! `set_value` / `update(dt)` calls.
//!
//! Expressions: `value * 100`, `clamp(value, 0, 1) ^ 2`, `value > 0.5 ? 1 : 0`. The time and
//! frame variables of channel expressions (`frame`, `time`, `fps`, `frames`, `t`) and `get(..)`
//! are not available here.

use crate::anim::{Ease, Track};
use crate::expr::{self, Ctx, Expr};
use crate::project::{Channel, Mode, Prop};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

fn yes() -> bool {
    true
}
fn one() -> f32 {
    1.0
}

/// Linear range remap `[in_min, in_max] -> [out_min, out_max]` with optional clamp and easing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Remap {
    pub in_min: f32,
    pub in_max: f32,
    pub out_min: f32,
    pub out_max: f32,
    /// Clamp the input to its range first (default true).
    #[serde(default = "yes")]
    pub clamp: bool,
    /// Easing applied to the normalised position (the input is clamped when this is not linear).
    #[serde(default)]
    pub curve: Ease,
}

/// What a binding drives.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    /// A property of a bone / layer / parameter / camera ... (same targets as a channel).
    Prop { target: String, prop: Prop },
    /// An input of a state machine. A trigger input fires on the rising edge (<= 0 to > 0).
    Input { machine: String, input: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub name: String,
    /// External value path, e.g. `player.health`.
    pub path: String,
    /// Optional expression over `value`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remap: Option<Remap>,
    /// Smoothing time constant in seconds (0 = none).
    #[serde(default)]
    pub smooth: f32,
    pub target: Target,
    /// How a property target combines with the animation below it.
    #[serde(default)]
    pub mode: Mode,
    /// Channel weight for property targets, 0..1.
    #[serde(default = "one")]
    pub weight: f32,
}

impl Binding {
    pub fn new(name: &str, path: &str, target: Target) -> Binding {
        Binding {
            name: name.into(),
            path: path.into(),
            expr: None,
            remap: None,
            smooth: 0.0,
            target,
            mode: Mode::Replace,
            weight: 1.0,
        }
    }
}

const FORBIDDEN_VARS: [&str; 5] = ["frame", "time", "fps", "frames", "t"];

/// Rewrite the identifier `value` to the expression engine's input variable and reject variables
/// that make no sense for a binding.
fn rewrite(src: &str) -> Result<String, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let prev_numeric = i > 0 && (chars[i - 1].is_ascii_digit() || chars[i - 1] == '.');
        if (c.is_alphabetic() || c == '_') && !prev_numeric {
            let s = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let id: String = chars[s..i].iter().collect();
            // A following '(' means a function name.
            let is_call = chars[i..].iter().find(|c| !c.is_whitespace()) == Some(&'(');
            if !is_call && id == "value" {
                out.push_str("frame");
            } else if !is_call && FORBIDDEN_VARS.contains(&id.as_str()) {
                return Err(format!(
                    "variable '{id}' is not available in binding expressions (use 'value')"
                ));
            } else {
                out.push_str(&id);
            }
            continue;
        }
        if c == '"' {
            return Err("strings (and get(...)) are not available in binding expressions".into());
        }
        out.push(c);
        i += 1;
    }
    Ok(out)
}

/// Parse a binding expression.
pub fn parse_expr(src: &str) -> Result<Expr, String> {
    if src.len() > expr::MAX_SOURCE_BYTES {
        return Err(format!(
            "expression is {} bytes long, the limit is {}",
            src.len(),
            expr::MAX_SOURCE_BYTES
        ));
    }
    let e = expr::parse(&rewrite(src)?)?;
    let mut gets = Vec::new();
    e.gets(&mut gets);
    if !gets.is_empty() {
        return Err("get(...) is not available in binding expressions".into());
    }
    Ok(e)
}

fn eval_expr(e: &Expr, value: f32) -> f32 {
    let get = |_: &str, _: &str| 0.0f32;
    e.eval(&Ctx {
        frame: value,
        fps: 1.0,
        frames: 1.0,
        get: &get,
    })
}

impl Remap {
    pub fn apply(&self, x: f32) -> f32 {
        let (lo, hi) = (self.in_min.min(self.in_max), self.in_min.max(self.in_max));
        let curved = self.curve != Ease::Linear;
        let x = if self.clamp || curved {
            x.clamp(lo, hi)
        } else {
            x
        };
        let mut u = (x - self.in_min) / (self.in_max - self.in_min);
        if curved {
            u = self.curve.apply(u);
        }
        self.out_min + u * (self.out_max - self.out_min)
    }
}

/// Valid external path: dot separated segments of letters, digits, `_`, `-`, `[`, `]`.
pub fn valid_path(p: &str) -> bool {
    !p.is_empty()
        && p.len() <= 256
        && p.split('.').all(|s| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '[' | ']'))
        })
}

/// Structural validation of `p.bindings` (property targets are checked by compiling an engine,
/// see [`crate::machine::trial_errors`]).
pub fn validate(p: &crate::project::Project) -> Vec<String> {
    let mut errs = Vec::new();
    let mut seen = HashSet::new();
    for b in &p.bindings {
        let n = &b.name;
        if n.trim().is_empty() {
            errs.push("binding needs a name".into());
        } else if !seen.insert(n.clone()) {
            errs.push(format!("duplicate binding name '{n}'"));
        }
        if !valid_path(&b.path) {
            errs.push(format!("binding '{n}': invalid value path '{}'", b.path));
        }
        if let Some(e) = &b.expr {
            if let Err(m) = parse_expr(e) {
                errs.push(format!("binding '{n}': expression error: {m}"));
            }
        }
        if let Some(r) = &b.remap {
            if ![r.in_min, r.in_max, r.out_min, r.out_max]
                .iter()
                .all(|v| v.is_finite())
            {
                errs.push(format!("binding '{n}': remap values must be finite"));
            } else if r.in_min == r.in_max {
                errs.push(format!(
                    "binding '{n}': remap in_min and in_max must differ"
                ));
            }
        }
        if !(b.smooth.is_finite() && b.smooth >= 0.0) {
            errs.push(format!(
                "binding '{n}': smooth must be a finite number >= 0"
            ));
        }
        if !(b.weight.is_finite() && (0.0..=1.0).contains(&b.weight)) {
            errs.push(format!("binding '{n}': weight must be within 0..1"));
        }
        match &b.target {
            Target::Prop { target, .. } => {
                if target.is_empty() && b.target_needs_name() {
                    errs.push(format!("binding '{n}': target name is empty"));
                }
            }
            Target::Input { machine, input } => {
                match p.machines.iter().find(|m| &m.name == machine) {
                    None => errs.push(format!("binding '{n}': unknown machine '{machine}'")),
                    Some(m) => {
                        if !m.inputs.iter().any(|i| &i.name == input) {
                            errs.push(format!(
                                "binding '{n}': machine '{machine}' has no input '{input}'"
                            ));
                        }
                    }
                }
            }
        }
    }
    errs
}

impl Binding {
    fn target_needs_name(&self) -> bool {
        match &self.target {
            Target::Prop { prop, .. } => prop.target_kind() != crate::project::TargetKind::Camera,
            _ => false,
        }
    }

    /// Expression then remap (no smoothing): the pure part of the pipeline.
    pub fn shape(&self, raw: f32) -> Result<f32, String> {
        let mut v = raw;
        if let Some(src) = &self.expr {
            v = eval_expr(&parse_expr(src)?, v);
        }
        if let Some(r) = &self.remap {
            v = r.apply(v);
        }
        Ok(v)
    }
}

/// Stateful evaluation of a set of bindings.
pub struct BindingRuntime {
    bindings: Vec<Binding>,
    exprs: Vec<Option<Expr>>,
    values: HashMap<String, f64>,
    smoothed: Vec<Option<f64>>,
    /// Last output per binding (None until its path has a value).
    out: Vec<Option<f64>>,
    prev_out: Vec<f64>,
}

impl BindingRuntime {
    pub fn new(bindings: &[Binding]) -> Result<BindingRuntime, String> {
        let mut exprs = Vec::new();
        for b in bindings {
            exprs.push(match &b.expr {
                Some(s) => Some(parse_expr(s).map_err(|m| format!("binding '{}': {m}", b.name))?),
                None => None,
            });
        }
        let n = bindings.len();
        Ok(BindingRuntime {
            bindings: bindings.to_vec(),
            exprs,
            values: HashMap::new(),
            smoothed: vec![None; n],
            out: vec![None; n],
            prev_out: vec![0.0; n],
        })
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Publish a value. Unknown paths are accepted (they simply drive nothing).
    pub fn set_value(&mut self, path: &str, v: f64) -> Result<(), String> {
        if !v.is_finite() {
            return Err(format!("value for '{path}' must be finite"));
        }
        if !valid_path(path) {
            return Err(format!("invalid value path '{path}'"));
        }
        self.values.insert(path.to_string(), v);
        Ok(())
    }

    pub fn paths(&self) -> Vec<String> {
        let mut v: Vec<String> = self.bindings.iter().map(|b| b.path.clone()).collect();
        v.sort();
        v.dedup();
        v
    }

    /// Recompute all outputs; `dt` (seconds) drives the smoothing. The first value of a binding
    /// is taken as is (no glide from zero).
    pub fn update(&mut self, dt: f64) {
        for (i, b) in self.bindings.iter().enumerate() {
            self.prev_out[i] = self.out[i].unwrap_or(0.0);
            let Some(&raw) = self.values.get(&b.path) else {
                continue;
            };
            let mut v = raw as f32;
            if let Some(e) = &self.exprs[i] {
                v = eval_expr(e, v);
            }
            if let Some(r) = &b.remap {
                v = r.apply(v);
            }
            let mut y = v as f64;
            if !y.is_finite() {
                continue;
            }
            if b.smooth > 0.0 {
                if let Some(prev) = self.smoothed[i] {
                    let a = 1.0 - (-dt / b.smooth as f64).exp();
                    y = prev + (y - prev) * a;
                }
            }
            self.smoothed[i] = Some(y);
            self.out[i] = Some(y);
        }
    }

    /// Current output of a binding by name.
    pub fn output(&self, name: &str) -> Option<f64> {
        let i = self.bindings.iter().position(|b| b.name == name)?;
        self.out[i]
    }

    /// Constant channels for the property targets that have a value.
    pub fn channels(&self) -> Vec<Channel> {
        let mut v = Vec::new();
        for (i, b) in self.bindings.iter().enumerate() {
            if let (Target::Prop { target, prop }, Some(y)) = (&b.target, self.out[i]) {
                let mut c = Channel::keys(
                    target,
                    *prop,
                    Track::with_keys(&[(0.0, y as f32, Ease::Linear)]),
                );
                c.mode = b.mode;
                c.weight = b.weight;
                v.push(c);
            }
        }
        v
    }

    /// `(machine, input, value, rising_edge)` for machine-input targets that have a value.
    pub fn input_outputs(&self) -> Vec<(String, String, f64, bool)> {
        let mut v = Vec::new();
        for (i, b) in self.bindings.iter().enumerate() {
            if let (Target::Input { machine, input }, Some(y)) = (&b.target, self.out[i]) {
                v.push((
                    machine.clone(),
                    input.clone(),
                    y,
                    self.prev_out[i] <= 0.0 && y > 0.0,
                ));
            }
        }
        v
    }
}

/// One-shot evaluation (no smoothing history) of `values`: `(binding name, output)` for every
/// binding whose path has a value.
pub fn eval_once(
    bindings: &[Binding],
    values: &[(String, f64)],
) -> Result<Vec<(String, f64)>, String> {
    let mut rt = BindingRuntime::new(bindings)?;
    for (p, v) in values {
        rt.set_value(p, *v)?;
    }
    rt.update(0.0);
    Ok(bindings
        .iter()
        .filter_map(|b| rt.output(&b.name).map(|v| (b.name.clone(), v)))
        .collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn binding_expressions_are_depth_and_length_capped() {
        assert!(parse_expr(&format!("{}1{}", "(".repeat(20_000), ")".repeat(20_000))).is_err());
        assert!(parse_expr(&format!("{}1", "-".repeat(200_000))).is_err());
        assert!(parse_expr("value * 2 + 1").is_ok());
        let mut b = prop("b", "player.health");
        b.expr = Some(format!("{}1{}", "(".repeat(20_000), ")".repeat(20_000)));
        let mut p = Project::new("p", 10.0, 10.0, 24.0, 2);
        p.bindings.push(b);
        assert!(!validate(&p).is_empty(), "validation reports it, no abort");
    }

    use super::*;
    use crate::project::{Project, Source};

    fn prop(name: &str, path: &str) -> Binding {
        Binding::new(
            name,
            path,
            Target::Prop {
                target: "arm".into(),
                prop: Prop::BoneAngle,
            },
        )
    }

    #[test]
    fn remap_and_clamp() {
        let mut b = prop("b", "player.health");
        b.remap = Some(Remap {
            in_min: 0.0,
            in_max: 100.0,
            out_min: 0.0,
            out_max: 1.0,
            clamp: true,
            curve: Ease::Linear,
        });
        assert!((b.shape(50.0).unwrap() - 0.5).abs() < 1e-6);
        assert_eq!(b.shape(250.0).unwrap(), 1.0);
        assert_eq!(b.shape(-5.0).unwrap(), 0.0);
        b.remap.as_mut().unwrap().clamp = false;
        assert!((b.shape(200.0).unwrap() - 2.0).abs() < 1e-6);
        // reversed output range
        b.remap = Some(Remap {
            in_min: 0.0,
            in_max: 10.0,
            out_min: 90.0,
            out_max: -90.0,
            clamp: true,
            curve: Ease::Linear,
        });
        assert!((b.shape(5.0).unwrap()).abs() < 1e-4);
        assert_eq!(b.shape(99.0).unwrap(), -90.0);
    }

    #[test]
    fn expression_uses_value() {
        let mut b = prop("b", "x");
        b.expr = Some("value * 2 + 1".into());
        assert_eq!(b.shape(3.0).unwrap(), 7.0);
        b.expr = Some("value > 0.5 ? 10 : clamp(value, 0, 0.25)".into());
        assert_eq!(b.shape(0.9).unwrap(), 10.0);
        assert_eq!(b.shape(0.1).unwrap(), 0.1);
        b.expr = Some("time + 1".into());
        assert!(b.shape(1.0).is_err());
        b.expr = Some("get(\"a\", \"bone_angle\")".into());
        assert!(b.shape(1.0).is_err());
        b.expr = Some("1e3 * value".into());
        assert_eq!(b.shape(2.0).unwrap(), 2000.0);
    }

    #[test]
    fn smoothing_is_exponential_and_first_value_snaps() {
        let mut b = prop("b", "x");
        b.smooth = 0.5;
        let mut rt = BindingRuntime::new(&[b]).unwrap();
        assert!(rt.channels().is_empty());
        rt.set_value("x", 1.0).unwrap();
        rt.update(0.1);
        assert_eq!(rt.output("b"), Some(1.0));
        rt.set_value("x", 0.0).unwrap();
        rt.update(0.5);
        let want = 1.0 - (1.0 - (-1.0f64).exp());
        assert!((rt.output("b").unwrap() - want).abs() < 1e-9);
        for _ in 0..100 {
            rt.update(0.5);
        }
        assert!(rt.output("b").unwrap().abs() < 1e-6);
    }

    #[test]
    fn channels_for_property_targets() {
        let mut b = prop("b", "p.v");
        b.mode = Mode::Add;
        b.weight = 0.5;
        let mut rt = BindingRuntime::new(&[b]).unwrap();
        rt.set_value("p.v", 30.0).unwrap();
        rt.update(0.0);
        let ch = rt.channels();
        assert_eq!(ch.len(), 1);
        assert_eq!(ch[0].mode, Mode::Add);
        assert_eq!(ch[0].weight, 0.5);
        match &ch[0].source {
            Source::Keys { track } => assert_eq!(track.eval(10.0), Some(30.0)),
            _ => panic!(),
        }
    }

    #[test]
    fn validation_errors() {
        let mut p = Project::new("t", 10.0, 10.0, 24.0, 10);
        let mut bad = prop("a", "bad path");
        bad.smooth = -1.0;
        bad.expr = Some("value +".into());
        bad.remap = Some(Remap {
            in_min: 1.0,
            in_max: 1.0,
            out_min: 0.0,
            out_max: 1.0,
            clamp: true,
            curve: Ease::Linear,
        });
        p.bindings = vec![
            bad,
            prop("a", "ok.path"),
            Binding::new(
                "c",
                "q",
                Target::Input {
                    machine: "m".into(),
                    input: "i".into(),
                },
            ),
        ];
        let errs = validate(&p).join("\n");
        assert!(errs.contains("invalid value path"), "{errs}");
        assert!(errs.contains("smooth"), "{errs}");
        assert!(errs.contains("expression error"), "{errs}");
        assert!(errs.contains("must differ"), "{errs}");
        assert!(errs.contains("duplicate binding name 'a'"), "{errs}");
        assert!(errs.contains("unknown machine 'm'"), "{errs}");
        assert!(BindingRuntime::new(&p.bindings).is_err());
        assert!(valid_path("player.items[2].hp") && !valid_path("a..b") && !valid_path(""));
    }

    #[test]
    fn eval_once_reports_outputs() {
        let mut b = prop("op", "ui.alpha");
        b.remap = Some(Remap {
            in_min: 0.0,
            in_max: 255.0,
            out_min: 0.0,
            out_max: 1.0,
            clamp: true,
            curve: Ease::Linear,
        });
        let r = eval_once(&[b], &[("ui.alpha".into(), 255.0), ("other".into(), 1.0)]).unwrap();
        assert_eq!(r, vec![("op".to_string(), 1.0)]);
    }
}
