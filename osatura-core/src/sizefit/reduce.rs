// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Key reduction of project tracks and removal of dead layers (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Key reduction (project tracks, exact error bound in value units)

/// Frames at which a track is compared: every integer frame of the timeline and every key frame.
pub(super) fn track_samples(t: &Track, frames: u32) -> Vec<f32> {
    let mut v: Vec<f32> = (0..frames.max(1)).map(|f| f as f32).collect();
    v.extend(t.keys.iter().map(|k| k.frame));
    v.sort_by(|a, b| a.total_cmp(b));
    v.dedup();
    v
}

/// Largest |reduced - original| over the integer frames of the timeline and all original key frames.
pub fn track_deviation(orig: &Track, red: &Track, frames: u32) -> f64 {
    track_samples(orig, frames)
        .into_iter()
        .map(|f| match (orig.eval(f), red.eval(f)) {
            (Some(a), Some(b)) => (a as f64 - b as f64).abs(),
            _ => 0.0,
        })
        .fold(0.0, f64::max)
}

/// Drop keys while `track_deviation <= tol`. First and last key are kept, easings of kept keys are
/// unchanged. Top-down Ramer-Douglas-Peucker on key indices followed by a repair loop that re-inserts
/// the removed key nearest to the worst sample until the bound holds for the *whole* track (it
/// terminates: at worst every key comes back and the deviation is 0).
pub fn reduce_track(t: &Track, tol: f32, frames: u32) -> Track {
    let n = t.keys.len();
    if n < 3 || tol < 0.0 || !tol.is_finite() {
        return t.clone();
    }
    let tol = tol as f64;
    let samples = track_samples(t, frames);
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
        // worst sample inside the span; split at the original key nearest to it
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
        // find the worst sample of the whole reduced track
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyReduction {
    pub channels: usize,
    pub keys_before: usize,
    pub keys_after: usize,
    pub max_deviation: f64,
}

/// Reduce every keyed channel of `p` in place.
pub fn reduce_project_keys(p: &mut Project, tol: f32) -> KeyReduction {
    let frames = p.frames;
    let mut r = KeyReduction::default();
    for c in &mut p.channels {
        if let Source::Keys { track } = &mut c.source {
            r.channels += 1;
            r.keys_before += track.keys.len();
            let red = reduce_track(track, tol, frames);
            r.max_deviation = r.max_deviation.max(track_deviation(track, &red, frames));
            r.keys_after += red.keys.len();
            *track = red;
        }
    }
    r
}

// Dead layers

pub(super) fn mask_targets(ls: &[crate::project::Layer], out: &mut std::collections::HashSet<String>) {
    for l in ls {
        if let Some(m) = &l.mask {
            out.insert(m.layer.clone());
        }
        mask_targets(l.children(), out);
    }
}

/// Layers that never draw anything: hidden (or fully transparent, or under a hidden ancestor) in every
/// frame, not used as a mask source, not a child of a switch (its index would shift). Each removal is
/// accepted only if the project still validates, and the final project must render pixel-identical
/// frames at quarter resolution on up to 12 sample frames - otherwise nothing is removed.
pub fn drop_dead_layers(p: &Project) -> (Project, Vec<String>) {
    let unchanged = || (p.clone(), Vec::new());
    let Ok(e0) = Engine::new(p.clone()) else { return unchanged() };
    let frames: Vec<Frame> = (0..p.frames).map(|f| e0.evaluate(f as f32)).collect();
    let n = frames.first().map_or(0, |f| f.nodes.len());
    let mut alive = vec![false; n];
    for f in &frames {
        fn walk(f: &Frame, id: usize, op: f32, vis: bool, alive: &mut [bool]) {
            let nd = &f.nodes[id];
            let vis = vis && nd.visible;
            let op = op * nd.opacity;
            if vis && op > 0.0 {
                alive[id] = true;
            }
            if vis {
                for &c in &nd.children {
                    walk(f, c, op, vis, alive);
                }
            }
        }
        for &r in &f.roots {
            walk(f, r, 1.0, true, &mut alive);
        }
        for nd in &f.nodes {
            if let Some((m, _)) = nd.mask {
                alive[m] = true;
            }
        }
    }
    let mut masked = std::collections::HashSet::new();
    mask_targets(&p.layers, &mut masked);
    // nodes whose parent is a switch
    let mut under_switch = vec![false; n];
    for nd in &frames[0].nodes {
        if nd.kind == NodeKind::Switch {
            // every layer path starting with this node's path + "/" is a (grand)child; only direct children matter
            let prefix = format!("{}/", nd.name);
            for m in &frames[0].nodes {
                if m.name.starts_with(&prefix) && !m.name[prefix.len()..].contains('/') {
                    under_switch[m.id] = true;
                }
            }
        }
    }
    let mut cands: Vec<String> = Vec::new();
    for nd in &frames[0].nodes {
        if alive[nd.id] || under_switch[nd.id] || masked.contains(&nd.name) {
            continue;
        }
        // topmost dead node only
        let parent_dead = nd.name.rfind('/').is_some_and(|i| {
            let pn = &nd.name[..i];
            frames[0].nodes.iter().any(|m| m.name == pn && !alive[m.id])
        });
        if !parent_dead {
            cands.push(nd.name.clone());
        }
    }
    if cands.is_empty() {
        return unchanged();
    }
    let mut q = p.clone();
    let mut dropped = Vec::new();
    for c in cands {
        let mut t = q.clone();
        if crate::scriptops::delete_layer(&mut t, &c).is_ok() && Engine::new(t.clone()).is_ok() {
            q = t;
            dropped.push(c);
        }
    }
    if dropped.is_empty() {
        return unchanged();
    }
    let Ok(e1) = Engine::new(q.clone()) else { return unchanged() };
    let opts = RasterOptions { scale: 0.25, show_bones: false, transparent: false };
    let m = p.frames.max(1);
    let k = 12.min(m);
    for j in 0..k {
        let f = (j * m / k) as f32;
        if render_rgba(&e0, f, &opts).2 != render_rgba(&e1, f, &opts).2 {
            return unchanged();
        }
    }
    (q, dropped)
}
