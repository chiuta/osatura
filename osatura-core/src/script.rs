// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Sandboxed scripting (Rhai, MIT/Apache-2.0) over the editor's validated operations.
//!
//! A script cannot touch files, the network or the process. It reads a snapshot of the project
//! (`fps()`, `frames()`, `bones()`, `layers()`) and *requests* keys with `set_key(...)`, bones with
//! `add_bone(name, parent_or_empty, length, x, y, angle)` and layers with `add_rect(name, x, y, w, h, fill)` /
//! `add_circle(name, x, y, r, fill)`, `add_polygon` / `add_blob(name, [x0, y0, x1, y1, ...], fill)`,
//! `add_line(name, points, colour, width, closed)`, `add_gradient_rect(name, x, y, w, h, top, bottom)`,
//! `add_ik(name, [bones], target, bend)` and `add_aim(name, bone, target[, offset, weight])`,
//! `add_copy_rotation(name, bone, source, weight)` and `add_pin(name, bone, target, weight)`.
//! Structure: `add_group(name[, [layer paths]])` / `add_switch(name, [layer paths])` wrap adjacent existing layers
//! (draw order and world transform are kept, references are rewritten), `move_layer(path, group_or_empty)`,
//! `set_parent(bone, parent_or_empty)` (keeps the rest pose), `delete_layer(path)` and `delete_bone(name)` (refused
//! while anything still needs the bone). Static properties: `set_layer(path, prop, value)` and `set_bone(name, prop, value)`.
//! Procedural channels: `add_channel_expr(target, prop, expr[, mode])` and `add_noise(target, prop, amp, freq, seed[, mode])`.
//! Metadata: `add_marker(name, frame[, end])` and `add_event(name, frame[, data])`.
//! Read helpers (of the project as it was when the script started): `bone_info(name)`, `layer_info(path)` (maps, or `()` when
//! missing) and `world(bone, frame)` (head position and angle of the evaluated pose). Requests
//! are applied afterwards through the validated [`Editor`] (so validation and channel locks apply),
//! one undo step each, and if any request is rejected, all of them are undone.

use crate::anim::Ease;
use crate::edit::Editor;
use crate::project::Prop;
use rhai::{Dynamic, Engine as Rhai, ImmutableString};
use std::cell::RefCell;
use std::rc::Rc;

const MAX_REQUESTS: usize = 5000;
const MAX_SCRIPT_BYTES: usize = 64 * 1024;

#[derive(Debug, Default, PartialEq)]
pub struct ScriptReport {
    pub applied: usize,
    pub log: Vec<String>,
}

type EditFn = Box<dyn Fn(&mut crate::project::Project) -> Result<(), String>>;

enum Request {
    Key { target: String, prop: Prop, frame: f32, value: f32, ease: Ease },
    Bone(crate::project::Bone),
    Layer(Box<crate::project::Layer>),
    Ik(crate::project::IkChain),
    Constraint(crate::project::Constraint),
    /// Edit of a copy of the project; the named top-level JSON sections are then stored as one step.
    Edit { what: String, sections: &'static [&'static str], f: EditFn },
}

const REF_SECTIONS: &[&str] = &["layers", "channels", "channel_meta", "skins", "comps"];

type Rq = Rc<RefCell<Vec<Request>>>;

fn queue(rq: &Rq, r: Request) -> Result<(), Box<rhai::EvalAltResult>> {
    if rq.borrow().len() >= MAX_REQUESTS {
        return Err(format!("a script may make at most {MAX_REQUESTS} requests").into());
    }
    rq.borrow_mut().push(r);
    Ok(())
}

fn edit(rq: &Rq, what: String, sections: &'static [&'static str], f: impl Fn(&mut crate::project::Project) -> Result<(), String> + 'static) -> Result<(), Box<rhai::EvalAltResult>> {
    queue(rq, Request::Edit { what, sections, f: Box::new(f) })
}

fn names(a: &rhai::Array) -> Result<Vec<String>, Box<rhai::EvalAltResult>> {
    if a.len() > 1000 {
        return Err("at most 1000 names".into());
    }
    a.iter().map(|b| b.clone().into_string().map_err(|_| "names must be strings".into())).collect()
}

fn val(d: &Dynamic) -> Result<crate::scriptops::Val, Box<rhai::EvalAltResult>> {
    use crate::scriptops::Val;
    if let Ok(b) = d.as_bool() {
        return Ok(Val::Bool(b));
    }
    if let Ok(s) = d.clone().into_string() {
        return Ok(Val::Str(s));
    }
    num(d).map(Val::Num).map_err(|e| e.into())
}

fn to_json(d: &Dynamic, depth: usize) -> Result<serde_json::Value, String> {
    use serde_json::Value;
    if depth > 4 {
        return Err("event data is nested too deeply".into());
    }
    if d.is_unit() {
        return Ok(Value::Null);
    }
    if let Ok(b) = d.as_bool() {
        return Ok(Value::Bool(b));
    }
    if let Ok(i) = d.as_int() {
        return Ok(Value::from(i));
    }
    if d.is_float() {
        return serde_json::Number::from_f64(num(d)? as f64).map(Value::Number).ok_or_else(|| "number must be finite".to_string());
    }
    if let Ok(s) = d.clone().into_string() {
        return if s.len() <= 1000 { Ok(Value::String(s)) } else { Err("event data strings are limited to 1000 bytes".into()) };
    }
    if let Some(a) = d.clone().try_cast::<rhai::Array>() {
        if a.len() > 100 {
            return Err("event data arrays are limited to 100 items".into());
        }
        return a.iter().map(|x| to_json(x, depth + 1)).collect::<Result<Vec<_>, _>>().map(Value::Array);
    }
    if let Some(m) = d.clone().try_cast::<rhai::Map>() {
        if m.len() > 100 {
            return Err("event data objects are limited to 100 keys".into());
        }
        let mut o = serde_json::Map::new();
        for (k, v) in m.iter() {
            o.insert(k.to_string(), to_json(v, depth + 1)?);
        }
        return Ok(Value::Object(o));
    }
    Err("event data must be numbers, strings, booleans, arrays or maps".into())
}

fn mode_of(m: &str) -> Result<crate::project::Mode, Box<rhai::EvalAltResult>> {
    use crate::project::Mode;
    match m {
        "replace" => Ok(Mode::Replace),
        "add" => Ok(Mode::Add),
        "multiply" => Ok(Mode::Multiply),
        _ => Err(format!("unknown mode '{m}' (replace, add, multiply)").into()),
    }
}

fn weight01(w: f32) -> Result<f32, Box<rhai::EvalAltResult>> {
    if (0.0..=1.0).contains(&w) { Ok(w) } else { Err("weight must be within 0..1".into()) }
}

fn int_u32(d: &Dynamic) -> Result<u32, String> {
    d.as_int().ok().and_then(|i| u32::try_from(i).ok()).ok_or_else(|| "seed must be an integer 0..4294967295".to_string())
}

fn num(d: &Dynamic) -> Result<f32, String> {
    let v = d.as_float().map(|f| f as f32).or_else(|_| d.as_int().map(|i| i as f32)).map_err(|_| "expected a number".to_string())?;
    if v.is_finite() { Ok(v) } else { Err("number must be finite".into()) }
}

fn colour(c: &str) -> Result<(), Box<rhai::EvalAltResult>> {
    if crate::style::Rgba::parse(c).is_none() {
        return Err(format!("bad colour '{c}' (use #rgb, #rrggbb, #rrggbbaa or black/white/red/green/blue/yellow/transparent)").into());
    }
    Ok(())
}

pub fn run_script(editor: &mut Editor, src: &str) -> Result<ScriptReport, String> {
    if src.len() > MAX_SCRIPT_BYTES {
        return Err("script is larger than 64 KB".into());
    }
    let p = editor.project();
    let (fps, frames) = (p.fps as f64, p.frames as i64);
    let bones: Vec<Dynamic> = p.bones.iter().map(|b| Dynamic::from(b.name.clone())).collect();
    let layers: Vec<Dynamic> = p.layers.iter().map(|l| Dynamic::from(l.name.clone())).collect();

    let snap = Rc::new(p.clone());
    let requests: Rc<RefCell<Vec<Request>>> = Rc::default();
    let log: Rc<RefCell<Vec<String>>> = Rc::default();

    let mut eng = Rhai::new();
    eng.set_max_operations(500_000);
    eng.set_max_call_levels(32);
    eng.set_max_expr_depths(64, 32);
    eng.set_max_string_size(10_000);
    eng.set_max_array_size(10_000);
    eng.set_max_map_size(1_000);
    {
        let l = log.clone();
        eng.on_print(move |s| {
            if l.borrow().len() < 1000 {
                l.borrow_mut().push(s.to_string());
            }
        });
    }
    eng.on_debug(|_, _, _| {});
    eng.register_fn("fps", move || fps);
    eng.register_fn("frames", move || frames);
    eng.register_fn("bones", move || bones.clone());
    eng.register_fn("layers", move || layers.clone());
    let add = |rq: &Rc<RefCell<Vec<Request>>>, target: ImmutableString, prop: ImmutableString, frame: Dynamic, value: Dynamic, ease: Option<ImmutableString>| -> Result<(), Box<rhai::EvalAltResult>> {
        let err = |m: String| -> Box<rhai::EvalAltResult> { m.into() };
        let prop = Prop::parse(&prop).ok_or_else(|| err(format!("unknown property '{prop}'")))?;
        let ease = match ease {
            Some(e) => serde_json::from_value::<Ease>(serde_json::json!({"type": e.as_str()})).map_err(|_| err(format!("unknown ease '{e}'")))?,
            None => Ease::Linear,
        };
        if rq.borrow().len() >= MAX_REQUESTS {
            return Err(err(format!("a script may request at most {MAX_REQUESTS} keys")));
        }
        rq.borrow_mut().push(Request::Key { target: target.to_string(), prop, frame: num(&frame).map_err(err)?, value: num(&value).map_err(err)?, ease });
        Ok(())
    };
    {
        let rq = requests.clone();
        eng.register_fn("set_key", move |t: ImmutableString, p: ImmutableString, f: Dynamic, v: Dynamic| add(&rq, t, p, f, v, None));
    }
    {
        let rq = requests.clone();
        eng.register_fn("set_key", move |t: ImmutableString, p: ImmutableString, f: Dynamic, v: Dynamic, e: ImmutableString| add(&rq, t, p, f, v, Some(e)));
    }
    // creation requests: bones and simple shape layers (validated when applied)
    {
        let rq = requests.clone();
        eng.register_fn("add_bone", move |name: ImmutableString, parent: ImmutableString, length: Dynamic, x: Dynamic, y: Dynamic, angle: Dynamic| -> Result<(), Box<rhai::EvalAltResult>> {
            let err = |m: String| -> Box<rhai::EvalAltResult> { m.into() };
            if rq.borrow().len() >= MAX_REQUESTS {
                return Err(err(format!("a script may make at most {MAX_REQUESTS} requests")));
            }
            let parent = if parent.is_empty() { None } else { Some(parent.as_str()) };
            let b = crate::project::Bone::new(&name, parent, num(&length).map_err(err)?, crate::math::Vec2::new(num(&x).map_err(err)?, num(&y).map_err(err)?), num(&angle).map_err(err)?);
            rq.borrow_mut().push(Request::Bone(b));
            Ok(())
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_rect", move |name: ImmutableString, x: Dynamic, y: Dynamic, w: Dynamic, h: Dynamic, fill: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let err = |m: String| -> Box<rhai::EvalAltResult> { m.into() };
            if rq.borrow().len() >= MAX_REQUESTS {
                return Err(err(format!("a script may make at most {MAX_REQUESTS} requests")));
            }
            colour(&fill)?;
            let path = crate::path::VPath::rect("shape", num(&x).map_err(err)?, num(&y).map_err(err)?, num(&w).map_err(err)?, num(&h).map_err(err)?).with_fill(&fill);
            rq.borrow_mut().push(Request::Layer(Box::new(crate::project::Layer::vector(&name, vec![path]))));
            Ok(())
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_circle", move |name: ImmutableString, x: Dynamic, y: Dynamic, r: Dynamic, fill: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let err = |m: String| -> Box<rhai::EvalAltResult> { m.into() };
            if rq.borrow().len() >= MAX_REQUESTS {
                return Err(err(format!("a script may make at most {MAX_REQUESTS} requests")));
            }
            colour(&fill)?;
            let c = crate::math::Vec2::new(num(&x).map_err(err)?, num(&y).map_err(err)?);
            let path = crate::path::VPath::circle("shape", c, num(&r).map_err(err)?).with_fill(&fill);
            rq.borrow_mut().push(Request::Layer(Box::new(crate::project::Layer::vector(&name, vec![path]))));
            Ok(())
        });
    }

    // polygons / smooth curves from flat coordinate arrays [x0, y0, x1, y1, ...]
    fn points(a: &rhai::Array) -> Result<Vec<crate::math::Vec2>, String> {
        if a.len() < 6 || !a.len().is_multiple_of(2) || a.len() > 2000 {
            return Err("points need an even number of coordinates: 3..1000 points".into());
        }
        a.chunks(2).map(|c| Ok(crate::math::Vec2::new(num(&c[0])?, num(&c[1])?))).collect()
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_polygon", move |name: ImmutableString, pts: rhai::Array, fill: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let err = |m: String| -> Box<rhai::EvalAltResult> { m.into() };
            if rq.borrow().len() >= MAX_REQUESTS {
                return Err(err(format!("a script may make at most {MAX_REQUESTS} requests")));
            }
            colour(&fill)?;
            let path = crate::path::VPath::polygon("shape", &points(&pts).map_err(err)?).with_fill(&fill);
            rq.borrow_mut().push(Request::Layer(Box::new(crate::project::Layer::vector(&name, vec![path]))));
            Ok(())
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_blob", move |name: ImmutableString, pts: rhai::Array, fill: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let err = |m: String| -> Box<rhai::EvalAltResult> { m.into() };
            if rq.borrow().len() >= MAX_REQUESTS {
                return Err(err(format!("a script may make at most {MAX_REQUESTS} requests")));
            }
            colour(&fill)?;
            let path = crate::path::VPath::smooth_through("shape", &points(&pts).map_err(err)?, true, 1.0).with_fill(&fill);
            rq.borrow_mut().push(Request::Layer(Box::new(crate::project::Layer::vector(&name, vec![path]))));
            Ok(())
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_line", move |name: ImmutableString, pts: rhai::Array, color: ImmutableString, width: Dynamic, closed: bool| -> Result<(), Box<rhai::EvalAltResult>> {
            let err = |m: String| -> Box<rhai::EvalAltResult> { m.into() };
            if rq.borrow().len() >= MAX_REQUESTS {
                return Err(err(format!("a script may make at most {MAX_REQUESTS} requests")));
            }
            colour(&color)?;
            let w = num(&width).map_err(err)?;
            if !(w > 0.0 && w <= 1000.0) {
                return Err("line width must be within 0..1000".into());
            }
            let path = crate::path::VPath::smooth_through("line", &points(&pts).map_err(err)?, closed, 1.0).with_stroke(&color, w);
            rq.borrow_mut().push(Request::Layer(Box::new(crate::project::Layer::vector(&name, vec![path]))));
            Ok(())
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_gradient_rect", move |name: ImmutableString, x: Dynamic, y: Dynamic, w: Dynamic, h: Dynamic, top: ImmutableString, bottom: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let err = |m: String| -> Box<rhai::EvalAltResult> { m.into() };
            if rq.borrow().len() >= MAX_REQUESTS {
                return Err(err(format!("a script may make at most {MAX_REQUESTS} requests")));
            }
            colour(&top)?;
            colour(&bottom)?;
            let (x, y, w, h) = (num(&x).map_err(err)?, num(&y).map_err(err)?, num(&w).map_err(err)?, num(&h).map_err(err)?);
            let paint = crate::style::Paint::linear(crate::math::Vec2::new(x, y), crate::math::Vec2::new(x, y + h), &[(0.0, &top), (1.0, &bottom)]);
            let path = crate::path::VPath::rect("shape", x, y, w, h).with_paint(paint);
            rq.borrow_mut().push(Request::Layer(Box::new(crate::project::Layer::vector(&name, vec![path]))));
            Ok(())
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_ik", move |name: ImmutableString, bones: rhai::Array, target: ImmutableString, bend: Dynamic| -> Result<(), Box<rhai::EvalAltResult>> {
            let err = |m: String| -> Box<rhai::EvalAltResult> { m.into() };
            if rq.borrow().len() >= MAX_REQUESTS {
                return Err(err(format!("a script may make at most {MAX_REQUESTS} requests")));
            }
            let names: Vec<String> = bones.iter().map(|b| b.clone().into_string().map_err(|_| "bone names must be strings".to_string())).collect::<Result<_, _>>().map_err(err)?;
            let v = serde_json::json!({"name": name.as_str(), "bones": names, "target": target.as_str(), "bend": num(&bend).map_err(err)?});
            let ik: crate::project::IkChain = serde_json::from_value(v).map_err(|e| err(e.to_string()))?;
            rq.borrow_mut().push(Request::Ik(ik));
            Ok(())
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_aim", move |name: ImmutableString, bone: ImmutableString, target: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            if rq.borrow().len() >= MAX_REQUESTS {
                return Err(format!("a script may make at most {MAX_REQUESTS} requests").into());
            }
            rq.borrow_mut().push(Request::Constraint(crate::project::Constraint { name: name.to_string(), bone: bone.to_string(), kind: crate::project::ConstraintKind::Aim { target: target.to_string(), offset: 0.0 }, weight: 1.0 }));
            Ok(())
        });
    }

    // constraints with weight, the other constraint kinds
    {
        let rq = requests.clone();
        eng.register_fn("add_aim", move |name: ImmutableString, bone: ImmutableString, target: ImmutableString, offset: Dynamic, weight: Dynamic| -> Result<(), Box<rhai::EvalAltResult>> {
            let (offset, w) = (num(&offset)?, weight01(num(&weight)?)?);
            queue(&rq, Request::Constraint(crate::project::Constraint { name: name.to_string(), bone: bone.to_string(), kind: crate::project::ConstraintKind::Aim { target: target.to_string(), offset }, weight: w }))
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_copy_rotation", move |name: ImmutableString, bone: ImmutableString, source: ImmutableString, weight: Dynamic| -> Result<(), Box<rhai::EvalAltResult>> {
            let w = weight01(num(&weight)?)?;
            queue(&rq, Request::Constraint(crate::project::Constraint { name: name.to_string(), bone: bone.to_string(), kind: crate::project::ConstraintKind::CopyRotation { source: source.to_string() }, weight: w }))
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_pin", move |name: ImmutableString, bone: ImmutableString, target: ImmutableString, weight: Dynamic| -> Result<(), Box<rhai::EvalAltResult>> {
            let w = weight01(num(&weight)?)?;
            queue(&rq, Request::Constraint(crate::project::Constraint { name: name.to_string(), bone: bone.to_string(), kind: crate::project::ConstraintKind::Pin { target: target.to_string() }, weight: w }))
        });
    }

    // structure: groups, switches, moving, parenting, deleting
    {
        let rq = requests.clone();
        eng.register_fn("add_group", move |name: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let n = name.to_string();
            edit(&rq, format!("group '{n}'"), &["layers"], move |p| crate::scriptops::wrap_layers(p, &n, &[], crate::scriptops::Wrap::Group))
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_group", move |name: ImmutableString, members: rhai::Array| -> Result<(), Box<rhai::EvalAltResult>> {
            let (n, m) = (name.to_string(), names(&members)?);
            edit(&rq, format!("group '{n}'"), REF_SECTIONS, move |p| crate::scriptops::wrap_layers(p, &n, &m, crate::scriptops::Wrap::Group))
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_switch", move |name: ImmutableString, members: rhai::Array| -> Result<(), Box<rhai::EvalAltResult>> {
            let (n, m) = (name.to_string(), names(&members)?);
            edit(&rq, format!("switch '{n}'"), REF_SECTIONS, move |p| crate::scriptops::wrap_layers(p, &n, &m, crate::scriptops::Wrap::Switch))
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("move_layer", move |path: ImmutableString, parent: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let (a, b) = (path.to_string(), parent.to_string());
            edit(&rq, format!("move layer '{a}'"), REF_SECTIONS, move |p| crate::scriptops::move_layer(p, &a, &b))
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("delete_layer", move |path: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let a = path.to_string();
            edit(&rq, format!("delete layer '{a}'"), REF_SECTIONS, move |p| crate::scriptops::delete_layer(p, &a))
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("delete_bone", move |name: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let a = name.to_string();
            edit(&rq, format!("delete bone '{a}'"), &["bones", "channels", "channel_meta", "bone_vis"], move |p| crate::scriptops::delete_bone(p, &a))
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("set_parent", move |bone: ImmutableString, parent: ImmutableString| -> Result<(), Box<rhai::EvalAltResult>> {
            let (b, par) = (bone.to_string(), parent.to_string());
            edit(&rq, format!("parent of bone '{b}'"), &["bones"], move |p| crate::rigedit::set_parent(p, &b, if par.is_empty() { None } else { Some(par.as_str()) }))
        });
    }
    // static properties
    {
        let rq = requests.clone();
        eng.register_fn("set_layer", move |path: ImmutableString, prop: ImmutableString, value: Dynamic| -> Result<(), Box<rhai::EvalAltResult>> {
            let (a, pr, v) = (path.to_string(), prop.to_string(), val(&value)?);
            edit(&rq, format!("layer '{a}'.{pr}"), &["layers"], move |p| crate::scriptops::set_layer_prop(p, &a, &pr, &v))
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("set_bone", move |name: ImmutableString, prop: ImmutableString, value: Dynamic| -> Result<(), Box<rhai::EvalAltResult>> {
            let (a, pr, v) = (name.to_string(), prop.to_string(), val(&value)?);
            edit(&rq, format!("bone '{a}'.{pr}"), &["bones"], move |p| crate::scriptops::set_bone_prop(p, &a, &pr, &v))
        });
    }
    // procedural channels (appended; locked channels are refused)
    fn add_channel(rq: &Rq, ch: crate::project::Channel) -> Result<(), Box<rhai::EvalAltResult>> {
        let what = format!("channel {}.{:?}", ch.target, ch.prop);
        edit(rq, what, &["channels"], move |p| {
            crate::chanmeta::assert_unlocked(p, &ch.target, ch.prop)?;
            p.channels.push(ch.clone());
            Ok(())
        })
    }
    fn expr_channel(rq: &Rq, t: ImmutableString, prop: ImmutableString, expr: ImmutableString, mode: crate::project::Mode) -> Result<(), Box<rhai::EvalAltResult>> {
        let prop = Prop::parse(&prop).ok_or_else(|| format!("unknown property '{prop}'"))?;
        if expr.len() > 2000 {
            return Err("expression is longer than 2000 bytes".into());
        }
        add_channel(rq, crate::project::Channel::expr(&t, prop, &expr, mode))
    }
    fn noise_channel(rq: &Rq, t: ImmutableString, prop: ImmutableString, amp: Dynamic, freq: Dynamic, seed: Dynamic, mode: Option<crate::project::Mode>) -> Result<(), Box<rhai::EvalAltResult>> {
        let prop = Prop::parse(&prop).ok_or_else(|| format!("unknown property '{prop}'"))?;
        let mut ch = crate::project::Channel::noise(&t, prop, num(&amp)?, num(&freq)?, int_u32(&seed)?);
        if let Some(m) = mode {
            ch.mode = m;
        }
        add_channel(rq, ch)
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_channel_expr", move |t: ImmutableString, p: ImmutableString, e: ImmutableString| expr_channel(&rq, t, p, e, crate::project::Mode::Replace));
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_channel_expr", move |t: ImmutableString, p: ImmutableString, e: ImmutableString, m: ImmutableString| {
            let m = mode_of(&m)?;
            expr_channel(&rq, t, p, e, m)
        });
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_noise", move |t: ImmutableString, p: ImmutableString, a: Dynamic, f: Dynamic, s: Dynamic| noise_channel(&rq, t, p, a, f, s, None));
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_noise", move |t: ImmutableString, p: ImmutableString, a: Dynamic, f: Dynamic, s: Dynamic, m: ImmutableString| {
            let m = mode_of(&m)?;
            noise_channel(&rq, t, p, a, f, s, Some(m))
        });
    }
    // markers and events (metadata)
    let frames_f = frames as f32;
    fn marker(rq: &Rq, name: ImmutableString, start: Dynamic, end: Option<Dynamic>, max: f32) -> Result<(), Box<rhai::EvalAltResult>> {
        let (s, e) = (num(&start)?, end.map(|e| num(&e)).transpose()?);
        if name.is_empty() || name.len() > 200 {
            return Err("marker name must be 1..200 bytes".into());
        }
        if !(0.0..=max).contains(&s) || e.is_some_and(|e| !(s..=max).contains(&e)) {
            return Err(format!("marker frames must lie within 0..{max} (and end must not precede start)").into());
        }
        let n = name.to_string();
        edit(rq, format!("marker '{n}'"), &["markers"], move |p| {
            if p.markers.iter().any(|m| m.name == n) {
                return Err(format!("a marker named '{n}' already exists"));
            }
            p.markers.push(crate::keyops::Marker { name: n.clone(), frame: s, end: e });
            Ok(())
        })
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_marker", move |n: ImmutableString, f: Dynamic| marker(&rq, n, f, None, frames_f));
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_marker", move |n: ImmutableString, f: Dynamic, e: Dynamic| marker(&rq, n, f, Some(e), frames_f));
    }
    fn event(rq: &Rq, name: ImmutableString, frame: Dynamic, data: serde_json::Value, max: f32) -> Result<(), Box<rhai::EvalAltResult>> {
        let f = num(&frame)?;
        if name.is_empty() || name.len() > 200 {
            return Err("event name must be 1..200 bytes".into());
        }
        if !(0.0..=max).contains(&f) {
            return Err(format!("event frame must lie within 0..{max}").into());
        }
        let n = name.to_string();
        edit(rq, format!("event '{n}'"), &["events"], move |p| {
            p.events.push(crate::comps::Event { name: n.clone(), frame: f, data: data.clone() });
            Ok(())
        })
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_event", move |n: ImmutableString, f: Dynamic| event(&rq, n, f, serde_json::Value::Null, frames_f));
    }
    {
        let rq = requests.clone();
        eng.register_fn("add_event", move |n: ImmutableString, f: Dynamic, d: Dynamic| {
            let j = to_json(&d, 0)?;
            event(&rq, n, f, j, frames_f)
        });
    }

    // read helpers over the snapshot taken when the script started
    {
        let snap = snap.clone();
        eng.register_fn("bone_info", move |name: ImmutableString| -> Dynamic {
            let Some(b) = snap.bones.iter().find(|b| b.name == name.as_str()) else { return Dynamic::UNIT };
            let mut m = rhai::Map::new();
            m.insert("name".into(), b.name.clone().into());
            m.insert("parent".into(), b.parent.clone().unwrap_or_default().into());
            m.insert("length".into(), (b.length as f64).into());
            m.insert("x".into(), (b.rest_offset.x as f64).into());
            m.insert("y".into(), (b.rest_offset.y as f64).into());
            m.insert("angle".into(), (b.rest_angle as f64).into());
            m.into()
        });
    }
    {
        let snap = snap.clone();
        eng.register_fn("layer_info", move |path: ImmutableString| -> Dynamic {
            let Some(l) = crate::scriptops::find_layer(&snap.layers, &path) else { return Dynamic::UNIT };
            let mut m = rhai::Map::new();
            m.insert("name".into(), l.name.clone().into());
            m.insert("path".into(), path.to_string().into());
            let kind = serde_json::to_value(&l.kind).ok().and_then(|v| v.get("type").and_then(|t| t.as_str().map(String::from))).unwrap_or_else(|| "layer".into());
            m.insert("kind".into(), kind.into());
            m.insert("opacity".into(), (l.opacity as f64).into());
            m.insert("visible".into(), l.visible.into());
            m.insert("x".into(), (l.transform.position.x as f64).into());
            m.insert("y".into(), (l.transform.position.y as f64).into());
            m.insert("rotation".into(), (l.transform.rotation as f64).into());
            m.insert("scale_x".into(), (l.transform.scale.x as f64).into());
            m.insert("scale_y".into(), (l.transform.scale.y as f64).into());
            m.insert("skew".into(), (l.transform.skew as f64).into());
            m.insert("parallax".into(), (l.parallax as f64).into());
            m.insert("bone".into(), l.bone.clone().unwrap_or_default().into());
            let kids: rhai::Array = l.children().iter().map(|c| Dynamic::from(c.name.clone())).collect();
            m.insert("children".into(), kids.into());
            m.into()
        });
    }
    {
        let snap = snap.clone();
        let eng_cell: Rc<RefCell<Option<Result<crate::engine::Engine, String>>>> = Rc::default();
        eng.register_fn("world", move |bone: ImmutableString, frame: Dynamic| -> Result<Dynamic, Box<rhai::EvalAltResult>> {
            let f = num(&frame)?;
            if !(0.0..=snap.frames as f32).contains(&f) {
                return Err(format!("frame must lie within 0..{}", snap.frames).into());
            }
            let bi = snap.bones.iter().position(|b| b.name == bone.as_str()).ok_or_else(|| format!("no bone '{bone}'"))?;
            let mut cell = eng_cell.borrow_mut();
            let e = cell.get_or_insert_with(|| {
                // bones only: layers (and images) are not needed to pose the skeleton
                let mut q = (*snap).clone();
                q.layers.clear();
                q.channels.retain(|c| matches!(c.prop.target_kind(), crate::project::TargetKind::Bone | crate::project::TargetKind::Ik | crate::project::TargetKind::Constraint | crate::project::TargetKind::Camera));
                (q.skins, q.comps, q.events, q.markers, q.sounds, q.shots, q.bone_vis, q.channel_meta) = Default::default();
                crate::engine::Engine::new(q).map_err(|e| e.join("; "))
            });
            let e = e.as_ref().map_err(|m| format!("world() is unavailable: {m}"))?;
            let w = e.evaluate(f).world[bi];
            let head = w.apply(crate::math::Vec2::ZERO);
            let mut m = rhai::Map::new();
            m.insert("x".into(), (head.x as f64).into());
            m.insert("y".into(), (head.y as f64).into());
            m.insert("angle".into(), (w.rotation().to_degrees() as f64).into());
            m.insert("scale".into(), (w.x_scale() as f64).into());
            Ok(m.into())
        });
    }

    eng.run(src).map_err(|e| format!("script error: {e}"))?;

    let reqs = std::mem::take(&mut *requests.borrow_mut());
    let mut done = 0;
    for r in &reqs {
        let res = match r {
            Request::Key { target, prop, frame, value, ease } => editor.set_key(target, *prop, *frame, *value, *ease),
            Request::Bone(b) => serde_json::to_value(b).map_err(|e| vec![e.to_string()]).and_then(|v| editor.apply(crate::edit::Op::Insert { path: "/bones/-".into(), value: v })),
            Request::Layer(l) => serde_json::to_value(l).map_err(|e| vec![e.to_string()]).and_then(|v| editor.apply(crate::edit::Op::Insert { path: "/layers/-".into(), value: v })),
            Request::Ik(i) => serde_json::to_value(i).map_err(|e| vec![e.to_string()]).and_then(|v| editor.apply(crate::edit::Op::Insert { path: "/ik/-".into(), value: v })),
            Request::Constraint(c) => serde_json::to_value(c).map_err(|e| vec![e.to_string()]).and_then(|v| editor.apply(crate::edit::Op::Insert { path: "/constraints/-".into(), value: v })),
            Request::Edit { sections, f, .. } => apply_edit(editor, sections, f),
        };
        if let Err(errs) = res {
            for _ in 0..done {
                editor.undo();
            }
            let what = match r {
                Request::Key { target, prop, frame, .. } => format!("{target}.{prop:?} at {frame}"),
                Request::Bone(b) => format!("bone '{}'", b.name),
                Request::Layer(l) => format!("layer '{}'", l.name),
                Request::Ik(i) => format!("IK chain '{}'", i.name),
                Request::Constraint(c) => format!("constraint '{}'", c.name),
                Request::Edit { what, .. } => what.clone(),
            };
            return Err(format!("request {} ({what}) was rejected, nothing changed: {}", done + 1, errs.join("; ")));
        }
        done += 1;
    }
    let log = log.borrow().clone();
    Ok(ScriptReport { applied: done, log })
}

/// Run an edit on a copy of the project and store the changed sections as one undo step.
fn apply_edit(editor: &mut Editor, sections: &[&str], f: &dyn Fn(&mut crate::project::Project) -> Result<(), String>) -> Result<(), Vec<String>> {
    use crate::edit::Op;
    let mut np = editor.project().clone();
    f(&mut np).map_err(|e| vec![e])?;
    let new = serde_json::to_value(&np).map_err(|e| vec![e.to_string()])?;
    let old = editor.doc();
    let mut ops = Vec::new();
    for s in sections {
        let path = format!("/{s}");
        match (new.get(*s), old.get(*s)) {
            (Some(n), Some(o)) if n == o => {}
            (Some(n), _) => ops.push(Op::Set { path, value: n.clone() }),
            (None, Some(_)) => ops.push(Op::Remove { path }),
            (None, None) => {}
        }
    }
    if ops.is_empty() {
        // nothing changed: still one (harmless) step so that undo counts stay one per request
        let s = sections[0];
        ops.push(Op::Set { path: format!("/{s}"), value: new.get(s).cloned().unwrap_or(serde_json::Value::Array(vec![])) });
    }
    editor.apply(Op::Batch { ops })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;
    use crate::project::Source;

    fn ed() -> Editor {
        Editor::new(demo::project()).unwrap()
    }

    #[test]
    fn script_reads_project_and_sets_keys() {
        let mut e = ed();
        let src = r#"
            let n = 0;
            for b in bones() { if b == "arm_up" { n += 1; } }
            print(`fps ${fps()} frames ${frames()} arm bones ${n}`);
            for i in 0..4 { set_key("arm_up", "bone_angle", i * 10, 5.0 * i); }
            set_key("arm_up", "bone_angle", 45.5, 1, "ease_in_out");
        "#;
        let r = run_script(&mut e, src).unwrap();
        assert_eq!(r.applied, 5);
        assert_eq!(r.log, vec!["fps 24.0 frames 48 arm bones 1".to_string()]);
        let ch = e.project().channels.iter().find(|c| c.target == "arm_up" && c.prop == Prop::BoneAngle && matches!(c.source, Source::Keys { .. })).unwrap();
        let Source::Keys { track } = &ch.source else { unreachable!() };
        assert_eq!(track.eval(30.0), Some(15.0));
    }

    #[test]
    fn failures_roll_back_everything() {
        let mut e = ed();
        let before = e.project().clone();
        let err = run_script(&mut e, r#"set_key("arm_up", "bone_angle", 3, 9.0); set_key("ghost", "bone_angle", 3, 9.0);"#).unwrap_err();
        assert!(err.contains("nothing changed"), "{err}");
        assert_eq!(e.project(), &before);
        let err = run_script(&mut e, r#"set_key("arm_up", "wiggle", 3, 9.0);"#).unwrap_err();
        assert!(err.contains("unknown property"), "{err}");
        assert_eq!(e.project(), &before);
    }

    #[test]
    fn locks_are_respected() {
        let mut p = demo::project();
        crate::chanmeta::set_meta(&mut p, "arm_up", None, &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        let mut e = Editor::new(p).unwrap();
        let before = e.project().clone();
        assert!(run_script(&mut e, r#"set_key("arm_up", "bone_angle", 3, 9.0);"#).unwrap_err().contains("locked"));
        assert_eq!(e.project(), &before);
    }

    #[test]
    fn sandbox_limits() {
        let mut e = ed();
        let err = run_script(&mut e, "loop { }").unwrap_err();
        assert!(err.to_lowercase().contains("operations"), "{err}");
        assert!(run_script(&mut e, "fn f(n) { f(n + 1) } f(0)").is_err(), "runaway recursion stops");
        assert!(run_script(&mut e, r#"let s = "x"; for i in 0..20 { s += s; }"#).is_err(), "string growth is capped");
        assert!(run_script(&mut e, r#"for i in 0..6000 { set_key("arm_up", "bone_angle", i, 1.0); }"#).unwrap_err().contains("at most"));
        assert!(run_script(&mut e, "import \"x\" as y;").is_err(), "no modules");
        assert!(run_script(&mut e, "open_file(\"/etc/passwd\")").is_err(), "no file access exists to call");
        assert!(run_script(&mut e, &"x".repeat(70_000)).is_err());
    }

    #[test]
    fn scripts_can_create_bones_layers_and_key_them() {
        let mut e = ed();
        let (nb, nl) = (e.project().bones.len(), e.project().layers.len());
        let src = r##"
            add_bone("tail1", "", 20, 100, 100, 0);
            add_bone("tail2", "tail1", 15, 20, 0, 0);
            add_circle("ball", 50, 50, 8, "#ff0000");
            add_rect("box", 0, 0, 10, 10, "#00ff00");
            for i in 0..3 { set_key("tail2", "bone_angle", i * 12, 10.0 * i); }
        "##;
        let r = run_script(&mut e, src).unwrap();
        assert_eq!(r.applied, 7);
        assert_eq!((e.project().bones.len(), e.project().layers.len()), (nb + 2, nl + 2));
        assert!(e.engine().evaluate(12.0).world.len() == nb + 2);
        // everything undoes as separate steps, one per request
        for _ in 0..7 { assert!(e.undo()); }
        assert_eq!((e.project().bones.len(), e.project().layers.len()), (nb, nl));
    }

    #[test]
    fn invalid_creation_rolls_back_all() {
        let mut e = ed();
        let before = e.project().clone();
        let err = run_script(&mut e, r##"add_circle("ok", 1, 1, 3, "#fff"); add_bone("x", "no_such_parent", 5, 0, 0, 0);"##).unwrap_err();
        assert!(err.contains("nothing changed"), "{err}");
        assert_eq!(e.project(), &before);
        let dup = run_script(&mut e, r##"add_bone("arm_up", "", 5, 0, 0, 0);"##).unwrap_err();
        assert!(dup.contains("rejected"), "{dup}");
    }

    #[test]
    fn scripts_create_shapes_ik_and_aim() {
        let mut e = ed();
        let src = r##"
            add_bone("u", "", 20, 300, 300, 0);
            add_bone("l", "u", 20, 20, 0, 0);
            add_bone("tgt", "", 1, 320, 300, 0);
            add_ik("chain", ["u", "l"], "tgt", 1);
            add_aim("look", "tgt", "u");
            add_polygon("tri", [0, 0, 10, 0, 5, 9], "#abcdef");
            add_blob("blob", [0, 0, 20, 0, 20, 20, 0, 20], "#123456");
        "##;
        let r = run_script(&mut e, src).unwrap();
        assert_eq!(r.applied, 7);
        assert!(e.project().ik.iter().any(|i| i.name == "chain") && e.project().constraints.iter().any(|c| c.name == "look"));
        assert!(e.project().layers.iter().any(|l| l.name == "blob"));
        let bad = run_script(&mut e, r##"add_polygon("x", [0, 0, 1, 1], "#fff");"##).unwrap_err();
        assert!(bad.contains("coordinates"), "{bad}");
        let bad2 = run_script(&mut e, r##"add_ik("c2", ["u", "ghost"], "tgt", 1);"##).unwrap_err();
        assert!(bad2.contains("rejected"), "{bad2}");
    }

    #[test]
    fn strokes_gradients_and_bad_colours() {
        let mut e = ed();
        let r = run_script(&mut e, r##"
            add_line("wire", [0, 0, 30, 10, 60, 0], "#336699", 2.5, false);
            add_gradient_rect("sky2", 0, 0, 100, 50, "#88ccff", "white");
        "##).unwrap();
        assert_eq!(r.applied, 2);
        let wire = e.project().layers.iter().find(|l| l.name == "wire").unwrap();
        assert!(matches!(&wire.kind, crate::project::LayerKind::Vector { paths } if paths[0].stroke.is_some() && paths[0].fill.is_none()));
        // bad colours are script errors, never panics
        for bad in [r##"add_rect("a", 0, 0, 1, 1, "nope");"##, r##"add_circle("a", 0, 0, 1, "#12");"##, r##"add_line("a", [0,0,1,1,2,2], "ö", 1, false);"##, r##"add_gradient_rect("a", 0, 0, 1, 1, "#fff", "#zzzzzz");"##] {
            let err = run_script(&mut e, bad).unwrap_err();
            assert!(err.contains("bad colour"), "{err}");
        }
        assert!(run_script(&mut e, r##"add_line("a", [0,0,1,1,2,2], "#fff", 0, false);"##).is_err());
    }

    fn px(e: &Editor, f: f32) -> Vec<u8> {
        crate::raster::render_rgba(e.engine(), f, &crate::raster::RasterOptions::default()).2
    }

    #[test]
    fn groups_keep_draw_order_look_and_rewrite_references() {
        let mut e = ed();
        let (a, b) = (px(&e, 0.0), px(&e, 20.0));
        let r = run_script(&mut e, r#"add_group("body", ["leg", "shadow", "target", "arm"]);"#).unwrap();
        assert_eq!(r.applied, 1);
        let names: Vec<_> = e.project().layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(&names[3..6], &["ground", "body", "blush"], "{names:?}");
        let g = e.project().layers.iter().find(|l| l.name == "body").unwrap();
        let kids: Vec<_> = g.children().iter().map(|c| c.name.as_str()).collect();
        assert_eq!(kids, ["shadow", "arm", "leg", "target"], "members keep their own order");
        // channels that animated the moved layers now address the new paths
        assert!(e.project().channels.iter().any(|c| c.target == "body/shadow"));
        assert!(e.project().channels.iter().all(|c| c.target != "shadow"));
        assert_eq!(px(&e, 0.0), a);
        assert_eq!(px(&e, 20.0), b);
        // one undo step restores everything
        assert!(e.undo());
        assert!(e.project().layers.iter().all(|l| l.name != "body"));
        assert!(e.project().channels.iter().any(|c| c.target == "shadow"));
        assert_eq!(px(&e, 20.0), b);
    }

    #[test]
    fn group_failures_roll_back() {
        let mut e = ed();
        let before = e.project().clone();
        for bad in [
            r#"add_group("g", ["sky", "hills"]);"#,                  // not adjacent (sun lies between)
            r#"add_group("g", ["arm", "ghost"]);"#,                   // unknown layer
            r#"add_group("g", ["arm", "arm"]);"#,                     // listed twice
            r#"add_group("g", ["arm", "mouth/open"]);"#,              // different containers
            r#"add_group("a/b", ["arm"]);"#,                          // bad name
            r#"add_group("", ["arm"]);"#,
            r#"add_group("leg", ["arm"]);"#,                          // name clash with a sibling
            r#"add_group("g", ["sky", "sun"]);"#,                     // different parallax
            r#"add_switch("s", []);"#,                                // empty switch
            r#"add_group("ok", ["arm"]); add_group("ok", ["leg"]);"#, // second request fails -> first undone
        ] {
            let err = run_script(&mut e, bad).unwrap_err();
            assert!(err.contains("rejected") || err.contains("script error"), "{bad}: {err}");
            assert_eq!(e.project(), &before, "{bad}");
        }
        // a mask that points at a moved layer is rewritten; an expression that names it cannot be, so the edit is refused
        let mut p = demo::project();
        p.channels.push(crate::project::Channel::expr("arm_up", Prop::BoneOffsetX, "get(\"target\", \"layer_x\")", crate::project::Mode::Add));
        let mut e2 = Editor::new(p).unwrap();
        let b2 = e2.project().clone();
        assert!(run_script(&mut e2, r#"add_group("g", ["target"]);"#).unwrap_err().contains("rejected"));
        assert_eq!(e2.project(), &b2);
    }

    #[test]
    fn empty_groups_move_layer_and_switch() {
        let mut e = ed();
        run_script(&mut e, r#"add_group("props"); move_layer("shadow", "props"); move_layer("props/shadow", "");"#).unwrap();
        assert!(e.project().layers.iter().any(|l| l.name == "props"));
        assert!(e.project().layers.iter().any(|l| l.name == "shadow"), "moved back to the top level");
        assert!(e.project().channels.iter().any(|c| c.target == "shadow"));
        // into a group and back keeps channel targets consistent at every step
        run_script(&mut e, r#"move_layer("target", "props");"#).unwrap();
        assert!(e.project().channels.iter().any(|c| c.target == "props/target"));
        let before = e.project().clone();
        for bad in [r#"move_layer("props", "props");"#, r#"move_layer("props/target", "props/target");"#, r#"move_layer("arm", "sky");"#, r#"move_layer("ghost", "");"#, r#"move_layer("arm", "ghost");"#, r#"move_layer("arm", "props"); move_layer("props/arm", "props/arm");"#] {
            let r = run_script(&mut e, bad);
            assert!(r.is_err(), "{bad}");
            assert_eq!(e.project(), &before, "{bad}");
        }
        // a switch built from adjacent layers
        let mut e = ed();
        let r = run_script(&mut e, r#"add_switch("face", ["blush", "mouth"]); set_key("face", "layer_switch", 0, 1);"#).unwrap();
        assert_eq!(r.applied, 2);
        let sw = e.project().layers.iter().find(|l| l.name == "face").unwrap();
        assert!(matches!(&sw.kind, crate::project::LayerKind::Selector { layers, selected: 0 } if layers.len() == 2));
        assert!(e.undo() && e.undo());
        assert!(e.project().layers.iter().all(|l| l.name != "face"));
    }

    #[test]
    fn copy_rotation_and_pin_constraints_with_weight() {
        let mut e = ed();
        let r = run_script(&mut e, r#"
            add_copy_rotation("cr", "hand", "thigh", 0.5);
            add_pin("pin", "foot_ctrl", "arm_up", 1);
            add_aim("aim2", "hand", "arm_up", 15, 0.25);
        "#).unwrap();
        assert_eq!(r.applied, 3);
        use crate::project::ConstraintKind as K;
        let c = &e.project().constraints;
        assert!(matches!(&c[0].kind, K::CopyRotation { source } if source == "thigh") && c[0].weight == 0.5);
        assert!(matches!(&c[1].kind, K::Pin { target } if target == "arm_up"));
        assert!(matches!(&c[2].kind, K::Aim { offset, .. } if *offset == 15.0) && c[2].weight == 0.25);
        let before = e.project().clone();
        for bad in [r#"add_pin("p", "hand", "hand", 1);"#, r#"add_pin("p", "hand", "ghost", 1);"#, r#"add_copy_rotation("c", "ghost", "hand", 1);"#, r#"add_pin("p", "hand", "arm_up", 1.5);"#, r#"add_copy_rotation("c", "hand", "arm_up", -0.1);"#, r#"add_aim("a", "hand", "arm_up", 0, 2);"#] {
            assert!(run_script(&mut e, bad).is_err(), "{bad}");
            assert_eq!(e.project(), &before);
        }
        for _ in 0..3 { assert!(e.undo()); }
        assert!(e.project().constraints.is_empty());
    }

    #[test]
    fn static_properties_parent_delete() {
        let mut e = ed();
        run_script(&mut e, r##"
            set_layer("ink", "opacity", 0.5);
            set_layer("ink", "visible", false);
            set_layer("ink", "x", 12);
            set_layer("ink", "scale_x", 2.0);
            set_layer("mouth/open", "rotation", 7);
            set_layer("sky", "parallax", 0.3);
            set_layer("blush", "bone", "arm_up");
            set_bone("arm_up", "length", 33);
            set_bone("arm_up", "angle_min", -45);
            set_bone("arm_up", "angle_max", 90);
        "##).unwrap();
        let p = e.project();
        let ink = p.layers.iter().find(|l| l.name == "ink").unwrap();
        assert_eq!((ink.opacity, ink.visible, ink.transform.position.x, ink.transform.scale.x), (0.5, false, 12.0, 2.0));
        assert_eq!(p.bones[0].length, 33.0);
        assert_eq!(p.bones[0].angle_limits, Some([-45.0, 90.0]));
        assert_eq!(p.layers.iter().find(|l| l.name == "blush").unwrap().bone.as_deref(), Some("arm_up"));
        let before = e.project().clone();
        for bad in [
            r#"set_layer("ink", "opacity", 2);"#, r#"set_layer("ink", "opacity", "x");"#, r#"set_layer("ink", "visible", 1);"#,
            r#"set_layer("ink", "wat", 1);"#, r#"set_layer("ghost", "x", 1);"#, r#"set_layer("mouth/open", "parallax", 1);"#,
            r#"set_layer("blush", "bone", "ghost");"#, r#"set_bone("ghost", "length", 1);"#, r#"set_bone("hand", "length", -1);"#,
            r#"set_bone("hand", "wat", 1);"#, r#"set_bone("arm_up", "angle_min", 120);"#, r#"set_layer("ink", "x", 1.0 / 0.0);"#,
            r#"set_layer("ink", "x", 1); set_layer("ink", "opacity", 9);"#,
        ] {
            let r = run_script(&mut e, bad);
            assert!(r.is_err(), "{bad}");
            assert_eq!(e.project(), &before, "{bad}");
        }
        // set_parent keeps the rest pose and reorders; cycles and unknown bones are errors
        let mut e = ed();
        let w0 = e.engine().evaluate(0.0).world.clone();
        run_script(&mut e, r#"set_parent("foot_ctrl", "arm_up"); set_parent("foot_ctrl", "");"#).unwrap();
        assert!(e.project().bones.iter().find(|b| b.name == "foot_ctrl").unwrap().parent.is_none());
        let w1 = e.engine().evaluate(0.0).world.clone();
        assert_eq!(w0.len(), w1.len());
        let before = e.project().clone();
        assert!(run_script(&mut e, r#"set_parent("arm_up", "hand");"#).is_err());
        assert!(run_script(&mut e, r#"set_parent("arm_up", "ghost");"#).is_err());
        assert_eq!(e.project(), &before);

        // delete: layer takes its channels with it, bone is refused while in use
        let mut e = ed();
        let nch = e.project().channels.len();
        run_script(&mut e, r#"delete_layer("ink"); delete_layer("mouth/smile");"#).unwrap();
        assert!(e.project().layers.iter().all(|l| l.name != "ink"));
        assert!(e.project().channels.len() < nch);
        assert!(e.undo() && e.undo());
        assert_eq!(e.project().channels.len(), nch);
        assert!(e.project().layers.iter().any(|l| l.name == "ink"));
        let before = e.project().clone();
        for bad in [r#"delete_layer("ghost");"#, r#"delete_bone("arm_up");"#, r#"delete_bone("hand");"#, r#"delete_bone("foot_ctrl");"#, r#"delete_bone("thigh");"#, r#"delete_bone("ghost");"#] {
            assert!(run_script(&mut e, bad).is_err(), "{bad}");
            assert_eq!(e.project(), &before, "{bad}");
        }
        run_script(&mut e, r#"add_bone("tmp", "hand", 5, 1, 1, 0); set_key("tmp", "bone_angle", 0, 3); delete_bone("tmp");"#).unwrap();
        assert!(e.project().bones.iter().all(|b| b.name != "tmp") && e.project().channels.iter().all(|c| c.target != "tmp"));
    }

    #[test]
    fn procedural_channels_markers_events() {
        let mut e = ed();
        let r = run_script(&mut e, r#"
            add_channel_expr("arm_up", "bone_offset_y", "5 * sin(pi * t)");
            add_channel_expr("arm_up", "bone_offset_x", "2", "add");
            add_noise("hand", "bone_angle", 3, 1.5, 7);
            add_noise("hand", "bone_offset_x", 1, 1, 0, "multiply");
            add_marker("intro", 0); add_marker("act", 10, 20);
            add_event("step", 12); add_event("sfx", 14, "boom.wav");
            add_event("cue", 16, #{ sound: "x.wav", loud: true, n: 3, v: [1, 2.5] });
        "#).unwrap();
        assert_eq!(r.applied, 9);
        let p = e.project();
        assert!(p.channels.iter().any(|c| matches!(&c.source, crate::project::Source::Expr { expr } if expr.starts_with("5 *"))));
        assert!(p.channels.iter().any(|c| matches!(c.source, crate::project::Source::Noise { seed: 7, .. }) && c.mode == crate::project::Mode::Add));
        assert!(p.channels.iter().any(|c| matches!(c.source, crate::project::Source::Noise { .. }) && c.mode == crate::project::Mode::Multiply));
        assert_eq!((p.markers.len(), p.events.len()), (2, 3));
        assert_eq!(p.events[1].data, serde_json::json!("boom.wav"));
        assert_eq!(p.events[2].data["sound"], "x.wav");
        assert_eq!(p.markers[1].end, Some(20.0));
        let _ = e.engine().evaluate(10.0);
        for _ in 0..9 { assert!(e.undo()); }
        assert!(e.project().markers.is_empty() && e.project().events.is_empty());
        assert!(e.redo());

        let mut e = ed();
        let before = e.project().clone();
        for bad in [
            r#"add_channel_expr("arm_up", "bone_angle", "1 +");"#, r#"add_channel_expr("ghost", "bone_angle", "1");"#, r#"add_channel_expr("arm_up", "nope", "1");"#,
            r#"add_channel_expr("arm_up", "bone_angle", "1", "mix");"#, r#"add_noise("arm_up", "bone_angle", 1, 1, -1);"#, r#"add_noise("arm_up", "bone_angle", 1, 1, 1.5);"#,
            r#"add_noise("ghost", "bone_angle", 1, 1, 1);"#, r#"add_noise("arm_up", "bone_angle", "a", 1, 1);"#,
            r#"add_marker("", 1);"#, r#"add_marker("m", -1);"#, r#"add_marker("m", 5, 2);"#, r#"add_marker("m", 1, 9999);"#, r#"add_marker("m", 1); add_marker("m", 2);"#,
            r#"add_event("", 1);"#, r#"add_event("e", 9999);"#, r#"add_event("e", 1, #{ a: |x| x });"#, r#"add_event("e", 1, ok_ok);"#,
        ] {
            assert!(run_script(&mut e, bad).is_err(), "{bad}");
            assert_eq!(e.project(), &before, "{bad}");
        }
        // channel locks apply to appended channels as well
        let mut p = demo::project();
        crate::chanmeta::set_meta(&mut p, "arm_up", None, &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        let mut e = Editor::new(p).unwrap();
        assert!(run_script(&mut e, r#"add_noise("arm_up", "bone_angle", 1, 1, 1);"#).unwrap_err().contains("locked"));
    }

    #[test]
    fn read_helpers_report_the_snapshot() {
        let mut e = ed();
        let src = r#"
            let b = bone_info("arm_low");
            print(`${b.name} parent ${b.parent} length ${b.length}`);
            print(`ghost ${bone_info("ghost") == ()}`);
            let l = layer_info("mouth");
            print(`${l.kind} ${l.children.len()} bone ${l.bone} op ${l.opacity} vis ${l.visible}`);
            print(`${layer_info("mouth/open").path} ${layer_info("nope/open") == ()}`);
            let w0 = world("hand", 0);
            let w1 = world("hand", 24);
            print(`moved ${w0.x != w1.x || w0.y != w1.y || w0.angle != w1.angle}`);
            add_bone("late", "", 1, 0, 0, 0);
            print(`late ${bone_info("late") == ()}`);
        "#;
        let r = run_script(&mut e, src).unwrap();
        assert!(r.log[0].starts_with("arm_low parent arm_up length "), "{:?}", r.log);
        assert_eq!(r.log[1], "ghost true");
        assert!(r.log[2].starts_with("switch 3 bone hand"), "{:?}", r.log);
        assert_eq!(r.log[3], "mouth/open true");
        assert_eq!(r.log[4], "moved true");
        assert_eq!(r.log[5], "late true", "helpers read the project as it was when the script started");
        // world() agrees with the engine
        let w = e.engine().evaluate(10.0).world[2];
        let r = run_script(&mut e, r#"let w = world("hand", 10); print(`${w.x} ${w.y}`);"#).unwrap();
        let h = w.apply(crate::math::Vec2::ZERO);
        assert_eq!(r.log[0], format!("{} {}", h.x as f64, h.y as f64));
        for bad in [r#"world("ghost", 1);"#, r#"world("hand", -1);"#, r#"world("hand", 99999);"#, r#"world("hand", "x");"#] {
            assert!(run_script(&mut e, bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn masks_follow_moved_layers_and_new_requests_are_capped() {
        let mut p = demo::project();
        p.layers.iter_mut().find(|l| l.name == "arm").unwrap().mask = Some(crate::project::MaskRef { layer: "leg".into(), invert: false });
        let mut e = Editor::new(p).unwrap();
        run_script(&mut e, r#"add_group("g", ["leg"]);"#).unwrap();
        let arm = e.project().layers.iter().find(|l| l.name == "arm").unwrap();
        assert_eq!(arm.mask.as_ref().unwrap().layer, "g/leg");
        // deleting a masking source is refused (dangling reference) and nothing changes
        let before = e.project().clone();
        assert!(run_script(&mut e, r#"delete_layer("g/leg");"#).unwrap_err().contains("rejected"));
        assert_eq!(e.project(), &before);
        // the request cap covers the new functions too
        let err = run_script(&mut e, r#"for i in 0..6000 { add_marker("m", 1); }"#).unwrap_err();
        assert!(err.contains("at most"), "{err}");
        assert_eq!(e.project(), &before);
        // odd inputs are errors, not panics
        for bad in [r#"set_layer("", "x", 1);"#, r#"set_layer("a//b", "x", 1);"#, r#"move_layer("", "");"#, r#"add_group("g", [1, 2]);"#, r#"add_group("g", [""]);"#, r#"layer_info(5);"#, r#"world("", 0);"#, r#"add_event("e", 1, #{a: #{b: #{c: #{d: #{e: #{f: 1}}}}}});"#] {
            assert!(run_script(&mut e, bad).is_err(), "{bad}");
            assert_eq!(e.project(), &before, "{bad}");
        }
    }
}
