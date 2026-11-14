// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Soft and stretchy IK.
#[cfg(test)]
mod tests {
    use crate::math::Vec2;
    use crate::project::{Bone, IkChain, Project};
    use crate::Engine;

    fn rig(target_x: f32, soft: f32, stretch: f32) -> Project {
        let mut p = Project::new("ik", 400.0, 200.0, 24.0, 2);
        p.bones = vec![
            Bone::new("upper", None, 30.0, Vec2::new(50.0, 100.0), 0.0),
            Bone::new("lower", Some("upper"), 30.0, Vec2::new(30.0, 0.0), 0.0),
            Bone::new("goal", None, 5.0, Vec2::new(50.0 + target_x, 100.0), 0.0),
        ];
        let mut c = IkChain::two_bone("arm", "upper", "lower", "goal", 1.0);
        c.soft = soft;
        c.stretch = stretch;
        p.ik = vec![c];
        p
    }

    fn tip_distance(p: Project) -> f32 {
        let e = Engine::new(p).unwrap();
        let f = e.evaluate(0.0);
        let tip = f.world[1].apply(Vec2::new(30.0, 0.0));
        tip.distance(Vec2::new(50.0, 100.0))
    }

    #[test]
    fn without_modes_the_chain_stops_at_full_length() {
        assert!((tip_distance(rig(80.0, 0.0, 1.0)) - 60.0).abs() < 0.5);
    }

    #[test]
    fn stretch_reaches_far_targets_up_to_its_limit() {
        let d = tip_distance(rig(80.0, 0.0, 2.0));
        assert!((d - 80.0).abs() < 0.8, "stretched to reach: {d}");
        let d = tip_distance(rig(80.0, 0.0, 1.2));
        assert!((d - 72.0).abs() < 0.8, "capped at 1.2× (72 px): {d}");
        // a reachable target is untouched by stretch
        let d = tip_distance(rig(45.0, 0.0, 3.0));
        assert!((d - 45.0).abs() < 0.5, "{d}");
    }

    #[test]
    fn soft_eases_the_last_stretch_of_extension() {
        let hard = tip_distance(rig(58.0, 0.0, 1.0));
        let soft = tip_distance(rig(58.0, 0.3, 1.0));
        assert!((hard - 58.0).abs() < 0.5);
        assert!(soft < hard - 0.5 && soft < 60.0, "soft {soft} vs hard {hard}");
        // far outside reach it still ends up (almost) straight, never beyond full length
        let far = tip_distance(rig(200.0, 0.3, 1.0));
        assert!(far <= 60.01 && far > 59.0, "{far}");
        // monotone in the target distance
        let mut prev = 0.0;
        for x in [20.0, 40.0, 50.0, 55.0, 58.0, 62.0, 80.0] {
            let d = tip_distance(rig(x, 0.3, 1.0));
            assert!(d >= prev - 1e-3, "monotone at {x}: {d} < {prev}");
            prev = d;
        }
    }

    /// Wave 7 (was L-70): soft and stretch together. The chain used to hold back inside full reach (soft) and then
    /// snap straight and stretch beyond it; now the soft zone is the end of the stretched reach, so the tip distance
    /// is continuous and monotone in the target distance, never beyond the stretch limit.
    #[test]
    fn soft_and_stretch_together_are_continuous_and_bounded() {
        let mut prev = 0.0f32;
        let mut max_step = 0.0f32;
        let mut x = 5.0;
        while x < 200.0 {
            let d = tip_distance(rig(x, 0.5, 2.0));
            assert!(d >= prev - 1e-2, "monotone at {x}: {d} < {prev}");
            assert!(d <= 120.01, "never beyond stretch x reach (120): {d}");
            if x > 5.0 {
                max_step = max_step.max(d - prev);
            }
            prev = d;
            x += 1.0;
        }
        assert!(max_step < 1.5, "no pop: largest 1 px step of the target moved the tip {max_step} px");
        assert!(prev > 110.0, "far targets approach the stretch limit: {prev}");
        // inside the soft zone of the stretched reach the tip is held back, below it the target is met exactly
        assert!((tip_distance(rig(40.0, 0.5, 2.0)) - 40.0).abs() < 0.5);
        assert!(tip_distance(rig(100.0, 0.5, 2.0)) < 99.0, "held back near the stretch limit");
        // the same rig without soft stretches to exactly the target (unchanged behaviour)
        assert!((tip_distance(rig(100.0, 0.0, 2.0)) - 100.0).abs() < 0.8);
    }

    #[test]
    fn invalid_values_are_rejected_and_old_files_still_load() {
        assert!(Engine::new(rig(40.0, 0.95, 1.0)).is_err());
        assert!(Engine::new(rig(40.0, 0.0, 0.5)).is_err());
        assert!(Engine::new(rig(40.0, 0.0, 9.0)).is_err());
        let mut v = serde_json::to_value(rig(40.0, 0.0, 1.0)).unwrap();
        v["ik"][0].as_object_mut().unwrap().remove("soft");
        v["ik"][0].as_object_mut().unwrap().remove("stretch");
        let p: Project = serde_json::from_value(v).unwrap();
        assert_eq!((p.ik[0].soft, p.ik[0].stretch), (0.0, 1.0));
        assert!(Engine::new(p).is_ok());
    }

    #[test]
    fn long_chains_stretch_too() {
        let mut p = Project::new("ik3", 400.0, 200.0, 24.0, 2);
        p.bones = vec![
            Bone::new("a", None, 20.0, Vec2::new(50.0, 100.0), 0.0),
            Bone::new("b", Some("a"), 20.0, Vec2::new(20.0, 0.0), 0.0),
            Bone::new("c", Some("b"), 20.0, Vec2::new(20.0, 0.0), 0.0),
            Bone::new("goal", None, 5.0, Vec2::new(50.0 + 90.0, 100.0), 0.0),
        ];
        let mut c = IkChain::chain("tentacle", &["a", "b", "c"], "goal");
        c.stretch = 2.0;
        p.ik = vec![c];
        let e = Engine::new(p).unwrap();
        let f = e.evaluate(0.0);
        let tip = f.world[2].apply(Vec2::new(20.0, 0.0));
        assert!(tip.distance(Vec2::new(140.0, 100.0)) < 2.0, "{tip:?}");
    }
}
