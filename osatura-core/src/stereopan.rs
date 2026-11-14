// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Equal-power stereo panning of a mono or stereo signal.
//!
//! Basis (public): W3C *Web Audio API*, section "Stereo Panning Algorithm" (StereoPannerNode), read through the public
//! history of the specification and the MDN description of the node. Spec: docs/clean-room/specs/03-audio-panoramare-stereo.md.
//!
//! The pan value `p` runs from -1 (full left) through 0 (centre) to 1 (full right); values outside are clamped, a value that is
//! not finite is an error. Gains are `left = cos(pi/2 * x)`, `right = sin(pi/2 * x)` with `x = (p + 1) / 2` for a mono input,
//! and for a stereo input `x = p + 1` when `p <= 0`, else `x = p`. Connected to `sound mix` and the exports through the
//! optional `pan` field of a sound track (`soundtrack::process_samples`), see docs/clean-room/specs/INTREBARI.md.

use std::f64::consts::FRAC_PI_2;

/// Clamps a pan value to `[-1, 1]`; `Err` when it is NaN or infinite.
pub fn clamp_pan(p: f64) -> Result<f64, String> {
    if p.is_finite() {
        Ok(p.clamp(-1.0, 1.0))
    } else {
        Err(format!("pan must be a finite number, got {p}"))
    }
}

/// `(left, right)` gains of a mono input.
pub fn mono_gains(p: f64) -> Result<(f64, f64), String> {
    let x = (clamp_pan(p)? + 1.0) / 2.0;
    Ok(((FRAC_PI_2 * x).cos(), (FRAC_PI_2 * x).sin()))
}

/// `(left, right)` gains of a stereo input (applied as described in [`pan_stereo_frame`]).
pub fn stereo_gains(p: f64) -> Result<(f64, f64), String> {
    let p = clamp_pan(p)?;
    let x = if p <= 0.0 { p + 1.0 } else { p };
    Ok(((FRAC_PI_2 * x).cos(), (FRAC_PI_2 * x).sin()))
}

/// One mono sample to a `(left, right)` pair.
pub fn pan_mono_frame(sample: f64, p: f64) -> Result<(f64, f64), String> {
    let (gl, gr) = mono_gains(p)?;
    Ok((sample * gl, sample * gr))
}

/// One stereo sample pair to a `(left, right)` pair.
/// `p <= 0`: left = in_left + in_right * gain_left, right = in_right * gain_right.
/// `p > 0`: left = in_left * gain_left, right = in_right + in_left * gain_right.
pub fn pan_stereo_frame(left: f64, right: f64, p: f64) -> Result<(f64, f64), String> {
    let (gl, gr) = stereo_gains(p)?;
    Ok(if clamp_pan(p)? <= 0.0 { (left + right * gl, right * gr) } else { (left * gl, right + left * gr) })
}

/// A mono buffer to interleaved stereo (`L R L R ...`) at a constant pan. An empty buffer gives an empty result.
pub fn pan_mono(input: &[f32], p: f64) -> Result<Vec<f32>, String> {
    let (gl, gr) = mono_gains(p)?;
    let mut out = Vec::with_capacity(input.len() * 2);
    for &s in input {
        out.push((f64::from(s) * gl) as f32);
        out.push((f64::from(s) * gr) as f32);
    }
    Ok(out)
}

/// Interleaved stereo to interleaved stereo at a constant pan. An odd length is an error.
pub fn pan_stereo(input: &[f32], p: f64) -> Result<Vec<f32>, String> {
    if !input.len().is_multiple_of(2) {
        return Err("an interleaved stereo buffer needs an even number of samples".into());
    }
    let mut out = Vec::with_capacity(input.len());
    for f in input.chunks_exact(2) {
        let (l, r) = pan_stereo_frame(f64::from(f[0]), f64::from(f[1]), p)?;
        out.push(l as f32);
        out.push(r as f32);
    }
    Ok(out)
}

/// Pan value at time `t` (seconds) from keys `(time, pan)` sorted by time: linear between keys, held before the first and after the last.
/// No keys = 0 (centre).
pub fn pan_at(keys: &[(f64, f64)], t: f64) -> f64 {
    match keys {
        [] => 0.0,
        [only] => only.1,
        _ => {
            if t <= keys[0].0 {
                return keys[0].1;
            }
            for w in keys.windows(2) {
                if t <= w[1].0 {
                    let span = w[1].0 - w[0].0;
                    return if span <= 0.0 { w[1].1 } else { w[0].1 + (w[1].1 - w[0].1) * (t - w[0].0) / span };
                }
            }
            keys[keys.len() - 1].1
        }
    }
}

/// A mono buffer panned along keys: the pan of each sample is `pan_at(keys, i / sample_rate)`.
pub fn pan_mono_keyed(input: &[f32], sample_rate: u32, keys: &[(f64, f64)]) -> Result<Vec<f32>, String> {
    if sample_rate == 0 {
        return Err("sample rate must be positive".into());
    }
    if keys.iter().any(|k| !k.0.is_finite() || !k.1.is_finite()) || keys.windows(2).any(|w| w[1].0 < w[0].0) {
        return Err("pan keys must be finite and sorted by time".into());
    }
    let mut out = Vec::with_capacity(input.len() * 2);
    for (i, &s) in input.iter().enumerate() {
        let (l, r) = pan_mono_frame(f64::from(s), pan_at(keys, i as f64 / f64::from(sample_rate)))?;
        out.push(l as f32);
        out.push(r as f32);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} vs {b}");
    }

    #[test]
    fn mono_cases() {
        let (l, r) = pan_mono_frame(1.0, -1.0).unwrap();
        close(l, 1.0);
        close(r, 0.0);
        let (l, r) = pan_mono_frame(1.0, 0.0).unwrap();
        close(l, std::f64::consts::FRAC_1_SQRT_2);
        close(r, std::f64::consts::FRAC_1_SQRT_2);
        close(l * l + r * r, 1.0);
        let (l, r) = pan_mono_frame(1.0, 1.0).unwrap();
        close(l, 0.0);
        close(r, 1.0);
    }

    #[test]
    fn stereo_cases() {
        let (l, r) = pan_stereo_frame(0.5, -0.25, 0.0).unwrap();
        close(l, 0.5);
        close(r, -0.25);
        let (l, r) = pan_stereo_frame(0.5, 0.5, -1.0).unwrap();
        close(l, 1.0);
        close(r, 0.0);
        let (l, r) = pan_stereo_frame(0.5, 0.5, 1.0).unwrap();
        close(l, 0.0);
        close(r, 1.0);
        let (l, r) = pan_stereo_frame(1.0, 0.0, 0.5).unwrap();
        close(l, std::f64::consts::FRAC_1_SQRT_2);
        close(r, std::f64::consts::FRAC_1_SQRT_2);
    }

    #[test]
    fn out_of_range_is_clamped_and_non_finite_is_an_error() {
        assert_eq!(pan_mono_frame(1.0, 2.5).unwrap(), pan_mono_frame(1.0, 1.0).unwrap());
        assert_eq!(pan_stereo_frame(0.3, 0.2, -9.0).unwrap(), pan_stereo_frame(0.3, 0.2, -1.0).unwrap());
        assert!(pan_mono_frame(1.0, f64::NAN).is_err());
        assert!(pan_stereo_frame(1.0, 1.0, f64::INFINITY).is_err());
        assert!(pan_mono(&[0.1], f64::NAN).is_err());
    }

    #[test]
    fn buffers() {
        assert!(pan_mono(&[], 0.3).unwrap().is_empty());
        assert!(pan_stereo(&[], 0.3).unwrap().is_empty());
        let out = pan_mono(&[1.0, -1.0], -1.0).unwrap();
        assert_eq!(out.len(), 4);
        close(f64::from(out[0]), 1.0);
        close(f64::from(out[2]), -1.0);
        let st = pan_stereo(&[0.5, -0.25, 0.1, 0.2], 0.0).unwrap();
        close(f64::from(st[0]), 0.5);
        close(f64::from(st[1]), -0.25);
        assert!(pan_stereo(&[0.5, 0.1, 0.2], 0.0).is_err());
        // nan in the signal propagates (the caller validates)
        assert!(pan_mono(&[f32::NAN], 0.0).unwrap()[0].is_nan());
    }

    #[test]
    fn keyed_pan_interpolates_linearly() {
        let keys = [(0.0, -1.0), (1.0, 1.0)];
        close(pan_at(&keys, 0.5), 0.0);
        close(pan_at(&keys, -3.0), -1.0);
        close(pan_at(&keys, 9.0), 1.0);
        close(pan_at(&[], 1.0), 0.0);
        // 4 samples at 4 Hz: pans -1, -0.5, 0, 0.5 -> first sample fully left
        let out = pan_mono_keyed(&[1.0; 4], 4, &keys).unwrap();
        close(f64::from(out[0]), 1.0);
        close(f64::from(out[1]), 0.0);
        close(f64::from(out[4]), std::f64::consts::FRAC_1_SQRT_2);
        assert!(pan_mono_keyed(&[1.0], 0, &keys).is_err());
        assert!(pan_mono_keyed(&[1.0], 4, &[(1.0, 0.0), (0.0, 0.0)]).is_err());
    }
}
