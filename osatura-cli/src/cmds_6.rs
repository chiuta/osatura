// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Commands part 6 of the dispatcher in `main.rs` (arms kept in their original order, so the first matching arm still wins): comp, event, snapshot, explain, spring-fit, compare, mcp, fx, layer-normal, onion, trail, proxy-bench, studio.
#![allow(unused_imports)]
use super::*;

/// `Some(result)` when one of this part's arms matched the command line, `None` to try the next part.
pub(crate) fn run(a: &Args, raw: &[String], pos: &[&str]) -> Option<Result<(), String>> {
    let _ = raw;
    let handled = std::cell::Cell::new(true);
    let r = (|| -> Result<(), String> {
        match pos {
        ["comp", mode @ ("capture" | "apply"), path, out, name] => {
            let mut p = load_project(path)?;
            if *mode == "capture" {
                comps::capture(&mut p, name)?;
            } else {
                for m in comps::apply(&mut p, name)? {
                    println!("warning: layer '{m}' no longer exists");
                }
            }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["comp", mode @ ("capture" | "apply"), ..] => Err(format!("comp {mode} needs <project.json> <out.json> <name>")),
        ["comp", other, ..] => Err(format!("unknown comp action '{other}' (capture, apply, list)")),
        ["event", path, out, name, frame] => {
            let mut p = load_project(path)?;
            let data = match a.opts.get("--data") {
                Some(d) => serde_json::from_str(d).map_err(|e| format!("--data is not valid JSON: {e}"))?,
                None => serde_json::Value::Null,
            };
            p.events.push(comps::Event { name: name.to_string(), frame: frame.parse().map_err(|_| format!("bad frame '{frame}'"))?, data });
            p.events.sort_by(|x, y| x.frame.total_cmp(&y.frame));
            write_json(&p, out)?;
            println!("wrote {out} ({} events)", p.events.len());
            Ok(())
        }
        ["snapshot", mode @ ("save" | "list"), path, dir] => {
            let s = snapshots::Snapshots::new(dir, num(a, "--keep", 20.0)? as usize);
            if *mode == "save" {
                match s.save(&load_project(path)?)? {
                    Some(r) => println!("saved revision {} ({})", r.number, r.hash),
                    None => println!("unchanged since the last revision"),
                }
            } else {
                for r in s.list() {
                    println!("{:>6}  {}  {}", r.number, r.hash, r.path.display());
                }
            }
            Ok(())
        }
        ["snapshot", mode @ ("restore" | "recover"), _path, dir, out] => {
            let s = snapshots::Snapshots::new(dir, 20);
            let p = if *mode == "recover" {
                s.recover().ok_or("no readable revision found")?
            } else {
                s.load(a.opts.get("--number").map(|n| n.parse::<u64>().map_err(|_| "bad --number".to_string())).transpose()?)?
            };
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["snapshot", mode @ ("save" | "list"), ..] => Err(format!("snapshot {mode} needs <project.json> <dir> and nothing else")),
        ["snapshot", mode @ ("restore" | "recover"), ..] => Err(format!("snapshot {mode} needs <project.json> <dir> <out.json>")),
        ["snapshot", other, ..] => Err(format!("unknown snapshot action '{other}' (save, list, restore, recover)")),
        ["explain", path, target, frame] => {
            let e = load(path)?;
            let x = explain::explain(&e, target, frame.parse().map_err(|_| format!("bad frame '{frame}'"))?)?;
            for l in &x.lines {
                println!("- {l}");
            }
            Ok(())
        }
        ["spring-fit"] => {
            let samples: Vec<f32> = a
                .opts
                .get("--samples")
                .ok_or("--samples a,b,c,… is required")?
                .split(',')
                .map(|s| s.trim().parse().map_err(|_| format!("bad sample '{s}'")))
                .collect::<Result<_, _>>()?;
            let fit = springfit::fit_spring(num(a, "--start", 0.0)?, &samples, num(a, "--fps", 24.0)?)?;
            println!("frequency {:.2} Hz, damping {:.2} (rms error {:.3} of the step size)", fit.frequency, fit.damping, fit.rms);
            println!("add to the project: {{\"bone\": \"<bone>\", \"frequency\": {:.2}, \"damping\": {:.2}}}", fit.frequency, fit.damping);
            Ok(())
        }
        ["compare", a_path, b_path, frame, out] => {
            let (ea, eb) = (load(a_path)?, load(b_path)?);
            let (w, h, px) = inspect::compare_frame(&ea, &eb, frame.parse().map_err(|_| format!("bad frame '{frame}'"))?, a.scale, num(a, "--gain", 4.0)?)?;
            fwrite(out, open_formats::encode_rgba_png(w, h, &px)?).map_err(|e| e.to_string())?;
            println!("wrote {out}");
            Ok(())
        }
        ["mcp", path] => {
            let p = load_project(path)?;
            let ed = Editor::with_asset_dir(p, asset_dir(path)).map_err(|errs| format!("{path} is not a valid project:\n  - {}", errs.join("\n  - ")))?;
            let mut ag = agent::Agent::new(ed, Some(std::path::PathBuf::from(path)));
            let stdin = std::io::stdin();
            ag.serve(stdin.lock(), std::io::stdout().lock()).map_err(|e| e.to_string())
        }
        ["fx", path, frame, chain_path, out] => {
            let e = load(path)?;
            let frame: f32 = frame.parse().map_err(|_| format!("bad frame number '{frame}'"))?;
            let frame = check_frame_in_range(frame, e.project.frames, a.flags.iter().any(|f| f == "--clamp"))?;
            let mut chain: Vec<postfx::Fx> = read_json(chain_path)?;
            let chain_dir = std::path::Path::new(chain_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
            postfx::resolve_normal_map_paths(&mut chain, &chain_dir)?;
            let (w, h, px) = if a.flags.iter().any(|f| f == "--layer-normals") {
                postfx::render_with_layer_normals(&e, frame, a.scale, &chain, &asset_dir(path))?
            } else {
                let (w, h, mut px) = osatura_core::raster::render_rgba(&e, frame, &RasterOptions { scale: a.scale, show_bones: false, transparent: false });
                postfx::apply(&chain, w as usize, h as usize, &mut px)?;
                (w, h, px)
            };
            fwrite(out, open_formats::encode_rgba_png(w, h, &px)?).map_err(|e| e.to_string())?;
            println!("wrote {out} ({} effect(s))", chain.len());
            Ok(())
        }
        ["layer-normal", path, out, layer, map] => {
            let mut p = load_project(path)?;
            let l = p.find_layer_mut(layer).ok_or_else(|| format!("no layer '{layer}'"))?;
            if *map == "none" {
                l.normal_map = None;
            } else {
                let bytes = fs::read(map).map_err(|e| format!("cannot read {map}: {e}"))?;
                let fit = match a.opts.get("--fit").map(String::as_str).unwrap_or("stretch") {
                    "stretch" => postfx::NormalFit::Stretch,
                    "tile" => postfx::NormalFit::Tile,
                    "exact" => postfx::NormalFit::Exact,
                    o => return Err(format!("unknown --fit '{o}' (stretch, tile, exact)")),
                };
                let nm = postfx::NormalMap { data: Some(osatura_core::base64::encode(&bytes)), path: None, fit, strength: num(a, "--strength", 1.0)?, green_down: a.flags.iter().any(|f| f == "--green-down") };
                l.normal_map = Some(nm);
            }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["onion", path, out] => {
            let e = load(path)?;
            let spec = onion_spec_from(a)?;
            let frame = num(a, "--frame", 0.0)?;
            if frame < 0.0 || frame.fract() != 0.0 {
                return Err("--frame needs a whole frame number".into());
            }
            let svg = if a.flags.iter().any(|f| f == "--ghosts-only") { overlays::onion_svg(&e, frame as u32, &spec, &[])? } else { overlays::onion_composite_svg(&e, frame as u32, &spec, &[])? };
            let n = overlays::ghosts_for(&e, frame as u32, &spec, &[])?.len();
            fwrite(out, svg)?;
            println!("wrote {out} ({n} ghost(s) around frame {frame})");
            Ok(())
        }
        ["trail", path] | ["trail", path, _] if a.opts.contains_key("--target") => {
            let e = load(path)?;
            let t = overlays::TrailTarget::parse(&a.opts["--target"])?;
            let from = num(a, "--from", 0.0)?;
            let to = num(a, "--to", e.project.frames.max(1) as f32 - 1.0)?;
            if from < 0.0 || to < 0.0 || from.fract() != 0.0 || to.fract() != 0.0 {
                return Err("--from/--to need whole frame numbers".into());
            }
            let sub = num(a, "--smooth", 0.0)?;
            if sub < 0.0 || sub.fract() != 0.0 {
                return Err("--smooth needs a whole number of subdivisions (0 = off)".into());
            }
            let tr = overlays::trails(&e, &[t], from as u32, to as u32, sub as u32)?;
            if let Some(out) = a.pos.get(2) {
                fwrite(out, overlays::trails_svg(&e, &tr, None))?;
                if !a.flags.iter().any(|f| f == "--json") {
                    println!("wrote {out} ({} ticks, {} key marker(s))", tr[0].points.len(), tr[0].keys.len());
                }
            }
            if a.flags.iter().any(|f| f == "--json") {
                println!("{}", serde_json::to_string_pretty(&overlays::trail_json(&tr[0])).unwrap_or_default());
            } else if a.pos.len() < 3 {
                println!("{}", tr[0].target);
                for (f, p) in tr[0].frames.iter().zip(&tr[0].points) {
                    let k = if tr[0].keys.iter().any(|k| (k - *f as f32).abs() <= 0.5) { " key" } else { "" };
                    println!("{f:>5}  {:>9.3} {:>9.3}{k}", p.x, p.y);
                }
            }
            Ok(())
        }
        ["proxy-bench", path] => {
            let e = load(path)?;
            let n = (num(a, "--frames", 24.0)? as u32).clamp(1, e.project.frames.max(1));
            let time = |level: u8| -> Result<f64, String> {
                let t0 = std::time::Instant::now();
                for f in 0..n {
                    osatura_core::scrub::render_proxy_rgba(&e, f as f32, level)?;
                }
                Ok(t0.elapsed().as_secs_f64() * 1000.0 / n as f64)
            };
            let svg_t0 = std::time::Instant::now();
            for f in 0..n {
                render_frame(&e, f as f32, &RenderOptions::default());
            }
            let svg_ms = svg_t0.elapsed().as_secs_f64() * 1000.0 / n as f64;
            let vec_ms = |level: u8| {
                let t = std::time::Instant::now();
                for f in 0..n {
                    let _ = osatura_core::scrub::proxy_svg_vector(&e, f as f32, level, false);
                }
                t.elapsed().as_secs_f64() * 1000.0 / n as f64
            };
            let (v0, v2, v4) = (vec_ms(0), vec_ms(2), vec_ms(4));
            let t0 = std::time::Instant::now();
            for f in 0..n {
                e.evaluate(f as f32);
            }
            let eval_ms = t0.elapsed().as_secs_f64() * 1000.0 / n as f64;
            let full = time(0)?;
            println!("{n} frames of {} ({}x{}), ms per frame, single thread:", e.project.name, e.project.width, e.project.height);
            println!("  evaluate only       {eval_ms:8.2}");
            println!("  svg (no raster)     {svg_ms:8.2}   vector proxy: level 0 {v0:.2}, level 2 {v2:.2}, level 4 {v4:.2}");
            println!("  raster level 0      {full:8.2}   (full quality)");
            for l in 1..=osatura_core::scrub::MAX_LEVEL {
                let t = time(l)?;
                println!("  raster level {l}      {t:8.2}   ({:.1}x faster than level 0)", full / t.max(1e-9));
            }
            Ok(())
        }
        ["onion", path, frame, out] => {
            let e = load(path)?;
            let frame: u32 = frame.parse().map_err(|_| format!("bad frame number '{frame}'"))?;
            let d = overlays::OnionOptions::default();
            let o = overlays::OnionOptions {
                before: num(a, "--before", d.before as f32)? as u32,
                after: num(a, "--after", d.after as f32)? as u32,
                step: num(a, "--step", d.step as f32)? as u32,
                scale: a.scale,
                ..d
            };
            let (w, h, px) = overlays::onion_skin(&e, frame, &o)?;
            fwrite(out, open_formats::encode_rgba_png(w, h, &px)?).map_err(|e| e.to_string())?;
            println!("wrote {out}");
            Ok(())
        }
        ["trail", path, bone, out] => {
            let e = load(path)?;
            let pts = overlays::motion_trail(&e, bone, true, num(a, "--from", 0.0)? as u32, num(a, "--to", (e.project.frames - 1) as f32)? as u32)?;
            fwrite(out, overlays::trail_svg(&e, &pts, "#ff3366")).map_err(|e| e.to_string())?;
            println!("wrote {out} ({} points)", pts.len());
            Ok(())
        }
        ["studio", path] => {
            let p = load_project(path)?;
            let ed = Editor::with_asset_dir(p, asset_dir(path))
                .map_err(|errs| format!("{path} is not a valid project:\n  - {}", errs.join("\n  - ")))?;
            for w in unknown_field_warnings(path) {
                eprintln!("warning: {w} (ignored)");
            }
            let port = a.opts.get("--port").map(String::as_str).unwrap_or("8080");
            let ui_path = a.opts.get("--ui-config").map(std::path::PathBuf::from).or_else(uiconfig::default_config_path);
            let bind = a.opts.get("--bind").map(String::as_str).unwrap_or("127.0.0.1");
            let token = a.opts.get("--collab-token").cloned();
            if let Some(t) = &token {
                collab_srv::check_token(t)?;
            }
            let secret = match a.opts.get("--collab-secret-file") {
                Some(f) => Some(collab_srv::read_secret_file(f)?),
                None => None,
            };
            let revoke: Vec<String> = a.opts.get("--collab-revoke").map(|v| v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()).unwrap_or_default();
            if !revoke.is_empty() && secret.is_none() {
                return Err("--collab-revoke needs --collab-secret-file (it revokes per-participant tokens)".into());
            }
            let rate = match a.opts.get("--collab-rate") {
                Some(v) => Some(v.parse::<f64>().ok().filter(|r| r.is_finite() && *r >= 0.0 && *r <= 100000.0).ok_or_else(|| format!("--collab-rate needs a number of requests per second (0 = off), got '{v}'"))?),
                None => None,
            };
            let security = studio::Security { token, secret, revoke, rate };
            let allow_hosts: Vec<String> = a.opts.get("--allow-host").map(|v| v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()).unwrap_or_default();
            if a.flags.iter().any(|f| f == "--collab-nosync") && !a.opts.contains_key("--collab-log") {
                return Err("--collab-nosync only changes how --collab-log is flushed; it needs --collab-log LOG (nothing was started)".into());
            }
            let collab_log = a.opts.get("--collab-log").cloned().map(|p| (p, !a.flags.iter().any(|f| f == "--collab-nosync")));
            let source_url = a.opts.get("--source-url").cloned();
            if let Some(u) = &source_url {
                studio::check_source_url(u)?;
            }
            let source_url = source_url.or_else(|| studio::default_source_url().map(String::from));
            let public_origin = a.opts.get("--public-origin").cloned();
            if let Some(o) = &public_origin {
                studio::public_origin_host(o)?;
            }
            studio::serve(ed, path, &format!("{bind}:{port}"), ui_path, security, collab_log, &allow_hosts, source_url, public_origin)
        }
            _ => {
                handled.set(false);
                Ok(())
            }
        }
    })();
    handled.get().then_some(r)
}
