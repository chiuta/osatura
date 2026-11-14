// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Motion library: record a time range of keyed channels as a named take, store takes in a
//! standalone JSON library, and apply them to any project (renaming targets, scaling speed).

use crate::anim::{Key, Track};
use crate::keyops::{copy_keys, paste_keys};
use crate::project::{Channel, Mode, Project, Prop, Source};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TakeTrack {
    pub target: String,
    pub prop: Prop,
    /// Frames relative to the start of the take.
    pub keys: Vec<Key>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Take {
    pub name: String,
    pub length: f32,
    pub tracks: Vec<TakeTrack>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Library {
    #[serde(default)]
    pub takes: Vec<Take>,
}

impl Library {
    pub fn from_json(s: &str) -> Result<Library, String> {
        serde_json::from_str(s).map_err(|e| format!("bad library: {e}"))
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("library serialises")
    }
    pub fn find(&self, name: &str) -> Option<&Take> {
        self.takes.iter().find(|t| t.name == name)
    }
}

/// Record keys of replace-mode keyed channels within `[from, to]` (optionally only for targets
/// starting with one of `only`). Same name overwrites.
pub fn record(lib: &mut Library, p: &Project, name: &str, from: f32, to: f32, only: &[String]) -> Result<usize, String> {
    if name.trim().is_empty() || (to.partial_cmp(&from) != Some(std::cmp::Ordering::Greater)) || !from.is_finite() || !to.is_finite() {
        return Err("take needs a name and to > from".into());
    }
    let mut tracks = Vec::new();
    for c in &p.channels {
        let Source::Keys { track } = &c.source else { continue };
        if c.mode == Mode::Replace || c.mode == Mode::Add {
            if !only.is_empty() && !only.iter().any(|o| c.target.starts_with(o.as_str())) {
                continue;
            }
            let keys = copy_keys(track, from, to);
            if !keys.is_empty() {
                tracks.push(TakeTrack { target: c.target.clone(), prop: c.prop, keys });
            }
        }
    }
    if tracks.is_empty() {
        return Err("no keys found in that range".into());
    }
    let n = tracks.len();
    let take = Take { name: name.into(), length: to - from, tracks };
    match lib.takes.iter_mut().find(|t| t.name == name) {
        Some(t) => *t = take,
        None => lib.takes.push(take),
    }
    Ok(n)
}

/// Paste a take at `at`. `rename` maps take targets to project targets; `speed` > 1 plays faster.
/// Returns the number of tracks applied.
pub fn apply(p: &mut Project, take: &Take, at: f32, rename: &[(String, String)], speed: f32) -> Result<usize, String> {
    if (speed.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !speed.is_finite() || !at.is_finite() {
        return Err("speed must be positive and at finite".into());
    }
    for t in &take.tracks {
        let target = rename.iter().find(|(f, _)| *f == t.target).map_or(t.target.as_str(), |(_, to)| to.as_str());
        crate::chanmeta::assert_unlocked(p, target, t.prop)?;
        let clip: Vec<Key> = t.keys.iter().map(|k| Key { frame: k.frame / speed, ..*k }).collect();
        let existing = p.channels.iter_mut().find(|c| c.target == target && c.prop == t.prop && c.mode == Mode::Replace && matches!(c.source, Source::Keys { .. }));
        match existing {
            Some(c) => {
                if let Source::Keys { track } = &mut c.source {
                    paste_keys(track, &clip, at);
                }
            }
            None => {
                let mut track = Track::new();
                paste_keys(&mut track, &clip, at);
                p.channels.push(Channel::keys(target, t.prop, track));
            }
        }
    }
    Ok(take.tracks.len())
}

fn smooth01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Like [`apply`], but cross-fades with what is already animated: over the first and last
/// `blend` frames of the placed take the result moves smoothly (smoothstep) between the existing
/// curve and the take. Tracks with no existing channel are pasted unchanged. The span is
/// resampled once per frame, so it becomes dense linear keys. `blend` 0 behaves like `apply`.
pub fn apply_blended(p: &mut Project, take: &Take, at: f32, rename: &[(String, String)], speed: f32, blend: f32) -> Result<usize, String> {
    if !matches!(blend.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) || !blend.is_finite() {
        return Err("blend must be >= 0".into());
    }
    if blend == 0.0 {
        return apply(p, take, at, rename, speed);
    }
    if (speed.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !speed.is_finite() || !at.is_finite() {
        return Err("speed must be positive and at finite".into());
    }
    let end = at + take.length / speed;
    for t in &take.tracks {
        let target = rename.iter().find(|(f, _)| *f == t.target).map_or(t.target.as_str(), |(_, to)| to.as_str()).to_string();
        crate::chanmeta::assert_unlocked(p, &target, t.prop)?;
        let clip: Vec<Key> = t.keys.iter().map(|k| Key { frame: k.frame / speed + at, ..*k }).collect();
        let tc = Track::with_keys(&clip.iter().map(|k| (k.frame, k.value, k.ease)).collect::<Vec<_>>());
        let existing = p.channels.iter_mut().find(|c| c.target == target && c.prop == t.prop && c.mode == Mode::Replace && matches!(c.source, Source::Keys { .. }));
        let Some(c) = existing else {
            let mut track = Track::new();
            for k in &clip {
                track.set_key(k.frame, k.value, k.ease);
            }
            p.channels.push(Channel::keys(&target, t.prop, track));
            continue;
        };
        let Source::Keys { track } = &mut c.source else { continue };
        if track.keys.is_empty() {
            for k in &clip {
                track.set_key(k.frame, k.value, k.ease);
            }
            continue;
        }
        let old = track.clone();
        track.keys.retain(|k| k.frame < at || k.frame > end);
        let mut f = at;
        loop {
            let fr = f.min(end);
            let w = smooth01((fr - at) / blend) * smooth01((end - fr) / blend);
            let tv = tc.eval(fr).unwrap_or(0.0);
            let ev = old.eval(fr).unwrap_or(tv);
            track.set_key(fr, ev * (1.0 - w) + tv * w, crate::anim::Ease::Linear);
            if fr >= end {
                break;
            }
            f += 1.0;
        }
    }
    Ok(take.tracks.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::Ease;
    use crate::math::Vec2;
    use crate::project::Bone;
    use crate::Engine;

    fn proj() -> Project {
        let mut p = Project::new("m", 100.0, 100.0, 24.0, 60);
        p.bones = vec![Bone::new("a", None, 10.0, Vec2::new(10.0, 10.0), 0.0), Bone::new("b", None, 10.0, Vec2::new(50.0, 10.0), 0.0)];
        p.channels = vec![Channel::keys("a", Prop::BoneAngle, Track::with_keys(&[(10.0, 0.0, Ease::Linear), (20.0, 40.0, Ease::Linear), (30.0, 0.0, Ease::Linear), (50.0, 99.0, Ease::Linear)]))];
        p
    }

    #[test]
    fn record_apply_roundtrip_with_rename_and_speed() {
        let p = proj();
        let mut lib = Library::default();
        assert_eq!(record(&mut lib, &p, "wave", 10.0, 30.0, &[]).unwrap(), 1);
        let take = lib.find("wave").unwrap().clone();
        assert_eq!(take.tracks[0].keys.len(), 3, "key at 50 is outside the range");
        let lib2 = Library::from_json(&lib.to_json()).unwrap();
        assert_eq!(lib2, lib);

        let mut q = proj();
        q.channels.clear();
        apply(&mut q, &take, 5.0, &[("a".into(), "b".into())], 2.0).unwrap();
        Engine::new(q.clone()).unwrap();
        let Source::Keys { track } = &q.channels[0].source else { panic!() };
        assert_eq!(q.channels[0].target, "b");
        assert!((track.eval(10.0).unwrap() - 40.0).abs() < 1e-3, "peak at 5 + 10/2");
        assert!((track.eval(15.0).unwrap()).abs() < 1e-3, "ends at 5 + 20/2");
    }

    #[test]
    fn locked_channels_refuse_takes() {
        let mut p = proj();
        let mut lib = Library::default();
        record(&mut lib, &p, "w", 10.0, 30.0, &[]).unwrap();
        crate::chanmeta::set_meta(&mut p, "a", Some(Prop::BoneAngle), &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        let before = p.channels.clone();
        assert!(apply(&mut p, &lib.takes[0].clone(), 40.0, &[], 1.0).unwrap_err().contains("locked"));
        assert_eq!(p.channels, before);
    }

    #[test]
    fn filters_and_errors() {
        let p = proj();
        let mut lib = Library::default();
        assert!(record(&mut lib, &p, "x", 10.0, 30.0, &["zzz".into()]).is_err());
        assert!(record(&mut lib, &p, "x", 30.0, 10.0, &[]).is_err());
        assert!(record(&mut lib, &p, "x", 100.0, 200.0, &[]).is_err());
        record(&mut lib, &p, "w", 10.0, 30.0, &[]).unwrap();
        record(&mut lib, &p, "w", 10.0, 30.0, &["a".into()]).unwrap();
        assert_eq!(lib.takes.len(), 1, "same name overwrites");
        let mut q = proj();
        assert!(apply(&mut q, &lib.takes[0], 0.0, &[], 0.0).is_err());
    }

    #[test]
    fn pasting_into_existing_channel_merges() {
        let mut p = proj();
        let mut lib = Library::default();
        record(&mut lib, &p, "w", 10.0, 30.0, &[]).unwrap();
        let before = p.channels.len();
        apply(&mut p, &lib.takes[0].clone(), 100.0, &[], 1.0).unwrap();
        assert_eq!(p.channels.len(), before);
        let Source::Keys { track } = &p.channels[0].source else { panic!() };
        assert!((track.eval(110.0).unwrap() - 40.0).abs() < 1e-3);
    }

    #[test]
    fn blended_take_fades_in_and_out() {
        let mut lib = Library::default();
        let p = proj();
        record(&mut lib, &p, "wave", 10.0, 30.0, &[]).unwrap();
        let take = lib.find("wave").unwrap().clone();
        let mut q = proj();
        q.channels = vec![Channel::keys("a", Prop::BoneAngle, Track::with_keys(&[(0.0, 10.0, Ease::Linear), (59.0, 10.0, Ease::Linear)]))];
        apply_blended(&mut q, &take, 20.0, &[], 1.0, 5.0).unwrap();
        let Source::Keys { track } = &q.channels[0].source else { panic!() };
        assert!((track.eval(20.0).unwrap() - 10.0).abs() < 1e-3, "starts at existing value");
        assert!((track.eval(30.0).unwrap() - 40.0).abs() < 1e-3, "full take in the middle");
        assert!((track.eval(40.0).unwrap() - 10.0).abs() < 1e-3, "back to existing at the end");
        assert!((track.eval(5.0).unwrap() - 10.0).abs() < 1e-3, "outside untouched");
        let v = track.eval(22.0).unwrap();
        assert!(v < 10.0 && v < track.eval(26.0).unwrap(), "ramps toward the take");
        Engine::new(q).unwrap();
    }
}
