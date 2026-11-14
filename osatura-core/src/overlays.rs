// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Authoring overlays that are never part of the exported animation: onion skin and
//! motion trails.

use crate::engine::Engine;
use crate::math::Vec2;
use crate::pick::PointKind;
use crate::project::{Prop, Source};
use crate::raster::{render_rgba, RasterOptions};
use crate::style::Rgba;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

#[derive(Clone, Debug)]
pub struct OnionOptions {
    pub before: u32,
    pub after: u32,
    /// Frame spacing between ghosts.
    pub step: u32,
    /// Opacity of the nearest ghost (0..1); farther ghosts fade linearly.
    pub opacity: f32,
    pub scale: f32,
}

impl Default for OnionOptions {
    fn default() -> Self {
        OnionOptions { before: 2, after: 1, step: 1, opacity: 0.45, scale: 1.0 }
    }
}

fn over_px(dst: &mut [u8], src: [u8; 4], alpha: f32) {
    let a = (src[3] as f32 / 255.0) * alpha;
    let da = dst[3] as f32 / 255.0;
    let oa = a + da * (1.0 - a);
    if oa <= 0.0 {
        return;
    }
    for c in 0..3 {
        let v = (src[c] as f32 * a + dst[c] as f32 * da * (1.0 - a)) / oa;
        dst[c] = v.round().clamp(0.0, 255.0) as u8;
    }
    dst[3] = (oa * 255.0).round() as u8;
}

/// Current frame with tinted ghosts of neighbouring frames drawn over it (red = past,
/// green = future). A ghost pixel is only drawn where it differs from the current frame, so
/// static background is left untouched and only the moving parts show.
pub fn onion_skin(engine: &Engine, frame: u32, o: &OnionOptions) -> Result<(u32, u32, Vec<u8>), String> {
    if o.step == 0 || !(0.0..=1.0).contains(&o.opacity) || o.before > 16 || o.after > 16 {
        return Err("invalid onion-skin options".into());
    }
    let full = RasterOptions { scale: o.scale, show_bones: false, transparent: false };
    let (w, h, mut out) = render_rgba(engine, frame as f32, &full);
    let current = out.clone();
    let n_frames = engine.project.frames as i64;
    let mut ghosts: Vec<(i64, [f32; 3], f32)> = Vec::new();
    for i in (1..=o.before as i64).rev() {
        ghosts.push((frame as i64 - i * o.step as i64, [255.0, 70.0, 70.0], i as f32));
    }
    for i in (1..=o.after as i64).rev() {
        ghosts.push((frame as i64 + i * o.step as i64, [60.0, 200.0, 110.0], i as f32));
    }
    let reach = o.before.max(o.after).max(1) as f32;
    for (gf, tint, dist) in ghosts {
        if gf < 0 || gf >= n_frames {
            continue;
        }
        let (_, _, g) = render_rgba(engine, gf as f32, &full);
        let alpha = o.opacity * (1.0 - 0.7 * ((dist - 1.0) / reach));
        for ((d, s), c) in out.chunks_exact_mut(4).zip(g.chunks_exact(4)).zip(current.chunks_exact(4)) {
            let diff = (0..3).map(|k| (s[k] as i32 - c[k] as i32).abs()).max().unwrap_or(0) as f32;
            if diff < 12.0 {
                continue;
            }
            let t = [(s[0] as f32 * 0.4 + tint[0] * 0.6) as u8, (s[1] as f32 * 0.4 + tint[1] * 0.6) as u8, (s[2] as f32 * 0.4 + tint[2] * 0.6) as u8, 255];
            over_px(d, t, alpha * (diff / 64.0).min(1.0));
        }
    }
    Ok((w, h, out))
}

/// Screen-space path of a bone tip (or head) over `[from, to]`, one point per frame.
pub fn motion_trail(engine: &Engine, bone: &str, tip: bool, from: u32, to: u32) -> Result<Vec<Vec2>, String> {
    let bi = engine.project.bones.iter().position(|b| b.name == bone).ok_or_else(|| format!("no bone '{bone}'"))?;
    if to < from || to > engine.project.frames.max(1) {
        return Err("frame range out of bounds".into());
    }
    let len = if tip { engine.project.bones[bi].length } else { 0.0 };
    Ok((from..=to)
        .map(|f| {
            let fr = engine.evaluate(f as f32);
            fr.camera.apply(fr.world[bi].apply(Vec2::new(len, 0.0)))
        })
        .collect())
}

/// Trail as an SVG overlay (polyline + dots) sized to the project, for drawing over a render.
pub fn trail_svg(engine: &Engine, pts: &[Vec2], color: &str) -> String {
    let c = Rgba::parse(color).map(|_| color).unwrap_or("#ff3366");
    let poly: Vec<String> = pts.iter().map(|p| format!("{:.2},{:.2}", p.x, p.y)).collect();
    let dots: String = pts.iter().map(|p| format!("<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"2\" fill=\"{c}\"/>", p.x, p.y)).collect();
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\"><polyline points=\"{}\" fill=\"none\" stroke=\"{c}\" stroke-width=\"1.5\"/>{}</svg>",
        engine.project.width, engine.project.height, engine.project.width, engine.project.height, poly.join(" "), dots
    )
}


// ================================================================================================
// Onion skin as SVG ghosts (spec -> descriptors -> composition)
// ================================================================================================

pub const MAX_GHOSTS_PER_SIDE: u32 = 16;

/// How ghost frames are chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnionMode {
    /// `current - i * step` and `current + i * step`.
    #[default]
    Relative,
    /// The nearest key frames before and after the current frame (`step` is ignored).
    Keyed,
}

/// Onion-skin settings. Plain data, validated by [`OnionSpec::validate`], stored in the user's UI config.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OnionSpec {
    pub enabled: bool,
    pub before: u32,
    pub after: u32,
    pub step: u32,
    pub mode: OnionMode,
    /// Colours (`#rgb`, `#rrggbb`) of past and future ghosts.
    pub tint_before: String,
    pub tint_after: String,
    /// Opacity of the nearest ghost, 0..1.
    pub opacity: f32,
    /// Fraction of `opacity` lost by the farthest ghost, 0..1 (linear in between).
    pub falloff: f32,
    /// Hairline outlines instead of tinted silhouettes.
    pub outline: bool,
    /// Leave out shapes that look the same in the current frame (static backgrounds), so only what moves ghosts.
    pub moving_only: bool,
    /// Ghost only the layers the caller passes as the current selection (all layers when it is empty).
    pub selection_only: bool,
    /// Fixed layer filter (paths); empty = all layers. Ignored while `selection_only` applies.
    pub layers: Vec<String>,
}

impl Default for OnionSpec {
    fn default() -> Self {
        OnionSpec {
            enabled: false,
            before: 2,
            after: 1,
            step: 1,
            mode: OnionMode::Relative,
            tint_before: "#ff4646".into(),
            tint_after: "#3cc86e".into(),
            opacity: 0.45,
            falloff: 0.7,
            outline: false,
            moving_only: true,
            selection_only: false,
            layers: Vec::new(),
        }
    }
}

impl OnionSpec {
    pub fn validate(&self) -> Result<(), String> {
        let mut e = Vec::new();
        if self.before > MAX_GHOSTS_PER_SIDE || self.after > MAX_GHOSTS_PER_SIDE {
            e.push(format!("at most {MAX_GHOSTS_PER_SIDE} ghosts per side"));
        }
        if self.step == 0 || self.step > 1000 {
            e.push("step must be 1..=1000".into());
        }
        if !(0.0..=1.0).contains(&self.opacity) || !self.opacity.is_finite() {
            e.push("opacity must be in 0..=1".into());
        }
        if !(0.0..=1.0).contains(&self.falloff) || !self.falloff.is_finite() {
            e.push("falloff must be in 0..=1".into());
        }
        for (n, c) in [("tint_before", &self.tint_before), ("tint_after", &self.tint_after)] {
            if Rgba::parse(c).is_none() {
                e.push(format!("{n}: bad colour '{c}'"));
            }
        }
        if self.layers.len() > 1000 {
            e.push("too many layer filters".into());
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e.join("; "))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Before,
    After,
}

/// One ghost to draw.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Ghost {
    pub frame: u32,
    pub side: Side,
    /// 1 = nearest ghost on its side.
    pub order: u32,
    pub tint: String,
    pub opacity: f32,
}

/// Opacity of ghost number `order` (1-based) of `count` on one side.
pub fn falloff_opacity(base: f32, falloff: f32, order: u32, count: u32) -> f32 {
    if count <= 1 {
        return base;
    }
    base * (1.0 - falloff * (order.saturating_sub(1)) as f32 / (count - 1) as f32)
}

fn norm_hex(c: &str) -> String {
    Rgba::parse(c).map(|c| Rgba { a: 1.0, ..c }.to_hex()).unwrap_or_else(|| "#888888".into())
}

/// Which layers a key on `target` can move: the layer itself, its ancestors, and (for `layer::path::..` targets) the layer.
fn related_to_layers(target: &str, layers: &[String]) -> bool {
    let base = target.split("::").next().unwrap_or(target);
    layers.iter().any(|l| base == l || l.starts_with(&format!("{base}/")) || base.starts_with(&format!("{l}/")))
}

/// Frames (rounded, sorted, unique) that carry a key in any keyed channel. With a non-empty `layers`, only
/// channels that can move one of those layers.
pub fn keyed_frames(engine: &Engine, layers: &[String]) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();
    for c in &engine.project.channels {
        let Source::Keys { track } = &c.source else { continue };
        if !layers.is_empty() && !related_to_layers(&c.target, layers) {
            continue;
        }
        out.extend(track.keys.iter().filter(|k| k.frame >= 0.0).map(|k| k.frame.round() as u32));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Ghost descriptors for `current`, farthest first on each side (so nearer ghosts are drawn on top);
/// past ghosts come before future ones. `keyed` is only read in keyed mode.
pub fn ghost_list(spec: &OnionSpec, current: u32, total_frames: u32, keyed: &[u32]) -> Vec<Ghost> {
    let (tb, ta) = (norm_hex(&spec.tint_before), norm_hex(&spec.tint_after));
    let mut before: Vec<u32> = Vec::new();
    let mut after: Vec<u32> = Vec::new();
    match spec.mode {
        OnionMode::Relative => {
            for i in 1..=spec.before as i64 {
                let f = current as i64 - i * spec.step as i64;
                if f >= 0 {
                    before.push(f as u32);
                }
            }
            for i in 1..=spec.after as i64 {
                let f = current as i64 + i * spec.step as i64;
                if f < total_frames as i64 {
                    after.push(f as u32);
                }
            }
        }
        OnionMode::Keyed => {
            before = keyed.iter().rev().copied().filter(|&f| f < current).take(spec.before as usize).collect();
            after = keyed.iter().copied().filter(|&f| f > current && f < total_frames).take(spec.after as usize).collect();
        }
    }
    let mut out = Vec::new();
    for (i, &f) in before.iter().enumerate().rev() {
        let order = i as u32 + 1;
        out.push(Ghost { frame: f, side: Side::Before, order, tint: tb.clone(), opacity: falloff_opacity(spec.opacity, spec.falloff, order, spec.before) });
    }
    for (i, &f) in after.iter().enumerate().rev() {
        let order = i as u32 + 1;
        out.push(Ghost { frame: f, side: Side::After, order, tint: ta.clone(), opacity: falloff_opacity(spec.opacity, spec.falloff, order, spec.after) });
    }
    out
}

/// The layer filter that applies: the selection when `selection_only` and something is selected, else
/// the fixed `layers` list; `None` = everything.
pub fn effective_filter(spec: &OnionSpec, selection: &[String]) -> Option<Vec<String>> {
    if spec.selection_only && !selection.is_empty() {
        Some(selection.to_vec())
    } else if !spec.selection_only && !spec.layers.is_empty() {
        Some(spec.layers.clone())
    } else {
        None
    }
}

/// Descriptors for the project's current state (computes the key frames in keyed mode).
pub fn ghosts_for(engine: &Engine, current: u32, spec: &OnionSpec, selection: &[String]) -> Result<Vec<Ghost>, String> {
    spec.validate()?;
    if current >= engine.project.frames.max(1) {
        return Err(format!("frame {current} out of range"));
    }
    let keyed = if spec.mode == OnionMode::Keyed {
        keyed_frames(engine, &effective_filter(spec, selection).unwrap_or_default())
    } else {
        Vec::new()
    };
    Ok(ghost_list(spec, current, engine.project.frames, &keyed))
}

fn onion_groups(engine: &Engine, current: u32, ghosts: &[Ghost], spec: &OnionSpec, filter: Option<&[String]>) -> String {
    let mut s = String::new();
    for g in ghosts {
        let body = crate::svg::ghost_fragment(engine, g.frame as f32, &g.tint, spec.outline, filter, if spec.moving_only { Some(current as f32) } else { None });
        let _ = writeln!(s, r#"<g data-frame="{}" data-side="{}" opacity="{:.3}">"#, g.frame, if g.side == Side::Before { "before" } else { "after" }, g.opacity);
        s.push_str(&body);
        s.push_str("</g>\n");
    }
    s
}

/// Ghost composition: a transparent SVG of the project's size, one `<g opacity>` per ghost, reusing the SVG
/// renderer's geometry code (flat tint, no gradients, masks or blend modes).
pub fn onion_svg(engine: &Engine, current: u32, spec: &OnionSpec, selection: &[String]) -> Result<String, String> {
    let ghosts = ghosts_for(engine, current, spec, selection)?;
    let filter = effective_filter(spec, selection);
    let p = &engine.project;
    let mut s = String::new();
    let _ = writeln!(s, r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">"#, w = p.width, h = p.height);
    s.push_str(&onion_groups(engine, current, &ghosts, spec, filter.as_deref()));
    s.push_str("</svg>\n");
    Ok(s)
}

/// The normal SVG render of `current` with the ghosts drawn over it (an exportable still).
pub fn onion_composite_svg(engine: &Engine, current: u32, spec: &OnionSpec, selection: &[String]) -> Result<String, String> {
    let ghosts = ghosts_for(engine, current, spec, selection)?;
    let filter = effective_filter(spec, selection);
    let base = crate::svg::render_frame(engine, current as f32, &crate::svg::RenderOptions::default());
    let cut = base.rfind("</svg>").ok_or("renderer produced no closing tag")?;
    Ok(format!("{}{}</svg>\n", &base[..cut], onion_groups(engine, current, &ghosts, spec, filter.as_deref())))
}

// ================================================================================================
// Motion trails
// ================================================================================================

pub const MAX_TRAIL_FRAMES: u32 = 5000;
pub const MAX_TRAIL_TARGETS: usize = 64;

/// What a trail follows. Positions are in screen (output) space, camera included, so a trail can be drawn
/// straight over the picture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrailTarget {
    /// The layer's pivot (the point its position places).
    Layer { path: String },
    /// A bone's head or tip.
    Bone { name: String, #[serde(default)] tip: bool },
    /// A control point of a layer (anchor, handle, cage/pin/spine/lattice point, mesh vertex).
    Point { layer: String, point: PointKind },
}

impl TrailTarget {
    /// `layer:PATH`, `bone:NAME` (tip), `bone:NAME:head|tip`, `point:LAYER:KIND:A[:B]` with KIND one of
    /// anchor, handle_in, handle_out (A = path, B = index), cage, pin, spine, lattice (A = deformer, B = index)
    /// or mesh_vertex (A = index).
    pub fn parse(s: &str) -> Result<TrailTarget, String> {
        let (kind, rest) = s.split_once(':').ok_or_else(|| format!("bad target '{s}' (layer:PATH, bone:NAME[:head|tip], point:LAYER:KIND:A[:B])"))?;
        let num = |t: &str| t.parse::<usize>().map_err(|_| format!("bad number '{t}' in target '{s}'"));
        match kind {
            "layer" if !rest.is_empty() => Ok(TrailTarget::Layer { path: rest.to_string() }),
            "bone" if !rest.is_empty() => {
                if let Some((n, end)) = rest.rsplit_once(':') {
                    match end {
                        "tip" => return Ok(TrailTarget::Bone { name: n.into(), tip: true }),
                        "head" => return Ok(TrailTarget::Bone { name: n.into(), tip: false }),
                        _ => {}
                    }
                }
                Ok(TrailTarget::Bone { name: rest.to_string(), tip: true })
            }
            "point" => {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() < 3 || parts[0].is_empty() {
                    return Err(format!("bad point target '{s}'"));
                }
                let (layer, k) = (parts[0].to_string(), parts[1]);
                let a = num(parts[2])?;
                let b = parts.get(3).map(|t| num(t)).transpose()?;
                let need_b = || b.ok_or_else(|| format!("point kind '{k}' needs two numbers in '{s}'"));
                let point = match k {
                    "anchor" => PointKind::Anchor { path: a, index: need_b()? },
                    "handle_in" => PointKind::HandleIn { path: a, index: need_b()? },
                    "handle_out" => PointKind::HandleOut { path: a, index: need_b()? },
                    "cage" => PointKind::Cage { deformer: a, index: need_b()? },
                    "pin" => PointKind::Pin { deformer: a, index: need_b()? },
                    "spine" => PointKind::Spine { deformer: a, index: need_b()? },
                    "lattice" => PointKind::Lattice { deformer: a, index: need_b()? },
                    "mesh_vertex" => PointKind::MeshVertex { index: a },
                    _ => return Err(format!("unknown point kind '{k}' in '{s}'")),
                };
                Ok(TrailTarget::Point { layer, point })
            }
            _ => Err(format!("bad target '{s}' (layer:PATH, bone:NAME[:head|tip], point:LAYER:KIND:A[:B])")),
        }
    }

    pub fn label(&self) -> String {
        match self {
            TrailTarget::Layer { path } => format!("layer:{path}"),
            TrailTarget::Bone { name, tip } => format!("bone:{name}:{}", if *tip { "tip" } else { "head" }),
            TrailTarget::Point { layer, point } => format!("point:{layer}:{}", serde_json::to_string(point).unwrap_or_default()),
        }
    }
}

/// A sampled path: one position per frame, plus the optional smoothed curve through them.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Trail {
    pub target: String,
    pub from: u32,
    pub to: u32,
    /// `from..=to`
    pub frames: Vec<u32>,
    /// Exact evaluated position at each of `frames` (the tick marks).
    pub points: Vec<Vec2>,
    /// Polyline to draw: equals `points` without smoothing, else a Catmull-Rom spline through them.
    pub curve: Vec<Vec2>,
    /// Frames in `from..=to` where the target (or something that moves it) has a key.
    pub keys: Vec<f32>,
}

/// Clamp `[current - before, current + after]` to the clip.
pub fn trail_range(total_frames: u32, current: u32, before: u32, after: u32) -> (u32, u32) {
    let last = total_frames.max(1) - 1;
    (current.saturating_sub(before).min(last), current.saturating_add(after).min(last))
}

/// Catmull-Rom (uniform) spline through `pts`, `sub` samples per segment; passes exactly through every point.
pub fn smooth_curve(pts: &[Vec2], sub: u32) -> Vec<Vec2> {
    if pts.len() < 3 || sub < 2 {
        return pts.to_vec();
    }
    let n = pts.len();
    let at = |i: isize| pts[i.clamp(0, n as isize - 1) as usize];
    let mut out = Vec::with_capacity((n - 1) * sub as usize + 1);
    for i in 0..n - 1 {
        let (p0, p1, p2, p3) = (at(i as isize - 1), at(i as isize), at(i as isize + 1), at(i as isize + 2));
        for k in 0..sub {
            let t = k as f32 / sub as f32;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f32, b: f32, c: f32, d: f32| 0.5 * ((2.0 * b) + (-a + c) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3);
            out.push(Vec2::new(f(p0.x, p1.x, p2.x, p3.x), f(p0.y, p1.y, p2.y, p3.y)));
        }
    }
    out.push(pts[n - 1]);
    out
}

fn bone_chain(engine: &Engine, name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = Some(name.to_string());
    while let Some(n) = cur {
        if out.contains(&n) {
            break;
        }
        cur = engine.project.bones.iter().find(|b| b.name == n).and_then(|b| b.parent.clone());
        out.push(n);
    }
    out
}

/// Key frames of the channels that can move `target`: its own, its ancestors' (group layers, parent bones) and the
/// bone a layer rides. Camera, IK, constraint and driven-key influence are not traced (see ROADMAP).
fn target_keys(engine: &Engine, target: &TrailTarget) -> Vec<f32> {
    let p = &engine.project;
    let mut names: Vec<String> = Vec::new();
    let layer_path = match target {
        TrailTarget::Layer { path } => Some(path.as_str()),
        TrailTarget::Point { layer, .. } => Some(layer.as_str()),
        TrailTarget::Bone { name, .. } => {
            names.extend(bone_chain(engine, name));
            None
        }
    };
    if let Some(path) = layer_path {
        let mut acc = String::new();
        for seg in path.split('/') {
            if !acc.is_empty() {
                acc.push('/');
            }
            acc.push_str(seg);
            names.push(acc.clone());
            if let Some(b) = p.find_layer(&acc).and_then(|l| l.bone.clone()) {
                names.extend(bone_chain(engine, &b));
            }
        }
    }
    let mut out: Vec<f32> = Vec::new();
    for c in &p.channels {
        let Source::Keys { track } = &c.source else { continue };
        let own = names.contains(&c.target);
        let point = matches!(c.prop, Prop::PointDx | Prop::PointDy | Prop::PathMorph | Prop::LatticeDx | Prop::LatticeDy)
            && matches!(target, TrailTarget::Point { .. })
            && layer_path.map(|l| c.target.starts_with(&format!("{l}::"))).unwrap_or(false);
        if own || point {
            out.extend(track.keys.iter().map(|k| k.frame));
        }
    }
    out.sort_by(|a, b| a.total_cmp(b));
    out.dedup();
    out
}

enum Resolved {
    Layer(usize, Vec2),
    Bone(usize, bool),
    Point(String, PointKind),
}

/// Sample trails for several targets with one `evaluate` per frame. `smooth_sub` = 0 or 1 turns smoothing off.
pub fn trails(engine: &Engine, targets: &[TrailTarget], from: u32, to: u32, smooth_sub: u32) -> Result<Vec<Trail>, String> {
    let p = &engine.project;
    if targets.is_empty() {
        return Err("no trail target".into());
    }
    if targets.len() > MAX_TRAIL_TARGETS {
        return Err(format!("at most {MAX_TRAIL_TARGETS} targets"));
    }
    if to < from || to >= p.frames.max(1) {
        return Err(format!("frame range {from}..{to} is outside the clip (0..{})", p.frames.max(1) - 1));
    }
    if to - from + 1 > MAX_TRAIL_FRAMES {
        return Err(format!("range longer than {MAX_TRAIL_FRAMES} frames"));
    }
    if smooth_sub > 32 {
        return Err("smoothing subdivisions must be 0..=32".into());
    }
    let first = engine.evaluate(from as f32);
    let mut res = Vec::new();
    for t in targets {
        res.push(match t {
            TrailTarget::Layer { path } => {
                let id = engine.node_index(path).ok_or_else(|| format!("no layer '{path}'"))?;
                let l = p.find_layer(path).ok_or_else(|| format!("no layer '{path}'"))?;
                Resolved::Layer(id, l.transform.anchor)
            }
            TrailTarget::Bone { name, tip } => Resolved::Bone(p.bones.iter().position(|b| b.name == *name).ok_or_else(|| format!("no bone '{name}'"))?, *tip),
            TrailTarget::Point { layer, point } => {
                engine.node_index(layer).ok_or_else(|| format!("no layer '{layer}'"))?;
                if !crate::pick::layer_points(engine, &first, layer).iter().any(|(k, _)| k == point) {
                    return Err(format!("layer '{layer}' has no such point"));
                }
                Resolved::Point(layer.clone(), point.clone())
            }
        });
    }
    let mut pts: Vec<Vec<Vec2>> = vec![Vec::new(); targets.len()];
    for f in from..=to {
        let fr = if f == from { None } else { Some(engine.evaluate(f as f32)) };
        let fr = fr.as_ref().unwrap_or(&first);
        for (i, r) in res.iter().enumerate() {
            let v = match r {
                Resolved::Layer(id, anchor) => fr.nodes[*id].world.apply(*anchor),
                Resolved::Bone(b, tip) => fr.camera.apply(fr.world[*b].apply(Vec2::new(if *tip { p.bones[*b].length } else { 0.0 }, 0.0))),
                Resolved::Point(layer, kind) => crate::pick::layer_points(engine, fr, layer).into_iter().find(|(k, _)| k == kind).map(|(_, v)| v).ok_or_else(|| format!("point vanished at frame {f}"))?,
            };
            pts[i].push(v);
        }
    }
    Ok(targets
        .iter()
        .zip(pts)
        .map(|(t, points)| {
            let keys = target_keys(engine, t).into_iter().filter(|k| *k >= from as f32 && *k <= to as f32).collect();
            Trail { target: t.label(), from, to, frames: (from..=to).collect(), curve: smooth_curve(&points, smooth_sub), points, keys }
        })
        .collect())
}

/// A tick under a point.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TickHit {
    pub trail: usize,
    pub frame: u32,
    pub dist: f32,
    /// A key exists at (within half a frame of) this tick: moving it retimes that key.
    pub keyed: bool,
}

/// The nearest tick (per-frame position) within `radius` of `p` over all trails; ties go to the lower frame.
/// A caller turns a hit into an edit with the existing key operations (select the keys at `frame`, move them).
pub fn hit_tick(trails: &[Trail], p: Vec2, radius: f32) -> Option<TickHit> {
    let mut best: Option<TickHit> = None;
    for (ti, t) in trails.iter().enumerate() {
        for (i, q) in t.points.iter().enumerate() {
            let d = q.distance(p);
            if d > radius {
                continue;
            }
            let frame = t.frames[i];
            let better = match &best {
                None => true,
                Some(b) => d < b.dist - 1e-6 || ((d - b.dist).abs() <= 1e-6 && frame < b.frame),
            };
            if better {
                best = Some(TickHit { trail: ti, frame, dist: d, keyed: t.keys.iter().any(|k| (k - frame as f32).abs() <= 0.5) });
            }
        }
    }
    best
}

/// Nearest point on the polyline through the ticks, as a fractional frame (linear between neighbours).
pub fn hit_path(trails: &[Trail], p: Vec2, radius: f32) -> Option<(usize, f32, f32)> {
    let mut best: Option<(usize, f32, f32)> = None;
    for (ti, t) in trails.iter().enumerate() {
        for i in 0..t.points.len().saturating_sub(1) {
            let (a, b) = (t.points[i], t.points[i + 1]);
            let ab = b - a;
            let l2 = ab.x * ab.x + ab.y * ab.y;
            let u = if l2 < 1e-12 { 0.0 } else { (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0) };
            let q = a + ab * u;
            let d = q.distance(p);
            if d <= radius && best.map(|x| d < x.2).unwrap_or(true) {
                best = Some((ti, t.frames[i] as f32 + u, d));
            }
        }
    }
    best
}

const TRAIL_COLORS: [&str; 6] = ["#ff5d8f", "#3cc8c8", "#ffb454", "#8f7bff", "#7ee08a", "#ff7f50"];

/// Trails as a transparent SVG overlay of the project's size: curve, a dot per frame, a diamond per key and a ring
/// at `current`.
pub fn trails_svg(engine: &Engine, trails: &[Trail], current: Option<u32>) -> String {
    let p = &engine.project;
    let mut s = String::new();
    let _ = writeln!(s, r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">"#, w = p.width, h = p.height);
    for (i, t) in trails.iter().enumerate() {
        let c = TRAIL_COLORS[i % TRAIL_COLORS.len()];
        let poly: Vec<String> = t.curve.iter().map(|q| format!("{:.2},{:.2}", q.x, q.y)).collect();
        let _ = writeln!(s, r#"<g data-trail="{}" fill="{c}" stroke="{c}">"#, crate::svg::esc(&t.target));
        let _ = writeln!(s, r#"<polyline points="{}" fill="none" stroke-width="1.5" vector-effect="non-scaling-stroke"/>"#, poly.join(" "));
        for (q, f) in t.points.iter().zip(&t.frames) {
            let keyed = t.keys.iter().any(|k| (k - *f as f32).abs() <= 0.5);
            if keyed {
                let _ = writeln!(s, r#"<path data-frame="{f}" d="M{:.2} {:.2}l4 4l-4 4l-4 -4z" stroke="none"/>"#, q.x, q.y - 4.0);
            } else {
                let _ = writeln!(s, r#"<circle data-frame="{f}" cx="{:.2}" cy="{:.2}" r="1.8" stroke="none"/>"#, q.x, q.y);
            }
            if current == Some(*f) {
                let _ = writeln!(s, r#"<circle cx="{:.2}" cy="{:.2}" r="6" fill="none" stroke-width="1.5" vector-effect="non-scaling-stroke"/>"#, q.x, q.y);
            }
        }
        s.push_str("</g>\n");
    }
    s.push_str("</svg>\n");
    s
}

/// JSON form used by the CLI and the studio (positions rounded to 1/100).
pub fn trail_json(t: &Trail) -> serde_json::Value {
    let r = |v: f32| (v as f64 * 100.0).round() / 100.0;
    let arr = |v: &[Vec2]| v.iter().map(|q| serde_json::json!([r(q.x), r(q.y)])).collect::<Vec<_>>();
    serde_json::json!({"target": t.target, "from": t.from, "to": t.to, "frames": t.frames, "points": arr(&t.points), "curve": arr(&t.curve), "keys": t.keys})
}


// ================================================================================================
// Persisted studio settings
// ================================================================================================

/// Trail display settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TrailSettings {
    pub enabled: bool,
    /// Frames shown before / after the playhead.
    pub before: u32,
    pub after: u32,
    /// Spline subdivisions per frame (0 = straight segments between ticks).
    pub smooth: u32,
    /// Draw a dot per frame (key frames always get a diamond).
    pub ticks: bool,
}

impl Default for TrailSettings {
    fn default() -> Self {
        TrailSettings { enabled: false, before: 12, after: 12, smooth: 4, ticks: true }
    }
}

/// Scrub-proxy settings: while the playhead is dragged the studio asks for `level` and returns to full quality on release.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProxySettings {
    pub enabled: bool,
    pub level: u8,
    /// `vector` (simplified SVG) or `raster` (low-resolution bitmap).
    pub kind: String,
    /// While scrubbing also draw a reduced-detail copy of the project (`lod::build`, level chosen by
    /// `lod::pick_level`) whose declared deviation stays within `lod_px` screen pixels.
    pub lod: bool,
    /// Largest deviation (screen pixels) the reduced-detail copy may have.
    pub lod_px: f32,
}

impl Default for ProxySettings {
    fn default() -> Self {
        ProxySettings { enabled: true, level: 2, kind: "vector".into(), lod: false, lod_px: 1.0 }
    }
}

/// Everything the overlay panel stores; lives in the user's UI config, never in the project.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OverlaySettings {
    pub onion: OnionSpec,
    pub trails: TrailSettings,
    pub proxy: ProxySettings,
}

impl OverlaySettings {
    pub fn validate(&self) -> Result<(), String> {
        let mut e = Vec::new();
        if let Err(x) = self.onion.validate() {
            e.push(format!("onion: {x}"));
        }
        let t = &self.trails;
        if t.before > 500 || t.after > 500 {
            e.push("trails: at most 500 frames before / after".into());
        }
        if t.smooth > 32 {
            e.push("trails: smooth must be 0..=32".into());
        }
        if crate::scrub::check_level(self.proxy.level).is_err() {
            e.push(format!("proxy: level must be 0..={}", crate::scrub::MAX_LEVEL));
        }
        if !(self.proxy.lod_px.is_finite() && self.proxy.lod_px >= 0.0 && self.proxy.lod_px <= 16.0) {
            e.push("proxy: lod_px must be between 0 and 16 pixels".into());
        }
        if self.proxy.kind != "vector" && self.proxy.kind != "raster" {
            e.push("proxy: kind must be 'vector' or 'raster'".into());
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e.join("; "))
        }
    }

    /// Merge a partial update (`{"onion": {"before": 3}, ...}`): sections are merged field by field, then validated.
    pub fn merged(&self, patch: &serde_json::Value) -> Result<OverlaySettings, String> {
        let obj = patch.as_object().ok_or("overlay settings must be a JSON object")?;
        let mut cur = serde_json::to_value(self).map_err(|e| e.to_string())?;
        for (k, v) in obj {
            let slot = cur.get_mut(k.as_str()).ok_or_else(|| format!("unknown overlay section '{k}'"))?;
            let vo = v.as_object().ok_or_else(|| format!("section '{k}' must be an object"))?;
            for (f, x) in vo {
                slot.as_object_mut().ok_or("bad section")?.insert(f.clone(), x.clone());
            }
        }
        let next: OverlaySettings = serde_json::from_value(cur).map_err(|e| e.to_string())?;
        next.validate()?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    fn eng() -> Engine {
        Engine::new(demo::project()).unwrap()
    }

    #[test]
    fn onion_skin_differs_from_plain_frame_and_keeps_size() {
        let e = eng();
        let o = OnionOptions { scale: 0.3, ..Default::default() };
        let (w, h, img) = onion_skin(&e, 12, &o).unwrap();
        let (pw, ph, plain) = render_rgba(&e, 12.0, &RasterOptions { scale: 0.3, show_bones: false, transparent: false });
        assert_eq!((w, h), (pw, ph));
        assert_ne!(img, plain, "ghosts change the picture");
        // first frame has no past ghosts but still renders
        assert!(onion_skin(&e, 0, &o).is_ok());
        assert!(onion_skin(&e, 12, &OnionOptions { step: 0, ..o.clone() }).is_err());
        assert!(onion_skin(&e, 12, &OnionOptions { opacity: 2.0, ..o }).is_err());
    }

    #[test]
    fn motion_trail_follows_the_bone() {
        let e = eng();
        let t = motion_trail(&e, "hand", true, 0, 23).unwrap();
        assert_eq!(t.len(), 24);
        let spread = t.iter().map(|p| p.distance(t[0])).fold(0.0, f32::max);
        assert!(spread > 5.0, "the hand moves: {spread}");
        assert!(motion_trail(&e, "ghost", true, 0, 5).is_err());
        assert!(motion_trail(&e, "hand", true, 5, 1).is_err());
        let svg = trail_svg(&e, &t, "#ff0000");
        assert!(svg.contains("<polyline") && svg.matches("<circle").count() == 24);
    }

    // ---- SVG onion skin ----

    fn spec(f: impl FnOnce(&mut OnionSpec)) -> OnionSpec {
        let mut s = OnionSpec { enabled: true, ..Default::default() };
        f(&mut s);
        s
    }

    #[test]
    fn ghost_descriptors_relative_numbers() {
        let s = spec(|s| {
            s.before = 3;
            s.after = 2;
            s.step = 2;
            s.opacity = 0.5;
            s.falloff = 0.8;
        });
        let g = ghost_list(&s, 10, 48, &[]);
        let fr: Vec<(u32, Side, u32)> = g.iter().map(|g| (g.frame, g.side, g.order)).collect();
        // farthest first on each side, past before future
        assert_eq!(fr, vec![(4, Side::Before, 3), (6, Side::Before, 2), (8, Side::Before, 1), (14, Side::After, 2), (12, Side::After, 1)]);
        let op: Vec<f32> = g.iter().map(|g| g.opacity).collect();
        // before: orders 1..3 of 3 -> 0.5, 0.5*(1-0.8*0.5)=0.3, 0.5*0.2=0.1 ; after: 0.5, 0.1
        for (a, b) in op.iter().zip([0.1, 0.3, 0.5, 0.1, 0.5]) {
            assert!((a - b).abs() < 1e-6, "{op:?}");
        }
        assert_eq!(g[0].tint, "#ff4646");
        assert_eq!(g[4].tint, "#3cc86e");
    }

    #[test]
    fn ghosts_are_clipped_to_the_clip() {
        let s = spec(|s| {
            s.before = 3;
            s.after = 3;
        });
        let g = ghost_list(&s, 1, 4, &[]);
        let fr: Vec<u32> = g.iter().map(|g| g.frame).collect();
        assert_eq!(fr, vec![0, 3, 2]); // before: only frame 0; after: 2, 3 (farthest first)
        assert!(ghost_list(&s, 0, 1, &[]).is_empty());
    }

    #[test]
    fn falloff_formula() {
        assert_eq!(falloff_opacity(0.4, 0.9, 1, 1), 0.4);
        assert_eq!(falloff_opacity(0.4, 0.5, 1, 5), 0.4);
        assert!((falloff_opacity(0.4, 0.5, 5, 5) - 0.2).abs() < 1e-6);
        assert!((falloff_opacity(0.4, 0.5, 3, 5) - 0.3).abs() < 1e-6);
        assert_eq!(falloff_opacity(0.4, 0.0, 4, 5), 0.4);
    }

    #[test]
    fn keyed_mode_picks_neighbouring_key_frames() {
        let s = spec(|s| {
            s.mode = OnionMode::Keyed;
            s.before = 2;
            s.after = 1;
            s.step = 5; // ignored
        });
        let keys = [0, 6, 12, 20, 30];
        let g = ghost_list(&s, 12, 48, &keys);
        let fr: Vec<(u32, Side)> = g.iter().map(|g| (g.frame, g.side)).collect();
        assert_eq!(fr, vec![(0, Side::Before), (6, Side::Before), (20, Side::After)]);
        // between keys: the current frame is not a key
        let g = ghost_list(&s, 8, 48, &keys);
        assert_eq!(g.iter().map(|g| g.frame).collect::<Vec<_>>(), vec![0, 6, 12]);
        // no keys: no ghosts
        assert!(ghost_list(&s, 8, 48, &[]).is_empty());
    }

    #[test]
    fn keyed_frames_come_from_channels_and_respect_the_filter() {
        let e = eng();
        let all = keyed_frames(&e, &[]);
        assert!(!all.is_empty() && all.windows(2).all(|w| w[0] < w[1]));
        // a filter naming a layer nothing animates leaves no frames
        assert!(keyed_frames(&e, &["no such layer".to_string()]).is_empty());
    }

    #[test]
    fn spec_validation_and_serde() {
        assert!(OnionSpec::default().validate().is_ok());
        assert!(spec(|s| s.before = 17).validate().is_err());
        assert!(spec(|s| s.step = 0).validate().is_err());
        assert!(spec(|s| s.opacity = 1.5).validate().is_err());
        assert!(spec(|s| s.falloff = f32::NAN).validate().is_err());
        assert!(spec(|s| s.tint_after = "nope".into()).validate().unwrap_err().contains("tint_after"));
        let j = serde_json::to_string(&OnionSpec::default()).unwrap();
        assert_eq!(serde_json::from_str::<OnionSpec>(&j).unwrap(), OnionSpec::default());
        assert!(serde_json::from_str::<OnionSpec>("{\"before\":1,\"bogus\":2}").is_err());
        assert_eq!(serde_json::from_str::<OnionSpec>("{\"before\":5}").unwrap().before, 5);
    }

    #[test]
    fn onion_svg_composes_one_group_per_ghost() {
        let e = eng();
        let sp = spec(|s| {
            s.before = 2;
            s.after = 1;
            s.step = 3;
        });
        let svg = onion_svg(&e, 12, &sp, &[]).unwrap();
        assert!(svg.starts_with("<svg") && svg.trim_end().ends_with("</svg>"));
        assert_eq!(svg.matches("<g data-frame=").count(), 3);
        assert!(svg.contains("data-frame=\"6\"") && svg.contains("data-frame=\"9\"") && svg.contains("data-frame=\"15\""));
        assert!(svg.contains("#ff4646") && svg.contains("#3cc86e"));
        assert!(!svg.contains("<linearGradient") && !svg.contains("mix-blend-mode"));
        let ol = onion_svg(&e, 12, &spec(|s| s.outline = true), &[]).unwrap();
        assert!(ol.contains("fill=\"none\" stroke=\"#ff4646\"") && ol.contains("vector-effect"));
        // deterministic
        assert_eq!(svg, onion_svg(&e, 12, &sp, &[]).unwrap());
        assert!(onion_svg(&e, 999, &sp, &[]).is_err());
        assert!(onion_svg(&e, 1, &spec(|s| s.step = 0), &[]).is_err());
    }

    #[test]
    fn moving_only_drops_static_shapes() {
        use crate::anim::{Ease, Track};
        use crate::project::{Channel, Layer, Project};
        let mut p = Project::new("s", 100.0, 100.0, 24.0, 8);
        let bg = Layer::vector("bg", vec![crate::path::VPath::rect("r", 0.0, 0.0, 100.0, 100.0).with_fill("#00ff00")]);
        let mover = Layer::vector("mover", vec![crate::path::VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#ff0000")]);
        p.layers = vec![bg, mover];
        let mut t = Track::new();
        t.set_key(0.0, 0.0, Ease::Linear);
        t.set_key(7.0, 70.0, Ease::Linear);
        p.channels = vec![Channel::keys("mover", Prop::LayerX, t)];
        let e = Engine::new(p).unwrap();
        let moving = onion_svg(&e, 4, &spec(|_| {}), &[]).unwrap();
        let all = onion_svg(&e, 4, &spec(|s| s.moving_only = false), &[]).unwrap();
        // 2 past + 1 future ghost: one shape each when only the mover counts, two when the background is included
        assert_eq!(moving.matches("<path").count(), 3, "{moving}");
        assert_eq!(all.matches("<path").count(), 6);
        assert!(all.contains("#ff4646") && moving.contains("#ff4646"));
        // in a project where nothing moves, nothing ghosts
        let mut q = Project::new("s", 50.0, 50.0, 24.0, 6);
        q.layers = vec![Layer::vector("a", vec![crate::path::VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#ff0000")])];
        let st = Engine::new(q).unwrap();
        assert!(!onion_svg(&st, 3, &spec(|_| {}), &[]).unwrap().contains("<path"));
        assert!(onion_svg(&st, 3, &spec(|s| s.moving_only = false), &[]).unwrap().contains("<path"));
    }

    #[test]
    fn composite_puts_ghosts_over_the_frame() {
        let e = eng();
        let sp = spec(|_| {});
        let c = onion_composite_svg(&e, 12, &sp, &[]).unwrap();
        assert_eq!(c.matches("<svg").count(), 1);
        assert!(c.trim_end().ends_with("</svg>"));
        let frame = crate::svg::render_frame(&e, 12.0, &crate::svg::RenderOptions::default());
        let body_end = frame.rfind("</svg>").unwrap();
        assert!(c.starts_with(&frame[..body_end]), "the frame comes first, ghosts after");
        assert!(c.rfind("data-frame=").unwrap() > body_end - 1);
    }

    #[test]
    fn onion_layer_filter_and_selection_only() {
        let e = eng();
        let all = onion_svg(&e, 12, &spec(|_| {}), &[]).unwrap();
        let arm = onion_svg(&e, 12, &spec(|s| s.layers = vec!["arm".into()]), &[]).unwrap();
        assert!(arm.len() < all.len() && arm.contains("<path"));
        let none = onion_svg(&e, 12, &spec(|s| s.layers = vec!["no such".into()]), &[]).unwrap();
        assert!(!none.contains("<path"));
        // selection_only uses the selection, ignores the fixed list, and falls back to everything when empty
        let sel = onion_svg(&e, 12, &spec(|s| s.selection_only = true), &["arm".to_string()]).unwrap();
        assert_eq!(sel, arm);
        let empty_sel = onion_svg(&e, 12, &spec(|s| s.selection_only = true), &[]).unwrap();
        assert_eq!(empty_sel, all);
        assert_eq!(effective_filter(&spec(|s| s.layers = vec!["x".into()]), &["y".to_string()]), Some(vec!["x".to_string()]));
    }

    // ---- motion trails ----

    use crate::anim::{Ease, Track};
    use crate::project::{Channel, Layer, Project};
    use crate::path::VPath;

    /// Group `g` holds layer `c` at (10, 0). `g` is keyed: x 0 -> 100 and rotation 0 -> 90 degrees over frames 0..10.
    fn parented() -> Engine {
        let mut p = Project::new("t", 200.0, 100.0, 24.0, 12);
        let mut c = Layer::vector("c", vec![VPath::rect("r", -2.0, -2.0, 4.0, 4.0).with_fill("#ff0000")]);
        c.transform.position = Vec2::new(10.0, 0.0);
        p.layers = vec![Layer::group("g", vec![c])];
        let key = |prop, a, b| {
            let mut t = Track::new();
            t.set_key(0.0, a, Ease::Linear);
            t.set_key(10.0, b, Ease::Linear);
            Channel::keys("g", prop, t)
        };
        p.channels = vec![key(Prop::LayerX, 0.0, 100.0), key(Prop::LayerRotation, 0.0, 90.0)];
        Engine::new(p).unwrap()
    }

    fn near(a: Vec2, x: f32, y: f32) -> bool {
        (a.x - x).abs() < 1e-3 && (a.y - y).abs() < 1e-3
    }

    #[test]
    fn layer_trail_follows_the_parent_exactly() {
        let e = parented();
        let t = &trails(&e, &[TrailTarget::Layer { path: "g/c".into() }], 0, 10, 0).unwrap()[0];
        assert_eq!(t.frames.len(), 11);
        // frame f: parent at (10f, 0) rotated 9f degrees; child offset (10, 0) rotates with it
        for f in 0..=10u32 {
            let a = (9.0 * f as f32).to_radians();
            let (x, y) = (10.0 * f as f32 + 10.0 * a.cos(), 10.0 * a.sin());
            assert!(near(t.points[f as usize], x, y), "frame {f}: {:?} vs ({x}, {y})", t.points[f as usize]);
        }
        assert!(near(t.points[0], 10.0, 0.0) && near(t.points[10], 100.0, 10.0) && near(t.points[5], 50.0 + 7.071_068, 7.071_068));
        assert_eq!(t.keys, vec![0.0, 10.0]);
        assert_eq!(t.curve, t.points, "no smoothing -> curve is the ticks");
        // the group's own pivot is just its position
        let g = &trails(&e, &[TrailTarget::Layer { path: "g".into() }], 2, 4, 0).unwrap()[0];
        assert!(near(g.points[0], 20.0, 0.0) && near(g.points[2], 40.0, 0.0));
        assert_eq!(g.keys, Vec::<f32>::new(), "no key inside 2..4");
    }

    #[test]
    fn trail_range_around_the_playhead() {
        assert_eq!(trail_range(12, 5, 3, 2), (2, 7));
        assert_eq!(trail_range(12, 1, 3, 40), (0, 11));
        assert_eq!(trail_range(12, 11, 0, 0), (11, 11));
        assert_eq!(trail_range(0, 0, 3, 3), (0, 0));
    }

    #[test]
    fn bone_tip_and_head_trails() {
        let e = eng();
        let tip = &trails(&e, &[TrailTarget::Bone { name: "hand".into(), tip: true }], 0, 23, 0).unwrap()[0];
        let head = &trails(&e, &[TrailTarget::Bone { name: "hand".into(), tip: false }], 0, 23, 0).unwrap()[0];
        // identical to the older single-bone helper
        let old = motion_trail(&e, "hand", true, 0, 23).unwrap();
        assert_eq!(tip.points, old);
        // tip is one bone length (45) from the head at every frame
        for (a, b) in tip.points.iter().zip(&head.points) {
            assert!((a.distance(*b) - 45.0).abs() < 0.05, "{}", a.distance(*b));
        }
        // keys: the hand's own and the parent chain's
        assert!(!tip.keys.is_empty());
        assert!(trails(&e, &[TrailTarget::Bone { name: "nope".into(), tip: true }], 0, 5, 0).is_err());
    }

    #[test]
    fn point_trail_matches_pick_points() {
        let e = parented();
        let t = TrailTarget::Point { layer: "g/c".into(), point: PointKind::Anchor { path: 0, index: 0 } };
        let tr = &trails(&e, &[t], 0, 10, 0).unwrap()[0];
        // rect corner (-2,-2) in layer space at frame 0: parent identity, child at (10,0) -> (8,-2)
        assert!(near(tr.points[0], 8.0, -2.0), "{:?}", tr.points[0]);
        let bad = TrailTarget::Point { layer: "g/c".into(), point: PointKind::Anchor { path: 0, index: 99 } };
        assert!(trails(&e, &[bad], 0, 3, 0).is_err());
        assert!(trails(&e, &[TrailTarget::Layer { path: "zz".into() }], 0, 3, 0).is_err());
    }

    #[test]
    fn trail_validation() {
        let e = parented();
        let l = TrailTarget::Layer { path: "g".into() };
        assert!(trails(&e, &[], 0, 3, 0).is_err());
        assert!(trails(&e, std::slice::from_ref(&l), 5, 2, 0).is_err());
        assert!(trails(&e, std::slice::from_ref(&l), 0, 12, 0).is_err(), "frame 12 does not exist in a 12-frame clip");
        assert!(trails(&e, std::slice::from_ref(&l), 0, 3, 99).is_err());
        assert!(trails(&e, &vec![l.clone(); MAX_TRAIL_TARGETS + 1], 0, 3, 0).is_err());
        assert_eq!(trails(&e, &[l.clone(), l], 3, 3, 0).unwrap().len(), 2);
    }

    #[test]
    fn target_parsing() {
        assert_eq!(TrailTarget::parse("layer:a/b").unwrap(), TrailTarget::Layer { path: "a/b".into() });
        assert_eq!(TrailTarget::parse("bone:hand").unwrap(), TrailTarget::Bone { name: "hand".into(), tip: true });
        assert_eq!(TrailTarget::parse("bone:hand:head").unwrap(), TrailTarget::Bone { name: "hand".into(), tip: false });
        assert_eq!(TrailTarget::parse("point:g/c:anchor:0:3").unwrap(), TrailTarget::Point { layer: "g/c".into(), point: PointKind::Anchor { path: 0, index: 3 } });
        assert_eq!(TrailTarget::parse("point:m:mesh_vertex:7").unwrap(), TrailTarget::Point { layer: "m".into(), point: PointKind::MeshVertex { index: 7 } });
        for bad in ["", "layer:", "bone:", "x:y", "point:a", "point:a:anchor:1", "point:a:wat:1:2", "point::anchor:1:2", "point:a:anchor:x:2"] {
            assert!(TrailTarget::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn smoothing_passes_through_the_ticks() {
        let pts: Vec<Vec2> = (0..6).map(|i| Vec2::new(i as f32 * 10.0, if i % 2 == 0 { 0.0 } else { 8.0 })).collect();
        let c = smooth_curve(&pts, 4);
        assert_eq!(c.len(), 5 * 4 + 1);
        for (i, p) in pts.iter().enumerate() {
            assert!(near(c[i * 4], p.x, p.y));
        }
        // collinear, evenly spaced ticks stay on the line
        let line: Vec<Vec2> = (0..5).map(|i| Vec2::new(i as f32 * 3.0, i as f32 * 3.0)).collect();
        assert!(smooth_curve(&line, 5).iter().all(|q| (q.x - q.y).abs() < 1e-4));
        // first segment (0,0)->(10,8), middle sample: Catmull-Rom gives y = 4.5, the chord would give 4.0
        let mid = smooth_curve(&pts, 4)[2];
        assert!(near(mid, mid.x, 4.5) && (mid.x - 5.0).abs() < 1.0, "{mid:?}");
        assert_eq!(smooth_curve(&pts[..2], 4), pts[..2].to_vec(), "two points: nothing to smooth");
        assert_eq!(smooth_curve(&pts, 1), pts);
    }

    #[test]
    fn smoothed_trail_keeps_exact_ticks() {
        let e = parented();
        let t = &trails(&e, &[TrailTarget::Layer { path: "g/c".into() }], 0, 10, 5).unwrap()[0];
        assert_eq!(t.curve.len(), 10 * 5 + 1);
        for (i, p) in t.points.iter().enumerate() {
            assert!(near(t.curve[i * 5], p.x, p.y));
        }
        assert_ne!(t.curve, t.points);
    }

    #[test]
    fn tick_hit_testing() {
        let e = parented();
        let ts = trails(&e, &[TrailTarget::Layer { path: "g/c".into() }], 0, 10, 0).unwrap();
        // exactly on frame 5's tick
        let p5 = ts[0].points[5];
        let h = hit_tick(&ts, p5, 3.0).unwrap();
        assert_eq!((h.frame, h.trail, h.keyed), (5, 0, false));
        assert!(h.dist < 1e-4);
        // a tick at a key
        let h0 = hit_tick(&ts, Vec2::new(10.5, 0.5), 3.0).unwrap();
        assert_eq!((h0.frame, h0.keyed), (0, true));
        let h10 = hit_tick(&ts, ts[0].points[10] + Vec2::new(1.0, 1.0), 3.0).unwrap();
        assert_eq!((h10.frame, h10.keyed), (10, true));
        // nothing near
        assert!(hit_tick(&ts, Vec2::new(500.0, 500.0), 3.0).is_none());
        // radius matters
        assert!(hit_tick(&ts, p5 + Vec2::new(0.0, 5.0), 3.0).is_none());
        assert!(hit_tick(&ts, p5 + Vec2::new(0.0, 5.0), 6.0).is_some());
        // the path between ticks 4 and 5 gives a fractional frame
        let mid = (ts[0].points[4] + ts[0].points[5]) * 0.5;
        let (ti, f, d) = hit_path(&ts, mid, 2.0).unwrap();
        assert_eq!(ti, 0);
        assert!((f - 4.5).abs() < 1e-3 && d < 1e-3, "{f} {d}");
    }

    #[test]
    fn closest_tick_wins_across_trails() {
        let e = parented();
        let ts = trails(&e, &[TrailTarget::Layer { path: "g".into() }, TrailTarget::Layer { path: "g/c".into() }], 0, 10, 0).unwrap();
        // frame 0: group pivot (0,0), child (10,0)
        assert_eq!(hit_tick(&ts, Vec2::new(1.0, 0.0), 20.0).unwrap().trail, 0);
        assert_eq!(hit_tick(&ts, Vec2::new(9.0, 0.0), 20.0).unwrap().trail, 1);
    }

    #[test]
    fn trails_svg_has_curve_ticks_and_current_marker() {
        let e = parented();
        let ts = trails(&e, &[TrailTarget::Layer { path: "g/c".into() }], 0, 10, 3).unwrap();
        let svg = trails_svg(&e, &ts, Some(4));
        assert!(svg.contains("<polyline") && svg.contains("data-trail=\"layer:g/c\""));
        // 2 keyed diamonds (0 and 10) and 9 plain dots
        assert_eq!(svg.matches("<path data-frame").count(), 2);
        assert_eq!(svg.matches("<circle data-frame").count(), 9);
        assert_eq!(svg.matches("r=\"6\"").count(), 1);
        let j = trail_json(&ts[0]);
        assert_eq!(j["frames"].as_array().unwrap().len(), 11);
        assert_eq!(j["points"][0], serde_json::json!([10.0, 0.0]));
    }

    #[test]
    fn overlay_settings_merge_and_validate() {
        let d = OverlaySettings::default();
        assert!(d.validate().is_ok());
        let m = d.merged(&serde_json::json!({"onion": {"before": 4, "enabled": true}, "trails": {"smooth": 0}})).unwrap();
        assert_eq!((m.onion.before, m.onion.after, m.onion.enabled, m.trails.smooth), (4, 1, true, 0));
        assert_eq!(m.proxy, d.proxy, "untouched section unchanged");
        for bad in [
            serde_json::json!({"onion": {"before": 99}}),
            serde_json::json!({"onion": {"opacity": -1}}),
            serde_json::json!({"onion": {"nope": 1}}),
            serde_json::json!({"proxy": {"level": 9}}),
            serde_json::json!({"proxy": {"kind": "png"}}),
            serde_json::json!({"trails": {"smooth": 99}}),
            serde_json::json!({"trails": {"before": 501}}),
            serde_json::json!({"wat": {}}),
            serde_json::json!({"onion": 3}),
            serde_json::json!([1]),
        ] {
            assert!(d.merged(&bad).is_err(), "{bad}");
        }
        let j = serde_json::to_string(&d).unwrap();
        assert_eq!(serde_json::from_str::<OverlaySettings>(&j).unwrap(), d);
    }

    #[test]
    fn proxy_lod_settings_default_off_validate_and_old_files_load() {
        let d = OverlaySettings::default();
        assert!(!d.proxy.lod && d.proxy.lod_px == 1.0 && d.validate().is_ok());
        // a settings file written before the LOD options existed still loads, with LOD off
        let old: OverlaySettings = serde_json::from_str(r#"{"proxy": {"enabled": true, "level": 3, "kind": "raster"}}"#).unwrap();
        assert!(!old.proxy.lod && old.proxy.level == 3);
        for bad in [-1.0f32, 17.0, f32::NAN, f32::INFINITY] {
            let mut s = OverlaySettings::default();
            s.proxy.lod_px = bad;
            assert!(s.validate().unwrap_err().contains("lod_px"), "{bad}");
        }
        let mut ok = OverlaySettings::default();
        ok.proxy.lod = true;
        ok.proxy.lod_px = 0.0;
        assert!(ok.validate().is_ok());
        assert!(serde_json::from_str::<OverlaySettings>(r#"{"proxy": {"lod": 1}}"#).is_err());
    }
}
