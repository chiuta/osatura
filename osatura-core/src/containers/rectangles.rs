// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `containers` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// Rectangles
// ---------------------------------------------------------------------------------------------

pub(super) fn pin_axis(lo: Option<f32>, hi: Option<f32>, size: Option<f32>, total: f32) -> (f32, f32) {
    match (lo, hi, size) {
        (Some(l), Some(h), _) => (l, (total - l - h).max(0.0)),
        (Some(l), None, Some(s)) => (l, s),
        (None, Some(h), Some(s)) => (total - h - s, s),
        (None, None, Some(s)) => ((total - s) * 0.5, s),
        (Some(l), None, None) => (l, (total - l).max(0.0)),
        (None, Some(h), None) => (0.0, (total - h).max(0.0)),
        (None, None, None) => (0.0, total),
    }
}

/// Resolve a rule against a canvas; `hug` is the content size used by `Hug`.
pub fn resolve_rule(rule: &RectRule, cw: f32, ch: f32, hug: [f32; 2]) -> [f32; 4] {
    match rule {
        RectRule::Fixed { x, y, w, h } => [*x, *y, *w, *h],
        RectRule::Pin { left, top, right, bottom, width, height } => {
            let (x, w) = pin_axis(*left, *right, *width, cw);
            let (y, h) = pin_axis(*top, *bottom, *height, ch);
            [x, y, w, h]
        }
        RectRule::Aspect { ratio, mode, margin, ax, ay } => {
            let (aw, ah) = ((cw - 2.0 * margin).max(0.0), (ch - 2.0 * margin).max(0.0));
            let wider = ah > 0.0 && aw / ah > *ratio;
            let (w, h) = match (mode, wider) {
                (AspectMode::Fit, true) | (AspectMode::Fill, false) => (ah * ratio, ah),
                _ => (aw, aw / ratio),
            };
            [margin + (aw - w) * ax, margin + (ah - h) * ay, w, h]
        }
        RectRule::Hug { x, y } => [*x, *y, hug[0], hug[1]],
    }
}

// ---------------------------------------------------------------------------------------------
// Solver
// ---------------------------------------------------------------------------------------------

pub(super) struct Solver<'a> {
    pub(super) p: &'a Project,
    pub(super) ids: HashMap<&'a str, usize>,
    /// Scroll that owns a container as its content.
    pub(super) scroll_of: HashMap<&'a str, usize>,
    pub(super) sol: Solution,
}

/// A measured member.
pub(super) struct Prep<'a> {
    pub(super) m: &'a Member,
    pub(super) nat: [f32; 2],
    pub(super) bounds: Option<(Vec2, Vec2)>,
}

pub(super) fn dirx(d: Dir) -> usize {
    if d == Dir::Row { 0 } else { 1 }
}

/// Share `avail` over items: grow when there is room, shrink (weighted by basis) when there is
/// not; members at their min / max are frozen and the remainder is re-shared.
pub(super) fn distribute(basis: &[f32], grow: &[f32], shrink: &[f32], lo: &[f32], hi: &[f32], avail: f32) -> Vec<f32> {
    let n = basis.len();
    let mut size: Vec<f32> = basis.to_vec();
    let total: f32 = basis.iter().sum();
    let growing = avail > total;
    if !growing && avail >= total {
        return size;
    }
    let weight: Vec<f32> = (0..n).map(|i| if growing { grow[i] } else { shrink[i] * basis[i] }).collect();
    let mut frozen: Vec<bool> = weight.iter().map(|w| *w <= 0.0).collect();
    for _ in 0..=n {
        let used: f32 = (0..n).map(|i| if frozen[i] { size[i] } else { basis[i] }).sum();
        let free = avail - used;
        let wsum: f32 = (0..n).filter(|&i| !frozen[i]).map(|i| weight[i]).sum();
        if wsum <= 0.0 || free.abs() < 1e-6 {
            for i in 0..n {
                if !frozen[i] {
                    size[i] = basis[i];
                }
            }
            break;
        }
        let mut violated = false;
        let mut tent = vec![0.0f32; n];
        for i in 0..n {
            if frozen[i] {
                continue;
            }
            let t = basis[i] + free * weight[i] / wsum;
            let c = t.max(lo[i]).min(hi[i]);
            tent[i] = t;
            if (c - t).abs() > 1e-6 {
                violated = true;
            }
        }
        if !violated {
            for i in 0..n {
                if !frozen[i] {
                    size[i] = tent[i];
                }
            }
            break;
        }
        // Freeze every violator at its clamped value and go round again.
        for i in 0..n {
            if !frozen[i] {
                let c = tent[i].max(lo[i]).min(hi[i]);
                if (c - tent[i]).abs() > 1e-6 {
                    size[i] = c;
                    frozen[i] = true;
                }
            }
        }
    }
    size
}

impl<'a> Solver<'a> {
    pub(super) fn new(p: &'a Project) -> Solver<'a> {
        let ids = p.containers.iter().enumerate().map(|(i, c)| (c.id.as_str(), i)).collect();
        let scroll_of = p.scrolls.iter().enumerate().map(|(i, s)| (s.content.as_str(), i)).collect();
        Solver { p, ids, scroll_of, sol: Solution::default() }
    }

    pub(super) fn prep(&self, c: &'a Container, depth: usize) -> Vec<Prep<'a>> {
        c.members
            .iter()
            .map(|m| {
                let (mut nat, mut bounds) = ([0.0f32; 2], None);
                if let Some(path) = &m.layer {
                    if let Some(l) = self.p.find_layer(path) {
                        bounds = layer_bounds(l);
                        match bounds {
                            Some((lo, hi)) => nat = [hi.x - lo.x, hi.y - lo.y],
                            None => bounds = None,
                        }
                    }
                } else if let Some(id) = &m.container {
                    nat = self.measure(self.ids[id.as_str()], [None, None], depth + 1);
                }
                // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
                #[allow(clippy::needless_range_loop)]
                for a in 0..2 {
                    if let Some(v) = m.fixed(a) {
                        nat[a] = v;
                    }
                }
                Prep { m, nat, bounds }
            })
            .collect()
    }

    /// Break members into lines along the main axis.
    pub(super) fn lines(&self, c: &Container, preps: &[Prep], inner_main: Option<f32>) -> Vec<Vec<usize>> {
        let m = dirx(c.axis);
        let limit = match (c.wrap, inner_main) {
            (Wrap::Wrap, Some(l)) => l,
            _ => return if preps.is_empty() { vec![] } else { vec![(0..preps.len()).collect()] },
        };
        let mut lines: Vec<Vec<usize>> = Vec::new();
        let mut cur: Vec<usize> = Vec::new();
        let mut sum = 0.0f32;
        for (i, pr) in preps.iter().enumerate() {
            let b = pr.m.clamp(m, pr.nat[m]);
            let need = b + if cur.is_empty() { 0.0 } else { c.gap };
            if !cur.is_empty() && sum + need > limit + 1e-4 {
                lines.push(std::mem::take(&mut cur));
                sum = 0.0;
            }
            sum += b + if cur.is_empty() { 0.0 } else { c.gap };
            cur.push(i);
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        lines
    }

    /// Intrinsic size (including padding) of a container; `avail` limits the wrapping width.
    pub(super) fn measure(&self, ci: usize, avail: [Option<f32>; 2], depth: usize) -> [f32; 2] {
        if depth > MAX_DEPTH {
            return [0.0, 0.0];
        }
        let c = &self.p.containers[ci];
        let m = dirx(c.axis);
        let x = 1 - m;
        let pad_main = if m == 0 { c.padding[0] + c.padding[2] } else { c.padding[1] + c.padding[3] };
        let pad_cross = if m == 0 { c.padding[1] + c.padding[3] } else { c.padding[0] + c.padding[2] };
        let preps = self.prep(c, depth);
        let inner_main = avail[m].map(|a| (a - pad_main).max(0.0));
        let lines = self.lines(c, &preps, inner_main);
        let cross_gap = c.cross_gap.unwrap_or(c.gap);
        let (mut main_ext, mut cross_ext) = (0.0f32, 0.0f32);
        for (j, line) in lines.iter().enumerate() {
            let mut sum = 0.0;
            let mut cross = 0.0f32;
            for (k, &i) in line.iter().enumerate() {
                sum += preps[i].m.clamp(m, preps[i].nat[m]) + if k > 0 { c.gap } else { 0.0 };
                cross = cross.max(preps[i].m.clamp(x, preps[i].nat[x]));
            }
            main_ext = main_ext.max(sum);
            cross_ext += cross + if j > 0 { cross_gap } else { 0.0 };
        }
        let mut out = [0.0; 2];
        out[m] = main_ext + pad_main;
        out[x] = cross_ext + pad_cross;
        out
    }

    pub(super) fn place(&mut self, path: &str, pr: &Prep, slot: [f32; 4]) -> Result<(), String> {
        let l = self.p.find_layer(path).ok_or_else(|| format!("unknown layer '{path}'"))?;
        let pl = placement(path, l, pr.bounds, slot, pr.m.resize)?;
        self.sol.layers.push(pl);
        Ok(())
    }

    pub(super) fn layout(&mut self, ci: usize, rect: [f32; 4], depth: usize) -> Result<(), String> {
        if depth > MAX_DEPTH {
            return Err("container nesting is too deep".into());
        }
        let p = self.p;
        let c: &'a Container = &p.containers[ci];
        self.sol.rects.push((c.id.clone(), rect));
        let m = dirx(c.axis);
        let x = 1 - m;
        let [pl, pt, pr_, pb] = c.padding;
        let inner = [(rect[2] - pl - pr_).max(0.0), (rect[3] - pt - pb).max(0.0)];
        let origin = [rect[0] + pl, rect[1] + pt];
        let preps = self.prep(c, depth);
        let lines = self.lines(c, &preps, Some(inner[m]));
        let cross_gap = c.cross_gap.unwrap_or(c.gap);
        let single = lines.len() == 1 && c.wrap == Wrap::None;
        let mut cross_cursor = 0.0f32;
        for line in &lines {
            let n = line.len();
            let line_cross = if single {
                inner[x]
            } else {
                line.iter().map(|&i| preps[i].m.clamp(x, preps[i].nat[x])).fold(0.0f32, f32::max)
            };
            let basis: Vec<f32> = line.iter().map(|&i| preps[i].m.clamp(m, preps[i].nat[m])).collect();
            let grow: Vec<f32> = line.iter().map(|&i| preps[i].m.grow).collect();
            let shrink: Vec<f32> = line.iter().map(|&i| preps[i].m.shrink).collect();
            let lo: Vec<f32> = line.iter().map(|&i| preps[i].m.limits(m).0).collect();
            let hi: Vec<f32> = line.iter().map(|&i| preps[i].m.limits(m).1).collect();
            let avail = inner[m] - c.gap * (n.saturating_sub(1)) as f32;
            let sizes = distribute(&basis, &grow, &shrink, &lo, &hi, avail);
            let free = (avail - sizes.iter().sum::<f32>()).max(0.0);
            let (start, extra) = match c.justify {
                Justify::Start => (0.0, 0.0),
                Justify::Center => (free * 0.5, 0.0),
                Justify::End => (free, 0.0),
                Justify::SpaceBetween if n > 1 => (0.0, free / (n - 1) as f32),
                Justify::SpaceBetween => (0.0, 0.0),
                Justify::SpaceAround => (free / n as f32 * 0.5, free / n as f32),
            };
            let mut cursor = start;
            for (k, &i) in line.iter().enumerate() {
                let pr = &preps[i];
                let al = pr.m.align.unwrap_or(c.align);
                let free_cross = pr.m.fixed(x).is_none();
                let resizable = pr.m.resize || pr.m.container.is_some() || pr.m.spacer;
                let cs = if al == Align::Stretch && free_cross && resizable {
                    pr.m.clamp(x, line_cross)
                } else {
                    pr.m.clamp(x, pr.nat[x])
                };
                let coff = match al {
                    Align::Start | Align::Stretch => 0.0,
                    Align::Center => (line_cross - cs) * 0.5,
                    Align::End => line_cross - cs,
                };
                let mut slot = [0.0f32; 4];
                slot[m] = origin[m] + cursor;
                slot[x] = origin[x] + cross_cursor + coff;
                slot[2 + m] = sizes[k];
                slot[2 + x] = cs;
                cursor += sizes[k] + c.gap + extra;
                if let Some(path) = &pr.m.layer {
                    self.place(path, pr, slot)?;
                } else if let Some(id) = &pr.m.container {
                    let child = self.ids[id.as_str()];
                    self.layout(child, slot, depth + 1)?;
                }
            }
            cross_cursor += line_cross + cross_gap;
        }
        Ok(())
    }
}

/// Offset / scale of a layer so that its rest bounding box lands in `slot` (resize) or at the
/// slot origin (no resize).
pub(super) fn placement(path: &str, l: &Layer, bounds: Option<(Vec2, Vec2)>, slot: [f32; 4], resize: bool) -> Result<Placement, String> {
    let t = &l.transform;
    match bounds {
        Some((lo, hi)) => {
            if resize {
                let (cl, ch) = content_bounds(l).ok_or_else(|| format!("layer '{path}' has no measurable geometry"))?;
                let (w, h) = (hi.x - lo.x, hi.y - lo.y);
                let kx = if w > 1e-6 { (slot[2] / w).max(1e-4) } else { 1.0 };
                let ky = if h > 1e-6 { (slot[3] / h).max(1e-4) } else { 1.0 };
                let (sx, sy) = (t.scale.x * kx, t.scale.y * ky);
                let minx = (sx * (cl.x - t.anchor.x)).min(sx * (ch.x - t.anchor.x));
                let miny = (sy * (cl.y - t.anchor.y)).min(sy * (ch.y - t.anchor.y));
                Ok(Placement { layer: path.into(), dx: slot[0] - minx - t.position.x, dy: slot[1] - miny - t.position.y, kx, ky })
            } else {
                Ok(Placement { layer: path.into(), dx: slot[0] - lo.x, dy: slot[1] - lo.y, kx: 1.0, ky: 1.0 })
            }
        }
        None => Ok(Placement { layer: path.into(), dx: slot[0] - t.position.x, dy: slot[1] - t.position.y, kx: 1.0, ky: 1.0 }),
    }
}

/// Offset actually used for a raw request: clamped, or rubber-banded past the ends.
pub fn effective_offset(raw: f32, max: f32, elastic: Option<&Elastic>) -> f32 {
    let max = max.max(0.0);
    match elastic {
        None => raw.max(0.0).min(max),
        Some(e) => {
            let rub = |o: f32| e.limit * (1.0 - (-o / e.limit).exp());
            if raw < 0.0 {
                -rub(-raw)
            } else if raw > max {
                max + rub(raw - max)
            } else {
                raw
            }
        }
    }
}

/// Inverse of [`effective_offset`] on the displayed side: the raw request that displays as `shown`
/// (a rubber-banded overscroll is only ever shown below `limit`). Used when a displayed position
/// (for example from a [`ScrollRuntime`]) must be written back as a scroll's requested offset.
pub fn raw_offset_for(shown: f32, max: f32, elastic: Option<&Elastic>) -> f32 {
    let max = max.max(0.0);
    let inv = |r: f32, e: &Elastic| -e.limit * (1.0 - (r / e.limit).min(0.999_999)).ln();
    match elastic {
        None => shown.max(0.0).min(max),
        Some(e) if shown < 0.0 => -inv(-shown, e),
        Some(e) if shown > max => max + inv(shown - max, e),
        Some(_) => shown,
    }
}

/// Solve every container and scroll for a canvas of `w` x `h`.
pub fn solve(p: &Project, w: f32, h: f32) -> Result<Solution, String> {
    let errs = validate(p);
    if !errs.is_empty() {
        return Err(errs.join("\n"));
    }
    if !(w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0) {
        return Err("canvas size must be finite and > 0".into());
    }
    let mut s = Solver::new(p);
    s.sol.canvas = (w, h);
    let members: HashSet<&str> = p.containers.iter().flat_map(|c| c.members.iter().filter_map(|m| m.container.as_deref())).collect();
    let mut infos: Vec<(usize, ScrollInfo)> = Vec::new();
    for (ci, c) in p.containers.iter().enumerate() {
        if members.contains(c.id.as_str()) {
            continue;
        }
        if let Some(&si) = s.scroll_of.get(c.id.as_str()) {
            let sc = &p.scrolls[si];
            let vp = resolve_rule(&sc.viewport, w, h, [0.0, 0.0]);
            let avail = [if sc.scrolls(0) { None } else { Some(vp[2]) }, if sc.scrolls(1) { None } else { Some(vp[3]) }];
            let intr = s.measure(ci, avail, 0);
            let mut size = [vp[2], vp[3]];
            let mut max = [0.0f32; 2];
            let mut off = [0.0f32; 2];
            for a in 0..2 {
                if sc.scrolls(a) {
                    size[a] = intr[a].max(vp[2 + a]);
                    max[a] = (size[a] - vp[2 + a]).max(0.0);
                    off[a] = effective_offset(sc.offset[a], max[a], sc.elastic.as_ref());
                }
            }
            let rect = [vp[0] - off[0], vp[1] - off[1], size[0], size[1]];
            s.layout(ci, rect, 0)?;
            infos.push((si, ScrollInfo { id: sc.id.clone(), viewport: vp, content: rect, max, offset: off }));
            if let Some(cl) = &sc.clip_layer {
                let l = p.find_layer(cl).ok_or_else(|| format!("unknown clip layer '{cl}'"))?;
                let pl = placement(cl, l, layer_bounds(l), vp, true)?;
                s.sol.layers.push(pl);
            }
        } else {
            let hug = s.measure(ci, [None, None], 0);
            let rect = resolve_rule(&c.rect, w, h, hug);
            s.layout(ci, rect, 0)?;
        }
    }
    infos.sort_by_key(|(i, _)| *i);
    s.sol.scrolls = infos.into_iter().map(|(_, i)| i).collect();
    for pl in &s.sol.layers {
        if ![pl.dx, pl.dy, pl.kx, pl.ky].iter().all(|v| v.is_finite()) {
            return Err(format!("non-finite value in the layout of layer '{}' (a size, padding or gap is too large)", pl.layer));
        }
    }
    for sc in &s.sol.scrolls {
        if ![sc.viewport.as_slice(), sc.content.as_slice(), sc.max.as_slice(), sc.offset.as_slice()].concat().iter().all(|v| v.is_finite()) {
            return Err(format!("non-finite value in the layout of scroll '{}' (a size, padding or gap is too large)", sc.id));
        }
    }
    Ok(s.sol)
}

/// Solve for the project's own canvas size.
pub fn solve_project(p: &Project) -> Result<Solution, String> {
    solve(p, p.width, p.height)
}

/// Layout offsets per engine node id (`[dx, dy, kx, ky]`), for the project's own canvas.
pub(crate) fn node_offsets(engine: &crate::engine::Engine) -> Result<Vec<Option<[f32; 4]>>, String> {
    let sol = solve_project(&engine.project)?;
    let mut out = vec![None; engine.node_count()];
    for pl in &sol.layers {
        let id = engine.node_index(&pl.layer).ok_or_else(|| format!("unknown layer '{}'", pl.layer))?;
        out[id] = Some([pl.dx, pl.dy, pl.kx, pl.ky]);
    }
    Ok(out)
}

/// Add the clip mask to every content layer of a scroll that names a `clip_layer`.
/// Call after [`validate`] succeeded.
pub fn apply_clip_masks(p: &mut Project) {
    let ids: HashMap<&str, usize> = p.containers.iter().enumerate().map(|(i, c)| (c.id.as_str(), i)).collect();
    let mut todo: Vec<(String, String)> = Vec::new();
    for s in &p.scrolls {
        if let Some(cl) = &s.clip_layer {
            let mut v = Vec::new();
            collect_layers(p, &ids, &s.content, &mut v, 0);
            todo.extend(v.into_iter().map(|l| (l.to_string(), cl.clone())));
        }
    }
    for (path, clip) in todo {
        if let Some(l) = p.find_layer_mut(&path) {
            l.mask = Some(MaskRef { layer: clip, invert: false });
        }
    }
}
