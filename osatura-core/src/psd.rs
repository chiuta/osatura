// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Read the layers of a Photoshop document (`.psd`) as a one-frame project with one image layer per raster layer.
//!
//! Written from Adobe's published "Photoshop File Formats Specification" (the header, the layer records and the
//! channel image data sections) and from nothing else; no Adobe code or library is used. What is read:
//! 8-bit RGB documents (version 1, not the large `.psb`), raw and PackBits (RLE) channel data, per-layer offset,
//! opacity, hidden flag, blend mode, the Unicode layer name, and layer groups (flattened; a group's opacity and
//! hidden flag apply to the layers inside it). What is not: other bit depths and colour modes (CMYK, Lab, indexed,
//! 16/32-bit), ZIP-compressed channels, layer and vector masks, adjustment, text, smart-object and shape layers
//! (those carry no pixels of their own and are skipped, with their names listed in the returned notes), clipping
//! groups and layer effects. The flattened composite at the end of the file is ignored.

use crate::math::Vec2;
use crate::open_formats::encode_rgba_png;
use crate::project::{Layer, Project};
use crate::style::Blend;

/// Largest layer or document edge accepted, in pixels.
const MAX_EDGE: i64 = 16384;
/// Largest number of pixels in one layer.
const MAX_PIXELS: usize = 64 * 1024 * 1024;
/// Largest number of layer records.
const MAX_LAYERS: usize = 2048;

struct Rd<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.i.checked_add(n).filter(|&e| e <= self.b.len()).ok_or("the file ends too early")?;
        let s = &self.b[self.i..end];
        self.i = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        let s = self.take(2)?;
        Ok(u16::from_be_bytes([s[0], s[1]]))
    }
    fn i16(&mut self) -> Result<i16, String> {
        Ok(self.u16()? as i16)
    }
    fn u32(&mut self) -> Result<u32, String> {
        let s = self.take(4)?;
        Ok(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn i32(&mut self) -> Result<i32, String> {
        Ok(self.u32()? as i32)
    }
    fn skip(&mut self, n: usize) -> Result<(), String> {
        self.take(n).map(|_| ())
    }
    fn sub(&mut self, n: usize) -> Result<Rd<'a>, String> {
        Ok(Rd { b: self.take(n)?, i: 0 })
    }
    fn left(&self) -> usize {
        self.b.len() - self.i
    }
}

struct Chan {
    id: i16,
    len: usize,
}

struct Rec {
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
    chans: Vec<Chan>,
    blend: Blend,
    opacity: f32,
    hidden: bool,
    name: String,
    /// `lsct` section divider type: 0 other, 1 open folder, 2 closed folder, 3 end marker.
    divider: u32,
}

fn blend_from_key(k: &[u8]) -> Blend {
    match k {
        b"mul " => Blend::Multiply,
        b"scrn" => Blend::Screen,
        b"over" => Blend::Overlay,
        b"dark" => Blend::Darken,
        b"lite" => Blend::Lighten,
        b"div " => Blend::ColorDodge,
        b"idiv" => Blend::ColorBurn,
        b"hLit" => Blend::HardLight,
        b"sLit" => Blend::SoftLight,
        b"diff" => Blend::Difference,
        b"smud" => Blend::Exclusion,
        b"lddg" => Blend::Add,
        _ => Blend::Normal,
    }
}

fn read_rec(r: &mut Rd) -> Result<Rec, String> {
    let (top, left, bottom, right) = (r.i32()?, r.i32()?, r.i32()?, r.i32()?);
    let n = r.u16()? as usize;
    if n > 56 {
        return Err("a layer claims too many channels".into());
    }
    let mut chans = Vec::with_capacity(n);
    for _ in 0..n {
        let id = r.i16()?;
        let len = r.u32()? as usize;
        chans.push(Chan { id, len });
    }
    if r.take(4)? != b"8BIM" {
        return Err("bad blend-mode signature in a layer record".into());
    }
    let key = r.take(4)?.to_vec();
    let opacity = r.u8()? as f32 / 255.0;
    let _clipping = r.u8()?;
    let flags = r.u8()?;
    r.skip(1)?;
    let extra_len = r.u32()? as usize;
    let mut ex = r.sub(extra_len)?;
    let mask = ex.u32()? as usize;
    ex.skip(mask)?;
    let ranges = ex.u32()? as usize;
    ex.skip(ranges)?;
    // Pascal-string name, padded to a multiple of 4 bytes (length byte included)
    let nl = ex.u8()? as usize;
    let nb = ex.take(nl)?.to_vec();
    let pad = (4 - (1 + nl) % 4) % 4;
    ex.skip(pad.min(ex.left()))?;
    let mut name = String::from_utf8_lossy(&nb).into_owned();
    let mut divider = 0;
    while ex.left() >= 12 {
        let sig = ex.take(4)?;
        if sig != b"8BIM" && sig != b"8B64" {
            break;
        }
        let k = ex.take(4)?.to_vec();
        let l = ex.u32()? as usize;
        let mut data = ex.sub(l.min(ex.left()))?;
        // blocks are padded to an even length
        if l % 2 == 1 && ex.left() > 0 {
            ex.skip(1)?;
        }
        match &k[..] {
            b"luni" => {
                // unicode name: u32 length in UTF-16 units, then the units
                if let Ok(units) = data.u32() {
                    let units = (units as usize).min(data.left() / 2);
                    let mut v = Vec::with_capacity(units);
                    for _ in 0..units {
                        v.push(data.u16()?);
                    }
                    let s = String::from_utf16_lossy(&v);
                    let s = s.trim_end_matches('\0').to_string();
                    if !s.is_empty() {
                        name = s;
                    }
                }
            }
            b"lsct" | b"lsdk" => {
                if let Ok(t) = data.u32() {
                    divider = t;
                }
            }
            _ => {}
        }
    }
    Ok(Rec { top, left, bottom, right, chans, blend: blend_from_key(&key), opacity, hidden: flags & 2 != 0, name, divider })
}

fn unpack_bits(src: &[u8], row_len: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(row_len);
    let mut i = 0;
    while out.len() < row_len {
        let n = *src.get(i).ok_or("a compressed row ends too early")? as i8;
        i += 1;
        if n >= 0 {
            let c = n as usize + 1;
            let s = src.get(i..i + c).ok_or("a compressed row ends too early")?;
            out.extend_from_slice(s);
            i += c;
        } else if n != -128 {
            let c = 1 - n as isize;
            let v = *src.get(i).ok_or("a compressed row ends too early")?;
            i += 1;
            out.extend(std::iter::repeat_n(v, c as usize));
        }
    }
    out.truncate(row_len);
    Ok(out)
}

/// Decode one channel's pixel data (`w × h` bytes).
fn read_channel(data: &[u8], w: usize, h: usize) -> Result<Vec<u8>, String> {
    let mut r = Rd { b: data, i: 0 };
    let comp = r.u16()?;
    match comp {
        0 => Ok(r.take(w * h)?.to_vec()),
        1 => {
            let mut lens = Vec::with_capacity(h);
            for _ in 0..h {
                lens.push(r.u16()? as usize);
            }
            let mut out = Vec::with_capacity(w * h);
            for l in lens {
                let row = r.take(l)?;
                out.extend(unpack_bits(row, w)?);
            }
            Ok(out)
        }
        2 | 3 => Err("ZIP-compressed channels are not supported (save the PSD with 'Maximize compatibility' or export as OpenRaster)".into()),
        c => Err(format!("unknown channel compression {c}")),
    }
}

/// Read a PSD. `notes` lists what was skipped (layers without pixels of their own).
pub fn import_psd(bytes: &[u8], name: &str, fps: f32, frames: u32) -> Result<(Project, Vec<String>), String> {
    let mut r = Rd { b: bytes, i: 0 };
    if r.take(4).map_err(|_| "not a PSD file")? != b"8BPS" {
        return Err("not a PSD file (missing the 8BPS signature)".into());
    }
    match r.u16()? {
        1 => {}
        2 => return Err("large-document (.psb) files are not supported".into()),
        v => return Err(format!("unknown PSD version {v}")),
    }
    r.skip(6)?;
    let _channels = r.u16()?;
    let (h, w) = (r.u32()? as i64, r.u32()? as i64);
    let depth = r.u16()?;
    let mode = r.u16()?;
    if !(1..=MAX_EDGE).contains(&w) || !(1..=MAX_EDGE).contains(&h) {
        return Err(format!("document size {w}×{h} is outside 1..{MAX_EDGE}"));
    }
    if depth != 8 {
        return Err(format!("only 8-bit documents are supported (this one is {depth}-bit)"));
    }
    if mode != 3 {
        return Err(format!("only RGB documents are supported (colour mode {mode}; convert to RGB in your paint tool)"));
    }
    let cm = r.u32()? as usize;
    r.skip(cm)?;
    let ir = r.u32()? as usize;
    r.skip(ir)?;
    let lm_len = r.u32()? as usize;
    let mut lm = r.sub(lm_len)?;
    let li_len = lm.u32()? as usize;
    if li_len == 0 {
        return Err("the file has no layers (only a flattened picture)".into());
    }
    let mut li = lm.sub(li_len)?;
    let count = li.i16()?;
    let count = (count as i32).unsigned_abs() as usize;
    if count == 0 || count > MAX_LAYERS {
        return Err(format!("{count} layers is outside 1..{MAX_LAYERS}"));
    }
    let mut recs = Vec::with_capacity(count);
    for _ in 0..count {
        recs.push(read_rec(&mut li)?);
    }
    let mut p = Project::new(name, w as f32, h as f32, fps, frames.max(1));
    let mut found: Vec<Layer> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    // open groups: index into `found` where the group's layers start
    let mut groups: Vec<usize> = Vec::new();
    for rec in &recs {
        // channel data follows the records, in record order, whether or not we use it
        let mut chan_data: Vec<&[u8]> = Vec::with_capacity(rec.chans.len());
        for c in &rec.chans {
            chan_data.push(li.take(c.len)?);
        }
        match rec.divider {
            3 => {
                groups.push(found.len());
                continue;
            }
            1 | 2 => {
                if let Some(start) = groups.pop() {
                    for l in &mut found[start..] {
                        l.opacity *= rec.opacity;
                        if rec.hidden {
                            l.visible = false;
                        }
                    }
                }
                continue;
            }
            _ => {}
        }
        let (lw, lh) = ((rec.right as i64 - rec.left as i64), (rec.bottom as i64 - rec.top as i64));
        if lw <= 0 || lh <= 0 {
            notes.push(format!("'{}' has no pixels (adjustment, text, shape or empty layer): skipped", rec.name));
            continue;
        }
        if lw > MAX_EDGE || lh > MAX_EDGE || (lw as usize).saturating_mul(lh as usize) > MAX_PIXELS {
            return Err(format!("layer '{}' is too large ({lw}×{lh})", rec.name));
        }
        let (lw, lh) = (lw as usize, lh as usize);
        let mut planes: [Option<Vec<u8>>; 4] = [None, None, None, None];
        for (c, d) in rec.chans.iter().zip(&chan_data) {
            let slot = match c.id {
                0 => 0,
                1 => 1,
                2 => 2,
                -1 => 3,
                _ => continue,
            };
            planes[slot] = Some(read_channel(d, lw, lh).map_err(|e| format!("layer '{}': {e}", rec.name))?);
        }
        if planes[0].is_none() || planes[1].is_none() || planes[2].is_none() {
            notes.push(format!("'{}' has no RGB channels: skipped", rec.name));
            continue;
        }
        let mut rgba = Vec::with_capacity(lw * lh * 4);
        for k in 0..lw * lh {
            rgba.push(planes[0].as_ref().map(|v| v[k]).unwrap_or(0));
            rgba.push(planes[1].as_ref().map(|v| v[k]).unwrap_or(0));
            rgba.push(planes[2].as_ref().map(|v| v[k]).unwrap_or(0));
            rgba.push(planes[3].as_ref().map(|v| v[k]).unwrap_or(255));
        }
        let png = encode_rgba_png(lw as u32, lh as u32, &rgba)?;
        crate::assets::check_png(&png).map_err(|e| format!("layer '{}': {e}", rec.name))?;
        let mut l = Layer::image_from_png(&rec.name, &png, Vec2::new(rec.left as f32, rec.top as f32));
        l.opacity = rec.opacity;
        l.visible = !rec.hidden;
        l.blend = rec.blend;
        found.push(l);
    }
    if found.is_empty() {
        return Err("no layer with pixels was found".into());
    }
    // unique names, as the ORA importer does
    let mut seen = std::collections::HashMap::new();
    for l in &mut found {
        let n = seen.entry(l.name.clone()).or_insert(0usize);
        if *n > 0 {
            l.name = format!("{}_{}", l.name, n);
        }
        *n += 1;
    }
    p.layers = found;
    Ok((p, notes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn be16(v: &mut Vec<u8>, x: u16) {
        v.extend(x.to_be_bytes());
    }
    fn be32(v: &mut Vec<u8>, x: u32) {
        v.extend(x.to_be_bytes());
    }

    struct L {
        name: &'static str,
        rect: [i32; 4],
        rgba: [u8; 4],
        key: &'static [u8; 4],
        hidden: bool,
        opacity: u8,
        rle: bool,
        divider: Option<u32>,
        alpha: bool,
    }

    fn lay(name: &'static str, rect: [i32; 4], rgba: [u8; 4]) -> L {
        L { name, rect, rgba, key: b"norm", hidden: false, opacity: 255, rle: false, divider: None, alpha: true }
    }

    fn channel(w: usize, h: usize, v: u8, rle: bool) -> Vec<u8> {
        let mut d = Vec::new();
        if !rle {
            be16(&mut d, 0);
            d.extend(std::iter::repeat_n(v, w * h));
        } else {
            be16(&mut d, 1);
            // each row: one repeat run (w ≤ 128) or two
            let mut rows = Vec::new();
            for _ in 0..h {
                let mut row = Vec::new();
                let mut left = w;
                while left > 0 {
                    let c = left.min(128);
                    row.push((1i16 - c as i16) as i8 as u8);
                    row.push(v);
                    left -= c;
                }
                rows.push(row);
            }
            for r in &rows {
                be16(&mut d, r.len() as u16);
            }
            for r in rows {
                d.extend(r);
            }
        }
        d
    }

    fn build(w: u32, h: u32, layers: &[L], depth: u16, mode: u16) -> Vec<u8> {
        let mut f = b"8BPS".to_vec();
        be16(&mut f, 1);
        f.extend([0u8; 6]);
        be16(&mut f, 3);
        be32(&mut f, h);
        be32(&mut f, w);
        be16(&mut f, depth);
        be16(&mut f, mode);
        be32(&mut f, 0);
        be32(&mut f, 0);
        let mut recs = Vec::new();
        let mut data = Vec::new();
        for l in layers {
            let (lw, lh) = ((l.rect[3] - l.rect[1]).max(0) as usize, (l.rect[2] - l.rect[0]).max(0) as usize);
            let mut chans: Vec<(i16, Vec<u8>)> = Vec::new();
            if l.divider.is_none() || lw > 0 {
                for (i, id) in [0i16, 1, 2].iter().enumerate() {
                    chans.push((*id, channel(lw, lh, l.rgba[i], l.rle)));
                }
                if l.alpha {
                    chans.push((-1, channel(lw, lh, l.rgba[3], l.rle)));
                }
            }
            for v in l.rect {
                be32(&mut recs, v as u32);
            }
            be16(&mut recs, chans.len() as u16);
            for (id, d) in &chans {
                be16(&mut recs, *id as u16);
                be32(&mut recs, d.len() as u32);
            }
            recs.extend(b"8BIM");
            recs.extend(l.key);
            recs.push(l.opacity);
            recs.push(0);
            recs.push(if l.hidden { 2 } else { 0 });
            recs.push(0);
            let mut ex = Vec::new();
            be32(&mut ex, 0);
            be32(&mut ex, 0);
            let nm = l.name.as_bytes();
            ex.push(nm.len() as u8);
            ex.extend(nm);
            while ex.len() % 4 != 0 {
                ex.push(0);
            }
            if let Some(t) = l.divider {
                ex.extend(b"8BIMlsct");
                be32(&mut ex, 4);
                be32(&mut ex, t);
            }
            be32(&mut recs, ex.len() as u32);
            recs.extend(ex);
            for (_, d) in chans {
                data.extend(d);
            }
        }
        let mut info = Vec::new();
        be16(&mut info, layers.len() as u16);
        info.extend(recs);
        info.extend(data);
        let mut lm = Vec::new();
        be32(&mut lm, info.len() as u32);
        lm.extend(info);
        be32(&mut f, lm.len() as u32);
        f.extend(lm);
        f
    }

    #[test]
    fn reads_layers_offsets_opacity_blend_hidden_and_both_compressions() {
        let mut a = lay("back", [0, 0, 4, 6], [10, 20, 30, 255]);
        a.rle = true;
        let mut b = lay("front", [2, 3, 5, 7], [200, 100, 50, 128]);
        b.key = b"mul ";
        b.opacity = 128;
        let mut c = lay("ghost", [0, 0, 1, 1], [1, 2, 3, 255]);
        c.hidden = true;
        c.alpha = false;
        let bytes = build(8, 8, &[a, b, c], 8, 3);
        let (p, notes) = import_psd(&bytes, "t", 24.0, 1).unwrap();
        assert!(notes.is_empty());
        assert_eq!((p.width, p.height), (8.0, 8.0));
        assert_eq!(p.layers.len(), 3);
        assert_eq!(p.layers[0].name, "back");
        assert_eq!(p.layers[1].name, "front");
        assert_eq!(p.layers[1].blend, Blend::Multiply);
        assert!((p.layers[1].opacity - 128.0 / 255.0).abs() < 1e-4);
        assert!(!p.layers[2].visible && p.layers[0].visible);
        if let crate::project::LayerKind::Image { origin, .. } = &p.layers[1].kind {
            assert_eq!((origin.x, origin.y), (3.0, 2.0));
        } else {
            panic!("image layer expected");
        }
        crate::engine::Engine::new(p).unwrap();
    }

    #[test]
    fn pixels_survive_the_round_trip() {
        let bytes = build(4, 4, &[lay("px", [0, 0, 2, 2], [11, 22, 33, 255])], 8, 3);
        let (p, _) = import_psd(&bytes, "t", 24.0, 1).unwrap();
        let crate::project::LayerKind::Image { source, .. } = &p.layers[0].kind else { panic!() };
        let png = crate::base64::decode(source.png_base64.as_ref().unwrap()).unwrap();
        let img = image_pixels(&png);
        assert_eq!(&img[..4], &[11, 22, 33, 255]);
    }

    fn image_pixels(png: &[u8]) -> Vec<u8> {
        let dec = png::Decoder::new(std::io::Cursor::new(png));
        let mut rd = dec.read_info().unwrap();
        let mut buf = vec![0; rd.output_buffer_size().unwrap()];
        rd.next_frame(&mut buf).unwrap();
        buf
    }

    #[test]
    fn groups_are_flattened_and_apply_opacity_and_hidden() {
        let mut end = lay("</Layer group>", [0, 0, 0, 0], [0, 0, 0, 0]);
        end.divider = Some(3);
        let inner = lay("inner", [0, 0, 2, 2], [9, 9, 9, 255]);
        let mut folder = lay("Group 1", [0, 0, 0, 0], [0, 0, 0, 0]);
        folder.divider = Some(1);
        folder.opacity = 128;
        folder.hidden = true;
        let outer = lay("outer", [0, 0, 2, 2], [1, 1, 1, 255]);
        let bytes = build(4, 4, &[end, inner, folder, outer], 8, 3);
        let (p, _) = import_psd(&bytes, "t", 24.0, 1).unwrap();
        assert_eq!(p.layers.len(), 2);
        assert!(!p.layers[0].visible && (p.layers[0].opacity - 128.0 / 255.0).abs() < 1e-4);
        assert!(p.layers[1].visible && p.layers[1].opacity > 0.99);
    }

    #[test]
    fn empty_layers_are_noted_and_duplicate_names_made_unique() {
        let txt = lay("Title", [0, 0, 0, 0], [0, 0, 0, 0]);
        let a = lay("same", [0, 0, 1, 1], [1, 1, 1, 255]);
        let b = lay("same", [0, 0, 1, 1], [2, 2, 2, 255]);
        let (p, notes) = import_psd(&build(2, 2, &[txt, a, b], 8, 3), "t", 24.0, 1).unwrap();
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("Title"));
        assert_eq!(p.layers[0].name, "same");
        assert_eq!(p.layers[1].name, "same_1");
    }

    #[test]
    fn unsupported_and_damaged_files_are_refused_with_a_reason() {
        let l = || lay("a", [0, 0, 1, 1], [1, 1, 1, 255]);
        assert!(import_psd(&build(2, 2, &[l()], 16, 3), "t", 24.0, 1).unwrap_err().contains("8-bit"));
        assert!(import_psd(&build(2, 2, &[l()], 8, 4), "t", 24.0, 1).unwrap_err().contains("RGB"));
        assert!(import_psd(b"not a psd", "t", 24.0, 1).is_err());
        assert!(import_psd(b"", "t", 24.0, 1).is_err());
        let mut psb = build(2, 2, &[l()], 8, 3);
        psb[5] = 2;
        assert!(import_psd(&psb, "t", 24.0, 1).unwrap_err().contains("psb"));
        // every truncation of a valid file is an error, never a panic
        let ok = build(3, 3, &[l(), lay("b", [0, 0, 2, 2], [5, 5, 5, 255])], 8, 3);
        for n in 0..ok.len() {
            let _ = import_psd(&ok[..n], "t", 24.0, 1);
        }
        // a layer claiming a huge rectangle is refused before anything is allocated
        let huge = build(2, 2, &[lay("h", [0, 0, 1, 20000], [1, 1, 1, 255])], 8, 3);
        assert!(import_psd(&huge, "t", 24.0, 1).is_err());
    }

    #[test]
    fn corrupt_bytes_never_panic() {
        let ok = build(3, 3, &[lay("a", [0, 0, 2, 2], [5, 5, 5, 255])], 8, 3);
        let mut seed = 12345u32;
        for _ in 0..400 {
            let mut b = ok.clone();
            for _ in 0..3 {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let i = (seed >> 8) as usize % b.len();
                b[i] = (seed >> 3) as u8;
            }
            let _ = import_psd(&b, "t", 24.0, 1);
        }
    }
}
