// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! SVG frame renderer.

use crate::engine::{Engine, Frame, NodeKind, RenderNode};
use crate::math::{Affine2, Vec2};
use crate::path::{n2, width_outline, VPath};
use crate::style::{Blend, Cap, Join, Paint, Rgba};
use std::fmt::Write;

#[derive(Clone, Debug, Default)]
pub struct RenderOptions {
    /// Draw the skeleton on top (heads as dots, bones as lines).
    pub show_bones: bool,
}

pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn clean(v: f32) -> f32 {
    // avoid "-0.00000" and sub-precision noise, so identical poses print identically
    if v.abs() < 5e-6 {
        0.0
    } else {
        v
    }
}

fn matrix(m: &Affine2) -> String {
    format!(
        "matrix({:.5} {:.5} {:.5} {:.5} {:.3} {:.3})",
        clean(m.a),
        clean(m.b),
        clean(m.c),
        clean(m.d),
        clean(m.tx),
        clean(m.ty)
    )
}

struct Out<'a> {
    frame: &'a Frame,
    defs: String,
    next_id: usize,
    width: f32,
    height: f32,
}

/// `(attribute value, opacity)` for a paint; gradients are emitted into `<defs>`.
/// `force`: a flat colour that replaces the paint's colours but keeps its alpha (mask sources: the
/// raster masks by the source's alpha, so the SVG mask is luminance of white x that alpha).
fn paint_value(o: &mut Out, paint: &Paint, force: Option<&str>) -> (String, f32) {
    // a gradient without extent is its last stop everywhere (SVG says so too; written flat to be safe)
    if let Some(c) = paint.degenerate_color() {
        return (force.map_or_else(|| Rgba { a: 1.0, ..c }.to_hex(), |f| f.to_string()), c.a);
    }
    match paint {
        Paint::Solid(c) => (force.map_or_else(|| Rgba { a: 1.0, ..*c }.to_hex(), |f| f.to_string()), c.a),
        Paint::Linear { from, to, stops } => {
            let id = o.next_id;
            o.next_id += 1;
            let _ = write!(
                o.defs,
                r#"<linearGradient id="g{id}" gradientUnits="userSpaceOnUse" x1="{}" y1="{}" x2="{}" y2="{}">"#,
                from.x, from.y, to.x, to.y
            );
            stop_list(&mut o.defs, stops, force);
            o.defs.push_str("</linearGradient>\n");
            (format!("url(#g{id})"), 1.0)
        }
        Paint::Radial { center, radius, stops } => {
            let id = o.next_id;
            o.next_id += 1;
            let _ = write!(
                o.defs,
                r#"<radialGradient id="g{id}" gradientUnits="userSpaceOnUse" cx="{}" cy="{}" r="{}">"#,
                center.x, center.y, radius
            );
            stop_list(&mut o.defs, stops, force);
            o.defs.push_str("</radialGradient>\n");
            (format!("url(#g{id})"), 1.0)
        }
    }
}

fn stop_list(s: &mut String, stops: &[crate::style::Stop], force: Option<&str>) {
    for st in &crate::style::sorted_stops(stops) {
        let _ = write!(
            s,
            r#"<stop offset="{}" stop-color="{}" stop-opacity="{}"/>"#,
            st.offset.clamp(0.0, 1.0),
            force.map_or_else(|| Rgba { a: 1.0, ..st.color }.to_hex(), |f| f.to_string()),
            st.color.a
        );
    }
}

fn poly_d(polys: &[Vec<Vec2>]) -> String {
    let mut d = String::new();
    for poly in polys {
        for (i, p) in poly.iter().enumerate() {
            let _ = write!(d, "{}{} {} ", if i == 0 { "M" } else { "L" }, crate::path::n2(p.x), crate::path::n2(p.y));
        }
        d.push('Z');
        d.push(' ');
    }
    d
}

fn emit_image(o: &mut Out, node: &RenderNode, force: Option<&str>, body: &mut String) {
    let Some(img) = &node.image else { return };
    let tf = matrix(&node.world);
    let (pw, ph) = (img.data.pixmap.width() as f32, img.data.pixmap.height() as f32);
    let href = format!("data:image/png;base64,{}", crate::base64::encode(&img.data.png));
    match &img.mesh {
        None => {
            if let Some(c) = force {
                let _ = writeln!(
                    body,
                    r#"<rect x="{}" y="{}" width="{}" height="{}" transform="{tf}" fill="{c}"/>"#,
                    n2(img.origin.x), n2(img.origin.y), n2(img.size.x), n2(img.size.y)
                );
            } else {
                let _ = writeln!(
                    body,
                    r#"<image href="{href}" x="{}" y="{}" width="{}" height="{}" preserveAspectRatio="none" transform="{tf}"/>"#,
                    n2(img.origin.x), n2(img.origin.y), n2(img.size.x), n2(img.size.y)
                );
            }
        }
        Some(mesh) => {
            let _ = writeln!(body, r#"<g transform="{tf}">"#);
            for tri in &mesh.tris {
                let ix = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
                let uv = [mesh.uvs[ix[0]], mesh.uvs[ix[1]], mesh.uvs[ix[2]]];
                let po = [mesh.positions[ix[0]], mesh.positions[ix[1]], mesh.positions[ix[2]]];
                let Some(m) = crate::math::tri_affine(uv, po) else { continue };
                let c = (po[0] + po[1] + po[2]) * (1.0 / 3.0);
                let g: Vec<Vec2> = po
                    .iter()
                    .map(|&p| {
                        let d = p - c;
                        let l = d.length();
                        if l < 1e-6 { p } else { p + d * (0.6 / l) }
                    })
                    .collect();
                let pts = format!(
                    "{},{} {},{} {},{}",
                    n2(g[0].x), n2(g[0].y), n2(g[1].x), n2(g[1].y), n2(g[2].x), n2(g[2].y)
                );
                if let Some(col) = force {
                    let _ = writeln!(body, r#"<polygon points="{pts}" fill="{col}"/>"#);
                    continue;
                }
                let cid = o.next_id;
                o.next_id += 1;
                let _ = writeln!(
                    body,
                    r#"<clipPath id="c{cid}"><polygon points="{pts}"/></clipPath><g clip-path="url(#c{cid})"><image href="{href}" width="{}" height="{}" transform="{}"/></g>"#,
                    n2(pw), n2(ph), matrix(&m)
                );
            }
            body.push_str("</g>\n");
        }
    }
}

fn emit_path(o: &mut Out, node: &RenderNode, path: &VPath, force: Option<&str>, body: &mut String) {
    if path.anchors.len() < 2 {
        return;
    }
    let tf = matrix(&node.world);
    let d = path.to_svg_d();
    let (fill, fill_op) = match &path.fill {
        Some(p) => paint_value(o, p, force),
        None => ("none".to_string(), 1.0),
    };
    let _ = write!(body, r#"<path d="{d}" transform="{tf}" fill="{fill}""#);
    if fill_op < 1.0 {
        let _ = write!(body, r#" fill-opacity="{fill_op:.3}""#);
    }
    match &path.stroke {
        Some(st) if st.width_profile.is_empty() => {
            let (sv, so) = paint_value(o, &st.paint, force);
            let cap = match st.cap {
                Cap::Butt => "butt",
                Cap::Round => "round",
                Cap::Square => "square",
            };
            let join = match st.join {
                Join::Miter => "miter",
                Join::Round => "round",
                Join::Bevel => "bevel",
            };
            let _ = write!(
                body,
                r#" stroke="{sv}" stroke-width="{}" stroke-linecap="{cap}" stroke-linejoin="{join}""#,
                st.width
            );
            if so < 1.0 {
                let _ = write!(body, r#" stroke-opacity="{so:.3}""#);
            }
            if !st.dash.is_empty() {
                let dash: Vec<String> = st.dash.iter().map(|v| v.to_string()).collect();
                let _ = write!(body, r#" stroke-dasharray="{}" stroke-dashoffset="{}""#, dash.join(" "), st.dash_offset);
            }
            body.push_str("/>\n");
        }
        Some(st) => {
            body.push_str("/>\n");
            // Variable width: filled outline in the stroke paint.
            let tol = 0.25 / node.world.x_scale().max(0.01);
            if let Some(out) = width_outline(&path.polyline(tol), st) {
                let (sv, so) = paint_value(o, &st.paint, force);
                let rule = if out.even_odd { "evenodd" } else { "nonzero" };
                let _ = write!(
                    body,
                    r#"<path d="{}" transform="{tf}" fill="{sv}" fill-rule="{rule}""#,
                    poly_d(&out.polys)
                );
                if so < 1.0 {
                    let _ = write!(body, r#" fill-opacity="{so:.3}""#);
                }
                body.push_str("/>\n");
            }
        }
        None => body.push_str("/>\n"),
    }
}

/// `force` paints everything in one flat colour and ignores visibility, blend
/// and nested masks (used to build mask luminance).
fn emit_node(o: &mut Out, id: usize, force: Option<&str>, body: &mut String) {
    let frame = o.frame;
    let node = &frame.nodes[id];
    if !node.visible && force.is_none() {
        return;
    }
    let simple = force.is_some();
    let filter = if simple { None } else { crate::layerfx::svg_filter(&node.effects, &format!("fx{}", o.next_id), o.width, o.height) };
    let wrap = node.opacity < 0.999 || filter.is_some() || (!simple && (node.blend != Blend::Normal || node.mask.is_some()));
    if wrap {
        let mut attrs = String::new();
        if let Some(f) = filter {
            let _ = write!(attrs, r#" filter="url(#fx{})""#, o.next_id);
            o.next_id += 1;
            o.defs.push_str(&f);
        }
        if node.opacity < 0.999 {
            let _ = write!(attrs, r#" opacity="{:.3}""#, node.opacity);
        }
        if !simple && node.blend != Blend::Normal {
            let _ = write!(attrs, r#" style="mix-blend-mode:{}""#, node.blend.css());
        }
        if let (false, Some((mid, invert))) = (simple, node.mask) {
            let mask_id = o.next_id;
            o.next_id += 1;
            let (bg, fg) = if invert { ("#fff", "#000") } else { ("#000", "#fff") };
            let mut inner = String::new();
            emit_node(o, mid, Some(fg), &mut inner);
            let _ = writeln!(
                o.defs,
                r#"<mask id="m{mask_id}" maskUnits="userSpaceOnUse" x="0" y="0" width="{w}" height="{h}" style="mask-type:luminance"><rect width="{w}" height="{h}" fill="{bg}"/>{inner}</mask>"#,
                w = o.width,
                h = o.height
            );
            let _ = write!(attrs, r#" mask="url(#m{mask_id})""#);
        }
        let _ = writeln!(body, r#"<g id="{}"{attrs}>"#, esc(&node.name));
    }
    match node.kind {
        NodeKind::Vector => {
            for p in &node.paths {
                emit_path(o, node, p, force, body);
            }
        }
        NodeKind::Group | NodeKind::Switch => {
            for &c in &node.children {
                emit_node(o, c, force, body);
            }
        }
        NodeKind::Image => emit_image(o, node, force, body),
    }
    if wrap {
        body.push_str("</g>\n");
    }
}

/// One frame as an SVG fragment (no `<svg>` wrapper, no background) painted in a single `tint`, for onion-skin
/// ghosts. `outline` draws hairline outlines (screen-constant width), otherwise tinted silhouettes. `only` limits it
/// to the listed layer paths and their descendants. Hidden layers are skipped. Masks, blend modes and per-layer
/// opacity are ignored on purpose: a ghost is a flat reference shape. With `moving_vs = Some(f)`, shapes that are
/// drawn identically at frame `f` are left out, so static backgrounds do not wash over the picture.
pub fn ghost_fragment(engine: &Engine, frame: f32, tint: &str, outline: bool, only: Option<&[String]>, moving_vs: Option<f32>) -> String {
    let f = engine.evaluate(frame);
    let cur = moving_vs.map(|m| engine.evaluate(m));
    let p = &engine.project;
    let mut o = Out { frame: &f, defs: String::new(), next_id: 0, width: p.width, height: p.height };
    let mut oc = cur.as_ref().map(|c| Out { frame: c, defs: String::new(), next_id: 0, width: p.width, height: p.height });
    let mut body = String::new();
    fn leaf(o: &mut Out, id: usize, tint: &str, outline: bool) -> String {
        let node = &o.frame.nodes[id];
        let mut body = String::new();
        match node.kind {
            NodeKind::Vector => {
                for path in &node.paths {
                    if !outline {
                        emit_path(o, node, path, Some(tint), &mut body);
                    } else if path.anchors.len() >= 2 {
                        let _ = writeln!(
                            body,
                            r#"<path d="{}" transform="{}" fill="none" stroke="{tint}" stroke-width="1.5" vector-effect="non-scaling-stroke"/>"#,
                            path.to_svg_d(),
                            matrix(&node.world)
                        );
                    }
                }
            }
            NodeKind::Image => {
                if outline {
                    if let Some(img) = &node.image {
                        let _ = writeln!(
                            body,
                            r#"<rect x="{}" y="{}" width="{}" height="{}" transform="{}" fill="none" stroke="{tint}" stroke-width="1.5" vector-effect="non-scaling-stroke"/>"#,
                            n2(img.origin.x), n2(img.origin.y), n2(img.size.x), n2(img.size.y), matrix(&node.world)
                        );
                    }
                } else {
                    emit_image(o, node, Some(tint), &mut body);
                }
            }
            _ => {}
        }
        body
    }
    // recursive walker threading its emission context; kept flat to avoid per-node struct rebuilding.
    #[allow(clippy::too_many_arguments)]
    fn walk(o: &mut Out, oc: &mut Option<Out>, id: usize, tint: &str, outline: bool, only: Option<&[String]>, inside: bool, body: &mut String) {
        let frame = o.frame;
        let node = &frame.nodes[id];
        if !node.visible || node.opacity <= 0.001 {
            return;
        }
        let inside = inside || only.map(|l| l.contains(&node.name)).unwrap_or(true);
        match node.kind {
            NodeKind::Vector | NodeKind::Image if inside => {
                let s = leaf(o, id, tint, outline);
                let same = oc.as_mut().map(|c| leaf(c, id, tint, outline) == s).unwrap_or(false);
                if !same {
                    body.push_str(&s);
                }
            }
            NodeKind::Group | NodeKind::Switch => {
                for &c in &node.children {
                    walk(o, oc, c, tint, outline, only, inside, body);
                }
            }
            _ => {}
        }
    }
    for &r in &f.roots {
        walk(&mut o, &mut oc, r, tint, outline, only, false, &mut body);
    }
    body
}

/// Render the project at `frame` as a standalone SVG document.
pub fn render_frame(engine: &Engine, frame: f32, opts: &RenderOptions) -> String {
    let f = engine.evaluate(frame);
    render_evaluated(engine, &f, frame, opts)
}

/// Like [`render_frame`] for an already evaluated (and possibly simplified, see `scrub`) frame.
pub fn render_evaluated(engine: &Engine, f: &Frame, frame: f32, opts: &RenderOptions) -> String {
    let p = &engine.project;
    let mut o = Out { frame: f, defs: String::new(), next_id: 0, width: p.width, height: p.height };
    let mut body = String::new();
    for &r in &f.roots {
        emit_node(&mut o, r, None, &mut body);
    }
    if opts.show_bones {
        body.push_str(r##"<g id="bones" stroke="#d6336c" fill="#d6336c" stroke-width="2" opacity="0.9">"##);
        body.push('\n');
        for (b, w) in p.bones.iter().zip(&f.world) {
            if !crate::bonevis::visible(p, &b.name, frame) {
                continue;
            }
            let head = f.camera.apply(w.apply(Vec2::ZERO));
            let tip = f.camera.apply(w.apply(Vec2::new(b.length, 0.0)));
            let _ = writeln!(
                body,
                r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}"/><circle cx="{:.2}" cy="{:.2}" r="4" stroke="none"/>"#,
                head.x, head.y, tip.x, tip.y, head.x, head.y
            );
        }
        body.push_str("</g>\n");
    }

    let mut s = String::new();
    let _ = writeln!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">"#,
        w = p.width,
        h = p.height
    );
    if !o.defs.is_empty() {
        let _ = writeln!(s, "<defs>\n{}</defs>", o.defs);
    }
    if let Some(bg) = &p.background {
        let _ = writeln!(
            s,
            r#"<rect width="100%" height="100%" fill="{}" fill-opacity="{:.3}"/>"#,
            Rgba { a: 1.0, ..*bg }.to_hex(),
            bg.a
        );
    }
    s.push_str(&body);
    s.push_str("</svg>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;
    use crate::project::{Layer, MaskRef, Project};
    use crate::style::{Paint, StrokeStyle};

    /// C7: the SVG mask keeps the mask source's alpha (the raster masks by that alpha): a source filled
    /// at alpha 0.376 inverts to 159, not to 0 or 255. Also: unsorted gradient stops come out sorted.
    #[test]
    fn mask_keeps_source_alpha_and_gradient_stops_are_sorted() {
        let mut p = Project::new("m", 40.0, 40.0, 12.0, 1);
        let mut target = Layer::vector("t", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_fill("#ff0000")]);
        target.mask = Some(MaskRef { layer: "h".into(), invert: true });
        let mut h = Layer::vector("h", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_fill("#ffffff60")]);
        h.visible = false;
        p.layers = vec![target, h];
        let e = Engine::new(p).unwrap();
        let svg = render_frame(&e, 0.0, &RenderOptions { show_bones: false });
        assert!(svg.contains(r##"fill="#000" fill-opacity="0.376""##) || svg.contains("fill-opacity=\"0.376\""), "{svg}");
        // gradient stops are sorted by offset
        let mut q = Project::new("g", 40.0, 40.0, 12.0, 1);
        let mut l = Layer::vector("g", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0)]);
        if let crate::project::LayerKind::Vector { paths } = &mut l.kind {
            paths[0].fill = Some(Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(40.0, 0.0), &[(1.0, "#0000ff"), (0.0, "#ff0000")]));
        }
        q.layers = vec![l];
        let svg = render_frame(&Engine::new(q).unwrap(), 0.0, &RenderOptions { show_bones: false });
        let a = svg.find(r#"offset="0""#).unwrap();
        let b = svg.find(r#"offset="1""#).unwrap();
        assert!(a < b, "stop at 0 comes first: {svg}");
    }

    #[test]
    fn renders_demo_frame() {
        let e = Engine::new(demo::project()).unwrap();
        let svg = render_frame(&e, 12.0, &RenderOptions { show_bones: true });
        assert!(svg.starts_with("<svg"));
        assert!(svg.trim_end().ends_with("</svg>"));
        assert!(svg.contains(r#"id="bones""#));
        assert!(svg.matches("<path").count() >= 3);
    }

    #[test]
    fn frames_differ_over_time_and_loop() {
        let e = Engine::new(demo::project()).unwrap();
        let o = RenderOptions::default();
        let a = render_frame(&e, 0.0, &o);
        let b = render_frame(&e, 12.0, &o);
        let end = render_frame(&e, e.project.frames as f32, &o);
        assert_ne!(a, b);
        assert_eq!(a, end, "demo loops seamlessly");
    }

    #[test]
    fn escapes_attribute_text() {
        assert_eq!(esc(r#"a"<&>"#), "a&quot;&lt;&amp;&gt;");
    }

    fn one_layer(l: Layer) -> Engine {
        let mut p = Project::new("t", 100.0, 100.0, 24.0, 1);
        p.layers = vec![l];
        Engine::new(p).unwrap()
    }

    #[test]
    fn gradients_dash_and_opacity_are_emitted() {
        let mut path = VPath::rect("r", 0.0, 0.0, 50.0, 50.0).with_paint(Paint::linear(
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            &[(0.0, "#ff0000"), (1.0, "#0000ff80")],
        ));
        let mut st = StrokeStyle::solid("#000", 2.0);
        st.dash = vec![4.0, 2.0];
        path.stroke = Some(st);
        let mut l = Layer::vector("v", vec![path]);
        l.opacity = 0.5;
        l.blend = Blend::Multiply;
        let svg = render_frame(&one_layer(l), 0.0, &RenderOptions::default());
        assert!(svg.contains("<linearGradient"), "{svg}");
        assert!(svg.contains(r#"stop-opacity="0.50"#) || svg.contains("stop-opacity=\"0.5"), "{svg}");
        assert!(svg.contains("stroke-dasharray=\"4 2\""));
        assert!(svg.contains("opacity=\"0.500\""));
        assert!(svg.contains("mix-blend-mode:multiply"));
    }

    #[test]
    fn masks_become_luminance_masks_and_invert_swaps_colours() {
        let mut p = Project::new("t", 100.0, 100.0, 24.0, 1);
        let shape = Layer::vector("shape", vec![VPath::rect("r", 0.0, 0.0, 100.0, 100.0).with_fill("#f00")]);
        let mut hole = Layer::vector("hole", vec![VPath::circle("c", Vec2::new(50.0, 50.0), 20.0).with_fill("#00f")]);
        hole.visible = false;
        let mut masked = shape.clone();
        masked.name = "masked".into();
        masked.mask = Some(MaskRef { layer: "hole".into(), invert: false });
        let mut inv = shape;
        inv.name = "inv".into();
        inv.mask = Some(MaskRef { layer: "hole".into(), invert: true });
        p.layers = vec![hole, masked, inv];
        let svg = render_frame(&Engine::new(p).unwrap(), 0.0, &RenderOptions::default());
        assert_eq!(svg.matches("<mask").count(), 2, "{svg}");
        assert!(svg.contains(r##"<rect width="100" height="100" fill="#000"/>"##));
        assert!(svg.contains(r##"<rect width="100" height="100" fill="#fff"/>"##));
        assert!(svg.contains("mask=\"url(#m"));
    }

    #[test]
    fn variable_width_stroke_becomes_an_outline_path() {
        let mut path = VPath::new(
            "line",
            false,
            vec![crate::path::Anchor::corner(Vec2::new(0.0, 0.0)), crate::path::Anchor::corner(Vec2::new(80.0, 0.0))],
        );
        let mut st = StrokeStyle::solid("#000", 10.0);
        st.width_profile = vec![[0.0, 1.0], [1.0, 0.1]];
        path.stroke = Some(st);
        let svg = render_frame(&one_layer(Layer::vector("v", vec![path])), 0.0, &RenderOptions::default());
        assert!(svg.contains("fill-rule=\"nonzero\""), "{svg}");
        assert!(!svg.contains("stroke-width"), "variable-width strokes are outlines, not strokes");
    }

    #[test]
    fn hidden_layers_are_skipped() {
        let mut l = Layer::vector("v", vec![VPath::rect("r", 0.0, 0.0, 5.0, 5.0).with_fill("#000")]);
        l.visible = false;
        let svg = render_frame(&one_layer(l), 0.0, &RenderOptions::default());
        assert_eq!(svg.matches("<path").count(), 0);
    }
}
