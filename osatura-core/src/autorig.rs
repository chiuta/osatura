// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tag-based auto-rig: from bone/layer names recognised by [`crate::vocab`], add a
//! non-destructive starter rig — IK chains for arms and legs (mix 0 = FK pose unchanged),
//! idle breathing on the torso and seeded blinking on eye layers.
//!
//! Everything added is ordinary project data; nothing existing is modified or removed.

use crate::math::Vec2;
use crate::procgen::{blink_channel, breath_channel, BlinkOptions};
use crate::project::{Bone, IkChain, Prop, Project};
use crate::rigedit::rest_world;
use crate::vocab::tag_rig;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AutoRigReport {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
}

fn bone_with_role<'a>(tags: &'a [crate::vocab::Tag], role: &str) -> Option<&'a str> {
    tags.iter().filter(|t| t.kind == "bone" && t.role == role).max_by(|a, b| a.confidence.total_cmp(&b.confidence)).map(|t| t.name.as_str())
}

/// Add IK (mix 0), breathing and blinking where the named parts exist.
pub fn auto_rig(p: &mut Project, seed: u64) -> Result<AutoRigReport, String> {
    let mut rep = AutoRigReport::default();
    let tags = tag_rig(p);
    let world = rest_world(p)?;

    for (limb, upper_r, lower_r, bend) in [("arm", "arm.upper", "arm.lower", 1.0f32), ("leg", "leg.upper", "leg.lower", -1.0)] {
        for side in ["l", "r"] {
            let label = format!("{limb}_{side}");
            let (Some(up), Some(lo)) = (bone_with_role(&tags, &format!("{upper_r}.{side}")), bone_with_role(&tags, &format!("{lower_r}.{side}"))) else {
                rep.skipped.push(format!("{label}: upper/lower bone not found"));
                continue;
            };
            if p.ik.iter().any(|c| c.bones.first().map(String::as_str) == Some(up) && c.bones.last().map(String::as_str) == Some(lo)) {
                rep.skipped.push(format!("{label}: already has an IK chain"));
                continue;
            }
            let (Some(ui), Some(li)) = (p.bones.iter().position(|b| b.name == up), p.bones.iter().position(|b| b.name == lo)) else { continue };
            if p.bones[li].parent.as_deref() != Some(up) {
                rep.skipped.push(format!("{label}: '{lo}' is not a child of '{up}'"));
                continue;
            }
            let ctrl = format!("{limb}_{side}_ctrl");
            if p.bones.iter().any(|b| b.name == ctrl) || p.ik.iter().any(|c| c.name == label) {
                rep.skipped.push(format!("{label}: name already in use"));
                continue;
            }
            let _ = ui;
            let (head, ang) = world[li];
            let a = ang.to_radians();
            let tip = head + Vec2::new(a.cos(), a.sin()) * p.bones[li].length;
            p.bones.push(Bone::new(&ctrl, None, 10.0, tip, 0.0));
            let mut chain = IkChain::two_bone(&label, up, lo, &ctrl, bend);
            chain.mix = 0.0;
            p.ik.push(chain);
            rep.created.push(format!("IK {label} -> {ctrl} (mix 0, bend {bend:+})"));
        }
    }

    if let Some(t) = bone_with_role(&tags, "torso") {
        let ch = breath_channel(t, Prop::BoneOffsetY, 1.5, p.fps.max(2.0) * 4.0)?;
        p.channels.push(ch);
        rep.created.push(format!("breathing on {t}"));
    } else {
        rep.skipped.push("breathing: no torso bone".into());
    }

    let eyes: Vec<&crate::vocab::Tag> = tags.iter().filter(|t| t.kind == "layer" && (t.role == "face.eye.l" || t.role == "face.eye.r")).collect();
    if eyes.is_empty() {
        rep.skipped.push("blink: no eye layers".into());
    }
    for e in eyes {
        // same seed on both eyes so they blink together
        let o = BlinkOptions { seed, ..BlinkOptions::default() };
        match blink_channel(&e.name, Prop::LayerScaleY, p.frames, p.fps, &o) {
            Ok(ch) => {
                p.channels.push(ch);
                rep.created.push(format!("blink on {}", e.name));
            }
            Err(m) => rep.skipped.push(format!("blink {}: {m}", e.name)),
        }
    }
    let clip = crate::eyeclip::auto_eye_clip(p);
    rep.created.extend(clip.clipped.iter().map(|c| format!("eye clip {c}")));
    rep.skipped.extend(clip.skipped.iter().map(|c| format!("eye clip {c}")));
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::VPath;
    use crate::project::Layer;
    use crate::Engine;

    fn humanoid() -> Project {
        let mut p = Project::new("h", 400.0, 400.0, 24.0, 120);
        p.bones = vec![
            Bone::new("torso", None, 60.0, Vec2::new(200.0, 200.0), -90.0),
            Bone::new("arm_upper_L", Some("torso"), 40.0, Vec2::new(60.0, -10.0), 90.0),
            Bone::new("arm_lower_L", Some("arm_upper_L"), 35.0, Vec2::new(40.0, 0.0), 0.0),
            Bone::new("thigh_R", None, 50.0, Vec2::new(210.0, 260.0), 90.0),
            Bone::new("shin_R", Some("thigh_R"), 50.0, Vec2::new(50.0, 0.0), 0.0),
        ];
        p.layers = vec![
            Layer::vector("eye_L", vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#000000")]),
            Layer::vector("eye_R", vec![VPath::rect("r", 20.0, 0.0, 10.0, 10.0).with_fill("#000000")]),
        ];
        p
    }

    #[test]
    fn builds_valid_nondestructive_rig() {
        let mut p = humanoid();
        let before = Engine::new(p.clone()).unwrap().evaluate(3.0);
        let rep = auto_rig(&mut p, 7).unwrap();
        assert!(rep.created.iter().any(|s| s.contains("arm_l")), "{rep:?}");
        assert!(rep.created.iter().any(|s| s.contains("leg_r")), "{rep:?}");
        assert!(rep.skipped.iter().any(|s| s.contains("arm_r")));
        assert_eq!(p.ik.len(), 2);
        assert!(p.ik.iter().all(|c| c.mix == 0.0));
        let after = Engine::new(p.clone()).unwrap().evaluate(0.0);
        // IK at mix 0 leaves FK pose untouched (breath is additive offset on torso only at frame 0 = 0)
        let b0 = Engine::new(humanoid()).unwrap().evaluate(0.0);
        for (a, b) in after.world.iter().zip(b0.world.iter()).take(5) {
            assert!((a.rotation() - b.rotation()).abs() < 1e-3);
        }
        let _ = before;
    }

    #[test]
    fn rerun_is_idempotent_for_ik_and_blinks_match() {
        let mut p = humanoid();
        auto_rig(&mut p, 3).unwrap();
        let n = p.ik.len();
        let rep = auto_rig(&mut p, 3).unwrap();
        assert_eq!(p.ik.len(), n);
        assert!(rep.skipped.iter().any(|s| s.contains("already has an IK")));
        let blinks: Vec<_> = p.channels.iter().filter(|c| c.prop == Prop::LayerScaleY).collect();
        assert!(blinks.len() >= 2);
        assert_eq!(format!("{:?}", blinks[0].source), format!("{:?}", blinks[1].source), "eyes share the seed");
    }

    #[test]
    fn empty_project_reports_skips() {
        let mut p = Project::new("e", 100.0, 100.0, 24.0, 10);
        let rep = auto_rig(&mut p, 1).unwrap();
        assert!(rep.created.is_empty());
        assert!(rep.skipped.len() >= 6);
    }
}
