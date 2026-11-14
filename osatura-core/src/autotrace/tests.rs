// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `autotrace`.

use super::*;

fn make(w: usize, h: usize, f: impl Fn(usize, usize) -> [u8; 4]) -> Image {
    let mut v = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        for x in 0..w {
            v.extend_from_slice(&f(x, y));
        }
    }
    Image::from_rgba(w, h, v).unwrap()
}
const WHITE: [u8; 4] = [255, 255, 255, 255];
const RED: [u8; 4] = [220, 30, 30, 255];
const BLUE: [u8; 4] = [30, 40, 200, 255];
const NONE: [u8; 4] = [0, 0, 0, 0];

fn bbox(p: &VPath) -> (f32, f32, f32, f32) {
    let mut b = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for a in &p.anchors {
        b = (b.0.min(a.pos.x), b.1.min(a.pos.y), b.2.max(a.pos.x), b.3.max(a.pos.y));
    }
    b
}

fn disc_img(w: usize, h: usize, cx: f32, cy: f32, r: f32, fg: [u8; 4], bg: [u8; 4]) -> Image {
    make(w, h, |x, y| if ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt() <= r { fg } else { bg })
}

#[test]
fn rectangle_gives_exact_box_and_four_straight_corners() {
    let img = make(60, 40, |x, y| if (10..30).contains(&x) && (10..20).contains(&y) { RED } else { WHITE });
    let r = trace(&img, &TraceOptions::default()).unwrap();
    assert_eq!(r.regions.len(), 2);
    // back to front: white background first, red rectangle on top
    assert_eq!(r.regions[0].color, [255, 255, 255]);
    let red = &r.regions[1];
    assert_eq!(red.color, [220, 30, 30]);
    assert_eq!(red.pixels, 200);
    assert_eq!(bbox(&red.outer), (10.0, 10.0, 30.0, 20.0));
    assert_eq!(red.outer.anchors.len(), 4, "exactly the four corners");
    assert!(red.outer.anchors.iter().all(|a| a.handle_in.is_none() && a.handle_out.is_none()), "straight edges carry no handles");
    assert!((red.outer_area - 200.0).abs() < 1e-3);
    assert!(red.holes.is_empty(), "the white background has no transparent hole to cut");
}

#[test]
fn disc_area_bbox_and_curve_economy() {
    let img = disc_img(80, 80, 40.0, 40.0, 25.0, RED, WHITE);
    let r = trace(&img, &TraceOptions::default()).unwrap();
    assert_eq!(r.regions.len(), 2);
    let d = &r.regions[1];
    let b = bbox(&d.outer);
    assert!((b.0 - 15.0).abs() < 1.5 && (b.2 - 65.0).abs() < 1.5 && (b.1 - 15.0).abs() < 1.5 && (b.3 - 65.0).abs() < 1.5, "bbox {b:?}");
    let exact = std::f32::consts::PI * 625.0;
    assert!((d.pixels as f32 - exact).abs() / exact < 0.02);
    assert!(d.outer.anchors.len() <= 24, "{} anchors for a circle", d.outer.anchors.len());
    assert!(d.outer.anchors.iter().any(|a| a.handle_out.is_some()), "curved, not polygonal");
    let f = measure(&img, &r).unwrap();
    assert!(f.mae < 1.5 && f.ssim > 0.97, "{f:?}");
}

#[test]
fn transparent_hole_is_cut_and_filled_hole_is_stacked() {
    // ring on a transparent background: outer region with one cut-out hole
    let ring = make(80, 80, |x, y| {
        let d = ((x as f32 + 0.5 - 40.0).powi(2) + (y as f32 + 0.5 - 40.0).powi(2)).sqrt();
        if d <= 30.0 && d > 12.0 { RED } else { NONE }
    });
    let r = trace(&ring, &TraceOptions::default()).unwrap();
    assert_eq!(r.regions.len(), 1);
    assert_eq!(r.regions[0].holes.len(), 1);
    let svg = to_svg(&r);
    assert!(svg.contains("evenodd") && svg.matches('M').count() == 2, "{svg}");
    let f = measure(&ring, &r).unwrap();
    assert!(f.mae < 2.0, "{f:?}");
    // render: the centre must stay empty (alpha 0)
    let eng = crate::Engine::new(to_project(&r, "t")).unwrap();
    let (w, _, px) = crate::raster::render_rgba(&eng, 0.0, &crate::raster::RasterOptions { transparent: true, ..Default::default() });
    assert_eq!(px[(40 * w as usize + 40) * 4 + 3], 0, "hole is empty");
    assert!(px[(40 * w as usize + 12) * 4 + 3] > 200, "ring is painted");
    // the same ring on an opaque blue background with a white centre: nothing is cut, three regions stack
    let solid = make(80, 80, |x, y| {
        let d = ((x as f32 + 0.5 - 40.0).powi(2) + (y as f32 + 0.5 - 40.0).powi(2)).sqrt();
        if d <= 12.0 { WHITE } else if d <= 30.0 { RED } else { BLUE }
    });
    let r = trace(&solid, &TraceOptions::default()).unwrap();
    assert_eq!(r.regions.len(), 3);
    assert!(r.regions.iter().all(|x| x.holes.is_empty()));
    assert!(r.regions.windows(2).all(|w| w[0].outer_area >= w[1].outer_area), "enclosing regions first");
    let f = measure(&solid, &r).unwrap();
    assert!(f.mae < 2.0 && f.ssim > 0.97, "{f:?}");
}

#[test]
fn triangle_keeps_three_corners_and_star_keeps_its_points() {
    let tri = make(80, 80, |x, y| {
        let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
        // triangle (40,10) (70,65) (10,65)
        let s = |a: (f32, f32), b: (f32, f32)| (b.0 - a.0) * (fy - a.1) - (b.1 - a.1) * (fx - a.0);
        let (d1, d2, d3) = (s((40.0, 10.0), (70.0, 65.0)), s((70.0, 65.0), (10.0, 65.0)), s((10.0, 65.0), (40.0, 10.0)));
        if (d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0) || (d1 <= 0.0 && d2 <= 0.0 && d3 <= 0.0) { RED } else { WHITE }
    });
    let r = trace(&tri, &TraceOptions::default()).unwrap();
    let t = &r.regions[1];
    let corners = t.outer.anchors.iter().filter(|a| {
        let (i, o) = ((a.pos - a.hin()).normalized(), (a.hout() - a.pos).normalized());
        a.handle_in.is_none() || a.handle_out.is_none() || i.dot(o) < 0.85
    }).count();
    assert!((3..=6).contains(&corners), "{corners} corner anchors");
    let b = bbox(&t.outer);
    assert!((b.0 - 10.0).abs() < 2.0 && (b.2 - 70.0).abs() < 2.0 && (b.1 - 10.0).abs() < 2.0 && (b.3 - 65.0).abs() < 2.0, "{b:?}");
    let f = measure(&tri, &r).unwrap();
    assert!(f.mae < 3.0 && f.ssim > 0.93, "{f:?}");
}

#[test]
fn schneider_fit_stays_within_tolerance_on_a_known_curve() {
    let b = [p(0.0, 0.0), p(30.0, 80.0), p(70.0, -40.0), p(100.0, 30.0)];
    let d: Vec<P> = (0..=100).map(|i| bez_at(&b, i as f64 / 100.0)).collect();
    let t1 = (b[1] - b[0]).norm();
    let t2 = (b[2] - b[3]).norm();
    let mut out = Vec::new();
    fit_cubic(&d, t1, t2, 0.25, &mut out); // tolerance 0.5 px, squared
    assert!(out.len() <= 2, "{} segments", out.len());
    // sample the fitted curve densely: every data point must be within 0.5 px of it
    let mut worst = 0.0f64;
    for q in &d {
        let mut best = f64::MAX;
        for s in &out {
            for k in 0..=400 {
                best = best.min((bez_at(&s.p, k as f64 / 400.0) - *q).len());
            }
        }
        worst = worst.max(best);
    }
    assert!(worst <= 0.5, "worst distance {worst}");
    // a straight line is a single segment with the data on it
    let line: Vec<P> = (0..20).map(|i| p(i as f64, 2.0 * i as f64)).collect();
    let mut out = Vec::new();
    fit_cubic(&line, (line[1] - line[0]).norm(), (line[18] - line[19]).norm(), 0.01, &mut out);
    assert_eq!(out.len(), 1);
    // two points
    let mut out = Vec::new();
    fit_cubic(&[p(0.0, 0.0), p(3.0, 4.0)], p(0.6, 0.8), p(-0.6, -0.8), 0.01, &mut out);
    assert_eq!(out.len(), 1);
}

#[test]
fn deterministic_and_seed_sensitive_only_where_it_should_be() {
    let img = make(64, 64, |x, y| [(x * 4) as u8, (y * 4) as u8, ((x + y) * 2) as u8, 255]);
    let o = TraceOptions { colors: 6, ..Default::default() };
    let a = trace(&img, &o).unwrap();
    let b = trace(&img, &o).unwrap();
    assert_eq!(a.palette, b.palette);
    assert_eq!(a.regions.len(), b.regions.len());
    assert_eq!(to_svg(&a), to_svg(&b), "same input, same output, byte for byte");
    let mc = trace(&img, &TraceOptions { quant: Quant::MedianCut, ..o.clone() }).unwrap();
    assert!(mc.palette.len() <= 6 && mc.palette.len() >= 2);
    assert_eq!(to_svg(&mc), to_svg(&trace(&img, &TraceOptions { quant: Quant::MedianCut, ..o }).unwrap()));
}

#[test]
fn quantiser_keeps_few_colours_exactly_and_reduces_many() {
    let img = make(30, 10, |x, _| if x < 10 { RED } else if x < 20 { BLUE } else { WHITE });
    let (pal, lab) = quantise(&img, 8, Quant::KMeans, 1);
    assert_eq!(pal.len(), 3);
    assert!(pal.contains(&[220, 30, 30]) && pal.contains(&[30, 40, 200]));
    assert_eq!(lab.len(), 300);
    let grad = make(256, 4, |x, _| [x as u8, x as u8, x as u8, 255]);
    let (pal, _) = quantise(&grad, 4, Quant::KMeans, 7);
    assert_eq!(pal.len(), 4);
    assert!(pal.windows(2).all(|w| lum(w[0]) <= lum(w[1])), "sorted dark to light");
    let (pal, lab) = quantise(&make(4, 4, |_, _| NONE), 4, Quant::KMeans, 1);
    assert!(pal.is_empty() && lab.iter().all(|&l| l == TRANSP));
}

#[test]
fn gradient_becomes_bands_with_bounded_error() {
    let grad = make(96, 32, |x, _| { let v = (x * 255 / 95) as u8; [v, v / 2, 255 - v, 255] });
    let o = TraceOptions { colors: 8, ..Default::default() };
    let r = trace(&grad, &o).unwrap();
    assert!(r.regions.len() >= 6 && r.regions.len() <= 12, "{}", r.regions.len());
    let f = measure(&grad, &r).unwrap();
    // 8 bands over 255 levels: half a step is about 16 levels
    assert!(f.mae < 12.0, "{f:?}");
    let f16 = measure(&grad, &trace(&grad, &TraceOptions { colors: 16, ..Default::default() }).unwrap()).unwrap();
    assert!(f16.mae < f.mae, "more colours, less error: {} vs {}", f16.mae, f.mae);
}

#[test]
fn despeckle_and_majority_filter_remove_noise() {
    let mut img = make(40, 40, |_, _| WHITE);
    img.rgba[(20 * 40 + 20) * 4..(20 * 40 + 20) * 4 + 4].copy_from_slice(&RED);
    img.rgba[(5 * 40 + 5) * 4..(5 * 40 + 5) * 4 + 4].copy_from_slice(&BLUE);
    let keep = trace(&img, &TraceOptions { min_area: 0.0, denoise: 0, ..Default::default() }).unwrap();
    assert_eq!(keep.regions.len(), 3, "specks traced when asked to");
    let clean = trace(&img, &TraceOptions { min_area: 4.0, denoise: 0, ..Default::default() }).unwrap();
    assert_eq!(clean.regions.len(), 1);
    let maj = trace(&img, &TraceOptions { min_area: 0.0, denoise: 1, ..Default::default() }).unwrap();
    assert_eq!(maj.regions.len(), 1, "a lone pixel loses the 3x3 vote");
    // a one pixel wide sliver between two areas disappears with denoise, a 4 px band stays
    let sliver = make(40, 20, |x, _| if x < 19 { RED } else if x == 19 { BLUE } else { WHITE });
    assert_eq!(trace(&sliver, &TraceOptions { denoise: 1, ..Default::default() }).unwrap().regions.len(), 3, "a boundary sliver ties the 3x3 vote and stays");
    let inner = make(40, 20, |x, y| if x == 10 && y > 2 && y < 17 { BLUE } else { RED });
    assert_eq!(trace(&inner, &TraceOptions { denoise: 1, ..Default::default() }).unwrap().regions.len(), 1, "a line inside an area loses the vote");
    let band = make(40, 20, |x, _| if x < 14 { RED } else if x < 18 { BLUE } else { WHITE });
    assert_eq!(trace(&band, &TraceOptions { denoise: 1, ..Default::default() }).unwrap().regions.len(), 3);
}

#[test]
fn border_following_finds_outer_and_hole_with_signed_area() {
    let (w, h) = (12, 12);
    let inside = |x: usize, y: usize| (2..10).contains(&x) && (2..10).contains(&y) && !((5..7).contains(&x) && (5..7).contains(&y));
    let loops = border_loops(w, h, &inside);
    assert_eq!(loops.len(), 2);
    let mut areas: Vec<i64> = loops.iter().map(|l| signed_area_i(&l.pts) / 2).collect();
    areas.sort();
    assert_eq!(areas, vec![-4, 64], "hole -4, outer +64");
    // two diagonal pixels stay separate (4-connectivity)
    let d = |x: usize, y: usize| (x, y) == (1, 1) || (x, y) == (2, 2);
    assert_eq!(border_loops(5, 5, &d).len(), 2);
}

#[test]
fn zhang_suen_thins_a_bar_to_one_pixel_and_keeps_connectivity() {
    let (w, h) = (40usize, 15usize);
    let bar: Vec<bool> = (0..w * h).map(|i| { let (x, y) = (i % w, i / w); (3..37).contains(&x) && (5..10).contains(&y) }).collect();
    let sk = zhang_suen(&bar, w, h);
    let n = sk.iter().filter(|&&b| b).count();
    assert!((26..=40).contains(&n), "{n} skeleton pixels");
    for x in 8..32 {
        let col: Vec<usize> = (0..h).filter(|&y| sk[y * w + x]).collect();
        assert_eq!(col.len(), 1, "column {x}: {col:?}");
        assert!((6..=8).contains(&col[0]));
    }
    assert!(sk.iter().zip(&bar).all(|(s, b)| !*s || *b), "skeleton lies inside the shape");
    assert_eq!(zhang_suen(&sk, w, h), sk, "thinning a skeleton changes nothing");
}

#[test]
fn line_mode_traces_a_cross_and_a_ring() {
    // white page, black plus sign 3 px thick
    let plus = make(60, 60, |x, y| if ((28..31).contains(&x) && (8..52).contains(&y)) || ((28..31).contains(&y) && (8..52).contains(&x)) { [0, 0, 0, 255] } else { WHITE });
    let o = TraceOptions { mode: Mode::Line, ..Default::default() };
    let r = trace(&plus, &o).unwrap();
    assert!((4..=6).contains(&r.strokes.len()), "{} strokes", r.strokes.len());
    assert!(r.strokes.iter().all(|s| (s.width - 3.0).abs() <= 1.0), "{:?}", r.strokes.iter().map(|s| s.width).collect::<Vec<_>>());
    let f = measure(&plus, &r).unwrap();
    assert!(f.mae < 12.0 && f.ssim > 0.7, "{f:?}");
    let ring = make(70, 70, |x, y| { let d = ((x as f32 + 0.5 - 35.0).powi(2) + (y as f32 + 0.5 - 35.0).powi(2)).sqrt(); if (22.0..25.0).contains(&d) { [0, 0, 0, 255] } else { WHITE } });
    let r = trace(&ring, &o).unwrap();
    assert_eq!(r.strokes.len(), 1);
    assert!(r.strokes[0].path.closed);
    assert!((r.strokes[0].width - 3.0).abs() <= 1.0);
    let f = measure(&ring, &r).unwrap();
    assert!(f.mae < 10.0 && f.ssim > 0.7, "{f:?}");
    // blank page and specks only
    let blank = make(20, 20, |_, _| WHITE);
    assert!(trace(&blank, &o).unwrap().strokes.is_empty());
    // inverted: white strokes on black
    let inv = make(60, 60, |x, y| if (28..31).contains(&x) && (8..52).contains(&y) { WHITE } else { [0, 0, 0, 255] });
    let r = trace(&inv, &TraceOptions { mode: Mode::Line, invert: true, ..Default::default() }).unwrap();
    assert_eq!(r.strokes.len(), 1);
}

#[test]
fn otsu_splits_two_clusters() {
    let mut h = [0usize; 256];
    h[40] = 100;
    h[200] = 100;
    let t = otsu(&h);
    assert!((40..200).contains(&(t as usize)));
    assert_eq!(otsu(&[0; 256]), 128);
}

#[test]
fn simplifiers_keep_corners_and_respect_tolerance() {
    let pts: Vec<Vec2> = (0..=20).map(|i| Vec2::new(i as f32, if i < 10 { 0.0 } else { (i - 10) as f32 })).collect();
    let s = douglas_peucker(&pts, 0.1);
    assert_eq!(s.len(), 3, "start, bend, end");
    assert_eq!(s[1], Vec2::new(10.0, 0.0));
    let noisy: Vec<Vec2> = (0..=20).map(|i| Vec2::new(i as f32, if i % 2 == 0 { 0.0 } else { 0.2 })).collect();
    assert_eq!(douglas_peucker(&noisy, 0.5).len(), 2);
    let sq = vec![Vec2::new(0.0, 0.0), Vec2::new(5.0, 0.1), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)];
    assert_eq!(douglas_peucker_closed(&sq, 0.5).len(), 4);
    assert_eq!(visvalingam_closed(&sq, 1.0).len(), 4, "the 0.5 area triangle goes");
    assert_eq!(visvalingam_closed(&sq, 0.0).len(), 5);
    let img = disc_img(60, 60, 30.0, 30.0, 20.0, RED, WHITE);
    let poly = trace(&img, &TraceOptions { fit: Fit::Polygon, tol: 1.0, ..Default::default() }).unwrap();
    assert!(poly.regions[1].outer.anchors.iter().all(|a| a.handle_in.is_none() && a.handle_out.is_none()));
    let vw = trace(&img, &TraceOptions { fit: Fit::PolygonVw, ..Default::default() }).unwrap();
    assert!(vw.regions[1].outer.anchors.len() >= 8);
}

#[test]
fn fidelity_metrics_behave() {
    let img = make(32, 32, |x, y| [(x * 8) as u8, (y * 8) as u8, 100, 255]);
    let same = compare(&img.rgba, &img.rgba, 32, 32).unwrap();
    assert!(same.mae == 0.0 && same.ssim > 0.999 && same.psnr >= 99.0);
    let inv: Vec<u8> = img.rgba.chunks_exact(4).flat_map(|p| [255 - p[0], 255 - p[1], 255 - p[2], 255]).collect();
    let d = compare(&img.rgba, &inv, 32, 32).unwrap();
    assert!(d.mae > 50.0 && d.ssim < 0.5 && d.psnr < 20.0, "{d:?}");
    assert!(compare(&img.rgba, &inv[..100], 32, 32).is_err());
}

#[test]
fn project_integration_and_names() {
    let img = disc_img(40, 40, 20.0, 20.0, 10.0, RED, NONE);
    let r = trace(&img, &TraceOptions::default()).unwrap();
    let mut p = Project::new("host", 200.0, 100.0, 24.0, 1);
    assert_eq!(append_to_project(&mut p, &r, "traced", true).unwrap(), "traced");
    assert_eq!(append_to_project(&mut p, &r, "traced", false).unwrap(), "traced_2");
    assert!(append_to_project(&mut p, &r, "a/b", false).is_err());
    assert!(append_to_project(&mut p, &r, " ", false).is_err());
    let l = &p.layers[0];
    assert!((l.transform.scale.x - 2.5).abs() < 1e-5, "40 px image fitted into 100 px stage height");
    crate::Engine::new(p.clone()).expect("project with traced layers validates");
    let j = p.to_json_pretty();
    assert!(Project::from_json(&j).is_ok());
}

#[test]
fn bad_input_is_an_error_not_a_panic() {
    assert!(decode_image(b"not an image").is_err());
    assert!(decode_image(&[]).is_err());
    assert!(decode_image(&[0xFF, 0xD8, 0xFF, 0x00]).is_err());
    assert!(Image::from_rgba(0, 4, vec![]).is_err());
    assert!(Image::from_rgba(2, 2, vec![0; 3]).is_err());
    let img = make(8, 8, |_, _| WHITE);
    for bad in [
        TraceOptions { colors: 1, ..Default::default() },
        TraceOptions { colors: 65, ..Default::default() },
        TraceOptions { tol: 0.0, ..Default::default() },
        TraceOptions { tol: f32::NAN, ..Default::default() },
        TraceOptions { min_area: -1.0, ..Default::default() },
        TraceOptions { corner_angle: 0.0, ..Default::default() },
        TraceOptions { corner_window: 1, ..Default::default() },
        TraceOptions { smooth: 1000, ..Default::default() },
    ] {
        assert!(trace(&img, &bad).is_err());
    }
    // a 1x1 image and an all-transparent image
    let one = make(1, 1, |_, _| RED);
    assert_eq!(trace(&one, &TraceOptions::default()).unwrap().regions.len(), 1);
    let none = make(5, 5, |_, _| NONE);
    assert!(trace(&none, &TraceOptions::default()).unwrap().regions.is_empty());
}

#[test]
fn benchmark_runs_and_meets_floor() {
    let rows = benchmark();
    assert_eq!(rows.len(), 8);
    for r in &rows {
        assert!(r.fidelity.mae.is_finite() && r.anchors > 0, "{r:?}");
    }
    let get = |n: &str, m: Mode, c: usize, t: f32| rows.iter().find(|r| r.name == n && r.mode == m && r.colors == c && r.tol == t).unwrap();
    let shapes = get("shapes", Mode::Fill, 6, 1.0);
    assert!(shapes.fidelity.mae < 4.0 && shapes.fidelity.ssim > 0.9, "{shapes:?}");
    let line = get("strokes", Mode::Line, 2, 1.0);
    assert!(line.fidelity.mae < 15.0 && line.fidelity.ssim > 0.6, "{line:?}");
    assert!(get("gradient", Mode::Fill, 32, 1.0).fidelity.mae < get("gradient", Mode::Fill, 8, 1.0).fidelity.mae);
    assert!(synthetic("nope").is_none());
}

#[test]
fn png_round_trip_decodes_with_alpha() {
    let img = disc_img(24, 24, 12.0, 12.0, 8.0, RED, NONE);
    let png = crate::open_formats::encode_rgba_png(24, 24, &img.rgba).unwrap();
    let back = decode_image(&png).unwrap();
    assert_eq!((back.w, back.h), (24, 24));
    assert_eq!(back.rgba[(12 * 24 + 12) * 4..(12 * 24 + 12) * 4 + 4], RED);
    assert_eq!(back.rgba[3], 0);
}
