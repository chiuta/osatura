// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Seeded, deterministic procedural animation generators: blinks, breathing, sway and
//! follow-through (overlap) chains. Everything returns ordinary channels, so results can be
//! edited, baked or exported like hand-keyed animation.
//!
//! Provenance note: generic animation-principles maths (periodic eases, delayed copies with
//! decay). No product behaviour is imitated.

use crate::anim::{Ease, Extrapolate, Track};
use crate::project::{Channel, Mode, Prop, Source};

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0x1234567))
    }
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        ((self.0 >> 40) as f32) / (1u64 << 24) as f32
    }
}

#[derive(Clone, Debug)]
pub struct BlinkOptions {
    /// Average gap between blinks, seconds.
    pub mean_interval_s: f32,
    /// Random spread as a fraction of the mean (0 = metronome, 0.5 = ±50 %).
    pub jitter: f32,
    /// Frames the eye stays shut.
    pub closed_frames: f32,
    pub open: f32,
    pub closed: f32,
    pub seed: u64,
}

impl Default for BlinkOptions {
    fn default() -> Self {
        BlinkOptions { mean_interval_s: 3.5, jitter: 0.5, closed_frames: 2.0, open: 1.0, closed: 0.0, seed: 1 }
    }
}

/// Keys for a blink pattern over `frames` frames. Blinks never overlap and never start
/// closer than 1 s to the end of the track, so a looping clip stays clean.
pub fn blink_track(frames: u32, fps: f32, o: &BlinkOptions) -> Result<Track, String> {
    if (o.mean_interval_s.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || (fps.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !matches!(o.closed_frames.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) || !(0.0..=0.95).contains(&o.jitter) {
        return Err("invalid blink options".into());
    }
    let mut rng = Rng::new(o.seed);
    let mut t = Track::new();
    t.set_key(0.0, o.open, Ease::Linear);
    let mean = o.mean_interval_s * fps;
    let min_gap = o.closed_frames + 4.0;
    let mut at = mean * (0.5 + 0.5 * rng.next());
    let end = frames as f32 - fps;
    while at < end {
        let f = at.round();
        let c = o.closed_frames.round();
        t.set_key(f, o.open, Ease::Linear);
        t.set_key(f + 1.0, o.closed, Ease::Linear);
        t.set_key(f + 1.0 + c, o.closed, Ease::Linear);
        t.set_key(f + 2.0 + c, o.open, Ease::Linear);
        let gap = mean * (1.0 + o.jitter * (2.0 * rng.next() - 1.0));
        at = f + gap.max(min_gap);
    }
    Ok(t)
}

/// Ready-made blink channel (e.g. `LayerScaleY` of an eyelid, or `LayerOpacity`).
pub fn blink_channel(target: &str, prop: Prop, frames: u32, fps: f32, o: &BlinkOptions) -> Result<Channel, String> {
    Ok(Channel::keys(target, prop, blink_track(frames, fps, o)?))
}

/// Additive breathing: value rises to `amp` and returns once per `period` frames, forever.
pub fn breath_channel(target: &str, prop: Prop, amp: f32, period: f32) -> Result<Channel, String> {
    if !matches!(period.partial_cmp(&2.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) || !amp.is_finite() {
        return Err("period must be at least 2 frames".into());
    }
    let track = Track::with_keys(&[(0.0, 0.0, Ease::EaseInOut), (period * 0.5, amp, Ease::EaseInOut), (period, 0.0, Ease::EaseInOut)]).looped(Extrapolate::Loop);
    Ok(Channel::keys(target, prop, track).with_mode(Mode::Add))
}

/// Additive slow sway from seeded fractal noise (`freq` cycles per second).
pub fn sway_channel(target: &str, prop: Prop, amp: f32, freq: f32, seed: u32) -> Channel {
    Channel::noise(target, prop, amp, freq, seed).with_mode(Mode::Add)
}

/// Follow-through chain: copy a keyed channel to each target with an increasing delay and
/// a decaying amplitude (`value × decayⁱ`, `frame + delay × i`, i = 1…). Results are additive.
pub fn follow_through(src: &Channel, targets: &[String], delay: f32, decay: f32) -> Result<Vec<Channel>, String> {
    follow_through_within(src, targets, delay, decay, None)
}

/// [`follow_through`] that never writes a key after `last_frame` (the project's `frames`: the timeline ends there and `lint` L007 flags later keys). A delayed
/// copy of a track that ends near the end of the timeline used to push its last keys past it (lint L007); now the
/// shifted track is cut at `last_frame`: the keys after it are dropped and, so that the value at `last_frame` is
/// still the delayed curve's value, the frames from the last kept key to `last_frame` are sampled as linear keys
/// (exact on every whole frame, whatever the cut key's easing was).
pub fn follow_through_within(src: &Channel, targets: &[String], delay: f32, decay: f32, last_frame: Option<f32>) -> Result<Vec<Channel>, String> {
    let Source::Keys { track } = &src.source else { return Err("follow-through needs a keyed source channel".into()) };
    if !matches!(delay.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) || !(0.0..=1.5).contains(&decay) {
        return Err("delay must be ≥ 0 and decay within 0..1.5".into());
    }
    let mut out = Vec::new();
    for (i, name) in targets.iter().enumerate() {
        let n = (i + 1) as f32;
        let mut t = track.clone();
        for k in &mut t.keys {
            k.frame += delay * n;
            k.value *= decay.powf(n);
        }
        if let Some(last) = last_frame {
            if t.keys.iter().any(|k| k.frame > last) {
                let at_last = t.eval(last).unwrap_or(0.0);
                let kept: Vec<crate::anim::Key> = t.keys.iter().copied().filter(|k| k.frame <= last).collect();
                let from = kept.last().map(|k| k.frame).unwrap_or(last - 1.0);
                let mut keys = kept;
                let mut f = from.floor() + 1.0;
                while f < last {
                    keys.push(crate::anim::Key { frame: f, value: t.eval(f).unwrap_or(at_last), ease: crate::anim::Ease::Linear });
                    f += 1.0;
                }
                if keys.last().is_none_or(|k| k.frame < last) {
                    keys.push(crate::anim::Key { frame: last, value: at_last, ease: crate::anim::Ease::Linear });
                }
                t.keys = keys;
            }
        }
        out.push(Channel { target: name.clone(), prop: src.prop, source: Source::Keys { track: t }, mode: Mode::Add, weight: src.weight });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blinks_are_deterministic_spaced_and_valid() {
        let o = BlinkOptions::default();
        let a = blink_track(480, 24.0, &o).unwrap();
        let b = blink_track(480, 24.0, &o).unwrap();
        assert_eq!(a, b, "same seed, same blinks");
        assert_ne!(a, blink_track(480, 24.0, &BlinkOptions { seed: 2, ..o.clone() }).unwrap());
        assert!(a.keys.len() >= 8, "20 s at ~3.5 s ⇒ several blinks, got {}", a.keys.len());
        for w in a.keys.windows(2) {
            assert!(w[1].frame > w[0].frame, "keys strictly increasing");
        }
        // shut eye reaches the closed value
        assert!(a.keys.iter().any(|k| k.value == 0.0));
        assert!(a.keys.last().unwrap().frame < 480.0);
    }

    #[test]
    fn follow_through_within_never_passes_the_last_frame() {
        let src = Channel::keys("a", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 40.0, Ease::Linear), (20.0, 0.0, Ease::Linear)]));
        let names: Vec<String> = vec!["b".into(), "c".into()];
        let plain = follow_through(&src, &names, 4.0, 0.5).unwrap();
        let cut = follow_through_within(&src, &names, 4.0, 0.5, Some(20.0)).unwrap();
        for (p, c) in plain.iter().zip(&cut) {
            let (Source::Keys { track: tp }, Source::Keys { track: tc }) = (&p.source, &c.source) else { panic!() };
            assert!(tc.keys.iter().all(|k| k.frame <= 20.0), "{:?}", tc.keys);
            assert!(tp.keys.last().unwrap().frame > 20.0, "the uncut track does overshoot");
            // the delayed curve is reproduced on every whole frame up to the last one
            for f in 0..=20 {
                assert!((tp.eval(f as f32).unwrap() - tc.eval(f as f32).unwrap()).abs() < 1e-3, "frame {f}");
            }
        }
        // a track that fits is left alone, and None means "do not cut"
        let same = follow_through_within(&src, &names, 4.0, 0.5, Some(100.0)).unwrap();
        assert_eq!(same, plain);
        assert_eq!(follow_through_within(&src, &names, 4.0, 0.5, None).unwrap(), plain);
        // a delay that moves every key behind the last frame leaves one key at the last frame
        let late = follow_through_within(&src, &names, 40.0, 1.0, Some(20.0)).unwrap();
        let Source::Keys { track } = &late[0].source else { panic!() };
        assert!(track.keys.iter().all(|k| k.frame <= 20.0) && track.keys.last().unwrap().frame == 20.0);
    }

    #[test]
    fn blink_rejects_bad_options() {
        let bad = BlinkOptions { mean_interval_s: 0.0, ..Default::default() };
        assert!(blink_track(100, 24.0, &bad).is_err());
        let bad = BlinkOptions { jitter: 2.0, ..Default::default() };
        assert!(blink_track(100, 24.0, &bad).is_err());
        assert!(blink_track(100, 0.0, &BlinkOptions::default()).is_err());
    }

    #[test]
    fn breath_is_periodic_and_additive() {
        let c = breath_channel("chest", Prop::BoneScale, 0.05, 48.0).unwrap();
        assert_eq!(c.mode, Mode::Add);
        let Source::Keys { track } = &c.source else { panic!() };
        assert!((track.eval(24.0).unwrap() - 0.05).abs() < 1e-5);
        assert!((track.eval(48.0 + 24.0).unwrap() - 0.05).abs() < 1e-4, "loops");
        assert!(track.eval(0.0).unwrap().abs() < 1e-6);
        assert!(breath_channel("c", Prop::BoneScale, 1.0, 1.0).is_err());
    }

    #[test]
    fn follow_through_delays_and_decays() {
        let src = Channel::keys("spine", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 20.0, Ease::Linear)]));
        let out = follow_through(&src, &["neck".into(), "head".into()], 3.0, 0.5).unwrap();
        let Source::Keys { track } = &out[0].source else { panic!() };
        assert_eq!(track.keys[1].frame, 13.0);
        assert_eq!(track.keys[1].value, 10.0);
        let Source::Keys { track } = &out[1].source else { panic!() };
        assert_eq!(track.keys[1].frame, 16.0);
        assert_eq!(track.keys[1].value, 5.0);
        assert!(out.iter().all(|c| c.mode == Mode::Add));
        let expr = Channel::expr("a", Prop::BoneAngle, "1", Mode::Add);
        assert!(follow_through(&expr, &["b".into()], 1.0, 0.5).is_err());
        assert!(follow_through(&src, &["b".into()], -1.0, 0.5).is_err());
    }
}
