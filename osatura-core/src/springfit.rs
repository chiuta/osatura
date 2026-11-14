// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Draw-the-overshoot spring fitting: given the follow-through curve an animator wants, find
//! the spring frequency and damping whose response comes closest to it.
//!
//! The fit simulates exactly the same integrator the engine uses for baked springs
//! ([`SpringState::step`]), so the fitted numbers reproduce the drawn curve in the engine.

use crate::rig::SpringState;

#[derive(Clone, Debug, PartialEq)]
pub struct SpringFit {
    pub frequency: f32,
    pub damping: f32,
    /// Root-mean-square error against the normalised curve (0 = perfect, 1 = full step size).
    pub rms: f32,
}

fn response(freq: f32, damp: f32, fps: f32, n: usize) -> Vec<f32> {
    let mut s = SpringState::at_rest(0.0);
    let dt = 1.0 / fps;
    (0..n)
        .map(|_| {
            s.step(1.0, freq, damp, dt, 8);
            s.x
        })
        .collect()
}

fn rms(a: &[f32], b: &[f32]) -> f32 {
    (a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f32>() / a.len().max(1) as f32).sqrt()
}

/// `samples[i]` is the drawn value at frame `i + 1` after a sudden target change from
/// `samples`' own start (`start`) to its end (`last`). Needs at least 4 samples and a non-zero step.
pub fn fit_spring(start: f32, samples: &[f32], fps: f32) -> Result<SpringFit, String> {
    if samples.len() < 4 || (fps.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) {
        return Err("need at least 4 samples and a positive frame rate".into());
    }
    if samples.iter().any(|v| !v.is_finite()) || !start.is_finite() {
        return Err("samples must be finite".into());
    }
    // the target is where the curve settles; use the mean of the last quarter
    let tail = &samples[samples.len() * 3 / 4..];
    let end = tail.iter().sum::<f32>() / tail.len() as f32;
    let step = end - start;
    if step.abs() < 1e-6 {
        return Err("the curve does not move from its start value".into());
    }
    let norm: Vec<f32> = samples.iter().map(|v| (v - start) / step).collect();
    let n = norm.len();
    let mut best = SpringFit { frequency: 1.0, damping: 1.0, rms: f32::MAX };
    let (mut f_lo, mut f_hi, mut d_lo, mut d_hi) = (0.3f32, (fps * 0.4).min(30.0), 0.02f32, 2.0f32);
    for _ in 0..4 {
        let (fs, ds) = (14, 14);
        for i in 0..=fs {
            let f = f_lo * (f_hi / f_lo).powf(i as f32 / fs as f32);
            for j in 0..=ds {
                let d = d_lo + (d_hi - d_lo) * j as f32 / ds as f32;
                let e = rms(&response(f, d, fps, n), &norm);
                if e < best.rms {
                    best = SpringFit { frequency: f, damping: d, rms: e };
                }
            }
        }
        // zoom in around the best cell
        let (fw, dw) = ((f_hi / f_lo).powf(1.0 / 5.0), (d_hi - d_lo) / 5.0);
        f_lo = (best.frequency / fw).max(0.2);
        f_hi = (best.frequency * fw).min(30.0);
        d_lo = (best.damping - dw).max(0.0);
        d_hi = best.damping + dw;
    }
    Ok(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_known_parameters() {
        for (f, d) in [(3.0f32, 0.3f32), (5.0, 0.6), (1.5, 0.15)] {
            let r: Vec<f32> = response(f, d, 24.0, 48).iter().map(|x| 10.0 + 40.0 * x).collect();
            let fit = fit_spring(10.0, &r, 24.0).unwrap();
            assert!(fit.rms < 0.05, "({f},{d}) → {fit:?}");
            assert!((fit.frequency - f).abs() / f < 0.15, "({f},{d}) frequency {fit:?}");
            assert!((fit.damping - d).abs() < 0.12, "({f},{d}) damping {fit:?}");
        }
    }

    #[test]
    fn handles_negative_steps_and_hand_drawn_curves() {
        // a hand-drawn overshoot: goes past the target by ~20 % then settles
        let drawn = [0.0, 0.55, 1.05, 1.2, 1.15, 1.05, 0.98, 0.97, 1.0, 1.01, 1.0, 1.0];
        let down: Vec<f32> = drawn.iter().map(|v| 5.0 - v * 5.0).collect();
        let fit = fit_spring(5.0, &down, 24.0).unwrap();
        assert!(fit.damping < 0.8 && fit.damping > 0.05, "{fit:?}");
        assert!(fit.rms < 0.15, "{fit:?}");
    }

    #[test]
    fn rejects_degenerate_input() {
        assert!(fit_spring(0.0, &[0.0; 10], 24.0).is_err(), "flat curve");
        assert!(fit_spring(0.0, &[1.0, 1.0], 24.0).is_err(), "too short");
        assert!(fit_spring(0.0, &[0.0, f32::NAN, 1.0, 1.0], 24.0).is_err());
        assert!(fit_spring(0.0, &[0.0, 0.5, 1.0, 1.0], 0.0).is_err());
    }
}
