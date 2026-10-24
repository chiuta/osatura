// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Names: normalisation and role classification (part of `mocap`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Names: normalisation and role classification

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Hips,
    Spine,
    Neck,
    Head,
    Clavicle,
    UpperArm,
    ForeArm,
    Hand,
    Thigh,
    Shin,
    Foot,
    Toe,
    Finger,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tag {
    pub role: Role,
    pub side: Option<Side>,
    pub conf: f32,
}

pub(super) const NOISE: &[&str] = &["bip", "biped", "mixamorig", "def", "org", "mch", "jnt", "joint", "bone", "bn", "sk", "skel", "skeleton", "rig", "armature", "cc", "base", "char", "character"];
pub(super) const LEFT: &[&str] = &["left", "l", "lt", "lft"];
pub(super) const RIGHT: &[&str] = &["right", "r", "rt", "rgt"];
pub(super) const UPW: &[&str] = &["up", "upper", "upr"];
pub(super) const LOWW: &[&str] = &["low", "lower", "lwr", "bottom"];
pub(super) const THIGH_W: &[&str] = &["thigh", "femur", "upleg", "upperleg", "uplegs", "upperlegs"];
pub(super) const SHIN_W: &[&str] = &["shin", "calf", "tibia", "lowleg", "lowerleg", "lowerlegs"];
pub(super) const UPARM_W: &[&str] = &["upperarm", "uparm", "humerus"];
pub(super) const FORE_W: &[&str] = &["forearm", "lowarm", "lowerarm", "radius"];
pub(super) const HAND_W: &[&str] = &["hand", "hands", "wrist", "palm"];
pub(super) const CLAV_W: &[&str] = &["shoulder", "clavicle", "collar", "scapula", "clav"];
pub(super) const FOOT_W: &[&str] = &["foot", "feet", "ankle"];
pub(super) const TOE_W: &[&str] = &["toe", "toes", "toebase", "ball"];
pub(super) const HIPS_W: &[&str] = &["hips", "hip", "pelvis", "pelvic", "root", "cog"];
pub(super) const SPINE_W: &[&str] = &["spine", "chest", "abdomen", "torso", "back", "waist", "stomach", "thorax", "trunk", "belly", "lowerback", "upperback", "ribcage", "sternum"];
pub(super) const NECK_W: &[&str] = &["neck", "lowerneck", "upperneck"];
pub(super) const HEAD_W: &[&str] = &["head", "skull", "cranium"];
pub(super) const FINGER_W: &[&str] = &["thumb", "index", "middle", "ring", "pinky", "pinkie", "little", "finger", "fingers", "digit"];
pub(super) const OTHER_W: &[&str] = &[
    "jaw", "eye", "eyes", "eyelid", "brow", "ear", "nose", "tongue", "teeth", "hair", "cheek", "lip", "lips", "mouth", "face", "tail", "wing", "cape", "skirt", "cloth", "breast", "twist", "roll",
    "corrective", "helper", "end", "site", "nub", "tip", "null", "dummy", "prop", "weapon", "gun", "sword", "ik", "pole", "target", "ctrl", "control",
];

pub(super) fn in_vocab(t: &str) -> bool {
    [THIGH_W, SHIN_W, UPARM_W, FORE_W, HAND_W, CLAV_W, FOOT_W, TOE_W, HIPS_W, SPINE_W, NECK_W, HEAD_W, FINGER_W, OTHER_W, UPW, LOWW, &["arm", "arms", "leg", "legs"]].iter().any(|l| l.contains(&t))
}

/// Split a bone name into lower-case tokens: namespaces (`mixamorig:`, `Armature|`) dropped,
/// camelCase, digits and separators split, glued side prefixes (`lthigh`, `lefthand`) separated
/// and rig-prefix noise (`Bip01`, `DEF-`) removed. Numeric tokens are kept (as digit strings).
pub fn normalize_name(name: &str) -> Vec<String> {
    let base = name.rsplit([':', '|']).next().unwrap_or(name);
    let chars: Vec<char> = base.chars().collect();
    let mut raw: Vec<String> = Vec::new();
    let mut cur = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_alphanumeric() {
            if !cur.is_empty() {
                raw.push(std::mem::take(&mut cur));
            }
            continue;
        }
        if let Some(&pc) = cur.chars().last().as_ref() {
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            let boundary = (pc.is_lowercase() && c.is_uppercase())
                || (pc.is_alphabetic() != c.is_alphabetic())
                || (pc.is_uppercase() && c.is_uppercase() && next_lower);
            if boundary {
                raw.push(std::mem::take(&mut cur));
            }
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        raw.push(cur);
    }
    let mut out: Vec<String> = Vec::new();
    let mut prev_noise = false;
    for t in raw.into_iter().map(|t| t.to_lowercase()) {
        let numeric = t.chars().all(|c| c.is_ascii_digit());
        if numeric {
            if prev_noise {
                continue; // Bip01, mixamorig1
            }
            out.push(t);
            continue;
        }
        if NOISE.contains(&t.as_str()) {
            prev_noise = true;
            continue;
        }
        prev_noise = false;
        // glued side prefix: lthigh, lefthand
        if !in_vocab(&t) && !LEFT.contains(&t.as_str()) && !RIGHT.contains(&t.as_str()) {
            let mut done = false;
            for p in ["left", "right", "l", "r"] {
                if let Some(rest) = t.strip_prefix(p) {
                    if rest.len() >= 2 && in_vocab(rest) {
                        out.push(p.to_string());
                        out.push(rest.to_string());
                        done = true;
                        break;
                    }
                }
            }
            if done {
                continue;
            }
        }
        out.push(t);
    }
    out
}

pub(super) fn any_of(ws: &[String], list: &[&str]) -> bool {
    ws.iter().any(|w| list.contains(&w.as_str()))
}
pub(super) fn has(ws: &[String], w: &str) -> bool {
    ws.iter().any(|x| x == w)
}

/// Guess the anatomical role of a bone from its name alone.
pub fn classify(name: &str) -> Tag {
    let toks = normalize_name(name);
    let mut side = None;
    let mut index: Option<u32> = None;
    let mut ws: Vec<String> = Vec::new();
    let non_side = toks.iter().filter(|t| !LEFT.contains(&t.as_str()) && !RIGHT.contains(&t.as_str())).count();
    for t in &toks {
        if t.chars().all(|c| c.is_ascii_digit()) {
            index = t.parse().ok();
        } else if LEFT.contains(&t.as_str()) && non_side > 0 {
            side = Some(Side::Left);
        } else if RIGHT.contains(&t.as_str()) && non_side > 0 {
            side = Some(Side::Right);
        } else {
            ws.push(t.clone());
        }
    }
    let _ = index;
    let t = |role, conf| Tag { role, side, conf };
    let other = Tag { role: Role::Other, side, conf: 0.0 };
    if ws.is_empty() || any_of(&ws, OTHER_W) {
        return other;
    }
    if any_of(&ws, FINGER_W) {
        return t(Role::Finger, 0.9);
    }
    if any_of(&ws, TOE_W) {
        return t(Role::Toe, if has(&ws, "ball") { 0.7 } else { 0.95 });
    }
    if any_of(&ws, FOOT_W) {
        return t(Role::Foot, if has(&ws, "ankle") { 0.85 } else { 0.95 });
    }
    if any_of(&ws, THIGH_W) {
        return t(Role::Thigh, if has(&ws, "femur") { 0.95 } else { 1.0 });
    }
    if any_of(&ws, SHIN_W) {
        return t(Role::Shin, if has(&ws, "tibia") { 0.95 } else { 1.0 });
    }
    if has(&ws, "leg") || has(&ws, "legs") {
        if any_of(&ws, UPW) {
            return t(Role::Thigh, 0.95);
        }
        if any_of(&ws, LOWW) {
            return t(Role::Shin, 0.95);
        }
        return t(Role::Shin, 0.55);
    }
    if any_of(&ws, UPARM_W) {
        return t(Role::UpperArm, 1.0);
    }
    if any_of(&ws, FORE_W) || (has(&ws, "fore") && (has(&ws, "arm") || has(&ws, "arms"))) {
        return t(Role::ForeArm, 1.0);
    }
    if has(&ws, "arm") || has(&ws, "arms") {
        if any_of(&ws, UPW) {
            return t(Role::UpperArm, 0.95);
        }
        if any_of(&ws, LOWW) {
            return t(Role::ForeArm, 0.95);
        }
        return t(Role::UpperArm, 0.6);
    }
    if any_of(&ws, HAND_W) {
        return t(Role::Hand, if has(&ws, "palm") { 0.6 } else { 0.95 });
    }
    if any_of(&ws, CLAV_W) {
        return t(Role::Clavicle, 0.85);
    }
    if any_of(&ws, NECK_W) {
        return t(Role::Neck, 1.0);
    }
    if any_of(&ws, HEAD_W) {
        return t(Role::Head, 1.0);
    }
    if has(&ws, "hips") {
        return t(Role::Hips, 1.0);
    }
    if has(&ws, "pelvis") || has(&ws, "pelvic") {
        return t(Role::Hips, 0.95);
    }
    if has(&ws, "hip") {
        return if side.is_some() { t(Role::Thigh, 0.5) } else { t(Role::Hips, 0.9) };
    }
    if has(&ws, "root") {
        return t(Role::Hips, 0.55);
    }
    if has(&ws, "cog") {
        return t(Role::Hips, 0.5);
    }
    if any_of(&ws, SPINE_W) {
        return t(Role::Spine, if has(&ws, "spine") { 1.0 } else { 0.85 });
    }
    other
}
