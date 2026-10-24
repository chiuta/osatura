// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `autotrace` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// project integration, SVG
// ---------------------------------------------------------------------------------------------

pub(super) fn rgba_of(c: [u8; 3]) -> Rgba {
    Rgba::new(c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, 1.0)
}

pub(super) fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

/// Build one group layer holding the traced geometry (back to front).
pub fn to_layer(res: &TraceResult, name: &str) -> Layer {
    let mut kids: Vec<Layer> = Vec::new();
    for (i, r) in res.regions.iter().enumerate() {
        let paint = Paint::Solid(rgba_of(r.color));
        let lname = format!("{}_{}", hex(r.color).trim_start_matches('#'), i);
        if r.holes.is_empty() {
            let mut pth = r.outer.clone();
            pth.name = "shape".into();
            kids.push(Layer::vector(&lname, vec![pth.with_paint(paint)]));
        } else {
            let mut paths = vec![r.outer.clone().with_paint(paint.clone())];
            for (k, h) in r.holes.iter().enumerate() {
                let mut hp = h.clone();
                hp.name = format!("hole{k}");
                paths.push(hp.with_paint(paint.clone()));
            }
            paths[0].name = "outer".into();
            kids.push(Layer::boolean(&lname, crate::fx::BoolOp::Difference, paths));
        }
    }
    if !res.strokes.is_empty() {
        let paths: Vec<VPath> = res
            .strokes
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut st = StrokeStyle::new(Paint::Solid(rgba_of(res.stroke_color)), s.width);
                st.cap = Cap::Round;
                st.join = Join::Round;
                let mut pth = s.path.clone();
                pth.name = format!("line{i}");
                pth.with_stroke_style(st)
            })
            .collect();
        kids.push(Layer::vector("lines", paths));
    }
    Layer::group(name, kids)
}

/// A new project, sized like the image, holding the traced group.
pub fn to_project(res: &TraceResult, name: &str) -> Project {
    let mut p = Project::new(name, res.width as f32, res.height as f32, 24.0, 1);
    p.layers.push(to_layer(res, "trace"));
    p
}

/// Append the traced group to an existing project (name made unique). The group is scaled to fit the
/// project's stage when `fit_stage` is set (uniform scale, centred); returns the layer name.
pub fn append_to_project(p: &mut Project, res: &TraceResult, name: &str, fit_stage: bool) -> Result<String, String> {
    if name.trim().is_empty() || name.contains('/') || name.contains("::") {
        return Err("layer name must be non-empty and contain no '/' or '::'".into());
    }
    let mut unique = name.to_string();
    let mut n = 2;
    while p.layers.iter().any(|l| l.name == unique) {
        unique = format!("{name}_{n}");
        n += 1;
    }
    let mut layer = to_layer(res, &unique);
    if fit_stage && res.width > 0 && res.height > 0 {
        let s = (p.width / res.width as f32).min(p.height / res.height as f32);
        if s.is_finite() && s > 0.0 {
            layer.transform.scale = Vec2::new(s, s);
            layer.transform.position = Vec2::new((p.width - res.width as f32 * s) * 0.5, (p.height - res.height as f32 * s) * 0.5);
        }
    }
    p.layers.push(layer);
    Ok(unique)
}

pub(super) fn svg_path_d(path: &VPath) -> String {
    let a = &path.anchors;
    if a.is_empty() {
        return String::new();
    }
    let mut d = format!("M{:.2} {:.2}", a[0].pos.x, a[0].pos.y);
    let seg = |d: &mut String, from: &Anchor, to: &Anchor| {
        if from.handle_out.is_none() && to.handle_in.is_none() {
            d.push_str(&format!("L{:.2} {:.2}", to.pos.x, to.pos.y));
        } else {
            let (c1, c2) = (from.hout(), to.hin());
            d.push_str(&format!("C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2}", c1.x, c1.y, c2.x, c2.y, to.pos.x, to.pos.y));
        }
    };
    for i in 1..a.len() {
        seg(&mut d, &a[i - 1], &a[i]);
    }
    if path.closed {
        seg(&mut d, &a[a.len() - 1], &a[0]);
        d.push('Z');
    }
    d
}

/// Stand-alone SVG. Holes use `fill-rule="evenodd"`.
pub fn to_svg(res: &TraceResult) -> String {
    let mut s = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n", w = res.width, h = res.height);
    for r in &res.regions {
        let mut d = svg_path_d(&r.outer);
        for h in &r.holes {
            d.push_str(&svg_path_d(h));
        }
        s.push_str(&format!("<path d=\"{}\" fill=\"{}\" fill-rule=\"evenodd\"/>\n", d, hex(r.color)));
    }
    for st in &res.strokes {
        s.push_str(&format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>\n",
            svg_path_d(&st.path),
            hex(res.stroke_color),
            st.width
        ));
    }
    s.push_str("</svg>\n");
    s
}

// ---------------------------------------------------------------------------------------------
// fidelity
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Fidelity {
    /// Mean absolute error over R, G, B (0..255), both images composited over white.
    pub mae: f64,
    pub rmse: f64,
    pub psnr: f64,
    /// Mean structural similarity of the luminance (Wang et al. 2004; 8x8 windows, stride 4, uniform weights).
    pub ssim: f64,
}

pub(super) fn over_white(rgba: &[u8]) -> Vec<[f64; 3]> {
    rgba.chunks_exact(4)
        .map(|p| {
            let a = p[3] as f64 / 255.0;
            [p[0] as f64 * a + 255.0 * (1.0 - a), p[1] as f64 * a + 255.0 * (1.0 - a), p[2] as f64 * a + 255.0 * (1.0 - a)]
        })
        .collect()
}

/// Compare two same-size straight-RGBA images.
pub fn compare(a: &[u8], b: &[u8], w: usize, h: usize) -> Result<Fidelity, String> {
    if a.len() != w * h * 4 || b.len() != w * h * 4 {
        return Err("images differ in size".into());
    }
    let (pa, pb) = (over_white(a), over_white(b));
    let (mut abs, mut sq) = (0.0, 0.0);
    for (x, y) in pa.iter().zip(&pb) {
        for c in 0..3 {
            let d = x[c] - y[c];
            abs += d.abs();
            sq += d * d;
        }
    }
    let n = (w * h * 3) as f64;
    let mse = sq / n;
    let psnr = if mse <= 1e-12 { 99.0 } else { 10.0 * (255.0f64 * 255.0 / mse).log10() };
    let ly = |v: &[[f64; 3]]| -> Vec<f64> { v.iter().map(|c| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]).collect() };
    let (la, lb) = (ly(&pa), ly(&pb));
    let win = 8usize.min(w).min(h);
    let (c1, c2) = ((0.01f64 * 255.0).powi(2), (0.03f64 * 255.0).powi(2));
    let (mut total, mut cnt) = (0.0, 0.0);
    let mut y = 0;
    while y + win <= h {
        let mut x = 0;
        while x + win <= w {
            let (mut ma, mut mb) = (0.0, 0.0);
            for j in 0..win {
                for i in 0..win {
                    ma += la[(y + j) * w + x + i];
                    mb += lb[(y + j) * w + x + i];
                }
            }
            let m = (win * win) as f64;
            ma /= m;
            mb /= m;
            let (mut va, mut vb, mut cov) = (0.0, 0.0, 0.0);
            for j in 0..win {
                for i in 0..win {
                    let (u, v) = (la[(y + j) * w + x + i] - ma, lb[(y + j) * w + x + i] - mb);
                    va += u * u;
                    vb += v * v;
                    cov += u * v;
                }
            }
            va /= m - 1.0;
            vb /= m - 1.0;
            cov /= m - 1.0;
            total += ((2.0 * ma * mb + c1) * (2.0 * cov + c2)) / ((ma * ma + mb * mb + c1) * (va + vb + c2));
            cnt += 1.0;
            x += 4;
        }
        y += 4;
    }
    Ok(Fidelity { mae: abs / n, rmse: mse.sqrt(), psnr, ssim: if cnt > 0.0 { total / cnt } else { 1.0 } })
}

/// Rasterise the trace with the engine's own renderer and compare it to the source bitmap.
pub fn measure(img: &Image, res: &TraceResult) -> Result<Fidelity, String> {
    let eng = crate::Engine::new(to_project(res, "fidelity")).map_err(|e| e.join("; "))?;
    let (w, h, px) = crate::raster::render_rgba(&eng, 0.0, &crate::raster::RasterOptions { transparent: true, ..Default::default() });
    if w as usize != img.w || h as usize != img.h {
        return Err("rendered size differs from the source".into());
    }
    compare(&img.rgba, &px, img.w, img.h)
}

// ---------------------------------------------------------------------------------------------
// benchmark on synthetic images (the numbers quoted in the docs come from here)
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct BenchRow {
    pub name: &'static str,
    pub mode: Mode,
    pub colors: usize,
    pub tol: f32,
    pub fidelity: Fidelity,
    pub regions: usize,
    pub strokes: usize,
    pub anchors: usize,
    pub millis: f64,
}

pub(super) fn render_source(p: Project) -> Image {
    let eng = crate::Engine::new(p).expect("synthetic source validates");
    let (w, h, px) = crate::raster::render_rgba(&eng, 0.0, &crate::raster::RasterOptions::default());
    Image { w: w as usize, h: h as usize, rgba: px }
}

/// Deterministic synthetic test images: anti-aliased flat shapes, a smooth gradient, text-like strokes,
/// and the shapes with pixel noise.
pub fn synthetic(name: &str) -> Option<Image> {
    let white = Rgba::new(1.0, 1.0, 1.0, 1.0);
    match name {
        "shapes" | "noisy_shapes" => {
            let mut p = Project::new("s", 160.0, 120.0, 24.0, 1);
            p.background = Some(white);
            p.layers = vec![
                Layer::vector("c", vec![VPath::circle("c", Vec2::new(40.0, 40.0), 26.0).with_fill("#d62828")]),
                Layer::vector("r", vec![VPath::rect("r", 85.0, 15.0, 55.0, 40.0).with_fill("#1d4ed8")]),
                Layer::vector("t", vec![VPath::polygon("t", &[Vec2::new(30.0, 105.0), Vec2::new(75.0, 70.0), Vec2::new(110.0, 110.0)]).with_fill("#15803d")]),
                Layer::vector("o", vec![VPath::circle("o", Vec2::new(125.0, 88.0), 22.0).with_stroke("#111111", 8.0)]),
            ];
            let mut img = render_source(p);
            if name == "noisy_shapes" {
                let mut rng = Rng::new(99);
                for px in img.rgba.chunks_exact_mut(4) {
                    for c in px.iter_mut().take(3) {
                        let n = (rng.unit() * 2.0 - 1.0) * 14.0;
                        *c = (*c as f64 + n).clamp(0.0, 255.0) as u8;
                    }
                }
            }
            Some(img)
        }
        "gradient" => {
            let mut p = Project::new("g", 160.0, 80.0, 24.0, 1);
            p.background = Some(white);
            let paint = Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(160.0, 0.0), &[(0.0, "#1e3a8a"), (0.5, "#f59e0b"), (1.0, "#fef08a")]);
            p.layers = vec![Layer::vector("g", vec![VPath::rect("g", 0.0, 0.0, 160.0, 80.0).with_paint(paint)])];
            Some(render_source(p))
        }
        "strokes" => {
            let mut p = Project::new("t", 160.0, 80.0, 24.0, 1);
            p.background = Some(white);
            let line = |n: &str, pts: &[(f32, f32)]| {
                let v: Vec<Vec2> = pts.iter().map(|&(x, y)| Vec2::new(x, y)).collect();
                VPath::new(n, false, v.into_iter().map(Anchor::corner).collect()).with_stroke("#000000", 4.0)
            };
            let mut paths = vec![
                line("A1", &[(10.0, 65.0), (25.0, 15.0), (40.0, 65.0)]),
                line("A2", &[(16.0, 48.0), (34.0, 48.0)]),
                line("H1", &[(55.0, 15.0), (55.0, 65.0)]),
                line("H2", &[(80.0, 15.0), (80.0, 65.0)]),
                line("H3", &[(55.0, 40.0), (80.0, 40.0)]),
            ];
            paths.push(VPath::ellipse("O", Vec2::new(122.0, 40.0), 16.0, 25.0).with_stroke("#000000", 4.0));
            p.layers = vec![Layer::vector("txt", paths)];
            Some(render_source(p))
        }
        _ => None,
    }
}

/// Run the tracer over the synthetic images and measure each result against its source.
pub fn benchmark() -> Vec<BenchRow> {
    let cases: [(&'static str, Mode, usize, f32, usize); 8] = [
        ("shapes", Mode::Fill, 6, 1.0, 0),
        ("shapes", Mode::Fill, 6, 0.5, 0),
        ("noisy_shapes", Mode::Fill, 6, 1.0, 1),
        ("gradient", Mode::Fill, 8, 1.0, 0),
        ("gradient", Mode::Fill, 32, 1.0, 0),
        ("strokes", Mode::Line, 2, 1.0, 0),
        ("strokes", Mode::Fill, 2, 1.0, 0),
        ("strokes", Mode::Fill, 2, 0.5, 0),
    ];
    let mut rows = Vec::new();
    for (name, mode, colors, tol, denoise) in cases {
        let img = synthetic(name).expect("known case");
        let o = TraceOptions { mode, colors, tol, denoise, ..Default::default() };
        let t0 = crate::clock::Stamp::now();
        let res = trace(&img, &o).expect("benchmark trace");
        let millis = t0.elapsed().as_secs_f64() * 1000.0;
        let fidelity = measure(&img, &res).expect("benchmark measure");
        rows.push(BenchRow { name, mode, colors, tol, fidelity, regions: res.regions.len(), strokes: res.strokes.len(), anchors: res.anchor_count(), millis });
    }
    rows
}
