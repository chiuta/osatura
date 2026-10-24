// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Skeleton abstraction shared by BVH and rig, slot assignment (part of `mocap`; see the module docs in `mod.rs`).
use super::*;

// Skeleton abstraction shared by BVH and rig, slot assignment

pub(super) struct Skel {
    pub(super) names: Vec<String>,
    pub(super) parent: Vec<Option<usize>>,
    pub(super) children: Vec<Vec<usize>>,
    /// Rest positions, y up.
    pub(super) pos: Vec<V3>,
    pub(super) depth: Vec<usize>,
    /// Lateral axis (0 = x, 2 = z) and the sign that points to the character's left.
    pub(super) lat_axis: usize,
    pub(super) lat_sign: f64,
    pub(super) lat_fixed: bool,
}

impl Skel {
    pub(super) fn new(names: Vec<String>, parent: Vec<Option<usize>>, pos: Vec<V3>, lat_fixed: bool) -> Skel {
        let n = names.len();
        let mut children = vec![vec![]; n];
        let mut depth = vec![0; n];
        for i in 0..n {
            if let Some(p) = parent[i] {
                children[p].push(i);
                depth[i] = depth[p] + 1;
            }
        }
        Skel { names, parent, children, pos, depth, lat_axis: 0, lat_sign: 1.0, lat_fixed }
    }
    pub(super) fn from_bvh(b: &Bvh) -> Skel {
        let pose = b.pose(None);
        Skel::new(b.joints.iter().map(|j| j.name.clone()).collect(), b.joints.iter().map(|j| j.parent).collect(), pose.pos, false)
    }
    pub(super) fn from_rig(p: &Project) -> Result<Skel, String> {
        let w = crate::rigedit::rest_world(p)?;
        let idx: BTreeMap<&str, usize> = p.bones.iter().enumerate().map(|(i, b)| (b.name.as_str(), i)).collect();
        let parent = p.bones.iter().map(|b| b.parent.as_deref().and_then(|n| idx.get(n).copied())).collect();
        Ok(Skel::new(p.bones.iter().map(|b| b.name.clone()).collect(), parent, w.iter().map(|(h, _)| [h.x as f64, -(h.y as f64), 0.0]).collect(), true))
    }
    pub(super) fn lat(&self, j: usize) -> f64 {
        self.pos[j][self.lat_axis] * self.lat_sign
    }
    /// Lateral coordinate of the subtree point farthest sideways from `origin` (signed).
    pub(super) fn lat_ext(&self, c: usize, origin: usize) -> f64 {
        let o = self.lat(origin);
        self.subtree(c).iter().map(|&q| self.lat(q)).fold(o, |best, v| if (v - o).abs() > (best - o).abs() { v } else { best })
    }
    pub(super) fn subtree(&self, j: usize) -> Vec<usize> {
        let mut out = vec![j];
        let mut i = 0;
        while i < out.len() {
            let c = out[i];
            out.extend(self.children[c].iter().copied());
            i += 1;
        }
        out
    }
    /// Single-child descent from `j`, including `j`; stops at a leaf or a branching joint (included).
    pub(super) fn chain(&self, j: usize) -> Vec<usize> {
        let mut out = vec![j];
        let mut c = j;
        while self.children[c].len() == 1 {
            c = self.children[c][0];
            out.push(c);
        }
        out
    }
    pub(super) fn seg(&self, chain: &[usize], i: usize) -> f64 {
        if i + 1 < chain.len() {
            dist3(self.pos[chain[i]], self.pos[chain[i + 1]])
        } else {
            0.0
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Via {
    Name,
    Topology,
    Manual,
}

impl Via {
    pub(super) fn label(self) -> &'static str {
        match self {
            Via::Name => "name",
            Via::Topology => "topology",
            Via::Manual => "manual",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Assign {
    pub(super) joint: usize,
    pub(super) conf: f32,
    pub(super) via: Via,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Slots {
    pub(super) single: BTreeMap<String, Assign>,
    pub(super) spine: Vec<Assign>,
}

pub(super) fn slot_key(role: Role, side: Option<Side>) -> Option<String> {
    let nm = match role {
        Role::Hips => return Some("hips".into()),
        Role::Neck => return Some("neck".into()),
        Role::Head => return Some("head".into()),
        Role::Clavicle => "clavicle",
        Role::UpperArm => "upperarm",
        Role::ForeArm => "forearm",
        Role::Hand => "hand",
        Role::Thigh => "thigh",
        Role::Shin => "shin",
        Role::Foot => "foot",
        Role::Toe => "toe",
        _ => return None,
    };
    Some(format!("{nm}.{}", if side? == Side::Left { "l" } else { "r" }))
}

pub(super) fn name_slots(s: &Skel) -> Slots {
    let mut out = Slots::default();
    let mut spine: Vec<Assign> = Vec::new();
    let mut tags: Vec<Tag> = s.names.iter().map(|n| classify(n)).collect();
    // context: a bare `Arm` that has a forearm child, or a bare `Leg` under a thigh, is certain
    let base = tags.clone();
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for j in 0..tags.len() {
        let child_is = |r: Role| s.children[j].iter().any(|&c| base[c].role == r);
        let parent_is = |r: Role| s.parent[j].is_some_and(|q| base[q].role == r);
        if tags[j].role == Role::UpperArm && tags[j].conf < 0.9 && child_is(Role::ForeArm) {
            tags[j].conf = 0.9;
        }
        if tags[j].role == Role::Shin && tags[j].conf < 0.9 && parent_is(Role::Thigh) {
            tags[j].conf = 0.9;
        }
    }
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for j in 0..s.names.len() {
        let tag = tags[j];
        if tag.conf <= 0.0 {
            continue;
        }
        if tag.role == Role::Spine {
            spine.push(Assign { joint: j, conf: tag.conf, via: Via::Name });
            continue;
        }
        let Some(key) = slot_key(tag.role, tag.side) else { continue };
        let a = Assign { joint: j, conf: tag.conf, via: Via::Name };
        match out.single.get(&key) {
            Some(old) if (old.conf, std::cmp::Reverse(s.depth[old.joint])) >= (a.conf, std::cmp::Reverse(s.depth[a.joint])) => {}
            _ => {
                out.single.insert(key, a);
            }
        }
    }
    spine.sort_by_key(|a| (s.depth[a.joint], a.joint));
    out.spine = spine;
    out
}

pub(super) fn topo_slots(s: &mut Skel, hips_hint: Option<usize>) -> Slots {
    let mut out = Slots::default();
    let n = s.names.len();
    if n == 0 {
        return out;
    }
    // hips: hint, else the first branching joint below the root
    let hips = match hips_hint {
        Some(h) => h,
        None => {
            let roots: Vec<usize> = (0..n).filter(|&i| s.parent[i].is_none()).collect();
            let Some(&r) = roots.iter().max_by_key(|&&r| (s.subtree(r).len(), std::cmp::Reverse(r))) else { return out };
            let mut c = r;
            while s.children[c].len() == 1 {
                c = s.children[c][0];
            }
            if s.children[c].len() < 2 {
                return out; // a single chain has no pelvis to find
            }
            c
        }
    };
    let tc = 0.6f32;
    let ts = 0.55f32;
    out.single.insert("hips".into(), Assign { joint: hips, conf: tc, via: Via::Topology });
    let hy = s.pos[hips][1];
    let mut legs: Vec<(usize, f64)> = Vec::new();
    let mut up: Vec<(usize, f64)> = Vec::new();
    for &c in &s.children[hips] {
        let st = s.subtree(c);
        let down = hy - st.iter().map(|&j| s.pos[j][1]).fold(f64::MAX, f64::min);
        let upv = st.iter().map(|&j| s.pos[j][1]).fold(f64::MIN, f64::max) - hy;
        if down > upv {
            legs.push((c, down));
        } else {
            up.push((c, upv));
        }
    }
    legs.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    up.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    // lateral axis of this skeleton from the leg separation
    let total_h = {
        let (lo, hi) = s.pos.iter().fold((f64::MAX, f64::MIN), |(l, h), p| (l.min(p[1]), h.max(p[1])));
        (hi - lo).max(1e-9)
    };
    let mut sided = false;
    if legs.len() >= 2 {
        let (a, b) = (legs[0].0, legs[1].0);
        if !s.lat_fixed {
            let ext = |c: usize, ax: usize| s.subtree(c).iter().map(|&q| s.pos[q][ax] - s.pos[hips][ax]).fold(0.0f64, |m, v| if v.abs() > m.abs() { v } else { m });
            let dx = (ext(a, 0) - ext(b, 0)).abs();
            let dz = (ext(a, 2) - ext(b, 2)).abs();
            if dz > dx {
                s.lat_axis = 2;
                s.lat_sign = -1.0; // facing +x: left is -z
            }
        }
        let (la, lb) = (s.lat_ext(a, hips), s.lat_ext(b, hips));
        sided = (la - lb).abs() > 0.01 * total_h;
        if sided {
            let (l, r) = if la > lb { (a, b) } else { (b, a) };
            for (c, side) in [(l, "l"), (r, "r")] {
                let ch = s.chain(c);
                for (k, role) in ["thigh", "shin", "foot", "toe"].iter().enumerate() {
                    if let Some(&j) = ch.get(k) {
                        out.single.insert(format!("{role}.{side}"), Assign { joint: j, conf: ts, via: Via::Topology });
                    }
                }
            }
        }
    }
    if let Some(&(sp, _)) = up.first() {
        // spine up to the joint where neck and arms branch
        let mut chain = vec![sp];
        let mut c = sp;
        while s.children[c].len() == 1 {
            c = s.children[c][0];
            chain.push(c);
        }
        let top = *chain.last().unwrap();
        let kids = s.children[top].clone();
        if kids.len() >= 3 {
            let spread = |j: usize| s.subtree(j).iter().map(|&q| (s.lat(q) - s.lat(top)).abs()).fold(0.0, f64::max);
            let mut ranked: Vec<(usize, f64)> = kids.iter().map(|&k| (k, spread(k))).collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let arms = [ranked[0].0, ranked[1].0];
            let neck_c = ranked[2..].iter().max_by(|a, b| s.pos[a.0][1].total_cmp(&s.pos[b.0][1]).then(b.0.cmp(&a.0))).map(|x| x.0);
            out.spine = chain.iter().map(|&j| Assign { joint: j, conf: ts, via: Via::Topology }).collect();
            if let Some(nc) = neck_c {
                let nch = s.chain(nc);
                if nch.len() >= 2 {
                    out.single.insert("neck".into(), Assign { joint: nch[0], conf: ts, via: Via::Topology });
                    out.single.insert("head".into(), Assign { joint: nch[1], conf: ts, via: Via::Topology });
                } else {
                    out.single.insert("head".into(), Assign { joint: nch[0], conf: ts, via: Via::Topology });
                }
            }
            let (l0, l1) = (s.lat_ext(arms[0], top), s.lat_ext(arms[1], top));
            if (l0 - l1).abs() > 0.01 * total_h {
                let (l, r) = if l0 > l1 { (arms[0], arms[1]) } else { (arms[1], arms[0]) };
                for (c, side) in [(l, "l"), (r, "r")] {
                    let ch = s.chain(c);
                    let shift = usize::from(ch.len() >= 4 && s.seg(&ch, 0) < 0.5 * s.seg(&ch, 1));
                    if shift == 1 {
                        out.single.insert(format!("clavicle.{side}"), Assign { joint: ch[0], conf: ts, via: Via::Topology });
                    }
                    for (k, role) in ["upperarm", "forearm", "hand"].iter().enumerate() {
                        if let Some(&j) = ch.get(k + shift) {
                            out.single.insert(format!("{role}.{side}"), Assign { joint: j, conf: ts, via: Via::Topology });
                        }
                    }
                }
            }
        } else {
            out.spine = chain.iter().map(|&j| Assign { joint: j, conf: ts, via: Via::Topology }).collect();
        }
    }
    let _ = sided;
    out
}

pub(super) fn merge_slots(name: Slots, topo: Slots) -> Slots {
    let mut out = name;
    let mut used: BTreeSet<usize> = out.single.values().map(|a| a.joint).chain(out.spine.iter().map(|a| a.joint)).collect();
    for (k, a) in topo.single {
        if !out.single.contains_key(&k) && !used.contains(&a.joint) {
            used.insert(a.joint);
            out.single.insert(k, a);
        }
    }
    if out.spine.is_empty() {
        out.spine = topo.spine.into_iter().filter(|a| !used.contains(&a.joint)).collect();
    }
    out
}
