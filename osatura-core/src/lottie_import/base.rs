// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `lottie_import` (see the module docs in `mod.rs`).
use super::*;

pub(super) const MAX_LAYERS: usize = 2000;

pub(super) const MAX_DEPTH: usize = 32;

pub(super) const MAX_FRAMES: f64 = 20_000.0;

pub struct Imported {
    pub project: Project,
    /// Kept for compatibility; always empty now. Blur and drop shadow are per-layer effects (`Layer::effects`).
    pub post_fx: Vec<crate::postfx::Fx>,
    pub warnings: Vec<String>,
}

pub(super) fn nums(v: &Value) -> Option<Vec<f32>> {
    match v {
        Value::Number(n) => Some(vec![n.as_f64()? as f32]),
        Value::Array(a) => a.iter().map(|x| x.as_f64().map(|f| f as f32)).collect(),
        _ => None,
    }
}

/// Static value of a Lottie property `{a:0, k:…}`.
pub(super) fn stat(prop: &Value) -> Option<Vec<f32>> {
    if prop.get("a").and_then(Value::as_i64).unwrap_or(0) == 1 {
        return None;
    }
    nums(prop.get("k")?)
}

pub(super) struct Kf {
    pub(super) t: f32,
    pub(super) v: Vec<f32>,
    pub(super) hold: bool,
    /// Easing of the segment that starts here: `(o.x, o.y, i.x, i.y)` per component.
    pub(super) bez: Option<[Vec<f32>; 4]>,
    /// The key has spatial tangents (`to`/`ti`): the motion to the next key is a curve.
    pub(super) spatial: bool,
    pub(super) to: Vec<f32>,
    pub(super) ti: Vec<f32>,
}

pub(super) fn handle_vals(v: Option<&Value>) -> Option<Vec<f32>> {
    match v? {
        Value::Array(a) => a.iter().map(|x| x.as_f64().map(|f| f as f32)).collect::<Option<Vec<_>>>().filter(|v| !v.is_empty()),
        x => x.as_f64().map(|f| vec![f as f32]),
    }
}

pub(super) fn bezier_of(k: &Value) -> Option<[Vec<f32>; 4]> {
    let (o, i) = (k.get("o")?, k.get("i")?);
    Some([handle_vals(o.get("x"))?, handle_vals(o.get("y"))?, handle_vals(i.get("x"))?, handle_vals(i.get("y"))?])
}

/// Largest keyframe time or value taken from a file (the project format allows +-1e7 everywhere).
pub(super) const KEY_LIMIT: f32 = 1.0e6;

pub(super) fn keyframes(prop: &Value, tm: (f32, f32)) -> Option<Vec<Kf>> {
    if prop.get("a").and_then(Value::as_i64).unwrap_or(0) != 1 {
        return None;
    }
    let arr = prop.get("k")?.as_array()?;
    let mut out: Vec<Kf> = Vec::new();
    let mut last_end: Option<Vec<f32>> = None;
    for k in arr {
        let t = k.get("t")?.as_f64()? as f32 * tm.0 + tm.1;
        let v = match k.get("s").and_then(nums) {
            Some(v) => v,
            None => last_end.clone()?, // final keyframe of the old format has only the previous `e`
        };
        // absurd times or values (1e30) would make the project unloadable: treat the property as not animated
        if !t.is_finite() || t.abs() > KEY_LIMIT || v.iter().any(|x| !x.is_finite() || x.abs() > KEY_LIMIT) {
            return None;
        }
        last_end = k.get("e").and_then(nums);
        let spatial = k.get("to").is_some_and(|x| nums(x).is_some_and(|n| n.iter().any(|v| v.abs() > 1e-6))) || k.get("ti").is_some_and(|x| nums(x).is_some_and(|n| n.iter().any(|v| v.abs() > 1e-6)));
        out.push(Kf { t, v, hold: k.get("h").and_then(Value::as_i64) == Some(1), bez: bezier_of(k), spatial, to: k.get("to").and_then(nums).unwrap_or_default(), ti: k.get("ti").and_then(nums).unwrap_or_default() });
    }
    if out.is_empty() { None } else { Some(out) }
}

pub(super) fn rgba(c: &[f32], opacity: f32) -> Rgba {
    Rgba { r: *c.first().unwrap_or(&0.0), g: *c.get(1).unwrap_or(&0.0), b: *c.get(2).unwrap_or(&0.0), a: (*c.get(3).unwrap_or(&1.0) * opacity).clamp(0.0, 1.0) }
}

pub(super) fn corner_or(pos: Vec2, i: Vec2, o: Vec2) -> Anchor {
    let nz = |d: Vec2| d.x.abs() > 1e-4 || d.y.abs() > 1e-4;
    Anchor { pos, handle_in: nz(i).then(|| pos + i), handle_out: nz(o).then(|| pos + o), weights: Vec::new() }
}

pub(super) fn vec2s(v: &Value) -> Option<Vec<Vec2>> {
    v.as_array()?.iter().map(|p| nums(p).filter(|n| n.len() >= 2).map(|n| Vec2::new(n[0], n[1]))).collect()
}

pub(super) fn path_from_shape(shape: &Value) -> Option<VPath> {
    let (v, i, o) = (vec2s(shape.get("v")?)?, vec2s(shape.get("i")?)?, vec2s(shape.get("o")?)?);
    if v.len() < 2 || i.len() != v.len() || o.len() != v.len() {
        return None;
    }
    let anchors = (0..v.len()).map(|k| corner_or(v[k], i[k], o[k])).collect();
    Some(VPath::new("path", shape.get("c").and_then(Value::as_bool).unwrap_or(false), anchors))
}

/// One keyframe of an animated `sh` property.
pub(super) struct ShapeKf {
    pub(super) t: f32,
    pub(super) shape: VPath,
    pub(super) hold: bool,
    pub(super) bez: Option<[Vec<f32>; 4]>,
}

/// Keyframed shape (`ks.a == 1`): the shapes at each key, or `None` when the property is malformed.
pub(super) fn shape_keyframes(prop: &Value, tm: (f32, f32)) -> Option<Vec<ShapeKf>> {
    let arr = prop.get("k")?.as_array()?;
    let mut out: Vec<ShapeKf> = Vec::new();
    let mut last_end: Option<VPath> = None;
    for k in arr {
        let t = k.get("t")?.as_f64()? as f32 * tm.0 + tm.1;
        let from = |v: &Value| -> Option<VPath> {
            match v {
                Value::Array(a) => a.first().and_then(path_from_shape),
                o => path_from_shape(o),
            }
        };
        let shape = match k.get("s").and_then(from) {
            Some(s) => s,
            None => last_end.clone()?, // last key of the old format only has the previous `e`
        };
        last_end = k.get("e").and_then(from);
        if !t.is_finite() || t.abs() > KEY_LIMIT {
            return None;
        }
        out.push(ShapeKf { t, shape, hold: k.get("h").and_then(Value::as_i64) == Some(1), bez: bezier_of(k) });
    }
    if out.is_empty() { None } else { Some(out) }
}

/// An animated path as a base shape (the first key) plus one morph target per further distinct key and the weight
/// tracks that blend between consecutive keys: `base + sum(w_j * (key_j - base))` with `w_j = 1` at key `j` and `0`
/// at the others reproduces linear interpolation between the neighbouring shapes, with each key's easing/hold.
/// Keys with a different topology (anchor count or open/closed) cannot be morphed: the first shape is used and `Err`
/// carries the reason.
pub(super) fn animated_path(kfs: &[ShapeKf]) -> Result<(VPath, Vec<(String, Track)>), String> {
    let base = &kfs[0].shape;
    if let Some(bad) = kfs.iter().find(|k| k.shape.anchors.len() != base.anchors.len() || k.shape.closed != base.closed) {
        return Err(format!("path keyframes at time {} change the number of points or open/closed; only the first shape was imported", bad.t));
    }
    let ease = |k: &ShapeKf| {
        let kf = Kf { t: k.t, v: vec![], hold: k.hold, bez: k.bez.clone(), spatial: false, to: vec![], ti: vec![] };
        ease_of(&kf, 0)
    };
    let mut p = base.clone();
    let mut weights = Vec::new();
    for j in 1..kfs.len() {
        if kfs[j].shape.anchors == base.anchors {
            continue; // identical to the base: contributes nothing
        }
        let name = format!("k{j}");
        let lo = j - 1;
        let hi = (j + 1).min(kfs.len() - 1);
        let keys: Vec<(f32, f32, Ease)> = (lo..=hi).map(|i| (kfs[i].t, if i == j { 1.0 } else { 0.0 }, ease(&kfs[i]))).collect();
        if keys.windows(2).any(|w| w[1].0.partial_cmp(&(w[0].0)) != Some(std::cmp::Ordering::Greater)) {
            return Err("path keyframes are not in increasing time order; only the first shape was imported".into());
        }
        p.morphs.push(crate::path::Morph { name: name.clone(), anchors: kfs[j].shape.anchors.clone() });
        weights.push((name, Track::with_keys(&keys)));
    }
    Ok((p, weights))
}

pub(super) fn ellipse(c: Vec2, r: Vec2) -> VPath {
    const K: f32 = 0.552_284_8;
    let a = |dx: f32, dy: f32, ix: f32, iy: f32, ox: f32, oy: f32| corner_or(c + Vec2::new(dx, dy), Vec2::new(ix, iy), Vec2::new(ox, oy));
    VPath::new("ellipse", true, vec![
        a(r.x, 0.0, 0.0, -K * r.y, 0.0, K * r.y),
        a(0.0, r.y, K * r.x, 0.0, -K * r.x, 0.0),
        a(-r.x, 0.0, 0.0, K * r.y, 0.0, -K * r.y),
        a(0.0, -r.y, -K * r.x, 0.0, K * r.x, 0.0),
    ])
}

pub(super) fn transform_path(p: &mut VPath, m: &Affine2) {
    for a in &mut p.anchors {
        let (hi, ho) = (a.handle_in.map(|h| m.apply(h)), a.handle_out.map(|h| m.apply(h)));
        a.pos = m.apply(a.pos);
        a.handle_in = hi;
        a.handle_out = ho;
    }
}

/// Animation of one gradient handle: `(frame, offset from the first key, ease of x, ease of y)`; for the radius of a
/// radial gradient only the x offset is used.
#[derive(Clone)]
pub(super) struct GradAnim {
    /// `"from"` (linear start / radial centre) or `"to"` (linear end / radial radius)
    pub(super) part: &'static str,
    pub(super) keys: Vec<(f32, [f32; 2], Ease, Ease)>,
}

/// Gradient geometry of a `gf` / `gs` item: the paint at its first key and the animation of the handles. The
/// project model animates gradients through the `grad_dx` / `grad_dy` channels of the path (`fill.from`, `fill.to`,
/// `stroke.from`, `stroke.to`): the start point (linear) or centre (radial) as offsets, the end point (linear) as an
/// offset and the radius (radial) as `|end - start|` sampled per frame.
pub(super) fn gradient(item: &Value, opacity: f32, warn: &mut Vec<String>, tm: (f32, f32)) -> Option<(Paint, Vec<GradAnim>)> {
    let (sp, ep) = (item.get("s")?, item.get("e")?);
    let sk = if stat(sp).is_some() { None } else { Some(keyframes(sp, tm)?) };
    let ek = if stat(ep).is_some() { None } else { Some(keyframes(ep, tm)?) };
    let s = stat(sp).or_else(|| sk.as_ref().map(|k| k[0].v.clone()))?;
    let e = stat(ep).or_else(|| ek.as_ref().map(|k| k[0].v.clone()))?;
    if s.len() < 2 || e.len() < 2 {
        return None;
    }
    let radial = item.get("t").and_then(Value::as_i64) == Some(2);
    let mut anims: Vec<GradAnim> = Vec::new();
    let offsets = |kf: &[Kf], base: &[f32]| -> Vec<(f32, [f32; 2], Ease, Ease)> {
        kf.iter().map(|k| (k.t, [k.v.first().copied().unwrap_or(0.0) - base[0], k.v.get(1).copied().unwrap_or(0.0) - base[1]], ease_of(k, 0), ease_of(k, 1))).collect()
    };
    if let Some(k) = &sk {
        anims.push(GradAnim { part: "from", keys: offsets(k, &s) });
    }
    if radial {
        if sk.is_some() || ek.is_some() {
            // radius = |end - start| is not piecewise linear in time: sample it on every frame between the first and last key
            let at = |k: &Option<Vec<Kf>>, base: &[f32], t: f32, c: usize| -> f32 {
                match k {
                    None => base[c],
                    Some(kf) => track(kf, c, 1.0).eval(t).unwrap_or(base[c]),
                }
            };
            let all: Vec<f32> = sk.iter().chain(ek.iter()).flatten().map(|k| k.t).collect();
            let (t0, t1) = (all.iter().cloned().fold(f32::MAX, f32::min), all.iter().cloned().fold(f32::MIN, f32::max));
            let r0 = ((e[0] - s[0]).powi(2) + (e[1] - s[1]).powi(2)).sqrt();
            let step = ((t1 - t0) / 2000.0).ceil().max(1.0);
            let mut keys = Vec::new();
            let mut t = t0;
            loop {
                let r = ((at(&ek, &e, t, 0) - at(&sk, &s, t, 0)).powi(2) + (at(&ek, &e, t, 1) - at(&sk, &s, t, 1)).powi(2)).sqrt();
                keys.push((t, [r - r0, 0.0], Ease::Linear, Ease::Linear));
                if t >= t1 {
                    break;
                }
                t = (t + step).min(t1);
            }
            anims.push(GradAnim { part: "to", keys });
        }
    } else if let Some(k) = &ek {
        anims.push(GradAnim { part: "to", keys: offsets(k, &e) });
    }
    let _ = warn;
    let count = item.pointer("/g/p")?.as_u64()? as usize;
    let k = stat(item.pointer("/g/k")?)?;
    if k.len() < count * 4 || count == 0 {
        return None;
    }
    // colour stops, then (optional) alpha stops [t, a]*: Lottie's "colour with opacity" layout
    let alpha_at = |t: f32| -> f32 {
        let base = count * 4;
        if k.len() < base + 2 {
            return 1.0;
        }
        let pts: Vec<(f32, f32)> = k[base..].chunks_exact(2).map(|c| (c[0], c[1])).collect();
        match pts.iter().position(|p| p.0 >= t) {
            None => pts.last().map_or(1.0, |p| p.1),
            Some(0) => pts[0].1,
            Some(i) => {
                let (a, b) = (pts[i - 1], pts[i]);
                if b.0 - a.0 < 1e-9 { b.1 } else { a.1 + (b.1 - a.1) * (t - a.0) / (b.0 - a.0) }
            }
        }
    };
    let stops: Vec<Stop> = (0..count).map(|i| Stop { offset: k[i * 4], color: Rgba { r: k[i * 4 + 1], g: k[i * 4 + 2], b: k[i * 4 + 3], a: opacity * alpha_at(k[i * 4]) } }).collect();
    let (a, b) = (Vec2::new(s[0], s[1]), Vec2::new(e[0], e[1]));
    let paint = if radial { Paint::Radial { center: a, radius: a.distance(b), stops } } else { Paint::Linear { from: a, to: b, stops } };
    Some((paint, anims))
}

/// Moves a paint through the group transform `xf` (Lottie gradient points live in the group's space).
pub(super) fn transform_paint(p: &mut Paint, xf: &Affine2) {
    let geo = (xf.a * xf.d - xf.b * xf.c).abs().sqrt();
    match p {
        Paint::Linear { from, to, .. } => {
            *from = xf.apply(*from);
            *to = xf.apply(*to);
        }
        Paint::Radial { center, radius, .. } => {
            *center = xf.apply(*center);
            *radius *= geo;
        }
        Paint::Solid(_) => {}
    }
}

/// Gradient animation of a path's fill and stroke (offsets in the space of the shape group that holds them).
#[derive(Default, Clone)]
pub(super) struct PaintAnim {
    pub(super) fill: Vec<GradAnim>,
    pub(super) stroke: Vec<GradAnim>,
}

impl PaintAnim {
    /// The same animation seen through the group transform `xf`: handle offsets go through its linear part, the
    /// radius of a radial gradient is scaled by the geometric mean scale.
    pub(super) fn through(&mut self, xf: &Affine2, fill_radial: bool, stroke_radial: bool) {
        let geo = (xf.a * xf.d - xf.b * xf.c).abs().sqrt();
        for (list, radial) in [(&mut self.fill, fill_radial), (&mut self.stroke, stroke_radial)] {
            for g in list.iter_mut() {
                for k in &mut g.keys {
                    k.1 = if radial && g.part == "to" { [k.1[0] * geo, 0.0] } else { [xf.a * k.1[0] + xf.c * k.1[1], xf.b * k.1[0] + xf.d * k.1[1]] };
                }
            }
        }
    }

    /// `(name, track)` entries; names start with `@` and read `@<fill|stroke>.<from|to>:<x|y>` (see [`path_channel`]).
    pub(super) fn tracks(&self, fill_radial: bool, stroke_radial: bool, out: &mut Vec<(String, Track)>) {
        for (which, list, radial) in [("fill", &self.fill, fill_radial), ("stroke", &self.stroke, stroke_radial)] {
            for g in list {
                let (mut kx, mut ky) = (Vec::new(), Vec::new());
                for &(t, d, ex, ey) in &g.keys {
                    kx.push((t, d[0], ex));
                    if !(radial && g.part == "to") {
                        ky.push((t, d[1], ey));
                    }
                }
                out.push((format!("@{which}.{}:x", g.part), Track::with_keys(&kx)));
                if !ky.is_empty() {
                    out.push((format!("@{which}.{}:y", g.part), Track::with_keys(&ky)));
                }
            }
        }
    }
}

/// Channel for a `(name, track)` entry of an animated path: a morph weight, or a gradient handle (`@...`).
pub(super) fn path_channel(target_prefix: &str, name: &str, tr: Track) -> Channel {
    match name.strip_prefix('@') {
        Some(rest) => {
            let (handle, axis) = rest.rsplit_once(':').unwrap_or((rest, "x"));
            Channel::keys(&format!("{target_prefix}::{handle}"), if axis == "y" { Prop::GradDy } else { Prop::GradDx }, tr)
        }
        None => Channel::keys(&format!("{target_prefix}::{name}"), Prop::PathMorph, tr),
    }
}

/// A path with the (morph name, weight track) pairs that animate it.
pub(super) type AnimPath = (VPath, Vec<(String, Track)>, PaintAnim);

pub(super) fn shapes(items: &[Value], depth: usize, warn: &mut Vec<String>, tm: (f32, f32)) -> Vec<AnimPath> {
    if depth > MAX_DEPTH {
        warn.push("shape groups nested too deeply; deeper ones skipped".into());
        return vec![];
    }
    let mut geo: Vec<AnimPath> = Vec::new();
    let mut styled: Vec<AnimPath> = Vec::new();
    let (mut fill, mut stroke): (Option<Paint>, Option<StrokeStyle>) = (None, None);
    let (mut fill_anim, mut stroke_anim): (Vec<GradAnim>, Vec<GradAnim>) = (Vec::new(), Vec::new());
    let mut xf = Affine2::IDENTITY;
    let mut group_opacity = 1.0f32;
    for it in items {
        match it.get("ty").and_then(Value::as_str).unwrap_or("") {
            "sh" => {
                let ks = it.get("ks");
                if ks.and_then(|k| k.get("a")).and_then(Value::as_i64) == Some(1) {
                    match ks.and_then(|k| shape_keyframes(k, tm)) {
                        Some(kfs) if kfs.len() == 1 => geo.push((kfs[0].shape.clone(), vec![], PaintAnim::default())),
                        Some(kfs) => match animated_path(&kfs) {
                            Ok(ap) => geo.push((ap.0, ap.1, PaintAnim::default())),
                            Err(why) => {
                                warn.push(why);
                                geo.push((kfs[0].shape.clone(), vec![], PaintAnim::default()));
                            }
                        },
                        None => warn.push("malformed animated path skipped".into()),
                    }
                } else {
                    match ks.and_then(|k| k.get("k")).and_then(path_from_shape) {
                        Some(p) => geo.push((p, vec![], PaintAnim::default())),
                        None => {
                            let empty = ks.and_then(|k| k.get("k")).and_then(|k| k.get("v")).and_then(Value::as_array).is_some_and(|v| v.is_empty());
                            if !empty {
                                warn.push("animated or malformed path skipped".into());
                            }
                        }
                    }
                }
            }
            "rc" => {
                if let (Some(p), Some(s)) = (it.get("p").and_then(stat), it.get("s").and_then(stat)) {
                    if p.len() >= 2 && s.len() >= 2 {
                        geo.push((VPath::rect("rect", p[0] - s[0] / 2.0, p[1] - s[1] / 2.0, s[0], s[1]), vec![], PaintAnim::default()));
                        if it.get("r").and_then(stat).is_some_and(|r| r.first().copied().unwrap_or(0.0) > 0.0) {
                            warn.push("rounded rectangle imported with square corners".into());
                        }
                        continue;
                    }
                }
                warn.push("animated rectangle skipped".into());
            }
            "el" => match (it.get("p").and_then(stat), it.get("s").and_then(stat)) {
                (Some(p), Some(s)) if p.len() >= 2 && s.len() >= 2 => geo.push((ellipse(Vec2::new(p[0], p[1]), Vec2::new(s[0] / 2.0, s[1] / 2.0)), vec![], PaintAnim::default())),
                _ => warn.push("animated ellipse skipped".into()),
            },
            "fl" => {
                let o = it.get("o").and_then(stat).and_then(|v| v.first().copied()).unwrap_or(100.0) / 100.0;
                fill = it.get("c").and_then(stat).map(|c| Paint::Solid(rgba(&c, o)));
                fill_anim.clear();
                if fill.is_none() {
                    warn.push("animated fill colour skipped".into());
                }
            }
            "gf" => {
                let o = it.get("o").and_then(stat).and_then(|v| v.first().copied()).unwrap_or(100.0) / 100.0;
                let g = gradient(it, o, warn, tm);
                (fill, fill_anim) = match g {
                    Some((p, a)) => (Some(p), a),
                    None => (None, Vec::new()),
                };
                if fill.is_none() {
                    warn.push("unsupported gradient skipped".into());
                }
            }
            "st" | "gs" => {
                let o = it.get("o").and_then(stat).and_then(|v| v.first().copied()).unwrap_or(100.0) / 100.0;
                let w = it.get("w").and_then(stat).and_then(|v| v.first().copied()).unwrap_or(1.0);
                stroke_anim.clear();
                let paint = if it.get("ty").and_then(Value::as_str) == Some("gs") {
                    gradient(it, o, warn, tm).map(|(p, a)| {
                        stroke_anim = a;
                        p
                    })
                } else {
                    it.get("c").and_then(stat).map(|c| Paint::Solid(rgba(&c, o)))
                };
                stroke = paint.map(|pt| {
                    let mut s = StrokeStyle::solid("#000000", w.max(0.0));
                    s.paint = pt;
                    s.cap = match it.get("lc").and_then(Value::as_i64) { Some(1) => crate::style::Cap::Butt, Some(3) => crate::style::Cap::Square, _ => crate::style::Cap::Round };
                    s.join = match it.get("lj").and_then(Value::as_i64) { Some(1) => crate::style::Join::Miter, Some(3) => crate::style::Join::Bevel, _ => crate::style::Join::Round };
                    let mut dash = Vec::new();
                    for d in it.get("d").and_then(Value::as_array).into_iter().flatten() {
                        let v = d.get("v").and_then(stat).and_then(|v| v.first().copied()).unwrap_or(0.0);
                        match d.get("n").and_then(Value::as_str) {
                            Some("o") => s.dash_offset = v,
                            Some("d") | Some("g") => dash.push(v),
                            _ => {}
                        }
                    }
                    if dash.len() >= 2 && dash.len() % 2 == 0 && dash.iter().all(|x| *x > 0.0) {
                        s.dash = dash;
                    }
                    s
                });
                if stroke.is_none() {
                    warn.push("unsupported stroke paint skipped".into());
                }
            }
            "gr" => {
                if let Some(inner) = it.get("it").and_then(Value::as_array) {
                    styled.extend(shapes(inner, depth + 1, warn, tm));
                }
            }
            "tr" => {
                let g = |k: &str, d: Vec<f32>| it.get(k).and_then(stat).filter(|v| v.len() >= d.len().min(1)).unwrap_or(d);
                let (p, a, s, r) = (g("p", vec![0.0, 0.0]), g("a", vec![0.0, 0.0]), g("s", vec![100.0, 100.0]), g("r", vec![0.0]));
                if ["p", "a", "s", "r", "o"].iter().any(|k| it.get(*k).and_then(|v| v.get("a")).and_then(Value::as_i64) == Some(1)) {
                    warn.push("animated group transform ignored (static part kept)".into());
                }
                xf = compose_transform(Vec2::new(p[0], *p.get(1).unwrap_or(&0.0)), r[0], Vec2::new(s[0] / 100.0, *s.get(1).unwrap_or(&100.0) / 100.0), 0.0, Vec2::new(a[0], *a.get(1).unwrap_or(&0.0)));
                group_opacity = it.get("o").and_then(stat).and_then(|v| v.first().copied()).unwrap_or(100.0) / 100.0;
            }
            "" => {}
            other => warn.push(format!("shape item '{other}' skipped")),
        }
    }
    let mut out = Vec::new();
    for (mut g, w, _) in geo {
        g.fill = fill.clone();
        g.stroke = stroke.clone();
        out.push((g, w, PaintAnim { fill: fill_anim.clone(), stroke: stroke_anim.clone() }));
    }
    out.extend(styled);
    for (p, _, anim) in &mut out {
        // the group transform applies to the shapes and to their gradients (points live in the group's space)
        let radial = |pt: Option<&Paint>| matches!(pt, Some(Paint::Radial { .. }));
        anim.through(&xf, radial(p.fill.as_ref()), radial(p.stroke.as_ref().map(|s| &s.paint)));
        if let Some(f) = &mut p.fill {
            transform_paint(f, &xf);
        }
        if let Some(st) = &mut p.stroke {
            transform_paint(&mut st.paint, &xf);
        }
        transform_path(p, &xf);
        for m in &mut p.morphs {
            let mut tmp = VPath::new("m", p.closed, std::mem::take(&mut m.anchors));
            transform_path(&mut tmp, &xf);
            m.anchors = tmp.anchors;
        }
        if group_opacity < 0.999 {
            if let Some(Paint::Solid(c)) = &mut p.fill {
                c.a *= group_opacity;
            }
            if let Some(s) = &mut p.stroke {
                if let Paint::Solid(c) = &mut s.paint {
                    c.a *= group_opacity;
                }
            }
        }
    }
    out
}
