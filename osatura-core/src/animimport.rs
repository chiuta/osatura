// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Decode animated GIF and APNG files into full-canvas RGBA frames with delays, so they can be
//! imported as frame-by-frame layers (see `seqimport`). Written against the public GIF89a and
//! APNG (PNG extension) specifications, using the `gif` and `png` decoder crates.

use crate::math::Vec2;
use crate::project::Project;

const MAX_FRAMES: usize = 2000;
const MAX_PIXELS: u64 = 4096 * 4096;

#[derive(Debug)]
pub struct Decoded {
    pub width: u32,
    pub height: u32,
    /// (RGBA, full canvas, delay in milliseconds)
    pub frames: Vec<(Vec<u8>, u32)>,
}

fn over(dst: &mut [u8], src: &[u8]) {
    // source-over for non-premultiplied RGBA
    let sa = src[3] as f32 / 255.0;
    if sa >= 1.0 {
        dst.copy_from_slice(src);
        return;
    }
    if sa <= 0.0 {
        return;
    }
    let da = dst[3] as f32 / 255.0;
    let oa = sa + da * (1.0 - sa);
    for i in 0..3 {
        dst[i] = ((src[i] as f32 * sa + dst[i] as f32 * da * (1.0 - sa)) / oa).round() as u8;
    }
    dst[3] = (oa * 255.0).round() as u8;
}

pub fn decode_gif(bytes: &[u8]) -> Result<Decoded, String> {
    let mut opts = gif::DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);
    let mut d = opts.read_info(bytes).map_err(|e| format!("invalid GIF: {e}"))?;
    let (w, h) = (d.width() as u32, d.height() as u32);
    if w == 0 || h == 0 || (w as u64) * (h as u64) > MAX_PIXELS {
        return Err("GIF canvas is empty or too large".into());
    }
    let mut canvas = vec![0u8; (w * h * 4) as usize];
    let mut frames = Vec::new();
    while let Some(f) = d.read_next_frame().map_err(|e| format!("invalid GIF: {e}"))? {
        if frames.len() >= MAX_FRAMES {
            return Err(format!("more than {MAX_FRAMES} frames"));
        }
        let before = canvas.clone();
        let (fx, fy, fw, fh) = (f.left as u32, f.top as u32, f.width as u32, f.height as u32);
        for y in 0..fh {
            for x in 0..fw {
                let (cx, cy) = (fx + x, fy + y);
                if cx >= w || cy >= h {
                    continue;
                }
                let s = ((y * fw + x) * 4) as usize;
                if f.buffer[s + 3] == 0 {
                    continue; // transparent index: keep what is below
                }
                let c = ((cy * w + cx) * 4) as usize;
                canvas[c..c + 4].copy_from_slice(&f.buffer[s..s + 4]);
            }
        }
        frames.push((canvas.clone(), (f.delay as u32) * 10));
        match f.dispose {
            gif::DisposalMethod::Background => {
                for y in fy..(fy + fh).min(h) {
                    for x in fx..(fx + fw).min(w) {
                        let c = ((y * w + x) * 4) as usize;
                        canvas[c..c + 4].fill(0);
                    }
                }
            }
            gif::DisposalMethod::Previous => canvas = before,
            _ => {}
        }
    }
    if frames.is_empty() {
        return Err("GIF has no frames".into());
    }
    Ok(Decoded { width: w, height: h, frames })
}

pub fn decode_apng(bytes: &[u8]) -> Result<Decoded, String> {
    crate::assets::check_png(bytes)?;
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut r = dec.read_info().map_err(|e| format!("invalid PNG: {e}"))?;
    let (w, h) = (r.info().width, r.info().height);
    if w == 0 || h == 0 || (w as u64) * (h as u64) > MAX_PIXELS {
        return Err("image is empty or too large".into());
    }
    let animated = r.info().animation_control.is_some();
    if !animated {
        return Err("PNG is not animated (no acTL chunk)".into());
    }
    let n = r.info().animation_control.map(|a| a.num_frames).unwrap_or(0) as usize;
    if n == 0 || n > MAX_FRAMES {
        return Err(format!("animation must have 1..{MAX_FRAMES} frames"));
    }
    let mut canvas = vec![0u8; (w * h * 4) as usize];
    let mut frames = Vec::new();
    let mut buf = vec![0u8; r.output_buffer_size().ok_or("image too large")?];
    // The default image may be outside the animation (no fcTL before IDAT): the decoder then
    // returns it without frame control, and it is skipped.
    for _ in 0..n + 1 {
        if frames.len() == n {
            break;
        }
        let oi = r.next_frame(&mut buf).map_err(|e| format!("invalid APNG: {e}"))?;
        let Some(fc) = r.info().frame_control else { continue };
        let (fx, fy, fw, fh) = (fc.x_offset, fc.y_offset, fc.width, fc.height);
        let ch = match oi.color_type {
            png::ColorType::Rgba => 4,
            png::ColorType::Rgb => 3,
            png::ColorType::GrayscaleAlpha => 2,
            png::ColorType::Grayscale => 1,
            png::ColorType::Indexed => return Err("unexpected indexed output".into()),
        };
        let before = canvas.clone();
        if matches!(fc.blend_op, png::BlendOp::Source) {
            for y in fy..(fy + fh).min(h) {
                for x in fx..(fx + fw).min(w) {
                    let c = ((y * w + x) * 4) as usize;
                    canvas[c..c + 4].fill(0);
                }
            }
        }
        for y in 0..fh {
            for x in 0..fw {
                let (cx, cy) = (fx + x, fy + y);
                if cx >= w || cy >= h {
                    continue;
                }
                let s = ((y * fw + x) as usize) * ch;
                let px: [u8; 4] = match ch {
                    4 => [buf[s], buf[s + 1], buf[s + 2], buf[s + 3]],
                    3 => [buf[s], buf[s + 1], buf[s + 2], 255],
                    2 => [buf[s], buf[s], buf[s], buf[s + 1]],
                    _ => [buf[s], buf[s], buf[s], 255],
                };
                let c = ((cy * w + cx) * 4) as usize;
                over(&mut canvas[c..c + 4], &px);
            }
        }
        let den = if fc.delay_den == 0 { 100 } else { fc.delay_den as u32 };
        frames.push((canvas.clone(), fc.delay_num as u32 * 1000 / den));
        match fc.dispose_op {
            png::DisposeOp::Background => {
                for y in fy..(fy + fh).min(h) {
                    for x in fx..(fx + fw).min(w) {
                        let c = ((y * w + x) * 4) as usize;
                        canvas[c..c + 4].fill(0);
                    }
                }
            }
            png::DisposeOp::Previous => canvas = before,
            png::DisposeOp::None => {}
        }
    }
    if frames.len() != n {
        return Err("APNG ended before all frames were read".into());
    }
    Ok(Decoded { width: w, height: h, frames })
}

/// Decode by content (GIF signature or PNG with animation).
pub fn decode_any(bytes: &[u8]) -> Result<Decoded, String> {
    if bytes.starts_with(b"GIF8") {
        decode_gif(bytes)
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        decode_apng(bytes)
    } else {
        Err("not a GIF or PNG file".into())
    }
}

/// Import an animated GIF/APNG as a switch layer. Delays are converted to project frames
/// (rounded, at least 1; a delay of 0 or 10 ms is treated as 100 ms like browsers do).
pub fn import_animation(p: &mut Project, name: &str, bytes: &[u8], origin: Vec2, start_frame: f32, looped: bool) -> Result<usize, String> {
    let d = decode_any(bytes)?;
    let mut files = Vec::new();
    let mut holds = Vec::new();
    for (i, (rgba, ms)) in d.frames.iter().enumerate() {
        files.push((format!("f{:04}.png", i + 1), crate::open_formats::encode_rgba_png(d.width, d.height, rgba)?));
        let ms = if *ms <= 10 { 100 } else { *ms };
        holds.push((ms as f32 / 1000.0 * p.fps).round().max(1.0));
    }
    crate::seqimport::import_png_sequence_timed(p, name, &files, &holds, origin, start_frame, looped)?;
    Ok(files.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_gif() -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut e = gif::Encoder::new(&mut out, 4, 4, &[]).unwrap();
            for (i, rgb) in [[255u8, 0, 0], [0, 255, 0]].iter().enumerate() {
                let px: Vec<u8> = (0..16).flat_map(|_| *rgb).collect();
                let mut f = gif::Frame::from_rgb(4, 4, &px);
                f.delay = if i == 0 { 20 } else { 5 };
                e.write_frame(&f).unwrap();
            }
        }
        out
    }

    #[test]
    fn gif_frames_and_delays() {
        let d = decode_gif(&solid_gif()).unwrap();
        assert_eq!((d.width, d.height, d.frames.len()), (4, 4, 2));
        assert_eq!(&d.frames[0].0[..4], &[255, 0, 0, 255]);
        assert_eq!(&d.frames[1].0[..4], &[0, 255, 0, 255]);
        assert_eq!((d.frames[0].1, d.frames[1].1), (200, 50));
    }

    #[test]
    fn gif_becomes_a_timed_switch_layer() {
        let mut p = Project::new("a", 64.0, 64.0, 10.0, 10);
        assert_eq!(import_animation(&mut p, "g", &solid_gif(), Vec2::ZERO, 0.0, false).unwrap(), 2);
        let crate::project::Source::Keys { track } = &p.channels[0].source else { panic!() };
        assert_eq!(track.keys.len(), 2);
        assert_eq!(track.keys[1].frame, 2.0, "200 ms at 10 fps = 2 frames");
        crate::Engine::new(p).unwrap();
    }

    #[test]
    fn apng_round_trip_with_our_own_export() {
        let mut p = crate::demo::project();
        p.frames = 4;
        let e = crate::Engine::new(p).unwrap();
        let bytes = crate::open_formats::apng_bytes(&e, 0.25).unwrap();
        let d = decode_apng(&bytes).unwrap();
        assert_eq!(d.frames.len(), 4);
        assert!(d.frames[0].0.chunks(4).any(|px| px[3] > 0), "non-empty picture");
    }

    #[test]
    fn rejects_other_files() {
        assert!(decode_any(b"hello").is_err());
        let still = crate::open_formats::encode_rgba_png(2, 2, &[0u8; 16]).unwrap();
        assert!(decode_any(&still).unwrap_err().contains("not animated"));
        assert!(decode_gif(&[1, 2, 3]).is_err());
    }
}
