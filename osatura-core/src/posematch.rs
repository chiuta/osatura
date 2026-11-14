// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Pose matching: find parameter values that put chosen bone tips where you want them.
//!
//! Clean-room design: a derivative-free (μ/μ_w, λ) evolution strategy with step-size
//! adaptation and elitism, driven by the deterministic [`Engine`]. Because the engine is a
//! pure function of the frame, a whole generation is evaluated by encoding candidate `i`
//! as the value held at frame `i` (step keys), so one compiled engine scores λ candidates.
//! The search starts from the rest pose plus the listed parameters (other channels and
//! springs are ignored — they depend on time).

use crate::anim::{Ease, Track};
use crate::engine::Engine;
use crate::math::Vec2;
use crate::project::{Channel, Project, Prop};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Param {
    pub target: String,
    pub prop: Prop,
    pub min: f32,
    pub max: f32,
    pub start: f32,
}

/// Where the tip (end) of `bone` should be in stage space.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Goal {
    pub bone: String,
    pub at: Vec2,
    pub weight: f32,
}

#[derive(Clone, Debug)]
pub struct MatchOptions {
    pub generations: usize,
    pub population: usize,
    pub seed: u64,
}

impl Default for MatchOptions {
    fn default() -> Self {
        MatchOptions {
            generations: 80,
            population: 24,
            seed: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MatchResult {
    pub values: Vec<f32>,
    /// Weighted sum of squared tip errors (px²).
    pub loss: f32,
    pub history: Vec<f32>,
    /// Constant (single-key) channels reproducing the pose.
    pub channels: Vec<Channel>,
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 40) as f32) / (1u64 << 24) as f32
    }
    fn normal(&mut self) -> f32 {
        let (u1, u2) = (self.next().max(1e-7), self.next());
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
    }
}

pub fn pose_match(
    project: &Project,
    params: &[Param],
    goals: &[Goal],
    opts: &MatchOptions,
) -> Result<MatchResult, String> {
    if params.is_empty() || goals.is_empty() {
        return Err("needs at least one parameter and one goal".into());
    }
    let lam = opts.population.clamp(4, 256);
    let bone_ix = |n: &str| {
        project
            .bones
            .iter()
            .position(|b| b.name == n)
            .ok_or_else(|| format!("unknown bone '{n}'"))
    };
    let goal_ix: Vec<usize> = goals
        .iter()
        .map(|g| bone_ix(&g.bone))
        .collect::<Result<_, _>>()?;
    for p in params {
        if p.min.partial_cmp(&p.max) != Some(std::cmp::Ordering::Less) {
            return Err(format!("parameter '{}' needs min < max", p.target));
        }
    }
    let mut base = project.clone();
    base.springs.clear();
    base.channels.clear();
    base.frames = base.frames.max(lam as u32 + 1);

    let score = |cands: &[Vec<f32>]| -> Result<Vec<f32>, String> {
        let mut p = base.clone();
        p.channels = params
            .iter()
            .enumerate()
            .map(|(k, prm)| {
                let keys: Vec<(f32, f32, Ease)> = cands
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i as f32, c[k], Ease::Step))
                    .collect();
                Channel::keys(&prm.target, prm.prop, Track::with_keys(&keys))
            })
            .collect();
        let eng = Engine::new(p).map_err(|e| e.join("; "))?;
        Ok((0..cands.len())
            .map(|i| {
                let fr = eng.evaluate(i as f32);
                goals
                    .iter()
                    .zip(&goal_ix)
                    .map(|(g, &bi)| {
                        let tip = fr.world[bi].apply(Vec2::new(project.bones[bi].length, 0.0));
                        g.weight * tip.distance(g.at).powi(2)
                    })
                    .sum()
            })
            .collect())
    };

    let n = params.len();
    let clampv = |k: usize, v: f32| v.clamp(params[k].min, params[k].max);
    let mut mean: Vec<f32> = params
        .iter()
        .enumerate()
        .map(|(k, p)| clampv(k, p.start))
        .collect();
    let mut sigma = 0.25f32;
    let mut rng = Rng(opts
        .seed
        .wrapping_mul(2654435761)
        .wrapping_add(88172645463325252));
    let mu = lam / 2;
    let wts: Vec<f32> = (0..mu)
        .map(|i| ((mu as f32 + 0.5).ln() - ((i + 1) as f32).ln()).max(1e-3))
        .collect();
    let wsum: f32 = wts.iter().sum();

    let mut best = (mean.clone(), score(&[mean.clone()])?[0]);
    let mut history = vec![best.1];
    for _ in 0..opts.generations {
        let mut cands: Vec<Vec<f32>> = vec![best.0.clone()]; // elitism
        for _ in 1..lam {
            cands.push(
                (0..n)
                    .map(|k| {
                        clampv(
                            k,
                            mean[k] + sigma * (params[k].max - params[k].min) * rng.normal(),
                        )
                    })
                    .collect(),
            );
        }
        let losses = score(&cands)?;
        let mut order: Vec<usize> = (0..lam).collect();
        order.sort_by(|&a, &b| losses[a].total_cmp(&losses[b]));
        let improved = losses[order[0]] < best.1 - 1e-9;
        if improved {
            best = (cands[order[0]].clone(), losses[order[0]]);
        }
        mean = (0..n)
            .map(|k| {
                order
                    .iter()
                    .take(mu)
                    .zip(&wts)
                    .map(|(&i, w)| cands[i][k] * w)
                    .sum::<f32>()
                    / wsum
            })
            .collect();
        sigma = if improved {
            (sigma * 1.1).min(0.5)
        } else {
            (sigma * 0.82).max(1e-4)
        };
        history.push(best.1);
        if best.1 < 1e-8 {
            break;
        }
    }
    let channels = params
        .iter()
        .zip(&best.0)
        .map(|(p, &v)| {
            Channel::keys(
                &p.target,
                p.prop,
                Track::with_keys(&[(0.0, v, Ease::Linear)]),
            )
        })
        .collect();
    Ok(MatchResult {
        values: best.0,
        loss: best.1,
        history,
        channels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Bone;

    fn arm() -> Project {
        let mut p = Project::new("a", 300.0, 300.0, 24.0, 4);
        p.bones = vec![
            Bone::new("u", None, 60.0, Vec2::new(100.0, 100.0), 0.0),
            Bone::new("l", Some("u"), 50.0, Vec2::new(60.0, 0.0), 0.0),
        ];
        p
    }

    fn tip(p: &Project, a: f32, b: f32) -> Vec2 {
        let mut q = p.clone();
        q.channels = vec![
            Channel::keys(
                "u",
                Prop::BoneAngle,
                Track::with_keys(&[(0.0, a, Ease::Linear)]),
            ),
            Channel::keys(
                "l",
                Prop::BoneAngle,
                Track::with_keys(&[(0.0, b, Ease::Linear)]),
            ),
        ];
        let e = Engine::new(q).unwrap();
        e.evaluate(0.0).world[1].apply(Vec2::new(50.0, 0.0))
    }

    fn params() -> Vec<Param> {
        vec![
            Param {
                target: "u".into(),
                prop: Prop::BoneAngle,
                min: -120.0,
                max: 120.0,
                start: 0.0,
            },
            Param {
                target: "l".into(),
                prop: Prop::BoneAngle,
                min: -150.0,
                max: 150.0,
                start: 0.0,
            },
        ]
    }

    #[test]
    fn finds_a_reachable_pose_deterministically() {
        let p = arm();
        let goal_pt = tip(&p, 35.0, -50.0);
        let goals = [Goal {
            bone: "l".into(),
            at: goal_pt,
            weight: 1.0,
        }];
        let r = pose_match(&p, &params(), &goals, &MatchOptions::default()).unwrap();
        let reached = tip(&p, r.values[0], r.values[1]);
        assert!(
            reached.distance(goal_pt) < 1.0,
            "reached {reached:?} vs {goal_pt:?}, loss {}",
            r.loss
        );
        assert!(
            r.history.windows(2).all(|w| w[1] <= w[0] + 1e-6),
            "elitism: loss never increases"
        );
        let r2 = pose_match(&p, &params(), &goals, &MatchOptions::default()).unwrap();
        assert_eq!(r.values, r2.values, "same seed, same answer");
        assert_eq!(r.channels.len(), 2);
    }

    #[test]
    fn respects_limits_and_reports_errors() {
        let p = arm();
        let far = Goal {
            bone: "l".into(),
            at: Vec2::new(1000.0, 100.0),
            weight: 1.0,
        };
        let r = pose_match(
            &p,
            &params(),
            &[far],
            &MatchOptions {
                generations: 30,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(r.values.iter().all(|v| v.abs() <= 150.0));
        assert!(r.loss > 1000.0, "unreachable goal leaves residual error");
        assert!(pose_match(&p, &[], &[], &MatchOptions::default()).is_err());
        let bad = Goal {
            bone: "nope".into(),
            at: Vec2::ZERO,
            weight: 1.0,
        };
        assert!(pose_match(&p, &params(), &[bad], &MatchOptions::default()).is_err());
    }
}
