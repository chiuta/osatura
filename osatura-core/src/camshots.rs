// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Saved camera shots and shot sequences (cuts and eased moves).

use crate::anim::{Ease, Track};
use crate::project::{Channel, Project, Prop, Source};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shot {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    #[serde(default)]
    pub rotation: f32,
}

/// Save (or overwrite) a named shot.
pub fn save_shot(
    p: &mut Project,
    name: &str,
    x: f32,
    y: f32,
    zoom: f32,
    rotation: f32,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("shot needs a name".into());
    }
    if ![x, y, zoom, rotation].iter().all(|v| v.is_finite()) || zoom <= 0.0 {
        return Err("shot values must be finite and zoom positive".into());
    }
    let s = Shot {
        name: name.into(),
        x,
        y,
        zoom,
        rotation,
    };
    match p.shots.iter_mut().find(|e| e.name == name) {
        Some(e) => *e = s,
        None => p.shots.push(s),
    }
    Ok(())
}

/// One step of a sequence: reach `shot` at `frame`; `cut` jumps there instead of moving.
#[derive(Clone, Debug)]
pub struct Step {
    pub frame: f32,
    pub shot: String,
    pub cut: bool,
}

/// Replace the project's camera channels with keys that visit the shots in order.
/// A move eases in/out from the previous step; a cut holds the previous shot until `frame`.
pub fn sequence(p: &mut Project, steps: &[Step]) -> Result<(), String> {
    if steps.is_empty() {
        return Err("need at least one step".into());
    }
    let mut st = steps.to_vec();
    st.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    if st.windows(2).any(|w| w[0].frame == w[1].frame)
        || st.iter().any(|s| !s.frame.is_finite() || s.frame < 0.0)
    {
        return Err("step frames must be distinct, finite and non-negative".into());
    }
    let shots: Vec<&Shot> = st
        .iter()
        .map(|s| {
            p.shots
                .iter()
                .find(|x| x.name == s.shot)
                .ok_or_else(|| format!("no shot '{}'", s.shot))
        })
        .collect::<Result<_, _>>()?;
    type PropKeys = (Prop, Vec<(f32, f32, Ease)>);
    let mut tracks: Vec<PropKeys> = [
        Prop::CameraX,
        Prop::CameraY,
        Prop::CameraZoom,
        Prop::CameraRotation,
    ]
    .iter()
    .map(|&pr| (pr, vec![]))
    .collect();
    for (i, (s, shot)) in st.iter().zip(&shots).enumerate() {
        let vals = [shot.x, shot.y, shot.zoom, shot.rotation];
        let next_cut = st.get(i + 1).is_some_and(|n| n.cut);
        for (t, v) in tracks.iter_mut().zip(vals) {
            // the ease of a key shapes the segment that follows it
            t.1.push((
                s.frame,
                v,
                if next_cut {
                    Ease::Step
                } else {
                    Ease::EaseInOut
                },
            ));
        }
    }
    p.channels.retain(|c| {
        !matches!(
            c.prop,
            Prop::CameraX | Prop::CameraY | Prop::CameraZoom | Prop::CameraRotation
        ) || !matches!(c.source, Source::Keys { .. })
    });
    for (prop, keys) in tracks {
        p.channels
            .push(Channel::keys("", prop, Track::with_keys(&keys)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Engine;

    fn cam(p: &Project, f: f32) -> (f32, f32) {
        let e = Engine::new(p.clone()).unwrap();
        let fr = e.evaluate(f);
        let c = fr.camera;
        let centre = c
            .inverse()
            .unwrap()
            .apply(crate::math::Vec2::new(p.width * 0.5, p.height * 0.5));
        (centre.x, c.a.hypot(c.b))
    }

    fn proj() -> Project {
        let mut p = Project::new("c", 400.0, 300.0, 24.0, 60);
        save_shot(&mut p, "wide", 200.0, 150.0, 1.0, 0.0).unwrap();
        save_shot(&mut p, "close", 300.0, 100.0, 2.0, 0.0).unwrap();
        p
    }

    #[test]
    fn move_eases_between_shots() {
        let mut p = proj();
        sequence(
            &mut p,
            &[
                Step {
                    frame: 0.0,
                    shot: "wide".into(),
                    cut: false,
                },
                Step {
                    frame: 30.0,
                    shot: "close".into(),
                    cut: false,
                },
            ],
        )
        .unwrap();
        let (x0, z0) = cam(&p, 0.0);
        let (xm, zm) = cam(&p, 15.0);
        let (x1, z1) = cam(&p, 30.0);
        assert!((x0 - 200.0).abs() < 0.01 && (z0 - 1.0).abs() < 0.01);
        assert!(
            (xm - 250.0).abs() < 0.5 && (zm - 1.5).abs() < 0.01,
            "ease in/out is symmetric: midpoint is halfway"
        );
        assert!((x1 - 300.0).abs() < 0.01 && (z1 - 2.0).abs() < 0.01);
    }

    #[test]
    fn cut_holds_then_jumps() {
        let mut p = proj();
        sequence(
            &mut p,
            &[
                Step {
                    frame: 0.0,
                    shot: "wide".into(),
                    cut: false,
                },
                Step {
                    frame: 30.0,
                    shot: "close".into(),
                    cut: true,
                },
            ],
        )
        .unwrap();
        assert!((cam(&p, 29.0).0 - 200.0).abs() < 0.01, "holds");
        assert!((cam(&p, 30.0).0 - 300.0).abs() < 0.01, "jumps");
    }

    #[test]
    fn rerun_replaces_and_errors_are_clean() {
        let mut p = proj();
        let steps = [Step {
            frame: 0.0,
            shot: "wide".into(),
            cut: false,
        }];
        sequence(&mut p, &steps).unwrap();
        let n = p.channels.len();
        sequence(&mut p, &steps).unwrap();
        assert_eq!(p.channels.len(), n);
        assert!(sequence(
            &mut p,
            &[Step {
                frame: 0.0,
                shot: "nope".into(),
                cut: false
            }]
        )
        .is_err());
        assert!(save_shot(&mut p, "bad", 0.0, 0.0, 0.0, 0.0).is_err());
        save_shot(&mut p, "wide", 1.0, 1.0, 1.0, 0.0).unwrap();
        assert_eq!(p.shots.len(), 2, "same name overwrites");
        let j = p.to_json_pretty();
        assert!(Project::from_json(&j).unwrap().shots.len() == 2);
    }
}
