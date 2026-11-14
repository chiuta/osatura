// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! C ABI for OSATURA. See `osatura.h` for the contract.
//!
//! Every entry point catches panics and turns them into error codes, validates pointers and
//! sizes, and never lets a Rust type cross the boundary except behind the opaque handle.
//! It is written against the stable `osatura_core::prelude` facade only.

use osatura_core::prelude::*;
use std::cell::RefCell;
use std::ffi::{c_char, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

pub const ABI_VERSION: u32 = 1;

pub struct OsaturaEngine {
    anim: Animation,
    info: AnimationInfo,
}

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::default());
}

fn set_error(msg: impl Into<String>) {
    let s = msg.into().replace('\0', " ");
    LAST_ERROR.with(|e| *e.borrow_mut() = CString::new(s).unwrap_or_default());
}

fn guarded<T>(fail: T, f: impl FnOnce() -> Result<T, String>) -> T {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => v,
        Ok(Err(m)) => {
            set_error(m);
            fail
        }
        Err(_) => {
            set_error("internal error (panic) inside osatura");
            fail
        }
    }
}

static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();

#[no_mangle]
pub extern "C" fn osatura_version() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn osatura_abi_version() -> u32 {
    ABI_VERSION
}

/// # Safety
/// `json` must point to `len` readable bytes (or be null with `len == 0`, which is an error).
#[no_mangle]
pub unsafe extern "C" fn osatura_open(json: *const c_char, len: usize) -> *mut OsaturaEngine {
    guarded(std::ptr::null_mut(), || {
        if json.is_null() || len == 0 || len > 256 * 1024 * 1024 {
            return Err("osatura_open: null pointer or bad length".into());
        }
        let bytes = std::slice::from_raw_parts(json as *const u8, len);
        let text = std::str::from_utf8(bytes).map_err(|e| format!("project is not UTF-8: {e}"))?;
        let anim = Animation::from_json(text).map_err(|e| e.to_string())?;
        let info = anim.info();
        Ok(Box::into_raw(Box::new(OsaturaEngine { anim, info })))
    })
}

/// # Safety
/// `engine` must be null or a pointer from `osatura_open` not yet closed.
#[no_mangle]
pub unsafe extern "C" fn osatura_close(engine: *mut OsaturaEngine) {
    if !engine.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(engine))));
    }
}

#[no_mangle]
pub extern "C" fn osatura_last_error() -> *const c_char {
    LAST_ERROR.with(|e| e.borrow().as_ptr())
}

fn eng<'a>(e: *const OsaturaEngine) -> Option<&'a OsaturaEngine> {
    // SAFETY: documented contract of the header: the pointer came from osatura_open and is live.
    unsafe { e.as_ref() }
}

macro_rules! getter {
    ($name:ident, $ty:ty, $zero:expr, |$p:ident| $body:expr) => {
        /// # Safety
        /// `engine` must be null or a live handle.
        #[no_mangle]
        pub unsafe extern "C" fn $name(engine: *const OsaturaEngine) -> $ty {
            match eng(engine) {
                Some(e) => {
                    let $p = &e.info;
                    $body
                }
                None => {
                    set_error(concat!(stringify!($name), ": null engine"));
                    $zero
                }
            }
        }
    };
}

getter!(osatura_frame_count, u32, 0, |p| p.frames);
getter!(osatura_fps, f32, 0.0, |p| p.fps);
getter!(osatura_width, u32, 0, |p| p.width.round() as u32);
getter!(osatura_height, u32, 0, |p| p.height.round() as u32);

/// # Safety
/// `engine` must be null or a live handle.
#[no_mangle]
pub unsafe extern "C" fn osatura_bone_count(engine: *const OsaturaEngine) -> u32 {
    match eng(engine) {
        Some(e) => e.anim.bone_names().len() as u32,
        None => {
            set_error("osatura_bone_count: null engine");
            0
        }
    }
}

/// # Safety
/// `engine` live or null; `out` must have `cap` writable bytes (or be null with `cap == 0`).
#[no_mangle]
pub unsafe extern "C" fn osatura_bone_name(engine: *const OsaturaEngine, index: u32, out: *mut c_char, cap: usize) -> i32 {
    guarded(-1, || {
        let e = eng(engine).ok_or("null engine")?;
        let names = e.anim.bone_names();
        let name = names.get(index as usize).ok_or("bone index out of range")?;
        if !out.is_null() && cap > 0 {
            let n = name.len().min(cap - 1);
            std::ptr::copy_nonoverlapping(name.as_ptr(), out as *mut u8, n);
            *out.add(n) = 0;
        }
        Ok(name.len() as i32)
    })
}

/// # Safety
/// `engine` live; `out` must have `cap_floats` writable floats.
#[no_mangle]
pub unsafe extern "C" fn osatura_bone_world(engine: *const OsaturaEngine, frame: f32, out: *mut f32, cap_floats: usize) -> i32 {
    guarded(-1, || {
        let e = eng(engine).ok_or("null engine")?;
        let n = e.anim.bone_names().len();
        if out.is_null() || cap_floats < n * 6 {
            return Err(format!("output buffer needs {} floats", n * 6));
        }
        let world = e.anim.bone_world(frame).map_err(|e| e.to_string())?;
        let dst = std::slice::from_raw_parts_mut(out, n * 6);
        for (i, m) in world.iter().enumerate() {
            dst[i * 6..i * 6 + 6].copy_from_slice(m);
        }
        Ok(0)
    })
}

/// # Safety
/// `engine` live; `out` has `cap` writable bytes; `w`/`h` writable or null.
#[no_mangle]
pub unsafe extern "C" fn osatura_render_rgba(engine: *const OsaturaEngine, frame: f32, scale: f32, out: *mut u8, cap: usize, w: *mut u32, h: *mut u32) -> i32 {
    guarded(-1, || {
        let e = eng(engine).ok_or("null engine")?;
        if !frame.is_finite() || !scale.is_finite() || !(0.05..=4.0).contains(&scale) {
            return Err("frame must be finite and scale within 0.05..4".into());
        }
        let (pw, ph) = ((e.info.width * scale).round().max(1.0) as u32, (e.info.height * scale).round().max(1.0) as u32);
        if !w.is_null() {
            *w = pw;
        }
        if !h.is_null() {
            *h = ph;
        }
        let need = pw as usize * ph as usize * 4;
        if out.is_null() || cap < need {
            set_error(format!("output buffer needs {need} bytes"));
            return Ok(-2);
        }
        let img = e.anim.render_rgba(frame, &RenderSettings::new(scale)).map_err(|e| e.to_string())?;
        let px = &img.pixels;
        if px.len() > cap {
            set_error(format!("output buffer needs {} bytes", px.len()));
            if !w.is_null() { *w = img.width; }
            if !h.is_null() { *h = img.height; }
            return Ok(-2);
        }
        if !w.is_null() { *w = img.width; }
        if !h.is_null() { *h = img.height; }
        std::ptr::copy_nonoverlapping(px.as_ptr(), out, px.len());
        Ok(0)
    })
}

// ---------------------------------------------------------------------------------------------
// Interactive playback: state machines (additive, ABI version unchanged)
// ---------------------------------------------------------------------------------------------

/// A running state machine. Independent of the engine handle it was created from.
pub struct OsaturaPlayer {
    player: Player,
}

unsafe fn cstr<'a>(p: *const c_char) -> Result<&'a str, String> {
    if p.is_null() {
        return Err("null string".into());
    }
    std::ffi::CStr::from_ptr(p).to_str().map_err(|_| "string is not UTF-8".to_string())
}

unsafe fn put_str(text: &str, out: *mut c_char, cap: usize) -> i32 {
    if !out.is_null() && cap > 0 {
        let n = text.len().min(cap - 1);
        std::ptr::copy_nonoverlapping(text.as_ptr(), out as *mut u8, n);
        *out.add(n) = 0;
    }
    text.len() as i32
}

/// # Safety
/// `engine` must be null or a live handle.
#[no_mangle]
pub unsafe extern "C" fn osatura_machine_count(engine: *const OsaturaEngine) -> u32 {
    match eng(engine) {
        Some(e) => e.anim.machine_names().len() as u32,
        None => {
            set_error("osatura_machine_count: null engine");
            0
        }
    }
}

/// # Safety
/// `engine` live; `out` has `cap` writable bytes (or is null).
#[no_mangle]
pub unsafe extern "C" fn osatura_machine_name(engine: *const OsaturaEngine, index: u32, out: *mut c_char, cap: usize) -> i32 {
    guarded(-1, || {
        let e = eng(engine).ok_or("null engine")?;
        let names = e.anim.machine_names();
        let name = names.get(index as usize).ok_or("machine index out of range")?;
        Ok(put_str(name, out, cap))
    })
}

/// Start a state machine of the project. Returns NULL on error.
/// # Safety
/// `engine` live; `machine` a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_open(engine: *const OsaturaEngine, machine: *const c_char) -> *mut OsaturaPlayer {
    guarded(std::ptr::null_mut(), || {
        let e = eng(engine).ok_or("null engine")?;
        let player = e.anim.player(cstr(machine)?).map_err(|e| e.to_string())?;
        Ok(Box::into_raw(Box::new(OsaturaPlayer { player })))
    })
}

/// # Safety
/// `p` must be null or a pointer from `osatura_player_open` not yet closed.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_close(p: *mut OsaturaPlayer) {
    if !p.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(p))));
    }
}

unsafe fn plr<'a>(p: *mut OsaturaPlayer) -> Result<&'a mut OsaturaPlayer, String> {
    p.as_mut().ok_or_else(|| "null player".to_string())
}

/// Set a bool input. Returns 0 or a negative value (see `osatura_last_error`).
/// # Safety
/// `p` live; `name` NUL-terminated UTF-8.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_set_bool(p: *mut OsaturaPlayer, name: *const c_char, value: i32) -> i32 {
    guarded(-1, || plr(p)?.player.set_bool(cstr(name)?, value != 0).map(|_| 0).map_err(|e| e.to_string()))
}

/// Set a number input.
/// # Safety
/// `p` live; `name` NUL-terminated UTF-8.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_set_number(p: *mut OsaturaPlayer, name: *const c_char, value: f64) -> i32 {
    guarded(-1, || plr(p)?.player.set_number(cstr(name)?, value).map(|_| 0).map_err(|e| e.to_string()))
}

/// Fire a trigger input.
/// # Safety
/// `p` live; `name` NUL-terminated UTF-8.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_fire(p: *mut OsaturaPlayer, name: *const c_char) -> i32 {
    guarded(-1, || plr(p)?.player.fire(cstr(name)?).map(|_| 0).map_err(|e| e.to_string()))
}

/// Publish an external value for the project's data bindings (for example `player.health`).
/// # Safety
/// `p` live; `path` NUL-terminated UTF-8.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_set_value(p: *mut OsaturaPlayer, path: *const c_char, value: f64) -> i32 {
    guarded(-1, || plr(p)?.player.set_value(cstr(path)?, value).map(|_| 0).map_err(|e| e.to_string()))
}

/// Advance time by `dt` seconds (0..10).
/// # Safety
/// `p` live.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_advance(p: *mut OsaturaPlayer, dt: f64) -> i32 {
    guarded(-1, || plr(p)?.player.advance(dt).map(|_| 0).map_err(|e| e.to_string()))
}

/// Copy the current state name of `layer` into `out`; returns its length or -1 for an unknown layer.
/// # Safety
/// `p` live; `layer` NUL-terminated UTF-8; `out` has `cap` writable bytes (or is null).
#[no_mangle]
pub unsafe extern "C" fn osatura_player_state(p: *mut OsaturaPlayer, layer: *const c_char, out: *mut c_char, cap: usize) -> i32 {
    guarded(-1, || {
        let s = plr(p)?.player.state(cstr(layer)?).ok_or("unknown layer")?;
        Ok(put_str(&s, out, cap))
    })
}

/// Copy the events since the last call (JSON array of {time, layer, kind, state}) into `out`.
/// Returns the full length; if it exceeds `cap - 1` the text is truncated and the events are
/// KEPT (call again with a larger buffer); otherwise they are consumed.
/// # Safety
/// `p` live; `out` has `cap` writable bytes (or is null to query the size).
#[no_mangle]
pub unsafe extern "C" fn osatura_player_events(p: *mut OsaturaPlayer, out: *mut c_char, cap: usize) -> i32 {
    guarded(-1, || {
        let pl = plr(p)?;
        let json = pl.player.events_json();
        if !out.is_null() && cap > json.len() {
            pl.player.clear_events();
        }
        Ok(put_str(&json, out, cap))
    })
}

/// Render the current pose at `scale` (0.05..4) into straight-alpha RGBA8. Same contract as
/// `osatura_render_rgba` (returns -2 and sets *w / *h when `cap` is too small).
/// # Safety
/// `p` live; `out` has `cap` writable bytes; `w`/`h` writable or null.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_render_rgba(p: *mut OsaturaPlayer, scale: f32, out: *mut u8, cap: usize, w: *mut u32, h: *mut u32) -> i32 {
    guarded(-1, || {
        let pl = plr(p)?;
        if !scale.is_finite() || !(0.05..=4.0).contains(&scale) {
            return Err("scale must be within 0.05..4".into());
        }
        let (cw, ch) = pl.player.canvas_size();
        let (pw, ph) = ((cw * scale).round().max(1.0) as u32, (ch * scale).round().max(1.0) as u32);
        if !w.is_null() {
            *w = pw;
        }
        if !h.is_null() {
            *h = ph;
        }
        let need = pw as usize * ph as usize * 4;
        if out.is_null() || cap < need {
            set_error(format!("output buffer needs {need} bytes"));
            return Ok(-2);
        }
        let img = pl.player.render_rgba(&RenderSettings::new(scale)).map_err(|e| e.to_string())?;
        if img.pixels.len() > cap {
            set_error(format!("output buffer needs {} bytes", img.pixels.len()));
            return Ok(-2);
        }
        std::ptr::copy_nonoverlapping(img.pixels.as_ptr(), out, img.pixels.len());
        Ok(0)
    })
}

// ---------------------------------------------------------------------------------------------
// Layout and scrolling, level of detail (additive; ABI version unchanged)
// ---------------------------------------------------------------------------------------------

unsafe fn put_pair(a: *mut f32, b: *mut f32, v: [f32; 2]) {
    if !a.is_null() {
        *a = v[0];
    }
    if !b.is_null() {
        *b = v[1];
    }
}

/// Lay the project out for another canvas size (stage units): containers re-solve, scroll limits
/// follow, and renders get that size. Returns 0 or a negative value.
/// # Safety
/// `p` live.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_set_canvas(p: *mut OsaturaPlayer, width: f32, height: f32) -> i32 {
    guarded(-1, || plr(p)?.player.set_canvas_size(width, height).map(|_| 0).map_err(|e| e.to_string()))
}

/// Current canvas size in stage units (either pointer may be NULL).
/// # Safety
/// `p` live; `w`/`h` writable or NULL.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_canvas_size(p: *mut OsaturaPlayer, w: *mut f32, h: *mut f32) -> i32 {
    guarded(-1, || {
        let (cw, ch) = plr(p)?.player.canvas_size();
        put_pair(w, h, [cw, ch]);
        Ok(0)
    })
}

/// Number of scroll regions of the project (0 for a NULL player).
/// # Safety
/// `p` live or NULL.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_scroll_count(p: *mut OsaturaPlayer) -> u32 {
    match p.as_ref() {
        Some(pl) => pl.player.scroll_ids().len() as u32,
        None => {
            set_error("osatura_player_scroll_count: null player");
            0
        }
    }
}

/// Id of scroll region `index` (same copy rules as osatura_bone_name).
/// # Safety
/// `p` live; `out` has `cap` writable bytes (or is NULL).
#[no_mangle]
pub unsafe extern "C" fn osatura_player_scroll_id(p: *mut OsaturaPlayer, index: u32, out: *mut c_char, cap: usize) -> i32 {
    guarded(-1, || {
        let ids = plr(p)?.player.scroll_ids();
        let id = ids.get(index as usize).ok_or("scroll index out of range")?;
        Ok(put_str(id, out, cap))
    })
}

/// Pointer drag: the raw offset in pixels the user asks for (content moved up / left by that much).
/// # Safety
/// `p` live; `id` NUL-terminated UTF-8.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_scroll_drag(p: *mut OsaturaPlayer, id: *const c_char, x: f32, y: f32) -> i32 {
    guarded(-1, || plr(p)?.player.scroll_drag(cstr(id)?, x, y).map(|_| 0).map_err(|e| e.to_string()))
}

/// Let go with a release velocity in pixels per second; osatura_player_advance then flings and settles.
/// # Safety
/// `p` live; `id` NUL-terminated UTF-8.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_scroll_release(p: *mut OsaturaPlayer, id: *const c_char, vx: f32, vy: f32) -> i32 {
    guarded(-1, || plr(p)?.player.scroll_release(cstr(id)?, vx, vy).map(|_| 0).map_err(|e| e.to_string()))
}

/// Displayed offset of a scroll region (either pointer may be NULL). -1 for an unknown id.
/// # Safety
/// `p` live; `id` NUL-terminated UTF-8; `x`/`y` writable or NULL.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_scroll_offset(p: *mut OsaturaPlayer, id: *const c_char, x: *mut f32, y: *mut f32) -> i32 {
    guarded(-1, || {
        let v = plr(p)?.player.scroll_offset(cstr(id)?).ok_or("unknown scroll")?;
        put_pair(x, y, v);
        Ok(0)
    })
}

/// Largest useful offset of a scroll region at the current canvas size. -1 for an unknown id.
/// # Safety
/// as osatura_player_scroll_offset.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_scroll_max(p: *mut OsaturaPlayer, id: *const c_char, x: *mut f32, y: *mut f32) -> i32 {
    guarded(-1, || {
        let v = plr(p)?.player.scroll_max(cstr(id)?).ok_or("unknown scroll")?;
        put_pair(x, y, v);
        Ok(0)
    })
}

/// 1 when the region is at rest, 0 while it flings or springs back, -1 for an unknown id.
/// # Safety
/// `p` live; `id` NUL-terminated UTF-8.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_scroll_settled(p: *mut OsaturaPlayer, id: *const c_char) -> i32 {
    guarded(-1, || Ok(plr(p)?.player.scroll_settled(cstr(id)?).ok_or("unknown scroll")? as i32))
}

/// Tell the player how large it is shown (pixels) so that it picks the coarsest level of detail whose
/// declared error bound stays within `max_error_px` pixels. Levels are built on first use.
/// # Safety
/// `p` live.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_set_screen(p: *mut OsaturaPlayer, width_px: f32, height_px: f32, max_error_px: f32) -> i32 {
    guarded(-1, || plr(p)?.player.set_screen_size(width_px, height_px, max_error_px).map(|_| 0).map_err(|e| e.to_string()))
}

/// Level of detail in use (0 = original, also without a screen size). Negative for a NULL player.
/// # Safety
/// `p` live.
#[no_mangle]
pub unsafe extern "C" fn osatura_player_lod_level(p: *mut OsaturaPlayer) -> i32 {
    guarded(-1, || Ok(plr(p)?.player.lod_level() as i32))
}

// ---------------------------------------------------------------------------------------------
// Editing and export to memory (additive; ABI version unchanged)
// ---------------------------------------------------------------------------------------------

fn eng_mut<'a>(e: *mut OsaturaEngine) -> Result<&'a mut OsaturaEngine, String> {
    // SAFETY: documented contract of the header: the pointer came from osatura_open and is live,
    // and a handle is used from one thread at a time.
    unsafe { e.as_mut() }.ok_or_else(|| "null engine".to_string())
}

fn opt_str<'a>(p: *const c_char) -> Result<Option<&'a str>, String> {
    if p.is_null() {
        Ok(None)
    } else {
        // SAFETY: caller contract: a non-null pointer is a NUL-terminated string.
        unsafe { cstr(p) }.map(Some)
    }
}

/// Run an edit and refresh the cached facts (an edit may in principle change them).
fn edit(engine: *mut OsaturaEngine, f: impl FnOnce(&mut Animation) -> Result<(), ApiError>) -> Result<i32, String> {
    let e = eng_mut(engine)?;
    f(&mut e.anim).map_err(|e| e.to_string())?;
    e.info = e.anim.info();
    Ok(0)
}

/// # Safety
/// `engine` live (exclusive use); `name`, `fill` NUL-terminated; `parent` NULL or NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn osatura_add_rect_layer(engine: *mut OsaturaEngine, name: *const c_char, x: f32, y: f32, w: f32, h: f32, fill: *const c_char, parent: *const c_char) -> i32 {
    guarded(-1, || {
        let (name, fill, parent) = (cstr(name)?, cstr(fill)?, opt_str(parent)?);
        edit(engine, |a| a.add_rect_layer(name, x, y, w, h, fill, parent))
    })
}

/// # Safety
/// As [`osatura_add_rect_layer`].
#[no_mangle]
pub unsafe extern "C" fn osatura_add_ellipse_layer(engine: *mut OsaturaEngine, name: *const c_char, cx: f32, cy: f32, rx: f32, ry: f32, fill: *const c_char, parent: *const c_char) -> i32 {
    guarded(-1, || {
        let (name, fill, parent) = (cstr(name)?, cstr(fill)?, opt_str(parent)?);
        edit(engine, |a| a.add_ellipse_layer(name, cx, cy, rx, ry, fill, parent))
    })
}

/// # Safety
/// `engine` live (exclusive use); `name` NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn osatura_add_group_layer(engine: *mut OsaturaEngine, name: *const c_char) -> i32 {
    guarded(-1, || {
        let name = cstr(name)?;
        edit(engine, |a| a.add_group_layer(name))
    })
}

/// # Safety
/// `engine` live (exclusive use); `name` NUL-terminated; `parent` NULL or NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn osatura_add_bone(engine: *mut OsaturaEngine, name: *const c_char, parent: *const c_char, length: f32, x: f32, y: f32, angle: f32) -> i32 {
    guarded(-1, || {
        let (name, parent) = (cstr(name)?, opt_str(parent)?);
        edit(engine, |a| a.add_bone(name, parent, length, x, y, angle))
    })
}

/// `interpolation`: 0 linear, 1 step, 2 ease-in, 3 ease-out, 4 ease-in-out, 5 smooth.
///
/// # Safety
/// `engine` live (exclusive use); `target`, `prop` NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn osatura_set_key(engine: *mut OsaturaEngine, target: *const c_char, prop: *const c_char, frame: f32, value: f32, interpolation: i32) -> i32 {
    guarded(-1, || {
        let (target, prop) = (cstr(target)?, cstr(prop)?);
        let ip = match interpolation {
            0 => Interpolation::Linear,
            1 => Interpolation::Step,
            2 => Interpolation::EaseIn,
            3 => Interpolation::EaseOut,
            4 => Interpolation::EaseInOut,
            5 => Interpolation::Smooth,
            n => return Err(format!("unknown interpolation {n} (0..5)")),
        };
        edit(engine, |a| a.set_key(target, prop, frame, value, ip))
    })
}

/// Simulate the comma-separated bone `chain` (root first) with a preset ("hair", "tail", "rope",
/// "antenna", "cape"). Returns the number of channels written (0 = nothing, see header) or negative.
///
/// # Safety
/// `engine` live (exclusive use); `chain`, `preset` NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn osatura_bake_physics(engine: *mut OsaturaEngine, chain: *const c_char, preset: *const c_char, overwrite: i32) -> i32 {
    guarded(-1, || {
        let (chain, preset) = (cstr(chain)?, cstr(preset)?);
        let names: Vec<&str> = chain.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
        let e = eng_mut(engine)?;
        let r = e.anim.bake_physics(&names, preset, overwrite != 0).map_err(|e| e.to_string())?;
        e.info = e.anim.info();
        Ok(r.added.min(i32::MAX as usize) as i32)
    })
}

/// Undo the last edit: 1 = undone, 0 = nothing to undo, negative = error.
///
/// # Safety
/// `engine` live (exclusive use).
#[no_mangle]
pub unsafe extern "C" fn osatura_undo(engine: *mut OsaturaEngine) -> i32 {
    guarded(-1, || {
        let e = eng_mut(engine)?;
        let r = e.anim.undo();
        e.info = e.anim.info();
        Ok(r as i32)
    })
}

/// Redo: 1 = redone, 0 = nothing to redo, negative = error.
///
/// # Safety
/// `engine` live (exclusive use).
#[no_mangle]
pub unsafe extern "C" fn osatura_redo(engine: *mut OsaturaEngine) -> i32 {
    guarded(-1, || {
        let e = eng_mut(engine)?;
        let r = e.anim.redo();
        e.info = e.anim.info();
        Ok(r as i32)
    })
}

/// # Safety
/// `engine` must be null or a live handle.
#[no_mangle]
pub unsafe extern "C" fn osatura_layer_count(engine: *const OsaturaEngine) -> u32 {
    match eng(engine) {
        Some(e) => e.anim.layer_names().len() as u32,
        None => {
            set_error("osatura_layer_count: null engine");
            0
        }
    }
}

/// Path of layer `index` ("group/child"); same copy rules as `osatura_bone_name`.
///
/// # Safety
/// `engine` live or null; `out` has `cap` writable bytes (or is NULL with `cap == 0`).
#[no_mangle]
pub unsafe extern "C" fn osatura_layer_name(engine: *const OsaturaEngine, index: u32, out: *mut c_char, cap: usize) -> i32 {
    guarded(-1, || {
        let e = eng(engine).ok_or("null engine")?;
        let names = e.anim.layer_names();
        let name = names.get(index as usize).ok_or("layer index out of range")?;
        Ok(put_str(name, out, cap))
    })
}

/// Owned byte buffer returned by the export functions.
pub struct OsaturaBlob {
    bytes: Vec<u8>,
}

fn blob(bytes: Vec<u8>) -> *mut OsaturaBlob {
    Box::into_raw(Box::new(OsaturaBlob { bytes }))
}

/// Encode the whole animation. `format`: 0 GIF, 1 binary glTF, 2 Lottie JSON, 3 HTML player,
/// 4 APNG. `scale` (0.05..4) applies to GIF and APNG. NULL on error.
///
/// # Safety
/// `engine` live.
#[no_mangle]
pub unsafe extern "C" fn osatura_export(engine: *const OsaturaEngine, format: i32, scale: f32) -> *mut OsaturaBlob {
    guarded(std::ptr::null_mut(), || {
        let e = eng(engine).ok_or("null engine")?;
        let f = match format {
            0 => ExportFormat::Gif,
            1 => ExportFormat::Glb,
            2 => ExportFormat::Lottie,
            3 => ExportFormat::Html,
            4 => ExportFormat::Apng,
            n => return Err(format!("unknown export format {n} (0..4)")),
        };
        Ok(blob(e.anim.export(f, scale).map_err(|e| e.to_string())?))
    })
}

/// One frame as a PNG file. NULL on error.
///
/// # Safety
/// `engine` live.
#[no_mangle]
pub unsafe extern "C" fn osatura_render_png(engine: *const OsaturaEngine, frame: f32, scale: f32) -> *mut OsaturaBlob {
    guarded(std::ptr::null_mut(), || {
        let e = eng(engine).ok_or("null engine")?;
        Ok(blob(e.anim.render_png(frame, &RenderSettings::new(scale)).map_err(|e| e.to_string())?))
    })
}

/// One frame as an OpenEXR file (linear, premultiplied); `float32` non-zero stores FLOAT, else HALF.
/// NULL on error.
///
/// # Safety
/// `engine` live.
#[no_mangle]
pub unsafe extern "C" fn osatura_render_exr(engine: *const OsaturaEngine, frame: f32, scale: f32, float32: i32) -> *mut OsaturaBlob {
    guarded(std::ptr::null_mut(), || {
        let e = eng(engine).ok_or("null engine")?;
        Ok(blob(e.anim.render_exr(frame, &RenderSettings::new(scale), float32 != 0).map_err(|e| e.to_string())?))
    })
}

/// The project as JSON text (UTF-8, no terminator in the blob). NULL on error.
///
/// # Safety
/// `engine` live.
#[no_mangle]
pub unsafe extern "C" fn osatura_to_json(engine: *const OsaturaEngine) -> *mut OsaturaBlob {
    guarded(std::ptr::null_mut(), || {
        let e = eng(engine).ok_or("null engine")?;
        Ok(blob(e.anim.to_json().into_bytes()))
    })
}

/// Pointer to the blob's bytes (valid until `osatura_blob_free`), or NULL for a NULL blob.
///
/// # Safety
/// `b` null or live.
#[no_mangle]
pub unsafe extern "C" fn osatura_blob_data(b: *const OsaturaBlob) -> *const u8 {
    match b.as_ref() {
        Some(b) => b.bytes.as_ptr(),
        None => std::ptr::null(),
    }
}

/// Size of the blob in bytes (0 for NULL).
///
/// # Safety
/// `b` null or live.
#[no_mangle]
pub unsafe extern "C" fn osatura_blob_size(b: *const OsaturaBlob) -> usize {
    b.as_ref().map_or(0, |b| b.bytes.len())
}

/// Free a blob exactly once; NULL is allowed.
///
/// # Safety
/// `b` null or a blob not yet freed.
#[no_mangle]
pub unsafe extern "C" fn osatura_blob_free(b: *mut OsaturaBlob) {
    if !b.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(b))));
    }
}
