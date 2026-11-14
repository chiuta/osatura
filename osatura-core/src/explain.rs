// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! "Why is it here?" — an ordered explanation of everything that determines a bone's or a
//! layer's value at a given frame: rest pose, channels (in evaluation order), limits, IK,
//! constraints, springs and the final result.

use crate::engine::Engine;
use crate::math::Vec2;
use crate::project::{Channel, Source};

#[derive(Clone, Debug, PartialEq)]
pub struct Explanation {
    pub target: String,
    pub frame: f32,
    pub lines: Vec<String>,
}

fn describe_channel(i: usize, c: &Channel, frame: f32) -> String {
    let mode = format!("{:?}", c.mode).to_lowercase();
    let src = match &c.source {
        Source::Keys { track } => match track.eval(frame) {
            Some(v) => format!("keys → {v:.3} ({} key(s), {:?} outside range)", track.keys.len(), track.extrapolate),
            None => "keys (empty)".into(),
        },
        Source::Expr { expr } => format!("expression `{expr}`"),
        Source::Noise { amp, freq, seed, octaves } => format!("noise ±{amp} at {freq} Hz (seed {seed}, {octaves} octave(s))"),
        Source::Driven { input, input_prop, .. } => format!("driven by {input}.{input_prop:?}"),
    };
    format!("channel #{i} {:?}: {src}; mode {mode}, weight {:.2}", c.prop, c.weight)
}

pub fn explain(engine: &Engine, target: &str, frame: f32) -> Result<Explanation, String> {
    let p = &engine.project;
    let mut lines = Vec::new();
    if let Some(bi) = p.bones.iter().position(|b| b.name == target) {
        let b = &p.bones[bi];
        lines.push(format!("bone '{}' (length {}), parent {}", b.name, b.length, b.parent.as_deref().map_or("none (root)".to_string(), |s| format!("'{s}'"))));
        let mut chain = vec![b.name.clone()];
        let mut cur = b.parent.clone();
        while let Some(c) = cur {
            chain.push(c.clone());
            cur = p.bones.iter().find(|x| x.name == c).and_then(|x| x.parent.clone());
        }
        lines.push(format!("ancestry: {}", chain.join(" ← ")));
        lines.push(format!("rest pose: offset ({:.2}, {:.2}), angle {:.2}°", b.rest_offset.x, b.rest_offset.y, b.rest_angle));
        let mut n = 0;
        for (i, c) in p.channels.iter().enumerate().filter(|(_, c)| c.target == target) {
            lines.push(describe_channel(i, c, frame));
            n += 1;
        }
        if n == 0 {
            lines.push("no channels animate this bone directly".into());
        }
        if let Some([lo, hi]) = b.angle_limits {
            lines.push(format!("angle limits: animated rotation is clamped to [{lo}, {hi}]°"));
        }
        if let Some([lo, hi]) = b.scale_limits {
            lines.push(format!("scale limits: [{lo}, {hi}]"));
        }
        for ik in p.ik.iter().filter(|c| c.bones.iter().any(|x| x == target)) {
            let pos = ik.bones.iter().position(|x| x == target).unwrap() + 1;
            lines.push(format!("IK chain '{}' (bone {pos} of {}) pulls toward target '{}' with bend {:+}, mix {:.2} (animate ik_mix to change)", ik.name, ik.bones.len(), ik.target, ik.bend, ik.mix));
        }
        if p.ik.iter().any(|c| c.target == target) {
            lines.push("this bone is the target of an IK chain".into());
        }
        for c in p.constraints.iter().filter(|c| c.bone == target) {
            lines.push(format!("constraint '{}' ({:?}) at weight {:.2}", c.name, c.kind, c.weight));
        }
        if let Some(s) = p.springs.iter().find(|s| s.bone == target) {
            lines.push(format!("spring: world rotation lags behind via a {:.1} Hz, damping {:.2} oscillator{}", s.frequency, s.damping, if s.warmup { " (pre-rolled for loops)" } else { "" }));
        }
        let fr = engine.evaluate(frame);
        let w = &fr.world[bi];
        let head = w.apply(Vec2::ZERO);
        let tip = w.apply(Vec2::new(b.length, 0.0));
        lines.push(format!("result at frame {frame}: head ({:.1}, {:.1}), tip ({:.1}, {:.1}), world angle {:.1}°", head.x, head.y, tip.x, tip.y, w.rotation().to_degrees()));
    } else if let Some(l) = p.find_layer(target) {
        lines.push(format!("layer '{target}': {}, opacity {:.2}, blend {:?}", if l.visible { "visible" } else { "hidden" }, l.opacity, l.blend));
        if let Some(b) = &l.bone {
            lines.push(format!("rides bone '{b}' rigidly"));
        }
        if l.parallax != 1.0 {
            lines.push(format!("parallax factor {} (camera moves it by that share)", l.parallax));
        }
        if !l.deformers.is_empty() {
            lines.push(format!("{} deformer(s) bend its artwork", l.deformers.len()));
        }
        if let Some(m) = &l.mask {
            lines.push(format!("masked: {m:?}"));
        }
        let mut n = 0;
        for (i, c) in p.channels.iter().enumerate().filter(|(_, c)| c.target == target || c.target.starts_with(&format!("{target}::"))) {
            lines.push(format!("{} (target '{}')", describe_channel(i, c, frame), c.target));
            n += 1;
        }
        if n == 0 {
            lines.push("no channels animate this layer".into());
        }
    } else {
        return Err(format!("'{target}' is neither a bone nor a layer path"));
    }
    Ok(Explanation { target: target.to_string(), frame, lines })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    #[test]
    fn explains_a_bone_in_evaluation_order() {
        let e = Engine::new(demo::project()).unwrap();
        let x = explain(&e, "arm_low", 12.0).unwrap();
        let text = x.lines.join("\n");
        assert!(text.contains("ancestry: arm_low ← arm_up"), "{text}");
        assert!(text.contains("channel #"), "{text}");
        assert!(text.contains("result at frame 12"), "{text}");
        // rest pose line comes before channels, result last
        assert!(x.lines[0].starts_with("bone 'arm_low'"));
        assert!(x.lines.last().unwrap().starts_with("result"));
    }

    #[test]
    fn mentions_ik_and_targets() {
        let e = Engine::new(demo::project()).unwrap();
        assert!(explain(&e, "shin", 0.0).unwrap().lines.iter().any(|l| l.contains("IK chain 'leg'")));
        assert!(explain(&e, "foot_ctrl", 0.0).unwrap().lines.iter().any(|l| l.contains("target of an IK chain")));
    }

    #[test]
    fn explains_layers_and_rejects_unknown() {
        let e = Engine::new(demo::project()).unwrap();
        let x = explain(&e, "mouth", 5.0).unwrap();
        assert!(x.lines.iter().any(|l| l.contains("LayerSwitch")), "{:?}", x.lines);
        assert!(explain(&e, "nothing_here", 0.0).is_err());
    }

    #[test]
    fn keyed_value_in_text_matches_the_track() {
        let e = Engine::new(demo::project()).unwrap();
        let ch = e.project.channels.iter().find(|c| c.target == "arm_up" && matches!(c.source, Source::Keys { .. })).unwrap();
        let Source::Keys { track } = &ch.source else { unreachable!() };
        let v = track.eval(6.0).unwrap();
        let x = explain(&e, "arm_up", 6.0).unwrap();
        assert!(x.lines.iter().any(|l| l.contains(&format!("{v:.3}"))), "{:?}", x.lines);
    }
}
