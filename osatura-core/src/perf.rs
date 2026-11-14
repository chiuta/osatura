// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Performance capture helpers: turn recorded samples into minimal keys, map arbitrary
//! input signals (sliders, MIDI, audio level, face-tracker output, gamepad…) onto
//! parameters, and retarget motion by *where the hand/foot goes relative to limb reach*.
//!
//! Clean-room design note: nothing here depends on a tracker or a vendor SDK. Any
//! program can write a CSV/JSON of numbers; OSATURA only maps and reduces them.

use crate::anim::{Ease, Track};
use crate::math::Vec2;
use crate::project::{Channel, Project, Prop, Source};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---- sample reduction ---------------------------------------------------------------------

/// Reduce `(frame, value)` samples to the fewest linear keys whose interpolation stays within
/// `tol` of every sample (Ramer–Douglas–Peucker on the value axis, iterative).
pub fn simplify_samples(samples: &[(f32, f32)], tol: f32) -> Track {
    let mut s: Vec<(f32, f32)> = samples
        .iter()
        .copied()
        .filter(|(f, v)| f.is_finite() && v.is_finite())
        .collect();
    s.sort_by(|a, b| a.0.total_cmp(&b.0));
    s.dedup_by(|a, b| a.0 == b.0);
    if s.len() <= 2 {
        return Track::with_keys(
            &s.iter()
                .map(|&(f, v)| (f, v, Ease::Linear))
                .collect::<Vec<_>>(),
        );
    }
    let mut keep = vec![false; s.len()];
    keep[0] = true;
    keep[s.len() - 1] = true;
    let mut stack = vec![(0usize, s.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        if b <= a + 1 {
            continue;
        }
        let ((fa, va), (fb, vb)) = (s[a], s[b]);
        let (mut worst, mut wi) = (0.0f32, a);
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for i in a + 1..b {
            let t = if fb > fa {
                (s[i].0 - fa) / (fb - fa)
            } else {
                0.0
            };
            let e = (s[i].1 - (va + (vb - va) * t)).abs();
            if e > worst {
                worst = e;
                wi = i;
            }
        }
        if worst > tol {
            keep[wi] = true;
            stack.push((a, wi));
            stack.push((wi, b));
        }
    }
    Track::with_keys(
        &s.iter()
            .zip(&keep)
            .filter(|(_, k)| **k)
            .map(|(&(f, v), _)| (f, v, Ease::Linear))
            .collect::<Vec<_>>(),
    )
}

/// A key-reduced channel from per-frame samples (frame = index).
pub fn record(target: &str, prop: Prop, per_frame: &[f32], tol: f32) -> Channel {
    let samples: Vec<(f32, f32)> = per_frame
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f32, v))
        .collect();
    Channel::keys(target, prop, simplify_samples(&samples, tol))
}

/// Largest absolute error of a track against its source samples (for reporting).
pub fn max_error(t: &Track, samples: &[(f32, f32)]) -> f32 {
    samples
        .iter()
        .map(|&(f, v)| (t.eval(f).unwrap_or(v) - v).abs())
        .fold(0.0, f32::max)
}

// ---- input mapping ------------------------------------------------------------------------

/// Response curve applied to the normalised 0..1 input.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Curve {
    Linear,
    /// `u^exp` (exp > 1 = gentle start).
    Pow {
        exp: f32,
    },
    /// Smoothstep.
    Smooth,
}

impl Curve {
    fn apply(self, u: f32) -> f32 {
        match self {
            Curve::Linear => u,
            Curve::Pow { exp } => u.powf(exp.max(0.01)),
            Curve::Smooth => u * u * (3.0 - 2.0 * u),
        }
    }
}

/// Maps one named input signal onto one parameter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputMap {
    pub input: String,
    pub target: String,
    pub prop: Prop,
    #[serde(default)]
    pub in_min: f32,
    #[serde(default = "one")]
    pub in_max: f32,
    #[serde(default)]
    pub out_min: f32,
    #[serde(default = "one")]
    pub out_max: f32,
    /// Fraction of the input range (from `in_min`) treated as zero.
    #[serde(default)]
    pub deadzone: f32,
    #[serde(default)]
    pub invert: bool,
    #[serde(default = "lin")]
    pub curve: Curve,
    /// 0 = raw, towards 1 = heavier exponential smoothing.
    #[serde(default)]
    pub smooth: f32,
}

fn one() -> f32 {
    1.0
}
fn lin() -> Curve {
    Curve::Linear
}

impl InputMap {
    pub fn new(input: &str, target: &str, prop: Prop) -> Self {
        InputMap {
            input: input.into(),
            target: target.into(),
            prop,
            in_min: 0.0,
            in_max: 1.0,
            out_min: 0.0,
            out_max: 1.0,
            deadzone: 0.0,
            invert: false,
            curve: Curve::Linear,
            smooth: 0.0,
        }
    }

    /// Map a whole signal (one value per frame).
    pub fn apply(&self, signal: &[f32]) -> Vec<f32> {
        let span = if (self.in_max - self.in_min).abs() < 1e-9 {
            1.0
        } else {
            self.in_max - self.in_min
        };
        let dz = self.deadzone.clamp(0.0, 0.99);
        let a = self.smooth.clamp(0.0, 0.999);
        let mut prev: Option<f32> = None;
        signal
            .iter()
            .map(|&x| {
                let mut u = ((x - self.in_min) / span).clamp(0.0, 1.0);
                u = if u < dz { 0.0 } else { (u - dz) / (1.0 - dz) };
                if self.invert {
                    u = 1.0 - u;
                }
                u = self.curve.apply(u);
                let y = self.out_min + u * (self.out_max - self.out_min);
                let y = match prev {
                    Some(p) => p * a + y * (1.0 - a),
                    None => y,
                };
                prev = Some(y);
                y
            })
            .collect()
    }
}

/// Parse a CSV with a header row; every column is a signal, row index = frame.
pub fn parse_signals_csv(text: &str) -> Result<HashMap<String, Vec<f32>>, String> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header: Vec<String> = lines
        .next()
        .ok_or("empty CSV")?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    if header.iter().any(|h| h.is_empty()) {
        return Err("CSV header has an empty column name".into());
    }
    let mut cols: Vec<Vec<f32>> = vec![Vec::new(); header.len()];
    for (n, line) in lines.enumerate() {
        let cells: Vec<&str> = line.split(',').collect();
        if cells.len() != header.len() {
            return Err(format!(
                "row {} has {} cells, expected {}",
                n + 2,
                cells.len(),
                header.len()
            ));
        }
        for (c, cell) in cells.iter().enumerate() {
            cols[c].push(
                cell.trim()
                    .parse::<f32>()
                    .map_err(|_| format!("row {}: '{}' is not a number", n + 2, cell.trim()))?,
            );
        }
    }
    Ok(header.into_iter().zip(cols).collect())
}

/// Turn signals into key-reduced channels using the given mappings.
pub fn drive(
    maps: &[InputMap],
    signals: &HashMap<String, Vec<f32>>,
    tol: f32,
) -> Result<Vec<Channel>, String> {
    maps.iter()
        .map(|m| {
            let s = signals
                .get(&m.input)
                .ok_or_else(|| format!("no input signal named '{}'", m.input))?;
            Ok(record(&m.target, m.prop, &m.apply(s), tol))
        })
        .collect()
}

/// Scale-free feature from tracker landmarks: `|a−b| / |ref_a−ref_b|` (e.g. mouth opening
/// relative to face height). Returns `None` if any point is missing or the reference is 0.
pub fn landmark_ratio(
    pts: &HashMap<String, Vec2>,
    a: &str,
    b: &str,
    ref_a: &str,
    ref_b: &str,
) -> Option<f32> {
    let d = |x: &str, y: &str| Some(pts.get(x)?.distance(*pts.get(y)?));
    let r = d(ref_a, ref_b)?;
    if r < 1e-6 {
        None
    } else {
        Some(d(a, b)? / r)
    }
}

// ---- retarget by effector reach -------------------------------------------------------------

/// Total bone length of an IK chain.
pub fn chain_reach(p: &Project, chain: &str) -> Option<f32> {
    let c = p.ik.iter().find(|c| c.name == chain)?;
    c.bones
        .iter()
        .map(|b| p.bones.iter().find(|x| &x.name == b).map(|x| x.length))
        .sum()
}

/// Retarget by *intent*: instead of copying joint angles, scale where each effector (IK target
/// bone) moves by the ratio of limb reaches, and point the channels at the destination rig's
/// target bone. `pairs` = (source chain, destination chain). Non-key channels on an effector
/// cannot be scaled and are reported in the returned notes (and passed through renamed).
pub fn retarget_effectors(
    channels: &[Channel],
    src: &Project,
    dst: &Project,
    pairs: &[(String, String)],
) -> Result<(Vec<Channel>, Vec<String>), String> {
    let mut map: HashMap<String, (String, f32)> = HashMap::new();
    for (sc, dc) in pairs {
        let s = src
            .ik
            .iter()
            .find(|c| &c.name == sc)
            .ok_or_else(|| format!("source rig has no IK chain '{sc}'"))?;
        let d = dst
            .ik
            .iter()
            .find(|c| &c.name == dc)
            .ok_or_else(|| format!("destination rig has no IK chain '{dc}'"))?;
        let (rs, rd) = (
            chain_reach(src, sc).ok_or("bad source chain")?,
            chain_reach(dst, dc).ok_or("bad destination chain")?,
        );
        if rs < 1e-6 {
            return Err(format!("source chain '{sc}' has zero reach"));
        }
        map.insert(s.target.clone(), (d.target.clone(), rd / rs));
    }
    let mut notes = Vec::new();
    let out = channels
        .iter()
        .map(|c| {
            let mut c = c.clone();
            if let Some((new_target, k)) = map.get(&c.target) {
                if matches!(c.prop, Prop::BoneOffsetX | Prop::BoneOffsetY) {
                    match &mut c.source {
                        Source::Keys { track } => {
                            for key in &mut track.keys {
                                key.value *= k;
                            }
                        }
                        _ => notes.push(format!(
                            "channel on '{}' ({:?}) is not keyed; renamed but not scaled",
                            c.target, c.prop
                        )),
                    }
                }
                c.target = new_target.clone();
            }
            c
        })
        .collect();
    Ok((out, notes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Bone;

    #[test]
    fn simplify_respects_tolerance_and_shrinks() {
        let samples: Vec<(f32, f32)> = (0..120)
            .map(|i| (i as f32, (i as f32 * 0.1).sin() * 20.0))
            .collect();
        let t = simplify_samples(&samples, 0.5);
        assert!(t.keys.len() < 40, "kept {}", t.keys.len());
        assert!(max_error(&t, &samples) <= 0.5 + 1e-4);
        assert_eq!(t.keys.first().unwrap().frame, 0.0);
        assert_eq!(t.keys.last().unwrap().frame, 119.0);
        // a straight line collapses to two keys
        let line: Vec<(f32, f32)> = (0..50).map(|i| (i as f32, 2.0 * i as f32)).collect();
        assert_eq!(simplify_samples(&line, 0.01).keys.len(), 2);
        assert!(simplify_samples(&[], 1.0).keys.is_empty());
    }

    #[test]
    fn tighter_tolerance_never_gives_fewer_keys() {
        let samples: Vec<(f32, f32)> = (0..80).map(|i| (i as f32, ((i * 7) % 13) as f32)).collect();
        let a = simplify_samples(&samples, 3.0).keys.len();
        let b = simplify_samples(&samples, 0.2).keys.len();
        assert!(b >= a);
        assert!(max_error(&simplify_samples(&samples, 0.2), &samples) <= 0.2 + 1e-4);
    }

    #[test]
    fn input_map_range_deadzone_invert_curve_smooth() {
        let mut m = InputMap::new("s", "b", Prop::BoneAngle);
        m.out_min = -30.0;
        m.out_max = 30.0;
        assert_eq!(
            m.apply(&[0.0, 0.5, 1.0, 2.0, -1.0]),
            vec![-30.0, 0.0, 30.0, 30.0, -30.0]
        );
        m.deadzone = 0.5;
        assert_eq!(
            m.apply(&[0.4])[0],
            -30.0,
            "inside the deadzone = bottom of range"
        );
        m.deadzone = 0.0;
        m.invert = true;
        assert_eq!(m.apply(&[0.0])[0], 30.0);
        m.invert = false;
        m.curve = Curve::Pow { exp: 2.0 };
        assert!((m.apply(&[0.5])[0] - (-30.0 + 0.25 * 60.0)).abs() < 1e-4);
        m.curve = Curve::Linear;
        m.smooth = 0.5;
        let y = m.apply(&[0.0, 1.0, 1.0]);
        assert_eq!(y[0], -30.0);
        assert!(
            y[1] < 30.0 && y[1] > y[0] && y[2] > y[1],
            "smoothing lags: {y:?}"
        );
    }

    #[test]
    fn csv_parses_and_reports_errors() {
        let s = parse_signals_csv("mouth,brow\n0.0,0.5\n1.0,0.25\n").unwrap();
        assert_eq!(s["mouth"], vec![0.0, 1.0]);
        assert_eq!(s["brow"], vec![0.5, 0.25]);
        assert!(parse_signals_csv("a,b\n1\n").unwrap_err().contains("row 2"));
        assert!(parse_signals_csv("a\nx\n")
            .unwrap_err()
            .contains("not a number"));
        assert!(parse_signals_csv("").is_err());
    }

    #[test]
    fn drive_produces_reduced_channels_and_reports_missing_inputs() {
        let sig: HashMap<String, Vec<f32>> = [(
            "mouth".to_string(),
            (0..60).map(|i| i as f32 / 59.0).collect(),
        )]
        .into();
        let mut m = InputMap::new("mouth", "jaw", Prop::BoneAngle);
        m.out_max = 20.0;
        let ch = drive(&[m.clone()], &sig, 0.05).unwrap();
        let Source::Keys { track } = &ch[0].source else {
            panic!()
        };
        assert_eq!(track.keys.len(), 2, "a ramp needs two keys");
        let mut bad = m;
        bad.input = "nope".into();
        assert!(drive(&[bad], &sig, 0.05).is_err());
    }

    #[test]
    fn landmark_ratio_is_scale_free() {
        let mk = |s: f32| -> HashMap<String, Vec2> {
            [
                ("up", Vec2::new(0.0, 0.0)),
                ("lo", Vec2::new(0.0, 4.0 * s)),
                ("top", Vec2::new(0.0, -20.0 * s)),
                ("chin", Vec2::new(0.0, 60.0 * s)),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect()
        };
        let a = landmark_ratio(&mk(1.0), "up", "lo", "top", "chin").unwrap();
        let b = landmark_ratio(&mk(3.0), "up", "lo", "top", "chin").unwrap();
        assert!((a - b).abs() < 1e-6 && (a - 0.05).abs() < 1e-6);
        assert!(landmark_ratio(&mk(1.0), "up", "x", "top", "chin").is_none());
    }

    fn arm(len1: f32, len2: f32) -> Project {
        let mut p = Project::new("a", 100.0, 100.0, 24.0, 10);
        p.bones = vec![
            Bone::new("u", None, len1, Vec2::ZERO, 0.0),
            Bone::new("l", Some("u"), len2, Vec2::new(len1, 0.0), 0.0),
            Bone::new("goal", None, 1.0, Vec2::new(len1 + len2, 0.0), 0.0),
        ];
        p.ik = vec![crate::project::IkChain::two_bone(
            "arm", "u", "l", "goal", 1.0,
        )];
        p
    }

    #[test]
    fn retarget_scales_effector_motion_by_reach_ratio() {
        let (src, dst) = (arm(30.0, 30.0), arm(60.0, 60.0));
        let ch = vec![
            Channel::keys(
                "goal",
                Prop::BoneOffsetX,
                Track::with_keys(&[(0.0, 0.0, Ease::Linear), (10.0, -20.0, Ease::Linear)]),
            ),
            Channel::keys(
                "u",
                Prop::BoneAngle,
                Track::with_keys(&[(0.0, 5.0, Ease::Linear)]),
            ),
        ];
        let (out, notes) =
            retarget_effectors(&ch, &src, &dst, &[("arm".into(), "arm".into())]).unwrap();
        assert!(notes.is_empty());
        let Source::Keys { track } = &out[0].source else {
            panic!()
        };
        assert_eq!(
            track.keys[1].value, -40.0,
            "reach ratio 2 doubles the motion"
        );
        let Source::Keys { track } = &out[1].source else {
            panic!()
        };
        assert_eq!(track.keys[0].value, 5.0, "non-effector channels untouched");
        assert!(retarget_effectors(&ch, &src, &dst, &[("nope".into(), "arm".into())]).is_err());
        assert_eq!(chain_reach(&dst, "arm"), Some(120.0));
    }
}
