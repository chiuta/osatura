// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `mocap`.

use super::*;
use crate::engine::Engine;
use crate::math::Vec2;
use crate::project::Bone;

// ---------------------------------------------------------------- synthetic fixtures

/// A stick figure joint: `role` identifies it for animation, `name` is what the file calls it.
struct J {
    name: String,
    role: &'static str,
    off: V3,
    kids: Vec<J>,
    end: Option<V3>,
}

fn j(name: &str, role: &'static str, off: V3, kids: Vec<J>, end: Option<V3>) -> J {
    J { name: name.into(), role, off, kids, end }
}

/// Names for the 20-joint figure (3 spine joints, clavicles, 4-joint legs).
struct Names {
    hips: &'static str,
    spine: [&'static str; 3],
    neck: &'static str,
    head: &'static str,
    clav: [&'static str; 2],
    uarm: [&'static str; 2],
    farm: [&'static str; 2],
    hand: [&'static str; 2],
    thigh: [&'static str; 2],
    shin: [&'static str; 2],
    foot: [&'static str; 2],
    toe: [&'static str; 2],
}

const MIXAMO: Names = Names {
    hips: "Hips",
    spine: ["Spine", "Spine1", "Spine2"],
    neck: "Neck",
    head: "Head",
    clav: ["LeftShoulder", "RightShoulder"],
    uarm: ["LeftArm", "RightArm"],
    farm: ["LeftForeArm", "RightForeArm"],
    hand: ["LeftHand", "RightHand"],
    thigh: ["LeftUpLeg", "RightUpLeg"],
    shin: ["LeftLeg", "RightLeg"],
    foot: ["LeftFoot", "RightFoot"],
    toe: ["LeftToeBase", "RightToeBase"],
};
const MIXAMO_NS: Names = Names {
    hips: "mixamorig:Hips",
    spine: ["mixamorig:Spine", "mixamorig:Spine1", "mixamorig:Spine2"],
    neck: "mixamorig:Neck",
    head: "mixamorig:Head",
    clav: ["mixamorig:LeftShoulder", "mixamorig:RightShoulder"],
    uarm: ["mixamorig:LeftArm", "mixamorig:RightArm"],
    farm: ["mixamorig:LeftForeArm", "mixamorig:RightForeArm"],
    hand: ["mixamorig:LeftHand", "mixamorig:RightHand"],
    thigh: ["mixamorig:LeftUpLeg", "mixamorig:RightUpLeg"],
    shin: ["mixamorig:LeftLeg", "mixamorig:RightLeg"],
    foot: ["mixamorig:LeftFoot", "mixamorig:RightFoot"],
    toe: ["mixamorig:LeftToeBase", "mixamorig:RightToeBase"],
};
const CMU: Names = Names {
    hips: "Hips",
    spine: ["lowerback", "upperback", "thorax"],
    neck: "lowerneck",
    head: "head",
    clav: ["lclavicle", "rclavicle"],
    uarm: ["lhumerus", "rhumerus"],
    farm: ["lradius", "rradius"],
    hand: ["lwrist", "rwrist"],
    thigh: ["lfemur", "rfemur"],
    shin: ["ltibia", "rtibia"],
    foot: ["lfoot", "rfoot"],
    toe: ["ltoes", "rtoes"],
};
const BIPED: Names = Names {
    hips: "Bip01 Pelvis",
    spine: ["Bip01 Spine", "Bip01 Spine1", "Bip01 Spine2"],
    neck: "Bip01 Neck",
    head: "Bip01 Head",
    clav: ["Bip01 L Clavicle", "Bip01 R Clavicle"],
    uarm: ["Bip01 L UpperArm", "Bip01 R UpperArm"],
    farm: ["Bip01 L Forearm", "Bip01 R Forearm"],
    hand: ["Bip01 L Hand", "Bip01 R Hand"],
    thigh: ["Bip01 L Thigh", "Bip01 R Thigh"],
    shin: ["Bip01 L Calf", "Bip01 R Calf"],
    foot: ["Bip01 L Foot", "Bip01 R Foot"],
    toe: ["Bip01 L Toe0", "Bip01 R Toe0"],
};
const ANON: Names = Names {
    hips: "j00",
    spine: ["j01", "j02", "j03"],
    neck: "j04",
    head: "j05",
    clav: ["j06", "j10"],
    uarm: ["j07", "j11"],
    farm: ["j08", "j12"],
    hand: ["j09", "j13"],
    thigh: ["j14", "j18"],
    shin: ["j15", "j19"],
    foot: ["j16", "j20"],
    toe: ["j17", "j21"],
};

/// T-pose figure facing +Z, y up, character-left = +X. Total leg 82, arm 50.
fn figure(n: &Names) -> J {
    let arm = |s: usize, sg: f64| {
        j(n.clav[s], if s == 0 { "clavL" } else { "clavR" }, [3.0 * sg, 8.0, 0.0], vec![
            j(n.uarm[s], if s == 0 { "uarmL" } else { "uarmR" }, [8.0 * sg, 0.0, 0.0], vec![
                j(n.farm[s], if s == 0 { "farmL" } else { "farmR" }, [26.0 * sg, 0.0, 0.0], vec![
                    j(n.hand[s], if s == 0 { "handL" } else { "handR" }, [24.0 * sg, 0.0, 0.0], vec![], Some([8.0 * sg, 0.0, 0.0])),
                ], None),
            ], None),
        ], None)
    };
    let leg = |s: usize, sg: f64| {
        j(n.thigh[s], if s == 0 { "thighL" } else { "thighR" }, [8.0 * sg, -4.0, 0.0], vec![
            j(n.shin[s], if s == 0 { "shinL" } else { "shinR" }, [0.0, -42.0, 0.0], vec![
                j(n.foot[s], if s == 0 { "footL" } else { "footR" }, [0.0, -40.0, 0.0], vec![
                    j(n.toe[s], if s == 0 { "toeL" } else { "toeR" }, [0.0, -6.0, 10.0], vec![], Some([0.0, 0.0, 6.0])),
                ], None),
            ], None),
        ], None)
    };
    j(n.hips, "hips", [0.0, 0.0, 0.0], vec![
        j(n.spine[0], "sp0", [0.0, 10.0, 0.0], vec![
            j(n.spine[1], "sp1", [0.0, 12.0, 0.0], vec![
                j(n.spine[2], "sp2", [0.0, 12.0, 0.0], vec![
                    j(n.neck, "neck", [0.0, 12.0, 0.0], vec![j(n.head, "head", [0.0, 8.0, 0.0], vec![], Some([0.0, 12.0, 0.0]))], None),
                    arm(0, 1.0),
                    arm(1, -1.0),
                ], None),
            ], None),
        ], None),
        leg(0, 1.0),
        leg(1, -1.0),
    ], None)
}

fn emit(n: &J, depth: usize, root: bool, out: &mut String, eol: &str) {
    let pad = "  ".repeat(depth);
    out.push_str(&format!("{pad}{} {}{eol}{pad}{{{eol}", if root { "ROOT" } else { "JOINT" }, n.name));
    out.push_str(&format!("{pad}  OFFSET {} {} {}{eol}", n.off[0], n.off[1], n.off[2]));
    if root {
        out.push_str(&format!("{pad}  CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation{eol}"));
    } else {
        out.push_str(&format!("{pad}  CHANNELS 3 Zrotation Xrotation Yrotation{eol}"));
    }
    for k in &n.kids {
        emit(k, depth + 1, false, out, eol);
    }
    if let Some(e) = n.end {
        out.push_str(&format!("{pad}  End Site{eol}{pad}  {{{eol}{pad}    OFFSET {} {} {}{eol}{pad}  }}{eol}", e[0], e[1], e[2]));
    }
    out.push_str(&format!("{pad}}}{eol}"));
}

fn row(n: &J, f: &dyn Fn(&str) -> [f64; 3], root_pos: V3, root: bool, out: &mut Vec<f64>) {
    if root {
        out.extend(root_pos);
    }
    out.extend(f(n.role));
    for k in &n.kids {
        row(k, f, root_pos, false, out);
    }
}

/// BVH text of `frames` frames of `dt` seconds; `rot(role, frame)` = (zrot, xrot, yrot) degrees,
/// `pos(frame)` = hips position.
fn make_bvh(n: &Names, frames: usize, dt: f64, rot: &dyn Fn(&str, usize) -> [f64; 3], pos: &dyn Fn(usize) -> V3, eol: &str) -> String {
    let fig = figure(n);
    let mut s = format!("HIERARCHY{eol}");
    emit(&fig, 0, true, &mut s, eol);
    s.push_str(&format!("MOTION{eol}Frames: {frames}{eol}Frame Time: {dt}{eol}"));
    for f in 0..frames {
        let mut r = Vec::new();
        row(&fig, &|role| rot(role, f), pos(f), true, &mut r);
        s.push_str(&r.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join(" "));
        s.push_str(eol);
    }
    s
}

fn still(_: &str, _: usize) -> [f64; 3] {
    [0.0; 3]
}
fn at_origin(_: usize) -> V3 {
    [0.0, 100.0, 0.0]
}

/// Names for the rig side.
struct RigNames {
    hips: &'static str,
    spine: [&'static str; 2],
    neck: &'static str,
    head: &'static str,
    clav: [&'static str; 2],
    uarm: [&'static str; 2],
    farm: [&'static str; 2],
    hand: [&'static str; 2],
    thigh: [&'static str; 2],
    shin: [&'static str; 2],
    foot: [&'static str; 2],
    toe: [&'static str; 2],
}
const BLENDER: RigNames = RigNames {
    hips: "pelvis",
    spine: ["spine", "chest"],
    neck: "neck",
    head: "head",
    clav: ["shoulder.L", "shoulder.R"],
    uarm: ["upper_arm.L", "upper_arm.R"],
    farm: ["forearm.L", "forearm.R"],
    hand: ["hand.L", "hand.R"],
    thigh: ["thigh.L", "thigh.R"],
    shin: ["shin.L", "shin.R"],
    foot: ["foot.L", "foot.R"],
    toe: ["toe.L", "toe.R"],
};
const RIG_ANON: RigNames = RigNames {
    hips: "b00",
    spine: ["b01", "b02"],
    neck: "b03",
    head: "b04",
    clav: ["b05", "b09"],
    uarm: ["b06", "b10"],
    farm: ["b07", "b11"],
    hand: ["b08", "b12"],
    thigh: ["b13", "b17"],
    shin: ["b14", "b18"],
    foot: ["b15", "b19"],
    toe: ["b16", "b20"],
};

/// Front-facing 2D rig (stage y down): character-left on +x. `k` scales all lengths.
fn rig(n: &RigNames, k: f32) -> Project {
    let mut p = Project::new("rig", 800.0, 600.0, 30.0, 40);
    let b = |name: &str, parent: Option<&str>, len: f32, off: (f32, f32), rest: f32| Bone::new(name, parent, len * k, Vec2::new(off.0 * k, off.1 * k), rest);
    p.bones.push(Bone::new(n.hips, None, 15.0 * k, Vec2::new(300.0, 300.0), -90.0));
    p.bones.push(b(n.spine[0], Some(n.hips), 30.0, (15.0, 0.0), 0.0));
    p.bones.push(b(n.spine[1], Some(n.spine[0]), 30.0, (30.0, 0.0), 0.0));
    p.bones.push(b(n.neck, Some(n.spine[1]), 10.0, (30.0, 0.0), 0.0));
    p.bones.push(b(n.head, Some(n.neck), 25.0, (10.0, 0.0), 0.0));
    for (s, sg) in [(0usize, 1.0f32), (1, -1.0)] {
        p.bones.push(b(n.clav[s], Some(n.spine[1]), 15.0, (25.0, 0.0), 90.0 * sg));
        let cl = p.bones.len() - 1;
        // left clavicle points to +x (world 0): local +90 under a -90 parent; right: world 180 -> local -90
        p.bones[cl].rest_angle = 90.0 * sg;
        p.bones.push(b(n.uarm[s], Some(n.clav[s]), 40.0, (15.0, 0.0), 0.0));
        p.bones.push(b(n.farm[s], Some(n.uarm[s]), 35.0, (40.0, 0.0), 0.0));
        p.bones.push(b(n.hand[s], Some(n.farm[s]), 15.0, (35.0, 0.0), 0.0));
    }
    for (s, sg) in [(0usize, 1.0f32), (1, -1.0)] {
        p.bones.push(b(n.thigh[s], Some(n.hips), 45.0, (0.0, 8.0 * sg), 180.0));
        p.bones.push(b(n.shin[s], Some(n.thigh[s]), 40.0, (45.0, 0.0), 0.0));
        p.bones.push(b(n.foot[s], Some(n.shin[s]), 8.0, (40.0, 0.0), 0.0));
        p.bones.push(b(n.toe[s], Some(n.foot[s]), 10.0, (8.0, 0.0), -90.0));
    }
    p
}

fn world_angle(e: &Engine, frame: f32, bone: usize) -> f64 {
    let w = e.evaluate(frame).world[bone];
    (w.b as f64).atan2(w.a as f64).to_degrees()
}

fn ang_diff(a: f64, b: f64) -> f64 {
    norm360(a - b).abs()
}

fn bone_idx(p: &Project, n: &str) -> usize {
    p.bones.iter().position(|b| b.name == n).unwrap_or_else(|| panic!("no bone {n}"))
}

fn pair<'a>(m: &'a MatchResult, bvh: &str, rig: &str) -> &'a BoneMatch {
    m.matches.iter().find(|x| x.bvh_name == bvh).unwrap_or_else(|| panic!("{bvh} unmatched\n{}", format_table(m))).clone_ref(rig)
}
trait CloneRef {
    fn clone_ref(&self, rig: &str) -> &Self;
}
impl CloneRef for BoneMatch {
    fn clone_ref(&self, rig: &str) -> &Self {
        assert_eq!(self.rig_name, rig, "{} matched to {} (slot {})", self.bvh_name, self.rig_name, self.slot);
        self
    }
}

// ---------------------------------------------------------------- parser

#[test]
fn parses_figure_and_reports_structure() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 3, 0.04, &still, &at_origin, "\n")).unwrap();
    assert_eq!(b.joints.len(), 22);
    assert_eq!(b.frames, 3);
    assert_eq!(b.num_channels, 6 + 21 * 3);
    assert!((b.fps() - 25.0).abs() < 1e-9);
    assert_eq!(b.joints[0].parent, None);
    assert_eq!(b.joints[b.joint_index("LeftHand").unwrap()].end_site, Some([8.0, 0.0, 0.0]));
    let s = b.summary();
    assert!(s.contains("22 joints") && s.contains("LeftForeArm") && s.contains("25.000 fps"), "{s}");
}

#[test]
fn tolerates_crlf_tabs_glued_braces_case_and_spaced_names() {
    let src = "HIERARCHY\r\nROOT Hips{\r\n\tOFFSET 0 0 0\r\n\tCHANNELS 3 Xposition Yposition Zposition\r\n\tJOINT Left Hip Joint\r\n\t{\r\n\t\tOFFSET 1 0 0\r\n\t\tCHANNELS 1 zrotation\r\n\t\tEnd Site\r\n\t\t{\r\n\t\t\tOFFSET 0 1 0\r\n\t\t}\r\n\t}\r\n}\r\nMOTION\r\nFrames:\t2\r\nFrame Time:\t0.5\r\n1 2 3 90\r\n\r\n4 5 6 0\r\n";
    let b = parse_bvh(src).unwrap();
    assert_eq!(b.joints[1].name, "Left Hip Joint");
    assert_eq!(b.frames, 2);
    assert_eq!(b.data, vec![1.0, 2.0, 3.0, 90.0, 4.0, 5.0, 6.0, 0.0]);
    // glued label forms and a BOM
    let src2 = format!("\u{feff}{}", src.replace("Frames:\t2", "Frames:2").replace("Frame Time:\t0.5", "Frame Time :0.5"));
    assert_eq!(parse_bvh(&src2).unwrap().frames, 2);
}

#[test]
fn rotation_order_follows_channel_order() {
    let mk = |ch: &str| {
        format!("HIERARCHY\nROOT A\n{{\nOFFSET 0 0 0\nCHANNELS 2 {ch}\nJOINT B\n{{\nOFFSET 1 0 0\nCHANNELS 0\nEnd Site\n{{\nOFFSET 1 0 0\n}}\n}}\n}}\nMOTION\nFrames: 1\nFrame Time: 1\n90 90\n")
    };
    // Rx * Ry applied to (1,0,0): Ry first -> (0,0,-1), then Rx(90) -> (0,1,0)
    let a = parse_bvh(&mk("Xrotation Yrotation")).unwrap().pose(Some(0));
    let q = a.pos[1];
    assert!((q[0]).abs() < 1e-12 && (q[1] - 1.0).abs() < 1e-12 && (q[2]).abs() < 1e-12, "{q:?}");
    // Ry * Rx: Rx leaves (1,0,0), Ry(90) sends it to (0,0,-1)
    let b = parse_bvh(&mk("Yrotation Xrotation")).unwrap().pose(Some(0));
    let q = b.pos[1];
    assert!((q[0]).abs() < 1e-12 && (q[1]).abs() < 1e-12 && (q[2] + 1.0).abs() < 1e-12, "{q:?}");
    // End Site follows the same transform: B is at (0,1,0), the end site adds Rx*Ry*(1,0,0)
    let e = a.end[1].unwrap();
    assert!((e[1] - 2.0).abs() < 1e-12, "{e:?}");
    // zrotation 90 sends +x to +y; position channels interleaved with rotations
    let c = parse_bvh("HIERARCHY\nROOT A\n{\nOFFSET 0 0 0\nCHANNELS 4 Zrotation Xposition Zrotation Yposition\nJOINT B\n{\nOFFSET 1 0 0\nCHANNELS 0\nEnd Site\n{\nOFFSET 1 0 0\n}\n}\n}\nMOTION\nFrames: 1\nFrame Time: 1\n90 2 -90 5\n").unwrap().pose(Some(0));
    // M = Rz(90) T(2,0,0) Rz(-90) T(0,5,0); child at M*(1,0,0) = Rz(90)(T(2,0,0)(Rz(-90)(1,5,0)))
    // Rz(-90)(1,5,0) = (5,-1,0); +2 -> (7,-1,0); Rz(90) -> (1,7,0)
    assert!((c.pos[1][0] - 1.0).abs() < 1e-12 && (c.pos[1][1] - 7.0).abs() < 1e-12, "{:?}", c.pos[1]);
}

#[test]
fn nested_hierarchy_and_multiple_roots() {
    let src = "HIERARCHY\nROOT A\n{\nOFFSET 0 0 0\nCHANNELS 0\nJOINT B\n{\nOFFSET 1 0 0\nCHANNELS 0\nJOINT C\n{\nOFFSET 1 0 0\nCHANNELS 0\nEnd Site\n{\nOFFSET 1 0 0\n}\n}\n}\n}\nROOT D\n{\nOFFSET 0 5 0\nCHANNELS 1 Zrotation\nEnd Site\n{\nOFFSET 0 1 0\n}\n}\nMOTION\nFrames: 1\nFrame Time: 0.1\n0\n";
    let b = parse_bvh(src).unwrap();
    assert_eq!(b.joints.len(), 4);
    assert_eq!(b.joints[2].parent, Some(1));
    assert_eq!(b.joints[3].parent, None);
    assert_eq!(b.pose(Some(0)).pos[2], [2.0, 0.0, 0.0]);
}

#[test]
fn errors_carry_line_numbers() {
    let good = make_bvh(&MIXAMO, 2, 0.04, &still, &at_origin, "\n");
    let lines: Vec<&str> = good.lines().collect();
    let last = lines.len();
    // bad channel name
    let bad = good.replacen("Zrotation", "Wrotation", 1);
    let e = parse_bvh(&bad).unwrap_err();
    assert!(e.starts_with("line 5:") && e.contains("Wrotation"), "{e}");
    // bad number in the data
    let bad = good.replacen("0 100 0", "0 abc 0", 1);
    let e = parse_bvh(&bad).unwrap_err();
    assert!(e.starts_with(&format!("line {}:", last - 1)) && e.contains("abc"), "{e}");
    // short data
    let mut l2 = lines.clone();
    l2.pop();
    let e = parse_bvh(&l2.join("\n")).unwrap_err();
    assert!(e.contains("expected") && e.starts_with(&format!("line {}:", last - 1)), "{e}");
    // extra data
    let e = parse_bvh(&format!("{good}1 2 3\n")).unwrap_err();
    assert!(e.contains("extra data") && e.starts_with(&format!("line {}:", last + 1)), "{e}");
    // missing closing brace -> end of file
    let e = parse_bvh("HIERARCHY\nROOT A\n{\nOFFSET 0 0 0\nCHANNELS 0\n").unwrap_err();
    assert!(e.starts_with("line 5:") && e.contains("end of file"), "{e}");
    // junk inside a joint
    let e = parse_bvh("HIERARCHY\nROOT A\n{\nOFFSET 0 0 0\nBANANA\n}\n").unwrap_err();
    assert!(e.starts_with("line 5:") && e.contains("BANANA"), "{e}");
    // missing OFFSET, bad frame time, no HIERARCHY, empty, zero channels count text
    assert!(parse_bvh("HIERARCHY\nROOT A\n{\nCHANNELS 0\n}\nMOTION\nFrames: 0\nFrame Time: 1\n").unwrap_err().contains("no OFFSET"));
    let e = parse_bvh(&good.replace("Frame Time: 0.04", "Frame Time: 0")).unwrap_err();
    assert!(e.contains("frame time must be positive"), "{e}");
    assert!(parse_bvh("MOTION\n").unwrap_err().starts_with("line 1:"));
    assert!(parse_bvh("   \n").is_err());
    assert!(parse_bvh("HIERARCHY\nROOT A\n{\nOFFSET 0 0 nan\n").unwrap_err().contains("finite"));
    assert!(parse_bvh("HIERARCHY\nROOT\n{\n").unwrap_err().contains("no name"));
    // absurd frame counts never allocate
    let e = parse_bvh(&good.replace("Frames: 2", "Frames: 99999999999")).unwrap_err();
    assert!(e.contains("exceeds"), "{e}");
}

#[test]
fn deep_nesting_is_refused_not_a_stack_overflow() {
    let mut s = String::from("HIERARCHY\nROOT A\n{\nOFFSET 0 0 0\nCHANNELS 0\n");
    for i in 0..700 {
        s.push_str(&format!("JOINT J{i}\n{{\nOFFSET 0 1 0\nCHANNELS 0\n"));
    }
    let e = parse_bvh(&s).unwrap_err();
    assert!(e.contains("nested deeper"), "{e}");
}

// ---------------------------------------------------------------- projection

#[test]
fn projected_angles_match_known_rotations() {
    // left upper arm rolled about Z by +40 degrees: points up-right, which is -40 on screen
    let rot = |role: &str, f: usize| if role == "uarmL" { [-(f as f64) * 10.0, 0.0, 0.0] } else { [0.0; 3] };
    let b = parse_bvh(&make_bvh(&MIXAMO, 5, 0.04, &rot, &at_origin, "\n")).unwrap();
    let pr = project_motion(&b, View::Front);
    let ia = b.joint_index("LeftArm").unwrap();
    for f in 0..5 {
        let a = pr.angles[f][ia].unwrap();
        assert!((a - f as f64 * 10.0).abs() < 1e-9, "frame {f}: {a}");
    }
    // right arm points to -x: angle 180 at rest
    let ir = b.joint_index("RightArm").unwrap();
    assert!((pr.angles[0][ir].unwrap().abs() - 180.0).abs() < 1e-9);
    // joint points are stage coordinates (y down): head above hips means smaller y
    let (ih, ihead) = (0, b.joint_index("Head").unwrap());
    assert!(pr.points[0][ihead][1] < pr.points[0][ih][1]);
    // a parent rotation is inherited (hips z +30 -> the whole arm is at -30 + its own)
    let rot2 = |role: &str, _f: usize| match role {
        "hips" => [30.0, 0.0, 0.0],
        "uarmL" => [-10.0, 0.0, 0.0],
        _ => [0.0; 3],
    };
    let b2 = parse_bvh(&make_bvh(&MIXAMO, 1, 0.04, &rot2, &at_origin, "\n")).unwrap();
    let a = project_motion(&b2, View::Front).angles[0][b2.joint_index("LeftArm").unwrap()].unwrap();
    assert!((a + 20.0).abs() < 1e-9, "{a}");
}

#[test]
fn views_side_top_and_yaw() {
    let p = [1.0, 2.0, 3.0];
    assert_eq!(View::Front.project(p), [1.0, -2.0]);
    assert_eq!(View::Side.project(p), [3.0, -2.0]);
    assert_eq!(View::Top.project(p), [1.0, -3.0]);
    let y = View::parse("yaw=90").unwrap().project(p);
    assert!((y[0] - 3.0).abs() < 1e-12 && (y[1] + 2.0).abs() < 1e-12);
    let y0 = View::parse("yaw=0").unwrap().project(p);
    assert_eq!(y0, [1.0, -2.0]);
    assert!(View::parse("yaw=abc").is_err() && View::parse("sideways").is_err());
    // an arm along X has no side-view angle (points at the camera) but has one from the front/top
    let b = parse_bvh(&make_bvh(&MIXAMO, 1, 0.04, &still, &at_origin, "\n")).unwrap();
    let ia = b.joint_index("LeftArm").unwrap();
    assert!(project_motion(&b, View::Side).angles[0][ia].is_none());
    assert!((project_motion(&b, View::Top).angles[0][ia].unwrap()).abs() < 1e-9);
    assert!(project_motion(&b, View::Yaw(90.0)).angles[0][ia].is_none());
    assert!(project_motion(&b, View::Yaw(45.0)).angles[0][ia].is_some());
}

// ---------------------------------------------------------------- names and matching

#[test]
fn name_normalisation_and_classification() {
    assert_eq!(normalize_name("mixamorig:LeftUpLeg"), ["left", "up", "leg"]);
    assert_eq!(normalize_name("Bip01 L Thigh"), ["l", "thigh"]);
    assert_eq!(normalize_name("upper_arm.L"), ["upper", "arm", "l"]);
    assert_eq!(normalize_name("lfemur"), ["l", "femur"]);
    assert_eq!(normalize_name("Spine1"), ["spine", "1"]);
    assert_eq!(normalize_name("Armature|DEF-forearm.R"), ["forearm", "r"]);
    let c = |n: &str| {
        let t = classify(n);
        (t.role, t.side)
    };
    assert_eq!(c("LeftUpLeg"), (Role::Thigh, Some(Side::Left)));
    assert_eq!(c("RightLeg"), (Role::Shin, Some(Side::Right)));
    assert_eq!(c("LeftArm"), (Role::UpperArm, Some(Side::Left)));
    assert_eq!(c("LeftForeArm"), (Role::ForeArm, Some(Side::Left)));
    assert_eq!(c("Bip01 L Calf"), (Role::Shin, Some(Side::Left)));
    assert_eq!(c("lhumerus"), (Role::UpperArm, Some(Side::Left)));
    assert_eq!(c("rradius"), (Role::ForeArm, Some(Side::Right)));
    assert_eq!(c("LeftShoulder"), (Role::Clavicle, Some(Side::Left)));
    assert_eq!(c("LeftForeArm"), (Role::ForeArm, Some(Side::Left)));
    assert_eq!(c("arm_upper_r"), (Role::UpperArm, Some(Side::Right)));
    assert_eq!(c("Pelvis"), (Role::Hips, None));
    assert_eq!(c("Spine2").0, Role::Spine);
    assert_eq!(c("LeftToeBase").0, Role::Toe);
    assert_eq!(c("LeftHandIndex1").0, Role::Finger);
    assert_eq!(c("HeadTop_End").0, Role::Other);
    assert_eq!(c("LeftToe_End").0, Role::Other);
    assert_eq!(c("foot_ik.L").0, Role::Other);
    assert_eq!(c("Bip01").0, Role::Other);
    assert_eq!(c("xyz").0, Role::Other);
    assert!(classify("LeftLeg").conf < classify("LeftUpLeg").conf);
}

fn check_full_match(bvh_names: &Names, rig_names: &RigNames, expect_via: &str, min_conf: f32) {
    let b = parse_bvh(&make_bvh(bvh_names, 1, 0.04, &still, &at_origin, "\n")).unwrap();
    let p = rig(rig_names, 1.0);
    let m = match_bones(&b, &p, None).unwrap();
    let n = bvh_names;
    let r = rig_names;
    pair(&m, n.hips, r.hips);
    pair(&m, n.spine[0], r.spine[0]);
    pair(&m, n.spine[2], r.spine[1]); // three spine joints onto two bones: first and last
    for s in 0..2 {
        pair(&m, n.clav[s], r.clav[s]);
        pair(&m, n.uarm[s], r.uarm[s]);
        pair(&m, n.farm[s], r.farm[s]);
        pair(&m, n.hand[s], r.hand[s]);
        pair(&m, n.thigh[s], r.thigh[s]);
        pair(&m, n.shin[s], r.shin[s]);
        pair(&m, n.foot[s], r.foot[s]);
        pair(&m, n.toe[s], r.toe[s]);
    }
    pair(&m, n.head, r.head);
    for x in &m.matches {
        if x.slot == "neck" || x.slot == "head" || x.slot.starts_with("spine") {
            continue;
        }
        assert!(x.confidence >= min_conf, "{}: {}", x.slot, x.confidence);
        assert_eq!(x.via, expect_via, "{} {}", x.slot, x.via);
    }
    // tips follow the chain: upper arm points at the forearm joint
    let ua = m.matches.iter().find(|x| x.slot == "upperarm.l").unwrap();
    assert_eq!(ua.tip, Tip::Joint(b.joint_index(n.farm[0]).unwrap()));
}

#[test]
fn matches_mixamo_style_by_name() {
    check_full_match(&MIXAMO, &BLENDER, "name", 0.85);
}
#[test]
fn matches_mixamo_namespace_by_name() {
    check_full_match(&MIXAMO_NS, &BLENDER, "name", 0.85);
}
#[test]
fn matches_cmu_style_by_name() {
    check_full_match(&CMU, &BLENDER, "name", 0.85);
}
#[test]
fn matches_biped_style_by_name() {
    check_full_match(&BIPED, &BLENDER, "name", 0.85);
}

#[test]
fn matches_anonymous_names_by_topology() {
    // rig has meaningless names; BVH has good ones -> topology on the rig side only
    let b = parse_bvh(&make_bvh(&MIXAMO, 1, 0.04, &still, &at_origin, "\n")).unwrap();
    let m = match_bones(&b, &rig(&RIG_ANON, 1.0), None).unwrap();
    pair(&m, "LeftArm", "b06");
    pair(&m, "RightArm", "b10");
    pair(&m, "LeftShoulder", "b05");
    pair(&m, "LeftForeArm", "b07");
    pair(&m, "LeftHand", "b08");
    pair(&m, "LeftUpLeg", "b13");
    pair(&m, "RightUpLeg", "b17");
    pair(&m, "LeftLeg", "b14");
    pair(&m, "LeftToeBase", "b16");
    pair(&m, "Head", "b04");
    pair(&m, "Neck", "b03");
    pair(&m, "Hips", "b00");
    let la = m.matches.iter().find(|x| x.bvh_name == "LeftArm").unwrap();
    assert_eq!(la.via, "topology");
    assert!(la.confidence <= 0.6);
    // both sides anonymous
    let b2 = parse_bvh(&make_bvh(&ANON, 1, 0.04, &still, &at_origin, "\n")).unwrap();
    let m2 = match_bones(&b2, &rig(&RIG_ANON, 1.0), None).unwrap();
    for (a, r) in [("j07", "b06"), ("j11", "b10"), ("j14", "b13"), ("j18", "b17"), ("j09", "b08"), ("j05", "b04"), ("j00", "b00"), ("j20", "b19"), ("j06", "b05")] {
        pair(&m2, a, r);
    }
}

#[test]
fn limb_length_ratio_reported_and_outlier_penalised() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 1, 0.04, &still, &at_origin, "\n")).unwrap();
    let mut p = rig(&BLENDER, 1.0);
    let m0 = match_bones(&b, &p, None).unwrap();
    let ua = m0.matches.iter().find(|x| x.slot == "upperarm.l").unwrap();
    let base = ua.confidence;
    assert!((ua.length_ratio.unwrap() - 40.0 / 26.0).abs() < 1e-6, "{:?}", ua.length_ratio);
    // make one forearm absurdly long
    let i = bone_idx(&p, "forearm.L");
    p.bones[i].length = 900.0;
    let m1 = match_bones(&b, &p, None).unwrap();
    let fa = m1.matches.iter().find(|x| x.slot == "forearm.l").unwrap();
    assert!(fa.confidence < 0.6 && fa.note.contains("off the median"), "{:?}", fa);
    assert_eq!(m1.matches.iter().find(|x| x.slot == "upperarm.l").unwrap().confidence, base);
    let t = format_table(&m1);
    assert!(t.contains("forearm.l") && t.contains("unmatched BVH joints"));
}

#[test]
fn manual_map_overrides_and_validates() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 1, 0.04, &still, &at_origin, "\n")).unwrap();
    let p = rig(&BLENDER, 1.0);
    // swap the two upper arms and drop the left hand
    let map = parse_map(r#"{"LeftArm": "upper_arm.R", "RightArm": "upper_arm.L", "LeftHand": null}"#).unwrap();
    let m = match_bones(&b, &p, Some(&map)).unwrap();
    let la = m.matches.iter().find(|x| x.bvh_name == "LeftArm").unwrap();
    assert_eq!((la.rig_name.as_str(), la.via, la.confidence), ("upper_arm.R", "manual", 1.0));
    assert!(m.matches.iter().all(|x| x.bvh_name != "LeftHand"));
    assert!(m.matches.iter().any(|x| x.bvh_name == "RightHand"));
    assert!(m.matches.iter().filter(|x| x.rig_name == "upper_arm.R").count() == 1);
    // wrapped form
    assert_eq!(parse_map(r#"{"map": {"A": "b"}}"#).unwrap().len(), 1);
    // errors
    assert!(match_bones(&b, &p, Some(&parse_map(r#"{"Nope": "spine"}"#).unwrap())).unwrap_err().contains("no BVH joint 'Nope'"));
    assert!(match_bones(&b, &p, Some(&parse_map(r#"{"Hips": "nope"}"#).unwrap())).unwrap_err().contains("no rig bone 'nope'"));
    assert!(match_bones(&b, &p, Some(&parse_map(r#"{"Hips": "spine", "Spine": "spine"}"#).unwrap())).unwrap_err().contains("both"));
    assert!(parse_map("[1]").is_err() && parse_map(r#"{"A": 3}"#).is_err() && parse_map("{").is_err());
    assert!(match_bones(&b, &Project::new("e", 10.0, 10.0, 24.0, 5), None).is_err());
}

// ---------------------------------------------------------------- retargeting

fn walk_rot(role: &str, f: usize) -> [f64; 3] {
    let t = f as f64 * 0.3;
    match role {
        "hips" => [4.0 * t.sin(), 0.0, 0.0],
        "sp0" => [3.0 * t.cos(), 0.0, 0.0],
        "uarmL" => [-40.0 * t.sin(), 0.0, 0.0],
        "farmL" => [-20.0 - 15.0 * t.sin(), 0.0, 0.0],
        "uarmR" => [30.0 * t.sin(), 0.0, 0.0],
        "head" => [10.0 * t.cos(), 0.0, 0.0],
        _ => [0.0; 3],
    }
}
fn walk_pos(f: usize) -> V3 {
    let t = f as f64 * 0.3;
    [2.0 * f as f64, 100.0 + 3.0 * t.sin(), 0.0]
}

#[test]
fn manual_hips_map_keeps_the_hips_slot_and_root_motion() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 40, 1.0 / 30.0, &walk_rot, &walk_pos, "\n")).unwrap();
    let names = RigNames { hips: "zz_root", ..BLENDER };
    let mut p = rig(&names, 1.0);
    p.frames = 40;
    let map = parse_map(r#"{"Hips":"zz_root"}"#).unwrap();
    let m = match_bones(&b, &p, Some(&map)).unwrap();
    assert!(m.matches.iter().any(|x| x.slot == "hips" && x.rig_name == "zz_root"), "{:?}", m.matches.iter().map(|x| (&x.slot, &x.rig_name)).collect::<Vec<_>>());
    assert!(!m.warnings.iter().any(|w| w.contains("no hips match")), "{:?}", m.warnings);
    let r = retarget(&p, &b, &m, &RetargetOptions::default()).unwrap();
    let root = bone_idx(&p, "zz_root");
    let chans: Vec<_> = r.channels.iter().filter(|c| c.target == "zz_root").collect();
    assert!(chans.iter().any(|c| matches!(c.prop, Prop::BoneOffsetX)), "no root travel");
    assert!(chans.iter().any(|c| matches!(c.prop, Prop::BoneOffsetY)), "no root bob");
    let _ = root;
}

#[test]
fn retarget_front_view_reproduces_bvh_angles_and_root_motion() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 40, 1.0 / 30.0, &walk_rot, &walk_pos, "\n")).unwrap();
    let mut p = rig(&BLENDER, 1.0);
    p.frames = 40;
    let m = match_bones(&b, &p, None).unwrap();
    let r = retarget(&p, &b, &m, &RetargetOptions::default()).unwrap();
    assert_eq!(r.out_frames, 40);
    assert!((r.scale - 85.0 / 82.0).abs() < 1e-9, "{}", r.scale);
    let mut q = p.clone();
    bake(&mut q, &r);
    let e = Engine::new(q.clone()).unwrap();
    // expected world angle = rig rest world angle + 2D rest-relative angle, computed from 3D
    let proj = project_motion(&b, View::Front);
    let rest = project_motion(&parse_bvh(&make_bvh(&MIXAMO, 1, 1.0 / 30.0, &still, &at_origin, "\n")).unwrap(), View::Front);
    let rw = crate::rigedit::rest_world(&p).unwrap();
    for (bj, rb) in [("LeftArm", "upper_arm.L"), ("LeftForeArm", "forearm.L"), ("RightArm", "upper_arm.R"), ("Head", "head"), ("Spine", "spine"), ("Hips", "pelvis"), ("LeftUpLeg", "thigh.L"), ("RightLeg", "shin.R"), ("LeftFoot", "foot.L")] {
        let (ji, ri) = (b.joint_index(bj).unwrap(), bone_idx(&p, rb));
        // the hips/spine tips differ from the default tip only for hips (default: longest child)
        for f in [0usize, 7, 19, 39] {
            let want = match bj {
                "Hips" => {
                    // hips tip is the spine joint
                    let si = b.joint_index("Spine").unwrap();
                    let pose = b.pose(Some(f));
                    let rp = b.pose(None);
                    norm360(angle_2d(pose.pos[0], pose.pos[si], View::Front, false).unwrap() - angle_2d(rp.pos[0], rp.pos[si], View::Front, false).unwrap())
                }
                _ => norm360(proj.angles[f][ji].unwrap() - rest.angles[0][ji].unwrap()),
            };
            let got = world_angle(&e, f as f32, ri) - rw[ri].1 as f64;
            assert!(ang_diff(got, want) < 0.02, "{bj} frame {f}: got {got} want {want}");
        }
    }
    // root travel: +x moves right by scale*dx, bob up is smaller y
    let hb = bone_idx(&p, "pelvis");
    let w0 = e.evaluate(0.0).world[hb];
    let w30 = e.evaluate(30.0).world[hb];
    let dx = 2.0 * 30.0 * r.scale;
    assert!(((w30.tx - w0.tx) as f64 - dx).abs() < 0.01, "{} vs {dx}", w30.tx - w0.tx);
    let t30: f64 = 30.0 * 0.3;
    let dy = -(3.0 * t30.sin()) * r.scale;
    assert!(((w30.ty - w0.ty) as f64 - dy).abs() < 0.01, "{} vs {dy}", w30.ty - w0.ty);
    // channels are appended, original data untouched
    assert_eq!(q.channels.len(), p.channels.len() + r.channels.len());
    assert_eq!(&q.bones, &p.bones);
}

#[test]
fn different_proportions_retarget_to_same_angles() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 20, 1.0 / 30.0, &walk_rot, &walk_pos, "\n")).unwrap();
    let mut angles = Vec::new();
    for k in [0.5f32, 1.0, 2.5] {
        let mut p = rig(&BLENDER, k);
        p.frames = 20;
        let m = match_bones(&b, &p, None).unwrap();
        let r = retarget(&p, &b, &m, &RetargetOptions::default()).unwrap();
        bake(&mut p, &r);
        let e = Engine::new(p.clone()).unwrap();
        angles.push(world_angle(&e, 13.0, bone_idx(&p, "forearm.L")));
        // root travel scales with the leg length
        let hb = bone_idx(&p, "pelvis");
        let travel = e.evaluate(19.0).world[hb].tx - e.evaluate(0.0).world[hb].tx;
        let expect = 2.0 * 19.0 * (k as f64 * 85.0 / 82.0);
        assert!(((travel as f64) - expect).abs() < 0.05 * expect.max(1.0), "k={k}: {travel} vs {expect}");
    }
    assert!(ang_diff(angles[0], angles[1]) < 0.05 && ang_diff(angles[1], angles[2]) < 0.05, "{angles:?}");
}

#[test]
fn side_view_knee_bend() {
    let rot = |role: &str, f: usize| match role {
        "thighL" => [0.0, 5.0 * f as f64, 0.0],
        "shinL" => [0.0, -3.0 * f as f64, 0.0],
        _ => [0.0; 3],
    };
    let b = parse_bvh(&make_bvh(&MIXAMO, 10, 1.0 / 30.0, &rot, &at_origin, "\n")).unwrap();
    let mut p = rig(&BLENDER, 1.0);
    p.frames = 10;
    let m = match_bones(&b, &p, None).unwrap();
    let o = RetargetOptions { view: View::Side, ..Default::default() };
    let r = retarget(&p, &b, &m, &o).unwrap();
    // arms point at the camera in the side view: reported, not a failure
    assert!(r.degenerate == 0, "arms have a rest angle problem, not frame problem");
    assert!(r.notes.iter().any(|n| n.contains("points along the view axis")), "{:?}", r.notes);
    bake(&mut p, &r);
    let e = Engine::new(p.clone()).unwrap();
    let (ti, si) = (bone_idx(&p, "thigh.L"), bone_idx(&p, "shin.L"));
    // thigh Delta = +5f (see derivation: side-view angle is 90 + alpha); shin world = thigh + (-3f)
    for f in [0usize, 4, 9] {
        assert!(ang_diff(world_angle(&e, f as f32, ti), 90.0 + 5.0 * f as f64) < 0.02);
        assert!(ang_diff(world_angle(&e, f as f32, si), 90.0 + 2.0 * f as f64) < 0.02, "{}", world_angle(&e, f as f32, si));
    }
}

#[test]
fn resamples_to_project_rate_and_extends_project() {
    // 60 fps, 61 frames (1 s): arm sweeps linearly 0 -> -60 deg Z (screen +60)
    let rot = |role: &str, f: usize| if role == "uarmL" { [-(f as f64), 0.0, 0.0] } else { [0.0; 3] };
    let b = parse_bvh(&make_bvh(&MIXAMO, 61, 1.0 / 60.0, &rot, &at_origin, "\n")).unwrap();
    let mut p = rig(&BLENDER, 1.0);
    p.fps = 24.0;
    p.frames = 10;
    let m = match_bones(&b, &p, None).unwrap();
    let r = retarget(&p, &b, &m, &RetargetOptions::default()).unwrap();
    assert_eq!(r.out_frames, 25);
    assert_eq!(r.project_frames, 25);
    let ch = r.channels.iter().find(|c| c.target == "upper_arm.L" && c.prop == Prop::BoneAngle).unwrap();
    let crate::project::Source::Keys { track } = &ch.source else { panic!() };
    assert_eq!(track.keys.len(), 25);
    for (k, key) in track.keys.iter().enumerate() {
        // world delta = 60 * k/24; upper_arm.L parent chain is at rest so local delta = same
        assert!((key.value as f64 - 60.0 * k as f64 / 24.0).abs() < 1e-3, "{k}: {}", key.value);
        assert_eq!(key.frame, k as f32);
    }
    let mut q = p.clone();
    bake(&mut q, &r);
    assert_eq!(q.frames, 25);
    // explicit fps and sub-range
    let o = RetargetOptions { fps: Some(10.0), start: Some(12), end: Some(42), ..Default::default() };
    let r2 = retarget(&p, &b, &m, &o).unwrap();
    assert_eq!(r2.out_frames, 6); // 30 source intervals of 1/60 s = 0.5 s -> 0..0.5 s at 10 fps
}

#[test]
fn smoothing_damps_jitter_and_keeps_endpoints() {
    let rot = |role: &str, f: usize| if role == "uarmL" { [if f.is_multiple_of(2) { 0.0 } else { -20.0 }, 0.0, 0.0] } else { [0.0; 3] };
    let b = parse_bvh(&make_bvh(&MIXAMO, 21, 1.0 / 30.0, &rot, &at_origin, "\n")).unwrap();
    let mut p = rig(&BLENDER, 1.0);
    p.frames = 21;
    let m = match_bones(&b, &p, None).unwrap();
    let var = |smooth: usize| {
        let o = RetargetOptions { smooth, ..Default::default() };
        let r = retarget(&p, &b, &m, &o).unwrap();
        let ch = r.channels.iter().find(|c| c.target == "upper_arm.L" && c.prop == Prop::BoneAngle).unwrap();
        let crate::project::Source::Keys { track } = &ch.source else { panic!() };
        let v: Vec<f32> = track.keys.iter().map(|k| k.value).collect();
        (v[0], v[20], v[1..20].iter().map(|x| (x - 10.0).abs()).fold(0.0, f32::max))
    };
    let (a0, a20, j0) = var(0);
    let (b0, b20, j1) = var(1);
    assert!(j0 > 9.0 && j1 < 0.5 * j0 + 3.5, "{j0} {j1}");
    assert_eq!((a0, a20), (b0, b20));
}

#[test]
fn deterministic_and_locks_respected() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 30, 1.0 / 30.0, &walk_rot, &walk_pos, "\n")).unwrap();
    let mut p = rig(&BLENDER, 1.0);
    let m1 = match_bones(&b, &p, None).unwrap();
    let m2 = match_bones(&b, &p, None).unwrap();
    assert_eq!(format_table(&m1), format_table(&m2));
    let r1 = retarget(&p, &b, &m1, &RetargetOptions::default()).unwrap();
    let r2 = retarget(&p, &b, &m2, &RetargetOptions::default()).unwrap();
    assert_eq!(serde_json::to_string(&r1.channels).unwrap(), serde_json::to_string(&r2.channels).unwrap());
    // lock one channel
    p.channels.push(Channel::keys("upper_arm.L", Prop::BoneAngle, Track::with_keys(&[(0.0, 5.0, Ease::Linear)])));
    p.channel_meta.push(crate::chanmeta::ChannelMeta { target: "upper_arm.L".into(), prop: Some(Prop::BoneAngle), lock: true, ..Default::default() });
    let r = retarget(&p, &b, &m1, &RetargetOptions::default()).unwrap();
    assert_eq!(r.blocked, vec!["upper_arm.L.BoneAngle".to_string()]);
    assert!(!r.channels.iter().any(|c| c.target == "upper_arm.L"));
    assert!(r.channels.iter().any(|c| c.target == "upper_arm.R"));
    assert!(r.notes.iter().any(|n| n.contains("locked")));
}

#[test]
fn editor_undo_removes_the_whole_retarget() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 50, 1.0 / 30.0, &walk_rot, &walk_pos, "\n")).unwrap();
    let mut p = rig(&BLENDER, 1.0);
    p.frames = 20;
    let m = match_bones(&b, &p, None).unwrap();
    let r = retarget(&p, &b, &m, &RetargetOptions::default()).unwrap();
    let mut ed = crate::edit::Editor::new(p.clone()).unwrap();
    let before = ed.doc().clone();
    ed.apply(bake_op(ed.project(), &r).unwrap()).unwrap();
    assert_eq!(ed.project().channels.len(), r.channels.len());
    assert_eq!(ed.project().frames, 50);
    let mut direct = p.clone();
    bake(&mut direct, &r);
    assert_eq!(&direct, ed.project());
    assert!(ed.undo());
    assert_eq!(ed.doc(), &before);
    assert!(ed.redo());
    assert_eq!(ed.project().channels.len(), r.channels.len());
}

#[test]
fn options_flags_and_validation() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 10, 1.0 / 30.0, &walk_rot, &walk_pos, "\n")).unwrap();
    let p = rig(&BLENDER, 1.0);
    let m = match_bones(&b, &p, None).unwrap();
    let has = |r: &RetargetResult, t: &str, pr: Prop| r.channels.iter().any(|c| c.target == t && c.prop == pr);
    let base = retarget(&p, &b, &m, &RetargetOptions::default()).unwrap();
    assert!(has(&base, "pelvis", Prop::BoneOffsetX) && has(&base, "pelvis", Prop::BoneOffsetY));
    let nr = retarget(&p, &b, &m, &RetargetOptions { root_motion: false, ..Default::default() }).unwrap();
    assert!(!has(&nr, "pelvis", Prop::BoneOffsetX));
    // in place: x travel zero, bob kept
    let ip = retarget(&p, &b, &m, &RetargetOptions { in_place: true, ..Default::default() }).unwrap();
    let get = |r: &RetargetResult, pr: Prop| {
        let c = r.channels.iter().find(|c| c.target == "pelvis" && c.prop == pr).unwrap();
        let crate::project::Source::Keys { track } = &c.source else { panic!() };
        track.keys.iter().map(|k| k.value).collect::<Vec<f32>>()
    };
    assert!(get(&ip, Prop::BoneOffsetX).iter().all(|v| *v == 0.0));
    assert_eq!(get(&ip, Prop::BoneOffsetY), get(&base, Prop::BoneOffsetY));
    // fixed scale and weight
    let fx = retarget(&p, &b, &m, &RetargetOptions { scale: Scale::Fixed(2.0), weight: 0.5, ..Default::default() }).unwrap();
    assert_eq!(fx.scale, 2.0);
    assert!(fx.channels.iter().all(|c| c.weight == 0.5));
    let x9 = get(&base, Prop::BoneOffsetX)[9];
    assert!((get(&fx, Prop::BoneOffsetX)[9] - x9 * 2.0 / base.scale as f32).abs() < 1e-3);
    // mirror flips the sign of root travel and of angle deltas
    let mi = retarget(&p, &b, &m, &RetargetOptions { mirror: true, ..Default::default() }).unwrap();
    assert!((get(&mi, Prop::BoneOffsetX)[9] + x9).abs() < 1e-3);
    // min confidence filters
    let hi = retarget(&p, &b, &m, &RetargetOptions { min_confidence: 0.99, ..Default::default() }).unwrap();
    assert!(hi.bones_used < base.bones_used);
    assert!(retarget(&p, &b, &m, &RetargetOptions { min_confidence: 2.0, ..Default::default() }).is_err());
    // errors
    for o in [
        RetargetOptions { start: Some(5), end: Some(2), ..Default::default() },
        RetargetOptions { end: Some(99), ..Default::default() },
        RetargetOptions { fps: Some(0.0), ..Default::default() },
        RetargetOptions { fps: Some(f64::NAN), ..Default::default() },
        RetargetOptions { scale: Scale::Fixed(-1.0), ..Default::default() },
        RetargetOptions { weight: 2.0, ..Default::default() },
        RetargetOptions { rest_frame: Some(50), ..Default::default() },
        RetargetOptions { smooth: 5000, ..Default::default() },
    ] {
        assert!(retarget(&p, &b, &m, &o).is_err(), "{o:?}");
    }
    let empty = parse_bvh("HIERARCHY\nROOT A\n{\nOFFSET 0 0 0\nCHANNELS 0\n}\nMOTION\nFrames: 0\nFrame Time: 0.1\n").unwrap();
    assert!(retarget(&p, &empty, &m, &RetargetOptions::default()).unwrap_err().contains("no frames"));
}

#[test]
fn rest_frame_option_uses_that_pose_as_reference() {
    // frame 0 already has the arm raised 30 degrees; with rest_frame 0 the delta starts at zero
    let rot = |role: &str, f: usize| if role == "uarmL" { [-30.0 - f as f64, 0.0, 0.0] } else { [0.0; 3] };
    let b = parse_bvh(&make_bvh(&MIXAMO, 5, 1.0 / 30.0, &rot, &at_origin, "\n")).unwrap();
    let mut p = rig(&BLENDER, 1.0);
    p.frames = 5;
    let m = match_bones(&b, &p, None).unwrap();
    let val = |o: &RetargetOptions| {
        let r = retarget(&p, &b, &m, o).unwrap();
        let c = r.channels.iter().find(|c| c.target == "upper_arm.L" && c.prop == Prop::BoneAngle).unwrap();
        let crate::project::Source::Keys { track } = &c.source else { panic!() };
        (track.keys[0].value, track.keys[4].value)
    };
    let (a, _) = val(&RetargetOptions::default());
    assert!((a - 30.0).abs() < 1e-4);
    let (a, z) = val(&RetargetOptions { rest_frame: Some(0), ..Default::default() });
    assert!(a.abs() < 1e-4 && (z - 4.0).abs() < 1e-4, "{a} {z}");
}

#[test]
fn foot_lock_reuses_footlock_module() {
    let b = parse_bvh(&make_bvh(&MIXAMO, 12, 1.0 / 30.0, &walk_rot, &walk_pos, "\n")).unwrap();
    let mut p = rig(&BLENDER, 1.0);
    p.bones.push(Bone::new("footctl", None, 5.0, Vec2::new(100.0, 200.0), 0.0));
    let m = match_bones(&b, &p, None).unwrap();
    let plain = retarget(&p, &b, &m, &RetargetOptions::default()).unwrap();
    let o = RetargetOptions { foot_lock: Some(("footctl".into(), 200.0)), ..Default::default() };
    let r = retarget(&p, &b, &m, &o).unwrap();
    assert_eq!(r.channels.len(), plain.channels.len() + 2);
    // controller sits on the ground the whole time: no contact change is needed but the call ran
    assert!(r.notes.iter().any(|n| n.contains("foot lock")), "{:?}", r.notes);
    // a child bone is not a valid controller
    let bad = RetargetOptions { foot_lock: Some(("shin.L".into(), 200.0)), ..Default::default() };
    assert!(retarget(&p, &b, &m, &bad).is_err());
}
