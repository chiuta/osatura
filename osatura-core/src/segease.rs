// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Two easing-curve families defined by a public specification: a piecewise-linear curve through author-given stops, and a
//! stepped curve with four jump placements; plus the validity rules and named presets of the cubic Bézier curve.
//!
//! Basis (public): W3C *CSS Easing Functions Level 2*, First Public Working Draft of 29 August 2024 (<https://www.w3.org/TR/css-easing-2>),
//! sections on the piecewise-linear function, the cubic Bézier function and the steps function (including its output algorithm).
//! The text is a working draft; details may differ in later versions. Spec: docs/clean-room/specs/05-easing-liniar-pe-segmente-si-trepte.md.
//!
//! The curves are plain functions of progress `p` (any real number, normally in `[0, 1]`) returning a real that may leave `[0, 1]`.
//! Not part of `anim::Ease` (that type is `Copy`, a stop list is not). Connected instead by baking: [`BakeCurve`] turns a curve into
//! the keys of a segment (`keyops::bake_curve`, CLI `keys curve`), see docs/clean-room/specs/INTREBARI.md.

/// Largest number of stops of a piecewise curve and of steps of a stepped curve (safety cap of the project).
pub const MAX_STOPS: usize = 4096;

/// One stop: an output value and zero, one or two inputs (progress positions, as fractions: 75 % = 0.75).
#[derive(Clone, Debug, PartialEq)]
pub struct Stop {
    pub output: f64,
    pub inputs: Vec<f64>,
}

impl Stop {
    /// A stop whose position is left to the rules.
    pub fn at_auto(output: f64) -> Stop {
        Stop {
            output,
            inputs: vec![],
        }
    }
    /// A stop at one position.
    pub fn at(output: f64, input: f64) -> Stop {
        Stop {
            output,
            inputs: vec![input],
        }
    }
    /// A stop at two positions (a vertical jump of the curve is not produced: both points have the same output).
    pub fn between(output: f64, from: f64, to: f64) -> Stop {
        Stop {
            output,
            inputs: vec![from, to],
        }
    }
}

/// Parses `"75%"` (-> 0.75) or a plain fraction `"0.75"`.
pub fn parse_input(text: &str) -> Result<f64, String> {
    let t = text.trim();
    let v = match t.strip_suffix('%') {
        Some(p) => p.trim().parse::<f64>().map(|x| x / 100.0),
        None => t.parse::<f64>(),
    }
    .map_err(|_| format!("'{text}' is not a number or a percentage"))?;
    if v.is_finite() {
        Ok(v)
    } else {
        Err(format!("'{text}' is not finite"))
    }
}

/// A piecewise-linear easing curve.
#[derive(Clone, Debug, PartialEq)]
pub struct PiecewiseCurve {
    /// `(input, output)`, inputs non-decreasing.
    points: Vec<(f64, f64)>,
}

impl PiecewiseCurve {
    /// The identity curve (the keyword without stops): same as stops 0 and 1.
    pub fn linear() -> PiecewiseCurve {
        PiecewiseCurve {
            points: vec![(0.0, 0.0), (1.0, 1.0)],
        }
    }

    /// Normalises the stops:
    /// 1. a stop with two inputs becomes two points with the same output;
    /// 2. a missing first input is 0, a missing last input is 1 (at least the largest input before it);
    /// 3. an explicit input below the largest input before it is raised to it;
    /// 4. other missing inputs are spread evenly (linearly by position) between the nearest neighbours that have one.
    ///
    /// Errors: fewer than two stops, more than [`MAX_STOPS`] points, an input or output that is not finite, more than two inputs.
    pub fn new(stops: &[Stop]) -> Result<PiecewiseCurve, String> {
        if stops.len() < 2 {
            return Err(format!(
                "a piecewise curve needs at least 2 stops, got {}",
                stops.len()
            ));
        }
        let mut pts: Vec<(Option<f64>, f64)> = Vec::new();
        for s in stops {
            if !s.output.is_finite() || s.inputs.iter().any(|i| !i.is_finite()) {
                return Err("stop values must be finite numbers".into());
            }
            match s.inputs.len() {
                0 => pts.push((None, s.output)),
                1 | 2 => pts.extend(s.inputs.iter().map(|&i| (Some(i), s.output))),
                n => return Err(format!("a stop has at most 2 inputs, got {n}")),
            }
            if pts.len() > MAX_STOPS {
                return Err(format!("too many points (limit {MAX_STOPS})"));
            }
        }
        let last = pts.len() - 1;
        if pts[0].0.is_none() {
            pts[0].0 = Some(0.0);
        }
        if pts[last].0.is_none() {
            let before = pts[..last]
                .iter()
                .filter_map(|p| p.0)
                .fold(f64::NEG_INFINITY, f64::max);
            pts[last].0 = Some(before.max(1.0));
        }
        let mut high = f64::NEG_INFINITY;
        for p in pts.iter_mut() {
            if let Some(i) = p.0 {
                high = high.max(i);
                p.0 = Some(high);
            }
        }
        // spread the gaps evenly
        let mut i = 0;
        while i < pts.len() {
            if pts[i].0.is_some() {
                i += 1;
                continue;
            }
            let lo = i - 1; // exists: the first point has an input
            let hi = (i..pts.len()).find(|&k| pts[k].0.is_some()).unwrap_or(last); // the last point has an input
            let (a, b) = (pts[lo].0.unwrap_or(0.0), pts[hi].0.unwrap_or(1.0));
            let n = (hi - lo) as f64;
            for (k, p) in pts.iter_mut().enumerate().take(hi).skip(i) {
                p.0 = Some(a + (b - a) * ((k - lo) as f64) / n);
            }
            i = hi;
        }
        Ok(PiecewiseCurve {
            points: pts
                .into_iter()
                .map(|(i, o)| (i.unwrap_or(0.0), o))
                .collect(),
        })
    }

    /// The normalised `(input, output)` points.
    pub fn points(&self) -> &[(f64, f64)] {
        &self.points
    }

    /// Output at progress `p`. Finds the last point whose input is `<= p` (the first point when none); when that is the last
    /// point it takes the one before; interpolates linearly to the next point (equal inputs give the next point's output).
    /// Outside the defined range the line through the two nearest points continues.
    pub fn eval(&self, p: f64) -> f64 {
        let n = self.points.len();
        let mut a = self.points.iter().rposition(|pt| pt.0 <= p).unwrap_or(0);
        if a == n - 1 {
            a -= 1;
        }
        let (x0, y0) = self.points[a];
        let (x1, y1) = self.points[a + 1];
        if x0 == x1 {
            y1
        } else {
            y0 + (y1 - y0) * (p - x0) / (x1 - x0)
        }
    }
}

/// Where the jumps of a stepped curve sit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPosition {
    /// The first jump happens at progress 0.
    JumpStart,
    /// The last jump happens at progress 1 (the default).
    JumpEnd,
    /// No jump at either end.
    JumpNone,
    /// A jump at both ends.
    JumpBoth,
}

impl StepPosition {
    /// `jump-start|start`, `jump-end|end`, `jump-none`, `jump-both` (case-insensitive).
    pub fn parse(s: &str) -> Result<StepPosition, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "jump-start" | "start" => Ok(StepPosition::JumpStart),
            "jump-end" | "end" => Ok(StepPosition::JumpEnd),
            "jump-none" => Ok(StepPosition::JumpNone),
            "jump-both" => Ok(StepPosition::JumpBoth),
            other => Err(format!(
                "unknown step position '{other}' (jump-start, jump-end, jump-none, jump-both)"
            )),
        }
    }
}

/// A stepped curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Steps {
    n: u32,
    pos: StepPosition,
}

impl Steps {
    /// `n` must be at least 1 (at least 2 for `JumpNone`) and at most [`MAX_STOPS`].
    pub fn new(n: i64, pos: StepPosition) -> Result<Steps, String> {
        if n <= 0 {
            return Err(format!("the number of steps must be positive, got {n}"));
        }
        if n as u64 > MAX_STOPS as u64 {
            return Err(format!("too many steps (limit {MAX_STOPS})"));
        }
        if n == 1 && pos == StepPosition::JumpNone {
            return Err("jump-none needs more than 1 step".into());
        }
        Ok(Steps { n: n as u32, pos })
    }
    /// One step, jump at the start.
    pub fn step_start() -> Steps {
        Steps {
            n: 1,
            pos: StepPosition::JumpStart,
        }
    }
    /// One step, jump at the end.
    pub fn step_end() -> Steps {
        Steps {
            n: 1,
            pos: StepPosition::JumpEnd,
        }
    }

    /// Output at progress `p`, evaluated at the start of an interval (the "before" flag of the algorithm is false). `p` must be finite.
    pub fn eval(&self, p: f64) -> f64 {
        self.eval_flagged(p, false)
    }

    /// Output with the "before" flag: when set and `p * n` is an integer the step just ended is reported.
    pub fn eval_flagged(&self, p: f64, before: bool) -> f64 {
        let n = self.n as f64;
        let scaled = p * n;
        let mut step = scaled.floor();
        if matches!(self.pos, StepPosition::JumpStart | StepPosition::JumpBoth) {
            step += 1.0;
        }
        if before && scaled.fract() == 0.0 {
            step -= 1.0;
        }
        if p >= 0.0 && step < 0.0 {
            step = 0.0;
        }
        let jumps = match self.pos {
            StepPosition::JumpStart | StepPosition::JumpEnd => n,
            StepPosition::JumpNone => n - 1.0,
            StepPosition::JumpBoth => n + 1.0,
        };
        if p <= 1.0 && step > jumps {
            step = jumps;
        }
        step / jumps
    }
}

/// A curve that can be written as keys of an ordinary track (see [`BakeCurve::keys`]).
#[derive(Clone, Debug, PartialEq)]
pub enum BakeCurve {
    Piecewise(PiecewiseCurve),
    Steps(Steps),
}

/// One key of a baked segment: progress `u` in `[0, 1)`, curve output `level` there, and whether the value is held until the next key
/// (a stepped plateau) instead of interpolated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BakeKey {
    pub u: f64,
    pub level: f64,
    pub hold: bool,
}

/// Largest absolute deviation from 0 at progress 0 and from 1 at progress 1 that still counts as "starts at 0, ends at 1".
const END_TOL: f64 = 1e-9;

impl BakeCurve {
    /// Parses a command-line description:
    /// * `piecewise:STOP,STOP,...` where a STOP is `output [input [input]]` with inputs as fractions or percentages
    ///   (`piecewise:0,0.25 75%,1`);
    /// * `steps:N[:position]` with a position as in [`StepPosition::parse`] (default `jump-end`).
    pub fn parse(spec: &str) -> Result<BakeCurve, String> {
        let (kind, rest) = spec.split_once(':').ok_or_else(|| {
            format!("curve '{spec}': write piecewise:STOP,STOP,... or steps:N[:position]")
        })?;
        match kind.trim().to_ascii_lowercase().as_str() {
            "piecewise" => {
                let mut stops = Vec::new();
                for part in rest.split(',') {
                    let mut it = part.split_whitespace();
                    let out = it.next().ok_or("an empty stop in the piecewise curve")?;
                    let output = parse_input(out)?;
                    let inputs: Vec<f64> = it.map(parse_input).collect::<Result<_, _>>()?;
                    stops.push(Stop { output, inputs });
                }
                Ok(BakeCurve::Piecewise(PiecewiseCurve::new(&stops)?))
            }
            "steps" => {
                let mut it = rest.split(':');
                let n: i64 = it.next().unwrap_or("").trim().parse().map_err(|_| {
                    format!("curve '{spec}': the number of steps must be an integer")
                })?;
                let pos = match it.next() {
                    Some(p) => StepPosition::parse(p)?,
                    None => StepPosition::JumpEnd,
                };
                if it.next().is_some() {
                    return Err(format!("curve '{spec}': write steps:N[:position]"));
                }
                Ok(BakeCurve::Steps(Steps::new(n, pos)?))
            }
            other => Err(format!("unknown curve kind '{other}' (piecewise, steps)")),
        }
    }

    /// The keys that reproduce the curve exactly between two keys of a track: the first has `u == 0`; the segment's end key is not
    /// listed. Piecewise curves give a key at every point inside `[0, 1)` (linear between them); stepped curves one held key per plateau.
    ///
    /// Errors, because a track cannot represent them: output at progress 0 other than 0 or at 1 other than 1 (that would change the
    /// neighbouring keys' values; this is the case of `jump-start` and `jump-both`), and a vertical jump inside a piecewise curve (two
    /// stops with the same input and different outputs).
    pub fn keys(&self) -> Result<Vec<BakeKey>, String> {
        let (at0, at1) = match self {
            BakeCurve::Piecewise(c) => (c.eval(0.0), c.eval(1.0)),
            BakeCurve::Steps(s) => (s.eval(0.0), s.eval(1.0)),
        };
        if at0.abs() > END_TOL {
            return Err(format!("the curve starts at {at0}, not 0: a track cannot show that without changing the key before (for steps use jump-end or jump-none)"));
        }
        if (at1 - 1.0).abs() > END_TOL {
            return Err(format!("the curve ends at {at1}, not 1: a track cannot show that without changing the next key"));
        }
        match self {
            BakeCurve::Piecewise(c) => {
                let pts = c.points();
                if pts.windows(2).any(|w| w[0].0 == w[1].0 && w[0].1 != w[1].1) {
                    return Err("the curve has a vertical jump (two stops with the same input and different outputs); a track cannot jump inside one frame range".into());
                }
                let mut us: Vec<f64> = vec![0.0];
                us.extend(pts.iter().map(|p| p.0).filter(|&x| x > 0.0 && x < 1.0));
                us.dedup();
                Ok(us
                    .into_iter()
                    .map(|u| BakeKey {
                        u,
                        level: c.eval(u),
                        hold: false,
                    })
                    .collect())
            }
            BakeCurve::Steps(s) => {
                let n = s.n as usize;
                Ok((0..n)
                    .map(|j| {
                        let u = j as f64 / n as f64;
                        BakeKey {
                            u,
                            level: s.eval(u + 0.5 / n as f64),
                            hold: true,
                        }
                    })
                    .collect())
            }
        }
    }
}

/// Checks the control points of a cubic Bézier easing curve (end points are fixed at (0,0) and (1,1)): all four numbers finite and
/// both abscissae in `[0, 1]`.
pub fn check_bezier(x1: f64, y1: f64, x2: f64, y2: f64) -> Result<[f64; 4], String> {
    if ![x1, y1, x2, y2].iter().all(|v| v.is_finite()) {
        return Err("Bézier control points must be finite".into());
    }
    if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
        return Err(format!(
            "Bézier abscissae must lie in [0, 1], got {x1} and {x2}"
        ));
    }
    Ok([x1, y1, x2, y2])
}

/// The control points of the named presets: `ease`, `ease-in`, `ease-out`, `ease-in-out`.
pub fn named_bezier(name: &str) -> Option<[f64; 4]> {
    match name.trim().to_ascii_lowercase().as_str() {
        "ease" => Some([0.25, 0.1, 0.25, 1.0]),
        "ease-in" => Some([0.42, 0.0, 1.0, 1.0]),
        "ease-out" => Some([0.0, 0.0, 0.58, 1.0]),
        "ease-in-out" => Some([0.42, 0.0, 0.58, 1.0]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} vs {b}");
    }

    fn curve_a() -> PiecewiseCurve {
        PiecewiseCurve::new(&[Stop::at_auto(0.0), Stop::at(0.25, 0.75), Stop::at_auto(1.0)])
            .unwrap()
    }

    #[test]
    fn curve_a_inside_and_outside() {
        let a = curve_a();
        close(a.eval(0.5), 0.166667);
        close(a.eval(0.75), 0.25);
        close(a.eval(1.0), 1.0);
        close(a.eval(1.2), 1.6);
        close(a.eval(-0.1), -0.033333);
    }

    #[test]
    fn curve_b_raises_a_smaller_input_and_jumps() {
        let b = PiecewiseCurve::new(&[
            Stop::at_auto(0.0),
            Stop::at(0.8, 0.6),
            Stop::at(0.4, 0.3),
            Stop::at_auto(1.0),
        ])
        .unwrap();
        assert_eq!(b.points()[2].0, 0.6);
        close(b.eval(0.59), 0.786667);
        close(b.eval(0.6), 0.4);
    }

    #[test]
    fn missing_inputs_are_spread_and_two_inputs_make_two_points() {
        let c = PiecewiseCurve::new(&[
            Stop::at_auto(0.0),
            Stop::at_auto(1.0),
            Stop::at_auto(2.0),
            Stop::at(3.0, 0.9),
        ])
        .unwrap();
        let xs: Vec<f64> = c.points().iter().map(|p| p.0).collect();
        close(xs[1], 0.3);
        close(xs[2], 0.6);
        let d = PiecewiseCurve::new(&[
            Stop::at_auto(0.0),
            Stop::between(5.0, 0.2, 0.4),
            Stop::at_auto(1.0),
        ])
        .unwrap();
        assert_eq!(d.points().len(), 4);
        assert_eq!((d.points()[1], d.points()[2]), ((0.2, 5.0), (0.4, 5.0)));
        assert_eq!(PiecewiseCurve::linear().eval(0.25), 0.25);
    }

    #[test]
    fn piecewise_errors() {
        assert!(PiecewiseCurve::new(&[Stop::at_auto(0.0)]).is_err());
        assert!(PiecewiseCurve::new(&[]).is_err());
        assert!(PiecewiseCurve::new(&[Stop::at_auto(0.0), Stop::at_auto(f64::NAN)]).is_err());
        assert!(PiecewiseCurve::new(&[Stop::at_auto(0.0), Stop::at(1.0, f64::INFINITY)]).is_err());
        let many: Vec<Stop> = (0..MAX_STOPS + 1)
            .map(|i| Stop::at_auto(i as f64))
            .collect();
        assert!(PiecewiseCurve::new(&many).is_err());
        assert_eq!(parse_input("75%").unwrap(), 0.75);
        assert_eq!(parse_input(" 0.5 ").unwrap(), 0.5);
        assert!(parse_input("abc").is_err() && parse_input("NaN").is_err());
    }

    #[test]
    fn steps_jump_end_and_start() {
        let e = Steps::new(4, StepPosition::JumpEnd).unwrap();
        close(e.eval(0.3), 0.25);
        close(e.eval(1.0), 1.0);
        close(e.eval(0.0), 0.0);
        let s = Steps::new(4, StepPosition::JumpStart).unwrap();
        close(s.eval(0.3), 0.5);
        close(s.eval(0.0), 0.25);
        assert_eq!(
            Steps::step_start(),
            Steps::new(1, StepPosition::JumpStart).unwrap()
        );
        assert_eq!(Steps::step_end().eval(0.5), 0.0);
    }

    #[test]
    fn steps_jump_none_and_both() {
        let n = Steps::new(4, StepPosition::JumpNone).unwrap();
        close(n.eval(0.5), 0.666667);
        close(n.eval(0.0), 0.0);
        close(n.eval(1.0), 1.0);
        let b = Steps::new(4, StepPosition::JumpBoth).unwrap();
        close(b.eval(0.0), 0.2);
        close(b.eval(1.0), 1.0);
    }

    #[test]
    fn steps_errors_and_positions() {
        assert!(Steps::new(1, StepPosition::JumpNone).is_err());
        for pos in [
            StepPosition::JumpStart,
            StepPosition::JumpEnd,
            StepPosition::JumpBoth,
            StepPosition::JumpNone,
        ] {
            assert!(Steps::new(0, pos).is_err());
            assert!(Steps::new(-3, pos).is_err());
        }
        assert!(Steps::new(MAX_STOPS as i64 + 1, StepPosition::JumpEnd).is_err());
        assert_eq!(
            StepPosition::parse("Start").unwrap(),
            StepPosition::JumpStart
        );
        assert!(StepPosition::parse("middle").is_err());
        // with the "before" flag a boundary reports the step just ended
        let e = Steps::new(4, StepPosition::JumpEnd).unwrap();
        close(e.eval_flagged(0.5, true), 0.25);
        close(e.eval_flagged(0.5, false), 0.5);
        // beyond [0, 1] the steps are not clamped on the open side
        close(e.eval(-0.1), -0.25);
        close(e.eval(1.5), 1.5);
    }

    #[test]
    fn bezier_validation_and_names() {
        assert!(check_bezier(1.5, 0.0, 0.5, 1.0).is_err());
        assert!(check_bezier(0.5, f64::NAN, 0.5, 1.0).is_err());
        assert!(
            check_bezier(0.5, -3.0, 0.5, 4.0).is_ok(),
            "ordinates may leave [0,1]"
        );
        assert_eq!(
            named_bezier("ease").unwrap(),
            check_bezier(0.25, 0.1, 0.25, 1.0).unwrap()
        );
        assert_eq!(named_bezier("ease-in-out").unwrap(), [0.42, 0.0, 0.58, 1.0]);
        assert!(named_bezier("bounce").is_none());
    }

    #[test]
    fn bake_curve_parses_both_families_and_refuses_what_a_track_cannot_show() {
        let c = BakeCurve::parse("piecewise:0,0.25 75%,1").unwrap();
        let k = c.keys().unwrap();
        assert_eq!(k.iter().map(|b| b.u).collect::<Vec<_>>(), vec![0.0, 0.75]);
        assert!((k[1].level - 0.25).abs() < 1e-12 && !k[1].hold);
        let s = BakeCurve::parse("steps:4").unwrap().keys().unwrap();
        assert_eq!(
            s.iter().map(|b| b.level).collect::<Vec<_>>(),
            vec![0.0, 0.25, 0.5, 0.75]
        );
        assert!(s.iter().all(|b| b.hold));
        let n = BakeCurve::parse("steps:3:jump-none")
            .unwrap()
            .keys()
            .unwrap();
        assert_eq!(
            n.iter().map(|b| b.level).collect::<Vec<_>>(),
            vec![0.0, 0.5, 1.0]
        );
        for bad in [
            "steps:4:jump-start",
            "steps:2:jump-both",
            "piecewise:0.2,1",
            "piecewise:0,0.5",
            "piecewise:0,0 50%,1 50%,1",
            "nonsense",
            "steps:x",
            "steps:0",
            "ramp:1,2",
        ] {
            assert!(
                BakeCurve::parse(bad).and_then(|c| c.keys()).is_err(),
                "{bad}"
            );
        }
    }
}
