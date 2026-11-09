// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Graph-editor maths for keyed tracks: curve sampling for display, value-range auto-fit, and
//! Bezier tangent handles (get / set / move with constraints) on top of the per-segment easing
//! that [`Ease`] already provides.
//!
//! Provenance note: textbook maths only. Hermite <-> Bezier control-point conversion, the
//! non-uniform Catmull-Rom tangent, and the Fritsch-Carlson monotone-cubic limiter are all
//! standard numerical-analysis material; the handle model (a segment's easing is a cubic Bezier
//! in normalised segment space) is this project's own and follows from `Ease::CubicBezier`.
//!
//! Handle model. A segment runs from key `i` to key `i+1`. Its shape is the cubic Bezier
//! `P0=(a.t,a.v)`, `P1=out(a)`, `P2=in(b)`, `P3=(b.t,b.v)`. The stored form is
//! `Ease::CubicBezier{x1,y1,x2,y2}` on the first key, normalised to the segment box, so the
//! absolute handle position is `(a.t + x1*span, a.v + y1*dv)`. A segment whose eased value
//! does not change (`dv == 0`) cannot carry vertical handle offsets (the normalisation divides
//! by `dv`); only horizontal handle movement is stored there.

use crate::anim::{Ease, Key, Track};
use serde::{Deserialize, Serialize};

const EPS: f32 = 1e-6;
const MAX_SAMPLES: usize = 4096;

/// Which tangent family automatic smoothing uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TangentKind {
    /// Non-uniform Catmull-Rom (what `Ease::Smooth` evaluates); may overshoot.
    CatmullRom,
    /// Fritsch-Carlson monotone cubic: never overshoots the neighbouring key values.
    Monotone,
}

/// Which end of a key a handle belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// Handle of the segment arriving at the key.
    In,
    /// Handle of the segment leaving the key.
    Out,
}

/// Constraint applied while a handle moves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleMode {
    /// Only the dragged handle changes.
    #[default]
    Free,
    /// The opposite handle takes the same slope (C1 continuity), keeping its own time length.
    Aligned,
    /// Both handles of the key are set from the monotone tangent; the position is ignored.
    Auto,
}

/// Absolute handle position (frame, value).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct HandlePos {
    pub t: f32,
    pub v: f32,
}

/// Both handles of one key; `None` where the segment does not exist or is not Bezier-representable
/// (step, spring, back, bounce).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct KeyHandles {
    pub index: usize,
    #[serde(rename = "in")]
    pub inn: Option<HandlePos>,
    pub out: Option<HandlePos>,
}

// ---------------------------------------------------------------- sampling

/// Polyline of `(frame, value)` for display: `n` uniform samples over `[f0, f1]` plus every key
/// frame inside the range; step segments get a vertical riser so the corner is drawn. Empty when
/// the track has no keys, the range is empty, or `n < 2`. Deterministic.
pub fn sample_track(t: &Track, f0: f32, f1: f32, n: usize) -> Vec<(f32, f32)> {
    if t.keys.is_empty() || n < 2 || (f1.partial_cmp(&f0) != Some(std::cmp::Ordering::Greater)) || !f0.is_finite() || !f1.is_finite() {
        return Vec::new();
    }
    let n = n.min(MAX_SAMPLES);
    let mut times: Vec<f32> = (0..n).map(|i| if i + 1 == n { f1 } else { f0 + (f1 - f0) * i as f32 / (n - 1) as f32 }).collect();
    times.extend(t.keys.iter().map(|k| k.frame).filter(|&f| f > f0 && f < f1));
    times.sort_by(|a, b| a.total_cmp(b));
    times.dedup();
    let mut out: Vec<(f32, f32)> = Vec::with_capacity(times.len() + 4);
    for f in times {
        // a step segment holds its value up to the next key: draw the riser first
        if let Some(i) = t.keys.iter().position(|k| k.frame == f) {
            if i > 0 && matches!(t.keys[i - 1].ease, Ease::Step) {
                out.push((f, t.keys[i - 1].value));
            }
        }
        out.push((f, t.eval(f).unwrap_or(0.0)));
    }
    out
}

/// `(lo, hi)` value range of the track over `[f0, f1]` (samples and keys inside), widened by
/// `margin` (fraction of the span) on each side; a flat curve gets a unit-high window.
pub fn value_range(t: &Track, f0: f32, f1: f32, n: usize, margin: f32) -> Option<(f32, f32)> {
    let pts = sample_track(t, f0, f1, n);
    if pts.is_empty() {
        return None;
    }
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for &(_, v) in &pts {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    for k in t.keys.iter().filter(|k| k.frame >= f0 && k.frame <= f1) {
        lo = lo.min(k.value);
        hi = hi.max(k.value);
    }
    if (hi - lo).partial_cmp(&(1e-6)) != Some(std::cmp::Ordering::Greater) {
        return Some((lo - 0.5, hi + 0.5));
    }
    let pad = (hi - lo) * margin.max(0.0);
    Some((lo - pad, hi + pad))
}

// ---------------------------------------------------------------- tangents

fn catmull_tangent(keys: &[Key], k: usize) -> f32 {
    let (p, n) = (k.saturating_sub(1), (k + 1).min(keys.len() - 1));
    let df = keys[n].frame - keys[p].frame;
    if df <= 0.0 {
        0.0
    } else {
        (keys[n].value - keys[p].value) / df
    }
}

/// Slope (value per frame) at every key. Catmull-Rom matches `Ease::Smooth` exactly; Monotone is
/// the Fritsch-Carlson construction: secant slopes, zero at local extrema, then every segment's
/// `(alpha, beta) = (m0/d, m1/d)` is pulled back into the radius-3 disc.
pub fn tangents(keys: &[Key], kind: TangentKind) -> Vec<f32> {
    let n = keys.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![0.0];
    }
    match kind {
        TangentKind::CatmullRom => (0..n).map(|k| catmull_tangent(keys, k)).collect(),
        TangentKind::Monotone => {
            let d: Vec<f32> = keys.windows(2).map(|w| (w[1].value - w[0].value) / (w[1].frame - w[0].frame)).collect();
            let mut m = vec![0.0f32; n];
            m[0] = d[0];
            m[n - 1] = d[n - 2];
            for k in 1..n - 1 {
                m[k] = if d[k - 1] * d[k] <= 0.0 { 0.0 } else { 0.5 * (d[k - 1] + d[k]) };
            }
            for k in 0..n - 1 {
                if d[k].abs() < EPS {
                    m[k] = 0.0;
                    m[k + 1] = 0.0;
                    continue;
                }
                let (a, b) = (m[k] / d[k], m[k + 1] / d[k]);
                if a < 0.0 {
                    m[k] = 0.0;
                }
                if b < 0.0 {
                    m[k + 1] = 0.0;
                }
                let (a, b) = (m[k] / d[k], m[k + 1] / d[k]);
                let s = a * a + b * b;
                if s > 9.0 {
                    let tau = 3.0 / s.sqrt();
                    m[k] = tau * a * d[k];
                    m[k + 1] = tau * b * d[k];
                }
            }
            m
        }
    }
}

/// Hermite segment (slopes `m0`, `m1`) as normalised Bezier parameters `[x1, y1, x2, y2]`.
/// `None` for a flat segment with non-zero slopes (a bump cannot be expressed relative to `dv = 0`).
pub fn hermite_params(a: &Key, b: &Key, m0: f32, m1: f32) -> Option<[f32; 4]> {
    let span = b.frame - a.frame;
    let dv = b.value - a.value;
    if dv.abs() < EPS {
        return if m0.abs() < EPS && m1.abs() < EPS { Some([1.0 / 3.0, 0.0, 2.0 / 3.0, 0.0]) } else { None };
    }
    Some([1.0 / 3.0, m0 * span / (3.0 * dv), 2.0 / 3.0, 1.0 - m1 * span / (3.0 * dv)])
}

/// Normalised Bezier parameters of the segment starting at key `i`, if it can be expressed.
/// Exact for linear, ease-in/out/in-out (a cubic Bezier with linear x reproduces them) and
/// cubic-bezier; the equivalent curve for `Smooth`. `None` for step / spring / back / bounce,
/// for the last key, and for a flat Smooth segment with slopes.
pub fn segment_params(t: &Track, i: usize) -> Option<[f32; 4]> {
    let a = t.keys.get(i)?;
    let b = t.keys.get(i + 1)?;
    const T: f32 = 1.0 / 3.0;
    match a.ease {
        Ease::Linear => Some([T, T, 2.0 * T, 2.0 * T]),
        Ease::EaseIn => Some([T, 0.0, 2.0 * T, T]),
        Ease::EaseOut => Some([T, 2.0 * T, 2.0 * T, 1.0]),
        Ease::EaseInOut => Some([T, 0.0, 2.0 * T, 1.0]),
        Ease::CubicBezier { x1, y1, x2, y2 } => Some([x1, y1, x2, y2]),
        Ease::Smooth => hermite_params(a, b, catmull_tangent(&t.keys, i), catmull_tangent(&t.keys, i + 1)),
        _ => None,
    }
}

fn abs_handles(a: &Key, b: &Key, p: [f32; 4]) -> (HandlePos, HandlePos) {
    let (span, dv) = (b.frame - a.frame, b.value - a.value);
    (HandlePos { t: a.frame + p[0] * span, v: a.value + p[1] * dv }, HandlePos { t: a.frame + p[2] * span, v: a.value + p[3] * dv })
}

/// Handles of every key (for drawing). Key `i`'s `out` belongs to segment `i`, its `in` to segment `i-1`.
pub fn handles(t: &Track) -> Vec<KeyHandles> {
    let n = t.keys.len();
    (0..n)
        .map(|i| {
            let out = segment_params(t, i).map(|p| abs_handles(&t.keys[i], &t.keys[i + 1], p).0);
            let inn = if i > 0 { segment_params(t, i - 1).map(|p| abs_handles(&t.keys[i - 1], &t.keys[i], p).1) } else { None };
            KeyHandles { index: i, inn, out }
        })
        .collect()
}

fn set_params(t: &mut Track, i: usize, p: [f32; 4]) {
    t.keys[i].ease = Ease::CubicBezier { x1: p[0].clamp(0.0, 1.0), y1: p[1], x2: p[2].clamp(0.0, 1.0), y2: p[3] };
}

/// Rewrite the segment starting at `i` as an explicit Bezier (so its handles become editable).
pub fn convert_to_bezier(t: &mut Track, i: usize) -> Result<(), String> {
    let p = segment_params(t, i).ok_or_else(|| format!("segment {i} cannot be expressed as a Bezier (step, spring, back, bounce, last key or flat smooth)"))?;
    set_params(t, i, p);
    Ok(())
}

/// Segment index and "is this the out (P1) end" for a handle of key `key`.
fn seg_of(t: &Track, key: usize, side: Side) -> Result<(usize, bool), String> {
    if key >= t.keys.len() {
        return Err(format!("key {key} out of range"));
    }
    match side {
        Side::Out if key + 1 < t.keys.len() => Ok((key, true)),
        Side::In if key > 0 => Ok((key - 1, false)),
        Side::Out => Err("the last key has no outgoing handle".into()),
        Side::In => Err("the first key has no incoming handle".into()),
    }
}

/// Write an absolute handle position into segment `seg`; returns the clamped absolute position.
fn put_handle(t: &mut Track, seg: usize, out_end: bool, pos: HandlePos) -> Result<HandlePos, String> {
    let mut p = segment_params(t, seg).ok_or_else(|| format!("segment {seg} has no editable handles (convert it to a Bezier first)"))?;
    let (a, b) = (t.keys[seg], t.keys[seg + 1]);
    let (span, dv) = (b.frame - a.frame, b.value - a.value);
    let x = ((pos.t - a.frame) / span).clamp(0.0, 1.0);
    let y = if dv.abs() < EPS { 0.0 } else { (pos.v - a.value) / dv };
    if out_end {
        p[0] = x;
        p[1] = y;
    } else {
        p[2] = x;
        p[3] = y;
    }
    set_params(t, seg, p);
    let (o, i) = abs_handles(&a, &b, p);
    Ok(if out_end { o } else { i })
}

/// Move one handle (absolute `pos`) with a constraint. Errors leave the track unchanged.
pub fn set_handle(t: &mut Track, key: usize, side: Side, pos: HandlePos, mode: HandleMode) -> Result<(), String> {
    if !pos.t.is_finite() || !pos.v.is_finite() {
        return Err("handle position must be finite".into());
    }
    let (seg, out_end) = seg_of(t, key, side)?;
    let backup = t.clone();
    let r = (|| {
        match mode {
            HandleMode::Auto => auto_handles(t, key),
            HandleMode::Free => put_handle(t, seg, out_end, pos).map(|_| ()),
            HandleMode::Aligned => {
                let placed = put_handle(t, seg, out_end, pos)?;
                let k = t.keys[key];
                let dt = placed.t - k.frame;
                if dt.abs() < EPS {
                    return Ok(()); // vertical handle: slope undefined, leave the opposite side alone
                }
                let slope = (placed.v - k.value) / dt;
                // opposite handle: other segment, same key
                let opp = match side {
                    Side::Out if key > 0 => Some((key - 1, false)),
                    Side::In if key + 1 < t.keys.len() => Some((key, true)),
                    _ => None,
                };
                if let Some((oseg, oout)) = opp {
                    if let Some(p) = segment_params(t, oseg) {
                        let (a, b) = (t.keys[oseg], t.keys[oseg + 1]);
                        let (o, i) = abs_handles(&a, &b, p);
                        let cur = if oout { o } else { i };
                        let len = (cur.t - k.frame).abs();
                        let sgn = if oout { 1.0 } else { -1.0 };
                        put_handle(t, oseg, oout, HandlePos { t: k.frame + sgn * len, v: k.value + slope * sgn * len })?;
                    }
                }
                Ok(())
            }
        }
    })();
    if r.is_err() {
        *t = backup;
    }
    r
}

/// Set both handles of key `key` from its monotone tangent (thirds of the segment spans).
pub fn auto_handles(t: &mut Track, key: usize) -> Result<(), String> {
    if key >= t.keys.len() {
        return Err(format!("key {key} out of range"));
    }
    let m = tangents(&t.keys, TangentKind::Monotone)[key];
    if key + 1 < t.keys.len() {
        let (a, b) = (t.keys[key], t.keys[key + 1]);
        let mut p = segment_params(t, key).unwrap_or([1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0]);
        let dv = b.value - a.value;
        p[0] = 1.0 / 3.0;
        p[1] = if dv.abs() < EPS { 0.0 } else { m * (b.frame - a.frame) / (3.0 * dv) };
        set_params(t, key, p);
    }
    if key > 0 {
        let (a, b) = (t.keys[key - 1], t.keys[key]);
        let mut p = segment_params(t, key - 1).unwrap_or([1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0]);
        let dv = b.value - a.value;
        p[2] = 2.0 / 3.0;
        p[3] = if dv.abs() < EPS { 0.0 } else { 1.0 - m * (b.frame - a.frame) / (3.0 * dv) };
        set_params(t, key - 1, p);
    }
    Ok(())
}

/// Auto-smooth the segments that start at the given key indices. `CatmullRom` sets `Ease::Smooth`;
/// `Monotone` bakes the Fritsch-Carlson tangents into explicit Bezier handles (no overshoot).
/// Returns the number of segments changed.
pub fn smooth_segments(t: &mut Track, starts: &[usize], kind: TangentKind) -> usize {
    let m = tangents(&t.keys, kind);
    let mut n = 0;
    for &i in starts {
        if i + 1 >= t.keys.len() {
            continue;
        }
        match kind {
            TangentKind::CatmullRom => t.keys[i].ease = Ease::Smooth,
            TangentKind::Monotone => match hermite_params(&t.keys[i], &t.keys[i + 1], m[i], m[i + 1]) {
                Some(p) => set_params(t, i, p),
                None => t.keys[i].ease = Ease::Linear,
            },
        }
        n += 1;
    }
    n
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    fn tr(k: &[(f32, f32, Ease)]) -> Track {
        Track::with_keys(k)
    }

    #[test]
    fn sampling_includes_keys_risers_and_is_deterministic() {
        let t = tr(&[(0.0, 0.0, Ease::Step), (10.0, 5.0, Ease::Linear), (20.0, 15.0, Ease::Linear)]);
        let s = sample_track(&t, 0.0, 20.0, 5);
        assert_eq!(s, sample_track(&t, 0.0, 20.0, 5));
        // uniform 0,5,10,15,20 plus riser at 10
        assert!(s.contains(&(10.0, 0.0)) && s.contains(&(10.0, 5.0)));
        assert!(s.windows(2).all(|w| w[0].0 <= w[1].0));
        assert_eq!(s.first(), Some(&(0.0, 0.0)));
        assert_eq!(s.last(), Some(&(20.0, 15.0)));
        assert!(sample_track(&Track::new(), 0.0, 1.0, 4).is_empty());
        assert!(sample_track(&t, 5.0, 5.0, 4).is_empty() && sample_track(&t, 0.0, 9.0, 1).is_empty());
        assert!(sample_track(&t, 0.0, 20.0, usize::MAX).len() <= MAX_SAMPLES + 8);
    }

    #[test]
    fn key_frames_inside_range_are_sampled_even_between_samples() {
        let t = tr(&[(0.0, 0.0, Ease::Linear), (3.0, 9.0, Ease::Linear), (10.0, 0.0, Ease::Linear)]);
        let s = sample_track(&t, 0.0, 10.0, 2); // only the end points uniformly
        assert!(s.contains(&(3.0, 9.0)));
    }

    #[test]
    fn range_fit_pads_and_handles_flat() {
        let t = tr(&[(0.0, 2.0, Ease::Linear), (10.0, 12.0, Ease::Linear)]);
        let (lo, hi) = value_range(&t, 0.0, 10.0, 16, 0.1).unwrap();
        assert!((lo - 1.0).abs() < 1e-5 && (hi - 13.0).abs() < 1e-5);
        let flat = tr(&[(0.0, 4.0, Ease::Linear), (10.0, 4.0, Ease::Linear)]);
        assert_eq!(value_range(&flat, 0.0, 10.0, 8, 0.1), Some((3.5, 4.5)));
        assert_eq!(value_range(&Track::new(), 0.0, 1.0, 8, 0.1), None);
        // a range that only sees the middle of an overshooting curve
        let (lo2, hi2) = value_range(&t, 4.0, 6.0, 8, 0.0).unwrap();
        assert!((lo2 - 6.0).abs() < 1e-4 && (hi2 - 8.0).abs() < 1e-4);
    }

    #[test]
    fn segment_params_reproduce_the_builtin_eases_exactly() {
        for e in [Ease::Linear, Ease::EaseIn, Ease::EaseOut, Ease::EaseInOut, Ease::Smooth] {
            let t = tr(&[(0.0, 0.0, e), (10.0, 10.0, Ease::Linear), (20.0, 4.0, Ease::Linear)]);
            let mut u = t.clone();
            convert_to_bezier(&mut u, 0).unwrap();
            assert!(matches!(u.keys[0].ease, Ease::CubicBezier { .. }));
            for i in 0..=40 {
                let f = i as f32 * 0.25;
                let (a, b) = (t.eval(f).unwrap(), u.eval(f).unwrap());
                assert!((a - b).abs() < 2e-3, "{e:?} at {f}: {a} vs {b}");
            }
        }
        let t = tr(&[(0.0, 0.0, Ease::Step), (10.0, 1.0, Ease::Linear)]);
        assert!(segment_params(&t, 0).is_none() && segment_params(&t, 1).is_none());
        let mut u = t.clone();
        assert!(convert_to_bezier(&mut u, 0).is_err());
        assert_eq!(u, t);
    }

    #[test]
    fn handles_are_one_third_points_for_linear() {
        let t = tr(&[(0.0, 0.0, Ease::Linear), (9.0, 6.0, Ease::Linear)]);
        let h = handles(&t);
        assert_eq!(h[0].out, Some(HandlePos { t: 3.0, v: 2.0 }));
        assert_eq!(h[1].inn, Some(HandlePos { t: 6.0, v: 4.0 }));
        assert!(h[0].inn.is_none() && h[1].out.is_none());
    }

    #[test]
    fn free_handle_move_writes_normalised_bezier_and_clamps_time() {
        let mut t = tr(&[(0.0, 0.0, Ease::Linear), (10.0, 10.0, Ease::Linear)]);
        set_handle(&mut t, 0, Side::Out, HandlePos { t: 4.0, v: 0.0 }, HandleMode::Free).unwrap();
        match t.keys[0].ease {
            Ease::CubicBezier { x1, y1, x2, y2 } => {
                assert!((x1 - 0.4).abs() < 1e-6 && y1.abs() < 1e-6);
                assert!((x2 - 2.0 / 3.0).abs() < 1e-6 && (y2 - 2.0 / 3.0).abs() < 1e-6, "other handle untouched");
            }
            e => panic!("{e:?}"),
        }
        // time beyond the segment clamps to the segment
        set_handle(&mut t, 1, Side::In, HandlePos { t: -50.0, v: 3.0 }, HandleMode::Free).unwrap();
        let h = handles(&t);
        assert_eq!(h[1].inn.unwrap().t, 0.0);
        assert!((h[1].inn.unwrap().v - 3.0).abs() < 1e-5);
        // errors: ends, bad numbers, non-bezier segments; track unchanged
        let before = t.clone();
        assert!(set_handle(&mut t, 1, Side::Out, HandlePos { t: 1.0, v: 1.0 }, HandleMode::Free).is_err());
        assert!(set_handle(&mut t, 0, Side::In, HandlePos { t: 1.0, v: 1.0 }, HandleMode::Free).is_err());
        assert!(set_handle(&mut t, 0, Side::Out, HandlePos { t: f32::NAN, v: 1.0 }, HandleMode::Free).is_err());
        assert!(set_handle(&mut t, 9, Side::Out, HandlePos { t: 1.0, v: 1.0 }, HandleMode::Free).is_err());
        assert_eq!(t, before);
        let mut s = tr(&[(0.0, 0.0, Ease::Step), (10.0, 10.0, Ease::Linear)]);
        assert!(set_handle(&mut s, 0, Side::Out, HandlePos { t: 1.0, v: 1.0 }, HandleMode::Free).is_err());
    }

    #[test]
    fn aligned_mode_equalises_slopes_and_keeps_opposite_length() {
        let mut t = tr(&[(0.0, 0.0, Ease::Linear), (10.0, 10.0, Ease::Linear), (30.0, 0.0, Ease::Linear)]);
        let before_len = (10.0 - handles(&t)[1].inn.unwrap().t).abs();
        // drag the out handle of the middle key up-right: slope (14-10)/(15-10)... pick t=14, v=14
        set_handle(&mut t, 1, Side::Out, HandlePos { t: 14.0, v: 14.0 }, HandleMode::Aligned).unwrap();
        let h = handles(&t);
        let (o, i) = (h[1].out.unwrap(), h[1].inn.unwrap());
        let so = (o.v - 10.0) / (o.t - 10.0);
        let si = (10.0 - i.v) / (10.0 - i.t);
        assert!((so - 1.0).abs() < 1e-4, "{so}");
        assert!((so - si).abs() < 1e-4, "slopes {so} vs {si}");
        assert!(((10.0 - i.t).abs() - before_len).abs() < 1e-4, "opposite keeps its time length");
        // the end key has no opposite handle: aligned degrades to free
        let mut u = tr(&[(0.0, 0.0, Ease::Linear), (10.0, 10.0, Ease::Linear)]);
        set_handle(&mut u, 0, Side::Out, HandlePos { t: 5.0, v: 0.0 }, HandleMode::Aligned).unwrap();
        assert_eq!(handles(&u)[0].out.unwrap(), HandlePos { t: 5.0, v: 0.0 });
    }

    #[test]
    fn catmull_rom_tangents_match_smooth_ease() {
        let keys = Track::with_keys(&[(0.0, 0.0, Ease::Smooth), (10.0, 10.0, Ease::Smooth), (25.0, 4.0, Ease::Smooth), (30.0, 9.0, Ease::Linear)]).keys;
        let m = tangents(&keys, TangentKind::CatmullRom);
        assert!((m[1] - (4.0 - 0.0) / 25.0).abs() < 1e-6);
        assert!((m[0] - 1.0).abs() < 1e-6 && (m[3] - 1.0).abs() < 1e-6);
        // baked to bezier it evaluates like Smooth
        let t = Track { keys: keys.clone(), ..Track::new() };
        let mut u = t.clone();
        for i in 0..3 {
            convert_to_bezier(&mut u, i).unwrap();
        }
        for i in 0..=60 {
            let f = i as f32 * 0.5;
            assert!((t.eval(f).unwrap() - u.eval(f).unwrap()).abs() < 3e-3, "at {f}");
        }
    }

    #[test]
    fn monotone_cubic_does_not_overshoot() {
        // Data with a sharp step-like rise: Catmull-Rom overshoots, monotone must not.
        let pts = [(0.0, 0.0), (10.0, 0.0), (12.0, 10.0), (14.0, 10.0), (30.0, 11.0), (31.0, 25.0), (50.0, 25.0)];
        let keys: Vec<(f32, f32, Ease)> = pts.iter().map(|&(f, v)| (f, v, Ease::Smooth)).collect();
        let mut cr = Track::with_keys(&keys);
        let mut mono = cr.clone();
        let n = mono.keys.len();
        smooth_segments(&mut mono, &(0..n - 1).collect::<Vec<_>>(), TangentKind::Monotone);
        smooth_segments(&mut cr, &(0..n - 1).collect::<Vec<_>>(), TangentKind::CatmullRom);
        let mut cr_over = false;
        let mut prev = f32::NEG_INFINITY;
        for i in 0..=2000 {
            let f = i as f32 * 0.025;
            let v = mono.eval(f).unwrap();
            assert!(v >= prev - 2e-3, "monotone data must give monotone curve at {f}: {v} < {prev}");
            assert!((-2e-3..=25.002).contains(&v), "overshoot {v} at {f}");
            // never outside the two neighbouring keys of its segment
            let j = mono.keys.partition_point(|k| k.frame <= f).clamp(1, n - 1);
            let (lo, hi) = (mono.keys[j - 1].value.min(mono.keys[j].value), mono.keys[j - 1].value.max(mono.keys[j].value));
            assert!(v >= lo - 2e-3 && v <= hi + 2e-3, "segment overshoot at {f}: {v} not in [{lo},{hi}]");
            prev = v;
            let c = cr.eval(f).unwrap();
            if c > hi + 0.05 || c < lo - 0.05 {
                cr_over = true;
            }
        }
        assert!(cr_over, "test data must actually make Catmull-Rom overshoot");
    }

    #[test]
    fn monotone_tangents_are_zero_at_extrema_and_limited() {
        let keys = Track::with_keys(&[(0.0, 0.0, Ease::Linear), (1.0, 5.0, Ease::Linear), (2.0, 0.0, Ease::Linear), (3.0, 0.1, Ease::Linear)]).keys;
        let m = tangents(&keys, TangentKind::Monotone);
        assert_eq!(m[1], 0.0, "local maximum");
        assert_eq!(m[2], 0.0, "local minimum");
        let flat = Track::with_keys(&[(0.0, 1.0, Ease::Linear), (1.0, 1.0, Ease::Linear), (2.0, 3.0, Ease::Linear)]).keys;
        let mf = tangents(&flat, TangentKind::Monotone);
        assert_eq!((mf[0], mf[1]), (0.0, 0.0));
        assert_eq!(tangents(&[], TangentKind::Monotone), Vec::<f32>::new());
    }

    #[test]
    fn auto_mode_sets_both_handles_to_thirds_with_equal_slope() {
        let mut t = tr(&[(0.0, 0.0, Ease::Linear), (10.0, 4.0, Ease::Linear), (20.0, 20.0, Ease::Linear)]);
        set_handle(&mut t, 1, Side::Out, HandlePos { t: 0.0, v: 0.0 }, HandleMode::Auto).unwrap();
        let h = handles(&t)[1];
        let (i, o) = (h.inn.unwrap(), h.out.unwrap());
        assert!((i.t - (10.0 - 10.0 / 3.0)).abs() < 1e-4 && (o.t - (10.0 + 10.0 / 3.0)).abs() < 1e-4);
        let (si, so) = ((4.0 - i.v) / (10.0 - i.t), (o.v - 4.0) / (o.t - 10.0));
        assert!((si - so).abs() < 1e-4);
        let m = tangents(&t.keys, TangentKind::Monotone)[1];
        assert!((so - m).abs() < 1e-3);
    }

    #[test]
    fn smooth_segments_only_touches_requested_and_flat_falls_back_to_linear() {
        let mut t = tr(&[(0.0, 0.0, Ease::Step), (5.0, 5.0, Ease::Step), (10.0, 5.0, Ease::Step), (15.0, 9.0, Ease::Linear)]);
        let n = smooth_segments(&mut t, &[1, 2, 3], TangentKind::Monotone);
        assert_eq!(n, 2, "the last key starts no segment");
        assert!(matches!(t.keys[0].ease, Ease::Step), "untouched");
        assert!(matches!(t.keys[1].ease, Ease::CubicBezier { .. }));
        assert!(matches!(t.keys[2].ease, Ease::CubicBezier { .. }));
        let mut u = tr(&[(0.0, 0.0, Ease::Step), (5.0, 5.0, Ease::Step)]);
        smooth_segments(&mut u, &[0], TangentKind::CatmullRom);
        assert!(matches!(u.keys[0].ease, Ease::Smooth));
    }
}
