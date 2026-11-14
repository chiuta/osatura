// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Magnets: pull a root controller bone toward a target point when it comes close.
//!
//! Baked as additive offset channels from per-frame sampling, so the original animation stays
//! intact and you can delete the two channels to remove the effect. The pull follows a smooth
//! falloff: full `strength` at the target, zero at `radius`.

use crate::anim::{Ease, Track};
use crate::math::Vec2;
use crate::project::{Channel, Mode, Project, Prop};
use crate::Engine;

#[derive(Clone, Debug)]
pub enum Target {
    Point(Vec2),
    /// Head of another bone, followed per frame.
    Bone(String),
}

pub struct Magnet {
    pub radius: f32,
    /// 0..1: how much of the remaining distance is closed at the target (1 = snaps fully).
    pub strength: f32,
}

fn falloff(d: f32, r: f32) -> f32 {
    let t = (1.0 - d / r).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns the number of frames in which the magnet pulled.
pub fn bake_magnet(p: &mut Project, ctrl: &str, target: &Target, m: &Magnet) -> Result<usize, String> {
    if (m.radius.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !(0.0..=1.0).contains(&m.strength) || !m.radius.is_finite() {
        return Err("radius must be positive and strength within 0..1".into());
    }
    crate::chanmeta::assert_unlocked(p, ctrl, Prop::BoneOffsetX)?;
    crate::chanmeta::assert_unlocked(p, ctrl, Prop::BoneOffsetY)?;
    let ci = p.bones.iter().position(|b| b.name == ctrl).ok_or_else(|| format!("no bone '{ctrl}'"))?;
    if p.bones[ci].parent.is_some() {
        return Err("a magnet needs a root controller bone (no parent)".into());
    }
    let ti = match target {
        Target::Bone(n) => Some(p.bones.iter().position(|b| &b.name == n).ok_or_else(|| format!("no bone '{n}'"))?),
        Target::Point(_) => None,
    };
    if ti == Some(ci) {
        return Err("target must differ from the controller".into());
    }
    let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
    let (mut kx, mut ky) = (Vec::new(), Vec::new());
    let mut pulled = 0;
    for f in 0..p.frames {
        let fr = e.evaluate(f as f32);
        let me = Vec2::new(fr.world[ci].tx, fr.world[ci].ty);
        let goal = match (ti, target) {
            (Some(i), _) => Vec2::new(fr.world[i].tx, fr.world[i].ty),
            (None, Target::Point(q)) => *q,
            _ => unreachable!(),
        };
        let d = me.distance(goal);
        let w = if d < m.radius { falloff(d, m.radius) * m.strength } else { 0.0 };
        if w > 0.0 {
            pulled += 1;
        }
        let corr = (goal - me) * w;
        kx.push((f as f32, corr.x, Ease::Linear));
        ky.push((f as f32, corr.y, Ease::Linear));
    }
    if pulled == 0 {
        return Ok(0);
    }
    p.channels.push(Channel::keys(ctrl, Prop::BoneOffsetX, Track::with_keys(&kx)).with_mode(Mode::Add));
    p.channels.push(Channel::keys(ctrl, Prop::BoneOffsetY, Track::with_keys(&ky)).with_mode(Mode::Add));
    Ok(pulled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Bone;

    fn moving() -> Project {
        let mut p = Project::new("m", 300.0, 100.0, 24.0, 21);
        p.bones = vec![Bone::new("hand", None, 5.0, Vec2::new(0.0, 50.0), 0.0), Bone::new("cup", None, 5.0, Vec2::new(100.0, 50.0), 0.0)];
        p.channels = vec![Channel::keys("hand", Prop::BoneOffsetX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (20.0, 200.0, Ease::Linear)]))];
        p
    }
    fn pos(p: &Project, f: f32) -> Vec2 {
        let w = Engine::new(p.clone()).unwrap().evaluate(f).world[0];
        Vec2::new(w.tx, w.ty)
    }

    #[test]
    fn full_strength_snaps_at_target_and_fades_out() {
        let mut p = moving();
        let n = bake_magnet(&mut p, "hand", &Target::Bone("cup".into()), &Magnet { radius: 30.0, strength: 1.0 }).unwrap();
        assert!(n > 0);
        assert!((pos(&p, 10.0).x - 100.0).abs() < 0.01, "hand passes x=100 at frame 10 and is pinned there");
        assert!((pos(&p, 0.0).x).abs() < 0.01 && (pos(&p, 20.0).x - 200.0).abs() < 0.01, "far away: untouched");
        assert!(pos(&p, 8.0).x > 80.0 && pos(&p, 8.0).x < 100.0, "pulled forward but not snapped");
    }

    #[test]
    fn partial_strength_and_point_target() {
        let mut p = moving();
        bake_magnet(&mut p, "hand", &Target::Point(Vec2::new(100.0, 70.0)), &Magnet { radius: 40.0, strength: 0.5 }).unwrap();
        let q = pos(&p, 10.0);
        assert!((q.y - 55.0).abs() < 0.01, "distance 20 of radius 40: falloff 0.5 x strength 0.5 = 25% of 20 px");
    }

    #[test]
    fn no_pull_adds_nothing_and_bad_input_fails() {
        let mut p = moving();
        assert_eq!(bake_magnet(&mut p, "hand", &Target::Point(Vec2::new(1000.0, 1000.0)), &Magnet { radius: 10.0, strength: 1.0 }).unwrap(), 0);
        assert_eq!(p.channels.len(), 1);
        assert!(bake_magnet(&mut p, "hand", &Target::Bone("hand".into()), &Magnet { radius: 10.0, strength: 1.0 }).is_err());
        assert!(bake_magnet(&mut p, "ghost", &Target::Point(Vec2::ZERO), &Magnet { radius: 10.0, strength: 1.0 }).is_err());
        assert!(bake_magnet(&mut p, "hand", &Target::Point(Vec2::ZERO), &Magnet { radius: 0.0, strength: 1.0 }).is_err());
        assert!(bake_magnet(&mut p, "hand", &Target::Point(Vec2::ZERO), &Magnet { radius: 5.0, strength: 2.0 }).is_err());
    }
}
