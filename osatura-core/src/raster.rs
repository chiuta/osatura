// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Software rasteriser (tiny-skia): PNG / RGBA frames with gradients, dashes,
//! variable-width strokes, blend modes, opacity groups and layer masks.

use crate::engine::{Engine, Frame, NodeKind};
use crate::math::{Affine2, Vec2};
use crate::path::{width_outline, VPath};
use crate::style::{Blend, Cap, Join, Paint, Rgba};
use tiny_skia as sk;

#[derive(Clone, Debug)]
pub struct RasterOptions {
    /// Output pixels per stage unit.
    pub scale: f32,
    pub show_bones: bool,
    /// Skip the project background (transparent output) — used for ghosts and compositing.
    pub transparent: bool,
}

impl Default for RasterOptions {
    fn default() -> Self {
        RasterOptions { scale: 1.0, show_bones: false, transparent: false }
    }
}

pub(crate) fn ts(m: &Affine2) -> sk::Transform {
    sk::Transform::from_row(m.a, m.b, m.c, m.d, m.tx, m.ty)
}

fn color(c: Rgba) -> sk::Color {
    sk::Color::from_rgba(c.r.clamp(0.0, 1.0), c.g.clamp(0.0, 1.0), c.b.clamp(0.0, 1.0), c.a.clamp(0.0, 1.0))
        .unwrap_or(sk::Color::BLACK)
}

fn blend_mode(b: Blend) -> sk::BlendMode {
    match b {
        Blend::Normal => sk::BlendMode::SourceOver,
        Blend::Multiply => sk::BlendMode::Multiply,
        Blend::Screen => sk::BlendMode::Screen,
        Blend::Overlay => sk::BlendMode::Overlay,
        Blend::Darken => sk::BlendMode::Darken,
        Blend::Lighten => sk::BlendMode::Lighten,
        Blend::ColorDodge => sk::BlendMode::ColorDodge,
        Blend::ColorBurn => sk::BlendMode::ColorBurn,
        Blend::HardLight => sk::BlendMode::HardLight,
        Blend::SoftLight => sk::BlendMode::SoftLight,
        Blend::Difference => sk::BlendMode::Difference,
        Blend::Exclusion => sk::BlendMode::Exclusion,
        Blend::Add => sk::BlendMode::Plus,
    }
}

fn stops(s: &[crate::style::Stop]) -> Vec<sk::GradientStop> {
    let mut sorted: Vec<_> = s.to_vec();
    sorted.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    sorted.iter().map(|st| sk::GradientStop::new(st.offset.clamp(0.0, 1.0), color(st.color))).collect()
}

fn shader(p: &Paint) -> sk::Shader<'static> {
    if let Some(c) = p.degenerate_color() {
        return sk::Shader::SolidColor(color(c));
    }
    match p {
        Paint::Solid(c) => sk::Shader::SolidColor(color(*c)),
        Paint::Linear { from, to, stops: s } => sk::LinearGradient::new(
            sk::Point::from_xy(from.x, from.y),
            sk::Point::from_xy(to.x, to.y),
            stops(s),
            sk::SpreadMode::Pad,
            sk::Transform::identity(),
        )
        .unwrap_or_else(|| sk::Shader::SolidColor(color(p.base_color()))),
        Paint::Radial { center, radius, stops: s } => {
            let c = sk::Point::from_xy(center.x, center.y);
            sk::RadialGradient::new(c, 0.0, c, radius.max(0.001), stops(s), sk::SpreadMode::Pad, sk::Transform::identity())
                .unwrap_or_else(|| sk::Shader::SolidColor(color(p.base_color())))
        }
    }
}

pub(crate) fn sk_path(path: &VPath) -> Option<sk::Path> {
    if path.anchors.len() < 2 {
        return None;
    }
    let mut pb = sk::PathBuilder::new();
    let first = &path.anchors[0];
    pb.move_to(first.pos.x, first.pos.y);
    for i in 0..path.segment_count() {
        let a = &path.anchors[i];
        let b = &path.anchors[(i + 1) % path.anchors.len()];
        let (c1, c2) = (a.hout(), b.hin());
        pb.cubic_to(c1.x, c1.y, c2.x, c2.y, b.pos.x, b.pos.y);
    }
    if path.closed {
        pb.close();
    }
    pb.finish()
}

pub(crate) fn polys_path(polys: &[Vec<Vec2>]) -> Option<sk::Path> {
    let mut pb = sk::PathBuilder::new();
    for poly in polys {
        for (i, p) in poly.iter().enumerate() {
            if i == 0 {
                pb.move_to(p.x, p.y);
            } else {
                pb.line_to(p.x, p.y);
            }
        }
        pb.close();
    }
    pb.finish()
}

pub(crate) struct Raster<'a> {
    pub(crate) frame: &'a Frame,
    pub(crate) base: sk::Transform,
    /// Blur radius (output pixels) per top-level node id, applied to the node's finished layer
    /// (after effects and mask, before opacity and blend). Used by depth of field.
    pub(crate) blur: Option<&'a std::collections::HashMap<usize, f32>>,
}

impl Raster<'_> {
    fn draw_path(&self, canvas: &mut sk::Pixmap, world: &Affine2, path: &VPath) {
        let Some(geom) = sk_path(path) else { return };
        let t = ts(world).post_concat(self.base);
        if let Some(fill) = &path.fill {
            let paint = sk::Paint { shader: shader(fill), anti_alias: true, ..sk::Paint::default() };
            canvas.fill_path(&geom, &paint, sk::FillRule::Winding, t, None);
        }
        if let Some(st) = &path.stroke {
            let mut paint = sk::Paint::default();
            paint.shader = shader(&st.paint);
            paint.anti_alias = true;
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
                canvas.stroke_path(&geom, &paint, &stroke, t, None);
            } else {
                let tol = 0.25 / (world.x_scale() * self.base.sx).max(0.01);
                if let Some(out) = width_outline(&path.polyline(tol), st) {
                    if let Some(p) = polys_path(&out.polys) {
                        let rule = if out.even_odd { sk::FillRule::EvenOdd } else { sk::FillRule::Winding };
                        canvas.fill_path(&p, &paint, rule, t, None);
                    }
                }
            }
        }
    }

    /// `force_visible`/`simple`: used for mask sources (ignore visibility, blend and nested masks).
    fn draw_node(&self, canvas: &mut sk::Pixmap, id: usize, force_visible: bool, simple: bool) {
        let node = &self.frame.nodes[id];
        if !node.visible && !force_visible {
            return;
        }
        let effects = !simple && !node.effects.is_empty();
        let blur = if simple { None } else { self.blur.and_then(|m| m.get(&id)).copied().filter(|r| *r >= 0.5) };
        let need_layer = node.opacity < 0.999 || effects || blur.is_some() || (!simple && (node.blend != Blend::Normal || node.mask.is_some()));
        if !need_layer {
            self.draw_content(canvas, id, force_visible, simple);
            return;
        }
        let Some(mut tmp) = sk::Pixmap::new(canvas.width(), canvas.height()) else { return };
        let pad = if effects { crate::layerfx::effect_pad(&node.effects, self.base.sx) } else { 0 };
        if pad > 0 {
            // draw on a canvas enlarged by the blur reach, filter, crop: content just outside the canvas
            // still contributes (Chromium and resvg do the same), so the canvas edge does not fade
            let (pw, ph) = (canvas.width() + 2 * pad as u32, canvas.height() + 2 * pad as u32);
            if let Some(mut big) = sk::Pixmap::new(pw, ph) {
                let inner = Raster { frame: self.frame, base: self.base.post_translate(pad as f32, pad as f32), blur: None };
                inner.draw_content(&mut big, id, force_visible, simple);
                crate::layerfx::apply_pixmap(&mut big, &node.effects, self.base.sx);
                let (cw, row) = (canvas.width() as usize, pw as usize);
                let src = big.data();
                let dst = tmp.data_mut();
                for y in 0..canvas.height() as usize {
                    let s = ((y + pad) * row + pad) * 4;
                    dst[y * cw * 4..(y + 1) * cw * 4].copy_from_slice(&src[s..s + cw * 4]);
                }
            }
        } else {
            self.draw_content(&mut tmp, id, force_visible, simple);
            if effects {
                crate::layerfx::apply_pixmap(&mut tmp, &node.effects, self.base.sx);
            }
        }
        if let (false, Some((mid, invert))) = (simple, node.mask) {
            if let Some(mut mp) = sk::Pixmap::new(canvas.width(), canvas.height()) {
                self.draw_node(&mut mp, mid, true, true);
                let mut mask = sk::Mask::from_pixmap(mp.as_ref(), sk::MaskType::Alpha);
                if invert {
                    mask.invert();
                }
                tmp.apply_mask(&mask);
            }
        }
        if let Some(r) = blur {
            let mut buf = to_straight_rgba(&tmp);
            if crate::postfx::apply(&[crate::postfx::Fx::Blur { radius: r }], tmp.width() as usize, tmp.height() as usize, &mut buf).is_ok() {
                for (dst, c) in tmp.pixels_mut().iter_mut().zip(buf.chunks_exact(4)) {
                    *dst = sk::ColorU8::from_rgba(c[0], c[1], c[2], c[3]).premultiply();
                }
            }
        }
        let pp = sk::PixmapPaint {
            opacity: node.opacity.clamp(0.0, 1.0),
            blend_mode: if simple { sk::BlendMode::SourceOver } else { blend_mode(node.blend) },
            quality: sk::FilterQuality::Nearest,
        };
        canvas.draw_pixmap(0, 0, tmp.as_ref(), &pp, sk::Transform::identity(), None);
    }

    fn draw_content(&self, canvas: &mut sk::Pixmap, id: usize, force_visible: bool, simple: bool) {
        let node = &self.frame.nodes[id];
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
            NodeKind::Image => self.draw_image(canvas, node),
        }
    }

    pub(crate) fn draw_image(&self, canvas: &mut sk::Pixmap, node: &crate::engine::RenderNode) {
        let Some(img) = &node.image else { return };
        let t = ts(&node.world).post_concat(self.base);
        let pm = img.data.pixmap.as_ref();
        let (pw, ph) = (pm.width() as f32, pm.height() as f32);
        let pattern = |m: sk::Transform| {
            sk::Pattern::new(pm, sk::SpreadMode::Pad, sk::FilterQuality::Bilinear, 1.0, m)
        };
        match &img.mesh {
            None => {
                // image pixel space → layer space
                let m = sk::Transform::from_row(img.size.x / pw, 0.0, 0.0, img.size.y / ph, img.origin.x, img.origin.y);
                let paint = sk::Paint { shader: pattern(m), anti_alias: true, ..sk::Paint::default() };
                if let Some(rect) = sk::Rect::from_xywh(img.origin.x, img.origin.y, img.size.x, img.size.y) {
                    canvas.fill_rect(rect, &paint, t, None);
                }
            }
            Some(mesh) => {
                for tri in &mesh.tris {
                    let ix = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
                    let uv = [mesh.uvs[ix[0]], mesh.uvs[ix[1]], mesh.uvs[ix[2]]];
                    let po = [mesh.positions[ix[0]], mesh.positions[ix[1]], mesh.positions[ix[2]]];
                    let Some(m) = crate::math::tri_affine(uv, po) else { continue };
                    // grow the triangle slightly so neighbours overlap (no AA seams)
                    let c = (po[0] + po[1] + po[2]) * (1.0 / 3.0);
                    let grow = |p: Vec2| {
                        let d = p - c;
                        let l = d.length();
                        if l < 1e-6 { p } else { p + d * (0.6 / l) }
                    };
                    let g = [grow(po[0]), grow(po[1]), grow(po[2])];
                    let mut pb = sk::PathBuilder::new();
                    pb.move_to(g[0].x, g[0].y);
                    pb.line_to(g[1].x, g[1].y);
                    pb.line_to(g[2].x, g[2].y);
                    pb.close();
                    let Some(path) = pb.finish() else { continue };
                    let paint = sk::Paint { shader: pattern(ts(&m)), anti_alias: true, ..sk::Paint::default() };
                    canvas.fill_path(&path, &paint, sk::FillRule::Winding, t, None);
                }
            }
        }
    }
}

/// Longest side, in pixels, of any raster output.
pub const MAX_OUT_SIDE: u32 = 16384;
/// Most pixels of any raster output (100 Mpixel = 400 MB as RGBA8).
pub const MAX_OUT_PIXELS: u64 = 100_000_000;

/// Output size in pixels of a `w` x `h` stage drawn at `scale`, or a clean error when it is empty, longer than
/// [`MAX_OUT_SIDE`] on a side or above [`MAX_OUT_PIXELS`] in total. Call it before allocating anything.
pub fn check_size(w: f32, h: f32, scale: f32) -> Result<(u32, u32), String> {
    let (pw, ph) = ((w * scale).round(), (h * scale).round());
    if !(pw.is_finite() && ph.is_finite()) || pw < 1.0 || ph < 1.0 {
        return Err(format!("output size {pw}x{ph} is not valid"));
    }
    if pw > MAX_OUT_SIDE as f32 || ph > MAX_OUT_SIDE as f32 {
        return Err(format!("output would be {pw}x{ph} pixels: a side may be at most {MAX_OUT_SIDE} (lower --scale or the stage size)"));
    }
    if (pw as f64) * (ph as f64) > MAX_OUT_PIXELS as f64 {
        return Err(format!("output would be {pw}x{ph} = {} Mpixel, more than the limit of {} Mpixel (lower --scale or the stage size)", ((pw as f64) * (ph as f64) / 1e6).round(), MAX_OUT_PIXELS / 1_000_000));
    }
    Ok((pw as u32, ph as u32))
}

/// Most bytes of decoded RGBA frames an exporter keeps in memory at once (2 GiB).
pub const MAX_SEQUENCE_BYTES: u64 = 2 << 30;

/// [`check_project_size`] plus the total of all frames, for exporters that hold every frame in memory.
pub fn check_sequence(p: &crate::project::Project, scale: f32) -> Result<(u32, u32), String> {
    let (w, h) = check_project_size(p, scale)?;
    let total = (w as u64) * (h as u64) * 4 * (p.frames.max(1) as u64);
    if total > MAX_SEQUENCE_BYTES {
        return Err(format!("{} frames of {w}x{h} would need {} MB of memory (limit {} MB): lower --scale or the frame count", p.frames, total >> 20, MAX_SEQUENCE_BYTES >> 20));
    }
    Ok((w, h))
}

/// [`check_size`] for the project's stage.
pub fn check_project_size(p: &crate::project::Project, scale: f32) -> Result<(u32, u32), String> {
    check_size(p.width, p.height, scale)
}

/// `scale` itself when the output is within the limits, otherwise the largest scale that is.
pub fn fit_scale(w: f32, h: f32, scale: f32) -> f32 {
    if check_size(w, h, scale).is_ok() {
        return scale;
    }
    let fit = (MAX_OUT_SIDE as f32 / w.max(h).max(1.0)).min((MAX_OUT_PIXELS as f32 / (w * h).max(1.0)).sqrt()) * 0.999;
    if scale.is_finite() { scale.min(fit).max(1e-4) } else { fit }
}

/// Render one frame to a premultiplied RGBA pixmap. A size beyond the output limits ([`check_size`]) is drawn
/// at the largest scale that fits instead of allocating without bound; entry points that can report an error
/// call [`check_size`] first and refuse.
pub fn render_pixmap(engine: &Engine, frame: f32, opts: &RasterOptions) -> sk::Pixmap {
    let f = engine.evaluate(frame);
    pixmap_from_frame(engine, &f, frame, opts)
}

/// Draw an already evaluated (and possibly simplified, see `scrub`) frame. `frame` only feeds bone visibility.
pub fn pixmap_from_frame(engine: &Engine, f: &Frame, frame: f32, opts: &RasterOptions) -> sk::Pixmap {
    pixmap_from_frame_blur(engine, f, frame, opts, None)
}

/// [`pixmap_from_frame`] with an optional blur radius per top-level node id (depth of field).
pub(crate) fn pixmap_from_frame_blur(engine: &Engine, f: &Frame, frame: f32, opts: &RasterOptions, blur: Option<&std::collections::HashMap<usize, f32>>) -> sk::Pixmap {
    let p = &engine.project;
    let mut opts = opts.clone();
    opts.scale = fit_scale(p.width, p.height, opts.scale);
    let opts = &opts;
    let w = ((p.width * opts.scale).round() as u32).max(1);
    let h = ((p.height * opts.scale).round() as u32).max(1);
    let mut pm = sk::Pixmap::new(w, h).unwrap_or_else(|| sk::Pixmap::new(1, 1).expect("1x1 pixmap"));
    let r = Raster { frame: f, base: sk::Transform::from_scale(opts.scale, opts.scale), blur };
    if p.color_space == crate::project::ColorSpace::Linear && blur.is_none() {
        // Linear-light compositing: the float compositor does the blending (tiny-skia only has 8-bit buffers, which
        // would band the dark half of a linear ramp); only the final conversion to 8-bit sRGB happens here.
        let img = crate::fraster::render_float_frame(engine, f, opts);
        if let Some(mut lin) = sk::Pixmap::new(img.width, img.height) {
            for (d, s) in lin.data_mut().chunks_exact_mut(4).zip(img.rgba.chunks_exact(4)) {
                let a = (s[3].clamp(0.0, 1.0) * 255.0).round() as u8;
                for k in 0..3 {
                    d[k] = ((s[k].clamp(0.0, 1.0) * 255.0).round() as u8).min(a);
                }
                d[3] = a;
            }
            pm = lin;
        }
    } else {
        if let (Some(bg), false) = (p.background, opts.transparent) {
            pm.fill(color(bg));
        }
        for &root in &f.roots {
            r.draw_node(&mut pm, root, false, false);
        }
    }
    if opts.show_bones {
        let mut paint = sk::Paint::default();
        paint.set_color_rgba8(0xd6, 0x33, 0x6c, 230);
        paint.anti_alias = true;
        let stroke = sk::Stroke { width: 2.0, ..Default::default() };
        for (b, wmat) in p.bones.iter().zip(&f.world) {
            if !crate::bonevis::visible(p, &b.name, frame) {
                continue;
            }
            let head = f.camera.apply(wmat.apply(Vec2::ZERO));
            let tip = f.camera.apply(wmat.apply(Vec2::new(b.length, 0.0)));
            let mut pb = sk::PathBuilder::new();
            pb.move_to(head.x, head.y);
            pb.line_to(tip.x, tip.y);
            if let Some(line) = pb.finish() {
                pm.stroke_path(&line, &paint, &stroke, r.base, None);
            }
            if let Some(dot) = sk::PathBuilder::from_circle(head.x, head.y, 4.0) {
                pm.fill_path(&dot, &paint, sk::FillRule::Winding, r.base, None);
            }
        }
    }
    pm
}

pub fn render_png(engine: &Engine, frame: f32, opts: &RasterOptions) -> Result<Vec<u8>, String> {
    check_project_size(&engine.project, opts.scale)?;
    render_pixmap(engine, frame, opts).encode_png().map_err(|e| e.to_string())
}

/// Straight-alpha RGBA bytes (what GIF/video encoders expect).
pub fn to_straight_rgba(pm: &sk::Pixmap) -> Vec<u8> {
    let mut out = Vec::with_capacity(pm.pixels().len() * 4);
    for px in pm.pixels() {
        let c = px.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    out
}

/// `(width, height, straight RGBA)`.
pub fn render_rgba(engine: &Engine, frame: f32, opts: &RasterOptions) -> (u32, u32, Vec<u8>) {
    let pm = render_pixmap(engine, frame, opts);
    (pm.width(), pm.height(), to_straight_rgba(&pm))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;
    use crate::project::{Layer, MaskRef, Project};
    use crate::style::StrokeStyle;

    fn project_with(layers: Vec<Layer>) -> Engine {
        let mut p = Project::new("t", 40.0, 40.0, 24.0, 1);
        p.layers = layers;
        Engine::new(p).unwrap()
    }

    fn px(pm: &sk::Pixmap, x: u32, y: u32) -> [u8; 4] {
        let c = pm.pixel(x, y).unwrap().demultiply();
        [c.red(), c.green(), c.blue(), c.alpha()]
    }

    fn render(e: &Engine) -> sk::Pixmap {
        render_pixmap(e, 0.0, &RasterOptions::default())
    }

    fn grad_layer(paint: Paint) -> Layer {
        let mut path = VPath::rect("r", 0.0, 0.0, 40.0, 40.0);
        path.fill = Some(paint);
        Layer::vector("g", vec![path])
    }

    /// C10a/b: unsorted stops and zero-length gradients mean the same in every renderer and exporter.
    #[test]
    fn unsorted_stops_and_degenerate_gradients_agree_everywhere() {
        use crate::math::Vec2;
        // zero-length linear and radius-0 radial: the LAST stop everywhere
        let lin = Paint::linear(Vec2::new(20.0, 20.0), Vec2::new(20.0, 20.0), &[(0.0, "#ff0000"), (1.0, "#0000ff")]);
        let rad = Paint::radial(Vec2::new(20.0, 20.0), 0.0, &[(0.0, "#ff0000"), (1.0, "#0000ff")]);
        for g in [lin, rad] {
            let e = project_with(vec![grad_layer(g)]);
            let pm = render(&e);
            for (x, y) in [(1, 1), (20, 20), (38, 30)] {
                assert_eq!(px(&pm, x, y), [0, 0, 255, 255], "8-bit raster at {x},{y}");
            }
            let f = crate::fraster::render_float(&e, 0.0, &RasterOptions::default()).to_straight_u8();
            assert_eq!(&f[(20 * 40 + 20) * 4..(20 * 40 + 20) * 4 + 4], [0, 0, 255, 255], "float renderer");
            let svg = crate::svg::render_frame(&e, 0.0, &crate::svg::RenderOptions { show_bones: false });
            assert!(svg.contains("#0000ff") && !svg.contains("Gradient"), "svg paints the flat last stop: {svg}");
            let lot = crate::export::lottie_json(&e).to_string();
            assert!(lot.contains("\"fl\"") && !lot.contains("\"gf\""), "lottie writes a flat fill");
        }
        // stops given out of order are sorted: the picture equals the sorted one in raster and float
        let unsorted = Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(40.0, 0.0), &[(1.0, "#0000ff"), (0.0, "#ff0000")]);
        let sorted = Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(40.0, 0.0), &[(0.0, "#ff0000"), (1.0, "#0000ff")]);
        let (a, b) = (project_with(vec![grad_layer(unsorted.clone())]), project_with(vec![grad_layer(sorted)]));
        assert_eq!(render(&a).data(), render(&b).data());
        let (fa, fb) = (crate::fraster::render_float(&a, 0.0, &RasterOptions::default()), crate::fraster::render_float(&b, 0.0, &RasterOptions::default()));
        assert_eq!(fa.rgba, fb.rgba);
        // check warns
        let mut proj = Project::new("t", 40.0, 40.0, 24.0, 1);
        proj.layers = vec![grad_layer(unsorted)];
        let w = crate::lint::gradient_warnings(&proj);
        assert!(w.len() == 1 && w[0].1.contains("ascending"), "{w:?}");
        assert!(crate::lint::lint(&proj, None).iter().any(|f| f.code == "L011"));
    }

    #[test]
    fn solid_fill_and_transparent_background() {
        let e = project_with(vec![Layer::vector("v", vec![VPath::rect("r", 10.0, 10.0, 20.0, 20.0).with_fill("#ff0000")])]);
        let pm = render(&e);
        assert_eq!(px(&pm, 20, 20), [255, 0, 0, 255]);
        assert_eq!(px(&pm, 2, 2)[3], 0, "no background → transparent");
    }

    #[test]
    fn layer_transform_moves_pixels() {
        let mut l = Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#00ff00")]);
        l.transform.position = Vec2::new(20.0, 20.0);
        let pm = render(&project_with(vec![l]));
        assert_eq!(px(&pm, 25, 25), [0, 255, 0, 255]);
        assert_eq!(px(&pm, 5, 5)[3], 0);
    }

    #[test]
    fn gradient_goes_from_first_to_last_stop() {
        let g = Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(40.0, 0.0), &[(0.0, "#000000"), (1.0, "#ffffff")]);
        let e = project_with(vec![Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_paint(g)])]);
        let pm = render(&e);
        let (l, m, r) = (px(&pm, 1, 20)[0], px(&pm, 20, 20)[0], px(&pm, 38, 20)[0]);
        assert!(l < 20 && r > 235 && (m as i32 - 128).abs() < 20, "{l} {m} {r}");
    }

    #[test]
    fn gradient_follows_the_layer_transform() {
        let g = Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), &[(0.0, "#000000"), (1.0, "#ffffff")]);
        let mut l = Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_paint(g)]);
        l.transform.position = Vec2::new(20.0, 0.0);
        let pm = render(&project_with(vec![l]));
        assert!(px(&pm, 21, 5)[0] < 60 && px(&pm, 28, 5)[0] > 190, "{:?} {:?}", px(&pm, 21, 5), px(&pm, 28, 5));
    }

    #[test]
    fn radial_gradient_centre_is_first_stop() {
        let g = Paint::radial(Vec2::new(20.0, 20.0), 20.0, &[(0.0, "#ffffff"), (1.0, "#000000")]);
        let pm = render(&project_with(vec![Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_paint(g)])]));
        assert!(px(&pm, 20, 20)[0] > 235);
        assert!(px(&pm, 2, 20)[0] < 40);
    }

    #[test]
    fn stroke_dash_leaves_gaps() {
        let mut path = VPath::new("l", false, vec![crate::path::Anchor::corner(Vec2::new(0.0, 20.0)), crate::path::Anchor::corner(Vec2::new(40.0, 20.0))]);
        let mut st = StrokeStyle::solid("#000000", 4.0);
        st.cap = Cap::Butt;
        st.dash = vec![10.0, 10.0];
        path.stroke = Some(st);
        let pm = render(&project_with(vec![Layer::vector("v", vec![path])]));
        assert_eq!(px(&pm, 5, 20)[3], 255, "inside a dash");
        assert_eq!(px(&pm, 15, 20)[3], 0, "inside a gap");
        assert_eq!(px(&pm, 25, 20)[3], 255);
    }

    #[test]
    fn variable_width_stroke_tapers() {
        let mut path = VPath::new("l", false, vec![crate::path::Anchor::corner(Vec2::new(2.0, 20.0)), crate::path::Anchor::corner(Vec2::new(38.0, 20.0))]);
        let mut st = StrokeStyle::solid("#000000", 12.0);
        st.cap = Cap::Butt;
        st.width_profile = vec![[0.0, 1.0], [1.0, 0.0]];
        path.stroke = Some(st);
        let pm = render(&project_with(vec![Layer::vector("v", vec![path])]));
        assert!(px(&pm, 5, 16)[3] > 0, "wide near the start");
        assert_eq!(px(&pm, 35, 16)[3], 0, "thin near the end");
    }

    #[test]
    fn group_opacity_composites_the_flattened_group() {
        // Two overlapping opaque shapes at 50 % group opacity: the overlap must not be darker.
        let a = Layer::vector("a", vec![VPath::rect("r", 0.0, 0.0, 30.0, 40.0).with_fill("#000000")]);
        let b = Layer::vector("b", vec![VPath::rect("r", 10.0, 0.0, 30.0, 40.0).with_fill("#000000")]);
        let mut g = Layer::group("g", vec![a, b]);
        g.opacity = 0.5;
        let pm = render(&project_with(vec![g]));
        let (only_a, overlap) = (px(&pm, 5, 20)[3], px(&pm, 20, 20)[3]);
        assert!((only_a as i32 - 128).abs() <= 2 && (overlap as i32 - 128).abs() <= 2, "{only_a} {overlap}");
    }

    #[test]
    fn multiply_blend_darkens_the_backdrop() {
        let base = Layer::vector("base", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_fill("#ff8000")]);
        let mut top = Layer::vector("top", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_fill("#808080")]);
        top.blend = Blend::Multiply;
        let pm = render(&project_with(vec![base, top]));
        let c = px(&pm, 20, 20);
        assert!((c[0] as i32 - 128).abs() <= 3 && (c[1] as i32 - 64).abs() <= 3 && c[2] <= 3, "{c:?}");
    }

    #[test]
    fn masks_cut_and_invert() {
        let mut hole = Layer::vector("hole", vec![VPath::rect("r", 10.0, 10.0, 20.0, 20.0).with_fill("#ffffff")]);
        hole.visible = false;
        let mut cut = Layer::vector("cut", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_fill("#ff0000")]);
        cut.mask = Some(MaskRef { layer: "hole".into(), invert: false });
        let pm = render(&project_with(vec![hole.clone(), cut.clone()]));
        assert_eq!(px(&pm, 20, 20), [255, 0, 0, 255], "inside the mask");
        assert_eq!(px(&pm, 3, 3)[3], 0, "outside the mask");

        cut.mask = Some(MaskRef { layer: "hole".into(), invert: true });
        let pm = render(&project_with(vec![hole, cut]));
        assert_eq!(px(&pm, 20, 20)[3], 0, "inverted: hole punched out");
        assert_eq!(px(&pm, 3, 3), [255, 0, 0, 255]);
    }

    #[test]
    fn scale_option_scales_output_size() {
        let e = project_with(vec![]);
        let pm = render_pixmap(&e, 0.0, &RasterOptions { scale: 2.0, show_bones: false, transparent: false });
        assert_eq!((pm.width(), pm.height()), (80, 80));
    }

    #[test]
    fn demo_renders_to_png_and_loops() {
        let e = Engine::new(demo::project()).unwrap();
        let o = RasterOptions::default();
        let png = render_png(&e, 12.0, &o).unwrap();
        assert_eq!(&png[1..4], b"PNG");
        let a = render_rgba(&e, 0.0, &o).2;
        let end = render_rgba(&e, e.project.frames as f32, &o).2;
        let mid = render_rgba(&e, 12.0, &o).2;
        assert_eq!(a, end);
        assert_ne!(a, mid);
    }
}
