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
    /// Constant stereo pan, -1 (left) .. 1 (right), equal-power law of `stereopan` (spec 03). `None` = no pan stage: the
    /// track goes through the ffmpeg graph exactly as before. Present = the file is first decoded and panned in the sample
    /// domain (`soundtrack::process_samples`), and the result replaces the file as the ffmpeg input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan: Option<f32>,
    /// Biquad filters applied in order to the whole file before the pan (spec 04). Empty = no filter stage (not written).
    /// Same sample-domain path as `pan`; tracks with neither keep the unchanged ffmpeg graph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<TrackFilter>,
}

/// One biquad filter of a track (canonical kind name, frequency, quality factor, gain for peaking and shelves).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrackFilter {
    /// `lowpass`, `highpass`, `bandpass`, `notch`, `allpass`, `peaking`, `lowshelf`, `highshelf`.
    pub kind: String,
    /// Corner / centre frequency, Hz (> 0; checked against the file's sample rate when the mix runs).
    pub freq: f32,
    #[serde(default = "default_q")]
    pub q: f32,
    /// dB, peaking and shelves only.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub gain_db: f32,
}

fn default_q() -> f32 {
    std::f32::consts::FRAC_1_SQRT_2
}

impl TrackFilter {
    /// `kind:freq[:q[:gain_db]]`, e.g. `lowpass:800`, `highshelf:6000:0.7:3`.
    pub fn parse(s: &str) -> Result<TrackFilter, String> {
        let bad = || format!("filter '{s}': write kind:freq[:q[:gain_db]], e.g. lowpass:800 or peaking:1000:2:6");
        let mut it = s.split(':');
        let kind = crate::biquad::FilterKind::parse(it.next().unwrap_or(""))?.name().to_string();
        let mut num = |what: &str, dflt: Option<f32>| -> Result<f32, String> {
            match it.next() {
                Some(t) => t.trim().parse::<f32>().map_err(|_| format!("filter '{s}': {what} must be a number")),
                None => dflt.ok_or_else(bad),
            }
        };
        let f = TrackFilter { kind, freq: num("frequency", None)?, q: num("q", Some(default_q()))?, gain_db: num("gain", Some(0.0))? };
        if it.next().is_some() {
            return Err(bad());
        }
        f.check()?;
        Ok(f)
    }

    /// Comma-separated list of filters (`none` or empty = no filters).
    pub fn parse_list(s: &str) -> Result<Vec<TrackFilter>, String> {
        if s.trim().is_empty() || s.trim() == "none" {
            return Ok(vec![]);
        }
        s.split(',').map(TrackFilter::parse).collect()
    }

    /// `kind:freq:q[:gain]` as `parse` reads it.
    pub fn spec(&self) -> String {
        if self.gain_db != 0.0 {
            format!("{}:{}:{}:{}", self.kind, self.freq, self.q, self.gain_db)
        } else {
            format!("{}:{}:{}", self.kind, self.freq, self.q)
        }
    }

    /// Checks that do not need the sample rate (the rest is checked by `biquad::check` when the mix runs).
    pub fn check(&self) -> Result<(), String> {
        crate::biquad::FilterKind::parse(&self.kind)?;
        if !self.freq.is_finite() || self.freq <= 0.0 || self.freq > 384_000.0 {
            return Err(format!("filter '{}': frequency must be within (0, 384000] Hz", self.kind));
        }
        if !self.q.is_finite() || self.q <= 0.0 || f64::from(self.q) > crate::biquad::MAX_Q {
            return Err(format!("filter '{}': q must be within (0, {}]", self.kind, crate::biquad::MAX_Q));
        }
        if !self.gain_db.is_finite() || self.gain_db.abs() > 100.0 {
            return Err(format!("filter '{}': gain must be within -100..100 dB", self.kind));
        }
        Ok(())
    }

    /// The filter as a `biquad` specification.
    pub fn to_biquad(&self) -> Result<crate::biquad::BiquadSpec, String> {
        self.check()?;
        Ok(crate::biquad::BiquadSpec { kind: crate::biquad::FilterKind::parse(&self.kind)?, f0: f64::from(self.freq), q: f64::from(self.q), gain_db: f64::from(self.gain_db) })
    }
}

fn one() -> f32 {
    1.0
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

impl SoundTrack {
    pub fn new(name: &str, file: &str, start: f32, gain: f32) -> SoundTrack {
        SoundTrack { name: name.into(), file: file.into(), start, gain, trim_in: 0.0, trim_out: None, fade_in: 0.0, fade_out: 0.0, pan: None, filters: vec![] }
    }

    /// Length of the clip after trimming, when known (`file_len` is the whole file's length).
    /// Does the track need the sample-domain path (pan and/or filters) before the ffmpeg graph?
    pub fn needs_sample_path(&self) -> bool {
        self.pan.is_some() || !self.filters.is_empty()
    }

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
    if let Some(p) = s.pan {
        if !p.is_finite() || !(-1.0..=1.0).contains(&p) {
            return Err(format!("sound '{n}': pan must be a number within -1 (left) .. 1 (right)"));
        }
    }
    for f in &s.filters {
        f.check().map_err(|e| format!("sound '{n}': {e}"))?;
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

/// The sample-domain stage of a track that has `filters` and/or `pan` (specs 04 and 03): `samples` are interleaved `channels`
/// of the decoded, untrimmed file at `fs` Hz. The filters run first, in order, each channel on its own (zero initial state, output
/// limited to -1..1 as `biquad::filter_interleaved` does); then the pan, which needs a mono or stereo file and makes a mono file
/// stereo. Returns the channel count and samples of the processed file. A track without the stage is returned unchanged.
pub fn process_samples(s: &SoundTrack, channels: usize, fs: f64, samples: &[f32]) -> Result<(usize, Vec<f32>), String> {
    let n = &s.name;
    let mut ch = channels;
    let mut buf: Vec<f32> = samples.to_vec();
    for f in &s.filters {
        let spec = f.to_biquad().map_err(|e| format!("sound '{n}': {e}"))?;
        crate::biquad::check(fs, &spec).map_err(|e| format!("sound '{n}': {e}"))?;
        buf = crate::biquad::filter_interleaved(&buf, ch, fs, &spec).map_err(|e| format!("sound '{n}': {e}"))?;
    }
    if let Some(p) = s.pan {
        buf = match ch {
            1 => {
                ch = 2;
                crate::stereopan::pan_mono(&buf, f64::from(p))?
            }
            2 => crate::stereopan::pan_stereo(&buf, f64::from(p))?,
            _ => return Err(format!("sound '{n}': pan needs a mono or stereo file, this one has {ch} channels")),
        };
    }
    Ok((ch, buf))
}

/// Read a WAV (any sample format `hound` reads) as interleaved `f32`: `(channels, sample rate, samples)`.
pub fn read_samples(path: &std::path::Path) -> Result<(usize, u32, Vec<f32>), String> {
    crate::assets::check_regular(path, crate::assets::MAX_AUDIO_BYTES, "sound file")?;
    let mut r = hound::WavReader::open(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let spec = r.spec();
    let v: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().collect::<Result<_, _>>(),
        hound::SampleFormat::Int => {
            let scale = 1.0 / (1u64 << (spec.bits_per_sample.clamp(1, 32) - 1)) as f32;
            r.samples::<i32>().map(|x| x.map(|x| x as f32 * scale)).collect::<Result<_, _>>()
        }
    }
    .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok((spec.channels as usize, spec.sample_rate, v))
}

/// Write interleaved `f32` samples as a 32-bit float WAV.
pub fn write_samples(path: &std::path::Path, channels: usize, rate: u32, samples: &[f32]) -> Result<(), String> {
    let ch = u16::try_from(channels).map_err(|_| "too many channels".to_string())?;
    let spec = hound::WavSpec { channels: ch, sample_rate: rate, bits_per_sample: 32, sample_format: hound::SampleFormat::Float };
    let mut w = hound::WavWriter::create(path, spec).map_err(|e| e.to_string())?;
    for &v in samples {
        w.write_sample(v).map_err(|e| e.to_string())?;
    }
    w.finalize().map_err(|e| e.to_string())
}

/// Input files that replace a track's own file in the ffmpeg inputs (index in `Project::sounds` -> path of a processed WAV).
pub type Overrides = std::collections::HashMap<usize, std::path::PathBuf>;

/// The whole mix as a pure function. `first_input` is the ffmpeg input number of the first audio
/// file (1 when a video input comes first, 0 for audio only). `file_lens[i]` is the length of
/// track i's file if known. `pad_to` (seconds) pads the result with silence to at least that length.
pub fn mux_plan(p: &Project, base_dir: &std::path::Path, first_input: usize, file_lens: &[Option<f32>], pad_to: Option<f32>) -> Result<Option<Mux>, String> {
    mux_plan_with(p, base_dir, first_input, file_lens, pad_to, &Overrides::new())
}

/// `mux_plan` where the tracks listed in `overrides` read their (already processed) WAV instead of their own file. The graph
/// of every track is built the same way; with no overrides the result is identical to `mux_plan`.
pub fn mux_plan_with(p: &Project, base_dir: &std::path::Path, first_input: usize, file_lens: &[Option<f32>], pad_to: Option<f32>, overrides: &Overrides) -> Result<Option<Mux>, String> {
    if p.sounds.is_empty() {
        return Ok(None);
    }
    let mut inputs = Vec::new();
    let mut parts = Vec::new();
    for (i, s) in p.sounds.iter().enumerate() {
        let path = match overrides.get(&i) {
            Some(o) => o.clone(),
            None => resolve(base_dir, &s.file)?,
        };
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
    mux_args_with(p, base_dir, first_input, pad_to, &Overrides::new())
}

/// `mux_args` with processed inputs for some tracks (see `mux_plan_with`).
pub fn mux_args_with(p: &Project, base_dir: &std::path::Path, first_input: usize, pad_to: Option<f32>, overrides: &Overrides) -> Result<Option<Mux>, String> {
    let lens: Vec<Option<f32>> = p
        .sounds
        .iter()
        .map(|s| if s.fade_out > 0.0 && s.trim_out.is_none() { resolve(base_dir, &s.file).ok().and_then(|f| file_duration(&f)) } else { None })
        .collect();
    mux_plan_with(p, base_dir, first_input, &lens, pad_to, overrides)
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

    #[test]
    fn float_wav_samples_round_trip() {
        let dir = crate::fsutil::private_temp_dir("osatura-snd-test").unwrap();
        let f = dir.path().join("t.wav");
        let v = vec![0.0f32, 0.25, -0.5, 1.0];
        write_samples(&f, 2, 8000, &v).unwrap();
        assert_eq!(read_samples(&f).unwrap(), (2, 8000, v));
        // 16-bit input is scaled to -1..1
        let spec = hound::WavSpec { channels: 1, sample_rate: 8000, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
        let mut w = hound::WavWriter::create(dir.path().join("i.wav"), spec).unwrap();
        w.write_sample(16384i16).unwrap();
        w.finalize().unwrap();
        assert_eq!(read_samples(&dir.path().join("i.wav")).unwrap(), (1, 8000, vec![0.5]));
    }

    #[test]
    fn a_project_without_pan_round_trips_byte_identically() {
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 10);
        let mut t = SoundTrack::new("a", "a.wav", 2.0, 0.5);
        t.fade_in = 0.25;
        add_track(&mut p, t).unwrap();
        let j = p.to_json_pretty();
        assert!(!j.contains("\"pan\""), "an absent pan is not written");
        let q = Project::from_json(&j).unwrap();
        assert_eq!(q.sounds[0].pan, None);
        assert_eq!(q.to_json_pretty(), j);
    }

    #[test]
    fn pan_is_validated_stored_and_round_trips() {
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 10);
        let mut t = SoundTrack::new("a", "a.wav", 0.0, 1.0);
        for bad in [1.5f32, -1.01, f32::NAN, f32::INFINITY] {
            t.pan = Some(bad);
            assert!(check_track(&t).unwrap_err().contains("pan"), "{bad}");
        }
        t.pan = Some(-0.5);
        add_track(&mut p, t.clone()).unwrap();
        let j = p.to_json_pretty();
        assert!(j.contains("\"pan\": -0.5"));
        assert_eq!(Project::from_json(&j).unwrap().sounds[0], t);
        assert!(t.needs_sample_path() && !SoundTrack::new("b", "b.wav", 0.0, 1.0).needs_sample_path());
    }

    #[test]
    fn process_samples_pans_mono_to_stereo_and_stereo_in_place_and_refuses_more_channels() {
        let mut t = SoundTrack::new("a", "a.wav", 0.0, 1.0);
        // no stage: unchanged
        assert_eq!(process_samples(&t, 1, 8000.0, &[0.5, -0.5]).unwrap(), (1, vec![0.5, -0.5]));
        t.pan = Some(-1.0);
        let (ch, v) = process_samples(&t, 1, 8000.0, &[1.0]).unwrap();
        assert_eq!(ch, 2);
        assert!((v[0] - 1.0).abs() < 1e-6 && v[1].abs() < 1e-6, "{v:?}");
        t.pan = Some(1.0);
        let (ch, v) = process_samples(&t, 2, 8000.0, &[0.5, 0.25]).unwrap();
        assert_eq!(ch, 2);
        assert!(v[0].abs() < 1e-6 && (v[1] - 0.75).abs() < 1e-6, "{v:?}");
        assert!(process_samples(&t, 6, 8000.0, &[0.0; 6]).unwrap_err().contains("6 channels"));
    }

    #[cfg(unix)]
    #[test]
    fn overrides_replace_only_the_input_path_and_leave_the_graph_alone() {
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 48);
        add(&mut p, "a", "x.wav", 12.0, 0.5).unwrap();
        add(&mut p, "b", "y.wav", 0.0, 1.0).unwrap();
        let plain = mux_plan(&p, Path::new("/d"), 1, &[], None).unwrap().unwrap();
        let same = mux_plan_with(&p, Path::new("/d"), 1, &[], None, &Overrides::new()).unwrap().unwrap();
        assert_eq!((plain.inputs.clone(), plain.filter.clone()), (same.inputs, same.filter));
        let mut o = Overrides::new();
        o.insert(1, std::path::PathBuf::from("/tmp/x/t1.wav"));
        let m = mux_plan_with(&p, Path::new("/d"), 1, &[], None, &o).unwrap().unwrap();
        assert_eq!(m.filter, plain.filter);
        assert_eq!(m.inputs[5], "/d/x.wav");
        assert_eq!(m.inputs[11], "/tmp/x/t1.wav");
    }

    #[test]
    fn filters_parse_validate_and_round_trip_and_absent_filters_write_nothing() {
        let f = TrackFilter::parse("LowPass:800").unwrap();
        assert_eq!((f.kind.as_str(), f.freq, f.gain_db), ("lowpass", 800.0, 0.0));
        assert!((f.q - 0.70710677).abs() < 1e-6);
        let g = TrackFilter::parse("highshelf:6000:0.7:3").unwrap();
        assert_eq!(TrackFilter::parse(&g.spec()).unwrap(), g);
        for bad in ["", "lowpass", "lowpass:x", "lowpass:-5", "lowpass:100:0", "lowpass:100:5000", "wobble:100", "peaking:100:1:2:9", "peaking:100:1:500"] {
            assert!(TrackFilter::parse(bad).is_err(), "{bad}");
        }
        assert_eq!(TrackFilter::parse_list("none").unwrap(), vec![]);
        assert_eq!(TrackFilter::parse_list("lowpass:800,notch:50:10").unwrap().len(), 2);
        let mut p = Project::new("s", 10.0, 10.0, 24.0, 10);
        add(&mut p, "a", "a.wav", 0.0, 1.0).unwrap();
        assert!(!p.to_json_pretty().contains("filters"));
        let mut t = p.sounds[0].clone();
        t.filters = TrackFilter::parse_list("lowpass:800,peaking:1000:2:6").unwrap();
        add_track(&mut p, t.clone()).unwrap();
        let j = p.to_json_pretty();
        assert!(j.contains("\"filters\"") && j.contains("\"gain_db\": 6.0") && !j.contains("\"gain_db\": 0"));
        assert_eq!(Project::from_json(&j).unwrap().sounds[0], t);
        t.filters[0].freq = f32::NAN;
        assert!(check_track(&t).unwrap_err().contains("sound 'a'"));
        assert!(t.needs_sample_path());
    }

    #[test]
    fn filters_run_before_the_pan_and_use_the_sample_rate() {
        let fs = 8000.0f64;
        let tone = |hz: f64| -> Vec<f32> { (0..8000).map(|n| (0.5 * (2.0 * std::f64::consts::PI * hz * n as f64 / fs).sin()) as f32).collect() };
        let peak = |v: &[f32]| v[4000..].iter().fold(0.0f32, |m, x| m.max(x.abs()));
        let mut t = SoundTrack::new("a", "a.wav", 0.0, 1.0);
        t.filters = vec![TrackFilter::parse("lowpass:400").unwrap()];
        let (ch, low) = process_samples(&t, 1, fs, &tone(100.0)).unwrap();
        let (_, high) = process_samples(&t, 1, fs, &tone(3000.0)).unwrap();
        assert_eq!(ch, 1, "filters alone keep the channel count");
        assert!(peak(&low) > 0.45 && peak(&high) < 0.02, "{} {}", peak(&low), peak(&high));
        // filter then pan: mono -> stereo, the right channel stays empty at full left
        t.pan = Some(-1.0);
        let (ch, v) = process_samples(&t, 1, fs, &tone(100.0)).unwrap();
        assert_eq!(ch, 2);
        assert!(v.iter().skip(1).step_by(2).all(|x| x.abs() < 1e-6) && v.iter().step_by(2).fold(0.0f32, |m, x| m.max(x.abs())) > 0.45);
        // a frequency at or above half the sample rate is refused when the rate is known
        t.filters = vec![TrackFilter::parse("lowpass:5000").unwrap()];
        assert!(process_samples(&t, 1, fs, &tone(100.0)).unwrap_err().contains("half the sample rate"));
        // interleaved stereo: channels filtered separately
        t.pan = None;
        t.filters = vec![TrackFilter::parse("lowpass:400").unwrap()];
        let st: Vec<f32> = tone(100.0).iter().flat_map(|&x| [x, 0.0]).collect();
        let (ch, v) = process_samples(&t, 2, fs, &st).unwrap();
        assert!(ch == 2 && v.iter().skip(1).step_by(2).all(|x| *x == 0.0));
    }
}
