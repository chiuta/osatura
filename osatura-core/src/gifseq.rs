// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Decoder of GIF files (static or animated) into a list of full-canvas RGBA frames with their durations.
//!
//! Basis (public): *Graphics Interchange Format, Version 89a* (CompuServe, 31 July 1990; copy at https://www.w3.org/Graphics/GIF/spec-gif89a.txt):
//! the logical screen descriptor, colour tables, graphic control extension, image descriptor, interlacing, the variable-length-code LZW
//! compression and the block grammar. Own code; the `gif` crate used elsewhere in the project is not involved. Spec:
//! docs/clean-room/specs/01-gif-decodare-secventa.md.
//!
//! Decisions of this implementation where the standard is silent: the canvas starts transparent; disposal method 2 ("restore to
//! background") makes the frame area transparent; methods 4-7 act like 0; the loop count is not read; a delay of 0 is reported as 0 ms;
//! output is atomic (any error returns no frames); a stream that ends cleanly after complete blocks without the 0x3B trailer is accepted
//! with a warning; a region outside the canvas is clipped.

/// Safety caps against decompression bombs (project choices, not from the standard). Checked before allocating.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_frames: usize,
    /// Pixels of the logical screen and of any single image region.
    pub max_frame_pixels: u64,
    /// Pixels of all output frames together (screen size x frame count).
    pub max_total_pixels: u64,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits { max_frames: 4096, max_frame_pixels: 16 * 1024 * 1024, max_total_pixels: 128 * 1024 * 1024 }
    }
}

/// One output frame: the whole canvas after drawing the image, RGBA with alpha 0 or 255.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub rgba: Vec<u8>,
    pub duration_ms: u32,
}

/// The decoded file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sequence {
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Frame>,
    pub warnings: Vec<String>,
}

struct Reader<'a> {
    d: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).filter(|e| *e <= self.d.len()).ok_or("truncated data")?;
        let s = &self.d[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn at_end(&self) -> bool {
        self.at >= self.d.len()
    }
    /// Concatenated payload of a run of sub-blocks, up to and including the zero-length terminator.
    fn sub_blocks(&mut self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        loop {
            let n = self.u8()? as usize;
            if n == 0 {
                return Ok(out);
            }
            out.extend_from_slice(self.take(n)?);
        }
    }
    fn skip_sub_blocks(&mut self) -> Result<(), String> {
        loop {
            let n = self.u8()? as usize;
            if n == 0 {
                return Ok(());
            }
            self.take(n)?;
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Control {
    disposal: u8,
    transparent: Option<u8>,
    delay_cs: u16,
}

fn read_table(r: &mut Reader, size_bits: u8) -> Result<Vec<[u8; 3]>, String> {
    let n = 1usize << (u32::from(size_bits & 7) + 1);
    let raw = r.take(n * 3)?;
    Ok(raw.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect())
}

/// Decodes the LZW data of one image into at most `want` colour indices.
fn lzw_decode(data: &[u8], min_bits: u8, want: usize) -> Result<Vec<u8>, String> {
    if !(1..=11).contains(&min_bits) {
        return Err(format!("invalid LZW minimum code size {min_bits}"));
    }
    let clear = 1u16 << min_bits;
    let eoi = clear + 1;
    let mut prefix = vec![0u16; 4096];
    let mut suffix = vec![0u8; 4096];
    let mut first = vec![0u8; 4096];
    for i in 0..clear as usize {
        suffix[i] = i as u8;
        first[i] = i as u8;
    }
    let mut next = clear + 2;
    let mut size = u32::from(min_bits) + 1;
    let mut prev: Option<u16> = None;
    let mut out: Vec<u8> = Vec::with_capacity(want.min(1 << 20));
    let (mut acc, mut nbits, mut pos) = (0u32, 0u32, 0usize);
    let mut stack: Vec<u8> = Vec::new();
    loop {
        while nbits < size {
            match data.get(pos) {
                Some(&b) => {
                    acc |= u32::from(b) << nbits;
                    nbits += 8;
                    pos += 1;
                }
                None => return Err("truncated data (LZW stream ended before the end-of-information code)".into()),
            }
        }
        let code = (acc & ((1 << size) - 1)) as u16;
        acc >>= size;
        nbits -= size;
        if code == clear {
            next = clear + 2;
            size = u32::from(min_bits) + 1;
            prev = None;
            continue;
        }
        if code == eoi {
            break;
        }
        match prev {
            None => {
                if code >= clear {
                    return Err(format!("invalid LZW code {code}"));
                }
                out.push(code as u8);
            }
            Some(p) => {
                if code > next || (code == next && next >= 4096) {
                    return Err(format!("invalid LZW code {code} (first free entry is {next})"));
                }
                // string of `code` (or, for the not-yet-defined code, string of prev + its first byte)
                let (start, extra) = if code < next { (code, None) } else { (p, Some(first[p as usize])) };
                stack.clear();
                let mut c = start;
                loop {
                    stack.push(suffix[c as usize]);
                    if c < clear {
                        break;
                    }
                    c = prefix[c as usize];
                }
                out.extend(stack.iter().rev());
                // the not-yet-defined code stands for prev + its first byte
                let entry_first = match extra {
                    Some(e) => {
                        out.push(e);
                        first[p as usize]
                    }
                    None => first[code as usize],
                };
                if next < 4096 {
                    prefix[next as usize] = p;
                    suffix[next as usize] = entry_first;
                    first[next as usize] = first[p as usize];
                    next += 1;
                    if u32::from(next) == (1 << size) && size < 12 {
                        size += 1;
                    }
                }
            }
        }
        prev = Some(code);
        if out.len() >= want {
            out.truncate(want);
            break;
        }
    }
    Ok(out)
}

fn deinterlace(rows_stored: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut order: Vec<usize> = Vec::with_capacity(h);
    for (start, step) in [(0usize, 8usize), (4, 8), (2, 4), (1, 2)] {
        order.extend((start..h).step_by(step));
    }
    let mut out = vec![0u8; w * h];
    for (k, &row) in order.iter().enumerate() {
        out[row * w..(row + 1) * w].copy_from_slice(&rows_stored[k * w..(k + 1) * w]);
    }
    out
}

/// Decodes a GIF file. Any error returns no frames.
pub fn decode(bytes: &[u8], limits: &Limits) -> Result<Sequence, String> {
    let mut r = Reader { d: bytes, at: 0 };
    let sig = r.take(6).map_err(|_| "not a GIF file (too short)".to_string())?;
    if sig != b"GIF87a" && sig != b"GIF89a" {
        return Err("not a GIF file (signature must be GIF87a or GIF89a)".into());
    }
    let (sw, sh) = (u32::from(r.u16()?), u32::from(r.u16()?));
    let packed = r.u8()?;
    let _background = r.u8()?;
    let _aspect = r.u8()?;
    if sw == 0 || sh == 0 {
        return Err("the logical screen has a zero dimension".into());
    }
    let canvas_px = u64::from(sw) * u64::from(sh);
    if canvas_px > limits.max_frame_pixels {
        return Err(format!("the logical screen ({sw}x{sh}) exceeds the limit of {} pixels", limits.max_frame_pixels));
    }
    let global = if packed & 0x80 != 0 { Some(read_table(&mut r, packed)?) } else { None };
    let (w, h) = (sw as usize, sh as usize);
    let mut canvas = vec![0u8; w * h * 4];
    let mut frames: Vec<Frame> = Vec::new();
    let mut warnings = Vec::new();
    let mut pending = Control::default();
    let mut terminated = false;
    while !r.at_end() {
        match r.u8()? {
            0x3B => {
                terminated = true;
                break;
            }
            0x21 => {
                let label = r.u8()?;
                if label == 0xF9 {
                    let n = r.u8()?;
                    if n != 4 {
                        return Err(format!("graphic control extension has block size {n}, expected 4"));
                    }
                    let b = r.take(4)?;
                    pending = Control { disposal: (b[0] >> 2) & 7, transparent: if b[0] & 1 != 0 { Some(b[3]) } else { None }, delay_cs: u16::from_le_bytes([b[1], b[2]]) };
                    if r.u8()? != 0 {
                        return Err("graphic control extension is not terminated".into());
                    }
                } else {
                    r.skip_sub_blocks()?;
                }
            }
            0x2C => {
                let (left, top) = (usize::from(r.u16()?), usize::from(r.u16()?));
                let (iw, ih) = (usize::from(r.u16()?), usize::from(r.u16()?));
                let ipacked = r.u8()?;
                if iw == 0 || ih == 0 {
                    return Err("an image has a zero dimension".into());
                }
                if (iw as u64) * (ih as u64) > limits.max_frame_pixels {
                    return Err("an image region exceeds the pixel limit".into());
                }
                if frames.len() >= limits.max_frames {
                    return Err(format!("more than {} frames", limits.max_frames));
                }
                if (frames.len() as u64 + 1) * canvas_px > limits.max_total_pixels {
                    return Err(format!("the decoded frames would exceed {} pixels in total", limits.max_total_pixels));
                }
                let local = if ipacked & 0x80 != 0 { Some(read_table(&mut r, ipacked)?) } else { None };
                let table = local.as_ref().or(global.as_ref()).ok_or("no colour table (neither global nor local)")?;
                let min_bits = r.u8()?;
                let data = r.sub_blocks()?;
                let idx = lzw_decode(&data, min_bits, iw * ih)?;
                if idx.len() < iw * ih {
                    return Err(format!("the image holds {} pixel indices, {} expected", idx.len(), iw * ih));
                }
                let idx = if ipacked & 0x40 != 0 { deinterlace(&idx, iw, ih) } else { idx };
                if idx.iter().any(|&i| usize::from(i) >= table.len()) {
                    return Err("a colour index is beyond the colour table".into());
                }
                let before = if pending.disposal == 3 { Some(canvas.clone()) } else { None };
                for y in 0..ih {
                    let cy = top + y;
                    if cy >= h {
                        break;
                    }
                    for x in 0..iw {
                        let cx = left + x;
                        if cx >= w {
                            break;
                        }
                        let i = idx[y * iw + x];
                        if pending.transparent == Some(i) {
                            continue;
                        }
                        let c = table[usize::from(i)];
                        let o = (cy * w + cx) * 4;
                        canvas[o..o + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
                    }
                }
                frames.push(Frame { rgba: canvas.clone(), duration_ms: u32::from(pending.delay_cs) * 10 });
                // disposal happens after display, before the next image
                let region = |f: &mut dyn FnMut(usize)| {
                    for y in top..(top + ih).min(h) {
                        for x in left..(left + iw).min(w) {
                            f((y * w + x) * 4);
                        }
                    }
                };
                match (pending.disposal, before) {
                    (2, _) => region(&mut |o| canvas[o..o + 4].copy_from_slice(&[0, 0, 0, 0])),
                    (3, Some(b)) => region(&mut |o| canvas[o..o + 4].copy_from_slice(&b[o..o + 4])),
                    _ => {}
                }
                pending = Control::default();
            }
            other => return Err(format!("unknown block introducer 0x{other:02x}")),
        }
    }
    if !terminated {
        if frames.is_empty() {
            return Err("truncated data (no image before the end of the file)".into());
        }
        warnings.push("the file has no 0x3B trailer".to_string());
    }
    Ok(Sequence { width: sw, height: sh, frames, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Vec<u8> {
        let s: String = s.split_whitespace().collect();
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    const VEC1: &str = "474946383961 0100 0100 90 00 00 000000 ffffff 2c 0000 0000 0100 0100 00 02 02 4c01 00 3b";

    /// Test encoder: textbook LZW with growing code widths and a Clear when the table is full.
    fn lzw_encode(indices: &[u8], min_bits: u8) -> Vec<u8> {
        use std::collections::HashMap;
        let clear = 1u32 << min_bits;
        let eoi = clear + 1;
        let mut codes: Vec<(u32, u32)> = Vec::new(); // (code, width)
        let mut size = u32::from(min_bits) + 1;
        let mut next = eoi + 1;
        let mut dict: HashMap<(u32, u8), u32> = HashMap::new();
        codes.push((clear, size));
        let mut cur: Option<u32> = None;
        for &b in indices {
            let Some(c) = cur else {
                cur = Some(u32::from(b));
                continue;
            };
            if let Some(&n) = dict.get(&(c, b)) {
                cur = Some(n);
                continue;
            }
            codes.push((c, size));
            if next < 4096 {
                dict.insert((c, b), next);
                if next == (1 << size) && size < 12 {
                    size += 1;
                }
                next += 1;
            } else {
                codes.push((clear, size));
                dict.clear();
                next = eoi + 1;
                size = u32::from(min_bits) + 1;
            }
            cur = Some(u32::from(b));
        }
        if let Some(c) = cur {
            codes.push((c, size));
        }
        codes.push((eoi, size));
        let (mut acc, mut n, mut out) = (0u64, 0u32, Vec::new());
        for (c, w) in codes {
            acc |= u64::from(c) << n;
            n += w;
            while n >= 8 {
                out.push((acc & 0xff) as u8);
                acc >>= 8;
                n -= 8;
            }
        }
        if n > 0 {
            out.push(acc as u8);
        }
        out
    }

    fn sub_blocks(data: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        for c in data.chunks(255) {
            v.push(c.len() as u8);
            v.extend_from_slice(c);
        }
        v.push(0);
        v
    }

    /// (control extension bytes, left, top, width, height, interlaced, indices)
    type TestImage = (Option<[u8; 4]>, u16, u16, u16, u16, bool, Vec<u8>);

    /// Builds a GIF with a 4-colour global table: red, green, blue, white.
    fn build(w: u16, h: u16, images: &[TestImage]) -> Vec<u8> {
        let mut g = b"GIF89a".to_vec();
        g.extend_from_slice(&w.to_le_bytes());
        g.extend_from_slice(&h.to_le_bytes());
        g.extend_from_slice(&[0x81, 0, 0]);
        g.extend_from_slice(&[255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]);
        for (gce, l, t, iw, ih, interlaced, idx) in images {
            if let Some(c) = gce {
                // c = [packed, delay lo, delay hi, transparent index]
                g.extend_from_slice(&[0x21, 0xF9, 4, c[0], c[1], c[2], c[3], 0]);
            }
            g.push(0x2C);
            for v in [l, t, iw, ih] {
                g.extend_from_slice(&v.to_le_bytes());
            }
            g.push(if *interlaced { 0x40 } else { 0 });
            g.push(2);
            g.extend(sub_blocks(&lzw_encode(idx, 2)));
        }
        g.push(0x3B);
        g
    }

    fn px(f: &Frame, i: usize) -> [u8; 4] {
        [f.rgba[i * 4], f.rgba[i * 4 + 1], f.rgba[i * 4 + 2], f.rgba[i * 4 + 3]]
    }

    #[test]
    fn single_white_pixel() {
        let s = decode(&hex(VEC1), &Limits::default()).unwrap();
        assert_eq!((s.width, s.height, s.frames.len()), (1, 1, 1));
        assert_eq!(px(&s.frames[0], 0), [255, 255, 255, 255]);
    }

    #[test]
    fn transparent_pixel_with_duration() {
        let mut v = hex(VEC1);
        let at = 19; // before the image descriptor
        let ext = hex("21f904 01 0700 01 00");
        v.splice(at..at, ext);
        let s = decode(&v, &Limits::default()).unwrap();
        assert_eq!(s.frames[0].duration_ms, 70);
        assert_eq!(px(&s.frames[0], 0)[3], 0);
    }

    #[test]
    fn interlaced_rows_are_put_back_in_order() {
        let g = build(1, 8, &[(None, 0, 0, 1, 8, true, vec![0, 1, 2, 3, 0, 1, 2, 3])]);
        // stored row k has index k only for k < 4 in this palette; use distinct colours through a longer check below
        let s = decode(&g, &Limits::default()).unwrap();
        let col: Vec<[u8; 4]> = (0..8).map(|i| px(&s.frames[0], i)).collect();
        let pal = [[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255], [255, 255, 255, 255]];
        // stored order -> output rows 0,4,2,6,1,3,5,7
        let stored = [0usize, 1, 2, 3, 0, 1, 2, 3];
        let rows = [0usize, 4, 2, 6, 1, 3, 5, 7];
        let mut want = [[0u8; 4]; 8];
        for (k, &r) in rows.iter().enumerate() {
            want[r] = pal[stored[k]];
        }
        assert_eq!(col, want.to_vec());
    }

    #[test]
    fn disposal_methods() {
        let red = vec![0u8, 0];
        let blue = vec![2u8];
        let mk = |dispose: u8| build(2, 1, &[(Some([dispose << 2, 5, 0, 0]), 0, 0, 2, 1, false, red.clone()), (Some([0, 5, 0, 0]), 1, 0, 1, 1, false, blue.clone())]);
        let s = decode(&mk(1), &Limits::default()).unwrap();
        assert_eq!(s.frames.len(), 2);
        assert_eq!((px(&s.frames[1], 0), px(&s.frames[1], 1)), ([255, 0, 0, 255], [0, 0, 255, 255]));
        let s = decode(&mk(3), &Limits::default()).unwrap();
        assert_eq!((px(&s.frames[1], 0), px(&s.frames[1], 1)), ([0, 0, 0, 0], [0, 0, 255, 255]));
        let s = decode(&mk(2), &Limits::default()).unwrap();
        assert_eq!(px(&s.frames[1], 0)[3], 0, "method 2 clears to transparent (project decision)");
        let s = decode(&mk(6), &Limits::default()).unwrap();
        assert_eq!(px(&s.frames[1], 0), [255, 0, 0, 255], "methods 4-7 act like 0");
        assert_eq!(s.frames[0].duration_ms, 50);
    }

    #[test]
    fn errors() {
        let mut v = hex(VEC1);
        v[3..6].copy_from_slice(b"90a");
        assert!(decode(&v, &Limits::default()).unwrap_err().contains("signature"));
        // second code 1 -> 7 (first free entry is 6): bits of 4c 01 are 100 001 | 101 ... codes 4,1,5 -> 4,7,5
        let mut v = hex(VEC1);
        let at = v.iter().position(|b| *b == 0x4c).unwrap();
        // codes (3 bits each, LSB first): clear=4, 7, eoi=5 -> 0b101_111_100 = 0x17c
        v[at] = 0x7c;
        v[at + 1] = 0x01;
        let e = decode(&v, &Limits::default()).unwrap_err();
        assert!(e.contains("invalid LZW code"), "{e}");
        // truncated inside the data sub-block
        let v = hex(VEC1);
        let e = decode(&v[..32], &Limits::default()).unwrap_err();
        assert!(e.contains("truncated"), "{e}");
        // too large a screen: no allocation
        let mut v = hex(VEC1);
        v[6..10].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        let lim = Limits { max_frame_pixels: 16 * 1024 * 1024, ..Limits::default() };
        assert!(decode(&v, &lim).unwrap_err().contains("exceeds the limit"));
        // zero dimension, bad block, no table
        let mut v = hex(VEC1);
        v[6] = 0;
        v[7] = 0;
        assert!(decode(&v, &Limits::default()).is_err());
        let mut v = hex(VEC1);
        v[10] = 0x00; // no global table
        assert!(decode(&v[..13], &Limits::default()).is_err());
        let mut v = hex(VEC1);
        v[19] = 0x55;
        assert!(decode(&v, &Limits::default()).unwrap_err().contains("unknown block"));
        assert!(decode(&[], &Limits::default()).is_err());
    }

    #[test]
    fn caps_on_frames_and_total_pixels() {
        let one = vec![0u8];
        let g = build(1, 1, &[(None, 0, 0, 1, 1, false, one.clone()), (None, 0, 0, 1, 1, false, one.clone()), (None, 0, 0, 1, 1, false, one)]);
        assert_eq!(decode(&g, &Limits::default()).unwrap().frames.len(), 3);
        assert!(decode(&g, &Limits { max_frames: 2, ..Limits::default() }).is_err());
        assert!(decode(&g, &Limits { max_total_pixels: 2, ..Limits::default() }).is_err());
    }

    #[test]
    fn long_images_exercise_code_growth_repeats_and_table_reset() {
        let pal = [[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255], [255, 255, 255, 255]];
        let check = |w: u16, h: u16, idx: Vec<u8>| {
            let g = build(w, h, &[(None, 0, 0, w, h, false, idx.clone())]);
            let s = decode(&g, &Limits::default()).unwrap();
            for (i, &c) in idx.iter().enumerate() {
                assert_eq!(px(&s.frames[0], i), pal[usize::from(c)], "pixel {i}");
            }
        };
        // long runs: the code that is not yet in the table (previous string + its first byte) occurs
        check(50, 40, vec![1; 2000]);
        // pseudo-random pixels: the table fills (4096 entries) and a Clear restarts it, several times
        let mut x = 12345u32;
        let idx: Vec<u8> = (0..30_000)
            .map(|_| {
                x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                ((x >> 16) & 3) as u8
            })
            .collect();
        check(200, 150, idx);
        let enc = lzw_encode(&vec![1u8; 2000], 2);
        assert!(enc.len() < 200, "the test encoder really compresses ({} bytes)", enc.len());
    }

    #[test]
    fn agrees_with_the_crate_based_importer_on_valid_files() {
        let mut x = 99u32;
        let idx: Vec<u8> = (0..60 * 30)
            .map(|_| {
                x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                ((x >> 20) & 3) as u8
            })
            .collect();
        let files = [
            build(60, 30, &[(None, 0, 0, 60, 30, false, idx.clone())]),
            build(60, 30, &[(None, 0, 0, 60, 30, true, idx.clone())]),
            build(2, 1, &[(Some([4, 5, 0, 0]), 0, 0, 2, 1, false, vec![0, 0]), (Some([0, 7, 0, 0]), 1, 0, 1, 1, false, vec![2])]),
            build(2, 1, &[(Some([12, 5, 0, 0]), 0, 0, 2, 1, false, vec![0, 1]), (Some([0, 7, 0, 0]), 1, 0, 1, 1, false, vec![2])]),
            build(4, 4, &[(Some([9, 3, 0, 1]), 1, 1, 2, 2, false, vec![0, 1, 2, 3]), (None, 0, 0, 4, 4, false, vec![1; 16])]),
        ];
        for (n, g) in files.iter().enumerate() {
            let mine = decode(g, &Limits::default()).unwrap();
            let theirs = crate::animimport::decode_gif(g).unwrap();
            assert_eq!(mine.frames.len(), theirs.frames.len(), "file {n}");
            for (i, (m, t)) in mine.frames.iter().zip(&theirs.frames).enumerate() {
                assert_eq!(m.duration_ms, t.1, "file {n} frame {i} delay");
                assert_eq!(m.rgba, t.0, "file {n} frame {i} pixels");
            }
        }
    }

    #[test]
    fn missing_trailer_is_accepted_with_a_warning_and_surplus_ignored() {
        let v = hex(VEC1);
        let s = decode(&v[..v.len() - 1], &Limits::default()).unwrap();
        assert_eq!(s.warnings.len(), 1);
        assert_eq!(s.frames.len(), 1);
        // more indices than width x height: the surplus is ignored; fewer: error
        let more = build(1, 1, &[(None, 0, 0, 1, 1, false, vec![2, 1, 0])]);
        assert_eq!(px(&decode(&more, &Limits::default()).unwrap().frames[0], 0), [0, 0, 255, 255]);
        let fewer = build(2, 1, &[(None, 0, 0, 2, 1, false, vec![2])]);
        assert!(decode(&fewer, &Limits::default()).unwrap_err().contains("expected"));
    }
}
