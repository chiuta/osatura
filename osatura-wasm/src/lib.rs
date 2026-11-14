// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! WebAssembly-facing ABI for OSATURA: plain `extern "C"` functions, no wasm-bindgen.
//!
//! Design: the host (JS) allocates a buffer with `arm_alloc`, copies the project JSON into the
//! module memory, calls `arm_open`, and later asks for a frame. Results (SVG text or RGBA
//! pixels) are kept inside the handle; `arm_out_ptr`/`arm_out_len` expose them and the host copies
//! them out before the next call. Errors are returned as negative codes / null and the text is
//! available through `arm_error_ptr`/`arm_error_len`. Panics are caught ONLY on targets that unwind (native
//! test builds, `wasm32-wasip1` with `-C panic=unwind`): on `wasm32-unknown-unknown` the default is
//! `panic=abort`, `catch_unwind` cannot intercept anything there, and a panic traps the module (the host sees a
//! `WebAssembly.RuntimeError: unreachable` and must discard the instance). The engine is written not to panic on
//! user input, and every size is checked before it is allocated, but this layer cannot promise to survive a bug.
//! Everything is single-threaded (wasm32-unknown-unknown has no threads), so a global error slot is used.
//! Only the stable `osatura_core::prelude` facade is used; nothing here touches fs/process/time.

use osatura_core::prelude::*;
use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};

pub const ABI_VERSION: u32 = 1;

pub struct Handle {
    anim: Animation,
    info: AnimationInfo,
    player: Option<Player>,
    out: Vec<u8>,
    out_w: u32,
    out_h: u32,
}

thread_local! {
    static LAST_ERROR: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn set_error(m: impl Into<String>) {
    let s = m.into();
    LAST_ERROR.with(|e| *e.borrow_mut() = s.into_bytes());
}

fn guarded<T>(fail: T, f: impl FnOnce() -> Result<T, String>) -> T {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => v,
        Ok(Err(m)) => {
            set_error(m);
            fail
        }
        // only reachable when panics unwind (see the module docs); under panic=abort the module has trapped already
        Err(_) => {
            set_error("internal error (panic) inside osatura-wasm");
            fail
        }
    }
}

const MAX_ALLOC: usize = 256 * 1024 * 1024;

#[no_mangle]
pub extern "C" fn arm_abi_version() -> u32 {
    ABI_VERSION
}

/// Tells the module what time it is. `wasm32-unknown-unknown` has no system clock (the std time functions panic), so the
/// module's clock (`osatura_core::clock`) stays at 0 until the host calls this, e.g. with `performance.now()` and
/// `Date.now()` before a call. Optional: without it durations are 0 and time budgets never expire. Added after ABI
/// version 1 was fixed; modules and hosts that do not know it are unaffected.
#[no_mangle]
pub extern "C" fn arm_set_clock(mono_ms: f64, unix_ms: f64) {
    osatura_core::clock::set_host_clock(mono_ms, unix_ms);
}

/// Allocate `len` zeroed bytes in module memory for the host to fill. Null on 0 or too large.
#[no_mangle]
pub extern "C" fn arm_alloc(len: usize) -> *mut u8 {
    if len == 0 || len > MAX_ALLOC {
        set_error("arm_alloc: bad length");
        return std::ptr::null_mut();
    }
    let b = vec![0u8; len].into_boxed_slice();
    Box::into_raw(b) as *mut u8
}

/// # Safety
/// `ptr`/`len` must be exactly a pair returned by / passed to `arm_alloc`, not yet freed.
#[no_mangle]
pub unsafe extern "C" fn arm_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 && len <= MAX_ALLOC {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
    }
}

/// Parse and validate a project from `len` bytes of UTF-8 JSON. Returns a handle or null.
/// # Safety
/// `json` must point to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_open(json: *const u8, len: usize) -> *mut Handle {
    guarded(std::ptr::null_mut(), || {
        if json.is_null() || len == 0 || len > MAX_ALLOC {
            return Err("arm_open: null pointer or bad length".into());
        }
        let text = std::str::from_utf8(std::slice::from_raw_parts(json, len)).map_err(|e| format!("project is not UTF-8: {e}"))?;
        let anim = Animation::from_json(text).map_err(|e| e.to_string())?;
        let info = anim.info();
        Ok(Box::into_raw(Box::new(Handle { anim, info, player: None, out: Vec::new(), out_w: 0, out_h: 0 })))
    })
}

/// # Safety
/// `h` must be null or a live handle from `arm_open`.
#[no_mangle]
pub unsafe extern "C" fn arm_close(h: *mut Handle) {
    if !h.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(h))));
    }
}

unsafe fn handle<'a>(h: *mut Handle) -> Result<&'a mut Handle, String> {
    h.as_mut().ok_or_else(|| "null handle".to_string())
}

macro_rules! getter {
    ($name:ident, $ty:ty, $zero:expr, |$i:ident| $body:expr) => {
        /// # Safety
        /// `h` must be null or a live handle.
        #[no_mangle]
        pub unsafe extern "C" fn $name(h: *mut Handle) -> $ty {
            match h.as_ref() {
                Some(x) => {
                    let $i = &x.info;
                    $body
                }
                None => {
                    set_error(concat!(stringify!($name), ": null handle"));
                    $zero
                }
            }
        }
    };
}
getter!(arm_frame_count, u32, 0, |i| i.frames);
getter!(arm_fps, f32, 0.0, |i| i.fps);
getter!(arm_width, f32, 0.0, |i| i.width);
getter!(arm_height, f32, 0.0, |i| i.height);

fn check_frame(frame: f32) -> Result<(), String> {
    if frame.is_finite() { Ok(()) } else { Err("frame must be finite".into()) }
}

/// Render an SVG document for `frame` into the handle's output buffer.
/// Returns the byte length (>= 0) or -1 on error.
/// # Safety
/// `h` must be a live handle.
#[no_mangle]
pub unsafe extern "C" fn arm_render_svg(h: *mut Handle, frame: f32, show_bones: i32) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        check_frame(frame)?;
        let svg = hd.anim.render_svg(frame, show_bones != 0).map_err(|e| e.to_string())?;
        hd.out = svg.into_bytes();
        hd.out_w = 0;
        hd.out_h = 0;
        i32::try_from(hd.out.len()).map_err(|_| "output too large".to_string())
    })
}

/// Render straight-alpha RGBA8 pixels (row-major, width*height*4 bytes) at `scale` (0.05..4).
/// Returns the byte length (>= 0) or -1; width/height via `arm_out_width`/`arm_out_height`.
/// # Safety
/// `h` must be a live handle.
#[no_mangle]
pub unsafe extern "C" fn arm_render_rgba(h: *mut Handle, frame: f32, scale: f32) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        check_frame(frame)?;
        if !scale.is_finite() || !(0.05..=4.0).contains(&scale) {
            return Err("scale must be within 0.05..4".into());
        }
        // refuse before allocating: the pixel buffer must fit the module's memory and an i32 length
        let (pw, ph) = ((hd.info.width * scale).round().max(1.0) as u64, (hd.info.height * scale).round().max(1.0) as u64);
        let need = pw.saturating_mul(ph).saturating_mul(4);
        if need > MAX_ALLOC as u64 {
            return Err(format!("output would be {pw}x{ph} pixels ({need} bytes), more than the limit of {MAX_ALLOC} bytes: lower the scale"));
        }
        let img = hd.anim.render_rgba(frame, &RenderSettings::new(scale)).map_err(|e| e.to_string())?;
        hd.out_w = img.width;
        hd.out_h = img.height;
        hd.out = img.pixels;
        i32::try_from(hd.out.len()).map_err(|_| "output too large".to_string())
    })
}

/// # Safety
/// `h` must be null or a live handle. The pointer is valid until the next render call or close.
#[no_mangle]
pub unsafe extern "C" fn arm_out_ptr(h: *mut Handle) -> *const u8 {
    h.as_ref().map_or(std::ptr::null(), |x| x.out.as_ptr())
}
/// # Safety
/// `h` must be null or a live handle.
#[no_mangle]
pub unsafe extern "C" fn arm_out_len(h: *mut Handle) -> usize {
    h.as_ref().map_or(0, |x| x.out.len())
}
/// # Safety
/// `h` must be null or a live handle.
#[no_mangle]
pub unsafe extern "C" fn arm_out_width(h: *mut Handle) -> u32 {
    h.as_ref().map_or(0, |x| x.out_w)
}
/// # Safety
/// `h` must be null or a live handle.
#[no_mangle]
pub unsafe extern "C" fn arm_out_height(h: *mut Handle) -> u32 {
    h.as_ref().map_or(0, |x| x.out_h)
}


// ---------------------------------------------------------------------------------------------
// Interactive playback: state machines. Strings are passed as (ptr, len) UTF-8 in module memory
// (copy them in with arm_alloc). Output goes through the same out buffer as the renders.
// ---------------------------------------------------------------------------------------------

unsafe fn text<'a>(ptr: *const u8, len: usize) -> Result<&'a str, String> {
    if ptr.is_null() || len == 0 || len > 4096 {
        return Err("bad string pointer or length".into());
    }
    std::str::from_utf8(std::slice::from_raw_parts(ptr, len)).map_err(|_| "string is not UTF-8".to_string())
}

fn player(hd: &mut Handle) -> Result<&mut Player, String> {
    hd.player.as_mut().ok_or_else(|| "no player started (call arm_player_start)".to_string())
}

/// Number of state machines in the project.
/// # Safety
/// `h` must be null or a live handle.
#[no_mangle]
pub unsafe extern "C" fn arm_machine_count(h: *mut Handle) -> u32 {
    match h.as_ref() {
        Some(x) => x.anim.machine_names().len() as u32,
        None => {
            set_error("arm_machine_count: null handle");
            0
        }
    }
}

/// Write the name of machine `index` into the out buffer; returns its byte length or -1.
/// # Safety
/// `h` must be a live handle.
#[no_mangle]
pub unsafe extern "C" fn arm_machine_name(h: *mut Handle, index: u32) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        let names = hd.anim.machine_names();
        let n = names.get(index as usize).ok_or("machine index out of range")?;
        hd.out = n.clone().into_bytes();
        Ok(hd.out.len() as i32)
    })
}

/// Start (or restart) the named machine. Returns 0 or -1.
/// # Safety
/// `h` live; `name` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_player_start(h: *mut Handle, name: *const u8, len: usize) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        hd.player = Some(hd.anim.player(text(name, len)?).map_err(|e| e.to_string())?);
        Ok(0)
    })
}

/// Drop the player.
/// # Safety
/// `h` must be null or a live handle.
#[no_mangle]
pub unsafe extern "C" fn arm_player_stop(h: *mut Handle) {
    if let Some(x) = h.as_mut() {
        x.player = None;
    }
}

/// # Safety
/// `h` live; `name` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_player_set_bool(h: *mut Handle, name: *const u8, len: usize, value: i32) -> i32 {
    guarded(-1, || player(handle(h)?)?.set_bool(text(name, len)?, value != 0).map(|_| 0).map_err(|e| e.to_string()))
}

/// # Safety
/// `h` live; `name` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_player_set_number(h: *mut Handle, name: *const u8, len: usize, value: f64) -> i32 {
    guarded(-1, || player(handle(h)?)?.set_number(text(name, len)?, value).map(|_| 0).map_err(|e| e.to_string()))
}

/// # Safety
/// `h` live; `name` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_player_fire(h: *mut Handle, name: *const u8, len: usize) -> i32 {
    guarded(-1, || player(handle(h)?)?.fire(text(name, len)?).map(|_| 0).map_err(|e| e.to_string()))
}

/// Publish an external value for the project's data bindings.
/// # Safety
/// `h` live; `path` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_player_set_value(h: *mut Handle, path: *const u8, len: usize, value: f64) -> i32 {
    guarded(-1, || player(handle(h)?)?.set_value(text(path, len)?, value).map(|_| 0).map_err(|e| e.to_string()))
}

/// Advance by `dt` seconds (0..10).
/// # Safety
/// `h` live.
#[no_mangle]
pub unsafe extern "C" fn arm_player_advance(h: *mut Handle, dt: f64) -> i32 {
    guarded(-1, || player(handle(h)?)?.advance(dt).map(|_| 0).map_err(|e| e.to_string()))
}

/// Write the layer's current state name into the out buffer; returns its length or -1.
/// # Safety
/// `h` live; `layer` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_player_state(h: *mut Handle, layer: *const u8, len: usize) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        let s = player(hd)?.state(text(layer, len)?).ok_or("unknown layer")?;
        hd.out = s.into_bytes();
        Ok(hd.out.len() as i32)
    })
}

/// Write the pending events (JSON array) into the out buffer and consume them.
/// # Safety
/// `h` live.
#[no_mangle]
pub unsafe extern "C" fn arm_player_events(h: *mut Handle) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        let j = player(hd)?.take_events_json();
        hd.out = j.into_bytes();
        Ok(hd.out.len() as i32)
    })
}

/// Render the player's current pose as SVG into the out buffer.
/// # Safety
/// `h` live.
#[no_mangle]
pub unsafe extern "C" fn arm_player_render_svg(h: *mut Handle, show_bones: i32) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        let svg = player(hd)?.render_svg(show_bones != 0).map_err(|e| e.to_string())?;
        hd.out = svg.into_bytes();
        hd.out_w = 0;
        hd.out_h = 0;
        i32::try_from(hd.out.len()).map_err(|_| "output too large".to_string())
    })
}

/// Render the player's current pose to RGBA8 (size via `arm_out_width` / `arm_out_height`).
/// # Safety
/// `h` live.
#[no_mangle]
pub unsafe extern "C" fn arm_player_render_rgba(h: *mut Handle, scale: f32) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        if !scale.is_finite() || !(0.05..=4.0).contains(&scale) {
            return Err("scale must be within 0.05..4".into());
        }
        let img = player(hd)?.render_rgba(&RenderSettings::new(scale)).map_err(|e| e.to_string())?;
        hd.out_w = img.width;
        hd.out_h = img.height;
        hd.out = img.pixels;
        i32::try_from(hd.out.len()).map_err(|_| "output too large".to_string())
    })
}

// ---------------------------------------------------------------------------------------------
// Layout, scrolling, level of detail (additive)
// ---------------------------------------------------------------------------------------------

fn json_list(items: &[String]) -> String {
    let q = |s: &String| {
        let mut o = String::from("\"");
        for c in s.chars() {
            match c {
                '"' => o.push_str("\\\""),
                '\\' => o.push_str("\\\\"),
                c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
                c => o.push(c),
            }
        }
        o.push('"');
        o
    };
    format!("[{}]", items.iter().map(q).collect::<Vec<_>>().join(","))
}

/// Lay the project out for another canvas size (stage units). Returns 0 or -1.
/// # Safety
/// `h` live.
#[no_mangle]
pub unsafe extern "C" fn arm_player_set_canvas(h: *mut Handle, width: f32, height: f32) -> i32 {
    guarded(-1, || player(handle(h)?)?.set_canvas_size(width, height).map(|_| 0).map_err(|e| e.to_string()))
}

/// Write `{"canvas":[w,h],"scrolls":[ids],"lod":level}` into the out buffer.
/// # Safety
/// `h` live.
#[no_mangle]
pub unsafe extern "C" fn arm_player_layout(h: *mut Handle) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        let pl = player(hd)?;
        let (w, hh) = pl.canvas_size();
        let j = format!("{{\"canvas\":[{w},{hh}],\"scrolls\":{},\"lod\":{}}}", json_list(&pl.scroll_ids()), pl.lod_level());
        hd.out = j.into_bytes();
        Ok(hd.out.len() as i32)
    })
}

/// Pointer drag of a scroll region: the raw offset in pixels. Returns 0 or -1.
/// # Safety
/// `h` live; `id` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_player_scroll_drag(h: *mut Handle, id: *const u8, len: usize, x: f32, y: f32) -> i32 {
    guarded(-1, || player(handle(h)?)?.scroll_drag(text(id, len)?, x, y).map(|_| 0).map_err(|e| e.to_string()))
}

/// Release a scroll region with a velocity in pixels per second. Returns 0 or -1.
/// # Safety
/// `h` live; `id` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_player_scroll_release(h: *mut Handle, id: *const u8, len: usize, vx: f32, vy: f32) -> i32 {
    guarded(-1, || player(handle(h)?)?.scroll_release(text(id, len)?, vx, vy).map(|_| 0).map_err(|e| e.to_string()))
}

/// Write `{"offset":[x,y],"max":[x,y],"settled":bool}` of a scroll region into the out buffer.
/// # Safety
/// `h` live; `id` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn arm_player_scroll_info(h: *mut Handle, id: *const u8, len: usize) -> i32 {
    guarded(-1, || {
        let hd = handle(h)?;
        let pl = player(hd)?;
        let id = text(id, len)?;
        let (o, m, s) = (pl.scroll_offset(id).ok_or("unknown scroll")?, pl.scroll_max(id).unwrap_or([0.0; 2]), pl.scroll_settled(id).unwrap_or(true));
        let j = format!("{{\"offset\":[{},{}],\"max\":[{},{}],\"settled\":{s}}}", o[0], o[1], m[0], m[1]);
        hd.out = j.into_bytes();
        Ok(hd.out.len() as i32)
    })
}

/// Screen size in pixels and the largest error accepted there; selects the level of detail.
/// # Safety
/// `h` live.
#[no_mangle]
pub unsafe extern "C" fn arm_player_set_screen(h: *mut Handle, width_px: f32, height_px: f32, max_error_px: f32) -> i32 {
    guarded(-1, || player(handle(h)?)?.set_screen_size(width_px, height_px, max_error_px).map(|_| 0).map_err(|e| e.to_string()))
}

/// UTF-8 bytes of the last error message (not NUL-terminated); valid until the next error.
#[no_mangle]
pub extern "C" fn arm_error_ptr() -> *const u8 {
    LAST_ERROR.with(|e| e.borrow().as_ptr())
}
#[no_mangle]
pub extern "C" fn arm_error_len() -> usize {
    LAST_ERROR.with(|e| e.borrow().len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_json() -> String {
        Animation::demo().to_json()
    }
    fn open(json: &str) -> *mut Handle {
        let p = arm_alloc(json.len());
        assert!(!p.is_null());
        unsafe {
            std::ptr::copy_nonoverlapping(json.as_ptr(), p, json.len());
            let h = arm_open(p, json.len());
            arm_free(p, json.len());
            h
        }
    }
    fn err() -> String {
        let (p, n) = (arm_error_ptr(), arm_error_len());
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(p, n) }).into_owned()
    }

    #[test]
    fn abi_and_info() {
        assert_eq!(arm_abi_version(), 1);
        // the clock export accepts any value without panicking (NaN and negative readings are ignored)
        arm_set_clock(10.0, 1.7e12);
        arm_set_clock(f64::NAN, -5.0);
        let h = open(&demo_json());
        assert!(!h.is_null());
        unsafe {
            assert!(arm_frame_count(h) > 0);
            assert!(arm_fps(h) > 0.0);
            assert!(arm_width(h) > 0.0 && arm_height(h) > 0.0);
            arm_close(h);
        }
    }

    #[test]
    fn svg_render() {
        let h = open(&demo_json());
        unsafe {
            let n = arm_render_svg(h, 3.0, 0);
            assert!(n > 0);
            assert_eq!(arm_out_len(h), n as usize);
            let s = std::str::from_utf8(std::slice::from_raw_parts(arm_out_ptr(h), n as usize)).unwrap();
            assert!(s.contains("<svg"));
            assert_eq!(arm_render_svg(h, f32::NAN, 0), -1);
            assert!(err().contains("finite"));
            arm_close(h);
        }
    }

    #[test]
    fn rgba_render_and_scale() {
        let h = open(&demo_json());
        unsafe {
            let n = arm_render_rgba(h, 0.0, 0.5);
            assert!(n > 0);
            let (w, hh) = (arm_out_width(h), arm_out_height(h));
            assert_eq!(n as usize, (w * hh * 4) as usize);
            assert!(std::slice::from_raw_parts(arm_out_ptr(h), n as usize).iter().any(|&b| b != 0));
            assert_eq!(arm_render_rgba(h, 0.0, 100.0), -1);
            assert_eq!(arm_render_rgba(h, 0.0, f32::NAN), -1);
            arm_close(h);
        }
    }

    #[test]
    fn huge_output_is_refused_before_allocating() {
        let mut p = osatura_core::demo::project();
        p.width = 12000.0;
        p.height = 12000.0;
        let h = open(&p.to_json_pretty());
        assert!(!h.is_null(), "{}", err());
        unsafe {
            assert_eq!(arm_render_rgba(h, 0.0, 1.0), -1);
            assert!(err().contains("more than the limit"), "{}", err());
            assert_eq!(arm_out_len(h), 0, "nothing was allocated");
            arm_close(h);
        }
    }

    #[test]
    fn error_paths() {
        assert!(open("{ not json").is_null());
        assert!(!err().is_empty());
        assert!(unsafe { arm_open(std::ptr::null(), 5) }.is_null());
        assert!(unsafe { arm_open(b"x".as_ptr(), 0) }.is_null());
        assert!(open("\u{0}").is_null());
        let bad = [0xffu8, 0xfe];
        assert!(unsafe { arm_open(bad.as_ptr(), 2) }.is_null());
        assert!(err().contains("UTF-8"));
        unsafe {
            assert_eq!(arm_render_svg(std::ptr::null_mut(), 0.0, 0), -1);
            assert_eq!(arm_render_rgba(std::ptr::null_mut(), 0.0, 1.0), -1);
            assert_eq!(arm_frame_count(std::ptr::null_mut()), 0);
            assert!(arm_out_ptr(std::ptr::null_mut()).is_null());
            arm_close(std::ptr::null_mut());
            arm_free(std::ptr::null_mut(), 4);
        }
        assert!(arm_alloc(0).is_null());
        assert!(arm_alloc(usize::MAX).is_null());
    }

    fn machine_project_json() -> String {
        use osatura_core::anim::{Ease, Key};
        use osatura_core::machine::*;
        use osatura_core::motionlib::{Take, TakeTrack};
        use osatura_core::project::Prop;
        let mut p = osatura_core::demo::project();
        let bone = p.bones[0].name.clone();
        let mk = |n: &str, v: f32| {
            let keys = vec![Key { frame: 0.0, value: v, ease: Ease::Linear }, Key { frame: 10.0, value: v, ease: Ease::Linear }];
            osatura_core::clipstack::Clip::new(n, Take { name: n.into(), length: 10.0, tracks: vec![TakeTrack { target: bone.clone(), prop: Prop::BoneAngle, keys }] })
        };
        p.clips = vec![mk("a", 0.0), mk("b", 40.0)];
        let st = |n: &str| State { name: n.into(), source: StateSource::Clip { clip: n.into() }, mode: PlayMode::Loop, speed: 1.0, blend_in: 0.0 };
        p.machines.push(Machine {
            name: "hero".into(),
            inputs: vec![Input { name: "go".into(), kind: InputKind::Bool, default: 0.0, hold: false }],
            layers: vec![Layer {
                name: "body".into(),
                mask: vec![],
                entry: "a".into(),
                states: vec![st("a"), st("b")],
                transitions: vec![Transition { from: "a".into(), to: "b".into(), conditions: vec![Condition { input: "go".into(), op: Cmp::Eq, value: 1.0 }], exit_time: None, duration: Some(0.0), ease: Ease::Linear }],
            }],
        });
        p.to_json_pretty()
    }

    fn out_string(h: *mut Handle) -> String {
        unsafe { String::from_utf8(std::slice::from_raw_parts(arm_out_ptr(h), arm_out_len(h)).to_vec()).unwrap() }
    }

    #[test]
    fn player_state_machine() {
        let h = open(&machine_project_json());
        assert!(!h.is_null());
        let (go, hero, body) = (b"go", b"hero", b"body");
        unsafe {
            assert_eq!(arm_machine_count(h), 1);
            assert_eq!(arm_machine_name(h, 0), 4);
            assert_eq!(out_string(h), "hero");
            assert_eq!(arm_machine_name(h, 5), -1);
            // not started yet
            assert_eq!(arm_player_advance(h, 0.1), -1);
            assert!(err().contains("arm_player_start"));
            assert_eq!(arm_player_start(h, b"nope".as_ptr(), 4), -1);
            assert_eq!(arm_player_start(h, hero.as_ptr(), 4), 0);
            assert_eq!(arm_player_state(h, body.as_ptr(), 4), 1);
            assert_eq!(out_string(h), "a");
            let svg_a = arm_player_render_svg(h, 0);
            assert!(svg_a > 0);
            let a = out_string(h);
            assert_eq!(arm_player_set_bool(h, go.as_ptr(), 2, 1), 0);
            assert_eq!(arm_player_advance(h, 0.1), 0);
            assert_eq!(arm_player_state(h, body.as_ptr(), 4), 1);
            assert_eq!(out_string(h), "b");
            assert!(arm_player_render_svg(h, 0) > 0);
            assert_ne!(out_string(h), a, "the pose changed");
            assert!(arm_player_events(h) > 20);
            assert!(out_string(h).contains("\"state\":\"b\""));
            assert_eq!(arm_player_events(h), 2);
            assert!(arm_player_render_rgba(h, 0.25) > 0 && arm_out_width(h) > 0);
            assert_eq!(arm_player_render_rgba(h, 50.0), -1);
            // errors
            assert_eq!(arm_player_set_number(h, go.as_ptr(), 2, 1.0), -1);
            assert_eq!(arm_player_fire(h, go.as_ptr(), 2), -1);
            assert_eq!(arm_player_set_value(h, b"bad path".as_ptr(), 8, 1.0), -1);
            assert_eq!(arm_player_set_value(h, b"ok.path".as_ptr(), 7, 1.0), 0);
            assert_eq!(arm_player_advance(h, f64::NAN), -1);
            assert_eq!(arm_player_set_bool(h, std::ptr::null(), 0, 1), -1);
            assert_eq!(arm_player_state(h, b"zz".as_ptr(), 2), -1);
            assert_eq!(arm_player_advance(std::ptr::null_mut(), 0.1), -1);
            arm_player_stop(h);
            assert_eq!(arm_player_advance(h, 0.1), -1);
            arm_player_stop(std::ptr::null_mut());
            arm_close(h);
        }
    }

    #[test]
    fn player_layout_scroll_lod() {
        use osatura_core::containers::{Container, Dir, Member, RectRule, Scroll};
        use osatura_core::path::VPath;
        use osatura_core::project::Layer;
        let mut p = osatura_core::project::Project::from_json(&machine_project_json()).unwrap();
        for i in 0..10 {
            p.layers.push(Layer::vector(&format!("row{i}"), vec![VPath::rect("r", 0.0, 0.0, 100.0, 20.0).with_fill("#336699")]));
        }
        p.containers.push(Container::new("col", Dir::Column, (0..10).map(|i| Member::layer(&format!("row{i}"))).collect()));
        let mut sc = Scroll::new("list", "col");
        sc.viewport = RectRule::Pin { left: Some(0.0), top: Some(0.0), right: None, bottom: Some(0.0), width: Some(100.0), height: None };
        p.scrolls.push(sc);
        let h = open(&p.to_json_pretty());
        assert!(!h.is_null());
        let (hero, list) = (b"hero", b"list");
        unsafe {
            assert_eq!(arm_player_set_canvas(h, 100.0, 100.0), -1, "no player yet");
            assert_eq!(arm_player_start(h, hero.as_ptr(), 4), 0);
            assert!(arm_player_layout(h) > 10);
            assert!(out_string(h).contains("\"scrolls\":[\"list\"]"), "{}", out_string(h));
            assert_eq!(arm_player_set_canvas(h, 100.0, 120.0), 0);
            assert!(arm_player_scroll_info(h, list.as_ptr(), 4) > 10);
            assert!(out_string(h).contains("\"max\":[0,80"), "{}", out_string(h));
            assert_eq!(arm_player_scroll_drag(h, list.as_ptr(), 4, 0.0, 50.0), 0);
            assert!(arm_player_scroll_info(h, list.as_ptr(), 4) > 10);
            assert!(out_string(h).contains("\"offset\":[0,50]"), "{}", out_string(h));
            assert_eq!(arm_player_scroll_release(h, list.as_ptr(), 4, 0.0, 0.0), 0);
            assert_eq!(arm_player_advance(h, 1.0), 0);
            assert_eq!(arm_player_scroll_drag(h, b"zz".as_ptr(), 2, 0.0, 1.0), -1);
            assert_eq!(arm_player_scroll_info(h, b"zz".as_ptr(), 2), -1);
            assert_eq!(arm_player_set_canvas(h, 0.0, 5.0), -1);
            assert_eq!(arm_player_set_screen(h, 100.0, 120.0, 4.0), 0);
            assert_eq!(arm_player_set_screen(h, f32::NAN, 120.0, 4.0), -1);
            assert!(arm_player_render_rgba(h, 1.0) > 0 && arm_out_height(h) == 120);
            arm_close(h);
        }
    }

    #[test]
    fn alloc_free_roundtrip() {
        let p = arm_alloc(16);
        unsafe {
            assert!(std::slice::from_raw_parts(p, 16).iter().all(|&b| b == 0));
            arm_free(p, 16);
        }
    }
}
