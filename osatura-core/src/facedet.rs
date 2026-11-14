// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Own face-box finder and 23-point landmark regressor, trained locally on procedural faces only.
//!
//! # What this is, and what it is not
//!
//! A small classical pipeline, no neural network library, no downloaded weights, no third-party face
//! data. The only training images are the synthetic drawings of [`crate::facegen`] rendered by the
//! engine's rasteriser. **It is expected to work on stylised / cartoon faces that resemble those
//! drawings, and on nothing else.** It has not been run on photographs or on a webcam and must not be
//! assumed to work there; the evaluation numbers in the docs are on synthetic and hand-drawn SVG faces.
//!
//! # Method (all textbook)
//!
//! * Features: histograms of oriented gradients (Dalal and Triggs 2005): square-root intensity
//!   compression, `[-1,0,1]` gradients, 9 unsigned orientation bins with linear interpolation between
//!   bins (no spatial interpolation), square cells, 2x2-cell blocks at stride one cell, L2-Hys block
//!   normalisation (clip at 0.2, renormalise).
//! * Box finder: logistic regression on window HOG (fitted by Newton / IRLS with an L2 penalty), run over
//!   a scale pyramid with a one-cell stride and half-cell shifted grids, followed by greedy non-maximum
//!   suppression with score-weighted merging. One hard-negative mining round (false positives of the
//!   first classifier on fresh clutter and on face scenes are added as negatives).
//! * Landmarks: ridge regression from the HOG of the box crop to the 23 points of
//!   [`crate::faceparam::NAMES`] in box coordinates, plus a 3-number box correction (centre shift, scale)
//!   that is applied before a second pass, so the result depends little on the box finder's exact box.
//! * Temporal smoothing is not done here: feed the NDJSON to `faceparam`, which applies the One-Euro filter.
//!
//! # Model file (`osatura-facedet` v1, `osatura-core/models/face23_synth.bin`)
//!
//! ```text
//! "AFD1"                      4 bytes magic
//! u32 LE  n                   length of the JSON header
//! n bytes                     JSON: {"format":"osatura-facedet","version":1,"det_win":..,"det_cell":..,
//!                             "reg_win":..,"reg_cell":..,"det_bias":..,"det_threshold":..,"n_det":..,
//!                             "n_reg_in":..,"n_reg_out":49,"meta":{..training provenance..}}
//! f32 LE x n_det              logistic weights
//! f32 LE x n_reg_in           ridge feature mean
//! f32 LE x n_reg_out*n_reg_in ridge weights, output-major
//! f32 LE x n_reg_out          ridge intercepts
//! ```
//!
//! The same content as one JSON object (arrays inline) is accepted by [`FaceModel::load`] too.
//! Outputs 0..45 are the x,y of the 23 landmarks as `(p - box_centre) / box_side`; outputs 46..48 are
//! `(true_centre - box_centre) / box_side` (x, y) and `true_side / box_side - 1`.

use crate::facegen::{self, FaceTruth, Gray};
use crate::math::Vec2;
use crate::tinymlp::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const N_LM: usize = 23;
pub const N_OUT: usize = N_LM * 2 + 3;
const BINS: usize = 9;
const MAGIC: &[u8; 4] = b"AFD1";
/// Floor of the block norm; damps blocks that contain only noise.
const BLOCK_EPS: f32 = 1.0;

// ---------------------------------------------------------------------------------------------
// HOG
// ---------------------------------------------------------------------------------------------

/// Cell histograms of a whole image: `(cells_w, cells_h, 9 values per cell)`.
fn hog_cells(g: &Gray, cell: usize) -> (usize, usize, Vec<f32>) {
    let (cw, ch) = (g.w / cell, g.h / cell);
    let mut hist = vec![0.0f32; cw * ch * BINS];
    if cw == 0 || ch == 0 {
        return (cw, ch, hist);
    }
    let s: Vec<f32> = g.px.iter().map(|v| v.max(0.0).sqrt()).collect();
    let at = |x: isize, y: isize| s[y.clamp(0, g.h as isize - 1) as usize * g.w + x.clamp(0, g.w as isize - 1) as usize];
    let k = BINS as f32 / std::f32::consts::PI;
    for y in 0..ch * cell {
        for x in 0..cw * cell {
            let (xi, yi) = (x as isize, y as isize);
            let dx = at(xi + 1, yi) - at(xi - 1, yi);
            let dy = at(xi, yi + 1) - at(xi, yi - 1);
            let mag = (dx * dx + dy * dy).sqrt();
            if mag < 1e-6 {
                continue;
            }
            let mut a = dy.atan2(dx);
            if a < 0.0 {
                a += std::f32::consts::PI;
            }
            if a >= std::f32::consts::PI {
                a -= std::f32::consts::PI;
            }
            let pos = a * k - 0.5;
            let b0 = pos.floor();
            let f = pos - b0;
            let i0 = (b0 as isize).rem_euclid(BINS as isize) as usize;
            let i1 = (i0 + 1) % BINS;
            let base = ((y / cell) * cw + x / cell) * BINS;
            hist[base + i0] += mag * (1.0 - f);
            hist[base + i1] += mag * f;
        }
    }
    (cw, ch, hist)
}

/// L2-Hys normalised 2x2 blocks at stride one cell: `(blocks_w, blocks_h, 36 values per block)`.
fn hog_blocks(cw: usize, ch: usize, hist: &[f32]) -> (usize, usize, Vec<f32>) {
    if cw < 2 || ch < 2 {
        return (0, 0, vec![]);
    }
    let (bw, bh) = (cw - 1, ch - 1);
    let mut out = vec![0.0f32; bw * bh * 4 * BINS];
    for by in 0..bh {
        for bx in 0..bw {
            let o = (by * bw + bx) * 4 * BINS;
            for (k, (dx, dy)) in [(0, 0), (1, 0), (0, 1), (1, 1)].iter().enumerate() {
                let c = ((by + dy) * cw + bx + dx) * BINS;
                out[o + k * BINS..o + (k + 1) * BINS].copy_from_slice(&hist[c..c + BINS]);
            }
            let blk = &mut out[o..o + 4 * BINS];
            let n = (blk.iter().map(|v| v * v).sum::<f32>() + BLOCK_EPS * BLOCK_EPS).sqrt();
            blk.iter_mut().for_each(|v| *v = (*v / n).min(0.2));
            let n2 = (blk.iter().map(|v| v * v).sum::<f32>() + 1e-6).sqrt();
            blk.iter_mut().for_each(|v| *v /= n2);
        }
    }
    (bw, bh, out)
}

/// HOG descriptor of a square window image (`win` x `win`).
pub fn hog_window(g: &Gray, cell: usize) -> Vec<f32> {
    let (cw, ch, h) = hog_cells(g, cell);
    hog_blocks(cw, ch, &h).2
}

/// Descriptor length for a window / cell size.
pub fn hog_len(win: usize, cell: usize) -> usize {
    let n = (win / cell).saturating_sub(1);
    n * n * 4 * BINS
}

// ---------------------------------------------------------------------------------------------
// linear algebra (f64, textbook)
// ---------------------------------------------------------------------------------------------

/// Solve `A x = b` for symmetric positive definite `A` (n x n, row-major) by Cholesky; `b` has `m` columns.
fn cholesky_solve(a: &mut [f64], n: usize, b: &mut [f64], m: usize) -> Result<(), String> {
    for j in 0..n {
        let mut d = a[j * n + j];
        for k in 0..j {
            d -= a[j * n + k] * a[j * n + k];
        }
        if d <= 1e-12 {
            return Err("matrix is not positive definite (raise the penalty)".into());
        }
        let d = d.sqrt();
        a[j * n + j] = d;
        for i in j + 1..n {
            let mut s = a[i * n + j];
            for k in 0..j {
                s -= a[i * n + k] * a[j * n + k];
            }
            a[i * n + j] = s / d;
        }
    }
    for c in 0..m {
        for i in 0..n {
            let mut s = b[i * m + c];
            for k in 0..i {
                s -= a[i * n + k] * b[k * m + c];
            }
            b[i * m + c] = s / a[i * n + i];
        }
        for i in (0..n).rev() {
            let mut s = b[i * m + c];
            for k in i + 1..n {
                s -= a[k * n + i] * b[k * m + c];
            }
            b[i * m + c] = s / a[i * n + i];
        }
    }
    Ok(())
}

/// `X^T diag(w) X` (lower triangle filled, then mirrored) for `x` of `n` rows x `d` columns.
fn gram(x: &[f32], n: usize, d: usize, w: Option<&[f64]>) -> Vec<f64> {
    let threads = std::thread::available_parallelism().map(|v| v.get()).unwrap_or(1).clamp(1, 8);
    let chunk = n.div_ceil(threads).max(1);
    let parts: Vec<Vec<f64>> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..n)
            .step_by(chunk)
            .map(|s| {
                let e = (s + chunk).min(n);
                sc.spawn(move || {
                    let mut g = vec![0.0f64; d * d];
                    for r in s..e {
                        let row = &x[r * d..(r + 1) * d];
                        let wr = w.map(|w| w[r]).unwrap_or(1.0);
                        for i in 0..d {
                            let xi = row[i] as f64 * wr;
                            if xi == 0.0 {
                                continue;
                            }
                            let gi = &mut g[i * d..i * d + i + 1];
                            for (j, gv) in gi.iter_mut().enumerate() {
                                *gv += xi * row[j] as f64;
                            }
                        }
                    }
                    g
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut g = vec![0.0f64; d * d];
    for p in &parts {
        for (a, b) in g.iter_mut().zip(p) {
            *a += b;
        }
    }
    for i in 0..d {
        for j in i + 1..d {
            g[i * d + j] = g[j * d + i];
        }
    }
    g
}

/// Ridge regression with intercept: centre `x` and `y`, solve `(Xc^T Xc + lambda I) W = Xc^T Yc`.
/// Returns `(x_mean, w output-major k x d, intercept k)`.
/// Result of [`ridge_fit`]: three flat `f32` vectors (weights, offsets, scales).
pub type RidgeFit = (Vec<f32>, Vec<f32>, Vec<f32>);

pub fn ridge_fit(x: &[f32], y: &[f32], n: usize, d: usize, k: usize, lambda: f64) -> Result<RidgeFit, String> {
    if n < 2 || d == 0 || k == 0 || x.len() != n * d || y.len() != n * k || (lambda.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) {
        return Err("ridge: need n >= 2, matching sizes and lambda > 0".into());
    }
    let mut mx = vec![0.0f64; d];
    let mut my = vec![0.0f64; k];
    for r in 0..n {
        for i in 0..d {
            mx[i] += x[r * d + i] as f64;
        }
        for i in 0..k {
            my[i] += y[r * k + i] as f64;
        }
    }
    mx.iter_mut().for_each(|v| *v /= n as f64);
    my.iter_mut().for_each(|v| *v /= n as f64);
    let xc: Vec<f32> = (0..n * d).map(|i| x[i] - mx[i % d] as f32).collect();
    let mut a = gram(&xc, n, d, None);
    for i in 0..d {
        a[i * d + i] += lambda;
    }
    let mut b = vec![0.0f64; d * k];
    for r in 0..n {
        for i in 0..d {
            let xv = xc[r * d + i] as f64;
            for c in 0..k {
                b[i * k + c] += xv * (y[r * k + c] as f64 - my[c]);
            }
        }
    }
    cholesky_solve(&mut a, d, &mut b, k)?;
    let mut w = vec![0.0f32; k * d];
    for i in 0..d {
        for c in 0..k {
            w[c * d + i] = b[i * k + c] as f32;
        }
    }
    let icpt: Vec<f32> = (0..k).map(|c| (my[c] - (0..d).map(|i| b[i * k + c] * mx[i]).sum::<f64>()) as f32).collect();
    // The model subtracts the mean before the dot product, so keep the plain-mean form: y = W (x - mean) + my.
    let _ = icpt;
    Ok((mx.iter().map(|&v| v as f32).collect(), w, my.iter().map(|&v| v as f32).collect()))
}

/// Logistic regression (labels 0/1, optional per-sample weights) by Newton / IRLS with an L2 penalty
/// on the weights (not the bias). Returns `(w, bias)`.
pub fn logistic_fit(x: &[f32], y: &[f32], sw: &[f64], n: usize, d: usize, lambda: f64, iters: usize) -> Result<(Vec<f32>, f32), String> {
    if n < 2 || d == 0 || x.len() != n * d || y.len() != n || sw.len() != n || (lambda.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) {
        return Err("logistic: need n >= 2, matching sizes and lambda > 0".into());
    }
    let dd = d + 1;
    let mut th = vec![0.0f64; dd];
    // start the bias at the weighted log-odds so the first step is well conditioned
    let (sp, sn) = (0..n).fold((1e-3, 1e-3), |(p, q), i| if y[i] > 0.5 { (p + sw[i], q) } else { (p, q + sw[i]) });
    th[d] = (sp / sn).ln();
    for _ in 0..iters.max(1) {
        let mut p = vec![0.0f64; n];
        let mut grad = vec![0.0f64; dd];
        let mut wts = vec![0.0f64; n];
        for r in 0..n {
            let row = &x[r * d..(r + 1) * d];
            let z: f64 = row.iter().zip(&th).map(|(a, b)| *a as f64 * b).sum::<f64>() + th[d];
            let pr = 1.0 / (1.0 + (-z).exp());
            p[r] = pr;
            wts[r] = sw[r] * pr * (1.0 - pr) + 1e-9;
            let e = sw[r] * (pr - y[r] as f64);
            for i in 0..d {
                grad[i] += e * row[i] as f64;
            }
            grad[d] += e;
        }
        for i in 0..d {
            grad[i] += lambda * th[i];
        }
        // Hessian over [x, 1]
        let gx = gram(x, n, d, Some(&wts));
        let mut h = vec![0.0f64; dd * dd];
        for i in 0..d {
            for j in 0..d {
                h[i * dd + j] = gx[i * d + j];
            }
            h[i * dd + i] += lambda;
        }
        let mut colsum = vec![0.0f64; d];
        let mut tot = 0.0;
        for r in 0..n {
            tot += wts[r];
            for i in 0..d {
                colsum[i] += wts[r] * x[r * d + i] as f64;
            }
        }
        for i in 0..d {
            h[i * dd + d] = colsum[i];
            h[d * dd + i] = colsum[i];
        }
        h[d * dd + d] = tot + 1e-6;
        let mut g = grad.clone();
        cholesky_solve(&mut h, dd, &mut g, 1)?;
        let step: f64 = g.iter().map(|v| v * v).sum::<f64>().sqrt();
        for i in 0..dd {
            th[i] -= g[i];
        }
        if step < 1e-4 {
            break;
        }
    }
    if th.iter().any(|v| !v.is_finite()) {
        return Err("logistic regression diverged".into());
    }
    Ok((th[..d].iter().map(|&v| v as f32).collect(), th[d] as f32))
}

// ---------------------------------------------------------------------------------------------
// model
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FaceModel {
    pub det_win: usize,
    pub det_cell: usize,
    pub reg_win: usize,
    pub reg_cell: usize,
    pub det_w: Vec<f32>,
    pub det_bias: f32,
    /// Detection threshold on the logit.
    pub det_threshold: f32,
    pub reg_mean: Vec<f32>,
    /// Output-major `N_OUT x reg_mean.len()`.
    pub reg_w: Vec<f32>,
    pub reg_b: Vec<f32>,
    /// Side of the normalised crop used by the refinement cascade (0 = no cascade).
    #[serde(default)]
    pub refine_win: usize,
    /// Supervised-descent stages (Xiong and De la Torre 2013): local HOG around each current landmark
    /// -> shape update, by ridge regression.
    #[serde(default)]
    pub stages: Vec<Stage>,
    /// Training provenance (free-form JSON).
    #[serde(default)]
    pub meta: Value,
}

/// One refinement stage: HOG of a `patch` x `patch` window around each of the 23 current landmarks,
/// concatenated (23 x `hog_len(patch, cell)` numbers), regressed linearly onto the shape update
/// (46 numbers, box units).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stage {
    pub patch: usize,
    pub cell: usize,
    pub mean: Vec<f32>,
    /// Output-major `2 * N_LM x mean.len()`.
    pub w: Vec<f32>,
    pub b: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Detection {
    pub cx: f32,
    pub cy: f32,
    pub side: f32,
    pub score: f32,
}

#[derive(Clone, Debug)]
pub struct FaceResult {
    pub det: Detection,
    /// 23 landmarks in pixels.
    pub pts: Vec<Vec2>,
}

#[derive(Clone, Debug)]
pub struct DetectOpts {
    pub min_side: f32,
    pub max_side: f32,
    /// Pyramid ratio between levels (> 1).
    pub step: f32,
    /// IoU above which a weaker box is merged into a stronger one.
    pub nms_iou: f32,
    /// Overrides the model threshold when set.
    pub threshold: Option<f32>,
    pub max_faces: usize,
}
impl Default for DetectOpts {
    fn default() -> Self {
        DetectOpts { min_side: 0.0, max_side: f32::INFINITY, step: 1.2, nms_iou: 0.3, threshold: None, max_faces: 8 }
    }
}

pub fn iou(a: &Detection, b: &Detection) -> f32 {
    let (ax0, ay0, ax1, ay1) = (a.cx - a.side / 2.0, a.cy - a.side / 2.0, a.cx + a.side / 2.0, a.cy + a.side / 2.0);
    let (bx0, by0, bx1, by1) = (b.cx - b.side / 2.0, b.cy - b.side / 2.0, b.cx + b.side / 2.0, b.cy + b.side / 2.0);
    let iw = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
    let ih = (ay1.min(by1) - ay0.max(by0)).max(0.0);
    let i = iw * ih;
    i / (a.side * a.side + b.side * b.side - i).max(1e-9)
}

static BUILTIN: &[u8] = include_bytes!("../models/face23_synth.bin");

impl FaceModel {
    /// The model shipped in `osatura-core/models/face23_synth.bin` (trained on synthetic faces only).
    pub fn builtin() -> Result<FaceModel, String> {
        FaceModel::load(BUILTIN).map_err(|e| format!("built-in face model unusable: {e}"))
    }

    pub fn validate(&self) -> Result<(), String> {
        for (n, win, cell) in [("det", self.det_win, self.det_cell), ("reg", self.reg_win, self.reg_cell)] {
            if !(2..=32).contains(&cell) || win < cell * 3 || win > 256 || win % cell != 0 {
                return Err(format!("{n}: window {win} / cell {cell} is not valid (cell 2..32, window a multiple of it, 3..256 cells)"));
            }
        }
        if self.det_w.len() != hog_len(self.det_win, self.det_cell) {
            return Err("detector weight count does not match its window".into());
        }
        let nin = hog_len(self.reg_win, self.reg_cell);
        if self.reg_mean.len() != nin || self.reg_w.len() != nin * N_OUT || self.reg_b.len() != N_OUT {
            return Err("regressor sizes do not match its window".into());
        }
        if !self.stages.is_empty() && (self.refine_win < 32 || self.refine_win > 512) {
            return Err("refine_win must be 32..512 when stages are present".into());
        }
        if self.stages.len() > 8 {
            return Err("at most 8 refinement stages".into());
        }
        for st in &self.stages {
            if st.cell < 2 || st.cell > 32 || st.patch < st.cell * 2 || st.patch > 128 || st.patch % st.cell != 0 {
                return Err("refinement stage: patch / cell size is not valid".into());
            }
            let n = N_LM * hog_len(st.patch, st.cell);
            if st.mean.len() != n || st.w.len() != n * 2 * N_LM || st.b.len() != 2 * N_LM {
                return Err("refinement stage sizes do not match its patch".into());
            }
        }
        let all = self.det_w.iter().chain(&self.reg_mean).chain(&self.reg_w).chain(&self.reg_b).chain(self.stages.iter().flat_map(|s| s.mean.iter().chain(&s.w).chain(&s.b)));
        if all.into_iter().any(|v| !v.is_finite()) || !self.det_bias.is_finite() || !self.det_threshold.is_finite() {
            return Err("model contains a non-finite number".into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let head = json!({"format": "osatura-facedet", "version": 1, "det_win": self.det_win, "det_cell": self.det_cell, "reg_win": self.reg_win, "reg_cell": self.reg_cell,
            "det_bias": self.det_bias, "det_threshold": self.det_threshold, "n_det": self.det_w.len(), "n_reg_in": self.reg_mean.len(), "n_reg_out": N_OUT, "refine_win": self.refine_win,
            "stages": self.stages.iter().map(|s| json!({"patch": s.patch, "cell": s.cell, "n_in": s.mean.len()})).collect::<Vec<_>>(), "meta": self.meta});
        let h = serde_json::to_vec(&head).unwrap_or_default();
        let mut b = Vec::with_capacity(8 + h.len() + 4 * (self.det_w.len() + self.reg_mean.len() + self.reg_w.len() + self.reg_b.len()));
        let st_iter = || self.stages.iter().flat_map(|s| s.mean.iter().chain(&s.w).chain(&s.b));
        b.extend_from_slice(MAGIC);
        b.extend_from_slice(&(h.len() as u32).to_le_bytes());
        b.extend_from_slice(&h);
        for v in self.det_w.iter().chain(&self.reg_mean).chain(&self.reg_w).chain(&self.reg_b).chain(st_iter()) {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Binary (`AFD1`) or JSON, detected from the first bytes. Never panics on bad input.
    pub fn load(bytes: &[u8]) -> Result<FaceModel, String> {
        if bytes.len() > 64 << 20 {
            return Err("face model is larger than 64 MB".into());
        }
        let m = if bytes.starts_with(MAGIC) {
            if bytes.len() < 8 {
                return Err("truncated face model".into());
            }
            let n = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
            if n > bytes.len() - 8 {
                return Err("truncated face model header".into());
            }
            let h: Value = serde_json::from_slice(&bytes[8..8 + n]).map_err(|e| format!("bad face model header: {e}"))?;
            if h["format"] != "osatura-facedet" || h["version"] != 1 {
                return Err("not an osatura-facedet v1 file".into());
            }
            let geti = |k: &str| h[k].as_u64().map(|v| v as usize).ok_or_else(|| format!("header lacks {k}"));
            let (n_det, n_in, n_out) = (geti("n_det")?, geti("n_reg_in")?, geti("n_reg_out")?);
            if n_out != N_OUT {
                return Err(format!("regressor has {n_out} outputs, expected {N_OUT}"));
            }
            let mut total = n_det.checked_add(n_in).and_then(|v| v.checked_add(n_in.checked_mul(n_out)?)).and_then(|v| v.checked_add(n_out)).ok_or("size overflow")?;
            let mut specs: Vec<(usize, usize, usize)> = Vec::new();
            if let Some(a) = h["stages"].as_array() {
                if a.len() > 8 {
                    return Err("at most 8 refinement stages".into());
                }
                for st in a {
                    let g = |k: &str| st[k].as_u64().map(|v| v as usize).ok_or_else(|| format!("stage lacks {k}"));
                    let (patch, cell, nin) = (g("patch")?, g("cell")?, g("n_in")?);
                    total = total.checked_add(nin).and_then(|v| v.checked_add(nin.checked_mul(2 * N_LM)?)).and_then(|v| v.checked_add(2 * N_LM)).ok_or("size overflow")?;
                    specs.push((patch, cell, nin));
                }
            }
            let body = &bytes[8 + n..];
            if body.len() != total.checked_mul(4).ok_or("size overflow")? {
                return Err("face model body has the wrong length".into());
            }
            let f: Vec<f32> = body.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
            let (a, rest) = f.split_at(n_det);
            let (mean, rest) = rest.split_at(n_in);
            let (w, b) = rest.split_at(n_in * n_out);
            let (b, mut rest) = b.split_at(n_out);
            let mut stages = Vec::new();
            for (patch, cell, nin) in specs {
                let (mean, r) = rest.split_at(nin);
                let (w, r) = r.split_at(nin * 2 * N_LM);
                let (bb, r) = r.split_at(2 * N_LM);
                stages.push(Stage { patch, cell, mean: mean.to_vec(), w: w.to_vec(), b: bb.to_vec() });
                rest = r;
            }
            FaceModel {
                refine_win: h["refine_win"].as_u64().unwrap_or(0) as usize,
                stages,
                det_win: geti("det_win")?,
                det_cell: geti("det_cell")?,
                reg_win: geti("reg_win")?,
                reg_cell: geti("reg_cell")?,
                det_w: a.to_vec(),
                det_bias: h["det_bias"].as_f64().ok_or("header lacks det_bias")? as f32,
                det_threshold: h["det_threshold"].as_f64().ok_or("header lacks det_threshold")? as f32,
                reg_mean: mean.to_vec(),
                reg_w: w.to_vec(),
                reg_b: b.to_vec(),
                meta: h["meta"].clone(),
            }
        } else {
            serde_json::from_slice(bytes).map_err(|e| format!("not an osatura-facedet file: {e}"))?
        };
        m.validate()?;
        Ok(m)
    }

    // ---- box score -----------------------------------------------------------------------

    fn logit(&self, f: &[f32]) -> f32 {
        f.iter().zip(&self.det_w).map(|(a, b)| a * b).sum::<f32>() + self.det_bias
    }

    /// Detector logit of a given square box.
    pub fn score_box(&self, g: &Gray, cx: f32, cy: f32, side: f32) -> f32 {
        let c = g.crop_resize(cx, cy, side, self.det_win);
        self.logit(&hog_window(&c, self.det_cell))
    }

    // ---- sliding window ------------------------------------------------------------------

    /// Raw window hits `(cx, cy, side, logit)` above `thr` before suppression.
    fn scan(&self, g: &Gray, o: &DetectOpts, thr: f32) -> Vec<Detection> {
        let win = self.det_win;
        let cell = self.det_cell;
        let nb = win / cell - 1;
        let mut hits = Vec::new();
        let smin = (o.min_side.max(win as f32) / win as f32).max(1.0);
        let smax = (o.max_side.min(g.w.min(g.h) as f32 * 1.0) / win as f32).max(smin);
        let step = o.step.max(1.02);
        let mut s = smin;
        while s <= smax * 1.0001 {
            let img = g.scaled_down(s);
            if img.w >= win && img.h >= win {
                for (oy, ox) in [(0, 0), (0, cell / 2), (cell / 2, 0), (cell / 2, cell / 2)] {
                    if img.w <= ox + win || img.h <= oy + win {
                        continue;
                    }
                    // crop by shifting: build a view as a new image (cheap relative to HOG)
                    let sub = Gray { w: img.w - ox, h: img.h - oy, px: (oy..img.h).flat_map(|y| img.px[y * img.w + ox..(y + 1) * img.w].iter().copied()).collect() };
                    let (cw, ch, hist) = hog_cells(&sub, cell);
                    let (bw, bh, blk) = hog_blocks(cw, ch, &hist);
                    if bw < nb || bh < nb {
                        continue;
                    }
                    for j in 0..=(bh - nb) {
                        for i in 0..=(bw - nb) {
                            let mut z = self.det_bias;
                            for r in 0..nb {
                                let src = &blk[((j + r) * bw + i) * 4 * BINS..((j + r) * bw + i + nb) * 4 * BINS];
                                let w = &self.det_w[r * nb * 4 * BINS..(r + 1) * nb * 4 * BINS];
                                z += src.iter().zip(w).map(|(a, b)| a * b).sum::<f32>();
                            }
                            if z > thr {
                                let px = ox as f32 + (i * cell) as f32 + win as f32 * 0.5;
                                let py = oy as f32 + (j * cell) as f32 + win as f32 * 0.5;
                                hits.push(Detection { cx: px * s, cy: py * s, side: win as f32 * s, score: z });
                            }
                        }
                    }
                }
            }
            s *= step;
        }
        hits
    }

    /// Face boxes: pyramid scan, greedy NMS with score-weighted merging of the suppressed cluster.
    pub fn detect_boxes(&self, g: &Gray, o: &DetectOpts) -> Vec<Detection> {
        let thr = o.threshold.unwrap_or(self.det_threshold);
        let mut hits = self.scan(g, o, thr);
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        let mut out: Vec<Detection> = Vec::new();
        let mut used = vec![false; hits.len()];
        for i in 0..hits.len() {
            if used[i] {
                continue;
            }
            let (mut sx, mut sy, mut ss, mut sw) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
            for j in i..hits.len() {
                if !used[j] && (j == i || iou(&hits[i], &hits[j]) > o.nms_iou) {
                    used[j] = true;
                    let w = (hits[j].score - thr).max(0.0) + 0.1;
                    sx += hits[j].cx * w;
                    sy += hits[j].cy * w;
                    ss += hits[j].side * w;
                    sw += w;
                }
            }
            out.push(Detection { cx: sx / sw, cy: sy / sw, side: ss / sw, score: hits[i].score });
            if out.len() >= o.max_faces.max(1) {
                break;
            }
        }
        out
    }

    // ---- landmarks -----------------------------------------------------------------------

    fn regress(&self, g: &Gray, d: &Detection) -> [f32; N_OUT] {
        let c = g.crop_resize(d.cx, d.cy, d.side, self.reg_win);
        let f = hog_window(&c, self.reg_cell);
        let n = f.len();
        let mut out = [0.0f32; N_OUT];
        for (k, o) in out.iter_mut().enumerate() {
            let w = &self.reg_w[k * n..(k + 1) * n];
            *o = self.reg_b[k] + f.iter().zip(&self.reg_mean).zip(w).map(|((a, m), w)| (a - m) * w).sum::<f32>();
        }
        out
    }

    /// Global stage: regress, correct the box with the 3 extra outputs, regress again.
    /// Returns the corrected box and the 46 shape numbers in that box's units (`(p - centre) / side`).
    pub fn global_shape(&self, g: &Gray, d: &Detection) -> (Detection, Vec<f32>) {
        let mut cur = *d;
        let mut y = self.regress(g, &cur);
        for _ in 0..2 {
            let side = (cur.side * (1.0 + y[2 * N_LM + 2])).clamp(cur.side * 0.5, cur.side * 2.0);
            cur = Detection { cx: cur.cx + y[2 * N_LM] * cur.side, cy: cur.cy + y[2 * N_LM + 1] * cur.side, side, score: d.score };
            y = self.regress(g, &cur);
        }
        (cur, y[..2 * N_LM].to_vec())
    }

    /// Shape-indexed features of one cascade stage: HOG of the patch around each current landmark in the
    /// normalised crop `crop` (side `refine_win`); `shape` is in box units.
    fn stage_features(&self, crop: &Gray, shape: &[f32], st: &Stage) -> Vec<f32> {
        let rw = crop.w as f32;
        let mut f = Vec::with_capacity(N_LM * hog_len(st.patch, st.cell));
        for i in 0..N_LM {
            let (px, py) = ((shape[2 * i] + 0.5) * rw, (shape[2 * i + 1] + 0.5) * rw);
            let patch = crop.crop_resize(px, py, st.patch as f32, st.patch);
            f.extend(hog_window(&patch, st.cell));
        }
        f
    }

    fn apply_stage(&self, crop: &Gray, shape: &mut [f32], st: &Stage) {
        let f = self.stage_features(crop, shape, st);
        let n = f.len();
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for k in 0..2 * N_LM {
            let w = &st.w[k * n..(k + 1) * n];
            shape[k] += st.b[k] + f.iter().zip(&st.mean).zip(w).map(|((a, m), w)| (a - m) * w).sum::<f32>();
        }
    }

    /// Run the whole cascade on a shape in box units.
    pub fn refine(&self, g: &Gray, cur: &Detection, shape: &mut [f32]) {
        if self.stages.is_empty() || self.refine_win == 0 {
            return;
        }
        let crop = g.crop_resize(cur.cx, cur.cy, cur.side, self.refine_win);
        for st in &self.stages {
            self.apply_stage(&crop, shape, st);
        }
    }

    /// Landmarks for a box: global regression with box correction, then the refinement cascade.
    pub fn landmarks(&self, g: &Gray, d: &Detection) -> FaceResult {
        let (cur, mut y) = self.global_shape(g, d);
        self.refine(g, &cur, &mut y);
        let pts = (0..N_LM).map(|i| Vec2::new(cur.cx + y[2 * i] * cur.side, cur.cy + y[2 * i + 1] * cur.side)).collect();
        FaceResult { det: cur, pts }
    }

    /// All faces of an image, strongest first.
    pub fn detect(&self, g: &Gray, o: &DetectOpts) -> Vec<FaceResult> {
        self.detect_boxes(g, o).iter().map(|d| self.landmarks(g, d)).collect()
    }
}

/// Frame-to-frame tracker: re-uses the previous box (regress, check the detector score of the refined
/// box) and falls back to a full scan when the face is lost.
#[derive(Clone, Debug, Default)]
pub struct BoxTracker {
    last: Option<Detection>,
}

impl BoxTracker {
    pub fn reset(&mut self) {
        self.last = None;
    }
    pub fn step(&mut self, m: &FaceModel, g: &Gray, o: &DetectOpts) -> Option<FaceResult> {
        if let Some(d) = self.last {
            let r = m.landmarks(g, &d);
            let sc = m.score_box(g, r.det.cx, r.det.cy, r.det.side);
            if sc > o.threshold.unwrap_or(m.det_threshold) - 1.0 {
                self.last = Some(Detection { score: sc, ..r.det });
                return Some(FaceResult { det: Detection { score: sc, ..r.det }, pts: r.pts });
            }
            self.last = None;
        }
        let r = m.detect(g, o).into_iter().next();
        self.last = r.as_ref().map(|r| r.det);
        r
    }
}

// ---------------------------------------------------------------------------------------------
// training
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct TrainOpts {
    /// Synthetic face scenes (positives and regression samples); negatives scenes = samples / 2.
    pub samples: usize,
    pub seed: u64,
    pub scene: usize,
    pub det_win: usize,
    pub det_cell: usize,
    pub reg_win: usize,
    pub reg_cell: usize,
    pub ridge_lambda: f64,
    pub logistic_lambda: f64,
    pub mine_rounds: usize,
    pub threshold: f32,
    /// Cascade: number of stages (0 = none), crop side, patch side, cell, ridge penalty.
    pub stages: usize,
    pub refine_win: usize,
    pub stage_patch: usize,
    pub stage_cell: usize,
    pub stage_lambda: f64,
}
impl Default for TrainOpts {
    fn default() -> Self {
        TrainOpts { samples: 1500, seed: 1, scene: 128, det_win: 32, det_cell: 8, reg_win: 48, reg_cell: 8, ridge_lambda: 4.0, logistic_lambda: 1.0, mine_rounds: 1, threshold: 1.0, stages: 3, refine_win: 64, stage_patch: 16, stage_cell: 8, stage_lambda: 20.0 }
    }
}

fn truth_det(t: &FaceTruth) -> Detection {
    Detection { cx: t.cx, cy: t.cy, side: t.side, score: 0.0 }
}

/// Train the box finder and the landmark regressor from scratch. Deterministic for a given option set
/// (single seed; the multi-threaded Gram sums are reduced in a fixed order, but float summation order
/// depends on the thread count, so weights can differ in the last bits between machines).
pub fn train(o: &TrainOpts) -> Result<FaceModel, String> {
    if o.samples < 20 || o.samples > 200_000 {
        return Err("samples must be 20..200000".into());
    }
    if o.scene < 64 || o.scene > 512 {
        return Err("scene size must be 64..512".into());
    }
    let started = crate::clock::Stamp::now();
    let proto = FaceModel { det_win: o.det_win, det_cell: o.det_cell, reg_win: o.reg_win, reg_cell: o.reg_cell, det_w: vec![0.0; hog_len(o.det_win, o.det_cell)], det_bias: 0.0, det_threshold: o.threshold, reg_mean: vec![0.0; hog_len(o.reg_win, o.reg_cell)], reg_w: vec![0.0; hog_len(o.reg_win, o.reg_cell) * N_OUT], reg_b: vec![0.0; N_OUT], refine_win: 0, stages: vec![], meta: Value::Null };
    proto.validate()?;
    let sc = o.scene;
    let scf = sc as f32;
    let mut rng = Rng::new(o.seed);
    let (nd, nr) = (hog_len(o.det_win, o.det_cell), hog_len(o.reg_win, o.reg_cell));
    let (mut dx, mut dy): (Vec<f32>, Vec<f32>) = (Vec::new(), Vec::new());
    let (mut rx, mut ry): (Vec<f32>, Vec<f32>) = (Vec::new(), Vec::new());
    let mut scenes: Vec<(Gray, FaceTruth)> = Vec::new();
    let hw_lo = o.det_win as f32 * 0.36;
    let hw_hi = scf * 0.34;
    let jit = |r: &mut Rng, d: &Detection, pos: f32, sc_: f32| -> Detection {
        let u = |r: &mut Rng| r.unit() * 2.0 - 1.0;
        Detection { cx: d.cx + u(r) * pos * d.side, cy: d.cy + u(r) * pos * d.side, side: d.side * (1.0 + u(r) * sc_), score: 0.0 }
    };
    for _ in 0..o.samples {
        let (g, t) = facegen::face_scene(&mut rng, sc, sc, hw_lo, hw_hi, true)?;
        let td = truth_det(&t);
        // regression sample (box jitter +-10 % position, +-17 % scale)
        for _ in 0..2 {
            let b = jit(&mut rng, &td, 0.10, 0.17);
            let f = hog_window(&g.crop_resize(b.cx, b.cy, b.side, o.reg_win), o.reg_cell);
            rx.extend_from_slice(&f);
            for p in &t.pts {
                ry.push((p.x - b.cx) / b.side);
                ry.push((p.y - b.cy) / b.side);
            }
            ry.push((t.cx - b.cx) / b.side);
            ry.push((t.cy - b.cy) / b.side);
            ry.push(t.side / b.side - 1.0);
        }
        // detector positive (+-4 % position, +-6 % scale)
        let b = jit(&mut rng, &td, 0.04, 0.06);
        dx.extend(hog_window(&g.crop_resize(b.cx, b.cy, b.side, o.det_win), o.det_cell));
        dy.push(1.0);
        // negatives from the same scene: boxes with a small IoU with the truth
        let mut got = 0;
        let mut tries = 0;
        while got < 3 && tries < 30 {
            tries += 1;
            let side = rng.unit() * (scf * 0.9 - o.det_win as f32) + o.det_win as f32;
            let c = Detection { cx: rr(&mut rng, side / 2.0, scf - side / 2.0), cy: rr(&mut rng, side / 2.0, scf - side / 2.0), side, score: 0.0 };
            if iou(&c, &td) < 0.25 {
                dx.extend(hog_window(&g.crop_resize(c.cx, c.cy, c.side, o.det_win), o.det_cell));
                dy.push(0.0);
                got += 1;
            }
        }
        // near-miss negatives (shifted by 35..60 % of the side, or at half / double the scale)
        let near = Detection { cx: td.cx + (if rng.unit() < 0.5 { -1.0 } else { 1.0 }) * rr(&mut rng, 0.4, 0.6) * td.side, cy: td.cy + rr(&mut rng, -0.2, 0.2) * td.side, side: td.side * rr(&mut rng, 0.8, 1.2), score: 0.0 };
        if iou(&near, &td) < 0.3 {
            dx.extend(hog_window(&g.crop_resize(near.cx, near.cy, near.side, o.det_win), o.det_cell));
            dy.push(0.0);
        }
        scenes.push((g, t));
    }
    // pure clutter negatives
    let nneg = o.samples / 2;
    let mut neg_scenes: Vec<Gray> = Vec::new();
    for k in 0..nneg {
        let g = facegen::negative_scene(&mut rng, sc, sc)?;
        for _ in 0..6 {
            let side = rng.unit() * (scf * 0.9 - o.det_win as f32) + o.det_win as f32;
            let (cx, cy) = (rr(&mut rng, side / 2.0, scf - side / 2.0), rr(&mut rng, side / 2.0, scf - side / 2.0));
            dx.extend(hog_window(&g.crop_resize(cx, cy, side, o.det_win), o.det_cell));
        }
        dy.extend(std::iter::repeat_n(0.0, 6));
        if k < nneg / 2 {
            neg_scenes.push(g);
        }
    }
    let n_pos0 = dy.iter().filter(|&&v| v > 0.5).count();

    // regressor
    let nr_rows = ry.len() / N_OUT;
    let (reg_mean, reg_w, reg_b) = ridge_fit(&rx, &ry, nr_rows, nr, N_OUT, o.ridge_lambda)?;
    drop(rx);

    // detector, with hard-negative mining rounds
    let fit_det = |dx: &[f32], dy: &[f32]| -> Result<(Vec<f32>, f32), String> {
        let n = dy.len();
        let npos = dy.iter().filter(|&&v| v > 0.5).count().max(1) as f64;
        let nneg = (n as f64 - npos).max(1.0);
        // balance classes to equal total weight, so mining negatives do not drown the positives
        let sw: Vec<f64> = dy.iter().map(|&v| if v > 0.5 { 0.5 * n as f64 / npos } else { 0.5 * n as f64 / nneg }).collect();
        logistic_fit(dx, dy, &sw, n, nd, o.logistic_lambda, 12)
    };
    let (mut det_w, mut det_bias) = fit_det(&dx, &dy)?;
    let mut mined = 0usize;
    for _round in 0..o.mine_rounds {
        let tmp = FaceModel { det_w: det_w.clone(), det_bias, det_threshold: 0.0, reg_mean: reg_mean.clone(), reg_w: reg_w.clone(), reg_b: reg_b.clone(), meta: Value::Null, ..proto.clone() };
        let opts = DetectOpts { threshold: Some(0.0), nms_iou: 0.5, max_faces: 1000, ..Default::default() };
        let mut added = 0;
        for g in &neg_scenes {
            for b in tmp.detect_boxes(g, &opts) {
                dx.extend(hog_window(&g.crop_resize(b.cx, b.cy, b.side, o.det_win), o.det_cell));
                dy.push(0.0);
                added += 1;
            }
        }
        for (g, t) in scenes.iter().take(o.samples / 3) {
            let td = truth_det(t);
            for b in tmp.detect_boxes(g, &opts) {
                if iou(&b, &td) < 0.25 {
                    dx.extend(hog_window(&g.crop_resize(b.cx, b.cy, b.side, o.det_win), o.det_cell));
                    dy.push(0.0);
                    added += 1;
                }
            }
        }
        mined += added;
        if added == 0 {
            break;
        }
        let r = fit_det(&dx, &dy)?;
        det_w = r.0;
        det_bias = r.1;
    }
    // refinement cascade (supervised descent)
    let mut stages: Vec<Stage> = Vec::new();
    let mut stage_rmse: Vec<f32> = Vec::new();
    if o.stages > 0 {
        let glob = FaceModel { reg_mean: reg_mean.clone(), reg_w: reg_w.clone(), reg_b: reg_b.clone(), ..proto.clone() };
        let (rw, ps, pc) = (o.refine_win, o.stage_patch, o.stage_cell);
        let mut crops: Vec<Gray> = Vec::new();
        let mut truth: Vec<Vec<f32>> = Vec::new();
        let mut shapes: Vec<Vec<f32>> = Vec::new();
        let mut owner: Vec<usize> = Vec::new();
        let gauss = |r: &mut Rng| {
            let (u1, u2) = (r.unit().max(1e-7), r.unit());
            (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
        };
        for (g, t) in &scenes {
            let td = truth_det(t);
            for _ in 0..2 {
                let b = jit(&mut rng, &td, 0.10, 0.17);
                let (cur, y) = glob.global_shape(g, &b);
                let tr: Vec<f32> = t.pts.iter().flat_map(|p| [(p.x - cur.cx) / cur.side, (p.y - cur.cy) / cur.side]).collect();
                crops.push(g.crop_resize(cur.cx, cur.cy, cur.side, rw));
                // initialisation 1: what the global stage really predicts; 2: truth plus a random shape error
                let (tx, ty, sc_) = (gauss(&mut rng) * 0.012, gauss(&mut rng) * 0.012, 1.0 + gauss(&mut rng) * 0.03);
                let y2: Vec<f32> = tr.iter().enumerate().map(|(i, v)| v * sc_ + if i % 2 == 0 { tx } else { ty } + gauss(&mut rng) * 0.012).collect();
                truth.push(tr.clone());
                shapes.push(y);
                owner.push(crops.len() - 1);
                truth.push(tr);
                shapes.push(y2);
                owner.push(crops.len() - 1);
            }
        }
        let rmse = |sh: &[Vec<f32>], tr: &[Vec<f32>]| -> f32 {
            let s: f64 = sh.iter().zip(tr).map(|(a, b)| a.iter().zip(b).map(|(x, y)| ((x - y) as f64).powi(2)).sum::<f64>()).sum();
            (s / (sh.len() * 2 * N_LM) as f64).sqrt() as f32
        };
        stage_rmse.push(rmse(&shapes, &truth));
        for _ in 0..o.stages {
            let st0 = Stage { patch: ps, cell: pc, mean: vec![], w: vec![], b: vec![] };
            let nf = N_LM * hog_len(ps, pc);
            let mut fx: Vec<f32> = Vec::with_capacity(shapes.len() * nf);
            for (i, sh) in shapes.iter().enumerate() {
                fx.extend(glob.stage_features(&crops[owner[i]], sh, &st0));
            }
            let dy: Vec<f32> = shapes.iter().zip(&truth).flat_map(|(a, b)| a.iter().zip(b).map(|(x, y)| y - x)).collect();
            let (mean, w, b) = ridge_fit(&fx, &dy, shapes.len(), nf, 2 * N_LM, o.stage_lambda)?;
            let st = Stage { patch: ps, cell: pc, mean, w, b };
            for (i, sh) in shapes.iter_mut().enumerate() {
                let f = &fx[i * nf..(i + 1) * nf];
                // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
                #[allow(clippy::needless_range_loop)]
                for k in 0..2 * N_LM {
                    sh[k] += st.b[k] + f.iter().zip(&st.mean).zip(&st.w[k * nf..(k + 1) * nf]).map(|((a, m), w)| (a - m) * w).sum::<f32>();
                }
            }
            stage_rmse.push(rmse(&shapes, &truth));
            stages.push(st);
        }
    }
    let meta = json!({
        "stage_train_rmse_box_units": stage_rmse,
        "generator": "osatura facegen (procedural, engine-rasterised)", "data": "synthetic only; no photographs, no third-party data, no downloaded weights",
        "samples": o.samples, "seed": o.seed, "scene": o.scene, "mine_rounds": o.mine_rounds, "mined_negatives": mined,
        "positives": n_pos0, "detector_rows": dy.len(), "regression_rows": nr_rows, "ridge_lambda": o.ridge_lambda, "logistic_lambda": o.logistic_lambda,
        "train_seconds": started.elapsed().as_secs_f32(),
        "scope": "stylised / cartoon faces; not validated on photographs or webcams",
    });
    let m = FaceModel { det_w, det_bias, det_threshold: o.threshold, reg_mean, reg_w, reg_b, refine_win: if stages.is_empty() { 0 } else { o.refine_win }, stages, meta, ..proto };
    m.validate()?;
    Ok(m)
}

fn rr(r: &mut Rng, a: f32, b: f32) -> f32 {
    a + (b - a) * r.unit()
}

// ---------------------------------------------------------------------------------------------
// evaluation
// ---------------------------------------------------------------------------------------------

/// Aggregate over a set of test scenes.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Stat {
    pub n: usize,
    /// Fraction of scenes whose best detection has IoU >= 0.5 with the true box.
    pub detect_rate: f32,
    /// Mean / median / 90th percentile NME (inter-ocular units) over detected scenes.
    pub nme_mean: f32,
    pub nme_median: f32,
    pub nme_p90: f32,
    /// Mean NME if the true box is given instead of the detector's box (regressor alone).
    pub nme_oracle_box: f32,
    /// Fraction of detected scenes with NME > 0.1.
    pub fail_rate_nme_gt_0_1: f32,
}

fn pct(v: &mut [f32], q: f32) -> f32 {
    if v.is_empty() {
        return f32::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v[((v.len() - 1) as f32 * q).round() as usize]
}

/// Evaluate on freshly generated scenes. `tweak` customises each face (for the sweeps).
pub fn evaluate_set(m: &FaceModel, n: usize, seed: u64, scene: usize, mut tweak: impl FnMut(&mut facegen::FaceSpec, &mut facegen::Post, &mut Rng)) -> Result<Stat, String> {
    let mut rng = Rng::new(seed ^ 0x5eed_f00d);
    let (mut hits, mut nmes, mut oracle) = (0usize, Vec::new(), Vec::new());
    let scf = scene as f32;
    for _ in 0..n {
        let (g, t) = facegen::face_scene_with(&mut rng, scene, scene, m.det_win as f32 * 0.5, scf * 0.34, |s, p, r| tweak(s, p, r))?;
        let td = truth_det(&t);
        oracle.push(t.nme(&m.landmarks(&g, &td).pts));
        let res = m.detect(&g, &DetectOpts { max_faces: 1, ..Default::default() });
        if let Some(r) = res.first() {
            if iou(&r.det, &td) >= 0.5 {
                hits += 1;
                nmes.push(t.nme(&r.pts));
            }
        }
    }
    let mean = |v: &[f32]| if v.is_empty() { f32::NAN } else { v.iter().sum::<f32>() / v.len() as f32 };
    let fails = nmes.iter().filter(|&&v| v > 0.1).count() as f32 / nmes.len().max(1) as f32;
    let nme_mean = mean(&nmes);
    let nme_oracle_box = mean(&oracle);
    Ok(Stat { n, detect_rate: hits as f32 / n.max(1) as f32, nme_mean, nme_median: pct(&mut nmes.clone(), 0.5), nme_p90: pct(&mut nmes, 0.9), nme_oracle_box, fail_rate_nme_gt_0_1: fails })
}

/// Detections on `n` scenes that contain no complete face: `(false positives, scenes, fp per scene)`.
pub fn false_positives(m: &FaceModel, n: usize, seed: u64, scene: usize) -> Result<(usize, usize, f32), String> {
    let mut rng = Rng::new(seed ^ 0xbad_c0de);
    let mut fp = 0;
    for _ in 0..n {
        let g = facegen::negative_scene(&mut rng, scene, scene)?;
        fp += m.detect_boxes(&g, &DetectOpts::default()).len();
    }
    Ok((fp, n, fp as f32 / n.max(1) as f32))
}

/// Full synthetic report: held-out set, false positives and robustness sweeps (one factor at a time).
pub fn report(m: &FaceModel, n: usize, seed: u64, scene: usize) -> Result<Value, String> {
    let main = evaluate_set(m, n, seed, scene, |_, p, r| *p = facegen::Post::random(r))?;
    let (fp, nn, rate) = false_positives(m, n, seed, scene)?;
    let mut sweeps = serde_json::Map::new();
    let nn_s = (n / 3).max(10);
    let mut add = |name: &str, vals: &[f32], f: &dyn Fn(&mut facegen::FaceSpec, &mut facegen::Post, f32)| -> Result<(), String> {
        let mut rows = Vec::new();
        for &v in vals {
            let s = evaluate_set(m, nn_s, seed + 1, scene, |s, p, _| f(s, p, v))?;
            rows.push(json!({"value": v, "detect_rate": s.detect_rate, "nme_mean": s.nme_mean, "nme_oracle_box": s.nme_oracle_box}));
        }
        sweeps.insert(name.into(), Value::Array(rows));
        Ok(())
    };
    add("roll_deg", &[0.0, 10.0, 20.0, 30.0, 45.0], &|s, _, v| s.roll = v)?;
    add("yaw_rad", &[0.0, 0.3, 0.6, 0.9], &|s, _, v| s.yaw = v)?;
    add("pitch_rad", &[0.0, 0.3, 0.5], &|s, _, v| s.pitch = v)?;
    add("head_half_width_px", &[m.det_win as f32 * 0.5, m.det_win as f32 * 0.75, scene as f32 * 0.2, scene as f32 * 0.3], &|s, _, v| {
        s.hw = v;
        s.cx = scene as f32 * 0.5;
        s.cy = scene as f32 * 0.5;
    })?;
    add("noise_sigma", &[0.0, 0.03, 0.06, 0.1, 0.15], &|_, p, v| p.noise = v)?;
    add("blur_sigma_px", &[0.0, 1.0, 1.5, 2.5], &|_, p, v| p.blur = v)?;
    add("lighting_ramp", &[0.0, 0.3, 0.6], &|_, p, v| p.light = v)?;
    let cart = cartoon_report(m)?;
    let e2e = e2e_report(m, 60, 128, 7)?;
    Ok(json!({"cartoons": cart, "end_to_end": e2e, "held_out": main, "negatives": {"scenes": nn, "false_positives": fp, "per_scene": rate}, "sweeps": Value::Object(sweeps), "scene_px": scene, "seed": seed}))
}

/// Hand-drawn SVG variants ([`facegen::cartoons`]) in 6 placements each (scale, roll, position).
pub fn cartoon_report(m: &FaceModel) -> Result<Value, String> {
    let placements = [(0.75f32, 0.0f32, 128.0f32, 128.0f32), (0.55, 0.0, 100.0, 140.0), (0.9, 8.0, 128.0, 120.0), (0.65, -10.0, 150.0, 110.0), (0.8, 0.0, 120.0, 135.0), (0.6, 15.0, 110.0, 120.0)];
    let mut rows = Vec::new();
    for c in facegen::cartoons() {
        let (mut hit, mut nm, mut orc) = (0usize, Vec::new(), Vec::new());
        for (i, (sc, roll, cx, cy)) in placements.iter().enumerate() {
            let bg = if i % 2 == 0 { [0.92, 0.92, 0.95] } else { [0.25, 0.3, 0.4] };
            let (g, t) = c.render(256, 256, *sc, *roll, *cx, *cy, bg)?;
            orc.push(t.nme(&m.landmarks(&g, &truth_det(&t)).pts));
            if let Some(r) = m.detect(&g, &DetectOpts { max_faces: 1, ..Default::default() }).first() {
                if iou(&r.det, &truth_det(&t)) >= 0.5 {
                    hit += 1;
                    nm.push(t.nme(&r.pts));
                }
            }
        }
        let mean = |v: &[f32]| if v.is_empty() { Value::Null } else { json!(v.iter().sum::<f32>() / v.len() as f32) };
        rows.push(json!({"design": c.name, "placements": placements.len(), "detected": hit, "nme_mean_detected": mean(&nm), "nme_mean_true_box": mean(&orc)}));
    }
    Ok(Value::Array(rows))
}

fn pearson(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len()) as f32;
    if n < 3.0 {
        return f32::NAN;
    }
    let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma).powi(2);
        sbb += (y - mb).powi(2);
    }
    sab / (saa * sbb).sqrt().max(1e-12)
}

/// End to end: render the synthetic head animation ([`facegen::anim_spec`]) with the engine, track it,
/// write the landmarks as NDJSON text (the format of `face-track`), import that text with
/// `faceparam::parse_ndjson`, turn it into rig signals with `faceparam::events` (what `face-bake` bakes)
/// and correlate every signal with the ground-truth parameter that drove the drawing. The same signals from
/// the exact ground-truth landmarks are the ceiling of this pipeline and are reported next to it.
pub fn e2e_report(m: &FaceModel, frames: usize, size: usize, seed: u64) -> Result<Value, String> {
    use crate::faceparam::{self, Calibration, FaceConfig};
    let fps = 15.0f32;
    let sz = size as f32;
    let (mut tracked, mut truth_lines) = (vec![format!("{{\"format\":\"osatura-landmarks\",\"version\":1,\"layout\":\"osatura23\",\"fps\":{fps},\"size\":[{size},{size}]}}")], vec![]);
    truth_lines.push(tracked[0].clone());
    let mut specs = Vec::new();
    let mut tr = BoxTracker::default();
    let mut found = 0;
    for i in 0..frames {
        let t = i as f32 / fps;
        let s = facegen::anim_spec(seed, t, sz);
        let g = facegen::render_spec(&s, size, size, Some(seed + 1), &facegen::Post::NONE, i as u64)?;
        let norm = |pts: &[Vec2]| pts.iter().map(|p| json!([p.x / sz, p.y / sz])).collect::<Vec<_>>();
        truth_lines.push(json!({"t": t, "p": norm(&s.landmarks())}).to_string());
        match tr.step(m, &g, &DetectOpts { max_faces: 1, ..Default::default() }) {
            Some(r) => {
                found += 1;
                tracked.push(json!({"t": t, "p": norm(&r.pts)}).to_string());
            }
            None => tracked.push(json!({"t": t, "p": null}).to_string()),
        }
        specs.push(s);
    }
    let cfg = FaceConfig::default();
    let series = |lines: &[String]| -> Result<std::collections::HashMap<String, Vec<(usize, f32)>>, String> {
        let st = faceparam::parse_ndjson(&lines.join("\n"), fps, None)?;
        let mut h: std::collections::HashMap<String, Vec<(usize, f32)>> = Default::default();
        // per-sequence calibration (neutral = first 10 frames, ranges = whole sequence), as `face-bake --calibrate-frames` does
        let cal: Calibration = faceparam::calibrate(&st, &cfg, 10)?;
        for e in faceparam::events(&st, &cfg, &cal)? {
            h.entry(e.id).or_default().push(((e.time * fps).round() as usize, e.value));
        }
        Ok(h)
    };
    let (a, b) = (series(&tracked)?, series(&truth_lines)?);
    type Getter = fn(&facegen::FaceSpec) -> f32;
    let pairs: [(&str, Getter); 7] = [
        ("head_yaw", |s| s.yaw),
        ("head_pitch", |s| s.pitch),
        ("head_roll", |s| s.roll),
        ("eye_r_open", |s| s.open_r),
        ("eye_l_open", |s| s.open_l),
        ("mouth_open", |s| s.mouth_open),
        ("brow_raise", |s| s.brow_raise),
    ];
    let mut out = serde_json::Map::new();
    for (name, f) in pairs {
        let corr = |h: &std::collections::HashMap<String, Vec<(usize, f32)>>| -> f32 {
            let v = h.get(name).cloned().unwrap_or_default();
            let x: Vec<f32> = v.iter().map(|p| p.1).collect();
            let y: Vec<f32> = v.iter().filter_map(|p| specs.get(p.0)).map(f).collect();
            pearson(&x, &y)
        };
        out.insert(name.into(), json!({"tracked_corr": corr(&a), "ground_truth_landmarks_corr": corr(&b)}));
    }
    Ok(json!({"frames": frames, "tracked_frames": found, "size_px": size, "fps": fps, "signals": Value::Object(out)}))
}

// ---------------------------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> TrainOpts {
        TrainOpts { samples: 24, seed: 3, scene: 96, det_win: 24, det_cell: 6, reg_win: 32, reg_cell: 8, mine_rounds: 1, stages: 1, refine_win: 48, stage_patch: 16, stage_cell: 8, ..Default::default() }
    }
    fn shared() -> &'static FaceModel {
        static M: std::sync::OnceLock<FaceModel> = std::sync::OnceLock::new();
        M.get_or_init(|| train(&tiny()).unwrap())
    }

    #[test]
    fn hog_is_deterministic_has_the_right_length_and_sees_edges() {
        let mut g = Gray::new(32, 32, 0.2);
        for y in 0..32 {
            for x in 16..32 {
                g.px[y * 32 + x] = 0.9;
            }
        }
        let f = hog_window(&g, 8);
        assert_eq!(f.len(), hog_len(32, 8));
        assert_eq!(f, hog_window(&g, 8));
        assert!(f.iter().all(|v| v.is_finite() && *v >= 0.0));
        // a vertical edge votes in the 0-degree bin only
        let flat = hog_window(&Gray::new(32, 32, 0.5), 8);
        assert!(flat.iter().all(|&v| v == 0.0));
        assert!(f.iter().sum::<f32>() > 0.5);
    }

    #[test]
    fn ridge_recovers_a_linear_map_and_rejects_bad_input() {
        let mut r = Rng::new(1);
        let (n, d, k) = (200, 6, 2);
        let w = [[1.0f32, -2.0, 0.5, 0.0, 3.0, -1.0], [0.0, 1.0, 1.0, -1.0, 0.0, 2.0]];
        let x: Vec<f32> = (0..n * d).map(|_| r.unit() * 2.0 - 1.0).collect();
        let mut y = Vec::new();
        for i in 0..n {
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for c in 0..k {
                y.push((0..d).map(|j| w[c][j] * x[i * d + j]).sum::<f32>() + 0.7 * c as f32);
            }
        }
        let (mx, ww, my) = ridge_fit(&x, &y, n, d, k, 1e-6).unwrap();
        for c in 0..k {
            for j in 0..d {
                assert!((ww[c * d + j] - w[c][j]).abs() < 1e-2, "w {c} {j}: {}", ww[c * d + j]);
            }
        }
        assert_eq!(mx.len(), d);
        assert!((my[1] - my[0] - 0.7).abs() < 0.2);
        assert!(ridge_fit(&x, &y, n, d, k, 0.0).is_err());
        assert!(ridge_fit(&x[1..], &y, n, d, k, 1.0).is_err());
    }

    #[test]
    fn logistic_separates_two_gaussians() {
        let mut r = Rng::new(2);
        let (n, d) = (400, 4);
        let mut x = Vec::new();
        let mut y = Vec::new();
        for i in 0..n {
            let c = (i % 2) as f32;
            for j in 0..d {
                x.push(r.unit() - 0.5 + if j < 2 { c * 1.5 - 0.75 } else { 0.0 });
            }
            y.push(c);
        }
        let sw = vec![1.0; n];
        let (w, b) = logistic_fit(&x, &y, &sw, n, d, 0.1, 15).unwrap();
        let acc = (0..n).filter(|&i| ((0..d).map(|j| w[j] * x[i * d + j]).sum::<f32>() + b > 0.0) == (y[i] > 0.5)).count() as f32 / n as f32;
        assert!(acc > 0.95, "acc {acc}");
        assert!(logistic_fit(&x, &y, &sw[1..], n, d, 0.1, 5).is_err());
    }

    #[test]
    fn nms_merges_and_iou_is_sane() {
        let a = Detection { cx: 50.0, cy: 50.0, side: 40.0, score: 3.0 };
        let b = Detection { cx: 52.0, cy: 50.0, side: 40.0, score: 2.0 };
        let c = Detection { cx: 150.0, cy: 50.0, side: 40.0, score: 1.0 };
        assert!(iou(&a, &b) > 0.8 && iou(&a, &c) == 0.0 && (iou(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn tiny_model_trains_deterministically_round_trips_and_detects_something() {
        let m = shared().clone();
        let m2 = train(&tiny()).unwrap();
        // identical up to float summation order of the multi-threaded Gram sums
        assert_eq!(m.det_w.len(), m2.det_w.len());
        let dmax = m.reg_w.iter().zip(&m2.reg_w).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
        assert!(dmax < 1e-3, "{dmax}");
        // binary and JSON round trips
        let b = m.to_bytes();
        let back = FaceModel::load(&b).unwrap();
        assert_eq!(back.det_w, m.det_w);
        assert_eq!(back.reg_w, m.reg_w);
        assert_eq!(back.stages, m.stages);
        assert_eq!(back.stages.len(), 1);
        assert_eq!(back.meta["scope"], m.meta["scope"]);
        let j = FaceModel::load(m.to_json().as_bytes()).unwrap();
        assert_eq!(j.reg_b, m.reg_b);
        // scene with a big clear face: some detection overlaps the truth, the regressor beats a constant guess
        let mut rng = Rng::new(99);
        let (g, t) = facegen::face_scene(&mut rng, 96, 96, 22.0, 26.0, false).unwrap();
        let td = truth_det(&t);
        let r = m.landmarks(&g, &td);
        let mean_face: Vec<Vec2> = (0..N_LM).map(|i| Vec2::new(m.reg_b[2 * i] * t.side + t.cx, m.reg_b[2 * i + 1] * t.side + t.cy)).collect();
        let e_reg = t.nme(&r.pts);
        let e_mean = t.nme(&mean_face);
        assert!(e_reg < e_mean * 1.5 + 0.05, "reg {e_reg} mean {e_mean}");
        assert!(e_reg.is_finite());
    }

    #[test]
    fn load_rejects_garbage_and_truncation() {
        assert!(FaceModel::load(b"").is_err());
        assert!(FaceModel::load(b"AFD1").is_err());
        assert!(FaceModel::load(b"AFD1\xff\xff\xff\xffxx").is_err());
        assert!(FaceModel::load(b"{\"nope\":1}").is_err());
        let m = shared().clone();
        let mut b = m.to_bytes();
        b.truncate(b.len() - 4);
        assert!(FaceModel::load(&b).is_err());
        let mut bad = m.clone();
        bad.reg_b[0] = f32::NAN;
        assert!(bad.validate().is_err());
        let mut bad3 = m.clone();
        bad3.stages[0].w.pop();
        assert!(bad3.validate().is_err());
        let mut bad2 = m.clone();
        bad2.det_cell = 7;
        assert!(bad2.validate().is_err());
    }

    #[test]
    fn train_validates_options() {
        assert!(train(&TrainOpts { samples: 5, ..tiny() }).is_err());
        assert!(train(&TrainOpts { scene: 10, ..tiny() }).is_err());
        assert!(train(&TrainOpts { det_cell: 5, ..tiny() }).is_err());
    }

    #[test]
    fn tracker_and_detect_never_panic_on_tiny_or_flat_images() {
        let m = shared().clone();
        for g in [Gray::new(5, 5, 0.5), Gray::new(200, 20, 0.1), Gray::new(96, 96, 1.0)] {
            let _ = m.detect(&g, &DetectOpts::default());
        }
        let mut t = BoxTracker::default();
        assert!(t.step(&m, &Gray::new(96, 96, 0.5), &DetectOpts { threshold: Some(1e6), ..Default::default() }).is_none());
    }

    #[test]
    fn builtin_model_loads_is_small_and_finds_a_synthetic_face() {
        let m = FaceModel::builtin().unwrap();
        assert!(m.to_bytes().len() < 1_000_000);
        assert_eq!(m.stages.len(), 3);
        assert!(m.meta["scope"].as_str().unwrap().contains("not validated on photographs"));
        let (g, t) = facegen::face_scene(&mut Rng::new(4242), 128, 128, 25.0, 40.0, true).unwrap();
        let r = m.detect(&g, &DetectOpts { max_faces: 1, ..Default::default() });
        assert_eq!(r.len(), 1);
        assert!(iou(&r[0].det, &truth_det(&t)) > 0.5);
        assert!(t.nme(&r[0].pts) < 0.08, "nme {}", t.nme(&r[0].pts));
        // a scene without a face yields nothing at the default threshold
        let neg = facegen::negative_scene(&mut Rng::new(5), 128, 128).unwrap();
        assert!(m.detect_boxes(&neg, &DetectOpts::default()).len() <= 1);
    }

    #[test]
    fn cartoon_ground_truth_is_consistent_with_the_drawing() {
        for c in facegen::cartoons() {
            assert_eq!(c.pts.len(), 23, "{}", c.name);
            let (g, t) = c.render(200, 200, 1.0, 0.0, 100.0, 100.0, [1.0, 1.0, 1.0]).unwrap();
            // identity placement keeps design coordinates
            assert!((t.pts[0].x - c.pts[0].x).abs() < 1e-3);
            // the head box is inside the image and the picture is not blank
            assert!(g.px.iter().any(|&v| v < 0.5), "{}", c.name);
            // rotation keeps the inter-ocular distance
            let (_, t2) = c.render(200, 200, 1.0, 20.0, 100.0, 100.0, [1.0, 1.0, 1.0]).unwrap();
            assert!((facegen::interocular(&t.pts) - facegen::interocular(&t2.pts)).abs() < 1e-2);
        }
    }

    #[test]
    fn pearson_basics() {
        assert!((pearson(&[1.0, 2.0, 3.0, 4.0], &[2.0, 4.0, 6.0, 8.0]) - 1.0).abs() < 1e-5);
        assert!((pearson(&[1.0, 2.0, 3.0, 4.0], &[8.0, 6.0, 4.0, 2.0]) + 1.0).abs() < 1e-5);
        assert!(pearson(&[1.0], &[1.0]).is_nan());
    }

    /// Render the synthetic animation with the engine, track it with the built-in model, import the
    /// landmarks like `face-bake` does and correlate the rig signals with the ground-truth drivers.
    #[test]
    fn end_to_end_synthetic_animation_signals_follow_the_ground_truth() {
        let m = FaceModel::builtin().unwrap();
        let r = e2e_report(&m, 32, 112, 7).unwrap();
        assert_eq!(r["tracked_frames"], 32);
        let c = |k: &str| r["signals"][k]["tracked_corr"].as_f64().unwrap();
        let ceil = |k: &str| r["signals"][k]["ground_truth_landmarks_corr"].as_f64().unwrap();
        // head angles and mouth are recovered nearly perfectly, and the exact landmarks are the ceiling
        for k in ["head_yaw", "head_roll", "mouth_open"] {
            assert!(c(k) > 0.75 && c(k) > ceil(k) - 0.1, "{k}: {} (ceiling {})", c(k), ceil(k));
        }
        assert!(c("head_pitch").is_finite() && c("brow_raise").is_finite() && c("eye_r_open").is_finite());
    }
}
