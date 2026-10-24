// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! WebM through ffmpeg (VP9 with alpha, constant quality) (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// WebM through ffmpeg (VP9 with alpha, constant quality)


pub fn ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg").arg("-version").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().is_ok_and(|s| s.success())
}

/// VP9 (libvpx) WebM of straight-RGBA frames at `fps`, odd sizes padded to even.
pub fn webm_encode<F: AsRef<[u8]>>(w: usize, h: usize, frames: &[F], fps: f32, crf: u8) -> Result<Vec<u8>, String> {
    let dir = crate::fsutil::private_temp_dir("osatura-fit")?;
    let out = dir.join("out.webm");
    let mut child = std::process::Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error", "-protocol_whitelist", "file,pipe,fd", "-f", "rawvideo", "-pix_fmt", "rgba", "-s", &format!("{w}x{h}"), "-framerate", &format!("{fps}"), "-i", "-"])
        .args(["-vf", "pad=ceil(iw/2)*2:ceil(ih/2)*2:color=0x00000000", "-c:v", "libvpx-vp9", "-pix_fmt", "yuva420p", "-b:v", "0", "-crf", &crf.to_string()])
        .args(["-auto-alt-ref", "0", "-deadline", "good", "-cpu-used", "2", "-threads", "1"])
        .arg(&out)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run ffmpeg ({e}); install it or choose another format"))?;
    {
        let mut si = child.stdin.take().ok_or("ffmpeg stdin")?;
        for f in frames {
            if si.write_all(f.as_ref()).is_err() {
                break;
            }
        }
    }
    let o = child.wait_with_output().map_err(|e| e.to_string())?;
    if o.status.success() { std::fs::read(&out).map_err(|e| e.to_string()) } else { Err(format!("ffmpeg failed: {}", String::from_utf8_lossy(&o.stderr).trim())) }
}

/// Decode a WebM to RGBA frames of `w x h` (cropping any padding).
pub fn webm_decode(bytes: &[u8], w: usize, h: usize) -> Result<Vec<Vec<u8>>, String> {
    let dir = crate::fsutil::private_temp_dir("osatura-fit")?;
    let inp = dir.join("in.webm");
    std::fs::write(&inp, bytes).map_err(|e| e.to_string())?;
    let (pw, ph) = (w + w % 2, h + h % 2);
    let o = std::process::Command::new("ffmpeg")
        .args(["-loglevel", "error", "-protocol_whitelist", "file", "-c:v", "libvpx-vp9", "-i"])
        .arg(&inp)
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-fps_mode", "passthrough", "-"])
        .output()
        .map_err(|e| format!("cannot run ffmpeg ({e})"));
    let o = o?;
    if !o.status.success() {
        return Err(format!("ffmpeg decode failed: {}", String::from_utf8_lossy(&o.stderr).trim()));
    }
    let fsz = pw * ph * 4;
    if fsz == 0 || o.stdout.len() % fsz != 0 {
        return Err("unexpected decoded size".into());
    }
    Ok(o.stdout
        .chunks_exact(fsz)
        .map(|f| {
            let mut v = Vec::with_capacity(w * h * 4);
            for y in 0..h {
                v.extend_from_slice(&f[y * pw * 4..y * pw * 4 + w * 4]);
            }
            v
        })
        .collect())
}

#[allow(dead_code)]
pub(super) fn _read_marker<R: Read>(_: R) {}
