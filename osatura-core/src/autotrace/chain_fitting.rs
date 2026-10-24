// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `autotrace` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// chain fitting (closed loops and open runs)
// ---------------------------------------------------------------------------------------------

pub(super) struct Chain {
    pub(super) pts: Vec<P>,
    pub(super) closed: bool,
}

impl Chain {
    pub(super) fn n(&self) -> usize {
        self.pts.len()
    }
    pub(super) fn at(&self, i: isize) -> P {
        let n = self.n() as isize;
        if self.closed {
            self.pts[i.rem_euclid(n) as usize]
        } else {
            self.pts[i.clamp(0, n - 1) as usize]
        }
    }
}

pub(super) fn resample_closed(poly: &[(i32, i32)]) -> Vec<P> {
    let n = poly.len();
    let per: f64 = (0..n).map(|i| ((poly[(i + 1) % n].0 - poly[i].0) as f64).hypot((poly[(i + 1) % n].1 - poly[i].1) as f64)).sum();
    let step = 1.0f64.min(per / 12.0);
    let mut out = Vec::new();
    for i in 0..n {
        let a = p(poly[i].0 as f64, poly[i].1 as f64);
        let b = p(poly[(i + 1) % n].0 as f64, poly[(i + 1) % n].1 as f64);
        let len = (b - a).len();
        let m = (len / step).ceil().max(1.0) as usize;
        for j in 0..m {
            out.push(a + (b - a) * (j as f64 / m as f64));
        }
    }
    out
}

pub(super) fn detect_corners(c: &Chain, k: usize, angle_deg: f64) -> Vec<bool> {
    let n = c.n();
    let mut dev = vec![0.0f64; n];
    let range: Vec<usize> = if c.closed { (0..n).collect() } else { (k..n.saturating_sub(k)).collect() };
    for &i in &range {
        let u = c.at(i as isize) - c.at(i as isize - k as isize);
        let v = c.at(i as isize + k as isize) - c.at(i as isize);
        let (lu, lv) = (u.len(), v.len());
        if lu < 1e-9 || lv < 1e-9 {
            continue;
        }
        dev[i] = (u.dot(v) / (lu * lv)).clamp(-1.0, 1.0).acos().to_degrees();
    }
    let mut flag = vec![false; n];
    for &i in &range {
        if dev[i] < angle_deg {
            continue;
        }
        let mut is_max = true;
        for off in 1..=k as isize {
            for j in [i as isize - off, i as isize + off] {
                let jj = if c.closed { j.rem_euclid(n as isize) as usize } else if j < 0 || j >= n as isize { continue } else { j as usize };
                if dev[jj] > dev[i] || (dev[jj] == dev[i] && jj < i && (c.closed || true)) {
                    is_max = false;
                }
            }
        }
        flag[i] = is_max;
    }
    flag
}

pub(super) fn smooth_chain(c: &mut Chain, pinned: &[bool], passes: usize) {
    for _ in 0..passes {
        let src: Vec<P> = c.pts.clone();
        let cc = Chain { pts: src, closed: c.closed };
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for i in 0..c.n() {
            let pin = pinned[i] || (!c.closed && (i == 0 || i == c.n() - 1));
            if !pin {
                c.pts[i] = (cc.at(i as isize - 1) + cc.at(i as isize) * 2.0 + cc.at(i as isize + 1)) * 0.25;
            }
        }
    }
}

/// Fit a chain with Bezier segments. Returns the segments in order; for a closed chain the last
/// segment ends where the first starts.
pub(super) fn fit_chain(mut c: Chain, o: &TraceOptions) -> Vec<Bez> {
    let n = c.n();
    if n < 2 {
        return Vec::new();
    }
    let k = o.corner_window.min((n / 6).max(2));
    let corner = if n >= 8 { detect_corners(&c, k, o.corner_angle as f64) } else { vec![true; n] };
    smooth_chain(&mut c, &corner, o.smooth);
    let err2 = (o.tol as f64) * (o.tol as f64);
    // cuts: indices where sections start/end. `hard[i]` = corner (one-sided tangents).
    let mut cuts: Vec<(usize, bool)> = Vec::new();
    if c.closed {
        for (i, _) in corner.iter().enumerate().take(n).filter(|(_, &k)| k) {
            cuts.push((i, true));
        }
        if cuts.is_empty() {
            let parts = if n >= 16 { 4 } else { 2 };
            for q in 0..parts {
                cuts.push((q * n / parts, false));
            }
        } else if cuts.len() == 1 && n >= 8 {
            cuts.push(((cuts[0].0 + n / 2) % n, false));
            cuts.sort();
        }
    } else {
        cuts.push((0, true));
        for (i, _) in corner.iter().enumerate().take(n - 1).skip(1).filter(|(_, &k)| k) {
            cuts.push((i, true));
        }
        cuts.push((n - 1, true));
    }
    let sections = if c.closed { cuts.len() } else { cuts.len() - 1 };
    let mut out = Vec::new();
    for s in 0..sections {
        let (a, ha) = cuts[s];
        let (b0, hb) = cuts[(s + 1) % cuts.len()];
        let b = if c.closed && b0 <= a { b0 + n } else { b0 };
        let data: Vec<P> = (a..=b).map(|i| c.at(i as isize)).collect();
        if data.len() < 2 {
            continue;
        }
        let m = 3.min(data.len() - 1).max(1);
        let straight = data.iter().all(|&q| dist_to_chord(data[0], data[data.len() - 1], q) <= (0.35f64).min(o.tol as f64 * 0.4));
        if straight {
            let (p0, p3) = (data[0], data[data.len() - 1]);
            out.push(Bez { p: [p0, p0, p3, p3], straight: true });
            continue;
        }
        let central = |i: isize| (c.at(i + m as isize) - c.at(i - m as isize)).norm();
        let t1 = if ha { (data[m] - data[0]).norm() } else { central(a as isize) };
        let t2 = if hb { (data[data.len() - 1 - m] - data[data.len() - 1]).norm() } else { central(b as isize) * -1.0 };
        let t1 = if t1.len() < 0.5 { (data[data.len() - 1] - data[0]).norm() } else { t1 };
        let t2 = if t2.len() < 0.5 { (data[0] - data[data.len() - 1]).norm() } else { t2 };
        fit_cubic(&data, t1, t2, err2, &mut out);
    }
    out
}

pub(super) fn anchors_from(bez: &[Bez], closed: bool) -> Vec<Anchor> {
    let n = bez.len();
    if n == 0 {
        return Vec::new();
    }
    let mut a: Vec<Anchor> = Vec::with_capacity(n + 1);
    for (i, b) in bez.iter().enumerate() {
        let mut an = Anchor::corner(b.p[0].v2());
        if !b.straight {
            an.handle_out = Some(b.p[1].v2());
        }
        if i > 0 && !bez[i - 1].straight {
            an.handle_in = Some(bez[i - 1].p[2].v2());
        }
        a.push(an);
    }
    if closed {
        if !bez[n - 1].straight {
            a[0].handle_in = Some(bez[n - 1].p[2].v2());
        }
    } else {
        let last = bez[n - 1];
        let mut an = Anchor::corner(last.p[3].v2());
        if !last.straight {
            an.handle_in = Some(last.p[2].v2());
        }
        a.push(an);
    }
    a
}

pub(super) fn polygon_anchors(poly: &[(i32, i32)], o: &TraceOptions) -> Vec<Anchor> {
    let pts: Vec<Vec2> = poly.iter().map(|&(x, y)| Vec2::new(x as f32, y as f32)).collect();
    let s = match o.fit {
        Fit::PolygonVw => visvalingam_closed(&pts, o.tol * 2.0),
        _ => douglas_peucker_closed(&pts, o.tol),
    };
    s.into_iter().map(Anchor::corner).collect()
}

pub(super) fn loop_to_anchors(l: &Loop, o: &TraceOptions) -> Vec<Anchor> {
    if o.fit != Fit::Bezier || l.pts.len() < 5 {
        return polygon_anchors(&l.pts, o);
    }
    let chain = Chain { pts: resample_closed(&l.pts), closed: true };
    let bez = fit_chain(chain, o);
    if bez.is_empty() {
        return polygon_anchors(&l.pts, o);
    }
    anchors_from(&bez, true)
}

// ---------------------------------------------------------------------------------------------
// fill mode
// ---------------------------------------------------------------------------------------------

pub(super) fn point_in_poly(pt: (f64, f64), poly: &[(i32, i32)]) -> bool {
    let mut inside = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        let (ay, by) = (a.1 as f64, b.1 as f64);
        if (ay > pt.1) != (by > pt.1) {
            let x = a.0 as f64 + (pt.1 - ay) / (by - ay) * (b.0 - a.0) as f64;
            if pt.0 < x {
                inside = !inside;
            }
        }
    }
    inside
}

/// Trace a bitmap into filled regions or centre lines.
pub fn trace(img: &Image, o: &TraceOptions) -> Result<TraceResult, String> {
    o.validate()?;
    match o.mode {
        Mode::Fill => trace_fill(img, o),
        Mode::Line => trace_line(img, o),
    }
}

pub(super) fn trace_fill(img: &Image, o: &TraceOptions) -> Result<TraceResult, String> {
    let (w, h) = (img.w, img.h);
    let (palette, mut lab) = quantise(img, o.colors, o.quant, o.seed);
    majority_filter(&mut lab, w, h, o.denoise);
    despeckle(&mut lab, w, h, o.min_area.ceil() as usize);
    let (comp, sizes) = components4(&lab, w, h);
    // bounding boxes
    let nc = sizes.len();
    let mut bb = vec![(usize::MAX, usize::MAX, 0usize, 0usize); nc];
    let mut firsts = vec![usize::MAX; nc];
    for y in 0..h {
        for x in 0..w {
            let c = comp[y * w + x] as usize;
            if firsts[c] == usize::MAX {
                firsts[c] = y * w + x;
            }
            let b = &mut bb[c];
            b.0 = b.0.min(x);
            b.1 = b.1.min(y);
            b.2 = b.2.max(x);
            b.3 = b.3.max(y);
        }
    }
    let mut regions: Vec<(Region, u16)> = Vec::new();
    for c in 0..nc {
        let first = firsts[c];
        if first == usize::MAX {
            continue;
        }
        let label = lab[first];
        if label == TRANSP {
            continue;
        }
        let (x0, y0, x1, y1) = bb[c];
        let (bw, bh) = (x1 - x0 + 1, y1 - y0 + 1);
        let inside = |x: usize, y: usize| comp[(y0 + y) * w + x0 + x] == c as u32;
        let loops = border_loops(bw, bh, &inside);
        let mut outers: Vec<&Loop> = Vec::new();
        let mut holes: Vec<&Loop> = Vec::new();
        for l in &loops {
            if signed_area_i(&l.pts) > 0 {
                outers.push(l);
            } else {
                holes.push(l);
            }
        }
        let off = (x0 as f32, y0 as f32);
        for ol in outers.iter() {
            let mut cuts: Vec<VPath> = Vec::new();
            // holes that are mostly transparent are cut out; the rest are stacked over
            for hl in &holes {
                let fgp = {
                    let (vx, vy) = hl.pts[0];
                    let d = if hl.pts[1].0 > vx { 0 } else if hl.pts[1].1 > vy { 1 } else if hl.pts[1].0 < vx { 2 } else { 3 };
                    let (fx, fy) = [(vx, vy), (vx - 1, vy), (vx - 1, vy - 1), (vx, vy - 1)][d];
                    (fx as f64 + 0.5, fy as f64 + 0.5)
                };
                let belongs = outers.len() == 1 || point_in_poly(fgp, &ol.pts);
                if !belongs {
                    continue;
                }
                // fraction of the hole border facing transparent pixels
                let (mut tr, mut tot) = (0usize, 0usize);
                let m = hl.pts.len();
                for i in 0..m {
                    let (a, b) = (hl.pts[i], hl.pts[(i + 1) % m]);
                    let len = ((b.0 - a.0).abs() + (b.1 - a.1).abs()) as usize;
                    let d = if b.0 > a.0 { 0 } else if b.1 > a.1 { 1 } else if b.0 < a.0 { 2 } else { 3 };
                    for s in 0..len {
                        let (vx, vy) = (a.0 + DX[d] * s as i32, a.1 + DY[d] * s as i32);
                        let (ox, oy) = [(vx, vy - 1), (vx, vy), (vx - 1, vy), (vx - 1, vy - 1)][d];
                        let (gx, gy) = (ox + x0 as i32, oy + y0 as i32);
                        tot += 1;
                        if gx >= 0 && gy >= 0 && (gx as usize) < w && (gy as usize) < h && lab[gy as usize * w + gx as usize] == TRANSP {
                            tr += 1;
                        }
                    }
                }
                if tot > 0 && tr * 2 >= tot {
                    let anchors = loop_to_anchors(hl, o);
                    if anchors.len() >= 2 {
                        let mut path = VPath::new("hole", true, anchors);
                        shift(&mut path, off);
                        cuts.push(path);
                    }
                }
            }
            let anchors = loop_to_anchors(ol, o);
            if anchors.len() < 2 {
                continue;
            }
            let mut outer = VPath::new("outer", true, anchors);
            shift(&mut outer, off);
            let oa = signed_area_i(&ol.pts) as f32 * 0.5;
            let col = palette[label as usize];
            regions.push((Region { color: col, outer, holes: cuts, pixels: sizes[c], outer_area: oa }, label));
        }
    }
    match o.zorder {
        ZOrder::Area => regions.sort_by(|a, b| b.0.outer_area.total_cmp(&a.0.outer_area).then(a.1.cmp(&b.1))),
        ZOrder::Luminance => regions.sort_by(|a, b| lum(b.0.color).total_cmp(&lum(a.0.color)).then(b.0.outer_area.total_cmp(&a.0.outer_area))),
    }
    Ok(TraceResult { width: w, height: h, mode: Mode::Fill, palette, regions: regions.into_iter().map(|r| r.0).collect(), strokes: Vec::new(), stroke_color: [0, 0, 0] })
}

pub(super) fn shift(path: &mut VPath, off: (f32, f32)) {
    let d = Vec2::new(off.0, off.1);
    for a in path.anchors.iter_mut() {
        a.pos += d;
        a.handle_in = a.handle_in.map(|h| h + d);
        a.handle_out = a.handle_out.map(|h| h + d);
    }
}
