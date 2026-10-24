// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Commands part 1 of the dispatcher in `main.rs` (arms kept in their original order, so the first matching arm still wins): demo, demo-person, demo-fx, check, render, frames, export, lipsync, walk, mirror, export-oca, import-ora, safety, reduced-motion, pose-db, pose-bench, pose-suggest, autotrace-bench, autotrace, trace, skin-diffuse, drive, state-bake, pose-match.
#![allow(unused_imports)]
use super::*;

/// `Some(result)` when one of this part's arms matched the command line, `None` to try the next part.
pub(crate) fn run(a: &Args, raw: &[String], pos: &[&str]) -> Option<Result<(), String>> {
    let _ = raw;
    let handled = std::cell::Cell::new(true);
    let r = (|| -> Result<(), String> {
        match pos {
        ["demo", out] => {
            write_json(&demo::project(), out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["demo-person", out] => {
            write_json(&demo::person(), out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["demo-fx", out] => {
            write_json(&demo::fx_project(), out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["check", path] => {
            let e = load(path)?;
            for w in unknown_field_warnings(path) {
                println!("warning: {w} (ignored)");
            }
            let p = &e.project;
            for (layer, msg) in lint::gradient_warnings(p).into_iter().chain(lint::switch_key_warnings(p)) {
                println!("warning: {layer}: {msg}");
            }
            // parts rendering never reads (skins, comps, markers, events, shots, channel_meta, bone_vis, clips, lods) and values it clamps
            for w in osatura_core::editordata::warning_lines(p) {
                println!("warning: {w}");
            }
            let mut bad_sounds = Vec::new();
            for s in &p.sounds {
                if let Err(err) = soundtrack::check_file(&asset_dir(path), &s.file) {
                    bad_sounds.push(format!("sound '{}': {err}", s.name));
                }
            }
            if !bad_sounds.is_empty() {
                return Err(format!("{path} has {} unusable sound track(s):\n  - {}", bad_sounds.len(), bad_sounds.join("\n  - ")));
            }
            println!(
                "ok: '{}' {}×{} @ {} fps, {} frames, {} bones, {} ik, {} layers, {} channels",
                p.name, p.width, p.height, p.fps, p.frames, p.bones.len(), p.ik.len(), e.node_count(), p.channels.len()
            );
            Ok(())
        }
        ["render", path, frame, out] => {
            let e = load_sized(path, a)?;
            let frame: f32 = frame.parse().map_err(|_| format!("bad frame number '{frame}'"))?;
            let frame = check_frame_in_range(frame, e.project.frames, a.flags.iter().any(|f| f == "--clamp"))?;
            write_frame(&e, frame, Path::new(out), a)?;
            println!("wrote {out}");
            Ok(())
        }
        ["frames", path, dir] => {
            let e = load_sized(path, a)?;
            e.check_all_frames_finite()?;
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            let ext = if a.flags.iter().any(|f| f == "--png") { "png" } else { "svg" };
            for f in 0..e.project.frames {
                write_frame(&e, f as f32, &Path::new(dir).join(format!("frame_{f:04}.{ext}")), a)?;
            }
            println!("wrote {} frames to {dir}", e.project.frames);
            Ok(())
        }
        ["export", path, out] => {
            let e = load_sized(path, a)?;
            e.check_all_frames_finite()?;
            if matches!(Path::new(out).extension().and_then(|x| x.to_str()).map(str::to_ascii_lowercase).as_deref(), Some("json" | "lottie" | "html" | "htm" | "glb")) {
                export::check_budget(&e.project)?;
            }
            let ext = Path::new(out).extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
            if ext == "png" {
                guard_out(path, &Path::new(out).with_extension("atlas.json").to_string_lossy())?;
            }
            match ext.as_str() {
                "gif" => {
                    warn_silent(&e, "GIF");
                    let (bytes, warns) = export::gif_bytes_warn(&e, a.scale)?;
                    report_warnings(a, path, "gif", warns.iter().map(|w| ("frame-rate", String::new(), w.clone())).collect())?;
                    fwrite(out, bytes).map_err(|e| e.to_string())?
                }
                "html" | "htm" => {
                    let au = html_audio(&e, Path::new(path).parent().unwrap_or(Path::new(".")));
                    fwrite(out, export::html_player_with_audio(&e, au.as_ref().map(|(m, b)| (*m, b.as_slice())))).map_err(|e| e.to_string())?
                }
                "json" => {
                    let x = export::lottie_export(&e);
                    report_warnings(a, path, "lottie", x.warnings.iter().map(|w| (w.code, w.layer.clone(), w.message.clone())).collect())?;
                    fwrite(out, x.doc.to_string()).map_err(|e| e.to_string())?
                }
                "mp4" => export_mp4(&e, out, a.scale, Path::new(path).parent().unwrap_or(Path::new(".")))?,
                "lottie" => {
                    let (bytes, warns) = open_formats::dotlottie_bytes_report(&e)?;
                    report_warnings(a, path, "dotlottie", warns.iter().map(|w| (w.code, w.layer.clone(), w.message.clone())).collect())?;
                    fwrite(out, bytes).map_err(|e| e.to_string())?
                }
                "apng" => { warn_silent(&e, "APNG"); fwrite(out, open_formats::apng_bytes(&e, a.scale)?).map_err(|e| e.to_string())? }
                "glb" => {
                    let (bytes, notes) = osatura_core::gltf::export_glb_report(&e)?;
                    fwrite(out, bytes).map_err(|e| e.to_string())?;
                    report_warnings(a, path, "glb", notes.into_iter().map(|n| ("glb-limit", String::new(), n)).collect())?;
                }
                "webm" => export_webm(&e, out, a.scale, a.opts.get("--codec").map(String::as_str).unwrap_or("vp9"), Path::new(path).parent().unwrap_or(Path::new(".")))?,
                "png" => {
                    let cols = num(a, "--columns", 0.0)? as usize;
                    // `<name>.atlas.json`, never `<name>.json`: that name is where a Lottie export of the same stem goes
                    let json_path = Path::new(out).with_extension("atlas.json");
                    let name = Path::new(out).file_name().and_then(|n| n.to_str()).unwrap_or("sheet.png");
                    let (img, meta) = open_formats::spritesheet(&e, a.scale, cols, name)?;
                    fwrite(out, img).map_err(|e| e.to_string())?;
                    fwrite(&json_path, serde_json::to_string_pretty(&meta).unwrap()).map_err(|e| e.to_string())?;
                    println!("wrote {}", json_path.display());
                }
                "" => return Err(format!("'{out}' has no file extension; the extension picks the format (use .gif .html .htm .json .lottie .mp4 .apng .webm .png .glb)")),
                other => return Err(format!("unknown export format '.{other}' (use .gif .html .htm .json .lottie .mp4 .apng .webm .png .glb)")),
            }
            println!("wrote {out}");
            Ok(())
        }
        ["lipsync", path, wav, layer, out] => {
            let mut p = load_project(path)?;
            let audio = read_wav_file(wav)?;
            let mapping: [usize; 5] = match a.opts.get("--map") {
                None => osatura_core::audio::default_lipsync_map(osatura_core::audio::switch_children(&p, layer).unwrap_or(5)),
                Some(m) => {
                    let v: Vec<usize> = m.split(',').map(|s| s.trim().parse().map_err(|_| format!("bad --map entry '{s}'"))).collect::<Result<_, _>>()?;
                    v.try_into().map_err(|_| "--map needs exactly 5 numbers (rest,small,open,round,wide)".to_string())?
                }
            };
            let ch = osatura_core::audio::lipsync_checked(&p, layer, &audio, &mapping)?;
            p.channels.retain(|c| !(c.target == *layer && c.prop == Prop::LayerSwitch && matches!(c.source, Source::Keys { .. })));
            p.channels.push(ch);
            compile(p.clone(), path)?; // validates the layer exists and is a switch
            write_json(&p, out)?;
            println!("wrote {out} ({:.2}s of audio → {} frames)", audio.duration(), p.frames);
            Ok(())
        }
        ["walk", path, out] => {
            let mut p = load_project(path)?;
            let g = |k: &str| a.opts.get(k).cloned();
            let rig = WalkRig {
                hips: g("--hips"),
                thigh_l: g("--thigh-l"),
                thigh_r: g("--thigh-r"),
                shin_l: g("--shin-l"),
                shin_r: g("--shin-r"),
                arm_l: g("--arm-l"),
                arm_r: g("--arm-r"),
            };
            let d = WalkStyle::default();
            let style = WalkStyle {
                period: num(a, "--period", d.period)?,
                stride: num(a, "--stride", d.stride)?,
                knee: num(a, "--knee", d.knee)?,
                bob: num(a, "--bob", d.bob)?,
                arm_swing: num(a, "--swing", d.arm_swing)?,
            };
            let ch = walk_cycle(&rig, &style);
            if ch.is_empty() {
                return Err("name at least one bone (--thigh-l, --thigh-r, …)".into());
            }
            let n = ch.len();
            p.channels.extend(ch);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} (+{n} channels)");
            Ok(())
        }
        ["mirror", path, out] => {
            let mut p = load_project(path)?;
            let before = p.channels.clone();
            p.channels = mirror(&p.channels);
            let swapped = before.iter().zip(&p.channels).filter(|(a, b)| a.target != b.target).count();
            if before.is_empty() {
                eprintln!("warning: mirror: the project has no channels, so there is nothing to mirror; {out} is a copy of the input");
            } else if swapped == 0 {
                eprintln!("warning: mirror: no channel target has a left/right name (L_/R_ prefix, _L/_R or _l/_r suffix, left/right, L./R.), so no target was swapped; only the sign of rotations, horizontal moves and skew was flipped on {} channel(s)", p.channels.iter().zip(&before).filter(|(a, b)| a != b).count());
            }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} ({swapped} of {} channel target(s) swapped sides)", before.len());
            Ok(())
        }
        ["export-oca", path, dir] => {
            let p = load_project(path)?;
            compile(p.clone(), path)?.check_all_frames_finite()?;
            let f = open_formats::write_oca(&p, &asset_dir(path), Path::new(dir), a.scale, a.flags.iter().any(|f| f == "--layers"))?;
            println!("wrote {}", f.display());
            Ok(())
        }
        ["import-ora", file, out] => {
            let bytes = fs::read(file).map_err(|e| format!("cannot read {file}: {e}"))?;
            let name = Path::new(file).file_stem().and_then(|n| n.to_str()).unwrap_or("ora");
            let p = open_formats::import_ora(&bytes, name, num(a, "--fps", 24.0)?, num(a, "--frames", 1.0)? as u32)?;
            Engine::new(p.clone()).map_err(|errs| errs.join("; "))?;
            write_json(&p, out)?;
            println!("wrote {out} ({} layers)", p.layers.len());
            Ok(())
        }
        ["safety", path] => {
            let e = load(path)?;
            let r = safety::analyze_engine(&e, a.scale.min(1.0));
            let m = safety::motion_report(&e);
            let line = |n: &str, f: &safety::FlashFinding| {
                let v = if f.fails { "FAIL" } else if f.warns { "WARN" } else { "ok" };
                println!("{n:<12} {v:<5} worst {} transitions/s (frame {})", f.worst_transitions, f.worst_frame);
            };
            line("flashes", &r.general);
            line("red flashes", &r.red);
            println!("camera       pan {:.2} widths/s, rotation {:.0}°/s, zoom ×{:.2}/s", m.max_pan_per_sec, m.max_rotation_deg_per_sec, m.max_zoom_ratio_per_sec);
            for w in &m.warnings {
                println!("  warning: {w}");
            }
            println!("(engineering aid based on the public WCAG 2.3.1 definition; not a broadcast certification)");
            if r.passes() { Ok(()) } else { Err("flash limits exceeded".into()) }
        }
        ["reduced-motion", path, out] => {
            let p = load_project(path)?;
            let q = safety::reduced_motion_twin(&p, num(a, "--factor", 0.25)?);
            compile(q.clone(), path)?;
            write_json(&q, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["pose-db", "build", out] => {
            let project = match a.opts.get("--project") {
                Some(p) => Some(load_project(p)?),
                None => None,
            };
            let lib = match a.opts.get("--library") {
                Some(p) => Some(read_json::<osatura_core::motionlib::Library>(p)?),
                None => None,
            };
            if project.is_none() && lib.is_none() {
                return Err("give --project and/or --library".into());
            }
            let step = if a.opts.contains_key("--step") { Some(num(a, "--step", 1.0)?) } else { None };
            let db = posesuggest::build_db(project.as_ref(), lib.as_ref(), step)?;
            fwrite(out, db.to_json())?;
            println!("wrote {out}: {} poses of {} bones in {} sequence(s)", db.poses.len(), db.dim(), db.sequences.len());
            Ok(())
        }
        ["pose-db", "info", path] => {
            let db = posesuggest::PoseDb::from_json(&fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?)?;
            let sg = posesuggest::Suggester::new(db)?;
            println!("{} poses, {} bones: {}", sg.db.poses.len(), sg.db.dim(), sg.db.bones.join(", "));
            for (i, s) in sg.db.sequences.iter().enumerate() {
                println!("  sequence {i}: {} ({}, {} poses)", s.name, s.source, s.poses);
            }
            println!("PCA: {} axes keep {:.1}% of the variance (total {:.1} deg^2)", sg.space.k(), sg.space.explained.last().copied().unwrap_or(0.0) * 100.0, sg.space.total_var);
            for (i, (e, c)) in sg.space.eig.iter().zip(&sg.space.explained).enumerate() {
                println!("  axis {}: std {:.2} deg, cumulative {:.1}%", i + 1, e.sqrt(), c * 100.0);
            }
            println!("typical nearest-pose spacing {:.2} deg, typical step between consecutive poses {:.2} deg", sg.spacing, sg.step);
            Ok(())
        }
        ["pose-db", "train-mlp", path, out] => {
            let db = posesuggest::PoseDb::from_json(&fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?)?;
            let mut o = tinymlp::TrainOpts::default();
            if let Some(h) = a.opts.get("--hidden") {
                o.hidden = h.split(',').map(|x| x.trim().parse::<usize>().map_err(|_| format!("--hidden needs comma separated sizes, got '{h}'"))).collect::<Result<_, _>>()?;
            }
            let ep = num(a, "--epochs", o.epochs as f32)?;
            if !(1.0..=100_000.0).contains(&ep) {
                return Err("--epochs must be between 1 and 100000".into());
            }
            o.epochs = ep as usize;
            o.lr = num(a, "--lr", o.lr)?;
            o.seed = num(a, "--seed", 1.0)? as u64;
            let (m, hist) = posesuggest::train_completion(&db, &o)?;
            let bytes = if out.ends_with(".json") { m.to_json().into_bytes() } else { m.to_bytes() };
            fwrite(out, bytes)?;
            println!("wrote {out}: {} -> {:?} -> {} (epochs {}, loss {:.4} -> {:.4})", m.n_in(), o.hidden, m.n_out(), o.epochs, hist[0], hist[hist.len() - 1]);
            println!("trained on the library itself (no held-out data here); run pose-bench for held-out accuracy");
            Ok(())
        }
        ["pose-bench"] => {
            for l in posesuggest::benchmark(1, &posesuggest::default_bench_opts())? {
                println!("{:<11} {:<42} {:>6.2} deg RMS", l.task, l.method, l.rms_deg);
            }
            println!("synthetic gait-like library, 24 training cycles / 6 held-out cycles; completion sees 3 of 10 bones. Lower is better.");
            Ok(())
        }
        ["pose-suggest", kind, path, db] if ["between", "complete", "next"].contains(kind) => {
            let p = load_project(path)?;
            let sg = pose_suggester(a, &p, db)?;
            let fr = |k: &str| -> Result<Option<f32>, String> { if a.opts.contains_key(k) { Ok(Some(num(a, k, 0.0)?)) } else { Ok(None) } };
            let mut constraints: Vec<posesuggest::Constraint> = Vec::new();
            if let Some(f) = a.opts.get("--fix") {
                for part in f.split(',').filter(|x| !x.is_empty()) {
                    let (b, v) = part.split_once('=').ok_or_else(|| format!("--fix needs bone=angle, got '{part}'"))?;
                    constraints.push(posesuggest::Constraint::Angle { bone: b.into(), value: v.parse().map_err(|_| format!("bad angle '{v}'"))?, weight: 1.0 });
                }
            }
            if let Some(r) = a.opts.get("--reach") {
                for part in r.split(';').filter(|x| !x.is_empty()) {
                    let (b, xy) = part.split_once('=').ok_or_else(|| format!("--reach needs bone=x,y, got '{part}'"))?;
                    let (x, y) = xy.split_once(',').ok_or_else(|| format!("--reach needs bone=x,y, got '{part}'"))?;
                    constraints.push(posesuggest::Constraint::Reach { bone: b.into(), at: [x.parse().map_err(|_| format!("bad x '{x}'"))?, y.parse().map_err(|_| format!("bad y '{y}'"))?], weight: 0.1 });
                }
            }
            let model = match a.opts.get("--model") {
                Some(m) => Some(tinymlp::Mlp::load(&fs::read(m).map_err(|e| format!("cannot read {m}: {e}"))?)?),
                None => None,
            };
            let req = posesuggest::Request {
                kind: (*kind).into(),
                a: fr("--a")?,
                b: fr("--b")?,
                t: fr("--t")?,
                frame: fr("--frame")?,
                constraints,
                free: a.opts.get("--free").map(|f| f.split(',').filter(|x| !x.is_empty()).map(String::from).collect()),
                top: num(a, "--top", 5.0)? as usize,
                snap: fr("--snap")?,
                prior: fr("--prior")?,
                k: fr("--k")?.map(|k| k as usize),
            };
            let s = posesuggest::suggest(&p, &sg, &req, model.as_ref())?;
            if a.flags.iter().any(|f| f == "--json") {
                println!("{}", serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?);
            } else {
                print_suggestions(&s);
            }
            if a.opts.contains_key("--apply") {
                let rank = num(a, "--apply", 1.0)? as usize;
                let out = a.opts.get("--out").ok_or("--apply needs --out <project.json>")?;
                let c = s.candidates.get(rank.wrapping_sub(1)).ok_or_else(|| format!("there is no candidate {rank} (1..{})", s.candidates.len()))?;
                let (op, skipped) = posesuggest::apply_op(&p, &c.values, c.frame.unwrap_or(0.0), osatura_core::anim::Ease::Smooth)?;
                let mut ed = osatura_core::edit::Editor::new(p.clone()).map_err(|e| e.join("; "))?;
                ed.apply(op).map_err(|e| e.join("; "))?;
                write_json(ed.project(), out)?;
                println!("applied candidate {rank} at frame {:.1} -> {out}{}", c.frame.unwrap_or(0.0), if skipped.is_empty() { String::new() } else { format!(" (locked, skipped: {})", skipped.join(", ")) });
            }
            Ok(())
        }
        ["autotrace-bench"] => {
            println!("{:<14} {:<5} {:>6} {:>4} {:>7} {:>7} {:>6} {:>6} {:>7} {:>7} {:>8} {:>7}", "image", "mode", "colors", "tol", "MAE", "RMSE", "PSNR", "SSIM", "regions", "strokes", "anchors", "ms");
            for r in autotrace::benchmark() {
                println!("{:<14} {:<5} {:>6} {:>4} {:>7.2} {:>7.2} {:>6.1} {:>6.3} {:>7} {:>7} {:>8} {:>7.1}", r.name, if r.mode == autotrace::Mode::Fill { "fill" } else { "line" }, r.colors, r.tol, r.fidelity.mae, r.fidelity.rmse, r.fidelity.psnr, r.fidelity.ssim, r.regions, r.strokes, r.anchors, r.millis);
            }
            println!("MAE/RMSE in 0..255 over R,G,B on white; SSIM on luminance, 8x8 windows. Classical tracer: photographs become colour bands.");
            Ok(())
        }
        ["autotrace", image, out] => {
            let bytes = fs::read(image).map_err(|e| format!("cannot read {image}: {e}"))?;
            let img = autotrace::decode_image(&bytes)?;
            let mode = match a.opts.get("--mode").map(String::as_str).unwrap_or("fill") {
                "fill" => autotrace::Mode::Fill,
                "line" => autotrace::Mode::Line,
                m => return Err(format!("--mode must be fill or line, not '{m}'")),
            };
            let mut o = autotrace::TraceOptions { mode, ..autotrace::TraceOptions::default() };
            let whole = |k: &str, d: usize| -> Result<usize, String> {
                let v = num(a, k, d as f32)?;
                if v < 0.0 || v.fract() != 0.0 {
                    return Err(format!("{k} needs a whole number"));
                }
                Ok(v as usize)
            };
            o.colors = whole("--colors", o.colors)?;
            o.min_area = num(a, "--min-area", o.min_area)?;
            o.tol = num(a, "--tol", o.tol)?;
            o.smooth = whole("--smooth", o.smooth)?;
            o.corner_angle = num(a, "--corner", o.corner_angle)?;
            o.denoise = whole("--denoise", o.denoise)?;
            o.prune = whole("--prune", o.prune)?;
            o.seed = whole("--seed", 1)? as u64;
            o.invert = a.flags.iter().any(|f| f == "--invert");
            if a.opts.contains_key("--threshold") {
                let t = num(a, "--threshold", 128.0)?;
                if !(0.0..=255.0).contains(&t) {
                    return Err("--threshold must be 0..255".into());
                }
                o.threshold = Some(t as u8);
            }
            o.quant = match a.opts.get("--quant").map(String::as_str).unwrap_or("kmeans") {
                "kmeans" => autotrace::Quant::KMeans,
                "mediancut" => autotrace::Quant::MedianCut,
                q => return Err(format!("--quant must be kmeans or mediancut, not '{q}'")),
            };
            o.fit = match a.opts.get("--fit").map(String::as_str).unwrap_or("bezier") {
                "bezier" => autotrace::Fit::Bezier,
                "polygon" => autotrace::Fit::Polygon,
                "vw" => autotrace::Fit::PolygonVw,
                f => return Err(format!("--fit must be bezier, polygon or vw, not '{f}'")),
            };
            o.zorder = match a.opts.get("--zorder").map(String::as_str).unwrap_or("area") {
                "area" => autotrace::ZOrder::Area,
                "luminance" => autotrace::ZOrder::Luminance,
                z => return Err(format!("--zorder must be area or luminance, not '{z}'")),
            };
            let res = autotrace::trace(&img, &o)?;
            let fid = autotrace::measure(&img, &res)?;
            if let Some(png) = a.opts.get("--png") {
                let eng = Engine::new(autotrace::to_project(&res, "preview")).map_err(|e| e.join("; "))?;
                let bytes = osatura_core::raster::render_png(&eng, 0.0, &Default::default())?;
                fwrite(png, bytes)?;
            }
            if out.ends_with(".svg") {
                if a.opts.contains_key("--into") {
                    return Err("--into needs a .json output".into());
                }
                fwrite(out, autotrace::to_svg(&res))?;
            } else {
                let mut p = match a.opts.get("--into") {
                    Some(path) => load_project(path)?,
                    None => Project::new(Path::new(image).file_stem().and_then(|s| s.to_str()).unwrap_or("traced"), img.w as f32, img.h as f32, 24.0, 24),
                };
                let name = a.opts.get("--name").map(String::as_str).unwrap_or("trace");
                let used = autotrace::append_to_project(&mut p, &res, name, a.flags.iter().any(|f| f == "--fit-stage"))?;
                Engine::new(p.clone()).map_err(|errs| errs.join("; "))?; // asset dir = the --into project's (load_project)
                write_json(&p, out)?;
                println!("layer group '{used}' added");
            }
            println!("wrote {out}: {} regions, {} strokes, {} paths, {} anchors", res.regions.len(), res.strokes.len(), res.path_count(), res.anchor_count());
            println!("fidelity vs source: MAE {:.2}/255, RMSE {:.2}, PSNR {:.1} dB, SSIM {:.3}", fid.mae, fid.rmse, fid.psnr, fid.ssim);
            Ok(())
        }
        ["trace", image, out] => {
            let bytes = fs::read(image).map_err(|e| format!("cannot read {image}: {e}"))?;
            let cs = trace::trace_png(&bytes, num(a, "--threshold", 128.0)? as u8, num(a, "--tolerance", 1.0)?, num(a, "--min-area", 12.0)?)?;
            let pm = tiny_skia::Pixmap::decode_png(&bytes).map_err(|e| e.to_string())?;
            let mut p = Project::new("traced", pm.width() as f32, pm.height() as f32, 24.0, 24);
            let fill = a.opts.get("--fill").map(String::as_str).unwrap_or("#000000");
            p.layers = vec![trace::contours_to_layer("traced", &cs, fill, osatura_core::math::Vec2::ZERO)];
            Engine::new(p.clone()).map_err(|errs| errs.join("; "))?;
            write_json(&p, out)?;
            println!("wrote {out} ({} contours)", cs.len());
            Ok(())
        }
        ["skin-diffuse", path, layer, out] => {
            let mut p = load_project(path)?;
            p.auto_skin_diffuse(layer, num(a, "--influences", 4.0)? as usize, weights::DiffusionOptions::default())?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["drive", path, csv, maps, out] => {
            let mut p = load_project(path)?;
            let sig = perf::parse_signals_csv(&fs::read_to_string(csv).map_err(|e| format!("cannot read {csv}: {e}"))?)?;
            let maps: Vec<perf::InputMap> = read_json(maps)?;
            let ch = perf::drive(&maps, &sig, num(a, "--tol", 0.5)?)?;
            let n = ch.len();
            p.channels.extend(ch);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} (+{n} channels)");
            Ok(())
        }
        ["state-bake", path, machine, events, out] => {
            let mut p = load_project(path)?;
            let sm: statemachine::StateMachine = read_json(machine)?;
            let ev: Vec<statemachine::InputEvent> = read_json(events)?;
            let timeline = statemachine::simulate(&sm, &ev, p.frames)?;
            let ch = statemachine::bake(&sm, &timeline, num(a, "--tol", 0.01)?);
            let n = ch.len();
            p.channels.extend(ch);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} (+{n} channels)");
            Ok(())
        }
        ["pose-match", path, params, goals, out] => {
            let mut p = load_project(path)?;
            let params: Vec<posematch::Param> = read_json(params)?;
            let goals: Vec<posematch::Goal> = read_json(goals)?;
            let d = posematch::MatchOptions::default();
            let gens = num(a, "--generations", d.generations as f32)?;
            if !(1.0..=100_000.0).contains(&gens) {
                return Err("--generations must be between 1 and 100000".into());
            }
            let opts = posematch::MatchOptions {
                generations: gens as usize,
                population: d.population,
                seed: num(a, "--seed", d.seed as f32)? as u64,
            };
            let r = posematch::pose_match(&p, &params, &goals, &opts)?;
            p.channels.extend(r.channels);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out} (loss {:.3} px²)", r.loss);
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
