// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Animated re-parenting, baked: a bone can be carried by different bones over time
//! (a hand holding a prop, then letting go), without any jump at the switch frame.
//!
//! At each switch frame the bone's current world pose relative to the new parent is captured;
//! from then on it keeps that relative pose (rigid follow), or stays frozen in the world when
//! the new parent is `none`. The bone is made a root and its world pose is written as plain
//! offset/angle keys for every frame. Its own animation channels remain in the file but are
//! overridden by the baked ones, which come later in the list. Scale is not carried.

use crate::anim::{Ease, Track};
use crate::math::Affine2;
use crate::project::{Channel, Project, Prop};
use crate::rigedit::set_parent;
use crate::Engine;

pub struct Switch {
    pub frame: u32,
    /// Bone name, or `None` to freeze the bone in world space.
    pub parent: Option<String>,
}

fn angle_deg(m: &Affine2) -> f32 {
    m.b.atan2(m.a).to_degrees()
}

pub fn bake_parent_switch(p: &mut Project, bone: &str, steps: &[Switch]) -> Result<(), String> {
    if steps.is_empty() {
        return Err("need at least one switch".into());
    }
    let bi = p.bones.iter().position(|b| b.name == bone).ok_or_else(|| format!("no bone '{bone}'"))?;
    let mut st: Vec<&Switch> = steps.iter().collect();
    st.sort_by_key(|s| s.frame);
    if st.windows(2).any(|w| w[0].frame == w[1].frame) || st.iter().any(|s| s.frame >= p.frames) {
        return Err("switch frames must be distinct and inside the timeline".into());
    }
    if p.ik.iter().any(|c| c.bones.iter().any(|b| b == bone) || c.target == bone) || p.constraints.iter().any(|c| c.bone == bone) || p.springs.iter().any(|s| s.bone == bone) {
        return Err(format!("'{bone}' takes part in IK, a constraint or a spring; bake would not be faithful"));
    }
    for pr in [Prop::BoneOffsetX, Prop::BoneOffsetY, Prop::BoneAngle] {
        crate::chanmeta::assert_unlocked(p, bone, pr)?;
    }
    // parents: indices, must not be the bone or its descendants
    let mut parents: Vec<Option<usize>> = Vec::new();
    for s in &st {
        match &s.parent {
            None => parents.push(None),
            Some(n) => {
                let pi = p.bones.iter().position(|b| &b.name == n).ok_or_else(|| format!("no bone '{n}'"))?;
                let mut cur = Some(pi);
                while let Some(c) = cur {
                    if c == bi {
                        return Err(format!("'{n}' is '{bone}' or one of its descendants"));
                    }
                    cur = p.bones[c].parent.as_ref().and_then(|x| p.bones.iter().position(|b| &b.name == x));
                }
                parents.push(Some(pi));
            }
        }
    }
    // 1. original world poses (before restructuring)
    let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
    let worlds: Vec<Vec<Affine2>> = (0..p.frames).map(|f| e.evaluate(f as f32).world).collect();

    // 2. world target of the bone for every frame. At a switch the bone's *current* pose (what it
    //    does under the previous segment) is captured relative to the new parent: no jump.
    let follow = |seg: Option<usize>, rel: Affine2, f: usize| -> Affine2 {
        match seg {
            Some(k) => match parents[k] {
                Some(pi) => worlds[f][pi].mul(rel),
                None => rel,
            },
            None => worlds[f][bi],
        }
    };
    let mut target: Vec<Affine2> = Vec::with_capacity(p.frames as usize);
    let mut seg: Option<usize> = None;
    let mut rel = Affine2::IDENTITY;
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for f in 0..p.frames as usize {
        let next = seg.map_or(0, |s| s + 1);
        if next < st.len() && st[next].frame as usize == f {
            let current = follow(seg, rel, f);
            rel = match parents[next] {
                Some(pi) => worlds[f][pi].inverse().ok_or("parent has a degenerate transform")?.mul(current),
                None => current,
            };
            seg = Some(next);
        }
        target.push(follow(seg, rel, f));
    }

    // 3. make it a root (rest pose preserved), then write the baked keys
    set_parent(p, bone, None)?;
    let bi = p.bones.iter().position(|b| b.name == bone).ok_or("bone vanished")?;
    let (rest_pos, rest_ang) = (p.bones[bi].rest_offset, p.bones[bi].rest_angle);
    let (mut kx, mut ky, mut ka) = (Vec::new(), Vec::new(), Vec::new());
    let mut last = f32::NAN;
    for (f, m) in target.iter().enumerate() {
        let mut a = angle_deg(m) - rest_ang;
        if last.is_finite() {
            while a - last > 180.0 {
                a -= 360.0;
            }
            while a - last < -180.0 {
                a += 360.0;
            }
        }
        last = a;
        kx.push((f as f32, m.tx - rest_pos.x, Ease::Linear));
        ky.push((f as f32, m.ty - rest_pos.y, Ease::Linear));
        ka.push((f as f32, a, Ease::Linear));
    }
    p.channels.push(Channel::keys(bone, Prop::BoneOffsetX, Track::with_keys(&kx)));
    p.channels.push(Channel::keys(bone, Prop::BoneOffsetY, Track::with_keys(&ky)));
    p.channels.push(Channel::keys(bone, Prop::BoneAngle, Track::with_keys(&ka)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::{Ease, Track};
    use crate::math::Vec2;
    use crate::project::Bone;

    /// arm (rotating) carries hand; prop slides to the right along x.
    fn rig() -> Project {
        let mut p = Project::new("ps", 400.0, 200.0, 24.0, 30);
        p.bones = vec![
            Bone::new("arm", None, 50.0, Vec2::new(100.0, 100.0), 0.0),
            Bone::new("hand", Some("arm"), 10.0, Vec2::new(50.0, 0.0), 0.0),
            Bone::new("prop", None, 10.0, Vec2::new(150.0, 150.0), 0.0),
        ];
        p.channels = vec![
            Channel::keys("arm", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (29.0, 60.0, Ease::Linear)])),
            Channel::keys("prop", Prop::BoneOffsetX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (29.0, 200.0, Ease::Linear)])),
        ];
        p
    }
    fn hand(p: &Project, f: f32) -> Vec2 {
        let i = p.bones.iter().position(|b| b.name == "hand").unwrap();
        let w = Engine::new(p.clone()).unwrap().evaluate(f).world[i];
        Vec2::new(w.tx, w.ty)
    }
    fn prop(p: &Project, f: f32) -> Vec2 {
        let i = p.bones.iter().position(|b| b.name == "prop").unwrap();
        let w = Engine::new(p.clone()).unwrap().evaluate(f).world[i];
        Vec2::new(w.tx, w.ty)
    }

    #[test]
    fn hand_switches_to_prop_without_a_jump() {
        let orig = rig();
        let mut p = rig();
        bake_parent_switch(&mut p, "hand", &[Switch { frame: 10, parent: Some("prop".into()) }]).unwrap();
        for f in [0.0, 5.0, 9.0, 10.0] {
            assert!(hand(&p, f).distance(hand(&orig, f)) < 0.05, "frame {f}: unchanged up to and at the switch");
        }
        let gap10 = hand(&p, 10.0) - prop(&p, 10.0);
        for f in [15.0, 29.0] {
            let gap = hand(&p, f) - prop(&p, f);
            assert!((gap - gap10).length() < 0.05, "frame {f}: keeps its offset from the prop");
        }
        assert!(hand(&p, 29.0).distance(hand(&orig, 29.0)) > 50.0, "no longer follows the arm");
        assert!(p.bones.iter().find(|b| b.name == "hand").unwrap().parent.is_none());
    }

    #[test]
    fn release_freezes_in_world_and_can_regrab() {
        let orig = rig();
        let mut p = rig();
        bake_parent_switch(&mut p, "hand", &[Switch { frame: 8, parent: None }, Switch { frame: 20, parent: Some("arm".into()) }]).unwrap();
        let held = hand(&p, 8.0);
        assert!(hand(&p, 14.0).distance(held) < 0.05 && hand(&p, 19.0).distance(held) < 0.05, "frozen between 8 and 20");
        assert!(hand(&p, 25.0).distance(hand(&orig, 25.0)) > 1.0, "re-grabbed arm at the pose it had when frozen, so it differs from the original path");
        let g20 = hand(&p, 20.0) - held;
        assert!(g20.length() < 0.05, "no jump at the regrab");
    }

    #[test]
    fn refuses_unsafe_requests() {
        let mut p = rig();
        assert!(bake_parent_switch(&mut p, "hand", &[]).is_err());
        assert!(bake_parent_switch(&mut p, "arm", &[Switch { frame: 3, parent: Some("hand".into()) }]).is_err(), "descendant");
        assert!(bake_parent_switch(&mut p, "hand", &[Switch { frame: 99, parent: None }]).is_err());
        assert!(bake_parent_switch(&mut p, "hand", &[Switch { frame: 3, parent: Some("ghost".into()) }]).is_err());
        p.ik.push(crate::project::IkChain::two_bone("i", "arm", "hand", "prop", 1.0));
        assert!(bake_parent_switch(&mut p, "hand", &[Switch { frame: 3, parent: None }]).is_err(), "IK member");
        assert_eq!(p.bones[1].parent.as_deref(), Some("arm"), "failed calls change nothing");
    }
}
