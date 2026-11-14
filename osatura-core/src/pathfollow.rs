// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Path constraint, baked: sample a vector path per frame into plain keyed channels so a
//! layer (or a root bone) travels along it, optionally turning to face the direction of travel.

use crate::anim::{Ease, Track};
use crate::path::VPath;
use crate::project::{Channel, Project, Prop};

pub struct FollowOptions {
    pub start_frame: f32,
    pub end_frame: f32,
    /// Path fractions travelled between start_frame and end_frame (0..1; closed paths may loop).
    pub from: f32,
    pub to: f32,
    pub orient: bool,
    /// Added to the facing angle, degrees.
    pub angle_offset: f32,
}

impl Default for FollowOptions {
    fn default() -> Self {
        FollowOptions {
            start_frame: 0.0,
            end_frame: 0.0,
            from: 0.0,
            to: 1.0,
            orient: true,
            angle_offset: 0.0,
        }
    }
}

fn unwrap_angles(v: &mut [f32]) {
    for i in 1..v.len() {
        while v[i] - v[i - 1] > 180.0 {
            v[i] -= 360.0;
        }
        while v[i] - v[i - 1] < -180.0 {
            v[i] += 360.0;
        }
    }
}

/// Channels (x, y and optionally rotation) that move `target` along `path`. For a layer the
/// path points become its position; for a root bone they become offsets from its rest position.
/// One key per frame, so timing is exact and the data stays editable.
pub fn follow_path(
    p: &Project,
    target: &str,
    is_bone: bool,
    path: &VPath,
    o: &FollowOptions,
) -> Result<Vec<Channel>, String> {
    if (o.end_frame.partial_cmp(&o.start_frame) != Some(std::cmp::Ordering::Greater))
        || !o.start_frame.is_finite()
        || !o.end_frame.is_finite()
        || o.start_frame < 0.0
    {
        return Err("end_frame must be greater than start_frame".into());
    }
    if !o.from.is_finite() || !o.to.is_finite() {
        return Err("from/to must be numbers".into());
    }
    let (xp, yp) = if is_bone {
        (Prop::BoneOffsetX, Prop::BoneOffsetY)
    } else {
        (Prop::LayerX, Prop::LayerY)
    };
    crate::chanmeta::assert_unlocked(p, target, xp)?;
    crate::chanmeta::assert_unlocked(p, target, yp)?;
    let pl = path.polyline(0.25);
    if pl.length() <= 1e-6 {
        return Err("path has no length".into());
    }
    let (px, py, pr) = if is_bone {
        let b = p
            .bones
            .iter()
            .find(|b| b.name == target)
            .ok_or_else(|| format!("no bone '{target}'"))?;
        if b.parent.is_some() {
            return Err("path following needs a root bone (no parent)".into());
        }
        (Prop::BoneOffsetX, Prop::BoneOffsetY, Prop::BoneAngle)
    } else {
        p.find_layer(target)
            .ok_or_else(|| format!("no layer '{target}'"))?;
        (Prop::LayerX, Prop::LayerY, Prop::LayerRotation)
    };
    let rest = if is_bone {
        p.bones
            .iter()
            .find(|b| b.name == target)
            .map(|b| b.rest_offset)
    } else {
        None
    }
    .unwrap_or(crate::math::Vec2::ZERO);
    let n = (o.end_frame - o.start_frame).ceil() as usize;
    let (mut xs, mut ys, mut rs) = (vec![], vec![], vec![]);
    for i in 0..=n {
        let f = (o.start_frame + i as f32).min(o.end_frame);
        let t = (f - o.start_frame) / (o.end_frame - o.start_frame);
        let frac = o.from + (o.to - o.from) * t;
        let (pt, tan) = pl.sample(frac * pl.length());
        xs.push((f, pt.x - rest.x));
        ys.push((f, pt.y - rest.y));
        rs.push(tan.y.atan2(tan.x).to_degrees() + o.angle_offset);
    }
    let keys = |v: &[(f32, f32)]| {
        Track::with_keys(
            &v.iter()
                .map(|&(f, x)| (f, x, Ease::Linear))
                .collect::<Vec<_>>(),
        )
    };
    let mut out = vec![
        Channel::keys(target, px, keys(&xs)),
        Channel::keys(target, py, keys(&ys)),
    ];
    if o.orient {
        unwrap_angles(&mut rs);
        let rv: Vec<(f32, f32)> = xs.iter().map(|k| k.0).zip(rs).collect();
        out.push(Channel::keys(target, pr, keys(&rv)));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;
    use crate::path::Anchor;
    use crate::project::{Bone, Layer};
    use crate::Engine;

    fn line() -> VPath {
        VPath::new(
            "p",
            false,
            vec![
                Anchor::corner(Vec2::new(0.0, 0.0)),
                Anchor::corner(Vec2::new(100.0, 0.0)),
                Anchor::corner(Vec2::new(100.0, 100.0)),
            ],
        )
    }

    #[test]
    fn layer_travels_and_faces_direction() {
        let mut p = Project::new("f", 200.0, 200.0, 24.0, 30);
        p.layers = vec![Layer::vector(
            "ball",
            vec![VPath::rect("r", -5.0, -5.0, 10.0, 10.0).with_fill("#00ff00")],
        )];
        let o = FollowOptions {
            end_frame: 20.0,
            ..Default::default()
        };
        p.channels
            .extend(follow_path(&p, "ball", false, &line(), &o).unwrap());
        let e = Engine::new(p.clone()).unwrap();
        let _ = e.evaluate(0.0);
        let get = |f: f32, pr: Prop| {
            let c = p.channels.iter().find(|c| c.prop == pr).unwrap();
            let crate::project::Source::Keys { track } = &c.source else {
                panic!()
            };
            track.eval(f).unwrap()
        };
        assert!(
            (get(10.0, Prop::LayerX) - 100.0).abs() < 1.0,
            "halfway = first corner"
        );
        assert!(get(10.0, Prop::LayerY).abs() < 1.0);
        assert!((get(20.0, Prop::LayerY) - 100.0).abs() < 1.0);
        assert!(
            get(2.0, Prop::LayerRotation).abs() < 1.0
                && (get(18.0, Prop::LayerRotation) - 90.0).abs() < 1.0
        );
    }

    #[test]
    fn root_bone_offsets_from_rest_and_errors() {
        let mut p = Project::new("f", 200.0, 200.0, 24.0, 30);
        p.bones = vec![
            Bone::new("root", None, 10.0, Vec2::new(10.0, 10.0), 0.0),
            Bone::new("kid", Some("root"), 5.0, Vec2::new(10.0, 0.0), 0.0),
        ];
        let o = FollowOptions {
            end_frame: 10.0,
            orient: false,
            ..Default::default()
        };
        let ch = follow_path(&p, "root", true, &line(), &o).unwrap();
        assert_eq!(ch.len(), 2);
        let crate::project::Source::Keys { track } = &ch[0].source else {
            panic!()
        };
        assert!(
            (track.eval(0.0).unwrap() + 10.0).abs() < 0.01,
            "path start minus rest offset"
        );
        assert!(follow_path(&p, "kid", true, &line(), &o).is_err());
        assert!(follow_path(&p, "nope", false, &line(), &o).is_err());
        assert!(
            follow_path(&p, "root", true, &line(), &FollowOptions::default()).is_err(),
            "zero duration"
        );
    }

    #[test]
    fn angles_do_not_spin_across_the_wrap() {
        let mut v = vec![170.0, -170.0, -160.0];
        unwrap_angles(&mut v);
        assert!(v[1] > 170.0 && v[2] > v[1]);
    }
}
