// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `machine`.
#![allow(unused_imports)]

use super::*;
use crate::binding::{Binding, Remap, Target};
use crate::clipstack::Clip;
use crate::motionlib::{Take, TakeTrack};
use crate::anim::Key;

fn k(f: f32, v: f32) -> Key {
    Key { frame: f, value: v, ease: Ease::Linear }
}

fn take(name: &str, len: f32, target: &str, prop: Prop, a: f32, b: f32) -> Take {
    Take { name: name.into(), length: len, tracks: vec![TakeTrack { target: target.into(), prop, keys: vec![k(0.0, a), k(len, b)] }] }
}

/// 24 fps project with a bone "arm", layers "body" and "eye", clips idle (angle 0), walk
/// (angle 10..20, 24 frames), run (angle 40, 24 frames), wave (once, 12 frames, angle 90).
fn project() -> Project {
    let mut p = Project::new("m", 100.0, 100.0, 24.0, 48);
    p.bones.push(crate::project::Bone::new("arm", None, 10.0, crate::math::Vec2::new(0.0, 0.0), 0.0));
    p.bones.push(crate::project::Bone::new("leg", None, 10.0, crate::math::Vec2::new(0.0, 0.0), 0.0));
    let mk = |n: &str, t: Take| Clip::new(n, t);
    p.clips = vec![
        mk("idle", take("idle", 24.0, "arm", Prop::BoneAngle, 0.0, 0.0)),
        mk("walk", take("walk", 24.0, "arm", Prop::BoneAngle, 10.0, 20.0)),
        mk("run", take("run", 24.0, "arm", Prop::BoneAngle, 40.0, 40.0)),
        mk("wave", take("wave", 12.0, "arm", Prop::BoneAngle, 90.0, 90.0)),
        mk("legsit", take("legsit", 24.0, "leg", Prop::BoneAngle, 5.0, 5.0)),
    ];
    p
}

fn st(name: &str, clip: &str) -> State {
    State { name: name.into(), source: StateSource::Clip { clip: clip.into() }, mode: PlayMode::Loop, speed: 1.0, blend_in: 0.0 }
}
fn tr(from: &str, to: &str, conds: Vec<Condition>) -> Transition {
    Transition { from: from.into(), to: to.into(), conditions: conds, exit_time: None, duration: Some(0.0), ease: Ease::Linear }
}
fn cond(input: &str, op: Cmp, value: f64) -> Condition {
    Condition { input: input.into(), op, value }
}
fn inp(name: &str, kind: InputKind) -> Input {
    Input { name: name.into(), kind, default: 0.0, hold: false }
}
fn layer(name: &str, entry: &str, states: Vec<State>, trans: Vec<Transition>) -> Layer {
    Layer { name: name.into(), mask: vec![], entry: entry.into(), states, transitions: trans }
}

fn base_machine() -> Machine {
    Machine {
        name: "m".into(),
        inputs: vec![inp("moving", InputKind::Bool), inp("speed", InputKind::Number), inp("wave", InputKind::Trigger)],
        layers: vec![layer(
            "body",
            "idle",
            vec![st("idle", "idle"), st("walk", "walk"), State { mode: PlayMode::Once, ..st("wave", "wave") }],
            vec![
                tr("idle", "walk", vec![cond("moving", Cmp::Eq, 1.0)]),
                tr("walk", "idle", vec![cond("moving", Cmp::Eq, 0.0)]),
                tr("*", "wave", vec![cond("wave", Cmp::Eq, 1.0)]),
                Transition { exit_time: Some(1.0), ..tr("wave", "idle", vec![]) },
            ],
        )],
    }
}

#[test]
fn unknown_keys_in_a_transition_are_an_error_not_an_always_firing_transition() {
    // `conditions` is not the key (it is `when`): this used to load as an unconditional transition
    let good = r#"{"from":"idle","to":"walk","when":[{"input":"moving"}]}"#;
    assert!(serde_json::from_str::<Transition>(good).is_ok());
    let typo = r#"{"from":"idle","to":"walk","conditions":[{"input":"moving"}]}"#;
    let err = serde_json::from_str::<Transition>(typo).unwrap_err().to_string();
    assert!(err.contains("unknown field `conditions`"), "{err}");
    let bad_cond = r#"{"from":"idle","to":"walk","when":[{"input":"moving","vaule":1}]}"#;
    assert!(serde_json::from_str::<Transition>(bad_cond).unwrap_err().to_string().contains("vaule"));
    // and through a whole project file
    let mut p = project();
    p.machines.push(base_machine());
    let text = p.to_json_pretty().replace("\"when\"", "\"conditions\"");
    let err = Project::from_json(&text).expect_err("must not load").to_string();
    assert!(err.contains("conditions"), "{err}");
}

fn rt_of(m: Machine, mut p: Project) -> MachineRuntime {
    p.machines.push(m);
    MachineRuntime::new(&p, "m").unwrap()
}

fn angle(rt: &MachineRuntime) -> f32 {
    rt.pose().get("arm", Prop::BoneAngle).map(|x| x.0).unwrap_or(f32::NAN)
}

#[test]
fn transitions_by_inputs() {
    let mut rt = rt_of(base_machine(), project());
    assert_eq!(rt.state_of("body"), Some("idle"));
    rt.step(0.1).unwrap();
    assert_eq!(rt.state_of("body"), Some("idle"));
    rt.set_bool("moving", true).unwrap();
    rt.step(0.05).unwrap();
    assert_eq!(rt.state_of("body"), Some("walk"));
    rt.step(0.25).unwrap(); // entered after the first 1/60 s substep, so 0.2833 s into the 1 s clip
    assert!((angle(&rt) - 12.8333).abs() < 0.01, "{}", angle(&rt));
    rt.set_bool("moving", false).unwrap();
    rt.step(0.02).unwrap();
    assert_eq!(rt.state_of("body"), Some("idle"));
    let ev = rt.take_events();
    let kinds: Vec<String> = ev.iter().map(|e| format!("{:?}:{}", e.kind, e.state)).collect();
    assert_eq!(kinds, ["Entered:idle", "Exited:idle", "Entered:walk", "Exited:walk", "Entered:idle"]);
    assert!(rt.take_events().is_empty());
}

#[test]
fn trigger_is_consumed_once_and_expires() {
    let mut rt = rt_of(base_machine(), project());
    rt.fire("wave").unwrap();
    rt.step(0.05).unwrap();
    assert_eq!(rt.state_of("body"), Some("wave"));
    assert_eq!(rt.input_value("wave"), Some(0.0));
    // finishing the once clip (0.5 s) goes back to idle and the consumed trigger does not refire
    rt.step(0.7).unwrap();
    assert_eq!(rt.state_of("body"), Some("idle"));
    rt.step(0.5).unwrap();
    assert_eq!(rt.state_of("body"), Some("idle"));
    // a trigger nobody tests lasts one step only
    let mut m = base_machine();
    m.layers[0].transitions[2].conditions = vec![cond("moving", Cmp::Eq, 1.0)];
    let mut rt = rt_of(m, project());
    rt.fire("wave").unwrap();
    assert_eq!(rt.input_value("wave"), Some(1.0));
    rt.step(0.1).unwrap();
    assert_eq!(rt.input_value("wave"), Some(0.0));
    // hold keeps it
    let mut m = base_machine();
    m.inputs[2].hold = true;
    m.layers[0].transitions[2].conditions = vec![cond("moving", Cmp::Eq, 1.0)];
    let mut rt = rt_of(m, project());
    rt.fire("wave").unwrap();
    rt.step(0.1).unwrap();
    assert_eq!(rt.input_value("wave"), Some(1.0));
    assert!(rt.fire("moving").is_err());
    assert!(rt.fire("nope").is_err());
}

#[test]
fn any_state_and_priority() {
    let mut rt = rt_of(base_machine(), project());
    rt.set_bool("moving", true).unwrap();
    rt.step(0.05).unwrap();
    assert_eq!(rt.state_of("body"), Some("walk"));
    rt.fire("wave").unwrap();
    rt.step(0.05).unwrap();
    // from walk via "*"
    assert_eq!(rt.state_of("body"), Some("wave"));
    // exit time 1.0: wave plays 0.5 s, then idle -> walk chain happens next step since moving is still true
    rt.step(0.6).unwrap();
    assert!(matches!(rt.state_of("body"), Some("idle") | Some("walk")));
    rt.step(0.05).unwrap();
    assert_eq!(rt.state_of("body"), Some("walk"));
    // an any-state transition never targets the current state: no restart
    rt.fire("wave").unwrap();
    rt.step(0.05).unwrap();
    let pos_wave = rt.layers[0].cur.pos;
    rt.fire("wave").unwrap();
    rt.step(0.05).unwrap();
    assert!(rt.layers[0].cur.pos > pos_wave);
}

#[test]
fn exit_time_waits_for_the_fraction() {
    let mut m = base_machine();
    m.layers[0].states.truncate(2);
    m.layers[0].entry = "walk".into();
    m.layers[0].transitions = vec![Transition { exit_time: Some(0.5), ..tr("walk", "idle", vec![]) }];
    let mut rt = rt_of(m, project());
    rt.step(0.4).unwrap(); // 0.4 s of a 1 s clip
    assert_eq!(rt.state_of("body"), Some("walk"));
    rt.step(0.15).unwrap(); // past 0.5
    assert_eq!(rt.state_of("body"), Some("idle"));
    // exit time combined with a condition needs both
    let mut m = base_machine();
    m.layers[0].states.truncate(2);
    m.layers[0].transitions = vec![Transition { exit_time: Some(0.5), ..tr("idle", "walk", vec![cond("moving", Cmp::Eq, 1.0)]) }];
    let mut rt = rt_of(m, project());
    rt.set_bool("moving", true).unwrap();
    rt.step(0.3).unwrap();
    assert_eq!(rt.state_of("body"), Some("idle"));
    rt.step(0.3).unwrap();
    assert_eq!(rt.state_of("body"), Some("walk"));
}

#[test]
fn crossfade_blends_and_finishes() {
    let mut m = base_machine();
    m.layers[0].transitions[0].duration = Some(1.0);
    m.layers[0].transitions[0].ease = Ease::Linear;
    let mut rt = rt_of(m, project());
    rt.set_bool("moving", true).unwrap();
    rt.step(0.0).unwrap();
    assert!(rt.is_fading("body"));
    rt.step(0.5).unwrap();
    // idle = 0, walk at 0.5 s = 15; halfway through the fade -> 7.5
    assert!((angle(&rt) - 7.5).abs() < 0.3, "{}", angle(&rt));
    rt.step(0.6).unwrap();
    assert!(!rt.is_fading("body"));
    // interrupt a fade: a snapshot is taken and fades toward the new state
    let mut m = base_machine();
    m.layers[0].transitions[0].duration = Some(1.0);
    m.layers[0].transitions[1].duration = Some(1.0);
    let mut rt = rt_of(m, project());
    rt.set_bool("moving", true).unwrap();
    rt.step(0.5).unwrap();
    let before = angle(&rt);
    rt.set_bool("moving", false).unwrap();
    rt.step(0.0).unwrap();
    assert!((angle(&rt) - before).abs() < 0.3, "no jump at the interrupt: {} vs {}", angle(&rt), before);
    rt.step(1.1).unwrap();
    assert!(angle(&rt).abs() < 1e-3);
}

#[test]
fn blend_tree_weights() {
    assert_eq!(blend_weights(&[0.0, 1.0, 2.0], -1.0), vec![1.0, 0.0, 0.0]);
    assert_eq!(blend_weights(&[0.0, 1.0, 2.0], 3.0), vec![0.0, 0.0, 1.0]);
    let w = blend_weights(&[0.0, 1.0, 2.0], 1.25);
    assert_eq!(w, vec![0.0, 0.75, 0.25]);
    let mut p = project();
    let blend = State {
        name: "loco".into(),
        source: StateSource::Blend {
            input: "speed".into(),
            children: vec![
                BlendChild { threshold: 0.0, source: StateSource::Clip { clip: "idle".into() } },
                BlendChild { threshold: 1.0, source: StateSource::Clip { clip: "walk".into() } },
                BlendChild { threshold: 2.0, source: StateSource::Clip { clip: "run".into() } },
            ],
        },
        mode: PlayMode::Loop,
        speed: 1.0,
        blend_in: 0.0,
    };
    let m = Machine { name: "m".into(), inputs: vec![inp("speed", InputKind::Number)], layers: vec![layer("body", "loco", vec![blend], vec![])] };
    p.machines.push(m);
    let mut rt = MachineRuntime::new(&p, "m").unwrap();
    rt.set_number("speed", 0.0).unwrap();
    assert!((angle(&rt) - 0.0).abs() < 1e-4);
    rt.set_number("speed", 1.0).unwrap();
    assert!((angle(&rt) - 10.0).abs() < 1e-4); // walk at phase 0
    rt.set_number("speed", 1.5).unwrap();
    assert!((angle(&rt) - 25.0).abs() < 1e-4); // mean of walk (10) and run (40)
    rt.set_number("speed", 0.5).unwrap();
    assert!((angle(&rt) - 5.0).abs() < 1e-4);
    rt.set_number("speed", 9.0).unwrap();
    assert!((angle(&rt) - 40.0).abs() < 1e-4);
    // all clips are 24 frames, so the state keeps its 1 s period
    rt.step(0.5).unwrap();
    assert!((angle(&rt) - 40.0).abs() < 1e-4);
    rt.set_number("speed", 1.0).unwrap();
    assert!((angle(&rt) - 15.0).abs() < 0.3, "{}", angle(&rt));
    assert!(rt.set_number("nope", 1.0).is_err());
    assert!(rt.set_number("speed", f64::NAN).is_err());
}

#[test]
fn blend_tree_clip_weights_use_stack_maths() {
    // a half-weight clip pulls toward the project value only by its coverage
    let mut p = project();
    p.clips[2].weight = 0.5; // run at half coverage
    let blend = State {
        name: "loco".into(),
        source: StateSource::Blend {
            input: "speed".into(),
            children: vec![
                BlendChild { threshold: 0.0, source: StateSource::Clip { clip: "walk".into() } },
                BlendChild { threshold: 1.0, source: StateSource::Clip { clip: "run".into() } },
            ],
        },
        mode: PlayMode::Loop,
        speed: 1.0,
        blend_in: 0.0,
    };
    p.machines.push(Machine { name: "m".into(), inputs: vec![inp("speed", InputKind::Number)], layers: vec![layer("b", "loco", vec![blend], vec![])] });
    let mut rt = MachineRuntime::new(&p, "m").unwrap();
    rt.set_number("speed", 1.0).unwrap();
    let (v, w) = rt.pose().get("arm", Prop::BoneAngle).unwrap();
    assert!((v - 40.0).abs() < 1e-4 && (w - 0.5).abs() < 1e-6);
    rt.set_number("speed", 0.5).unwrap();
    let (v, w) = rt.pose().get("arm", Prop::BoneAngle).unwrap();
    // normalised average of walk(10, cov 1)*0.5 and run(40, cov .5)*0.5
    assert!((v - (0.5 * 10.0 + 0.25 * 40.0) / 0.75).abs() < 1e-4 && (w - 0.75).abs() < 1e-6);
}

#[test]
fn ping_pong_and_speed() {
    let mut p = project();
    let s = State { mode: PlayMode::PingPong, speed: 2.0, ..st("walk", "walk") };
    p.machines.push(Machine { name: "m".into(), inputs: vec![], layers: vec![layer("b", "walk", vec![s], vec![])] });
    let mut rt = MachineRuntime::new(&p, "m").unwrap();
    rt.step(0.25).unwrap(); // 0.5 loops forward
    assert!((angle(&rt) - 15.0).abs() < 0.2);
    rt.step(0.5).unwrap(); // pos 1.5 -> mirrored back to 0.5
    assert!((angle(&rt) - 15.0).abs() < 0.2);
    rt.step(0.25).unwrap(); // pos 2.0 -> start
    assert!((angle(&rt) - 10.0).abs() < 0.2);
}

#[test]
fn parallel_layers_on_disjoint_groups() {
    let mut p = project();
    let mut m = base_machine();
    m.layers[0].mask = vec!["arm".into()];
    m.layers.push(layer("legs", "sit", vec![st("sit", "legsit")], vec![]));
    m.layers[1].mask = vec!["leg".into()];
    p.machines.push(m);
    assert!(validate(&p).is_empty(), "{:?}", validate(&p));
    let mut rt = MachineRuntime::new(&p, "m").unwrap();
    rt.set_bool("moving", true).unwrap();
    rt.step(0.3).unwrap();
    assert_eq!(rt.state_of("body"), Some("walk"));
    assert_eq!(rt.state_of("legs"), Some("sit"));
    let pose = rt.pose();
    assert!(pose.get("arm", Prop::BoneAngle).is_some());
    assert_eq!(pose.get("leg", Prop::BoneAngle).unwrap().0, 5.0);
    // overlapping layers are rejected
    let mut q = p.clone();
    q.machines[0].layers[1].mask.clear();
    q.machines[0].layers[1].states[0].source = StateSource::Clip { clip: "walk".into() };
    let e = validate(&q).join("\n");
    assert!(e.contains("disjoint channel groups"), "{e}");
}

#[test]
fn validation_errors() {
    let base = |m: Machine| {
        let mut p = project();
        p.machines.push(m);
        validate(&p).join("\n")
    };
    assert_eq!(base(base_machine()), "");
    // unknown clip
    let mut m = base_machine();
    m.layers[0].states[0] = st("idle", "ghost");
    assert!(base(m).contains("unknown clip 'ghost'"));
    // duplicate names
    let mut m = base_machine();
    m.layers[0].states.push(st("idle", "idle"));
    m.inputs.push(inp("moving", InputKind::Bool));
    let e = base(m);
    assert!(e.contains("duplicate state name 'idle'") && e.contains("duplicate input 'moving'"), "{e}");
    // missing target / source state, unknown input
    let mut m = base_machine();
    m.layers[0].transitions.push(tr("idle", "nowhere", vec![]));
    m.layers[0].transitions.push(tr("zip", "idle", vec![]));
    m.layers[0].transitions.push(tr("idle", "walk", vec![cond("ghost", Cmp::Eq, 1.0)]));
    let e = base(m);
    assert!(e.contains("transition to missing state 'nowhere'") && e.contains("unknown source state 'zip'") && e.contains("unknown input 'ghost'"), "{e}");
    // unreachable state
    let mut m = base_machine();
    m.layers[0].states.push(st("lonely", "run"));
    assert!(base(m).contains("state 'lonely' is unreachable"));
    // zero-duration unconditional cycle
    let mut m = base_machine();
    m.layers[0].transitions.push(tr("walk", "wave", vec![]));
    m.layers[0].transitions.push(tr("wave", "walk", vec![]));
    assert!(base(m).contains("form a cycle"));
    // but a cycle that waits for an exit time is fine, and so is one with conditions
    let mut m = base_machine();
    m.layers[0].transitions.push(Transition { exit_time: Some(1.0), ..tr("wave", "walk", vec![]) });
    assert_eq!(base(m), "");
    // bad entry, bad numbers, bad blend
    let mut m = base_machine();
    m.layers[0].entry = "zzz".into();
    m.layers[0].states[0].speed = 0.0;
    m.layers[0].transitions[0].exit_time = Some(2.0);
    m.layers[0].transitions[1].duration = Some(-1.0);
    let e = base(m);
    assert!(e.contains("entry state 'zzz'") && e.contains("speed must") && e.contains("exit_time") && e.contains("duration must"), "{e}");
    let mut m = base_machine();
    m.layers[0].states[0].source = StateSource::Blend {
        input: "moving".into(),
        children: vec![BlendChild { threshold: 1.0, source: StateSource::Clip { clip: "idle".into() } }, BlendChild { threshold: 0.0, source: StateSource::Clip { clip: "idle".into() } }],
    };
    let e = base(m);
    assert!(e.contains("must be a number input") && e.contains("strictly increasing"), "{e}");
    // segments outside the project, regions that are not regions
    let mut m = base_machine();
    m.layers[0].states[0].source = StateSource::Segment { start: 10.0, end: 999.0 };
    assert!(base(m).contains("beyond"));
    let mut p = project();
    p.markers.push(crate::keyops::Marker { name: "pt".into(), frame: 3.0, end: None });
    let mut m = base_machine();
    m.layers[0].states[0].source = StateSource::Region { marker: "pt".into() };
    p.machines.push(m);
    assert!(validate(&p).join("\n").contains("not a region"));
    // a machine named like another
    let mut p = project();
    p.machines.push(base_machine());
    p.machines.push(base_machine());
    assert!(validate(&p).join("\n").contains("duplicate machine name"));
}

#[test]
fn engine_new_rejects_bad_machines_and_unknown_targets() {
    let mut p = project();
    p.machines.push(base_machine());
    assert!(Engine::new(p.clone()).is_ok());
    let mut bad = p.clone();
    bad.machines[0].layers[0].states[0] = st("idle", "ghost");
    let e = Engine::new(bad).err().unwrap().join("\n");
    assert!(e.contains("unknown clip 'ghost'"), "{e}");
    // a clip that animates a bone the project does not have is caught by the trial compile
    let mut bad = p.clone();
    bad.clips[1].take.tracks[0].target = "no_such_bone".into();
    let e = Engine::new(bad).err().unwrap().join("\n");
    assert!(e.contains("machine/binding targets"), "{e}");
    // JSON round trip and omission when empty
    let json = p.to_json_pretty();
    let back = Project::from_json(&json).unwrap();
    assert_eq!(back.machines, p.machines);
    assert!(!project().to_json_pretty().contains("machines"));
    assert!(!project().to_json_pretty().contains("bindings"));
}

#[test]
fn frame_is_rendered_from_the_pose() {
    let mut p = project();
    p.machines.push(base_machine());
    let mut rt = MachineRuntime::new(&p, "m").unwrap();
    rt.set_bool("moving", true).unwrap();
    rt.step(0.25).unwrap();
    let f = rt.frame().unwrap();
    // bone "arm" is rotated 12.5 degrees: world matrix rotation
    let rot = f.world[0].rotation().to_degrees();
    assert!((rot - 12.5).abs() < 0.3, "{rot}");
}

#[test]
fn segments_and_regions_play_project_channels() {
    let mut p = project();
    p.channels.push(Channel::keys("arm", Prop::BoneAngle, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (24.0, 48.0, Ease::Linear)])));
    p.markers.push(crate::keyops::Marker { name: "second".into(), frame: 12.0, end: Some(24.0) });
    let s1 = State { source: StateSource::Segment { start: 0.0, end: 24.0 }, ..st("seg", "") };
    let s2 = State { source: StateSource::Region { marker: "second".into() }, ..st("reg", "") };
    p.machines.push(Machine {
        name: "m".into(),
        inputs: vec![inp("go", InputKind::Trigger)],
        layers: vec![layer("a", "seg", vec![s1, s2], vec![tr("seg", "reg", vec![cond("go", Cmp::Eq, 1.0)])])],
    });
    let mut rt = MachineRuntime::new(&p, "m").unwrap();
    rt.step(0.5).unwrap();
    assert!((angle(&rt) - 24.0).abs() < 0.2);
    rt.fire("go").unwrap();
    rt.step(0.0).unwrap();
    assert_eq!(rt.state_of("a"), Some("reg"));
    assert!((angle(&rt) - 24.0).abs() < 0.2); // frame 12 of the project curve
    rt.step(0.25).unwrap();
    assert!((angle(&rt) - 36.0).abs() < 0.3);
}

#[test]
fn deterministic_and_chunk_independent() {
    let run = |chunks: &[f64]| {
        let mut rt = rt_of(base_machine(), project());
        rt.set_bool("moving", true).unwrap();
        let mut out = vec![];
        for &c in chunks {
            rt.step(c).unwrap();
            out.push((rt.state_of("body").unwrap().to_string(), angle(&rt)));
        }
        (rt.time(), out.last().unwrap().clone(), rt.take_events().len())
    };
    let a = run(&[0.1, 0.2, 0.3, 0.4]);
    let b = run(&[0.1, 0.2, 0.3, 0.4]);
    assert_eq!(a, b);
    let c = run(&[0.5, 0.5]); // same total time, other chunking
    assert!((a.0 - c.0).abs() < 1e-9 && a.1 .0 == c.1 .0 && (a.1 .1 - c.1 .1).abs() < 0.3);
    let mut rt = rt_of(base_machine(), project());
    assert!(rt.step(-1.0).is_err() && rt.step(f64::NAN).is_err() && rt.step(11.0).is_err());
}

#[test]
fn script_simulation_logs_states() {
    let mut p = project();
    p.machines.push(base_machine());
    let script = Script {
        machine: None,
        duration: Some(2.0),
        events: vec![
            ScriptEvent { t: 0.5, set: [("moving".to_string(), serde_json::json!(true))].into_iter().collect(), ..Default::default() },
            ScriptEvent { t: 1.0, trigger: vec!["wave".into()], ..Default::default() },
        ],
    };
    let mut seen = 0;
    let log = simulate(&p, &script, 24.0, std::path::Path::new("."), |_, _| {
        seen += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(seen, 49);
    assert_eq!(log.ticks.len(), 49);
    assert_eq!(log.ticks[0].states[0].1, "idle");
    assert_eq!(log.ticks[13].states[0].1, "walk"); // t = 13/24 > 0.5
    assert_eq!(log.ticks[25].states[0].1, "wave");
    assert!(log.events.iter().any(|e| e.state == "wave" && e.kind == EventKind::Entered));
    // causality: no state is left before the input that causes it was given
    let left_idle = log.events.iter().find(|e| e.state == "idle" && e.kind == EventKind::Exited).unwrap();
    assert!(left_idle.time >= 0.5 - 1e-9, "left idle at {} before the input at 0.5", left_idle.time);
    let entered_wave = log.events.iter().find(|e| e.state == "wave" && e.kind == EventKind::Entered).unwrap();
    assert!(entered_wave.time >= 1.0 - 1e-9, "entered wave at {} before the trigger at 1.0", entered_wave.time);
    let log2 = simulate(&p, &script, 24.0, std::path::Path::new("."), |_, _| Ok(())).unwrap();
    assert_eq!(serde_json::to_string(&log).unwrap(), serde_json::to_string(&log2).unwrap());
    // errors
    let bad = Script { events: vec![ScriptEvent { t: 0.0, set: [("ghost".to_string(), serde_json::json!(1))].into_iter().collect(), ..Default::default() }], ..Default::default() };
    assert!(simulate(&p, &bad, 24.0, std::path::Path::new("."), |_, _| Ok(())).is_err());
    assert!(simulate(&project(), &script, 24.0, std::path::Path::new("."), |_, _| Ok(())).is_err());
}

#[test]
fn bindings_drive_properties_and_inputs() {
    let mut p = project();
    p.machines.push(base_machine());
    let mut hp = Binding::new("hp", "player.health", Target::Prop { target: "leg".into(), prop: Prop::BoneAngle });
    hp.remap = Some(Remap { in_min: 0.0, in_max: 100.0, out_min: -30.0, out_max: 30.0, clamp: true, curve: Ease::Linear });
    let mv = Binding::new("mv", "player.moving", Target::Input { machine: "m".into(), input: "moving".into() });
    let mut spd = Binding::new("spd", "player.fire", Target::Input { machine: "m".into(), input: "wave".into() });
    spd.expr = Some("value * 2".into());
    p.bindings = vec![hp, mv, spd];
    assert!(Engine::new(p.clone()).is_ok());
    let mut rt = MachineRuntime::new(&p, "m").unwrap();
    rt.set_value("player.health", 75.0).unwrap();
    rt.set_value("player.moving", 1.0).unwrap();
    rt.step(0.1).unwrap();
    assert_eq!(rt.state_of("body"), Some("walk"));
    let ch = rt.pose_channels();
    let leg = ch.iter().find(|c| c.target == "leg").unwrap();
    match &leg.source {
        Source::Keys { track } => assert!((track.eval(0.0).unwrap() - 15.0).abs() < 1e-4),
        _ => panic!(),
    }
    // rendered frame has the bound property
    let f = rt.frame().unwrap();
    assert!((f.world[1].rotation().to_degrees() - 15.0).abs() < 0.3);
    // trigger binding fires on the rising edge only
    rt.set_value("player.fire", 0.0).unwrap();
    rt.step(0.05).unwrap();
    assert_eq!(rt.state_of("body"), Some("walk"));
    rt.set_value("player.fire", 1.0).unwrap();
    rt.step(0.05).unwrap();
    assert_eq!(rt.state_of("body"), Some("wave"));
    // a binding to an unknown bone is rejected when the engine is built
    let mut bad = p.clone();
    bad.bindings[0].target = Target::Prop { target: "ghost".into(), prop: Prop::BoneAngle };
    assert!(Engine::new(bad).err().unwrap().join("\n").contains("machine/binding targets"));
    // editing helpers
    let mut q = project();
    assert!(add_binding(&mut q, Binding::new("x", "a.b", Target::Prop { target: "ghost".into(), prop: Prop::BoneAngle })).is_err());
    assert!(add_binding(&mut q, Binding::new("x", "a.b", Target::Prop { target: "arm".into(), prop: Prop::BoneAngle })).is_ok());
    assert!(add_binding(&mut q, Binding::new("x", "a.c", Target::Prop { target: "arm".into(), prop: Prop::BoneAngle })).is_err());
    remove_binding(&mut q, "x").unwrap();
    assert!(remove_binding(&mut q, "x").is_err());
    add(&mut q, base_machine()).unwrap();
    assert!(add(&mut q, base_machine()).is_err());
    remove(&mut q, "m").unwrap();
    assert!(remove(&mut q, "m").is_err());
}
