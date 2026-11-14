// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Float rendering path (R-038): the frame is composited in f32 premultiplied RGBA instead of
//! tiny-skia's 8-bit buffers, so gradients, opacity and blend modes carry no 8-bit quantisation.
//!
//! * Coverage: own scanline rasteriser, 16 sub-scanlines per pixel with *exact* horizontal span
//!   coverage (non-zero and even-odd rules), accumulated in f32.
//! * Geometry: the same flattening of fills and the same tiny-skia stroker/dasher/variable-width
//!   outlines as the 8-bit path (the stroker only produces geometry; no pixels come from it).
//! * Shading: solid, linear and radial gradients evaluated per pixel centre in f32
//!   (straight-colour interpolation, then premultiply — the same convention as the 8-bit path).
//! * Compositing: source-over, all 13 blend modes (W3C compositing formulas), group opacity and
//!   layer masks, in f32. Values stay in the same non-colour-managed (sRGB-encoded) space as the
//!   8-bit rasteriser so the two renderers are directly comparable; `hdr` converts to linear light.
//!
//! NOT float: image layers (bitmap and mesh-deformed) are drawn by the 8-bit tiny-skia path into a
//! temporary layer and composited in float (bitmap sources are 8-bit anyway); bone overlays are
//! not drawn. Hairline-thin strokes use the stroker's geometry, not Skia's hairline mode.

use crate::engine::{Engine, Frame, NodeKind, RenderNode};
use crate::path::{width_outline, VPath};
use crate::raster::{RasterOptions, Raster};
use crate::style::{Blend, Cap, Join, Paint, Rgba, Stop};
use tiny_skia as sk;

/// Premultiplied RGBA, f32, row-major, sRGB-encoded (not linearised).
#[derive(Clone, Debug)]
pub struct FloatImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<f32>,
}

impl FloatImage {
    /// Straight-alpha 8-bit RGBA (round to nearest) — for comparisons and legacy encoders.
    pub fn to_straight_u8(&self) -> Vec<u8> {
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let mut out = Vec::with_capacity(self.rgba.len());
        for p in self.rgba.chunks_exact(4) {
            let a = p[3];
            let d = if a > 0.0 { 1.0 / a } else { 0.0 };
            out.extend_from_slice(&[q(p[0] * d), q(p[1] * d), q(p[2] * d), q(a)]);
        }
        out
    }
}

type Px = [f32; 4];

struct Buf {
    w: usize,
    h: usize,
    d: Vec<Px>,
}

impl Buf {
    fn new(w: usize, h: usize) -> Buf {
        Buf { w, h, d: vec![[0.0; 4]; w * h] }
    }
}

// ----------------------------------------------------------------------------------------------
// Blending
// ----------------------------------------------------------------------------------------------

fn hard_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 { cb * 2.0 * cs } else { let s = 2.0 * cs - 1.0; cb + s - cb * s }
}

fn sep(mode: Blend, cb: f32, cs: f32) -> f32 {
    match mode {
        Blend::Normal | Blend::Add => cs,
        Blend::Multiply => cb * cs,
        Blend::Screen => cb + cs - cb * cs,
        Blend::Overlay => hard_light(cs, cb),
        Blend::HardLight => hard_light(cb, cs),
        Blend::Darken => cb.min(cs),
        Blend::Lighten => cb.max(cs),
        Blend::ColorDodge => {
            if cb == 0.0 { 0.0 } else if cs >= 1.0 { 1.0 } else { (cb / (1.0 - cs)).min(1.0) }
        }
        Blend::ColorBurn => {
            if cb >= 1.0 { 1.0 } else if cs <= 0.0 { 0.0 } else { 1.0 - ((1.0 - cb) / cs).min(1.0) }
        }
        Blend::SoftLight => {
            if cs <= 0.5 {
                cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
            } else {
                let d = if cb <= 0.25 { ((16.0 * cb - 12.0) * cb + 4.0) * cb } else { cb.sqrt() };
                cb + (2.0 * cs - 1.0) * (d - cb)
            }
        }
        Blend::Difference => (cb - cs).abs(),
        Blend::Exclusion => cb + cs - 2.0 * cb * cs,
    }
}

/// Composite premultiplied `s` over premultiplied `d` with a blend mode.
fn blend_px(d: Px, s: Px, mode: Blend) -> Px {
    let (sa, da) = (s[3], d[3]);
    if sa <= 0.0 {
        return d;
    }
    match mode {
        Blend::Normal => [0, 1, 2, 3].map(|i| s[i] + d[i] * (1.0 - sa)),
        Blend::Add => [0, 1, 2, 3].map(|i| (s[i] + d[i]).min(1.0)),
        _ => {
            let mut o = [0.0; 4];
            for i in 0..3 {
                let cs = (s[i] / sa).clamp(0.0, 1.0);
                let cb = if da > 0.0 { (d[i] / da).clamp(0.0, 1.0) } else { 0.0 };
                o[i] = s[i] * (1.0 - da) + d[i] * (1.0 - sa) + sa * da * sep(mode, cb, cs);
            }
            o[3] = sa + da - sa * da;
            o
        }
    }
}

// ----------------------------------------------------------------------------------------------
// Coverage rasteriser
// ----------------------------------------------------------------------------------------------

const SUB: usize = 16;

struct Cov {
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    d: Vec<f32>,
}

fn flatten(path: &sk::Path, t: sk::Transform) -> Vec<Vec<(f32, f32)>> {
    let mut polys: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut cur: Vec<(f32, f32)> = Vec::new();
    let mut last = (0.0f32, 0.0f32);
    let mp = |p: sk::Point| {
        let mut v = [p];
        t.map_points(&mut v);
        (v[0].x, v[0].y)
    };
    let dist = |a: (f32, f32), b: (f32, f32)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    for seg in path.segments() {
        match seg {
            sk::PathSegment::MoveTo(p) => {
                if cur.len() > 2 {
                    polys.push(std::mem::take(&mut cur));
                } else {
                    cur.clear();
                }
                last = mp(p);
                cur.push(last);
            }
            sk::PathSegment::LineTo(p) => {
                last = mp(p);
                cur.push(last);
            }
            sk::PathSegment::QuadTo(c, p) => {
                let (c, p) = (mp(c), mp(p));
                let n = ((dist(last, c) + dist(c, p)).sqrt() * 3.0).ceil().clamp(4.0, 128.0) as usize;
                for i in 1..=n {
                    let u = i as f32 / n as f32;
                    let v = 1.0 - u;
                    cur.push((v * v * last.0 + 2.0 * v * u * c.0 + u * u * p.0, v * v * last.1 + 2.0 * v * u * c.1 + u * u * p.1));
                }
                last = p;
            }
            sk::PathSegment::CubicTo(c1, c2, p) => {
                let (c1, c2, p) = (mp(c1), mp(c2), mp(p));
                let n = ((dist(last, c1) + dist(c1, c2) + dist(c2, p)).sqrt() * 3.0).ceil().clamp(4.0, 128.0) as usize;
                for i in 1..=n {
                    let u = i as f32 / n as f32;
                    let v = 1.0 - u;
                    let (b0, b1, b2, b3) = (v * v * v, 3.0 * v * v * u, 3.0 * v * u * u, u * u * u);
                    cur.push((b0 * last.0 + b1 * c1.0 + b2 * c2.0 + b3 * p.0, b0 * last.1 + b1 * c1.1 + b2 * c2.1 + b3 * p.1));
                }
                last = p;
            }
            sk::PathSegment::Close => {}
        }
    }
    if cur.len() > 2 {
        polys.push(cur);
    }
    polys
}

fn add_span(row: &mut [f32], cx0: usize, xa: f32, xb: f32) {
    let (lo, hi) = (cx0 as f32, (cx0 + row.len()) as f32);
    let (xa, xb) = (xa.clamp(lo, hi), xb.clamp(lo, hi));
    if xb <= xa {
        return;
    }
    let k = 1.0 / SUB as f32;
    let (i0, i1) = ((xa.floor() as usize).min(cx0 + row.len() - 1), (xb.floor() as usize).min(cx0 + row.len() - 1));
    if i0 == i1 {
        row[i0 - cx0] += (xb - xa) * k;
        return;
    }
    row[i0 - cx0] += ((i0 + 1) as f32 - xa) * k;
    for v in &mut row[i0 + 1 - cx0..i1 - cx0] {
        *v += k;
    }
    row[i1 - cx0] += (xb - i1 as f32) * k;
}

fn coverage(polys: &[Vec<(f32, f32)>], even_odd: bool, w: usize, h: usize) -> Option<Cov> {
    let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &(x, y) in polys.iter().flatten() {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        minx = minx.min(x);
        maxx = maxx.max(x);
        miny = miny.min(y);
        maxy = maxy.max(y);
    }
    if minx > maxx {
        return None;
    }
    let cx0 = minx.floor().max(0.0) as usize;
    let cx1 = (maxx.ceil().max(0.0) as usize).min(w);
    let cy0 = miny.floor().max(0.0) as usize;
    let cy1 = (maxy.ceil().max(0.0) as usize).min(h);
    if cx1 <= cx0 || cy1 <= cy0 {
        return None;
    }
    let (cw, ch) = (cx1 - cx0, cy1 - cy0);
    let nsub = ch * SUB;
    let mut buckets: Vec<Vec<(f32, i32)>> = vec![Vec::new(); nsub];
    for poly in polys {
        for i in 0..poly.len() {
            let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
            if p.1 == q.1 {
                continue;
            }
            let (dir, a, b) = if q.1 > p.1 { (1, p, q) } else { (-1, q, p) };
            let to_k = |y: f32| ((y - cy0 as f32) * SUB as f32 - 0.5).ceil();
            let k0 = to_k(a.1).max(0.0) as usize;
            let k1 = to_k(b.1).clamp(0.0, nsub as f32) as usize;
            for (k, bucket) in buckets.iter_mut().enumerate().take(k1).skip(k0) {
                let sy = cy0 as f32 + (k as f32 + 0.5) / SUB as f32;
                bucket.push((a.0 + (sy - a.1) * (b.0 - a.0) / (b.1 - a.1), dir));
            }
        }
    }
    let mut d = vec![0.0f32; cw * ch];
    for (k, xs) in buckets.iter_mut().enumerate() {
        if xs.len() < 2 {
            continue;
        }
        xs.sort_by(|a, b| a.0.total_cmp(&b.0));
        let row = &mut d[(k / SUB) * cw..(k / SUB + 1) * cw];
        let (mut wind, mut start) = (0i32, 0.0f32);
        for &(x, dir) in xs.iter() {
            let was = if even_odd { wind & 1 != 0 } else { wind != 0 };
            wind += dir;
            let is = if even_odd { wind & 1 != 0 } else { wind != 0 };
            if !was && is {
                start = x;
            } else if was && !is {
                add_span(row, cx0, start, x);
            }
        }
    }
    for v in &mut d {
        *v = v.min(1.0);
    }
    Some(Cov { x0: cx0, y0: cy0, w: cw, h: ch, d })
}

// ----------------------------------------------------------------------------------------------
// Shading
// ----------------------------------------------------------------------------------------------

fn clamp_c(c: Rgba) -> [f32; 4] {
    [c.r.clamp(0.0, 1.0), c.g.clamp(0.0, 1.0), c.b.clamp(0.0, 1.0), c.a.clamp(0.0, 1.0)]
}

enum Shader {
    Solid(Px),
    Linear { from: (f32, f32), d: (f32, f32), inv_len2: f32, stops: Vec<(f32, [f32; 4])> },
    Radial { c: (f32, f32), inv_r: f32, stops: Vec<(f32, [f32; 4])> },
}

/// Straight RGBA with the colour channels converted from sRGB-encoded to linear light (alpha untouched).
fn to_linear_c(c: [f32; 4]) -> [f32; 4] {
    [crate::hdr::srgb_to_linear(c[0]), crate::hdr::srgb_to_linear(c[1]), crate::hdr::srgb_to_linear(c[2]), c[3]]
}

/// The colour as the compositor works with it: encoded as stored, or in linear light.
fn work_c(c: Rgba, linear: bool) -> [f32; 4] {
    if linear { to_linear_c(clamp_c(c)) } else { clamp_c(c) }
}

fn sorted_stops(s: &[Stop], linear: bool) -> Vec<(f32, [f32; 4])> {
    let mut v: Vec<_> = s.iter().map(|st| (st.offset.clamp(0.0, 1.0), work_c(st.color, linear))).collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v
}

fn premul(c: [f32; 4]) -> Px {
    [c[0] * c[3], c[1] * c[3], c[2] * c[3], c[3]]
}

fn solid(c: Rgba, linear: bool) -> Shader {
    Shader::Solid(premul(work_c(c, linear)))
}

fn make_shader(p: &Paint, linear: bool) -> Shader {
    if let Some(c) = p.degenerate_color() {
        return solid(c, linear);
    }
    match p {
        Paint::Solid(c) => solid(*c, linear),
        Paint::Linear { from, to, stops } => {
            let d = (to.x - from.x, to.y - from.y);
            let l2 = d.0 * d.0 + d.1 * d.1;
            if stops.is_empty() || (l2.partial_cmp(&(1e-12)) != Some(std::cmp::Ordering::Greater)) {
                return solid(p.base_color(), linear);
            }
            Shader::Linear { from: (from.x, from.y), d, inv_len2: 1.0 / l2, stops: sorted_stops(stops, linear) }
        }
        Paint::Radial { center, radius, stops } => {
            if stops.is_empty() {
                return solid(p.base_color(), linear);
            }
            Shader::Radial { c: (center.x, center.y), inv_r: 1.0 / radius.max(0.001), stops: sorted_stops(stops, linear) }
        }
    }
}

fn ramp(stops: &[(f32, [f32; 4])], t: f32) -> Px {
    let t = t.clamp(0.0, 1.0);
    let (first, last) = (stops[0], stops[stops.len() - 1]);
    if t <= first.0 {
        return premul(first.1);
    }
    for w in stops.windows(2) {
        if t < w[1].0 {
            let span = w[1].0 - w[0].0;
            let u = if span > 0.0 { (t - w[0].0) / span } else { 1.0 };
            return premul([0, 1, 2, 3].map(|i| w[0].1[i] + (w[1].1[i] - w[0].1[i]) * u));
        }
    }
    premul(last.1)
}

impl Shader {
    /// Premultiplied colour at local-space point `(x, y)`.
    fn at(&self, x: f32, y: f32) -> Px {
        match self {
            Shader::Solid(c) => *c,
            Shader::Linear { from, d, inv_len2, stops } => {
                ramp(stops, ((x - from.0) * d.0 + (y - from.1) * d.1) * inv_len2)
            }
            Shader::Radial { c, inv_r, stops } => ramp(stops, ((x - c.0).powi(2) + (y - c.1).powi(2)).sqrt() * inv_r),
        }
    }
}

// ----------------------------------------------------------------------------------------------
// Renderer
// ----------------------------------------------------------------------------------------------

struct FRaster<'a> {
    frame: &'a Frame,
    base: sk::Transform,
    scale: f32,
    /// Composite in linear light (`ColorSpace::Linear`); the buffer then holds premultiplied LINEAR values.
    linear: bool,
}

fn paint_cov(canvas: &mut Buf, cov: &Cov, shader: &Shader, inv: sk::Transform) {
    for ry in 0..cov.h {
        for rx in 0..cov.w {
            let c = cov.d[ry * cov.w + rx];
            if c <= 0.0 {
                continue;
            }
            let (x, y) = (cov.x0 + rx, cov.y0 + ry);
            let mut p = [sk::Point::from_xy(x as f32 + 0.5, y as f32 + 0.5)];
            inv.map_points(&mut p);
            let s = shader.at(p[0].x, p[0].y).map(|v| v * c);
            let d = &mut canvas.d[y * canvas.w + x];
            *d = blend_px(*d, s, Blend::Normal);
        }
    }
}

impl FRaster<'_> {
    fn fill_geom(&self, canvas: &mut Buf, polys: &[Vec<(f32, f32)>], even_odd: bool, paint: &Paint, inv: sk::Transform) {
        if let Some(cov) = coverage(polys, even_odd, canvas.w, canvas.h) {
            paint_cov(canvas, &cov, &make_shader(paint, self.linear), inv);
        }
    }

    fn draw_path(&self, canvas: &mut Buf, world: &crate::math::Affine2, path: &VPath) {
        let Some(geom) = crate::raster::sk_path(path) else { return };
        let t = crate::raster::ts(world).post_concat(self.base);
        let Some(inv) = t.invert() else { return };
        if let Some(fill) = &path.fill {
            self.fill_geom(canvas, &flatten(&geom, t), false, fill, inv);
        }
        let Some(st) = &path.stroke else { return };
        if st.width_profile.is_empty() {
            let mut stroke = sk::Stroke { width: st.width.max(0.0), ..Default::default() };
            stroke.line_cap = match st.cap {
                Cap::Butt => sk::LineCap::Butt,
                Cap::Round => sk::LineCap::Round,
                Cap::Square => sk::LineCap::Square,
            };
            stroke.line_join = match st.join {
                Join::Miter => sk::LineJoin::Miter,
                Join::Round => sk::LineJoin::Round,
                Join::Bevel => sk::LineJoin::Bevel,
            };
            if st.dash.len() >= 2 && st.dash.len() % 2 == 0 && st.dash.iter().all(|d| *d > 0.0) {
                stroke.dash = sk::StrokeDash::new(st.dash.clone(), st.dash_offset);
            }
            let res = (t.sx * t.sy - t.kx * t.ky).abs().sqrt().max(0.01);
            let src = match stroke.dash.take() {
                Some(d) => geom.dash(&d, res),
                None => Some(geom.clone()),
            };
            if let Some(outline) = src.and_then(|g| g.stroke(&stroke, res)) {
                self.fill_geom(canvas, &flatten(&outline, t), false, &st.paint, inv);
            }
        } else {
            let tol = 0.25 / (world.x_scale() * self.base.sx).max(0.01);
            if let Some(out) = width_outline(&path.polyline(tol), st) {
                if let Some(p) = crate::raster::polys_path(&out.polys) {
                    self.fill_geom(canvas, &flatten(&p, t), out.even_odd, &st.paint, inv);
                }
            }
        }
    }

    fn draw_node(&self, canvas: &mut Buf, id: usize, force_visible: bool, simple: bool) {
        let node = &self.frame.nodes[id];
        if !node.visible && !force_visible {
            return;
        }
        let effects = !simple && !node.effects.is_empty();
        let need_layer = node.opacity < 0.999 || effects || (!simple && (node.blend != Blend::Normal || node.mask.is_some()));
        if !need_layer {
            self.draw_content(canvas, id, force_visible, simple);
            return;
        }
        let mut tmp = Buf::new(canvas.w, canvas.h);
        let pad = if effects { crate::layerfx::effect_pad(&node.effects, self.base.sx) } else { 0 };
        if pad > 0 {
            // see raster.rs: filter on a canvas enlarged by the blur reach, then crop
            let mut big = Buf::new(canvas.w + 2 * pad, canvas.h + 2 * pad);
            let inner = FRaster { frame: self.frame, base: self.base.post_translate(pad as f32, pad as f32), scale: self.scale, linear: self.linear };
            inner.draw_content(&mut big, id, force_visible, simple);
            let mut flat: Vec<f32> = big.d.iter().flatten().copied().collect();
            crate::layerfx::apply_premul(&mut flat, big.w, big.h, &node.effects, self.base.sx);
            for y in 0..canvas.h {
                for x in 0..canvas.w {
                    let s = ((y + pad) * big.w + x + pad) * 4;
                    tmp.d[y * canvas.w + x] = [flat[s], flat[s + 1], flat[s + 2], flat[s + 3]];
                }
            }
        } else {
            self.draw_content(&mut tmp, id, force_visible, simple);
            if effects {
                let mut flat: Vec<f32> = tmp.d.iter().flatten().copied().collect();
                crate::layerfx::apply_premul(&mut flat, tmp.w, tmp.h, &node.effects, self.base.sx);
                for (d, s) in tmp.d.iter_mut().zip(flat.chunks_exact(4)) {
                    *d = [s[0], s[1], s[2], s[3]];
                }
            }
        }
        if let (false, Some((mid, invert))) = (simple, node.mask) {
            let mut mp = Buf::new(canvas.w, canvas.h);
            self.draw_node(&mut mp, mid, true, true);
            for (p, m) in tmp.d.iter_mut().zip(&mp.d) {
                let k = if invert { 1.0 - m[3] } else { m[3] };
                *p = p.map(|v| v * k);
            }
        }
        let op = node.opacity.clamp(0.0, 1.0);
        let mode = if simple { Blend::Normal } else { node.blend };
        for (d, s) in canvas.d.iter_mut().zip(&tmp.d) {
            if s[3] > 0.0 {
                *d = blend_px(*d, s.map(|v| v * op), mode);
            }
        }
    }

    fn draw_content(&self, canvas: &mut Buf, id: usize, force_visible: bool, simple: bool) {
        let node: &RenderNode = &self.frame.nodes[id];
        match node.kind {
            NodeKind::Vector => {
                for p in &node.paths {
                    self.draw_path(canvas, &node.world, p);
                }
            }
            NodeKind::Group | NodeKind::Switch => {
                for &c in &node.children {
                    self.draw_node(canvas, c, force_visible, simple);
                }
            }
            NodeKind::Image => {
                // 8-bit fallback (bitmap data is 8-bit; mesh deformation uses tiny-skia sampling).
                let Some(mut pm) = sk::Pixmap::new(canvas.w as u32, canvas.h as u32) else { return };
                let r = Raster { frame: self.frame, base: self.base, blur: None };
                r.draw_image(&mut pm, node);
                for (d, s) in canvas.d.iter_mut().zip(pm.pixels()) {
                    if s.alpha() > 0 {
                        let mut s = [s.red() as f32, s.green() as f32, s.blue() as f32, s.alpha() as f32].map(|v| v / 255.0);
                        if self.linear {
                            // the 8-bit layer is sRGB-encoded premultiplied: straight, linearise, premultiply again
                            let a = s[3];
                            s = premul(to_linear_c([s[0] / a, s[1] / a, s[2] / a, a].map(|v| v.clamp(0.0, 1.0))));
                        }
                        *d = blend_px(*d, s, Blend::Normal);
                    }
                }
            }
        }
    }
}

/// Render one frame in f32. Mirrors `raster::render_pixmap` (bone overlays excluded). Composites in the
/// project's [`ColorSpace`](crate::project::ColorSpace); the result is always sRGB-encoded premultiplied.
pub fn render_float(engine: &Engine, frame: f32, opts: &RasterOptions) -> FloatImage {
    let f = engine.evaluate(frame);
    render_float_frame(engine, &f, opts)
}

/// [`render_float`] for an already evaluated frame.
pub fn render_float_frame(engine: &Engine, f: &Frame, opts: &RasterOptions) -> FloatImage {
    render_float_in(engine, f, opts, engine.project.color_space == crate::project::ColorSpace::Linear)
}

/// [`render_float_frame`] with the compositing space chosen by the caller (`linear = false`: sRGB-encoded values).
pub fn render_float_in(engine: &Engine, f: &Frame, opts: &RasterOptions, linear: bool) -> FloatImage {
    let p = &engine.project;
    // beyond the output limits the picture is drawn at the largest scale that fits (callers that can report an error refuse first)
    let mut opts = opts.clone();
    opts.scale = crate::raster::fit_scale(p.width, p.height, opts.scale);
    let opts = &opts;
    let w = ((p.width * opts.scale).round() as u32).max(1);
    let h = ((p.height * opts.scale).round() as u32).max(1);
    let mut buf = Buf::new(w as usize, h as usize);
    if let (Some(bg), false) = (p.background, opts.transparent) {
        let c = premul(work_c(bg, linear));
        buf.d.iter_mut().for_each(|d| *d = c);
    }
    let r = FRaster { frame: f, base: sk::Transform::from_scale(opts.scale, opts.scale), scale: opts.scale, linear };
    let _ = r.scale;
    for &root in &f.roots {
        r.draw_node(&mut buf, root, false, false);
    }
    if linear {
        // back to sRGB-encoded premultiplied, the representation every consumer of `FloatImage` expects
        for px in buf.d.iter_mut() {
            let a = px[3];
            if a > 0.0 {
                let enc = |v: f32| crate::hdr::linear_to_srgb((v / a).clamp(0.0, 1.0)) * a;
                *px = [enc(px[0]), enc(px[1]), enc(px[2]), a];
            }
        }
    }
    FloatImage { width: w, height: h, rgba: buf.d.iter().flatten().copied().collect() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{Layer, Project};
    use crate::style::Rgba;

    fn eng(layers: Vec<Layer>, w: f32, h: f32) -> Engine {
        let mut p = Project::new("t", w, h, 24.0, 1);
        p.layers = layers;
        Engine::new(p).unwrap()
    }

    #[test]
    fn coverage_is_exact_for_axis_aligned_rect() {
        // 0.25 .. 2.5 horizontally, 0.5 .. 1.5 vertically
        let poly = vec![vec![(0.25, 0.5), (2.5, 0.5), (2.5, 1.5), (0.25, 1.5)]];
        let c = coverage(&poly, false, 4, 4).unwrap();
        let at = |x: usize, y: usize| c.d[(y - c.y0) * c.w + (x - c.x0)];
        assert!((at(0, 0) - 0.75 * 0.5).abs() < 1e-5, "{}", at(0, 0));
        assert!((at(1, 0) - 0.5).abs() < 1e-5);
        assert!((at(2, 0) - 0.25).abs() < 1e-5);
        assert!((at(2, 1) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn even_odd_and_nonzero_differ_on_nested_same_direction_squares() {
        let sq = |a: f32, b: f32| vec![(a, a), (b, a), (b, b), (a, b)];
        let polys = vec![sq(0.0, 8.0), sq(2.0, 6.0)];
        let nz = coverage(&polys, false, 8, 8).unwrap();
        let eo = coverage(&polys, true, 8, 8).unwrap();
        assert!((nz.d[4 * 8 + 4] - 1.0).abs() < 1e-6);
        assert!(eo.d[4 * 8 + 4].abs() < 1e-6);
    }

    fn opts() -> RasterOptions {
        RasterOptions { scale: 1.0, show_bones: false, transparent: true }
    }

    #[test]
    fn parity_with_8bit_raster_on_demo() {
        // Premultiplied 8-bit values of both renderers must agree to <= 2/255 everywhere the image
        // is locally smooth (gradients, flat fills, blend/opacity results: tiny-skia's low-precision
        // pipeline vs exact f32 differ by rounding, <= 2 steps). Anti-aliased edges legitimately
        // differ more (tiny-skia coverage is 4x4 supersampled, ours 16 x exact): there the mean
        // error and the share of pixels beyond 2/255 must stay small.
        let e = Engine::new(crate::demo::project()).unwrap();
        for frame in [0.0, 12.0] {
            let (w, h, a8) = crate::raster::render_rgba(&e, frame, &opts());
            let f = render_float(&e, frame, &opts());
            assert_eq!((w, h), (f.width, f.height));
            let pre = |v: &[u8]| -> Vec<[i32; 4]> {
                v.chunks_exact(4).map(|p| [0, 1, 2].map(|k| p[k] as i32 * p[3] as i32 / 255)).zip(v.chunks_exact(4)).map(|(c, p)| [c[0], c[1], c[2], p[3] as i32]).collect()
            };
            let (a, b) = (pre(&a8), pre(&f.to_straight_u8()));
            let (w, h) = (w as usize, h as usize);
            let (mut big, mut sum, mut flat_max, mut edge_px) = (0usize, 0u64, 0i32, 0usize);
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    let d = (0..4).map(|k| (a[i][k] - b[i][k]).abs()).max().unwrap();
                    sum += d as u64;
                    let mut nb = 0;
                    for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                        let (nx, ny) = ((x as i32 + dx).clamp(0, w as i32 - 1) as usize, (y as i32 + dy).clamp(0, h as i32 - 1) as usize);
                        nb = nb.max((0..4).map(|k| (a[i][k] - a[ny * w + nx][k]).abs()).max().unwrap());
                    }
                    if nb <= 4 {
                        flat_max = flat_max.max(d);
                    } else {
                        edge_px += 1;
                    }
                    if d > 2 {
                        big += 1;
                    }
                }
            }
            let n = (w * h) as f64;
            assert!(flat_max <= 2, "frame {frame}: smooth-area difference {flat_max}/255");
            assert!(sum as f64 / (n * 255.0) < 0.002, "frame {frame}: mean abs diff {}", sum as f64 / (n * 255.0));
            assert!((big as f64) < n * 0.015 && big <= edge_px, "frame {frame}: {big} px beyond 2/255 ({edge_px} edge px)");
        }
    }

    #[test]
    fn long_gradient_has_more_than_256_levels() {
        use crate::math::Vec2;
        use crate::path::VPath;
        use crate::style::Paint;
        let g = Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(1024.0, 0.0), &[(0.0, "#000000"), (1.0, "#ffffff")]);
        let e = eng(vec![Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 1024.0, 4.0).with_paint(g)])], 1024.0, 4.0);
        let f = render_float(&e, 0.0, &opts());
        let row = &f.rgba[(2 * 1024) * 4..(3 * 1024) * 4];
        let eight: std::collections::BTreeSet<u8> = f.to_straight_u8().chunks_exact(4).map(|p| p[0]).collect();
        assert!(eight.len() <= 256);
        let png = crate::hdr::encode_png16_buf(f.width, f.height, &f.rgba, false).unwrap();
        let mut dec = png::Decoder::new(std::io::Cursor::new(png));
        dec.set_transformations(png::Transformations::IDENTITY);
        let mut r = dec.read_info().unwrap();
        let mut buf = vec![0; r.output_buffer_size().unwrap()];
        r.next_frame(&mut buf).unwrap();
        let levels: std::collections::BTreeSet<u16> =
            (0..1024).map(|x| { let i = (2 * 1024 + x) * 8; u16::from_be_bytes([buf[i], buf[i + 1]]) }).collect();
        assert!(levels.len() > 900, "16-bit PNG has {} levels", levels.len());
        // exact ramp value at a pixel centre
        let x = 300usize;
        assert!((row[x * 4] - (x as f32 + 0.5) / 1024.0).abs() < 1e-5);
    }

    #[test]
    fn opacity_and_multiply_are_not_quantised() {
        use crate::math::Vec2;
        use crate::path::VPath;
        use crate::style::Paint;
        let g = Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(1024.0, 0.0), &[(0.0, "#000000"), (1.0, "#ffffff")]);
        let mut l = Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 1024.0, 4.0).with_paint(g)]);
        l.opacity = 0.5;
        let mut top = Layer::vector("t", vec![VPath::rect("r", 0.0, 0.0, 1024.0, 4.0).with_fill("#808080")]);
        top.blend = Blend::Multiply;
        let mut p = Project::new("t", 1024.0, 4.0, 24.0, 1);
        p.background = Rgba::parse("#ffffff");
        p.layers = vec![l, top];
        let e = Engine::new(p).unwrap();
        let f = render_float(&e, 0.0, &RasterOptions::default());
        let distinct: std::collections::BTreeSet<u32> = (0..1024).map(|x| f.rgba[(1024 * 2 + x) * 4].to_bits()).collect();
        assert!(distinct.len() > 900, "{}", distinct.len());
        // x=511: gradient 0.5-ish * 0.5 opacity over white = 1-0.5*(1-g); multiply by 128/255
        let g511 = 511.5 / 1024.0;
        let want = (1.0 - 0.5 * (1.0 - g511)) * (128.0 / 255.0);
        assert!((f.rgba[(1024 * 2 + 511) * 4] - want).abs() < 1e-4, "{} vs {want}", f.rgba[(1024 * 2 + 511) * 4]);
    }

    #[test]
    fn exr_float_samples_are_bit_exact_f32() {
        // header sizes: parse our own file (independent OpenEXR reader check is run out of band)
        let buf = vec![0.25f32, 0.5, 0.75, 1.0, 0.0, 0.0, 0.0, 0.0];
        let b = crate::hdr::encode_exr_buf(2, 1, &buf, crate::hdr::Precision::Float).unwrap();
        let h = crate::hdr::encode_exr_buf(2, 1, &buf, crate::hdr::Precision::Half).unwrap();
        assert_eq!(b.len() - h.len(), 2 * 4 * 2, "FLOAT is 2 bytes/sample larger than HALF");
        assert!(b.windows(8).any(|w| w == b"channels"));
        let pos = b.len() - 2 * 4 * 4; // last 32 bytes = A,B,G,R planes of 2 px
        let a0 = f32::from_le_bytes(b[pos..pos + 4].try_into().unwrap());
        assert_eq!(a0, 1.0);
        let r0 = f32::from_le_bytes(b[pos + 24..pos + 28].try_into().unwrap());
        assert_eq!(r0, crate::hdr::srgb_to_linear(0.25));
        assert!(crate::hdr::Precision::parse("double").is_err());
        assert_eq!(crate::hdr::Precision::parse("Float").unwrap(), crate::hdr::Precision::Float);
        assert!(crate::hdr::encode_exr_buf(2, 2, &buf, crate::hdr::Precision::Float).is_err());
    }

    #[test]
    fn masks_group_opacity_and_dash_match_8bit_semantics() {
        use crate::project::MaskRef;
        use crate::style::StrokeStyle;
        let px = |f: &FloatImage, x: usize, y: usize| -> [f32; 4] { let i = (y * f.width as usize + x) * 4; [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2], f.rgba[i + 3]] };
        // mask + invert
        let mut hole = Layer::vector("hole", vec![crate::path::VPath::rect("r", 10.0, 10.0, 20.0, 20.0).with_fill("#ffffff")]);
        hole.visible = false;
        let mut cut = Layer::vector("cut", vec![crate::path::VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_fill("#ff0000")]);
        cut.mask = Some(MaskRef { layer: "hole".into(), invert: false });
        let f = render_float(&eng(vec![hole.clone(), cut.clone()], 40.0, 40.0), 0.0, &opts());
        assert_eq!(px(&f, 20, 20), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(px(&f, 3, 3)[3], 0.0);
        cut.mask = Some(MaskRef { layer: "hole".into(), invert: true });
        let f = render_float(&eng(vec![hole, cut], 40.0, 40.0), 0.0, &opts());
        assert_eq!(px(&f, 20, 20)[3], 0.0);
        assert_eq!(px(&f, 3, 3), [1.0, 0.0, 0.0, 1.0]);
        // group opacity flattens the group: overlap not darker, and exactly 0.5
        let a = Layer::vector("a", vec![crate::path::VPath::rect("r", 0.0, 0.0, 30.0, 40.0).with_fill("#000000")]);
        let b = Layer::vector("b", vec![crate::path::VPath::rect("r", 10.0, 0.0, 30.0, 40.0).with_fill("#000000")]);
        let mut g = Layer::group("g", vec![a, b]);
        g.opacity = 0.5;
        let f = render_float(&eng(vec![g], 40.0, 40.0), 0.0, &opts());
        assert!((px(&f, 5, 20)[3] - 0.5).abs() < 1e-6 && (px(&f, 20, 20)[3] - 0.5).abs() < 1e-6);
        // dashed butt stroke leaves gaps
        let mut path = crate::path::VPath::new("l", false, vec![crate::path::Anchor::corner(crate::math::Vec2::new(0.0, 20.0)), crate::path::Anchor::corner(crate::math::Vec2::new(40.0, 20.0))]);
        let mut st = StrokeStyle::solid("#000000", 4.0);
        st.cap = Cap::Butt;
        st.dash = vec![10.0, 10.0];
        path.stroke = Some(st);
        let f = render_float(&eng(vec![Layer::vector("v", vec![path])], 40.0, 40.0), 0.0, &opts());
        assert_eq!((px(&f, 5, 20)[3], px(&f, 15, 20)[3], px(&f, 25, 20)[3]), (1.0, 0.0, 1.0));
    }

    /// L-22 (wave 7): `color_space: "linear"` composites in linear light. 50 % black over white is 0.5 in linear
    /// light = sRGB 0.7354 (188/255), not 0.5 (128/255); the default is unchanged.
    #[test]
    fn linear_light_blend_of_half_black_over_white() {
        use crate::project::ColorSpace;
        let mut top = Layer::vector("t", vec![crate::path::VPath::rect("r", 0.0, 0.0, 8.0, 8.0).with_fill("#000000")]);
        top.opacity = 0.5;
        let build = |cs: ColorSpace| {
            let mut p = Project::new("t", 8.0, 8.0, 24.0, 1);
            p.background = Rgba::parse("#ffffff");
            p.layers = vec![top.clone()];
            p.color_space = cs;
            Engine::new(p).unwrap()
        };
        let want = crate::hdr::linear_to_srgb(0.5);
        assert!((want - 0.7354).abs() < 1e-3);
        let f = render_float(&build(ColorSpace::Linear), 0.0, &RasterOptions::default());
        let i = (4 * 8 + 4) * 4;
        for k in 0..3 {
            assert!((f.rgba[i + k] - want).abs() < 1e-4, "channel {k}: {} vs {want}", f.rgba[i + k]);
        }
        assert_eq!(f.rgba[i + 3], 1.0);
        // default (sRGB-encoded) compositing is the historical 0.5
        let f = render_float(&build(ColorSpace::Srgb), 0.0, &RasterOptions::default());
        assert!((f.rgba[i] - 0.5).abs() < 1e-6);
        // the 8-bit renderer follows the project's colour space too (float compositing, one conversion to 8 bit)
        let (_, _, px) = crate::raster::render_rgba(&build(ColorSpace::Linear), 0.0, &RasterOptions::default());
        assert!((px[i] as i32 - 188).abs() <= 1 && px[i + 3] == 255, "8-bit linear result {}", px[i]);
        let (_, _, px) = crate::raster::render_rgba(&build(ColorSpace::Srgb), 0.0, &RasterOptions::default());
        assert!((px[i] as i32 - 128).abs() <= 1, "8-bit sRGB result {}", px[i]);
    }

    /// Opaque colours are unchanged by the linear round trip, gradients interpolate in linear light, and transparent
    /// output keeps straight colours (a half-transparent red stays red, not darker).
    #[test]
    fn linear_light_keeps_flat_colours_and_interpolates_gradients_in_linear_light() {
        use crate::math::Vec2;
        use crate::project::ColorSpace;
        use crate::style::Paint;
        let g = Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), &[(0.0, "#000000"), (1.0, "#ffffff")]);
        let mut p = Project::new("t", 100.0, 4.0, 24.0, 1);
        p.layers = vec![
            Layer::vector("g", vec![crate::path::VPath::rect("r", 0.0, 0.0, 100.0, 2.0).with_paint(g)]),
            Layer::vector("flat", vec![crate::path::VPath::rect("r", 0.0, 2.0, 100.0, 2.0).with_fill("#3366cc80")]),
        ];
        p.color_space = ColorSpace::Linear;
        let f = render_float(&Engine::new(p).unwrap(), 0.0, &opts());
        let px = |x: usize, y: usize| -> [f32; 4] { let i = (y * 100 + x) * 4; [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2], f.rgba[i + 3]] };
        // gradient mid-point: linear 0.505 (x = 50.5 / 100), encoded 0.7367
        let mid = px(50, 0)[0];
        assert!((mid - crate::hdr::linear_to_srgb(0.505)).abs() < 1e-3, "{mid}");
        // straight colour of the translucent fill is unchanged after the round trip
        let c = px(10, 3);
        let a = c[3];
        assert!((a - 128.0 / 255.0).abs() < 1e-3);
        for (k, want) in [0x33, 0x66, 0xcc].iter().enumerate() {
            assert!((c[k] / a - *want as f32 / 255.0).abs() < 1e-3, "channel {k}: {}", c[k] / a);
        }
    }

    #[test]
    fn colour_space_serialises_only_when_not_default_and_rejects_bad_values() {
        use crate::project::ColorSpace;
        let mut p = Project::new("t", 8.0, 8.0, 24.0, 1);
        assert!(!p.to_json_pretty().contains("color_space"), "the default is not written, so old goldens and files are unchanged");
        p.color_space = ColorSpace::Linear;
        let text = p.to_json_pretty();
        assert!(text.contains("\"color_space\": \"linear\""));
        let back: Project = serde_json::from_str(&text).unwrap();
        assert_eq!(back.color_space, ColorSpace::Linear);
        assert!(serde_json::from_str::<Project>(&text.replace("linear", "p3")).is_err());
        assert_eq!(ColorSpace::parse(" Linear ").unwrap(), ColorSpace::Linear);
        assert!(ColorSpace::parse("rec2020").unwrap_err().contains("srgb or linear"));
    }

    #[test]
    fn blend_modes_match_known_values() {
        let d = [0.5, 0.25, 0.0, 1.0];
        let s = [0.5, 0.5, 0.5, 1.0];
        let m = blend_px(d, s, Blend::Multiply);
        assert!((m[0] - 0.25).abs() < 1e-6 && (m[1] - 0.125).abs() < 1e-6 && m[2].abs() < 1e-6);
        let sc = blend_px(d, s, Blend::Screen);
        assert!((sc[0] - 0.75).abs() < 1e-6);
        let half = blend_px(d, [0.0, 0.0, 0.0, 0.5], Blend::Normal);
        assert!((half[0] - 0.25).abs() < 1e-6 && (half[3] - 1.0).abs() < 1e-6);
        assert_eq!(blend_px(d, [1.0, 1.0, 1.0, 1.0], Blend::Add)[0], 1.0);
    }
}
