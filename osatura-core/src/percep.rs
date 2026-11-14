// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Perceptual image comparison for animation CI (I-06): textbook metrics only.
//!
//! * colour difference: CIE76 and CIEDE2000 (CIE Technical Report 142-2001; the implementation
//!   follows the formulae as printed in Sharma, Wu, Dalal 2005, validated against their test
//!   pairs in the unit tests), sRGB -> linear -> XYZ (D65) -> CIELAB;
//! * SSIM (Wang, Bovik, Sheikh, Simoncelli 2004): 11x11 Gaussian window, sigma 1.5, K1 0.01,
//!   K2 0.03, L 255, on luma; window weights are renormalised at the image border;
//! * MSE / PSNR on the 8-bit channels;
//! * a pHash-style 64-bit DCT hash (32x32 area-average, 2-D DCT-II, 8x8 low frequencies, median
//!   threshold) and its Hamming distance;
//! * anti-aliasing tolerance, connected-region summary, heat map, side-by-side, overlay and flicker.
//!
//! Images are straight-alpha RGBA8. Colour metrics evaluate each pixel composited over white
//! and over black and keep the larger value, so a change of alpha alone is visible.

use crate::vcs::sha256_hex;
use std::sync::OnceLock;

// ---- image -------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Img {
    pub w: usize,
    pub h: usize,
    /// Straight RGBA, row-major, `w * h * 4` bytes.
    pub rgba: Vec<u8>,
}

impl Img {
    pub fn new(w: usize, h: usize, rgba: Vec<u8>) -> Result<Img, String> {
        if w == 0 || h == 0 {
            return Err("image has no pixels".into());
        }
        if rgba.len() != w * h * 4 {
            return Err(format!("pixel buffer has {} bytes, expected {}", rgba.len(), w * h * 4));
        }
        Ok(Img { w, h, rgba })
    }

    pub fn solid(w: usize, h: usize, c: [u8; 4]) -> Img {
        let mut rgba = Vec::with_capacity(w * h * 4);
        for _ in 0..w * h {
            rgba.extend_from_slice(&c);
        }
        Img { w, h, rgba }
    }

    pub fn px(&self, x: usize, y: usize) -> [u8; 4] {
        let i = (y * self.w + x) * 4;
        [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]]
    }

    pub fn set(&mut self, x: usize, y: usize, c: [u8; 4]) {
        let i = (y * self.w + x) * 4;
        self.rgba[i..i + 4].copy_from_slice(&c);
    }

    /// Decode a PNG (any colour type and depth) to RGBA8. Size capped at 64 Mpixel.
    pub fn from_png(bytes: &[u8]) -> Result<Img, String> {
        crate::assets::check_png(bytes).map_err(|e| format!("not a readable PNG: {e}"))?;
        let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
        dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut rd = dec.read_info().map_err(|e| format!("not a readable PNG: {e}"))?;
        let (w, h) = (rd.info().width as usize, rd.info().height as usize);
        if w == 0 || h == 0 || w * h > 64_000_000 {
            return Err(format!("PNG size {w}x{h} is outside 1..64 Mpixel"));
        }
        let mut buf = vec![0u8; rd.output_buffer_size().ok_or("PNG too large")?];
        let info = rd.next_frame(&mut buf).map_err(|e| format!("cannot decode PNG: {e}"))?;
        let n = match info.color_type {
            png::ColorType::Rgba => 4,
            png::ColorType::Rgb => 3,
            png::ColorType::GrayscaleAlpha => 2,
            png::ColorType::Grayscale => 1,
            png::ColorType::Indexed => return Err("unexpected indexed PNG output".into()),
        };
        let mut rgba = Vec::with_capacity(w * h * 4);
        for p in buf[..w * h * n].chunks_exact(n) {
            match n {
                4 => rgba.extend_from_slice(p),
                3 => rgba.extend_from_slice(&[p[0], p[1], p[2], 255]),
                2 => rgba.extend_from_slice(&[p[0], p[0], p[0], p[1]]),
                _ => rgba.extend_from_slice(&[p[0], p[0], p[0], 255]),
            }
        }
        Ok(Img { w, h, rgba })
    }

    pub fn to_png(&self) -> Result<Vec<u8>, String> {
        crate::open_formats::encode_rgba_png(self.w as u32, self.h as u32, &self.rgba)
    }

    /// SHA-256 over the dimensions and the pixels (hex).
    pub fn content_hash(&self) -> String {
        let mut b = Vec::with_capacity(self.rgba.len() + 16);
        b.extend_from_slice(&(self.w as u64).to_le_bytes());
        b.extend_from_slice(&(self.h as u64).to_le_bytes());
        b.extend_from_slice(&self.rgba);
        sha256_hex(&b)
    }

    /// Copy of `rect = [x, y, w, h]` (pixels, clipped) as a new image; `None` when empty.
    pub fn crop(&self, rect: [usize; 4]) -> Option<Img> {
        let (x0, y0) = (rect[0].min(self.w), rect[1].min(self.h));
        let (x1, y1) = ((rect[0] + rect[2]).min(self.w), (rect[1] + rect[3]).min(self.h));
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        let mut out = Vec::with_capacity((x1 - x0) * (y1 - y0) * 4);
        for y in y0..y1 {
            out.extend_from_slice(&self.rgba[(y * self.w + x0) * 4..(y * self.w + x1) * 4]);
        }
        Some(Img { w: x1 - x0, h: y1 - y0, rgba: out })
    }
}

// ---- colour ------------------------------------------------------------------------------------

fn lin_lut() -> &'static [f64; 256] {
    static L: OnceLock<[f64; 256]> = OnceLock::new();
    L.get_or_init(|| {
        let mut t = [0.0; 256];
        for (i, v) in t.iter_mut().enumerate() {
            let c = i as f64 / 255.0;
            *v = if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
        }
        t
    })
}

/// sRGB (0..255) to CIELAB, D65 white.
pub fn srgb_to_lab(r: u8, g: u8, b: u8) -> [f64; 3] {
    let t = lin_lut();
    let (r, g, b) = (t[r as usize], t[g as usize], t[b as usize]);
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;
    let f = |t: f64| if t > 216.0 / 24389.0 { t.cbrt() } else { (24389.0 / 27.0 * t + 16.0) / 116.0 };
    let (fx, fy, fz) = (f(x / 0.95047), f(y), f(z / 1.08883));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

pub fn de76(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// CIEDE2000 colour difference (kL = kC = kH = 1).
pub fn de2000(l1: [f64; 3], l2: [f64; 3]) -> f64 {
    let (lum1, a1, b1) = (l1[0], l1[1], l1[2]);
    let (lum2, a2, b2) = (l2[0], l2[1], l2[2]);
    let pow7 = |x: f64| x.powi(7);
    let c1 = a1.hypot(b1);
    let c2 = a2.hypot(b2);
    let cbar = (c1 + c2) / 2.0;
    let g = 0.5 * (1.0 - (pow7(cbar) / (pow7(cbar) + pow7(25.0))).sqrt());
    let a1p = (1.0 + g) * a1;
    let a2p = (1.0 + g) * a2;
    let c1p = a1p.hypot(b1);
    let c2p = a2p.hypot(b2);
    let hue = |b: f64, ap: f64| {
        if b == 0.0 && ap == 0.0 {
            0.0
        } else {
            let h = b.atan2(ap).to_degrees();
            if h < 0.0 {
                h + 360.0
            } else {
                h
            }
        }
    };
    let h1p = hue(b1, a1p);
    let h2p = hue(b2, a2p);
    let dlp = lum2 - lum1;
    let dcp = c2p - c1p;
    let dhp_deg = if c1p * c2p == 0.0 {
        0.0
    } else {
        let d = h2p - h1p;
        if d > 180.0 {
            d - 360.0
        } else if d < -180.0 {
            d + 360.0
        } else {
            d
        }
    };
    let dhp = 2.0 * (c1p * c2p).sqrt() * (dhp_deg.to_radians() / 2.0).sin();
    let lbp = (lum1 + lum2) / 2.0;
    let cbp = (c1p + c2p) / 2.0;
    let hbp = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        (h1p + h2p) / 2.0
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) / 2.0
    } else {
        (h1p + h2p - 360.0) / 2.0
    };
    let t = 1.0 - 0.17 * (hbp - 30.0).to_radians().cos() + 0.24 * (2.0 * hbp).to_radians().cos() + 0.32 * (3.0 * hbp + 6.0).to_radians().cos()
        - 0.20 * (4.0 * hbp - 63.0).to_radians().cos();
    let dtheta = 30.0 * (-((hbp - 275.0) / 25.0).powi(2)).exp();
    let rc = 2.0 * (pow7(cbp) / (pow7(cbp) + pow7(25.0))).sqrt();
    let sl = 1.0 + 0.015 * (lbp - 50.0).powi(2) / (20.0 + (lbp - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * cbp;
    let sh = 1.0 + 0.015 * cbp * t;
    let rt = -(2.0 * dtheta).to_radians().sin() * rc;
    let (x, y, z) = (dlp / sl, dcp / sc, dhp / sh);
    (x * x + y * y + z * z + rt * y * z).max(0.0).sqrt()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeMetric {
    De76,
    De2000,
}

impl DeMetric {
    pub fn parse(s: &str) -> Result<DeMetric, String> {
        match s {
            "de76" | "cie76" => Ok(DeMetric::De76),
            "de2000" | "ciede2000" => Ok(DeMetric::De2000),
            _ => Err(format!("unknown colour metric '{s}' (de76 or de2000)")),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            DeMetric::De76 => "de76",
            DeMetric::De2000 => "de2000",
        }
    }
    fn eval(self, a: [f64; 3], b: [f64; 3]) -> f64 {
        match self {
            DeMetric::De76 => de76(a, b),
            DeMetric::De2000 => de2000(a, b),
        }
    }
}

fn over(c: [u8; 4], bg: u8) -> (u8, u8, u8) {
    let a = c[3] as u32;
    let m = |v: u8| ((v as u32 * a + bg as u32 * (255 - a) + 127) / 255) as u8;
    (m(c[0]), m(c[1]), m(c[2]))
}

/// Colour difference of two straight RGBA pixels, worst of "over white" and "over black".
pub fn pixel_de(a: [u8; 4], b: [u8; 4], metric: DeMetric) -> f64 {
    if a == b {
        return 0.0;
    }
    let mut worst = 0.0f64;
    for bg in [255u8, 0u8] {
        let (r1, g1, b1) = over(a, bg);
        let (r2, g2, b2) = over(b, bg);
        let d = metric.eval(srgb_to_lab(r1, g1, b1), srgb_to_lab(r2, g2, b2));
        worst = worst.max(d);
    }
    worst
}

// ---- MSE / PSNR --------------------------------------------------------------------------------

/// True when any pixel of either image is not fully opaque.
fn any_translucent(a: &Img, b: &Img) -> bool {
    a.rgba.chunks_exact(4).any(|p| p[3] != 255) || b.rgba.chunks_exact(4).any(|p| p[3] != 255)
}

/// Mean squared error, the standard definition: over the R, G and B samples. Alpha is included
/// (as a fourth channel) only when some pixel of either image is not fully opaque, so an opaque
/// image does not get its error diluted by a constant 255 alpha channel.
pub fn mse(a: &Img, b: &Img) -> Result<f64, String> {
    same_size(a, b)?;
    let with_alpha = any_translucent(a, b);
    let (mut s, mut n) = (0.0f64, 0usize);
    for (pa, pb) in a.rgba.chunks_exact(4).zip(b.rgba.chunks_exact(4)) {
        for k in 0..if with_alpha { 4 } else { 3 } {
            s += ((pa[k] as f64) - (pb[k] as f64)).powi(2);
            n += 1;
        }
    }
    Ok(if n == 0 { 0.0 } else { s / n as f64 })
}

/// PSNR in dB, capped at 100 (identical images).
pub fn psnr_from_mse(mse: f64) -> f64 {
    if mse <= 0.0 {
        100.0
    } else {
        (10.0 * (255.0f64 * 255.0 / mse).log10()).min(100.0)
    }
}

fn same_size(a: &Img, b: &Img) -> Result<(), String> {
    if a.w != b.w || a.h != b.h {
        Err(format!("image sizes differ: {}x{} vs {}x{}", a.w, a.h, b.w, b.h))
    } else {
        Ok(())
    }
}

// ---- SSIM --------------------------------------------------------------------------------------

const SSIM_R: usize = 5; // 11 taps
const SSIM_SIGMA: f64 = 1.5;

fn gauss_taps() -> [f64; 2 * SSIM_R + 1] {
    let mut k = [0.0; 2 * SSIM_R + 1];
    let mut s = 0.0;
    for (i, v) in k.iter_mut().enumerate() {
        let d = i as f64 - SSIM_R as f64;
        *v = (-d * d / (2.0 * SSIM_SIGMA * SSIM_SIGMA)).exp();
        s += *v;
    }
    k.iter_mut().for_each(|v| *v /= s);
    k
}

/// Separable Gaussian blur with window weights renormalised where the window leaves the image.
fn blur(src: &[f64], w: usize, h: usize) -> Vec<f64> {
    let k = gauss_taps();
    let r = SSIM_R as isize;
    let mut tmp = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            let (mut s, mut ws) = (0.0, 0.0);
            for t in -r..=r {
                let xx = x as isize + t;
                if xx >= 0 && (xx as usize) < w {
                    let kk = k[(t + r) as usize];
                    s += kk * src[y * w + xx as usize];
                    ws += kk;
                }
            }
            tmp[y * w + x] = s / ws;
        }
    }
    let mut out = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            let (mut s, mut ws) = (0.0, 0.0);
            for t in -r..=r {
                let yy = y as isize + t;
                if yy >= 0 && (yy as usize) < h {
                    let kk = k[(t + r) as usize];
                    s += kk * tmp[yy as usize * w + x];
                    ws += kk;
                }
            }
            out[y * w + x] = s / ws;
        }
    }
    out
}

fn luma_over(img: &Img, bg: u8) -> Vec<f64> {
    img.rgba
        .chunks_exact(4)
        .map(|p| {
            let (r, g, b) = over([p[0], p[1], p[2], p[3]], bg);
            0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64
        })
        .collect()
}

fn ssim_map_luma(x: &[f64], y: &[f64], w: usize, h: usize) -> Vec<f64> {
    let (c1, c2) = ((0.01f64 * 255.0).powi(2), (0.03f64 * 255.0).powi(2));
    let mx = blur(x, w, h);
    let my = blur(y, w, h);
    let xx: Vec<f64> = x.iter().map(|v| v * v).collect();
    let yy: Vec<f64> = y.iter().map(|v| v * v).collect();
    let xy: Vec<f64> = x.iter().zip(y).map(|(a, b)| a * b).collect();
    let exx = blur(&xx, w, h);
    let eyy = blur(&yy, w, h);
    let exy = blur(&xy, w, h);
    (0..w * h)
        .map(|i| {
            let vx = exx[i] - mx[i] * mx[i];
            let vy = eyy[i] - my[i] * my[i];
            let cxy = exy[i] - mx[i] * my[i];
            ((2.0 * mx[i] * my[i] + c1) * (2.0 * cxy + c2)) / ((mx[i] * mx[i] + my[i] * my[i] + c1) * (vx + vy + c2))
        })
        .collect()
}

/// Per-pixel SSIM map (worst of "over white" and "over black") and its mean (MSSIM).
pub fn ssim_map(a: &Img, b: &Img) -> Result<(Vec<f32>, f64), String> {
    same_size(a, b)?;
    let (w, h) = (a.w, a.h);
    let mut best: Vec<f64> = vec![1.0; w * h];
    let has_alpha = a.rgba.chunks_exact(4).any(|p| p[3] != 255) || b.rgba.chunks_exact(4).any(|p| p[3] != 255);
    let bgs: &[u8] = if has_alpha { &[255, 0] } else { &[255] };
    for &bg in bgs {
        let m = ssim_map_luma(&luma_over(a, bg), &luma_over(b, bg), w, h);
        for (o, v) in best.iter_mut().zip(m) {
            *o = o.min(v);
        }
    }
    let mean = best.iter().sum::<f64>() / best.len() as f64;
    Ok((best.iter().map(|&v| v as f32).collect(), mean))
}

pub fn ssim(a: &Img, b: &Img) -> Result<f64, String> {
    Ok(ssim_map(a, b)?.1)
}

fn channel_over(img: &Img, bg: u8, k: usize) -> Vec<f64> {
    img.rgba
        .chunks_exact(4)
        .map(|p| {
            let (r, g, b) = over([p[0], p[1], p[2], p[3]], bg);
            [r, g, b][k] as f64
        })
        .collect()
}

/// Mean SSIM of each of the R, G and B channels (worst of over-white / over-black when translucent).
/// The luma SSIM of [`ssim`] cannot see a change of hue at equal luminance (red vs the green of the
/// same luma scores 1.0); the smallest channel value can.
pub fn ssim_channels(a: &Img, b: &Img) -> Result<[f64; 3], String> {
    same_size(a, b)?;
    let (w, h) = (a.w, a.h);
    let bgs: &[u8] = if any_translucent(a, b) { &[255, 0] } else { &[255] };
    let mut out = [1.0f64; 3];
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for k in 0..3 {
        for &bg in bgs {
            let m = ssim_map_luma(&channel_over(a, bg, k), &channel_over(b, bg, k), w, h);
            out[k] = out[k].min(m.iter().sum::<f64>() / m.len().max(1) as f64);
        }
    }
    Ok(out)
}

// ---- pHash -------------------------------------------------------------------------------------

/// 64-bit perceptual hash: 32x32 area average of luma (over white), 2-D DCT-II, top-left 8x8
/// coefficients, bit = coefficient above the median of the 63 AC terms.
pub fn phash(img: &Img) -> u64 {
    const N: usize = 32;
    let luma = luma_over(img, 255);
    let mut g = [[0.0f64; N]; N];
    for (j, row) in g.iter_mut().enumerate() {
        let (y0, y1) = (j * img.h / N, ((j + 1) * img.h).div_ceil(N).min(img.h));
        for (i, cell) in row.iter_mut().enumerate() {
            let (x0, x1) = (i * img.w / N, ((i + 1) * img.w).div_ceil(N).min(img.w));
            let (mut s, mut n) = (0.0f64, 0.0f64);
            for y in y0..y1.max(y0 + 1).min(img.h) {
                for x in x0..x1.max(x0 + 1).min(img.w) {
                    s += luma[y * img.w + x];
                    n += 1.0;
                }
            }
            *cell = s / n.max(1.0);
        }
    }
    let mut cosv = [[0.0f64; N]; 8];
    for (u, row) in cosv.iter_mut().enumerate() {
        for (x, c) in row.iter_mut().enumerate() {
            *c = ((2 * x + 1) as f64 * u as f64 * std::f64::consts::PI / (2.0 * N as f64)).cos();
        }
    }
    let mut coef = [[0.0f64; 8]; 8];
    for v in 0..8 {
        for u in 0..8 {
            let mut s = 0.0;
            for y in 0..N {
                for x in 0..N {
                    s += g[y][x] * cosv[u][x] * cosv[v][y];
                }
            }
            coef[v][u] = s;
        }
    }
    let mut ac: Vec<f64> = (0..64).filter(|&i| i != 0).map(|i| coef[i / 8][i % 8]).collect();
    ac.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = ac[ac.len() / 2];
    let mut bits = 0u64;
    for i in 0..64 {
        if i != 0 && coef[i / 8][i % 8] > median {
            bits |= 1 << i;
        }
    }
    bits
}

pub fn hamming(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

// ---- diff engine -------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct CompareOptions {
    pub metric: DeMetric,
    /// A pixel counts as different when its colour difference exceeds this.
    pub pixel_de: f64,
    /// Differing pixels forming 8-connected groups of at most this many pixels, all lying on an
    /// edge (in either image), are forgiven as anti-aliasing noise. 0 = off.
    pub aa_isolated: usize,
    /// Edge pixels whose difference does not exceed this are forgiven. 0 = off.
    pub aa_edge_de: f64,
    /// Rectangles `[x, y, w, h]` in pixels of the compared images that are not compared.
    pub ignore: Vec<[i64; 4]>,
    /// Counted pixels closer than this (Chebyshev, pixels) join one region.
    pub merge_dist: usize,
    pub max_regions: usize,
}

impl Default for CompareOptions {
    fn default() -> Self {
        CompareOptions { metric: DeMetric::De2000, pixel_de: 1.0, aa_isolated: 2, aa_edge_de: 0.0, ignore: vec![], merge_dist: 2, max_regions: 50 }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Region {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub pixels: usize,
    pub max_de: f64,
}

pub const PX_SAME: u8 = 0;
pub const PX_DIFF: u8 = 1;
pub const PX_FORGIVEN: u8 = 2;
pub const PX_IGNORED: u8 = 3;

#[derive(Clone, Debug)]
pub struct Diff {
    pub w: usize,
    pub h: usize,
    pub ssim: f64,
    /// Smallest mean SSIM over the R, G and B channels: the chroma-aware companion of the luma `ssim`.
    pub ssim_rgb: f64,
    pub mse: f64,
    pub psnr: f64,
    /// Largest difference among counted pixels (after anti-aliasing tolerance).
    pub max_de: f64,
    /// Largest difference among all differing pixels, forgiven or not.
    pub max_de_raw: f64,
    pub mean_de: f64,
    pub diff_pixels: usize,
    pub forgiven_pixels: usize,
    pub ignored_pixels: usize,
    /// Counted pixels over compared (non-ignored) pixels.
    pub diff_fraction: f64,
    pub phash_distance: u32,
    pub regions: Vec<Region>,
    pub regions_total: usize,
    pub state: Vec<u8>,
    pub de: Vec<f32>,
    pub ssim_map: Vec<f32>,
}

/// A pixel lies on an edge of `img` when two opposite neighbours (clamped at the border) differ by
/// more than 40 in some channel. The pixel's own value is not used, so a stray pixel in a flat
/// area is not an edge but the anti-aliased rim of a shape is.
fn is_edge(img: &Img, x: usize, y: usize) -> bool {
    let at = |dx: isize, dy: isize| img.px((x as isize + dx).clamp(0, img.w as isize - 1) as usize, (y as isize + dy).clamp(0, img.h as isize - 1) as usize);
    [((-1, 0), (1, 0)), ((0, -1), (0, 1)), ((-1, -1), (1, 1)), ((1, -1), (-1, 1))].iter().any(|&((ax, ay), (bx, by))| {
        let (p, q) = (at(ax, ay), at(bx, by));
        (0..4).any(|k| (p[k] as i32 - q[k] as i32).abs() > 40)
    })
}

pub fn compare(a: &Img, b: &Img, o: &CompareOptions) -> Result<Diff, String> {
    same_size(a, b)?;
    if !(o.pixel_de.is_finite() && o.pixel_de >= 0.0) {
        return Err("pixel_de must be a number >= 0".into());
    }
    let (w, h) = (a.w, a.h);
    let mut state = vec![PX_SAME; w * h];
    let mut ignored = 0;
    for r in &o.ignore {
        let (x0, y0) = (r[0].max(0) as usize, r[1].max(0) as usize);
        let (x1, y1) = (((r[0] + r[2]).max(0) as usize).min(w), ((r[1] + r[3]).max(0) as usize).min(h));
        for y in y0..y1 {
            for x in x0..x1 {
                if state[y * w + x] != PX_IGNORED {
                    state[y * w + x] = PX_IGNORED;
                    ignored += 1;
                }
            }
        }
    }
    // masked pixels are made equal so every metric ignores them
    let mut b2 = b.clone();
    if ignored > 0 {
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for i in 0..w * h {
            if state[i] == PX_IGNORED {
                b2.rgba[i * 4..i * 4 + 4].copy_from_slice(&a.rgba[i * 4..i * 4 + 4]);
            }
        }
    }
    let b = &b2;
    let mut de = vec![0.0f32; w * h];
    let mut raw_max = 0.0f64;
    let mut sum_de = 0.0f64;
    let mut cand = Vec::new();
    for i in 0..w * h {
        if state[i] == PX_IGNORED {
            continue;
        }
        let (pa, pb) = ([a.rgba[i * 4], a.rgba[i * 4 + 1], a.rgba[i * 4 + 2], a.rgba[i * 4 + 3]], [b.rgba[i * 4], b.rgba[i * 4 + 1], b.rgba[i * 4 + 2], b.rgba[i * 4 + 3]]);
        if pa == pb {
            continue;
        }
        let d = pixel_de(pa, pb, o.metric);
        de[i] = d as f32;
        sum_de += d;
        raw_max = raw_max.max(d);
        if d > o.pixel_de {
            cand.push(i);
            state[i] = PX_DIFF;
        }
    }
    // anti-aliasing tolerance
    let edge: std::collections::HashMap<usize, bool> = if o.aa_isolated > 0 || o.aa_edge_de > 0.0 {
        cand.iter().map(|&i| (i, is_edge(a, i % w, i / w) || is_edge(b, i % w, i / w))).collect()
    } else {
        Default::default()
    };
    if o.aa_edge_de > 0.0 {
        for &i in &cand {
            if edge[&i] && (de[i] as f64) <= o.aa_edge_de {
                state[i] = PX_FORGIVEN;
            }
        }
    }
    if o.aa_isolated > 0 {
        let mut seen = vec![false; w * h];
        for &s in &cand {
            if state[s] != PX_DIFF || seen[s] {
                continue;
            }
            let mut comp = vec![s];
            seen[s] = true;
            let mut qi = 0;
            let mut all_edge = true;
            while qi < comp.len() {
                let i = comp[qi];
                qi += 1;
                all_edge &= edge[&i];
                if comp.len() > o.aa_isolated {
                    all_edge = false;
                }
                let (x, y) = ((i % w) as isize, (i / w) as isize);
                for dy in -1..=1isize {
                    for dx in -1..=1isize {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= h {
                            continue;
                        }
                        let j = ny as usize * w + nx as usize;
                        if state[j] == PX_DIFF && !seen[j] {
                            seen[j] = true;
                            comp.push(j);
                        }
                    }
                }
            }
            if comp.len() <= o.aa_isolated && all_edge {
                for i in comp {
                    state[i] = PX_FORGIVEN;
                }
            }
        }
    }
    let counted: Vec<usize> = cand.iter().copied().filter(|&i| state[i] == PX_DIFF).collect();
    let forgiven = cand.len() - counted.len();
    let max_de = counted.iter().map(|&i| de[i] as f64).fold(0.0, f64::max);
    let compared = (w * h - ignored).max(1);
    let (regions, regions_total) = regions_of(&counted, &de, w, o.merge_dist, o.max_regions);
    let (smap, smean) = if a.rgba == b.rgba { (vec![1.0f32; w * h], 1.0) } else { ssim_map(a, b)? };
    let ssim_rgb = if a.rgba == b.rgba { 1.0 } else { ssim_channels(a, b)?.iter().cloned().fold(1.0, f64::min) };
    let m = mse(a, b)?;
    Ok(Diff {
        w,
        h,
        ssim: smean,
        ssim_rgb,
        mse: m,
        psnr: psnr_from_mse(m),
        max_de,
        max_de_raw: raw_max,
        mean_de: sum_de / compared as f64,
        diff_pixels: counted.len(),
        forgiven_pixels: forgiven,
        ignored_pixels: ignored,
        diff_fraction: counted.len() as f64 / compared as f64,
        phash_distance: hamming(phash(a), phash(b)),
        regions,
        regions_total,
        state,
        de,
        ssim_map: smap,
    })
}

fn find(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}

/// Connected groups of counted pixels (joined within `merge` pixels), biggest first.
fn regions_of(counted: &[usize], de: &[f32], w: usize, merge: usize, cap: usize) -> (Vec<Region>, usize) {
    if counted.is_empty() {
        return (vec![], 0);
    }
    let idx: std::collections::HashMap<usize, usize> = counted.iter().enumerate().map(|(k, &i)| (i, k)).collect();
    let mut parent: Vec<usize> = (0..counted.len()).collect();
    let d = merge.max(1) as isize;
    for (k, &i) in counted.iter().enumerate() {
        let (x, y) = ((i % w) as isize, (i / w) as isize);
        for dy in 0..=d {
            for dx in -d..=d {
                if dy == 0 && dx <= 0 {
                    continue;
                }
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx as usize >= w {
                    continue;
                }
                if let Some(&k2) = idx.get(&(ny as usize * w + nx as usize)) {
                    let (ra, rb) = (find(&mut parent, k), find(&mut parent, k2));
                    if ra != rb {
                        parent[ra] = rb;
                    }
                }
            }
        }
    }
    let mut map: std::collections::BTreeMap<usize, Region> = Default::default();
    for (k, &i) in counted.iter().enumerate() {
        let root = find(&mut parent, k);
        let (x, y) = (i % w, i / w);
        let e = map.entry(root).or_insert(Region { x, y, w: 1, h: 1, pixels: 0, max_de: 0.0 });
        let (x1, y1) = ((e.x + e.w).max(x + 1), (e.y + e.h).max(y + 1));
        e.x = e.x.min(x);
        e.y = e.y.min(y);
        e.w = x1 - e.x;
        e.h = y1 - e.y;
        e.pixels += 1;
        e.max_de = e.max_de.max(de[i] as f64);
    }
    let mut v: Vec<Region> = map.into_values().collect();
    v.sort_by(|a, b| b.pixels.cmp(&a.pixels).then(a.y.cmp(&b.y)).then(a.x.cmp(&b.x)));
    let total = v.len();
    v.truncate(cap);
    (v, total)
}

// ---- visual outputs ----------------------------------------------------------------------------

/// Colour ramp for the heat map: pale yellow -> orange -> red -> dark red as `de` grows to 30.
fn heat_color(de: f32) -> [u8; 4] {
    let t = (de / 30.0).clamp(0.0, 1.0);
    let (r, g, b) = if t < 0.5 {
        let u = t / 0.5;
        (255.0, 230.0 - 130.0 * u, 60.0 - 40.0 * u)
    } else {
        let u = (t - 0.5) / 0.5;
        (255.0 - 115.0 * u, 100.0 - 100.0 * u, 20.0 - 20.0 * u)
    };
    [r as u8, g as u8, b as u8, 255]
}

/// Heat map of a comparison: the first image faded to 30 %, counted differences in the ramp,
/// forgiven anti-aliasing in light blue, ignored areas hatched grey, region boxes in magenta.
pub fn heatmap(a: &Img, d: &Diff) -> Img {
    let mut out = Img::solid(a.w, a.h, [255, 255, 255, 255]);
    for y in 0..a.h {
        for x in 0..a.w {
            let i = y * a.w + x;
            let p = a.px(x, y);
            let (r, g, b) = over(p, 255);
            let f = |v: u8| (255.0 - (255.0 - v as f32) * 0.30) as u8;
            let mut c = [f(r), f(g), f(b), 255];
            match d.state[i] {
                PX_DIFF => c = heat_color(d.de[i]),
                PX_FORGIVEN => c = [150, 200, 255, 255],
                PX_IGNORED
                    if (x + y) % 4 < 2 => {
                        c = [170, 170, 170, 255];
                    }
                _ => {}
            }
            out.set(x, y, c);
        }
    }
    for r in &d.regions {
        let (x1, y1) = (r.x + r.w - 1, r.y + r.h - 1);
        let (bx0, by0) = (r.x.saturating_sub(1), r.y.saturating_sub(1));
        let (bx1, by1) = ((x1 + 1).min(a.w - 1), (y1 + 1).min(a.h - 1));
        for x in bx0..=bx1 {
            out.set(x, by0, [220, 0, 220, 255]);
            out.set(x, by1, [220, 0, 220, 255]);
        }
        for y in by0..=by1 {
            out.set(bx0, y, [220, 0, 220, 255]);
            out.set(bx1, y, [220, 0, 220, 255]);
        }
    }
    out
}

/// Images laid out left to right with a 4 px gap.
pub fn side_by_side(imgs: &[&Img]) -> Img {
    let h = imgs.iter().map(|i| i.h).max().unwrap_or(1);
    let w: usize = imgs.iter().map(|i| i.w).sum::<usize>() + 4 * imgs.len().saturating_sub(1);
    let mut out = Img::solid(w.max(1), h.max(1), [128, 128, 128, 255]);
    let mut x0 = 0;
    for im in imgs {
        for y in 0..im.h {
            for x in 0..im.w {
                out.set(x0 + x, y, im.px(x, y));
            }
        }
        x0 += im.w + 4;
    }
    out
}

/// Red/cyan overlay: red channel from the first image's luma, green and blue from the second's.
/// Grey where they agree, cyan where only the first has dark content, red where only the second does.
pub fn overlay(a: &Img, b: &Img) -> Result<Img, String> {
    same_size(a, b)?;
    let (la, lb) = (luma_over(a, 255), luma_over(b, 255));
    let mut out = Img::solid(a.w, a.h, [255; 4]);
    for i in 0..a.w * a.h {
        out.rgba[i * 4] = la[i] as u8;
        out.rgba[i * 4 + 1] = lb[i] as u8;
        out.rgba[i * 4 + 2] = lb[i] as u8;
    }
    Ok(out)
}

/// Two-frame looping GIF alternating `a` and `b` (a flicker test).
pub fn flicker_gif(a: &Img, b: &Img) -> Result<Vec<u8>, String> {
    same_size(a, b)?;
    crate::export::gif_from_frames(a.w as u32, a.h as u32, vec![a.rgba.clone(), b.rgba.clone()], 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lab(l: f64, a: f64, b: f64) -> [f64; 3] {
        [l, a, b]
    }

    /// Published test pairs from Sharma, Wu, Dalal (2005), "The CIEDE2000 colour-difference
    /// formula: implementation notes, supplementary test data, and mathematical observations".
    #[test]
    fn ciede2000_matches_published_test_data() {
        let table: &[([f64; 3], [f64; 3], f64)] = &[
            (lab(50.0, 2.6772, -79.7751), lab(50.0, 0.0, -82.7485), 2.0425),
            (lab(50.0, 3.1571, -77.2803), lab(50.0, 0.0, -82.7485), 2.8615),
            (lab(50.0, 2.8361, -74.0200), lab(50.0, 0.0, -82.7485), 3.4412),
            (lab(50.0, -1.3802, -84.2814), lab(50.0, 0.0, -82.7485), 1.0000),
            (lab(50.0, 0.0, 0.0), lab(50.0, -1.0, 2.0), 2.3669),
            (lab(50.0, -1.0, 2.0), lab(50.0, 0.0, 0.0), 2.3669),
            (lab(50.0, 2.49, -0.001), lab(50.0, -2.49, 0.0009), 7.1792),
            (lab(50.0, 2.49, -0.001), lab(50.0, -2.49, 0.0011), 7.2195),
            (lab(50.0, -0.001, 2.49), lab(50.0, 0.0011, -2.49), 4.7461),
            (lab(50.0, 2.5, 0.0), lab(50.0, 0.0, -2.5), 4.3065),
            (lab(50.0, 2.5, 0.0), lab(73.0, 25.0, -18.0), 27.1492),
            (lab(50.0, 2.5, 0.0), lab(61.0, -5.0, 29.0), 22.8977),
            (lab(50.0, 2.5, 0.0), lab(56.0, -27.0, -3.0), 31.9030),
            (lab(50.0, 2.5, 0.0), lab(58.0, 24.0, 15.0), 19.4535),
            (lab(60.2574, -34.0099, 36.2677), lab(60.4626, -34.1751, 39.4387), 1.2644),
            (lab(63.0109, -31.0961, -5.8663), lab(62.8187, -29.7946, -4.0864), 1.2630),
            (lab(61.2901, 3.7196, -5.3901), lab(61.4292, 2.2480, -4.9620), 1.8731),
            (lab(35.0831, -44.1164, 3.7933), lab(35.0232, -40.0716, 1.5901), 1.8645),
            (lab(22.7233, 20.0904, -46.6940), lab(23.0331, 14.9730, -42.5619), 2.0373),
            (lab(36.4612, 47.8580, 18.3852), lab(36.2715, 50.5065, 21.2231), 1.4146),
            (lab(90.8027, -2.0831, 1.4410), lab(91.1528, -1.6435, 0.0447), 1.4441),
            (lab(90.9257, -0.5406, -0.9208), lab(88.6381, -0.8985, -0.7239), 1.5381),
            (lab(6.7747, -0.2908, -2.4247), lab(5.8714, -0.0985, -2.2286), 0.6377),
            (lab(2.0776, 0.0795, -1.1350), lab(0.9033, -0.0636, -0.5514), 0.9082),
        ];
        for (i, (a, b, want)) in table.iter().enumerate() {
            let got = de2000(*a, *b);
            assert!((got - want).abs() < 6e-5, "pair {i}: got {got}, published {want}");
            let back = de2000(*b, *a);
            assert!((got - back).abs() < 1e-9, "symmetric");
        }
    }

    #[test]
    fn lab_of_known_srgb_colours() {
        let w = srgb_to_lab(255, 255, 255);
        assert!((w[0] - 100.0).abs() < 0.01 && w[1].abs() < 0.01 && w[2].abs() < 0.01, "{w:?}");
        let k = srgb_to_lab(0, 0, 0);
        assert!(k[0].abs() < 1e-9);
        // sRGB red is L 53.24, a 80.09, b 67.20 (D65)
        let r = srgb_to_lab(255, 0, 0);
        assert!((r[0] - 53.24).abs() < 0.05 && (r[1] - 80.09).abs() < 0.1 && (r[2] - 67.20).abs() < 0.1, "{r:?}");
        assert!((de76(w, k) - 100.0).abs() < 0.02);
    }

    #[test]
    fn identical_images_have_perfect_scores() {
        let a = Img::solid(20, 16, [10, 120, 200, 255]);
        let d = compare(&a, &a.clone(), &CompareOptions::default()).unwrap();
        assert_eq!(d.ssim, 1.0);
        assert_eq!(d.max_de, 0.0);
        assert_eq!(d.mse, 0.0);
        assert_eq!(d.psnr, 100.0);
        assert_eq!(d.diff_pixels, 0);
        assert!(d.regions.is_empty());
        assert_eq!(d.phash_distance, 0);
    }

    #[test]
    fn ssim_of_constant_images_matches_the_closed_form() {
        let (c1, c2) = ((0.01f64 * 255.0).powi(2), (0.03f64 * 255.0).powi(2));
        let a = Img::solid(30, 30, [100, 100, 100, 255]);
        let b = Img::solid(30, 30, [150, 150, 150, 255]);
        let got = ssim(&a, &b).unwrap();
        // flat windows: variances and covariance are 0, so SSIM = (2 mx my + C1) / (mx^2 + my^2 + C1) * (C2 / C2)
        let want = (2.0 * 100.0 * 150.0 + c1) / (100.0f64.powi(2) + 150.0f64.powi(2) + c1);
        let _ = c2;
        assert!((got - want).abs() < 1e-6, "{got} vs {want}");
    }

    fn noise_img(w: usize, h: usize, seed: u32) -> Img {
        let mut s = seed;
        let mut rgba = Vec::new();
        for _ in 0..w * h {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            let v = (s >> 24) as u8;
            rgba.extend_from_slice(&[v, v, v, 255]);
        }
        Img::new(w, h, rgba).unwrap()
    }

    #[test]
    fn ssim_centre_pixel_matches_brute_force_window_sum() {
        let (w, h) = (24, 24);
        let a = noise_img(w, h, 1);
        let mut b = noise_img(w, h, 1);
        for i in 0..w * h {
            let v = b.rgba[i * 4] as i32 + ((i * 7) % 23) as i32 - 11;
            let v = v.clamp(0, 255) as u8;
            b.rgba[i * 4..i * 4 + 3].copy_from_slice(&[v, v, v]);
        }
        let (map, _) = ssim_map(&a, &b).unwrap();
        let (cx, cy) = (12usize, 12usize);
        let k = gauss_taps();
        let (c1, c2) = ((0.01f64 * 255.0).powi(2), (0.03f64 * 255.0).powi(2));
        let (la, lb) = (luma_over(&a, 255), luma_over(&b, 255));
        let (mut mx, mut my) = (0.0, 0.0);
        for dy in 0..11 {
            for dx in 0..11 {
                let wgt = k[dx] * k[dy];
                let i = (cy + dy - 5) * w + cx + dx - 5;
                mx += wgt * la[i];
                my += wgt * lb[i];
            }
        }
        let (mut vx, mut vy, mut cv) = (0.0, 0.0, 0.0);
        for dy in 0..11 {
            for dx in 0..11 {
                let wgt = k[dx] * k[dy];
                let i = (cy + dy - 5) * w + cx + dx - 5;
                vx += wgt * (la[i] - mx).powi(2);
                vy += wgt * (lb[i] - my).powi(2);
                cv += wgt * (la[i] - mx) * (lb[i] - my);
            }
        }
        let want = ((2.0 * mx * my + c1) * (2.0 * cv + c2)) / ((mx * mx + my * my + c1) * (vx + vy + c2));
        assert!((map[cy * w + cx] as f64 - want).abs() < 1e-5, "{} vs {want}", map[cy * w + cx]);
        assert!(want < 1.0 && want > 0.5);
    }

    #[test]
    fn ssim_drops_for_inverted_structure() {
        let a = noise_img(32, 32, 5);
        let mut b = a.clone();
        for i in 0..32 * 32 {
            for c in 0..3 {
                b.rgba[i * 4 + c] = 255 - b.rgba[i * 4 + c];
            }
        }
        assert!(ssim(&a, &b).unwrap() < 0.1);
        assert!(ssim(&a, &a).unwrap() > 0.999999);
    }

    #[test]
    fn mse_and_psnr_of_a_known_offset() {
        let a = Img::solid(8, 8, [100, 100, 100, 255]);
        let b = Img::solid(8, 8, [110, 100, 100, 255]);
        // one of three colour channels differs by 10 (an opaque image: alpha is not part of the MSE)
        assert!((mse(&a, &b).unwrap() - 100.0 / 3.0).abs() < 1e-9);
        // with a translucent pixel anywhere alpha counts as a fourth channel
        let mut c = a.clone();
        c.set(0, 0, [100, 100, 100, 128]);
        let mut d = b.clone();
        d.set(0, 0, [100, 100, 100, 128]);
        assert!((mse(&c, &d).unwrap() - (100.0 * 63.0) / (64.0 * 4.0)).abs() < 1e-9);
        assert!((psnr_from_mse(25.0) - 10.0 * (65025.0f64 / 25.0).log10()).abs() < 1e-9);
        assert!(mse(&a, &Img::solid(4, 4, [0; 4])).is_err());
    }

    /// C9: red and the green of equal luma have luma-SSIM 1.0 but a huge colour difference; the
    /// per-channel SSIM sees it.
    #[test]
    fn equiluminant_hue_change_is_not_a_match() {
        // luma 0.299*255 = 76.2 for red; green with the same luma: 76.2/0.587 = 129.8
        let red = Img::solid(32, 32, [255, 0, 0, 255]);
        let mut green = Img::solid(32, 32, [0, 130, 0, 255]);
        // make the luma exactly equal by a little blue-free tweak (0.587 * 129.8 ~ 76.2)
        green.set(0, 0, [0, 130, 0, 255]);
        let l = ssim(&red, &green).unwrap();
        let ch = ssim_channels(&red, &green).unwrap();
        let d = compare(&red, &green, &CompareOptions::default()).unwrap();
        assert!(l > 0.9, "luma SSIM is blind to it: {l}");
        assert!(ch.iter().cloned().fold(1.0, f64::min) < 0.5, "channel SSIM {ch:?}");
        assert!(d.ssim_rgb < 0.5 && d.max_de > 40.0);
        let same = compare(&red, &red, &CompareOptions::default()).unwrap();
        assert_eq!(same.ssim_rgb, 1.0);
    }

    #[test]
    fn de_of_white_and_black_and_alpha_changes() {
        let w = [255, 255, 255, 255];
        let k = [0, 0, 0, 255];
        assert!((pixel_de(w, k, DeMetric::De76) - 100.0).abs() < 0.02);
        assert!(pixel_de(w, k, DeMetric::De2000) > 90.0);
        // same colour, different alpha: visible over one of the two backgrounds
        assert!(pixel_de([200, 0, 0, 255], [200, 0, 0, 128], DeMetric::De2000) > 5.0);
        assert_eq!(pixel_de(k, k, DeMetric::De2000), 0.0);
    }

    fn rect_img(w: usize, h: usize, x0: usize, x1: usize) -> Img {
        let mut a = Img::solid(w, h, [255, 255, 255, 255]);
        for y in 8..h - 8 {
            for x in x0..x1 {
                a.set(x, y, [0, 0, 0, 255]);
            }
        }
        a
    }

    #[test]
    fn shifted_rectangle_gives_one_region_with_its_bbox() {
        let a = rect_img(64, 48, 10, 30);
        let b = rect_img(64, 48, 14, 34);
        let d = compare(&a, &b, &CompareOptions { aa_isolated: 0, ..Default::default() }).unwrap();
        // 4 columns vanish on the left, 4 appear on the right: two strips 40 px tall
        assert_eq!(d.diff_pixels, 2 * 4 * 32);
        assert_eq!(d.regions_total, 2);
        assert_eq!((d.regions[0].x, d.regions[0].w, d.regions[0].y, d.regions[0].h), (10, 4, 8, 32));
        assert_eq!((d.regions[1].x, d.regions[1].w), (30, 4));
        assert!(d.max_de > 90.0 && d.ssim < 0.95);
        // a wide merge distance joins the strips into one region
        let d2 = compare(&a, &b, &CompareOptions { aa_isolated: 0, merge_dist: 20, ..Default::default() }).unwrap();
        assert_eq!(d2.regions_total, 1);
        assert_eq!((d2.regions[0].x, d2.regions[0].w), (10, 24));
        assert_eq!(d2.regions[0].pixels, 2 * 4 * 32);
    }

    #[test]
    fn isolated_edge_pixel_is_forgiven_but_flat_area_noise_and_big_changes_are_not() {
        let a = rect_img(40, 40, 10, 30);
        // one slightly different pixel on the rectangle's left edge
        let mut b = a.clone();
        b.set(10, 20, [60, 60, 60, 255]);
        let o = CompareOptions::default();
        let d = compare(&a, &b, &o).unwrap();
        assert_eq!((d.diff_pixels, d.forgiven_pixels), (0, 1));
        assert_eq!(d.state[20 * 40 + 10], PX_FORGIVEN);
        assert!(d.max_de_raw > 5.0 && d.max_de == 0.0);
        // switched off: it counts
        let d0 = compare(&a, &b, &CompareOptions { aa_isolated: 0, ..o.clone() }).unwrap();
        assert_eq!(d0.diff_pixels, 1);
        // the same change in a flat white area is not an edge: counted
        let mut c = a.clone();
        c.set(2, 2, [60, 60, 60, 255]);
        assert_eq!(compare(&a, &c, &o).unwrap().diff_pixels, 1);
        // a group bigger than the limit is counted
        let mut e = a.clone();
        for y in 15..20 {
            e.set(10, y, [60, 60, 60, 255]);
        }
        let de = compare(&a, &e, &o).unwrap();
        assert_eq!((de.diff_pixels, de.forgiven_pixels), (5, 0));
    }

    #[test]
    fn edge_tolerance_forgives_small_differences_on_edges_only() {
        let a = rect_img(40, 40, 10, 30);
        let mut b = a.clone();
        for y in 10..30 {
            b.set(10, y, [30, 30, 30, 255]); // long line on the edge: too big for the isolated rule
        }
        let o = CompareOptions { aa_isolated: 0, aa_edge_de: 0.0, ..Default::default() };
        assert_eq!(compare(&a, &b, &o).unwrap().diff_pixels, 20);
        let o2 = CompareOptions { aa_edge_de: 15.0, ..o.clone() };
        let d = compare(&a, &b, &o2).unwrap();
        assert_eq!((d.diff_pixels, d.forgiven_pixels), (0, 20));
        let o3 = CompareOptions { aa_edge_de: 5.0, ..o };
        assert_eq!(compare(&a, &b, &o3).unwrap().diff_pixels, 20, "difference larger than the edge tolerance");
    }

    #[test]
    fn ignore_rectangles_remove_pixels_from_every_metric() {
        let a = rect_img(64, 48, 10, 30);
        let b = rect_img(64, 48, 14, 34);
        let o = CompareOptions { aa_isolated: 0, ignore: vec![[0, 0, 64, 48]], ..Default::default() };
        let d = compare(&a, &b, &o).unwrap();
        assert_eq!((d.diff_pixels, d.max_de, d.ssim, d.mse), (0, 0.0, 1.0, 0.0));
        assert_eq!(d.ignored_pixels, 64 * 48);
        // partial: only the left strip is ignored
        let o2 = CompareOptions { aa_isolated: 0, ignore: vec![[8, 0, 8, 48]], ..Default::default() };
        let d2 = compare(&a, &b, &o2).unwrap();
        assert_eq!(d2.diff_pixels, 4 * 32);
        assert_eq!(d2.regions[0].x, 30);
        // rectangles outside or negative are clipped, not errors
        let o3 = CompareOptions { ignore: vec![[-5, -5, 3, 3], [100, 100, 5, 5]], ..Default::default() };
        assert_eq!(compare(&a, &a, &o3).unwrap().ignored_pixels, 0);
    }

    #[test]
    fn size_mismatch_and_bad_options_are_errors() {
        let a = Img::solid(4, 4, [0; 4]);
        let b = Img::solid(5, 4, [0; 4]);
        assert!(compare(&a, &b, &CompareOptions::default()).unwrap_err().contains("sizes differ"));
        assert!(compare(&a, &a, &CompareOptions { pixel_de: f64::NAN, ..Default::default() }).is_err());
        assert!(Img::new(0, 3, vec![]).is_err());
        assert!(Img::new(2, 2, vec![0; 3]).is_err());
        assert!(DeMetric::parse("lab").is_err());
        assert_eq!(DeMetric::parse("cie76").unwrap(), DeMetric::De76);
        assert!(Img::from_png(b"not a png").is_err());
    }

    #[test]
    fn png_round_trip_and_content_hash() {
        let a = noise_img(13, 9, 3);
        let b = Img::from_png(&a.to_png().unwrap()).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.content_hash(), b.content_hash());
        let mut c = a.clone();
        c.rgba[0] ^= 1;
        assert_ne!(a.content_hash(), c.content_hash());
        // same pixels, different shape: different hash
        let d = Img::new(9, 13, a.rgba.clone()).unwrap();
        assert_ne!(a.content_hash(), d.content_hash());
    }

    #[test]
    fn phash_is_stable_and_separates_different_pictures() {
        let a = rect_img(64, 64, 10, 30);
        assert_eq!(phash(&a), phash(&a.clone()));
        let b = rect_img(64, 64, 14, 34);
        let c = noise_img(64, 64, 9);
        assert!(hamming(phash(&a), phash(&b)) < hamming(phash(&a), phash(&c)));
        // a tiny brightness change keeps the hash of a textured picture close
        let mut e = c.clone();
        for p in e.rgba.chunks_exact_mut(4) {
            for v in p.iter_mut().take(3) {
                *v = v.saturating_add(2);
            }
        }
        assert!(hamming(phash(&c), phash(&e)) <= 4, "{}", hamming(phash(&c), phash(&e)));
    }

    #[test]
    fn heatmap_marks_differences_and_images_compose() {
        let a = rect_img(40, 40, 10, 30);
        let b = rect_img(40, 40, 12, 32);
        let d = compare(&a, &b, &CompareOptions { aa_isolated: 0, ..Default::default() }).unwrap();
        let hm = heatmap(&a, &d);
        assert_eq!((hm.w, hm.h), (40, 40));
        assert_ne!(hm.px(10, 20), hm.px(20, 20), "changed strip is coloured, interior is faded");
        let s = side_by_side(&[&a, &b, &hm]);
        assert_eq!((s.w, s.h), (40 * 3 + 8, 40));
        let o = overlay(&a, &b).unwrap();
        assert_eq!(o.px(10, 20)[0..3], [0, 255, 255], "only the first is dark here: cyan");
        let g = flicker_gif(&a, &b).unwrap();
        assert_eq!(&g[..3], b"GIF");
        assert!(overlay(&a, &Img::solid(3, 3, [0; 4])).is_err());
    }
}
