// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `containers` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// Responsive canvas
// ---------------------------------------------------------------------------------------------

/// A copy of the project with another canvas size. Containers re-solve against it when the
/// copy is compiled; nothing else in the project changes.
pub fn responsive(p: &Project, w: f32, h: f32) -> Result<Project, String> {
    if !(w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0) {
        return Err("size must be two numbers > 0".into());
    }
    if w > crate::engine::MAX_STAGE || h > crate::engine::MAX_STAGE {
        return Err(format!("size must be at most {}", crate::engine::MAX_STAGE));
    }
    let mut q = p.clone();
    q.width = w;
    q.height = h;
    Ok(q)
}

/// Parse `WxH` (also `W×H`, `W,H`).
pub fn parse_size(s: &str) -> Result<(f32, f32), String> {
    let t = s.replace(['X', '×', ','], "x");
    let (a, b) = t.split_once(['x']).ok_or_else(|| format!("size must look like 800x600, got '{s}'"))?;
    let w: f32 = a.trim().parse().map_err(|_| format!("bad width in '{s}'"))?;
    let h: f32 = b.trim().parse().map_err(|_| format!("bad height in '{s}'"))?;
    if !(w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0) {
        return Err(format!("size must be positive, got '{s}'"));
    }
    Ok((w, h))
}

/// Set scroll offsets from external values: each scroll with an `input` path takes that value
/// (pixels) along its main axis (y for `both`). Unknown paths are ignored.
pub fn apply_inputs(p: &Project, values: &HashMap<String, f64>) -> Project {
    let mut q = p.clone();
    for s in &mut q.scrolls {
        if let Some(v) = s.input.as_ref().and_then(|i| values.get(i)) {
            if v.is_finite() {
                match s.axis {
                    ScrollAxis::X => s.offset[0] = *v as f32,
                    _ => s.offset[1] = *v as f32,
                }
            }
        }
    }
    q
}

/// Set the requested offset of a scroll by id.
pub fn set_offset(p: &mut Project, id: &str, offset: [f32; 2]) -> Result<(), String> {
    if !offset.iter().all(|v| v.is_finite()) {
        return Err("offset must be finite".into());
    }
    let s = p.scrolls.iter_mut().find(|s| s.id == id).ok_or_else(|| format!("no scroll '{id}'"))?;
    s.offset = offset;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Scroll runtime
// ---------------------------------------------------------------------------------------------

/// Nearest snap point (points are clamped into `0..=max`); no points = plain clamp.
pub fn snap_to(v: f32, points: &[f32], max: f32) -> f32 {
    let max = max.max(0.0);
    let v = v.max(0.0).min(max);
    let mut best: Option<f32> = None;
    for &q in points {
        let q = q.max(0.0).min(max);
        match best {
            Some(b) if (b - v).abs() <= (q - v).abs() => {}
            _ => best = Some(q),
        }
    }
    best.unwrap_or(v)
}

/// Deterministic scrolling simulation: drag (with rubber band), release with velocity (momentum
/// lands on the nearest snap point), spring return. Fixed 1/240 s sub-steps.
#[derive(Clone, Debug)]
pub struct ScrollRuntime {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub(super) target: Option<[f32; 2]>,
    pub(super) max: [f32; 2],
    pub(super) scroll: Scroll,
}

impl ScrollRuntime {
    /// `max` is the largest useful offset per axis (see [`ScrollInfo::max`]).
    pub fn new(scroll: &Scroll, max: [f32; 2]) -> ScrollRuntime {
        let mut r = ScrollRuntime { pos: [0.0; 2], vel: [0.0; 2], target: None, max, scroll: scroll.clone() };
        r.drag_to(scroll.offset);
        r
    }

    pub fn is_settled(&self) -> bool {
        self.target.is_none()
    }

    /// Replace the largest useful offset (after the canvas or the content changed). The current
    /// position is re-clamped (or left in its rubber band) accordingly.
    pub fn set_max(&mut self, max: [f32; 2]) {
        let raw = [0, 1].map(|a| raw_offset_for(self.pos[a], self.max[a], self.scroll.elastic.as_ref()));
        self.max = max;
        if self.target.is_none() {
            self.drag_to(raw);
        } else {
            for a in 0..2 {
                self.pos[a] = self.pos[a].max(0.0).min(max[a].max(0.0));
                if let Some(t) = &mut self.target {
                    t[a] = t[a].max(0.0).min(max[a].max(0.0));
                }
            }
        }
    }

    /// The largest useful offset per axis currently used.
    pub fn max(&self) -> [f32; 2] {
        self.max
    }

    /// Finger / pointer position: the raw offset the user asks for. Past the ends the displayed
    /// position is rubber-banded (elastic) or clamped.
    pub fn drag_to(&mut self, raw: [f32; 2]) {
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for a in 0..2 {
            self.pos[a] = if self.scroll.scrolls(a) {
                effective_offset(if raw[a].is_finite() { raw[a] } else { 0.0 }, self.max[a], self.scroll.elastic.as_ref())
            } else {
                0.0
            };
            self.vel[a] = 0.0;
        }
        self.target = None;
    }

    /// Let go with a release velocity (pixels per second). Momentum is projected `fling` seconds
    /// ahead, clamped and snapped; the position then springs to that landing offset.
    pub fn release(&mut self, velocity: [f32; 2]) {
        let mut t = [0.0f32; 2];
        for a in 0..2 {
            if !self.scroll.scrolls(a) {
                continue;
            }
            let v = if velocity[a].is_finite() { velocity[a] } else { 0.0 };
            let proj = self.pos[a] + v * self.scroll.fling;
            let pts = if a == 0 { &self.scroll.snap_x } else { &self.scroll.snap_y };
            t[a] = snap_to(proj, pts, self.max[a]);
        }
        self.vel = [0.0; 2];
        self.target = Some(t);
    }

    pub fn step(&mut self, dt: f32) {
        let Some(t) = self.target else { return };
        if !(dt.is_finite() && dt > 0.0) {
            return;
        }
        let (k, zeta) = match &self.scroll.elastic {
            Some(e) => (e.stiffness, e.damping_ratio),
            None => (def_stiffness(), 1.0),
        };
        let c = 2.0 * zeta * k.sqrt();
        let n = ((dt * 240.0).ceil() as usize).clamp(1, 100_000);
        let h = dt / n as f32;
        for _ in 0..n {
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for a in 0..2 {
                let acc = -k * (self.pos[a] - t[a]) - c * self.vel[a];
                self.vel[a] += acc * h;
                self.pos[a] += self.vel[a] * h;
            }
        }
        if (0..2).all(|a| (self.pos[a] - t[a]).abs() < 1e-3 && self.vel[a].abs() < 1e-2) {
            self.pos = t;
            self.vel = [0.0; 2];
            self.target = None;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Editing (project-level; each returns an undoable editor op)
// ---------------------------------------------------------------------------------------------

pub(super) fn vec_op<T: Serialize>(field: &str, present: bool, v: &T) -> Result<crate::edit::Op, String> {
    let value = serde_json::to_value(v).map_err(|e| e.to_string())?;
    Ok(if present {
        crate::edit::Op::Insert { path: format!("/{field}/-"), value }
    } else {
        crate::edit::Op::Set { path: format!("/{field}"), value: serde_json::json!([value]) }
    })
}

pub(super) fn check_trial(trial: &Project) -> Result<(), String> {
    let errs = validate(trial);
    if !errs.is_empty() {
        return Err(errs.join("\n"));
    }
    solve_project(trial).map(|_| ())
}

/// Add a container to a project (validated).
pub fn add_container(p: &mut Project, c: Container) -> Result<(), String> {
    if p.containers.iter().any(|x| x.id == c.id) {
        return Err(format!("container '{}' already exists", c.id));
    }
    let mut t = p.clone();
    t.containers.push(c.clone());
    check_trial(&t)?;
    p.containers.push(c);
    Ok(())
}

/// Remove a container; refused while something still refers to it.
pub fn remove_container(p: &mut Project, id: &str) -> Result<(), String> {
    let i = p.containers.iter().position(|c| c.id == id).ok_or_else(|| format!("no container '{id}'"))?;
    let mut t = p.clone();
    t.containers.remove(i);
    check_trial(&t)?;
    p.containers.remove(i);
    Ok(())
}

pub fn add_scroll(p: &mut Project, s: Scroll) -> Result<(), String> {
    if p.scrolls.iter().any(|x| x.id == s.id) {
        return Err(format!("scroll '{}' already exists", s.id));
    }
    let mut t = p.clone();
    t.scrolls.push(s.clone());
    check_trial(&t)?;
    p.scrolls.push(s);
    Ok(())
}

pub fn remove_scroll(p: &mut Project, id: &str) -> Result<(), String> {
    let i = p.scrolls.iter().position(|s| s.id == id).ok_or_else(|| format!("no scroll '{id}'"))?;
    p.scrolls.remove(i);
    Ok(())
}

/// The same as [`add_container`] as one editor operation (one undo step).
pub fn add_container_op(p: &Project, c: Container) -> Result<crate::edit::Op, String> {
    let mut t = p.clone();
    add_container(&mut t, c.clone())?;
    vec_op("containers", !p.containers.is_empty(), &c)
}

pub fn remove_container_op(p: &Project, id: &str) -> Result<crate::edit::Op, String> {
    let i = p.containers.iter().position(|c| c.id == id).ok_or_else(|| format!("no container '{id}'"))?;
    let mut t = p.clone();
    remove_container(&mut t, id)?;
    Ok(crate::edit::Op::Remove { path: format!("/containers/{i}") })
}

pub fn add_scroll_op(p: &Project, s: Scroll) -> Result<crate::edit::Op, String> {
    let mut t = p.clone();
    add_scroll(&mut t, s.clone())?;
    vec_op("scrolls", !p.scrolls.is_empty(), &s)
}

pub fn remove_scroll_op(p: &Project, id: &str) -> Result<crate::edit::Op, String> {
    let i = p.scrolls.iter().position(|s| s.id == id).ok_or_else(|| format!("no scroll '{id}'"))?;
    Ok(crate::edit::Op::Remove { path: format!("/scrolls/{i}") })
}

/// SVG overlay (outlines of every container and viewport) for previews.
pub fn overlay_svg(sol: &Solution) -> String {
    let mut s = String::from("<g fill=\"none\" stroke-width=\"1\" font-family=\"monospace\" font-size=\"10\">");
    for (id, r) in &sol.rects {
        s.push_str(&format!(
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" stroke=\"#e0409a\" stroke-dasharray=\"4 3\"/><text x=\"{:.2}\" y=\"{:.2}\" fill=\"#e0409a\">{}</text>",
            r[0], r[1], r[2], r[3], r[0] + 2.0, r[1] + 10.0, crate::svg::esc(id)
        ));
    }
    for sc in &sol.scrolls {
        let r = sc.viewport;
        s.push_str(&format!(
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" stroke=\"#2090e0\"/>",
            r[0], r[1], r[2], r[3]
        ));
    }
    s.push_str("</g>");
    s
}

/// Insert an overlay group before the closing `</svg>`.
pub fn with_overlay(svg: &str, sol: &Solution) -> String {
    match svg.rfind("</svg>") {
        Some(i) => format!("{}{}{}", &svg[..i], overlay_svg(sol), &svg[i..]),
        None => svg.to_string(),
    }
}
