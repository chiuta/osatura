// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Decoder of animated PNG (APNG) files into a list of full-canvas RGBA frames with their durations.
//!
//! Basis (public): the *APNG Specification* (https://wiki.mozilla.org/APNG_Specification: `acTL`, `fcTL`, `fdAT`, sequence numbers,
//! `dispose_op`, `blend_op`) on top of the PNG chunk structure (length, type, data, CRC-32) and the "alpha compositing" rule for the
//! over operator. The pixel data of each frame is decoded as an ordinary PNG image of the frame's size by the `png` crate already used by
//! the project; chunk parsing, ordering rules, compositing and disposal are own code. Spec: docs/clean-room/specs/02-apng-decodare-secventa.md.
//!
//! Decisions where the specification text read is silent: the canvas starts fully transparent (transparent black); `OVER` uses the
//! non-premultiplied formula in floating point rounded to the nearest integer; a file without `acTL` is one frame of duration 0; a delay
//! of 0 stays 0 ms (no project minimum); ancillary chunks other than `PLTE` and `tRNS` are checked for CRC and then ignored.
//! Output is atomic: any error returns no frames.

use crate::gifseq::{Frame, Limits};

const SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const MAX_CHUNKS: usize = 1 << 20;

/// The decoded file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApngSequence {
    pub width: u32,
    pub height: u32,
    /// Number of plays from `acTL`: `Some(0)` = forever; `None` for a PNG without animation.
    pub plays: Option<u32>,
    pub frames: Vec<Frame>,
}

/// CRC-32 (IEEE 802.3, reflected, polynomial 0xEDB88320) as used by PNG chunks, computed over `bytes`.
pub fn crc32(bytes: &[u8]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let t = TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for (n, e) in t.iter_mut().enumerate() {
            let mut c = n as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            *e = c;
        }
        t
    });
    let mut c = 0xFFFF_FFFFu32;
    for &b in bytes {
        c = t[((c ^ u32::from(b)) & 0xff) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

struct Chunk<'a> {
    kind: [u8; 4],
    data: &'a [u8],
}

fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

fn parse_chunks(bytes: &[u8]) -> Result<Vec<Chunk<'_>>, String> {
    if bytes.len() < 8 || bytes[..8] != SIGNATURE {
        return Err("not a PNG file (bad signature)".into());
    }
    let mut at = 8usize;
    let mut out = Vec::new();
    let mut ended = false;
    while at < bytes.len() {
        if out.len() >= MAX_CHUNKS {
            return Err("too many chunks".into());
        }
        let head = bytes.get(at..at + 8).ok_or("truncated chunk header")?;
        let len = be32(head) as usize;
        let kind = [head[4], head[5], head[6], head[7]];
        let end = (at + 8).checked_add(len).and_then(|e| e.checked_add(4)).filter(|e| *e <= bytes.len()).ok_or("truncated chunk")?;
        let data = &bytes[at + 8..at + 8 + len];
        let want = be32(&bytes[at + 8 + len..end]);
        if crc32(&bytes[at + 4..at + 8 + len]) != want {
            return Err(format!("CRC mismatch in chunk {}", String::from_utf8_lossy(&kind)));
        }
        out.push(Chunk { kind, data });
        at = end;
        if &kind == b"IEND" {
            ended = true;
            break;
        }
    }
    if !ended {
        return Err("truncated file (no IEND chunk)".into());
    }
    Ok(out)
}

#[derive(Clone, Copy)]
struct FrameControl {
    seq: u32,
    w: u32,
    h: u32,
    x: u32,
    y: u32,
    delay_num: u16,
    delay_den: u16,
    dispose: u8,
    blend: u8,
}

struct RawFrame {
    ctl: FrameControl,
    data: Vec<u8>,
}

fn parse_fctl(d: &[u8]) -> Result<FrameControl, String> {
    if d.len() != 26 {
        return Err(format!("fcTL has {} bytes, expected 26", d.len()));
    }
    let c = FrameControl {
        seq: be32(&d[0..4]),
        w: be32(&d[4..8]),
        h: be32(&d[8..12]),
        x: be32(&d[12..16]),
        y: be32(&d[16..20]),
        delay_num: u16::from_be_bytes([d[20], d[21]]),
        delay_den: u16::from_be_bytes([d[22], d[23]]),
        dispose: d[24],
        blend: d[25],
    };
    if c.dispose > 2 {
        return Err(format!("unknown dispose_op {}", c.dispose));
    }
    if c.blend > 1 {
        return Err(format!("unknown blend_op {}", c.blend));
    }
    Ok(c)
}

fn delay_ms(c: &FrameControl) -> u32 {
    let den = if c.delay_den == 0 { 100 } else { u64::from(c.delay_den) };
    ((u64::from(c.delay_num) * 1000 + den / 2) / den).min(u64::from(u32::MAX)) as u32
}

fn push_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// Decodes the compressed data of one frame as a plain PNG of size `w x h` with the header properties of the file, into RGBA8.
fn decode_region(ihdr: &[u8], extra: &[&Chunk], w: u32, h: u32, idat: &[u8]) -> Result<Vec<u8>, String> {
    let mut png_bytes = SIGNATURE.to_vec();
    let mut hdr = ihdr.to_vec();
    hdr[0..4].copy_from_slice(&w.to_be_bytes());
    hdr[4..8].copy_from_slice(&h.to_be_bytes());
    push_chunk(&mut png_bytes, b"IHDR", &hdr);
    for c in extra {
        push_chunk(&mut png_bytes, &c.kind, c.data);
    }
    push_chunk(&mut png_bytes, b"IDAT", idat);
    push_chunk(&mut png_bytes, b"IEND", &[]);
    let mut dec = png::Decoder::new(std::io::Cursor::new(png_bytes));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = dec.read_info().map_err(|e| format!("invalid frame data: {e}"))?;
    let size = reader.output_buffer_size().ok_or("frame too large")?;
    let mut buf = vec![0u8; size];
    let info = reader.next_frame(&mut buf).map_err(|e| format!("invalid frame data: {e}"))?;
    if info.width != w || info.height != h {
        return Err("decoded frame has a different size than its fcTL".into());
    }
    let px = (w as usize) * (h as usize);
    let src = &buf[..info.buffer_size()];
    let mut out = Vec::with_capacity(px * 4);
    let need = |n: usize| if src.len() == px * n { Ok(()) } else { Err("frame data has an unexpected length".to_string()) };
    match info.color_type {
        png::ColorType::Rgba => {
            need(4)?;
            out.extend_from_slice(src);
        }
        png::ColorType::Rgb => {
            need(3)?;
            src.chunks_exact(3).for_each(|p| out.extend_from_slice(&[p[0], p[1], p[2], 255]));
        }
        png::ColorType::Grayscale => {
            need(1)?;
            src.iter().for_each(|&g| out.extend_from_slice(&[g, g, g, 255]));
        }
        png::ColorType::GrayscaleAlpha => {
            need(2)?;
            src.chunks_exact(2).for_each(|p| out.extend_from_slice(&[p[0], p[0], p[0], p[1]]));
        }
        png::ColorType::Indexed => return Err("unexpanded palette data".into()),
    }
    Ok(out)
}

fn over(dst: &mut [u8], src: &[u8]) {
    let sa = f64::from(src[3]) / 255.0;
    if src[3] == 255 || dst[3] == 0 {
        dst.copy_from_slice(src);
        return;
    }
    if src[3] == 0 {
        return;
    }
    let da = f64::from(dst[3]) / 255.0;
    let oa = sa + da * (1.0 - sa);
    for i in 0..3 {
        dst[i] = ((f64::from(src[i]) * sa + f64::from(dst[i]) * da * (1.0 - sa)) / oa).round().clamp(0.0, 255.0) as u8;
    }
    dst[3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
}

/// Decodes an APNG (or a plain PNG, as one frame). Any error returns no frames.
pub fn decode(bytes: &[u8], limits: &Limits) -> Result<ApngSequence, String> {
    let chunks = parse_chunks(bytes)?;
    let first = chunks.first().ok_or("empty file")?;
    if &first.kind != b"IHDR" || first.data.len() != 13 {
        return Err("the first chunk must be a 13-byte IHDR".into());
    }
    let ihdr = first.data;
    let (width, height) = (be32(&ihdr[0..4]), be32(&ihdr[4..8]));
    if width == 0 || height == 0 {
        return Err("the image has a zero dimension".into());
    }
    let canvas_px = u64::from(width) * u64::from(height);
    if canvas_px > limits.max_frame_pixels {
        return Err(format!("the image ({width}x{height}) exceeds the limit of {} pixels", limits.max_frame_pixels));
    }
    let shared: Vec<&Chunk> = chunks.iter().filter(|c| &c.kind == b"PLTE" || &c.kind == b"tRNS").collect();

    let mut actl: Option<(u32, u32)> = None;
    let mut seen_idat = false;
    let mut default_data: Vec<u8> = Vec::new();
    let mut frames: Vec<RawFrame> = Vec::new();
    let mut in_default_frame = false; // the last fcTL came before IDAT
    let mut expected_seq = 0u32;
    let mut after_idat_frames = false;
    for c in &chunks[1..] {
        match &c.kind {
            b"acTL" => {
                if seen_idat {
                    return Err("acTL after the first IDAT".into());
                }
                if actl.is_some() {
                    return Err("more than one acTL".into());
                }
                if c.data.len() != 8 {
                    return Err("acTL must have 8 bytes".into());
                }
                actl = Some((be32(&c.data[0..4]), be32(&c.data[4..8])));
            }
            b"fcTL" => {
                let ctl = parse_fctl(c.data)?;
                if ctl.seq != expected_seq {
                    return Err(format!("sequence number {} where {expected_seq} was expected", ctl.seq));
                }
                expected_seq += 1;
                if frames.len() >= limits.max_frames {
                    return Err(format!("more than {} frames", limits.max_frames));
                }
                if !seen_idat {
                    if frames.is_empty() {
                        in_default_frame = true;
                        if ctl.x != 0 || ctl.y != 0 || ctl.w != width || ctl.h != height {
                            return Err("the fcTL before IDAT must cover the whole image at offset 0".into());
                        }
                    } else {
                        return Err("a second fcTL before IDAT".into());
                    }
                } else {
                    after_idat_frames = true;
                    in_default_frame = false;
                }
                frames.push(RawFrame { ctl, data: Vec::new() });
            }
            b"IDAT" => {
                if after_idat_frames {
                    return Err("IDAT after animation frame data".into());
                }
                seen_idat = true;
                default_data.extend_from_slice(c.data);
                if in_default_frame {
                    if let Some(f) = frames.last_mut() {
                        f.data.extend_from_slice(c.data);
                    }
                }
            }
            b"fdAT" => {
                if c.data.len() < 4 {
                    return Err("fdAT too short".into());
                }
                let seq = be32(&c.data[0..4]);
                if seq != expected_seq {
                    return Err(format!("sequence number {seq} where {expected_seq} was expected"));
                }
                expected_seq += 1;
                match frames.last_mut() {
                    Some(f) if !in_default_frame => f.data.extend_from_slice(&c.data[4..]),
                    _ => return Err("fdAT without a preceding fcTL".into()),
                }
            }
            _ => {}
        }
    }
    if !seen_idat {
        return Err("no IDAT chunk".into());
    }
    let Some((n_frames, plays)) = actl else {
        // an ordinary PNG: one frame
        let rgba = decode_region(ihdr, &shared, width, height, &default_data)?;
        return Ok(ApngSequence { width, height, plays: None, frames: vec![Frame { rgba, duration_ms: 0 }] });
    };
    if n_frames == 0 {
        return Err("acTL declares 0 frames".into());
    }
    if n_frames as usize != frames.len() {
        return Err(format!("acTL declares {n_frames} frames but the file has {} fcTL chunks", frames.len()));
    }
    if (frames.len() as u64) * canvas_px > limits.max_total_pixels {
        return Err(format!("the decoded frames would exceed {} pixels in total", limits.max_total_pixels));
    }
    for (i, f) in frames.iter().enumerate() {
        let c = &f.ctl;
        if c.w == 0 || c.h == 0 {
            return Err(format!("frame {i} has a zero dimension"));
        }
        if u64::from(c.x) + u64::from(c.w) > u64::from(width) || u64::from(c.y) + u64::from(c.h) > u64::from(height) {
            return Err(format!("frame {i} lies outside the image"));
        }
        if f.data.is_empty() {
            return Err(format!("frame {i} has no image data"));
        }
    }
    let (w, h) = (width as usize, height as usize);
    let mut canvas = vec![0u8; w * h * 4];
    let mut out = Vec::with_capacity(frames.len());
    for (i, f) in frames.iter().enumerate() {
        let c = &f.ctl;
        let px = decode_region(ihdr, &shared, c.w, c.h, &f.data).map_err(|e| format!("frame {i}: {e}"))?;
        let (x0, y0, fw, fh) = (c.x as usize, c.y as usize, c.w as usize, c.h as usize);
        let saved: Option<Vec<u8>> = if c.dispose == 2 && i > 0 {
            let mut s = Vec::with_capacity(fw * fh * 4);
            for y in 0..fh {
                let o = ((y0 + y) * w + x0) * 4;
                s.extend_from_slice(&canvas[o..o + fw * 4]);
            }
            Some(s)
        } else {
            None
        };
        for y in 0..fh {
            for x in 0..fw {
                let o = ((y0 + y) * w + x0 + x) * 4;
                let s = (y * fw + x) * 4;
                if c.blend == 0 {
                    canvas[o..o + 4].copy_from_slice(&px[s..s + 4]);
                } else {
                    over(&mut canvas[o..o + 4], &px[s..s + 4]);
                }
            }
        }
        out.push(Frame { rgba: canvas.clone(), duration_ms: delay_ms(c) });
        // the first frame's "previous" is treated as "background"
        match (c.dispose, saved) {
            (1, _) | (2, None) => {
                for y in 0..fh {
                    let o = ((y0 + y) * w + x0) * 4;
                    canvas[o..o + fw * 4].fill(0);
                }
            }
            (2, Some(s)) => {
                for y in 0..fh {
                    let o = ((y0 + y) * w + x0) * 4;
                    canvas[o..o + fw * 4].copy_from_slice(&s[y * fw * 4..(y + 1) * fw * 4]);
                }
            }
            _ => {}
        }
    }
    Ok(ApngSequence { width, height, plays: Some(plays), frames: out })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        push_chunk(&mut v, kind, data);
        v
    }

    /// The IDAT payload of an RGBA PNG of `w x h` filled with one colour.
    fn idat_of(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let data: Vec<u8> = (0..w * h).flat_map(|_| rgba).collect();
        let png = crate::open_formats::encode_rgba_png(w, h, &data).unwrap();
        let mut out = Vec::new();
        for c in parse_chunks(&png).unwrap() {
            if &c.kind == b"IDAT" {
                out.extend_from_slice(c.data);
            }
        }
        out
    }

    fn ihdr(w: u32, h: u32) -> Vec<u8> {
        let mut d = Vec::new();
        d.extend_from_slice(&w.to_be_bytes());
        d.extend_from_slice(&h.to_be_bytes());
        d.extend_from_slice(&[8, 6, 0, 0, 0]);
        chunk(b"IHDR", &d)
    }

    #[allow(clippy::too_many_arguments)]
    fn fctl(seq: u32, w: u32, h: u32, x: u32, y: u32, delay: (u16, u16), dispose: u8, blend: u8) -> Vec<u8> {
        let mut d = Vec::new();
        for v in [seq, w, h, x, y] {
            d.extend_from_slice(&v.to_be_bytes());
        }
        d.extend_from_slice(&delay.0.to_be_bytes());
        d.extend_from_slice(&delay.1.to_be_bytes());
        d.extend_from_slice(&[dispose, blend]);
        chunk(b"fcTL", &d)
    }

    fn fdat(seq: u32, payload: &[u8]) -> Vec<u8> {
        let mut d = seq.to_be_bytes().to_vec();
        d.extend_from_slice(payload);
        chunk(b"fdAT", &d)
    }

    fn actl(frames: u32, plays: u32) -> Vec<u8> {
        let mut d = frames.to_be_bytes().to_vec();
        d.extend_from_slice(&plays.to_be_bytes());
        chunk(b"acTL", &d)
    }

    fn file(parts: &[Vec<u8>]) -> Vec<u8> {
        let mut v = SIGNATURE.to_vec();
        for p in parts {
            v.extend_from_slice(p);
        }
        v.extend(chunk(b"IEND", &[]));
        v
    }

    const RED: [u8; 4] = [255, 0, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];

    #[test]
    fn crc_known_vector() {
        // the CRC of the IEND chunk type is a well-known constant of every PNG file
        assert_eq!(crc32(b"IEND"), 0xAE42_6082);
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn hidden_default_image_and_two_frames() {
        let f = file(&[
            ihdr(2, 2),
            actl(2, 0),
            chunk(b"IDAT", &idat_of(2, 2, [9, 9, 9, 255])),
            fctl(0, 2, 2, 0, 0, (1, 0), 0, 0),
            fdat(1, &idat_of(2, 2, RED)),
            fctl(2, 2, 2, 0, 0, (1, 4), 0, 0),
            fdat(3, &idat_of(2, 2, BLUE)),
        ]);
        let s = decode(&f, &Limits::default()).unwrap();
        assert_eq!(s.plays, Some(0));
        assert_eq!(s.frames.len(), 2);
        assert_eq!(&s.frames[0].rgba[..4], &RED);
        assert_eq!(&s.frames[1].rgba[..4], &BLUE);
        assert_eq!((s.frames[0].duration_ms, s.frames[1].duration_ms), (10, 250));
    }

    #[test]
    fn default_image_is_the_first_frame_when_fctl_precedes_idat() {
        let f = file(&[ihdr(2, 2), actl(2, 3), fctl(0, 2, 2, 0, 0, (3, 100), 0, 0), chunk(b"IDAT", &idat_of(2, 2, RED)), fctl(1, 2, 2, 0, 0, (0, 0), 0, 0), fdat(2, &idat_of(2, 2, BLUE))]);
        let s = decode(&f, &Limits::default()).unwrap();
        assert_eq!(s.plays, Some(3));
        assert_eq!(s.frames.len(), 2);
        assert_eq!(&s.frames[0].rgba[..4], &RED);
        assert_eq!(s.frames[0].duration_ms, 30);
        assert_eq!(s.frames[1].duration_ms, 0);
    }

    #[test]
    fn first_frame_dispose_previous_acts_as_background() {
        let f = file(&[ihdr(1, 1), actl(1, 0), fctl(0, 1, 1, 0, 0, (1, 10), 2, 0), chunk(b"IDAT", &idat_of(1, 1, RED))]);
        let s = decode(&f, &Limits::default()).unwrap();
        assert_eq!(s.frames.len(), 1);
        assert_eq!(s.frames[0].rgba, RED);
        // the frame shows the colour; the canvas used for the next frame would be transparent: see the two-frame check below
        let g = file(&[ihdr(1, 1), actl(2, 0), fctl(0, 1, 1, 0, 0, (1, 10), 2, 0), chunk(b"IDAT", &idat_of(1, 1, RED)), fctl(1, 1, 1, 0, 0, (1, 10), 0, 0), fdat(2, &idat_of(1, 1, [0, 0, 0, 0]))]);
        let s = decode(&g, &Limits::default()).unwrap();
        assert_eq!(s.frames[1].rgba, [0, 0, 0, 0]);
    }

    #[test]
    fn dispose_previous_restores_the_region() {
        let f = file(&[
            ihdr(2, 1),
            actl(3, 0),
            fctl(0, 2, 1, 0, 0, (1, 10), 0, 0),
            chunk(b"IDAT", &idat_of(2, 1, RED)),
            fctl(1, 1, 1, 1, 0, (1, 10), 2, 0),
            fdat(2, &idat_of(1, 1, BLUE)),
            fctl(3, 1, 1, 0, 0, (1, 10), 0, 0),
            fdat(4, &idat_of(1, 1, BLUE)),
        ]);
        let s = decode(&f, &Limits::default()).unwrap();
        assert_eq!(&s.frames[1].rgba[4..8], &BLUE);
        // after frame 2 (dispose previous) the second pixel is red again
        assert_eq!(&s.frames[2].rgba[4..8], &RED);
        assert_eq!(&s.frames[2].rgba[..4], &BLUE);
    }

    #[test]
    fn over_and_source_blending() {
        let mk = |blend: u8| {
            file(&[ihdr(1, 1), actl(2, 0), fctl(0, 1, 1, 0, 0, (1, 10), 0, 0), chunk(b"IDAT", &idat_of(1, 1, RED)), fctl(1, 1, 1, 0, 0, (1, 10), 0, blend), fdat(2, &idat_of(1, 1, [0, 0, 255, 128]))])
        };
        assert_eq!(decode(&mk(1), &Limits::default()).unwrap().frames[1].rgba, [127, 0, 128, 255]);
        assert_eq!(decode(&mk(0), &Limits::default()).unwrap().frames[1].rgba, [0, 0, 255, 128]);
    }

    #[test]
    fn errors() {
        let ok = |parts: Vec<Vec<u8>>| decode(&file(&parts), &Limits::default());
        let idat = || chunk(b"IDAT", &idat_of(1, 1, RED));
        // sequence numbers 0, 2
        let e = ok(vec![ihdr(1, 1), actl(2, 0), fctl(0, 1, 1, 0, 0, (1, 1), 0, 0), idat(), fctl(2, 1, 1, 0, 0, (1, 1), 0, 0), fdat(3, &idat_of(1, 1, RED))]).unwrap_err();
        assert!(e.contains("sequence number"), "{e}");
        // zero frames
        assert!(ok(vec![ihdr(1, 1), actl(0, 0), idat()]).is_err());
        // offset + width beyond the header width
        assert!(ok(vec![ihdr(1, 1), actl(1, 0), idat(), fctl(0, 1, 1, 1, 0, (1, 1), 0, 0), fdat(1, &idat_of(1, 1, RED))]).unwrap_err().contains("outside"));
        // frame without fdAT
        assert!(ok(vec![ihdr(1, 1), actl(1, 0), idat(), fctl(0, 1, 1, 0, 0, (1, 1), 0, 0)]).unwrap_err().contains("no image data"));
        // frame count differs from the number of fcTL
        assert!(ok(vec![ihdr(1, 1), actl(2, 0), idat(), fctl(0, 1, 1, 0, 0, (1, 1), 0, 0), fdat(1, &idat_of(1, 1, RED))]).unwrap_err().contains("declares 2"));
        // acTL after IDAT
        assert!(ok(vec![ihdr(1, 1), idat(), actl(1, 0)]).unwrap_err().contains("acTL after"));
        // fdAT without fcTL
        assert!(ok(vec![ihdr(1, 1), actl(1, 0), idat(), fdat(0, &[1, 2, 3])]).is_err());
        // not a PNG, truncated
        assert!(decode(b"GIF89a", &Limits::default()).is_err());
        let good = file(&[ihdr(1, 1), idat()]);
        assert!(decode(&good[..good.len() - 5], &Limits::default()).is_err());
    }

    #[test]
    fn crc_error_in_fctl() {
        let mut f = file(&[ihdr(1, 1), actl(1, 0), fctl(0, 1, 1, 0, 0, (1, 1), 0, 0), chunk(b"IDAT", &idat_of(1, 1, RED))]);
        let at = f.windows(4).position(|w| w == b"fcTL").unwrap() + 6;
        f[at] ^= 0x40;
        let e = decode(&f, &Limits::default()).unwrap_err();
        assert!(e.contains("CRC"), "{e}");
    }

    #[test]
    fn plain_png_is_one_frame() {
        let data: Vec<u8> = (0..4u8).flat_map(|i| [i, i, i, 255]).collect();
        let png = crate::open_formats::encode_rgba_png(2, 2, &data).unwrap();
        let s = decode(&png, &Limits::default()).unwrap();
        assert_eq!((s.width, s.height, s.frames.len(), s.plays), (2, 2, 1, None));
        assert_eq!(s.frames[0].rgba, data);
        assert_eq!(s.frames[0].duration_ms, 0);
    }

    #[test]
    fn other_colour_types_are_expanded() {
        let mut buf = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut buf, 2, 1);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&[1, 2, 3, 4, 5, 6]).unwrap();
        }
        let s = decode(&buf, &Limits::default()).unwrap();
        assert_eq!(s.frames[0].rgba, [1, 2, 3, 255, 4, 5, 6, 255]);
        let mut buf = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut buf, 1, 1);
            enc.set_color(png::ColorType::Indexed);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_palette(vec![10, 20, 30]);
            enc.set_trns(vec![100]);
            enc.write_header().unwrap().write_image_data(&[0]).unwrap();
        }
        assert_eq!(decode(&buf, &Limits::default()).unwrap().frames[0].rgba, [10, 20, 30, 100]);
    }

    #[test]
    fn agrees_with_the_crate_based_importer_on_valid_files() {
        let files = [
            file(&[
                ihdr(2, 1),
                actl(3, 0),
                fctl(0, 2, 1, 0, 0, (1, 10), 0, 0),
                chunk(b"IDAT", &idat_of(2, 1, RED)),
                fctl(1, 1, 1, 1, 0, (1, 10), 2, 0),
                fdat(2, &idat_of(1, 1, BLUE)),
                fctl(3, 1, 1, 0, 0, (2, 10), 1, 1),
                fdat(4, &idat_of(1, 1, [0, 255, 0, 128])),
            ]),
            file(&[
                ihdr(2, 2),
                actl(2, 0),
                chunk(b"IDAT", &idat_of(2, 2, [9, 9, 9, 255])),
                fctl(0, 2, 2, 0, 0, (1, 0), 0, 0),
                fdat(1, &idat_of(2, 2, RED)),
                fctl(2, 1, 2, 1, 0, (1, 4), 0, 1),
                fdat(3, &idat_of(1, 2, [0, 0, 255, 128])),
            ]),
        ];
        for (n, f) in files.iter().enumerate() {
            let mine = decode(f, &Limits::default()).unwrap();
            let theirs = crate::animimport::decode_apng(f).unwrap();
            assert_eq!(mine.frames.len(), theirs.frames.len(), "file {n}");
            for (i, (m, t)) in mine.frames.iter().zip(&theirs.frames).enumerate() {
                assert_eq!(m.duration_ms, t.1, "file {n} frame {i} delay");
                assert_eq!(m.rgba, t.0, "file {n} frame {i} pixels");
            }
        }
    }

    #[test]
    fn caps_are_checked_before_decoding() {
        let f = file(&[ihdr(2, 2), chunk(b"IDAT", &idat_of(2, 2, RED))]);
        assert!(decode(&f, &Limits { max_frame_pixels: 3, ..Limits::default() }).unwrap_err().contains("exceeds"));
        let a = file(&[ihdr(1, 1), actl(2, 0), fctl(0, 1, 1, 0, 0, (1, 1), 0, 0), chunk(b"IDAT", &idat_of(1, 1, RED)), fctl(1, 1, 1, 0, 0, (1, 1), 0, 0), fdat(2, &idat_of(1, 1, RED))]);
        assert!(decode(&a, &Limits { max_frames: 1, ..Limits::default() }).is_err());
        assert!(decode(&a, &Limits { max_total_pixels: 1, ..Limits::default() }).is_err());
    }
}
