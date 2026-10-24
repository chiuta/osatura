// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `autotrace` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// label map clean-up and components
// ---------------------------------------------------------------------------------------------

/// 3x3 majority filter: a pixel changes only when another label has strictly more votes than its own.
pub fn majority_filter(lab: &mut [u16], w: usize, h: usize, passes: usize) {
    for _ in 0..passes {
        let src = lab.to_vec();
        for y in 0..h {
            for x in 0..w {
                let own = src[y * w + x];
                let mut votes: Vec<(u16, u8)> = Vec::with_capacity(9);
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let l = src[ny as usize * w + nx as usize];
                        match votes.iter_mut().find(|v| v.0 == l) {
                            Some(v) => v.1 += 1,
                            None => votes.push((l, 1)),
                        }
                    }
                }
                let own_n = votes.iter().find(|v| v.0 == own).map_or(0, |v| v.1);
                let mut best = (own_n, own);
                for &(l, n) in &votes {
                    if n > best.0 || (n == best.0 && n > own_n && l < best.1) {
                        best = (n, l);
                    }
                }
                lab[y * w + x] = best.1;
            }
        }
    }
}

/// 4-connected components of equal labels. Returns `(component id per pixel, pixel counts)`.
pub fn components4(lab: &[u16], w: usize, h: usize) -> (Vec<u32>, Vec<usize>) {
    let mut comp = vec![u32::MAX; w * h];
    let mut sizes = Vec::new();
    let mut stack = Vec::new();
    for s in 0..w * h {
        if comp[s] != u32::MAX {
            continue;
        }
        let id = sizes.len() as u32;
        let l = lab[s];
        comp[s] = id;
        stack.push(s);
        let mut n = 0;
        while let Some(p) = stack.pop() {
            n += 1;
            let (x, y) = (p % w, p / w);
            let mut visit = |q: usize| {
                if comp[q] == u32::MAX && lab[q] == l {
                    comp[q] = id;
                    stack.push(q);
                }
            };
            if x > 0 {
                visit(p - 1);
            }
            if x + 1 < w {
                visit(p + 1);
            }
            if y > 0 {
                visit(p - w);
            }
            if y + 1 < h {
                visit(p + w);
            }
        }
        sizes.push(n);
    }
    (comp, sizes)
}

/// Merge every component smaller than `min_area` pixels into the neighbouring label it shares the most
/// border with (several rounds, smallest first).
pub fn despeckle(lab: &mut [u16], w: usize, h: usize, min_area: usize) {
    if min_area <= 1 {
        return;
    }
    for _ in 0..8 {
        let (comp, sizes) = components4(lab, w, h);
        let mut small: Vec<usize> = (0..sizes.len()).filter(|&c| sizes[c] < min_area).collect();
        if small.is_empty() {
            return;
        }
        small.sort_by_key(|&c| (sizes[c], c));
        let mut pixels: Vec<Vec<usize>> = vec![Vec::new(); sizes.len()];
        for (p, &c) in comp.iter().enumerate() {
            if sizes[c as usize] < min_area {
                pixels[c as usize].push(p);
            }
        }
        let mut changed = false;
        for c in small {
            let own = lab[pixels[c][0]];
            let mut tally: BTreeMap<u16, usize> = BTreeMap::new();
            for &p in &pixels[c] {
                let (x, y) = (p % w, p / w);
                let mut look = |q: usize| {
                    if comp[q] != c as u32 || lab[q] != own {
                        // neighbour outside this component (its current label may have been merged this round)
                        if comp[q] != c as u32 {
                            *tally.entry(lab[q]).or_insert(0) += 1;
                        }
                    }
                };
                if x > 0 {
                    look(p - 1);
                }
                if x + 1 < w {
                    look(p + 1);
                }
                if y > 0 {
                    look(p - w);
                }
                if y + 1 < h {
                    look(p + w);
                }
            }
            tally.remove(&own);
            if let Some((&best, _)) = tally.iter().max_by_key(|(l, n)| (**n, std::cmp::Reverse(**l))) {
                for &p in &pixels[c] {
                    lab[p] = best;
                }
                changed = true;
            }
        }
        if !changed {
            return;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// border following on pixel cracks
// ---------------------------------------------------------------------------------------------

/// Closed loop of integer corner coordinates (x right, y down). Outer borders have positive signed
/// area (clockwise on screen), hole borders negative.
#[derive(Clone, Debug)]
pub struct Loop {
    pub pts: Vec<(i32, i32)>,
}

pub(super) fn signed_area_i(p: &[(i32, i32)]) -> i64 {
    let mut s = 0i64;
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        s += a.0 as i64 * b.1 as i64 - b.0 as i64 * a.1 as i64;
    }
    s
}

pub(super) const DX: [i32; 4] = [1, 0, -1, 0];

pub(super) const DY: [i32; 4] = [0, 1, 0, -1];

/// All border loops of the pixels where `inside(x, y)` is true inside the window `[0,w) x [0,h)`.
/// Foreground stays on the right of the walking direction; at pinch points the walk turns toward the
/// foreground, so foreground is 4-connected and background 8-connected.
pub fn border_loops(w: usize, h: usize, inside: &dyn Fn(usize, usize) -> bool) -> Vec<Loop> {
    let vw = w + 1;
    let mut edge = vec![0u8; vw * (h + 1)];
    let fg = |x: i32, y: i32| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && inside(x as usize, y as usize);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            if !fg(x, y) {
                continue;
            }
            if !fg(x, y - 1) {
                edge[y as usize * vw + x as usize] |= 1; // right, from (x,y)
            }
            if !fg(x + 1, y) {
                edge[y as usize * vw + x as usize + 1] |= 2; // down, from (x+1,y)
            }
            if !fg(x, y + 1) {
                edge[(y as usize + 1) * vw + x as usize + 1] |= 4; // left, from (x+1,y+1)
            }
            if !fg(x - 1, y) {
                edge[(y as usize + 1) * vw + x as usize] |= 8; // up, from (x,y+1)
            }
        }
    }
    let mut loops = Vec::new();
    for sy in 0..=h {
        for sx in 0..=w {
            for sd in 0..4 {
                if edge[sy * vw + sx] & (1 << sd) == 0 {
                    continue;
                }
                let (mut x, mut y, mut d) = (sx as i32, sy as i32, sd);
                let mut pts = Vec::new();
                let mut prev_d = usize::MAX;
                loop {
                    edge[y as usize * vw + x as usize] &= !(1 << d);
                    if d != prev_d {
                        pts.push((x, y));
                    }
                    prev_d = d;
                    x += DX[d];
                    y += DY[d];
                    let mut here = edge[y as usize * vw + x as usize];
                    if x == sx as i32 && y == sy as i32 {
                        here |= 1 << sd;
                    }
                    let mut next = None;
                    for nd in [(d + 1) % 4, d, (d + 3) % 4] {
                        if here & (1 << nd) != 0 {
                            next = Some(nd);
                            break;
                        }
                    }
                    match next {
                        Some(nd) => d = nd,
                        None => break,
                    }
                    if x == sx as i32 && y == sy as i32 && d == sd {
                        break;
                    }
                }
                // the first point may be a straight continuation of the closing edge
                if pts.len() >= 2 && prev_d == sd && pts.len() > 2 {
                    // closing edge has the same direction as the first: first point is collinear
                    pts.remove(0);
                }
                if pts.len() >= 4 {
                    loops.push(Loop { pts });
                }
            }
        }
    }
    loops
}
