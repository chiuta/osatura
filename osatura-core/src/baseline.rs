// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Baseline alignment of the members of one layout line.
//!
//! Basis (public): W3C *CSS Flexible Box Layout Module Level 1*, section 8.3 "Cross-axis Alignment" (the baseline alignment rule) and
//! the description of section 9.4 "Cross Size Determination" (size of a single line). Where the text was not read the choice is marked
//! below and in docs/clean-room/specs/06-layout-aliniere-pe-linia-de-baza.md.
//!
//! The functions are pure geometry on one line of a row container (main axis horizontal, cross axis vertical, top = cross start):
//! every member that takes part is shifted so that the distances from its top margin edge to its baseline coincide with the largest of
//! them, which is placed against the start of the line. A member without a baseline gets one at the bottom of its border box
//! (a project decision). In a column container the baseline mode behaves like `Start`. Wrapped containers call [`align_line`] once per
//! line. Not yet connected to `containers::Container` (see docs/clean-room/specs/INTREBARI.md).

/// Cross-axis alignment mode of one member of the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossMode {
    Baseline,
    Start,
    Center,
    End,
    Stretch,
}

/// One member of the line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Member {
    /// Cross size of the border box (>= 0).
    pub height: f64,
    pub margin_start: f64,
    pub margin_end: f64,
    pub mode: CrossMode,
    /// Distance of the baseline from the start edge of the border box; `None` = synthesised at the end of the border box.
    pub baseline: Option<f64>,
}

impl Member {
    /// A baseline member without margins.
    pub fn on_baseline(height: f64, baseline: Option<f64>) -> Member {
        Member { height, margin_start: 0.0, margin_end: 0.0, mode: CrossMode::Baseline, baseline }
    }
    /// A member of another mode, without margins.
    pub fn other(height: f64, mode: CrossMode) -> Member {
        Member { height, margin_start: 0.0, margin_end: 0.0, mode, baseline: None }
    }
    fn outer(&self) -> f64 {
        self.margin_start + self.height + self.margin_end
    }
}

/// Result for one line.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Cross-axis offset of each member's margin-box start from the start of the line, in input order.
    pub offsets: Vec<f64>,
    /// Cross size of the line.
    pub size: f64,
}

/// Aligns one line. `cross_size` is the container's definite cross size, `None` for automatic. `row` is false for a column container
/// (the baseline mode then acts as `Start`). A `Stretch` member is positioned at the start edge (its size is the caller's business).
///
/// Errors: a negative or non-finite height, margin, baseline distance or container size.
pub fn align_line(members: &[Member], cross_size: Option<f64>, row: bool) -> Result<Line, String> {
    for (i, m) in members.iter().enumerate() {
        if !m.height.is_finite() || m.height < 0.0 {
            return Err(format!("member {i}: height must be finite and >= 0"));
        }
        if !m.margin_start.is_finite() || !m.margin_end.is_finite() {
            return Err(format!("member {i}: margins must be finite"));
        }
        if let Some(b) = m.baseline {
            if !b.is_finite() || b < 0.0 {
                return Err(format!("member {i}: the baseline distance must be finite and >= 0, got {b}"));
            }
        }
    }
    if let Some(c) = cross_size {
        if !c.is_finite() || c < 0.0 {
            return Err("the container cross size must be finite and >= 0".into());
        }
    }
    let on_baseline = |m: &Member| row && m.mode == CrossMode::Baseline;
    // distance from the start margin edge to the baseline; `after` is the rest of the margin box
    let before = |m: &Member| m.margin_start + m.baseline.unwrap_or(m.height);
    let max_before = members.iter().filter(|m| on_baseline(m)).map(before).fold(0.0f64, f64::max);
    let max_after = members.iter().filter(|m| on_baseline(m)).map(|m| m.outer() - before(m)).fold(0.0f64, f64::max);
    let others = members.iter().filter(|m| !on_baseline(m)).map(Member::outer).fold(0.0f64, f64::max);
    let size = cross_size.unwrap_or_else(|| (max_before + max_after).max(others));
    let offsets = members
        .iter()
        .map(|m| {
            if on_baseline(m) {
                max_before - before(m)
            } else {
                match m.mode {
                    CrossMode::Center => (size - m.outer()) / 2.0,
                    CrossMode::End => size - m.outer(),
                    CrossMode::Baseline | CrossMode::Start | CrossMode::Stretch => 0.0,
                }
            }
        })
        .collect();
    Ok(Line { offsets, size })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn abc() -> Vec<Member> {
        vec![Member::on_baseline(20.0, Some(15.0)), Member::on_baseline(30.0, Some(10.0)), Member::on_baseline(10.0, Some(10.0))]
    }

    #[test]
    fn three_members_automatic_size() {
        let l = align_line(&abc(), None, true).unwrap();
        assert_eq!(l.offsets, vec![0.0, 5.0, 5.0]);
        assert_eq!(l.size, 35.0);
    }

    #[test]
    fn definite_container_size_keeps_the_group_at_the_start() {
        let l = align_line(&abc(), Some(100.0), true).unwrap();
        assert_eq!(l.offsets, vec![0.0, 5.0, 5.0]);
        assert_eq!(l.size, 100.0);
    }

    #[test]
    fn single_member_and_empty_line() {
        let l = align_line(&abc()[..1], None, true).unwrap();
        assert_eq!((l.offsets, l.size), (vec![0.0], 20.0));
        let e = align_line(&[], None, true).unwrap();
        assert!(e.offsets.is_empty());
        assert_eq!(e.size, 0.0);
    }

    #[test]
    fn synthesised_baseline_is_the_bottom_edge() {
        let l = align_line(&[Member::on_baseline(30.0, Some(10.0)), Member::on_baseline(12.0, None)], None, true).unwrap();
        assert_eq!(l.offsets, vec![2.0, 0.0]);
        assert_eq!(l.size, 32.0);
    }

    #[test]
    fn the_distance_includes_the_start_margin() {
        let mut a = Member::on_baseline(20.0, Some(15.0));
        a.margin_start = 4.0;
        let b = Member::on_baseline(30.0, Some(10.0));
        let l = align_line(&[a, b], None, true).unwrap();
        assert_eq!(l.offsets, vec![0.0, 9.0]);
        // size: max before 19, max after max(20+4-19, 30-10) = 20
        assert_eq!(l.size, 39.0);
    }

    #[test]
    fn other_modes_do_not_take_part() {
        let mut v = abc();
        v.push(Member::other(10.0, CrossMode::Center));
        v.push(Member::other(10.0, CrossMode::End));
        v.push(Member::other(10.0, CrossMode::Start));
        let l = align_line(&v, None, true).unwrap();
        assert_eq!(l.size, 35.0);
        assert_eq!(l.offsets[3], 12.5);
        assert_eq!(l.offsets[4], 25.0);
        assert_eq!(l.offsets[5], 0.0);
        assert_eq!(&l.offsets[..3], &[0.0, 5.0, 5.0]);
        // a tall member of another mode can make the line larger
        let l = align_line(&[Member::on_baseline(10.0, None), Member::other(50.0, CrossMode::Start)], None, true).unwrap();
        assert_eq!(l.size, 50.0);
    }

    #[test]
    fn column_container_behaves_like_start() {
        let l = align_line(&abc(), None, false).unwrap();
        assert_eq!(l.offsets, vec![0.0, 0.0, 0.0]);
        assert_eq!(l.size, 30.0);
    }

    #[test]
    fn errors() {
        assert!(align_line(&[Member::on_baseline(10.0, Some(-1.0))], None, true).is_err());
        assert!(align_line(&[Member::on_baseline(10.0, Some(f64::NAN))], None, true).is_err());
        assert!(align_line(&[Member::on_baseline(f64::INFINITY, None)], None, true).is_err());
        assert!(align_line(&[Member::on_baseline(-1.0, None)], None, true).is_err());
        assert!(align_line(&abc(), Some(f64::NAN), true).is_err());
        let mut m = Member::on_baseline(1.0, None);
        m.margin_end = f64::NAN;
        assert!(align_line(&[m], None, true).is_err());
    }

    #[test]
    fn equal_distances_give_zero_offsets_and_far_baselines_are_accepted() {
        let l = align_line(&[Member::on_baseline(10.0, Some(5.0)), Member::on_baseline(20.0, Some(5.0))], None, true).unwrap();
        assert_eq!(l.offsets, vec![0.0, 0.0]);
        // baseline below the box: allowed
        let l = align_line(&[Member::on_baseline(10.0, Some(25.0)), Member::on_baseline(10.0, None)], None, true).unwrap();
        assert_eq!(l.offsets, vec![0.0, 15.0]);
    }
}
