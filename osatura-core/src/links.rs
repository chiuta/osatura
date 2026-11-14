// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Persistent vertex constraints between layers: **mesh locks** and **glue** (R-063).
//!
//! Both are stored in the project (`Project::mesh_locks`, `Project::glues`, serde defaults, old
//! files load unchanged) and are evaluated by the engine on every frame, after all layers are
//! posed and deformed, in a deterministic order. They work on *geometry units*: one path of a
//! vector layer (its anchors) or the mesh of an image layer (grid or free mesh). All maths is done
//! in stage space (layer transform, parents and bone attachment included), so layers with different
//! transforms deform together correctly. Layers under different root layers must share the same
//! `parallax` (stage space ignores the camera).
//!
//! # Mesh lock (displacement transfer)
//! Follower vertex `f` is bound at rest to the host surface: the triangle of the host image mesh
//! that contains it (barycentric weights; if it lies outside the mesh, the nearest point of the
//! nearest triangle), or, for a vector host path, the nearest point of its polyline (two weights).
//! With bound host vertices `h_k`, weights `w_k` and stage positions `h_k^rest`, `h_k^now`:
//! `f^now = f^rest + sum_k w_k (h_k^now - h_k^rest)`. A follower at rest is therefore untouched,
//! a rigid translation of the host moves it exactly, and the offset to the surface is kept as a
//! displacement (not rotated). `strength` (0..1) blends between the follower's own pose (0) and
//! the lock (1). Handles of a locked anchor are translated with it.
//!
//! # Glue (vertex pairs)
//! A glue lists pairs `(a, b)`: `a` drives, `b` follows. Modes:
//! * `snap`   : `b` coincides with `a`.
//! * `rigid`  : `b = a + M (b_rest - a_rest)` where `M` is the change of `a`'s layer frame since
//!   rest (rotation, scale, shear of the layer including its bone attachment). The link behaves
//!   like a rod fixed to the layer of `a`.
//! * `offset` : `b = a + (b_rest - a_rest)`, a constant stage-space offset.
//!
//! # Order and cycles
//! Every constraint is an edge `driver unit -> follower unit`. Cycles (also `A -> A`) are rejected
//! when the project is compiled, a vertex may be driven by one constraint only, and constraints
//! are evaluated in the topological order of their follower units (declaration order inside).
//! Rest positions use the static (un-animated) layer transforms and the rest pose of the bones.
//!
//! Method: barycentric coordinates on triangles (textbook), closest point on a triangle/segment
//! (textbook), Kahn's topological sort. Nothing here is taken from another product.

use crate::engine::{Engine, RenderNode};
use crate::math::{Affine2, Vec2};
use crate::project::Project;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

fn one_f() -> f32 {
    1.0
}

/// One geometry unit: a layer (`/`-joined path) and, for vector layers, a path index.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeomRef {
    pub layer: String,
    #[serde(default)]
    pub path: usize,
}

/// A mesh lock: `follower` vertices follow the surface of `host`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshLock {
    pub name: String,
    pub host: GeomRef,
    pub follower: GeomRef,
    /// Follower vertex indices; empty = every vertex.
    #[serde(default)]
    pub vertices: Vec<usize>,
    /// 0 = follower keeps its own pose, 1 = fully locked.
    #[serde(default = "one_f")]
    pub strength: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlueMode {
    Snap,
    #[default]
    Rigid,
    Offset,
}

impl GlueMode {
    pub fn parse(s: &str) -> Option<GlueMode> {
        match s {
            "snap" => Some(GlueMode::Snap),
            "rigid" => Some(GlueMode::Rigid),
            "offset" => Some(GlueMode::Offset),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VertexRef {
    pub layer: String,
    #[serde(default)]
    pub path: usize,
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GluePair {
    /// Driver vertex.
    pub a: VertexRef,
    /// Follower vertex.
    pub b: VertexRef,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Glue {
    pub name: String,
    #[serde(default)]
    pub mode: GlueMode,
    pub pairs: Vec<GluePair>,
    #[serde(default = "one_f")]
    pub strength: f32,
}

/// Rest-pose geometry of one layer, supplied by the engine.
pub struct UnitRest {
    /// Layer space -> stage at rest (static transforms of the layer and its parents).
    pub stage: Affine2,
    /// Per path (image layers: one entry): vertex positions in layer space.
    pub paths: Vec<Vec<Vec2>>,
    pub closed: Vec<bool>,
    /// Triangles of an image mesh.
    pub tris: Vec<[usize; 3]>,
    pub is_image: bool,
    pub has_mesh: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Unit {
    node: usize,
    path: usize,
}

#[derive(Clone, Debug)]
struct Bind {
    v: [usize; 3],
    w: [f32; 3],
}

#[derive(Clone, Debug)]
struct RLock {
    f: Unit,
    h: Unit,
    idx: Vec<usize>,
    f_rest: Vec<Vec2>,
    h_rest: Vec<Vec2>,
    bind: Vec<Bind>,
    strength: f32,
}

#[derive(Clone, Debug)]
struct RGlue {
    a: Unit,
    b: Unit,
    pairs: Vec<(usize, usize)>,
    a_rest: Vec<Vec2>,
    b_rest: Vec<Vec2>,
    a_frame_rest: Affine2,
    mode: GlueMode,
    strength: f32,
}

#[derive(Clone, Debug)]
enum Item {
    Lock(RLock),
    Glue(RGlue),
}

/// Compiled constraints in evaluation order.
#[derive(Clone, Debug)]
pub struct Resolved {
    items: Vec<Item>,
}

// ---- small geometry helpers (f64 internally) ----

fn sub(a: Vec2, b: Vec2) -> [f64; 2] {
    [a.x as f64 - b.x as f64, a.y as f64 - b.y as f64]
}

/// Closest point of segment `ab` to `p`: parameter `t` in [0, 1] and squared distance.
fn closest_on_segment(p: Vec2, a: Vec2, b: Vec2) -> (f32, f64) {
    let ab = sub(b, a);
    let ap = sub(p, a);
    let l2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if l2 <= 0.0 { 0.0 } else { ((ap[0] * ab[0] + ap[1] * ab[1]) / l2).clamp(0.0, 1.0) };
    let d = [ap[0] - t * ab[0], ap[1] - t * ab[1]];
    (t as f32, d[0] * d[0] + d[1] * d[1])
}

/// Barycentric weights of the point of triangle `abc` closest to `p`, and the squared distance
/// (0 when `p` is inside).
fn closest_on_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> ([f32; 3], f64) {
    let (v0, v1, v2) = (sub(b, a), sub(c, a), sub(p, a));
    let den = v0[0] * v1[1] - v1[0] * v0[1];
    if den.abs() > 1e-12 {
        let u = (v2[0] * v1[1] - v1[0] * v2[1]) / den;
        let v = (v0[0] * v2[1] - v2[0] * v0[1]) / den;
        if u >= 0.0 && v >= 0.0 && u + v <= 1.0 {
            return ([(1.0 - u - v) as f32, u as f32, v as f32], 0.0);
        }
    }
    let mut best = ([1.0, 0.0, 0.0], f64::MAX);
    for (i, (p0, p1)) in [(a, b), (b, c), (c, a)].into_iter().enumerate() {
        let (t, d2) = closest_on_segment(p, p0, p1);
        if d2 < best.1 {
            let mut w = [0.0; 3];
            w[i] = 1.0 - t;
            w[(i + 1) % 3] = t;
            best = (w, d2);
        }
    }
    best
}

fn vec_ok(v: Vec2) -> bool {
    v.x.is_finite() && v.y.is_finite()
}

/// Compile the constraints of an engine's project. `Ok(None)` when there are none.
pub(crate) fn resolve(e: &Engine) -> Result<Option<Resolved>, Vec<String>> {
    let p = &e.project;
    if p.mesh_locks.is_empty() && p.glues.is_empty() {
        return Ok(None);
    }
    let mut errs: Vec<String> = Vec::new();
    let mut rests: HashMap<usize, UnitRest> = HashMap::new();

    // fetch a unit and validate that the path exists and (for images) has a mesh
    let unit = |what: &str, g: &GeomRef, errs: &mut Vec<String>, rests: &mut HashMap<usize, UnitRest>| -> Option<Unit> {
        let Some(node) = e.node_index(&g.layer) else {
            errs.push(format!("{what}: no layer '{}'", g.layer));
            return None;
        };
        if let std::collections::hash_map::Entry::Vacant(slot) = rests.entry(node) {
            match e.link_unit(node) {
                Some(r) => {
                    slot.insert(r);
                }
                None => {
                    errs.push(format!("{what}: layer '{}' is neither a vector layer nor an image layer", g.layer));
                    return None;
                }
            }
        }
        let r = &rests[&node];
        if r.is_image && !r.has_mesh {
            errs.push(format!("{what}: image layer '{}' has no mesh (run mesh/auto-skin first)", g.layer));
            return None;
        }
        if g.path >= r.paths.len() {
            errs.push(format!("{what}: layer '{}' has no path {}", g.layer, g.path));
            return None;
        }
        if r.is_image && g.path != 0 {
            errs.push(format!("{what}: image layer '{}' has only path 0", g.layer));
            return None;
        }
        if r.paths[g.path].iter().any(|v| !vec_ok(*v)) {
            errs.push(format!("{what}: layer '{}' has non-finite vertices", g.layer));
            return None;
        }
        Some(Unit { node, path: g.path })
    };

    let mut names = HashSet::new();
    for l in &p.mesh_locks {
        if l.name.trim().is_empty() || !names.insert(("lock", l.name.clone())) {
            errs.push(format!("mesh lock name '{}' is empty or duplicated", l.name));
        }
    }
    for g in &p.glues {
        if g.name.trim().is_empty() || !names.insert(("glue", g.name.clone())) {
            errs.push(format!("glue name '{}' is empty or duplicated", g.name));
        }
    }

    // (driver unit, follower unit) edges and the resolved items with their follower unit
    let mut driven: HashSet<(Unit, usize)> = HashSet::new();
    let mut items: Vec<(Unit, usize, Item)> = Vec::new();
    let mut edges: Vec<(Unit, Unit, String)> = Vec::new();
    let mut seq = 0usize;
    let parallax_of = |u: Unit| e.link_root_parallax(u.node);

    for l in &p.mesh_locks {
        let what = format!("mesh lock '{}'", l.name);
        if !(l.strength.is_finite() && (0.0..=1.0).contains(&l.strength)) {
            errs.push(format!("{what}: strength must be within 0..1"));
            continue;
        }
        let (Some(h), Some(f)) = (unit(&what, &l.host, &mut errs, &mut rests), unit(&what, &l.follower, &mut errs, &mut rests)) else {
            continue;
        };
        if h == f {
            errs.push(format!("{what}: a mesh cannot be locked to itself"));
            continue;
        }
        if (parallax_of(h) - parallax_of(f)).abs() > 1e-6 {
            errs.push(format!("{what}: host and follower must have the same parallax"));
            continue;
        }
        let (hr, fr) = (&rests[&h.node], &rests[&f.node]);
        let hverts = &hr.paths[h.path];
        let fverts = &fr.paths[f.path];
        let idx: Vec<usize> = if l.vertices.is_empty() { (0..fverts.len()).collect() } else { l.vertices.clone() };
        if idx.is_empty() {
            errs.push(format!("{what}: the follower has no vertices"));
            continue;
        }
        let mut bad = false;
        let mut seen = HashSet::new();
        for &i in &idx {
            if i >= fverts.len() {
                errs.push(format!("{what}: follower vertex {i} does not exist (0..{})", fverts.len().saturating_sub(1)));
                bad = true;
                break;
            }
            if !seen.insert(i) {
                errs.push(format!("{what}: follower vertex {i} listed twice"));
                bad = true;
                break;
            }
            if !driven.insert((f, i)) {
                errs.push(format!("{what}: vertex {i} of '{}' is already driven by another constraint", l.follower.layer));
                bad = true;
                break;
            }
        }
        if bad {
            continue;
        }
        let hs: Vec<Vec2> = hverts.iter().map(|v| hr.stage.apply(*v)).collect();
        let fs: Vec<Vec2> = idx.iter().map(|&i| fr.stage.apply(fverts[i])).collect();
        let mut bind: Vec<Bind> = Vec::with_capacity(idx.len());
        if hr.is_image {
            if hr.tris.is_empty() || hr.tris.iter().any(|t| t.iter().any(|&k| k >= hs.len())) {
                errs.push(format!("{what}: host mesh has no valid triangles"));
                continue;
            }
            for &q in &fs {
                let mut best: Option<(f64, Bind)> = None;
                for t in &hr.tris {
                    let (w, d2) = closest_on_triangle(q, hs[t[0]], hs[t[1]], hs[t[2]]);
                    if best.as_ref().is_none_or(|b| d2 < b.0) {
                        best = Some((d2, Bind { v: *t, w }));
                    }
                }
                bind.push(best.unwrap().1);
            }
        } else {
            let n = hs.len();
            if n < 2 {
                errs.push(format!("{what}: a host path needs at least 2 anchors"));
                continue;
            }
            let segs = if hr.closed[h.path] { n } else { n - 1 };
            for &q in &fs {
                let mut best: Option<(f64, Bind)> = None;
                for s in 0..segs {
                    let (i, j) = (s, (s + 1) % n);
                    let (t, d2) = closest_on_segment(q, hs[i], hs[j]);
                    if best.as_ref().is_none_or(|b| d2 < b.0) {
                        best = Some((d2, Bind { v: [i, j, j], w: [1.0 - t, t, 0.0] }));
                    }
                }
                bind.push(best.unwrap().1);
            }
        }
        edges.push((h, f, what.clone()));
        items.push((f, seq, Item::Lock(RLock { f, h, idx, f_rest: fs, h_rest: hs, bind, strength: l.strength })));
        seq += 1;
    }

    for g in &p.glues {
        let what = format!("glue '{}'", g.name);
        if !(g.strength.is_finite() && (0.0..=1.0).contains(&g.strength)) {
            errs.push(format!("{what}: strength must be within 0..1"));
            continue;
        }
        if g.pairs.is_empty() {
            errs.push(format!("{what}: no vertex pairs"));
            continue;
        }
        let ga = GeomRef { layer: g.pairs[0].a.layer.clone(), path: g.pairs[0].a.path };
        let gb = GeomRef { layer: g.pairs[0].b.layer.clone(), path: g.pairs[0].b.path };
        if g.pairs.iter().any(|q| q.a.layer != ga.layer || q.a.path != ga.path || q.b.layer != gb.layer || q.b.path != gb.path) {
            errs.push(format!("{what}: all pairs must connect the same two paths (a side and b side)"));
            continue;
        }
        let (Some(a), Some(b)) = (unit(&what, &ga, &mut errs, &mut rests), unit(&what, &gb, &mut errs, &mut rests)) else { continue };
        if a == b {
            errs.push(format!("{what}: cannot glue a path to itself"));
            continue;
        }
        if (parallax_of(a) - parallax_of(b)).abs() > 1e-6 {
            errs.push(format!("{what}: both sides must have the same parallax"));
            continue;
        }
        let (ar, br) = (&rests[&a.node], &rests[&b.node]);
        let (av, bv) = (&ar.paths[a.path], &br.paths[b.path]);
        let mut pairs = Vec::new();
        let mut bad = false;
        for q in &g.pairs {
            if q.a.index >= av.len() || q.b.index >= bv.len() {
                errs.push(format!("{what}: pair ({}, {}) is out of range", q.a.index, q.b.index));
                bad = true;
                break;
            }
            if !driven.insert((b, q.b.index)) {
                errs.push(format!("{what}: vertex {} of '{}' is already driven by another constraint", q.b.index, gb.layer));
                bad = true;
                break;
            }
            pairs.push((q.a.index, q.b.index));
        }
        if bad {
            continue;
        }
        let a_rest = pairs.iter().map(|&(i, _)| ar.stage.apply(av[i])).collect();
        let b_rest = pairs.iter().map(|&(_, j)| br.stage.apply(bv[j])).collect();
        edges.push((a, b, what.clone()));
        items.push((b, seq, Item::Glue(RGlue { a, b, pairs, a_rest, b_rest, a_frame_rest: ar.stage, mode: g.mode, strength: g.strength })));
        seq += 1;
    }

    // topological order of units (Kahn); a cycle is an error
    let mut units: Vec<Unit> = Vec::new();
    for (d, f, _) in &edges {
        for u in [d, f] {
            if !units.contains(u) {
                units.push(*u);
            }
        }
    }
    let mut indeg: HashMap<Unit, usize> = units.iter().map(|u| (*u, 0)).collect();
    for (_, f, _) in &edges {
        *indeg.get_mut(f).unwrap() += 1;
    }
    let mut rank: HashMap<Unit, usize> = HashMap::new();
    let mut ready: Vec<Unit> = units.iter().filter(|u| indeg[u] == 0).copied().collect();
    let mut next = 0;
    while let Some(u) = ready.first().copied() {
        ready.remove(0);
        rank.insert(u, next);
        next += 1;
        for (d, f, _) in &edges {
            if *d == u {
                let c = indeg.get_mut(f).unwrap();
                *c -= 1;
                if *c == 0 {
                    ready.push(*f);
                }
            }
        }
    }
    if rank.len() < units.len() {
        let stuck: Vec<&str> = edges.iter().filter(|(_, f, _)| !rank.contains_key(f)).map(|(_, _, w)| w.as_str()).collect();
        errs.push(format!("constraint cycle: {} depend on each other", stuck.join(", ")));
    }
    if !errs.is_empty() {
        return Err(errs);
    }
    items.sort_by_key(|(f, s, _)| (rank[f], *s));
    Ok(Some(Resolved { items: items.into_iter().map(|x| x.2).collect() }))
}

// ---- per frame ----

fn get_stage(nodes: &[RenderNode], stage: &[Affine2], u: Unit, i: usize) -> Option<Vec2> {
    let n = nodes.get(u.node)?;
    let local = if let Some(img) = &n.image {
        img.mesh.as_ref()?.positions.get(i).copied()?
    } else {
        n.paths.get(u.path)?.anchors.get(i)?.pos
    };
    Some(stage[u.node].apply(local))
}

/// Move vertex `i` of `u` towards stage-space `target` by `strength`.
fn set_stage(nodes: &mut [RenderNode], stage: &[Affine2], u: Unit, i: usize, target: Vec2, strength: f32) {
    let Some(inv) = stage[u.node].inverse() else { return };
    let want = inv.apply(target);
    if !vec_ok(want) {
        return;
    }
    let n = &mut nodes[u.node];
    if let Some(img) = &mut n.image {
        if let Some(pos) = img.mesh.as_mut().and_then(|m| m.positions.get_mut(i)) {
            *pos = pos.lerp(want, strength);
        }
    } else if let Some(a) = n.paths.get_mut(u.path).and_then(|p| p.anchors.get_mut(i)) {
        let new = a.pos.lerp(want, strength);
        let d = new - a.pos;
        a.pos = new;
        if let Some(h) = &mut a.handle_in {
            *h += d;
        }
        if let Some(h) = &mut a.handle_out {
            *h += d;
        }
    }
}

impl Resolved {
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Apply every constraint to the posed nodes (`stage[id]` = layer space -> stage).
    pub(crate) fn apply(&self, nodes: &mut [RenderNode], stage: &[Affine2]) {
        for it in &self.items {
            match it {
                Item::Lock(l) => {
                    // read the host first (the follower is a different unit, so order is irrelevant)
                    let mut now: Vec<Option<Vec2>> = vec![None; l.h_rest.len()];
                    for b in &l.bind {
                        for k in 0..3 {
                            if b.w[k] != 0.0 && now[b.v[k]].is_none() {
                                now[b.v[k]] = get_stage(nodes, stage, l.h, b.v[k]);
                            }
                        }
                    }
                    for (n, &fi) in l.idx.iter().enumerate() {
                        let b = &l.bind[n];
                        let mut d = Vec2::ZERO;
                        let mut ok = true;
                        for k in 0..3 {
                            if b.w[k] == 0.0 {
                                continue;
                            }
                            match now[b.v[k]] {
                                Some(h) => d += (h - l.h_rest[b.v[k]]) * b.w[k],
                                None => ok = false,
                            }
                        }
                        if ok {
                            set_stage(nodes, stage, l.f, fi, l.f_rest[n] + d, l.strength);
                        }
                    }
                }
                Item::Glue(g) => {
                    let rel = stage[g.a.node].mul(g.a_frame_rest.inverse().unwrap_or(Affine2::IDENTITY));
                    for (n, &(ai, bi)) in g.pairs.iter().enumerate() {
                        let Some(a_now) = get_stage(nodes, stage, g.a, ai) else { continue };
                        let target = match g.mode {
                            GlueMode::Snap => a_now,
                            GlueMode::Offset => a_now + (g.b_rest[n] - g.a_rest[n]),
                            GlueMode::Rigid => a_now + rel.apply(g.b_rest[n]) - rel.apply(g.a_rest[n]),
                        };
                        set_stage(nodes, stage, g.b, bi, target, g.strength);
                    }
                }
            }
        }
    }
}

// ---- editing helpers (Project -> Project, validated by compiling an engine) ----

fn compile(q: Project) -> Result<Project, String> {
    Engine::new(q.clone()).map_err(|v| v.join("; "))?;
    Ok(q)
}

pub fn add_mesh_lock(p: &Project, l: MeshLock) -> Result<Project, String> {
    if p.mesh_locks.iter().any(|x| x.name == l.name) {
        return Err(format!("a mesh lock named '{}' already exists", l.name));
    }
    let mut q = p.clone();
    q.mesh_locks.push(l);
    compile(q)
}

pub fn remove_mesh_lock(p: &Project, name: &str) -> Result<Project, String> {
    let i = p.mesh_locks.iter().position(|x| x.name == name).ok_or_else(|| format!("no mesh lock named '{name}'"))?;
    let mut q = p.clone();
    q.mesh_locks.remove(i);
    compile(q)
}

pub fn add_glue(p: &Project, g: Glue) -> Result<Project, String> {
    if p.glues.iter().any(|x| x.name == g.name) {
        return Err(format!("a glue named '{}' already exists", g.name));
    }
    let mut q = p.clone();
    q.glues.push(g);
    compile(q)
}

pub fn remove_glue(p: &Project, name: &str) -> Result<Project, String> {
    let i = p.glues.iter().position(|x| x.name == name).ok_or_else(|| format!("no glue named '{name}'"))?;
    let mut q = p.clone();
    q.glues.remove(i);
    compile(q)
}

/// Pair every vertex of `b` that lies within `radius` (stage space, rest pose) of a vertex of `a`
/// with its nearest `a` vertex (ties: lowest index). Used by `glue add ... auto=R`.
pub fn auto_pairs(p: &Project, a: &GeomRef, b: &GeomRef, radius: f32) -> Result<Vec<GluePair>, String> {
    if !(radius.is_finite() && radius >= 0.0) {
        return Err("auto radius must be a number >= 0".into());
    }
    let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
    let fetch = |g: &GeomRef| -> Result<Vec<Vec2>, String> {
        let n = e.node_index(&g.layer).ok_or_else(|| format!("no layer '{}'", g.layer))?;
        let r = e.link_unit(n).ok_or_else(|| format!("layer '{}' has no editable vertices", g.layer))?;
        let v = r.paths.get(g.path).ok_or_else(|| format!("layer '{}' has no path {}", g.layer, g.path))?;
        Ok(v.iter().map(|q| r.stage.apply(*q)).collect())
    };
    let (av, bv) = (fetch(a)?, fetch(b)?);
    let mut out = Vec::new();
    for (j, q) in bv.iter().enumerate() {
        let mut best: Option<(f32, usize)> = None;
        for (i, r) in av.iter().enumerate() {
            let d = r.distance(*q);
            if d <= radius && best.is_none_or(|b| d < b.0) {
                best = Some((d, i));
            }
        }
        if let Some((_, i)) = best {
            out.push(GluePair {
                a: VertexRef { layer: a.layer.clone(), path: a.path, index: i },
                b: VertexRef { layer: b.layer.clone(), path: b.path, index: j },
            });
        }
    }
    if out.is_empty() {
        return Err(format!("no vertex of '{}' lies within {radius} of a vertex of '{}'", b.layer, a.layer));
    }
    Ok(out)
}

/// Parse `layer` or `layer:path`.
pub fn parse_geom(s: &str) -> Result<GeomRef, String> {
    match s.rsplit_once(':') {
        Some((l, n)) if n.chars().all(|c| c.is_ascii_digit()) && !n.is_empty() && !l.is_empty() => {
            Ok(GeomRef { layer: l.to_string(), path: n.parse().map_err(|_| format!("bad path index '{n}'"))? })
        }
        _ if s.is_empty() => Err("empty layer reference".into()),
        _ => Ok(GeomRef { layer: s.to_string(), path: 0 }),
    }
}

/// Parse `i:j;i:j` into (a index, b index) pairs.
pub fn parse_pairs(s: &str) -> Result<Vec<(usize, usize)>, String> {
    let mut v = Vec::new();
    for part in s.split(';').filter(|x| !x.trim().is_empty()) {
        let (a, b) = part.split_once(':').ok_or_else(|| format!("bad pair '{part}' (use a:b)"))?;
        let n = |t: &str| t.trim().parse::<usize>().map_err(|_| format!("bad vertex index '{t}'"));
        v.push((n(a)?, n(b)?));
    }
    Ok(v)
}

/// Parse `1,2,5-8` into indices.
pub fn parse_indices(s: &str) -> Result<Vec<usize>, String> {
    let mut v = Vec::new();
    for part in s.split(',').filter(|x| !x.trim().is_empty()) {
        let n = |t: &str| t.trim().parse::<usize>().map_err(|_| format!("bad vertex index '{t}'"));
        match part.split_once('-') {
            Some((a, b)) => {
                let (a, b) = (n(a)?, n(b)?);
                if b < a || b - a > 100_000 {
                    return Err(format!("bad index range '{part}'"));
                }
                v.extend(a..=b);
            }
            None => v.push(n(part)?),
        }
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::{Ease, Track};
    use crate::path::{Anchor, VPath};
    use crate::project::{Bone, Channel, Layer, LayerKind, MeshSpec, Prop};

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }
    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut pm = tiny_skia::Pixmap::new(w, h).unwrap();
        pm.fill(tiny_skia::Color::from_rgba8(200, 40, 40, 255));
        pm.encode_png().unwrap()
    }
    fn two(a: f32, b: f32) -> Track {
        Track::with_keys(&[(0.0, a, Ease::Linear), (10.0, b, Ease::Linear)])
    }
    fn gref(l: &str) -> GeomRef {
        GeomRef { layer: l.into(), path: 0 }
    }
    fn vref(l: &str, p: usize, i: usize) -> VertexRef {
        VertexRef { layer: l.into(), path: p, index: i }
    }
    fn img(name: &str, at: Vec2, cols: usize, rows: usize) -> Layer {
        let mut l = Layer::image_from_png(name, &png(60, 60), at);
        if let LayerKind::Image { mesh, size, .. } = &mut l.kind {
            *size = Some(v(60.0, 60.0));
            *mesh = Some(MeshSpec { cols, rows, ..Default::default() });
        }
        l
    }
    fn pts(name: &str, ps: &[(f32, f32)]) -> Layer {
        Layer::vector(name, vec![VPath::new("p", false, ps.iter().map(|&(x, y)| Anchor::corner(v(x, y))).collect())])
    }
    fn base() -> Project {
        let mut p = Project::new("l", 400.0, 400.0, 24.0, 10);
        p.bones = vec![Bone::new("b", None, 100.0, v(100.0, 100.0), 0.0)];
        p
    }
    fn anchors(f: &crate::engine::Frame, n: usize) -> Vec<Vec2> {
        f.nodes[n].paths[0].anchors.iter().map(|a| f.nodes[n].world.apply(a.pos)).collect()
    }

    #[test]
    fn closest_point_helpers() {
        let (w, d) = closest_on_triangle(v(0.25, 0.25), v(0.0, 0.0), v(1.0, 0.0), v(0.0, 1.0));
        assert_eq!(d, 0.0);
        assert!((w[0] - 0.5).abs() < 1e-6 && (w[1] - 0.25).abs() < 1e-6 && (w[2] - 0.25).abs() < 1e-6);
        let (w, d) = closest_on_triangle(v(2.0, 0.0), v(0.0, 0.0), v(1.0, 0.0), v(0.0, 1.0));
        assert!((d - 1.0).abs() < 1e-9 && (w[1] - 1.0).abs() < 1e-6, "{w:?} {d}");
        let (t, d2) = closest_on_segment(v(5.0, 3.0), v(0.0, 0.0), v(10.0, 0.0));
        assert!((t - 0.5).abs() < 1e-6 && (d2 - 9.0).abs() < 1e-9);
    }

    #[test]
    fn locked_vector_follower_moves_with_the_host_mesh() {
        let mut p = base();
        // host: image mesh skinned fully to bone b; follower: three points over it
        p.layers = vec![img("body", v(100.0, 100.0), 2, 2), pts("coat", &[(110.0, 110.0), (130.0, 150.0), (150.0, 125.0)])];
        p.auto_skin("body", 1).unwrap();
        p.channels = vec![Channel::keys("b", Prop::BoneAngle, two(0.0, 90.0))];
        p.mesh_locks = vec![MeshLock { name: "l".into(), host: gref("body"), follower: gref("coat"), vertices: vec![], strength: 1.0 }];
        let e = Engine::new(p.clone()).unwrap();
        let rest = anchors(&e.evaluate(0.0), 1);
        for (a, b) in rest.iter().zip([v(110.0, 110.0), v(130.0, 150.0), v(150.0, 125.0)]) {
            assert!(a.distance(b) < 1e-3, "{a:?}");
        }
        // bone b sits at (100,100): rotating 90 degrees about it maps (x,y) -> (100-(y-100), 100+(x-100))
        let turned = anchors(&e.evaluate(10.0), 1);
        for (a, (x, y)) in turned.iter().zip([(110.0, 110.0), (130.0, 150.0), (150.0, 125.0)]) {
            let want = v(100.0 - (y - 100.0), 100.0 + (x - 100.0));
            assert!(a.distance(want) < 0.05, "{a:?} vs {want:?}");
        }
        // without the lock the coat (unskinned) stays put
        let mut q = p.clone();
        q.mesh_locks.clear();
        let still = anchors(&Engine::new(q).unwrap().evaluate(10.0), 1);
        assert!(still[1].distance(v(130.0, 150.0)) < 1e-3);
    }

    #[test]
    fn lock_strength_blends_and_handles_travel_with_the_anchor() {
        let mut p = base();
        p.layers = vec![img("body", v(100.0, 100.0), 2, 2), pts("coat", &[(110.0, 110.0), (150.0, 150.0)])];
        p.auto_skin("body", 1).unwrap();
        if let LayerKind::Vector { paths } = &mut p.layers[1].kind {
            paths[0].anchors[0].handle_out = Some(v(120.0, 110.0));
        }
        p.channels = vec![Channel::keys("b", Prop::BoneOffsetX, two(0.0, 40.0))];
        p.mesh_locks = vec![MeshLock { name: "l".into(), host: gref("body"), follower: gref("coat"), vertices: vec![0], strength: 0.5 }];
        let e = Engine::new(p).unwrap();
        let f = e.evaluate(10.0);
        let a = &f.nodes[1].paths[0].anchors;
        assert!(a[0].pos.distance(v(130.0, 110.0)) < 1e-3, "{:?}", a[0].pos);
        assert!(a[0].handle_out.unwrap().distance(v(140.0, 110.0)) < 1e-3);
        assert!(a[1].pos.distance(v(150.0, 150.0)) < 1e-6, "vertex 1 is not locked");
    }

    #[test]
    fn point_outside_the_host_uses_the_nearest_surface_point() {
        let mut p = base();
        // follower sits 30 px above the top edge of the body: it still follows the edge
        p.layers = vec![img("body", v(100.0, 100.0), 1, 1), pts("hat", &[(130.0, 70.0)])];
        p.auto_skin("body", 1).unwrap();
        p.channels = vec![Channel::keys("b", Prop::BoneOffsetX, two(0.0, 25.0))];
        p.mesh_locks = vec![MeshLock { name: "l".into(), host: gref("body"), follower: gref("hat"), vertices: vec![], strength: 1.0 }];
        let e = Engine::new(p).unwrap();
        let a = anchors(&e.evaluate(10.0), 1)[0];
        assert!(a.distance(v(155.0, 70.0)) < 1e-3, "{a:?}");
    }

    #[test]
    fn lock_to_a_vector_host_follows_the_polyline() {
        let mut p = base();
        let mut wrist = pts("wrist", &[(100.0, 200.0), (200.0, 200.0)]);
        if let LayerKind::Vector { paths } = &mut wrist.kind {
            for a in &mut paths[0].anchors {
                a.weights = vec![crate::path::BoneWeight { bone: "b".into(), weight: 1.0 }];
            }
        }
        p.layers = vec![wrist, pts("cuff", &[(150.0, 190.0)])];
        p.channels = vec![Channel::keys("b", Prop::BoneOffsetY, two(0.0, 30.0))];
        p.mesh_locks = vec![MeshLock { name: "c".into(), host: gref("wrist"), follower: gref("cuff"), vertices: vec![], strength: 1.0 }];
        let e = Engine::new(p).unwrap();
        let a = anchors(&e.evaluate(10.0), 1)[0];
        assert!(a.distance(v(150.0, 220.0)) < 1e-3, "{a:?}");
    }

    #[test]
    fn glue_rigid_keeps_the_offset_under_bone_animation() {
        let mut p = base();
        let mut a = pts("a", &[(150.0, 100.0)]);
        a.bone = Some("b".into());
        let mut b = pts("b2", &[(150.0, 130.0)]);
        b.bone = Some("b".into());
        // a third layer glued rigidly to a, but itself not attached to any bone
        p.layers = vec![a, b, pts("c", &[(180.0, 100.0)])];
        p.channels = vec![Channel::keys("b", Prop::BoneAngle, two(0.0, 90.0))];
        p.glues = vec![Glue {
            name: "g".into(),
            mode: GlueMode::Rigid,
            pairs: vec![GluePair { a: vref("a", 0, 0), b: vref("c", 0, 0) }],
            strength: 1.0,
        }];
        let e = Engine::new(p).unwrap();
        for f in [0.0, 3.0, 10.0] {
            let fr = e.evaluate(f);
            let (pa, pb, pc) = (anchors(&fr, 0)[0], anchors(&fr, 1)[0], anchors(&fr, 2)[0]);
            // a rod from a: constant length, rotating with the bone like b2 does
            assert!((pc.distance(pa) - 30.0).abs() < 1e-3, "frame {f}: {}", pc.distance(pa));
            assert!((pb.distance(pa) - 30.0).abs() < 1e-3);
            let ang_c = (pc.y - pa.y).atan2(pc.x - pa.x);
            // (30, 0) rotated with the layer: the bone angle at this frame
            let want = (f / 10.0 * 90.0f32).to_radians();
            assert!((ang_c - want).abs() < 2e-3, "frame {f}: {ang_c} vs {want}");
        }
    }

    #[test]
    fn glue_snap_and_offset_modes() {
        let mut p = base();
        let mut a = pts("a", &[(150.0, 100.0)]);
        a.bone = Some("b".into());
        p.layers = vec![a, pts("s", &[(200.0, 220.0)]), pts("o", &[(200.0, 240.0)])];
        p.channels = vec![Channel::keys("b", Prop::BoneOffsetX, two(0.0, 40.0))];
        let pair = |l: &str| vec![GluePair { a: vref("a", 0, 0), b: vref(l, 0, 0) }];
        p.glues = vec![
            Glue { name: "s".into(), mode: GlueMode::Snap, pairs: pair("s"), strength: 1.0 },
            Glue { name: "o".into(), mode: GlueMode::Offset, pairs: pair("o"), strength: 1.0 },
        ];
        let e = Engine::new(p).unwrap();
        let fr = e.evaluate(10.0);
        let pa = anchors(&fr, 0)[0];
        assert!(pa.distance(v(190.0, 100.0)) < 1e-3);
        assert!(anchors(&fr, 1)[0].distance(pa) < 1e-3, "snap coincides");
        assert!(anchors(&fr, 2)[0].distance(v(190.0 + 50.0, 100.0 + 140.0)) < 1e-3, "offset kept");
    }

    #[test]
    fn chains_are_ordered_by_dependency_not_declaration() {
        let mut p = base();
        let mut a = pts("a", &[(150.0, 100.0)]);
        a.bone = Some("b".into());
        // c follows b2 which follows a; declared in the "wrong" order
        p.layers = vec![a, pts("b2", &[(150.0, 130.0)]), pts("c", &[(150.0, 160.0)])];
        p.channels = vec![Channel::keys("b", Prop::BoneOffsetX, two(0.0, 40.0))];
        let g = |n: &str, x: &str, y: &str| Glue { name: n.into(), mode: GlueMode::Offset, pairs: vec![GluePair { a: vref(x, 0, 0), b: vref(y, 0, 0) }], strength: 1.0 };
        p.glues = vec![g("second", "b2", "c"), g("first", "a", "b2")];
        let e = Engine::new(p).unwrap();
        let fr = e.evaluate(10.0);
        assert!(anchors(&fr, 2)[0].distance(v(190.0, 160.0)) < 1e-3, "{:?}", anchors(&fr, 2)[0]);
    }

    #[test]
    fn cycles_self_references_and_overlaps_are_rejected() {
        let mut p = base();
        p.layers = vec![pts("a", &[(0.0, 0.0), (10.0, 0.0)]), pts("b", &[(0.0, 5.0), (10.0, 5.0)]), pts("c", &[(0.0, 9.0)])];
        let pair = |x: &str, y: &str| vec![GluePair { a: vref(x, 0, 0), b: vref(y, 0, 0) }];
        let glue = |n: &str, x: &str, y: &str| Glue { name: n.into(), mode: GlueMode::Snap, pairs: pair(x, y), strength: 1.0 };
        // a <- b <- a
        let mut q = p.clone();
        q.glues = vec![glue("g1", "a", "b"), glue("g2", "b", "a")];
        let err = Engine::new(q).err().unwrap().join(";");
        assert!(err.contains("cycle"), "{err}");
        // cycle through a lock and a glue
        let mut q = p.clone();
        q.glues = vec![glue("g1", "a", "b")];
        q.mesh_locks = vec![MeshLock { name: "m".into(), host: gref("b"), follower: gref("a"), vertices: vec![], strength: 1.0 }];
        assert!(Engine::new(q).err().unwrap().join(";").contains("cycle"));
        // a lock to itself / a glue of a path with itself
        let mut q = p.clone();
        q.mesh_locks = vec![MeshLock { name: "m".into(), host: gref("a"), follower: gref("a"), vertices: vec![], strength: 1.0 }];
        assert!(Engine::new(q).err().unwrap().join(";").contains("itself"));
        let mut q = p.clone();
        q.glues = vec![glue("g", "a", "a")];
        assert!(Engine::new(q).err().unwrap().join(";").contains("itself"));
        // two constraints driving the same vertex
        let mut q = p.clone();
        q.glues = vec![glue("g1", "a", "c"), glue("g2", "b", "c")];
        assert!(Engine::new(q).err().unwrap().join(";").contains("already driven"));
        // the add helpers refuse the same things and leave the input alone
        let mut q = p.clone();
        q.glues = vec![glue("g1", "a", "b")];
        assert!(add_glue(&q, glue("g2", "b", "a")).is_err());
        assert!(add_glue(&q, glue("g1", "a", "c")).is_err(), "duplicate name");
        assert_eq!(q.glues.len(), 1);
    }

    #[test]
    fn validation_errors_are_reported_not_panicked() {
        let mut p = base();
        p.layers = vec![pts("a", &[(0.0, 0.0), (10.0, 0.0)]), pts("b", &[(0.0, 5.0)]), Layer::image_from_png("im", &png(8, 8), v(0.0, 0.0))];
        let lock = |h: &str, f: &str, vs: Vec<usize>, s: f32| MeshLock { name: "m".into(), host: gref(h), follower: gref(f), vertices: vs, strength: s };
        for (l, needle) in [
            (lock("zz", "b", vec![], 1.0), "no layer"),
            (lock("a", "b", vec![5], 1.0), "does not exist"),
            (lock("a", "b", vec![0, 0], 1.0), "twice"),
            (lock("a", "b", vec![], 1.5), "strength"),
            (lock("a", "im", vec![], 1.0), "no mesh"),
            (lock("b", "a", vec![], 1.0), "at least 2 anchors"),
            (MeshLock { host: GeomRef { layer: "a".into(), path: 3 }, ..lock("a", "b", vec![], 1.0) }, "no path 3"),
        ] {
            let mut q = p.clone();
            q.mesh_locks = vec![l];
            let err = Engine::new(q).err().unwrap_or_default().join(";");
            assert!(err.contains(needle), "wanted '{needle}' in '{err}'");
        }
        let mut q = p.clone();
        q.glues = vec![Glue { name: "g".into(), mode: GlueMode::Snap, pairs: vec![], strength: 1.0 }];
        assert!(Engine::new(q).err().unwrap().join(";").contains("no vertex pairs"));
        let mut q = p.clone();
        q.glues = vec![Glue { name: "g".into(), mode: GlueMode::Snap, pairs: vec![GluePair { a: vref("a", 0, 9), b: vref("b", 0, 0) }], strength: 1.0 }];
        assert!(Engine::new(q).err().unwrap().join(";").contains("out of range"));
        assert!(remove_glue(&p, "none").is_err() && remove_mesh_lock(&p, "none").is_err());
    }

    #[test]
    fn serde_defaults_and_old_projects_load() {
        let p = base();
        let j = p.to_json_pretty();
        assert!(!j.contains("mesh_locks") && !j.contains("glues"), "empty lists are not written");
        let back = Project::from_json(&j).unwrap();
        assert!(back.mesh_locks.is_empty() && back.glues.is_empty());
        // the bundled demo (written before this feature) loads and evaluates unchanged
        let d = crate::demo::project();
        let before = Engine::new(d.clone()).unwrap().evaluate(7.0).nodes.len();
        assert!(before > 0);
        let mut q = base();
        q.layers = vec![pts("a", &[(0.0, 0.0)]), pts("b", &[(0.0, 5.0)])];
        let q = add_glue(&q, Glue { name: "g".into(), mode: GlueMode::Rigid, pairs: vec![GluePair { a: vref("a", 0, 0), b: vref("b", 0, 0) }], strength: 1.0 }).unwrap();
        let again = Project::from_json(&q.to_json_pretty()).unwrap();
        assert_eq!(again.glues, q.glues);
        assert_eq!(again.glues[0].mode, GlueMode::Rigid);
    }

    #[test]
    fn evaluation_is_deterministic() {
        let mut p = base();
        p.layers = vec![img("body", v(100.0, 100.0), 3, 3), pts("coat", &[(110.0, 110.0), (130.0, 150.0)])];
        p.auto_skin("body", 1).unwrap();
        p.channels = vec![Channel::keys("b", Prop::BoneAngle, two(0.0, 70.0))];
        p.mesh_locks = vec![MeshLock { name: "l".into(), host: gref("body"), follower: gref("coat"), vertices: vec![], strength: 1.0 }];
        let e = Engine::new(p).unwrap();
        let (a, b) = (anchors(&e.evaluate(4.0), 1), anchors(&e.evaluate(4.0), 1));
        assert_eq!(a, b);
    }

    #[test]
    fn auto_pairs_and_parsers() {
        let mut p = base();
        p.layers = vec![pts("a", &[(0.0, 0.0), (100.0, 0.0)]), pts("b", &[(1.0, 1.0), (50.0, 0.0), (101.0, 0.0)])];
        let pr = auto_pairs(&p, &gref("a"), &gref("b"), 3.0).unwrap();
        let ix: Vec<(usize, usize)> = pr.iter().map(|x| (x.a.index, x.b.index)).collect();
        assert_eq!(ix, vec![(0, 0), (1, 2)]);
        assert!(auto_pairs(&p, &gref("a"), &gref("b"), 0.1).is_err());
        assert!(auto_pairs(&p, &gref("a"), &gref("b"), f32::NAN).is_err());
        assert_eq!(parse_geom("arm:2").unwrap(), GeomRef { layer: "arm".into(), path: 2 });
        assert_eq!(parse_geom("g/arm").unwrap(), gref("g/arm"));
        assert!(parse_geom("").is_err());
        assert_eq!(parse_pairs("0:1;2:3").unwrap(), vec![(0, 1), (2, 3)]);
        assert!(parse_pairs("0-1").is_err());
        assert_eq!(parse_indices("1,3-5").unwrap(), vec![1, 3, 4, 5]);
        assert!(parse_indices("5-3").is_err() && parse_indices("x").is_err());
    }

    #[test]
    fn works_through_the_editor_with_undo() {
        use crate::edit::{Editor, Op};
        let mut p = base();
        p.layers = vec![pts("a", &[(0.0, 0.0)]), pts("b", &[(0.0, 5.0)])];
        let mut ed = Editor::new(p).unwrap();
        let g = |n: &str, x: &str, y: &str| Glue { name: n.into(), mode: GlueMode::Snap, pairs: vec![GluePair { a: vref(x, 0, 0), b: vref(y, 0, 0) }], strength: 1.0 };
        // the empty list is not serialised, so the first glue creates the key
        let first = Op::Set { path: "/glues".into(), value: serde_json::json!([g("g1", "a", "b")]) };
        let ins = |gl: &Glue| Op::Insert { path: "/glues/-".into(), value: serde_json::to_value(gl).unwrap() };
        ed.apply(first).unwrap();
        assert_eq!(ed.project().glues.len(), 1);
        // a cycle is rolled back by the editor
        assert!(ed.apply(ins(&g("g2", "b", "a"))).is_err());
        assert_eq!(ed.project().glues.len(), 1);
        assert!(ed.undo());
        assert!(ed.project().glues.is_empty());
        assert!(ed.redo());
        assert_eq!(ed.project().glues.len(), 1);
    }
}
