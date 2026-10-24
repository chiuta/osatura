// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Reports (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Reports

/// Non-dominated candidates (smaller budgeted size, higher mean SSIM), by index into `all`.
pub fn pareto_front(all: &[Candidate]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..all.len()).filter(|&i| all[i].valid).collect();
    idx.sort_by(|&a, &b| all[a].budgeted.cmp(&all[b].budgeted).then(all[b].ssim_mean.total_cmp(&all[a].ssim_mean)).then(all[a].pos.cmp(&all[b].pos)));
    let mut front = Vec::new();
    let mut best = f64::NEG_INFINITY;
    for i in idx {
        if all[i].ssim_mean > best + 1e-12 {
            best = all[i].ssim_mean;
            front.push(i);
        }
    }
    front
}

impl FitResult {
    pub fn to_json(&self) -> Value {
        let o = &self.options;
        let c = &self.chosen;
        json!({
            "schema": SCHEMA,
            "format": o.format.name(),
            "budget_bytes": o.budget,
            "measure": o.measure.name(),
            "met": self.met,
            "reason": self.reason,
            "search": {
                "strategy": o.search.name(),
                "evaluations": self.evaluations,
                "time_limited": self.time_limited,
                "max_seconds": o.max_seconds,
                "chain_length": self.chain_len,
                "chain_kind": self.chain_kind,
                "probes": self.probes.iter().map(|&(r, b)| json!({"rung": r, "bytes": if b == usize::MAX { Value::Null } else { json!(b) }})).collect::<Vec<_>>(),
                "monotone": self.monotone,
                "fallback_linear": self.fallback_linear,
                "audit_agrees": self.audit_agrees,
            },
            "chosen": {
                "position": c.pos,
                "settings": c.settings.to_json(),
                "raw_bytes": c.raw,
                "gzip_bytes": c.gzip,
                "budgeted_bytes": c.budgeted,
                "floors": self.floors,
            },
            "quality": {
                "metric": "ssim",
                "sample_frames": self.sample_frames,
                "ssim_mean": self.quality.ssim_mean,
                "ssim_min": self.quality.ssim_min,
                "worst_frame": self.quality.worst_frame,
                "max_ciede2000": self.quality.de_max,
                "min_ssim_required": o.min_ssim,
                "per_frame": self.quality.per_frame.iter().map(|&(f, s, d)| json!({"frame": f, "ssim": s, "max_ciede2000": d})).collect::<Vec<_>>(),
            },
            "baseline": self.baseline.map(|(r, g)| json!({"raw_bytes": r, "gzip_bytes": g, "note": "the existing exporter at its default settings"})),
            "dead_layers": self.dead_layers,
            "axes": self.axes.iter().map(|(n, v)| json!({"name": n, "values": v})).collect::<Vec<_>>(),
        })
    }

    /// CSV of every produced candidate with the Pareto front marked.
    pub fn pareto_csv(&self) -> String {
        let front: std::collections::HashSet<usize> = pareto_front(&self.all).into_iter().collect();
        let mut s = String::from("position,decimals,path_eps,key_tol,frame_step,scale,colors,dither,delta_tol,crf,raw_bytes,gzip_bytes,budgeted_bytes,ssim_mean,ssim_min,valid,fits,on_front,chosen\n");
        for (i, c) in self.all.iter().enumerate() {
            let t = &c.settings;
            s.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},{},{},{},{}\n",
                c.pos.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("-"),
                t.decimals,
                t.path_eps,
                t.key_tol,
                t.frame_step,
                t.scale,
                t.colors,
                t.dither as u8,
                t.delta_tol,
                t.crf,
                c.raw,
                c.gzip,
                if c.valid { c.budgeted.to_string() } else { String::new() },
                c.ssim_mean,
                c.ssim_min,
                c.valid as u8,
                (c.valid && c.budgeted <= self.options.budget) as u8,
                front.contains(&i) as u8,
                (c.pos == self.chosen.pos) as u8
            ));
        }
        s
    }
}

pub(super) fn layer_bytes(l: &crate::project::Layer) -> usize {
    serde_json::to_vec(l).map_or(0, |v| v.len())
}

/// What contributes bytes: the project file per layer, image and channel; the baked Lottie per layer; the
/// HTML page per frame and per layer. `tol` is the key tolerance used for the "removable keys" estimate.
pub fn size_report(project: &Project, tol: f32) -> Result<Value, String> {
    let engine = Engine::new(project.clone()).map_err(|e| format!("project does not validate: {}", e.join("; ")))?;
    let pj = serde_json::to_vec(project).map_err(|e| e.to_string())?;
    // project file
    let mut layers: Vec<Value> = Vec::new();
    fn walk(ls: &[crate::project::Layer], prefix: &str, out: &mut Vec<Value>) {
        for l in ls {
            let path = if prefix.is_empty() { l.name.clone() } else { format!("{prefix}/{}", l.name) };
            let img = match &l.kind {
                crate::project::LayerKind::Image { source, .. } => source.png_base64.as_ref().map_or(0, |b| b.len()),
                _ => 0,
            };
            out.push(json!({"layer": path, "json_bytes": layer_bytes(l), "embedded_image_bytes": img}));
            walk(l.children(), &path, out);
        }
    }
    walk(&project.layers, "", &mut layers);
    layers.sort_by(|a, b| b["json_bytes"].as_u64().cmp(&a["json_bytes"].as_u64()));
    let mut chans: Vec<Value> = Vec::new();
    for c in &project.channels {
        let bytes = serde_json::to_vec(c).map_or(0, |v| v.len());
        let (keys, removable) = match &c.source {
            Source::Keys { track } => (track.keys.len(), track.keys.len() - reduce_track(track, tol, project.frames).keys.len()),
            _ => (0, 0),
        };
        chans.push(json!({"channel": format!("{}.{:?}", c.target, c.prop), "json_bytes": bytes, "keys": keys, "removable_keys_at_tol": removable}));
    }
    chans.sort_by(|a, b| b["json_bytes"].as_u64().cmp(&a["json_bytes"].as_u64()));
    // baked Lottie
    let (doc, map) = crate::export::lottie_json_mapped(&engine);
    let lj = serde_json::to_vec(&doc).map_err(|e| e.to_string())?;
    let mut lottie_layers: Vec<Value> = doc
        .get("layers")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .enumerate()
                .map(|(i, l)| {
                    let mc = map.iter().find(|m| m.index == i).is_some_and(|m| m.matte_copy);
                    json!({"layer": l.get("nm").cloned().unwrap_or(Value::Null), "json_bytes": serde_json::to_vec(l).map_or(0, |v| v.len()), "matte_copy": mc})
                })
                .collect()
        })
        .unwrap_or_default();
    lottie_layers.sort_by(|a, b| b["json_bytes"].as_u64().cmp(&a["json_bytes"].as_u64()));
    // HTML
    let n = project.frames.max(1) as usize;
    let mut frame_bytes = Vec::with_capacity(n);
    let mut by_group: HashMap<String, usize> = HashMap::new();
    let mut defs = 0usize;
    for f in 0..n {
        let svg = crate::svg::render_frame(&engine, f as f32, &crate::svg::RenderOptions::default());
        frame_bytes.push(svg.len());
        if let (Some(a), Some(b)) = (svg.find("<defs>"), svg.find("</defs>")) {
            defs += b + 7 - a;
        }
        let mut parts = svg.split("<g id=\"");
        parts.next();
        for p in parts {
            if let Some(e) = p.find('"') {
                *by_group.entry(p[..e].to_string()).or_insert(0) += p.len().min(p.find("</g>").map_or(p.len(), |x| x + 4));
            }
        }
    }
    let html = crate::export::html_player(&engine);
    let mut groups: Vec<(String, usize)> = by_group.into_iter().collect();
    groups.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(json!({
        "project": {"json_bytes": pj.len(), "gzip_bytes": gzip_len(&pj), "layers": layers, "channels": chans, "key_tolerance": tol},
        "lottie": {"raw_bytes": lj.len(), "gzip_bytes": gzip_len(&lj), "layers": lottie_layers},
        "html": {"raw_bytes": html.len(), "gzip_bytes": gzip_len(html.as_bytes()), "frames": n,
                 "frame_bytes_min": frame_bytes.iter().min(), "frame_bytes_max": frame_bytes.iter().max(), "frame_bytes_mean": frame_bytes.iter().sum::<usize>() / n,
                 "defs_bytes_total": defs,
                 "layers": groups.iter().map(|(g, b)| json!({"layer": g, "svg_bytes_all_frames": b})).collect::<Vec<_>>()},
        "raster": {"frames": n, "width": project.width, "height": project.height,
                   "raw_rgba_bytes": (project.width * project.height) as usize * 4 * n,
                   "note": "GIF / APNG / WebM sizes depend on the knobs; use export-fit to measure"},
    }))
}

/// Human-readable table for [`size_report`] (top `top` rows of each list).
pub fn size_report_text(r: &Value, top: usize) -> String {
    let mut s = String::new();
    let u = |v: &Value, k: &str| v.get(k).and_then(Value::as_u64).unwrap_or(0);
    let p = &r["project"];
    s.push_str(&format!("project file: {} bytes ({} gzip)\n  by layer (JSON bytes; embedded image base64 in brackets):\n", u(p, "json_bytes"), u(p, "gzip_bytes")));
    for l in p["layers"].as_array().into_iter().flatten().take(top) {
        s.push_str(&format!("    {:>9}  {}{}\n", u(l, "json_bytes"), l["layer"].as_str().unwrap_or("?"), if u(l, "embedded_image_bytes") > 0 { format!("  [image {}]", u(l, "embedded_image_bytes")) } else { String::new() }));
    }
    s.push_str(&format!("  by channel (keys, removable at tol {}):\n", p["key_tolerance"]));
    for c in p["channels"].as_array().into_iter().flatten().take(top) {
        s.push_str(&format!("    {:>9}  {}  keys {} removable {}\n", u(c, "json_bytes"), c["channel"].as_str().unwrap_or("?"), u(c, "keys"), u(c, "removable_keys_at_tol")));
    }
    let l = &r["lottie"];
    s.push_str(&format!("baked Lottie: {} bytes ({} gzip), by layer:\n", u(l, "raw_bytes"), u(l, "gzip_bytes")));
    for x in l["layers"].as_array().into_iter().flatten().take(top) {
        s.push_str(&format!("    {:>9}  {}{}\n", u(x, "json_bytes"), x["layer"].as_str().unwrap_or("?"), if x["matte_copy"].as_bool() == Some(true) { "  (matte copy)" } else { "" }));
    }
    let h = &r["html"];
    s.push_str(&format!("HTML page: {} bytes ({} gzip), {} frames of {}..{} bytes (mean {}), gradient defs {} bytes in total; by layer over all frames:\n", u(h, "raw_bytes"), u(h, "gzip_bytes"), u(h, "frames"), u(h, "frame_bytes_min"), u(h, "frame_bytes_max"), u(h, "frame_bytes_mean"), u(h, "defs_bytes_total")));
    for x in h["layers"].as_array().into_iter().flatten().take(top) {
        s.push_str(&format!("    {:>9}  {}\n", u(x, "svg_bytes_all_frames"), x["layer"].as_str().unwrap_or("?")));
    }
    s.push_str(&format!("raster: {} frames of {}x{} = {} bytes uncompressed RGBA\n", u(&r["raster"], "frames"), r["raster"]["width"], r["raster"]["height"], u(&r["raster"], "raw_rgba_bytes")));
    s
}
