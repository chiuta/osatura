// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! A tiny multilayer perceptron with its own weight-file format, and a plain SGD trainer.
//!
//! This is the optional "bring your own small model" hook of the pose suggestions. It is textbook
//! feed-forward maths (affine layers, tanh / ReLU / linear activations, mean-squared error, back-propagation,
//! mini-batch stochastic gradient descent with momentum and a linearly decaying step). There is no
//! pretrained model, no dependency, no network: a model is whatever the user trains (here, or with any
//! tool that can write the format below) on their own pose library.
//!
//! # Weight files
//!
//! JSON (`*.json`, human readable) or binary (anything else). Both carry the same content.
//!
//! ```json
//! { "format": "osatura-mlp", "version": 1,
//!   "bones": ["thigh", "shin"],            // pose-completion models: the d bone names, in order
//!   "mean": [0.0, 0.0], "scale": [1.0, 1.0],   // per-bone normalisation, length d (may be empty = identity)
//!   "layers": [ { "act": "tanh", "n_in": 4, "n_out": 8, "w": [ ...n_out*n_in, row-major... ], "b": [ ...n_out... ] },
//!               { "act": "linear", "n_in": 8, "n_out": 2, "w": [ ... ], "b": [ ... ] } ] }
//! ```
//!
//! Binary layout, little endian: magic `ARMLP\x01`, `u32` d, d names (`u16` byte length + UTF-8),
//! d `f32` mean, d `f32` scale, `u32` layer count, then per layer `u8` activation (0 linear, 1 tanh,
//! 2 relu), `u32` n_in, `u32` n_out, `n_out*n_in` `f32` weights (row-major), `n_out` `f32` biases.
//!
//! Pose-completion contract (what `posesuggest` expects): `n_in` of the first layer is `2*d` and `n_out`
//! of the last is `d`. The input is `[normalised value or 0 for each bone | 1 if observed else 0 for each
//! bone]`, the output is the full normalised pose; `value = out * scale + mean`.
//!
//! Limits (enforced on load): at most 64 bones, 8 layers, 4096 units per layer, 4 million weights.

use serde::{Deserialize, Serialize};

pub const MAGIC: &[u8; 6] = b"ARMLP\x01";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Act {
    Linear,
    Tanh,
    Relu,
}

impl Act {
    fn f(self, x: f32) -> f32 {
        match self {
            Act::Linear => x,
            Act::Tanh => x.tanh(),
            Act::Relu => x.max(0.0),
        }
    }
    /// Derivative as a function of the activation's *output* `y`.
    fn df(self, y: f32) -> f32 {
        match self {
            Act::Linear => 1.0,
            Act::Tanh => 1.0 - y * y,
            Act::Relu => {
                if y > 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
    fn code(self) -> u8 {
        match self {
            Act::Linear => 0,
            Act::Tanh => 1,
            Act::Relu => 2,
        }
    }
    fn from_code(c: u8) -> Result<Act, String> {
        Ok(match c {
            0 => Act::Linear,
            1 => Act::Tanh,
            2 => Act::Relu,
            _ => return Err(format!("unknown activation code {c}")),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dense {
    pub act: Act,
    pub n_in: usize,
    pub n_out: usize,
    /// Row-major, `n_out` rows of `n_in`.
    pub w: Vec<f32>,
    pub b: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mlp {
    #[serde(default = "fmt_name")]
    pub format: String,
    #[serde(default = "one")]
    pub version: u32,
    #[serde(default)]
    pub bones: Vec<String>,
    #[serde(default)]
    pub mean: Vec<f32>,
    #[serde(default)]
    pub scale: Vec<f32>,
    pub layers: Vec<Dense>,
}

fn fmt_name() -> String {
    "osatura-mlp".into()
}
fn one() -> u32 {
    1
}

impl Mlp {
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "osatura-mlp" || self.version != 1 {
            return Err(format!("not an osatura-mlp version 1 file (format '{}', version {})", self.format, self.version));
        }
        if self.layers.is_empty() || self.layers.len() > 8 {
            return Err("an MLP needs 1 to 8 layers".into());
        }
        if self.bones.len() > 64 {
            return Err("at most 64 bones".into());
        }
        let d = self.bones.len();
        if !self.mean.is_empty() && self.mean.len() != d || !self.scale.is_empty() && self.scale.len() != d {
            return Err("mean and scale must be empty or have one value per bone".into());
        }
        let mut total = 0usize;
        for (i, l) in self.layers.iter().enumerate() {
            if l.n_in == 0 || l.n_out == 0 || l.n_in > 4096 || l.n_out > 4096 {
                return Err(format!("layer {i}: sizes must be 1..4096"));
            }
            if l.w.len() != l.n_in * l.n_out || l.b.len() != l.n_out {
                return Err(format!("layer {i}: expected {} weights and {} biases", l.n_in * l.n_out, l.n_out));
            }
            if i > 0 && self.layers[i - 1].n_out != l.n_in {
                return Err(format!("layer {i}: n_in {} does not match the previous layer's n_out {}", l.n_in, self.layers[i - 1].n_out));
            }
            total += l.w.len() + l.b.len();
            if l.w.iter().chain(&l.b).any(|v| !v.is_finite()) {
                return Err(format!("layer {i}: non-finite weight"));
            }
        }
        if total > 4_000_000 {
            return Err("more than 4 million weights".into());
        }
        if self.mean.iter().chain(&self.scale).any(|v| !v.is_finite()) || self.scale.contains(&0.0) {
            return Err("mean/scale must be finite and scale non-zero".into());
        }
        Ok(())
    }

    pub fn n_in(&self) -> usize {
        self.layers[0].n_in
    }
    pub fn n_out(&self) -> usize {
        self.layers[self.layers.len() - 1].n_out
    }

    /// Forward pass.
    pub fn forward(&self, x: &[f32]) -> Vec<f32> {
        let mut a = x.to_vec();
        for l in &self.layers {
            let mut o = vec![0.0f32; l.n_out];
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for j in 0..l.n_out {
                let row = &l.w[j * l.n_in..(j + 1) * l.n_in];
                let s: f32 = row.iter().zip(&a).map(|(w, x)| w * x).sum::<f32>() + l.b[j];
                o[j] = l.act.f(s);
            }
            a = o;
        }
        a
    }

    pub fn from_json(s: &str) -> Result<Mlp, String> {
        let m: Mlp = serde_json::from_str(s).map_err(|e| format!("bad MLP file: {e}"))?;
        m.validate()?;
        Ok(m)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("mlp serialises")
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = MAGIC.to_vec();
        let d = self.bones.len();
        o.extend((d as u32).to_le_bytes());
        for b in &self.bones {
            o.extend((b.len() as u16).to_le_bytes());
            o.extend(b.as_bytes());
        }
        for v in (0..d).map(|i| self.mean.get(i).copied().unwrap_or(0.0)).chain((0..d).map(|i| self.scale.get(i).copied().unwrap_or(1.0))) {
            o.extend(v.to_le_bytes());
        }
        o.extend((self.layers.len() as u32).to_le_bytes());
        for l in &self.layers {
            o.push(l.act.code());
            o.extend((l.n_in as u32).to_le_bytes());
            o.extend((l.n_out as u32).to_le_bytes());
            for v in l.w.iter().chain(&l.b) {
                o.extend(v.to_le_bytes());
            }
        }
        o
    }

    pub fn from_bytes(b: &[u8]) -> Result<Mlp, String> {
        struct R<'a>(&'a [u8], usize);
        impl<'a> R<'a> {
            fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
                if self.1 + n > self.0.len() {
                    return Err("truncated MLP file".into());
                }
                let s = &self.0[self.1..self.1 + n];
                self.1 += n;
                Ok(s)
            }
            fn u32(&mut self) -> Result<u32, String> {
                Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
            }
            fn f32(&mut self) -> Result<f32, String> {
                Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
            }
        }
        if b.len() < MAGIC.len() || &b[..MAGIC.len()] != MAGIC {
            return Err("not an osatura-mlp binary file (bad magic)".into());
        }
        let mut r = R(b, MAGIC.len());
        let d = r.u32()? as usize;
        if d > 64 {
            return Err("at most 64 bones".into());
        }
        let mut bones = Vec::new();
        for _ in 0..d {
            let n = u16::from_le_bytes(r.take(2)?.try_into().unwrap()) as usize;
            bones.push(String::from_utf8(r.take(n)?.to_vec()).map_err(|_| "bone name is not UTF-8".to_string())?);
        }
        let mean = (0..d).map(|_| r.f32()).collect::<Result<Vec<_>, _>>()?;
        let scale = (0..d).map(|_| r.f32()).collect::<Result<Vec<_>, _>>()?;
        let nl = r.u32()? as usize;
        if nl == 0 || nl > 8 {
            return Err("an MLP needs 1 to 8 layers".into());
        }
        let mut layers = Vec::new();
        let mut total = 0usize;
        for _ in 0..nl {
            let act = Act::from_code(r.take(1)?[0])?;
            let (n_in, n_out) = (r.u32()? as usize, r.u32()? as usize);
            if n_in == 0 || n_out == 0 || n_in > 4096 || n_out > 4096 {
                return Err("layer sizes must be 1..4096".into());
            }
            total += n_in * n_out + n_out;
            if total > 4_000_000 {
                return Err("more than 4 million weights".into());
            }
            let w = (0..n_in * n_out).map(|_| r.f32()).collect::<Result<Vec<_>, _>>()?;
            let bi = (0..n_out).map(|_| r.f32()).collect::<Result<Vec<_>, _>>()?;
            layers.push(Dense { act, n_in, n_out, w, b: bi });
        }
        if r.1 != b.len() {
            return Err("trailing bytes after the last layer".into());
        }
        let m = Mlp { format: fmt_name(), version: 1, bones, mean, scale, layers };
        m.validate()?;
        Ok(m)
    }

    /// Load JSON when the bytes start with `{` (after whitespace), binary otherwise.
    pub fn load(bytes: &[u8]) -> Result<Mlp, String> {
        if bytes.len() > 64 << 20 {
            return Err("MLP file is larger than 64 MB".into());
        }
        if bytes.starts_with(MAGIC) {
            Mlp::from_bytes(bytes)
        } else {
            Mlp::from_json(std::str::from_utf8(bytes).map_err(|_| "not an osatura-mlp file".to_string())?)
        }
    }
}

// ---------------------------------------------------------------------------------------------
// training
// ---------------------------------------------------------------------------------------------

pub struct Rng(pub u64);
impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xA076_1D64_78BD_642F | 1)
    }
    // Not `Iterator::next`: an endless generator returning `u64`, and the name is part of the stable API.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

#[derive(Clone, Debug)]
pub struct TrainOpts {
    pub hidden: Vec<usize>,
    pub epochs: usize,
    pub lr: f32,
    pub momentum: f32,
    pub batch: usize,
    pub seed: u64,
}

impl Default for TrainOpts {
    fn default() -> Self {
        TrainOpts { hidden: vec![48], epochs: 200, lr: 0.05, momentum: 0.9, batch: 16, seed: 1 }
    }
}

/// One training sample: (input, target, per-output weight).
pub type Sample = (Vec<f32>, Vec<f32>, Vec<f32>);

/// Plain mini-batch SGD with momentum on `(input, target, weight-per-output)` samples drawn by `sample(rng)`.
/// `samples_per_epoch` samples are drawn each epoch; the learning rate decays linearly to 10 % of `lr`.
/// Returns the mean loss of each epoch.
pub fn train(
    n_in: usize,
    n_out: usize,
    o: &TrainOpts,
    samples_per_epoch: usize,
    sample: &mut dyn FnMut(&mut Rng) -> Sample,
) -> Result<(Vec<Dense>, Vec<f32>), String> {
    if o.epochs == 0 || o.batch == 0 || (o.lr.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !o.lr.is_finite() || !(0.0..1.0).contains(&o.momentum) {
        return Err("training needs epochs >= 1, batch >= 1, lr > 0 and 0 <= momentum < 1".into());
    }
    if o.hidden.iter().any(|&h| h == 0 || h > 1024) || o.hidden.len() > 4 {
        return Err("hidden layers: at most 4, each 1..1024 units".into());
    }
    let mut rng = Rng::new(o.seed);
    let mut sizes = vec![n_in];
    sizes.extend(&o.hidden);
    sizes.push(n_out);
    let mut layers: Vec<Dense> = (0..sizes.len() - 1)
        .map(|i| {
            let (a, b) = (sizes[i], sizes[i + 1]);
            // Glorot / Xavier uniform initialisation
            let lim = (6.0 / (a + b) as f32).sqrt();
            Dense { act: if i + 2 == sizes.len() { Act::Linear } else { Act::Tanh }, n_in: a, n_out: b, w: (0..a * b).map(|_| (rng.unit() * 2.0 - 1.0) * lim).collect(), b: vec![0.0; b] }
        })
        .collect();
    let mut vel_w: Vec<Vec<f32>> = layers.iter().map(|l| vec![0.0; l.w.len()]).collect();
    let mut vel_b: Vec<Vec<f32>> = layers.iter().map(|l| vec![0.0; l.b.len()]).collect();
    let mut history = Vec::with_capacity(o.epochs);
    for ep in 0..o.epochs {
        let lr = o.lr * (1.0 - 0.9 * ep as f32 / o.epochs.max(1) as f32);
        let mut epoch_loss = 0.0f64;
        let mut seen = 0usize;
        while seen < samples_per_epoch {
            let nb = o.batch.min(samples_per_epoch - seen);
            let mut gw: Vec<Vec<f32>> = layers.iter().map(|l| vec![0.0; l.w.len()]).collect();
            let mut gb: Vec<Vec<f32>> = layers.iter().map(|l| vec![0.0; l.b.len()]).collect();
            for _ in 0..nb {
                let (x, t, wt) = sample(&mut rng);
                // forward, keeping activations
                let mut acts: Vec<Vec<f32>> = vec![x];
                for l in &layers {
                    let a = acts.last().unwrap();
                    let mut out = vec![0.0f32; l.n_out];
                    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
                    #[allow(clippy::needless_range_loop)]
                    for j in 0..l.n_out {
                        let s: f32 = l.w[j * l.n_in..(j + 1) * l.n_in].iter().zip(a).map(|(w, x)| w * x).sum::<f32>() + l.b[j];
                        out[j] = l.act.f(s);
                    }
                    acts.push(out);
                }
                let y = acts.last().unwrap();
                let wsum: f32 = wt.iter().sum::<f32>().max(1e-9);
                let mut delta: Vec<f32> = (0..n_out).map(|j| 2.0 * wt[j] * (y[j] - t[j]) / wsum).collect();
                epoch_loss += (0..n_out).map(|j| (wt[j] * (y[j] - t[j]).powi(2)) as f64).sum::<f64>() / wsum as f64;
                for li in (0..layers.len()).rev() {
                    let l = &layers[li];
                    for j in 0..l.n_out {
                        delta[j] *= l.act.df(acts[li + 1][j]);
                    }
                    let mut prev = vec![0.0f32; l.n_in];
                    for j in 0..l.n_out {
                        gb[li][j] += delta[j];
                        for i in 0..l.n_in {
                            gw[li][j * l.n_in + i] += delta[j] * acts[li][i];
                            prev[i] += delta[j] * l.w[j * l.n_in + i];
                        }
                    }
                    delta = prev;
                }
            }
            let inv = 1.0 / nb as f32;
            for li in 0..layers.len() {
                for k in 0..layers[li].w.len() {
                    vel_w[li][k] = o.momentum * vel_w[li][k] - lr * gw[li][k] * inv;
                    layers[li].w[k] += vel_w[li][k];
                }
                for k in 0..layers[li].b.len() {
                    vel_b[li][k] = o.momentum * vel_b[li][k] - lr * gb[li][k] * inv;
                    layers[li].b[k] += vel_b[li][k];
                }
            }
            seen += nb;
        }
        history.push((epoch_loss / samples_per_epoch.max(1) as f64) as f32);
    }
    if layers.iter().any(|l| l.w.iter().chain(&l.b).any(|v| !v.is_finite())) {
        return Err("training diverged (non-finite weights); lower the learning rate".into());
    }
    Ok((layers, history))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> Mlp {
        Mlp {
            format: fmt_name(),
            version: 1,
            bones: vec!["a".into(), "b".into()],
            mean: vec![1.0, -1.0],
            scale: vec![2.0, 4.0],
            layers: vec![
                Dense { act: Act::Tanh, n_in: 4, n_out: 3, w: (0..12).map(|i| i as f32 * 0.1 - 0.5).collect(), b: vec![0.1, -0.1, 0.0] },
                Dense { act: Act::Linear, n_in: 3, n_out: 2, w: vec![0.5, -0.5, 0.25, 1.0, 0.0, -1.0], b: vec![0.0, 0.2] },
            ],
        }
    }

    #[test]
    fn forward_matches_a_hand_computation() {
        let m = Dense { act: Act::Relu, n_in: 2, n_out: 2, w: vec![1.0, 2.0, -1.0, 0.5], b: vec![0.5, 0.0] };
        let net = Mlp { format: fmt_name(), version: 1, bones: vec![], mean: vec![], scale: vec![], layers: vec![m] };
        // row0: 1*3 + 2*(-1) + 0.5 = 1.5; row1: -3 - 0.5 = -3.5 -> relu 0
        assert_eq!(net.forward(&[3.0, -1.0]), vec![1.5, 0.0]);
        assert_eq!((net.n_in(), net.n_out()), (2, 2));
        let t = tiny();
        let y = t.forward(&[0.3, -0.2, 1.0, 0.0]);
        let h: Vec<f32> = (0..3).map(|j| ((0..4).map(|i| t.layers[0].w[j * 4 + i] * [0.3f32, -0.2, 1.0, 0.0][i]).sum::<f32>() + t.layers[0].b[j]).tanh()).collect();
        let want0 = 0.5 * h[0] - 0.5 * h[1] + 0.25 * h[2];
        assert!((y[0] - want0).abs() < 1e-6);
    }

    #[test]
    fn json_and_binary_round_trip_exactly() {
        let m = tiny();
        assert_eq!(Mlp::from_json(&m.to_json()).unwrap(), m);
        assert_eq!(Mlp::from_bytes(&m.to_bytes()).unwrap(), m);
        assert_eq!(Mlp::load(&m.to_bytes()).unwrap(), m);
        assert_eq!(Mlp::load(m.to_json().as_bytes()).unwrap(), m);
    }

    #[test]
    fn corrupt_files_are_errors() {
        let m = tiny();
        let mut b = m.to_bytes();
        assert!(Mlp::from_bytes(&b[..b.len() - 3]).is_err(), "truncated");
        b.push(0);
        assert!(Mlp::from_bytes(&b).is_err(), "trailing bytes");
        assert!(Mlp::from_bytes(b"ARMLP\x02").is_err());
        assert!(Mlp::from_bytes(&[]).is_err());
        let mut bad = m.clone();
        bad.layers[1].n_in = 5;
        assert!(bad.validate().is_err());
        let mut bad = m.clone();
        bad.layers[0].w.pop();
        assert!(bad.validate().is_err());
        let mut bad = m.clone();
        bad.layers[0].w[0] = f32::NAN;
        assert!(bad.validate().is_err());
        let mut bad = m.clone();
        bad.scale[0] = 0.0;
        assert!(bad.validate().is_err());
        let mut bad = m.clone();
        bad.format = "other".into();
        assert!(bad.validate().is_err());
        assert!(Mlp::from_json("{\"layers\": 3}").is_err());
        assert!(Mlp::from_json("nonsense").is_err());
        assert!(Mlp::load(&[0xff, 0xfe]).is_err());
        let mut bad = m.clone();
        bad.layers.clear();
        assert!(bad.validate().is_err());
    }

    #[test]
    fn sgd_learns_xor_and_a_linear_map_deterministically() {
        let data = [([0.0f32, 0.0], 0.0f32), ([0.0, 1.0], 1.0), ([1.0, 0.0], 1.0), ([1.0, 1.0], 0.0)];
        let o = TrainOpts { hidden: vec![8], epochs: 1500, lr: 0.05, momentum: 0.9, batch: 4, seed: 3 };
        let run = |o: &TrainOpts| {
            let mut s = |r: &mut Rng| {
                let (x, y) = data[r.below(4)];
                (x.to_vec(), vec![y], vec![1.0])
            };
            train(2, 1, o, 16, &mut s).unwrap()
        };
        let (layers, hist) = run(&o);
        assert!(hist.last().unwrap() < &0.01, "final loss {}", hist.last().unwrap());
        assert!(hist[0] > hist[hist.len() - 1] * 5.0);
        let net = Mlp { format: fmt_name(), version: 1, bones: vec![], mean: vec![], scale: vec![], layers: layers.clone() };
        for (x, y) in data {
            assert!((net.forward(&x)[0] - y).abs() < 0.2, "{x:?}");
        }
        assert_eq!(run(&o).0, layers, "same seed, same weights");
        let (l2, _) = run(&TrainOpts { seed: 4, ..o.clone() });
        assert_ne!(l2, layers, "other seed, other weights");
    }

    #[test]
    fn training_validates_its_options() {
        let mut s = |_: &mut Rng| (vec![0.0], vec![0.0], vec![1.0]);
        for o in [
            TrainOpts { epochs: 0, ..Default::default() },
            TrainOpts { lr: 0.0, ..Default::default() },
            TrainOpts { lr: f32::NAN, ..Default::default() },
            TrainOpts { momentum: 1.0, ..Default::default() },
            TrainOpts { batch: 0, ..Default::default() },
            TrainOpts { hidden: vec![0], ..Default::default() },
            TrainOpts { hidden: vec![2000], ..Default::default() },
        ] {
            assert!(train(1, 1, &o, 4, &mut s).is_err());
        }
    }
}
