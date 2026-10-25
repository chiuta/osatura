// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Open-format outputs and import (codec patent status not assessed; see docs/PATENT_NOTES.md).
//!
//! * APNG, PNG sprite sheet + JSON, Open Cel Animation (OCA) folder — export.
//! * OpenRaster (`.ora`, a zip of PNG layers + `stack.xml`) — import into a [`Project`].
//!
//! Clean-room design note: all formats here are openly specified; no vendor SDK, codec
//! pool or proprietary file layout is involved.

use crate::engine::Engine;
use crate::math::Vec2;
use crate::project::{Layer, Project};
use crate::raster::{render_rgba, RasterOptions};
use crate::style::Blend;
use serde_json::{json, Value};
use std::io::{Cursor, Read};
use std::path::Path;

pub fn encode_rgba_png(w: u32, h: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        wr.write_image_data(rgba).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

fn frames_rgba(engine: &Engine, scale: f32) -> (u32, u32, Vec<Vec<u8>>) {
    let opts = RasterOptions { scale, show_bones: false, transparent: false };
    let (mut w, mut h) = (0, 0);
    let frames = (0..engine.project.frames)
        .map(|f| {
            let (fw, fh, d) = render_rgba(engine, f as f32, &opts);
            (w, h) = (fw, fh);
            d
        })
        .collect();
    (w, h, frames)
}

/// Looping animated PNG (full 8-bit alpha, lossless, open specification).
pub fn apng_bytes(engine: &Engine, scale: f32) -> Result<Vec<u8>, String> {
    crate::raster::check_sequence(&engine.project, scale)?;
    let (w, h, frames) = frames_rgba(engine, scale);
    apng_from_frames(w, h, &frames, engine.project.fps)
}

/// Looping animated PNG from ready RGBA frames (all `w`x`h`) at `fps`.
pub fn apng_from_frames(w: u32, h: u32, frames: &[Vec<u8>], fps: f32) -> Result<Vec<u8>, String> {
    if frames.is_empty() {
        return Err("no frames".into());
    }
    let n = frames.len() as u32;
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_animated(n, 0).map_err(|e| e.to_string())?;
        // delay = 1000 / fps milliseconds, expressed as a fraction
        enc.set_frame_delay(1000, (fps * 1000.0).round().max(1.0) as u16).map_err(|e| e.to_string())?;
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        for f in frames {
            wr.write_image_data(f).map_err(|e| e.to_string())?;
        }
        wr.finish().map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// Sprite sheet: all frames tiled on one transparent PNG, plus a JSON description in
/// OSATURA's own simple layout (`frames[i] = {x, y, w, h}`).
pub fn spritesheet(engine: &Engine, scale: f32, columns: usize, image_name: &str) -> Result<(Vec<u8>, Value), String> {
    let n = engine.project.frames as usize;
    let cols = if columns == 0 { (n as f32).sqrt().ceil() as usize } else { columns }.clamp(1, n.max(1));
    let rows = n.div_ceil(cols);
    // check the size *before* rendering anything
    let fw = (engine.project.width * scale).ceil() as usize;
    let fh = (engine.project.height * scale).ceil() as usize;
    let (sw, sh) = (fw * cols, fh * rows);
    if sw > 16384 || sh > 16384 {
        return Err(format!("sheet would be {sw}×{sh}; lower --scale or use more columns"));
    }
    crate::raster::check_size(sw as f32, sh as f32, 1.0).map_err(|e| format!("sprite sheet: {e}"))?;
    crate::raster::check_sequence(&engine.project, scale)?;
    let (w, h, frames) = frames_rgba(engine, scale);
    let mut sheet = vec![0u8; sw * sh * 4];
    let mut meta = Vec::with_capacity(n);
    for (i, f) in frames.iter().enumerate() {
        let (cx, cy) = ((i % cols) * w as usize, (i / cols) * h as usize);
        for y in 0..h as usize {
            let src = &f[y * w as usize * 4..(y + 1) * w as usize * 4];
            let dst = ((cy + y) * sw + cx) * 4;
            sheet[dst..dst + src.len()].copy_from_slice(src);
        }
        meta.push(json!({"index": i, "x": cx, "y": cy, "w": w, "h": h}));
    }
    let png = encode_rgba_png(sw as u32, sh as u32, &sheet)?;
    let j = json!({
        "image": image_name, "size": {"w": sw, "h": sh}, "frame_size": {"w": w, "h": h},
        "fps": engine.project.fps, "columns": cols, "rows": rows, "frames": meta
    });
    Ok((png, j))
}

/// Write an Open Cel Animation folder: `<name>.oca` (JSON) plus PNG cels. With `per_layer`,
/// each top-level layer becomes its own OCA layer (rendered alone on transparency); otherwise
/// one `render` layer holds the composited frames.
pub fn write_oca(project: &Project, asset_dir: &Path, out_dir: &Path, scale: f32, per_layer: bool) -> Result<std::path::PathBuf, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let mut layers_json = Vec::new();
    let groups: Vec<(String, Project)> = if per_layer {
        project
            .layers
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let mut p = project.clone();
                p.background = None;
                for (j, x) in p.layers.iter_mut().enumerate() {
                    x.visible = x.visible && j == i;
                }
                (l.name.clone(), p)
            })
            .collect()
    } else {
        vec![("render".to_string(), project.clone())]
    };
    for (li, (name, p)) in groups.into_iter().enumerate() {
        let eng = Engine::with_asset_dir(p, asset_dir).map_err(|e| e.join("; "))?;
        crate::raster::check_sequence(&eng.project, scale)?;
        let dir = out_dir.join(format!("layer{li}"));
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let (w, h, frames) = frames_rgba(&eng, scale);
        let mut fr = Vec::new();
        for (i, f) in frames.iter().enumerate() {
            let file = format!("layer{li}/f{i:04}.png");
            crate::fsutil::write_atomic(&out_dir.join(&file), &encode_rgba_png(w, h, f)?)?;
            fr.push(json!({"name": format!("f{i}"), "frameNumber": i, "fileName": file, "opacity": 1.0, "position": [0, 0]}));
        }
        layers_json.push(json!({"name": name, "type": "paintlayer", "visible": true, "opacity": 1.0, "frames": fr}));
    }
    let doc = json!({
        "name": project.name, "ocaVersion": "1.1", "frameRate": project.fps,
        "startTime": 0, "endTime": project.frames.saturating_sub(1),
        "width": (project.width * scale).round().max(1.0) as u64, "height": (project.height * scale).round().max(1.0) as u64,
        "backgroundColour": "#00000000", "originApp": "OSATURA", "originAppVersion": env!("CARGO_PKG_VERSION"),
        "layers": layers_json
    });
    let path = out_dir.join(format!("{}.oca", project.name.replace(['/', '\\'], "_")));
    crate::fsutil::write_atomic(&path, serde_json::to_string_pretty(&doc).unwrap().as_bytes())?;
    Ok(path)
}

/// dotLottie 2.0 container (zip): `manifest.json` (`version` "2") + `a/animation.json`
/// (<https://dotlottie.io/spec/2.0/>; playback options such as loop belong to the player or a state machine).
pub fn dotlottie_bytes(engine: &Engine) -> Result<Vec<u8>, String> {
    dotlottie_bytes_report(engine).map(|(b, _)| b)
}

/// [`dotlottie_bytes`] plus the Lottie export warnings (everything the format could not carry).
pub fn dotlottie_bytes_report(engine: &Engine) -> Result<(Vec<u8>, Vec<crate::lottie_export::LottieWarning>), String> {
    use std::io::Write;
    let x = crate::lottie_export::lottie_export(engine);
    let lottie = x.doc.to_string();
    let manifest = json!({"version": "2", "generator": "OSATURA", "animations": [{"id": "animation"}]});
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let o = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        z.start_file("manifest.json", o).map_err(|e| e.to_string())?;
        z.write_all(manifest.to_string().as_bytes()).map_err(|e| e.to_string())?;
        z.start_file("a/animation.json", o).map_err(|e| e.to_string())?;
        z.write_all(lottie.as_bytes()).map_err(|e| e.to_string())?;
        z.finish().map_err(|e| e.to_string())?;
    }
    Ok((buf, x.warnings))
}

// ---- OpenRaster import -------------------------------------------------------------------------

const MAX_ENTRIES: usize = 2000;
const MAX_ENTRY_BYTES: u64 = 128 * 1024 * 1024;

fn attr(tag: &str, name: &str) -> Option<String> {
    let pat = format!(" {name}=");
    let i = tag.find(&pat)? + pat.len();
    let q = tag[i..].chars().next()?;
    if q != '"' && q != '\'' {
        return None;
    }
    let rest = &tag[i + 1..];
    let end = rest.find(q)?;
    Some(
        rest[..end]
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&"),
    )
}

fn blend_from_ora(op: &str) -> Blend {
    match op {
        "svg:multiply" => Blend::Multiply,
        "svg:screen" => Blend::Screen,
        "svg:overlay" => Blend::Overlay,
        "svg:darken" => Blend::Darken,
        "svg:lighten" => Blend::Lighten,
        "svg:color-dodge" => Blend::ColorDodge,
        "svg:color-burn" => Blend::ColorBurn,
        "svg:hard-light" => Blend::HardLight,
        "svg:soft-light" => Blend::SoftLight,
        "svg:difference" => Blend::Difference,
        "svg:exclusion" => Blend::Exclusion,
        "svg:plus" => Blend::Add,
        _ => Blend::Normal,
    }
}

/// Import an OpenRaster document as a one-frame project with one image layer per raster layer
/// (bottom-to-top, hidden layers kept but invisible, groups flattened). Layer PNGs are embedded.
pub fn import_ora(bytes: &[u8], name: &str, fps: f32, frames: u32) -> Result<Project, String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("not a zip/ORA file: {e}"))?;
    if zip.len() > MAX_ENTRIES {
        return Err("archive has too many entries".into());
    }
    let mut read = |path: &str| -> Result<Vec<u8>, String> {
        let f = zip.by_name(path).map_err(|_| format!("missing '{path}' in archive"))?;
        if f.size() > MAX_ENTRY_BYTES {
            return Err(format!("'{path}' is too large"));
        }
        let mut buf = Vec::with_capacity(f.size() as usize);
        f.take(MAX_ENTRY_BYTES).read_to_end(&mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    };
    let xml = String::from_utf8(read("stack.xml")?).map_err(|_| "stack.xml is not UTF-8".to_string())?;
    // image size
    let img_tag = xml.find("<image").map(|i| &xml[i..]).and_then(|s| s.find('>').map(|e| &s[..e])).ok_or("stack.xml has no <image>")?;
    let w: f32 = attr(img_tag, "w").and_then(|s| s.parse().ok()).ok_or("missing image width")?;
    let h: f32 = attr(img_tag, "h").and_then(|s| s.parse().ok()).ok_or("missing image height")?;
    let mut p = Project::new(name, w, h, fps, frames.max(1));
    // walk tags; track group visibility
    // per open <stack>: (hidden, opacity, composite-op). A nested stack's opacity multiplies into every
    // layer below it, a hidden stack hides them, and a non-normal composite-op of a stack is inherited by
    // layers that have none of their own. (Approximation: the stack is not flattened first, so overlapping
    // layers inside a translucent stack show through each other.)
    let mut stack_attrs: Vec<(bool, f32, Option<Blend>)> = Vec::new();
    let mut found: Vec<Layer> = Vec::new();
    let mut i = 0;
    while let Some(off) = xml[i..].find('<') {
        let start = i + off;
        let Some(end_rel) = xml[start..].find('>') else { break };
        let tag = &xml[start..start + end_rel + 1];
        i = start + end_rel + 1;
        if tag.starts_with("<stack") {
            let hid = attr(tag, "visibility").is_some_and(|v| v == "hidden");
            let op = attr(tag, "opacity").and_then(|s| s.parse::<f32>().ok()).filter(|v| v.is_finite()).unwrap_or(1.0).clamp(0.0, 1.0);
            let bl = attr(tag, "composite-op").map(|o| blend_from_ora(&o)).filter(|b| *b != Blend::Normal);
            if !tag.ends_with("/>") {
                stack_attrs.push((hid, op, bl));
            }
        } else if tag.starts_with("</stack") {
            stack_attrs.pop();
        } else if tag.starts_with("<layer") {
            let src = attr(tag, "src").ok_or("layer without src")?;
            let lname = attr(tag, "name").unwrap_or_else(|| format!("layer{}", found.len()));
            let (x, y) = (
                attr(tag, "x").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0),
                attr(tag, "y").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0),
            );
            let png = read(&src)?;
            crate::assets::check_png(&png).map_err(|e| format!("layer '{lname}': {e}"))?;
            let mut l = Layer::image_from_png(&lname, &png, Vec2::new(x, y));
            let own = attr(tag, "opacity").and_then(|s| s.parse::<f32>().ok()).filter(|v| v.is_finite()).unwrap_or(1.0f32).clamp(0.0, 1.0);
            l.opacity = stack_attrs.iter().fold(own, |o, s| o * s.1);
            l.visible = attr(tag, "visibility").is_none_or(|v| v != "hidden") && !stack_attrs.iter().any(|s| s.0);
            l.blend = match attr(tag, "composite-op").map(|o| blend_from_ora(&o)) {
                Some(b) if b != Blend::Normal => b,
                _ => stack_attrs.iter().rev().find_map(|s| s.2).unwrap_or(Blend::Normal),
            };
            found.push(l);
        }
    }
    if found.is_empty() {
        return Err("no layers found in stack.xml".into());
    }
    // ORA lists the top layer first; OSATURA draws the first layer first (at the back).
    found.reverse();
    // de-duplicate names so layer paths stay unique
    let mut seen = std::collections::HashMap::new();
    for l in &mut found {
        let n = seen.entry(l.name.clone()).or_insert(0usize);
        if *n > 0 {
            l.name = format!("{}_{}", l.name, n);
        }
        *n += 1;
    }
    p.layers = found;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;
    use std::io::Write;

    fn solid_png(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        encode_rgba_png(w, h, &rgba.repeat((w * h) as usize)).unwrap()
    }

    fn engine() -> Engine {
        let mut p = demo::project();
        p.frames = 4;
        Engine::new(p).unwrap()
    }

    #[test]
    fn apng_has_animation_chunks_and_frame_count() {
        let b = apng_bytes(&engine(), 0.25).unwrap();
        assert_eq!(&b[..8], b"\x89PNG\r\n\x1a\n");
        let has = |tag: &[u8]| b.windows(4).any(|w| w == tag);
        assert!(has(b"acTL") && has(b"fcTL") && has(b"fdAT"));
        let dec = png::Decoder::new(Cursor::new(&b));
        let r = dec.read_info().unwrap();
        assert_eq!(r.info().animation_control().unwrap().num_frames, 4);
        assert_eq!(r.info().animation_control().unwrap().num_plays, 0, "loops forever");
    }

    #[test]
    fn spritesheet_layout_and_json() {
        let (img, j) = spritesheet(&engine(), 0.25, 2, "s.png").unwrap();
        let r = png::Decoder::new(Cursor::new(&img)).read_info().unwrap();
        let (fw, fh) = (j["frame_size"]["w"].as_u64().unwrap() as u32, j["frame_size"]["h"].as_u64().unwrap() as u32);
        assert_eq!((r.info().width, r.info().height), (fw * 2, fh * 2));
        assert_eq!(j["frames"].as_array().unwrap().len(), 4);
        assert_eq!(j["frames"][3]["x"].as_u64().unwrap() as u32, fw);
        assert_eq!(j["frames"][3]["y"].as_u64().unwrap() as u32, fh);
        assert!(spritesheet(&engine(), 40.0, 1, "x.png").is_err(), "oversize sheet refused");
    }

    #[test]
    fn oca_folder_is_written() {
        let scratch = crate::fsutil::ScratchDir::new("osatura-oca");
        let dir = scratch.join("oca");
        let mut p = demo::project();
        p.frames = 3;
        let path = write_oca(&p, Path::new("."), &dir, 0.25, false).unwrap();
        let j: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(j["frameRate"], p.fps);
        // C12e: width and height are integers (not 160.0)
        assert!(j["width"].is_u64() && j["height"].is_u64(), "{} {}", j["width"], j["height"]);
        assert_eq!(j["width"], (p.width * 0.25).round() as u64);
        assert_eq!(j["layers"][0]["frames"].as_array().unwrap().len(), 3);
        let first = j["layers"][0]["frames"][0]["fileName"].as_str().unwrap();
        assert!(dir.join(first).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn make_ora() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
            let o = zip::write::SimpleFileOptions::default();
            z.start_file("mimetype", o).unwrap();
            z.write_all(b"image/openraster").unwrap();
            z.start_file("stack.xml", o).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><image w="8" h="6"><stack>
                <layer name="top &amp; fx" src="data/top.png" x="2" y="1" opacity="0.5" composite-op="svg:multiply"/>
                <stack visibility="hidden"><layer name="in-hidden-group" src="data/mid.png"/></stack>
                <layer name="bg" src="data/bg.png" visibility="visible"/></stack></image>"#,
            )
            .unwrap();
            for (n, c) in [("data/top.png", [255, 0, 0, 255]), ("data/mid.png", [0, 255, 0, 255]), ("data/bg.png", [0, 0, 255, 255])] {
                z.start_file(n, o).unwrap();
                z.write_all(&solid_png(4, 3, c)).unwrap();
            }
            z.finish().unwrap();
        }
        buf
    }

    #[test]
    fn ora_import_builds_a_valid_project() {
        let p = import_ora(&make_ora(), "ora", 24.0, 1).unwrap();
        assert_eq!((p.width, p.height), (8.0, 6.0));
        let names: Vec<_> = p.layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["bg", "in-hidden-group", "top & fx"], "bottom-to-top");
        assert!(!p.layers[1].visible, "layer inside a hidden group is hidden");
        assert_eq!(p.layers[2].opacity, 0.5);
        assert_eq!(p.layers[2].blend, Blend::Multiply);
        let eng = Engine::new(p).expect("imported project validates");
        let (w, h, rgba) = render_rgba(&eng, 0.0, &RasterOptions::default());
        assert_eq!((w, h), (8, 6));
        let px = |x: usize, y: usize| &rgba[(y * 8 + x) * 4..(y * 8 + x) * 4 + 4];
        assert_eq!(px(0, 0), [0, 0, 255, 255], "background layer visible where nothing is above");
    }

    /// C8: nested <stack opacity> multiplies into the layers below it (it used to be dropped).
    #[test]
    fn ora_nested_stack_opacity_and_composite_op_are_inherited() {
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
            let o = zip::write::SimpleFileOptions::default();
            z.start_file("mimetype", o).unwrap();
            z.write_all(b"image/openraster").unwrap();
            z.start_file("stack.xml", o).unwrap();
            z.write_all(
                br#"<image w="8" h="6"><stack opacity="0.8"><stack opacity="0.5" composite-op="svg:multiply">
                <layer name="a" src="data/a.png" opacity="0.5"/><layer name="b" src="data/a.png" composite-op="svg:screen"/></stack>
                <layer name="c" src="data/a.png"/></stack></image>"#,
            )
            .unwrap();
            z.start_file("data/a.png", o).unwrap();
            z.write_all(&solid_png(4, 3, [255, 0, 0, 255])).unwrap();
            z.finish().unwrap();
        }
        let p = import_ora(&buf, "n", 24.0, 1).unwrap();
        let by = |n: &str| p.layers.iter().find(|l| l.name == n).unwrap();
        assert!((by("a").opacity - 0.5 * 0.5 * 0.8).abs() < 1e-6, "{}", by("a").opacity);
        assert!((by("b").opacity - 0.5 * 0.8).abs() < 1e-6);
        assert!((by("c").opacity - 0.8).abs() < 1e-6);
        assert_eq!(by("a").blend, Blend::Multiply, "inherited from the stack");
        assert_eq!(by("b").blend, Blend::Screen, "own op wins");
        assert_eq!(by("c").blend, Blend::Normal);
    }

    #[test]
    fn ora_import_rejects_bad_input() {
        assert!(import_ora(b"not a zip", "x", 24.0, 1).is_err());
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
            z.start_file("stack.xml", zip::write::SimpleFileOptions::default()).unwrap();
            z.write_all(b"<image w=\"4\" h=\"4\"><stack></stack></image>").unwrap();
            z.finish().unwrap();
        }
        assert!(import_ora(&buf, "x", 24.0, 1).unwrap_err().contains("no layers"));
    }

    #[test]
    fn dotlottie_is_a_zip_with_manifest_and_animation() {
        let b = dotlottie_bytes(&engine()).unwrap();
        let mut z = zip::ZipArchive::new(Cursor::new(b)).unwrap();
        let mut m = String::new();
        z.by_name("manifest.json").unwrap().read_to_string(&mut m).unwrap();
        let mv: Value = serde_json::from_str(&m).unwrap();
        assert_eq!(mv["version"], "2", "dotLottie 2.0 manifest");
        assert_eq!(mv["animations"][0], json!({"id": "animation"}), "v2 entries carry no autoplay/loop");
        let mut a = String::new();
        z.by_name("a/animation.json").unwrap().read_to_string(&mut a).unwrap();
        let v: Value = serde_json::from_str(&a).unwrap();
        assert!(v["layers"].is_array(), "valid Lottie inside");
    }

    #[test]
    fn attr_parser() {
        assert_eq!(attr(r#"<layer name="a &lt; b" x='3'/>"#, "name").as_deref(), Some("a < b"));
        assert_eq!(attr(r#"<layer name="a" x='3'/>"#, "x").as_deref(), Some("3"));
        assert_eq!(attr(r#"<layer name="a"/>"#, "y"), None);
    }
}
