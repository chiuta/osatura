// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Commands part 3 of the dispatcher in `main.rs` (arms kept in their original order, so the first matching arm still wins): follow, take-record, take-apply, take-list, sound, make-loop, foot-lock, stretch, magnet, physics, cloth, eye-clip, parallax, chan-meta, reference, import-anim, dof, dof-export, import-seq, render-hdr, import-lottie.
#![allow(unused_imports)]
use super::*;

/// `Some(result)` when one of this part's arms matched the command line, `None` to try the next part.
pub(crate) fn run(a: &Args, raw: &[String], pos: &[&str]) -> Option<Result<(), String>> {
    let _ = raw;
    let handled = std::cell::Cell::new(true);
    let r = (|| -> Result<(), String> {
        match pos {
        ["follow", path, out, target, path_layer, a, b] | ["follow", path, out, target, path_layer, a, b, _] => {
            let mut p = load_project(path)?;
            let n = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            let src = match &p.find_layer(path_layer).ok_or_else(|| format!("no layer '{path_layer}'"))?.kind {
                osatura_core::project::LayerKind::Vector { paths } => paths.first().cloned().ok_or("layer has no path")?,
                _ => return Err("path layer must be a vector layer".into()),
            };
            let (name, is_bone) = match target.strip_prefix("bone:") { Some(b) => (b, true), None => (*target, false) };
            let o = pathfollow::FollowOptions { start_frame: n(a)?, end_frame: n(b)?, orient: pos.get(8) != Some(&"no-orient"), ..Default::default() };
            let ch = pathfollow::follow_path(&p, name, is_bone, &src, &o)?;
            p.channels.extend(ch);
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["take-record", path, lib, name, a, b] | ["take-record", path, lib, name, a, b, _] => {
            let p = load_project(path)?;
            let mut l = if Path::new(lib).exists() { load_library(lib)? } else { motionlib::Library::default() };
            let n = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            let only: Vec<String> = pos.get(6).map(|s| s.split(',').map(str::to_string).collect()).unwrap_or_default();
            let k = motionlib::record(&mut l, &p, name, n(a)?, n(b)?, &only)?;
            fwrite(lib, l.to_json()).map_err(|e| e.to_string())?;
            println!("recorded {k} tracks into {lib}");
            Ok(())
        }
        ["take-apply", path, out, lib, name, at] | ["take-apply", path, out, lib, name, at, _] | ["take-apply", path, out, lib, name, at, _, _] => {
            let mut p = load_project(path)?;
            let l = load_library(lib)?;
            let take = l.find(name).ok_or_else(|| format!("no take '{name}'"))?;
            let rename: Vec<(String, String)> = match pos.get(6) {
                Some(s) if *s != "-" => parse_take_rename(s, take)?,
                _ => vec![],
            };
            let speed: f32 = pos.get(7).map(|s| s.parse::<f32>().ok().filter(|v| v.is_finite() && *v > 0.0).ok_or_else(|| format!("bad speed '{s}': a positive number such as 1 or 0.5 (nothing was written)"))).transpose()?.unwrap_or(1.0);
            let k = motionlib::apply_blended(&mut p, take, at.parse().map_err(|_| format!("bad frame '{at}'"))?, &rename, speed, num(a, "--blend", 0.0)?)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("applied {k} tracks; wrote {out}");
            Ok(())
        }
        ["take-list", lib] => {
            let l = load_library(lib)?;
            for t in &l.takes { println!("{} ({} frames, {} tracks)", t.name, t.length, t.tracks.len()); }
            Ok(())
        }
        ["sound", "list", path] => {
            let p = load_project(path)?;
            for s in &p.sounds {
                let mut extra = String::new();
                if s.trim_in > 0.0 || s.trim_out.is_some() { extra += &format!(" trim {}s..{}", s.trim_in, s.trim_out.map(|o| format!("{o}s")).unwrap_or_else(|| "end".into())); }
                if s.fade_in > 0.0 { extra += &format!(" fade-in {}s", s.fade_in); }
                if s.fade_out > 0.0 { extra += &format!(" fade-out {}s", s.fade_out); }
                println!("{}: {} at frame {} gain {}{extra}", s.name, s.file, s.start, s.gain);
            }
            if p.sounds.is_empty() { println!("no sound tracks"); }
            Ok(())
        }
        ["sound", "add", path, out, name, file] | ["sound", "add", path, out, name, file, _] | ["sound", "add", path, out, name, file, _, _] => {
            let mut p = load_project(path)?;
            let start: f32 = pos.get(6).map(|s| s.parse().map_err(|_| "bad start frame".to_string())).transpose()?.unwrap_or(0.0);
            let gain: f32 = pos.get(7).map(|s| s.parse().map_err(|_| "bad gain".to_string())).transpose()?.unwrap_or(1.0);
            soundtrack::check_file(&asset_dir(path), file).map_err(|e| format!("sound add: {e} (nothing was added)"))?;
            let mut t = soundtrack::SoundTrack::new(name, file, start, gain);
            apply_sound_opts(&mut t, a, p.fps)?;
            soundtrack::add_track(&mut p, t)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["sound", "edit", path, out, name] => {
            let mut p = load_project(path)?;
            let fps = p.fps;
            let mut t = p.sounds.iter().find(|s| s.name == *name).cloned().ok_or_else(|| format!("no sound '{name}'"))?;
            if a.opts.contains_key("--start") { t.start = num(a, "--start", t.start)?; }
            if a.opts.contains_key("--gain") { t.gain = num(a, "--gain", t.gain)?; }
            apply_sound_opts(&mut t, a, fps)?;
            soundtrack::add_track(&mut p, t)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["sound", "remove", path, out, name] => {
            let mut p = load_project(path)?;
            if !soundtrack::remove(&mut p, name) { return Err(format!("no sound '{name}'")); }
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["sound", "mix", path, out] => {
            let e = load(path)?;
            soundio::mix_to_file(&e.project, Path::new(path).parent().unwrap_or(Path::new(".")), Path::new(out))?;
            println!("wrote {out} ({:.2} s)", e.project.frames as f32 / e.project.fps);
            Ok(())
        }
        ["make-loop", path, out] | ["make-loop", path, out, _] | ["make-loop", path, out, _, _] => {
            let mut p = load_project(path)?;
            let only: Vec<String> = pos.get(3).filter(|s| &***s != "all").map(|s| s.split(',').map(|x| x.trim().to_string()).collect()).unwrap_or_default();
            let props: Vec<Prop> = match pos.get(4) {
                Some(s) => s.split(',').map(|x| serde_json::from_str(&format!("\"{}\"", x.trim())).map_err(|_| format!("unknown property '{x}'"))).collect::<Result<_, _>>()?,
                None => vec![],
            };
            let r = looper::make_loop_mode(&mut p, &only, &props, if a.flags.iter().any(|f| f == "--inclusive") { looper::LoopMode::Inclusive } else { looper::LoopMode::Exclusive })?;
            for (t, pr, d) in &r.converted { println!("{t} {pr:?}: removed drift {d:.2}"); }
            for (t, pr) in &r.flattened { println!("note: {t} {pr:?} is now constant: all of its change was travel, which make-loop removes on purpose (in-place loop); lint reports it as L005; name other targets / props to keep a channel's travel"); }
            for t in &r.skipped_locked { println!("{t}: locked, skipped"); }
            if r.converted.is_empty() { println!("no keyed channel converted"); }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["foot-lock", path, out, bone, ground] | ["foot-lock", path, out, bone, ground, _] | ["foot-lock", path, out, bone, ground, _, _] => {
            let mut p = load_project(path)?;
            let mut o = footlock::LockOptions { ground_y: ground.parse().map_err(|_| "bad ground_y".to_string())?, ..Default::default() };
            if let Some(t) = pos.get(6) { o.tolerance = t.parse().map_err(|_| "bad tolerance".to_string())?; }
            if let Some(m) = pos.get(7) { o.min_frames = m.parse().map_err(|_| "bad min_frames".to_string())?; }
            let c = footlock::lock_feet(&mut p, bone, &o)?;
            for k in &c { println!("contact frames {}..{} pinned at ({:.1}, {:.1})", k.start, k.end, k.x, k.y); }
            if c.is_empty() { println!("no ground contact found; nothing changed"); }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["stretch", path, out, follower, leader] | ["stretch", path, out, follower, leader, _] | ["stretch", path, out, follower, leader, _, _] => {
            let mut p = load_project(path)?;
            let mut o = osatura_core::stretch::StretchOptions::default();
            if let Some(v) = pos.get(5) { o.min_factor = v.parse().map_err(|_| "bad min".to_string())?; }
            if let Some(v) = pos.get(6) { o.max_factor = v.parse().map_err(|_| "bad max".to_string())?; }
            let n = osatura_core::stretch::bake_stretch(&mut p, follower, leader, &o)?;
            if n == 0 { println!("already reaching the leader; nothing changed"); } else { println!("corrected {n} frames"); }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["magnet", path, out, ctrl, target, radius, strength] => {
            let mut p = load_project(path)?;
            let t = match target.strip_prefix("bone:") {
                Some(b) => magnet::Target::Bone(b.to_string()),
                None => {
                    let (x, y) = target.split_once(',').ok_or("target is bone:NAME or x,y")?;
                    magnet::Target::Point(osatura_core::math::Vec2::new(x.parse().map_err(|_| "bad x".to_string())?, y.parse().map_err(|_| "bad y".to_string())?))
                }
            };
            let m = magnet::Magnet { radius: radius.parse().map_err(|_| "bad radius".to_string())?, strength: strength.parse().map_err(|_| "bad strength".to_string())? };
            let n = magnet::bake_magnet(&mut p, ctrl, &t, &m)?;
            println!("pulled on {n} frames");
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["physics", path, out, chain, rest @ ..] => {
            let mut p = load_project(path)?;
            let mut o = physics::PhysicsOptions::default();
            let mut overwrite = false;
            let mut lateral = 0.0f32;
            if let Some(pr) = rest.iter().find_map(|kv| kv.strip_prefix("preset=")) {
                o = physics::preset(pr).ok_or_else(|| format!("unknown preset '{pr}' (try {})", physics::PRESETS.join(", ")))?;
            }
            let num = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            let split_follow = |c: &str| match c.split_once('@') { Some((a, b)) => (a.to_string(), Some(b.to_string())), None => (c.to_string(), None) };
            let mut follow_all: Option<String> = None;
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "gravity" => o.gravity = osatura_core::math::Vec2::new(0.0, num(v)?),
                    "wind" => { let (x, y) = v.split_once(',').ok_or("wind is x,y")?; o.wind = osatura_core::math::Vec2::new(num(x)?, num(y)?); }
                    "preset" => {}
                    "lateral" => lateral = num(v)?,
                    "merge" => overwrite = match v { "keep" => false, "replace" => true, _ => return Err("merge is keep or replace".into()) },
                    "gust" => o.gust = num(v)?,
                    "damping" => o.damping = num(v)?,
                    "circle" => {
                        for c in v.split(';') {
                            let (c, fb) = split_follow(c);
                            let f: Vec<&str> = c.split(',').collect();
                            if f.len() != 3 { return Err("circle is x,y,r[@bone]".into()); }
                            o.colliders.push(physics::Circle { centre: osatura_core::math::Vec2::new(num(f[0])?, num(f[1])?), radius: num(f[2])?, follow: fb });
                        }
                    }
                    "capsule" => {
                        for c in v.split(';') {
                            let (c, fb) = split_follow(c);
                            let f: Vec<&str> = c.split(',').collect();
                            if f.len() != 5 { return Err("capsule is x1,y1,x2,y2,r[@bone]".into()); }
                            o.capsules.push(physics::Capsule { a: osatura_core::math::Vec2::new(num(f[0])?, num(f[1])?), b: osatura_core::math::Vec2::new(num(f[2])?, num(f[3])?), radius: num(f[4])?, follow: fb });
                        }
                    }
                    "follow" => follow_all = Some(v.to_string()),
                    "drive" => {
                        for d in v.split(';') {
                            let (b, g) = d.rsplit_once(':').ok_or("drive is bone:gain")?;
                            o.drives.push(physics::Drive { bone: b.to_string(), gain: num(g)? });
                        }
                    }
                    "vortex" => {
                        for d in v.split(';') {
                            let f: Vec<&str> = d.split(':').collect();
                            if f.len() < 2 || f.len() > 4 { return Err("vortex is bone|x,y:swirl[:radial[:radius]]".into()); }
                            let (centre, follow) = match f[0].split_once(',') {
                                Some((x, y)) => (osatura_core::math::Vec2::new(num(x)?, num(y)?), None),
                                None => (osatura_core::math::Vec2::ZERO, Some(f[0].to_string())),
                            };
                            o.vortices.push(physics::Vortex { centre, follow, swirl: num(f[1])?, radial: if f.len() > 2 { num(f[2])? } else { 0.0 }, radius: if f.len() > 3 { num(f[3])? } else { 200.0 } });
                        }
                    }
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            if let Some(fb) = follow_all {
                for c in o.colliders.iter_mut().filter(|c| c.follow.is_none()) { c.follow = Some(fb.clone()); }
                for c in o.capsules.iter_mut().filter(|c| c.follow.is_none()) { c.follow = Some(fb.clone()); }
            }
            let strands: Vec<Vec<String>> = chain.split('/').map(|c| c.split(',').map(|s| s.trim().to_string()).collect()).collect();
            let ch = physics::simulate_strip(&p, &strands, lateral, &o)?;
            let (added, skipped) = physics::merge_channels(&mut p, ch, overwrite);
            for b in &skipped { println!("kept existing animation on '{b}' (use merge=replace to overwrite)"); }
            if added == 0 && !a.flags.iter().any(|f| f == "--allow-empty") {
                let why = if skipped.is_empty() { "the simulation produced no channels".to_string() } else { format!("{} bone(s) already have animation and merge=keep (the default) leaves it alone", skipped.len()) };
                return Err(format!("physics: nothing baked - {why}; nothing was written. Use merge=replace to overwrite the existing animation, or --allow-empty to accept an unchanged file"));
            }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("baked {added} bones; wrote {out}");
            Ok(())
        }
        ["cloth", path, out, layer, rest @ ..] => {
            let mut p = load_project(path)?;
            let mut o = cloth::ClothOptions::default();
            let num = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            let int = |s: &str| s.parse::<u32>().map_err(|_| format!("bad whole number '{s}'"));
            let split_follow = |c: &str| match c.split_once('@') { Some((a, b)) => (a.to_string(), Some(b.to_string())), None => (c.to_string(), None) };
            let v2 = |s: &str, what: &str| -> Result<osatura_core::math::Vec2, String> { let (x, y) = s.split_once(',').ok_or(format!("{what} is x,y"))?; Ok(osatura_core::math::Vec2::new(num(x)?, num(y)?)) };
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "pin" => o.pins = cloth::parse_pins(v)?,
                    "gravity" => o.gravity = osatura_core::math::Vec2::new(0.0, num(v)?),
                    "wind" => o.wind = v2(v, "wind")?,
                    "gust" => o.gust = num(v)?,
                    "gust-hz" => o.gust_hz = num(v)?,
                    "gust-len" => o.gust_len = num(v)?,
                    "stiffness" => o.stiffness = num(v)?,
                    "shear" => o.shear = num(v)?,
                    "bend" => o.bend = num(v)?,
                    "max-stretch" => o.max_stretch = num(v)?,
                    "damping" => o.damping = num(v)?,
                    "cols" => o.cols = Some(int(v)? as usize),
                    "rows" => o.rows = Some(int(v)? as usize),
                    "rect" => { let f: Vec<&str> = v.split(',').collect(); if f.len() != 4 { return Err("rect is x,y,w,h".into()); } o.rect = Some([num(f[0])?, num(f[1])?, num(f[2])?, num(f[3])?]); }
                    "name" => o.name = v.to_string(),
                    "substeps" => o.substeps = int(v)?,
                    "iterations" => o.iterations = int(v)?,
                    "preroll" => o.preroll = int(v)?,
                    "from" => o.from = Some(int(v)?),
                    "to" => o.to = Some(int(v)?),
                    "circle" => {
                        for c in v.split(';') {
                            let (c, fb) = split_follow(c);
                            let f: Vec<&str> = c.split(',').collect();
                            if f.len() != 3 { return Err("circle is x,y,r[@bone]".into()); }
                            o.colliders.push(physics::Circle { centre: osatura_core::math::Vec2::new(num(f[0])?, num(f[1])?), radius: num(f[2])?, follow: fb });
                        }
                    }
                    "capsule" => {
                        for c in v.split(';') {
                            let (c, fb) = split_follow(c);
                            let f: Vec<&str> = c.split(',').collect();
                            if f.len() != 5 { return Err("capsule is x1,y1,x2,y2,r[@bone]".into()); }
                            o.capsules.push(physics::Capsule { a: osatura_core::math::Vec2::new(num(f[0])?, num(f[1])?), b: osatura_core::math::Vec2::new(num(f[2])?, num(f[3])?), radius: num(f[4])?, follow: fb });
                        }
                    }
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            if !rest.iter().any(|kv| kv.starts_with("pin=")) { o.pins = cloth::parse_pins("top")?; }
            p = cloth::bake_cloth(&p, layer, &o)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("baked cloth on '{layer}' ({} channels); wrote {out}", p.channels.iter().filter(|c| c.target.contains("::") && matches!(c.prop, Prop::LatticeDx | Prop::LatticeDy)).count());
            Ok(())
        }
        ["eye-clip", path, out] => {
            let mut p = load_project(path)?;
            let r = eyeclip::auto_eye_clip(&mut p);
            for c in &r.clipped { println!("+ {c}"); }
            for s in &r.skipped { println!("- {s}"); }
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["parallax", path, out, layers, far, near] => {
            let mut p = load_project(path)?;
            let names: Vec<String> = layers.split(',').map(|s| s.trim().to_string()).collect();
            depth::spread_parallax(&mut p, &names, far.parse().map_err(|_| "bad far value".to_string())?, near.parse().map_err(|_| "bad near value".to_string())?)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["chan-meta", path, out, target, prop, rest @ ..] => {
            let mut p = load_project(path)?;
            let pr = if *prop == "*" { None } else { Some(Prop::parse(prop).ok_or_else(|| format!("unknown property '{prop}'"))?) };
            let onoff = |v: &str| match v { "on" => Ok(true), "off" => Ok(false), _ => Err("use on or off".to_string()) };
            let mut c = chanmeta::Change::default();
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "color" => c.color = Some(v.to_string()),
                    "shy" => c.shy = Some(onoff(v)?),
                    "solo" => c.solo = Some(onoff(v)?),
                    "lock" => c.lock = Some(onoff(v)?),
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            chanmeta::set_meta(&mut p, target, pr, &c)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["reference", path, out, layer, state] => {
            let mut p = load_project(path)?;
            let on = match *state { "on" => true, "off" => false, _ => return Err("use on or off".into()) };
            p.find_layer_mut(layer).ok_or_else(|| format!("no layer '{layer}'"))?.reference = on;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("wrote {out}");
            Ok(())
        }
        ["import-anim", path, out, name, file, rest @ ..] => {
            let mut p = load_project(path)?;
            let (mut looped, mut start, mut x, mut y) = (false, 0.0f32, 0.0f32, 0.0f32);
            let num = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "loop" => looped = v == "on",
                    "start" => start = num(v)?,
                    "x" => x = num(v)?,
                    "y" => y = num(v)?,
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            let bytes = fs::read(file).map_err(|e| format!("{file}: {e}"))?;
            let n = osatura_core::animimport::import_animation(&mut p, name, &bytes, osatura_core::math::Vec2::new(x, y), start, looped)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("imported {n} frames as layer '{name}'; wrote {out}");
            Ok(())
        }
        ["dof", path, frame, out] => {
            let e = load(path)?;
            let f: f32 = frame.parse().map_err(|_| "bad frame".to_string())?;
            let f = check_frame_in_range(f, e.project.frames, a.flags.iter().any(|f| f == "--clamp"))?;
            let d = osatura_core::dof::DofOptions { scale: a.scale, focus: num(a, "--focus", 1.0)?, strength: num(a, "--strength", 12.0)?, max_radius: num(a, "--max-radius", 24.0)? };
            let (w, h, rgba) = osatura_core::dof::render_dof_animated(&e.project, f, &d, a.opts.contains_key("--focus"), a.opts.contains_key("--strength"))?;
            fwrite(out, open_formats::encode_rgba_png(w, h, &rgba)?).map_err(|e| e.to_string())?;
            println!("wrote {out}");
            Ok(())
        }
        ["dof-export", path, out] => {
            let e = load(path)?;
            let d = osatura_core::dof::DofOptions { scale: a.scale, focus: num(a, "--focus", 1.0)?, strength: num(a, "--strength", 12.0)?, max_radius: num(a, "--max-radius", 24.0)? };
            let whole = |k: &str, v: f32| -> Result<u32, String> {
                if v.is_finite() && v >= 0.0 && v.fract() == 0.0 { Ok(v as u32) } else { Err(format!("{k} needs a whole frame number >= 0")) }
            };
            let from = whole("--from", num(a, "--from", 0.0)?)?;
            let to = if a.opts.contains_key("--to") { Some(whole("--to", num(a, "--to", 0.0)?)?) } else { None };
            let (ff, fs_) = (a.opts.contains_key("--focus"), a.opts.contains_key("--strength"));
            dof_export_preflight(out, a)?;
            let lower = out.to_lowercase();
            if lower.ends_with(".gif") {
                fwrite(out, osatura_core::dof::dof_gif(&e.project, from, to, &d, ff, fs_)?).map_err(|e| e.to_string())?;
            } else if lower.ends_with(".apng") {
                fwrite(out, osatura_core::dof::dof_apng(&e.project, from, to, &d, ff, fs_)?).map_err(|e| e.to_string())?;
            } else if lower.ends_with(".mp4") || lower.ends_with(".webm") {
                let webm = lower.ends_with(".webm");
                let codec = a.opts.get("--codec").map(String::as_str).unwrap_or("vp9");
                if !e.project.sounds.is_empty() {
                    eprintln!("note: dof-export video carries no audio; the project's {} sound track(s) are not included (use export for sound)", e.project.sounds.len());
                }
                let pngs = osatura_core::dof::dof_png_sequence(&e.project, from, to, &d, ff, fs_)?;
                encode_png_video(&pngs, e.project.fps, out, webm, codec)?;
            } else if Path::new(out).extension().is_some() {
                return Err(format!("dof-export writes .gif, .apng, .mp4, .webm or a folder (no extension); '{out}' is none of these (nothing was written)"));
            } else {
                let pngs = osatura_core::dof::dof_png_sequence(&e.project, from, to, &d, ff, fs_)?;
                fs::create_dir_all(out).map_err(|e| format!("{out}: {e}"))?;
                for (i, b) in pngs.iter().enumerate() {
                    fwrite(Path::new(out).join(format!("f{:04}.png", from as usize + i)), b).map_err(|e| e.to_string())?;
                }
            }
            println!("wrote {out}");
            Ok(())
        }
        ["import-seq", path, out, name, folder, rest @ ..] => {
            let mut p = load_project(path)?;
            let (mut hold, mut looped, mut start, mut x, mut y) = (1.0f32, false, 0.0f32, 0.0f32, 0.0f32);
            let num = |s: &str| s.parse::<f32>().map_err(|_| format!("bad number '{s}'"));
            for kv in rest {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("option '{kv}' must be key=value"))?;
                match k {
                    "hold" => hold = num(v)?,
                    "loop" => looped = v == "on",
                    "start" => start = num(v)?,
                    "x" => x = num(v)?,
                    "y" => y = num(v)?,
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
            let mut names: Vec<String> = fs::read_dir(folder).map_err(|e| format!("{folder}: {e}"))?
                .filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.to_lowercase().ends_with(".png")).collect();
            // natural order: compare digit runs as numbers (frame2 before frame10)
            let key = |s: &String| -> Vec<(u64, String)> {
                let mut out = Vec::new();
                let (mut num_s, mut txt) = (String::new(), String::new());
                for ch in s.chars() {
                    if ch.is_ascii_digit() { if !txt.is_empty() { out.push((0, std::mem::take(&mut txt))); } num_s.push(ch); }
                    else { if !num_s.is_empty() { out.push((num_s.parse().unwrap_or(0), String::new())); num_s.clear(); } txt.push(ch); }
                }
                if !num_s.is_empty() { out.push((num_s.parse().unwrap_or(0), String::new())); }
                if !txt.is_empty() { out.push((0, txt)); }
                out
            };
            names.sort_by_key(|a| key(a));
            if names.is_empty() { return Err(format!("no .png files in {folder}")); }
            let mut frames = Vec::new();
            for n in &names { frames.push((n.clone(), fs::read(Path::new(folder).join(n)).map_err(|e| format!("{n}: {e}"))?)); }
            seqimport::import_png_sequence(&mut p, name, &frames, osatura_core::math::Vec2::new(x, y), start, hold, looped)?;
            compile(p.clone(), path)?;
            write_json(&p, out)?;
            println!("imported {} images; wrote {out}", frames.len());
            Ok(())
        }
        ["render-hdr", path, frame, out] | ["render-hdr", path, frame, out, _] => {
            let e = load(path)?;
            let frame: f32 = frame.parse().map_err(|_| format!("bad frame number '{frame}'"))?;
            let frame = check_frame_in_range(frame, e.project.frames, a.flags.iter().any(|f| f == "--clamp"))?;
            e.evaluate_checked(frame)?;
            let mut prec = match a.opts.get("--precision") { Some(v) => hdr::Precision::parse(v)?, None => hdr::Precision::Half };
            if a.flags.iter().any(|f| f == "--float32") { prec = hdr::Precision::Float; }
            let legacy = a.flags.iter().any(|f| f == "--8bit");
            if legacy && prec == hdr::Precision::Float { return Err("--8bit source cannot be combined with float32 output".into()); }
            osatura_core::raster::check_project_size(&e.project, a.scale)?;
            let ro = RasterOptions { scale: a.scale, show_bones: false, transparent: true };
            let lower = out.to_lowercase();
            let linear = pos.get(4) == Some(&"linear");
            let (w, h, bytes) = if legacy {
                let (w, h, px) = osatura_core::raster::render_rgba(&e, frame, &ro);
                let b = if lower.ends_with(".exr") { hdr::encode_exr_half(w, h, &px)? }
                    else if lower.ends_with(".png") { hdr::encode_png16(w, h, &px, linear)? }
                    else { return Err("output must end in .exr or .png".into()) };
                (w, h, b)
            } else {
                let img = osatura_core::fraster::render_float(&e, frame, &ro);
                let b = if lower.ends_with(".exr") { hdr::encode_exr_buf(img.width, img.height, &img.rgba, prec)? }
                    else if lower.ends_with(".png") { hdr::encode_png16_buf(img.width, img.height, &img.rgba, linear)? }
                    else { return Err("output must end in .exr or .png".into()) };
                (img.width, img.height, b)
            };
            fwrite(out, bytes).map_err(|e| e.to_string())?;
            println!("wrote {out} ({w}x{h})");
            Ok(())
        }
        ["import-lottie", inp, out, rest @ ..] if rest.len() <= 1 => {
            let text = fs::read_to_string(inp).map_err(|e| format!("{inp}: {e}"))?;
            let r = lottie_import::import_lottie(&text)?;
            for w in &r.warnings { println!("warning: {w}"); }
            compile(r.project.clone(), inp)?;
            write_json(&r.project, out)?;
            if let Some(fxp) = rest.first() {
                fwrite(fxp, serde_json::to_string_pretty(&r.post_fx).map_err(|e| e.to_string())?)?;
                println!("wrote {} post effect(s) to {fxp}", r.post_fx.len());
            } else if !r.post_fx.is_empty() {
                println!("note: {} blur / drop shadow effect(s) found; pass a third path to save them as a post-effect chain", r.post_fx.len());
            }
            println!("imported {} layers; wrote {out}", r.project.layers.len());
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
