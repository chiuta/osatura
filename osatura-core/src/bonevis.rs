// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Per-bone visibility ranges for the bone overlay (editor/preview aid): hide a control bone
//! outside the frames where you animate it. Never affects the rig or the rendered artwork.

use crate::project::Project;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoneVis {
    pub bone: String,
    pub from: f32,
    pub to: f32,
}

/// Show `bone` in the overlay only for frames `from..=to`.
pub fn set_range(p: &mut Project, bone: &str, from: f32, to: f32) -> Result<(), String> {
    if !p.bones.iter().any(|b| b.name == bone) {
        return Err(format!("no bone '{bone}'"));
    }
    if !from.is_finite() || !to.is_finite() || to < from {
        return Err("range needs from <= to".into());
    }
    match p.bone_vis.iter_mut().find(|v| v.bone == bone) {
        Some(v) => {
            v.from = from;
            v.to = to;
        }
        None => p.bone_vis.push(BoneVis { bone: bone.into(), from, to }),
    }
    Ok(())
}

pub fn clear(p: &mut Project, bone: &str) -> bool {
    let n = p.bone_vis.len();
    p.bone_vis.retain(|v| v.bone != bone);
    p.bone_vis.len() != n
}

/// Should the overlay draw `bone` at `frame`? Bones without a range are always drawn, except
/// per-skin bones that a skin switch has collapsed at that frame.
pub fn visible(p: &Project, bone: &str, frame: f32) -> bool {
    !crate::skins::bone_collapsed(p, bone, frame) && p.bone_vis.iter().find(|v| v.bone == bone).is_none_or(|v| frame >= v.from && frame <= v.to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;
    use crate::project::Bone;
    use crate::raster::{render_rgba, RasterOptions};
    use crate::Engine;

    fn proj() -> Project {
        let mut p = Project::new("v", 60.0, 60.0, 24.0, 20);
        p.bones = vec![Bone::new("a", None, 20.0, Vec2::new(10.0, 30.0), 0.0), Bone::new("b", None, 20.0, Vec2::new(10.0, 10.0), 0.0)];
        p
    }

    #[test]
    fn range_logic_and_validation() {
        let mut p = proj();
        assert!(visible(&p, "a", 5.0));
        set_range(&mut p, "a", 4.0, 8.0).unwrap();
        assert!(!visible(&p, "a", 3.0) && visible(&p, "a", 4.0) && visible(&p, "a", 8.0) && !visible(&p, "a", 9.0));
        assert!(visible(&p, "b", 100.0));
        set_range(&mut p, "a", 0.0, 2.0).unwrap();
        assert_eq!(p.bone_vis.len(), 1, "overwrites");
        assert!(set_range(&mut p, "ghost", 0.0, 1.0).is_err() && set_range(&mut p, "a", 5.0, 1.0).is_err());
        assert!(clear(&mut p, "a") && !clear(&mut p, "a"));
        let back = Project::from_json(&{ set_range(&mut p, "b", 1.0, 2.0).unwrap(); p.to_json_pretty() }).unwrap();
        assert_eq!(back.bone_vis, p.bone_vis);
    }

    #[test]
    fn overlay_respects_ranges_in_raster_and_svg_but_artwork_is_untouched() {
        let mut p = proj();
        set_range(&mut p, "a", 0.0, 5.0).unwrap();
        let e = Engine::new(p).unwrap();
        let o = RasterOptions { scale: 1.0, show_bones: true, transparent: true };
        let (_, _, early) = render_rgba(&e, 3.0, &o);
        let (_, _, late) = render_rgba(&e, 12.0, &o);
        let px = |buf: &Vec<u8>, x: usize, y: usize| buf[(y * 60 + x) * 4 + 3];
        assert!(px(&early, 20, 30) > 0, "bone a drawn inside its range");
        assert_eq!(px(&late, 20, 30), 0, "bone a hidden outside its range");
        assert!(px(&late, 20, 10) > 0, "bone b has no range and stays");
        let svg_early = crate::svg::render_frame(&e, 3.0, &crate::svg::RenderOptions { show_bones: true });
        let svg_late = crate::svg::render_frame(&e, 12.0, &crate::svg::RenderOptions { show_bones: true });
        assert_eq!(svg_early.matches("<line").count(), 2);
        assert_eq!(svg_late.matches("<line").count(), 1);
    }
}
