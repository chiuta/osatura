// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of the size fitter.
#![allow(unused_imports)]

/// C12a: the gzip budget also holds for the system gzip at -9 and at its default -6.
#[test]
fn gzip_budget_holds_for_system_gzip() {
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    if Command::new("gzip").arg("--version").output().is_err() {
        return; // no system gzip here
    }
    let e = crate::Engine::new(crate::demo::project()).unwrap();
    let lottie = crate::export::lottie_json(&e).to_string().into_bytes();
    let html = crate::export::html_player(&e).into_bytes();
    let project = serde_json::to_vec_pretty(&e.project).unwrap();
    for (name, data) in [("lottie", lottie), ("html", html), ("project", project)] {
        let budget = gzip_budget_len(&data);
        for level in ["-9", "-6", "-1"] {
            let mut c = Command::new("gzip").args([level, "-c"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
            let mut stdin = c.stdin.take().unwrap();
            let d = data.clone();
            let t = std::thread::spawn(move || {
                let _ = stdin.write_all(&d);
            });
            let out = c.wait_with_output().unwrap();
            t.join().unwrap();
            match level {
                "-9" => assert!(out.stdout.len() <= budget, "{name} gzip -9: {} > budget {budget}", out.stdout.len()),
                // the default level is a weaker compressor: documented to be up to ~10 % larger
                "-6" => assert!(out.stdout.len() * 100 <= budget * 110, "{name} gzip -6: {} vs budget {budget}", out.stdout.len()),
                _ => {}
            }
        }
    }
}

use super::*;
use crate::anim::{Ease, Extrapolate};
use crate::project::{Channel, Layer, Prop};

/// The demo rig, shrunk so the debug-build tests stay fast, 12 frames.
fn small_demo() -> Project {
    let mut p = crate::containers::responsive(&crate::demo::project(), 96.0, 72.0).unwrap();
    p.frames = 8;
    p
}

/// Flat-colour scene: moving square on a background, a ball with a smooth bob.
fn flat() -> Project {
    let mut p = Project::new("flat", 64.0, 48.0, 12.0, 16);
    p.background = Some(crate::style::Rgba::hex("#ffffff"));
    let sq = Layer::vector("sq", vec![VPath::rect("r", 6.0, 8.0, 14.0, 14.0).with_fill("#cc2222").with_stroke("#000000", 1.5)]);
    let ball = Layer::vector("ball", vec![VPath::circle("c", Vec2::new(40.0, 24.0), 7.0).with_fill("#2255cc")]);
    p.layers = vec![sq, ball];
    p.channels = vec![
        Channel::keys("sq", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (15.0, 30.0, Ease::Linear)])),
        Channel::keys("ball", Prop::LayerY, Track::with_keys(&[(0.0, 0.0, Ease::Smooth), (5.0, -10.0, Ease::Smooth), (10.0, 6.0, Ease::Smooth), (15.0, 0.0, Ease::Smooth)])),
    ];
    p
}

fn opts(fmt: Format, budget: usize) -> FitOptions {
    let mut o = FitOptions::new(fmt, budget);
    o.sample_frames = 2;
    o.baseline = false;
    o
}

fn chain_sizes(p: &Project, fmt: Format) -> Vec<usize> {
    use std::sync::{Mutex, OnceLock};
    type SizeCache = Mutex<HashMap<(Format, String, u32), Vec<usize>>>;
    static CACHE: OnceLock<SizeCache> = OnceLock::new();
    let key = (fmt, p.name.clone(), p.frames);
    if let Some(v) = CACHE.get_or_init(Default::default).lock().unwrap().get(&key) {
        return v.clone();
    }
    let mut f = Fit::new(p, opts(fmt, usize::MAX)).unwrap();
    let v: Vec<usize> = chain(fmt).iter().map(|pos| f.eval(pos).budgeted).collect();
    CACHE.get_or_init(Default::default).lock().unwrap().insert(key, v.clone());
    v
}

// ---- small utilities

#[test]
fn size_parsing() {
    assert_eq!(parse_size("500KB").unwrap(), 500_000);
    assert_eq!(parse_size("1.5 MB").unwrap(), 1_500_000);
    assert_eq!(parse_size("64000").unwrap(), 64_000);
    assert_eq!(parse_size("2kib").unwrap(), 2048);
    assert!(parse_size("fast").is_err() && parse_size("12 parsecs").is_err() && parse_size("0").is_err() && parse_size("").is_err());
}

#[test]
fn precision_rounding_and_number_text() {
    assert_eq!(round_dec(1.2345678, 3), 1.235);
    assert_eq!(round_dec(-0.0004, 2), 0.0);
    assert!(round_dec(-0.0004, 2).is_sign_positive());
    assert_eq!(round_dec(5.5, 0), 6.0);
    assert_eq!(round_dec(0.1 + 0.2, 6), 0.3);
    assert_eq!(fmt_dec(12.30, 2), "12.3");
    assert_eq!(fmt_dec(60.0, 2), "60");
    assert_eq!(fmt_dec(-0.001, 2), "0");
    assert_eq!(fmt_dec(0.125, 2), "0.13");
    assert_eq!(num(3.0), json!(3));
    assert_eq!(num(3.25), json!(3.25));
}

#[test]
fn svg_text_rounding_keeps_identifiers_colours_and_data() {
    let svg = "<svg viewBox=\"0 0 640 480\">\n<g id=\"g12\" opacity=\"1.000\">\n<path d=\"M60.00 412.567 C-0.004 3.14159 1.5-2.25 7 8\" fill=\"#123456\" stroke-width=\"3.456\" clip-path=\"url(#c1.5)\"/>\n<image href=\"data:image/png;base64,AB12.5+9\" transform=\"matrix(0.123456789 0 0 1.5 10.123456 20.5)\"/>\n</g>\n</svg>\n";
    let r = svg_round(svg, 1);
    assert!(r.contains("id=\"g12\"") && r.contains("fill=\"#123456\"") && r.contains("url(#c1.5)") && r.contains("AB12.5+9"));
    assert!(r.contains("d=\"M60 412.6 C0 3.1 1.5-2.3 7 8\""), "{r}");
    assert!(r.contains("stroke-width=\"3.5\""));
    assert!(!r.contains("opacity=\"1\"") && !r.contains('\n'), "{r}");
    assert!(r.contains("matrix(0.1235 0 0 1.5 10.1 20.5)"), "linear part keeps d+3 decimals: {r}");
    // idempotent
    assert_eq!(svg_round(&r, 1), r);
}

#[test]
fn chain_is_a_proper_ladder() {
    for f in [Format::Lottie, Format::Html, Format::Gif, Format::Apng, Format::Webm, Format::Project] {
        let ch = chain(f);
        let ax = axes(f);
        assert_eq!(ch.len(), 1 + ax.iter().filter(|a| a.in_chain).map(|a| a.values.len() - 1).sum::<usize>());
        assert!(ch[0].iter().all(|&p| p == 0));
        assert_eq!(*ch.last().unwrap(), ax.iter().map(|a| if a.in_chain { a.values.len() - 1 } else { 0 }).collect::<Vec<_>>());
        for w in ch.windows(2) {
            let d: Vec<i64> = w[1].iter().zip(&w[0]).map(|(a, b)| *a as i64 - *b as i64).collect();
            assert_eq!(d.iter().filter(|&&x| x != 0).count(), 1, "each step moves one knob");
            assert_eq!(d.iter().sum::<i64>(), 1, "by exactly one rung");
        }
        // rung 0 reduces nothing
        let s0 = settings_at(f, &ch[0]);
        assert!(s0.path_eps == 0.0 && s0.key_tol == 0.0 && s0.frame_step == 1 && s0.scale == 1.0 && s0.delta_tol == 0 && s0.crf == 24, "{f:?}: {s0:?}");
        assert!(s0.decimals >= 2 && s0.drop_dead, "{f:?}: {s0:?}");
        // every axis list is strictly ordered from best to worst
        for a in ax.iter().filter(|a| a.in_chain && !(a.name == "colors" && a.values[0] == 0.0)) {
            assert!(a.values.len() >= 2, "{f:?} {}", a.name);
            let up = a.values[1] > a.values[0];
            assert!(a.values.windows(2).all(|w| (w[1] > w[0]) == up && w[1] != w[0]), "{f:?} {}", a.name);
        }
    }
    assert!(Format::parse("svg").unwrap_err().contains("not supported") && Format::parse("gltf").is_err() && Format::parse("mp3").is_err());
    assert_eq!(Format::from_path("a/b.GIF"), Some(Format::Gif));
    assert_eq!(Format::from_path("x.json"), Some(Format::Lottie));
    assert_eq!(Format::from_path("x.txt"), None);
}

// ---- geometry

#[test]
fn path_simplification_respects_its_tolerance() {
    // a circle with 64 anchors: smooth tangents, so many are redundant
    let n = 64;
    let r = 50.0;
    let nodes: Vec<PNode> = (0..n)
        .map(|k| {
            let a = k as f64 / n as f64 * std::f64::consts::TAU;
            let h = r * 4.0 / 3.0 * (std::f64::consts::TAU / n as f64 / 4.0).tan();
            PNode { p: [r * a.cos(), r * a.sin()], i: [h * a.sin(), -h * a.cos()], o: [-h * a.sin(), h * a.cos()] }
        })
        .collect();
    for eps in [0.05, 0.5, 2.0] {
        let mut fr = vec![nodes.clone()];
        let removed = simplify_paths(&mut fr, true, eps);
        assert_eq!(fr[0].len(), n - removed);
        assert!(fr[0].len() >= 3);
        if eps >= 0.5 {
            assert!(removed > 20, "eps {eps}: removed only {removed}");
        }
        // independent check: sampled distance of the new outline from the true circle <= eps + sampling slack
        let m = fr[0].len();
        let mut worst: f64 = 0.0;
        for k in 0..m {
            let c = seg_ctrl(&fr[0][k], &fr[0][(k + 1) % m]);
            for j in 0..=20 {
                let q = cubic_at(&c, j as f64 / 20.0);
                worst = worst.max((len2(q) - r).abs());
            }
        }
        assert!(worst <= eps * 1.5 + 0.05, "eps {eps}: deviation {worst}");
    }
    // off: unchanged
    let mut fr = vec![nodes.clone()];
    assert_eq!(simplify_paths(&mut fr, true, 0.0), 0);
    assert_eq!(fr[0], nodes);
}

#[test]
fn collinear_vertices_go_corners_stay_and_open_ends_stay() {
    let c = |x: f64, y: f64| PNode { p: [x, y], i: [0.0, 0.0], o: [0.0, 0.0] };
    // open polyline with a collinear middle point and a real corner
    let mut fr = vec![vec![c(0.0, 0.0), c(5.0, 0.0), c(10.0, 0.0), c(10.0, 10.0)]];
    let removed = simplify_paths(&mut fr, false, 0.01);
    assert_eq!(removed, 1);
    assert_eq!(fr[0].iter().map(|n| n.p).collect::<Vec<_>>(), vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]);
    // a closed triangle keeps its 3 corners whatever the tolerance
    let mut tri = vec![vec![c(0.0, 0.0), c(10.0, 0.0), c(5.0, 8.0)]];
    assert_eq!(simplify_paths(&mut tri, true, 100.0), 0);
    // the same vertices go in every frame; a vertex that is needed in only one frame stays
    let mut two = vec![
        vec![c(0.0, 0.0), c(5.0, 0.0), c(10.0, 0.0), c(10.0, 10.0)],
        vec![c(0.0, 0.0), c(5.0, 3.0), c(10.0, 0.0), c(10.0, 10.0)],
    ];
    assert_eq!(simplify_paths(&mut two, false, 0.01), 0);
    assert_eq!(two[0].len(), 4);
    // mismatching topology: nothing happens
    let mut bad = vec![vec![c(0.0, 0.0), c(1.0, 0.0), c(2.0, 0.0)], vec![c(0.0, 0.0), c(2.0, 0.0)]];
    assert_eq!(simplify_paths(&mut bad, false, 1.0), 0);
}

// ---- key reduction

fn lcg(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*seed >> 33) as f32) / (1u64 << 31) as f32
}

#[test]
fn key_reduction_bound_is_exact_for_every_ease_and_extrapolation() {
    let eases = [Ease::Linear, Ease::Step, Ease::EaseIn, Ease::EaseOut, Ease::EaseInOut, Ease::Smooth, Ease::CubicBezier { x1: 0.2, y1: 0.8, x2: 0.7, y2: 0.1 }, Ease::Back { overshoot: 1.2 }, Ease::Bounce];
    let extr = [Extrapolate::Hold, Extrapolate::Linear, Extrapolate::Loop, Extrapolate::PingPong, Extrapolate::LoopOffset];
    let mut seed = 7u64;
    let mut reduced_any = false;
    for trial in 0..60 {
        let n = 3 + (lcg(&mut seed) * 20.0) as usize;
        let mut t = Track::new();
        let mut f = (lcg(&mut seed) * 3.0).floor();
        for _ in 0..n {
            t.set_key(f, (lcg(&mut seed) - 0.5) * 100.0 * if trial % 3 == 0 { 0.01 } else { 1.0 }, eases[(lcg(&mut seed) * eases.len() as f32) as usize % eases.len()]);
            f += 1.0 + (lcg(&mut seed) * 4.0).floor() + if trial % 4 == 0 { 0.5 } else { 0.0 };
        }
        t.extrapolate = extr[trial % extr.len()];
        let frames = (f as u32 + 5).max(10);
        for tol in [0.0f32, 0.001, 0.05, 0.5, 5.0] {
            let r = reduce_track(&t, tol, frames);
            assert!(r.keys.len() <= t.keys.len() && r.keys.len() >= 2);
            assert_eq!(r.keys.first().unwrap().frame, t.keys.first().unwrap().frame);
            assert_eq!(r.keys.last().unwrap().frame, t.keys.last().unwrap().frame);
            // independent evaluation: every integer frame of the timeline and every original key frame
            let mut worst = 0.0f64;
            let mut probes: Vec<f32> = (0..frames).map(|x| x as f32).collect();
            probes.extend(t.keys.iter().map(|k| k.frame));
            for x in probes {
                worst = worst.max((t.eval(x).unwrap() as f64 - r.eval(x).unwrap() as f64).abs());
            }
            assert!(worst <= tol as f64, "trial {trial} tol {tol}: deviation {worst}");
            assert_eq!(worst, track_deviation(&t, &r, frames));
            if tol == 0.0 {
                assert_eq!(worst, 0.0);
            }
            reduced_any |= r.keys.len() < t.keys.len();
        }
    }
    assert!(reduced_any, "the reduction never removed a key");
    // a straight line collapses to its two ends
    let t = Track::with_keys(&[(0.0, 0.0, Ease::Linear), (1.0, 1.0, Ease::Linear), (2.0, 2.0, Ease::Linear), (3.0, 3.0, Ease::Linear)]);
    assert_eq!(reduce_track(&t, 1e-4, 4).keys.len(), 2);
    // a negative or NaN tolerance leaves the track alone
    assert_eq!(reduce_track(&t, f32::NAN, 4), t);
    assert_eq!(reduce_track(&t, -1.0, 4), t);
}

#[test]
fn project_key_reduction_reports_and_bounds_every_channel() {
    let mut p = crate::demo::project();
    let before: usize = p.channels.iter().map(|c| if let Source::Keys { track } = &c.source { track.keys.len() } else { 0 }).sum();
    let orig = p.clone();
    let r = reduce_project_keys(&mut p, 0.5);
    assert_eq!(r.keys_before, before);
    assert!(r.keys_after <= r.keys_before && r.max_deviation <= 0.5);
    for (a, b) in orig.channels.iter().zip(&p.channels) {
        if let (Source::Keys { track: ta }, Source::Keys { track: tb }) = (&a.source, &b.source) {
            assert!(track_deviation(ta, tb, orig.frames) <= 0.5);
        }
    }
    let mut q = orig.clone();
    let z = reduce_project_keys(&mut q, 0.0);
    assert_eq!(z.max_deviation, 0.0);
    let mut e = orig.clone();
    let hi = reduce_project_keys(&mut e, 50.0);
    assert!(hi.keys_after < hi.keys_before, "a large tolerance must drop keys on the demo");
    Engine::new(e).expect("a reduced project still validates");
}

// ---- dead layers

#[test]
fn dead_layers_are_dropped_only_when_provably_invisible() {
    let mut p = flat();
    let mut hidden = Layer::vector("ghost", vec![VPath::rect("g", 1.0, 1.0, 5.0, 5.0).with_fill("#00ff00")]);
    hidden.visible = false;
    let mut clear = Layer::vector("clear", vec![VPath::rect("g", 30.0, 30.0, 5.0, 5.0).with_fill("#00ff00")]);
    clear.opacity = 0.0;
    // a hidden mask source must stay (it shapes another layer)
    let mut msrc = Layer::vector("msrc", vec![VPath::rect("m", 0.0, 0.0, 30.0, 30.0).with_fill("#ffffff")]);
    msrc.visible = false;
    let mut masked = Layer::vector("masked", vec![VPath::rect("g", 0.0, 0.0, 64.0, 48.0).with_fill("#ffaa00")]);
    masked.mask = Some(crate::project::MaskRef { layer: "msrc".into(), invert: false });
    p.layers.extend([hidden, clear, msrc, masked]);
    p.channels.push(Channel::keys("ghost", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (5.0, 9.0, Ease::Linear)])));
    let before = serde_json::to_string(&p).unwrap();
    let (q, dropped) = drop_dead_layers(&p);
    assert_eq!(serde_json::to_string(&p).unwrap(), before, "the input is not touched");
    assert_eq!(dropped, vec!["ghost".to_string(), "clear".to_string()]);
    assert!(q.find_layer("ghost").is_none() && q.find_layer("msrc").is_some() && q.find_layer("masked").is_some());
    assert!(!q.channels.iter().any(|c| c.target == "ghost"), "the dead layer's channels go with it");
    // pixel-identical picture
    let (e0, e1) = (Engine::new(p).unwrap(), Engine::new(q).unwrap());
    let o = RasterOptions { scale: 1.0, show_bones: false, transparent: false };
    for f in [0.0, 7.0, 15.0] {
        assert_eq!(render_rgba(&e0, f, &o).2, render_rgba(&e1, f, &o).2);
    }
    // nothing to drop: same project back
    let (r, d) = drop_dead_layers(&flat());
    assert!(d.is_empty() && serde_json::to_string(&r).unwrap() == serde_json::to_string(&flat()).unwrap());
}

// ---- quantiser, GIF, APNG

fn test_frames(w: usize, h: usize, n: usize, colours: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|k| {
            let mut v = vec![255u8; w * h * 4];
            for y in 0..h {
                for x in 0..w {
                    let o = (y * w + x) * 4;
                    let band = ((x + y) / 3) % colours;
                    let moving = x >= k * 2 && x < k * 2 + 6 && (4..10).contains(&y);
                    let c = if moving { [250, 250, 20] } else { [(band * 40 % 256) as u8, (band * 97 % 256) as u8, (band * 23 % 256) as u8] };
                    v[o..o + 3].copy_from_slice(&c);
                }
            }
            v
        })
        .collect()
}

#[test]
fn median_cut_is_exact_for_few_colours_and_bounded_for_many() {
    let fr = test_frames(24, 16, 3, 5);
    let refs: Vec<&[u8]> = fr.iter().map(|f| f.as_slice()).collect();
    let pal = median_cut(&refs, 256);
    assert_eq!(pal.len(), 6, "5 band colours + the moving block");
    let rich: Vec<Vec<u8>> = (0..2).map(|k| (0..32 * 32).flat_map(|i| [(i * 7 + k) as u8, (i * 13) as u8, (i / 3) as u8, 255]).collect()).collect();
    let r2: Vec<&[u8]> = rich.iter().map(|f| f.as_slice()).collect();
    for max in [2, 16, 64] {
        let p = median_cut(&r2, max);
        assert!(!p.is_empty() && p.len() <= max);
        assert_eq!(p, median_cut(&r2, max), "deterministic");
    }
    // more colours never increases the quantisation error
    let err = |max: usize| {
        let q = quantize(&rich, 32, 32, max + 1, false, true);
        let mut e = 0u64;
        for (f, idx) in rich.iter().zip(&q.frames) {
            for (p, &i) in f.chunks_exact(4).zip(idx) {
                let c = q.palette[i as usize];
                e += (0..3).map(|a| (p[a] as i64 - c[a] as i64).pow(2) as u64).sum::<u64>();
            }
        }
        e
    };
    assert!(err(64) <= err(16) && err(16) <= err(4));
}

#[test]
fn gif_roundtrip_is_exact_for_few_colours_and_deltas_are_lossless() {
    let fr = test_frames(40, 24, 6, 5);
    let q = quantize(&fr, 40, 24, 256, false, true);
    let plain = gif_encode(&q, 12.0, 1, false, 0).unwrap();
    let delta = gif_encode(&q, 12.0, 1, true, 0).unwrap();
    assert!(delta.len() < plain.len(), "deltas shrink a mostly static clip ({} vs {})", delta.len(), plain.len());
    let (w, h, a) = gif_decode(&plain).unwrap();
    let (_, _, b) = gif_decode(&delta).unwrap();
    assert_eq!((w, h, a.len()), (40, 24, 6));
    assert_eq!(a, b, "lossless delta decodes to the same pictures");
    for (orig, dec) in fr.iter().zip(&a) {
        assert_eq!(orig, dec, "few colours: the GIF holds the frames exactly");
    }
    // lossy delta: per-channel error to the indexed truth is bounded by the tolerance
    let q2 = quantize(&test_frames(40, 24, 6, 40), 40, 24, 64, false, true);
    let truth: Vec<Vec<u8>> = q2.frames.iter().map(|f| f.iter().flat_map(|&i| { let c = q2.palette[i as usize]; [c[0], c[1], c[2], 255] }).collect()).collect();
    let (_, _, lossy) = gif_decode(&gif_encode(&q2, 12.0, 1, true, 12).unwrap()).unwrap();
    let mut worst = 0i32;
    for (t, d) in truth.iter().zip(&lossy) {
        for (x, y) in t.chunks_exact(4).zip(d.chunks_exact(4)) {
            if x[0..3] != [0, 0, 0] || y[0..3] != [0, 0, 0] {
                worst = worst.max((0..3).map(|c| (x[c] as i32 - y[c] as i32).abs()).max().unwrap());
            }
        }
    }
    assert!(worst <= 12, "lossy delta error {worst}");
    // real transparency: frames are drawn on a cleared canvas
    let mut t = test_frames(16, 16, 3, 4);
    for f in &mut t {
        for p in f.chunks_exact_mut(4).take(40) {
            p[3] = 0;
        }
    }
    let qt = quantize(&t, 16, 16, 256, false, true);
    let (_, _, dec) = gif_decode(&gif_encode(&qt, 12.0, 1, false, 0).unwrap()).unwrap();
    assert!(dec.iter().all(|f| f.chunks_exact(4).take(40).all(|p| p[3] == 0)));
    assert!(gif_decode(b"not a gif").is_err());
}

#[test]
fn gif_delay_follows_step_and_fps() {
    assert_eq!(delay_cs(24.0, 1), 4);
    assert_eq!(delay_cs(24.0, 3), 13);
    assert_eq!(delay_cs(100.0, 1), 2, "GIF cannot go below 2 cs");
}

#[test]
fn apng_roundtrip_indexed_and_truecolour() {
    let fr = test_frames(40, 24, 5, 5);
    let q = quantize(&fr, 40, 24, 256, false, true);
    assert!(q.palette.len() <= 8);
    for delta in [false, true] {
        let b = apng_encode_indexed(&q, 12.0, 1, delta, 0).unwrap();
        let (w, h, d) = apng_decode(&b).unwrap();
        assert_eq!((w, h, d.len()), (40, 24, 5));
        for (o, x) in fr.iter().zip(&d) {
            assert_eq!(o, x, "delta={delta}");
        }
    }
    let tc = apng_encode_rgba(40, 24, &fr, 12.0, 1).unwrap();
    let (_, _, d) = apng_decode(&tc).unwrap();
    assert_eq!(d, fr);
    // bit depths pack and unpack: 2, 3-4, 5-16 and more colours
    for ncol in [2usize, 4, 13, 40] {
        let f = test_frames(17, 9, 3, ncol);
        let q = quantize(&f, 17, 9, 256, false, true);
        let (_, _, d) = apng_decode(&apng_encode_indexed(&q, 12.0, 1, true, 0).unwrap()).unwrap();
        assert_eq!(d, f, "{ncol} colours");
    }
    assert!(apng_decode(b"nope").is_err());
    assert!(apng_encode_rgba(4, 4, &Vec::<Vec<u8>>::new(), 12.0, 1).is_err());
}

#[test]
fn webm_encodes_and_decodes_when_ffmpeg_exists() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg missing: webm test skipped");
        return;
    }
    let fr = test_frames(33, 21, 4, 5); // odd size: padded and cropped
    let b = webm_encode(33, 21, &fr, 12.0, 30).unwrap();
    assert_eq!(&b[..4], &[0x1a, 0x45, 0xdf, 0xa3], "EBML header");
    let d = webm_decode(&b, 33, 21).unwrap();
    assert_eq!(d.len(), 4);
    assert!(d.iter().all(|f| f.len() == 33 * 21 * 4));
    assert!(webm_decode(b"junk", 33, 21).is_err());
}

// ---- Lottie

#[test]
fn lottie_reduction_honours_precision_and_the_key_bound() {
    let p = small_demo();
    let e = Engine::new(p.clone()).unwrap();
    let (base, map) = crate::export::lottie_json_mapped(&e);
    let n = p.frames as usize;
    // the full rung is lossless to 1e-6 and strictly smaller than the raw exporter output
    let full = lottie_reduce(&base, &Settings::full());
    let (b0, b1) = (base.to_string().len(), full.to_string().len());
    assert!(b1 < b0, "lossless merging and number cleaning shrink the file ({b1} vs {b0})");
    let layers = base["layers"].as_array().unwrap();
    type Shapes = Vec<Vec<Vec<(bool, Vec<PNode>)>>>;
    let read_all = |doc: &Value| -> Shapes {
        doc["layers"].as_array().unwrap().iter().map(|l| {
            l.get("shapes").and_then(Value::as_array).into_iter().flatten().flat_map(|g| g["it"].as_array().unwrap().iter().filter(|i| i["ty"] == "sh")).map(|sh| (0..n).map(|f| read_shape_at(&sh["ks"], f).unwrap()).collect()).collect()
        }).collect()
    };
    let want = read_all(&base);
    let got = read_all(&full);
    let dev = |a: &Shapes, b: &Shapes| -> f64 {
        let mut worst: f64 = 0.0;
        for (la, lb) in a.iter().zip(b) {
            for (ta, tb) in la.iter().zip(lb) {
                for (sa, sb) in ta.iter().zip(tb) {
                    assert_eq!((sa.0, sa.1.len()), (sb.0, sb.1.len()));
                    for (x, y) in sa.1.iter().zip(&sb.1) {
                        for c in 0..2 {
                            worst = worst.max((x.p[c] - y.p[c]).abs()).max((x.i[c] - y.i[c]).abs()).max((x.o[c] - y.o[c]).abs());
                        }
                    }
                }
            }
        }
        worst
    };
    assert!(dev(&want, &got) <= 5.1e-7, "full rung deviation {}", dev(&want, &got));
    assert!(!layers.is_empty() && !map.is_empty());
    // precision: every non-integer number in the shape data has at most d decimals and moved by <= 0.5e-d
    for d in [0u8, 1, 2, 3] {
        let s = Settings { decimals: d, ..Settings::full() };
        let doc = lottie_reduce(&base, &s);
        let text = doc.to_string();
        let m = 10f64.powi(d as i32);
        let got = read_all(&doc);
        assert!(dev(&want, &got) <= 0.5 / m + 1e-9, "d={d}: {}", dev(&want, &got));
        for (_, v) in got.iter().flatten().flatten() {
            for nd in v {
                for x in [nd.p[0], nd.p[1], nd.i[0], nd.i[1], nd.o[0], nd.o[1]] {
                    assert!((x * m - (x * m).round()).abs() < 1e-6, "d={d}: {x}");
                }
            }
        }
        assert!(text.len() < b1 || d == 3, "d={d}");
    }
    // key reduction: linear keys within tol of the baked frames, fewer keyframes, valid JSON
    let count = |doc: &Value| doc.to_string().matches("\"t\":").count();
    let mut last = count(&full);
    for tol in [0.05f32, 0.5, 2.0] {
        let doc = lottie_reduce(&base, &Settings { key_tol: tol, ..Settings::full() });
        let got = read_all(&doc);
        let d = dev(&want, &got);
        assert!(d <= tol as f64 + 1e-5, "tol {tol}: deviation {d}");
        let c = count(&doc);
        assert!(c <= last, "tol {tol}: {c} keyframes vs {last}");
        last = c;
        let _: Value = serde_json::from_str(&doc.to_string()).unwrap();
    }
    assert!(last < count(&full), "a 2 unit tolerance must drop keyframes on the demo");
    // frame decimation: odd frames show the even frame before them (hold)
    let dec = lottie_reduce(&base, &Settings { frame_step: 2, ..Settings::full() });
    let got = read_all(&dec);
    for (la, lb) in want.iter().zip(&got) {
        for (ta, tb) in la.iter().zip(lb) {
            for f in 0..n {
                let (a, b) = (&ta[f - f % 2], &tb[f]);
                assert_eq!((a.0, a.1.len()), (b.0, b.1.len()));
                for (x, y) in a.1.iter().zip(&b.1) {
                    for c in 0..2 {
                        assert!((x.p[c] - y.p[c]).abs() <= 5.1e-7 && (x.i[c] - y.i[c]).abs() <= 5.1e-7 && (x.o[c] - y.o[c]).abs() <= 5.1e-7, "frame {f}");
                    }
                }
            }
        }
    }
}

#[test]
fn lottie_model_matches_the_engine_at_the_top_rung() {
    let p = flat();
    let o = opts(Format::Lottie, usize::MAX);
    let mut f = Fit::new(&p, o).unwrap();
    let c = f.eval(&chain(Format::Lottie)[0]);
    assert!(c.valid, "{:?}", c.error);
    assert!(c.ssim_min > 0.98, "baked Lottie read back vs the engine: {}", c.ssim_min);
    // constant tracks become static and layer opacity too
    let (doc, _) = crate::export::lottie_json_mapped(&Engine::new(p).unwrap());
    let red = lottie_reduce(&doc, &Settings::full());
    let kf = red["layers"][0]["shapes"][0]["it"][0]["ks"].clone();
    assert!(kf["a"] == 1 || kf["a"] == 0);
    assert!(red["layers"][0]["ks"]["o"]["a"] == 0, "constant opacity is static: {}", red["layers"][0]["ks"]["o"]);
}

// ---- HTML

#[test]
fn html_page_is_self_contained_dedupes_and_parses() {
    let p = flat();
    let o = opts(Format::Html, usize::MAX);
    let mut f = Fit::new(&p, o).unwrap();
    let (bytes, imgs) = f.produce(&Settings::full()).unwrap();
    let h = String::from_utf8(bytes).unwrap();
    assert!(h.starts_with("<!doctype html>") && h.contains("requestAnimationFrame") && h.contains("<svg"));
    assert!(!h.contains("src=\"http") && !h.contains("href=\"http"));
    assert_eq!(h.matches("</script>").count(), 1);
    assert_eq!(imgs.len(), 2);
    // a still project: one unique frame, an index list of the right length
    let mut still = flat();
    still.channels.clear();
    let mut f = Fit::new(&still, opts(Format::Html, usize::MAX)).unwrap();
    let (bytes, _) = f.produce(&Settings::full()).unwrap();
    let h = String::from_utf8(bytes).unwrap();
    assert_eq!(h.matches("<svg").count(), 1, "identical frames are stored once");
    assert!(h.contains(",X=[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]"), "{}", &h[h.find("const F").unwrap_or(0)..].chars().take(40).collect::<String>());
    // step halves the displayed frames
    let mut f = Fit::new(&p, opts(Format::Html, usize::MAX)).unwrap();
    let (bytes, _) = f.produce(&Settings { frame_step: 4, ..Settings::full() }).unwrap();
    assert_eq!(String::from_utf8(bytes).unwrap().matches("<svg").count(), 4);
}

// ---- the search

#[test]
fn the_adaptive_chain_is_a_proper_chain_ordered_by_measured_efficiency() {
    let p = flat();
    let mut f = Fit::new(&p, opts(Format::Gif, 20_000)).unwrap();
    let ch = f.adaptive_chain();
    let ax = axes(Format::Gif);
    assert_eq!(ch.len(), 1 + ax.iter().filter(|a| a.in_chain).map(|a| a.values.len() - 1).sum::<usize>());
    assert!(ch[0].iter().all(|&x| x == 0));
    assert_eq!(*ch.last().unwrap(), ax.iter().map(|a| if a.in_chain { a.values.len() - 1 } else { 0 }).collect::<Vec<_>>());
    for w in ch.windows(2) {
        let d: Vec<i64> = w[1].iter().zip(&w[0]).map(|(a, b)| *a as i64 - *b as i64).collect();
        assert!(d.iter().filter(|&&x| x != 0).count() == 1 && d.iter().sum::<i64>() == 1);
    }
    // deterministic
    assert_eq!(ch, f.adaptive_chain());
    // and a descent-search report says which chain it bisected
    let r = fit(&p, opts(Format::Gif, 6_000)).unwrap();
    assert!(["static", "adaptive"].contains(&r.chain_kind) && r.to_json()["search"]["chain_kind"].is_string());
}

#[test]
fn sizes_fall_along_the_ladders() {
    let p = small_demo();
    for fmt in [Format::Lottie, Format::Html, Format::Project] {
        let s = chain_sizes(&p, fmt);
        assert!(s.iter().all(|&b| b < usize::MAX), "{fmt:?}: every rung must build");
        let factor = if fmt == Format::Project { 1 } else { 2 };
        assert!(s.last().unwrap() * factor < s[0], "{fmt:?}: the floor is smaller than the top ({} vs {})", s.last().unwrap(), s[0]);
        // near-monotone: any increase is below 2 % of the previous rung (the search checks and falls back)
        for w in s.windows(2) {
            assert!(w[1] as f64 <= w[0] as f64 * 1.02, "{fmt:?}: size grew along the ladder: {s:?}");
        }
        let ups = s.windows(2).filter(|w| w[1] > w[0]).count();
        assert!(ups * 4 <= s.len(), "{fmt:?}: {ups} increases in {} rungs: {s:?}", s.len());
    }
    let g = flat();
    for fmt in [Format::Gif, Format::Apng] {
        let s = chain_sizes(&g, fmt);
        assert!(s.iter().all(|&b| b < usize::MAX), "{fmt:?}");
        assert!(s.last().unwrap() < &(s[0] / 2), "{fmt:?}: {s:?}");
        // monotone up to a small slack (palette changes can move LZW / deflate output slightly)
        for w in s.windows(2) {
            assert!(w[1] as f64 <= w[0] as f64 * 1.03 + 150.0, "{fmt:?}: {s:?}");
        }
    }
}

#[test]
fn bisection_finds_the_same_rung_as_a_brute_force_scan() {
    let (mut bisected, mut scanned) = (0, 0);
    for (p, fmt) in [(small_demo(), Format::Lottie), (small_demo(), Format::Html), (small_demo(), Format::Project), (flat(), Format::Gif)] {
        let sizes = chain_sizes(&p, fmt);
        let mut sorted = sizes.clone();
        sorted.sort();
        for (bi, budget) in [sorted[sorted.len() / 4], sorted[sorted.len() * 3 / 4]].into_iter().enumerate() {
            let brute = sizes.iter().position(|&s| s <= budget).unwrap();
            let mut o = opts(fmt, budget);
            o.search = Search::Ladder;
            let plain = fit(&p, o.clone()).unwrap();
            assert_eq!(plain.chosen.pos, chain(fmt)[brute], "{fmt:?} budget {budget} (no audit)");
            o.audit = bi == 0;
            let r = fit(&p, o).unwrap();
            assert!(r.met, "{fmt:?} budget {budget}");
            assert_eq!(r.chosen.pos, chain(fmt)[brute], "{fmt:?} budget {budget}");
            assert!(bi != 0 || r.audit_agrees == Some(true) || r.fallback_linear);
            assert!(r.chosen.budgeted <= budget);
            assert_eq!(r.bytes.len(), r.chosen.raw);
            if plain.monotone && !plain.fallback_linear {
                assert!(plain.probes.len() < sizes.len(), "bisection probes fewer rungs than the scan ({} of {})", plain.probes.len(), sizes.len());
                bisected += 1;
            } else {
                assert!(plain.fallback_linear, "a non-monotone chain must trigger the linear scan");
                scanned += 1;
            }
        }
    }
    assert!(bisected > 0, "at least one chain must be searched by bisection (scanned {scanned})");
}

#[test]
fn a_roomy_budget_gives_the_top_rung_and_descent_never_does_worse_than_the_ladder() {
    let p = small_demo();
    let big = fit(&p, opts(Format::Lottie, usize::MAX / 2)).unwrap();
    assert!(big.met && big.chosen.pos.iter().all(|&x| x == 0) && big.probes.len() == 1);
    let sizes = chain_sizes(&p, Format::Lottie);
    let budget = sizes[sizes.len() / 2];
    let mut lo = opts(Format::Lottie, budget);
    lo.search = Search::Ladder;
    let a = fit(&p, lo.clone()).unwrap();
    lo.search = Search::Descent;
    let b = fit(&p, lo.clone()).unwrap();
    assert!(b.chosen.budgeted <= budget && a.chosen.budgeted <= budget);
    assert!(b.chosen.ssim_mean >= a.chosen.ssim_mean - 1e-12, "descent {} vs ladder {}", b.chosen.ssim_mean, a.chosen.ssim_mean);
}

#[test]
fn grid_is_at_least_as_good_as_descent_and_pareto_front_is_consistent() {
    let p = small_demo();
    let sizes = chain_sizes(&p, Format::Project);
    let budget = sizes[sizes.len() / 2];
    let mut o = opts(Format::Project, budget);
    o.search = Search::Descent;
    let d = fit(&p, o.clone()).unwrap();
    o.search = Search::Grid;
    let g = fit(&p, o).unwrap();
    assert!(g.met && g.chosen.budgeted <= budget);
    assert!(g.chosen.ssim_mean >= d.chosen.ssim_mean - 1e-12);
    let front = pareto_front(&g.all);
    assert!(!front.is_empty());
    for w in front.windows(2) {
        assert!(g.all[w[1]].budgeted >= g.all[w[0]].budgeted && g.all[w[1]].ssim_mean > g.all[w[0]].ssim_mean);
    }
    // nothing outside the front dominates a front member
    for &i in &front {
        assert!(!g.all.iter().any(|c| c.valid && c.budgeted < g.all[i].budgeted && c.ssim_mean > g.all[i].ssim_mean + 1e-12));
    }
    let csv = g.pareto_csv();
    assert_eq!(csv.lines().count(), g.all.len() + 1);
    assert!(csv.lines().next().unwrap().starts_with("position,decimals") && csv.contains(",1,"), "front rows are flagged");
    assert_eq!(csv.lines().filter(|l| l.ends_with(",1")).count(), 1, "exactly one chosen row");
}

#[test]
fn an_impossible_budget_is_reported_with_the_best_achievable_size() {
    let p = small_demo();
    for fmt in [Format::Lottie, Format::Html] {
        let r = fit(&p, opts(fmt, 10)).unwrap();
        assert!(!r.met);
        assert!(r.reason.as_deref().unwrap().contains("floor"), "{:?}", r.reason);
        let floor = chain(fmt).last().unwrap().clone();
        assert_eq!(r.chosen.pos, floor, "the smallest export is the floor rung");
        assert_eq!(r.floors.len(), axes(fmt).iter().filter(|a| a.in_chain).count(), "every knob is at its floor");
        assert!(r.chosen.budgeted > 10 && r.bytes.len() == r.chosen.raw);
        let j = r.to_json();
        assert_eq!(j["met"], false);
        assert!(j["reason"].is_string());
    }
    assert!(fit(&p, FitOptions::new(Format::Lottie, 0)).is_err());
}

/// C12c: the search has a wall-clock budget (default 60 s). With the budget spent the descent does
/// not run, the bisection result is kept and the report says so; 0 means unlimited.
#[test]
fn search_has_a_time_budget() {
    assert_eq!(FitOptions::new(Format::Gif, 1000).max_seconds, 60.0);
    let p = small_demo();
    let sizes = chain_sizes(&p, Format::Lottie);
    let budget = sizes[sizes.len() * 3 / 4];
    let mut o = opts(Format::Lottie, budget);
    o.max_seconds = 1e-9;
    let limited = fit(&p, o).unwrap();
    assert!(limited.time_limited && limited.met && limited.chosen.budgeted <= budget);
    assert_eq!(limited.to_json()["search"]["time_limited"], true);
    let mut o = opts(Format::Lottie, budget);
    o.max_seconds = 0.0;
    let full = fit(&p, o).unwrap();
    assert!(!full.time_limited);
    assert!(full.evaluations >= limited.evaluations, "{} vs {}", full.evaluations, limited.evaluations);
    assert!(full.quality.ssim_mean >= limited.quality.ssim_mean - 1e-9, "the unbounded search is at least as good");
}

/// M3 (wave 4): `max_seconds` used to be checked only between whole candidates, so a GIF search overshot it by
/// minutes; and a budget that ran out part-way left rungs unbuilt which the linear scan then indexed directly
/// (panic). The clock is replaced by a count of built candidates, so every moment of expiry is covered
/// deterministically: never a panic, `time_limited` reported, best-so-far kept, and unbuilt rungs never "fit".
#[test]
fn an_expiring_budget_never_indexes_unbuilt_rungs() {
    let p = small_demo();
    let sizes = chain_sizes(&p, Format::Lottie);
    let budget = sizes[sizes.len() * 3 / 4];
    for search in [Search::Ladder, Search::Descent, Search::Grid] {
        for expire in 0..14usize {
            TEST_EXPIRE_AFTER_EVALS.with(|c| c.set(Some(expire)));
            let mut o = opts(Format::Lottie, budget);
            o.max_seconds = 1000.0; // only the test clock decides
            o.search = search;
            o.audit = expire % 2 == 0; // the audit forces the linear scan over the chain
            let r = fit(&p, o);
            TEST_EXPIRE_AFTER_EVALS.with(|c| c.set(None));
            match r {
                Ok(r) => {
                    if r.met {
                        assert!(r.chosen.budgeted <= budget);
                    }
                    if r.evaluations > expire {
                        assert!(r.time_limited, "{search:?}/{expire}: the budget ran out but is not reported");
                        assert!(r.reason.as_deref().unwrap_or("").contains("time budget") || r.met, "{:?}", r.reason);
                    }
                }
                Err(e) => assert!(e.contains("budget"), "{search:?}/{expire}: {e}"),
            }
        }
    }
}

#[test]
fn the_time_budget_is_enforced_inside_candidates() {
    let p = small_demo();
    // the real clock. A GIF search with a tiny budget stops within 1.5x the budget plus a fixed slack
    let budget_s = 0.2f64;
    let mut o = opts(Format::Gif, 1);
    o.max_seconds = budget_s;
    let t = crate::clock::Stamp::now();
    let r = fit(&p, o);
    let took = t.elapsed().as_secs_f64();
    assert!(took < budget_s * 1.5 + 3.0, "took {took} s for a {budget_s} s budget");
    if let (Ok(r), true) = (r, took > budget_s) {
        assert!(r.time_limited && r.reason.as_deref().unwrap_or("").contains("time budget"), "{:?}", r.reason);
    }
}

#[test]
fn min_ssim_is_a_hard_constraint() {
    let p = small_demo();
    let sizes = chain_sizes(&p, Format::Lottie);
    let budget = sizes[sizes.len() * 3 / 4];
    let free = fit(&p, opts(Format::Lottie, budget)).unwrap();
    assert!(free.met);
    // ask for slightly more quality than the best fit can have
    let mut o = opts(Format::Lottie, budget);
    o.min_ssim = Some(1.0);
    o.search = Search::Ladder;
    let r = fit(&p, o).unwrap();
    assert!(!r.met && r.reason.as_deref().unwrap().contains("min-ssim"), "{:?}", r.reason);
    // an achievable floor is honoured
    let mut o = opts(Format::Lottie, budget);
    o.min_ssim = Some((free.quality.ssim_min - 0.01).max(0.0));
    let r = fit(&p, o.clone()).unwrap();
    assert!(r.met && r.quality.ssim_min >= o.min_ssim.unwrap() - 1e-9);
    let mut bad = opts(Format::Lottie, budget);
    bad.min_ssim = Some(1.5);
    assert!(fit(&p, bad).is_err());
}

#[test]
fn gzip_measure_budgets_the_compressed_size() {
    let p = small_demo();
    let raw_top = chain_sizes(&p, Format::Html)[0];
    let mut o = opts(Format::Html, raw_top / 6);
    o.measure = Measure::Gzip;
    o.search = Search::Ladder;
    let r = fit(&p, o).unwrap();
    assert!(r.met, "{:?}", r.reason);
    assert!(r.chosen.budgeted <= raw_top / 6 && r.chosen.budgeted >= r.chosen.gzip + r.chosen.gzip / 100 + 32);
    assert!(r.chosen.raw > r.chosen.gzip, "the raw file is bigger than the budget while its gzip fits");
    assert_eq!(gzip_len(&r.bytes), r.chosen.gzip);
    assert_eq!(r.to_json()["measure"], "gzip");
}

#[test]
fn the_search_is_deterministic_and_leaves_the_project_alone() {
    let p = small_demo();
    let before = serde_json::to_string(&p).unwrap();
    let sizes = chain_sizes(&p, Format::Lottie);
    let o = opts(Format::Lottie, sizes[sizes.len() / 2]);
    let a = fit(&p, o.clone()).unwrap();
    let b = fit(&p, o).unwrap();
    assert_eq!(a.bytes, b.bytes);
    assert_eq!(a.to_json(), b.to_json());
    assert_eq!(a.pareto_csv(), b.pareto_csv());
    assert_eq!(serde_json::to_string(&p).unwrap(), before, "source project untouched");
}

#[test]
fn report_schema() {
    let p = flat();
    let mut o = opts(Format::Gif, 8000);
    o.baseline = true;
    o.search = Search::Ladder;
    let r = fit(&p, o).unwrap();
    let j = r.to_json();
    assert_eq!(j["schema"], SCHEMA);
    for k in ["format", "budget_bytes", "measure", "met", "reason", "search", "chosen", "quality", "baseline", "dead_layers", "axes"] {
        assert!(j.get(k).is_some(), "missing {k}");
    }
    for k in ["strategy", "evaluations", "chain_length", "probes", "monotone", "fallback_linear", "audit_agrees"] {
        assert!(j["search"].get(k).is_some(), "search.{k}");
    }
    for k in ["position", "settings", "raw_bytes", "gzip_bytes", "budgeted_bytes", "floors"] {
        assert!(j["chosen"].get(k).is_some(), "chosen.{k}");
    }
    for k in ["metric", "sample_frames", "ssim_mean", "ssim_min", "worst_frame", "max_ciede2000", "per_frame", "min_ssim_required"] {
        assert!(j["quality"].get(k).is_some(), "quality.{k}");
    }
    assert!(j["quality"]["per_frame"].as_array().unwrap().len() == r.sample_frames.len());
    assert!(j["baseline"]["raw_bytes"].as_u64().unwrap() > 0);
    assert_eq!(j["format"], "gif");
    assert!(j["quality"]["ssim_min"].as_f64().unwrap() <= j["quality"]["ssim_mean"].as_f64().unwrap() + 1e-12);
}

#[test]
fn gif_and_apng_quality_falls_with_the_palette() {
    let p = small_demo();
    let mut f = Fit::new(&p, opts(Format::Gif, usize::MAX)).unwrap();
    let ch = chain(Format::Gif);
    let top = f.eval(&ch[0]);
    let low = f.eval(ch.last().unwrap());
    assert!(top.valid && low.valid);
    assert!(top.ssim_mean > low.ssim_mean && top.ssim_mean > 0.9, "top {} low {}", top.ssim_mean, low.ssim_mean);
    assert!(low.budgeted < top.budgeted);
}

#[test]
fn webm_ladder_when_ffmpeg_exists() {
    if !ffmpeg_available() {
        return;
    }
    let p = flat();
    let mut f = Fit::new(&p, opts(Format::Webm, usize::MAX)).unwrap();
    let ch = chain(Format::Webm);
    let a = f.eval(&ch[0]);
    let b = f.eval(ch.last().unwrap());
    assert!(a.valid && b.valid, "{:?} {:?}", a.error, b.error);
    assert!(b.budgeted < a.budgeted && a.ssim_mean >= b.ssim_mean);
}

#[test]
fn size_report_names_the_contributors() {
    let p = small_demo();
    let r = size_report(&p, 0.05).unwrap();
    assert!(r["project"]["json_bytes"].as_u64().unwrap() > 1000);
    assert!(!r["project"]["layers"].as_array().unwrap().is_empty() && !r["project"]["channels"].as_array().unwrap().is_empty());
    assert!(r["lottie"]["raw_bytes"].as_u64().unwrap() > 0 && r["html"]["frames"] == 8);
    assert!(!r["html"]["layers"].as_array().unwrap().is_empty());
    let lj = r["project"]["layers"].as_array().unwrap();
    for w in lj.windows(2) {
        assert!(w[0]["json_bytes"].as_u64() >= w[1]["json_bytes"].as_u64(), "sorted by size");
    }
    let t = size_report_text(&r, 3);
    assert!(t.contains("project file") && t.contains("baked Lottie") && t.contains("HTML page"));
}

#[test]
fn invalid_options_are_errors_not_panics() {
    let p = flat();
    let mut o = FitOptions::new(Format::Gif, 1000);
    o.min_ssim = Some(-0.1);
    assert!(Fit::new(&p, o).is_err());
    assert!(Measure::parse("zip").is_err() && Search::parse("random").is_err());
    let mut bad = flat();
    bad.channels.push(Channel::keys("nobody", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear)])));
    assert!(Fit::new(&bad, FitOptions::new(Format::Lottie, 1000)).is_err());
    assert_eq!(sample_list(1, 8), vec![0]);
    assert_eq!(sample_list(10, 3), vec![0, 5, 9]);
    assert_eq!(sample_list(48, 1), vec![0]);
}
