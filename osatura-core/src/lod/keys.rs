// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Key reduction with an L-infinity bound (part of `lod`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Key reduction with an L-infinity bound

pub(super) fn merged_samples(t: &Track, grid: &[f32]) -> Vec<f32> {
    let mut v: Vec<f32> = grid.to_vec();
    v.extend(t.keys.iter().map(|k| k.frame));
    v.sort_by(|a, b| a.total_cmp(b));
    v.dedup();
    v
}

/// Largest `|reduced - original|` over `samples`.
pub fn track_deviation_at(orig: &Track, red: &Track, samples: &[f32]) -> f64 {
    samples
        .iter()
        .map(|&f| match (orig.eval(f), red.eval(f)) {
            (Some(a), Some(b)) => (a as f64 - b as f64).abs(),
            _ => 0.0,
        })
        .fold(0.0, f64::max)
}

/// Drop keys while the deviation over `grid` (plus every original key frame) stays within `tol`.
/// First and last key stay; kept keys keep their easing. Ramer-Douglas-Peucker on key indices, then a
/// repair loop re-inserts keys until the bound holds for the whole sample set.
pub fn reduce_track_bounded(t: &Track, tol: f64, grid: &[f32]) -> Track {
    let n = t.keys.len();
    if n < 3 || !matches!(tol.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) || !tol.is_finite() {
        return t.clone();
    }
    let samples = merged_samples(t, grid);
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    let sub = |lo: usize, hi: usize| Track { keys: vec![t.keys[lo], t.keys[hi]], extrapolate: t.extrapolate };
    let mut stack = vec![(0usize, n - 1)];
    while let Some((lo, hi)) = stack.pop() {
        if hi - lo < 2 {
            continue;
        }
        let cand = sub(lo, hi);
        let (f0, f1) = (t.keys[lo].frame, t.keys[hi].frame);
        let mut worst = (0.0f64, f0);
        for &f in samples.iter().filter(|&&f| f > f0 && f < f1) {
            let e = (t.eval(f).unwrap_or(0.0) as f64 - cand.eval(f).unwrap_or(0.0) as f64).abs();
            if e > worst.0 {
                worst = (e, f);
            }
        }
        if worst.0 <= tol {
            continue;
        }
        let split = (lo + 1..hi).min_by(|&a, &b| (t.keys[a].frame - worst.1).abs().total_cmp(&(t.keys[b].frame - worst.1).abs())).unwrap();
        keep[split] = true;
        stack.push((lo, split));
        stack.push((split, hi));
    }
    let build = |keep: &[bool]| Track { keys: t.keys.iter().zip(keep).filter(|(_, &k)| k).map(|(k, _)| *k).collect(), extrapolate: t.extrapolate };
    let mut red = build(&keep);
    loop {
        let mut worst = (0.0f64, 0.0f32);
        for &f in &samples {
            let e = (t.eval(f).unwrap_or(0.0) as f64 - red.eval(f).unwrap_or(0.0) as f64).abs();
            if e > worst.0 {
                worst = (e, f);
            }
        }
        if worst.0 <= tol {
            return red;
        }
        let Some(add) = (0..n).filter(|&i| !keep[i]).min_by(|&a, &b| (t.keys[a].frame - worst.1).abs().total_cmp(&(t.keys[b].frame - worst.1).abs())) else {
            return red;
        };
        keep[add] = true;
        red = build(&keep);
    }
}

pub(super) struct KeyOutcome {
    pub(super) channels: Vec<Channel>,
    pub(super) keys_px: f64,
    pub(super) reduced: usize,
    pub(super) notes: Vec<String>,
}

/// Reduce the eligible channels so that the summed pixel error at any layer stays within `budget_px`.
pub(super) fn reduce_keys(c: &Ctx, an: &Analysis, chans: &[(usize, LKind)], scale: f64, budget_px: f64, grid: &[f32]) -> KeyOutcome {
    let p = c.p;
    let mut out = p.channels.clone();
    let mut notes = Vec::new();
    // co-influence count K: how many reducible channels can move the same layer
    let usable: Vec<usize> = (0..chans.len()).filter(|&i| !an.blocked[i] && an.lever[i].values().any(|&v| v > 0.0)).collect();
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for i in 0..chans.len() {
        if an.blocked[i] {
            notes.push(format!("channel {} {:?} skipped: lever not bounded", p.channels[chans[i].0].target, p.channels[chans[i].0].prop));
        }
    }
    let mut count: BTreeMap<usize, usize> = BTreeMap::new();
    for &i in &usable {
        for (&node, &v) in &an.lever[i] {
            if v > 0.0 {
                *count.entry(node).or_insert(0) += 1;
            }
        }
    }
    let k = count.values().copied().max().unwrap_or(1).max(1) as f64;
    let mut dev_by: Vec<(usize, f64)> = Vec::new(); // (index into chans, deviation in px-weighted units = value dev * weight)
    let mut reduced = 0;
    for &i in &usable {
        let ch_i = chans[i].0;
        let ch = &p.channels[ch_i];
        let Source::Keys { track } = &ch.source else { continue };
        let lmax = an.lever[i].values().copied().fold(0.0, f64::max) * scale;
        let w = ch.weight as f64;
        let eps_value = budget_px / (k * lmax * w);
        let red = reduce_track_bounded(track, eps_value, grid);
        let samples = merged_samples(track, grid);
        let dev = track_deviation_at(track, &red, &samples) * w;
        if red.keys.len() < track.keys.len() {
            reduced += 1;
        }
        dev_by.push((i, dev));
        out[ch_i].source = Source::Keys { track: red };
    }
    // declared: per layer, sum over the channels that can move it
    let mut per_node: BTreeMap<usize, f64> = BTreeMap::new();
    for &(i, dev) in &dev_by {
        for (&node, &v) in &an.lever[i] {
            *per_node.entry(node).or_insert(0.0) += dev * v * scale;
        }
    }
    let keys_px = per_node.values().copied().fold(0.0, f64::max) * MARGIN;
    KeyOutcome { channels: out, keys_px, reduced, notes }
}
