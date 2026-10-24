// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `autotrace` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// line mode: threshold, thinning, skeleton graph
// ---------------------------------------------------------------------------------------------

/// Otsu (1979) threshold of a 256-bin histogram.
pub fn otsu(hist: &[usize; 256]) -> u8 {
    let total: usize = hist.iter().sum();
    if total == 0 {
        return 128;
    }
    let sum_all: f64 = hist.iter().enumerate().map(|(i, &c)| i as f64 * c as f64).sum();
    let (mut wb, mut sb, mut best, mut thr) = (0.0, 0.0, -1.0, 128u8);
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for t in 0..256 {
        wb += hist[t] as f64;
        if wb == 0.0 {
            continue;
        }
        let wf = total as f64 - wb;
        if wf == 0.0 {
            break;
        }
        sb += t as f64 * hist[t] as f64;
        let (mb, mf) = (sb / wb, (sum_all - sb) / wf);
        let between = wb * wf * (mb - mf) * (mb - mf);
        if between > best {
            best = between;
            thr = t as u8;
        }
    }
    thr
}

/// Zhang-Suen (1984) thinning of a binary image (true = ink). Returns the skeleton.
pub fn zhang_suen(bin: &[bool], w: usize, h: usize) -> Vec<bool> {
    let mut img = bin.to_vec();
    let get = |im: &[bool], x: i32, y: i32| -> u8 { (x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && im[y as usize * w + x as usize]) as u8 };
    loop {
        let mut changed = false;
        for step in 0..2 {
            let mut del = Vec::new();
            for y in 0..h as i32 {
                for x in 0..w as i32 {
                    if get(&img, x, y) == 0 {
                        continue;
                    }
                    // P2..P9 clockwise from north
                    let n = [get(&img, x, y - 1), get(&img, x + 1, y - 1), get(&img, x + 1, y), get(&img, x + 1, y + 1), get(&img, x, y + 1), get(&img, x - 1, y + 1), get(&img, x - 1, y), get(&img, x - 1, y - 1)];
                    let b: u8 = n.iter().sum();
                    if !(2..=6).contains(&b) {
                        continue;
                    }
                    let a = (0..8).filter(|&i| n[i] == 0 && n[(i + 1) % 8] == 1).count();
                    if a != 1 {
                        continue;
                    }
                    let (p2, p4, p6, p8) = (n[0], n[2], n[4], n[6]);
                    let ok = if step == 0 { p2 * p4 * p6 == 0 && p4 * p6 * p8 == 0 } else { p2 * p4 * p8 == 0 && p2 * p6 * p8 == 0 };
                    if ok {
                        del.push(y as usize * w + x as usize);
                    }
                }
            }
            if !del.is_empty() {
                changed = true;
            }
            for i in del {
                img[i] = false;
            }
        }
        if !changed {
            break;
        }
    }
    // remove redundant L-corner pixels so 8-connected lines have two neighbours
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            if get(&img, x, y) == 0 {
                continue;
            }
            let dirs: [(i32, i32); 8] = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)];
            for q in 0..4 {
                // orthogonal neighbours a = dirs[2q], b = dirs[(2q+2)%8], diagonal between = dirs[2q+1]
                let (a, d, b) = (dirs[2 * q], dirs[2 * q + 1], dirs[(2 * q + 2) % 8]);
                if get(&img, x + a.0, y + a.1) == 1 && get(&img, x + b.0, y + b.1) == 1 {
                    let mut others = 0;
                    for (i, dd) in dirs.iter().enumerate() {
                        if i == 2 * q || i == (2 * q + 2) % 8 || (*dd == d) {
                            continue;
                        }
                        others += get(&img, x + dd.0, y + dd.1);
                    }
                    if others == 0 {
                        img[y as usize * w + x as usize] = false;
                        break;
                    }
                }
            }
        }
    }
    img
}

/// 3-4 chamfer distance transform to the nearest non-ink pixel, in pixels.
pub fn chamfer(bin: &[bool], w: usize, h: usize) -> Vec<f32> {
    let big = 1_000_000i32;
    let mut d: Vec<i32> = bin.iter().map(|&b| if b { big } else { 0 }).collect();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if d[i] == 0 {
                continue;
            }
            let mut v = d[i];
            if x > 0 {
                v = v.min(d[i - 1] + 3);
            } else {
                v = v.min(3);
            }
            if y > 0 {
                v = v.min(d[i - w] + 3);
                if x > 0 {
                    v = v.min(d[i - w - 1] + 4);
                }
                if x + 1 < w {
                    v = v.min(d[i - w + 1] + 4);
                }
            } else {
                v = v.min(3);
            }
            d[i] = v;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            if d[i] == 0 {
                continue;
            }
            let mut v = d[i];
            if x + 1 < w {
                v = v.min(d[i + 1] + 3);
            } else {
                v = v.min(3);
            }
            if y + 1 < h {
                v = v.min(d[i + w] + 3);
                if x + 1 < w {
                    v = v.min(d[i + w + 1] + 4);
                }
                if x > 0 {
                    v = v.min(d[i + w - 1] + 4);
                }
            } else {
                v = v.min(3);
            }
            d[i] = v;
        }
    }
    d.iter().map(|&v| v as f32 / 3.0).collect()
}

pub(super) struct RawChain {
    pub(super) pix: Vec<usize>,
    /// node ids at the ends (None = an isolated closed loop)
    pub(super) ends: Option<(usize, usize)>,
}

pub(super) fn skeleton_chains(sk: &[bool], w: usize, h: usize, prune: usize) -> Vec<(Vec<P>, bool)> {
    let nb = |i: usize| -> Vec<usize> {
        let (x, y) = ((i % w) as i32, (i / w) as i32);
        let mut v = Vec::new();
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if (dx, dy) == (0, 0) {
                    continue;
                }
                let (nx, ny) = (x + dx, y + dy);
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h && sk[ny as usize * w + nx as usize] {
                    v.push(ny as usize * w + nx as usize);
                }
            }
        }
        v
    };
    let deg: Vec<usize> = (0..w * h).map(|i| if sk[i] { nb(i).len() } else { 0 }).collect();
    // node pixels: degree != 2; cluster the junction pixels (degree >= 3) into single nodes
    let mut node_of = vec![usize::MAX; w * h];
    let mut nodes: Vec<Vec<usize>> = Vec::new();
    for i in 0..w * h {
        if !sk[i] || deg[i] == 2 || node_of[i] != usize::MAX {
            continue;
        }
        let id = nodes.len();
        let mut members = vec![i];
        node_of[i] = id;
        if deg[i] >= 3 {
            let mut stack = vec![i];
            while let Some(q) = stack.pop() {
                for r in nb(q) {
                    if deg[r] >= 3 && node_of[r] == usize::MAX {
                        node_of[r] = id;
                        members.push(r);
                        stack.push(r);
                    }
                }
            }
        }
        nodes.push(members);
    }
    let centre = |id: usize| -> P {
        let m = &nodes[id];
        let (sx, sy) = m.iter().fold((0.0, 0.0), |a, &q| (a.0 + (q % w) as f64 + 0.5, a.1 + (q / w) as f64 + 0.5));
        p(sx / m.len() as f64, sy / m.len() as f64)
    };
    let mut visited = vec![false; w * h];
    let mut chains: Vec<RawChain> = Vec::new();
    let mut direct: HashSet<(usize, usize)> = HashSet::new();
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for id in 0..nodes.len() {
        for &m in &nodes[id].clone() {
            for r in nb(m) {
                if node_of[r] == id {
                    continue;
                }
                if node_of[r] != usize::MAX {
                    // node to node directly
                    let key = (m.min(r), m.max(r));
                    if direct.insert(key) {
                        chains.push(RawChain { pix: vec![], ends: Some((id, node_of[r])) });
                    }
                    continue;
                }
                if visited[r] {
                    continue;
                }
                let mut pix = vec![];
                let (mut prev, mut cur) = (m, r);
                let end;
                loop {
                    visited[cur] = true;
                    pix.push(cur);
                    let next: Vec<usize> = nb(cur).into_iter().filter(|&q| q != prev && !(pix.contains(&q))).collect();
                    // prefer a node neighbour
                    if let Some(&nn) = next.iter().find(|&&q| node_of[q] != usize::MAX) {
                        end = node_of[nn];
                        break;
                    }
                    match next.iter().find(|&&q| !visited[q]) {
                        Some(&q) => {
                            prev = cur;
                            cur = q;
                        }
                        None => {
                            end = id;
                            break;
                        }
                    }
                }
                chains.push(RawChain { pix, ends: Some((id, end)) });
            }
        }
    }
    // isolated loops
    for i in 0..w * h {
        if sk[i] && !visited[i] && node_of[i] == usize::MAX {
            let mut pix = vec![i];
            visited[i] = true;
            let mut cur = i;
            loop {
                let next = nb(cur).into_iter().find(|&q| !visited[q] && node_of[q] == usize::MAX);
                match next {
                    Some(q) => {
                        visited[q] = true;
                        pix.push(q);
                        cur = q;
                    }
                    None => break,
                }
            }
            chains.push(RawChain { pix, ends: None });
        }
    }
    // prune spurs: a chain with a free tip (degree-1 node) shorter than `prune`, attached to a junction
    let node_deg: Vec<usize> = nodes.iter().map(|m| if m.len() == 1 && deg[m[0]] == 1 { 1 } else { 3 }).collect();
    let mut keep: Vec<bool> = vec![true; chains.len()];
    for (ci, c) in chains.iter().enumerate() {
        if let Some((a, b)) = c.ends {
            let tip = (node_deg[a] == 1) != (node_deg[b] == 1);
            if tip && c.pix.len() < prune {
                keep[ci] = false;
            }
        }
    }
    let chains: Vec<RawChain> = chains.into_iter().zip(&keep).filter(|(_, k)| **k).map(|(c, _)| c).collect();
    // incident count per node after pruning, then merge pass-through nodes (exactly two chain ends)
    type Poly = (Vec<P>, Option<(usize, usize)>);
    let mut polys: Vec<Poly> = chains
        .iter()
        .map(|c| {
            let mut v = Vec::new();
            if let Some((a, _)) = c.ends {
                v.push(centre(a));
            }
            v.extend(c.pix.iter().map(|&q| p((q % w) as f64 + 0.5, (q / w) as f64 + 0.5)));
            if let Some((_, b)) = c.ends {
                v.push(centre(b));
            }
            (v, c.ends)
        })
        .collect();
    loop {
        let mut inc: BTreeMap<usize, Vec<(usize, bool)>> = BTreeMap::new();
        for (ci, (_, e)) in polys.iter().enumerate() {
            if let Some((a, b)) = e {
                inc.entry(*a).or_default().push((ci, true));
                inc.entry(*b).or_default().push((ci, false));
            }
        }
        let mut merged = false;
        for (nid, list) in &inc {
            if node_deg[*nid] == 1 || list.len() != 2 || list[0].0 == list[1].0 {
                continue;
            }
            let (c1, s1) = list[0];
            let (c2, s2) = list[1];
            // orient chain 1 to end at the node, chain 2 to start at it
            let (mut v1, e1) = polys[c1].clone();
            let (mut v2, e2) = polys[c2].clone();
            let (e1, e2) = (e1.unwrap(), e2.unwrap());
            let far1 = if s1 { e1.1 } else { e1.0 };
            let far2 = if s2 { e2.1 } else { e2.0 };
            if s1 {
                v1.reverse();
            }
            if !s2 {
                v2.reverse();
            }
            v1.pop();
            v1.extend(v2);
            // far1 .. far2 (v1 starts at far1)
            let (hi, lo) = (c1.max(c2), c1.min(c2));
            polys.remove(hi);
            polys[lo] = (v1, Some((far1, far2)));
            merged = true;
            break;
        }
        if !merged {
            break;
        }
    }
    polys
        .into_iter()
        .map(|(v, e)| {
            let closed = e.is_none_or(|(a, b)| a == b && v.len() > 4);
            (v, closed)
        })
        .collect()
}

pub(super) fn trace_line(img: &Image, o: &TraceOptions) -> Result<TraceResult, String> {
    let (w, h) = (img.w, img.h);
    let lumv: Vec<f64> = (0..w * h)
        .map(|i| {
            let a = img.rgba[i * 4 + 3] as f64 / 255.0;
            let c = |k: usize| img.rgba[i * 4 + k] as f64 * a + 255.0 * (1.0 - a);
            0.2126 * c(0) + 0.7152 * c(1) + 0.0722 * c(2)
        })
        .collect();
    let mut hist = [0usize; 256];
    for &l in &lumv {
        hist[l.round().clamp(0.0, 255.0) as usize] += 1;
    }
    let thr = o.threshold.unwrap_or_else(|| otsu(&hist)) as f64;
    let mut ink: Vec<bool> = lumv.iter().map(|&l| if o.invert { l > thr } else { l <= thr }).collect();
    // drop tiny ink specks (8-connected)
    let min_a = o.min_area.ceil() as usize;
    if min_a > 1 {
        let mut seen = vec![false; w * h];
        for s in 0..w * h {
            if !ink[s] || seen[s] {
                continue;
            }
            let mut comp = vec![s];
            seen[s] = true;
            let mut i = 0;
            while i < comp.len() {
                let q = comp[i];
                i += 1;
                let (x, y) = ((q % w) as i32, (q / w) as i32);
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                            let r = ny as usize * w + nx as usize;
                            if ink[r] && !seen[r] {
                                seen[r] = true;
                                comp.push(r);
                            }
                        }
                    }
                }
            }
            if comp.len() < min_a {
                for q in comp {
                    ink[q] = false;
                }
            }
        }
    }
    let dt = chamfer(&ink, w, h);
    let sk = zhang_suen(&ink, w, h);
    let chains = skeleton_chains(&sk, w, h, o.prune);
    // ink colour: mean colour of ink pixels
    let (mut sr, mut sg, mut sb, mut sn) = (0u64, 0u64, 0u64, 0u64);
    for (i, _) in ink.iter().enumerate().take(w * h).filter(|(_, &k)| k) {
        sr += img.rgba[i * 4] as u64;
        sg += img.rgba[i * 4 + 1] as u64;
        sb += img.rgba[i * 4 + 2] as u64;
        sn += 1;
    }
    let ink_col = match (sr.checked_div(sn), sg.checked_div(sn), sb.checked_div(sn)) {
        (Some(r), Some(g), Some(b)) => [r as u8, g as u8, b as u8],
        _ => [0, 0, 0],
    };
    let mut strokes = Vec::new();
    for (pts, closed) in chains {
        if pts.len() < 2 {
            continue;
        }
        // stroke width from the distance transform along the chain
        let mut ws = 0.0f64;
        let mut cnt = 0.0;
        for q in &pts {
            let (x, y) = ((q.x - 0.5).round() as isize, (q.y - 0.5).round() as isize);
            if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                ws += (2.0 * dt[y as usize * w + x as usize] as f64 - 1.0).max(1.0);
                cnt += 1.0;
            }
        }
        let width = if cnt > 0.0 { (ws / cnt * 10.0).round() / 10.0 } else { 1.0 };
        let path = if o.fit == Fit::Bezier && pts.len() >= 3 {
            let bez = fit_chain(Chain { pts: pts.clone(), closed }, o);
            VPath::new("line", closed, anchors_from(&bez, closed))
        } else {
            let v: Vec<Vec2> = pts.iter().map(|q| q.v2()).collect();
            let s = if closed { douglas_peucker_closed(&v, o.tol) } else { douglas_peucker(&v, o.tol) };
            VPath::new("line", closed, s.into_iter().map(Anchor::corner).collect())
        };
        if path.anchors.len() >= 2 {
            strokes.push(Stroke { path, width: width as f32 });
        }
    }
    Ok(TraceResult { width: w, height: h, mode: Mode::Line, palette: vec![ink_col], regions: Vec::new(), strokes, stroke_color: ink_col })
}
