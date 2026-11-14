// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
use osatura_capi::*;
use std::ffi::CStr;

fn demo_json() -> String {
    osatura_core::demo::project().to_json_pretty()
}

fn last_error() -> String {
    unsafe {
        CStr::from_ptr(osatura_last_error())
            .to_string_lossy()
            .into_owned()
    }
}

#[test]
fn open_query_render_close() {
    let j = demo_json();
    unsafe {
        let e = osatura_open(j.as_ptr() as *const _, j.len());
        assert!(!e.is_null(), "{}", last_error());
        assert_eq!(osatura_frame_count(e), 48);
        assert_eq!(osatura_width(e), 640);
        assert_eq!(osatura_height(e), 480);
        assert_eq!(osatura_fps(e), 24.0);
        let nb = osatura_bone_count(e);
        assert!(nb >= 6);
        let mut name = [0 as std::ffi::c_char; 32];
        let full = osatura_bone_name(e, 0, name.as_mut_ptr(), name.len());
        assert!(full > 0);
        assert_eq!(
            CStr::from_ptr(name.as_ptr()).to_str().unwrap().len(),
            full as usize
        );
        assert_eq!(osatura_bone_name(e, nb, name.as_mut_ptr(), name.len()), -1);
        let mut m = vec![0f32; nb as usize * 6];
        assert_eq!(osatura_bone_world(e, 5.0, m.as_mut_ptr(), m.len()), 0);
        assert!(m.iter().any(|v| *v != 0.0));
        assert_eq!(osatura_bone_world(e, 5.0, m.as_mut_ptr(), 3), -1);

        let (mut w, mut h) = (0u32, 0u32);
        assert_eq!(
            osatura_render_rgba(e, 3.0, 0.25, std::ptr::null_mut(), 0, &mut w, &mut h),
            -2
        );
        assert_eq!((w, h), (160, 120));
        let mut buf = vec![0u8; (w * h * 4) as usize];
        assert_eq!(
            osatura_render_rgba(e, 3.0, 0.25, buf.as_mut_ptr(), buf.len(), &mut w, &mut h),
            0
        );
        assert!(buf.iter().any(|v| *v != 0));
        assert_eq!(
            osatura_render_rgba(
                e,
                f32::NAN,
                0.25,
                buf.as_mut_ptr(),
                buf.len(),
                &mut w,
                &mut h
            ),
            -1
        );
        assert_eq!(
            osatura_render_rgba(e, 1.0, 100.0, buf.as_mut_ptr(), buf.len(), &mut w, &mut h),
            -1
        );
        osatura_close(e);
        osatura_close(std::ptr::null_mut());
    }
}

#[test]
fn bad_input_gives_errors_not_crashes() {
    unsafe {
        assert!(osatura_open(std::ptr::null(), 0).is_null());
        assert!(last_error().contains("null pointer"));
        let junk = b"{\"nope\": 1}";
        assert!(osatura_open(junk.as_ptr() as *const _, junk.len()).is_null());
        assert!(!last_error().is_empty());
        let bin = [0xffu8, 0xfe, 0x00];
        assert!(osatura_open(bin.as_ptr() as *const _, bin.len()).is_null());
        assert_eq!(osatura_frame_count(std::ptr::null()), 0);
        assert_eq!(osatura_abi_version(), 1);
        assert!(CStr::from_ptr(osatura_version())
            .to_str()
            .unwrap()
            .starts_with("0."));
    }
}

fn machine_json() -> String {
    use osatura_core::anim::{Ease, Key};
    use osatura_core::machine::*;
    use osatura_core::motionlib::{Take, TakeTrack};
    use osatura_core::project::Prop;
    let mut p = osatura_core::demo::project();
    let bone = p.bones[0].name.clone();
    let mk = |n: &str, v: f32| {
        let keys = vec![
            Key {
                frame: 0.0,
                value: v,
                ease: Ease::Linear,
            },
            Key {
                frame: 10.0,
                value: v,
                ease: Ease::Linear,
            },
        ];
        osatura_core::clipstack::Clip::new(
            n,
            Take {
                name: n.into(),
                length: 10.0,
                tracks: vec![TakeTrack {
                    target: bone.clone(),
                    prop: Prop::BoneAngle,
                    keys,
                }],
            },
        )
    };
    p.clips = vec![mk("a", 0.0), mk("b", 40.0)];
    let st = |n: &str| State {
        name: n.into(),
        source: StateSource::Clip { clip: n.into() },
        mode: PlayMode::Loop,
        speed: 1.0,
        blend_in: 0.0,
    };
    p.machines.push(Machine {
        name: "hero".into(),
        inputs: vec![
            Input {
                name: "go".into(),
                kind: InputKind::Bool,
                default: 0.0,
                hold: false,
            },
            Input {
                name: "zap".into(),
                kind: InputKind::Trigger,
                default: 0.0,
                hold: false,
            },
        ],
        layers: vec![Layer {
            name: "body".into(),
            mask: vec![],
            entry: "a".into(),
            states: vec![st("a"), st("b")],
            transitions: vec![
                Transition {
                    from: "a".into(),
                    to: "b".into(),
                    conditions: vec![Condition {
                        input: "go".into(),
                        op: Cmp::Eq,
                        value: 1.0,
                    }],
                    exit_time: None,
                    duration: Some(0.0),
                    ease: Ease::Linear,
                },
                Transition {
                    from: "b".into(),
                    to: "a".into(),
                    conditions: vec![Condition {
                        input: "zap".into(),
                        op: Cmp::Eq,
                        value: 1.0,
                    }],
                    exit_time: None,
                    duration: Some(0.0),
                    ease: Ease::Linear,
                },
            ],
        }],
    });
    p.to_json_pretty()
}

#[test]
fn player_state_machine_roundtrip() {
    use std::ffi::CString;
    let j = machine_json();
    let c = |s: &str| CString::new(s).unwrap();
    unsafe {
        let e = osatura_open(j.as_ptr() as *const _, j.len());
        assert!(!e.is_null(), "{}", last_error());
        assert_eq!(osatura_machine_count(e), 1);
        let mut name = [0 as std::ffi::c_char; 16];
        assert_eq!(osatura_machine_name(e, 0, name.as_mut_ptr(), name.len()), 4);
        assert_eq!(
            osatura_machine_name(e, 1, name.as_mut_ptr(), name.len()),
            -1
        );
        assert!(osatura_player_open(e, c("nope").as_ptr()).is_null());
        assert!(last_error().contains("nope"));
        let p = osatura_player_open(e, c("hero").as_ptr());
        assert!(!p.is_null(), "{}", last_error());
        osatura_close(e); // the player owns its copy
        let state = |p| {
            let mut b = [0 as std::ffi::c_char; 16];
            let n = osatura_player_state(p, c("body").as_ptr(), b.as_mut_ptr(), b.len());
            assert!(n > 0);
            CStr::from_ptr(b.as_ptr()).to_str().unwrap().to_string()
        };
        assert_eq!(state(p), "a");
        assert_eq!(osatura_player_advance(p, 0.1), 0);
        assert_eq!(osatura_player_set_bool(p, c("go").as_ptr(), 1), 0);
        assert_eq!(osatura_player_advance(p, 0.1), 0);
        assert_eq!(state(p), "b");
        assert_eq!(osatura_player_set_bool(p, c("go").as_ptr(), 0), 0);
        assert_eq!(osatura_player_fire(p, c("zap").as_ptr()), 0);
        assert_eq!(osatura_player_advance(p, 0.05), 0);
        assert_eq!(state(p), "a");
        // events: query size, too-small buffer keeps them, big buffer consumes them
        let need = osatura_player_events(p, std::ptr::null_mut(), 0);
        assert!(need > 10);
        let mut small = [0 as std::ffi::c_char; 8];
        assert_eq!(
            osatura_player_events(p, small.as_mut_ptr(), small.len()),
            need
        );
        let mut big = vec![0 as std::ffi::c_char; need as usize + 1];
        assert_eq!(osatura_player_events(p, big.as_mut_ptr(), big.len()), need);
        let text = CStr::from_ptr(big.as_ptr()).to_str().unwrap().to_string();
        assert!(
            text.contains("\"entered\"") && text.contains("\"state\":\"b\""),
            "{text}"
        );
        assert_eq!(osatura_player_events(p, std::ptr::null_mut(), 0), 2); // "[]"
                                                                          // render
        let (mut w, mut h) = (0u32, 0u32);
        assert_eq!(
            osatura_player_render_rgba(p, 0.25, std::ptr::null_mut(), 0, &mut w, &mut h),
            -2
        );
        let mut buf = vec![0u8; (w * h * 4) as usize];
        assert_eq!(
            osatura_player_render_rgba(p, 0.25, buf.as_mut_ptr(), buf.len(), &mut w, &mut h),
            0
        );
        assert!(buf.iter().any(|v| *v != 0));
        // errors
        assert_eq!(osatura_player_set_bool(p, c("zap").as_ptr(), 1), -1);
        assert_eq!(osatura_player_set_number(p, c("nope").as_ptr(), 1.0), -1);
        assert_eq!(osatura_player_set_number(p, c("go").as_ptr(), f64::NAN), -1);
        assert_eq!(osatura_player_advance(p, -1.0), -1);
        assert_eq!(osatura_player_set_value(p, c("bad path").as_ptr(), 1.0), -1);
        assert_eq!(osatura_player_set_value(p, c("ok.path").as_ptr(), 1.0), 0);
        assert_eq!(
            osatura_player_state(p, c("zzz").as_ptr(), std::ptr::null_mut(), 0),
            -1
        );
        assert_eq!(
            osatura_player_render_rgba(p, 99.0, buf.as_mut_ptr(), buf.len(), &mut w, &mut h),
            -1
        );
        assert_eq!(osatura_player_fire(p, std::ptr::null()), -1);
        assert_eq!(osatura_player_advance(std::ptr::null_mut(), 0.1), -1);
        osatura_player_close(p);
        osatura_player_close(std::ptr::null_mut());
    }
}

fn scroll_json() -> String {
    use osatura_core::containers::{Container, Dir, Member, RectRule, Scroll};
    use osatura_core::path::VPath;
    use osatura_core::project::Layer;
    let mut p = osatura_core::project::Project::from_json(&machine_json()).unwrap();
    for i in 0..10 {
        p.layers.push(Layer::vector(
            &format!("row{i}"),
            vec![VPath::rect("r", 0.0, 0.0, 100.0, 20.0).with_fill("#336699")],
        ));
    }
    p.containers.push(Container::new(
        "col",
        Dir::Column,
        (0..10).map(|i| Member::layer(&format!("row{i}"))).collect(),
    ));
    let mut s = Scroll::new("list", "col");
    s.viewport = RectRule::Pin {
        left: Some(0.0),
        top: Some(0.0),
        right: None,
        bottom: Some(0.0),
        width: Some(100.0),
        height: None,
    };
    s.snap_y = vec![0.0, 40.0, 80.0, 120.0, 150.0];
    s.input = Some("ui.scroll".into());
    p.scrolls.push(s);
    p.to_json_pretty()
}

#[test]
fn player_layout_scroll_and_lod() {
    use std::ffi::CString;
    let j = scroll_json();
    let c = |s: &str| CString::new(s).unwrap();
    unsafe {
        let e = osatura_open(j.as_ptr() as *const _, j.len());
        assert!(!e.is_null(), "{}", last_error());
        let p = osatura_player_open(e, c("hero").as_ptr());
        assert!(!p.is_null(), "{}", last_error());
        osatura_close(e);
        assert_eq!(osatura_player_scroll_count(p), 1);
        let mut id = [0 as std::ffi::c_char; 16];
        assert_eq!(osatura_player_scroll_id(p, 0, id.as_mut_ptr(), id.len()), 4);
        assert_eq!(
            osatura_player_scroll_id(p, 1, id.as_mut_ptr(), id.len()),
            -1
        );
        let (mut w, mut h) = (0f32, 0f32);
        assert_eq!(osatura_player_canvas_size(p, &mut w, &mut h), 0);
        assert!(w > 0.0 && h > 0.0);
        // the viewport follows the canvas height: 200 content - 120 canvas = 80 to scroll
        assert_eq!(osatura_player_set_canvas(p, w, 120.0), 0);
        let (mut mx, mut my) = (0f32, 0f32);
        assert_eq!(
            osatura_player_scroll_max(p, c("list").as_ptr(), &mut mx, &mut my),
            0
        );
        assert!((my - 80.0).abs() < 1e-3 && mx == 0.0, "{my}");
        // a render now has the canvas size
        let (mut pw, mut ph) = (0u32, 0u32);
        assert_eq!(
            osatura_player_render_rgba(p, 1.0, std::ptr::null_mut(), 0, &mut pw, &mut ph),
            -2
        );
        assert_eq!((pw, ph), (w.round() as u32, 120));
        // drag, release, advance until it rests on a snap point
        assert_eq!(
            osatura_player_scroll_drag(p, c("list").as_ptr(), 0.0, 30.0),
            0
        );
        assert_eq!(
            osatura_player_scroll_release(p, c("list").as_ptr(), 0.0, 100.0),
            0
        );
        assert_eq!(osatura_player_scroll_settled(p, c("list").as_ptr()), 0);
        for _ in 0..300 {
            assert_eq!(osatura_player_advance(p, 1.0 / 60.0), 0);
        }
        assert_eq!(osatura_player_scroll_settled(p, c("list").as_ptr()), 1);
        let (mut ox, mut oy) = (0f32, 0f32);
        assert_eq!(
            osatura_player_scroll_offset(p, c("list").as_ptr(), &mut ox, &mut oy),
            0
        );
        assert_eq!(oy, 40.0);
        // linked input path
        assert_eq!(
            osatura_player_set_value(p, c("ui.scroll").as_ptr(), 60.0),
            0
        );
        assert_eq!(
            osatura_player_scroll_offset(p, c("list").as_ptr(), std::ptr::null_mut(), &mut oy),
            0
        );
        assert_eq!(oy, 60.0);
        // errors
        assert_eq!(
            osatura_player_scroll_drag(p, c("nope").as_ptr(), 0.0, 0.0),
            -1
        );
        assert_eq!(
            osatura_player_scroll_drag(p, c("list").as_ptr(), f32::NAN, 0.0),
            -1
        );
        assert_eq!(
            osatura_player_scroll_release(p, c("list").as_ptr(), 0.0, f32::INFINITY),
            -1
        );
        assert_eq!(
            osatura_player_scroll_offset(p, c("nope").as_ptr(), &mut ox, &mut oy),
            -1
        );
        assert_eq!(osatura_player_scroll_settled(p, std::ptr::null()), -1);
        assert_eq!(osatura_player_set_canvas(p, 0.0, 10.0), -1);
        assert_eq!(osatura_player_set_canvas(p, f32::NAN, 10.0), -1);
        assert_eq!(
            osatura_player_set_canvas(std::ptr::null_mut(), 10.0, 10.0),
            -1
        );
        assert_eq!(osatura_player_scroll_count(std::ptr::null_mut()), 0);
        // level of detail
        assert_eq!(osatura_player_lod_level(p), 0);
        assert_eq!(osatura_player_set_screen(p, w, 120.0, 0.0), 0);
        let exact = osatura_player_lod_level(p); // only levels with a declared bound of 0 qualify
        assert_eq!(osatura_player_set_screen(p, w, 120.0, 8.0), 0);
        assert!(osatura_player_lod_level(p) >= exact);
        assert_eq!(
            osatura_player_render_rgba(p, 0.25, std::ptr::null_mut(), 0, &mut pw, &mut ph),
            -2,
            "still renders"
        );
        assert_eq!(osatura_player_set_screen(p, 0.0, 120.0, 1.0), -1);
        assert_eq!(osatura_player_set_screen(p, w, 120.0, -1.0), -1);
        assert_eq!(osatura_player_lod_level(std::ptr::null_mut()), -1);
        osatura_player_close(p);
    }
}

/// The behaviour osatura.h documents for the render size outputs and for the last-error text.
#[test]
fn documented_size_outputs_and_last_error_semantics() {
    let j = demo_json();
    unsafe {
        let e = osatura_open(j.as_ptr() as *const _, j.len());
        assert!(!e.is_null());
        assert_eq!(
            last_error(),
            "",
            "empty until the first failure on this thread"
        );
        // width/height: rounded stage size; a render is round(size * scale), at least 1
        let (mut w, mut h) = (7u32, 7u32);
        assert_eq!(
            osatura_render_rgba(e, 0.0, 0.05, std::ptr::null_mut(), 0, &mut w, &mut h),
            -2
        );
        assert_eq!((w, h), (32, 24), "round(640*0.05), round(480*0.05)");
        assert!(last_error().contains("needs"), "{}", last_error());
        // a rejected frame/scale or a null engine does not touch *w / *h
        let (mut w, mut h) = (7u32, 9u32);
        let mut buf = vec![0u8; 4];
        assert_eq!(
            osatura_render_rgba(
                e,
                f32::NAN,
                1.0,
                buf.as_mut_ptr(),
                buf.len(),
                &mut w,
                &mut h
            ),
            -1
        );
        assert_eq!((w, h), (7, 9));
        assert_eq!(
            osatura_render_rgba(
                std::ptr::null(),
                0.0,
                1.0,
                buf.as_mut_ptr(),
                buf.len(),
                &mut w,
                &mut h
            ),
            -1
        );
        assert_eq!((w, h), (7, 9));
        // the error text of the failure stays after a successful call; only the next failing call replaces it
        let before = last_error();
        assert!(before.contains("null engine"), "{before}");
        assert_eq!(osatura_frame_count(e), 48);
        let mut big = vec![0u8; 32 * 24 * 4];
        assert_eq!(
            osatura_render_rgba(e, 0.0, 0.05, big.as_mut_ptr(), big.len(), &mut w, &mut h),
            0
        );
        assert_eq!((w, h), (32, 24), "also written on success");
        assert_eq!(
            last_error(),
            before,
            "a success does not clear the error text"
        );
        assert_eq!(osatura_bone_name(e, 9999, std::ptr::null_mut(), 0), -1);
        osatura_close(e);
    }
}

fn cs(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap()
}

unsafe fn take(b: *mut OsaturaBlob) -> Vec<u8> {
    assert!(!b.is_null(), "{}", last_error());
    let v = std::slice::from_raw_parts(osatura_blob_data(b), osatura_blob_size(b)).to_vec();
    osatura_blob_free(b);
    v
}

#[test]
fn editing_basics_and_undo() {
    let j = demo_json();
    unsafe {
        let e = osatura_open(j.as_ptr() as *const _, j.len());
        assert!(!e.is_null());
        let l0 = osatura_layer_count(e);
        let b0 = osatura_bone_count(e);
        assert_eq!(osatura_add_group_layer(e, cs("grp").as_ptr()), 0);
        assert_eq!(
            osatura_add_rect_layer(
                e,
                cs("box").as_ptr(),
                0.0,
                0.0,
                40.0,
                30.0,
                cs("#336699").as_ptr(),
                cs("grp").as_ptr()
            ),
            0
        );
        assert_eq!(
            osatura_add_ellipse_layer(
                e,
                cs("dot").as_ptr(),
                20.0,
                20.0,
                5.0,
                5.0,
                cs("#f00").as_ptr(),
                std::ptr::null()
            ),
            0
        );
        assert_eq!(osatura_layer_count(e), l0 + 3);
        let mut name = [0 as std::ffi::c_char; 64];
        let n = osatura_layer_name(e, l0 + 1, name.as_mut_ptr(), name.len());
        assert_eq!(CStr::from_ptr(name.as_ptr()).to_str().unwrap(), "grp/box");
        assert_eq!(n as usize, "grp/box".len());
        assert_eq!(
            osatura_layer_name(e, l0 + 3, name.as_mut_ptr(), name.len()),
            -1
        );

        // failures change nothing and report
        assert_eq!(osatura_add_group_layer(e, cs("grp").as_ptr()), -1);
        assert!(last_error().contains("already exists"));
        assert_eq!(
            osatura_add_rect_layer(
                e,
                cs("bad").as_ptr(),
                0.0,
                0.0,
                1.0,
                1.0,
                cs("nonsense").as_ptr(),
                std::ptr::null()
            ),
            -1
        );
        assert_eq!(
            osatura_add_rect_layer(
                e,
                std::ptr::null(),
                0.0,
                0.0,
                1.0,
                1.0,
                cs("#fff").as_ptr(),
                std::ptr::null()
            ),
            -1
        );
        assert_eq!(
            osatura_add_rect_layer(
                e,
                cs("nan").as_ptr(),
                f32::NAN,
                0.0,
                1.0,
                1.0,
                cs("#fff").as_ptr(),
                std::ptr::null()
            ),
            -1
        );
        assert_eq!(
            osatura_add_group_layer(std::ptr::null_mut(), cs("x").as_ptr()),
            -1
        );
        assert_eq!(osatura_layer_count(e), l0 + 3);

        let mut root = [0 as std::ffi::c_char; 32];
        osatura_bone_name(e, 0, root.as_mut_ptr(), root.len());
        assert_eq!(
            osatura_add_bone(e, cs("tip").as_ptr(), root.as_ptr(), 10.0, 5.0, 0.0, 0.0),
            0
        );
        assert_eq!(osatura_bone_count(e), b0 + 1);
        assert_eq!(
            osatura_add_bone(
                e,
                cs("tip2").as_ptr(),
                cs("ghost").as_ptr(),
                10.0,
                0.0,
                0.0,
                0.0
            ),
            -1
        );
        assert_eq!(
            osatura_set_key(
                e,
                cs("tip").as_ptr(),
                cs("bone_angle").as_ptr(),
                8.0,
                25.0,
                4
            ),
            0
        );
        assert_eq!(
            osatura_set_key(
                e,
                cs("tip").as_ptr(),
                cs("bone_angle").as_ptr(),
                8.0,
                25.0,
                9
            ),
            -1
        );
        assert_eq!(
            osatura_set_key(e, cs("tip").as_ptr(), cs("nope").as_ptr(), 8.0, 25.0, 0),
            -1
        );

        let mut m = vec![0f32; (b0 as usize + 1) * 6];
        assert_eq!(osatura_bone_world(e, 8.0, m.as_mut_ptr(), m.len()), 0);

        // undo the key, the bone and the three layers
        for _ in 0..5 {
            assert_eq!(osatura_undo(e), 1);
        }
        assert_eq!(osatura_layer_count(e), l0);
        assert_eq!(osatura_bone_count(e), b0);
        assert_eq!(osatura_undo(e), 0, "nothing left to undo");
        assert_eq!(osatura_redo(e), 1);
        assert_eq!(osatura_layer_count(e), l0 + 1);
        assert_eq!(osatura_undo(std::ptr::null_mut()), -1);
        osatura_close(e);
    }
}

#[test]
fn export_to_bytes_and_physics() {
    let j = demo_json();
    unsafe {
        let e = osatura_open(j.as_ptr() as *const _, j.len());
        assert!(!e.is_null());
        let gif = take(osatura_export(e, 0, 0.25));
        assert_eq!(&gif[0..3], b"GIF");
        let glb = take(osatura_export(e, 1, 1.0));
        assert_eq!(&glb[0..4], b"glTF");
        let lottie = take(osatura_export(e, 2, 1.0));
        assert!(std::str::from_utf8(&lottie).unwrap().starts_with('{'));
        let html = take(osatura_export(e, 3, 1.0));
        assert!(
            std::str::from_utf8(&html).unwrap().contains("<html")
                || std::str::from_utf8(&html).unwrap().contains("<!DOCTYPE")
                || std::str::from_utf8(&html).unwrap().contains("<!doctype")
        );
        let apng = take(osatura_export(e, 4, 0.25));
        assert_eq!(&apng[1..4], b"PNG");
        assert!(apng.windows(4).any(|w| w == b"acTL"));
        assert!(osatura_export(e, 99, 1.0).is_null());
        assert!(last_error().contains("unknown export format"));
        assert!(osatura_export(e, 0, 0.0).is_null());
        assert!(osatura_export(std::ptr::null(), 0, 1.0).is_null());

        let png = take(osatura_render_png(e, 2.0, 0.25));
        assert_eq!(&png[1..4], b"PNG");
        assert!(osatura_render_png(e, f32::NAN, 0.25).is_null());
        let half = take(osatura_render_exr(e, 2.0, 0.25, 0));
        let full = take(osatura_render_exr(e, 2.0, 0.25, 1));
        assert_eq!(&half[0..4], &[0x76, 0x2f, 0x31, 0x01]);
        assert!(full.len() > half.len());
        let json = take(osatura_to_json(e));
        assert!(std::str::from_utf8(&json).unwrap().contains("\"bones\""));
        // the JSON reopens
        let e2 = osatura_open(json.as_ptr() as *const _, json.len());
        assert!(!e2.is_null(), "{}", last_error());
        osatura_close(e2);

        // NULL blob is harmless
        assert!(osatura_blob_data(std::ptr::null()).is_null());
        assert_eq!(osatura_blob_size(std::ptr::null()), 0);
        osatura_blob_free(std::ptr::null_mut());

        // physics bake
        assert_eq!(
            osatura_add_bone(
                e,
                cs("h1").as_ptr(),
                std::ptr::null(),
                20.0,
                10.0,
                10.0,
                90.0
            ),
            0
        );
        assert_eq!(
            osatura_add_bone(
                e,
                cs("h2").as_ptr(),
                cs("h1").as_ptr(),
                20.0,
                20.0,
                0.0,
                0.0
            ),
            0
        );
        assert_eq!(
            osatura_bake_physics(e, cs("h1, h2").as_ptr(), cs("hair").as_ptr(), 0),
            2
        );
        assert_eq!(
            osatura_bake_physics(e, cs("h1,h2").as_ptr(), cs("hair").as_ptr(), 0),
            0
        );
        assert_eq!(
            osatura_bake_physics(e, cs("h1,h2").as_ptr(), cs("lava").as_ptr(), 0),
            -1
        );
        assert_eq!(
            osatura_bake_physics(e, cs("").as_ptr(), cs("hair").as_ptr(), 0),
            -1
        );
        assert_eq!(
            osatura_bake_physics(e, cs("ghost").as_ptr(), cs("hair").as_ptr(), 0),
            -1
        );
        osatura_close(e);
    }
}
