// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Channel metadata for the editor: colour tag, shy (hide in lists), solo (preview only these),
//! lock (refuse edits). Stored beside the channels; evaluation ignores it except through
//! [`solo_preview`], which builds a project containing only the soloed channels.

use crate::project::{Project, Prop};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChannelMeta {
    pub target: String,
    pub prop: Option<Prop>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default)]
    pub shy: bool,
    #[serde(default)]
    pub solo: bool,
    #[serde(default)]
    pub lock: bool,
}

/// What to change; `None` leaves a flag as it is.
#[derive(Default, Clone, Debug)]
pub struct Change {
    pub color: Option<String>,
    pub shy: Option<bool>,
    pub solo: Option<bool>,
    pub lock: Option<bool>,
}

/// `prop = None` addresses every channel of the target.
pub fn set_meta(p: &mut Project, target: &str, prop: Option<Prop>, c: &Change) -> Result<(), String> {
    if !p.channels.iter().any(|ch| ch.target == target && prop.is_none_or(|pr| ch.prop == pr)) {
        return Err(format!("no channel matches '{target}'"));
    }
    if let Some(col) = &c.color {
        if crate::style::Rgba::parse(col).is_none() {
            return Err(format!("bad colour '{col}'"));
        }
    }
    let i = match p.channel_meta.iter().position(|m| m.target == target && m.prop == prop) {
        Some(i) => i,
        None => {
            p.channel_meta.push(ChannelMeta { target: target.into(), prop, ..Default::default() });
            p.channel_meta.len() - 1
        }
    };
    let m = &mut p.channel_meta[i];
    if let Some(col) = &c.color {
        m.color = Some(col.clone());
    }
    if let Some(v) = c.shy { m.shy = v; }
    if let Some(v) = c.solo { m.solo = v; }
    if let Some(v) = c.lock { m.lock = v; }
    if m.color.is_none() && !m.shy && !m.solo && !m.lock {
        p.channel_meta.remove(i);
    }
    Ok(())
}

fn matches(m: &ChannelMeta, target: &str, prop: Prop) -> bool {
    m.target == target && m.prop.is_none_or(|p| p == prop)
}

/// True when the channel belongs to a layer (or a path/deformer of one) that has its `locked` flag.
pub fn layer_locked(p: &Project, target: &str, prop: Prop) -> bool {
    use crate::project::TargetKind;
    match prop.target_kind() {
        TargetKind::Layer => p.is_layer_locked(target),
        TargetKind::Sub => p.is_layer_locked(target.split("::").next().unwrap_or(target)),
        _ => false,
    }
}

/// Locked by the channel's own flag or because its layer is locked.
pub fn is_locked(p: &Project, target: &str, prop: Prop) -> bool {
    p.channel_meta.iter().any(|m| m.lock && matches(m, target, prop)) || layer_locked(p, target, prop)
}

/// Error if the channel is locked (call before editing keys).
pub fn assert_unlocked(p: &Project, target: &str, prop: Prop) -> Result<(), String> {
    if layer_locked(p, target, prop) {
        Err(format!("layer '{}' is locked (channel {target}.{prop:?})", target.split("::").next().unwrap_or(target)))
    } else if is_locked(p, target, prop) {
        Err(format!("channel {target}.{prop:?} is locked"))
    } else {
        Ok(())
    }
}

/// Split channels into those that may be added and the names of locked ones that were dropped.
pub fn split_unlocked(p: &Project, chans: Vec<crate::project::Channel>) -> (Vec<crate::project::Channel>, Vec<String>) {
    let (mut ok, mut blocked) = (Vec::new(), Vec::new());
    for c in chans {
        if is_locked(p, &c.target, c.prop) {
            blocked.push(format!("{}.{:?}", c.target, c.prop));
        } else {
            ok.push(c);
        }
    }
    (ok, blocked)
}

/// Copy of the project where only soloed channels remain (unchanged if nothing is soloed).
pub fn solo_preview(p: &Project) -> Project {
    let mut q = p.clone();
    if p.channel_meta.iter().any(|m| m.solo) {
        q.channels.retain(|c| p.channel_meta.iter().any(|m| m.solo && matches(m, &c.target, c.prop)));
    }
    q
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::{Ease, Track};
    use crate::project::Channel;

    fn proj() -> Project {
        let mut p = Project::new("c", 10.0, 10.0, 24.0, 5);
        p.bones = vec![crate::project::Bone::new("a", None, 5.0, crate::math::Vec2::ZERO, 0.0), crate::project::Bone::new("b", None, 5.0, crate::math::Vec2::ZERO, 0.0)];
        let k = |t: &str, pr: Prop| Channel::keys(t, pr, Track::with_keys(&[(0.0, 1.0, Ease::Linear)]));
        p.channels = vec![k("a", Prop::BoneAngle), k("a", Prop::BoneScale), k("b", Prop::BoneAngle)];
        p
    }

    #[test]
    fn flags_lock_and_cleanup() {
        let mut p = proj();
        set_meta(&mut p, "a", Some(Prop::BoneAngle), &Change { lock: Some(true), color: Some("#ff8800".into()), ..Default::default() }).unwrap();
        assert!(is_locked(&p, "a", Prop::BoneAngle) && !is_locked(&p, "a", Prop::BoneScale));
        assert!(assert_unlocked(&p, "a", Prop::BoneAngle).is_err());
        set_meta(&mut p, "a", None, &Change { lock: Some(true), ..Default::default() }).unwrap();
        assert!(is_locked(&p, "a", Prop::BoneScale), "target-wide lock");
        set_meta(&mut p, "a", Some(Prop::BoneAngle), &Change { lock: Some(false), color: Some("#000000".into()), ..Default::default() }).unwrap();
        let j = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Project>(&j).unwrap().channel_meta, p.channel_meta);
        let mut q = proj();
        set_meta(&mut q, "b", None, &Change { shy: Some(true), ..Default::default() }).unwrap();
        set_meta(&mut q, "b", None, &Change { shy: Some(false), ..Default::default() }).unwrap();
        assert!(q.channel_meta.is_empty(), "all-default entries are removed");
    }

    #[test]
    fn bake_tools_respect_locks() {
        use crate::{footlock, magnet, physics};
        let mut p = proj();
        p.bones[0].parent = None;
        set_meta(&mut p, "a", Some(Prop::BoneOffsetX), &Change { lock: Some(true), ..Default::default() }).unwrap_err(); // no such channel yet
        p.channels.push(Channel::keys("a", Prop::BoneOffsetX, Track::with_keys(&[(0.0, 0.0, Ease::Linear)])));
        set_meta(&mut p, "a", Some(Prop::BoneOffsetX), &Change { lock: Some(true), ..Default::default() }).unwrap();
        let o = footlock::LockOptions::default();
        assert!(footlock::lock_feet(&mut p, "a", &o).unwrap_err().contains("locked"));
        assert!(magnet::bake_magnet(&mut p, "a", &magnet::Target::Point(crate::math::Vec2::ZERO), &magnet::Magnet { radius: 5.0, strength: 1.0 }).unwrap_err().contains("locked"));
        set_meta(&mut p, "b", Some(Prop::BoneAngle), &Change { lock: Some(true), ..Default::default() }).unwrap();
        let sim = vec![Channel::keys("b", Prop::BoneAngle, Track::with_keys(&[(0.0, 1.0, Ease::Linear)])), Channel::keys("a", Prop::BoneScale, Track::with_keys(&[(0.0, 1.0, Ease::Linear)]))];
        let n = p.channels.len();
        let (added, skipped) = physics::merge_channels(&mut p, sim, true);
        assert_eq!((added, skipped), (1, vec!["b".to_string()]));
        assert_eq!(p.channels.len(), n, "locked b.angle kept, a.scale replaced");
    }

    #[test]
    fn solo_keeps_only_soloed_and_errors_are_clean() {
        let mut p = proj();
        assert_eq!(solo_preview(&p).channels.len(), 3, "no solo: unchanged");
        set_meta(&mut p, "b", None, &Change { solo: Some(true), ..Default::default() }).unwrap();
        let s = solo_preview(&p);
        assert_eq!(s.channels.len(), 1);
        assert_eq!(s.channels[0].target, "b");
        assert_eq!(p.channels.len(), 3, "original untouched");
        assert!(set_meta(&mut p, "ghost", None, &Change::default()).is_err());
        assert!(set_meta(&mut p, "a", None, &Change { color: Some("nope".into()), ..Default::default() }).is_err());
    }
}
