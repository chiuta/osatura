// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `autotrace` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// options and result types
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Fill,
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quant {
    KMeans,
    MedianCut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZOrder {
    /// Large outlines at the back (correct for nested regions). Default.
    Area,
    /// Light colours at the back, dark in front (can put a nested region under its parent).
    Luminance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    Bezier,
    /// Douglas-Peucker simplified polygon.
    Polygon,
    /// Visvalingam-Whyatt simplified polygon (`tol` is the smallest triangle area kept, in px squared, times 2).
    PolygonVw,
}

#[derive(Clone, Debug)]
pub struct TraceOptions {
    pub mode: Mode,
    pub colors: usize,
    pub quant: Quant,
    pub seed: u64,
    /// Smallest region kept, in pixels (despeckle).
    pub min_area: f32,
    /// Curve fitting tolerance in pixels.
    pub tol: f32,
    /// Smoothing passes of the [1 2 1] filter on the outline (0 = none).
    pub smooth: usize,
    /// Turning angle in degrees over the corner window above which a point is a corner.
    pub corner_angle: f32,
    /// Corner window in samples.
    pub corner_window: usize,
    /// Majority filter passes on the label map before tracing (removes one-pixel slivers).
    pub denoise: usize,
    pub zorder: ZOrder,
    pub fit: Fit,
    /// Line mode: luminance threshold 0..255 (None = Otsu). Ink is darker than the threshold.
    pub threshold: Option<u8>,
    /// Line mode: ink is lighter than the background.
    pub invert: bool,
    /// Line mode: skeleton branches shorter than this many pixels that end in a free tip are removed.
    pub prune: usize,
}

impl Default for TraceOptions {
    fn default() -> Self {
        TraceOptions {
            mode: Mode::Fill,
            colors: 8,
            quant: Quant::KMeans,
            seed: 1,
            min_area: 8.0,
            tol: 1.0,
            smooth: 3,
            corner_angle: 55.0,
            corner_window: 5,
            denoise: 0,
            zorder: ZOrder::Area,
            fit: Fit::Bezier,
            threshold: None,
            invert: false,
            prune: 4,
        }
    }
}

impl TraceOptions {
    pub fn validate(&self) -> Result<(), String> {
        if self.colors < 2 || self.colors > 64 {
            return Err("colors must be 2..64".into());
        }
        if !self.min_area.is_finite() || self.min_area < 0.0 {
            return Err("min area must be >= 0".into());
        }
        if !self.tol.is_finite() || self.tol <= 0.0 || self.tol > 50.0 {
            return Err("tolerance must be in (0, 50]".into());
        }
        if self.smooth > 50 || self.denoise > 5 || self.prune > 1000 {
            return Err("smooth <= 50, denoise <= 5, prune <= 1000".into());
        }
        if !self.corner_angle.is_finite() || self.corner_angle <= 5.0 || self.corner_angle >= 175.0 {
            return Err("corner angle must be in (5, 175) degrees".into());
        }
        if self.corner_window < 2 || self.corner_window > 40 {
            return Err("corner window must be 2..40".into());
        }
        Ok(())
    }
}

/// Decoded bitmap, straight (non-premultiplied) RGBA.
#[derive(Clone, Debug)]
pub struct Image {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

impl Image {
    pub fn from_rgba(w: usize, h: usize, rgba: Vec<u8>) -> Result<Image, String> {
        if w == 0 || h == 0 || rgba.len() != w * h * 4 {
            return Err("image size does not match its pixel data".into());
        }
        if w * h > 16_000_000 {
            return Err("image is larger than 16 million pixels".into());
        }
        Ok(Image { w, h, rgba })
    }
}

/// One traced area: an outline and the holes that must be cut out (transparent holes only).
#[derive(Clone, Debug)]
pub struct Region {
    pub color: [u8; 3],
    pub outer: VPath,
    pub holes: Vec<VPath>,
    /// Pixels of the region itself.
    pub pixels: usize,
    /// Area inside the outline in px squared (holes not subtracted); the z-order key.
    pub outer_area: f32,
}

/// One centre line of the line-art mode.
#[derive(Clone, Debug)]
pub struct Stroke {
    pub path: VPath,
    pub width: f32,
}

#[derive(Clone, Debug)]
pub struct TraceResult {
    pub width: usize,
    pub height: usize,
    pub mode: Mode,
    pub palette: Vec<[u8; 3]>,
    /// Back to front.
    pub regions: Vec<Region>,
    pub strokes: Vec<Stroke>,
    pub stroke_color: [u8; 3],
}

impl TraceResult {
    pub fn path_count(&self) -> usize {
        self.regions.iter().map(|r| 1 + r.holes.len()).sum::<usize>() + self.strokes.len()
    }
    pub fn anchor_count(&self) -> usize {
        self.regions.iter().map(|r| r.outer.anchors.len() + r.holes.iter().map(|h| h.anchors.len()).sum::<usize>()).sum::<usize>()
            + self.strokes.iter().map(|s| s.path.anchors.len()).sum::<usize>()
    }
}

// ---------------------------------------------------------------------------------------------
// decoding
// ---------------------------------------------------------------------------------------------

/// Decode PNG or baseline/progressive JPEG (8-bit grey or RGB).
pub fn decode_image(bytes: &[u8]) -> Result<Image, String> {
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        let mut d = jpeg_decoder::Decoder::new(bytes);
        d.read_info().map_err(|e| format!("invalid JPEG: {e}"))?;
        let info = d.info().ok_or("invalid JPEG")?;
        crate::assets::check_dims(info.width as u32, info.height as u32)?;
        let (w, h) = (info.width as usize, info.height as usize);
        let px = d.decode().map_err(|e| format!("invalid JPEG: {e}"))?;
        let rgba: Vec<u8> = match info.pixel_format {
            jpeg_decoder::PixelFormat::L8 => px.iter().flat_map(|&g| [g, g, g, 255]).collect(),
            jpeg_decoder::PixelFormat::RGB24 => px.chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
            _ => return Err("only 8-bit grey and RGB JPEGs are supported".into()),
        };
        return Image::from_rgba(w, h, rgba);
    }
    let pm = crate::assets::decode_png_checked(bytes).map_err(|e| format!("not a readable PNG or JPEG: {e}"))?;
    let mut rgba = Vec::with_capacity(pm.pixels().len() * 4);
    for px in pm.pixels() {
        let c = px.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Image::from_rgba(pm.width() as usize, pm.height() as usize, rgba)
}

// ---------------------------------------------------------------------------------------------
// small deterministic helpers
// ---------------------------------------------------------------------------------------------

pub(super) struct Rng( pub(super) u64);

impl Rng {
    pub(super) fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03 | 1)
    }
    pub(super) fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub(super) fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

pub(super) fn lum(c: [u8; 3]) -> f64 {
    0.2126 * c[0] as f64 + 0.7152 * c[1] as f64 + 0.0722 * c[2] as f64
}

// ---------------------------------------------------------------------------------------------
// colour quantisation
// ---------------------------------------------------------------------------------------------

pub(super) const TRANSP: u16 = u16::MAX;

pub(super) type V3 = [f64; 3];

pub(super) fn d2(a: &V3, b: &V3) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

pub(super) fn kmeans(pts: &[(V3, f64)], k: usize, seed: u64) -> Vec<V3> {
    let k = k.min(pts.len());
    let mut rng = Rng::new(seed);
    let total: f64 = pts.iter().map(|p| p.1).sum();
    // weighted k-means++ seeding
    let mut pick = rng.unit() * total;
    let mut first = 0;
    for (i, p) in pts.iter().enumerate() {
        pick -= p.1;
        if pick <= 0.0 {
            first = i;
            break;
        }
    }
    let mut c: Vec<V3> = vec![pts[first].0];
    let mut dist: Vec<f64> = pts.iter().map(|p| d2(&p.0, &c[0])).collect();
    while c.len() < k {
        let s: f64 = pts.iter().zip(&dist).map(|(p, d)| p.1 * d).sum();
        if s <= 0.0 {
            break;
        }
        let mut pick = rng.unit() * s;
        let mut chosen = pts.len() - 1;
        for (i, (p, d)) in pts.iter().zip(&dist).enumerate() {
            pick -= p.1 * d;
            if pick <= 0.0 && *d > 0.0 {
                chosen = i;
                break;
            }
        }
        c.push(pts[chosen].0);
        for (i, p) in pts.iter().enumerate() {
            dist[i] = dist[i].min(d2(&p.0, c.last().unwrap()));
        }
    }
    // Lloyd iterations
    let mut assign = vec![usize::MAX; pts.len()];
    for _ in 0..40 {
        let mut changed = false;
        for (i, p) in pts.iter().enumerate() {
            let mut best = (f64::MAX, 0);
            for (j, cj) in c.iter().enumerate() {
                let d = d2(&p.0, cj);
                if d < best.0 {
                    best = (d, j);
                }
            }
            if assign[i] != best.1 {
                assign[i] = best.1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
        let mut sum = vec![[0.0f64; 3]; c.len()];
        let mut wt = vec![0.0f64; c.len()];
        for (i, p) in pts.iter().enumerate() {
            let j = assign[i];
            for (s, &x) in sum[j].iter_mut().zip(&p.0) {
                *s += x * p.1;
            }
            wt[j] += p.1;
        }
        for j in 0..c.len() {
            if wt[j] > 0.0 {
                c[j] = [sum[j][0] / wt[j], sum[j][1] / wt[j], sum[j][2] / wt[j]];
            }
        }
    }
    c
}

pub(super) fn median_cut(pts: &[(V3, f64)], k: usize) -> Vec<V3> {
    let mut boxes: Vec<Vec<usize>> = vec![(0..pts.len()).collect()];
    let range = |b: &Vec<usize>| -> (usize, f64) {
        let mut best = (0, 0.0);
        for ch in 0..3 {
            let lo = b.iter().map(|&i| pts[i].0[ch]).fold(f64::MAX, f64::min);
            let hi = b.iter().map(|&i| pts[i].0[ch]).fold(f64::MIN, f64::max);
            if hi - lo > best.1 {
                best = (ch, hi - lo);
            }
        }
        best
    };
    while boxes.len() < k {
        let mut pick: Option<(usize, f64)> = None;
        for (bi, b) in boxes.iter().enumerate() {
            if b.len() < 2 {
                continue;
            }
            let r = range(b).1;
            if r > 0.0 && pick.is_none_or(|p| r > p.1) {
                pick = Some((bi, r));
            }
        }
        let Some((bi, _)) = pick else { break };
        let mut b = boxes.swap_remove(bi);
        let ch = range(&b).0;
        b.sort_by(|&x, &y| pts[x].0[ch].total_cmp(&pts[y].0[ch]).then(x.cmp(&y)));
        let total: f64 = b.iter().map(|&i| pts[i].1).sum();
        let mut acc = 0.0;
        let mut cut = 1;
        for (n, &i) in b.iter().enumerate() {
            acc += pts[i].1;
            if acc >= total / 2.0 {
                cut = (n + 1).clamp(1, b.len() - 1);
                break;
            }
        }
        let right = b.split_off(cut);
        boxes.push(b);
        boxes.push(right);
    }
    boxes
        .iter()
        .map(|b| {
            let w: f64 = b.iter().map(|&i| pts[i].1).sum();
            let mut m = [0.0; 3];
            for &i in b {
                for (mc, &x) in m.iter_mut().zip(&pts[i].0) {
                    *mc += x * pts[i].1;
                }
            }
            [m[0] / w, m[1] / w, m[2] / w]
        })
        .collect()
}

/// Palette (sorted dark to light) and per-pixel labels (`TRANSP` for alpha < 128).
pub fn quantise(img: &Image, k: usize, method: Quant, seed: u64) -> (Vec<[u8; 3]>, Vec<u16>) {
    let n = img.w * img.h;
    let opaque = |i: usize| img.rgba[i * 4 + 3] >= 128;
    let col = |i: usize| [img.rgba[i * 4], img.rgba[i * 4 + 1], img.rgba[i * 4 + 2]];
    let mut exact: BTreeMap<[u8; 3], ()> = BTreeMap::new();
    for i in 0..n {
        if opaque(i) {
            exact.insert(col(i), ());
            if exact.len() > k {
                break;
            }
        }
    }
    let mut labels = vec![TRANSP; n];
    if exact.len() <= k {
        let mut pal: Vec<[u8; 3]> = exact.keys().copied().collect();
        pal.sort_by(|a, b| lum(*a).total_cmp(&lum(*b)).then(a.cmp(b)));
        let idx: BTreeMap<[u8; 3], u16> = pal.iter().enumerate().map(|(i, c)| (*c, i as u16)).collect();
        for i in 0..n {
            if opaque(i) {
                labels[i] = idx[&col(i)];
            }
        }
        return (pal, labels);
    }
    // 5-bit histogram
    let bin = |c: [u8; 3]| ((c[0] as usize >> 3) << 10) | ((c[1] as usize >> 3) << 5) | (c[2] as usize >> 3);
    let mut cnt = vec![0.0f64; 32768];
    let mut sum = vec![[0.0f64; 3]; 32768];
    for i in 0..n {
        if opaque(i) {
            let c = col(i);
            let b = bin(c);
            cnt[b] += 1.0;
            for ch in 0..3 {
                sum[b][ch] += c[ch] as f64;
            }
        }
    }
    let bins: Vec<usize> = (0..32768).filter(|&b| cnt[b] > 0.0).collect();
    let pts: Vec<(V3, f64)> = bins.iter().map(|&b| ([sum[b][0] / cnt[b], sum[b][1] / cnt[b], sum[b][2] / cnt[b]], cnt[b])).collect();
    let centres = match method {
        Quant::KMeans => kmeans(&pts, k, seed),
        Quant::MedianCut => median_cut(&pts, k),
    };
    let mut bin_label = vec![0usize; 32768];
    let mut psum = vec![[0.0f64; 3]; centres.len()];
    let mut pwt = vec![0.0f64; centres.len()];
    for (pi, &b) in bins.iter().enumerate() {
        let mut best = (f64::MAX, 0);
        for (j, c) in centres.iter().enumerate() {
            let d = d2(&pts[pi].0, c);
            if d < best.0 {
                best = (d, j);
            }
        }
        bin_label[b] = best.1;
        for ch in 0..3 {
            psum[best.1][ch] += sum[b][ch];
        }
        pwt[best.1] += cnt[b];
    }
    // drop empty clusters, sort dark to light
    let mut pal: Vec<(usize, [u8; 3])> = (0..centres.len())
        .filter(|&j| pwt[j] > 0.0)
        .map(|j| (j, [(psum[j][0] / pwt[j]).round() as u8, (psum[j][1] / pwt[j]).round() as u8, (psum[j][2] / pwt[j]).round() as u8]))
        .collect();
    pal.sort_by(|a, b| lum(a.1).total_cmp(&lum(b.1)).then(a.1.cmp(&b.1)));
    let mut remap = vec![0u16; centres.len()];
    for (new, (old, _)) in pal.iter().enumerate() {
        remap[*old] = new as u16;
    }
    for i in 0..n {
        if opaque(i) {
            labels[i] = remap[bin_label[bin(col(i))]];
        }
    }
    (pal.iter().map(|p| p.1).collect(), labels)
}
