// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Transcript-driven lip-sync and speech-aware secondary motion.
//!
//! **What this is, honestly:** an *energy + text* aligner. There is no acoustic model and no
//! phoneme recognition. The pipeline is
//!
//! 1. voice-activity detection (short-time RMS + zero-crossing rate, adaptive threshold from
//!    the clip's own noise floor and peak, hysteresis, gap merging) -> voiced regions;
//! 2. the transcript is split into words and syllables with language-agnostic vowel-group
//!    heuristics (Romanian and English tweaks); punctuation marks expected pauses;
//! 3. a textbook segmental dynamic programme (Viterbi over a left-to-right chain of syllable
//!    and gap units) places every syllable on the 10 ms grid. Costs: unvoiced frames inside a
//!    syllable, voiced frames inside a gap, a Gaussian duration prior around the syllable's
//!    expected length (letter count x a speaking rate estimated from the audio itself), and a
//!    penalty for cutting between two syllables where the energy contour is high (so internal
//!    boundaries fall into energy dips);
//! 4. a rule-based grapheme-to-viseme table (Romanian incl. ăâîșț, English digraphs) spreads
//!    the five mouth shapes over each syllable, then a minimum-hold / coarticulation pass
//!    removes flicker.
//!
//! It cannot know *which* phoneme was really said, and it trusts the transcript. Speech that
//! differs from the text, music under the voice, or heavy noise will mis-place syllables.
//! Everything is deterministic (no clock, no global RNG).

use crate::anim::{Ease, Track};
use crate::audio::{exposures, Audio, Viseme};
use crate::project::{Channel, LayerKind, Mode, Project, Prop, Source};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// Language
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Ro,
    En,
}

impl Lang {
    pub fn parse(s: &str) -> Result<Lang, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ro" | "ron" | "rom" | "romanian" => Ok(Lang::Ro),
            "en" | "eng" | "english" => Ok(Lang::En),
            o => Err(format!("unknown language '{o}' (use ro or en)")),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Voice activity detection
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct VadOptions {
    /// Analysis step, seconds (also the alignment grid).
    pub hop: f32,
    /// RMS window length, seconds.
    pub window: f32,
    /// Level (0..1 between noise floor and peak) that starts speech.
    pub on: f32,
    /// Level below which speech ends (hysteresis; must be below `on`).
    pub off: f32,
    /// Zero-crossing rate that lets a weak (fricative-like) frame start speech.
    pub zcr_on: f32,
    /// Regions shorter than this are dropped, seconds.
    pub min_speech: f32,
    /// Gaps shorter than this between regions are bridged, seconds.
    pub min_gap: f32,
}

impl Default for VadOptions {
    fn default() -> Self {
        VadOptions { hop: 0.01, window: 0.02, on: 0.12, off: 0.06, zcr_on: 0.2, min_speech: 0.04, min_gap: 0.04 }
    }
}

#[derive(Debug)]
pub struct Vad {
    pub hop: f32,
    /// Level per hop, 0..1 relative to noise floor and peak.
    pub level: Vec<f32>,
    pub voiced: Vec<bool>,
    /// Voiced regions `(start, end)` in seconds.
    pub regions: Vec<(f32, f32)>,
}

fn check_vad(o: &VadOptions) -> Result<(), String> {
    let ok = |x: f32| x.is_finite() && x > 0.0;
    if !ok(o.hop) || !ok(o.window) || o.hop > 0.1 || o.window > 0.2 {
        return Err("VAD hop/window must be positive and small (hop <= 0.1 s, window <= 0.2 s)".into());
    }
    if !(o.on > 0.0 && o.on <= 1.0 && o.off > 0.0 && o.off < o.on) {
        return Err("VAD thresholds need 0 < off < on <= 1".into());
    }
    if !(o.min_speech >= 0.0 && o.min_gap >= 0.0 && o.min_speech.is_finite() && o.min_gap.is_finite() && o.zcr_on.is_finite()) {
        return Err("VAD min_speech/min_gap must be non-negative numbers".into());
    }
    Ok(())
}

/// Find voiced regions. Errors on empty, silent or featureless audio.
pub fn detect_speech(a: &Audio, o: &VadOptions) -> Result<Vad, String> {
    check_vad(o)?;
    if a.sample_rate == 0 || a.samples.is_empty() {
        return Err("audio is empty".into());
    }
    if a.samples.iter().any(|s| !s.is_finite()) {
        return Err("audio contains non-finite samples".into());
    }
    let sr = a.sample_rate as f32;
    let n = a.samples.len();
    let hops = ((n as f32 / sr) / o.hop).ceil().max(1.0) as usize;
    // prefix sums of squares and of zero crossings
    // remove any DC offset so a constant level is not mistaken for speech
    let mean = (a.samples.iter().map(|&x| x as f64).sum::<f64>() / n as f64) as f32;
    let mut sq = vec![0f64; n + 1];
    let mut zc = vec![0u32; n + 1];
    for i in 0..n {
        let s = (a.samples[i] - mean) as f64;
        sq[i + 1] = sq[i] + s * s;
        let c = i > 0 && (a.samples[i - 1] - mean >= 0.0) != (a.samples[i] - mean >= 0.0);
        zc[i + 1] = zc[i] + c as u32;
    }
    let half = o.window * 0.5;
    let mut rms = vec![0f32; hops];
    let mut zcr = vec![0f32; hops];
    for i in 0..hops {
        let c = (i as f32 + 0.5) * o.hop;
        let s = (((c - half) * sr).max(0.0)) as usize;
        let e = (((c + half) * sr) as usize).min(n);
        if e > s + 1 {
            rms[i] = ((sq[e] - sq[s]) / (e - s) as f64).sqrt() as f32;
            zcr[i] = (zc[e] - zc[s]) as f32 / (e - s) as f32;
        }
    }
    let peak = rms.iter().cloned().fold(0.0f32, f32::max);
    if peak < 1e-4 {
        return Err("audio is silent (peak below -80 dBFS)".into());
    }
    let mut sorted = rms.clone();
    sorted.sort_by(|x, y| x.total_cmp(y));
    let floor = sorted[sorted.len() / 10].min(0.2 * peak);
    let span = peak - floor;
    if span < 1e-4 {
        return Err("no speech detected: audio level is flat".into());
    }
    let level: Vec<f32> = rms.iter().map(|r| ((r - floor) / span).clamp(0.0, 1.0)).collect();
    // hysteresis over hop indices
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut start: Option<usize> = None;
    for i in 0..hops {
        let l = level[i];
        let on = l >= o.on || (l >= 0.5 * o.on && zcr[i] >= o.zcr_on);
        match start {
            None => {
                if on {
                    start = Some(i);
                }
            }
            Some(s) => {
                if l < o.off {
                    runs.push((s, i - 1));
                    start = None;
                }
            }
        }
    }
    if let Some(s) = start {
        runs.push((s, hops - 1));
    }
    // bridge short gaps (in hops)
    let gap_h = (o.min_gap / o.hop).round() as usize;
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for r in runs {
        match merged.last_mut() {
            Some(l) if r.0 <= l.1 + 1 + gap_h => l.1 = r.1,
            _ => merged.push(r),
        }
    }
    // convert to seconds, undoing the half-window smear of the RMS estimate
    let w2 = half;
    let mut regions: Vec<(f32, f32)> = Vec::new();
    for (i0, i1) in merged {
        let c0 = (i0 as f32 + 0.5) * o.hop;
        let c1 = (i1 as f32 + 0.5) * o.hop;
        let mut s = if i0 == 0 { 0.0 } else { c0 + w2 - o.hop * 0.5 };
        let mut e = if i1 == hops - 1 { n as f32 / sr } else { c1 - w2 + o.hop * 0.5 };
        if e <= s {
            s = c0 - o.hop * 0.5;
            e = c1 + o.hop * 0.5;
        }
        if e - s >= o.min_speech {
            regions.push((s.max(0.0), e));
        }
    }
    if regions.is_empty() {
        return Err("no speech detected in the audio".into());
    }
    let voiced: Vec<bool> = (0..hops)
        .map(|i| {
            let c = (i as f32 + 0.5) * o.hop;
            regions.iter().any(|&(s, e)| c >= s && c < e)
        })
        .collect();
    Ok(Vad { hop: o.hop, level, voiced, regions })
}

// ---------------------------------------------------------------------------------------------
// Transcript -> words -> syllables
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Word {
    /// Letters only (punctuation stripped).
    pub text: String,
    pub syllables: Vec<String>,
    /// Strongest punctuation mark that follows the word: `. ! ? , ; :`.
    pub punct: Option<char>,
}

fn is_vowel_char(c: char) -> bool {
    matches!(
        c,
        'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'ă' | 'â' | 'î' | 'à' | 'á' | 'ä' | 'å' | 'è' | 'é' | 'ê' | 'ë' | 'ì' | 'í' | 'ï' | 'ò' | 'ó' | 'ô' | 'ö' | 'ù' | 'ú' | 'û' | 'ü'
    )
}

fn lower(c: char) -> char {
    // comma-below / cedilla variants fold together
    let l = c.to_lowercase().next().unwrap_or(c);
    match l {
        'ş' => 'ș',
        'ţ' => 'ț',
        o => o,
    }
}

fn punct_rank(c: char) -> u8 {
    match c {
        '?' | '!' => 4,
        '.' => 3,
        ';' | ':' => 2,
        ',' => 1,
        _ => 0,
    }
}

fn punct_of(c: char) -> Option<char> {
    match c {
        '?' | '!' | '.' | ';' | ':' | ',' => Some(c),
        '…' => Some('.'),
        _ => None,
    }
}

fn is_cluster(a: char, b: char) -> bool {
    (matches!(a, 'b' | 'c' | 'd' | 'f' | 'g' | 'p' | 't' | 'k') && matches!(b, 'l' | 'r'))
        || (matches!(a, 'c' | 's' | 't' | 'g' | 'p' | 'w') && b == 'h')
}

/// Split one word into syllables by vowel groups (heuristic; not a hyphenation dictionary).
pub fn syllabify(word: &str, lang: Lang) -> Vec<String> {
    let ch: Vec<char> = word.chars().collect();
    if ch.is_empty() {
        return vec![];
    }
    if ch.iter().all(|c| c.is_ascii_digit()) {
        return ch.iter().map(|c| c.to_string()).collect();
    }
    let lw: Vec<char> = ch.iter().map(|&c| lower(c)).collect();
    let n = lw.len();
    let vowel = |i: usize| is_vowel_char(lw[i]) && !(lw[i] == 'u' && i > 0 && lw[i - 1] == 'q');
    // vowel groups
    let mut nuclei: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < n {
        if vowel(i) {
            let s = i;
            while i < n && vowel(i) {
                // hiatus: identical vowels, and a few Romanian pairs
                if i > s {
                    let (p, c) = (lw[i - 1], lw[i]);
                    if p == c || (lang == Lang::Ro && matches!((p, c), ('a', 'e') | ('a', 'o') | ('e', 'o') | ('o', 'e'))) {
                        break;
                    }
                }
                i += 1;
            }
            nuclei.push((s, i));
        } else {
            i += 1;
        }
    }
    // English silent final e (make, time) but not table / bee
    if lang == Lang::En && nuclei.len() > 1 {
        let (s, e) = *nuclei.last().unwrap();
        if e == n && e - s == 1 && lw[s] == 'e' && s > 0 && !is_vowel_char(lw[s - 1]) {
            let le = lw[s - 1] == 'l' && s > 1 && !is_vowel_char(lw[s - 2]);
            if !le {
                nuclei.pop();
            }
        }
    }
    if nuclei.is_empty() {
        return vec![ch.iter().collect()];
    }
    let mut cuts: Vec<usize> = Vec::new();
    for w in nuclei.windows(2) {
        let (a, b) = (w[0], w[1]);
        let run = b.0 - a.1;
        let cut = if run == 0 {
            a.1
        } else if run == 1 {
            b.0 - 1
        } else {
            let two = is_cluster(lw[b.0 - 2], lw[b.0 - 1]);
            b.0 - if two { 2 } else { 1 }
        };
        cuts.push(cut);
    }
    let mut out = Vec::new();
    let mut prev = 0;
    for c in cuts {
        out.push(ch[prev..c].iter().collect());
        prev = c;
    }
    out.push(ch[prev..].iter().collect());
    out
}

/// Split a transcript into words with trailing-punctuation marks.
pub fn parse_transcript(text: &str, lang: Lang) -> Result<Vec<Word>, String> {
    let mut words: Vec<Word> = Vec::new();
    for tok in text.split_whitespace() {
        let letters: String = tok.chars().filter(|c| c.is_alphanumeric()).collect();
        let mut best: Option<char> = None;
        let trailing: Vec<char> = tok.chars().rev().take_while(|c| !c.is_alphanumeric()).collect();
        for c in trailing {
            if let Some(p) = punct_of(c) {
                if best.is_none_or(|b| punct_rank(p) > punct_rank(b)) {
                    best = Some(p);
                }
            }
        }
        if letters.is_empty() {
            // a stray "…", "—" or "?" after a space attaches to the previous word
            if let (Some(p), Some(last)) = (best, words.last_mut()) {
                if last.punct.is_none_or(|b| punct_rank(p) > punct_rank(b)) {
                    last.punct = Some(p);
                }
            }
            continue;
        }
        let syllables = syllabify(&letters, lang);
        words.push(Word { text: letters, syllables, punct: best });
    }
    if words.is_empty() {
        return Err("transcript has no words".into());
    }
    Ok(words)
}

fn syllable_weight(s: &str) -> f32 {
    (0.8 + 0.12 * s.chars().count() as f32).clamp(0.8, 1.8)
}

// ---------------------------------------------------------------------------------------------
// Forced alignment (segmental Viterbi)
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SyllableTiming {
    pub text: String,
    pub start: f32,
    pub end: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WordTiming {
    pub text: String,
    pub start: f32,
    pub end: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub punct: Option<char>,
    pub syllables: Vec<SyllableTiming>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Alignment {
    pub words: Vec<WordTiming>,
    /// Voiced regions found by the VAD.
    pub regions: Vec<(f32, f32)>,
    pub duration: f32,
    /// Estimated seconds per unit of syllable weight (speaking rate).
    pub tempo: f32,
}

#[derive(Clone, Debug)]
pub struct AlignOptions {
    pub vad: VadOptions,
    /// Cost per unvoiced frame inside a syllable, and per voiced frame inside a gap.
    pub mismatch: f32,
    /// Cost per frame of an unpunctuated pause between words.
    pub plain_pause: f32,
    /// Penalty (x energy level) for cutting between two syllables of a word.
    pub boundary: f32,
}

impl Default for AlignOptions {
    fn default() -> Self {
        AlignOptions { vad: VadOptions::default(), mismatch: 1.0, plain_pause: 0.25, boundary: 2.0 }
    }
}

struct Unit {
    /// `Some((word, syllable))` for a syllable, `None` for a gap.
    syl: Option<(usize, usize)>,
    /// Expected length in frames (syllables).
    d: f32,
    /// Per unvoiced frame cost (gaps).
    gap_cost: f32,
}

const MAX_CELLS: usize = 40_000_000;

/// Align a transcript to the audio. See the module docs for the (limited) model.
pub fn align_transcript(a: &Audio, transcript: &str, lang: Lang, o: &AlignOptions) -> Result<Alignment, String> {
    let words = parse_transcript(transcript, lang)?;
    let nsyl: usize = words.iter().map(|w| w.syllables.len()).sum();
    // VAD, coarsening the grid if the DP table would be too large
    let mut vo = o.vad.clone();
    let units_n = nsyl + words.len() + 1;
    let vad = loop {
        let v = detect_speech(a, &vo)?;
        if units_n.saturating_mul(v.voiced.len() + 1) <= MAX_CELLS {
            break v;
        }
        vo.hop *= 2.0;
        vo.window = vo.window.max(vo.hop * 1.5);
        if vo.hop > 0.04 {
            return Err("audio/transcript too long to align in one piece; split both into chunks".into());
        }
    };
    let hop = vad.hop;
    let t_n = vad.voiced.len();
    if t_n < 3 * nsyl {
        return Err(format!("audio too short for {nsyl} syllables"));
    }
    let voiced_s = vad.voiced.iter().filter(|v| **v).count() as f32 * hop;
    if voiced_s / (nsyl as f32) < 0.04 {
        return Err(format!("transcript has {nsyl} syllables but only {voiced_s:.2} s of speech was found (under 40 ms per syllable)"));
    }
    // weights (final lengthening before . ! ?)
    let mut weights: Vec<Vec<f32>> = words.iter().map(|w| w.syllables.iter().map(|s| syllable_weight(s)).collect()).collect();
    for (wi, w) in words.iter().enumerate() {
        if matches!(w.punct, Some('.') | Some('!') | Some('?')) {
            if let Some(l) = weights[wi].last_mut() {
                *l *= 1.2;
            }
        }
    }
    let wsum: f32 = weights.iter().flatten().sum();
    let tempo = (voiced_s / wsum).clamp(0.06, 0.8);
    // chain of units
    let mut units: Vec<Unit> = vec![Unit { syl: None, d: 0.0, gap_cost: 0.0 }];
    for (wi, w) in words.iter().enumerate() {
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for si in 0..w.syllables.len() {
            units.push(Unit { syl: Some((wi, si)), d: (weights[wi][si] * tempo / hop).max(3.0), gap_cost: 0.0 });
        }
        let last = wi + 1 == words.len();
        let gc = if last || w.punct.is_some() { 0.0 } else { o.plain_pause };
        units.push(Unit { syl: None, d: 0.0, gap_cost: gc });
    }
    // prefix counts of voiced frames
    let mut pv = vec![0u32; t_n + 1];
    for i in 0..t_n {
        pv[i + 1] = pv[i] + vad.voiced[i] as u32;
    }
    let lam = o.mismatch;
    let inf = f32::INFINITY;
    let mut prev = vec![inf; t_n + 1];
    prev[0] = 0.0;
    let mut back: Vec<Vec<u32>> = Vec::with_capacity(units.len());
    for u in &units {
        let mut cur = vec![inf; t_n + 1];
        let mut bp = vec![0u32; t_n + 1];
        match u.syl {
            None => {
                for t in 0..=t_n {
                    let mut best = prev[t];
                    let mut from = t as u32;
                    if t > 0 && cur[t - 1].is_finite() {
                        let c = if vad.voiced[t - 1] { lam } else { u.gap_cost };
                        let v = cur[t - 1] + c;
                        if v < best {
                            best = v;
                            from = bp[t - 1];
                        }
                    }
                    cur[t] = best;
                    bp[t] = from;
                }
            }
            Some(_) => {
                let lmin = 3usize;
                let lmax = ((u.d * 3.0).ceil() as usize + 2).max(lmin + 2);
                let sigma = 0.4 * u.d;
                for t in lmin..=t_n {
                    let mut best = inf;
                    let mut from = 0u32;
                    for l in lmin..=lmax.min(t) {
                        let t0 = t - l;
                        let p = prev[t0];
                        if !p.is_finite() {
                            continue;
                        }
                        let unv = l as u32 - (pv[t] - pv[t0]);
                        let z = (l as f32 - u.d) / sigma;
                        let mut c = p + 0.5 * z * z + lam * unv as f32;
                        // cutting inside speech costs the energy at the cut; a start after
                        // silence is free (this also covers zero-length gaps between words)
                        if t0 > 0 && vad.voiced[t0 - 1] && vad.voiced[t0] {
                            c += o.boundary * vad.level[t0 - 1].min(vad.level[t0]);
                        }
                        if c < best {
                            best = c;
                            from = t0 as u32;
                        }
                    }
                    cur[t] = best;
                    bp[t] = from;
                }
            }
        }
        back.push(bp);
        prev = cur;
    }
    if !prev[t_n].is_finite() {
        return Err("transcript does not fit the audio (too many syllables for the speech found)".into());
    }
    // backtrack
    let mut ends = vec![0usize; units.len()];
    let mut starts = vec![0usize; units.len()];
    let mut t = t_n;
    for k in (0..units.len()).rev() {
        ends[k] = t;
        let s = back[k][t] as usize;
        starts[k] = s;
        t = s;
    }
    // collect
    let mut out_words: Vec<WordTiming> = Vec::new();
    for (wi, w) in words.iter().enumerate() {
        out_words.push(WordTiming { text: w.text.clone(), start: 0.0, end: 0.0, punct: w.punct, syllables: Vec::new() });
        let _ = wi;
    }
    for (k, u) in units.iter().enumerate() {
        if let Some((wi, si)) = u.syl {
            out_words[wi].syllables.push(SyllableTiming { text: words[wi].syllables[si].clone(), start: (starts[k] as f64 * hop as f64 * 1e4).round() as f32 / 1e4, end: (ends[k] as f64 * hop as f64 * 1e4).round() as f32 / 1e4 });
        }
    }
    for w in &mut out_words {
        w.start = w.syllables.first().map_or(0.0, |s| s.start);
        w.end = w.syllables.last().map_or(0.0, |s| s.end);
    }
    Ok(Alignment { words: out_words, regions: vad.regions, duration: a.duration(), tempo })
}

// ---------------------------------------------------------------------------------------------
// Word timing export
// ---------------------------------------------------------------------------------------------

pub fn words_json(al: &Alignment) -> Result<String, String> {
    serde_json::to_string_pretty(al).map_err(|e| e.to_string())
}

fn srt_time(t: f32) -> String {
    let ms = (t.max(0.0) * 1000.0).round() as u64;
    format!("{:02}:{:02}:{:02},{:03}", ms / 3_600_000, (ms / 60_000) % 60, (ms / 1000) % 60, ms % 1000)
}

/// SubRip subtitles: a cue ends at sentence punctuation, after 7 words / 42 characters, or at
/// a pause longer than 0.6 s.
pub fn words_srt(al: &Alignment) -> String {
    let mut out = String::new();
    let mut n = 0;
    let mut i = 0;
    while i < al.words.len() {
        let mut j = i;
        let mut len = al.words[i].text.chars().count();
        while j + 1 < al.words.len() {
            let w = &al.words[j];
            let nx = &al.words[j + 1];
            let stop = matches!(w.punct, Some('.') | Some('!') | Some('?')) || j + 1 - i >= 7 || len + 1 + nx.text.chars().count() > 42 || nx.start - w.end > 0.6;
            if stop {
                break;
            }
            len += 1 + nx.text.chars().count();
            j += 1;
        }
        let text: Vec<String> = (i..=j).map(|k| format!("{}{}", al.words[k].text, al.words[k].punct.map(|p| p.to_string()).unwrap_or_default())).collect();
        n += 1;
        out.push_str(&format!("{}\n{} --> {}\n{}\n\n", n, srt_time(al.words[i].start), srt_time(al.words[j].end), text.join(" ")));
        i = j + 1;
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Grapheme -> viseme
// ---------------------------------------------------------------------------------------------

fn vowel_viseme(c: char) -> Option<Viseme> {
    Some(match c {
        'a' | 'à' | 'á' | 'ä' | 'å' => Viseme::Open,
        'e' | 'è' | 'é' | 'ê' | 'ë' | 'i' | 'ì' | 'í' | 'ï' | 'y' => Viseme::Wide,
        'o' | 'ò' | 'ó' | 'ô' | 'ö' | 'u' | 'ù' | 'ú' | 'û' | 'ü' => Viseme::Round,
        'ă' | 'â' | 'î' => Viseme::Small,
        _ => return None,
    })
}

fn consonant_viseme(c: char) -> Viseme {
    match c {
        'm' | 'b' | 'p' => Viseme::Rest, // lips closed
        'f' | 'v' => Viseme::Small,
        's' | 'z' | 'ț' => Viseme::Wide,
        'ș' | 'j' | 'w' => Viseme::Round,
        _ => Viseme::Small,
    }
}

/// Letter groups -> `(viseme, relative duration weight)`. Vowels weigh 3 (long vowel digraphs
/// 4.5), consonants about 1. Bilabial closures map to [`Viseme::Rest`].
pub fn graphemes_to_visemes(word: &str, lang: Lang) -> Vec<(Viseme, f32)> {
    let c: Vec<char> = word.chars().map(lower).filter(|c| c.is_alphanumeric()).collect();
    let n = c.len();
    let mut out: Vec<(Viseme, f32)> = Vec::new();
    let at = |i: usize| c.get(i).copied().unwrap_or('\0');
    let mut i = 0;
    while i < n {
        let (x, y, z) = (c[i], at(i + 1), at(i + 2));
        let front_next = matches!(z, 'e' | 'i' | 'é' | 'î' | 'è' | 'ê' | 'í');
        // digits: a neutral small opening
        if x.is_ascii_digit() {
            out.push((Viseme::Small, 2.0));
            i += 1;
            continue;
        }
        // digraphs (both languages)
        match (x, y) {
            ('s', 'h') if lang == Lang::En => {
                out.push((Viseme::Round, 1.2));
                i += 2;
                continue;
            }
            ('c', 'h') => {
                // Ro: che/chi = k; En: ch = tʃ
                let v = if lang == Lang::Ro && matches!(z, 'e' | 'i') { Viseme::Small } else { Viseme::Round };
                out.push((v, 1.2));
                i += 2;
                continue;
            }
            ('g', 'h') if lang == Lang::Ro && matches!(z, 'e' | 'i') => {
                out.push((Viseme::Small, 1.0));
                i += 2;
                continue;
            }
            ('t', 'h') | ('w', 'h') | ('p', 'h') if lang == Lang::En => {
                let v = if x == 'w' { Viseme::Round } else { Viseme::Small };
                out.push((v, 1.0));
                i += 2;
                continue;
            }
            ('q', 'u') => {
                out.push((Viseme::Round, 1.2));
                i += 2;
                continue;
            }
            ('s', 'c') if lang == Lang::Ro && matches!(z, 'e' | 'i') => {
                out.push((Viseme::Round, 1.2)); // ʃ
                i += 2;
                continue;
            }
            _ => {}
        }
        if lang == Lang::Ro && matches!(x, 'c' | 'g') && matches!(y, 'e' | 'i') {
            out.push((Viseme::Round, 1.0)); // tʃ / dʒ before a front vowel
            i += 1;
            continue;
        }
        // English long vowel digraphs
        if lang == Lang::En {
            let pair = (x, y);
            let v = match pair {
                ('o', 'o') | ('o', 'u') | ('o', 'w') | ('u', 'e') => Some(Viseme::Round),
                ('e', 'e') | ('e', 'a') | ('i', 'e') | ('e', 'i') => Some(Viseme::Wide),
                ('a', 'i') | ('a', 'y') | ('a', 'r') | ('a', 'h') | ('a', 'w') => Some(Viseme::Open),
                _ => None,
            };
            if let Some(v) = v {
                out.push((v, 4.5));
                i += 2;
                continue;
            }
        }
        if let Some(v) = vowel_viseme(x) {
            // doubled vowel = long
            if y == x {
                out.push((v, 4.5));
                i += 2;
                continue;
            }
            // English silent final e after a consonant, in a word with an earlier vowel
            if lang == Lang::En && x == 'e' && i + 1 == n && i >= 2 && !is_vowel_char(c[i - 1]) && c[..i - 1].iter().any(|&q| is_vowel_char(q)) && !(c[i - 1] == 'l' && !is_vowel_char(c[i - 2])) {
                i += 1;
                continue;
            }
            // Romanian word-final unstressed i/u after a consonant is weak
            let weak = lang == Lang::Ro && i + 1 == n && i > 0 && matches!(x, 'i' | 'u') && !is_vowel_char(c[i - 1]);
            out.push((v, if weak { 1.2 } else { 3.0 }));
            i += 1;
            continue;
        }
        let w = if matches!(x, 'm' | 'b' | 'p') { 1.1 } else { 0.9 };
        let _ = front_next;
        if y == x && !is_vowel_char(x) {
            out.push((consonant_viseme(x), w * 1.5)); // geminate
            i += 2;
        } else {
            out.push((consonant_viseme(x), w));
            i += 1;
        }
    }
    out
}

#[derive(Clone, Debug)]
pub struct LipOptions {
    /// Shortest hold of a non-closed mouth shape, in frames.
    pub min_hold: u32,
}

impl Default for LipOptions {
    fn default() -> Self {
        LipOptions { min_hold: 2 }
    }
}

/// Timeline of `(start, end, shape)` segments from the aligned syllables.
pub fn viseme_segments(al: &Alignment, lang: Lang, bridge: f32) -> Vec<(f32, f32, Viseme)> {
    let mut segs: Vec<(f32, f32, Viseme)> = Vec::new();
    for w in &al.words {
        for s in &w.syllables {
            let toks = graphemes_to_visemes(&s.text, lang);
            let tot: f32 = toks.iter().map(|t| t.1).sum();
            if tot <= 0.0 || s.end <= s.start {
                continue;
            }
            let mut t = s.start;
            for (v, wt) in toks {
                let e = t + (s.end - s.start) * wt / tot;
                segs.push((t, e, v));
                t = e;
            }
        }
    }
    // hold the last shape across very short gaps (inter-word breaths)
    for i in 1..segs.len() {
        let gap = segs[i].0 - segs[i - 1].1;
        if gap > 0.0 && gap < bridge {
            let mid = segs[i - 1].1 + gap;
            segs[i - 1].1 = mid;
        }
    }
    segs
}

/// One mouth shape per project frame, with minimum hold and coarticulation clean-up.
pub fn viseme_frames(al: &Alignment, lang: Lang, fps: f32, frames: u32, o: &LipOptions) -> Result<Vec<Viseme>, String> {
    if !(fps.is_finite() && fps > 0.0) || frames == 0 {
        return Err("project needs fps > 0 and at least one frame".into());
    }
    let hold = o.min_hold.max(1) as usize;
    let segs = viseme_segments(al, lang, hold as f32 / fps);
    let mut v: Vec<Viseme> = Vec::with_capacity(frames as usize);
    let mut si = 0;
    for f in 0..frames {
        let t = (f as f32 + 0.5) / fps;
        while si < segs.len() && segs[si].1 <= t {
            si += 1;
        }
        v.push(if si < segs.len() && segs[si].0 <= t { segs[si].2 } else { Viseme::Rest });
    }
    // runs of non-closed shapes shorter than `hold` are absorbed by the longer neighbour
    for _ in 0..8 {
        let mut runs: Vec<(usize, usize, Viseme)> = Vec::new(); // start, len, shape
        for (i, &s) in v.iter().enumerate() {
            match runs.last_mut() {
                Some(r) if r.2 == s => r.1 += 1,
                _ => runs.push((i, 1, s)),
            }
        }
        let mut changed = false;
        for k in 0..runs.len() {
            let (s, l, shape) = runs[k];
            if shape == Viseme::Rest || l >= hold {
                continue;
            }
            let left = if k > 0 && runs[k - 1].2 != Viseme::Rest { Some(runs[k - 1]) } else { None };
            let right = if k + 1 < runs.len() && runs[k + 1].2 != Viseme::Rest { Some(runs[k + 1]) } else { None };
            let to = match (left, right) {
                (Some(a), Some(b)) => Some(if a.1 > b.1 { a.2 } else { b.2 }),
                (Some(a), None) => Some(a.2),
                (None, Some(b)) => Some(b.2),
                (None, None) => None,
            };
            if let Some(t) = to {
                for x in &mut v[s..s + l] {
                    *x = t;
                }
                changed = true;
                break; // recompute runs
            }
        }
        if !changed {
            break;
        }
    }
    Ok(v)
}

fn lock_free_replace(p: &mut Project, ch: Channel) -> Result<(), String> {
    crate::chanmeta::assert_unlocked(p, &ch.target, ch.prop)?;
    p.channels.retain(|c| !(c.target == ch.target && c.prop == ch.prop && c.mode == ch.mode && matches!(c.source, Source::Keys { .. })));
    p.channels.push(ch);
    Ok(())
}

/// Bake the transcript-aligned mouth shapes into a switch layer's exposure channel.
/// Returns the number of exposure changes.
pub fn apply_lipsync(p: &mut Project, layer: &str, al: &Alignment, lang: Lang, mapping: &[usize; 5], o: &LipOptions) -> Result<usize, String> {
    let l = p.find_layer(layer).ok_or_else(|| format!("no layer '{layer}'"))?;
    let n = match &l.kind {
        LayerKind::Switch { layers, .. } => layers.len(),
        _ => return Err(format!("layer '{layer}' is not a switch layer")),
    };
    if let Some(bad) = mapping.iter().find(|&&m| m >= n) {
        return Err(format!("--map index {bad} is out of range: '{layer}' has {n} children"));
    }
    crate::chanmeta::assert_unlocked(p, layer, Prop::LayerSwitch)?;
    let vis = viseme_frames(al, lang, p.fps, p.frames, o)?;
    let ex = exposures(&vis, mapping);
    let count = ex.len();
    lock_free_replace(p, Channel::exposure(layer, &ex).with_mode(Mode::Replace))?;
    Ok(count)
}

// ---------------------------------------------------------------------------------------------
// Speech-aware secondary motion
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct MotionOptions {
    /// Bone (or layer) that nods and falls with the speech.
    pub head: Option<String>,
    /// Bone (or layer) that carries the eyebrows; raised by moving it up (negative y).
    pub brow: Option<String>,
    /// Degrees of a stress nod (sign picks the direction; 0 disables).
    pub nod_deg: f32,
    /// Degrees of the sentence-final fall.
    pub fall_deg: f32,
    /// Degrees of the sideways tilt on questions (opposite sign to the nod).
    pub tilt_deg: f32,
    /// Pixels of brow lift on questions (stress flicks use 35 percent of it).
    pub brow_px: f32,
    pub seed: u64,
}

impl Default for MotionOptions {
    fn default() -> Self {
        MotionOptions { head: None, brow: None, nod_deg: 3.0, fall_deg: 2.5, tilt_deg: 3.0, brow_px: 4.0, seed: 1 }
    }
}

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03 | 1)
    }
    /// 0..1
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        ((self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32) / 16_777_216.0
    }
}

/// Raised-cosine pulse: rises over `att`, holds for `hold`, falls over `rel`.
fn pulse(t: f32, att: f32, hold: f32, rel: f32) -> f32 {
    if t < 0.0 {
        0.0
    } else if t < att {
        0.5 - 0.5 * (std::f32::consts::PI * t / att.max(1e-4)).cos()
    } else if t < att + hold {
        1.0
    } else if t < att + hold + rel {
        0.5 + 0.5 * (std::f32::consts::PI * (t - att - hold) / rel.max(1e-4)).cos()
    } else {
        0.0
    }
}

fn add_pulse(v: &mut [f32], fps: f32, start: f32, amp: f32, att: f32, hold: f32, rel: f32) {
    let f0 = (start * fps).floor().max(0.0) as usize;
    let f1 = (((start + att + hold + rel) * fps).ceil() as usize + 1).min(v.len());
    for (f, x) in v.iter_mut().enumerate().take(f1).skip(f0) {
        *x += amp * pulse(f as f32 / fps - start, att, hold, rel);
    }
}

fn track_from(v: &[f32]) -> Track {
    let n = v.len();
    let mut keys: Vec<(f32, f32, Ease)> = Vec::new();
    for f in 0..n {
        let edge = f == 0 || f + 1 == n || (v[f] - v[f - 1]).abs() > 1e-6 || (v[f + 1] - v[f]).abs() > 1e-6;
        if edge {
            keys.push((f as f32, v[f], Ease::Linear));
        }
    }
    Track::with_keys(&keys)
}

fn motion_target(p: &Project, name: &str, bone_prop: Prop, layer_prop: Prop) -> Result<Prop, String> {
    if p.bones.iter().any(|b| b.name == name) {
        Ok(bone_prop)
    } else if p.find_layer(name).is_some() {
        Ok(layer_prop)
    } else {
        Err(format!("no bone or layer named '{name}'"))
    }
}

/// Small additive keyed channels (head angle, brow lift) driven by the aligned speech:
/// nods on stressed words (long syllables) and sentence starts, a fall at sentence ends,
/// brow raise and head tilt on questions, a lighter raise on `!`. Amplitudes are jittered
/// +-15 percent with a seeded generator, so the same inputs always give the same keys.
pub fn speech_motion(p: &Project, al: &Alignment, o: &MotionOptions) -> Result<Vec<Channel>, String> {
    if o.head.is_none() && o.brow.is_none() {
        return Err("give a head target and/or a brow target".into());
    }
    for (n, x, lim) in [("nod", o.nod_deg, 45.0), ("fall", o.fall_deg, 45.0), ("tilt", o.tilt_deg, 45.0), ("brow", o.brow_px, 200.0)] {
        if !x.is_finite() || x.abs() > lim {
            return Err(format!("{n} amplitude must be a finite number within +-{lim}"));
        }
    }
    if !(p.fps.is_finite() && p.fps > 0.0) || p.frames == 0 {
        return Err("project needs fps > 0 and at least one frame".into());
    }
    if al.words.is_empty() {
        return Err("alignment has no words".into());
    }
    let head_prop = o.head.as_deref().map(|h| motion_target(p, h, Prop::BoneAngle, Prop::LayerRotation)).transpose()?;
    let brow_prop = o.brow.as_deref().map(|b| motion_target(p, b, Prop::BoneOffsetY, Prop::LayerY)).transpose()?;
    if let (Some(h), Some(pr)) = (&o.head, head_prop) {
        crate::chanmeta::assert_unlocked(p, h, pr)?;
    }
    if let (Some(b), Some(pr)) = (&o.brow, brow_prop) {
        crate::chanmeta::assert_unlocked(p, b, pr)?;
    }
    let fps = p.fps;
    let n = p.frames as usize;
    let mut head = vec![0f32; n];
    let mut brow = vec![0f32; n];
    let mut rng = Rng::new(o.seed);
    // median syllable duration = the "unstressed" yardstick
    let mut durs: Vec<f32> = al.words.iter().flat_map(|w| w.syllables.iter().map(|s| s.end - s.start)).collect();
    durs.sort_by(|a, b| a.total_cmp(b));
    let median = durs.get(durs.len() / 2).copied().unwrap_or(0.2).max(0.02);
    let mut last_nod = f32::NEG_INFINITY;
    let mut sentence_start = true;
    for (i, w) in al.words.iter().enumerate() {
        let jit = 0.85 + 0.3 * rng.next();
        let tj = (rng.next() - 0.5) * 0.04;
        let (peak_syl, peak_dur) = w.syllables.iter().fold((w.start, 0.0f32), |acc, s| if s.end - s.start > acc.1 { (s.start, s.end - s.start) } else { acc });
        let stress = peak_dur / median;
        let ends_sentence = matches!(w.punct, Some('.') | Some('!') | Some('?')) || i + 1 == al.words.len();
        if (stress >= 1.2 || sentence_start) && peak_syl - last_nod >= 0.35 {
            let a = if sentence_start { (0.4 + (stress - 1.0).max(0.0)).clamp(0.5, 1.0) } else { (0.4 + (stress - 1.0)).clamp(0.4, 1.0) };
            add_pulse(&mut head, fps, peak_syl + tj, o.nod_deg * a * jit, 0.07, 0.0, 0.25);
            last_nod = peak_syl;
        }
        if stress >= 1.3 {
            add_pulse(&mut brow, fps, peak_syl + tj, -0.35 * o.brow_px * jit, 0.06, 0.04, 0.25);
        }
        match w.punct {
            Some('?') => {
                let from = if i > 0 { al.words[i - 1].start.max(w.start - 0.4) } else { w.start };
                let hold = (w.end - from).max(0.0);
                add_pulse(&mut brow, fps, from, -o.brow_px * jit, 0.15, hold, 0.35);
                add_pulse(&mut head, fps, from, -o.tilt_deg * jit, 0.2, hold, 0.4);
            }
            Some('!') => {
                add_pulse(&mut brow, fps, w.start, -0.6 * o.brow_px * jit, 0.1, (w.end - w.start).max(0.0), 0.3);
            }
            _ => {}
        }
        if ends_sentence && w.punct != Some('?') {
            let s = w.syllables.last().map_or(w.start, |s| s.start);
            let att = (w.end - s).max(0.15);
            add_pulse(&mut head, fps, s, o.fall_deg * jit, att, 0.1, 0.4);
        }
        sentence_start = ends_sentence;
    }
    let mut out = Vec::new();
    if let (Some(h), Some(pr)) = (&o.head, head_prop) {
        out.push(Channel::keys(h, pr, track_from(&head)).with_mode(Mode::Add));
    }
    if let (Some(b), Some(pr)) = (&o.brow, brow_prop) {
        let lim = 1.2 * o.brow_px.abs();
        let clamped: Vec<f32> = brow.iter().map(|x| x.clamp(-lim, lim)).collect();
        out.push(Channel::keys(b, pr, track_from(&clamped)).with_mode(Mode::Add));
    }
    Ok(out)
}

/// Add the motion channels. Earlier additive keyed channels on the same target and property
/// (a previous run) are replaced so repeating the command does not double the motion.
pub fn apply_speech_motion(p: &mut Project, chans: Vec<Channel>) -> Result<usize, String> {
    for c in &chans {
        crate::chanmeta::assert_unlocked(p, &c.target, c.prop)?;
    }
    let n = chans.len();
    for c in chans {
        lock_free_replace(p, c)?;
    }
    Ok(n)
}

// ---------------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{Layer, Project};

    const SR: u32 = 16000;

    fn tone(a: &mut Vec<f32>, hz: f32, amp: f32, secs: f32) {
        let start = a.len();
        let n = (secs * SR as f32).round() as usize;
        for i in 0..n {
            a.push(amp * (std::f32::consts::TAU * hz * (start + i) as f32 / SR as f32).sin());
        }
    }
    fn quiet(a: &mut Vec<f32>, secs: f32) {
        a.extend(std::iter::repeat_n(0.0, (secs * SR as f32).round() as usize));
    }
    /// bursts as (start, end) seconds; returns audio of `total` seconds
    fn bursts(spans: &[(f32, f32)], total: f32) -> Audio {
        let mut s = Vec::new();
        let mut t = 0.0;
        for &(a, b) in spans {
            quiet(&mut s, a - t);
            tone(&mut s, 220.0, 0.6, b - a);
            t = b;
        }
        quiet(&mut s, total - t);
        Audio { samples: s, sample_rate: SR }
    }
    /// A burst whose amplitude dips to 15 percent at each of the given times.
    fn dipped(start: f32, end: f32, dips: &[f32], total: f32) -> Audio {
        let mut a = bursts(&[(start, end)], total);
        for (i, x) in a.samples.iter_mut().enumerate() {
            let t = i as f32 / SR as f32;
            let d = dips.iter().map(|&d| (t - d).abs()).fold(1.0f32, f32::min);
            *x *= (0.15 + 0.85 * (d / 0.05).min(1.0)).min(1.0);
        }
        a
    }

    fn near(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn vad_finds_each_burst_within_two_hops() {
        let spans = [(0.30, 0.90), (1.40, 1.90), (2.60, 3.40)];
        let a = bursts(&spans, 4.0);
        let v = detect_speech(&a, &VadOptions::default()).unwrap();
        assert_eq!(v.regions.len(), 3, "{:?}", v.regions);
        for (r, s) in v.regions.iter().zip(spans) {
            assert!(near(r.0, s.0, 0.012) && near(r.1, s.1, 0.012), "{r:?} vs {s:?}");
        }
    }

    #[test]
    fn vad_bridges_short_gaps_and_drops_clicks() {
        let a = bursts(&[(0.30, 0.60), (0.63, 0.90), (1.5, 1.52)], 2.0);
        let v = detect_speech(&a, &VadOptions::default()).unwrap();
        assert_eq!(v.regions.len(), 1, "{:?}", v.regions);
    }

    #[test]
    fn vad_uses_zero_crossings_for_weak_fricatives() {
        // loud vowel then a weak 5 kHz hiss-like tone (7 percent amplitude)
        let mut s = Vec::new();
        quiet(&mut s, 0.2);
        tone(&mut s, 220.0, 0.8, 0.4);
        quiet(&mut s, 0.4);
        tone(&mut s, 5000.0, 0.07, 0.3);
        quiet(&mut s, 0.3);
        let a = Audio { samples: s, sample_rate: SR };
        assert_eq!(detect_speech(&a, &VadOptions::default()).unwrap().regions.len(), 2);
        // the same weak level at low pitch is below the threshold
        let mut s = Vec::new();
        quiet(&mut s, 0.2);
        tone(&mut s, 220.0, 0.8, 0.4);
        quiet(&mut s, 0.4);
        tone(&mut s, 220.0, 0.07, 0.3);
        quiet(&mut s, 0.3);
        let a = Audio { samples: s, sample_rate: SR };
        assert_eq!(detect_speech(&a, &VadOptions::default()).unwrap().regions.len(), 1);
    }

    #[test]
    fn vad_errors() {
        let silent = Audio { samples: vec![0.0; 16000], sample_rate: SR };
        assert!(detect_speech(&silent, &VadOptions::default()).unwrap_err().contains("silent"));
        let empty = Audio { samples: vec![], sample_rate: SR };
        assert!(detect_speech(&empty, &VadOptions::default()).is_err());
        let flat = Audio { samples: vec![0.3; 16000], sample_rate: SR };
        assert!(detect_speech(&flat, &VadOptions::default()).is_err(), "constant DC has no speech");
        let bad = VadOptions { off: 0.5, on: 0.2, ..VadOptions::default() };
        assert!(detect_speech(&bursts(&[(0.1, 0.5)], 1.0), &bad).is_err());
        let nan = Audio { samples: vec![f32::NAN; 100], sample_rate: SR };
        assert!(detect_speech(&nan, &VadOptions::default()).is_err());
    }

    #[test]
    fn syllables_follow_vowel_groups() {
        let s = |w: &str, l| syllabify(w, l);
        assert_eq!(s("mere", Lang::Ro), vec!["me", "re"]);
        assert_eq!(s("foarte", Lang::Ro), vec!["foar", "te"]);
        assert_eq!(s("bună", Lang::Ro).len(), 2);
        assert_eq!(s("România", Lang::Ro).len(), 3, "known limit: final -ia is read as one diphthong (real: 4)");
        assert_eq!(s("aer", Lang::Ro), vec!["a", "er"]);
        assert_eq!(s("make", Lang::En), vec!["make"]);
        assert_eq!(s("table", Lang::En).len(), 2);
        assert_eq!(s("hello", Lang::En), vec!["hel", "lo"]);
        assert_eq!(s("strengths", Lang::En), vec!["strengths"]);
        assert_eq!(s("123", Lang::En).len(), 3);
        assert_eq!(s("a", Lang::En), vec!["a"]);
        let joined: String = s("transcriere", Lang::Ro).concat();
        assert_eq!(joined, "transcriere", "syllables concatenate back to the word");
    }

    #[test]
    fn transcript_parsing_keeps_order_and_marks_punctuation() {
        let w = parse_transcript("Ana are mere, dar... ce faci?  — Bine!", Lang::Ro).unwrap();
        let t: Vec<&str> = w.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(t, vec!["Ana", "are", "mere", "dar", "ce", "faci", "Bine"]);
        assert_eq!(w[2].punct, Some(','));
        assert_eq!(w[3].punct, Some('.'));
        assert_eq!(w[5].punct, Some('?'));
        assert_eq!(w[6].punct, Some('!'));
        assert!(parse_transcript("   ", Lang::Ro).is_err());
        assert!(parse_transcript("... — ?!", Lang::En).is_err(), "punctuation alone is not a word");
        assert!(Lang::parse("xx").is_err());
    }

    fn spans_of(al: &Alignment) -> Vec<(f32, f32)> {
        al.words.iter().map(|w| (w.start, w.end)).collect()
    }

    #[test]
    fn syllables_land_inside_bursts_in_order_and_error_is_small() {
        // "Ana are" | "mere." | "Foarte bune!"   (4 + 2 + 4 syllables)
        let spans = [(0.3073, 1.1041), (1.4118, 1.9087), (2.6036, 3.4059)];
        let a = bursts(&spans, 4.0);
        let al = align_transcript(&a, "Ana are mere. Foarte bune!", Lang::Ro, &AlignOptions::default()).unwrap();
        assert_eq!(al.words.iter().map(|w| w.text.as_str()).collect::<Vec<_>>(), vec!["Ana", "are", "mere", "Foarte", "bune"]);
        assert_eq!(al.words.iter().map(|w| w.syllables.len()).sum::<usize>(), 10);
        let burst_of = |t: f32| spans.iter().position(|s| t >= s.0 - 0.02 && t <= s.1 + 0.02);
        let mut prev_end = 0.0;
        let mut worst = 0.0f32;
        for w in &al.words {
            assert!(w.start >= prev_end - 1e-4, "word order / overlap: {w:?}");
            prev_end = w.end;
            let mut sp = w.start;
            for s in &w.syllables {
                assert!(s.start >= sp - 1e-4 && s.end > s.start, "{s:?}");
                sp = s.end;
                let (b0, b1) = (burst_of(s.start + 0.001), burst_of(s.end - 0.001));
                assert!(b0.is_some() && b0 == b1, "syllable {s:?} not inside one burst");
            }
        }
        // edges: first word start and last word end of each burst
        let firsts = [al.words[0].start, al.words[2].start, al.words[3].start];
        let lasts = [al.words[1].end, al.words[2].end, al.words[4].end];
        for (i, s) in spans.iter().enumerate() {
            worst = worst.max((firsts[i] - s.0).abs()).max((lasts[i] - s.1).abs());
        }
        println!("measured max burst-edge error (clean tone bursts): {:.1} ms", worst * 1000.0);
        assert!(worst <= 0.021, "edge error {worst}");
        assert_eq!(al.regions.len(), 3);
    }

    #[test]
    fn background_noise_costs_little_accuracy() {
        let spans = [(0.3073, 1.1041), (1.4118, 1.9087)];
        let mut a = bursts(&spans, 2.5);
        let mut x = 12345u32;
        for s in a.samples.iter_mut() {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            *s += ((x >> 8) as f32 / 8388608.0 - 1.0) * 0.03; // about -30 dB hiss
        }
        let al = align_transcript(&a, "Ana are mere", Lang::Ro, &AlignOptions::default()).unwrap();
        let worst = [(al.words[0].start, spans[0].0), (al.words[1].end, spans[0].1), (al.words[2].start, spans[1].0), (al.words[2].end, spans[1].1)]
            .iter()
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        println!("measured max burst-edge error with -30 dB noise: {:.1} ms", worst * 1000.0);
        assert!(worst <= 0.03, "{worst}");
    }

    #[test]
    fn internal_boundaries_snap_to_energy_dips() {
        // two words of two syllables each in one 1.2 s burst, dips at 0.443 / 0.837 / 1.068 s (equal splits would be 0.5 / 0.8 / 1.1);
        // syllable letter-counts are equal so only the dip cost can pull the cuts there
        let dips = [0.443, 0.837, 1.068];
        let a = dipped(0.2, 1.4, &dips, 2.0);
        let al = align_transcript(&a, "mama tata", Lang::Ro, &AlignOptions::default()).unwrap();
        let cuts = [al.words[0].syllables[0].end, al.words[0].syllables[1].end, al.words[1].syllables[0].end];
        let mut worst = 0.0f32;
        for (c, d) in cuts.iter().zip(dips) {
            worst = worst.max((c - d).abs());
        }
        println!("measured max boundary error vs energy dips: {:.1} ms", worst * 1000.0);
        assert!(worst <= 0.04, "cuts {cuts:?} vs dips {dips:?}");
    }

    #[test]
    fn punctuation_prefers_the_real_pause() {
        // two bursts, pause between; commas elsewhere must not move the split
        let a = bursts(&[(0.2, 0.8), (1.3, 1.9)], 2.5);
        let al = align_transcript(&a, "bine bine, tare tare", Lang::Ro, &AlignOptions::default()).unwrap();
        assert!(al.words[1].end <= 0.85 && al.words[2].start >= 1.25, "{:?}", spans_of(&al));
        // no punctuation at all: the pause is still found, at the same place
        let al = align_transcript(&a, "bine bine tare tare", Lang::Ro, &AlignOptions::default()).unwrap();
        assert!(al.words[1].end <= 0.85 && al.words[2].start >= 1.25, "{:?}", spans_of(&al));
    }

    #[test]
    fn alignment_errors_and_determinism() {
        let a = bursts(&[(0.2, 0.8)], 1.0);
        let o = AlignOptions::default();
        assert!(align_transcript(&a, "", Lang::Ro, &o).unwrap_err().contains("no words"));
        let silent = Audio { samples: vec![0.0; 16000], sample_rate: SR };
        assert!(align_transcript(&silent, "ana", Lang::Ro, &o).unwrap_err().contains("silent"));
        let long = "ana are mere si pere si multe alte fructe delicioase de toamna pentru toata lumea";
        assert!(align_transcript(&a, long, Lang::Ro, &o).is_err(), "far too many syllables for 0.6 s of speech");
        let x = align_transcript(&a, "ana are", Lang::Ro, &o).unwrap();
        let y = align_transcript(&a, "ana are", Lang::Ro, &o).unwrap();
        assert_eq!(x, y);
        assert_eq!(words_json(&x).unwrap(), words_json(&y).unwrap());
    }

    #[test]
    fn json_and_srt_export() {
        let a = bursts(&[(0.3, 1.1), (1.4, 1.9)], 2.2);
        let al = align_transcript(&a, "Ana are mere.", Lang::Ro, &AlignOptions::default()).unwrap();
        let j: serde_json::Value = serde_json::from_str(&words_json(&al).unwrap()).unwrap();
        assert_eq!(j["words"].as_array().unwrap().len(), 3);
        assert!(j["words"][0]["syllables"].is_array());
        let srt = words_srt(&al);
        assert!(srt.starts_with("1\n00:00:00,"), "{srt}");
        assert!(srt.contains(" --> ") && srt.contains("Ana are mere."), "{srt}");
        assert_eq!(srt_time(3725.25), "01:02:05,250");
    }

    fn vis(w: &str, l: Lang) -> Vec<Viseme> {
        graphemes_to_visemes(w, l).into_iter().map(|x| x.0).collect()
    }

    #[test]
    fn viseme_rules_romanian() {
        use Viseme::*;
        assert_eq!(vis("mama", Lang::Ro), vec![Rest, Open, Rest, Open]);
        assert_eq!(vis("foc", Lang::Ro), vec![Small, Round, Small]);
        assert_eq!(vis("casă", Lang::Ro), vec![Small, Open, Wide, Small]);
        assert_eq!(vis("țară", Lang::Ro), vec![Wide, Open, Small, Small]);
        assert_eq!(vis("șase", Lang::Ro), vec![Round, Open, Wide, Wide]);
        assert_eq!(vis("ŞI", Lang::Ro)[0], Round, "cedilla form folds to comma-below");
        assert_eq!(vis("vâna", Lang::Ro)[1], Small);
        assert_eq!(vis("în", Lang::Ro)[0], Small);
        assert_eq!(vis("ceai", Lang::Ro)[0], Round, "c before e is tʃ");
        assert_eq!(vis("chitară", Lang::Ro)[0], Small, "chi = k");
        assert_eq!(vis("lupi", Lang::Ro).last(), Some(&Wide));
        let w = graphemes_to_visemes("lupi", Lang::Ro);
        assert!(w.last().unwrap().1 < 2.0, "word-final i is weak");
    }

    #[test]
    fn viseme_rules_english() {
        use Viseme::*;
        assert_eq!(vis("moon", Lang::En), vec![Rest, Round, Small]);
        assert_eq!(vis("make", Lang::En), vec![Rest, Open, Small], "silent final e");
        assert_eq!(vis("she", Lang::En), vec![Round, Wide]);
        assert_eq!(vis("the", Lang::En), vec![Small, Wide]);
        assert_eq!(vis("bee", Lang::En), vec![Rest, Wide]);
        assert_eq!(vis("five", Lang::En)[0], Small);
        assert_eq!(vis("queen", Lang::En)[0], Round);
        assert_eq!(vis("table", Lang::En).last(), Some(&Wide), "-le keeps its e");
    }

    fn mouth_project(frames: u32) -> Project {
        let mut p = Project::new("t", 100.0, 100.0, 24.0, frames);
        let kids: Vec<Layer> = (0..5).map(|i| Layer::group(&format!("m{i}"), vec![])).collect();
        p.layers.push(Layer::switch("mouth", kids));
        p
    }

    #[test]
    fn lipsync_keys_close_on_labials_open_on_a_and_rest_in_pauses() {
        let a = bursts(&[(0.5, 1.3)], 2.5);
        let al = align_transcript(&a, "mama", Lang::Ro, &AlignOptions::default()).unwrap();
        let v = viseme_frames(&al, Lang::Ro, 24.0, 60, &LipOptions::default()).unwrap();
        assert_eq!(v[2], Viseme::Rest, "before speech");
        assert_eq!(v[55], Viseme::Rest, "after speech");
        let mid = &v[12..32];
        assert!(mid.contains(&Viseme::Open) && mid.contains(&Viseme::Rest), "{mid:?}");
        // minimum hold: every non-rest run lasts >= 2 frames
        let mut run = 0;
        for i in 0..v.len() {
            if v[i] != Viseme::Rest && i + 1 < v.len() && v[i] == v[i + 1] {
                run += 1;
            } else if v[i] != Viseme::Rest {
                assert!(run + 1 >= 2, "short run ending at {i}: {v:?}");
                run = 0;
            }
        }
        let mut p = mouth_project(60);
        let n = apply_lipsync(&mut p, "mouth", &al, Lang::Ro, &[0, 1, 2, 3, 4], &LipOptions::default()).unwrap();
        assert!(n >= 4);
        assert_eq!(p.channels.len(), 1);
        // repeat run replaces, not stacks
        apply_lipsync(&mut p, "mouth", &al, Lang::Ro, &[0, 1, 2, 3, 4], &LipOptions::default()).unwrap();
        assert_eq!(p.channels.len(), 1);
    }

    #[test]
    fn lipsync_validation_errors() {
        let a = bursts(&[(0.5, 1.3)], 2.5);
        let al = align_transcript(&a, "mama", Lang::Ro, &AlignOptions::default()).unwrap();
        let o = LipOptions::default();
        let mut p = mouth_project(60);
        assert!(apply_lipsync(&mut p, "nope", &al, Lang::Ro, &[0, 1, 2, 3, 4], &o).unwrap_err().contains("no layer"));
        p.layers.push(Layer::group("grp", vec![]));
        assert!(apply_lipsync(&mut p, "grp", &al, Lang::Ro, &[0, 1, 2, 3, 4], &o).unwrap_err().contains("not a switch"));
        assert!(apply_lipsync(&mut p, "mouth", &al, Lang::Ro, &[0, 1, 2, 3, 9], &o).unwrap_err().contains("out of range"));
        apply_lipsync(&mut p, "mouth", &al, Lang::Ro, &[0, 1, 2, 3, 4], &o).unwrap();
        crate::chanmeta::set_meta(&mut p, "mouth", Some(Prop::LayerSwitch), &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        assert!(apply_lipsync(&mut p, "mouth", &al, Lang::Ro, &[0, 1, 2, 3, 4], &o).unwrap_err().contains("locked"));
        assert!(viseme_frames(&al, Lang::Ro, 0.0, 10, &o).is_err());
    }

    fn motion_project(frames: u32) -> Project {
        let mut p = Project::new("t", 100.0, 100.0, 24.0, frames);
        p.bones.push(crate::project::Bone::new("head", None, 10.0, crate::math::Vec2::new(0.0, 0.0), 0.0));
        p.layers.push(Layer::group("brows", vec![]));
        p
    }

    fn q_alignment() -> Alignment {
        let a = bursts(&[(0.3, 1.5), (2.0, 3.0)], 3.6);
        align_transcript(&a, "Ana are mere. Tu vrei mere?", Lang::Ro, &AlignOptions::default()).unwrap()
    }

    #[test]
    fn question_raises_brows_and_stress_nods_head() {
        let al = q_alignment();
        let p = motion_project(100);
        let o = MotionOptions { head: Some("head".into()), brow: Some("brows".into()), ..MotionOptions::default() };
        let ch = speech_motion(&p, &al, &o).unwrap();
        assert_eq!(ch.len(), 2);
        assert!(ch.iter().all(|c| c.mode == Mode::Add));
        assert_eq!(ch[0].prop, Prop::BoneAngle);
        assert_eq!(ch[1].prop, Prop::LayerY);
        let val = |c: &Channel, f: f32| match &c.source {
            Source::Keys { track } => track.eval(f).unwrap_or(0.0),
            _ => panic!(),
        };
        // brow: still before the question, raised (negative y) at its end, within amplitude
        let q_end = al.words.last().unwrap().end * 24.0;
        assert!(val(&ch[1], 1.0).abs() < 1e-4);
        assert!(val(&ch[1], q_end - 1.0) < -3.0, "{}", val(&ch[1], q_end - 1.0));
        let lo = (0..100).map(|f| val(&ch[1], f as f32)).fold(0.0f32, f32::min);
        assert!(lo >= -1.2 * 4.0 - 1e-3, "brow stays within amplitude, got {lo}");
        // head: moves, and stays small
        let hv: Vec<f32> = (0..100).map(|f| val(&ch[0], f as f32)).collect();
        assert!(hv.iter().any(|x| x.abs() > 1.0));
        assert!(hv.iter().all(|x| x.abs() <= 3.0 * 1.15 + 2.5 * 1.15 + 3.0 * 1.15));
        // tail of the clip is back at rest
        assert!(hv[99].abs() < 1e-3);
    }

    #[test]
    fn motion_is_deterministic_and_seed_changes_it() {
        let al = q_alignment();
        let p = motion_project(100);
        let o = MotionOptions { head: Some("head".into()), brow: Some("brows".into()), ..MotionOptions::default() };
        let a = speech_motion(&p, &al, &o).unwrap();
        let b = speech_motion(&p, &al, &o).unwrap();
        assert_eq!(a, b);
        let c = speech_motion(&p, &al, &MotionOptions { seed: 99, ..o.clone() }).unwrap();
        assert_ne!(a, c);
        let zero = speech_motion(&p, &al, &MotionOptions { nod_deg: 0.0, fall_deg: 0.0, tilt_deg: 0.0, ..o.clone() }).unwrap();
        match &zero[0].source {
            Source::Keys { track } => assert!(track.keys.iter().all(|k| k.value == 0.0)),
            _ => panic!(),
        }
    }

    #[test]
    fn motion_validation_and_idempotent_apply() {
        let al = q_alignment();
        let mut p = motion_project(100);
        let base = MotionOptions { head: Some("head".into()), brow: Some("brows".into()), ..MotionOptions::default() };
        assert!(speech_motion(&p, &al, &MotionOptions::default()).is_err());
        assert!(speech_motion(&p, &al, &MotionOptions { head: Some("ghost".into()), ..MotionOptions::default() }).unwrap_err().contains("ghost"));
        assert!(speech_motion(&p, &al, &MotionOptions { nod_deg: f32::NAN, ..base.clone() }).is_err());
        assert!(speech_motion(&p, &al, &MotionOptions { nod_deg: 500.0, ..base.clone() }).is_err());
        let empty = Alignment { words: vec![], regions: vec![], duration: 1.0, tempo: 0.2 };
        assert!(speech_motion(&p, &empty, &base).is_err());
        let ch = speech_motion(&p, &al, &base).unwrap();
        apply_speech_motion(&mut p, ch.clone()).unwrap();
        apply_speech_motion(&mut p, ch).unwrap();
        assert_eq!(p.channels.len(), 2, "re-running replaces instead of doubling");
        crate::chanmeta::set_meta(&mut p, "head", Some(Prop::BoneAngle), &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        assert!(speech_motion(&p, &al, &base).unwrap_err().contains("locked"));
    }
}
