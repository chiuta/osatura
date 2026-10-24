// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `gltf` (see the module docs in `mod.rs`).
use super::*;

pub(super) const MAX_ANIM_VALUES: usize = 4_000_000;

/// Most vertices of one polygon that `triangulate` accepts (more: the result is empty and the caller reports it).
pub const MAX_TRIANGULATE_POINTS: usize = 200_000;

/// Ear-clipping triangulation of a simple polygon (no holes). Returns index triples.
///
/// Linked-list ear clipping that goes on from where it stopped instead of starting over, and tests an ear only
/// against the reflex vertices (the only ones that can lie inside it): a convex polygon of n points costs O(n),
/// a general one O(n r) with r reflex vertices. (The first version rescanned all vertices for every ear, O(n^2)
/// and worse: 60000 anchors took 40 s.) Polygons above [`MAX_TRIANGULATE_POINTS`] points, or so tangled that the
/// reflex tests would exceed a fixed work budget, give an empty result.
pub fn triangulate(pts: &[Vec2]) -> Vec<[usize; 3]> {
    let n = pts.len();
    if !(3..=MAX_TRIANGULATE_POINTS).contains(&n) {
        return vec![];
    }
    let area: f32 = (0..n).map(|i| pts[i].x * pts[(i + 1) % n].y - pts[(i + 1) % n].x * pts[i].y).sum::<f32>() * 0.5;
    let order: Vec<usize> = if area >= 0.0 { (0..n).collect() } else { (0..n).rev().collect() };
    // f64: with f32 the cross product of neighbouring points of a fine polygon is rounding noise and half of its
    // convex corners would look reflex
    let cross = |a: Vec2, b: Vec2, c: Vec2| (b.x as f64 - a.x as f64) * (c.y as f64 - a.y as f64) - (b.y as f64 - a.y as f64) * (c.x as f64 - a.x as f64);
    let inside = |p: Vec2, a: Vec2, b: Vec2, c: Vec2| cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0;
    let at = |k: usize| pts[order[k]];
    let mut prev: Vec<usize> = (0..n).map(|k| (k + n - 1) % n).collect();
    let mut next: Vec<usize> = (0..n).map(|k| (k + 1) % n).collect();
    let convex = |k: usize, prev: &[usize], next: &[usize]| cross(at(prev[k]), at(k), at(next[k])) > 1e-12;
    let mut is_reflex: Vec<bool> = (0..n).map(|k| !convex(k, &prev, &next)).collect();
    // The reflex vertices sit in a uniform grid, so an ear is tested only against the few that are near it. (Even a
    // finely subdivided circle has many reflex corners in f32: coordinate rounding is larger than its sagitta.)
    let nr = is_reflex.iter().filter(|r| **r).count();
    let (mut lo, mut hi) = (Vec2::new(f32::MAX, f32::MAX), Vec2::new(f32::MIN, f32::MIN));
    for p in pts {
        lo = Vec2::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Vec2::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    let g = ((nr as f32).sqrt().ceil() as usize).clamp(1, 1024);
    let (w, h) = ((hi.x - lo.x).max(1e-9), (hi.y - lo.y).max(1e-9));
    let cell_of = |p: Vec2| -> (usize, usize) { ((((p.x - lo.x) / w * g as f32) as usize).min(g - 1), (((p.y - lo.y) / h * g as f32) as usize).min(g - 1)) };
    let mut cells: Vec<Vec<u32>> = vec![Vec::new(); g * g];
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for k in 0..n {
        if is_reflex[k] {
            let (cx, cy) = cell_of(at(k));
            cells[cy * g + cx].push(k as u32);
        }
    }
    let mut removed = vec![false; n];
    let mut out = Vec::with_capacity(n.saturating_sub(2));
    let (mut alive, mut cur, mut fails) = (n, 0usize, 0usize);
    let mut work = 0usize;
    while alive > 3 {
        let (a, c) = (prev[cur], next[cur]);
        let ear = !is_reflex[cur] && {
            let (pa, pb, pc) = (at(a), at(cur), at(c));
            let (x0, y0) = cell_of(Vec2::new(pa.x.min(pb.x).min(pc.x), pa.y.min(pb.y).min(pc.y)));
            let (x1, y1) = cell_of(Vec2::new(pa.x.max(pb.x).max(pc.x), pa.y.max(pb.y).max(pc.y)));
            let mut blocked = false;
            'scan: for cy in y0..=y1 {
                for cx in x0..=x1 {
                    let cell = &cells[cy * g + cx];
                    work += cell.len() + 1;
                    for &j in cell {
                        let j = j as usize;
                        if !removed[j] && is_reflex[j] && j != a && j != cur && j != c && inside(at(j), pa, pb, pc) {
                            blocked = true;
                            break 'scan;
                        }
                    }
                }
            }
            if work > 600_000_000 {
                return vec![]; // too complex to finish in reasonable time: the caller reports "cannot be triangulated"
            }
            !blocked
        };
        if ear {
            out.push([order[a], order[cur], order[c]]);
            removed[cur] = true;
            next[a] = c;
            prev[c] = a;
            alive -= 1;
            for k in [a, c] {
                if is_reflex[k] && convex(k, &prev, &next) {
                    is_reflex[k] = false;
                }
            }
            // skip the neighbour: clipping every other corner halves the polygon per pass (clipping one after the
            // other would grow a long fan whose triangles span ever more of the grid)
            cur = if alive > 3 { next[c] } else { c };
            fails = 0;
        } else {
            cur = next[cur];
            fails += 1;
            if fails > alive {
                // self-intersecting or degenerate input: fan what is left so we always terminate
                let mut rest = vec![cur];
                let mut k = next[cur];
                while k != cur {
                    rest.push(k);
                    k = next[k];
                }
                for w in 1..rest.len() - 1 {
                    out.push([order[rest[0]], order[rest[w]], order[rest[w + 1]]]);
                }
                return out;
            }
        }
    }
    if alive == 3 {
        out.push([order[prev[cur]], order[cur], order[next[cur]]]);
    }
    out
}

pub(super) struct Bin {
    pub(super) data: Vec<u8>,
    pub(super) views: Vec<Value>,
    pub(super) accessors: Vec<Value>,
}

impl Bin {
    /// A bare buffer view (used for embedded images). Returns its index.
    pub(super) fn view(&mut self, bytes: &[u8]) -> usize {
        while !self.data.len().is_multiple_of(4) {
            self.data.push(0);
        }
        let off = self.data.len();
        self.data.extend_from_slice(bytes);
        self.views.push(json!({"buffer": 0, "byteOffset": off, "byteLength": bytes.len()}));
        self.views.len() - 1
    }

    pub(super) fn add(&mut self, bytes: &[u8], component: u32, count: usize, ty: &str, min_max: Option<(Vec<f32>, Vec<f32>)>, target: Option<u32>) -> usize {
        while !self.data.len().is_multiple_of(4) {
            self.data.push(0);
        }
        let off = self.data.len();
        self.data.extend_from_slice(bytes);
        let mut view = json!({"buffer": 0, "byteOffset": off, "byteLength": bytes.len()});
        if let Some(t) = target {
            view["target"] = json!(t);
        }
        self.views.push(view);
        let mut acc = json!({"bufferView": self.views.len() - 1, "componentType": component, "count": count, "type": ty});
        if let Some((mn, mx)) = min_max {
            acc["min"] = json!(mn);
            acc["max"] = json!(mx);
        }
        self.accessors.push(acc);
        self.accessors.len() - 1
    }
}

pub(super) fn f32s(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

/// Local TRS (glTF, y-up) of `m` relative to `parent` (both stage space, y-down).
pub(super) fn trs(m: Affine2, parent: Option<Affine2>) -> ([f32; 3], [f32; 4], [f32; 3]) {
    let l = match parent.and_then(|p| p.inverse()) {
        Some(inv) => inv.mul(m),
        None => m,
    };
    let rot = l.b.atan2(l.a);
    let sx = l.a.hypot(l.b).max(1e-6);
    let sy = (l.a * l.d - l.b * l.c) / sx;
    let half = -rot * 0.5; // y flip negates the angle
    ([l.tx, -l.ty, 0.0], [0.0, 0.0, half.sin(), half.cos()], [sx, sy, 1.0])
}

pub(super) fn collect_shapes<'a>(ls: &'a [Layer], prefix: &str, out: &mut Vec<(&'a Layer, String)>) {
    for l in ls {
        let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
        if matches!(l.kind, LayerKind::Vector { .. } | LayerKind::Boolean { .. } | LayerKind::Image { .. }) {
            out.push((l, path.clone()));
        }
        if let LayerKind::Group { layers } | LayerKind::Switch { layers, .. } = &l.kind {
            collect_shapes(layers, &path, out);
        }
    }
}

pub(super) fn bin_ibm(bin: &mut Bin, ibm: &[f32], nb: usize) -> usize {
    bin.add(&f32s(ibm), 5126, nb, "MAT4", None, None)
}

// ---------------------------------------------------------------------------------------------
// small geometry helpers
// ---------------------------------------------------------------------------------------------

pub(super) fn cr(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

pub(super) fn nrm(d: Vec2) -> Vec2 {
    Vec2::new(-d.y, d.x)
}

pub(super) type Mix = ([u8; 4], [f32; 4]);

/// Top four influences, normalised. `mix` is `(bone index, weight)` with duplicates already merged.
pub(super) fn pack_mix(mut mix: Vec<(usize, f32)>) -> Mix {
    mix.sort_by(|x, y| y.1.total_cmp(&x.1));
    mix.truncate(4);
    if mix.iter().map(|m| m.1).sum::<f32>() <= 1e-6 {
        mix = vec![(0, 1.0)];
    }
    let sum: f32 = mix.iter().map(|m| m.1).sum();
    let (mut j, mut w) = ([0u8; 4], [0f32; 4]);
    for k in 0..4 {
        let (bi, bw) = mix.get(k).copied().unwrap_or((0, 0.0));
        // a zero weight must point at joint 0 (glTF validators insist on it)
        j[k] = if bw > 0.0 { bi as u8 } else { 0 };
        w[k] = bw / sum;
    }
    (j, w)
}

pub(super) fn add_mix(mix: &mut Vec<(usize, f32)>, bi: usize, w: f32) {
    match mix.iter_mut().find(|m| m.0 == bi) {
        Some(m) => m.1 += w,
        None => mix.push((bi, w)),
    }
}

/// Skin mix per flattened point of `wp` (the source path): weights interpolated along the segment.
pub(super) fn skin_table(wp: &VPath, tp: &[(Vec2, usize, f32)], bone_ix: &HashMap<&str, usize>) -> Vec<Mix> {
    tp.iter()
        .map(|&(_, seg, t)| {
            let a0 = &wp.anchors[seg % wp.anchors.len()];
            let a1 = &wp.anchors[(seg + 1) % wp.anchors.len()];
            let mut mix = Vec::new();
            for (a, k) in [(a0, 1.0 - t), (a1, t)] {
                for bw in &a.weights {
                    if let Some(&bi) = bone_ix.get(bw.bone.as_str()) {
                        add_mix(&mut mix, bi, bw.weight * k);
                    }
                }
            }
            pack_mix(mix)
        })
        .collect()
}

/// Triangle soup built by the stroker; `src` maps each vertex to the flattened path point it
/// derives from (for skinning).
#[derive(Default)]
pub(super) struct Soup {
    pub(super) pos: Vec<Vec2>,
    pub(super) src: Vec<usize>,
    pub(super) tris: Vec<[u32; 3]>,
    pub(super) note: Option<&'static str>,
}

impl Soup {
    pub(super) fn vert(&mut self, p: Vec2, src: usize) -> u32 {
        self.pos.push(p);
        self.src.push(src);
        (self.pos.len() - 1) as u32
    }
    /// Adds a triangle, wound counter-clockwise in layer space (positive cross).
    pub(super) fn tri(&mut self, a: u32, b: u32, c: u32) {
        let (pa, pb, pc) = (self.pos[a as usize], self.pos[b as usize], self.pos[c as usize]);
        let z = cr(pb - pa, pc - pa);
        if z.abs() < 1e-12 {
            return;
        }
        self.tris.push(if z > 0.0 { [a, b, c] } else { [a, c, b] });
    }
}

pub(super) type Pt = (Vec2, f32, usize); // position, arc length from the path start, source point index

const COS_SMOOTH: f32 = 0.906; // turns below ~25 degrees are treated as smooth (mitred ribbon)
const MITER_LIMIT: f32 = 4.0;

pub(super) const MAX_DASHES: f32 = 20_000.0;

pub(super) fn dash_runs(pts: &[Vec2], cum: &[f32], total: f32, closed: bool, dash: &[f32], offset: f32) -> Vec<(Vec<Pt>, bool)> {
    let n = pts.len();
    let ne = if closed { n } else { n - 1 };
    let cycle: f32 = dash.iter().sum();
    let mut idx = 0usize;
    let mut rem = dash[0];
    let mut off = offset.rem_euclid(cycle);
    while off > 0.0 {
        if off >= rem {
            off -= rem;
            idx = (idx + 1) % dash.len();
            rem = dash[idx];
        } else {
            rem -= off;
            off = 0.0;
        }
    }
    let mut runs: Vec<(Vec<Pt>, bool)> = Vec::new();
    let mut cur: Option<Vec<Pt>> = None;
    let finish = |cur: &mut Option<Vec<Pt>>, runs: &mut Vec<(Vec<Pt>, bool)>| {
        if let Some(mut r) = cur.take() {
            r.dedup_by(|a, b| a.0.distance(b.0) < 1e-4);
            if r.len() >= 2 {
                runs.push((r, false));
            }
        }
    };
    for e in 0..ne {
        let (a, b) = (e, (e + 1) % n);
        let len = pts[a].distance(pts[b]);
        if len < 1e-9 {
            continue;
        }
        let at = |u: f32| -> Pt { (pts[a].lerp(pts[b], u / len), (cum[a] + u).min(total), if u < len * 0.5 { a } else { b }) };
        let mut u = 0.0f32;
        while u < len - 1e-7 {
            let step = rem.min(len - u);
            if idx.is_multiple_of(2) {
                if cur.is_none() {
                    cur = Some(vec![at(u)]);
                }
                cur.as_mut().unwrap().push(at(u + step));
            }
            u += step;
            rem -= step;
            if rem <= 1e-7 {
                if idx.is_multiple_of(2) {
                    finish(&mut cur, &mut runs);
                }
                idx = (idx + 1) % dash.len();
                rem = dash[idx];
            }
        }
    }
    finish(&mut cur, &mut runs);
    runs
}

/// Triangulate a stroke of `style` along the polyline `pts` (layer space).
pub(super) fn build_stroke(pts: &[Vec2], closed: bool, style: &StrokeStyle) -> Option<Soup> {
    let n = pts.len();
    if n < 2 || (style.width.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !style.width.is_finite() {
        return None;
    }
    let mut cum = vec![0.0f32; n];
    for i in 1..n {
        cum[i] = cum[i - 1] + pts[i].distance(pts[i - 1]);
    }
    let total = (cum[n - 1] + if closed { pts[n - 1].distance(pts[0]) } else { 0.0 }).max(1e-6);
    let mut soup = Soup::default();
    let dash_ok = style.dash.len() >= 2 && style.dash.len().is_multiple_of(2) && style.dash.iter().all(|d| *d > 0.0 && d.is_finite());
    let cycle: f32 = style.dash.iter().sum();
    let mut runs: Vec<(Vec<Pt>, bool)> = Vec::new();
    if dash_ok && total / cycle * (style.dash.len() / 2) as f32 <= MAX_DASHES {
        runs = dash_runs(pts, &cum, total, closed, &style.dash, style.dash_offset);
    } else {
        if dash_ok {
            soup.note = Some("a stroke with more than 20000 dashes was exported solid");
        }
        runs.push(((0..n).map(|i| (pts[i], cum[i], i)).collect(), closed));
    }
    for (q, c) in runs {
        add_run(&mut soup, q, c, style, total);
    }
    Some(soup)
}

pub(super) fn add_run(out: &mut Soup, mut q: Vec<Pt>, mut wrap: bool, style: &StrokeStyle, total: f32) {
    q.dedup_by(|a, b| a.0.distance(b.0) < 1e-4);
    if wrap {
        if q.len() >= 2 && q[0].0.distance(q[q.len() - 1].0) < 1e-4 {
            q.pop();
        }
        if q.len() < 3 {
            wrap = false;
        }
    }
    if q.len() < 2 {
        return;
    }
    let dir_of = |q: &[Pt], a: usize, b: usize| (q[b].0 - q[a].0).normalized();
    if wrap {
        let m = q.len();
        let corner = |k: usize| dir_of(&q, (k + m - 1) % m, k).dot(dir_of(&q, k, (k + 1) % m)) < COS_SMOOTH;
        if let Some(k0) = (0..m).find(|&k| corner(k)) {
            q.rotate_left(k0);
        }
        q.push(q[0]);
    }
    let last = q.len() - 1;
    let dir = |a: usize, b: usize| dir_of(&q, a, b);
    let d_in = |j: usize| -> Option<Vec2> {
        if j > 0 {
            Some(dir(j - 1, j))
        } else if wrap {
            Some(dir(last - 1, last))
        } else {
            None
        }
    };
    let d_out = |j: usize| -> Option<Vec2> {
        if j < last {
            Some(dir(j, j + 1))
        } else if wrap {
            Some(dir(0, 1))
        } else {
            None
        }
    };
    let is_corner = |j: usize| matches!((d_in(j), d_out(j)), (Some(a), Some(b)) if a.dot(b) < COS_SMOOTH);
    let hw = |j: usize| 0.5 * style.width * profile_at(&style.width_profile, q[j].1 / total).max(0.0);
    let mut splits = vec![0usize];
    splits.extend((1..last).filter(|&j| is_corner(j)));
    splits.push(last);
    for w in splits.windows(2) {
        let (a, b) = (w[0], w[1]);
        let mut prev: Option<(u32, u32)> = None;
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for j in a..=b {
            let (m, k) = if is_corner(j) {
                match (j == a, d_in(j), d_out(j)) {
                    (true, _, Some(d)) => (nrm(d), 1.0),
                    (_, Some(d), _) => (nrm(d), 1.0),
                    _ => continue,
                }
            } else {
                match (d_in(j), d_out(j)) {
                    (Some(di), Some(dout)) => {
                        let mut m = (nrm(di) + nrm(dout)).normalized();
                        if m == Vec2::ZERO {
                            m = nrm(dout);
                        }
                        (m, 1.0 / m.dot(nrm(dout)).max(0.5))
                    }
                    (None, Some(d)) | (Some(d), None) => (nrm(d), 1.0),
                    _ => continue,
                }
            };
            let off = m * (hw(j) * k);
            let l = out.vert(q[j].0 + off, q[j].2);
            let r = out.vert(q[j].0 - off, q[j].2);
            if let Some((pl, pr)) = prev {
                out.tri(pl, pr, l);
                out.tri(pr, r, l);
            }
            prev = Some((l, r));
        }
    }
    // joins at the corners
    let first_join = if wrap { 0 } else { 1 };
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for j in first_join..last {
        if !is_corner(j) {
            continue;
        }
        let (Some(di), Some(dout)) = (d_in(j), d_out(j)) else { continue };
        let (v, h, src) = (q[j].0, hw(j), q[j].2);
        let sigma = if cr(di, dout) > 0.0 { -1.0 } else { 1.0 };
        let (n0, n1) = (nrm(di) * sigma, nrm(dout) * sigma);
        let vc = out.vert(v, src);
        let va = out.vert(v + n0 * h, src);
        let vb = out.vert(v + n1 * h, src);
        match style.join {
            Join::Bevel => out.tri(vc, va, vb),
            Join::Miter => {
                let denom = 1.0 + n0.dot(n1);
                if denom > 1e-6 && (2.0 / denom).sqrt() <= MITER_LIMIT {
                    let vt = out.vert(v + (n0 + n1) * (h / denom), src);
                    out.tri(vc, va, vt);
                    out.tri(vc, vt, vb);
                } else {
                    out.tri(vc, va, vb);
                }
            }
            Join::Round => {
                let a0 = n0.y.atan2(n0.x);
                let delta = cr(n0, n1).atan2(n0.dot(n1));
                let steps = ((delta.abs() / (std::f32::consts::PI / 12.0)).ceil() as usize).max(1);
                let mut prev = va;
                for s in 1..=steps {
                    let cur = if s == steps { vb } else { out.vert(v + Vec2::from_angle(a0 + delta * s as f32 / steps as f32) * h, src) };
                    out.tri(vc, prev, cur);
                    prev = cur;
                }
            }
        }
    }
    // caps
    if !wrap {
        for (j, d) in [(0usize, d_out(0).map(|d| -d)), (last, d_in(last))] {
            let Some(d) = d else { continue };
            let (v, h, src) = (q[j].0, hw(j), q[j].2);
            let n = nrm(d);
            match style.cap {
                Cap::Butt => {}
                Cap::Square => {
                    let (a, b) = (out.vert(v + n * h, src), out.vert(v - n * h, src));
                    let (c, e) = (out.vert(v + n * h + d * h, src), out.vert(v - n * h + d * h, src));
                    out.tri(a, b, c);
                    out.tri(b, e, c);
                }
                Cap::Round => {
                    let vc = out.vert(v, src);
                    let a0 = n.y.atan2(n.x);
                    let steps = 12;
                    let mut prev = out.vert(v + n * h, src);
                    for s in 1..=steps {
                        let cur = out.vert(v + Vec2::from_angle(a0 - std::f32::consts::PI * s as f32 / steps as f32) * h, src);
                        out.tri(vc, prev, cur);
                        prev = cur;
                    }
                }
            }
        }
    }
}
