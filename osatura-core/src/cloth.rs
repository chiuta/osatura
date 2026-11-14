// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Cloth surface (flags, capes, curtains, banners): a deterministic position-based-dynamics
//! mass-spring grid (Verlet integration, structural + shear + bend distance constraints, pinned
//! vertices, gravity, wind with gusts and a travelling gust wave, circle and capsule colliders).
//! The simulated vertex positions are baked non-destructively: the layer gets a
//! `Deformer::Cloth` per-vertex displacement grid (bilinear, so each vertex moves exactly) and
//! one keyed `lattice_dx` / `lattice_dy` channel per vertex that actually moves. An image layer
//! with a matching mesh therefore deforms like cloth; its texture follows the grid. Textbook
//! PBD (Verlet + iterated distance projection); no external data.

use crate::anim::{Ease, Track};
use crate::chanmeta;
use crate::math::{Affine2, Vec2};
use crate::physics::{capsule_normal, closest_on_segment, Capsule, Circle};
use crate::project::{Channel, Deformer, LayerKind, MeshSpec, Project, Prop, Source};
use crate::Engine;

/// Which grid vertices are pinned.
#[derive(Clone, Debug, PartialEq)]
pub enum PinSel {
    Top,
    Bottom,
    Left,
    Right,
    /// The four corners.
    Corners,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    /// A whole grid row / column (0-based).
    Row(usize),
    Col(usize),
    /// One vertex by row-major index.
    Index(usize),
}

/// A set of pinned vertices. Without `bone` they ride the layer's own transform; with `bone`
/// they ride that bone's motion since frame 0 (so a hand can carry a flag).
#[derive(Clone, Debug, PartialEq)]
pub struct ClothPin {
    pub select: PinSel,
    pub bone: Option<String>,
}

/// Parse `top,left@hand,row:2,v:5,none`: comma-separated groups, each `selector[@bone]`.
/// Selectors: top bottom left right corners topleft topright bottomleft bottomright row:N col:N v:INDEX.
pub fn parse_pins(s: &str) -> Result<Vec<ClothPin>, String> {
    let mut out = Vec::new();
    for g in s.split(',').map(str::trim).filter(|g| !g.is_empty()) {
        if g == "none" {
            continue;
        }
        let (sel, bone) = match g.split_once('@') {
            Some((a, b)) if !b.is_empty() => (a, Some(b.to_string())),
            Some(_) => return Err(format!("pin '{g}': missing bone name after @")),
            None => (g, None),
        };
        let num = |v: &str| {
            v.parse::<usize>()
                .map_err(|_| format!("pin '{g}': bad number '{v}'"))
        };
        let select = match sel {
            "top" => PinSel::Top,
            "bottom" => PinSel::Bottom,
            "left" => PinSel::Left,
            "right" => PinSel::Right,
            "corners" => PinSel::Corners,
            "topleft" => PinSel::TopLeft,
            "topright" => PinSel::TopRight,
            "bottomleft" => PinSel::BottomLeft,
            "bottomright" => PinSel::BottomRight,
            _ => match sel.split_once(':') {
                Some(("row", n)) => PinSel::Row(num(n)?),
                Some(("col", n)) => PinSel::Col(num(n)?),
                Some(("v", n)) => PinSel::Index(num(n)?),
                _ => return Err(format!("unknown pin '{g}' (top bottom left right corners topleft.. row:N col:N v:N, optional @bone)")),
            },
        };
        out.push(ClothPin { select, bone });
    }
    Ok(out)
}

#[derive(Clone, Debug)]
pub struct ClothOptions {
    /// Grid cells; `None` = the image mesh's size if it has one, else 12.
    pub cols: Option<usize>,
    pub rows: Option<usize>,
    /// `[x, y, w, h]` in layer space; required for vector layers, default = the image rectangle.
    pub rect: Option<[f32; 4]>,
    /// Name of the deformer that is created / reused.
    pub name: String,
    pub pins: Vec<ClothPin>,
    /// px/s², +y falls.
    pub gravity: Vec2,
    pub wind: Vec2,
    /// wind x (1 + gust sin(2 pi (gust_hz t - s / gust_len))), s = distance along the wind.
    pub gust: f32,
    pub gust_hz: f32,
    /// Wavelength (px) of the gust travelling along the wind; 0 = the whole cloth gusts together.
    pub gust_len: f32,
    /// Velocity kept per substep (0.5..1).
    pub damping: f32,
    /// Structural (edge) stiffness: fraction of the stretch removed per substep, (0, 1].
    pub stiffness: f32,
    /// Shear (diagonal) stiffness, 0..1 (same meaning). The cloth lives in the picture plane, so a stiff shear
    /// makes the grid a rigid truss; the default leaves it compliant so the sheet can sway.
    pub shear: f32,
    /// Bend (skip-one) stiffness, 0..1; 0 = fully floppy.
    pub bend: f32,
    /// Strain limit: edges are pulled back to at most this multiple of their rest length (>= 1).
    pub max_stretch: f32,
    pub substeps: u32,
    pub iterations: u32,
    /// Frames of settling under the start frame's pose before the first baked frame.
    pub preroll: u32,
    pub colliders: Vec<Circle>,
    pub capsules: Vec<Capsule>,
    /// Frame range to bake (inclusive); default = the whole project.
    pub from: Option<u32>,
    pub to: Option<u32>,
}

impl Default for ClothOptions {
    fn default() -> Self {
        ClothOptions {
            cols: None,
            rows: None,
            rect: None,
            name: "cloth".into(),
            pins: vec![],
            gravity: Vec2::new(0.0, 900.0),
            wind: Vec2::ZERO,
            gust: 0.0,
            gust_hz: 0.5,
            gust_len: 0.0,
            damping: 0.99,
            stiffness: 1.0,
            shear: 0.05,
            bend: 0.2,
            max_stretch: 1.1,
            substeps: 4,
            iterations: 10,
            preroll: 0,
            colliders: vec![],
            capsules: vec![],
            from: None,
            to: None,
        }
    }
}

fn vertex_pinned(sel: &PinSel, c: usize, r: usize, cols: usize, rows: usize) -> bool {
    match sel {
        PinSel::Top => r == 0,
        PinSel::Bottom => r == rows,
        PinSel::Left => c == 0,
        PinSel::Right => c == cols,
        PinSel::Corners => (c == 0 || c == cols) && (r == 0 || r == rows),
        PinSel::TopLeft => c == 0 && r == 0,
        PinSel::TopRight => c == cols && r == 0,
        PinSel::BottomLeft => c == 0 && r == rows,
        PinSel::BottomRight => c == cols && r == rows,
        PinSel::Row(n) => r == *n,
        PinSel::Col(n) => c == *n,
        PinSel::Index(n) => r * (cols + 1) + c == *n,
    }
}

fn sub(a: Vec2, b: Vec2) -> Vec2 {
    a + (-b)
}

/// One distance constraint between particles `a` and `b`.
struct Link {
    a: usize,
    b: usize,
    rest: f32,
    k: f32,
    structural: bool,
}

/// Build the constraints of a `(cols+1)x(rows+1)` grid at rest positions `x`.
fn build_links(
    x: &[Vec2],
    cols: usize,
    rows: usize,
    stiffness: f32,
    shear: f32,
    bend: f32,
) -> Vec<Link> {
    let ix = |c: usize, r: usize| r * (cols + 1) + c;
    let mut v = Vec::new();
    let mut add = |a: usize, b: usize, k: f32, structural: bool| {
        if k > 0.0 {
            v.push(Link {
                a,
                b,
                rest: x[a].distance(x[b]),
                k,
                structural,
            });
        }
    };
    for r in 0..=rows {
        for c in 0..=cols {
            if c < cols {
                add(ix(c, r), ix(c + 1, r), stiffness, true);
            }
            if r < rows {
                add(ix(c, r), ix(c, r + 1), stiffness, true);
            }
            if c < cols && r < rows {
                add(ix(c, r), ix(c + 1, r + 1), shear, false);
                add(ix(c + 1, r), ix(c, r + 1), shear, false);
            }
            if c + 2 <= cols {
                add(ix(c, r), ix(c + 2, r), bend, false);
            }
            if r + 2 <= rows {
                add(ix(c, r), ix(c, r + 2), bend, false);
            }
        }
    }
    v
}

fn check_options(o: &ClothOptions) -> Result<(), String> {
    let fin2 = |v: Vec2| v.x.is_finite() && v.y.is_finite();
    if !fin2(o.gravity)
        || !fin2(o.wind)
        || !o.gust.is_finite()
        || !o.gust_hz.is_finite()
        || !o.gust_len.is_finite()
        || o.gust_len < 0.0
    {
        return Err("gravity, wind and gust must be finite and gust_len >= 0".into());
    }
    if !(0.5..=1.0).contains(&o.damping) {
        return Err("damping must be within 0.5..1".into());
    }
    if !(o.stiffness > 0.0 && o.stiffness <= 1.0) {
        return Err("stiffness must be within (0, 1]".into());
    }
    if !(0.0..=1.0).contains(&o.bend) || !(0.0..=1.0).contains(&o.shear) {
        return Err("bend and shear must be within 0..1".into());
    }
    if !(o.max_stretch.is_finite() && o.max_stretch >= 1.0) {
        return Err("max_stretch must be a number >= 1".into());
    }
    if o.substeps == 0 || o.substeps > 64 || o.iterations == 0 || o.iterations > 64 {
        return Err("substeps and iterations must be within 1..64".into());
    }
    if o.preroll > 600 {
        return Err("preroll is at most 600 frames".into());
    }
    if o.colliders.len() + o.capsules.len() > 256 {
        return Err("too many colliders".into());
    }
    for c in &o.colliders {
        if !fin2(c.centre) || !c.radius.is_finite() || c.radius < 0.0 {
            return Err("circle needs finite centre and a radius >= 0".into());
        }
    }
    for c in &o.capsules {
        if !fin2(c.a) || !fin2(c.b) || !c.radius.is_finite() || c.radius < 0.0 {
            return Err("capsule needs finite end points and a radius >= 0".into());
        }
    }
    if o.name.is_empty() || o.name.contains("::") || o.name.contains(',') {
        return Err("cloth name must be non-empty and contain neither '::' nor ','".into());
    }
    if let Some(r) = o.rect {
        if !r.iter().all(|v| v.is_finite()) || !(r[2] > 0.0 && r[3] > 0.0) {
            return Err("rect needs finite numbers and a positive size".into());
        }
    }
    Ok(())
}

/// Simulate the cloth of `layer` and return a new project with the deformer and the baked
/// channels added (the input project is not modified). Existing keyed `lattice_dx/dy` channels
/// of the same deformer are replaced; a locked one aborts the whole bake.
pub fn bake_cloth(p: &Project, layer: &str, o: &ClothOptions) -> Result<Project, String> {
    check_options(o)?;
    let lay = p
        .find_layer(layer)
        .ok_or_else(|| format!("no layer at '{layer}'"))?;
    if !matches!(lay.kind, LayerKind::Image { .. } | LayerKind::Vector { .. }) {
        return Err(format!("layer '{layer}' must be an image or vector layer"));
    }
    let (f_from, f_to) = (
        o.from.unwrap_or(0),
        o.to.unwrap_or(p.frames.saturating_sub(1)),
    );
    if f_from > f_to || f_to >= p.frames {
        return Err(format!(
            "frame range {f_from}..{f_to} is outside the project (0..{})",
            p.frames.saturating_sub(1)
        ));
    }
    // grid size
    let mesh_dims = match &lay.kind {
        LayerKind::Image { mesh: Some(m), .. } => Some((m.cols, m.rows)),
        _ => None,
    };
    if let LayerKind::Image { mesh: Some(m), .. } = &lay.kind {
        if m.free.is_some() {
            return Err("cloth needs a grid mesh; the layer has a free-form (traced) mesh".into());
        }
        if m.weights.iter().any(|w| !w.is_empty()) {
            return Err("the layer's mesh is bone-skinned; cloth would be skinned twice. Use an unskinned mesh".into());
        }
    }
    let cols = o.cols.or(mesh_dims.map(|d| d.0)).unwrap_or(12);
    let rows = o.rows.or(mesh_dims.map(|d| d.1)).unwrap_or(12);
    if !(1..=64).contains(&cols) || !(1..=64).contains(&rows) {
        return Err("cloth grid must be 1..64 cells per side".into());
    }
    for pn in &o.pins {
        if let Some(b) = &pn.bone {
            if !p.bones.iter().any(|x| &x.name == b) {
                return Err(format!("pin bone '{b}' does not exist"));
            }
        }
        match pn.select {
            PinSel::Row(n) if n > rows => return Err(format!("pin row {n} outside 0..{rows}")),
            PinSel::Col(n) if n > cols => return Err(format!("pin col {n} outside 0..{cols}")),
            PinSel::Index(n) if n >= (cols + 1) * (rows + 1) => {
                return Err(format!(
                    "pin vertex {n} outside 0..{}",
                    (cols + 1) * (rows + 1) - 1
                ))
            }
            _ => {}
        }
    }
    let nv = (cols + 1) * (rows + 1);

    // simulation world: layer -> stage without the camera
    let mut sp = p.clone();
    sp.camera = Default::default();
    sp.channels.retain(|c| {
        !matches!(
            c.prop,
            Prop::CameraX | Prop::CameraY | Prop::CameraZoom | Prop::CameraRotation
        )
    });
    // do not feed back earlier cloth keys of this deformer
    let tprefix = format!("{layer}::{}::", o.name);
    sp.channels.retain(|c| {
        !(c.target.starts_with(&tprefix) && matches!(c.prop, Prop::LatticeDx | Prop::LatticeDy))
    });
    let e = Engine::new(sp).map_err(|v| v.join("; "))?;
    let id = e
        .node_index(layer)
        .ok_or_else(|| format!("no layer at '{layer}'"))?;
    let f0 = e.evaluate(0.0);
    let rect = match (o.rect, f0.nodes[id].image.as_ref()) {
        (Some(r), _) => r,
        (None, Some(im)) => [im.origin.x, im.origin.y, im.size.x, im.size.y],
        (None, None) => return Err("a vector layer needs rect=x,y,w,h".into()),
    };
    if !(rect[2] > 0.0 && rect[3] > 0.0) {
        return Err("the cloth rectangle has no size".into());
    }
    let rest: Vec<Vec2> = (0..=rows)
        .flat_map(|r| (0..=cols).map(move |c| (c, r)))
        .map(|(c, r)| {
            Vec2::new(
                rect[0] + rect[2] * c as f32 / cols as f32,
                rect[1] + rect[3] * r as f32 / rows as f32,
            )
        })
        .collect();

    // pin table: per vertex None | Some(bone index option)
    let bone_ix = |n: &str| p.bones.iter().position(|b| b.name == n);
    let mut pin: Vec<Option<Option<usize>>> = vec![None; nv];
    for pn in &o.pins {
        let bi = pn.bone.as_deref().and_then(bone_ix);
        for r in 0..=rows {
            for c in 0..=cols {
                if vertex_pinned(&pn.select, c, r, cols, rows) {
                    pin[r * (cols + 1) + c] = Some(bi);
                }
            }
        }
    }
    let node_world = |fr: &crate::engine::Frame| fr.nodes[id].world;
    let m0 = node_world(&f0);
    let x0: Vec<Vec2> = rest.iter().map(|&v| m0.apply(v)).collect();
    // stiffness is the fraction of the error removed per substep, whatever the iteration count
    let per_iter = |k: f32| 1.0 - (1.0 - k).powf(1.0 / o.iterations as f32);
    let links = build_links(
        &x0,
        cols,
        rows,
        per_iter(o.stiffness),
        per_iter(o.shear),
        per_iter(o.bend),
    );
    let inv_m0 = |b: usize| f0.world[b].inverse();
    for pn in &o.pins {
        if let Some(b) = pn.bone.as_deref().and_then(bone_ix) {
            inv_m0(b)
                .ok_or_else(|| format!("bone '{}' has a degenerate transform", p.bones[b].name))?;
        }
    }
    let circ_bone: Vec<Option<usize>> = o
        .colliders
        .iter()
        .map(|c| {
            c.follow
                .as_deref()
                .map(|n| bone_ix(n).ok_or_else(|| format!("no bone '{n}'")))
                .transpose()
        })
        .collect::<Result<_, _>>()?;
    let cap_bone: Vec<Option<usize>> = o
        .capsules
        .iter()
        .map(|c| {
            c.follow
                .as_deref()
                .map(|n| bone_ix(n).ok_or_else(|| format!("no bone '{n}'")))
                .transpose()
        })
        .collect::<Result<_, _>>()?;

    type Shape = (Vec2, Vec2, f32);
    let shapes_at = |fr: &crate::engine::Frame| -> Vec<Shape> {
        let mv = |bone: Option<usize>| -> Option<Affine2> {
            bone.and_then(|i| f0.world[i].inverse().map(|inv| fr.world[i].mul(inv)))
        };
        let mut v: Vec<Shape> = Vec::new();
        for (c, b) in o.colliders.iter().zip(&circ_bone) {
            let q = mv(*b).map_or(c.centre, |m| m.apply(c.centre));
            v.push((q, q, c.radius));
        }
        for (c, b) in o.capsules.iter().zip(&cap_bone) {
            let m = mv(*b);
            v.push((
                m.map_or(c.a, |m| m.apply(c.a)),
                m.map_or(c.b, |m| m.apply(c.b)),
                c.radius,
            ));
        }
        v
    };
    // pinned target positions at one frame
    let targets_at = |fr: &crate::engine::Frame| -> Vec<Option<Vec2>> {
        let m = node_world(fr);
        (0..nv)
            .map(|i| {
                pin[i].map(|b| match b {
                    None => m.apply(rest[i]),
                    Some(bi) => f0.world[bi]
                        .inverse()
                        .map_or(x0[i], |inv| fr.world[bi].mul(inv).apply(x0[i])),
                })
            })
            .collect()
    };

    let dt = 1.0 / p.fps;
    let h = dt / o.substeps as f32;
    let fr_from = e.evaluate(f_from as f32);
    // start: the rest grid carried along by the layer's own motion since frame 0
    let m_from = node_world(&fr_from);
    let mut pos: Vec<Vec2> = rest.iter().map(|&v| m_from.apply(v)).collect();
    let tg0 = targets_at(&fr_from);
    for i in 0..nv {
        if let Some(t) = tg0[i] {
            pos[i] = t;
        }
    }
    let mut prev = pos.clone();
    let mut prev_targets = tg0.clone();
    let mut prev_shapes = shapes_at(&fr_from);
    let wind_dir = if o.wind.length() > 1e-9 {
        o.wind.normalized()
    } else {
        Vec2::ZERO
    };

    // one frame of simulation towards `targets` / `shapes`
    let step = |pos: &mut Vec<Vec2>,
                prev: &mut Vec<Vec2>,
                prev_targets: &mut Vec<Option<Vec2>>,
                prev_shapes: &mut Vec<Shape>,
                targets: &[Option<Vec2>],
                shapes: &[Shape],
                t: f32| {
        for sstep in 0..o.substeps {
            let u = (sstep + 1) as f32 / o.substeps as f32;
            let cur: Vec<Shape> = prev_shapes
                .iter()
                .zip(shapes)
                .map(|(a, b)| (a.0.lerp(b.0, u), a.1.lerp(b.1, u), b.2))
                .collect();
            for i in 0..nv {
                if let (Some(a), Some(b)) = (prev_targets[i], targets[i]) {
                    let q = a.lerp(b, u);
                    pos[i] = q;
                    prev[i] = q;
                    continue;
                }
                let v = sub(pos[i], prev[i]) * o.damping;
                prev[i] = pos[i];
                let phase = if o.gust_len > 0.0 {
                    wind_dir.dot(x0[i]) / o.gust_len
                } else {
                    0.0
                };
                let wind = o.wind
                    * (1.0 + o.gust * (std::f32::consts::TAU * (o.gust_hz * t - phase)).sin());
                pos[i] = pos[i] + v + (o.gravity + wind) * (h * h);
            }
            for _ in 0..o.iterations {
                for l in &links {
                    let (wa, wb) = (
                        if prev_targets[l.a].is_some() && targets[l.a].is_some() {
                            0.0
                        } else {
                            1.0
                        },
                        if prev_targets[l.b].is_some() && targets[l.b].is_some() {
                            0.0
                        } else {
                            1.0
                        },
                    );
                    let w = wa + wb;
                    if w == 0.0 {
                        continue;
                    }
                    let d = sub(pos[l.b], pos[l.a]);
                    let len = d.length().max(1e-6);
                    let corr = d * ((len - l.rest) / len * l.k / w);
                    pos[l.a] += corr * wa;
                    pos[l.b] = sub(pos[l.b], corr * wb);
                }
                for i in 0..nv {
                    if targets[i].is_some() && prev_targets[i].is_some() {
                        continue;
                    }
                    for &(a, b, r) in &cur {
                        let cp = closest_on_segment(a, b, pos[i]);
                        let d = sub(pos[i], cp);
                        let l = d.length();
                        if l < r {
                            pos[i] = cp
                                + if l > 1e-6 {
                                    d * (r / l)
                                } else {
                                    capsule_normal(a, b) * r
                                };
                        }
                    }
                }
            }
            // strain limiting on the edges
            for _ in 0..4 {
                for l in links.iter().filter(|l| l.structural) {
                    let (wa, wb) = (
                        if targets[l.a].is_some() && prev_targets[l.a].is_some() {
                            0.0
                        } else {
                            1.0
                        },
                        if targets[l.b].is_some() && prev_targets[l.b].is_some() {
                            0.0
                        } else {
                            1.0
                        },
                    );
                    let w = wa + wb;
                    if w == 0.0 {
                        continue;
                    }
                    let d = sub(pos[l.b], pos[l.a]);
                    let len = d.length();
                    let maxl = l.rest * o.max_stretch;
                    if len > maxl {
                        let corr = d * ((len - maxl) / len / w);
                        pos[l.a] += corr * wa;
                        pos[l.b] = sub(pos[l.b], corr * wb);
                    }
                }
            }
        }
        *prev_targets = targets.to_vec();
        *prev_shapes = shapes.to_vec();
    };

    // pre-roll: settle under the start frame's pose
    let shapes_from = shapes_at(&fr_from);
    for k in 0..o.preroll {
        let t = (f_from as f32 - (o.preroll - k) as f32) * dt;
        step(
            &mut pos,
            &mut prev,
            &mut prev_targets,
            &mut prev_shapes,
            &tg0,
            &shapes_from,
            t,
        );
    }

    // frames
    let nf = (f_to - f_from + 1) as usize;
    let mut keys: Vec<Vec<(f32, Vec2)>> = vec![Vec::with_capacity(nf); nv];
    for f in f_from..=f_to {
        let fr = e.evaluate(f as f32);
        if f > f_from || o.preroll > 0 {
            let tg = targets_at(&fr);
            let sh = shapes_at(&fr);
            if f > f_from {
                step(
                    &mut pos,
                    &mut prev,
                    &mut prev_targets,
                    &mut prev_shapes,
                    &tg,
                    &sh,
                    f as f32 * dt,
                );
            }
        }
        let minv = node_world(&fr)
            .inverse()
            .ok_or_else(|| format!("layer '{layer}' has a degenerate transform at frame {f}"))?;
        for i in 0..nv {
            if !pos[i].x.is_finite() || !pos[i].y.is_finite() {
                return Err("cloth simulation diverged (non-finite position); lower the forces or raise iterations".into());
            }
            let mut off = sub(minv.apply(pos[i]), rest[i]);
            if off.x.abs() < 1e-4 {
                off.x = 0.0;
            }
            if off.y.abs() < 1e-4 {
                off.y = 0.0;
            }
            keys[i].push((f as f32, off));
        }
    }

    // apply to a copy: deformer, mesh, channels
    let mut q = p.clone();
    {
        let l = q
            .find_layer_mut(layer)
            .ok_or_else(|| format!("no layer at '{layer}'"))?;
        let same = |a: [f32; 4]| a.iter().zip(rect.iter()).all(|(x, y)| (x - y).abs() < 1e-3);
        match l.deformers.iter().find(|d| d.name() == o.name) {
            Some(Deformer::Cloth {
                cols: c,
                rows: r,
                rect: rc,
                ..
            }) if *c == cols && *r == rows && same(*rc) => {}
            Some(_) => return Err(format!(
                "layer '{layer}' already has a different deformer named '{}'; choose another name=",
                o.name
            )),
            None => l.deformers.push(Deformer::Cloth {
                name: o.name.clone(),
                cols,
                rows,
                rect,
                offsets: vec![],
            }),
        }
        if let LayerKind::Image { mesh, .. } = &mut l.kind {
            match mesh {
                None => {
                    *mesh = Some(MeshSpec {
                        cols,
                        rows,
                        weights: vec![],
                        ..Default::default()
                    })
                }
                Some(m) => {
                    m.cols = cols;
                    m.rows = rows;
                }
            }
        }
    }
    // locks: every vertex channel of the deformer, written or removed
    let tgt = |i: usize| format!("{layer}::{}::{},{}", o.name, i % (cols + 1), i / (cols + 1));
    for i in 0..nv {
        chanmeta::assert_unlocked(p, &tgt(i), Prop::LatticeDx)?;
        chanmeta::assert_unlocked(p, &tgt(i), Prop::LatticeDy)?;
    }
    q.channels.retain(|c| {
        !(c.target.starts_with(&tprefix)
            && matches!(c.prop, Prop::LatticeDx | Prop::LatticeDy)
            && matches!(c.source, Source::Keys { .. }))
    });
    // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
    #[allow(clippy::needless_range_loop)]
    for i in 0..nv {
        if keys[i].iter().all(|k| k.1.x == 0.0 && k.1.y == 0.0) {
            continue;
        }
        let kx: Vec<(f32, f32, Ease)> =
            keys[i].iter().map(|k| (k.0, k.1.x, Ease::Linear)).collect();
        let ky: Vec<(f32, f32, Ease)> =
            keys[i].iter().map(|k| (k.0, k.1.y, Ease::Linear)).collect();
        q.channels.push(Channel::keys(
            &tgt(i),
            Prop::LatticeDx,
            Track::with_keys(&kx),
        ));
        q.channels.push(Channel::keys(
            &tgt(i),
            Prop::LatticeDy,
            Track::with_keys(&ky),
        ));
    }
    Engine::new(q.clone()).map_err(|v| v.join("; "))?;
    Ok(q)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{Bone, Layer};

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }
    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut pm = tiny_skia::Pixmap::new(w, h).unwrap();
        pm.fill(tiny_skia::Color::from_rgba8(200, 40, 40, 255));
        pm.encode_png().unwrap()
    }
    fn proj() -> Project {
        let mut p = Project::new("c", 400.0, 400.0, 24.0, 48);
        p.layers = vec![Layer::image_from_png(
            "flag",
            &png(120, 120),
            v(140.0, 40.0),
        )];
        p
    }
    fn opts() -> ClothOptions {
        ClothOptions {
            cols: Some(8),
            rows: Some(8),
            pins: parse_pins("top").unwrap(),
            ..Default::default()
        }
    }
    fn verts(p: &Project, f: f32) -> Vec<Vec2> {
        let e = Engine::new(p.clone()).unwrap();
        let fr = e.evaluate(f);
        let id = e.node_index("flag").unwrap();
        fr.nodes[id]
            .image
            .as_ref()
            .unwrap()
            .mesh
            .as_ref()
            .unwrap()
            .positions
            .clone()
    }
    const W: usize = 9;

    #[test]
    fn parse_pin_specs() {
        let p = parse_pins("top, left@hand,row:2,v:5,none").unwrap();
        assert_eq!(p.len(), 4);
        assert_eq!(
            p[1],
            ClothPin {
                select: PinSel::Left,
                bone: Some("hand".into())
            }
        );
        assert_eq!(p[2].select, PinSel::Row(2));
        assert_eq!(p[3].select, PinSel::Index(5));
        assert!(parse_pins("sideways").is_err());
        assert!(parse_pins("row:x").is_err());
        assert!(parse_pins("top@").is_err());
        assert!(parse_pins("none").unwrap().is_empty());
    }

    #[test]
    fn pinned_top_row_stays_and_free_corner_falls_within_stretch() {
        let p = proj();
        let q = bake_cloth(&p, "flag", &opts()).unwrap();
        let f0 = verts(&q, 0.0);
        assert!((f0[0].x - 140.0).abs() < 1e-3 && (f0[8 * W + 8].y - 160.0).abs() < 1e-3);
        for f in [0.0, 10.0, 25.0, 47.0] {
            let vs = verts(&q, f);
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for c in 0..W {
                let want = v(140.0 + 15.0 * c as f32, 40.0);
                assert!(
                    vs[c].distance(want) < 1e-2,
                    "top vertex {c} moved at frame {f}: {:?}",
                    vs[c]
                );
            }
        }
        // hanging: with 4 rows free below the pins the bottom stays within the strain limit
        let last = verts(&q, 47.0);
        for r in 0..=8 {
            for c in 0..8 {
                let d = last[r * W + c].distance(last[r * W + c + 1]);
                assert!(d <= 15.0 * 1.15, "edge {c},{r} stretched to {d}");
            }
        }
        for r in 0..8 {
            for c in 0..W {
                let d = last[r * W + c].distance(last[(r + 1) * W + c]);
                assert!(d <= 15.0 * 1.15, "vertical edge {c},{r} stretched to {d}");
            }
        }
        // free corner released: pin only the top-left vertex, the bottom-right corner swings down
        let q2 = bake_cloth(
            &p,
            "flag",
            &ClothOptions {
                pins: parse_pins("topleft").unwrap(),
                ..opts()
            },
        )
        .unwrap();
        let a = verts(&q2, 0.0)[8 * W + 8];
        let b = verts(&q2, 47.0)[8 * W + 8];
        assert!(b.y > a.y + 40.0, "corner fell from {a:?} to {b:?}");
        assert!(
            b.distance(v(140.0, 40.0)) <= 16.0 * 15.0 * 1.15,
            "corner within reach of the pin (grid path)"
        );
    }

    #[test]
    fn unpinned_cloth_falls_freely() {
        let p = proj();
        let q = bake_cloth(
            &p,
            "flag",
            &ClothOptions {
                pins: vec![],
                ..opts()
            },
        )
        .unwrap();
        let (a, b) = (verts(&q, 0.0)[4 * W + 4], verts(&q, 47.0)[4 * W + 4]);
        assert!(b.y > a.y + 500.0, "free fall {a:?} -> {b:?}");
    }

    #[test]
    fn circle_collider_stops_the_cloth() {
        let p = proj();
        // the wind leans the hanging cloth against a ball that sits beside it
        let ball = Circle {
            centre: v(290.0, 110.0),
            radius: 25.0,
            follow: None,
        };
        let windy = ClothOptions {
            wind: v(1500.0, 0.0),
            ..opts()
        };
        let free = bake_cloth(&p, "flag", &windy).unwrap();
        let hit = bake_cloth(
            &p,
            "flag",
            &ClothOptions {
                colliders: vec![ball.clone()],
                ..windy
            },
        )
        .unwrap();
        let inside = |vs: &[Vec2]| {
            vs.iter()
                .filter(|q| q.distance(ball.centre) < ball.radius - 0.5)
                .count()
        };
        assert!(
            (0..48).any(|f| inside(&verts(&free, f as f32)) > 0),
            "without a collider the cloth hangs through the ball"
        );
        for f in 0..48 {
            assert_eq!(
                inside(&verts(&hit, f as f32)),
                0,
                "frame {f}: a vertex is inside the ball"
            );
        }
    }

    #[test]
    fn capsule_collider_stops_the_cloth() {
        let p = proj();
        let cap = Capsule {
            a: v(285.0, 60.0),
            b: v(285.0, 160.0),
            radius: 15.0,
            follow: None,
        };
        let windy = ClothOptions {
            wind: v(1500.0, 0.0),
            ..opts()
        };
        let free = bake_cloth(&p, "flag", &windy).unwrap();
        let near = |q: &Project| {
            (0..48).any(|f| {
                verts(q, f as f32).iter().any(|pt| {
                    closest_on_segment(cap.a, cap.b, *pt).distance(*pt) < cap.radius - 0.5
                })
            })
        };
        assert!(near(&free), "without the capsule the cloth enters it");
        let q = bake_cloth(
            &p,
            "flag",
            &ClothOptions {
                capsules: vec![cap.clone()],
                ..windy
            },
        )
        .unwrap();
        for f in 0..48 {
            for pt in verts(&q, f as f32) {
                let cp = closest_on_segment(cap.a, cap.b, pt);
                assert!(
                    cp.distance(pt) >= cap.radius - 0.5,
                    "vertex {pt:?} inside capsule at {f}"
                );
            }
        }
    }

    #[test]
    fn deterministic_and_non_destructive() {
        let mut p = proj();
        p.bones = vec![Bone::new("b", None, 10.0, v(0.0, 0.0), 0.0)];
        let before = serde_json::to_string(&p).unwrap();
        let o = ClothOptions {
            wind: v(300.0, 0.0),
            gust: 0.5,
            gust_len: 80.0,
            ..opts()
        };
        let a = bake_cloth(&p, "flag", &o).unwrap();
        let b = bake_cloth(&p, "flag", &o).unwrap();
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            before,
            "input project untouched"
        );
        // baking again replaces the keys instead of piling them up
        let c = bake_cloth(&a, "flag", &o).unwrap();
        assert_eq!(c.channels.len(), a.channels.len());
        assert_eq!(c.layers[0].deformers.len(), 1);
        assert_eq!(
            serde_json::to_string(&c).unwrap(),
            serde_json::to_string(&a).unwrap(),
            "re-bake does not feed back"
        );
    }

    #[test]
    fn wind_pushes_sideways() {
        let p = proj();
        let q = bake_cloth(
            &p,
            "flag",
            &ClothOptions {
                wind: v(1500.0, 0.0),
                ..opts()
            },
        )
        .unwrap();
        let last = verts(&q, 47.0);
        assert!(
            last[8 * W + 4].x > 200.0 + 25.0,
            "bottom middle blown right: {:?}",
            last[8 * W + 4]
        );
    }

    #[test]
    fn serde_roundtrip_and_old_projects() {
        let q = bake_cloth(&proj(), "flag", &opts()).unwrap();
        let s = serde_json::to_string(&q).unwrap();
        let r: Project = serde_json::from_str(&s).unwrap();
        assert_eq!(serde_json::to_string(&r).unwrap(), s);
        assert_eq!(verts(&q, 20.0), verts(&r, 20.0));
        // a project without any cloth deformer or channel still loads and has no deformers
        let old: Project = serde_json::from_str(&serde_json::to_string(&proj()).unwrap()).unwrap();
        assert!(old.layers[0].deformers.is_empty());
        // the new variant parses from plain JSON with defaults
        let d: Deformer = serde_json::from_str(
            r#"{"type":"cloth","name":"c","cols":2,"rows":2,"rect":[0,0,10,10]}"#,
        )
        .unwrap();
        assert!(matches!(d, Deformer::Cloth { ref offsets, .. } if offsets.is_empty()));
    }

    #[test]
    fn pins_follow_a_bone_and_the_layer() {
        let mut p = proj();
        p.bones = vec![Bone::new("hand", None, 10.0, v(100.0, 40.0), 0.0)];
        p.channels = vec![Channel::keys(
            "hand",
            Prop::BoneOffsetX,
            Track::with_keys(&[(0.0, 0.0, Ease::Linear), (47.0, 100.0, Ease::Linear)]),
        )];
        let q = bake_cloth(
            &p,
            "flag",
            &ClothOptions {
                pins: parse_pins("top@hand").unwrap(),
                ..opts()
            },
        )
        .unwrap();
        let vs = verts(&q, 47.0);
        assert!(
            vs[0].distance(v(240.0, 40.0)) < 0.05 && vs[8].distance(v(360.0, 40.0)) < 0.05,
            "{:?} {:?}",
            vs[0],
            vs[8]
        );
        assert!(vs[8 * W].x > 200.0, "the cloth is dragged along");
        // a layer that itself moves: the pinned row rides along, offsets stay in layer space
        let mut p2 = proj();
        p2.channels = vec![Channel::keys(
            "flag",
            Prop::LayerX,
            Track::with_keys(&[(0.0, 0.0, Ease::Linear), (47.0, 80.0, Ease::Linear)]),
        )];
        let q2 = bake_cloth(&p2, "flag", &opts()).unwrap();
        let top = verts(&q2, 47.0)[0];
        assert!(
            top.distance(v(140.0, 40.0)) < 1e-3,
            "layer-space top row unchanged: {top:?}"
        );
        let e = Engine::new(q2).unwrap();
        let id = e.node_index("flag").unwrap();
        let w = e.evaluate(47.0).nodes[id].world.apply(top);
        assert!(w.distance(v(220.0, 40.0)) < 1e-3, "{w:?}");
    }

    #[test]
    fn frame_range_limits_the_keys() {
        let q = bake_cloth(
            &proj(),
            "flag",
            &ClothOptions {
                from: Some(10),
                to: Some(20),
                ..opts()
            },
        )
        .unwrap();
        for c in &q.channels {
            if let Source::Keys { track } = &c.source {
                let (a, b) = (track.eval(9.0), track.eval(10.0));
                assert_eq!(a, b, "constant before the range");
            }
        }
        assert!(!q.channels.is_empty());
    }

    #[test]
    fn validation_errors() {
        let p = proj();
        let bad = |o: ClothOptions| bake_cloth(&p, "flag", &o).unwrap_err();
        assert!(bad(ClothOptions {
            stiffness: 0.0,
            ..opts()
        })
        .contains("stiffness"));
        assert!(bad(ClothOptions {
            stiffness: f32::NAN,
            ..opts()
        })
        .contains("stiffness"));
        assert!(bad(ClothOptions {
            bend: 2.0,
            ..opts()
        })
        .contains("bend"));
        assert!(bad(ClothOptions {
            shear: -0.1,
            ..opts()
        })
        .contains("shear"));
        assert!(bad(ClothOptions {
            damping: 0.1,
            ..opts()
        })
        .contains("damping"));
        assert!(bad(ClothOptions {
            max_stretch: 0.5,
            ..opts()
        })
        .contains("max_stretch"));
        assert!(bad(ClothOptions {
            gravity: v(f32::NAN, 0.0),
            ..opts()
        })
        .contains("finite"));
        assert!(bad(ClothOptions {
            wind: v(f32::INFINITY, 0.0),
            ..opts()
        })
        .contains("finite"));
        assert!(bad(ClothOptions {
            substeps: 0,
            ..opts()
        })
        .contains("substeps"));
        assert!(bad(ClothOptions {
            cols: Some(0),
            ..opts()
        })
        .contains("grid"));
        assert!(bad(ClothOptions {
            cols: Some(100),
            ..opts()
        })
        .contains("grid"));
        assert!(bad(ClothOptions {
            from: Some(30),
            to: Some(10),
            ..opts()
        })
        .contains("frame range"));
        assert!(bad(ClothOptions {
            to: Some(999),
            ..opts()
        })
        .contains("frame range"));
        assert!(bad(ClothOptions {
            pins: parse_pins("row:99").unwrap(),
            ..opts()
        })
        .contains("row"));
        assert!(bad(ClothOptions {
            pins: parse_pins("v:999").unwrap(),
            ..opts()
        })
        .contains("vertex"));
        assert!(bad(ClothOptions {
            pins: parse_pins("top@ghost").unwrap(),
            ..opts()
        })
        .contains("ghost"));
        assert!(bad(ClothOptions {
            colliders: vec![Circle {
                centre: v(0.0, 0.0),
                radius: -1.0,
                follow: None
            }],
            ..opts()
        })
        .contains("radius"));
        assert!(bad(ClothOptions {
            colliders: vec![Circle {
                centre: v(0.0, 0.0),
                radius: 5.0,
                follow: Some("nobody".into())
            }],
            ..opts()
        })
        .contains("nobody"));
        assert!(bad(ClothOptions {
            name: "a::b".into(),
            ..opts()
        })
        .contains("name"));
        assert!(bake_cloth(&p, "nope", &opts())
            .unwrap_err()
            .contains("no layer"));
        // vector layers need a rect
        let mut pv = proj();
        pv.layers = vec![Layer::vector("vec", vec![])];
        assert!(bake_cloth(&pv, "vec", &opts())
            .unwrap_err()
            .contains("rect"));
        assert!(bake_cloth(
            &pv,
            "vec",
            &ClothOptions {
                rect: Some([0.0, 0.0, 0.0, 10.0]),
                ..opts()
            }
        )
        .unwrap_err()
        .contains("rect"));
        // a different deformer with the same name is not overwritten
        let mut pd = proj();
        pd.layers[0].deformers = vec![Deformer::Lattice {
            name: "cloth".into(),
            cols: 1,
            rows: 1,
            rect: [0.0, 0.0, 1.0, 1.0],
            offsets: vec![],
        }];
        assert!(bake_cloth(&pd, "flag", &opts())
            .unwrap_err()
            .contains("already has"));
        // a skinned mesh is refused
        let mut ps = proj();
        ps.bones = vec![Bone::new("b", None, 10.0, v(0.0, 0.0), 0.0)];
        if let LayerKind::Image { mesh, .. } = &mut ps.layers[0].kind {
            *mesh = Some(MeshSpec {
                cols: 8,
                rows: 8,
                weights: vec![
                    vec![crate::path::BoneWeight {
                        bone: "b".into(),
                        weight: 1.0
                    }];
                    81
                ],
                ..Default::default()
            });
        }
        assert!(bake_cloth(&ps, "flag", &opts())
            .unwrap_err()
            .contains("skinned"));
    }

    #[test]
    fn vector_layer_with_rect_deforms() {
        let mut p = proj();
        p.layers = vec![Layer::vector(
            "vec",
            vec![crate::path::VPath::rect("r", 140.0, 40.0, 120.0, 120.0)],
        )];
        let q = bake_cloth(
            &p,
            "vec",
            &ClothOptions {
                rect: Some([140.0, 40.0, 120.0, 120.0]),
                ..opts()
            },
        )
        .unwrap();
        assert!(matches!(q.layers[0].deformers[0], Deformer::Cloth { .. }));
        assert!(!q.channels.is_empty());
    }

    #[test]
    fn locked_channel_aborts_without_changes() {
        let p = proj();
        let mut q = bake_cloth(&p, "flag", &opts()).unwrap();
        let target = q
            .channels
            .iter()
            .find(|c| c.prop == Prop::LatticeDx)
            .unwrap()
            .target
            .clone();
        chanmeta::set_meta(
            &mut q,
            &target,
            Some(Prop::LatticeDx),
            &chanmeta::Change {
                lock: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        let before = serde_json::to_string(&q).unwrap();
        let err = bake_cloth(
            &q,
            "flag",
            &ClothOptions {
                gravity: v(0.0, 400.0),
                ..opts()
            },
        )
        .unwrap_err();
        assert!(err.contains("locked"), "{err}");
        assert_eq!(serde_json::to_string(&q).unwrap(), before);
    }

    #[test]
    fn cloth_deformer_is_exact_at_vertices_and_validated() {
        let mut p = proj();
        let mut offs = vec![Vec2::ZERO; 9];
        offs[4] = v(7.0, -3.0);
        p.layers[0].deformers = vec![Deformer::Cloth {
            name: "k".into(),
            cols: 2,
            rows: 2,
            rect: [140.0, 40.0, 120.0, 120.0],
            offsets: offs,
        }];
        if let LayerKind::Image { mesh, .. } = &mut p.layers[0].kind {
            *mesh = Some(MeshSpec {
                cols: 2,
                rows: 2,
                weights: vec![],
                ..Default::default()
            });
        }
        let vs = verts(&p, 0.0);
        assert!(
            vs[4].distance(v(200.0 + 7.0, 100.0 - 3.0)) < 1e-4,
            "centre vertex moved exactly"
        );
        assert!(vs[0].distance(v(140.0, 40.0)) < 1e-6 && vs[8].distance(v(260.0, 160.0)) < 1e-6);
        assert!(vs[1].distance(v(200.0, 40.0 - 0.0)) < 1e-4);
        // wrong offset count and empty grid are validation errors
        p.layers[0].deformers = vec![Deformer::Cloth {
            name: "k".into(),
            cols: 2,
            rows: 2,
            rect: [0.0, 0.0, 1.0, 1.0],
            offsets: vec![Vec2::ZERO; 3],
        }];
        assert!(Engine::new(p.clone()).is_err());
        p.layers[0].deformers = vec![Deformer::Cloth {
            name: "k".into(),
            cols: 0,
            rows: 2,
            rect: [0.0, 0.0, 1.0, 1.0],
            offsets: vec![],
        }];
        assert!(Engine::new(p).is_err());
    }
}
