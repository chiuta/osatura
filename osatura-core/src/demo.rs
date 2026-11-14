// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Built-in demo: an FK-waved arm, an IK-driven leg, multiplane parallax camera,
//! gradients, blend modes, variable-width ink, a switch-layer mouth, expression /
//! noise / driven channels.

use crate::anim::{Ease, Extrapolate, Track};
use crate::math::{Affine2, Vec2};
use crate::path::{Anchor, BoneWeight, VPath};
use crate::project::{Bone, Channel, IkChain, Layer, Mode, Project, Prop};
use crate::style::{Blend, Cap, Paint, Rgba, StrokeStyle};

fn bone(name: &str, parent: Option<&str>, length: f32, off: (f32, f32), rest: f32) -> Bone {
    Bone::new(name, parent, length, Vec2::new(off.0, off.1), rest)
}

/// Closed, smoothed outline around a bone chain (rest pose), tapering from
/// `r0` at the first head to `r1` at the last tip.
pub fn limb_outline(name: &str, rest: &[Affine2], bones: &[Bone], ids: &[usize], r0: f32, r1: f32) -> VPath {
    let dir = |i: usize| Vec2::from_angle(rest[i].rotation());
    let mut samples: Vec<(Vec2, Vec2)> = Vec::new();
    for (k, &i) in ids.iter().enumerate() {
        let head = rest[i].apply(Vec2::ZERO);
        let d = if k == 0 { dir(i) } else { (dir(ids[k - 1]) + dir(i)).normalized() };
        samples.push((head, d));
        samples.push((rest[i].apply(Vec2::new(bones[i].length * 0.5, 0.0)), dir(i)));
    }
    let last = *ids.last().unwrap();
    samples.push((rest[last].apply(Vec2::new(bones[last].length, 0.0)), dir(last)));

    let n = samples.len();
    let radius = |k: usize| r0 + (r1 - r0) * (k as f32 / (n - 1) as f32);
    let normal = |d: Vec2| Vec2::new(-d.y, d.x);
    let top: Vec<Vec2> = samples.iter().enumerate().map(|(k, &(c, d))| c + normal(d) * radius(k)).collect();
    let bot: Vec<Vec2> = samples.iter().enumerate().map(|(k, &(c, d))| c - normal(d) * radius(k)).collect();
    let pts: Vec<Vec2> = top.into_iter().chain(bot.into_iter().rev()).collect();
    VPath::smooth_through(name, &pts, true, 1.0)
}

fn hex(s: &str) -> Rgba {
    Rgba::hex(s)
}

pub fn project() -> Project {
    let bones = vec![
        // FK arm
        bone("arm_up", None, 110.0, (150.0, 330.0), -70.0),
        bone("arm_low", Some("arm_up"), 100.0, (110.0, 0.0), -10.0),
        bone("hand", Some("arm_low"), 45.0, (100.0, 0.0), 0.0),
        // IK leg (rest pose points straight down)
        bone("thigh", None, 120.0, (440.0, 170.0), 90.0),
        bone("shin", Some("thigh"), 120.0, (120.0, 0.0), 0.0),
        // free control bone the foot is pulled toward
        bone("foot_ctrl", None, 10.0, (440.0, 385.0), 0.0),
    ];

    let mut p = Project::new("OSATURA demo", 640.0, 480.0, 24.0, 48);
    p.background = Some(hex("#f6f1e7"));
    p.bones = bones;
    p.ik = vec![IkChain::two_bone("leg", "thigh", "shin", "foot_ctrl", -1.0)];

    let rest = p.rest_world().expect("demo rig is valid");

    // --- backdrop (parallax layers wider than the stage so the camera can pan) ---
    let sky_paint = Paint::linear(
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 480.0),
        &[(0.0, "#9ec5e8"), (0.65, "#f7e3c4"), (1.0, "#f6f1e7")],
    );
    let mut sky = Layer::vector("sky", vec![VPath::rect("sky", -300.0, 0.0, 1240.0, 480.0).with_paint(sky_paint)]);
    sky.parallax = 0.15;
    let mut sun = Layer::vector(
        "sun",
        vec![VPath::circle("sun", Vec2::new(520.0, 90.0), 70.0).with_paint(Paint::radial(
            Vec2::new(520.0, 90.0),
            70.0,
            &[(0.0, "#fffbe6"), (0.5, "#ffd98a"), (1.0, "#ffd98a00")],
        ))],
    );
    sun.parallax = 0.2;
    sun.blend = Blend::Screen;
    let hills_pts: Vec<Vec2> = (0..=12)
        .map(|i| {
            let x = -250.0 + i as f32 * 90.0;
            Vec2::new(x, 350.0 - 40.0 * ((i as f32) * 0.9).sin() - 20.0)
        })
        .chain([Vec2::new(830.0, 480.0), Vec2::new(-250.0, 480.0)])
        .collect();
    let mut hills = Layer::vector(
        "hills",
        vec![VPath::smooth_through("hills", &hills_pts, true, 0.6).with_fill("#c3d3a8").with_stroke("#a7ba8a", 2.0)],
    );
    hills.parallax = 0.5;
    hills.opacity = 0.95;

    // --- foreground rig layers ---
    let arm_fill = Paint::linear(Vec2::new(150.0, 230.0), Vec2::new(150.0, 340.0), &[(0.0, "#f2b98f"), (1.0, "#e08f5f")]);
    let leg_fill = Paint::linear(Vec2::new(380.0, 170.0), Vec2::new(500.0, 170.0), &[(0.0, "#9db8d6"), (1.0, "#6d8fb8")]);
    let arm = limb_outline("arm", &rest, &p.bones, &[0, 1, 2], 24.0, 12.0)
        .with_paint(arm_fill)
        .with_stroke("#3d2c2e", 3.0);
    let leg = limb_outline("leg", &rest, &p.bones, &[3, 4], 26.0, 14.0)
        .with_paint(leg_fill)
        .with_stroke("#3d2c2e", 3.0);

    let mut marker = VPath::circle("foot target", Vec2::new(440.0, 385.0), 8.0)
        .with_fill("#d6336c")
        .with_stroke("#3d2c2e", 2.0);
    for a in &mut marker.anchors {
        a.weights = vec![BoneWeight { bone: "foot_ctrl".into(), weight: 1.0 }];
    }
    let mut dash = StrokeStyle::solid("#8a7f72", 4.0);
    dash.cap = Cap::Butt;
    dash.dash = vec![14.0, 8.0];
    let ground = VPath::new(
        "ground",
        false,
        vec![Anchor::corner(Vec2::new(-200.0, 412.0)), Anchor::corner(Vec2::new(840.0, 412.0))],
    )
    .with_stroke_style(dash);

    // variable-width ink swoosh, swayed by an expression
    let mut ink_stroke = StrokeStyle::new(
        Paint::linear(Vec2::new(60.0, 80.0), Vec2::new(300.0, 120.0), &[(0.0, "#6a4c93"), (1.0, "#d6336c")]),
        16.0,
    );
    ink_stroke.width_profile = vec![[0.0, 0.05], [0.3, 1.0], [0.7, 0.6], [1.0, 0.02]];
    let swoosh = VPath::smooth_through(
        "swoosh",
        &[Vec2::new(60.0, 110.0), Vec2::new(130.0, 70.0), Vec2::new(210.0, 100.0), Vec2::new(300.0, 70.0)],
        false,
        1.0,
    )
    .with_stroke_style(ink_stroke);
    let mut ink = Layer::vector("ink", vec![swoosh]);
    ink.transform.anchor = Vec2::new(60.0, 110.0);
    ink.transform.position = Vec2::new(60.0, 110.0);

    // soft shadow under the leg (multiply)
    let mut shadow = Layer::vector(
        "shadow",
        vec![VPath::ellipse("shadow", Vec2::new(440.0, 412.0), 52.0, 9.0).with_paint(Paint::radial(
            Vec2::new(440.0, 412.0),
            52.0,
            &[(0.0, "#3d2c2eaa"), (1.0, "#3d2c2e00")],
        ))],
    );
    shadow.blend = Blend::Multiply;
    shadow.transform.anchor = Vec2::new(440.0, 412.0);
    shadow.transform.position = Vec2::new(440.0, 412.0);

    // blush on the arm: scale driven by the elbow angle
    let mut blush = Layer::vector(
        "blush",
        vec![VPath::circle("blush", Vec2::ZERO, 9.0).with_paint(Paint::radial(
            Vec2::ZERO,
            9.0,
            &[(0.0, "#ff6b6bcc"), (1.0, "#ff6b6b00")],
        ))],
    );
    blush.bone = Some("arm_low".into());
    blush.transform.position = rest[1].apply(Vec2::new(40.0, 0.0));

    // mouth: a switch layer attached to the hand
    let mouth_shapes = vec![
        Layer::vector("closed", vec![VPath::new(
            "m",
            false,
            vec![Anchor::corner(Vec2::new(-8.0, 0.0)), Anchor::corner(Vec2::new(8.0, 0.0))],
        )
        .with_stroke("#3d2c2e", 3.0)]),
        Layer::vector("open", vec![VPath::ellipse("m", Vec2::ZERO, 8.0, 6.0).with_fill("#3d2c2e")]),
        Layer::vector("smile", vec![VPath::smooth_through(
            "m",
            &[Vec2::new(-9.0, -2.0), Vec2::new(0.0, 5.0), Vec2::new(9.0, -2.0)],
            false,
            1.0,
        )
        .with_stroke("#3d2c2e", 3.0)]),
    ];
    let mut mouth = Layer::switch("mouth", mouth_shapes);
    mouth.bone = Some("hand".into());
    mouth.transform.position = rest[2].apply(Vec2::new(24.0, 0.0));

    p.layers = vec![
        sky,
        sun,
        hills,
        Layer::vector("ground", vec![ground]),
        shadow,
        Layer::vector("arm", vec![arm]),
        Layer::vector("leg", vec![leg]),
        Layer::vector("target", vec![marker]),
        blush,
        mouth,
        ink,
    ];
    p.auto_skin("arm", 2).unwrap();
    p.auto_skin("leg", 2).unwrap();

    // --- animation (every looping channel has first key == last key) ---
    let ease = Ease::EaseInOut;
    let keys = |target: &str, prop: Prop, ks: &[(f32, f32)]| {
        Channel::keys(target, prop, Track::with_keys(&ks.iter().map(|&(f, v)| (f, v, ease)).collect::<Vec<_>>()))
    };
    p.channels = vec![
        keys("arm_up", Prop::BoneAngle, &[(0.0, 0.0), (12.0, -25.0), (24.0, 0.0), (36.0, 15.0), (48.0, 0.0)]),
        keys("arm_low", Prop::BoneAngle, &[(0.0, 0.0), (12.0, 35.0), (24.0, -15.0), (36.0, 30.0), (48.0, 0.0)]),
        keys("hand", Prop::BoneAngle, &[(0.0, 0.0), (12.0, 20.0), (24.0, -20.0), (36.0, 20.0), (48.0, 0.0)]),
        // procedural wiggle layered on the hand (additive; zero-mean so the loop stays tidy)
        Channel::expr("hand", Prop::BoneAngle, "6 * sin(time * 2 * pi * 2)", Mode::Add),
        keys("foot_ctrl", Prop::BoneOffsetX, &[(0.0, -40.0), (12.0, 0.0), (24.0, 40.0), (36.0, 0.0), (48.0, -40.0)]),
        keys("foot_ctrl", Prop::BoneOffsetY, &[(0.0, 0.0), (12.0, -45.0), (24.0, 0.0), (36.0, 0.0), (48.0, 0.0)]),
        keys("target", Prop::LayerOpacity, &[(0.0, 0.4), (24.0, 1.0), (48.0, 0.4)]),
        // shadow shrinks when the foot lifts (expression reads the foot's height)
        Channel::expr("shadow", Prop::LayerScaleX, "1 + get(\"foot_ctrl\", \"bone_offset_y\") / 160", Mode::Replace),
        Channel::expr("shadow", Prop::LayerX, "440 + get(\"foot_ctrl\", \"bone_offset_x\")", Mode::Replace),
        // blush grows as the elbow bends (driven key)
        Channel::driven(
            "blush",
            Prop::LayerScaleX,
            "arm_low",
            Prop::BoneAngle,
            Track::with_keys(&[(-15.0, 0.6, Ease::Linear), (35.0, 2.2, Ease::Linear)]).looped(Extrapolate::Hold),
        ),
        Channel::driven(
            "blush",
            Prop::LayerScaleY,
            "arm_low",
            Prop::BoneAngle,
            Track::with_keys(&[(-15.0, 0.6, Ease::Linear), (35.0, 2.2, Ease::Linear)]),
        ),
        // mouth swaps with step keys
        Channel::keys(
            "mouth",
            Prop::LayerSwitch,
            Track::with_keys(&[(0.0, 0.0, Ease::Step), (10.0, 1.0, Ease::Step), (16.0, 2.0, Ease::Step), (30.0, 1.0, Ease::Step), (38.0, 0.0, Ease::Step), (48.0, 0.0, Ease::Step)]),
        ),
        // ink swoosh sways, loops seamlessly
        Channel::expr("ink", Prop::LayerRotation, "8 * sin(t * 2 * pi)", Mode::Replace),
        // slow camera pan
        keys("", Prop::CameraX, &[(0.0, 320.0), (24.0, 350.0), (48.0, 320.0)]),
    ];
    p
}

/// Second demo: liquid metaballs bound to bones, a boolean-cut frame and a particle fountain.
pub fn fx_project() -> Project {
    use crate::fx::{Blob, BoolOp, EmitterSpec};
    let mut p = Project::new("OSATURA fx demo", 640.0, 480.0, 24.0, 72);
    p.background = Some(Rgba::hex("#10182b"));
    p.bones = vec![
        Bone::new("a", None, 10.0, Vec2::new(200.0, 240.0), 0.0),
        Bone::new("b", None, 10.0, Vec2::new(440.0, 240.0), 0.0),
        Bone::new("c", None, 10.0, Vec2::new(320.0, 120.0), 0.0),
    ];
    let b = |bone: &str, r: f32| Blob { pos: Vec2::ZERO, radius: r, bone: Some(bone.into()) };
    p.layers = vec![
        Layer::boolean(
            "window",
            BoolOp::Difference,
            vec![
                VPath::rect("wall", 20.0, 20.0, 600.0, 440.0).with_paint(Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(0.0, 480.0), &[(0.0, "#2b3a67"), (1.0, "#1b2340")])),
                VPath::circle("hole", Vec2::new(320.0, 240.0), 190.0),
            ],
        ),
        {
            let mut l = Layer::liquid(
                "goo",
                vec![b("a", 34.0), b("b", 30.0), b("c", 26.0), Blob { pos: Vec2::new(320.0, 300.0), radius: 40.0, bone: None }],
                Paint::radial(Vec2::new(320.0, 240.0), 200.0, &[(0.0, "#7ef0d0"), (1.0, "#1fa38a")]),
            );
            l.blend = Blend::Normal;
            l
        },
        Layer::emitter(
            "sparks",
            EmitterSpec {
                pos: Vec2::new(320.0, 300.0),
                rate: 60.0,
                lifetime: 1.4,
                lifetime_jitter: 0.3,
                speed: 190.0,
                speed_jitter: 0.4,
                direction: -90.0,
                spread: 50.0,
                gravity: Vec2::new(0.0, 260.0),
                size_start: 4.0,
                size_end: 1.0,
                color_start: Rgba::hex("#fff2a8"),
                color_end: Rgba::hex("#ff5a36").with_alpha(0.0),
                seed: 11,
                ..Default::default()
            },
        ),
    ];
    let loopk = |lo: f32, hi: f32| Track::with_keys(&[(0.0, lo, Ease::Smooth), (36.0, hi, Ease::Smooth), (72.0, lo, Ease::Smooth)]);
    p.channels = vec![
        Channel::keys("a", Prop::BoneOffsetX, loopk(0.0, 110.0)),
        Channel::keys("b", Prop::BoneOffsetX, loopk(0.0, -110.0)),
        Channel::keys("c", Prop::BoneOffsetY, loopk(0.0, 120.0)),
    ];
    p
}

// ---------------------------------------------------------------------------------------------------------
// Full-figure demo: a walking person seen from the side (head with ear and ponytail, neck, trunk, two arms
// with hands and five fingers each, two legs with feet and toes). It is built from bones given in world
// coordinates; every body part is a skinned or bone-attached vector layer and the walk cycle is computed
// (two-bone leg solver, heel strike / toe-off, pelvis bob, counter-swinging arms, overlapping motion of
// forearm, hand, fingers, head, ear and ponytail), then written as ordinary keys.
// ---------------------------------------------------------------------------------------------------------

struct Skel {
    bones: Vec<Bone>,
    /// world head and world angle (degrees) of every bone at rest
    world: Vec<(Vec2, f32)>,
}

impl Skel {
    fn new() -> Skel {
        Skel { bones: Vec::new(), world: Vec::new() }
    }
    fn ix(&self, name: &str) -> usize {
        self.bones.iter().position(|b| b.name == name).unwrap_or_else(|| panic!("no bone {name}"))
    }
    /// Add a bone from its world head position and world angle (degrees, y points down).
    fn add(&mut self, name: &str, parent: Option<&str>, length: f32, head: (f32, f32), angle: f32) {
        let head = Vec2::new(head.0, head.1);
        let (off, rel) = match parent {
            None => (head, angle),
            Some(pn) => {
                let (ph, pa) = self.world[self.ix(pn)];
                let d = head - ph;
                let r = (-pa).to_radians();
                (Vec2::new(d.x * r.cos() - d.y * r.sin(), d.x * r.sin() + d.y * r.cos()), angle - pa)
            }
        };
        self.bones.push(Bone::new(name, parent, length, off, rel));
        self.world.push((head, angle));
    }
    fn head(&self, name: &str) -> Vec2 {
        self.world[self.ix(name)].0
    }
    fn tip(&self, name: &str) -> Vec2 {
        let i = self.ix(name);
        self.world[i].0 + Vec2::from_angle(self.world[i].1.to_radians()) * self.bones[i].length
    }
    /// Skin a path along a chain of bones: the weight moves from one bone to the next with a smooth blend of
    /// half-width `blend[k]` around the joint k (so knees, elbows and knuckles bend without creases).
    fn chain_skin(&self, path: &mut VPath, names: &[&str], blend: &[f32]) {
        let ids: Vec<usize> = names.iter().map(|n| self.ix(n)).collect();
        let segs: Vec<(Vec2, Vec2)> = ids.iter().map(|&i| (self.head(&self.bones[i].name), self.tip(&self.bones[i].name))).collect();
        let mut cum = vec![0.0f32];
        for &(a, b) in &segs {
            cum.push(cum.last().unwrap() + (b - a).length());
        }
        for an in &mut path.anchors {
            // arc position of the anchor along the chain
            let (mut best, mut s) = (f32::MAX, 0.0);
            for (k, &(a, b)) in segs.iter().enumerate() {
                let ab = b - a;
                let l2 = (ab.x * ab.x + ab.y * ab.y).max(1e-6);
                let t = (((an.pos.x - a.x) * ab.x + (an.pos.y - a.y) * ab.y) / l2).clamp(0.0, 1.0);
                let q = a + ab * t;
                let d = (an.pos - q).length();
                if d < best {
                    best = d;
                    s = cum[k] + t * l2.sqrt();
                }
            }
            let mut idx = 0.0f32;
            for k in 0..ids.len() - 1 {
                let w = blend.get(k).copied().unwrap_or(8.0).max(0.5);
                let x = ((s - (cum[k + 1] - w)) / (2.0 * w)).clamp(0.0, 1.0);
                idx += x * x * (3.0 - 2.0 * x);
            }
            let lo = (idx.floor() as usize).min(ids.len() - 1);
            let fr = idx - lo as f32;
            an.weights = if fr < 1e-3 || lo + 1 >= ids.len() {
                vec![BoneWeight { bone: names[lo].into(), weight: 1.0 }]
            } else {
                vec![BoneWeight { bone: names[lo].into(), weight: 1.0 - fr }, BoneWeight { bone: names[lo + 1].into(), weight: fr }]
            };
        }
    }
    /// A tapered closed outline around a chain of bones (rest pose).
    fn limb(&self, name: &str, names: &[&str], r0: f32, r1: f32) -> VPath {
        let ids: Vec<usize> = names.iter().map(|n| self.ix(n)).collect();
        let rest: Vec<Affine2> = self.world.iter().map(|&(h, a)| Affine2::translate(h).mul(Affine2::rotate(a.to_radians()))).collect();
        limb_outline(name, &rest, &self.bones, &ids, r0, r1)
    }
}

/// Ankle position (relative to the hip, y absolute) and foot pitch of one leg at walk phase `u` in [0,1).
/// Stance 0..BETA (heel strike, flat, heel-off about the ball of the foot), then swing with toe clearance.
fn gait(u: f32, hx: f32, step: f32) -> (f32, f32, f32, f32) {
    const BETA: f32 = 0.62;
    const GROUND: f32 = 430.0;
    let sm = |x: f32| {
        let t = x.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let (sin, cos) = (|d: f32| d.to_radians().sin(), |d: f32| d.to_radians().cos());
    // ankle height 10 above the sole, ball of the foot 28 ahead of the ankle, heel 14 behind it
    let strike = |flat: f32, psi: f32| (flat - 14.0 + 14.0 * cos(psi) + 10.0 * sin(psi), GROUND + 14.0 * sin(psi) - 10.0 * cos(psi));
    let toeoff = |flat: f32, psi: f32| (flat + 28.0 * (1.0 - cos(psi)) + 10.0 * sin(psi), GROUND - 28.0 * sin(psi) - 10.0 * cos(psi));
    let (psi_strike, psi_off) = (-12.0f32, 35.0f32);
    if u < BETA {
        let flat = hx + step / 2.0 - step * u / BETA;
        let (x, y, psi) = if u < 0.10 {
            let psi = psi_strike * (1.0 - sm(u / 0.10));
            let (x, y) = strike(flat, psi);
            (x, y, psi)
        } else if u < 0.42 {
            (flat, GROUND - 10.0, 0.0)
        } else {
            let psi = psi_off * sm((u - 0.42) / (BETA - 0.42));
            let (x, y) = toeoff(flat, psi);
            (x, y, psi)
        };
        (x, y, psi, 0.0)
    } else {
        let s = (u - BETA) / (1.0 - BETA);
        let (x0, y0) = toeoff(hx - step / 2.0 + 0.0, psi_off);
        let (x1, y1) = strike(hx + step / 2.0, psi_strike);
        // Hermite in x with the ground's speed at both ends (the foot is carried back while it lifts / lands)
        let m = -step * (1.0 - BETA) / BETA;
        let (s2, s3) = (s * s, s * s * s);
        let x = (2.0 * s3 - 3.0 * s2 + 1.0) * x0 + (s3 - 2.0 * s2 + s) * m + (-2.0 * s3 + 3.0 * s2) * x1 + (s3 - s2) * m;
        let y = y0 + (y1 - y0) * sm(s) - 15.0 * (std::f32::consts::PI * s).sin();
        let psi = if s < 0.4 { psi_off + (-5.0 - psi_off) * sm(s / 0.4) } else { -5.0 + (psi_strike + 5.0) * sm((s - 0.4) / 0.6) };
        // toes droop a little at lift-off
        (x, y, psi, 0.35 * psi * sm(s / 0.15))
    }
}

/// Two-bone solver, knee forward (+x). Returns the world angles (degrees) of thigh and shin.
fn leg_ik(hip: Vec2, ankle: Vec2, l1: f32, l2: f32) -> (f32, f32) {
    let d = ankle - hip;
    let dist = d.length().clamp(20.0, (l1 + l2) * 0.9985);
    let base = d.y.atan2(d.x).to_degrees();
    let g = ((l1 * l1 + dist * dist - l2 * l2) / (2.0 * l1 * dist)).clamp(-1.0, 1.0).acos().to_degrees();
    let a = base - g; // knee toward +x
    let k = hip + Vec2::from_angle(a.to_radians()) * l1;
    let tgt = hip + d.normalized() * dist;
    let b = (tgt.y - k.y).atan2(tgt.x - k.x).to_degrees();
    (a, b)
}

pub fn person() -> Project {
    use std::f32::consts::PI;
    const N: u32 = 32; // frames per walk cycle (two steps)
    const STEP: f32 = 90.0; // foot travel in the stance phase, px
    const BETA: f32 = 0.62;
    let ground_run = STEP / BETA; // px the ground moves per cycle so that planted feet do not slide

    let skin = "#f1c7a3";
    let skin_far = "#d6a47f";
    let shirt = "#4f86c6";
    let shirt_far = "#3d6aa0";
    let pants = "#3c4560";
    let pants_far = "#2e364d";
    let ink = "#3d2c2e";
    let hair = "#5a3826";

    let mut k = Skel::new();
    // trunk and head (all pointing up = -90 degrees)
    k.add("hips", None, 14.0, (320.0, 255.0), -90.0);
    k.add("spine", Some("hips"), 60.0, (320.0, 255.0), -90.0);
    k.add("chest", Some("spine"), 52.0, (320.0, 195.0), -90.0);
    k.add("neck", Some("chest"), 16.0, (320.0, 143.0), -90.0);
    k.add("head", Some("neck"), 66.0, (320.0, 127.0), -90.0);
    k.add("ear", Some("head"), 9.0, (311.0, 99.0), 90.0);
    k.add("hair1", Some("head"), 28.0, (300.0, 76.0), 125.0);
    let h1 = k.tip("hair1");
    k.add("hair2", Some("hair1"), 26.0, (h1.x, h1.y), 128.0);
    // legs: far leg first (drawn behind)
    for (suffix, _) in [("_far", 0), ("", 0)] {
        let t = |n: &str| format!("{n}{suffix}");
        k.add(&t("thigh"), Some("hips"), 82.0, (320.0, 255.0), 90.0);
        k.add(&t("shin"), Some(&t("thigh")), 83.0, (320.0, 337.0), 90.0);
        let foot_len = (28.0f32 * 28.0 + 10.0 * 10.0).sqrt();
        let foot_ang = (10.0f32).atan2(28.0).to_degrees();
        k.add(&t("foot"), Some(&t("shin")), foot_len, (320.0, 420.0), foot_ang);
        k.add(&t("toes"), Some(&t("foot")), 14.0, (348.0, 430.0), 0.0);
    }
    // arms and hands
    for suffix in ["_far", ""] {
        let t = |n: &str| format!("{n}{suffix}");
        k.add(&t("arm_up"), Some("chest"), 62.0, (322.0, 154.0), 90.0);
        k.add(&t("arm_low"), Some(&t("arm_up")), 56.0, (322.0, 216.0), 90.0);
        k.add(&t("hand"), Some(&t("arm_low")), 17.0, (322.0, 272.0), 90.0);
        // (name, lateral offset at the palm end, fan angle, proximal length, distal length)
        for (nm, dx, ang, l1, l2) in [("index", 4.5, 87.0, 10.5, 8.0), ("middle", 1.5, 90.0, 11.5, 8.5), ("ring", -1.5, 93.0, 10.5, 8.0), ("pinky", -4.5, 97.0, 8.0, 6.5)] {
            k.add(&t(&format!("{nm}1")), Some(&t("hand")), l1, (322.0 + dx, 289.0), ang);
            let a = ang.to_radians();
            let tip = Vec2::new(322.0 + dx, 289.0) + Vec2::new(a.cos(), a.sin()) * l1;
            k.add(&t(&format!("{nm}2")), Some(&t(&format!("{nm}1"))), l2, (tip.x, tip.y), ang);
        }
        k.add(&t("thumb1"), Some(&t("hand")), 9.5, (326.0, 277.0), 66.0);
        let a = 66.0f32.to_radians();
        let tip = Vec2::new(326.0, 277.0) + Vec2::new(a.cos(), a.sin()) * 9.5;
        k.add(&t("thumb2"), Some(&t("thumb1")), 7.5, (tip.x, tip.y), 58.0);
    }

    let mut p = Project::new("OSATURA walking person", 640.0, 480.0, 24.0, N);
    p.background = Some(hex("#f6f1e7"));
    p.bones = k.bones.clone();

    let st = |path: VPath, fill: &str, w: f32| path.with_fill(fill).with_stroke(ink, w);
    let rigid = |name: &str, bone: &str, paths: Vec<VPath>| {
        let mut l = Layer::vector(name, paths);
        l.bone = Some(bone.into());
        l
    };
    let skinned = |name: &str, mut paths: Vec<VPath>, chain: &[&str], blend: &[f32]| {
        for pa in &mut paths {
            k.chain_skin(pa, chain, blend);
        }
        Layer::vector(name, paths)
    };
    let pt = |x: f32, y: f32| Vec2::new(x, y);

    // ---- scenery ----
    let sky_paint = Paint::linear(pt(0.0, 0.0), pt(0.0, 430.0), &[(0.0, "#9ec5e8"), (0.7, "#f7e3c4"), (1.0, "#f6f1e7")]);
    let sky = Layer::vector("sky", vec![VPath::rect("sky", -50.0, -20.0, 740.0, 470.0).with_paint(sky_paint)]);
    let mut sun = Layer::vector("sun", vec![VPath::circle("sun", pt(520.0, 90.0), 70.0).with_paint(Paint::radial(pt(520.0, 90.0), 70.0, &[(0.0, "#fffbe6"), (0.5, "#ffd98a"), (1.0, "#ffd98a00")]))]);
    sun.blend = Blend::Screen;
    let hills_pts: Vec<Vec2> = (0..=10).map(|i| pt(-40.0 + i as f32 * 75.0, 352.0 - 34.0 * ((i as f32) * 0.8).sin() - 18.0)).chain([pt(740.0, 470.0), pt(-40.0, 470.0)]).collect();
    let mut hills = Layer::vector("hills", vec![VPath::smooth_through("hills", &hills_pts, true, 0.6).with_fill("#c3d3a8").with_stroke("#a7ba8a", 2.0)]);
    hills.opacity = 0.95;
    let meadow = Layer::vector("meadow", vec![VPath::rect("meadow", -50.0, 430.0, 740.0, 60.0).with_fill("#b9cf98")]);
    // fence posts and ground ticks scroll with the ground so that the planted foot does not slide
    let post_gap = ground_run / 2.0; // two ticks per cycle
    let ticks: Vec<VPath> = (-1..=((700.0 / post_gap) as i32 + 1)).map(|i| {
        let x = i as f32 * post_gap;
        VPath::new(&format!("tick {i}"), false, vec![Anchor::corner(pt(x, 438.0)), Anchor::corner(pt(x + 16.0, 438.0))]).with_stroke("#8a7f72", 3.0)
    }).collect();
    let mut ground = Layer::vector("ground ticks", ticks);
    ground.transform.anchor = Vec2::ZERO;
    let fence_gap = ground_run * 0.6; // slower: further back
    let posts: Vec<VPath> = (-1..=((700.0 / fence_gap) as i32 + 1)).map(|i| {
        let x = i as f32 * fence_gap;
        VPath::rect(&format!("post {i}"), x, 392.0, 7.0, 38.0).with_fill("#a98a66").with_stroke("#7b6248", 1.5)
    }).chain([VPath::new("rail", false, vec![Anchor::corner(pt(-60.0, 402.0)), Anchor::corner(pt(760.0, 402.0))]).with_stroke("#7b6248", 3.0)]).collect();
    let fence = Layer::vector("fence", posts);
    let mut shadow = Layer::vector("shadow", vec![VPath::ellipse("shadow", pt(322.0, 431.0), 78.0, 8.0).with_paint(Paint::radial(pt(322.0, 431.0), 78.0, &[(0.0, "#3d2c2eaa"), (1.0, "#3d2c2e00")]))]);
    shadow.blend = Blend::Multiply;
    shadow.transform.anchor = pt(322.0, 431.0);
    shadow.transform.position = pt(322.0, 431.0);

    // ---- limbs ----
    let arm_layers = |sfx: &str, sk: &str, sh: &str| -> Vec<Layer> {
        let t = |n: &str| format!("{n}{sfx}");
        let mut v = Vec::new();
        let (up, low, hand) = (t("arm_up"), t("arm_low"), t("hand"));
        let arm = st(k.limb("arm", &[&up, &low], 9.5, 6.8), sk, 2.5);
        v.push(skinned(&format!("arm{sfx}"), vec![arm], &[&up, &low], &[11.0]));
        // sleeve: a short rigid cuff on the upper arm, with the shoulder cap
        let sleeve = VPath::smooth_through("sleeve", &[pt(311.0, 150.0), pt(322.0, 143.0), pt(334.0, 150.0), pt(333.0, 180.0), pt(311.0, 180.0)], true, 0.7);
        v.push(rigid(&format!("sleeve{sfx}"), &up, vec![st(sleeve, sh, 2.5)]));
        // palm: a rounded fill without an outline on its wrist side (so there is no seam at the wrist), plus an open
        // outline along the sides and the bottom; rigid on the hand bone
        let palm_pts = [pt(317.0, 266.0), pt(328.0, 266.0), pt(329.0, 281.0), pt(327.0, 290.0), pt(318.0, 290.0), pt(316.0, 281.0)];
        let palm_fill = VPath::smooth_through("palm", &palm_pts, true, 0.8).with_fill(sk);
        let palm_edge = VPath::smooth_through("palm edge", &[pt(316.5, 271.0), pt(316.0, 281.0), pt(318.0, 290.0), pt(327.0, 290.0), pt(329.0, 281.0), pt(328.5, 271.0)], false, 0.8).with_stroke(ink, 2.0);
        v.push(rigid(&format!("palm{sfx}"), &hand, vec![palm_fill, palm_edge]));
        // fingers: each is a small skinned outline with a knuckle blend
        for nm in ["pinky", "ring", "middle", "index"] {
            let (a, b) = (t(&format!("{nm}1")), t(&format!("{nm}2")));
            let f = st(k.limb(nm, &[&a, &b], 2.2, 1.7), sk, 1.1);
            v.push(skinned(&format!("{nm}{sfx}"), vec![f], &[&a, &b], &[2.5]));
        }
        let (a, b) = (t("thumb1"), t("thumb2"));
        let th = st(k.limb("thumb", &[&a, &b], 2.8, 2.0), sk, 1.2);
        v.push(skinned(&format!("thumb{sfx}"), vec![th], &[&a, &b], &[2.5]));
        v
    };
    let leg_layers = |sfx: &str, pa: &str| -> Vec<Layer> {
        let t = |n: &str| format!("{n}{sfx}");
        let (thigh, shin, foot, toes) = (t("thigh"), t("shin"), t("foot"), t("toes"));
        let leg = st(k.limb("leg", &[&thigh, &shin], 17.0, 8.6), pa, 2.5);
        let shoe_col = if sfx.is_empty() { "#f4f1ea" } else { "#cdc6b8" };
        let sole_col = if sfx.is_empty() { "#d6483a" } else { "#a8392f" };
        let shoe = VPath::smooth_through("shoe", &[pt(311.0, 410.0), pt(327.0, 408.0), pt(331.0, 418.0), pt(346.0, 420.0), pt(358.0, 424.0), pt(362.0, 430.0), pt(304.0, 430.0), pt(303.0, 420.0)], true, 0.55);
        let sole = VPath::smooth_through("sole", &[pt(304.0, 426.0), pt(362.0, 426.0), pt(362.0, 431.0), pt(303.0, 431.0)], true, 0.3);
        vec![
            skinned(&format!("leg{sfx}"), vec![leg], &[&thigh, &shin], &[14.0]),
            skinned(&format!("shoe{sfx}"), vec![st(shoe, shoe_col, 2.5)], &[&foot, &toes], &[4.0]),
            skinned(&format!("sole{sfx}"), vec![st(sole, sole_col, 1.5)], &[&foot, &toes], &[4.0]),
        ]
    };

    // ---- torso, neck, head ----
    let torso_pts = [pt(309.0, 142.0), pt(299.0, 152.0), pt(297.0, 190.0), pt(301.0, 232.0), pt(300.0, 262.0), pt(342.0, 262.0), pt(345.0, 232.0), pt(351.0, 190.0), pt(346.0, 153.0), pt(333.0, 141.0)];
    let torso = st(VPath::smooth_through("torso", &torso_pts, true, 0.75), shirt, 2.5);
    let pelvis_pts = [pt(303.0, 236.0), pt(341.0, 236.0), pt(343.0, 262.0), pt(336.0, 281.0), pt(305.0, 282.0), pt(300.0, 262.0)];
    let pelvis = st(VPath::smooth_through("pelvis", &pelvis_pts, true, 0.6), pants, 2.5);
    let neck = st(VPath::smooth_through("neck", &[pt(311.0, 108.0), pt(331.0, 108.0), pt(332.0, 144.0), pt(310.0, 146.0)], true, 0.5), skin, 2.5);
    let head_pts = [
        pt(308.0, 120.0), pt(291.0, 104.0), pt(289.0, 82.0), pt(299.0, 64.0), pt(321.0, 57.0), pt(343.0, 64.0), pt(353.0, 80.0), pt(355.0, 91.0),
        pt(363.0, 101.0), pt(354.0, 105.0), pt(353.0, 112.0), pt(354.0, 117.0), pt(347.0, 127.0), pt(329.0, 130.0), pt(314.0, 126.0),
    ];
    let head = st(VPath::smooth_through("head", &head_pts, true, 0.7), skin, 2.5);
    let ear = VPath::smooth_through("ear", &[pt(311.0, 89.0), pt(317.0, 93.0), pt(317.0, 103.0), pt(312.0, 110.0), pt(306.0, 104.0), pt(305.0, 94.0)], true, 0.8);
    let ear_in = VPath::smooth_through("ear inner", &[pt(311.0, 95.0), pt(313.0, 99.0), pt(311.0, 104.0), pt(309.0, 99.0)], true, 0.8).with_stroke("#b9805f", 1.6);
    let hair_cap = VPath::smooth_through("hair", &[pt(290.0, 96.0), pt(288.0, 76.0), pt(299.0, 60.0), pt(321.0, 53.0), pt(344.0, 60.0), pt(353.0, 74.0), pt(338.0, 68.0), pt(322.0, 70.0), pt(311.0, 78.0), pt(305.0, 92.0), pt(299.0, 102.0)], true, 0.7);
    let pony = VPath::smooth_through("ponytail", &[pt(298.0, 70.0), pt(288.0, 80.0), pt(285.0, 104.0), pt(284.0, 128.0), pt(291.0, 138.0), pt(296.0, 122.0), pt(298.0, 96.0), pt(303.0, 82.0)], true, 0.8);
    let brow = VPath::smooth_through("brow", &[pt(337.0, 77.0), pt(346.0, 75.0), pt(352.0, 79.0)], false, 0.8).with_stroke(ink, 2.6);
    let eye_open = vec![VPath::ellipse("eye", pt(344.0, 87.0), 3.6, 4.6).with_fill("#ffffff").with_stroke(ink, 1.5), VPath::circle("pupil", pt(345.5, 87.5), 2.4).with_fill(ink)];
    let eye_shut = vec![VPath::smooth_through("lid", &[pt(339.0, 88.0), pt(344.0, 91.0), pt(349.0, 88.0)], false, 0.8).with_stroke(ink, 2.4)];
    let mouth_shapes = vec![
        Layer::vector("smile", vec![VPath::smooth_through("m", &[pt(340.0, 113.0), pt(346.0, 117.0), pt(353.0, 113.0)], false, 0.8).with_stroke(ink, 2.4)]),
        Layer::vector("open", vec![VPath::smooth_through("m", &[pt(340.0, 113.0), pt(347.0, 120.0), pt(353.0, 113.0)], true, 0.8).with_fill("#8b3a3a").with_stroke(ink, 2.0)]),
    ];
    let blush = VPath::circle("blush", pt(338.0, 105.0), 8.0).with_paint(Paint::radial(pt(338.0, 105.0), 8.0, &[(0.0, "#ff6b6b99"), (1.0, "#ff6b6b00")]));
    let nostril = VPath::circle("nostril", pt(356.0, 100.0), 1.0).with_fill(ink);

    let mut layers = vec![sky, sun, hills, meadow, fence, ground, shadow];
    layers.extend(arm_layers("_far", skin_far, shirt_far));
    layers.extend(leg_layers("_far", pants_far));
    layers.push(rigid("pelvis", "hips", vec![pelvis]));
    layers.push(skinned("torso", vec![torso], &["hips", "spine", "chest"], &[16.0, 16.0]));
    layers.extend(leg_layers("", pants));
    layers.push(rigid("neck", "neck", vec![neck]));
    layers.push(skinned("ponytail", vec![st(pony, hair, 2.0)], &["head", "hair1", "hair2"], &[14.0, 12.0]));
    layers.push(rigid("head", "head", vec![head]));
    layers.push(rigid("blush", "head", vec![blush]));
    layers.push(rigid("ear", "ear", vec![st(ear, skin, 2.2), ear_in]));
    layers.push(rigid("hair", "head", vec![st(hair_cap, hair, 2.2)]));
    layers.push(rigid("brow", "head", vec![brow]));
    let mut eye = Layer::switch("eye", vec![Layer::vector("open", eye_open), Layer::vector("closed", eye_shut)]);
    eye.bone = Some("head".into());
    layers.push(eye);
    layers.push(rigid("nostril", "head", vec![nostril]));
    let mut mouth = Layer::switch("mouth", mouth_shapes);
    mouth.bone = Some("head".into());
    layers.push(mouth);
    layers.extend(arm_layers("", skin, shirt));
    p.layers = layers;

    // ---- the walk cycle ----
    let ease = Ease::Smooth;
    let mut channels: Vec<Channel> = Vec::new();
    // sampled every 2 frames; one wrap-around key before frame 0 and one after the last frame keep the loop seam as smooth as the rest
    let sample = |f: &dyn Fn(f32) -> f32| -> Track {
        let ks: Vec<(f32, f32, Ease)> = (-1..=(N as i32 / 2 + 1)).map(|i| {
            let fr = (i * 2) as f32;
            (fr, (f(fr.rem_euclid(N as f32)) * 100.0).round() / 100.0, ease)
        }).collect();
        Track::with_keys(&ks)
    };
    let mut add = |target: &str, prop: Prop, f: &dyn Fn(f32) -> f32| channels.push(Channel::keys(target, prop, sample(f)));
    let ph = |fr: f32| fr / N as f32; // phase 0..1
    let tau = 2.0 * PI;
    let hip_y = |fr: f32| 255.0 + 7.0 * (0.5 - 0.5 * (2.0 * tau * (ph(fr) - BETA / 2.0)).cos());
    // body: pelvis bob and tilt, trunk lean and counter-rotation, head kept level with a small lagging nod
    let hd = |fr: f32| 1.4 * (tau * (2.0 * ph(fr) - 0.05)).sin();
    let sd = |fr: f32| 3.0 + 1.6 * (tau * (2.0 * ph(fr) - 0.10)).sin();
    let cd = |fr: f32| 0.8 + 1.2 * (tau * (ph(fr) - 0.20)).sin();
    let nd = |fr: f32| -0.35 * (hd(fr) + sd(fr) + cd(fr)) + 0.6 * (tau * (2.0 * ph(fr) - 0.22)).sin();
    let hed = |fr: f32| -(hd(fr) + sd(fr) + cd(fr) + nd(fr)) + 2.2 * (tau * (2.0 * ph(fr) - 0.26)).sin() + 1.0;
    add("hips", Prop::BoneOffsetY, &|fr| hip_y(fr) - 255.0);
    add("hips", Prop::BoneAngle, &hd);
    add("spine", Prop::BoneAngle, &sd);
    add("chest", Prop::BoneAngle, &cd);
    add("neck", Prop::BoneAngle, &nd);
    add("head", Prop::BoneAngle, &hed);
    // ear and ponytail follow late and overshoot a little (overlapping action)
    add("ear", Prop::BoneAngle, &|fr| 5.0 * (tau * (2.0 * ph(fr) - 0.38)).sin());
    add("hair1", Prop::BoneAngle, &|fr| 4.0 + 8.0 * (tau * (2.0 * ph(fr) - 0.32)).sin() + 4.0 * (tau * (ph(fr) - 0.42)).sin());
    add("hair2", Prop::BoneAngle, &|fr| 3.0 + 13.0 * (tau * (2.0 * ph(fr) - 0.46)).sin());
    // legs
    for (sfx, off) in [("", 0.0f32), ("_far", 0.5)] {
        let leg = move |fr: f32| -> (f32, f32, f32, f32, f32, f32) {
            let u = (ph(fr) + off).rem_euclid(1.0);
            let (ax, ay, psi, toe_w) = gait(u, 320.0, STEP);
            let hip = Vec2::new(320.0, hip_y(fr));
            let (a, b) = leg_ik(hip, Vec2::new(ax, ay), 82.0, 83.0);
            (a, b, psi, toe_w, ax, ay)
        };
        let t = |n: &str| format!("{n}{sfx}");
        add(&t("thigh"), Prop::BoneAngle, &|fr| leg(fr).0 - 90.0 - hd(fr));
        add(&t("shin"), Prop::BoneAngle, &|fr| leg(fr).1 - leg(fr).0);
        add(&t("foot"), Prop::BoneAngle, &|fr| leg(fr).2 + 90.0 - leg(fr).1);
        add(&t("toes"), Prop::BoneAngle, &|fr| leg(fr).3 - leg(fr).2);
    }
    // arms: opposite to the legs; elbows bend more on the forward swing; hands and fingers trail
    for (sfx, off) in [("", 0.0f32), ("_far", 0.5)] {
        let t = |n: &str| format!("{n}{sfx}");
        let swing = move |fr: f32| 21.0 * (tau * (ph(fr) + off)).cos() - 3.0 * (tau * (2.0 * (ph(fr) + off))).cos();
        let fwd = move |fr: f32| 0.5 - 0.5 * (tau * (ph(fr) + off - 0.06)).cos(); // 1 when the arm is forward (lagged)
        add(&t("arm_up"), Prop::BoneAngle, &|fr| swing(fr) - (hd(fr) + sd(fr) + cd(fr)));
        add(&t("arm_low"), Prop::BoneAngle, &|fr| -(9.0 + 22.0 * fwd(fr)));
        add(&t("hand"), Prop::BoneAngle, &|fr| 7.0 * (tau * (ph(fr) + off - 0.30)).sin() - 3.0);
        // relaxed fingers: a slight curl that grows towards the little finger, tightening a little on the forward swing
        for (i, nm) in ["index", "middle", "ring", "pinky"].iter().enumerate() {
            let c = 12.0 + 5.0 * i as f32;
            add(&t(&format!("{nm}1")), Prop::BoneAngle, &|fr| -(c + 5.0 * fwd(fr)) + 3.0 * (tau * (ph(fr) + off - 0.38 - 0.03 * i as f32)).sin());
            add(&t(&format!("{nm}2")), Prop::BoneAngle, &|fr| -(c * 1.4 + 7.0 * fwd(fr)) + 4.0 * (tau * (ph(fr) + off - 0.46 - 0.03 * i as f32)).sin());
        }
        add(&t("thumb1"), Prop::BoneAngle, &|fr| -(6.0 + 4.0 * fwd(fr)));
        add(&t("thumb2"), Prop::BoneAngle, &|fr| -(8.0 + 5.0 * fwd(fr)) + 3.0 * (tau * (ph(fr) + off - 0.45)).sin());
    }
    // face: a blink and a small smile that opens briefly
    channels.push(Channel::keys("eye", Prop::LayerSwitch, Track::with_keys(&[(0.0, 0.0, Ease::Step), (21.0, 1.0, Ease::Step), (23.0, 0.0, Ease::Step), (32.0, 0.0, Ease::Step)])));
    channels.push(Channel::keys("mouth", Prop::LayerSwitch, Track::with_keys(&[(0.0, 0.0, Ease::Step), (9.0, 1.0, Ease::Step), (14.0, 0.0, Ease::Step), (32.0, 0.0, Ease::Step)])));
    // the ground, the fence and the shadow: scroll so that a planted foot stays put
    let lin = |a: f32, b: f32| Track::with_keys(&[(0.0, a, Ease::Linear), (N as f32, b, Ease::Linear)]);
    channels.push(Channel::keys("ground ticks", Prop::LayerX, lin(0.0, -ground_run)));
    channels.push(Channel::keys("fence", Prop::LayerX, lin(0.0, -fence_gap)));
    channels.push(Channel::expr("shadow", Prop::LayerScaleX, "1 - get(\"hips\", \"bone_offset_y\") / 120", Mode::Replace));
    p.channels = channels;
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Engine;

    #[test]
    fn demo_compiles_and_round_trips_through_json() {
        let p = project();
        let json = p.to_json_pretty();
        let back = Project::from_json(&json).unwrap();
        assert_eq!(p, back);
        assert!(Engine::new(back).is_ok());
    }

    #[test]
    fn demo_foot_tracks_the_control_bone_every_frame() {
        let e = Engine::new(project()).unwrap();
        for f in 0..e.project.frames {
            let fr = e.evaluate(f as f32);
            let shin_tip = fr.world[4].apply(Vec2::new(120.0, 0.0));
            let target = fr.world[5].apply(Vec2::ZERO);
            assert!(shin_tip.distance(target) < 0.1, "frame {f}: {shin_tip:?} vs {target:?}");
        }
    }

    #[test]
    fn person_demo_is_valid_loops_and_keeps_planted_feet_still() {
        let p = person();
        let back = Project::from_json(&p.to_json_pretty()).expect("round trip");
        assert_eq!(back.bones.len(), p.bones.len());
        for name in ["thigh", "shin", "foot", "toes", "arm_up", "arm_low", "hand", "index1", "pinky2", "thumb2", "ear", "head", "neck", "chest", "hair2"] {
            assert!(p.bones.iter().any(|b| b.name == name), "bone {name}");
        }
        let toes = p.bones.iter().position(|b| b.name == "toes").unwrap();
        let e = Engine::new(p.clone()).expect("compiles");
        // the cycle closes: frame 0 and a would-be frame 32 give the same pose
        let (a, b) = (e.evaluate(0.0), e.evaluate(p.frames as f32));
        for (x, y) in a.world.iter().zip(b.world.iter()) {
            assert!(x.apply(Vec2::new(30.0, 0.0)).distance(y.apply(Vec2::new(30.0, 0.0))) < 0.35, "seam");
        }
        // flat-foot stance (frames 4..=12): the ball of the foot stays on the ground and moves back exactly as fast
        // as the ground scrolls, so the foot does not slide
        let speed = 90.0 / 0.62 / 32.0;
        let mut prev: Option<Vec2> = None;
        for f in 4..=12 {
            let ball = e.evaluate(f as f32).world[toes].apply(Vec2::ZERO);
            assert!((ball.y - 430.0).abs() < 0.8, "frame {f}: ball at y {}", ball.y);
            if let Some(q) = prev {
                assert!((q.x - ball.x - speed).abs() < 0.6, "frame {f}: foot moved {} instead of {speed}", q.x - ball.x);
            }
            prev = Some(ball);
        }
    }

    #[test]
    fn demo_uses_the_new_features() {
        let e = Engine::new(project()).unwrap();
        let f0 = e.evaluate(0.0);
        let mouth = f0.nodes[e.node_index("mouth").unwrap()].children[0];
        assert_eq!(f0.nodes[mouth].name, "mouth/closed");
        let f14 = e.evaluate(14.0);
        let mouth = f14.nodes[e.node_index("mouth").unwrap()].children[0];
        assert_eq!(f14.nodes[mouth].name, "mouth/open");
        // camera pan moves near layers more than far ones
        let f24 = e.evaluate(24.0);
        let dx = |name: &str| {
            let i = e.node_index(name).unwrap();
            f24.nodes[i].world.tx - f0.nodes[i].world.tx
        };
        assert!(dx("ground").abs() > dx("hills").abs() && dx("hills").abs() > dx("sky").abs());
    }
}
