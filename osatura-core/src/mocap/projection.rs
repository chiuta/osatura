// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Projection to 2D (part of `mocap`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Projection to 2D

/// How the 3D skeleton is flattened. BVH is y-up; the result uses stage coordinates (y down).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum View {
    /// Looking at a character that faces +Z: stage x = x.
    Front,
    /// stage x = z (a character facing +Z faces right).
    Side,
    /// Seen from above: stage x = x, stage y = -z (forward is up the screen).
    Top,
    /// Front view after turning the skeleton by this many degrees about the vertical axis.
    Yaw(f64),
}

impl View {
    pub fn parse(s: &str) -> Result<View, String> {
        match s {
            "front" => Ok(View::Front),
            "side" => Ok(View::Side),
            "top" => Ok(View::Top),
            _ => {
                let d = s.strip_prefix("yaw=").or_else(|| s.strip_prefix("yaw:")).ok_or_else(|| format!("unknown view '{s}' (front, side, top or yaw=DEG)"))?;
                let v: f64 = d.parse().map_err(|_| format!("bad yaw angle '{d}'"))?;
                if !v.is_finite() {
                    return Err(format!("bad yaw angle '{d}'"));
                }
                Ok(View::Yaw(v))
            }
        }
    }
    pub fn project(&self, p: V3) -> [f64; 2] {
        match *self {
            View::Front => [p[0], -p[1]],
            View::Side => [p[2], -p[1]],
            View::Top => [p[0], -p[2]],
            View::Yaw(d) => {
                let (s, c) = d.to_radians().sin_cos();
                [p[0] * c + p[2] * s, -p[1]]
            }
        }
    }
}

/// Angle in degrees (stage convention, y down, so positive turns clockwise on screen) of the
/// bone `a -> b` after projection. `None` when the bone points (nearly) along the view axis.
pub fn angle_2d(a: V3, b: V3, view: View, mirror: bool) -> Option<f64> {
    let (pa, pb) = (view.project(a), view.project(b));
    let (mut dx, dy) = (pb[0] - pa[0], pb[1] - pa[1]);
    if mirror {
        dx = -dx;
    }
    let l3 = dist3(a, b);
    let l2 = (dx * dx + dy * dy).sqrt();
    if l3 < 1e-9 || l2 < 0.05 * l3 {
        return None;
    }
    Some(dy.atan2(dx).to_degrees())
}

/// Per-frame 2D data for the whole BVH.
#[derive(Clone, Debug)]
pub struct Projected {
    pub frame_time: f64,
    /// `points[frame][joint]`
    pub points: Vec<Vec<[f64; 2]>>,
    /// `angles[frame][joint]`: angle of the bone from the joint to its default tip.
    pub angles: Vec<Vec<Option<f64>>>,
}

pub fn project_motion(b: &Bvh, view: View) -> Projected {
    let reach = b.reach();
    let tips: Vec<Tip> = (0..b.joints.len()).map(|j| b.default_tip_with(j, &reach)).collect();
    let mut points = Vec::with_capacity(b.frames);
    let mut angles = Vec::with_capacity(b.frames);
    for f in 0..b.frames {
        let pose = b.pose(Some(f));
        points.push(pose.pos.iter().map(|&p| view.project(p)).collect());
        angles.push((0..b.joints.len()).map(|j| b.tip_point(&pose, tips[j]).and_then(|t| angle_2d(pose.pos[j], t, view, false))).collect());
    }
    Projected { frame_time: b.frame_time, points, angles }
}
