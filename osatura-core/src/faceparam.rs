// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Face landmarks -> rig signals. Fully local.
//!
//! This module itself contains no detector. OSATURA's own detector ([`crate::facedet`], trained
//! on procedural synthetic faces only, stylised / cartoon faces, **not** validated on photographs or
//! webcams) writes the landmark format below; any detector you installed yourself can too. This module starts
//! one step later: it takes 2D face landmarks that *something else* produced (a detector you
//! installed yourself, a hand-made file, a synthetic test sequence), computes textbook geometric
//! measurements from them, calibrates them per user, smooths them, and hands the result to the
//! existing live-input mapping ([`crate::liveinput`]) as events of source [`InputSource::Face`].
//! Mapping onto parameters, bone angles and switch layers, triggers, locks and baking therefore
//! reuse [`crate::liveinput::bake`] unchanged.
//!
//! # Landmark stream format (NDJSON, "osatura-landmarks" v1)
//!
//! One JSON value per line; blank lines and lines starting with `#` are ignored.
//!
//! ```text
//! {"format":"osatura-landmarks","version":1,"layout":"ibug68","fps":30,"size":[640,480]}
//! {"t":0.000,"p":[[0.31,0.42],[0.32,0.47], ...]}
//! {"t":0.033,"p":[{"x":0.31,"y":0.42}, ...]}
//! {"frame":2,"p":[0.31,0.42,0.32,0.47, ...]}
//! {"t":0.100,"named":{"nose_tip":[0.5,0.5], ...}}
//! {"t":0.133,"p":null}
//! ```
//!
//! * The header line is optional. `layout` is `"osatura23"` (own compact layout, the points in
//!   the order of [`NAMES`]), `"ibug68"` (the common 68-point layout), `"mesh468"` (the 468-point
//!   dense mesh layout; 478-point files with extra iris points are accepted, the extras are
//!   ignored) or an object `{"points":{"r_eye_1":36, ...}}` giving every name of [`NAMES`] an index.
//!   Without a layout it is inferred from the point count (23, 68, 468, 478).
//! * `size` `[w,h]`: coordinates are multiplied by it first, so normalised 0..1 coordinates
//!   are measured in isotropic units. Omit it when the coordinates are already pixels (or square).
//! * A point is `[x,y]`, `[x,y,z]` (z ignored) or `{"x":..,"y":..}`; `p` may also be a flat
//!   `[x0,y0,x1,y1,...]` list. y grows downwards (image convention).
//! * Time: `t` seconds, else `frame` / `fps`, else the running line number / `fps`.
//! * `p: null`, a non-finite coordinate or too few points = tracking lost for that frame: no
//!   event is produced and the previous value is held when baked.
//! * The importer is tolerant: bad lines become warnings and are skipped. It fails only when no
//!   usable frame remains or the header is unusable.
//!
//! The 68/468 index tables below are written from the publicly documented layouts. They are
//! data: if your detector differs, give an explicit `points` object instead.
//!
//! # Geometry (all dimensionless, scale invariant)
//!
//! `r_` = the subject's right side (image left in an unmirrored picture). Set `mirror` for a
//! selfie-mirrored picture (x is negated before measuring).
//!
//! * `mirror`: the picture is a selfie mirror view but the landmark labels are anatomical (`r_` is the
//!   subject's right, which then appears on the image right). Negating x restores the geometry below.
//! * roll = angle of the line from the right to the left eye centre, degrees, positive clockwise on screen.
//! * yaw: nose-tip offset from the midpoint of the two face sides, measured along the eye line and
//!   divided by half the face width, `t = offset / yaw_depth`, angle = atan(t - t_neutral) (a nose
//!   standing `yaw_depth` half-widths in front of the face plane). Positive = nose moves towards +x.
//! * pitch: nose tip position between the eye-line midpoint and the chin along the face's vertical
//!   axis, `r`; `t = (0.5 - r) / pitch_depth`, angle = atan(t - t_neutral). Positive = looking up.
//!   Yaw and pitch use an average-head depth guess; they are meaningful *relative to the calibrated neutral pose*.
//! * eye aspect ratio, Soukupova and Cech 2016: (|p2-p6| + |p3-p5|) / (2 |p1-p4|).
//! * mouth aspect ratio = inner-lip gap / inner mouth width; width ratio = mouth width / face width.
//! * brow raise = (eye centre - brow point) along the face's vertical axis / inter-eye distance.
//!
//! These are 2D approximations, not a 3D head pose. They degrade at large angles.
//!
//! # Signals
//!
//! Eleven [`SIGNALS`], each 0..1 after calibration, smoothed by a One-Euro filter (Casiez, Roussel,
//! Vogel 2012) and a dead zone. Head angles are bipolar: 0.5 = neutral, 0 and 1 = the calibrated
//! (or default) negative/positive span, so a `ParamMap` with `out_min=-S, out_max=S` gives degrees
//! when S equals the span ([`FaceConfig::span_deg`]).
//!
//! Nothing here was run on a real webcam or a real detector; tests use synthetic geometry.

use crate::liveinput::{InputEvent, InputSource, Mapping};
use crate::math::Vec2;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

/// Canonical landmark names. A [`Face`] holds one point per name in this order. Eye points follow
/// the EAR paper's p1..p6: p1 and p4 are the corners, p2/p3 the upper lid and p6/p5 the lower lid,
/// listed left to right in an unmirrored picture (right eye: p1 outer; left eye: p1 inner).
pub const NAMES: [&str; 23] = [
    "r_eye_1",
    "r_eye_2",
    "r_eye_3",
    "r_eye_4",
    "r_eye_5",
    "r_eye_6", //
    "l_eye_1",
    "l_eye_2",
    "l_eye_3",
    "l_eye_4",
    "l_eye_5",
    "l_eye_6", //
    "mouth_r",
    "mouth_l",
    "mouth_top",
    "mouth_bot", // corners, inner-lip centres
    "r_brow",
    "l_brow", // middle of each eyebrow
    "nose_tip",
    "nose_bridge",
    "chin",
    "face_r",
    "face_l", // face sides at eye level
];
const N: usize = NAMES.len();

/// Point indices per name ([`NAMES`] order) for the common 68-point layout (0-based).
pub const IBUG68: [usize; N] = [
    36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 54, 62, 66, 19, 24, 30, 27, 8, 0, 16,
];
/// Same for the 468-point dense mesh layout.
pub const MESH468: [usize; N] = [
    33, 160, 158, 133, 153, 144, 362, 385, 387, 263, 373, 380, 61, 291, 13, 14, 105, 334, 1, 168,
    152, 234, 454,
];

pub type Face = [Vec2; N];

/// How the points of a frame are numbered.
#[derive(Clone, Debug, PartialEq)]
pub enum Layout {
    Osatura23,
    Ibug68,
    Mesh468,
    Custom(Vec<usize>),
}

impl Layout {
    pub fn parse(s: &str) -> Result<Layout, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "osatura23" | "23" => Ok(Layout::Osatura23),
            "ibug68" | "68" => Ok(Layout::Ibug68),
            "mesh468" | "468" | "478" | "mesh478" => Ok(Layout::Mesh468),
            o => Err(format!(
                "unknown layout '{o}' (osatura23, ibug68, mesh468 or an explicit points object)"
            )),
        }
    }
    pub fn from_json(v: &Value) -> Result<Layout, String> {
        match v {
            Value::String(s) => Layout::parse(s),
            Value::Object(o) => {
                let pts = o
                    .get("points")
                    .and_then(|p| p.as_object())
                    .ok_or("a layout object needs a points object")?;
                let mut idx = Vec::with_capacity(N);
                for n in NAMES {
                    let i = pts
                        .get(n)
                        .and_then(|x| x.as_u64())
                        .ok_or_else(|| format!("layout points: missing index for '{n}'"))?;
                    if i > 100_000 {
                        return Err(format!("layout points: index for '{n}' is too large"));
                    }
                    idx.push(i as usize);
                }
                Ok(Layout::Custom(idx))
            }
            _ => Err("layout must be a name or an object".into()),
        }
    }
    pub fn infer(count: usize) -> Option<Layout> {
        match count {
            23 => Some(Layout::Osatura23),
            68 => Some(Layout::Ibug68),
            468 | 478 => Some(Layout::Mesh468),
            _ => None,
        }
    }
    fn table(&self) -> Vec<usize> {
        match self {
            Layout::Osatura23 => (0..N).collect(),
            Layout::Ibug68 => IBUG68.to_vec(),
            Layout::Mesh468 => MESH468.to_vec(),
            Layout::Custom(v) => v.clone(),
        }
    }
    /// Smallest point count this layout needs.
    pub fn min_points(&self) -> usize {
        self.table().into_iter().max().unwrap_or(0) + 1
    }
    /// Pick the 23 named points out of a frame's points.
    pub fn select(&self, pts: &[Vec2]) -> Result<Face, String> {
        if pts.len() < self.min_points() {
            return Err(format!(
                "{} points, the layout needs at least {}",
                pts.len(),
                self.min_points()
            ));
        }
        let t = self.table();
        let mut f = [Vec2::new(0.0, 0.0); N];
        for (i, &k) in t.iter().enumerate() {
            f[i] = pts[k];
        }
        Ok(f)
    }
}

/// One frame of a stream. `face` is `None` when tracking was lost.
#[derive(Clone, Debug, PartialEq)]
pub struct LandmarkFrame {
    pub t: f32,
    pub face: Option<Face>,
}

#[derive(Clone, Debug, Default)]
pub struct LandmarkStream {
    pub frames: Vec<LandmarkFrame>,
    pub warnings: Vec<String>,
    pub fps: Option<f32>,
}

pub const MAX_FRAMES: usize = 1_000_000;
const MAX_WARNINGS: usize = 50;

struct Ctx {
    layout: Option<Layout>,
    size: (f32, f32),
}

fn num(v: &Value) -> Option<f32> {
    v.as_f64().map(|x| x as f32).filter(|x| x.is_finite())
}

/// `None` = a coordinate is missing or not finite.
fn point(v: &Value) -> Option<[f32; 2]> {
    match v {
        Value::Array(a) if a.len() >= 2 => Some([num(&a[0])?, num(&a[1])?]),
        Value::Object(o) => Some([num(o.get("x")?)?, num(o.get("y")?)?]),
        _ => None,
    }
}

fn points_of(p: &Value, size: (f32, f32)) -> Result<Option<Vec<Vec2>>, String> {
    let sc = |a: [f32; 2]| Vec2::new(a[0] * size.0, a[1] * size.1);
    let arr = match p {
        Value::Null => return Ok(None),
        Value::Array(a) => a,
        _ => return Err("p must be an array or null".into()),
    };
    if !arr.is_empty() && arr.iter().all(|x| x.is_number()) {
        if arr.len() % 2 != 0 {
            return Err("a flat p list needs an even number of values".into());
        }
        let mut out = Vec::with_capacity(arr.len() / 2);
        for c in arr.chunks(2) {
            match (num(&c[0]), num(&c[1])) {
                (Some(x), Some(y)) => out.push(sc([x, y])),
                _ => return Ok(None),
            }
        }
        return Ok(Some(out));
    }
    let mut out = Vec::with_capacity(arr.len());
    for x in arr {
        match point(x) {
            Some(q) => out.push(sc(q)),
            None => return Ok(None), // non-finite coordinate: lost
        }
    }
    Ok(Some(out))
}

fn named_face(
    o: &serde_json::Map<String, Value>,
    size: (f32, f32),
) -> Result<Option<Face>, String> {
    let mut f = [Vec2::new(0.0, 0.0); N];
    for (i, n) in NAMES.iter().enumerate() {
        let v = o
            .get(*n)
            .ok_or_else(|| format!("named point '{n}' is missing"))?;
        match point(v) {
            Some(q) => f[i] = Vec2::new(q[0] * size.0, q[1] * size.1),
            None => return Ok(None),
        }
    }
    Ok(Some(f))
}

/// Parse one frame object (a stream line, or one entry of a studio push). `index` = running number.
fn parse_frame(v: &Value, c: &mut Ctx, fps: f32, index: usize) -> Result<LandmarkFrame, String> {
    let wrapped;
    let o = match v {
        Value::Object(o) => o,
        Value::Array(_) => {
            wrapped = serde_json::json!({ "p": v });
            wrapped.as_object().unwrap()
        }
        _ => return Err("a frame must be an object or an array of points".into()),
    };
    let t = if let Some(t) = o.get("t") {
        t.as_f64()
            .filter(|x| x.is_finite())
            .ok_or("t must be a finite number")? as f32
    } else if let Some(fr) = o.get("frame") {
        fr.as_f64()
            .filter(|x| x.is_finite())
            .ok_or("frame must be a finite number")? as f32
            / fps
    } else {
        index as f32 / fps
    };
    if !(0.0..=7.0 * 86400.0).contains(&t) {
        return Err(format!("time {t} is outside 0..7 days"));
    }
    let face = if let Some(n) = o.get("named") {
        named_face(n.as_object().ok_or("named must be an object")?, c.size)?
    } else if let Some(p) = o.get("p") {
        match points_of(p, c.size)? {
            None => None,
            Some(pts) => {
                if c.layout.is_none() {
                    c.layout = Some(Layout::infer(pts.len()).ok_or_else(|| {
                        format!(
                            "cannot infer a layout from {} points (23, 68, 468, 478 are known)",
                            pts.len()
                        )
                    })?);
                }
                {
                    let f = c.layout.as_ref().unwrap().select(&pts)?;
                    Some(f)
                }
            }
        }
    } else {
        return Err("a frame needs p or named".into());
    };
    Ok(LandmarkFrame { t, face })
}

/// Tolerant NDJSON importer. `default_fps` is used for frames without a time (a header `fps` wins),
/// `layout` overrides header and inference.
pub fn parse_ndjson(
    text: &str,
    default_fps: f32,
    layout: Option<Layout>,
) -> Result<LandmarkStream, String> {
    if (default_fps.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater))
        || !default_fps.is_finite()
    {
        return Err("fps must be a positive number".into());
    }
    let mut c = Ctx {
        layout: layout.clone(),
        size: (1.0, 1.0),
    };
    let (mut fps, mut header_fps) = (default_fps, None);
    let mut out = LandmarkStream::default();
    let (mut seen_data, mut bad_total) = (false, 0usize);
    let warn = |out: &mut LandmarkStream, s: String| {
        if out.warnings.len() < MAX_WARNINGS {
            out.warnings.push(s);
        }
    };
    let mut idx = 0usize;
    for (ln, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let v: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                bad_total += 1;
                warn(
                    &mut out,
                    format!("line {}: not JSON ({e}), skipped", ln + 1),
                );
                continue;
            }
        };
        if !seen_data && v.get("format").is_some() {
            if v["format"] != "osatura-landmarks" {
                return Err(format!("line {}: unknown format {}", ln + 1, v["format"]));
            }
            if let Some(ver) = v.get("version").and_then(|x| x.as_u64()) {
                if ver != 1 {
                    return Err(format!("unsupported landmark stream version {ver}"));
                }
            }
            if let Some(l) = v.get("layout") {
                let l = Layout::from_json(l).map_err(|e| format!("line {}: {e}", ln + 1))?;
                if layout.is_none() {
                    c.layout = Some(l);
                }
            }
            if let Some(f) = v.get("fps") {
                let f = num(f)
                    .filter(|f| *f > 0.0)
                    .ok_or_else(|| format!("line {}: header fps must be positive", ln + 1))?;
                header_fps = Some(f);
                fps = f;
            }
            if let Some(s) = v.get("size") {
                let a = s
                    .as_array()
                    .filter(|a| a.len() == 2)
                    .ok_or_else(|| format!("line {}: size must be [w,h]", ln + 1))?;
                match (num(&a[0]), num(&a[1])) {
                    (Some(w), Some(h)) if w > 0.0 && h > 0.0 => c.size = (w, h),
                    _ => {
                        return Err(format!(
                            "line {}: size must be two positive numbers",
                            ln + 1
                        ))
                    }
                }
            }
            continue;
        }
        seen_data = true;
        if out.frames.len() >= MAX_FRAMES {
            return Err(format!("more than {MAX_FRAMES} frames"));
        }
        match parse_frame(&v, &mut c, fps, idx) {
            Ok(f) => out.frames.push(f),
            Err(e) => {
                bad_total += 1;
                warn(&mut out, format!("line {}: {e}, skipped", ln + 1));
            }
        }
        idx += 1;
    }
    if out.frames.is_empty() {
        return Err(match out.warnings.first() {
            Some(w) => format!("no usable landmark frame ({w})"),
            None => "no landmark frames in the input".into(),
        });
    }
    if bad_total > MAX_WARNINGS {
        out.warnings
            .push(format!("{} lines were skipped in total", bad_total));
    }
    out.fps = header_fps;
    let mut unsorted = false;
    for w in out.frames.windows(2) {
        if w[1].t < w[0].t {
            unsorted = true;
        }
    }
    if unsorted {
        out.frames.sort_by(|a, b| a.t.total_cmp(&b.t));
        out.warnings
            .push("frames were not in time order and were sorted".into());
    }
    Ok(out)
}

/// Parse studio-style frame objects (already decoded JSON).
pub fn parse_frames(
    frames: &[Value],
    layout: Option<Layout>,
    size: Option<(f32, f32)>,
    fps: f32,
) -> Result<Vec<LandmarkFrame>, String> {
    let mut c = Ctx {
        layout,
        size: size.unwrap_or((1.0, 1.0)),
    };
    let mut out = Vec::new();
    for (i, f) in frames.iter().enumerate() {
        out.push(parse_frame(f, &mut c, fps, i).map_err(|e| format!("frame {i}: {e}"))?);
    }
    Ok(out)
}

// ---- geometry ------------------------------------------------------------------------------

/// Raw measurement keys, in order. `yaw`/`pitch` are in tangent space (see the module doc).
pub const RAW_KEYS: [&str; 9] = [
    "yaw", "pitch", "roll", "ear_r", "ear_l", "mar", "width", "brow_r", "brow_l",
];
pub type Raw = [f32; 9];

fn mid(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5)
}
fn dist(a: Vec2, b: Vec2) -> f32 {
    (a - b).length()
}
fn eye_centre(f: &Face, o: usize) -> Vec2 {
    let mut s = Vec2::new(0.0, 0.0);
    for i in 0..6 {
        s += f[o + i];
    }
    s * (1.0 / 6.0)
}
/// Eye aspect ratio of the six points starting at `o`.
fn ear(f: &Face, o: usize) -> Option<f32> {
    let w = dist(f[o], f[o + 3]);
    if w < 1e-6 {
        return None;
    }
    Some((dist(f[o + 1], f[o + 5]) + dist(f[o + 2], f[o + 4])) / (2.0 * w))
}

/// Geometry parameters of the measurement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Geometry {
    /// Negate x first (the picture is a selfie mirror view).
    pub mirror: bool,
    /// Nose-tip standing distance in front of the face plane, in half face widths (yaw).
    pub yaw_depth: f32,
    /// Same relative to the eye-to-chin distance (pitch).
    pub pitch_depth: f32,
}
impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            mirror: false,
            yaw_depth: 0.30,
            pitch_depth: 0.22,
        }
    }
}

/// Raw measurements of one face; `None` for degenerate geometry (coincident eyes, zero face width...).
pub fn measure(face: &Face, g: &Geometry) -> Option<Raw> {
    let mut f = *face;
    if g.mirror {
        for p in f.iter_mut() {
            p.x = -p.x;
        }
    }
    let rc = eye_centre(&f, 0);
    let lc = eye_centre(&f, 6);
    let iod = dist(rc, lc);
    let fw = dist(f[21], f[22]);
    if iod < 1e-6 || fw < 1e-6 || g.yaw_depth <= 0.0 || g.pitch_depth <= 0.0 {
        return None;
    }
    let u = (lc - rc) * (1.0 / iod); // along the eye line
    let v = Vec2::new(-u.y, u.x); // face "down"
    let roll = (lc.y - rc.y).atan2(lc.x - rc.x).to_degrees();
    // yaw: nose offset from the face-side midpoint along u, over half the face width
    let side_mid = mid(f[21], f[22]);
    let yaw_t = (f[18] - side_mid).dot(u) / (fw * 0.5) / g.yaw_depth;
    // pitch: nose between the eye-line midpoint and the chin along v
    let em = mid(rc, lc);
    let h = (f[20] - em).dot(v);
    if h.abs() < 1e-6 {
        return None;
    }
    let r = (f[18] - em).dot(v) / h;
    let pitch_t = (0.5 - r) / g.pitch_depth;
    let (ear_r, ear_l) = (ear(&f, 0)?, ear(&f, 6)?);
    let mw = dist(f[12], f[13]);
    if mw < 1e-6 {
        return None;
    }
    let mar = dist(f[14], f[15]) / mw;
    let width = mw / fw;
    let brow_r = (rc - f[16]).dot(v) / iod;
    let brow_l = (lc - f[17]).dot(v) / iod;
    let raw = [
        yaw_t, pitch_t, roll, ear_r, ear_l, mar, width, brow_r, brow_l,
    ];
    raw.iter().all(|x| x.is_finite()).then_some(raw)
}

// ---- calibration ---------------------------------------------------------------------------

/// Per-user reference: the neutral pose and the observed range of each raw measurement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    pub neutral: BTreeMap<String, f32>,
    pub lo: BTreeMap<String, f32>,
    pub hi: BTreeMap<String, f32>,
}

fn pick(m: &BTreeMap<String, f32>, what: &str) -> Result<Raw, String> {
    let mut r = [0.0; 9];
    for (i, k) in RAW_KEYS.iter().enumerate() {
        r[i] = *m
            .get(*k)
            .ok_or_else(|| format!("calibration {what}: missing '{k}'"))?;
        if !r[i].is_finite() {
            return Err(format!("calibration {what}: '{k}' is not finite"));
        }
    }
    Ok(r)
}
fn to_map(r: &Raw) -> BTreeMap<String, f32> {
    RAW_KEYS
        .iter()
        .zip(r.iter())
        .map(|(k, v)| (k.to_string(), *v))
        .collect()
}

fn percentile(sorted: &[f32], q: f32) -> f32 {
    let x = q * (sorted.len() - 1) as f32;
    let (i, fr) = (x.floor() as usize, x - x.floor());
    let j = (i + 1).min(sorted.len() - 1);
    sorted[i] * (1.0 - fr) + sorted[j] * fr
}

impl Calibration {
    /// Average face, used when no calibration was captured. Only ratios are nominal; angles are 0.
    pub fn nominal() -> Calibration {
        let n: Raw = [0.0, 0.0, 0.0, 0.28, 0.28, 0.03, 0.40, 0.30, 0.30];
        Calibration {
            neutral: to_map(&n),
            lo: to_map(&n),
            hi: to_map(&n),
        }
    }

    /// Neutral = per-key median of `neutral_rows`; lo/hi = 2nd/98th percentile of `range_rows`
    /// (the neutral rows when empty). Needs at least 3 neutral rows.
    pub fn from_rows(neutral_rows: &[Raw], range_rows: &[Raw]) -> Result<Calibration, String> {
        if neutral_rows.len() < 3 {
            return Err(format!(
                "calibration needs at least 3 usable neutral frames, got {}",
                neutral_rows.len()
            ));
        }
        let range = if range_rows.is_empty() {
            neutral_rows
        } else {
            range_rows
        };
        let (mut n, mut lo, mut hi) = ([0.0; 9], [0.0; 9], [0.0; 9]);
        for k in 0..9 {
            let mut a: Vec<f32> = neutral_rows.iter().map(|r| r[k]).collect();
            a.sort_by(|x, y| x.total_cmp(y));
            n[k] = percentile(&a, 0.5);
            let mut b: Vec<f32> = range.iter().map(|r| r[k]).collect();
            b.sort_by(|x, y| x.total_cmp(y));
            lo[k] = percentile(&b, 0.02);
            hi[k] = percentile(&b, 0.98);
        }
        Ok(Calibration {
            neutral: to_map(&n),
            lo: to_map(&lo),
            hi: to_map(&hi),
        })
    }

    pub fn from_json(s: &str) -> Result<Calibration, String> {
        let c: Calibration =
            serde_json::from_str(s).map_err(|e| format!("bad calibration: {e}"))?;
        c.arrays()?;
        Ok(c)
    }

    fn arrays(&self) -> Result<(Raw, Raw, Raw), String> {
        Ok((
            pick(&self.neutral, "neutral")?,
            pick(&self.lo, "lo")?,
            pick(&self.hi, "hi")?,
        ))
    }
}

// ---- signals -------------------------------------------------------------------------------

pub const SIGNALS: [&str; 11] = [
    "head_yaw",
    "head_pitch",
    "head_roll",
    "eye_r_open",
    "eye_l_open",
    "mouth_open",
    "mouth_wide",
    "mouth_round",
    "brow_r_raise",
    "brow_l_raise",
    "brow_raise",
];
const BIPOLAR: usize = 3; // the first three signals

/// Smoothing and shaping.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FaceConfig {
    pub geometry: Geometry,
    /// One-Euro minimum cutoff (Hz): smoothing at rest. Lower = smoother and laggier.
    pub mincutoff: f32,
    /// One-Euro speed coefficient: higher = less lag during fast moves.
    pub beta: f32,
    /// One-Euro cutoff for the derivative (Hz).
    pub dcutoff: f32,
    /// Fraction of the 0..1 range treated as zero near the rest value, 0..1.
    pub deadzone: f32,
    /// Half-range in degrees of yaw, pitch, roll when the calibration cannot tell.
    pub span_deg: [f32; 3],
    /// Pre-set `(mincutoff, beta)` per signal name.
    pub per_signal: BTreeMap<String, (f32, f32)>,
}
impl Default for FaceConfig {
    fn default() -> Self {
        FaceConfig {
            geometry: Geometry::default(),
            mincutoff: 1.5,
            beta: 1.0,
            dcutoff: 1.0,
            deadzone: 0.02,
            span_deg: [35.0, 25.0, 30.0],
            per_signal: BTreeMap::new(),
        }
    }
}
impl FaceConfig {
    pub fn validate(&self) -> Result<(), String> {
        let g = &self.geometry;
        if !(g.yaw_depth > 0.0 && g.yaw_depth <= 4.0 && g.pitch_depth > 0.0 && g.pitch_depth <= 4.0)
        {
            return Err("yaw_depth and pitch_depth must be within (0, 4]".into());
        }
        if !(self.mincutoff > 0.0
            && self.mincutoff <= 1000.0
            && self.dcutoff > 0.0
            && self.dcutoff <= 1000.0
            && self.beta >= 0.0
            && self.beta <= 1.0e6)
        {
            return Err("mincutoff and dcutoff must be positive, beta non-negative".into());
        }
        if !(0.0..1.0).contains(&self.deadzone) {
            return Err("deadzone must be within 0..1 (1 excluded)".into());
        }
        if !self
            .span_deg
            .iter()
            .all(|s| s.is_finite() && *s > 1.0 && *s < 90.0)
        {
            return Err("span_deg entries must be within (1, 90) degrees".into());
        }
        for (k, (m, b)) in &self.per_signal {
            if !SIGNALS.contains(&k.as_str()) {
                return Err(format!("per_signal: unknown signal '{k}'"));
            }
            if !(*m > 0.0 && m.is_finite() && *b >= 0.0 && b.is_finite()) {
                return Err(format!(
                    "per_signal {k}: mincutoff must be positive and beta non-negative"
                ));
            }
        }
        Ok(())
    }
}

fn unit(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}
/// 0 at `a`, 1 at `b` (either order); degenerate ranges (|b-a| < `min`) use `fallback_b`.
fn ramp(x: f32, a: f32, b: f32, min: f32, fallback_b: f32) -> f32 {
    let b = if (b - a).abs() < min { fallback_b } else { b };
    unit((x - a) / (b - a))
}

/// Calibrated, unfiltered 0..1 value of every signal (bipolar ones centred on 0.5).
pub fn normalise(raw: &Raw, cal: &Calibration, cfg: &FaceConfig) -> Result<[f32; 11], String> {
    let (n, lo, hi) = cal.arrays()?;
    let mut o = [0.0f32; 11];
    // angles
    for k in 0..3 {
        let ang = |x: f32| {
            if k < 2 {
                (x - n[k]).atan().to_degrees()
            } else {
                x - n[k]
            }
        };
        let (a_lo, a_hi) = (ang(lo[k]), ang(hi[k]));
        let mut span = a_lo.abs().max(a_hi.abs());
        if span < 3.0 {
            span = cfg.span_deg[k];
        }
        o[k] = unit(0.5 + 0.5 * ang(raw[k]) / span);
    }
    let (ear_r, ear_l, mar, width, br, bl) = (raw[3], raw[4], raw[5], raw[6], raw[7], raw[8]);
    // eyes: 1 at the neutral (open) value, 0 at the lowest observed (closed); when the calibration
    // saw no blink (range < 0.02) the closed value is assumed to be 25 % of neutral
    let eye = |e: f32, nv: f32, lov: f32| {
        let closed = if nv - lov < 0.02 { 0.25 * nv } else { lov };
        unit((e - closed) / (nv - closed))
    };
    o[3] = eye(ear_r, n[3], lo[3]);
    o[4] = eye(ear_l, n[4], lo[4]);
    // mouth
    o[5] = ramp(mar, n[5], hi[5], 0.05, n[5] + 0.6);
    o[6] = ramp(width, n[6], hi[6], 0.02, n[6] * 1.25);
    o[7] = ramp(width, n[6], lo[6], 0.02, n[6] * 0.7);
    // brows
    o[8] = ramp(br, n[7], hi[7], 0.03, n[7] + 0.15);
    o[9] = ramp(bl, n[8], hi[8], 0.03, n[8] + 0.15);
    o[10] = 0.5 * (o[8] + o[9]);
    Ok(o)
}

// ---- One-Euro filter -----------------------------------------------------------------------

/// One-Euro filter (Casiez, Roussel, Vogel 2012): a low-pass filter whose cutoff rises with the speed.
#[derive(Clone, Debug, Default)]
pub struct OneEuro {
    prev: Option<(f32, f32, f32)>, // x_hat, dx_hat, t
}

fn alpha(cutoff: f32, dt: f32) -> f32 {
    let tau = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
    1.0 / (1.0 + tau / dt)
}

impl OneEuro {
    pub fn reset(&mut self) {
        self.prev = None;
    }
    pub fn filter(&mut self, x: f32, t: f32, mincutoff: f32, beta: f32, dcutoff: f32) -> f32 {
        match self.prev {
            None => {
                self.prev = Some((x, 0.0, t));
                x
            }
            Some((xp, dxp, tp)) => {
                let dt = t - tp;
                if dt <= 1e-6 {
                    return xp; // same timestamp: keep the state
                }
                let dx = (x - xp) / dt;
                let dxh = dxp + alpha(dcutoff, dt) * (dx - dxp);
                let cut = mincutoff + beta * dxh.abs();
                let xh = xp + alpha(cut, dt) * (x - xp);
                self.prev = Some((xh, dxh, t));
                xh
            }
        }
    }
}

fn dead(v: f32, dz: f32, bipolar: bool) -> f32 {
    if dz <= 0.0 {
        return v;
    }
    if bipolar {
        let d = (v - 0.5) * 2.0;
        if d.abs() <= dz {
            0.5
        } else {
            0.5 + 0.5 * d.signum() * (d.abs() - dz) / (1.0 - dz)
        }
    } else if v <= dz {
        0.0
    } else {
        (v - dz) / (1.0 - dz)
    }
}

/// Stateful per-frame pipeline: measure -> calibrate -> One-Euro -> dead zone -> events.
pub struct FaceTracker {
    pub cfg: FaceConfig,
    pub cal: Calibration,
    filters: Vec<OneEuro>,
    last_t: Option<f32>,
}

/// A gap longer than this restarts the filters (seconds).
const RESET_GAP: f32 = 0.5;

impl FaceTracker {
    pub fn new(cfg: FaceConfig, cal: Calibration) -> Result<FaceTracker, String> {
        cfg.validate()?;
        cal.arrays()?;
        Ok(FaceTracker {
            cfg,
            cal,
            filters: vec![OneEuro::default(); SIGNALS.len()],
            last_t: None,
        })
    }

    pub fn set_calibration(&mut self, cal: Calibration) -> Result<(), String> {
        cal.arrays()?;
        self.cal = cal;
        self.reset();
        Ok(())
    }

    pub fn reset(&mut self) {
        for f in &mut self.filters {
            f.reset();
        }
        self.last_t = None;
    }

    /// Smoothed signal values of one frame. Empty when the geometry is degenerate.
    pub fn process(&mut self, t: f32, face: &Face) -> Result<Vec<InputEvent>, String> {
        let Some(raw) = measure(face, &self.cfg.geometry) else {
            return Ok(vec![]);
        };
        let norm = normalise(&raw, &self.cal, &self.cfg)?;
        if self.last_t.is_some_and(|lt| t - lt > RESET_GAP) {
            for f in &mut self.filters {
                f.reset();
            }
        }
        self.last_t = Some(t);
        let mut ev = Vec::with_capacity(SIGNALS.len());
        for (i, name) in SIGNALS.iter().enumerate() {
            let (mc, b) = self
                .cfg
                .per_signal
                .get(*name)
                .copied()
                .unwrap_or((self.cfg.mincutoff, self.cfg.beta));
            let y = self.filters[i].filter(norm[i], t, mc, b, self.cfg.dcutoff);
            let y = dead(unit(y), self.cfg.deadzone, i < BIPOLAR);
            ev.push(InputEvent {
                source: InputSource::Face,
                id: name.to_string(),
                value: unit(y),
                time: t.max(0.0),
            });
        }
        Ok(ev)
    }
}

// ---- batch ---------------------------------------------------------------------------------

/// Raw measurements of the usable frames (index, time, raw).
pub fn measure_stream(s: &LandmarkStream, g: &Geometry) -> Vec<(usize, f32, Raw)> {
    s.frames
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            f.face
                .as_ref()
                .and_then(|face| measure(face, g))
                .map(|r| (i, f.t, r))
        })
        .collect()
}

/// Calibrate from a recording: the first `neutral_frames` frames (usable ones counted) are the
/// neutral pose, the whole recording gives the range.
pub fn calibrate(
    s: &LandmarkStream,
    cfg: &FaceConfig,
    neutral_frames: usize,
) -> Result<Calibration, String> {
    cfg.validate()?;
    if neutral_frames < 3 {
        return Err("--calibrate-frames must be at least 3".into());
    }
    let rows = measure_stream(s, &cfg.geometry);
    if rows.len() < neutral_frames.min(3) || rows.is_empty() {
        return Err("too few usable frames to calibrate".into());
    }
    let n = neutral_frames.min(rows.len());
    let all: Vec<Raw> = rows.iter().map(|r| r.2).collect();
    Calibration::from_rows(&all[..n], &all)
}

/// All events of a stream (deterministic).
pub fn events(
    s: &LandmarkStream,
    cfg: &FaceConfig,
    cal: &Calibration,
) -> Result<Vec<InputEvent>, String> {
    let mut tr = FaceTracker::new(cfg.clone(), cal.clone())?;
    let mut out = Vec::new();
    for f in &s.frames {
        if let Some(face) = &f.face {
            out.extend(tr.process(f.t, face)?);
        }
    }
    if out.is_empty() {
        return Err("no frame had usable face geometry".into());
    }
    Ok(out)
}

/// Reject mapping entries on the face source whose id is not a known signal (typo guard).
pub fn check_mapping(m: &Mapping) -> Result<(), String> {
    for (i, p) in m.params.iter().enumerate() {
        if p.source == InputSource::Face && p.id != "*" && !SIGNALS.contains(&p.id.as_str()) {
            return Err(format!(
                "map {i}: unknown face signal '{}' (known: {})",
                p.id,
                SIGNALS.join(", ")
            ));
        }
    }
    for (i, t) in m.triggers.iter().enumerate() {
        if t.source == InputSource::Face && t.id != "*" && !SIGNALS.contains(&t.id.as_str()) {
            return Err(format!("trigger {i}: unknown face signal '{}'", t.id));
        }
    }
    Ok(())
}

/// The mapping file of `face-bake`: a [`Mapping`] plus an optional `face` section (a [`FaceConfig`]).
pub fn parse_mapfile(text: &str) -> Result<(Mapping, FaceConfig), String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("bad map file: {e}"))?;
    let cfg: FaceConfig = match v.get("face") {
        Some(f) => {
            serde_json::from_value(f.clone()).map_err(|e| format!("bad face section: {e}"))?
        }
        None => FaceConfig::default(),
    };
    cfg.validate()?;
    let mut v = v;
    if let Some(o) = v.as_object_mut() {
        o.remove("face"); // the face section was parsed above; the mapping itself rejects every other unknown key
    }
    let m: Mapping = serde_json::from_value(v).map_err(|e| format!("bad mapping: {e}"))?;
    check_mapping(&m)?;
    Ok((m, cfg))
}

/// Bake a landmark stream into the project as keyed channels, through the live-input mapping.
/// Locks are respected and the result is all-or-nothing (see [`crate::liveinput::bake`]).
pub fn bake(
    p: &mut crate::project::Project,
    s: &LandmarkStream,
    m: &Mapping,
    cfg: &FaceConfig,
    cal: &Calibration,
    tol: f32,
) -> Result<crate::liveinput::BakeReport, String> {
    check_mapping(m)?;
    let ev = events(s, cfg, cal)?;
    crate::liveinput::bake(p, &ev, m, None, tol)
}

/// Group helper for tests and the CLI summary: last value per signal.
pub fn last_values(ev: &[InputEvent]) -> HashMap<String, f32> {
    ev.iter().map(|e| (e.id.clone(), e.value)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{Bone, Project, Prop, Source};

    /// C1 regression: an eye with corners 30 px apart and lids at +-5 px has EAR 1/3 (Soukupova
    /// and Cech) in EVERY layout; the old iBUG-68 table swapped the lower-lid points so EAR was 0.4714.
    #[test]
    fn ear_is_the_textbook_value_in_every_layout() {
        // (layout, right-eye p1..p6 point indices, left-eye p1..p6 point indices)
        let cases: Vec<(Layout, [usize; 6], [usize; 6])> = vec![
            (Layout::Osatura23, [0, 1, 2, 3, 4, 5], [6, 7, 8, 9, 10, 11]),
            (
                Layout::Ibug68,
                [36, 37, 38, 39, 40, 41],
                [42, 43, 44, 45, 46, 47],
            ),
            (
                Layout::Mesh468,
                [33, 160, 158, 133, 153, 144],
                [362, 385, 387, 263, 373, 380],
            ),
            (
                Layout::Custom(IBUG68.to_vec()),
                [36, 37, 38, 39, 40, 41],
                [42, 43, 44, 45, 46, 47],
            ),
        ];
        for (layout, r, l) in cases {
            let mut pts = vec![Vec2::new(500.0, 500.0); layout.min_points().max(478)];
            // textbook geometry: p1 left corner, p2/p3 upper lid, p4 right corner, p5 lower right, p6 lower left
            let shape = [
                (0.0, 0.0),
                (10.0, -5.0),
                (20.0, -5.0),
                (30.0, 0.0),
                (20.0, 5.0),
                (10.0, 5.0),
            ];
            for (k, &(x, y)) in shape.iter().enumerate() {
                pts[r[k]] = Vec2::new(x, y + 100.0);
                pts[l[k]] = Vec2::new(x + 100.0, y + 100.0);
            }
            let f = layout.select(&pts).unwrap();
            for o in [0usize, 6] {
                let e = ear(&f, o).unwrap();
                assert!((e - 1.0 / 3.0).abs() < 1e-5, "{layout:?} eye {o}: EAR {e}");
            }
            // a closed eye is 0, a wider lid gap is larger
            for (k, &(x, _)) in shape.iter().enumerate() {
                pts[r[k]] = Vec2::new(x, 100.0);
            }
            let f = layout.select(&pts).unwrap();
            assert!(ear(&f, 0).unwrap().abs() < 1e-6);
        }
    }

    /// Synthetic head: half face width 1, y down, z towards the viewer, orthographic projection.
    #[derive(Clone, Copy)]
    struct Pose {
        yaw: f32,
        pitch: f32,
        roll: f32,
        eye: f32,  // lid gap (width 0.36)
        gap: f32,  // inner lip gap
        mw: f32,   // mouth width
        brow: f32, // brow height above the eye line
    }
    const NEUTRAL: Pose = Pose {
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
        eye: 0.108,
        gap: 0.02,
        mw: 0.8,
        brow: 0.27,
    };

    fn model(p: &Pose) -> Vec<[f32; 3]> {
        let mut v = Vec::new();
        for c in [-0.45f32, 0.45] {
            let h = p.eye * 0.5;
            v.extend([
                [c - 0.18, -0.25, 0.0],
                [c - 0.06, -0.25 - h, 0.0],
                [c + 0.06, -0.25 - h, 0.0],
                [c + 0.18, -0.25, 0.0],
                [c + 0.06, -0.25 + h, 0.0],
                [c - 0.06, -0.25 + h, 0.0],
            ]);
        }
        v.extend([
            [-p.mw / 2.0, 0.55, 0.0],
            [p.mw / 2.0, 0.55, 0.0],
            [0.0, 0.55 - p.gap / 2.0, 0.0],
            [0.0, 0.55 + p.gap / 2.0, 0.0],
        ]);
        v.extend([[-0.45, -0.25 - p.brow, 0.0], [0.45, -0.25 - p.brow, 0.0]]);
        v.extend([
            [0.0, 0.15, 0.3],
            [0.0, -0.3, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, -0.25, 0.0],
            [1.0, -0.25, 0.0],
        ]);
        v
    }

    fn project(p: &Pose) -> Vec<(f32, f32)> {
        let (yw, pt, rl) = (
            p.yaw.to_radians(),
            p.pitch.to_radians(),
            p.roll.to_radians(),
        );
        model(p)
            .into_iter()
            .map(|[x, y, z]| {
                let (x, z) = (x * yw.cos() + z * yw.sin(), -x * yw.sin() + z * yw.cos());
                let (y, _z) = (y * pt.cos() - z * pt.sin(), y * pt.sin() + z * pt.cos());
                let (x, y) = (x * rl.cos() - y * rl.sin(), x * rl.sin() + y * rl.cos());
                (x * 100.0 + 320.0, y * 100.0 + 240.0)
            })
            .collect()
    }

    fn face(p: &Pose) -> Face {
        let pts = project(p);
        let mut f = [Vec2::new(0.0, 0.0); N];
        for (i, q) in pts.iter().enumerate() {
            f[i] = Vec2::new(q.0, q.1);
        }
        f
    }

    fn cfg() -> FaceConfig {
        FaceConfig {
            geometry: Geometry {
                pitch_depth: 0.24,
                ..Geometry::default()
            },
            ..FaceConfig::default()
        }
    }

    fn raw(p: &Pose) -> Raw {
        measure(&face(p), &cfg().geometry).unwrap()
    }

    /// Calibration from the neutral pose plus a blink, a wide-open mouth, a smile, a pucker and raised brows.
    fn cal() -> Calibration {
        let n = raw(&NEUTRAL);
        let mut rows = vec![n; 5];
        for p in [
            Pose {
                eye: 0.01,
                ..NEUTRAL
            },
            Pose {
                gap: 0.5,
                ..NEUTRAL
            },
            Pose { mw: 1.0, ..NEUTRAL },
            Pose { mw: 0.5, ..NEUTRAL },
            Pose {
                brow: 0.45,
                ..NEUTRAL
            },
        ] {
            for _ in 0..3 {
                rows.push(raw(&p));
            }
        }
        // the percentile range is taken over all rows: make the extremes exact by repeating them
        Calibration::from_rows(&rows[..5], &rows).unwrap()
    }

    fn norm(p: &Pose) -> [f32; 11] {
        normalise(&raw(p), &cal(), &cfg()).unwrap()
    }
    fn sig(a: &[f32; 11], name: &str) -> f32 {
        a[SIGNALS.iter().position(|s| *s == name).unwrap()]
    }

    #[test]
    fn index_tables_are_consistent() {
        for t in [IBUG68.as_slice(), MESH468.as_slice()] {
            let mut u = t.to_vec();
            u.sort();
            u.dedup();
            assert_eq!(u.len(), N, "indices unique");
        }
        assert!(IBUG68.iter().all(|&i| i < 68) && MESH468.iter().all(|&i| i < 468));
        assert_eq!(Layout::Ibug68.min_points(), 67);
        assert_eq!(Layout::Mesh468.min_points(), 455);
    }

    #[test]
    fn eye_aspect_ratio_matches_the_formula() {
        let r = raw(&Pose {
            eye: 0.108,
            ..NEUTRAL
        });
        assert!(
            (r[3] - 0.3).abs() < 1e-4 && (r[4] - 0.3).abs() < 1e-4,
            "{:?}",
            r
        );
        let r = raw(&Pose {
            eye: 0.0,
            ..NEUTRAL
        });
        assert!(r[3].abs() < 1e-5);
        let r = raw(&NEUTRAL);
        assert!((r[5] - 0.02 / 0.8).abs() < 1e-4, "mar");
        assert!((r[6] - 0.4).abs() < 1e-4, "width ratio");
        assert!((r[7] - 0.27 / 0.9).abs() < 1e-4, "brow");
    }

    #[test]
    fn head_angles_are_recovered() {
        let c = Calibration::from_rows(&[raw(&NEUTRAL); 3], &[]).unwrap();
        let n = |p: &Pose| normalise(&raw(p), &c, &cfg()).unwrap();
        // default spans: 35, 25, 30 degrees; value = 0.5 + 0.5 * angle / span
        for a in [-30.0f32, -10.0, 15.0, 30.0] {
            assert!(
                (sig(&n(&Pose { yaw: a, ..NEUTRAL }), "head_yaw") - (0.5 + 0.5 * a / 35.0)).abs()
                    < 2e-3,
                "yaw {a}"
            );
            assert!(
                (sig(
                    &n(&Pose {
                        pitch: a * 0.6,
                        ..NEUTRAL
                    }),
                    "head_pitch"
                ) - (0.5 + 0.5 * a * 0.6 / 25.0))
                    .abs()
                    < 2e-3,
                "pitch {a}"
            );
            assert!(
                (sig(&n(&Pose { roll: a, ..NEUTRAL }), "head_roll") - (0.5 + 0.5 * a / 30.0)).abs()
                    < 1e-4,
                "roll {a}"
            );
        }
        assert!((sig(&n(&NEUTRAL), "head_yaw") - 0.5).abs() < 1e-5);
        // a combined small pose stays close (2D approximation)
        let v = sig(
            &n(&Pose {
                yaw: 12.0,
                pitch: 8.0,
                roll: 5.0,
                ..NEUTRAL
            }),
            "head_yaw",
        );
        assert!((v - (0.5 + 0.5 * 12.0 / 35.0)).abs() < 0.03, "{v}");
    }

    #[test]
    fn pitch_sign_and_mirror() {
        let up = raw(&Pose {
            pitch: 15.0,
            ..NEUTRAL
        });
        let down = raw(&Pose {
            pitch: -15.0,
            ..NEUTRAL
        });
        assert!(up[1] > raw(&NEUTRAL)[1] && down[1] < raw(&NEUTRAL)[1]);
        // mirroring the picture (negating x) is undone by the mirror flag
        let mut f = face(&Pose {
            yaw: 20.0,
            roll: 10.0,
            ..NEUTRAL
        });
        let a = measure(&f, &cfg().geometry).unwrap();
        for p in f.iter_mut() {
            p.x = -p.x;
        }
        let b = measure(
            &f,
            &Geometry {
                mirror: true,
                ..cfg().geometry
            },
        )
        .unwrap();
        for k in 0..9 {
            assert!((a[k] - b[k]).abs() < 1e-3, "{k}");
        }
        let c = measure(&f, &cfg().geometry).unwrap();
        assert!(
            c[2].abs() > 90.0,
            "without the flag a mirrored picture is not understood (roll {})",
            c[2]
        );
    }

    #[test]
    fn eyes_mouth_and_brows() {
        assert!((sig(&norm(&NEUTRAL), "eye_r_open") - 1.0).abs() < 1e-3);
        assert!(
            sig(
                &norm(&Pose {
                    eye: 0.01,
                    ..NEUTRAL
                }),
                "eye_r_open"
            ) < 0.05
        );
        let half = sig(
            &norm(&Pose {
                eye: 0.06,
                ..NEUTRAL
            }),
            "eye_l_open",
        );
        assert!(half > 0.3 && half < 0.7, "{half}");
        let (m0, m1) = (
            sig(&norm(&NEUTRAL), "mouth_open"),
            sig(
                &norm(&Pose {
                    gap: 0.5,
                    ..NEUTRAL
                }),
                "mouth_open",
            ),
        );
        assert!(m0 < 0.01 && m1 > 0.95, "{m0} {m1}");
        assert!(
            sig(&norm(&Pose { mw: 1.0, ..NEUTRAL }), "mouth_wide") > 0.95
                && sig(&norm(&NEUTRAL), "mouth_wide") < 0.01
        );
        assert!(sig(&norm(&Pose { mw: 0.5, ..NEUTRAL }), "mouth_round") > 0.95);
        assert!(sig(&norm(&Pose { mw: 1.0, ..NEUTRAL }), "mouth_round") < 0.01);
        let b = norm(&Pose {
            brow: 0.45,
            ..NEUTRAL
        });
        assert!(
            sig(&b, "brow_r_raise") > 0.95
                && sig(&b, "brow_l_raise") > 0.95
                && sig(&b, "brow_raise") > 0.95
        );
        assert!(sig(&norm(&NEUTRAL), "brow_raise") < 0.01);
        // one-sided blink: the other eye stays open
        let mut f = face(&NEUTRAL);
        let c = face(&Pose {
            eye: 0.0,
            ..NEUTRAL
        });
        f[0..6].copy_from_slice(&c[0..6]);
        let n = normalise(&measure(&f, &cfg().geometry).unwrap(), &cal(), &cfg()).unwrap();
        assert!(sig(&n, "eye_r_open") < 0.05 && sig(&n, "eye_l_open") > 0.95);
    }

    #[test]
    fn distance_does_not_matter() {
        let f = face(&Pose {
            eye: 0.07,
            gap: 0.2,
            yaw: 10.0,
            ..NEUTRAL
        });
        let g: Face = f.map(|p| Vec2::new(p.x * 0.37 + 11.0, p.y * 0.37 - 5.0));
        let (a, b) = (
            measure(&f, &cfg().geometry).unwrap(),
            measure(&g, &cfg().geometry).unwrap(),
        );
        for k in 0..9 {
            assert!((a[k] - b[k]).abs() < 1e-3, "{k}");
        }
    }

    #[test]
    fn calibration_median_range_and_errors() {
        let mut rows = vec![raw(&NEUTRAL); 7];
        rows.push(raw(&Pose {
            eye: 0.0,
            ..NEUTRAL
        })); // an outlier among the neutral frames
        let c = Calibration::from_rows(&rows, &[]).unwrap();
        assert!(
            (c.neutral["ear_r"] - 0.3).abs() < 1e-4,
            "median ignores one outlier"
        );
        assert!(c.lo["ear_r"] < 0.3, "range sees the outlier");
        assert!(Calibration::from_rows(&rows[..2], &[])
            .unwrap_err()
            .contains("at least 3"));
        // json round trip, missing key, non finite
        let j = serde_json::to_string(&c).unwrap();
        assert_eq!(Calibration::from_json(&j).unwrap(), c);
        let mut bad = c.clone();
        bad.hi.remove("mar");
        assert!(
            Calibration::from_json(&serde_json::to_string(&bad).unwrap())
                .unwrap_err()
                .contains("mar")
        );
        assert!(Calibration::from_json("{").is_err());
        // without a blink in the calibration the eye uses the 25 % fallback
        let c2 = Calibration::from_rows(&[raw(&NEUTRAL); 3], &[]).unwrap();
        let closed = normalise(
            &raw(&Pose {
                eye: 0.0,
                ..NEUTRAL
            }),
            &c2,
            &cfg(),
        )
        .unwrap();
        assert_eq!(sig(&closed, "eye_r_open"), 0.0);
        // degenerate default calibration keeps everything finite
        let nom = normalise(&raw(&NEUTRAL), &Calibration::nominal(), &cfg()).unwrap();
        assert!(nom.iter().all(|x| (0.0..=1.0).contains(x)));
    }

    #[test]
    fn one_euro_reduces_jitter_and_lag_depends_on_beta() {
        // deterministic pseudo noise (LCG) on a constant 0.5 at 60 Hz
        let mut s = 12345u32;
        let mut noise = || {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            ((s >> 8) as f32 / (1u32 << 24) as f32 - 0.5) * 0.1
        };
        let xs: Vec<f32> = (0..600).map(|_| 0.5 + noise()).collect();
        let mut f = OneEuro::default();
        let ys: Vec<f32> = xs
            .iter()
            .enumerate()
            .map(|(i, x)| f.filter(*x, i as f32 / 60.0, 1.0, 0.0, 1.0))
            .collect();
        let sd = |v: &[f32]| {
            let m = v.iter().sum::<f32>() / v.len() as f32;
            (v.iter().map(|x| (x - m).powi(2)).sum::<f32>() / v.len() as f32).sqrt()
        };
        assert!(
            sd(&ys[60..]) < 0.4 * sd(&xs[60..]),
            "{} vs {}",
            sd(&ys[60..]),
            sd(&xs[60..])
        );
        // step response: time until 90 % of a unit step; a larger beta reacts faster
        let step = |beta: f32| {
            let mut f = OneEuro::default();
            f.filter(0.0, 0.0, 1.0, beta, 1.0);
            for i in 1..600 {
                let y = f.filter(1.0, i as f32 / 60.0, 1.0, beta, 1.0);
                if y >= 0.9 {
                    return i;
                }
            }
            999
        };
        let (slow, fast) = (step(0.0), step(50.0));
        assert!(fast < slow && slow > 10, "slow {slow} fast {fast}");
        // first sample passes through, equal timestamps change nothing
        let mut f = OneEuro::default();
        assert_eq!(f.filter(0.7, 1.0, 1.0, 0.0, 1.0), 0.7);
        assert_eq!(f.filter(0.1, 1.0, 1.0, 0.0, 1.0), 0.7);
    }

    #[test]
    fn deadzone_shapes() {
        assert_eq!(dead(0.01, 0.05, false), 0.0);
        assert!((dead(0.525, 0.05, false) - 0.5).abs() < 1e-5);
        assert_eq!(dead(1.0, 0.05, false), 1.0);
        assert_eq!(dead(0.51, 0.05, true), 0.5);
        assert_eq!(dead(0.0, 0.05, true), 0.0);
        assert_eq!(dead(1.0, 0.05, true), 1.0);
        assert!(dead(0.6, 0.05, true) > 0.5 && dead(0.4, 0.05, true) < 0.5);
    }

    fn stream_from(poses: &[(f32, Option<Pose>)]) -> LandmarkStream {
        LandmarkStream {
            frames: poses
                .iter()
                .map(|(t, p)| LandmarkFrame {
                    t: *t,
                    face: p.as_ref().map(face),
                })
                .collect(),
            warnings: vec![],
            fps: None,
        }
    }

    /// 3 s at 30 fps: neutral for 1 s, then two blinks and a mouth opening.
    fn performance() -> LandmarkStream {
        let mut v = Vec::new();
        for i in 0..90 {
            let t = i as f32 / 30.0;
            let mut p = NEUTRAL;
            if (45..49).contains(&i) || (70..74).contains(&i) {
                p.eye = 0.005;
            }
            if (30..60).contains(&i) {
                p.gap = 0.5 * ((i - 30) as f32 / 30.0 * std::f32::consts::PI).sin();
            }
            v.push((t, Some(p)));
        }
        stream_from(&v)
    }

    #[test]
    fn stream_is_deterministic_and_events_are_face_sourced() {
        let s = performance();
        let c = calibrate(&s, &cfg(), 25).unwrap();
        let a = events(&s, &cfg(), &c).unwrap();
        let b = events(&s, &cfg(), &c).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 90 * SIGNALS.len());
        assert!(a
            .iter()
            .all(|e| e.source == InputSource::Face && (0.0..=1.0).contains(&e.value)));
        let mo: Vec<f32> = a
            .iter()
            .filter(|e| e.id == "mouth_open")
            .map(|e| e.value)
            .collect();
        assert!(
            mo[10] < 0.05 && mo.iter().cloned().fold(0.0, f32::max) > 0.8,
            "{:?}",
            &mo[..5]
        );
        let eyes: Vec<f32> = a
            .iter()
            .filter(|e| e.id == "eye_r_open")
            .map(|e| e.value)
            .collect();
        assert!(
            eyes[10] > 0.95 && eyes.iter().cloned().fold(1.0, f32::min) < 0.6,
            "blink visible after smoothing"
        );
    }

    #[test]
    fn lost_frames_and_gaps() {
        let mut v: Vec<(f32, Option<Pose>)> =
            (0..10).map(|i| (i as f32 / 30.0, Some(NEUTRAL))).collect();
        v[4].1 = None;
        let s = stream_from(&v);
        let ev = events(&s, &cfg(), &Calibration::nominal()).unwrap();
        assert_eq!(ev.len(), 9 * SIGNALS.len(), "a lost frame gives no events");
        assert!(events(
            &stream_from(&[(0.0, None)]),
            &cfg(),
            &Calibration::nominal()
        )
        .is_err());
        // degenerate geometry (all points equal) counts as lost
        let z = LandmarkStream {
            frames: vec![LandmarkFrame {
                t: 0.0,
                face: Some([Vec2::new(1.0, 1.0); N]),
            }],
            ..Default::default()
        };
        assert!(events(&z, &cfg(), &Calibration::nominal()).is_err());
        // after a long gap the filter restarts instead of gliding
        let mut tr = FaceTracker::new(cfg(), cal()).unwrap();
        tr.process(0.0, &face(&NEUTRAL)).unwrap();
        let e = tr
            .process(
                5.0,
                &face(&Pose {
                    gap: 0.5,
                    ..NEUTRAL
                }),
            )
            .unwrap();
        assert!(last_values(&e)["mouth_open"] > 0.95);
    }

    #[test]
    fn config_and_calibrate_errors() {
        let s = performance();
        assert!(calibrate(&s, &cfg(), 2).is_err());
        let few = stream_from(&[(0.0, Some(NEUTRAL)), (0.1, Some(NEUTRAL))]);
        assert!(calibrate(&few, &cfg(), 10).is_err());
        let mut c = cfg();
        c.deadzone = 1.0;
        assert!(c.validate().is_err());
        let mut c = cfg();
        c.mincutoff = 0.0;
        assert!(FaceTracker::new(c, Calibration::nominal()).is_err());
        let mut c = cfg();
        c.per_signal.insert("nope".into(), (1.0, 1.0));
        assert!(c.validate().unwrap_err().contains("nope"));
        let mut c = cfg();
        c.span_deg[0] = 0.5;
        assert!(c.validate().is_err());
        let mut c = cfg();
        c.geometry.yaw_depth = f32::NAN;
        assert!(c.validate().is_err());
    }

    // ---- importer --------------------------------------------------------------------------

    fn frame68(p: &Pose) -> Vec<[f32; 2]> {
        let f = face(p);
        let mut v = vec![[7.0f32, 7.0]; 68];
        for (i, &k) in IBUG68.iter().enumerate() {
            v[k] = [f[i].x, f[i].y];
        }
        v
    }
    fn frame468(p: &Pose, total: usize) -> Vec<[f32; 2]> {
        let f = face(p);
        let mut v = vec![[7.0f32, 7.0]; total];
        for (i, &k) in MESH468.iter().enumerate() {
            v[k] = [f[i].x, f[i].y];
        }
        v
    }
    fn line(t: f32, pts: &[[f32; 2]]) -> String {
        serde_json::json!({ "t": t, "p": pts }).to_string()
    }

    #[test]
    fn layouts_68_468_478_and_23_give_the_same_face() {
        let p = Pose {
            yaw: 10.0,
            eye: 0.07,
            ..NEUTRAL
        };
        let f23: Vec<[f32; 2]> = face(&p).iter().map(|q| [q.x, q.y]).collect();
        let want = face(&p);
        for (pts, name) in [
            (f23, "23"),
            (frame68(&p), "68"),
            (frame468(&p, 468), "468"),
            (frame468(&p, 478), "478"),
        ] {
            let s = parse_ndjson(&line(0.0, &pts), 30.0, None).unwrap();
            assert_eq!(s.frames[0].face.unwrap(), want, "layout {name}");
        }
        // header layout, explicit override, wrong layout for the point count
        let hdr = r#"{"format":"osatura-landmarks","version":1,"layout":"ibug68","fps":25}"#;
        let s = parse_ndjson(&format!("{hdr}\n{}", line(0.5, &frame68(&p))), 30.0, None).unwrap();
        assert_eq!(s.fps, Some(25.0));
        let short = &frame68(&p)[..60];
        assert!(
            parse_ndjson(&format!("{hdr}\n{}", line(0.5, short)), 30.0, None)
                .unwrap_err()
                .contains("needs at least 67")
        );
    }

    #[test]
    fn importer_is_tolerant() {
        let p = Pose::clone(&NEUTRAL);
        let pts = frame68(&p);
        let flat: Vec<f32> = pts.iter().flat_map(|q| [q[0], q[1]]).collect();
        let objs: Vec<Value> = pts
            .iter()
            .map(|q| serde_json::json!({"x": q[0], "y": q[1]}))
            .collect();
        let tri: Vec<Value> = pts
            .iter()
            .map(|q| serde_json::json!([q[0], q[1], 0.3]))
            .collect();
        let mut named = serde_json::Map::new();
        for (i, n) in NAMES.iter().enumerate() {
            let q = face(&p)[i];
            named.insert(n.to_string(), serde_json::json!([q.x, q.y]));
        }
        let text = [
            "# a comment".to_string(),
            String::new(),
            serde_json::json!({"frame": 3, "p": flat}).to_string(),
            serde_json::json!({"t": 0.2, "p": objs}).to_string(),
            serde_json::json!({"p": tri}).to_string(), // no time: running index / fps
            serde_json::json!({"t": 0.4, "named": named}).to_string(),
            serde_json::json!({"t": 0.5, "p": null}).to_string(),
            "this is not json".to_string(),
            serde_json::json!({"t": 0.6}).to_string(),
            serde_json::json!({"t": -1.0, "p": pts}).to_string(),
            serde_json::json!({"t": 0.7, "p": [[1.0, 2.0]]}).to_string(),
            serde_json::to_string(&pts).unwrap(), // a bare array of points
        ]
        .join("\n");
        let s = parse_ndjson(&text, 10.0, None).unwrap();
        assert_eq!(s.frames.len(), 6, "{:?}", s.warnings);
        assert!(s.warnings.len() >= 4, "{:?}", s.warnings);
        assert!(
            s.warnings.iter().any(|w| w.contains("not JSON"))
                && s.warnings.iter().any(|w| w.contains("needs p or named"))
        );
        let t: Vec<f32> = s.frames.iter().map(|f| f.t).collect();
        assert!(s.warnings.iter().any(|w| w.contains("sorted")), "{t:?}");
        assert!(
            (t[0] - 0.0).abs() < 1e-6 || t.windows(2).all(|w| w[0] <= w[1]),
            "sorted by time"
        );
        assert!(
            s.frames.iter().any(|f| (f.t - 0.3).abs() < 1e-5),
            "frame 3 at 10 fps = 0.3 s"
        );
        assert_eq!(
            s.frames.iter().filter(|f| f.face.is_none()).count(),
            1,
            "null = lost"
        );
        // NaN as a string-free JSON cannot occur; a null coordinate is a lost frame
        let lost = r#"{"t":0,"p":[[null,1],[2,3]]}"#;
        assert!(parse_ndjson(lost, 30.0, None).unwrap().frames[0]
            .face
            .is_none());
    }

    #[test]
    fn importer_size_scaling_and_errors() {
        // normalised coordinates of a 640 x 480 picture become isotropic
        let f = face(&NEUTRAL);
        let norm: Vec<[f32; 2]> = f.iter().map(|q| [q.x / 640.0, q.y / 480.0]).collect();
        let hdr = r#"{"format":"osatura-landmarks","layout":"osatura23","size":[640,480]}"#;
        let s = parse_ndjson(&format!("{hdr}\n{}", line(0.0, &norm)), 30.0, None).unwrap();
        let g = s.frames[0].face.unwrap();
        for i in 0..N {
            assert!((g[i].x - f[i].x).abs() < 0.01 && (g[i].y - f[i].y).abs() < 0.01);
        }
        for (txt, what) in [
            ("", "no landmark frames"),
            ("garbage", "no usable"),
            (r#"{"format":"other"}"#, "unknown format"),
            (r#"{"format":"osatura-landmarks","version":2}"#, "version"),
            (
                r#"{"format":"osatura-landmarks","layout":"zzz"}"#,
                "unknown layout",
            ),
            (r#"{"format":"osatura-landmarks","size":[0,5]}"#, "size"),
            (r#"{"format":"osatura-landmarks","fps":-1}"#, "fps"),
            (
                r#"{"format":"osatura-landmarks","layout":{"points":{"nose_tip":1}}}"#,
                "missing index",
            ),
            (r#"{"t":0,"p":[[1,2],[3,4],[5,6]]}"#, "infer"),
        ] {
            let e = parse_ndjson(txt, 30.0, None).unwrap_err();
            assert!(e.contains(what), "{txt} -> {e}");
        }
        assert!(parse_ndjson("{}", 0.0, None).is_err());
        // an explicit points layout works
        let mut pts = serde_json::Map::new();
        for (i, n) in NAMES.iter().enumerate() {
            pts.insert(n.to_string(), serde_json::json!(N - 1 - i));
        }
        let rev: Vec<[f32; 2]> = f.iter().rev().map(|q| [q.x, q.y]).collect();
        let hdr =
            serde_json::json!({"format":"osatura-landmarks","layout":{"points":pts}}).to_string();
        let s = parse_ndjson(&format!("{hdr}\n{}", line(0.0, &rev)), 30.0, None).unwrap();
        assert_eq!(s.frames[0].face.unwrap(), f);
    }

    // ---- bake ------------------------------------------------------------------------------

    fn proj() -> Project {
        let mut p = Project::new("f", 200.0, 200.0, 30.0, 90);
        p.bones = vec![Bone::new("jaw", None, 20.0, Vec2::new(100.0, 100.0), 0.0)];
        p
    }

    const MAP: &str = r#"{
        "face": {"mincutoff": 3.0},
        "params": [
          {"source":"face","id":"mouth_open","target":"jaw","prop":"bone_angle","out_min":0,"out_max":-30}
        ],
        "triggers": [
          {"source":"face","id":"eye_r_open","edge":"release","threshold":0.5,"action":{"type":"fire_event","name":"blink"}}
        ]}"#;

    #[test]
    fn bake_keys_the_jaw_and_fires_blinks() {
        let (m, c) = parse_mapfile(MAP).unwrap();
        assert_eq!(c.mincutoff, 3.0);
        let s = performance();
        let c = FaceConfig {
            geometry: cfg().geometry,
            ..c
        };
        let cal = calibrate(&s, &c, 25).unwrap();
        let mut p = proj();
        let r = bake(&mut p, &s, &m, &c, &cal, 0.05).unwrap();
        assert_eq!(r.channels, 1);
        assert_eq!(r.events, 2, "two blinks: {:?}", p.events);
        assert!(p.events.iter().all(|e| e.name == "blink"));
        let Source::Keys { track } = &p.channels[0].source else {
            panic!()
        };
        assert!(track.keys.len() >= 3);
        crate::engine::Engine::new(p.clone()).unwrap();
        let (mid, rest) = (track.eval(45.0).unwrap(), track.eval(5.0).unwrap());
        assert!(mid < -15.0 && rest.abs() < 2.0, "{mid} {rest}");
    }

    #[test]
    fn bake_respects_locks_and_rejects_bad_maps() {
        let (m, c) = parse_mapfile(MAP).unwrap();
        let s = performance();
        let c = FaceConfig {
            geometry: cfg().geometry,
            ..c
        };
        let cal = calibrate(&s, &c, 25).unwrap();
        let mut p = proj();
        p.channels.push(crate::project::Channel::keys(
            "jaw",
            Prop::BoneAngle,
            crate::anim::Track::with_keys(&[(0.0, 0.0, crate::anim::Ease::Linear)]),
        ));
        crate::chanmeta::set_meta(
            &mut p,
            "jaw",
            Some(Prop::BoneAngle),
            &crate::chanmeta::Change {
                lock: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        let before = p.clone();
        let e = bake(&mut p, &s, &m, &c, &cal, 0.05).unwrap_err();
        assert!(e.to_lowercase().contains("lock"), "{e}");
        assert_eq!(p, before, "untouched on error");
        let bad = MAP.replace("mouth_open", "mouth_opn");
        assert!(parse_mapfile(&bad).unwrap_err().contains("mouth_opn"));
        assert!(parse_mapfile(&MAP.replace("eye_r_open", "eyeopen"))
            .unwrap_err()
            .contains("trigger"));
        assert!(parse_mapfile("{").is_err());
        assert!(parse_mapfile(r#"{"face":{"deadzone":3}}"#).is_err());
        assert!(parse_mapfile(r#"{"face":{"mincutoff":"x"}}"#).is_err());
        // unknown bone target is reported by the engine compile
        let mut q = proj();
        let (m2, _) = parse_mapfile(&MAP.replace("\"jaw\"", "\"nope\"")).unwrap();
        assert!(bake(&mut q, &s, &m2, &c, &cal, 0.05).is_err());
    }

    #[test]
    fn switch_layer_can_follow_a_signal() {
        // a threshold trigger sets a layer switch key through the ordinary SetKey action
        let m: Mapping = serde_json::from_str(
            r#"{"triggers":[{"source":"face","id":"mouth_open","edge":"press","threshold":0.5,"action":{"type":"set_key","target":"mouth","prop":"layer_switch","value":2}}]}"#,
        )
        .unwrap();
        check_mapping(&m).unwrap();
        assert_eq!(m.triggers[0].source, InputSource::Face);
    }
}
