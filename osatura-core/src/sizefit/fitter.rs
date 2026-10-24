// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! The fitter: search over the quality ladder for the best file that fits the budget (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// The fitter

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Search {
    /// Bisection over the chain only.
    Ladder,
    /// Chain bisection, then coordinate descent; an exhaustive grid is tried when constraints cannot be met.
    Descent,
    /// Exhaustive (thinned) grid over all axes.
    Grid,
}

impl Search {
    pub fn parse(s: &str) -> Result<Search, String> {
        match s {
            "ladder" => Ok(Search::Ladder),
            "descent" => Ok(Search::Descent),
            "grid" => Ok(Search::Grid),
            o => Err(format!("unknown --search '{o}' (ladder, descent or grid)")),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Search::Ladder => "ladder",
            Search::Descent => "descent",
            Search::Grid => "grid",
        }
    }
}

#[derive(Clone, Debug)]
pub struct FitOptions {
    pub format: Format,
    pub budget: usize,
    pub measure: Measure,
    /// Worst sampled frame must reach this SSIM.
    pub min_ssim: Option<f64>,
    /// Number of frames compared with the original render.
    pub sample_frames: usize,
    pub search: Search,
    /// Upper bound on produced candidates in the descent.
    pub max_evals: usize,
    /// Wall-clock budget of the whole search in seconds (default 60; 0 = unlimited). When it runs out the
    /// bisection result found so far is kept, the descent / adaptive / grid refinements stop, and the report
    /// says `time_limited`. A GIF candidate renders and encodes every frame (seconds each at 640x480x48), which
    /// is why the unbounded default search took minutes.
    pub max_seconds: f64,
    /// Largest grid enumerated (axes are thinned to fit).
    pub grid_max: usize,
    /// Evaluate every rung of the chain too and compare with the bisection (slow; used by tests).
    pub audit: bool,
    /// Also measure the project's existing exporter at its defaults.
    pub baseline: bool,
}

impl FitOptions {
    pub fn new(format: Format, budget: usize) -> FitOptions {
        FitOptions { format, budget, measure: Measure::Raw, min_ssim: None, sample_frames: 8, search: Search::Descent, max_evals: 120, max_seconds: 60.0, grid_max: 250, audit: false, baseline: true }
    }
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub pos: Vec<usize>,
    pub settings: Settings,
    pub valid: bool,
    pub error: Option<String>,
    pub raw: usize,
    pub gzip: usize,
    /// The measured size the budget applies to.
    pub budgeted: usize,
    pub ssim_mean: f64,
    pub ssim_min: f64,
    pub worst_frame: usize,
    pub per_frame: Vec<(usize, f64)>,
}

#[derive(Clone, Debug)]
pub struct Quality {
    pub ssim_mean: f64,
    pub ssim_min: f64,
    pub worst_frame: usize,
    /// (frame, ssim, max CIEDE2000)
    pub per_frame: Vec<(usize, f64, f64)>,
    pub de_max: f64,
}

#[derive(Clone, Debug)]
pub struct FitResult {
    pub options: FitOptions,
    pub met: bool,
    pub reason: Option<String>,
    pub chosen: Candidate,
    pub bytes: Vec<u8>,
    pub quality: Quality,
    /// Existing exporter at its defaults: (raw, gzip).
    pub baseline: Option<(usize, usize)>,
    pub floors: Vec<String>,
    pub evaluations: usize,
    /// The search stopped refining because `max_seconds` ran out.
    pub time_limited: bool,
    /// (chain rung, measured budgeted bytes) in probe order.
    pub probes: Vec<(usize, usize)>,
    pub chain_len: usize,
    /// "static" (fixed balanced chain) or "adaptive" (ordered by measured bytes saved per SSIM lost).
    pub chain_kind: &'static str,
    pub monotone: bool,
    pub fallback_linear: bool,
    pub audit_agrees: Option<bool>,
    pub dead_layers: Vec<String>,
    pub sample_frames: Vec<usize>,
    /// Every produced candidate (for the Pareto listing).
    pub all: Vec<Candidate>,
    pub axes: Vec<(String, Vec<f64>)>,
}

pub(super) struct Ladder {
    pub(super) idx: Option<usize>,
    pub(super) probes: Vec<(usize, usize)>,
    pub(super) monotone: bool,
    pub(super) fallback: bool,
    pub(super) audit_agrees: Option<bool>,
    pub(super) len: usize,
}

/// Rasterised reference frames: `(width, height, per-frame RGBA)`.
pub(super) type RasterEntry = (usize, usize, Vec<Option<Vec<u8>>>);
/// Quantiser cache key: (palette size, frame, quality, dither, transparent).
pub(super) type QuantKey = (u32, usize, u16, bool, bool);

pub struct Fit<'a> {
    fmt: Format,
    o: FitOptions,
    axes: Vec<Axis>,
    n: usize,
    samples: Vec<usize>,
    pre_engine: Engine,
    pre_project: Project,
    pub dead: Vec<String>,
    refs: Vec<Img>,
    ref_engine: Engine,
    frames_cache: Vec<Option<Frame>>,
    lottie_base: Option<(Value, Vec<crate::export::LottieLayerMap>)>,
    raster_cache: HashMap<u32, RasterEntry>,
    cache: HashMap<Vec<usize>, Candidate>,
    quant_cache: Vec<(QuantKey, std::rc::Rc<Indexed>)>,
    order: Vec<Vec<usize>>,
    pub evals: usize,
    started: crate::clock::Stamp,
    time_limited: std::cell::Cell<bool>,
    /// while set, the time budget is not enforced (the final rebuild and the first fallback candidate must finish)
    grace: std::cell::Cell<bool>,
    src: &'a Project,
}

#[cfg(test)]
thread_local! {
    /// Test clock: the budget counts as spent once this many candidates were built (None = real clock).
    pub(super) static TEST_EXPIRE_AFTER_EVALS: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

/// Error text of a candidate that was abandoned because `max_seconds` ran out.
pub(super) const TIME_UP: &str = "time budget exhausted";

pub(super) fn sample_list(n: usize, k: usize) -> Vec<usize> {
    let k = k.clamp(1, n.max(1));
    if k == 1 || n <= 1 {
        return vec![0];
    }
    let mut v: Vec<usize> = (0..k).map(|i| (i * (n - 1) + (k - 1) / 2) / (k - 1)).collect();
    v.dedup();
    v
}

impl<'a> Fit<'a> {
    pub fn new(src: &'a Project, o: FitOptions) -> Result<Fit<'a>, String> {
        let started = crate::clock::Stamp::now(); // the budget counts from here: reference renders are part of the run
        if o.budget == 0 {
            return Err("the size budget must be at least 1 byte".into());
        }
        if let Some(m) = o.min_ssim {
            if !(0.0..=1.0).contains(&m) {
                return Err("--min-ssim must be between 0 and 1".into());
            }
        }
        if o.format == Format::Webm && !ffmpeg_available() {
            return Err("webm needs ffmpeg on PATH (it is not available); choose gif or apng".into());
        }
        let ref_engine = Engine::new(src.clone()).map_err(|e| format!("project does not validate: {}", e.join("; ")))?;
        let n = src.frames.max(1) as usize;
        let (pre_project, dead) = if matches!(o.format, Format::Lottie | Format::Html | Format::Project) { drop_dead_layers(src) } else { (src.clone(), vec![]) };
        let pre_engine = Engine::new(pre_project.clone()).map_err(|e| format!("reduced project does not validate: {}", e.join("; ")))?;
        let samples = sample_list(n, o.sample_frames);
        let ro = RasterOptions { scale: 1.0, show_bones: false, transparent: false };
        let mut refs = Vec::new();
        for &f in &samples {
            let (w, h, d) = render_rgba(&ref_engine, f as f32, &ro);
            refs.push(Img::new(w as usize, h as usize, d)?);
        }
        let axes = axes(o.format);
        let order = chain(o.format);
        Ok(Fit { fmt: o.format, o, axes, n, samples, pre_engine, pre_project, dead, refs, ref_engine, frames_cache: (0..n).map(|_| None).collect(), lottie_base: None, raster_cache: HashMap::new(), cache: HashMap::new(), quant_cache: Vec::new(), order, evals: 0, started, time_limited: std::cell::Cell::new(false), grace: std::cell::Cell::new(false), src })
    }

    pub(super) fn ensure_frame(&mut self, f: usize) {
        if self.frames_cache[f].is_none() {
            self.frames_cache[f] = Some(self.pre_engine.evaluate(f as f32));
        }
    }

    /// `Err(TIME_UP)` once the wall-clock budget is spent (and not in grace mode); remembered for the report.
    pub(super) fn time_up(&self) -> Result<(), String> {
        if self.expired() {
            self.time_limited.set(true);
            return Err(TIME_UP.to_string());
        }
        Ok(())
    }

    /// The wall-clock budget is spent (never in grace mode or with `max_seconds` 0). Tests can replace the clock with
    /// a count of candidate builds (`TEST_EXPIRE_AFTER_EVALS`) to make the moment of expiry deterministic.
    pub(super) fn expired(&self) -> bool {
        if self.grace.get() || self.o.max_seconds <= 0.0 {
            return false;
        }
        #[cfg(test)]
        {
            if let Some(n) = TEST_EXPIRE_AFTER_EVALS.with(|c| c.get()) {
                return self.evals >= n;
            }
        }
        self.started.elapsed().as_secs_f64() > self.o.max_seconds
    }

    pub(super) fn fill_raster(&mut self, scale: f32, frames: &[usize]) -> Result<(), String> {
        let key = scale.to_bits();
        let n = self.n;
        self.raster_cache.entry(key).or_insert_with(|| (0, 0, (0..n).map(|_| None).collect()));
        let opts = RasterOptions { scale, show_bones: false, transparent: false };
        for &f in frames {
            if self.raster_cache[&key].2[f].is_none() {
                if self.expired() {
                    self.time_limited.set(true);
                    return Err(TIME_UP.to_string());
                }
                let (w, h, d) = render_rgba(&self.ref_engine, f as f32, &opts);
                let e = self.raster_cache.get_mut(&key).unwrap();
                e.0 = w as usize;
                e.1 = h as usize;
                e.2[f] = Some(d);
            }
        }
        if self.raster_cache.len() > 3 {
            let drop: Vec<u32> = self.raster_cache.keys().copied().filter(|&k| k != key).take(self.raster_cache.len() - 3).collect();
            for k in drop {
                self.raster_cache.remove(&k);
            }
        }
        Ok(())
    }

    pub(super) fn sample_imgs_from_frames(&self, w: usize, h: usize, decoded: &[Vec<u8>], step: usize) -> Result<Vec<Img>, String> {
        let (rw, rh) = (self.refs[0].w, self.refs[0].h);
        let mut out = Vec::new();
        for &f in &self.samples {
            let i = (f / step).min(decoded.len().saturating_sub(1));
            let d = decoded.get(i).ok_or("decoder returned no frames")?;
            if d.len() != w * h * 4 {
                return Err("decoded frame has the wrong size".into());
            }
            out.push(Img::new(rw, rh, resize_rgba(d, w, h, rw, rh))?);
        }
        Ok(out)
    }

    /// Produce the export for `s` and the decoded sample frames at the reference size.
    pub(super) fn produce(&mut self, s: &Settings) -> Result<(Vec<u8>, Vec<Img>), String> {
        let step = s.frame_step.max(1) as usize;
        let ro = RasterOptions { scale: 1.0, show_bones: false, transparent: false };
        match self.fmt {
            Format::Lottie => {
                if self.lottie_base.is_none() {
                    self.lottie_base = Some(crate::export::lottie_json_mapped(&self.pre_engine));
                }
                let (base, map) = self.lottie_base.clone().unwrap();
                self.time_up()?;
                let doc = lottie_reduce(&base, s);
                let bytes = serde_json::to_vec(&doc).map_err(|e| e.to_string())?;
                let parsed: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                let mut imgs = Vec::new();
                for k in 0..self.samples.len() {
                    self.time_up()?;
                    let f = self.samples[k];
                    self.ensure_frame(f);
                    let fr = lottie_model_frame(self.frames_cache[f].as_ref().unwrap(), &parsed, &map, f);
                    let pm = pixmap_from_frame(&self.pre_engine, &fr, f as f32, &ro);
                    imgs.push(Img::new(pm.width() as usize, pm.height() as usize, to_straight_rgba(&pm))?);
                }
                Ok((bytes, imgs))
            }
            Format::Html => {
                let disp: Vec<usize> = (0..self.n).step_by(step).collect();
                let mut sim: Vec<Frame> = Vec::with_capacity(disp.len());
                let mut strs: Vec<String> = Vec::with_capacity(disp.len());
                for &f in &disp {
                    self.time_up()?;
                    self.ensure_frame(f);
                    let mut fr = clone_frame(self.frames_cache[f].as_ref().unwrap());
                    simplify_frame_paths(&mut fr, s.path_eps as f64);
                    let svg = crate::svg::render_evaluated(&self.pre_engine, &fr, f as f32, &crate::svg::RenderOptions::default());
                    strs.push(svg_round(&svg, s.decimals));
                    sim.push(fr);
                }
                let mut uniq: Vec<String> = Vec::new();
                let mut seen: HashMap<String, usize> = HashMap::new();
                let mut x: Vec<usize> = Vec::with_capacity(strs.len());
                for st in strs {
                    let id = *seen.entry(st.clone()).or_insert_with(|| {
                        uniq.push(st);
                        uniq.len() - 1
                    });
                    x.push(id);
                }
                let identity = x.iter().enumerate().all(|(i, &v)| i == v);
                let p = &self.pre_project;
                let page = html_page(&p.name, p.width, p.fps / step as f32, &uniq, if identity { None } else { Some(&x) });
                let mut imgs = Vec::new();
                for &f in &self.samples {
                    self.time_up()?;
                    let m = f / step;
                    let mut fr = clone_frame(&sim[m]);
                    round_frame(&mut fr, s.decimals);
                    let pm = pixmap_from_frame(&self.pre_engine, &fr, (m * step) as f32, &ro);
                    imgs.push(Img::new(pm.width() as usize, pm.height() as usize, to_straight_rgba(&pm))?);
                }
                Ok((page.into_bytes(), imgs))
            }
            Format::Project => {
                let bytes = project_bytes(&self.pre_project, s)?;
                let p2: Project = serde_json::from_slice(&bytes).map_err(|e| format!("reduced project does not parse: {e}"))?;
                let e2 = Engine::new(p2).map_err(|e| format!("reduced project does not validate: {}", e.join("; ")))?;
                let mut imgs = Vec::new();
                for &f in &self.samples {
                    self.time_up()?;
                    let (w, h, d) = render_rgba(&e2, f as f32, &ro);
                    imgs.push(Img::new(w as usize, h as usize, d)?);
                }
                Ok((bytes, imgs))
            }
            Format::Gif | Format::Apng | Format::Webm => {
                let disp: Vec<usize> = (0..self.n).step_by(step).collect();
                self.fill_raster(s.scale, &disp)?;
                let (w, h, fps) = {
                    let c = &self.raster_cache[&s.scale.to_bits()];
                    (c.0, c.1, self.src.fps)
                };
                let c = &self.raster_cache[&s.scale.to_bits()];
                let refs: Vec<&[u8]> = disp.iter().map(|&f| c.2[f].as_deref().unwrap()).collect();
                let has_alpha = refs.iter().any(|f| f.chunks_exact(4).any(|p| p[3] < 128));
                let reserve = has_alpha || refs.len() > 1;
                self.time_up()?;
                let (bytes, decoded) = match self.fmt {
                    Format::Gif => {
                        let key = (s.scale.to_bits(), step, s.colors, s.dither, reserve);
                        let q = match self.quant_cache.iter().find(|(k, _)| *k == key) {
                            Some((_, q)) => q.clone(),
                            None => {
                                let q = std::rc::Rc::new(quantize(&refs, w, h, s.colors as usize, s.dither, reserve));
                                self.quant_cache.push((key, q.clone()));
                                if self.quant_cache.len() > 4 {
                                    self.quant_cache.remove(0);
                                }
                                q
                            }
                        };
                        self.time_up()?;
                        let b = gif_encode(&q, fps, step as u32, !has_alpha, s.delta_tol)?;
                        let (dw, dh, d) = gif_decode(&b)?;
                        if (dw, dh) != (w, h) {
                            return Err("decoded GIF has the wrong size".into());
                        }
                        (b, d)
                    }
                    Format::Apng => {
                        let b = if s.colors == 0 {
                            apng_encode_rgba(w, h, &refs, fps, step as u32)?
                        } else {
                            let key = (s.scale.to_bits(), step, s.colors, s.dither, reserve);
                            let q = match self.quant_cache.iter().find(|(k, _)| *k == key) {
                                Some((_, q)) => q.clone(),
                                None => {
                                    let q = std::rc::Rc::new(quantize(&refs, w, h, s.colors as usize, s.dither, reserve));
                                    self.quant_cache.push((key, q.clone()));
                                    if self.quant_cache.len() > 4 {
                                        self.quant_cache.remove(0);
                                    }
                                    q
                                }
                            };
                            apng_encode_indexed(&q, fps, step as u32, !has_alpha, 0)?
                        };
                        let (dw, dh, d) = apng_decode(&b)?;
                        if (dw, dh) != (w, h) {
                            return Err("decoded APNG has the wrong size".into());
                        }
                        (b, d)
                    }
                    _ => {
                        self.time_up()?;
                        let b = webm_encode(w, h, &refs, fps / step as f32, s.crf)?;
                        let d = webm_decode(&b, w, h)?;
                        (b, d)
                    }
                };
                let imgs = self.sample_imgs_from_frames(w, h, &decoded, 1)?;
                Ok((bytes, imgs))
            }
        }
    }

    /// Build candidate `pos` (cached).
    pub fn eval(&mut self, pos: &[usize]) -> Candidate {
        if let Some(c) = self.cache.get(pos) {
            return c.clone();
        }
        self.evals += 1;
        let settings = settings_at(self.fmt, pos);
        let c = match self.produce(&settings) {
            Ok((bytes, imgs)) => {
                let raw = bytes.len();
                let gz = gzip_len(&bytes);
                let gz_budget = gzip_budget_len(&bytes);
                let mut per = Vec::new();
                let mut err = None;
                for (k, img) in imgs.iter().enumerate() {
                    match percep::ssim(&self.refs[k], img) {
                        Ok(v) => per.push((self.samples[k], v)),
                        Err(e) => err = Some(e),
                    }
                }
                let worst = per.iter().cloned().fold((0usize, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });
                let mean = if per.is_empty() { 0.0 } else { per.iter().map(|p| p.1).sum::<f64>() / per.len() as f64 };
                Candidate {
                    pos: pos.to_vec(),
                    settings,
                    valid: err.is_none() && !per.is_empty(),
                    error: err,
                    raw,
                    gzip: gz,
                    budgeted: if self.o.measure == Measure::Raw { raw } else { gz_budget },
                    ssim_mean: mean,
                    ssim_min: if worst.1.is_finite() { worst.1 } else { 0.0 },
                    worst_frame: worst.0,
                    per_frame: per,
                }
            }
            Err(e) if e == TIME_UP => {
                // interrupted by the time budget: not a result, not remembered
                self.evals -= 1;
                return Candidate { pos: pos.to_vec(), settings, valid: false, error: Some(e), raw: 0, gzip: 0, budgeted: usize::MAX, ssim_mean: 0.0, ssim_min: 0.0, worst_frame: 0, per_frame: vec![] };
            }
            Err(e) => Candidate { pos: pos.to_vec(), settings, valid: false, error: Some(e), raw: 0, gzip: 0, budgeted: usize::MAX, ssim_mean: 0.0, ssim_min: 0.0, worst_frame: 0, per_frame: vec![] },
        };
        self.cache.insert(pos.to_vec(), c.clone());
        c
    }

    pub(super) fn fits(&self, c: &Candidate) -> bool {
        c.valid && c.budgeted <= self.o.budget
    }
    pub(super) fn good(&self, c: &Candidate) -> bool {
        self.fits(c) && self.o.min_ssim.is_none_or(|m| c.ssim_min >= m)
    }
    /// `a` strictly better than `b`: higher mean SSIM, then higher worst SSIM (differences below 1e-6 count as
    /// equal), then fewer reductions (smaller sum of rungs), then smaller, then lower position.
    pub(super) fn better(a: &Candidate, b: &Candidate) -> bool {
        let eps = 1e-6;
        if (a.ssim_mean - b.ssim_mean).abs() > eps {
            return a.ssim_mean > b.ssim_mean;
        }
        if (a.ssim_min - b.ssim_min).abs() > eps {
            return a.ssim_min > b.ssim_min;
        }
        let (ra, rb): (usize, usize) = (a.pos.iter().sum(), b.pos.iter().sum());
        if ra != rb {
            return ra < rb;
        }
        if a.budgeted != b.budgeted {
            return a.budgeted < b.budgeted;
        }
        a.pos < b.pos
    }

    /// True once the wall-clock budget is spent (remembered for the report).
    pub(super) fn out_of_time(&mut self) -> bool {
        if self.expired() {
            self.time_limited.set(true);
        }
        self.time_limited.get()
    }

    pub(super) fn descend(&mut self, start: Vec<usize>) -> Vec<usize> {
        let lens: Vec<usize> = self.axes.iter().map(|a| a.values.len()).collect();
        let na = lens.len();
        let mut cur = start;
        let mut cur_c = self.eval(&cur);
        for _ in 0..60 {
            if self.evals >= self.o.max_evals || self.out_of_time() {
                break;
            }
            let mut best: Option<Candidate> = None;
            let consider = |me: &mut Self, q: Vec<usize>, best: &mut Option<Candidate>| {
                let c = me.eval(&q);
                if me.good(&c) && Self::better(&c, best.as_ref().unwrap_or(&cur_c)) {
                    *best = Some(c);
                }
            };
            for a in 0..na {
                if !self.axes[a].in_chain {
                    // a free knob: try each other value
                    for v in 0..lens[a] {
                        if v != cur[a] && self.evals < self.o.max_evals && !self.out_of_time() {
                            let mut q = cur.clone();
                            q[a] = v;
                            consider(self, q, &mut best);
                        }
                    }
                } else if cur[a] > 0 {
                    let mut q = cur.clone();
                    q[a] -= 1;
                    consider(self, q, &mut best);
                }
            }
            if best.is_none() {
                'trade: for a in 0..na {
                    if cur[a] == 0 || !self.axes[a].in_chain {
                        continue;
                    }
                    for b in 0..na {
                        if b != a && self.axes[b].in_chain && cur[b] + 1 < lens[b] {
                            if self.evals >= self.o.max_evals || self.out_of_time() {
                                break 'trade;
                            }
                            let mut q = cur.clone();
                            q[a] -= 1;
                            q[b] += 1;
                            consider(self, q, &mut best);
                        }
                    }
                }
            }
            match best {
                Some(c) => {
                    cur = c.pos.clone();
                    cur_c = c;
                }
                None => break,
            }
        }
        cur
    }

    /// All grid positions, axes thinned (endpoints kept) so that the product is at most `max`.
    pub fn grid_positions(&self, max: usize) -> Vec<Vec<usize>> {
        let mut idx: Vec<Vec<usize>> = self.axes.iter().map(|a| (0..a.values.len()).collect()).collect();
        while idx.iter().map(|v| v.len()).product::<usize>() > max {
            let (k, _) = idx.iter().enumerate().max_by(|a, b| a.1.len().cmp(&b.1.len()).then(b.0.cmp(&a.0))).unwrap();
            if idx[k].len() <= 2 {
                break;
            }
            let last = idx[k].len() - 1;
            let mut v: Vec<usize> = idx[k].iter().copied().enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, x)| x).collect();
            if v.last() != Some(&idx[k][last]) {
                v.push(idx[k][last]);
            }
            idx[k] = v;
        }
        let mut out = vec![vec![]];
        for v in &idx {
            let mut next = Vec::with_capacity(out.len() * v.len());
            for p in &out {
                for &x in v {
                    let mut q = p.clone();
                    q.push(x);
                    next.push(q);
                }
            }
            out = next;
        }
        out
    }

    /// Bisection over `chain` (rungs ordered from best to smallest) on measured sizes, with an empirical
    /// monotonicity check over everything probed and a linear-scan fallback.
    pub(super) fn ladder(&mut self, chain: &[Vec<usize>], audit: bool) -> Ladder {
        let n = chain.len();
        let mut out = Ladder { idx: None, probes: Vec::new(), monotone: true, fallback: false, audit_agrees: None, len: n };
        let probe = |me: &mut Self, i: usize, probes: &mut Vec<(usize, usize)>| -> Candidate {
            let c = me.eval(&chain[i]);
            if !probes.iter().any(|p| p.0 == i) {
                probes.push((i, c.budgeted));
            }
            c
        };
        let c0 = probe(self, 0, &mut out.probes);
        if self.fits(&c0) {
            // fits at the top: nothing further down the chain can be better in quality
            out.idx = Some(0);
        } else {
            let cl = probe(self, n - 1, &mut out.probes);
            if self.fits(&cl) {
                let (mut lo, mut hi) = (0usize, n - 1);
                while hi - lo > 1 {
                    let mid = (lo + hi) / 2;
                    let c = probe(self, mid, &mut out.probes);
                    if self.fits(&c) {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                out.idx = Some(hi);
            } else {
                // a few extra probes to confirm the floor is the smallest
                for q in [n / 4, n / 2, 3 * n / 4] {
                    if q > 0 && q < n - 1 {
                        probe(self, q, &mut out.probes);
                    }
                }
            }
        }
        let monotone_of = |probes: &[(usize, usize)]| {
            let mut p = probes.to_vec();
            p.sort();
            p.windows(2).all(|w| w[1].1 <= w[0].1 || w[1].1 == usize::MAX && w[0].1 == usize::MAX)
        };
        out.monotone = monotone_of(&out.probes);
        if !out.monotone || audit {
            // linear scan of the chain: the lowest-index rung that fits
            for i in 0..n {
                probe(self, i, &mut out.probes);
            }
            // a rung the exhausted time budget kept from being built is unknown, which counts as not fitting
            let scan = (0..n).find(|&i| self.cache.get(&chain[i]).is_some_and(|c| self.fits(c)));
            if !out.monotone {
                out.fallback = true;
                out.idx = scan;
            } else {
                out.audit_agrees = Some(scan == out.idx);
                if scan != out.idx {
                    out.fallback = true;
                    out.idx = scan;
                }
            }
            out.monotone = monotone_of(&out.probes);
        }
        out
    }

    /// A chain ordered by measured efficiency. Each knob is scanned alone (others at their best rung); the
    /// chain then lowers, at every step, the knob whose next rung saves the most bytes per unit of SSIM lost.
    /// (Assumes the knobs act roughly independently; the descent afterwards does not.)
    pub(super) fn adaptive_chain(&mut self) -> Vec<Vec<usize>> {
        if self.out_of_time() {
            return self.order.clone();
        }
        let na = self.axes.len();
        let chain_axes: Vec<usize> = (0..na).filter(|&a| self.axes[a].in_chain).collect();
        let mut scan: Vec<Vec<(f64, f64)>> = vec![Vec::new(); na];
        let base = self.eval(&vec![0usize; na]);
        for &a in &chain_axes {
            scan[a].push((base.budgeted as f64, base.ssim_mean));
            for r in 1..self.axes[a].values.len() {
                let mut q = vec![0usize; na];
                q[a] = r;
                let c = self.eval(&q);
                scan[a].push((if c.valid { c.budgeted as f64 } else { f64::INFINITY }, if c.valid { c.ssim_mean } else { 0.0 }));
            }
        }
        let mut pos = vec![0usize; na];
        let mut out = vec![pos.clone()];
        loop {
            let mut best: Option<(f64, usize)> = None;
            for &a in &chain_axes {
                let r = pos[a];
                if r + 1 >= scan[a].len() {
                    continue;
                }
                let ((s0, q0), (s1, q1)) = (scan[a][r], scan[a][r + 1]);
                let saved = if s1.is_infinite() || s0.is_infinite() { f64::NEG_INFINITY } else { s0 - s1 };
                let ratio = saved / (q0 - q1).max(1e-4);
                if best.is_none_or(|(b, _)| ratio > b) {
                    best = Some((ratio, a));
                }
            }
            let Some((_, a)) = best else { break };
            pos[a] += 1;
            out.push(pos.clone());
        }
        out
    }

    /// Run the search and assemble the result.
    pub fn run(mut self) -> Result<FitResult, String> {
        let budget = self.o.budget;
        let chosen: Option<Candidate>;
        let mut reason: Option<String> = None;
        let static_chain = self.order.clone();
        let mut primary = Ladder { idx: None, probes: Vec::new(), monotone: true, fallback: false, audit_agrees: None, len: static_chain.len() };
        let mut chain_kind = "static";
        let mut static_start: Option<Vec<usize>> = None;
        let mut adaptive_start: Option<Vec<usize>> = None;
        if self.o.search != Search::Grid {
            let audit = self.o.audit;
            primary = self.ladder(&static_chain, audit);
            if let Some(i) = primary.idx {
                let c = self.eval(&static_chain[i]);
                if self.good(&c) {
                    static_start = Some(static_chain[i].clone());
                }
            }
            let top_ok = primary.idx == Some(0) && static_start.is_some();
            if self.o.search == Search::Descent && !top_ok && primary.idx.is_some() && !self.out_of_time() {
                let ach = self.adaptive_chain();
                let lo = self.ladder(&ach, false);
                if let Some(i) = lo.idx {
                    let c = self.eval(&ach[i]);
                    if self.good(&c) {
                        adaptive_start = Some(ach[i].clone());
                    }
                }
                if lo.idx.is_some() {
                    primary = lo;
                    chain_kind = "adaptive";
                }
            }
        }
        let ladder_idx = primary.idx;
        let mut best_pos: Option<Vec<usize>> = static_start.clone().or(adaptive_start.clone());
        if self.o.search == Search::Descent {
            let mut scored: Vec<(Vec<usize>, Candidate)> = Vec::new();
            for q in [static_start.clone(), adaptive_start.clone()].into_iter().flatten() {
                if !scored.iter().any(|(p, _)| *p == q) {
                    let c = self.eval(&q);
                    scored.push((q, c));
                }
            }
            scored.sort_by(|a, b| if Self::better(&a.1, &b.1) { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater });
            let mut best: Option<Candidate> = None;
            for (q, _) in scored {
                let end = self.descend(q);
                let c = self.eval(&end);
                if self.good(&c) && best.as_ref().is_none_or(|b| Self::better(&c, b)) {
                    best = Some(c);
                }
            }
            best_pos = best.map(|c| c.pos).or(best_pos);
        }
        let (probes, monotone, fallback, audit_agrees, n) = (primary.probes.clone(), primary.monotone, primary.fallback, primary.audit_agrees, primary.len);
        // grid: asked for, or the only way left to satisfy the constraints
        let need_grid = self.o.search == Search::Grid || (self.o.search == Search::Descent && best_pos.is_none() && self.o.min_ssim.is_some() && ladder_idx.is_some());
        if need_grid {
            let mut gbest: Option<Candidate> = best_pos.as_ref().map(|p| self.eval(p));
            for p in self.grid_positions(self.o.grid_max) {
                if self.out_of_time() {
                    break;
                }
                let c = self.eval(&p);
                if self.good(&c) && gbest.as_ref().is_none_or(|b| Self::better(&c, b)) {
                    gbest = Some(c);
                }
            }
            best_pos = gbest.map(|c| c.pos);
        }
        if best_pos.is_none() && self.time_limited.get() {
            // the budget ran out before any candidate was finished: build the cheapest one (the chain floor) anyway,
            // so that there is a best-so-far result to return
            self.grace.set(true);
            if let Some(fp) = static_chain.last() {
                let c = self.eval(fp);
                if self.good(&c) {
                    best_pos = Some(fp.clone());
                }
            }
        }
        let met;
        if let Some(p) = best_pos {
            chosen = Some(self.eval(&p));
            met = true;
        } else {
            met = false;
            // best achievable: smallest valid size seen; if constraints were the problem, say so
            let all: Vec<Candidate> = self.cache.values().filter(|c| c.valid).cloned().collect();
            let smallest = all.iter().min_by(|a, b| a.budgeted.cmp(&b.budgeted).then(a.pos.cmp(&b.pos))).cloned();
            let fitting_any = all.iter().any(|c| c.budgeted <= budget);
            let time_note = if self.time_limited.get() { format!(" [time budget of {} s exhausted: the search was cut short, a longer --max-seconds may find a fit]", self.o.max_seconds) } else { String::new() };
            reason = Some(if fitting_any {
                format!("a candidate fits {budget} bytes but none reaches --min-ssim {:.4}", self.o.min_ssim.unwrap_or(0.0))
            } else {
                format!("the smallest export found is {} bytes ({}), above the budget of {budget} bytes: every knob is at its floor", smallest.as_ref().map_or(0, |c| c.budgeted), self.o.measure.name())
            });
            if self.o.search != Search::Grid {
                // an unmet size budget is only a statement about the chain floor and the probes
                if !fitting_any {
                    reason = reason.map(|r| format!("{r} (checked the chain floor and {} probes; sizes assumed monotone)", probes.len()));
                }
            }
            reason = reason.map(|r| format!("{r}{time_note}"));
            chosen = smallest.or_else(|| all.first().cloned());
            if chosen.is_none() {
                let errs: Vec<String> = self.cache.values().filter_map(|c| c.error.clone()).take(3).collect();
                return Err(format!("no candidate could be produced: {}", errs.join("; ")));
            }
        }
        let chosen = chosen.unwrap();
        // rebuild the chosen file (deterministic) and measure the full quality incl. CIEDE2000; this must finish
        self.grace.set(true);
        let (bytes, imgs) = self.produce(&chosen.settings)?;
        let mut per = Vec::new();
        let mut de_max: f64 = 0.0;
        for (k, img) in imgs.iter().enumerate() {
            let d = percep::compare(&self.refs[k], img, &percep::CompareOptions::default())?;
            de_max = de_max.max(d.max_de_raw);
            per.push((self.samples[k], d.ssim, d.max_de_raw));
        }
        let worst = per.iter().cloned().fold((0usize, f64::INFINITY, 0.0), |a, b| if b.1 < a.1 { b } else { a });
        let quality = Quality {
            ssim_mean: per.iter().map(|p| p.1).sum::<f64>() / per.len().max(1) as f64,
            ssim_min: worst.1,
            worst_frame: worst.0,
            per_frame: per,
            de_max,
        };
        let baseline = if self.o.baseline && !self.time_limited.get() { self.baseline().ok() } else { None };
        let last = self.axes.iter().map(|a| a.values.len() - 1).collect::<Vec<_>>();
        let floors: Vec<String> = self.axes.iter().enumerate().filter(|(i, a)| a.in_chain && chosen.pos[*i] == last[*i]).map(|(_, a)| a.name.to_string()).collect();
        let mut all: Vec<Candidate> = self.cache.values().cloned().collect();
        all.sort_by(|a, b| a.pos.cmp(&b.pos));
        Ok(FitResult {
            options: self.o.clone(),
            met,
            reason,
            chosen,
            bytes,
            quality,
            baseline,
            floors,
            evaluations: self.evals,
            time_limited: self.time_limited.get(),
            probes,
            chain_len: n,
            chain_kind,
            monotone,
            fallback_linear: fallback,
            audit_agrees,
            dead_layers: self.dead.clone(),
            sample_frames: self.samples.clone(),
            all,
            axes: self.axes.iter().map(|a| (a.name.to_string(), a.values.clone())).collect(),
        })
    }

    /// Size of the existing exporter at its default settings: (raw, gzip).
    pub(super) fn baseline(&mut self) -> Result<(usize, usize), String> {
        let bytes: Vec<u8> = match self.fmt {
            Format::Lottie => crate::export::lottie_json(&self.ref_engine).to_string().into_bytes(),
            Format::Html => crate::export::html_player(&self.ref_engine).into_bytes(),
            Format::Gif => crate::export::gif_bytes(&self.ref_engine, 1.0)?,
            Format::Apng => crate::open_formats::apng_bytes(&self.ref_engine, 1.0)?,
            Format::Project => serde_json::to_vec(self.src).map_err(|e| e.to_string())?,
            Format::Webm => {
                let all: Vec<usize> = (0..self.n).collect();
                self.fill_raster(1.0, &all)?;
                let c = &self.raster_cache[&1f32.to_bits()];
                let refs: Vec<&[u8]> = all.iter().map(|&f| c.2[f].as_deref().unwrap()).collect();
                webm_encode(c.0, c.1, &refs, self.src.fps, 30)?
            }
        };
        Ok((bytes.len(), gzip_len(&bytes)))
    }
}

/// Fit `project` into the budget. See the module notes.
pub fn fit(project: &Project, o: FitOptions) -> Result<FitResult, String> {
    Fit::new(project, o)?.run()
}
