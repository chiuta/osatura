// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `gizmo` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// Drag maths
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Modifiers {
    /// Axis lock (move), 15 degree steps (rotate), proportional (scale), rotate only (bone tip).
    pub constrain: bool,
    /// Scale around the centre instead of the opposite handle.
    pub center: bool,
    /// Disable snapping.
    pub no_snap: bool,
}

#[derive(Clone, Debug)]
pub struct DragMatrix {
    pub matrix: Affine2,
    pub guides: Vec<Guide>,
    pub summary: String,
}

pub(super) fn wrap_pi(a: f32) -> f32 {
    crate::rig::wrap_pi(a)
}

/// The canvas-space map for dragging `handle` from `start` by `delta`. `bx` is the selection box
/// (for bone handles pass the bone's ends in `bone` instead).
#[allow(clippy::too_many_arguments)]
pub fn drag_matrix(bx: Option<&BoxFrame>, bone: Option<(Vec2, Vec2)>, handle: Handle, start: Vec2, delta: Vec2, m: &Modifiers, snap: &SnapOptions, targets: &SnapTargets, pivot: Option<Vec2>) -> Result<DragMatrix, String> {
    let snap_on = !m.no_snap;
    match handle {
        Handle::Move | Handle::BoneHead => {
            let (lo, hi) = match (handle, bone, bx) {
                (Handle::BoneHead, Some((h, _)), _) => (h, h),
                (_, _, Some(b)) => {
                    let c = b.corners();
                    (
                        Vec2::new(c.iter().map(|q| q.x).fold(f32::INFINITY, f32::min), c.iter().map(|q| q.y).fold(f32::INFINITY, f32::min)),
                        Vec2::new(c.iter().map(|q| q.x).fold(f32::NEG_INFINITY, f32::max), c.iter().map(|q| q.y).fold(f32::NEG_INFINITY, f32::max)),
                    )
                }
                _ => return Err("nothing to move".into()),
            };
            let (d, guides) = if snap_on {
                snap_move(lo, hi, delta, m.constrain, snap, targets)
            } else if m.constrain {
                snap_move(lo, hi, delta, true, &SnapOptions { grid: None, objects: false, canvas: false, tolerance: 0.0 }, &SnapTargets::default())
            } else {
                (delta, vec![])
            };
            Ok(DragMatrix { matrix: Affine2::translate(d), guides, summary: format!("move {:.1}, {:.1}", d.x, d.y) })
        }
        Handle::Rotate => {
            let b = bx.ok_or("rotate needs a box")?;
            let pv = pivot.unwrap_or_else(|| b.centre());
            let (v0, v1) = (start - pv, start + delta - pv);
            if v0.length() < 1e-3 || v1.length() < 1e-6 {
                return Err("pointer is on the pivot".into());
            }
            let mut ang = wrap_pi(v1.y.atan2(v1.x) - v0.y.atan2(v0.x));
            if m.constrain {
                let step = 15f32.to_radians();
                ang = (ang / step).round() * step;
            }
            let mat = Affine2::translate(pv).mul(Affine2::rotate(ang)).mul(Affine2::translate(-pv));
            Ok(DragMatrix { matrix: mat, guides: vec![], summary: format!("rotate {:.1} deg", ang.to_degrees()) })
        }
        Handle::Scale { at } => {
            let b = bx.ok_or("scale needs a box")?;
            if !b.nondegenerate() {
                return Err("selection has no extent to scale".into());
            }
            let basis = b.basis();
            let inv = basis.inverse().ok_or("degenerate box")?;
            let (hx, hy) = at.local();
            let (q0, q1) = (inv.apply(start), inv.apply(start + delta));
            let pl = if m.center { (0.5, 0.5) } else { (1.0 - hx, 1.0 - hy) };
            let pv = basis.apply(Vec2::new(pl.0, pl.1));
            let factor = |q0: f32, q1: f32, p: f32| -> Result<f32, String> {
                let den = q0 - p;
                if den.abs() < 1e-6 {
                    return Err("pointer started on the pivot".into());
                }
                Ok(clamp_factor((q1 - p) / den))
            };
            let mut sx = if hx == 0.5 { 1.0 } else { factor(q0.x, q1.x, pl.0)? };
            let mut sy = if hy == 0.5 { 1.0 } else { factor(q0.y, q1.y, pl.1)? };
            if m.constrain {
                let s = if at.is_corner() {
                    let (a, c) = (start - pv, start + delta - pv);
                    let l2 = a.dot(a);
                    if l2 < 1e-9 {
                        return Err("pointer started on the pivot".into());
                    }
                    clamp_factor(c.dot(a) / l2)
                } else if hx == 0.5 {
                    sy
                } else {
                    sx
                };
                sx = s;
                sy = s;
            }
            let pl_v = Vec2::new(pl.0, pl.1);
            let mat = basis.mul(Affine2::translate(pl_v)).mul(Affine2::scale_xy(sx, sy)).mul(Affine2::translate(-pl_v)).mul(inv);
            Ok(DragMatrix { matrix: mat, guides: vec![], summary: format!("scale {sx:.3} x {sy:.3}") })
        }
        Handle::BoneTip => {
            let (h, t) = bone.ok_or("bone tip needs a bone")?;
            let v0 = t - h;
            let v1 = v0 + delta;
            if v0.length() < 1e-6 || v1.length() < 1e-6 {
                return Err("bone has no length".into());
            }
            let (a0, a1) = (v0.y.atan2(v0.x), v1.y.atan2(v1.x));
            let mut ang = wrap_pi(a1 - a0);
            let mut s = v1.length() / v0.length();
            if m.constrain {
                let step = 15f32.to_radians();
                ang = wrap_pi((a1 / step).round() * step - a0);
                s = 1.0;
            }
            let mat = Affine2::translate(h).mul(Affine2::rotate(ang)).mul(Affine2::scale(s)).mul(Affine2::translate(-h));
            Ok(DragMatrix { matrix: mat, guides: vec![], summary: format!("bone tip: rotate {:.1} deg, length x{:.3}", ang.to_degrees(), s) })
        }
    }
}

pub(super) fn clamp_factor(s: f32) -> f32 {
    if s.abs() < 1e-3 {
        if s < 0.0 {
            -1e-3
        } else {
            1e-3
        }
    } else {
        s
    }
}

/// Split a local matrix into layer transform properties (`T(pos) R Shear S T(-anchor)`),
/// choosing the rotation closest to `prev_rot_deg` so key values stay continuous and a mirror shows
/// up as a negative scale rather than a half-turn.
pub fn decompose(m: &Affine2, anchor: Vec2, prev_rot_deg: f32) -> Result<Transform2D, String> {
    let prev = prev_rot_deg.to_radians();
    let a0 = m.b.atan2(m.a);
    let cand = if wrap_pi(a0 - prev).abs() <= wrap_pi(a0 + std::f32::consts::PI - prev).abs() { a0 } else { a0 + std::f32::consts::PI };
    let rot = prev + wrap_pi(cand - prev);
    let (s, c) = rot.sin_cos();
    let sx = m.a * c + m.b * s;
    let sy = -m.c * s + m.d * c;
    let k = m.c * c + m.d * s;
    if sy.abs() < 1e-6 || sx.abs() < 1e-6 {
        return Err("transform would collapse the layer".into());
    }
    let skew = (k / sy).atan();
    let pos = Vec2::new(m.tx + m.a * anchor.x + m.c * anchor.y, m.ty + m.b * anchor.x + m.d * anchor.y);
    Ok(Transform2D { position: pos, rotation: rot.to_degrees(), scale: Vec2::new(sx, sy), skew: skew.to_degrees(), anchor })
}

// ---------------------------------------------------------------------------------------------
// Edit planning
// ---------------------------------------------------------------------------------------------

/// One property change, for reporting.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditInfo {
    pub target: String,
    pub prop: String,
    pub from: f32,
    pub to: f32,
    /// `key` (keyframe at the current frame) or `static`.
    pub mode: String,
}

pub struct Plan {
    pub ops: Vec<Op>,
    pub edits: Vec<EditInfo>,
}

pub(super) fn eps_changed(a: f32, b: f32) -> bool {
    (a - b).abs() > 1e-4
}

pub(super) fn keyed_index(engine: &Engine, target: &str, prop: Prop) -> Option<usize> {
    engine.project.channels.iter().position(|c| c.target == target && c.prop == prop && matches!(c.source, Source::Keys { .. }) && !crate::skins::is_collapse_channel(c))
}

pub(super) fn prop_name(p: Prop) -> String {
    serde_json::to_value(p).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default()
}

/// Op that sets a key at `frame` so that the property's total becomes `new_total`.
pub(super) fn key_op(engine: &Engine, target: &str, prop: Prop, frame: f32, old_total: f32, new_total: f32, base: f32) -> Result<Op, String> {
    crate::chanmeta::assert_unlocked(&engine.project, target, prop)?;
    match keyed_index(engine, target, prop) {
        Some(i) => {
            let mut ch = engine.project.channels[i].clone();
            let Source::Keys { track } = &mut ch.source else { unreachable!() };
            let cur = track.eval(frame).unwrap_or(base);
            let v = new_total - (old_total - cur);
            let ease = track.keys.iter().find(|k| k.frame == frame).map(|k| k.ease).unwrap_or(Ease::Linear);
            track.set_key(frame, v, ease);
            Ok(Op::Set { path: format!("/channels/{i}"), value: serde_json::to_value(&ch).map_err(|e| e.to_string())? })
        }
        None => {
            let ch = Channel::keys(target, prop, crate::anim::Track::with_keys(&[(frame, new_total, Ease::Linear)]));
            Ok(Op::Insert { path: "/channels/-".into(), value: serde_json::to_value(&ch).map_err(|e| e.to_string())? })
        }
    }
}

pub(super) fn layer_ptr(engine: &Engine, id: usize) -> String {
    let ix = engine.node_idx_path(id);
    let mut s = format!("/layers/{}", ix[0]);
    for i in &ix[1..] {
        s.push_str(&format!("/layers/{i}"));
    }
    s
}

pub(super) fn has_selected_ancestor(path: &str, all: &[&str]) -> bool {
    all.iter().any(|a| *a != path && path.starts_with(&format!("{a}/")))
}

pub(super) fn bone_driven(engine: &Engine, name: &str) -> Option<String> {
    let p = &engine.project;
    if let Some(c) = p.ik.iter().find(|c| c.bones.iter().any(|b| b == name)) {
        return Some(format!("bone '{name}' is driven by IK chain '{}'; drag its target bone instead", c.name));
    }
    if let Some(c) = p.constraints.iter().find(|c| c.bone == name) {
        return Some(format!("bone '{name}' is driven by constraint '{}'", c.name));
    }
    if p.springs.iter().any(|s| s.bone == name) {
        return Some(format!("bone '{name}' has a spring; its rotation is simulated"));
    }
    None
}

/// Turn the canvas map `d` into edits for everything in `sel` at `frame_no`.
pub fn plan(ed: &Editor, frame_no: f32, sel: &Selection, d: &Affine2, auto_key: bool, locked: &[String]) -> Result<Plan, String> {
    let engine = ed.engine();
    let frame = engine.evaluate(frame_no);
    let mut ops: Vec<Op> = Vec::new();
    let mut edits: Vec<EditInfo> = Vec::new();

    let has_points = sel.items.iter().any(|t| matches!(t, Target::Point { .. }));
    if has_points && !sel.items.iter().all(|t| matches!(t, Target::Point { .. })) {
        return Err("control points cannot be edited together with layers or bones".into());
    }

    if has_points {
        plan_points(engine, &frame, frame_no, sel, d, auto_key, locked, &mut ops, &mut edits)?;
    } else {
        let layers = sel.layers();
        for path in &layers {
            if has_selected_ancestor(path, &layers) {
                continue;
            }
            if locked.iter().any(|l| l == path || path.starts_with(&format!("{l}/"))) || engine.project.is_layer_locked(path) {
                return Err(format!("layer '{path}' is locked"));
            }
            plan_layer(engine, &frame, frame_no, path, d, auto_key, &mut ops, &mut edits)?;
        }
        let bones = sel.bones();
        for name in &bones {
            // skip bones under another selected bone: they follow it
            let mut anc = engine.project.bones.iter().find(|b| b.name == *name).and_then(|b| b.parent.clone());
            let mut skip = false;
            while let Some(a) = anc {
                if bones.contains(&a.as_str()) {
                    skip = true;
                    break;
                }
                anc = engine.project.bones.iter().find(|b| b.name == a).and_then(|b| b.parent.clone());
            }
            if skip {
                continue;
            }
            if locked.iter().any(|l| l == &format!("bone:{name}")) {
                return Err(format!("bone '{name}' is locked"));
            }
            plan_bone(engine, &frame, frame_no, name, d, &mut ops, &mut edits)?;
        }
    }
    Ok(Plan { ops, edits })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_layer(engine: &Engine, frame: &Frame, frame_no: f32, path: &str, d: &Affine2, auto_key: bool, ops: &mut Vec<Op>, edits: &mut Vec<EditInfo>) -> Result<(), String> {
    let id = engine.node_index(path).ok_or_else(|| format!("no layer '{path}'"))?;
    let layer = engine.project.find_layer(path).ok_or_else(|| format!("no layer '{path}'"))?;
    let node = &frame.nodes[id];
    let old = engine.layer_local_at(frame_no, id);
    let lm = old.matrix();
    let inv_lm = lm.inverse().ok_or_else(|| format!("layer '{path}' has a singular transform"))?;
    let parent = node.world.mul(inv_lm);
    let pinv = parent.inverse().ok_or("parent transform is singular")?;
    let new = decompose(&pinv.mul(*d).mul(parent).mul(lm), old.anchor, old.rotation).map_err(|e| format!("layer '{path}': {e}"))?;
    let st = &layer.transform;
    let ptr = layer_ptr(engine, id);

    let props: [(Prop, f32, f32, f32); 6] = [
        (Prop::LayerX, old.position.x, new.position.x, st.position.x),
        (Prop::LayerY, old.position.y, new.position.y, st.position.y),
        (Prop::LayerRotation, old.rotation, new.rotation, st.rotation),
        (Prop::LayerScaleX, old.scale.x, new.scale.x, st.scale.x),
        (Prop::LayerScaleY, old.scale.y, new.scale.y, st.scale.y),
        (Prop::LayerSkew, old.skew, new.skew, st.skew),
    ];
    let mut spos = st.position;
    let mut sscale = st.scale;
    let (mut pos_static, mut scale_static) = (false, false);
    for (prop, o, n, base) in props {
        if !eps_changed(o, n) {
            continue;
        }
        let keyed = keyed_index(engine, path, prop).is_some();
        if keyed || auto_key {
            ops.push(key_op(engine, path, prop, frame_no, o, n, base)?);
            edits.push(EditInfo { target: path.into(), prop: prop_name(prop), from: o, to: n, mode: "key".into() });
        } else {
            crate::chanmeta::assert_unlocked(&engine.project, path, prop)?;
            // static value: shift by the same amount the evaluated value moves
            let v = base + (n - o);
            match prop {
                Prop::LayerX => {
                    spos.x = v;
                    pos_static = true;
                }
                Prop::LayerY => {
                    spos.y = v;
                    pos_static = true;
                }
                Prop::LayerScaleX => {
                    sscale.x = v;
                    scale_static = true;
                }
                Prop::LayerScaleY => {
                    sscale.y = v;
                    scale_static = true;
                }
                Prop::LayerRotation => ops.push(Op::Set { path: format!("{ptr}/transform/rotation"), value: json!(v) }),
                _ => ops.push(Op::Set { path: format!("{ptr}/transform/skew"), value: json!(v) }),
            }
            edits.push(EditInfo { target: path.into(), prop: prop_name(prop), from: o, to: v, mode: "static".into() });
        }
    }
    if pos_static {
        ops.push(Op::Set { path: format!("{ptr}/transform/position"), value: json!([spos.x, spos.y]) });
    }
    if scale_static {
        ops.push(Op::Set { path: format!("{ptr}/transform/scale"), value: json!([sscale.x, sscale.y]) });
    }
    Ok(())
}

pub(super) fn plan_bone(engine: &Engine, frame: &Frame, frame_no: f32, name: &str, d: &Affine2, ops: &mut Vec<Op>, edits: &mut Vec<EditInfo>) -> Result<(), String> {
    if let Some(why) = bone_driven(engine, name) {
        return Err(why);
    }
    let bi = engine.bone_index(name).ok_or_else(|| format!("no bone '{name}'"))?;
    let bone = &engine.project.bones[bi];
    let parent_w = bone.parent.as_ref().and_then(|p| engine.bone_index(p)).map(|pi| frame.world[pi]).unwrap_or(Affine2::IDENTITY);
    let pinv = parent_w.inverse().ok_or("parent bone transform is singular")?;
    let cam_inv = frame.camera.inverse().ok_or("camera is singular")?;
    let w = frame.world[bi];
    let (head0, tip0) = (w.apply(Vec2::ZERO), w.apply(Vec2::new(bone.length, 0.0)));
    // the map acts in canvas space; bring head / tip there and back
    let (h1, t1) = (cam_inv.apply(d.apply(frame.camera.apply(head0))), cam_inv.apply(d.apply(frame.camera.apply(tip0))));

    let pscale = parent_w.x_scale().max(1e-6);
    let off_old = pinv.apply(head0) - bone.rest_offset;
    let off_new = pinv.apply(h1) - bone.rest_offset;
    let ang_old = (w.rotation() - parent_w.rotation()).to_degrees() - bone.rest_angle;
    let dir = t1 - h1;
    let ang_target = (dir.y.atan2(dir.x) - parent_w.rotation()).to_degrees() - bone.rest_angle;
    // keep key values continuous: pick the representative nearest to the current value
    let ang_new = ang_old + wrap_pi((ang_target - ang_old).to_radians()).to_degrees();
    let sc_old = w.x_scale() / pscale;
    let sc_new = if bone.length > 1e-6 { dir.length() / (bone.length * pscale) } else { sc_old };

    let items: [(Prop, f32, f32, f32); 4] = [
        (Prop::BoneOffsetX, off_old.x, off_new.x, 0.0),
        (Prop::BoneOffsetY, off_old.y, off_new.y, 0.0),
        (Prop::BoneAngle, ang_old, ang_new, 0.0),
        (Prop::BoneScale, sc_old, sc_new, 1.0),
    ];
    for (prop, o, n, base) in items {
        let tol = if prop == Prop::BoneScale { 1e-3 * o.abs().max(1.0) } else { 1e-3 };
        if (o - n).abs() <= tol {
            continue;
        }
        ops.push(key_op(engine, name, prop, frame_no, o, n, base)?);
        edits.push(EditInfo { target: name.into(), prop: prop_name(prop), from: o, to: n, mode: "key".into() });
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_points(engine: &Engine, frame: &Frame, frame_no: f32, sel: &Selection, d: &Affine2, auto_key: bool, locked: &[String], ops: &mut Vec<Op>, edits: &mut Vec<EditInfo>) -> Result<(), String> {
    for t in &sel.items {
        let Target::Point { layer: lp, point } = t else { continue };
        if locked.iter().any(|l| lp == l || lp.starts_with(&format!("{l}/"))) || engine.project.is_layer_locked(lp) {
            return Err(format!("layer '{lp}' is locked"));
        }
        let id = engine.node_index(lp).ok_or_else(|| format!("no layer '{lp}'"))?;
        let layer = engine.project.find_layer(lp).ok_or_else(|| format!("no layer '{lp}'"))?;
        let node = &frame.nodes[id];
        let winv = node.world.inverse().ok_or("layer transform is singular")?;
        let cur = pick::layer_points(engine, frame, lp).into_iter().find(|(k, _)| k == point).map(|(_, p)| p).ok_or_else(|| format!("control point not found on '{lp}'"))?;
        let delta = winv.apply(d.apply(cur)) - winv.apply(cur);
        if delta.length() < 1e-5 {
            continue;
        }
        let ptr = layer_ptr(engine, id);
        let vec_json = |v: Vec2| json!([v.x, v.y]);

        // keyed offset helper: channels `Dx`/`Dy` on "layer::name::sub"
        let offset_edit = |ops: &mut Vec<Op>, edits: &mut Vec<EditInfo>, name: &str, dx: Prop, dy: Prop, stat: Vec2, static_ops: &mut dyn FnMut() -> Vec<Op>| -> Result<(), String> {
            let tgt = format!("{lp}::{name}");
            let key_x = keyed_index(engine, &tgt, dx);
            let key_y = keyed_index(engine, &tgt, dy);
            let eval = |ix: Option<usize>, fallback: f32| -> f32 {
                ix.and_then(|i| match &engine.project.channels[i].source {
                    Source::Keys { track } => track.eval(frame_no),
                    _ => None,
                })
                .unwrap_or(fallback)
            };
            let (ox, oy) = (eval(key_x, stat.x), eval(key_y, stat.y));
            if key_x.is_some() || key_y.is_some() || auto_key {
                for (prop, o, delta_c) in [(dx, ox, delta.x), (dy, oy, delta.y)] {
                    if delta_c.abs() < 1e-6 {
                        continue;
                    }
                    ops.push(key_op(engine, &tgt, prop, frame_no, o, o + delta_c, o)?);
                    edits.push(EditInfo { target: tgt.clone(), prop: prop_name(prop), from: o, to: o + delta_c, mode: "key".into() });
                }
            } else {
                ops.extend(static_ops());
                edits.push(EditInfo { target: tgt, prop: "static_offset".into(), from: 0.0, to: delta.length(), mode: "static".into() });
            }
            Ok(())
        };

        match point {
            PointKind::Anchor { path, index } => {
                let LayerKind::Vector { paths } = &layer.kind else { return Err("not a vector layer".into()) };
                let p = paths.get(*path).ok_or("no such path")?;
                let a = p.anchors.get(*index).ok_or("no such anchor")?;
                let pname = if p.name.is_empty() { format!("path{path}") } else { p.name.clone() };
                let tgt = format!("{lp}::{pname}::{index}");
                let (kx, ky) = (keyed_index(engine, &tgt, Prop::PointDx), keyed_index(engine, &tgt, Prop::PointDy));
                if kx.is_some() || ky.is_some() || auto_key {
                    let ev = |ix: Option<usize>| ix.and_then(|i| match &engine.project.channels[i].source {
                        Source::Keys { track } => track.eval(frame_no),
                        _ => None,
                    });
                    for (prop, o, dc) in [(Prop::PointDx, ev(kx).unwrap_or(0.0), delta.x), (Prop::PointDy, ev(ky).unwrap_or(0.0), delta.y)] {
                        if dc.abs() < 1e-6 {
                            continue;
                        }
                        ops.push(key_op(engine, &tgt, prop, frame_no, o, o + dc, 0.0)?);
                        edits.push(EditInfo { target: tgt.clone(), prop: prop_name(prop), from: o, to: o + dc, mode: "key".into() });
                    }
                } else {
                    let base = format!("{ptr}/paths/{path}/anchors/{index}");
                    ops.push(Op::Set { path: format!("{base}/pos"), value: vec_json(a.pos + delta) });
                    if let Some(h) = a.handle_in {
                        ops.push(Op::Set { path: format!("{base}/handle_in"), value: vec_json(h + delta) });
                    }
                    if let Some(h) = a.handle_out {
                        ops.push(Op::Set { path: format!("{base}/handle_out"), value: vec_json(h + delta) });
                    }
                    edits.push(EditInfo { target: tgt, prop: "anchor".into(), from: 0.0, to: delta.length(), mode: "static".into() });
                }
            }
            PointKind::HandleIn { path, index } | PointKind::HandleOut { path, index } => {
                let LayerKind::Vector { paths } = &layer.kind else { return Err("not a vector layer".into()) };
                let a = paths.get(*path).and_then(|p| p.anchors.get(*index)).ok_or("no such anchor")?;
                let (key, h) = if matches!(point, PointKind::HandleIn { .. }) { ("handle_in", a.handle_in) } else { ("handle_out", a.handle_out) };
                let h = h.ok_or("handle is not set")?;
                ops.push(Op::Set { path: format!("{ptr}/paths/{path}/anchors/{index}/{key}"), value: vec_json(h + delta) });
                edits.push(EditInfo { target: format!("{lp}::path{path}::{index}"), prop: key.into(), from: 0.0, to: delta.length(), mode: "static".into() });
            }
            PointKind::MeshVertex { index } => {
                let LayerKind::Image { mesh: Some(m), .. } = &layer.kind else { return Err("not a mesh layer".into()) };
                let f = m.free.as_ref().ok_or("only free-form meshes have editable vertices")?;
                let v = *f.vertices.get(*index).ok_or("no such vertex")?;
                ops.push(Op::Set { path: format!("{ptr}/mesh/free/vertices/{index}"), value: vec_json(v + delta) });
                edits.push(EditInfo { target: format!("{lp}::mesh::{index}"), prop: "vertex".into(), from: 0.0, to: delta.length(), mode: "static".into() });
            }
            PointKind::Cage { deformer, index } | PointKind::Pin { deformer, index } | PointKind::Spine { deformer, index } | PointKind::Lattice { deformer, index } => {
                let def = layer.deformers.get(*deformer).ok_or("no such deformer")?;
                let (name, offsets, n, dxp, dyp, sub) = match (def, point) {
                    (Deformer::Cage { name, offsets, rest }, PointKind::Cage { .. }) => (name.clone(), offsets.clone(), rest.len(), Prop::CageDx, Prop::CageDy, index.to_string()),
                    (Deformer::Pin { name, offsets, pins, .. }, PointKind::Pin { .. }) => (name.clone(), offsets.clone(), pins.len(), Prop::PinDx, Prop::PinDy, index.to_string()),
                    (Deformer::Spine { name, offsets, rest, .. }, PointKind::Spine { .. }) => (name.clone(), offsets.clone(), rest.len(), Prop::SpineDx, Prop::SpineDy, index.to_string()),
                    (Deformer::Lattice { name, offsets, cols, rows, .. }, PointKind::Lattice { .. }) => {
                        (name.clone(), offsets.clone(), (cols + 1) * (rows + 1), Prop::LatticeDx, Prop::LatticeDy, format!("{},{}", index % (cols + 1), index / (cols + 1)))
                    }
                    _ => return Err("point kind does not match the deformer".into()),
                };
                if *index >= n {
                    return Err("no such deformer point".into());
                }
                let stat = offsets.get(*index).copied().unwrap_or(Vec2::ZERO);
                let off_ptr = format!("{ptr}/deformers/{deformer}/offsets");
                let mut static_ops = || -> Vec<Op> {
                    let mut all = if offsets.len() == n { offsets.clone() } else { vec![Vec2::ZERO; n] };
                    all[*index] += delta;
                    vec![Op::Set { path: off_ptr.clone(), value: json!(all.iter().map(|v| [v.x, v.y]).collect::<Vec<_>>()) }]
                };
                offset_edit(ops, edits, &format!("{name}::{sub}"), dxp, dyp, stat, &mut static_ops)?;
            }
        }
    }
    Ok(())
}
