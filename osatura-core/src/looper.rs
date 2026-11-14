// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Loop conversion: make keyed channels end where they began.
//!
//! For each hand-keyed `Replace` channel the curve is sampled on every frame `0..=last`, the straight
//! line joining its first and last value is subtracted (drift removal) and the first value is added
//! back, so frame `last` equals frame 0 and the track can cycle. Applied to a travelling bone's X
//! offset this also turns a walk into an in-place walk. Own design, textbook linear detrending.
//!
//! # Seam semantics ([`LoopMode`])
//!
//! `Exclusive` (default, correct for a player that repeats frames `0..N-1`): the loop period is
//! `N = project.frames`. The original span `0..=N-1` is resampled onto `0..=N`, so frame `N` (never
//! played, the start of the next cycle) equals frame 0 and frame `N-1` is one normal step before
//! it: no frame is shown twice at the seam. `Inclusive` keeps the earlier behaviour (frame `N-1`
//! equals frame 0, period `N-1`); use it only when the player or exporter drops the last frame.

use crate::anim::{Ease, Extrapolate, Track};
use crate::project::{Mode, Project, Prop, Source};

/// Which frame closes the loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LoopMode {
    /// Period `N`: frame `N` equals frame 0, frame `N-1` does not (no duplicated seam frame).
    #[default]
    Exclusive,
    /// Period `N-1`: frame `N-1` equals frame 0 (the seam frame is shown twice when frames `0..N-1` repeat).
    Inclusive,
}

pub struct LoopReport {
    pub converted: Vec<(String, Prop, f32)>,
    pub skipped_locked: Vec<String>,
    /// Converted channels whose result is a constant (their whole change was travel, e.g. the x offset of a walk):
    /// the in-place conversion is working as designed, and `lint` reports such a channel as L005 afterwards.
    pub flattened: Vec<(String, Prop)>,
}

/// `only`: restrict to channels whose target is in the list (empty = all). `props`: restrict to
/// properties (empty = all). Drift removed is reported per channel.
pub fn make_loop(p: &mut Project, only: &[String], props: &[Prop]) -> Result<LoopReport, String> {
    make_loop_mode(p, only, props, LoopMode::Exclusive)
}

pub fn make_loop_mode(
    p: &mut Project,
    only: &[String],
    props: &[Prop],
    mode: LoopMode,
) -> Result<LoopReport, String> {
    if p.frames < 2 {
        return Err("need at least 2 frames".into());
    }
    let last = (p.frames - 1) as f32;
    let mut rep = LoopReport {
        converted: vec![],
        skipped_locked: vec![],
        flattened: vec![],
    };
    for i in 0..p.channels.len() {
        let (target, prop) = (p.channels[i].target.clone(), p.channels[i].prop);
        if p.channels[i].mode != Mode::Replace
            || prop == Prop::LayerSwitch
            || (!only.is_empty() && !only.contains(&target))
            || (!props.is_empty() && !props.contains(&prop))
        {
            continue;
        }
        let track = match &p.channels[i].source {
            Source::Keys { track } if track.keys.len() >= 2 => track.clone(),
            _ => continue,
        };
        if crate::chanmeta::assert_unlocked(p, &target, prop).is_err() {
            rep.skipped_locked.push(target);
            continue;
        }
        let v0 = track.eval(0.0).unwrap_or(0.0);
        let v1 = track.eval(last).unwrap_or(0.0);
        let drift = v1 - v0;
        let mut keys: Vec<(f32, f32, Ease)> = Vec::new();
        match mode {
            LoopMode::Inclusive => {
                for f in 0..p.frames {
                    let v = track.eval(f as f32).unwrap_or(0.0) - drift * (f as f32 / last);
                    keys.push((f as f32, v, Ease::Linear));
                }
            }
            LoopMode::Exclusive => {
                // period N: output frame f (0..=N) samples the source at f * (N-1)/N
                let n = p.frames as f32;
                for f in 0..=p.frames {
                    let t = if f == p.frames {
                        last
                    } else {
                        f as f32 * last / n
                    };
                    let v = track.eval(t).unwrap_or(0.0) - drift * (t / last);
                    keys.push((f as f32, v, Ease::Linear));
                }
            }
        }
        let (lo, hi) = keys.iter().fold((f32::MAX, f32::MIN), |(lo, hi), k| {
            (lo.min(k.1), hi.max(k.1))
        });
        if hi - lo <= 1e-4 {
            rep.flattened.push((target.clone(), prop));
        }
        let mut t = Track::with_keys(&keys);
        t.extrapolate = Extrapolate::Loop;
        p.channels[i].source = Source::Keys { track: t };
        rep.converted.push((target, prop, drift));
    }
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;
    use crate::project::{Bone, Channel};
    use crate::Engine;

    fn walk() -> Project {
        let mut p = Project::new("l", 200.0, 200.0, 24.0, 25);
        p.bones = vec![Bone::new("b", None, 10.0, Vec2::new(0.0, 100.0), 0.0)];
        let x: Vec<(f32, f32, Ease)> = vec![(0.0, 0.0, Ease::Linear), (24.0, 96.0, Ease::Linear)];
        let r: Vec<(f32, f32, Ease)> = vec![
            (0.0, 0.0, Ease::Linear),
            (12.0, 30.0, Ease::Linear),
            (24.0, 10.0, Ease::Linear),
        ];
        p.channels = vec![
            Channel::keys("b", Prop::BoneOffsetX, Track::with_keys(&x)),
            Channel::keys("b", Prop::BoneAngle, Track::with_keys(&r)),
        ];
        p
    }

    /// Values of a bone's angle and x over the frames 0..N-1 as a player would show them.
    fn played(p: &Project) -> Vec<(f32, f32)> {
        let e = Engine::new(p.clone()).unwrap();
        (0..p.frames)
            .map(|f| {
                let w = e.evaluate(f as f32).world[0];
                (w.tx, w.rotation())
            })
            .collect()
    }

    /// Wave 7 (was L-70): the travel of a walk is removed on purpose (a project that plays 0..N-1 repeatedly cannot
    /// travel for ever); the report now names the channels that became constant so the L005 finding is expected.
    #[test]
    fn flattened_channels_are_reported() {
        let mut p = walk();
        let r = make_loop(&mut p, &[], &[]).unwrap();
        assert_eq!(
            r.flattened,
            vec![("b".to_string(), Prop::BoneOffsetX)],
            "only the pure travel channel became constant"
        );
        let mut q = walk();
        let r = make_loop(&mut q, &[], &[Prop::BoneAngle]).unwrap();
        assert!(
            r.flattened.is_empty() && r.converted.len() == 1,
            "restricting the props keeps the travel"
        );
    }

    /// C5: by default the loop has period N: frames 0..N-1 repeated show no duplicated seam frame and
    /// the wrap step is no larger than a normal step.
    #[test]
    fn default_loop_is_seamless_without_a_duplicated_frame() {
        let mut p = walk();
        make_loop(&mut p, &[], &[]).unwrap();
        let seq = played(&p);
        let n = seq.len();
        for ch in 0..2 {
            let get = |i: usize| if ch == 0 { seq[i].0 } else { seq[i].1 };
            let steps: Vec<f32> = (1..n).map(|i| (get(i) - get(i - 1)).abs()).collect();
            let max_step = steps.iter().cloned().fold(0.0, f32::max);
            let wrap = (get(0) - get(n - 1)).abs();
            // (channel 0 is the travel, which make-loop turns into a constant: nothing to duplicate)
            assert!(
                ch == 0 || wrap > 1e-4,
                "channel {ch}: frame N-1 duplicates frame 0"
            );
            assert!(
                wrap <= max_step * 1.01 + 1e-4,
                "channel {ch}: wrap {wrap} vs max step {max_step}"
            );
        }
        // frame N (next cycle) equals frame 0, and the track loops with period N
        let e = Engine::new(p.clone()).unwrap();
        let (a, b) = (
            e.evaluate(0.0).world[0],
            e.evaluate(p.frames as f32).world[0],
        );
        assert!((a.tx - b.tx).abs() < 1e-3 && (a.rotation() - b.rotation()).abs() < 1e-3);
        let c = e.evaluate(p.frames as f32 + 3.0).world[0];
        let d = e.evaluate(3.0).world[0];
        assert!((c.rotation() - d.rotation()).abs() < 1e-3, "keeps cycling");
    }

    #[test]
    fn inclusive_mode_keeps_the_legacy_duplicate_seam_frame() {
        let mut p = walk();
        make_loop_mode(&mut p, &[], &[], LoopMode::Inclusive).unwrap();
        let seq = played(&p);
        assert!((seq[0].1 - seq[seq.len() - 1].1).abs() < 1e-3);
    }

    #[test]
    fn ends_match_and_travel_removed() {
        let mut p = walk();
        let r = make_loop_mode(&mut p, &[], &[], LoopMode::Inclusive).unwrap();
        assert_eq!(r.converted.len(), 2);
        let e = Engine::new(p).unwrap();
        let (a, b) = (e.evaluate(0.0).world[0], e.evaluate(24.0).world[0]);
        assert!((a.tx - b.tx).abs() < 1e-3 && (a.rotation() - b.rotation()).abs() < 1e-3);
        assert!(
            (e.evaluate(12.0).world[0].tx - a.tx).abs() < 1e-3,
            "in place"
        );
    }

    #[test]
    fn props_filter_and_lock() {
        let mut p = walk();
        make_loop(&mut p, &[], &[Prop::BoneAngle]).unwrap();
        let e = Engine::new(p.clone()).unwrap();
        assert!(
            e.evaluate(24.0).world[0].tx - e.evaluate(0.0).world[0].tx > 90.0,
            "x untouched"
        );
        let mut q = walk();
        crate::chanmeta::set_meta(
            &mut q,
            "b",
            Some(Prop::BoneOffsetX),
            &crate::chanmeta::Change {
                lock: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        let r = make_loop(&mut q, &[], &[Prop::BoneOffsetX]).unwrap();
        assert!(r.converted.is_empty() && r.skipped_locked.len() == 1);
    }
}
