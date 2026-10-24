// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// Output that cannot be delivered (a closed pipe: `osatura ... | head`; a closed stderr) must never panic or
// change the exit status: these macros replace the std ones for the whole crate and drop what cannot be written.
macro_rules! println {
    () => {{ use std::io::Write as _; let _ = std::io::stdout().lock().write_all(b"\n"); }};
    ($($t:tt)*) => {{ use std::io::Write as _; let _ = writeln!(std::io::stdout().lock(), $($t)*); }};
}
#[allow(unused_macros)]
macro_rules! print {
    ($($t:tt)*) => {{ use std::io::Write as _; let _ = write!(std::io::stdout().lock(), $($t)*); }};
}
macro_rules! eprintln {
    () => {{ use std::io::Write as _; let _ = std::io::stderr().lock().write_all(b"\n"); }};
    ($($t:tt)*) => {{ use std::io::Write as _; let _ = writeln!(std::io::stderr().lock(), $($t)*); }};
}
#[allow(unused_macros)]
macro_rules! eprint {
    ($($t:tt)*) => {{ use std::io::Write as _; let _ = write!(std::io::stderr().lock(), $($t)*); }};
}
mod http;
mod collab_srv;
mod soundio;
mod vcs_cmd;
mod fuzz_cmd;
mod ci_cmd;
mod face_cmd;
mod fit_cmd;
mod lod_cmd;
mod studio;
mod uiconfig;

use osatura_core::anim_tools::{mirror, walk_cycle, WalkRig, WalkStyle};
use osatura_core::audio::read_wav_file;
use osatura_core::edit::Editor;
use osatura_core::export;
use osatura_core::{autotrace, posesuggest, tinymlp, speech, links, warp, cage, clipstack, cloth, soundtrack, looper, bonevis, parentswitch, skins, audiomap, lottie_import, hdr, seqimport, chanmeta, depth, eyeclip, physics, magnet, footlock, motionlib, camshots, pathfollow, layout, autorig, drivers, comps, explain, rigedit, snapshots, springfit, vocab, agent, overlays, postfx, importers, inspect, keyops, lint, procgen, open_formats, perf, posematch, safety, statemachine, trace, meshtrace, weights};
use osatura_core::project::{Prop, Source};
use osatura_core::math::Vec2;
use osatura_core::raster::{render_png, RasterOptions};
use osatura_core::svg::{render_frame, RenderOptions};
use osatura_core::{demo, Engine, Project};
use std::collections::HashMap;
use std::{env, fs, path::Path, process::{Command, ExitCode}};

mod usage;
use usage::USAGE;

/// Landmark stream plus face config for the `face-*` commands (config from the `face` section of a map file).
fn face_inputs(a: &Args, marks: &str, map: Option<&str>, default_fps: f32) -> Result<(osatura_core::faceparam::LandmarkStream, osatura_core::faceparam::FaceConfig), String> {
    let fps = num(a, "--fps", default_fps)?;
    let layout = a.opts.get("--layout").map(|l| osatura_core::faceparam::Layout::parse(l)).transpose()?;
    let stream = osatura_core::faceparam::parse_ndjson(&fs::read_to_string(marks).map_err(|e| format!("cannot read {marks}: {e}"))?, fps, layout)?;
    for w in &stream.warnings { println!("warning: {w}"); }
    let mf = map.or(a.opts.get("--map").map(String::as_str));
    let cfg = match mf {
        Some(m) => osatura_core::faceparam::parse_mapfile(&fs::read_to_string(m).map_err(|e| format!("cannot read {m}: {e}"))?)?.1,
        None => Default::default(),
    };
    Ok((stream, cfg))
}

fn mocap_map(a: &Args) -> Result<Option<osatura_core::mocap::ManualMap>, String> {
    a.opts.get("--map").map(|f| osatura_core::mocap::parse_map(&fs::read_to_string(f).map_err(|e| format!("cannot read {f}: {e}"))?)).transpose()
}


fn pick_opts(a: &Args) -> Result<osatura_core::pick::PickOptions, String> {
    let mut o = osatura_core::pick::PickOptions::default();
    o.tolerance = num(a, "--tol", o.tolerance)?;
    if !o.tolerance.is_finite() || o.tolerance < 0.0 {
        return Err(format!("--tol needs a number >= 0 (stage units), got '{}'", a.opts.get("--tol").map(String::as_str).unwrap_or("")));
    }
    o.bones = a.flags.iter().any(|f| f == "--bones");
    if a.flags.iter().any(|f| f == "--root") {
        o.group_mode = osatura_core::pick::GroupMode::Root;
    }
    let list = |k: &str| a.opts.get(k).map(|v| v.split(',').filter(|s| !s.is_empty()).map(String::from).collect::<Vec<_>>()).unwrap_or_default();
    o.point_layers = list("--points");
    o.locked = list("--locked");
    Ok(o)
}

fn describe_target(t: &osatura_core::pick::Target) -> String {
    use osatura_core::pick::Target;
    match t {
        Target::Layer { path } => format!("layer {path}"),
        Target::Bone { name } => format!("bone {name}"),
        Target::Point { layer, point } => format!("point {} of {layer}", serde_json::to_string(point).unwrap_or_default()),
    }
}

fn timeline_text(m: &osatura_core::timeline::TimelineModel) -> String {
    use osatura_core::timeline::Row;
    let mut s = format!("timeline: {} frames at {} fps, {} channels\n", m.frames, m.fps, m.tracks.len());
    for r in &m.rows {
        match *r {
            Row::Group(g) => {
                let g = &m.groups[g];
                s.push_str(&format!("{}{}{}\n", "  ".repeat(g.depth), g.label, if g.locked { " [locked]" } else { "" }));
            }
            Row::Track(t) => {
                let t = &m.tracks[t];
                let depth = m.groups[t.group].depth + 1;
                let keys: Vec<String> = t.keys.iter().map(|k| format!("{}", k.frame)).collect();
                let mut flags = String::new();
                if t.locked { flags.push_str(" [locked]"); }
                if t.shy { flags.push_str(" [shy]"); }
                if t.solo { flags.push_str(" [solo]"); }
                if let Some(c) = &t.color { flags.push_str(&format!(" {c}")); }
                let body = if t.source == "keys" { format!("keys @ {}", keys.join(" ")) } else { t.source.to_string() };
                s.push_str(&format!("{}#{} {} ({}){}\n", "  ".repeat(depth), t.ch, t.label, body, flags));
            }
        }
    }
    s
}

/// `TARGET:PROP[@F0..F1]` items joined by `;`, `#CH[@F0..F1]`, or `all`.
fn parse_key_sel(m: &osatura_core::timeline::TimelineModel, spec: &str) -> Result<Vec<osatura_core::timeline::KeyRef>, String> {
    use osatura_core::timeline::{normalize, KeyRef};
    let mut out: Vec<KeyRef> = Vec::new();
    if spec == "all" {
        return Ok(osatura_core::timeline::select_all(m, false));
    }
    for item in spec.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let (head, range) = match item.split_once('@') {
            None => (item, None),
            Some((h, r)) => {
                let (a, b) = r.split_once("..").ok_or_else(|| format!("bad range '{r}' (use F0..F1)"))?;
                let f = |x: &str| x.trim().parse::<f32>().map_err(|_| format!("bad frame '{x}' in range"));
                (h, Some((f(a)?, f(b)?)))
            }
        };
        let chans: Vec<usize> = if let Some(n) = head.strip_prefix('#') {
            vec![n.parse::<usize>().map_err(|_| format!("bad channel '{n}'"))?]
        } else {
            let (target, prop) = head.rsplit_once(':').ok_or_else(|| format!("selector '{head}' needs TARGET:PROP"))?;
            let prop = Prop::parse(prop).ok_or_else(|| format!("unknown property '{prop}'"))?;
            m.tracks.iter().filter(|t| t.target == target && t.prop == prop && t.editable).map(|t| t.ch).collect()
        };
        if chans.is_empty() {
            return Err(format!("nothing matches '{head}'"));
        }
        let (lo, hi) = range.unwrap_or((f32::NEG_INFINITY, f32::INFINITY));
        let mut any = false;
        for c in chans {
            let t = m.tracks.get(c).ok_or_else(|| format!("no channel {c}"))?;
            for k in t.keys.iter().filter(|k| k.frame >= lo.min(hi) && k.frame <= lo.max(hi)) {
                out.push(KeyRef { ch: c, frame: k.frame });
                any = true;
            }
        }
        if !any {
            return Err(format!("no keys in '{item}'"));
        }
    }
    normalize(&mut out);
    if out.is_empty() {
        return Err("empty selection".into());
    }
    Ok(out)
}

fn parse_ease_arg(s: &str) -> Result<osatura_core::anim::Ease, String> {
    use osatura_core::anim::Ease;
    if s.trim_start().starts_with('{') {
        return serde_json::from_str(s).map_err(|e| format!("bad ease JSON: {e}"));
    }
    if let Some(r) = s.strip_prefix("bezier:") {
        let n: Vec<f32> = r.split(',').map(|x| x.trim().parse::<f32>().map_err(|_| format!("bad number '{x}'"))).collect::<Result<_, _>>()?;
        if n.len() != 4 {
            return Err("bezier needs x1,y1,x2,y2".into());
        }
        return Ok(Ease::CubicBezier { x1: n[0], y1: n[1], x2: n[2], y2: n[3] });
    }
    serde_json::from_value(serde_json::json!({"type": s})).map_err(|_| format!("unknown ease '{s}'"))
}

fn parse_list_usize(s: &str) -> Result<Vec<usize>, String> {
    s.split(',').filter(|x| !x.is_empty()).map(|x| x.trim().parse::<usize>().map_err(|_| format!("bad channel index '{x}'"))).collect()
}

fn keyedit_cmd(a: &Args, path: &str, out: &str, sub: &str) -> Result<(), String> {
    use osatura_core::timeline::{self as tl, Clip, Collision, EditOpts, Remap, RetimeMode, TlOp};
    use osatura_core::timeline_graph::{HandleMode, Side, TangentKind};
    let p = load_project(path)?;
    let m = tl::build(&p);
    let mut ed = Editor::with_asset_dir(p, asset_dir(path)).map_err(|errs| format!("{path} is not a valid project:\n  - {}", errs.join("\n  - ")))?;
    let flag = |f: &str| a.flags.iter().any(|x| x == f);
    let opt = |k: &str| a.opts.get(k).map(String::as_str);
    let need = |k: &str| opt(k).ok_or_else(|| format!("keyedit {sub} needs {k}"));
    let mut opts = EditOpts { snap: !flag("--no-snap"), ..EditOpts::default() };
    if flag("--free") {
        opts.bounds = tl::Bounds::Free;
    }
    if let Some(c) = opt("--collision") {
        opts.collision = Collision::parse(c).ok_or_else(|| format!("--collision must be replace, merge or refuse, got '{c}'"))?;
    }
    let sel = || -> Result<Vec<tl::KeyRef>, String> { parse_key_sel(&m, need("--sel")?) };
    let n = |k: &str, d: f32| num(a, k, d);
    let op = match sub {
        "move" => TlOp::MoveKeys { keys: sel()?, dt: n("--dt", 0.0)?, dv: n("--dv", 0.0)?, opts },
        "scale" => TlOp::ScaleKeys { keys: sel()?, pivot: n("--pivot", 0.0)?, factor: n("--factor", 1.0)?, opts },
        "dup" => TlOp::Duplicate { keys: sel()?, offset: n("--offset", f32::NAN)?, opts },
        "delete" => TlOp::Delete { keys: sel()? },
        "ease" => TlOp::SetEase { keys: sel()?, ease: parse_ease_arg(need("--ease")?)? },
        "value" => TlOp::SetValue { keys: sel()?, value: n("--value", f32::NAN)? },
        "smooth" => TlOp::Smooth {
            keys: sel()?,
            kind: match opt("--kind").unwrap_or("monotone") {
                "monotone" => TangentKind::Monotone,
                "catmull_rom" => TangentKind::CatmullRom,
                k => return Err(format!("--kind must be monotone or catmull_rom, got '{k}'")),
            },
        },
        "copy" => TlOp::Copy { keys: sel()? },
        "paste" => {
            let f = need("--clip")?;
            let clip: Clip = serde_json::from_str(&fs::read_to_string(f).map_err(|e| format!("cannot read {f}: {e}"))?).map_err(|e| format!("bad clip {f}: {e}"))?;
            let mut remap = Vec::new();
            if let Some(r) = opt("--remap") {
                let (from, to) = r.split_once('=').ok_or("--remap needs from_target:prop=to_target:prop")?;
                let (ft, fp) = from.rsplit_once(':').ok_or("--remap source needs target:prop")?;
                let (tt, tp) = to.rsplit_once(':').ok_or("--remap destination needs target:prop")?;
                remap.push(Remap { from_target: ft.into(), from_prop: Prop::parse(fp).ok_or_else(|| format!("unknown property '{fp}'"))?, to_target: tt.into(), to_prop: Prop::parse(tp).ok_or_else(|| format!("unknown property '{tp}'"))? });
            }
            TlOp::Paste { clip, at: n("--at", 0.0)?, remap, opts }
        }
        "add" => TlOp::AddKey {
            target: need("--target")?.to_string(),
            prop: Prop::parse(need("--prop")?).ok_or("unknown --prop")?,
            frame: n("--at", 0.0)?,
            value: opt("--value").map(|_| n("--value", 0.0)).transpose()?,
            ease: opt("--ease").map(parse_ease_arg).transpose()?,
            opts,
        },
        "ripple" => TlOp::Ripple {
            at: n("--at", 0.0)?,
            count: n("--by", f32::NAN)?,
            mode: match opt("--mode").unwrap_or("insert") {
                "insert" => RetimeMode::Insert,
                "delete" => RetimeMode::Delete,
                x => return Err(format!("--mode must be insert or delete, got '{x}'")),
            },
            scope: opt("--scope").map(parse_list_usize).transpose()?,
            adjust_frames: flag("--adjust-frames"),
            opts,
        },
        "offset" => TlOp::Offset { channels: parse_list_usize(need("--channels")?)?, dt: n("--dt", f32::NAN)?, opts },
        "extrapolate" => TlOp::Extrapolate {
            channels: parse_list_usize(need("--channels")?)?,
            extrapolate: serde_json::from_value(serde_json::Value::String(need("--extrapolate")?.into())).map_err(|_| "--extrapolate must be hold, linear, loop, ping_pong or loop_offset")?,
        },
        "loop" => TlOp::MakeLoop { channels: parse_list_usize(need("--channels")?)?, end: opt("--end").map(|_| n("--end", 0.0)).transpose()?, extrapolate: None, opts },
        "handle" => {
            let s = sel()?;
            if s.len() != 1 {
                return Err("handle needs --sel to match exactly one key".into());
            }
            TlOp::Handle {
                ch: s[0].ch,
                frame: s[0].frame,
                side: match need("--side")? {
                    "in" => Side::In,
                    "out" => Side::Out,
                    x => return Err(format!("--side must be in or out, got '{x}'")),
                },
                t: n("--t", f32::NAN)?,
                v: n("--v", f32::NAN)?,
                mode: match opt("--mode").unwrap_or("free") {
                    "free" => HandleMode::Free,
                    "aligned" => HandleMode::Aligned,
                    "auto" => HandleMode::Auto,
                    x => return Err(format!("--mode must be free, aligned or auto, got '{x}'")),
                },
            }
        }
        "lock" => {
            let on = !matches!(opt("--value"), Some("off") | Some("false") | Some("0"));
            TlOp::Flags { target: need("--target")?.to_string(), prop: opt("--prop").map(|x| Prop::parse(x).ok_or("unknown --prop")).transpose()?, lock: Some(on), shy: None, solo: None, color: None }
        }
        x => return Err(format!("unknown keyedit subcommand '{x}' (move, scale, dup, delete, ease, value, smooth, copy, paste, add, ripple, offset, extrapolate, loop, handle, lock)")),
    };
    let plan = tl::plan(ed.project(), &op)?;
    if let (Some(clip), Some(f)) = (&plan.clip, opt("--clip")) {
        fwrite(f, serde_json::to_string_pretty(clip).map_err(|e| e.to_string())?)?;
    }
    if out == "-" {
        println!("{}", serde_json::to_string_pretty(&plan).map_err(|e| e.to_string())?);
        return Ok(());
    }
    if let Some(o) = &plan.op {
        ed.apply(o.clone()).map_err(|e| e.join("; "))?;
        fwrite(out, serde_json::to_string_pretty(ed.doc()).map_err(|e| e.to_string())?)?;
    }
    println!("{sub}: {} keys changed{}{}", plan.changed, if plan.op.is_some() { format!(", wrote {out}") } else { ", project unchanged".into() }, if plan.clip.is_some() { ", clip saved" } else { "" });
    for note in &plan.notes {
        println!("  note: {note}");
    }
    Ok(())
}

/// `unknown field <path>` lines for keys of the project file that the model ignores (typos, newer versions).
fn unknown_field_warnings(path: &str) -> Vec<String> {
    match read_text_capped(path) {
        Ok(t) => osatura_core::unknownfields::unknown_fields(&t).unwrap_or_default().into_iter().map(|f| format!("unknown field {f}")).collect(),
        Err(_) => Vec::new(),
    }
}

/// Largest project file `load_project` reads (override with `--max-input-bytes N`).
static MAX_INPUT_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(256 << 20);

/// Reads a project file as text: a regular file only (never a device or FIFO such as /dev/zero) and at most
/// `MAX_INPUT_BYTES` (also enforced while reading, in case the file grows).
fn read_text_capped(path: &str) -> Result<String, String> {
    use std::io::Read;
    let max = MAX_INPUT_BYTES.load(std::sync::atomic::Ordering::SeqCst);
    let md = fs::metadata(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    if !md.is_file() {
        return Err(format!("cannot read {path}: not a regular file (devices, pipes and directories are refused)"));
    }
    if md.len() > max {
        return Err(format!("cannot read {path}: {} bytes exceed the input limit of {max} bytes (raise it with --max-input-bytes N)", md.len()));
    }
    let f = fs::File::open(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let mut buf = Vec::new();
    f.take(max + 1).read_to_end(&mut buf).map_err(|e| format!("cannot read {path}: {e}"))?;
    if buf.len() as u64 > max {
        return Err(format!("cannot read {path}: larger than the input limit of {max} bytes (raise it with --max-input-bytes N)"));
    }
    String::from_utf8(buf).map_err(|_| format!("cannot read {path}: stream did not contain valid UTF-8"))
}

fn load_project(path: &str) -> Result<Project, String> {
    let text = read_text_capped(path)?;
    let p = Project::from_json(&text).map_err(|e| format!("invalid JSON in {path}: {e}"))?;
    // relative assets of every derived project (validation in write_json, lod, dof, ...) are looked up next to the input
    osatura_core::assets::set_default_dir(Some(asset_dir(path)));
    Ok(p)
}

/// The overall verdict of `collab-sim`: convergence alone is not enough, the merged document must
/// also compile as a project.
fn collab_ok(converged_ok: bool, merged_compiles: bool) -> bool {
    converged_ok && merged_compiles
}

/// `osatura collab-sim`: random concurrent editing by N replicas (see `osatura_core::collab`).
fn collab_sim(a: &Args, path: &str) -> Result<(), String> {
    use osatura_core::collab::{simulate, Replica, SimConfig, SimRng};
    let p = load_project(path)?;
    let doc = serde_json::to_value(&p).map_err(|e| e.to_string())?;
    let whole = |k: &str, d: usize, lo: usize, hi: usize| -> Result<usize, String> {
        let v = num(a, k, d as f32)?;
        if v < lo as f32 || v > hi as f32 || v.fract() != 0.0 {
            return Err(format!("{k} needs a whole number in {lo}..={hi}"));
        }
        Ok(v as usize)
    };
    let mut cfg = SimConfig::new(whole("--clients", 3, 1, 64)?, whole("--ops", 100, 1, 200_000)?, a.opts.get("--seed").map(|s| s.parse::<u64>().map_err(|_| "--seed needs a whole number".to_string())).transpose()?.unwrap_or(1));
    cfg.undo_pct = whole("--undo-pct", 12, 0, 90)?;
    let raw = a.flags.iter().any(|f| f == "--raw");
    cfg.project_aware = !raw;
    let replays = whole("--replays", 10, 0, 1000)?;
    let rep = simulate(&doc, &cfg)?;
    // the same envelopes, delivered to fresh replicas in other random orders, must give the same state
    let mut replay_ok = 0;
    let mut first_bad = None;
    for k in 0..replays {
        let mut order = rep.trace.clone();
        SimRng(cfg.seed ^ (k as u64 + 1).wrapping_mul(0x9E37_79B9)).shuffle(&mut order);
        let mut r = Replica::new(1000 + k as u32, &doc)?;
        let mut bad = false;
        for e in order {
            if r.receive(e).is_err() {
                bad = true;
                break;
            }
        }
        if !bad && r.pending_len() == 0 && r.json() == rep.final_doc {
            replay_ok += 1;
        } else if first_bad.is_none() {
            first_bad = Some(k);
        }
    }
    // informative only: does the merged result still compile as a project? (random edits are arbitrary)
    let valid = serde_json::from_value::<Project>(rep.final_doc.clone()).ok().map(|pr| Engine::new(pr).is_ok()).unwrap_or(false);
    let converged_ok = rep.converged && rep.state_identical && rep.clocks_identical && rep.pending_left == 0 && replay_ok == replays;
    let ok = collab_ok(converged_ok, valid);
    let strict = a.flags.iter().any(|f| f == "--require-valid");
    if a.flags.iter().any(|f| f == "--json") {
        println!("{}", serde_json::json!({
            "ok": ok, "converged_ok": converged_ok, "clients": rep.clients, "seed": cfg.seed, "edits": rep.edits, "envelopes": rep.envelopes, "noops": rep.noops, "rejected_local": rep.rejected_local,
            "undo_redo_steps": rep.undos, "deliveries": rep.deliveries, "duplicates_dropped": rep.duplicates, "buffered": rep.buffered, "dropped_prims": rep.dropped_prims,
            "converged": rep.converged, "state_identical": rep.state_identical, "clocks_identical": rep.clocks_identical, "pending_left": rep.pending_left,
            "replays": replays, "replays_identical": replay_ok, "tombstones": rep.tombstones, "final_json_bytes": rep.doc_bytes, "merged_project_compiles": valid,
        }));
    } else {
        println!("{} replicas, {} edits ({} envelopes, {} no-ops, {} rejected locally, {} undo/redo steps), seed {}", rep.clients, rep.edits, rep.envelopes, rep.noops, rep.rejected_local, rep.undos, cfg.seed);
        println!("deliveries {} (duplicates ignored {}, held back by the causal buffer {}), primitives dropped {}", rep.deliveries, rep.duplicates, rep.buffered, rep.dropped_prims);
        println!("visible documents identical: {}; internal states identical: {}; vector clocks identical: {}; envelopes still waiting: {}", rep.converged, rep.state_identical, rep.clocks_identical, rep.pending_left);
        println!("replays of the same envelopes in other random orders: {replay_ok}/{replays} identical");
        println!("merged document: {} bytes, {} tombstones; compiles as a project: {} (informative with --raw: those edits are arbitrary JSON)", rep.doc_bytes, rep.tombstones, valid);
        println!("{}", if converged_ok { "CONVERGED" } else { "DIVERGED" });
    }
    if converged_ok && (ok || !strict) {
        Ok(())
    } else if converged_ok {
        Err("replicas converged, but the merged document does not compile as a project".into())
    } else {
        Err(rep.divergence.unwrap_or_else(|| format!("replay {} differs", first_bad.unwrap_or(0))))
    }
}

/// `pattern` or `pattern:prop` (prop in snake_case, e.g. bone_angle).
fn parse_pattern_prop(s: &str) -> Result<(String, Option<osatura_core::project::Prop>), String> {
    match s.split_once(':') {
        None => Ok((s.to_string(), None)),
        Some((pat, pr)) => {
            let prop = serde_json::from_value(serde_json::Value::String(pr.to_string())).map_err(|_| format!("unknown property '{pr}'"))?;
            Ok((pat.to_string(), Some(prop)))
        }
    }
}

fn asset_dir(path: &str) -> std::path::PathBuf {
    Path::new(path).parent().filter(|p| !p.as_os_str().is_empty()).map(|p| p.to_path_buf()).unwrap_or_else(|| ".".into())
}

fn write_words(a: &Args, al: &speech::Alignment) -> Result<(), String> {
    if let Some(f) = a.opts.get("--words-out") {
        let body = if f.to_ascii_lowercase().ends_with(".srt") { speech::words_srt(al) } else { speech::words_json(al)? };
        fwrite(f, body)?;
    }
    Ok(())
}

fn compile(project: Project, path: &str) -> Result<Engine, String> {
    Engine::with_asset_dir(project, &asset_dir(path))
        .map_err(|errs| format!("{path} is not a valid project:\n  - {}", errs.join("\n  - ")))
}

fn load(path: &str) -> Result<Engine, String> {
    compile(load_project(path)?, path)
}

/// Like `load`, but honours `--size WxH`: layout containers re-solve for that canvas size (the file is not changed).
fn load_sized(path: &str, a: &Args) -> Result<Engine, String> {
    let mut p = load_project(path)?;
    if let Some(sz) = a.opts.get("--size") {
        let (w, h) = osatura_core::containers::parse_size(sz)?;
        p = osatura_core::containers::responsive(&p, w, h)?;
    }
    compile(p, path)
}

mod args;
mod cmds_1;
mod cmds_2;
mod cmds_3;
mod cmds_4;
mod cmds_5;
mod cmds_6;
use args::*;

/// A bake that read events but produced nothing (typo'd ids, wrong source, every key skipped) is a failure, not a success.
fn check_bake_not_empty(a: &Args, inputs: usize, r: &osatura_core::liveinput::BakeReport, cmd: &str, what: &str) -> Result<(), String> {
    if inputs > 0 && r.channels == 0 && r.keys_set == 0 && r.takes == 0 && r.events == 0 && !a.flags.iter().any(|f| f == "--allow-empty") {
        return Err(format!("{cmd}: {inputs} input(s) were read but nothing was produced (0 channels, 0 keys, 0 takes, 0 events); check the ids in {what}; nothing was written (use --allow-empty to accept an empty result)"));
    }
    Ok(())
}

fn num(a: &Args, key: &str, default: f32) -> Result<f32, String> {
    match a.opts.get(key) {
        None => Ok(default),
        Some(v) => v.parse().map_err(|_| format!("{key} needs a number, got '{v}'")),
    }
}

fn onion_spec_from(a: &Args) -> Result<overlays::OnionSpec, String> {
    let d = overlays::OnionSpec { enabled: true, ..Default::default() };
    let whole = |k: &str, def: u32| -> Result<u32, String> {
        let v = num(a, k, def as f32)?;
        if v < 0.0 || v.fract() != 0.0 {
            return Err(format!("{k} needs a whole number"));
        }
        Ok(v as u32)
    };
    let spec = overlays::OnionSpec {
        before: whole("--before", d.before)?,
        after: whole("--after", d.after)?,
        step: whole("--step", d.step)?,
        mode: if a.flags.iter().any(|f| f == "--keyed") { overlays::OnionMode::Keyed } else { overlays::OnionMode::Relative },
        tint_before: a.opts.get("--tint-before").cloned().unwrap_or(d.tint_before.clone()),
        tint_after: a.opts.get("--tint-after").cloned().unwrap_or(d.tint_after.clone()),
        opacity: num(a, "--opacity", d.opacity)?,
        falloff: num(a, "--falloff", d.falloff)?,
        outline: a.flags.iter().any(|f| f == "--outline"),
        moving_only: !a.flags.iter().any(|f| f == "--all-parts"),
        layers: a.opts.get("--layers").map(|l| l.split(',').filter(|x| !x.is_empty()).map(String::from).collect()).unwrap_or_default(),
        ..d
    };
    spec.validate()?;
    Ok(spec)
}

fn write_frame(e: &Engine, frame: f32, out: &Path, a: &Args) -> Result<(), String> {
    let bones = a.flags.iter().any(|f| f == "--bones");
    let png = out.extension().map(|x| x.eq_ignore_ascii_case("png")).unwrap_or(false);
    let svg = out.extension().map(|x| x.eq_ignore_ascii_case("svg")).unwrap_or(false);
    if !png && !svg {
        return Err(format!("render writes .png or .svg; '{}' has another extension (nothing was written)", out.display()));
    }
    let proxy = match a.opts.get("--proxy") {
        None => 0u8,
        Some(v) => v.parse::<u8>().map_err(|_| format!("--proxy needs a level 0..={}, got '{v}'", osatura_core::scrub::MAX_LEVEL))?,
    };
    osatura_core::scrub::check_level(proxy)?;
    e.evaluate_checked(frame)?;
    let bytes = if proxy > 0 && png {
        osatura_core::scrub::render_proxy_png(e, frame, proxy, a.scale, bones)?
    } else if proxy > 0 {
        osatura_core::scrub::proxy_svg_vector(e, frame, proxy, bones)?.into_bytes()
    } else if png {
        render_png(e, frame, &RasterOptions { scale: a.scale, show_bones: bones, transparent: false })?
    } else {
        render_frame(e, frame, &RenderOptions { show_bones: bones }).into_bytes()
    };
    fwrite(out, bytes)
}

/// `render-gpu`: wgpu render of one frame to PNG; `--bench N` also times N GPU and CPU renders.
#[cfg(feature = "gpu")]
fn render_gpu_cmd(a: &Args, path: &str, out: &str) -> Result<(), String> {
    use osatura_gpu::{AdapterChoice, GpuContext, GpuOptions};
    let e = load_sized(path, a)?;
    let frame: f32 = match a.opts.get("--frame") {
        None => 0.0,
        Some(v) => v.parse().map_err(|_| format!("bad frame number '{v}'"))?,
    };
    let choice: AdapterChoice = a.opts.get("--adapter").map(|s| s.parse()).transpose()?.unwrap_or(AdapterChoice::Auto);
    let ctx = GpuContext::new(choice).map_err(|e| e.to_string())?;
    let opts = GpuOptions { scale: a.scale, show_bones: a.flags.iter().any(|f| f == "--bones"), ..Default::default() };
    let img = osatura_gpu::render(&ctx, &e, frame, &opts).map_err(|e| e.to_string())?;
    fwrite(out, img.encode_png()?)?;
    println!("wrote {out} ({}x{}) on {}", img.width, img.height, ctx.adapter);
    if ctx.adapter.is_software {
        println!("note: software adapter (CPU implementation of the GPU API); timings below say nothing about real GPUs");
    }
    if let Some(n) = a.opts.get("--bench") {
        let n: usize = n.parse().ok().filter(|n| *n > 0 && *n <= 10_000).ok_or("--bench needs a count in 1..=10000")?;
        let f = e.evaluate(frame);
        let t = std::time::Instant::now();
        for _ in 0..n {
            osatura_gpu::render_frame(&ctx, &e, &f, &opts).map_err(|e| e.to_string())?;
        }
        let gpu_ms = t.elapsed().as_secs_f64() * 1e3 / n as f64;
        let ro = RasterOptions { scale: a.scale, show_bones: false, transparent: false };
        let t = std::time::Instant::now();
        for _ in 0..n {
            osatura_core::raster::pixmap_from_frame(&e, &f, frame, &ro);
        }
        let cpu_ms = t.elapsed().as_secs_f64() * 1e3 / n as f64;
        println!("bench ({n} runs, evaluate excluded): gpu {gpu_ms:.2} ms/frame, cpu (tiny-skia) {cpu_ms:.2} ms/frame, supersample {}x, {} passes", img.timing.supersample, img.timing.passes);
    }
    Ok(())
}

/// Writes a project file. The text is parsed back and run through `Engine::new` first, so a generator can never
/// leave a file behind that the other commands then reject; the real cause is reported ("output would be
/// invalid: ..."). The write is atomic (temp file + rename; symbolic links are refused).
///
/// Relative asset paths (image files, normal maps, sound tracks) are validated against the directory of the INPUT
/// project. When the output goes to another directory they are rewritten to stay valid from there if the file lies
/// below the output directory; otherwise a warning on stderr says which assets the output will not find (a project
/// may not use `..`).
fn write_json(p: &Project, out: &str) -> Result<(), String> {
    let text = p.to_json_pretty();
    validate_project_text(&text)?;
    let from = osatura_core::assets::default_dir();
    let to = asset_dir(out);
    let mut q = p.clone();
    let notes = osatura_core::assets::rewrite_relative_assets(&mut q, &from, &to);
    for n in &notes {
        eprintln!("warning: {out}: {n}");
    }
    let text = if q == *p { text } else { q.to_json_pretty() };
    fwrite(out, text.as_bytes())
}

/// `output would be invalid: <cause>` unless `text` is a project that parses and passes `Engine::new`.
fn validate_project_text(text: &str) -> Result<(), String> {
    let back = Project::from_json(text).map_err(|e| format!("output would be invalid: {e}"))?;
    Engine::new(back).map(|_| ()).map_err(|errs| format!("output would be invalid: {}", errs.join("; ")))
}

/// Atomic file write used for every output of the CLI (see `osatura_core::fsutil::write_atomic`).
fn fwrite(out: impl AsRef<Path>, data: impl AsRef<[u8]>) -> Result<(), String> {
    let out = out.as_ref();
    let special = fs::metadata(out).map(|m| !m.is_file()).unwrap_or(false);
    if special {
        // /dev/stdout, a FIFO, ...: nothing to rename over
        return fs::write(out, data).map_err(|e| format!("cannot write {}: {e}", out.display()));
    }
    osatura_core::fsutil::write_atomic(out, data.as_ref()).map_err(|e| format!("cannot write {}: {e}", out.display()))
}

/// Read a motion library (`take-record` file): every error names the file.
fn load_library(path: &str) -> Result<motionlib::Library, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("cannot read the take library {path}: {e}"))?;
    motionlib::Library::from_json(&text).map_err(|e| format!("{path}: {e}"))
}

/// `from=to,from=to` retargeting words of `take-apply`: every entry must be `from=to` and `from` must be a target the
/// take really has (a stray word such as `speed=1` used to be taken for a rename of a target called `speed`).
fn parse_take_rename(s: &str, take: &motionlib::Take) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for item in s.split(',').filter(|x| !x.is_empty()) {
        let Some((from, to)) = item.split_once('=').filter(|(f, t)| !f.is_empty() && !t.is_empty()) else {
            return Err(format!("'{item}' is not a rename: write from=to (take target = project target), or '-' for none; the speed is a plain number after it (nothing was written)"));
        };
        if !take.tracks.iter().any(|t| t.target == from) {
            let mut targets: Vec<&str> = take.tracks.iter().map(|t| t.target.as_str()).collect();
            targets.sort();
            targets.dedup();
            return Err(format!("rename '{item}': take '{}' has no track for '{from}' (its targets: {}); the speed is a plain number, not speed=N (nothing was written)", take.name, targets.join(", ")));
        }
        out.push((from.to_string(), to.to_string()));
    }
    Ok(out)
}

/// A frame given on the command line must lie in `0..=frames-1` (the studio's `/api/frame` refuses anything else).
/// `clamp` opts into the nearest valid frame instead of the error.
fn check_frame_in_range(frame: f32, frames: u32, clamp: bool) -> Result<f32, String> {
    let last = frames.saturating_sub(1) as f32;
    if !frame.is_finite() {
        return Err(format!("frame {frame} is not a number the animation has (valid frames are 0..{last})"));
    }
    if (0.0..=last).contains(&frame) {
        return Ok(frame);
    }
    if clamp {
        let c = frame.clamp(0.0, last);
        eprintln!("warning: frame {frame} is outside the animation (0..{last}); --clamp renders frame {c}");
        return Ok(c);
    }
    Err(format!("frame {frame} is outside the animation: the project has {frames} frames, so the valid frames are 0..{last}; pass --clamp to render the nearest frame instead"))
}

/// Refuses an output path that is the input project itself (render/export/... would silently destroy it).
fn guard_out(input: &str, out: &str) -> Result<(), String> {
    let same = match (fs::canonicalize(input), fs::canonicalize(out)) {
        (Ok(a), Ok(b)) => a == b,
        _ => Path::new(input) == Path::new(out),
    };
    if same {
        Err(format!("refusing to overwrite the input project {input} with output of the same name"))
    } else {
        Ok(())
    }
}

/// Extra ffmpeg arguments that mix the project's sound tracks (none when it has no sounds).
/// Inputs must come before the output; the filter/map/codec go after the video options.
fn audio_inputs(e: &Engine, base: &Path) -> Result<(Vec<String>, Vec<String>, bool), String> {
    Ok(match soundtrack::mux_args(&e.project, base, 1, None)? {
        Some(m) => (m.inputs, vec!["-filter_complex".into(), m.filter, "-map".into(), "0:v".into(), "-map".into(), "[aout]".into(), "-t".into(), format!("{}", e.project.frames as f32 / e.project.fps)], true),
        None => (vec![], vec![], false),
    })
}

/// Seconds from `1.5`, `1.5s` or a frame count such as `36f` (at the project frame rate).
fn parse_time(s: &str, fps: f32, what: &str) -> Result<f32, String> {
    let (num, scale) = match s.strip_suffix('f') {
        Some(n) => (n, 1.0 / fps),
        None => (s.strip_suffix('s').unwrap_or(s), 1.0),
    };
    num.parse::<f32>().ok().filter(|v| v.is_finite()).map(|v| v * scale).ok_or_else(|| format!("{what} needs seconds (1.5) or frames (36f), got '{s}'"))
}

/// Apply --trim-in/--trim-out/--fade-in/--fade-out to a track (a trim-out of 0 means "to the end").
fn apply_sound_opts(t: &mut soundtrack::SoundTrack, a: &Args, fps: f32) -> Result<(), String> {
    if let Some(v) = a.opts.get("--trim-in") { t.trim_in = parse_time(v, fps, "--trim-in")?; }
    if let Some(v) = a.opts.get("--trim-out") { let o = parse_time(v, fps, "--trim-out")?; t.trim_out = if o == 0.0 { None } else { Some(o) }; }
    if let Some(v) = a.opts.get("--fade-in") { t.fade_in = parse_time(v, fps, "--fade-in")?; }
    if let Some(v) = a.opts.get("--fade-out") { t.fade_out = parse_time(v, fps, "--fade-out")?; }
    Ok(())
}

/// GIF and APNG have no audio track: say so instead of silently dropping the sounds.
/// Prints every export warning on stderr and, with `--report FILE`, writes them as JSON
/// (`{"format","count","warnings":[{"code","layer","message"}]}`). The file may not be the input project.
fn report_warnings(a: &Args, project: &str, format: &str, warns: Vec<(&'static str, String, String)>) -> Result<(), String> {
    for (code, layer, msg) in &warns {
        if layer.is_empty() {
            eprintln!("warning: [{code}] {msg}");
        } else {
            eprintln!("warning: [{code}] {layer}: {msg}");
        }
    }
    if let Some(r) = a.opts.get("--report") {
        guard_out(project, r)?;
        let v = serde_json::json!({
            "format": format,
            "count": warns.len(),
            "warnings": warns.iter().map(|(c, l, m)| serde_json::json!({"code": c, "layer": l, "message": m})).collect::<Vec<_>>(),
        });
        fwrite(r, serde_json::to_string_pretty(&v).unwrap()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn warn_silent(e: &Engine, kind: &str) {
    if !e.project.sounds.is_empty() {
        eprintln!("note: {kind} cannot carry audio; the project's {} sound track(s) are not included (use .mp4, .webm or .html)", e.project.sounds.len());
    }
}

/// The mixed sound as an mp3 for the HTML player, or None (with a warning on stderr) when there is
/// none, ffmpeg fails, or it would be too large to embed.
fn html_audio(e: &Engine, base: &Path) -> Option<(&'static str, Vec<u8>)> {
    if e.project.sounds.is_empty() {
        return None;
    }
    match soundio::mix_bytes(&e.project, base, "mp3") {
        Ok((_, b)) if b.len() > export::HTML_AUDIO_MAX => {
            eprintln!("warning: the mixed audio is {:.1} MB (limit {} MB for the HTML player); exporting without sound - shorten the sounds or use .mp4", b.len() as f64 / 1048576.0, export::HTML_AUDIO_MAX >> 20);
            None
        }
        Ok((m, b)) => {
            if b.len() > export::HTML_AUDIO_WARN {
                eprintln!("warning: the embedded audio is {:.1} MB; the HTML file will be about {:.1} MB larger", b.len() as f64 / 1048576.0, b.len() as f64 * 1.34 / 1048576.0);
            }
            Some((m, b))
        }
        Err(err) => {
            eprintln!("warning: audio not embedded: {err}");
            None
        }
    }
}

fn export_mp4(e: &Engine, out: &str, scale: f32, base: &Path) -> Result<(), String> {
    osatura_core::fsutil::refuse_symlink(Path::new(out))?;
    let guard = osatura_core::fsutil::private_temp_dir("osatura-mp4")?;
    let dir = guard.path().to_path_buf();
    let (ain, aargs, has_audio) = audio_inputs(e, base)?;
    let result = (|| {
        for f in 0..e.project.frames {
            let png = render_png(e, f as f32, &RasterOptions { scale, show_bones: false, transparent: false })?;
            fwrite(dir.join(format!("f_{f:05}.png")), png).map_err(|e| e.to_string())?;
        }
        let status = Command::new("ffmpeg")
            .args(["-y", "-loglevel", "error", "-protocol_whitelist", "file", "-framerate", &e.project.fps.to_string(), "-i"])
            .arg(dir.join("f_%05d.png"))
            .args(&ain)
            .args(["-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2", "-pix_fmt", "yuv420p", "-c:v", "libx264"])
            .args(&aargs)
            .args(if has_audio { &["-c:a", "aac"][..] } else { &[][..] })
            .args(["-protocol_whitelist", "file"])
            .arg(osatura_core::fsutil::file_arg(out))
            .status()
            .map_err(|e| format!("cannot run ffmpeg ({e}); install it or export a GIF/HTML instead"))?;
        if status.success() { Ok(()) } else { Err("ffmpeg failed".to_string()) }
    })();
    drop(guard);
    result
}

/// WebM with alpha. VP9 (default) or AV1, encoded by the user's own ffmpeg (external process, never bundled); patent/licensing status of the codecs is not assessed here (docs/PATENT_NOTES.md). Needs ffmpeg on PATH.
fn export_webm(e: &Engine, out: &str, scale: f32, codec: &str, base: &Path) -> Result<(), String> {
    let enc: &[&str] = match codec {
        "vp9" => &["-c:v", "libvpx-vp9", "-pix_fmt", "yuva420p", "-b:v", "0", "-crf", "30", "-auto-alt-ref", "0"],
        "av1" => &["-c:v", "libaom-av1", "-pix_fmt", "yuva420p", "-b:v", "0", "-crf", "35", "-cpu-used", "6"],
        other => return Err(format!("unknown --codec '{other}' (vp9 or av1)")),
    };
    osatura_core::fsutil::refuse_symlink(Path::new(out))?;
    let guard = osatura_core::fsutil::private_temp_dir("osatura-webm")?;
    let dir = guard.path().to_path_buf();
    let (ain, aargs, has_audio) = audio_inputs(e, base)?;
    let result = (|| {
        for f in 0..e.project.frames {
            let png = render_png(e, f as f32, &RasterOptions { scale, show_bones: false, transparent: false })?;
            fwrite(dir.join(format!("f_{f:05}.png")), png).map_err(|e| e.to_string())?;
        }
        let status = Command::new("ffmpeg")
            .args(["-y", "-loglevel", "error", "-protocol_whitelist", "file", "-framerate", &e.project.fps.to_string(), "-i"])
            .arg(dir.join("f_%05d.png"))
            .args(&ain)
            .args(enc)
            .args(&aargs)
            .args(if has_audio { &["-c:a", "libopus"][..] } else { &[][..] })
            .args(["-protocol_whitelist", "file"])
            .arg(osatura_core::fsutil::file_arg(out))
            .status()
            .map_err(|e| format!("cannot run ffmpeg ({e}); install it or export an APNG instead"))?;
        if status.success() { Ok(()) } else { Err("ffmpeg failed (does your build include the chosen codec?)".to_string()) }
    })();
    drop(guard);
    result
}

/// Encode already rendered PNG frames as .mp4 (H.264) or .webm (VP9/AV1 with alpha) through the user's ffmpeg.
/// Before any frame is rendered: the output's directory exists and, for video, ffmpeg is there and lists the encoder
/// the chosen format needs (a failure after minutes of rendering used to be the first sign).
fn dof_export_preflight(out: &str, a: &Args) -> Result<(), String> {
    let lower = out.to_lowercase();
    let is_video = lower.ends_with(".mp4") || lower.ends_with(".webm");
    let is_file = is_video || lower.ends_with(".gif") || lower.ends_with(".apng");
    if is_file {
        let dir = asset_dir(out);
        if !dir.is_dir() {
            return Err(format!("dof-export: the directory {} does not exist (nothing was rendered; create it first)", dir.display()));
        }
    }
    if is_video {
        let webm = lower.ends_with(".webm");
        let codec = a.opts.get("--codec").map(String::as_str).unwrap_or("vp9");
        let encoder = if webm {
            match codec {
                "vp9" => "libvpx-vp9",
                "av1" => "libaom-av1",
                other => return Err(format!("unknown --codec '{other}' (vp9 or av1)")),
            }
        } else {
            "libx264"
        };
        let listing = Command::new("ffmpeg").args(["-hide_banner", "-loglevel", "error", "-encoders"]).output().map_err(|e| format!("dof-export: cannot run ffmpeg ({e}); install it or export a GIF/APNG instead (nothing was rendered)"))?;
        if !String::from_utf8_lossy(&listing.stdout).split_whitespace().any(|w| w == encoder) {
            return Err(format!("dof-export: this ffmpeg has no {encoder} encoder, needed for {} (nothing was rendered); use another format or codec", if webm { "WebM" } else { "MP4" }));
        }
    }
    Ok(())
}

fn encode_png_video(pngs: &[Vec<u8>], fps: f32, out: &str, webm: bool, codec: &str) -> Result<(), String> {
    let enc: Vec<&str> = if webm {
        match codec {
            "vp9" => vec!["-c:v", "libvpx-vp9", "-pix_fmt", "yuva420p", "-b:v", "0", "-crf", "30", "-auto-alt-ref", "0"],
            "av1" => vec!["-c:v", "libaom-av1", "-pix_fmt", "yuva420p", "-b:v", "0", "-crf", "35", "-cpu-used", "6"],
            other => return Err(format!("unknown --codec '{other}' (vp9 or av1)")),
        }
    } else {
        vec!["-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2", "-pix_fmt", "yuv420p", "-c:v", "libx264"]
    };
    osatura_core::fsutil::refuse_symlink(Path::new(out))?;
    let guard = osatura_core::fsutil::private_temp_dir("osatura-dofvid")?;
    let dir = guard.path().to_path_buf();
    let result = (|| {
        for (i, b) in pngs.iter().enumerate() {
            fwrite(dir.join(format!("f_{i:05}.png")), b).map_err(|e| e.to_string())?;
        }
        let status = Command::new("ffmpeg")
            .args(["-y", "-loglevel", "error", "-protocol_whitelist", "file", "-framerate", &fps.to_string(), "-i"])
            .arg(dir.join("f_%05d.png"))
            .args(&enc)
            .args(["-protocol_whitelist", "file"])
            .arg(osatura_core::fsutil::file_arg(out))
            .status()
            .map_err(|e| format!("cannot run ffmpeg ({e}); install it or export a GIF/APNG instead"))?;
        if status.success() { Ok(()) } else { Err("ffmpeg failed (does your build include the chosen codec?)".to_string()) }
    })();
    drop(guard);
    result
}

/// Read a user-authored config file; unknown keys (typos) are an error, not silently ignored.
fn read_json<T: serde::Serialize + serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let text = read_text_capped(path)?;
    osatura_core::unknownfields::parse_strict(&text).map_err(|e| format!("invalid JSON in {path}: {e}"))
}

fn pose_suggester(a: &Args, project: &Project, db_path: &str) -> Result<posesuggest::Suggester, String> {
    let db = if db_path == "-" {
        let lib = match a.opts.get("--library") {
            Some(p) => Some(read_json::<osatura_core::motionlib::Library>(p)?),
            None => None,
        };
        posesuggest::build_db(Some(project), lib.as_ref(), None)?
    } else {
        posesuggest::PoseDb::from_json(&fs::read_to_string(db_path).map_err(|e| format!("cannot read {db_path}: {e}"))?)?
    };
    posesuggest::Suggester::new(db)
}

fn print_suggestions(s: &posesuggest::Suggestions) {
    println!("{}: {} library poses, {} PCA axes ({:.0}% of the variance)", s.kind, s.poses_in_library, s.pca_axes, s.pca_explained * 100.0);
    for n in &s.notes {
        println!("  note: {n}");
    }
    for (i, c) in s.candidates.iter().enumerate() {
        let err = c.constraint_error.map(|e| format!("  constraint error {e:.2}")).unwrap_or_default();
        println!("{:>2}. {:<11} score {:.2}  plausibility {:.2}{err}  key at frame {:.1}", i + 1, c.method, c.score, c.plausibility, c.frame.unwrap_or(0.0));
        println!("      {}", c.label);
        println!("      {}", c.values.iter().map(|v| format!("{}={:.1}", v.bone, v.angle)).collect::<Vec<_>>().join(" "));
        for n in &c.notes {
            println!("      note: {n}");
        }
    }
    println!("(scores rank candidates against this library; they are heuristics, not probabilities)");
}

fn run(raw: &[String]) -> Result<(), String> {
    let expanded = expand_eq(raw)?;
    let raw = expanded.as_slice();
    if let Some(n) = raw.iter().position(|x| x == "--max-input-bytes") {
        let v = raw.get(n + 1).ok_or("--max-input-bytes needs a value")?;
        let n: u64 = v.parse().ok().filter(|n| *n > 0).ok_or_else(|| format!("--max-input-bytes needs a positive whole number, got '{v}'"))?;
        MAX_INPUT_BYTES.store(n, std::sync::atomic::Ordering::SeqCst);
    }
    let a = parse_args(raw)?;
    check_flags(&a, raw)?;
    let pos: Vec<&str> = a.pos.iter().map(String::as_str).collect();
    if let Some(c) = pos.first().filter(|_| a.flags.iter().any(|f| f == "--help")) {
        if let Some(u) = usage_for(c) {
            println!("{u}");
            return Ok(());
        }
    }
    if !pos.is_empty() && a.flags.iter().any(|f| f == "--version") {
        println!("osatura {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if pos.is_empty() && a.flags.iter().any(|f| f == "--version") {
        println!("osatura {} (osatura-cli, AGPL-3.0-or-later; engine libraries MPL-2.0)\n{}", env!("CARGO_PKG_VERSION"), studio::source_offer_text(studio::default_source_url()));
        return Ok(());
    }
    // commands that read a project and write a different kind of file: never over the project itself
    match pos.as_slice() {
        ["render", p, _, o] | ["render-hdr", p, _, o] | ["render-hdr", p, _, o, _] | ["dof", p, _, o] | ["dof-export", p, o]
        | ["export", p, o] | ["layout-preview", p, o] | ["render-gpu", p, o] | ["export-fit", p, o] | ["fuzz-grid", p, o]
        | ["onion", p, o] | ["onion", p, _, o] | ["trail", p, _, o] | ["golden", _, p, o] => guard_out(p, o)?,
        // the Lottie file is the source of the import: never overwrite it with the project (or the effects file)
        ["import-lottie", inp, o, rest @ ..] => {
            guard_out(inp, o)?;
            if let Some(fx) = rest.first() {
                guard_out(inp, fx)?;
                guard_out(o, fx)?;
            }
        }
        _ => {}
    }
    if let Some(r) = cmds_1::run(&a, raw, &pos) {
        return r;
    }
    if let Some(r) = cmds_2::run(&a, raw, &pos) {
        return r;
    }
    if let Some(r) = cmds_3::run(&a, raw, &pos) {
        return r;
    }
    if let Some(r) = cmds_4::run(&a, raw, &pos) {
        return r;
    }
    if let Some(r) = cmds_5::run(&a, raw, &pos) {
        return r;
    }
    if let Some(r) = cmds_6::run(&a, raw, &pos) {
        return r;
    }
    match pos.as_slice() {
        ["ui", rest @ ..] => ui_cmd(&a, rest),
        _ => Err(USAGE.to_string()),
    }
}

/// `osatura ui ...`: look at the studio command registry, shortcuts and layouts without a browser.
fn ui_cmd(a: &Args, rest: &[&str]) -> Result<(), String> {
    use uiconfig::*;
    let json = a.flags.iter().any(|f| f == "--json");
    let store = || -> UiStore {
        let p = a.opts.get("--ui-config").map(std::path::PathBuf::from).or_else(default_config_path).filter(|p| p.exists());
        UiStore::open(p)
    };
    match rest {
        ["commands"] => {
            let s = store();
            if json {
                println!("{}", s.view()["commands"]);
                return Ok(());
            }
            let eff = effective_bindings(&s.cfg.shortcuts).unwrap_or_default();
            let mut cat = "";
            for c in COMMANDS {
                if c.category != cat {
                    cat = c.category;
                    println!("{cat}");
                }
                let keys: Vec<String> = eff.iter().filter(|b| b.command == c.id).map(|b| format!("{}:{}", b.context.name(), b.chord)).collect();
                println!("  {:<28} {:<34} {}", c.id, c.title, keys.join("  "));
            }
            println!("{} commands", COMMANDS.len());
            Ok(())
        }
        ["shortcuts"] => {
            let s = store();
            let eff = effective_bindings(&s.cfg.shortcuts).map_err(|e| e.join("\n"))?;
            if json {
                let rows: Vec<serde_json::Value> = eff.iter().map(|b| serde_json::json!({"command": b.command, "context": b.context.name(), "chord": b.chord.to_string(), "user": b.user})).collect();
                println!("{}", serde_json::Value::Array(rows));
                return Ok(());
            }
            for ctx in Context::ALL {
                println!("{}", ctx.name());
                let mut rows: Vec<&Binding> = eff.iter().filter(|b| b.context == ctx).collect();
                rows.sort_by(|x, y| x.chord.cmp(&y.chord).then(x.command.cmp(&y.command)));
                for b in rows {
                    println!("  {:<22} {}{}", b.chord.to_string(), b.command, if b.user { "  (yours)" } else { "" });
                }
            }
            let cf = conflicts(&eff);
            for c in &cf {
                println!("CONFLICT {}: {} vs {}", c.chord, c.a.0, c.b.0);
            }
            println!("{} bindings, {} conflicts", eff.len(), cf.len());
            Ok(())
        }
        ["layout", "presets"] => {
            for n in PRESET_NAMES {
                let l = preset(n).ok_or("preset missing")?;
                let ok = l.validate().and_then(|_| l.fits(1100.0, 640.0).map_err(|e| vec![e]));
                println!("{:<8} {}  [{}]", n, l.summary(), if ok.is_ok() { "valid" } else { "INVALID" });
            }
            Ok(())
        }
        ["layout", "show", name] => {
            let l = preset(name).ok_or_else(|| format!("no preset '{name}' (try: {})", PRESET_NAMES.join(", ")))?;
            println!("{}", serde_json::to_string_pretty(&l).map_err(|e| e.to_string())?);
            Ok(())
        }
        ["layout", "validate", file] => {
            let t = fs::read_to_string(file).map_err(|e| format!("cannot read {file}: {e}"))?;
            let l: Layout = serde_json::from_str(&t).map_err(|e| format!("{file}: {e}"))?;
            l.validate().map_err(|e| format!("{file} is not a valid layout:\n  - {}", e.join("\n  - ")))?;
            println!("{file}: valid layout ({})", l.summary());
            Ok(())
        }
        ["search", q @ ..] if !q.is_empty() => {
            let s = store();
            let v = s.search_view(&q.join(" "), 20);
            if json {
                println!("{v}");
                return Ok(());
            }
            let hits = v["hits"].as_array().cloned().unwrap_or_default();
            for h in &hits {
                let keys: Vec<&str> = h["bindings"].as_array().map(|b| b.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
                println!("{:>4}  {:<28} {:<30} {}", h["score"], h["id"].as_str().unwrap_or(""), h["title"].as_str().unwrap_or(""), keys.join("  "));
            }
            if hits.is_empty() {
                println!("no command matches '{}'", q.join(" "));
            }
            Ok(())
        }
        _ => Err("usage: osatura ui commands|shortcuts [--json] | ui layout presets|show <name>|validate <file> | ui search <query>".into()),
    }
}

fn main() -> ExitCode {
    // Windows gives the main thread a 1 MiB stack (Linux/macOS: 8 MiB); deep recursion in the evaluator
    // and the renderers overflowed it (found by the Windows CI run). Run everything on a big-stack thread.
    match std::thread::Builder::new().stack_size(256 << 20).spawn(real_main) {
        Ok(h) => h.join().unwrap_or(ExitCode::FAILURE),
        Err(_) => real_main(),
    }
}

/// True when this process is the only one attached to its console, which is what happens when the .exe is opened
/// from Explorer (a terminal's shell is a second process on the console).
#[cfg(windows)]
fn launched_by_double_click() -> bool {
    extern "system" {
        fn GetConsoleProcessList(list: *mut u32, count: u32) -> u32;
    }
    let mut ids = [0u32; 4];
    // SAFETY: `ids` is a valid buffer of the stated length; the call only writes process ids into it.
    let n = unsafe { GetConsoleProcessList(ids.as_mut_ptr(), ids.len() as u32) };
    n == 1
}

#[cfg(windows)]
fn wait_for_enter(msg: &str) {
    println!("{msg}");
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
}

/// Double-click on Windows: instead of printing the usage and closing the window at once, create a first
/// project (never overwriting an existing one) in `Documents\OSATURA`, open the studio in the default browser
/// and keep running until the window is closed.
#[cfg(windows)]
fn double_click_launcher() -> ExitCode {
    println!("OSATURA {}  -  command-line tool and browser studio", env!("CARGO_PKG_VERSION"));
    println!("This window runs the studio. Close it (or press Ctrl+C) to stop. For the commands, open a terminal: osatura --help\n");
    let home = env::var("USERPROFILE").unwrap_or_else(|_| ".".into());
    let dir = std::path::Path::new(&home).join("Documents").join("OSATURA");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        wait_for_enter(&format!("cannot create {}: {e}\nPress Enter to close.", dir.display()));
        return ExitCode::FAILURE;
    }
    let project = dir.join("first-rig.json").to_string_lossy().into_owned();
    if !std::path::Path::new(&project).exists() {
        if let Err(e) = run(&["demo-person".to_string(), project.clone()]) {
            wait_for_enter(&format!("{e}\nPress Enter to close."));
            return ExitCode::FAILURE;
        }
    }
    println!("Project: {project} (edits are saved there)");
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let _ = Command::new("cmd").args(["/C", "start", "", "http://127.0.0.1:8080"]).spawn();
    });
    let r = run(&["studio".to_string(), project, "--port".to_string(), "8080".to_string()]);
    if let Err(e) = r {
        wait_for_enter(&format!("{e}\n(If another program uses port 8080, close it, or run: osatura studio <project.json> --port 8090)\nPress Enter to close."));
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn real_main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    // project files may only reference files inside their own directory unless this is given
    osatura_core::assets::set_allow_external(args.iter().any(|a| a == "--allow-external-assets"));
    osatura_core::fsutil::set_follow_symlinks(args.iter().any(|a| a == "--follow-symlinks"));
    #[cfg(windows)]
    if args.is_empty() && launched_by_double_click() {
        return double_click_launcher();
    }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            match ci_cmd::EXIT.load(std::sync::atomic::Ordering::SeqCst) {
                0 | 1 => ExitCode::FAILURE,
                c => ExitCode::from(c),
            }
        }
    }
}

#[cfg(test)]
mod usage_tests;
