// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Non-linear animation helpers: clip retiming, mirroring, retargeting, crossfade baking
//! and a procedural walk-cycle generator. All operate on plain [`Channel`] data.

use crate::anim::{Ease, Extrapolate, Track};
use crate::project::{Channel, Prop, Source};
use std::collections::HashMap;

/// Retime a track: `new_frame = offset + old_frame * scale` (scale must be > 0).
pub fn retime_track(t: &Track, offset: f32, scale: f32) -> Track {
    let mut out = t.clone();
    for k in &mut out.keys {
        k.frame = offset + k.frame * scale;
    }
    out
}

/// A clip is a set of channels placed on the timeline: shifted by `offset` frames and
/// stretched by `scale`. Non-keyed sources (expressions, noise) are copied unchanged.
pub fn clip(channels: &[Channel], offset: f32, scale: f32) -> Vec<Channel> {
    channels
        .iter()
        .map(|c| {
            let mut c = c.clone();
            if let Source::Keys { track } = &c.source {
                c.source = Source::Keys {
                    track: retime_track(track, offset, scale.max(1e-4)),
                };
            }
            c
        })
        .collect()
}

/// Rename channel targets (retargeting a clip onto another rig with different bone names).
/// Unmapped names are kept.
pub fn retarget(channels: &[Channel], map: &HashMap<String, String>) -> Vec<Channel> {
    channels
        .iter()
        .map(|c| {
            let mut c = c.clone();
            if let Some(n) = map.get(&c.target) {
                c.target = n.clone();
            }
            c
        })
        .collect()
}

/// Swap left/right in a name: `L_`/`R_` prefixes, `_L`/`_R`, `_l`/`_r` suffixes and
/// the words `left`/`right` (case preserved for the common forms).
pub fn mirror_name(name: &str) -> String {
    const PAIRS: [(&str, &str); 8] = [
        ("L_", "R_"),
        ("l_", "r_"),
        ("_L", "_R"),
        ("_l", "_r"),
        ("Left", "Right"),
        ("left", "right"),
        ("LEFT", "RIGHT"),
        ("L.", "R."),
    ];
    for (a, b) in PAIRS {
        for (x, y) in [(a, b), (b, a)] {
            if x.starts_with('_') || x.ends_with('.') || x.ends_with('_') {
                if x.ends_with('_') || x.ends_with('.') {
                    if let Some(rest) = name.strip_prefix(x) {
                        return format!("{y}{rest}");
                    }
                } else if let Some(rest) = name.strip_suffix(x) {
                    return format!("{rest}{y}");
                }
            } else if name.contains(x) {
                return name.replacen(x, y, 1);
            }
        }
    }
    name.to_string()
}

/// Mirror channels across the vertical axis: targets swap sides, rotations and
/// horizontal translations change sign.
pub fn mirror(channels: &[Channel]) -> Vec<Channel> {
    channels
        .iter()
        .map(|c| {
            let mut c = c.clone();
            c.target = mirror_name(&c.target);
            let flip = matches!(
                c.prop,
                Prop::BoneAngle
                    | Prop::BoneOffsetX
                    | Prop::LayerRotation
                    | Prop::LayerX
                    | Prop::LayerSkew
            );
            if flip {
                match &mut c.source {
                    Source::Keys { track } => {
                        for k in &mut track.keys {
                            k.value = -k.value;
                        }
                    }
                    Source::Noise { amp, .. } => *amp = -*amp,
                    Source::Expr { expr } => *expr = format!("-({expr})"),
                    Source::Driven { curve, .. } => {
                        for k in &mut curve.keys {
                            k.value = -k.value;
                        }
                    }
                }
            }
            c
        })
        .collect()
}

/// Crossfade two tracks over `[start, start+len]`: `a` before, `b` after, a baked
/// per-frame blend (eased) in between.
pub fn crossfade(a: &Track, b: &Track, start: f32, len: f32, ease: Ease) -> Track {
    let end = start + len.max(1.0);
    let mut out = Track::new();
    for k in a.keys.iter().filter(|k| k.frame < start) {
        out.keys.push(*k);
    }
    let mut f = start.floor();
    while f <= end.ceil() {
        let t = ease.apply(((f - start) / (end - start)).clamp(0.0, 1.0));
        let (va, vb) = (a.eval(f).unwrap_or(0.0), b.eval(f).unwrap_or(0.0));
        out.set_key(f, va + (vb - va) * t, Ease::Linear);
        f += 1.0;
    }
    for k in b.keys.iter().filter(|k| k.frame > end.ceil()) {
        out.keys.push(*k);
    }
    out.extrapolate = b.extrapolate;
    out
}

/// Names of the bones a walk cycle drives. Missing (None) parts are skipped.
#[derive(Clone, Debug, Default)]
pub struct WalkRig {
    pub hips: Option<String>,
    pub thigh_l: Option<String>,
    pub thigh_r: Option<String>,
    pub shin_l: Option<String>,
    pub shin_r: Option<String>,
    pub arm_l: Option<String>,
    pub arm_r: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct WalkStyle {
    /// Frames for one full cycle (two steps).
    pub period: f32,
    /// Thigh swing amplitude (degrees).
    pub stride: f32,
    /// Peak knee bend (degrees).
    pub knee: f32,
    /// Vertical hip bob (pixels, positive = down on screen).
    pub bob: f32,
    /// Arm swing amplitude (degrees).
    pub arm_swing: f32,
}

impl Default for WalkStyle {
    fn default() -> Self {
        WalkStyle {
            period: 24.0,
            stride: 28.0,
            knee: 40.0,
            bob: 4.0,
            arm_swing: 22.0,
        }
    }
}

/// Generate a looping walk cycle as keyed channels (Loop extrapolation, 8 keys per step).
pub fn walk_cycle(rig: &WalkRig, style: &WalkStyle) -> Vec<Channel> {
    let mut out = Vec::new();
    let steps = 16; // keys per cycle (linear segments of a sampled curve: no easing overshoot)
    let make = |f: &dyn Fn(f32) -> f32| -> Track {
        let mut t = Track::new();
        for i in 0..=steps {
            let u = i as f32 / steps as f32;
            t.set_key(u * style.period, f(u * std::f32::consts::TAU), Ease::Linear);
        }
        t.extrapolate = Extrapolate::Loop;
        t
    };
    let mut add = |name: &Option<String>, prop: Prop, track: Track| {
        if let Some(n) = name {
            out.push(Channel::keys(n, prop, track));
        }
    };
    let thigh = |phase: f32| move |th: f32| style.stride * (th + phase).sin();
    let knee = |phase: f32| move |th: f32| style.knee * ((th + phase - 1.2).sin().max(0.0));
    let arm = |phase: f32| move |th: f32| style.arm_swing * (th + phase).sin();
    use std::f32::consts::PI;
    add(&rig.thigh_l, Prop::BoneAngle, make(&thigh(0.0)));
    add(&rig.thigh_r, Prop::BoneAngle, make(&thigh(PI)));
    add(&rig.shin_l, Prop::BoneAngle, make(&knee(0.0)));
    add(&rig.shin_r, Prop::BoneAngle, make(&knee(PI)));
    // arms swing opposite to the same-side leg
    add(&rig.arm_l, Prop::BoneAngle, make(&arm(PI)));
    add(&rig.arm_r, Prop::BoneAngle, make(&arm(0.0)));
    add(
        &rig.hips,
        Prop::BoneOffsetY,
        make(&|th: f32| style.bob * (2.0 * th).cos()),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kc(target: &str, prop: Prop, keys: &[(f32, f32)]) -> Channel {
        Channel::keys(
            target,
            prop,
            Track::with_keys(
                &keys
                    .iter()
                    .map(|&(f, v)| (f, v, Ease::Linear))
                    .collect::<Vec<_>>(),
            ),
        )
    }
    fn track(c: &Channel) -> &Track {
        match &c.source {
            Source::Keys { track } => track,
            _ => panic!(),
        }
    }

    #[test]
    fn clip_shifts_and_scales() {
        let c = clip(
            &[kc("a", Prop::BoneAngle, &[(0.0, 0.0), (10.0, 90.0)])],
            5.0,
            2.0,
        );
        let t = track(&c[0]);
        assert_eq!((t.keys[0].frame, t.keys[1].frame), (5.0, 25.0));
        assert!((t.eval(15.0).unwrap() - 45.0).abs() < 1e-3);
    }

    #[test]
    fn mirror_names() {
        assert_eq!(mirror_name("L_arm"), "R_arm");
        assert_eq!(mirror_name("R_arm"), "L_arm");
        assert_eq!(mirror_name("arm_l"), "arm_r");
        assert_eq!(mirror_name("leg_R"), "leg_L");
        assert_eq!(mirror_name("leftEye"), "rightEye");
        assert_eq!(mirror_name("head"), "head");
    }

    #[test]
    fn mirror_flips_rotation_and_x_only() {
        let m = mirror(&[
            kc("arm_l", Prop::BoneAngle, &[(0.0, 30.0)]),
            kc("arm_l", Prop::BoneScale, &[(0.0, 1.5)]),
            kc("hand_l", Prop::BoneOffsetX, &[(0.0, 4.0)]),
        ]);
        assert_eq!(m[0].target, "arm_r");
        assert_eq!(track(&m[0]).keys[0].value, -30.0);
        assert_eq!(track(&m[1]).keys[0].value, 1.5);
        assert_eq!(track(&m[2]).keys[0].value, -4.0);
        // mirroring twice is the identity
        let mm = mirror(&m);
        assert_eq!(mm[0].target, "arm_l");
        assert_eq!(track(&mm[0]).keys[0].value, 30.0);
    }

    #[test]
    fn retarget_renames() {
        let map: HashMap<String, String> = [("a".to_string(), "spine".to_string())].into();
        let c = retarget(
            &[
                kc("a", Prop::BoneAngle, &[(0.0, 1.0)]),
                kc("z", Prop::BoneAngle, &[(0.0, 1.0)]),
            ],
            &map,
        );
        assert_eq!((c[0].target.as_str(), c[1].target.as_str()), ("spine", "z"));
    }

    #[test]
    fn crossfade_blends_between_clips() {
        let a = Track::with_keys(&[(0.0, 0.0, Ease::Linear), (30.0, 0.0, Ease::Linear)]);
        let b = Track::with_keys(&[(0.0, 100.0, Ease::Linear), (30.0, 100.0, Ease::Linear)]);
        let x = crossfade(&a, &b, 10.0, 10.0, Ease::Linear);
        assert!((x.eval(5.0).unwrap() - 0.0).abs() < 1e-3);
        assert!((x.eval(15.0).unwrap() - 50.0).abs() < 1e-3);
        assert!((x.eval(25.0).unwrap() - 100.0).abs() < 1e-3);
    }

    #[test]
    fn walk_cycle_loops_and_alternates_legs() {
        let rig = WalkRig {
            hips: Some("hips".into()),
            thigh_l: Some("thigh_l".into()),
            thigh_r: Some("thigh_r".into()),
            shin_l: Some("shin_l".into()),
            shin_r: Some("shin_r".into()),
            arm_l: Some("arm_l".into()),
            arm_r: Some("arm_r".into()),
        };
        let ch = walk_cycle(&rig, &WalkStyle::default());
        assert_eq!(ch.len(), 7);
        let (l, r) = (track(&ch[0]), track(&ch[1]));
        for f in [0.0, 3.0, 7.5, 12.0] {
            assert!(
                (l.eval(f).unwrap() + r.eval(f).unwrap()).abs() < 1.0,
                "legs are in opposition at {f}"
            );
            assert!(
                (l.eval(f).unwrap() - l.eval(f + 24.0).unwrap()).abs() < 1e-3,
                "periodic"
            );
        }
        let peak = (0..24)
            .map(|f| l.eval(f as f32).unwrap())
            .fold(0.0f32, f32::max);
        assert!(peak > 25.0 && peak < 30.0, "{peak}");
        let knee = track(&ch[2]);
        let kmin = (0..24)
            .map(|f| knee.eval(f as f32).unwrap())
            .fold(f32::MAX, f32::min);
        assert!(kmin >= -1e-3, "knees only bend one way: {kmin}");
    }

    #[test]
    fn walk_cycle_skips_missing_bones() {
        let ch = walk_cycle(
            &WalkRig {
                thigh_l: Some("t".into()),
                ..Default::default()
            },
            &WalkStyle::default(),
        );
        assert_eq!(ch.len(), 1);
    }
}
