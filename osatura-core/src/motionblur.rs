// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Motion blur by temporal sampling: the picture of a frame is the average of several renders of the scene at moments spread over the
//! exposure interval of a shutter.
//!
//! Basis: the classic method of averaging renders over the exposure time (Potmesil and Chakravarty 1983; Korein and Badler 1983; Cook,
//! Porter and Carpenter 1984, stratified temporal sampling), and the cinema convention that the exposure time is `angle / 360` of the
//! frame duration. None of the papers was read for this implementation; it follows docs/clean-room/specs/11-estompare-de-miscare-prin-acumulare.md
//! and its arithmetic acceptance tests.
//!
//! The samples are at the middles of `n` equal sub-intervals of `[t + phase, t + phase + E]` (deterministic, no random numbers), with
//! `E = angle / 360 / fps`. A shutter angle of 0 renders once at `t + phase`. Images are averaged with premultiplied alpha, in the space the
//! renderer already produced (the project's compositing space), in exact integer arithmetic, and rounded once at the end (halves up).
//!
//! Used by the CLI through `--shutter` on `render`, `frames --png` and `export` to `.gif`, `.apng`, `.mp4` and `.webm` (see
//! docs/clean-room/specs/INTREBARI.md); `render-hdr`, the studio and the other exports are not connected.

use crate::engine::Engine;
use crate::raster::{render_rgba, RasterOptions};

/// Largest number of samples per frame (cost cap of the project).
pub const MAX_SAMPLES: u32 = 256;

/// The shutter: angle in degrees, number of samples and phase (start of the exposure relative to the frame time, seconds).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shutter {
    pub angle_deg: f64,
    pub samples: u32,
    pub phase_s: f64,
}

impl Shutter {
    /// A shutter that starts at the frame time.
    pub fn new(angle_deg: f64, samples: u32) -> Shutter {
        Shutter { angle_deg, samples, phase_s: 0.0 }
    }

    /// Checks angle in `[0, 360]`, `1 <= samples <= MAX_SAMPLES` and a finite phase.
    pub fn validate(&self) -> Result<(), String> {
        if !self.angle_deg.is_finite() || !(0.0..=360.0).contains(&self.angle_deg) {
            return Err(format!("the shutter angle must be in [0, 360] degrees, got {}", self.angle_deg));
        }
        if self.samples < 1 || self.samples > MAX_SAMPLES {
            return Err(format!("the number of samples must be 1..={MAX_SAMPLES}, got {}", self.samples));
        }
        if !self.phase_s.is_finite() {
            return Err("the shutter phase must be finite".into());
        }
        Ok(())
    }

    /// Exposure time `E = angle / 360 / fps` in seconds.
    pub fn exposure_s(&self, fps: f64) -> Result<f64, String> {
        self.validate()?;
        if !fps.is_finite() || fps <= 0.0 {
            return Err(format!("fps must be positive, got {fps}"));
        }
        Ok(self.angle_deg / 360.0 / fps)
    }

    /// The sample moments for the frame at time `t` (seconds). With angle 0 the single moment is `t + phase`.
    pub fn sample_times(&self, t: f64, fps: f64) -> Result<Vec<f64>, String> {
        let e = self.exposure_s(fps)?;
        if !t.is_finite() {
            return Err("the frame time must be finite".into());
        }
        if self.angle_deg == 0.0 {
            return Ok(vec![t + self.phase_s]);
        }
        let n = f64::from(self.samples);
        Ok((0..self.samples).map(|i| t + self.phase_s + (f64::from(i) + 0.5) / n * e).collect())
    }
}

/// Averages straight-alpha RGBA8 images of equal length: colour is the alpha-weighted mean, alpha the plain mean, both rounded once
/// to the nearest integer (halves up).
pub fn accumulate(images: &[&[u8]]) -> Result<Vec<u8>, String> {
    let first = images.first().ok_or("no images to average")?;
    if first.len() % 4 != 0 || images.iter().any(|i| i.len() != first.len()) {
        return Err("the images must be RGBA with the same size".into());
    }
    if images.len() > MAX_SAMPLES as usize {
        return Err(format!("more than {MAX_SAMPLES} images"));
    }
    let n = images.len() as u64;
    let mut out = vec![0u8; first.len()];
    for (px, o) in out.chunks_exact_mut(4).enumerate() {
        let (mut sc, mut sa) = ([0u64; 3], 0u64);
        for img in images {
            let p = &img[px * 4..px * 4 + 4];
            let a = u64::from(p[3]);
            for c in 0..3 {
                sc[c] += u64::from(p[c]) * a;
            }
            sa += a;
        }
        if sa == 0 {
            continue;
        }
        for c in 0..3 {
            o[c] = ((2 * sc[c] + sa) / (2 * sa)).min(255) as u8;
        }
        o[3] = ((2 * sa + n) / (2 * n)).min(255) as u8;
    }
    Ok(out)
}

/// Renders a frame with motion blur through `render(time_seconds) -> (width, height, straight RGBA8)`. Times are limited to
/// `[0, duration_s]` when a duration is given. With angle 0 the single render is returned unchanged (byte for byte).
pub fn render_blurred<F>(shutter: &Shutter, t: f64, fps: f64, duration_s: Option<f64>, mut render: F) -> Result<(u32, u32, Vec<u8>), String>
where
    F: FnMut(f64) -> Result<(u32, u32, Vec<u8>), String>,
{
    let clamp = |x: f64| match duration_s {
        Some(d) if d.is_finite() && d >= 0.0 => x.clamp(0.0, d),
        _ => x,
    };
    let times: Vec<f64> = shutter.sample_times(t, fps)?.into_iter().map(clamp).collect();
    if times.len() == 1 {
        return render(times[0]);
    }
    let mut size: Option<(u32, u32)> = None;
    let mut imgs: Vec<Vec<u8>> = Vec::with_capacity(times.len());
    for &s in &times {
        let (w, h, px) = render(s)?;
        if *size.get_or_insert((w, h)) != (w, h) {
            return Err("the renders have different sizes".into());
        }
        imgs.push(px);
    }
    let refs: Vec<&[u8]> = imgs.iter().map(Vec::as_slice).collect();
    let (w, h) = size.ok_or("no samples")?;
    Ok((w, h, accumulate(&refs)?))
}

/// Motion-blurred render of `frame` (a frame number of the project) with the raster renderer.
pub fn render_engine(engine: &Engine, frame: f32, opts: &RasterOptions, shutter: &Shutter) -> Result<(u32, u32, Vec<u8>), String> {
    crate::raster::check_project_size(&engine.project, opts.scale)?;
    let p = &engine.project;
    let fps = f64::from(p.fps);
    let t = f64::from(frame) / fps;
    let duration = f64::from(p.frames) / fps;
    render_blurred(shutter, t, fps, Some(duration), |s| {
        let f = (s * fps) as f32;
        engine.evaluate_checked(f)?;
        Ok(render_rgba(engine, f, opts))
    })
}

/// Most renders (frames x samples) one blurred sequence may take; a sequence over it is refused before the first render.
pub const MAX_SEQUENCE_RENDERS: u64 = 100_000;

/// Checks the shutter and the total cost of blurring `frames` frames.
pub fn check_sequence_cost(frames: u32, shutter: &Shutter) -> Result<(), String> {
    shutter.validate()?;
    let n = if shutter.angle_deg == 0.0 { 1 } else { u64::from(shutter.samples) };
    let total = u64::from(frames) * n;
    if total > MAX_SEQUENCE_RENDERS {
        return Err(format!("{frames} frames with {n} samples each are {total} renders (limit {MAX_SEQUENCE_RENDERS}): lower --samples or shorten the animation"));
    }
    Ok(())
}

/// Every frame of the project with motion blur, as `(width, height, straight RGBA frames)`; the same layout as the unblurred sequence
/// writers use, so the existing GIF / APNG encoders take it unchanged. With angle 0 the frames are the plain renders.
pub fn frames_rgba(engine: &Engine, scale: f32, shutter: &Shutter) -> Result<(u32, u32, Vec<Vec<u8>>), String> {
    crate::raster::check_sequence(&engine.project, scale)?;
    check_sequence_cost(engine.project.frames, shutter)?;
    let opts = RasterOptions { scale, show_bones: false, transparent: false };
    let (mut w, mut h) = (0, 0);
    let mut out = Vec::with_capacity(engine.project.frames as usize);
    for f in 0..engine.project.frames {
        let (fw, fh, d) = render_engine(engine, f as f32, &opts, shutter)?;
        (w, h) = (fw, fh);
        out.push(d);
    }
    Ok((w, h, out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} vs {b}");
    }

    #[test]
    fn sample_moments() {
        let s = Shutter::new(180.0, 4);
        let t = 10.0 / 24.0;
        close(s.exposure_s(24.0).unwrap(), 0.020_833_3);
        let v = s.sample_times(t, 24.0).unwrap();
        for (got, off) in v.iter().zip([0.002_604_2, 0.007_812_5, 0.013_020_8, 0.018_229_2]) {
            close(got - t, off);
        }
    }

    #[test]
    fn one_sample_full_shutter_and_centred_phase() {
        close(Shutter::new(360.0, 1).sample_times(0.0, 25.0).unwrap()[0], 0.02);
        let mut s = Shutter::new(360.0, 2);
        s.phase_s = -s.exposure_s(25.0).unwrap() / 2.0;
        let v = s.sample_times(1.0, 25.0).unwrap();
        close(v[0], 1.0 - 0.01);
        close(v[1], 1.0 + 0.01);
        // the last sample of a full shutter does not reach the next frame time
        assert!(*Shutter::new(360.0, 8).sample_times(0.0, 25.0).unwrap().last().unwrap() < 0.04);
        // angle 0: one render at t + phase whatever N is
        let mut z = Shutter::new(0.0, 7);
        z.phase_s = 0.5;
        assert_eq!(z.sample_times(2.0, 24.0).unwrap(), vec![2.5]);
    }

    #[test]
    fn invalid_parameters() {
        assert!(Shutter::new(400.0, 4).validate().is_err());
        assert!(Shutter::new(-1.0, 4).validate().is_err());
        assert!(Shutter::new(180.0, 0).validate().is_err());
        assert!(Shutter::new(180.0, MAX_SAMPLES + 1).validate().is_err());
        assert!(Shutter::new(f64::NAN, 4).validate().is_err());
        assert!(Shutter::new(180.0, 4).sample_times(0.0, 0.0).is_err());
        assert!(Shutter::new(180.0, 4).sample_times(0.0, -3.0).is_err());
        assert!(Shutter::new(180.0, 4).sample_times(f64::NAN, 24.0).is_err());
        let mut s = Shutter::new(180.0, 4);
        s.phase_s = f64::INFINITY;
        assert!(s.validate().is_err());
    }

    #[test]
    fn mean_of_opaque_greys() {
        let imgs: Vec<Vec<u8>> = [0u8, 254, 0, 254].iter().map(|&g| vec![g, g, g, 255]).collect();
        let refs: Vec<&[u8]> = imgs.iter().map(Vec::as_slice).collect();
        assert_eq!(accumulate(&refs).unwrap(), vec![127, 127, 127, 255]);
    }

    #[test]
    fn premultiplied_average_keeps_the_colour_of_the_visible_sample() {
        let mut imgs = vec![vec![255u8, 0, 0, 255]];
        imgs.extend(std::iter::repeat_n(vec![0u8, 0, 0, 0], 3));
        let refs: Vec<&[u8]> = imgs.iter().map(Vec::as_slice).collect();
        // colour (255, 0, 0) with alpha 0.25 = 63.75 -> 64; a plain mean of colours would give red 64
        assert_eq!(accumulate(&refs).unwrap(), vec![255, 0, 0, 64]);
    }

    #[test]
    fn identical_images_average_to_themselves_and_errors() {
        let img: Vec<u8> = (0..64u8).map(|i| i.wrapping_mul(37)).collect();
        let refs: Vec<&[u8]> = vec![&img; 8];
        assert_eq!(accumulate(&refs).unwrap(), img.iter().enumerate().map(|(i, v)| if i % 4 == 3 || img[i / 4 * 4 + 3] != 0 { *v } else { 0 }).collect::<Vec<u8>>());
        assert!(accumulate(&[]).is_err());
        assert!(accumulate(&[&[0u8; 4][..], &[0u8; 8][..]]).is_err());
        assert!(accumulate(&[&[0u8; 5][..]]).is_err());
    }

    #[test]
    fn render_blurred_calls_the_renderer_at_the_sample_times() {
        let mut seen = Vec::new();
        let s = Shutter::new(360.0, 2);
        let r = render_blurred(&s, 1.0, 2.0, None, |t| {
            seen.push(t);
            let g = if t < 1.25 { 0u8 } else { 254 };
            Ok((1, 1, vec![g, g, g, 255]))
        })
        .unwrap();
        assert_eq!(seen, vec![1.125, 1.375]);
        assert_eq!(r, (1, 1, vec![127, 127, 127, 255]));
        // angle 0: the render is returned unchanged
        let r = render_blurred(&Shutter::new(0.0, 9), 1.0, 2.0, None, |t| Ok((1, 1, vec![(t * 10.0) as u8, 1, 2, 3]))).unwrap();
        assert_eq!(r.2, vec![10, 1, 2, 3]);
        // times are limited to [0, duration]
        let mut seen = Vec::new();
        let _ = render_blurred(&Shutter::new(360.0, 2), 0.0, 2.0, Some(0.2), |t| {
            seen.push(t);
            Ok((1, 1, vec![0, 0, 0, 255]))
        });
        assert_eq!(seen, vec![0.125, 0.2]);
        // errors from the renderer and size changes are reported
        assert!(render_blurred(&s, 0.0, 2.0, None, |_| Err("boom".to_string())).is_err());
        let mut n = 0;
        assert!(render_blurred(&s, 0.0, 2.0, None, |_| {
            n += 1;
            Ok((n, 1, vec![0; 4 * n as usize]))
        })
        .is_err());
    }

    fn static_engine() -> Engine {
        use crate::path::VPath;
        use crate::project::{Layer, Project};
        use crate::style::Paint;
        let mut p = Project::new("s", 16.0, 16.0, 24.0, 10);
        let mut r = VPath::rect("r", 2.0, 2.0, 12.0, 12.0);
        r.fill = Some(Paint::solid("#3366cc"));
        p.layers = vec![Layer::vector("g", vec![r])];
        Engine::new(p).unwrap()
    }

    #[test]
    fn engine_level_identity_cases() {
        let e = static_engine();
        let opts = RasterOptions::default();
        let plain = render_rgba(&e, 3.0, &opts);
        // angle 0, any N: byte for byte
        assert_eq!(render_engine(&e, 3.0, &opts, &Shutter::new(0.0, 5)).unwrap(), plain);
        // a static scene: the average of identical renders
        assert_eq!(render_engine(&e, 3.0, &opts, &Shutter::new(180.0, 8)).unwrap(), plain);
    }

    #[test]
    fn engine_level_blur_of_a_moving_scene() {
        let e = Engine::new(crate::demo::project()).unwrap();
        let opts = RasterOptions::default();
        let plain = render_rgba(&e, 6.0, &opts);
        let blurred = render_engine(&e, 6.0, &opts, &Shutter::new(360.0, 4)).unwrap();
        assert_eq!((blurred.0, blurred.1, blurred.2.len()), (plain.0, plain.1, plain.2.len()));
        assert_ne!(blurred.2, plain.2, "the demo moves, so the average differs from a single render");
        let again = render_engine(&e, 6.0, &opts, &Shutter::new(360.0, 4)).unwrap();
        assert_eq!(again, blurred, "same inputs, same bytes");
    }

    #[test]
    fn sequence_cost_is_capped_before_any_render() {
        assert!(check_sequence_cost(390, &Shutter::new(180.0, 256)).is_ok());
        let e = check_sequence_cost(391, &Shutter::new(180.0, 256)).unwrap_err();
        assert!(e.contains("limit"), "{e}");
        // angle 0 renders once per frame whatever the sample count says
        assert!(check_sequence_cost(100_000, &Shutter::new(0.0, 256)).is_ok());
        assert!(check_sequence_cost(10, &Shutter::new(400.0, 4)).is_err());
    }
}
