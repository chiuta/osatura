// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Leader/follower limb stretch: a follower bone aims at the head of a leader bone and is scaled
//! so that its tip reaches it (within limits). Baked per frame as an additive angle channel and a
//! multiplying scale channel, so the original animation stays and removing the two channels
//! undoes the effect. The scale is uniform (the bone's `scale` is a single number), so the
//! follower's own children are scaled with it.

use crate::anim::{Ease, Track};
use crate::project::{Channel, Mode, Project, Prop};
use crate::Engine;

pub struct StretchOptions {
    /// Lowest scale factor applied to the follower (1 = never shrink).
    pub min_factor: f32,
    /// Highest scale factor (limits how far a limb can stretch).
    pub max_factor: f32,
}

impl Default for StretchOptions {
    fn default() -> Self {
        StretchOptions {
            min_factor: 1.0,
            max_factor: 3.0,
        }
    }
}

fn is_descendant(p: &Project, mut i: usize, ancestor: usize) -> bool {
    while let Some(par) = p.bones[i].parent.clone() {
        let Some(pi) = p.bones.iter().position(|b| b.name == par) else {
            return false;
        };
        if pi == ancestor {
            return true;
        }
        i = pi;
    }
    false
}

/// Returns how many frames needed a correction (angle or scale differing from the original).
pub fn bake_stretch(
    p: &mut Project,
    follower: &str,
    leader: &str,
    o: &StretchOptions,
) -> Result<usize, String> {
    if (o.min_factor.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater))
        || !matches!(
            o.max_factor.partial_cmp(&o.min_factor),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        )
        || !o.max_factor.is_finite()
    {
        return Err("factors must satisfy 0 < min <= max".into());
    }
    crate::chanmeta::assert_unlocked(p, follower, Prop::BoneAngle)?;
    crate::chanmeta::assert_unlocked(p, follower, Prop::BoneScale)?;
    let fi = p
        .bones
        .iter()
        .position(|b| b.name == follower)
        .ok_or_else(|| format!("no bone '{follower}'"))?;
    let li = p
        .bones
        .iter()
        .position(|b| b.name == leader)
        .ok_or_else(|| format!("no bone '{leader}'"))?;
    if fi == li {
        return Err("follower and leader must differ".into());
    }
    if is_descendant(p, li, fi) {
        return Err("the leader depends on the follower (it is one of its descendants)".into());
    }
    let len = p.bones[fi].length;
    if len.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return Err("the follower bone has no length".into());
    }
    let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
    let (mut ka, mut ks) = (Vec::new(), Vec::new());
    let mut changed = 0;
    let mut prev = 0.0f32;
    for f in 0..p.frames {
        let fr = e.evaluate(f as f32);
        let w = fr.world[fi];
        let head = crate::math::Vec2::new(w.tx, w.ty);
        let tip = w.apply(crate::math::Vec2::new(len, 0.0));
        let goal = crate::math::Vec2::new(fr.world[li].tx, fr.world[li].ty);
        let cur = tip - head;
        let want = goal - head;
        let (cur_len, want_len) = (cur.length(), want.length());
        let (mut delta, mut factor) = (0.0f32, 1.0f32);
        if cur_len > 1e-4 && want_len > 1e-4 {
            let mut d = (want.y.atan2(want.x) - cur.y.atan2(cur.x)).to_degrees();
            // keep the baked curve continuous across the +-180 degree seam
            while d - prev > 180.0 {
                d -= 360.0;
            }
            while d - prev < -180.0 {
                d += 360.0;
            }
            delta = d;
            factor = (want_len / cur_len).clamp(o.min_factor, o.max_factor);
        }
        prev = delta;
        if delta.abs() > 1e-3 || (factor - 1.0).abs() > 1e-3 {
            changed += 1;
        }
        ka.push((f as f32, delta, Ease::Linear));
        ks.push((f as f32, factor, Ease::Linear));
    }
    if changed == 0 {
        return Ok(0);
    }
    p.channels
        .push(Channel::keys(follower, Prop::BoneAngle, Track::with_keys(&ka)).with_mode(Mode::Add));
    p.channels.push(
        Channel::keys(follower, Prop::BoneScale, Track::with_keys(&ks)).with_mode(Mode::Multiply),
    );
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;
    use crate::project::Bone;

    fn rig() -> Project {
        let mut p = Project::new("s", 200.0, 200.0, 24.0, 10);
        p.bones = vec![
            Bone::new("arm", None, 10.0, Vec2::new(0.0, 0.0), 0.0),
            Bone::new("hand", None, 5.0, Vec2::new(30.0, 40.0), 0.0),
        ];
        // the leader slides along x
        let xs = [(0.0, 30.0, Ease::Linear), (9.0, 60.0, Ease::Linear)];
        p.channels = vec![Channel::keys(
            "hand",
            Prop::BoneOffsetX,
            Track::with_keys(
                &xs.iter()
                    .map(|k| (k.0, k.1 - 30.0, k.2))
                    .collect::<Vec<_>>(),
            ),
        )];
        p
    }

    fn tip(p: &Project, f: f32) -> (Vec2, Vec2) {
        let e = Engine::new(p.clone()).unwrap();
        let fr = e.evaluate(f);
        let w = fr.world[0];
        (
            w.apply(Vec2::new(10.0, 0.0)),
            Vec2::new(fr.world[1].tx, fr.world[1].ty),
        )
    }

    #[test]
    fn tip_reaches_the_leader_every_frame() {
        let mut p = rig();
        let n = bake_stretch(
            &mut p,
            "arm",
            "hand",
            &StretchOptions {
                max_factor: 10.0,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(n, 10);
        for f in [0.0, 4.0, 9.0] {
            let (t, g) = tip(&p, f);
            assert!(t.distance(g) < 0.2, "frame {f}: tip {t:?} goal {g:?}");
        }
    }

    #[test]
    fn limits_and_errors() {
        let mut p = rig();
        bake_stretch(
            &mut p,
            "arm",
            "hand",
            &StretchOptions {
                max_factor: 2.0,
                ..Default::default()
            },
        )
        .unwrap();
        let (t, g) = tip(&p, 0.0);
        assert!((t.distance(Vec2::ZERO) - 20.0).abs() < 0.1, "capped at 2x");
        assert!(t.distance(g) > 20.0);
        let mut q = rig();
        assert!(bake_stretch(&mut q, "arm", "arm", &StretchOptions::default()).is_err());
        assert!(bake_stretch(&mut q, "arm", "nope", &StretchOptions::default()).is_err());
        assert!(bake_stretch(
            &mut q,
            "arm",
            "hand",
            &StretchOptions {
                min_factor: 2.0,
                max_factor: 1.0
            }
        )
        .is_err());
        q.bones[1].parent = Some("arm".into());
        assert!(
            bake_stretch(&mut q, "arm", "hand", &StretchOptions::default())
                .unwrap_err()
                .contains("descendants")
        );
        let before = q.channels.len();
        assert_eq!(q.channels.len(), before, "failed bake adds nothing");
    }
}
