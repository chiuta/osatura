// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Layered clip stack: non-destructive layered animation.
//!
//! A [`Clip`] is an instance of a motion-library take placed on the timeline (start frame, speed,
//! loop count, blend mode, weight that may be keyed, per-target masks, per-track weight rules,
//! fade-in/out). Clips live in `Project.clips`; later clips layer over earlier ones. Each clip
//! embeds a copy of its take, so a project stays self-contained.
//!
//! [`bake`] turns the stack into channels, sampled once per integer frame (linear in between):
//!
//! * replace: `cur*(1-w) + v*w` as a multiply channel `(1-w)` followed by an add channel `v*w`
//!   (exact, even when `w` varies per frame);
//! * additive: an add channel `w*(v - v_first)`, i.e. the delta from the clip's first-frame pose;
//! * multiply: a multiply channel `1 + (v-1)*w`.
//!
//! The baked channels are appended after all hand-made channels, and a snapshot of exactly what was
//! generated is kept in `Project.clip_baked`. Re-baking removes channels equal to that snapshot and
//! nothing else, so hand-made channels (and hand-edited copies of baked ones) are never touched, and
//! baking twice gives the same project. Locked channels make a bake fail without changing anything.

use crate::anim::{Ease, Track};
use crate::motionlib::{Library, Take};
use crate::project::{Channel, Mode, Project, Prop};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipMode {
    /// Blend toward the clip's value by the weight.
    #[default]
    Replace,
    /// Add the clip's change relative to its first-frame pose, scaled by the weight.
    Additive,
    /// Scale what is below by the clip's value (1 = no change), blended by the weight.
    Multiply,
}

/// Weight factor for the targets (glob, `*` and `?`) a rule matches; optionally only one property.
/// The first matching rule of a clip applies, and it multiplies the clip's own weight.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrackWeight {
    pub pattern: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prop: Option<Prop>,
    pub weight: f32,
}

fn one() -> f32 {
    1.0
}
fn one_u() -> u32 {
    1
}
fn track_empty(t: &Track) -> bool {
    t.keys.is_empty()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub name: String,
    /// Embedded copy of the take (its name is the library name it came from).
    pub take: Take,
    /// First project frame of the clip.
    #[serde(default)]
    pub start: f32,
    /// Playback speed; 2 plays twice as fast.
    #[serde(default = "one")]
    pub speed: f32,
    /// How many times the take repeats (>= 1).
    #[serde(default = "one_u")]
    pub loops: u32,
    #[serde(default)]
    pub mode: ClipMode,
    /// Constant weight 0..1, used when `weight_keys` is empty.
    #[serde(default = "one")]
    pub weight: f32,
    /// Weight over project time (frames); when it has keys it replaces `weight` (clamped to 0..1).
    #[serde(default, skip_serializing_if = "track_empty")]
    pub weight_keys: Track,
    /// Only targets matching one of these globs (empty = all).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<String>,
    /// Targets matching these globs are skipped.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub track_weights: Vec<TrackWeight>,
    /// Frames over which the weight ramps up from 0 at the start (smoothstep).
    #[serde(default)]
    pub fade_in: f32,
    /// Frames over which the weight ramps down to 0 at the end (smoothstep).
    #[serde(default)]
    pub fade_out: f32,
    /// Take target -> project target. Masks apply to the renamed target.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rename: Vec<(String, String)>,
}

impl Clip {
    pub fn new(name: &str, take: Take) -> Clip {
        Clip {
            name: name.into(),
            take,
            start: 0.0,
            speed: 1.0,
            loops: 1,
            mode: ClipMode::Replace,
            weight: 1.0,
            weight_keys: Track::new(),
            include: vec![],
            exclude: vec![],
            track_weights: vec![],
            fade_in: 0.0,
            fade_out: 0.0,
            rename: vec![],
        }
    }

    /// Project frame at which the last loop ends.
    pub fn end(&self) -> f32 {
        self.start + self.loops as f32 * self.take.length / self.speed
    }

    fn target_of(&self, take_target: &str) -> String {
        self.rename.iter().find(|(f, _)| f == take_target).map_or(take_target, |(_, t)| t.as_str()).to_string()
    }

    /// Whether the mask lets `target` (project name) through.
    pub fn passes_mask(&self, target: &str) -> bool {
        (self.include.is_empty() || self.include.iter().any(|g| glob(g, target))) && !self.exclude.iter().any(|g| glob(g, target))
    }

    /// Effective weight of the clip for one channel at a project frame: base (constant or keyed)
    /// times the matching per-track factor times the fades. 0 outside `[start, end]`.
    pub fn weight_at(&self, target: &str, prop: Prop, frame: f32) -> f32 {
        let end = self.end();
        if frame < self.start || frame > end {
            return 0.0;
        }
        let mut w = if self.weight_keys.keys.is_empty() { self.weight } else { self.weight_keys.eval(frame).unwrap_or(self.weight) };
        w = w.clamp(0.0, 1.0);
        if let Some(r) = self.track_weights.iter().find(|r| glob(&r.pattern, target) && r.prop.is_none_or(|p| p == prop)) {
            w *= r.weight;
        }
        if self.fade_in > 0.0 {
            w *= smooth01((frame - self.start) / self.fade_in);
        }
        if self.fade_out > 0.0 {
            w *= smooth01((end - frame) / self.fade_out);
        }
        w
    }

    /// Raw take value at a project frame (looping and speed applied); `None` outside the clip.
    pub fn value_at(&self, track: &Track, frame: f32) -> Option<f32> {
        if frame < self.start || frame > self.end() {
            return None;
        }
        let len = self.take.length;
        let u = (frame - self.start) * self.speed;
        let total = self.loops as f32 * len;
        let local = if u >= total { len } else { u.rem_euclid(len) };
        track.eval(local)
    }

    pub fn validate(&self) -> Result<(), String> {
        let n = &self.name;
        if self.name.trim().is_empty() {
            return Err("clip needs a name".into());
        }
        let fin = |v: f32, what: &str| if v.is_finite() { Ok(()) } else { Err(format!("clip '{n}': {what} must be a finite number")) };
        fin(self.start, "start")?;
        fin(self.speed, "speed")?;
        fin(self.weight, "weight")?;
        fin(self.fade_in, "fade-in")?;
        fin(self.fade_out, "fade-out")?;
        if self.speed.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
            return Err(format!("clip '{n}': speed must be positive"));
        }
        if self.loops == 0 || self.loops > 100_000 {
            return Err(format!("clip '{n}': loops must be 1..100000"));
        }
        if !(0.0..=1.0).contains(&self.weight) {
            return Err(format!("clip '{n}': weight must be within 0..1"));
        }
        if self.fade_in < 0.0 || self.fade_out < 0.0 {
            return Err(format!("clip '{n}': fades must be >= 0"));
        }
        if !self.take.length.is_finite() || (self.take.length.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) {
            return Err(format!("clip '{n}': take length must be positive"));
        }
        if !self.end().is_finite() {
            return Err(format!("clip '{n}': duration is not finite"));
        }
        for t in &self.take.tracks {
            if t.keys.iter().any(|k| !k.frame.is_finite() || !k.value.is_finite()) {
                return Err(format!("clip '{n}': take track {} has a non-finite key", t.target));
            }
        }
        if self.weight_keys.keys.iter().any(|k| !k.frame.is_finite() || !k.value.is_finite()) {
            return Err(format!("clip '{n}': weight keys must be finite"));
        }
        for r in &self.track_weights {
            if r.pattern.is_empty() || !r.weight.is_finite() || !(0.0..=1.0).contains(&r.weight) {
                return Err(format!("clip '{n}': track weight '{}' needs a pattern and a weight within 0..1", r.pattern));
            }
        }
        if self.include.iter().chain(&self.exclude).any(|g| g.is_empty()) {
            return Err(format!("clip '{n}': empty mask pattern"));
        }
        Ok(())
    }
}

fn smooth01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Glob match: `*` any run of characters, `?` one character.
pub fn glob(pat: &str, s: &str) -> bool {
    fn go(p: &[char], s: &[char]) -> bool {
        match p.split_first() {
            None => s.is_empty(),
            Some(('*', rest)) => (0..=s.len()).any(|i| go(rest, &s[i..])),
            Some(('?', rest)) => !s.is_empty() && go(rest, &s[1..]),
            Some((c, rest)) => s.first() == Some(c) && go(rest, &s[1..]),
        }
    }
    let (p, s): (Vec<char>, Vec<char>) = (pat.chars().collect(), s.chars().collect());
    go(&p, &s)
}

/// Validate every clip and that names are unique.
pub fn validate(p: &Project) -> Result<(), String> {
    for (i, c) in p.clips.iter().enumerate() {
        c.validate()?;
        if p.clips[..i].iter().any(|o| o.name == c.name) {
            return Err(format!("duplicate clip name '{}'", c.name));
        }
    }
    Ok(())
}

/// Add a clip on top of the stack, embedding a copy of library take `take_name`.
pub fn add(p: &mut Project, lib: &Library, take_name: &str, mut clip: Clip) -> Result<(), String> {
    let take = lib.find(take_name).ok_or_else(|| format!("no take '{take_name}' in the library"))?;
    clip.take = take.clone();
    add_clip(p, clip)
}

/// Add a clip that already carries its take.
pub fn add_clip(p: &mut Project, clip: Clip) -> Result<(), String> {
    clip.validate()?;
    if p.clips.iter().any(|c| c.name == clip.name) {
        return Err(format!("clip '{}' already exists", clip.name));
    }
    p.clips.push(clip);
    Ok(())
}

pub fn remove(p: &mut Project, name: &str) -> Result<(), String> {
    let i = p.clips.iter().position(|c| c.name == name).ok_or_else(|| format!("no clip '{name}'"))?;
    p.clips.remove(i);
    Ok(())
}

fn find_mut<'a>(p: &'a mut Project, name: &str) -> Result<&'a mut Clip, String> {
    p.clips.iter_mut().find(|c| c.name == name).ok_or_else(|| format!("no clip '{name}'"))
}

/// Constant clip weight (clears keyed weight).
pub fn set_weight(p: &mut Project, name: &str, w: f32) -> Result<(), String> {
    let c = find_mut(p, name)?;
    let mut n = c.clone();
    n.weight = w;
    n.weight_keys = Track::new();
    n.validate()?;
    *c = n;
    Ok(())
}

/// Key the clip weight over project frames (`(frame, weight)` pairs, linear).
pub fn set_weight_keys(p: &mut Project, name: &str, keys: &[(f32, f32)]) -> Result<(), String> {
    if keys.is_empty() || keys.iter().any(|(f, v)| !f.is_finite() || !v.is_finite() || !(0.0..=1.0).contains(v)) {
        return Err("weight keys need finite frames and weights within 0..1".into());
    }
    let c = find_mut(p, name)?;
    c.weight_keys = Track::with_keys(&keys.iter().map(|&(f, v)| (f, v, Ease::Linear)).collect::<Vec<_>>());
    Ok(())
}

/// Set (or replace) the per-track weight rule with the same pattern and property.
pub fn set_track_weight(p: &mut Project, name: &str, pattern: &str, prop: Option<Prop>, w: f32) -> Result<(), String> {
    let rule = TrackWeight { pattern: pattern.into(), prop, weight: w };
    let c = find_mut(p, name)?;
    let mut n = c.clone();
    match n.track_weights.iter_mut().find(|r| r.pattern == pattern && r.prop == prop) {
        Some(r) => *r = rule,
        None => n.track_weights.push(rule),
    }
    n.validate()?;
    *c = n;
    Ok(())
}

/// Drop interior keys that lie on the line between their neighbours.
fn simplify(keys: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let mut out: Vec<(f32, f32)> = Vec::with_capacity(keys.len());
    for i in 0..keys.len() {
        if i > 0 && i + 1 < keys.len() {
            let (a, b, c) = (out[out.len() - 1], keys[i], keys[i + 1]);
            let t = (b.0 - a.0) / (c.0 - a.0);
            if (a.1 + (c.1 - a.1) * t - b.1).abs() < 1e-6 {
                continue;
            }
        }
        out.push(keys[i]);
    }
    out
}

fn neutral_track(samples: &[(f32, f32)], neutral: f32, lo: i64, hi: i64, frames: u32) -> Option<Track> {
    if samples.iter().all(|&(_, v)| (v - neutral).abs() < 1e-7) {
        return None;
    }
    let mut keys = Vec::with_capacity(samples.len() + 2);
    if lo >= 1 {
        keys.push(((lo - 1) as f32, neutral));
    }
    keys.extend_from_slice(samples);
    if hi < frames as i64 - 1 {
        keys.push(((hi + 1) as f32, neutral));
    }
    let keys = simplify(keys);
    Some(Track::with_keys(&keys.iter().map(|&(f, v)| (f, v, Ease::Linear)).collect::<Vec<_>>()))
}

/// Channels the stack generates, in clip order. Pure: depends only on `p.clips` and the frame count.
pub fn generate(p: &Project) -> Result<Vec<Channel>, String> {
    validate(p)?;
    let mut out = Vec::new();
    let last = p.frames as i64 - 1;
    for clip in &p.clips {
        let lo = (clip.start.ceil() as i64).max(0);
        let hi = (clip.end().floor() as i64).min(last);
        if lo > hi {
            continue;
        }
        for tt in &clip.take.tracks {
            let target = clip.target_of(&tt.target);
            if !clip.passes_mask(&target) || tt.keys.is_empty() {
                continue;
            }
            let track = Track::with_keys(&tt.keys.iter().map(|k| (k.frame, k.value, k.ease)).collect::<Vec<_>>());
            let base = track.keys.first().map_or(0.0, |k| k.value);
            let (mut mul, mut add) = (Vec::new(), Vec::new());
            for f in lo..=hi {
                let fr = f as f32;
                let w = clip.weight_at(&target, tt.prop, fr);
                let v = clip.value_at(&track, fr).unwrap_or(base);
                match clip.mode {
                    ClipMode::Replace => {
                        mul.push((fr, 1.0 - w));
                        add.push((fr, v * w));
                    }
                    ClipMode::Additive => add.push((fr, w * (v - base))),
                    ClipMode::Multiply => mul.push((fr, 1.0 + (v - 1.0) * w)),
                }
            }
            if let Some(t) = neutral_track(&mul, 1.0, lo, hi, p.frames) {
                out.push(Channel::keys(&target, tt.prop, t).with_mode(Mode::Multiply));
            }
            if let Some(t) = neutral_track(&add, 0.0, lo, hi, p.frames) {
                out.push(Channel::keys(&target, tt.prop, t).with_mode(Mode::Add));
            }
        }
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq)]
pub struct BakeReport {
    pub clips: usize,
    /// Channels now owned by the stack.
    pub channels: usize,
    /// Previously baked channels that were replaced.
    pub replaced: usize,
}

/// Re-generate the stack's channels: remove the ones from the last bake, append the new ones.
/// Fails (leaving `p` unchanged) on invalid clips, locked channels, or targets that do not exist.
pub fn bake(p: &mut Project) -> Result<BakeReport, String> {
    let fresh = generate(p)?;
    for c in &fresh {
        crate::chanmeta::assert_unlocked(p, &c.target, c.prop).map_err(|e| format!("{e}; exclude it in the clip mask"))?;
    }
    let mut q = p.clone();
    let mut replaced = 0;
    for old in std::mem::take(&mut q.clip_baked) {
        if let Some(i) = q.channels.iter().position(|c| *c == old) {
            crate::chanmeta::assert_unlocked(p, &old.target, old.prop)?;
            q.channels.remove(i);
            replaced += 1;
        }
    }
    let n = fresh.len();
    q.channels.extend(fresh.iter().cloned());
    q.clip_baked = fresh;
    crate::Engine::new(q.clone()).map_err(|e| format!("bake produces an invalid project: {}", e.join("; ")))?;
    let clips = q.clips.len();
    *p = q;
    Ok(BakeReport { clips, channels: n, replaced })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::Key;
    use crate::math::Vec2;
    use crate::motionlib::TakeTrack;
    use crate::project::Bone;
    use crate::Engine;

    fn k(f: f32, v: f32) -> Key {
        Key { frame: f, value: v, ease: Ease::Linear }
    }

    /// Take of length 10: arm_up angle 10 -> 30, leg angle 0 -> 20.
    fn take() -> Take {
        Take {
            name: "wave".into(),
            length: 10.0,
            tracks: vec![
                TakeTrack { target: "arm_up".into(), prop: Prop::BoneAngle, keys: vec![k(0.0, 10.0), k(10.0, 30.0)] },
                TakeTrack { target: "leg".into(), prop: Prop::BoneAngle, keys: vec![k(0.0, 0.0), k(10.0, 20.0)] },
            ],
        }
    }

    fn proj() -> Project {
        let mut p = Project::new("c", 100.0, 100.0, 24.0, 60);
        p.bones = ["arm_up", "leg", "arm_low"].iter().enumerate().map(|(i, n)| Bone::new(n, None, 10.0, Vec2::new(10.0 * i as f32, 0.0), 0.0)).collect();
        // hand-made: everything at 100 degrees
        for b in ["arm_up", "leg", "arm_low"] {
            p.channels.push(Channel::keys(b, Prop::BoneAngle, Track::with_keys(&[(0.0, 100.0, Ease::Linear)])));
        }
        p
    }

    fn angle(p: &Project, bone: &str, frame: f32) -> f32 {
        let e = Engine::new(p.clone()).unwrap();
        let i = p.bones.iter().position(|b| b.name == bone).unwrap();
        e.evaluate(frame).world[i].rotation().to_degrees()
    }

    fn with(c: Clip) -> Project {
        let mut p = proj();
        add_clip(&mut p, c).unwrap();
        bake(&mut p).unwrap();
        p
    }

    fn clip() -> Clip {
        Clip::new("c1", take())
    }

    #[test]
    fn weights_zero_half_one_blend_exactly() {
        for (w, expect) in [(0.0, 100.0), (0.5, 0.5 * 100.0 + 0.5 * 20.0), (1.0, 20.0)] {
            let mut c = clip();
            c.weight = w;
            c.start = 5.0;
            let p = with(c);
            // frame 10 = take frame 5 -> arm_up 20
            assert!((angle(&p, "arm_up", 10.0) - expect).abs() < 1e-3, "w={w}: {}", angle(&p, "arm_up", 10.0));
        }
    }

    #[test]
    fn weight_zero_bakes_nothing() {
        let mut c = clip();
        c.weight = 0.0;
        let p = with(c);
        assert!(p.clip_baked.is_empty());
        assert_eq!(p.channels.len(), 3);
    }

    #[test]
    fn additive_adds_delta_from_first_pose_replace_does_not() {
        let mut c = clip();
        c.mode = ClipMode::Additive;
        let p = with(c);
        // arm_up at frame 5: 100 + (20 - 10)
        assert!((angle(&p, "arm_up", 5.0) - 110.0).abs() < 1e-3);
        assert!((angle(&p, "arm_up", 0.0) - 100.0).abs() < 1e-3, "no delta at the first frame");
        assert!((angle(&p, "arm_up", 40.0) - 100.0).abs() < 1e-3, "neutral after the clip");
        let mut r = clip();
        r.mode = ClipMode::Replace;
        let q = with(r);
        assert!((angle(&q, "arm_up", 5.0) - 20.0).abs() < 1e-3);
        assert!((angle(&q, "arm_up", 40.0) - 100.0).abs() < 1e-3);
    }

    #[test]
    fn multiply_mode_scales_with_weight() {
        let mut t = take();
        t.tracks = vec![TakeTrack { target: "arm_up".into(), prop: Prop::BoneAngle, keys: vec![k(0.0, 0.5), k(10.0, 0.5)] }];
        let mut c = Clip::new("m", t);
        c.mode = ClipMode::Multiply;
        c.weight = 0.5;
        let p = with(c);
        assert!((angle(&p, "arm_up", 5.0) - 100.0 * (1.0 + (0.5 - 1.0) * 0.5)).abs() < 1e-3);
    }

    #[test]
    fn mask_includes_only_listed_targets() {
        let mut c = clip();
        c.include = vec!["arm_*".into()];
        let p = with(c);
        assert!((angle(&p, "arm_up", 5.0) - 20.0).abs() < 1e-3);
        assert!((angle(&p, "leg", 5.0) - 100.0).abs() < 1e-3, "leg not in the mask");
        let mut c = clip();
        c.exclude = vec!["arm_up".into()];
        let p = with(c);
        assert!((angle(&p, "arm_up", 5.0) - 100.0).abs() < 1e-3);
        assert!((angle(&p, "leg", 5.0) - 10.0).abs() < 1e-3);
        assert!(p.clip_baked.iter().all(|ch| ch.target == "leg"));
    }

    #[test]
    fn per_track_weights_upper_body_over_walk() {
        let mut c = clip();
        c.track_weights = vec![TrackWeight { pattern: "arm_*".into(), prop: None, weight: 1.0 }, TrackWeight { pattern: "leg".into(), prop: None, weight: 0.25 }];
        c.weight = 0.5;
        let p = with(c);
        assert!((angle(&p, "arm_up", 5.0) - (0.5 * 100.0 + 0.5 * 20.0)).abs() < 1e-3);
        // leg: w = 0.5*0.25 = 0.125, value 10
        assert!((angle(&p, "leg", 5.0) - (0.875 * 100.0 + 0.125 * 10.0)).abs() < 1e-3);
    }

    #[test]
    fn looping_speed_and_end() {
        let mut c = clip();
        c.loops = 2;
        c.speed = 2.0; // one cycle = 5 frames, total 10
        c.start = 10.0;
        let p = with(c.clone());
        assert!((angle(&p, "arm_up", 10.0) - 10.0).abs() < 1e-3, "start");
        assert!((angle(&p, "arm_up", 12.0) - 18.0).abs() < 1e-3, "speed 2: take frame 4");
        assert!((angle(&p, "arm_up", 15.0) - 30.0).abs() < 1e-3 || (angle(&p, "arm_up", 15.0) - 10.0).abs() < 1e-3, "loop boundary");
        assert!((angle(&p, "arm_up", 16.0) - 14.0).abs() < 1e-3, "second loop, take frame 2");
        assert!((angle(&p, "arm_up", 20.0) - 30.0).abs() < 1e-3, "final pose at the end");
        assert!((angle(&p, "arm_up", 25.0) - 100.0).abs() < 1e-3, "released after the end");
        assert_eq!(c.end(), 20.0);
    }

    #[test]
    fn keyed_weight_and_fades() {
        let mut c = clip();
        c.start = 0.0;
        c.take.length = 40.0;
        c.take.tracks[0].keys = vec![k(0.0, 20.0), k(40.0, 20.0)];
        let mut p = proj();
        add_clip(&mut p, c).unwrap();
        set_weight_keys(&mut p, "c1", &[(0.0, 0.0), (20.0, 1.0)]).unwrap();
        bake(&mut p).unwrap();
        assert!((angle(&p, "arm_up", 0.0) - 100.0).abs() < 1e-3);
        assert!((angle(&p, "arm_up", 10.0) - 60.0).abs() < 1e-3);
        assert!((angle(&p, "arm_up", 30.0) - 20.0).abs() < 1e-3, "held at 1 after the last weight key");
        let mut c = clip();
        c.take.length = 40.0;
        c.take.tracks[0].keys = vec![k(0.0, 20.0), k(40.0, 20.0)];
        c.fade_in = 10.0;
        c.fade_out = 10.0;
        let p = with(c);
        assert!((angle(&p, "arm_up", 0.0) - 100.0).abs() < 1e-3);
        assert!((angle(&p, "arm_up", 5.0) - 60.0).abs() < 1e-3, "smoothstep half at the middle of the fade");
        assert!((angle(&p, "arm_up", 20.0) - 20.0).abs() < 1e-3);
        assert!((angle(&p, "arm_up", 35.0) - 60.0).abs() < 1e-3);
    }

    #[test]
    fn later_clips_layer_over_earlier() {
        let mut a = clip();
        a.name = "a".into();
        let mut t = take();
        t.tracks = vec![TakeTrack { target: "arm_up".into(), prop: Prop::BoneAngle, keys: vec![k(0.0, 50.0), k(10.0, 50.0)] }];
        let mut b = Clip::new("b", t);
        b.weight = 0.5;
        let mut p = proj();
        add_clip(&mut p, a).unwrap();
        add_clip(&mut p, b).unwrap();
        bake(&mut p).unwrap();
        // after a: 20 at frame 5; b halfway to 50
        assert!((angle(&p, "arm_up", 5.0) - 35.0).abs() < 1e-3);
    }

    #[test]
    fn rebake_is_idempotent_and_replaces_only_owned() {
        let mut p = proj();
        add_clip(&mut p, clip()).unwrap();
        bake(&mut p).unwrap();
        let once = p.clone();
        let r = bake(&mut p).unwrap();
        assert_eq!(p, once, "idempotent");
        assert_eq!(r.replaced, once.clip_baked.len());
        // hand-made channel added after the first bake survives; editing the stack replaces the owned ones
        p.channels.push(Channel::keys("arm_low", Prop::BoneScale, Track::with_keys(&[(0.0, 2.0, Ease::Linear)])));
        set_weight(&mut p, "c1", 0.5).unwrap();
        bake(&mut p).unwrap();
        assert_eq!(p.channels.iter().filter(|c| c.prop == Prop::BoneScale).count(), 1);
        assert_eq!(p.channels[..3], once.channels[..3], "hand-made untouched");
        assert!((angle(&p, "arm_up", 5.0) - 60.0).abs() < 1e-3);
        // removing the clip and baking clears the owned channels only
        remove(&mut p, "c1").unwrap();
        bake(&mut p).unwrap();
        assert!(p.clip_baked.is_empty());
        assert_eq!(p.channels.len(), 4);
        // a hand-edited copy of a baked channel is no longer owned
        add_clip(&mut p, clip()).unwrap();
        bake(&mut p).unwrap();
        let n = p.channels.len();
        if let crate::project::Source::Keys { track } = &mut p.channels[n - 1].source {
            track.set_key(59.0, 7.0, Ease::Linear);
        }
        bake(&mut p).unwrap();
        assert_eq!(p.channels.len(), n + 1);
    }

    #[test]
    fn locks_are_respected() {
        let mut p = proj();
        add_clip(&mut p, clip()).unwrap();
        crate::chanmeta::set_meta(&mut p, "leg", Some(Prop::BoneAngle), &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        let before = p.clone();
        assert!(bake(&mut p).unwrap_err().contains("locked"));
        assert_eq!(p, before, "nothing changed");
        p.clips[0].exclude = vec!["leg".into()];
        bake(&mut p).unwrap();
        // locking a target that owns baked channels blocks the next rebake too
        crate::chanmeta::set_meta(&mut p, "arm_up", Some(Prop::BoneAngle), &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        let before = p.clone();
        assert!(bake(&mut p).is_err());
        assert_eq!(p, before);
    }

    #[test]
    fn serde_roundtrip_and_defaults() {
        let mut c = clip();
        c.mode = ClipMode::Additive;
        c.include = vec!["arm_*".into()];
        c.track_weights = vec![TrackWeight { pattern: "arm_up".into(), prop: Some(Prop::BoneAngle), weight: 0.3 }];
        c.weight_keys = Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, 1.0, Ease::Linear)]);
        c.rename = vec![("leg".into(), "arm_low".into())];
        let mut p = proj();
        add_clip(&mut p, c).unwrap();
        bake(&mut p).unwrap();
        let q = Project::from_json(&p.to_json_pretty()).unwrap();
        assert_eq!(q, p);
        let minimal: Clip = serde_json::from_str(r#"{"name":"x","take":{"name":"t","length":4,"tracks":[]}}"#).unwrap();
        assert_eq!((minimal.speed, minimal.loops, minimal.weight, minimal.mode), (1.0, 1, 1.0, ClipMode::Replace));
        let plain = Project::new("e", 1.0, 1.0, 24.0, 2).to_json_pretty();
        assert!(!plain.contains("clips") && !plain.contains("clip_baked"));
    }

    #[test]
    fn rename_applies_before_mask() {
        let mut c = clip();
        c.rename = vec![("leg".into(), "arm_low".into())];
        c.include = vec!["arm_low".into()];
        let p = with(c);
        assert!((angle(&p, "arm_low", 5.0) - 10.0).abs() < 1e-3);
        assert!((angle(&p, "arm_up", 5.0) - 100.0).abs() < 1e-3);
    }

    #[test]
    fn validation_errors() {
        let mut p = proj();
        let lib = Library { takes: vec![take()] };
        assert!(add(&mut p, &lib, "nope", clip()).unwrap_err().contains("no take"));
        let bad = |f: &dyn Fn(&mut Clip)| {
            let mut c = clip();
            f(&mut c);
            c.validate().unwrap_err()
        };
        assert!(bad(&|c| c.speed = -1.0).contains("speed"));
        assert!(bad(&|c| c.speed = 0.0).contains("speed"));
        assert!(bad(&|c| c.speed = f32::NAN).contains("finite"));
        assert!(bad(&|c| c.weight = f32::NAN).contains("finite"));
        assert!(bad(&|c| c.weight = 1.5).contains("0..1"));
        assert!(bad(&|c| c.start = f32::INFINITY).contains("finite"));
        assert!(bad(&|c| c.loops = 0).contains("loops"));
        assert!(bad(&|c| c.fade_in = -1.0).contains("fades"));
        assert!(bad(&|c| c.name = " ".into()).contains("name"));
        assert!(bad(&|c| c.take.tracks[0].keys[0].value = f32::NAN).contains("non-finite"));
        assert!(bad(&|c| c.track_weights = vec![TrackWeight { pattern: "a".into(), prop: None, weight: f32::NAN }]).contains("track weight"));
        add(&mut p, &lib, "wave", clip()).unwrap();
        assert!(add(&mut p, &lib, "wave", clip()).unwrap_err().contains("already exists"));
        assert!(remove(&mut p, "zzz").is_err());
        assert!(set_weight(&mut p, "c1", 2.0).is_err());
        assert!(set_weight(&mut p, "zzz", 0.5).is_err());
        assert_eq!(p.clips[0].weight, 1.0, "failed edit leaves the clip alone");
        assert!(set_weight_keys(&mut p, "c1", &[]).is_err());
        assert!(set_weight_keys(&mut p, "c1", &[(0.0, f32::NAN)]).is_err());
        // unknown target in the take is caught by the bake, project unchanged
        let mut t = take();
        t.tracks[0].target = "ghost".into();
        let mut q = proj();
        add_clip(&mut q, Clip::new("g", t)).unwrap();
        let before = q.clone();
        assert!(bake(&mut q).is_err());
        assert_eq!(q, before);
    }

    #[test]
    fn glob_matching() {
        assert!(glob("arm_*", "arm_up") && glob("*", "") && glob("a?c", "abc") && glob("*_l", "hand_l"));
        assert!(!glob("arm_*", "leg") && !glob("a?c", "ac") && !glob("arm", "arm_up"));
    }

    #[test]
    fn clip_outside_project_is_ignored() {
        let mut c = clip();
        c.start = 500.0;
        let p = with(c);
        assert!(p.clip_baked.is_empty());
    }
}
