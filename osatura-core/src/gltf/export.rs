// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `gltf` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// export
// ---------------------------------------------------------------------------------------------

pub(super) enum PaintSrc {
    Paint(Paint),
    Image { png: Vec<u8>, key: String, has_alpha: bool },
}

pub(super) enum UvSrc {
    /// computed from the paint (gradients) or absent (solid)
    Paint,
    /// affine over a rectangle in layer space
    Quad { origin: Vec2, size: Vec2 },
    Explicit(Vec<[f32; 2]>),
}

pub(super) struct Prim {
    pub(super) name: String,
    pub(super) pos: Vec<Vec2>, // layer space
    pub(super) tris: Vec<[u32; 3]>,
    pub(super) skin: Option<Vec<Mix>>,
    pub(super) paint: PaintSrc,
    pub(super) uv: UvSrc,
    pub(super) opacity: f32,
    pub(super) morph: Option<MorphData>,
}

pub(super) struct Out {
    pub(super) bin: Bin,
    pub(super) nodes: Vec<Value>,
    pub(super) scene_nodes: Vec<usize>,
    pub(super) meshes: Vec<Value>,
    pub(super) materials: Vec<Value>,
    pub(super) textures: Vec<Value>,
    pub(super) images: Vec<Value>,
    pub(super) tex_cache: HashMap<String, usize>,
    pub(super) shape_count: usize,
    pub(super) skinned_any: bool,
    pub(super) morph_anims: Vec<(usize, Vec<f32>, usize)>, // node, frame-major weights, targets
}

pub(super) fn bounds3(flat: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let (mut mn, mut mx) = (vec![f32::MAX; 3], vec![f32::MIN; 3]);
    for v in flat.chunks_exact(3) {
        for k in 0..3 {
            mn[k] = mn[k].min(v[k]);
            mx[k] = mx[k].max(v[k]);
        }
    }
    (mn, mx)
}

impl Out {
    pub(super) fn texture(&mut self, key: String, png: impl FnOnce() -> Vec<u8>) -> usize {
        if let Some(&t) = self.tex_cache.get(&key) {
            return t;
        }
        let bytes = png();
        let bv = self.bin.view(&bytes);
        self.images.push(json!({"bufferView": bv, "mimeType": "image/png"}));
        self.textures.push(json!({"sampler": 0, "source": self.images.len() - 1}));
        self.tex_cache.insert(key, self.textures.len() - 1);
        self.textures.len() - 1
    }

    /// Write one primitive as a mesh node. `place` maps layer space to the node's space.
    pub(super) fn emit(&mut self, pr: Prim, to_stage: Affine2, to_host: Affine2, host: Option<usize>) {
        let skinned = pr.skin.is_some();
        let place = if skinned { to_stage } else { to_host };
        let z = if skinned { self.shape_count as f32 * 0.01 } else { 0.0 };
        let gp = |q: Vec2| {
            let p = place.apply(q);
            [p.x, -p.y, z]
        };
        let flat: Vec<f32> = pr.pos.iter().flat_map(|q| gp(*q)).collect();
        let bounds = bounds3(&flat);
        let nv = pr.pos.len();
        let pos = self.bin.add(&f32s(&flat), 5126, nv, "VEC3", Some(bounds), Some(34962));
        let mut attrs = json!({"POSITION": pos});
        if let Some(sk) = &pr.skin {
            self.skinned_any = true;
            let joints: Vec<u8> = sk.iter().flat_map(|m| m.0).collect();
            let wts: Vec<f32> = sk.iter().flat_map(|m| m.1).collect();
            attrs["JOINTS_0"] = json!(self.bin.add(&joints, 5121, nv, "VEC4", None, Some(34962)));
            attrs["WEIGHTS_0"] = json!(self.bin.add(&f32s(&wts), 5126, nv, "VEC4", None, Some(34962)));
        }
        // material + texture coordinates
        let (mut gmap, mut tex): (Option<GradMap>, Option<usize>) = (None, None);
        let mut translucent = pr.opacity < 0.999;
        let mut factor = [1.0, 1.0, 1.0, pr.opacity];
        match &pr.paint {
            PaintSrc::Paint(p) => {
                gmap = grad_map(p);
                if gmap.is_some() {
                    tex = Some(self.texture(format!("grad:{}", serde_json::to_string(p).unwrap_or_default()), || gradient_png(p)));
                    translucent |= stops_translucent(p);
                } else {
                    let c = p.degenerate_color().unwrap_or_else(|| p.base_color());
                    factor = [srgb_to_linear(c.r), srgb_to_linear(c.g), srgb_to_linear(c.b), c.a * pr.opacity];
                    translucent = factor[3] < 0.999;
                }
            }
            PaintSrc::Image { png, key, has_alpha } => {
                tex = Some(self.texture(key.clone(), || png.clone()));
                translucent |= *has_alpha;
            }
        }
        let uv_of = |q: Vec2| -> [f32; 2] {
            match (&pr.uv, &gmap) {
                (UvSrc::Quad { origin, size }, _) => [(q.x - origin.x) / size.x, (q.y - origin.y) / size.y],
                (_, Some(m)) => grad_uv(m, q),
                _ => [0.0, 0.0],
            }
        };
        let has_uv = tex.is_some();
        if has_uv {
            let uvs: Vec<f32> = match &pr.uv {
                UvSrc::Explicit(v) => v.iter().flatten().copied().collect(),
                _ => pr.pos.iter().flat_map(|q| uv_of(*q)).collect(),
            };
            attrs["TEXCOORD_0"] = json!(self.bin.add(&f32s(&uvs), 5126, nv, "VEC2", None, Some(34962)));
        }
        let idx_bytes: Vec<u8> = pr.tris.iter().flat_map(|t| t.iter().rev()).flat_map(|&i| i.to_le_bytes()).collect(); // reverse: y flip flips winding
        let ind = self.bin.add(&idx_bytes, 5125, pr.tris.len() * 3, "SCALAR", None, Some(34963));
        let mut mat = json!({"pbrMetallicRoughness": {"baseColorFactor": factor, "metallicFactor": 0.0, "roughnessFactor": 1.0}, "doubleSided": true});
        if let Some(t) = tex {
            mat["pbrMetallicRoughness"]["baseColorTexture"] = json!({"index": t, "texCoord": 0});
        }
        if translucent {
            mat["alphaMode"] = json!("BLEND");
        }
        self.materials.push(mat);
        let mut prim = json!({"attributes": attrs, "indices": ind, "material": self.materials.len() - 1});
        let mut mesh = json!({"name": pr.name});
        let nt = pr.morph.as_ref().map_or(0, |m| m.names.len());
        if let Some(md) = &pr.morph {
            let mut targets = Vec::new();
            for d in &md.deltas {
                let dflat: Vec<f32> = pr.pos.iter().zip(d).flat_map(|(b, dd)| {
                    let (a, c) = (gp(*b), gp(*b + *dd));
                    [c[0] - a[0], c[1] - a[1], 0.0]
                }).collect();
                let mut t = json!({"POSITION": self.bin.add(&f32s(&dflat), 5126, nv, "VEC3", Some(bounds3(&dflat)), Some(34962))});
                if has_uv && gmap.is_some() {
                    let duv: Vec<f32> = pr.pos.iter().zip(d).flat_map(|(b, dd)| {
                        let (a, c) = (uv_of(*b), uv_of(*b + *dd));
                        [c[0] - a[0], c[1] - a[1]]
                    }).collect();
                    t["TEXCOORD_0"] = json!(self.bin.add(&f32s(&duv), 5126, nv, "VEC2", None, Some(34962)));
                }
                targets.push(t);
            }
            prim["targets"] = json!(targets);
            mesh["weights"] = json!(md.w0);
            mesh["extras"] = json!({"targetNames": md.names});
        }
        mesh["primitives"] = json!([prim]);
        self.meshes.push(mesh);
        let mi = self.meshes.len() - 1;
        if skinned {
            self.nodes.push(json!({"name": format!("skinned {}", pr.name), "mesh": mi, "skin": 0}));
            self.shape_count += 1;
            self.scene_nodes.push(self.nodes.len() - 1);
            return;
        }
        self.nodes.push(json!({"name": format!("shape {}", pr.name), "mesh": mi, "translation": [0.0, 0.0, self.shape_count as f32 * 0.01]}));
        self.shape_count += 1;
        let ni = self.nodes.len() - 1;
        if let Some(md) = pr.morph {
            self.morph_anims.push((ni, md.frames, nt));
        }
        match host {
            Some(h) => match self.nodes[h].get_mut("children").and_then(Value::as_array_mut) {
                Some(a) => a.push(json!(ni)),
                None => self.nodes[h]["children"] = json!([ni]),
            },
            None => self.scene_nodes.push(ni),
        }
    }
}

pub(super) fn note(notes: &mut Vec<String>, s: String) {
    if !notes.contains(&s) {
        notes.push(s);
    }
}

/// Things glTF cannot carry (or this exporter does not write) that the project uses: reported so a silent loss never looks like success.
pub(super) fn dropped_features(p: &crate::project::Project, notes: &mut Vec<String>) {
    use crate::project::Prop;
    fn walk(ls: &[Layer], prefix: &str, notes: &mut Vec<String>) {
        for l in ls {
            let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
            if l.blend != crate::style::Blend::Normal {
                note(notes, format!("blend mode {:?} of '{path}' is not exported (glTF materials only alpha-blend); it is drawn as normal", l.blend));
            }
            if !l.effects.is_empty() {
                note(notes, format!("layer effects of '{path}' are not exported"));
            }
            if !l.deformers.is_empty() {
                note(notes, format!("deformers of '{path}' are exported only at their frame-0 shape (the deformation is not animated in the file)"));
            }
            walk(l.children(), &path, notes);
        }
    }
    walk(&p.layers, "", notes);
    for c in &p.channels {
        let (what, msg) = match c.prop {
            Prop::LayerOpacity => ("opacity", "animated opacity of '{}' is not exported (frozen at frame 0)"),
            Prop::LatticeDx | Prop::LatticeDy | Prop::SpineDx | Prop::SpineDy | Prop::PinDx | Prop::PinDy | Prop::CageDx | Prop::CageDy => ("deformer", "animated deformer '{}' is not exported (frozen at frame 0)"),
            Prop::PointDx | Prop::PointDy => ("point", "animated path point '{}' is not exported (frozen at frame 0; only morphs become morph targets)"),
            Prop::LayerX | Prop::LayerY | Prop::LayerRotation | Prop::LayerScaleX | Prop::LayerScaleY | Prop::LayerSkew | Prop::LayerSwitch | Prop::LayerWobble => ("layer", "animated layer transform/switch/wobble of '{}' is not exported (only bone animation is)"),
            Prop::FxBlur | Prop::FxDx | Prop::FxDy | Prop::FxSoft | Prop::FxAmount => ("effect", "animated layer effect '{}' is not exported"),
            Prop::GradDx | Prop::GradDy => ("gradient", "animated gradient handle '{}' is not exported (frozen at frame 0)"),
            _ => continue,
        };
        let _ = what;
        note(notes, msg.replace("{}", &c.target));
    }
}

pub(super) fn dedup_flat(mut tp: Vec<(Vec2, usize, f32)>, closed_hint: bool) -> Vec<(Vec2, usize, f32)> {
    tp.dedup_by(|a, b| a.0.distance(b.0) < 1e-4);
    if tp.len() >= 2 && tp[0].0.distance(tp.last().unwrap().0) < 1e-4 && closed_hint {
        tp.pop();
    }
    tp
}

pub(super) fn to_u32(t: Vec<[usize; 3]>) -> Vec<[u32; 3]> {
    t.into_iter().map(|t| [t[0] as u32, t[1] as u32, t[2] as u32]).collect()
}

/// Build the .glb bytes.
pub fn export_glb(engine: &Engine) -> Result<Vec<u8>, String> {
    export_glb_report(engine).map(|r| r.0)
}

/// Build the .glb bytes and a list of things that could not be exported exactly.
pub fn export_glb_report(engine: &Engine) -> Result<(Vec<u8>, Vec<String>), String> {
    let p = &engine.project;
    let nb = p.bones.len();
    if nb == 0 {
        return Err("the project has no bones to export".into());
    }
    if nb * p.frames as usize * 10 > MAX_ANIM_VALUES {
        return Err("animation too large for glTF export".into());
    }
    let mut notes: Vec<String> = Vec::new();
    dropped_features(p, &mut notes);
    let parent: Vec<Option<usize>> = p.bones.iter().map(|b| b.parent.as_ref().and_then(|n| p.bones.iter().position(|x| &x.name == n))).collect();
    let bone_ix: HashMap<&str, usize> = p.bones.iter().enumerate().map(|(i, b)| (b.name.as_str(), i)).collect();
    let f0 = engine.evaluate(0.0);
    let mut out = Out {
        bin: Bin { data: Vec::new(), views: vec![], accessors: vec![] },
        nodes: Vec::new(),
        scene_nodes: Vec::new(),
        meshes: Vec::new(),
        materials: Vec::new(),
        textures: Vec::new(),
        images: Vec::new(),
        tex_cache: HashMap::new(),
        shape_count: 0,
        skinned_any: false,
        morph_anims: Vec::new(),
    };
    let mut children: Vec<Vec<usize>> = vec![vec![]; nb];
    let mut roots = Vec::new();
    for (i, pa) in parent.iter().enumerate() {
        match pa {
            Some(q) => children[*q].push(i),
            None => roots.push(i),
        }
    }
    for i in 0..nb {
        let (t, r, s) = trs(f0.world[i], parent[i].map(|q| f0.world[q]));
        let mut n = json!({"name": p.bones[i].name, "translation": t, "rotation": r, "scale": s});
        if !children[i].is_empty() {
            n["children"] = json!(children[i]);
        }
        out.nodes.push(n);
    }
    out.scene_nodes = roots.clone();

    // shapes (frame 0)
    let mut shapes = Vec::new();
    collect_shapes(&p.layers, "", &mut shapes);
    let node_by_name: HashMap<&str, usize> = f0.nodes.iter().map(|n| (n.name.as_str(), n.id)).collect();
    let cam_inv = f0.camera.inverse().unwrap_or(Affine2::IDENTITY);
    let mut morph_cache: MorphCache = None;
    for (layer, lpath) in shapes {
        let Some(&nid) = node_by_name.get(lpath.as_str()) else { continue };
        let node = &f0.nodes[nid];
        if node.kind == NodeKind::Switch || node.kind == NodeKind::Group {
            continue;
        }
        // the node and all its ancestors: visibility, selected switch child, opacity, masks
        let mut chain = vec![nid];
        let mut name = lpath.as_str();
        while let Some((pre, _)) = name.rsplit_once('/') {
            if let Some(&a) = node_by_name.get(pre) {
                chain.push(a);
            }
            name = pre;
        }
        let mut drawn = true;
        let mut opacity = 1.0f32;
        let mut mask_refs: Vec<(usize, bool)> = Vec::new();
        for (k, &id) in chain.iter().enumerate() {
            let n = &f0.nodes[id];
            drawn &= n.visible;
            opacity *= n.opacity;
            if k > 0 && n.kind == NodeKind::Switch && !n.children.contains(&chain[k - 1]) {
                drawn = false;
            }
            if let Some(m) = n.mask {
                mask_refs.push(m);
            }
        }
        if !drawn {
            continue;
        }
        let to_stage = cam_inv.mul(node.world);
        let host = layer.bone.as_ref().and_then(|b| bone_ix.get(b.as_str()).copied());
        let to_host = match host.and_then(|h| f0.world[h].inverse()) {
            Some(inv) => inv.mul(to_stage),
            None => to_stage,
        };

        // masks -> triangle lists in this layer's space
        let mut masks: Vec<(Vec<[Vec2; 3]>, bool)> = Vec::new();
        for (mid, invert) in mask_refs {
            let m = &f0.nodes[mid];
            let mut mop = m.opacity;
            let mut mname = m.name.as_str();
            while let Some((pre, _)) = mname.rsplit_once('/') {
                if let Some(&a) = node_by_name.get(pre) {
                    mop *= f0.nodes[a].opacity;
                }
                mname = pre;
            }
            let opaque = |pt: &Paint| match pt {
                Paint::Solid(c) => c.a >= 0.999,
                Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => stops.iter().all(|s| s.color.a >= 0.999),
            };
            let eligible = m.kind == NodeKind::Vector && mop >= 0.999 && m.paths.iter().all(|pa| pa.stroke.is_none() && pa.fill.as_ref().is_none_or(opaque));
            let Some(inv) = node.world.inverse().filter(|_| eligible) else {
                note(&mut notes, format!("mask '{}' on '{}' was not baked (mask source must be a vector layer with opaque fills, no strokes and full opacity); the layer is exported unmasked", m.name, lpath));
                continue;
            };
            let mm = inv.mul(m.world);
            let mut tris = Vec::new();
            for pa in m.paths.iter().filter(|pa| pa.fill.is_some()) {
                let pts: Vec<Vec2> = dedup_flat(pa.flatten_t(0.25), true).iter().map(|x| mm.apply(x.0)).collect();
                for t in triangulate(&pts) {
                    let mut tr = [pts[t[0]], pts[t[1]], pts[t[2]]];
                    if cr(tr[1] - tr[0], tr[2] - tr[0]) < 0.0 {
                        tr.swap(1, 2);
                    }
                    tris.push(tr);
                }
            }
            masks.push((tris, invert));
        }
        if !masks.is_empty() {
            note(&mut notes, format!("'{}' has layer masks: baked at frame 0 and exported rigid (not skinned, not morphed)", lpath));
        }

        match &layer.kind {
            LayerKind::Image { mesh: spec, .. } => {
                let Some(img) = &node.image else { continue };
                let (pw, ph) = (img.data.pixmap.width() as f32, img.data.pixmap.height() as f32);
                let (mut pos, mut tris, mut uv, mut skin) = match &img.mesh {
                    Some(m) => {
                        let uv: Vec<[f32; 2]> = m.uvs.iter().map(|u| [u.x / pw, u.y / ph]).collect();
                        let skin = spec.as_ref().filter(|s| nb <= 255 && s.weights.len() == m.positions.len() && s.weights.iter().all(|w| !w.is_empty())).map(|s| {
                            s.weights.iter().map(|ws| {
                                let mut mix = Vec::new();
                                for bw in ws {
                                    if let Some(&bi) = bone_ix.get(bw.bone.as_str()) {
                                        add_mix(&mut mix, bi, bw.weight);
                                    }
                                }
                                pack_mix(mix)
                            }).collect::<Vec<Mix>>()
                        });
                        (m.positions.clone(), m.tris.clone(), UvSrc::Explicit(uv), skin)
                    }
                    None => {
                        let (o, s) = (img.origin, img.size);
                        let pos = vec![o, o + Vec2::new(s.x, 0.0), o + s, o + Vec2::new(0.0, s.y)];
                        (pos, vec![[0, 1, 2], [0, 2, 3]], UvSrc::Quad { origin: o, size: s }, None)
                    }
                };
                if !masks.is_empty() {
                    if matches!(uv, UvSrc::Quad { .. }) {
                        match clip_mesh(&pos, &tris, &masks, 100_000) {
                            Some((p2, t2)) => {
                                pos = p2;
                                tris = t2;
                                skin = None;
                            }
                            None => note(&mut notes, format!("mask on '{lpath}' produced too many pieces; exported unmasked")),
                        }
                    } else {
                        note(&mut notes, format!("mask on meshed image '{lpath}' was not baked; exported unmasked"));
                    }
                }
                if tris.is_empty() {
                    continue;
                }
                if matches!(uv, UvSrc::Explicit(_)) {
                    if let UvSrc::Explicit(v) = &uv {
                        if v.len() != pos.len() {
                            continue;
                        }
                    }
                } else if skin.is_some() {
                    skin = None;
                }
                let key = {
                    use std::hash::{Hash, Hasher};
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    img.data.png.hash(&mut h);
                    format!("img:{}:{:x}", img.data.png.len(), h.finish())
                };
                let has_alpha = img.data.pixmap.data().chunks_exact(4).any(|px| px[3] < 255);
                if let UvSrc::Quad { .. } = uv {
                    uv = UvSrc::Quad { origin: img.origin, size: img.size };
                }
                out.emit(Prim { name: node.name.clone(), pos, tris, skin, paint: PaintSrc::Image { png: img.data.png.clone(), key, has_alpha }, uv, opacity, morph: None }, to_stage, to_host, host);
            }
            _ => {
                let src_paths: &[VPath] = match &layer.kind {
                    LayerKind::Vector { paths } => paths,
                    _ => &[],
                };
                for (pi, path) in node.paths.iter().enumerate() {
                    let wpath = src_paths.get(pi).filter(|s| s.anchors.len() == path.anchors.len());
                    let skinned_ok = masks.is_empty() && nb <= 255 && !path.anchors.is_empty() && wpath.is_some_and(|s| s.anchors.iter().all(|a| !a.weights.is_empty()));
                    let pname = if path.name.is_empty() { format!("path{pi}") } else { path.name.clone() };
                    // ---- fill
                    if let Some(fill) = &path.fill {
                        let mut tp = dedup_flat(path.flatten_t(1.0), true);
                        let mut pts: Vec<Vec2> = tp.iter().map(|x| x.0).collect();
                        let mut tris = to_u32(triangulate(&pts));
                        let mut skin: Option<Vec<Mix>> = None;
                        let mut morph = None;
                        if !tris.is_empty() {
                            if skinned_ok {
                                skin = Some(skin_table(wpath.expect("skinned implies weights"), &tp, &bone_ix));
                            } else if masks.is_empty() {
                                if let Some(sp) = wpath.filter(|s| !s.morphs.is_empty() && s.fill.is_some()) {
                                    match try_morph(engine, &mut morph_cache, nid, pi, sp) {
                                        Ok((base, t, md)) => {
                                            pts = base;
                                            tris = t;
                                            morph = Some(md);
                                        }
                                        Err(e) => note(&mut notes, format!("morphs of '{lpath}' path '{pname}' not exported as targets: {e}")),
                                    }
                                }
                            }
                            if !masks.is_empty() {
                                match clip_mesh(&pts, &tris, &masks, 200_000) {
                                    Some((p2, t2)) => {
                                        pts = p2;
                                        tris = t2;
                                    }
                                    None => note(&mut notes, format!("mask on '{lpath}' produced too many pieces; exported unmasked")),
                                }
                            }
                            tp.clear();
                            if !tris.is_empty() {
                                out.emit(Prim { name: format!("{}:{}", node.name, pname), pos: pts, tris, skin, paint: PaintSrc::Paint(fill.clone()), uv: UvSrc::Paint, opacity, morph }, to_stage, to_host, host);
                            }
                        }
                    }
                    // ---- stroke
                    if let Some(st) = &path.stroke {
                        let tp = dedup_flat(path.flatten_t(0.25), path.closed);
                        let pts: Vec<Vec2> = tp.iter().map(|x| x.0).collect();
                        let Some(mut soup) = build_stroke(&pts, path.closed, st) else { continue };
                        if let Some(n) = soup.note {
                            note(&mut notes, format!("'{lpath}': {n}"));
                        }
                        if !path.morphs.is_empty() || wpath.is_some_and(|s| !s.morphs.is_empty()) {
                            note(&mut notes, format!("stroke of morphing path '{lpath}' path '{pname}' is exported as its frame-0 shape"));
                        }
                        let mut skin: Option<Vec<Mix>> = None;
                        if skinned_ok {
                            let table = skin_table(wpath.expect("skinned implies weights"), &tp, &bone_ix);
                            skin = Some(soup.src.iter().map(|&s| table[s.min(table.len() - 1)]).collect());
                        }
                        if !masks.is_empty() {
                            match clip_mesh(&soup.pos, &soup.tris, &masks, 400_000) {
                                Some((p2, t2)) => {
                                    soup.pos = p2;
                                    soup.tris = t2;
                                }
                                None => note(&mut notes, format!("mask on stroke of '{lpath}' produced too many pieces; exported unmasked")),
                            }
                        }
                        if soup.tris.is_empty() {
                            continue;
                        }
                        out.emit(Prim { name: format!("{}:{}:stroke", node.name, pname), pos: soup.pos, tris: soup.tris, skin, paint: PaintSrc::Paint(st.paint.clone()), uv: UvSrc::Paint, opacity, morph: None }, to_stage, to_host, host);
                    }
                }
            }
        }
    }

    // animation
    let times: Vec<f32> = (0..p.frames).map(|f| f as f32 / p.fps).collect();
    let mut tr = vec![Vec::new(); nb];
    let mut ro = vec![Vec::new(); nb];
    let mut sc = vec![Vec::new(); nb];
    for f in 0..p.frames {
        let fr = engine.evaluate(f as f32);
        for i in 0..nb {
            let (t, r, s) = trs(fr.world[i], parent[i].map(|q| fr.world[q]));
            tr[i].extend_from_slice(&t);
            ro[i].extend_from_slice(&r);
            sc[i].extend_from_slice(&s);
        }
    }
    let tmax = times.last().copied().unwrap_or(0.0);
    let t_acc = out.bin.add(&f32s(&times), 5126, times.len(), "SCALAR", Some((vec![0.0], vec![tmax])), None);
    let (mut samplers, mut channels) = (Vec::new(), Vec::new());
    for i in 0..nb {
        for (path, data, ty) in [("translation", &tr[i], "VEC3"), ("rotation", &ro[i], "VEC4"), ("scale", &sc[i], "VEC3")] {
            let acc = out.bin.add(&f32s(data), 5126, times.len(), ty, None, None);
            samplers.push(json!({"input": t_acc, "output": acc, "interpolation": "LINEAR"}));
            channels.push(json!({"sampler": samplers.len() - 1, "target": {"node": i, "path": path}}));
        }
    }
    for (ni, data, nt) in std::mem::take(&mut out.morph_anims) {
        let acc = out.bin.add(&f32s(&data), 5126, times.len() * nt, "SCALAR", None, None);
        samplers.push(json!({"input": t_acc, "output": acc, "interpolation": "LINEAR"}));
        channels.push(json!({"sampler": samplers.len() - 1, "target": {"node": ni, "path": "weights"}}));
    }

    if out.skinned_any {
        // inverse bind matrices from the frame-0 pose, in glTF space (y up), column-major
        let mut ibm = Vec::with_capacity(nb * 16);
        for i in 0..nb {
            let w = f0.world[i];
            let g = |x: f32, y: f32| {
                let q = w.apply(Vec2::new(x, -y));
                Vec2::new(q.x, -q.y)
            };
            let o = g(0.0, 0.0);
            let (c0, c1) = (g(1.0, 0.0) - o, g(0.0, 1.0) - o);
            let det = c0.x * c1.y - c1.x * c0.y;
            let det = if det.abs() < 1e-9 { 1e-9 } else { det };
            let (i00, i01, i10, i11) = (c1.y / det, -c1.x / det, -c0.y / det, c0.x / det);
            let (tx, ty) = (-(i00 * o.x + i01 * o.y), -(i10 * o.x + i11 * o.y));
            ibm.extend_from_slice(&[i00, i10, 0.0, 0.0, i01, i11, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, tx, ty, 0.0, 1.0]);
        }
        bin_ibm(&mut out.bin, &ibm, nb);
    }
    let ibm_acc = out.bin.accessors.len() - 1;
    // a skin needs one common root: with several root bones, hang them under a plain node
    let mut skeleton = roots[0];
    if out.skinned_any && roots.len() > 1 {
        out.nodes.push(json!({"name": "armature", "children": roots}));
        skeleton = out.nodes.len() - 1;
        out.scene_nodes.retain(|n| !roots.contains(n));
        out.scene_nodes.insert(0, skeleton);
    }
    let mut doc = json!({
        "asset": {"version": "2.0", "generator": "osatura"},
        "scene": 0,
        "scenes": [{"nodes": out.scene_nodes}],
        "nodes": out.nodes,
        "animations": [{"name": "main", "samplers": samplers, "channels": channels}],
        "bufferViews": out.bin.views,
        "accessors": out.bin.accessors,
        "buffers": [{"byteLength": out.bin.data.len()}],
    });
    if out.skinned_any {
        doc["skins"] = json!([{"joints": (0..nb).collect::<Vec<_>>(), "inverseBindMatrices": ibm_acc, "skeleton": skeleton}]);
    }
    if !out.meshes.is_empty() {
        doc["meshes"] = json!(out.meshes);
        doc["materials"] = json!(out.materials);
    }
    if !out.textures.is_empty() {
        doc["textures"] = json!(out.textures);
        doc["images"] = json!(out.images);
        doc["samplers"] = json!([{"magFilter": 9729, "minFilter": 9729, "wrapS": 33071, "wrapT": 33071}]);
    }
    let mut json_bytes = serde_json::to_vec(&doc).map_err(|e| e.to_string())?;
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }
    let mut data = out.bin.data;
    while !data.len().is_multiple_of(4) {
        data.push(0);
    }
    let total = 12 + 8 + json_bytes.len() + 8 + data.len();
    let mut glb = Vec::with_capacity(total);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total as u32).to_le_bytes());
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&(data.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(&data);
    Ok((glb, notes))
}
