// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Foot lock: stop a foot-control bone from sliding while it is on the ground.
//!
//! The controller's world position is sampled every frame. Runs of frames where it touches the
//! ground line (within a tolerance) are "contacts"; for each, an additive correction channel
//! pins the controller to where the contact began. Original animation stays untouched.

use crate::anim::{Ease, Track};
use crate::project::{Channel, Mode, Project, Prop};
use crate::Engine;

pub struct LockOptions {
    /// World y of the ground (stage y points down; a foot is down when y >= ground - tolerance).
    pub ground_y: f32,
    pub tolerance: f32,
    /// Shortest contact worth locking, frames.
    pub min_frames: usize,
}

impl Default for LockOptions {
    fn default() -> Self {
        LockOptions {
            ground_y: 0.0,
            tolerance: 2.0,
            min_frames: 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Contact {
    pub start: usize,
    pub end: usize,
    pub x: f32,
    pub y: f32,
}

/// Contacts found for a root controller bone.
/// Contacts found plus the sampled `(frame, height)` curve.
pub type ContactScan = (Vec<Contact>, Vec<(f32, f32)>);

pub fn find_contacts(p: &Project, bone: &str, o: &LockOptions) -> Result<ContactScan, String> {
    let bi = p
        .bones
        .iter()
        .position(|b| b.name == bone)
        .ok_or_else(|| format!("no bone '{bone}'"))?;
    if p.bones[bi].parent.is_some() {
        return Err("foot lock needs a root controller bone (no parent)".into());
    }
    if !o.ground_y.is_finite()
        || !matches!(
            o.tolerance.partial_cmp(&0.0),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        )
    {
        return Err("invalid ground or tolerance".into());
    }
    let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
    let pos: Vec<(f32, f32)> = (0..p.frames)
        .map(|f| {
            let w = e.evaluate(f as f32).world[bi];
            (w.tx, w.ty)
        })
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < pos.len() {
        if pos[i].1 >= o.ground_y - o.tolerance {
            let s = i;
            while i < pos.len() && pos[i].1 >= o.ground_y - o.tolerance {
                i += 1;
            }
            if i - s >= o.min_frames {
                out.push(Contact {
                    start: s,
                    end: i - 1,
                    x: pos[s].0,
                    y: pos[s].1,
                });
            }
        } else {
            i += 1;
        }
    }
    Ok((out, pos))
}

/// Add two additive channels (`BoneOffsetX`/`BoneOffsetY`) that pin the bone during contacts.
/// Returns the contacts locked.
pub fn lock_feet(p: &mut Project, bone: &str, o: &LockOptions) -> Result<Vec<Contact>, String> {
    crate::chanmeta::assert_unlocked(p, bone, Prop::BoneOffsetX)?;
    crate::chanmeta::assert_unlocked(p, bone, Prop::BoneOffsetY)?;
    let (contacts, pos) = find_contacts(p, bone, o)?;
    if contacts.is_empty() {
        return Ok(contacts);
    }
    let mut kx: Vec<(f32, f32, Ease)> = Vec::new();
    let mut ky: Vec<(f32, f32, Ease)> = Vec::new();
    let mut push = |f: usize, dx: f32, dy: f32| {
        kx.push((f as f32, dx, Ease::Linear));
        ky.push((f as f32, dy, Ease::Linear));
    };
    for c in &contacts {
        if c.start > 0 {
            push(c.start - 1, 0.0, 0.0);
        }
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for f in c.start..=c.end {
            push(f, c.x - pos[f].0, c.y - pos[f].1);
        }
        if c.end + 1 < pos.len() {
            push(c.end + 1, 0.0, 0.0);
        }
    }
    // consecutive contacts can share a boundary frame: keep the later value
    for k in [&mut kx, &mut ky] {
        k.dedup_by(|b, a| {
            if a.0 == b.0 {
                a.1 = b.1;
                true
            } else {
                false
            }
        });
    }
    p.channels
        .push(Channel::keys(bone, Prop::BoneOffsetX, Track::with_keys(&kx)).with_mode(Mode::Add));
    p.channels
        .push(Channel::keys(bone, Prop::BoneOffsetY, Track::with_keys(&ky)).with_mode(Mode::Add));
    Ok(contacts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;
    use crate::project::Bone;

    /// Foot slides right at 2 px/frame the whole time but dips to the ground in frames 10..=19.
    fn sliding() -> Project {
        let mut p = Project::new("f", 400.0, 200.0, 24.0, 40);
        p.bones = vec![Bone::new("foot", None, 10.0, Vec2::new(0.0, 100.0), 0.0)];
        let xs: Vec<(f32, f32, Ease)> = vec![(0.0, 0.0, Ease::Linear), (39.0, 78.0, Ease::Linear)];
        let ys: Vec<(f32, f32, Ease)> = vec![
            (0.0, -30.0, Ease::Linear),
            (9.0, -30.0, Ease::Linear),
            (10.0, 0.0, Ease::Linear),
            (19.0, 0.0, Ease::Linear),
            (20.0, -30.0, Ease::Linear),
            (39.0, -30.0, Ease::Linear),
        ];
        p.channels = vec![
            Channel::keys("foot", Prop::BoneOffsetX, Track::with_keys(&xs)),
            Channel::keys("foot", Prop::BoneOffsetY, Track::with_keys(&ys)),
        ];
        p
    }

    fn x_at(p: &Project, f: f32) -> f32 {
        Engine::new(p.clone()).unwrap().evaluate(f).world[0].tx
    }

    #[test]
    fn detects_one_contact() {
        let p = sliding();
        let o = LockOptions {
            ground_y: 100.0,
            ..Default::default()
        };
        let (c, _) = find_contacts(&p, "foot", &o).unwrap();
        assert_eq!(c.len(), 1);
        assert_eq!((c[0].start, c[0].end), (10, 19));
    }

    #[test]
    fn locking_removes_sliding_only_during_contact() {
        let mut p = sliding();
        let o = LockOptions {
            ground_y: 100.0,
            ..Default::default()
        };
        assert!(x_at(&p, 19.0) - x_at(&p, 10.0) > 10.0, "slides before");
        lock_feet(&mut p, "foot", &o).unwrap();
        for f in 10..=19 {
            assert!(
                (x_at(&p, f as f32) - x_at(&p, 10.0)).abs() < 0.01,
                "frame {f}"
            );
        }
        assert!((x_at(&p, 5.0) - 10.0).abs() < 0.01, "free frames unchanged");
        assert!((x_at(&p, 30.0) - 60.0).abs() < 0.01);
    }

    #[test]
    fn no_contact_and_bad_input() {
        let mut p = sliding();
        let o = LockOptions {
            ground_y: 500.0,
            ..Default::default()
        };
        assert!(lock_feet(&mut p, "foot", &o).unwrap().is_empty());
        assert_eq!(p.channels.len(), 2);
        assert!(lock_feet(&mut p, "ghost", &o).is_err());
        let mut q = sliding();
        q.bones.push(Bone::new(
            "kid",
            Some("foot"),
            5.0,
            Vec2::new(5.0, 0.0),
            0.0,
        ));
        assert!(lock_feet(&mut q, "kid", &o).is_err());
    }
}
