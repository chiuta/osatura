// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Depth of field by layers: each top-level layer is rendered on its own, blurred in proportion to
//! how far its `parallax` depth is from the focus depth, and the results are stacked in layer
//! order. Own design (a layered "circle of confusion" approximation using Gaussian blur): there
//! is no per-pixel depth, so a single layer is always uniformly sharp or blurred.

use crate::project::Project;
use crate::raster::RasterOptions;
use crate::Engine;

pub struct DofOptions {
    pub scale: f32,
    /// `parallax` value that is in focus.
    pub focus: f32,
    /// Blur radius in output pixels per unit of parallax distance from the focus.
    pub strength: f32,
    /// Upper limit for the radius, output pixels.
    pub max_radius: f32,
}

impl Default for DofOptions {
    fn default() -> Self {
        DofOptions {
            scale: 1.0,
            focus: 1.0,
            strength: 12.0,
            max_radius: 24.0,
        }
    }
}

/// Blur radius a layer at `parallax` gets.
pub fn radius_for(o: &DofOptions, parallax: f32) -> f32 {
    ((parallax - o.focus).abs() * o.strength).min(o.max_radius)
}

/// `(width, height, straight RGBA)` of the frame with depth of field applied.
///
/// The frame is composited by the normal rasteriser: every top-level layer is blurred as a whole
/// (after its effects and mask, before its own opacity and blend mode), so blend modes, group
/// opacity and masks behave as in `render`; with `strength 0` the result equals `render`.
pub fn render_dof(p: &Project, frame: f32, o: &DofOptions) -> Result<(u32, u32, Vec<u8>), String> {
    if (o.scale.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater))
        || !o.focus.is_finite()
        || !matches!(
            o.strength.partial_cmp(&0.0),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        )
        || !matches!(
            o.max_radius.partial_cmp(&0.0),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        )
    {
        return Err("invalid depth-of-field options".into());
    }
    let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
    let f = e.evaluate(frame);
    let ro = RasterOptions {
        scale: o.scale,
        show_bones: false,
        transparent: false,
    };
    let mut blur = std::collections::HashMap::new();
    for &r in &f.roots {
        let rad = radius_for(o, e.link_root_parallax(r));
        if rad >= 0.5 {
            blur.insert(r, rad);
        }
    }
    let pm = crate::raster::pixmap_from_frame_blur(&e, &f, frame, &ro, Some(&blur));
    Ok((
        pm.width(),
        pm.height(),
        crate::raster::to_straight_rgba(&pm),
    ))
}

/// Options for frame `frame`: the camera's animated `focus` / `strength` (channels
/// `camera_focus`, `camera_blur`) unless the caller pinned them (`fixed_*`); scale and max radius
/// always come from `base`.
pub fn options_at(
    e: &Engine,
    frame: f32,
    base: &DofOptions,
    fixed_focus: bool,
    fixed_strength: bool,
) -> DofOptions {
    let (f, s) = e.dof_at(frame);
    DofOptions {
        scale: base.scale,
        focus: if fixed_focus { base.focus } else { f },
        strength: if fixed_strength { base.strength } else { s },
        max_radius: base.max_radius,
    }
}

/// Depth-of-field render of one frame with focus/strength taken from the camera (see `options_at`).
pub fn render_dof_animated(
    p: &Project,
    frame: f32,
    base: &DofOptions,
    fixed_focus: bool,
    fixed_strength: bool,
) -> Result<(u32, u32, Vec<u8>), String> {
    let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
    render_dof(
        p,
        frame,
        &options_at(&e, frame, base, fixed_focus, fixed_strength),
    )
}

/// Largest frame count a depth-of-field export accepts (every frame renders each layer alone).
pub const MAX_EXPORT_FRAMES: u32 = 2000;

/// Frames `from..=to` (whole frames, `to` defaults to the last) rendered with animated depth of
/// field. Returns `(width, height, frames)`.
pub fn dof_frames(
    p: &Project,
    from: u32,
    to: Option<u32>,
    base: &DofOptions,
    fixed_focus: bool,
    fixed_strength: bool,
) -> Result<(u32, u32, Vec<Vec<u8>>), String> {
    let to = to.unwrap_or(p.frames.saturating_sub(1));
    if p.frames == 0 || to >= p.frames {
        return Err(format!(
            "--to {to} is outside the timeline (0..={})",
            p.frames.saturating_sub(1)
        ));
    }
    if from > to {
        return Err(format!("--from {from} is after --to {to}"));
    }
    if to - from + 1 > MAX_EXPORT_FRAMES {
        return Err(format!("at most {MAX_EXPORT_FRAMES} frames per export"));
    }
    let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
    let (mut w, mut h) = (0, 0);
    let mut out = Vec::new();
    for f in from..=to {
        let o = options_at(&e, f as f32, base, fixed_focus, fixed_strength);
        let (fw, fh, buf) = render_dof(p, f as f32, &o)?;
        (w, h) = (fw, fh);
        out.push(buf);
    }
    Ok((w, h, out))
}

/// Looping GIF of the depth-of-field frames (shared GIF encoder).
pub fn dof_gif(
    p: &Project,
    from: u32,
    to: Option<u32>,
    base: &DofOptions,
    ff: bool,
    fs: bool,
) -> Result<Vec<u8>, String> {
    let (w, h, frames) = dof_frames(p, from, to, base, ff, fs)?;
    if w > u16::MAX as u32 || h > u16::MAX as u32 {
        return Err("image too large for GIF".into());
    }
    crate::export::gif_from_frames(w, h, frames, p.fps)
}

/// Looping animated PNG of the depth-of-field frames (shared APNG encoder, keeps alpha).
pub fn dof_apng(
    p: &Project,
    from: u32,
    to: Option<u32>,
    base: &DofOptions,
    ff: bool,
    fs: bool,
) -> Result<Vec<u8>, String> {
    let (w, h, frames) = dof_frames(p, from, to, base, ff, fs)?;
    crate::open_formats::apng_from_frames(w, h, &frames, p.fps)
}

/// One PNG per frame (alpha kept).
pub fn dof_png_sequence(
    p: &Project,
    from: u32,
    to: Option<u32>,
    base: &DofOptions,
    ff: bool,
    fs: bool,
) -> Result<Vec<Vec<u8>>, String> {
    let (w, h, frames) = dof_frames(p, from, to, base, ff, fs)?;
    frames
        .iter()
        .map(|f| crate::open_formats::encode_rgba_png(w, h, f))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::VPath;
    use crate::project::Layer;
    use crate::raster::render_rgba;

    fn proj() -> Project {
        let mut p = Project::new("d", 40.0, 20.0, 24.0, 1);
        p.background = None;
        let mut far = Layer::vector(
            "far",
            vec![VPath::rect("r", 0.0, 0.0, 20.0, 20.0).with_fill("#00ff00")],
        );
        far.parallax = 0.0;
        let near = Layer::vector(
            "near",
            vec![VPath::rect("r", 20.0, 0.0, 20.0, 20.0).with_fill("#ff0000")],
        );
        p.layers = vec![far, near];
        p
    }

    #[test]
    fn radius_grows_with_distance() {
        let o = DofOptions::default();
        assert_eq!(radius_for(&o, 1.0), 0.0);
        assert!(radius_for(&o, 0.0) > radius_for(&o, 0.5));
        assert_eq!(
            radius_for(
                &DofOptions {
                    max_radius: 3.0,
                    ..Default::default()
                },
                0.0
            ),
            3.0
        );
    }

    #[test]
    fn out_of_focus_layer_is_soft_and_focused_one_is_sharp() {
        let p = proj();
        let alpha = |b: &[u8], x: usize| b[(10 * 40 + x) * 4 + 3];
        let (_, _, sharp) = render_dof(
            &p,
            0.0,
            &DofOptions {
                strength: 0.0,
                ..Default::default()
            },
        )
        .unwrap();
        let (_, _, soft) = render_dof(&p, 0.0, &DofOptions::default()).unwrap();
        assert_eq!(alpha(&sharp, 19), 255);
        // the far (green) layer's outer-left border fades when it is out of focus
        assert!(
            alpha(&soft, 0) < 255 && alpha(&sharp, 0) == 255,
            "far layer edge fades"
        );
        assert_eq!(alpha(&soft, 21), 255, "focused layer unchanged");
        assert_eq!(soft[(10 * 40 + 30) * 4], 255, "red stays red");
    }

    use crate::anim::{Ease, Track};
    use crate::project::{Channel, Prop};

    fn key_channel(prop: Prop, keys: &[(f32, f32)]) -> Channel {
        let k: Vec<(f32, f32, Ease)> = keys.iter().map(|&(t, v)| (t, v, Ease::Linear)).collect();
        Channel::keys("", prop, Track::with_keys(&k))
    }

    /// far (parallax 0, green left) and near (parallax 1, red right); focus goes from near to far.
    fn rack_focus() -> Project {
        let mut p = proj();
        p.frames = 5;
        p.channels
            .push(key_channel(Prop::CameraFocus, &[(0.0, 1.0), (4.0, 0.0)]));
        p
    }

    fn alpha(b: &[u8], x: usize) -> u8 {
        b[(10 * 40 + x) * 4 + 3]
    }

    #[test]
    fn animated_focus_swaps_which_layer_is_soft() {
        let p = rack_focus();
        let base = DofOptions::default();
        let (_, _, f0) = render_dof_animated(&p, 0.0, &base, false, false).unwrap();
        let (_, _, f4) = render_dof_animated(&p, 4.0, &base, false, false).unwrap();
        // frame 0: focus 1 -> far layer (left edge x=0) soft, near layer (right edge x=39) sharp
        assert!(alpha(&f0, 0) < 255 && alpha(&f0, 39) == 255);
        // frame 4: focus 0 -> far layer sharp, near layer soft
        assert!(alpha(&f4, 0) == 255 && alpha(&f4, 39) < 255);
        // in between, both are partly soft and the far layer sharpens monotonically
        let (_, _, f2) = render_dof_animated(&p, 2.0, &base, false, false).unwrap();
        assert!(alpha(&f2, 0) > alpha(&f0, 0) && alpha(&f2, 0) <= alpha(&f4, 0));
        assert!(alpha(&f2, 39) < 255);
    }

    #[test]
    fn pinned_focus_ignores_the_animation() {
        let p = rack_focus();
        let pinned = DofOptions {
            focus: 1.0,
            ..Default::default()
        };
        let (_, _, f4) = render_dof_animated(&p, 4.0, &pinned, true, false).unwrap();
        assert!(alpha(&f4, 0) < 255 && alpha(&f4, 39) == 255);
    }

    #[test]
    fn animated_blur_strength() {
        let mut p = proj();
        p.channels
            .push(key_channel(Prop::CameraBlur, &[(0.0, 0.0), (4.0, 12.0)]));
        let base = DofOptions::default();
        let e = Engine::new(p.clone()).unwrap();
        assert_eq!(options_at(&e, 0.0, &base, false, false).strength, 0.0);
        assert_eq!(options_at(&e, 4.0, &base, false, false).strength, 12.0);
        let (_, _, a) = render_dof_animated(&p, 0.0, &base, false, false).unwrap();
        assert_eq!(alpha(&a, 0), 255, "strength 0 means nothing is blurred");
    }

    #[test]
    fn exported_gif_apng_and_sequence_have_the_right_frame_count() {
        let p = rack_focus();
        let base = DofOptions::default();
        let gif = dof_gif(&p, 1, Some(3), &base, false, false).unwrap();
        let mut dec = gif::DecodeOptions::new();
        dec.set_color_output(gif::ColorOutput::RGBA);
        let mut r = dec.read_info(&gif[..]).unwrap();
        let mut n = 0;
        while r.read_next_frame().unwrap().is_some() {
            n += 1;
        }
        assert_eq!(n, 3);
        let apng = dof_apng(&p, 0, None, &base, false, false).unwrap();
        let d = png::Decoder::new(std::io::Cursor::new(&apng[..]));
        let r = d.read_info().unwrap();
        assert_eq!(r.info().animation_control().unwrap().num_frames, 5);
        assert_eq!(
            dof_png_sequence(&p, 2, Some(3), &base, false, false)
                .unwrap()
                .len(),
            2
        );
    }

    fn max_diff(a: &[u8], b: &[u8]) -> i32 {
        a.iter()
            .zip(b)
            .map(|(x, y)| (*x as i32 - *y as i32).abs())
            .max()
            .unwrap_or(0)
    }

    #[test]
    fn strength_zero_equals_render_on_the_demo() {
        let p = crate::demo::project();
        let e = Engine::new(p.clone()).unwrap();
        for f in [0.0, 7.0] {
            let (w, h, want) = render_rgba(&e, f, &RasterOptions::default());
            let (dw, dh, got) = render_dof(
                &p,
                f,
                &DofOptions {
                    strength: 0.0,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!((w, h), (dw, dh));
            assert!(max_diff(&want, &got) <= 1, "frame {f}");
        }
    }

    #[test]
    fn blend_modes_survive_depth_of_field() {
        use crate::style::Blend;
        let mut p = Project::new("b", 40.0, 20.0, 24.0, 1);
        p.background = Some(crate::style::Rgba::parse("#808080").unwrap());
        let mut a = Layer::vector(
            "a",
            vec![VPath::rect("r", 0.0, 0.0, 30.0, 20.0).with_fill("#ff8000")],
        );
        a.blend = Blend::Screen;
        let mut b = Layer::vector(
            "b",
            vec![VPath::rect("r", 10.0, 0.0, 30.0, 20.0).with_fill("#4080ff")],
        );
        b.blend = Blend::Multiply;
        b.parallax = 0.0;
        let mut c = Layer::vector(
            "c",
            vec![VPath::rect("r", 5.0, 5.0, 10.0, 10.0).with_fill("#ffffff")],
        );
        c.opacity = 0.5;
        p.layers = vec![a, b, c];
        let e = Engine::new(p.clone()).unwrap();
        let (_, _, want) = render_rgba(&e, 0.0, &RasterOptions::default());
        let (_, _, got) = render_dof(
            &p,
            0.0,
            &DofOptions {
                strength: 0.0,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            max_diff(&want, &got) <= 1,
            "strength 0 must equal render with screen + multiply + opacity"
        );
        // with blur the multiply layer is still multiplied (not normal-composited): deep inside it stays the product colour
        let (_, _, soft) = render_dof(&p, 0.0, &DofOptions::default()).unwrap();
        let px = |b: &[u8], x: usize, y: usize| {
            [
                b[(y * 40 + x) * 4],
                b[(y * 40 + x) * 4 + 1],
                b[(y * 40 + x) * 4 + 2],
            ]
        };
        let a_only = px(&want, 35, 10); // multiply layer over background only
        let blurred = px(&soft, 35, 10);
        assert_eq!(a_only[2], 128);
        assert!(blurred[2] <= 0x84, "multiply keeps blue at the backdrop's 0x80, a normal composite would be brighter: {blurred:?}");
        assert!(blurred[0] < 0x80 && blurred[1] < 0x80, "{blurred:?}");
    }

    #[test]
    fn export_validation_errors() {
        let p = rack_focus();
        let b = DofOptions::default();
        assert!(dof_frames(&p, 3, Some(1), &b, false, false)
            .unwrap_err()
            .contains("after"));
        assert!(dof_frames(&p, 0, Some(9), &b, false, false)
            .unwrap_err()
            .contains("outside"));
        let mut bad = p.clone();
        bad.camera.focus = f32::NAN;
        assert!(Engine::new(bad).is_err());
        let mut neg = p.clone();
        neg.camera.blur = -1.0;
        assert!(Engine::new(neg).is_err());
    }

    #[test]
    fn rejects_bad_options() {
        assert!(render_dof(
            &proj(),
            0.0,
            &DofOptions {
                scale: 0.0,
                ..Default::default()
            }
        )
        .is_err());
    }
}
