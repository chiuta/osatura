// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Audio tracks stored in the project as *references* (file path + start frame + gain, plus an
//! optional trim window and fades) and the ffmpeg arguments that mix them into an export. No audio
//! data is embedded or decoded here. Own design; the filters used (`atrim`, `asetpts`, `afade`,
//! `adelay`, `volume`, `amix`, `apad`) are ordinary ffmpeg options, and the graph builder is pure
//! (strings in, strings out) so it can be unit-tested without ffmpeg.

use crate::project::Project;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundTrack {
    pub name: String,
    /// Path, relative to the project file (or absolute).
    pub file: String,
    /// Frame at which the sound starts (>= 0).
    #[serde(default)]
    pub start: f32,
    /// Linear gain 0..4.
    #[serde(default = "one")]
    pub gain: f32,
    /// In point inside the audio file, seconds (>= 0).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub trim_in: f32,
    /// Out point inside the audio file, seconds (> trim_in); `None` = to the end of the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trim_out: Option<f32>,
    /// Fade-in length from the (trimmed) start, seconds.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fade_in: f32,
    /// Fade-out length ending at the (trimmed) end, seconds.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fade_out: f32,
}

fn one() -> f32 {
    1.0
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

impl SoundTrack {
    pub fn new(name: &str, file: &str, start: f32, gain: f32) -> SoundTrack {
        SoundTrack { name: name.into(), file: file.into(), start, gain, trim_in: 0.0, trim_out: None, fade_in: 0.0, fade_out: 0.0 }
    }

    /// Length of the clip after trimming, when known (`file_len` is the whole file's length).
    pub fn clip_len(&self, file_len: Option<f32>) -> Option<f32> {
        self.trim_out.or(file_len).map(|end| (end - self.trim_in).max(0.0))
    }
}

/// Check one track; the message names the track.
pub fn check_track(s: &SoundTrack) -> Result<(), String> {
    let n = &s.name;
    if s.name.trim().is_empty() || s.file.trim().is_empty() {
        return Err("sound needs a name and a file".into());
    }
    if !s.start.is_finite() || s.start < 0.0 || !s.gain.is_finite() || !(0.0..=4.0).contains(&s.gain) {
        return Err("start must be >= 0 and gain within 0..4".into());
    }
    if !s.trim_in.is_finite() || s.trim_in < 0.0 || s.trim_in > 86_400.0 {
        return Err(format!("sound '{n}': trim in must be within 0..86400 seconds"));
    }
    if let Some(o) = s.trim_out {
        if !o.is_finite() || o <= s.trim_in || o > 86_400.0 {
            return Err(format!("sound '{n}': trim out must be greater than trim in"));
        }
    }
    for (what, v) in [("fade in", s.fade_in), ("fade out", s.fade_out)] {
        if !v.is_finite() || !(0.0..=86_400.0).contains(&v) {
            return Err(format!("sound '{n}': {what} must be a number of seconds >= 0"));
        }
    }
    if let Some(len) = s.clip_len(None) {
        if s.fade_in + s.fade_out > len + 1e-4 {
            return Err(format!("sound '{n}': fade in + fade out ({}s) is longer than the trimmed clip ({len}s)", s.fade_in + s.fade_out));
        }
    }
    Ok(())
}

/// Every problem in the project's sound list (used by `Engine` validation).
pub fn validate(p: &Project) -> Vec<String> {
    let mut errs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for s in &p.sounds {
        if let Err(e) = check_track(s) {
            errs.push(e);
        }
        if !seen.insert(s.name.as_str()) {
            errs.push(format!("duplicate sound name '{}'", s.name));
        }
    }
    errs
}

pub fn add(p: &mut Project, name: &str, file: &str, start: f32, gain: f32) -> Result<(), String> {
    add_track(p, SoundTrack::new(name, file, start, gain))
}

/// Add or replace (by name) a fully specified, validated track.
pub fn add_track(p: &mut Project, t: SoundTrack) -> Result<(), String> {
    check_track(&t)?;
    match p.sounds.iter_mut().find(|s| s.name == t.name) {
        Some(s) => *s = t,
        None => p.sounds.push(t),
    }
    Ok(())
}

pub fn remove(p: &mut Project, name: &str) -> bool {
    let n = p.sounds.len();
    p.sounds.retain(|s| s.name != name);
    p.sounds.len() != n
}

/// ffmpeg pieces for muxing: `inputs` are `-protocol_whitelist file -i <file>` triples, `filter` is a `-filter_complex`
/// graph ending in `[aout]`.
#[derive(Debug)]
pub struct Mux {
    pub inputs: Vec<String>,
    pub filter: String,
}

fn fnum(v: f32) -> String {
    let s = format!("{:.4}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Filter chain for one track: `[idx:a]` -> trim -> fades -> (delay to `start`) -> gain -> `[label]`.
/// `placed = false` leaves out the start delay (clip-local audio, e.g. for a waveform).
/// `file_len` (seconds) is only needed for a fade-out when no trim-out is set.
pub fn track_chain(s: &SoundTrack, fps: f32, idx: usize, label: &str, file_len: Option<f32>, placed: bool) -> Result<String, String> {
    check_track(s)?;
    let mut f: Vec<String> = Vec::new();
    if s.trim_in > 0.0 || s.trim_out.is_some() {
        let mut t = format!("atrim=start={}", fnum(s.trim_in));
        if let Some(o) = s.trim_out {
            t += &format!(":end={}", fnum(o));
        }
        f.push(t);
        f.push("asetpts=PTS-STARTPTS".into());
    }
    if s.fade_in > 0.0 {
        f.push(format!("afade=t=in:st=0:d={}", fnum(s.fade_in)));
    }
    if s.fade_out > 0.0 {
        let len = s.clip_len(file_len).ok_or_else(|| format!("sound '{}': the length of '{}' is needed for a fade-out (set a trim out, or install ffprobe)", s.name, s.file))?;
        if s.fade_out > len + 1e-4 {
            return Err(format!("sound '{}': fade out ({}s) is longer than the clip ({}s)", s.name, s.fade_out, fnum(len)));
        }
        f.push(format!("afade=t=out:st={}:d={}", fnum((len - s.fade_out).max(0.0)), fnum(s.fade_out)));
    }
    if placed {
        let ms = (s.start / fps * 1000.0).round().max(0.0) as u64;
        f.push(format!("adelay={ms}|{ms}"));
        // Re-stamp the frames from the sample count: after atrim + adelay the lossy encoders (mp3,
        // vorbis, aac) otherwise lose the leading silence (found with ffmpeg 6.1, checked by decoding).
        f.push("asetpts=N/SR/TB".into());
    }
    f.push(format!("volume={}", s.gain));
    Ok(format!("[{idx}:a]{}[{label}]", f.join(",")))
}

/// The demuxer to force for a file whose extension names one of the formats the studio accepts (`.wav`, `.mp3`, `.ogg`).
/// Without it ffmpeg probes the content and will open any container it knows (playlists, `concat` scripts, image
/// sequences) under an audio name; with it a file that is not what its name says is refused. Other extensions keep
/// ffmpeg's own detection. Used by every ffmpeg/ffprobe call that reads a sound file.
pub fn forced_demuxer(path: &std::path::Path) -> Option<&'static str> {
    match path.extension().and_then(|x| x.to_str()).map(|x| x.to_ascii_lowercase()).as_deref() {
        Some("wav") => Some("wav"),
        Some("mp3") => Some("mp3"),
        Some("ogg") => Some("ogg"),
        _ => None,
    }
}

/// The whole mix as a pure function. `first_input` is the ffmpeg input number of the first audio
/// file (1 when a video input comes first, 0 for audio only). `file_lens[i]` is the length of
/// track i's file if known. `pad_to` (seconds) pads the result with silence to at least that length.
pub fn mux_plan(p: &Project, base_dir: &std::path::Path, first_input: usize, file_lens: &[Option<f32>], pad_to: Option<f32>) -> Result<Option<Mux>, String> {
    if p.sounds.is_empty() {
        return Ok(None);
    }
    let mut inputs = Vec::new();
    let mut parts = Vec::new();
    for (i, s) in p.sounds.iter().enumerate() {
        let path = resolve(base_dir, &s.file)?;
        // only the plain file protocol: a file name such as http://... or concat:... is never fetched
        inputs.push("-protocol_whitelist".to_string());
        inputs.push("file".to_string());
        if let Some(f) = forced_demuxer(&path) {
            inputs.push("-f".to_string());
            inputs.push(f.to_string());
        }
        inputs.push("-i".to_string());
        inputs.push(path.to_string_lossy().into_owned());
        parts.push(track_chain(s, p.fps, first_input + i, &format!("a{i}"), file_lens.get(i).copied().flatten(), true)?);
    }
    let n = p.sounds.len();
    let labels: String = (0..n).map(|i| format!("[a{i}]")).collect();
    let tail = match pad_to {
        Some(d) if d > 0.0 => format!("apad=whole_dur={}", fnum(d)),
        _ => "anull".to_string(),
    };
    let filter = if n == 1 {
        format!("{};[a0]{tail}[aout]", parts[0])
    } else {
        format!("{};{labels}amix=inputs={n}:normalize=0,{tail}[aout]", parts.join(";"))
    };
    Ok(Some(Mux { inputs, filter }))
}

/// Length of an audio file in seconds: WAV header through `hound`, anything else through `ffprobe`.
pub fn file_duration(path: &std::path::Path) -> Option<f32> {
    // regular files up to the audio cap only (a FIFO or /dev/zero would hang the probe)
    crate::assets::check_regular(path, crate::assets::MAX_AUDIO_BYTES, "sound file").ok()?;
    if let Ok(r) = hound::WavReader::open(path) {
        let sp = r.spec();
        return Some(r.duration() as f32 / sp.sample_rate.max(1) as f32);
    }
    // an absolute or ./-prefixed path never reads as a URL; only the file protocol is allowed
    let path = if path.is_absolute() || path.starts_with(".") { path.to_path_buf() } else { std::path::Path::new(".").join(path) };
    let out = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-protocol_whitelist", "file"])
        .args(forced_demuxer(&path).map(|f| ["-f", f]).unwrap_or(["-hide_banner", "-hide_banner"]))
        .args(["-show_entries", "format=duration", "-of", "default=nw=1:nk=1", "-i"])
        .arg(&path)
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse::<f32>().ok().filter(|d| d.is_finite() && *d >= 0.0)
}

/// Check that the file of a sound track exists, is a regular readable file within the audio cap and is audio:
/// a WAV header is read directly, anything else is probed with `ffprobe` when it is installed (without it the format
/// cannot be verified and only existence is checked).
pub fn check_file(base_dir: &std::path::Path, file: &str) -> Result<(), String> {
    let path = resolve(base_dir, file)?;
    crate::assets::check_regular(&path, crate::assets::MAX_AUDIO_BYTES, "sound file")?;
    std::fs::File::open(&path).map_err(|e| format!("cannot read sound file {}: {e}", path.display()))?;
    if hound::WavReader::open(&path).is_ok() {
        return Ok(());
    }
    let arg = if path.is_absolute() || path.starts_with(".") { path.clone() } else { std::path::Path::new(".").join(&path) };
    match std::process::Command::new("ffprobe")
        .args(["-v", "error", "-protocol_whitelist", "file"])
        .args(forced_demuxer(&path).map(|f| ["-f", f]).unwrap_or(["-hide_banner", "-hide_banner"]))
        .args(["-show_entries", "format=duration", "-of", "default=nw=1:nk=1", "-i"])
        .arg(&arg)
        .output()
    {
        Err(_) => Ok(()), // no ffprobe: existence only
        Ok(o) if o.status.success() && String::from_utf8_lossy(&o.stdout).trim().parse::<f32>().is_ok() => Ok(()),
        Ok(_) => Err(format!("{} is not an audio file that ffmpeg can read", path.display())),
    }
}

/// A track's file under the project-directory rule (see `assets::resolve`); an empty `base_dir` means `.`.
fn resolve(base_dir: &std::path::Path, file: &str) -> Result<std::path::PathBuf, String> {
    crate::assets::resolve(base_dir, file)
}

/// Like `mux_plan`, probing file lengths only for tracks that need one (fade-out without trim-out).
pub fn mux_args(p: &Project, base_dir: &std::path::Path, first_input: usize, pad_to: Option<f32>) -> Result<Option<Mux>, String> {
    let lens: Vec<Option<f32>> = p
        .sounds
        .iter()
        .map(|s| if s.fade_out > 0.0 && s.trim_out.is_none() { resolve(base_dir, &s.file).ok().and_then(|f| file_duration(&f)) } else { None })
        .collect();
    mux_plan(p, base_dir, first_input, &lens, pad_to)
}

/// Resolved path of a track's file.
pub fn track_path(base_dir: &std::path::Path, s: &SoundTrack) -> Result<std::path::PathBuf, String> {
    resolve(base_dir, &s.file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn check_file_flags_missing_and_non_audio_files() {
        let dir = crate::fsutil::private_temp_dir("osatura-snd-test").unwrap();
        let d = dir.path();
        assert!(check_file(d, "nope.wav").unwrap_err().contains("cannot read"));
        std::fs::write(d.join("junk.wav"), b"this is not audio at all").unwrap();
        // a text file named .wav is not WAV; with ffprobe installed it is refused, without it only existence is known
        let r = check_file(d, "junk.wav");
        if std::process::Command::new("ffprobe").arg("-version").output().is_ok() {
            assert!(r.unwrap_err().contains("not an audio file"));
        }
        let spec = hound::WavSpec { channels: 1, sample_rate: 8000, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
        let mut w = hound::WavWriter::create(d.join("ok.wav"), spec).unwrap();
        for _ in 0..80 {
            w.write_sample(0i16).unwrap();
        }
        w.finalize().unwrap();
        assert!(check_file(d, "ok.wav").is_ok());
    }

    #[test]
    fn add_replace_remove_and_validate() {
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 10);
        add(&mut p, "a", "x.wav", 0.0, 1.0).unwrap();
        add(&mut p, "a", "y.wav", 12.0, 0.5).unwrap();
        assert_eq!(p.sounds.len(), 1);
        assert_eq!(p.sounds[0].file, "y.wav");
        assert!(add(&mut p, "b", "z.wav", -1.0, 1.0).is_err());
        assert!(add(&mut p, "b", "z.wav", 0.0, 9.0).is_err());
        assert!(remove(&mut p, "a") && !remove(&mut p, "a"));
        let j = p.to_json_pretty();
        assert!(!j.contains("sounds"), "empty list is not serialised");
    }

    #[test]
    fn the_demuxer_is_forced_only_for_the_three_accepted_extensions() {
        let d = |f: &str| forced_demuxer(Path::new(f));
        assert_eq!(d("a.wav"), Some("wav"));
        assert_eq!(d("dir/B.WAV"), Some("wav"));
        assert_eq!(d("a.Mp3"), Some("mp3"));
        assert_eq!(d("a.ogg"), Some("ogg"));
        assert_eq!(d("a.flac"), None);
        assert_eq!(d("a.m3u"), None);
        assert_eq!(d("wav"), None);
        assert_eq!(d("a.wav.txt"), None);
    }

    #[cfg(unix)] // asserts POSIX path separators
    #[test]
    fn filter_graph() {
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 48);
        assert!(mux_plan(&p, Path::new("."), 1, &[], None).unwrap().is_none());
        add(&mut p, "a", "x.wav", 12.0, 0.5).unwrap();
        let m = mux_plan(&p, Path::new("/d"), 1, &[], None).unwrap().unwrap();
        assert_eq!(m.inputs, vec!["-protocol_whitelist", "file", "-f", "wav", "-i", "/d/x.wav"]);
        assert!(m.filter.contains("adelay=500|500") && m.filter.contains("volume=0.5") && m.filter.ends_with("[aout]"));
        add(&mut p, "b", "sub/y.wav", 0.0, 1.0).unwrap();
        let m = mux_plan(&p, Path::new("/d"), 1, &[], None).unwrap().unwrap();
        assert!(m.filter.contains("amix=inputs=2") && m.inputs[11] == "/d/sub/y.wav");
    }

    #[cfg(unix)] // asserts POSIX path separators
    #[test]
    fn files_outside_the_project_directory_are_refused_and_an_empty_base_never_yields_a_url() {
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 48);
        add(&mut p, "a", "/etc/passwd", 0.0, 1.0).unwrap();
        let e = mux_plan(&p, Path::new("/d"), 1, &[], None).unwrap_err();
        assert!(e.contains("absolute"), "{e}");
        p.sounds[0].file = "../x.wav".into();
        assert!(mux_plan(&p, Path::new("/d"), 1, &[], None).unwrap_err().contains("'..'"));
        // a project given as `p.json` has an empty parent: the URL must not reach ffmpeg as a bare string
        p.sounds[0].file = "http://example.invalid/a.wav".into();
        let m = mux_plan(&p, Path::new(""), 1, &[], None).unwrap().unwrap();
        let input = &m.inputs[5];
        assert!(input.starts_with("./"), "{input}");
        assert_eq!(&m.inputs[..2], ["-protocol_whitelist", "file"]);
    }

    #[test]
    fn trim_and_fades_build_the_expected_chain() {
        let mut t = SoundTrack::new("m", "m.wav", 24.0, 2.0);
        t.trim_in = 1.5;
        t.trim_out = Some(5.5);
        t.fade_in = 0.5;
        t.fade_out = 1.0;
        let c = track_chain(&t, 24.0, 1, "a0", None, true).unwrap();
        assert_eq!(c, "[1:a]atrim=start=1.5:end=5.5,asetpts=PTS-STARTPTS,afade=t=in:st=0:d=0.5,afade=t=out:st=3:d=1,adelay=1000|1000,asetpts=N/SR/TB,volume=2[a0]");
        let local = track_chain(&t, 24.0, 0, "w", None, false).unwrap();
        assert!(!local.contains("adelay") && local.starts_with("[0:a]atrim"));
        // no trim-out: fade-out needs the file length
        t.trim_out = None;
        assert!(track_chain(&t, 24.0, 1, "a0", None, true).unwrap_err().contains("length"));
        let c = track_chain(&t, 24.0, 1, "a0", Some(10.0), true).unwrap();
        assert!(c.contains("atrim=start=1.5,") && c.contains("afade=t=out:st=7.5:d=1"), "{c}");
        // untouched track keeps the plain chain
        let plain = track_chain(&SoundTrack::new("p", "p.wav", 0.0, 1.0), 24.0, 1, "a0", None, true).unwrap();
        assert_eq!(plain, "[1:a]adelay=0|0,asetpts=N/SR/TB,volume=1[a0]");
    }

    #[test]
    fn trim_fade_validation() {
        let mut t = SoundTrack::new("m", "m.wav", 0.0, 1.0);
        t.trim_in = -1.0;
        assert!(check_track(&t).is_err());
        t.trim_in = 2.0;
        t.trim_out = Some(2.0);
        assert!(check_track(&t).unwrap_err().contains("trim out"));
        t.trim_out = Some(3.0);
        t.fade_in = 0.8;
        t.fade_out = 0.8;
        assert!(check_track(&t).unwrap_err().contains("longer"));
        t.fade_out = 0.2;
        assert!(check_track(&t).is_ok());
        t.fade_in = f32::NAN;
        assert!(check_track(&t).is_err());
        t.fade_in = -0.1;
        assert!(check_track(&t).is_err());
        // project-level validation reports every bad track and duplicates
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 10);
        p.sounds = vec![t.clone(), SoundTrack::new("m", "x.wav", 0.0, 1.0)];
        assert_eq!(validate(&p).len(), 2);
        assert!(crate::engine::Engine::new(p).is_err());
        // add_track validates and round-trips through JSON
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 10);
        let mut g = SoundTrack::new("g", "g.wav", 0.0, 1.0);
        g.trim_in = 0.25;
        g.fade_out = 0.5;
        add_track(&mut p, g.clone()).unwrap();
        let q = Project::from_json(&p.to_json_pretty()).unwrap();
        assert_eq!(q.sounds[0], g);
        // old projects (no trim keys) still load
        let j = p.to_json_pretty().replace("\"trim_in\"", "\"x_trim_in\"");
        assert_eq!(Project::from_json(&j).unwrap().sounds[0].trim_in, 0.0);
    }

    #[test]
    fn mix_graph_pads_and_numbers_inputs_from_zero() {
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 48);
        add(&mut p, "a", "x.wav", 0.0, 1.0).unwrap();
        add(&mut p, "b", "y.wav", 24.0, 1.0).unwrap();
        let m = mux_plan(&p, Path::new("."), 0, &[], Some(2.0)).unwrap().unwrap();
        assert!(m.filter.starts_with("[0:a]") && m.filter.contains(";[1:a]") && m.filter.ends_with("apad=whole_dur=2[aout]"), "{}", m.filter);
    }
}
