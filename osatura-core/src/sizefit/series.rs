// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Time-series reduction shared by Lottie tracks (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Time series reduction shared by Lottie tracks (step decimation, key reduction, deduplication)

/// One sampled value of an animated track: a topology signature (closed flag, vertex count) and the
/// flattened numbers. Two samples can only be interpolated if their signatures are equal.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Samp {
    pub(super) sig: (bool, usize),
    pub(super) v: Vec<f64>,
}

pub(super) fn samp_of_shape(closed: bool, nodes: &[PNode], d: u8) -> Samp {
    let mut v = Vec::with_capacity(nodes.len() * 6);
    for n in nodes {
        v.extend(n.p.iter().map(|&x| round_dec(x, d)));
    }
    for n in nodes {
        v.extend(n.i.iter().map(|&x| round_dec(x, d)));
    }
    for n in nodes {
        v.extend(n.o.iter().map(|&x| round_dec(x, d)));
    }
    Samp { sig: (closed, nodes.len()), v }
}

pub(super) fn shape_of_samp(s: &Samp) -> Value {
    let n = s.sig.1;
    let pair = |off: usize, k: usize| json!([num(s.v[off + 2 * k]), num(s.v[off + 2 * k + 1])]);
    let vs: Vec<Value> = (0..n).map(|k| pair(0, k)).collect();
    let is: Vec<Value> = (0..n).map(|k| pair(2 * n, k)).collect();
    let os: Vec<Value> = (0..n).map(|k| pair(4 * n, k)).collect();
    json!({"i": is, "o": os, "v": vs, "c": s.sig.0})
}

pub(super) fn parse_shape(v: &Value) -> Option<(bool, Vec<PNode>)> {
    let arr = |k: &str| -> Option<Vec<[f64; 2]>> {
        v.get(k)?.as_array()?.iter().map(|p| { let a = p.as_array()?; Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?]) }).collect()
    };
    let (vv, ii, oo) = (arr("v")?, arr("i")?, arr("o")?);
    if vv.len() != ii.len() || vv.len() != oo.len() {
        return None;
    }
    let closed = v.get("c").and_then(Value::as_bool).unwrap_or(false);
    Some((closed, vv.iter().zip(&ii).zip(&oo).map(|((&p, &i), &o)| PNode { p, i, o }).collect()))
}

/// Choose the keyframes of a series. `step` keeps every `step`-th frame; with `linear` the kept frames are
/// reduced by Ramer-Douglas-Peucker so that linear interpolation between neighbours stays within `tol`
/// (max over all numbers, at the dropped frames); without it the track is held and only repeats are dropped.
/// Returns `(frame, hold_after)`.
pub(super) fn select_keys(s: &[Samp], step: usize, tol: f64, linear: bool) -> Vec<(usize, bool)> {
    let n = s.len();
    let k: Vec<usize> = (0..n).step_by(step.max(1)).collect();
    if k.is_empty() {
        return vec![];
    }
    if !linear {
        let mut out: Vec<(usize, bool)> = Vec::new();
        for &f in &k {
            if out.last().is_none_or(|&(pf, _)| s[pf] != s[f]) {
                out.push((f, true));
            }
        }
        return out;
    }
    let mut keep = vec![false; k.len()];
    let mut run = 0usize;
    for i in 0..=k.len() {
        if i < k.len() && s[k[i]].sig == s[k[run]].sig {
            continue;
        }
        let (a0, b0) = (run, i - 1);
        keep[a0] = true;
        keep[b0] = true;
        let mut stack = vec![(a0, b0)];
        while let Some((a, b)) = stack.pop() {
            if b - a < 2 {
                continue;
            }
            let (fa, fb) = (k[a] as f64, k[b] as f64);
            let mut worst = (0.0f64, 0usize);
            for j in a + 1..b {
                let t = (k[j] as f64 - fa) / (fb - fa);
                let e = (0..s[k[j]].v.len()).map(|c| (s[k[j]].v[c] - lerp(s[k[a]].v[c], s[k[b]].v[c], t)).abs()).fold(0.0, f64::max);
                if e > worst.0 {
                    worst = (e, j);
                }
            }
            if worst.0 > tol {
                keep[worst.1] = true;
                stack.push((a, worst.1));
                stack.push((worst.1, b));
            }
        }
        run = i;
    }
    let kept: Vec<usize> = (0..k.len()).filter(|&i| keep[i]).collect();
    // drop keys that repeat their predecessor and (if any) their successor: a constant stretch
    let mut out: Vec<usize> = Vec::new();
    for (qi, &i) in kept.iter().enumerate() {
        let same_prev = out.last().is_some_and(|&p| s[k[p]] == s[k[i]]);
        let same_next = kept.get(qi + 1).is_none_or(|&nx| s[k[nx]] == s[k[i]]);
        if same_prev && same_next {
            continue;
        }
        out.push(i);
    }
    (0..out.len())
        .map(|q| {
            let hold = out.get(q + 1).is_none_or(|&nx| s[k[nx]].sig != s[k[out[q]]].sig);
            (k[out[q]], hold)
        })
        .collect()
}

pub(super) fn emit_keys(s: &[Samp], keys: &[(usize, bool)], linear: bool, mk: &dyn Fn(&Samp) -> Value) -> Value {
    if keys.len() <= 1 {
        let f = keys.first().map_or(0, |k| k.0);
        return json!({"a": 0, "k": mk(&s[f])});
    }
    let arr: Vec<Value> = keys
        .iter()
        .enumerate()
        .map(|(q, &(f, hold))| {
            let mut o = json!({"t": f, "s": [mk(&s[f])]});
            if q + 1 < keys.len() {
                if hold || !linear {
                    o["h"] = json!(1);
                } else {
                    o["o"] = json!({"x": 0, "y": 0});
                    o["i"] = json!({"x": 1, "y": 1});
                }
            } else if !linear {
                o["h"] = json!(1);
            }
            o
        })
        .collect();
    json!({"a": 1, "k": arr})
}

/// Hold keyframes (exact at the kept frames) or, when `linear`, linear keyframes within `tol`, whichever
/// serialises smaller: linear keys carry easing handles, so they only pay off when they remove enough keys.
pub(super) fn pick_encoding(s: &[Samp], step: usize, tol: f64, linear: bool, mk: &dyn Fn(&Samp) -> Value) -> Value {
    let hold = emit_keys(s, &select_keys(s, step, 0.0, false), false, mk);
    if !linear {
        return hold;
    }
    let lin = emit_keys(s, &select_keys(s, step, tol, true), true, mk);
    if lin.to_string().len() < hold.to_string().len() {
        lin
    } else {
        hold
    }
}
