// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
use serde::{Deserialize, Serialize};

/// Interpolation applied on the segment that *starts* at a key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Ease {
    #[default]
    Linear,
    /// Hold the key's value until the next key.
    Step,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// CSS-style cubic-bezier easing curve through (0,0) and (1,1).
    CubicBezier {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    /// Catmull-Rom spline through neighbouring keys (smooth, no overshoot control).
    Smooth,
    /// Damped-spring settle: overshoots and rings, then lands exactly on the target.
    Spring {
        bounces: f32,
        decay: f32,
    },
    /// Ease-out with overshoot ("back"): cubic with start slope `3 + overshoot`, own derivation (see `back`).
    Back {
        overshoot: f32,
    },
    /// Bouncing ball landing: restitution series e = 0.6, three rebounds (own derivation, see `bounce`).
    Bounce,
}

impl Ease {
    /// Map normalised time `u ∈ [0,1]` to eased progress.
    /// (`Smooth` is handled by [`Track::eval`], which needs neighbouring keys; here it is linear.)
    pub fn apply(self, u: f32) -> f32 {
        let u = u.clamp(0.0, 1.0);
        match self {
            Ease::Linear | Ease::Smooth => u,
            Ease::Step => 0.0,
            Ease::EaseIn => u * u,
            Ease::EaseOut => 1.0 - (1.0 - u) * (1.0 - u),
            Ease::EaseInOut => u * u * (3.0 - 2.0 * u),
            Ease::CubicBezier { x1, y1, x2, y2 } => cubic_bezier(x1, y1, x2, y2, u),
            Ease::Spring { bounces, decay } => {
                let window = 1.0 - smooth01((u - 0.85) / 0.15);
                1.0 - (-decay * u).exp() * (std::f32::consts::TAU * bounces * u).cos() * window
            }
            Ease::Back { overshoot } => back(overshoot, u),
            Ease::Bounce => bounce(u),
        }
    }
}

/// Own derivation of the ease-out "back" curve (the family is common, the derivation below is ours).
///
/// Ask for a cubic `f(t) = a*t + b*t^2 + c*t^3` with `f(0) = 0`, `f(1) = 1` (lands exactly) and `f'(1) = 0` (arrives at
/// rest). Writing the start slope as `k = f'(0) = a`, the two remaining conditions give
/// `b = 3 - 2k`, `c = k - 2`. For `k = 3` this is `1 - (1-t)^3` (no overshoot); for `k > 3` the slope changes sign
/// before `t = 1` and the curve passes 1 at its peak, at `t* = k / (3 (k - 2))`. We parametrise the start slope as
/// `k = 3 + s` (`s` = the `overshoot` field, 0 = none, larger = more), which gives the monotone start the user sees.
/// The cubic with these boundary conditions is unique, so it coincides with the well-known back easing for the same
/// start slope; no table or empirical constant is involved. `s < -3` is clamped (k >= 0 keeps the start monotone).
fn back(s: f32, t: f32) -> f32 {
    let k = (3.0 + s).max(0.0);
    k * t + (3.0 - 2.0 * k) * t * t + (k - 2.0) * t * t * t
}

/// Coefficient of restitution of the bounce curve and number of rebounds (own choice).
const BOUNCE_E: f32 = 0.6;
const BOUNCE_N: u32 = 3;

/// Own physically derived bounce ("ball dropped onto the target"). Progress `y` is the height fallen, `y = 1` is the
/// floor. A ball falling from rest under constant gravity takes `T0 = 1` (time unit) to fall `y: 0 -> 1`, so
/// `y(tau) = tau^2` there. Each impact multiplies the speed by the restitution `e`: the k-th rebound is a parabola
/// that leaves and returns to `y = 1`, lasts `2 e^k` and rises to height `e^(2k)`, so
/// `y(tau) = 1 - e^(2k) * (1 - ((tau - c_k) / e^k)^2)` with its centre `c_k` at the middle of the arc. Landing instants
/// are `tau_0 = 1`, `tau_k = 1 + 2 (e + ... + e^k)`; the curve is continuous (every arc starts and ends at 1), never
/// exceeds 1 and, after `BOUNCE_N` rebounds, ends at `tau_N` with `y = 1` exactly. Documented points for
/// `e = 0.6`, `N = 3` (total time `tau_3 = 3.352`): first impact at `u = 1/3.352`, first peak `y = 0.64` at
/// `tau = 1.6`, second peak `0.8704` at `tau = 2.56`, third peak `0.953344` at `tau = 3.136`.
fn bounce(u: f32) -> f32 {
    let mut total = 1.0;
    let mut ek = 1.0;
    for _ in 0..BOUNCE_N {
        ek *= BOUNCE_E;
        total += 2.0 * ek;
    }
    let tau = u * total;
    if tau < 1.0 {
        return tau * tau;
    }
    let (mut start, mut ek) = (1.0f32, 1.0f32);
    for _ in 0..BOUNCE_N {
        ek *= BOUNCE_E;
        let end = start + 2.0 * ek;
        if tau <= end {
            let x = (tau - (start + ek)) / ek;
            return 1.0 - ek * ek * (1.0 - x * x);
        }
        start = end;
    }
    1.0
}

fn smooth01(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, u: f32) -> f32 {
    let bez = |a: f32, b: f32, t: f32| {
        let m = 1.0 - t;
        3.0 * a * m * m * t + 3.0 * b * m * t * t + t * t * t
    };
    // Solve x(t) = u by bisection (x is monotonic for x1,x2 in [0,1]).
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if bez(x1, x2, mid) < u {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    bez(y1, y2, 0.5 * (lo + hi))
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Key {
    pub frame: f32,
    pub value: f32,
    #[serde(default)]
    pub ease: Ease,
}

/// What a track does outside its first/last key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Extrapolate {
    /// Hold the end values.
    #[default]
    Hold,
    /// Continue along the end slope.
    Linear,
    /// Repeat the key range.
    Loop,
    /// Play forward then backward.
    PingPong,
    /// Repeat, adding the first→last delta each cycle (e.g. a walking x offset).
    LoopOffset,
}

/// A sorted list of keys for one scalar channel.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub keys: Vec<Key>,
    #[serde(default)]
    pub extrapolate: Extrapolate,
}

impl Track {
    pub fn new() -> Self {
        Track {
            keys: Vec::new(),
            extrapolate: Extrapolate::Hold,
        }
    }

    /// Insert or replace a key, keeping the list sorted by frame.
    pub fn set_key(&mut self, frame: f32, value: f32, ease: Ease) {
        let key = Key { frame, value, ease };
        match self.keys.binary_search_by(|k| k.frame.total_cmp(&frame)) {
            Ok(i) => self.keys[i] = key,
            Err(i) => self.keys.insert(i, key),
        }
    }

    pub fn remove_key(&mut self, frame: f32) -> bool {
        match self.keys.binary_search_by(|k| k.frame.total_cmp(&frame)) {
            Ok(i) => {
                self.keys.remove(i);
                true
            }
            Err(_) => false,
        }
    }

    pub fn with_keys(keys: &[(f32, f32, Ease)]) -> Self {
        let mut t = Track::new();
        for &(f, v, e) in keys {
            t.set_key(f, v, e);
        }
        t
    }

    pub fn looped(mut self, e: Extrapolate) -> Self {
        self.extrapolate = e;
        self
    }

    pub fn first_frame(&self) -> Option<f32> {
        self.keys.first().map(|k| k.frame)
    }
    pub fn last_frame(&self) -> Option<f32> {
        self.keys.last().map(|k| k.frame)
    }

    /// Value at `frame`, honouring the extrapolation mode.
    pub fn eval(&self, frame: f32) -> Option<f32> {
        let (first, last) = (self.keys.first()?, self.keys.last()?);
        let span = last.frame - first.frame;
        if frame >= first.frame && frame <= last.frame {
            return Some(self.eval_inside(frame));
        }
        if span <= 0.0 || self.keys.len() < 2 {
            return Some(first.value);
        }
        let before = frame < first.frame;
        Some(match self.extrapolate {
            Extrapolate::Hold => {
                if before {
                    first.value
                } else {
                    last.value
                }
            }
            Extrapolate::Linear => {
                if before {
                    let k1 = &self.keys[1];
                    let slope = (k1.value - first.value) / (k1.frame - first.frame);
                    first.value + slope * (frame - first.frame)
                } else {
                    let k0 = &self.keys[self.keys.len() - 2];
                    let slope = (last.value - k0.value) / (last.frame - k0.frame);
                    last.value + slope * (frame - last.frame)
                }
            }
            Extrapolate::Loop => {
                self.eval_inside(first.frame + (frame - first.frame).rem_euclid(span))
            }
            Extrapolate::PingPong => {
                let m = (frame - first.frame).rem_euclid(2.0 * span);
                self.eval_inside(first.frame + if m <= span { m } else { 2.0 * span - m })
            }
            Extrapolate::LoopOffset => {
                let n = ((frame - first.frame) / span).floor();
                self.eval_inside(frame - n * span) + n * (last.value - first.value)
            }
        })
    }

    fn eval_inside(&self, frame: f32) -> f32 {
        let keys = &self.keys;
        // First key with k.frame > frame; the segment is [i-1, i].
        let i = keys.partition_point(|k| k.frame <= frame);
        if i == 0 {
            return keys[0].value;
        }
        if i >= keys.len() {
            return keys[keys.len() - 1].value;
        }
        let (a, b) = (&keys[i - 1], &keys[i]);
        let span = b.frame - a.frame;
        let u = if span <= 0.0 {
            1.0
        } else {
            (frame - a.frame) / span
        };
        if let Ease::Smooth = a.ease {
            // Cubic Hermite with non-uniform Catmull-Rom tangents.
            let tan = |k: usize| -> f32 {
                let (p, n) = (k.saturating_sub(1), (k + 1).min(keys.len() - 1));
                let df = keys[n].frame - keys[p].frame;
                if df <= 0.0 {
                    0.0
                } else {
                    (keys[n].value - keys[p].value) / df
                }
            };
            let (m0, m1) = (tan(i - 1) * span, tan(i) * span);
            let (u2, u3) = (u * u, u * u * u);
            return (2.0 * u3 - 3.0 * u2 + 1.0) * a.value
                + (u3 - 2.0 * u2 + u) * m0
                + (-2.0 * u3 + 3.0 * u2) * b.value
                + (u3 - u2) * m1;
        }
        a.value + (b.value - a.value) * a.ease.apply(u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_interpolation_and_hold() {
        let t = Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 100.0, Ease::Linear)]);
        assert_eq!(t.eval(-5.0), Some(0.0));
        assert_eq!(t.eval(5.0), Some(50.0));
        assert_eq!(t.eval(99.0), Some(100.0));
        assert_eq!(Track::new().eval(1.0), None);
    }

    #[test]
    fn step_holds_until_next_key() {
        let t = Track::with_keys(&[(0.0, 1.0, Ease::Step), (10.0, 2.0, Ease::Linear)]);
        assert_eq!(t.eval(9.9), Some(1.0));
        assert_eq!(t.eval(10.0), Some(2.0));
    }

    #[test]
    fn keys_stay_sorted_replace_and_remove() {
        let mut t = Track::new();
        t.set_key(10.0, 1.0, Ease::Linear);
        t.set_key(0.0, 0.0, Ease::Linear);
        t.set_key(10.0, 5.0, Ease::Linear);
        assert_eq!(t.keys.len(), 2);
        assert_eq!(t.keys[0].frame, 0.0);
        assert_eq!(t.keys[1].value, 5.0);
        assert!(t.remove_key(10.0));
        assert!(!t.remove_key(10.0));
    }

    #[test]
    fn bezier_ease_hits_endpoints_and_is_monotone() {
        let e = Ease::CubicBezier {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        };
        assert!(e.apply(0.0).abs() < 1e-4);
        assert!((e.apply(1.0) - 1.0).abs() < 1e-4);
        let mut prev = 0.0;
        for i in 1..=20 {
            let v = e.apply(i as f32 / 20.0);
            assert!(v >= prev - 1e-5);
            prev = v;
        }
        assert!((e.apply(0.5) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn spring_back_and_bounce_land_exactly_and_overshoot() {
        let s = Ease::Spring {
            bounces: 2.5,
            decay: 5.0,
        };
        assert!(s.apply(0.0).abs() < 1e-5);
        assert!((s.apply(1.0) - 1.0).abs() < 1e-5);
        assert!(
            (0..=100).any(|i| s.apply(i as f32 / 100.0) > 1.02),
            "spring overshoots"
        );

        let b = Ease::Back { overshoot: 1.7 };
        assert!(b.apply(0.0).abs() < 1e-5 && (b.apply(1.0) - 1.0).abs() < 1e-5);
        assert!((0..=100).any(|i| b.apply(i as f32 / 100.0) > 1.05));

        let bo = Ease::Bounce;
        assert!(bo.apply(0.0).abs() < 1e-5 && (bo.apply(1.0) - 1.0).abs() < 1e-4);
        assert!(
            (0..=100).all(|i| bo.apply(i as f32 / 100.0) <= 1.0001),
            "bounce never exceeds target"
        );
    }

    #[test]
    fn back_is_the_documented_cubic() {
        // s = 0: no overshoot, equals 1 - (1-t)^3, monotone
        let b0 = Ease::Back { overshoot: 0.0 };
        let mut prev = 0.0;
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let v = b0.apply(t);
            assert!((v - (1.0 - (1.0 - t).powi(3))).abs() < 1e-5);
            assert!(v >= prev - 1e-6 && v <= 1.0 + 1e-6);
            prev = v;
        }
        // s = 1.7: start slope 3 + s, peak at t* = k / (3 (k - 2)), exact end value, monotone rise before the peak
        let s = 1.7f32;
        let k = 3.0 + s;
        let b = Ease::Back { overshoot: s };
        assert!(
            (b.apply(0.001) / 0.001 - k).abs() < 0.05,
            "start slope {}",
            b.apply(0.001) / 0.001
        );
        let tp = k / (3.0 * (k - 2.0));
        let peak = b.apply(tp);
        assert!(peak > 1.09 && peak < 1.12, "peak {peak}");
        assert!((0..=100).all(|i| b.apply(i as f32 / 100.0) <= peak + 1e-5));
        assert!((0..100)
            .filter(|&i| (i as f32 / 100.0) < tp - 0.01)
            .all(|i| b.apply((i + 1) as f32 / 100.0) >= b.apply(i as f32 / 100.0)));
        assert_eq!(b.apply(1.0), 1.0);
        // more overshoot gives a higher peak; very negative values are clamped, never NaN
        let hi = Ease::Back { overshoot: 4.0 };
        assert!(
            (0..=100)
                .map(|i| hi.apply(i as f32 / 100.0))
                .fold(0.0, f32::max)
                > peak
        );
        assert!(Ease::Back { overshoot: -50.0 }.apply(0.5).is_finite());
    }

    #[test]
    fn bounce_hits_the_documented_points() {
        let total = 3.352f32;
        let bo = Ease::Bounce;
        assert_eq!(bo.apply(0.0), 0.0);
        assert_eq!(bo.apply(1.0), 1.0);
        // free fall: y = tau^2 until the first impact at tau = 1
        assert!((bo.apply(0.5 / total) - 0.25).abs() < 1e-4);
        assert!((bo.apply(1.0 / total) - 1.0).abs() < 1e-4);
        // rebound peaks 1 - e^(2k) at the middle of each arc
        assert!((bo.apply(1.6 / total) - 0.64).abs() < 1e-4);
        assert!((bo.apply(2.56 / total) - 0.8704).abs() < 1e-4);
        assert!((bo.apply(3.136 / total) - 0.953344).abs() < 1e-4);
        // impacts in between touch 1; continuity and bound over a fine sweep
        assert!((bo.apply(2.2 / total) - 1.0).abs() < 1e-4);
        let mut prev = bo.apply(0.0);
        for i in 1..=2000 {
            let v = bo.apply(i as f32 / 2000.0);
            assert!(
                v <= 1.0 + 1e-6 && (v - prev).abs() < 0.05,
                "jump at {i}: {prev} -> {v}"
            );
            prev = v;
        }
    }

    #[test]
    fn smooth_passes_through_keys_without_overshoot_on_monotone_data() {
        let t = Track::with_keys(&[
            (0.0, 0.0, Ease::Smooth),
            (10.0, 10.0, Ease::Smooth),
            (20.0, 30.0, Ease::Smooth),
            (30.0, 30.0, Ease::Linear),
        ]);
        assert!((t.eval(10.0).unwrap() - 10.0).abs() < 1e-4);
        assert!((t.eval(20.0).unwrap() - 30.0).abs() < 1e-4);
        let mid = t.eval(15.0).unwrap();
        assert!(mid > 10.0 && mid < 30.0);
        // Smooth is C1: no kink at an interior key.
        let (a, b) = (t.eval(9.99).unwrap(), t.eval(10.01).unwrap());
        let (c, d) = (t.eval(9.98).unwrap(), t.eval(10.02).unwrap());
        assert!(((b - a) - (d - c) / 2.0).abs() < 0.01);
    }

    #[test]
    fn extrapolation_modes() {
        let base = Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 10.0, Ease::Linear)]);
        assert_eq!(base.eval(25.0), Some(10.0));
        let lin = base.clone().looped(Extrapolate::Linear);
        assert_eq!(lin.eval(15.0), Some(15.0));
        assert_eq!(lin.eval(-5.0), Some(-5.0));
        let lp = base.clone().looped(Extrapolate::Loop);
        assert!((lp.eval(13.0).unwrap() - 3.0).abs() < 1e-5);
        assert!((lp.eval(-3.0).unwrap() - 7.0).abs() < 1e-5);
        let pp = base.clone().looped(Extrapolate::PingPong);
        assert!((pp.eval(13.0).unwrap() - 7.0).abs() < 1e-5);
        assert!((pp.eval(23.0).unwrap() - 3.0).abs() < 1e-5);
        let lo = base.looped(Extrapolate::LoopOffset);
        assert!((lo.eval(13.0).unwrap() - 13.0).abs() < 1e-5);
        assert!((lo.eval(-3.0).unwrap() - -3.0).abs() < 1e-5);
    }
}
