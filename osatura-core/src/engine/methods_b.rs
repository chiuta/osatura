// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `engine` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

impl Engine {
    pub fn bone_index(&self, name: &str) -> Option<usize> {
        self.bone_ix.get(name).copied()
    }
    pub fn node_index(&self, path: &str) -> Option<usize> {
        self.node_ix.get(path).copied()
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Index path of node `id` into `project.layers` / `children()` (for building JSON pointers).
    pub fn node_idx_path(&self, id: usize) -> &[usize] {
        &self.nodes[id].idx_path
    }
    /// Parent node of `id` (`None` for top-level layers).
    pub fn node_parent(&self, id: usize) -> Option<usize> {
        self.nodes[id].parent
    }
    /// The layer's evaluated local transform at `frame` (all channels, layout offsets and the
    /// static transform combined; the anchor is the layer's). Used by the canvas gizmos.
    pub fn layer_local_at(&self, frame: f32, id: usize) -> crate::project::Transform2D {
        let (mut st, _) = self.pose_pipeline(frame);
        if let Some(Some([dx, dy, kx, ky])) = self.layout.get(id) {
            let lp = &mut st.layers[id];
            lp.x += dx;
            lp.y += dy;
            lp.sx *= kx;
            lp.sy *= ky;
        }
        let lp = &st.layers[id];
        crate::project::Transform2D {
            position: Vec2::new(lp.x, lp.y),
            rotation: lp.rot,
            scale: Vec2::new(lp.sx, lp.sy),
            skew: lp.skew,
            anchor: self.layer_ref(id).transform.anchor,
        }
    }

    /// Rest-pose geometry of node `id` for `links.rs` (layer space, static transforms only).
    pub(crate) fn link_unit(&self, id: usize) -> Option<crate::links::UnitRest> {
        let layer = self.layer_ref(id);
        let mut stage = Affine2::IDENTITY;
        let mut chain = vec![id];
        while let Some(pa) = self.nodes[*chain.last().unwrap()].parent {
            chain.push(pa);
        }
        for &n in chain.iter().rev() {
            stage = stage.mul(self.layer_ref(n).transform.matrix());
        }
        match &layer.kind {
            LayerKind::Vector { paths } => Some(crate::links::UnitRest {
                stage,
                paths: paths.iter().map(|p| p.anchors.iter().map(|a| a.pos).collect()).collect(),
                closed: paths.iter().map(|p| p.closed).collect(),
                tris: Vec::new(),
                is_image: false,
                has_mesh: true,
            }),
            LayerKind::Image { origin, size, mesh, .. } => {
                let data = self.images[id].as_ref()?;
                let size = size.unwrap_or(Vec2::new(data.pixmap.width() as f32, data.pixmap.height() as f32));
                let (verts, tris) = match mesh {
                    Some(m) => (m.rest_vertices(*origin, size), m.tri_indices()),
                    None => (Vec::new(), Vec::new()),
                };
                Some(crate::links::UnitRest { stage, paths: vec![verts], closed: vec![false], tris, is_image: true, has_mesh: mesh.is_some() })
            }
            _ => None,
        }
    }
    pub(crate) fn link_root_parallax(&self, id: usize) -> f32 {
        self.layer_ref(self.nodes[id].root).parallax
    }

    /// Enabled effects of node `id` with channel-driven values filled in.
    pub(super) fn node_effects(&self, id: usize, layer: &Layer, st: &State) -> Vec<crate::layerfx::LayerEffect> {
        let mut out = Vec::new();
        for (ei, e) in layer.effects.iter().enumerate() {
            if !e.enabled {
                continue;
            }
            let mut e = e.clone();
            if let Some(slots) = self.fx_slots.get(id).and_then(|s| s.get(ei)) {
                for &(pr, slot) in slots {
                    e.set(pr, st.pvals[slot]);
                }
            }
            out.push(e);
        }
        out
    }

    pub(super) fn layer_ref(&self, id: usize) -> &Layer {
        layer_at(&self.project, &self.nodes[id].idx_path)
    }

    pub(super) fn initial_state(&self) -> State {
        let p = &self.project;
        let layers = (0..self.nodes.len())
            .map(|id| {
                let l = self.layer_ref(id);
                LayerProps {
                    x: l.transform.position.x,
                    y: l.transform.position.y,
                    rot: l.transform.rotation,
                    sx: l.transform.scale.x,
                    sy: l.transform.scale.y,
                    skew: l.transform.skew,
                    opacity: l.opacity.clamp(0.0, 1.0),
                    switch: match &l.kind {
                        LayerKind::Switch { selected, .. } => *selected as f32,
                        _ => 0.0,
                    },
                    wobble: l.wobble.as_ref().map(|w| w.amp).unwrap_or(0.0),
                }
            })
            .collect();
        let centre = Vec2::new(p.width * 0.5, p.height * 0.5);
        let pos = p.camera.position.unwrap_or(centre);
        State {
            bones: vec![BonePose::default(); p.bones.len()],
            layers,
            cam: CamProps { x: pos.x, y: pos.y, zoom: p.camera.zoom.max(0.01), rot: p.camera.rotation, focus: p.camera.focus, blur: p.camera.blur.max(0.0) },
            ik_mix: p.ik.iter().map(|c| c.mix.clamp(0.0, 1.0)).collect(),
            cweight: p.constraints.iter().map(|c| c.weight.clamp(0.0, 1.0)).collect(),
            pvals: self.pslot_init.clone(),
        }
    }

    /// Current value of `target.prop` (used by expressions); 0 if unknown.
    pub(super) fn lookup(&self, st: &State, target: &str, prop: &str) -> f32 {
        let Some(prop) = Prop::parse(prop) else { return 0.0 };
        let slot = match prop.target_kind() {
            TargetKind::Bone => self.bone_ix.get(target).copied(),
            TargetKind::Layer => self.node_ix.get(target).copied(),
            TargetKind::Camera => Some(0),
            TargetKind::Ik => self.project.ik.iter().position(|c| c.name == target),
            TargetKind::Constraint => self.project.constraints.iter().position(|c| c.name == target),
            TargetKind::Sub => self.slot_ix.get(&(target.to_string(), prop)).copied(),
        };
        slot.map(|s| st.read(prop, s)).unwrap_or(0.0)
    }

    pub(super) fn apply_channels(&self, st: &mut State, frame: f32) {
        let p = &self.project;
        for c in &self.chans {
            let ch = &p.channels[c.chan];
            let v: Option<f32> = match &ch.source {
                Source::Keys { track } => track.eval(frame),
                Source::Expr { .. } => {
                    let get = |t: &str, pr: &str| self.lookup(st, t, pr);
                    let ctx = Ctx { frame, fps: p.fps, frames: p.frames as f32, get: &get };
                    c.expr.as_ref().map(|e| e.eval(&ctx))
                }
                Source::Noise { amp, freq, seed, octaves } => Some(amp * fbm1(frame / p.fps * freq, *seed, *octaves)),
                Source::Driven { curve, .. } => {
                    let (ip, is) = c.input.expect("resolved at compile time");
                    curve.eval(st.read(ip, is))
                }
            };
            let Some(v) = v else { continue };
            if !v.is_finite() {
                continue;
            }
            let cur = st.read(c.prop, c.slot);
            let w = ch.weight;
            let new = match ch.mode {
                Mode::Replace => cur + (v - cur) * w,
                Mode::Add => cur + v * w,
                Mode::Multiply => cur * (1.0 + (v - 1.0) * w),
            };
            st.write(c.prop, c.slot, new);
        }
    }

    pub(super) fn camera_matrix(&self, cam: &CamProps, parallax: f32) -> Affine2 {
        let p = &self.project;
        let centre = Vec2::new(p.width * 0.5, p.height * 0.5);
        let zoom = (1.0 + (cam.zoom - 1.0) * parallax).max(0.01);
        let pos = centre + (Vec2::new(cam.x, cam.y) - centre) * parallax;
        Affine2::translate(centre)
            .mul(Affine2::rotate(-cam.rot.to_radians() * parallax))
            .mul(Affine2::scale(zoom))
            .mul(Affine2::translate(-pos))
    }

    pub(super) fn clamp_limits(&self, poses: &mut [BonePose]) {
        for (b, p) in self.project.bones.iter().zip(poses.iter_mut()) {
            if let Some([lo, hi]) = b.angle_limits {
                p.angle = p.angle.clamp(lo.to_radians(), hi.to_radians());
            }
            if let Some([lo, hi]) = b.scale_limits {
                p.scale = p.scale.clamp(lo, hi);
            }
        }
    }

    /// Solve one IK chain, writing blended pose angles (and, for stretchy chains, the root
    /// bone's scale) into `poses`.
    pub(super) fn solve_ik(&self, chain: &IkResolved, mix: f32, poses: &mut [BonePose], world: &[Affine2]) {
        let bones = &self.project.bones;
        let raw_target = world[chain.target].apply(Vec2::ZERO);
        let root_head = world[chain.bones[0]].apply(Vec2::ZERO);
        let last = *chain.bones.last().unwrap();
        let mut joints: Vec<Vec2> = chain.bones.iter().map(|&b| world[b].apply(Vec2::ZERO)).collect();
        joints.push(world[last].apply(Vec2::new(bones[last].length, 0.0)));
        let n = chain.bones.len();
        let lengths: Vec<f32> = (0..n).map(|i| joints[i].distance(joints[i + 1])).collect();
        let reach: f32 = lengths.iter().sum();
        let to = raw_target - root_head;
        let dist = to.length();
        // stretch: lengthen the chain up to `stretch`×; otherwise soften the approach to full extension
        let mut scale = 1.0f32;
        let mut target = raw_target;
        if chain.stretch > 1.0 && chain.soft > 0.0 && reach > 1e-4 && dist > 1e-6 {
            // soft + stretch (wave 7; both were exclusive before and the chain popped from "held back" to "straight"
            // at full reach): the soft zone is the last `soft` fraction of the STRETCHED reach, so the chain lengthens
            // continuously and eases into its maximum length
            let eased = soft_distance(dist, reach * chain.stretch, chain.soft);
            scale = (eased / reach).max(1.0);
            target = root_head + to.normalized() * eased;
        } else if chain.stretch > 1.0 && reach > 1e-4 && dist > reach {
            scale = (dist / reach).min(chain.stretch);
        } else if chain.soft > 0.0 && reach > 1e-4 && dist > 1e-6 {
            target = root_head + to.normalized() * soft_distance(dist, reach, chain.soft);
        }
        let old: Vec<f32> = chain.bones.iter().map(|&b| poses[b].angle).collect();
        let new: Vec<f32> = if n == 2 {
            match solve_two_bone_scaled(bones, &self.parent, world, chain.bones[0], chain.bones[1], target, chain.bend, scale) {
                Some((a, b)) => vec![a, b],
                None => return,
            }
        } else {
            if lengths.iter().any(|l| *l < 1e-4) {
                return;
            }
            let scaled: Vec<f32> = lengths.iter().map(|l| l * scale).collect();
            solve_fabrik(&mut joints, &scaled, target, (chain.iterations as usize).max(crate::rig::FABRIK_MAX_ITERS), crate::rig::FABRIK_TOLERANCE);
            let root = chain.bones[0];
            let mut parent_rot = self.parent[root].map(|p| world[p].rotation()).unwrap_or(0.0);
            let mut out = Vec::with_capacity(n);
            for i in 0..n {
                let d = joints[i + 1] - joints[i];
                let wa = d.y.atan2(d.x);
                out.push(wa - parent_rot - bones[chain.bones[i]].rest_angle.to_radians());
                parent_rot = wa;
            }
            out
        };
        if scale > 1.0 {
            poses[chain.bones[0]].scale *= 1.0 + (scale - 1.0) * mix;
        }
        for (i, &b) in chain.bones.iter().enumerate() {
            poses[b].angle = old[i] + wrap_pi(new[i] - old[i]) * mix;
        }
    }

    pub(super) fn apply_constraint(&self, c: &ConstraintResolved, weight: f32, poses: &mut [BonePose], world: &[Affine2]) {
        let head = world[c.bone].apply(Vec2::ZERO);
        match c.kind {
            CKind::Aim { target, offset } => {
                let t = world[target].apply(Vec2::ZERO) - head;
                if t.length() < 1e-6 {
                    return;
                }
                let desired = t.y.atan2(t.x) + offset.to_radians();
                poses[c.bone].angle += wrap_pi(desired - world[c.bone].rotation()) * weight;
            }
            CKind::CopyRotation { source } => {
                let desired = world[source].rotation();
                poses[c.bone].angle += wrap_pi(desired - world[c.bone].rotation()) * weight;
            }
            CKind::Pin { target } => {
                let dw = (world[target].apply(Vec2::ZERO) - head) * weight;
                let local = match self.parent[c.bone] {
                    Some(p) => {
                        let inv = world[p].inverse().unwrap_or(Affine2::IDENTITY);
                        Vec2::new(inv.a * dw.x + inv.c * dw.y, inv.b * dw.x + inv.d * dw.y)
                    }
                    None => dw,
                };
                poses[c.bone].offset += local;
            }
        }
    }

    /// Channels → limits → IK → constraints. (Springs are applied afterwards.)
    pub(super) fn pose_pipeline(&self, frame: f32) -> (State, Vec<Affine2>) {
        let p = &self.project;
        let mut st = self.initial_state();
        self.apply_channels(&mut st, frame);
        self.clamp_limits(&mut st.bones);
        let mut world = fk(&p.bones, &self.parent, &st.bones);
        for (i, chain) in self.ik.iter().enumerate() {
            let mix = st.ik_mix[i];
            if mix <= 0.0 {
                continue;
            }
            self.solve_ik(chain, mix, &mut st.bones, &world);
            self.clamp_limits(&mut st.bones);
            world = fk(&p.bones, &self.parent, &st.bones);
        }
        for (i, c) in self.constraints.iter().enumerate() {
            let w = st.cweight[i];
            if w <= 0.0 {
                continue;
            }
            self.apply_constraint(c, w, &mut st.bones, &world);
            self.clamp_limits(&mut st.bones);
            world = fk(&p.bones, &self.parent, &st.bones);
        }
        (st, world)
    }

    pub(super) fn bake_springs(&mut self) {
        if self.springs.is_empty() {
            return;
        }
        let n = self.project.frames as usize;
        let dt = 1.0 / self.project.fps;
        let passes = if self.springs.iter().any(|s| s.warmup) { 2 } else { 1 };
        let mut sim = vec![SpringState::default(); self.springs.len()];
        let mut baked = vec![vec![0.0f32; self.springs.len()]; n];
        for pass in 0..passes {
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for f in 0..n {
                let (mut st, mut world) = self.pose_pipeline(f as f32);
                for (k, sp) in self.springs.iter().enumerate() {
                    let bi = sp.bone;
                    let target = world[bi].rotation();
                    if f == 0 && (pass == 0 || !sp.warmup) {
                        sim[k] = SpringState::at_rest(target);
                    }
                    sim[k].step(target, sp.frequency, sp.damping, dt, 8);
                    let parent_rot = self.parent[bi].map(|p| world[p].rotation()).unwrap_or(0.0);
                    st.bones[bi].angle = sim[k].x - parent_rot - self.project.bones[bi].rest_angle.to_radians();
                    world = fk(&self.project.bones, &self.parent, &st.bones);
                    if pass == passes - 1 {
                        baked[f][k] = st.bones[bi].angle;
                    }
                }
            }
        }
        self.baked = baked;
    }

    pub(super) fn baked_angle(&self, k: usize, frame: f32) -> f32 {
        let n = self.baked.len();
        let f = frame.clamp(0.0, (n - 1) as f32);
        let i = f.floor() as usize;
        let t = f - i as f32;
        let a = self.baked[i][k];
        if i + 1 >= n {
            return a;
        }
        a + (self.baked[i + 1][k] - a) * t
    }

    // ---- geometry ---------------------------------------------------------------------------

    pub(super) fn lattice_disp(rect: [f32; 4], cols: usize, rows: usize, offs: &[Vec2], p: Vec2) -> Vec2 {
        let u = ((p.x - rect[0]) / rect[2]).clamp(0.0, 1.0);
        let v = ((p.y - rect[1]) / rect[3]).clamp(0.0, 1.0);
        let mut d = Vec2::ZERO;
        for r in 0..=rows {
            let bv = bernstein(rows, r, v);
            for c in 0..=cols {
                d += offs[r * (cols + 1) + c] * (bernstein(cols, c, u) * bv);
            }
        }
        d
    }

    /// Bilinear per-vertex displacement (exact at the grid vertices).
    pub(super) fn cloth_disp(rect: [f32; 4], cols: usize, rows: usize, offs: &[Vec2], p: Vec2) -> Vec2 {
        let u = ((p.x - rect[0]) / rect[2]).clamp(0.0, 1.0) * cols as f32;
        let v = ((p.y - rect[1]) / rect[3]).clamp(0.0, 1.0) * rows as f32;
        let (c, r) = ((u.floor() as usize).min(cols - 1), (v.floor() as usize).min(rows - 1));
        let (fu, fv) = (u - c as f32, v - r as f32);
        let at = |cc: usize, rr: usize| offs[rr * (cols + 1) + cc];
        let top = at(c, r) * (1.0 - fu) + at(c + 1, r) * fu;
        let bot = at(c, r + 1) * (1.0 - fu) + at(c + 1, r + 1) * fu;
        top * (1.0 - fv) + bot * fv
    }

    pub(super) fn spine_disp(rest_pts: &[Vec2], rest_poly: &Polyline, offs: &[Vec2], radius: Option<f32>, p: Vec2) -> Vec2 {
        let def_pts: Vec<Vec2> = rest_pts.iter().zip(offs).map(|(&r, &o)| r + o).collect();
        let def_poly = VPath::smooth_through("", &def_pts, false, 1.0).polyline(0.5);
        let (rest_len, def_len) = (rest_poly.length(), def_poly.length());
        if rest_len < 1e-6 || def_len < 1e-6 {
            return Vec2::ZERO;
        }
        let (u, v) = rest_poly.project(p);
        let (q0, t0) = rest_poly.sample(u);
        let (q1, t1) = def_poly.sample(u * def_len / rest_len);
        let (n0, n1) = (Vec2::new(-t0.y, t0.x), Vec2::new(-t1.y, t1.x));
        let w = match radius {
            None => 1.0,
            Some(r) if r > 0.0 => {
                let x = ((v.abs() - 0.5 * r) / (0.5 * r)).clamp(0.0, 1.0);
                1.0 - x * x * (3.0 - 2.0 * x)
            }
            Some(_) => 0.0,
        };
        ((q1 - q0) + (n1 * v - n0 * v)) * w
    }

    /// Apply all deformers, then wobble, to a point (rest space).
    pub(super) fn deform_point(&self, id: usize, st: &State, tick: f32, p: Vec2, mesh_vertex: Option<usize>) -> Vec2 {
        let mut out = p;
        for (di, d) in self.geo[id].deformers.iter().enumerate() {
            let disp = match d {
                DefInfo::Lattice { cols, rows, rect, slots } => {
                    let offs: Vec<Vec2> = slots.iter().map(|s| Vec2::new(st.pvals[s[0]], st.pvals[s[1]])).collect();
                    Self::lattice_disp(*rect, *cols, *rows, &offs, out)
                }
                DefInfo::Cloth { cols, rows, rect, slots } => {
                    let offs: Vec<Vec2> = slots.iter().map(|s| Vec2::new(st.pvals[s[0]], st.pvals[s[1]])).collect();
                    Self::cloth_disp(*rect, *cols, *rows, &offs, out)
                }
                DefInfo::Spine { rest_pts, rest_poly, slots, radius } => {
                    let offs: Vec<Vec2> = slots.iter().map(|s| Vec2::new(st.pvals[s[0]], st.pvals[s[1]])).collect();
                    Self::spine_disp(rest_pts, rest_poly, &offs, *radius, out)
                }
                DefInfo::Pin { solver, slots } => {
                    let offs: Vec<Vec2> = slots.iter().map(|s| Vec2::new(st.pvals[s[0]], st.pvals[s[1]])).collect();
                    solver.displacement(&offs, out)
                }
                DefInfo::Cage { solver, slots, mesh_w } => {
                    let offs: Vec<Vec2> = slots.iter().map(|s| Vec2::new(st.pvals[s[0]], st.pvals[s[1]])).collect();
                    match mesh_vertex.filter(|_| di == 0).and_then(|vi| mesh_w.get(vi)) {
                        Some(w) => crate::cage::CageSolver::apply_weights(w, &offs),
                        None => solver.displacement(&offs, out),
                    }
                }
            };
            out += disp;
        }
        if let Some(w) = &self.layer_ref(id).wobble {
            let amp = st.layers[id].wobble;
            if amp != 0.0 {
                out += wobble_disp(out, tick, w, amp);
            }
        }
        out
    }

    pub(super) fn vector_geometry(&self, id: usize, st: &State, skin_mats: &[Affine2], tick: f32) -> Vec<VPath> {
        let LayerKind::Vector { paths } = &self.layer_ref(id).kind else { return Vec::new() };
        let has_deform = !self.geo[id].deformers.is_empty() || st.layers[id].wobble != 0.0;
        paths
            .iter()
            .enumerate()
            .map(|(pi, path)| {
                let mut o = path.clone();
                // morphs (additive blend shapes)
                for (mi, &slot) in self.geo[id].morph_slots[pi].iter().enumerate() {
                    let w = st.pvals[slot];
                    if w == 0.0 {
                        continue;
                    }
                    let m = &path.morphs[mi];
                    for (ai, a) in o.anchors.iter_mut().enumerate() {
                        let (b, t) = (&path.anchors[ai], &m.anchors[ai]);
                        let (hin, hout) = (a.hin() + (t.hin() - b.hin()) * w, a.hout() + (t.hout() - b.hout()) * w);
                        a.pos += (t.pos - b.pos) * w;
                        if b.handle_in.is_some() || t.handle_in.is_some() {
                            a.handle_in = Some(hin);
                        }
                        if b.handle_out.is_some() || t.handle_out.is_some() {
                            a.handle_out = Some(hout);
                        }
                    }
                }
                // per-anchor offsets
                for (ai, a) in o.anchors.iter_mut().enumerate() {
                    let s = self.geo[id].point_slots[pi][ai];
                    let d = Vec2::new(st.pvals[s[0]], st.pvals[s[1]]);
                    if d != Vec2::ZERO {
                        a.map_points(&mut |pt| pt + d);
                    }
                }
                // animated gradient handles
                if let Some(gs) = self.geo[id].grad_slots.get(pi) {
                    let d = |k: usize| gs[k].map(|s| Vec2::new(st.pvals[s[0]], st.pvals[s[1]])).unwrap_or(Vec2::ZERO);
                    if let Some(f) = &mut o.fill {
                        shift_gradient(f, d(0), d(1));
                    }
                    if let Some(sk) = &mut o.stroke {
                        shift_gradient(&mut sk.paint, d(2), d(3));
                    }
                }
                // deformers + wobble (anchor displacement is shared with its handles → smooth curves)
                if has_deform {
                    for a in o.anchors.iter_mut() {
                        let d = self.deform_point(id, st, tick, a.pos, None) - a.pos;
                        a.map_points(&mut |pt| pt + d);
                    }
                }
                // skinning
                for (ai, a) in o.anchors.iter_mut().enumerate() {
                    let ws = &self.skin[id][pi][ai];
                    if ws.is_empty() {
                        continue;
                    }
                    a.map_points(&mut |pt| {
                        let mut acc = Vec2::ZERO;
                        for &(bi, w) in ws {
                            acc += skin_mats[bi].apply(pt) * w;
                        }
                        acc
                    });
                }
                o
            })
            .collect()
    }

}
