// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! A small, original vocabulary of standard rig roles (`arm.upper.l`, `face.mouth`, …) and a
//! name-based tagger that proposes roles for the bones and layers of any project.
//!
//! The vocabulary and the matching rules are our own; they use common English anatomy words
//! only, no product-specific parameter names.

use crate::project::{Layer, Project};

/// Roles OSATURA understands, in head-to-toe order.
pub const ROLES: &[&str] = &[
    "head", "neck", "torso", "hips", "face.eye.l", "face.eye.r", "face.brow.l", "face.brow.r", "face.mouth", "face.nose", "face.jaw",
    "arm.upper.l", "arm.lower.l", "hand.l", "arm.upper.r", "arm.lower.r", "hand.r",
    "leg.upper.l", "leg.lower.l", "foot.l", "leg.upper.r", "leg.lower.r", "foot.r", "tail", "hair",
];

#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    /// `"bone"` or `"layer"`.
    pub kind: &'static str,
    pub name: String,
    pub role: String,
    /// 0..1: how many of the role's words were found.
    pub confidence: f32,
}

fn tokens(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    for ch in name.chars() {
        if ch.is_alphanumeric() {
            if ch.is_uppercase() && prev_lower && !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            prev_lower = ch.is_lowercase() || ch.is_ascii_digit();
            cur.extend(ch.to_lowercase());
        } else {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            prev_lower = false;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn has(t: &[String], words: &[&str]) -> bool {
    t.iter().any(|x| words.contains(&x.as_str()))
}

fn side(t: &[String]) -> Option<&'static str> {
    if has(t, &["l", "left", "lt", "stanga", "stang"]) {
        Some("l")
    } else if has(t, &["r", "right", "rt", "dreapta", "drept"]) {
        Some("r")
    } else {
        None
    }
}

/// Best-guess role for one name, or `None`.
pub fn role_of(name: &str) -> Option<(String, f32)> {
    let t = tokens(name);
    let sd = side(&t);
    let sided = |base: &str, conf: f32| -> Option<(String, f32)> {
        match sd {
            Some(s) => Some((format!("{base}.{s}"), conf)),
            None => Some((format!("{base}.l"), conf * 0.5)), // side unknown: low confidence
        }
    };
    if has(&t, &["eye", "eyes", "ochi"]) && !has(&t, &["brow", "lid"]) {
        return sided("face.eye", 0.9);
    }
    if has(&t, &["brow", "eyebrow", "sprancene"]) {
        return sided("face.brow", 0.9);
    }
    if has(&t, &["mouth", "lips", "lip", "gura"]) {
        return Some(("face.mouth".into(), 0.9));
    }
    if has(&t, &["nose", "nas"]) {
        return Some(("face.nose".into(), 0.9));
    }
    if has(&t, &["jaw"]) {
        return Some(("face.jaw".into(), 0.9));
    }
    if has(&t, &["head", "cap", "skull"]) {
        return Some(("head".into(), 0.9));
    }
    if has(&t, &["neck", "gat"]) {
        return Some(("neck".into(), 0.9));
    }
    if has(&t, &["hand", "wrist", "palm", "mana"]) {
        return sided("hand", 0.9);
    }
    if has(&t, &["foot", "ankle", "feet", "picior"]) && !has(&t, &["thigh", "shin"]) {
        return sided("foot", 0.85);
    }
    if has(&t, &["thigh", "femur"]) || (has(&t, &["leg"]) && has(&t, &["upper", "up"])) {
        return sided("leg.upper", 0.9);
    }
    if has(&t, &["shin", "calf", "tibia"]) || (has(&t, &["leg"]) && has(&t, &["lower", "low"])) {
        return sided("leg.lower", 0.9);
    }
    if (has(&t, &["arm"]) && has(&t, &["upper", "up"])) || has(&t, &["bicep", "humerus"]) {
        return sided("arm.upper", 0.9);
    }
    if has(&t, &["forearm", "elbow", "radius"]) || (has(&t, &["arm"]) && has(&t, &["lower", "low", "fore"])) {
        return sided("arm.lower", 0.9);
    }
    if has(&t, &["hips", "hip", "pelvis"]) {
        return Some(("hips".into(), 0.9));
    }
    if has(&t, &["torso", "chest", "spine", "body", "trunk", "back"]) {
        return Some(("torso".into(), 0.8));
    }
    if has(&t, &["tail"]) {
        return Some(("tail".into(), 0.9));
    }
    if has(&t, &["hair", "fringe", "bang", "bangs"]) {
        return Some(("hair".into(), 0.9));
    }
    None
}

fn walk(ls: &[Layer], prefix: &str, out: &mut Vec<Tag>) {
    for l in ls {
        let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
        if let Some((role, confidence)) = role_of(&l.name) {
            out.push(Tag { kind: "layer", name: path.clone(), role, confidence });
        }
        walk(l.children(), &path, out);
    }
}

/// `"l"` / `"r"` when the name carries a side word.
pub fn side_of(name: &str) -> Option<&'static str> {
    side(&tokens(name))
}

/// Names that describe an iris or pupil (the part that should be clipped by the eye shape).
pub fn is_iris(name: &str) -> bool {
    has(&tokens(name), &["iris", "pupil", "pupila", "irisul"])
}

/// Propose roles for every bone and layer whose name looks anatomical.
pub fn tag_rig(p: &Project) -> Vec<Tag> {
    let mut out = Vec::new();
    for b in &p.bones {
        if let Some((role, confidence)) = role_of(&b.name) {
            out.push(Tag { kind: "bone", name: b.name.clone(), role, confidence });
        }
    }
    walk(&p.layers, "", &mut out);
    out
}

/// Roles from [`ROLES`] that no *bone* carries (useful to see what a humanoid rig lacks).
pub fn missing_bone_roles(p: &Project) -> Vec<&'static str> {
    let have: Vec<String> = tag_rig(p).into_iter().filter(|t| t.kind == "bone").map(|t| t.role).collect();
    ROLES.iter().copied().filter(|r| !have.iter().any(|h| h == r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    #[test]
    fn tokeniser_splits_cases_and_separators() {
        assert_eq!(tokens("armUpper_L"), vec!["arm", "upper", "l"]);
        assert_eq!(tokens("Left-Eye.02"), vec!["left", "eye", "02"]);
        assert!(tokens("___").is_empty());
    }

    #[test]
    fn roles_by_name() {
        let r = |n: &str| role_of(n).map(|x| x.0);
        assert_eq!(r("arm_up_L").as_deref(), Some("arm.upper.l"));
        assert_eq!(r("forearmR").as_deref(), Some("arm.lower.r"));
        assert_eq!(r("Left Eye").as_deref(), Some("face.eye.l"));
        assert_eq!(r("brow_right").as_deref(), Some("face.brow.r"));
        assert_eq!(r("shin_r").as_deref(), Some("leg.lower.r"));
        assert_eq!(r("thigh").as_deref(), Some("leg.upper.l"));
        assert_eq!(r("mouth").as_deref(), Some("face.mouth"));
        assert_eq!(r("sunset"), None);
        assert_eq!(r("gura"), Some("face.mouth".to_string()), "Romanian words work too");
        let (_, conf_unsided) = role_of("thigh").unwrap();
        let (_, conf_sided) = role_of("thigh_L").unwrap();
        assert!(conf_sided > conf_unsided);
    }

    #[test]
    fn tags_the_demo_rig() {
        let p = demo::project();
        let tags = tag_rig(&p);
        assert!(tags.iter().any(|t| t.kind == "bone" && t.role.starts_with("arm.")));
        assert!(tags.iter().any(|t| t.kind == "layer" && t.role == "face.mouth"), "{tags:?}");
        let missing = missing_bone_roles(&p);
        assert!(missing.contains(&"face.eye.l") && !missing.contains(&"hand.l"), "{missing:?}");
    }
}
