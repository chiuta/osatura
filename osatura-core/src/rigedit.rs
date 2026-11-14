// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Rig-structure edits that preserve the rest pose.

use crate::math::Vec2;
use crate::project::Project;

fn rot(v: Vec2, deg: f32) -> Vec2 {
    let (s, c) = deg.to_radians().sin_cos();
    Vec2::new(c * v.x - s * v.y, s * v.x + c * v.y)
}

/// World head position and world angle (degrees) of every bone in the rest pose.
pub fn rest_world(p: &Project) -> Result<Vec<(Vec2, f32)>, String> {
    let mut out: Vec<(Vec2, f32)> = Vec::with_capacity(p.bones.len());
    for b in &p.bones {
        let w = match &b.parent {
            None => (b.rest_offset, b.rest_angle),
            Some(pn) => {
                let pi = p.bones.iter().position(|x| &x.name == pn).filter(|&i| i < out.len()).ok_or_else(|| format!("bone '{}' lists parent '{pn}' after itself or not at all", b.name))?;
                let (pp, pa) = out[pi];
                (pp + rot(b.rest_offset, pa), pa + b.rest_angle)
            }
        };
        out.push(w);
    }
    Ok(out)
}

/// Make `new_parent` (or nothing, for a root) the parent of `bone` without moving it in the
/// rest pose. Bones are re-ordered so parents always precede children. Animation channels
/// keep their meaning (they are deltas from rest), but the *animated* pose follows the new parent.
pub fn set_parent(p: &mut Project, bone: &str, new_parent: Option<&str>) -> Result<(), String> {
    let bi = p.bones.iter().position(|b| b.name == bone).ok_or_else(|| format!("no bone '{bone}'"))?;
    if let Some(np) = new_parent {
        if !p.bones.iter().any(|b| b.name == np) {
            return Err(format!("no bone '{np}'"));
        }
        // the new parent must not be the bone itself or one of its descendants
        let mut cur = Some(np.to_string());
        while let Some(c) = cur {
            if c == bone {
                return Err(format!("'{np}' is '{bone}' or one of its descendants; that would create a cycle"));
            }
            cur = p.bones.iter().find(|b| b.name == c).and_then(|b| b.parent.clone());
        }
    }
    let world = rest_world(p)?;
    let (pos, ang) = world[bi];
    let (new_off, new_ang) = match new_parent {
        None => (pos, ang),
        Some(np) => {
            let pj = p.bones.iter().position(|b| b.name == np).unwrap();
            let (pp, pa) = world[pj];
            (rot(pos - pp, -pa), ang - pa)
        }
    };
    {
        let b = &mut p.bones[bi];
        b.parent = new_parent.map(String::from);
        b.rest_offset = new_off;
        b.rest_angle = new_ang;
    }
    // stable re-order: emit a bone once its parent has been emitted
    let mut done: Vec<bool> = vec![false; p.bones.len()];
    let mut order: Vec<usize> = Vec::with_capacity(p.bones.len());
    while order.len() < p.bones.len() {
        let before = order.len();
        for i in 0..p.bones.len() {
            if done[i] {
                continue;
            }
            let ready = match &p.bones[i].parent {
                None => true,
                Some(pn) => p.bones.iter().position(|b| &b.name == pn).is_some_and(|j| done[j]),
            };
            if ready {
                done[i] = true;
                order.push(i);
            }
        }
        if order.len() == before {
            return Err("bone hierarchy is cyclic or refers to a missing parent".into());
        }
    }
    let old = std::mem::take(&mut p.bones);
    p.bones = order.into_iter().map(|i| old[i].clone()).collect();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Bone;

    fn p() -> Project {
        let mut p = Project::new("t", 100.0, 100.0, 24.0, 10);
        p.bones = vec![
            Bone::new("a", None, 20.0, Vec2::new(10.0, 10.0), 30.0),
            Bone::new("b", Some("a"), 20.0, Vec2::new(20.0, 0.0), 15.0),
            Bone::new("c", None, 10.0, Vec2::new(50.0, 40.0), -20.0),
        ];
        p
    }

    fn close(a: (Vec2, f32), b: (Vec2, f32)) -> bool {
        a.0.distance(b.0) < 1e-3 && (a.1 - b.1).abs() < 1e-3
    }

    #[test]
    fn reparent_keeps_the_rest_world_pose() {
        let mut q = p();
        let before = rest_world(&q).unwrap();
        set_parent(&mut q, "c", Some("b")).unwrap();
        assert_eq!(q.bones.iter().find(|b| b.name == "c").unwrap().parent.as_deref(), Some("b"));
        let after = rest_world(&q).unwrap();
        for (i, b) in p().bones.iter().enumerate() {
            let j = q.bones.iter().position(|x| x.name == b.name).unwrap();
            assert!(close(before[i], after[j]), "{} moved: {:?} vs {:?}", b.name, before[i], after[j]);
        }
    }

    #[test]
    fn unparent_and_reorder() {
        let mut q = p();
        set_parent(&mut q, "b", None).unwrap();
        assert!(q.bones.iter().find(|b| b.name == "b").unwrap().parent.is_none());
        // now make 'a' a child of 'c' (listed later) — order must be fixed
        set_parent(&mut q, "a", Some("c")).unwrap();
        let names: Vec<_> = q.bones.iter().map(|b| b.name.as_str()).collect();
        assert!(names.iter().position(|n| *n == "c") < names.iter().position(|n| *n == "a"), "{names:?}");
        assert!(rest_world(&q).is_ok());
        assert!(crate::Engine::new(q).is_ok());
    }

    #[test]
    fn cycles_and_unknowns_are_rejected_without_changes() {
        let mut q = p();
        let snapshot = q.clone();
        assert!(set_parent(&mut q, "a", Some("b")).is_err(), "b is a's child");
        assert!(set_parent(&mut q, "a", Some("a")).is_err());
        assert!(set_parent(&mut q, "zzz", None).is_err());
        assert!(set_parent(&mut q, "a", Some("zzz")).is_err());
        assert_eq!(q, snapshot);
    }
}
