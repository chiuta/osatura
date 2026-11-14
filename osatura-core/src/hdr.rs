// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Colour-managed high-precision output: sRGB ↔ linear conversion, 16-bit PNG and OpenEXR.
//!
//! The 8-bit rasteriser carries no extra detail into these outputs; the `fraster` float renderer
//! does (`encode_exr_buf`, `encode_png16_buf`). Either way the colour space is correct for
//! compositing (linear light, premultiplied for EXR) and the containers are what pipelines expect. OpenEXR is an open, openly documented format; this writer emits
//! the simplest valid variant: single-part, scanline, uncompressed, HALF A/B/G/R channels.

pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// IEEE 754 binary16 from f32 (round to nearest even, overflow to infinity, subnormals kept).
pub fn f32_to_f16(v: f32) -> u16 {
    let b = v.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32;
    let man = b & 0x7f_ffff;
    if exp == 0xff {
        return sign | 0x7c00 | if man != 0 { 0x200 } else { 0 };
    }
    let e = exp - 127 + 15;
    if e >= 0x1f {
        return sign | 0x7c00;
    }
    if e <= 0 {
        if e < -10 {
            return sign;
        }
        let m = man | 0x80_0000;
        let shift = (14 - e) as u32;
        let mut h = (m >> shift) as u16;
        let rem = m & ((1 << shift) - 1);
        let half = 1 << (shift - 1);
        if rem > half || (rem == half && h & 1 == 1) {
            h += 1;
        }
        return sign | h;
    }
    let mut h = sign | ((e as u16) << 10) | (man >> 13) as u16;
    let rem = man & 0x1fff;
    if rem > 0x1000 || (rem == 0x1000 && h & 1 == 1) {
        h += 1; // may carry into the exponent, which is correct
    }
    h
}

pub fn f16_to_f32(h: u16) -> f32 {
    let sign = ((h & 0x8000) as u32) << 16;
    let exp = ((h >> 10) & 0x1f) as u32;
    let man = (h & 0x3ff) as u32;
    let bits = match (exp, man) {
        (0, 0) => sign,
        (0, m) => {
            let mut e = 127 - 15 + 1;
            let mut m = m;
            while m & 0x400 == 0 {
                m <<= 1;
                e -= 1;
            }
            sign | ((e as u32) << 23) | ((m & 0x3ff) << 13)
        }
        (0x1f, 0) => sign | 0x7f80_0000,
        (0x1f, m) => sign | 0x7f80_0000 | (m << 13),
        (e, m) => sign | ((e + 127 - 15) << 23) | (m << 13),
    };
    f32::from_bits(bits)
}

fn check(w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
    if w == 0
        || h == 0
        || rgba.len() != (w as usize) * (h as usize) * 4
        || (w as u64) * (h as u64) > 64_000_000
    {
        return Err("bad image size".into());
    }
    Ok(())
}

/// 16-bit RGBA PNG. With `linear` the colour channels are converted from sRGB to linear light
/// first (straight alpha stays as is).
pub fn encode_png16(w: u32, h: u32, rgba: &[u8], linear: bool) -> Result<Vec<u8>, String> {
    check(w, h, rgba)?;
    let mut data = Vec::with_capacity(rgba.len() * 2);
    for p in rgba.chunks_exact(4) {
        for (i, &c) in p.iter().enumerate() {
            let v = c as f32 / 255.0;
            let v = if i < 3 && linear {
                srgb_to_linear(v)
            } else {
                v
            };
            let q = (v * 65535.0).round().clamp(0.0, 65535.0) as u16;
            data.extend_from_slice(&q.to_be_bytes());
        }
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Sixteen);
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        wr.write_image_data(&data).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

fn attr(out: &mut Vec<u8>, name: &str, ty: &str, value: &[u8]) {
    out.extend_from_slice(name.as_bytes());
    out.push(0);
    out.extend_from_slice(ty.as_bytes());
    out.push(0);
    out.extend_from_slice(&(value.len() as i32).to_le_bytes());
    out.extend_from_slice(value);
}

/// EXR sample precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precision {
    /// IEEE binary16 (OpenEXR `HALF`).
    Half,
    /// IEEE binary32 (OpenEXR `FLOAT`), exact for the f32 render buffer.
    Float,
}

impl Precision {
    pub fn parse(s: &str) -> Result<Precision, String> {
        match s.to_ascii_lowercase().as_str() {
            "half" | "16" | "f16" => Ok(Precision::Half),
            "float" | "full" | "32" | "f32" => Ok(Precision::Float),
            o => Err(format!("unknown precision '{o}' (use half or float)")),
        }
    }
}

/// Write a scanline, uncompressed, single-part EXR from linear premultiplied RGBA f32 samples.
fn write_exr(w: u32, h: u32, lin: &[f32], prec: Precision) -> Vec<u8> {
    let (ptype, bps) = match prec {
        Precision::Half => (1i32, 2usize),
        Precision::Float => (2i32, 4usize),
    };
    let mut out = vec![0x76, 0x2f, 0x31, 0x01, 2, 0, 0, 0];
    let mut chlist = Vec::new();
    for name in ["A", "B", "G", "R"] {
        chlist.extend_from_slice(name.as_bytes());
        chlist.push(0);
        chlist.extend_from_slice(&ptype.to_le_bytes());
        chlist.extend_from_slice(&[0, 0, 0, 0]); // pLinear + reserved
        chlist.extend_from_slice(&1i32.to_le_bytes());
        chlist.extend_from_slice(&1i32.to_le_bytes());
    }
    chlist.push(0);
    attr(&mut out, "channels", "chlist", &chlist);
    attr(&mut out, "compression", "compression", &[0]);
    let win: Vec<u8> = [0i32, 0, w as i32 - 1, h as i32 - 1]
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    attr(&mut out, "dataWindow", "box2i", &win);
    attr(&mut out, "displayWindow", "box2i", &win);
    attr(&mut out, "lineOrder", "lineOrder", &[0]);
    attr(&mut out, "pixelAspectRatio", "float", &1.0f32.to_le_bytes());
    attr(
        &mut out,
        "screenWindowCenter",
        "v2f",
        &[0.0f32.to_le_bytes(), 0.0f32.to_le_bytes()].concat(),
    );
    attr(
        &mut out,
        "screenWindowWidth",
        "float",
        &1.0f32.to_le_bytes(),
    );
    out.push(0); // end of header
    let line_bytes = w as usize * 4 * bps;
    let table_at = out.len();
    out.extend(std::iter::repeat_n(0u8, h as usize * 8));
    for y in 0..h as usize {
        let off = out.len() as u64;
        out[table_at + y * 8..table_at + y * 8 + 8].copy_from_slice(&off.to_le_bytes());
        out.extend_from_slice(&(y as i32).to_le_bytes());
        out.extend_from_slice(&(line_bytes as i32).to_le_bytes());
        let row = &lin[y * w as usize * 4..(y + 1) * w as usize * 4];
        for ch in [3usize, 2, 1, 0] {
            for p in row.chunks_exact(4) {
                match prec {
                    Precision::Half => out.extend_from_slice(&f32_to_f16(p[ch]).to_le_bytes()),
                    Precision::Float => out.extend_from_slice(&p[ch].to_le_bytes()),
                }
            }
        }
    }
    out
}

/// OpenEXR, half float RGBA, linear light, premultiplied alpha (the format's convention).
pub fn encode_exr_half(w: u32, h: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    check(w, h, rgba)?;
    let mut lin = Vec::with_capacity(rgba.len());
    for p in rgba.chunks_exact(4) {
        let a = p[3] as f32 / 255.0;
        lin.extend_from_slice(&[
            srgb_to_linear(p[0] as f32 / 255.0) * a,
            srgb_to_linear(p[1] as f32 / 255.0) * a,
            srgb_to_linear(p[2] as f32 / 255.0) * a,
            a,
        ]);
    }
    Ok(write_exr(w, h, &lin, Precision::Half))
}

fn check_buf(w: u32, h: u32, buf: &[f32]) -> Result<(), String> {
    if w == 0
        || h == 0
        || buf.len() != (w as usize) * (h as usize) * 4
        || (w as u64) * (h as u64) > 64_000_000
    {
        return Err("bad image size".into());
    }
    Ok(())
}

/// OpenEXR from the float renderer's buffer (premultiplied, sRGB-encoded RGBA f32): colour is
/// un-premultiplied, converted to linear light and premultiplied again (alpha untouched).
pub fn encode_exr_buf(
    w: u32,
    h: u32,
    premul_encoded: &[f32],
    prec: Precision,
) -> Result<Vec<u8>, String> {
    check_buf(w, h, premul_encoded)?;
    let mut lin = Vec::with_capacity(premul_encoded.len());
    for p in premul_encoded.chunks_exact(4) {
        let a = p[3];
        let d = if a > 0.0 { 1.0 / a } else { 0.0 };
        lin.extend_from_slice(&[
            srgb_to_linear((p[0] * d).clamp(0.0, 1.0)) * a,
            srgb_to_linear((p[1] * d).clamp(0.0, 1.0)) * a,
            srgb_to_linear((p[2] * d).clamp(0.0, 1.0)) * a,
            a,
        ]);
    }
    Ok(write_exr(w, h, &lin, prec))
}

/// 16-bit RGBA PNG from the float buffer. Straight alpha, plain round-to-nearest quantisation to
/// 65535 levels — deliberately *no dither*, so output is deterministic and loss-free enough
/// (1/65535 steps) that no banding survives. With `linear` colour is converted to linear light.
pub fn encode_png16_buf(
    w: u32,
    h: u32,
    premul_encoded: &[f32],
    linear: bool,
) -> Result<Vec<u8>, String> {
    check_buf(w, h, premul_encoded)?;
    let mut data = Vec::with_capacity(premul_encoded.len() * 2);
    for p in premul_encoded.chunks_exact(4) {
        let a = p[3];
        let d = if a > 0.0 { 1.0 / a } else { 0.0 };
        for (i, &c) in p.iter().enumerate() {
            let v = if i == 3 { a } else { (c * d).clamp(0.0, 1.0) };
            let v = if i < 3 && linear {
                srgb_to_linear(v)
            } else {
                v
            };
            data.extend_from_slice(
                &((v * 65535.0).round().clamp(0.0, 65535.0) as u16).to_be_bytes(),
            );
        }
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Sixteen);
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        wr.write_image_data(&data).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_curves_roundtrip_and_known_points() {
        assert!((srgb_to_linear(0.5) - 0.2140).abs() < 1e-3);
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-6);
        for i in 0..=255 {
            let v = i as f32 / 255.0;
            assert!((linear_to_srgb(srgb_to_linear(v)) - v).abs() < 1e-4);
        }
    }

    #[test]
    fn half_float_conversion() {
        for (v, h) in [
            (0.0f32, 0x0000u16),
            (1.0, 0x3c00),
            (-2.0, 0xc000),
            (0.5, 0x3800),
            (65504.0, 0x7bff),
            (f32::INFINITY, 0x7c00),
        ] {
            assert_eq!(f32_to_f16(v), h, "{v}");
            if v.is_finite() {
                assert_eq!(f16_to_f32(h), v);
            }
        }
        assert_eq!(f32_to_f16(1.0e6), 0x7c00, "overflow to inf");
        let tiny = 3.0e-6f32; // subnormal in half
        assert!((f16_to_f32(f32_to_f16(tiny)) - tiny).abs() < 1e-6);
        for i in 0..=1000 {
            let v = i as f32 / 1000.0;
            assert!((f16_to_f32(f32_to_f16(v)) - v).abs() < 1e-3);
        }
    }

    fn sample() -> Vec<u8> {
        vec![
            255, 128, 0, 255, 10, 20, 30, 128, 0, 0, 0, 0, 255, 255, 255, 255,
        ]
    }

    #[test]
    fn png16_decodes_with_expected_values() {
        let bytes = encode_png16(2, 2, &sample(), false).unwrap();
        let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
        dec.set_transformations(png::Transformations::IDENTITY);
        let mut r = dec.read_info().unwrap();
        assert_eq!(r.info().bit_depth, png::BitDepth::Sixteen);
        let mut buf = vec![0; r.output_buffer_size().unwrap()];
        r.next_frame(&mut buf).unwrap();
        let px = |i: usize| u16::from_be_bytes([buf[i * 2], buf[i * 2 + 1]]);
        assert_eq!(px(0), 65535);
        assert_eq!(px(1), (128.0f32 / 255.0 * 65535.0).round() as u16);
        let lin = encode_png16(2, 2, &sample(), true).unwrap();
        let mut dec = png::Decoder::new(std::io::Cursor::new(lin));
        dec.set_transformations(png::Transformations::IDENTITY);
        let mut r = dec.read_info().unwrap();
        let mut b2 = vec![0; r.output_buffer_size().unwrap()];
        r.next_frame(&mut b2).unwrap();
        let g = u16::from_be_bytes([b2[2], b2[3]]);
        assert!((g as f32 / 65535.0 - srgb_to_linear(128.0 / 255.0)).abs() < 1e-3);
        let a = u16::from_be_bytes([b2[6], b2[7]]);
        assert_eq!(a, 65535, "alpha is not colour-converted");
    }

    #[test]
    fn exr_structure_and_pixels() {
        let b = encode_exr_half(2, 2, &sample()).unwrap();
        assert_eq!(&b[..4], &[0x76, 0x2f, 0x31, 0x01]);
        assert_eq!(b[4], 2);
        let find = |needle: &[u8]| b.windows(needle.len()).position(|w| w == needle).unwrap();
        assert!(find(b"channels\0chlist\0") < 40);
        let hdr_end =
            find(b"screenWindowWidth\0float\0") + b"screenWindowWidth\0float\0".len() + 4 + 4;
        assert_eq!(b[hdr_end], 0, "header terminator");
        let table = hdr_end + 1;
        let off1 = u64::from_le_bytes(b[table..table + 8].try_into().unwrap()) as usize;
        let off2 = u64::from_le_bytes(b[table + 8..table + 16].try_into().unwrap()) as usize;
        assert_eq!(off1, table + 16);
        assert_eq!(off2, off1 + 8 + 16, "each line: y + size + 2px x 4ch x 2B");
        assert_eq!(i32::from_le_bytes(b[off1..off1 + 4].try_into().unwrap()), 0);
        assert_eq!(
            i32::from_le_bytes(b[off1 + 4..off1 + 8].try_into().unwrap()),
            16
        );
        assert_eq!(b.len(), off2 + 8 + 16);
        // line 0, channel A first: pixel 0 alpha = 1.0, pixel 1 alpha = 128/255
        let a0 = u16::from_le_bytes([b[off1 + 8], b[off1 + 9]]);
        assert_eq!(f16_to_f32(a0), 1.0);
        // channel B for pixel 1 (10,20,30,a=128): premultiplied linear blue
        let bidx = off1 + 8 + 4 + 2;
        let blue = f16_to_f32(u16::from_le_bytes([b[bidx], b[bidx + 1]]));
        let want = srgb_to_linear(30.0 / 255.0) * (128.0 / 255.0);
        assert!((blue - want).abs() < 1e-3, "{blue} vs {want}");
        assert!(encode_exr_half(0, 2, &[]).is_err() && encode_exr_half(2, 2, &[0; 3]).is_err());
    }
}
