// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `gizmo` (see the module docs in `mod.rs`).
use super::*;

/// Distance of the rotate knob from the box edge, canvas units.
pub const ROTATE_OFFSET: f32 = 26.0;

/// Smallest box extent the gizmo shows, canvas units.
pub const MIN_EXTENT: f32 = 10.0;

// ---------------------------------------------------------------------------------------------
// Handles
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleHandle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
}

pub const SCALE_HANDLES: [ScaleHandle; 8] = [ScaleHandle::Nw, ScaleHandle::Ne, ScaleHandle::Se, ScaleHandle::Sw, ScaleHandle::N, ScaleHandle::E, ScaleHandle::S, ScaleHandle::W];

impl ScaleHandle {
    /// Position in the box's unit square.
    pub fn local(self) -> (f32, f32) {
        match self {
            ScaleHandle::Nw => (0.0, 0.0),
            ScaleHandle::N => (0.5, 0.0),
            ScaleHandle::Ne => (1.0, 0.0),
            ScaleHandle::E => (1.0, 0.5),
            ScaleHandle::Se => (1.0, 1.0),
            ScaleHandle::S => (0.5, 1.0),
            ScaleHandle::Sw => (0.0, 1.0),
            ScaleHandle::W => (0.0, 0.5),
        }
    }
    pub fn is_corner(self) -> bool {
        matches!(self, ScaleHandle::Nw | ScaleHandle::Ne | ScaleHandle::Se | ScaleHandle::Sw)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Handle {
    Move,
    Rotate,
    Scale { at: ScaleHandle },
    BoneHead,
    BoneTip,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HandlePos {
    pub handle: Handle,
    pub pos: Vec2,
    /// CSS cursor name for hover feedback.
    pub cursor: String,
}

pub(super) fn cursor_for_dir(d: Vec2) -> &'static str {
    let a = d.y.atan2(d.x).to_degrees().rem_euclid(180.0);
    if !(22.5..157.5).contains(&a) {
        "ew-resize"
    } else if a < 67.5 {
        "nwse-resize"
    } else if a < 112.5 {
        "ns-resize"
    } else {
        "nesw-resize"
    }
}

/// Handle positions for a box gizmo: 8 scale handles and the rotate knob above the top edge.
pub fn box_handles(bx: &BoxFrame) -> Vec<HandlePos> {
    let c = bx.centre();
    let mut v: Vec<HandlePos> = SCALE_HANDLES
        .iter()
        .map(|&h| {
            let (s, t) = h.local();
            let p = bx.point(s, t);
            HandlePos { handle: Handle::Scale { at: h }, pos: p, cursor: cursor_for_dir(p - c).into() }
        })
        .collect();
    let lv = bx.v.length().max(1e-6);
    let up = bx.v * (-1.0 / lv);
    v.push(HandlePos { handle: Handle::Rotate, pos: bx.point(0.5, 0.0) + up * ROTATE_OFFSET, cursor: "grab".into() });
    v
}

/// Which handle (if any) lies under `p`. The rotate knob and corners win over edges; `Move` is
/// reported for a point inside the box.
pub fn hit_handle(bx: &BoxFrame, p: Vec2, radius: f32) -> Option<Handle> {
    let hs = box_handles(bx);
    let order = |h: &Handle| match h {
        Handle::Rotate => 0,
        Handle::Scale { at } if at.is_corner() => 1,
        _ => 2,
    };
    let mut best: Option<(&HandlePos, f32)> = None;
    for h in &hs {
        let d = h.pos.distance(p);
        if d <= radius {
            let better = match best {
                None => true,
                Some((b, bd)) => order(&h.handle) < order(&b.handle) || (order(&h.handle) == order(&b.handle) && d < bd),
            };
            if better {
                best = Some((h, d));
            }
        }
    }
    if let Some((h, _)) = best {
        return Some(h.handle);
    }
    if bx.contains(p) {
        return Some(Handle::Move);
    }
    None
}

/// What to draw for the current selection.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GizmoView {
    /// `none`, `box`, `bone` or `points`.
    pub kind: String,
    pub corners: Option<[Vec2; 4]>,
    pub handles: Vec<HandlePos>,
    pub points: Vec<Vec2>,
}

pub fn gizmo_view(engine: &Engine, frame: &Frame, sel: &Selection) -> GizmoView {
    let mut g = GizmoView { kind: "none".into(), ..Default::default() };
    if sel.is_empty() {
        return g;
    }
    if sel.items.iter().all(|t| matches!(t, Target::Point { .. })) {
        g.kind = "points".into();
        g.points = sel.items.iter().flat_map(|t| pick::target_points(engine, frame, t)).collect();
        return g;
    }
    if let [Target::Bone { name }] = sel.items.as_slice() {
        if let Some((h, t)) = bone_ends(engine, frame, name) {
            g.kind = "bone".into();
            g.handles = vec![
                HandlePos { handle: Handle::BoneHead, pos: h, cursor: "move".into() },
                HandlePos { handle: Handle::BoneTip, pos: t, cursor: "crosshair".into() },
            ];
            return g;
        }
    }
    if sel.items.iter().any(|t| matches!(t, Target::Point { .. })) {
        return g;
    }
    if let Some(bx) = box_frame(engine, frame, sel, MIN_EXTENT) {
        g.kind = "box".into();
        g.corners = Some(bx.corners());
        g.handles = box_handles(&bx);
    }
    g
}

// ---------------------------------------------------------------------------------------------
// Snapping
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SnapOptions {
    /// Grid spacing in canvas units; `None` = no grid snapping.
    pub grid: Option<f32>,
    /// Snap to other layers' bounds and centres.
    pub objects: bool,
    /// Snap to the canvas edges and centre.
    pub canvas: bool,
    /// Capture distance in canvas units.
    pub tolerance: f32,
}

impl Default for SnapOptions {
    fn default() -> Self {
        SnapOptions { grid: None, objects: true, canvas: true, tolerance: 6.0 }
    }
}

/// One snap line: a coordinate and the extent of the thing it came from along the other axis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapLine {
    pub at: f32,
    pub from: f32,
    pub to: f32,
    /// `object`, `canvas` or `grid`.
    pub kind: String,
}

#[derive(Clone, Debug, Default)]
pub struct SnapTargets {
    pub xs: Vec<SnapLine>,
    pub ys: Vec<SnapLine>,
}

/// A line to draw while snapping (`axis` "x" = vertical line at x = `at`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Guide {
    pub axis: String,
    pub at: f32,
    pub from: f32,
    pub to: f32,
    pub kind: String,
}

/// Snap candidates from the canvas and from every visible layer outside the selection.
pub fn snap_targets(engine: &Engine, frame: &Frame, sel: &Selection, o: &SnapOptions) -> SnapTargets {
    let mut t = SnapTargets::default();
    let add = |t: &mut SnapTargets, lo: Vec2, hi: Vec2, kind: &str| {
        for x in [lo.x, (lo.x + hi.x) * 0.5, hi.x] {
            t.xs.push(SnapLine { at: x, from: lo.y, to: hi.y, kind: kind.into() });
        }
        for y in [lo.y, (lo.y + hi.y) * 0.5, hi.y] {
            t.ys.push(SnapLine { at: y, from: lo.x, to: hi.x, kind: kind.into() });
        }
    };
    if o.canvas {
        let p = &engine.project;
        let c: Vec<Vec2> = [Vec2::ZERO, Vec2::new(p.width, 0.0), Vec2::new(p.width, p.height), Vec2::new(0.0, p.height)].iter().map(|q| frame.camera.apply(*q)).collect();
        let lo = Vec2::new(c.iter().map(|q| q.x).fold(f32::INFINITY, f32::min), c.iter().map(|q| q.y).fold(f32::INFINITY, f32::min));
        let hi = Vec2::new(c.iter().map(|q| q.x).fold(f32::NEG_INFINITY, f32::max), c.iter().map(|q| q.y).fold(f32::NEG_INFINITY, f32::max));
        add(&mut t, lo, hi, "canvas");
    }
    if o.objects {
        let selected: Vec<&str> = sel.layers();
        for id in pick::draw_order(frame) {
            let name = &frame.nodes[id].name;
            if selected.iter().any(|s| name == s || name.starts_with(&format!("{s}/"))) {
                continue;
            }
            let pts: Vec<Vec2> = pick::node_shapes(frame, id).into_iter().flat_map(|s| s.pts).collect();
            if pts.is_empty() {
                continue;
            }
            let lo = Vec2::new(pts.iter().map(|q| q.x).fold(f32::INFINITY, f32::min), pts.iter().map(|q| q.y).fold(f32::INFINITY, f32::min));
            let hi = Vec2::new(pts.iter().map(|q| q.x).fold(f32::NEG_INFINITY, f32::max), pts.iter().map(|q| q.y).fold(f32::NEG_INFINITY, f32::max));
            add(&mut t, lo, hi, "object");
        }
    }
    t
}

/// Snap a translation of the extent `(lo, hi)`. Returns the adjusted delta and the guides.
pub fn snap_move(lo: Vec2, hi: Vec2, delta: Vec2, constrain: bool, o: &SnapOptions, t: &SnapTargets) -> (Vec2, Vec<Guide>) {
    let mut d = delta;
    let mut locked = [false, false];
    if constrain {
        if d.x.abs() >= d.y.abs() {
            d.y = 0.0;
            locked[1] = true;
        } else {
            d.x = 0.0;
            locked[0] = true;
        }
    }
    let mut picked: [Option<SnapLine>; 2] = [None, None];
    for axis in 0..2 {
        if locked[axis] {
            continue;
        }
        let (l0, l1, cands) = if axis == 0 { (lo.x, hi.x, &t.xs) } else { (lo.y, hi.y, &t.ys) };
        let off = if axis == 0 { d.x } else { d.y };
        let lines = [l0 + off, (l0 + l1) * 0.5 + off, l1 + off];
        let mut best: Option<(f32, SnapLine)> = None;
        for l in lines {
            for c in cands.iter() {
                let diff = c.at - l;
                if diff.abs() <= o.tolerance && best.as_ref().map(|(b, _)| diff.abs() < b.abs()).unwrap_or(true) {
                    best = Some((diff, c.clone()));
                }
            }
            if let Some(g) = o.grid.filter(|g| *g > 0.0) {
                let diff = (l / g).round() * g - l;
                if diff.abs() <= o.tolerance && best.as_ref().map(|(b, _)| diff.abs() < b.abs()).unwrap_or(true) {
                    best = Some((diff, SnapLine { at: l + diff, from: f32::NAN, to: f32::NAN, kind: "grid".into() }));
                }
            }
        }
        if let Some((diff, line)) = best {
            if axis == 0 {
                d.x += diff;
            } else {
                d.y += diff;
            }
            picked[axis] = Some(line);
        }
    }
    let mut guides = Vec::new();
    for (axis, p) in picked.iter().enumerate() {
        let Some(l) = p else { continue };
        // extent of the moving box along the other axis, after the move
        let (m0, m1) = if axis == 0 { (lo.y + d.y, hi.y + d.y) } else { (lo.x + d.x, hi.x + d.x) };
        let (from, to) = if l.from.is_nan() { (m0 - 20.0, m1 + 20.0) } else { (m0.min(l.from), m1.max(l.to)) };
        guides.push(Guide { axis: if axis == 0 { "x".into() } else { "y".into() }, at: l.at, from, to, kind: l.kind.clone() });
    }
    (d, guides)
}
