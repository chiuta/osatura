// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Frame post-effects on straight-alpha RGBA8 buffers: blur, drop shadow, glow, outline,
//! posterise, pixelate, levels/gamma, fade. A chain is plain JSON, applied after rasterising.
//!
//! Clean-room note: textbook image processing (separable Gaussian, morphological dilation,
//! per-channel look-up maths). Parameters and names are our own.

use crate::style::Rgba;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Fx {
    /// Gaussian blur, radius in output pixels (σ = radius / 2).
    Blur { radius: f32 },
    DropShadow { dx: f32, dy: f32, blur: f32, color: String, opacity: f32 },
    /// Soft additive halo of the artwork's own colours (or `color`).
    Glow { radius: f32, strength: f32, #[serde(default)] color: Option<String> },
    /// Solid outline drawn behind the artwork, `width` px.
    Outline { width: f32, color: String },
    Posterize { levels: u32 },
    Pixelate { size: u32 },
    Levels { in_black: f32, in_white: f32, gamma: f32, #[serde(default)] out_black: f32, #[serde(default = "one")] out_white: f32 },
    /// Multiply alpha by (1 − amount).
    Fade { amount: f32 },
    /// Lambert lighting from a bevel heightfield made of the blurred alpha channel (own design,
    /// no normal map needed). `bevel` = softness radius in px, `height` = how steep the bevel is.
    /// With `normal_map` the shading normal comes from that tangent-space map instead of the bevel
    /// (the blurred alpha still drives shadows and the ground shadow). `ambient_color` tints the
    /// ambient term (default white).
    Relight {
        ambient: f32,
        bevel: f32,
        height: f32,
        lights: Vec<Light>,
        #[serde(default)]
        ground_shadow: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        normal_map: Option<NormalMap>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ambient_color: Option<String>,
    },
}

/// How a normal map of another size is fitted to the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalFit {
    /// Bilinearly scaled to the frame size (default).
    #[default]
    Stretch,
    /// Repeated from the top-left corner, texel for texel.
    Tile,
    /// Must have exactly the frame size, otherwise an error.
    Exact,
}

/// A tangent-space normal map: RGB 0..255 decodes to -1..1 (x right, z toward the viewer).
/// Green is "up on screen" (the common OpenGL convention) unless `green_down` is set.
/// Give the PNG as base64 or a `data:image/png;base64,` URI in `data`, or a file in `path`
/// (the CLI reads it relative to the chain JSON; `resolve_normal_map_paths` does the same).
/// As the `relight` effect's `normal_map` it applies to the whole frame; as `Layer::normal_map` it is
/// fitted to the pixel bounding box of that layer (see [`layer_normal_field`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NormalMap {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default)]
    pub fit: NormalFit,
    /// Scales the x/y tilt before renormalising (1 = as stored, 0 = flat).
    #[serde(default = "one")]
    pub strength: f32,
    #[serde(default)]
    pub green_down: bool,
}

/// A cone limiting a light: `angle` is the half-angle in degrees (0..90], `soft` the fraction of
/// it (0..1) over which the edge fades with a smoothstep; the direction need not be normalised.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spot {
    pub dir_x: f32,
    pub dir_y: f32,
    pub dir_z: f32,
    pub angle: f32,
    #[serde(default)]
    pub soft: f32,
}

/// A point light above the image plane (`z` px high). `radius` 0 = no distance falloff.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Light {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(default = "white")]
    pub color: String,
    #[serde(default = "one")]
    pub intensity: f32,
    #[serde(default)]
    pub radius: f32,
    /// Ray-march the bevel heightfield toward this light: shadowed parts get no direct light.
    #[serde(default)]
    pub cast_shadow: bool,
    /// Turns the point light into a spot light.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spot: Option<Spot>,
    /// Distance falloff exponent (used with `radius`): (1 - d/radius)^falloff.
    #[serde(default = "two")]
    pub falloff: f32,
}

fn two() -> f32 {
    2.0
}

fn white() -> String {
    "#ffffff".into()
}

fn one() -> f32 {
    1.0
}

const MAX_PIXELS: usize = 16_000_000;
const MAX_RADIUS: f32 = 64.0;

fn hex(c: &str) -> Result<[f32; 3], String> {
    let p = Rgba::parse(c).ok_or_else(|| format!("bad colour '{c}'"))?;
    Ok([p.r, p.g, p.b])
}

/// Premultiplied float working copy.
fn to_premul(rgba: &[u8]) -> Vec<f32> {
    let mut v = Vec::with_capacity(rgba.len());
    for p in rgba.chunks_exact(4) {
        let a = p[3] as f32 / 255.0;
        v.extend_from_slice(&[p[0] as f32 / 255.0 * a, p[1] as f32 / 255.0 * a, p[2] as f32 / 255.0 * a, a]);
    }
    v
}

fn from_premul(v: &[f32], out: &mut [u8]) {
    for (o, p) in out.chunks_exact_mut(4).zip(v.chunks_exact(4)) {
        let a = p[3].clamp(0.0, 1.0);
        let un = |c: f32| if a > 1e-6 { (c / a).clamp(0.0, 1.0) } else { 0.0 };
        o[0] = (un(p[0]) * 255.0).round() as u8;
        o[1] = (un(p[1]) * 255.0).round() as u8;
        o[2] = (un(p[2]) * 255.0).round() as u8;
        o[3] = (a * 255.0).round() as u8;
    }
}

fn kernel(radius: f32) -> Vec<f32> {
    let sigma = (radius * 0.5).max(0.3);
    let r = (sigma * 3.0).ceil() as i32;
    let mut k: Vec<f32> = (-r..=r).map(|i| (-(i * i) as f32 / (2.0 * sigma * sigma)).exp()).collect();
    let s: f32 = k.iter().sum();
    k.iter_mut().for_each(|x| *x /= s);
    k
}

/// Pixels the blur of `radius` reaches beyond a shape's edge (the kernel half width).
pub(crate) fn blur_extent(radius: f32) -> usize {
    if radius <= 0.0 { 0 } else { kernel(radius).len() / 2 }
}

pub(crate) fn blur_premul(src: &[f32], w: usize, h: usize, radius: f32) -> Vec<f32> {
    if radius <= 0.0 {
        return src.to_vec();
    }
    let k = kernel(radius);
    let r = (k.len() / 2) as i32;
    let mut tmp = vec![0.0f32; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for (i, kv) in k.iter().enumerate() {
                let xx = (x as i32 + i as i32 - r).clamp(0, w as i32 - 1) as usize;
                let o = (y * w + xx) * 4;
                // pixels outside the frame count as transparent, so edges fade rather than smear
                let inside = (x as i32 + i as i32 - r) >= 0 && (x as i32 + i as i32 - r) < w as i32;
                if inside {
                    for c in 0..4 {
                        acc[c] += kv * src[o + c];
                    }
                }
            }
            tmp[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&acc);
        }
    }
    let mut out = vec![0.0f32; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for (i, kv) in k.iter().enumerate() {
                let yy = y as i32 + i as i32 - r;
                if yy >= 0 && yy < h as i32 {
                    let o = (yy as usize * w + x) * 4;
                    for c in 0..4 {
                        acc[c] += kv * tmp[o + c];
                    }
                }
            }
            out[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&acc);
        }
    }
    out
}

/// `top` over `bottom`, both premultiplied.
pub(crate) fn over(top: &[f32], bottom: &[f32]) -> Vec<f32> {
    top.chunks_exact(4)
        .zip(bottom.chunks_exact(4))
        .flat_map(|(t, b)| {
            let k = 1.0 - t[3];
            [t[0] + b[0] * k, t[1] + b[1] * k, t[2] + b[2] * k, t[3] + b[3] * k]
        })
        .collect()
}

pub(crate) fn shift(src: &[f32], w: usize, h: usize, dx: i32, dy: i32) -> Vec<f32> {
    let mut out = vec![0.0f32; src.len()];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let (sx, sy) = (x - dx, y - dy);
            if sx >= 0 && sy >= 0 && sx < w as i32 && sy < h as i32 {
                let (o, s) = ((y as usize * w + x as usize) * 4, (sy as usize * w + sx as usize) * 4);
                out[o..o + 4].copy_from_slice(&src[s..s + 4]);
            }
        }
    }
    out
}

pub(crate) fn solid_from_alpha(alpha: &[f32], color: [f32; 3], opacity: f32) -> Vec<f32> {
    alpha.iter().flat_map(|&a| { let a = (a * opacity).clamp(0.0, 1.0); [color[0] * a, color[1] * a, color[2] * a, a] }).collect()
}

pub(crate) fn alpha_of(p: &[f32]) -> Vec<f32> {
    p.chunks_exact(4).map(|c| c[3]).collect()
}

fn dilate(alpha: &[f32], w: usize, h: usize, r: i32) -> Vec<f32> {
    let offs: Vec<(i32, i32)> = (-r..=r).flat_map(|y| (-r..=r).map(move |x| (x, y))).filter(|(x, y)| x * x + y * y <= r * r).collect();
    let mut out = vec![0.0f32; alpha.len()];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let mut m = 0.0f32;
            for &(ox, oy) in &offs {
                let (sx, sy) = (x + ox, y + oy);
                if sx >= 0 && sy >= 0 && sx < w as i32 && sy < h as i32 {
                    m = m.max(alpha[sy as usize * w + sx as usize]);
                }
            }
            out[y as usize * w + x as usize] = m;
        }
    }
    out
}

fn check_params(fx: &Fx) -> Result<(), String> {
    let finite = |v: f32| v.is_finite();
    let ok = match fx {
        Fx::Blur { radius } => finite(*radius) && (0.0..=MAX_RADIUS).contains(radius),
        Fx::DropShadow { dx, dy, blur, opacity, .. } => [dx, dy, blur, opacity].iter().all(|v| finite(**v)) && (0.0..=MAX_RADIUS).contains(blur) && (0.0..=1.0).contains(opacity) && dx.abs() <= 512.0 && dy.abs() <= 512.0,
        Fx::Glow { radius, strength, .. } => finite(*radius) && finite(*strength) && (0.0..=MAX_RADIUS).contains(radius) && (0.0..=8.0).contains(strength),
        Fx::Outline { width, .. } => finite(*width) && (0.0..=16.0).contains(width),
        Fx::Posterize { levels } => (2..=256).contains(levels),
        Fx::Pixelate { size } => (1..=256).contains(size),
        Fx::Levels { in_black, in_white, gamma, out_black, out_white } => {
            [in_black, in_white, gamma, out_black, out_white].iter().all(|v| finite(**v)) && in_white > in_black && *gamma > 0.0
        }
        Fx::Fade { amount } => finite(*amount) && (0.0..=1.0).contains(amount),
        Fx::Relight { ambient, bevel, height, lights, ground_shadow, normal_map, ambient_color } => {
            ambient_color.as_ref().is_none_or(|c| Rgba::parse(c).is_some())
                && normal_map.as_ref().is_none_or(|n| finite(n.strength) && (0.0..=8.0).contains(&n.strength) && (n.data.is_some() || n.path.is_some()))
                && lights.iter().all(|l| finite(l.falloff) && (0.1..=8.0).contains(&l.falloff) && l.spot.as_ref().is_none_or(|s| {
                    [s.dir_x, s.dir_y, s.dir_z, s.angle, s.soft].iter().all(|v| finite(*v)) && s.angle > 0.0 && s.angle <= 90.0 && (0.0..=1.0).contains(&s.soft) && s.dir_x * s.dir_x + s.dir_y * s.dir_y + s.dir_z * s.dir_z > 1e-12
                }))
                && finite(*ground_shadow) && (0.0..=1.0).contains(ground_shadow) && finite(*ambient) && (0.0..=4.0).contains(ambient) && finite(*bevel) && (0.0..=MAX_RADIUS).contains(bevel) && finite(*height) && (0.0..=64.0).contains(height) && lights.len() <= 16
                && lights.iter().all(|l| [l.x, l.y, l.intensity, l.radius].iter().all(|v| finite(*v)) && l.z.is_finite() && l.z > 0.0 && l.intensity >= 0.0 && l.radius >= 0.0)
        }
    };
    if ok { Ok(()) } else { Err(format!("invalid parameters for {fx:?}")) }
}

const MAX_NORMAL_B64: usize = 48_000_000;

/// Decode a normal-map PNG to unit vectors in screen space (x right, y down, z toward viewer).
fn decode_normal_png(nm: &NormalMap) -> Result<(usize, usize, Vec<[f32; 3]>), String> {
    let data = nm.data.as_deref().ok_or("normal map has only a path; resolve it first (CLI does, or resolve_normal_map_paths)")?;
    let b64 = match data.strip_prefix("data:") {
        Some(rest) => rest.split_once("base64,").ok_or("normal map data URI must be base64")?.1,
        None => data,
    };
    if b64.len() > MAX_NORMAL_B64 {
        return Err("normal map is too large".into());
    }
    let bytes = crate::base64::decode(b64.trim())?;
    crate::assets::check_png(&bytes).map_err(|e| format!("normal map: {e}"))?;
    let mut dec = png::Decoder::new(std::io::Cursor::new(&bytes[..]));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut r = dec.read_info().map_err(|e| format!("normal map is not a valid PNG: {e}"))?;
    let (mw, mh) = (r.info().width as usize, r.info().height as usize);
    if mw == 0 || mh == 0 || mw * mh > MAX_PIXELS {
        return Err("normal map is empty or too large".into());
    }
    let mut buf = vec![0u8; r.output_buffer_size().ok_or("normal map is too large")?];
    let info = r.next_frame(&mut buf).map_err(|e| format!("normal map is not a valid PNG: {e}"))?;
    let ch = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => return Err("normal map must be an RGB or RGBA PNG".into()),
    };
    if buf.len() < mw * mh * ch {
        return Err("normal map data is truncated".into());
    }
    let sy = if nm.green_down { 1.0 } else { -1.0 };
    let mut out = Vec::with_capacity(mw * mh);
    for p in buf.chunks_exact(ch).take(mw * mh) {
        let v = [p[0] as f32 / 255.0 * 2.0 - 1.0, (p[1] as f32 / 255.0 * 2.0 - 1.0) * sy, p[2] as f32 / 255.0 * 2.0 - 1.0];
        out.push(unit_tilt(v, nm.strength));
    }
    Ok((mw, mh, out))
}

fn unit_tilt(v: [f32; 3], strength: f32) -> [f32; 3] {
    let (x, y, z) = (v[0] * strength, v[1] * strength, v[2]);
    let l = (x * x + y * y + z * z).sqrt();
    if l < 1e-3 { [0.0, 0.0, 1.0] } else { [x / l, y / l, z / l] }
}

/// A normal map as one unit vector per frame pixel (`w × h`).
fn fit_normal_map(nm: &NormalMap, w: usize, h: usize) -> Result<Vec<[f32; 3]>, String> {
    let (mw, mh, src) = decode_normal_png(nm)?;
    let mut out = Vec::with_capacity(w * h);
    match nm.fit {
        NormalFit::Exact => {
            if (mw, mh) != (w, h) {
                return Err(format!("normal map is {mw}x{mh} but the frame is {w}x{h} (fit is exact)"));
            }
            out = src;
        }
        NormalFit::Tile => {
            for y in 0..h {
                for x in 0..w {
                    out.push(src[(y % mh) * mw + x % mw]);
                }
            }
        }
        NormalFit::Stretch => {
            for y in 0..h {
                let fy = ((y as f32 + 0.5) * mh as f32 / h as f32 - 0.5).clamp(0.0, (mh - 1) as f32);
                let (y0, ty) = (fy.floor() as usize, fy.fract());
                let y1 = (y0 + 1).min(mh - 1);
                for x in 0..w {
                    let fx = ((x as f32 + 0.5) * mw as f32 / w as f32 - 0.5).clamp(0.0, (mw - 1) as f32);
                    let (x0, tx) = (fx.floor() as usize, fx.fract());
                    let x1 = (x0 + 1).min(mw - 1);
                    let mut v = [0.0f32; 3];
                    for k in 0..3 {
                        let top = src[y0 * mw + x0][k] * (1.0 - tx) + src[y0 * mw + x1][k] * tx;
                        let bot = src[y1 * mw + x0][k] * (1.0 - tx) + src[y1 * mw + x1][k] * tx;
                        v[k] = top * (1.0 - ty) + bot * ty;
                    }
                    out.push(unit_tilt(v, 1.0));
                }
            }
        }
    }
    Ok(out)
}

/// Replace every `normal_map.path` in the chain with the file's base64 contents (relative paths
/// are resolved against `base`). Fails on unreadable files.
pub fn resolve_normal_map_paths(chain: &mut [Fx], base: &std::path::Path) -> Result<(), String> {
    for fx in chain {
        if let Fx::Relight { normal_map: Some(nm), .. } = fx {
            if let Some(p) = nm.path.take() {
                let bytes = crate::assets::read(base, &p, crate::assets::MAX_IMAGE_BYTES, "normal map")?;
                nm.data = Some(crate::base64::encode(&bytes));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Per-layer normal maps
// ---------------------------------------------------------------------------------------------

/// Most leaf layers that [`layer_normal_field`] will render one by one.
pub const MAX_NORMAL_LAYERS: usize = 512;

fn check_layer_nm(nm: &NormalMap) -> bool {
    nm.strength.is_finite() && (0.0..=8.0).contains(&nm.strength) && (nm.data.is_some() || nm.path.is_some())
}

/// Problems with the `normal_map` of layers (parameters only; the PNG is decoded when used).
pub fn validate_layer_normal_maps(p: &crate::project::Project) -> Vec<String> {
    fn walk(ls: &[crate::project::Layer], prefix: &str, errs: &mut Vec<String>) {
        for l in ls {
            let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
            if let Some(nm) = &l.normal_map {
                if !check_layer_nm(nm) {
                    errs.push(format!("layer '{path}': normal map needs data or path and a strength within 0..=8"));
                }
            }
            if let crate::project::LayerKind::Group { layers } | crate::project::LayerKind::Selector { layers, .. } = &l.kind {
                walk(layers, &path, errs);
            }
        }
    }
    let mut errs = Vec::new();
    walk(&p.layers, "", &mut errs);
    errs
}

/// `(path, map)` of every leaf layer in paint order; the map is the layer's own or the nearest
/// ancestor's.
fn leaves_with_maps(p: &crate::project::Project) -> Vec<(String, Option<NormalMap>)> {
    fn walk(ls: &[crate::project::Layer], prefix: &str, inherited: Option<&NormalMap>, out: &mut Vec<(String, Option<NormalMap>)>) {
        for l in ls {
            let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
            let here = l.normal_map.as_ref().or(inherited);
            match &l.kind {
                crate::project::LayerKind::Group { layers } | crate::project::LayerKind::Selector { layers, .. } => walk(layers, &path, here, out),
                _ => out.push((path, here.cloned())),
            }
        }
    }
    let mut out = Vec::new();
    walk(&p.layers, "", None, &mut out);
    out
}

/// Per-pixel shading normals from the layers' own normal maps, for a render of `frame` at `scale`.
///
/// Every leaf layer is rendered alone (the others hidden) to find the pixels it covers (alpha at
/// least 50 %). Going through the layers in paint order, a layer with a normal map (its own or its
/// nearest ancestor's) writes its normals there and a layer without one clears them, so the topmost
/// layer decides and a plain layer in front of a mapped one hides it. The map is fitted
/// (`fit`: stretch / tile / exact) to the axis-aligned pixel bounding box of the layer's coverage;
/// it does not turn with a rotated layer. Returns `None` when no layer has a map.
/// Relative `path`s resolve against `asset_dir`. Layers hidden by a static `visible: false` are
/// skipped; opacity and blend modes are not considered.
pub fn layer_normal_field(engine: &crate::engine::Engine, frame: f32, scale: f32, asset_dir: &std::path::Path) -> Result<Option<NormalField>, String> {
    let leaves = leaves_with_maps(&engine.project);
    let Some(first) = leaves.iter().position(|(_, m)| m.is_some()) else { return Ok(None) };
    if leaves.len() > MAX_NORMAL_LAYERS {
        return Err(format!("more than {MAX_NORMAL_LAYERS} layers: per-layer normal maps render each layer separately"));
    }
    let opts = crate::raster::RasterOptions { scale, show_bones: false, transparent: true };
    let (mut fw, mut fh) = (0usize, 0usize);
    let mut normals: Vec<Option<[f32; 3]>> = Vec::new();
    for (path, nm) in leaves.iter().skip(first) {
        let mut q = engine.project.clone();
        for (other, _) in &leaves {
            if other != path {
                if let Some(l) = q.find_layer_mut(other) {
                    l.visible = false;
                }
            }
        }
        let solo = crate::engine::Engine::with_asset_dir(q, asset_dir).map_err(|e| format!("layer '{path}': {}", e.join("; ")))?;
        let (w, h, px) = crate::raster::render_rgba(&solo, frame, &opts);
        let (w, h) = (w as usize, h as usize);
        if normals.is_empty() {
            (fw, fh) = (w, h);
            normals = vec![None; w * h];
        }
        let covered = |i: usize| px[i * 4 + 3] >= 128;
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
        for y in 0..h {
            for x in 0..w {
                if covered(y * w + x) {
                    (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                }
            }
        }
        if x0 == usize::MAX {
            continue;
        }
        match nm {
            None => {
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        if covered(y * w + x) {
                            normals[y * w + x] = None;
                        }
                    }
                }
            }
            Some(nm) => {
                let mut nm = nm.clone();
                if let Some(rel) = nm.path.take() {
                    if nm.data.is_none() {
                        let bytes = crate::assets::read(asset_dir, &rel, crate::assets::MAX_IMAGE_BYTES, "normal map").map_err(|e| format!("layer '{path}': {e}"))?;
                        nm.data = Some(crate::base64::encode(&bytes));
                    }
                }
                let (bw, bh) = (x1 - x0 + 1, y1 - y0 + 1);
                let fitted = fit_normal_map(&nm, bw, bh).map_err(|e| format!("layer '{path}': {e}"))?;
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        if covered(y * w + x) {
                            normals[y * w + x] = Some(fitted[(y - y0) * bw + (x - x0)]);
                        }
                    }
                }
            }
        }
    }
    Ok(Some(NormalField { w: fw, h: fh, normals }))
}

/// Render `frame` and apply `chain`, using the layers' own normal maps in every `relight`.
/// Without any layer normal map this equals rendering and calling [`apply`].
pub fn render_with_layer_normals(engine: &crate::engine::Engine, frame: f32, scale: f32, chain: &[Fx], asset_dir: &std::path::Path) -> Result<(u32, u32, Vec<u8>), String> {
    let (w, h, mut px) = crate::raster::render_rgba(engine, frame, &crate::raster::RasterOptions { scale, show_bones: false, transparent: false });
    let field = layer_normal_field(engine, frame, scale, asset_dir)?;
    apply_with_normals(chain, w as usize, h as usize, &mut px, field.as_ref())?;
    Ok((w, h, px))
}

/// True when the heightfield blocks the straight line from pixel `(x, y)` (surface height `z0`) to
/// a point light `(dx, dy)` pixels away and `lz` high. Steps one pixel at a time, at most 96.
// hot per-pixel helper; the scalars are independent ray parameters, bundling would only add copies.
#[allow(clippy::too_many_arguments)]
fn shadow_ray(at: &dyn Fn(i32, i32) -> f32, x: i32, y: i32, z0: f32, dx: f32, dy: f32, lz: f32, height: f32) -> bool {
    let d2 = (dx * dx + dy * dy).sqrt();
    if d2 < 1.0 {
        return false;
    }
    let (ux, uy) = (dx / d2, dy / d2);
    let steps = (d2.floor() as i32).min(96);
    for s in 1..=steps {
        let t = s as f32;
        let ray_z = z0 + (lz - z0) * t / d2;
        let sx = (x as f32 + ux * t).round() as i32;
        let sy = (y as f32 + uy * t).round() as i32;
        if at(sx, sy) * height > ray_z + 0.05 {
            return true;
        }
    }
    false
}

/// Apply a chain to a straight-alpha RGBA8 buffer of `w × h`, in place.
pub fn apply(chain: &[Fx], w: usize, h: usize, rgba: &mut [u8]) -> Result<(), String> {
    apply_with_normals(chain, w, h, rgba, None)
}

/// Per-pixel shading normals that override the `relight` effect's own (see [`layer_normal_field`]).
#[derive(Clone, Debug, PartialEq)]
pub struct NormalField {
    pub w: usize,
    pub h: usize,
    /// One entry per pixel, row-major: a unit vector (x right, y down, z toward the viewer), or
    /// `None` to use the effect's global normal map or bevel there.
    pub normals: Vec<Option<[f32; 3]>>,
}

/// Like [`apply`], with per-pixel normals from the layers' own normal maps for every `relight`.
pub fn apply_with_normals(chain: &[Fx], w: usize, h: usize, rgba: &mut [u8], field: Option<&NormalField>) -> Result<(), String> {
    if let Some(f) = field {
        if f.w != w || f.h != h || f.normals.len() != w * h {
            return Err("normal field does not match the frame size".into());
        }
    }
    if rgba.len() != w * h * 4 || w * h == 0 || w * h > MAX_PIXELS {
        return Err("bad buffer size".into());
    }
    for fx in chain {
        check_params(fx)?;
        match fx {
            Fx::Blur { radius } => {
                let b = blur_premul(&to_premul(rgba), w, h, *radius);
                from_premul(&b, rgba);
            }
            Fx::DropShadow { dx, dy, blur, color, opacity } => {
                let src = to_premul(rgba);
                let a = shift(&alpha_of(&src).iter().flat_map(|&a| [0.0, 0.0, 0.0, a]).collect::<Vec<_>>(), w, h, dx.round() as i32, dy.round() as i32);
                let a = blur_premul(&a, w, h, *blur);
                let shadow = solid_from_alpha(&alpha_of(&a), hex(color)?, *opacity);
                from_premul(&over(&src, &shadow), rgba);
            }
            Fx::Glow { radius, strength, color } => {
                let src = to_premul(rgba);
                let halo = match color {
                    Some(c) => {
                        let a = blur_premul(&src.chunks_exact(4).flat_map(|p| [0.0, 0.0, 0.0, p[3]]).collect::<Vec<_>>(), w, h, *radius);
                        solid_from_alpha(&alpha_of(&a), hex(c)?, *strength)
                    }
                    None => blur_premul(&src, w, h, *radius).iter().map(|v| v * strength).collect(),
                };
                let out: Vec<f32> = src.iter().zip(&halo).map(|(s, h)| (s + h * (1.0 - 0.0)).min(1.0)).collect();
                // keep the original on top where it is opaque, add light elsewhere
                let out = over(&src, &out.iter().zip(&src).map(|(o, s)| (o - s).max(0.0)).collect::<Vec<_>>());
                from_premul(&out, rgba);
            }
            Fx::Outline { width, color } => {
                let src = to_premul(rgba);
                let d = dilate(&alpha_of(&src), w, h, width.round() as i32);
                let ring = solid_from_alpha(&d, hex(color)?, 1.0);
                from_premul(&over(&src, &ring), rgba);
            }
            Fx::Posterize { levels } => {
                let n = (*levels - 1) as f32;
                for p in rgba.chunks_exact_mut(4) {
                    for c in &mut p[..3] {
                        *c = ((*c as f32 / 255.0 * n).round() / n * 255.0).round() as u8;
                    }
                }
            }
            Fx::Relight { ambient, bevel, height, lights, ground_shadow, normal_map, ambient_color } => {
                let alpha: Vec<f32> = rgba.chunks_exact(4).flat_map(|p| [0.0, 0.0, 0.0, p[3] as f32 / 255.0]).collect();
                let hm = alpha_of(&blur_premul(&alpha, w, h, *bevel));
                let at = |x: i32, y: i32| hm[(y.clamp(0, h as i32 - 1) as usize) * w + x.clamp(0, w as i32 - 1) as usize];
                let cols: Vec<[f32; 3]> = lights.iter().map(|l| hex(&l.color)).collect::<Result<_, _>>()?;
                let amb_col = match ambient_color { Some(c) => hex(c)?, None => [1.0; 3] };
                let nmap = match normal_map { Some(nm) => Some(fit_normal_map(nm, w, h)?), None => None };
                // unit spot axes and cosines of the cone edges, precomputed per light
                let spots: Vec<Option<([f32; 3], f32, f32)>> = lights
                    .iter()
                    .map(|l| {
                        l.spot.as_ref().map(|s| {
                            let n = (s.dir_x * s.dir_x + s.dir_y * s.dir_y + s.dir_z * s.dir_z).sqrt();
                            let outer = s.angle.to_radians().cos();
                            let inner = (s.angle * (1.0 - s.soft)).to_radians().cos();
                            ([s.dir_x / n, s.dir_y / n, s.dir_z / n], outer, inner)
                        })
                    })
                    .collect();
                for y in 0..h {
                    for x in 0..w {
                        let (xi, yi) = (x as i32, y as i32);
                        let own = field.and_then(|f| f.normals[y * w + x]);
                        let n = match (&own, &nmap) {
                            (Some(v), _) => *v,
                            (None, Some(m)) => m[y * w + x],
                            (None, None) => {
                                let gx = (at(xi + 1, yi) - at(xi - 1, yi)) * 0.5 * height;
                                let gy = (at(xi, yi + 1) - at(xi, yi - 1)) * 0.5 * height;
                                let inv = 1.0 / (gx * gx + gy * gy + 1.0).sqrt();
                                [-gx * inv, -gy * inv, inv]
                            }
                        };
                        let mut lit = [*ambient * amb_col[0], *ambient * amb_col[1], *ambient * amb_col[2]];
                        let mut ground = 0.0f32;
                        for ((l, c), spot) in lights.iter().zip(&cols).zip(&spots) {
                            let (dx, dy, dz) = (l.x - x as f32, l.y - y as f32, l.z);
                            let d = (dx * dx + dy * dy + dz * dz).sqrt();
                            // spot cone: cosine between the cone axis and the direction light -> pixel
                            let cone = match spot {
                                Some((axis, outer, inner)) => {
                                    let cos = -(axis[0] * dx + axis[1] * dy + axis[2] * dz) / d;
                                    if cos <= *outer {
                                        0.0
                                    } else if cos >= *inner {
                                        1.0
                                    } else {
                                        let t = (cos - outer) / (inner - outer);
                                        t * t * (3.0 - 2.0 * t)
                                    }
                                }
                                None => 1.0,
                            };
                            if cone <= 0.0 {
                                continue;
                            }
                            let shadowed = l.cast_shadow && {
                                let z0 = at(xi, yi) * height;
                                shadow_ray(&at, xi, yi, z0, dx, dy, l.z, *height)
                            };
                            if shadowed {
                                ground = ground.max((1.0 - alpha[(y * w + x) * 4 + 3]) * cone);
                                continue;
                            }
                            let diff = ((n[0] * dx + n[1] * dy + n[2] * dz) / d).max(0.0);
                            let atten = if l.radius > 0.0 { (1.0 - d / l.radius).clamp(0.0, 1.0).powf(l.falloff) } else { 1.0 };
                            for k in 0..3 {
                                lit[k] += c[k] * l.intensity * diff * atten * cone;
                            }
                        }
                        let o = (y * w + x) * 4;
                        for k in 0..3 {
                            rgba[o + k] = (rgba[o + k] as f32 * lit[k]).clamp(0.0, 255.0).round() as u8;
                        }
                        // shadow cast onto the empty backdrop: black, as opaque as `ground_shadow` allows
                        if ground > 0.0 && *ground_shadow > 0.0 {
                            let a = rgba[o + 3] as f32 / 255.0;
                            let sa = ground * ground_shadow;
                            let oa = a + sa * (1.0 - a);
                            for k in 0..3 {
                                rgba[o + k] = (rgba[o + k] as f32 * a / oa).round() as u8;
                            }
                            rgba[o + 3] = (oa * 255.0).round() as u8;
                        }
                    }
                }
            }
            Fx::Pixelate { size } => {
                let s = *size as usize;
                let src = to_premul(rgba);
                let mut out = src.clone();
                for by in (0..h).step_by(s) {
                    for bx in (0..w).step_by(s) {
                        let (ye, xe) = ((by + s).min(h), (bx + s).min(w));
                        let mut acc = [0.0f32; 4];
                        for y in by..ye {
                            for x in bx..xe {
                                for c in 0..4 {
                                    acc[c] += src[(y * w + x) * 4 + c];
                                }
                            }
                        }
                        let n = ((ye - by) * (xe - bx)) as f32;
                        for y in by..ye {
                            for x in bx..xe {
                                for c in 0..4 {
                                    out[(y * w + x) * 4 + c] = acc[c] / n;
                                }
                            }
                        }
                    }
                }
                from_premul(&out, rgba);
            }
            Fx::Levels { in_black, in_white, gamma, out_black, out_white } => {
                for p in rgba.chunks_exact_mut(4) {
                    for c in &mut p[..3] {
                        let v = ((*c as f32 / 255.0 - in_black) / (in_white - in_black)).clamp(0.0, 1.0).powf(1.0 / gamma);
                        *c = ((out_black + v * (out_white - out_black)).clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            }
            Fx::Fade { amount } => {
                for p in rgba.chunks_exact_mut(4) {
                    p[3] = (p[3] as f32 * (1.0 - amount)).round() as u8;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod relight_tests {
    use super::*;

    fn disc(w: usize, h: usize) -> Vec<u8> {
        let mut v = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let d = ((x as f32 - 32.0).powi(2) + (y as f32 - 32.0).powi(2)).sqrt();
                if d < 20.0 {
                    v[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[200, 200, 200, 255]);
                }
            }
        }
        v
    }
    fn fx(lx: f32) -> Fx {
        Fx::Relight { ambient: 0.2, bevel: 6.0, height: 12.0, lights: vec![Light { x: lx, y: 32.0, z: 30.0, color: "#ffffff".into(), intensity: 1.0, radius: 0.0, cast_shadow: false, spot: None, falloff: 2.0 }], ground_shadow: 0.0, normal_map: None, ambient_color: None }
    }

    #[test]
    fn lit_side_is_brighter_and_alpha_untouched() {
        let mut img = disc(64, 64);
        let orig = img.clone();
        apply(&[fx(-40.0)], 64, 64, &mut img).unwrap();
        let px = |x: usize, y: usize| img[(y * 64 + x) * 4] as i32;
        assert!(px(16, 32) > px(48, 32) + 10, "left edge faces the light on the left: {} vs {}", px(16, 32), px(48, 32));
        assert!(img.chunks_exact(4).zip(orig.chunks_exact(4)).all(|(a, b)| a[3] == b[3]));
        let mut mirrored = disc(64, 64);
        apply(&[fx(104.0)], 64, 64, &mut mirrored).unwrap();
        assert!(mirrored[(32 * 64 + 48) * 4] as i32 > mirrored[(32 * 64 + 16) * 4] as i32 + 10);
    }

    #[test]
    fn validation_and_determinism() {
        let bad = Fx::Relight { ambient: 0.2, bevel: 4.0, height: 8.0, lights: vec![Light { x: 0.0, y: 0.0, z: 0.0, color: "#fff".into(), intensity: 1.0, radius: 0.0, cast_shadow: false, spot: None, falloff: 2.0 }], ground_shadow: 0.0, normal_map: None, ambient_color: None };
        assert!(apply(&[bad], 64, 64, &mut disc(64, 64)).is_err(), "light must be above the plane");
        let (mut a, mut b) = (disc(64, 64), disc(64, 64));
        apply(&[fx(0.0)], 64, 64, &mut a).unwrap();
        apply(&[fx(0.0)], 64, 64, &mut b).unwrap();
        assert_eq!(a, b);
        let j = serde_json::to_string(&fx(1.0)).unwrap();
        assert_eq!(serde_json::from_str::<Fx>(&j).unwrap(), fx(1.0));
    }

    #[test]
    fn cast_shadow_lands_on_the_far_side_only_when_enabled() {
        let mk = |cast: bool, ground: f32| Fx::Relight { ambient: 0.2, bevel: 6.0, height: 12.0, lights: vec![Light { x: -40.0, y: 32.0, z: 30.0, color: "#ffffff".into(), intensity: 1.0, radius: 0.0, cast_shadow: cast, spot: None, falloff: 2.0 }], ground_shadow: ground, normal_map: None, ambient_color: None };
        let alpha_at = |img: &[u8], x: usize, y: usize| img[(y * 64 + x) * 4 + 3];
        let mut on = disc(64, 64);
        apply(&[mk(true, 0.8)], 64, 64, &mut on).unwrap();
        assert!(alpha_at(&on, 58, 32) > 100, "shadow behind the disc: {}", alpha_at(&on, 58, 32));
        assert_eq!(alpha_at(&on, 5, 32), 0, "no shadow on the lit side");
        assert_eq!(alpha_at(&on, 58, 2), 0, "none away from the shadow line");
        let mut off = disc(64, 64);
        apply(&[mk(false, 0.8)], 64, 64, &mut off).unwrap();
        assert_eq!(alpha_at(&off, 58, 32), 0, "disabled by default");
        let mut none = disc(64, 64);
        apply(&[mk(true, 0.0)], 64, 64, &mut none).unwrap();
        assert_eq!(alpha_at(&none, 58, 32), 0, "ground opacity 0 draws nothing");
        assert!(apply(&[mk(true, 2.0)], 64, 64, &mut disc(64, 64)).is_err());
        // old JSON without the new fields still loads
        let old = r##"{"type":"relight","ambient":0.2,"bevel":4.0,"height":8.0,"lights":[{"x":0,"y":0,"z":10}]}"##;
        assert!(serde_json::from_str::<Fx>(old).is_ok());
    }
}

#[cfg(test)]
mod normal_spot_tests {
    use super::*;

    fn flat(w: usize, h: usize) -> Vec<u8> {
        [200u8, 200, 200, 255].iter().copied().cycle().take(w * h * 4).collect()
    }
    fn png_of(w: u32, h: u32, rgb: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut e = png::Encoder::new(&mut out, w, h);
        e.set_color(png::ColorType::Rgb);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(rgb).unwrap();
        out
    }
    /// uniform normal map tilted toward screen-left (+x tilt negative) or right
    fn tilted(w: u32, h: u32, right: bool) -> NormalMap {
        let px = if right { [218u8, 128, 218] } else { [37u8, 128, 218] };
        let rgb: Vec<u8> = px.iter().copied().cycle().take((w * h * 3) as usize).collect();
        NormalMap { data: Some(format!("data:image/png;base64,{}", crate::base64::encode(&png_of(w, h, &rgb)))), path: None, fit: NormalFit::Stretch, strength: 1.0, green_down: false }
    }
    fn light(x: f32, spot: Option<Spot>) -> Light {
        Light { x, y: 32.0, z: 30.0, color: "#ffffff".into(), intensity: 1.0, radius: 0.0, cast_shadow: false, spot, falloff: 2.0 }
    }
    fn relight(nm: Option<NormalMap>, lights: Vec<Light>) -> Fx {
        Fx::Relight { ambient: 0.1, bevel: 0.0, height: 0.0, lights, ground_shadow: 0.0, normal_map: nm, ambient_color: None }
    }
    fn r(img: &[u8], x: usize, y: usize) -> i32 {
        img[(y * 64 + x) * 4] as i32
    }

    #[test]
    fn tilted_normal_map_lights_the_correct_side() {
        // surface leans toward the right: a light on the right lights it more than one on the left
        let mut a = flat(64, 64);
        apply(&[relight(Some(tilted(8, 8, true)), vec![light(120.0, None)])], 64, 64, &mut a).unwrap();
        let mut b = flat(64, 64);
        apply(&[relight(Some(tilted(8, 8, true)), vec![light(-60.0, None)])], 64, 64, &mut b).unwrap();
        assert!(r(&a, 32, 32) > r(&b, 32, 32) + 40, "{} vs {}", r(&a, 32, 32), r(&b, 32, 32));
        let mut c = flat(64, 64);
        apply(&[relight(Some(tilted(8, 8, false)), vec![light(-60.0, None)])], 64, 64, &mut c).unwrap();
        assert!(r(&c, 32, 32) > r(&a, 5, 5) - 200 && r(&c, 32, 32) > r(&b, 32, 32) + 40, "left-leaning map prefers the left light");
        // green_down flips the y tilt: map with strong green up, light above (small y)
        let mut up = tilted(8, 8, true);
        let rgb: Vec<u8> = [128u8, 218, 218].iter().copied().cycle().take(8 * 8 * 3).collect();
        up.data = Some(crate::base64::encode(&png_of(8, 8, &rgb)));
        let top = Light { y: -60.0, ..light(32.0, None) };
        let (mut p, mut q) = (flat(64, 64), flat(64, 64));
        apply(&[relight(Some(up.clone()), vec![top.clone()])], 64, 64, &mut p).unwrap();
        up.green_down = true;
        apply(&[relight(Some(up), vec![top])], 64, 64, &mut q).unwrap();
        assert!(r(&p, 32, 32) > r(&q, 32, 32) + 40, "green up faces a light above the frame");
    }

    #[test]
    fn fit_modes_and_errors() {
        let mut ok = flat(64, 64);
        assert!(apply(&[relight(Some(tilted(3, 5, true)), vec![light(0.0, None)])], 64, 64, &mut ok).is_ok(), "stretch");
        let mut t = tilted(3, 5, true);
        t.fit = NormalFit::Tile;
        assert!(apply(&[relight(Some(t.clone()), vec![light(0.0, None)])], 64, 64, &mut flat(64, 64)).is_ok());
        t.fit = NormalFit::Exact;
        assert!(apply(&[relight(Some(t), vec![light(0.0, None)])], 64, 64, &mut flat(64, 64)).unwrap_err().contains("exact"));
        assert!(apply(&[relight(Some(tilted(64, 64, true)), vec![])], 64, 64, &mut flat(64, 64)).is_ok());
        let mut bad = tilted(4, 4, true);
        bad.data = Some("not base64 !!".into());
        assert!(apply(&[relight(Some(bad.clone()), vec![])], 64, 64, &mut flat(64, 64)).is_err());
        bad.data = Some(crate::base64::encode(b"definitely not a png"));
        assert!(apply(&[relight(Some(bad.clone()), vec![])], 64, 64, &mut flat(64, 64)).is_err());
        bad.data = Some("data:image/png,xyz".into());
        assert!(apply(&[relight(Some(bad.clone()), vec![])], 64, 64, &mut flat(64, 64)).is_err());
        let mut nan = tilted(4, 4, true);
        nan.strength = f32::NAN;
        assert!(apply(&[relight(Some(nan), vec![])], 64, 64, &mut flat(64, 64)).is_err());
        let none = NormalMap { data: None, path: None, fit: NormalFit::Stretch, strength: 1.0, green_down: false };
        assert!(apply(&[relight(Some(none), vec![])], 64, 64, &mut flat(64, 64)).is_err());
        let unresolved = NormalMap { path: Some("x.png".into()), ..tilted(2, 2, true) };
        let unresolved = NormalMap { data: None, ..unresolved };
        assert!(apply(&[relight(Some(unresolved.clone()), vec![])], 64, 64, &mut flat(64, 64)).is_err());
        let mut chain = vec![relight(Some(unresolved), vec![])];
        assert!(resolve_normal_map_paths(&mut chain, std::path::Path::new("/nonexistent-dir")).is_err());
    }

    #[test]
    fn path_is_resolved_to_data() {
        let scratch = crate::fsutil::ScratchDir::new("osatura_nm");
        let dir = scratch.to_path_buf();
        let rgb = vec![128u8; 4 * 4 * 3];
        std::fs::write(dir.join("n.png"), png_of(4, 4, &rgb)).unwrap();
        let nm = NormalMap { data: None, path: Some("n.png".into()), fit: NormalFit::Stretch, strength: 1.0, green_down: false };
        let mut chain = vec![relight(Some(nm), vec![light(0.0, None)])];
        resolve_normal_map_paths(&mut chain, &dir).unwrap();
        assert!(apply(&chain, 64, 64, &mut flat(64, 64)).is_ok());
    }

    #[test]
    fn spot_cone_cuts_and_softens() {
        // spot above (32,32), pointing straight down, 20 degrees half angle; z = 30 => radius ~ 10.9 px
        let spot = |soft: f32| Spot { dir_x: 0.0, dir_y: 0.0, dir_z: -1.0, angle: 20.0, soft };
        let run = |soft: f32| {
            let mut img = flat(64, 64);
            apply(&[relight(None, vec![light(32.0, Some(spot(soft)))])], 64, 64, &mut img).unwrap();
            img
        };
        let hard = run(0.0);
        let amb = (200.0f32 * 0.1).round() as i32;
        assert!(r(&hard, 32, 32) > 150, "centre lit");
        assert_eq!(r(&hard, 32 + 20, 32), amb, "outside the cone only ambient");
        assert_eq!(r(&hard, 32 - 20, 32), amb);
        let soft = run(1.0);
        // fully soft: the edge fades smoothly, so the in-between sample is between ambient and centre
        let mid = r(&soft, 32 + 7, 32);
        assert!(mid > amb + 5 && mid < r(&soft, 32, 32) - 5, "soft edge {mid}");
        assert!(r(&hard, 32 + 9, 32) > r(&soft, 32 + 9, 32), "hard edge is brighter just inside than a soft one");
        // an off-axis spot lights the other side
        let mut tilt = flat(64, 64);
        let l = light(0.0, Some(Spot { dir_x: 1.0, dir_y: 0.0, dir_z: -1.0, angle: 15.0, soft: 0.3 }));
        apply(&[relight(None, vec![Light { z: 30.0, ..l }])], 64, 64, &mut tilt).unwrap();
        assert!(r(&tilt, 30, 32) > amb + 50 && r(&tilt, 5, 5) == amb);
    }

    #[test]
    fn spot_shadows_and_validation() {
        // opaque disc, spot at the left aimed along +x, casting a ground shadow to the right
        let mut img = vec![0u8; 64 * 64 * 4];
        for y in 0..64usize {
            for x in 0..64usize {
                if ((x as f32 - 32.0).powi(2) + (y as f32 - 32.0).powi(2)).sqrt() < 12.0 {
                    img[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4].copy_from_slice(&[200, 200, 200, 255]);
                }
            }
        }
        let mk = |angle: f32, dy: f32| Fx::Relight {
            ambient: 0.2, bevel: 6.0, height: 12.0, ground_shadow: 0.8, normal_map: None, ambient_color: None,
            lights: vec![Light { x: -40.0, y: 32.0, z: 30.0, color: "#fff".into(), intensity: 1.0, radius: 0.0, cast_shadow: true, spot: Some(Spot { dir_x: 1.0, dir_y: dy, dir_z: -0.3, angle, soft: 0.1 }), falloff: 2.0 }],
        };
        let mut a = img.clone();
        apply(&[mk(60.0, 0.0)], 64, 64, &mut a).unwrap();
        assert!(a[(32 * 64 + 56) * 4 + 3] > 80, "spot shadow behind the disc");
        let mut off_axis = img.clone();
        apply(&[mk(3.0, 0.8)], 64, 64, &mut off_axis).unwrap();
        assert_eq!(off_axis[(32 * 64 + 56) * 4 + 3], 0, "no shadow where the cone does not reach");
        let bad = |s: Spot| relight(None, vec![light(0.0, Some(s))]);
        let ok = Spot { dir_x: 0.0, dir_y: 0.0, dir_z: -1.0, angle: 30.0, soft: 0.5 };
        assert!(apply(&[bad(ok.clone())], 8, 8, &mut flat(8, 8)).is_ok());
        for s in [
            Spot { dir_x: 0.0, dir_y: 0.0, dir_z: 0.0, ..ok.clone() },
            Spot { angle: 0.0, ..ok.clone() },
            Spot { angle: 91.0, ..ok.clone() },
            Spot { soft: 1.5, ..ok.clone() },
            Spot { dir_x: f32::NAN, ..ok.clone() },
            Spot { angle: f32::INFINITY, ..ok.clone() },
        ] {
            assert!(apply(&[bad(s)], 8, 8, &mut flat(8, 8)).is_err());
        }
    }

    #[test]
    fn ambient_colour_falloff_and_old_json() {
        let mut img = flat(8, 8);
        let fx = Fx::Relight { ambient: 1.0, bevel: 0.0, height: 0.0, lights: vec![], ground_shadow: 0.0, normal_map: None, ambient_color: Some("#ff0000".into()) };
        apply(&[fx], 8, 8, &mut img).unwrap();
        assert_eq!(&img[0..3], &[200, 0, 0]);
        let bad = Fx::Relight { ambient: 1.0, bevel: 0.0, height: 0.0, lights: vec![], ground_shadow: 0.0, normal_map: None, ambient_color: Some("nope".into()) };
        assert!(apply(&[bad], 8, 8, &mut flat(8, 8)).is_err());
        let near = |f: f32| {
            let mut i = flat(64, 64);
            let l = Light { radius: 80.0, falloff: f, ..light(32.0, None) };
            apply(&[relight(None, vec![l])], 64, 64, &mut i).unwrap();
            r(&i, 60, 32)
        };
        assert!(near(1.0) > near(4.0), "higher exponent dims faster");
        assert!(apply(&[relight(None, vec![Light { falloff: 0.0, ..light(1.0, None) }])], 8, 8, &mut flat(8, 8)).is_err());
        let old = r##"[{"type":"relight","ambient":0.2,"bevel":4.0,"height":8.0,"lights":[{"x":0,"y":0,"z":10,"cast_shadow":true}],"ground_shadow":0.5}]"##;
        let chain: Vec<Fx> = serde_json::from_str(old).unwrap();
        match &chain[0] {
            Fx::Relight { normal_map, ambient_color, lights, .. } => {
                assert!(normal_map.is_none() && ambient_color.is_none() && lights[0].spot.is_none() && lights[0].falloff == 2.0);
            }
            _ => panic!(),
        }
        let j = serde_json::to_string(&chain).unwrap();
        assert!(!j.contains("normal_map") && !j.contains("spot"));
        let new = r##"[{"type":"relight","ambient":0.2,"bevel":0,"height":0,"lights":[{"x":0,"y":0,"z":10,"spot":{"dir_x":0,"dir_y":0,"dir_z":-1,"angle":25}}],"normal_map":{"data":"AAAA"}}]"##;
        let c2: Vec<Fx> = serde_json::from_str(new).unwrap();
        assert_eq!(serde_json::from_str::<Vec<Fx>>(&serde_json::to_string(&c2).unwrap()).unwrap(), c2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot(w: usize, h: usize) -> Vec<u8> {
        let mut v = vec![0u8; w * h * 4];
        let o = ((h / 2) * w + w / 2) * 4;
        v[o..o + 4].copy_from_slice(&[255, 0, 0, 255]);
        v
    }
    fn alpha_sum(v: &[u8]) -> f32 {
        v.chunks_exact(4).map(|p| p[3] as f32 / 255.0).sum()
    }

    #[test]
    fn blur_spreads_and_conserves_energy() {
        let mut v = dot(31, 31);
        let before = alpha_sum(&v);
        apply(&[Fx::Blur { radius: 4.0 }], 31, 31, &mut v).unwrap();
        let centre = v[((15 * 31) + 15) * 4 + 3];
        assert!(centre < 120 && centre > 5, "centre dimmed but present: {centre}");
        assert!(v[((15 * 31) + 18) * 4 + 3] > 0, "neighbour gained alpha");
        assert!((alpha_sum(&v) - before).abs() < 0.15 * before + 0.5, "{} vs {}", alpha_sum(&v), before);
        assert_eq!(&v[((15 * 31) + 15) * 4..((15 * 31) + 15) * 4 + 3], &[255, 0, 0], "hue preserved");
    }

    #[test]
    fn shadow_is_offset_and_behind() {
        let mut v = dot(21, 21);
        apply(&[Fx::DropShadow { dx: 4.0, dy: 3.0, blur: 0.0, color: "#0000ff".into(), opacity: 1.0 }], 21, 21, &mut v).unwrap();
        let at = |x: usize, y: usize| &v[(y * 21 + x) * 4..(y * 21 + x) * 4 + 4];
        assert_eq!(at(10, 10), [255, 0, 0, 255], "original stays on top");
        assert_eq!(at(14, 13), [0, 0, 255, 255], "shadow at the offset");
        assert_eq!(at(5, 5)[3], 0);
    }

    #[test]
    fn outline_grows_the_silhouette() {
        let mut v = dot(21, 21);
        apply(&[Fx::Outline { width: 3.0, color: "#00ff00".into() }], 21, 21, &mut v).unwrap();
        assert_eq!(&v[(10 * 21 + 12) * 4..(10 * 21 + 12) * 4 + 4], &[0, 255, 0, 255]);
        assert_eq!(v[(10 * 21 + 14) * 4 + 3], 0, "beyond the width stays clear");
    }

    #[test]
    fn glow_adds_light_without_removing_the_source() {
        let mut v = dot(21, 21);
        apply(&[Fx::Glow { radius: 5.0, strength: 2.0, color: None }], 21, 21, &mut v).unwrap();
        assert_eq!(&v[(10 * 21 + 10) * 4..(10 * 21 + 10) * 4 + 4], &[255, 0, 0, 255]);
        assert!(v[(10 * 21 + 13) * 4 + 3] > 0, "halo around the dot");
    }

    #[test]
    fn posterize_and_levels_and_fade() {
        let mut v = vec![10, 128, 250, 200];
        apply(&[Fx::Posterize { levels: 2 }], 1, 1, &mut v).unwrap();
        assert_eq!(v, vec![0, 255, 255, 200]);
        let mut w = vec![100, 100, 100, 255];
        apply(&[Fx::Levels { in_black: 0.0, in_white: 1.0, gamma: 1.0, out_black: 0.0, out_white: 1.0 }], 1, 1, &mut w).unwrap();
        assert_eq!(w, vec![100, 100, 100, 255], "identity levels");
        apply(&[Fx::Levels { in_black: 0.2, in_white: 0.6, gamma: 1.0, out_black: 0.0, out_white: 1.0 }], 1, 1, &mut w).unwrap();
        assert!((w[0] as i32 - 122).abs() <= 1, "{w:?}");
        let mut f = vec![10, 20, 30, 200];
        apply(&[Fx::Fade { amount: 0.5 }], 1, 1, &mut f).unwrap();
        assert_eq!(f[3], 100);
    }

    #[test]
    fn pixelate_makes_uniform_blocks() {
        let mut v: Vec<u8> = (0..16).flat_map(|i| [(i * 15) as u8, 0, 0, 255]).collect();
        apply(&[Fx::Pixelate { size: 2 }], 4, 4, &mut v).unwrap();
        assert_eq!(&v[0..4], &v[4..8]);
        assert_eq!(&v[0..4], &v[16..20]);
        assert_ne!(&v[0..4], &v[8..12]);
    }

    #[test]
    fn invalid_input_is_rejected() {
        let mut v = dot(4, 4);
        assert!(apply(&[Fx::Blur { radius: -1.0 }], 4, 4, &mut v).is_err());
        assert!(apply(&[Fx::Blur { radius: f32::NAN }], 4, 4, &mut v).is_err());
        assert!(apply(&[Fx::Posterize { levels: 1 }], 4, 4, &mut v).is_err());
        assert!(apply(&[Fx::Outline { width: 2.0, color: "nope".into() }], 4, 4, &mut v).is_err());
        assert!(apply(&[Fx::Fade { amount: 0.1 }], 5, 4, &mut v).is_err(), "buffer size mismatch");
        assert!(apply(&[Fx::Levels { in_black: 0.5, in_white: 0.5, gamma: 1.0, out_black: 0.0, out_white: 1.0 }], 4, 4, &mut v).is_err());
    }

    #[test]
    fn chain_json_round_trip() {
        let c = vec![Fx::Blur { radius: 2.0 }, Fx::DropShadow { dx: 1.0, dy: 2.0, blur: 3.0, color: "#000".into(), opacity: 0.5 }, Fx::Levels { in_black: 0.0, in_white: 1.0, gamma: 1.2, out_black: 0.0, out_white: 1.0 }];
        let j = serde_json::to_string(&c).unwrap();
        let d: Vec<Fx> = serde_json::from_str(&j).unwrap();
        assert_eq!(c, d);
        let short: Vec<Fx> = serde_json::from_str(r#"[{"type":"levels","in_black":0,"in_white":1,"gamma":1}]"#).unwrap();
        assert_eq!(short.len(), 1);
    }
}

#[cfg(test)]
mod layer_normal_tests {
    use super::*;
    use crate::path::VPath;
    use crate::project::{Layer, Project};

    fn png_of(w: u32, h: u32, rgb: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut e = png::Encoder::new(&mut out, w, h);
        e.set_color(png::ColorType::Rgb);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(rgb).unwrap();
        out
    }

    /// A 32x32 sphere normal map (OpenGL convention, green = up) with the sphere filling the map.
    fn sphere_map() -> NormalMap {
        let n = 32usize;
        let mut rgb = Vec::new();
        for j in 0..n {
            for i in 0..n {
                let x = (i as f32 + 0.5 - 16.0) / 16.0;
                let y = -(j as f32 + 0.5 - 16.0) / 16.0;
                let r2 = x * x + y * y;
                let v = if r2 < 1.0 { [x, y, (1.0 - r2).sqrt()] } else { [0.0, 0.0, 1.0] };
                rgb.extend(v.iter().map(|c| ((c * 0.5 + 0.5) * 255.0).round() as u8));
            }
        }
        NormalMap { data: Some(crate::base64::encode(&png_of(n as u32, n as u32, &rgb))), path: None, fit: NormalFit::Stretch, strength: 1.0, green_down: false }
    }

    fn square(name: &str, x: f32, y: f32, s: f32) -> Layer {
        Layer::vector(name, vec![VPath::rect("r", x, y, s, s).with_fill("#c8c8c8")])
    }

    fn scene(layers: Vec<Layer>) -> crate::engine::Engine {
        let mut p = Project::new("n", 64.0, 64.0, 24.0, 1);
        p.layers = layers;
        crate::engine::Engine::new(p).unwrap()
    }

    fn flat_light(x: f32, y: f32, z: f32) -> Fx {
        Fx::Relight {
            ambient: 0.1,
            bevel: 0.0,
            height: 0.0,
            lights: vec![Light { x, y, z, color: "#ffffff".into(), intensity: 1.0, radius: 0.0, cast_shadow: false, spot: None, falloff: 2.0 }],
            ground_shadow: 0.0,
            normal_map: None,
            ambient_color: None,
        }
    }

    fn red(px: &[u8], x: usize, y: usize) -> f32 {
        px[(y * 64 + x) * 4] as f32
    }

    /// Lambert value of the analytic sphere under the point light, for the frame pixel (x, y) of a
    /// 48 px square at (8, 8) carrying the sphere map.
    fn expected(x: usize, y: usize, light: (f32, f32, f32)) -> f32 {
        let nx = (x as f32 + 0.5 - 32.0) / 24.0;
        let ny = -(y as f32 + 0.5 - 32.0) / 24.0;
        let nz = (1.0 - nx * nx - ny * ny).sqrt();
        // the effect's normal is x right, y down
        let n = [nx, -ny, nz];
        let (dx, dy, dz) = (light.0 - x as f32, light.1 - y as f32, light.2);
        let d = (dx * dx + dy * dy + dz * dz).sqrt();
        200.0 * (0.1 + ((n[0] * dx + n[1] * dy + n[2] * dz) / d).max(0.0))
    }

    #[test]
    fn a_layer_normal_map_lights_a_flat_square_like_a_sphere() {
        let mut ball = square("ball", 8.0, 8.0, 48.0);
        ball.normal_map = Some(sphere_map());
        let e = scene(vec![ball]);
        let chain = [flat_light(32.0, 32.0, 40.0)];
        let (w, h, px) = render_with_layer_normals(&e, 0.0, 1.0, &chain, std::path::Path::new(".")).unwrap();
        assert_eq!((w, h), (64, 64));
        // centre faces the viewer and the light: 200 * (0.1 + 1)
        assert!((red(&px, 32, 32) - expected(32, 32, (32.0, 32.0, 40.0))).abs() < 4.0, "{}", red(&px, 32, 32));
        // off-centre pixels follow the analytic sphere (bilinear fit and 8-bit quantisation allowed)
        for (x, y) in [(20usize, 32usize), (44, 32), (32, 20), (32, 44), (24, 24), (41, 40)] {
            let want = expected(x, y, (32.0, 32.0, 40.0));
            assert!((red(&px, x, y) - want).abs() < 14.0, "({x},{y}): got {} want {want}", red(&px, x, y));
        }
        // the same square without a map is flat: uniform under the same light except for distance to the light
        let plain = scene(vec![square("ball", 8.0, 8.0, 48.0)]);
        let (_, _, q) = render_with_layer_normals(&plain, 0.0, 1.0, &chain, std::path::Path::new(".")).unwrap();
        assert!((red(&q, 32, 32) - red(&px, 32, 32)).abs() < 4.0);
        assert!(red(&q, 14, 32) > red(&px, 14, 32) + 15.0, "a flat square faces the light more than the sphere rim");
    }

    #[test]
    fn side_light_brightens_the_facing_half_only_with_a_map() {
        let mut ball = square("ball", 8.0, 8.0, 48.0);
        ball.normal_map = Some(sphere_map());
        let e = scene(vec![ball]);
        let chain = [flat_light(-60.0, 32.0, 30.0)];
        let (_, _, px) = render_with_layer_normals(&e, 0.0, 1.0, &chain, std::path::Path::new(".")).unwrap();
        assert!(red(&px, 16, 32) > red(&px, 48, 32) + 60.0, "{} vs {}", red(&px, 16, 32), red(&px, 48, 32));
        let plain = scene(vec![square("ball", 8.0, 8.0, 48.0)]);
        let (_, _, q) = render_with_layer_normals(&plain, 0.0, 1.0, &chain, std::path::Path::new(".")).unwrap();
        assert!((red(&q, 16, 32) - red(&q, 48, 32)).abs() < 60.0 && red(&q, 16, 32) - red(&q, 48, 32) < red(&px, 16, 32) - red(&px, 48, 32));
    }

    #[test]
    fn maps_apply_per_layer_in_paint_order_and_inherit_from_groups() {
        // two squares side by side: only the left one has a map; the right keeps the global bevel look
        let mut left = square("left", 2.0, 8.0, 28.0);
        left.normal_map = Some(sphere_map());
        let e = scene(vec![left, square("right", 34.0, 8.0, 28.0)]);
        let field = layer_normal_field(&e, 0.0, 1.0, std::path::Path::new(".")).unwrap().unwrap();
        assert!(field.normals[22 * 64 + 16].is_some() && field.normals[22 * 64 + 48].is_none());
        // a plain layer painted over the mapped one clears the field where it overlaps
        let mut under = square("under", 8.0, 8.0, 48.0);
        under.normal_map = Some(sphere_map());
        let e = scene(vec![under, square("over", 8.0, 8.0, 20.0)]);
        let f = layer_normal_field(&e, 0.0, 1.0, std::path::Path::new(".")).unwrap().unwrap();
        assert!(f.normals[14 * 64 + 14].is_none(), "covered by the plain layer");
        assert!(f.normals[40 * 64 + 40].is_some());
        // a group's map reaches its children
        let mut g = Layer::group("g", vec![square("a", 8.0, 8.0, 20.0), square("b", 36.0, 36.0, 20.0)]);
        g.normal_map = Some(sphere_map());
        let f = layer_normal_field(&scene(vec![g]), 0.0, 1.0, std::path::Path::new(".")).unwrap().unwrap();
        assert!(f.normals[18 * 64 + 18].is_some() && f.normals[46 * 64 + 46].is_some());
        // no map anywhere: no field, and the result equals a plain render + apply
        let plain = scene(vec![square("a", 8.0, 8.0, 20.0)]);
        assert!(layer_normal_field(&plain, 0.0, 1.0, std::path::Path::new(".")).unwrap().is_none());
        let chain = [flat_light(10.0, 10.0, 30.0)];
        let (w, h, a) = render_with_layer_normals(&plain, 0.0, 1.0, &chain, std::path::Path::new(".")).unwrap();
        let (_, _, mut b) = crate::raster::render_rgba(&plain, 0.0, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: false });
        apply(&chain, w as usize, h as usize, &mut b).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn errors_are_values() {
        // parameters are validated when the engine is built
        let mut bad = square("a", 8.0, 8.0, 20.0);
        bad.normal_map = Some(NormalMap { strength: 99.0, ..sphere_map() });
        let mut p = Project::new("n", 64.0, 64.0, 24.0, 1);
        p.layers = vec![bad];
        assert!(crate::engine::Engine::new(p.clone()).err().unwrap().join(" ").contains("normal map"));
        p.layers[0].normal_map = Some(NormalMap { data: None, path: None, ..sphere_map() });
        assert!(crate::engine::Engine::new(p.clone()).is_err());
        // a fit of "exact" needs the bounding box to match the map
        let mut ex = square("a", 8.0, 8.0, 20.0);
        ex.normal_map = Some(NormalMap { fit: NormalFit::Exact, ..sphere_map() });
        let e = scene(vec![ex]);
        assert!(layer_normal_field(&e, 0.0, 1.0, std::path::Path::new(".")).unwrap_err().contains("exact"));
        // not a PNG, unreadable path
        let mut junk = square("a", 8.0, 8.0, 20.0);
        junk.normal_map = Some(NormalMap { data: Some(crate::base64::encode(b"nope")), ..sphere_map() });
        assert!(layer_normal_field(&scene(vec![junk]), 0.0, 1.0, std::path::Path::new(".")).is_err());
        let mut gone = square("a", 8.0, 8.0, 20.0);
        gone.normal_map = Some(NormalMap { data: None, path: Some("no/such/file.png".into()), ..sphere_map() });
        assert!(layer_normal_field(&scene(vec![gone]), 0.0, 1.0, std::path::Path::new(".")).unwrap_err().contains("cannot read"));
        // a field of the wrong size is refused
        let f = NormalField { w: 2, h: 2, normals: vec![None; 4] };
        assert!(apply_with_normals(&[], 8, 8, &mut vec![0u8; 256], Some(&f)).is_err());
        // a project without a map serialises without the field, with one it round-trips
        let plain = square("a", 8.0, 8.0, 20.0);
        assert!(!serde_json::to_string(&plain).unwrap().contains("normal_map"));
        let mut mapped = plain.clone();
        mapped.normal_map = Some(sphere_map());
        let back: Layer = serde_json::from_str(&serde_json::to_string(&mapped).unwrap()).unwrap();
        assert_eq!(back.normal_map, mapped.normal_map);
    }
}
