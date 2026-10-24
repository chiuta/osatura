// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Static structure of a project (part of `lod`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Static structure of a project

/// Which animation channels can be reduced and what moves when they change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum LKind {
    BoneAngle(usize),
    BoneOff(usize),
    LayerX(usize),
    LayerY(usize),
    LayerRot(usize),
    CamXY,
}

/// A skinned rest point and its `(bone, weight)` pairs.
pub(super) type SkinPt = (P, Vec<(usize, f64)>);

pub(super) struct Ctx<'a> {
    pub(super) p: &'a Project,
    pub(super) e: &'a Engine,
    pub(super) paths: Vec<String>,
    pub(super) parent: Vec<Option<usize>>,
    pub(super) children: Vec<Vec<usize>>,
    pub(super) bone_parent: Vec<Option<usize>>,
    /// `sub[b][k]`: bone `k` is `b` or below it.
    pub(super) sub: Vec<Vec<bool>>,
    pub(super) rest_inv: Vec<A>,
    /// Layers that carry something the lever analysis cannot model (deformers, morphs, point offsets, wobble).
    pub(super) complex: Vec<bool>,
    /// Per node: skinned rest points `(rest position, normalised weights)`.
    pub(super) skin_pts: Vec<Vec<SkinPt>>,
}

pub(super) fn layer_by_path<'a>(p: &'a Project, path: &str) -> Option<&'a Layer> {
    p.find_layer(path)
}

impl<'a> Ctx<'a> {
    pub(super) fn new(p: &'a Project, e: &'a Engine) -> Result<Ctx<'a>, String> {
        let n = e.node_count();
        let paths: Vec<String> = {
            let mut v = vec![String::new(); n];
            for path in p.layer_paths() {
                if let Some(i) = e.node_index(&path) {
                    v[i] = path;
                }
            }
            v
        };
        let parent: Vec<Option<usize>> = (0..n).map(|i| e.node_parent(i)).collect();
        let mut children = vec![Vec::new(); n];
        for (i, pa) in parent.iter().enumerate() {
            if let Some(pa) = pa {
                children[*pa].push(i);
            }
        }
        let bi = |name: &str| p.bones.iter().position(|b| b.name == name);
        let bone_parent: Vec<Option<usize>> = p.bones.iter().map(|b| b.parent.as_deref().and_then(bi)).collect();
        let nb = p.bones.len();
        let mut sub = vec![vec![false; nb]; nb];
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for k in 0..nb {
            let mut cur = Some(k);
            while let Some(c) = cur {
                sub[c][k] = true;
                cur = bone_parent[c];
            }
        }
        let rest = p.rest_world()?;
        let rest_inv: Vec<A> = rest.iter().map(|m| a_inv(&a_of(m)).unwrap_or(A_ID)).collect();
        let sub_targets: BTreeSet<String> = p
            .channels
            .iter()
            .filter(|c| c.prop.target_kind() == crate::project::TargetKind::Sub)
            .map(|c| c.target.split("::").next().unwrap_or("").to_string())
            .collect();
        let mut complex = vec![false; n];
        let mut skin_pts: Vec<Vec<SkinPt>> = vec![Vec::new(); n];
        let norm_w = |ws: &[crate::path::BoneWeight]| -> Vec<(usize, f64)> {
            let mut out: Vec<(usize, f64)> = ws.iter().filter_map(|w| bi(&w.bone).map(|b| (b, w.weight as f64))).collect();
            let sum: f64 = out.iter().map(|x| x.1).sum();
            if sum > 0.0 {
                for x in &mut out {
                    x.1 /= sum;
                }
            } else {
                out.clear();
            }
            out
        };
        for i in 0..n {
            let Some(l) = layer_by_path(p, &paths[i]) else { continue };
            let mut c = !l.deformers.is_empty() || l.wobble.is_some() || sub_targets.contains(&paths[i]);
            match &l.kind {
                LayerKind::Vector { paths: ps } => {
                    for path in ps {
                        if !path.morphs.is_empty() {
                            c = true;
                        }
                        for a in &path.anchors {
                            let w = norm_w(&a.weights);
                            if w.is_empty() {
                                continue;
                            }
                            for pt in [a.pos, a.hin(), a.hout()] {
                                skin_pts[i].push((p_of(pt), w.clone()));
                            }
                        }
                    }
                }
                LayerKind::Image { mesh: Some(m), origin, size, .. } => {
                    // rest vertices need the displayed size: take it from the frame at sample time
                    // (stored lazily); here only the weights are recorded, positions filled by `fill_mesh_rest`.
                    let _ = (origin, size);
                    for w in &m.weights {
                        skin_pts[i].push(([f64::NAN, f64::NAN], norm_w(w)));
                    }
                }
                _ => {}
            }
            complex[i] = c;
        }
        Ok(Ctx { p, e, paths, parent, children, bone_parent, sub, rest_inv, complex, skin_pts })
    }

    pub(super) fn subtree(&self, id: usize) -> Vec<usize> {
        let mut out = vec![id];
        let mut i = 0;
        while i < out.len() {
            let c = out[i];
            out.extend(self.children[c].iter().copied());
            i += 1;
        }
        out
    }

    pub(super) fn layer(&self, id: usize) -> Option<&Layer> {
        layer_by_path(self.p, &self.paths[id])
    }
}

pub(super) fn touched_bones(p: &Project) -> BTreeSet<usize> {
    let bi = |name: &str| p.bones.iter().position(|b| b.name == name);
    let mut t = BTreeSet::new();
    for c in &p.ik {
        for b in c.bones.iter().chain(std::iter::once(&c.target)) {
            t.extend(bi(b));
        }
    }
    for c in &p.constraints {
        t.extend(bi(&c.bone));
        match &c.kind {
            crate::project::ConstraintKind::Aim { target, .. } | crate::project::ConstraintKind::Pin { target } => t.extend(bi(target)),
            crate::project::ConstraintKind::CopyRotation { source } => t.extend(bi(source)),
        }
    }
    for s in &p.springs {
        t.extend(bi(&s.bone));
    }
    t
}

/// True if geometry-affecting reductions are impossible for the whole project.
pub fn links_present(p: &Project) -> bool {
    !p.mesh_locks.is_empty() || !p.glues.is_empty()
}

/// Channels that may be reduced, with what they drive. Everything else is left alone.
pub(super) fn eligible_channels(c: &Ctx) -> Vec<(usize, LKind)> {
    let p = c.p;
    if links_present(p) {
        return vec![];
    }
    let touched = touched_bones(p);
    let blocked_text: String = {
        let mut s = String::new();
        for ch in &p.channels {
            match &ch.source {
                Source::Expr { expr } => {
                    s.push_str(expr);
                    s.push('\n');
                }
                Source::Driven { input, .. } => {
                    s.push_str(&format!("\"{input}\"\n"));
                }
                _ => {}
            }
        }
        s.push_str(&serde_json::to_string(&p.machines).unwrap_or_default());
        s.push_str(&serde_json::to_string(&p.bindings).unwrap_or_default());
        s.push_str(&serde_json::to_string(&p.scrolls).unwrap_or_default());
        s
    };
    let referenced = |name: &str| blocked_text.contains(&format!("\"{name}\""));
    let mut out = Vec::new();
    for (i, ch) in p.channels.iter().enumerate() {
        let Source::Keys { track } = &ch.source else { continue };
        if track.keys.len() < 3 || ch.mode == Mode::Multiply || (ch.weight.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !ch.weight.is_finite() || (!ch.target.is_empty() && referenced(&ch.target)) {
            continue;
        }
        use Prop::*;
        let kind = match ch.prop {
            BoneAngle | BoneOffsetX | BoneOffsetY => {
                let Some(b) = p.bones.iter().position(|b| b.name == ch.target) else { continue };
                if touched.contains(&b) || (0..p.bones.len()).any(|k| c.sub[b][k] && touched.contains(&k)) {
                    continue;
                }
                if ch.prop == BoneAngle {
                    LKind::BoneAngle(b)
                } else {
                    LKind::BoneOff(b)
                }
            }
            LayerX | LayerY | LayerRotation => {
                let Some(id) = c.e.node_index(&ch.target) else { continue };
                match ch.prop {
                    LayerX => LKind::LayerX(id),
                    LayerY => LKind::LayerY(id),
                    _ => LKind::LayerRot(id),
                }
            }
            CameraX | CameraY => LKind::CamXY,
            _ => continue,
        };
        out.push((i, kind));
    }
    out
}
