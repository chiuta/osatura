// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! GPU-vs-CPU parity tests. The CPU reference is the tiny-skia rasteriser in osatura-core.
//! Tests skip (with a message on stderr) when no adapter can be opened. Set OSATURA_GPU_REQUIRE=1 to make that a failure.
//! Numbers are printed with `cargo test -p osatura-gpu -- --nocapture`.

use osatura_core::engine::Engine;
use osatura_core::math::Vec2;
use osatura_core::path::{Anchor, VPath};
use osatura_core::project::{Layer, MaskRef, Project};
use osatura_core::raster::{render_pixmap, RasterOptions};
use osatura_core::style::{Blend, Cap, Join, Paint, Rgba, StrokeStyle};
use osatura_gpu::compare::{compare, Parity};
use osatura_gpu::{render, AdapterChoice, GpuContext, GpuOptions};
use std::sync::OnceLock;

fn ctx() -> Option<&'static GpuContext> {
    static CTX: OnceLock<Option<GpuContext>> = OnceLock::new();
    let c = CTX.get_or_init(|| match GpuContext::new(AdapterChoice::Software) {
        Ok(c) => {
            eprintln!("parity: using adapter {}", c.adapter);
            Some(c)
        }
        Err(e) => {
            eprintln!("SKIP parity tests: {e}");
            None
        }
    });
    if c.is_none() && std::env::var("OSATURA_GPU_REQUIRE").map(|v| !v.is_empty() && v != "0").unwrap_or(false) {
        panic!("OSATURA_GPU_REQUIRE is set but no adapter is available");
    }
    c.as_ref()
}

fn engine(w: f32, h: f32, bg: Option<&str>, layers: Vec<Layer>) -> Engine {
    let mut p = Project::new("t", w, h, 24.0, 1);
    p.background = bg.map(Rgba::hex);
    p.layers = layers;
    Engine::new(p).unwrap()
}

#[derive(Clone, Copy)]
struct Limits {
    max_abs: u8,
    mean_abs: f64,
    frac_over: f64,
    min_ssim: f64,
}

/// Render on both back ends, print the numbers, check the limits.
fn parity(name: &str, e: &Engine, scale: f32, lim: Limits) -> Option<Parity> {
    let ctx = ctx()?;
    let cpu = render_pixmap(e, 0.0, &RasterOptions { scale, ..Default::default() });
    let gpu = render(ctx, e, 0.0, &GpuOptions { scale, ..Default::default() }).unwrap_or_else(|err| panic!("{name}: {err}"));
    assert_eq!((gpu.width, gpu.height), (cpu.width(), cpu.height()), "{name}: size");
    let p = compare(cpu.data(), &gpu.premul_rgba, gpu.width, gpu.height, 24).unwrap();
    eprintln!("parity {name:<22} {p}");
    if let Ok(dir) = std::env::var("OSATURA_GPU_DUMP") {
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(format!("{dir}/{name}_cpu.png"), cpu.encode_png().unwrap());
        let _ = std::fs::write(format!("{dir}/{name}_gpu.png"), gpu.encode_png().unwrap());
    }
    assert!(p.max_abs <= lim.max_abs, "{name}: max_abs {} > {}", p.max_abs, lim.max_abs);
    assert!(p.mean_abs <= lim.mean_abs, "{name}: mean_abs {} > {}", p.mean_abs, lim.mean_abs);
    assert!(p.frac_over <= lim.frac_over, "{name}: frac_over {} > {}", p.frac_over, lim.frac_over);
    assert!(p.ssim >= lim.min_ssim, "{name}: ssim {} < {}", p.ssim, lim.min_ssim);
    Some(p)
}

// Documented tolerances (see ROADMAP R-110). Edge pixels differ because the CPU uses analytic coverage and the GPU a 4x4 box.
const FILL: Limits = Limits { max_abs: 80, mean_abs: 0.35, frac_over: 0.02, min_ssim: 0.995 };

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn stroke(p: VPath, s: StrokeStyle) -> VPath {
    p.with_stroke_style(s)
}

fn line(name: &str, pts: &[(f32, f32)], closed: bool) -> VPath {
    VPath::new(name, closed, pts.iter().map(|&(x, y)| Anchor::corner(v(x, y))).collect())
}

fn style(hex: &str, w: f32, cap: Cap, join: Join) -> StrokeStyle {
    let mut s = StrokeStyle::solid(hex, w);
    s.cap = cap;
    s.join = join;
    s
}

#[test]
fn scene_solid_fills() {
    let star = VPath::polygon("star", &[v(60.0, 8.0), v(70.0, 40.0), v(44.0, 20.0), v(76.0, 20.0), v(50.0, 40.0)]).with_fill("#2266cc");
    let l = Layer::vector(
        "v",
        vec![
            VPath::rect("r", 4.0, 4.0, 30.0, 20.0).with_fill("#cc2222"),
            VPath::circle("c", v(20.0, 44.0), 12.0).with_fill("#22aa44"),
            star,
            VPath::ellipse("e", v(60.0, 52.0), 14.0, 6.0).with_fill("#ffaa0080"),
        ],
    );
    let e = engine(80.0, 64.0, Some("#ffffff"), vec![l]);
    parity("solid_fills", &e, 1.0, FILL);
    parity("solid_fills_x2", &e, 2.0, FILL);
}

#[test]
fn scene_transparent_background() {
    let e = engine(48.0, 48.0, None, vec![Layer::vector("v", vec![VPath::circle("c", v(24.0, 24.0), 16.0).with_fill("#8844cc")])]);
    parity("transparent_bg", &e, 1.0, Limits { mean_abs: 0.6, ..FILL });
}

#[test]
fn scene_strokes_caps_and_joins() {
    let zig = [(6.0, 40.0), (20.0, 10.0), (32.0, 40.0), (46.0, 10.0)];
    let mut paths = Vec::new();
    for (i, (cap, join)) in [(Cap::Butt, Join::Miter), (Cap::Round, Join::Round), (Cap::Square, Join::Bevel)].into_iter().enumerate() {
        let pts: Vec<(f32, f32)> = zig.iter().map(|&(x, y)| (x, y + i as f32 * 48.0)).collect();
        paths.push(stroke(line(&format!("z{i}"), &pts, false), style("#112233", 7.0, cap, join)));
    }
    paths.push(stroke(line("closed", &[(60.0, 10.0), (90.0, 14.0), (80.0, 40.0), (62.0, 30.0)], true), style("#aa3300", 5.0, Cap::Butt, Join::Miter)));
    paths.push(stroke(VPath::circle("c", v(76.0, 80.0), 14.0), style("#006688", 4.0, Cap::Butt, Join::Round)));
    let e = engine(100.0, 150.0, Some("#f4f4f4"), vec![Layer::vector("v", paths)]);
    parity("strokes_caps_joins", &e, 1.0, Limits { max_abs: 110, mean_abs: 0.5, frac_over: 0.025, min_ssim: 0.99 });
}

#[test]
fn scene_dashes_and_variable_width() {
    let mut dashed = style("#000000", 4.0, Cap::Butt, Join::Round);
    dashed.dash = vec![8.0, 5.0];
    dashed.dash_offset = 3.0;
    let mut round_dash = style("#c0392b", 5.0, Cap::Round, Join::Round);
    round_dash.dash = vec![1.0, 9.0];
    let mut taper = style("#2c3e50", 14.0, Cap::Butt, Join::Round);
    taper.width_profile = vec![[0.0, 1.0], [1.0, 0.1]];
    let mut ring = style("#16a085", 10.0, Cap::Butt, Join::Round);
    ring.width_profile = vec![[0.0, 0.3], [0.5, 1.0], [1.0, 0.3]];
    let e = engine(
        90.0,
        90.0,
        Some("#ffffff"),
        vec![Layer::vector(
            "v",
            vec![
                stroke(line("d", &[(5.0, 10.0), (85.0, 10.0)], false), dashed),
                stroke(VPath::circle("c", v(25.0, 40.0), 14.0), round_dash),
                stroke(VPath::smooth_through("t", &[v(45.0, 60.0), v(60.0, 30.0), v(85.0, 45.0)], false, 1.0), taper),
                stroke(VPath::circle("ring", v(30.0, 72.0), 12.0), ring),
            ],
        )],
    );
    // tiny-skia measures dash lengths on the cubic form of a straight segment and drifts by up to ~0.25 px over 80 px
    // (and places dots on circles up to ~1 px off); the GPU path dashes the exact flattened polyline. Hence the looser limits.
    parity("dashes_variable", &e, 1.0, Limits { max_abs: 130, mean_abs: 0.9, frac_over: 0.03, min_ssim: 0.98 });
}

#[test]
fn scene_gradients() {
    let lin = Paint::linear(v(0.0, 0.0), v(60.0, 0.0), &[(0.0, "#ff0000"), (0.5, "#ffff00"), (1.0, "#0000ff")]);
    let rad = Paint::radial(v(30.0, 30.0), 24.0, &[(0.0, "#ffffff"), (1.0, "#003366")]);
    let alpha = Paint::linear(v(0.0, 0.0), v(0.0, 30.0), &[(0.0, "#000000ff"), (1.0, "#00000000")]);
    let diag = Paint::linear(v(0.0, 0.0), v(30.0, 30.0), &[(0.2, "#00ff00"), (0.8, "#ff00ff")]);
    let mut l = Layer::vector(
        "v",
        vec![
            VPath::rect("a", 0.0, 0.0, 60.0, 20.0).with_paint(lin),
            VPath::rect("b", 0.0, 22.0, 60.0, 60.0).with_paint(rad),
            VPath::rect("d", 0.0, 84.0, 60.0, 20.0).with_paint(diag),
        ],
    );
    l.transform.position = v(5.0, 3.0);
    let mut g2 = Layer::vector("g2", vec![VPath::rect("c", 0.0, 0.0, 40.0, 30.0).with_paint(alpha)]);
    g2.transform.position = v(70.0, 10.0);
    let st = StrokeStyle::new(Paint::linear(v(0.0, 0.0), v(40.0, 0.0), &[(0.0, "#ff8800"), (1.0, "#0088ff")]), 8.0);
    let mut g3 = Layer::vector("g3", vec![stroke(line("s", &[(0.0, 10.0), (40.0, 10.0)], false), st)]);
    g3.transform.position = v(70.0, 60.0);
    let e = engine(115.0, 112.0, Some("#ffffff"), vec![l, g2, g3]);
    // Smooth colour: tolerance is in 8-bit steps of interpolation rounding.
    parity("gradients", &e, 1.0, Limits { max_abs: 80, mean_abs: 0.5, frac_over: 0.02, min_ssim: 0.995 });
}

#[test]
fn scene_opacity_and_groups() {
    let a = Layer::vector("a", vec![VPath::rect("r", 0.0, 0.0, 30.0, 40.0).with_fill("#000000")]);
    let b = Layer::vector("b", vec![VPath::rect("r", 14.0, 8.0, 30.0, 40.0).with_fill("#cc0000")]);
    let mut g = Layer::group("g", vec![a, b]);
    g.opacity = 0.5;
    g.transform.position = v(4.0, 4.0);
    let mut single = Layer::vector("s", vec![VPath::circle("c", v(70.0, 30.0), 18.0).with_fill("#0044ff")]);
    single.opacity = 0.35;
    let e = engine(100.0, 60.0, Some("#ffffff"), vec![g, single]);
    parity("opacity_groups", &e, 1.0, Limits { max_abs: 90, mean_abs: 0.4, frac_over: 0.02, min_ssim: 0.995 });
}

#[test]
fn scene_blend_modes() {
    let modes = [
        Blend::Multiply, Blend::Screen, Blend::Overlay, Blend::Darken, Blend::Lighten, Blend::ColorDodge, Blend::ColorBurn,
        Blend::HardLight, Blend::SoftLight, Blend::Difference, Blend::Exclusion, Blend::Add,
    ];
    let cell = 14.0;
    let g = Paint::linear(v(0.0, 0.0), v(0.0, 36.0), &[(0.0, "#ff6000"), (0.5, "#40c0a0"), (1.0, "#303090")]);
    let mut layers = vec![Layer::vector("base", vec![VPath::rect("r", 0.0, 0.0, cell * modes.len() as f32, 36.0).with_paint(g)])];
    for (i, m) in modes.iter().enumerate() {
        let mut top = Layer::vector(
            &format!("top{i}"),
            vec![VPath::rect("r", i as f32 * cell + 1.0, 2.0, cell - 2.0, 32.0).with_paint(Paint::linear(
                v(0.0, 0.0),
                v(cell, 0.0),
                &[(0.0, "#8040c0"), (1.0, "#e0e080")],
            ))],
        );
        top.transform.position = v(0.0, 0.0);
        top.blend = *m;
        layers.push(top);
    }
    let e = engine(cell * modes.len() as f32, 36.0, Some("#ffffff"), layers);
    let p = parity("blend_modes", &e, 2.0, Limits { max_abs: 60, mean_abs: 0.8, frac_over: 0.03, min_ssim: 0.99 });
    let _ = p;
}

#[test]
fn scene_blend_each_mode_separately() {
    // One scene per mode so a single wrong formula cannot hide in an average.
    let modes = [
        (Blend::Multiply, "multiply"), (Blend::Screen, "screen"), (Blend::Overlay, "overlay"), (Blend::Darken, "darken"),
        (Blend::Lighten, "lighten"), (Blend::ColorDodge, "dodge"), (Blend::ColorBurn, "burn"), (Blend::HardLight, "hardlight"),
        (Blend::SoftLight, "softlight"), (Blend::Difference, "difference"), (Blend::Exclusion, "exclusion"), (Blend::Add, "add"),
    ];
    for (m, name) in modes {
        let base = Layer::vector("base", vec![VPath::rect("r", 0.0, 0.0, 32.0, 32.0).with_paint(Paint::linear(v(0.0, 0.0), v(32.0, 0.0), &[(0.0, "#102030"), (0.5, "#a0b040"), (1.0, "#f0e0d0")]))]);
        let mut top = Layer::vector("top", vec![VPath::rect("r", 4.0, 4.0, 24.0, 24.0).with_paint(Paint::linear(v(0.0, 0.0), v(0.0, 32.0), &[(0.0, "#c06020"), (1.0, "#3060d0")]))]);
        top.blend = m;
        let e = engine(32.0, 32.0, Some("#ffffff"), vec![base, top]);
        parity(&format!("blend_{name}"), &e, 1.0, Limits { max_abs: 40, mean_abs: 0.5, frac_over: 0.03, min_ssim: 0.99 });
    }
}

#[test]
fn scene_nested_transforms() {
    let leaf = Layer::vector("leaf", vec![VPath::rect("r", -6.0, -4.0, 12.0, 8.0).with_fill("#cc3333"), stroke(VPath::circle("c", v(0.0, 0.0), 3.0), StrokeStyle::solid("#ffffff", 1.5))]);
    let mut inner = Layer::group("inner", vec![leaf]);
    inner.transform.position = v(16.0, 0.0);
    inner.transform.rotation = 30.0;
    inner.transform.scale = v(1.5, 0.7);
    inner.transform.skew = 10.0;
    let mut outer = Layer::group("outer", vec![inner]);
    outer.transform.position = v(40.0, 32.0);
    outer.transform.rotation = -50.0;
    outer.transform.scale = v(1.2, 1.2);
    outer.transform.anchor = v(-4.0, 2.0);
    let e = engine(80.0, 64.0, Some("#eeeeee"), vec![outer]);
    parity("nested_transforms", &e, 1.0, FILL);
    parity("nested_transforms_x3", &e, 3.0, FILL);
}

#[test]
fn scene_masks() {
    let mut hole = Layer::vector("hole", vec![VPath::circle("c", v(20.0, 20.0), 12.0).with_fill("#ffffff")]);
    hole.visible = false;
    let mut cut = Layer::vector("cut", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_paint(Paint::linear(v(0.0, 0.0), v(40.0, 0.0), &[(0.0, "#ff0000"), (1.0, "#0000ff")]))]);
    cut.mask = Some(MaskRef { layer: "hole".into(), invert: false });
    let mut cut2 = Layer::vector("cut2", vec![VPath::rect("r", 0.0, 0.0, 40.0, 40.0).with_fill("#008800")]);
    cut2.transform.position = v(44.0, 0.0);
    cut2.mask = Some(MaskRef { layer: "hole".into(), invert: true });
    // The second mask sits at the origin as well, so shift the clipped layer's mask by using a mask copy.
    let mut hole2 = Layer::vector("hole2", vec![VPath::circle("c", v(64.0, 20.0), 12.0).with_fill("#ffffff80")]);
    hole2.visible = false;
    cut2.mask = Some(MaskRef { layer: "hole2".into(), invert: true });
    let e = engine(90.0, 40.0, Some("#ffffff"), vec![hole, hole2, cut, cut2]);
    parity("masks", &e, 1.0, Limits { max_abs: 90, mean_abs: 0.4, frac_over: 0.02, min_ssim: 0.995 });
}

#[test]
fn scene_demo_project() {
    let e = Engine::new(osatura_core::demo::project()).unwrap();
    let Some(ctx) = ctx() else { return };
    for frame in [0.0f32, 12.0] {
        let cpu = render_pixmap(&e, frame, &RasterOptions::default());
        match render(ctx, &e, frame, &GpuOptions::default()) {
            Ok(g) => {
                let p = compare(cpu.data(), &g.premul_rgba, g.width, g.height, 24).unwrap();
                eprintln!("parity demo_frame_{frame:<10} {p}");
                assert!(p.mean_abs < 2.0 && p.ssim > 0.95, "demo frame {frame}: {p}");
            }
            Err(osatura_gpu::GpuError::Unsupported(m)) => eprintln!("demo frame {frame}: unsupported on GPU ({m}), skipped"),
            Err(e) => panic!("{e}"),
        }
    }
}

#[test]
fn unsupported_features_error_clearly() {
    let Some(ctx) = ctx() else { return };
    let e = engine(16.0, 16.0, None, vec![Layer::vector("v", vec![VPath::rect("r", 1.0, 1.0, 5.0, 5.0).with_fill("#000")])]);
    let err = render(ctx, &e, 0.0, &GpuOptions { show_bones: true, ..Default::default() }).err().unwrap();
    assert!(matches!(err, osatura_gpu::GpuError::Unsupported(ref m) if m.contains("bone")), "{err}");
    let err = render(ctx, &e, 0.0, &GpuOptions { scale: -1.0, ..Default::default() }).err().unwrap();
    assert!(matches!(err, osatura_gpu::GpuError::Unsupported(_)));
    let many = Paint::Linear { from: v(0.0, 0.0), to: v(10.0, 0.0), stops: (0..20).map(|i| osatura_core::style::Stop { offset: i as f32 / 19.0, color: Rgba::BLACK }).collect() };
    let e = engine(16.0, 16.0, None, vec![Layer::vector("v", vec![VPath::rect("r", 1.0, 1.0, 5.0, 5.0).with_paint(many)])]);
    let err = render(ctx, &e, 0.0, &GpuOptions::default()).err().unwrap();
    assert!(err.to_string().contains("20 stops"), "{err}");
}

#[test]
fn empty_project_renders_background_only() {
    let Some(ctx) = ctx() else { return };
    let e = engine(8.0, 8.0, Some("#336699"), vec![]);
    let g = render(ctx, &e, 0.0, &GpuOptions::default()).unwrap();
    assert!(g.premul_rgba.chunks(4).all(|p| p == [0x33, 0x66, 0x99, 255]));
    let t = render(ctx, &e, 0.0, &GpuOptions { transparent: true, ..Default::default() }).unwrap();
    assert!(t.premul_rgba.iter().all(|b| *b == 0));
}
