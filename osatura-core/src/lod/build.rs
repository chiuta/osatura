// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Building levels (part of `lod`; see the module docs in `mod.rs`).
use super::*;

// Building levels

pub(super) fn path_eligible(l: &Layer, path: &VPath) -> bool {
    if path.anchors.len() < 4 || !path.morphs.is_empty() || path.anchors.iter().any(|a| !a.weights.is_empty()) {
        return false;
    }
    if !l.deformers.is_empty() || l.wobble.is_some() {
        return false;
    }
    match &path.stroke {
        None => true,
        Some(s) => s.join == crate::style::Join::Round && s.dash.is_empty() && s.width_profile.is_empty(),
    }
}

pub(super) fn mask_targets(ls: &[Layer], out: &mut BTreeSet<String>) {
    for l in ls {
        if let Some(m) = &l.mask {
            out.insert(m.layer.clone());
        }
        mask_targets(l.children(), out);
    }
}

/// Mesh nodes that may be collapsed: image layers with a mesh and without deformers / wobble /
/// point offsets, whose texture coordinates are an affine function of the position.
pub(super) fn mesh_eligible(c: &Ctx, id: usize) -> bool {
    let Some(l) = c.layer(id) else { return false };
    let LayerKind::Image { mesh: Some(m), .. } = &l.kind else { return false };
    if !l.deformers.is_empty() || l.wobble.is_some() || c.complex[id] {
        return false;
    }
    if let Some(f) = &m.free {
        if !f.uvs.is_empty() {
            if f.uvs.len() != f.vertices.len() || f.vertices.len() < 3 {
                return false;
            }
            // uv = M * pos + t globally: fit through the first three non-collinear vertices
            let v = &f.vertices;
            let u = &f.uvs;
            let mut base = None;
            'outer: for i in 1..v.len() {
                for j in i + 1..v.len() {
                    let d = area2(p_of(v[0]), p_of(v[i]), p_of(v[j]));
                    if d.abs() > 1e-9 {
                        base = Some((i, j, d));
                        break 'outer;
                    }
                }
            }
            let Some((i, j, _)) = base else { return false };
            let tri = [p_of(v[0]), p_of(v[i]), p_of(v[j])];
            let size = v.iter().fold(0.0f64, |a, q| a.max((q.x as f64).abs()).max((q.y as f64).abs())).max(1.0);
            for k in 0..v.len() {
                let lam = bary(p_of(v[k]), tri);
                for ax in 0..2 {
                    let get = |q: Vec2| if ax == 0 { q.x as f64 } else { q.y as f64 };
                    let pred = lam[0] * get(u[0]) + lam[1] * get(u[i]) + lam[2] * get(u[j]);
                    if (pred - get(u[k])).abs() > 1e-4 * size.max(1.0) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Every string value of the structures that refer to layers by name (containers, scrolls, links, glues, machines, bindings,
/// mesh locks, shots, clips): a layer named there is not culled.
pub(super) fn referenced_strings(p: &Project) -> BTreeSet<String> {
    fn collect(v: &serde_json::Value, out: &mut BTreeSet<String>) {
        match v {
            serde_json::Value::String(s) => {
                out.insert(s.clone());
            }
            serde_json::Value::Array(a) => a.iter().for_each(|x| collect(x, out)),
            serde_json::Value::Object(o) => o.values().for_each(|x| collect(x, out)),
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    for v in [
        serde_json::to_value(&p.containers),
        serde_json::to_value(&p.scrolls),
        serde_json::to_value(&p.mesh_locks),
        serde_json::to_value(&p.glues),
        serde_json::to_value(&p.machines),
        serde_json::to_value(&p.bindings),
        serde_json::to_value(&p.shots),
        serde_json::to_value(&p.clips),
    ].into_iter().flatten() {
        collect(&v, &mut out);
    }
    out
}

/// Paths of the layers a container (member) or scroll (clip layer) lays out by their geometry bounds. Simplifying the
/// path of such a layer, or culling a part of it, changes its bounding box and the solver re-flows the siblings: the
/// motion of the siblings would then be an error that the declared bound does not cover. These layers (and everything
/// below a member group) are therefore left exactly as they are (`skipped: layout member` in the level notes); the
/// declared bound stays a proof, at the price of not reducing those layers.
pub(super) fn layout_members(p: &Project) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for c in &p.containers {
        for m in &c.members {
            if let Some(l) = &m.layer {
                out.insert(l.clone());
            }
        }
    }
    for s in &p.scrolls {
        if let Some(l) = &s.clip_layer {
            out.insert(l.clone());
        }
    }
    out
}

/// `path` is a layout member or lies below one.
pub(super) fn is_layout_dependent(members: &BTreeSet<String>, path: &str) -> bool {
    members.iter().any(|m| path == m || (path.starts_with(m.as_str()) && path.as_bytes().get(m.len()) == Some(&b'/')))
}

pub(super) fn is_boundary_layer_kind(k: &LayerKind) -> bool {
    matches!(k, LayerKind::Vector { .. } | LayerKind::Image { .. })
}

/// Levels L0..Ln of `p`. L0 is an unchanged copy. See the module notes for the guarantees.
pub fn build(p: &Project, o: &LodOptions) -> Result<Vec<Lod>, String> {
    o.check()?;
    Engine::new(p.clone()).map_err(|e| format!("project does not validate: {}", e.join("; ")))?;
    let mut base_rec = LodRecord {
        level: 0,
        tolerance_px: 0.0,
        scale: o.scale,
        declared_px: 0.0,
        bound_stage: 0.0,
        path_px: 0.0,
        mesh_px: 0.0,
        keys_px: 0.0,
        cull_px: 0.0,
        stats: count_stats(p),
        notes: vec![],
    };
    let mut out = vec![Lod { project: p.clone(), record: base_rec.clone() }];
    base_rec.level = 0;
    let finish = |mut q: Project, mut rec: LodRecord| -> Lod {
        q.lods.clear();
        rec.stats = count_stats(&q);
        rec.declared_px = ceil6(rec.declared_px);
        rec.path_px = ceil6(rec.path_px);
        rec.mesh_px = ceil6(rec.mesh_px);
        rec.keys_px = ceil6(rec.keys_px);
        rec.cull_px = ceil6(rec.cull_px);
        rec.bound_stage = ceil6(rec.declared_px / rec.scale);
        q.lods = vec![rec.clone()];
        Lod { project: q, record: rec }
    };
    if links_present(p) {
        for k in 1..=o.levels {
            let mut rec = base_rec.clone();
            rec.level = k;
            rec.tolerance_px = o.level_tolerance(k);
            rec.notes.push("mesh locks / glues present: no reduction applied".into());
            out.push(finish(p.clone(), rec));
        }
        return Ok(out);
    }
    // lossless step shared by all levels
    let (mut pb, mut dead) = crate::sizefit::drop_dead_layers(p);
    if Engine::new(pb.clone()).is_err() {
        // dropping dead layers broke a reference (container, machine, ...): keep them
        pb = p.clone();
        dead.clear();
    }
    let referenced = referenced_strings(&pb);
    let layout_dep = layout_members(&pb);
    let eb = Engine::new(pb.clone()).map_err(|e| e.join("; "))?;
    let cb = Ctx::new(&pb, &eb)?;
    let samples = sample_frames(&pb);
    let chans = eligible_channels(&cb);
    let an0 = analyze(&cb, &samples, &chans, &BTreeSet::new());
    let mut masks = BTreeSet::new();
    mask_targets(&pb.layers, &mut masks);
    let mut prev_declared = 0.0f64;
    let mut prev_channels: Option<Vec<Channel>> = None;
    let mut prev_paths: BTreeMap<(String, usize), (Vec<Anchor>, f64)> = BTreeMap::new();
    for k in 1..=o.levels {
        let tol = o.level_tolerance(k);
        let (geo_b, key_b, cull_b) = (tol * o.geometry_share, tol * o.keys_share, tol * o.cull_share);
        let mut notes: Vec<String> = Vec::new();
        if !dead.is_empty() {
            notes.push(format!("dropped {} dead layer(s): {}", dead.len(), dead.join(", ")));
        }
        // 1. keys
        let mut ko = reduce_keys(&cb, &an0, &chans, o.scale, key_b, &samples);
        if let Some(prev) = &prev_channels {
            // never keep more keys than the previous (finer) level: reuse its track when it is smaller
            let mut per_node: BTreeMap<usize, f64> = BTreeMap::new();
            let mut changed = false;
            for (ci, (chi, _)) in chans.iter().enumerate() {
                if let (Source::Keys { track: nt }, Source::Keys { track: pt }) = (&ko.channels[*chi].source, &prev[*chi].source) {
                    if pt.keys.len() < nt.keys.len() {
                        ko.channels[*chi].source = prev[*chi].source.clone();
                        changed = true;
                    }
                }
                let _ = ci;
            }
            if changed {
                for (ci, (chi, _)) in chans.iter().enumerate() {
                    if an0.blocked[ci] {
                        continue;
                    }
                    let (Source::Keys { track: orig }, Source::Keys { track: red }) = (&pb.channels[*chi].source, &ko.channels[*chi].source) else { continue };
                    let dev = track_deviation_at(orig, red, &merged_samples(orig, &samples)) * pb.channels[*chi].weight as f64;
                    for (&node, &v) in &an0.lever[ci] {
                        *per_node.entry(node).or_insert(0.0) += dev * v * o.scale;
                    }
                }
                ko.keys_px = per_node.values().copied().fold(0.0, f64::max) * MARGIN;
            }
        }
        prev_channels = Some(ko.channels.clone());
        let mut pk = pb.clone();
        pk.channels = ko.channels.clone();
        let ek = match Engine::new(pk.clone()) {
            Ok(e) => e,
            Err(e) => return Err(format!("internal: reduced keys do not validate: {}", e.join("; "))),
        };
        notes.extend(ko.notes.iter().cloned());
        let keyed = pb.channels.iter().filter(|c| matches!(c.source, Source::Keys { .. })).count();
        notes.push(format!("keys: {} of {} keyed channels have a documented lever, {} reduced", chans.len() - an0.blocked.iter().filter(|b| **b).count(), keyed, ko.reduced));
        // 2. geometry on the reduced animation
        let ck = Ctx::new(&pk, &ek)?;
        let mesh_nodes: BTreeSet<usize> = (0..ck.paths.len()).filter(|&i| mesh_eligible(&ck, i)).collect();
        let an1 = analyze(&ck, &samples, &[], &mesh_nodes);
        let (paths_v, complex_v, children_v, parent_v) = (ck.paths.clone(), ck.complex.clone(), ck.children.clone(), ck.parent.clone());
        drop(ck);
        let mut path_px = 0.0f64;
        let mut mesh_px = 0.0f64;
        let mut removed_anchors = 0usize;
        let mut collapses = 0usize;
        let mut new_paths: BTreeMap<(String, usize), (Vec<Anchor>, f64)> = BTreeMap::new();
        let mut skipped_layout: Vec<String> = Vec::new();
        for id in 0..paths_v.len() {
            let Some(l) = pk.find_layer(&paths_v[id]) else { continue };
            let LayerKind::Vector { paths } = &l.kind else { continue };
            if complex_v[id] {
                continue;
            }
            if is_layout_dependent(&layout_dep, &paths_v[id]) {
                if paths.iter().any(|path| path_eligible(l, path) && path.anchors.len() > 2) {
                    skipped_layout.push(paths_v[id].clone());
                }
                continue;
            }
            let s_used = an1.s_max[id] * MARGIN;
            if s_used.partial_cmp(&(1e-9)) != Some(std::cmp::Ordering::Greater) {
                continue;
            }
            let eps_local = geo_b / (o.scale * s_used);
            for (pi, path) in paths.iter().enumerate() {
                if !path_eligible(l, path) {
                    continue;
                }
                let key = (paths_v[id].clone(), pi);
                let mut sp = simplify_path_certified(&path.anchors, path.closed, eps_local);
                let mut bound_local = sp.bound;
                if let Some((pa, pb_)) = prev_paths.get(&key) {
                    if pa.len() < sp.anchors.len() && *pb_ <= eps_local {
                        sp.anchors = pa.clone();
                        bound_local = *pb_;
                    }
                }
                if sp.anchors.len() < path.anchors.len() {
                    removed_anchors += path.anchors.len() - sp.anchors.len();
                    path_px = path_px.max(bound_local * o.scale * s_used);
                    new_paths.insert(key, (sp.anchors, bound_local));
                }
            }
        }
        // apply path results
        for ((lp, pi), (anchors, _)) in &new_paths {
            if let Some(l) = pk.find_layer_mut(lp) {
                if let LayerKind::Vector { paths } = &mut l.kind {
                    paths[*pi].anchors = anchors.clone();
                }
            }
        }
        // 3. meshes
        for &id in &mesh_nodes {
            let Some(mt) = an1.mesh.get(&id) else { continue };
            let nv = mt.rest.len();
            if nv < 4 || mt.y.len() != samples.len() || mt.y.iter().any(|f| f.len() != nv) || mt.tris.is_empty() {
                continue;
            }
            let layer_path = paths_v[id].clone();
            let Some(l) = pk.find_layer(&layer_path) else { continue };
            let LayerKind::Image { mesh: Some(m), .. } = &l.kind else { continue };
            // attributes: weights of the (at most 16 most used) bones times the displacement scale
            let lever = mt
                .y
                .iter()
                .flat_map(|f| f.iter().zip(&mt.rest).map(|(y, r)| (y[0] as f64 - r[0]).hypot(y[1] as f64 - r[1])))
                .fold(0.0, f64::max)
                .max(1e-9);
            let mut bones: Vec<String> = m.weights.iter().flatten().map(|w| w.bone.clone()).collect();
            bones.sort();
            bones.dedup();
            bones.truncate(16);
            let attr: Vec<Vec<f64>> = (0..nv)
                .map(|v| {
                    let ws = m.weights.get(v);
                    let sum: f64 = ws.map_or(0.0, |w| w.iter().map(|x| x.weight as f64).sum());
                    bones
                        .iter()
                        .map(|b| match ws {
                            Some(w) if sum > 0.0 => w.iter().filter(|x| &x.bone == b).map(|x| x.weight as f64).sum::<f64>() / sum * lever,
                            _ => 0.0,
                        })
                        .collect()
                })
                .collect();
            let s: Vec<f64> = an1.s[id].iter().map(|v| v * o.scale * MARGIN).collect();
            let q = qem_simplify(&QemInput { rest: &mt.rest, tris: &mt.tris, attr: &attr, y: &mt.y, s: &s }, geo_b);
            if q.collapses == 0 {
                continue;
            }
            let mut remap = vec![usize::MAX; nv];
            for (ni, &oi) in q.keep.iter().enumerate() {
                remap[oi] = ni;
            }
            let Some(l) = pk.find_layer_mut(&layer_path) else { continue };
            let LayerKind::Image { mesh: Some(m), .. } = &mut l.kind else { continue };
            let old_uvs: Vec<Vec2> = m.free.as_ref().map(|f| f.uvs.clone()).unwrap_or_default();
            let weights: Vec<Vec<crate::path::BoneWeight>> = if m.weights.is_empty() { vec![] } else { q.keep.iter().map(|&i| m.weights[i].clone()).collect() };
            m.free = Some(crate::project::FreeMesh {
                vertices: q.keep.iter().map(|&i| Vec2::new(mt.rest[i][0] as f32, mt.rest[i][1] as f32)).collect(),
                triangles: q.tris.iter().map(|t| [remap[t[0]] as u32, remap[t[1]] as u32, remap[t[2]] as u32]).collect(),
                uvs: if old_uvs.len() == nv { q.keep.iter().map(|&i| old_uvs[i]).collect() } else { vec![] },
            });
            m.weights = weights;
            m.cols = 0;
            m.rows = 0;
            collapses += q.collapses;
            mesh_px = mesh_px.max(q.bound_px);
        }
        // 4. culling
        let mut cull_px = 0.0f64;
        let mut culled: Vec<String> = Vec::new();
        let mut skipped_ref: Vec<String> = Vec::new();
        for id in 0..paths_v.len() {
            let path = &paths_v[id];
            let Some(l) = pk.find_layer(path) else { continue };
            if !children_v[id].is_empty() || !is_boundary_layer_kind(&l.kind) || l.blend != crate::style::Blend::Normal || l.mask.is_some() || l.reference {
                continue;
            }
            if masks.contains(path) || !an1.has_geo[id] {
                continue;
            }
            if is_layout_dependent(&layout_dep, path) {
                continue; // removing it would change the bounds the layout is solved from
            }
            if let Some(pa) = parent_v[id] {
                if matches!(pk.find_layer(&paths_v[pa]).map(|x| &x.kind), Some(LayerKind::Switch { .. })) {
                    continue;
                }
            }
            let d = an1.diam_max[id] * o.scale * MARGIN;
            if d <= cull_b {
                let short = path.rsplit('/').next().unwrap_or(path);
                if referenced.contains(path.as_str()) || referenced.contains(short) {
                    // a container member (removal re-flows its siblings), scroll, link, glue, machine or binding names it
                    skipped_ref.push(path.clone());
                    continue;
                }
                culled.push(path.clone());
                cull_px = cull_px.max(d);
            }
        }
        for path in &culled {
            // skip a layer already removed with an ancestor (cannot happen: only leaves are culled)
            let _ = crate::scriptops::delete_layer(&mut pk, path);
        }
        if !skipped_layout.is_empty() {
            notes.push(format!("skipped: layout member - {} layer(s) left unreduced because a container or scroll lays them out by their bounds (simplifying them would re-flow their siblings): {}", skipped_layout.len(), skipped_layout.join(", ")));
        }
        if removed_anchors > 0 {
            notes.push(format!("removed {removed_anchors} path anchor(s)"));
        }
        if collapses > 0 {
            notes.push(format!("collapsed {collapses} mesh edge(s)"));
        }
        if !culled.is_empty() {
            notes.push(format!("culled {} layer(s): {}", culled.len(), culled.join(", ")));
        }
        if !skipped_ref.is_empty() {
            notes.push(format!("skipped: referenced - {} small layer(s) kept because a container, scroll, link, glue, machine or binding names them: {}", skipped_ref.len(), skipped_ref.join(", ")));
        }
        let mut rec = LodRecord {
            level: k,
            tolerance_px: tol,
            scale: o.scale,
            declared_px: cull_px.max(path_px.max(mesh_px) + ko.keys_px).max(prev_declared),
            bound_stage: 0.0,
            path_px,
            mesh_px,
            keys_px: ko.keys_px,
            cull_px,
            stats: LodStats::default(),
            notes,
        };
        match Engine::new(pk.clone()) {
            Ok(_) => prev_paths = new_paths,
            Err(e) => {
                // never fail the whole run: this level repeats the previous one, with the reason
                let prev = out.last().expect("level 0 exists");
                pk = prev.project.clone();
                rec.path_px = prev.record.path_px;
                rec.mesh_px = prev.record.mesh_px;
                rec.keys_px = prev.record.keys_px;
                rec.cull_px = prev.record.cull_px;
                rec.declared_px = prev.record.declared_px;
                rec.notes = vec![format!("level {k} did not validate ({}); it repeats level {} (no further reduction)", e.join("; "), k - 1)];
            }
        }
        prev_declared = rec.declared_px.max(prev_declared);
        out.push(finish(pk, rec));
    }
    Ok(out)
}

/// The coarsest level whose declared bound, scaled to `screen_scale` (pixels per stage unit), stays
/// within `max_px`. Level 0 (bound 0) is always acceptable.
pub fn pick_level(records: &[LodRecord], screen_scale: f64, max_px: f64) -> usize {
    let mut best = 0;
    if (screen_scale.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !matches!(max_px.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) {
        return 0;
    }
    for r in records {
        if r.level > 0 && r.bound_stage * screen_scale <= max_px && r.level > best {
            // a level can only be chosen if every finer level is also acceptable (bounds are non-decreasing)
            best = r.level;
        }
    }
    best
}

/// Output pixels per stage unit that fits the project into a `w x h` screen.
pub fn fit_scale(p: &Project, w: f64, h: f64) -> f64 {
    (w / p.width as f64).min(h / p.height as f64)
}
