// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Colour quantisation (median cut), dithering, indexed frames, GIF and APNG encode/decode (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Colour quantisation (median cut), dithering, indexed frame sequences

#[derive(Default, Clone)]
pub(super) struct FastHash(u64);
impl std::hash::Hasher for FastHash {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = (self.0 ^ x as u64).wrapping_mul(0x100000001b3);
        }
    }
    fn write_u32(&mut self, x: u32) {
        self.0 = (x as u64).wrapping_mul(0x9E3779B97F4A7C15);
        self.0 ^= self.0 >> 29;
    }
}
pub(super) type FastMap<K, V> = HashMap<K, V, std::hash::BuildHasherDefault<FastHash>>;

pub(super) fn pack(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | b as u32
}

/// Median-cut palette (Heckbert 1982) of at most `max` colours from the opaque pixels of `frames`.
/// If there are no more distinct colours than `max` the palette is exact. Deterministic.
pub fn median_cut(frames: &[&[u8]], max: usize) -> Vec<[u8; 3]> {
    let max = max.max(1);
    let total: usize = frames.iter().map(|f| f.len() / 4).sum();
    let stride = (total / 3_000_000).max(1);
    let mut hist: FastMap<u32, u32> = FastMap::default();
    let mut k = 0usize;
    for f in frames {
        for p in f.chunks_exact(4) {
            k += 1;
            if k.is_multiple_of(stride) && p[3] >= 128 {
                *hist.entry(pack(p[0], p[1], p[2])).or_insert(0) += 1;
            }
        }
    }
    let mut items: Vec<(u32, u32)> = hist.into_iter().collect();
    items.sort_unstable();
    if items.len() <= max {
        return items.iter().map(|&(c, _)| [(c >> 16) as u8, (c >> 8) as u8, c as u8]).collect();
    }
    let ch = |c: u32, a: usize| ((c >> (16 - 8 * a)) & 255) as f64;
    // boxes are ranges of `items`
    struct B {
        lo: usize,
        hi: usize,
        sse: f64,
    }
    let sse_of = |items: &[(u32, u32)]| -> f64 {
        let mut w = 0.0;
        let mut m = [0.0; 3];
        for &(c, n) in items {
            w += n as f64;
            for (a, ma) in m.iter_mut().enumerate() {
                *ma += ch(c, a) * n as f64;
            }
        }
        if w == 0.0 {
            return 0.0;
        }
        let mean = [m[0] / w, m[1] / w, m[2] / w];
        items.iter().map(|&(c, n)| (0..3).map(|a| (ch(c, a) - mean[a]).powi(2)).sum::<f64>() * n as f64).sum()
    };
    let mut boxes = vec![B { lo: 0, hi: items.len(), sse: sse_of(&items) }];
    while boxes.len() < max {
        let Some((bi, _)) = boxes.iter().enumerate().filter(|(_, b)| b.hi - b.lo > 1).max_by(|a, b| a.1.sse.total_cmp(&b.1.sse).then(b.0.cmp(&a.0))) else { break };
        let (lo, hi) = (boxes[bi].lo, boxes[bi].hi);
        // longest axis
        let mut mn = [255.0f64; 3];
        let mut mx = [0.0f64; 3];
        for &(c, _) in &items[lo..hi] {
            for a in 0..3 {
                mn[a] = mn[a].min(ch(c, a));
                mx[a] = mx[a].max(ch(c, a));
            }
        }
        let axis = (0..3).max_by(|&a, &b| (mx[a] - mn[a]).total_cmp(&(mx[b] - mn[b])).then(b.cmp(&a))).unwrap();
        items[lo..hi].sort_by(|x, y| ch(x.0, axis).total_cmp(&ch(y.0, axis)).then(x.0.cmp(&y.0)));
        let pop: u64 = items[lo..hi].iter().map(|x| x.1 as u64).sum();
        let mut acc = 0u64;
        let mut cut = lo + 1;
        for (i, x) in items[lo..hi].iter().enumerate() {
            acc += x.1 as u64;
            if acc * 2 >= pop {
                cut = lo + i + 1;
                break;
            }
        }
        let cut = cut.clamp(lo + 1, hi - 1);
        let (s1, s2) = (sse_of(&items[lo..cut]), sse_of(&items[cut..hi]));
        boxes[bi] = B { lo, hi: cut, sse: s1 };
        boxes.push(B { lo: cut, hi, sse: s2 });
    }
    boxes.sort_by_key(|b| b.lo);
    boxes
        .iter()
        .map(|b| {
            let mut w = 0.0;
            let mut m = [0.0; 3];
            for &(c, n) in &items[b.lo..b.hi] {
                w += n as f64;
                for (a, ma) in m.iter_mut().enumerate() {
                    *ma += ch(c, a) * n as f64;
                }
            }
            [(m[0] / w).round() as u8, (m[1] / w).round() as u8, (m[2] / w).round() as u8]
        })
        .collect()
}

pub(super) struct Mapper<'a> {
    pub(super) pal: &'a [[u8; 3]],
    pub(super) base: u8,
    pub(super) cache: FastMap<u32, u8>,
}

impl<'a> Mapper<'a> {
    pub(super) fn new(pal: &'a [[u8; 3]], base: u8) -> Self {
        Mapper { pal, base, cache: FastMap::default() }
    }
    pub(super) fn nearest(&mut self, r: u8, g: u8, b: u8) -> u8 {
        let key = pack(r, g, b);
        if let Some(&i) = self.cache.get(&key) {
            return i;
        }
        let mut best = (u32::MAX, 0usize);
        for (i, c) in self.pal.iter().enumerate() {
            let d = (c[0] as i32 - r as i32).pow(2) as u32 * 3 + (c[1] as i32 - g as i32).pow(2) as u32 * 4 + (c[2] as i32 - b as i32).pow(2) as u32 * 2;
            if d < best.0 {
                best = (d, i);
            }
        }
        let idx = self.base + best.1 as u8;
        self.cache.insert(key, idx);
        idx
    }
}

/// Frames reduced to a shared palette. Index 0 is reserved (transparent / "unchanged") when
/// `transparent` is set; colours follow.
pub struct Indexed {
    pub w: usize,
    pub h: usize,
    /// Full palette including the reserved entry (black) at index 0 when `transparent`.
    pub palette: Vec<[u8; 3]>,
    pub transparent: bool,
    pub frames: Vec<Vec<u8>>,
}

/// Quantise `frames` (straight RGBA, all `w x h`) to at most `colors` palette entries in total.
/// Pixels with alpha < 128 map to the reserved index 0 (so `reserve` must be true if any exist).
pub fn quantize<F: AsRef<[u8]>>(frames: &[F], w: usize, h: usize, colors: usize, dither: bool, reserve: bool) -> Indexed {
    let colors = colors.clamp(2, 256);
    let ncol = colors - reserve as usize;
    let refs: Vec<&[u8]> = frames.iter().map(|f| f.as_ref()).collect();
    let pal = median_cut(&refs, ncol);
    let base = reserve as u8;
    let mut full: Vec<[u8; 3]> = Vec::new();
    if reserve {
        full.push([0, 0, 0]);
    }
    full.extend(&pal);
    let mut mapper = Mapper::new(&pal, base);
    let mut out = Vec::with_capacity(frames.len());
    for fr in frames {
        let fr = fr.as_ref();
        let mut idx = vec![0u8; w * h];
        if dither && !pal.is_empty() {
            // Floyd-Steinberg, left to right, two error rows
            let mut cur = vec![[0.0f32; 3]; w + 2];
            let mut nxt = vec![[0.0f32; 3]; w + 2];
            for y in 0..h {
                for e in nxt.iter_mut() {
                    *e = [0.0; 3];
                }
                for x in 0..w {
                    let o = (y * w + x) * 4;
                    if fr[o + 3] < 128 {
                        idx[y * w + x] = 0;
                        continue;
                    }
                    let mut v = [0u8; 3];
                    let mut want = [0.0f32; 3];
                    for a in 0..3 {
                        want[a] = (fr[o + a] as f32 + cur[x + 1][a]).clamp(0.0, 255.0);
                        v[a] = want[a].round() as u8;
                    }
                    let i = mapper.nearest(v[0], v[1], v[2]);
                    idx[y * w + x] = i;
                    let c = full[i as usize];
                    for a in 0..3 {
                        let err = want[a] - c[a] as f32;
                        cur[x + 2][a] += err * 7.0 / 16.0;
                        nxt[x][a] += err * 3.0 / 16.0;
                        nxt[x + 1][a] += err * 5.0 / 16.0;
                        nxt[x + 2][a] += err * 1.0 / 16.0;
                    }
                }
                std::mem::swap(&mut cur, &mut nxt);
            }
        } else {
            for (p, slot) in idx.iter_mut().enumerate().take(w * h) {
                let o = p * 4;
                *slot = if fr[o + 3] < 128 { 0 } else { mapper.nearest(fr[o], fr[o + 1], fr[o + 2]) };
            }
        }
        out.push(idx);
    }
    Indexed { w, h, palette: full, transparent: reserve, frames: out }
}

/// One output frame: a rectangle of indices.
pub struct Rect {
    pub left: usize,
    pub top: usize,
    pub w: usize,
    pub h: usize,
    pub idx: Vec<u8>,
}

/// Inter-frame deltas. Frame 0 is whole. Later frames hold only the bounding box of changed pixels;
/// unchanged pixels inside it carry index 0 (transparent). A pixel counts as unchanged if its palette
/// colour is within `tol` levels (per channel) of what the previous output left on screen, so the error
/// to the true indexed frame never exceeds `tol`. Requires `q.transparent`.
pub fn delta_rects(q: &Indexed, tol: u8) -> Vec<Rect> {
    let (w, h) = (q.w, q.h);
    let mut out = Vec::new();
    let mut shown: Vec<u8> = Vec::new();
    for (k, fr) in q.frames.iter().enumerate() {
        if k == 0 {
            out.push(Rect { left: 0, top: 0, w, h, idx: fr.clone() });
            shown = fr.clone();
            continue;
        }
        let same = |a: u8, b: u8| -> bool {
            if a == b {
                return true;
            }
            if tol == 0 || a == 0 || b == 0 {
                return false;
            }
            let (ca, cb) = (q.palette[a as usize], q.palette[b as usize]);
            (0..3).all(|c| (ca[c] as i32 - cb[c] as i32).abs() <= tol as i32)
        };
        let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
        for y in 0..h {
            for x in 0..w {
                if !same(fr[y * w + x], shown[y * w + x]) {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        if x1 <= x0 {
            out.push(Rect { left: 0, top: 0, w: 1, h: 1, idx: vec![0] });
            continue;
        }
        let (rw, rh) = (x1 - x0, y1 - y0);
        let mut idx = vec![0u8; rw * rh];
        for y in 0..rh {
            for x in 0..rw {
                let p = (y + y0) * w + x + x0;
                if !same(fr[p], shown[p]) {
                    idx[y * rw + x] = fr[p];
                    shown[p] = fr[p];
                }
            }
        }
        out.push(Rect { left: x0, top: y0, w: rw, h: rh, idx });
    }
    out
}

pub(super) fn delay_cs(fps: f32, step: u32) -> u16 {
    ((100.0 * step as f32 / fps.max(0.001)).round() as u16).max(2)
}

/// Looping GIF with a global palette, optional deltas. `alpha` = frames contain transparent pixels
/// (then no deltas are used and every frame is redrawn on a cleared canvas).
pub fn gif_encode(q: &Indexed, fps: f32, step: u32, delta: bool, delta_tol: u8) -> Result<Vec<u8>, String> {
    if q.w > u16::MAX as usize || q.h > u16::MAX as usize {
        return Err("image too large for GIF".into());
    }
    let nent = q.palette.len().max(2).next_power_of_two();
    let mut pal = Vec::with_capacity(nent * 3);
    for i in 0..nent {
        let c = q.palette.get(i).copied().unwrap_or([0, 0, 0]);
        pal.extend_from_slice(&c);
    }
    let mut out = Vec::new();
    {
        let mut e = gif::Encoder::new(&mut out, q.w as u16, q.h as u16, &pal).map_err(|e| e.to_string())?;
        e.set_repeat(gif::Repeat::Infinite).map_err(|e| e.to_string())?;
        let delay = delay_cs(fps, step);
        let rects: Vec<Rect> = if delta && q.transparent && q.frames.len() > 1 {
            delta_rects(q, delta_tol)
        } else {
            q.frames.iter().map(|f| Rect { left: 0, top: 0, w: q.w, h: q.h, idx: f.clone() }).collect()
        };
        for (k, r) in rects.into_iter().enumerate() {
            let mut fr = gif::Frame {
                left: r.left as u16,
                top: r.top as u16,
                width: r.w as u16,
                height: r.h as u16,
                delay,
                buffer: std::borrow::Cow::Owned(r.idx),
                ..gif::Frame::default()
            };
            if q.transparent {
                fr.transparent = Some(0);
                // with deltas the canvas is kept; with real transparency each frame clears the previous one
                fr.dispose = if delta && q.frames.len() > 1 { gif::DisposalMethod::Keep } else { gif::DisposalMethod::Background };
            }
            let _ = k;
            e.write_frame(&fr).map_err(|e| e.to_string())?;
        }
    }
    Ok(out)
}

/// Decode a GIF to full composited RGBA frames (`Keep` / `Background` disposal; others treated as Keep).
pub fn gif_decode(bytes: &[u8]) -> Result<(usize, usize, Vec<Vec<u8>>), String> {
    let mut opts = gif::DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);
    let mut d = opts.read_info(bytes).map_err(|e| e.to_string())?;
    let (w, h) = (d.width() as usize, d.height() as usize);
    let mut canvas = vec![0u8; w * h * 4];
    let mut frames = Vec::new();
    let mut clear: Option<(usize, usize, usize, usize)> = None;
    while let Some(fr) = d.read_next_frame().map_err(|e| e.to_string())? {
        if let Some((l, t, rw, rh)) = clear.take() {
            for y in t..(t + rh).min(h) {
                for x in l..(l + rw).min(w) {
                    canvas[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
        let (l, t, rw, rh) = (fr.left as usize, fr.top as usize, fr.width as usize, fr.height as usize);
        for y in 0..rh {
            for x in 0..rw {
                let (cx, cy) = (l + x, t + y);
                if cx >= w || cy >= h {
                    continue;
                }
                let s = (y * rw + x) * 4;
                if fr.buffer[s + 3] > 0 {
                    let o = (cy * w + cx) * 4;
                    canvas[o..o + 4].copy_from_slice(&fr.buffer[s..s + 4]);
                }
            }
        }
        if fr.dispose == gif::DisposalMethod::Background {
            clear = Some((l, t, rw, rh));
        }
        frames.push(canvas.clone());
    }
    Ok((w, h, frames))
}

pub(super) fn bit_depth(n: usize) -> (png::BitDepth, usize) {
    if n <= 2 {
        (png::BitDepth::One, 1)
    } else if n <= 4 {
        (png::BitDepth::Two, 2)
    } else if n <= 16 {
        (png::BitDepth::Four, 4)
    } else {
        (png::BitDepth::Eight, 8)
    }
}

pub(super) fn pack_rows(idx: &[u8], w: usize, h: usize, bits: usize) -> Vec<u8> {
    if bits == 8 {
        return idx.to_vec();
    }
    let per = 8 / bits;
    let row_bytes = w.div_ceil(per);
    let mut out = vec![0u8; row_bytes * h];
    for y in 0..h {
        for x in 0..w {
            let v = idx[y * w + x] << (8 - bits * (x % per + 1));
            out[y * row_bytes + x / per] |= v;
        }
    }
    out
}

/// Looping APNG from an indexed sequence (palette + tRNS entry 0, bit depth by palette size, optional deltas).
pub fn apng_encode_indexed(q: &Indexed, fps: f32, step: u32, delta: bool, delta_tol: u8) -> Result<Vec<u8>, String> {
    let n = q.frames.len() as u32;
    if n == 0 {
        return Err("no frames".into());
    }
    let (depth, bits) = bit_depth(q.palette.len());
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, q.w as u32, q.h as u32);
        enc.set_color(png::ColorType::Indexed);
        enc.set_depth(depth);
        enc.set_compression(png::Compression::High);
        enc.set_filter(png::Filter::Adaptive);
        enc.set_palette(q.palette.iter().flat_map(|c| c.iter().copied()).collect::<Vec<u8>>());
        if q.transparent {
            enc.set_trns(vec![0u8]);
        }
        enc.set_animated(n, 0).map_err(|e| e.to_string())?;
        // not `clamp`: a NaN `fps` must land on the lower bound as `max` does, `clamp` would pass NaN through
        #[allow(clippy::manual_clamp)]
        enc.set_frame_delay((100 * step).min(65535) as u16, (fps * 100.0).round().max(1.0).min(65535.0) as u16).map_err(|e| e.to_string())?;
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        let rects: Vec<Rect> = if delta && q.transparent && q.frames.len() > 1 {
            delta_rects(q, delta_tol)
        } else {
            q.frames.iter().map(|f| Rect { left: 0, top: 0, w: q.w, h: q.h, idx: f.clone() }).collect()
        };
        for (k, r) in rects.iter().enumerate() {
            if k > 0 {
                wr.set_frame_dimension(r.w as u32, r.h as u32).map_err(|e| e.to_string())?;
                wr.set_frame_position(r.left as u32, r.top as u32).map_err(|e| e.to_string())?;
                let over = delta && q.transparent;
                wr.set_blend_op(if over { png::BlendOp::Over } else { png::BlendOp::Source }).map_err(|e| e.to_string())?;
                wr.set_dispose_op(png::DisposeOp::None).map_err(|e| e.to_string())?;
            }
            wr.write_image_data(&pack_rows(&r.idx, r.w, r.h, bits)).map_err(|e| e.to_string())?;
        }
        wr.finish().map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// Truecolour looping APNG at maximum deflate effort with adaptive filtering.
pub fn apng_encode_rgba<F: AsRef<[u8]>>(w: usize, h: usize, frames: &[F], fps: f32, step: u32) -> Result<Vec<u8>, String> {
    if frames.is_empty() {
        return Err("no frames".into());
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w as u32, h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::High);
        enc.set_filter(png::Filter::Adaptive);
        enc.set_animated(frames.len() as u32, 0).map_err(|e| e.to_string())?;
        // not `clamp`: a NaN `fps` must land on the lower bound as `max` does, `clamp` would pass NaN through
        #[allow(clippy::manual_clamp)]
        enc.set_frame_delay((100 * step).min(65535) as u16, (fps * 100.0).round().max(1.0).min(65535.0) as u16).map_err(|e| e.to_string())?;
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        for f in frames {
            wr.write_image_data(f.as_ref()).map_err(|e| e.to_string())?;
        }
        wr.finish().map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// Decode an APNG to full composited RGBA frames (blend `Source` / `Over`; dispose `None` / `Background` /
/// `Previous`).
pub fn apng_decode(bytes: &[u8]) -> Result<(usize, usize, Vec<Vec<u8>>), String> {
    crate::assets::check_png(bytes)?;
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
    dec.set_transformations(png::Transformations::EXPAND);
    let mut r = dec.read_info().map_err(|e| e.to_string())?;
    let (w, h) = (r.info().width as usize, r.info().height as usize);
    let n = r.info().animation_control.map_or(1, |a| a.num_frames as usize);
    let mut canvas = vec![0u8; w * h * 4];
    let mut frames = Vec::new();
    for _ in 0..n {
        let mut buf = vec![0u8; r.output_buffer_size().ok_or("png buffer size overflow")?];
        let oi = r.next_frame(&mut buf).map_err(|e| e.to_string())?;
        let fc = r.info().frame_control;
        let (fx, fy, fw, fh, blend, dispose) = match fc {
            Some(c) => (c.x_offset as usize, c.y_offset as usize, c.width as usize, c.height as usize, c.blend_op, c.dispose_op),
            None => (0, 0, w, h, png::BlendOp::Source, png::DisposeOp::None),
        };
        let px: Vec<u8> = match oi.color_type {
            png::ColorType::Rgba => buf[..fw * fh * 4].to_vec(),
            png::ColorType::Rgb => buf[..fw * fh * 3].chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
            png::ColorType::GrayscaleAlpha => buf[..fw * fh * 2].chunks_exact(2).flat_map(|c| [c[0], c[0], c[0], c[1]]).collect(),
            png::ColorType::Grayscale => buf[..fw * fh].iter().flat_map(|&c| [c, c, c, 255]).collect(),
            png::ColorType::Indexed => return Err("indexed output after expand".into()),
        };
        let before = canvas.clone();
        for y in 0..fh {
            for x in 0..fw {
                let (cx, cy) = (fx + x, fy + y);
                if cx >= w || cy >= h {
                    continue;
                }
                let s = (y * fw + x) * 4;
                let o = (cy * w + cx) * 4;
                match blend {
                    png::BlendOp::Source => canvas[o..o + 4].copy_from_slice(&px[s..s + 4]),
                    png::BlendOp::Over => {
                        let a = px[s + 3] as u32;
                        if a == 255 {
                            canvas[o..o + 4].copy_from_slice(&px[s..s + 4]);
                        } else if a > 0 {
                            let da = canvas[o + 3] as u32;
                            let oa = a + da * (255 - a) / 255;
                            for c in 0..3 {
                                canvas[o + c] = ((px[s + c] as u32 * a + canvas[o + c] as u32 * da * (255 - a) / 255) / oa.max(1)) as u8;
                            }
                            canvas[o + 3] = oa as u8;
                        }
                    }
                }
            }
        }
        frames.push(canvas.clone());
        match dispose {
            png::DisposeOp::None => {}
            png::DisposeOp::Background => {
                for y in fy..(fy + fh).min(h) {
                    for x in fx..(fx + fw).min(w) {
                        canvas[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[0, 0, 0, 0]);
                    }
                }
            }
            png::DisposeOp::Previous => canvas = before,
        }
    }
    Ok((w, h, frames))
}
