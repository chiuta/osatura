// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Audio-driven lip-sync. This is an amplitude / zero-crossing heuristic, **not** phoneme
//! recognition: it picks one of five mouth shapes per video frame from loudness and
//! spectral "brightness", then cleans up flicker.

use crate::project::Channel;
use std::io::{Read, Seek};

/// Mouth shapes, in the order a switch layer's children should be drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Viseme {
    /// Closed / silence.
    Rest = 0,
    /// Slightly open (soft consonants, quiet vowels).
    Small = 1,
    /// Wide open (loud open vowels: "ah").
    Open = 2,
    /// Rounded ("oo", "oh").
    Round = 3,
    /// Wide / teeth (bright sounds: "ee", "s").
    Wide = 4,
}

/// Mono samples in −1..1 and the sample rate.
pub struct Audio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl Audio {
    pub fn duration(&self) -> f32 {
        self.samples.len() as f32 / self.sample_rate.max(1) as f32
    }
}

/// Decode a WAV (8/16/24/32-bit int or 32-bit float, any channel count → mono mix).
pub fn read_wav<R: Read + Seek>(r: R) -> Result<Audio, String> {
    let mut rd = hound::WavReader::new(r).map_err(|e| format!("invalid WAV: {e}"))?;
    let spec = rd.spec();
    let ch = spec.channels.max(1) as usize;
    let raw: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Float, _) => rd.samples::<f32>().collect::<Result<_, _>>().map_err(|e| e.to_string())?,
        (hound::SampleFormat::Int, b) if b <= 32 => {
            let scale = 1.0 / (1i64 << (b - 1)) as f32;
            rd.samples::<i32>().map(|s| s.map(|v| v as f32 * scale)).collect::<Result<_, _>>().map_err(|e| e.to_string())?
        }
        _ => return Err("unsupported WAV sample format".into()),
    };
    let samples = raw.chunks(ch).map(|c| c.iter().sum::<f32>() / ch as f32).collect();
    Ok(Audio { samples, sample_rate: spec.sample_rate })
}

pub fn read_wav_file(path: &str) -> Result<Audio, String> {
    crate::assets::check_regular(std::path::Path::new(path), crate::assets::MAX_AUDIO_BYTES, "WAV file")?;
    let f = std::fs::File::open(path).map_err(|e| format!("cannot open {path}: {e}"))?;
    read_wav(std::io::BufReader::new(f))
}

/// Per-frame loudness (RMS, 0..1 relative to the loudest frame) and zero-crossing rate.
pub fn frame_features(a: &Audio, fps: f32, frames: u32) -> Vec<(f32, f32)> {
    let spf = a.sample_rate as f32 / fps;
    let mut raw: Vec<(f32, f32)> = (0..frames)
        .map(|f| {
            // window centred on the frame, 1.5 frames wide
            let c = (f as f32 + 0.5) * spf;
            let (s, e) = (((c - spf * 0.75).max(0.0)) as usize, ((c + spf * 0.75) as usize).min(a.samples.len()));
            if e <= s + 1 {
                return (0.0, 0.0);
            }
            let w = &a.samples[s..e];
            let rms = (w.iter().map(|x| x * x).sum::<f32>() / w.len() as f32).sqrt();
            let zc = w.windows(2).filter(|p| (p[0] >= 0.0) != (p[1] >= 0.0)).count() as f32 / w.len() as f32;
            (rms, zc)
        })
        .collect();
    let peak = raw.iter().map(|x| x.0).fold(0.0f32, f32::max);
    if peak > 1e-6 {
        for x in &mut raw {
            x.0 /= peak;
        }
    }
    raw
}

/// Classify each video frame into a viseme. `gate` is the relative loudness below which the
/// mouth is closed (0.08 is a good default).
pub fn visemes(a: &Audio, fps: f32, frames: u32, gate: f32) -> Vec<Viseme> {
    let feats = frame_features(a, fps, frames);
    // Zero-crossing rate scales with pitch/brightness: rate ≈ 2·f/sr for a tone.
    let hz = |zc: f32| zc * a.sample_rate as f32 * 0.5;
    let raw: Vec<Viseme> = feats
        .iter()
        .map(|&(level, zc)| {
            if level < gate {
                Viseme::Rest
            } else if hz(zc) > 2500.0 {
                Viseme::Wide
            } else if hz(zc) < 450.0 && level > 0.25 {
                Viseme::Round
            } else if level > 0.55 {
                Viseme::Open
            } else {
                Viseme::Small
            }
        })
        .collect();
    smooth(&raw)
}

/// Remove one-frame blips (a shape that lasts a single frame between two equal neighbours
/// takes the neighbours' shape) so the mouth doesn't flutter.
pub fn smooth(v: &[Viseme]) -> Vec<Viseme> {
    let mut o = v.to_vec();
    for i in 1..o.len().saturating_sub(1) {
        if o[i - 1] == o[i + 1] && o[i] != o[i - 1] {
            o[i] = o[i - 1];
        }
    }
    o
}

/// Run-length encode into `(frame, child index)` exposures for a switch layer.
pub fn exposures(v: &[Viseme], mapping: &[usize; 5]) -> Vec<(f32, usize)> {
    let mut out: Vec<(f32, usize)> = Vec::new();
    for (f, &s) in v.iter().enumerate() {
        let idx = mapping[s as usize];
        if out.last().map(|l| l.1) != Some(idx) {
            out.push((f as f32, idx));
        }
    }
    out
}

/// Number of children of the switch layer `layer` (`None` if there is no such switch layer).
pub fn switch_children(p: &crate::project::Project, layer: &str) -> Option<usize> {
    match &p.find_layer(layer)?.kind {
        crate::project::LayerKind::Selector { layers, .. } => Some(layers.len()),
        _ => None,
    }
}

/// Default viseme -> child mapping `[rest, small, open, round, wide]` for a mouth switch with `n` children, in this
/// child order: 1 child: always it; 2: closed, open; 3: closed, open, round-or-wide; 4: closed, small, open,
/// round-or-wide; 5 or more: rest, small, open, round, wide (extra children are never used). Every index is `< n`.
pub fn default_lipsync_map(n: usize) -> [usize; 5] {
    match n {
        0 | 1 => [0; 5],
        2 => [0, 0, 1, 1, 0],
        3 => [0, 0, 1, 2, 2],
        4 => [0, 1, 2, 3, 3],
        _ => [0, 1, 2, 3, 4],
    }
}

/// A ready-made channel: drives the switch layer `target` with the lip-sync shapes.
pub fn lipsync_channel(target: &str, a: &Audio, fps: f32, frames: u32, mapping: &[usize; 5]) -> Channel {
    Channel::exposure(target, &exposures(&visemes(a, fps, frames, 0.08), mapping))
}

/// [`lipsync_channel`] with the checks the `lipsync` command needs: `layer` must be a switch layer, every
/// `--map` index must be a child of it (an out-of-range exposure would show nothing), and silent audio
/// (peak below -80 dBFS, as `lipsync-text` refuses it too) is an error instead of a channel that never moves.
pub fn lipsync_checked(p: &crate::project::Project, layer: &str, a: &Audio, mapping: &[usize; 5]) -> Result<Channel, String> {
    let l = p.find_layer(layer).ok_or_else(|| format!("no layer '{layer}'"))?;
    let n = match &l.kind {
        crate::project::LayerKind::Selector { layers, .. } => layers.len(),
        _ => return Err(format!("layer '{layer}' is not a switch layer")),
    };
    if let Some(bad) = mapping.iter().find(|&&m| m >= n) {
        return Err(format!("--map index {bad} is out of range: '{layer}' has {n} children"));
    }
    let peak = a.samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if a.samples.is_empty() || peak < 1e-4 {
        return Err("audio is silent (peak below -80 dBFS)".into());
    }
    Ok(lipsync_channel(layer, a, p.fps, p.frames, mapping))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn wav(samples: &[f32], sr: u32, channels: u16) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let spec = hound::WavSpec { channels, sample_rate: sr, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
            let mut w = hound::WavWriter::new(&mut buf, spec).unwrap();
            for s in samples {
                w.write_sample((s.clamp(-1.0, 1.0) * 32767.0) as i16).unwrap();
            }
            w.finalize().unwrap();
        }
        buf.into_inner()
    }
    fn tone(hz: f32, amp: f32, secs: f32, sr: u32) -> Vec<f32> {
        (0..(secs * sr as f32) as usize).map(|i| amp * (std::f32::consts::TAU * hz * i as f32 / sr as f32).sin()).collect()
    }

    #[test]
    fn wav_round_trip_and_stereo_mix() {
        let a = read_wav(Cursor::new(wav(&tone(220.0, 0.5, 0.1, 8000), 8000, 1))).unwrap();
        assert_eq!(a.sample_rate, 8000);
        assert_eq!(a.samples.len(), 800);
        let st = read_wav(Cursor::new(wav(&[0.5, -0.5, 0.5, -0.5], 8000, 2))).unwrap();
        assert!(st.samples.iter().all(|s| s.abs() < 1e-3), "opposite channels cancel in the mono mix");
        assert!(read_wav(Cursor::new(vec![1, 2, 3])).is_err());
    }

    /// C12d: `lipsync` validates the layer, the --map indices and silent audio.
    #[test]
    fn default_map_fits_every_child_count() {
        for n in 1..9 {
            let m = default_lipsync_map(n);
            assert!(m.iter().all(|&i| i < n), "n={n}: {m:?}");
        }
        assert_eq!(default_lipsync_map(3), [0, 0, 1, 2, 2]);
        assert_eq!(default_lipsync_map(5), [0, 1, 2, 3, 4]);
    }

    #[test]
    fn lipsync_checked_validates_layer_map_and_audio() {
        use crate::project::{Layer, Project};
        let mut p = Project::new("l", 100.0, 100.0, 24.0, 24);
        let kids: Vec<Layer> = (0..3).map(|i| Layer::vector(&format!("m{i}"), vec![crate::path::VPath::rect("r", 0.0, 0.0, 5.0, 5.0)])).collect();
        p.layers = vec![Layer::switch("mouth", kids), Layer::vector("plain", vec![crate::path::VPath::rect("r", 0.0, 0.0, 5.0, 5.0)])];
        let loud = Audio { samples: tone(300.0, 0.8, 0.5, 16000), sample_rate: 16000 };
        assert!(lipsync_checked(&p, "mouth", &loud, &[0, 1, 2, 1, 0]).is_ok());
        assert!(lipsync_checked(&p, "mouth", &loud, &[0, 1, 2, 3, 4]).unwrap_err().contains("out of range"));
        assert!(lipsync_checked(&p, "plain", &loud, &[0; 5]).unwrap_err().contains("not a switch"));
        assert!(lipsync_checked(&p, "nope", &loud, &[0; 5]).is_err());
        let silent = Audio { samples: vec![0.0; 16000], sample_rate: 16000 };
        assert!(lipsync_checked(&p, "mouth", &silent, &[0, 1, 2, 1, 0]).unwrap_err().contains("silent"));
        assert!(lipsync_checked(&p, "mouth", &Audio { samples: vec![], sample_rate: 16000 }, &[0; 5]).is_err());
    }

    #[test]
    fn silence_is_rest() {
        let a = Audio { samples: vec![0.0; 16000], sample_rate: 16000 };
        assert!(visemes(&a, 24.0, 24, 0.08).iter().all(|v| *v == Viseme::Rest));
    }

    #[test]
    fn speech_like_bursts_open_the_mouth_and_gaps_close_it() {
        let sr = 16000;
        let mut s = tone(300.0, 0.0, 0.5, sr); // silence
        s.extend(tone(300.0, 0.8, 0.5, sr)); // low loud tone
        s.extend(tone(300.0, 0.0, 0.5, sr));
        let a = Audio { samples: s, sample_rate: sr };
        let v = visemes(&a, 24.0, 36, 0.08);
        assert_eq!(v[3], Viseme::Rest);
        assert!(v[18] == Viseme::Round || v[18] == Viseme::Open, "{:?}", v[18]);
        assert_eq!(v[33], Viseme::Rest);
    }

    #[test]
    fn bright_sounds_pick_the_wide_shape_and_loud_mid_picks_open() {
        let sr = 16000;
        let bright = Audio { samples: tone(4000.0, 0.7, 0.5, sr), sample_rate: sr };
        assert_eq!(visemes(&bright, 24.0, 12, 0.08)[6], Viseme::Wide);
        let mid = Audio { samples: tone(900.0, 0.9, 0.5, sr), sample_rate: sr };
        assert_eq!(visemes(&mid, 24.0, 12, 0.08)[6], Viseme::Open);
    }

    #[test]
    fn smoothing_removes_single_frame_flicker() {
        use Viseme::*;
        assert_eq!(smooth(&[Open, Rest, Open, Open]), vec![Open, Open, Open, Open]);
        assert_eq!(smooth(&[Open, Rest, Rest, Open]), vec![Open, Rest, Rest, Open], "two-frame holds are kept");
    }

    #[test]
    fn exposures_are_run_length_encoded_and_mapped() {
        use Viseme::*;
        let e = exposures(&[Rest, Rest, Open, Open, Rest], &[0, 1, 2, 3, 4]);
        assert_eq!(e, vec![(0.0, 0), (2.0, 2), (4.0, 0)]);
        let e = exposures(&[Rest, Small], &[7, 7, 0, 0, 0]);
        assert_eq!(e, vec![(0.0, 7)], "viseme→drawing mapping can merge shapes");
    }
}
