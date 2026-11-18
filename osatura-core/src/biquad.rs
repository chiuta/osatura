// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Second-order (biquad) audio filters: low-pass, high-pass, band-pass, notch, all-pass, peaking, low shelf, high shelf.
//!
//! Basis (public): W3C Working Group Note *Audio EQ Cookbook* (8 June 2021, after R. Bristow-Johnson), <https://www.w3.org/TR/audio-eq-cookbook>
//! (the `Q` forms of the coefficient formulas; the bandwidth and shelf-slope forms are not offered). Spec:
//! docs/clean-room/specs/04-audio-filtru-biquad.md.
//!
//! `H(z) = (b0 + b1 z^-1 + b2 z^-2) / (a0 + a1 z^-1 + a2 z^-2)` evaluated as the difference equation with `a0` normalised away and zero
//! initial state, in `f64`. Connected to `sound mix` and the exports through the optional `filters`
//! list of a sound track (`soundtrack::process_samples`), see docs/clean-room/specs/INTREBARI.md.

use std::f64::consts::PI;

/// Safety cap of the project on `Q` (the source sets none).
pub const MAX_Q: f64 = 1000.0;

/// The filter shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterKind {
    LowPass,
    HighPass,
    /// Band-pass with 0 dB peak gain.
    BandPass,
    Notch,
    AllPass,
    /// Peaking EQ ("bell").
    Peaking,
    LowShelf,
    HighShelf,
}

impl FilterKind {
    /// The canonical name accepted by [`FilterKind::parse`].
    pub fn name(self) -> &'static str {
        match self {
            FilterKind::LowPass => "lowpass",
            FilterKind::HighPass => "highpass",
            FilterKind::BandPass => "bandpass",
            FilterKind::Notch => "notch",
            FilterKind::AllPass => "allpass",
            FilterKind::Peaking => "peaking",
            FilterKind::LowShelf => "lowshelf",
            FilterKind::HighShelf => "highshelf",
        }
    }

    /// `lowpass`, `highpass`, `bandpass`, `notch`, `allpass`, `peaking`, `lowshelf`, `highshelf` (case-insensitive; `-` and `_` ignored).
    pub fn parse(s: &str) -> Result<FilterKind, String> {
        let t: String = s.chars().filter(|c| *c != '-' && *c != '_').collect::<String>().to_ascii_lowercase();
        Ok(match t.as_str() {
            "lowpass" => FilterKind::LowPass,
            "highpass" => FilterKind::HighPass,
            "bandpass" => FilterKind::BandPass,
            "notch" => FilterKind::Notch,
            "allpass" => FilterKind::AllPass,
            "peaking" | "bell" => FilterKind::Peaking,
            "lowshelf" => FilterKind::LowShelf,
            "highshelf" => FilterKind::HighShelf,
            _ => return Err(format!("unknown filter type '{s}' (lowpass, highpass, bandpass, notch, allpass, peaking, lowshelf, highshelf)")),
        })
    }
}

/// Parameters of one filter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiquadSpec {
    pub kind: FilterKind,
    /// Centre / corner frequency, Hz, `0 < f0 < fs / 2`.
    pub f0: f64,
    /// Quality factor, `0 < q <= MAX_Q`.
    pub q: f64,
    /// Gain in dB (peaking and shelves only).
    pub gain_db: f64,
}

/// Normalised coefficients (`a0 == 1`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coeffs {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
}

/// Checks the parameters against the sample rate.
pub fn check(fs: f64, s: &BiquadSpec) -> Result<(), String> {
    if !(fs.is_finite() && fs > 0.0) {
        return Err(format!("sample rate must be a positive finite number, got {fs}"));
    }
    if !(s.f0.is_finite() && s.q.is_finite() && s.gain_db.is_finite()) {
        return Err("filter parameters must be finite".into());
    }
    if s.f0 <= 0.0 || s.f0 >= fs / 2.0 {
        return Err(format!("frequency {} Hz must be above 0 and below half the sample rate ({} Hz)", s.f0, fs / 2.0));
    }
    if s.q <= 0.0 {
        return Err(format!("Q must be positive, got {}", s.q));
    }
    if s.q > MAX_Q {
        return Err(format!("Q {} is above the limit {MAX_Q}", s.q));
    }
    Ok(())
}

/// Advisory messages: a frequency above 0.45 of the sample rate may be numerically unstable.
pub fn warnings(fs: f64, s: &BiquadSpec) -> Vec<String> {
    if s.f0 > 0.45 * fs {
        vec![format!("frequency {} Hz is above 0.45 x the sample rate; the result may be numerically unstable", s.f0)]
    } else {
        vec![]
    }
}

/// Coefficients of the filter, normalised by `a0`.
pub fn coefficients(fs: f64, s: &BiquadSpec) -> Result<Coeffs, String> {
    check(fs, s)?;
    let w0 = 2.0 * PI * s.f0 / fs;
    let (sn, c) = (w0.sin(), w0.cos());
    let alpha = sn / (2.0 * s.q);
    let a = 10f64.powf(s.gain_db / 40.0);
    let (b, ar) = match s.kind {
        FilterKind::LowPass => ([(1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0], [1.0 + alpha, -2.0 * c, 1.0 - alpha]),
        FilterKind::HighPass => ([(1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0], [1.0 + alpha, -2.0 * c, 1.0 - alpha]),
        FilterKind::BandPass => ([alpha, 0.0, -alpha], [1.0 + alpha, -2.0 * c, 1.0 - alpha]),
        FilterKind::Notch => ([1.0, -2.0 * c, 1.0], [1.0 + alpha, -2.0 * c, 1.0 - alpha]),
        FilterKind::AllPass => ([1.0 - alpha, -2.0 * c, 1.0 + alpha], [1.0 + alpha, -2.0 * c, 1.0 - alpha]),
        FilterKind::Peaking => ([1.0 + alpha * a, -2.0 * c, 1.0 - alpha * a], [1.0 + alpha / a, -2.0 * c, 1.0 - alpha / a]),
        FilterKind::LowShelf => {
            let t = 2.0 * a.sqrt() * alpha;
            (
                [a * ((a + 1.0) - (a - 1.0) * c + t), 2.0 * a * ((a - 1.0) - (a + 1.0) * c), a * ((a + 1.0) - (a - 1.0) * c - t)],
                [(a + 1.0) + (a - 1.0) * c + t, -2.0 * ((a - 1.0) + (a + 1.0) * c), (a + 1.0) + (a - 1.0) * c - t],
            )
        }
        FilterKind::HighShelf => {
            let t = 2.0 * a.sqrt() * alpha;
            (
                [a * ((a + 1.0) + (a - 1.0) * c + t), -2.0 * a * ((a - 1.0) + (a + 1.0) * c), a * ((a + 1.0) + (a - 1.0) * c - t)],
                [(a + 1.0) - (a - 1.0) * c + t, 2.0 * ((a - 1.0) - (a + 1.0) * c), (a + 1.0) - (a - 1.0) * c - t],
            )
        }
    };
    let k = ar[0];
    Ok(Coeffs { b0: b[0] / k, b1: b[1] / k, b2: b[2] / k, a1: ar[1] / k, a2: ar[2] / k })
}

/// Running filter state (one per channel).
#[derive(Clone, Debug)]
pub struct Biquad {
    c: Coeffs,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl Biquad {
    /// A filter with zero initial state.
    pub fn new(c: Coeffs) -> Biquad {
        Biquad { c, x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }

    /// One sample in, one sample out.
    pub fn process(&mut self, x: f64) -> f64 {
        let c = &self.c;
        let y = c.b0 * x + c.b1 * self.x1 + c.b2 * self.x2 - c.a1 * self.y1 - c.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Filters a buffer in `f64` without clamping.
pub fn filter_f64(samples: &[f64], fs: f64, s: &BiquadSpec) -> Result<Vec<f64>, String> {
    let mut f = Biquad::new(coefficients(fs, s)?);
    Ok(samples.iter().map(|&x| f.process(x)).collect())
}

/// Filters a mono buffer of `f32` samples; the result is limited to `[-1, 1]`. A non-finite output sample is an error (the input held NaN or infinity,
/// or the filter blew up).
pub fn filter(samples: &[f32], fs: f64, s: &BiquadSpec) -> Result<Vec<f32>, String> {
    let mut f = Biquad::new(coefficients(fs, s)?);
    let mut out = Vec::with_capacity(samples.len());
    for (i, &x) in samples.iter().enumerate() {
        let y = f.process(f64::from(x));
        if !y.is_finite() {
            return Err(format!("the filter output is not finite at sample {i}"));
        }
        out.push(y.clamp(-1.0, 1.0) as f32);
    }
    Ok(out)
}

/// Filters interleaved multi-channel audio, each channel separately.
pub fn filter_interleaved(samples: &[f32], channels: usize, fs: f64, s: &BiquadSpec) -> Result<Vec<f32>, String> {
    if channels == 0 || !samples.len().is_multiple_of(channels) {
        return Err("the sample count does not match the channel count".into());
    }
    let c = coefficients(fs, s)?;
    let mut states = vec![Biquad::new(c); channels];
    let mut out = Vec::with_capacity(samples.len());
    for (i, &x) in samples.iter().enumerate() {
        let y = states[i % channels].process(f64::from(x));
        if !y.is_finite() {
            return Err(format!("the filter output is not finite at sample {}", i / channels));
        }
        out.push(y.clamp(-1.0, 1.0) as f32);
    }
    Ok(out)
}

/// `|H(e^{jw})|` for the frequency `w` in radians per sample.
pub fn magnitude(c: &Coeffs, w: f64) -> f64 {
    let (c1, s1, c2, s2) = (w.cos(), w.sin(), (2.0 * w).cos(), (2.0 * w).sin());
    let nr = c.b0 + c.b1 * c1 + c.b2 * c2;
    let ni = -(c.b1 * s1 + c.b2 * s2);
    let dr = 1.0 + c.a1 * c1 + c.a2 * c2;
    let di = -(c.a1 * s1 + c.a2 * s2);
    ((nr * nr + ni * ni) / (dr * dr + di * di)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FS: f64 = 48_000.0;
    const Q: f64 = std::f64::consts::FRAC_1_SQRT_2;

    fn spec(kind: FilterKind, q: f64, gain_db: f64) -> BiquadSpec {
        BiquadSpec { kind, f0: FS / 4.0, q, gain_db }
    }

    fn near(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "{a} vs {b}");
    }

    #[test]
    fn low_pass_coefficients_and_impulse_response() {
        let c = coefficients(FS, &spec(FilterKind::LowPass, Q, 0.0)).unwrap();
        near(c.b0, 0.292893, 1e-5);
        near(c.b1, 0.585786, 1e-5);
        near(c.b2, 0.292893, 1e-5);
        near(c.a1, 0.0, 1e-9);
        near(c.a2, 0.171573, 1e-5);
        let mut x = vec![0.0; 6];
        x[0] = 1.0;
        let y = filter_f64(&x, FS, &spec(FilterKind::LowPass, Q, 0.0)).unwrap();
        for (got, want) in y.iter().zip([0.292893, 0.585786, 0.242641, -0.100505, -0.041631, 0.017244]) {
            near(*got, want, 1e-5);
        }
    }

    #[test]
    fn low_pass_passes_dc() {
        let y = filter_f64(&vec![1.0; 400], FS, &spec(FilterKind::LowPass, Q, 0.0)).unwrap();
        near(y[399], 1.0, 1e-5);
    }

    #[test]
    fn high_pass_nyquist_and_dc() {
        let s = spec(FilterKind::HighPass, Q, 0.0);
        let alt: Vec<f64> = (0..400).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        let y = filter_f64(&alt, FS, &s).unwrap();
        near(y[398].abs(), 1.0, 1e-5);
        near(y[399].abs(), 1.0, 1e-5);
        let c = coefficients(FS, &s).unwrap();
        near(magnitude(&c, PI), 1.0, 1e-9);
        let dc = filter_f64(&vec![1.0; 400], FS, &s).unwrap();
        near(dc[399], 0.0, 1e-5);
    }

    #[test]
    fn peaking_filter() {
        let db = 6.0206;
        let s = spec(FilterKind::Peaking, 1.0, db);
        let c = coefficients(FS, &s).unwrap();
        near(c.b0, 1.261204, 1e-5);
        near(c.b1, 0.0, 1e-9);
        near(c.b2, 0.216388, 1e-5);
        near(c.a2, 0.477592, 1e-5);
        near(magnitude(&c, 2.0 * PI * s.f0 / FS), 2.0, 1e-4);
    }

    #[test]
    fn peaking_with_zero_gain_is_identity() {
        let x: Vec<f64> = (0..200).map(|i| ((i * 37 % 101) as f64 / 50.0) - 1.0).collect();
        let y = filter_f64(&x, FS, &spec(FilterKind::Peaking, 1.0, 0.0)).unwrap();
        for (a, b) in x.iter().zip(&y) {
            near(*a, *b, 1e-12);
        }
    }

    #[test]
    fn low_shelf_gains() {
        let c = coefficients(FS, &spec(FilterKind::LowShelf, Q, 6.0206)).unwrap();
        near(magnitude(&c, 1e-9), 2.0, 1e-4);
        near(magnitude(&c, PI), 1.0, 1e-4);
        let h = coefficients(FS, &spec(FilterKind::HighShelf, Q, 6.0206)).unwrap();
        near(magnitude(&h, 1e-9), 1.0, 1e-4);
        near(magnitude(&h, PI), 2.0, 1e-4);
    }

    #[test]
    fn band_notch_allpass_shapes() {
        let w0 = PI / 2.0;
        let bp = coefficients(FS, &spec(FilterKind::BandPass, 2.0, 0.0)).unwrap();
        near(magnitude(&bp, w0), 1.0, 1e-9);
        let n = coefficients(FS, &spec(FilterKind::Notch, 2.0, 0.0)).unwrap();
        near(magnitude(&n, w0), 0.0, 1e-9);
        let ap = coefficients(FS, &spec(FilterKind::AllPass, 2.0, 0.0)).unwrap();
        for w in [0.1, 1.0, 2.5] {
            near(magnitude(&ap, w), 1.0, 1e-9);
        }
    }

    #[test]
    fn errors_and_warnings() {
        let mut s = spec(FilterKind::LowPass, Q, 0.0);
        s.f0 = FS / 2.0;
        assert!(coefficients(FS, &s).is_err());
        s.f0 = -10.0;
        assert!(coefficients(FS, &s).is_err());
        s.f0 = 1000.0;
        s.q = 0.0;
        assert!(coefficients(FS, &s).is_err());
        s.q = MAX_Q + 1.0;
        assert!(coefficients(FS, &s).is_err());
        s.q = f64::NAN;
        assert!(coefficients(FS, &s).is_err());
        s.q = 1.0;
        assert!(coefficients(0.0, &s).is_err());
        s.gain_db = f64::INFINITY;
        assert!(coefficients(FS, &s).is_err());
        let mut hi = spec(FilterKind::LowPass, Q, 0.0);
        hi.f0 = 0.46 * FS;
        assert!(coefficients(FS, &hi).is_ok());
        assert_eq!(warnings(FS, &hi).len(), 1);
        assert!(warnings(FS, &spec(FilterKind::LowPass, Q, 0.0)).is_empty());
    }

    #[test]
    fn buffers_are_limited_and_validated() {
        let s = spec(FilterKind::Peaking, 1.0, 20.0);
        let y = filter(&[0.9; 50], FS, &s).unwrap();
        assert!(y.iter().all(|v| (-1.0..=1.0).contains(v)));
        assert!(filter(&[0.1, f32::NAN], FS, &s).is_err());
        assert!(filter(&[], FS, &s).unwrap().is_empty());
        let two = filter_interleaved(&[0.5, -0.5, 0.5, -0.5], 2, FS, &spec(FilterKind::LowPass, Q, 0.0)).unwrap();
        near(f64::from(two[0]), -f64::from(two[1]), 1e-6);
        assert!(filter_interleaved(&[0.5; 3], 2, FS, &s).is_err());
        assert_eq!(FilterKind::parse("Low-Pass").unwrap(), FilterKind::LowPass);
        assert!(FilterKind::parse("comb").is_err());
    }
}
