// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `engine` (see the module docs in `mod.rs`).
use super::*;

impl Engine {
    pub(super) fn image_draw(&self, id: usize, st: &State, skin_mats: &[Affine2], tick: f32) -> Option<ImageDraw> {
        let data = self.images[id].clone()?;
        let LayerKind::Image { origin, size, mesh, .. } = &self.layer_ref(id).kind else { return None };
        let (pw, ph) = (data.pixmap.width() as f32, data.pixmap.height() as f32);
        let size = size.unwrap_or(Vec2::new(pw, ph));
        let has_deform = !self.geo[id].deformers.is_empty() || st.layers[id].wobble != 0.0;
        let mesh_out = if mesh.is_some() || has_deform {
            let free = mesh.as_ref().and_then(|m| m.free.as_ref());
            let (cols, rows) = match mesh {
                Some(m) => (m.cols.max(1), m.rows.max(1)),
                None => (16, 16),
            };
            let rest = match free {
                Some(f) => f.vertices.clone(),
                None => crate::project::mesh_vertices(*origin, size, cols, rows),
            };
            let uvs: Vec<Vec2> = match free {
                Some(f) if f.uvs.len() == f.vertices.len() => f.uvs.clone(),
                _ => rest.iter().map(|v| Vec2::new((v.x - origin.x) / size.x * pw, (v.y - origin.y) / size.y * ph)).collect(),
            };
            let positions: Vec<Vec2> = rest
                .iter()
                .enumerate()
                .map(|(vi, &v)| {
                    let mut p = self.deform_point(id, st, tick, v, Some(vi));
                    if let Some(ws) = self.mesh_skin[id].get(vi).filter(|w| !w.is_empty()) {
                        let mut acc = Vec2::ZERO;
                        for &(bi, w) in ws {
                            acc += skin_mats[bi].apply(p) * w;
                        }
                        p = acc;
                    }
                    p
                })
                .collect();
            let tris = match free {
                Some(f) => f.triangles.clone(),
                None => {
                    let mut tris = Vec::with_capacity(cols * rows * 2);
                    for r in 0..rows {
                        for c in 0..cols {
                            let i = (r * (cols + 1) + c) as u32;
                            let j = i + 1;
                            let k = i + (cols + 1) as u32;
                            let l = k + 1;
                            tris.push([i, j, k]);
                            tris.push([j, l, k]);
                        }
                    }
                    tris
                }
            };
            Some(MeshOut { positions, uvs, tris })
        } else {
            None
        };
        Some(ImageDraw { data, origin: *origin, size, mesh: mesh_out })
    }

    /// Pose, solve IK, skin and return everything drawable at `frame`.
    /// Depth-of-field `(focus, blur strength)` at `frame`, after camera channels (`camera_focus`,
    /// `camera_blur`) have been applied.
    pub fn dof_at(&self, frame: f32) -> (f32, f32) {
        let (st, _) = self.pose_pipeline(frame);
        (st.cam.focus, st.cam.blur)
    }

    /// Weight of every path morph at `frame`, indexed `[node id][path index][morph index]`
    /// (empty for layers that are not vector layers). Used by exporters that keep morphs as targets.
    pub fn morph_weights(&self, frame: f32) -> Vec<Vec<Vec<f32>>> {
        let (st, _) = self.pose_pipeline(frame);
        self.geo.iter().map(|g| g.morph_slots.iter().map(|ms| ms.iter().map(|&s| st.pvals[s]).collect()).collect()).collect()
    }

    pub fn evaluate(&self, frame: f32) -> Frame {
        let p = &self.project;
        let (mut st, mut world) = self.pose_pipeline(frame);
        for (id, o) in self.layout.iter().enumerate() {
            if let Some([dx, dy, kx, ky]) = o {
                let lp = &mut st.layers[id];
                lp.x += dx;
                lp.y += dy;
                lp.sx *= kx;
                lp.sy *= ky;
            }
        }

        if !self.springs.is_empty() {
            for (k, sp) in self.springs.iter().enumerate() {
                st.bones[sp.bone].angle = self.baked_angle(k, frame);
            }
            world = fk(&p.bones, &self.parent, &st.bones);
        }
        let skin_mats: Vec<Affine2> = world.iter().zip(&self.inv_rest).map(|(w, inv)| w.mul(*inv)).collect();

        let cams: HashMap<usize, Affine2> =
            self.roots.iter().map(|&r| (r, self.camera_matrix(&st.cam, self.layer_ref(r).parallax))).collect();

        let mut stage: Vec<Affine2> = Vec::with_capacity(self.nodes.len());
        let mut eff_attach: Vec<Affine2> = Vec::with_capacity(self.nodes.len());
        let mut chain_no_bone: Vec<Affine2> = Vec::with_capacity(self.nodes.len());
        let mut out: Vec<RenderNode> = Vec::with_capacity(self.nodes.len());
        for (id, info) in self.nodes.iter().enumerate() {
            let layer = self.layer_ref(id);
            let lp = &st.layers[id];
            let local =
                compose_transform(Vec2::new(lp.x, lp.y), lp.rot, Vec2::new(lp.sx, lp.sy), lp.skew, layer.transform.anchor);
            // The skin matrix of a bone is a STAGE-space motion (bone world * inverse rest), so it is applied after the
            // layer's whole parent chain: `world = skin * chain * local`. Applying it inside a scaled parent group (the
            // old `parent_stage * skin * local`) scaled the posed travel by the group's scale (L-71). A layer without a
            // bone of its own follows the bone of its nearest bone-attached ancestor; its own bone replaces that of an
            // ancestor (the two motions are not stacked).
            let eff = info.bone.map(|b| skin_mats[b]).or_else(|| info.parent.map(|pa| eff_attach[pa])).unwrap_or(Affine2::IDENTITY);
            let chain = match info.parent {
                Some(pa) => chain_no_bone[pa].mul(local),
                None => local,
            };
            eff_attach.push(eff);
            chain_no_bone.push(chain);
            let m = eff.mul(chain);
            stage.push(m);
            let screen = cams[&info.root].mul(m);
            let layer_tick = match &layer.wobble {
                Some(w) => (frame / p.fps * w.rate).floor(),
                None => 0.0,
            };

            let mut image = None;
            let (kind, paths, children) = match &layer.kind {
                LayerKind::Vector { .. } => {
                    (NodeKind::Vector, self.vector_geometry(id, &st, &skin_mats, layer_tick), Vec::new())
                }
                LayerKind::Group { .. } => (NodeKind::Group, Vec::new(), info.children.clone()),
                LayerKind::Switch { .. } => {
                    let n = info.children.len();
                    let sel = if n == 0 { None } else { Some((lp.switch.round().max(0.0) as usize).min(n - 1)) };
                    (NodeKind::Switch, Vec::new(), sel.map(|s| vec![info.children[s]]).unwrap_or_default())
                }
                LayerKind::Image { .. } => {
                    image = self.image_draw(id, &st, &skin_mats, layer_tick);
                    (NodeKind::Image, Vec::new(), Vec::new())
                }
                LayerKind::Liquid { blobs, threshold, cell, fill, stroke } => {
                    let pts: Vec<(Vec2, f32)> = blobs
                        .iter()
                        .map(|b| match b.bone.as_ref().and_then(|n| self.bone_ix.get(n)) {
                            Some(&bi) => (world[bi].apply(b.pos), b.radius * world[bi].x_scale().abs()),
                            None => (b.pos, b.radius),
                        })
                        .collect();
                    (NodeKind::Vector, crate::fx::liquid_paths(&pts, *threshold, *cell, fill, stroke), Vec::new())
                }
                LayerKind::Boolean { op, paths } => {
                    (NodeKind::Vector, crate::fx::boolean_paths(paths, *op, 0.25), Vec::new())
                }
                LayerKind::Emitter { spec } => {
                    let origin = match spec.bone.as_ref().and_then(|n| self.bone_ix.get(n)) {
                        Some(&bi) => world[bi].apply(spec.pos),
                        None => spec.pos,
                    };
                    let ps = spec.particles(frame / p.fps, origin);
                    (NodeKind::Vector, crate::fx::particle_paths(&ps), Vec::new())
                }
            };

            out.push(RenderNode {
                id,
                name: info.path.clone(),
                kind,
                world: screen,
                opacity: lp.opacity,
                blend: layer.blend,
                visible: layer.visible && (!layer.reference || self.show_reference),
                mask: info.mask,
                paths,
                image,
                children,
                effects: self.node_effects(id, layer, &st),
            });
        }

        if let Some(l) = &self.links {
            l.apply(&mut out, &stage);
        }
        let camera = self.camera_matrix(&st.cam, 1.0);
        let mut f = Frame { world, camera, nodes: out, roots: self.roots.clone(), nonfinite: None };
        self.repair_nonfinite(&mut f);
        f
    }

    /// Hide nodes with non-finite numbers and reset non-finite bone transforms, remembering the first place found.
    pub(super) fn repair_nonfinite(&self, f: &mut Frame) {
        let mut first: Option<String> = None;
        for (i, w) in f.world.iter_mut().enumerate() {
            if !aff_finite(w) {
                first.get_or_insert_with(|| format!("bone '{}' (transform)", self.project.bones.get(i).map(|b| b.name.as_str()).unwrap_or("?")));
                *w = Affine2::IDENTITY;
            }
        }
        if !aff_finite(&f.camera) {
            first.get_or_insert_with(|| "the camera".to_string());
            f.camera = Affine2::IDENTITY;
        }
        for n in f.nodes.iter_mut() {
            let bad_geometry = n.paths.iter().any(|p| p.anchors.iter().any(|a| !(a.pos.x.is_finite() && a.pos.y.is_finite())));
            let what = if !aff_finite(&n.world) {
                Some("transform")
            } else if !n.opacity.is_finite() {
                Some("opacity")
            } else if bad_geometry {
                Some("geometry")
            } else {
                None
            };
            if let Some(w) = what {
                first.get_or_insert_with(|| format!("layer '{}' ({w})", n.name));
                n.world = Affine2::IDENTITY;
                n.opacity = 0.0;
                n.visible = false;
                n.paths.clear();
                n.image = None;
            }
        }
        f.nonfinite = first;
    }

    /// Evaluate the frame and refuse it if a number overflowed (see [`Frame::nonfinite`]).
    pub fn evaluate_checked(&self, frame: f32) -> Result<Frame, String> {
        let f = self.evaluate(frame);
        f.check_finite().map_err(|e| format!("{e} at frame {frame}"))?;
        Ok(f)
    }

    /// Evaluate every frame and report the first one with a non-finite number. Exporters call this before
    /// writing anything.
    pub fn check_all_frames_finite(&self) -> Result<(), String> {
        for fr in 0..self.project.frames {
            self.evaluate_checked(fr as f32)?;
        }
        Ok(())
    }
}
