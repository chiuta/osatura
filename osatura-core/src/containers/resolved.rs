// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `containers` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// Results
// ---------------------------------------------------------------------------------------------

/// What the engine adds to one layer: a translation and a scale factor.
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub layer: String,
    pub dx: f32,
    pub dy: f32,
    pub kx: f32,
    pub ky: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollInfo {
    pub id: String,
    pub viewport: [f32; 4],
    /// Content rectangle including the scroll shift.
    pub content: [f32; 4],
    /// Largest useful offset per axis.
    pub max: [f32; 2],
    /// Offset actually applied (clamped, or rubber-banded when elastic).
    pub offset: [f32; 2],
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Solution {
    pub canvas: (f32, f32),
    /// `(container id, x, y, w, h)` in container order.
    pub rects: Vec<(String, [f32; 4])>,
    pub layers: Vec<Placement>,
    pub scrolls: Vec<ScrollInfo>,
    pub notes: Vec<String>,
}

// ---------------------------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------------------------

pub(super) fn finite(v: f32) -> bool {
    v.is_finite()
}

pub(super) fn check_rule(r: &RectRule, who: &str, errs: &mut Vec<String>) {
    match r {
        RectRule::Fixed { x, y, w, h } => {
            if ![*x, *y, *w, *h].iter().all(|v| finite(*v)) {
                errs.push(format!("{who}: rect values must be finite"));
            } else if *w <= 0.0 || *h <= 0.0 {
                errs.push(format!("{who}: fixed rect needs w > 0 and h > 0"));
            }
        }
        RectRule::Pin { left, top, right, bottom, width, height } => {
            for (n, v) in [("left", left), ("top", top), ("right", right), ("bottom", bottom), ("width", width), ("height", height)] {
                if let Some(v) = v {
                    if !finite(*v) {
                        errs.push(format!("{who}: {n} must be finite"));
                    } else if (n == "width" || n == "height") && *v <= 0.0 {
                        errs.push(format!("{who}: {n} must be > 0"));
                    }
                }
            }
            if left.is_some() && right.is_some() && width.is_some() {
                errs.push(format!("{who}: left, right and width are all set (over-constrained)"));
            }
            if top.is_some() && bottom.is_some() && height.is_some() {
                errs.push(format!("{who}: top, bottom and height are all set (over-constrained)"));
            }
        }
        RectRule::Aspect { ratio, margin, ax, ay, .. } => {
            if !(finite(*ratio) && *ratio > 0.0) {
                errs.push(format!("{who}: aspect ratio must be a finite number > 0"));
            }
            if !(finite(*margin) && *margin >= 0.0) {
                errs.push(format!("{who}: margin must be >= 0"));
            }
            if !(finite(*ax) && finite(*ay) && (0.0..=1.0).contains(ax) && (0.0..=1.0).contains(ay)) {
                errs.push(format!("{who}: ax and ay must be within 0..1"));
            }
        }
        RectRule::Hug { x, y } => {
            if !(finite(*x) && finite(*y)) {
                errs.push(format!("{who}: hug position must be finite"));
            }
        }
    }
}

pub(super) fn layer_is_stretchable(l: &Layer) -> bool {
    let t = &l.transform;
    t.rotation == 0.0 && t.skew == 0.0 && layer_bounds(l).is_some()
}

/// Structural validation of `containers` and `scrolls`. Returns every problem found.
pub fn validate(p: &Project) -> Vec<String> {
    let mut errs = Vec::new();
    if p.containers.is_empty() && p.scrolls.is_empty() {
        return errs;
    }
    let mut ids: HashMap<&str, usize> = HashMap::new();
    for (i, c) in p.containers.iter().enumerate() {
        if c.id.trim().is_empty() {
            errs.push("container needs an id".into());
        } else if ids.insert(c.id.as_str(), i).is_some() {
            errs.push(format!("duplicate container id '{}'", c.id));
        }
    }
    let total_members: usize = p.containers.iter().map(|c| c.members.len()).sum();
    if total_members > MAX_MEMBERS {
        errs.push(format!("at most {MAX_MEMBERS} container members"));
        return errs;
    }
    let mut layer_owner: HashMap<&str, &str> = HashMap::new();
    let mut cont_owner: HashMap<&str, &str> = HashMap::new();
    for c in &p.containers {
        let who = format!("container '{}'", c.id);
        if !(finite(c.gap) && c.gap >= 0.0) {
            errs.push(format!("{who}: gap must be a finite number >= 0"));
        }
        if let Some(g) = c.cross_gap {
            if !(finite(g) && g >= 0.0) {
                errs.push(format!("{who}: cross_gap must be a finite number >= 0"));
            }
        }
        if !c.padding.iter().all(|v| finite(*v) && *v >= 0.0) {
            errs.push(format!("{who}: padding values must be finite and >= 0"));
        }
        check_rule(&c.rect, &who, &mut errs);
        for m in &c.members {
            let kinds = m.layer.is_some() as u8 + m.container.is_some() as u8 + m.spacer as u8;
            if kinds != 1 {
                errs.push(format!("{who}: a member needs exactly one of layer, container, spacer"));
                continue;
            }
            let ml = m.label();
            if !(finite(m.grow) && m.grow >= 0.0) || !(finite(m.shrink) && m.shrink >= 0.0) {
                errs.push(format!("{who}: {ml}: grow and shrink must be finite numbers >= 0"));
            }
            for (n, v) in [("width", m.width), ("height", m.height)] {
                if let Some(v) = v {
                    if !(finite(v) && v > 0.0) {
                        errs.push(format!("{who}: {ml}: {n} must be a finite number > 0"));
                    }
                }
            }
            for (a, (lo, hi)) in [(0, (m.min_w, m.max_w)), (1, (m.min_h, m.max_h))] {
                let n = if a == 0 { "w" } else { "h" };
                if let Some(v) = lo {
                    if !(finite(v) && v >= 0.0) {
                        errs.push(format!("{who}: {ml}: min_{n} must be a finite number >= 0"));
                    }
                }
                if let Some(v) = hi {
                    if !(finite(v) && v > 0.0) {
                        errs.push(format!("{who}: {ml}: max_{n} must be a finite number > 0"));
                    }
                }
                if let (Some(l), Some(h)) = (lo, hi) {
                    if finite(l) && finite(h) && l > h {
                        errs.push(format!("{who}: {ml}: min_{n} is larger than max_{n}"));
                    }
                }
            }
            if let Some(path) = &m.layer {
                match p.find_layer(path) {
                    None => errs.push(format!("{who}: unknown layer '{path}'")),
                    Some(l) => {
                        if let Some(prev) = layer_owner.insert(path.as_str(), c.id.as_str()) {
                            errs.push(format!("layer '{path}' is a member of both container '{prev}' and '{}'", c.id));
                        }
                        let measurable = layer_bounds(l).is_some();
                        if !measurable && (m.width.is_none() || m.height.is_none()) {
                            errs.push(format!("{who}: layer '{path}' has no measurable geometry; give width and height"));
                        }
                        if m.resize && !layer_is_stretchable(l) {
                            errs.push(format!("{who}: layer '{path}': resize needs measurable geometry and no rotation or skew"));
                        }
                    }
                }
            }
            if let Some(id) = &m.container {
                if !ids.contains_key(id.as_str()) {
                    errs.push(format!("{who}: unknown container '{id}'"));
                } else if let Some(prev) = cont_owner.insert(id.as_str(), c.id.as_str()) {
                    errs.push(format!("container '{id}' is nested in both '{prev}' and '{}'", c.id));
                }
            }
        }
    }
    // Cycles in nesting (including self-nesting).
    let mut state: HashMap<&str, u8> = HashMap::new();
    fn visit<'a>(p: &'a Project, ids: &HashMap<&str, usize>, id: &'a str, state: &mut HashMap<&'a str, u8>, errs: &mut Vec<String>) {
        match state.get(id) {
            Some(2) => return,
            Some(1) => {
                errs.push(format!("container nesting cycle through '{id}'"));
                return;
            }
            _ => {}
        }
        state.insert(id, 1);
        if let Some(&i) = ids.get(id) {
            for m in &p.containers[i].members {
                if let Some(ch) = &m.container {
                    if ids.contains_key(ch.as_str()) {
                        visit(p, ids, ch, state, errs);
                    }
                }
            }
        }
        state.insert(id, 2);
    }
    for c in &p.containers {
        visit(p, &ids, &c.id, &mut state, &mut errs);
    }
    if !errs.is_empty() {
        errs.sort();
        errs.dedup();
    }
    // Scrolls.
    let cycle_free = !errs.iter().any(|e| e.contains("cycle"));
    let mut sids = HashSet::new();
    let mut contents: HashMap<&str, &str> = HashMap::new();
    for s in &p.scrolls {
        let who = format!("scroll '{}'", s.id);
        if s.id.trim().is_empty() {
            errs.push("scroll needs an id".into());
        } else if !sids.insert(s.id.as_str()) {
            errs.push(format!("duplicate scroll id '{}'", s.id));
        }
        check_rule(&s.viewport, &who, &mut errs);
        if matches!(s.viewport, RectRule::Hug { .. }) {
            errs.push(format!("{who}: the viewport cannot be a hug rectangle"));
        }
        if !ids.contains_key(s.content.as_str()) {
            errs.push(format!("{who}: unknown content container '{}'", s.content));
        } else {
            if cont_owner.contains_key(s.content.as_str()) {
                errs.push(format!("{who}: content container '{}' is nested in another container", s.content));
            }
            if let Some(prev) = contents.insert(s.content.as_str(), s.id.as_str()) {
                errs.push(format!("container '{}' is the content of both scroll '{prev}' and '{}'", s.content, s.id));
            }
        }
        if !s.offset.iter().all(|v| finite(*v)) {
            errs.push(format!("{who}: offset must be finite"));
        }
        if !(finite(s.fling) && s.fling >= 0.0) {
            errs.push(format!("{who}: fling must be a finite number >= 0"));
        }
        if let Some(e) = &s.elastic {
            if !(finite(e.limit) && e.limit > 0.0 && finite(e.stiffness) && e.stiffness > 0.0 && finite(e.damping_ratio) && e.damping_ratio > 0.0) {
                errs.push(format!("{who}: elastic limit, stiffness and damping_ratio must be finite numbers > 0"));
            }
        }
        if !s.snap_x.iter().chain(&s.snap_y).all(|v| finite(*v)) {
            errs.push(format!("{who}: snap points must be finite"));
        }
        if let Some(cl) = &s.clip_layer {
            match p.find_layer(cl) {
                None => errs.push(format!("{who}: unknown clip layer '{cl}'")),
                Some(l) => {
                    if !layer_is_stretchable(l) {
                        errs.push(format!("{who}: clip layer '{cl}' needs measurable geometry and no rotation or skew"));
                    }
                    if layer_owner.contains_key(cl.as_str()) {
                        errs.push(format!("{who}: clip layer '{cl}' must not be a container member"));
                    }
                }
            }
        }
    }
    if cycle_free && errs.is_empty() {
        for s in &p.scrolls {
            let mut members = Vec::new();
            collect_layers(p, &ids, &s.content, &mut members, 0);
            for path in members {
                if let Some(l) = p.find_layer(path) {
                    let already = s.clip_layer.as_ref().map(|c| l.mask.as_ref().map(|m| &m.layer) == Some(c) && !l.mask.as_ref().unwrap().invert).unwrap_or(false);
                    if s.clip_layer.is_some() && l.mask.is_some() && !already {
                        errs.push(format!("scroll '{}': layer '{path}' already has a mask, so it cannot be clipped", s.id));
                    }
                }
                if Some(path) == s.clip_layer.as_deref() {
                    errs.push(format!("scroll '{}': the clip layer is part of its own content", s.id));
                }
            }
        }
    }
    errs.sort();
    errs.dedup();
    errs
}

/// Layer paths of a container and everything nested in it.
pub(super) fn collect_layers<'a>(p: &'a Project, ids: &HashMap<&str, usize>, id: &str, out: &mut Vec<&'a str>, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    let Some(&i) = ids.get(id) else { return };
    for m in &p.containers[i].members {
        if let Some(l) = &m.layer {
            out.push(l.as_str());
        }
        if let Some(c) = &m.container {
            collect_layers(p, ids, c, out, depth + 1);
        }
    }
}
