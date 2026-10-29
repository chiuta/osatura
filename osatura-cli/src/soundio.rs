// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! ffmpeg-backed audio helpers for the CLI and studio: mixing the project's sound tracks to a file
//! and decoding one trimmed/faded track to waveform peaks. The filter graphs come from
//! `osatura_core::soundtrack` (pure, unit-tested); this module only runs ffmpeg.

use osatura_core::audio::read_wav_file;
use osatura_core::audiomap::waveform_peaks;
use osatura_core::project::Project;
use osatura_core::soundtrack;
use std::path::Path;
use std::process::Command;

fn run(cmd: &mut Command) -> Result<(), String> {
    let out = cmd.output().map_err(|e| format!("cannot run ffmpeg ({e}); install it to use audio"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!("ffmpeg failed: {}", String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("")))
    }
}

/// Codec arguments for a mix written as `ext` (lower case).
fn codec_for(ext: &str) -> Result<&'static [&'static str], String> {
    match ext {
        "wav" => Ok(&["-c:a", "pcm_s16le", "-ar", "44100"]),
        "ogg" => Ok(&["-c:a", "libvorbis", "-q:a", "4"]),
        "mp3" => Ok(&["-c:a", "libmp3lame", "-b:a", "96k", "-ar", "44100"]),
        other => Err(format!("unknown audio format '.{other}' (use .wav, .ogg or .mp3)")),
    }
}

/// Mix all sound tracks of the project (start, gain, trim, fades applied) into `out`, padded or cut
/// to the project length. Needs at least one sound track.
pub fn mix_to_file(p: &Project, base: &Path, out: &Path) -> Result<(), String> {
    let ext = out.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
    let codec = codec_for(&ext)?;
    let dur = p.frames as f32 / p.fps;
    for s in &p.sounds {
        osatura_core::assets::check_regular(&soundtrack::track_path(base, s)?, osatura_core::assets::MAX_AUDIO_BYTES, "sound file")?;
    }
    let mux = soundtrack::mux_args(p, base, 0, Some(dur))?.ok_or("the project has no sound tracks (see: sound add)")?;
    run(Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error"])
        .args(&mux.inputs)
        .args(["-filter_complex", &mux.filter, "-map", "[aout]", "-t", &format!("{dur}")])
        .args(codec)
        .arg(out))
}

/// The mix as bytes plus a MIME type (for embedding into the HTML player).
pub fn mix_bytes(p: &Project, base: &Path, ext: &str) -> Result<(&'static str, Vec<u8>), String> {
    let mime = match ext {
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        _ => "audio/wav",
    };
    let dir = osatura_core::fsutil::private_temp_dir("osatura-snd")?;
    let tmp = dir.join(&format!("mix.{ext}"));
    mix_to_file(p, base, &tmp).and_then(|_| std::fs::read(&tmp).map_err(|e| e.to_string())).map(|b| (mime, b))
}

/// Min/max peaks of track `idx` after trim, fades and gain (clip-local time, not delayed), plus the
/// clip length in seconds. Decoded as 8 kHz mono, so it is cheap for any file ffmpeg can read.
pub fn clip_peaks(p: &Project, base: &Path, idx: usize, buckets: usize) -> Result<(f32, Vec<(f32, f32)>), String> {
    let s = p.sounds.get(idx).ok_or("no such sound")?;
    let path = soundtrack::track_path(base, s)?;
    let file_len = if s.fade_out > 0.0 && s.trim_out.is_none() { soundtrack::file_duration(&path) } else { None };
    let chain = soundtrack::track_chain(s, p.fps, 0, "w", file_len, false)?;
    let dir = osatura_core::fsutil::private_temp_dir("osatura-snd")?;
    let tmp = dir.join("peaks.wav");
    let r = run(Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error", "-protocol_whitelist", "file"])
        .args(soundtrack::forced_demuxer(&path).map(|f| ["-f", f]).unwrap_or(["-hide_banner", "-hide_banner"]))
        .arg("-i")
        .arg(&path)
        .args(["-filter_complex", &chain, "-map", "[w]", "-ac", "1", "-ar", "8000", "-t", "3600", "-c:a", "pcm_s16le"])
        .arg(&tmp))
    .and_then(|_| read_wav_file(&tmp.to_string_lossy()));
    let a = r?;
    let peaks = waveform_peaks(&a, buckets)?;
    Ok((a.duration(), peaks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use osatura_core::soundtrack::SoundTrack;

    fn have_ffmpeg() -> bool {
        Command::new("ffmpeg").arg("-version").output().map(|o| o.status.success()).unwrap_or(false)
    }

    /// 16-bit mono 8 kHz WAV: 1 s at amplitude 0.2 then 1 s at 0.8 then 1 s at 0.2, constant DC-free square.
    fn steps_wav(path: &Path) {
        let sr = 8000u32;
        let mut pcm: Vec<u8> = Vec::new();
        for (sec, amp) in [(0, 0.2f32), (1, 0.8), (2, 0.2)] {
            let _ = sec;
            for n in 0..sr {
                let v = if (n / 20) % 2 == 0 { amp } else { -amp };
                pcm.extend(((v * 32767.0) as i16).to_le_bytes());
            }
        }
        let mut w = Vec::new();
        w.extend(b"RIFF");
        w.extend((36 + pcm.len() as u32).to_le_bytes());
        w.extend(b"WAVEfmt ");
        w.extend(16u32.to_le_bytes());
        w.extend(1u16.to_le_bytes());
        w.extend(1u16.to_le_bytes());
        w.extend(sr.to_le_bytes());
        w.extend((sr * 2).to_le_bytes());
        w.extend(2u16.to_le_bytes());
        w.extend(16u16.to_le_bytes());
        w.extend(b"data");
        w.extend((pcm.len() as u32).to_le_bytes());
        w.extend(pcm);
        std::fs::write(path, w).unwrap();
    }

    fn level(a: &osatura_core::audio::Audio, from: f32, to: f32) -> f32 {
        let s = &a.samples[(from * a.sample_rate as f32) as usize..(to * a.sample_rate as f32) as usize];
        s.iter().fold(0.0f32, |m, v| m.max(v.abs()))
    }

    #[test]
    fn mix_applies_start_trim_fades_and_length() {
        if !have_ffmpeg() {
            return;
        }
        let scratch = osatura_core::fsutil::ScratchDir::new("osatura_soundio_test");
        let dir = scratch.to_path_buf();
        steps_wav(&dir.join("steps.wav"));
        // 4 s project at 10 fps; the 0.8-level second (file 1..2 s) is trimmed out and placed at 1 s,
        // with a 0.5 s fade-in and 0.5 s fade-out.
        let mut p = Project::new("t", 10.0, 10.0, 10.0, 40);
        let mut t = SoundTrack::new("s", "steps.wav", 10.0, 1.0);
        t.trim_in = 1.0;
        t.trim_out = Some(2.0);
        t.fade_in = 0.5;
        t.fade_out = 0.5;
        soundtrack::add_track(&mut p, t).unwrap();
        let out = dir.join("mix.wav");
        mix_to_file(&p, &dir, &out).unwrap();
        let a = read_wav_file(&out.to_string_lossy()).unwrap();
        assert!((a.duration() - 4.0).abs() < 0.02, "padded to the project length: {}", a.duration());
        assert!(level(&a, 0.0, 0.95) < 0.01, "silent before the start");
        assert!(level(&a, 1.0, 1.1) < 0.25, "fade-in begins near zero: {}", level(&a, 1.0, 1.1));
        assert!(level(&a, 1.4, 1.6) > 0.7, "trimmed loud second plays (not the 0.2 parts): {}", level(&a, 1.4, 1.6));
        assert!(level(&a, 1.9, 2.0) < 0.25, "fade-out ends near zero");
        assert!(level(&a, 2.05, 4.0) < 0.01, "nothing after the trimmed clip");
        // fade-out with no trim-out uses the file length (3 s): trim_in 2 -> a 1 s clip at 0.2 level
        let mut u = SoundTrack::new("u", "steps.wav", 0.0, 1.0);
        u.trim_in = 1.0;
        u.fade_out = 2.0;
        let mut p2 = Project::new("t", 10.0, 10.0, 10.0, 40);
        soundtrack::add_track(&mut p2, u).unwrap();
        mix_to_file(&p2, &dir, &out).unwrap();
        let b = read_wav_file(&out.to_string_lossy()).unwrap();
        assert!(level(&b, 0.0, 0.2) > 0.7 && level(&b, 1.8, 2.0) < 0.3, "fade-out over the last 2 s of the 2 s clip");
        // waveform of the processed clip: clip-local, so 1 s long and ramping up then down
        let (dur, peaks) = clip_peaks(&p, &dir, 0, 20).unwrap();
        assert!((dur - 1.0).abs() < 0.02 && peaks.len() == 20, "{dur}");
        let amp = |i: usize| peaks[i].1.max(-peaks[i].0);
        assert!(amp(0) < 0.25 && amp(10) > 0.7 && amp(19) < 0.25);
        // unsupported extension and empty project are errors, not panics
        assert!(mix_to_file(&p, &dir, &dir.join("x.flac")).is_err());
        assert!(mix_to_file(&Project::new("e", 1.0, 1.0, 10.0, 10), &dir, &out).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
