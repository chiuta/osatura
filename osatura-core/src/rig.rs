// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
use crate::math::{Affine2, Vec2};
use crate::project::Bone;
use std::collections::HashMap;

/// Per-bone animated deltas applied on top of the rest pose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BonePose {
    /// Extra rotation in radians.
    pub angle: f32,
    /// Uniform scale (inherited by children).
    pub scale: f32,
    /// Extra translation in the parent's space.
    pub offset: Vec2,
}

impl Default for BonePose {
    fn default() -> Self {
        BonePose {
            angle: 0.0,
            scale: 1.0,
            offset: Vec2::ZERO,
        }
    }
}

/// Resolve parent names to indices. Parents must be listed before children.
pub fn resolve_parents(bones: &[Bone]) -> Result<Vec<Option<usize>>, String> {
    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut out = Vec::with_capacity(bones.len());
    for (i, b) in bones.iter().enumerate() {
        if index.insert(b.name.as_str(), i).is_some() {
            return Err(format!("duplicate bone name '{}'", b.name));
        }
        out.push(match &b.parent {
            None => None,
            Some(p) => match index.get(p.as_str()) {
                Some(&pi) if pi < i => Some(pi),
                _ => {
                    return Err(format!(
                        "bone '{}' has parent '{}' which is missing or listed after it",
                        b.name, p
                    ))
                }
            },
        });
    }
    Ok(out)
}

/// Forward kinematics: world transform of every bone (bone-local x axis runs
/// along the bone, origin at its head).
pub fn fk(bones: &[Bone], parent: &[Option<usize>], poses: &[BonePose]) -> Vec<Affine2> {
    let mut world: Vec<Affine2> = Vec::with_capacity(bones.len());
    for (i, b) in bones.iter().enumerate() {
        let p = &poses[i];
        let local = Affine2::translate(b.rest_offset + p.offset)
            .mul(Affine2::rotate(b.rest_angle.to_radians() + p.angle))
            .mul(Affine2::scale(p.scale));
        let m = match parent[i] {
            Some(pi) => world[pi].mul(local),
            None => local,
        };
        world.push(m);
    }
    world
}

/// Wrap an angle to (−π, π].
pub fn wrap_pi(a: f32) -> f32 {
    let x = (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    if x <= -std::f32::consts::PI {
        std::f32::consts::PI
    } else {
        x
    }
}

/// Most FABRIK iterations the engine runs per chain and frame. A chain's own `iterations` is a floor:
/// the solver stops as soon as the tip is within [`FABRIK_TOLERANCE`] of the target, and otherwise
/// keeps going up to `max(iterations, FABRIK_MAX_ITERS)`, so an awkward pose with the default 16
/// iterations no longer stops up to a few pixels short. Deterministic (fixed update order).
pub const FABRIK_MAX_ITERS: usize = 256;
/// Tip-to-target distance (pixels) at which FABRIK stops early.
pub const FABRIK_TOLERANCE: f32 = 0.01;

/// FABRIK (forward-and-backward reaching IK) for a chain of joints.
/// `joints` has `n + 1` points (heads, then the tip), `lengths` has `n`.
/// The first joint stays fixed. Unreachable targets straighten the chain toward them.
pub fn solve_fabrik(
    joints: &mut [Vec2],
    lengths: &[f32],
    target: Vec2,
    iterations: usize,
    tolerance: f32,
) {
    let n = lengths.len();
    if n == 0 || joints.len() != n + 1 {
        return;
    }
    let root = joints[0];
    let total: f32 = lengths.iter().sum();
    let dir_or = |d: Vec2| {
        if d.length() < 1e-9 {
            Vec2::new(1.0, 0.0)
        } else {
            d.normalized()
        }
    };
    if root.distance(target) >= total {
        let dir = dir_or(target - root);
        for i in 0..n {
            joints[i + 1] = joints[i] + dir * lengths[i];
        }
        return;
    }
    for _ in 0..iterations.max(1) {
        joints[n] = target;
        for i in (0..n).rev() {
            joints[i] = joints[i + 1] + dir_or(joints[i] - joints[i + 1]) * lengths[i];
        }
        joints[0] = root;
        for i in 0..n {
            joints[i + 1] = joints[i] + dir_or(joints[i + 1] - joints[i]) * lengths[i];
        }
        if joints[n].distance(target) < tolerance {
            break;
        }
    }
}

/// Angular spring (damped harmonic oscillator) in world-angle space.
#[derive(Clone, Copy, Debug, Default)]
pub struct SpringState {
    pub x: f32,
    pub v: f32,
}

impl SpringState {
    pub fn at_rest(angle: f32) -> Self {
        SpringState { x: angle, v: 0.0 }
    }

    /// Advance by `dt` seconds toward `target` (semi-implicit Euler, `substeps` per call).
    pub fn step(
        &mut self,
        target: f32,
        frequency_hz: f32,
        damping_ratio: f32,
        dt: f32,
        substeps: u32,
    ) {
        let w = std::f32::consts::TAU * frequency_hz;
        let h = dt / substeps.max(1) as f32;
        for _ in 0..substeps.max(1) {
            let diff = wrap_pi(target - self.x);
            let acc = w * w * diff - 2.0 * damping_ratio * w * self.v;
            self.v += acc * h;
            self.x += self.v * h;
        }
    }
}

/// Distance from `p` to segment `a`–`b`.
pub fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let l2 = ab.dot(ab);
    if l2 < 1e-12 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / l2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// "Soft IK" distance remap: inside the last `soft` fraction of `reach` the extension approaches
/// full length exponentially instead of reaching it abruptly (the classic smooth-knee trick).
/// Continuous, monotone, never exceeds `reach`, and identical to `dist` below the soft zone.
pub fn soft_distance(dist: f32, reach: f32, soft: f32) -> f32 {
    let d = soft.clamp(0.0, 0.9) * reach;
    if d <= 1e-6 || dist <= reach - d {
        return dist.min(reach.max(0.0)).max(0.0).min(dist);
    }
    let x = dist - (reach - d);
    (reach - d) + d * (1.0 - (-x / d).exp())
}

/// Two-bone analytic IK in world space.
///
/// Returns the new `(root_angle, tip_angle)` pose deltas (radians) so that the
/// tip bone's end reaches `target` (clamped to reachable distance). `bend` is
/// +1 or −1 and picks which way the joint folds.
pub fn solve_two_bone(
    bones: &[Bone],
    parent: &[Option<usize>],
    world: &[Affine2],
    root: usize,
    tip: usize,
    target: Vec2,
    bend: f32,
) -> Option<(f32, f32)> {
    solve_two_bone_scaled(bones, parent, world, root, tip, target, bend, 1.0)
}

/// Like [`solve_two_bone`] but solves as if both segment lengths were multiplied by `scale`
/// (used for stretchy chains: the caller then applies the same scale to the root bone).
// public solver signature is stable (documented geometric inputs).
#[allow(clippy::too_many_arguments)]
pub fn solve_two_bone_scaled(
    bones: &[Bone],
    parent: &[Option<usize>],
    world: &[Affine2],
    root: usize,
    tip: usize,
    target: Vec2,
    bend: f32,
    scale: f32,
) -> Option<(f32, f32)> {
    let origin = world[root].apply(Vec2::ZERO);
    let joint = world[tip].apply(Vec2::ZERO);
    let la = origin.distance(joint) * scale;
    let lb = bones[tip].length * world[tip].x_scale() * scale;
    if la < 1e-4 || lb < 1e-4 {
        return None;
    }

    let to = target - origin;
    let dist = to.length();
    let (min_reach, max_reach) = ((la - lb).abs() + 1e-3, la + lb - 1e-3);
    if max_reach <= min_reach {
        return None;
    }
    let reach = dist.clamp(min_reach, max_reach);

    let cos_a = ((la * la + reach * reach - lb * lb) / (2.0 * la * reach)).clamp(-1.0, 1.0);
    let base = if dist < 1e-6 { 0.0 } else { to.y.atan2(to.x) };
    let world_a = base + bend.signum() * cos_a.acos();
    let elbow = origin + Vec2::from_angle(world_a) * la;
    let aim = origin + to.normalized() * reach;
    let d = aim - elbow;
    let world_b = d.y.atan2(d.x);

    let parent_rot = parent[root].map(|p| world[p].rotation()).unwrap_or(0.0);
    let a_delta = world_a - parent_rot - bones[root].rest_angle.to_radians();
    let b_delta = world_b - world_a - bones[tip].rest_angle.to_radians();
    Some((a_delta, b_delta))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_distance_is_identity_then_eases_to_reach() {
        assert_eq!(soft_distance(50.0, 100.0, 0.0), 50.0);
        assert_eq!(soft_distance(50.0, 100.0, 0.2), 50.0, "below the soft zone");
        let a = soft_distance(85.0, 100.0, 0.2);
        let b = soft_distance(95.0, 100.0, 0.2);
        let c = soft_distance(200.0, 100.0, 0.2);
        assert!(a < 85.0 + 1e-3 && a > 80.0 - 1e-3, "{a}");
        assert!(
            b > a && b < 100.0 && c < 100.0 && c > b,
            "monotone and bounded: {a} {b} {c}"
        );
        // continuity at the start of the zone
        assert!((soft_distance(80.0 + 1e-3, 100.0, 0.2) - 80.0).abs() < 1e-2);
        // slope near the start of the soft zone is 1 (C1 join)
        let eps = 0.01;
        assert!(
            ((soft_distance(80.0 + eps, 100.0, 0.2) - soft_distance(80.0, 100.0, 0.2)) / eps - 1.0)
                .abs()
                < 0.02
        );
    }

    #[test]
    fn wrap_pi_range() {
        for k in -20..20 {
            let a = k as f32 * 0.9;
            let w = wrap_pi(a);
            assert!(w > -std::f32::consts::PI - 1e-6 && w <= std::f32::consts::PI + 1e-6);
            assert!(((a - w) / std::f32::consts::TAU).round().abs() >= 0.0);
            assert!((w.sin() - a.sin()).abs() < 1e-4);
        }
    }

    #[test]
    fn fabrik_reaches_and_preserves_lengths() {
        let mut j = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(150.0, 0.0),
        ];
        let lens = [50.0, 50.0, 50.0];
        let target = Vec2::new(60.0, 70.0);
        solve_fabrik(&mut j, &lens, target, 32, 0.01);
        assert!(j[3].distance(target) < 0.05, "{:?}", j[3]);
        assert_eq!(j[0], Vec2::new(0.0, 0.0));
        for i in 0..3 {
            assert!((j[i].distance(j[i + 1]) - lens[i]).abs() < 0.05);
        }
    }

    #[test]
    fn fabrik_unreachable_straightens() {
        let mut j = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(20.0, 0.0),
        ];
        solve_fabrik(&mut j, &[10.0, 10.0], Vec2::new(0.0, 100.0), 8, 0.01);
        assert!(j[2].distance(Vec2::new(0.0, 20.0)) < 1e-4);
    }

    #[test]
    fn spring_converges_overshoots_and_is_deterministic() {
        let run = |zeta: f32| {
            let mut s = SpringState::at_rest(0.0);
            let mut peak: f32 = 0.0;
            for _ in 0..240 {
                s.step(1.0, 3.0, zeta, 1.0 / 24.0, 8);
                peak = peak.max(s.x);
            }
            (s.x, peak)
        };
        let (end, peak) = run(0.2);
        assert!(
            (end - 1.0).abs() < 0.01 && peak > 1.2,
            "underdamped overshoots: end {end} peak {peak}"
        );
        let (end, peak) = run(1.2);
        assert!(
            (end - 1.0).abs() < 0.01 && peak < 1.01,
            "overdamped never overshoots: peak {peak}"
        );
        assert_eq!(run(0.5), run(0.5));
    }

    #[test]
    fn spring_takes_the_short_way_around() {
        let mut s = SpringState::at_rest(3.0);
        s.step(-3.0, 4.0, 0.7, 1.0 / 24.0, 8); // target is 0.28 rad away across ±π, not 6 rad
        assert!(
            s.x > 3.0 || s.x < -3.0,
            "moves across the wrap, got {}",
            s.x
        );
    }
}
