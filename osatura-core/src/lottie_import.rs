// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Lottie (Bodymovin JSON) import, a deliberate subset: shape layers with paths (`sh`, static),
//! rectangles, ellipses, solid and gradient fills (also with animated end points / centre / radius, imported as `grad_dx`/`grad_dy` channels), strokes and groups with static transforms;
//! layer transforms (static or keyframed position / scale / rotation / opacity), blend modes and a
//! solid background layer. Everything else is skipped and reported in `warnings`.
//!
//! Also imported: image layers with embedded PNG / JPEG data URIs, null layers as parents, layer
//! `parent` links (the parent's transform is baked into the child, see `bake_parented`), layer
//! masks (add / subtract / intersect with opacity, inversion and keyed shapes, built from hidden
//! source layers and layer masks) and luma mattes (approximated). Layer effects (`ef`): gaussian
//! blur, drop shadow, tint and fill become per-layer effects (`Layer::effects`, see `layerfx`) on any
//! layer kind, in any composition, with animated numbers as channels (keys and easing as for
//! transforms; a shadow's direction / distance are sampled per frame into x / y offsets). Static
//! tint, fill, tritone and levels on shape layers are baked into the colours instead (exact, also for
//! gradients); tritone and levels are not imported on precomp / image layers. Effect colours cannot
//! be animated (first value); all other effects are reported as warnings.
//!
//! Keyframe easing: hold keys become steps, bezier handles (`o`/`i`) become cubic-bezier easing,
//! keys without handles are linear. Layer in/out ranges become a multiplying opacity step track. Curved position paths (`to`/`ti`) are baked per frame.

use crate::anim::{Ease, Track};
use crate::layerfx::LayerEffect;
use crate::math::{Affine2, Vec2};
use crate::path::{Anchor, VPath};
use crate::project::{compose_transform, Channel, Layer, Mode, Project, Prop};
use crate::style::{Blend, Paint, Rgba, Stop, StrokeStyle};
use serde_json::Value;

const MAX_LAYERS: usize = 2000;
const MAX_DEPTH: usize = 32;
const MAX_FRAMES: f64 = 20_000.0;

pub struct Imported {
    pub project: Project,
    /// Kept for compatibility; always empty now. Blur and drop shadow are per-layer effects (`Layer::effects`).
    pub post_fx: Vec<crate::postfx::Fx>,
    pub warnings: Vec<String>,
}

fn nums(v: &Value) -> Option<Vec<f32>> {
    match v {
        Value::Number(n) => Some(vec![n.as_f64()? as f32]),
        Value::Array(a) => a.iter().map(|x| x.as_f64().map(|f| f as f32)).collect(),
        _ => None,
    }
}

/// Static value of a Lottie property `{a:0, k:…}`.
fn stat(prop: &Value) -> Option<Vec<f32>> {
    if prop.get("a").and_then(Value::as_i64).unwrap_or(0) == 1 {
        return None;
    }
    nums(prop.get("k")?)
}

struct Kf {
    t: f32,
    v: Vec<f32>,
    hold: bool,
    /// Easing of the segment that starts here: `(o.x, o.y, i.x, i.y)` per component.
    bez: Option<[Vec<f32>; 4]>,
    /// The key has spatial tangents (`to`/`ti`): the motion to the next key is a curve.
    spatial: bool,
    to: Vec<f32>,
    ti: Vec<f32>,
}

fn handle_vals(v: Option<&Value>) -> Option<Vec<f32>> {
    match v? {
        Value::Array(a) => a.iter().map(|x| x.as_f64().map(|f| f as f32)).collect::<Option<Vec<_>>>().filter(|v| !v.is_empty()),
        x => x.as_f64().map(|f| vec![f as f32]),
    }
}

fn bezier_of(k: &Value) -> Option<[Vec<f32>; 4]> {
    let (o, i) = (k.get("o")?, k.get("i")?);
    Some([handle_vals(o.get("x"))?, handle_vals(o.get("y"))?, handle_vals(i.get("x"))?, handle_vals(i.get("y"))?])
}

/// Largest keyframe time or value taken from a file (the project format allows +-1e7 everywhere).
const KEY_LIMIT: f32 = 1.0e6;

fn keyframes(prop: &Value, tm: (f32, f32)) -> Option<Vec<Kf>> {
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

fn rgba(c: &[f32], opacity: f32) -> Rgba {
    Rgba { r: *c.first().unwrap_or(&0.0), g: *c.get(1).unwrap_or(&0.0), b: *c.get(2).unwrap_or(&0.0), a: (*c.get(3).unwrap_or(&1.0) * opacity).clamp(0.0, 1.0) }
}

fn corner_or(pos: Vec2, i: Vec2, o: Vec2) -> Anchor {
    let nz = |d: Vec2| d.x.abs() > 1e-4 || d.y.abs() > 1e-4;
    Anchor { pos, handle_in: nz(i).then(|| pos + i), handle_out: nz(o).then(|| pos + o), weights: Vec::new() }
}

fn vec2s(v: &Value) -> Option<Vec<Vec2>> {
    v.as_array()?.iter().map(|p| nums(p).filter(|n| n.len() >= 2).map(|n| Vec2::new(n[0], n[1]))).collect()
}

fn path_from_shape(shape: &Value) -> Option<VPath> {
    let (v, i, o) = (vec2s(shape.get("v")?)?, vec2s(shape.get("i")?)?, vec2s(shape.get("o")?)?);
    if v.len() < 2 || i.len() != v.len() || o.len() != v.len() {
        return None;
    }
    let anchors = (0..v.len()).map(|k| corner_or(v[k], i[k], o[k])).collect();
    Some(VPath::new("path", shape.get("c").and_then(Value::as_bool).unwrap_or(false), anchors))
}

/// One keyframe of an animated `sh` property.
struct ShapeKf {
    t: f32,
    shape: VPath,
    hold: bool,
    bez: Option<[Vec<f32>; 4]>,
}

/// Keyframed shape (`ks.a == 1`): the shapes at each key, or `None` when the property is malformed.
fn shape_keyframes(prop: &Value, tm: (f32, f32)) -> Option<Vec<ShapeKf>> {
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
fn animated_path(kfs: &[ShapeKf]) -> Result<(VPath, Vec<(String, Track)>), String> {
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

fn ellipse(c: Vec2, r: Vec2) -> VPath {
    const K: f32 = 0.552_284_8;
    let a = |dx: f32, dy: f32, ix: f32, iy: f32, ox: f32, oy: f32| corner_or(c + Vec2::new(dx, dy), Vec2::new(ix, iy), Vec2::new(ox, oy));
    VPath::new("ellipse", true, vec![
        a(r.x, 0.0, 0.0, -K * r.y, 0.0, K * r.y),
        a(0.0, r.y, K * r.x, 0.0, -K * r.x, 0.0),
        a(-r.x, 0.0, 0.0, K * r.y, 0.0, -K * r.y),
        a(0.0, -r.y, -K * r.x, 0.0, K * r.x, 0.0),
    ])
}

fn transform_path(p: &mut VPath, m: &Affine2) {
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
struct GradAnim {
    /// `"from"` (linear start / radial centre) or `"to"` (linear end / radial radius)
    part: &'static str,
    keys: Vec<(f32, [f32; 2], Ease, Ease)>,
}

/// Gradient geometry of a `gf` / `gs` item: the paint at its first key and the animation of the handles. The
/// project model animates gradients through the `grad_dx` / `grad_dy` channels of the path (`fill.from`, `fill.to`,
/// `stroke.from`, `stroke.to`): the start point (linear) or centre (radial) as offsets, the end point (linear) as an
/// offset and the radius (radial) as `|end - start|` sampled per frame.
fn gradient(item: &Value, opacity: f32, warn: &mut Vec<String>, tm: (f32, f32)) -> Option<(Paint, Vec<GradAnim>)> {
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
fn transform_paint(p: &mut Paint, xf: &Affine2) {
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
struct PaintAnim {
    fill: Vec<GradAnim>,
    stroke: Vec<GradAnim>,
}

impl PaintAnim {
    /// The same animation seen through the group transform `xf`: handle offsets go through its linear part, the
    /// radius of a radial gradient is scaled by the geometric mean scale.
    fn through(&mut self, xf: &Affine2, fill_radial: bool, stroke_radial: bool) {
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
    fn tracks(&self, fill_radial: bool, stroke_radial: bool, out: &mut Vec<(String, Track)>) {
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
fn path_channel(target_prefix: &str, name: &str, tr: Track) -> Channel {
    match name.strip_prefix('@') {
        Some(rest) => {
            let (handle, axis) = rest.rsplit_once(':').unwrap_or((rest, "x"));
            Channel::keys(&format!("{target_prefix}::{handle}"), if axis == "y" { Prop::GradDy } else { Prop::GradDx }, tr)
        }
        None => Channel::keys(&format!("{target_prefix}::{name}"), Prop::PathMorph, tr),
    }
}

/// A path with the (morph name, weight track) pairs that animate it.
type AnimPath = (VPath, Vec<(String, Track)>, PaintAnim);

fn shapes(items: &[Value], depth: usize, warn: &mut Vec<String>, tm: (f32, f32)) -> Vec<AnimPath> {
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

fn blend_from(bm: i64) -> Blend {
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
fn ease_of(k: &Kf, comp: usize) -> Ease {
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
fn bake_spatial(kfs: &[Kf]) -> (Track, Track) {
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

fn track(kfs: &[Kf], comp: usize, scale: f32) -> Track {
    let keys: Vec<(f32, f32, Ease)> = kfs.iter().map(|k| (k.t, k.v.get(comp).copied().unwrap_or(0.0) * scale, ease_of(k, comp))).collect();
    Track::with_keys(&keys)
}

// ---------------------------------------------------------------------------------------------
// Layer parenting (baked), masks, luma mattes, image assets and effects
// ---------------------------------------------------------------------------------------------

/// A transform property: constant or keyframed (one [`Track`] per component, already in project frames).
enum Val {
    Const(Vec<f32>),
    Keys { tracks: Vec<Track>, times: Vec<f32>, holds: Vec<f32> },
}

impl Val {
    fn at(&self, t: f32, c: usize, default: f32) -> f32 {
        match self {
            Val::Const(v) => v.get(c).copied().unwrap_or(default),
            Val::Keys { tracks, .. } => tracks.get(c).and_then(|tr| tr.eval(t)).unwrap_or(default),
        }
    }
}

fn val_of(prop: &Value, tm: (f32, f32), position: bool) -> Option<Val> {
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
struct LocalXf {
    p: Option<Val>,
    s: Option<Val>,
    r: Option<Val>,
    anchor: Vec2,
}

impl LocalXf {
    fn of(l: &Value, tm: (f32, f32), warn: &mut Vec<String>) -> LocalXf {
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
    fn animated(&self) -> bool {
        [&self.p, &self.s, &self.r].iter().any(|v| matches!(v, Some(Val::Keys { .. })))
    }
    fn matrix(&self, t: f32) -> Affine2 {
        let g = |v: &Option<Val>, c: usize, d: f32| v.as_ref().map_or(d, |v| v.at(t, c, d));
        compose_transform(Vec2::new(g(&self.p, 0, 0.0), g(&self.p, 1, 0.0)), g(&self.r, 0, 0.0), Vec2::new(g(&self.s, 0, 100.0) / 100.0, g(&self.s, 1, 100.0) / 100.0), 0.0, self.anchor)
    }
    fn key_times(&self, out: &mut Vec<f32>) {
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
fn decompose(m: &Affine2) -> (Vec2, f32, Vec2, f32) {
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
fn parent_chain(l: &Value, by_ind: &std::collections::HashMap<i64, &Value>, tm: (f32, f32), warn: &mut Vec<String>) -> Option<Vec<LocalXf>> {
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
fn bake_parented(chain: &[LocalXf], total: f32, path: &str, budget: &mut usize) -> Option<(crate::project::Transform2D, Vec<Channel>)> {
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

struct MPath {
    path: VPath,
    /// Times and easing of every shape key (empty for a static mask path).
    keys: Vec<(f32, Ease)>,
}

/// One factor of the layer's mask coverage: the union of `paths` (or its complement).
struct Stage {
    invert: bool,
    paths: Vec<MPath>,
}

fn white(a: f32) -> Paint {
    Paint::Solid(Rgba { r: 1.0, g: 1.0, b: 1.0, a })
}

fn mask_path(m: &Value, idx: usize, opacity: f32, tm: (f32, f32), nm: &str, warn: &mut Vec<String>) -> Option<MPath> {
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
fn parse_masks(l: &Value, nm: &str, tm: (f32, f32), warn: &mut Vec<String>) -> Vec<Stage> {
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
fn stage_sources(stages: &[Stage], path: &str, chans: &mut Vec<Channel>) -> Vec<Layer> {
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
fn masked_group(name: &str, path: &str, mut core: Layer, stages: &[Stage], chans: &mut Vec<Channel>) -> Layer {
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
fn content_path(path: &str, stages: usize) -> String {
    if stages == 0 {
        return path.to_string();
    }
    let ms: String = (2..=stages).rev().map(|j| format!("m{j}/")).collect();
    format!("{path}/{ms}content")
}

/// Engine paths are addressed by name inside a layer, so duplicates (two `rc` items, say) get a suffix.
fn unique_path_names(paths: &mut [VPath]) {
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

fn luma_of(c: &Rgba) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

fn luma_paint(p: &Paint) -> Paint {
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
fn to_luma_matte(l: &mut Layer) -> bool {
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
type ImageAsset = Result<(Vec<u8>, Option<Vec2>), String>;

fn jpeg_to_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
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

fn parse_image_asset(a: &Value) -> ImageAsset {
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

// ---- effects ---------------------------------------------------------------------------------

fn effect_name(ty: i64) -> Option<&'static str> {
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
enum ColorFx {
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
type FxTracks = Vec<(Prop, Track)>;

enum Eff {
    /// Colour effect. `bake` recolours shape layers directly (exact for static values); `layer` is the
    /// per-layer effect form (Tint and Fill only) with the tracks of animated numbers.
    Color { name: &'static str, bake: ColorFx, layer: Option<(LayerEffect, FxTracks)> },
    /// Blur and drop shadow: always per-layer effects.
    Layer { fx: LayerEffect, tracks: FxTracks },
}

fn luma(c: Rgba) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

impl ColorFx {
    fn apply(&self, c: Rgba) -> Rgba {
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

fn recolor_paint(p: &mut Paint, f: &ColorFx) {
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
struct EP {
    first: Vec<f32>,
    kfs: Option<Vec<Kf>>,
}

fn eff_p(e: &Value, i: usize, tm: (f32, f32)) -> Option<EP> {
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
    fn animated(&self) -> bool {
        self.kfs.is_some()
    }
    fn num(&self) -> f32 {
        self.first[0]
    }
    /// Keys of component 0 scaled by `scale` (None when static).
    fn track(&self, scale: f32) -> Option<Track> {
        self.kfs.as_ref().map(|k| track(k, 0, scale))
    }
    /// Value at project frame `t` (the first value when static).
    fn at(&self, t: f32) -> f32 {
        match &self.track(1.0) {
            Some(tr) => tr.eval(t).unwrap_or(self.first[0]),
            None => self.first[0],
        }
    }
    fn times(&self) -> Vec<f32> {
        self.kfs.iter().flatten().map(|k| k.t).collect()
    }
}

fn eff_col_p(e: &Value, i: usize, tm: (f32, f32), colour_anim: &mut bool) -> Option<Rgba> {
    let p = eff_p(e, i, tm)?;
    *colour_anim |= p.animated();
    (p.first.len() >= 3).then(|| Rgba::new(p.first[0].clamp(0.0, 1.0), p.first[1].clamp(0.0, 1.0), p.first[2].clamp(0.0, 1.0), 1.0))
}

/// Most frames sampled for an animated drop-shadow direction / distance.
const MAX_SHADOW_SAMPLES: f32 = 2000.0;

/// Read a layer's `ef` array. Supported effects become `Eff`s (in order); disabled ones (`en` = 0)
/// and expression controls (types 0..=7) are skipped silently, everything else is reported by name.
fn parse_effects(l: &Value, nm: &str, tm: (f32, f32), warn: &mut Vec<String>) -> Vec<Eff> {
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

struct Cx {
    warnings: Vec<String>,
    easing_noted: bool,
    spatial_noted: bool,
    assets: std::collections::HashMap<String, Vec<Value>>,
    /// Frames of the whole document.
    total: f32,
    count: usize,
    background: Option<Rgba>,
    images: std::collections::HashMap<String, ImageAsset>,
    /// Remaining number of baked keys (parent links) before the importer falls back.
    bake_budget: usize,
}

/// Convert Lottie layers (bottom of the list = first drawn) into layers and channels. `tm` maps
/// the keyframe time of these layers to project frames: `frame = t * tm.0 + tm.1`. `prefix` is the
/// `/`-joined path of the enclosing precomp group, used in channel targets.
fn convert_layers(cx: &mut Cx, layers: &[Value], tm: (f32, f32), prefix: &str, depth: usize, top: bool) -> (Vec<Layer>, Vec<Channel>) {
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
const MAX_VALUES: usize = 4_000_000;

fn count_values(v: &Value) -> usize {
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

#[cfg(test)]
mod tests {
    #[test]
    fn files_with_millions_of_values_are_refused_up_front() {
        let kf = r#"{"t":0,"s":[0]}"#;
        let many = vec![kf; 1_500_000].join(",");
        let doc = format!(r#"{{"w":10,"h":10,"fr":24,"ip":0,"op":10,"layers":[{{"ty":4,"ks":{{"o":{{"a":1,"k":[{many}]}}}}}}]}}"#);
        let err = import_lottie(&doc).err().expect("must be refused");
        assert!(err.contains("more than the limit"), "{err}");
        assert!(import_lottie(r#"{"w":10,"h":10,"fr":24,"ip":0,"op":10,"layers":[]}"#).is_ok());
        let err = import_lottie(r#"{"w":1e30,"h":10,"fr":24,"ip":0,"op":10,"layers":[]}"#).err().expect("1e30 stage");
        assert!(err.contains("supported range"), "{err}");
    }

    use super::*;
    use crate::project::Source;
    use crate::Engine;
    use serde_json::json;

    fn doc() -> String {
        json!({
            "v": "5.7.0", "fr": 24, "ip": 0, "op": 25, "w": 200, "h": 100, "nm": "t", "assets": [],
            "layers": [
                {"ty": 4, "nm": "top", "ind": 1, "ip": 0, "op": 25, "bm": 1,
                 "ks": {"o": {"a": 0, "k": 50}, "p": {"a": 1, "k": [{"t": 0, "s": [10, 20, 0], "h": 0}, {"t": 24, "s": [110, 20, 0]}]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}},
                 "shapes": [{"ty": "gr", "it": [
                    {"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [40, 20]}, "r": {"a": 0, "k": 0}},
                    {"ty": "el", "p": {"a": 0, "k": [60, 20]}, "s": {"a": 0, "k": [20, 20]}},
                    {"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}},
                    {"ty": "st", "c": {"a": 0, "k": [0, 0, 1, 1]}, "o": {"a": 0, "k": 100}, "w": {"a": 0, "k": 3}},
                    {"ty": "tr", "p": {"a": 0, "k": [0, 0]}, "a": {"a": 0, "k": [0, 0]}, "s": {"a": 0, "k": [100, 100]}, "r": {"a": 0, "k": 0}, "o": {"a": 0, "k": 100}}]}]},
                {"ty": 4, "nm": "bottom", "ind": 2, "ks": {},
                 "shapes": [{"ty": "sh", "ks": {"a": 0, "k": {"i": [[0, 0], [0, 0], [0, 0]], "o": [[0, 0], [0, 0], [0, 0]], "v": [[0, 0], [50, 0], [25, 40]], "c": true}}},
                            {"ty": "fl", "c": {"a": 0, "k": [0, 1, 0, 1]}, "o": {"a": 0, "k": 100}}]},
                {"ty": 1, "nm": "bg", "sc": "#112233", "sw": 200, "sh": 100, "ks": {}},
                {"ty": 0, "nm": "precomp", "ks": {}}
            ]})
        .to_string()
    }

    #[test]
    fn imports_shapes_styles_and_animation() {
        let r = import_lottie(&doc()).unwrap();
        let p = &r.project;
        assert_eq!((p.width, p.height, p.fps, p.frames), (200.0, 100.0, 24.0, 25));
        assert_eq!(p.layers.len(), 2);
        assert_eq!(p.layers[0].name, "bottom", "Lottie lists the top layer first; we draw bottom first");
        assert_eq!(p.layers[1].blend, Blend::Multiply);
        assert!((p.layers[1].opacity - 0.5).abs() < 1e-6);
        assert!(p.background.is_some());
        let crate::project::LayerKind::Vector { paths } = &p.layers[1].kind else { panic!() };
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().all(|x| x.fill.is_some() && x.stroke.is_some()));
        assert!(paths[1].anchors.iter().any(|a| a.handle_out.is_some()), "ellipse has curve handles");
        let e = Engine::new(p.clone()).unwrap();
        let x = |f: f32| e.evaluate(f).nodes.iter().find(|n| n.name == "top").unwrap().world.tx;
        assert!((x(0.0) - 10.0).abs() < 1e-3 && (x(24.0) - 110.0).abs() < 1e-3 && (x(12.0) - 60.0).abs() < 1e-3);
        assert!(r.warnings.iter().any(|w| w.contains("precomp layer")), "{:?}", r.warnings);
        assert!(r.warnings.iter().any(|w| w.contains("without easing")));
    }

    #[test]
    fn roundtrips_with_our_own_exporter() {
        let e = Engine::new(crate::demo::project()).unwrap();
        let j = crate::export::lottie_json(&e).to_string();
        let back = import_lottie(&j).unwrap();
        assert_eq!(back.project.frames, e.project.frames);
        assert!(!back.project.layers.is_empty());
        Engine::new(back.project).unwrap();
    }

    /// Export -> import -> render: the frames agree (static paths, animated paths, transforms).
    #[test]
    fn roundtrip_pictures_match_for_the_demo() {
        let e = Engine::new(crate::demo::project()).unwrap();
        let j = crate::export::lottie_json(&e).to_string();
        let back = import_lottie(&j).unwrap();
        assert!(!back.warnings.iter().any(|w| w.contains("path")), "{:?}", back.warnings);
        let re = Engine::new(back.project).unwrap();
        for f in [0.0, 6.0, 13.0, 25.0] {
            let (a, b) = (render_img(&e, f), render_img(&re, f));
            let ssim = crate::percep::ssim(&a, &b).unwrap();
            eprintln!("demo frame {f}: ssim {ssim}");
            assert!(ssim > 0.97, "frame {f}: ssim {ssim}");
        }
    }

    /// A path that morphs (point offsets) and a moving layer survive export -> import.
    #[test]
    fn roundtrip_animated_path_and_transform() {
        use crate::project::Channel;
        let mut p = Project::new("ap", 80.0, 60.0, 12.0, 13);
        p.background = None;
        let tri = crate::path::VPath::new("tri", true, vec![
            crate::path::Anchor::corner(Vec2::new(10.0, 40.0)),
            crate::path::Anchor::corner(Vec2::new(30.0, 40.0)),
            crate::path::Anchor::corner(Vec2::new(20.0, 10.0)),
        ]).with_fill("#cc2244");
        let still = Layer::vector("still", vec![crate::path::VPath::rect("r", 50.0, 5.0, 20.0, 10.0).with_fill("#2255cc")]);
        p.layers = vec![Layer::vector("tri", vec![tri]), still];
        p.channels = vec![
            Channel::keys("tri::tri::2", Prop::PointDy, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, -8.0, Ease::Linear)])),
            Channel::keys("tri::tri::2", Prop::PointDx, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, 15.0, Ease::Linear)])),
            Channel::keys("still", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, 10.0, Ease::Linear)])),
        ];
        let e = Engine::new(p).unwrap();
        let j = crate::export::lottie_json(&e).to_string();
        let back = import_lottie(&j).unwrap();
        assert!(back.warnings.iter().all(|w| !w.contains("skipped")), "{:?}", back.warnings);
        let re = Engine::new(back.project).unwrap();
        for f in [0.0, 3.0, 6.0, 12.0] {
            let (a, b) = (render_img(&e, f), render_img(&re, f));
            let mse = crate::percep::mse(&a, &b).unwrap();
            assert!(mse < 12.0, "frame {f}: mse {mse}");
        }
        // the triangle's tip really moved in the re-imported file
        let (a0, a12) = (render_img(&re, 0.0), render_img(&re, 12.0));
        assert!(crate::percep::mse(&a0, &a12).unwrap() > 50.0);
    }

    /// Keyframes that change the number of points cannot be morphed: first shape + a warning.
    #[test]
    fn path_keys_with_other_topology_warn_and_use_the_first_shape() {
        let sh = |pts: &[[f64; 2]]| json!({"v": pts, "i": pts.iter().map(|_| [0, 0]).collect::<Vec<_>>(), "o": pts.iter().map(|_| [0, 0]).collect::<Vec<_>>(), "c": true});
        let ks = json!({"a": 1, "k": [
            {"t": 0, "s": [sh(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]])], "h": 1},
            {"t": 5, "s": [sh(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])], "h": 1}]});
        let shapes = json!([{"ty": "sh", "ks": ks}, {"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}}]);
        let r = run(lay("a", 1, stat_ks([0.0, 0.0]), shapes, json!({})));
        assert!(r.warnings.iter().any(|w| w.contains("number of points")), "{:?}", r.warnings);
        match &r.project.layers[0].kind {
            crate::project::LayerKind::Vector { paths } => assert_eq!(paths[0].anchors.len(), 3),
            _ => panic!(),
        }
    }

    /// Eased keys and holds carry over to the morph weights.
    #[test]
    fn animated_path_keys_keep_hold_and_interpolate_linearly() {
        let sq = |x: f64| json!({"v": [[x, 0.0], [x + 10.0, 0.0], [x + 10.0, 10.0], [x, 10.0]], "i": [[0,0],[0,0],[0,0],[0,0]], "o": [[0,0],[0,0],[0,0],[0,0]], "c": true});
        let ks = json!({"a": 1, "k": [
            {"t": 0, "s": [sq(0.0)], "i": {"x": [1], "y": [1]}, "o": {"x": [0], "y": [0]}},
            {"t": 10, "s": [sq(20.0)], "h": 1},
            {"t": 20, "s": [sq(20.0)]}]});
        let shapes = json!([{"ty": "sh", "ks": ks}, {"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}}]);
        let r = run(lay("a", 1, stat_ks([0.0, 0.0]), shapes, json!({})));
        let e = Engine::new(r.project).unwrap();
        let x0 = |f: f32| { let fr = e.evaluate(f); fr.nodes.iter().find_map(|n| n.paths.first().map(|p| p.anchors[0].pos.x)).unwrap() };
        assert!((x0(0.0) - 0.0).abs() < 1e-3 && (x0(5.0) - 10.0).abs() < 1e-3 && (x0(10.0) - 20.0).abs() < 1e-3, "{} {} {}", x0(0.0), x0(5.0), x0(10.0));
        assert!((x0(15.0) - 20.0).abs() < 1e-3);
    }

    #[test]
    fn rejects_garbage_and_limits() {
        assert!(import_lottie("not json").is_err());
        assert!(import_lottie("{}").is_err());
        assert!(import_lottie(&json!({"fr": 24, "ip": 0, "op": 10, "w": 0, "h": 10, "layers": []}).to_string()).is_err());
        assert!(import_lottie(&json!({"fr": 24, "ip": 0, "op": 1e9, "w": 10, "h": 10, "layers": []}).to_string()).is_err());
        let r = import_lottie(&json!({"fr": 24, "ip": 0, "op": 10, "w": 10, "h": 10, "layers": [{"ty": 4, "nm": "a", "ks": {}, "shapes": [{"ty": "sh", "ks": {"a": 0, "k": {"v": "x"}}}]}]}).to_string()).unwrap();
        assert!(r.warnings.iter().any(|w| w.contains("malformed")));
    }

    #[test]
    fn bezier_handles_become_easing() {
        let d = json!({
            "v": "5.7.0", "fr": 24, "ip": 0, "op": 25, "w": 100, "h": 100, "nm": "e", "assets": [],
            "layers": [{"ty": 4, "nm": "l", "ind": 1, "ip": 0, "op": 25,
                "ks": {"o": {"a": 1, "k": [
                    {"t": 0, "s": [0], "o": {"x": [0.9], "y": [0.0]}, "i": {"x": [0.1], "y": [1.0]}},
                    {"t": 24, "s": [100]}]}, "p": {"a": 0, "k": [0, 0, 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}},
                "shapes": [{"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}]}]
        });
        let imp = import_lottie(&d.to_string()).unwrap();
        assert!(!imp.warnings.iter().any(|w| w.contains("without easing")), "{:?}", imp.warnings);
        let ch = imp.project.channels.iter().find(|c| c.prop == Prop::LayerOpacity).unwrap();
        let Source::Keys { track } = &ch.source else { panic!() };
        assert!(matches!(track.keys[0].ease, Ease::CubicBezier { .. }));
        let mid = track.eval(6.0).unwrap();
        assert!(mid < 0.2, "strong ease-in: slow start, got {mid}");
        // diagonal handles are plain linear
        let lin = d.to_string().replace("\"x\":[0.9]", "\"x\":[0.3]").replace("\"y\":[0.0]", "\"y\":[0.3]").replace("\"x\":[0.1]", "\"x\":[0.7]").replace("\"y\":[1.0]", "\"y\":[0.7]");
        let imp = import_lottie(&lin).unwrap();
        let ch = imp.project.channels.iter().find(|c| c.prop == Prop::LayerOpacity).unwrap();
        let Source::Keys { track } = &ch.source else { panic!() };
        assert_eq!(track.keys[0].ease, Ease::Linear);
    }

    #[test]
    fn curved_position_path_is_baked() {
        let d = json!({
            "v": "5.7.0", "fr": 24, "ip": 0, "op": 25, "w": 200, "h": 100, "nm": "c", "assets": [],
            "layers": [{"ty": 4, "nm": "l", "ind": 1, "ip": 0, "op": 25,
                "ks": {"o": {"a": 0, "k": 100}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0},
                    "p": {"a": 1, "k": [
                        {"t": 0, "s": [0, 0, 0], "to": [0, 50, 0], "ti": [0, 50, 0], "o": {"x": [0.3], "y": [0.3]}, "i": {"x": [0.7], "y": [0.7]}},
                        {"t": 24, "s": [100, 0, 0]}]}},
                "shapes": [{"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}]}]
        });
        let imp = import_lottie(&d.to_string()).unwrap();
        assert!(!imp.warnings.iter().any(|w| w.contains("motion paths")), "{:?}", imp.warnings);
        let get = |prop: Prop| {
            let ch = imp.project.channels.iter().find(|c| c.prop == prop).unwrap();
            let Source::Keys { track } = &ch.source else { panic!() };
            track.clone()
        };
        let (x, y) = (get(Prop::LayerX), get(Prop::LayerY));
        assert!((x.eval(12.0).unwrap() - 50.0).abs() < 0.5 && (y.eval(12.0).unwrap() - 37.5).abs() < 0.5, "{:?} {:?}", x.eval(12.0), y.eval(12.0));
        assert!(y.eval(0.0).unwrap().abs() < 1e-3 && y.eval(24.0).unwrap().abs() < 1e-3 && (x.eval(24.0).unwrap() - 100.0).abs() < 1e-3);
        assert_eq!(x.keys.len(), 25, "one key per frame, no duplicates");
    }

    #[test]
    fn layer_in_out_range_hides_outside() {
        let d = json!({
            "v": "5.7.0", "fr": 24, "ip": 0, "op": 30, "w": 100, "h": 100, "nm": "r", "assets": [],
            "layers": [{"ty": 4, "nm": "l", "ind": 1, "ip": 10, "op": 20,
                "ks": {"o": {"a": 0, "k": 100}, "p": {"a": 0, "k": [0, 0, 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}},
                "shapes": [{"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}]}]
        });
        let imp = import_lottie(&d.to_string()).unwrap();
        assert!(!imp.warnings.iter().any(|w| w.contains("in/out")), "{:?}", imp.warnings);
        let e = Engine::new(imp.project).unwrap();
        let op = |f: f32| e.evaluate(f).nodes.iter().find(|n| n.name == "l").map(|n| n.opacity).unwrap_or(0.0);
        assert_eq!((op(5.0), op(10.0), op(19.0), op(20.0), op(29.0)), (0.0, 1.0, 1.0, 0.0, 0.0));
    }

    #[test]
    fn precomp_becomes_a_group_with_time_mapping() {
        let d = json!({
            "v": "5.7.0", "fr": 24, "ip": 0, "op": 50, "w": 200, "h": 100, "nm": "pc",
            "assets": [{"id": "comp_0", "layers": [
                {"ty": 4, "nm": "dot", "ind": 1, "ip": 0, "op": 50,
                 "ks": {"o": {"a": 0, "k": 100}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0},
                        "p": {"a": 1, "k": [{"t": 0, "s": [0, 0, 0]}, {"t": 10, "s": [100, 0, 0]}]}},
                 "shapes": [{"ty": "rc", "p": {"a": 0, "k": [0, 0]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}]}]}],
            "layers": [{"ty": 0, "nm": "pre", "refId": "comp_0", "ind": 1, "ip": 0, "op": 50, "st": 20, "sr": 2,
                "ks": {"o": {"a": 0, "k": 100}, "p": {"a": 0, "k": [5, 7, 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}}}]
        });
        let imp = import_lottie(&d.to_string()).unwrap();
        assert!(!imp.warnings.iter().any(|w| w.contains("not imported") || w.contains("skipped")), "{:?}", imp.warnings);
        let e = Engine::new(imp.project.clone()).unwrap();
        let x = |f: f32| e.evaluate(f).nodes.iter().find(|n| n.name == "pre/dot").unwrap().world.tx;
        // comp time 0..10 plays over project frames 20..40 (start 20, stretch 2), plus the group offset of 5
        assert!((x(10.0) - 5.0).abs() < 1e-3, "before the start: first value");
        assert!((x(30.0) - 55.0).abs() < 1e-3, "halfway through: {}", x(30.0));
        assert!((x(45.0) - 105.0).abs() < 1e-3, "after the end: last value");
        assert!(imp.project.channels.iter().filter(|c| c.target == "pre/dot").count() > 0);
    }

    #[test]
    fn alpha_mattes_become_layer_masks() {
        let rect = |x: f64| json!([{"ty": "rc", "p": {"a": 0, "k": [x, 20]}, "s": {"a": 0, "k": [20, 20]}, "r": {"a": 0, "k": 0}}, {"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}}]);
        let d = json!({
            "v": "5.7.0", "fr": 24, "ip": 0, "op": 5, "w": 100, "h": 100, "nm": "m", "assets": [],
            "layers": [
                {"ty": 4, "nm": "matte", "ind": 1, "td": 1, "ks": {}, "shapes": rect(20.0)},
                {"ty": 4, "nm": "art", "ind": 2, "tt": 2, "ks": {}, "shapes": rect(20.0)}]
        });
        let imp = import_lottie(&d.to_string()).unwrap();
        assert!(!imp.warnings.iter().any(|w| w.contains("matte") || w.contains("'tt'") || w.contains("'td'")), "{:?}", imp.warnings);
        let art = imp.project.layers.iter().find(|l| l.name == "art").unwrap();
        let m = art.mask.as_ref().unwrap();
        assert_eq!((m.layer.as_str(), m.invert), ("matte", true));
        assert!(!imp.project.layers.iter().find(|l| l.name == "matte").unwrap().visible);
        Engine::new(imp.project).unwrap();
        // round trip through our own export: the matte pair comes back as one masked layer
        let mut p = Project::new("rt", 100.0, 100.0, 24.0, 2);
        let mut a = Layer::vector("art", vec![crate::path::VPath::rect("r", 0.0, 0.0, 30.0, 30.0).with_fill("#f00")]);
        a.mask = Some(crate::project::MaskRef { layer: "hole".into(), invert: false });
        let mut h = Layer::vector("hole", vec![crate::path::VPath::rect("r", 5.0, 5.0, 10.0, 10.0).with_fill("#fff")]);
        h.visible = false;
        p.layers = vec![h, a];
        let lot = crate::export::lottie_json(&Engine::new(p).unwrap()).to_string();
        let back = import_lottie(&lot).unwrap();
        assert!(back.project.layers.iter().any(|l| l.mask.is_some()), "{:?}", back.warnings);
    }

    // ---- parenting, masks, luma mattes, images, effects ----

    fn base_doc(layers: Value, assets: Value) -> String {
        json!({"v": "5.7.0", "fr": 10, "ip": 0, "op": 11, "w": 100, "h": 100, "nm": "x", "assets": assets, "layers": layers}).to_string()
    }

    fn rect_shapes(x: f64, y: f64, w: f64, h: f64, rgb: [f64; 3]) -> Value {
        json!([{"ty": "rc", "p": {"a": 0, "k": [x + w / 2.0, y + h / 2.0]}, "s": {"a": 0, "k": [w, h]}, "r": {"a": 0, "k": 0}},
               {"ty": "fl", "c": {"a": 0, "k": [rgb[0], rgb[1], rgb[2], 1]}, "o": {"a": 0, "k": 100}}])
    }

    fn lay(nm: &str, ind: i64, ks: Value, shapes: Value, extra: Value) -> Value {
        let mut base = json!({"ty": 4, "nm": nm, "ind": ind, "ip": 0, "op": 11, "ks": ks, "shapes": shapes});
        for (k, v) in extra.as_object().unwrap() {
            base[k] = v.clone();
        }
        base
    }

    fn stat_ks(p: [f64; 2]) -> Value {
        json!({"o": {"a": 0, "k": 100}, "p": {"a": 0, "k": [p[0], p[1], 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}, "r": {"a": 0, "k": 0}})
    }

    fn rect_shape_json(x0: f64, y0: f64, x1: f64, y1: f64) -> Value {
        json!({"i": [[0, 0], [0, 0], [0, 0], [0, 0]], "o": [[0, 0], [0, 0], [0, 0], [0, 0]], "v": [[x0, y0], [x1, y0], [x1, y1], [x0, y1]], "c": true})
    }

    fn mask(mode: &str, inv: bool, op: f64, x0: f64, y0: f64, x1: f64, y1: f64) -> Value {
        json!({"mode": mode, "inv": inv, "nm": "m", "o": {"a": 0, "k": op}, "x": {"a": 0, "k": 0}, "pt": {"a": 0, "k": rect_shape_json(x0, y0, x1, y1)}})
    }

    fn alpha_at(e: &Engine, frame: f32, x: usize, y: usize) -> u8 {
        let (w, _, d) = crate::raster::render_rgba(e, frame, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: true });
        d[(y * w as usize + x) * 4 + 3]
    }

    fn masked(masks: Value) -> Engine {
        let l = lay("art", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({"masksProperties": masks}));
        let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        Engine::new(imp.project).unwrap()
    }

    #[test]
    fn static_parent_is_baked_and_opacity_is_not_inherited() {
        let mut ks_null = stat_ks([10.0, 5.0]);
        ks_null["s"] = json!({"a": 0, "k": [200, 200, 100]});
        ks_null["o"] = json!({"a": 0, "k": 10});
        let nl = json!({"ty": 3, "nm": "ctl", "ind": 1, "ip": 0, "op": 11, "ks": ks_null});
        let mut ks = stat_ks([3.0, 4.0]);
        ks["o"] = json!({"a": 0, "k": 50});
        let child = lay("child", 2, ks, rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 1}));
        let imp = import_lottie(&base_doc(json!([child, nl]), json!([]))).unwrap();
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        assert_eq!(imp.project.layers.len(), 1, "the null layer is not a drawn layer");
        assert!(imp.project.channels.is_empty(), "static chain: no channels");
        let e = Engine::new(imp.project).unwrap();
        let f = e.evaluate(0.0);
        let n = f.nodes.iter().find(|n| n.name == "child").unwrap();
        assert!((n.world.tx - 16.0).abs() < 1e-4 && (n.world.ty - 13.0).abs() < 1e-4 && (n.world.a - 2.0).abs() < 1e-4);
        assert!((n.opacity - 0.5).abs() < 1e-5, "Lottie parenting inherits the transform only");
    }

    #[test]
    fn animated_parent_chain_matches_the_reference_matrix() {
        // grandparent rotates, parent has non-uniform scale and is itself a drawn layer, child is rotated about an anchor
        let rot = json!({"a": 1, "k": [{"t": 0, "s": [0], "o": {"x": [0.2], "y": [0.0]}, "i": {"x": [0.8], "y": [1.0]}}, {"t": 10, "s": [90]}]});
        let mut gks = stat_ks([50.0, 50.0]);
        gks["r"] = rot.clone();
        let mut pks = stat_ks([20.0, 0.0]);
        pks["s"] = json!({"a": 0, "k": [100, 50, 100]});
        let mut cks = stat_ks([5.0, 7.0]);
        cks["r"] = json!({"a": 0, "k": 30});
        cks["a"] = json!({"a": 0, "k": [2, 1, 0]});
        let g = json!({"ty": 3, "nm": "g", "ind": 1, "ip": 0, "op": 11, "ks": gks});
        let p = lay("p", 2, pks, rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 1.0, 0.0]), json!({"parent": 1}));
        let c = lay("c", 3, cks, rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 2}));
        let imp = import_lottie(&base_doc(json!([c, p, g]), json!([]))).unwrap();
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        assert_eq!(imp.project.layers.len(), 2, "draw order c above p is kept");
        assert_eq!(imp.project.layers[1].name, "c");
        let e = Engine::new(imp.project).unwrap();
        // reference: ease through the same cubic bezier
        let ease = Ease::CubicBezier { x1: 0.2, y1: 0.0, x2: 0.8, y2: 1.0 };
        for f in [0.0f32, 1.0, 4.0, 5.0, 9.0, 10.0] {
            let angle = 90.0 * ease.apply(f / 10.0);
            let m = compose_transform(Vec2::new(50.0, 50.0), angle, Vec2::new(1.0, 1.0), 0.0, Vec2::ZERO)
                .mul(compose_transform(Vec2::new(20.0, 0.0), 0.0, Vec2::new(1.0, 0.5), 0.0, Vec2::ZERO))
                .mul(compose_transform(Vec2::new(5.0, 7.0), 30.0, Vec2::new(1.0, 1.0), 0.0, Vec2::new(2.0, 1.0)));
            let w = e.evaluate(f).nodes.iter().find(|n| n.name == "c").unwrap().world;
            for (got, want) in [(w.a, m.a), (w.b, m.b), (w.c, m.c), (w.d, m.d), (w.tx, m.tx), (w.ty, m.ty)] {
                assert!((got - want).abs() < 2e-3, "frame {f}: {w:?} vs {m:?}");
            }
        }
    }

    #[test]
    fn parent_links_that_cannot_be_followed_are_reported() {
        let a = lay("a", 1, stat_ks([1.0, 2.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 2}));
        let b = lay("b", 2, stat_ks([3.0, 4.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 1}));
        let lost = lay("lost", 3, stat_ks([3.0, 4.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"parent": 99}));
        let imp = import_lottie(&base_doc(json!([a, b, lost]), json!([]))).unwrap();
        assert_eq!(imp.project.layers.len(), 3);
        assert!(imp.warnings.iter().any(|w| w.contains("cycle")), "{:?}", imp.warnings);
        assert!(imp.warnings.iter().any(|w| w.contains("parent layer 99 not found")));
        Engine::new(imp.project).unwrap();
    }

    #[test]
    fn mask_modes_add_subtract_intersect_invert_opacity() {
        let e = masked(json!([mask("a", false, 100.0, 10.0, 10.0, 30.0, 30.0)]));
        assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (255, 0), "add");
        let e = masked(json!([mask("s", false, 100.0, 10.0, 10.0, 30.0, 30.0)]));
        assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (0, 255), "subtract");
        let e = masked(json!([mask("a", true, 100.0, 10.0, 10.0, 30.0, 30.0)]));
        assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (0, 255), "inverted add");
        let e = masked(json!([mask("a", false, 100.0, 0.0, 0.0, 30.0, 30.0), mask("i", false, 100.0, 10.0, 10.0, 40.0, 40.0)]));
        assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 5, 5), alpha_at(&e, 0.0, 35, 35)), (255, 0, 0), "intersect");
        let e = masked(json!([mask("a", false, 100.0, 0.0, 0.0, 40.0, 40.0), mask("s", false, 100.0, 10.0, 10.0, 30.0, 30.0)]));
        assert_eq!((alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (0, 255), "ring: add then subtract");
        let e = masked(json!([mask("a", false, 100.0, 0.0, 0.0, 20.0, 40.0), mask("a", false, 100.0, 20.0, 0.0, 40.0, 40.0)]));
        assert_eq!((alpha_at(&e, 0.0, 10, 20), alpha_at(&e, 0.0, 30, 20)), (255, 255), "two adds are a union");
        let e = masked(json!([mask("a", false, 50.0, 10.0, 10.0, 30.0, 30.0)]));
        let a = alpha_at(&e, 0.0, 20, 20);
        assert!((a as i32 - 128).abs() <= 2, "mask opacity: {a}");
        let e = masked(json!([mask("n", false, 100.0, 10.0, 10.0, 30.0, 30.0)]));
        assert_eq!(alpha_at(&e, 0.0, 3, 3), 255, "mode none draws nothing");
    }

    #[test]
    fn keyed_mask_path_morphs_and_follows_the_layer() {
        let keyed = json!({"mode": "a", "inv": false, "nm": "k", "o": {"a": 0, "k": 100}, "x": {"a": 0, "k": 0},
            "pt": {"a": 1, "k": [{"t": 0, "s": [rect_shape_json(10.0, 10.0, 30.0, 30.0)]}, {"t": 10, "s": [rect_shape_json(0.0, 0.0, 40.0, 40.0)]}]}});
        let e = masked(json!([keyed.clone()]));
        assert_eq!((alpha_at(&e, 0.0, 7, 20), alpha_at(&e, 5.0, 7, 20), alpha_at(&e, 5.0, 3, 20), alpha_at(&e, 10.0, 2, 20)), (0, 255, 0, 255));
        // the layer is moved: masks live in layer space
        let l = lay("art", 1, stat_ks([20.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({"masksProperties": [mask("a", false, 100.0, 10.0, 10.0, 30.0, 30.0)]}));
        let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
        let e = Engine::new(imp.project).unwrap();
        assert_eq!((alpha_at(&e, 0.0, 35, 20), alpha_at(&e, 0.0, 15, 20)), (255, 0));
    }

    #[test]
    fn masks_work_on_precomps_and_with_mattes() {
        let inner = lay("dot", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({}));
        let pre = json!({"ty": 0, "nm": "pre", "refId": "c0", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]),
            "masksProperties": [mask("a", false, 100.0, 10.0, 10.0, 30.0, 30.0), mask("s", false, 100.0, 18.0, 18.0, 22.0, 22.0)]});
        let imp = import_lottie(&base_doc(json!([pre]), json!([{"id": "c0", "layers": [inner]}]))).unwrap();
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        let e = Engine::new(imp.project).unwrap();
        assert_eq!((alpha_at(&e, 0.0, 12, 12), alpha_at(&e, 0.0, 20, 20), alpha_at(&e, 0.0, 3, 3)), (255, 0, 0));
        // a masked layer as matte source: masks are dropped with a warning
        let src = lay("src", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 1.0, 1.0]), json!({"td": 1, "masksProperties": [mask("a", false, 100.0, 10.0, 10.0, 30.0, 30.0)]}));
        let art = lay("art", 2, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({"tt": 1}));
        let imp = import_lottie(&base_doc(json!([src, art]), json!([]))).unwrap();
        assert!(imp.warnings.iter().any(|w| w.contains("track-matte source")), "{:?}", imp.warnings);
        Engine::new(imp.project).unwrap();
    }

    #[test]
    fn luma_mattes_use_luminance() {
        let shapes = json!([{"ty": "gr", "it": rect_shapes(0.0, 0.0, 20.0, 40.0, [1.0, 1.0, 1.0])}, {"ty": "gr", "it": rect_shapes(20.0, 0.0, 20.0, 40.0, [0.2, 0.2, 0.2])}]);
        let build = |tt: i64| {
            let src = lay("src", 1, stat_ks([0.0, 0.0]), shapes.clone(), json!({"td": 1}));
            let art = lay("art", 2, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 40.0, 40.0, [1.0, 0.0, 0.0]), json!({"tt": tt}));
            let imp = import_lottie(&base_doc(json!([src, art]), json!([]))).unwrap();
            assert!(imp.warnings.iter().any(|w| w.contains("luma matte approximated")), "{:?}", imp.warnings);
            assert!(!imp.warnings.iter().any(|w| w.contains("not imported")));
            Engine::new(imp.project).unwrap()
        };
        let e = build(3);
        let (a, b) = (alpha_at(&e, 0.0, 10, 20), alpha_at(&e, 0.0, 30, 20));
        assert!(a >= 254 && (b as i32 - 51).abs() <= 2, "{a} {b}");
        let e = build(4);
        let (a, b, outside) = (alpha_at(&e, 0.0, 10, 20), alpha_at(&e, 0.0, 30, 20), alpha_at(&e, 0.0, 60, 20));
        assert!(a <= 1 && (b as i32 - 204).abs() <= 2 && outside == 0, "{a} {b} {outside}");
    }

    fn png_uri(rgba: [u8; 4]) -> String {
        let px: Vec<u8> = (0..16).flat_map(|_| rgba).collect();
        format!("data:image/png;base64,{}", crate::base64::encode(&crate::open_formats::encode_rgba_png(4, 4, &px).unwrap()))
    }

    #[test]
    fn embedded_images_become_image_layers() {
        let asset = |id: &str, p: String| json!({"id": id, "w": 20, "h": 20, "u": "", "p": p, "e": 1});
        let il = |nm: &str, r: &str| json!({"ty": 2, "nm": nm, "refId": r, "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([10.0, 10.0])});
        let imp = import_lottie(&base_doc(json!([il("pic", "i0")]), json!([asset("i0", png_uri([0, 0, 255, 255]))]))).unwrap();
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        let crate::project::LayerKind::Image { size, .. } = &imp.project.layers[0].kind else { panic!() };
        assert_eq!(*size, Some(Vec2::new(20.0, 20.0)), "asset w/h is the display size");
        let e = Engine::new(imp.project).unwrap();
        assert_eq!((alpha_at(&e, 0.0, 15, 15), alpha_at(&e, 0.0, 5, 5)), (255, 0));
        // JPEG is re-encoded to PNG
        let jpg = "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAIBAQEBAQIBAQECAgICAgQDAgICAgUEBAMEBgUGBgYFBgYGBwkIBgcJBwYGCAsICQoKCgoKBggLDAsKDAkKCgr/2wBDAQICAgICAgUDAwUKBwYHCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgr/wAARCAAIAAgDASIAAhEBAxEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwD5Hooor+Xz/eA//9k=";
        let imp = import_lottie(&base_doc(json!([il("jpg", "j")]), json!([asset("j", format!("data:image/jpeg;base64,{jpg}"))]))).unwrap();
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        let e = Engine::new(imp.project).unwrap();
        let (w, _, d) = crate::raster::render_rgba(&e, 0.0, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: true });
        let o = (15 * w as usize + 15) * 4;
        assert!(d[o] > 200 && d[o + 1] < 60 && d[o + 3] == 255, "{:?}", &d[o..o + 4]);
    }

    #[test]
    fn unusable_images_are_skipped_with_a_reason() {
        let il = |nm: &str, r: &str| json!({"ty": 2, "nm": nm, "refId": r, "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0])});
        let assets = json!([{"id": "ext", "w": 10, "h": 10, "u": "images/", "p": "a.png", "e": 0},
            {"id": "bad", "w": 10, "h": 10, "u": "", "p": "data:image/png;base64,@@@@", "e": 1},
            {"id": "gif", "w": 10, "h": 10, "u": "", "p": "data:image/gif;base64,R0lGOA==", "e": 1}]);
        let imp = import_lottie(&base_doc(json!([il("a", "ext"), il("b", "bad"), il("c", "gif"), il("d", "none")]), assets)).unwrap();
        assert!(imp.project.layers.is_empty());
        let w = imp.warnings.join("\n");
        assert!(w.contains("external file 'images/a.png'") && w.contains("invalid base64") && w.contains("image/gif") && w.contains("asset not found"), "{w}");
    }

    #[test]
    fn effects_are_reported_by_name_and_controls_are_not() {
        let ef = json!([{"ty": 29, "nm": "Gaussian Blur", "en": 1}, {"ty": 25, "en": 1}, {"ty": 0, "nm": "Slider Control"}, {"ty": 21, "en": 0}]);
        let l = lay("fx", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"ef": ef}));
        let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
        let fx: Vec<&String> = imp.warnings.iter().filter(|w| w.contains("effect")).collect();
        assert_eq!(fx.len(), 2, "{:?}", imp.warnings);
        assert!(fx.iter().any(|w| w.contains("Gaussian Blur")) && fx.iter().any(|w| w.contains("Drop Shadow")));
        assert_eq!(imp.project.layers.len(), 1);
    }

    // ---- layer effects ----
    fn efp(ty: i64, v: Value) -> Value {
        json!({"ty": ty, "nm": "p", "v": {"a": 0, "k": v}})
    }
    fn effect(ty: i64, params: Vec<Value>) -> Value {
        json!({"ty": ty, "nm": "e", "en": 1, "ef": params})
    }
    fn fx_layer(shapes: Value, ef: Vec<Value>) -> Value {
        lay("fx", 1, stat_ks([0.0, 0.0]), shapes, json!({"ef": ef}))
    }
    fn first_fill(imp: &Imported) -> Rgba {
        let crate::project::LayerKind::Vector { paths } = &imp.project.layers[0].kind else { panic!() };
        let Some(Paint::Solid(c)) = &paths[0].fill else { panic!() };
        *c
    }
    fn close(a: Rgba, r: f32, g: f32, b: f32) {
        assert!((a.r - r).abs() < 1e-3 && (a.g - g).abs() < 1e-3 && (a.b - b).abs() < 1e-3, "{a:?} vs {r} {g} {b}");
    }
    fn run(l: Value) -> Imported {
        import_lottie(&base_doc(json!([l]), json!([]))).unwrap()
    }

    #[test]
    fn fill_effect_replaces_colour() {
        let e = effect(21, vec![efp(7, json!(0)), efp(7, json!(0)), efp(2, json!([0, 1, 0])), efp(7, json!(0)), efp(0, json!(0)), efp(0, json!(0)), efp(0, json!(1))]);
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![e.clone()]));
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        close(first_fill(&imp), 0.0, 1.0, 0.0);
        // half opacity (0..1 scale and 0..100 scale agree)
        for half in [0.5, 50.0] {
            let mut e2 = e.clone();
            e2["ef"][6] = efp(0, json!(half));
            close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![e2]))), 0.5, 0.5, 0.0);
        }
        // the engine renders the new colour
        let e = Engine::new(imp.project).unwrap();
        let (_, _, d) = crate::raster::render_rgba(&e, 0.0, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: true });
        assert!(d[1] > 250 && d[0] < 5 || d.chunks(4).any(|p| p[1] > 250 && p[0] < 5 && p[3] == 255));
    }

    #[test]
    fn tint_maps_luminance_between_two_colours() {
        let e = effect(20, vec![efp(2, json!([0, 0, 1])), efp(2, json!([1, 1, 0])), efp(0, json!(100))]);
        // white stays on the "white" colour, black on the "black" colour
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 1.0, 1.0]), vec![e.clone()]))), 1.0, 1.0, 0.0);
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 0.0, 0.0]), vec![e.clone()]))), 0.0, 0.0, 1.0);
        // amount 0 leaves the colour alone; 50 mixes halfway
        let mut e0 = e.clone();
        e0["ef"][2] = efp(0, json!(0));
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![e0]))), 1.0, 0.0, 0.0);
        let mut eh = e;
        eh["ef"][2] = efp(0, json!(50));
        let l = luma(Rgba::new(1.0, 0.0, 0.0, 1.0));
        let t = Rgba::new(0.0, 0.0, 1.0, 1.0).lerp(Rgba::new(1.0, 1.0, 0.0, 1.0), l);
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![eh]))), 0.5 + 0.5 * t.r, 0.5 * t.g, 0.5 * t.b);
    }

    #[test]
    fn tritone_and_levels() {
        let tri = effect(23, vec![efp(2, json!([1, 1, 1])), efp(2, json!([1, 0, 0])), efp(2, json!([0, 0, 0])), efp(0, json!(0))]);
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 1.0, 1.0]), vec![tri.clone()]))), 1.0, 1.0, 1.0);
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 0.0, 0.0]), vec![tri.clone()]))), 0.0, 0.0, 0.0);
        let g = 0.5 * 0.7152 / 0.5; // luma of (0,0.5/.7152..) == 0.5 -> midtone colour
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 0.5 / 0.7152, 0.0]), vec![tri.clone()]))), 1.0, 0.0, 0.0);
        let _ = g;
        // blend 100 keeps the original
        let mut keep = tri;
        keep["ef"][3] = efp(0, json!(100));
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.2, 0.4, 0.6]), vec![keep]))), 0.2, 0.4, 0.6);
        // levels: input 51..204 of 255 stretches 0.2..0.8 to 0..1
        let lv = effect(24, vec![efp(7, json!(1)), efp(0, json!(51)), efp(0, json!(204)), efp(0, json!(1)), efp(0, json!(0)), efp(0, json!(255))]);
        close(first_fill(&run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.5, 0.2, 0.8]), vec![lv.clone()]))), 0.5, 0.0, 1.0);
        let mut bad = lv;
        bad["ef"][2] = efp(0, json!(10));
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.5, 0.2, 0.8]), vec![bad]));
        close(first_fill(&imp), 0.5, 0.2, 0.8);
        assert!(imp.warnings.iter().any(|w| w.contains("Levels") && w.contains("invalid")), "{:?}", imp.warnings);
    }

    #[test]
    fn effects_apply_in_order_and_reach_gradients_and_strokes() {
        let shapes = json!([{"ty": "rc", "p": {"a": 0, "k": [5, 5]}, "s": {"a": 0, "k": [4, 4]}, "r": {"a": 0, "k": 0}},
            {"ty": "gf", "t": 1, "s": {"a": 0, "k": [0, 0]}, "e": {"a": 0, "k": [10, 0]}, "g": {"p": 2, "k": {"a": 0, "k": [0, 1, 0, 0, 1, 1, 1, 1]}}, "o": {"a": 0, "k": 100}},
            {"ty": "st", "c": {"a": 0, "k": [1, 1, 1, 1]}, "o": {"a": 0, "k": 100}, "w": {"a": 0, "k": 1}}]);
        let fill = |c: [f64; 3]| effect(21, vec![efp(7, json!(0)), efp(7, json!(0)), efp(2, json!(c)), efp(7, json!(0)), efp(0, json!(0)), efp(0, json!(0)), efp(0, json!(1))]);
        let inv_levels = effect(24, vec![efp(7, json!(1)), efp(0, json!(0)), efp(0, json!(255)), efp(0, json!(1)), efp(0, json!(255)), efp(0, json!(0))]);
        let imp = run(fx_layer(shapes, vec![fill([1.0, 0.0, 0.0]), inv_levels]));
        let crate::project::LayerKind::Vector { paths } = &imp.project.layers[0].kind else { panic!() };
        // red filled, then inverted by the second effect -> cyan, also in the stroke
        let Some(Paint::Linear { stops, .. }) = &paths[0].fill else { panic!("{:?}", paths[0].fill) };
        assert_eq!(stops.len(), 2);
        close(stops[0].color, 0.0, 1.0, 1.0);
        close(stops[1].color, 0.0, 1.0, 1.0);
        let Paint::Solid(s) = &paths[0].stroke.as_ref().unwrap().paint else { panic!() };
        close(*s, 0.0, 1.0, 1.0);
    }

    fn anim_p(ty: i64, keys: Value) -> Value {
        json!({"ty": ty, "nm": "p", "v": {"a": 1, "k": keys}})
    }
    fn blur_ef(r: Value) -> Value {
        effect(29, vec![r, efp(7, json!(1)), efp(7, json!(0))])
    }
    fn shadow_ef(color: [f64; 3], op: Value, dir: Value, dist: Value, soft: Value) -> Value {
        effect(25, vec![efp(2, json!(color)), op, dir, dist, soft, efp(7, json!(0))])
    }
    fn rgba_img(imp: &Imported, frame: f32) -> crate::percep::Img {
        let e = Engine::new(imp.project.clone()).unwrap();
        let (w, h, d) = crate::raster::render_rgba(&e, frame, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: true });
        crate::percep::Img::new(w as usize, h as usize, d).unwrap()
    }

    #[test]
    fn blur_and_shadow_become_layer_effects() {
        // direction 90 degrees = right (clockwise from up), distance 5, softness 3, opacity 127.5/255
        let sh = shadow_ef([0.0, 0.0, 0.0], efp(0, json!(127.5)), efp(0, json!(90)), efp(0, json!(5)), efp(0, json!(3)));
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![blur_ef(efp(0, json!(8))), sh]));
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        assert!(imp.post_fx.is_empty());
        let fx = &imp.project.layers[0].effects;
        assert_eq!(fx.len(), 2);
        assert_eq!(fx[0], LayerEffect::blur(8.0));
        let crate::layerfx::EffectKind::DropShadow { dx, dy, blur, color, opacity } = &fx[1].kind else { panic!() };
        assert!((dx - 5.0).abs() < 1e-4 && dy.abs() < 1e-4 && (*blur - 3.0).abs() < 1e-6 && *color == Rgba::BLACK && (opacity - 0.5).abs() < 1e-6);
        // direction 135 degrees points down and right
        let sh2 = shadow_ef([0.0, 0.0, 0.0], efp(0, json!(255)), efp(0, json!(135)), efp(0, json!(10)), efp(0, json!(0)));
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![sh2]));
        let crate::layerfx::EffectKind::DropShadow { dx, dy, .. } = &imp.project.layers[0].effects[0].kind else { panic!() };
        assert!(*dx > 7.0 && *dy > 7.0, "{dx} {dy}");
        // it renders: the halo is there
        let i = rgba_img(&imp, 0.0);
        assert!(i.px(10, 10)[3] > 0, "{:?}", i.px(10, 10));
    }

    fn two_layer_doc(fx_on_a: Vec<Value>) -> Imported {
        let a = lay("a", 2, stat_ks([0.0, 0.0]), rect_shapes(4.0, 14.0, 10.0, 10.0, [1.0, 0.0, 0.0]), json!({"ef": fx_on_a}));
        let b = lay("b", 1, stat_ks([0.0, 0.0]), rect_shapes(44.0, 14.0, 10.0, 10.0, [0.0, 0.0, 1.0]), json!({}));
        import_lottie(&base_doc(json!([a, b]), json!([]))).unwrap()
    }

    #[test]
    fn effects_work_in_multi_layer_compositions_and_stay_isolated() {
        let with = two_layer_doc(vec![blur_ef(efp(0, json!(5))), shadow_ef([0.0, 0.0, 0.0], efp(0, json!(255)), efp(0, json!(135)), efp(0, json!(4)), efp(0, json!(2)))]);
        assert!(with.warnings.is_empty(), "{:?}", with.warnings);
        assert!(with.post_fx.is_empty());
        let without = two_layer_doc(vec![]);
        assert_eq!(with.project.layers.len(), 2);
        // Lottie draws the bottom of the list first: `b` is layer 0, `a` layer 1
        let (b, a) = (&with.project.layers[0], &with.project.layers[1]);
        assert!(b.effects.is_empty() && a.effects.len() == 2, "{:?}", with.project.layers.iter().map(|l| (&l.name, l.effects.len())).collect::<Vec<_>>());
        let (iw, io) = (rgba_img(&with, 0.0), rgba_img(&without, 0.0));
        assert_ne!(iw, io);
        for y in 0..iw.h {
            for x in 30..iw.w {
                assert_eq!(iw.px(x, y), io.px(x, y), "({x},{y})");
            }
        }
        assert_eq!(iw.px(44, 19), [0, 0, 255, 255]);
    }

    #[test]
    fn animated_effect_values_become_channels() {
        // blur 0 -> 10 over 10 frames, linear; drop shadow distance 0 -> 10 at 90 degrees, opacity 255 -> 0
        let blur = blur_ef(anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [10]}])));
        let sh = shadow_ef([0.0, 0.0, 0.0], anim_p(0, json!([{"t": 0, "s": [255]}, {"t": 10, "s": [0]}])), efp(0, json!(90)), anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [10]}])), efp(0, json!(0)));
        let tint = effect(20, vec![efp(2, json!([0, 0, 0])), efp(2, json!([1, 1, 1])), anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [100]}]))]);
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![blur, sh, tint]));
        assert!(imp.warnings.iter().all(|w| !w.contains("first value")), "{:?}", imp.warnings);
        // animated tint stays a layer effect (cannot be baked), static colours of the shape are untouched
        close(first_fill(&imp), 1.0, 0.0, 0.0);
        assert_eq!(imp.project.layers[0].effects.len(), 3);
        let e = Engine::new(imp.project.clone()).unwrap();
        for (f, r, dx, op, amt) in [(0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32), (5.0, 5.0, 5.0, 0.5, 0.5), (10.0, 10.0, 10.0, 0.0, 1.0)] {
            let fx = &e.evaluate(f).nodes[0].effects;
            assert!((fx[0].get(Prop::FxBlur).unwrap() - r).abs() < 1e-3, "frame {f}");
            assert!((fx[1].get(Prop::FxDx).unwrap() - dx).abs() < 1e-3 && fx[1].get(Prop::FxDy).unwrap().abs() < 1e-3, "frame {f}: {:?}", fx[1]);
            assert!((fx[1].get(Prop::FxAmount).unwrap() - op).abs() < 1e-3);
            assert!((fx[2].get(Prop::FxAmount).unwrap() - amt).abs() < 1e-3);
        }
        assert_ne!(rgba_img(&imp, 0.0), rgba_img(&imp, 8.0));
    }

    #[test]
    fn hold_and_bezier_easing_carry_over_to_effect_channels() {
        let hold = blur_ef(anim_p(0, json!([{"t": 0, "s": [2], "h": 1}, {"t": 10, "s": [20]}])));
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![hold]));
        let e = Engine::new(imp.project).unwrap();
        assert_eq!(e.evaluate(9.0).nodes[0].effects[0].get(Prop::FxBlur), Some(2.0));
        assert_eq!(e.evaluate(10.0).nodes[0].effects[0].get(Prop::FxBlur), Some(20.0));
        let ease = blur_ef(anim_p(0, json!([{"t": 0, "s": [0], "o": {"x": [0.9], "y": [0.0]}, "i": {"x": [0.9], "y": [0.0]}}, {"t": 10, "s": [20]}])));
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![ease]));
        let e = Engine::new(imp.project).unwrap();
        let mid = e.evaluate(5.0).nodes[0].effects[0].get(Prop::FxBlur).unwrap();
        assert!(mid < 5.0, "ease-in stays low at the middle: {mid}");
    }

    #[test]
    fn shadow_direction_animation_follows_the_arc() {
        // direction 0 -> 180 degrees at distance 10: halfway the shadow points right (dx = 10), not at the straight-line midpoint (dx = 0)
        let sh = shadow_ef([0.0, 0.0, 0.0], efp(0, json!(255)), anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [180]}])), efp(0, json!(10)), efp(0, json!(0)));
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![sh]));
        let e = Engine::new(imp.project).unwrap();
        let fx = &e.evaluate(5.0).nodes[0].effects[0];
        assert!((fx.get(Prop::FxDx).unwrap() - 10.0).abs() < 0.2 && fx.get(Prop::FxDy).unwrap().abs() < 0.2, "{fx:?}");
        let fx = &e.evaluate(0.0).nodes[0].effects[0];
        assert!(fx.get(Prop::FxDx).unwrap().abs() < 1e-3 && (fx.get(Prop::FxDy).unwrap() + 10.0).abs() < 1e-3);
    }

    #[test]
    fn colour_effects_reach_precomp_and_image_layers() {
        let green = effect(21, vec![efp(7, json!(0)), efp(7, json!(0)), efp(2, json!([0, 1, 0])), efp(7, json!(0)), efp(0, json!(0)), efp(0, json!(0)), efp(0, json!(1))]);
        let pre = json!({"ty": 0, "nm": "pc", "refId": "c", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]), "ef": [green.clone()]});
        let assets = json!([{"id": "c", "layers": [lay("in", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({}))]}]);
        let imp = import_lottie(&base_doc(json!([pre]), assets.clone())).unwrap();
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        assert_eq!(imp.project.layers[0].effects, vec![LayerEffect::fill(Rgba::new(0.0, 1.0, 0.0, 1.0), 1.0)]);
        let i = rgba_img(&imp, 0.0);
        assert_eq!(i.px(2, 2), [0, 255, 0, 255]);
        // tint on a precomp, with animated amount
        let tint = effect(20, vec![efp(2, json!([0, 0, 1])), efp(2, json!([1, 1, 0])), anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [100]}]))]);
        let pre = json!({"ty": 0, "nm": "pc", "refId": "c", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]), "ef": [tint]});
        let imp = import_lottie(&base_doc(json!([pre]), assets.clone())).unwrap();
        assert_eq!(rgba_img(&imp, 0.0).px(2, 2), [255, 0, 0, 255]);
        let end = rgba_img(&imp, 10.0).px(2, 2);
        assert!(end[0].abs_diff(54) <= 1 && end[1].abs_diff(54) <= 1 && end[2].abs_diff(201) <= 1, "red has luminance .21 -> between blue and yellow: {end:?}");
        // an image layer
        let img = json!({"ty": 2, "nm": "im", "refId": "i", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]), "ef": [green.clone()]});
        let assets = json!([{"id": "i", "w": 2, "h": 2, "p": png_uri([255, 0, 0, 255]), "e": 1}]);
        let imp = import_lottie(&base_doc(json!([img]), assets)).unwrap();
        assert_eq!(imp.project.layers[0].effects.len(), 1, "{:?}", imp.warnings);
        let p = rgba_img(&imp, 0.0).px(0, 0);
        assert!(p[1] > 250 && p[0] < 5, "{p:?}");
        // tritone / levels have no layer form: named, not applied
        let tri = effect(23, vec![efp(2, json!([1, 1, 1])), efp(2, json!([0.5, 0.5, 0.5])), efp(2, json!([0, 0, 0])), efp(0, json!(0))]);
        let pre = json!({"ty": 0, "nm": "pc", "refId": "c", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0]), "ef": [tri]});
        let assets = json!([{"id": "c", "layers": [lay("in", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({}))]}]);
        let imp = import_lottie(&base_doc(json!([pre]), assets)).unwrap();
        assert!(imp.project.layers[0].effects.is_empty());
        assert!(imp.warnings.iter().any(|w| w.contains("Tritone") && w.contains("shape layers")), "{:?}", imp.warnings);
    }

    #[test]
    fn effects_inside_precomps_use_prefixed_channel_targets() {
        let blur = blur_ef(anim_p(0, json!([{"t": 0, "s": [0]}, {"t": 10, "s": [6]}])));
        let inner = lay("in", 1, stat_ks([0.0, 0.0]), rect_shapes(10.0, 10.0, 10.0, 10.0, [1.0, 0.0, 0.0]), json!({"ef": [blur]}));
        let pre = json!({"ty": 0, "nm": "pc", "refId": "c", "ind": 1, "ip": 0, "op": 11, "ks": stat_ks([0.0, 0.0])});
        let imp = import_lottie(&base_doc(json!([pre]), json!([{"id": "c", "layers": [inner]}]))).unwrap();
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        assert!(imp.project.channels.iter().any(|c| c.prop == Prop::FxBlur && c.target.ends_with("/in::fx0")), "{:?}", imp.project.channels.iter().map(|c| &c.target).collect::<Vec<_>>());
        let e = Engine::new(imp.project.clone()).unwrap();
        let n = e.evaluate(10.0).nodes.iter().find(|n| n.name.ends_with("/in")).unwrap().effects.clone();
        assert_eq!(n[0].get(Prop::FxBlur), Some(6.0));
        // the precomp group itself is not blurred
        assert!(e.evaluate(10.0).nodes[0].effects.is_empty());
    }

    #[test]
    fn effects_survive_masks_and_two_effect_orders() {
        let mk = || vec![mask("a", false, 100.0, 5.0, 5.0, 15.0, 15.0)];
        let l = lay("m", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 30.0, 30.0, [1.0, 0.0, 0.0]), json!({"masksProperties": mk(), "ef": [blur_ef(efp(0, json!(4)))]}));
        let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
        // the effect sits on the outer (masked) layer, after the mask
        let top = &imp.project.layers.last().unwrap();
        assert_eq!(top.effects.len(), 1, "{:?}", imp.warnings);
        Engine::new(imp.project).unwrap();
    }

    #[test]
    fn disabled_unsupported_and_malformed_effects_never_crash() {
        let green = effect(21, vec![efp(7, json!(0)), efp(7, json!(0)), efp(2, json!([0, 1, 0]))]);
        let mut off = green.clone();
        off["en"] = json!(0);
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![off]));
        close(first_fill(&imp), 1.0, 0.0, 0.0);
        assert!(imp.warnings.is_empty(), "{:?}", imp.warnings);
        // disabled blur / shadow / tint produce no effects at all
        let mut offs = vec![blur_ef(efp(0, json!(5))), shadow_ef([0.0; 3], efp(0, json!(255)), efp(0, json!(0)), efp(0, json!(3)), efp(0, json!(1))), effect(20, vec![efp(2, json!([0, 0, 0])), efp(2, json!([1, 1, 1])), efp(0, json!(100))])];
        offs.iter_mut().for_each(|e| e["en"] = json!(0));
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), offs));
        assert!(imp.project.layers[0].effects.is_empty() && imp.warnings.is_empty(), "{:?}", imp.warnings);
        // unsupported ones are named
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![effect(30, vec![]), json!({"ty": 99, "nm": "Weird Glow", "en": 1}), effect(26, vec![])]));
        let w = imp.warnings.join("\n");
        assert!(w.contains("Twirl") && w.contains("Weird Glow") && w.contains("Radial Wipe"), "{w}");
        // one-dimensional blur is not supported and says so
        let hb = effect(29, vec![efp(0, json!(8)), efp(7, json!(2))]);
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), vec![hb]));
        assert!(imp.project.layers[0].effects.is_empty() && imp.warnings.iter().any(|w| w.contains("both dimensions")), "{:?}", imp.warnings);
        // malformed shapes of the ef array and of its values
        for bad in [json!("x"), json!(5), json!({"a": 1}), json!([1, "a", null, [], {"ty": "z"}]),
                    json!([{"ty": 21, "en": 1, "ef": "no"}, {"ty": 21, "en": 1, "ef": [null, {"v": 3}, {"v": {"a": 0, "k": "red"}}]}]),
                    json!([{"ty": 29, "en": 1, "ef": [{"v": {"a": 0, "k": [1e300]}}]}, {"ty": 25, "ef": [{"v": {"a": 1, "k": []}}]}]),
                    json!([{"ty": 29, "en": 1, "ef": [{"v": {"a": 0, "k": -5}}]}, {"ty": 20, "en": 1, "ef": [{"v": {"a": 0, "k": [1]}}]}]),
                    json!([{"ty": 29, "en": 1, "ef": [{"v": {"a": 1, "k": [{"t": 0}, {"t": "x", "s": [1]}, 7]}}]}]),
                    json!([{"ty": 25, "en": 1, "ef": [{"v": {"a": 0, "k": [0, 0, 0]}}, {"v": {"a": 0, "k": 255}}, {"v": {"a": 1, "k": [{"t": 0, "s": [1e30]}, {"t": 1e30, "s": [5]}]}}, {"v": {"a": 0, "k": 3}}, {"v": {"a": 0, "k": 1}}]}]),
                    json!((0..200).map(|_| blur_ef(efp(0, json!(1)))).collect::<Vec<_>>())] {
            let l = lay("m", 1, stat_ks([0.0, 0.0]), rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 0.0, 0.0]), json!({"ef": bad}));
            let imp = import_lottie(&base_doc(json!([l]), json!([]))).unwrap();
            assert_eq!(imp.project.layers.len(), 1);
            close(first_fill(&imp), 1.0, 0.0, 0.0);
            Engine::new(imp.project.clone()).expect("whatever was imported must be a valid project");
            assert!(imp.project.layers[0].effects.len() <= crate::layerfx::MAX_EFFECTS);
        }
    }

    #[test]
    fn animated_colours_are_reported_as_first_value() {
        let anim = json!({"ty": 2, "nm": "c", "v": {"a": 1, "k": [{"t": 0, "s": [1, 0, 0]}, {"t": 10, "s": [0, 0, 1]}]}});
        let e = effect(20, vec![anim, efp(2, json!([1, 1, 1])), efp(0, json!(100))]);
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [0.0, 0.0, 0.0]), vec![e]));
        assert!(imp.warnings.iter().any(|w| w.contains("Tint") && w.contains("first value")), "{:?}", imp.warnings);
        close(first_fill(&imp), 1.0, 0.0, 0.0);
    }

    #[test]
    fn static_colour_effects_stay_baked_on_shape_layers_but_follow_blur_order() {
        // alone: baked (no layer effect); next to a blur: kept as layer effects in `ef` order
        let tint = || effect(20, vec![efp(2, json!([0, 0, 1])), efp(2, json!([1, 1, 0])), efp(0, json!(100))]);
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 1.0, 1.0]), vec![tint()]));
        assert!(imp.project.layers[0].effects.is_empty());
        let imp = run(fx_layer(rect_shapes(0.0, 0.0, 4.0, 4.0, [1.0, 1.0, 1.0]), vec![tint(), blur_ef(efp(0, json!(3)))]));
        close(first_fill(&imp), 1.0, 1.0, 1.0);
        let k: Vec<_> = imp.project.layers[0].effects.iter().map(|e| std::mem::discriminant(&e.kind)).collect();
        assert_eq!(k, vec![std::mem::discriminant(&LayerEffect::tint(Rgba::BLACK, Rgba::WHITE, 1.0).kind), std::mem::discriminant(&LayerEffect::blur(1.0).kind)]);
    }

    // ---- exporter round trip ----
    fn effects_at(e: &Engine, frame: f32, name: &str) -> Vec<LayerEffect> {
        e.evaluate(frame).nodes.iter().find(|n| n.name == name).map(|n| n.effects.clone()).unwrap_or_default()
    }

    fn near(a: &[LayerEffect], b: &[LayerEffect]) -> bool {
        use crate::layerfx::EffectKind as K;
        let f = |x: f32, y: f32| (x - y).abs() < 0.02;
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| match (&x.kind, &y.kind) {
            (K::Blur { radius: p }, K::Blur { radius: q }) => f(*p, *q),
            (K::DropShadow { dx, dy, blur, color, opacity }, K::DropShadow { dx: dx2, dy: dy2, blur: b2, color: c2, opacity: o2 }) => f(*dx, *dx2) && f(*dy, *dy2) && f(*blur, *b2) && f(*opacity, *o2) && color == c2,
            (K::Tint { black, white, amount }, K::Tint { black: b2, white: w2, amount: a2 }) => black == b2 && white == w2 && f(*amount, *a2),
            (K::Fill { color, opacity }, K::Fill { color: c2, opacity: o2 }) => color == c2 && f(*opacity, *o2),
            _ => false,
        })
    }

    fn render_img(e: &Engine, frame: f32) -> crate::percep::Img {
        let (w, h, d) = crate::raster::render_rgba(e, frame, &crate::raster::RasterOptions { scale: 1.0, show_bones: false, transparent: false });
        crate::percep::Img::new(w as usize, h as usize, d).unwrap()
    }

    #[test]
    fn exporter_writes_layer_effects_and_reimport_gives_them_back() {
        use crate::project::Channel;
        let mut p = Project::new("rt", 60.0, 40.0, 10.0, 11);
        let mut a = Layer::vector("a", vec![crate::path::VPath::rect("r", 10.0, 10.0, 20.0, 15.0).with_fill("#ff0000")]);
        a.effects = vec![
            LayerEffect::blur(3.0),
            LayerEffect::drop_shadow(4.0, -3.0, 2.0, Rgba::new(0.0, 0.0, 0.5, 1.0), 0.6),
            LayerEffect::tint(Rgba::new(0.0, 0.0, 0.2, 1.0), Rgba::new(1.0, 1.0, 0.5, 1.0), 0.5),
        ];
        let mut b = Layer::vector("b", vec![crate::path::VPath::rect("r", 40.0, 10.0, 10.0, 10.0).with_fill("#00ff00")]);
        b.effects = vec![LayerEffect::fill(Rgba::new(0.0, 0.0, 1.0, 1.0), 0.5)];
        let mut off = LayerEffect::blur(9.0);
        off.enabled = false;
        b.effects.push(off);
        p.layers = vec![a, b];
        // animated: blur radius and shadow offset
        p.channels = vec![
            Channel::keys("a::fx0", Prop::FxBlur, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 6.0, Ease::Linear)])),
            Channel::keys("a::fx1", Prop::FxDx, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 8.0, Ease::Linear)])),
            Channel::keys("a::fx2", Prop::FxAmount, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 1.0, Ease::Linear)])),
        ];
        let orig = Engine::new(p).unwrap();
        let json = crate::export::lottie_json(&orig).to_string();
        let back = import_lottie(&json).unwrap();
        assert!(back.warnings.iter().all(|w| !w.contains("effect")), "{:?}", back.warnings);
        let re = Engine::new(back.project.clone()).unwrap();
        // the disabled effect is not written
        assert!(!json.contains("\"en\":0"));
        for f in 0..11 {
            let f = f as f32;
            // layer "b" has a plain colour effect: on a shape layer with geometry the importer bakes it into the fill
            // (its picture is compared below), "a" keeps its effects as effects
            let (x, y) = (effects_at(&orig, f, "a"), effects_at(&re, f, "a"));
            assert!(near(&x, &y), "a frame {f}: {x:?} vs {y:?}");
        }
        for f in [0.0, 5.0, 10.0] {
            let (ia, ib) = (render_img(&orig, f), render_img(&re, f));
            let d = crate::percep::mse(&ia, &ib).unwrap();
            assert!(d < 40.0, "frame {f}: re-imported picture differs (mse {d})");
        }
        // export is deterministic
        assert_eq!(json, crate::export::lottie_json(&orig).to_string());
    }

    #[test]
    fn exporter_moves_single_leaf_group_effects_and_skips_multi_leaf_groups() {
        let mut one = Layer::group("g1", vec![Layer::vector("a", vec![crate::path::VPath::rect("r", 5.0, 5.0, 10.0, 10.0).with_fill("#f00")])]);
        one.effects = vec![LayerEffect::blur(2.0)];
        let mut two = Layer::group("g2", vec![Layer::vector("b", vec![crate::path::VPath::rect("r", 30.0, 5.0, 10.0, 10.0).with_fill("#00f")]), Layer::vector("c", vec![crate::path::VPath::rect("r", 45.0, 5.0, 10.0, 10.0).with_fill("#0f0")])]);
        two.effects = vec![LayerEffect::blur(2.0)];
        let mut p = Project::new("g", 60.0, 40.0, 10.0, 2);
        p.layers = vec![one, two];
        let j = crate::export::lottie_json(&Engine::new(p).unwrap());
        let layers = j["layers"].as_array().unwrap();
        let with_ef: Vec<&str> = layers.iter().filter(|l| l.get("ef").is_some()).map(|l| l["nm"].as_str().unwrap()).collect();
        assert_eq!(with_ef, vec!["g1/a"], "{with_ef:?}");
    }

    #[test]
    fn exporter_without_effects_writes_no_ef() {
        let e = Engine::new(crate::demo::project()).unwrap();
        assert!(!crate::export::lottie_json(&e).to_string().contains("\"ef\""));
    }

    // ---- animated gradients (wave 4, M4) ---------------------------------------------------------

    fn gf_json(t: i64, s: Value, e: Value) -> Value {
        json!({"ty": "gf", "t": t, "s": s, "e": e, "o": {"a": 0, "k": 100}, "g": {"p": 2, "k": {"a": 0, "k": [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]}}})
    }

    fn fill_of(e: &Engine, f: f32) -> Paint {
        let fr = e.evaluate(f);
        fr.nodes.iter().find_map(|n| n.paths.first().and_then(|p| p.fill.clone())).unwrap()
    }

    /// The centre and the radius of an animated radial gradient follow the Lottie keys on every frame; they used to
    /// stay at the first key (a warning said so).
    #[test]
    fn animated_radial_gradient_geometry_is_imported() {
        let s = json!({"a": 1, "k": [{"t": 0, "s": [10.0, 10.0], "i": {"x": [1], "y": [1]}, "o": {"x": [0], "y": [0]}}, {"t": 10, "s": [30.0, 20.0]}]});
        let e = json!({"a": 1, "k": [{"t": 0, "s": [20.0, 10.0], "i": {"x": [1], "y": [1]}, "o": {"x": [0], "y": [0]}}, {"t": 10, "s": [30.0, 50.0]}]});
        let shapes = json!([{"ty": "rc", "p": {"a": 0, "k": [20, 20]}, "s": {"a": 0, "k": [40, 40]}, "r": {"a": 0, "k": 0}}, gf_json(2, s, e)]);
        let r = run(lay("g", 1, stat_ks([0.0, 0.0]), shapes, json!({})));
        assert!(!r.warnings.iter().any(|w| w.contains("gradient")), "{:?}", r.warnings);
        let en = Engine::new(r.project).unwrap();
        for (f, c, rad) in [(0.0f32, (10.0f32, 10.0f32), 10.0f32), (5.0, (20.0, 15.0), (5.0f32 * 5.0 + 15.0 * 15.0).sqrt()), (10.0, (30.0, 20.0), 30.0)] {
            match fill_of(&en, f) {
                Paint::Radial { center, radius, .. } => {
                    assert!((center.x - c.0).abs() < 1e-3 && (center.y - c.1).abs() < 1e-3, "frame {f}: centre {center:?}");
                    assert!((radius - rad).abs() < 0.05, "frame {f}: radius {radius} vs {rad}");
                }
                other => panic!("{other:?}"),
            }
        }
    }

    #[test]
    fn animated_linear_gradient_end_points_and_group_transforms() {
        let s = json!({"a": 0, "k": [0.0, 0.0]});
        let e = json!({"a": 1, "k": [{"t": 0, "s": [10.0, 0.0], "i": {"x": [1], "y": [1]}, "o": {"x": [0], "y": [0]}}, {"t": 10, "s": [10.0, 20.0]}]});
        // the group moves by (100, 0) and scales by 2: gradient points live in the group's space
        let tr = json!({"ty": "tr", "p": {"a": 0, "k": [100, 0]}, "a": {"a": 0, "k": [0, 0]}, "s": {"a": 0, "k": [200, 200]}, "r": {"a": 0, "k": 0}, "o": {"a": 0, "k": 100}});
        let shapes = json!([{"ty": "gr", "it": [{"ty": "rc", "p": {"a": 0, "k": [5, 5]}, "s": {"a": 0, "k": [10, 10]}, "r": {"a": 0, "k": 0}}, gf_json(1, s, e), tr]}]);
        let r = run(lay("g", 1, stat_ks([0.0, 0.0]), shapes, json!({})));
        let en = Engine::new(r.project).unwrap();
        for (f, to) in [(0.0f32, (120.0f32, 0.0f32)), (10.0, (120.0, 40.0))] {
            match fill_of(&en, f) {
                Paint::Linear { from, to: t, .. } => {
                    assert!((from.x - 100.0).abs() < 1e-3 && from.y.abs() < 1e-3, "{from:?}");
                    assert!((t.x - to.0).abs() < 1e-3 && (t.y - to.1).abs() < 1e-3, "frame {f}: {t:?}");
                }
                other => panic!("{other:?}"),
            }
        }
    }

    /// Osatura -> Lottie -> Osatura: a path whose radial gradient moves (and grows) keeps looking the same on every
    /// frame. With the gradient frozen at the first key (the old import) the pictures differ by a lot.
    #[test]
    fn animated_gradient_survives_the_round_trip() {
        use crate::project::Channel;
        let mut p = Project::new("grad", 80.0, 60.0, 12.0, 13);
        p.background = Some(Rgba::hex("#ffffff"));
        let path = crate::path::VPath::rect("blob", 5.0, 5.0, 70.0, 50.0).with_paint(Paint::radial(Vec2::new(20.0, 30.0), 12.0, &[(0.0, "#ff0000"), (1.0, "#0000ff")]));
        p.layers = vec![Layer::vector("blush", vec![path])];
        p.channels = vec![
            Channel::keys("blush::blob::fill.from", Prop::GradDx, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, 40.0, Ease::Linear)])),
            Channel::keys("blush::blob::fill.from", Prop::GradDy, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, -10.0, Ease::Linear)])),
            Channel::keys("blush::blob::fill.to", Prop::GradDx, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (12.0, 18.0, Ease::Linear)])),
        ];
        let e = Engine::new(p.clone()).unwrap();
        let j = crate::export::lottie_json(&e).to_string();
        let back = import_lottie(&j).unwrap();
        assert!(!back.warnings.iter().any(|w| w.contains("gradient")), "{:?}", back.warnings);
        assert!(back.project.channels.iter().any(|c| c.prop == Prop::GradDx), "gradient channels were created");
        let re = Engine::new(back.project.clone()).unwrap();
        let mut frozen = back.project;
        frozen.channels.retain(|c| !matches!(c.prop, Prop::GradDx | Prop::GradDy));
        let fe = Engine::new(frozen).unwrap();
        let mut worst_frozen = 0.0f64;
        for f in 0..=12 {
            let f = f as f32;
            let (a, b, c) = (render_img(&e, f), render_img(&re, f), render_img(&fe, f));
            let mse = crate::percep::mse(&a, &b).unwrap();
            assert!(mse < 4.0, "frame {f}: mse {mse}");
            worst_frozen = worst_frozen.max(crate::percep::mse(&a, &c).unwrap());
        }
        assert!(worst_frozen > 400.0, "the test must be able to tell a frozen gradient from a moving one ({worst_frozen})");
    }

    /// The reviewer's case: the demo's bone-driven radial "blush" must be where it was on every frame after a
    /// round trip (it used to vanish on ~40 of 48 frames because the gradient stayed at the first key). The gradient is
    /// compared in stage coordinates, frame by frame.
    #[test]
    fn demo_round_trip_keeps_the_blush_on_every_frame() {
        let e = Engine::new(crate::demo::project()).unwrap();
        let back = import_lottie(&crate::export::lottie_json(&e).to_string()).unwrap();
        let re = Engine::new(back.project).unwrap();
        let stage = |en: &Engine, f: f32| -> Option<(Vec2, f32)> {
            let fr = en.evaluate(f);
            fr.nodes.iter().filter(|n| n.name.ends_with("blush")).find_map(|n| {
                n.paths.first().and_then(|p| match &p.fill {
                    Some(Paint::Radial { center, radius, .. }) => Some((n.world.apply(*center), radius * (n.world.a * n.world.d - n.world.b * n.world.c).abs().sqrt())),
                    _ => None,
                })
            })
        };
        let mut moved = 0.0f32;
        let first = stage(&e, 0.0).expect("blush in the original");
        for f in 0..e.project.frames {
            let (a, b) = (stage(&e, f as f32).unwrap(), stage(&re, f as f32).expect("blush after the round trip"));
            assert!(a.0.distance(b.0) < 1.0, "frame {f}: centre {:?} vs {:?}", a.0, b.0);
            assert!((a.1 - b.1).abs() < 0.1 * a.1.max(1.0), "frame {f}: radius {} vs {}", a.1, b.1);
            moved = moved.max(a.0.distance(first.0));
        }
        assert!(moved > 5.0, "the blush moves in the demo ({moved}), so this test can tell a frozen gradient");
    }

    /// One shape layer holding `n` identical "sh" items in a group, all with the default name.
    fn many_shapes_doc(n: usize) -> String {
        let sh = json!({"ty": "sh", "ks": {"a": 0, "k": rect_shape_json(0.0, 0.0, 4.0, 4.0)}});
        let mut items: Vec<Value> = (0..n).map(|_| sh.clone()).collect();
        items.push(json!({"ty": "fl", "c": {"a": 0, "k": [1, 0, 0, 1]}, "o": {"a": 0, "k": 100}}));
        base_doc(json!([lay("many", 1, stat_ks([0.0, 0.0]), json!([{"ty": "gr", "it": items}]), json!({}))]), json!([])).to_string()
    }

    /// Importing grows about linearly with the number of shapes (the duplicate-name search used to be quadratic: 2000
    /// shapes took 1.1 s, 4000 took 5.6 s).
    #[test]
    fn importing_thousands_of_shapes_is_not_quadratic() {
        let time = |n: usize| {
            let doc = many_shapes_doc(n);
            let t = crate::clock::Stamp::now();
            let imp = import_lottie(&doc).unwrap();
            let el = t.elapsed().as_secs_f64();
            let count = match &imp.project.layers[0].kind {
                crate::project::LayerKind::Vector { paths } => paths.len(),
                other => panic!("{other:?}"),
            };
            assert_eq!(count, n);
            el
        };
        let (a, b) = (time(1500), time(6000));
        eprintln!("1500 shapes {a:.3} s, 6000 shapes {b:.3} s, ratio {:.1}", b / a);
        // linear would be 4x; quadratic 16x. The margin leaves room for timing noise.
        assert!(b < 10.0 * a.max(0.02), "1500 shapes took {a:.3} s, 6000 took {b:.3} s");
        // unique, stable names
        let imp = import_lottie(&many_shapes_doc(5)).unwrap();
        if let crate::project::LayerKind::Vector { paths } = &imp.project.layers[0].kind {
            let names: Vec<&str> = paths.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(names, vec!["path", "path_2", "path_3", "path_4", "path_5"]);
        }
    }
}
