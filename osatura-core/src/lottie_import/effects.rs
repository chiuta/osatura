// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `lottie_import` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---- effects ---------------------------------------------------------------------------------

pub(super) fn effect_name(ty: i64) -> Option<&'static str> {
    Some(match ty {
        20 => "Tint",
        21 => "Fill",
        22 => "Stroke",
        23 => "Tritone",
        24 => "Levels",
        25 => "Drop Shadow",
        26 => "Radial Wipe",
        27 => "Displacement Map",
        28 => "Set Matte",
        29 => "Gaussian Blur",
        30 => "Twirl",
        31 => "Mesh Warp",
        32 => "Ripple",
        33 => "Spherize",
        _ => return None,
    })
}

/// A colour effect that is baked into the solid and gradient colours of a vector layer.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ColorFx {
    /// Map black to `black` and white to `white` by luminance, `amount` 0..1.
    Tint { black: Rgba, white: Rgba, amount: f32 },
    /// Replace the colour, `opacity` 0..1.
    Fill { color: Rgba, opacity: f32 },
    /// Shadows / midtones / highlights ramp by luminance; `blend` 0..1 keeps that share of the original.
    Tritone { high: Rgba, mid: Rgba, shadow: Rgba, blend: f32 },
    /// Input / output range and gamma, all 0..1.
    Levels { in_black: f32, in_white: f32, gamma: f32, out_black: f32, out_white: f32 },
}

/// Tracks that animate a layer effect: `(property, track)`.
pub(super) type FxTracks = Vec<(Prop, Track)>;

pub(super) enum Eff {
    /// Colour effect. `bake` recolours shape layers directly (exact for static values); `layer` is the
    /// per-layer effect form (Tint and Fill only) with the tracks of animated numbers.
    Color { name: &'static str, bake: ColorFx, layer: Option<(LayerEffect, FxTracks)> },
    /// Blur and drop shadow: always per-layer effects.
    Layer { fx: LayerEffect, tracks: FxTracks },
}

pub(super) fn luma(c: Rgba) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

impl ColorFx {
    pub(super) fn apply(&self, c: Rgba) -> Rgba {
        let out = match *self {
            ColorFx::Tint { black, white, amount } => c.lerp(black.lerp(white, luma(c).clamp(0.0, 1.0)), amount),
            ColorFx::Fill { color, opacity } => c.lerp(color, opacity),
            ColorFx::Tritone { high, mid, shadow, blend } => {
                let l = luma(c).clamp(0.0, 1.0);
                let t = if l < 0.5 { shadow.lerp(mid, l * 2.0) } else { mid.lerp(high, (l - 0.5) * 2.0) };
                t.lerp(c, blend)
            }
            ColorFx::Levels { in_black, in_white, gamma, out_black, out_white } => {
                let f = |v: f32| (out_black + ((v - in_black) / (in_white - in_black)).clamp(0.0, 1.0).powf(1.0 / gamma) * (out_white - out_black)).clamp(0.0, 1.0);
                Rgba::new(f(c.r), f(c.g), f(c.b), c.a)
            }
        };
        Rgba { a: c.a, ..out }
    }
}

pub(super) fn recolor_paint(p: &mut Paint, f: &ColorFx) {
    match p {
        Paint::Solid(c) => *c = f.apply(*c),
        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
            for s in stops {
                s.color = f.apply(s.color);
            }
        }
    }
}

/// One effect parameter (`ef[i].v`): its first value and, when animated, the keyframes (in project frames).
pub(super) struct EP {
    pub(super) first: Vec<f32>,
    pub(super) kfs: Option<Vec<Kf>>,
}

pub(super) fn eff_p(e: &Value, i: usize, tm: (f32, f32)) -> Option<EP> {
    let p = e.get("ef")?.as_array()?.get(i)?.get("v")?;
    if !p.is_object() {
        return None;
    }
    let good = |v: &Vec<f32>| !v.is_empty() && v.iter().all(|x| x.is_finite());
    if let Some(v) = stat(p) {
        return Some(EP { first: v, kfs: None }).filter(|e| good(&e.first));
    }
    let k = keyframes(p, tm)?;
    if !k.iter().all(|k| good(&k.v) && k.t.is_finite()) {
        return None;
    }
    Some(EP { first: k[0].v.clone(), kfs: Some(k) })
}

impl EP {
    pub(super) fn animated(&self) -> bool {
        self.kfs.is_some()
    }
    pub(super) fn num(&self) -> f32 {
        self.first[0]
    }
    /// Keys of component 0 scaled by `scale` (None when static).
    pub(super) fn track(&self, scale: f32) -> Option<Track> {
        self.kfs.as_ref().map(|k| track(k, 0, scale))
    }
    /// Value at project frame `t` (the first value when static).
    pub(super) fn at(&self, t: f32) -> f32 {
        match &self.track(1.0) {
            Some(tr) => tr.eval(t).unwrap_or(self.first[0]),
            None => self.first[0],
        }
    }
    pub(super) fn times(&self) -> Vec<f32> {
        self.kfs.iter().flatten().map(|k| k.t).collect()
    }
}

pub(super) fn eff_col_p(e: &Value, i: usize, tm: (f32, f32), colour_anim: &mut bool) -> Option<Rgba> {
    let p = eff_p(e, i, tm)?;
    *colour_anim |= p.animated();
    (p.first.len() >= 3).then(|| Rgba::new(p.first[0].clamp(0.0, 1.0), p.first[1].clamp(0.0, 1.0), p.first[2].clamp(0.0, 1.0), 1.0))
}

/// Most frames sampled for an animated drop-shadow direction / distance.
pub(super) const MAX_SHADOW_SAMPLES: f32 = 2000.0;

/// Read a layer's `ef` array. Supported effects become `Eff`s (in order); disabled ones (`en` = 0)
/// and expression controls (types 0..=7) are skipped silently, everything else is reported by name.
pub(super) fn parse_effects(l: &Value, nm: &str, tm: (f32, f32), warn: &mut Vec<String>) -> Vec<Eff> {
    let mut out = Vec::new();
    let Some(list) = l.get("ef") else { return out };
    let Some(list) = list.as_array() else {
        warn.push(format!("layer '{nm}': malformed effects list ignored"));
        return out;
    };
    for e in list.iter().take(64) {
        if !e.is_object() {
            warn.push(format!("layer '{nm}': malformed effect entry ignored"));
            continue;
        }
        let ty = e.get("ty").and_then(Value::as_i64).unwrap_or(-1);
        if (0..=7).contains(&ty) || e.get("en").and_then(Value::as_i64) == Some(0) {
            continue;
        }
        let what = effect_name(ty).map(str::to_string).or_else(|| e.get("nm").and_then(Value::as_str).map(str::to_string)).unwrap_or_else(|| format!("type {ty}"));
        let mut colour_anim = false; // animated colours (or numbers we cannot animate): first value is used
        let ca = &mut colour_anim;
        let p = |i: usize| eff_p(e, i, tm);
        let num = |i: usize| p(i).map(|x| x.num());
        let unit = |v: f32| (v / 100.0).clamp(0.0, 1.0);
        let parsed: Option<Result<Eff, String>> = match ty {
            20 => (|| {
                let (black, white) = (eff_col_p(e, 0, tm, ca)?, eff_col_p(e, 1, tm, ca)?);
                let amt = p(2)?;
                let a0 = unit(amt.num());
                let tracks: FxTracks = amt.track(0.01).map(|t| vec![(Prop::FxAmount, t)]).unwrap_or_default();
                Some(Ok(Eff::Color { name: "Tint", bake: ColorFx::Tint { black, white, amount: a0 }, layer: Some((LayerEffect::tint(black, white, a0), tracks)) }))
            })(),
            21 => (|| {
                let color = eff_col_p(e, 2, tm, ca)?;
                let op = p(6);
                // opacity is 0..1 in the format; some writers use 0..100
                let pct = op.as_ref().is_some_and(|o| o.first[0] > 1.0 || o.kfs.iter().flatten().any(|k| k.v[0] > 1.0));
                let k = if pct { 0.01 } else { 1.0 };
                let o0 = (op.as_ref().map_or(1.0, |o| o.first[0]) * k).clamp(0.0, 1.0);
                let tracks: FxTracks = op.and_then(|o| o.track(k)).map(|t| vec![(Prop::FxAmount, t)]).unwrap_or_default();
                Some(Ok(Eff::Color { name: "Fill", bake: ColorFx::Fill { color, opacity: o0 }, layer: Some((LayerEffect::fill(color, o0), tracks)) }))
            })(),
            23 => (|| {
                let (high, mid, shadow) = (eff_col_p(e, 0, tm, ca)?, eff_col_p(e, 1, tm, ca)?, eff_col_p(e, 2, tm, ca)?);
                let b = p(3);
                *ca |= b.as_ref().is_some_and(|b| b.animated());
                Some(Ok(Eff::Color { name: "Tritone", bake: ColorFx::Tritone { high, mid, shadow, blend: b.map_or(0.0, |b| unit(b.num())) }, layer: None }))
            })(),
            24 => (|| {
                // layout assumed: 1 input black, 2 input white, 3 gamma, 4 output black, 5 output white (0..255)
                let ps: Vec<Option<EP>> = (1..=5).map(p).collect();
                *ca |= ps.iter().flatten().any(|x| x.animated());
                let n = |i: usize| ps[i].as_ref().map(|x| x.num());
                let (ib, iw) = (n(0)? / 255.0, n(1)? / 255.0);
                let g = n(2)?;
                let ob = n(3).map(|v| v / 255.0).unwrap_or(0.0);
                let ow = n(4).map(|v| v / 255.0).unwrap_or(1.0);
                if !(iw > ib && g > 0.0) {
                    return Some(Err("invalid input range or gamma".to_string()));
                }
                Some(Ok(Eff::Color { name: "Levels", bake: ColorFx::Levels { in_black: ib, in_white: iw, gamma: g, out_black: ob, out_white: ow }, layer: None }))
            })(),
            25 => (|| {
                let color = eff_col_p(e, 0, tm, ca)?;
                let (op, dir, dist, soft) = (p(1)?, p(2)?, p(3)?, p(4)?);
                if dist.first[0].abs() > 4096.0 || !(0.0..=512.0).contains(&soft.first[0]) {
                    return Some(Err("distance or softness out of range".to_string()));
                }
                let mut tracks: FxTracks = Vec::new();
                if let Some(t) = op.track(1.0 / 255.0) {
                    tracks.push((Prop::FxAmount, t));
                }
                if let Some(t) = soft.track(1.0) {
                    tracks.push((Prop::FxSoft, t));
                }
                let xy = |t: f32| {
                    let (d, r) = (dir.at(t).to_radians(), dist.at(t));
                    (d.sin() * r, -d.cos() * r)
                };
                let t0 = dir.times().into_iter().chain(dist.times()).fold(f32::INFINITY, f32::min);
                let (mut dx, mut dy) = xy(if t0.is_finite() { t0 } else { 0.0 });
                if dir.animated() || dist.animated() {
                    let mut times: Vec<f32> = dir.times().into_iter().chain(dist.times()).collect();
                    let (lo, hi) = times.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |a, &t| (a.0.min(t), a.1.max(t)));
                    if hi - lo <= MAX_SHADOW_SAMPLES {
                        // direction is an angle and distance a length, so dx / dy are curved between keys: sample every frame
                        times.extend((lo.ceil() as i32..=hi.floor() as i32).map(|f| f as f32));
                    }
                    times.sort_by(|a, b| a.total_cmp(b));
                    times.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
                    let kx: Vec<(f32, f32, Ease)> = times.iter().map(|&t| (t, xy(t).0, Ease::Linear)).collect();
                    let ky: Vec<(f32, f32, Ease)> = times.iter().map(|&t| (t, xy(t).1, Ease::Linear)).collect();
                    (dx, dy) = (kx[0].1, ky[0].1);
                    tracks.push((Prop::FxDx, Track::with_keys(&kx)));
                    tracks.push((Prop::FxDy, Track::with_keys(&ky)));
                }
                let fx = LayerEffect::drop_shadow(dx, dy, soft.num(), color, (op.num() / 255.0).clamp(0.0, 1.0));
                Some(Ok(Eff::Layer { fx, tracks }))
            })(),
            29 => (|| {
                let r = p(0)?;
                if !(0.0..=512.0).contains(&r.num()) {
                    return Some(Err("blurriness out of range".to_string()));
                }
                match num(1) {
                    Some(d) if d != 1.0 => Some(Err("only blurring in both dimensions is supported".to_string())),
                    _ => Some(Ok(Eff::Layer { fx: LayerEffect::blur(r.num()), tracks: r.track(1.0).map(|t| vec![(Prop::FxBlur, t)]).unwrap_or_default() })),
                }
            })(),
            _ => {
                warn.push(format!("layer '{nm}': effect '{what}' is not imported (no equivalent)"));
                continue;
            }
        };
        match parsed {
            None => warn.push(format!("layer '{nm}': effect '{what}' is malformed (missing or invalid values); not imported")),
            Some(Err(why)) => warn.push(format!("layer '{nm}': effect '{what}' is not imported: {why}")),
            Some(Ok(eff)) => {
                if colour_anim {
                    warn.push(format!("layer '{nm}': effect '{what}' has animated colours or settings that cannot be animated; the first value is used"));
                }
                if ty == 25 && e.get("ef").and_then(Value::as_array).and_then(|a| a.get(5)).and_then(|p| p.get("v")).and_then(stat).is_some_and(|v| v[0] != 0.0) {
                    warn.push(format!("layer '{nm}': 'shadow only' of Drop Shadow is ignored"));
                }
                out.push(eff);
            }
        }
    }
    out
}

pub(super) struct Cx {
    pub(super) warnings: Vec<String>,
    pub(super) easing_noted: bool,
    pub(super) spatial_noted: bool,
    pub(super) assets: std::collections::HashMap<String, Vec<Value>>,
    /// Frames of the whole document.
    pub(super) total: f32,
    pub(super) count: usize,
    pub(super) background: Option<Rgba>,
    pub(super) images: std::collections::HashMap<String, ImageAsset>,
    /// Remaining number of baked keys (parent links) before the importer falls back.
    pub(super) bake_budget: usize,
}

/// Convert Lottie layers (bottom of the list = first drawn) into layers and channels. `tm` maps
/// the keyframe time of these layers to project frames: `frame = t * tm.0 + tm.1`. `prefix` is the
/// `/`-joined path of the enclosing precomp group, used in channel targets.
pub(super) fn convert_layers(cx: &mut Cx, layers: &[Value], tm: (f32, f32), prefix: &str, depth: usize, top: bool) -> (Vec<Layer>, Vec<Channel>) {
    let mut out: Vec<Layer> = Vec::new();
    let mut all_chans: Vec<Channel> = Vec::new();
    let mut used: Vec<String> = Vec::new();
    // (index in `out`, inverted, luma) of the layer waiting for its matte source, which comes next
    let mut pending_matte: Option<(usize, bool, bool)> = None;
    let by_ind: std::collections::HashMap<i64, &Value> = layers.iter().filter_map(|l| Some((l.get("ind")?.as_i64()?, l))).collect();
    for l in layers.iter().rev() {
        let ty = l.get("ty").and_then(Value::as_i64).unwrap_or(-1);
        let nm = l.get("nm").and_then(Value::as_str).unwrap_or("layer");
        cx.count += 1;
        if cx.count > MAX_LAYERS {
            cx.warnings.push("too many layers; the rest were skipped".into());
            break;
        }
        match ty {
            1 if top && cx.background.is_none() && out.is_empty() => {
                if let Some(c) = l.get("sc").and_then(Value::as_str).and_then(Rgba::parse) {
                    cx.background = Some(c);
                }
                continue;
            }
            4 => {}
            3 => continue, // null layer: only a transform holder; children get it baked in
            2 => match l.get("refId").and_then(Value::as_str).and_then(|r| cx.images.get(r)) {
                Some(Ok(_)) => {}
                Some(Err(e)) => {
                    cx.warnings.push(format!("image layer '{nm}' skipped: {e}"));
                    continue;
                }
                None => {
                    cx.warnings.push(format!("image layer '{nm}' skipped: image asset not found"));
                    continue;
                }
            },
            0 if depth < 4 && l.get("refId").and_then(Value::as_str).is_some_and(|r| cx.assets.contains_key(r)) => {}
            0 => {
                cx.warnings.push(format!("precomp layer '{nm}' skipped (composition not found or nested too deeply)"));
                continue;
            }
            other => {
                cx.warnings.push(format!("layer '{nm}' of type {other} skipped"));
                continue;
            }
        }
        let effs = parse_effects(l, nm, tm, &mut cx.warnings);
        // colour effects are baked into shape colours only when static and nothing else needs the layer pass
        let any_layer_fx = effs.iter().any(|e| matches!(e, Eff::Layer { .. }));
        let bakeable = |e: &Eff| matches!(e, Eff::Color { layer, .. } if ty == 4 && !any_layer_fx && layer.as_ref().is_none_or(|(_, t)| t.is_empty()));
        let td = l.get("td").and_then(Value::as_i64).unwrap_or(0);
        let mut stages = parse_masks(l, nm, tm, &mut cx.warnings);
        if td == 1 && !stages.is_empty() {
            cx.warnings.push(format!("layer '{nm}': masks on a track-matte source are not imported (the matte renderer ignores nested masks)"));
            stages.clear();
        }
        let mut name = nm.replace('/', "-");
        let mut n = 2;
        while used.contains(&name) {
            name = format!("{}_{n}", nm.replace('/', "-"));
            n += 1;
        }
        used.push(name.clone());
        let path = format!("{prefix}{name}");
        let mut chans: Vec<Channel> = Vec::new();
        let mut paths_ok = false; // shape layer with at least one path: colours can be baked into it
        let cpath = content_path(&path, stages.len());
        let cname = if stages.is_empty() { name.as_str() } else { "content" };
        let core = if ty == 0 {
            let reference = l.get("refId").and_then(Value::as_str).unwrap_or("");
            let sub = if depth >= 4 { None } else { cx.assets.get(reference).cloned() };
            let (kids, kid_chans) = match sub {
                Some(sub_layers) => {
                    let sr = l.get("sr").and_then(Value::as_f64).unwrap_or(1.0) as f32;
                    let st = l.get("st").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                    let child_tm = (tm.0 * sr, st * tm.0 + tm.1);
                    convert_layers(cx, &sub_layers, child_tm, &format!("{cpath}/"), depth + 1, false)
                }
                None => {
                    cx.warnings.push(format!("precomp layer '{nm}': composition '{reference}' not found or nested too deeply"));
                    (vec![], vec![])
                }
            };
            chans.extend(kid_chans);
            Layer::group(cname, kids)
        } else if ty == 2 {
            let reference = l.get("refId").and_then(Value::as_str).unwrap_or("");
            let Some(Ok((png, size))) = cx.images.get(reference) else { continue };
            let mut im = Layer::image_from_png(cname, png, Vec2::ZERO);
            if let crate::project::LayerKind::Image { size: sz, .. } = &mut im.kind {
                *sz = *size;
            }
            im
        } else {
            let items = l.get("shapes").and_then(Value::as_array).cloned().unwrap_or_default();
            let mut paths: Vec<VPath> = Vec::new();
            let mut anims: Vec<(Vec<(String, Track)>, PaintAnim)> = Vec::new();
            for (vp, ws, pa) in shapes(&items, 0, &mut cx.warnings, tm) {
                paths.push(vp);
                anims.push((ws, pa));
            }
            unique_path_names(&mut paths);
            for (vp, (mut ws, pa)) in paths.iter().zip(anims) {
                let radial = |pt: Option<&Paint>| matches!(pt, Some(Paint::Radial { .. }));
                pa.tracks(radial(vp.fill.as_ref()), radial(vp.stroke.as_ref().map(|s| &s.paint)), &mut ws);
                for (mname, tr) in ws {
                    chans.push(path_channel(&format!("{cpath}::{}", vp.name), &mname, tr));
                }
            }
            paths_ok = !paths.is_empty();
            for f in effs.iter().filter(|e| paths_ok && bakeable(e)).filter_map(|e| if let Eff::Color { bake, .. } = e { Some(bake) } else { None }) {
                for vp in &mut paths {
                    if let Some(p) = &mut vp.fill {
                        recolor_paint(p, f);
                    }
                    if let Some(st) = &mut vp.stroke {
                        recolor_paint(&mut st.paint, f);
                    }
                }
            }
            Layer::vector(cname, paths)
        };
        // everything else becomes a per-layer effect (in `ef` order) with channels for animated numbers
        let mut layer_fx: Vec<(LayerEffect, FxTracks)> = Vec::new();
        for e in effs {
            match e {
                _ if paths_ok && bakeable(&e) => {
                    if !layer_fx.is_empty() {
                        cx.warnings.push(format!("layer '{nm}': a colour effect after a blur or shadow is applied before it"));
                    }
                }
                Eff::Color { layer: Some(fx), .. } => layer_fx.push(fx),
                Eff::Color { name, .. } => cx.warnings.push(format!("layer '{nm}': colour effect '{name}' is only imported on shape layers (and not together with an animated or blur / shadow effect there); not imported")),
                Eff::Layer { fx, tracks } => layer_fx.push((fx, tracks)),
            }
        }
        if layer_fx.len() > crate::layerfx::MAX_EFFECTS {
            cx.warnings.push(format!("layer '{nm}': only the first {} effects are imported", crate::layerfx::MAX_EFFECTS));
            layer_fx.truncate(crate::layerfx::MAX_EFFECTS);
        }
        let mut layer = if stages.is_empty() { core } else { masked_group(&name, &path, core, &stages, &mut chans) };
        for (i, (fx, tracks)) in layer_fx.into_iter().enumerate() {
            layer.effects.push(fx);
            for (prop, tr) in tracks {
                chans.push(Channel::keys(&crate::layerfx::target(&path, i), prop, tr));
            }
        }
        layer.blend = blend_from(l.get("bm").and_then(Value::as_i64).unwrap_or(0));
        let ks = l.get("ks").cloned().unwrap_or(Value::Null);
        let chain = if l.get("parent").is_some_and(|p| !p.is_null()) { parent_chain(l, &by_ind, tm, &mut cx.warnings) } else { None };
        let baked = chain.and_then(|c| {
            let r = bake_parented(&c, cx.total, &path, &mut cx.bake_budget);
            if r.is_none() {
                cx.warnings.push(format!("layer '{nm}': parent chain animation is too long to bake; parent link ignored"));
            }
            r.map(|r| (c, r))
        });
        let parented = baked.is_some();
        let (easing_noted, spatial_noted) = (&mut cx.easing_noted, &mut cx.spatial_noted);
        let mut handle = |key: &str, props: &[(Prop, usize, f32)], apply: &mut dyn FnMut(&[f32])| {
            let Some(prop) = ks.get(key) else { return };
            if parented && key != "o" {
                return; // transform comes from the baked parent chain below
            }
            if let Some(v) = stat(prop) {
                apply(&v);
            } else if let Some(kfs) = keyframes(prop, tm) {
                apply(&kfs[0].v);
                if kfs[..kfs.len() - 1].iter().any(|k| !k.hold && k.bez.is_none()) {
                    *easing_noted = true;
                }
                if kfs.iter().any(|k| k.spatial) && key != "p" {
                    *spatial_noted = true;
                }
                if key == "p" && kfs.iter().any(|k| k.spatial) && props.len() == 2 {
                    let (tx, ty) = bake_spatial(&kfs);
                    chans.push(Channel::keys(&path, props[0].0, tx));
                    chans.push(Channel::keys(&path, props[1].0, ty));
                    return;
                }
                for &(pr, comp, sc) in props {
                    chans.push(Channel::keys(&path, pr, track(&kfs, comp, sc)));
                }
            }
        };
        let mut t = layer.transform.clone();
        let mut opacity = 1.0f32;
        handle("p", &[(Prop::LayerX, 0, 1.0), (Prop::LayerY, 1, 1.0)], &mut |v| t.position = Vec2::new(v[0], *v.get(1).unwrap_or(&0.0)));
        handle("a", &[], &mut |v| t.anchor = Vec2::new(v[0], *v.get(1).unwrap_or(&0.0)));
        handle("s", &[(Prop::LayerScaleX, 0, 0.01), (Prop::LayerScaleY, 1, 0.01)], &mut |v| t.scale = Vec2::new(v[0] / 100.0, *v.get(1).unwrap_or(&100.0) / 100.0));
        handle("r", &[(Prop::LayerRotation, 0, 1.0)], &mut |v| t.rotation = v[0]);
        handle("o", &[(Prop::LayerOpacity, 0, 0.01)], &mut |v| opacity = v[0] / 100.0);
        layer.transform = t;
        if let Some((_, (tf, bc))) = baked {
            layer.transform = tf;
            chans.extend(bc);
        }
        layer.opacity = opacity.clamp(0.0, 1.0);
        if ks.get("a").is_some_and(|a| a.get("a").and_then(Value::as_i64) == Some(1)) {
            cx.warnings.push(format!("layer '{nm}': animated anchor point uses its first value"));
        }
        // visibility window as a multiplying opacity step track (Lottie `op` is exclusive)
        let map = |v: f64| v as f32 * tm.0 + tm.1;
        let lip = l.get("ip").and_then(Value::as_f64).map(map).filter(|v| *v > 0.5);
        let lop = l.get("op").and_then(Value::as_f64).map(map).filter(|v| *v < cx.total - 0.5);
        if lip.is_some() || lop.is_some() {
            let mut keys: Vec<(f32, f32, Ease)> = vec![(0.0, if lip.is_some() { 0.0 } else { 1.0 }, Ease::Step)];
            if let Some(a) = lip {
                keys.push((a, 1.0, Ease::Step));
            }
            if let Some(b) = lop {
                keys.push((b, 0.0, Ease::Step));
            }
            chans.push(Channel::keys(&path, Prop::LayerOpacity, Track::with_keys(&keys)).with_mode(Mode::Multiply));
        }
        // track mattes: `tt` 1/2 = alpha / inverted alpha, 3/4 = luma / inverted luma of the layer above (`td`), which is not drawn itself
        let tt = l.get("tt").and_then(Value::as_i64).unwrap_or(0);
        if td == 1 {
            match pending_matte.take() {
                Some((idx, invert, luma)) => {
                    if luma {
                        if !to_luma_matte(&mut layer) {
                            cx.warnings.push(format!("layer '{nm}': an image used as luma matte is treated by its alpha"));
                        }
                        cx.warnings.push(format!("layer '{nm}': luma matte approximated by white shapes with alpha = alpha x luminance (exact only where shapes do not overlap)"));
                    }
                    out[idx].mask = Some(crate::project::MaskRef { layer: path.clone(), invert });
                    layer.visible = false;
                }
                None => cx.warnings.push(format!("layer '{nm}': matte source without a matted layer is drawn normally")),
            }
        } else {
            pending_matte = None;
        }
        match tt {
            0 => {}
            1 | 2 => pending_matte = Some((out.len(), tt == 2, false)),
            3 | 4 => pending_matte = Some((out.len(), tt == 4, true)),
            other => cx.warnings.push(format!("layer '{nm}': track matte type {other} is not imported")),
        }
        out.push(layer);
        all_chans.extend(chans);
    }
    (out, all_chans)
}

/// Most array elements plus object members a Lottie file may hold (keyframes and vertices are arrays); the
/// importer needs memory and time proportional to it, some of it quadratic in shape morphs.
pub(super) const MAX_VALUES: usize = 4_000_000;

pub(super) fn count_values(v: &Value) -> usize {
    match v {
        Value::Array(a) => a.len() + a.iter().map(count_values).sum::<usize>(),
        Value::Object(m) => m.len() + m.values().map(count_values).sum::<usize>(),
        _ => 0,
    }
}

pub fn import_lottie(json: &str) -> Result<Imported, String> {
    let doc: Value = serde_json::from_str(json).map_err(|e| format!("not valid JSON: {e}"))?;
    let values = count_values(&doc);
    if values > MAX_VALUES {
        return Err(format!("the file holds {values} values (keyframes, vertices, ...), more than the limit of {MAX_VALUES}"));
    }
    let num = |k: &str| doc.get(k).and_then(Value::as_f64).ok_or_else(|| format!("missing '{k}'"));
    let (w, h, fr, ip, op) = (num("w")?, num("h")?, num("fr")?, num("ip")?, num("op")?);
    if !(w > 0.0 && h > 0.0 && fr > 0.0 && fr <= 1000.0 && op > ip && op - ip <= MAX_FRAMES) {
        return Err("size, frame rate or frame range is out of bounds".into());
    }
    let layers = doc.get("layers").and_then(Value::as_array).ok_or("missing 'layers'")?;
    if layers.len() > MAX_LAYERS {
        return Err("too many layers".into());
    }
    let mut p = Project::new(doc.get("nm").and_then(Value::as_str).unwrap_or("lottie"), w as f32, h as f32, fr as f32, (op - ip).round() as u32);
    let mut warnings = Vec::new();
    let mut assets: std::collections::HashMap<String, Vec<Value>> = std::collections::HashMap::new();
    let mut images: std::collections::HashMap<String, ImageAsset> = std::collections::HashMap::new();
    let mut other_assets = false;
    for a in doc.get("assets").and_then(Value::as_array).map(|v| v.as_slice()).unwrap_or(&[]) {
        match (a.get("id").and_then(Value::as_str), a.get("layers").and_then(Value::as_array)) {
            (Some(id), Some(ls)) => {
                assets.insert(id.to_string(), ls.clone());
            }
            (Some(id), None) if a.get("p").is_some() => {
                images.insert(id.to_string(), parse_image_asset(a));
            }
            _ => other_assets = true,
        }
    }
    if other_assets {
        warnings.push("assets that are neither precompositions nor images (for example audio) are not imported".into());
    }
    let mut cx = Cx { warnings, easing_noted: false, spatial_noted: false, assets, total: (op - ip) as f32, count: 0, background: None, images, bake_budget: 3_000_000 };
    let (lays, chans) = convert_layers(&mut cx, layers, (1.0, -(ip as f32)), "", 0, true);
    p.layers = lays;
    p.channels = chans;
    p.background = cx.background;
    let (mut warnings, easing_noted, spatial_noted) = (cx.warnings, cx.easing_noted, cx.spatial_noted);
    if easing_noted {
        warnings.push("keyframes without easing handles are linear".into());
    }
    if spatial_noted {
        warnings.push("curved motion paths on properties other than position are imported as straight segments".into());
    }
    warnings.sort();
    warnings.dedup();
    crate::engine::check_magnitudes(&p).map_err(|e| format!("the file cannot be imported: {e}"))?;
    Ok(Imported { project: p, post_fx: Vec::new(), warnings })
}
