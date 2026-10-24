// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `gltf`.
#![allow(unused_imports)]

use super::*;
use crate::demo;

fn parse(glb: &[u8]) -> (Value, Vec<u8>) {
    assert_eq!(&glb[..4], b"glTF");
    assert_eq!(u32::from_le_bytes(glb[4..8].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize, glb.len());
    let jl = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    assert_eq!(&glb[16..20], b"JSON");
    let json: Value = serde_json::from_slice(&glb[20..20 + jl]).unwrap();
    let bo = 20 + jl;
    let bl = u32::from_le_bytes(glb[bo..bo + 4].try_into().unwrap()) as usize;
    assert_eq!(&glb[bo + 4..bo + 8], b"BIN\0");
    assert_eq!(bo + 8 + bl, glb.len());
    (json, glb[bo + 8..].to_vec())
}

#[test]
fn triangulates_concave_and_either_winding() {
    let l_shape = [Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0), Vec2::new(4.0, 1.0), Vec2::new(1.0, 1.0), Vec2::new(1.0, 4.0), Vec2::new(0.0, 4.0)];
    for pts in [l_shape.to_vec(), l_shape.iter().rev().copied().collect()] {
        let t = triangulate(&pts);
        assert_eq!(t.len(), 4);
        let area: f32 = t.iter().map(|[a, b, c]| ((pts[*b].x - pts[*a].x) * (pts[*c].y - pts[*a].y) - (pts[*b].y - pts[*a].y) * (pts[*c].x - pts[*a].x)).abs() * 0.5).sum();
        assert!((area - 7.0).abs() < 1e-4, "area {area}");
    }
    assert!(triangulate(&[Vec2::ZERO, Vec2::new(1.0, 0.0)]).is_empty());
}

#[test]
fn triangulation_of_huge_polygons_is_fast_and_exact() {
    let t0 = crate::clock::Stamp::now();
    // convex: 60000 points (the old rescanning version needed about 40 s)
    let n = 60_000;
    let circle: Vec<Vec2> = (0..n).map(|i| { let a = i as f32 * std::f32::consts::TAU / n as f32; Vec2::new(300.0 * a.cos(), 300.0 * a.sin()) }).collect();
    let t = triangulate(&circle);
    assert_eq!(t.len(), n - 2);
    // a comb: many reflex vertices
    let teeth = 3000;
    let mut comb = vec![Vec2::new(0.0, 0.0)];
    for i in 0..teeth {
        let x = i as f32 * 2.0;
        comb.extend([Vec2::new(x, 10.0), Vec2::new(x + 1.0, 10.0), Vec2::new(x + 1.0, 1.0), Vec2::new(x + 2.0, 1.0)]);
    }
    comb.push(Vec2::new(teeth as f32 * 2.0, 0.0));
    comb.reverse();
    let tc = triangulate(&comb);
    assert_eq!(tc.len(), comb.len() - 2);
    let area: f32 = tc.iter().map(|[a, b, c]| ((comb[*b].x - comb[*a].x) * (comb[*c].y - comb[*a].y) - (comb[*b].y - comb[*a].y) * (comb[*c].x - comb[*a].x)).abs() * 0.5).sum();
    let want = (0..teeth).map(|_| 1.0 * 9.0 + 1.0 * 1.0).sum::<f32>() + 1.0;
    assert!(area > 0.0 && (area - want).abs() / want < 0.2, "{area} vs {want}");
    assert!(t0.elapsed().as_secs() < 20, "took {:?}", t0.elapsed());
    // beyond the cap: refused (empty), not slow
    let huge: Vec<Vec2> = (0..MAX_TRIANGULATE_POINTS + 1).map(|i| Vec2::new(i as f32, (i * i % 7) as f32)).collect();
    assert!(triangulate(&huge).is_empty());
}

#[test]
fn demo_exports_consistent_glb() {
    let e = Engine::new(demo::project()).unwrap();
    let glb = export_glb(&e).unwrap();
    let (j, bin) = parse(&glb);
    assert_eq!(j["asset"]["version"], "2.0");
    let nbones = e.project.bones.len();
    assert!(j["nodes"].as_array().unwrap().len() >= nbones);
    assert_eq!(j["animations"][0]["channels"].as_array().unwrap().len(), nbones * 3);
    assert_eq!(j["buffers"][0]["byteLength"].as_u64().unwrap() as usize, bin.len());
    for v in j["bufferViews"].as_array().unwrap() {
        assert!(v["byteOffset"].as_u64().unwrap() + v["byteLength"].as_u64().unwrap() <= bin.len() as u64);
    }
    for a in j["accessors"].as_array().unwrap() {
        let comps = match a["type"].as_str().unwrap() { "SCALAR" => 1, "VEC2" => 2, "VEC3" => 3, "VEC4" => 4, "MAT4" => 16, t => panic!("{t}") };
        let bytes = match a["componentType"].as_u64().unwrap() { 5121 => 1, 5126 | 5125 => 4, c => panic!("{c}") };
        let size = a["count"].as_u64().unwrap() * comps * bytes;
        assert!(size <= j["bufferViews"][a["bufferView"].as_u64().unwrap() as usize]["byteLength"].as_u64().unwrap());
    }
    assert!(j["meshes"].as_array().is_some_and(|m| !m.is_empty()), "demo has filled shapes");
    // the demo's skinned limbs come out as skinned meshes
    let skin = &j["skins"][0];
    assert_eq!(skin["joints"].as_array().unwrap().len(), nbones);
    let skinned: Vec<&Value> = j["nodes"].as_array().unwrap().iter().filter(|n| n.get("skin").is_some()).collect();
    assert!(!skinned.is_empty());
    let mesh = &j["meshes"][skinned[0]["mesh"].as_u64().unwrap() as usize];
    let at = &mesh["primitives"][0]["attributes"];
    assert!(at.get("JOINTS_0").is_some() && at.get("WEIGHTS_0").is_some());
    // weights of every vertex sum to 1 and joints are valid bone indices
    let wacc = &j["accessors"][at["WEIGHTS_0"].as_u64().unwrap() as usize];
    let wv = &j["bufferViews"][wacc["bufferView"].as_u64().unwrap() as usize];
    let off = wv["byteOffset"].as_u64().unwrap() as usize;
    for v in 0..wacc["count"].as_u64().unwrap() as usize {
        let s: f32 = (0..4).map(|k| f32::from_le_bytes(bin[off + (v * 4 + k) * 4..off + (v * 4 + k) * 4 + 4].try_into().unwrap())).sum();
        assert!((s - 1.0).abs() < 1e-4, "vertex {v} weights sum {s}");
    }
    let jacc = &j["accessors"][at["JOINTS_0"].as_u64().unwrap() as usize];
    let jo = j["bufferViews"][jacc["bufferView"].as_u64().unwrap() as usize]["byteOffset"].as_u64().unwrap() as usize;
    assert!(bin[jo..jo + jacc["count"].as_u64().unwrap() as usize * 4].iter().all(|&b| (b as usize) < nbones));
}

#[test]
fn rotation_sign_and_errors() {
    let ([_, ty, _], r, _) = {
        let m = Affine2 { a: 0.0, b: 1.0, c: -1.0, d: 0.0, tx: 5.0, ty: 7.0 }; // +90 deg in y-down space
        trs(m, None)
    };
    assert_eq!(ty, -7.0);
    assert!(r[2] < 0.0, "clockwise on screen = negative about +z in y-up");
    let mut p = demo::project();
    p.bones.clear();
    p.ik.clear();
    p.channels.clear();
    p.springs.clear();
    p.constraints.clear();
    p.layers.iter_mut().for_each(|l| l.bone = None);
    if let Ok(e) = Engine::new(p) {
        assert!(export_glb(&e).is_err());
    }
}

// ---- helpers for the feature tests ----------------------------------------------------

use crate::anim::{Ease, Track};
use crate::path::{Anchor, BoneWeight};
use crate::project::{Bone, Channel, MaskRef, MeshSpec, Project, Prop};
use crate::style::{Paint, StrokeStyle};

fn engine_with(layers: Vec<Layer>, frames: u32, channels: Vec<Channel>) -> Engine {
    let mut p = Project::new("t", 200.0, 200.0, 10.0, frames);
    p.bones.push(Bone::new("root", None, 10.0, Vec2::ZERO, 0.0));
    p.layers = layers;
    p.channels = channels;
    Engine::new(p).unwrap()
}

fn acc_bytes<'a>(j: &Value, bin: &'a [u8], i: usize) -> (&'a [u8], usize) {
    let a = &j["accessors"][i];
    let v = &j["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
    let off = v["byteOffset"].as_u64().unwrap() as usize + a["byteOffset"].as_u64().unwrap_or(0) as usize;
    let count = a["count"].as_u64().unwrap() as usize;
    let comps = match a["type"].as_str().unwrap() { "SCALAR" => 1, "VEC2" => 2, "VEC3" => 3, "VEC4" => 4, _ => 16 };
    let bytes = if a["componentType"] == 5121 { 1 } else { 4 };
    (&bin[off..off + count * comps * bytes], comps)
}

fn acc_f(j: &Value, bin: &[u8], i: usize) -> Vec<f32> {
    acc_bytes(j, bin, i).0.chunks_exact(4).map(|b| f32::from_le_bytes(b.try_into().unwrap())).collect()
}

fn acc_u(j: &Value, bin: &[u8], i: usize) -> Vec<u32> {
    acc_bytes(j, bin, i).0.chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect()
}

fn mesh_named<'a>(j: &'a Value, suffix: &str) -> &'a Value {
    j["meshes"].as_array().unwrap().iter().find(|m| m["name"].as_str().unwrap().ends_with(suffix)).unwrap_or_else(|| panic!("no mesh ending '{suffix}'"))
}

fn mesh_xy(j: &Value, bin: &[u8], m: &Value) -> (Vec<[f32; 2]>, Vec<u32>) {
    let at = &m["primitives"][0]["attributes"];
    let pos = acc_f(j, bin, at["POSITION"].as_u64().unwrap() as usize);
    let idx = acc_u(j, bin, m["primitives"][0]["indices"].as_u64().unwrap() as usize);
    (pos.chunks_exact(3).map(|p| [p[0], p[1]]).collect(), idx)
}

fn area(j: &Value, bin: &[u8], m: &Value) -> f32 {
    let (p, idx) = mesh_xy(j, bin, m);
    idx.chunks_exact(3)
        .map(|t| {
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() * 0.5
        })
        .sum()
}

fn line(name: &str, pts: &[(f32, f32)], st: StrokeStyle) -> Layer {
    let v: Vec<Vec2> = pts.iter().map(|&(x, y)| Vec2::new(x, y)).collect();
    let mut path = VPath::polygon(name, &v);
    path.closed = false;
    path.stroke = Some(st);
    Layer::vector(name, vec![path])
}

fn export(layers: Vec<Layer>) -> (Value, Vec<u8>, Vec<String>) {
    let e = engine_with(layers, 2, vec![]);
    let (glb, notes) = export_glb_report(&e).unwrap();
    let (j, bin) = parse(&glb);
    (j, bin, notes)
}

fn style(w: f32, cap: Cap, join: Join) -> StrokeStyle {
    let mut s = StrokeStyle::solid("#ff0000", w);
    s.cap = cap;
    s.join = join;
    s
}

// ---- strokes ---------------------------------------------------------------------------

#[test]
fn stroke_caps_have_the_right_area() {
    for (cap, expect, tol) in [(Cap::Butt, 1000.0, 0.5), (Cap::Square, 1100.0, 0.5), (Cap::Round, 1000.0 + std::f32::consts::PI * 25.0, 1.5)] {
        let (j, bin, _) = export(vec![line("l", &[(10.0, 50.0), (110.0, 50.0)], style(10.0, cap, Join::Round))]);
        let a = area(&j, &bin, mesh_named(&j, ":stroke"));
        assert!((a - expect).abs() < tol, "{cap:?}: area {a}, expected {expect}");
    }
}

#[test]
fn stroke_joins_miter_bevel_round() {
    // an L: two 50 x 10 ribbons (butt caps) plus the join piece; the inner overlap is counted twice
    let pts = [(10.0, 10.0), (60.0, 10.0), (60.0, 60.0)];
    let want = [(Join::Miter, 1025.0), (Join::Bevel, 1012.5), (Join::Round, 1000.0 + std::f32::consts::PI * 25.0 / 4.0)];
    for (join, expect) in want {
        let (j, bin, _) = export(vec![line("l", &pts, style(10.0, Cap::Butt, join))]);
        let a = area(&j, &bin, mesh_named(&j, ":stroke"));
        assert!((a - expect).abs() < 1.0, "{join:?}: area {a}, expected {expect}");
    }
    // a very sharp turn exceeds the miter limit and falls back to a bevel (no long spike)
    let (j, bin, _) = export(vec![line("l", &[(10.0, 50.0), (110.0, 50.0), (10.0, 60.0)], style(4.0, Cap::Butt, Join::Miter))]);
    let (p, _) = mesh_xy(&j, &bin, mesh_named(&j, ":stroke"));
    assert!(p.iter().all(|v| v[0] < 116.0), "no miter spike: {:?}", p.iter().map(|v| v[0]).fold(0.0, f32::max));
}

#[test]
fn stroke_dashes_and_width_profile() {
    let mut st = style(4.0, Cap::Butt, Join::Round);
    st.dash = vec![10.0, 10.0];
    let (j, bin, _) = export(vec![line("l", &[(10.0, 50.0), (110.0, 50.0)], st.clone())]);
    let a = area(&j, &bin, mesh_named(&j, ":stroke"));
    assert!((a - 5.0 * 10.0 * 4.0).abs() < 0.5, "five dashes: {a}");
    st.dash_offset = 5.0; // starts half way through a dash: 5 + 4*10 + 5 = 50 of 100
    let (j, bin, _) = export(vec![line("l", &[(10.0, 50.0), (110.0, 50.0)], st)]);
    assert!((area(&j, &bin, mesh_named(&j, ":stroke")) - 200.0).abs() < 0.5);
    // taper to nothing: triangle-like area = length * width / 2
    let mut st = style(10.0, Cap::Butt, Join::Round);
    st.width_profile = vec![[0.0, 1.0], [1.0, 0.0]];
    let (j, bin, _) = export(vec![line("l", &[(10.0, 50.0), (60.0, 50.0), (110.0, 50.0)], st)]);
    assert!((area(&j, &bin, mesh_named(&j, ":stroke")) - 500.0).abs() < 2.0);
}

#[test]
fn stroker_rejects_bad_input() {
    let st = StrokeStyle::solid("#000", 4.0);
    assert!(build_stroke(&[Vec2::ZERO], false, &st).is_none());
    let mut zero = st.clone();
    zero.width = 0.0;
    assert!(build_stroke(&[Vec2::ZERO, Vec2::new(5.0, 0.0)], false, &zero).is_none());
    zero.width = f32::NAN;
    assert!(build_stroke(&[Vec2::ZERO, Vec2::new(5.0, 0.0)], false, &zero).is_none());
    // closed ring: every triangle has positive area, no NaN
    let ring = VPath::circle("c", Vec2::new(50.0, 50.0), 30.0).polyline(0.25).pts;
    let s = build_stroke(&ring, true, &st).unwrap();
    assert!(s.pos.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
    let a: f32 = s.tris.iter().map(|t| cr(s.pos[t[1] as usize] - s.pos[t[0] as usize], s.pos[t[2] as usize] - s.pos[t[0] as usize]) * 0.5).sum();
    assert!((a - std::f32::consts::TAU * 30.0 * 4.0).abs() < 4.0, "ring area {a}");
}

#[test]
fn translucent_and_inherited_opacity_become_blend() {
    let mut g = Layer::group("g", vec![Layer::vector("a", vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#00ff00")])]);
    g.opacity = 0.5;
    let (j, _, _) = export(vec![g, line("l", &[(10.0, 50.0), (110.0, 50.0)], StrokeStyle::solid("#ff000080", 4.0))]);
    let mats = j["materials"].as_array().unwrap();
    assert_eq!(mats.len(), 2);
    assert!(mats.iter().all(|m| m["alphaMode"] == "BLEND"));
    let a0 = mats[0]["pbrMetallicRoughness"]["baseColorFactor"][3].as_f64().unwrap();
    assert!((a0 - 0.5).abs() < 1e-4, "group opacity multiplied in: {a0}");
}

#[test]
fn dropped_features_are_reported() {
    let mut a = Layer::vector("a", vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#00ff00")]);
    a.blend = crate::style::Blend::Multiply;
    let ch = vec![Channel::keys("a", Prop::LayerOpacity, Track::with_keys(&[(0.0, 1.0, Ease::Linear), (1.0, 0.0, Ease::Linear)])),
        Channel::keys("a::r::0", Prop::PointDx, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (1.0, 3.0, Ease::Linear)]))];
    let e = engine_with(vec![a], 2, ch);
    let (_, notes) = export_glb_report(&e).unwrap();
    let all = notes.join("\n");
    assert!(all.contains("blend mode") && all.contains("animated opacity of 'a'") && all.contains("animated path point"), "{all}");
    let (_, quiet) = export_glb_report(&engine_with(vec![Layer::vector("b", vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#00ff00")])], 2, vec![])).unwrap();
    assert!(quiet.is_empty(), "{quiet:?}");
}

// ---- gradients -------------------------------------------------------------------------

fn decode_png(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
    dec.set_transformations(png::Transformations::IDENTITY);
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut buf).unwrap();
    (info.width, info.height, buf)
}

fn texture_png(j: &Value, bin: &[u8], tex: usize) -> (u32, u32, Vec<u8>) {
    let img = &j["images"][j["textures"][tex]["source"].as_u64().unwrap() as usize];
    let v = &j["bufferViews"][img["bufferView"].as_u64().unwrap() as usize];
    let off = v["byteOffset"].as_u64().unwrap() as usize;
    decode_png(&bin[off..off + v["byteLength"].as_u64().unwrap() as usize])
}

fn sample_tex((w, h, px): &(u32, u32, Vec<u8>), uv: [f32; 2]) -> [f32; 4] {
    let (w, h) = (*w as usize, *h as usize);
    let x = (uv[0] * w as f32 - 0.5).clamp(0.0, (w - 1) as f32);
    let y = (uv[1] * h as f32 - 0.5).clamp(0.0, (h - 1) as f32);
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let at = |xx: usize, yy: usize, c: usize| px[(yy * w + xx) * 4 + c] as f32 / 255.0;
    let mut o = [0.0; 4];
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for c in 0..4 {
        o[c] = (at(x0, y0, c) * (1.0 - fx) + at(x1, y0, c) * fx) * (1.0 - fy) + (at(x0, y1, c) * (1.0 - fx) + at(x1, y1, c) * fx) * fy;
    }
    o
}

fn paint_color(p: &Paint, q: Vec2) -> [f32; 4] {
    let c = match p {
        Paint::Linear { from, to, stops } => {
            let d = *to - *from;
            color_at(stops, ((q - *from).dot(d) / d.dot(d)).clamp(0.0, 1.0))
        }
        Paint::Radial { center, radius, stops } => color_at(stops, (q.distance(*center) / radius).clamp(0.0, 1.0)),
        Paint::Solid(c) => *c,
    };
    [c.r, c.g, c.b, c.a]
}

#[test]
fn linear_and_radial_gradients_match_the_paint_on_every_vertex() {
    let lin = Paint::linear(Vec2::new(20.0, 0.0), Vec2::new(120.0, 0.0), &[(0.0, "#ff0000"), (0.4, "#00ff00"), (1.0, "#0000ff80")]);
    let rad = Paint::radial(Vec2::new(60.0, 100.0), 40.0, &[(0.0, "#ffffff"), (1.0, "#000000")]);
    let mut a = Layer::vector("a", vec![VPath::rect("r", 10.0, 10.0, 120.0, 60.0).with_paint(lin.clone())]);
    // a transformed layer: the gradient lives in layer space, positions are placed through the transform
    a.transform.position = Vec2::new(5.0, 7.0);
    a.transform.scale = Vec2::new(1.5, 0.5);
    let b = Layer::vector("b", vec![VPath::circle("c", Vec2::new(60.0, 100.0), 50.0).with_paint(rad.clone())]);
    let (j, bin, _) = export(vec![a, b]);
    assert_eq!(j["textures"].as_array().unwrap().len(), 2);
    for (suffix, paint, tr) in [("a:r", &lin, Some((Vec2::new(5.0, 7.0), Vec2::new(1.5, 0.5)))), ("b:c", &rad, None)] {
        let m = mesh_named(&j, suffix);
        let at = &m["primitives"][0]["attributes"];
        let (pos, _) = mesh_xy(&j, &bin, m);
        let uv = acc_f(&j, &bin, at["TEXCOORD_0"].as_u64().unwrap() as usize);
        let mat = &j["materials"][m["primitives"][0]["material"].as_u64().unwrap() as usize];
        let tex = texture_png(&j, &bin, mat["pbrMetallicRoughness"]["baseColorTexture"]["index"].as_u64().unwrap() as usize);
        let mut worst = 0.0f32;
        for (v, p) in pos.iter().enumerate() {
            // back to layer space (stage y was negated; the layer transform was position + scale)
            let mut q = Vec2::new(p[0], -p[1]);
            if let Some((t, s)) = tr {
                q = Vec2::new((q.x - t.x) / s.x, (q.y - t.y) / s.y);
            }
            let got = sample_tex(&tex, [uv[v * 2], uv[v * 2 + 1]]);
            let want = paint_color(paint, q);
            for c in 0..4 {
                worst = worst.max((got[c] - want[c]).abs());
            }
        }
        assert!(worst < 0.012, "{suffix}: worst channel error {worst}");
    }
}

#[test]
fn degenerate_gradients_fall_back_to_a_flat_colour() {
    let p = Paint::linear(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0), &[(0.0, "#ff0000"), (1.0, "#0000ff")]);
    assert!(grad_map(&p).is_none());
    let (j, _, _) = export(vec![Layer::vector("a", vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_paint(p)])]);
    assert!(j.get("textures").is_none());
    // a zero-length gradient paints its last stop everywhere (as in SVG and the rasteriser)
    let f = &j["materials"][0]["pbrMetallicRoughness"]["baseColorFactor"];
    assert!(f[0].as_f64().unwrap().abs() < 1e-4 && (f[2].as_f64().unwrap() - 1.0).abs() < 1e-4, "{f}");
}

// ---- masks -----------------------------------------------------------------------------

fn masked(mask: Layer, invert: bool, shape: Layer) -> (Value, Vec<u8>, Vec<String>) {
    let mut shape = shape;
    shape.mask = Some(MaskRef { layer: mask.name.clone(), invert });
    export(vec![mask, shape])
}

#[test]
fn masks_are_clipped_exactly() {
    let mk = || {
        let mut hole = Layer::vector("hole", vec![VPath::rect("r", 20.0, 20.0, 40.0, 40.0).with_fill("#ffffff")]);
        hole.visible = false;
        hole
    };
    let shape = || Layer::vector("cut", vec![VPath::rect("r", 0.0, 0.0, 100.0, 100.0).with_fill("#ff0000")]);
    let (j, bin, notes) = masked(mk(), false, shape());
    assert!((area(&j, &bin, mesh_named(&j, "cut:r")) - 1600.0).abs() < 0.1);
    assert!(notes.iter().any(|n| n.contains("baked")));
    let (j, bin, _) = masked(mk(), true, shape());
    assert!((area(&j, &bin, mesh_named(&j, "cut:r")) - 8400.0).abs() < 0.1);
    // a half-covering mask on a stroke
    let s = line("cut", &[(0.0, 50.0), (100.0, 50.0)], style(10.0, Cap::Butt, Join::Round));
    let (j, bin, _) = masked(mk(), false, s);
    assert!((area(&j, &bin, mesh_named(&j, ":stroke")) - 400.0).abs() < 0.1, "stroke clipped to the 40 x 10 overlap");
    // a circle mask
    let mut c = Layer::vector("hole", vec![VPath::circle("c", Vec2::new(50.0, 50.0), 30.0).with_fill("#fff")]);
    c.visible = false;
    let (j, bin, _) = masked(c, false, shape());
    let a = area(&j, &bin, mesh_named(&j, "cut:r"));
    assert!((a - std::f32::consts::PI * 900.0).abs() < 25.0, "circle area {a}");
}

#[test]
fn masks_that_cannot_be_baked_are_reported_not_faked() {
    let mut m = Layer::vector("hole", vec![VPath::rect("r", 20.0, 20.0, 40.0, 40.0).with_fill("#ffffff").with_stroke("#000000", 2.0)]);
    m.visible = false;
    let (j, bin, notes) = masked(m, false, Layer::vector("cut", vec![VPath::rect("r", 0.0, 0.0, 100.0, 100.0).with_fill("#ff0000")]));
    assert!((area(&j, &bin, mesh_named(&j, "cut:r")) - 10000.0).abs() < 0.1, "exported unmasked");
    assert!(notes.iter().any(|n| n.contains("not baked")), "{notes:?}");
    // the piece limit is honoured
    let pos = [Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(0.0, 10.0)];
    let many: Vec<[Vec2; 3]> = (0..50).map(|i| { let o = i as f32 * 0.1; [Vec2::new(o, o), Vec2::new(o + 5.0, o), Vec2::new(o, o + 5.0)] }).collect();
    assert!(clip_mesh(&pos, &[[0, 1, 2]], &[(many, true)], 3).is_none());
}

// ---- images ----------------------------------------------------------------------------

fn tiny_png() -> Vec<u8> {
    let mut px = Vec::new();
    for i in 0..16u8 {
        px.extend_from_slice(&[i * 16, 255 - i * 16, 40, if i == 0 { 0 } else { 255 }]);
    }
    encode_rgba_png(4, 4, &px)
}

#[test]
fn image_layers_become_textured_quads() {
    let png = tiny_png();
    let img = Layer::image_from_png("pic", &png, Vec2::new(30.0, 40.0));
    let (j, bin, _) = export(vec![img]);
    let m = &j["meshes"][0];
    let at = &m["primitives"][0]["attributes"];
    let (pos, idx) = mesh_xy(&j, &bin, m);
    assert_eq!((pos.len(), idx.len()), (4, 6));
    assert!((area(&j, &bin, m) - 16.0).abs() < 1e-3, "4 x 4 px image");
    let uv = acc_f(&j, &bin, at["TEXCOORD_0"].as_u64().unwrap() as usize);
    assert_eq!(uv, vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0]);
    // the embedded bytes are the layer's PNG, untouched
    let v = &j["bufferViews"][j["images"][0]["bufferView"].as_u64().unwrap() as usize];
    let off = v["byteOffset"].as_u64().unwrap() as usize;
    assert_eq!(&bin[off..off + v["byteLength"].as_u64().unwrap() as usize], &png[..]);
    assert_eq!(j["images"][0]["mimeType"], "image/png");
    let mat = &j["materials"][0];
    assert_eq!(mat["alphaMode"], "BLEND", "the image has a transparent pixel");
    assert_eq!(mat["pbrMetallicRoughness"]["baseColorTexture"]["index"], 0);
}

#[test]
fn masked_image_quads_keep_affine_uvs_and_meshed_images_are_skinned() {
    let png = tiny_png();
    let mut hole = Layer::vector("hole", vec![VPath::rect("r", 32.0, 40.0, 1.0, 2.0).with_fill("#fff")]);
    hole.visible = false;
    let (j, bin, _) = masked(hole, false, Layer::image_from_png("pic", &png, Vec2::new(30.0, 40.0)));
    let m = mesh_named(&j, "pic");
    let (pos, _) = mesh_xy(&j, &bin, m);
    let uv = acc_f(&j, &bin, m["primitives"][0]["attributes"]["TEXCOORD_0"].as_u64().unwrap() as usize);
    assert!((area(&j, &bin, m) - 2.0).abs() < 1e-3);
    for (v, p) in pos.iter().enumerate() {
        assert!((uv[v * 2] - (p[0] - 30.0) / 4.0).abs() < 1e-5 && (uv[v * 2 + 1] - (-p[1] - 40.0) / 4.0).abs() < 1e-5);
    }
    // meshed image with complete weights -> JOINTS_0/WEIGHTS_0
    let mut l = Layer::image_from_png("pic", &png, Vec2::new(0.0, 0.0));
    if let LayerKind::Image { mesh, size, .. } = &mut l.kind {
        *size = Some(Vec2::new(40.0, 40.0));
        *mesh = Some(MeshSpec { cols: 2, rows: 2, weights: vec![vec![BoneWeight { bone: "root".into(), weight: 1.0 }]; 9], ..Default::default() });
    }
    let (j, _, _) = export(vec![l]);
    let at = &j["meshes"][0]["primitives"][0]["attributes"];
    assert!(at.get("JOINTS_0").is_some() && at.get("WEIGHTS_0").is_some() && at.get("TEXCOORD_0").is_some());
    assert_eq!(j["nodes"].as_array().unwrap().iter().filter(|n| n.get("skin").is_some()).count(), 1);
}

// ---- skinned strokes -------------------------------------------------------------------

#[test]
fn strokes_of_weighted_paths_are_skinned() {
    let w = |a: Anchor| Anchor { weights: vec![BoneWeight { bone: "root".into(), weight: 1.0 }], ..a };
    let mut path = VPath::new("p", false, vec![w(Anchor::corner(Vec2::new(0.0, 0.0))), w(Anchor::corner(Vec2::new(50.0, 10.0))), w(Anchor::corner(Vec2::new(90.0, 0.0)))]);
    path.stroke = Some(StrokeStyle::solid("#00f", 6.0));
    let (j, bin, _) = export(vec![Layer::vector("s", vec![path])]);
    let m = mesh_named(&j, ":stroke");
    let at = &m["primitives"][0]["attributes"];
    let n = j["accessors"][at["POSITION"].as_u64().unwrap() as usize]["count"].as_u64().unwrap();
    assert_eq!(j["accessors"][at["WEIGHTS_0"].as_u64().unwrap() as usize]["count"].as_u64().unwrap(), n);
    let wts = acc_f(&j, &bin, at["WEIGHTS_0"].as_u64().unwrap() as usize);
    assert!(wts.chunks_exact(4).all(|c| (c.iter().sum::<f32>() - 1.0).abs() < 1e-4));
}

// ---- morph targets ---------------------------------------------------------------------

fn morph_rect() -> VPath {
    VPath::rect("p", 0.0, 0.0, 10.0, 10.0).with_morph("wide", |a| {
        let mut b = a.clone();
        b.pos.x *= 3.0;
        b
    })
}

fn morph_keys() -> Vec<Channel> {
    vec![Channel::keys("s::p::wide", Prop::PathMorph, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 1.0, Ease::Linear)]))]
}

#[test]
fn path_morphs_become_targets_with_animated_weights() {
    let mut path = morph_rect();
    path.fill = Some(Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(30.0, 0.0), &[(0.0, "#f00"), (1.0, "#00f")]));
    let e = engine_with(vec![Layer::vector("s", vec![path])], 11, morph_keys());
    let (glb, notes) = export_glb_report(&e).unwrap();
    assert!(notes.is_empty(), "{notes:?}");
    let (j, bin) = parse(&glb);
    let m = &j["meshes"][0];
    let prim = &m["primitives"][0];
    assert_eq!(prim["targets"].as_array().unwrap().len(), 1);
    assert_eq!(m["extras"]["targetNames"][0], "wide");
    assert_eq!(m["weights"][0].as_f64().unwrap(), 0.0);
    // animation channel on the mesh node: linear 0..1 over 11 frames
    let ch = j["animations"][0]["channels"].as_array().unwrap().iter().find(|c| c["target"]["path"] == "weights").unwrap();
    let out = j["animations"][0]["samplers"][ch["sampler"].as_u64().unwrap() as usize]["output"].as_u64().unwrap() as usize;
    let w = acc_f(&j, &bin, out);
    assert_eq!(w.len(), 11);
    assert!(w.iter().enumerate().all(|(i, v)| (v - i as f32 / 10.0).abs() < 1e-5));
    assert!(j["nodes"][ch["target"]["node"].as_u64().unwrap() as usize].get("mesh").is_some());
    // base + w * delta spans x in [0, 10 + 20 w]
    let base = acc_f(&j, &bin, prim["attributes"]["POSITION"].as_u64().unwrap() as usize);
    let delta = acc_f(&j, &bin, prim["targets"][0]["POSITION"].as_u64().unwrap() as usize);
    for (wt, want) in [(0.0f32, 10.0f32), (0.5, 20.0), (1.0, 30.0)] {
        let mx = (0..base.len() / 3).map(|v| base[v * 3] + wt * delta[v * 3]).fold(f32::MIN, f32::max);
        assert!((mx - want).abs() < 1e-3, "w={wt}: {mx}");
    }
    // the gradient texture coordinate morphs with the vertex
    let uv0 = acc_f(&j, &bin, prim["attributes"]["TEXCOORD_0"].as_u64().unwrap() as usize);
    let duv = acc_f(&j, &bin, prim["targets"][0]["TEXCOORD_0"].as_u64().unwrap() as usize);
    let vmax = (0..base.len() / 3).max_by(|&a, &b| (base[a * 3] + delta[a * 3]).total_cmp(&(base[b * 3] + delta[b * 3]))).unwrap();
    let u1 = uv0[vmax * 2] + duv[vmax * 2];
    let LayerKind::Vector { paths } = &e.project.layers[0].kind else { panic!() };
    let map = grad_map(paths[0].fill.as_ref().unwrap()).unwrap();
    assert!((u1 - grad_uv(&map, Vec2::new(30.0, 0.0))[0]).abs() < 1e-4 || (u1 - grad_uv(&map, Vec2::new(30.0, 10.0))[0]).abs() < 1e-4);
}

#[test]
fn morphs_that_are_also_deformed_stay_static_with_a_reason() {
    let path = morph_rect().with_fill("#ff0000");
    let mut ch = morph_keys();
    ch.push(Channel::keys("s::p::0", Prop::PointDy, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 5.0, Ease::Linear)])));
    let e = engine_with(vec![Layer::vector("s", vec![path])], 11, ch);
    let (glb, notes) = export_glb_report(&e).unwrap();
    let (j, _) = parse(&glb);
    assert!(j["meshes"][0]["primitives"][0].get("targets").is_none());
    assert!(notes.iter().any(|n| n.contains("not exported as targets")), "{notes:?}");
    // morph never driven: static, with a reason
    let e = engine_with(vec![Layer::vector("s", vec![morph_rect().with_fill("#ff0000")])], 3, vec![]);
    let (_, notes) = export_glb_report(&e).unwrap();
    assert!(notes.iter().any(|n| n.contains("zero on every frame")), "{notes:?}");
}

#[test]
fn skin_has_a_common_root_and_zero_weights_use_joint_zero() {
    let e = Engine::new(demo::project()).unwrap();
    let (j, bin) = parse(&export_glb(&e).unwrap());
    let skel = j["skins"][0]["skeleton"].as_u64().unwrap() as usize;
    let mut seen = vec![skel];
    let mut k = 0;
    while k < seen.len() {
        if let Some(c) = j["nodes"][seen[k]]["children"].as_array() {
            seen.extend(c.iter().map(|x| x.as_u64().unwrap() as usize));
        }
        k += 1;
    }
    for joint in j["skins"][0]["joints"].as_array().unwrap() {
        assert!(seen.contains(&(joint.as_u64().unwrap() as usize)), "joint {joint} not under the skeleton root");
    }
    assert!(!j["skins"][0]["joints"].as_array().unwrap().iter().any(|x| x.as_u64().unwrap() as usize == skel), "skeleton is not a joint");
    for m in j["meshes"].as_array().unwrap() {
        let at = &m["primitives"][0]["attributes"];
        if at.get("JOINTS_0").is_none() {
            continue;
        }
        let w = acc_f(&j, &bin, at["WEIGHTS_0"].as_u64().unwrap() as usize);
        let (jb, _) = acc_bytes(&j, &bin, at["JOINTS_0"].as_u64().unwrap() as usize);
        assert!(w.iter().zip(jb).all(|(w, j)| *w > 0.0 || *j == 0));
    }
}
