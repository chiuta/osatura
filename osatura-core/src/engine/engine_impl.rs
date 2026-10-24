// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `engine` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------------------------

/// Per anchor: `(bone, normalised weight)` pairs.
pub(super) type AnchorWeights = Vec<(usize, f32)>;

/// A validated project with all names resolved to indices.
pub struct Engine {
    pub project: Project,
    /// Draw layers flagged `reference` (editor previews); exports leave this off.
    pub show_reference: bool,
    pub(super) parent: Vec<Option<usize>>,
    pub(super) ik: Vec<IkResolved>,
    pub(super) constraints: Vec<ConstraintResolved>,
    pub(super) springs: Vec<SpringResolved>,
    /// frame → spring → absolute local pose angle (radians)
    pub(super) baked: Vec<Vec<f32>>,
    pub(super) inv_rest: Vec<Affine2>,
    pub(super) nodes: Vec<NodeInfo>,
    pub(super) roots: Vec<usize>,
    /// node → path → anchor → (bone, normalised weight); empty for non-vector nodes
    pub(super) skin: Vec<Vec<Vec<AnchorWeights>>>,
    /// node → mesh vertex → (bone, weight) for image meshes
    pub(super) mesh_skin: Vec<Vec<Vec<(usize, f32)>>>,
    pub(super) geo: Vec<NodeGeo>,
    /// node → per effect (index in `Layer::effects`) → animatable `(prop, slot)` pairs
    pub(super) fx_slots: Vec<Vec<Vec<(Prop, usize)>>>,
    pub(super) images: Vec<Option<Arc<ImageData>>>,
    pub(super) chans: Vec<ChanResolved>,
    pub(super) bone_ix: HashMap<String, usize>,
    pub(super) node_ix: HashMap<String, usize>,
    /// `(target, prop)` → slot, for every addressable path/deformer sub-target
    pub(super) slot_ix: HashMap<(String, Prop), usize>,
    pub(super) pslot_init: Vec<f32>,
    /// Resolved mesh locks and glues (`links.rs`); `None` when the project has none.
    pub(super) links: Option<crate::links::Resolved>,
    /// Container layout per node: `[dx, dy, kx, ky]` (`containers.rs`); empty when unused.
    pub(super) layout: Vec<Option<[f32; 4]>>,
}

pub(super) fn layer_at<'a>(p: &'a Project, idx: &[usize]) -> &'a Layer {
    let mut cur = &p.layers[idx[0]];
    for &i in &idx[1..] {
        cur = &cur.children()[i];
    }
    cur
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_nodes(
    layers: &[Layer],
    parent: Option<usize>,
    prefix: &str,
    idx_prefix: &[usize],
    root: Option<usize>,
    nodes: &mut Vec<NodeInfo>,
    node_ix: &mut HashMap<String, usize>,
    errs: &mut Vec<String>,
) {
    for (i, l) in layers.iter().enumerate() {
        let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
        let id = nodes.len();
        let mut idx_path = idx_prefix.to_vec();
        idx_path.push(i);
        if l.name.is_empty() || l.name.contains('/') || l.name.contains("::") {
            errs.push(format!("layer name '{}' must be non-empty and contain neither '/' nor '::'", l.name));
        }
        if node_ix.insert(path.clone(), id).is_some() {
            errs.push(format!("duplicate layer path '{path}'"));
        }
        nodes.push(NodeInfo {
            path: path.clone(),
            parent,
            children: vec![],
            idx_path: idx_path.clone(),
            bone: None,
            mask: None,
            root: root.unwrap_or(id),
        });
        if let Some(p) = parent {
            nodes[p].children.push(id);
        }
        build_nodes(l.children(), Some(id), &path, &idx_path, Some(root.unwrap_or(id)), nodes, node_ix, errs);
    }
}

/// Structural checks of a free-form mesh (finite coordinates, valid indices, matching UV count).
pub(super) fn validate_free_mesh(f: &crate::project::FreeMesh) -> Result<(), String> {
    if f.vertices.len() < 3 || f.triangles.is_empty() {
        return Err("needs at least 3 vertices and 1 triangle".into());
    }
    if f.vertices.len() > 200_000 || f.triangles.len() > 400_000 {
        return Err("too large (max 200000 vertices / 400000 triangles)".into());
    }
    if f.vertices.iter().any(|v| !v.x.is_finite() || !v.y.is_finite()) {
        return Err("vertex coordinates must be finite".into());
    }
    if !f.uvs.is_empty() && (f.uvs.len() != f.vertices.len() || f.uvs.iter().any(|v| !v.x.is_finite() || !v.y.is_finite())) {
        return Err("uvs must be empty or one finite pair per vertex".into());
    }
    let n = f.vertices.len() as u32;
    if let Some(t) = f.triangles.iter().find(|t| t.iter().any(|&i| i >= n)) {
        return Err(format!("triangle {:?} refers to a missing vertex (have {n})", t));
    }
    Ok(())
}

pub(super) fn load_image(src: &ImageSource, asset_dir: &std::path::Path) -> Result<ImageData, String> {
    let png = match (&src.png_base64, &src.file) {
        (Some(b64), _) => crate::base64::decode(b64)?,
        (None, Some(f)) => crate::assets::read(asset_dir, f, crate::assets::MAX_IMAGE_BYTES, "image")?,
        (None, None) => return Err("image has neither 'png_base64' nor 'file'".into()),
    };
    let pixmap = crate::assets::decode_png_checked(&png)?;
    Ok(ImageData { pixmap, png })
}

pub(super) fn bernstein(n: usize, i: usize, t: f32) -> f32 {
    let mut c = 1.0f32;
    for k in 0..i {
        c = c * (n - k) as f32 / (k + 1) as f32;
    }
    c * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32)
}

pub(super) fn wobble_disp(p: Vec2, tick: f32, w: &Wobble, amp: f32) -> Vec2 {
    let k = w.scale;
    Vec2::new(
        noise1(p.y * k + tick * 13.37, w.seed),
        noise1(p.x * k + tick * 7.77 + 100.0, w.seed.wrapping_add(1)),
    ) * amp
}

/// Resource caps that keep a corrupt or hostile file from exhausting memory or time.
pub const MAX_FRAMES: u32 = 200_000;

pub const MAX_STAGE: f32 = 32_768.0;

pub const MAX_BONES: usize = 4096;

impl Engine {
    /// Validate and compile (assets resolve relative to `assets::default_dir()`: the current directory unless the command line set it to the loaded project's directory).
    pub fn new(project: Project) -> Result<Engine, Vec<String>> {
        Engine::with_asset_dir(project, &crate::assets::default_dir())
    }

    /// Validate and compile. Returns every problem found, not just the first.
    pub fn with_asset_dir(mut project: Project, asset_dir: &std::path::Path) -> Result<Engine, Vec<String>> {
        let mut errs: Vec<String> = Vec::new();

        if !(project.fps > 0.0 && project.fps <= 1000.0) {
            errs.push("fps must be within (0, 1000]".into());
        }
        if project.frames == 0 || project.frames > MAX_FRAMES {
            errs.push(format!("frames must be within 1..={MAX_FRAMES}"));
        }
        if let Err(m) = check_magnitudes(&project) {
            errs.push(m);
        }
        if !(project.width > 0.0 && project.height > 0.0) {
            errs.push("width and height must be > 0".into());
        } else if project.width > MAX_STAGE || project.height > MAX_STAGE {
            errs.push(format!("width and height must be at most {MAX_STAGE}"));
        }
        if !project.camera.focus.is_finite() || !project.camera.blur.is_finite() || project.camera.blur < 0.0 {
            errs.push("camera focus must be finite and camera blur a finite number >= 0".into());
        }
        errs.extend(crate::soundtrack::validate(&project));
        errs.extend(crate::editordata::check(&project).errors);
        errs.extend(crate::postfx::validate_layer_normal_maps(&project));
        errs.extend(crate::machine::validate(&project));
        let cerrs = crate::containers::validate(&project);
        if cerrs.is_empty() {
            crate::containers::apply_clip_masks(&mut project);
        } else {
            errs.extend(cerrs);
        }
        if project.bones.len() > MAX_BONES {
            errs.push(format!("at most {MAX_BONES} bones"));
            return Err(errs);
        }

        let parent = match resolve_parents(&project.bones) {
            Ok(p) => p,
            Err(e) => {
                errs.push(e);
                return Err(errs);
            }
        };
        for b in &project.bones {
            if !(b.length.is_finite() && b.length >= 0.0) {
                errs.push(format!("bone '{}' has invalid length", b.name));
            }
            if let Some([lo, hi]) = b.angle_limits {
                if !matches!(lo.partial_cmp(&hi), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)) {
                    errs.push(format!("bone '{}': angle_limits min must be <= max", b.name));
                }
            }
            if let Some([lo, hi]) = b.scale_limits {
                if !(lo <= hi && lo > 0.0) {
                    errs.push(format!("bone '{}': scale_limits must satisfy 0 < min <= max", b.name));
                }
            }
        }
        let bone_ix: HashMap<String, usize> =
            project.bones.iter().enumerate().map(|(i, b)| (b.name.clone(), i)).collect();

        // Layer tree → arena (preorder).
        let mut nodes: Vec<NodeInfo> = Vec::new();
        let mut node_ix: HashMap<String, usize> = HashMap::new();
        build_nodes(&project.layers, None, "", &[], None, &mut nodes, &mut node_ix, &mut errs);
        let roots: Vec<usize> = (0..nodes.len()).filter(|&i| nodes[i].parent.is_none()).collect();

        // Per-layer references: bone attachment, masks.
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for id in 0..nodes.len() {
            let l = layer_at(&project, &nodes[id].idx_path);
            if let Some(b) = &l.bone {
                match bone_ix.get(b) {
                    Some(&bi) => nodes[id].bone = Some(bi),
                    None => errs.push(format!("layer '{}': unknown bone '{}'", nodes[id].path, b)),
                }
            }
            if let Some(m) = &l.mask {
                match node_ix.get(&m.layer) {
                    Some(&mi) if mi != id => nodes[id].mask = Some((mi, m.invert)),
                    Some(_) => errs.push(format!("layer '{}' cannot mask itself", nodes[id].path)),
                    None => errs.push(format!("layer '{}': unknown mask layer '{}'", nodes[id].path, m.layer)),
                }
            }
        }
        for id in 0..nodes.len() {
            let (mut cur, mut seen) = (id, HashSet::new());
            while let Some((m, _)) = nodes[cur].mask {
                if !seen.insert(m) || m == id {
                    errs.push(format!("mask cycle involving layer '{}'", nodes[id].path));
                    break;
                }
                cur = m;
            }
        }

        // IK chains
        let mut ik = Vec::new();
        for c in &project.ik {
            let mut ids = Vec::new();
            for n in &c.bones {
                match bone_ix.get(n) {
                    Some(&i) => ids.push(i),
                    None => errs.push(format!("ik '{}': unknown bone '{}'", c.name, n)),
                }
            }
            if !(0.0..=0.9).contains(&c.soft) || !(1.0..=4.0).contains(&c.stretch) {
                errs.push(format!("ik '{}': soft must be within 0..0.9 and stretch within 1..4", c.name));
            }
            let target = bone_ix.get(&c.target).copied();
            if target.is_none() {
                errs.push(format!("ik '{}': unknown target bone '{}'", c.name, c.target));
            }
            if ids.len() == c.bones.len() {
                if ids.len() < 2 {
                    errs.push(format!("ik '{}': needs at least two bones", c.name));
                } else if ids.windows(2).any(|w| parent[w[1]] != Some(w[0])) {
                    errs.push(format!("ik '{}': each bone must be a direct child of the previous one", c.name));
                } else if target.map(|t| ids.contains(&t)).unwrap_or(false) {
                    errs.push(format!("ik '{}': target must not be part of the chain", c.name));
                } else if let Some(target) = target {
                    ik.push(IkResolved { bones: ids, target, bend: c.bend, iterations: c.iterations.max(1), soft: c.soft, stretch: c.stretch });
                    continue;
                }
            }
            // keep indices aligned with project.ik even on error (we return Err below anyway)
            ik.push(IkResolved { bones: vec![], target: 0, bend: 1.0, iterations: 1, soft: 0.0, stretch: 1.0 });
        }
        let ik_ix: HashMap<&str, usize> = project.ik.iter().enumerate().map(|(i, c)| (c.name.as_str(), i)).collect();

        // Constraints
        let mut constraints = Vec::new();
        for c in &project.constraints {
            let b = bone_ix.get(&c.bone).copied();
            if b.is_none() {
                errs.push(format!("constraint '{}': unknown bone '{}'", c.name, c.bone));
            }
            let other = |n: &str, errs: &mut Vec<String>| {
                let r = bone_ix.get(n).copied();
                if r.is_none() {
                    errs.push(format!("constraint '{}': unknown bone '{}'", c.name, n));
                }
                r
            };
            let kind = match &c.kind {
                ConstraintKind::Aim { target, offset } => {
                    other(target, &mut errs).map(|t| CKind::Aim { target: t, offset: *offset })
                }
                ConstraintKind::CopyRotation { source } => {
                    other(source, &mut errs).map(|s| CKind::CopyRotation { source: s })
                }
                ConstraintKind::Pin { target } => other(target, &mut errs).map(|t| CKind::Pin { target: t }),
            };
            if let (Some(bone), Some(kind)) = (b, kind) {
                let partner = match kind {
                    CKind::Aim { target, .. } | CKind::Pin { target } => target,
                    CKind::CopyRotation { source } => source,
                };
                if partner == bone {
                    errs.push(format!("constraint '{}': bone cannot constrain itself", c.name));
                }
                constraints.push(ConstraintResolved { bone, kind });
            } else {
                constraints.push(ConstraintResolved { bone: 0, kind: CKind::Pin { target: 0 } });
            }
        }
        let cons_ix: HashMap<&str, usize> =
            project.constraints.iter().enumerate().map(|(i, c)| (c.name.as_str(), i)).collect();

        // Springs
        let mut springs = Vec::new();
        for s in &project.springs {
            match bone_ix.get(&s.bone) {
                Some(&b) => {
                    if !(s.frequency > 0.0 && s.frequency <= 30.0) || !(s.damping >= 0.0 && s.damping.is_finite()) {
                        errs.push(format!("spring on '{}': frequency must be in (0, 30] Hz and damping >= 0", s.bone));
                    }
                    if springs.iter().any(|x: &SpringResolved| x.bone == b) {
                        errs.push(format!("spring on '{}': bone already has a spring", s.bone));
                    }
                    springs.push(SpringResolved { bone: b, frequency: s.frequency, damping: s.damping, warmup: s.warmup });
                }
                None => errs.push(format!("spring: unknown bone '{}'", s.bone)),
            }
        }
        // Springs are baked in bone-index order so parents settle before children.
        springs.sort_by_key(|s| s.bone);

        // Skin weights, geometry slots, images, meshes
        let mut skin = Vec::with_capacity(nodes.len());
        let mut mesh_skin = Vec::with_capacity(nodes.len());
        let mut geo: Vec<NodeGeo> = Vec::with_capacity(nodes.len());
        let mut fx_slots: Vec<Vec<Vec<(Prop, usize)>>> = Vec::with_capacity(nodes.len());
        let mut images: Vec<Option<Arc<ImageData>>> = Vec::with_capacity(nodes.len());
        let mut slot_ix: HashMap<(String, Prop), usize> = HashMap::new();
        let mut pslot_init: Vec<f32> = Vec::new();
        let mut new_slot = |key: (String, Prop), init: f32, errs: &mut Vec<String>| -> usize {
            let i = pslot_init.len();
            pslot_init.push(init);
            if slot_ix.insert(key.clone(), i).is_some() {
                errs.push(format!("duplicate animatable target '{}' ({:?})", key.0, key.1));
            }
            i
        };
        let normalise = |ws: &[crate::path::BoneWeight], what: &str, errs: &mut Vec<String>| -> Vec<(usize, f32)> {
            let mut out: Vec<(usize, f32)> = Vec::new();
            for w in ws {
                match bone_ix.get(w.bone.as_str()) {
                    Some(&bi) if w.weight.is_finite() && w.weight >= 0.0 => out.push((bi, w.weight)),
                    Some(_) => errs.push(format!("{what}: invalid weight for '{}'", w.bone)),
                    None => errs.push(format!("{what}: unknown bone '{}'", w.bone)),
                }
            }
            let sum: f32 = out.iter().map(|w| w.1).sum();
            if sum > 0.0 {
                for w in &mut out {
                    w.1 /= sum;
                }
            } else {
                out.clear();
            }
            out
        };

        for node in &nodes {
            let l = layer_at(&project, &node.idx_path);
            let mut lp = Vec::new();
            let mut g = NodeGeo::default();
            let mut ms = Vec::new();
            let mut img = None;

            match &l.kind {
                LayerKind::Vector { paths } => {
                    let mut seen_paths = HashSet::new();
                    for (pi, p) in paths.iter().enumerate() {
                        let pname = if p.name.is_empty() { format!("path{pi}") } else { p.name.clone() };
                        if p.name.contains("::") {
                            errs.push(format!("layer '{}': path name '{}' must not contain '::'", node.path, p.name));
                        }
                        if !seen_paths.insert(pname.clone()) {
                            errs.push(format!("layer '{}': duplicate path name '{}'", node.path, pname));
                        }
                        let mut pa = Vec::new();
                        for (ai, a) in p.anchors.iter().enumerate() {
                            let what = format!("layer '{}' path '{}' anchor {}", node.path, pname, ai);
                            pa.push(normalise(&a.weights, &what, &mut errs));
                        }
                        lp.push(pa);

                        let mut mslots = Vec::new();
                        for m in &p.morphs {
                            if m.anchors.len() != p.anchors.len() {
                                errs.push(format!(
                                    "layer '{}' path '{}': morph '{}' has {} anchors, base has {}",
                                    node.path,
                                    pname,
                                    m.name,
                                    m.anchors.len(),
                                    p.anchors.len()
                                ));
                            }
                            mslots.push(new_slot(
                                (format!("{}::{}::{}", node.path, pname, m.name), Prop::PathMorph),
                                0.0,
                                &mut errs,
                            ));
                        }
                        g.morph_slots.push(mslots);
                        let mut pslots = Vec::new();
                        for ai in 0..p.anchors.len() {
                            let key = format!("{}::{}::{}", node.path, pname, ai);
                            let dx = new_slot((key.clone(), Prop::PointDx), 0.0, &mut errs);
                            let dy = new_slot((key, Prop::PointDy), 0.0, &mut errs);
                            pslots.push([dx, dy]);
                        }
                        g.point_slots.push(pslots);
                        let mut gs: [Option<[usize; 2]>; 4] = [None; 4];
                        let is_grad = |pt: &crate::style::Paint| matches!(pt, crate::style::Paint::Linear { .. } | crate::style::Paint::Radial { .. });
                        for (k, (has, name)) in [(p.fill.as_ref().is_some_and(is_grad), "fill.from"), (p.fill.as_ref().is_some_and(is_grad), "fill.to"), (p.stroke.as_ref().is_some_and(|s| is_grad(&s.paint)), "stroke.from"), (p.stroke.as_ref().is_some_and(|s| is_grad(&s.paint)), "stroke.to")].into_iter().enumerate() {
                            if has {
                                let key = format!("{}::{}::{}", node.path, pname, name);
                                let dx = new_slot((key.clone(), Prop::GradDx), 0.0, &mut errs);
                                let dy = new_slot((key, Prop::GradDy), 0.0, &mut errs);
                                gs[k] = Some([dx, dy]);
                            }
                        }
                        g.grad_slots.push(gs);
                    }
                }
                LayerKind::Image { source, mesh, .. } => match load_image(source, asset_dir) {
                    Ok(d) => {
                        if let Some(m) = mesh {
                            let n = m.vertex_count();
                            if let Some(fm) = &m.free {
                                if let Err(e) = validate_free_mesh(fm) {
                                    errs.push(format!("layer '{}': free mesh: {e}", node.path));
                                }
                            }
                            if m.free.is_none() && (m.cols == 0 || m.rows == 0) {
                                errs.push(format!("layer '{}': mesh needs cols >= 1 and rows >= 1", node.path));
                            } else if !m.weights.is_empty() && m.weights.len() != n {
                                errs.push(format!(
                                    "layer '{}': mesh has {} weight lists, expected {}",
                                    node.path,
                                    m.weights.len(),
                                    n
                                ));
                            } else {
                                for (vi, w) in m.weights.iter().enumerate() {
                                    let what = format!("layer '{}' mesh vertex {}", node.path, vi);
                                    ms.push(normalise(w, &what, &mut errs));
                                }
                            }
                        }
                        img = Some(Arc::new(d));
                    }
                    Err(e) => errs.push(format!("layer '{}': {e}", node.path)),
                },
                LayerKind::Liquid { blobs, threshold, cell, .. } => {
                    if blobs.is_empty() {
                        errs.push(format!("layer '{}': liquid needs at least one blob", node.path));
                    }
                    if ((*threshold).partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !matches!((*cell).partial_cmp(&0.25), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) {
                        errs.push(format!("layer '{}': liquid needs threshold > 0 and cell >= 0.25", node.path));
                    }
                    for b in blobs {
                        if b.radius.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
                            errs.push(format!("layer '{}': blob radius must be > 0", node.path));
                        }
                        if let Some(n) = &b.bone {
                            if !bone_ix.contains_key(n) {
                                errs.push(format!("layer '{}': blob bound to unknown bone '{n}'", node.path));
                            }
                        }
                    }
                }
                LayerKind::Boolean { paths, .. } => {
                    if paths.len() < 2 {
                        errs.push(format!("layer '{}': boolean needs at least two paths", node.path));
                    }
                }
                LayerKind::Emitter { spec } => {
                    if let Err(m) = spec.validate() {
                        errs.push(format!("layer '{}': emitter: {m}", node.path));
                    }
                    if let Some(n) = &spec.bone {
                        if !bone_ix.contains_key(n) {
                            errs.push(format!("layer '{}': emitter bound to unknown bone '{n}'", node.path));
                        }
                    }
                }
                _ => {}
            }

            // Deformers (vector and image layers)
            let mut seen_def = HashSet::new();
            for d in &l.deformers {
                if !seen_def.insert(d.name().to_string()) {
                    errs.push(format!("layer '{}': duplicate deformer name '{}'", node.path, d.name()));
                }
                if d.name().contains("::") {
                    errs.push(format!("layer '{}': deformer name '{}' must not contain '::'", node.path, d.name()));
                }
                match d {
                    Deformer::Lattice { name, cols, rows, rect, offsets } | Deformer::Cloth { name, cols, rows, rect, offsets } => {
                        let is_cloth = matches!(d, Deformer::Cloth { .. });
                        let (cols, rows) = (*cols, *rows);
                        let n = (cols + 1) * (rows + 1);
                        if cols == 0 || rows == 0 || !(rect[2] > 0.0 && rect[3] > 0.0) {
                            errs.push(format!(
                                "layer '{}' lattice '{}': needs cols,rows >= 1 and a rect with positive size",
                                node.path, name
                            ));
                            continue;
                        }
                        if !offsets.is_empty() && offsets.len() != n {
                            errs.push(format!("layer '{}' lattice '{}': offsets must have {} entries", node.path, name, n));
                            continue;
                        }
                        let mut slots = Vec::with_capacity(n);
                        for r in 0..=rows {
                            for c in 0..=cols {
                                let off = offsets.get(r * (cols + 1) + c).copied().unwrap_or(Vec2::ZERO);
                                let key = format!("{}::{}::{},{}", node.path, name, c, r);
                                let dx = new_slot((key.clone(), Prop::LatticeDx), off.x, &mut errs);
                                let dy = new_slot((key, Prop::LatticeDy), off.y, &mut errs);
                                slots.push([dx, dy]);
                            }
                        }
                        g.deformers.push(if is_cloth {
                            DefInfo::Cloth { cols, rows, rect: *rect, slots }
                        } else {
                            DefInfo::Lattice { cols, rows, rect: *rect, slots }
                        });
                    }
                    Deformer::Spine { name, rest, offsets, radius } => {
                        if rest.len() < 2 {
                            errs.push(format!("layer '{}' spine '{}': needs at least two points", node.path, name));
                            continue;
                        }
                        if !offsets.is_empty() && offsets.len() != rest.len() {
                            errs.push(format!("layer '{}' spine '{}': offsets must match the point count", node.path, name));
                            continue;
                        }
                        let mut slots = Vec::with_capacity(rest.len());
                        for (i, _) in rest.iter().enumerate() {
                            let off = offsets.get(i).copied().unwrap_or(Vec2::ZERO);
                            let key = format!("{}::{}::{}", node.path, name, i);
                            let dx = new_slot((key.clone(), Prop::SpineDx), off.x, &mut errs);
                            let dy = new_slot((key, Prop::SpineDy), off.y, &mut errs);
                            slots.push([dx, dy]);
                        }
                        let rest_poly = VPath::smooth_through("", rest, false, 1.0).polyline(0.5);
                        g.deformers.push(DefInfo::Spine { rest_pts: rest.clone(), rest_poly, slots, radius: *radius });
                    }
                    Deformer::Pin { name, pins, offsets, radius } => {
                        if !offsets.is_empty() && offsets.len() != pins.len() {
                            errs.push(format!("layer '{}' pin warp '{}': offsets must match the pin count", node.path, name));
                            continue;
                        }
                        let solver = match crate::pinwarp::PinSolver::new(pins, *radius) {
                            Ok(s) => s,
                            Err(m) => {
                                errs.push(format!("layer '{}' pin warp '{}': {m}", node.path, name));
                                continue;
                            }
                        };
                        let mut slots = Vec::with_capacity(pins.len());
                        for i in 0..pins.len() {
                            let off = offsets.get(i).copied().unwrap_or(Vec2::ZERO);
                            let key = format!("{}::{}::{}", node.path, name, i);
                            let dx = new_slot((key.clone(), Prop::PinDx), off.x, &mut errs);
                            let dy = new_slot((key, Prop::PinDy), off.y, &mut errs);
                            slots.push([dx, dy]);
                        }
                        g.deformers.push(DefInfo::Pin { solver, slots });
                    }
                    Deformer::Cage { name, rest, offsets } => {
                        if !offsets.is_empty() && offsets.len() != rest.len() {
                            errs.push(format!("layer '{}' cage '{}': offsets must match the vertex count", node.path, name));
                            continue;
                        }
                        let solver = match crate::cage::CageSolver::new(rest) {
                            Ok(s) => s,
                            Err(m) => {
                                errs.push(format!("layer '{}' cage '{}': {m}", node.path, name));
                                continue;
                            }
                        };
                        let mut slots = Vec::with_capacity(rest.len());
                        for i in 0..rest.len() {
                            let off = offsets.get(i).copied().unwrap_or(Vec2::ZERO);
                            let key = format!("{}::{}::{}", node.path, name, i);
                            let dx = new_slot((key.clone(), Prop::CageDx), off.x, &mut errs);
                            let dy = new_slot((key, Prop::CageDy), off.y, &mut errs);
                            slots.push([dx, dy]);
                        }
                        // weights of the rest mesh are fixed: compute them once (first deformer only,
                        // because later deformers see already displaced points)
                        let mut mesh_w = Vec::new();
                        if g.deformers.is_empty() {
                            if let (LayerKind::Image { origin, size, mesh, .. }, Some(d)) = (&l.kind, &img) {
                                let (pw, ph) = (d.pixmap.width() as f32, d.pixmap.height() as f32);
                                let sz = size.unwrap_or(Vec2::new(pw, ph));
                                let (c, r) = match mesh {
                                    Some(m) => (m.cols.max(1), m.rows.max(1)),
                                    None => (16, 16),
                                };
                                let rest_v = match mesh {
                                    Some(m) if m.free.is_some() => m.rest_vertices(*origin, sz),
                                    _ => crate::project::mesh_vertices(*origin, sz, c, r),
                                };
                                if rest_v.len() * rest.len() <= 4_000_000 {
                                    let mut w = Vec::new();
                                    for v in rest_v {
                                        mesh_w.push(if solver.weights(v, &mut w) { w.clone() } else { Vec::new() });
                                    }
                                }
                            }
                        }
                        g.deformers.push(DefInfo::Cage { solver, slots, mesh_w });
                    }
                }
            }
            if !l.deformers.is_empty() && !matches!(l.kind, LayerKind::Vector { .. } | LayerKind::Image { .. }) {
                errs.push(format!("layer '{}': deformers only apply to vector and image layers", node.path));
            }

            errs.extend(crate::layerfx::validate_list(&format!("layer '{}'", node.path), &l.effects));
            let mut fxs = Vec::new();
            for (ei, e) in l.effects.iter().enumerate() {
                let key = crate::layerfx::target(&node.path, ei);
                let mut v = Vec::new();
                for &pr in e.props() {
                    let init = e.get(pr).filter(|x| x.is_finite()).unwrap_or(0.0);
                    v.push((pr, new_slot((key.clone(), pr), init, &mut errs)));
                }
                fxs.push(v);
            }
            fx_slots.push(fxs);

            skin.push(lp);
            mesh_skin.push(ms);
            geo.push(g);
            images.push(img);
        }

        // Channels
        let resolve = |target: &str, prop: Prop| -> Option<usize> {
            match prop.target_kind() {
                TargetKind::Bone => bone_ix.get(target).copied(),
                TargetKind::Layer => node_ix.get(target).copied(),
                TargetKind::Camera => Some(0),
                TargetKind::Ik => ik_ix.get(target).copied(),
                TargetKind::Constraint => cons_ix.get(target).copied(),
                TargetKind::Sub => slot_ix.get(&(target.to_string(), prop)).copied(),
            }
        };
        let mut chans = Vec::new();
        for (ci, c) in project.channels.iter().enumerate() {
            let Some(slot) = resolve(&c.target, c.prop) else {
                errs.push(format!("channel {ci}: unknown target '{}' for {:?}", c.target, c.prop));
                continue;
            };
            if c.prop == Prop::LayerSwitch {
                let l = layer_at(&project, &nodes[slot].idx_path);
                if !matches!(l.kind, LayerKind::Switch { .. }) {
                    errs.push(format!("channel {ci}: layer_switch needs a switch layer, '{}' is not one", c.target));
                }
            }
            if !c.weight.is_finite() {
                errs.push(format!("channel {ci}: weight must be finite"));
            }
            let check_track = |t: &crate::anim::Track, errs: &mut Vec<String>| {
                let sorted = t.keys.windows(2).all(|w| w[0].frame < w[1].frame);
                let finite = t.keys.iter().all(|k| k.frame.is_finite() && k.value.is_finite());
                if !sorted || !finite {
                    errs.push(format!("channel {ci} ('{}'): keys must be finite and strictly increasing in frame", c.target));
                }
            };
            let (mut parsed, mut input) = (None, None);
            match &c.source {
                Source::Keys { track } => check_track(track, &mut errs),
                Source::Expr { expr: src } => match expr::parse(src) {
                    Ok(e) => {
                        let mut gets = Vec::new();
                        e.gets(&mut gets);
                        for (t, p) in gets {
                            match Prop::parse(&p) {
                                Some(pp) if resolve(&t, pp).is_some() => {}
                                Some(_) => errs.push(format!("channel {ci}: get(\"{t}\", \"{p}\"): unknown target")),
                                None => errs.push(format!("channel {ci}: get(\"{t}\", \"{p}\"): unknown property")),
                            }
                        }
                        parsed = Some(e);
                    }
                    Err(m) => errs.push(format!("channel {ci} ('{}'): expression error: {m}", c.target)),
                },
                Source::Noise { amp, freq, .. } => {
                    if !(amp.is_finite() && freq.is_finite()) {
                        errs.push(format!("channel {ci}: noise amp/freq must be finite"));
                    }
                }
                Source::Driven { input: it, input_prop, curve } => {
                    check_track(curve, &mut errs);
                    match resolve(it, *input_prop) {
                        Some(s) => input = Some((*input_prop, s)),
                        None => errs.push(format!("channel {ci}: unknown driver input '{it}' for {input_prop:?}")),
                    }
                }
            }
            chans.push(ChanResolved { prop: c.prop, slot, chan: ci, expr: parsed, input });
        }

        if !errs.is_empty() {
            return Err(errs);
        }

        let rest = fk(&project.bones, &parent, &vec![BonePose::default(); project.bones.len()]);
        let inv_rest = rest.iter().map(|m| m.inverse().unwrap_or(Affine2::IDENTITY)).collect();

        let trial = crate::machine::trial_errors(&project, asset_dir);
        if !trial.is_empty() {
            return Err(trial);
        }
        let mut engine = Engine {
            project,
            show_reference: false,
            parent,
            ik,
            constraints,
            springs,
            baked: Vec::new(),
            inv_rest,
            nodes,
            roots,
            skin,
            mesh_skin,
            geo,
            fx_slots,
            images,
            chans,
            bone_ix,
            node_ix,
            slot_ix,
            pslot_init,
            links: None,
            layout: Vec::new(),
        };
        engine.bake_springs();
        engine.links = crate::links::resolve(&engine)?;
        if !engine.project.containers.is_empty() {
            engine.layout = crate::containers::node_offsets(&engine).map_err(|e| vec![e])?;
        }
        Ok(engine)
    }

}
