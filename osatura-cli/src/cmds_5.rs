// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Commands part 5 of the dispatcher in `main.rs` (arms kept in their original order, so the first matching arm still wins): mocap-info, mocap-match, mocap-retarget, machine, machine-sim, bind, bind-eval, container, scroll, layout-preview, pick, select-box, timeline, keyedit, render-gpu, collab-sim, vcs, fuzz, fuzz-grid, ci, export-fit, size-report, lod, lod-pick, face-train, face-eval, face-detect, face-track, face-synth, collab-token, color-space, reparent, comp.
#![allow(unused_imports)]
use super::*;

/// `Some(result)` when one of this part's arms matched the command line, `None` to try the next part.
pub(crate) fn run(a: &Args, raw: &[String], pos: &[&str]) -> Option<Result<(), String>> {
    let _ = raw;
    let handled = std::cell::Cell::new(true);
    let r = (|| -> Result<(), String> {
        match pos {
        ["mocap-info", file] => {
            let b = osatura_core::mocap::parse_bvh(&fs::read_to_string(file).map_err(|e| format!("cannot read {file}: {e}"))?).map_err(|e| format!("{file}: {e}"))?;
            print!("{}", b.summary());
            Ok(())
        }
        ["mocap-match", path, file] => {
            let p = load_project(path)?;
            let b = osatura_core::mocap::parse_bvh(&fs::read_to_string(file).map_err(|e| format!("cannot read {file}: {e}"))?).map_err(|e| format!("{file}: {e}"))?;
            let map = mocap_map(a)?;
            let m = osatura_core::mocap::match_bones(&b, &p, map.as_ref())?;
            print!("{}", osatura_core::mocap::format_table(&m));
            Ok(())
        }
        ["mocap-retarget", path, file, out] => {
            use osatura_core::mocap as mc;
            let mut p = load_project(path)?;
            let b = mc::parse_bvh(&fs::read_to_string(file).map_err(|e| format!("cannot read {file}: {e}"))?).map_err(|e| format!("{file}: {e}"))?;
            let map = mocap_map(a)?;
            let m = mc::match_bones(&b, &p, map.as_ref())?;
            let idx = |key: &str| -> Result<Option<usize>, String> {
                a.opts.get(key).map(|v| v.parse::<usize>().map_err(|_| format!("{key} needs a frame number, got '{v}'"))).transpose()
            };
            let mut o = mc::RetargetOptions::default();
            if let Some(v) = a.opts.get("--view") { o.view = mc::View::parse(v)?; }
            o.scale = match a.opts.get("--scale").map(String::as_str) {
                None | Some("auto") => mc::Scale::Auto,
                Some(v) => mc::Scale::Fixed(v.parse().map_err(|_| format!("--scale needs 'auto' or a number, got '{v}'"))?),
            };
            if a.opts.contains_key("--fps") { o.fps = Some(num(a, "--fps", 0.0)? as f64); }
            o.smooth = num(a, "--smooth", 0.0)? as usize;
            o.start = idx("--start")?;
            o.end = idx("--end")?;
            o.rest_frame = idx("--rest-frame")?;
            o.min_confidence = num(a, "--min-conf", o.min_confidence)?;
            o.weight = num(a, "--weight", 1.0)?;
            o.mirror = a.flags.iter().any(|f| f == "--mirror");
            o.in_place = a.flags.iter().any(|f| f == "--in-place");
            o.root_motion = !a.flags.iter().any(|f| f == "--no-root");
            if let Some(bone) = a.opts.get("--footlock") {
                o.foot_lock = Some((bone.clone(), num(a, "--ground", 0.0)?));
            }
            let r = mc::retarget(&p, &b, &m, &o)?;
            print!("{}", mc::format_table(&m));
            // the library's own "project length extended" note is replaced by the fuller message printed after the summary
            for n in r.notes.iter().filter(|n| !n.starts_with("project length extended")) { println!("note: {n}"); }
            let old_frames = p.frames;
            let keep_length = a.flags.iter().any(|f| f == "--keep-length");
            let mut cut_keys = 0usize;
            if keep_length && r.project_frames > old_frames {
                // bake the part of the clip that fits: the last frame of the project gets an interpolated key, later keys go
                let mut r2 = r.clone();
                let last = old_frames.saturating_sub(1) as f32;
                for c in &mut r2.channels {
                    if let Source::Keys { track } = &mut c.source {
                        let end = track.eval(last);
                        let before = track.keys.len();
                        track.keys.retain(|k| k.frame <= last);
                        cut_keys += before - track.keys.len();
                        if let Some(v) = end {
                            if track.keys.last().is_none_or(|k| k.frame < last) {
                                track.set_key(last, v, osatura_core::anim::Ease::Linear);
                            }
                        }
                    }
                }
                r2.project_frames = old_frames;
                mc::bake(&mut p, &r2);
            } else {
                mc::bake(&mut p, &r);
            }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("{} bones, {} frames at {} fps, root scale {:.4}, +{} channels; wrote {out}", r.bones_used, r.out_frames, o.fps.unwrap_or(p.fps as f64), r.scale, r.channels.len());
            if p.frames > old_frames {
                println!("note: the project length was extended from {old_frames} to {} frames to hold the {}-frame clip (every other channel keeps its keys; pass --keep-length to cut the clip at {old_frames} frames instead)", p.frames, r.out_frames);
            } else if cut_keys > 0 {
                println!("note: --keep-length: the clip is {} frames, the project {old_frames}; {cut_keys} key(s) after frame {} were not baked", r.out_frames, old_frames.saturating_sub(1));
            }
            Ok(())
        }
        ["machine", "add", path, out, file] => {
            let mut p = load_project(path)?;
            let m: osatura_core::machine::Machine = read_json(file)?;
            let name = m.name.clone();
            osatura_core::machine::add(&mut p, m)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("added machine '{name}' ({} machines); wrote {out}", p.machines.len());
            Ok(())
        }
        ["machine", "list", path] => {
            let p = load_project(path)?;
            for m in &p.machines {
                let ins: Vec<String> = m.inputs.iter().map(|i| format!("{}:{:?}", i.name, i.kind).to_lowercase()).collect();
                println!("{}  inputs [{}]", m.name, ins.join(", "));
                for l in &m.layers {
                    println!("  layer {} (entry {}, mask [{}]): {} states, {} transitions", l.name, l.entry, l.mask.join(","), l.states.len(), l.transitions.len());
                }
            }
            if p.machines.is_empty() { println!("no machines"); }
            Ok(())
        }
        ["machine", "remove", path, out, name] => {
            let mut p = load_project(path)?;
            osatura_core::machine::remove(&mut p, name)?;
            write_json(&p, out)?;
            println!("removed machine '{name}'");
            Ok(())
        }
        ["machine", "validate", path] => {
            let p = load_project(path)?;
            let mut errs = osatura_core::machine::validate(&p);
            if errs.is_empty() {
                errs.extend(osatura_core::machine::trial_errors(&p, &asset_dir(path)));
            }
            if errs.is_empty() {
                println!("{} machines, {} bindings: ok", p.machines.len(), p.bindings.len());
                Ok(())
            } else {
                Err(format!("{} problem(s):\n  - {}", errs.len(), errs.join("\n  - ")))
            }
        }
        ["machine-sim", path, script, outp] => {
            let p = load_project(path)?;
            let sc: osatura_core::machine::Script = read_json(script)?;
            let fps = num(a, "--fps", p.fps)?;
            let as_dir = !outp.to_ascii_lowercase().ends_with(".json");
            let n_svg = match a.opts.get("--svg-frames") {
                Some(v) => v.parse::<usize>().map_err(|_| format!("--svg-frames needs a whole number, got '{v}'"))?,
                None => if as_dir { 12 } else { 0 },
            };
            if as_dir { fs::create_dir_all(outp).map_err(|e| format!("cannot create {outp}: {e}"))?; }
            // total ticks are needed to pick evenly spaced frames: run once without rendering
            let dir = asset_dir(path);
            let total = osatura_core::machine::simulate(&p, &sc, fps, &dir, |_, _| Ok(()))?.ticks.len();
            let pick: Vec<usize> = if n_svg == 0 { vec![] } else if n_svg >= total { (0..total).collect() } else { (0..n_svg).map(|i| if n_svg == 1 { 0 } else { i * (total - 1) / (n_svg - 1) }).collect() };
            let mut written = 0;
            let log = osatura_core::machine::simulate(&p, &sc, fps, &dir, |tick, rt| {
                if pick.contains(&tick) {
                    let eng = rt.engine()?;
                    let svg = render_frame(&eng, rt.base_frame(), &RenderOptions { show_bones: false });
                    let f = Path::new(outp).join(format!("frame_{tick:05}.svg"));
                    fwrite(&f, svg)?;
                    written += 1;
                }
                Ok(())
            })?;
            let json_path = if as_dir { Path::new(outp).join("sim.json").to_string_lossy().to_string() } else { outp.to_string() };
            fwrite(&json_path, serde_json::to_string_pretty(&log).map_err(|e| e.to_string())?)?;
            for e in &log.events {
                println!("{:8.3}s  {:8} {:?} {}", e.time, e.layer, e.kind, e.state);
            }
            println!("{} ticks at {fps} fps, {} events, {written} svg frames; wrote {json_path}", log.ticks.len(), log.events.len());
            Ok(())
        }
        ["bind", "add", path, out, name, vpath, target] => {
            use osatura_core::binding::{Binding, Remap, Target};
            let mut p = load_project(path)?;
            let tgt = if let Some(rest) = target.strip_prefix('@') {
                let (m, i) = rest.split_once(':').ok_or("machine target must be @machine:input")?;
                Target::Input { machine: m.to_string(), input: i.to_string() }
            } else {
                let (t, pr) = target.rsplit_once(':').ok_or("property target must be target:prop (e.g. arm:bone_angle) or @machine:input")?;
                let prop = osatura_core::project::Prop::parse(pr).ok_or_else(|| format!("unknown property '{pr}'"))?;
                Target::Prop { target: t.to_string(), prop }
            };
            let mut b = Binding::new(name, vpath, tgt);
            b.expr = a.opts.get("--expr").cloned();
            b.smooth = num(a, "--smooth", 0.0)?;
            b.weight = num(a, "--weight", 1.0)?;
            if let Some(m) = a.opts.get("--mode") {
                b.mode = serde_json::from_value(serde_json::Value::String(m.clone())).map_err(|_| format!("unknown mode '{m}' (replace, add, multiply)"))?;
            }
            if let Some(r) = a.opts.get("--remap") {
                let v: Vec<f32> = r.split(',').map(|x| x.trim().parse::<f32>().map_err(|_| format!("bad number '{x}' in --remap"))).collect::<Result<_, _>>()?;
                if v.len() != 4 { return Err("--remap needs in_min,in_max,out_min,out_max".into()); }
                b.remap = Some(Remap { in_min: v[0], in_max: v[1], out_min: v[2], out_max: v[3], clamp: !a.flags.iter().any(|f| f == "--no-clamp"), curve: osatura_core::Ease::Linear });
            }
            osatura_core::machine::add_binding(&mut p, b)?;
            write_json(&p, out)?;
            println!("added binding '{name}' ({} bindings); wrote {out}", p.bindings.len());
            Ok(())
        }
        ["bind", "list", path] => {
            let p = load_project(path)?;
            for b in &p.bindings {
                let tgt = match &b.target {
                    osatura_core::binding::Target::Prop { target, prop } => format!("{target}:{prop:?}"),
                    osatura_core::binding::Target::Input { machine, input } => format!("@{machine}:{input}"),
                };
                let mut extra = String::new();
                if let Some(e) = &b.expr { extra += &format!(" expr \"{e}\""); }
                if let Some(r) = &b.remap { extra += &format!(" remap {}..{} -> {}..{}{}", r.in_min, r.in_max, r.out_min, r.out_max, if r.clamp { " clamp" } else { "" }); }
                if b.smooth > 0.0 { extra += &format!(" smooth {}s", b.smooth); }
                println!("{}  {} -> {tgt}{extra}", b.name, b.path);
            }
            if p.bindings.is_empty() { println!("no bindings"); }
            Ok(())
        }
        ["bind", "remove", path, out, name] => {
            let mut p = load_project(path)?;
            osatura_core::machine::remove_binding(&mut p, name)?;
            write_json(&p, out)?;
            println!("removed binding '{name}'");
            Ok(())
        }
        ["bind-eval", path, values] => {
            let p = load_project(path)?;
            let vals: std::collections::BTreeMap<String, f64> = read_json(values)?;
            let list: Vec<(String, f64)> = vals.into_iter().collect();
            let outv = osatura_core::binding::eval_once(&p.bindings, &list)?;
            let mut obj = serde_json::Map::new();
            for (n, v) in &outv {
                println!("{n} = {v}");
                obj.insert(n.clone(), serde_json::json!(v));
            }
            if let Some(o) = a.opts.get("--data") {
                fwrite(o, serde_json::to_string_pretty(&obj).map_err(|e| e.to_string())?)?;
            }
            let unbound = p.bindings.len() - outv.len();
            if unbound > 0 { println!("{unbound} binding(s) have no value in {values}"); }
            Ok(())
        }
        ["container", "add", path, out, file] => {
            let mut p = load_project(path)?;
            let c: osatura_core::containers::Container = read_json(file)?;
            let id = c.id.clone();
            osatura_core::containers::add_container(&mut p, c)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("added container '{id}' ({} containers); wrote {out}", p.containers.len());
            Ok(())
        }
        ["container", "list", path] => {
            let p = load_project(path)?;
            for c in &p.containers {
                println!("{}: {:?}, {} members, gap {}, justify {:?}, align {:?}, wrap {:?}, rect {}", c.id, c.axis, c.members.len(), c.gap, c.justify, c.align, c.wrap, serde_json::to_string(&c.rect).unwrap_or_default());
            }
            if p.containers.is_empty() { println!("no containers"); }
            Ok(())
        }
        ["container", "remove", path, out, id] => {
            let mut p = load_project(path)?;
            osatura_core::containers::remove_container(&mut p, id)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("removed container '{id}'");
            Ok(())
        }
        ["container", "solve", path] => {
            let mut p = load_project(path)?;
            if let Some(sz) = a.opts.get("--size") {
                let (w, h) = osatura_core::containers::parse_size(sz)?;
                p = osatura_core::containers::responsive(&p, w, h)?;
            }
            let sol = osatura_core::containers::solve_project(&p)?;
            println!("canvas {}x{}", sol.canvas.0, sol.canvas.1);
            for (id, r) in &sol.rects { println!("container {id}: x {:.2} y {:.2} w {:.2} h {:.2}", r[0], r[1], r[2], r[3]); }
            for l in &sol.layers { println!("layer {}: dx {:.3} dy {:.3} scale {:.4} x {:.4}", l.layer, l.dx, l.dy, l.kx, l.ky); }
            for s in &sol.scrolls { println!("scroll {}: viewport {:?} content {:?} max {:?} offset {:?}", s.id, s.viewport, s.content, s.max, s.offset); }
            if let Some(d) = a.opts.get("--data") {
                let v = serde_json::json!({
                    "canvas": [sol.canvas.0, sol.canvas.1],
                    "containers": sol.rects.iter().map(|(i, r)| serde_json::json!({"id": i, "rect": r})).collect::<Vec<_>>(),
                    "layers": sol.layers.iter().map(|l| serde_json::json!({"layer": l.layer, "dx": l.dx, "dy": l.dy, "kx": l.kx, "ky": l.ky})).collect::<Vec<_>>(),
                    "scrolls": sol.scrolls.iter().map(|s| serde_json::json!({"id": s.id, "viewport": s.viewport, "content": s.content, "max": s.max, "offset": s.offset})).collect::<Vec<_>>(),
                });
                fwrite(d, serde_json::to_string_pretty(&v).unwrap_or_default())?;
            }
            Ok(())
        }
        ["scroll", "add", path, out, file] => {
            let mut p = load_project(path)?;
            let s: osatura_core::containers::Scroll = read_json(file)?;
            let id = s.id.clone();
            osatura_core::containers::add_scroll(&mut p, s)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("added scroll '{id}' ({} scrolls); wrote {out}", p.scrolls.len());
            Ok(())
        }
        ["scroll", "list", path] => {
            let p = load_project(path)?;
            for s in &p.scrolls {
                println!("{}: content {}, axis {:?}, offset {:?}, elastic {}, snap x {:?} y {:?}, clip {}", s.id, s.content, s.axis, s.offset, s.elastic.is_some(), s.snap_x, s.snap_y, s.clip_layer.as_deref().unwrap_or("-"));
            }
            if p.scrolls.is_empty() { println!("no scroll regions"); }
            Ok(())
        }
        ["scroll", "remove", path, out, id] => {
            let mut p = load_project(path)?;
            osatura_core::containers::remove_scroll(&mut p, id)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("removed scroll '{id}'");
            Ok(())
        }
        ["scroll", "eval", path, id] => {
            use osatura_core::containers as ct;
            let mut p = load_project(path)?;
            if let Some(sz) = a.opts.get("--size") {
                let (w, h) = ct::parse_size(sz)?;
                p = ct::responsive(&p, w, h)?;
            }
            let pair = |key: &str| -> Result<Option<[f32; 2]>, String> {
                match a.opts.get(key) {
                    None => Ok(None),
                    Some(v) => {
                        let (x, y) = v.split_once(',').ok_or_else(|| format!("{key} needs X,Y"))?;
                        Ok(Some([x.trim().parse().map_err(|_| format!("bad number in {key}"))?, y.trim().parse().map_err(|_| format!("bad number in {key}"))?]))
                    }
                }
            };
            if let Some(o) = pair("--offset")? { ct::set_offset(&mut p, id, o)?; }
            let sol = ct::solve_project(&p)?;
            let info = sol.scrolls.iter().find(|s| &s.id == id).ok_or_else(|| format!("no scroll '{id}'"))?;
            println!("viewport {:?} content {:?} max {:?}", info.viewport, info.content, info.max);
            println!("requested offset {:?} -> applied {:?}", p.scrolls.iter().find(|s| &s.id == id).map(|s| s.offset).unwrap_or([0.0; 2]), info.offset);
            if let Some(v) = pair("--velocity")? {
                let sc = p.scrolls.iter().find(|s| &s.id == id).ok_or_else(|| format!("no scroll '{id}'"))?;
                let mut rt = ct::ScrollRuntime::new(sc, info.max);
                rt.drag_to(sc.offset);
                rt.release(v);
                let total = num(a, "--time", 2.0)?;
                let steps = (total * 60.0).round().max(1.0) as usize;
                for i in 1..=steps {
                    rt.step(1.0 / 60.0);
                    if i % 6 == 0 || i == steps || rt.is_settled() { println!("t {:.3}s offset [{:.3}, {:.3}]", i as f32 / 60.0, rt.pos[0], rt.pos[1]); }
                    if rt.is_settled() { println!("settled"); break; }
                }
            }
            Ok(())
        }
        ["layout-preview", path, out] => {
            use osatura_core::containers as ct;
            let mut p = load_project(path)?;
            let sz = a.opts.get("--size").ok_or("layout-preview needs --size WxH")?;
            let (w, h) = ct::parse_size(sz)?;
            p = ct::responsive(&p, w, h)?;
            if let Some(spec) = a.opts.get("--scroll") {
                for part in spec.split(';') {
                    let (id, v) = part.split_once('=').ok_or("--scroll needs id=X,Y")?;
                    let (x, y) = v.split_once(',').ok_or("--scroll needs id=X,Y")?;
                    ct::set_offset(&mut p, id, [x.trim().parse().map_err(|_| "bad scroll x")?, y.trim().parse().map_err(|_| "bad scroll y")?])?;
                }
            }
            let sol = ct::solve_project(&p)?;
            let e = compile(p, path)?;
            let frame = num(a, "--frame", 0.0)?;
            let svg = ct::with_overlay(&render_frame(&e, frame, &RenderOptions { show_bones: false }), &sol);
            fwrite(out, svg)?;
            println!("wrote {out} ({w}x{h}, {} containers, {} layers placed)", sol.rects.len(), sol.layers.len());
            Ok(())
        }
        ["pick", path] => {
            let e = load(path)?;
            let frame = num(a, "--frame", 0.0)?;
            let p = Vec2::new(num(a, "--x", f32::NAN)?, num(a, "--y", f32::NAN)?);
            if !p.x.is_finite() || !p.y.is_finite() {
                return Err("pick needs --x and --y".into());
            }
            let o = pick_opts(a)?;
            let fr = e.evaluate(frame);
            let hits = osatura_core::pick::pick_all(&e, &fr, p, &o);
            if a.flags.iter().any(|f| f == "--json") {
                println!("{}", serde_json::json!({"hit": hits.first(), "hits": hits}));
            } else if hits.is_empty() {
                println!("nothing at ({}, {}) on frame {frame}", p.x, p.y);
            } else {
                for (i, h) in hits.iter().enumerate() {
                    println!("{}{} (distance {:.2})", if i == 0 { "hit: " } else { "  under: " }, describe_target(&h.target), h.distance);
                }
            }
            Ok(())
        }
        ["select-box", path] => {
            use osatura_core::pick::{MarqueeMode, Region};
            let e = load(path)?;
            let frame = num(a, "--frame", 0.0)?;
            let o = pick_opts(a)?;
            let mode = match a.opts.get("--mode") {
                None => MarqueeMode::Intersect,
                Some(m) => MarqueeMode::parse(m).ok_or_else(|| format!("--mode must be bbox, contain or intersect, got '{m}'"))?,
            };
            let nums = |s: &str, sep: char| -> Result<Vec<f32>, String> { s.split(sep).map(|t| t.trim().parse::<f32>().map_err(|_| format!("bad number '{t}'"))).collect() };
            let region = if let Some(r) = a.opts.get("--rect") {
                let n = nums(r, ',')?;
                if n.len() != 4 {
                    return Err("--rect needs x0,y0,x1,y1".into());
                }
                Region::rect(Vec2::new(n[0], n[1]), Vec2::new(n[2], n[3]))
            } else if let Some(l) = a.opts.get("--lasso") {
                let mut pts = Vec::new();
                for pair in l.split(';') {
                    let n = nums(pair, ',')?;
                    if n.len() != 2 {
                        return Err("--lasso needs x,y;x,y;x,y".into());
                    }
                    pts.push(Vec2::new(n[0], n[1]));
                }
                if pts.len() < 3 {
                    return Err("--lasso needs at least 3 points".into());
                }
                Region::Lasso(pts)
            } else {
                return Err("select-box needs --rect or --lasso".into());
            };
            let t = osatura_core::pick::marquee(&e, &e.evaluate(frame), &region, mode, &o);
            if a.flags.iter().any(|f| f == "--json") {
                println!("{}", serde_json::json!({"targets": t}));
            } else if t.is_empty() {
                println!("nothing selected");
            } else {
                for x in &t {
                    println!("{}", describe_target(x));
                }
            }
            Ok(())
        }
        ["timeline", path] => {
            let p = load_project(path)?;
            let m = osatura_core::timeline::build(&p);
            if a.flags.iter().any(|f| f == "--json") {
                println!("{}", serde_json::to_string(&m).map_err(|e| e.to_string())?);
            } else {
                print!("{}", timeline_text(&m));
            }
            Ok(())
        }
        ["keyedit", path, out, sub] => keyedit_cmd(a, path, out, sub),
        #[cfg(feature = "gpu")]
        ["render-gpu", path, out] => render_gpu_cmd(a, path, out),
        #[cfg(not(feature = "gpu"))]
        ["render-gpu", ..] => Err("render-gpu needs a build with the gpu feature: cargo build -p osatura-cli --features gpu".to_string()),
        ["collab-sim", path] => collab_sim(a, path),
        ["vcs", rest @ ..] => vcs_cmd::run(rest, &a.opts, &a.flags),
        ["fuzz", path] => fuzz_cmd::fuzz(path, &a.opts, &a.flags),
        ["fuzz-grid", path, out] => fuzz_cmd::grid(path, out, &a.opts, &a.flags),
        ["ci", ..] => ci_cmd::run(raw),
        ["export-fit", path, out] => fit_cmd::export_fit(path, out, &a.opts, &a.flags),
        ["size-report", path] => fit_cmd::size_report(path, &a.opts, &a.flags),
        ["lod", path] => lod_cmd::lod(path, a.scale, &a.opts, &a.flags),
        ["lod-pick", path] => lod_cmd::pick(path, &a.opts),
        ["face-train"] => face_cmd::train(&a.opts),
        ["face-eval"] => face_cmd::eval(&a.opts, &a.flags),
        ["face-detect", image] => face_cmd::detect(image, &a.opts, &a.flags),
        ["face-track", input, out] => face_cmd::track(input, out, &a.opts),
        ["face-synth", dir] => face_cmd::synth(dir, &a.opts),
        ["collab-token", name] => {
            let secret = collab_srv::secret_from(a.opts.get("--secret-file").map(String::as_str))?;
            let ttl = num(a, "--ttl", 28800.0)?;
            if !matches!(ttl.partial_cmp(&1.0), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) || ttl.fract() != 0.0 {
                return Err("--ttl needs a whole number of seconds (1 to 2592000)".into());
            }
            let now = osatura_core::clock::unix_seconds();
            let token = osatura_core::collabauth::mint_token(&secret, name, now, ttl as u64)?;
            println!("{token}");
            eprintln!("token for '{name}', valid until Unix time {} (in {} s); give it to that person only; send it as the X-Collab-Token header or enter it on the login page", now + ttl as u64, ttl as u64);
            Ok(())
        }
        ["color-space", path, out, space] => {
            let mut p = load_project(path)?;
            p.color_space = osatura_core::project::ColorSpace::parse(space)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} (color_space: {})", if p.color_space.is_default() { "srgb" } else { "linear" });
            Ok(())
        }
        ["reparent", path, out, bone, parent] => {
            let mut p = load_project(path)?;
            rigedit::set_parent(&mut p, bone, if *parent == "none" { None } else { Some(parent) })?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["comp", "list", path] | ["comp", "list", path, _] => {
            // the 4th word is accepted for old scripts (`comp list p.json p.json`); nothing is written
            for c in &load_project(path)?.comps {
                println!("{} ({} layers)", c.name, c.layers.len());
            }
            Ok(())
        }
            _ => {
                handled.set(false);
                Ok(())
            }
        }
    })();
    handled.get().then_some(r)
}
