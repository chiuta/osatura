// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Baked chain dynamics (hair, tails, ropes, antennae): a deterministic Verlet simulation with
//! gravity, wind (steady + gusts), circle and capsule colliders (static or riding a bone),
//! body-driven drag from a bone's velocity, vortex/radial fields and fixed-length links. The first bone's head
//! follows the animated rig; the simulated directions are written back as plain keyed angle
//! channels. Standard position-based dynamics; no external data.

use crate::anim::{Ease, Track};
use crate::math::Vec2;
use crate::project::{Channel, Project, Prop, Source};
use crate::Engine;

#[derive(Clone, Debug)]
pub struct Circle {
    pub centre: Vec2,
    pub radius: f32,
    /// Bone the collider is attached to: `centre` is given in world space at frame 0 and then
    /// moves rigidly with that bone's world transform. `None` = static.
    pub follow: Option<String>,
}

/// A segment `a`-`b` thickened by `radius` (a pill). Chain points are pushed out of it and so
/// are the links between chain points (not only the points). `follow` works as for [`Circle`].
#[derive(Clone, Debug)]
pub struct Capsule {
    pub a: Vec2,
    pub b: Vec2,
    pub radius: f32,
    pub follow: Option<String>,
}

/// Body-driven wind: every frame the chain feels an extra acceleration of `-gain` times the
/// world velocity (px/s) of `bone`, i.e. drag/inertia. A cape lags behind a running body.
#[derive(Clone, Debug)]
pub struct Drive {
    pub bone: String,
    pub gain: f32,
}

/// Radial/vortex force field centred on a point (static `centre`, or the origin of `follow` bone
/// when set, in which case `centre` is ignored). `swirl` is a tangential acceleration (px/s², positive =
/// clockwise on screen since y points down), `radial` is outward acceleration (negative = suck in).
/// Both fade linearly to zero at distance `radius`.
#[derive(Clone, Debug)]
pub struct Vortex {
    pub centre: Vec2,
    pub follow: Option<String>,
    pub swirl: f32,
    pub radial: f32,
    pub radius: f32,
}

#[derive(Clone, Debug)]
pub struct PhysicsOptions {
    /// Pixels per second² (stage y points down, so +y falls).
    pub gravity: Vec2,
    /// Steady wind acceleration, px/s².
    pub wind: Vec2,
    /// Extra wind multiplier swing: wind × (1 + gust·sin(2π·gust_hz·t)).
    pub gust: f32,
    pub gust_hz: f32,
    /// Velocity kept per sub-step (0.9..1).
    pub damping: f32,
    pub substeps: u32,
    pub iterations: u32,
    pub colliders: Vec<Circle>,
    pub capsules: Vec<Capsule>,
    pub drives: Vec<Drive>,
    pub vortices: Vec<Vortex>,
    /// Opt-in chain self-collision (spec 10, `capsule2d`): every segment of a strand is a capsule of this radius and capsules that are
    /// not neighbours push each other apart. `None` (the default) leaves the simulation exactly as it was. The radius must be smaller
    /// than half the shortest bone of the strand (otherwise a straight chain would push itself).
    pub self_collision: Option<f32>,
}

impl Default for PhysicsOptions {
    fn default() -> Self {
        PhysicsOptions { gravity: Vec2::new(0.0, 900.0), wind: Vec2::ZERO, gust: 0.0, gust_hz: 0.5, damping: 0.985, substeps: 4, iterations: 8, colliders: vec![], capsules: vec![], drives: vec![], vortices: vec![], self_collision: None }
    }
}

fn wrap_deg(mut a: f32) -> f32 {
    while a > 180.0 {
        a -= 360.0;
    }
    while a < -180.0 {
        a += 360.0;
    }
    a
}

/// Simulate `chain` (each bone the child of the previous) over the project's frames and return
/// one replace-mode angle channel per bone. Append them to the project's channels.
/// With `merge=keep` the CLI refuses a run that bakes nothing (existing channels win) unless `--allow-empty` is given.
pub fn simulate_chain(p: &Project, chain: &[String], o: &PhysicsOptions) -> Result<Vec<Channel>, String> {
    simulate_strip(p, &[chain.to_vec()], 0.0, o)
}

/// Several parallel chains ("strands") that stay roughly side by side: hair locks, a cape, a
/// banner. Neighbouring strands are tied at every joint below the root by a distance link;
/// `lateral` (0..1) is how firmly (0 = independent strands). Returns channels strand by strand.
pub fn simulate_strip(p: &Project, strands: &[Vec<String>], lateral: f32, o: &PhysicsOptions) -> Result<Vec<Channel>, String> {
    if strands.is_empty() || strands.iter().any(|c| c.is_empty()) {
        return Err("chain needs at least one bone".into());
    }
    if strands.len() > 32 || strands.iter().map(Vec::len).sum::<usize>() > 512 {
        return Err("too many strands or bones".into());
    }
    if !(0.0..=1.0).contains(&lateral) || !lateral.is_finite() {
        return Err("lateral must be within 0..1".into());
    }
    if !(0.5..=1.0).contains(&o.damping) || o.substeps == 0 || o.iterations == 0 || o.substeps > 64 || o.iterations > 64 {
        return Err("damping must be within 0.5..1 and substeps/iterations within 1..64".into());
    }
    let bone_ix = |n: &str| p.bones.iter().position(|b| b.name == n).ok_or_else(|| format!("no bone '{n}'"));
    let fin2 = |v: Vec2| v.x.is_finite() && v.y.is_finite();
    if !fin2(o.gravity) || !fin2(o.wind) || !o.gust.is_finite() || !o.gust_hz.is_finite() {
        return Err("gravity, wind and gust must be finite numbers".into());
    }
    if o.colliders.len() + o.capsules.len() > 256 || o.drives.len() > 32 || o.vortices.len() > 32 {
        return Err("too many colliders, drives or vortices".into());
    }
    let mut circ_bone: Vec<Option<usize>> = Vec::new();
    for c in &o.colliders {
        if !fin2(c.centre) || !c.radius.is_finite() || c.radius < 0.0 {
            return Err("circle needs finite centre and a radius >= 0".into());
        }
        circ_bone.push(c.follow.as_deref().map(bone_ix).transpose()?);
    }
    let mut cap_bone: Vec<Option<usize>> = Vec::new();
    for c in &o.capsules {
        if !fin2(c.a) || !fin2(c.b) || !c.radius.is_finite() || c.radius < 0.0 {
            return Err("capsule needs finite end points and a radius >= 0".into());
        }
        cap_bone.push(c.follow.as_deref().map(bone_ix).transpose()?);
    }
    let mut drive_bone: Vec<(usize, f32)> = Vec::new();
    for d in &o.drives {
        if !d.gain.is_finite() {
            return Err("drive gain must be finite".into());
        }
        drive_bone.push((bone_ix(&d.bone)?, d.gain));
    }
    let mut vort_bone: Vec<Option<usize>> = Vec::new();
    for v in &o.vortices {
        if !fin2(v.centre) || !v.swirl.is_finite() || !v.radial.is_finite() || !v.radius.is_finite() || v.radius < 0.0 {
            return Err("vortex needs finite values and a radius >= 0".into());
        }
        vort_bone.push(v.follow.as_deref().map(bone_ix).transpose()?);
    }
    if let Some(r) = o.self_collision {
        if !r.is_finite() || r <= 0.0 {
            return Err("self-collision radius must be a finite number > 0".into());
        }
    }
    struct Strand {
        idx: Vec<usize>,
        lens: Vec<f32>,
        parent: Option<usize>,
    }
    let mut ss: Vec<Strand> = Vec::new();
    for chain in strands {
        let idx: Vec<usize> = chain.iter().map(|n| p.bones.iter().position(|b| &b.name == n).ok_or_else(|| format!("no bone '{n}'"))).collect::<Result<_, _>>()?;
        for w in 0..idx.len().saturating_sub(1) {
            if p.bones[idx[w + 1]].parent.as_deref() != Some(chain[w].as_str()) {
                return Err(format!("'{}' is not a child of '{}'", chain[w + 1], chain[w]));
            }
        }
        let parent = p.bones[idx[0]].parent.as_ref().and_then(|n| p.bones.iter().position(|b| &b.name == n));
        let lens: Vec<f32> = idx.iter().map(|&i| p.bones[i].length).collect();
        if lens.iter().any(|&l| l.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) {
            return Err("chain bones need positive length".into());
        }
        if let Some(r) = o.self_collision {
            if idx.len() > MAX_SELF_COLLISION_BONES {
                return Err(format!("self-collision is limited to {MAX_SELF_COLLISION_BONES} bones per strand, got {}", idx.len()));
            }
            let shortest = lens.iter().copied().fold(f32::INFINITY, f32::min);
            if 2.0 * r >= shortest {
                return Err(format!("self-collision radius {r} is too large: it must be smaller than half the shortest bone ({shortest}), or a straight chain would push itself"));
            }
        }
        ss.push(Strand { idx, lens, parent });
    }
    let e = Engine::new(p.clone()).map_err(|v| v.join("; "))?;
    let f0 = e.evaluate(0.0);
    let mut pts: Vec<Vec<Vec2>> = Vec::new();
    for s in &ss {
        let n = s.idx.len();
        let mut v: Vec<Vec2> = s.idx.iter().map(|&i| Vec2::new(f0.world[i].tx, f0.world[i].ty)).collect();
        let a = f0.world[s.idx[n - 1]].rotation();
        v.push(v[n - 1] + Vec2::new(a.cos(), a.sin()) * s.lens[n - 1]);
        pts.push(v);
    }
    // rest distances between neighbouring strands at each shared joint (root joints are pinned)
    let mut links: Vec<(usize, usize, usize, f32)> = Vec::new();
    if lateral > 0.0 {
        for c in 0..ss.len().saturating_sub(1) {
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for i in 1..pts[c].len().min(pts[c + 1].len()) {
                links.push((c, c + 1, i, pts[c][i].distance(pts[c + 1][i])));
            }
        }
    }
    let mut prev = pts.clone();
    let dt = 1.0 / p.fps;
    let h = dt / o.substeps as f32;
    let mut tracks: Vec<Vec<Vec<(f32, f32, Ease)>>> = ss.iter().map(|s| vec![Vec::new(); s.idx.len()]).collect();
    let mut last_deg: Vec<Vec<f32>> = ss.iter().map(|s| vec![f32::NAN; s.idx.len()]).collect();
    // collider geometry in world space at one frame: (a, b, radius, links_too). Capsules and
    // attached circles also keep the links between chain points out (a moving collider can pass
    // between two joints); static circles keep their original points-only behaviour.
    type Shape = (Vec2, Vec2, f32, bool);
    let shapes_at = |fr: &crate::engine::Frame| -> Vec<Shape> {
        let mv = |bone: Option<usize>| -> Option<crate::math::Affine2> {
            match bone {
                None => None,
                Some(i) => f0.world[i].inverse().map(|inv| fr.world[i].mul(inv)),
            }
        };
        let mut v: Vec<Shape> = Vec::new();
        for (c, b) in o.colliders.iter().zip(&circ_bone) {
            let m = mv(*b);
            let q = m.map_or(c.centre, |m| m.apply(c.centre));
            v.push((q, q, c.radius, b.is_some()));
        }
        for (c, b) in o.capsules.iter().zip(&cap_bone) {
            let m = mv(*b);
            v.push((m.map_or(c.a, |m| m.apply(c.a)), m.map_or(c.b, |m| m.apply(c.b)), c.radius, true));
        }
        v
    };
    let mut prev_shapes = shapes_at(&f0);
    let mut prev_drive: Vec<Vec2> = drive_bone.iter().map(|&(i, _)| Vec2::new(f0.world[i].tx, f0.world[i].ty)).collect();
    for f in 0..p.frames {
        let fr = e.evaluate(f as f32);
        let t = f as f32 * dt;
        let shapes = shapes_at(&fr);
        let mut body_acc = Vec2::ZERO;
        for (k, &(i, gain)) in drive_bone.iter().enumerate() {
            let pos = Vec2::new(fr.world[i].tx, fr.world[i].ty);
            if f > 0 {
                body_acc += (pos - prev_drive[k]) * (-gain / dt);
            }
            prev_drive[k] = pos;
        }
        let vorts: Vec<(Vec2, &Vortex)> = o.vortices.iter().zip(&vort_bone).map(|(v, b)| (b.map_or(v.centre, |i| Vec2::new(fr.world[i].tx, fr.world[i].ty)), v)).collect();
        let wind = o.wind * (1.0 + o.gust * (std::f32::consts::TAU * o.gust_hz * t).sin());
        let acc = o.gravity + wind + body_acc;
        let heads: Vec<Vec2> = ss.iter().map(|s| Vec2::new(fr.world[s.idx[0]].tx, fr.world[s.idx[0]].ty)).collect();
        for sub in 0..o.substeps {
            // colliders glide from last frame's pose to this frame's across the substeps
            let u = (sub + 1) as f32 / o.substeps as f32;
            let cur: Vec<Shape> = prev_shapes.iter().zip(&shapes).map(|(a, b)| (a.0.lerp(b.0, u), a.1.lerp(b.1, u), b.2, b.3)).collect();
            for (c, pc) in pts.iter_mut().enumerate() {
                for i in 1..pc.len() {
                    let v = (pc[i] - prev[c][i]) * o.damping;
                    prev[c][i] = pc[i];
                    let mut a = acc;
                    for (ctr, vx) in &vorts {
                        let d = pc[i] - *ctr;
                        let l = d.length();
                        if l > 1e-6 && l < vx.radius {
                            let fall = 1.0 - l / vx.radius;
                            let n = d * (1.0 / l);
                            a += (Vec2::new(-n.y, n.x) * vx.swirl + n * vx.radial) * fall;
                        }
                    }
                    pc[i] = pc[i] + v + a * (h * h);
                }
                pc[0] = heads[c];
                prev[c][0] = heads[c];
            }
            for _ in 0..o.iterations {
                for (c, s) in ss.iter().enumerate() {
                    for i in 0..s.idx.len() {
                        let (a, b) = (pts[c][i], pts[c][i + 1]);
                        let d = b - a;
                        let l = d.length().max(1e-6);
                        let corr = d * ((l - s.lens[i]) / l);
                        if i == 0 {
                            pts[c][i + 1] = b - corr;
                        } else {
                            pts[c][i] = a + corr * 0.5;
                            pts[c][i + 1] = b - corr * 0.5;
                        }
                    }
                }
                for &(c0, c1, i, rest) in &links {
                    let (a, b) = (pts[c0][i], pts[c1][i]);
                    let d = b - a;
                    let l = d.length().max(1e-6);
                    let corr = d * ((l - rest) / l * 0.5 * lateral);
                    pts[c0][i] = a + corr;
                    pts[c1][i] = b - corr;
                }
                for pc in pts.iter_mut() {
                    for q in pc.iter_mut().skip(1) {
                        for &(a, b, r, _) in &cur {
                            let cp = closest_on_segment(a, b, *q);
                            let d = *q - cp;
                            let l = d.length();
                            if l < r {
                                *q = cp + if l > 1e-6 { d * (r / l) } else { capsule_normal(a, b) * r };
                            }
                        }
                    }
                    if let Some(r) = o.self_collision {
                        push_apart(pc, r)?;
                    }
                    // links between chain points must not cut through a capsule either
                    for &(a, b, r, _) in cur.iter().filter(|s| s.3) {
                        for i in 0..pc.len() - 1 {
                            let (p0, p1) = (pc[i], pc[i + 1]);
                            let (s, _, c1, c2) = seg_seg(p0, p1, a, b);
                            let d = c1 - c2;
                            let l = d.length();
                            if l >= r || !(0.02..=0.98).contains(&s) {
                                continue; // end points were already handled above
                            }
                            let n = if l > 1e-6 { d * (1.0 / l) } else { capsule_normal(a, b) };
                            let (im0, im1) = (if i == 0 { 0.0 } else { 1.0 }, 1.0f32);
                            let (b0, b1) = (1.0 - s, s);
                            let den = im0 * b0 * b0 + im1 * b1 * b1;
                            if den < 1e-6 {
                                continue;
                            }
                            let pen = r - l;
                            pc[i] = p0 + n * (pen * im0 * b0 / den);
                            pc[i + 1] = p1 + n * (pen * im1 * b1 / den);
                        }
                    }
                }
            }
        }
        prev_shapes = shapes;
        for (c, s) in ss.iter().enumerate() {
            let mut parent_world = s.parent.map_or(0.0, |i| fr.world[i].rotation().to_degrees());
            for i in 0..s.idx.len() {
                let d = pts[c][i + 1] - pts[c][i];
                let world = d.y.atan2(d.x).to_degrees();
                let mut local = wrap_deg(world - parent_world - p.bones[s.idx[i]].rest_angle);
                let ld = last_deg[c][i];
                if ld.is_finite() {
                    while local - ld > 180.0 {
                        local -= 360.0;
                    }
                    while local - ld < -180.0 {
                        local += 360.0;
                    }
                }
                last_deg[c][i] = local;
                tracks[c][i].push((f as f32, local, Ease::Linear));
                parent_world = world;
            }
        }
    }
    let mut out = Vec::new();
    for (chain, tr) in strands.iter().zip(tracks) {
        for (b, t) in chain.iter().zip(tr) {
            out.push(Channel::keys(b, Prop::BoneAngle, Track::with_keys(&t)));
        }
    }
    Ok(out)
}

/// Most bones of one strand that the opt-in self-collision handles (the pair test is quadratic).
pub const MAX_SELF_COLLISION_BONES: usize = 128;

/// One pass of self-collision on a strand: capsule `i` runs from point `i` to point `i + 1`; the corrections of `capsule2d::chain_corrections`
/// (capsule 0 holds the pinned root, so it counts half as much as the others) are shared by the capsule's two points, a point between two
/// capsules takes the mean of both, and the root point never moves. The bone lengths are restored by the next link pass.
fn push_apart(pc: &mut [Vec2], r: f32) -> Result<(), String> {
    use crate::capsule2d::{chain_capsules, chain_corrections};
    let pts: Vec<[f64; 2]> = pc.iter().map(|q| [f64::from(q.x), f64::from(q.y)]).collect();
    let caps = chain_capsules(&pts, f64::from(r));
    let mut w = vec![1.0; caps.len()];
    if let Some(first) = w.first_mut() {
        *first = 0.5;
    }
    let corr = chain_corrections(&caps, &w)?;
    for i in 1..pc.len() {
        let (mut sx, mut sy, mut n) = (0.0f64, 0.0f64, 0.0f64);
        for k in [i - 1, i] {
            if let Some(c) = corr.get(k).filter(|_| k < caps.len()) {
                sx += c[0];
                sy += c[1];
                n += 1.0;
            }
        }
        if n > 0.0 {
            pc[i] = Vec2::new(pc[i].x + (sx / n) as f32, pc[i].y + (sy / n) as f32);
        }
    }
    Ok(())
}

pub(crate) fn closest_on_segment(a: Vec2, b: Vec2, q: Vec2) -> Vec2 {
    let ab = b - a;
    let l2 = ab.dot(ab);
    if l2 < 1e-12 {
        return a;
    }
    a + ab * ((q - a).dot(ab) / l2).clamp(0.0, 1.0)
}

/// Unit normal of a capsule axis (any fixed direction for a point-like one).
pub(crate) fn capsule_normal(a: Vec2, b: Vec2) -> Vec2 {
    let d = b - a;
    if d.length() < 1e-6 {
        Vec2::new(1.0, 0.0)
    } else {
        let n = d.normalized();
        Vec2::new(-n.y, n.x)
    }
}

/// Closest points of segments p1-q1 and p2-q2: (s on the first, t on the second, point1, point2).
fn seg_seg(p1: Vec2, q1: Vec2, p2: Vec2, q2: Vec2) -> (f32, f32, Vec2, Vec2) {
    let (d1, d2, r) = (q1 - p1, q2 - p2, p1 - p2);
    let (a, e, f) = (d1.dot(d1), d2.dot(d2), d2.dot(r));
    let (s, t);
    if a <= 1e-9 && e <= 1e-9 {
        (s, t) = (0.0, 0.0);
    } else if a <= 1e-9 {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = d1.dot(r);
        if e <= 1e-9 {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = d1.dot(d2);
            let den = a * e - b * b;
            let mut s0 = if den > 1e-9 { ((b * f - c * e) / den).clamp(0.0, 1.0) } else { 0.0 };
            let mut t0 = (b * s0 + f) / e;
            if t0 < 0.0 {
                t0 = 0.0;
                s0 = (-c / a).clamp(0.0, 1.0);
            } else if t0 > 1.0 {
                t0 = 1.0;
                s0 = ((b - c) / a).clamp(0.0, 1.0);
            }
            (s, t) = (s0, t0);
        }
    }
    (s, t, p1 + d1 * s, p2 + d2 * t)
}

/// Names of the built-in presets.
pub const PRESETS: &[&str] = &["hair", "tail", "rope", "antenna", "cape"];

/// Ready-made option sets: tuned feel, not tied to any product.
pub fn preset(name: &str) -> Option<PhysicsOptions> {
    let base = PhysicsOptions::default();
    Some(match name {
        "hair" => PhysicsOptions { gravity: Vec2::new(0.0, 500.0), damping: 0.96, ..base },
        "tail" => PhysicsOptions { gravity: Vec2::new(0.0, 700.0), damping: 0.97, gust: 0.0, ..base },
        "rope" => PhysicsOptions { gravity: Vec2::new(0.0, 1200.0), damping: 0.99, iterations: 16, ..base },
        "antenna" => PhysicsOptions { gravity: Vec2::new(0.0, 150.0), damping: 0.9, ..base },
        "cape" => PhysicsOptions { gravity: Vec2::new(0.0, 400.0), wind: Vec2::new(120.0, 0.0), gust: 0.6, gust_hz: 0.4, damping: 0.95, ..base },
        _ => return None,
    })
}

/// Add simulated channels to the project. With `overwrite = false`, bones that already have a
/// keyed angle channel are left alone (their hand-made animation wins); with `true`, those
/// channels are replaced. Returns `(added, skipped_bones)`.
pub fn merge_channels(p: &mut Project, new: Vec<Channel>, overwrite: bool) -> (usize, Vec<String>) {
    let mut added = 0;
    let mut skipped = Vec::new();
    let (new, blocked) = crate::chanmeta::split_unlocked(p, new);
    skipped.extend(blocked.into_iter().map(|b| b.split('.').next().unwrap_or("").to_string()));
    for ch in new {
        let has = |c: &Channel| c.target == ch.target && c.prop == ch.prop && matches!(c.source, Source::Keys { .. });
        if p.channels.iter().any(has) {
            if !overwrite {
                skipped.push(ch.target.clone());
                continue;
            }
            p.channels.retain(|c| !has(c));
        }
        p.channels.push(ch);
        added += 1;
    }
    (added, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Bone;

    fn rope() -> Project {
        let mut p = Project::new("r", 400.0, 400.0, 24.0, 160);
        p.bones = vec![
            Bone::new("a", None, 40.0, Vec2::new(200.0, 50.0), 0.0),
            Bone::new("b", Some("a"), 40.0, Vec2::new(40.0, 0.0), 0.0),
            Bone::new("c", Some("b"), 40.0, Vec2::new(40.0, 0.0), 0.0),
        ];
        p
    }
    fn tip(p: &Project, chain: &[Channel], f: f32) -> Vec2 {
        let mut q = p.clone();
        q.channels.extend(chain.iter().cloned());
        let w = Engine::new(q).unwrap().evaluate(f).world;
        let a = w[2].rotation();
        Vec2::new(w[2].tx + a.cos() * 40.0, w[2].ty + a.sin() * 40.0)
    }
    fn names() -> Vec<String> {
        vec!["a".into(), "b".into(), "c".into()]
    }

    #[test]
    fn hangs_down_under_gravity_and_is_deterministic() {
        let p = rope();
        let ch = simulate_chain(&p, &names(), &PhysicsOptions { damping: 0.95, ..Default::default() }).unwrap();
        let t = tip(&p, &ch, 159.0);
        assert!((t.x - 200.0).abs() < 3.0 && (t.y - 170.0).abs() < 3.0, "tip {t:?}: straight down 120 px below the root");
        let ch2 = simulate_chain(&p, &names(), &PhysicsOptions { damping: 0.95, ..Default::default() }).unwrap();
        let (Source::Keys { track: a }, Source::Keys { track: b }) = (&ch[1].source, &ch2[1].source) else { panic!() };
        assert_eq!(a.eval(77.0), b.eval(77.0));
    }

    #[test]
    fn wind_pushes_sideways_and_collider_blocks() {
        let p = rope();
        let calm = simulate_chain(&p, &names(), &PhysicsOptions { damping: 0.95, ..Default::default() }).unwrap();
        let windy = simulate_chain(&p, &names(), &PhysicsOptions { damping: 0.95, wind: Vec2::new(600.0, 0.0), ..Default::default() }).unwrap();
        assert!(tip(&p, &windy, 159.0).x > tip(&p, &calm, 159.0).x + 20.0);
        let wall = PhysicsOptions { damping: 0.95, colliders: vec![Circle { centre: Vec2::new(200.0, 160.0), radius: 30.0, follow: None }], ..Default::default() };
        let blocked = simulate_chain(&p, &names(), &wall).unwrap();
        let t = tip(&p, &blocked, 159.0);
        assert!(t.distance(Vec2::new(200.0, 160.0)) >= 29.0, "tip stays outside the circle: {t:?}");
    }

    #[test]
    fn presets_exist_and_run() {
        let p = rope();
        for n in PRESETS {
            let o = preset(n).unwrap_or_else(|| panic!("{n}"));
            assert_eq!(simulate_chain(&p, &names(), &o).unwrap().len(), 3);
        }
        assert!(preset("nope").is_none());
    }

    #[test]
    fn merge_keeps_hand_animation_unless_told() {
        let mut p = rope();
        p.channels.push(Channel::keys("b", Prop::BoneAngle, Track::with_keys(&[(0.0, 5.0, Ease::Linear)])));
        let sim = simulate_chain(&p, &names(), &PhysicsOptions::default()).unwrap();
        let (added, skipped) = merge_channels(&mut p, sim.clone(), false);
        assert_eq!((added, skipped), (2, vec!["b".to_string()]));
        let (added, skipped) = merge_channels(&mut p, sim, true);
        assert_eq!(added, 3);
        assert!(skipped.is_empty());
        assert_eq!(p.channels.iter().filter(|c| c.target == "b").count(), 1, "replaced, not duplicated");
    }

    fn cape() -> Project {
        let mut p = Project::new("c", 400.0, 400.0, 24.0, 120);
        for (s, x) in [("l", 180.0f32), ("r", 220.0)] {
            p.bones.push(Bone::new(&format!("{s}1"), None, 40.0, Vec2::new(x, 50.0), 90.0));
            p.bones.push(Bone::new(&format!("{s}2"), Some(&format!("{s}1")), 40.0, Vec2::new(40.0, 0.0), 0.0));
        }
        p
    }
    fn strands() -> Vec<Vec<String>> {
        vec![vec!["l1".into(), "l2".into()], vec!["r1".into(), "r2".into()]]
    }
    fn tip_x(p: &Project, ch: &[Channel], bone: usize) -> f32 {
        let mut q = p.clone();
        q.channels.extend(ch.iter().cloned());
        let w = Engine::new(q).unwrap().evaluate(119.0).world;
        let a = w[bone].rotation();
        w[bone].tx + a.cos() * 40.0
    }

    #[test]
    fn strip_links_strands_and_keeps_them_apart() {
        let p = cape();
        let o = PhysicsOptions { damping: 0.95, wind: Vec2::new(500.0, 0.0), ..Default::default() };
        let loose = simulate_strip(&p, &strands(), 0.0, &o).unwrap();
        let tied = simulate_strip(&p, &strands(), 1.0, &o).unwrap();
        assert_eq!(tied.len(), 4);
        // bones are l1 l2 r1 r2 -> indices 0..3; compare the tip gap of the two strands
        let gap = |ch: &[Channel]| (tip_x(&p, ch, 3) - tip_x(&p, ch, 1)).abs();
        assert!((gap(&tied) - 40.0).abs() < 4.0, "tied strands keep their 40 px spacing, gap {}", gap(&tied));
        assert!(gap(&tied) <= gap(&loose) + 0.5);
        let again = simulate_strip(&p, &strands(), 1.0, &o).unwrap();
        assert_eq!(format!("{:?}", tied[0].source), format!("{:?}", again[0].source), "deterministic");
        assert!(simulate_strip(&p, &strands(), 1.5, &o).is_err());
        assert!(simulate_strip(&p, &[], 0.5, &o).is_err());
    }

    #[test]
    fn rejects_bad_chains_and_options() {
        let p = rope();
        assert!(simulate_chain(&p, &[], &PhysicsOptions::default()).is_err());
        assert!(simulate_chain(&p, &["a".into(), "c".into()], &PhysicsOptions::default()).is_err(), "not consecutive");
        assert!(simulate_chain(&p, &["zz".into()], &PhysicsOptions::default()).is_err());
        assert!(simulate_chain(&p, &names(), &PhysicsOptions { damping: 0.1, ..Default::default() }).is_err());
    }

    // ---- capsules, attached colliders, body-driven fields ----
    fn hang() -> Project {
        let mut p = Project::new("h", 400.0, 400.0, 24.0, 120);
        p.bones = vec![
            Bone::new("body", None, 30.0, Vec2::new(100.0, 50.0), 0.0),
            Bone::new("pusher", None, 20.0, Vec2::new(120.0, 100.0), 0.0),
            Bone::new("a", Some("body"), 40.0, Vec2::new(0.0, 0.0), 90.0),
            Bone::new("b", Some("a"), 40.0, Vec2::new(40.0, 0.0), 0.0),
            Bone::new("c", Some("b"), 40.0, Vec2::new(40.0, 0.0), 0.0),
        ];
        p
    }
    fn hang_names() -> Vec<String> {
        vec!["a".into(), "b".into(), "c".into()]
    }
    fn slide(p: &mut Project, bone: &str, dx: f32) {
        p.channels.push(Channel::keys(bone, Prop::BoneOffsetX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (119.0, dx, Ease::Linear)])));
    }
    /// world joints of the chain a,b,c plus the tip at frame `f`
    fn joints(p: &Project, ch: &[Channel], f: f32) -> Vec<Vec2> {
        let mut q = p.clone();
        q.channels.extend(ch.iter().cloned());
        let w = Engine::new(q).unwrap().evaluate(f).world;
        let mut v: Vec<Vec2> = (2..5).map(|i| Vec2::new(w[i].tx, w[i].ty)).collect();
        let a = w[4].rotation();
        v.push(v[2] + Vec2::new(a.cos(), a.sin()) * 40.0);
        v
    }
    fn seg_dist(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> f32 {
        let (_, _, p, q) = seg_seg(a, b, c, d);
        p.distance(q)
    }
    fn opts() -> PhysicsOptions {
        PhysicsOptions { damping: 0.95, ..Default::default() }
    }

    #[test]
    fn seg_seg_matches_hand_cases() {
        let (s, t, p, q) = seg_seg(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(5.0, -3.0), Vec2::new(5.0, 3.0));
        assert!((s - 0.5).abs() < 1e-5 && (t - 0.5).abs() < 1e-5 && p.distance(q) < 1e-4);
        let (_, _, p, q) = seg_seg(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(0.0, 4.0), Vec2::new(10.0, 4.0));
        assert!((p.distance(q) - 4.0).abs() < 1e-4, "parallel");
        let (_, _, p, q) = seg_seg(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        assert!((p.distance(q) - Vec2::new(1.0, 0.0).distance(Vec2::new(5.0, 5.0))).abs() < 1e-4, "degenerate");
    }

    #[test]
    fn chain_stops_on_a_capsule_and_links_do_not_cut_through() {
        // the chain starts horizontal at y=50 and falls onto the capsule
        let mut p = hang();
        p.bones[2].rest_angle = 0.0;
        let free = simulate_chain(&p, &hang_names(), &opts()).unwrap();
        let free_tip = joints(&p, &free, 119.0)[3];
        assert!(free_tip.y > 160.0, "control hangs to y=170");
        for (a, b, r) in [(Vec2::new(60.0, 110.0), Vec2::new(140.0, 110.0), 10.0), (Vec2::new(90.0, 110.0), Vec2::new(110.0, 110.0), 3.0)] {
            let o = PhysicsOptions { capsules: vec![Capsule { a, b, radius: r, follow: None }], ..opts() };
            let ch = simulate_chain(&p, &hang_names(), &o).unwrap();
            for f in [30.0, 60.0, 119.0] {
                let j = joints(&p, &ch, f);
                for q in &j[1..] {
                    assert!(seg_dist(*q, *q, a, b) >= r - 1.0, "frame {f}: joint {q:?} inside the capsule");
                }
                for w in j.windows(2) {
                    assert!(seg_dist(w[0], w[1], a, b) >= r - 1.5, "frame {f}: link {w:?} cuts through the capsule");
                }
            }
            let tip = joints(&p, &ch, 119.0)[3];
            assert!(tip.distance(free_tip) > 5.0, "the capsule changed the result");
        }
    }

    #[test]
    fn collider_on_a_moving_bone_pushes_the_chain() {
        let mut p = hang();
        slide(&mut p, "pusher", 0.0);
        let still = simulate_chain(&p, &hang_names(), &opts()).unwrap();
        // the pusher bone slides left->right through the hanging chain (x=100 at the root)
        p.channels.clear();
        p.bones[1].parent = None;
        let mut moving = p.clone();
        slide(&mut moving, "pusher", -60.0);
        let cap = |follow: Option<&str>| PhysicsOptions {
            capsules: vec![Capsule { a: Vec2::new(150.0, 70.0), b: Vec2::new(150.0, 150.0), radius: 8.0, follow: follow.map(String::from) }],
            ..opts()
        };
        // static capsule at x=150: chain unaffected; attached to the pusher it travels 60 px to the left through x=100
        let st = simulate_chain(&moving, &hang_names(), &cap(None)).unwrap();
        let fl = simulate_chain(&moving, &hang_names(), &cap(Some("pusher"))).unwrap();
        let tip_st = joints(&moving, &st, 119.0)[3];
        let tip_fl = joints(&moving, &fl, 119.0)[3];
        assert!((tip_st.x - 100.0).abs() < 3.0, "static capsule is out of the way: {tip_st:?}");
        assert!(tip_fl.x < 90.0, "the moving capsule pushed the tip left: {tip_fl:?}");
        let _ = still;
        // the same with a circle
        let circ = PhysicsOptions { colliders: vec![Circle { centre: Vec2::new(150.0, 110.0), radius: 12.0, follow: Some("pusher".into()) }], ..opts() };
        let fc = simulate_chain(&moving, &hang_names(), &circ).unwrap();
        let jc = joints(&moving, &fc, 119.0);
        assert!(jc[1..].iter().any(|q| q.x < 92.0), "attached circle pushes too: {jc:?}");
        // determinism
        let again = simulate_chain(&moving, &hang_names(), &cap(Some("pusher"))).unwrap();
        assert_eq!(format!("{:?}", fl), format!("{:?}", again));
    }

    #[test]
    fn body_velocity_drives_a_trailing_chain_the_opposite_way() {
        let mut p = hang();
        slide(&mut p, "body", 500.0);
        let none = simulate_chain(&p, &hang_names(), &opts()).unwrap();
        let o = PhysicsOptions { drives: vec![Drive { bone: "body".into(), gain: 5.0 }], ..opts() };
        let dr = simulate_chain(&p, &hang_names(), &o).unwrap();
        let rel = |ch: &[Channel]| {
            let j = joints(&p, ch, 100.0);
            j[3].x - j[0].x
        };
        assert!(rel(&dr) < rel(&none) - 15.0, "body runs right, chain trails left: {}", rel(&dr));
        let neg = PhysicsOptions { drives: vec![Drive { bone: "body".into(), gain: -5.0 }], ..opts() };
        let lead = simulate_chain(&p, &hang_names(), &neg).unwrap();
        assert!(rel(&lead) > rel(&none) + 15.0, "negative gain pushes with the motion");
        assert_eq!(format!("{:?}", dr), format!("{:?}", simulate_chain(&p, &hang_names(), &o).unwrap()));
    }

    #[test]
    fn vortex_swirls_and_radial_field_pushes() {
        let p = hang();
        let base = joints(&p, &simulate_chain(&p, &hang_names(), &opts()).unwrap(), 119.0)[3];
        let v = |swirl: f32, radial: f32, follow: Option<&str>| PhysicsOptions {
            vortices: vec![Vortex { centre: Vec2::new(140.0, 150.0), follow: follow.map(String::from), swirl, radial, radius: 120.0 }],
            ..opts()
        };
        let sw = joints(&p, &simulate_chain(&p, &hang_names(), &v(800.0, 0.0, None)).unwrap(), 119.0)[3];
        assert!(sw.distance(base) > 10.0, "swirl moves the tip: {sw:?} vs {base:?}");
        let ra = joints(&p, &simulate_chain(&p, &hang_names(), &v(0.0, -800.0, None)).unwrap(), 119.0)[3];
        assert!(ra.distance(base) > 5.0 || ra != base, "radial field has an effect");
        // centred on a bone it behaves like a static field when the bone does not move
        let bone = joints(&p, &simulate_chain(&p, &hang_names(), &v(800.0, 0.0, Some("pusher"))).unwrap(), 119.0)[3];
        assert!(bone.distance(base) > 1.0);
    }

    #[test]
    fn new_options_are_validated() {
        let p = hang();
        let n = hang_names();
        let bad = |o: PhysicsOptions| simulate_chain(&p, &n, &o).is_err();
        let cap = |a: Vec2, r: f32, f: Option<&str>| PhysicsOptions { capsules: vec![Capsule { a, b: Vec2::new(0.0, 0.0), radius: r, follow: f.map(String::from) }], ..opts() };
        assert!(bad(cap(Vec2::new(f32::NAN, 0.0), 5.0, None)));
        assert!(bad(cap(Vec2::new(1.0, 0.0), -1.0, None)));
        assert!(bad(cap(Vec2::new(1.0, 0.0), f32::INFINITY, None)));
        assert!(bad(cap(Vec2::new(1.0, 0.0), 5.0, Some("ghost"))));
        assert!(!bad(cap(Vec2::new(1.0, 0.0), 0.0, None)));
        assert!(bad(PhysicsOptions { colliders: vec![Circle { centre: Vec2::ZERO, radius: -2.0, follow: None }], ..opts() }));
        assert!(bad(PhysicsOptions { colliders: vec![Circle { centre: Vec2::ZERO, radius: 2.0, follow: Some("nope".into()) }], ..opts() }));
        assert!(bad(PhysicsOptions { drives: vec![Drive { bone: "nope".into(), gain: 1.0 }], ..opts() }));
        assert!(bad(PhysicsOptions { drives: vec![Drive { bone: "body".into(), gain: f32::NAN }], ..opts() }));
        assert!(bad(PhysicsOptions { vortices: vec![Vortex { centre: Vec2::ZERO, follow: Some("x".into()), swirl: 1.0, radial: 0.0, radius: 10.0 }], ..opts() }));
        assert!(bad(PhysicsOptions { vortices: vec![Vortex { centre: Vec2::ZERO, follow: None, swirl: f32::NAN, radial: 0.0, radius: 10.0 }], ..opts() }));
        assert!(bad(PhysicsOptions { vortices: vec![Vortex { centre: Vec2::ZERO, follow: None, swirl: 1.0, radial: 0.0, radius: -10.0 }], ..opts() }));
        assert!(bad(PhysicsOptions { wind: Vec2::new(f32::NAN, 0.0), ..opts() }));
    }

    #[test]
    fn locked_channels_are_not_overwritten_by_new_fields() {
        let mut p = hang();
        p.channels.push(Channel::keys("b", Prop::BoneAngle, Track::with_keys(&[(0.0, 5.0, Ease::Linear)])));
        crate::chanmeta::set_meta(&mut p, "b", Some(Prop::BoneAngle), &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        let o = PhysicsOptions { drives: vec![Drive { bone: "body".into(), gain: 1.0 }], ..opts() };
        let sim = simulate_chain(&p, &hang_names(), &o).unwrap();
        let (added, skipped) = merge_channels(&mut p, sim, true);
        assert_eq!(added, 2);
        assert!(skipped.contains(&"b".to_string()));
    }

    /// A square spiral whose last segment ends where the first starts: the capsules of segments 0 and 3 overlap at the root.
    fn spiral() -> Project {
        let mut p = Project::new("s", 300.0, 300.0, 24.0, 60);
        p.bones = vec![
            Bone::new("a", None, 20.0, Vec2::new(100.0, 100.0), 0.0),
            Bone::new("b", Some("a"), 20.0, Vec2::new(20.0, 0.0), 90.0),
            Bone::new("c", Some("b"), 20.0, Vec2::new(20.0, 0.0), 90.0),
            Bone::new("d", Some("c"), 20.0, Vec2::new(20.0, 0.0), 90.0),
        ];
        p
    }
    fn spiral_joints(p: &Project, ch: &[Channel], f: f32) -> Vec<Vec2> {
        let mut q = p.clone();
        q.channels.extend(ch.iter().cloned());
        let w = Engine::new(q).unwrap().evaluate(f).world;
        let mut v: Vec<Vec2> = w.iter().map(|m| Vec2::new(m.tx, m.ty)).collect();
        let a = w[3].rotation();
        v.push(v[3] + Vec2::new(a.cos(), a.sin()) * 20.0);
        v
    }

    #[test]
    fn self_collision_is_opt_in_and_keeps_overlapping_segments_apart() {
        let p = spiral();
        let names: Vec<String> = ["a", "b", "c", "d"].iter().map(|s| s.to_string()).collect();
        let still = PhysicsOptions { gravity: Vec2::ZERO, damping: 0.95, ..Default::default() };
        assert_eq!(still.self_collision, None);
        let off = simulate_chain(&p, &names, &still).unwrap();
        let j = spiral_joints(&p, &off, 59.0);
        assert!(seg_dist(j[0], j[1], j[3], j[4]) < 1.0, "without the option the tip stays on the root: {j:?}");
        // the default output is bit-identical to what the same options without the field gave: no extra work was done
        assert_eq!(off, simulate_chain(&p, &names, &PhysicsOptions { self_collision: None, ..still.clone() }).unwrap());
        let on = simulate_chain(&p, &names, &PhysicsOptions { self_collision: Some(4.0), ..still.clone() }).unwrap();
        for f in [10.0, 30.0, 59.0] {
            let j = spiral_joints(&p, &on, f);
            assert!(seg_dist(j[0], j[1], j[3], j[4]) >= 2.0 * 4.0 - 1.5, "frame {f}: segments 0 and 3 still overlap: {j:?}");
            for w in j.windows(2) {
                assert!((w[0].distance(w[1]) - 20.0).abs() < 1.0, "bone lengths hold: {j:?}");
            }
        }
        assert_eq!(on, simulate_chain(&p, &names, &PhysicsOptions { self_collision: Some(4.0), ..still.clone() }).unwrap(), "deterministic");
    }

    #[test]
    fn self_collision_radius_is_validated() {
        let p = spiral();
        let names: Vec<String> = ["a", "b", "c", "d"].iter().map(|s| s.to_string()).collect();
        for (r, what) in [(0.0, "finite number > 0"), (-1.0, "finite number > 0"), (f32::NAN, "finite number > 0"), (10.0, "too large"), (12.0, "too large")] {
            let e = simulate_chain(&p, &names, &PhysicsOptions { self_collision: Some(r), ..opts() }).unwrap_err();
            assert!(e.contains(what), "{r}: {e}");
        }
    }
}
