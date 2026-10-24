// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Commands part 4 of the dispatcher in `main.rs` (arms kept in their original order, so the first matching arm still wins): script, audio-react, beats, waveform, skin, parent-switch, bone-vis, face-calibrate, face-bake, live-map, clip, cage, cage-pose, mesh-trace, warp-pivot, layer-pivot, mesh-lock, glue.
#![allow(unused_imports)]
use super::*;

/// `Some(result)` when one of this part's arms matched the command line, `None` to try the next part.
pub(crate) fn run(a: &Args, raw: &[String], pos: &[&str]) -> Option<Result<(), String>> {
    let _ = raw;
    let handled = std::cell::Cell::new(true);
    let r = (|| -> Result<(), String> {
        match pos {
        #[cfg(not(feature = "scripting"))]
        ["script", ..] => Err("this build has no scripting support (built without the `scripting` feature)".to_string()),
        #[cfg(feature = "scripting")]
        ["script", path, out, file] => {
            let p = load_project(path)?;
            let src = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
            let mut ed = Editor::with_asset_dir(p, asset_dir(path)).map_err(|errs| format!("{path} is not a valid project:\n  - {}", errs.join("\n  - ")))?;
            let r = osatura_core::script::run_script(&mut ed, &src)?;
            for l in &r.log { println!("{l}"); }
            if r.applied == 0 {
                eprintln!("warning: script: 0 requests were applied (the script changed nothing), so {out} is a copy of the input");
            }
            write_json(ed.project(), out)?;
            println!("{} request(s) applied; wrote {out}", r.applied);
            Ok(())
        }
        ["audio-react", path, out, wav, target, prop, rest @ ..] => {
            let mut p = load_project(path)?;
            let audio = read_wav_file(wav)?;
            let pr = Prop::parse(prop).ok_or_else(|| format!("unknown property '{prop}'"))?;
            let num = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            let mut o = audiomap::LevelOptions::default();
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "gain" => o.gain = num(v)?, "offset" => o.offset = num(v)?, "floor" => o.floor = num(v)?,
                    "attack" => o.attack = num(v)?, "release" => o.release = num(v)?, "add" => o.additive = v == "on",
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            let ch = audiomap::level_channel(target, pr, &audio, p.fps, p.frames, &o)?;
            chanmeta::assert_unlocked(&p, target, pr)?;
            p.channels.push(ch);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["beats", path, out, wav, rest @ ..] => {
            let mut p = load_project(path)?;
            let audio = read_wav_file(wav)?;
            let (mut sens, mut gap, mut as_events, mut name) = (1.5f32, 4u32, false, "beat".to_string());
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "sensitivity" => sens = v.parse().map_err(|_| "bad sensitivity".to_string())?,
                    "gap" => gap = v.parse().map_err(|_| "bad gap".to_string())?,
                    "as" => as_events = match v { "events" => true, "markers" => false, _ => return Err("as is markers or events".into()) },
                    "name" => name = v.to_string(),
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            let b = audiomap::detect_beats(&audio, p.fps, p.frames, sens, gap)?;
            if as_events { audiomap::beats_to_events(&mut p, &b, &name) } else { audiomap::beats_to_markers(&mut p, &b, &name) }
            write_json(&p, out)?;
            println!("{} beats at frames {:?}; wrote {out}", b.len(), b);
            Ok(())
        }
        ["waveform", wav, out] | ["waveform", wav, out, _] => {
            let audio = read_wav_file(wav)?;
            let n: usize = match pos.get(3) { Some(s) => s.trim_start_matches("buckets=").parse().map_err(|_| "bad bucket count".to_string())?, None => 800 };
            let peaks = audiomap::waveform_peaks(&audio, n)?;
            let json = serde_json::json!({"sample_rate": audio.sample_rate, "duration": audio.duration(), "peaks": peaks});
            fwrite(out, serde_json::to_string(&json).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            println!("wrote {out} ({n} buckets)");
            Ok(())
        }
        ["skin", path, out, action, group, rest @ ..] => {
            let mut p = load_project(path)?;
            match (*action, rest) {
                ("add", [name, layers]) => {
                    let ls: Vec<String> = layers.split(',').map(|s| s.trim().to_string()).collect();
                    skins::add_skin(&mut p, group, name, &ls)?;
                }
                ("rig", [name, constraints, iks]) => {
                    let list = |s: &str| -> Vec<String> { if s == "-" { vec![] } else { s.split(',').map(|x| x.trim().to_string()).collect() } };
                    skins::set_rig(&mut p, group, name, &list(constraints), &list(iks))?;
                }
                ("bones", [name, bones]) => {
                    let bs: Vec<String> = if *bones == "-" { vec![] } else { bones.split(',').map(|x| x.trim().to_string()).collect() };
                    skins::set_bones(&mut p, group, name, &bs)?;
                }
                ("apply", [name]) => skins::apply_skin(&mut p, group, name)?,
                ("schedule", [steps]) => {
                    let mut v = Vec::new();
                    for part in steps.split(',') {
                        let (f, n) = part.split_once(':').ok_or("step must be frame:name")?;
                        v.push((f.trim().parse::<f32>().map_err(|_| "bad frame".to_string())?, n.trim().to_string()));
                    }
                    skins::schedule(&mut p, group, &v)?;
                }
                _ => return Err("usage: skin <project> <out> add <group> <name> <layers> | rig <group> <name> <constraints|-> <iks|-> | bones <group> <name> <bones|-> | apply <group> <name> | schedule <group> <frame:name,...>".into()),
            }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["parent-switch", path, out, bone, steps] => {
            let mut p = load_project(path)?;
            let mut v = Vec::new();
            for part in steps.split(',') {
                let (f, n) = part.split_once(':').ok_or("step must be frame:parent")?;
                v.push(parentswitch::Switch { frame: f.trim().parse().map_err(|_| "bad frame".to_string())?, parent: if n.trim() == "none" { None } else { Some(n.trim().to_string()) } });
            }
            parentswitch::bake_parent_switch(&mut p, bone, &v)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["bone-vis", path, out, bone, from, to] => {
            let mut p = load_project(path)?;
            bonevis::set_range(&mut p, bone, from.parse().map_err(|_| "bad from".to_string())?, to.parse().map_err(|_| "bad to".to_string())?)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["bone-vis", path, out, bone, "clear"] => {
            let mut p = load_project(path)?;
            println!("{}", if bonevis::clear(&mut p, bone) { "range removed" } else { "no range set" });
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["face-calibrate", marks, out] => {
            let (stream, cfg) = face_inputs(a, marks, None, 30.0)?;
            let n = num(a, "--calibrate-frames", 30.0)? as usize;
            let cal = osatura_core::faceparam::calibrate(&stream, &cfg, n)?;
            fwrite(out, serde_json::to_string_pretty(&cal).map_err(|e| e.to_string())?)?;
            println!("{} frames, neutral from the first {n}; wrote {out}", stream.frames.len());
            Ok(())
        }
        ["face-bake", path, marks, map, out] => {
            let mut p = load_project(path)?;
            let (stream, cfg) = face_inputs(a, marks, Some(map), p.fps)?;
            let (mapping, _) = osatura_core::faceparam::parse_mapfile(&fs::read_to_string(map).map_err(|e| format!("cannot read {map}: {e}"))?)?;
            let cal = match (a.opts.get("--calib"), num(a, "--calibrate-frames", 30.0)? as usize) {
                (Some(f), _) => osatura_core::faceparam::Calibration::from_json(&fs::read_to_string(f).map_err(|e| format!("cannot read {f}: {e}"))?)?,
                (None, 0) => osatura_core::faceparam::Calibration::nominal(),
                (None, n) => osatura_core::faceparam::calibrate(&stream, &cfg, n)?,
            };
            let r = osatura_core::faceparam::bake(&mut p, &stream, &mapping, &cfg, &cal, num(a, "--tol", 0.01)?)?;
            for n in &r.notes { println!("note: {n}"); }
            check_bake_not_empty(a, stream.frames.len(), &r, "face-bake", "mapfile 'params' (source \"face\", signal id)")?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("{} landmark frames -> +{} channels ({} keys in them), {} trigger keys set, {} events; wrote {out}", stream.frames.len(), r.channels, r.param_keys, r.keys_set, r.events);
            if r.cut_frames > 0 {
                eprintln!("warning: face-bake: the landmark stream is {} frame(s) longer than the project ({} frames at {} fps); those frames were not baked (the project length is unchanged)", r.cut_frames, p.frames, p.fps);
            }
            Ok(())
        }
        ["live-map", path, events, map, out] => {
            let mut p = load_project(path)?;
            let (ev, warns) = if events.ends_with(".mid") || events.ends_with(".midi") {
                (osatura_core::midi::smf_to_events(&fs::read(events).map_err(|e| format!("cannot read {events}: {e}"))?)?, vec![])
            } else {
                osatura_core::liveinput::parse_recording_json(&fs::read_to_string(events).map_err(|e| format!("cannot read {events}: {e}"))?)?
            };
            for w in &warns { println!("warning: {w}"); }
            let mapping: osatura_core::liveinput::Mapping = read_json(map)?;
            let lib = match a.opts.get("--takes") {
                Some(l) => Some(load_library(l)?),
                None => None,
            };
            let r = osatura_core::liveinput::bake(&mut p, &ev, &mapping, lib.as_ref(), num(a, "--tol", 0.01)?)?;
            for n in &r.notes { println!("note: {n}"); }
            check_bake_not_empty(a, ev.len(), &r, "live-map", "mapping.json 'params'/'triggers' (source, id)")?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("{} events -> +{} channels ({} keys in them), {} trigger keys set, {} takes, {} events ({} after the last frame skipped); wrote {out}", ev.len(), r.channels, r.param_keys, r.keys_set, r.takes, r.events, r.skipped);
            if r.cut_frames > 0 {
                eprintln!("warning: live-map: the recording is {} frame(s) longer than the project ({} frames at {} fps); those frames were not baked (the project length is unchanged)", r.cut_frames, p.frames, p.fps);
            }
            Ok(())
        }
        ["clip", "add", path, out, lib, take, name] => {
            let mut p = load_project(path)?;
            let l = load_library(lib)?;
            let list = |k: &str| -> Vec<String> { a.opts.get(k).map(|s| s.split(',').filter(|x| !x.is_empty()).map(str::to_string).collect()).unwrap_or_default() };
            let mut c = clipstack::Clip::new(name, motionlib::Take { name: take.to_string(), length: 1.0, tracks: vec![] });
            c.start = num(a, "--at", 0.0)?;
            c.speed = num(a, "--speed", 1.0)?;
            c.loops = a.opts.get("--loops").map(|v| v.parse::<u32>().map_err(|_| format!("--loops needs a whole number, got '{v}'"))).transpose()?.unwrap_or(1);
            c.weight = num(a, "--weight", 1.0)?;
            c.fade_in = num(a, "--fade-in", 0.0)?;
            c.fade_out = num(a, "--fade-out", 0.0)?;
            c.mode = match a.opts.get("--mode").map(String::as_str) {
                None | Some("replace") => clipstack::ClipMode::Replace,
                Some("additive") | Some("add") => clipstack::ClipMode::Additive,
                Some("multiply") => clipstack::ClipMode::Multiply,
                Some(m) => return Err(format!("unknown clip mode '{m}' (replace, additive, multiply)")),
            };
            c.include = list("--include");
            c.exclude = list("--exclude");
            c.rename = list("--rename").iter().filter_map(|kv| kv.split_once('=')).map(|(x, y)| (x.to_string(), y.to_string())).collect();
            for r in list("--track-weight") {
                let (pat, w) = r.split_once('=').ok_or("--track-weight needs pattern=weight")?;
                let (pat, prop) = parse_pattern_prop(pat)?;
                c.track_weights.push(clipstack::TrackWeight { pattern: pat, prop, weight: w.parse().map_err(|_| format!("bad weight '{w}'"))? });
            }
            clipstack::add(&mut p, &l, take, c)?;
            write_json(&p, out)?;
            println!("added clip '{name}' ({} clips); run clip bake to generate channels", p.clips.len());
            Ok(())
        }
        ["clip", "list", path] => {
            let p = load_project(path)?;
            for (i, c) in p.clips.iter().enumerate() {
                let mut extra = String::new();
                if !c.weight_keys.keys.is_empty() { extra += &format!(" weight-keys {}", c.weight_keys.keys.len()); }
                if !c.include.is_empty() { extra += &format!(" include {}", c.include.join(",")); }
                if !c.exclude.is_empty() { extra += &format!(" exclude {}", c.exclude.join(",")); }
                for r in &c.track_weights { extra += &format!(" {}{}={}", r.pattern, r.prop.map(|p| format!(":{p:?}")).unwrap_or_default(), r.weight); }
                println!("{}. {} take '{}' frames {}..{} x{} speed {} {:?} weight {}{extra}", i + 1, c.name, c.take.name, c.start, c.end(), c.loops, c.speed, c.mode, c.weight);
            }
            if p.clips.is_empty() { println!("no clips"); }
            println!("{} baked channels", p.clip_baked.len());
            Ok(())
        }
        ["clip", "remove", path, out, name] => {
            let mut p = load_project(path)?;
            clipstack::remove(&mut p, name)?;
            write_json(&p, out)?;
            println!("removed clip '{name}'; run clip bake to update channels");
            Ok(())
        }
        ["clip", "set-weight", path, out, name] | ["clip", "set-weight", path, out, name, _] => {
            let mut p = load_project(path)?;
            let w: Option<f32> = pos.get(5).map(|s| s.parse().map_err(|_| format!("bad weight '{s}'"))).transpose()?;
            if let Some(k) = a.opts.get("--keys") {
                let mut keys = Vec::new();
                for part in k.split(',') {
                    let (f, v) = part.split_once(':').ok_or("--keys needs frame:weight pairs")?;
                    keys.push((f.parse::<f32>().map_err(|_| format!("bad frame '{f}'"))?, v.parse::<f32>().map_err(|_| format!("bad weight '{v}'"))?));
                }
                clipstack::set_weight_keys(&mut p, name, &keys)?;
            } else if let Some(t) = a.opts.get("--track") {
                let (pat, prop) = parse_pattern_prop(t)?;
                clipstack::set_track_weight(&mut p, name, &pat, prop, w.ok_or("--track needs a weight")?)?;
            } else {
                clipstack::set_weight(&mut p, name, w.ok_or("give a weight or --keys")?)?;
            }
            write_json(&p, out)?;
            println!("updated clip '{name}'; run clip bake to update channels");
            Ok(())
        }
        ["clip", "bake", path, out] => {
            let mut p = load_project(path)?;
            let r = clipstack::bake(&mut p)?;
            write_json(&p, out)?;
            println!("baked {} clips into {} channels ({} replaced); wrote {out}", r.clips, r.channels, r.replaced);
            Ok(())
        }
        ["cage", path, out, layer, rest @ ..] => {
            let mut p = load_project(path)?;
            let mut o = cage::CageOptions::default();
            let num = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            for kv in rest {
                let (k, v) = kv.split_once('=').map(|(k, v)| (k, v.to_string())).unwrap_or((&kv[..], String::new()));
                match k {
                    "auto" => o.rect = None,
                    "hull" => o.hull = true,
                    "rect" => {
                        let f: Vec<&str> = v.split(',').collect();
                        if f.len() != 4 { return Err("rect is x,y,w,h".into()); }
                        o.rect = Some([num(f[0])?, num(f[1])?, num(f[2])?, num(f[3])?]);
                    }
                    "verts" => o.verts = v.parse::<usize>().map_err(|_| format!("bad whole number '{v}'"))?,
                    "pad" => o.pad = num(&v)?,
                    "name" => o.name = v,
                    other => return Err(format!("unknown option '{other}' (rect=, auto, hull, verts=, pad=, name=)")),
                }
            }
            p = cage::add_cage(&p, layer, &o)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            let n = match p.find_layer(layer).and_then(|l| l.deformers.iter().find(|d| d.name() == o.name)) {
                Some(osatura_core::project::Deformer::Cage { rest, .. }) => rest.len(),
                _ => 0,
            };
            println!("added cage '{}' ({n} vertices) to '{layer}'; wrote {out}", o.name);
            Ok(())
        }
        ["cage-pose", path, out, layer, frame, rest @ ..] => {
            let p = load_project(path)?;
            let fr: f32 = frame.parse().map_err(|_| format!("bad frame '{frame}'"))?;
            let (mut name, mut moves, mut all, mut ease) = ("cage".to_string(), Vec::new(), None, osatura_core::anim::Ease::Linear);
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("bad option '{kv}' (use name=, move=i:dx,dy;..., all=dx,dy, ease=)"))?;
                match k {
                    "name" => name = v.to_string(),
                    "move" => moves = cage::parse_moves(v)?,
                    "all" => {
                        let (a, b) = v.split_once(',').ok_or("all is dx,dy")?;
                        all = Some((a.trim().parse::<f32>().map_err(|_| format!("bad number '{a}'"))?, b.trim().parse::<f32>().map_err(|_| format!("bad number '{b}'"))?));
                    }
                    "ease" => ease = serde_json::from_str(&format!("{{\"type\":\"{v}\"}}")).map_err(|_| format!("unknown ease '{v}'"))?,
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            let q = cage::pose_cage(&p, layer, &name, fr, &moves, all, ease)?;
            compile(q.clone(), path)?;
            write_json(&q, out)?;
            println!("keyed cage '{name}' on '{layer}' at frame {fr}; wrote {out}");
            Ok(())
        }
        ["mesh-trace", path, out, layer, rest @ ..] => {
            let mut p = load_project(path)?;
            let mut o = meshtrace::TraceOptions::default();
            let (mut skin, mut bones_r, mut influences) = (true, None, 4usize);
            let num = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "density" => o.density = num(v)?,
                    "tolerance" => o.tolerance = num(v)?,
                    "edge" => o.edge_size = num(v)?,
                    "grading" => o.grading = num(v)?,
                    "threshold" => o.threshold = num(v)?.clamp(0.0, 255.0) as u8,
                    "smooth" => o.smooth.iterations = v.parse().map_err(|_| format!("smooth needs a whole number, got '{v}'"))?,
                    "lambda" => o.smooth.lambda = num(v)?,
                    "taubin" => o.smooth.taubin = v != "0",
                    "smooth-boundary" => o.smooth.boundary = v != "0",
                    "hot-size" => o.hotspot_size = num(v)?,
                    "bones" => bones_r = Some(num(v)?),
                    "skin" => skin = v != "0",
                    "influences" => influences = v.parse().map_err(|_| format!("influences needs a whole number, got '{v}'"))?,
                    "hotspot" => {
                        for h in v.split(';').filter(|h| !h.is_empty()) {
                            let f: Vec<&str> = h.split(',').collect();
                            if f.len() != 3 {
                                return Err(format!("hotspot '{h}' must be x,y,r"));
                            }
                            o.hotspots.push(meshtrace::Hotspot { pos: osatura_core::math::Vec2::new(num(f[0])?, num(f[1])?), radius: num(f[2])? });
                        }
                    }
                    _ => return Err(format!("unknown mesh-trace option '{k}'")),
                }
            }
            if let Some(r) = bones_r {
                o.hotspots.extend(meshtrace::bone_hotspots(&p, r)?);
            }
            let st = meshtrace::trace_layer_mesh(&mut p, layer, &o, &asset_dir(path), skin, influences)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!(
                "wrote {out}: {} vertices ({} on the outline), {} triangles, area {:.0} px2\n  quality min {:.3} mean {:.3}, min angle {:.1} deg, mean edge {:.1} px at the outline / {:.1} px inside{}",
                st.vertices, st.boundary_vertices, st.triangles, st.area, st.min_quality, st.mean_quality, st.min_angle_deg, st.mean_boundary_edge, st.mean_interior_edge,
                if st.unresolved_edges > 0 { format!("\n  warning: {} outline edges could not be recovered", st.unresolved_edges) } else { String::new() }
            );
            Ok(())
        }
        ["warp-pivot", path, out, layer, frame, rest @ ..] | ["soft-scale", path, out, layer, frame, rest @ ..] => {
            let soft = pos[0] == "soft-scale";
            let p = load_project(path)?;
            let fr: f32 = frame.parse().map_err(|_| format!("bad frame '{frame}'"))?;
            let num = |s: &str| s.trim().parse::<f32>().ok().filter(|v| v.is_finite()).ok_or_else(|| format!("bad number '{s}'"));
            let mut o = warp::WarpOptions { frame: fr, ..Default::default() };
            let (mut resize, mut keep, mut sel_given) = (None, true, false);
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "deformer" => o.deformer = Some(v.to_string()),
                    "verts" => {
                        o.selection.indices = links::parse_indices(v)?;
                        sel_given = true;
                    }
                    "region" => {
                        o.selection.region = Some(warp::parse_region(v)?);
                        sel_given = true;
                    }
                    "pivot" => o.pivot = warp::Pivot::parse(v)?,
                    "rotate" => o.xform.rotate_deg = num(v)?,
                    "scale" => {
                        o.xform.scale = warp::parse_scale(v)?;
                        resize = Some(warp::ResizeTo::Factor(num(v.split(',').next().unwrap_or(v))?));
                    }
                    "move" => {
                        let (a, b) = v.split_once(',').ok_or("move is dx,dy")?;
                        o.xform.translate = osatura_core::math::Vec2::new(num(a)?, num(b)?);
                    }
                    "radius" => o.radius = num(v)?,
                    "falloff" => o.falloff = warp::Falloff::parse(v)?,
                    "width" => resize = Some(warp::ResizeTo::Width(num(v)?)),
                    "height" => resize = Some(warp::ResizeTo::Height(num(v)?)),
                    "size" => {
                        let (a, b) = v.split_once('x').ok_or("size is WxH")?;
                        resize = Some(warp::ResizeTo::Size(num(a)?, num(b)?));
                    }
                    "aspect" => keep = match v { "keep" => true, "free" => false, _ => return Err("aspect is keep or free".into()) },
                    "ease" => o.ease = serde_json::from_str(&format!("{{\"type\":\"{v}\"}}")).map_err(|_| format!("unknown ease '{v}'"))?,
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            let q = if soft && !sel_given {
                let to = resize.ok_or("soft-scale without verts=/region= resizes the whole mesh: give scale=, width=, height= or size=")?;
                warp::resize_mesh(&p, layer, o.deformer.as_deref(), to, keep, fr, o.ease)?
            } else {
                warp::warp_pivot(&p, layer, &o)?
            };
            compile(q.clone(), path)?;
            write_json(&q, out)?;
            println!("{} on '{layer}' keyed at frame {fr}; wrote {out}", pos[0]);
            Ok(())
        }
        ["layer-pivot", path, out, layer, rest @ ..] => {
            let p = load_project(path)?;
            let mut piv = None;
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "pivot" => piv = Some(warp::Pivot::parse(v)?),
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            let q = warp::set_layer_pivot(&p, layer, &piv.ok_or("layer-pivot needs pivot=x,y or pivot=bone:NAME")?)?;
            compile(q.clone(), path)?;
            write_json(&q, out)?;
            println!("moved the pivot of '{layer}'; wrote {out}");
            Ok(())
        }
        ["mesh-lock", "list", path] => {
            let p = load_project(path)?;
            for l in &p.mesh_locks {
                println!("{}: {}:{} follows {}:{} vertices {} strength {}", l.name, l.follower.layer, l.follower.path, l.host.layer, l.host.path,
                    if l.vertices.is_empty() { "all".to_string() } else { format!("{:?}", l.vertices) }, l.strength);
            }
            println!("{} mesh lock(s)", p.mesh_locks.len());
            Ok(())
        }
        ["mesh-lock", "remove", path, out, name] => {
            let q = links::remove_mesh_lock(&load_project(path)?, name)?;
            write_json(&q, out)?;
            println!("removed mesh lock '{name}'; wrote {out}");
            Ok(())
        }
        ["mesh-lock", "add", path, out, name, rest @ ..] => {
            let p = load_project(path)?;
            let (mut host, mut fol, mut verts, mut strength) = (None, None, Vec::new(), 1.0f32);
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "host" => host = Some(links::parse_geom(v)?),
                    "follower" => fol = Some(links::parse_geom(v)?),
                    "verts" => verts = links::parse_indices(v)?,
                    "strength" => strength = v.parse().map_err(|_| format!("bad number '{v}'"))?,
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            let l = links::MeshLock { name: name.to_string(), host: host.ok_or("mesh-lock add needs host=")?, follower: fol.ok_or("mesh-lock add needs follower=")?, vertices: verts, strength };
            let q = links::add_mesh_lock(&p, l)?;
            compile(q.clone(), path)?;
            write_json(&q, out)?;
            println!("added mesh lock '{name}'; wrote {out}");
            Ok(())
        }
        ["glue", "list", path] => {
            let p = load_project(path)?;
            for g in &p.glues {
                let pr: Vec<String> = g.pairs.iter().map(|x| format!("{}:{}#{}->{}:{}#{}", x.a.layer, x.a.path, x.a.index, x.b.layer, x.b.path, x.b.index)).collect();
                println!("{}: {:?} strength {} {}", g.name, g.mode, g.strength, pr.join(" "));
            }
            println!("{} glue(s)", p.glues.len());
            Ok(())
        }
        ["glue", "remove", path, out, name] => {
            let q = links::remove_glue(&load_project(path)?, name)?;
            write_json(&q, out)?;
            println!("removed glue '{name}'; wrote {out}");
            Ok(())
        }
        ["glue", "add", path, out, name, rest @ ..] => {
            let p = load_project(path)?;
            let (mut a, mut b, mut pairs, mut auto, mut mode, mut strength) = (None, None, None, None, links::GlueMode::default(), 1.0f32);
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "a" => a = Some(links::parse_geom(v)?),
                    "b" => b = Some(links::parse_geom(v)?),
                    "pairs" => pairs = Some(links::parse_pairs(v)?),
                    "auto" => auto = Some(v.parse::<f32>().map_err(|_| format!("bad number '{v}'"))?),
                    "mode" => mode = links::GlueMode::parse(v).ok_or_else(|| format!("unknown mode '{v}' (rigid, snap, offset)"))?,
                    "strength" => strength = v.parse().map_err(|_| format!("bad number '{v}'"))?,
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            let (a, b) = (a.ok_or("glue add needs a=")?, b.ok_or("glue add needs b=")?);
            let list = match (pairs, auto) {
                (Some(_), Some(_)) => return Err("give pairs= or auto=, not both".into()),
                (Some(pr), None) => pr
                    .into_iter()
                    .map(|(i, j)| links::GluePair {
                        a: links::VertexRef { layer: a.layer.clone(), path: a.path, index: i },
                        b: links::VertexRef { layer: b.layer.clone(), path: b.path, index: j },
                    })
                    .collect(),
                (None, Some(r)) => links::auto_pairs(&p, &a, &b, r)?,
                (None, None) => return Err("glue add needs pairs=i:j;... or auto=RADIUS".into()),
            };
            let n = list.len();
            let q = links::add_glue(&p, links::Glue { name: name.to_string(), mode, pairs: list, strength })?;
            compile(q.clone(), path)?;
            write_json(&q, out)?;
            println!("added glue '{name}' ({n} pair(s)); wrote {out}");
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
