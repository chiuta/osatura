// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Lottie: reduce a baked document, read it back (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Lottie: reduce a baked document, read it back

/// Hold-keyframe list of an animated track expanded to one value per frame (`n` frames).
pub(super) fn expand_hold(k: &Value, n: usize) -> Option<Vec<Value>> {
    let arr = k.as_array()?;
    let mut keys: Vec<(usize, &Value)> = arr.iter().filter_map(|e| Some((e.get("t")?.as_f64()? as usize, e.get("s")?.as_array()?.first()?))).collect();
    keys.sort_by_key(|k| k.0);
    if keys.is_empty() {
        return None;
    }
    Some((0..n).map(|f| { let i = keys.partition_point(|k| k.0 <= f); keys[i.saturating_sub(1)].1.clone() }).collect())
}

pub(super) fn round_doc(v: &mut Value, d: u8, cd: u8, colour: bool) {
    match v {
        Value::Number(x) => {
            if let Some(f) = x.as_f64() {
                if f.fract() != 0.0 {
                    *v = num(round_dec(f, if colour { cd } else { d }));
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|e| round_doc(e, d, cd, colour)),
        Value::Object(m) => {
            for (k, e) in m.iter_mut() {
                if matches!(k.as_str(), "fr" | "w" | "h" | "sw" | "sh" | "ip" | "op" | "t" | "s_") {
                    continue;
                }
                round_doc(e, d, cd, colour || k == "c" || k == "g");
            }
        }
        _ => {}
    }
}

/// Reduce a baked Lottie document ([`crate::export::lottie_json_mapped`]) with the lottie settings:
/// `decimals`, `path_eps` (spatial simplification over all frames, same vertices removed everywhere),
/// `key_tol` (0 = hold keyframes at every changed frame; > 0 = linear interpolation with the bound of
/// [`select_keys`]) and `frame_step`. Repeated frames are always merged and constant tracks become static.
pub fn lottie_reduce(base: &Value, s: &Settings) -> Value {
    let mut doc = base.clone();
    let n = doc.get("op").and_then(Value::as_f64).unwrap_or(1.0).max(1.0) as usize;
    let d = s.decimals;
    let linear = s.key_tol > 0.0;
    let tol = s.key_tol as f64;
    let step = s.frame_step.max(1) as usize;
    let Some(layers) = doc.get_mut("layers").and_then(Value::as_array_mut) else { return doc };
    for layer in layers.iter_mut() {
        // layer opacity
        if let Some(o) = layer.get_mut("ks").and_then(|k| k.get_mut("o")) {
            if o.get("a").and_then(Value::as_i64) == Some(1) {
                if let Some(vals) = o.get("k").and_then(|k| expand_hold(k, n)) {
                    let samp: Vec<Samp> = vals.iter().map(|v| Samp { sig: (false, 1), v: vec![round_dec(v.as_f64().unwrap_or(0.0), d)] }).collect();
                    *o = pick_encoding(&samp, step, tol, linear, &|sm| num(sm.v[0]));
                }
            }
        }
        // shape tracks
        let Some(groups) = layer.get_mut("shapes").and_then(Value::as_array_mut) else { continue };
        for g in groups.iter_mut() {
            let Some(items) = g.get_mut("it").and_then(Value::as_array_mut) else { continue };
            for it in items.iter_mut() {
                if it.get("ty").and_then(Value::as_str) != Some("sh") {
                    continue;
                }
                let Some(ks) = it.get_mut("ks") else { continue };
                if ks.get("a").and_then(Value::as_i64) != Some(1) {
                    continue;
                }
                let Some(vals) = ks.get("k").and_then(|k| expand_hold(k, n)) else { continue };
                let parsed: Vec<Option<(bool, Vec<PNode>)>> = vals.iter().map(parse_shape).collect();
                if parsed.iter().any(|p| p.is_none()) {
                    continue;
                }
                let mut shapes: Vec<(bool, Vec<PNode>)> = parsed.into_iter().map(|p| p.unwrap()).collect();
                // spatial simplification when the topology is the same in every frame
                if s.path_eps > 0.0 {
                    let (c0, n0) = (shapes[0].0, shapes[0].1.len());
                    if n0 >= 3 && shapes.iter().all(|(c, v)| *c == c0 && v.len() == n0) {
                        let mut fr: Vec<Vec<PNode>> = shapes.iter().map(|(_, v)| v.clone()).collect();
                        if simplify_paths(&mut fr, c0, s.path_eps as f64) > 0 {
                            shapes = fr.into_iter().map(|v| (c0, v)).collect();
                        }
                    }
                }
                let samp: Vec<Samp> = shapes.iter().map(|(c, v)| samp_of_shape(*c, v, d)).collect();
                *ks = pick_encoding(&samp, step, tol, linear, &shape_of_samp);
            }
        }
    }
    let cd = d.max(2);
    round_doc(&mut doc, d, cd, false);
    doc
}

/// Value of an animated or static track at integer frame `f` as the reader of the format would show it.
pub(super) fn read_shape_at(ks: &Value, f: usize) -> Option<(bool, Vec<PNode>)> {
    if ks.get("a").and_then(Value::as_i64) != Some(1) {
        return parse_shape(ks.get("k")?);
    }
    let arr = ks.get("k")?.as_array()?;
    let mut keys: Vec<(usize, &Value)> = arr.iter().filter_map(|e| Some((e.get("t")?.as_f64()? as usize, e))).collect();
    keys.sort_by_key(|k| k.0);
    let i = keys.partition_point(|k| k.0 <= f).saturating_sub(1);
    let (t0, e0) = keys[i];
    let a = parse_shape(e0.get("s")?.as_array()?.first()?)?;
    let hold = e0.get("h").and_then(Value::as_i64) == Some(1);
    if hold || i + 1 >= keys.len() || f <= t0 {
        return Some(a);
    }
    let (t1, e1) = keys[i + 1];
    let b = parse_shape(e1.get("s")?.as_array()?.first()?)?;
    if a.0 != b.0 || a.1.len() != b.1.len() || t1 <= t0 {
        return Some(a);
    }
    let u = (f - t0) as f64 / (t1 - t0) as f64;
    let mix = |p: [f64; 2], q: [f64; 2]| [lerp(p[0], q[0], u), lerp(p[1], q[1], u)];
    Some((a.0, a.1.iter().zip(&b.1).map(|(x, y)| PNode { p: mix(x.p, y.p), i: mix(x.i, y.i), o: mix(x.o, y.o) }).collect()))
}

pub(super) fn read_scalar_at(o: &Value, f: usize) -> f64 {
    if o.get("a").and_then(Value::as_i64) != Some(1) {
        return o.get("k").and_then(Value::as_f64).unwrap_or(100.0);
    }
    let Some(arr) = o.get("k").and_then(Value::as_array) else { return 100.0 };
    let mut keys: Vec<(usize, f64, bool)> = arr
        .iter()
        .filter_map(|e| Some((e.get("t")?.as_f64()? as usize, e.get("s")?.as_array()?.first()?.as_f64()?, e.get("h").and_then(Value::as_i64) == Some(1))))
        .collect();
    keys.sort_by_key(|a| a.0);
    if keys.is_empty() {
        return 100.0;
    }
    let i = keys.partition_point(|k| k.0 <= f).saturating_sub(1);
    let (t0, v0, hold) = keys[i];
    if hold || i + 1 >= keys.len() || f <= t0 {
        return v0;
    }
    let (t1, v1, _) = keys[i + 1];
    lerp(v0, v1, (f - t0) as f64 / (t1 - t0).max(1) as f64)
}

pub fn clone_frame(f: &Frame) -> Frame {
    Frame { world: f.world.clone(), camera: f.camera, nodes: f.nodes.clone(), roots: f.roots.clone(), nonfinite: f.nonfinite.clone() }
}

/// The picture a Lottie player would show at frame `f`, built from the evaluated engine frame `eval`
/// (structure, fills, strokes, masks) with the geometry and the layer opacity replaced by what is *read
/// from the produced document*. Baked vertices are in screen space, so layer transforms become identity;
/// group opacity is flattened into the leaves, as the exporter does.
pub fn lottie_model_frame(eval: &Frame, doc: &Value, map: &[crate::export::LottieLayerMap], f: usize) -> Frame {
    let mut fr = clone_frame(eval);
    // groups whose opacity lives in a precomp (a vector leaf below them has no top-level layer) keep it
    let mapped: std::collections::HashSet<usize> = map.iter().map(|m| m.node).collect();
    let mut keeps = vec![false; fr.nodes.len()];
    for id in 0..fr.nodes.len() {
        if fr.nodes[id].kind == NodeKind::Vector && !mapped.contains(&id) {
            let mut cur = Some(id);
            let mut guard = 0;
            // ancestors: nodes listing it as a child
            while let Some(c) = cur {
                guard += 1;
                if guard > 64 {
                    break;
                }
                cur = (0..fr.nodes.len()).find(|&n| fr.nodes[n].children.contains(&c));
                if let Some(a) = cur {
                    keeps[a] = true;
                }
            }
        }
    }
    for (i, nd) in fr.nodes.iter_mut().enumerate() {
        if nd.kind != NodeKind::Vector && keeps[i] {
            continue;
        }
        if nd.kind != NodeKind::Vector {
            nd.opacity = 1.0;
            nd.mask = None;
            nd.blend = crate::style::Blend::Normal;
        }
    }
    let layers = doc.get("layers").and_then(Value::as_array);
    for m in map.iter().filter(|m| !m.matte_copy) {
        let Some(layer) = layers.and_then(|l| l.get(m.index)) else { continue };
        let nd = &mut fr.nodes[m.node];
        nd.world = Affine2::IDENTITY;
        nd.visible = true;
        if let Some(o) = layer.get("ks").and_then(|k| k.get("o")) {
            nd.opacity = (read_scalar_at(o, f) / 100.0).clamp(0.0, 1.0) as f32;
        }
        for g in layer.get("shapes").and_then(Value::as_array).into_iter().flatten() {
            let pi: Option<usize> = g.get("nm").and_then(Value::as_str).and_then(|s| s.strip_prefix('g')).and_then(|s| s.parse().ok());
            let (Some(pi), Some(items)) = (pi, g.get("it").and_then(Value::as_array)) else { continue };
            let Some(sh) = items.iter().find(|i| i.get("ty").and_then(Value::as_str) == Some("sh")) else { continue };
            let Some(path) = nd.paths.get_mut(pi) else { continue };
            match sh.get("ks").and_then(|k| read_shape_at(k, f)) {
                Some((closed, nodes)) if nodes.len() >= 2 => {
                    path.anchors = anchors_of(&nodes);
                    path.closed = closed;
                }
                _ => path.anchors.clear(),
            }
        }
    }
    fr
}
