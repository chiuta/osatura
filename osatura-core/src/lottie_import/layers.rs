// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `lottie_import` (see the module docs in `mod.rs`).
use super::*;

pub(super) fn blend_from(bm: i64) -> Blend {
    match bm {
        1 => Blend::Multiply,
        2 => Blend::Screen,
        3 => Blend::Overlay,
        4 => Blend::Darken,
        5 => Blend::Lighten,
        6 => Blend::ColorDodge,
        7 => Blend::ColorBurn,
        8 => Blend::HardLight,
        9 => Blend::SoftLight,
        10 => Blend::Difference,
        11 => Blend::Exclusion,
        _ => Blend::Normal,
    }
}

/// Easing for the segment starting at `k`: hold -> step; handles on the diagonal -> linear;
/// otherwise a CSS-style cubic bezier from the keyframe's `o` and `i` handles.
pub(super) fn ease_of(k: &Kf, comp: usize) -> Ease {
    if k.hold {
        return Ease::Step;
    }
    let Some(b) = &k.bez else { return Ease::Linear };
    let g = |a: &Vec<f32>| a[comp.min(a.len() - 1)];
    let (x1, y1, x2, y2) = (g(&b[0]).clamp(0.0, 1.0), g(&b[1]), g(&b[2]).clamp(0.0, 1.0), g(&b[3]));
    if !(x1.is_finite() && y1.is_finite() && x2.is_finite() && y2.is_finite()) || ((x1 - y1).abs() < 1e-3 && (x2 - y2).abs() < 1e-3) {
        return Ease::Linear;
    }
    Ease::CubicBezier { x1, y1, x2, y2 }
}

/// Position keys with curved motion paths, baked on every frame: each segment follows the cubic
/// bezier `P0, P0+to, P1+ti, P1` (Lottie's tangents are offsets), with time eased by the key's
/// handles. The path is sampled by its curve parameter, not by arc length, so speed along a very
/// uneven curve can differ slightly from a player that re-parameterises by length.
pub(super) fn bake_spatial(kfs: &[Kf]) -> (Track, Track) {
    type Keys = Vec<(f32, f32, Ease)>;
    let (mut kx, mut ky): (Keys, Keys) = (vec![], vec![]);
    let pt = |k: &Kf, c: usize| k.v.get(c).copied().unwrap_or(0.0);
    for w in kfs.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let ease = ease_of(a, 0);
        let (t0, t1) = (a.t, b.t);
        let n = ((t1 - t0).ceil() as i32).max(1);
        for s in 0..=n {
            if s == n && !std::ptr::eq(b, kfs.last().unwrap()) {
                break; // the next segment starts with this key
            }
            let t = if s == n { t1 } else { t0 + s as f32 * (t1 - t0) / n as f32 };
            let u = if matches!(ease, Ease::Step) { 0.0 } else { ease.apply(((t - t0) / (t1 - t0)).clamp(0.0, 1.0)) };
            let comp = |c: usize| {
                let (p0, p3) = (pt(a, c), pt(b, c));
                let (p1, p2) = (p0 + a.to.get(c).copied().unwrap_or(0.0), p3 + a.ti.get(c).copied().unwrap_or(0.0));
                let m = 1.0 - u;
                m * m * m * p0 + 3.0 * m * m * u * p1 + 3.0 * m * u * u * p2 + u * u * u * p3
            };
            kx.push((t, comp(0), Ease::Linear));
            ky.push((t, comp(1), Ease::Linear));
        }
    }
    if kfs.len() == 1 {
        kx.push((kfs[0].t, pt(&kfs[0], 0), Ease::Linear));
        ky.push((kfs[0].t, pt(&kfs[0], 1), Ease::Linear));
    }
    (Track::with_keys(&kx), Track::with_keys(&ky))
}

pub(super) fn track(kfs: &[Kf], comp: usize, scale: f32) -> Track {
    let keys: Vec<(f32, f32, Ease)> = kfs.iter().map(|k| (k.t, k.v.get(comp).copied().unwrap_or(0.0) * scale, ease_of(k, comp))).collect();
    Track::with_keys(&keys)
}

// ---------------------------------------------------------------------------------------------
// Layer parenting (baked), masks, luma mattes, image assets and effects
// ---------------------------------------------------------------------------------------------

/// A transform property: constant or keyframed (one [`Track`] per component, already in project frames).
pub(super) enum Val {
    Const(Vec<f32>),
    Keys { tracks: Vec<Track>, times: Vec<f32>, holds: Vec<f32> },
}

impl Val {
    pub(super) fn at(&self, t: f32, c: usize, default: f32) -> f32 {
        match self {
            Val::Const(v) => v.get(c).copied().unwrap_or(default),
            Val::Keys { tracks, .. } => tracks.get(c).and_then(|tr| tr.eval(t)).unwrap_or(default),
        }
    }
}

pub(super) fn val_of(prop: &Value, tm: (f32, f32), position: bool) -> Option<Val> {
    if let Some(v) = stat(prop) {
        return Some(Val::Const(v));
    }
    let kfs = keyframes(prop, tm)?;
    let n = kfs[0].v.len().max(1);
    let tracks = if position && n >= 2 && kfs.iter().any(|k| k.spatial) {
        let (x, y) = bake_spatial(&kfs);
        vec![x, y]
    } else {
        (0..n).map(|c| track(&kfs, c, 1.0)).collect()
    };
    let holds = kfs.windows(2).filter(|w| w[0].hold).map(|w| w[1].t).collect();
    Some(Val::Keys { tracks, times: kfs.iter().map(|k| k.t).collect(), holds })
}

/// The local transform of one Lottie layer (`ks`), evaluable at any project frame.
pub(super) struct LocalXf {
    pub(super) p: Option<Val>,
    pub(super) s: Option<Val>,
    pub(super) r: Option<Val>,
    pub(super) anchor: Vec2,
}

impl LocalXf {
    pub(super) fn of(l: &Value, tm: (f32, f32), warn: &mut Vec<String>) -> LocalXf {
        let ks = l.get("ks").cloned().unwrap_or(Value::Null);
        let anchor = ks
            .get("a")
            .and_then(|a| stat(a).or_else(|| keyframes(a, tm).map(|k| k[0].v.clone())))
            .map(|v| Vec2::new(v.first().copied().unwrap_or(0.0), v.get(1).copied().unwrap_or(0.0)))
            .unwrap_or(Vec2::ZERO);
        if ks.get("a").is_some_and(|a| a.get("a").and_then(Value::as_i64) == Some(1)) {
            let nm = l.get("nm").and_then(Value::as_str).unwrap_or("layer");
            warn.push(format!("layer '{nm}': animated anchor point uses its first value"));
        }
        LocalXf { p: ks.get("p").and_then(|v| val_of(v, tm, true)), s: ks.get("s").and_then(|v| val_of(v, tm, false)), r: ks.get("r").and_then(|v| val_of(v, tm, false)), anchor }
    }
    pub(super) fn animated(&self) -> bool {
        [&self.p, &self.s, &self.r].iter().any(|v| matches!(v, Some(Val::Keys { .. })))
    }
    pub(super) fn matrix(&self, t: f32) -> Affine2 {
        let g = |v: &Option<Val>, c: usize, d: f32| v.as_ref().map_or(d, |v| v.at(t, c, d));
        compose_transform(Vec2::new(g(&self.p, 0, 0.0), g(&self.p, 1, 0.0)), g(&self.r, 0, 0.0), Vec2::new(g(&self.s, 0, 100.0) / 100.0, g(&self.s, 1, 100.0) / 100.0), 0.0, self.anchor)
    }
    pub(super) fn key_times(&self, out: &mut Vec<f32>) {
        for v in [&self.p, &self.s, &self.r].into_iter().flatten() {
            if let Val::Keys { times, holds, .. } = v {
                out.extend(times);
                out.extend(holds.iter().map(|h| h - 0.001)); // just before a hold releases
            }
        }
    }
}

/// Split an affine map into our layer transform `T · R · ShearX · S` (anchor at the origin):
/// `(position, rotation degrees, scale, skew degrees)`. This is a QR decomposition, so every
/// non-singular map is represented exactly (a negative `scale.y` encodes a reflection).
pub(super) fn decompose(m: &Affine2) -> (Vec2, f32, Vec2, f32) {
    let sx = (m.a * m.a + m.b * m.b).sqrt();
    if sx < 1e-9 {
        return (Vec2::new(m.tx, m.ty), 0.0, Vec2::new(0.0, (m.c * m.c + m.d * m.d).sqrt()), 0.0);
    }
    let theta = m.b.atan2(m.a);
    let (s, c) = theta.sin_cos();
    let sy = m.c * -s + m.d * c;
    let along = m.c * c + m.d * s;
    let skew = if sy.abs() < 1e-9 { 0.0 } else { (along / sy).atan().to_degrees() };
    (Vec2::new(m.tx, m.ty), theta.to_degrees(), Vec2::new(sx, sy), skew)
}

/// Layer chain `[self, parent, grandparent, …]` of a layer with a `parent` link.
pub(super) fn parent_chain(l: &Value, by_ind: &std::collections::HashMap<i64, &Value>, tm: (f32, f32), warn: &mut Vec<String>) -> Option<Vec<LocalXf>> {
    let nm = l.get("nm").and_then(Value::as_str).unwrap_or("layer");
    let mut cur = l;
    let mut seen: Vec<i64> = l.get("ind").and_then(Value::as_i64).into_iter().collect();
    let mut chain = vec![LocalXf::of(l, tm, warn)];
    while let Some(pi) = cur.get("parent").and_then(Value::as_i64) {
        let Some(p) = by_ind.get(&pi) else {
            warn.push(format!("layer '{nm}': parent layer {pi} not found; parent link ignored"));
            return None;
        };
        if seen.contains(&pi) || chain.len() > MAX_DEPTH {
            warn.push(format!("layer '{nm}': parent links form a cycle or are nested too deeply; parent link ignored"));
            return None;
        }
        seen.push(pi);
        chain.push(LocalXf::of(p, tm, warn));
        cur = p;
    }
    if chain.len() > 1 { Some(chain) } else { None }
}

/// Baked world transform of a layer under its parents: `(transform, per-frame channels)`.
/// Static chains give a static transform; animated ones are sampled on every project frame inside
/// the animated range (plus the exact key times and the instant before each hold release), so
/// the result is exact on every integer frame and linear in between.
pub(super) fn bake_parented(chain: &[LocalXf], total: f32, path: &str, budget: &mut usize) -> Option<(crate::project::Transform2D, Vec<Channel>)> {
    let world = |t: f32| chain.iter().rev().fold(Affine2::IDENTITY, |m, x| m.mul(x.matrix(t)));
    let mk = |m: &Affine2| {
        let (pos, rot, scale, skew) = decompose(m);
        crate::project::Transform2D { position: pos, rotation: rot, scale, skew, anchor: Vec2::ZERO }
    };
    if !chain.iter().any(LocalXf::animated) {
        return Some((mk(&world(0.0)), vec![]));
    }
    let mut times = Vec::new();
    for x in chain {
        x.key_times(&mut times);
    }
    let lo = times.iter().cloned().fold(f32::INFINITY, f32::min).max(0.0).floor();
    let hi = times.iter().cloned().fold(f32::NEG_INFINITY, f32::max).min(total).ceil().max(lo);
    let mut samples: Vec<f32> = (lo as i64..=hi as i64).map(|f| f as f32).collect();
    samples.extend(times.iter().filter(|t| **t > lo && **t < hi));
    samples.sort_by(|a, b| a.total_cmp(b));
    samples.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    if *budget < samples.len() * 6 {
        return None;
    }
    *budget -= samples.len() * 6;
    let mut cols: Vec<Vec<(f32, f32, Ease)>> = vec![Vec::new(); 6];
    let mut prev_rot: Option<f32> = None;
    for &t in &samples {
        let (pos, mut rot, scale, skew) = decompose(&world(t));
        if let Some(p) = prev_rot {
            rot += 360.0 * ((p - rot) / 360.0).round();
        }
        prev_rot = Some(rot);
        for (c, v) in [pos.x, pos.y, rot, scale.x, scale.y, skew].into_iter().enumerate() {
            cols[c].push((t, v, Ease::Linear));
        }
    }
    let props = [Prop::LayerX, Prop::LayerY, Prop::LayerRotation, Prop::LayerScaleX, Prop::LayerScaleY, Prop::LayerSkew];
    let first = |c: usize| cols[c][0].1;
    let tf = crate::project::Transform2D { position: Vec2::new(first(0), first(1)), rotation: first(2), scale: Vec2::new(first(3), first(4)), skew: first(5), anchor: Vec2::ZERO };
    let chans = cols
        .iter()
        .enumerate()
        .filter(|(_, k)| k.iter().any(|x| (x.1 - k[0].1).abs() > 1e-5))
        .map(|(c, k)| Channel::keys(path, props[c], Track::with_keys(k)))
        .collect();
    Some((tf, chans))
}

// ---- masks -----------------------------------------------------------------------------------

pub(super) struct MPath {
    pub(super) path: VPath,
    /// Times and easing of every shape key (empty for a static mask path).
    pub(super) keys: Vec<(f32, Ease)>,
}

/// One factor of the layer's mask coverage: the union of `paths` (or its complement).
pub(super) struct Stage {
    pub(super) invert: bool,
    pub(super) paths: Vec<MPath>,
}

pub(super) fn white(a: f32) -> Paint {
    Paint::Solid(Rgba { r: 1.0, g: 1.0, b: 1.0, a })
}

pub(super) fn mask_path(m: &Value, idx: usize, opacity: f32, tm: (f32, f32), nm: &str, warn: &mut Vec<String>) -> Option<MPath> {
    let pt = m.get("pt")?;
    let shape_of = |v: &Value| -> Option<VPath> { path_from_shape(if v.is_array() { v.get(0)? } else { v }) };
    let mut out = if pt.get("a").and_then(Value::as_i64) == Some(1) {
        let ks = pt.get("k")?.as_array()?;
        let mut shapes: Vec<(f32, VPath, Ease)> = Vec::new();
        let mut last_end: Option<VPath> = None;
        for k in ks {
            let t = k.get("t")?.as_f64()? as f32 * tm.0 + tm.1;
            let sh = match k.get("s").and_then(shape_of) {
                Some(s) => s,
                None => last_end.clone()?,
            };
            last_end = k.get("e").and_then(shape_of);
            let kf = Kf { t, v: vec![], hold: k.get("h").and_then(Value::as_i64) == Some(1), bez: bezier_of(k), spatial: false, to: vec![], ti: vec![] };
            shapes.push((t, sh, ease_of(&kf, 0)));
        }
        let first = shapes.first()?.1.clone();
        let mut base = first.clone();
        if shapes.len() > 1 && shapes.iter().any(|s| s.1.anchors.len() != first.anchors.len()) {
            warn.push(format!("layer '{nm}': mask path keys with different vertex counts; the first shape is used"));
            shapes.truncate(1);
        }
        for (i, s) in shapes.iter().enumerate().skip(1) {
            base.morphs.push(crate::path::Morph { name: format!("k{i}"), anchors: s.1.anchors.clone() });
        }
        let keys = if shapes.len() > 1 { shapes.iter().map(|s| (s.0, s.2)).collect() } else { vec![] };
        MPath { path: base, keys }
    } else {
        MPath { path: shape_of(pt.get("k")?)?, keys: vec![] }
    };
    out.path.closed = true;
    out.path.name = format!("p{idx}");
    out.path.fill = Some(white(opacity));
    Some(out)
}

/// Layer masks (`masksProperties`) as a product of coverage stages, applied one by one:
/// add = union (merged into the first stage), subtract = the complement of the shape, intersect =
/// the shape itself. The first mask of a layer starts from "everything" for subtract and from
/// nothing for add / intersect, as in the usual compositing convention.
pub(super) fn parse_masks(l: &Value, nm: &str, tm: (f32, f32), warn: &mut Vec<String>) -> Vec<Stage> {
    let mut stages: Vec<Stage> = Vec::new();
    for (idx, m) in l.get("masksProperties").and_then(Value::as_array).map(|v| v.as_slice()).unwrap_or(&[]).iter().enumerate() {
        let mode = m.get("mode").and_then(Value::as_str).unwrap_or("a");
        if mode == "n" {
            continue;
        }
        let inv = m.get("inv").and_then(Value::as_bool).unwrap_or(false);
        let animated = |k: &str| m.get(k).and_then(|v| v.get("a")).and_then(Value::as_i64) == Some(1);
        let first = |k: &str, d: f32| m.get(k).and_then(|v| stat(v).or_else(|| keyframes(v, tm).map(|k| k[0].v.clone()))).and_then(|v| v.first().copied()).unwrap_or(d);
        let opacity = (first("o", 100.0) / 100.0).clamp(0.0, 1.0);
        if animated("o") {
            warn.push(format!("layer '{nm}': animated mask opacity uses its first value"));
        }
        if first("x", 0.0).abs() > 1e-6 || animated("x") {
            warn.push(format!("layer '{nm}': mask expansion is not imported"));
        }
        let Some(mp) = mask_path(m, idx, opacity, tm, nm, warn) else {
            warn.push(format!("layer '{nm}': malformed mask path skipped"));
            continue;
        };
        let (kind, approximated) = match mode {
            "a" | "s" | "i" => (mode, false),
            "l" => ("a", true),
            "d" => ("i", true),
            other => {
                warn.push(format!("layer '{nm}': mask mode '{other}' is not imported"));
                continue;
            }
        };
        if approximated {
            warn.push(format!("layer '{nm}': mask mode '{mode}' (lighten/darken) is approximated as {}", if kind == "a" { "add" } else { "intersect" }));
        }
        match kind {
            "a" if stages.is_empty() => stages.push(Stage { invert: inv, paths: vec![mp] }),
            "a" if stages.len() == 1 && !stages[0].invert && !inv => stages[0].paths.push(mp),
            "a" => warn.push(format!("layer '{nm}': an add mask after other masks that cannot be merged into one union is not imported")),
            "s" => stages.push(Stage { invert: !inv, paths: vec![mp] }),
            _ => stages.push(Stage { invert: inv, paths: vec![mp] }),
        }
    }
    stages
}

/// Hidden source layers `src1..` for the stages, with the shape-key morph weight channels.
pub(super) fn stage_sources(stages: &[Stage], path: &str, chans: &mut Vec<Channel>) -> Vec<Layer> {
    stages
        .iter()
        .enumerate()
        .map(|(j, st)| {
            let sname = format!("src{}", j + 1);
            let mut lay = Layer::vector(&sname, st.paths.iter().map(|m| m.path.clone()).collect());
            lay.visible = false;
            for m in &st.paths {
                let n = m.keys.len();
                for i in 1..n {
                    let mut keys = vec![(m.keys[i - 1].0, 0.0, m.keys[i - 1].1), (m.keys[i].0, 1.0, m.keys[i].1)];
                    if i + 1 < n {
                        keys.push((m.keys[i + 1].0, 0.0, Ease::Linear));
                    }
                    chans.push(Channel::keys(&format!("{path}/{sname}::{}::k{i}", m.path.name), Prop::PathMorph, Track::with_keys(&keys)));
                }
            }
            lay
        })
        .collect()
}

/// Wrap `core` (named `content`) so that every stage multiplies its coverage:
/// `G[src1.., m_k[ … m_2[ content(mask src1) ] (mask src_k) ]]`.
pub(super) fn masked_group(name: &str, path: &str, mut core: Layer, stages: &[Stage], chans: &mut Vec<Channel>) -> Layer {
    let mut kids = stage_sources(stages, path, chans);
    core.mask = Some(crate::project::MaskRef { layer: format!("{path}/src1"), invert: stages[0].invert });
    let mut cur = core;
    for (j, st) in stages.iter().enumerate().skip(1) {
        let mut g = Layer::group(&format!("m{}", j + 1), vec![cur]);
        g.mask = Some(crate::project::MaskRef { layer: format!("{path}/src{}", j + 1), invert: st.invert });
        cur = g;
    }
    kids.push(cur);
    Layer::group(name, kids)
}

/// Path of the `content` layer inside the wrapper made by [`masked_group`].
pub(super) fn content_path(path: &str, stages: usize) -> String {
    if stages == 0 {
        return path.to_string();
    }
    let ms: String = (2..=stages).rev().map(|j| format!("m{j}/")).collect();
    format!("{path}/{ms}content")
}

/// Engine paths are addressed by name inside a layer, so duplicates (two `rc` items, say) get a suffix.
pub(super) fn unique_path_names(paths: &mut [VPath]) {
    // a counter per base name: the old retry loop probed `name_2`, `name_3`, ... from the start for every duplicate,
    // which is quadratic for a group of thousands of shapes that all carry the default name
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut next: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for p in paths {
        if seen.insert(p.name.clone()) {
            continue;
        }
        let base = p.name.clone();
        let n = next.entry(base.clone()).or_insert(2);
        loop {
            let cand = format!("{base}_{n}");
            *n += 1;
            if seen.insert(cand.clone()) {
                p.name = cand;
                break;
            }
        }
    }
}

// ---- luma mattes -----------------------------------------------------------------------------

pub(super) fn luma_of(c: &Rgba) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

pub(super) fn luma_paint(p: &Paint) -> Paint {
    let conv = |c: &Rgba| Rgba { r: 1.0, g: 1.0, b: 1.0, a: (c.a * luma_of(c)).clamp(0.0, 1.0) };
    match p {
        Paint::Solid(c) => Paint::Solid(conv(c)),
        Paint::Linear { from, to, stops } => Paint::Linear { from: *from, to: *to, stops: stops.iter().map(|s| Stop { offset: s.offset, color: conv(&s.color) }).collect() },
        Paint::Radial { center, radius, stops } => Paint::Radial { center: *center, radius: *radius, stops: stops.iter().map(|s| Stop { offset: s.offset, color: conv(&s.color) }).collect() },
    }
}

/// Turn a matte source into an alpha-only equivalent of its luminance: every paint becomes white
/// with alpha = alpha · luma(colour) (Rec. 709 weights on the colour values as stored). Exact
/// where shapes do not overlap; overlapping shapes of different brightness composite
/// differently from a real luminance matte. Returns false when an image layer was met.
pub(super) fn to_luma_matte(l: &mut Layer) -> bool {
    use crate::project::LayerKind;
    let mut ok = true;
    match &mut l.kind {
        LayerKind::Vector { paths } => {
            for p in paths {
                p.fill = p.fill.as_ref().map(luma_paint);
                if let Some(s) = &mut p.stroke {
                    s.paint = luma_paint(&s.paint);
                }
            }
        }
        LayerKind::Group { layers } | LayerKind::Switch { layers, .. } => {
            for k in layers {
                ok &= to_luma_matte(k);
            }
        }
        LayerKind::Image { .. } => ok = false,
        _ => {}
    }
    ok
}

// ---- image assets ----------------------------------------------------------------------------

/// An embedded image asset: PNG bytes and the display size from the asset's `w`/`h`.
pub(super) type ImageAsset = Result<(Vec<u8>, Option<Vec2>), String>;

pub(super) fn jpeg_to_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut d = jpeg_decoder::Decoder::new(bytes);
    d.read_info().map_err(|e| format!("invalid JPEG: {e}"))?;
    let info = d.info().ok_or("invalid JPEG")?;
    crate::assets::check_dims(info.width as u32, info.height as u32)?;
    let (w, h) = (info.width as usize, info.height as usize);
    let px = d.decode().map_err(|e| format!("invalid JPEG: {e}"))?;
    let rgba: Vec<u8> = match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => px.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        jpeg_decoder::PixelFormat::RGB24 => px.chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
        _ => return Err("only 8-bit grey and RGB JPEGs are supported".into()),
    };
    crate::open_formats::encode_rgba_png(w as u32, h as u32, &rgba)
}

pub(super) fn parse_image_asset(a: &Value) -> ImageAsset {
    let p = a.get("p").and_then(Value::as_str).unwrap_or("");
    let u = a.get("u").and_then(Value::as_str).unwrap_or("");
    let Some(rest) = p.strip_prefix("data:") else {
        return Err(format!("it refers to the external file '{u}{p}', which cannot be fetched"));
    };
    let (head, data) = rest.split_once(',').ok_or("malformed data URI")?;
    if !head.split(';').any(|x| x == "base64") {
        return Err("data URI is not base64".into());
    }
    let bytes = crate::base64::decode(data)?;
    let png = match head.split(';').next().unwrap_or("") {
        "image/png" if bytes.starts_with(&[0x89, b'P', b'N', b'G']) => bytes,
        "image/png" => return Err("data is not a PNG".into()),
        "image/jpeg" | "image/jpg" => jpeg_to_png(&bytes)?,
        other => return Err(format!("image type '{other}' is not supported (PNG and JPEG are)")),
    };
    let dim = |k: &str| a.get(k).and_then(Value::as_f64).filter(|v| *v > 0.0 && *v <= 32768.0).map(|v| v as f32);
    Ok((png, dim("w").zip(dim("h")).map(|(w, h)| Vec2::new(w, h))))
}
