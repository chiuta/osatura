// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Per-layer effects (R-034 follow-up): an ordered list on `Layer::effects` that is rendered in an
//! isolated pass for just that layer (and its children) before its mask, opacity and blend mode.
//!
//! Effects: gaussian blur, drop shadow, tint (luminance ramp between two colours) and fill
//! (colour replacement). Their numbers can be animated with ordinary channels whose target is
//! `"<layer path>::fx<index>"` (`index` counts all effects of the layer, enabled or not) and whose
//! property is one of `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` (see [`LayerEffect::props`]).
//! Colours are not animatable.
//!
//! Renderers: the 8-bit raster and the float renderer run [`apply_premul`] on the layer's offscreen
//! buffer; SVG gets one `<filter>` per layer ([`svg_filter`]) built from feGaussianBlur, feOffset,
//! feColorMatrix and feMerge. Sizes (blur radius, offsets) are stage units: they follow the render
//! scale but not the layer's own transform or the camera zoom. Own design; standard maths only
//! (separable gaussian, Rec. 709 luma weights as in `style`).

use crate::project::Prop;
use crate::style::Rgba;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// At most this many effects per layer.
pub const MAX_EFFECTS: usize = 16;
/// Blur radius and shadow softness limit, stage units (sigma = radius / 2).
pub const MAX_BLUR: f32 = 64.0;
/// Shadow offset limit, stage units.
pub const MAX_OFFSET: f32 = 1024.0;
/// Hard limit of the blur radius in device pixels after scaling.
const MAX_BLUR_PX: f32 = 256.0;

fn yes() -> bool {
    true
}

fn one() -> f32 {
    1.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectKind {
    /// Gaussian blur, `radius` in stage units (sigma = radius / 2).
    Blur { radius: f32 },
    /// Shadow of the layer's alpha, offset by `dx`/`dy`, blurred by `blur`, drawn behind the layer.
    DropShadow {
        dx: f32,
        dy: f32,
        blur: f32,
        color: Rgba,
        #[serde(default = "one")]
        opacity: f32,
    },
    /// Map luminance between `black` and `white`; `amount` 0..1 keeps the rest of the original.
    Tint {
        black: Rgba,
        white: Rgba,
        #[serde(default = "one")]
        amount: f32,
    },
    /// Replace the colour; `opacity` 0..1 is the share of the new colour.
    Fill {
        color: Rgba,
        #[serde(default = "one")]
        opacity: f32,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerEffect {
    /// A disabled effect keeps its place in the list (and its channels) but is not drawn.
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(flatten)]
    pub kind: EffectKind,
}

impl LayerEffect {
    pub fn new(kind: EffectKind) -> LayerEffect {
        LayerEffect {
            enabled: true,
            kind,
        }
    }

    pub fn blur(radius: f32) -> LayerEffect {
        LayerEffect::new(EffectKind::Blur { radius })
    }

    pub fn drop_shadow(dx: f32, dy: f32, blur: f32, color: Rgba, opacity: f32) -> LayerEffect {
        LayerEffect::new(EffectKind::DropShadow {
            dx,
            dy,
            blur,
            color,
            opacity,
        })
    }

    pub fn tint(black: Rgba, white: Rgba, amount: f32) -> LayerEffect {
        LayerEffect::new(EffectKind::Tint {
            black,
            white,
            amount,
        })
    }

    pub fn fill(color: Rgba, opacity: f32) -> LayerEffect {
        LayerEffect::new(EffectKind::Fill { color, opacity })
    }

    /// The animatable numbers of this effect (channel properties).
    pub fn props(&self) -> &'static [Prop] {
        match self.kind {
            EffectKind::Blur { .. } => &[Prop::FxBlur],
            EffectKind::DropShadow { .. } => {
                &[Prop::FxDx, Prop::FxDy, Prop::FxSoft, Prop::FxAmount]
            }
            EffectKind::Tint { .. } | EffectKind::Fill { .. } => &[Prop::FxAmount],
        }
    }

    pub fn get(&self, p: Prop) -> Option<f32> {
        Some(match (&self.kind, p) {
            (EffectKind::Blur { radius }, Prop::FxBlur) => *radius,
            (EffectKind::DropShadow { dx, .. }, Prop::FxDx) => *dx,
            (EffectKind::DropShadow { dy, .. }, Prop::FxDy) => *dy,
            (EffectKind::DropShadow { blur, .. }, Prop::FxSoft) => *blur,
            (EffectKind::DropShadow { opacity, .. }, Prop::FxAmount) => *opacity,
            (EffectKind::Tint { amount, .. }, Prop::FxAmount) => *amount,
            (EffectKind::Fill { opacity, .. }, Prop::FxAmount) => *opacity,
            _ => return None,
        })
    }

    /// Set one number, clamped to its valid range (so a noisy channel can never leave it). Non-finite values are ignored.
    pub fn set(&mut self, p: Prop, v: f32) {
        if !v.is_finite() {
            return;
        }
        match (&mut self.kind, p) {
            (EffectKind::Blur { radius }, Prop::FxBlur) => *radius = v.clamp(0.0, MAX_BLUR),
            (EffectKind::DropShadow { dx, .. }, Prop::FxDx) => {
                *dx = v.clamp(-MAX_OFFSET, MAX_OFFSET)
            }
            (EffectKind::DropShadow { dy, .. }, Prop::FxDy) => {
                *dy = v.clamp(-MAX_OFFSET, MAX_OFFSET)
            }
            (EffectKind::DropShadow { blur, .. }, Prop::FxSoft) => *blur = v.clamp(0.0, MAX_BLUR),
            (EffectKind::DropShadow { opacity, .. }, Prop::FxAmount) => {
                *opacity = v.clamp(0.0, 1.0)
            }
            (EffectKind::Tint { amount, .. }, Prop::FxAmount) => *amount = v.clamp(0.0, 1.0),
            (EffectKind::Fill { opacity, .. }, Prop::FxAmount) => *opacity = v.clamp(0.0, 1.0),
            _ => {}
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let fin = |v: f32| v.is_finite();
        let col = |c: &Rgba| [c.r, c.g, c.b, c.a].iter().all(|v| v.is_finite());
        let ok = match &self.kind {
            EffectKind::Blur { radius } => fin(*radius) && (0.0..=MAX_BLUR).contains(radius),
            EffectKind::DropShadow {
                dx,
                dy,
                blur,
                color,
                opacity,
            } => {
                fin(*dx)
                    && fin(*dy)
                    && dx.abs() <= MAX_OFFSET
                    && dy.abs() <= MAX_OFFSET
                    && fin(*blur)
                    && (0.0..=MAX_BLUR).contains(blur)
                    && col(color)
                    && fin(*opacity)
                    && (0.0..=1.0).contains(opacity)
            }
            EffectKind::Tint {
                black,
                white,
                amount,
            } => col(black) && col(white) && fin(*amount) && (0.0..=1.0).contains(amount),
            EffectKind::Fill { color, opacity } => {
                col(color) && fin(*opacity) && (0.0..=1.0).contains(opacity)
            }
        };
        if ok {
            Ok(())
        } else {
            Err(format!("invalid parameters for {:?}", self.kind))
        }
    }
}

/// Validation of a layer's whole list (count and every effect). Errors are prefixed with `what`.
pub fn validate_list(what: &str, effects: &[LayerEffect]) -> Vec<String> {
    let mut errs = Vec::new();
    if effects.len() > MAX_EFFECTS {
        errs.push(format!("{what}: at most {MAX_EFFECTS} effects per layer"));
    }
    for (i, e) in effects.iter().enumerate() {
        if let Err(m) = e.validate() {
            errs.push(format!("{what}: effect {i}: {m}"));
        }
    }
    errs
}

/// Channel target of effect `index` on the layer with path `layer_path`.
pub fn target(layer_path: &str, index: usize) -> String {
    format!("{layer_path}::fx{index}")
}

const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

fn rgb(c: &Rgba) -> [f32; 3] {
    [
        c.r.clamp(0.0, 1.0),
        c.g.clamp(0.0, 1.0),
        c.b.clamp(0.0, 1.0),
    ]
}

/// The 4x5 colour matrix (row major, acting on straight RGBA) of a colour effect; `None` for the others.
fn color_matrix(k: &EffectKind) -> Option<[f32; 20]> {
    match k {
        EffectKind::Tint {
            black,
            white,
            amount,
        } => {
            let (b, w, a) = (rgb(black), rgb(white), amount.clamp(0.0, 1.0));
            let mut m = [0.0f32; 20];
            for c in 0..3 {
                let d = a * (w[c] - b[c]);
                for (j, l) in LUMA.iter().enumerate() {
                    m[c * 5 + j] = d * l + if j == c { 1.0 - a } else { 0.0 };
                }
                m[c * 5 + 4] = a * b[c];
            }
            m[18] = 1.0;
            Some(m)
        }
        EffectKind::Fill { color, opacity } => {
            let (c, o) = (rgb(color), opacity.clamp(0.0, 1.0));
            let mut m = [0.0f32; 20];
            for i in 0..3 {
                m[i * 5 + i] = 1.0 - o;
                m[i * 5 + 4] = o * c[i];
            }
            m[18] = 1.0;
            Some(m)
        }
        _ => None,
    }
}

fn apply_matrix(buf: &mut [f32], m: &[f32; 20]) {
    for p in buf.chunks_exact_mut(4) {
        let a = p[3];
        if a <= 0.0 {
            continue;
        }
        let c = [p[0] / a, p[1] / a, p[2] / a];
        for i in 0..3 {
            let v = (m[i * 5] * c[0] + m[i * 5 + 1] * c[1] + m[i * 5 + 2] * c[2] + m[i * 5 + 4])
                .clamp(0.0, 1.0);
            p[i] = v * a;
        }
    }
}

/// Pixels of padding around the canvas an effect chain needs so that content just outside the canvas
/// still blurs / casts shadows into it (a blur at the canvas edge then does not fade to transparent:
/// the layer is drawn on a canvas enlarged by this margin, filtered, and cropped back, as SVG
/// renderers do with the real content around the filter region). 0 when no effect spreads.
pub fn effect_pad(effects: &[LayerEffect], scale: f32) -> usize {
    let mut pad = 0usize;
    for e in effects.iter().filter(|e| e.enabled && e.validate().is_ok()) {
        match &e.kind {
            EffectKind::Blur { radius } => {
                pad += crate::postfx::blur_extent((radius * scale).min(MAX_BLUR_PX))
            }
            EffectKind::DropShadow { dx, dy, blur, .. } => {
                pad += (dx * scale).abs().max((dy * scale).abs()).ceil() as usize
                    + crate::postfx::blur_extent((blur * scale).min(MAX_BLUR_PX));
            }
            _ => {}
        }
    }
    pad.min(1024)
}

/// Run the enabled effects, in order, over a premultiplied RGBA f32 buffer (`w * h * 4`). `scale` is
/// the render scale (stage units to pixels). Invalid effects are skipped.
pub fn apply_premul(buf: &mut Vec<f32>, w: usize, h: usize, effects: &[LayerEffect], scale: f32) {
    if buf.len() != w * h * 4 {
        return;
    }
    for e in effects.iter().filter(|e| e.enabled && e.validate().is_ok()) {
        match &e.kind {
            EffectKind::Blur { radius } => {
                let r = (radius * scale).min(MAX_BLUR_PX);
                if r > 0.0 {
                    *buf = crate::postfx::blur_premul(buf, w, h, r);
                }
            }
            EffectKind::DropShadow {
                dx,
                dy,
                blur,
                color,
                opacity,
            } => {
                let alpha: Vec<f32> = buf
                    .chunks_exact(4)
                    .flat_map(|p| [0.0, 0.0, 0.0, p[3]])
                    .collect();
                let moved = crate::postfx::shift(
                    &alpha,
                    w,
                    h,
                    (dx * scale).round() as i32,
                    (dy * scale).round() as i32,
                );
                let soft = (blur * scale).min(MAX_BLUR_PX);
                let moved = if soft > 0.0 {
                    crate::postfx::blur_premul(&moved, w, h, soft)
                } else {
                    moved
                };
                let shadow = crate::postfx::solid_from_alpha(
                    &crate::postfx::alpha_of(&moved),
                    rgb(color),
                    opacity * color.a.clamp(0.0, 1.0),
                );
                *buf = crate::postfx::over(buf, &shadow);
            }
            k => {
                if let Some(m) = color_matrix(k) {
                    apply_matrix(buf, &m);
                }
            }
        }
    }
}

/// [`apply_premul`] on a tiny-skia pixmap (premultiplied 8-bit).
pub fn apply_pixmap(pm: &mut tiny_skia::Pixmap, effects: &[LayerEffect], scale: f32) {
    let (w, h) = (pm.width() as usize, pm.height() as usize);
    let mut f: Vec<f32> = pm.data().iter().map(|&b| b as f32 / 255.0).collect();
    apply_premul(&mut f, w, h, effects, scale);
    for (o, v) in pm.data_mut().iter_mut().zip(&f) {
        *o = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}

fn num(v: f32) -> String {
    let s = format!("{v:.5}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}

/// An SVG `<filter id="{id}">` with the same chain, covering the canvas (`width` x `height` user
/// units; SVG user space is the stage, so no scale is applied). `None` when nothing would be drawn.
pub fn svg_filter(effects: &[LayerEffect], id: &str, width: f32, height: f32) -> Option<String> {
    let mut prims = String::new();
    let mut cur = "SourceGraphic".to_string();
    for (k, e) in effects
        .iter()
        .enumerate()
        .filter(|(_, e)| e.enabled && e.validate().is_ok())
    {
        match &e.kind {
            EffectKind::Blur { radius } => {
                if *radius > 0.0 {
                    let _ = writeln!(
                        prims,
                        r#"<feGaussianBlur in="{cur}" stdDeviation="{}" result="r{k}"/>"#,
                        num(radius * 0.5)
                    );
                    cur = format!("r{k}");
                }
            }
            EffectKind::DropShadow {
                dx,
                dy,
                blur,
                color,
                opacity,
            } => {
                let c = rgb(color);
                let a = opacity * color.a.clamp(0.0, 1.0);
                let _ = writeln!(
                    prims,
                    r#"<feColorMatrix in="{cur}" type="matrix" values="0 0 0 0 {} 0 0 0 0 {} 0 0 0 0 {} 0 0 0 {} 0" result="s{k}"/>"#,
                    num(c[0]),
                    num(c[1]),
                    num(c[2]),
                    num(a)
                );
                let mut sh = format!("s{k}");
                let (ox, oy) = (dx.round(), dy.round());
                if ox != 0.0 || oy != 0.0 {
                    let _ = writeln!(
                        prims,
                        r#"<feOffset in="{sh}" dx="{}" dy="{}" result="o{k}"/>"#,
                        num(ox),
                        num(oy)
                    );
                    sh = format!("o{k}");
                }
                if *blur > 0.0 {
                    let _ = writeln!(
                        prims,
                        r#"<feGaussianBlur in="{sh}" stdDeviation="{}" result="b{k}"/>"#,
                        num(blur * 0.5)
                    );
                    sh = format!("b{k}");
                }
                let _ = writeln!(
                    prims,
                    r#"<feMerge result="r{k}"><feMergeNode in="{sh}"/><feMergeNode in="{cur}"/></feMerge>"#
                );
                cur = format!("r{k}");
            }
            kind => {
                if let Some(m) = color_matrix(kind) {
                    let vals: Vec<String> = m.iter().map(|v| num(*v)).collect();
                    let _ = writeln!(
                        prims,
                        r#"<feColorMatrix in="{cur}" type="matrix" values="{}" result="r{k}"/>"#,
                        vals.join(" ")
                    );
                    cur = format!("r{k}");
                }
            }
        }
    }
    if prims.is_empty() {
        return None;
    }
    Some(format!(
        "<filter id=\"{id}\" filterUnits=\"userSpaceOnUse\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" color-interpolation-filters=\"sRGB\">\n{prims}</filter>\n",
        num(width),
        num(height)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_and_defaults() {
        let e = vec![
            LayerEffect::blur(3.0),
            LayerEffect::drop_shadow(2.0, 3.0, 4.0, Rgba::BLACK, 0.5),
        ];
        let j = serde_json::to_string(&e).unwrap();
        let back: Vec<LayerEffect> = serde_json::from_str(&j).unwrap();
        assert_eq!(e, back);
        let d: LayerEffect =
            serde_json::from_str(r##"{"type":"tint","black":"#000000","white":"#ff0000"}"##)
                .unwrap();
        assert!(d.enabled && d.get(Prop::FxAmount) == Some(1.0));
        assert!(serde_json::from_str::<LayerEffect>(r#"{"type":"nope"}"#).is_err());
    }

    #[test]
    fn set_clamps_and_ignores_nan() {
        let mut e = LayerEffect::blur(3.0);
        e.set(Prop::FxBlur, 1e9);
        assert_eq!(e.get(Prop::FxBlur), Some(MAX_BLUR));
        e.set(Prop::FxBlur, f32::NAN);
        assert_eq!(e.get(Prop::FxBlur), Some(MAX_BLUR));
        e.set(Prop::FxDx, 1.0);
        assert_eq!(e.get(Prop::FxDx), None);
    }

    #[test]
    fn validation_rejects_bad_values() {
        assert!(validate_list("l", &[LayerEffect::blur(-1.0)]).len() == 1);
        assert!(validate_list("l", &[LayerEffect::blur(f32::NAN)]).len() == 1);
        assert!(validate_list("l", &[LayerEffect::tint(Rgba::BLACK, Rgba::WHITE, 2.0)]).len() == 1);
        assert!(validate_list("l", &vec![LayerEffect::blur(1.0); MAX_EFFECTS + 1]).len() == 1);
        assert!(validate_list("l", &[LayerEffect::blur(1.0)]).is_empty());
    }

    #[test]
    fn svg_filter_composes_standard_primitives() {
        let f = svg_filter(
            &[
                LayerEffect::blur(4.0),
                LayerEffect::drop_shadow(3.0, 2.0, 2.0, Rgba::BLACK, 0.5),
                LayerEffect::tint(Rgba::BLACK, Rgba::WHITE, 1.0),
            ],
            "f0",
            10.0,
            10.0,
        )
        .unwrap();
        for p in [
            "feGaussianBlur",
            "feOffset",
            "feMerge",
            "feColorMatrix",
            "sRGB",
        ] {
            assert!(f.contains(p), "{p} missing in {f}");
        }
        assert!(svg_filter(&[LayerEffect::blur(0.0)], "f", 1.0, 1.0).is_none());
        let mut off = LayerEffect::blur(4.0);
        off.enabled = false;
        assert!(svg_filter(&[off], "f", 1.0, 1.0).is_none());
    }

    // ---- engine-level behaviour -------------------------------------------------------------

    use crate::anim::{Ease, Track};
    use crate::engine::Engine;
    use crate::path::VPath;
    use crate::percep::Img;
    use crate::project::{Channel, Layer, Project};
    use crate::raster::{render_rgba, RasterOptions};

    const W: usize = 60;
    const H: usize = 40;

    fn rect(name: &str, x: f32, y: f32, w: f32, h: f32, col: &str) -> Layer {
        Layer::vector(name, vec![VPath::rect("r", x, y, w, h).with_fill(col)])
    }

    fn project(layers: Vec<Layer>, channels: Vec<Channel>) -> Project {
        let mut p = Project::new("fx", W as f32, H as f32, 10.0, 11);
        p.layers = layers;
        p.channels = channels;
        p
    }

    fn img(p: &Project, frame: f32) -> Img {
        let e = Engine::new(p.clone()).unwrap();
        let (w, h, d) = render_rgba(
            &e,
            frame,
            &RasterOptions {
                scale: 1.0,
                show_bones: false,
                transparent: true,
            },
        );
        Img::new(w as usize, h as usize, d).unwrap()
    }

    fn with_fx(mut l: Layer, fx: Vec<LayerEffect>) -> Layer {
        l.effects = fx;
        l
    }

    #[test]
    fn blur_softens_edges_and_keeps_the_middle() {
        let sharp = img(
            &project(vec![rect("a", 20.0, 10.0, 20.0, 20.0, "#ff0000")], vec![]),
            0.0,
        );
        let soft = img(
            &project(
                vec![with_fx(
                    rect("a", 20.0, 10.0, 20.0, 20.0, "#ff0000"),
                    vec![LayerEffect::blur(6.0)],
                )],
                vec![],
            ),
            0.0,
        );
        assert_eq!(sharp.px(18, 20)[3], 0);
        assert!(
            soft.px(18, 20)[3] > 10,
            "halo outside the edge: {:?}",
            soft.px(18, 20)
        );
        assert!(soft.px(21, 20)[3] < 250, "edge pixel is no longer opaque");
        assert!(soft.px(30, 20)[3] >= 254, "centre stays opaque");
        let sum = |i: &Img| i.rgba.chunks(4).map(|p| p[3] as f64).sum::<f64>();
        assert!(
            (sum(&sharp) - sum(&soft)).abs() / sum(&sharp) < 0.02,
            "alpha energy is conserved"
        );
        assert_eq!(
            &soft.px(18, 20)[..3],
            &[255, 0, 0],
            "colour of the halo stays red"
        );
    }

    /// C10c: a layer that extends past the canvas blurs against its real surroundings: the canvas edge
    /// does not fade to transparent (Chromium: red to the edge, verified with a 200x160 scene, max diff 0).
    /// A layer that really ends at the edge of the canvas still falls off there.
    #[test]
    fn blur_at_the_canvas_edge_uses_the_content_beyond_it() {
        let big = with_fx(
            rect(
                "a",
                -30.0,
                -30.0,
                W as f32 + 60.0,
                H as f32 + 60.0,
                "#ff0000",
            ),
            vec![LayerEffect::blur(8.0)],
        );
        let p = project(vec![big], vec![]);
        let i = img(&p, 0.0);
        assert!(
            i.px(0, H / 2)[3] >= 253 && i.px(W / 2, 0)[3] >= 253 && i.px(W - 1, H - 1)[3] >= 253,
            "edge pixels stay opaque: {:?}",
            i.px(0, H / 2)
        );
        // the float renderer agrees
        let e = Engine::new(p).unwrap();
        let o = RasterOptions {
            scale: 1.0,
            show_bones: false,
            transparent: true,
        };
        let f = crate::fraster::render_float(&e, 0.0, &o).to_straight_u8();
        let a = |x: usize, y: usize| f[(y * W + x) * 4 + 3];
        assert!(
            a(0, H / 2) >= 253 && a(W / 2, 0) >= 253,
            "float edge {}",
            a(0, H / 2)
        );
        // a layer that really ends at the canvas edge still falls off there
        let flush = project(
            vec![with_fx(
                rect("a", 0.0, 0.0, W as f32, H as f32, "#ff0000"),
                vec![LayerEffect::blur(8.0)],
            )],
            vec![],
        );
        let j = img(&flush, 0.0);
        assert!(
            j.px(0, H / 2)[3] < 200 && j.px(W / 2, H / 2)[3] >= 250,
            "{:?} {:?}",
            j.px(0, H / 2),
            j.px(W / 2, H / 2)
        );
    }

    #[test]
    fn drop_shadow_is_offset_coloured_and_behind() {
        let l = with_fx(
            rect("a", 10.0, 10.0, 10.0, 10.0, "#ff0000"),
            vec![LayerEffect::drop_shadow(
                8.0,
                6.0,
                0.0,
                Rgba::new(0.0, 0.0, 1.0, 1.0),
                0.5,
            )],
        );
        let i = img(&project(vec![l], vec![]), 0.0);
        assert_eq!(i.px(12, 12), [255, 0, 0, 255], "layer stays on top");
        let s = i.px(25, 23);
        assert_eq!(&s[..3], &[0, 0, 255], "{s:?}");
        assert!((s[3] as i32 - 128).abs() <= 1, "{s:?}");
        assert_eq!(i.px(40, 35)[3], 0);
    }

    #[test]
    fn tint_and_fill_recolour_groups_and_images() {
        let group = Layer::group(
            "g",
            vec![
                rect("a", 0.0, 0.0, 10.0, 10.0, "#ffffff"),
                rect("b", 20.0, 0.0, 10.0, 10.0, "#000000"),
            ],
        );
        let tinted = with_fx(
            group.clone(),
            vec![LayerEffect::tint(
                Rgba::new(0.0, 0.0, 1.0, 1.0),
                Rgba::new(1.0, 1.0, 0.0, 1.0),
                1.0,
            )],
        );
        let i = img(&project(vec![tinted], vec![]), 0.0);
        assert_eq!(i.px(5, 5), [255, 255, 0, 255]);
        assert_eq!(i.px(25, 5), [0, 0, 255, 255]);
        let filled = with_fx(
            group,
            vec![LayerEffect::fill(Rgba::new(0.0, 1.0, 0.0, 1.0), 0.5)],
        );
        let i = img(&project(vec![filled], vec![]), 0.0);
        assert_eq!(i.px(5, 5), [128, 255, 128, 255]);
        // an image layer (2x2 red PNG)
        let mut enc = Vec::new();
        {
            let mut e = png::Encoder::new(&mut enc, 2, 2);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            let mut w = e.write_header().unwrap();
            w.write_image_data(&[255, 0, 0, 255].repeat(4)).unwrap();
        }
        let im = with_fx(
            Layer::image_from_png("im", &enc, crate::math::Vec2::ZERO),
            vec![LayerEffect::fill(Rgba::new(0.0, 1.0, 0.0, 1.0), 1.0)],
        );
        let i = img(&project(vec![im], vec![]), 0.0);
        let p = i.px(0, 0);
        assert!(p[1] > 250 && p[0] < 5 && p[3] > 250, "{p:?}");
    }

    #[test]
    fn effects_are_isolated_to_their_layer() {
        let a = rect("a", 4.0, 14.0, 10.0, 10.0, "#ff0000");
        let b = rect("b", 44.0, 14.0, 10.0, 10.0, "#0000ff");
        let plain = img(&project(vec![a.clone(), b.clone()], vec![]), 0.0);
        let fx = img(
            &project(
                vec![
                    with_fx(
                        a.clone(),
                        vec![
                            LayerEffect::blur(5.0),
                            LayerEffect::drop_shadow(3.0, 3.0, 2.0, Rgba::BLACK, 1.0),
                        ],
                    ),
                    b.clone(),
                ],
                vec![],
            ),
            0.0,
        );
        assert_ne!(plain, fx);
        // everything right of x = 30 (b and the space around it) is bit-identical
        for y in 0..H {
            for x in 30..W {
                assert_eq!(plain.px(x, y), fx.px(x, y), "({x},{y})");
            }
        }
        // and b is still crisp
        assert_eq!(fx.px(44, 19), [0, 0, 255, 255]);
        assert_eq!(fx.px(43, 19)[3], 0);
        // the same layer alone renders like it does in company (no cross talk the other way round)
        let alone = img(
            &project(
                vec![with_fx(a.clone(), vec![LayerEffect::blur(5.0)])],
                vec![],
            ),
            0.0,
        );
        let together = img(
            &project(vec![with_fx(a, vec![LayerEffect::blur(5.0)]), b], vec![]),
            0.0,
        );
        for y in 0..H {
            for x in 0..30 {
                assert_eq!(alone.px(x, y), together.px(x, y));
            }
        }
    }

    #[test]
    fn effects_apply_before_opacity_and_in_order() {
        let base = rect("a", 20.0, 10.0, 20.0, 20.0, "#ff0000");
        let mut l = with_fx(base.clone(), vec![LayerEffect::blur(6.0)]);
        l.opacity = 0.5;
        let i = img(&project(vec![l], vec![]), 0.0);
        assert!(
            (i.px(30, 20)[3] as i32 - 128).abs() <= 1,
            "opacity multiplies the effect result: {:?}",
            i.px(30, 20)
        );
        // order matters: a fill after the shadow recolours the shadow too, a fill before it leaves the shadow black
        let sh = || LayerEffect::drop_shadow(14.0, 0.0, 0.0, Rgba::BLACK, 1.0);
        let green = || LayerEffect::fill(Rgba::new(0.0, 1.0, 0.0, 1.0), 1.0);
        let shadow_first = img(
            &project(vec![with_fx(base.clone(), vec![sh(), green()])], vec![]),
            0.0,
        );
        let fill_first = img(
            &project(vec![with_fx(base, vec![green(), sh()])], vec![]),
            0.0,
        );
        assert_eq!(shadow_first.px(50, 20), [0, 255, 0, 255]);
        assert_eq!(fill_first.px(50, 20), [0, 0, 0, 255]);
        assert_eq!(fill_first.px(30, 20), [0, 255, 0, 255]);
    }

    #[test]
    fn disabled_effects_do_nothing_and_keep_their_slot() {
        let base = rect("a", 20.0, 10.0, 20.0, 20.0, "#ff0000");
        let mut off = LayerEffect::blur(8.0);
        off.enabled = false;
        let none = img(&project(vec![base.clone()], vec![]), 0.0);
        let dis = img(
            &project(vec![with_fx(base.clone(), vec![off.clone()])], vec![]),
            0.0,
        );
        assert_eq!(none, dis);
        // a channel on a disabled effect still resolves (index counts all effects) and changes nothing
        let ch = vec![Channel::keys(
            "a::fx0",
            Prop::FxBlur,
            Track::with_keys(&[(0.0, 1.0, Ease::Linear), (10.0, 9.0, Ease::Linear)]),
        )];
        let p = project(vec![with_fx(base.clone(), vec![off.clone()])], ch.clone());
        assert_eq!(img(&p, 5.0), none);
        let e = Engine::new(p).unwrap();
        assert!(e.evaluate(5.0).nodes[0].effects.is_empty());
        // index counts disabled ones: the second effect is fx1
        let ch1 = vec![Channel::keys(
            "a::fx1",
            Prop::FxBlur,
            Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 8.0, Ease::Linear)]),
        )];
        let p = project(vec![with_fx(base, vec![off, LayerEffect::blur(0.0)])], ch1);
        let e = Engine::new(p).unwrap();
        assert_eq!(
            e.evaluate(5.0).nodes[0].effects[0].get(Prop::FxBlur),
            Some(4.0)
        );
    }

    #[test]
    fn channels_animate_every_effect_number() {
        let base = rect("a", 20.0, 10.0, 20.0, 20.0, "#ff0000");
        let key = |t: &str, p: Prop, a: f32, b: f32| {
            Channel::keys(
                t,
                p,
                Track::with_keys(&[(0.0, a, Ease::Linear), (10.0, b, Ease::Linear)]),
            )
        };
        let ch = vec![
            key("a::fx0", Prop::FxBlur, 0.0, 8.0),
            key("a::fx1", Prop::FxDx, 0.0, 10.0),
            key("a::fx1", Prop::FxDy, 0.0, -4.0),
            key("a::fx1", Prop::FxSoft, 0.0, 2.0),
            key("a::fx1", Prop::FxAmount, 1.0, 0.0),
            key("a::fx2", Prop::FxAmount, 0.0, 1.0),
        ];
        let l = with_fx(
            base,
            vec![
                LayerEffect::blur(0.0),
                LayerEffect::drop_shadow(0.0, 0.0, 0.0, Rgba::BLACK, 1.0),
                LayerEffect::tint(Rgba::BLACK, Rgba::WHITE, 0.0),
            ],
        );
        let p = project(vec![l], ch);
        let e = Engine::new(p.clone()).unwrap();
        let fx = &e.evaluate(5.0).nodes[0].effects;
        assert_eq!(fx[0].get(Prop::FxBlur), Some(4.0));
        assert_eq!(
            (
                fx[1].get(Prop::FxDx),
                fx[1].get(Prop::FxDy),
                fx[1].get(Prop::FxSoft),
                fx[1].get(Prop::FxAmount)
            ),
            (Some(5.0), Some(-2.0), Some(1.0), Some(0.5))
        );
        assert_eq!(fx[2].get(Prop::FxAmount), Some(0.5));
        // the picture changes over time and is a pure function of the frame
        let (a, b, c) = (img(&p, 0.0), img(&p, 5.0), img(&p, 10.0));
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_eq!(img(&p, 5.0), b);
        // values that overshoot (noise, add mode) are clamped, never invalid
        let wild = vec![Channel::keys(
            "a::fx0",
            Prop::FxBlur,
            Track::with_keys(&[(0.0, -50.0, Ease::Linear), (10.0, 1e6, Ease::Linear)]),
        )];
        let e = Engine::new(project(
            vec![with_fx(
                rect("a", 1.0, 1.0, 5.0, 5.0, "#000"),
                vec![LayerEffect::blur(1.0)],
            )],
            wild,
        ))
        .unwrap();
        for f in [0.0, 5.0, 10.0] {
            let r = e.evaluate(f).nodes[0].effects[0].get(Prop::FxBlur).unwrap();
            assert!((0.0..=MAX_BLUR).contains(&r));
        }
    }

    #[test]
    fn bad_effects_and_channels_are_rejected_not_panicked() {
        let base = || rect("a", 1.0, 1.0, 5.0, 5.0, "#000");
        for bad in [
            LayerEffect::blur(f32::NAN),
            LayerEffect::blur(-1.0),
            LayerEffect::blur(1e9),
            LayerEffect::drop_shadow(f32::INFINITY, 0.0, 0.0, Rgba::BLACK, 1.0),
            LayerEffect::tint(Rgba::BLACK, Rgba::WHITE, 5.0),
            LayerEffect::fill(Rgba::new(f32::NAN, 0.0, 0.0, 1.0), 1.0),
        ] {
            let errs = Engine::new(project(vec![with_fx(base(), vec![bad])], vec![]))
                .err()
                .expect("must be rejected");
            assert!(errs.iter().any(|e| e.contains("effect 0")), "{errs:?}");
        }
        let errs = Engine::new(project(
            vec![with_fx(
                base(),
                vec![LayerEffect::blur(1.0); MAX_EFFECTS + 1],
            )],
            vec![],
        ))
        .err()
        .unwrap();
        assert!(errs.iter().any(|e| e.contains("at most")));
        // channel to a missing effect, or to a property the effect does not have
        let k = |t: &str, p: Prop| {
            vec![Channel::keys(
                t,
                p,
                Track::with_keys(&[(0.0, 1.0, Ease::Linear)]),
            )]
        };
        assert!(Engine::new(project(
            vec![with_fx(base(), vec![LayerEffect::blur(1.0)])],
            k("a::fx3", Prop::FxBlur)
        ))
        .is_err());
        assert!(Engine::new(project(
            vec![with_fx(base(), vec![LayerEffect::blur(1.0)])],
            k("a::fx0", Prop::FxDx)
        ))
        .is_err());
        assert!(Engine::new(project(vec![base()], k("a::fx0", Prop::FxBlur))).is_err());
    }

    #[test]
    fn old_projects_load_and_empty_lists_are_not_written() {
        let p = project(vec![rect("a", 1.0, 1.0, 5.0, 5.0, "#000")], vec![]);
        let j = serde_json::to_string(&p).unwrap();
        assert!(!j.contains("effects"), "{j}");
        let back: Project = serde_json::from_str(&j).unwrap();
        assert!(back.layers[0].effects.is_empty());
        let with = project(
            vec![with_fx(
                rect("a", 1.0, 1.0, 5.0, 5.0, "#000"),
                vec![LayerEffect::blur(2.0)],
            )],
            vec![],
        );
        let j = serde_json::to_string(&with).unwrap();
        assert!(
            j.contains(r#""effects":[{"enabled":true,"type":"blur","radius":2.0}]"#),
            "{j}"
        );
        let back: Project = serde_json::from_str(&j).unwrap();
        assert_eq!(back.layers[0].effects, with.layers[0].effects);
    }

    #[test]
    fn editor_edits_are_undoable() {
        use crate::edit::{Editor, Op};
        let mut ed =
            Editor::new(project(vec![rect("a", 1.0, 1.0, 5.0, 5.0, "#000")], vec![])).unwrap();
        let fx = serde_json::to_value(vec![LayerEffect::blur(3.0)]).unwrap();
        ed.apply(Op::Set {
            path: "/layers/0/effects".into(),
            value: fx,
        })
        .unwrap();
        assert_eq!(ed.project().layers[0].effects.len(), 1);
        ed.apply(Op::Set {
            path: "/layers/0/effects/0/radius".into(),
            value: serde_json::json!(7.0),
        })
        .unwrap();
        assert_eq!(
            ed.engine().evaluate(0.0).nodes[0].effects[0].get(Prop::FxBlur),
            Some(7.0)
        );
        // an invalid edit is refused and leaves the document as it was
        assert!(ed
            .apply(Op::Set {
                path: "/layers/0/effects/0/radius".into(),
                value: serde_json::json!(-4.0)
            })
            .is_err());
        assert_eq!(
            ed.project().layers[0].effects[0].get(Prop::FxBlur),
            Some(7.0)
        );
        assert!(ed.undo() && ed.undo());
        assert!(ed.project().layers[0].effects.is_empty());
        assert!(!ed.doc()["layers"][0]
            .as_object()
            .unwrap()
            .contains_key("effects"));
        assert!(ed.redo());
        assert_eq!(ed.project().layers[0].effects.len(), 1);
    }

    #[test]
    fn float_renderer_matches_the_8_bit_raster() {
        let l = with_fx(
            rect("a", 15.0, 8.0, 25.0, 20.0, "#2288dd"),
            vec![
                LayerEffect::tint(Rgba::BLACK, Rgba::new(1.0, 0.8, 0.2, 1.0), 0.8),
                LayerEffect::drop_shadow(5.0, 4.0, 3.0, Rgba::BLACK, 0.6),
                LayerEffect::blur(2.0),
            ],
        );
        let e = Engine::new(project(vec![l], vec![])).unwrap();
        let o = RasterOptions {
            scale: 1.0,
            show_bones: false,
            transparent: true,
        };
        let (w, h, d) = render_rgba(&e, 0.0, &o);
        // compare over white: straight colour of nearly transparent pixels is quantisation noise
        let over_white = |px: &[u8]| -> Vec<u8> {
            px.chunks(4)
                .flat_map(|p| {
                    let a = p[3] as f32 / 255.0;
                    [p[0], p[1], p[2]]
                        .map(|c| (c as f32 * a + 255.0 * (1.0 - a)).round() as u8)
                        .into_iter()
                        .chain([255])
                })
                .collect()
        };
        let a = Img::new(w as usize, h as usize, over_white(&d)).unwrap();
        let f = crate::fraster::render_float(&e, 0.0, &o);
        let b = Img::new(
            f.width as usize,
            f.height as usize,
            over_white(&f.to_straight_u8()),
        )
        .unwrap();
        let ssim = crate::percep::ssim(&a, &b).unwrap();
        let mse = crate::percep::mse(&a, &b).unwrap();
        assert!(ssim > 0.995 && mse < 4.0, "ssim {ssim} mse {mse}");
    }

    // ---- SVG parity: an independent reading of the SVG filter primitives -------------------

    /// Attribute `name="…"` of the first tag `<tag …>` in `s` after byte `from`; returns value and end offset.
    fn attr(s: &str, from: usize, name: &str) -> Option<String> {
        let tag_end = s[from..].find('>')? + from;
        let key = format!(" {name}=\"");
        let i = s[from..tag_end].find(&key)? + from + key.len();
        Some(s[i..s[i..].find('"')? + i].to_string())
    }

    /// Straight 2D gaussian (not separable), edges transparent, on premultiplied RGBA in 0..1.
    fn gauss2d(src: &[f32], sigma: f32) -> Vec<f32> {
        let r = (sigma * 3.0).ceil() as i32;
        let k: Vec<f32> = (-r..=r)
            .map(|i| (-(i * i) as f32 / (2.0 * sigma * sigma)).exp())
            .collect();
        let norm: f32 = k.iter().sum();
        let mut out = vec![0.0f32; src.len()];
        for y in 0..H as i32 {
            for x in 0..W as i32 {
                for (j, kj) in k.iter().enumerate() {
                    for (i, ki) in k.iter().enumerate() {
                        let (sx, sy) = (x + i as i32 - r, y + j as i32 - r);
                        if sx < 0 || sy < 0 || sx >= W as i32 || sy >= H as i32 {
                            continue;
                        }
                        for c in 0..4 {
                            out[(y as usize * W + x as usize) * 4 + c] +=
                                src[(sy as usize * W + sx as usize) * 4 + c] * ki * kj
                                    / (norm * norm);
                        }
                    }
                }
            }
        }
        out
    }

    /// Run the primitives of an emitted `<filter>` over premultiplied RGBA with the semantics of the SVG filter spec.
    fn run_svg_filter(filter: &str, src: &[f32]) -> Vec<f32> {
        let mut named: std::collections::HashMap<String, Vec<f32>> =
            std::collections::HashMap::new();
        named.insert("SourceGraphic".into(), src.to_vec());
        let mut last = src.to_vec();
        let mut pos = 0;
        while let Some(i) = filter[pos..].find("<fe") {
            let at = pos + i;
            let tag: String = filter[at + 1..]
                .chars()
                .take_while(|c| c.is_alphanumeric())
                .collect();
            let input = |n: &str| named.get(&attr(filter, at, n).unwrap()).unwrap().clone();
            let out = match tag.as_str() {
                "feGaussianBlur" => gauss2d(
                    &input("in"),
                    attr(filter, at, "stdDeviation").unwrap().parse().unwrap(),
                ),
                "feOffset" => {
                    let (dx, dy): (i32, i32) = (
                        attr(filter, at, "dx").unwrap().parse::<f32>().unwrap() as i32,
                        attr(filter, at, "dy").unwrap().parse::<f32>().unwrap() as i32,
                    );
                    let s = input("in");
                    let mut o = vec![0.0; s.len()];
                    for y in 0..H as i32 {
                        for x in 0..W as i32 {
                            let (sx, sy) = (x - dx, y - dy);
                            if sx >= 0 && sy >= 0 && sx < W as i32 && sy < H as i32 {
                                for c in 0..4 {
                                    o[(y as usize * W + x as usize) * 4 + c] =
                                        s[(sy as usize * W + sx as usize) * 4 + c];
                                }
                            }
                        }
                    }
                    o
                }
                "feColorMatrix" => {
                    let m: Vec<f32> = attr(filter, at, "values")
                        .unwrap()
                        .split_whitespace()
                        .map(|v| v.parse().unwrap())
                        .collect();
                    let mut s = input("in");
                    for p in s.chunks_exact_mut(4) {
                        let a = p[3];
                        let c = if a > 0.0 {
                            [p[0] / a, p[1] / a, p[2] / a, a]
                        } else {
                            [0.0; 4]
                        };
                        let n: Vec<f32> = (0..4)
                            .map(|r| {
                                (m[r * 5] * c[0]
                                    + m[r * 5 + 1] * c[1]
                                    + m[r * 5 + 2] * c[2]
                                    + m[r * 5 + 3] * c[3]
                                    + m[r * 5 + 4])
                                    .clamp(0.0, 1.0)
                            })
                            .collect();
                        p.copy_from_slice(&[n[0] * n[3], n[1] * n[3], n[2] * n[3], n[3]]);
                    }
                    s
                }
                "feMerge" => {
                    let end = filter[at..].find("</feMerge>").unwrap() + at;
                    let mut acc = vec![0.0f32; src.len()];
                    let mut j = at;
                    while let Some(k) = filter[j..end].find("<feMergeNode") {
                        let n = attr(filter, j + k, "in").unwrap();
                        let layer = named.get(&n).unwrap();
                        for (a, l) in acc.chunks_exact_mut(4).zip(layer.chunks_exact(4)) {
                            let k = 1.0 - l[3];
                            for c in 0..4 {
                                a[c] = l[c] + a[c] * k;
                            }
                        }
                        j += k + 1;
                    }
                    acc
                }
                _ => {
                    pos = at + 1;
                    continue;
                }
            };
            if let Some(r) = attr(filter, at, "result") {
                named.insert(r, out.clone());
            }
            last = out;
            pos = at + 1;
        }
        last
    }

    #[test]
    fn svg_filter_semantics_match_the_raster_pass() {
        let effects = vec![
            LayerEffect::tint(
                Rgba::new(0.1, 0.0, 0.3, 1.0),
                Rgba::new(1.0, 0.9, 0.4, 1.0),
                0.7,
            ),
            LayerEffect::blur(4.0),
            LayerEffect::drop_shadow(5.0, 3.0, 4.0, Rgba::new(0.0, 0.0, 0.5, 1.0), 0.6),
            LayerEffect::fill(Rgba::new(0.9, 0.2, 0.2, 1.0), 0.25),
        ];
        let src = img(
            &project(vec![rect("a", 15.0, 8.0, 25.0, 20.0, "#2288dd")], vec![]),
            0.0,
        );
        let premul: Vec<f32> = src
            .rgba
            .chunks(4)
            .flat_map(|p| {
                let a = p[3] as f32 / 255.0;
                [
                    p[0] as f32 / 255.0 * a,
                    p[1] as f32 / 255.0 * a,
                    p[2] as f32 / 255.0 * a,
                    a,
                ]
            })
            .collect();
        let mut via_raster = premul.clone();
        apply_premul(&mut via_raster, W, H, &effects, 1.0);
        let filter = svg_filter(&effects, "t", W as f32, H as f32).unwrap();
        let via_svg = run_svg_filter(&filter, &premul);
        let to_img = |v: &[f32]| {
            let mut px = Vec::new();
            for p in v.chunks(4) {
                let a = p[3].clamp(0.0, 1.0);
                let un = |c: f32| {
                    if a > 0.0 {
                        ((c / a).clamp(0.0, 1.0) * 255.0).round() as u8
                    } else {
                        0
                    }
                };
                px.extend_from_slice(&[un(p[0]), un(p[1]), un(p[2]), (a * 255.0).round() as u8]);
            }
            Img::new(W, H, px).unwrap()
        };
        let (a, b) = (to_img(&via_raster), to_img(&via_svg));
        let ssim = crate::percep::ssim(&a, &b).unwrap();
        let mse = crate::percep::mse(&a, &b).unwrap();
        assert!(ssim > 0.99 && mse < 6.0, "ssim {ssim} mse {mse}");
    }

    #[test]
    fn svg_output_filters_only_the_layer_with_effects() {
        let p = project(
            vec![
                with_fx(
                    rect("a", 4.0, 4.0, 10.0, 10.0, "#f00"),
                    vec![LayerEffect::blur(3.0)],
                ),
                rect("b", 30.0, 4.0, 10.0, 10.0, "#00f"),
            ],
            vec![],
        );
        let e = Engine::new(p).unwrap();
        let svg = crate::svg::render_frame(&e, 0.0, &Default::default());
        assert_eq!(svg.matches("<filter").count(), 1, "{svg}");
        assert_eq!(svg.matches("filter=\"url(#fx").count(), 1);
        assert!(svg.find("filter=\"url").unwrap() < svg.find("id=\"b\"").unwrap_or(usize::MAX));
        // no effects: no filter markup at all (old output unchanged)
        let plain = Engine::new(project(
            vec![rect("a", 4.0, 4.0, 10.0, 10.0, "#f00")],
            vec![],
        ))
        .unwrap();
        assert!(!crate::svg::render_frame(&plain, 0.0, &Default::default()).contains("filter"));
        // deterministic
        assert_eq!(svg, crate::svg::render_frame(&e, 0.0, &Default::default()));
    }
}
