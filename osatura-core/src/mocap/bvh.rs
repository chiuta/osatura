// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! BVH data model and parser (part of `mocap`; see the module docs in `mod.rs`).
use super::*;

// BVH data model and parser

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chan {
    Xpos,
    Ypos,
    Zpos,
    Xrot,
    Yrot,
    Zrot,
}

impl Chan {
    pub(super) fn parse(s: &str) -> Option<Chan> {
        Some(match s.to_ascii_lowercase().as_str() {
            "xposition" => Chan::Xpos,
            "yposition" => Chan::Ypos,
            "zposition" => Chan::Zpos,
            "xrotation" => Chan::Xrot,
            "yrotation" => Chan::Yrot,
            "zrotation" => Chan::Zrot,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Joint {
    pub name: String,
    pub parent: Option<usize>,
    pub offset: V3,
    pub channels: Vec<Chan>,
    /// Index of this joint's first value in a frame row.
    pub chan_start: usize,
    pub end_site: Option<V3>,
    pub children: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bvh {
    /// Parents always precede their children (file order).
    pub joints: Vec<Joint>,
    /// Seconds per frame.
    pub frame_time: f64,
    pub frames: usize,
    pub num_channels: usize,
    /// `frames * num_channels` values, row per frame.
    pub data: Vec<f64>,
}

pub(super) struct Tok<'a> {
    pub(super) line: usize,
    pub(super) s: &'a str,
}

pub(super) fn tokenize(text: &str) -> Vec<Tok<'_>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut out = Vec::new();
    for (ln, line) in text.lines().enumerate() {
        for sub in line.split('\r') {
            for w in sub.split_whitespace() {
                // braces are separate tokens even when glued to a word
                let mut last = 0;
                for (i, c) in w.char_indices() {
                    if c == '{' || c == '}' {
                        if i > last {
                            out.push(Tok { line: ln + 1, s: &w[last..i] });
                        }
                        out.push(Tok { line: ln + 1, s: &w[i..i + 1] });
                        last = i + 1;
                    }
                }
                if last < w.len() {
                    out.push(Tok { line: ln + 1, s: &w[last..] });
                }
            }
        }
    }
    out
}

pub(super) struct Parser<'a> {
    pub(super) t: Vec<Tok<'a>>,
    pub(super) i: usize,
    pub(super) joints: Vec<Joint>,
    pub(super) nch: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn last_line(&self) -> usize {
        self.t.last().map_or(1, |t| t.line)
    }
    pub(super) fn peek(&self) -> Option<&Tok<'a>> {
        self.t.get(self.i)
    }
    pub(super) fn next(&mut self, ctx: &str) -> Result<(usize, &'a str), String> {
        match self.t.get(self.i) {
            Some(t) => {
                self.i += 1;
                Ok((t.line, t.s))
            }
            None => Err(format!("line {}: unexpected end of file while reading {ctx}", self.last_line())),
        }
    }
    pub(super) fn expect(&mut self, word: &str) -> Result<usize, String> {
        let (l, s) = self.next(word)?;
        if s.eq_ignore_ascii_case(word) {
            Ok(l)
        } else {
            Err(format!("line {l}: expected '{word}', found '{s}'"))
        }
    }
    pub(super) fn num(&mut self, ctx: &str) -> Result<f64, String> {
        let (l, s) = self.next(ctx)?;
        let v: f64 = s.parse().map_err(|_| format!("line {l}: '{s}' is not a number ({ctx})"))?;
        if !v.is_finite() {
            return Err(format!("line {l}: '{s}' is not a finite number ({ctx})"));
        }
        Ok(v)
    }
    pub(super) fn offset(&mut self) -> Result<V3, String> {
        self.expect("OFFSET")?;
        Ok([self.num("OFFSET x")?, self.num("OFFSET y")?, self.num("OFFSET z")?])
    }
    pub(super) fn joint(&mut self, parent: Option<usize>, kw_line: usize, depth: usize) -> Result<(), String> {
        if depth > MAX_DEPTH {
            return Err(format!("line {kw_line}: hierarchy nested deeper than {MAX_DEPTH}"));
        }
        if self.joints.len() >= MAX_JOINTS {
            return Err(format!("line {kw_line}: more than {MAX_JOINTS} joints"));
        }
        let mut name = Vec::new();
        while let Some(t) = self.peek() {
            if t.line != kw_line || t.s == "{" || t.s == "}" {
                break;
            }
            name.push(t.s);
            self.i += 1;
        }
        if name.is_empty() {
            return Err(format!("line {kw_line}: joint has no name"));
        }
        let name = name.join(" ");
        let idx = self.joints.len();
        self.joints.push(Joint { name: name.clone(), parent, offset: [0.0; 3], channels: vec![], chan_start: 0, end_site: None, children: vec![] });
        if let Some(p) = parent {
            self.joints[p].children.push(idx);
        }
        self.expect("{")?;
        let mut have_offset = false;
        loop {
            let (l, s) = self.next(&format!("joint '{name}'"))?;
            match s.to_ascii_lowercase().as_str() {
                "offset" => {
                    self.i -= 1;
                    self.joints[idx].offset = self.offset()?;
                    have_offset = true;
                }
                "channels" => {
                    let (nl, ns) = self.next("channel count")?;
                    let n: usize = ns.parse().map_err(|_| format!("line {nl}: '{ns}' is not a channel count"))?;
                    if n > 24 {
                        return Err(format!("line {nl}: {n} channels on one joint is not plausible"));
                    }
                    let mut ch = Vec::with_capacity(n);
                    for _ in 0..n {
                        let (cl, cs) = self.next("channel name")?;
                        ch.push(Chan::parse(cs).ok_or_else(|| format!("line {cl}: unknown channel '{cs}'"))?);
                    }
                    self.joints[idx].chan_start = self.nch;
                    self.nch += n;
                    self.joints[idx].channels = ch;
                }
                "joint" => self.joint(Some(idx), l, depth + 1)?,
                "end" => {
                    let sl = self.expect("Site")?;
                    // optional name on the same line is ignored
                    while self.peek().is_some_and(|t| t.line == sl && t.s != "{") {
                        self.i += 1;
                    }
                    self.expect("{")?;
                    let off = self.offset()?;
                    self.expect("}")?;
                    self.joints[idx].end_site = Some(off);
                }
                "}" => {
                    if !have_offset {
                        return Err(format!("line {l}: joint '{name}' has no OFFSET"));
                    }
                    return Ok(());
                }
                _ => return Err(format!("line {l}: unexpected '{s}' inside joint '{name}'")),
            }
        }
    }
    /// `Frames: N` / `Frame Time: T`, tolerant of spacing around the colon.
    pub(super) fn labeled(&mut self, words: &[&str]) -> Result<(usize, &'a str), String> {
        let mut value: Option<(usize, &'a str)> = None;
        for (wi, w) in words.iter().enumerate() {
            let (l, s) = self.next(&format!("'{}'", words.join(" ")))?;
            let lower = s.to_ascii_lowercase();
            if !lower.starts_with(w) {
                return Err(format!("line {l}: expected '{}:', found '{s}'", words.join(" ")));
            }
            let rest = &s[w.len()..];
            if wi + 1 == words.len() {
                let rest = rest.strip_prefix(':').unwrap_or(rest);
                if !rest.is_empty() {
                    value = Some((l, rest));
                } else if self.peek().is_some_and(|t| t.s.starts_with(':')) {
                    let (nl, ns) = self.next("value")?;
                    if ns.len() > 1 {
                        value = Some((nl, &ns[1..]));
                    }
                }
            }
        }
        match value {
            Some(v) => Ok(v),
            None => self.next("value"),
        }
    }
}

/// Parse BVH text.
pub fn parse_bvh(text: &str) -> Result<Bvh, String> {
    let mut p = Parser { t: tokenize(text), i: 0, joints: vec![], nch: 0 };
    if p.t.is_empty() {
        return Err("line 1: empty file".into());
    }
    p.expect("HIERARCHY")?;
    let mut roots = 0;
    loop {
        match p.peek() {
            None => return Err(format!("line {}: missing MOTION section", p.last_line())),
            Some(t) if t.s.eq_ignore_ascii_case("root") => {
                let l = t.line;
                p.i += 1;
                p.joint(None, l, 0)?;
                roots += 1;
            }
            Some(t) if t.s.eq_ignore_ascii_case("motion") => {
                if roots == 0 {
                    return Err(format!("line {}: no ROOT before MOTION", t.line));
                }
                p.i += 1;
                break;
            }
            Some(t) => return Err(format!("line {}: expected ROOT or MOTION, found '{}'", t.line, t.s)),
        }
    }
    let (fl, fv) = p.labeled(&["frames"])?;
    let frames: usize = fv.parse().map_err(|_| format!("line {fl}: '{fv}' is not a frame count"))?;
    if frames > MAX_FRAMES {
        return Err(format!("line {fl}: {frames} frames exceeds the limit of {MAX_FRAMES}"));
    }
    let (tl, tv) = p.labeled(&["frame", "time"])?;
    let frame_time: f64 = tv.parse().map_err(|_| format!("line {tl}: '{tv}' is not a frame time"))?;
    if !(frame_time.is_finite() && frame_time > 0.0) {
        return Err(format!("line {tl}: frame time must be positive, got '{tv}'"));
    }
    let nch = p.nch;
    let expected = frames.checked_mul(nch).filter(|&n| n <= MAX_VALUES).ok_or("motion data too large")?;
    let rest = &p.t[p.i..];
    if rest.len() < expected {
        let l = rest.last().map_or(tl, |t| t.line);
        return Err(format!("line {l}: motion data has {} values, expected {expected} ({frames} frames x {nch} channels)", rest.len()));
    }
    if rest.len() > expected {
        return Err(format!("line {}: extra data after {frames} frames", rest[expected].line));
    }
    let mut data = Vec::with_capacity(expected);
    for t in rest {
        let v: f64 = t.s.parse().map_err(|_| format!("line {}: '{}' is not a number", t.line, t.s))?;
        if !v.is_finite() {
            return Err(format!("line {}: '{}' is not a finite number", t.line, t.s));
        }
        data.push(v);
    }
    Ok(Bvh { joints: p.joints, frame_time, frames, num_channels: nch, data })
}
