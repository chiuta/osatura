// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `containers`.
#![allow(unused_imports)]

use super::*;
use crate::anim::{Ease, Track};
use crate::edit::Editor;
use crate::engine::Engine;
use crate::path::VPath;
use crate::project::{Channel, Prop};

fn rect_layer(n: &str, w: f32, h: f32) -> Layer {
    Layer::vector(n, vec![VPath::rect("r", 0.0, 0.0, w, h).with_fill("#ff0000")])
}

fn proj(sizes: &[(f32, f32)]) -> Project {
    let mut p = Project::new("c", 400.0, 200.0, 24.0, 5);
    p.layers = sizes.iter().enumerate().map(|(i, (w, h))| rect_layer(&format!("l{i}"), *w, *h)).collect();
    p
}

fn row(p: &Project, n: usize) -> Container {
    Container::new("c", Dir::Row, (0..n).map(|i| Member::layer(&format!("l{i}"))).collect())
        .tap(|c| {
            let _ = p;
            c.rect = RectRule::default();
        })
}

trait Tap: Sized {
    fn tap(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}
impl Tap for Container {}

fn xs(s: &Solution) -> Vec<f32> {
    s.layers.iter().map(|p| p.dx).collect()
}
fn ys(s: &Solution) -> Vec<f32> {
    s.layers.iter().map(|p| p.dy).collect()
}
fn near(a: &[f32], b: &[f32]) {
    assert_eq!(a.len(), b.len(), "{a:?} vs {b:?}");
    for (x, y) in a.iter().zip(b) {
        assert!((x - y).abs() < 1e-3, "{a:?} vs {b:?}");
    }
}

#[test]
fn row_with_padding_and_gap_is_exact() {
    let mut p = proj(&[(50.0, 20.0), (60.0, 20.0), (70.0, 20.0)]);
    let mut c = row(&p, 3);
    c.padding = [10.0; 4];
    c.gap = 10.0;
    p.containers.push(c);
    let s = solve_project(&p).unwrap();
    near(&xs(&s), &[10.0, 70.0, 140.0]);
    near(&ys(&s), &[10.0, 10.0, 10.0]);
    assert_eq!(s.rects[0].1, [0.0, 0.0, 400.0, 200.0]);
}

#[test]
fn column_distribution() {
    let mut p = proj(&[(50.0, 20.0), (60.0, 30.0)]);
    let mut c = row(&p, 2);
    c.axis = Dir::Column;
    c.gap = 5.0;
    p.containers.push(c);
    let s = solve_project(&p).unwrap();
    near(&xs(&s), &[0.0, 0.0]);
    near(&ys(&s), &[0.0, 25.0]);
}

#[test]
fn justify_modes() {
    let mk = |j: Justify| {
        let mut p = proj(&[(50.0, 20.0), (60.0, 20.0), (70.0, 20.0)]);
        let mut c = row(&p, 3);
        c.justify = j;
        p.containers.push(c);
        xs(&solve_project(&p).unwrap())
    };
    near(&mk(Justify::Start), &[0.0, 50.0, 110.0]);
    near(&mk(Justify::Center), &[110.0, 160.0, 220.0]);
    near(&mk(Justify::End), &[220.0, 270.0, 330.0]);
    near(&mk(Justify::SpaceBetween), &[0.0, 160.0, 330.0]);
    let third = 220.0 / 3.0;
    near(&mk(Justify::SpaceAround), &[third / 2.0, third / 2.0 + 50.0 + third, third / 2.0 + 110.0 + 2.0 * third]);
    // one item: space_between falls back to start
    let mut p = proj(&[(50.0, 20.0)]);
    let mut c = row(&p, 1);
    c.justify = Justify::SpaceBetween;
    p.containers.push(c);
    near(&xs(&solve_project(&p).unwrap()), &[0.0]);
}

#[test]
fn cross_axis_alignment_and_stretch() {
    let mk = |a: Align, resize: bool| {
        let mut p = proj(&[(50.0, 20.0)]);
        let mut c = row(&p, 1);
        c.align = a;
        c.members[0].resize = resize;
        p.containers.push(c);
        solve_project(&p).unwrap().layers[0].clone()
    };
    assert!((mk(Align::Start, false).dy - 0.0).abs() < 1e-4);
    assert!((mk(Align::Center, false).dy - 90.0).abs() < 1e-4);
    assert!((mk(Align::End, false).dy - 180.0).abs() < 1e-4);
    let st = mk(Align::Stretch, true);
    assert!((st.ky - 10.0).abs() < 1e-4 && (st.kx - 1.0).abs() < 1e-6 && st.dy.abs() < 1e-4);
    // stretch without resize leaves the layer alone
    let ns = mk(Align::Stretch, false);
    assert!((ns.ky - 1.0).abs() < 1e-6);
}

#[test]
fn member_align_overrides_container() {
    let mut p = proj(&[(50.0, 20.0), (50.0, 20.0)]);
    let mut c = row(&p, 2);
    c.members[1].align = Some(Align::End);
    p.containers.push(c);
    near(&ys(&solve_project(&p).unwrap()), &[0.0, 180.0]);
}

#[test]
fn grow_shares_free_space_by_weight() {
    let mut p = proj(&[(50.0, 20.0), (60.0, 20.0), (70.0, 20.0)]);
    let mut c = row(&p, 3);
    c.members[0].grow = 1.0;
    c.members[2].grow = 2.0;
    for m in &mut c.members {
        m.resize = true;
    }
    p.containers.push(c);
    let s = solve_project(&p).unwrap();
    let w0 = 50.0 + 220.0 / 3.0;
    let w2 = 70.0 + 440.0 / 3.0;
    near(&xs(&s), &[0.0, w0, w0 + 60.0]);
    near(&s.layers.iter().map(|l| l.kx).collect::<Vec<_>>(), &[w0 / 50.0, 1.0, w2 / 70.0]);
}

#[test]
fn grow_respects_max_and_redistributes() {
    let mut p = proj(&[(50.0, 20.0), (50.0, 20.0)]);
    let mut c = row(&p, 2);
    p.width = 300.0;
    for m in &mut c.members {
        m.grow = 1.0;
        m.resize = true;
    }
    c.members[0].max_w = Some(100.0);
    p.containers.push(c);
    let s = solve_project(&p).unwrap();
    near(&xs(&s), &[0.0, 100.0]);
    near(&s.layers.iter().map(|l| l.kx).collect::<Vec<_>>(), &[2.0, 4.0]);
}

#[test]
fn shrink_is_weighted_by_size_and_min_freezes() {
    let mut p = proj(&[(50.0, 20.0), (60.0, 20.0), (70.0, 20.0)]);
    p.width = 100.0;
    let mut c = row(&p, 3);
    for m in &mut c.members {
        m.resize = true;
    }
    p.containers.push(c.clone());
    let s = solve_project(&p).unwrap();
    let k = 100.0 / 180.0;
    near(&s.layers.iter().map(|l| l.kx).collect::<Vec<_>>(), &[k, k, k]);
    near(&xs(&s), &[0.0, 50.0 * k, 110.0 * k]);
    // middle member refuses to shrink below 55
    p.containers[0].members[1].min_w = Some(55.0);
    let s = solve_project(&p).unwrap();
    near(&xs(&s), &[0.0, 18.75, 18.75 + 55.0]);
    // a member with shrink 0 keeps its size
    p.containers[0].members[1].min_w = None;
    p.containers[0].members[0].shrink = 0.0;
    let s = solve_project(&p).unwrap();
    assert!((s.layers[0].kx - 1.0).abs() < 1e-6);
    assert!((xs(&s)[1] - 50.0).abs() < 1e-3);
}

#[test]
fn wrap_breaks_lines() {
    let mut p = proj(&[(40.0, 20.0); 4]);
    p.width = 100.0;
    let mut c = row(&p, 4);
    c.wrap = Wrap::Wrap;
    c.gap = 10.0;
    p.containers.push(c);
    let s = solve_project(&p).unwrap();
    near(&xs(&s), &[0.0, 50.0, 0.0, 50.0]);
    near(&ys(&s), &[0.0, 0.0, 30.0, 30.0]);
    // separate line gap
    p.containers[0].cross_gap = Some(2.0);
    near(&ys(&solve_project(&p).unwrap()), &[0.0, 0.0, 22.0, 22.0]);
}

#[test]
fn nested_containers() {
    let mut p = proj(&[(50.0, 20.0), (50.0, 20.0), (30.0, 40.0)]);
    let mut inner = Container::new("in", Dir::Row, vec![Member::layer("l0"), Member::layer("l1")]);
    inner.gap = 5.0;
    let mut outer = Container::new("out", Dir::Column, vec![Member::nested("in"), Member::layer("l2")]);
    outer.gap = 10.0;
    p.containers = vec![inner, outer.clone()];
    let s = solve_project(&p).unwrap();
    near(&xs(&s), &[0.0, 55.0, 0.0]);
    near(&ys(&s), &[0.0, 0.0, 30.0]);
    assert_eq!(s.rects.iter().find(|r| r.0 == "in").unwrap().1, [0.0, 0.0, 105.0, 20.0]);
    // centred: the intrinsic nested box is centred in the outer width
    p.containers[1].align = Align::Center;
    let s = solve_project(&p).unwrap();
    near(&xs(&s), &[147.5, 202.5, 185.0]);
    // nested stretch gives the inner container the full width, so space_between spreads it
    p.containers[1].align = Align::Stretch;
    p.containers[0].justify = Justify::SpaceBetween;
    let s = solve_project(&p).unwrap();
    near(&xs(&s), &[0.0, 350.0, 0.0]);
}

#[test]
fn spacers_push_members_apart() {
    let mut p = proj(&[(50.0, 20.0), (50.0, 20.0)]);
    let mut c = row(&p, 2);
    c.members.insert(1, Member::spacer(1.0));
    p.containers.push(c);
    near(&xs(&solve_project(&p).unwrap()), &[0.0, 350.0]);
}

#[test]
fn rect_rules_follow_the_canvas() {
    let pin = RectRule::Pin { left: Some(10.0), top: Some(20.0), right: Some(30.0), bottom: None, width: None, height: Some(50.0) };
    assert_eq!(resolve_rule(&pin, 400.0, 200.0, [0.0; 2]), [10.0, 20.0, 360.0, 50.0]);
    assert_eq!(resolve_rule(&pin, 800.0, 600.0, [0.0; 2]), [10.0, 20.0, 760.0, 50.0]);
    let br = RectRule::Pin { left: None, top: None, right: Some(10.0), bottom: Some(10.0), width: Some(100.0), height: Some(40.0) };
    assert_eq!(resolve_rule(&br, 400.0, 200.0, [0.0; 2]), [290.0, 150.0, 100.0, 40.0]);
    let ctr = RectRule::Pin { left: None, top: None, right: None, bottom: None, width: Some(100.0), height: Some(100.0) };
    assert_eq!(resolve_rule(&ctr, 400.0, 200.0, [0.0; 2]), [150.0, 50.0, 100.0, 100.0]);
    let fit = RectRule::Aspect { ratio: 16.0 / 9.0, mode: AspectMode::Fit, margin: 0.0, ax: 0.5, ay: 0.5 };
    let r = resolve_rule(&fit, 400.0, 400.0, [0.0; 2]);
    assert!((r[2] - 400.0).abs() < 1e-3 && (r[3] - 225.0).abs() < 1e-3 && (r[1] - 87.5).abs() < 1e-3);
    let fill = RectRule::Aspect { ratio: 16.0 / 9.0, mode: AspectMode::Fill, margin: 0.0, ax: 0.5, ay: 0.5 };
    let r = resolve_rule(&fill, 400.0, 400.0, [0.0; 2]);
    assert!((r[3] - 400.0).abs() < 1e-3 && (r[2] - 711.111).abs() < 1e-2 && (r[0] + 155.5555).abs() < 1e-2);
    let m = RectRule::Aspect { ratio: 1.0, mode: AspectMode::Fit, margin: 10.0, ax: 0.0, ay: 0.0 };
    assert_eq!(resolve_rule(&m, 400.0, 200.0, [0.0; 2]), [10.0, 10.0, 180.0, 180.0]);
    // margins larger than the canvas clamp to zero size
    let big = RectRule::Pin { left: Some(300.0), top: None, right: Some(300.0), bottom: None, width: None, height: None };
    assert_eq!(resolve_rule(&big, 400.0, 200.0, [0.0; 2])[2], 0.0);
}

#[test]
fn container_reflows_when_the_canvas_changes() {
    let mut p = proj(&[(50.0, 20.0), (50.0, 20.0)]);
    let mut c = row(&p, 2);
    c.justify = Justify::SpaceBetween;
    c.align = Align::End;
    c.rect = RectRule::Pin { left: Some(10.0), right: Some(10.0), top: None, bottom: Some(10.0), width: None, height: Some(40.0) };
    p.containers.push(c);
    let a = solve_project(&p).unwrap();
    near(&xs(&a), &[10.0, 340.0]);
    near(&ys(&a), &[170.0, 170.0]);
    let q = responsive(&p, 800.0, 600.0).unwrap();
    let b = solve_project(&q).unwrap();
    near(&xs(&b), &[10.0, 740.0]);
    near(&ys(&b), &[570.0, 570.0]);
    // the source project is untouched
    assert_eq!((p.width, p.height), (400.0, 200.0));
    assert!(responsive(&p, 0.0, 10.0).is_err() && responsive(&p, f32::NAN, 10.0).is_err());
    assert_eq!(parse_size("800x600").unwrap(), (800.0, 600.0));
    assert!(parse_size("800").is_err() && parse_size("0x5").is_err() && parse_size("axb").is_err());
    // an engine compiled from the responsive copy renders members at the new place
    let e = Engine::new(q).unwrap();
    let f = e.evaluate(0.0);
    let id = e.node_index("l1").unwrap();
    let o = f.nodes[id].world.apply(Vec2::ZERO);
    let e0 = Engine::new(p.clone()).unwrap();
    let o0 = e0.evaluate(0.0).nodes[e0.node_index("l1").unwrap()].world.apply(Vec2::ZERO);
    assert!((o.x - o0.x - 400.0).abs() < 1e-2 && (o.y - o0.y - 400.0).abs() < 1e-2, "{o:?} {o0:?}");
}

#[test]
fn validation_errors() {
    let base = proj(&[(50.0, 20.0), (50.0, 20.0)]);
    let bad = |f: &dyn Fn(&mut Project)| {
        let mut p = base.clone();
        f(&mut p);
        validate(&p).join("\n")
    };
    assert!(bad(&|p| p.containers.push(Container::new("c", Dir::Row, vec![Member::layer("ghost")]))).contains("unknown layer"));
    assert!(bad(&|p| {
        p.containers.push(Container::new("a", Dir::Row, vec![Member::layer("l0")]));
        p.containers.push(Container::new("b", Dir::Row, vec![Member::layer("l0")]));
    })
    .contains("both container"));
    assert!(bad(&|p| p.containers.push(Container::new("a", Dir::Row, vec![Member::layer("l0"), Member::layer("l0")]))).contains("both container"));
    assert!(bad(&|p| p.containers.push(Container::new("a", Dir::Row, vec![Member::nested("a")]))).contains("cycle"));
    assert!(bad(&|p| {
        p.containers.push(Container::new("a", Dir::Row, vec![Member::nested("b")]));
        p.containers.push(Container::new("b", Dir::Row, vec![Member::nested("a")]));
    })
    .contains("cycle"));
    assert!(bad(&|p| {
        p.containers.push(Container::new("a", Dir::Row, vec![]));
        p.containers.push(Container::new("a", Dir::Row, vec![]));
    })
    .contains("duplicate container"));
    let mut m = Member::layer("l0");
    m.width = Some(0.0);
    assert!(bad(&|p| p.containers.push(Container::new("a", Dir::Row, vec![m.clone()]))).contains("width must be"));
    let mut m = Member::layer("l0");
    m.height = Some(-3.0);
    assert!(bad(&|p| p.containers.push(Container::new("a", Dir::Row, vec![m.clone()]))).contains("height must be"));
    let mut m = Member::layer("l0");
    m.grow = f32::NAN;
    assert!(bad(&|p| p.containers.push(Container::new("a", Dir::Row, vec![m.clone()]))).contains("grow"));
    let mut m = Member::layer("l0");
    m.min_w = Some(10.0);
    m.max_w = Some(5.0);
    assert!(bad(&|p| p.containers.push(Container::new("a", Dir::Row, vec![m.clone()]))).contains("larger than max"));
    assert!(bad(&|p| {
        let mut c = Container::new("a", Dir::Row, vec![]);
        c.gap = f32::NAN;
        p.containers.push(c);
    })
    .contains("gap"));
    assert!(bad(&|p| {
        let mut c = Container::new("a", Dir::Row, vec![]);
        c.padding = [0.0, -1.0, 0.0, 0.0];
        p.containers.push(c);
    })
    .contains("padding"));
    assert!(bad(&|p| {
        let mut c = Container::new("a", Dir::Row, vec![]);
        c.rect = RectRule::Fixed { x: 0.0, y: 0.0, w: 0.0, h: 10.0 };
        p.containers.push(c);
    })
    .contains("fixed rect"));
    assert!(bad(&|p| {
        let mut c = Container::new("a", Dir::Row, vec![]);
        c.rect = RectRule::Pin { left: Some(1.0), right: Some(1.0), width: Some(5.0), top: None, bottom: None, height: None };
        p.containers.push(c);
    })
    .contains("over-constrained"));
    assert!(bad(&|p| {
        let mut c = Container::new("a", Dir::Row, vec![]);
        c.rect = RectRule::Aspect { ratio: 0.0, mode: AspectMode::Fit, margin: 0.0, ax: 0.5, ay: 0.5 };
        p.containers.push(c);
    })
    .contains("aspect ratio"));
    assert!(bad(&|p| p.containers.push(Container::new("a", Dir::Row, vec![Member { layer: None, ..Member::spacer(0.0) }.tap_none()]))).contains("exactly one"));
    // unmeasurable layer needs an explicit size
    assert!(bad(&|p| {
        p.layers.push(Layer::group("g", vec![]));
        p.containers.push(Container::new("a", Dir::Row, vec![Member::layer("g")]));
    })
    .contains("no measurable geometry"));
    // resize on a rotated layer
    assert!(bad(&|p| {
        p.layers[0].transform.rotation = 10.0;
        let mut m = Member::layer("l0");
        m.resize = true;
        p.containers.push(Container::new("a", Dir::Row, vec![m]));
    })
    .contains("resize needs"));
    // solve refuses invalid input rather than panicking
    let mut p = base.clone();
    p.containers.push(Container::new("a", Dir::Row, vec![Member::nested("a")]));
    assert!(solve_project(&p).is_err());
    assert!(solve(&base, f32::NAN, 5.0).is_err());
    // the engine refuses it too
    assert!(Engine::new(p).is_err());
}

trait TapNone {
    fn tap_none(self) -> Self;
}
impl TapNone for Member {
    fn tap_none(mut self) -> Self {
        self.spacer = false;
        self
    }
}

fn scroll_project() -> Project {
    let mut p = proj(&[(100.0, 20.0); 10]);
    let mut c = row(&p, 10);
    c.axis = Dir::Column;
    p.containers.push(c);
    let mut s = Scroll::new("s", "c");
    s.viewport = RectRule::Fixed { x: 0.0, y: 0.0, w: 100.0, h: 50.0 };
    p.scrolls.push(s);
    p
}

#[test]
fn scroll_clamps_offset() {
    let mut p = scroll_project();
    let s = solve_project(&p).unwrap();
    assert!((s.scrolls[0].max[1] - 150.0).abs() < 1e-3);
    near(&ys(&s)[..3], &[0.0, 20.0, 40.0]);
    set_offset(&mut p, "s", [30.0, 80.0]).unwrap();
    let s = solve_project(&p).unwrap();
    assert_eq!(s.scrolls[0].offset, [0.0, 80.0], "x does not scroll on a vertical region");
    near(&ys(&s)[..3], &[-80.0, -60.0, -40.0]);
    set_offset(&mut p, "s", [0.0, 500.0]).unwrap();
    assert!((solve_project(&p).unwrap().scrolls[0].offset[1] - 150.0).abs() < 1e-3);
    set_offset(&mut p, "s", [0.0, -20.0]).unwrap();
    assert_eq!(solve_project(&p).unwrap().scrolls[0].offset, [0.0, 0.0]);
    assert!(set_offset(&mut p, "s", [f32::NAN, 0.0]).is_err() && set_offset(&mut p, "zz", [0.0; 2]).is_err());
    // content shorter than the viewport cannot scroll
    let mut q = proj(&[(100.0, 20.0)]);
    q.containers.push(row(&q, 1));
    let mut sc = Scroll::new("s", "c");
    sc.viewport = RectRule::Fixed { x: 0.0, y: 0.0, w: 100.0, h: 50.0 };
    sc.offset = [0.0, 40.0];
    q.scrolls.push(sc);
    assert_eq!(solve_project(&q).unwrap().scrolls[0].max, [0.0, 0.0]);
}

#[test]
fn scroll_horizontal_and_both_axes() {
    let mut p = proj(&[(100.0, 20.0); 4]);
    p.containers.push(row(&p, 4));
    let mut s = Scroll::new("s", "c");
    s.axis = ScrollAxis::X;
    s.viewport = RectRule::Fixed { x: 10.0, y: 5.0, w: 150.0, h: 50.0 };
    s.offset = [60.0, 99.0];
    p.scrolls.push(s);
    let r = solve_project(&p).unwrap();
    assert_eq!(r.scrolls[0].max, [250.0, 0.0]);
    near(&xs(&r), &[10.0 - 60.0, 50.0, 150.0, 250.0].map(|v: f32| v - 0.0).iter().enumerate().map(|(i, _)| 10.0 - 60.0 + 100.0 * i as f32).collect::<Vec<_>>());
    near(&ys(&r), &[5.0; 4]);
    p.scrolls[0].axis = ScrollAxis::Both;
    p.scrolls[0].offset = [60.0, 99.0];
    let r = solve_project(&p).unwrap();
    assert_eq!(r.scrolls[0].offset, [60.0, 0.0], "content is not taller than the viewport");
}

#[test]
fn scroll_wraps_to_the_viewport_width() {
    let mut p = proj(&[(40.0, 20.0); 6]);
    let mut c = row(&p, 6);
    c.wrap = Wrap::Wrap;
    c.gap = 10.0;
    p.containers.push(c);
    let mut s = Scroll::new("s", "c");
    s.viewport = RectRule::Fixed { x: 0.0, y: 0.0, w: 100.0, h: 40.0 };
    p.scrolls.push(s);
    let r = solve_project(&p).unwrap();
    // two per line, three lines of 20 with gaps of 10: height 80, so max offset 40
    assert_eq!(r.scrolls[0].max, [0.0, 40.0]);
    near(&ys(&r), &[0.0, 0.0, 30.0, 30.0, 60.0, 60.0]);
}

#[test]
fn scroll_elastic_overscroll_and_runtime() {
    let mut p = scroll_project();
    p.scrolls[0].elastic = Some(Elastic { limit: 40.0, stiffness: 180.0, damping_ratio: 0.9 });
    set_offset(&mut p, "s", [0.0, 250.0]).unwrap();
    let o = solve_project(&p).unwrap().scrolls[0].offset[1];
    let want = 150.0 + 40.0 * (1.0 - (-100.0f32 / 40.0).exp());
    assert!((o - want).abs() < 1e-3 && o < 190.0 && o > 150.0, "{o}");
    set_offset(&mut p, "s", [0.0, -30.0]).unwrap();
    let o = solve_project(&p).unwrap().scrolls[0].offset[1];
    assert!(o < 0.0 && o > -30.0, "{o}");
    // monotone saturation
    let a = effective_offset(200.0, 150.0, p.scrolls[0].elastic.as_ref());
    let b = effective_offset(400.0, 150.0, p.scrolls[0].elastic.as_ref());
    assert!(a < b && b < 190.0);
    // runtime: drag past the end, let go, spring back to the clamp and stop exactly
    let mut rt = ScrollRuntime::new(&p.scrolls[0], [0.0, 150.0]);
    rt.drag_to([0.0, 300.0]);
    assert!(rt.pos[1] > 150.0);
    rt.release([0.0, 0.0]);
    for _ in 0..600 {
        rt.step(1.0 / 60.0);
    }
    assert!(rt.is_settled());
    assert_eq!(rt.pos, [0.0, 150.0]);
    // determinism
    let run = || {
        let mut r = ScrollRuntime::new(&p.scrolls[0], [0.0, 150.0]);
        r.drag_to([0.0, -80.0]);
        r.release([0.0, 0.0]);
        let mut v = Vec::new();
        for _ in 0..30 {
            r.step(1.0 / 60.0);
            v.push(r.pos[1].to_bits());
        }
        v
    };
    assert_eq!(run(), run());
    // without elastic the drag is clamped
    p.scrolls[0].elastic = None;
    let mut rt = ScrollRuntime::new(&p.scrolls[0], [0.0, 150.0]);
    rt.drag_to([0.0, 999.0]);
    assert_eq!(rt.pos[1], 150.0);
}

#[test]
fn raw_offset_inverts_effective_offset_and_set_max_reclamps() {
    let el = Elastic { limit: 40.0, stiffness: 180.0, damping_ratio: 0.9 };
    for raw in [-90.0f32, -5.0, 0.0, 70.0, 150.0, 151.0, 260.0, 400.0] {
        let shown = effective_offset(raw, 150.0, Some(&el));
        let back = raw_offset_for(shown, 150.0, Some(&el));
        assert!((back - raw).abs() < 0.05 * (1.0 + raw.abs()), "{raw} -> {shown} -> {back}");
    }
    assert_eq!(raw_offset_for(-3.0, 150.0, None), 0.0);
    assert_eq!(raw_offset_for(500.0, 150.0, None), 150.0);
    // a saturated value maps to a finite raw offset
    assert!(raw_offset_for(1e9, 150.0, Some(&el)).is_finite());
    let p = scroll_project();
    let mut rt = ScrollRuntime::new(&p.scrolls[0], [0.0, 150.0]);
    rt.drag_to([0.0, 140.0]);
    rt.set_max([0.0, 100.0]);
    assert_eq!(rt.pos[1], 100.0);
    assert_eq!(rt.max(), [0.0, 100.0]);
    rt.set_max([0.0, 300.0]);
    assert_eq!(rt.pos[1], 100.0);
}

#[test]
fn scroll_snap_points_and_momentum() {
    let mut p = scroll_project();
    p.scrolls[0].snap_y = vec![0.0, 50.0, 100.0, 400.0];
    assert_eq!(snap_to(70.0, &p.scrolls[0].snap_y, 150.0), 50.0);
    assert_eq!(snap_to(80.0, &p.scrolls[0].snap_y, 150.0), 100.0);
    assert_eq!(snap_to(140.0, &p.scrolls[0].snap_y, 150.0), 150.0, "out of range snap points clamp to the end");
    assert_eq!(snap_to(33.0, &[], 150.0), 33.0);
    let mut rt = ScrollRuntime::new(&p.scrolls[0], [0.0, 150.0]);
    rt.drag_to([0.0, 70.0]);
    rt.release([0.0, 0.0]);
    for _ in 0..600 {
        rt.step(1.0 / 60.0);
    }
    assert_eq!(rt.pos[1], 50.0);
    // a fast flick carries over to the next snap point: 70 + 300 * 0.15 = 115 -> 100
    rt.drag_to([0.0, 70.0]);
    rt.release([0.0, 300.0]);
    for _ in 0..600 {
        rt.step(1.0 / 60.0);
    }
    assert_eq!(rt.pos[1], 100.0);
    // the x axis is inert on a vertical region
    rt.drag_to([40.0, 10.0]);
    assert_eq!(rt.pos[0], 0.0);
}

#[test]
fn scroll_clip_layer_masks_content_and_follows_the_viewport() {
    let mut p = scroll_project();
    let mut clip = rect_layer("clip", 10.0, 10.0);
    clip.visible = false;
    p.layers.push(clip);
    p.scrolls[0].clip_layer = Some("clip".into());
    p.scrolls[0].viewport = RectRule::Pin { left: Some(20.0), right: Some(20.0), top: Some(10.0), bottom: None, width: None, height: Some(50.0) };
    let e = Engine::new(p.clone()).unwrap();
    let f = e.evaluate(0.0);
    let cid = e.node_index("clip").unwrap();
    for i in 0..10 {
        let n = &f.nodes[e.node_index(&format!("l{i}")).unwrap()];
        assert_eq!(n.mask, Some((cid, false)));
    }
    // clip layer now covers the viewport: origin (20, 10), scaled 360 x 50 from 10 x 10
    let s = solve_project(&p).unwrap();
    let c = s.layers.iter().find(|l| l.layer == "clip").unwrap();
    near(&[c.dx, c.dy, c.kx, c.ky], &[20.0, 10.0, 36.0, 5.0]);
    // a content layer that is already masked is rejected
    let mut q = p.clone();
    q.layers.push(rect_layer("other", 5.0, 5.0));
    q.layers[0].mask = Some(MaskRef { layer: "other".into(), invert: false });
    assert!(validate(&q).join("\n").contains("already has a mask"));
    // the clip layer must not be a container member or part of the content
    let mut q = p.clone();
    q.containers[0].members.push(Member::layer("clip"));
    assert!(!validate(&q).is_empty());
}

#[test]
fn scroll_validation() {
    let mut p = scroll_project();
    p.scrolls.push(Scroll::new("s", "c"));
    assert!(validate(&p).join("\n").contains("duplicate scroll"));
    let mut p = scroll_project();
    p.scrolls[0].content = "nope".into();
    assert!(validate(&p).join("\n").contains("unknown content"));
    let mut p = scroll_project();
    p.scrolls.push(Scroll::new("t", "c"));
    assert!(validate(&p).join("\n").contains("content of both"));
    let mut p = scroll_project();
    p.containers.push(Container::new("o", Dir::Row, vec![Member::nested("c")]));
    assert!(validate(&p).join("\n").contains("nested in another"));
    let mut p = scroll_project();
    p.scrolls[0].elastic = Some(Elastic { limit: 0.0, stiffness: 1.0, damping_ratio: 1.0 });
    assert!(validate(&p).join("\n").contains("elastic"));
    let mut p = scroll_project();
    p.scrolls[0].offset = [f32::INFINITY, 0.0];
    assert!(validate(&p).join("\n").contains("offset"));
    let mut p = scroll_project();
    p.scrolls[0].snap_y = vec![f32::NAN];
    assert!(validate(&p).join("\n").contains("snap"));
    let mut p = scroll_project();
    p.scrolls[0].clip_layer = Some("ghost".into());
    assert!(validate(&p).join("\n").contains("unknown clip"));
    let mut p = scroll_project();
    p.scrolls[0].viewport = RectRule::Hug { x: 0.0, y: 0.0 };
    assert!(validate(&p).join("\n").contains("hug"));
    assert!(Engine::new(p).is_err());
}

#[test]
fn scroll_input_values_drive_offsets() {
    let mut p = scroll_project();
    p.scrolls[0].input = Some("page.scroll".into());
    let mut v = HashMap::new();
    v.insert("page.scroll".to_string(), 60.0);
    let q = apply_inputs(&p, &v);
    assert_eq!(q.scrolls[0].offset, [0.0, 60.0]);
    near(&ys(&solve_project(&q).unwrap())[..2], &[-60.0, -40.0]);
    v.insert("page.scroll".to_string(), f64::NAN);
    assert_eq!(apply_inputs(&p, &v).scrolls[0].offset, [0.0, 0.0]);
}

#[test]
fn layout_is_additive_to_animation_in_the_engine() {
    let mut p = proj(&[(50.0, 20.0), (50.0, 20.0)]);
    let mut c = row(&p, 2);
    c.justify = Justify::End;
    p.containers.push(c);
    p.channels.push(Channel::keys("l0", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (4.0, 40.0, Ease::Linear)])));
    let e = Engine::new(p.clone()).unwrap();
    let at = |f: f32, name: &str| {
        let fr = e.evaluate(f);
        fr.nodes[e.node_index(name).unwrap()].world.apply(Vec2::ZERO)
    };
    // reference: same animation without containers
    let mut plain = p.clone();
    plain.containers.clear();
    let ep = Engine::new(plain).unwrap();
    let atp = |f: f32, name: &str| ep.evaluate(f).nodes[ep.node_index(name).unwrap()].world.apply(Vec2::ZERO);
    for f in [0.0, 2.0, 4.0] {
        let (a, b) = (at(f, "l0"), atp(f, "l0"));
        assert!((a.x - b.x - 300.0).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3, "frame {f}: {a:?} {b:?}");
        let (a, b) = (at(f, "l1"), atp(f, "l1"));
        assert!((a.x - b.x - 350.0).abs() < 1e-3);
    }
    // determinism of the compiled result
    let e2 = Engine::new(p).unwrap();
    assert_eq!(e2.evaluate(3.0).nodes[0].world, e.evaluate(3.0).nodes[0].world);
}

#[test]
fn resize_changes_the_rendered_size() {
    let mut p = proj(&[(50.0, 20.0)]);
    let mut c = row(&p, 1);
    c.members[0].resize = true;
    c.members[0].grow = 1.0;
    c.padding = [10.0, 10.0, 10.0, 10.0];
    c.align = Align::Stretch;
    p.containers.push(c);
    // anchor away from the origin and a non-unit static scale must not matter
    p.layers[0].transform.anchor = Vec2::new(25.0, 10.0);
    p.layers[0].transform.position = Vec2::new(100.0, 100.0);
    p.layers[0].transform.scale = Vec2::new(2.0, 1.0);
    let e = Engine::new(p).unwrap();
    let f = e.evaluate(0.0);
    let n = &f.nodes[e.node_index("l0").unwrap()];
    let a = n.world.apply(Vec2::new(0.0, 0.0));
    let b = n.world.apply(Vec2::new(50.0, 20.0));
    assert!((a.x - 10.0).abs() < 1e-2 && (a.y - 10.0).abs() < 1e-2, "{a:?}");
    assert!((b.x - 390.0).abs() < 1e-2 && (b.y - 190.0).abs() < 1e-2, "{b:?}");
}

#[test]
fn unmeasurable_layers_use_explicit_sizes() {
    let mut p = proj(&[(50.0, 20.0)]);
    p.layers.push(Layer::group("g", vec![]));
    p.layers[1].transform.position = Vec2::new(7.0, 3.0);
    let mut m = Member::layer("g");
    m.width = Some(30.0);
    m.height = Some(10.0);
    p.containers.push(Container::new("c", Dir::Row, vec![Member::layer("l0"), m]));
    let s = solve_project(&p).unwrap();
    near(&xs(&s), &[0.0, 50.0 - 7.0]);
    near(&ys(&s), &[0.0, -3.0]);
}

#[test]
fn editor_ops_are_undoable_and_atomic() {
    let p = proj(&[(50.0, 20.0), (50.0, 20.0)]);
    let mut ed = Editor::new(p).unwrap();
    let c = Container::new("c", Dir::Row, vec![Member::layer("l0"), Member::layer("l1")]);
    let op = add_container_op(ed.project(), c.clone()).unwrap();
    ed.apply(op).unwrap();
    assert_eq!(ed.project().containers.len(), 1);
    // second container goes through the append path
    let d = Container::new("d", Dir::Row, vec![]);
    ed.apply(add_container_op(ed.project(), d).unwrap()).unwrap();
    assert_eq!(ed.project().containers.len(), 2);
    assert!(ed.undo());
    assert_eq!(ed.project().containers.len(), 1);
    assert!(ed.undo());
    assert!(ed.project().containers.is_empty());
    assert!(ed.redo());
    assert_eq!(ed.project().containers[0], c);
    // duplicates and invalid containers never reach the editor
    assert!(add_container_op(ed.project(), c.clone()).is_err());
    assert!(add_container_op(ed.project(), Container::new("z", Dir::Row, vec![Member::layer("ghost")])).is_err());
    // an op that would leave a scroll dangling is rolled back by the engine check
    ed.apply(add_scroll_op(ed.project(), Scroll::new("s", "c")).unwrap()).unwrap();
    assert!(remove_container_op(ed.project(), "c").is_err());
    let dangling = crate::edit::Op::Remove { path: "/containers/0".into() };
    assert!(ed.apply(dangling).is_err());
    assert_eq!(ed.project().containers.len(), 1);
    ed.apply(remove_scroll_op(ed.project(), "s").unwrap()).unwrap();
    ed.apply(remove_container_op(ed.project(), "c").unwrap()).unwrap();
    assert!(ed.project().containers.is_empty());
    assert!(remove_container_op(ed.project(), "c").is_err());
    assert!(remove_scroll_op(ed.project(), "s").is_err());
}

#[test]
fn project_fields_are_omitted_when_empty_and_round_trip() {
    let mut p = proj(&[(50.0, 20.0)]);
    let json = p.to_json_pretty();
    assert!(!json.contains("containers") && !json.contains("scrolls"));
    p.containers.push(row(&p, 1));
    p.scrolls.push(Scroll::new("s", "c"));
    let q = Project::from_json(&p.to_json_pretty()).unwrap();
    assert_eq!(q.containers, p.containers);
    assert_eq!(q.scrolls, p.scrolls);
    // minimal hand-written JSON uses the defaults
    let c: Container = serde_json::from_str(r#"{"id":"x","members":[{"layer":"a","grow":1}],"justify":"space_between","rect":{"kind":"aspect","ratio":1.5}}"#).unwrap();
    assert_eq!(c.members[0].shrink, 1.0);
    assert_eq!(c.justify, Justify::SpaceBetween);
    assert!(serde_json::from_str::<Container>(r#"{"id":"x","members":[],"justify":"sideways"}"#).is_err());
}

#[test]
fn overlay_marks_every_rectangle() {
    let mut p = scroll_project();
    p.layers.push(rect_layer("extra", 5.0, 5.0));
    let s = solve_project(&p).unwrap();
    let svg = with_overlay("<svg></svg>", &s);
    assert!(svg.starts_with("<svg><g") && svg.ends_with("</g></svg>"));
    assert_eq!(svg.matches("<rect").count(), 2);
}
