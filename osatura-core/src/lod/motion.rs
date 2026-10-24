// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Motion analysis: scale factors, extents, levers, mesh trajectories (part of `lod`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Motion analysis: scale factors, extents, levers, mesh trajectories

/// Frames at which motion is sampled: `SUB` per frame plus every (fractional) key frame in range.
pub fn sample_frames(p: &Project) -> Vec<f32> {
    let n = p.frames.max(1) as usize;
    let mut v: Vec<f32> = (0..n * SUB).map(|i| i as f32 / SUB as f32).collect();
    for c in &p.channels {
        if let Source::Keys { track } = &c.source {
            for k in &track.keys {
                if k.frame >= 0.0 && k.frame < n as f32 {
                    v.push(k.frame);
                }
            }
        }
    }
    v.sort_by(|a, b| a.total_cmp(b));
    v.dedup();
    v
}

pub(super) struct MeshTraj {
    /// `y[sample][vertex]`: deformed vertex positions in layer space.
    pub(super) y: Vec<Vec<[f32; 2]>>,
    pub(super) rest: Vec<P>,
    pub(super) tris: Vec<[usize; 3]>,
}

#[derive(Default)]
pub(super) struct Analysis {
    /// `s[node][sample]`: largest singular value of the node's world matrix (screen units per layer unit).
    pub(super) s: Vec<Vec<f64>>,
    pub(super) s_max: Vec<f64>,
    /// Largest screen diagonal (screen units) of the node's own geometry over all samples.
    pub(super) diam_max: Vec<f64>,
    pub(super) has_geo: Vec<bool>,
    pub(super) mesh: BTreeMap<usize, MeshTraj>,
    /// Per eligible channel: screen units per value unit, per node (max over samples).
    pub(super) lever: Vec<BTreeMap<usize, f64>>,
    /// Channels whose lever could not be bounded.
    pub(super) blocked: Vec<bool>,
}

pub(super) fn node_points(nd: &crate::engine::RenderNode, w: &A) -> Vec<P> {
    let mut v = Vec::new();
    for path in &nd.paths {
        for a in &path.anchors {
            v.push(a_apply(w, p_of(a.pos)));
            v.push(a_apply(w, p_of(a.hin())));
            v.push(a_apply(w, p_of(a.hout())));
        }
    }
    if let Some(img) = &nd.image {
        if let Some(m) = &img.mesh {
            for q in &m.positions {
                v.push(a_apply(w, p_of(*q)));
            }
        } else {
            let (o, s) = (p_of(img.origin), p_of(img.size));
            for c in [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
                v.push(a_apply(w, [o[0] + s[0] * c[0], o[1] + s[1] * c[1]]));
            }
        }
    }
    v
}

pub(super) fn analyze(c: &Ctx, samples: &[f32], chans: &[(usize, LKind)], mesh_nodes: &BTreeSet<usize>) -> Analysis {
    let n = c.paths.len();
    let p = c.p;
    let mut an = Analysis {
        s: vec![vec![0.0; samples.len()]; n],
        s_max: vec![0.0; n],
        diam_max: vec![0.0; n],
        has_geo: vec![false; n],
        lever: vec![BTreeMap::new(); chans.len()],
        blocked: vec![false; chans.len()],
        ..Default::default()
    };
    for &id in mesh_nodes {
        an.mesh.insert(id, MeshTraj { y: Vec::with_capacity(samples.len()), rest: Vec::new(), tris: Vec::new() });
    }
    let subtrees: Vec<Vec<usize>> = (0..n).map(|i| c.subtree(i)).collect();
    let need_levers = !chans.is_empty();
    let pi180 = std::f64::consts::PI / 180.0;
    let mut skin_rest: Vec<Option<Vec<P>>> = vec![None; n];
    for (si, &t) in samples.iter().enumerate() {
        let fr = c.e.evaluate(t);
        let worlds: Vec<A> = fr.nodes.iter().map(|nd| a_of(&nd.world)).collect();
        let pts: Vec<Vec<P>> = fr.nodes.iter().enumerate().map(|(id, nd)| node_points(nd, &worlds[id])).collect();
        for id in 0..n {
            let s = lin_norm(&worlds[id]);
            an.s[id][si] = s;
            an.s_max[id] = an.s_max[id].max(s);
            let nd = &fr.nodes[id];
            if !pts[id].is_empty() {
                an.has_geo[id] = true;
                let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
                for q in &pts[id] {
                    x0 = x0.min(q[0]);
                    y0 = y0.min(q[1]);
                    x1 = x1.max(q[0]);
                    y1 = y1.max(q[1]);
                }
                let hw = nd.paths.iter().filter_map(|pp| pp.stroke.as_ref().map(|s| s.width as f64 * 0.5)).fold(0.0, f64::max) * s;
                let d = ((x1 - x0) + 2.0 * hw).hypot((y1 - y0) + 2.0 * hw);
                if d > an.diam_max[id] {
                    an.diam_max[id] = d;
                }
            }
            if let Some(mt) = an.mesh.get_mut(&id) {
                if let Some(m) = nd.image.as_ref().and_then(|i| i.mesh.as_ref()) {
                    if si == 0 {
                        mt.tris = m.tris.iter().map(|t| [t[0] as usize, t[1] as usize, t[2] as usize]).collect();
                        if let (Some(l), Some(img)) = (c.layer(id), nd.image.as_ref()) {
                            if let LayerKind::Image { mesh: Some(ms), origin, .. } = &l.kind {
                                mt.rest = ms.rest_vertices(*origin, img.size).iter().map(|v| p_of(*v)).collect();
                            }
                        }
                    }
                    mt.y.push(m.positions.iter().map(|q| [q.x, q.y]).collect());
                }
            }
        }
        if !need_levers {
            continue;
        }
        // rest points of skinned image meshes (need the displayed size from the frame)
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for id in 0..n {
            if skin_rest[id].is_none() {
                if let (Some(l), Some(img)) = (c.layer(id), fr.nodes[id].image.as_ref()) {
                    if let LayerKind::Image { mesh: Some(m), origin, .. } = &l.kind {
                        skin_rest[id] = Some(m.rest_vertices(*origin, img.size).iter().map(|v| p_of(*v)).collect());
                    }
                }
            }
        }
        let skin: Vec<A> = fr.world.iter().zip(&c.rest_inv).map(|(w, ri)| a_mul(&a_of(w), ri)).collect();
        let mut locals: Vec<Option<A>> = vec![None; n];
        let local_of = |id: usize, locals: &mut Vec<Option<A>>| -> A {
            if locals[id].is_none() {
                locals[id] = Some(local_matrix(&c.e.layer_local_at(t, id)));
            }
            locals[id].unwrap()
        };
        for (ci, (_, kind)) in chans.iter().enumerate() {
            let upd = |lever: &mut BTreeMap<usize, f64>, id: usize, v: f64| {
                let e = lever.entry(id).or_insert(0.0);
                if v > *e {
                    *e = v;
                }
            };
            match *kind {
                LKind::CamXY => {
                    let z = lin_norm(&a_of(&fr.camera));
                    for id in 0..n {
                        let par = p.layers.get(c.e.node_idx_path(id)[0]).map_or(1.0, |l| l.parallax as f64);
                        let ze = (1.0 + (z - 1.0) * par).max(0.01);
                        upd(&mut an.lever[ci], id, par.abs() * ze);
                    }
                }
                LKind::LayerX(nid) | LKind::LayerY(nid) | LKind::LayerRot(nid) => {
                    let loc = local_of(nid, &mut locals);
                    let Some(li) = a_inv(&loc) else {
                        an.blocked[ci] = true;
                        continue;
                    };
                    let pn = a_mul(&worlds[nid], &li);
                    let Some(pinv) = a_inv(&pn) else {
                        an.blocked[ci] = true;
                        continue;
                    };
                    let ln = lin_norm(&pn);
                    let lt = c.e.layer_local_at(t, nid);
                    let posv = [lt.position.x as f64, lt.position.y as f64];
                    for &m in &subtrees[nid] {
                        let v = match *kind {
                            LKind::LayerRot(_) => {
                                let r = pts[m].iter().map(|q| dist(a_apply(&pinv, *q), posv)).fold(0.0, f64::max);
                                ln * pi180 * r
                            }
                            _ => ln,
                        };
                        upd(&mut an.lever[ci], m, v);
                    }
                }
                LKind::BoneAngle(b) | LKind::BoneOff(b) => {
                    let is_angle = matches!(kind, LKind::BoneAngle(_));
                    let ob = a_apply(&a_of(&fr.world[b]), [0.0, 0.0]);
                    let parent_lin = c.bone_parent[b].map_or(1.0, |pb| lin_norm(&a_of(&fr.world[pb])));
                    for id in 0..n {
                        // skinned geometry
                        let mut skin_part = 0.0f64;
                        let sp = &c.skin_pts[id];
                        if !sp.is_empty() {
                            let rest_mesh = skin_rest[id].as_ref();
                            let mut lev = 0.0f64;
                            let mut hit = false;
                            for (k, (rp, ws)) in sp.iter().enumerate() {
                                let wsub: f64 = ws.iter().filter(|(bk, _)| c.sub[b][*bk]).map(|x| x.1).sum();
                                if wsub <= 0.0 {
                                    continue;
                                }
                                hit = true;
                                if c.complex[id] {
                                    an.blocked[ci] = true;
                                    continue;
                                }
                                let rp = if rp[0].is_nan() { rest_mesh.and_then(|r| r.get(k).copied()).unwrap_or([0.0, 0.0]) } else { *rp };
                                let v = if is_angle {
                                    ws.iter().filter(|(bk, _)| c.sub[b][*bk]).map(|(bk, w)| w * dist(a_apply(&skin[*bk], rp), ob)).sum::<f64>() * pi180
                                } else {
                                    wsub * parent_lin
                                };
                                lev = lev.max(v);
                            }
                            if hit && lev > 0.0 {
                                skin_part = lev * an.s[id][si];
                            }
                        }
                        // riding layers: every ancestor-or-self that rides a bone at or below b
                        let mut cur = Some(id);
                        let mut total = 0.0f64;
                        while let Some(nn) = cur {
                            if let Some(bn) = c.layer(nn).and_then(|l| l.bone.as_deref()) {
                                if let Some(k) = p.bones.iter().position(|bb| bb.name == bn) {
                                    if c.sub[b][k] {
                                        let loc = local_of(nn, &mut locals);
                                        let att = skin[k];
                                        let q = a_inv(&loc).and_then(|li| a_inv(&att).map(|ai| a_mul(&a_mul(&worlds[nn], &li), &ai)));
                                        match q.and_then(|q| a_inv(&q).map(|qi| (q, qi))) {
                                            Some((q, qi)) => {
                                                let lq = lin_norm(&q);
                                                total += if is_angle {
                                                    lq * pi180 * pts[id].iter().map(|s| dist(a_apply(&qi, *s), ob)).fold(0.0, f64::max)
                                                } else {
                                                    lq * parent_lin
                                                };
                                            }
                                            None => an.blocked[ci] = true,
                                        }
                                    }
                                }
                            }
                            cur = c.parent[nn];
                        }
                        if total + skin_part > 0.0 {
                            upd(&mut an.lever[ci], id, total + skin_part);
                        }
                    }
                }
            }
        }
    }
    an
}
