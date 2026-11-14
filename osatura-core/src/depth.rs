// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Depth helper for multiplane parallax: spread the `parallax` factor of top-level layers
//! evenly from far to near, in the order given. Far layers move less with the camera.

use crate::project::Project;

/// `names` ordered far → near. Values interpolate linearly from `far` to `near`.
pub fn spread_parallax(
    p: &mut Project,
    names: &[String],
    far: f32,
    near: f32,
) -> Result<(), String> {
    if names.len() < 2 {
        return Err("give at least two layers (far to near)".into());
    }
    if !far.is_finite() || !near.is_finite() || far < 0.0 || near < 0.0 {
        return Err("parallax values must be finite and not negative".into());
    }
    for n in names {
        if !p.layers.iter().any(|l| &l.name == n) {
            return Err(format!(
                "'{n}' is not a top-level layer (parallax only applies to those)"
            ));
        }
    }
    let k = (names.len() - 1) as f32;
    for (i, n) in names.iter().enumerate() {
        let v = far + (near - far) * i as f32 / k;
        if let Some(l) = p.layers.iter_mut().find(|l| &l.name == n) {
            l.parallax = v;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::VPath;
    use crate::project::Layer;

    fn proj() -> Project {
        let mut p = Project::new("d", 100.0, 100.0, 24.0, 2);
        p.layers = ["sky", "hills", "trees", "fence"]
            .iter()
            .map(|n| {
                Layer::vector(
                    n,
                    vec![VPath::rect("r", 0.0, 0.0, 5.0, 5.0).with_fill("#fff")],
                )
            })
            .collect();
        p
    }

    #[test]
    fn spreads_evenly_far_to_near() {
        let mut p = proj();
        let names: Vec<String> = ["sky", "hills", "trees", "fence"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        spread_parallax(&mut p, &names, 0.2, 1.4).unwrap();
        let v: Vec<f32> = p.layers.iter().map(|l| l.parallax).collect();
        for (a, b) in v.iter().zip([0.2, 0.6, 1.0, 1.4]) {
            assert!((a - b).abs() < 1e-5, "{v:?}");
        }
    }

    #[test]
    fn rejects_bad_input_without_changes() {
        let mut p = proj();
        assert!(spread_parallax(&mut p, &["sky".into()], 0.2, 1.0).is_err());
        assert!(spread_parallax(&mut p, &["sky".into(), "ghost".into()], 0.2, 1.0).is_err());
        assert!(spread_parallax(&mut p, &["sky".into(), "hills".into()], -1.0, 1.0).is_err());
        assert!(p.layers.iter().all(|l| l.parallax == 1.0));
    }
}
