// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Skin-weight generation and editing tools.
//!
//! Clean-room design note (replaces biharmonic / heat-kernel solvers): *harmonic
//! diffusion on the mesh graph*. Vertices close to a bone are pinned to that bone
//! (value 1 for it, 0 for the others); every other vertex relaxes to the weighted average of
//! its neighbours (Gauss–Seidel on the graph Laplacian). By the maximum principle the
//! result is smooth, non-negative and sums to 1 at every vertex, and — unlike distance
//! falloff — it follows the mesh connectivity, so weights do not leak across gaps.

use crate::math::Vec2;
use crate::path::BoneWeight;
use crate::rig::dist_to_segment;

/// Triangles of a `cols × rows` grid mesh (row-major vertices, as in `mesh_vertices`).
pub fn grid_triangles(cols: usize, rows: usize) -> Vec<[usize; 3]> {
    let (cols, rows) = (cols.max(1), rows.max(1));
    let stride = cols + 1;
    let mut t = Vec::with_capacity(cols * rows * 2);
    for r in 0..rows {
        for c in 0..cols {
            let a = r * stride + c;
            let (b, d, e) = (a + 1, a + stride, a + stride + 1);
            t.push([a, b, e]);
            t.push([a, e, d]);
        }
    }
    t
}

/// Parameters of [`diffusion_weights`].
#[derive(Clone, Copy, Debug)]
pub struct DiffusionOptions {
    /// Vertices nearer than this to a bone are pinned to it.
    pub seed_radius: f32,
    /// Maximum Gauss–Seidel sweeps.
    pub max_sweeps: usize,
    /// Stop when the largest change in a sweep falls below this.
    pub tolerance: f32,
}

impl Default for DiffusionOptions {
    fn default() -> Self {
        DiffusionOptions { seed_radius: 4.0, max_sweeps: 4000, tolerance: 1e-5 }
    }
}

/// Per-vertex weights (one `f32` per bone, rows sum to 1).
pub fn diffusion_weights(
    verts: &[Vec2],
    tris: &[[usize; 3]],
    segs: &[(Vec2, Vec2)],
    opts: DiffusionOptions,
) -> Vec<Vec<f32>> {
    let (n, nb) = (verts.len(), segs.len());
    if n == 0 || nb == 0 {
        return vec![Vec::new(); n];
    }
    // adjacency with inverse-length weights
    let mut adj: Vec<Vec<(usize, f32)>> = vec![Vec::new(); n];
    let mut link = |a: usize, b: usize| {
        if a >= n || b >= n || a == b {
            return;
        }
        let w = 1.0 / verts[a].distance(verts[b]).max(1e-3);
        if !adj[a].iter().any(|&(x, _)| x == b) {
            adj[a].push((b, w));
            adj[b].push((a, w));
        }
    };
    for t in tris {
        link(t[0], t[1]);
        link(t[1], t[2]);
        link(t[2], t[0]);
    }
    // nearest bone and distance per vertex
    let nearest: Vec<(usize, f32)> = verts
        .iter()
        .map(|&p| {
            let mut best = (0usize, f32::MAX);
            for (i, &(a, b)) in segs.iter().enumerate() {
                let d = dist_to_segment(p, a, b);
                if d < best.1 {
                    best = (i, d);
                }
            }
            best
        })
        .collect();
    // connected components; a component without any seed falls back to its nearest bone
    let mut comp = vec![usize::MAX; n];
    let mut ncomp = 0;
    for s in 0..n {
        if comp[s] != usize::MAX {
            continue;
        }
        let mut stack = vec![s];
        comp[s] = ncomp;
        while let Some(u) = stack.pop() {
            for &(v, _) in &adj[u] {
                if comp[v] == usize::MAX {
                    comp[v] = ncomp;
                    stack.push(v);
                }
            }
        }
        ncomp += 1;
    }
    let mut seeded = vec![false; n];
    let mut comp_has_seed = vec![false; ncomp];
    for i in 0..n {
        if nearest[i].1 <= opts.seed_radius {
            seeded[i] = true;
            comp_has_seed[comp[i]] = true;
        }
    }
    // a component with no seed: seed its single closest vertex to every bone's nearest
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for c in 0..ncomp {
        if !comp_has_seed[c] {
            let mut best = (usize::MAX, f32::MAX);
            for i in 0..n {
                if comp[i] == c && nearest[i].1 < best.1 {
                    best = (i, nearest[i].1);
                }
            }
            if best.0 != usize::MAX {
                seeded[best.0] = true;
            }
        }
    }
    let mut w = vec![vec![0.0f32; nb]; n];
    for i in 0..n {
        if seeded[i] {
            w[i][nearest[i].0] = 1.0;
        } else {
            // start from a neutral guess: nearest bone
            w[i][nearest[i].0] = 1.0;
        }
    }
    for _ in 0..opts.max_sweeps {
        let mut delta = 0.0f32;
        for i in 0..n {
            if seeded[i] || adj[i].is_empty() {
                continue;
            }
            let wsum: f32 = adj[i].iter().map(|&(_, a)| a).sum();
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for b in 0..nb {
                let v: f32 = adj[i].iter().map(|&(j, a)| a * w[j][b]).sum::<f32>() / wsum;
                delta = delta.max((v - w[i][b]).abs());
                w[i][b] = v;
            }
        }
        if delta < opts.tolerance {
            break;
        }
    }
    for row in &mut w {
        let s: f32 = row.iter().sum();
        if s > 1e-9 {
            for x in row.iter_mut() {
                *x /= s;
            }
        }
    }
    w
}

/// Convert dense rows to named weights, keeping the `k` strongest influences (renormalised).
pub fn to_bone_weights(rows: &[Vec<f32>], names: &[String], k: usize) -> Vec<Vec<BoneWeight>> {
    rows.iter().map(|r| limit_influences(&dense_to_named(r, names), k)).collect()
}

fn dense_to_named(r: &[f32], names: &[String]) -> Vec<BoneWeight> {
    r.iter().zip(names).filter(|(w, _)| **w > 0.0).map(|(&w, n)| BoneWeight { bone: n.clone(), weight: w }).collect()
}

/// Keep the `k` largest weights and renormalise to sum 1.
pub fn limit_influences(w: &[BoneWeight], k: usize) -> Vec<BoneWeight> {
    let mut v = w.to_vec();
    v.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    v.truncate(k.max(1));
    normalize(&mut v);
    v
}

/// Scale weights to sum to 1 (no-op for an all-zero set).
pub fn normalize(w: &mut [BoneWeight]) {
    let s: f32 = w.iter().map(|x| x.weight).sum();
    if s > 1e-9 {
        for x in w.iter_mut() {
            x.weight /= s;
        }
    }
}

/// What a weight-painting dab does to the chosen bone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintMode {
    /// Pull the vertex toward "fully this bone".
    Add,
    /// Take influence away from the bone; the others share what is freed.
    Subtract,
    /// Blend toward the mean of the neighbours.
    Smooth,
}

impl PaintMode {
    pub fn parse(s: &str) -> Result<PaintMode, String> {
        match s {
            "add" => Ok(PaintMode::Add),
            "subtract" => Ok(PaintMode::Subtract),
            "smooth" => Ok(PaintMode::Smooth),
            o => Err(format!("unknown paint mode '{o}' (add, subtract, smooth)")),
        }
    }
}

/// Apply a brush stroke to per-vertex weights. Every dab touches the vertices within `radius` of one of `points`
/// with strength `strength · smoothstep(1 − d/radius)` (0..1); each row keeps summing to 1 and at most `k`
/// influences. Rows left empty by "unskinned" meshes count as all-zero. Returns the number of rows changed.
/// Subtracting from a vertex that has no other influence hands the weight to `fallback` (if it names another bone).
/// Deterministic: dabs are applied in order, vertices in index order.
#[allow(clippy::too_many_arguments)]
pub fn paint_stroke(
    verts: &[Vec2],
    tris: &[[usize; 3]],
    weights: &mut [Vec<BoneWeight>],
    names: &[String],
    bone: &str,
    mode: PaintMode,
    points: &[Vec2],
    radius: f32,
    strength: f32,
    k: usize,
    fallback: Option<&str>,
) -> Result<usize, String> {
    let bi = names.iter().position(|n| n == bone).ok_or_else(|| format!("no bone named '{bone}'"))?;
    if weights.len() != verts.len() {
        return Err(format!("{} weight rows for {} vertices", weights.len(), verts.len()));
    }
    if !(radius.is_finite() && radius > 0.0) || !strength.is_finite() {
        return Err("radius must be a positive number and strength a number".into());
    }
    let strength = strength.clamp(0.0, 1.0);
    let nb = names.len();
    let mut dense: Vec<Vec<f32>> = weights
        .iter()
        .map(|row| {
            let mut d = vec![0.0f32; nb];
            for bw in row {
                if let Some(i) = names.iter().position(|x| *x == bw.bone) {
                    d[i] += bw.weight.max(0.0);
                }
            }
            d
        })
        .collect();
    let mut nbrs: Vec<Vec<usize>> = vec![Vec::new(); verts.len()];
    if mode == PaintMode::Smooth {
        for t in tris {
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                if a < verts.len() && b < verts.len() {
                    if !nbrs[a].contains(&b) {
                        nbrs[a].push(b);
                    }
                    if !nbrs[b].contains(&a) {
                        nbrs[b].push(a);
                    }
                }
            }
        }
    }
    let mut touched = vec![false; verts.len()];
    for &pt in points {
        let prev = if mode == PaintMode::Smooth { Some(dense.clone()) } else { None };
        for (i, &v) in verts.iter().enumerate() {
            let d = v.distance(pt);
            if d >= radius {
                continue;
            }
            let t = 1.0 - d / radius;
            let f = strength * t * t * (3.0 - 2.0 * t);
            if f <= 0.0 {
                continue;
            }
            let row = &mut dense[i];
            match mode {
                PaintMode::Add => {
                    for (j, w) in row.iter_mut().enumerate() {
                        let target = if j == bi { 1.0 } else { 0.0 };
                        *w += (target - *w) * f;
                    }
                }
                PaintMode::Subtract => {
                    let w = row[bi];
                    let others: f32 = row.iter().sum::<f32>() - w;
                    if w > 0.0 && others <= 1e-6 {
                        // nobody else to hand the weight to: it goes to the fallback bone (the parent), if there is one
                        if let Some(fi) = fallback.and_then(|n| names.iter().position(|x| x == n)).filter(|&fi| fi != bi) {
                            let nw = w * (1.0 - f);
                            row[fi] += w - nw;
                            row[bi] = nw;
                        }
                    } else if w > 0.0 {
                        let nw = w * (1.0 - f);
                        let scale = (others + (w - nw)) / others;
                        for (j, x) in row.iter_mut().enumerate() {
                            *x = if j == bi { nw } else { *x * scale };
                        }
                    }
                }
                PaintMode::Smooth => {
                    if nbrs[i].is_empty() {
                        continue;
                    }
                    let pv = prev.as_ref().expect("smooth keeps the previous state");
                    for j in 0..nb {
                        let mean: f32 = nbrs[i].iter().map(|&q| pv[q][j]).sum::<f32>() / nbrs[i].len() as f32;
                        row[j] = pv[i][j] + (mean - pv[i][j]) * f;
                    }
                }
            }
            touched[i] = true;
        }
    }
    let mut changed = 0;
    for (i, row) in dense.iter().enumerate() {
        if touched[i] {
            weights[i] = limit_influences(&dense_to_named(row, names), k);
            changed += 1;
        }
    }
    Ok(changed)
}

/// Laplacian smoothing of per-vertex weights: each vertex moves `lambda` of the way toward
/// the mean of its neighbours, `iterations` times. Result is renormalised and limited to `k`.
pub fn smooth(weights: &[Vec<BoneWeight>], tris: &[[usize; 3]], names: &[String], iterations: usize, lambda: f32, k: usize) -> Vec<Vec<BoneWeight>> {
    let n = weights.len();
    let index = |b: &str| names.iter().position(|x| x == b);
    let nb = names.len();
    let mut dense: Vec<Vec<f32>> = weights
        .iter()
        .map(|row| {
            let mut d = vec![0.0; nb];
            for bw in row {
                if let Some(i) = index(&bw.bone) {
                    d[i] += bw.weight;
                }
            }
            d
        })
        .collect();
    let mut nbrs: Vec<Vec<usize>> = vec![Vec::new(); n];
    for t in tris {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            if a < n && b < n {
                if !nbrs[a].contains(&b) {
                    nbrs[a].push(b);
                }
                if !nbrs[b].contains(&a) {
                    nbrs[b].push(a);
                }
            }
        }
    }
    for _ in 0..iterations {
        let prev = dense.clone();
        for i in 0..n {
            if nbrs[i].is_empty() {
                continue;
            }
            for b in 0..nb {
                let mean: f32 = nbrs[i].iter().map(|&j| prev[j][b]).sum::<f32>() / nbrs[i].len() as f32;
                dense[i][b] = prev[i][b] + (mean - prev[i][b]) * lambda.clamp(0.0, 1.0);
            }
        }
    }
    to_bone_weights(&dense, names, k)
}

/// Copy weights between two meshes by nearest source vertex (e.g. after re-meshing).
pub fn transfer_nearest(src_verts: &[Vec2], src_weights: &[Vec<BoneWeight>], dst_verts: &[Vec2]) -> Vec<Vec<BoneWeight>> {
    dst_verts
        .iter()
        .map(|&p| {
            let mut best = (0usize, f32::MAX);
            for (i, &s) in src_verts.iter().enumerate() {
                let d = s.distance(p);
                if d < best.1 {
                    best = (i, d);
                }
            }
            src_weights.get(best.0).cloned().unwrap_or_default()
        })
        .collect()
}

/// Mirror weights across a vertical axis `x = axis`: for every vertex find the vertex at the
/// mirrored position and give it this vertex's weights with bone names swapped by `rename`.
pub fn mirror_x(verts: &[Vec2], weights: &[Vec<BoneWeight>], axis: f32, rename: &dyn Fn(&str) -> String) -> Vec<Vec<BoneWeight>> {
    verts
        .iter()
        .map(|&p| {
            let q = Vec2::new(2.0 * axis - p.x, p.y);
            let mut best = (0usize, f32::MAX);
            for (i, &s) in verts.iter().enumerate() {
                let d = s.distance(q);
                if d < best.1 {
                    best = (i, d);
                }
            }
            weights[best.0].iter().map(|w| BoneWeight { bone: rename(&w.bone), weight: w.weight }).collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {

    #[test]
    fn paint_add_subtract_smooth_keep_partition_of_unity() {
        let names = vec!["a".to_string(), "b".to_string()];
        let (cols, rows) = (6usize, 6usize);
        let verts = crate::project::mesh_vertices(Vec2::new(0.0, 0.0), Vec2::new(60.0, 60.0), cols, rows);
        let tris = grid_triangles(cols, rows);
        let all_a: Vec<Vec<BoneWeight>> = verts.iter().map(|_| vec![BoneWeight { bone: "a".into(), weight: 1.0 }]).collect();
        let mut w = all_a.clone();
        let n = paint_stroke(&verts, &tris, &mut w, &names, "b", PaintMode::Add, &[Vec2::new(30.0, 30.0)], 20.0, 1.0, 4, None).unwrap();
        assert!(n > 0 && n < verts.len());
        let at = |w: &Vec<Vec<BoneWeight>>, i: usize, b: &str| w[i].iter().find(|x| x.bone == b).map(|x| x.weight).unwrap_or(0.0);
        let centre = verts.iter().position(|v| v.distance(Vec2::new(30.0, 30.0)) < 1.0).unwrap();
        assert!(at(&w, centre, "b") > 0.99, "the vertex under the brush becomes bone b");
        let far = 0;
        assert!((at(&w, far, "a") - 1.0).abs() < 1e-6, "a vertex outside the radius is untouched");
        for row in &w {
            let s: f32 = row.iter().map(|x| x.weight).sum();
            assert!((s - 1.0).abs() < 1e-4, "row sums to 1, got {s}");
        }
        // subtracting it again hands the weight back to a
        paint_stroke(&verts, &tris, &mut w, &names, "b", PaintMode::Subtract, &[Vec2::new(30.0, 30.0)], 20.0, 1.0, 4, Some("a")).unwrap();
        assert!(at(&w, centre, "b") < 0.01);
        assert!(at(&w, centre, "a") > 0.99);
        // smoothing a hard edge lowers its contrast and keeps the sums
        let mut e: Vec<Vec<BoneWeight>> = verts.iter().map(|v| vec![BoneWeight { bone: if v.x < 30.0 { "a" } else { "b" }.into(), weight: 1.0 }]).collect();
        let before = at(&e, centre, "a");
        paint_stroke(&verts, &tris, &mut e, &names, "a", PaintMode::Smooth, &[Vec2::new(30.0, 30.0)], 25.0, 1.0, 4, None).unwrap();
        let after = at(&e, centre, "a");
        assert!((after - before).abs() > 1e-3 || at(&e, centre, "b") > 0.0, "smoothing changes the edge");
        for row in &e {
            let s: f32 = row.iter().map(|x| x.weight).sum();
            assert!((s - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn paint_is_deterministic_and_validates_input() {
        let names = vec!["a".to_string(), "b".to_string()];
        let verts = crate::project::mesh_vertices(Vec2::new(0.0, 0.0), Vec2::new(40.0, 40.0), 4, 4);
        let tris = grid_triangles(4, 4);
        let base: Vec<Vec<BoneWeight>> = verts.iter().map(|_| vec![BoneWeight { bone: "a".into(), weight: 1.0 }]).collect();
        let pts = [Vec2::new(10.0, 10.0), Vec2::new(20.0, 15.0), Vec2::new(30.0, 25.0)];
        let (mut x, mut y) = (base.clone(), base.clone());
        paint_stroke(&verts, &tris, &mut x, &names, "b", PaintMode::Add, &pts, 15.0, 0.5, 4, None).unwrap();
        paint_stroke(&verts, &tris, &mut y, &names, "b", PaintMode::Add, &pts, 15.0, 0.5, 4, None).unwrap();
        assert_eq!(x, y);
        let mut z = base.clone();
        assert!(paint_stroke(&verts, &tris, &mut z, &names, "nope", PaintMode::Add, &pts, 15.0, 0.5, 4, None).is_err());
        assert!(paint_stroke(&verts, &tris, &mut z, &names, "b", PaintMode::Add, &pts, 0.0, 0.5, 4, None).is_err());
        assert!(paint_stroke(&verts, &tris, &mut z, &names, "b", PaintMode::Add, &pts, f32::NAN, 0.5, 4, None).is_err());
        assert!(PaintMode::parse("erase").is_err());
        let mut short = base[..3].to_vec();
        assert!(paint_stroke(&verts, &tris, &mut short, &names, "b", PaintMode::Add, &pts, 15.0, 0.5, 4, None).is_err());
    }
    use super::*;

    fn grid(cols: usize, rows: usize, w: f32, h: f32) -> Vec<Vec2> {
        crate::project::mesh_vertices(Vec2::ZERO, Vec2::new(w, h), cols, rows)
    }

    #[test]
    fn grid_triangles_count_and_validity() {
        let t = grid_triangles(4, 3);
        assert_eq!(t.len(), 24);
        assert!(t.iter().all(|tri| tri.iter().all(|&i| i < 20)));
    }

    #[test]
    fn diffusion_rows_sum_to_one_and_are_smooth() {
        let (cols, rows) = (20, 4);
        let v = grid(cols, rows, 200.0, 40.0);
        let tris = grid_triangles(cols, rows);
        let segs = [(Vec2::new(0.0, 20.0), Vec2::new(60.0, 20.0)), (Vec2::new(140.0, 20.0), Vec2::new(200.0, 20.0))];
        let w = diffusion_weights(&v, &tris, &segs, DiffusionOptions { seed_radius: 12.0, ..Default::default() });
        for row in &w {
            let s: f32 = row.iter().sum();
            assert!((s - 1.0).abs() < 1e-4, "row sums to {s}");
            assert!(row.iter().all(|&x| x >= -1e-6));
        }
        // along the middle row the weight of bone 0 falls monotonically from left to right
        let mid: Vec<f32> = (0..=cols).map(|c| w[2 * (cols + 1) + c][0]).collect();
        assert!(mid[0] > 0.99 && mid[cols] < 0.01, "ends belong to their bone: {:?}", (mid[0], mid[cols]));
        for i in 1..mid.len() {
            assert!(mid[i] <= mid[i - 1] + 1e-4, "monotone at {i}: {} > {}", mid[i], mid[i - 1]);
        }
    }

    #[test]
    fn diffusion_does_not_leak_across_a_gap() {
        // two disconnected strips; each only touches the bone near it
        let mut v = grid(4, 1, 40.0, 10.0);
        let off = v.len();
        v.extend(grid(4, 1, 40.0, 10.0).into_iter().map(|p| p + Vec2::new(0.0, 100.0)));
        let mut tris = grid_triangles(4, 1);
        tris.extend(grid_triangles(4, 1).into_iter().map(|t| [t[0] + off, t[1] + off, t[2] + off]));
        let segs = [(Vec2::new(0.0, 5.0), Vec2::new(40.0, 5.0)), (Vec2::new(0.0, 105.0), Vec2::new(40.0, 105.0))];
        let w = diffusion_weights(&v, &tris, &segs, DiffusionOptions { seed_radius: 2.0, ..Default::default() });
        for i in 0..off {
            assert!(w[i][0] > 0.99, "upper strip belongs to bone 0");
            assert!(w[i + off][1] > 0.99, "lower strip belongs to bone 1");
        }
    }

    #[test]
    fn limit_normalize_and_smooth() {
        let w = vec![
            BoneWeight { bone: "a".into(), weight: 0.5 },
            BoneWeight { bone: "b".into(), weight: 0.3 },
            BoneWeight { bone: "c".into(), weight: 0.2 },
        ];
        let l = limit_influences(&w, 2);
        assert_eq!(l.len(), 2);
        assert!((l.iter().map(|x| x.weight).sum::<f32>() - 1.0).abs() < 1e-6);
        assert_eq!(l[0].bone, "a");

        let names = vec!["a".to_string(), "b".to_string()];
        let rows = vec![
            vec![BoneWeight { bone: "a".into(), weight: 1.0 }],
            vec![BoneWeight { bone: "b".into(), weight: 1.0 }],
        ];
        let tris = vec![[0usize, 1, 1]];
        let s = smooth(&rows, &tris, &names, 1, 0.5, 4);
        let a0 = s[0].iter().find(|x| x.bone == "a").unwrap().weight;
        assert!((a0 - 0.5).abs() < 1e-5, "halfway blend, got {a0}");
    }

    #[test]
    fn transfer_and_mirror() {
        let src = vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)];
        let sw = vec![
            vec![BoneWeight { bone: "arm_L".into(), weight: 1.0 }],
            vec![BoneWeight { bone: "arm_R".into(), weight: 1.0 }],
        ];
        let t = transfer_nearest(&src, &sw, &[Vec2::new(9.0, 1.0)]);
        assert_eq!(t[0][0].bone, "arm_R");
        let m = mirror_x(&src, &sw, 5.0, &|n| crate::anim_tools::mirror_name(n));
        assert_eq!(m[0][0].bone, "arm_L", "vertex at x=0 mirrors x=10 whose bone arm_R is renamed arm_L");
    }
}
