// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Automatic eye clipping: mask each iris/pupil layer by the eye-shape layer on the same side,
//! so the iris never leaves the eye while it moves. Uses ordinary layer masks.

use crate::project::{Layer, MaskRef, Project};
use crate::vocab::{is_iris, role_of, side_of};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClipReport {
    pub clipped: Vec<String>,
    pub skipped: Vec<String>,
}

fn collect(ls: &[Layer], prefix: &str, eyes: &mut Vec<String>, irises: &mut Vec<(String, bool)>) {
    for l in ls {
        let path = if prefix.is_empty() {
            l.name.clone()
        } else {
            format!("{prefix}/{}", l.name)
        };
        if is_iris(&l.name) {
            irises.push((path.clone(), l.mask.is_some()));
        } else if role_of(&l.name).is_some_and(|(r, _)| r.starts_with("face.eye")) {
            eyes.push(path.clone());
        }
        collect(l.children(), &path, eyes, irises);
    }
}

fn parent_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(a, _)| a)
}

pub fn auto_eye_clip(p: &mut Project) -> ClipReport {
    let (mut eyes, mut irises) = (Vec::new(), Vec::new());
    collect(&p.layers, "", &mut eyes, &mut irises);
    let mut rep = ClipReport::default();
    for (iris, has_mask) in irises {
        if has_mask {
            rep.skipped.push(format!("{iris}: already masked"));
            continue;
        }
        let s = side_of(iris.rsplit('/').next().unwrap_or(&iris));
        let cands: Vec<&String> = eyes
            .iter()
            .filter(|e| side_of(e.rsplit('/').next().unwrap_or(e)) == s)
            .collect();
        let pick = cands
            .iter()
            .find(|e| parent_of(e) == parent_of(&iris))
            .or(cands.first())
            .map(|e| (*e).clone());
        match pick {
            Some(eye) if cands.len() == 1 || s.is_some() || parent_of(&eye) == parent_of(&iris) => {
                if let Some(l) = p.find_layer_mut(&iris) {
                    l.mask = Some(MaskRef {
                        layer: eye.clone(),
                        invert: false,
                    });
                    rep.clipped.push(format!("{iris} <- {eye}"));
                }
            }
            _ => rep.skipped.push(format!("{iris}: no matching eye layer")),
        }
    }
    rep
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::VPath;
    use crate::Engine;

    fn face() -> Project {
        let mut p = Project::new("e", 200.0, 100.0, 24.0, 4);
        let sh = |n: &str, x: f32| {
            Layer::vector(
                n,
                vec![VPath::rect("r", x, 20.0, 30.0, 20.0).with_fill("#ffffff")],
            )
        };
        p.layers = vec![
            sh("eye_L", 20.0),
            sh("iris_L", 25.0),
            sh("eye_R", 120.0),
            sh("iris_R", 125.0),
            sh("pupil", 0.0),
        ];
        p
    }

    #[test]
    fn clips_each_iris_to_its_own_side_and_keeps_project_valid() {
        let mut p = face();
        let rep = auto_eye_clip(&mut p);
        assert_eq!(rep.clipped.len(), 2, "{rep:?}");
        assert_eq!(
            p.find_layer("iris_L").unwrap().mask.as_ref().unwrap().layer,
            "eye_L"
        );
        assert_eq!(
            p.find_layer("iris_R").unwrap().mask.as_ref().unwrap().layer,
            "eye_R"
        );
        assert!(
            rep.skipped.iter().any(|s| s.starts_with("pupil")),
            "side-less pupil with two eyes is ambiguous"
        );
        Engine::new(p).unwrap();
    }

    #[test]
    fn idempotent_and_respects_existing_masks() {
        let mut p = face();
        auto_eye_clip(&mut p);
        let rep = auto_eye_clip(&mut p);
        assert!(rep.clipped.is_empty());
        assert!(rep.skipped.iter().any(|s| s.contains("already masked")));
    }

    #[test]
    fn single_eye_without_side_is_unambiguous() {
        let mut p = Project::new("e", 100.0, 100.0, 24.0, 2);
        let sh = |n: &str| {
            Layer::vector(
                n,
                vec![VPath::rect("r", 0.0, 0.0, 10.0, 10.0).with_fill("#fff")],
            )
        };
        p.layers = vec![sh("eye"), sh("iris")];
        assert_eq!(auto_eye_clip(&mut p).clipped.len(), 1);
    }
}
