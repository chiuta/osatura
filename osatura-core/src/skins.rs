// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Skin sets: swap whole groups of layers together (outfits, expressions, damage states).
//!
//! A skin belongs to a `group` ("outfit"); applying one shows its layers and hides the layers of
//! every other skin in the same group. [`schedule`] animates the choice with step keys on layer
//! opacity (the project has no animatable "visible" flag), so use it on layers whose opacity
//! you do not otherwise animate.
//!
//! # Per-skin bones
//! A skin may also list `bones` that take part only while it is active (a cape or tail chain used
//! by one outfit). Bones cannot be removed from the evaluated rig without breaking indices, so an
//! *inactive* skin bone is **collapsed**: a recognisable multiply channel on `bone_scale` (step
//! keys 0 / 1, see [`is_collapse_channel`]) makes its scale 0, and its children inherit that, so
//! everything riding or weighted to it (skinned layers, `Layer.bone` followers) shrinks to a point.
//! `apply` writes a constant 0 channel for inactive bones and removes it for active ones;
//! `schedule` step-keys it, so the switch is exact at the given frames. Rules enforced by
//! [`validate`]: a bone belongs to at most one skin; every descendant of a skin bone is listed in
//! the same skin; skin bones have no `scale_limits` (a clamp would stop the collapse); constraints
//! and IK chains that touch a skin bone must be in that skin's rig list and in no other skin of
//! the group (so they switch off with the bone). The bone overlay hides collapsed bones too
//! ([`bone_collapsed`], used by `bonevis::visible`). Springs on a skin bone keep running unseen.
//! The channel is appended last because channels run in list order: a later `replace` scale
//! channel would undo the collapse. Layers weighted to a skin bone should belong to that skin
//! (or they vanish with the bone while the skin is off).

use crate::anim::{Ease, Track};
use crate::project::{Channel, ConstraintKind, Mode, Project, Prop, Source};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Skin {
    pub name: String,
    pub group: String,
    /// `/`-joined layer paths.
    pub layers: Vec<String>,
    /// Constraints that are only active (weight 1) while this skin is; 0 when another skin of the
    /// group is active.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
    /// IK chains that are only active (mix 1) while this skin is; 0 otherwise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub iks: Vec<String>,
    /// Bones that take part only while this skin is active (collapsed to scale 0 otherwise).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bones: Vec<String>,
}

pub fn add_skin(p: &mut Project, group: &str, name: &str, layers: &[String]) -> Result<(), String> {
    if group.trim().is_empty() || name.trim().is_empty() {
        return Err("skin needs a group and a name".into());
    }
    if layers.is_empty() {
        return Err("a skin needs at least one layer".into());
    }
    for l in layers {
        if p.find_layer(l).is_none() {
            return Err(format!("no layer '{l}'"));
        }
    }
    let (constraints, iks, bones) = p
        .skins
        .iter()
        .find(|x| x.group == group && x.name == name)
        .map(|x| (x.constraints.clone(), x.iks.clone(), x.bones.clone()))
        .unwrap_or_default();
    let s = Skin {
        name: name.into(),
        group: group.into(),
        layers: layers.to_vec(),
        constraints,
        iks,
        bones,
    };
    match p
        .skins
        .iter_mut()
        .find(|x| x.group == group && x.name == name)
    {
        Some(x) => *x = s,
        None => p.skins.push(s),
    }
    Ok(())
}

fn group_layers(p: &Project, group: &str) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    for s in p.skins.iter().filter(|s| s.group == group) {
        for l in &s.layers {
            if !all.contains(l) {
                all.push(l.clone());
            }
        }
    }
    all
}

/// Attach rig parts to a skin: the named constraints and IK chains are on only while it is active.
pub fn set_rig(
    p: &mut Project,
    group: &str,
    name: &str,
    constraints: &[String],
    iks: &[String],
) -> Result<(), String> {
    for c in constraints {
        if !p.constraints.iter().any(|x| &x.name == c) {
            return Err(format!("no constraint '{c}'"));
        }
    }
    for i in iks {
        if !p.ik.iter().any(|x| &x.name == i) {
            return Err(format!("no IK chain '{i}'"));
        }
    }
    let s = p
        .skins
        .iter_mut()
        .find(|s| s.group == group && s.name == name)
        .ok_or_else(|| format!("no skin '{name}' in group '{group}'"))?;
    let old = (
        std::mem::replace(&mut s.constraints, constraints.to_vec()),
        std::mem::replace(&mut s.iks, iks.to_vec()),
    );
    if let Err(e) = validate(p) {
        let s = p
            .skins
            .iter_mut()
            .find(|s| s.group == group && s.name == name)
            .unwrap();
        (s.constraints, s.iks) = old;
        return Err(e);
    }
    Ok(())
}

/// Declare the bones that exist only while skin `name` is active (empty list = none).
/// Everything is checked first ([`validate`]); on error the project is unchanged. Collapse
/// channels of bones that leave the list are removed (their scale channel must not be locked).
pub fn set_bones(p: &mut Project, group: &str, name: &str, bones: &[String]) -> Result<(), String> {
    let idx = p
        .skins
        .iter()
        .position(|s| s.group == group && s.name == name)
        .ok_or_else(|| format!("no skin '{name}' in group '{group}'"))?;
    let mut list: Vec<String> = Vec::new();
    for b in bones {
        if !p.bones.iter().any(|x| &x.name == b) {
            return Err(format!("no bone '{b}'"));
        }
        if !list.contains(b) {
            list.push(b.clone());
        }
    }
    let dropped: Vec<String> = p.skins[idx]
        .bones
        .iter()
        .filter(|b| !list.contains(b))
        .cloned()
        .collect();
    for b in &dropped {
        crate::chanmeta::assert_unlocked(p, b, Prop::BoneScale)?;
    }
    let old = std::mem::replace(&mut p.skins[idx].bones, list);
    if let Err(e) = validate(p) {
        p.skins[idx].bones = old;
        return Err(e);
    }
    p.channels
        .retain(|c| !(is_collapse_channel(c) && dropped.contains(&c.target)));
    Ok(())
}

/// Is this the multiply step channel (keys 0 / 1) that collapses an inactive skin bone?
pub fn is_collapse_channel(c: &Channel) -> bool {
    c.prop == Prop::BoneScale
        && c.mode == Mode::Multiply
        && c.weight == 1.0
        && matches!(&c.source, Source::Keys { track } if !track.keys.is_empty() && track.keys.iter().all(|k| k.ease == Ease::Step && (k.value == 0.0 || k.value == 1.0)))
}

/// True when `bone` is collapsed by a skin switch at `frame` (so overlays can hide it).
pub fn bone_collapsed(p: &Project, bone: &str, frame: f32) -> bool {
    p.channels.iter().any(|c| {
        c.target == bone
            && is_collapse_channel(c)
            && matches!(&c.source, Source::Keys { track } if track.eval(frame) == Some(0.0))
    })
}

fn group_bones(p: &Project, group: &str) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    for s in p.skins.iter().filter(|s| s.group == group) {
        for b in &s.bones {
            if !all.contains(b) {
                all.push(b.clone());
            }
        }
    }
    all
}

fn constraint_bones(c: &crate::project::Constraint) -> Vec<&str> {
    let mut v = vec![c.bone.as_str()];
    match &c.kind {
        ConstraintKind::Aim { target, .. } | ConstraintKind::Pin { target } => v.push(target),
        ConstraintKind::CopyRotation { source } => v.push(source),
    }
    v
}

/// Check the per-skin bone rules (see the module docs). Projects without skin bones always pass.
pub fn validate(p: &Project) -> Result<(), String> {
    let owner = |b: &str| p.skins.iter().find(|s| s.bones.iter().any(|x| x == b));
    for s in &p.skins {
        for b in &s.bones {
            let bone = p
                .bones
                .iter()
                .find(|x| &x.name == b)
                .ok_or_else(|| format!("skin '{}': no bone '{b}'", s.name))?;
            if bone.scale_limits.is_some() {
                return Err(format!(
                    "skin bone '{b}' has scale_limits, which would stop it collapsing; remove them"
                ));
            }
            if let Some(o) = p
                .skins
                .iter()
                .find(|o| (o.group != s.group || o.name != s.name) && o.bones.contains(b))
            {
                return Err(if o.group == s.group {
                    format!(
                        "bone '{b}' is used by two skins of group '{}' ('{}' and '{}')",
                        s.group, s.name, o.name
                    )
                } else {
                    format!(
                        "bone '{b}' is used by skins '{}/{}' and '{}/{}'",
                        s.group, s.name, o.group, o.name
                    )
                });
            }
        }
    }
    // children of a skin bone collapse with it, so they must belong to the same skin
    for b in &p.bones {
        let mut cur = b.parent.as_deref();
        let mut guard = 0;
        while let Some(pn) = cur {
            guard += 1;
            if guard > p.bones.len() {
                break;
            }
            if let Some(ps) = owner(pn) {
                if !ps.bones.contains(&b.name) {
                    return Err(format!("bone '{}' is a child of skin bone '{pn}' (skin '{}') and collapses with it: list it in that skin too", b.name, ps.name));
                }
            }
            cur = p
                .bones
                .iter()
                .find(|x| x.name == pn)
                .and_then(|x| x.parent.as_deref());
        }
    }
    // rig parts touching a skin bone are on only while that one skin is
    let check = |what: &str,
                 name: &str,
                 touched: Vec<&str>,
                 in_rig: &dyn Fn(&Skin) -> bool|
     -> Result<(), String> {
        let mut owners: Vec<&Skin> = Vec::new();
        for t in touched {
            if let Some(o) = owner(t) {
                if !owners.iter().any(|x| std::ptr::eq(*x, o)) {
                    owners.push(o);
                }
            }
        }
        match owners.len() {
            0 => Ok(()),
            1 => {
                let o = owners[0];
                if !in_rig(o) {
                    return Err(format!("{what} '{name}' uses bones of skin '{}': add it to that skin's rig list first", o.name));
                }
                if let Some(t) = p
                    .skins
                    .iter()
                    .find(|t| t.group == o.group && t.name != o.name && in_rig(t))
                {
                    return Err(format!("{what} '{name}' uses bones of skin '{}' but is also in the rig of skin '{}'", o.name, t.name));
                }
                Ok(())
            }
            _ => Err(format!("{what} '{name}' uses bones of two different skins")),
        }
    };
    for c in &p.constraints {
        check("constraint", &c.name, constraint_bones(c), &|s: &Skin| {
            s.constraints.contains(&c.name)
        })?;
    }
    for i in &p.ik {
        let mut t: Vec<&str> = i.bones.iter().map(|x| x.as_str()).collect();
        t.push(&i.target);
        check("IK chain", &i.name, t, &|s: &Skin| s.iks.contains(&i.name))?;
    }
    Ok(())
}

fn group_rig(p: &Project, group: &str) -> (Vec<String>, Vec<String>) {
    let (mut cs, mut is): (Vec<String>, Vec<String>) = (vec![], vec![]);
    for s in p.skins.iter().filter(|s| s.group == group) {
        for c in &s.constraints {
            if !cs.contains(c) {
                cs.push(c.clone());
            }
        }
        for i in &s.iks {
            if !is.contains(i) {
                is.push(i.clone());
            }
        }
    }
    (cs, is)
}

fn collapse(bone: &str, keys: &[(f32, f32)]) -> Channel {
    let k: Vec<(f32, f32, Ease)> = keys.iter().map(|&(f, v)| (f, v, Ease::Step)).collect();
    Channel::keys(bone, Prop::BoneScale, Track::with_keys(&k)).with_mode(Mode::Multiply)
}

/// Show `name`'s layers and hide the rest of its group (static `visible` flags).
pub fn apply_skin(p: &mut Project, group: &str, name: &str) -> Result<(), String> {
    validate(p)?;
    let gb = group_bones(p, group);
    for b in &gb {
        crate::chanmeta::assert_unlocked(p, b, Prop::BoneScale)?;
    }
    let skin = p
        .skins
        .iter()
        .find(|s| s.group == group && s.name == name)
        .ok_or_else(|| format!("no skin '{name}' in group '{group}'"))?
        .clone();
    for l in group_layers(p, group) {
        let on = skin.layers.contains(&l);
        if let Some(layer) = p.find_layer_mut(&l) {
            layer.visible = on;
        }
    }
    for b in &gb {
        p.channels
            .retain(|c| !(c.target == *b && is_collapse_channel(c)));
        if !skin.bones.contains(b) {
            p.channels.push(collapse(b, &[(0.0, 0.0)]));
        }
    }
    let (cs, is) = group_rig(p, group);
    for c in cs {
        let on = skin.constraints.contains(&c);
        if let Some(x) = p.constraints.iter_mut().find(|x| x.name == c) {
            x.weight = if on { 1.0 } else { 0.0 };
        }
    }
    for i in is {
        let on = skin.iks.contains(&i);
        if let Some(x) = p.ik.iter_mut().find(|x| x.name == i) {
            x.mix = if on { 1.0 } else { 0.0 };
        }
    }
    Ok(())
}

/// Step-key the group's layer opacities so the skin changes at the given frames.
/// Each layer's opacity is 1 while its skin is active and 0 otherwise.
pub fn schedule(p: &mut Project, group: &str, steps: &[(f32, String)]) -> Result<(), String> {
    if steps.is_empty() {
        return Err("need at least one (frame, skin) step".into());
    }
    let mut st = steps.to_vec();
    st.sort_by(|a, b| a.0.total_cmp(&b.0));
    if st.windows(2).any(|w| w[0].0 == w[1].0) || st.iter().any(|s| !s.0.is_finite() || s.0 < 0.0) {
        return Err("step frames must be distinct, finite and non-negative".into());
    }
    let skins: Vec<&crate::skins::Skin> = st
        .iter()
        .map(|(_, n)| {
            p.skins
                .iter()
                .find(|s| s.group == group && &s.name == n)
                .ok_or_else(|| format!("no skin '{n}' in group '{group}'"))
        })
        .collect::<Result<_, _>>()?;
    validate(p)?;
    let layers = group_layers(p, group);
    for l in &layers {
        crate::chanmeta::assert_unlocked(p, l, Prop::LayerOpacity)?;
    }
    let gb = group_bones(p, group);
    for b in &gb {
        crate::chanmeta::assert_unlocked(p, b, Prop::BoneScale)?;
    }
    let mut new = Vec::new();
    for l in &layers {
        let keys: Vec<(f32, f32, Ease)> = st
            .iter()
            .zip(&skins)
            .map(|((f, _), s)| (*f, if s.layers.contains(l) { 1.0 } else { 0.0 }, Ease::Step))
            .collect();
        new.push(Channel::keys(
            l,
            Prop::LayerOpacity,
            Track::with_keys(&keys),
        ));
    }
    let (cs, is) = group_rig(p, group);
    for c in &cs {
        crate::chanmeta::assert_unlocked(p, c, Prop::ConstraintWeight)?;
    }
    for i in &is {
        crate::chanmeta::assert_unlocked(p, i, Prop::IkMix)?;
    }
    for c in &cs {
        let keys: Vec<(f32, f32, Ease)> = st
            .iter()
            .zip(&skins)
            .map(|((f, _), s)| {
                (
                    *f,
                    if s.constraints.contains(c) { 1.0 } else { 0.0 },
                    Ease::Step,
                )
            })
            .collect();
        new.push(Channel::keys(
            c,
            Prop::ConstraintWeight,
            Track::with_keys(&keys),
        ));
    }
    for i in &is {
        let keys: Vec<(f32, f32, Ease)> = st
            .iter()
            .zip(&skins)
            .map(|((f, _), s)| (*f, if s.iks.contains(i) { 1.0 } else { 0.0 }, Ease::Step))
            .collect();
        new.push(Channel::keys(i, Prop::IkMix, Track::with_keys(&keys)));
    }
    for c in new {
        p.channels.retain(|x| {
            !(x.target == c.target
                && x.prop == c.prop
                && matches!(x.source, crate::project::Source::Keys { .. }))
        });
        p.channels.push(c);
    }
    for b in &gb {
        let keys: Vec<(f32, f32)> = st
            .iter()
            .zip(&skins)
            .map(|((f, _), s)| (*f, if s.bones.contains(b) { 1.0 } else { 0.0 }))
            .collect();
        p.channels
            .retain(|x| !(x.target == *b && is_collapse_channel(x)));
        p.channels.push(collapse(b, &keys));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::VPath;
    use crate::project::Layer;
    use crate::Engine;

    fn proj() -> Project {
        let mut p = Project::new("s", 100.0, 100.0, 24.0, 20);
        p.layers = ["hat", "cap", "coat", "scarf"]
            .iter()
            .map(|n| {
                Layer::vector(
                    n,
                    vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#fff")],
                )
            })
            .collect();
        add_skin(&mut p, "outfit", "winter", &["hat".into(), "coat".into()]).unwrap();
        add_skin(&mut p, "outfit", "summer", &["cap".into()]).unwrap();
        p
    }
    fn visible(p: &Project, f: f32) -> Vec<String> {
        let fr = Engine::new(p.clone()).unwrap().evaluate(f);
        fr.nodes
            .iter()
            .filter(|n| n.visible && n.opacity > 0.5)
            .map(|n| n.name.clone())
            .collect()
    }

    #[test]
    fn apply_swaps_the_group_and_leaves_others_alone() {
        let mut p = proj();
        apply_skin(&mut p, "outfit", "winter").unwrap();
        assert_eq!(
            visible(&p, 0.0),
            vec!["hat", "coat", "scarf"],
            "scarf is in no skin: untouched"
        );
        apply_skin(&mut p, "outfit", "summer").unwrap();
        assert_eq!(visible(&p, 0.0), vec!["cap", "scarf"]);
        assert!(apply_skin(&mut p, "outfit", "spring").is_err());
        assert!(apply_skin(&mut p, "mood", "winter").is_err());
    }

    #[test]
    fn schedule_changes_skin_over_time() {
        let mut p = proj();
        schedule(
            &mut p,
            "outfit",
            &[(0.0, "summer".into()), (10.0, "winter".into())],
        )
        .unwrap();
        assert_eq!(visible(&p, 5.0), vec!["cap", "scarf"]);
        assert_eq!(visible(&p, 12.0), vec!["hat", "coat", "scarf"]);
        schedule(&mut p, "outfit", &[(0.0, "summer".into())]).unwrap();
        assert_eq!(
            p.channels
                .iter()
                .filter(|c| c.prop == Prop::LayerOpacity)
                .count(),
            3,
            "re-running replaces, not duplicates"
        );
        assert!(schedule(&mut p, "outfit", &[]).is_err());
        assert!(schedule(&mut p, "outfit", &[(1.0, "x".into())]).is_err());
    }

    #[test]
    fn validation_and_roundtrip() {
        let mut p = proj();
        assert!(add_skin(&mut p, "outfit", "bad", &["ghost".into()]).is_err());
        assert!(add_skin(&mut p, "", "n", &["hat".into()]).is_err());
        add_skin(&mut p, "outfit", "winter", &["hat".into()]).unwrap();
        assert_eq!(p.skins.len(), 2, "same name overwrites");
        let back = Project::from_json(&p.to_json_pretty()).unwrap();
        assert_eq!(back.skins, p.skins);
    }

    #[test]
    fn skins_switch_constraints_and_ik() {
        use crate::project::{Bone, Constraint, ConstraintKind, IkChain};
        let mut p = proj();
        p.bones = vec![
            Bone::new("a", None, 10.0, crate::math::Vec2::ZERO, 0.0),
            Bone::new("b", Some("a"), 10.0, crate::math::Vec2::new(10.0, 0.0), 0.0),
            Bone::new("t", None, 1.0, crate::math::Vec2::new(5.0, 5.0), 0.0),
        ];
        p.constraints = vec![Constraint {
            name: "aim".into(),
            bone: "a".into(),
            kind: ConstraintKind::Aim {
                target: "t".into(),
                offset: 0.0,
            },
            weight: 1.0,
        }];
        p.ik = vec![serde_json::from_str::<IkChain>(
            r#"{"name":"arm","bones":["a","b"],"target":"t"}"#,
        )
        .unwrap()];
        set_rig(&mut p, "outfit", "winter", &["aim".into()], &["arm".into()]).unwrap();
        assert!(set_rig(&mut p, "outfit", "winter", &["ghost".into()], &[]).is_err());
        apply_skin(&mut p, "outfit", "summer").unwrap();
        assert_eq!((p.constraints[0].weight, p.ik[0].mix), (0.0, 0.0));
        apply_skin(&mut p, "outfit", "winter").unwrap();
        assert_eq!((p.constraints[0].weight, p.ik[0].mix), (1.0, 1.0));
        schedule(
            &mut p,
            "outfit",
            &[(0.0, "summer".into()), (10.0, "winter".into())],
        )
        .unwrap();
        assert_eq!(
            p.channels
                .iter()
                .filter(|c| c.prop == Prop::ConstraintWeight || c.prop == Prop::IkMix)
                .count(),
            2
        );
        Engine::new(p.clone()).unwrap();
        let j = p.to_json_pretty();
        assert_eq!(Project::from_json(&j).unwrap().skins, p.skins);
    }

    // ---- per-skin bones ----

    fn bone_proj() -> Project {
        use crate::project::Bone;
        let mut p = proj();
        let v = crate::math::Vec2::new;
        p.bones = vec![
            Bone::new("root", None, 10.0, v(10.0, 50.0), 0.0),
            Bone::new("cape1", Some("root"), 10.0, v(10.0, 0.0), 0.0),
            Bone::new("cape2", Some("cape1"), 10.0, v(10.0, 0.0), 0.0),
            Bone::new("tail", Some("root"), 10.0, v(0.0, 10.0), 0.0),
        ];
        // a rigid follower on each skin bone, in no skin, so only the bone decides its fate
        for (n, b, x, y) in [
            ("capef", "cape2", 26.0, 46.0),
            ("tailf", "tail", 6.0, 56.0),
            ("rootf", "root", 6.0, 46.0),
        ] {
            let mut l = Layer::vector(n, vec![VPath::rect("r", x, y, 8.0, 8.0).with_fill("#fff")]);
            l.bone = Some(b.into());
            p.layers.push(l);
        }
        set_bones(
            &mut p,
            "outfit",
            "winter",
            &["cape1".into(), "cape2".into()],
        )
        .unwrap();
        set_bones(&mut p, "outfit", "summer", &["tail".into()]).unwrap();
        p
    }
    fn area(p: &Project, f: f32, bone: &str) -> f32 {
        let e = Engine::new(p.clone()).unwrap();
        let w = e.evaluate(f).world[p.bones.iter().position(|b| b.name == bone).unwrap()];
        (w.a * w.d - w.b * w.c).abs()
    }
    fn ink(p: &Project, f: f32, layer: &str) -> usize {
        let mut q = p.clone();
        for l in q.layers.iter_mut() {
            l.visible = l.name == layer;
        }
        let e = Engine::new(q).unwrap();
        let o = crate::raster::RasterOptions {
            scale: 1.0,
            show_bones: false,
            transparent: true,
        };
        let (_, _, px) = crate::raster::render_rgba(&e, f, &o);
        px.chunks(4).filter(|c| c[3] > 0).count()
    }

    #[test]
    fn apply_collapses_inactive_bones_and_their_children() {
        let mut p = bone_proj();
        apply_skin(&mut p, "outfit", "winter").unwrap();
        assert!(
            area(&p, 0.0, "cape1") > 0.5
                && area(&p, 0.0, "cape2") > 0.5
                && area(&p, 0.0, "tail") < 1e-9
        );
        assert!(
            ink(&p, 0.0, "capef") > 0 && ink(&p, 0.0, "tailf") == 0,
            "rigid follower of the inactive tail vanishes"
        );
        assert!(ink(&p, 0.0, "rootf") > 0, "root is in no skin");
        apply_skin(&mut p, "outfit", "summer").unwrap();
        assert!(
            area(&p, 0.0, "cape1") < 1e-9
                && area(&p, 0.0, "cape2") < 1e-9
                && area(&p, 0.0, "tail") > 0.5
        );
        assert!(ink(&p, 0.0, "capef") == 0 && ink(&p, 0.0, "tailf") > 0);
        assert_eq!(
            p.channels.iter().filter(|c| is_collapse_channel(c)).count(),
            2,
            "re-applying replaces"
        );
        apply_skin(&mut p, "outfit", "winter").unwrap();
        apply_skin(&mut p, "outfit", "winter").unwrap();
        assert_eq!(
            p.channels.iter().filter(|c| is_collapse_channel(c)).count(),
            1
        );
    }

    #[test]
    fn schedule_switches_bones_exactly_and_keeps_user_scale_animation() {
        let mut p = bone_proj();
        p.channels.push(Channel::keys(
            "tail",
            Prop::BoneScale,
            Track::with_keys(&[(0.0, 2.0, Ease::Linear)]),
        ));
        schedule(
            &mut p,
            "outfit",
            &[
                (0.0, "winter".into()),
                (10.0, "summer".into()),
                (15.0, "winter".into()),
            ],
        )
        .unwrap();
        assert!(
            area(&p, 9.0, "tail") < 1e-9 && area(&p, 10.0, "tail") > 3.9,
            "user scale 2 (area 4) survives and returns exactly at the key"
        );
        assert!(area(&p, 14.0, "tail") > 3.9 && area(&p, 15.0, "tail") < 1e-9);
        assert!(
            area(&p, 9.0, "cape2") > 0.5
                && area(&p, 10.0, "cape2") < 1e-9
                && area(&p, 15.0, "cape2") > 0.5
        );
        assert!(ink(&p, 12.0, "tailf") > 0 && ink(&p, 12.0, "capef") == 0);
        assert!(bone_collapsed(&p, "tail", 9.0) && !bone_collapsed(&p, "tail", 11.0));
        schedule(&mut p, "outfit", &[(0.0, "summer".into())]).unwrap();
        assert_eq!(
            p.channels.iter().filter(|c| is_collapse_channel(c)).count(),
            3,
            "replaced, not duplicated"
        );
        assert_eq!(
            p.channels.iter().filter(|c| c.target == "tail").count(),
            2,
            "user channel kept"
        );
    }

    #[test]
    fn overlay_hides_collapsed_bones() {
        let mut p = bone_proj();
        assert!(crate::bonevis::visible(&p, "tail", 3.0));
        apply_skin(&mut p, "outfit", "winter").unwrap();
        assert!(
            !crate::bonevis::visible(&p, "tail", 3.0) && crate::bonevis::visible(&p, "cape1", 3.0)
        );
        crate::bonevis::set_range(&mut p, "cape1", 0.0, 2.0).unwrap();
        assert!(
            crate::bonevis::visible(&p, "cape1", 1.0) && !crate::bonevis::visible(&p, "cape1", 3.0),
            "ranges still apply"
        );
    }

    #[test]
    fn bone_validation_rules() {
        let mut p = bone_proj();
        assert!(set_bones(&mut p, "outfit", "winter", &["ghost".into()])
            .unwrap_err()
            .contains("no bone"));
        assert!(set_bones(&mut p, "outfit", "nope", &[]).is_err());
        let before = p.skins.clone();
        let e = set_bones(
            &mut p,
            "outfit",
            "summer",
            &["cape1".into(), "cape2".into()],
        )
        .unwrap_err();
        assert!(e.contains("two skins"), "{e}");
        assert_eq!(p.skins, before, "failed edit leaves the project unchanged");
        assert!(
            set_bones(&mut p, "outfit", "winter", &["cape1".into()])
                .unwrap_err()
                .contains("child of skin bone"),
            "cape2 would collapse with cape1 but is unlisted"
        );
        let e = set_bones(&mut p, "outfit", "summer", &["root".into()]).unwrap_err();
        assert!(e.contains("child of skin bone"), "{e}");
        p.bones[3].scale_limits = Some([0.5, 2.0]);
        assert!(set_bones(&mut p, "outfit", "summer", &["tail".into()])
            .unwrap_err()
            .contains("scale_limits"));
        p.bones[3].scale_limits = None;
        // same bone in a skin of another group is refused as well
        add_skin(&mut p, "mood", "sad", &["hat".into()]).unwrap();
        assert!(set_bones(&mut p, "mood", "sad", &["tail".into()])
            .unwrap_err()
            .contains("used by"));
        // dropping a bone removes its collapse channel; locked scale channels block apply/schedule
        apply_skin(&mut p, "outfit", "winter").unwrap();
        assert_eq!(
            p.channels.iter().filter(|c| is_collapse_channel(c)).count(),
            1
        );
        set_bones(&mut p, "outfit", "summer", &[]).unwrap();
        assert_eq!(
            p.channels.iter().filter(|c| is_collapse_channel(c)).count(),
            0
        );
        set_bones(&mut p, "outfit", "summer", &["tail".into()]).unwrap();
        p.channel_meta.push(crate::chanmeta::ChannelMeta {
            target: "cape1".into(),
            prop: Some(Prop::BoneScale),
            color: None,
            shy: false,
            solo: false,
            lock: true,
        });
        let n = p.channels.len();
        assert!(apply_skin(&mut p, "outfit", "summer")
            .unwrap_err()
            .contains("locked"));
        assert!(schedule(&mut p, "outfit", &[(0.0, "summer".into())])
            .unwrap_err()
            .contains("locked"));
        assert_eq!(p.channels.len(), n, "nothing written when locked");
    }

    #[test]
    fn rig_parts_on_skin_bones_follow_their_skin() {
        use crate::project::{Constraint, ConstraintKind};
        let mut p = bone_proj();
        p.constraints = vec![Constraint {
            name: "tailaim".into(),
            bone: "tail".into(),
            kind: ConstraintKind::Aim {
                target: "root".into(),
                offset: 0.0,
            },
            weight: 1.0,
        }];
        let e = validate(&p).unwrap_err();
        assert!(e.contains("rig list"), "{e}");
        assert!(
            apply_skin(&mut p, "outfit", "summer").is_err(),
            "hand-edited invalid project is refused"
        );
        set_rig(&mut p, "outfit", "summer", &["tailaim".into()], &[]).unwrap();
        assert!(
            set_rig(&mut p, "outfit", "winter", &["tailaim".into()], &[])
                .unwrap_err()
                .contains("also in the rig")
        );
        assert!(
            p.skins
                .iter()
                .find(|s| s.name == "winter")
                .unwrap()
                .constraints
                .is_empty(),
            "rolled back"
        );
        assert!(
            set_rig(&mut p, "outfit", "summer", &[], &[]).is_err(),
            "cannot detach a rig part from the skin that owns its bone"
        );
        apply_skin(&mut p, "outfit", "winter").unwrap();
        assert_eq!(p.constraints[0].weight, 0.0);
        Engine::new(p.clone()).unwrap().evaluate(0.0);
        apply_skin(&mut p, "outfit", "summer").unwrap();
        assert_eq!(p.constraints[0].weight, 1.0);
    }

    #[test]
    fn old_projects_load_and_bones_roundtrip() {
        let p = proj();
        let j = p.to_json_pretty();
        let v: serde_json::Value = serde_json::from_str(&j).unwrap();
        assert!(
            v["skins"][0].get("bones").is_none(),
            "empty skin bones are not written"
        );
        let old = r#"{"name":"winter","group":"outfit","layers":["hat"]}"#;
        let s: Skin = serde_json::from_str(old).unwrap();
        assert!(s.bones.is_empty() && s.constraints.is_empty());
        let q = bone_proj();
        let back = Project::from_json(&q.to_json_pretty()).unwrap();
        assert_eq!(back.skins, q.skins);
        assert_eq!(back.skins[0].bones, vec!["cape1", "cape2"]);
    }

    #[test]
    fn undo_restores_the_geometry() {
        let mut p = bone_proj();
        let mut ed = crate::edit::Editor::new(p.clone()).unwrap();
        assert!(area(ed.project(), 0.0, "tail") > 0.5);
        apply_skin(&mut p, "outfit", "winter").unwrap();
        let doc = serde_json::to_value(&p).unwrap();
        ed.apply(crate::edit::Op::Set {
            path: String::new(),
            value: doc,
        })
        .unwrap();
        assert!(area(ed.project(), 0.0, "tail") < 1e-9);
        // the editor's key tools never write into the collapse channel
        ed.set_key("tail", Prop::BoneScale, 0.0, 1.5, Ease::Linear)
            .unwrap();
        assert!(
            ed.project()
                .channels
                .iter()
                .filter(|c| is_collapse_channel(c))
                .count()
                == 1
        );
        assert!(ed.undo() && ed.undo());
        assert!(
            area(ed.project(), 0.0, "tail") > 0.5,
            "undo brings the bone back"
        );
        assert!(ed.redo() && area(ed.project(), 0.0, "tail") < 1e-9);
    }
}
