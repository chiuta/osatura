// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Warp pivots, soft (proportional) scaling and whole-mesh resize (R-063, parts a and b).
//!
//! These are *editing* operations: they read the control points of a deformer (cage vertices,
//! lattice / cloth grid vertices, pin-warp pins, spine points; a plain grid mesh of an image layer
//! gets a per-vertex `cloth` grid on demand), compute new positions and bake them as keyed
//! `cage_dx/dy`, `lattice_dx/dy`, `pin_dx/dy` or `spine_dx/dy` channels at one frame. Nothing is
//! evaluated at run time, so exports, the editor and the engine see ordinary keys. Locked channels
//! are refused, the result is validated by compiling an engine, the input project is never touched.
//!
//! Maths (textbook): about a pivot `c` a point `p` with weight `w` moves to
//! `c + R(w*theta) * S(1 + w (s - 1)) * (p - c) + w * t`, i.e. scale, then rotate, then translate,
//! each scaled by the weight (a weight of 1 is the full transform, 0 leaves the point alone).
//! Selected points have weight 1; with a soft radius every other point gets
//! `falloff(distance to the nearest selected rest position, radius)`, which is 1 at distance 0,
//! non-increasing and 0 from `radius` on. Falloff curves: `linear` `1-t`, `smooth` the cubic
//! smoothstep `1-(3t^2-2t^3)`, `gauss` a Gaussian bell truncated to reach exactly 0 at the radius,
//! `sharp` `(1-t)^2`, `constant` a plateau (1 inside the radius, 0 outside), with `t = d / radius`.
//!
//! Pivots: a point in layer space, the head of a bone in the rest pose (converted to layer space
//! through the layer's static transform chain), the centroid of the selection or the centre of the
//! bounding box of all control points. Layer pivots (`set_layer_pivot`) move the layer's `anchor`
//! and compensate `position`, so the picture does not change but later rotation/scale channels turn
//! about the new pivot.

use crate::anim::{Ease, Track};
use crate::chanmeta;
use crate::engine::Engine;
use crate::math::{Affine2, Vec2};
use crate::project::{Channel, Deformer, LayerKind, Project, Prop, Source};

#[derive(Clone, Debug, PartialEq)]
pub enum Pivot {
    /// A point in layer space.
    Point(Vec2),
    /// Head of a bone in the rest pose.
    Bone(String),
    /// Centroid of the selected control points (current positions).
    Centroid,
    /// Centre of the bounding box of all control points (current positions).
    Center,
}

impl Pivot {
    /// `x,y` | `bone:NAME` | `centroid` | `center`.
    pub fn parse(s: &str) -> Result<Pivot, String> {
        match s.trim() {
            "centroid" => Ok(Pivot::Centroid),
            "center" | "centre" => Ok(Pivot::Center),
            t if t.starts_with("bone:") && t.len() > 5 => Ok(Pivot::Bone(t[5..].to_string())),
            t => {
                let (a, b) = t.split_once(',').ok_or_else(|| format!("bad pivot '{s}' (use x,y | bone:NAME | centroid | center)"))?;
                let n = |q: &str| q.trim().parse::<f32>().ok().filter(|v| v.is_finite()).ok_or_else(|| format!("bad number '{q}'"));
                Ok(Pivot::Point(Vec2::new(n(a)?, n(b)?)))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Falloff {
    Linear,
    Smooth,
    Gauss,
    Sharp,
    Constant,
}

impl Falloff {
    pub fn parse(s: &str) -> Result<Falloff, String> {
        Ok(match s {
            "linear" => Falloff::Linear,
            "smooth" => Falloff::Smooth,
            "gauss" => Falloff::Gauss,
            "sharp" => Falloff::Sharp,
            "constant" => Falloff::Constant,
            _ => return Err(format!("unknown falloff '{s}' (linear, smooth, gauss, sharp, constant)")),
        })
    }
}

/// Weight at distance `d` from the selection for a soft radius `radius` (> 0).
pub fn falloff_weight(kind: Falloff, d: f32, radius: f32) -> f32 {
    if (radius.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !d.is_finite() || d < 0.0 {
        return if d == 0.0 { 1.0 } else { 0.0 };
    }
    if d >= radius {
        return if kind == Falloff::Constant && d == radius { 1.0 } else { 0.0 };
    }
    let t = d / radius;
    match kind {
        Falloff::Linear => 1.0 - t,
        Falloff::Smooth => 1.0 - t * t * (3.0 - 2.0 * t),
        Falloff::Sharp => (1.0 - t) * (1.0 - t),
        Falloff::Gauss => {
            let k = 4.0f32;
            ((-k * t * t).exp() - (-k).exp()) / (1.0 - (-k).exp())
        }
        Falloff::Constant => 1.0,
    }
}

/// Which control points are selected: explicit indices and/or those whose rest position lies in
/// `region = [x, y, w, h]` (layer space, borders included).
#[derive(Clone, Debug, Default)]
pub struct Selection {
    pub indices: Vec<usize>,
    pub region: Option<[f32; 4]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Cage,
    Lattice,
    Pin,
    Spine,
}

/// Control points of one deformer.
#[derive(Clone, Debug)]
pub struct Controls {
    kind: Kind,
    cols: usize,
    /// Rest positions in layer space.
    pub rest: Vec<Vec2>,
    /// Offsets at the edited frame (keyed value if there is a keyed channel, else the static one).
    pub cur: Vec<Vec2>,
}

impl Controls {
    fn target(&self, layer: &str, name: &str, i: usize) -> (String, Prop, Prop) {
        match self.kind {
            Kind::Cage => (format!("{layer}::{name}::{i}"), Prop::CageDx, Prop::CageDy),
            Kind::Pin => (format!("{layer}::{name}::{i}"), Prop::PinDx, Prop::PinDy),
            Kind::Spine => (format!("{layer}::{name}::{i}"), Prop::SpineDx, Prop::SpineDy),
            Kind::Lattice => (format!("{layer}::{name}::{},{}", i % (self.cols + 1), i / (self.cols + 1)), Prop::LatticeDx, Prop::LatticeDy),
        }
    }
    pub fn len(&self) -> usize {
        self.rest.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }
    /// Current positions (rest + offset).
    pub fn positions(&self) -> Vec<Vec2> {
        self.rest.iter().zip(&self.cur).map(|(r, c)| *r + *c).collect()
    }
}

fn grid_points(rect: [f32; 4], cols: usize, rows: usize) -> Vec<Vec2> {
    let mut v = Vec::with_capacity((cols + 1) * (rows + 1));
    for r in 0..=rows {
        for c in 0..=cols {
            v.push(Vec2::new(rect[0] + rect[2] * c as f32 / cols as f32, rect[1] + rect[3] * r as f32 / rows as f32));
        }
    }
    v
}

/// Static transform chain: layer space -> stage at rest (no animation, bones at rest).
fn rest_stage(p: &Project, layer: &str) -> Result<Affine2, String> {
    p.find_layer(layer).ok_or_else(|| format!("no layer at '{layer}'"))?;
    let mut m = Affine2::IDENTITY;
    let mut acc = String::new();
    for part in layer.split('/') {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(part);
        let l = p.find_layer(&acc).ok_or_else(|| format!("no layer at '{acc}'"))?;
        m = m.mul(l.transform.matrix());
    }
    Ok(m)
}

/// Make sure `layer` has a deformer to key and return its name. With `name = None` the layer's
/// only deformer is used; an image layer with a grid mesh and no deformer gets a per-vertex `cloth`
/// grid called `warp`.
fn ensure_deformer(p: &Project, layer: &str, name: Option<&str>) -> Result<(Project, String), String> {
    let lay = p.find_layer(layer).ok_or_else(|| format!("no layer at '{layer}'"))?;
    let chosen = match name {
        Some(n) => n.to_string(),
        None => match lay.deformers.len() {
            1 => lay.deformers[0].name().to_string(),
            0 => "warp".to_string(),
            _ => return Err(format!("layer '{layer}' has several deformers; pick one with deformer=NAME")),
        },
    };
    if lay.deformers.iter().any(|d| d.name() == chosen) {
        return Ok((p.clone(), chosen));
    }
    match &lay.kind {
        LayerKind::Image { mesh: Some(m), .. } if m.free.is_none() => {
            let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
            let id = e.node_index(layer).ok_or("layer not found")?;
            let u = e.link_unit(id).ok_or("layer has no mesh")?;
            let v = &u.paths[0];
            let (cols, rows) = (m.cols.max(1), m.rows.max(1));
            if v.len() != (cols + 1) * (rows + 1) {
                return Err("mesh size mismatch".into());
            }
            let (a, b) = (v[0], v[v.len() - 1]);
            let mut q = p.clone();
            let l = q.find_layer_mut(layer).unwrap();
            l.deformers.push(Deformer::Cloth { name: chosen.clone(), cols, rows, rect: [a.x, a.y, b.x - a.x, b.y - a.y], offsets: vec![] });
            Ok((q, chosen))
        }
        LayerKind::Image { mesh: Some(_), .. } => Err(format!("layer '{layer}' has a free-form mesh: add a cage deformer ('cage') and warp its vertices")),
        LayerKind::Image { mesh: None, .. } => Err(format!("image layer '{layer}' has no mesh: run mesh/auto-skin first or add a cage deformer")),
        _ => Err(format!("layer '{layer}' has no deformer '{chosen}': add one first (cage, lattice, pin, spine)")),
    }
}

fn keyed_value(p: &Project, target: &str, prop: Prop, frame: f32) -> Option<f32> {
    p.channels.iter().find(|c| c.target == target && c.prop == prop).and_then(|c| match &c.source {
        Source::Keys { track } => track.eval(frame),
        _ => None,
    })
}

/// Read the control points of `layer`'s deformer `name` as they are at `frame`.
pub fn controls(p: &Project, layer: &str, name: &str, frame: f32) -> Result<Controls, String> {
    let lay = p.find_layer(layer).ok_or_else(|| format!("no layer at '{layer}'"))?;
    let d = lay.deformers.iter().find(|d| d.name() == name).ok_or_else(|| format!("layer '{layer}' has no deformer '{name}'"))?;
    let (kind, cols, rest, stat): (Kind, usize, Vec<Vec2>, &Vec<Vec2>) = match d {
        Deformer::Cage { rest, offsets, .. } => (Kind::Cage, 0, rest.clone(), offsets),
        Deformer::Pin { pins, offsets, .. } => (Kind::Pin, 0, pins.clone(), offsets),
        Deformer::Spine { rest, offsets, .. } => (Kind::Spine, 0, rest.clone(), offsets),
        Deformer::Lattice { cols, rows, rect, offsets, .. } | Deformer::Cloth { cols, rows, rect, offsets, .. } => {
            if *cols == 0 || *rows == 0 {
                return Err(format!("deformer '{name}' has an empty grid"));
            }
            (Kind::Lattice, *cols, grid_points(*rect, *cols, *rows), offsets)
        }
    };
    let mut c = Controls { kind, cols, rest, cur: vec![] };
    c.cur = (0..c.rest.len())
        .map(|i| {
            let (t, px, py) = c.target(layer, name, i);
            let base = stat.get(i).copied().unwrap_or(Vec2::ZERO);
            Vec2::new(keyed_value(p, &t, px, frame).unwrap_or(base.x), keyed_value(p, &t, py, frame).unwrap_or(base.y))
        })
        .collect();
    Ok(c)
}

fn select(c: &Controls, s: &Selection) -> Result<Vec<usize>, String> {
    let mut v: Vec<usize> = Vec::new();
    for &i in &s.indices {
        if i >= c.len() {
            return Err(format!("control point {i} does not exist (0..{})", c.len() - 1));
        }
        v.push(i);
    }
    if let Some([x, y, w, h]) = s.region {
        if ![x, y, w, h].iter().all(|n| n.is_finite()) || w < 0.0 || h < 0.0 {
            return Err("region must be x,y,w,h with w,h >= 0".into());
        }
        for (i, r) in c.rest.iter().enumerate() {
            if r.x >= x && r.x <= x + w && r.y >= y && r.y <= y + h {
                v.push(i);
            }
        }
    }
    v.sort_unstable();
    v.dedup();
    if v.is_empty() {
        return Err("the selection is empty (give verts= and/or region=)".into());
    }
    Ok(v)
}

/// Weight of every control point: 1 for the selection, `falloff` of the rest-position distance to
/// the nearest selected point otherwise (0 when `radius` is 0).
pub fn selection_weights(rest: &[Vec2], sel: &[usize], radius: f32, kind: Falloff) -> Vec<f32> {
    let mut w = vec![0.0f32; rest.len()];
    for (i, wi) in w.iter_mut().enumerate() {
        if sel.contains(&i) {
            *wi = 1.0;
        } else if radius > 0.0 {
            let d = sel.iter().map(|&s| rest[s].distance(rest[i])).fold(f32::MAX, f32::min);
            *wi = falloff_weight(kind, d, radius);
        }
    }
    w
}

/// The transform applied about the pivot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Xform {
    pub rotate_deg: f32,
    pub scale: Vec2,
    pub translate: Vec2,
}

impl Default for Xform {
    fn default() -> Self {
        Xform { rotate_deg: 0.0, scale: Vec2::new(1.0, 1.0), translate: Vec2::ZERO }
    }
}

impl Xform {
    fn check(&self) -> Result<(), String> {
        let ok = self.rotate_deg.is_finite() && vec_ok(self.scale) && vec_ok(self.translate);
        if !ok {
            return Err("rotate, scale and move must be finite numbers".into());
        }
        if self.scale.x.abs() < 1e-6 || self.scale.y.abs() < 1e-6 {
            return Err("scale must not be zero".into());
        }
        Ok(())
    }
    /// Apply with weight `w` to `p` about `c`.
    pub fn apply(&self, p: Vec2, c: Vec2, w: f32) -> Vec2 {
        let s = Vec2::new(1.0 + (self.scale.x - 1.0) * w, 1.0 + (self.scale.y - 1.0) * w);
        let (sn, cs) = (self.rotate_deg * w).to_radians().sin_cos();
        let r = p - c;
        let r = Vec2::new(r.x * s.x, r.y * s.y);
        c + Vec2::new(r.x * cs - r.y * sn, r.x * sn + r.y * cs) + self.translate * w
    }
}

fn vec_ok(v: Vec2) -> bool {
    v.x.is_finite() && v.y.is_finite()
}

fn resolve_pivot(p: &Project, layer: &str, piv: &Pivot, c: &Controls, sel: &[usize]) -> Result<Vec2, String> {
    let pos = c.positions();
    match piv {
        Pivot::Point(v) if vec_ok(*v) => Ok(*v),
        Pivot::Point(_) => Err("pivot must be finite".into()),
        Pivot::Centroid => {
            let mut s = Vec2::ZERO;
            for &i in sel {
                s += pos[i];
            }
            Ok(s * (1.0 / sel.len().max(1) as f32))
        }
        Pivot::Center => {
            let (mut lo, mut hi) = (Vec2::new(f32::MAX, f32::MAX), Vec2::new(f32::MIN, f32::MIN));
            for q in &pos {
                lo = Vec2::new(lo.x.min(q.x), lo.y.min(q.y));
                hi = Vec2::new(hi.x.max(q.x), hi.y.max(q.y));
            }
            Ok((lo + hi) * 0.5)
        }
        Pivot::Bone(name) => bone_head_in_layer(p, layer, name),
    }
}

/// Head of bone `name` in the rest pose, expressed in `layer`'s own coordinates.
pub fn bone_head_in_layer(p: &Project, layer: &str, name: &str) -> Result<Vec2, String> {
    let bi = p.bones.iter().position(|b| b.name == name).ok_or_else(|| format!("no bone '{name}'"))?;
    let head = p.rest_world()?[bi].apply(Vec2::ZERO);
    let m = rest_stage(p, layer)?;
    let inv = m.inverse().ok_or("the layer transform is not invertible")?;
    Ok(inv.apply(head))
}

/// Key `new` positions (`(index, new offset)`) of a deformer at `frame`.
fn key_changes(p: &Project, layer: &str, name: &str, c: &Controls, changes: &[(usize, Vec2)], frame: f32, ease: Ease) -> Result<Project, String> {
    if changes.is_empty() {
        return Err("nothing to change: the transform leaves every control point where it is".into());
    }
    for &(i, o) in changes {
        let (t, px, py) = c.target(layer, name, i);
        if !vec_ok(o) {
            return Err("computed offsets are not finite".into());
        }
        chanmeta::assert_unlocked(p, &t, px)?;
        chanmeta::assert_unlocked(p, &t, py)?;
    }
    let stat: Vec<Vec2> = match p.find_layer(layer).and_then(|l| l.deformers.iter().find(|d| d.name() == name)) {
        Some(Deformer::Cage { offsets, .. }) | Some(Deformer::Pin { offsets, .. }) | Some(Deformer::Spine { offsets, .. }) | Some(Deformer::Lattice { offsets, .. }) | Some(Deformer::Cloth { offsets, .. }) => offsets.clone(),
        None => vec![],
    };
    let mut q = p.clone();
    for &(i, o) in changes {
        let (t, px, py) = c.target(layer, name, i);
        let s0 = stat.get(i).copied().unwrap_or(Vec2::ZERO);
        for (prop, v, base) in [(px, o.x, s0.x), (py, o.y, s0.y)] {
            let existing = q.channels.iter_mut().find(|ch| ch.target == t && ch.prop == prop && matches!(ch.source, Source::Keys { .. }));
            match existing {
                Some(Channel { source: Source::Keys { track }, .. }) => track.set_key(frame, v, ease),
                _ => {
                    let mut tr = Track::new();
                    if frame > 0.0 {
                        tr.set_key(0.0, base, Ease::Linear);
                    }
                    tr.set_key(frame, v, ease);
                    q.channels.push(Channel::keys(&t, prop, tr));
                }
            }
        }
    }
    Engine::new(q.clone()).map_err(|v| v.join("; "))?;
    Ok(q)
}

/// Options shared by the transform operations.
#[derive(Clone, Debug)]
pub struct WarpOptions {
    /// Deformer name; `None` = the layer's only deformer (or an on-demand `warp` grid).
    pub deformer: Option<String>,
    pub selection: Selection,
    pub pivot: Pivot,
    pub xform: Xform,
    /// Soft radius (layer units); 0 = only the selection moves.
    pub radius: f32,
    pub falloff: Falloff,
    pub frame: f32,
    pub ease: Ease,
}

impl Default for WarpOptions {
    fn default() -> Self {
        WarpOptions {
            deformer: None,
            selection: Selection::default(),
            pivot: Pivot::Centroid,
            xform: Xform::default(),
            radius: 0.0,
            falloff: Falloff::Smooth,
            frame: 0.0,
            ease: Ease::Linear,
        }
    }
}

/// Rotate / scale / translate a selection of control points about a pivot (optionally with a soft
/// falloff) and key the result at `o.frame`. Used by `warp-pivot` (hard selection) and
/// `soft-scale` (with a radius).
pub fn warp_pivot(p: &Project, layer: &str, o: &WarpOptions) -> Result<Project, String> {
    if !o.frame.is_finite() || o.frame < 0.0 {
        return Err("frame must be a number >= 0".into());
    }
    if !(o.radius.is_finite() && o.radius >= 0.0) {
        return Err("radius must be a number >= 0".into());
    }
    o.xform.check()?;
    let (q, name) = ensure_deformer(p, layer, o.deformer.as_deref())?;
    let c = controls(&q, layer, &name, o.frame)?;
    let sel = select(&c, &o.selection)?;
    let pivot = resolve_pivot(&q, layer, &o.pivot, &c, &sel)?;
    let w = selection_weights(&c.rest, &sel, o.radius, o.falloff);
    let pos = c.positions();
    let mut changes = Vec::new();
    for i in 0..c.len() {
        if w[i] <= 0.0 {
            continue;
        }
        let np = o.xform.apply(pos[i], pivot, w[i]);
        let off = np - c.rest[i];
        if (off - c.cur[i]).length() > 1e-6 {
            changes.push((i, off));
        }
    }
    key_changes(&q, layer, &name, &c, &changes, o.frame, o.ease)
}

/// What a whole-mesh resize aims at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResizeTo {
    Factor(f32),
    Width(f32),
    Height(f32),
    Size(f32, f32),
}

/// Scale **all** control points of a deformer about the centre of their bounding box. With
/// `keep_aspect` (the default of the CLI) a single factor is applied to both axes: `Width(w)` and
/// `Height(h)` derive it from the current box, `Size(w, h)` is refused unless it has the same
/// aspect ratio (1e-3 relative).
pub fn resize_mesh(p: &Project, layer: &str, deformer: Option<&str>, to: ResizeTo, keep_aspect: bool, frame: f32, ease: Ease) -> Result<Project, String> {
    if !frame.is_finite() || frame < 0.0 {
        return Err("frame must be a number >= 0".into());
    }
    let (q, name) = ensure_deformer(p, layer, deformer)?;
    let c = controls(&q, layer, &name, frame)?;
    let pos = c.positions();
    let (mut lo, mut hi) = (Vec2::new(f32::MAX, f32::MAX), Vec2::new(f32::MIN, f32::MIN));
    for r in &pos {
        lo = Vec2::new(lo.x.min(r.x), lo.y.min(r.y));
        hi = Vec2::new(hi.x.max(r.x), hi.y.max(r.y));
    }
    let (bw, bh) = (hi.x - lo.x, hi.y - lo.y);
    let pos_f = |v: f32| v.is_finite() && v > 0.0;
    let s = match to {
        ResizeTo::Factor(f) if pos_f(f) => Vec2::new(f, f),
        ResizeTo::Width(w) if pos_f(w) && bw > 0.0 => {
            let f = w / bw;
            if keep_aspect { Vec2::new(f, f) } else { Vec2::new(f, 1.0) }
        }
        ResizeTo::Height(h) if pos_f(h) && bh > 0.0 => {
            let f = h / bh;
            if keep_aspect { Vec2::new(f, f) } else { Vec2::new(1.0, f) }
        }
        ResizeTo::Size(w, h) if pos_f(w) && pos_f(h) && bw > 0.0 && bh > 0.0 => {
            let (fx, fy) = (w / bw, h / bh);
            if keep_aspect && (fx - fy).abs() > 1e-3 * fx.max(fy) {
                return Err(format!("size {w}x{h} changes the aspect ratio of the {bw:.2}x{bh:.2} mesh; use width= or height=, or aspect=free"));
            }
            Vec2::new(fx, fy)
        }
        _ => return Err("the resize target must be a positive number and the mesh must have a non-zero extent".into()),
    };
    let centre = (lo + hi) * 0.5;
    let x = Xform { scale: s, ..Default::default() };
    let changes: Vec<(usize, Vec2)> = (0..c.len())
        .map(|i| (i, x.apply(pos[i], centre, 1.0) - c.rest[i]))
        .filter(|(i, off)| (*off - c.cur[*i]).length() > 1e-6)
        .collect();
    key_changes(&q, layer, &name, &c, &changes, frame, ease)
}

/// Move the pivot (`transform.anchor`) of a layer to `pivot` without changing how it looks.
/// Refused when position / rotation / scale / skew are animated by channels (the compensation
/// would only be right at rest).
pub fn set_layer_pivot(p: &Project, layer: &str, pivot: &Pivot) -> Result<Project, String> {
    let lay = p.find_layer(layer).ok_or_else(|| format!("no layer at '{layer}'"))?;
    let animated = p.channels.iter().any(|c| {
        c.target == layer
            && matches!(c.prop, Prop::LayerX | Prop::LayerY | Prop::LayerRotation | Prop::LayerScaleX | Prop::LayerScaleY | Prop::LayerSkew)
    });
    if animated {
        return Err(format!("layer '{layer}' has animated transform channels; the pivot can only be moved on a static transform"));
    }
    for prop in [Prop::LayerX, Prop::LayerY, Prop::LayerRotation, Prop::LayerScaleX, Prop::LayerScaleY, Prop::LayerSkew] {
        chanmeta::assert_unlocked(p, layer, prop)?;
    }
    let q_pt = match pivot {
        Pivot::Point(v) if vec_ok(*v) => *v,
        Pivot::Point(_) => return Err("pivot must be finite".into()),
        Pivot::Bone(n) => bone_head_in_layer(p, layer, n)?,
        _ => return Err("a layer pivot is a point (x,y) or bone:NAME".into()),
    };
    let t = &lay.transform;
    // layer-to-parent matrix without the translation part: L = R * Shear * S
    let lin = Affine2::rotate(t.rotation.to_radians()).mul(Affine2::shear_x(t.skew.to_radians())).mul(Affine2::scale_xy(t.scale.x, t.scale.y));
    let d = q_pt - t.anchor;
    let shift = Vec2::new(lin.a * d.x + lin.c * d.y, lin.b * d.x + lin.d * d.y);
    let mut q = p.clone();
    let l = q.find_layer_mut(layer).unwrap();
    l.transform.anchor = q_pt;
    l.transform.position += shift;
    Engine::new(q.clone()).map_err(|v| v.join("; "))?;
    Ok(q)
}

/// Parse `x,y,w,h`.
pub fn parse_region(s: &str) -> Result<[f32; 4], String> {
    let v: Vec<f32> = s.split(',').map(|t| t.trim().parse::<f32>().map_err(|_| format!("bad number '{t}'"))).collect::<Result<_, _>>()?;
    if v.len() != 4 || !v.iter().all(|n| n.is_finite()) {
        return Err("region is x,y,w,h".into());
    }
    Ok([v[0], v[1], v[2], v[3]])
}

/// Parse `s` or `sx,sy`.
pub fn parse_scale(s: &str) -> Result<Vec2, String> {
    let n = |t: &str| t.trim().parse::<f32>().ok().filter(|v| v.is_finite()).ok_or_else(|| format!("bad number '{t}'"));
    match s.split_once(',') {
        Some((a, b)) => Ok(Vec2::new(n(a)?, n(b)?)),
        None => {
            let v = n(s)?;
            Ok(Vec2::new(v, v))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{Bone, Layer};

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }
    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut pm = tiny_skia::Pixmap::new(w, h).unwrap();
        pm.fill(tiny_skia::Color::from_rgba8(200, 40, 40, 255));
        pm.encode_png().unwrap()
    }
    fn cage_proj() -> Project {
        let mut p = Project::new("w", 400.0, 400.0, 24.0, 20);
        p.bones = vec![Bone::new("hip", None, 50.0, v(300.0, 100.0), 0.0)];
        let mut l = Layer::vector("a", vec![crate::path::VPath::rect("r", 60.0, 60.0, 80.0, 80.0)]);
        l.deformers.push(Deformer::Cage { name: "c".into(), rest: vec![v(0.0, 0.0), v(200.0, 0.0), v(200.0, 200.0), v(0.0, 200.0)], offsets: vec![] });
        p.layers = vec![l];
        p
    }
    fn grid_proj(cols: usize, rows: usize) -> Project {
        let mut p = Project::new("g", 400.0, 400.0, 24.0, 20);
        let mut l = Layer::image_from_png("img", &png(80, 80), v(100.0, 100.0));
        if let LayerKind::Image { mesh, size, .. } = &mut l.kind {
            *size = Some(v(80.0, 80.0));
            *mesh = Some(crate::project::MeshSpec { cols, rows, ..Default::default() });
        }
        p.layers = vec![l];
        p
    }
    fn pos_at(p: &Project, layer: &str, name: &str, frame: f32) -> Vec<Vec2> {
        controls(p, layer, name, frame).unwrap().positions()
    }

    #[test]
    fn pivot_rotation_moves_the_selection_by_the_exact_rotation() {
        let p = cage_proj();
        let o = WarpOptions {
            deformer: Some("c".into()),
            selection: Selection { indices: vec![1, 2], region: None },
            pivot: Pivot::Point(v(100.0, 100.0)),
            xform: Xform { rotate_deg: 90.0, ..Default::default() },
            frame: 5.0,
            ..Default::default()
        };
        let q = warp_pivot(&p, "a", &o).unwrap();
        let pos = pos_at(&q, "a", "c", 5.0);
        // (x,y) -> c + R90 (p - c); y down, rotate(+90): (dx,dy) -> (-dy, dx)
        assert!(pos[1].distance(v(200.0, 200.0)) < 1e-3, "{:?}", pos[1]);
        assert!(pos[2].distance(v(0.0, 200.0)) < 1e-3, "{:?}", pos[2]);
        // unselected vertices are untouched, and the rotation is rigid: distances to the pivot kept
        assert!(pos[0].distance(v(0.0, 0.0)) < 1e-6 && pos[3].distance(v(0.0, 200.0)) < 1e-6);
        for i in [1, 2] {
            assert!((pos[i].distance(v(100.0, 100.0)) - 141.42136).abs() < 1e-3);
        }
        // frame 0 keeps the rest pose (a base key was written), the input project is unchanged
        assert!(pos_at(&q, "a", "c", 0.0)[1].distance(v(200.0, 0.0)) < 1e-6);
        assert!(p.channels.is_empty());
        // and the engine accepts and evaluates it
        Engine::new(q).unwrap().evaluate(5.0);
    }

    #[test]
    fn bone_head_and_centroid_pivots() {
        let p = cage_proj();
        let o = |pv: Pivot| WarpOptions {
            deformer: Some("c".into()),
            selection: Selection { indices: vec![0, 1, 2, 3], region: None },
            pivot: pv,
            xform: Xform { rotate_deg: 180.0, ..Default::default() },
            ..Default::default()
        };
        // rotating the whole square by 180 about the bone head at (300,100) mirrors it through that point
        let q = warp_pivot(&p, "a", &o(Pivot::Bone("hip".into()))).unwrap();
        let pos = pos_at(&q, "a", "c", 0.0);
        assert!(pos[0].distance(v(600.0, 200.0)) < 1e-3, "{:?}", pos[0]);
        // about the centroid it maps the square onto itself
        let q = warp_pivot(&p, "a", &o(Pivot::Centroid)).unwrap();
        let pos = pos_at(&q, "a", "c", 0.0);
        assert!(pos[0].distance(v(200.0, 200.0)) < 1e-3 && pos[2].distance(v(0.0, 0.0)) < 1e-3);
        // the bone head honours the layer transform
        let mut t = p.clone();
        t.layers[0].transform.position = v(50.0, 0.0);
        let h = bone_head_in_layer(&t, "a", "hip").unwrap();
        assert!(h.distance(v(250.0, 100.0)) < 1e-4, "{h:?}");
        assert!(bone_head_in_layer(&t, "a", "nope").is_err());
        assert_eq!(Pivot::parse("bone:hip").unwrap(), Pivot::Bone("hip".into()));
        assert_eq!(Pivot::parse("3, 4").unwrap(), Pivot::Point(v(3.0, 4.0)));
        assert!(Pivot::parse("a,b").is_err() && Pivot::parse("zzz").is_err() && Pivot::parse("bone:").is_err());
    }

    #[test]
    fn falloff_weights_are_monotone_and_bounded() {
        for k in [Falloff::Linear, Falloff::Smooth, Falloff::Gauss, Falloff::Sharp, Falloff::Constant] {
            let mut prev = 2.0f32;
            for i in 0..=240 {
                let d = i as f32 * 0.5; // 0..120 with radius 100
                let w = falloff_weight(k, d, 100.0);
                assert!((0.0..=1.0).contains(&w), "{k:?} {d} {w}");
                assert!(w <= prev + 1e-6, "{k:?} not monotone at {d}: {w} > {prev}");
                prev = w;
            }
            assert_eq!(falloff_weight(k, 0.0, 100.0), 1.0, "{k:?}");
            assert!(falloff_weight(k, 100.5, 100.0) == 0.0, "{k:?}");
        }
        assert!(falloff_weight(Falloff::Linear, 50.0, 100.0) - 0.5 < 1e-6);
        assert!((falloff_weight(Falloff::Smooth, 50.0, 100.0) - 0.5).abs() < 1e-6);
        assert!(Falloff::parse("bogus").is_err());
    }

    #[test]
    fn soft_scale_drags_neighbours_with_decreasing_strength() {
        // 8x1 cloth grid over an 80x80 image: control points on a 10 px pitch
        let p = grid_proj(8, 1);
        let o = WarpOptions {
            selection: Selection { indices: vec![8], region: None }, // right end of the top row
            pivot: Pivot::Point(v(100.0, 100.0)),
            xform: Xform { scale: v(2.0, 1.0), ..Default::default() },
            radius: 40.0,
            falloff: Falloff::Smooth,
            ..Default::default()
        };
        let q = warp_pivot(&p, "img", &o).unwrap(); // auto-creates the 'warp' cloth grid
        let c = controls(&q, "img", "warp", 0.0).unwrap();
        // displacement along x of the top row: monotone non-increasing away from the selection
        let dx: Vec<f32> = (0..=8).map(|i| c.cur[i].x).collect();
        assert!((dx[8] - 80.0).abs() < 1e-3, "selected point doubles its distance: {dx:?}");
        for i in 1..=8 {
            assert!(dx[i] >= dx[i - 1] - 1e-4, "{dx:?}");
        }
        assert!(dx[4] == 0.0 || dx[4] < dx[5], "{dx:?}");
        assert_eq!(dx[0], 0.0);
        // points at 40+ px from the selection (index <= 4) are not affected
        assert!(dx[..=4].iter().all(|d| d.abs() < 1e-6), "{dx:?}");
        // the bottom row is within the radius too (distance 80 > 40: unaffected here)
        assert!(c.cur[9..].iter().all(|d| d.length() < 1e-6));
        // the engine draws the deformed mesh: the selected vertex moved, an unselected far one did not
        let e = Engine::new(q).unwrap();
        let f = e.evaluate(0.0);
        let m = f.nodes[0].image.as_ref().unwrap().mesh.as_ref().unwrap();
        assert!(m.positions[8].distance(v(260.0, 100.0)) < 1e-3, "{:?}", m.positions[8]);
        assert!(m.positions[0].distance(v(100.0, 100.0)) < 1e-3);
    }

    #[test]
    fn selection_by_region_and_weights() {
        let p = grid_proj(4, 4);
        let (q, name) = ensure_deformer(&p, "img", None).unwrap();
        let c = controls(&q, "img", &name, 0.0).unwrap();
        assert_eq!(c.len(), 25);
        let s = select(&c, &Selection { indices: vec![0], region: Some([140.0, 100.0, 40.0, 0.0]) }).unwrap();
        assert_eq!(s, vec![0, 2, 3, 4], "x = 140..180 on the first row plus index 0");
        let w = selection_weights(&c.rest, &s, 40.0, Falloff::Linear);
        assert_eq!(w[0], 1.0);
        assert!((w[1] - 0.5).abs() < 1e-6, "{}", w[1]); // 20 px (the pitch) from selected 0 and 2
        assert!(select(&c, &Selection::default()).is_err());
        assert!(select(&c, &Selection { indices: vec![99], region: None }).is_err());
        assert!(select(&c, &Selection { indices: vec![], region: Some([f32::NAN, 0.0, 1.0, 1.0]) }).is_err());
    }

    #[test]
    fn whole_mesh_resize_keeps_the_aspect_around_the_bbox_centre() {
        let p = grid_proj(4, 2); // 80 x 80 image at (100,100): centre (140,140)
        let q = resize_mesh(&p, "img", None, ResizeTo::Factor(1.5), true, 0.0, Ease::Linear).unwrap();
        let pos = pos_at(&q, "img", "warp", 0.0);
        let (lo, hi) = (pos[0], pos[pos.len() - 1]);
        assert!(lo.distance(v(80.0, 80.0)) < 1e-3 && hi.distance(v(200.0, 200.0)) < 1e-3, "{lo:?} {hi:?}");
        // width target derives the factor and keeps the proportions
        let q = resize_mesh(&p, "img", None, ResizeTo::Width(40.0), true, 0.0, Ease::Linear).unwrap();
        let pos = pos_at(&q, "img", "warp", 0.0);
        assert!((pos[pos.len() - 1].x - pos[0].x - 40.0).abs() < 1e-3 && (pos[pos.len() - 1].y - pos[0].y - 40.0).abs() < 1e-3);
        assert!(((pos[0] + pos[pos.len() - 1]) * 0.5).distance(v(140.0, 140.0)) < 1e-3);
        // free aspect changes only one axis; a size with another aspect is refused when aspect is kept
        let q = resize_mesh(&p, "img", None, ResizeTo::Width(160.0), false, 0.0, Ease::Linear).unwrap();
        let pos = pos_at(&q, "img", "warp", 0.0);
        assert!((pos[pos.len() - 1].y - pos[0].y - 80.0).abs() < 1e-3);
        assert!(resize_mesh(&p, "img", None, ResizeTo::Size(160.0, 80.0), true, 0.0, Ease::Linear).is_err());
        assert!(resize_mesh(&p, "img", None, ResizeTo::Size(160.0, 80.0), false, 0.0, Ease::Linear).is_ok());
        assert!(resize_mesh(&p, "img", None, ResizeTo::Factor(-1.0), true, 0.0, Ease::Linear).is_err());
        assert!(resize_mesh(&p, "img", None, ResizeTo::Factor(f32::NAN), true, 0.0, Ease::Linear).is_err());
        assert!(resize_mesh(&p, "img", None, ResizeTo::Factor(1.0), true, 0.0, Ease::Linear).is_err(), "identity changes nothing");
    }

    #[test]
    fn repeated_edits_compose_and_locks_are_respected() {
        let p = cage_proj();
        let mk = |deg: f32, frame: f32| WarpOptions {
            deformer: Some("c".into()),
            selection: Selection { indices: vec![1], region: None },
            pivot: Pivot::Point(v(100.0, 100.0)),
            xform: Xform { rotate_deg: deg, ..Default::default() },
            frame,
            ..Default::default()
        };
        let q = warp_pivot(&p, "a", &mk(45.0, 10.0)).unwrap();
        let q = warp_pivot(&q, "a", &mk(45.0, 10.0)).unwrap();
        // the second edit starts from the keyed value: 90 degrees in total
        let pos = pos_at(&q, "a", "c", 10.0);
        assert!(pos[1].distance(v(200.0, 200.0)) < 1e-3, "{:?}", pos[1]);
        // locked channel => refusal, project untouched
        let mut l = q.clone();
        crate::chanmeta::set_meta(&mut l, "a::c::1", Some(Prop::CageDx), &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        let err = warp_pivot(&l, "a", &mk(10.0, 10.0)).unwrap_err();
        assert!(err.to_lowercase().contains("lock"), "{err}");
        // validation paths
        assert!(warp_pivot(&p, "a", &WarpOptions { frame: -1.0, ..mk(10.0, 0.0) }).is_err());
        assert!(warp_pivot(&p, "a", &WarpOptions { radius: -3.0, ..mk(10.0, 0.0) }).is_err());
        assert!(warp_pivot(&p, "a", &WarpOptions { xform: Xform { scale: v(0.0, 1.0), ..Default::default() }, ..mk(0.0, 0.0) }).is_err());
        assert!(warp_pivot(&p, "a", &WarpOptions { xform: Xform { rotate_deg: f32::NAN, ..Default::default() }, ..mk(0.0, 0.0) }).is_err());
        assert!(warp_pivot(&p, "zz", &mk(10.0, 0.0)).is_err());
        assert!(warp_pivot(&p, "a", &WarpOptions { deformer: Some("nope".into()), ..mk(10.0, 0.0) }).is_err());
        assert!(warp_pivot(&p, "a", &mk(0.0, 0.0)).is_err(), "a no-op is reported");
    }

    #[test]
    fn lattice_pin_and_free_mesh_cases() {
        let mut p = cage_proj();
        p.layers[0].deformers.push(Deformer::Lattice { name: "lat".into(), cols: 2, rows: 2, rect: [0.0, 0.0, 100.0, 100.0], offsets: vec![] });
        p.layers[0].deformers.push(Deformer::Pin { name: "pins".into(), pins: vec![v(10.0, 10.0), v(90.0, 90.0)], offsets: vec![], radius: 50.0 });
        let o = WarpOptions {
            deformer: Some("lat".into()),
            selection: Selection { indices: vec![8], region: None },
            pivot: Pivot::Point(v(0.0, 0.0)),
            xform: Xform { translate: v(5.0, -5.0), ..Default::default() },
            ..Default::default()
        };
        let q = warp_pivot(&p, "a", &o).unwrap();
        assert!(q.channels.iter().any(|c| c.target == "a::lat::2,2" && c.prop == Prop::LatticeDx));
        assert!(pos_at(&q, "a", "lat", 0.0)[8].distance(v(105.0, 95.0)) < 1e-4);
        let o2 = WarpOptions { deformer: Some("pins".into()), selection: Selection { indices: vec![1], region: None }, ..o.clone() };
        let q = warp_pivot(&p, "a", &o2).unwrap();
        assert!(q.channels.iter().any(|c| c.target == "a::pins::1" && c.prop == Prop::PinDx));
        // several deformers and no name => asks for one
        assert!(warp_pivot(&p, "a", &WarpOptions { deformer: None, ..o.clone() }).unwrap_err().contains("deformer=NAME"));
        // free mesh => explains the alternative
        let mut f = grid_proj(2, 2);
        if let LayerKind::Image { mesh: Some(m), .. } = &mut f.layers[0].kind {
            m.free = Some(crate::project::FreeMesh { vertices: vec![v(100.0, 100.0), v(180.0, 100.0), v(100.0, 180.0)], triangles: vec![[0, 1, 2]], uvs: vec![] });
        }
        assert!(warp_pivot(&f, "img", &o).unwrap_err().contains("cage"));
    }

    #[test]
    fn layer_pivot_keeps_the_picture_and_turns_about_the_new_point() {
        let mut p = cage_proj();
        p.layers[0].transform.rotation = 30.0;
        p.layers[0].transform.scale = v(1.5, 0.8);
        p.layers[0].transform.position = v(20.0, 10.0);
        let before = Engine::new(p.clone()).unwrap().evaluate(0.0).nodes[0].world;
        let q = set_layer_pivot(&p, "a", &Pivot::Point(v(100.0, 100.0))).unwrap();
        let after = Engine::new(q.clone()).unwrap().evaluate(0.0).nodes[0].world;
        for pt in [v(0.0, 0.0), v(60.0, 140.0), v(-30.0, 7.0)] {
            assert!(before.apply(pt).distance(after.apply(pt)) < 1e-3, "{pt:?}");
        }
        // the new pivot is a fixed point of the layer transform
        let c = after.apply(v(100.0, 100.0));
        let mut r = q.clone();
        r.layers[0].transform.rotation = 120.0;
        let rot = Engine::new(r).unwrap().evaluate(0.0).nodes[0].world;
        assert!(rot.apply(v(100.0, 100.0)).distance(c) < 1e-3);
        // bone pivot, animated channels refused, bad pivot kinds refused
        assert!(set_layer_pivot(&p, "a", &Pivot::Bone("hip".into())).is_ok());
        let mut a = p.clone();
        a.channels.push(Channel::keys("a", Prop::LayerRotation, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (5.0, 9.0, Ease::Linear)])));
        assert!(set_layer_pivot(&a, "a", &Pivot::Point(v(1.0, 1.0))).is_err());
        assert!(set_layer_pivot(&p, "a", &Pivot::Centroid).is_err());
        assert!(set_layer_pivot(&p, "zz", &Pivot::Point(v(1.0, 1.0))).is_err());
    }

    #[test]
    fn parsers() {
        assert_eq!(parse_region("1,2,3,4").unwrap(), [1.0, 2.0, 3.0, 4.0]);
        assert!(parse_region("1,2,3").is_err() && parse_region("a,b,c,d").is_err());
        assert_eq!(parse_scale("2").unwrap(), v(2.0, 2.0));
        assert_eq!(parse_scale("2,3").unwrap(), v(2.0, 3.0));
        assert!(parse_scale("x").is_err());
    }
}
