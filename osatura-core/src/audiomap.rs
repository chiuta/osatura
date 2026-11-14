// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Audio-driven animation: waveform peaks for display, a smoothed loudness channel that can
//! drive any property (sound-reactive motion), and beat/onset detection written as timeline
//! markers or events. All offline and deterministic.

use crate::anim::{Ease, Track};
use crate::audio::{frame_features, Audio};
use crate::comps::Event;
use crate::keyops::Marker;
use crate::project::{Channel, Mode, Project, Prop};

/// `(min, max)` per bucket for drawing a waveform.
pub fn waveform_peaks(a: &Audio, buckets: usize) -> Result<Vec<(f32, f32)>, String> {
    if buckets == 0 || buckets > 100_000 {
        return Err("buckets must be within 1..100000".into());
    }
    if a.samples.is_empty() {
        return Ok(vec![(0.0, 0.0); buckets]);
    }
    let n = a.samples.len();
    Ok((0..buckets)
        .map(|b| {
            let (s, e) = (b * n / buckets, ((b + 1) * n / buckets).max(b * n / buckets + 1).min(n));
            a.samples[s..e].iter().fold((f32::MAX, f32::MIN), |(lo, hi), &x| (lo.min(x), hi.max(x)))
        })
        .collect())
}

#[derive(Clone, Debug)]
pub struct LevelOptions {
    /// Output = floor-gated level × gain + offset.
    pub gain: f32,
    pub offset: f32,
    /// Levels below this (0..1, relative to the loudest frame) count as silence.
    pub floor: f32,
    /// Frames the value takes to rise / fall (0 = instant).
    pub attack: f32,
    pub release: f32,
    pub additive: bool,
}

impl Default for LevelOptions {
    fn default() -> Self {
        LevelOptions { gain: 1.0, offset: 0.0, floor: 0.05, attack: 1.0, release: 4.0, additive: false }
    }
}

/// One key per frame following the smoothed loudness.
pub fn level_channel(target: &str, prop: Prop, a: &Audio, fps: f32, frames: u32, o: &LevelOptions) -> Result<Channel, String> {
    if (fps.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || frames == 0 || ![o.gain, o.offset, o.floor, o.attack, o.release].iter().all(|v| v.is_finite()) || o.attack < 0.0 || o.release < 0.0 || !(0.0..1.0).contains(&o.floor) {
        return Err("invalid level options".into());
    }
    let feats = frame_features(a, fps, frames);
    let mut level = 0.0f32;
    let keys: Vec<(f32, f32, Ease)> = feats
        .iter()
        .enumerate()
        .map(|(f, (rms, _))| {
            let target_level = if *rms < o.floor { 0.0 } else { (rms - o.floor) / (1.0 - o.floor) };
            let tau = if target_level > level { o.attack } else { o.release };
            let k = if tau <= 0.0 { 1.0 } else { 1.0 - (-1.0 / tau).exp() };
            level += (target_level - level) * k;
            (f as f32, level * o.gain + o.offset, Ease::Linear)
        })
        .collect();
    let ch = Channel::keys(target, prop, Track::with_keys(&keys));
    Ok(if o.additive { ch.with_mode(Mode::Add) } else { ch.with_mode(Mode::Replace) })
}

/// Frames where loudness jumps above its recent average (`sensitivity` 1.2…3, higher = fewer
/// beats) with at least `min_gap` frames between beats.
pub fn detect_beats(a: &Audio, fps: f32, frames: u32, sensitivity: f32, min_gap: u32) -> Result<Vec<u32>, String> {
    if (sensitivity.partial_cmp(&1.0) != Some(std::cmp::Ordering::Greater)) || !sensitivity.is_finite() || fps <= 0.0 {
        return Err("sensitivity must be above 1".into());
    }
    let e: Vec<f32> = frame_features(a, fps, frames).iter().map(|x| x.0).collect();
    let win = (fps * 0.5).round().max(3.0) as usize;
    let mut out: Vec<u32> = Vec::new();
    for i in 1..e.len() {
        let lo = i.saturating_sub(win);
        let avg = e[lo..i].iter().sum::<f32>() / (i - lo) as f32;
        let rising = e[i] > e[i - 1];
        if rising && e[i] > 0.15 && e[i] > avg * sensitivity && out.last().is_none_or(|&l| i as u32 - l >= min_gap.max(1)) {
            out.push(i as u32);
        }
    }
    Ok(out)
}

pub fn beats_to_markers(p: &mut Project, beats: &[u32], name: &str) {
    p.markers.extend(beats.iter().map(|&f| Marker { name: name.into(), frame: f as f32, end: None }));
}

pub fn beats_to_events(p: &mut Project, beats: &[u32], name: &str) {
    p.events.extend(beats.iter().map(|&f| Event { name: name.into(), frame: f as f32, data: serde_json::Value::Null }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Source;

    /// 24 fps, 2 s: silence, then clicks (short loud bursts) at 0.5 s, 1.0 s, 1.5 s.
    fn clicks() -> Audio {
        let sr = 8000u32;
        let mut s = vec![0.0f32; sr as usize * 2];
        for t in [0.5f32, 1.0, 1.5] {
            let i0 = (t * sr as f32) as usize;
            for k in 0..400 {
                s[i0 + k] = if k % 2 == 0 { 0.8 } else { -0.8 };
            }
        }
        Audio { samples: s, sample_rate: sr }
    }

    #[test]
    fn peaks_cover_the_signal() {
        let p = waveform_peaks(&clicks(), 20).unwrap();
        assert_eq!(p.len(), 20);
        assert!(p[0] == (0.0, 0.0) && p[5].1 > 0.7 && p[5].0 < -0.7);
        assert_eq!(waveform_peaks(&Audio { samples: vec![], sample_rate: 8000 }, 4).unwrap().len(), 4);
        assert!(waveform_peaks(&clicks(), 0).is_err());
    }

    #[test]
    fn level_follows_loudness_with_release() {
        let ch = level_channel("a", Prop::BoneScale, &clicks(), 24.0, 48, &LevelOptions::default()).unwrap();
        let Source::Keys { track } = &ch.source else { panic!() };
        let v = |f: f32| track.eval(f).unwrap();
        assert!(v(2.0) < 0.01, "silence");
        assert!(v(12.0) > 0.3, "click at 0.5 s = frame 12");
        assert!(v(14.0) < v(12.0) && v(14.0) > 0.0, "release is gradual");
        let instant = level_channel("a", Prop::BoneScale, &clicks(), 24.0, 48, &LevelOptions { release: 0.0, ..Default::default() }).unwrap();
        let Source::Keys { track: t2 } = &instant.source else { panic!() };
        assert!(t2.eval(16.0).unwrap() < 0.05);
        assert!(level_channel("a", Prop::BoneScale, &clicks(), 24.0, 48, &LevelOptions { floor: 1.5, ..Default::default() }).is_err());
        let add = level_channel("a", Prop::BoneScale, &clicks(), 24.0, 48, &LevelOptions { additive: true, ..Default::default() }).unwrap();
        assert_eq!(add.mode, Mode::Add);
    }

    #[test]
    fn beats_found_near_the_clicks_and_gap_respected() {
        let b = detect_beats(&clicks(), 24.0, 48, 1.5, 4).unwrap();
        assert_eq!(b.len(), 3, "{b:?}");
        for (got, want) in b.iter().zip([12u32, 24, 36]) {
            assert!((*got as i32 - want as i32).abs() <= 1, "{got} vs {want}");
        }
        assert!(detect_beats(&clicks(), 24.0, 48, 1.0, 4).is_err());
        let mut p = Project::new("b", 10.0, 10.0, 24.0, 48);
        beats_to_markers(&mut p, &b, "beat");
        beats_to_events(&mut p, &b, "hit");
        assert_eq!((p.markers.len(), p.events.len()), (3, 3));
    }
}
