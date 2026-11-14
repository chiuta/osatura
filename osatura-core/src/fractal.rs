// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Fractal texture generator: deterministic, seamless-tileable surface textures with a height map, a tangent-space
//! normal map, a flat albedo and a fully lit colour image.
//!
//! * Noise family (all tileable with `tile`): fractional Brownian motion, ridged multifractal, billow, domain warp,
//!   cellular (Worley) cracks and marble veins. Every octave is limited to the Nyquist frequency of the image, so
//!   large textures keep gaining real detail instead of aliasing.
//! * Escape-time family (never tiled): Mandelbrot, Julia and Burning Ship with smooth (band-free) iteration counts
//!   and supersampling.
//! * Shading: Sobel normals from the height field, Lambert + Blinn-Phong with a metallic/dielectric specular term,
//!   two-scale cavity ambient occlusion and optional emission in the low parts (lava, cracks).
//!
//! Pure function of [`Params`]: the same parameters give the same bytes on every thread count (each pixel
//! depends only on its own coordinates). The ideas (gradient noise, fBm, ridged multifractal, Worley cells, domain
//! warping, smooth escape-time colouring) are textbook techniques; the code is written from the definitions.
//! Third-party constants: the golden-ratio/lowbias32 hash is [`crate::noise::hash01`] (see CLEANROOM.md); the Julia
//! and Mandelbrot view centres are mathematical facts about the sets, not code.

use crate::noise::hash01;

/// Largest side in pixels. A 4096² texture is 64 MB of RGBA.
pub const MAX_SIZE: u32 = 4096;
/// Smallest side in pixels.
pub const MIN_SIZE: u32 = 8;
const MAX_SAMPLES: u64 = 1 << 27;
/// Worst-case escape-time iterations (samples x iteration limit); about half a minute on a laptop.
const MAX_ESCAPE_WORK: u64 = 1 << 34;

/// The pattern that makes the height field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Fbm,
    Ridged,
    Billow,
    Warp,
    Cellular,
    Marble,
    Crag,
    Mandelbrot,
    Julia,
    BurningShip,
}

impl Kind {
    pub const ALL: [Kind; 10] = [
        Kind::Fbm,
        Kind::Ridged,
        Kind::Billow,
        Kind::Warp,
        Kind::Cellular,
        Kind::Marble,
        Kind::Crag,
        Kind::Mandelbrot,
        Kind::Julia,
        Kind::BurningShip,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Kind::Fbm => "fbm",
            Kind::Ridged => "ridged",
            Kind::Billow => "billow",
            Kind::Warp => "warp",
            Kind::Cellular => "cellular",
            Kind::Marble => "marble",
            Kind::Crag => "crag",
            Kind::Mandelbrot => "mandelbrot",
            Kind::Julia => "julia",
            Kind::BurningShip => "burning-ship",
        }
    }
    pub fn parse(s: &str) -> Option<Kind> {
        Kind::ALL.iter().copied().find(|k| k.name() == s)
    }
    /// Escape-time patterns cannot be tiled.
    pub fn is_escape(self) -> bool {
        matches!(self, Kind::Mandelbrot | Kind::Julia | Kind::BurningShip)
    }
}

/// Colour ramp: stops `(position 0..1, linear RGB)`, sorted by position.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette(pub Vec<(f32, [f32; 3])>);

fn srgb_to_lin(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
fn lin_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

impl Palette {
    /// From `(position, 0xRRGGBB sRGB)` stops.
    pub fn hex(stops: &[(f32, u32)]) -> Palette {
        Palette(
            stops
                .iter()
                .map(|&(p, c)| {
                    (
                        p,
                        [
                            srgb_to_lin(((c >> 16) & 255) as f32 / 255.0),
                            srgb_to_lin(((c >> 8) & 255) as f32 / 255.0),
                            srgb_to_lin((c & 255) as f32 / 255.0),
                        ],
                    )
                })
                .collect(),
        )
    }
    /// Linear RGB at `t` (clamped to 0..1).
    pub fn at(&self, t: f32) -> [f32; 3] {
        let s = &self.0;
        if s.is_empty() {
            return [t, t, t];
        }
        if t <= s[0].0 {
            return s[0].1;
        }
        for w in s.windows(2) {
            if t <= w[1].0 {
                let k = if w[1].0 > w[0].0 {
                    (t - w[0].0) / (w[1].0 - w[0].0)
                } else {
                    0.0
                };
                return [0, 1, 2].map(|i| w[0].1[i] + (w[1].1[i] - w[0].1[i]) * k);
            }
        }
        s[s.len() - 1].1
    }
}

/// Everything that decides the output.
#[derive(Clone, Debug)]
pub struct Params {
    pub kind: Kind,
    /// Side in pixels (square), `MIN_SIZE..=MAX_SIZE`.
    pub size: u32,
    pub seed: u32,
    /// Noise octaves 1..=16 (extra octaves above the Nyquist limit are skipped).
    pub octaves: u32,
    /// Frequency growth per octave (forced to 2 when tiling so the octaves stay periodic).
    pub lacunarity: f32,
    /// Amplitude decay per octave.
    pub gain: f32,
    /// Lattice cells across the image at the first octave.
    pub cells: u32,
    /// Domain-warp strength (Warp and Marble), in first-octave cells.
    pub warp: f32,
    /// Seamless on all four edges (noise kinds only).
    pub tile: bool,
    /// Escape-time zoom (1 = the whole set) and the view preset picked by `seed`.
    pub zoom: f64,
    /// Supersampling per axis 1..=4.
    pub ssaa: u32,
    /// Height contrast applied after generation (1 = as generated).
    pub contrast: f32,
    /// Bump strength of the normal map and of the lighting.
    pub relief: f32,
    /// Light direction: azimuth in degrees (0 = right, 90 = up) and elevation above the surface.
    pub light_az: f32,
    pub light_elev: f32,
    /// Specular strength 0..1 and shininess.
    pub specular: f32,
    pub shininess: f32,
    /// Cavity darkening 0..1.
    pub ao: f32,
    /// Emission in the low parts, 0 = none.
    pub glow: f32,
    /// How fast the palette cycles over the escape-time iteration count.
    pub color_speed: f32,
    pub palette: Palette,
}

impl Default for Params {
    fn default() -> Params {
        Params {
            kind: Kind::Fbm,
            size: 1024,
            seed: 1,
            octaves: 10,
            lacunarity: 2.0,
            gain: 0.5,
            cells: 4,
            warp: 1.5,
            tile: true,
            zoom: 1.0,
            ssaa: 1,
            contrast: 1.0,
            relief: 1.0,
            light_az: 135.0,
            light_elev: 45.0,
            specular: 0.25,
            shininess: 24.0,
            ao: 0.7,
            glow: 0.0,
            color_speed: 1.0,
            palette: Palette::hex(&[(0.0, 0x202020), (1.0, 0xe0e0e0)]),
        }
    }
}

/// The generated maps, all `size × size`, row-major, top row first.
#[derive(Debug)]
pub struct Texture {
    pub size: u32,
    /// Height 0..1.
    pub height: Vec<f32>,
    /// Straight-alpha RGBA8 sRGB, no lighting.
    pub albedo: Vec<u8>,
    /// Straight-alpha RGBA8: albedo with lighting, cavity occlusion, specular and emission.
    pub color: Vec<u8>,
    /// RGBA8 tangent-space normal map, +Y up (OpenGL convention): flat = (128, 128, 255).
    pub normal: Vec<u8>,
}

// ---------------------------------------------------------------- noise ----

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Seed of octave `o`: a hash of both, so two different seeds never share an octave pattern
/// (a plain `seed + o * K` makes seed `s` octave 1 identical to seed `s + K` octave 0).
fn octave_seed(seed: u32, o: u32) -> u32 {
    let h = (hash01(o as i32, seed) * 16_777_216.0) as u32;
    h ^ seed.rotate_left(7 * (o % 31) + 1)
}

fn lattice(ix: i32, iy: i32, per: i32, seed: u32) -> (f32, f32) {
    let (x, y) = if per > 0 {
        (ix.rem_euclid(per), iy.rem_euclid(per))
    } else {
        (ix, iy)
    };
    let a = hash01(x, seed);
    let h = hash01(
        ((a * 16_777_216.0) as i32) ^ y.wrapping_mul(0x1B87_3593),
        seed ^ 0xA5A5_5A5A,
    );
    // 32 unit directions (cos, sin of k * 11.25 degrees): a table lookup instead of two trig calls per lattice corner
    GRADS[((h * 32.0) as usize).min(31)]
}

const R: f32 = std::f32::consts::FRAC_1_SQRT_2;

#[rustfmt::skip]
const GRADS: [(f32, f32); 32] = [
    (1.0, 0.0), (0.980_785, 0.195_090), (0.923_880, 0.382_683), (0.831_470, 0.555_570),
    (R, R), (0.555_570, 0.831_470), (0.382_683, 0.923_880), (0.195_090, 0.980_785),
    (0.0, 1.0), (-0.195_090, 0.980_785), (-0.382_683, 0.923_880), (-0.555_570, 0.831_470),
    (-R, R), (-0.831_470, 0.555_570), (-0.923_880, 0.382_683), (-0.980_785, 0.195_090),
    (-1.0, 0.0), (-0.980_785, -0.195_090), (-0.923_880, -0.382_683), (-0.831_470, -0.555_570),
    (-R, -R), (-0.555_570, -0.831_470), (-0.382_683, -0.923_880), (-0.195_090, -0.980_785),
    (0.0, -1.0), (0.195_090, -0.980_785), (0.382_683, -0.923_880), (0.555_570, -0.831_470),
    (R, -R), (0.831_470, -0.555_570), (0.923_880, -0.382_683), (0.980_785, -0.195_090),
];

/// Gradient noise, about -1..1. `per > 0` repeats every `per` lattice cells on both axes.
fn gnoise(x: f32, y: f32, per: i32, seed: u32) -> f32 {
    let (fx, fy) = (x.floor(), y.floor());
    let (ix, iy) = (fx as i32, fy as i32);
    let (tx, ty) = (x - fx, y - fy);
    let d = |dx: i32, dy: i32| {
        let (gx, gy) = lattice(ix + dx, iy + dy, per, seed);
        gx * (tx - dx as f32) + gy * (ty - dy as f32)
    };
    let (u, v) = (fade(tx), fade(ty));
    let a = d(0, 0) + (d(1, 0) - d(0, 0)) * u;
    let b = d(0, 1) + (d(1, 1) - d(0, 1)) * u;
    (a + (b - a) * v) * std::f32::consts::SQRT_2
}

/// Worley distances to the nearest and second nearest feature point.
fn worley(x: f32, y: f32, per: i32, seed: u32) -> (f32, f32) {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (mut f1, mut f2) = (9.0f32, 9.0f32);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (cx, cy) = (ix + dx, iy + dy);
            let (wx, wy) = if per > 0 {
                (cx.rem_euclid(per), cy.rem_euclid(per))
            } else {
                (cx, cy)
            };
            // both offsets depend on the whole cell (wx, wy): an offset from wx alone lines the points up in columns
            let a = (hash01(wx, seed) * 16_777_216.0) as i32;
            let jx = hash01(a ^ wy.wrapping_mul(0x1B87_3593), seed ^ 0x3C6E_F372);
            let jy = hash01(a ^ wy.wrapping_mul(0x2545_F491), seed ^ 0xA54F_F53A);
            let px = cx as f32 + 0.1 + 0.8 * jx;
            let py = cy as f32 + 0.1 + 0.8 * jy;
            let d = ((px - x) * (px - x) + (py - y) * (py - y)).sqrt();
            if d < f1 {
                f2 = f1;
                f1 = d;
            } else if d < f2 {
                f2 = d;
            }
        }
    }
    (f1, f2)
}

struct Field {
    kind: Kind,
    base: f32,
    per0: i32,
    octaves: u32,
    lac: f32,
    gain: f32,
    warp: f32,
    seed: u32,
    tile: bool,
}

impl Field {
    fn new(p: &Params) -> Field {
        let tile = p.tile && !p.kind.is_escape();
        let cells = p.cells.clamp(1, 256);
        // Nyquist: an octave with more than size/2 cells would alias.
        let max_cells = (p.size / 2).max(1);
        let lac = if tile {
            2.0
        } else {
            p.lacunarity.clamp(1.2, 4.0)
        };
        let mut octaves = 0u32;
        let mut c = cells as f32;
        while octaves < p.octaves.clamp(1, 16) && (octaves == 0 || c <= max_cells as f32) {
            octaves += 1;
            c *= lac;
        }
        Field {
            kind: p.kind,
            base: cells as f32,
            per0: if tile { cells as i32 } else { 0 },
            octaves,
            lac,
            gain: p.gain.clamp(0.1, 0.95),
            warp: p.warp,
            seed: p.seed,
            tile,
        }
    }

    /// Octave sum of `f(x, y, period, seed)` at `(u, v)` in 0..1 image coordinates, normalised to about -1..1.
    fn sum(&self, u: f32, v: f32, off: f32, f: impl Fn(f32, f32, i32, u32) -> f32) -> f32 {
        let (mut amp, mut freq, mut acc, mut norm) = (1.0f32, self.base, 0.0f32, 0.0f32);
        let mut per = self.per0 as f64;
        for o in 0..self.octaves {
            let p = per.round() as i32;
            let s = octave_seed(self.seed, o);
            // the offset must be a whole number of periods to stay seamless
            let (x, y) = if self.tile {
                (
                    u * freq + (off * (o as f32 + 1.0)).round() * p as f32,
                    v * freq + (off * 1.7 * (o as f32 + 1.0)).round() * p as f32,
                )
            } else {
                (
                    u * freq + off * (o as f32 + 1.0),
                    v * freq + off * 1.7 * (o as f32 + 1.0),
                )
            };
            acc += amp * f(x, y, if self.tile { p } else { 0 }, s);
            norm += amp;
            amp *= self.gain;
            freq *= self.lac;
            per *= self.lac as f64;
        }
        acc / norm.max(1e-6)
    }

    fn fbm(&self, u: f32, v: f32, off: f32) -> f32 {
        self.sum(u, v, off, gnoise)
    }

    /// Ridged multifractal, 0..1.
    fn ridged(&self, u: f32, v: f32) -> f32 {
        let (mut amp, mut freq, mut acc, mut norm, mut w) =
            (1.0f32, self.base, 0.0f32, 0.0f32, 1.0f32);
        let mut per = self.per0 as f64;
        for o in 0..self.octaves {
            let p = per.round() as i32;
            let n = gnoise(
                u * freq,
                v * freq,
                if self.tile { p } else { 0 },
                octave_seed(self.seed, o),
            );
            let mut s = 1.0 - n.abs();
            s *= s;
            s *= w;
            w = (s * 2.0).clamp(0.0, 1.0);
            acc += s * amp;
            norm += amp;
            amp *= self.gain;
            freq *= self.lac;
            per *= self.lac as f64;
        }
        (acc / norm.max(1e-6)).clamp(0.0, 1.0)
    }

    /// Height 0..1 at image coordinates `(u, v)`.
    fn height(&self, u: f32, v: f32) -> f32 {
        match self.kind {
            Kind::Fbm => 0.5 + 0.5 * self.fbm(u, v, 0.0) * 1.35,
            Kind::Billow => {
                let s = self.sum(u, v, 0.0, |x, y, p, s| gnoise(x, y, p, s).abs() * 2.0 - 1.0);
                0.5 + 0.5 * s * 1.4
            }
            Kind::Ridged => self.ridged(u, v),
            Kind::Crag => {
                // weathered rock: a ridged mass cut by two scales of fractures
                let mass = 0.65 * self.ridged(u, v) + 0.35 * (0.5 + 0.5 * self.fbm(u, v, 2.0));
                let (f1, f2) = worley(
                    u * self.base * 2.0,
                    v * self.base * 2.0,
                    self.per0 * 2,
                    self.seed ^ 0x51,
                );
                let (g1, g2) = worley(
                    u * self.base * 8.0,
                    v * self.base * 8.0,
                    self.per0 * 8,
                    self.seed ^ 0x77,
                );
                let step = |x: f32, w: f32| {
                    let t = (x / w).clamp(0.0, 1.0);
                    t * t * (3.0 - 2.0 * t)
                };
                let big = step(f2 - f1, 0.10);
                let small = step(g2 - g1, 0.05);
                let slab = 0.82 + 0.18 * (1.0 - f1);
                let depth = 0.35 + 0.35 * (0.5 + 0.5 * self.fbm(u, v, 6.0));
                (mass * slab * (1.0 - depth * (1.0 - big)) * (0.92 + 0.08 * small)).clamp(0.0, 1.0)
            }
            Kind::Warp => {
                let k = self.warp * 0.12;
                let qx = self.fbm(u, v, 3.0);
                let qy = self.fbm(u, v, 7.0);
                let (wu, wv) = (u + k * qx, v + k * qy);
                // a second pass folds the warp into itself, which gives the marbled, flowing look
                let rx = self.fbm(wu, wv, 11.0);
                let ry = self.fbm(wu, wv, 13.0);
                let h = self.fbm(u + k * 1.4 * rx, v + k * 1.4 * ry, 0.0);
                0.5 + 0.5 * h * 1.5
            }
            Kind::Cellular => {
                // cracked plates: the plate surface is rough noise, the cracks are the thin zone where F2-F1 is small
                let (f1, f2) = worley(u * self.base, v * self.base, self.per0, self.seed);
                let edge = (f2 - f1).clamp(0.0, 1.0);
                let crack = (edge * 5.0).min(1.0);
                let crack = crack * crack * (3.0 - 2.0 * crack);
                let plate = 0.55 + 0.25 * (1.0 - f1) + 0.3 * self.fbm(u, v, 5.0);
                (plate * crack * 0.9 + 0.05).clamp(0.0, 1.0)
            }
            Kind::Marble => {
                let t = self.fbm(u, v, 9.0);
                let ph =
                    (u * self.base.round() * 2.0 + v * self.base.round() + self.warp * t * 2.0)
                        * std::f32::consts::TAU;
                let vein = 1.0 - ph.sin().abs();
                let vein = vein.powf(3.0);
                let fine = 1.0 - ((ph * 3.0 + self.fbm(u, v, 4.0) * 9.0).sin()).abs();
                (0.7 + 0.2 * self.fbm(u, v, 0.0)
                    - 0.5 * vein
                    - 0.18 * fine.powf(8.0) * (0.4 + vein))
                    .clamp(0.0, 1.0)
            }
            _ => 0.0,
        }
    }
}

// ----------------------------------------------------------- escape time ----

const MANDEL_VIEWS: [(f64, f64); 6] = [
    (-0.743_643_887_037_151, 0.131_825_904_205_33), // seahorse valley
    (-0.088, 0.654),                                // triple spiral
    (0.25, 0.0),                                    // elephant valley cusp
    (-1.754_877_666_246_693, 0.0),                  // period-3 minibrot
    (-0.7453, 0.1127),                              // the spiral arm between the two bulbs
    (-0.16, 1.0405),                                // a filament of the upper antenna
];
const JULIA_C: [(f64, f64); 6] = [
    (-0.8, 0.156),
    (-0.4, 0.6),
    (0.285, 0.01),
    (-0.701_76, -0.3842),
    (-0.835, -0.2321),
    (-0.7269, 0.1889),
];
const SHIP_VIEWS: [(f64, f64); 3] = [(-1.762, -0.028), (-1.861, -0.004), (-1.7443, -0.0173)];

struct Escape {
    kind: Kind,
    cx: f64,
    cy: f64,
    span: f64,
    jc: (f64, f64),
    max_iter: u32,
}

/// Iteration limit for a zoom level.
fn max_iter_for(zoom: f64) -> u32 {
    (256.0 + 96.0 * zoom.max(1.0).log2()).min(6000.0) as u32
}

impl Escape {
    fn new(p: &Params) -> Escape {
        let zoom = p.zoom.clamp(1.0, 1e12);
        let i = p.seed as usize;
        // the view slides from the whole set to the seed's target between zoom 1 and 8, so no zoom value jumps
        let k = (zoom.log2() / 3.0).clamp(0.0, 1.0);
        let k = k * k * (3.0 - 2.0 * k);
        let lerp = |a: f64, b: f64| a + (b - a) * k;
        let (cx, cy, span, jc) = match p.kind {
            Kind::Julia => (0.0, 0.0, 3.0 / zoom, JULIA_C[i % JULIA_C.len()]),
            Kind::BurningShip => {
                let (x, y) = SHIP_VIEWS[i % SHIP_VIEWS.len()];
                (lerp(-0.5, x), lerp(-0.5, y), 3.2 / zoom, (0.0, 0.0))
            }
            _ => {
                let (x, y) = MANDEL_VIEWS[i % MANDEL_VIEWS.len()];
                (lerp(-0.6, x), lerp(0.0, y), 3.2 / zoom, (0.0, 0.0))
            }
        };
        Escape {
            kind: p.kind,
            cx,
            cy,
            span,
            jc,
            max_iter: max_iter_for(zoom),
        }
    }

    /// Smooth iteration count at image coordinates, `None` inside the set.
    fn nu(&self, u: f64, v: f64) -> Option<f64> {
        let x = self.cx + (u - 0.5) * self.span;
        let y = self.cy + (v - 0.5) * self.span;
        if self.kind == Kind::Mandelbrot {
            // main cardioid and period-2 bulb are inside: exact, and they are the slowest pixels
            let q = (x - 0.25) * (x - 0.25) + y * y;
            if q * (q + (x - 0.25)) <= 0.25 * y * y || (x + 1.0) * (x + 1.0) + y * y <= 0.0625 {
                return None;
            }
        }
        let (mut zr, mut zi, cr, ci) = match self.kind {
            Kind::Julia => (x, y, self.jc.0, self.jc.1),
            _ => (0.0, 0.0, x, y),
        };
        let bail = 256.0 * 256.0;
        for n in 0..self.max_iter {
            let (a, b) = if self.kind == Kind::BurningShip {
                (zr.abs(), zi.abs())
            } else {
                (zr, zi)
            };
            zr = a * a - b * b + cr;
            zi = 2.0 * a * b + ci;
            let m = zr * zr + zi * zi;
            if m > bail {
                return Some(n as f64 + 1.0 - (m.ln() * 0.5).ln() / std::f64::consts::LN_2);
            }
        }
        None
    }
}

// ----------------------------------------------------------- generation ----

fn par_rows<T: Send, F: Fn(usize, &mut [T]) + Sync>(buf: &mut [T], w: usize, f: &F) {
    let h = buf.len() / w.max(1);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let n = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .min(16)
            .min(h.max(1));
        if n > 1 {
            let per = h.div_ceil(n);
            std::thread::scope(|s| {
                for (bi, band) in buf.chunks_mut(per * w).enumerate() {
                    s.spawn(move || {
                        for (r, row) in band.chunks_mut(w).enumerate() {
                            f(bi * per + r, row);
                        }
                    });
                }
            });
            return;
        }
    }
    for (r, row) in buf.chunks_mut(w).enumerate() {
        f(r, row);
    }
}

fn blur(src: &[f32], n: usize, r: usize, wrap: bool) -> Vec<f32> {
    let idx = |i: isize| -> usize {
        if wrap {
            i.rem_euclid(n as isize) as usize
        } else {
            i.clamp(0, n as isize - 1) as usize
        }
    };
    let k = (2 * r + 1) as f32;
    let mut tmp = vec![0.0f32; n * n];
    par_rows(&mut tmp, n, &|y, out: &mut [f32]| {
        let row = &src[y * n..(y + 1) * n];
        let mut acc: f32 = (-(r as isize)..=r as isize).map(|d| row[idx(d)]).sum();
        for (x, o) in out.iter_mut().enumerate() {
            *o = acc / k;
            acc += row[idx(x as isize + r as isize + 1)] - row[idx(x as isize - r as isize)];
        }
    });
    // vertical pass: one running sum per column, advanced a whole row at a time
    let mut out = vec![0.0f32; n * n];
    let mut acc = vec![0.0f32; n];
    for d in -(r as isize)..=r as isize {
        let row = &tmp[idx(d) * n..(idx(d) + 1) * n];
        for (a, v) in acc.iter_mut().zip(row) {
            *a += v;
        }
    }
    for y in 0..n {
        for (o, a) in out[y * n..(y + 1) * n].iter_mut().zip(&acc) {
            *o = a / k;
        }
        let add = &tmp[idx(y as isize + r as isize + 1) * n..][..n];
        let sub = &tmp[idx(y as isize - r as isize) * n..][..n];
        for ((a, p), q) in acc.iter_mut().zip(add).zip(sub) {
            *a += p - q;
        }
    }
    out
}

/// Check the parameters; the error says which one is wrong.
pub fn validate(p: &Params) -> Result<(), String> {
    if !(MIN_SIZE..=MAX_SIZE).contains(&p.size) {
        return Err(format!(
            "size must be {MIN_SIZE}..{MAX_SIZE} pixels, got {}",
            p.size
        ));
    }
    if !(1..=4).contains(&p.ssaa) {
        return Err(format!("ssaa must be 1..4, got {}", p.ssaa));
    }
    if (p.size as u64).pow(2) * (p.ssaa as u64).pow(2) > MAX_SAMPLES {
        return Err(format!(
            "{}x{} at ssaa {} is too many samples; lower the size or the ssaa",
            p.size, p.size, p.ssaa
        ));
    }
    for (n, v) in [
        ("lacunarity", p.lacunarity),
        ("gain", p.gain),
        ("warp", p.warp),
        ("contrast", p.contrast),
        ("relief", p.relief),
        ("light-az", p.light_az),
        ("light-elev", p.light_elev),
        ("specular", p.specular),
        ("shininess", p.shininess),
        ("ao", p.ao),
        ("glow", p.glow),
        ("color-speed", p.color_speed),
    ] {
        if !v.is_finite() {
            return Err(format!("{n} must be a finite number"));
        }
    }
    if !p.zoom.is_finite() {
        return Err("zoom must be a finite number".into());
    }
    if p.kind.is_escape() {
        let work = (p.size as u64).pow(2)
            * (p.ssaa as u64).pow(2)
            * max_iter_for(p.zoom.clamp(1.0, 1e12)) as u64;
        if work > MAX_ESCAPE_WORK {
            return Err(format!(
                "{}x{} at ssaa {} and zoom {} needs too many iterations; lower the size, the ssaa or the zoom",
                p.size, p.size, p.ssaa, p.zoom
            ));
        }
    }
    Ok(())
}

/// Generate all maps.
pub fn generate(p: &Params) -> Result<Texture, String> {
    validate(p)?;
    let n = p.size as usize;
    let aa = p.ssaa as usize;
    let tile = p.tile && !p.kind.is_escape();
    let esc_kind = p.kind.is_escape();
    let field = Field::new(p);
    let esc = Escape::new(p);
    // Height per pixel; for escape-time kinds also the linear colour, averaged over the sub-samples
    // (averaging the palette position instead would blend the interior into the boundary colours).
    let mut hgt = vec![0.0f32; n * n];
    let mut col = vec![[0.0f32; 3]; if esc_kind { n * n } else { 0 }];
    let sample = |px: usize, py: usize| -> (f32, [f32; 3]) {
        let (mut hs, mut cs) = (0.0f32, [0.0f32; 3]);
        for sy in 0..aa {
            for sx in 0..aa {
                let u = (px as f64 + (sx as f64 + 0.5) / aa as f64) / n as f64;
                let v = (py as f64 + (sy as f64 + 0.5) / aa as f64) / n as f64;
                if esc_kind {
                    let c = match esc.nu(u, v) {
                        Some(nu) => {
                            let nu = nu as f32;
                            hs += 1.0 - (-nu * 0.045).exp();
                            let t = nu.max(1.0).ln() * 0.9 * p.color_speed;
                            p.palette.at(1.0 - (2.0 * t.rem_euclid(1.0) - 1.0).abs())
                        }
                        None => [0.012, 0.012, 0.018],
                    };
                    for k in 0..3 {
                        cs[k] += c[k];
                    }
                } else {
                    hs += field.height(u as f32, v as f32);
                }
            }
        }
        let k = (aa * aa) as f32;
        (hs / k, [cs[0] / k, cs[1] / k, cs[2] / k])
    };
    if esc_kind {
        let mut both = vec![(0.0f32, [0.0f32; 3]); n * n];
        par_rows(&mut both, n, &|y, row: &mut [(f32, [f32; 3])]| {
            for (x, o) in row.iter_mut().enumerate() {
                *o = sample(x, y);
            }
        });
        for (i, (h, c)) in both.into_iter().enumerate() {
            hgt[i] = h;
            col[i] = c;
        }
    } else {
        par_rows(&mut hgt, n, &|y, row: &mut [f32]| {
            for (x, o) in row.iter_mut().enumerate() {
                *o = sample(x, y).0;
            }
        });
    }
    // contrast around the mean, then clamp
    let mean = (hgt.iter().map(|&v| v as f64).sum::<f64>() / (n * n) as f64) as f32;
    let mut height = hgt;
    for v in height.iter_mut() {
        *v = ((*v - mean) * p.contrast + mean).clamp(0.0, 1.0);
    }
    if !esc_kind {
        // stretch to the full range so every preset uses the whole palette
        let (lo, hi) = height
            .iter()
            .fold((1.0f32, 0.0f32), |a, &v| (a.0.min(v), a.1.max(v)));
        if hi - lo > 1e-4 {
            for v in height.iter_mut() {
                *v = (*v - lo) / (hi - lo);
            }
        }
    }
    // cavity occlusion at two scales
    let (b1, b2) = (
        blur(&height, n, (n / 128).max(2), tile),
        blur(&height, n, (n / 32).max(4), tile),
    );
    let (az, el) = (
        p.light_az.to_radians(),
        p.light_elev.clamp(1.0, 89.0).to_radians(),
    );
    let ld = [el.cos() * az.cos(), el.cos() * az.sin(), el.sin()];
    let hv = {
        let h = [ld[0], ld[1], ld[2] + 1.0];
        let l = (h[0] * h[0] + h[1] * h[1] + h[2] * h[2]).sqrt();
        [h[0] / l, h[1] / l, h[2] / l]
    };
    let at = |x: isize, y: isize| -> f32 {
        let (xi, yi) = if tile {
            (
                x.rem_euclid(n as isize) as usize,
                y.rem_euclid(n as isize) as usize,
            )
        } else {
            (
                x.clamp(0, n as isize - 1) as usize,
                y.clamp(0, n as isize - 1) as usize,
            )
        };
        height[yi * n + xi]
    };
    let slope = p.relief * n as f32 / 256.0 * 2.0;
    // shading pass: albedo, lit colour and normal of one pixel, 12 bytes
    let mut px = vec![[0u8; 12]; n * n];
    par_rows(&mut px, n, &|y, row: &mut [[u8; 12]]| {
        for (x, o) in row.iter_mut().enumerate() {
            let i = y * n + x;
            let (xi, yi) = (x as isize, y as isize);
            // Sobel normal, +Y up
            let dx = (at(xi + 1, yi - 1) + 2.0 * at(xi + 1, yi) + at(xi + 1, yi + 1))
                - (at(xi - 1, yi - 1) + 2.0 * at(xi - 1, yi) + at(xi - 1, yi + 1));
            let dy = (at(xi - 1, yi + 1) + 2.0 * at(xi, yi + 1) + at(xi + 1, yi + 1))
                - (at(xi - 1, yi - 1) + 2.0 * at(xi, yi - 1) + at(xi + 1, yi - 1));
            let (nx, ny, nz) = (-dx * 0.125 * slope, dy * 0.125 * slope, 1.0f32);
            let l = (nx * nx + ny * ny + nz * nz).sqrt();
            let nv = [nx / l, ny / l, nz / l];
            let h = height[i];
            let alb = if esc_kind { col[i] } else { p.palette.at(h) };
            let diff = (nv[0] * ld[0] + nv[1] * ld[1] + nv[2] * ld[2]).max(0.0);
            let spec = (nv[0] * hv[0] + nv[1] * hv[1] + nv[2] * hv[2])
                .max(0.0)
                .powf(p.shininess.max(1.0))
                * p.specular
                * (0.35 + 0.65 * h);
            let cav = 0.6 * (b1[i] - h).max(0.0) / 0.12 + 0.4 * (b2[i] - h).max(0.0) / 0.3;
            let ao = (1.0 - p.ao * cav.min(1.0) * 0.85).clamp(0.15, 1.0);
            let glow = p.glow * (1.0 - h).powi(3) * 2.5;
            for k in 0..3 {
                let lit = alb[k] * (0.22 + 0.95 * diff) * ao + spec * ao + alb[k] * glow;
                o[k] = (lin_to_srgb(alb[k]) * 255.0 + 0.5) as u8;
                o[4 + k] = (lin_to_srgb(lit) * 255.0 + 0.5) as u8;
                o[8 + k] = ((nv[k] * 0.5 + 0.5) * 255.0 + 0.5) as u8;
            }
            o[3] = 255;
            o[7] = 255;
            o[11] = 255;
        }
    });
    let mut albedo = Vec::with_capacity(n * n * 4);
    let mut color = Vec::with_capacity(n * n * 4);
    let mut normal = Vec::with_capacity(n * n * 4);
    for q in &px {
        albedo.extend_from_slice(&q[0..4]);
        color.extend_from_slice(&q[4..8]);
        normal.extend_from_slice(&q[8..12]);
    }
    Ok(Texture {
        size: p.size,
        height,
        albedo,
        color,
        normal,
    })
}

/// RGBA8 straight-alpha bytes as a PNG.
pub fn rgba_png(rgba: &[u8], size: u32) -> Result<Vec<u8>, String> {
    let sz = tiny_skia::IntSize::from_wh(size, size).ok_or("bad size")?;
    // every pixel is opaque, so straight and premultiplied alpha are the same bytes
    let pm = tiny_skia::Pixmap::from_vec(rgba.to_vec(), sz).ok_or("bad pixel buffer")?;
    pm.encode_png()
        .map_err(|e| format!("cannot encode PNG: {e}"))
}

/// The height field as a 16-bit grey PNG (65535 levels, no banding).
pub fn height_png16(t: &Texture) -> Result<Vec<u8>, String> {
    let mut f = Vec::with_capacity(t.height.len() * 4);
    for &h in &t.height {
        f.extend_from_slice(&[h, h, h, 1.0]);
    }
    crate::hdr::encode_png16_buf(t.size, t.size, &f, false)
}

// -------------------------------------------------------------- presets ----

/// Names of the ready-made looks.
pub const PRESETS: [&str; 14] = [
    "rock",
    "granite",
    "lava",
    "marble",
    "moss",
    "terrain",
    "clouds",
    "ice",
    "sand",
    "bark",
    "nebula",
    "mandelbrot",
    "julia",
    "burning-ship",
];

/// Parameters of a named preset, or `None` for an unknown name. The caller may override any field afterwards.
pub fn preset(name: &str) -> Option<Params> {
    let d = Params::default();
    let p = match name {
        "rock" => Params {
            kind: Kind::Crag,
            cells: 3,
            octaves: 12,
            gain: 0.52,
            relief: 3.0,
            specular: 0.12,
            shininess: 12.0,
            ao: 0.9,
            palette: Palette::hex(&[
                (0.0, 0x24211f),
                (0.35, 0x5b534b),
                (0.7, 0x8d8579),
                (1.0, 0xc9c1b3),
            ]),
            ..d
        },
        "granite" => Params {
            kind: Kind::Fbm,
            cells: 24,
            octaves: 8,
            gain: 0.62,
            relief: 1.2,
            specular: 0.45,
            shininess: 40.0,
            ao: 0.5,
            palette: Palette::hex(&[
                (0.0, 0x2b2b2e),
                (0.3, 0x6e6a6c),
                (0.55, 0xb9a9a2),
                (0.8, 0xd9d3cc),
                (1.0, 0xf1ece6),
            ]),
            ..d
        },
        "lava" => Params {
            kind: Kind::Cellular,
            cells: 5,
            octaves: 8,
            relief: 2.0,
            specular: 0.05,
            ao: 0.9,
            glow: 1.0,
            palette: Palette::hex(&[
                (0.0, 0xff5a0a),
                (0.12, 0xffa21a),
                (0.22, 0x5a1500),
                (0.5, 0x1d0e0a),
                (1.0, 0x3a312d),
            ]),
            ..d
        },
        "marble" => Params {
            kind: Kind::Marble,
            cells: 2,
            octaves: 9,
            warp: 1.6,
            relief: 0.4,
            specular: 0.7,
            shininess: 80.0,
            ao: 0.25,
            palette: Palette::hex(&[
                (0.0, 0x3a3c40),
                (0.35, 0xb8bbc0),
                (0.7, 0xf0efec),
                (1.0, 0xffffff),
            ]),
            ..d
        },
        "moss" => Params {
            kind: Kind::Warp,
            cells: 6,
            octaves: 12,
            gain: 0.58,
            warp: 1.0,
            relief: 3.0,
            specular: 0.08,
            shininess: 8.0,
            ao: 1.0,
            palette: Palette::hex(&[
                (0.0, 0x0e1a08),
                (0.4, 0x2e5a14),
                (0.75, 0x6a9a2a),
                (1.0, 0xb4c862),
            ]),
            ..d
        },
        "terrain" => Params {
            kind: Kind::Ridged,
            cells: 2,
            octaves: 14,
            gain: 0.5,
            relief: 3.2,
            specular: 0.04,
            ao: 0.8,
            palette: Palette::hex(&[
                (0.0, 0x1b3a5a),
                (0.22, 0x2f6a8a),
                (0.26, 0xd9c98f),
                (0.34, 0x5f8a3a),
                (0.62, 0x6d5a3e),
                (0.85, 0x9a948d),
                (1.0, 0xffffff),
            ]),
            ..d
        },
        "clouds" => Params {
            kind: Kind::Warp,
            cells: 3,
            octaves: 12,
            warp: 1.4,
            relief: 0.6,
            specular: 0.0,
            ao: 0.5,
            light_elev: 60.0,
            palette: Palette::hex(&[
                (0.0, 0x4d7fc4),
                (0.45, 0x9cc0ea),
                (0.75, 0xeef4fb),
                (1.0, 0xffffff),
            ]),
            ..d
        },
        "ice" => Params {
            kind: Kind::Cellular,
            cells: 6,
            octaves: 10,
            relief: 1.1,
            specular: 0.9,
            shininess: 90.0,
            ao: 0.4,
            palette: Palette::hex(&[
                (0.0, 0x103a5a),
                (0.3, 0x5aa8d6),
                (0.7, 0xbfe6f7),
                (1.0, 0xffffff),
            ]),
            ..d
        },
        "sand" => Params {
            kind: Kind::Billow,
            cells: 10,
            octaves: 12,
            gain: 0.48,
            relief: 1.6,
            specular: 0.04,
            shininess: 6.0,
            ao: 0.6,
            palette: Palette::hex(&[(0.0, 0x8c6a3c), (0.5, 0xc9a566), (1.0, 0xeed9a6)]),
            ..d
        },
        "bark" => Params {
            kind: Kind::Ridged,
            cells: 3,
            octaves: 12,
            gain: 0.55,
            warp: 1.0,
            relief: 3.2,
            specular: 0.05,
            ao: 1.0,
            palette: Palette::hex(&[
                (0.0, 0x120c08),
                (0.4, 0x3b2a1c),
                (0.75, 0x6a4f35),
                (1.0, 0x9b805e),
            ]),
            ..d
        },
        "nebula" => Params {
            kind: Kind::Warp,
            cells: 2,
            octaves: 14,
            gain: 0.55,
            warp: 2.4,
            relief: 0.2,
            specular: 0.0,
            ao: 0.0,
            glow: 0.0,
            light_elev: 85.0,
            palette: Palette::hex(&[
                (0.0, 0x03030c),
                (0.3, 0x2a0f5c),
                (0.55, 0xb8267a),
                (0.78, 0xff9a4a),
                (1.0, 0xfff3d0),
            ]),
            ..d
        },
        "mandelbrot" => Params {
            kind: Kind::Mandelbrot,
            tile: false,
            ssaa: 2,
            zoom: 400.0,
            relief: 0.8,
            specular: 0.5,
            shininess: 60.0,
            ao: 0.4,
            palette: Palette::hex(&[
                (0.0, 0x050a2e),
                (0.25, 0x1f5fbf),
                (0.5, 0xf3f0d0),
                (0.75, 0xf2a21d),
                (1.0, 0x3a0a02),
            ]),
            ..d
        },
        "julia" => Params {
            kind: Kind::Julia,
            tile: false,
            ssaa: 2,
            zoom: 1.2,
            relief: 0.7,
            specular: 0.5,
            shininess: 50.0,
            ao: 0.4,
            palette: Palette::hex(&[
                (0.0, 0x10002b),
                (0.3, 0x7b2cbf),
                (0.6, 0xff6d9d),
                (0.85, 0xffd6a5),
                (1.0, 0x240046),
            ]),
            ..d
        },
        "burning-ship" => Params {
            kind: Kind::BurningShip,
            tile: false,
            ssaa: 2,
            zoom: 7.0,
            relief: 0.8,
            specular: 0.6,
            shininess: 70.0,
            ao: 0.4,
            palette: Palette::hex(&[
                (0.0, 0x0b0300),
                (0.3, 0x7a1a00),
                (0.6, 0xff7a00),
                (0.85, 0xffe08a),
                (1.0, 0x0b0300),
            ]),
            ..d
        },
        _ => return None,
    };
    Some(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small(name: &str) -> Params {
        Params {
            size: 64,
            ssaa: 1,
            ..preset(name).unwrap()
        }
    }

    #[test]
    fn same_parameters_give_the_same_bytes_and_other_seeds_differ() {
        let a = generate(&small("rock")).unwrap();
        let b = generate(&small("rock")).unwrap();
        assert_eq!(a.color, b.color);
        assert_eq!(a.normal, b.normal);
        let c = generate(&Params {
            seed: 2,
            ..small("rock")
        })
        .unwrap();
        assert_ne!(a.color, c.color);
    }

    #[test]
    fn every_preset_makes_a_textured_image_in_range() {
        for name in PRESETS {
            let t = generate(&small(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(t.color.len(), 64 * 64 * 4, "{name}");
            let n = t.height.len() as f32;
            let mean = t.height.iter().sum::<f32>() / n;
            let var = t
                .height
                .iter()
                .map(|h| (h - mean) * (h - mean))
                .sum::<f32>()
                / n;
            assert!(
                t.height.iter().all(|h| (0.0..=1.0).contains(h)),
                "{name}: height out of range"
            );
            assert!(var > 1e-4, "{name}: flat, variance {var}");
            assert!(t.color.chunks(4).all(|p| p[3] == 255), "{name}: opaque");
        }
    }

    #[test]
    fn tileable_noise_repeats_exactly_at_the_edges() {
        for kind in [
            Kind::Fbm,
            Kind::Ridged,
            Kind::Billow,
            Kind::Warp,
            Kind::Cellular,
            Kind::Marble,
            Kind::Crag,
        ]
        .into_iter()
        .flat_map(|k| [1u32, 2, 3, 5, 7, 24].map(|c| (k, c)))
        {
            let (kind, cells) = kind;
            let p = Params {
                kind,
                cells,
                size: 64,
                tile: true,
                ..Params::default()
            };
            let f = Field::new(&p);
            for i in 0..16 {
                let t = i as f32 / 16.0 + 0.013;
                assert!(
                    (f.height(0.0, t) - f.height(1.0, t)).abs() < 1e-3,
                    "{kind:?} cells {cells} u"
                );
                assert!(
                    (f.height(t, 0.0) - f.height(t, 1.0)).abs() < 1e-3,
                    "{kind:?} cells {cells} v"
                );
            }
        }
    }

    #[test]
    fn a_tiled_texture_has_no_seam_in_its_normal_map() {
        let t = generate(&Params {
            size: 64,
            ..preset("rock").unwrap()
        })
        .unwrap();
        // the wrapped neighbour of the last column is the first one: the jump across the seam is as small as inside
        let n = 64usize;
        let jump = |a: usize, b: usize| {
            (0..n)
                .map(|y| (t.height[y * n + a] - t.height[y * n + b]).abs())
                .sum::<f32>()
                / n as f32
        };
        let inside = (1..n).map(|x| jump(x - 1, x)).sum::<f32>() / (n - 1) as f32;
        assert!(
            jump(n - 1, 0) < inside * 2.5,
            "seam {} vs interior {}",
            jump(n - 1, 0),
            inside
        );
    }

    #[test]
    fn extra_octaves_stop_at_the_nyquist_limit() {
        let p = Params {
            size: 64,
            octaves: 16,
            cells: 4,
            ..Params::default()
        };
        let f = Field::new(&p);
        assert!(
            f.octaves <= 6,
            "64 px with 4 cells keeps at most 5-6 octaves, got {}",
            f.octaves
        );
        let big = Field::new(&Params { size: 2048, ..p });
        assert!(big.octaves > f.octaves);
    }

    #[test]
    fn flat_height_gives_the_flat_normal_colour() {
        let h = vec![0.5f32; 16 * 16];
        let n = 16usize;
        let at = |x: isize, y: isize| h[(y.clamp(0, 15) as usize) * n + x.clamp(0, 15) as usize];
        let dx = at(1, 0) - at(-1, 0);
        assert_eq!(dx, 0.0);
        let t = generate(&Params {
            kind: Kind::Fbm,
            size: MIN_SIZE,
            contrast: 0.0,
            ..Params::default()
        })
        .unwrap();
        assert!(
            t.normal
                .chunks(4)
                .all(|p| p[0] == 128 && p[1] == 128 && p[2] == 255),
            "a flat surface points straight at the viewer"
        );
    }

    #[test]
    fn limits_are_errors_not_panics() {
        assert!(generate(&Params {
            size: 4,
            ..Params::default()
        })
        .is_err());
        assert!(generate(&Params {
            size: 5000,
            ..Params::default()
        })
        .is_err());
        assert!(generate(&Params {
            size: 4096,
            ssaa: 4,
            ..Params::default()
        })
        .unwrap_err()
        .contains("too many samples"));
        assert!(generate(&Params {
            relief: f32::NAN,
            ..Params::default()
        })
        .is_err());
        assert!(preset("nope").is_none());
    }

    #[test]
    fn escape_time_has_an_inside_and_smooth_outside() {
        let t = generate(&Params {
            size: 64,
            ..preset("mandelbrot").unwrap()
        })
        .unwrap();
        let distinct: std::collections::HashSet<u32> =
            t.height.iter().map(|h| (h * 4096.0) as u32).collect();
        assert!(
            distinct.len() > 200,
            "smooth colouring gives many levels, got {}",
            distinct.len()
        );
        let m = generate(&Params {
            size: 32,
            zoom: 1.0,
            ..preset("mandelbrot").unwrap()
        })
        .unwrap();
        assert!(
            m.height.contains(&0.0),
            "the whole set shows the black interior"
        );
    }

    #[test]
    fn png_encoders_round_trip() {
        let t = generate(&small("marble")).unwrap();
        let png = rgba_png(&t.color, 64).unwrap();
        let back = tiny_skia::Pixmap::decode_png(&png).unwrap();
        assert_eq!((back.width(), back.height()), (64, 64));
        assert!(height_png16(&t)
            .unwrap()
            .starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn octave_seeds_do_not_overlap_between_seeds() {
        // the old `seed + o * 0x9E37` made seed s octave 1 equal to seed s + 0x9E37 octave 0
        let mut seen = std::collections::HashSet::new();
        for seed in [0u32, 1, 2, 0x9E37, 0x9E37 + 1, 12345] {
            for o in 0..16 {
                assert!(
                    seen.insert(octave_seed(seed, o)),
                    "seed {seed} octave {o} repeats another"
                );
            }
        }
        let a = generate(&Params {
            size: 64,
            seed: 1,
            ..preset("rock").unwrap()
        })
        .unwrap();
        let b = generate(&Params {
            size: 64,
            seed: 1 + 0x9E37,
            ..preset("rock").unwrap()
        })
        .unwrap();
        let same = a
            .height
            .iter()
            .zip(&b.height)
            .filter(|(x, y)| (*x - *y).abs() < 1e-6)
            .count();
        assert!(
            same < a.height.len() / 20,
            "heights of unrelated seeds agree at {same} pixels"
        );
    }

    #[test]
    fn the_escape_view_is_continuous_in_zoom() {
        let near = |z: f64| {
            Escape::new(&Params {
                kind: Kind::Mandelbrot,
                zoom: z,
                ..Params::default()
            })
        };
        let (a, b) = (near(1.0), near(1.0001));
        assert!(
            (a.cx - b.cx).abs() < 1e-3 && (a.cy - b.cy).abs() < 1e-3,
            "view jumps between zoom 1 and 1.0001"
        );
        let c = near(8.0);
        assert!(
            (c.cx - MANDEL_VIEWS[1].0).abs() < 1e-12,
            "from zoom 8 on the view is the seed's target"
        );
    }

    #[test]
    fn an_unbounded_escape_request_is_refused_before_any_work() {
        let r = generate(&Params {
            kind: Kind::Mandelbrot,
            tile: false,
            size: 4096,
            ssaa: 2,
            zoom: 1e12,
            ..Params::default()
        });
        assert!(r.unwrap_err().contains("too many iterations"));
    }

    #[test]
    fn supersampled_boundary_pixels_blend_colours_not_palette_positions() {
        // across the set's edge every albedo must be a mix of the two sides, never a colour from elsewhere on the ramp
        let p = Params {
            size: 96,
            ssaa: 3,
            zoom: 1.0,
            ..preset("mandelbrot").unwrap()
        };
        let t = generate(&p).unwrap();
        let dark = lin_to_srgb(0.012);
        let inside = t
            .albedo
            .chunks(4)
            .filter(|q| (q[0] as f32 / 255.0 - dark).abs() < 0.01)
            .count();
        assert!(inside > 0 && inside < 96 * 96);
    }

    #[test]
    fn random_parameters_never_panic_or_produce_non_finite_maps() {
        let mut r = crate::noise::Rng::new(7);
        for _ in 0..60 {
            let kind = Kind::ALL[(r.f32() * 10.0) as usize % Kind::ALL.len()];
            let p = Params {
                kind,
                size: 8 + (r.f32() * 40.0) as u32,
                seed: (r.f32() * 4e9) as u32,
                octaves: (r.f32() * 20.0) as u32,
                lacunarity: r.f32() * 6.0,
                gain: r.f32() * 1.2,
                cells: (r.f32() * 300.0) as u32,
                warp: (r.f32() - 0.3) * 40.0,
                tile: r.f32() > 0.5,
                zoom: (r.f32() * 30.0 - 5.0) as f64,
                ssaa: 1 + (r.f32() * 3.0) as u32,
                contrast: (r.f32() - 0.3) * 30.0,
                relief: (r.f32() - 0.3) * 50.0,
                light_az: r.f32() * 720.0 - 360.0,
                light_elev: r.f32() * 200.0 - 50.0,
                specular: r.f32() * 3.0,
                shininess: r.f32() * 500.0 - 20.0,
                ao: r.f32() * 3.0,
                glow: r.f32() * 5.0,
                color_speed: r.f32() * 20.0 - 5.0,
                palette: Palette(vec![]),
            };
            let t = generate(&p).unwrap_or_else(|e| panic!("{p:?}: {e}"));
            assert!(
                t.height
                    .iter()
                    .all(|h| h.is_finite() && (0.0..=1.0).contains(h)),
                "{p:?}"
            );
        }
    }

    #[test]
    fn worley_points_are_not_aligned_in_columns_or_rows() {
        // the first version drew the x offset from the column alone, so every cell of a column shared it
        let off = |cx: i32, cy: i32| {
            let a = (hash01(cx, 3) * 16_777_216.0) as i32;
            (
                hash01(a ^ cy.wrapping_mul(0x1B87_3593), 3 ^ 0x3C6E_F372),
                hash01(a ^ cy.wrapping_mul(0x2545_F491), 3 ^ 0xA54F_F53A),
            )
        };
        let col: std::collections::HashSet<u32> =
            (0..32).map(|y| (off(5, y).0 * 1e6) as u32).collect();
        let row: std::collections::HashSet<u32> =
            (0..32).map(|x| (off(x, 5).1 * 1e6) as u32).collect();
        assert!(col.len() > 28 && row.len() > 28);
        let (f1, _) = worley(0.5, 0.5, 0, 9);
        assert!(f1.is_finite() && f1 < 1.5);
    }
}
