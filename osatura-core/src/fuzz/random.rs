// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `fuzz` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// random numbers and low-discrepancy sequences
// ---------------------------------------------------------------------------------------------

/// SplitMix64: a tiny, well-known 64-bit generator (public-domain algorithm), deterministic across platforms.
#[derive(Clone, Debug)]
pub struct Rng(pub u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in `[0, 1)` with 24 bits of resolution.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    /// Uniform integer in `0..n` (`n > 0`).
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n.max(1) as u64) as usize
    }
    /// Standard normal (Box-Muller).
    pub fn gauss(&mut self) -> f32 {
        let u1 = self.unit().max(1e-7);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }
}

/// The first `n` primes (trial division).
pub fn first_primes(n: usize) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::with_capacity(n);
    let mut c = 2u32;
    while out.len() < n {
        if out.iter().take_while(|p| *p * *p <= c).all(|p| !c.is_multiple_of(*p)) {
            out.push(c);
        }
        c += 1;
    }
    out
}

/// Radical inverse of `i` in `base` (the van der Corput sequence): `i = Σ dₖ·bᵏ  ->  Σ dₖ·b^-(k+1)`.
pub fn radical_inverse(mut i: u64, base: u32) -> f64 {
    let b = base as u64;
    let (mut f, mut r) = (1.0f64 / base as f64, 0.0f64);
    while i > 0 {
        r += f * (i % b) as f64;
        i /= b;
        f /= base as f64;
    }
    r
}

/// Halton point `index` (starting at 1) in `dim` dimensions: coordinate `k` is the radical inverse in the `k`-th prime.
pub fn halton_point(index: u64, dim: usize) -> Vec<f32> {
    first_primes(dim).iter().map(|&p| radical_inverse(index, p) as f32).collect()
}

// ---------------------------------------------------------------------------------------------
// parameters
// ---------------------------------------------------------------------------------------------

/// One thing the fuzzer may vary.
#[derive(Clone, Debug, Serialize)]
pub struct Param {
    /// Stable text id: `bone:NAME:angle|scale|offx|offy`, `ik:NAME:mix`, `constraint:NAME:weight`,
    /// `deform:LAYER::DEFORMER::INDEX:dx|dy`.
    pub id: String,
    pub target: String,
    pub prop: Prop,
    pub lo: f32,
    pub hi: f32,
    /// Value at the rest pose.
    pub rest: f32,
    /// Hard limit from the rig (bones only); values outside count as limit violations.
    pub limit: Option<[f32; 2]>,
}

/// Which parameters to enumerate and how wide their ranges are.
#[derive(Clone, Debug)]
pub struct ParamOptions {
    /// Angle range (± degrees around rest) for bones without `angle_limits`.
    pub angle_range: f32,
    /// Extra degrees beyond `angle_limits` / scale-limit fraction (exercises limit handling); 0 = stay inside.
    pub limit_margin: f32,
    /// Scale range for bones without `scale_limits`.
    pub scale_range: [f32; 2],
    /// Offset range (± px); `None` = max(half the bone length, a tenth of the smaller canvas side).
    pub offset_range: Option<f32>,
    pub ik_mix: bool,
    pub constraint_weight: bool,
    /// Deformer control points (capped by `max_deformer_params`, spread evenly).
    pub deformers: bool,
    pub max_deformer_params: usize,
    pub deformer_range: f32,
    /// Restrict to these bone names (empty = all).
    pub only_bones: Vec<String>,
    /// Restrict bone parameters to these kinds: `angle`, `scale`, `offx`, `offy` (empty = all).
    pub bone_props: Vec<String>,
}

impl Default for ParamOptions {
    fn default() -> Self {
        ParamOptions {
            angle_range: 180.0,
            limit_margin: 0.0,
            scale_range: [0.5, 2.0],
            offset_range: None,
            ik_mix: true,
            constraint_weight: true,
            deformers: true,
            max_deformer_params: 24,
            deformer_range: 30.0,
            only_bones: vec![],
            bone_props: vec![],
        }
    }
}

/// All parameters of a project, in a stable order.
pub fn enumerate_params(p: &Project, o: &ParamOptions) -> Vec<Param> {
    // outputs of driven keys are derived values: not probed (the driver stays alive)
    let driven_out: Vec<(&str, Prop)> =
        p.channels.iter().filter(|c| matches!(c.source, Source::Driven { .. })).map(|c| (c.target.as_str(), c.prop)).collect();
    // key range of curves that read a bone property: the natural range of a driver input
    let mut input_range: std::collections::HashMap<(String, Prop), (f32, f32)> = std::collections::HashMap::new();
    for c in &p.channels {
        if let Source::Driven { input, input_prop, curve } = &c.source {
            if let (Some(a), Some(b)) = (curve.first_frame(), curve.last_frame()) {
                let m = 0.1 * (b - a).abs();
                let e = input_range.entry((input.clone(), *input_prop)).or_insert((a - m, b + m));
                e.0 = e.0.min(a - m);
                e.1 = e.1.max(b + m);
            }
        }
    }
    let skip = |t: &str, pr: Prop| driven_out.iter().any(|(tt, pp)| *tt == t && *pp == pr);
    let mut out = Vec::new();
    let side = p.width.min(p.height);
    for b in &p.bones {
        if !o.only_bones.is_empty() && !o.only_bones.contains(&b.name) {
            continue;
        }
        let mut push = |suffix: &str, prop: Prop, lo: f32, hi: f32, rest: f32, limit: Option<[f32; 2]>| {
            if skip(&b.name, prop) || !(o.bone_props.is_empty() || o.bone_props.iter().any(|x| x == suffix)) || (lo.partial_cmp(&hi) != Some(std::cmp::Ordering::Less)) || !lo.is_finite() || !hi.is_finite() {
                return;
            }
            let (lo, hi) = match input_range.get(&(b.name.clone(), prop)) {
                Some(&(a, c)) if limit.is_none() => (a, c),
                _ => (lo, hi),
            };
            out.push(Param { id: format!("bone:{}:{}", b.name, suffix), target: b.name.clone(), prop, lo, hi, rest, limit });
        };
        match b.angle_limits {
            Some([a, c]) => push("angle", Prop::BoneAngle, a - o.limit_margin, c + o.limit_margin, 0.0f32.clamp(a, c), Some([a, c])),
            None => push("angle", Prop::BoneAngle, -o.angle_range, o.angle_range, 0.0, None),
        }
        match b.scale_limits {
            Some([a, c]) => {
                let m = o.limit_margin / 180.0;
                push("scale", Prop::BoneScale, (a - m).max(0.01), c + m, 1.0f32.clamp(a, c), Some([a, c]))
            }
            None => push("scale", Prop::BoneScale, o.scale_range[0], o.scale_range[1], 1.0, None),
        }
        let r = o.offset_range.unwrap_or_else(|| (0.5 * b.length).max(0.1 * side)).abs();
        push("offx", Prop::BoneOffsetX, -r, r, 0.0, None);
        push("offy", Prop::BoneOffsetY, -r, r, 0.0, None);
    }
    // `only_bones` / `bone_props` restrict EVERY sampled parameter, not just the bone ones: an IK mix or constraint
    // weight is kept only when `bone_props` is empty (they are not bone properties) and, with `only_bones`, when the
    // chain / constraint touches one of those bones; deformer points are bone-independent and are dropped.
    let restrict_props = !o.bone_props.is_empty();
    let in_only = |n: &str| o.only_bones.is_empty() || o.only_bones.iter().any(|b| b == n);
    if o.ik_mix && !restrict_props {
        for c in p.ik.iter().filter(|c| in_only(&c.target) || c.bones.iter().any(|b| in_only(b))) {
            out.push(Param { id: format!("ik:{}:mix", c.name), target: c.name.clone(), prop: Prop::IkMix, lo: 0.0, hi: 1.0, rest: c.mix, limit: None });
        }
    }
    if o.constraint_weight && !restrict_props {
        for c in p.constraints.iter().filter(|c| in_only(&c.bone)) {
            out.push(Param {
                id: format!("constraint:{}:weight", c.name),
                target: c.name.clone(),
                prop: Prop::ConstraintWeight,
                lo: 0.0,
                hi: 1.0,
                rest: c.weight,
                limit: None,
            });
        }
    }
    if o.deformers && o.only_bones.is_empty() && !restrict_props {
        let mut dp: Vec<Param> = Vec::new();
        fn walk(layers: &[crate::project::Layer], prefix: &str, o: &ParamOptions, dp: &mut Vec<Param>) {
            for l in layers {
                let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
                for d in &l.deformers {
                    let (n, ctl, props, offs): (usize, bool, (Prop, Prop), &Vec<Vec2>) = match d {
                        Deformer::Lattice { cols, rows, offsets, .. } => ((cols + 1) * (rows + 1), true, (Prop::LatticeDx, Prop::LatticeDy), offsets),
                        Deformer::Cloth { cols, rows, offsets, .. } => ((cols + 1) * (rows + 1), true, (Prop::LatticeDx, Prop::LatticeDy), offsets),
                        Deformer::Spine { rest, offsets, .. } => (rest.len(), false, (Prop::SpineDx, Prop::SpineDy), offsets),
                        Deformer::Pin { pins, offsets, .. } => (pins.len(), false, (Prop::PinDx, Prop::PinDy), offsets),
                        Deformer::Cage { rest, offsets, .. } => (rest.len(), false, (Prop::CageDx, Prop::CageDy), offsets),
                    };
                    let cols = match d {
                        Deformer::Lattice { cols, .. } | Deformer::Cloth { cols, .. } => *cols,
                        _ => 0,
                    };
                    for i in 0..n {
                        let key = if ctl { format!("{path}::{}::{},{}", d.name(), i % (cols + 1), i / (cols + 1)) } else { format!("{path}::{}::{i}", d.name()) };
                        let off = offs.get(i).copied().unwrap_or(Vec2::ZERO);
                        let r = o.deformer_range;
                        dp.push(Param { id: format!("deform:{key}:dx"), target: key.clone(), prop: props.0, lo: off.x - r, hi: off.x + r, rest: off.x, limit: None });
                        dp.push(Param { id: format!("deform:{key}:dy"), target: key, prop: props.1, lo: off.y - r, hi: off.y + r, rest: off.y, limit: None });
                    }
                }
                walk(l.children(), &path, o, dp);
            }
        }
        walk(&p.layers, "", o, &mut dp);
        if dp.len() > o.max_deformer_params {
            let n = o.max_deformer_params;
            let total = dp.len();
            dp = (0..n).map(|k| dp[k * total / n.max(1)].clone()).collect();
        }
        out.extend(dp);
    }
    out
}
