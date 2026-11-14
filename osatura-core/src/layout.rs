// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Layout helpers (align, distribute) and a baked "trim path" operation.
//!
//! Plain geometry: bounding boxes of flattened curves, even spacing, arc-length sampling.

use crate::math::Vec2;
use crate::path::{Anchor, VPath};
use crate::project::{Layer, LayerKind, Project};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignMode {
    Left,
    Right,
    Top,
    Bottom,
    CenterX,
    CenterY,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Distribute {
    /// Equal gaps between neighbouring boxes.
    Gaps,
    /// Equal distance between box centres.
    Centers,
}

pub fn parse_align(s: &str) -> Option<AlignMode> {
    Some(match s {
        "left" => AlignMode::Left,
        "right" => AlignMode::Right,
        "top" => AlignMode::Top,
        "bottom" => AlignMode::Bottom,
        "center-x" => AlignMode::CenterX,
        "center-y" => AlignMode::CenterY,
        _ => return None,
    })
}

fn content_points(l: &Layer) -> Vec<Vec2> {
    let mut pts: Vec<Vec2> = Vec::new();
    match &l.kind {
        LayerKind::Vector { paths } => {
            for p in paths {
                pts.extend(p.flatten(0.5));
            }
        }
        LayerKind::Group { layers } => {
            for c in layers {
                if let Some((lo, hi)) = layer_bounds(c) {
                    pts.extend([lo, hi]);
                }
            }
        }
        _ => {}
    }
    pts
}

fn bbox_of(pts: impl Iterator<Item = Vec2>) -> Option<(Vec2, Vec2)> {
    let mut it = pts;
    let f = it.next()?;
    let (mut lo, mut hi) = (f, f);
    for p in it {
        lo = Vec2::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Vec2::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    Some((lo, hi))
}

/// Bounding box `(min, max)` of a layer's geometry in its parent's space (vector layers and
/// groups of them; other kinds have none).
pub fn layer_bounds(l: &Layer) -> Option<(Vec2, Vec2)> {
    let m = l.transform.matrix();
    bbox_of(content_points(l).into_iter().map(|p| m.apply(p)))
}

/// Bounding box of a layer's geometry in the layer's own coordinates (before its transform).
pub fn content_bounds(l: &Layer) -> Option<(Vec2, Vec2)> {
    bbox_of(content_points(l).into_iter())
}

fn get(p: &Project, name: &str) -> Result<(Vec2, Vec2), String> {
    let l = p
        .find_layer(name)
        .ok_or_else(|| format!("no layer '{name}'"))?;
    layer_bounds(l).ok_or_else(|| format!("layer '{name}' has no measurable geometry"))
}

fn shift(p: &mut Project, name: &str, d: Vec2) {
    if let Some(l) = p.find_layer_mut(name) {
        l.transform.position += d;
    }
}

/// Align layers to each other (`to_stage = false`: to the union box of the selection) or to the
/// stage rectangle. Moves the static layer position; keyed positions are not touched.
pub fn align(
    p: &mut Project,
    names: &[String],
    mode: AlignMode,
    to_stage: bool,
) -> Result<(), String> {
    if names.is_empty() || (!to_stage && names.len() < 2) {
        return Err("select at least two layers (or one with --stage)".into());
    }
    let boxes: Vec<(Vec2, Vec2)> = names.iter().map(|n| get(p, n)).collect::<Result<_, _>>()?;
    let (mut lo, mut hi) = if to_stage {
        (Vec2::ZERO, Vec2::new(p.width, p.height))
    } else {
        boxes[0]
    };
    if !to_stage {
        for (a, b) in &boxes {
            lo = Vec2::new(lo.x.min(a.x), lo.y.min(a.y));
            hi = Vec2::new(hi.x.max(b.x), hi.y.max(b.y));
        }
    }
    for (n, (a, b)) in names.iter().zip(boxes) {
        let d = match mode {
            AlignMode::Left => Vec2::new(lo.x - a.x, 0.0),
            AlignMode::Right => Vec2::new(hi.x - b.x, 0.0),
            AlignMode::Top => Vec2::new(0.0, lo.y - a.y),
            AlignMode::Bottom => Vec2::new(0.0, hi.y - b.y),
            AlignMode::CenterX => Vec2::new((lo.x + hi.x) * 0.5 - (a.x + b.x) * 0.5, 0.0),
            AlignMode::CenterY => Vec2::new(0.0, (lo.y + hi.y) * 0.5 - (a.y + b.y) * 0.5),
        };
        shift(p, n, d);
    }
    Ok(())
}

/// Spread layers evenly between the two outermost ones along an axis.
pub fn distribute(
    p: &mut Project,
    names: &[String],
    axis: Axis,
    how: Distribute,
) -> Result<(), String> {
    if names.len() < 3 {
        return Err("distribute needs at least three layers".into());
    }
    let c = |v: Vec2| if axis == Axis::X { v.x } else { v.y };
    let mut items: Vec<(String, f32, f32)> = Vec::new(); // name, min, max
    for n in names {
        let (a, b) = get(p, n)?;
        items.push((n.clone(), c(a), c(b)));
    }
    items.sort_by(|a, b| (a.1 + a.2).total_cmp(&(b.1 + b.2)));
    let k = items.len();
    let mut targets = vec![0.0f32; k]; // desired min
    match how {
        Distribute::Centers => {
            let c0 = (items[0].1 + items[0].2) * 0.5;
            let c1 = (items[k - 1].1 + items[k - 1].2) * 0.5;
            for (i, it) in items.iter().enumerate() {
                let want = c0 + (c1 - c0) * i as f32 / (k - 1) as f32;
                targets[i] = want - (it.2 - it.1) * 0.5;
            }
        }
        Distribute::Gaps => {
            let total_w: f32 = items.iter().map(|i| i.2 - i.1).sum();
            let span = items[k - 1].2 - items[0].1;
            let gap = (span - total_w) / (k - 1) as f32;
            let mut x = items[0].1;
            for (i, it) in items.iter().enumerate() {
                targets[i] = x;
                x += it.2 - it.1 + gap;
            }
        }
    }
    for (it, t) in items.iter().zip(targets) {
        let d = t - it.1;
        shift(
            p,
            &it.0,
            if axis == Axis::X {
                Vec2::new(d, 0.0)
            } else {
                Vec2::new(0.0, d)
            },
        );
    }
    Ok(())
}

/// Keep only the part of a path between `start` and `end` (fractions 0..1 of its length,
/// after shifting both by `offset`; closed paths wrap). Returns an open polyline path with
/// the same stroke and no fill. Bake it per frame for animated "draw-on" effects.
pub fn trim_path(
    src: &VPath,
    start: f32,
    end: f32,
    offset: f32,
    tolerance: f32,
) -> Result<VPath, String> {
    if ![start, end, offset].iter().all(|v| v.is_finite())
        || !(0.0..=1.0).contains(&start)
        || !(0.0..=1.0).contains(&end)
    {
        return Err("start and end must be within 0..1".into());
    }
    let pl = src.polyline(tolerance.max(0.01));
    let len = pl.length();
    if len <= 1e-6 {
        return Err("path has no length".into());
    }
    let (mut a, mut b) = (start.min(end) + offset, start.max(end) + offset);
    if !src.closed {
        a = a.clamp(0.0, 1.0);
        b = b.clamp(0.0, 1.0);
    }
    let (da, db) = (a * len, b * len);
    let n = (((db - da) / (tolerance.max(0.05) * 2.0)).ceil() as usize).clamp(1, 4000);
    let mut anchors = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let d = da + (db - da) * i as f32 / n as f32;
        anchors.push(Anchor::corner(pl.sample(d).0));
    }
    // keep original vertices inside the range so corners stay sharp
    let mut out = VPath::new(&src.name, false, anchors);
    out.stroke = src.stroke.clone();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proj() -> Project {
        let mut p = Project::new("l", 400.0, 200.0, 24.0, 5);
        let r = |n: &str, x: f32, y: f32, w: f32| {
            Layer::vector(
                n,
                vec![VPath::rect("r", x, y, w, 20.0).with_fill("#ff0000")],
            )
        };
        p.layers = vec![
            r("a", 10.0, 10.0, 20.0),
            r("b", 100.0, 50.0, 40.0),
            r("c", 300.0, 90.0, 20.0),
        ];
        p
    }
    fn names() -> Vec<String> {
        vec!["a".into(), "b".into(), "c".into()]
    }
    fn bx(p: &Project, n: &str) -> (Vec2, Vec2) {
        get(p, n).unwrap()
    }

    #[test]
    fn align_left_right_center_and_stage() {
        let mut p = proj();
        align(&mut p, &names(), AlignMode::Left, false).unwrap();
        assert!(names().iter().all(|n| (bx(&p, n).0.x - 10.0).abs() < 0.6));
        let mut p = proj();
        align(&mut p, &names(), AlignMode::Right, false).unwrap();
        assert!(names().iter().all(|n| (bx(&p, n).1.x - 320.0).abs() < 0.6));
        let mut p = proj();
        align(&mut p, &names(), AlignMode::CenterY, true).unwrap();
        assert!(names().iter().all(|n| {
            let (a, b) = bx(&p, n);
            ((a.y + b.y) * 0.5 - 100.0).abs() < 0.6
        }));
        assert!(align(&mut proj(), &["a".into()], AlignMode::Left, false).is_err());
        assert!(align(
            &mut proj(),
            &["a".into(), "zz".into()],
            AlignMode::Left,
            false
        )
        .is_err());
    }

    #[test]
    fn distribute_gives_equal_gaps_and_centres() {
        let mut p = proj();
        distribute(&mut p, &names(), Axis::X, Distribute::Gaps).unwrap();
        let (a, b, c) = (bx(&p, "a"), bx(&p, "b"), bx(&p, "c"));
        assert!(((b.0.x - a.1.x) - (c.0.x - b.1.x)).abs() < 1.0);
        assert!(
            (a.0.x - 10.0).abs() < 0.6 && (c.1.x - 320.0).abs() < 0.6,
            "ends stay put"
        );
        let mut p = proj();
        distribute(&mut p, &names(), Axis::X, Distribute::Centers).unwrap();
        let mid = |n: &str| (bx(&p, n).0.x + bx(&p, n).1.x) * 0.5;
        assert!(((mid("b") - mid("a")) - (mid("c") - mid("b"))).abs() < 1.0);
        assert!(distribute(&mut proj(), &names()[..2], Axis::X, Distribute::Gaps).is_err());
    }

    #[test]
    fn trim_keeps_requested_fraction() {
        let line = VPath::new(
            "l",
            false,
            vec![
                Anchor::corner(Vec2::new(0.0, 0.0)),
                Anchor::corner(Vec2::new(100.0, 0.0)),
            ],
        );
        let t = trim_path(&line, 0.25, 0.75, 0.0, 0.5).unwrap();
        let pl = t.polyline(0.5);
        assert!((pl.length() - 50.0).abs() < 0.5);
        assert!((t.anchors[0].pos.x - 25.0).abs() < 0.1);
        assert!((t.anchors.last().unwrap().pos.x - 75.0).abs() < 0.1);
        let full = trim_path(&line, 0.0, 1.0, 0.0, 0.5).unwrap();
        assert!((full.polyline(0.5).length() - 100.0).abs() < 0.5);
        assert!(trim_path(&line, -0.1, 1.0, 0.0, 0.5).is_err());
        let sq = VPath::rect("s", 0.0, 0.0, 10.0, 10.0);
        let w = trim_path(&sq, 0.0, 0.5, 0.25, 0.5).unwrap();
        assert!(
            (w.polyline(0.5).length() - 20.0).abs() < 0.5,
            "closed paths wrap with offset"
        );
    }
}
