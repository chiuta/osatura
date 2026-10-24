// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `gizmo` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------------------------

/// One op batch that removes the selected layers (and the channels that target them).
pub fn delete_op(ed: &Editor, sel: &Selection) -> Result<Op, String> {
    let engine = ed.engine();
    if sel.items.iter().any(|t| !matches!(t, Target::Layer { .. })) {
        return Err("only layers can be deleted from the canvas".into());
    }
    let layers = sel.layers();
    if let Some(l) = layers.iter().find(|l| engine.project.is_layer_locked(l)) {
        return Err(format!("layer '{l}' is locked"));
    }
    let mut victims: Vec<(Vec<usize>, String)> = Vec::new();
    for p in &layers {
        if has_selected_ancestor(p, &layers) {
            continue;
        }
        let id = engine.node_index(p).ok_or_else(|| format!("no layer '{p}'"))?;
        victims.push((engine.node_idx_path(id).to_vec(), p.to_string()));
    }
    if victims.is_empty() {
        return Err("nothing selected".into());
    }
    let mut ops: Vec<Op> = Vec::new();
    // channels first (they sit at the end of the document order, indices independent of layers)
    let mut chan_ix: Vec<usize> = engine
        .project
        .channels
        .iter()
        .enumerate()
        .filter(|(_, c)| victims.iter().any(|(_, p)| c.target == *p || c.target.starts_with(&format!("{p}/")) || c.target.starts_with(&format!("{p}::"))))
        .map(|(i, _)| i)
        .collect();
    chan_ix.sort_unstable_by(|a, b| b.cmp(a));
    for i in chan_ix {
        ops.push(Op::Remove { path: format!("/channels/{i}") });
    }
    // then layers, last index first so earlier pointers stay valid
    victims.sort_by(|a, b| b.0.cmp(&a.0));
    for (ix, _) in victims {
        let mut s = format!("/layers/{}", ix[0]);
        for i in &ix[1..] {
            s.push_str(&format!("/layers/{i}"));
        }
        ops.push(Op::Remove { path: s });
    }
    Ok(Op::Batch { ops })
}

// ---------------------------------------------------------------------------------------------
// Request / preview / commit
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct DragRequest {
    pub frame: f32,
    pub selection: Vec<Target>,
    pub handle: Handle,
    pub start: Vec2,
    pub delta: Vec2,
    #[serde(default)]
    pub modifiers: Modifiers,
    #[serde(default)]
    pub snap: SnapOptions,
    #[serde(default)]
    pub auto_key: bool,
    /// Locked layer paths and `bone:<name>` entries.
    #[serde(default)]
    pub locked: Vec<String>,
    #[serde(default)]
    pub pivot: Option<Vec2>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Outline {
    pub pts: Vec<Vec2>,
    pub closed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct DragPreview {
    /// `[a, b, c, d, tx, ty]`.
    pub matrix: [f32; 6],
    pub summary: String,
    pub guides: Vec<Guide>,
    /// Selected geometry after the map, for a ghost overlay.
    pub outline: Vec<Outline>,
    pub box_after: Option<[Vec2; 4]>,
    pub edits: Vec<EditInfo>,
    pub changed: bool,
}

pub(super) struct Prepared {
    pub(super) d: DragMatrix,
    pub(super) plan: Plan,
    pub(super) sel: Selection,
    pub(super) outline: Vec<Outline>,
    pub(super) box_after: Option<[Vec2; 4]>,
}

pub(super) fn prepare(ed: &Editor, req: &DragRequest) -> Result<Prepared, String> {
    if !req.frame.is_finite() || !req.start.x.is_finite() || !req.start.y.is_finite() || !req.delta.x.is_finite() || !req.delta.y.is_finite() {
        return Err("non-finite number in request".into());
    }
    let engine = ed.engine();
    let frame = engine.evaluate(req.frame);
    let mut sel = Selection::from_items(req.selection.clone());
    sel.prune(engine);
    if sel.is_empty() {
        return Err("nothing selected".into());
    }
    let points_only = sel.items.iter().all(|t| matches!(t, Target::Point { .. }));
    let bone = match (&req.handle, sel.items.as_slice()) {
        (Handle::BoneHead | Handle::BoneTip, [Target::Bone { name }]) => Some(bone_ends(engine, &frame, name).ok_or("bone not found")?),
        (Handle::BoneHead | Handle::BoneTip, _) => return Err("bone handles need exactly one selected bone".into()),
        _ => None,
    };
    let bx = if points_only || bone.is_some() { None } else { box_frame(engine, &frame, &sel, MIN_EXTENT) };
    if points_only && !matches!(req.handle, Handle::Move) {
        return Err("control points can only be moved".into());
    }
    if bx.is_none() && bone.is_none() && !points_only {
        return Err("selection has no geometry".into());
    }
    let targets = if req.modifiers.no_snap { SnapTargets::default() } else { snap_targets(engine, &frame, &sel, &req.snap) };
    // points move like a degenerate box around their own position
    let pt_box;
    let bx_ref = if points_only {
        let pts: Vec<Vec2> = sel.items.iter().flat_map(|t| pick::target_points(engine, &frame, t)).collect();
        let lo = Vec2::new(pts.iter().map(|q| q.x).fold(f32::INFINITY, f32::min), pts.iter().map(|q| q.y).fold(f32::INFINITY, f32::min));
        let hi = Vec2::new(pts.iter().map(|q| q.x).fold(f32::NEG_INFINITY, f32::max), pts.iter().map(|q| q.y).fold(f32::NEG_INFINITY, f32::max));
        pt_box = BoxFrame::from_aabb(lo, hi);
        Some(&pt_box)
    } else {
        bx.as_ref()
    };
    let d = drag_matrix(bx_ref, bone, req.handle, req.start, req.delta, &req.modifiers, &req.snap, &targets, req.pivot)?;
    let plan = plan(ed, req.frame, &sel, &d.matrix, req.auto_key, &req.locked)?;

    let mut outline = Vec::new();
    for t in &sel.items {
        match t {
            Target::Layer { path } => {
                let prefix = format!("{path}/");
                for (id, n) in frame.nodes.iter().enumerate() {
                    if &n.name == path || n.name.starts_with(&prefix) {
                        for s in pick::node_shapes(&frame, id) {
                            outline.push(Outline { pts: s.pts.iter().map(|q| d.matrix.apply(*q)).collect(), closed: s.closed });
                        }
                    }
                }
            }
            Target::Bone { name } => {
                if let Some((h, t)) = bone_ends(engine, &frame, name) {
                    outline.push(Outline { pts: vec![d.matrix.apply(h), d.matrix.apply(t)], closed: false });
                }
            }
            Target::Point { .. } => {}
        }
    }
    let box_after = bx.map(|b| {
        let c = b.corners();
        [d.matrix.apply(c[0]), d.matrix.apply(c[1]), d.matrix.apply(c[2]), d.matrix.apply(c[3])]
    });
    Ok(Prepared { d, plan, sel, outline, box_after })
}

pub(super) fn preview_of(p: &Prepared) -> DragPreview {
    let m = p.d.matrix;
    DragPreview {
        matrix: [m.a, m.b, m.c, m.d, m.tx, m.ty],
        summary: p.d.summary.clone(),
        guides: p.d.guides.clone(),
        outline: p.outline.clone(),
        box_after: p.box_after,
        edits: p.plan.edits.clone(),
        changed: !p.plan.ops.is_empty(),
    }
}

/// What the drag would do, without touching the project.
pub fn preview(ed: &Editor, req: &DragRequest) -> Result<DragPreview, String> {
    Ok(preview_of(&prepare(ed, req)?))
}

/// Apply the drag as one undoable batch. A drag that changes nothing leaves no history entry.
pub fn commit(ed: &mut Editor, req: &DragRequest) -> Result<DragPreview, Vec<String>> {
    let p = prepare(ed, req).map_err(|e| vec![e])?;
    let out = preview_of(&p);
    if !p.plan.ops.is_empty() {
        ed.apply(Op::Batch { ops: p.plan.ops })?;
    }
    let _ = p.sel;
    Ok(out)
}

/// Nudge the selection by `(dx, dy)` canvas units as one undoable step (no snapping).
pub fn nudge(ed: &mut Editor, frame: f32, selection: &[Target], delta: Vec2, auto_key: bool, locked: &[String]) -> Result<DragPreview, Vec<String>> {
    let engine = ed.engine();
    let mut sel = Selection::from_items(selection.to_vec());
    sel.prune(engine);
    if sel.is_empty() {
        return Err(vec!["nothing selected".into()]);
    }
    let d = Affine2::translate(delta);
    let plan = plan(ed, frame, &sel, &d, auto_key, locked).map_err(|e| vec![e])?;
    let out = DragPreview {
        matrix: [1.0, 0.0, 0.0, 1.0, delta.x, delta.y],
        summary: format!("nudge {:.1}, {:.1}", delta.x, delta.y),
        guides: vec![],
        outline: vec![],
        box_after: None,
        edits: plan.edits.clone(),
        changed: !plan.ops.is_empty(),
    };
    if !plan.ops.is_empty() {
        ed.apply(Op::Batch { ops: plan.ops })?;
    }
    Ok(out)
}

/// Delete the selected layers as one undoable step.
pub fn delete(ed: &mut Editor, selection: &[Target]) -> Result<(), Vec<String>> {
    let mut sel = Selection::from_items(selection.to_vec());
    sel.prune(ed.engine());
    let op = delete_op(ed, &sel).map_err(|e| vec![e])?;
    ed.apply(op)
}
