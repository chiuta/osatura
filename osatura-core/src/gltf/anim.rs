// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Node animation for glTF 2.0: the sampling rules of the three interpolation modes (STEP, LINEAR, CUBICSPLINE), spherical interpolation of
//! rotations, the conversion of a 2D angle to a quaternion, and the encoding of keyed tracks as glTF animation samplers and channels.
//!
//! Basis (public): the Khronos glTF 2.0 specification, animation section and interpolation appendix, as summarised by the official
//! Khronos tutorial chapter "Animations" (only the tutorial was read; the normative text was not, see
//! docs/clean-room/specs/07-gltf-animatie-transformari-strat.md). Accessor details (component type, `min`/`max` of the time accessor) follow
//! what the existing bone-animation export in this module already writes.
//!
//! Not yet called by the exporter: the project's layer-to-node mapping for non-bone layers is not specified, so layer transform animation
//! is still frozen at frame 0 (see docs/clean-room/specs/INTREBARI.md).
#![allow(dead_code)]

use super::geometry::{f32s, Bin};
use serde_json::{json, Value};
use std::f64::consts::PI;

/// Interpolation mode of a track (glTF `interpolation`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interp {
    Step,
    Linear,
    CubicSpline,
}

impl Interp {
    pub fn name(self) -> &'static str {
        match self {
            Interp::Step => "STEP",
            Interp::Linear => "LINEAR",
            Interp::CubicSpline => "CUBICSPLINE",
        }
    }
}

/// One key of an `N`-component track. `tangents` = `(in, out)` and is used only by `CubicSpline` (missing = zero tangents).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key<const N: usize> {
    pub time: f64,
    pub value: [f64; N],
    pub tangents: Option<([f64; N], [f64; N])>,
}

impl<const N: usize> Key<N> {
    pub fn new(time: f64, value: [f64; N]) -> Key<N> {
        Key { time, value, tangents: None }
    }
}

/// Times must be non-empty, finite and strictly increasing; values finite.
pub fn validate<const N: usize>(keys: &[Key<N>]) -> Result<(), String> {
    if keys.is_empty() {
        return Err("a track needs at least one key".into());
    }
    for (i, k) in keys.iter().enumerate() {
        let tan_ok = k.tangents.is_none_or(|(a, b)| a.iter().chain(&b).all(|v| v.is_finite()));
        if !k.time.is_finite() || !k.value.iter().all(|v| v.is_finite()) || !tan_ok {
            return Err(format!("key {i} has a non-finite number"));
        }
        if i > 0 && k.time <= keys[i - 1].time {
            return Err(format!("key times must be strictly increasing (key {i}: {} after {})", k.time, keys[i - 1].time));
        }
    }
    Ok(())
}

/// Spherical interpolation of unit quaternions `(x, y, z, w)` along the shortest path: the second quaternion is negated when the dot product
/// is negative; nearly identical quaternions fall back to a normalised linear blend.
pub fn slerp(a: [f64; 4], b: [f64; 4], t: f64) -> [f64; 4] {
    let mut d: f64 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
    let mut b = b;
    if d < 0.0 {
        d = -d;
        b = b.map(|v| -v);
    }
    let (ka, kb) = if d > 0.9995 {
        (1.0 - t, t)
    } else {
        let theta = d.clamp(-1.0, 1.0).acos();
        let s = theta.sin();
        (((1.0 - t) * theta).sin() / s, (t * theta).sin() / s)
    };
    let q = [0, 1, 2, 3].map(|i| ka * a[i] + kb * b[i]);
    normalise(q)
}

/// The quaternion `(x, y, z, w)` scaled to length 1 (a zero quaternion becomes the identity).
pub fn normalise(q: [f64; 4]) -> [f64; 4] {
    let l = q.iter().map(|v| v * v).sum::<f64>().sqrt();
    if l > 0.0 && l.is_finite() {
        q.map(|v| v / l)
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}

/// Rotation by `angle` radians about the Z axis as a quaternion `(x, y, z, w)`.
pub fn quat_z(angle: f64) -> [f64; 4] {
    [0.0, 0.0, (angle / 2.0).sin(), (angle / 2.0).cos()]
}

/// Value of a track at `time` (held before the first and after the last key). `rotation` selects spherical interpolation for LINEAR
/// (the track must then have 4 components). Keys must pass [`validate`].
pub fn sample<const N: usize>(keys: &[Key<N>], interp: Interp, rotation: bool, time: f64) -> [f64; N] {
    if keys.len() == 1 || time <= keys[0].time {
        return keys[0].value;
    }
    let last = keys.len() - 1;
    if time >= keys[last].time {
        return keys[last].value;
    }
    let i = keys.iter().rposition(|k| k.time <= time).unwrap_or(0).min(last - 1);
    let (a, b) = (&keys[i], &keys[i + 1]);
    let dt = b.time - a.time;
    let t = (time - a.time) / dt;
    match interp {
        Interp::Step => a.value,
        Interp::Linear => {
            if rotation && N == 4 {
                let (qa, qb) = ([0, 1, 2, 3].map(|c| a.value[c % N]), [0, 1, 2, 3].map(|c| b.value[c % N]));
                let q = slerp(qa, qb, t);
                std::array::from_fn(|c| q[c % 4])
            } else {
                std::array::from_fn(|c| a.value[c] + (b.value[c] - a.value[c]) * t)
            }
        }
        Interp::CubicSpline => {
            let zero = [0.0; N];
            let m0 = a.tangents.map_or(zero, |t| t.1);
            let m1 = b.tangents.map_or(zero, |t| t.0);
            let (t2, t3) = (t * t, t * t * t);
            std::array::from_fn(|c| (2.0 * t3 - 3.0 * t2 + 1.0) * a.value[c] + (t3 - 2.0 * t2 + t) * dt * m0[c] + (-2.0 * t3 + 3.0 * t2) * b.value[c] + (t3 - t2) * dt * m1[c])
        }
    }
}

/// Inserts keys so that no step of a LINEAR rotation track turns by more than a quarter turn: spherical interpolation takes the shortest
/// way, so a key pair more than half a turn apart would otherwise rotate the wrong way. `keys` are `(time, angle in radians)`.
pub fn densify_rotation(keys: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(keys.len());
    for (i, &(t, a)) in keys.iter().enumerate() {
        if i > 0 {
            let (t0, a0) = keys[i - 1];
            if (a - a0).abs() > PI {
                let parts = ((a - a0).abs() / (PI / 2.0)).ceil() as usize;
                for k in 1..parts {
                    let f = k as f64 / parts as f64;
                    out.push((t0 + (t - t0) * f, a0 + (a - a0) * f));
                }
            }
        }
        out.push((t, a));
    }
    out
}

/// Which transform a channel animates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    Translation,
    Rotation,
    Scale,
}

impl Path {
    fn name(self) -> &'static str {
        match self {
            Path::Translation => "translation",
            Path::Rotation => "rotation",
            Path::Scale => "scale",
        }
    }
}

/// A 2D layer track ready to be written: a node, a path and the keys of the 2D quantity.
#[derive(Clone, Debug)]
pub enum NodeTrack {
    /// 2D position, written with z = 0.
    Translation { node: usize, interp: Interp, keys: Vec<Key<2>> },
    /// Angle in radians about Z; written as unit quaternions.
    Rotation { node: usize, interp: Interp, keys: Vec<Key<1>> },
    /// 2D scale, written with z = 1.
    Scale { node: usize, interp: Interp, keys: Vec<Key<2>> },
}

/// Appends the data of the tracks to `bin` and returns the glTF `samplers` and `channels` arrays. Errors: an empty track, times that are not
/// strictly increasing, non-finite values. A track with one key becomes a sampler with one input (and interpolation LINEAR).
pub(super) fn append_animation(bin: &mut Bin, tracks: &[NodeTrack]) -> Result<(Vec<Value>, Vec<Value>), String> {
    let (mut samplers, mut channels) = (Vec::new(), Vec::new());
    for tr in tracks {
        let (node, path, interp, times, data, ty): (usize, Path, Interp, Vec<f64>, Vec<f32>, &str) = match tr {
            NodeTrack::Translation { node, interp, keys } => {
                validate(keys)?;
                let mut d = Vec::new();
                for k in keys {
                    if *interp == Interp::CubicSpline {
                        let (i, o) = k.tangents.unwrap_or(([0.0; 2], [0.0; 2]));
                        d.extend([i[0], i[1], 0.0, k.value[0], k.value[1], 0.0, o[0], o[1], 0.0].map(|v| v as f32));
                    } else {
                        d.extend([k.value[0], k.value[1], 0.0].map(|v| v as f32));
                    }
                }
                (*node, Path::Translation, *interp, keys.iter().map(|k| k.time).collect(), d, "VEC3")
            }
            NodeTrack::Scale { node, interp, keys } => {
                validate(keys)?;
                let mut d = Vec::new();
                for k in keys {
                    if *interp == Interp::CubicSpline {
                        let (i, o) = k.tangents.unwrap_or(([0.0; 2], [0.0; 2]));
                        d.extend([i[0], i[1], 0.0, k.value[0], k.value[1], 1.0, o[0], o[1], 0.0].map(|v| v as f32));
                    } else {
                        d.extend([k.value[0], k.value[1], 1.0].map(|v| v as f32));
                    }
                }
                (*node, Path::Scale, *interp, keys.iter().map(|k| k.time).collect(), d, "VEC3")
            }
            NodeTrack::Rotation { node, interp, keys } => {
                validate(keys)?;
                let mut d = Vec::new();
                let q_of = |k: &Key<1>| quat_z(k.value[0]);
                for k in keys {
                    if *interp == Interp::CubicSpline {
                        return Err("CUBICSPLINE rotation is not supported (the tangent mapping for angles is not specified)".into());
                    } else {
                        d.extend(q_of(k).map(|v| v as f32));
                    }
                }
                (*node, Path::Rotation, *interp, keys.iter().map(|k| k.time).collect(), d, "VEC4")
            }
        };
        let times32: Vec<f32> = times.iter().map(|t| *t as f32).collect();
        let (tmin, tmax) = (times32[0], times32[times32.len() - 1]);
        let t_acc = bin.add(&f32s(&times32), 5126, times32.len(), "SCALAR", Some((vec![tmin], vec![tmax])), None);
        let per_key = if interp == Interp::CubicSpline { 3 } else { 1 };
        let o_acc = bin.add(&f32s(&data), 5126, times32.len() * per_key, ty, None, None);
        let mode = if times32.len() == 1 && interp == Interp::CubicSpline { Interp::Linear } else { interp };
        samplers.push(json!({"input": t_acc, "output": o_acc, "interpolation": mode.name()}));
        channels.push(json!({"sampler": samplers.len() - 1, "target": {"node": node, "path": path.name()}}));
    }
    Ok((samplers, channels))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-5, "{a} vs {b}");
    }

    fn two() -> Vec<Key<2>> {
        vec![Key::new(0.0, [0.0, 0.0]), Key::new(2.0, [10.0, 0.0])]
    }

    #[test]
    fn linear_and_step() {
        let v = sample(&two(), Interp::Linear, false, 1.0);
        close(v[0], 5.0);
        close(v[1], 0.0);
        assert_eq!(sample(&two(), Interp::Step, false, 1.99), [0.0, 0.0]);
        assert_eq!(sample(&two(), Interp::Step, false, 2.0), [10.0, 0.0]);
        // held outside the key range
        assert_eq!(sample(&two(), Interp::Linear, false, -3.0), [0.0, 0.0]);
        assert_eq!(sample(&two(), Interp::Linear, false, 9.0), [10.0, 0.0]);
    }

    #[test]
    fn cubic_spline_with_tangents() {
        let k = vec![Key { time: 0.0, value: [0.0], tangents: Some(([0.0], [10.0])) }, Key { time: 2.0, value: [10.0], tangents: Some(([0.0], [0.0])) }];
        close(sample(&k, Interp::CubicSpline, false, 1.0)[0], 7.5);
        let flat = vec![Key::new(0.0, [0.0]), Key::new(2.0, [10.0])];
        close(sample(&flat, Interp::CubicSpline, false, 1.0)[0], 5.0);
    }

    #[test]
    fn rotation_quaternion_and_slerp() {
        let q = quat_z(PI / 2.0);
        close(q[2], std::f64::consts::FRAC_1_SQRT_2);
        close(q[3], std::f64::consts::FRAC_1_SQRT_2);
        assert_eq!((q[0], q[1]), (0.0, 0.0));
        let half = slerp([0.0, 0.0, 0.0, 1.0], q, 0.5);
        for (g, w) in half.iter().zip([0.0, 0.0, 0.382_683, 0.923_880]) {
            close(*g, w);
        }
        // opposite sign: same rotation, shortest path
        let neg = slerp([0.0, 0.0, 0.0, 1.0], [0.0, 0.0, -std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2], 0.5);
        for (g, w) in neg.iter().zip([0.0, 0.0, 0.382_683, 0.923_880]) {
            close(*g, w);
        }
        // sampled through a rotation track
        let keys = vec![Key::new(0.0, [0.0, 0.0, 0.0, 1.0]), Key::new(1.0, q)];
        let mid = sample(&keys, Interp::Linear, true, 0.5);
        close(mid[2], 0.382_683);
        // nearly identical quaternions: normalised linear blend
        let near = slerp([0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1e-4, 1.0], 0.5);
        close(near.iter().map(|v| v * v).sum::<f64>(), 1.0);
    }

    #[test]
    fn invalid_tracks() {
        assert!(validate::<2>(&[]).is_err());
        assert!(validate(&[Key::new(1.0, [0.0]), Key::new(1.0, [1.0])]).unwrap_err().contains("strictly"));
        assert!(validate(&[Key::new(1.0, [0.0]), Key::new(0.5, [1.0])]).is_err());
        assert!(validate(&[Key::new(f64::NAN, [0.0])]).is_err());
        assert!(validate(&[Key::new(0.0, [f64::INFINITY])]).is_err());
        assert!(validate(&[Key::new(0.0, [1.0])]).is_ok());
    }

    #[test]
    fn long_turns_get_intermediate_keys() {
        let dense = densify_rotation(&[(0.0, 0.0), (1.0, 3.0 * PI / 2.0), (2.0, 3.0 * PI / 2.0 + 0.1)]);
        assert!(dense.len() > 3);
        assert!(dense.windows(2).all(|w| (w[1].1 - w[0].1).abs() <= PI / 2.0 + 1e-9));
        assert_eq!(dense[0], (0.0, 0.0));
        assert_eq!(*dense.last().unwrap(), (2.0, 3.0 * PI / 2.0 + 0.1));
        // times stay strictly increasing
        assert!(dense.windows(2).all(|w| w[1].0 > w[0].0));
        // short turns are left alone
        assert_eq!(densify_rotation(&[(0.0, 0.0), (1.0, 1.0)]).len(), 2);
    }

    fn bin() -> Bin {
        Bin { data: Vec::new(), views: Vec::new(), accessors: Vec::new() }
    }

    fn read_f32(b: &Bin, acc: usize) -> Vec<f32> {
        let a = &b.accessors[acc];
        let v = &b.views[a["bufferView"].as_u64().unwrap() as usize];
        let off = v["byteOffset"].as_u64().unwrap() as usize;
        let len = v["byteLength"].as_u64().unwrap() as usize;
        b.data[off..off + len].chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
    }

    #[test]
    fn encodes_samplers_and_channels() {
        let mut b = bin();
        let tracks = vec![
            NodeTrack::Translation { node: 3, interp: Interp::Linear, keys: two() },
            NodeTrack::Rotation { node: 3, interp: Interp::Step, keys: vec![Key::new(0.0, [0.0]), Key::new(2.0, [PI / 2.0])] },
            NodeTrack::Scale { node: 4, interp: Interp::Linear, keys: vec![Key::new(0.5, [2.0, 3.0])] },
        ];
        let (s, c) = append_animation(&mut b, &tracks).unwrap();
        assert_eq!((s.len(), c.len()), (3, 3));
        // two channels may target the same node
        assert_eq!(c[0]["target"], json!({"node": 3, "path": "translation"}));
        assert_eq!(c[1]["target"], json!({"node": 3, "path": "rotation"}));
        assert_eq!(c[2]["target"], json!({"node": 4, "path": "scale"}));
        assert_eq!(s[1]["interpolation"], "STEP");
        // translation: 2 keys, VEC3, z = 0
        let t_in = s[0]["input"].as_u64().unwrap() as usize;
        let t_out = s[0]["output"].as_u64().unwrap() as usize;
        assert_eq!(read_f32(&b, t_in), vec![0.0, 2.0]);
        assert_eq!(read_f32(&b, t_out), vec![0.0, 0.0, 0.0, 10.0, 0.0, 0.0]);
        assert_eq!(b.accessors[t_in]["min"], json!([0.0]));
        assert_eq!(b.accessors[t_in]["max"], json!([2.0]));
        assert_eq!(b.accessors[t_out]["type"], "VEC3");
        assert_eq!(b.accessors[t_out]["componentType"], 5126);
        // rotation: unit quaternions
        let q = read_f32(&b, s[1]["output"].as_u64().unwrap() as usize);
        assert_eq!(q.len(), 8);
        assert!((f64::from(q[6]) - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-5 && (f64::from(q[7]) - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-5);
        assert!(q.chunks(4).all(|c| (c.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 1e-6));
        // scale: z = 1; a single key
        assert_eq!(read_f32(&b, s[2]["output"].as_u64().unwrap() as usize), vec![2.0, 3.0, 1.0]);
    }

    #[test]
    fn cubic_spline_output_has_three_elements_per_key() {
        let mut b = bin();
        let keys = vec![Key { time: 0.0, value: [1.0, 2.0], tangents: Some(([0.1, 0.2], [0.3, 0.4])) }, Key { time: 1.0, value: [3.0, 4.0], tangents: Some(([0.5, 0.6], [0.7, 0.8])) }];
        let (s, _) = append_animation(&mut b, &[NodeTrack::Translation { node: 0, interp: Interp::CubicSpline, keys }]).unwrap();
        let out = s[0]["output"].as_u64().unwrap() as usize;
        assert_eq!(b.accessors[out]["count"], 6);
        let d = read_f32(&b, out);
        // in-tangent, value, out-tangent for the first key
        assert_eq!(&d[..9], &[0.1, 0.2, 0.0, 1.0, 2.0, 0.0, 0.3, 0.4, 0.0]);
    }

    #[test]
    fn encoding_errors() {
        let mut b = bin();
        assert!(append_animation(&mut b, &[NodeTrack::Translation { node: 0, interp: Interp::Linear, keys: vec![] }]).is_err());
        let dup = vec![Key::new(1.0, [0.0, 0.0]), Key::new(1.0, [1.0, 1.0])];
        assert!(append_animation(&mut b, &[NodeTrack::Scale { node: 0, interp: Interp::Linear, keys: dup }]).is_err());
        assert!(append_animation(&mut b, &[NodeTrack::Rotation { node: 0, interp: Interp::Linear, keys: vec![Key::new(0.0, [f64::NAN])] }]).is_err());
        assert!(append_animation(&mut b, &[NodeTrack::Rotation { node: 0, interp: Interp::CubicSpline, keys: vec![Key::new(0.0, [0.0])] }]).is_err());
    }
}
