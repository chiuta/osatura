// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Import a PNG image sequence as one switch layer (frame-by-frame animation) with step keys.
//! Each PNG is validated and embedded, so the project stays self-contained.

use crate::anim::{Ease, Track};
use crate::math::Vec2;
use crate::project::{Channel, Layer, Project, Prop};

const MAX_FRAMES: usize = 2000;
const MAX_BYTES: usize = 64 * 1024 * 1024;

/// `frames` are `(file name, PNG bytes)` in playback order. Each image is shown for `hold`
/// project frames starting at `start_frame`; `looped` repeats the sequence.
pub fn import_png_sequence(p: &mut Project, name: &str, frames: &[(String, Vec<u8>)], origin: Vec2, start_frame: f32, hold: f32, looped: bool) -> Result<(), String> {
    import_png_sequence_timed(p, name, frames, &vec![hold; frames.len()], origin, start_frame, looped)
}

/// Like [`import_png_sequence`] with one hold (in project frames, each >= 1) per image.
pub fn import_png_sequence_timed(p: &mut Project, name: &str, frames: &[(String, Vec<u8>)], holds: &[f32], origin: Vec2, start_frame: f32, looped: bool) -> Result<(), String> {
    if frames.is_empty() || frames.len() > MAX_FRAMES {
        return Err(format!("a sequence needs 1..{MAX_FRAMES} images"));
    }
    if holds.len() != frames.len() || holds.iter().any(|h| !matches!((*h).partial_cmp(&1.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) || !h.is_finite()) || !start_frame.is_finite() || start_frame < 0.0 {
        return Err("hold must be at least 1 frame and start_frame non-negative".into());
    }
    if p.find_layer(name).is_some() {
        return Err(format!("a layer named '{name}' already exists"));
    }
    if frames.iter().map(|f| f.1.len()).sum::<usize>() > MAX_BYTES {
        return Err("sequence is larger than 64 MB".into());
    }
    let mut kids = Vec::with_capacity(frames.len());
    let mut size: Option<(u32, u32)> = None;
    for (file, bytes) in frames {
        let pm = crate::assets::decode_png_checked(bytes).map_err(|e| format!("{file}: {e}"))?;
        let s = (pm.width(), pm.height());
        if let Some(first) = size {
            if first != s {
                return Err(format!("{file}: size {}x{} differs from the first image ({}x{})", s.0, s.1, first.0, first.1));
            }
        }
        size = Some(s);
        kids.push(Layer::image_from_png(file, bytes, origin));
    }
    let n = kids.len();
    let mut starts = Vec::with_capacity(n);
    let mut acc = start_frame;
    for h in holds {
        starts.push(acc);
        acc += h;
    }
    let cycle = acc - start_frame;
    let mut keys: Vec<(f32, f32, Ease)> = starts.iter().enumerate().map(|(i, t)| (*t, i as f32, Ease::Step)).collect();
    if looped {
        let mut t0 = start_frame + cycle;
        while t0 < p.frames as f32 {
            keys.extend(starts.iter().enumerate().map(|(i, t)| (t0 + (t - start_frame), i as f32, Ease::Step)));
            t0 += cycle;
        }
    }
    p.layers.push(Layer::switch(name, kids));
    p.channels.push(Channel::keys(name, Prop::LayerSwitch, Track::with_keys(&keys)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Engine;

    fn png(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let data: Vec<u8> = (0..w * h).flat_map(|_| rgba).collect();
        crate::open_formats::encode_rgba_png(w, h, &data).unwrap()
    }
    fn proj() -> Project {
        Project::new("s", 64.0, 64.0, 24.0, 12)
    }
    fn files() -> Vec<(String, Vec<u8>)> {
        vec![("f1.png".into(), png(8, 8, [255, 0, 0, 255])), ("f2.png".into(), png(8, 8, [0, 255, 0, 255])), ("f3.png".into(), png(8, 8, [0, 0, 255, 255]))]
    }
    /// One bool per child: true for the child the switch currently selects.
    fn shown(p: &Project, f: f32) -> Vec<bool> {
        let fr = Engine::new(p.clone()).unwrap().evaluate(f);
        let sw = fr.nodes.iter().find(|n| n.name == "seq").unwrap();
        let chosen: Vec<&str> = sw.children.iter().map(|&c| fr.nodes[c].name.as_str()).collect();
        ["seq/f1.png", "seq/f2.png", "seq/f3.png"].iter().map(|n| chosen.contains(n)).collect()
    }

    #[test]
    fn plays_frames_with_hold() {
        let mut p = proj();
        import_png_sequence(&mut p, "seq", &files(), Vec2::ZERO, 0.0, 2.0, false).unwrap();
        assert_eq!(shown(&p, 0.0), vec![true, false, false]);
        assert_eq!(shown(&p, 1.0), vec![true, false, false]);
        assert_eq!(shown(&p, 2.0), vec![false, true, false]);
        assert_eq!(shown(&p, 11.0), vec![false, false, true], "last image holds");
    }

    #[test]
    fn loops_and_validates() {
        let mut p = proj();
        import_png_sequence(&mut p, "seq", &files(), Vec2::ZERO, 0.0, 1.0, true).unwrap();
        assert_eq!(shown(&p, 3.0), vec![true, false, false]);
        assert_eq!(shown(&p, 7.0), vec![false, true, false]);
        assert!(import_png_sequence(&mut p, "seq", &files(), Vec2::ZERO, 0.0, 1.0, false).is_err(), "duplicate name");
        let mut q = proj();
        let mut mixed = files();
        mixed.push(("big.png".into(), png(9, 8, [0, 0, 0, 255])));
        assert!(import_png_sequence(&mut q, "x", &mixed, Vec2::ZERO, 0.0, 1.0, false).unwrap_err().contains("differs"));
        assert!(import_png_sequence(&mut q, "x", &[("bad.png".into(), vec![1, 2, 3])], Vec2::ZERO, 0.0, 1.0, false).is_err());
        assert!(import_png_sequence(&mut q, "x", &[], Vec2::ZERO, 0.0, 1.0, false).is_err());
        assert!(import_png_sequence(&mut q, "x", &files(), Vec2::ZERO, 0.0, 0.5, false).is_err());
        assert!(q.layers.is_empty() && q.channels.is_empty(), "failed imports change nothing");
    }
}
