// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Forward kinematics (part of `mocap`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Forward kinematics

pub(super) type M = [[f64; 4]; 3];

pub(super) fn mmul(a: &M, b: &M) -> M {
    let mut r = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..4 {
            let mut s = 0.0;
            for k in 0..3 {
                s += a[i][k] * b[k][j];
            }
            if j == 3 {
                s += a[i][3];
            }
            r[i][j] = s;
        }
    }
    r
}

pub(super) fn trans(v: V3) -> M {
    [[1.0, 0.0, 0.0, v[0]], [0.0, 1.0, 0.0, v[1]], [0.0, 0.0, 1.0, v[2]]]
}

/// Right-handed elementary rotation about axis 0/1/2 by `deg` degrees.
pub(super) fn rotm(axis: usize, deg: f64) -> M {
    let (s, c) = deg.to_radians().sin_cos();
    match axis {
        0 => [[1.0, 0.0, 0.0, 0.0], [0.0, c, -s, 0.0], [0.0, s, c, 0.0]],
        1 => [[c, 0.0, s, 0.0], [0.0, 1.0, 0.0, 0.0], [-s, 0.0, c, 0.0]],
        _ => [[c, -s, 0.0, 0.0], [s, c, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]],
    }
}

/// World positions of every joint head and End Site for one pose.
#[derive(Clone, Debug)]
pub struct Pose {
    pub pos: Vec<V3>,
    pub end: Vec<Option<V3>>,
}

/// Where a bone points: a joint, an End Site, or the parent direction continued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tip {
    Joint(usize),
    End(usize),
    Extrap(usize),
}

pub(super) fn dist3(a: V3, b: V3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl Bvh {
    pub fn fps(&self) -> f64 {
        1.0 / self.frame_time
    }
    pub fn duration(&self) -> f64 {
        self.frames.saturating_sub(1) as f64 * self.frame_time
    }
    pub fn joint_index(&self, name: &str) -> Option<usize> {
        self.joints.iter().position(|j| j.name == name)
    }

    /// Pose of a frame, or the rest pose (all channels zero) for `None`.
    pub fn pose(&self, frame: Option<usize>) -> Pose {
        let row: Option<&[f64]> = frame.filter(|&f| f < self.frames).map(|f| &self.data[f * self.num_channels..(f + 1) * self.num_channels]);
        let mut world: Vec<M> = Vec::with_capacity(self.joints.len());
        for j in &self.joints {
            let mut m = trans(j.offset);
            for (k, ch) in j.channels.iter().enumerate() {
                let v = row.map_or(0.0, |r| r[j.chan_start + k]);
                let c = match ch {
                    Chan::Xpos => trans([v, 0.0, 0.0]),
                    Chan::Ypos => trans([0.0, v, 0.0]),
                    Chan::Zpos => trans([0.0, 0.0, v]),
                    Chan::Xrot => rotm(0, v),
                    Chan::Yrot => rotm(1, v),
                    Chan::Zrot => rotm(2, v),
                };
                m = mmul(&m, &c);
            }
            let w = match j.parent {
                Some(p) => mmul(&world[p], &m),
                None => m,
            };
            world.push(w);
        }
        let pos = world.iter().map(|m| [m[0][3], m[1][3], m[2][3]]).collect();
        let end = self
            .joints
            .iter()
            .zip(&world)
            .map(|(j, w)| {
                j.end_site.map(|e| {
                    let m = mmul(w, &trans(e));
                    [m[0][3], m[1][3], m[2][3]]
                })
            })
            .collect();
        Pose { pos, end }
    }

    /// Default tip of a joint: its longest-reaching child, else its End Site, else the parent
    /// direction continued.
    pub fn default_tip(&self, j: usize) -> Tip {
        let reach = self.reach();
        self.default_tip_with(j, &reach)
    }

    pub(super) fn reach(&self) -> Vec<f64> {
        let mut h = vec![0.0f64; self.joints.len()];
        for j in (0..self.joints.len()).rev() {
            let jt = &self.joints[j];
            let mut best = jt.end_site.map_or(0.0, |e| dist3(e, [0.0; 3]));
            for &c in &jt.children {
                best = best.max(dist3(self.joints[c].offset, [0.0; 3]) + h[c]);
            }
            h[j] = best;
        }
        h
    }

    pub(super) fn default_tip_with(&self, j: usize, reach: &[f64]) -> Tip {
        let jt = &self.joints[j];
        let mut best: Option<(usize, f64)> = None;
        for &c in &jt.children {
            let r = dist3(self.joints[c].offset, [0.0; 3]) + reach[c];
            if best.is_none_or(|(_, br)| r > br + 1e-12) {
                best = Some((c, r));
            }
        }
        match best {
            Some((c, _)) => Tip::Joint(c),
            None if jt.end_site.is_some() => Tip::End(j),
            None => Tip::Extrap(j),
        }
    }

    pub fn tip_point(&self, pose: &Pose, tip: Tip) -> Option<V3> {
        match tip {
            Tip::Joint(c) => Some(pose.pos[c]),
            Tip::End(j) => pose.end[j],
            Tip::Extrap(j) => {
                let p = self.joints[j].parent?;
                let (a, b) = (pose.pos[p], pose.pos[j]);
                Some([2.0 * b[0] - a[0], 2.0 * b[1] - a[1], 2.0 * b[2] - a[2]])
            }
        }
    }

    /// Short human-readable description for `mocap-info`.
    pub fn summary(&self) -> String {
        let mut s = String::new();
        let n_end = self.joints.iter().filter(|j| j.end_site.is_some()).count();
        s.push_str(&format!("{} joints ({} end sites), {} channels per frame\n", self.joints.len(), n_end, self.num_channels));
        s.push_str(&format!("{} frames at {:.4} s ({:.3} fps), duration {:.3} s\n", self.frames, self.frame_time, self.fps(), self.duration()));
        fn walk(b: &Bvh, j: usize, depth: usize, s: &mut String) {
            let jt = &b.joints[j];
            let ch: Vec<&str> = jt
                .channels
                .iter()
                .map(|c| match c {
                    Chan::Xpos => "Xpos",
                    Chan::Ypos => "Ypos",
                    Chan::Zpos => "Zpos",
                    Chan::Xrot => "Xrot",
                    Chan::Yrot => "Yrot",
                    Chan::Zrot => "Zrot",
                })
                .collect();
            s.push_str(&format!("{}{}  offset ({:.3}, {:.3}, {:.3})  [{}]\n", "  ".repeat(depth), jt.name, jt.offset[0], jt.offset[1], jt.offset[2], ch.join(" ")));
            for &c in &jt.children {
                walk(b, c, depth + 1, s);
            }
        }
        for (i, j) in self.joints.iter().enumerate() {
            if j.parent.is_none() {
                walk(self, i, 0, &mut s);
            }
        }
        if self.frames > 0 {
            let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
            for f in 0..self.frames {
                for p in self.pose(Some(f)).pos {
                    for k in 0..3 {
                        lo[k] = lo[k].min(p[k]);
                        hi[k] = hi[k].max(p[k]);
                    }
                }
            }
            s.push_str(&format!("joint extent over the clip: x {:.2}..{:.2}, y {:.2}..{:.2}, z {:.2}..{:.2}\n", lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]));
        }
        s
    }
}
