// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Commands part 2 of the dispatcher in `main.rs` (arms kept in their original order, so the first matching arm still wins): retarget, stats, diff, golden, lint, contracts, loop-solve, coach, timecode, keys, marker, idle, follow, import-svg, import-bvh, lipsync-text, speech-motion, mouth, tag, autorig, slider, view-swap, align, distribute, trim, shot, camera-seq.
#![allow(unused_imports)]
use super::*;

/// `Some(result)` when one of this part's arms matched the command line, `None` to try the next part.
pub(crate) fn run(a: &Args, raw: &[String], pos: &[&str]) -> Option<Result<(), String>> {
    let _ = raw;
    let handled = std::cell::Cell::new(true);
    let r = (|| -> Result<(), String> {
        match pos {
        ["retarget", src, dst, out] => {
            let sp = load_project(src)?;
            let mut dp = load_project(dst)?;
            let pairs: Vec<(String, String)> = a
                .opts
                .get("--pairs")
                .ok_or("--pairs srcChain=dstChain[,..] is required")?
                .split(',')
                .map(|s| s.split_once('=').map(|(x, y)| (x.trim().to_string(), y.trim().to_string())).ok_or_else(|| format!("bad pair '{s}'")))
                .collect::<Result<_, _>>()?;
            let (ch, notes) = perf::retarget_effectors(&sp.channels, &sp, &dp, &pairs)?;
            let n = ch.len();
            dp.channels.extend(ch);
            compile(dp.clone(), dst)?;
            write_json(&dp, out)?;
            for note in notes {
                println!("note: {note}");
            }
            println!("wrote {out} (+{n} channels)");
            Ok(())
        }
        ["stats", path] => {
            let p = load_project(path)?;
            println!("{}", serde_json::to_string_pretty(&inspect::stats(&p)).unwrap());
            Ok(())
        }
        ["diff", a_path, b_path] => {
            let d = inspect::diff(&load_project(a_path)?, &load_project(b_path)?);
            if d.is_empty() {
                println!("no differences");
                Ok(())
            } else {
                println!("{}", d.join("\n"));
                Err(format!("{} difference(s)", d.len()))
            }
        }
        ["golden", mode, path, gpath] => {
            let e = load(path)?;
            match *mode {
                "write" => {
                    // goldens are regression thumbnails of the CPU raster: a scale above 1 is refused (it used to be clamped silently)
                    if a.scale > 1.0 {
                        return Err(format!("golden write: --scale {} is above 1; goldens are recorded at a scale in (0, 1] (nothing was written)", a.scale));
                    }
                    let g = inspect::golden_record(&e, a.scale);
                    fwrite(gpath, serde_json::to_string(&g).unwrap()).map_err(|e| e.to_string())?;
                    println!("wrote {gpath} ({} frames)", g.frames.len());
                    Ok(())
                }
                "check" => {
                    let g: inspect::Golden = read_json(gpath)?;
                    // exact by default (the CPU rasteriser is deterministic); --tol T lets frames whose 16x16 thumbnail differs by at most T pass
                    let tol = num(a, "--tol", 0.0)?;
                    if !tol.is_finite() || tol < 0.0 {
                        return Err(format!("--tol needs a number >= 0 (0 = exact), got '{tol}'"));
                    }
                    let bad = inspect::golden_check(&e, &g, tol);
                    if bad.is_empty() {
                        println!("all {} frames match{}", g.frames.len(), if tol > 0.0 { format!(" (within thumbnail tolerance {tol})") } else { " exactly".to_string() });
                        Ok(())
                    } else {
                        for b in &bad {
                            if b.missing {
                                println!("frame {} exists on one side only (project has {} frames, golden {})", b.frame, e.project.frames, g.frames.len());
                            } else {
                                println!("frame {} differs: pixel hash changed, thumbnail distance {:.4}, {} of 256 thumbnail cells differ, max channel difference {}/255", b.frame, b.distance, b.cells_differing, b.max_diff);
                            }
                        }
                        Err(format!("{} frame(s) differ (exact comparison{}; --tol T accepts small differences)", bad.len(), if tol > 0.0 { format!(", tolerance {tol}") } else { String::new() }))
                    }
                }
                _ => Err("golden needs 'write' or 'check'".into()),
            }
        }
        ["lint", path] => {
            let p = load_project(path)?;
            for w in unknown_field_warnings(path) {
                println!("Warning unknown-field [{path}] {w} (ignored)");
            }
            let f = lint::lint(&p, Some(&asset_dir(path)));
            for x in &f {
                println!("{:?} {} [{}] {}", x.severity, x.code, x.target, x.message);
            }
            let mut errors = f.iter().filter(|x| x.severity == lint::Severity::Error).count();
            let mut total = f.len();
            // the same unusable-sound-track check as `check` (a missing, empty or oversized file, a bad format)
            for s in &p.sounds {
                if let Err(err) = soundtrack::check_file(&asset_dir(path), &s.file) {
                    println!("Error sound-unusable [{}] {err}", s.name);
                    errors += 1;
                    total += 1;
                }
            }
            println!("{total} finding(s), {errors} error(s)");
            if errors > 0 { Err("lint errors".into()) } else { Ok(()) }
        }
        ["contracts", path, cpath] => {
            let e = load(path)?;
            let contracts: Vec<lint::Contract> = read_json(cpath)?;
            let v = lint::check_contracts(&e, &contracts)?;
            if v.is_empty() {
                println!("all {} contract(s) hold", contracts.len());
                Ok(())
            } else {
                for x in &v {
                    println!("contract {} violated at frame {}: {}", x.contract, x.frame, x.message);
                }
                Err(format!("{} violation(s)", v.len()))
            }
        }
        ["loop-solve", path, out] => {
            let mut p = load_project(path)?;
            let r = lint::solve_loop_mode(&mut p, num(a, "--window", 24.0)?, if a.flags.iter().any(|f| f == "--inclusive") { looper::LoopMode::Inclusive } else { looper::LoopMode::Exclusive })?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}: {} channel(s) adjusted, seam {:.3} -> {:.3}", r.channels_adjusted, r.seam_before, r.seam_after);
            Ok(())
        }
        ["coach", path] => {
            let adv = lint::coach(&load_project(path)?);
            if adv.is_empty() {
                println!("no suggestions");
            }
            for x in &adv {
                println!("{} [{}] frame {}: {}", x.code, x.target, x.frame, x.message);
            }
            Ok(())
        }
        ["timecode", frame] => {
            let fps: f64 = match a.opts.get("--fps") {
                None => 24.0,
                Some(v) => v.parse().map_err(|_| format!("--fps needs a number, got '{v}'"))?,
            };
            let drop = a.flags.iter().any(|f| f == "--drop");
            let nodrop = a.flags.iter().any(|f| f == "--nodrop");
            if drop && nodrop {
                return Err("--drop and --nodrop exclude each other".into());
            }
            let mode = if drop { keyops::TcMode::Drop } else if nodrop { keyops::TcMode::NoDrop } else { keyops::TcMode::Auto };
            if frame.contains(':') || frame.contains(';') {
                println!("{}", keyops::parse_timecode_mode(frame, fps, mode)?);
            } else {
                let f: u32 = frame.parse().map_err(|_| format!("bad frame '{frame}'"))?;
                println!("{}", keyops::timecode_mode(f, fps, mode)?);
            }
            Ok(())
        }
        ["keys", op, path, out] => {
            let mut p = load_project(path)?;
            let (from, to) = (num(a, "--from", f32::MIN)?, num(a, "--to", f32::MAX)?);
            let want_target = a.opts.get("--target");
            let want_prop: Option<Prop> = match a.opts.get("--prop") {
                Some(s) => Some(serde_json::from_str(&format!("\"{s}\"")).map_err(|_| format!("unknown property '{s}'"))?),
                None => None,
            };
            let ease: Option<osatura_core::anim::Ease> = match a.opts.get("--ease") {
                Some(s) => Some(serde_json::from_str(&format!("{{\"type\":\"{s}\"}}")).map_err(|_| format!("unknown ease '{s}'"))?),
                None => None,
            };
            let by = a.opts.get("--by").map(|_| num(a, "--by", 0.0)).transpose()?;
            let factor = a.opts.get("--factor").map(|_| num(a, "--factor", 1.0)).transpose()?;
            if !["shift", "scale", "ease", "reverse"].contains(op) {
                return Err(format!("unknown keys operation '{op}' (shift, scale, ease, reverse)"));
            }
            match *op {
                "shift" if by.is_none() => return Err("keys shift needs --by N".into()),
                "scale" if factor.is_none() => return Err("keys scale needs --factor X".into()),
                "ease" if ease.is_none() => return Err("--ease is required".into()),
                _ => {}
            }
            let mut touched = 0;
            let locked_view = p.clone();
            for c in &mut p.channels {
                if want_target.is_some_and(|t| &c.target != t) || want_prop.is_some_and(|pr| c.prop != pr) {
                    continue;
                }
                let Source::Keys { track } = &mut c.source else { continue };
                if chanmeta::is_locked(&locked_view, &c.target, c.prop) {
                    println!("skipped locked channel {}.{:?}", c.target, c.prop);
                    continue;
                }
                touched += match *op {
                    "shift" => keyops::shift_keys(track, from, to, by.ok_or("keys shift needs --by N")?),
                    "scale" => keyops::scale_keys(track, from, to, num(a, "--pivot", 0.0)?, factor.ok_or("keys scale needs --factor X")?)?,
                    "ease" => keyops::set_ease(track, from, to, ease.ok_or("--ease is required")?),
                    "reverse" => keyops::reverse_keys(track, from, to),
                    other => return Err(format!("unknown keys operation '{other}' (shift, scale, ease, reverse)")),
                };
            }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} ({touched} key(s) changed)");
            Ok(())
        }
        ["marker", path, out, name, frame] => {
            let mut p = load_project(path)?;
            let f: f32 = frame.parse().map_err(|_| format!("bad frame '{frame}'"))?;
            let end = match a.opts.get("--end") {
                Some(v) => Some(v.parse::<f32>().map_err(|_| format!("bad --end '{v}'"))?),
                None => None,
            };
            p.markers.retain(|m| m.name != *name);
            p.markers.push(keyops::Marker { name: name.to_string(), frame: f, end });
            p.markers.sort_by(|x, y| x.frame.total_cmp(&y.frame));
            write_json(&p, out)?;
            println!("wrote {out} (marker '{name}' at {} = {})", f, keyops::timecode(f as u32, p.fps).unwrap_or_else(|e| e));
            Ok(())
        }
        ["idle", kind, path, out] => {
            let mut p = load_project(path)?;
            let target = a.opts.get("--target").ok_or("--target is required")?;
            let prop: Prop = {
                let s = a.opts.get("--prop").ok_or("--prop is required")?;
                serde_json::from_str(&format!("\"{s}\"")).map_err(|_| format!("unknown property '{s}'"))?
            };
            let ch = match *kind {
                "blink" => {
                    let d = procgen::BlinkOptions::default();
                    let o = procgen::BlinkOptions {
                        mean_interval_s: num(a, "--interval", d.mean_interval_s)?,
                        jitter: num(a, "--jitter", d.jitter)?,
                        closed_frames: num(a, "--closed", d.closed_frames)?,
                        seed: num(a, "--seed", d.seed as f32)? as u64,
                        ..d
                    };
                    procgen::blink_channel(target, prop, p.frames, p.fps, &o)?
                }
                "breath" => procgen::breath_channel(target, prop, num(a, "--amp", 0.03)?, num(a, "--period", p.fps * 3.0)?)?,
                "sway" => procgen::sway_channel(target, prop, num(a, "--amp", 2.0)?, num(a, "--freq", 0.3)?, num(a, "--seed", 1.0)? as u32),
                other => return Err(format!("unknown generator '{other}' (blink, breath, sway)")),
            };
            p.channels.push(ch);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["follow", path, out] => {
            let mut p = load_project(path)?;
            let target = a.opts.get("--target").ok_or("--target is required")?;
            let prop: Prop = {
                let s = a.opts.get("--prop").ok_or("--prop is required")?;
                serde_json::from_str(&format!("\"{s}\"")).map_err(|_| format!("unknown property '{s}'"))?
            };
            let src = p.channels.iter().find(|c| &c.target == target && c.prop == prop).cloned().ok_or("no such source channel")?;
            let bones: Vec<String> = a.opts.get("--chain").ok_or("--chain a,b,c is required")?.split(',').map(|s| s.trim().to_string()).collect();
            let chs = procgen::follow_through_within(&src, &bones, num(a, "--delay", 3.0)?, num(a, "--decay", 0.7)?, Some(p.frames as f32))?;
            let n = chs.len();
            p.channels.extend(chs);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} (+{n} channels)");
            Ok(())
        }
        ["import-svg", file, out] => {
            let text = fs::read_to_string(file).map_err(|e| format!("cannot read {file}: {e}"))?;
            let r = importers::import_svg(&text)?;
            let name = Path::new(file).file_stem().and_then(|n| n.to_str()).unwrap_or("svg");
            let (w, h) = if r.width > 0.0 && r.height > 0.0 { (r.width, r.height) } else { (512.0, 512.0) };
            let mut p = Project::new(name, w, h, 24.0, 24);
            if a.flags.iter().any(|f| f == "--split") {
                // one layer per shape, named after its SVG id (made unique), in document order: ready to be attached to bones
                let mut seen = std::collections::HashSet::new();
                p.layers = r
                    .paths
                    .iter()
                    .map(|path| {
                        let base = if path.name.is_empty() { "shape".to_string() } else { path.name.clone() };
                        let mut n = base.clone();
                        let mut k = 2;
                        while !seen.insert(n.clone()) {
                            n = format!("{base} {k}");
                            k += 1;
                        }
                        let mut path = path.clone();
                        path.name = n.clone();
                        osatura_core::project::Layer::vector(&n, vec![path])
                    })
                    .collect();
            } else {
                p.layers = vec![osatura_core::project::Layer::vector("svg", r.paths.clone())];
            }
            Engine::new(p.clone()).map_err(|errs| errs.join("; "))?;
            write_json(&p, out)?;
            for w in &r.warnings {
                println!("warning: {w}");
            }
            if a.flags.iter().any(|f| f == "--split") {
                println!("wrote {out} ({} paths, {} layers)", r.paths.len(), p.layers.len());
            } else {
                println!("wrote {out} ({} paths)", r.paths.len());
            }
            Ok(())
        }
        ["import-bvh", path, bvh, out] => {
            let mut p = load_project(path)?;
            let b = importers::parse_bvh(&fs::read_to_string(bvh).map_err(|e| format!("cannot read {bvh}: {e}"))?)?;
            let pairs: Vec<(String, String)> = a
                .opts
                .get("--joint-map")
                .ok_or("--joint-map Joint=bone[,..] is required")?
                .split(',')
                .map(|s| s.split_once('=').map(|(x, y)| (x.trim().to_string(), y.trim().to_string())).ok_or_else(|| format!("bad pair '{s}'")))
                .collect::<Result<_, _>>()?;
            let axis = a.opts.get("--axis").map(String::as_str).unwrap_or("Zrotation");
            let ch = importers::bvh_to_channels(&b, &pairs, axis, p.fps, p.frames, num(a, "--tol", 0.5)?)?;
            let n = ch.len();
            p.channels.extend(ch);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} (+{n} channels from {} BVH frames)", b.rows.len());
            Ok(())
        }
        ["lipsync-text", path, wav, transcript, layer, out] => {
            let mut p = load_project(path)?;
            let audio = read_wav_file(wav)?;
            let text = fs::read_to_string(transcript).map_err(|e| format!("cannot read {transcript}: {e}"))?;
            let lang = speech::Lang::parse(a.opts.get("--lang").map(|s| s.as_str()).unwrap_or("ro"))?;
            let mapping: [usize; 5] = match a.opts.get("--map") {
                None => osatura_core::audio::default_lipsync_map(osatura_core::audio::switch_children(&p, layer).unwrap_or(5)),
                Some(m) => {
                    let v: Vec<usize> = m.split(',').map(|s| s.trim().parse().map_err(|_| format!("bad --map entry '{s}'"))).collect::<Result<_, _>>()?;
                    v.try_into().map_err(|_| "--map needs exactly 5 numbers (rest,small,open,round,wide)".to_string())?
                }
            };
            let al = speech::align_transcript(&audio, &text, lang, &speech::AlignOptions::default())?;
            let n = speech::apply_lipsync(&mut p, layer, &al, lang, &mapping, &speech::LipOptions::default())?;
            compile(p.clone(), path)?;
            write_words(a, &al)?;
            write_json(&p, out)?;
            println!("wrote {out} ({} words, {} exposure changes; energy + text alignment, not phoneme recognition)", al.words.len(), n);
            Ok(())
        }
        ["speech-motion", path, wav, transcript, out] => {
            let mut p = load_project(path)?;
            let audio = read_wav_file(wav)?;
            let text = fs::read_to_string(transcript).map_err(|e| format!("cannot read {transcript}: {e}"))?;
            let lang = speech::Lang::parse(a.opts.get("--lang").map(|s| s.as_str()).unwrap_or("ro"))?;
            let mut o = speech::MotionOptions { head: a.opts.get("--head").cloned(), brow: a.opts.get("--brow").cloned(), ..Default::default() };
            let num = |k: &str, d: f32| -> Result<f32, String> { a.opts.get(k).map(|v| v.parse::<f32>().map_err(|_| format!("{k} needs a number"))).transpose().map(|x| x.unwrap_or(d)) };
            o.nod_deg = num("--nod", o.nod_deg)?;
            o.fall_deg = num("--fall", o.fall_deg)?;
            o.tilt_deg = num("--tilt", o.tilt_deg)?;
            o.brow_px = num("--brow-px", o.brow_px)?;
            if let Some(s) = a.opts.get("--seed") { o.seed = s.parse().map_err(|_| "--seed needs a whole number".to_string())?; }
            let al = speech::align_transcript(&audio, &text, lang, &speech::AlignOptions::default())?;
            let ch = speech::speech_motion(&p, &al, &o)?;
            let n = speech::apply_speech_motion(&mut p, ch)?;
            compile(p.clone(), path)?;
            write_words(a, &al)?;
            write_json(&p, out)?;
            println!("wrote {out} ({n} additive channel(s) from {} words)", al.words.len());
            Ok(())
        }
        ["mouth", path, cues, layer, out] => {
            let mut p = load_project(path)?;
            let text = fs::read_to_string(cues).map_err(|e| format!("cannot read {cues}: {e}"))?;
            let cs = importers::parse_mouth_cues(&text, p.fps)?;
            let mapping: [usize; 5] = match a.opts.get("--map") {
                None => [0, 1, 2, 3, 4],
                Some(m) => {
                    let v: Vec<usize> = m.split(',').map(|s| s.trim().parse().map_err(|_| format!("bad --map entry '{s}'"))).collect::<Result<_, _>>()?;
                    v.try_into().map_err(|_| "--map needs exactly 5 numbers (rest,small,open,round,wide)".to_string())?
                }
            };
            let ch = importers::mouth_channel(layer, &cs, &mapping, a.flags.iter().any(|f| f == "--rhubarb"));
            p.channels.retain(|c| !(c.target == *layer && c.prop == Prop::LayerSwitch && matches!(c.source, Source::Keys { .. })));
            p.channels.push(ch);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} ({} cues)", cs.len());
            Ok(())
        }
        ["tag", path] => {
            let p = load_project(path)?;
            for t in vocab::tag_rig(&p) {
                println!("{:<5} {:<24} -> {:<14} ({:.0}%)", t.kind, t.name, t.role, t.confidence * 100.0);
            }
            let missing = vocab::missing_bone_roles(&p);
            if !missing.is_empty() {
                println!("no bone found for: {}", missing.join(", "));
            }
            Ok(())
        }
        ["autorig", path, out] | ["autorig", path, out, _] => {
            let mut p = load_project(path)?;
            let seed: u64 = pos.get(3).map(|s| s.parse().map_err(|_| "seed must be a number".to_string())).transpose()?.unwrap_or(1);
            let rep = autorig::auto_rig(&mut p, seed)?;
            for c in &rep.created { println!("+ {c}"); }
            for s in &rep.skipped { println!("- {s}"); }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["slider", path, out, name, poses] => {
            let mut p = load_project(path)?;
            let mut list = Vec::new();
            for part in poses.split(';') {
                let (at, vals) = part.split_once(':').ok_or("pose needs at:values")?;
                let at: f32 = at.trim().parse().map_err(|_| "bad pose position".to_string())?;
                let mut values = Vec::new();
                for v in vals.split(',').filter(|v| !v.trim().is_empty()) {
                    let (tp, val) = v.split_once('=').ok_or("value needs target.prop=number")?;
                    let (target, prop) = tp.rsplit_once('.').ok_or("target.prop expected")?;
                    let prop: Prop = serde_json::from_str(&format!("\"{}\"", prop.trim())).map_err(|_| format!("unknown property '{prop}'"))?;
                    values.push((target.trim().to_string(), prop, val.trim().parse::<f32>().map_err(|_| "bad number".to_string())?));
                }
                list.push(drivers::PoseEntry { at, values });
            }
            let ctrl = drivers::slider_driver(&mut p, name, &list)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("controller bone: {ctrl} (animate its offset_x)\nwrote {out}");
            Ok(())
        }
        ["view-swap", path, out, layer, driver, ranges] => {
            let mut p = load_project(path)?;
            let (bone, prop) = driver.rsplit_once('.').ok_or("driver must be bone.prop")?;
            let prop: Prop = serde_json::from_str(&format!("\"{prop}\"")).map_err(|_| format!("unknown property '{prop}'"))?;
            let mut r = Vec::new();
            for part in ranges.split(',') {
                let (a, c) = part.split_once(':').ok_or("range needs from:child")?;
                r.push((a.trim().parse::<f32>().map_err(|_| "bad range start".to_string())?, c.trim().parse::<usize>().map_err(|_| "bad child index".to_string())?));
            }
            drivers::view_swap(&mut p, layer, bone, prop, &r)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["align", path, out, mode, layers] | ["align", path, out, mode, layers, _] => {
            let mut p = load_project(path)?;
            let m = layout::parse_align(mode).ok_or("mode: left right top bottom center-x center-y")?;
            let names: Vec<String> = layers.split(',').map(|s| s.trim().to_string()).collect();
            layout::align(&mut p, &names, m, pos.get(5) == Some(&"stage"))?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["distribute", path, out, axis, how, layers] => {
            let mut p = load_project(path)?;
            let ax = match *axis { "x" => layout::Axis::X, "y" => layout::Axis::Y, _ => return Err("axis: x or y".into()) };
            let h = match *how { "gaps" => layout::Distribute::Gaps, "centers" => layout::Distribute::Centers, _ => return Err("how: gaps or centers".into()) };
            let names: Vec<String> = layers.split(',').map(|s| s.trim().to_string()).collect();
            layout::distribute(&mut p, &names, ax, h)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["trim", path, out, layer, idx, a, b] | ["trim", path, out, layer, idx, a, b, _] => {
            let mut p = load_project(path)?;
            let num = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            let idx: usize = idx.parse().map_err(|_| "path_index must be a number".to_string())?;
            let off = pos.get(7).map(|s| num(s)).transpose()?.unwrap_or(0.0);
            let l = p.find_layer_mut(layer).ok_or_else(|| format!("no layer '{layer}'"))?;
            let osatura_core::project::LayerKind::Vector { paths } = &mut l.kind else { return Err("not a vector layer".into()) };
            let src = paths.get(idx).ok_or("path_index out of range")?;
            let t = osatura_core::layout::trim_path(src, num(a)?, num(b)?, off, 0.5)?;
            paths[idx] = t;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["shot", path, out, name, x, y, zoom] | ["shot", path, out, name, x, y, zoom, _] => {
            let mut p = load_project(path)?;
            let n = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            let rot = pos.get(8).map(|s| n(s)).transpose()?.unwrap_or(0.0);
            camshots::save_shot(&mut p, name, n(x)?, n(y)?, n(zoom)?, rot)?;
            write_json(&p, out)?;
            println!("wrote {out} ({} shots)", p.shots.len());
            Ok(())
        }
        ["camera-seq", path, out, steps] => {
            let mut p = load_project(path)?;
            let mut list = Vec::new();
            for part in steps.split(',') {
                let f: Vec<&str> = part.split(':').collect();
                if f.len() < 2 { return Err("step must be frame:shot[:cut]".into()); }
                list.push(camshots::Step { frame: f[0].parse().map_err(|_| "bad frame".to_string())?, shot: f[1].to_string(), cut: f.get(2) == Some(&"cut") });
            }
            camshots::sequence(&mut p, &list)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
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
