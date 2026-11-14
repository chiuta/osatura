// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Lottie (Bodymovin JSON) exporter.
//!
//! Every vector layer becomes one shape layer whose path geometry is baked per frame into world
//! space (hold keyframes), so rigs, IK, deformers, springs and particles export faithfully.
//!
//! What is written: solid and gradient fills and strokes (gradient colour stops *with* alpha, end
//! points that follow the animated layer transform), dash patterns, caps, joins and miter limit,
//! stroke widths scaled by the layer's scale, blend modes, layer opacity, alpha / inverted-alpha
//! track mattes for layer masks, layer effects (blur, drop shadow, tint, fill) on single-leaf
//! layers, and **group opacity as a pre-composition**: a group whose opacity is not 1 becomes a
//! precomp layer with its own opacity, so its children are flattened first and overlaps do not
//! show through (Lottie's parenting does not carry opacity).
//!
//! What is NOT written is never dropped silently: it is returned as a [`LottieWarning`] (the CLI
//! prints them on stderr and in the `--report` JSON). The list: image layers, variable-width
//! strokes (base width used), blend mode `add` (screen used), strokes and gradients on a layer
//! with non-uniform scale or skew (approximated by the geometric-mean scale), blend / mask /
//! effects on groups, a mask whose source is not a vector layer, depth-of-field channels and sound.
//!
//! Numbers that came from `f32` are written as their shortest decimal (`23.976`, not
//! `23.97599983215332`).

use crate::engine::{Engine, Frame, NodeKind};
use crate::math::{Affine2, Vec2};
use crate::path::VPath;
use crate::project::Project;
use crate::style::{Cap, Join, Paint, Stop, StrokeStyle};
use serde_json::{json, Value};

/// One thing the Lottie format cannot carry (or carries only approximately).
#[derive(Clone, Debug, PartialEq)]
pub struct LottieWarning {
    /// Stable short code, e.g. `blend-add`, `variable-width-stroke`, `image-layer`.
    pub code: &'static str,
    /// Layer or node the warning is about (empty = whole document).
    pub layer: String,
    pub message: String,
}

impl std::fmt::Display for LottieWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.layer.is_empty() {
            write!(f, "[{}] {}", self.code, self.message)
        } else {
            write!(f, "[{}] {}: {}", self.code, self.layer, self.message)
        }
    }
}

/// `{"format":"lottie","warnings":[{code,layer,message}...],"count":n}` for the CLI `--report` option.
pub fn lottie_report_json(warnings: &[LottieWarning]) -> Value {
    json!({
        "format": "lottie",
        "count": warnings.len(),
        "warnings": warnings.iter().map(|w| json!({"code": w.code, "layer": w.layer, "message": w.message})).collect::<Vec<_>>(),
    })
}

/// Which engine node a Lottie layer was baked from (`sizefit` uses this to read reduced files back).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LottieLayerMap {
    /// Index into the document's top-level `layers` array.
    pub index: usize,
    /// Engine node id the layer's geometry comes from.
    pub node: usize,
    /// True for the extra copy of a mask source that acts as the track matte.
    pub matte_copy: bool,
}

/// The document, the layer map and everything that could not be represented.
pub struct LottieExport {
    pub doc: Value,
    pub map: Vec<LottieLayerMap>,
    pub warnings: Vec<LottieWarning>,
}

/// Export vector layers as a Lottie file (see the module documentation).
pub fn lottie_json(engine: &Engine) -> Value {
    lottie_export(engine).doc
}

/// [`lottie_json`] plus the layer-to-node map (background solids have no entry).
pub fn lottie_json_mapped(engine: &Engine) -> (Value, Vec<LottieLayerMap>) {
    let x = lottie_export(engine);
    (x.doc, x.map)
}

// ---- small helpers ---------------------------------------------------------------------------

fn lottie_shape(path: &VPath, world: &Affine2) -> Value {
    let n = path.anchors.len();
    let (mut v, mut i, mut o) = (
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    );
    for a in &path.anchors {
        let p = world.apply(a.pos);
        let (hi, ho) = (world.apply(a.hin()), world.apply(a.hout()));
        v.push(json!([p.x, p.y]));
        i.push(json!([hi.x - p.x, hi.y - p.y]));
        o.push(json!([ho.x - p.x, ho.y - p.y]));
    }
    json!({"i": i, "o": o, "v": v, "c": path.closed})
}

fn lottie_color(c: crate::style::Rgba) -> Value {
    json!([c.r, c.g, c.b, 1.0])
}

fn lottie_blend(b: crate::style::Blend) -> u8 {
    use crate::style::Blend::*;
    match b {
        Normal => 0,
        Multiply => 1,
        Screen => 2,
        Overlay => 3,
        Darken => 4,
        Lighten => 5,
        ColorDodge => 6,
        ColorBurn => 7,
        HardLight => 8,
        SoftLight => 9,
        Difference => 10,
        Exclusion => 11,
        Add => 2, // no linear dodge in Lottie: screen, with a warning
    }
}

/// A value that is the same on every frame is static, otherwise one hold key per frame.
fn anim_scalar(vals: &[f32]) -> Value {
    if vals.iter().all(|v| (v - vals[0]).abs() < 1e-4) {
        json!({"a": 0, "k": vals[0]})
    } else {
        json!({"a": 1, "k": vals.iter().enumerate().map(|(t, v)| json!({"t": t, "s": [v], "h": 1})).collect::<Vec<_>>()})
    }
}

fn anim_vec2(vals: &[[f32; 2]]) -> Value {
    if vals
        .iter()
        .all(|v| (v[0] - vals[0][0]).abs() < 1e-4 && (v[1] - vals[0][1]).abs() < 1e-4)
    {
        json!({"a": 0, "k": [vals[0][0], vals[0][1]]})
    } else {
        json!({"a": 1, "k": vals.iter().enumerate().map(|(t, v)| json!({"t": t, "s": [v[0], v[1]], "h": 1})).collect::<Vec<_>>()})
    }
}

/// Mean (geometric) scale of an affine map: the factor a stroke width / dash length scales by.
fn geo_scale(m: &Affine2) -> f32 {
    (m.a * m.d - m.b * m.c).abs().sqrt()
}

/// Largest / smallest singular value of the linear part (1 = similarity, shear and non-uniform scale > 1).
fn anisotropy(m: &Affine2) -> f32 {
    let t = m.a * m.a + m.b * m.b + m.c * m.c + m.d * m.d;
    let d = ((m.a * m.a + m.b * m.b - m.c * m.c - m.d * m.d).powi(2)
        + 4.0 * (m.a * m.c + m.b * m.d).powi(2))
    .sqrt();
    let (hi, lo) = (
        ((t + d) * 0.5).max(0.0).sqrt(),
        ((t - d) * 0.5).max(0.0).sqrt(),
    );
    if lo < 1e-9 {
        f32::INFINITY
    } else {
        hi / lo
    }
}

/// Colour stops (`[t,r,g,b]*`) followed, when any stop is translucent, by alpha stops (`[t,a]*`):
/// Lottie's "colour with opacity" gradient layout. `p` is the number of colour stops.
fn gradient_stops(stops: &[Stop]) -> (usize, Vec<f32>) {
    let stops = crate::style::sorted_stops(stops);
    let mut k = Vec::new();
    for s in &stops {
        k.extend([s.offset, s.color.r, s.color.g, s.color.b]);
    }
    if stops.iter().any(|s| s.color.a < 0.999) {
        for s in &stops {
            k.extend([s.offset, s.color.a.clamp(0.0, 1.0)]);
        }
    }
    (stops.len(), k)
}

fn gradient_stops_of(p: &Paint) -> &[Stop] {
    match p {
        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => stops,
        Paint::Solid(_) => &[],
    }
}

/// Start / end points of a gradient under `world`. Linear: the mapped end points. Radial: the mapped
/// centre and a point `radius * geometric scale` away along the mapped x axis.
fn gradient_points(p: &Paint, world: &Affine2) -> ([f32; 2], [f32; 2]) {
    match p {
        Paint::Linear { from, to, .. } => {
            let (a, b) = (world.apply(*from), world.apply(*to));
            ([a.x, a.y], [b.x, b.y])
        }
        Paint::Radial { center, radius, .. } => {
            let c = world.apply(*center);
            let ax = Vec2::new(world.a, world.b);
            let dir = if ax.length() > 1e-9 {
                ax * (1.0 / ax.length())
            } else {
                Vec2::new(1.0, 0.0)
            };
            let e = c + dir * (radius * geo_scale(world));
            ([c.x, c.y], [e.x, e.y])
        }
        Paint::Solid(_) => ([0.0; 2], [0.0; 2]),
    }
}

fn is_gradient(p: &Paint) -> bool {
    !matches!(p, Paint::Solid(_))
}

fn fx_prop(vals: &[f32]) -> Value {
    anim_scalar(vals)
}

fn fx_param(ty: i64, v: Value) -> Value {
    json!({"ty": ty, "nm": "p", "v": v})
}

/// Layer effects of the node `id` and of ancestors whose only vector leaf it is (a group effect over several leaves
/// cannot be split per leaf and is not written), innermost first, as Lottie `ef` entries with per-frame values.
fn lottie_effects(evals: &[Frame], pars: &[Vec<Option<usize>>], id: usize) -> Vec<Value> {
    let mut chain = vec![id];
    let mut cur = id;
    while let Some(pp) = pars[0][cur] {
        // an ancestor counts when every vector leaf under it is `id`
        let mut leaves = 0;
        let mut stack = vec![pp];
        while let Some(n) = stack.pop() {
            match evals[0].nodes[n].kind {
                NodeKind::Vector => leaves += 1,
                _ => stack.extend(evals[0].nodes[n].children.iter().copied()),
            }
        }
        if leaves == 1 {
            chain.push(pp);
        }
        cur = pp;
    }
    let mut out = Vec::new();
    for nid in chain {
        for k in 0..evals[0].nodes[nid].effects.len() {
            let at = |fi: usize| evals[fi].nodes[nid].effects.get(k);
            let per = |f: &dyn Fn(&crate::layerfx::EffectKind) -> f32| -> Vec<f32> {
                (0..evals.len())
                    .map(|fi| at(fi).map_or(0.0, |e| f(&e.kind)))
                    .collect()
            };
            use crate::layerfx::EffectKind as K;
            let col = |c: crate::style::Rgba| json!({"a": 0, "k": [c.r, c.g, c.b, 1.0]});
            match &evals[0].nodes[nid].effects[k].kind {
                K::Blur { .. } => out.push(json!({"ty": 29, "nm": "Gaussian Blur", "en": 1, "ef": [
                    fx_param(0, fx_prop(&per(&|e| if let K::Blur { radius } = e { *radius } else { 0.0 }))),
                    fx_param(7, json!({"a": 0, "k": 1})), fx_param(7, json!({"a": 0, "k": 0}))]})),
                K::DropShadow { color, .. } => {
                    let g = |e: &K| if let K::DropShadow { dx, dy, blur, opacity, .. } = e { (*dx, *dy, *blur, *opacity) } else { (0.0, 0.0, 0.0, 0.0) };
                    let dir = per(&|e| { let (dx, dy, ..) = g(e); if dx == 0.0 && dy == 0.0 { 0.0 } else { dx.atan2(-dy).to_degrees() } });
                    let dist = per(&|e| { let (dx, dy, ..) = g(e); dx.hypot(dy) });
                    let soft = per(&|e| g(e).2);
                    let op = per(&|e| g(e).3 * color.a.clamp(0.0, 1.0) * 255.0);
                    out.push(json!({"ty": 25, "nm": "Drop Shadow", "en": 1, "ef": [
                        fx_param(2, col(*color)), fx_param(0, fx_prop(&op)), fx_param(0, fx_prop(&dir)), fx_param(0, fx_prop(&dist)),
                        fx_param(0, fx_prop(&soft)), fx_param(7, json!({"a": 0, "k": 0}))]}));
                }
                K::Tint { black, white, .. } => {
                    let a = per(&|e| if let K::Tint { amount, .. } = e { amount * 100.0 } else { 0.0 });
                    out.push(json!({"ty": 20, "nm": "Tint", "en": 1, "ef": [fx_param(2, col(*black)), fx_param(2, col(*white)), fx_param(0, fx_prop(&a))]}));
                }
                K::Fill { color, .. } => {
                    let o = per(&|e| if let K::Fill { opacity, .. } = e { *opacity } else { 0.0 });
                    let z = || fx_param(7, json!({"a": 0, "k": 0}));
                    out.push(json!({"ty": 21, "nm": "Fill", "en": 1, "ef": [z(), z(), fx_param(2, col(*color)), z(), z(), z(), fx_param(0, fx_prop(&o))]}));
                }
            }
        }
    }
    out
}

/// Replace every number that is an `f32` widened to `f64` by the shortest decimal of that `f32`
/// (`23.976000308990479` becomes `23.976`), and integral floats keep their float form.
fn tidy(v: &mut Value) {
    match v {
        Value::Number(n) => {
            if let Some(x) = n.as_f64() {
                if n.is_f64() && x.is_finite() && (x as f32) as f64 == x {
                    if let Some(y) = format!("{}", x as f32)
                        .parse::<f64>()
                        .ok()
                        .and_then(serde_json::Number::from_f64)
                    {
                        *n = y;
                    }
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(tidy),
        Value::Object(o) => o.values_mut().for_each(tidy),
        _ => {}
    }
}

// ---- the builder -----------------------------------------------------------------------------

struct Builder<'a> {
    p: &'a Project,
    evals: Vec<Frame>,
    /// Parent per node and frame (the selected switch child only appears under its parent in its frame).
    pars: Vec<Vec<Option<usize>>>,
    /// Parent per node, any frame (structure).
    par_any: Vec<Option<usize>>,
    /// Node is reachable from the roots in that frame.
    drawn: Vec<Vec<bool>>,
    /// Group / switch whose opacity is not 1 on some frame: exported as a precomp.
    iso: Vec<bool>,
    warns: Vec<LottieWarning>,
    assets: Vec<Value>,
}

impl<'a> Builder<'a> {
    fn warn(&mut self, code: &'static str, layer: &str, message: String) {
        if !self
            .warns
            .iter()
            .any(|w| w.code == code && w.layer == layer)
        {
            self.warns.push(LottieWarning {
                code,
                layer: layer.to_string(),
                message,
            });
        }
    }

    fn nf(&self) -> usize {
        self.p.frames as usize
    }

    fn name(&self, id: usize) -> String {
        self.evals[0].nodes[id].name.clone()
    }

    /// Product of opacity and visibility from `from` up to (excluding) `stop`.
    fn factor(&self, fi: usize, from: usize, stop: Option<usize>) -> (f32, bool) {
        let f = &self.evals[fi];
        let (mut o, mut vis) = (1.0f32, self.drawn[fi][from]);
        let mut id = from;
        loop {
            if Some(id) == stop {
                break;
            }
            o *= f.nodes[id].opacity;
            vis &= f.nodes[id].visible;
            match self.pars[fi][id] {
                Some(pp) => id = pp,
                None => break,
            }
        }
        (o, vis)
    }

    /// Outermost opacity group strictly between `id` and `stop`.
    fn outer_iso(&self, id: usize, stop: Option<usize>) -> Option<usize> {
        let mut found = None;
        let mut cur = self.par_any[id];
        while let Some(c) = cur {
            if Some(c) == stop {
                break;
            }
            if self.iso[c] {
                found = Some(c);
            }
            cur = self.par_any[c];
        }
        found
    }

    fn opacity_value(&self, from: usize, stop: Option<usize>, force_on: bool) -> Value {
        let ops: Vec<f32> = (0..self.nf())
            .map(|fi| {
                let (o, vis) = self.factor(fi, from, stop);
                if force_on {
                    100.0
                } else if vis {
                    o * 100.0
                } else {
                    0.0
                }
            })
            .collect();
        anim_scalar(&ops)
    }

    /// Layers for `leaves` (bottom to top draw order), top of the stack first. `stop` = the group
    /// whose precomp this is (its own opacity is applied by the caller), `top` = the main composition.
    fn emit(
        &mut self,
        leaves: &[usize],
        stop: Option<usize>,
        map: Option<&mut Vec<LottieLayerMap>>,
    ) -> Vec<Value> {
        enum Unit {
            Leaf(usize),
            Block(usize, Vec<usize>),
        }
        let mut units: Vec<Unit> = Vec::new();
        for &id in leaves {
            match self.outer_iso(id, stop) {
                None => units.push(Unit::Leaf(id)),
                Some(x) => match units.last_mut() {
                    Some(Unit::Block(bx, ids)) if *bx == x => ids.push(id),
                    _ => units.push(Unit::Block(x, vec![id])),
                },
            }
        }
        let mut layers: Vec<Value> = Vec::new();
        let mut local_map: Vec<LottieLayerMap> = Vec::new();
        let mut ind = 1usize;
        for unit in units.into_iter().rev() {
            match unit {
                Unit::Leaf(id) => {
                    let mask = self.evals[0].nodes[id].mask;
                    let valid =
                        mask.filter(|&(mid, _)| self.evals[0].nodes[mid].kind == NodeKind::Vector);
                    if let (Some((mid, _)), None) = (mask, valid) {
                        let nm = self.name(id);
                        let src = self.name(mid);
                        self.warn("mask-source-not-vector", &nm, format!("the mask source '{src}' is not a vector layer, so the mask is not exported"));
                    }
                    match valid {
                        Some((mid, inverted)) => {
                            local_map.push(LottieLayerMap {
                                index: layers.len(),
                                node: mid,
                                matte_copy: true,
                            });
                            layers.push(self.make(mid, ind, true, true, None, stop));
                            ind += 1;
                            local_map.push(LottieLayerMap {
                                index: layers.len(),
                                node: id,
                                matte_copy: false,
                            });
                            layers.push(self.make(
                                id,
                                ind,
                                false,
                                false,
                                Some(if inverted { 2 } else { 1 }),
                                stop,
                            ));
                            ind += 1;
                        }
                        None => {
                            local_map.push(LottieLayerMap {
                                index: layers.len(),
                                node: id,
                                matte_copy: false,
                            });
                            layers.push(self.make(id, ind, false, false, None, stop));
                            ind += 1;
                        }
                    }
                }
                Unit::Block(x, ids) => {
                    let inner = self.emit(&ids, Some(x), None);
                    let asset_id = format!("comp_{}", self.assets.len());
                    self.assets
                        .push(json!({"id": asset_id, "nm": self.name(x), "layers": inner}));
                    let (w, h) = (self.p.width, self.p.height);
                    let xn = &self.evals[0].nodes[x];
                    let bm = lottie_blend(xn.blend);
                    let (blend, mask, fxn) = (xn.blend, xn.mask.is_some(), !xn.effects.is_empty());
                    let nm = self.name(x);
                    if blend == crate::style::Blend::Add {
                        self.warn(
                            "blend-add",
                            &nm,
                            "blend mode 'add' has no Lottie equivalent; 'screen' is used".into(),
                        );
                    }
                    if mask {
                        self.warn(
                            "group-mask",
                            &nm,
                            "a mask on a group is not exported (masks on single layers are)".into(),
                        );
                    }
                    if fxn {
                        self.warn(
                            "group-effect",
                            &nm,
                            "layer effects on a group are not exported".into(),
                        );
                    }
                    let o = self.opacity_value(x, stop, false);
                    layers.push(json!({
                        "ddd": 0, "ind": ind, "ty": 0, "nm": nm, "refId": asset_id, "sr": 1,
                        "ks": {"o": o, "r": {"a": 0, "k": 0}, "p": {"a": 0, "k": [0, 0, 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}},
                        "ao": 0, "w": w, "h": h, "ip": 0, "op": self.p.frames, "st": 0, "bm": bm
                    }));
                    ind += 1;
                }
            }
        }
        if let Some(m) = map {
            *m = local_map;
        }
        layers
    }

    /// One shape layer for the vector node `id`.
    fn make(
        &mut self,
        id: usize,
        ind: usize,
        force_on: bool,
        matte: bool,
        tt: Option<u8>,
        stop: Option<usize>,
    ) -> Value {
        let nf = self.nf();
        let layer_name = self.evals[0].nodes[id].name.clone();
        let npaths = (0..nf)
            .map(|fi| self.evals[fi].nodes[id].paths.len())
            .max()
            .unwrap_or(0);
        let mut shapes = Vec::new();
        for pi in (0..npaths).rev() {
            let template = self
                .evals
                .iter()
                .find_map(|f| f.nodes[id].paths.get(pi))
                .cloned();
            let Some(tmpl) = template else { continue };
            let keyframes: Vec<Value> = (0..nf)
                .map(|fi| {
                    let n = &self.evals[fi].nodes[id];
                    let shape = n
                        .paths
                        .get(pi)
                        .filter(|pp| pp.anchors.len() >= 2)
                        .map(|pp| lottie_shape(pp, &n.world))
                        .unwrap_or_else(|| json!({"i": [], "o": [], "v": [], "c": false}));
                    json!({"t": fi, "s": [shape], "h": 1})
                })
                .collect();
            // a path that never changes is written as a static shape (smaller file, and readers need no keyframes)
            let all_same = keyframes.windows(2).all(|w| w[0]["s"] == w[1]["s"]);
            let ks = if all_same {
                json!({"a": 0, "k": keyframes[0]["s"][0].clone()})
            } else {
                json!({"a": 1, "k": keyframes})
            };
            let mut items = vec![json!({"ty": "sh", "nm": format!("path{pi}"), "ks": ks})];
            // the paint of this path on every frame (falls back to the template where it is absent)
            let paint_at = |fi: usize, stroke: bool| -> Option<Paint> {
                let pth = self.evals[fi].nodes[id].paths.get(pi).unwrap_or(&tmpl);
                if stroke {
                    pth.stroke.as_ref().map(|s| s.paint.clone())
                } else {
                    pth.fill.clone()
                }
            };
            let world_at = |fi: usize| self.evals[fi].nodes[id].world;
            let mut gradient_world_warn = false;
            let mut fill_item: Option<Value> = None;
            if let Some(fill) = &tmpl.fill {
                match fill {
                    Paint::Linear { .. } | Paint::Radial { .. }
                        if fill.degenerate_color().is_none() =>
                    {
                        let (np, k) = gradient_stops(gradient_stops_of(fill));
                        let mut se = (Vec::new(), Vec::new());
                        for fi in 0..nf {
                            let pt = paint_at(fi, false).unwrap_or_else(|| fill.clone());
                            let w = world_at(fi);
                            if anisotropy(&w) > 1.01 {
                                gradient_world_warn = true;
                            }
                            let (s, e) = gradient_points(&pt, &w);
                            se.0.push(s);
                            se.1.push(e);
                        }
                        fill_item = Some(
                            json!({"ty": "gf", "nm": "gradient fill", "o": {"a": 0, "k": 100}, "r": 1, "t": if matches!(fill, Paint::Radial { .. }) { 2 } else { 1 },
                            "s": anim_vec2(&se.0), "e": anim_vec2(&se.1), "h": {"a": 0, "k": 0}, "a": {"a": 0, "k": 0},
                            "g": {"p": np, "k": {"a": 0, "k": k}}}),
                        );
                    }
                    _ => {
                        let c = fill.degenerate_color().unwrap_or_else(|| fill.base_color());
                        fill_item = Some(
                            json!({"ty": "fl", "c": {"a": 0, "k": lottie_color(c)}, "o": {"a": 0, "k": c.a * 100.0}, "r": 1}),
                        );
                    }
                }
            }
            if let Some(st) = &tmpl.stroke {
                let scales: Vec<f32> = (0..nf).map(|fi| geo_scale(&world_at(fi))).collect();
                let aniso = (0..nf)
                    .map(|fi| anisotropy(&world_at(fi)))
                    .fold(1.0f32, f32::max);
                let width = anim_scalar(
                    &scales
                        .iter()
                        .map(|s| st.width.max(0.0) * s)
                        .collect::<Vec<_>>(),
                );
                let lc = match st.cap {
                    Cap::Butt => 1,
                    Cap::Round => 2,
                    Cap::Square => 3,
                };
                let lj = match st.join {
                    Join::Miter => 1,
                    Join::Round => 2,
                    Join::Bevel => 3,
                };
                let dash = dash_entries(st, &scales);
                let mut item = if is_gradient(&st.paint) && st.paint.degenerate_color().is_none() {
                    let (np, k) = gradient_stops(gradient_stops_of(&st.paint));
                    let mut se = (Vec::new(), Vec::new());
                    for fi in 0..nf {
                        let pt = paint_at(fi, true).unwrap_or_else(|| st.paint.clone());
                        let w = world_at(fi);
                        if anisotropy(&w) > 1.01 {
                            gradient_world_warn = true;
                        }
                        let (s, e) = gradient_points(&pt, &w);
                        se.0.push(s);
                        se.1.push(e);
                    }
                    json!({"ty": "gs", "nm": "gradient stroke", "o": {"a": 0, "k": 100}, "w": width, "t": if matches!(st.paint, Paint::Radial { .. }) { 2 } else { 1 },
                        "s": anim_vec2(&se.0), "e": anim_vec2(&se.1), "h": {"a": 0, "k": 0}, "a": {"a": 0, "k": 0},
                        "g": {"p": np, "k": {"a": 0, "k": k}}, "lc": lc, "lj": lj, "ml": 4})
                } else {
                    let c = st
                        .paint
                        .degenerate_color()
                        .unwrap_or_else(|| st.paint.base_color());
                    json!({"ty": "st", "nm": "stroke", "c": {"a": 0, "k": lottie_color(c)}, "o": {"a": 0, "k": c.a * 100.0}, "w": width, "lc": lc, "lj": lj, "ml": 4})
                };
                if !dash.is_empty() {
                    item["d"] = Value::Array(dash);
                }
                items.push(item);
                if !st.width_profile.is_empty() {
                    self.warn("variable-width-stroke", &layer_name, "variable-width stroke profiles are not supported by Lottie; the constant base width is used".into());
                }
                if aniso > 1.01 {
                    self.warn("stroke-anisotropic", &layer_name, format!("the layer has a non-uniform scale or skew (ratio {aniso:.2}); Lottie strokes are uniform, so the width is scaled by the geometric mean scale"));
                }
            }
            // Lottie draws the first item of a group on top: the stroke goes before the fill
            items.extend(fill_item);
            if gradient_world_warn {
                self.warn("gradient-anisotropic", &layer_name, "a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)".into());
            }
            items.push(json!({"ty": "tr", "p": {"a": 0, "k": [0, 0]}, "a": {"a": 0, "k": [0, 0]}, "s": {"a": 0, "k": [100, 100]},
                "r": {"a": 0, "k": 0}, "o": {"a": 0, "k": 100}}));
            shapes.push(json!({"ty": "gr", "nm": format!("g{pi}"), "it": items}));
        }
        let first = &self.evals[0].nodes[id];
        let blend = first.blend;
        if blend == crate::style::Blend::Add {
            self.warn(
                "blend-add",
                &layer_name,
                "blend mode 'add' has no Lottie equivalent; 'screen' is used".into(),
            );
        }
        let o_val = self.opacity_value(id, stop, force_on);
        let mut layer = json!({
            "ddd": 0, "ind": ind, "ty": 4, "nm": layer_name, "sr": 1,
            "ks": {"o": o_val, "r": {"a": 0, "k": 0}, "p": {"a": 0, "k": [0, 0, 0]}, "a": {"a": 0, "k": [0, 0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}},
            "ao": 0, "shapes": shapes, "ip": 0, "op": self.p.frames, "st": 0, "bm": lottie_blend(blend)
        });
        if matte {
            layer["td"] = json!(1);
        } else {
            let ef = lottie_effects(&self.evals, &self.pars, id);
            if !ef.is_empty() {
                layer["ef"] = Value::Array(ef);
            }
        }
        if let Some(t) = tt {
            layer["tt"] = json!(t);
        }
        layer
    }
}

/// Lottie `d` entries (dash, gap, ..., offset), lengths scaled by the layer scale per frame.
/// Same validity rule as the rasteriser: an even number (>= 2) of positive lengths.
fn dash_entries(st: &StrokeStyle, scales: &[f32]) -> Vec<Value> {
    if !(st.dash.len() >= 2 && st.dash.len().is_multiple_of(2) && st.dash.iter().all(|d| *d > 0.0))
    {
        return vec![];
    }
    let mut out = Vec::new();
    for (i, d) in st.dash.iter().enumerate() {
        let v: Vec<f32> = scales.iter().map(|s| d * s).collect();
        out.push(json!({"n": if i % 2 == 0 { "d" } else { "g" }, "nm": if i % 2 == 0 { "dash" } else { "gap" }, "v": anim_scalar(&v)}));
    }
    if st.dash_offset != 0.0 {
        let v: Vec<f32> = scales.iter().map(|s| st.dash_offset * s).collect();
        out.push(json!({"n": "o", "nm": "offset", "v": anim_scalar(&v)}));
    }
    out
}

/// Exports the project and reports everything Lottie cannot carry.
pub fn lottie_export(engine: &Engine) -> LottieExport {
    let p = &engine.project;
    let evals: Vec<Frame> = (0..p.frames).map(|f| engine.evaluate(f as f32)).collect();
    let n_nodes = evals.first().map_or(0, |f| f.nodes.len());

    let pars: Vec<Vec<Option<usize>>> = evals
        .iter()
        .map(|f| {
            let mut par = vec![None; f.nodes.len()];
            for n in &f.nodes {
                for &c in &n.children {
                    par[c] = Some(n.id);
                }
            }
            par
        })
        .collect();
    let mut par_any = vec![None; n_nodes];
    for pr in &pars {
        for (i, x) in pr.iter().enumerate() {
            if par_any[i].is_none() {
                par_any[i] = *x;
            }
        }
    }
    let drawn: Vec<Vec<bool>> = evals
        .iter()
        .map(|f| {
            let mut d = vec![false; f.nodes.len()];
            let mut stack: Vec<usize> = f.roots.clone();
            while let Some(n) = stack.pop() {
                if !d[n] {
                    d[n] = true;
                    stack.extend(f.nodes[n].children.iter().copied());
                }
            }
            d
        })
        .collect();
    let iso: Vec<bool> = (0..n_nodes)
        .map(|i| {
            matches!(evals[0].nodes[i].kind, NodeKind::Group | NodeKind::Switch)
                && evals.iter().any(|f| f.nodes[i].opacity < 0.999)
        })
        .collect();

    // Visible vector leaves in draw order (bottom to top): the union over frames (switch selection varies).
    fn walk(f: &Frame, id: usize, out: &mut Vec<usize>) {
        let n = &f.nodes[id];
        match n.kind {
            NodeKind::Vector => out.push(id),
            NodeKind::Group | NodeKind::Switch => {
                for &c in &n.children {
                    walk(f, c, out);
                }
            }
            NodeKind::Image => {}
        }
    }
    let mut order: Vec<usize> = Vec::new();
    for f in &evals {
        let mut here = Vec::new();
        for &r in &f.roots {
            walk(f, r, &mut here);
        }
        for id in here {
            if !order.contains(&id) {
                order.push(id);
            }
        }
    }
    order.sort_unstable(); // preorder ids = draw order within a tree

    let mut b = Builder {
        p,
        evals,
        pars,
        par_any,
        drawn,
        iso,
        warns: Vec::new(),
        assets: Vec::new(),
    };

    // things that are not written at all
    for id in 0..n_nodes {
        let n0 = &b.evals[0].nodes[id];
        let name = n0.name.clone();
        match n0.kind {
            NodeKind::Image => {
                let ever = (0..b.nf()).any(|fi| {
                    let (_, vis) = b.factor(fi, id, None);
                    vis
                });
                if ever {
                    b.warn(
                        "image-layer",
                        &name,
                        "image layers are not exported (Lottie output holds vector layers only)"
                            .into(),
                    );
                }
            }
            NodeKind::Group | NodeKind::Switch if !b.iso[id] => {
                let (blend, mask, fxn) = (n0.blend, n0.mask.is_some(), !n0.effects.is_empty());
                if blend != crate::style::Blend::Normal {
                    b.warn(
                        "group-blend",
                        &name,
                        "a blend mode on a group is not exported".into(),
                    );
                }
                if mask {
                    b.warn(
                        "group-mask",
                        &name,
                        "a mask on a group is not exported (masks on single layers are)".into(),
                    );
                }
                if fxn {
                    let mut leaves = 0;
                    let mut st = vec![id];
                    while let Some(x) = st.pop() {
                        match b.evals[0].nodes[x].kind {
                            NodeKind::Vector => leaves += 1,
                            _ => st.extend(b.evals[0].nodes[x].children.iter().copied()),
                        }
                    }
                    if leaves != 1 {
                        b.warn(
                            "group-effect",
                            &name,
                            "layer effects on a group of several layers are not exported".into(),
                        );
                    }
                }
            }
            _ => {}
        }
    }
    if !p.sounds.is_empty() {
        b.warn(
            "audio",
            "",
            format!(
                "Lottie cannot carry audio; {} sound track(s) are not included",
                p.sounds.len()
            ),
        );
    }
    if p.channels.iter().any(|c| {
        matches!(
            c.prop,
            crate::project::Prop::CameraFocus | crate::project::Prop::CameraBlur
        )
    }) {
        b.warn(
            "depth-of-field",
            "",
            "depth-of-field channels (camera focus / blur) are not applied; use dof-export".into(),
        );
    }

    let mut map = Vec::new();
    let mut layers = b.emit(&order, None, Some(&mut map));
    let mut next_ind = layers.len() + 1;
    // Background as a bottom solid.
    if let Some(bg) = p.background {
        let _ = &mut next_ind;
        layers.push(json!({
            "ddd": 0, "ind": 9999, "ty": 1, "nm": "background", "sr": 1,
            "ks": {"o": {"a": 0, "k": bg.a * 100.0}, "r": {"a": 0, "k": 0}, "p": {"a": 0, "k": [p.width / 2.0, p.height / 2.0, 0]},
                   "a": {"a": 0, "k": [p.width / 2.0, p.height / 2.0, 0]}, "s": {"a": 0, "k": [100, 100, 100]}},
            "ao": 0, "sw": p.width, "sh": p.height, "sc": bg.to_hex(), "ip": 0, "op": p.frames, "st": 0, "bm": 0
        }));
    }
    let mut doc = json!({
        "v": "5.7.0", "fr": p.fps, "ip": 0, "op": p.frames, "w": p.width, "h": p.height,
        "nm": p.name, "ddd": 0, "assets": b.assets, "layers": layers
    });
    tidy(&mut doc);
    LottieExport {
        doc,
        map,
        warnings: b.warns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rect(x: f32, y: f32, w: f32, h: f32, extra: Value) -> Value {
        let mut p = json!({"name": "r", "closed": true, "anchors": [{"pos": [x, y]}, {"pos": [x + w, y]}, {"pos": [x + w, y + h]}, {"pos": [x, y + h]}]});
        for (k, v) in extra.as_object().unwrap() {
            p[k] = v.clone();
        }
        p
    }
    fn tr() -> Value {
        json!({"position": [0, 0], "rotation": 0, "scale": [1, 1], "skew": 0, "anchor": [0, 0]})
    }
    fn vec_layer(name: &str, paths: Vec<Value>) -> Value {
        json!({"name": name, "type": "vector", "paths": paths, "transform": tr(), "opacity": 1.0, "visible": true, "blend": "normal"})
    }
    fn project(layers: Vec<Value>, fps: f64) -> Engine {
        let d = json!({"format": 1, "name": "t", "width": 200, "height": 160, "fps": fps, "frames": 3, "background": "#ffffff",
            "camera": {"position": null, "zoom": 1, "rotation": 0}, "bones": [], "layers": layers, "channels": []});
        Engine::new(Project::from_json(&d.to_string()).unwrap()).unwrap()
    }
    fn items(doc: &Value, layer: usize) -> &Vec<Value> {
        doc["layers"][layer]["shapes"][0]["it"].as_array().unwrap()
    }
    fn item<'a>(doc: &'a Value, layer: usize, ty: &str) -> &'a Value {
        items(doc, layer)
            .iter()
            .find(|i| i["ty"] == ty)
            .unwrap_or_else(|| panic!("no {ty} item"))
    }

    /// C3a: translucent gradient stops are written in Lottie's colour-then-alpha layout.
    #[test]
    fn gradient_stop_alpha_is_written() {
        let g = json!({"from": [0, 0], "to": [100, 0], "stops": [{"offset": 1, "color": "#0000ff00"}, {"offset": 0, "color": "#ff0000ff"}]});
        let st = json!({"paint": {"from": [0, 0], "to": [100, 0], "stops": [{"offset": 0, "color": "#ff000080"}, {"offset": 1, "color": "#0000ffff"}]}, "width": 4});
        let e = project(
            vec![vec_layer(
                "a",
                vec![rect(
                    0.0,
                    0.0,
                    100.0,
                    50.0,
                    json!({"fill": g, "stroke": st}),
                )],
            )],
            12.0,
        );
        let x = lottie_export(&e);
        let gf = item(&x.doc, 0, "gf");
        assert_eq!(gf["g"]["p"], 2);
        let k: Vec<f64> = gf["g"]["k"]["k"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        assert_eq!(k.len(), 2 * 4 + 2 * 2, "colour stops then alpha stops");
        // unsorted input is sorted: offset 0 (red, alpha 1) first
        assert_eq!(&k[..4], &[0.0, 1.0, 0.0, 0.0]);
        assert_eq!(&k[8..], &[0.0, 1.0, 1.0, 0.0]);
        // d) a gradient stroke is a `gs`, with its alpha stop 0.5 (128/255)
        let gs = item(&x.doc, 0, "gs");
        let ks: Vec<f64> = gs["g"]["k"]["k"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        assert!((ks[9] - 128.0 / 255.0).abs() < 1e-3, "{ks:?}");
        assert!(
            items(&x.doc, 0).iter().all(|i| i["ty"] != "st"),
            "no first-stop solid stroke"
        );
        // the stroke is listed before the fill: Lottie draws the first item on top, the raster draws the stroke over the fill
        let order: Vec<&str> = items(&x.doc, 0)
            .iter()
            .map(|i| i["ty"].as_str().unwrap())
            .collect();
        assert_eq!(order, ["sh", "gs", "gf", "tr"]);
    }

    /// C3b/c: dash arrays, caps, joins and the miter limit.
    #[test]
    fn dash_caps_joins_and_miter_limit() {
        let st = json!({"paint": "#000000", "width": 8, "cap": "square", "join": "bevel", "dash": [12, 6], "dash_offset": 3});
        let st2 = json!({"paint": "#000000", "width": 8, "cap": "butt", "join": "miter"});
        let e = project(
            vec![
                vec_layer(
                    "a",
                    vec![rect(10.0, 10.0, 50.0, 50.0, json!({"stroke": st}))],
                ),
                vec_layer(
                    "b",
                    vec![rect(10.0, 10.0, 50.0, 50.0, json!({"stroke": st2}))],
                ),
            ],
            12.0,
        );
        let doc = lottie_json(&e);
        // layers are top first: b, a
        let s = item(&doc, 1, "st");
        assert_eq!(
            (s["lc"].as_i64(), s["lj"].as_i64(), s["ml"].as_f64()),
            (Some(3), Some(3), Some(4.0))
        );
        let d = s["d"].as_array().unwrap();
        let tags: Vec<&str> = d.iter().map(|x| x["n"].as_str().unwrap()).collect();
        assert_eq!(tags, ["d", "g", "o"]);
        assert_eq!(d[0]["v"]["k"], 12.0);
        assert_eq!(d[1]["v"]["k"], 6.0);
        assert_eq!(d[2]["v"]["k"], 3.0);
        let s2 = item(&doc, 0, "st");
        assert_eq!((s2["lc"].as_i64(), s2["lj"].as_i64()), (Some(1), Some(1)));
        assert!(s2.get("d").is_none());
    }

    /// C3e: group opacity is applied to the flattened group (a precomp layer), not multiplied into every leaf.
    #[test]
    fn group_opacity_is_a_precomp_with_its_own_opacity() {
        let g = json!({"name": "grp", "type": "group", "layers": [vec_layer("a", vec![rect(0.0, 0.0, 40.0, 40.0, json!({"fill": "#0000ff"}))]), vec_layer("b", vec![rect(20.0, 20.0, 40.0, 40.0, json!({"fill": "#00aa00"}))])],
            "transform": tr(), "opacity": 0.5, "visible": true, "blend": "normal"});
        let e = project(vec![g], 12.0);
        let doc = lottie_json(&e);
        let layers = doc["layers"].as_array().unwrap();
        let pre = layers
            .iter()
            .find(|l| l["ty"] == 0)
            .expect("a precomp layer");
        assert_eq!(pre["ks"]["o"]["k"], 50.0);
        let asset = doc["assets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == pre["refId"])
            .unwrap();
        let inner = asset["layers"].as_array().unwrap();
        assert_eq!(inner.len(), 2);
        for l in inner {
            assert_eq!(
                l["ks"]["o"]["k"], 100.0,
                "children keep full opacity inside the precomp"
            );
        }
        // no top-level shape layer carries the group's 50 %
        assert!(layers
            .iter()
            .filter(|l| l["ty"] == 4)
            .all(|l| l["ks"]["o"]["k"] != 50.0));
        // nested groups nest precomps
        let inner_g = json!({"name": "in", "type": "group", "layers": [vec_layer("p", vec![rect(0.0, 0.0, 10.0, 10.0, json!({"fill": "#000000"}))]), vec_layer("q", vec![rect(5.0, 5.0, 10.0, 10.0, json!({"fill": "#ff0000"}))])], "transform": tr(), "opacity": 0.5, "visible": true, "blend": "normal"});
        let outer = json!({"name": "out", "type": "group", "layers": [inner_g], "transform": tr(), "opacity": 0.8, "visible": true, "blend": "normal"});
        let doc = lottie_json(&project(vec![outer], 12.0));
        assert_eq!(doc["assets"].as_array().unwrap().len(), 2);
    }

    /// C3f: gradient end points follow an animated layer transform.
    #[test]
    fn gradient_follows_the_animated_transform() {
        let g = json!({"from": [-10, 0], "to": [10, 0], "stops": [{"offset": 0, "color": "#ff0000"}, {"offset": 1, "color": "#0000ff"}]});
        let mut l = vec_layer(
            "a",
            vec![rect(-10.0, -10.0, 20.0, 20.0, json!({"fill": g}))],
        );
        l["transform"]["position"] = json!([50, 50]);
        let d = json!({"format": 1, "name": "t", "width": 200, "height": 160, "fps": 12, "frames": 3, "background": "#ffffff", "camera": {"position": null, "zoom": 1, "rotation": 0}, "bones": [], "layers": [l],
            "channels": [{"target": "a", "prop": "layer_x", "source": "keys", "track": {"keys": [{"frame": 0, "value": 0, "ease": {"type": "linear"}}, {"frame": 2, "value": 40, "ease": {"type": "linear"}}], "extrapolate": "hold"}, "mode": "replace", "weight": 1.0}]});
        let e = Engine::new(Project::from_json(&d.to_string()).unwrap()).unwrap();
        let doc = lottie_json(&e);
        let gf = item(&doc, 0, "gf");
        assert_eq!(gf["s"]["a"], 1, "start point is animated");
        let k = gf["s"]["k"].as_array().unwrap();
        assert!(
            (k[2]["s"][0].as_f64().unwrap() - k[0]["s"][0].as_f64().unwrap() - 40.0).abs() < 1e-3,
            "moves with the layer"
        );
    }

    /// C3g-i, k: unsupported features are reported, never dropped silently.
    #[test]
    fn unsupported_features_are_warned_about() {
        let st = json!({"paint": "#000000", "width": 8, "width_profile": [[0, 1], [1, 0.2]]});
        let mut add = vec_layer(
            "add",
            vec![rect(0.0, 0.0, 20.0, 20.0, json!({"fill": "#ffffff"}))],
        );
        add["blend"] = json!("add");
        let mut sk = vec_layer(
            "skew",
            vec![rect(
                -10.0,
                -10.0,
                20.0,
                20.0,
                json!({"stroke": {"paint": "#000000", "width": 4}}),
            )],
        );
        sk["transform"]["scale"] = json!([2, 1]);
        let img = json!({"name": "pic", "type": "image", "source": {"embedded": "iVBORw0KGgo="}, "transform": tr(), "opacity": 1.0, "visible": true, "blend": "normal"});
        let _ = img;
        let mut grp = json!({"name": "g", "type": "group", "layers": [vec_layer("c", vec![rect(0.0, 0.0, 5.0, 5.0, json!({"fill": "#000000"}))])], "transform": tr(), "opacity": 1.0, "visible": true, "blend": "multiply"});
        grp["blend"] = json!("multiply");
        let e = project(
            vec![
                vec_layer(
                    "vw",
                    vec![rect(0.0, 0.0, 30.0, 30.0, json!({"stroke": st}))],
                ),
                add,
                sk,
                grp,
            ],
            12.0,
        );
        let x = lottie_export(&e);
        let codes: Vec<&str> = x.warnings.iter().map(|w| w.code).collect();
        for c in [
            "variable-width-stroke",
            "blend-add",
            "stroke-anisotropic",
            "group-blend",
        ] {
            assert!(codes.contains(&c), "missing {c}: {codes:?}");
        }
        let add_layer = x.doc["layers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["nm"] == "add")
            .unwrap();
        assert_eq!(add_layer["bm"], 2, "mapped to screen, with the warning");
        let rep = lottie_report_json(&x.warnings);
        assert_eq!(rep["count"].as_u64().unwrap() as usize, x.warnings.len());
        assert!(rep["warnings"][0]["message"].is_string());
        // a clean project has no warnings
        let clean = project(
            vec![vec_layer(
                "a",
                vec![rect(0.0, 0.0, 10.0, 10.0, json!({"fill": "#ff0000"}))],
            )],
            12.0,
        );
        assert!(lottie_export(&clean).warnings.is_empty());
    }

    /// C3i: the stroke width follows the layer scale (it used to ignore it).
    #[test]
    fn stroke_width_follows_layer_scale() {
        let mut l = vec_layer(
            "s",
            vec![rect(
                -10.0,
                -10.0,
                20.0,
                20.0,
                json!({"stroke": {"paint": "#000000", "width": 4}}),
            )],
        );
        l["transform"]["scale"] = json!([2, 2]);
        let doc = lottie_json(&project(vec![l], 12.0));
        assert_eq!(item(&doc, 0, "st")["w"]["k"], 8.0);
    }

    /// C3j: the frame rate is an exact decimal, and no f32 artefacts leak into the file.
    #[test]
    fn frame_rate_is_exact_decimal() {
        for (fps, text) in [
            (23.976, "23.976"),
            (29.97, "29.97"),
            (12.5, "12.5"),
            (24.0, "24.0"),
            (59.94, "59.94"),
        ] {
            let e = project(
                vec![vec_layer(
                    "a",
                    vec![rect(0.0, 0.0, 10.0, 10.0, json!({"fill": "#ff0000"}))],
                )],
                fps,
            );
            let s = lottie_json(&e).to_string();
            assert!(
                s.contains(&format!("\"fr\":{text}")),
                "{fps}: {}",
                &s[..120.min(s.len())]
            );
        }
        let s = lottie_json(&project(
            vec![vec_layer(
                "a",
                vec![rect(0.1, 0.0, 10.0, 10.0, json!({"fill": "#ff0000"}))],
            )],
            12.0,
        ))
        .to_string();
        assert!(!s.contains("0.10000000149"), "f32 widening artefacts");
    }

    /// Export -> import-lottie round trip keeps gradient alpha, dashes, caps and joins.
    #[test]
    fn round_trips_through_the_importer() {
        let g = json!({"from": [0, 0], "to": [100, 0], "stops": [{"offset": 0, "color": "#ff0000ff"}, {"offset": 1, "color": "#ff000000"}]});
        let st = json!({"paint": {"from": [0, 0], "to": [100, 0], "stops": [{"offset": 0, "color": "#0000ffff"}, {"offset": 1, "color": "#00ff00ff"}]}, "width": 6, "cap": "butt", "join": "bevel", "dash": [10, 5]});
        let e = project(
            vec![vec_layer(
                "a",
                vec![rect(
                    0.0,
                    0.0,
                    100.0,
                    50.0,
                    json!({"fill": g, "stroke": st}),
                )],
            )],
            12.0,
        );
        let doc = lottie_json(&e);
        let imp = crate::lottie_import::import_lottie(&doc.to_string()).unwrap();
        let mut found = false;
        fn walk(l: &crate::project::Layer, found: &mut bool) {
            if let crate::project::LayerKind::Vector { paths } = &l.kind {
                for p in paths {
                    if let Some(Paint::Linear { stops, .. }) = &p.fill {
                        if stops.last().is_some_and(|s| s.color.a < 0.01) && stops[0].color.a > 0.99
                        {
                            let s = p.stroke.as_ref().expect("stroke");
                            assert_eq!(s.dash, vec![10.0, 5.0]);
                            assert_eq!((s.cap, s.join), (Cap::Butt, Join::Bevel));
                            assert!(matches!(s.paint, Paint::Linear { .. }));
                            *found = true;
                        }
                    }
                }
            }
            for c in l.children() {
                walk(c, found);
            }
        }
        for l in &imp.project.layers {
            walk(l, &mut found);
        }
        assert!(
            found,
            "gradient alpha, dash, cap and join survive the round trip: {:?} {:#?}",
            imp.warnings,
            imp.project
                .layers
                .iter()
                .map(|l| format!("{:?}", l.kind))
                .collect::<Vec<_>>()
        );
    }
}
