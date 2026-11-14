// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Compiled as an external crate: proves the prelude exports exactly what the docs promise and
//! that an outside user can drive the whole facade without touching any unstable module.
use osatura_core::prelude::{
    Animation, AnimationInfo, ApiError, ExportFormat, Image, Interpolation, Player, RenderSettings, API_VERSION, API_VERSION_MAJOR,
};

#[test]
fn prelude_names_exist_and_work() {
    let mut a: Animation = Animation::demo();
    let _: AnimationInfo = a.info();
    let img: Image = a.render_rgba(0.0, &RenderSettings::new(0.25).with_bones(true).with_transparent(true)).unwrap();
    assert!(img.width > 0);
    let bone = a.bone_names()[0].clone();
    a.set_key(&bone, "bone_angle", 2.0, 5.0, Interpolation::Linear).unwrap();
    assert!(!a.export(ExportFormat::Html, 1.0).unwrap().is_empty());
    let e: Box<dyn std::error::Error> = Box::new(ApiError::Export("x".into()));
    assert!(e.to_string().contains("export failed"));
    assert_eq!(API_VERSION.split('.').next().unwrap(), API_VERSION_MAJOR.to_string());
    // non_exhaustive enums force a wildcard arm for downstream code
    match ApiError::Internal("x".into()) {
        ApiError::Internal(_) => {}
        _ => unreachable!(),
    }
}

#[test]
fn bad_input_never_panics() {
    for s in ["", "null", "{}", "[]", "{\"name\":1}", "\u{0}", "{\"format\":4294967296}", &"[".repeat(100000)] {
        assert!(Animation::from_json(s).is_err());
    }
}

#[test]
fn player_is_reachable_from_the_prelude() {
    let a = Animation::demo();
    assert!(a.machine_names().is_empty());
    let r: Result<Player, ApiError> = a.player("none");
    assert!(r.is_err());
}

#[test]
fn tools_are_reachable_from_the_prelude() {
    use osatura_core::prelude::{ci_run, trace_image_svg, vcs_log, CiOutcome, FitFormat, FuzzSummary, LodLevel, MocapReport, MocapSettings, SizeFit, VcsCommit};
    let a = Animation::demo();
    let lv: Vec<LodLevel> = a.lod_levels(1.0, 1.0, 2).unwrap();
    assert_eq!(lv.len(), 3);
    let _: FuzzSummary = a.fuzz(5, 1).unwrap();
    let _ = (FitFormat::Lottie, MocapSettings::new(), None::<SizeFit>, None::<MocapReport>, None::<VcsCommit>, None::<CiOutcome>);
    assert!(trace_image_svg(b"x", 4).is_err() && vcs_log("/nonexistent-osatura", 3).is_err() && ci_run("/nonexistent-osatura.json").is_err());
}

#[test]
fn api_version_is_bumped_for_the_additions() {
    let a = Animation::demo();
    assert!(a.player("none").is_err()); // Player methods (canvas, scroll, LOD) are exercised in api::tests
    assert_eq!(osatura_core::prelude::API_VERSION, "1.4.0");
}

#[test]
fn vcs_and_ci_write_ops_are_reachable_from_the_prelude() {
    use osatura_core::prelude::{ci_bless, ci_init, ci_run_mode, vcs_abort, vcs_branch, vcs_checkout, vcs_cherry_pick, vcs_commit, vcs_conflicts, vcs_init, vcs_merge, vcs_resolve, vcs_revert, vcs_tag, CiInit, CiMode, VcsChoice, VcsConflict, VcsMergeOutcome};
    let _ = (CiMode::Verify, VcsChoice::Ours, None::<CiInit>, None::<VcsConflict>, None::<VcsMergeOutcome>);
    let n = "/nonexistent-osatura";
    assert!(vcs_init(n, None).is_err() && vcs_commit(n, Some("m"), None).is_err() && vcs_branch(n, "b", None).is_err());
    assert!(vcs_checkout(n, "b", false).is_err() && vcs_merge(n, "b", false, None, None).is_err() && vcs_tag(n, "t", None).is_err());
    assert!(vcs_revert(n, "r", None).is_err() && vcs_cherry_pick(n, "r", None).is_err() && vcs_abort(n).is_err());
    assert!(vcs_resolve(n, "/p", &VcsChoice::Theirs).is_err() && vcs_conflicts(n).is_err());
    assert!(ci_init(n, None, false).is_err() && ci_bless(n).is_err() && ci_run_mode(n, CiMode::Update).is_err());
}
