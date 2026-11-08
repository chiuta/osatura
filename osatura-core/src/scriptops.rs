// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Pure project edits used by the scripting layer (`script.rs`): grouping, moving and deleting
//! layers, deleting bones, and setting static properties. Each function mutates a *copy* of the
//! project; the editor then validates the result and stores it as one undo step.
//! Layer paths are `/`-joined names (`group/child`); nothing here panics on bad input.

use crate::project::{Layer, LayerKind, Project};

/// A script value for the `set_layer` / `set_bone` property setters.
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Num(f32),
    Bool(bool),
    Str(String),
}

impl Val {
    fn num(&self, what: &str) -> Result<f32, String> {
        match self {
            Val::Num(v) if v.is_finite() => Ok(*v),
            _ => Err(format!("'{what}' needs a finite number")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrap {
    Group,
    Switch,
}

/// `old` -> `new` when `s` is `old` or lies below it (`sep` is "/" for paths, "::" for sub-targets).
fn rw(s: &str, old: &str, new: &str) -> Option<String> {
    if s == old {
        return Some(new.to_string());
    }
    for sep in ["/", "::"] {
        if let Some(rest) = s.strip_prefix(old).and_then(|r| r.strip_prefix(sep)) {
            return Some(format!("{new}{sep}{rest}"));
        }
    }
    None
}

fn fix_masks(ls: &mut [Layer], old: &str, new: &str) {
    for l in ls {
        if let Some(m) = &mut l.mask {
            if let Some(n) = rw(&m.layer, old, new) {
                m.layer = n;
            }
        }
        if let Some(c) = l.children_mut() {
            fix_masks(c, old, new);
        }
    }
}

/// Rewrite every reference to layer path `old` (and its descendants) after it moved to `new`:
/// channel targets and driver inputs, channel metadata, masks, skins and comps. References inside
/// expression text (`get("name", ...)`) cannot be rewritten; the validation then rejects the edit.
pub fn rewrite_layer_refs(p: &mut Project, old: &str, new: &str) {
    for c in &mut p.channels {
        if let Some(n) = rw(&c.target, old, new) {
            c.target = n;
        }
        if let crate::project::Source::Driven { input, .. } = &mut c.source {
            if let Some(n) = rw(input, old, new) {
                *input = n;
            }
        }
    }
    for m in &mut p.channel_meta {
        if let Some(n) = rw(&m.target, old, new) {
            m.target = n;
        }
    }
    fix_masks(&mut p.layers, old, new);
    for s in &mut p.skins {
        for l in &mut s.layers {
            if let Some(n) = rw(l, old, new) {
                *l = n;
            }
        }
    }
    for c in &mut p.comps {
        let old_map = std::mem::take(&mut c.layers);
        for (k, v) in old_map {
            c.layers.insert(rw(&k, old, new).unwrap_or(k), v);
        }
    }
}

fn split_path(path: &str) -> (&str, &str) {
    match path.rfind('/') {
        Some(i) => (&path[..i], &path[i + 1..]),
        None => ("", path),
    }
}

/// The child vector at `parent` ("" = top level); only groups and switches have children.
fn vec_at<'a>(root: &'a mut Vec<Layer>, parent: &str) -> Result<&'a mut Vec<Layer>, String> {
    if parent.is_empty() {
        return Ok(root);
    }
    let mut cur = root;
    for seg in parent.split('/') {
        let i = cur.iter().position(|l| l.name == seg).ok_or_else(|| format!("no layer '{parent}'"))?;
        cur = cur[i].children_mut().ok_or_else(|| format!("layer '{parent}' is not a group or switch"))?;
    }
    Ok(cur)
}

pub fn find_layer<'a>(ls: &'a [Layer], path: &str) -> Option<&'a Layer> {
    let mut cur = ls;
    let mut found = None;
    for seg in path.split('/') {
        let l = cur.iter().find(|l| l.name == seg)?;
        found = Some(l);
        cur = l.children();
    }
    found
}

fn find_layer_mut<'a>(ls: &'a mut Vec<Layer>, path: &str) -> Result<&'a mut Layer, String> {
    let (parent, name) = split_path(path);
    let v = vec_at(ls, parent)?;
    v.iter_mut().find(|l| l.name == name).ok_or_else(|| format!("no layer '{path}'"))
}

/// Create a group (or switch) holding the existing layers `members` (paths in one container, adjacent
/// to each other). The wrapper replaces them at the position of the first one, so the draw order
/// is unchanged; it has an identity transform so every member keeps its world transform. Members keep
/// their own relative order (not the order of the list).
pub fn wrap_layers(p: &mut Project, name: &str, members: &[String], kind: Wrap) -> Result<(), String> {
    if name.is_empty() || name.contains('/') || name.contains("::") {
        return Err(format!("bad layer name '{name}'"));
    }
    let wrapper = |children: Vec<Layer>| match kind {
        Wrap::Group => Layer::group(name, children),
        Wrap::Switch => Layer::switch(name, children),
    };
    if members.is_empty() {
        if kind == Wrap::Switch {
            return Err("a switch needs at least one layer".into());
        }
        p.layers.push(wrapper(Vec::new()));
        return Ok(());
    }
    let (parent, _) = split_path(&members[0]);
    let parent = parent.to_string();
    let mut names: Vec<&str> = Vec::new();
    for m in members {
        let (pp, n) = split_path(m);
        if pp != parent {
            return Err(format!("'{m}' is not in the same container as '{}'", members[0]));
        }
        if names.contains(&n) {
            return Err(format!("layer '{m}' listed twice"));
        }
        names.push(n);
    }
    let top = parent.is_empty();
    let v = vec_at(&mut p.layers, &parent)?;
    let mut idx: Vec<usize> = Vec::new();
    for n in &names {
        idx.push(v.iter().position(|l| l.name == *n).ok_or_else(|| format!("no layer '{}'", join(&parent, n)))?);
    }
    idx.sort_unstable();
    if idx.windows(2).any(|w| w[1] != w[0] + 1) {
        return Err("the layers must be adjacent in the draw order (other layers lie between them)".into());
    }
    if v.iter().enumerate().any(|(i, l)| l.name == name && !idx.contains(&i)) {
        return Err(format!("a layer named '{name}' already exists there"));
    }
    let children: Vec<Layer> = v.drain(idx[0]..=idx[idx.len() - 1]).collect();
    let mut w = wrapper(children);
    if top {
        // parallax belongs to the top-level ancestor; the members must agree or the picture would change
        let par = match &w.kind {
            LayerKind::Group { layers } | LayerKind::Selector { layers, .. } => layers[0].parallax,
            _ => 1.0,
        };
        if w.children().iter().any(|c| c.parallax != par) {
            return Err("the layers have different parallax values; set them equal first".into());
        }
        w.parallax = par;
    }
    let child_names: Vec<String> = w.children().iter().map(|c| c.name.clone()).collect();
    v.insert(idx[0], w);
    for n in child_names {
        rewrite_layer_refs(p, &join(&parent, &n), &join(&join(&parent, name), &n));
    }
    Ok(())
}

fn join(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

/// Move the layer at `path` to the end of group/switch `new_parent` ("" = top level), keeping references valid.
pub fn move_layer(p: &mut Project, path: &str, new_parent: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("empty layer path".into());
    }
    if new_parent == path || new_parent.starts_with(&format!("{path}/")) {
        return Err(format!("cannot move '{path}' into itself"));
    }
    if !new_parent.is_empty() {
        match find_layer(&p.layers, new_parent) {
            None => return Err(format!("no layer '{new_parent}'")),
            Some(l) if l.children_mut_probe() => {}
            Some(_) => return Err(format!("layer '{new_parent}' is not a group or switch")),
        }
    }
    let (parent, name) = split_path(path);
    let v = vec_at(&mut p.layers, parent)?;
    let i = v.iter().position(|l| l.name == name).ok_or_else(|| format!("no layer '{path}'"))?;
    let l = v.remove(i);
    let dest = vec_at(&mut p.layers, new_parent)?;
    if dest.iter().any(|x| x.name == name) {
        return Err(format!("'{new_parent}' already has a layer named '{name}'"));
    }
    dest.push(l);
    rewrite_layer_refs(p, path, &join(new_parent, name));
    Ok(())
}

trait ChildProbe {
    fn children_mut_probe(&self) -> bool;
}
impl ChildProbe for Layer {
    fn children_mut_probe(&self) -> bool {
        matches!(self.kind, LayerKind::Group { .. } | LayerKind::Selector { .. })
    }
}

/// Delete a layer and the channels, channel metadata, skin and comp entries that address it.
/// Masks, driver inputs and expressions that still refer to it make the validation reject the edit.
pub fn delete_layer(p: &mut Project, path: &str) -> Result<(), String> {
    let (parent, name) = split_path(path);
    let v = vec_at(&mut p.layers, parent)?;
    let i = v.iter().position(|l| l.name == name).ok_or_else(|| format!("no layer '{path}'"))?;
    v.remove(i);
    let gone = |s: &str| rw(s, path, "").is_some();
    p.channels.retain(|c| !gone(&c.target));
    p.channel_meta.retain(|m| !gone(&m.target));
    for s in &mut p.skins {
        s.layers.retain(|l| !gone(l));
    }
    for c in &mut p.comps {
        c.layers.retain(|k, _| !gone(k));
    }
    Ok(())
}

fn rides(ls: &[Layer], bone: &str) -> Option<String> {
    for l in ls {
        if l.bone.as_deref() == Some(bone) {
            return Some(l.name.clone());
        }
        if let Some(n) = rides(l.children(), bone) {
            return Some(format!("{}/{n}", l.name));
        }
    }
    None
}

/// Delete a bone that nothing else needs: no child bones, IK chains, constraints, springs or layers
/// riding it. Its own channels and overlay ranges are removed with it.
pub fn delete_bone(p: &mut Project, name: &str) -> Result<(), String> {
    let i = p.bones.iter().position(|b| b.name == name).ok_or_else(|| format!("no bone '{name}'"))?;
    if let Some(c) = p.bones.iter().find(|b| b.parent.as_deref() == Some(name)) {
        return Err(format!("bone '{name}' is the parent of '{}'", c.name));
    }
    if let Some(c) = p.ik.iter().find(|c| c.target == name || c.bones.iter().any(|b| b == name)) {
        return Err(format!("bone '{name}' is used by IK chain '{}'", c.name));
    }
    use crate::project::ConstraintKind as K;
    if let Some(c) = p.constraints.iter().find(|c| {
        c.bone == name
            || match &c.kind {
                K::Aim { target, .. } | K::Pin { target } => target == name,
                K::CopyRotation { source } => source == name,
            }
    }) {
        return Err(format!("bone '{name}' is used by constraint '{}'", c.name));
    }
    if p.springs.iter().any(|s| s.bone == name) {
        return Err(format!("bone '{name}' has a spring"));
    }
    if let Some(l) = rides(&p.layers, name) {
        return Err(format!("layer '{l}' rides bone '{name}'"));
    }
    p.bones.remove(i);
    p.channels.retain(|c| !(c.prop.target_kind() == crate::project::TargetKind::Bone && c.target == name));
    p.channel_meta.retain(|m| m.target != name);
    p.bone_vis.retain(|b| b.bone != name);
    Ok(())
}

/// Set a static (non-animated) layer property.
pub fn set_layer_prop(p: &mut Project, path: &str, prop: &str, v: &Val) -> Result<(), String> {
    let top = !path.contains('/');
    let l = find_layer_mut(&mut p.layers, path)?;
    match prop {
        "opacity" => {
            let o = v.num(prop)?;
            if !(0.0..=1.0).contains(&o) {
                return Err("opacity must be within 0..1".into());
            }
            l.opacity = o;
        }
        "visible" => match v {
            Val::Bool(b) => l.visible = *b,
            _ => return Err("'visible' needs true or false".into()),
        },
        "reference" => match v {
            Val::Bool(b) => l.reference = *b,
            _ => return Err("'reference' needs true or false".into()),
        },
        "x" => l.transform.position.x = v.num(prop)?,
        "y" => l.transform.position.y = v.num(prop)?,
        "rotation" => l.transform.rotation = v.num(prop)?,
        "scale_x" => l.transform.scale.x = v.num(prop)?,
        "scale_y" => l.transform.scale.y = v.num(prop)?,
        "skew" => l.transform.skew = v.num(prop)?,
        "anchor_x" => l.transform.anchor.x = v.num(prop)?,
        "anchor_y" => l.transform.anchor.y = v.num(prop)?,
        "parallax" => {
            let x = v.num(prop)?;
            if !top {
                return Err("parallax can only be set on top-level layers".into());
            }
            if !(0.0..=100.0).contains(&x) {
                return Err("parallax must be within 0..100".into());
            }
            l.parallax = x;
        }
        "bone" => match v {
            Val::Str(s) => l.bone = if s.is_empty() { None } else { Some(s.clone()) },
            _ => return Err("'bone' needs a bone name (empty = none)".into()),
        },
        _ => {
            return Err(format!("unknown layer property '{prop}' (opacity, visible, reference, x, y, rotation, scale_x, scale_y, skew, anchor_x, anchor_y, parallax, bone)"));
        }
    }
    Ok(())
}

/// Set a static bone property (rest pose in the parent's space; children follow).
pub fn set_bone_prop(p: &mut Project, name: &str, prop: &str, v: &Val) -> Result<(), String> {
    let b = p.bones.iter_mut().find(|b| b.name == name).ok_or_else(|| format!("no bone '{name}'"))?;
    match prop {
        "length" => {
            let x = v.num(prop)?;
            if x < 0.0 {
                return Err("bone length must not be negative".into());
            }
            b.length = x;
        }
        "x" => b.rest_offset.x = v.num(prop)?,
        "y" => b.rest_offset.y = v.num(prop)?,
        "angle" => b.rest_angle = v.num(prop)?,
        "angle_min" | "angle_max" => {
            let x = v.num(prop)?;
            let mut lim = b.angle_limits.unwrap_or([-360.0, 360.0]);
            if prop == "angle_min" {
                lim[0] = x;
            } else {
                lim[1] = x;
            }
            if lim[0] > lim[1] {
                return Err("angle_min must not exceed angle_max".into());
            }
            b.angle_limits = Some(lim);
        }
        _ => return Err(format!("unknown bone property '{prop}' (length, x, y, angle, angle_min, angle_max)")),
    }
    Ok(())
}
