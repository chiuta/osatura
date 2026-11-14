// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Image comparison metrics for the GPU-vs-CPU parity tests (pure CPU, tested without a GPU).

/// How two same-sized premultiplied RGBA8 images differ.
#[derive(Clone, Debug)]
pub struct Parity {
    pub width: u32,
    pub height: u32,
    /// Largest absolute channel difference (0..=255).
    pub max_abs: u8,
    /// Mean absolute channel difference.
    pub mean_abs: f64,
    /// Fraction of pixels where any channel differs by more than `threshold`.
    pub frac_over: f64,
    pub threshold: u8,
    /// Peak signal-to-noise ratio in dB (infinite for identical images).
    pub psnr: f64,
    /// Mean structural similarity over 8x8 blocks of the RGB mean (1.0 = identical).
    pub ssim: f64,
}

impl std::fmt::Display for Parity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}x{} max_abs={} mean_abs={:.4} frac(>{})={:.5} psnr={:.2}dB ssim={:.5}",
            self.width,
            self.height,
            self.max_abs,
            self.mean_abs,
            self.threshold,
            self.frac_over,
            self.psnr,
            self.ssim
        )
    }
}

pub fn compare(
    a: &[u8],
    b: &[u8],
    width: u32,
    height: u32,
    threshold: u8,
) -> Result<Parity, String> {
    let n = width as usize * height as usize * 4;
    if a.len() != n || b.len() != n {
        return Err(format!(
            "size mismatch: expected {n} bytes, got {} and {}",
            a.len(),
            b.len()
        ));
    }
    if n == 0 {
        return Err("empty image".into());
    }
    let (mut max_abs, mut sum, mut sq, mut over) = (0u8, 0u64, 0u64, 0usize);
    for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
        let mut bad = false;
        for k in 0..4 {
            let d = pa[k].abs_diff(pb[k]);
            max_abs = max_abs.max(d);
            sum += d as u64;
            sq += (d as u64) * (d as u64);
            bad |= d > threshold;
        }
        over += bad as usize;
    }
    let mse = sq as f64 / n as f64;
    let psnr = if mse == 0.0 {
        f64::INFINITY
    } else {
        10.0 * (255.0f64 * 255.0 / mse).log10()
    };
    Ok(Parity {
        width,
        height,
        max_abs,
        mean_abs: sum as f64 / n as f64,
        frac_over: over as f64 / (n / 4) as f64,
        threshold,
        psnr,
        ssim: ssim(a, b, width as usize, height as usize),
    })
}

fn luma(img: &[u8], i: usize) -> f64 {
    (img[i * 4] as f64 + img[i * 4 + 1] as f64 + img[i * 4 + 2] as f64) / 3.0
}

fn ssim(a: &[u8], b: &[u8], w: usize, h: usize) -> f64 {
    const BLOCK: usize = 8;
    let (c1, c2) = ((0.01f64 * 255.0).powi(2), (0.03f64 * 255.0).powi(2));
    let (mut total, mut count) = (0.0f64, 0usize);
    let mut by = 0;
    while by < h {
        let mut bx = 0;
        while bx < w {
            let (x1, y1) = ((bx + BLOCK).min(w), (by + BLOCK).min(h));
            let n = ((x1 - bx) * (y1 - by)) as f64;
            let (mut ma, mut mb) = (0.0, 0.0);
            for y in by..y1 {
                for x in bx..x1 {
                    ma += luma(a, y * w + x);
                    mb += luma(b, y * w + x);
                }
            }
            ma /= n;
            mb /= n;
            let (mut va, mut vb, mut cov) = (0.0, 0.0, 0.0);
            for y in by..y1 {
                for x in bx..x1 {
                    let (da, db) = (luma(a, y * w + x) - ma, luma(b, y * w + x) - mb);
                    va += da * da;
                    vb += db * db;
                    cov += da * db;
                }
            }
            va /= n;
            vb /= n;
            cov /= n;
            total += ((2.0 * ma * mb + c1) * (2.0 * cov + c2))
                / ((ma * ma + mb * mb + c1) * (va + vb + c2));
            count += 1;
            bx += BLOCK;
        }
        by += BLOCK;
    }
    total / count as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images_are_perfect() {
        let img: Vec<u8> = (0..16 * 16 * 4).map(|i| (i * 7 % 256) as u8).collect();
        let p = compare(&img, &img, 16, 16, 8).unwrap();
        assert_eq!((p.max_abs, p.mean_abs, p.frac_over), (0, 0.0, 0.0));
        assert!(p.psnr.is_infinite());
        assert!((p.ssim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn metrics_see_a_known_difference() {
        let a = vec![100u8; 8 * 8 * 4];
        let mut b = a.clone();
        b[0] = 150; // one channel of one pixel
        let p = compare(&a, &b, 8, 8, 10).unwrap();
        assert_eq!(p.max_abs, 50);
        assert!((p.mean_abs - 50.0 / 256.0).abs() < 1e-9);
        assert!((p.frac_over - 1.0 / 64.0).abs() < 1e-9);
        assert!(p.psnr.is_finite() && p.ssim <= 1.0);
        let q = compare(&a, &b, 8, 8, 60).unwrap();
        assert_eq!(q.frac_over, 0.0);
    }

    #[test]
    fn ssim_drops_for_unrelated_images() {
        let a: Vec<u8> = (0..32 * 32 * 4)
            .map(|i| ((i / 4) % 2 * 255) as u8)
            .collect();
        let b: Vec<u8> = (0..32 * 32 * 4)
            .map(|i| ((i / 4 / 32) % 2 * 255) as u8)
            .collect();
        assert!(compare(&a, &b, 32, 32, 8).unwrap().ssim < 0.5);
    }

    #[test]
    fn size_errors_are_reported() {
        assert!(compare(&[0; 4], &[0; 8], 1, 1, 0).is_err());
        assert!(compare(&[], &[], 0, 0, 0).is_err());
    }
}
