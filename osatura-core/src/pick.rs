// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Canvas picking and selection (pure, testable).
//!
//! Everything works in **canvas space as drawn**: the space of an evaluated [`Frame`] (layer
//! transforms, parent chains, bone attachment, parallax and the camera are all already applied).
//! Layer geometry is taken from the evaluated nodes, so deformed, skinned and animated shapes are
//! hit where they are drawn, not where they were authored.
//!
//! * [`pick`] / [`pick_all`]: hit-test one point. Candidates are ordered control points of the
//!   layers named in `point_layers`, then bones (capsules), then layers topmost first.
//! * [`marquee`]: rectangle or lasso selection in `bbox`, `contain` or `intersect` mode.
//! * [`Selection`]: ordered, duplicate-free set with replace / add / toggle / subtract updates.
//! * [`box_frame`] / `selection_points`: the selection's bounds, oriented for a single layer.
//!
//! Limits: layer masks, blend modes and image alpha are ignored (an image is hit by its box or,
//! with a mesh, by its deformed triangles); dashes and variable stroke width are approximated by
//! the maximum width; fills use the non-zero rule (what the renderers use) unless
//! [`FillRule::EvenOdd`] is asked for.

use crate::engine::{Engine, Frame, NodeKind};
use crate::math::{Affine2, Vec2};
use crate::project::{Deformer, LayerKind};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// Targets
// ---------------------------------------------------------------------------------------------

/// A control point that can be picked and dragged.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PointKind {
    /// Anchor `index` of path number `path` of a vector layer.
    Anchor { path: usize, index: usize },
    HandleIn { path: usize, index: usize },
    HandleOut { path: usize, index: usize },
    /// Cage vertex / pin / spine point / lattice control point `index` of deformer number `deformer`
    /// (lattice: row-major, `row * (cols + 1) + col`).
    Cage { deformer: usize, index: usize },
    Pin { deformer: usize, index: usize },
    Spine { deformer: usize, index: usize },
    Lattice { deformer: usize, index: usize },
    /// Vertex of a free-form image mesh.
    MeshVertex { index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    Layer { path: String },
    Bone { name: String },
    Point { layer: String, point: PointKind },
}

impl Target {
    pub fn layer(path: &str) -> Target {
        Target::Layer { path: path.to_string() }
    }
    pub fn bone(name: &str) -> Target {
        Target::Bone { name: name.to_string() }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

/// What a hit on a layer inside groups selects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupMode {
    /// The leaf layer that was hit (the "enter group" behaviour).
    #[default]
    Leaf,
    /// The top-level ancestor (groups behave like single objects).
    Root,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PickOptions {
    /// Extra slack in canvas units: added to a stroke's half width, to a fill's outline (a click
    /// within this distance outside a fill still hits it) and to the bone capsule radius.
    pub tolerance: f32,
    /// Base capsule radius of a bone, canvas units; the hit radius is `bone_radius + tolerance`.
    pub bone_radius: f32,
    /// Pick radius of control points, canvas units.
    pub point_radius: f32,
    pub layers: bool,
    pub bones: bool,
    /// Layers whose control points (anchors, handles, deformer points, mesh vertices) are pickable.
    pub point_layers: Vec<String>,
    /// Locked layer paths; a lock covers the layer's descendants. Locked layers are skipped, so a
    /// click passes through to whatever lies below.
    pub locked: Vec<String>,
    pub locked_bones: Vec<String>,
    /// Do not honour the persistent `Layer::locked` flag (the paths in `locked` still count).
    pub ignore_layer_locks: bool,
    pub group_mode: GroupMode,
    pub fill_rule: FillRule,
    /// Ignore layers whose evaluated opacity is 0.
    pub skip_transparent: bool,
    /// Allow tracing-aid (`reference`) layers.
    pub include_reference: bool,
}

impl Default for PickOptions {
    fn default() -> Self {
        PickOptions {
            tolerance: 3.0,
            bone_radius: 4.0,
            point_radius: 7.0,
            layers: true,
            bones: true,
            point_layers: Vec::new(),
            locked: Vec::new(),
            locked_bones: Vec::new(),
            ignore_layer_locks: false,
            group_mode: GroupMode::Leaf,
            fill_rule: FillRule::NonZero,
            skip_transparent: true,
            include_reference: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub target: Target,
    /// 0 for fills and boxes; otherwise distance to the stroke / bone axis / point.
    pub distance: f32,
}

// ---------------------------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------------------------

/// A flattened outline in canvas space.
#[derive(Clone, Debug)]
pub struct Shape {
    pub pts: Vec<Vec2>,
    pub closed: bool,
    pub filled: bool,
    /// Half stroke width in canvas units (0 = no stroke).
    pub half_w: f32,
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

/// Point-in-polygon for a (possibly self-intersecting) ring.
pub fn point_in_poly(p: Vec2, poly: &[Vec2], rule: FillRule) -> bool {
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut winding = 0i32;
    let mut crossings = 0u32;
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a.y <= p.y) != (b.y <= p.y) {
            let t = (p.y - a.y) / (b.y - a.y);
            let x = a.x + t * (b.x - a.x);
            if x > p.x {
                crossings += 1;
                winding += if b.y > a.y { 1 } else { -1 };
            }
        }
    }
    match rule {
        FillRule::NonZero => winding != 0,
        FillRule::EvenOdd => crossings % 2 == 1,
    }
}

fn dist_seg(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    crate::rig::dist_to_segment(p, a, b)
}

fn seg_cross(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let (r, s) = (b - a, d - c);
    let den = cross(r, s);
    if den.abs() < 1e-12 {
        return false;
    }
    let t = cross(c - a, s) / den;
    let u = cross(c - a, r) / den;
    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

fn seg_seg_dist(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> f32 {
    if seg_cross(a, b, c, d) {
        return 0.0;
    }
    dist_seg(a, c, d).min(dist_seg(b, c, d)).min(dist_seg(c, a, b)).min(dist_seg(d, a, b))
}

fn aabb(pts: &[Vec2]) -> Option<(Vec2, Vec2)> {
    let f = *pts.first()?;
    let (mut lo, mut hi) = (f, f);
    for p in pts {
        lo = Vec2::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Vec2::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    Some((lo, hi))
}

fn det_scale(m: &Affine2) -> f32 {
    (m.a * m.d - m.b * m.c).abs().sqrt()
}

fn contains_path(list: &[String], path: &str) -> bool {
    list.iter().any(|l| l == path || path.starts_with(&format!("{l}/")))
}

fn paint_visible(p: &crate::style::Paint) -> bool {
    !matches!(p, crate::style::Paint::Solid(c) if c.a <= 0.0)
}

/// Outlines of one node's own geometry in canvas space (empty for groups and switches).
pub fn node_shapes(frame: &Frame, id: usize) -> Vec<Shape> {
    let node = &frame.nodes[id];
    let mut out = Vec::new();
    match node.kind {
        NodeKind::Vector => {
            let k = det_scale(&node.world);
            for p in &node.paths {
                let filled = p.fill.as_ref().map(paint_visible).unwrap_or(false);
                let stroked = p.stroke.as_ref().map(|s| s.width > 0.0 && paint_visible(&s.paint)).unwrap_or(false);
                if !filled && !stroked {
                    continue;
                }
                let pts: Vec<Vec2> = p.flatten(0.25).into_iter().map(|q| node.world.apply(q)).collect();
                if pts.is_empty() {
                    continue;
                }
                let half_w = match (&p.stroke, stroked) {
                    (Some(s), true) => {
                        let f = s.width_profile.iter().map(|w| w[1]).fold(1.0f32, f32::max);
                        0.5 * s.width * f * k
                    }
                    _ => 0.0,
                };
                out.push(Shape { pts, closed: p.closed, filled: filled && pts_len_ok(p.anchors.len()), half_w });
            }
        }
        NodeKind::Image => {
            if let Some(img) = &node.image {
                match &img.mesh {
                    Some(m) if !m.tris.is_empty() => {
                        for t in &m.tris {
                            let pts: Vec<Vec2> = t.iter().filter_map(|&i| m.positions.get(i as usize)).map(|q| node.world.apply(*q)).collect();
                            if pts.len() == 3 {
                                out.push(Shape { pts, closed: true, filled: true, half_w: 0.0 });
                            }
                        }
                    }
                    _ => {
                        let (o, s) = (img.origin, img.size);
                        let pts = [o, o + Vec2::new(s.x, 0.0), o + s, o + Vec2::new(0.0, s.y)].iter().map(|q| node.world.apply(*q)).collect();
                        out.push(Shape { pts, closed: true, filled: true, half_w: 0.0 });
                    }
                }
            }
        }
        _ => {}
    }
    out
}

fn pts_len_ok(n: usize) -> bool {
    n >= 3
}

fn shape_hit(sh: &Shape, p: Vec2, tol: f32, rule: FillRule) -> Option<f32> {
    if sh.filled && point_in_poly(p, &sh.pts, rule) {
        return Some(0.0);
    }
    if sh.filled && tol > 0.0 && sh.pts.len() >= 2 {
        let n = sh.pts.len();
        let d = (0..n).map(|i| dist_seg(p, sh.pts[i], sh.pts[(i + 1) % n])).fold(f32::INFINITY, f32::min);
        if d <= tol {
            return Some(d);
        }
    }
    if sh.half_w > 0.0 {
        let n = sh.pts.len();
        let segs = if sh.closed { n } else { n.saturating_sub(1) };
        let mut best = f32::INFINITY;
        if n == 1 {
            best = p.distance(sh.pts[0]);
        }
        for i in 0..segs {
            best = best.min(dist_seg(p, sh.pts[i], sh.pts[(i + 1) % n]));
        }
        if best <= sh.half_w + tol {
            return Some((best - sh.half_w).max(0.0));
        }
    }
    None
}

/// Visible nodes in draw order (bottom to top), groups included before their children.
pub fn draw_order(frame: &Frame) -> Vec<usize> {
    fn walk(frame: &Frame, id: usize, out: &mut Vec<usize>) {
        let n = &frame.nodes[id];
        if !n.visible {
            return;
        }
        out.push(id);
        for &c in &n.children {
            walk(frame, c, out);
        }
    }
    let mut out = Vec::new();
    for &r in &frame.roots {
        walk(frame, r, &mut out);
    }
    out
}

fn is_reference(engine: &Engine, path: &str) -> bool {
    engine.project.find_layer(path).map(|l| l.reference).unwrap_or(false)
}

/// Skipped by picking: named in `o.locked`, or flagged `Layer::locked` (itself or a group above it).
fn layer_is_locked(engine: &Engine, o: &PickOptions, path: &str) -> bool {
    contains_path(&o.locked, path) || (!o.ignore_layer_locks && engine.project.is_layer_locked(path))
}

fn selectable(engine: &Engine, frame: &Frame, id: usize, o: &PickOptions) -> bool {
    let n = &frame.nodes[id];
    if layer_is_locked(engine, o, &n.name) {
        return false;
    }
    if o.skip_transparent && n.opacity <= 0.0 {
        return false;
    }
    if !o.include_reference && is_reference(engine, &n.name) {
        return false;
    }
    true
}

/// The target a leaf layer resolves to under `mode`.
pub fn resolve_layer(path: &str, mode: GroupMode) -> Target {
    match mode {
        GroupMode::Leaf => Target::layer(path),
        GroupMode::Root => Target::layer(path.split('/').next().unwrap_or(path)),
    }
}

// ---------------------------------------------------------------------------------------------
// Control points
// ---------------------------------------------------------------------------------------------

/// Canvas-space position of every pickable control point of layer `path`.
pub fn layer_points(engine: &Engine, frame: &Frame, path: &str) -> Vec<(PointKind, Vec2)> {
    let mut out = Vec::new();
    let (Some(id), Some(layer)) = (engine.node_index(path), engine.project.find_layer(path)) else { return out };
    let node = &frame.nodes[id];
    let w = node.world;
    match &layer.kind {
        LayerKind::Vector { .. } => {
            for (pi, p) in node.paths.iter().enumerate() {
                for (ai, a) in p.anchors.iter().enumerate() {
                    out.push((PointKind::Anchor { path: pi, index: ai }, w.apply(a.pos)));
                    if let Some(h) = a.handle_in {
                        out.push((PointKind::HandleIn { path: pi, index: ai }, w.apply(h)));
                    }
                    if let Some(h) = a.handle_out {
                        out.push((PointKind::HandleOut { path: pi, index: ai }, w.apply(h)));
                    }
                }
            }
        }
        LayerKind::Image { mesh: Some(m), .. } if m.free.is_some() => {
            if let Some(mo) = node.image.as_ref().and_then(|i| i.mesh.as_ref()) {
                for (i, q) in mo.positions.iter().enumerate() {
                    out.push((PointKind::MeshVertex { index: i }, w.apply(*q)));
                }
            }
        }
        _ => {}
    }
    for (di, d) in layer.deformers.iter().enumerate() {
        let off = |offsets: &Vec<Vec2>, i: usize| offsets.get(i).copied().unwrap_or(Vec2::ZERO);
        match d {
            Deformer::Cage { rest, offsets, .. } => {
                for (i, r) in rest.iter().enumerate() {
                    out.push((PointKind::Cage { deformer: di, index: i }, w.apply(*r + off(offsets, i))));
                }
            }
            Deformer::Pin { pins, offsets, .. } => {
                for (i, r) in pins.iter().enumerate() {
                    out.push((PointKind::Pin { deformer: di, index: i }, w.apply(*r + off(offsets, i))));
                }
            }
            Deformer::Spine { rest, offsets, .. } => {
                for (i, r) in rest.iter().enumerate() {
                    out.push((PointKind::Spine { deformer: di, index: i }, w.apply(*r + off(offsets, i))));
                }
            }
            Deformer::Lattice { cols, rows, rect, offsets, .. } => {
                for r in 0..=*rows {
                    for c in 0..=*cols {
                        let i = r * (cols + 1) + c;
                        let base = Vec2::new(rect[0] + rect[2] * c as f32 / (*cols).max(1) as f32, rect[1] + rect[3] * r as f32 / (*rows).max(1) as f32);
                        out.push((PointKind::Lattice { deformer: di, index: i }, w.apply(base + off(offsets, i))));
                    }
                }
            }
            Deformer::Cloth { .. } => {}
        }
    }
    out
}

/// Bone endpoints `(head, tip)` in canvas space.
pub fn bone_ends(engine: &Engine, frame: &Frame, name: &str) -> Option<(Vec2, Vec2)> {
    let bi = engine.bone_index(name)?;
    let w = frame.camera.mul(frame.world[bi]);
    Some((w.apply(Vec2::ZERO), w.apply(Vec2::new(engine.project.bones[bi].length, 0.0))))
}

// ---------------------------------------------------------------------------------------------
// Picking
// ---------------------------------------------------------------------------------------------

/// All hits at `p`, best first: control points (nearest first), bones (nearest first), then layers
/// topmost first. Locked and hidden items are skipped.
pub fn pick_all(engine: &Engine, frame: &Frame, p: Vec2, o: &PickOptions) -> Vec<Hit> {
    let mut hits: Vec<Hit> = Vec::new();

    let mut pts: Vec<Hit> = Vec::new();
    for lp in &o.point_layers {
        let Some(id) = engine.node_index(lp) else { continue };
        if !frame.nodes[id].visible || layer_is_locked(engine, o, lp) {
            continue;
        }
        for (kind, pos) in layer_points(engine, frame, lp) {
            let d = pos.distance(p);
            if d <= o.point_radius {
                pts.push(Hit { target: Target::Point { layer: lp.clone(), point: kind }, distance: d });
            }
        }
    }
    pts.sort_by(|a, b| a.distance.total_cmp(&b.distance));
    hits.extend(pts);

    if o.bones {
        let mut bs: Vec<Hit> = Vec::new();
        for b in &engine.project.bones {
            if o.locked_bones.contains(&b.name) {
                continue;
            }
            if let Some((h, t)) = bone_ends(engine, frame, &b.name) {
                let d = dist_seg(p, h, t);
                if d <= o.bone_radius + o.tolerance {
                    bs.push(Hit { target: Target::bone(&b.name), distance: d });
                }
            }
        }
        bs.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        hits.extend(bs);
    }

    if o.layers {
        for &id in draw_order(frame).iter().rev() {
            if !selectable(engine, frame, id, o) {
                continue;
            }
            let best = node_shapes(frame, id).iter().filter_map(|s| shape_hit(s, p, o.tolerance, o.fill_rule)).fold(None::<f32>, |a, d| Some(a.map_or(d, |x| x.min(d))));
            if let Some(d) = best {
                let t = resolve_layer(&frame.nodes[id].name, o.group_mode);
                if !hits.iter().any(|h| h.target == t) {
                    hits.push(Hit { target: t, distance: d });
                }
            }
        }
    }
    hits
}

/// The best hit at `p`, if any.
pub fn pick(engine: &Engine, frame: &Frame, p: Vec2, o: &PickOptions) -> Option<Hit> {
    pick_all(engine, frame, p, o).into_iter().next()
}

/// Evaluate `frame_no` and pick (convenience for the CLI and server).
pub fn pick_at(engine: &Engine, frame_no: f32, p: Vec2, o: &PickOptions) -> Option<Hit> {
    pick(engine, &engine.evaluate(frame_no), p, o)
}

// ---------------------------------------------------------------------------------------------
// Marquee
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarqueeMode {
    /// The item's bounding box touches the region.
    Bbox,
    /// All of the item's geometry lies inside the region.
    Contain,
    /// Some of the item's geometry touches the region.
    #[default]
    Intersect,
}

impl MarqueeMode {
    pub fn parse(s: &str) -> Option<MarqueeMode> {
        match s {
            "bbox" => Some(MarqueeMode::Bbox),
            "contain" => Some(MarqueeMode::Contain),
            "intersect" => Some(MarqueeMode::Intersect),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Region {
    Rect { min: Vec2, max: Vec2 },
    Lasso(Vec<Vec2>),
}

impl Region {
    /// A rectangle from two opposite corners in any order.
    pub fn rect(a: Vec2, b: Vec2) -> Region {
        Region::Rect { min: Vec2::new(a.x.min(b.x), a.y.min(b.y)), max: Vec2::new(a.x.max(b.x), a.y.max(b.y)) }
    }
    fn polygon(&self) -> Vec<Vec2> {
        match self {
            Region::Rect { min, max } => vec![*min, Vec2::new(max.x, min.y), *max, Vec2::new(min.x, max.y)],
            Region::Lasso(p) => p.clone(),
        }
    }
}

fn shape_in_region(sh: &Shape, poly: &[Vec2], rbox: (Vec2, Vec2), mode: MarqueeMode) -> bool {
    match mode {
        MarqueeMode::Contain => sh.pts.iter().all(|q| point_in_poly(*q, poly, FillRule::EvenOdd)),
        MarqueeMode::Bbox => shape_bbox_overlap(sh, rbox),
        MarqueeMode::Intersect => {
            if !shape_bbox_overlap(sh, rbox) {
                return false;
            }
            if sh.pts.iter().any(|q| point_in_poly(*q, poly, FillRule::EvenOdd)) {
                return true;
            }
            let n = sh.pts.len();
            let segs = if sh.closed { n } else { n.saturating_sub(1) };
            let m = poly.len();
            for i in 0..segs {
                let (a, b) = (sh.pts[i], sh.pts[(i + 1) % n]);
                for j in 0..m {
                    if seg_seg_dist(a, b, poly[j], poly[(j + 1) % m]) <= sh.half_w {
                        return true;
                    }
                }
            }
            sh.filled && poly.first().map(|q| point_in_poly(*q, &sh.pts, FillRule::NonZero)).unwrap_or(false)
        }
    }
}

fn shape_bbox_overlap(sh: &Shape, rbox: (Vec2, Vec2)) -> bool {
    let Some((lo, hi)) = aabb(&sh.pts) else { return false };
    let (lo, hi) = (lo - Vec2::new(sh.half_w, sh.half_w), hi + Vec2::new(sh.half_w, sh.half_w));
    lo.x <= rbox.1.x && hi.x >= rbox.0.x && lo.y <= rbox.1.y && hi.y >= rbox.0.y
}

/// Select by region. Returns targets in a stable order (points, else bones then layers in draw
/// order). With `GroupMode::Root` a group matches when any leaf matches (`bbox`, `intersect`) or
/// when every selectable leaf under it matches (`contain`).
pub fn marquee(engine: &Engine, frame: &Frame, region: &Region, mode: MarqueeMode, o: &PickOptions) -> Vec<Target> {
    let poly = region.polygon();
    let Some(rbox) = aabb(&poly) else { return Vec::new() };
    if poly.len() < 3 {
        return Vec::new();
    }
    let mut out: Vec<Target> = Vec::new();

    // control points win when any are inside
    for lp in &o.point_layers {
        let Some(id) = engine.node_index(lp) else { continue };
        if !frame.nodes[id].visible || layer_is_locked(engine, o, lp) {
            continue;
        }
        for (kind, pos) in layer_points(engine, frame, lp) {
            if point_in_poly(pos, &poly, FillRule::EvenOdd) {
                out.push(Target::Point { layer: lp.clone(), point: kind });
            }
        }
    }
    if !out.is_empty() {
        return out;
    }

    if o.bones {
        for b in &engine.project.bones {
            if o.locked_bones.contains(&b.name) {
                continue;
            }
            let Some((h, t)) = bone_ends(engine, frame, &b.name) else { continue };
            let ok = match mode {
                MarqueeMode::Contain => point_in_poly(h, &poly, FillRule::EvenOdd) && point_in_poly(t, &poly, FillRule::EvenOdd),
                MarqueeMode::Bbox => {
                    let (lo, hi) = (Vec2::new(h.x.min(t.x), h.y.min(t.y)), Vec2::new(h.x.max(t.x), h.y.max(t.y)));
                    lo.x <= rbox.1.x && hi.x >= rbox.0.x && lo.y <= rbox.1.y && hi.y >= rbox.0.y
                }
                MarqueeMode::Intersect => {
                    point_in_poly(h, &poly, FillRule::EvenOdd)
                        || point_in_poly(t, &poly, FillRule::EvenOdd)
                        || (0..poly.len()).any(|j| seg_cross(h, t, poly[j], poly[(j + 1) % poly.len()]))
                }
            };
            if ok {
                out.push(Target::bone(&b.name));
            }
        }
    }

    if o.layers {
        // leaf matches
        let mut leaves: Vec<(usize, bool)> = Vec::new(); // (node, matched)
        for id in draw_order(frame) {
            if !selectable(engine, frame, id, o) {
                continue;
            }
            let shapes = node_shapes(frame, id);
            if shapes.is_empty() {
                continue;
            }
            let matched = match mode {
                MarqueeMode::Contain => shapes.iter().all(|s| shape_in_region(s, &poly, rbox, mode)),
                _ => shapes.iter().any(|s| shape_in_region(s, &poly, rbox, mode)),
            };
            leaves.push((id, matched));
        }
        match o.group_mode {
            GroupMode::Leaf => {
                for (id, m) in &leaves {
                    if *m {
                        out.push(Target::layer(&frame.nodes[*id].name));
                    }
                }
            }
            GroupMode::Root => {
                let mut roots: Vec<(String, bool, bool)> = Vec::new(); // (root, any, all)
                for (id, m) in &leaves {
                    let root = frame.nodes[*id].name.split('/').next().unwrap_or("").to_string();
                    match roots.iter_mut().find(|r| r.0 == root) {
                        Some(r) => {
                            r.1 |= *m;
                            r.2 &= *m;
                        }
                        None => roots.push((root, *m, *m)),
                    }
                }
                for (root, any, all) in roots {
                    if (mode == MarqueeMode::Contain && all) || (mode != MarqueeMode::Contain && any) {
                        out.push(Target::Layer { path: root });
                    }
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Selection set
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectMode {
    Replace,
    /// Add missing items.
    Add,
    /// Flip each given item.
    Toggle,
    Subtract,
}

impl SelectMode {
    pub fn parse(s: &str) -> Option<SelectMode> {
        match s {
            "replace" => Some(SelectMode::Replace),
            "add" => Some(SelectMode::Add),
            "toggle" => Some(SelectMode::Toggle),
            "subtract" => Some(SelectMode::Subtract),
            _ => None,
        }
    }
}

/// An ordered set of targets without duplicates (order = order of selection).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub items: Vec<Target>,
}

impl Selection {
    pub fn new() -> Selection {
        Selection::default()
    }
    pub fn from_items(items: Vec<Target>) -> Selection {
        let mut s = Selection::new();
        s.apply(&items, SelectMode::Replace);
        s
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn contains(&self, t: &Target) -> bool {
        self.items.contains(t)
    }
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Update with `hits`. `Replace` with no hits clears the selection (click on empty space).
    pub fn apply(&mut self, hits: &[Target], mode: SelectMode) {
        match mode {
            SelectMode::Replace => {
                self.items.clear();
                for h in hits {
                    if !self.items.contains(h) {
                        self.items.push(h.clone());
                    }
                }
            }
            SelectMode::Add => {
                for h in hits {
                    if !self.items.contains(h) {
                        self.items.push(h.clone());
                    }
                }
            }
            SelectMode::Toggle => {
                for h in hits {
                    match self.items.iter().position(|x| x == h) {
                        Some(i) => {
                            self.items.remove(i);
                        }
                        None => self.items.push(h.clone()),
                    }
                }
            }
            SelectMode::Subtract => self.items.retain(|x| !hits.contains(x)),
        }
    }

    /// Drop items that no longer exist (after undo, delete or a project replacement).
    pub fn prune(&mut self, engine: &Engine) {
        self.items.retain(|t| match t {
            Target::Layer { path } => engine.node_index(path).is_some(),
            Target::Bone { name } => engine.bone_index(name).is_some(),
            Target::Point { layer, .. } => engine.node_index(layer).is_some(),
        });
    }

    /// Layers only.
    pub fn layers(&self) -> Vec<&str> {
        self.items.iter().filter_map(|t| if let Target::Layer { path } = t { Some(path.as_str()) } else { None }).collect()
    }
    pub fn bones(&self) -> Vec<&str> {
        self.items.iter().filter_map(|t| if let Target::Bone { name } = t { Some(name.as_str()) } else { None }).collect()
    }
}

// ---------------------------------------------------------------------------------------------
// Bounds
// ---------------------------------------------------------------------------------------------

/// Canvas-space points that describe a target's extent (for bounds).
pub fn target_points(engine: &Engine, frame: &Frame, t: &Target) -> Vec<Vec2> {
    match t {
        Target::Layer { path } => {
            let prefix = format!("{path}/");
            let mut pts = Vec::new();
            for (id, n) in frame.nodes.iter().enumerate() {
                if &n.name == path || n.name.starts_with(&prefix) {
                    for s in node_shapes(frame, id) {
                        pts.extend(s.pts);
                    }
                }
            }
            pts
        }
        Target::Bone { name } => bone_ends(engine, frame, name).map(|(h, t)| vec![h, t]).unwrap_or_default(),
        Target::Point { layer, point } => layer_points(engine, frame, layer).into_iter().filter(|(k, _)| k == point).map(|(_, p)| p).collect(),
    }
}

/// Axis-aligned bounds of a selection in canvas space.
pub fn selection_bounds(engine: &Engine, frame: &Frame, sel: &Selection) -> Option<(Vec2, Vec2)> {
    let pts: Vec<Vec2> = sel.items.iter().flat_map(|t| target_points(engine, frame, t)).collect();
    aabb(&pts)
}

/// Canvas-space rectangle `[x0, y0, x1, y1]` around the named layers (and their children) at `frame_no`.
/// Unknown names contribute nothing; `None` when nothing is left. Used by the shared session to show a
/// participant's selection to the others (computed on the server, so a client cannot claim other bounds).
pub fn layers_bounds(engine: &Engine, frame_no: f32, paths: &[String]) -> Option<[f32; 4]> {
    if !frame_no.is_finite() {
        return None;
    }
    let max = engine.project.frames.max(1) as f32;
    let fr = engine.evaluate(frame_no.clamp(0.0, max));
    let sel = Selection::from_items(paths.iter().take(64).map(|p| Target::layer(p)).collect());
    let (lo, hi) = selection_bounds(engine, &fr, &sel)?;
    let b = [lo.x, lo.y, hi.x, hi.y];
    b.iter().all(|v| v.is_finite()).then_some(b)
}

/// A parallelogram `origin + s·u + t·v` for `s, t` in `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoxFrame {
    pub origin: Vec2,
    pub u: Vec2,
    pub v: Vec2,
}

impl BoxFrame {
    pub fn from_aabb(lo: Vec2, hi: Vec2) -> BoxFrame {
        BoxFrame { origin: lo, u: Vec2::new(hi.x - lo.x, 0.0), v: Vec2::new(0.0, hi.y - lo.y) }
    }
    /// Unit square to canvas.
    pub fn basis(&self) -> Affine2 {
        Affine2 { a: self.u.x, b: self.u.y, c: self.v.x, d: self.v.y, tx: self.origin.x, ty: self.origin.y }
    }
    pub fn point(&self, s: f32, t: f32) -> Vec2 {
        self.origin + self.u * s + self.v * t
    }
    /// Top-left, top-right, bottom-right, bottom-left.
    pub fn corners(&self) -> [Vec2; 4] {
        [self.point(0.0, 0.0), self.point(1.0, 0.0), self.point(1.0, 1.0), self.point(0.0, 1.0)]
    }
    pub fn centre(&self) -> Vec2 {
        self.point(0.5, 0.5)
    }
    /// `None` when the box has no area.
    pub fn nondegenerate(&self) -> bool {
        cross(self.u, self.v).abs() > 1e-6
    }
    pub fn contains(&self, p: Vec2) -> bool {
        match self.basis().inverse() {
            Some(inv) => {
                let q = inv.apply(p);
                (0.0..=1.0).contains(&q.x) && (0.0..=1.0).contains(&q.y)
            }
            None => false,
        }
    }
}

/// The box the gizmo is drawn on: for a single layer, its content bounds in the layer's own axes
/// (so a rotated layer gets a rotated box); otherwise the axis-aligned bounds. A box with no extent
/// along an axis is widened to `min_extent` so handles stay usable.
pub fn box_frame(engine: &Engine, frame: &Frame, sel: &Selection, min_extent: f32) -> Option<BoxFrame> {
    let pad = |b: BoxFrame| -> BoxFrame {
        let (lu, lv) = (b.u.length(), b.v.length());
        let mut b = b;
        let uh = if lu < 1e-6 { Vec2::new(1.0, 0.0) } else { b.u * (1.0 / lu) };
        let vh = if lv < 1e-6 { Vec2::new(0.0, 1.0) } else { b.v * (1.0 / lv) };
        if lu < min_extent {
            b.origin = b.origin - uh * ((min_extent - lu) * 0.5);
            b.u = uh * min_extent;
        }
        if lv < min_extent {
            b.origin = b.origin - vh * ((min_extent - lv) * 0.5);
            b.v = vh * min_extent;
        }
        b
    };
    if let [Target::Layer { path }] = sel.items.as_slice() {
        let id = engine.node_index(path)?;
        let pts = target_points(engine, frame, &sel.items[0]);
        if let Some(inv) = frame.nodes[id].world.inverse() {
            let local: Vec<Vec2> = pts.iter().map(|p| inv.apply(*p)).collect();
            if let Some((lo, hi)) = aabb(&local) {
                let w = frame.nodes[id].world;
                let o = w.apply(lo);
                let b = BoxFrame { origin: o, u: w.apply(Vec2::new(hi.x, lo.y)) - o, v: w.apply(Vec2::new(lo.x, hi.y)) - o };
                return Some(pad(b));
            }
        }
    }
    let (lo, hi) = selection_bounds(engine, frame, sel)?;
    Some(pad(BoxFrame::from_aabb(lo, hi)))
}

// ---------------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::VPath;
    use crate::project::{Bone, Layer, Project, Transform2D};

    fn proj(layers: Vec<Layer>) -> Project {
        let mut p = Project::new("t", 400.0, 300.0, 24.0, 10);
        p.layers = layers;
        p
    }
    fn rect(name: &str, x: f32, y: f32, w: f32, h: f32, col: &str) -> Layer {
        Layer::vector(name, vec![VPath::rect("r", x, y, w, h).with_fill(col)])
    }
    fn eng(p: Project) -> Engine {
        Engine::new(p).unwrap()
    }
    fn at(e: &Engine, x: f32, y: f32, o: &PickOptions) -> Option<Target> {
        pick_at(e, 0.0, Vec2::new(x, y), o).map(|h| h.target)
    }

    #[test]
    fn fill_hit_and_miss_with_transform_and_rotation() {
        let mut l = rect("a", 0.0, 0.0, 100.0, 20.0, "#f00");
        l.transform = Transform2D { position: Vec2::new(200.0, 100.0), rotation: 90.0, ..Default::default() };
        let e = eng(proj(vec![l]));
        let o = PickOptions::default();
        // rotated 90 degrees: the box occupies x in [180,200], y in [100,200]
        assert_eq!(at(&e, 190.0, 150.0, &o), Some(Target::layer("a")));
        assert_eq!(at(&e, 250.0, 110.0, &o), None, "where the unrotated box would be");
        assert_eq!(at(&e, 190.0, 250.0, &o), None);
    }

    #[test]
    fn z_order_topmost_wins_and_pick_all_lists_both() {
        let e = eng(proj(vec![rect("under", 0.0, 0.0, 100.0, 100.0, "#00f"), rect("over", 50.0, 50.0, 100.0, 100.0, "#f00")]));
        let o = PickOptions::default();
        assert_eq!(at(&e, 75.0, 75.0, &o), Some(Target::layer("over")));
        assert_eq!(at(&e, 25.0, 25.0, &o), Some(Target::layer("under")));
        let f = e.evaluate(0.0);
        let all = pick_all(&e, &f, Vec2::new(75.0, 75.0), &o);
        assert_eq!(all.iter().map(|h| h.target.clone()).collect::<Vec<_>>(), vec![Target::layer("over"), Target::layer("under")]);
    }

    #[test]
    fn locked_passes_through_and_hidden_is_skipped() {
        let mut hid = rect("hid", 0.0, 0.0, 100.0, 100.0, "#0f0");
        hid.visible = false;
        let e = eng(proj(vec![rect("base", 0.0, 0.0, 100.0, 100.0, "#00f"), rect("top", 0.0, 0.0, 100.0, 100.0, "#f00"), hid]));
        let mut o = PickOptions::default();
        assert_eq!(at(&e, 10.0, 10.0, &o), Some(Target::layer("top")));
        o.locked = vec!["top".into()];
        assert_eq!(at(&e, 10.0, 10.0, &o), Some(Target::layer("base")), "click passes through a locked layer");
        o.locked.push("base".into());
        assert_eq!(at(&e, 10.0, 10.0, &o), None);
    }

    #[test]
    fn stroke_tolerance() {
        let line = Layer::vector("ln", vec![VPath::new("p", false, vec![crate::path::Anchor::corner(Vec2::new(0.0, 50.0)), crate::path::Anchor::corner(Vec2::new(200.0, 50.0))]).with_stroke("#000", 4.0)]);
        let e = eng(proj(vec![line]));
        let mut o = PickOptions { tolerance: 0.0, ..Default::default() };
        assert!(at(&e, 100.0, 51.5, &o).is_some(), "inside half width 2");
        assert!(at(&e, 100.0, 53.0, &o).is_none(), "outside with zero slack");
        o.tolerance = 3.0;
        assert!(at(&e, 100.0, 53.0, &o).is_some(), "slack makes thin lines clickable");
        assert!(at(&e, 100.0, 60.0, &o).is_none());
        let h = pick_at(&e, 0.0, Vec2::new(100.0, 53.0), &o).unwrap();
        assert!((h.distance - 1.0).abs() < 1e-3, "{}", h.distance);
    }

    #[test]
    fn tolerance_applies_to_fills_and_bones() {
        let e = eng(proj(vec![rect("r", 0.0, 0.0, 50.0, 50.0, "#f00")]));
        let mut o = PickOptions { tolerance: 0.0, ..Default::default() };
        assert!(at(&e, 52.0, 25.0, &o).is_none());
        o.tolerance = 3.0;
        assert_eq!(at(&e, 52.0, 25.0, &o), Some(Target::layer("r")), "fill outline slack");
        assert!(at(&e, 55.0, 25.0, &o).is_none());
        let mut p = proj(vec![]);
        p.bones = vec![Bone::new("b", None, 100.0, Vec2::new(50.0, 50.0), 0.0)];
        let e = eng(p);
        let mut o = PickOptions { tolerance: 0.0, ..Default::default() };
        assert!(at(&e, 100.0, 56.0, &o).is_none());
        o.tolerance = 3.0;
        assert_eq!(at(&e, 100.0, 56.0, &o), Some(Target::bone("b")));
    }

    #[test]
    fn groups_leaf_versus_root_and_nested_transforms() {
        let mut inner = rect("kid", 0.0, 0.0, 50.0, 50.0, "#f00");
        inner.transform.position = Vec2::new(100.0, 0.0);
        let mut g = Layer::group("grp", vec![inner]);
        g.transform.position = Vec2::new(0.0, 100.0);
        let e = eng(proj(vec![g]));
        let mut o = PickOptions::default();
        assert_eq!(at(&e, 120.0, 120.0, &o), Some(Target::layer("grp/kid")));
        o.group_mode = GroupMode::Root;
        assert_eq!(at(&e, 120.0, 120.0, &o), Some(Target::layer("grp")));
        assert_eq!(at(&e, 20.0, 20.0, &o), None);
    }

    #[test]
    fn animated_transform_is_respected() {
        let mut p = proj(vec![rect("a", 0.0, 0.0, 40.0, 40.0, "#f00")]);
        p.channels = vec![crate::project::Channel::keys(
            "a",
            crate::project::Prop::LayerX,
            crate::anim::Track::with_keys(&[(0.0, 0.0, crate::anim::Ease::Linear), (10.0, 100.0, crate::anim::Ease::Linear)]),
        )];
        let e = eng(p);
        let o = PickOptions::default();
        assert!(pick_at(&e, 0.0, Vec2::new(20.0, 20.0), &o).is_some());
        assert!(pick_at(&e, 10.0, Vec2::new(20.0, 20.0), &o).is_none());
        assert!(pick_at(&e, 10.0, Vec2::new(120.0, 20.0), &o).is_some());
    }

    #[test]
    fn even_odd_hole_versus_non_zero() {
        // outer square and an inner square with the same winding: non-zero fills the hole, even-odd does not
        let mut anchors = vec![];
        for (x, y) in [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0), (0.0, 0.0), (25.0, 25.0), (75.0, 25.0), (75.0, 75.0), (25.0, 75.0), (25.0, 25.0), (0.0, 0.0)] {
            anchors.push(crate::path::Anchor::corner(Vec2::new(x, y)));
        }
        let l = Layer::vector("h", vec![VPath::new("p", true, anchors).with_fill("#000")]);
        let e = eng(proj(vec![l]));
        let c = Vec2::new(50.0, 50.0);
        let nz = PickOptions::default();
        let eo = PickOptions { fill_rule: FillRule::EvenOdd, ..Default::default() };
        assert!(pick_at(&e, 0.0, c, &nz).is_some());
        assert!(pick_at(&e, 0.0, c, &eo).is_none());
        assert!(pick_at(&e, 0.0, Vec2::new(10.0, 50.0), &eo).is_some());
    }

    #[test]
    fn image_hit_by_box_with_rotation() {
        let mut pm = tiny_skia::Pixmap::new(4, 4).unwrap();
        pm.fill(tiny_skia::Color::from_rgba8(255, 0, 0, 255));
        let png = pm.encode_png().unwrap();
        let mut l = Layer::image_from_png("img", &png, Vec2::new(0.0, 0.0));
        if let LayerKind::Image { size, .. } = &mut l.kind {
            *size = Some(Vec2::new(100.0, 20.0));
        }
        l.transform = Transform2D { position: Vec2::new(150.0, 100.0), rotation: 90.0, ..Default::default() };
        let e = eng(proj(vec![l]));
        let o = PickOptions::default();
        assert_eq!(at(&e, 140.0, 150.0, &o), Some(Target::layer("img")));
        assert_eq!(at(&e, 200.0, 105.0, &o), None);
    }

    #[test]
    fn bone_capsule_and_priority_over_layers() {
        let mut p = proj(vec![rect("a", 0.0, 0.0, 300.0, 300.0, "#f00")]);
        p.bones = vec![Bone::new("b", None, 100.0, Vec2::new(50.0, 50.0), 0.0)];
        let e = eng(p);
        let o = PickOptions::default();
        assert_eq!(at(&e, 100.0, 54.0, &o), Some(Target::bone("b")), "bone wins over the layer under it");
        assert_eq!(at(&e, 100.0, 70.0, &o), Some(Target::layer("a")));
        assert_eq!(at(&e, 152.0, 50.0, &o), Some(Target::bone("b")), "round cap past the tip");
        let o2 = PickOptions { bones: false, ..Default::default() };
        assert_eq!(at(&e, 100.0, 54.0, &o2), Some(Target::layer("a")));
        let o3 = PickOptions { locked_bones: vec!["b".into()], ..Default::default() };
        assert_eq!(at(&e, 100.0, 54.0, &o3), Some(Target::layer("a")));
    }

    #[test]
    fn control_points_only_for_listed_layers_and_nearest_first() {
        let e = eng(proj(vec![rect("a", 10.0, 10.0, 100.0, 100.0, "#f00")]));
        let p = Vec2::new(12.0, 11.0);
        let none = pick_at(&e, 0.0, p, &PickOptions::default()).unwrap();
        assert_eq!(none.target, Target::layer("a"));
        let o = PickOptions { point_layers: vec!["a".into()], ..Default::default() };
        let h = pick_at(&e, 0.0, p, &o).unwrap();
        assert_eq!(h.target, Target::Point { layer: "a".into(), point: PointKind::Anchor { path: 0, index: 0 } });
        assert!(h.distance < 3.0);
        assert_eq!(at(&e, 60.0, 60.0, &o), Some(Target::layer("a")));
    }

    #[test]
    fn marquee_modes_rect() {
        let e = eng(proj(vec![rect("small", 10.0, 10.0, 20.0, 20.0, "#f00"), rect("big", 0.0, 0.0, 200.0, 200.0, "#00f"), rect("far", 300.0, 200.0, 20.0, 20.0, "#0f0")]));
        let f = e.evaluate(0.0);
        let o = PickOptions::default();
        let r = Region::rect(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let names = |v: Vec<Target>| v.into_iter().map(|t| if let Target::Layer { path } = t { path } else { String::new() }).collect::<Vec<_>>();
        assert_eq!(names(marquee(&e, &f, &r, MarqueeMode::Contain, &o)), vec!["small"]);
        assert_eq!(names(marquee(&e, &f, &r, MarqueeMode::Intersect, &o)), vec!["small", "big"]);
        assert_eq!(names(marquee(&e, &f, &r, MarqueeMode::Bbox, &o)), vec!["small", "big"]);
        // a region wholly inside the big fill touches it by intersection (fill under the region)
        let inside = Region::rect(Vec2::new(100.0, 100.0), Vec2::new(150.0, 150.0));
        assert_eq!(names(marquee(&e, &f, &inside, MarqueeMode::Intersect, &o)), vec!["big"]);
        assert!(marquee(&e, &f, &inside, MarqueeMode::Contain, &o).is_empty());
    }

    #[test]
    fn marquee_bbox_versus_intersect_on_a_diagonal_stroke_and_lasso() {
        let diag = Layer::vector("d", vec![VPath::new("p", false, vec![crate::path::Anchor::corner(Vec2::new(0.0, 0.0)), crate::path::Anchor::corner(Vec2::new(100.0, 100.0))]).with_stroke("#000", 2.0)]);
        let e = eng(proj(vec![diag]));
        let f = e.evaluate(0.0);
        let o = PickOptions::default();
        let corner = Region::rect(Vec2::new(70.0, 0.0), Vec2::new(100.0, 30.0)); // off the diagonal, inside its bbox
        assert_eq!(marquee(&e, &f, &corner, MarqueeMode::Bbox, &o).len(), 1);
        assert!(marquee(&e, &f, &corner, MarqueeMode::Intersect, &o).is_empty());
        let lasso = Region::Lasso(vec![Vec2::new(40.0, 30.0), Vec2::new(60.0, 30.0), Vec2::new(60.0, 70.0), Vec2::new(40.0, 70.0)]);
        assert_eq!(marquee(&e, &f, &lasso, MarqueeMode::Intersect, &o).len(), 1);
        let tri = Region::Lasso(vec![Vec2::new(70.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(100.0, 30.0)]);
        assert!(marquee(&e, &f, &tri, MarqueeMode::Intersect, &o).is_empty());
        assert!(marquee(&e, &f, &Region::Lasso(vec![Vec2::new(0.0, 0.0), Vec2::new(5.0, 5.0)]), MarqueeMode::Bbox, &o).is_empty(), "two points are no region");
    }

    #[test]
    fn marquee_group_root_contain_needs_all_children() {
        let g = Layer::group("g", vec![rect("a", 0.0, 0.0, 20.0, 20.0, "#f00"), rect("b", 100.0, 0.0, 20.0, 20.0, "#00f")]);
        let e = eng(proj(vec![g]));
        let f = e.evaluate(0.0);
        let o = PickOptions { group_mode: GroupMode::Root, ..Default::default() };
        let part = Region::rect(Vec2::new(-5.0, -5.0), Vec2::new(30.0, 30.0));
        assert!(marquee(&e, &f, &part, MarqueeMode::Contain, &o).is_empty());
        assert_eq!(marquee(&e, &f, &part, MarqueeMode::Intersect, &o), vec![Target::layer("g")]);
        let all = Region::rect(Vec2::new(-5.0, -5.0), Vec2::new(130.0, 30.0));
        assert_eq!(marquee(&e, &f, &all, MarqueeMode::Contain, &o), vec![Target::layer("g")]);
        let leaf = PickOptions::default();
        assert_eq!(marquee(&e, &f, &part, MarqueeMode::Intersect, &leaf), vec![Target::layer("g/a")]);
    }

    #[test]
    fn marquee_skips_locked_and_hidden() {
        let mut hid = rect("hid", 0.0, 0.0, 10.0, 10.0, "#0f0");
        hid.visible = false;
        let e = eng(proj(vec![rect("a", 0.0, 0.0, 10.0, 10.0, "#f00"), rect("b", 0.0, 0.0, 10.0, 10.0, "#f00"), hid]));
        let f = e.evaluate(0.0);
        let o = PickOptions { locked: vec!["a".into()], ..Default::default() };
        let r = Region::rect(Vec2::new(-1.0, -1.0), Vec2::new(20.0, 20.0));
        assert_eq!(marquee(&e, &f, &r, MarqueeMode::Contain, &o), vec![Target::layer("b")]);
    }

    #[test]
    fn selection_semantics() {
        let (a, b, c) = (Target::layer("a"), Target::layer("b"), Target::layer("c"));
        let mut s = Selection::new();
        s.apply(std::slice::from_ref(&a), SelectMode::Replace);
        s.apply(std::slice::from_ref(&b), SelectMode::Add);
        s.apply(std::slice::from_ref(&b), SelectMode::Add);
        assert_eq!(s.items, vec![a.clone(), b.clone()]);
        s.apply(&[a.clone(), c.clone()], SelectMode::Toggle);
        assert_eq!(s.items, vec![b.clone(), c.clone()]);
        s.apply(std::slice::from_ref(&b), SelectMode::Subtract);
        assert_eq!(s.items, vec![c.clone()]);
        s.apply(&[a.clone(), a.clone()], SelectMode::Replace);
        assert_eq!(s.items, vec![a.clone()], "duplicates collapse");
        s.apply(&[], SelectMode::Replace);
        assert!(s.is_empty(), "click on empty space clears");
        assert_eq!(SelectMode::parse("toggle"), Some(SelectMode::Toggle));
        assert_eq!(SelectMode::parse("x"), None);
    }

    #[test]
    fn prune_and_target_json_round_trip() {
        let e = eng(proj(vec![rect("a", 0.0, 0.0, 10.0, 10.0, "#f00")]));
        let mut s = Selection::from_items(vec![Target::layer("a"), Target::layer("gone"), Target::bone("nope")]);
        s.prune(&e);
        assert_eq!(s.items, vec![Target::layer("a")]);
        let t = Target::Point { layer: "a".into(), point: PointKind::Lattice { deformer: 1, index: 4 } };
        let j = serde_json::to_string(&t).unwrap();
        assert_eq!(serde_json::from_str::<Target>(&j).unwrap(), t);
        assert!(j.contains("\"kind\":\"point\"") && j.contains("\"type\":\"lattice\""), "{j}");
    }

    #[test]
    fn bounds_axis_aligned_and_oriented_single_layer() {
        let mut l = rect("a", 0.0, 0.0, 100.0, 20.0, "#f00");
        l.transform = Transform2D { position: Vec2::new(200.0, 100.0), rotation: 90.0, ..Default::default() };
        let e = eng(proj(vec![l, rect("b", 0.0, 0.0, 10.0, 10.0, "#00f")]));
        let f = e.evaluate(0.0);
        let one = Selection::from_items(vec![Target::layer("a")]);
        let (lo, hi) = selection_bounds(&e, &f, &one).unwrap();
        assert!(lo.distance(Vec2::new(180.0, 100.0)) < 1e-3 && hi.distance(Vec2::new(200.0, 200.0)) < 1e-3, "{lo:?} {hi:?}");
        let bf = box_frame(&e, &f, &one, 1.0).unwrap();
        // origin is the layer's own top-left corner, u runs along the rotated x axis
        assert!(bf.origin.distance(Vec2::new(200.0, 100.0)) < 1e-3, "{:?}", bf.origin);
        assert!(bf.u.distance(Vec2::new(0.0, 100.0)) < 1e-3 && bf.v.distance(Vec2::new(-20.0, 0.0)) < 1e-3);
        assert!(bf.contains(Vec2::new(190.0, 150.0)) && !bf.contains(Vec2::new(250.0, 110.0)));
        let two = Selection::from_items(vec![Target::layer("a"), Target::layer("b")]);
        let bb = box_frame(&e, &f, &two, 1.0).unwrap();
        assert!(bb.u.y == 0.0 && bb.v.x == 0.0, "multi selection uses an axis-aligned box");
        assert!(selection_bounds(&e, &f, &Selection::new()).is_none());
    }

    #[test]
    fn degenerate_box_is_padded() {
        let line = Layer::vector("ln", vec![VPath::new("p", false, vec![crate::path::Anchor::corner(Vec2::new(0.0, 50.0)), crate::path::Anchor::corner(Vec2::new(100.0, 50.0))]).with_stroke("#000", 2.0)]);
        let e = eng(proj(vec![line]));
        let f = e.evaluate(0.0);
        let s = Selection::from_items(vec![Target::layer("ln")]);
        let b = box_frame(&e, &f, &s, 8.0).unwrap();
        assert!(b.nondegenerate() && (b.v.length() - 8.0).abs() < 1e-3 && b.centre().distance(Vec2::new(50.0, 50.0)) < 1e-3, "{b:?}");
    }

    #[test]
    fn deterministic() {
        let e = eng(proj(vec![rect("a", 0.0, 0.0, 50.0, 50.0, "#f00"), rect("b", 25.0, 25.0, 50.0, 50.0, "#00f")]));
        let f = e.evaluate(0.0);
        let r = Region::rect(Vec2::new(0.0, 0.0), Vec2::new(60.0, 60.0));
        let o = PickOptions::default();
        assert_eq!(marquee(&e, &f, &r, MarqueeMode::Intersect, &o), marquee(&e, &f, &r, MarqueeMode::Intersect, &o));
        assert_eq!(pick_all(&e, &f, Vec2::new(30.0, 30.0), &o), pick_all(&e, &f, Vec2::new(30.0, 30.0), &o));
    }

    #[test]
    fn the_persistent_layer_lock_is_skipped_by_clicks_and_boxes_and_covers_groups() {
        let mut top = rect("top", 0.0, 0.0, 100.0, 100.0, "#f00");
        top.locked = true;
        let mut grp = Layer::group("grp", vec![rect("kid", 0.0, 0.0, 100.0, 100.0, "#0f0")]);
        let e0 = eng(proj(vec![rect("base", 0.0, 0.0, 100.0, 100.0, "#00f"), grp.clone(), top.clone()]));
        let o = PickOptions::default();
        assert_eq!(at(&e0, 10.0, 10.0, &o), Some(Target::layer("grp/kid")), "the locked top layer is passed through");
        let mut o2 = o.clone();
        o2.ignore_layer_locks = true;
        assert_eq!(at(&e0, 10.0, 10.0, &o2), Some(Target::layer("top")));
        // lock the group: its child cannot be picked either
        grp.locked = true;
        let e1 = eng(proj(vec![rect("base", 0.0, 0.0, 100.0, 100.0, "#00f"), grp, top]));
        assert_eq!(at(&e1, 10.0, 10.0, &o), Some(Target::layer("base")));
        let f = e1.evaluate(0.0);
        let r = Region::rect(Vec2::new(-1.0, -1.0), Vec2::new(200.0, 200.0));
        assert_eq!(marquee(&e1, &f, &r, MarqueeMode::Contain, &o), vec![Target::layer("base")]);
        assert_eq!(marquee(&e1, &f, &r, MarqueeMode::Contain, &o2).len(), 3);
        assert!(pick_all(&e1, &f, Vec2::new(10.0, 10.0), &o).iter().all(|h| h.target == Target::layer("base")));
    }

    #[test]
    fn layers_bounds_covers_children_ignores_unknown_names_and_bad_frames() {
        let grp = Layer::group("g", vec![rect("a", 10.0, 20.0, 30.0, 10.0, "#f00"), rect("b", 100.0, 50.0, 20.0, 20.0, "#f00")]);
        let e = eng(proj(vec![grp, rect("z", 500.0, 500.0, 1.0, 1.0, "#000")]));
        let b = layers_bounds(&e, 0.0, &["g".to_string()]).unwrap();
        assert!((b[0] - 10.0).abs() < 0.01 && (b[1] - 20.0).abs() < 0.01 && (b[2] - 120.0).abs() < 0.01 && (b[3] - 70.0).abs() < 0.01, "{b:?}");
        assert_eq!(layers_bounds(&e, 0.0, &["g/a".to_string(), "nope".to_string()]).unwrap()[2].round(), 40.0);
        assert!(layers_bounds(&e, 0.0, &["nope".to_string()]).is_none());
        assert!(layers_bounds(&e, 0.0, &[]).is_none());
        assert!(layers_bounds(&e, f32::INFINITY, &["g".to_string()]).is_none());
        assert!(layers_bounds(&e, 9999.0, &["g".to_string()]).is_some(), "frames are clamped, not an error");
    }
}
