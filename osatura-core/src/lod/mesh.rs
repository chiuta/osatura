// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Mesh simplification: half-edge collapse, generalised QEM order, certified error (part of `lod`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Mesh simplification: half-edge collapse, generalised QEM order, certified error

/// Input of [`qem_simplify`].
pub struct QemInput<'a> {
    /// Rest positions (layer space).
    pub rest: &'a [[f64; 2]],
    pub tris: &'a [[usize; 3]],
    /// Per vertex attribute vector (bone weights times a lever) steering the QEM order.
    pub attr: &'a [Vec<f64>],
    /// Deformed vertex positions per sample frame, `y[sample][vertex]` (layer space).
    pub y: &'a [Vec<[f32; 2]>],
    /// Pixels per layer unit at each sample (render scale times the world matrix norm).
    pub s: &'a [f64],
}

#[derive(Clone, Debug, PartialEq)]
pub struct QemOutput {
    /// Original indices of the kept vertices, ascending.
    pub keep: Vec<usize>,
    /// Triangles in terms of original vertex indices.
    pub tris: Vec<[usize; 3]>,
    /// Certified maximum deviation (px) of the deformed surface over the sampled frames.
    pub bound_px: f64,
    pub collapses: usize,
}

#[derive(Clone)]
pub(super) struct Quadric {
    pub(super) a: Vec<f64>,
    pub(super) b: Vec<f64>,
    pub(super) c: f64,
}

impl Quadric {
    pub(super) fn zero(d: usize) -> Quadric {
        Quadric { a: vec![0.0; d * d], b: vec![0.0; d], c: 0.0 }
    }
    pub(super) fn add(&mut self, o: &Quadric) {
        for (x, y) in self.a.iter_mut().zip(&o.a) {
            *x += y;
        }
        for (x, y) in self.b.iter_mut().zip(&o.b) {
            *x += y;
        }
        self.c += o.c;
    }
    pub(super) fn eval(&self, x: &[f64]) -> f64 {
        let d = x.len();
        let mut q = 0.0;
        for i in 0..d {
            let mut r = 0.0;
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for j in 0..d {
                r += self.a[i * d + j] * x[j];
            }
            q += x[i] * r - 2.0 * self.b[i] * x[i];
        }
        q + self.c
    }
}

pub(super) fn tri_quadric(x0: &[f64], x1: &[f64], x2: &[f64], w: f64) -> Option<Quadric> {
    let d = x0.len();
    let e1: Vec<f64> = (0..d).map(|i| x1[i] - x0[i]).collect();
    let e2: Vec<f64> = (0..d).map(|i| x2[i] - x0[i]).collect();
    let l1 = e1.iter().map(|v| v * v).sum::<f64>().sqrt();
    if l1 < 1e-300 {
        return None;
    }
    let u1: Vec<f64> = e1.iter().map(|v| v / l1).collect();
    let pr: f64 = (0..d).map(|i| e2[i] * u1[i]).sum();
    let mut u2: Vec<f64> = (0..d).map(|i| e2[i] - pr * u1[i]).collect();
    let l2 = u2.iter().map(|v| v * v).sum::<f64>().sqrt();
    if l2 < 1e-300 {
        return None;
    }
    for v in &mut u2 {
        *v /= l2;
    }
    let mut q = Quadric::zero(d);
    for i in 0..d {
        for j in 0..d {
            let a = (if i == j { 1.0 } else { 0.0 }) - u1[i] * u1[j] - u2[i] * u2[j];
            q.a[i * d + j] = a * w;
        }
    }
    for i in 0..d {
        q.b[i] = (0..d).map(|j| q.a[i * d + j] * x0[j]).sum();
    }
    q.c = (0..d).map(|i| q.b[i] * x0[i]).sum();
    Some(q)
}

#[derive(Clone, Copy)]
pub(super) struct Cand {
    pub(super) cost: f64,
    pub(super) u: usize,
    pub(super) v: usize,
    pub(super) vu: u32,
    pub(super) vv: u32,
}
impl PartialEq for Cand {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == std::cmp::Ordering::Equal
    }
}
impl Eq for Cand {}
impl PartialOrd for Cand {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Cand {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        // smaller cost = greater priority; ties by (u, v) ascending
        o.cost.total_cmp(&self.cost).then(o.u.cmp(&self.u)).then(o.v.cmp(&self.v))
    }
}

pub(super) fn area2(a: P, b: P, c: P) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// Clip the triangle `t` (a convex polygon) against triangle `o`; returns the polygon corners.
pub(super) fn clip_tri(t: [P; 3], o: [P; 3], tol: f64) -> Vec<P> {
    let sgn = if area2(o[0], o[1], o[2]) >= 0.0 { 1.0 } else { -1.0 };
    let mut poly: Vec<P> = t.to_vec();
    for k in 0..3 {
        let (a, b) = (o[k], o[(k + 1) % 3]);
        let side = |p: P| sgn * ((b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]));
        let mut next: Vec<P> = Vec::with_capacity(poly.len() + 2);
        for i in 0..poly.len() {
            let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
            let (sp, sq) = (side(p), side(q));
            let (ip, iq) = (sp >= -tol, sq >= -tol);
            if ip {
                next.push(p);
            }
            if ip != iq {
                let t = sp / (sp - sq);
                next.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
            }
        }
        poly = next;
        if poly.is_empty() {
            break;
        }
    }
    poly
}

pub(super) fn bary(p: P, t: [P; 3]) -> [f64; 3] {
    let d = area2(t[0], t[1], t[2]);
    if d.abs() < 1e-300 {
        return [1.0, 0.0, 0.0];
    }
    let l0 = area2(p, t[1], t[2]) / d;
    let l1 = area2(t[0], p, t[2]) / d;
    [l0, l1, 1.0 - l0 - l1]
}

/// Greedy half-edge collapse (QEM order); every accepted collapse keeps the certified deviation from the
/// original mesh within `eps_px`. The domain (outline) of the mesh is preserved exactly.
pub fn qem_simplify(inp: &QemInput, eps_px: f64) -> QemOutput {
    let n = inp.rest.len();
    let nt = inp.tris.len();
    let identity = || QemOutput { keep: (0..n).collect(), tris: inp.tris.to_vec(), bound_px: 0.0, collapses: 0 };
    if n == 0 || nt == 0 || (eps_px.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || inp.y.is_empty() || inp.y.iter().any(|f| f.len() != n) || inp.s.len() != inp.y.len() {
        return identity();
    }
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for r in inp.rest {
        x0 = x0.min(r[0]);
        y0 = y0.min(r[1]);
        x1 = x1.max(r[0]);
        y1 = y1.max(r[1]);
    }
    let diag = (x1 - x0).hypot(y1 - y0).max(1e-12);
    let atol = 1e-12 * diag * diag;
    let d = 2 + inp.attr.first().map_or(0, |a| a.len());
    let xv = |v: usize| -> Vec<f64> {
        let mut x = vec![inp.rest[v][0], inp.rest[v][1]];
        x.extend_from_slice(&inp.attr[v]);
        x
    };
    let mut tris: Vec<[usize; 3]> = inp.tris.to_vec();
    let mut alive = vec![true; nt];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (t, tr) in tris.iter().enumerate() {
        for &v in tr {
            adj[v].push(t);
        }
    }
    let mut quad: Vec<Quadric> = vec![Quadric::zero(d); n];
    for tr in &tris {
        let (a, b, c) = (xv(tr[0]), xv(tr[1]), xv(tr[2]));
        let w = area2(inp.rest[tr[0]], inp.rest[tr[1]], inp.rest[tr[2]]).abs() * 0.5 / (diag * diag);
        if let Some(q) = tri_quadric(&a, &b, &c, w) {
            for &v in tr {
                quad[v].add(&q);
            }
        }
    }
    let mut cover: Vec<Vec<usize>> = (0..nt).map(|t| vec![t]).collect();
    let mut terr = vec![0.0f64; nt];
    let mut ver = vec![0u32; n];

    let neighbours = |adj: &Vec<Vec<usize>>, tris: &Vec<[usize; 3]>, v: usize| -> Vec<usize> {
        let mut s: Vec<usize> = adj[v].iter().flat_map(|&t| tris[t].iter().copied()).filter(|&x| x != v).collect();
        s.sort_unstable();
        s.dedup();
        s
    };
    let common_tris = |adj: &Vec<Vec<usize>>, u: usize, v: usize| -> Vec<usize> { adj[u].iter().copied().filter(|t| adj[v].contains(t)).collect() };

    // validity of the collapse of u into v (topology, domain, orientation)
    let valid = |adj: &Vec<Vec<usize>>, tris: &Vec<[usize; 3]>, u: usize, v: usize| -> bool {
        let tuv = common_tris(adj, u, v);
        if tuv.is_empty() || tuv.len() > 2 {
            return false;
        }
        // boundary vertex rules
        let nu = neighbours(adj, tris, u);
        let mut boundary_nbrs: Vec<usize> = Vec::new();
        for &x in &nu {
            if common_tris(adj, u, x).len() == 1 {
                boundary_nbrs.push(x);
            }
        }
        if !boundary_nbrs.is_empty() {
            if tuv.len() != 1 || boundary_nbrs.len() != 2 || !boundary_nbrs.contains(&v) {
                return false;
            }
            let w = *boundary_nbrs.iter().find(|&&x| x != v).unwrap();
            if dist_pt_seg(inp.rest[u], inp.rest[v], inp.rest[w]) > 1e-9 * diag {
                return false;
            }
        }
        // link condition
        let nv = neighbours(adj, tris, v);
        let common: BTreeSet<usize> = nu.iter().copied().filter(|x| nv.contains(x)).collect();
        let apex: BTreeSet<usize> = tuv.iter().flat_map(|&t| tris[t].iter().copied()).filter(|&x| x != u && x != v).collect();
        if common != apex {
            return false;
        }
        // orientation of the new triangles
        for &t in &adj[u] {
            if tuv.contains(&t) {
                continue;
            }
            let tr = tris[t];
            let old = area2(inp.rest[tr[0]], inp.rest[tr[1]], inp.rest[tr[2]]);
            let nt_ = tr.map(|x| if x == u { v } else { x });
            let new = area2(inp.rest[nt_[0]], inp.rest[nt_[1]], inp.rest[nt_[2]]);
            if new * old <= 0.0 || new.abs() < 1e-9 * diag * diag {
                return false;
            }
        }
        true
    };

    let qcost = |quad: &Vec<Quadric>, u: usize, v: usize| -> f64 {
        let mut q = quad[u].clone();
        q.add(&quad[v]);
        let e = q.eval(&xv(v)).max(0.0);
        let l = dist(inp.rest[u], inp.rest[v]) / diag;
        e + 1e-9 * l * l
    };

    let mut heap: BinaryHeap<Cand> = BinaryHeap::new();
    for u in 0..n {
        for v in neighbours(&adj, &tris, u) {
            if valid(&adj, &tris, u, v) {
                heap.push(Cand { cost: qcost(&quad, u, v), u, v, vu: 0, vv: 0 });
            }
        }
    }

    // exact error of a (possibly collapsed) triangle against the original mesh
    let tri_error = |t3: [usize; 3], cov: &[usize]| -> f64 {
        let tp = [inp.rest[t3[0]], inp.rest[t3[1]], inp.rest[t3[2]]];
        let (tx0, ty0, tx1, ty1) = (
            tp.iter().map(|p| p[0]).fold(f64::MAX, f64::min),
            tp.iter().map(|p| p[1]).fold(f64::MAX, f64::min),
            tp.iter().map(|p| p[0]).fold(f64::MIN, f64::max),
            tp.iter().map(|p| p[1]).fold(f64::MIN, f64::max),
        );
        // corner list: (coefficients on original vertices of the new tri, of the original tri)
        let mut worst = 0.0f64;
        for &o in cov {
            let ot = inp.tris[o];
            let op = [inp.rest[ot[0]], inp.rest[ot[1]], inp.rest[ot[2]]];
            if op.iter().map(|p| p[0]).fold(f64::MAX, f64::min) > tx1 + 1e-12 * diag
                || op.iter().map(|p| p[0]).fold(f64::MIN, f64::max) < tx0 - 1e-12 * diag
                || op.iter().map(|p| p[1]).fold(f64::MAX, f64::min) > ty1 + 1e-12 * diag
                || op.iter().map(|p| p[1]).fold(f64::MIN, f64::max) < ty0 - 1e-12 * diag
            {
                continue;
            }
            let poly = clip_tri(tp, op, 1e-12 * diag * diag);
            if poly.len() < 3 {
                continue;
            }
            let mut ar = 0.0;
            for i in 1..poly.len() - 1 {
                ar += area2(poly[0], poly[i], poly[i + 1]).abs();
            }
            if ar < atol {
                continue;
            }
            for c in &poly {
                let lt = bary(*c, tp);
                let lo = bary(*c, op);
                for (f, yf) in inp.y.iter().enumerate() {
                    let mut dx = 0.0f64;
                    let mut dy = 0.0f64;
                    for j in 0..3 {
                        let a = yf[ot[j]];
                        let b = yf[t3[j]];
                        dx += lo[j] * a[0] as f64 - lt[j] * b[0] as f64;
                        dy += lo[j] * a[1] as f64 - lt[j] * b[1] as f64;
                    }
                    let e = dx.hypot(dy) * inp.s[f];
                    if e > worst {
                        worst = e;
                    }
                }
            }
        }
        worst
    };

    let mut collapses = 0;
    while let Some(c) = heap.pop() {
        let (u, v) = (c.u, c.v);
        if ver[u] != c.vu || ver[v] != c.vv || adj[u].is_empty() || adj[v].is_empty() {
            continue;
        }
        if !valid(&adj, &tris, u, v) {
            continue;
        }
        // certify
        let tuv = common_tris(&adj, u, v);
        let mut union: Vec<usize> = adj[u].iter().flat_map(|&t| cover[t].iter().copied()).collect();
        union.sort_unstable();
        union.dedup();
        let mut news: Vec<(usize, [usize; 3], f64)> = Vec::new();
        let mut ok = true;
        for &t in &adj[u] {
            if tuv.contains(&t) {
                continue;
            }
            let nt3 = tris[t].map(|x| if x == u { v } else { x });
            let e = tri_error(nt3, &union);
            if e > eps_px {
                ok = false;
                break;
            }
            news.push((t, nt3, e));
        }
        if !ok {
            continue;
        }
        // apply
        for &t in &tuv {
            alive[t] = false;
            for &x in &tris[t] {
                if x != u {
                    adj[x].retain(|&q| q != t);
                }
            }
        }
        let ring = neighbours(&adj, &tris, u);
        for (t, nt3, e) in news {
            tris[t] = nt3;
            cover[t] = union.clone();
            terr[t] = e;
            if !adj[v].contains(&t) {
                adj[v].push(t);
            }
        }
        adj[u].clear();
        adj[v].sort_unstable();
        let qu = quad[u].clone();
        quad[v].add(&qu);
        collapses += 1;
        // refresh candidates around the change
        let mut aff: BTreeSet<usize> = ring.iter().copied().collect();
        aff.insert(v);
        aff.extend(neighbours(&adj, &tris, v));
        for &x in &aff {
            ver[x] += 1;
        }
        for &x in &aff {
            for y in neighbours(&adj, &tris, x) {
                if valid(&adj, &tris, x, y) {
                    heap.push(Cand { cost: qcost(&quad, x, y), u: x, v: y, vu: ver[x], vv: ver[y] });
                }
                if !aff.contains(&y) && valid(&adj, &tris, y, x) {
                    heap.push(Cand { cost: qcost(&quad, y, x), u: y, v: x, vu: ver[y], vv: ver[x] });
                }
            }
        }
    }
    if collapses == 0 {
        return identity();
    }
    let mut used = vec![false; n];
    let mut referenced = vec![false; n];
    for tr in inp.tris {
        for &v in tr {
            referenced[v] = true;
        }
    }
    let mut out_tris = Vec::new();
    let mut bound = 0.0f64;
    for t in 0..nt {
        if alive[t] {
            out_tris.push(tris[t]);
            for &v in &tris[t] {
                used[v] = true;
            }
            bound = bound.max(terr[t]);
        }
    }
    let keep: Vec<usize> = (0..n).filter(|&v| used[v] || !referenced[v]).collect();
    QemOutput { keep, tris: out_tris, bound_px: bound, collapses }
}
