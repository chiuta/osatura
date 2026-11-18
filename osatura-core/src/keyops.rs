// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Timeline editing operations on keyed tracks, plus markers/regions and SMPTE timecode.
//!
//! Provenance note: generic dope-sheet maths (shift, scale about a pivot, range copy/paste,
//! batch easing). Nothing here depends on any product's behaviour or naming.

use crate::anim::{Ease, Key, Track};
use crate::project::{Channel, Source};
use serde::{Deserialize, Serialize};

/// A named point (`end == None`) or region on the timeline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub name: String,
    pub frame: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<f32>,
}

fn in_range(k: &Key, from: f32, to: f32) -> bool {
    k.frame >= from && k.frame <= to
}

/// Re-place `moved` keys into `t`; a moved key replaces a stationary key on the same frame.
fn place(t: &mut Track, moved: Vec<Key>) {
    t.keys.retain(|k| !moved.iter().any(|m| m.frame == k.frame));
    t.keys.extend(moved);
    t.keys.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    t.keys.dedup_by(|b, a| a.frame == b.frame); // two moved keys on one frame: first wins
}

/// Take the keys in `[from, to]` out of the track.
fn take(t: &mut Track, from: f32, to: f32) -> Vec<Key> {
    let (sel, rest): (Vec<Key>, Vec<Key>) = t.keys.drain(..).partition(|k| in_range(k, from, to));
    t.keys = rest;
    sel
}

/// Move every key in `[from, to]` by `delta` frames.
pub fn shift_keys(t: &mut Track, from: f32, to: f32, delta: f32) -> usize {
    let mut moved = take(t, from, to);
    for k in &mut moved {
        k.frame += delta;
    }
    let n = moved.len();
    place(t, moved);
    n
}

/// Scale the spacing of keys in `[from, to]` about `pivot` (`factor` 2 = twice as slow).
pub fn scale_keys(t: &mut Track, from: f32, to: f32, pivot: f32, factor: f32) -> Result<usize, String> {
    if (factor.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !factor.is_finite() {
        return Err("scale factor must be positive".into());
    }
    let mut moved = take(t, from, to);
    for k in &mut moved {
        k.frame = pivot + (k.frame - pivot) * factor;
    }
    let n = moved.len();
    place(t, moved);
    Ok(n)
}

/// Set the ease of every key in `[from, to]` (batch "ease in/out/both").
pub fn set_ease(t: &mut Track, from: f32, to: f32, ease: Ease) -> usize {
    let mut n = 0;
    for k in t.keys.iter_mut().filter(|k| in_range(k, from, to)) {
        k.ease = ease;
        n += 1;
    }
    n
}

/// Most keys a track may hold after a bake (so a hostile curve cannot exhaust memory).
pub const MAX_KEYS_AFTER_BAKE: usize = 200_000;

/// Most keys a whole project may hold after `keys curve` (the per-track limit alone does not bound a project with many tracks).
pub const MAX_PROJECT_KEYS_AFTER_BAKE: usize = 2_000_000;

/// Write a curve of `segease` into the segments that start at a key in `[from, to]`: the start key keeps its value and gets the
/// curve's first ease, and keys are inserted between it and the next key so that the ordinary key interpolation (linear or held)
/// reproduces the curve exactly. Returns how many segments were baked. The curve is not stored: the result is plain keys, which
/// stay editable but no longer know the curve. Errors are those of `BakeCurve::keys`, and a track that would exceed
/// [`MAX_KEYS_AFTER_BAKE`] keys is left untouched.
pub fn bake_curve(t: &mut Track, from: f32, to: f32, curve: &crate::segease::BakeCurve) -> Result<usize, String> {
    let plan = curve.keys()?;
    let ease_of = |hold: bool| if hold { Ease::Step } else { Ease::Linear };
    let mut out: Vec<Key> = Vec::with_capacity(t.keys.len());
    let mut baked = 0;
    for i in 0..t.keys.len() {
        let a = t.keys[i];
        let Some(b) = t.keys.get(i + 1).copied().filter(|b| in_range(&a, from, to) && b.frame > a.frame) else {
            out.push(a);
            continue;
        };
        out.push(Key { ease: ease_of(plan[0].hold), ..a });
        let mut prev = a.frame;
        for k in plan.iter().skip(1) {
            let frame = a.frame + (k.u as f32) * (b.frame - a.frame);
            if frame > prev && frame < b.frame {
                out.push(Key { frame, value: a.value + (b.value - a.value) * k.level as f32, ease: ease_of(k.hold) });
                prev = frame;
            }
        }
        baked += 1;
        if out.len() + (t.keys.len() - i - 1) > MAX_KEYS_AFTER_BAKE {
            return Err(format!("the bake would give more than {MAX_KEYS_AFTER_BAKE} keys (use a curve with fewer stops or steps, or a smaller range)"));
        }
    }
    t.keys = out;
    Ok(baked)
}

/// Copy keys in `[from, to]`, returned with frames relative to `from`.
pub fn copy_keys(t: &Track, from: f32, to: f32) -> Vec<Key> {
    t.keys.iter().filter(|k| in_range(k, from, to)).map(|k| Key { frame: k.frame - from, ..*k }).collect()
}

/// Paste relative keys at `at`, replacing keys on the same frames.
pub fn paste_keys(t: &mut Track, clip: &[Key], at: f32) {
    for k in clip {
        t.set_key(at + k.frame, k.value, k.ease);
    }
}

/// Delay (positive) or advance a whole channel's keys; non-key channels are returned unchanged.
pub fn time_offset(ch: &mut Channel, delta: f32) -> bool {
    if let Source::Keys { track } = &mut ch.source {
        for k in &mut track.keys {
            k.frame += delta;
        }
        true
    } else {
        false
    }
}

/// Reverse a track in time within `[from, to]` (keys keep their values; frames mirror).
pub fn reverse_keys(t: &mut Track, from: f32, to: f32) -> usize {
    let mut moved = take(t, from, to);
    for k in &mut moved {
        k.frame = from + to - k.frame;
    }
    let n = moved.len();
    place(t, moved);
    n
}

/// How [`timecode_mode`] counts frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TcMode {
    /// SMPTE drop-frame for 29.97 / 59.94 / 119.88, non-drop for every other rate.
    Auto,
    /// Force drop-frame (only defined for the 29.97 / 59.94 / 119.88 family).
    Drop,
    /// Force non-drop counting (29.97 is counted as 30 per timecode second and drifts from the wall clock).
    NoDrop,
}

/// How a frame rate is counted in a timecode.
enum TcBase {
    /// `nominal` frames per timecode second; `drop` frame numbers skipped at each minute except every 10th.
    Counted { nominal: u64, drop: u64 },
    /// Non-integer, non-NTSC rate: seconds follow the real clock, the frame field restarts at each
    /// whole second (so a second holds floor or ceil of `fps` frames).
    Real(f64),
}

fn tc_base(fps: f64, mode: TcMode) -> Result<TcBase, String> {
    if !fps.is_finite() || fps <= 0.0 {
        return Err(format!("frame rate must be a positive finite number, got {fps}"));
    }
    if fps > 1000.0 {
        return Err(format!("frame rate {fps} is unreasonably high for a timecode"));
    }
    // NTSC family: 24000/1001, 30000/1001, 60000/1001, 120000/1001 (23.976, 29.97, 59.94, 119.88)
    let ntsc = [(24u64, 24000.0 / 1001.0, false), (30, 30000.0 / 1001.0, true), (60, 60000.0 / 1001.0, true), (120, 120000.0 / 1001.0, true)]
        .into_iter()
        .find(|&(_, r, _)| (fps - r).abs() < 0.005);
    if let Some((nominal, _, can_drop)) = ntsc {
        let drop = if can_drop { nominal / 30 * 2 } else { 0 };
        return match mode {
            TcMode::Drop if !can_drop => Err("drop-frame timecode is only defined for 29.97, 59.94 and 119.88 fps".into()),
            TcMode::Drop | TcMode::Auto => Ok(TcBase::Counted { nominal, drop }),
            TcMode::NoDrop => Ok(TcBase::Counted { nominal, drop: 0 }),
        };
    }
    if mode == TcMode::Drop {
        return Err("drop-frame timecode is only defined for 29.97, 59.94 and 119.88 fps".into());
    }
    let r = fps.round();
    if (fps - r).abs() < 1e-3 {
        Ok(TcBase::Counted { nominal: r as u64, drop: 0 })
    } else {
        Ok(TcBase::Real(fps))
    }
}

fn first_frame_of_second(sec: u64, fps: f64) -> u64 {
    (sec as f64 * fps - 1e-9).ceil().max(0.0) as u64
}

/// SMPTE timecode `HH:MM:SS:FF` (`HH:MM:SS;FF` for drop-frame). 29.97 / 59.94 / 119.88 fps use
/// drop-frame, 23.976 counts 24 per second, other fractional rates (12.5) follow the real clock.
/// Errors for a frame rate that is not a positive finite number.
pub fn timecode(frame: u32, fps: f32) -> Result<String, String> {
    timecode_mode(frame, fps as f64, TcMode::Auto)
}

pub fn timecode_mode(frame: u32, fps: f64, mode: TcMode) -> Result<String, String> {
    let frame = frame as u64;
    let (total_s, ff, sep) = match tc_base(fps, mode)? {
        TcBase::Counted { nominal, drop } => {
            let mut f = frame;
            if drop > 0 {
                let fpm = nominal * 60 - drop; // frames in a minute that drops
                let fp10 = fpm * 10 + drop; // frames in ten minutes
                let (d, m) = (f / fp10, f % fp10);
                f += 9 * drop * d + if m >= drop { drop * ((m - drop) / fpm) } else { 0 };
            }
            (f / nominal, f % nominal, if drop > 0 { ';' } else { ':' })
        }
        TcBase::Real(r) => {
            let mut sec = (frame as f64 / r).floor() as u64;
            while first_frame_of_second(sec + 1, r) <= frame {
                sec += 1;
            }
            while sec > 0 && first_frame_of_second(sec, r) > frame {
                sec -= 1;
            }
            (sec, frame - first_frame_of_second(sec, r), ':')
        }
    };
    Ok(format!("{:02}:{:02}:{:02}{}{:02}", total_s / 3600, (total_s / 60) % 60, total_s % 60, sep, ff))
}

/// Inverse of [`timecode`]. The separator before the frame field picks the counting when it is
/// ambiguous: `;` (or `.`) = drop-frame, `:` = non-drop.
pub fn parse_timecode(tc: &str, fps: f32) -> Result<u32, String> {
    parse_timecode_mode(tc, fps as f64, TcMode::Auto)
}

pub fn parse_timecode_mode(tc: &str, fps: f64, mode: TcMode) -> Result<u32, String> {
    let tc = tc.trim();
    let sep_pos = tc.rfind([':', ';', '.']).ok_or("timecode must be HH:MM:SS:FF")?;
    let sep = tc.as_bytes()[sep_pos] as char;
    let parts: Vec<&str> = tc[..sep_pos].split(':').chain(std::iter::once(&tc[sep_pos + 1..])).collect();
    if parts.len() != 4 {
        return Err("timecode must be HH:MM:SS:FF".into());
    }
    let n: Result<Vec<u64>, _> = parts.iter().map(|p| p.parse::<u64>()).collect();
    let n = n.map_err(|_| "timecode fields must be whole numbers".to_string())?;
    if n[0] > 10_000 {
        return Err("timecode hours out of range".into());
    }
    // an explicit separator decides between drop and non-drop when the mode is Auto
    let mode = match (mode, sep) {
        (TcMode::Auto, ';') | (TcMode::Auto, '.') => TcMode::Drop,
        (TcMode::Auto, _) => {
            // ':' means non-drop for the NTSC family, the natural counting otherwise
            TcMode::NoDrop
        }
        (m, _) => m,
    };
    let frame = match tc_base(fps, mode)? {
        TcBase::Counted { nominal, drop } => {
            if n[3] >= nominal || n[2] >= 60 || n[1] >= 60 {
                return Err("timecode field out of range".into());
            }
            let total_min = n[0] * 60 + n[1];
            if drop > 0 && n[2] == 0 && n[3] < drop && total_min % 10 != 0 {
                return Err(format!("{tc} does not exist in drop-frame timecode (frames 0..{} are skipped at the start of each minute except every 10th)", drop - 1));
            }
            nominal * 3600 * n[0] + nominal * 60 * n[1] + nominal * n[2] + n[3] - drop * (total_min - total_min / 10)
        }
        TcBase::Real(r) => {
            if n[2] >= 60 || n[1] >= 60 {
                return Err("timecode field out of range".into());
            }
            let sec = n[0] * 3600 + n[1] * 60 + n[2];
            let f = first_frame_of_second(sec, r) + n[3];
            if f >= first_frame_of_second(sec + 1, r) {
                return Err("timecode field out of range".into());
            }
            f
        }
    };
    u32::try_from(frame).map_err(|_| "timecode is beyond the frame range".to_string())
}

/// Markers sorted by frame; returns the region/marker names active at `frame`.
pub fn active_markers(markers: &[Marker], frame: f32) -> Vec<&str> {
    markers
        .iter()
        .filter(|m| match m.end {
            Some(e) => frame >= m.frame && frame <= e,
            None => (frame - m.frame).abs() < 0.5,
        })
        .map(|m| m.name.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t() -> Track {
        Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 5.0, Ease::Linear), (20.0, 1.0, Ease::Linear), (30.0, 2.0, Ease::Linear)])
    }

    #[test]
    fn shift_moves_only_selected_and_merges_collisions() {
        let mut a = t();
        assert_eq!(shift_keys(&mut a, 10.0, 20.0, 10.0), 2);
        let f: Vec<f32> = a.keys.iter().map(|k| k.frame).collect();
        assert_eq!(f, vec![0.0, 20.0, 30.0], "key shifted onto frame 30 replaced the old one");
        assert_eq!(a.keys[2].value, 1.0);
    }

    #[test]
    fn scale_about_pivot() {
        let mut a = t();
        scale_keys(&mut a, 0.0, 30.0, 0.0, 2.0).unwrap();
        assert_eq!(a.keys.last().unwrap().frame, 60.0);
        assert!(scale_keys(&mut a, 0.0, 1.0, 0.0, 0.0).is_err());
        assert!(scale_keys(&mut a, 0.0, 1.0, 0.0, f32::NAN).is_err());
    }

    #[test]
    fn copy_paste_round_trip() {
        let mut a = t();
        let clip = copy_keys(&a, 10.0, 20.0);
        assert_eq!(clip.iter().map(|k| k.frame).collect::<Vec<_>>(), vec![0.0, 10.0]);
        paste_keys(&mut a, &clip, 40.0);
        assert_eq!(a.eval(40.0), Some(5.0));
        assert_eq!(a.eval(50.0), Some(1.0));
    }

    #[test]
    fn batch_ease_and_reverse() {
        let mut a = t();
        assert_eq!(set_ease(&mut a, 0.0, 10.0, Ease::EaseInOut), 2);
        assert_eq!(a.keys[1].ease, Ease::EaseInOut);
        assert_eq!(a.keys[2].ease, Ease::Linear);
        let mut b = t();
        reverse_keys(&mut b, 0.0, 30.0);
        assert_eq!(b.keys[0].value, 2.0);
        assert_eq!(b.keys[3].value, 0.0);
    }

    #[test]
    fn offset_applies_to_key_channels_only() {
        let mut c = Channel::keys("b", crate::project::Prop::BoneAngle, t());
        assert!(time_offset(&mut c, 3.0));
        if let Source::Keys { track } = &c.source {
            assert_eq!(track.keys[0].frame, 3.0);
        }
        let mut e = Channel::expr("b", crate::project::Prop::BoneAngle, "1", crate::project::Mode::Add);
        assert!(!time_offset(&mut e, 3.0));
    }

    #[test]
    fn timecode_round_trip_and_errors() {
        assert_eq!(timecode(0, 24.0).unwrap(), "00:00:00:00");
        assert_eq!(timecode(24 * 3661 + 5, 24.0).unwrap(), "01:01:01:05");
        for f in [0, 23, 24, 1000, 90_000] {
            assert_eq!(parse_timecode(&timecode(f, 24.0).unwrap(), 24.0).unwrap(), f);
        }
        assert!(parse_timecode("00:00:00:24", 24.0).is_err());
        assert!(parse_timecode("1:2:3", 24.0).is_err());
        assert!(parse_timecode("a:b:c:d", 24.0).is_err());
    }

    /// C4: SMPTE drop-frame values (ST 12-1): 29.97 skips frame numbers 0,1 at each minute but every 10th.
    #[test]
    fn drop_frame_known_values_and_inverse() {
        let df = |f: u32| timecode_mode(f, 30000.0 / 1001.0, TcMode::Auto).unwrap();
        assert_eq!(df(0), "00:00:00;00");
        assert_eq!(df(1799), "00:00:59;29");
        assert_eq!(df(1800), "00:01:00;02");
        assert_eq!(df(17980), "00:09:59;28");
        assert_eq!(df(17981), "00:09:59;29");
        assert_eq!(df(17982), "00:10:00;00");
        assert_eq!(df(17982 + 1798 + 2), "00:11:00;02"); // minute 11 drops again
        assert_eq!(df(107_892), "01:00:00;00"); // one hour of 29.97 DF = 107892 frames
        for f in (0..300_000).step_by(7).chain([1799, 1800, 1801, 17981, 17982, 17983, 107_891, 107_892]) {
            let s = df(f);
            assert_eq!(parse_timecode_mode(&s, 30000.0 / 1001.0, TcMode::Auto).unwrap(), f, "{s}");
            assert_eq!(parse_timecode(&s, 29.97).unwrap(), f);
        }
        // every label is unique and monotone
        let mut prev = String::new();
        for f in 0..40_000 {
            let s = df(f);
            assert!(s > prev || f == 0, "{prev} then {s}");
            prev = s;
        }
        // the dropped labels do not exist
        assert!(parse_timecode("00:01:00;00", 29.97).is_err());
        assert!(parse_timecode("00:01:00;01", 29.97).is_err());
        assert!(parse_timecode("00:10:00;00", 29.97).is_ok());
        // 59.94 drops 4
        let df60 = |f: u32| timecode_mode(f, 60000.0 / 1001.0, TcMode::Auto).unwrap();
        assert_eq!(df60(3600), "00:01:00;04");
        assert_eq!(df60(35964), "00:10:00;00");
        assert_eq!(parse_timecode(&df60(123_456), 59.94).unwrap(), 123_456);
        // drop-frame stays within one frame of the wall clock after an hour
        let secs = 107_892f64 * 1001.0 / 30000.0;
        assert!((secs - 3600.0).abs() < 0.01);
    }

    #[test]
    fn non_drop_ntsc_fractional_rates_and_bad_fps() {
        // 23.976 counts 24 per second, with ':'
        assert_eq!(timecode_mode(24, 24000.0 / 1001.0, TcMode::Auto).unwrap(), "00:00:01:00");
        assert_eq!(timecode(1440, 23.976).unwrap(), "00:01:00:00");
        // 29.97 forced non-drop counts 30 and uses ':'
        assert_eq!(timecode_mode(1800, 29.97, TcMode::NoDrop).unwrap(), "00:01:00:00");
        assert_eq!(parse_timecode("00:01:00:00", 29.97).unwrap(), 1800);
        // drop-frame is refused where it is not defined
        assert!(timecode_mode(10, 30.0, TcMode::Drop).is_err());
        assert!(timecode_mode(10, 23.976, TcMode::Drop).is_err());
        assert!(timecode_mode(10, 12.5, TcMode::Drop).is_err());
        // 12.5 fps: seconds follow the real clock (frame 25 = exactly 2 s), FF restarts each second
        assert_eq!(timecode(0, 12.5).unwrap(), "00:00:00:00");
        assert_eq!(timecode(12, 12.5).unwrap(), "00:00:00:12");
        assert_eq!(timecode(13, 12.5).unwrap(), "00:00:01:00");
        assert_eq!(timecode(24, 12.5).unwrap(), "00:00:01:11");
        assert_eq!(timecode(25, 12.5).unwrap(), "00:00:02:00");
        assert_eq!(timecode(12 * 25, 12.5).unwrap(), "00:00:24:00");
        for f in 0..2000 {
            assert_eq!(parse_timecode(&timecode(f, 12.5).unwrap(), 12.5).unwrap(), f);
        }
        assert!(parse_timecode("00:00:01:12", 12.5).is_err()); // second 1 has only 12 frames (13..24)
        // rejects nonsense rates
        for bad in [0.0f64, -24.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e9] {
            assert!(timecode_mode(5, bad, TcMode::Auto).is_err(), "{bad}");
            assert!(parse_timecode_mode("00:00:00:01", bad, TcMode::Auto).is_err());
        }
        assert!(timecode(5, f32::NAN).is_err());
    }

    #[test]
    fn markers_regions() {
        let m = vec![
            Marker { name: "hit".into(), frame: 12.0, end: None },
            Marker { name: "walk".into(), frame: 0.0, end: Some(20.0) },
        ];
        assert_eq!(active_markers(&m, 12.0), vec!["hit", "walk"]);
        assert_eq!(active_markers(&m, 30.0), Vec::<&str>::new());
    }

    #[test]
    fn baked_curves_reproduce_the_curve_through_the_ordinary_key_interpolation() {
        use crate::segease::BakeCurve;
        let base = Track::with_keys(&[(0.0, 10.0, Ease::Linear), (20.0, 30.0, Ease::Linear), (40.0, 0.0, Ease::EaseIn)]);
        // piecewise: 0 -> 0.25 at 75% -> 1 (overshoot-free), first segment only
        let c = BakeCurve::parse("piecewise:0,0.25 75%,1").unwrap();
        let mut t = base.clone();
        assert_eq!(bake_curve(&mut t, 0.0, 5.0, &c).unwrap(), 1);
        assert_eq!(t.keys.len(), 4);
        for f in [0.0, 3.0, 10.0, 15.0, 17.5, 20.0] {
            let u = f / 20.0;
            let want = 10.0 + 20.0 * if let BakeCurve::Piecewise(p) = &c { p.eval(u as f64) as f32 } else { 0.0 };
            assert!((t.eval(f).unwrap() - want).abs() < 1e-4, "frame {f}");
        }
        // the second segment and its ease are untouched
        assert_eq!(t.keys[3], base.keys[2]);
        assert_eq!(t.eval(30.0), base.eval(30.0));
        // steps:4 on the second segment: four plateaus, the jump at each quarter
        let s = BakeCurve::parse("steps:4").unwrap();
        let mut t = base.clone();
        assert_eq!(bake_curve(&mut t, 20.0, 20.0, &s).unwrap(), 1);
        for (f, want) in [(20.0, 30.0), (24.9, 30.0), (25.0, 22.5), (29.9, 22.5), (30.0, 15.0), (35.0, 7.5), (39.9, 7.5), (40.0, 0.0)] {
            assert!((t.eval(f).unwrap() - want).abs() < 1e-4, "frame {f}: {:?}", t.eval(f));
        }
        // the last key has no next key: nothing to bake
        let mut one = Track::with_keys(&[(0.0, 1.0, Ease::Linear)]);
        assert_eq!(bake_curve(&mut one, 0.0, 9.0, &s).unwrap(), 0);
        // a failing bake leaves the track as it was
        let mut t = base.clone();
        let jump_start = BakeCurve::parse("steps:2:jump-start").unwrap();
        assert!(bake_curve(&mut t, 0.0, 40.0, &jump_start).is_err());
        assert_eq!(t, base);
    }
}
