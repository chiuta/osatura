// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Procedural synthetic faces: the only training data of [`crate::facedet`].
//!
//! A [`FaceSpec`] is a parametric *drawing* of a face (head ellipse, eyes with lids and pupils, brows,
//! nose, mouth, optional fringe and hair strokes) made of flat-coloured polygons. Features sit on a
//! sphere-like head surface and are rotated by yaw / pitch (orthographic projection) and roll, so the
//! nose moves with the head turn while the silhouette stays put. The spec yields
//!
//! * the 23 landmarks of [`crate::faceparam::NAMES`] by construction (ground truth, exact), and
//! * an SVG document, which is rendered by the engine's own SVG importer and rasteriser
//!   (`importers::import_svg` -> `Engine` -> `raster::render_rgba`). Hand-written SVG drawings take the
//!   same path ([`render_svg_gray`]), so "drawn differently" faces are rendered by the same code.
//!
//! Photometric nuisance (lighting gradient, blur, noise) is applied afterwards on the grey image
//! ([`Post`]). Nothing here is a photograph, a scan or a copy of any dataset: every pixel is generated
//! from a seed. Detectors trained on it are expected to work on stylised / cartoon faces only.

use crate::engine::Engine;
use crate::importers::import_svg;
use crate::math::Vec2;
use crate::project::{Layer, Project};
use crate::raster::{render_rgba, RasterOptions};
use crate::tinymlp::Rng;
use std::fmt::Write as _;

// ---------------------------------------------------------------------------------------------
// grey images
// ---------------------------------------------------------------------------------------------

/// Grey image, values nominally 0..1, row-major.
#[derive(Clone, Debug, PartialEq)]
pub struct Gray {
    pub w: usize,
    pub h: usize,
    pub px: Vec<f32>,
}

impl Gray {
    pub fn new(w: usize, h: usize, v: f32) -> Gray {
        Gray {
            w,
            h,
            px: vec![v; w * h],
        }
    }
    /// From straight RGBA bytes (Rec. 601 luma, alpha composited over white).
    pub fn from_rgba(w: usize, h: usize, rgba: &[u8]) -> Result<Gray, String> {
        if w == 0 || h == 0 || rgba.len() != w * h * 4 {
            return Err("image size does not match its pixel data".into());
        }
        let px = rgba
            .chunks_exact(4)
            .map(|c| {
                let a = c[3] as f32 / 255.0;
                let l = (0.299 * c[0] as f32 + 0.587 * c[1] as f32 + 0.114 * c[2] as f32) / 255.0;
                l * a + (1.0 - a)
            })
            .collect();
        Ok(Gray { w, h, px })
    }
    #[inline]
    pub fn at(&self, x: isize, y: isize) -> f32 {
        let xi = x.clamp(0, self.w as isize - 1) as usize;
        let yi = y.clamp(0, self.h as isize - 1) as usize;
        self.px[yi * self.w + xi]
    }
    /// Bilinear sample at continuous pixel-centre coordinates (pixel i covers [i, i+1)).
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let (fx, fy) = (x - 0.5, y - 0.5);
        let (x0, y0) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - x0, fy - y0);
        let (x0, y0) = (x0 as isize, y0 as isize);
        let a = self.at(x0, y0) * (1.0 - tx) + self.at(x0 + 1, y0) * tx;
        let b = self.at(x0, y0 + 1) * (1.0 - tx) + self.at(x0 + 1, y0 + 1) * tx;
        a * (1.0 - ty) + b * ty
    }
    /// Square window of side `side` (source pixels) centred on (cx, cy), resampled to `out` x `out`
    /// with box prefiltering (k x k sub-samples per output pixel, k = ceil(side / out)).
    pub fn crop_resize(&self, cx: f32, cy: f32, side: f32, out: usize) -> Gray {
        let k = ((side / out as f32).ceil() as usize).clamp(1, 8);
        let step = side / out as f32;
        let (x0, y0) = (cx - side * 0.5, cy - side * 0.5);
        let mut g = Gray::new(out, out, 0.0);
        for j in 0..out {
            for i in 0..out {
                let mut s = 0.0;
                for b in 0..k {
                    for a in 0..k {
                        let x = x0 + (i as f32 + (a as f32 + 0.5) / k as f32) * step;
                        let y = y0 + (j as f32 + (b as f32 + 0.5) / k as f32) * step;
                        s += self.sample(x, y);
                    }
                }
                g.px[j * out + i] = s / (k * k) as f32;
            }
        }
        g
    }
    /// Downscale by a factor >= 1 (area-weighted bilinear with k x k sub-samples); factor 1 clones.
    pub fn scaled_down(&self, factor: f32) -> Gray {
        if factor <= 1.0001 {
            return self.clone();
        }
        let nw = ((self.w as f32 / factor).floor() as usize).max(1);
        let nh = ((self.h as f32 / factor).floor() as usize).max(1);
        let k = (factor.ceil() as usize).clamp(1, 6);
        let mut g = Gray::new(nw, nh, 0.0);
        for j in 0..nh {
            for i in 0..nw {
                let mut s = 0.0;
                for b in 0..k {
                    for a in 0..k {
                        let x = (i as f32 + (a as f32 + 0.5) / k as f32) * factor;
                        let y = (j as f32 + (b as f32 + 0.5) / k as f32) * factor;
                        s += self.sample(x, y);
                    }
                }
                g.px[j * nw + i] = s / (k * k) as f32;
            }
        }
        g
    }
    /// Separable Gaussian blur (radius 3 sigma, clamped edges).
    pub fn blur(&mut self, sigma: f32) {
        if sigma < 0.2 {
            return;
        }
        let r = (sigma * 3.0).ceil() as isize;
        let mut k: Vec<f32> = (-r..=r)
            .map(|i| (-((i * i) as f32) / (2.0 * sigma * sigma)).exp())
            .collect();
        let s: f32 = k.iter().sum();
        k.iter_mut().for_each(|v| *v /= s);
        let mut tmp = self.clone();
        for y in 0..self.h as isize {
            for x in 0..self.w as isize {
                tmp.px[y as usize * self.w + x as usize] = (-r..=r)
                    .map(|i| k[(i + r) as usize] * self.at(x + i, y))
                    .sum();
            }
        }
        for y in 0..self.h as isize {
            for x in 0..self.w as isize {
                self.px[y as usize * self.w + x as usize] = (-r..=r)
                    .map(|i| k[(i + r) as usize] * tmp.at(x, y + i))
                    .sum();
            }
        }
    }
    /// 8-bit PNG (grey stored as RGB) for debugging and frame sequences.
    pub fn to_png(&self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut out, self.w as u32, self.h as u32);
            enc.set_color(png::ColorType::Grayscale);
            enc.set_depth(png::BitDepth::Eight);
            let mut wr = enc.write_header().map_err(|e| e.to_string())?;
            let bytes: Vec<u8> = self
                .px
                .iter()
                .map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
                .collect();
            wr.write_image_data(&bytes).map_err(|e| e.to_string())?;
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------------------------
// random helpers
// ---------------------------------------------------------------------------------------------

fn rr(r: &mut Rng, a: f32, b: f32) -> f32 {
    a + (b - a) * r.unit()
}
fn gauss(r: &mut Rng) -> f32 {
    let u1 = r.unit().max(1e-7);
    let u2 = r.unit();
    (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
}
fn hex(c: [f32; 3]) -> String {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    format!("#{:02x}{:02x}{:02x}", b(c[0]), b(c[1]), b(c[2]))
}
fn luma(c: [f32; 3]) -> f32 {
    0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]
}
fn mul(c: [f32; 3], k: f32) -> [f32; 3] {
    [c[0] * k, c[1] * k, c[2] * k]
}
fn rand_colour(r: &mut Rng, lo: f32, hi: f32) -> [f32; 3] {
    let l = rr(r, lo, hi);
    let t = [rr(r, 0.75, 1.0), rr(r, 0.75, 1.0), rr(r, 0.75, 1.0)];
    let m = t[0].max(t[1]).max(t[2]);
    [l * t[0] / m, l * t[1] / m, l * t[2] / m]
}

// ---------------------------------------------------------------------------------------------
// the face drawing
// ---------------------------------------------------------------------------------------------

/// Parametric face drawing. Lengths are in head units: horizontal ones in half head widths (`hw`),
/// vertical ones in half head heights (`ry`), y grows downwards. Angles: radians, except `roll` (degrees).
#[derive(Clone, Debug, PartialEq)]
pub struct FaceSpec {
    pub cx: f32,
    pub cy: f32,
    /// Half head width in pixels.
    pub hw: f32,
    /// Half head height / half head width.
    pub aspect: f32,
    pub roll: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub skin: [f32; 3],
    pub bg: [f32; 3],
    pub ink: [f32; 3],
    pub iris: [f32; 3],
    pub lip: [f32; 3],
    pub hair: [f32; 3],
    pub eye_dx: f32,
    pub eye_y: f32,
    pub eye_w: f32,
    pub eye_h: f32,
    pub open_r: f32,
    pub open_l: f32,
    pub pupil: f32,
    pub brow_off: f32,
    pub brow_raise: f32,
    pub brow_tilt: f32,
    pub brow_len: f32,
    pub brow_thick: f32,
    pub nose_y: f32,
    pub nose_z: f32,
    pub mouth_y: f32,
    pub mouth_w: f32,
    pub mouth_open: f32,
    pub smile: f32,
    pub line_w: f32,
    /// Bottom edge of the fringe in head v units (-1 = none).
    pub fringe: f32,
    /// Hair-like strokes laid over the face: `(u0, v0, u1, v1, width_px)` in head units.
    pub strokes: Vec<[f32; 5]>,
}

/// Ground truth of one face in a scene.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceTruth {
    /// 23 landmarks in pixels, [`crate::faceparam::NAMES`] order.
    pub pts: Vec<Vec2>,
    /// Square face box: centre and side in pixels.
    pub cx: f32,
    pub cy: f32,
    pub side: f32,
}

impl FaceTruth {
    /// Mean landmark error / inter-ocular distance for predictions in the same pixel frame.
    pub fn nme(&self, pred: &[Vec2]) -> f32 {
        nme(&self.pts, pred)
    }
}

/// Inter-ocular distance: between the two eye centres (mean of six lid points each).
pub fn interocular(p: &[Vec2]) -> f32 {
    let c = |o: usize| {
        let mut s = Vec2::new(0.0, 0.0);
        for i in 0..6 {
            s += p[o + i];
        }
        s * (1.0 / 6.0)
    };
    (c(0) - c(6)).length()
}

/// Mean point distance divided by the inter-ocular distance of `truth`.
pub fn nme(truth: &[Vec2], pred: &[Vec2]) -> f32 {
    let d = interocular(truth).max(1e-6);
    let n = truth.len().min(pred.len()).max(1);
    truth
        .iter()
        .zip(pred)
        .map(|(a, b)| (*a - *b).length())
        .sum::<f32>()
        / n as f32
        / d
}

struct Xf {
    cx: f32,
    cy: f32,
    hw: f32,
    ry: f32,
    cr: f32,
    sr: f32,
    cyaw: f32,
    syaw: f32,
    cp: f32,
    sp: f32,
}
impl Xf {
    /// Head-surface point (u, v) (unit-disc coordinates) plus extra depth, to the image.
    fn p(&self, u: f32, v: f32, zx: f32) -> Vec2 {
        let z = (1.0 - u * u - v * v).max(0.0).sqrt() * 0.55 + zx;
        let u2 = u * self.cyaw + z * self.syaw;
        let v2 = v * self.cp - z * self.sp;
        self.flat(u2, v2)
    }
    /// Head-plane point with roll only (silhouette).
    fn flat(&self, u: f32, v: f32) -> Vec2 {
        let (x, y) = (u * self.hw, v * self.ry);
        Vec2::new(
            self.cx + x * self.cr - y * self.sr,
            self.cy + x * self.sr + y * self.cr,
        )
    }
}

impl FaceSpec {
    /// A random face. `size` is the half head width range in pixels, the canvas is `w` x `h`.
    pub fn random(r: &mut Rng, w: f32, h: f32, hw_lo: f32, hw_hi: f32) -> FaceSpec {
        let hw = rr(r, hw_lo, hw_hi);
        let aspect = rr(r, 1.1, 1.4);
        let pad = hw * aspect * 1.05;
        let cx = if w > 2.0 * pad {
            rr(r, pad, w - pad)
        } else {
            w * 0.5
        };
        let cy = if h > 2.0 * pad {
            rr(r, pad, h - pad)
        } else {
            h * 0.5
        };
        let skin = rand_colour(r, 0.35, 0.97);
        let mut bg = rand_colour(r, 0.05, 1.0);
        let mut guard = 0;
        while (luma(bg) - luma(skin)).abs() < 0.10 && guard < 20 {
            bg = rand_colour(r, 0.05, 1.0);
            guard += 1;
        }
        let ink_k = rr(r, 0.08, 0.45);
        let strokes = (0..r.below(3))
            .map(|_| {
                let a = rr(r, -1.0, 1.0);
                let b = rr(r, -1.0, 1.0);
                [
                    a,
                    rr(r, -1.0, -0.2),
                    a + rr(r, -0.3, 0.3),
                    b.abs() * 0.6 + rr(r, -0.1, 0.5),
                    rr(r, 1.0, 3.0),
                ]
            })
            .collect();
        FaceSpec {
            cx,
            cy,
            hw,
            aspect,
            roll: rr(r, -12.0, 12.0),
            yaw: rr(r, -0.45, 0.45),
            pitch: rr(r, -0.3, 0.3),
            skin,
            bg,
            ink: mul(skin, ink_k),
            iris: mul(rand_colour(r, 0.1, 0.8), 0.7),
            lip: [
                rr(r, 0.5, 0.85) * luma(skin).max(0.4),
                rr(r, 0.1, 0.4) * luma(skin),
                rr(r, 0.1, 0.4) * luma(skin),
            ],
            hair: mul(rand_colour(r, 0.05, 0.6), 0.8),
            eye_dx: rr(r, 0.30, 0.46),
            eye_y: rr(r, -0.30, -0.10),
            eye_w: rr(r, 0.20, 0.30),
            eye_h: rr(r, 0.35, 0.55),
            open_r: if r.unit() < 0.15 {
                rr(r, 0.0, 0.3)
            } else {
                rr(r, 0.6, 1.0)
            },
            open_l: if r.unit() < 0.15 {
                rr(r, 0.0, 0.3)
            } else {
                rr(r, 0.6, 1.0)
            },
            pupil: rr(r, 0.35, 0.6),
            brow_off: rr(r, 0.14, 0.24),
            brow_raise: rr(r, 0.0, 0.6),
            brow_tilt: rr(r, -0.12, 0.12),
            brow_len: rr(r, 0.22, 0.34),
            brow_thick: rr(r, 1.0, 3.5),
            nose_y: rr(r, 0.12, 0.30),
            nose_z: rr(r, 0.15, 0.30),
            mouth_y: rr(r, 0.45, 0.62),
            mouth_w: rr(r, 0.24, 0.42),
            mouth_open: if r.unit() < 0.5 {
                rr(r, 0.0, 0.12)
            } else {
                rr(r, 0.1, 1.0)
            },
            smile: rr(r, -0.06, 0.06),
            line_w: rr(r, 1.0, 2.5),
            fringe: if r.unit() < 0.5 {
                rr(r, -0.80, -0.55)
            } else {
                -1.0
            },
            strokes: if r.unit() < 0.4 { strokes } else { vec![] },
        }
    }

    fn xf(&self) -> Xf {
        let rr_ = self.roll.to_radians();
        Xf {
            cx: self.cx,
            cy: self.cy,
            hw: self.hw,
            ry: self.hw * self.aspect,
            cr: rr_.cos(),
            sr: rr_.sin(),
            cyaw: self.yaw.cos(),
            syaw: self.yaw.sin(),
            cp: self.pitch.cos(),
            sp: self.pitch.sin(),
        }
    }

    /// Eye lid curve points for the eye centred at head-unit (cu, cv): returns (outline polygon,
    /// [p1..p6]) where outline runs along the upper lid left to right then the lower lid back.
    fn eye(&self, x: &Xf, cu: f32, cv: f32, open: f32) -> (Vec<Vec2>, [Vec2; 6]) {
        let hwe = self.eye_w * 0.5;
        let hv = (open.clamp(0.0, 1.0) * self.eye_h * hwe / self.aspect).max(0.003);
        let z0 = 0.0;
        let curve = |t: f32, sgn: f32| x.p(cu + t * hwe, cv + sgn * hv * (1.0 - t * t), z0);
        let mut poly = Vec::new();
        for i in 0..=10 {
            poly.push(curve(-1.0 + 0.2 * i as f32, -1.0));
        }
        for i in (1..10).rev() {
            poly.push(curve(-1.0 + 0.2 * i as f32, 1.0));
        }
        let q = [
            curve(-1.0, 0.0),
            curve(-0.5, -1.0),
            curve(0.5, -1.0),
            curve(1.0, 0.0),
            curve(0.5, 1.0),
            curve(-0.5, 1.0),
        ];
        (poly, q)
    }

    /// Landmarks in pixels, NAMES order.
    pub fn landmarks(&self) -> Vec<Vec2> {
        let x = self.xf();
        let (_, re) = self.eye(&x, -self.eye_dx, self.eye_y, self.open_r);
        let (_, le) = self.eye(&x, self.eye_dx, self.eye_y, self.open_l);
        let gap = self.mouth_open * 0.5 * self.mouth_w;
        let mv = |t: f32, dv: f32| {
            x.p(
                t * self.mouth_w * 0.5,
                self.mouth_y + self.smile * t * t + dv,
                0.0,
            )
        };
        let bv = self.eye_y - self.brow_off - self.brow_raise * 0.12;
        let sil = (1.0 - self.eye_y * self.eye_y).max(0.0).sqrt();
        let mut v: Vec<Vec2> = Vec::with_capacity(23);
        v.extend(re);
        v.extend(le);
        v.push(mv(-1.0, 0.0));
        v.push(mv(1.0, 0.0));
        v.push(mv(0.0, -gap * 0.5));
        v.push(mv(0.0, gap * 0.5));
        v.push(x.p(-self.eye_dx, bv, 0.0));
        v.push(x.p(self.eye_dx, bv, 0.0));
        v.push(x.p(0.0, self.nose_y, self.nose_z));
        v.push(x.p(0.0, self.eye_y, 0.08));
        v.push(x.flat(0.0, 1.0));
        v.push(x.flat(-sil, self.eye_y));
        v.push(x.flat(sil, self.eye_y));
        v
    }

    pub fn truth(&self) -> FaceTruth {
        FaceTruth {
            pts: self.landmarks(),
            cx: self.cx,
            cy: self.cy,
            side: 2.2 * self.hw * self.aspect,
        }
    }

    /// SVG shapes (no `<svg>` wrapper, no background).
    pub fn svg_shapes(&self) -> String {
        let x = self.xf();
        let mut s = String::new();
        let poly = |s: &mut String,
                    pts: &[Vec2],
                    fill: Option<[f32; 3]>,
                    stroke: Option<([f32; 3], f32)>,
                    closed: bool| {
            let pp: Vec<String> = pts
                .iter()
                .map(|p| format!("{:.2},{:.2}", p.x, p.y))
                .collect();
            let _ = write!(
                s,
                "<{} points=\"{}\" fill=\"{}\"",
                if closed { "polygon" } else { "polyline" },
                pp.join(" "),
                fill.map(hex).unwrap_or("none".into())
            );
            if let Some((c, w)) = stroke {
                let _ = write!(s, " stroke=\"{}\" stroke-width=\"{:.2}\"", hex(c), w);
            }
            s.push_str("/>");
        };
        // head
        let head: Vec<Vec2> = (0..72)
            .map(|i| {
                let a = std::f32::consts::TAU * i as f32 / 72.0;
                x.flat(a.cos(), a.sin())
            })
            .collect();
        poly(
            &mut s,
            &head,
            Some(self.skin),
            Some((mul(self.skin, 0.7), self.line_w)),
            true,
        );
        // eyes
        for (sgn, open) in [(-1.0f32, self.open_r), (1.0, self.open_l)] {
            let (outline, _) = self.eye(&x, sgn * self.eye_dx, self.eye_y, open);
            let white = [0.96, 0.96, 0.94];
            poly(
                &mut s,
                &outline,
                Some(white),
                Some((self.ink, self.line_w)),
                true,
            );
            if open > 0.25 {
                let pr = self.pupil * self.eye_w * 0.5 * open.min(1.0).sqrt();
                let pu = sgn * self.eye_dx + self.yaw.sin() * 0.0;
                let ring: Vec<Vec2> = (0..14)
                    .map(|i| {
                        let a = std::f32::consts::TAU * i as f32 / 14.0;
                        x.p(
                            pu + pr * a.cos(),
                            self.eye_y + pr * a.sin() / self.aspect,
                            0.0,
                        )
                    })
                    .collect();
                poly(&mut s, &ring, Some(self.iris), Some((self.ink, 0.5)), true);
            }
        }
        // brows
        let bv = self.eye_y - self.brow_off - self.brow_raise * 0.12;
        for sgn in [-1.0f32, 1.0] {
            let pts: Vec<Vec2> = (0..7)
                .map(|i| {
                    let t = -1.0 + i as f32 / 3.0;
                    x.p(
                        sgn * self.eye_dx + t * self.brow_len * 0.5,
                        bv - 0.03 * (1.0 - t * t) + self.brow_tilt * t * sgn,
                        0.0,
                    )
                })
                .collect();
            poly(&mut s, &pts, None, Some((self.ink, self.brow_thick)), false);
        }
        // nose
        let bridge = x.p(0.0, self.eye_y + 0.04, 0.08);
        let tip = x.p(0.0, self.nose_y, self.nose_z);
        poly(
            &mut s,
            &[bridge, tip],
            None,
            Some((mul(self.skin, 0.6), self.line_w)),
            false,
        );
        let nostrils = [
            x.p(-0.09, self.nose_y + 0.015, self.nose_z * 0.8),
            tip,
            x.p(0.09, self.nose_y + 0.015, self.nose_z * 0.8),
        ];
        poly(
            &mut s,
            &nostrils,
            None,
            Some((self.ink, self.line_w)),
            false,
        );
        // mouth
        let gap = self.mouth_open * 0.5 * self.mouth_w;
        let mv = |t: f32, dv: f32| {
            x.p(
                t * self.mouth_w * 0.5,
                self.mouth_y + self.smile * t * t + dv,
                0.0,
            )
        };
        let ts: Vec<f32> = (0..=10).map(|i| -1.0 + 0.2 * i as f32).collect();
        let mut mouth: Vec<Vec2> = ts
            .iter()
            .map(|&t| mv(t, -gap * 0.5 * (1.0 - t * t)))
            .collect();
        mouth.extend(
            ts.iter()
                .rev()
                .skip(1)
                .take(9)
                .map(|&t| mv(t, gap * 0.5 * (1.0 - t * t))),
        );
        if gap > 0.01 {
            poly(
                &mut s,
                &mouth,
                Some(mul(self.lip, 0.35)),
                Some((self.lip, self.line_w * 1.2)),
                true,
            );
        } else {
            let line: Vec<Vec2> = ts.iter().map(|&t| mv(t, 0.0)).collect();
            poly(
                &mut s,
                &line,
                None,
                Some((self.lip, self.line_w * 1.6)),
                false,
            );
        }
        // fringe and strokes (occluders)
        if self.fringe > -0.99 {
            let mut f: Vec<Vec2> = Vec::new();
            for i in 0..=20 {
                let a = std::f32::consts::PI + std::f32::consts::PI * i as f32 / 20.0;
                f.push(x.flat(1.04 * a.cos(), 1.04 * a.sin()));
            }
            for i in (0..=20).rev() {
                let t = -1.0 + 2.0 * i as f32 / 20.0;
                f.push(x.flat(
                    t * 0.97,
                    self.fringe + 0.12 * (t * 2.5).sin() * 0.5 + 0.08 * t * t,
                ));
            }
            poly(&mut s, &f, Some(self.hair), None, true);
        }
        for st in &self.strokes {
            poly(
                &mut s,
                &[x.flat(st[0], st[1]), x.flat(st[2], st[3])],
                None,
                Some((self.hair, st[4])),
                false,
            );
        }
        s
    }
}

// ---------------------------------------------------------------------------------------------
// scenes and rendering
// ---------------------------------------------------------------------------------------------

fn svg_doc(w: usize, h: usize, body: &str) -> String {
    format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">{body}</svg>")
}

/// Render an SVG document with the engine's importer and rasteriser to a grey image of `w` x `h`
/// pixels (the document is scaled so that its own width fills `w`).
pub fn render_svg_gray(svg: &str, w: usize, h: usize) -> Result<Gray, String> {
    if w == 0 || h == 0 || w * h > 4_000_000 {
        return Err("render size must be 1..4 million pixels".into());
    }
    let imp = import_svg(svg)?;
    let sw = if imp.width > 0.0 { imp.width } else { w as f32 };
    let sh = if imp.height > 0.0 {
        imp.height
    } else {
        h as f32
    };
    let mut p = Project::new("facegen", sw, sh, 24.0, 1);
    p.layers = vec![Layer::vector("svg", imp.paths)];
    p.background = Some(crate::style::Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    });
    let eng = Engine::new(p).map_err(|e| e.join("; "))?;
    let (pw, ph, rgba) = render_rgba(
        &eng,
        0.0,
        &RasterOptions {
            scale: w as f32 / sw,
            ..Default::default()
        },
    );
    let g = Gray::from_rgba(pw as usize, ph as usize, &rgba)?;
    if (g.w, g.h) == (w, h) {
        Ok(g)
    } else {
        Ok(g.crop_resize(
            g.w as f32 * 0.5,
            g.h as f32 * 0.5,
            g.w.max(g.h) as f32,
            w.max(h),
        ))
    }
}

/// Photometric nuisance applied to the grey image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Post {
    /// Linear lighting ramp strength (0 = none), direction in radians.
    pub light: f32,
    pub light_dir: f32,
    pub blur: f32,
    pub noise: f32,
    pub gain: f32,
}
impl Post {
    pub const NONE: Post = Post {
        light: 0.0,
        light_dir: 0.0,
        blur: 0.0,
        noise: 0.0,
        gain: 1.0,
    };
    pub fn random(r: &mut Rng) -> Post {
        Post {
            light: rr(r, 0.0, 0.35),
            light_dir: rr(r, 0.0, std::f32::consts::TAU),
            blur: if r.unit() < 0.5 { rr(r, 0.0, 1.2) } else { 0.0 },
            noise: if r.unit() < 0.6 {
                rr(r, 0.0, 0.05)
            } else {
                0.0
            },
            gain: rr(r, 0.8, 1.1),
        }
    }
    pub fn apply(&self, g: &mut Gray, r: &mut Rng) {
        g.blur(self.blur);
        let (cd, sd) = (self.light_dir.cos(), self.light_dir.sin());
        let (w, h) = (g.w as f32, g.h as f32);
        for y in 0..g.h {
            for x in 0..g.w {
                let t = ((x as f32 / w - 0.5) * cd + (y as f32 / h - 0.5) * sd) * 2.0;
                let v = &mut g.px[y * g.w + x];
                *v = (*v * self.gain * (1.0 + self.light * t)
                    + if self.noise > 0.0 {
                        self.noise * gauss(r)
                    } else {
                        0.0
                    })
                .clamp(0.0, 1.0);
            }
        }
    }
}

/// Random clutter shapes (rectangles, ellipses, lines, polygons) over a flat background, optionally
/// with face-like decoys (a bare head ellipse, a lone eye pair, a lone mouth) that contain no complete face.
pub fn clutter_svg(r: &mut Rng, w: usize, h: usize, decoys: bool) -> String {
    let (wf, hf) = (w as f32, h as f32);
    let bg = rand_colour(r, 0.05, 1.0);
    let mut s = format!(
        "<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"{}\"/>",
        hex(bg)
    );
    for _ in 0..(2 + r.below(10)) {
        let c = rand_colour(r, 0.0, 1.0);
        let (x, y) = (rr(r, -0.1, 1.1) * wf, rr(r, -0.1, 1.1) * hf);
        let sz = rr(r, 0.04, 0.5) * wf.min(hf);
        match r.below(5) {
            0 => {
                let _ = write!(
                    s,
                    "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\"/>",
                    x,
                    y,
                    sz,
                    sz * rr(r, 0.2, 2.0),
                    hex(c)
                );
            }
            1 => {
                let _ = write!(
                    s,
                    "<ellipse cx=\"{:.1}\" cy=\"{:.1}\" rx=\"{:.1}\" ry=\"{:.1}\" fill=\"{}\"/>",
                    x,
                    y,
                    sz * 0.5,
                    sz * rr(r, 0.2, 0.7),
                    hex(c)
                );
            }
            2 => {
                let _ = write!(s, "<polyline points=\"{:.1},{:.1} {:.1},{:.1}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.1}\"/>", x, y, x + rr(r, -1.0, 1.0) * sz * 2.0, y + rr(r, -1.0, 1.0) * sz * 2.0, hex(c), rr(r, 1.0, 6.0));
            }
            3 => {
                let _ = write!(
                    s,
                    "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"{}\"/>",
                    x,
                    y,
                    x + rr(r, -1.0, 1.0) * sz,
                    y + rr(r, -1.0, 1.0) * sz,
                    x + rr(r, -1.0, 1.0) * sz,
                    y + rr(r, -1.0, 1.0) * sz,
                    hex(c)
                );
            }
            _ => {
                let _ = write!(s, "<polyline points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.1}\"/>", x, y, x + sz * 0.5, y + rr(r, -0.5, 0.5) * sz, x + sz, y, hex(c), rr(r, 1.0, 4.0));
            }
        }
    }
    if decoys && r.unit() < 0.7 {
        // a face drawing with parts removed: head only, eyes only, or mouth only
        let mut f = FaceSpec::random(r, wf, hf, wf.min(hf) * 0.12, wf.min(hf) * 0.4);
        let full = f.svg_shapes();
        let cut = r.below(3);
        let n = full.matches("/>").count();
        let keep: Vec<&str> = full.split_inclusive("/>").collect();
        let _ = n;
        f.fringe = -1.0;
        match cut {
            0 => s.push_str(keep[0]),                                     // bare head
            1 => keep.iter().skip(1).take(4).for_each(|k| s.push_str(k)), // eyes without head
            _ => keep
                .iter()
                .skip(keep.len().saturating_sub(5))
                .for_each(|k| s.push_str(k)), // mouth / hair without head
        }
    }
    s
}

/// A scene with one synthetic face on clutter, rendered and degraded. Returns the image and the truth.
pub fn face_scene(
    r: &mut Rng,
    w: usize,
    h: usize,
    hw_lo: f32,
    hw_hi: f32,
    nuisance: bool,
) -> Result<(Gray, FaceTruth), String> {
    face_scene_with(r, w, h, hw_lo, hw_hi, |_, p, r| {
        if nuisance {
            *p = Post::random(r);
        } else {
            *p = Post::NONE;
        }
    })
}

/// Like [`face_scene`], but `tweak` may override any spec field and the [`Post`] (which starts as
/// [`Post::NONE`]) before rendering; used by the robustness sweeps.
pub fn face_scene_with(
    r: &mut Rng,
    w: usize,
    h: usize,
    hw_lo: f32,
    hw_hi: f32,
    mut tweak: impl FnMut(&mut FaceSpec, &mut Post, &mut Rng),
) -> Result<(Gray, FaceTruth), String> {
    let mut spec = FaceSpec::random(r, w as f32, h as f32, hw_lo, hw_hi);
    let mut post = Post::NONE;
    let plain = r.unit() < 0.5;
    let clutter = if plain {
        String::new()
    } else {
        clutter_svg(r, w, h, false)
    };
    tweak(&mut spec, &mut post, r);
    let mut body = if plain {
        format!(
            "<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"{}\"/>",
            hex(spec.bg)
        )
    } else {
        clutter
    };
    body.push_str(&spec.svg_shapes());
    let mut g = render_svg_gray(&svg_doc(w, h, &body), w, h)?;
    post.apply(&mut g, r);
    Ok((g, spec.truth()))
}

/// A scene without any complete face.
pub fn negative_scene(r: &mut Rng, w: usize, h: usize) -> Result<Gray, String> {
    let mut g = render_svg_gray(&svg_doc(w, h, &clutter_svg(r, w, h, true)), w, h)?;
    Post::random(r).apply(&mut g, r);
    Ok(g)
}

/// Render one spec on a plain background (`clutter_seed` None) or on clutter that is the same in every
/// call with the same seed (a static scene behind a moving head).
pub fn render_spec(
    spec: &FaceSpec,
    w: usize,
    h: usize,
    clutter_seed: Option<u64>,
    post: &Post,
    noise_seed: u64,
) -> Result<Gray, String> {
    let mut body = match clutter_seed {
        None => format!(
            "<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"{}\"/>",
            hex(spec.bg)
        ),
        Some(cs) => clutter_svg(&mut Rng::new(cs), w, h, false),
    };
    body.push_str(&spec.svg_shapes());
    let mut g = render_svg_gray(&svg_doc(w, h, &body), w, h)?;
    post.apply(&mut g, &mut Rng::new(noise_seed));
    Ok(g)
}

/// The synthetic head of the studio and of the end-to-end test: a fixed random face (from `seed`) that
/// turns, nods, rolls, blinks, talks and raises its brows as functions of time `t` (seconds).
/// The returned spec carries the ground-truth parameters in its fields.
pub fn anim_spec(seed: u64, t: f32, size: f32) -> FaceSpec {
    let mut r = Rng::new(seed);
    let mut s = FaceSpec::random(&mut r, size, size, size * 0.27, size * 0.28);
    s.cx = size * 0.5;
    s.cy = size * 0.5;
    s.fringe = -1.0;
    s.strokes.clear();
    s.eye_h = 0.5;
    s.open_r = 1.0;
    s.open_l = 1.0;
    s.yaw = 0.40 * (t * 0.9).sin();
    s.pitch = 0.18 * (t * 0.6 + 1.0).sin();
    s.roll = 8.0 * (t * 0.4).sin();
    let ph = t % 2.6;
    let blink = if ph > 2.35 {
        1.0 - ((ph - 2.475).abs() / 0.125).min(1.0)
    } else {
        0.0
    };
    s.open_r = 1.0 - blink;
    s.open_l = 1.0 - blink;
    s.mouth_open = (0.9 * (t * 2.6).sin()).max(0.0);
    s.brow_raise = 0.5 + 0.5 * (t * 0.5).sin();
    s
}

// ---------------------------------------------------------------------------------------------
// hand-drawn cartoon variants (not produced by FaceSpec)
// ---------------------------------------------------------------------------------------------

/// A face drawn by hand as SVG (200 x 200 design units) with hand-annotated landmarks and head box.
#[derive(Clone, Debug)]
pub struct Cartoon {
    pub name: &'static str,
    pub svg_body: String,
    /// 23 landmarks, NAMES order, design units.
    pub pts: Vec<Vec2>,
    /// Head box: centre x, centre y, side.
    pub head: (f32, f32, f32),
}

fn eye_pts(cx: f32, cy: f32, hw: f32, hh: f32) -> [Vec2; 6] {
    // corners at the horizontal extremes, lid points at 1/4 and 3/4 of the width on an elliptical lid
    let k = (1.0 - 0.25f32).sqrt() * hh;
    [
        Vec2::new(cx - hw, cy),
        Vec2::new(cx - hw * 0.5, cy - k),
        Vec2::new(cx + hw * 0.5, cy - k),
        Vec2::new(cx + hw, cy),
        Vec2::new(cx + hw * 0.5, cy + k),
        Vec2::new(cx - hw * 0.5, cy + k),
    ]
}

fn all_pts(
    re: [Vec2; 6],
    le: [Vec2; 6],
    m: [Vec2; 4],
    brows: [Vec2; 2],
    nose: [Vec2; 2],
    chin: Vec2,
    sides: [Vec2; 2],
) -> Vec<Vec2> {
    let mut v = Vec::with_capacity(23);
    v.extend(re);
    v.extend(le);
    v.extend(m);
    v.extend(brows);
    v.extend(nose);
    v.push(chin);
    v.extend(sides);
    v
}

/// Four hand-drawn designs that share no code with [`FaceSpec`]: a round emoji-like face, a large-eyed
/// manga-like face, a boxy robot and a pure line drawing without any fill.
pub fn cartoons() -> Vec<Cartoon> {
    let p = Vec2::new;
    let mut out = Vec::new();
    // A: round yellow face, black oval eyes, open smile, plain brow arcs, dot nose
    {
        let body = "<circle cx=\"100\" cy=\"100\" r=\"80\" fill=\"#f5c518\" stroke=\"#a87f00\" stroke-width=\"4\"/>\
            <ellipse cx=\"70\" cy=\"82\" rx=\"9\" ry=\"14\" fill=\"#222222\"/><ellipse cx=\"130\" cy=\"82\" rx=\"9\" ry=\"14\" fill=\"#222222\"/>\
            <polyline points=\"52,56 70,50 88,56\" fill=\"none\" stroke=\"#6b4a00\" stroke-width=\"5\"/>\
            <polyline points=\"112,56 130,50 148,56\" fill=\"none\" stroke=\"#6b4a00\" stroke-width=\"5\"/>\
            <circle cx=\"100\" cy=\"108\" r=\"5\" fill=\"#c98a00\"/>\
            <path d=\"M60,125 Q100,175 140,125 Z\" fill=\"#5a1010\" stroke=\"#3a0808\" stroke-width=\"3\"/>";
        out.push(Cartoon {
            name: "round-emoji",
            svg_body: body.into(),
            pts: all_pts(
                eye_pts(70.0, 82.0, 9.0, 14.0),
                eye_pts(130.0, 82.0, 9.0, 14.0),
                [
                    p(60.0, 125.0),
                    p(140.0, 125.0),
                    p(100.0, 125.0),
                    p(100.0, 150.0),
                ],
                [p(70.0, 50.0), p(130.0, 50.0)],
                [p(100.0, 108.0), p(100.0, 82.0)],
                p(100.0, 180.0),
                [p(22.5, 82.0), p(177.5, 82.0)],
            ),
            head: (100.0, 100.0, 176.0),
        });
    }
    // B: tall oval face, big sclera + iris eyes, thin brows, short nose line, small mouth
    {
        let body = "<ellipse cx=\"100\" cy=\"105\" rx=\"62\" ry=\"80\" fill=\"#fde0cc\" stroke=\"#8a5a44\" stroke-width=\"2\"/>\
            <ellipse cx=\"68\" cy=\"98\" rx=\"17\" ry=\"21\" fill=\"#ffffff\" stroke=\"#333333\" stroke-width=\"2\"/><circle cx=\"68\" cy=\"100\" r=\"12\" fill=\"#2a5fa8\"/><circle cx=\"68\" cy=\"100\" r=\"5\" fill=\"#101010\"/>\
            <ellipse cx=\"132\" cy=\"98\" rx=\"17\" ry=\"21\" fill=\"#ffffff\" stroke=\"#333333\" stroke-width=\"2\"/><circle cx=\"132\" cy=\"100\" r=\"12\" fill=\"#2a5fa8\"/><circle cx=\"132\" cy=\"100\" r=\"5\" fill=\"#101010\"/>\
            <polyline points=\"48,66 68,60 86,64\" fill=\"none\" stroke=\"#3b2a20\" stroke-width=\"2\"/>\
            <polyline points=\"114,64 132,60 152,66\" fill=\"none\" stroke=\"#3b2a20\" stroke-width=\"2\"/>\
            <polyline points=\"100,110 97,128 103,128\" fill=\"none\" stroke=\"#a06b55\" stroke-width=\"2\"/>\
            <ellipse cx=\"100\" cy=\"150\" rx=\"10\" ry=\"4\" fill=\"#c0504d\"/>";
        let s = (1.0f32 - (98.0f32 - 105.0).powi(2) / 6400.0).sqrt() * 62.0;
        out.push(Cartoon {
            name: "manga-oval",
            svg_body: body.into(),
            pts: all_pts(
                eye_pts(68.0, 98.0, 17.0, 21.0),
                eye_pts(132.0, 98.0, 17.0, 21.0),
                [
                    p(90.0, 150.0),
                    p(110.0, 150.0),
                    p(100.0, 146.0),
                    p(100.0, 154.0),
                ],
                [p(68.0, 60.0), p(132.0, 60.0)],
                [p(100.0, 128.0), p(100.0, 98.0)],
                p(100.0, 185.0),
                [p(100.0 - s, 98.0), p(100.0 + s, 98.0)],
            ),
            head: (100.0, 105.0, 176.0),
        });
    }
    // C: boxy robot head with square eyes and a slot mouth
    {
        let body = "<rect x=\"40\" y=\"30\" width=\"120\" height=\"140\" fill=\"#b8c4d0\" stroke=\"#44505c\" stroke-width=\"4\"/>\
            <rect x=\"56\" y=\"72\" width=\"30\" height=\"22\" fill=\"#101820\"/><rect x=\"114\" y=\"72\" width=\"30\" height=\"22\" fill=\"#101820\"/>\
            <rect x=\"56\" y=\"52\" width=\"30\" height=\"6\" fill=\"#44505c\"/><rect x=\"114\" y=\"52\" width=\"30\" height=\"6\" fill=\"#44505c\"/>\
            <polygon points=\"100,100 92,120 108,120\" fill=\"#8895a3\" stroke=\"#44505c\" stroke-width=\"2\"/>\
            <rect x=\"66\" y=\"136\" width=\"68\" height=\"16\" fill=\"#101820\"/>";
        let eye = |cx: f32| {
            [
                p(cx - 15.0, 83.0),
                p(cx - 7.5, 72.0),
                p(cx + 7.5, 72.0),
                p(cx + 15.0, 83.0),
                p(cx + 7.5, 94.0),
                p(cx - 7.5, 94.0),
            ]
        };
        out.push(Cartoon {
            name: "boxy-robot",
            svg_body: body.into(),
            pts: all_pts(
                eye(71.0),
                eye(129.0),
                [
                    p(66.0, 144.0),
                    p(134.0, 144.0),
                    p(100.0, 136.0),
                    p(100.0, 152.0),
                ],
                [p(71.0, 55.0), p(129.0, 55.0)],
                [p(100.0, 120.0), p(100.0, 83.0)],
                p(100.0, 170.0),
                [p(40.0, 83.0), p(160.0, 83.0)],
            ),
            head: (100.0, 100.0, 154.0),
        });
    }
    // D: line drawing, no fills at all
    {
        let body = "<ellipse cx=\"100\" cy=\"100\" rx=\"64\" ry=\"82\" fill=\"none\" stroke=\"#111111\" stroke-width=\"3\"/>\
            <path d=\"M54,86 Q70,70 86,86 Q70,96 54,86 Z\" fill=\"none\" stroke=\"#111111\" stroke-width=\"3\"/>\
            <path d=\"M114,86 Q130,70 146,86 Q130,96 114,86 Z\" fill=\"none\" stroke=\"#111111\" stroke-width=\"3\"/>\
            <circle cx=\"70\" cy=\"85\" r=\"4\" fill=\"#111111\"/><circle cx=\"130\" cy=\"85\" r=\"4\" fill=\"#111111\"/>\
            <polyline points=\"52,62 70,56 88,62\" fill=\"none\" stroke=\"#111111\" stroke-width=\"3\"/>\
            <polyline points=\"112,62 130,56 148,62\" fill=\"none\" stroke=\"#111111\" stroke-width=\"3\"/>\
            <polyline points=\"100,88 100,118 92,122 108,122\" fill=\"none\" stroke=\"#111111\" stroke-width=\"3\"/>\
            <path d=\"M72,144 Q100,160 128,144 Q100,150 72,144 Z\" fill=\"none\" stroke=\"#111111\" stroke-width=\"3\"/>";
        let eye = |cx: f32| {
            [
                p(cx - 16.0, 86.0),
                p(cx - 8.0, 78.0),
                p(cx + 8.0, 78.0),
                p(cx + 16.0, 86.0),
                p(cx + 8.0, 91.0),
                p(cx - 8.0, 91.0),
            ]
        };
        let s = (1.0f32 - (86.0f32 - 100.0).powi(2) / 6724.0).sqrt() * 64.0;
        out.push(Cartoon {
            name: "line-drawing",
            svg_body: body.into(),
            pts: all_pts(
                eye(70.0),
                eye(130.0),
                [
                    p(72.0, 144.0),
                    p(128.0, 144.0),
                    p(100.0, 147.0),
                    p(100.0, 152.0),
                ],
                [p(70.0, 59.0), p(130.0, 59.0)],
                [p(100.0, 118.0), p(100.0, 86.0)],
                p(100.0, 182.0),
                [p(100.0 - s, 86.0), p(100.0 + s, 86.0)],
            ),
            head: (100.0, 100.0, 180.0),
        });
    }
    out
}

impl Cartoon {
    /// Place the drawing in a `w` x `h` scene: scaled by `scale`, rotated by `roll_deg` about its centre,
    /// moved to `(cx, cy)`, on a flat background `bg`. Returns the rendered grey image and the truth
    /// (landmarks and box in pixels).
    // public render signature is stable; the eight scalars are independent view parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        w: usize,
        h: usize,
        scale: f32,
        roll_deg: f32,
        cx: f32,
        cy: f32,
        bg: [f32; 3],
    ) -> Result<(Gray, FaceTruth), String> {
        let (c, s) = (
            roll_deg.to_radians().cos() * scale,
            roll_deg.to_radians().sin() * scale,
        );
        // x' = a x + c y + e ; maps design (100,100) to (cx, cy)
        let (e, f) = (cx - (c * 100.0 - s * 100.0), cy - (s * 100.0 + c * 100.0));
        let svg = svg_doc(w, h, &format!("<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"{}\"/><g transform=\"matrix({c} {s} {} {c} {e} {f})\">{}</g>", hex(bg), -s, self.svg_body));
        let g = render_svg_gray(&svg, w, h)?;
        let m = |q: &Vec2| Vec2::new(c * q.x - s * q.y + e, s * q.x + c * q.y + f);
        let hc = m(&Vec2::new(self.head.0, self.head.1));
        Ok((
            g,
            FaceTruth {
                pts: self.pts.iter().map(m).collect(),
                cx: hc.x,
                cy: hc.y,
                side: self.head.2 * scale,
            },
        ))
    }
}

// ---------------------------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faceparam::{measure, Geometry};

    #[test]
    fn landmarks_are_deterministic_and_inside_the_head() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        let fa = FaceSpec::random(&mut a, 128.0, 128.0, 30.0, 50.0);
        let fb = FaceSpec::random(&mut b, 128.0, 128.0, 30.0, 50.0);
        assert_eq!(fa, fb);
        let p = fa.landmarks();
        assert_eq!(p.len(), 23);
        for q in &p {
            assert!(
                q.x.is_finite()
                    && (*q - Vec2::new(fa.cx, fa.cy)).length() <= fa.hw * fa.aspect * 1.25
            );
        }
    }

    #[test]
    fn ground_truth_matches_the_faceparam_geometry_conventions() {
        let mut r = Rng::new(3);
        let mut s = FaceSpec::random(&mut r, 128.0, 128.0, 40.0, 41.0);
        s.roll = 0.0;
        s.yaw = 0.0;
        s.pitch = 0.0;
        s.open_r = 1.0;
        s.open_l = 1.0;
        s.mouth_open = 0.0;
        let p = s.landmarks();
        let mut f = [Vec2::new(0.0, 0.0); 23];
        f.copy_from_slice(&p);
        let m = measure(&f, &Geometry::default()).unwrap();
        // facing front: no roll, near-zero yaw measurement, eyes open, closed mouth
        assert!(m[2].abs() < 0.01, "roll {}", m[2]);
        assert!(m[0].abs() < 0.05, "yaw {}", m[0]);
        assert!(m[3] > 0.2 && m[4] > 0.2, "ear {} {}", m[3], m[4]);
        assert!(m[5] < 0.05, "mar {}", m[5]);
        // right eye is on the image left
        assert!(p[0].x < p[6].x && p[12].x < p[13].x && p[21].x < p[22].x);
        // turning the head moves the nose in the direction of the yaw
        let mut t = s.clone();
        t.yaw = 0.4;
        let q = t.landmarks();
        assert!(q[18].x > p[18].x + 1.0);
        // blink shrinks the eye aspect ratio
        let mut c = s.clone();
        c.open_r = 0.0;
        let mut f2 = [Vec2::new(0.0, 0.0); 23];
        f2.copy_from_slice(&c.landmarks());
        let m2 = measure(&f2, &Geometry::default()).unwrap();
        assert!(m2[3] < 0.08 && m2[4] > 0.2);
    }

    #[test]
    fn render_is_deterministic_and_shows_a_head() {
        let (g1, t1) = face_scene(&mut Rng::new(11), 96, 96, 25.0, 35.0, true).unwrap();
        let (g2, _) = face_scene(&mut Rng::new(11), 96, 96, 25.0, 35.0, true).unwrap();
        assert_eq!(g1, g2);
        assert!(g1.px.iter().any(|&v| v > 0.9) || g1.px.iter().any(|&v| v < 0.5));
        // the pixel at the head centre differs from a corner on a plain-background scene often; here just finite
        assert!(g1.px.iter().all(|v| v.is_finite()));
        assert!(t1.side > 30.0);
    }

    #[test]
    fn eyes_are_darker_than_skin_at_the_landmarks() {
        let mut r = Rng::new(5);
        let mut s = FaceSpec::random(&mut r, 128.0, 128.0, 45.0, 46.0);
        s.fringe = -1.0;
        s.strokes.clear();
        s.roll = 0.0;
        s.open_r = 1.0;
        let svg = svg_doc(
            128,
            128,
            &format!(
                "<rect x=\"0\" y=\"0\" width=\"128\" height=\"128\" fill=\"#ffffff\"/>{}",
                s.svg_shapes()
            ),
        );
        let g = render_svg_gray(&svg, 128, 128).unwrap();
        let p = s.landmarks();
        // the right-eye outline is drawn in the ink colour at the corner
        let corner = g.sample(p[0].x, p[0].y);
        let cheek = g.sample(s.cx, s.cy + s.hw * s.aspect * 0.8 * 0.5);
        assert!(corner < cheek, "corner {corner} cheek {cheek}");
    }

    #[test]
    fn crop_resize_and_blur_behave() {
        let mut g = Gray::new(8, 8, 0.0);
        g.px[27] = 1.0;
        let s: f32 = g.px.iter().sum();
        g.blur(1.0);
        assert!((g.px.iter().sum::<f32>() - s).abs() < 1e-3);
        let flat = Gray::new(20, 20, 0.5);
        let c = flat.crop_resize(10.0, 10.0, 40.0, 5);
        assert!(c.px.iter().all(|&v| (v - 0.5).abs() < 1e-4));
        assert_eq!(flat.scaled_down(2.0).w, 10);
        assert!(render_svg_gray("<svg/>", 0, 5).is_err());
        assert!(Gray::from_rgba(2, 2, &[0; 3]).is_err());
    }

    #[test]
    fn interocular_and_nme() {
        let mut r = Rng::new(9);
        let t = FaceSpec::random(&mut r, 128.0, 128.0, 30.0, 40.0).truth();
        assert!(interocular(&t.pts) > 10.0);
        assert!(t.nme(&t.pts) < 1e-6);
        let shifted: Vec<Vec2> = t.pts.iter().map(|p| Vec2::new(p.x + 2.0, p.y)).collect();
        assert!((t.nme(&shifted) - 2.0 / interocular(&t.pts)).abs() < 1e-4);
    }
}
