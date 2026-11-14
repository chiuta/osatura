// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Scrub proxies: a cheap preview path for dragging the playhead, and an LRU frame cache.
//!
//! A proxy frame is the normally evaluated frame, simplified, drawn by the ordinary software
//! rasteriser at a fraction of the resolution. `level` 0 is the full-quality picture; each level
//! halves the resolution (1 = 1/2, 2 = 1/4, 3 = 1/8, 4 = 1/16) and, from level 1 up, drops detail
//! that cannot be seen at that size:
//!
//! * level >= 1: strokes thinner than about a third of an output pixel are dropped; the rest lose
//!   dashes and width profiles (constant width);
//! * level >= 2: gradients become flat colours (the middle stop), blend modes become Normal, image
//!   meshes are replaced by the plain image rectangle (the mesh subdivision is capped at zero);
//! * level >= 3: layer masks are ignored.
//!
//! Proxies are for orientation while scrubbing only; the exact picture is always the level-0 render.
//! Frame evaluation (rig, IK, physics, expressions) is not simplified: it is the same `evaluate`.
//! There are no post effects in this path (they are not applied by the studio's normal view either).

use crate::engine::{Engine, Frame};
use crate::raster::{pixmap_from_frame, to_straight_rgba, RasterOptions};
use crate::style::{Paint, Rgba};
use std::collections::HashMap;
use std::sync::Arc;

pub const MAX_LEVEL: u8 = 4;

/// Output pixels per stage unit for a proxy level (1.0 for level 0).
pub fn proxy_scale(level: u8) -> f32 {
    0.5f32.powi(level.min(MAX_LEVEL) as i32)
}

pub fn check_level(level: u8) -> Result<(), String> {
    if level > MAX_LEVEL {
        Err(format!("proxy level {level} out of range (0..={MAX_LEVEL})"))
    } else {
        Ok(())
    }
}

fn flatten(p: &Paint) -> Paint {
    match p {
        Paint::Solid(_) => p.clone(),
        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
            let c = stops.get(stops.len() / 2).map(|s| s.color).unwrap_or(Rgba::BLACK);
            Paint::Solid(c)
        }
    }
}

/// Drop detail from an evaluated frame (see the module notes). `scale` is output pixels per stage unit.
pub fn simplify_frame(f: &mut Frame, level: u8, scale: f32) {
    if level == 0 {
        return;
    }
    for node in &mut f.nodes {
        let px = node.world.x_scale() * scale;
        for p in &mut node.paths {
            if let Some(st) = &mut p.stroke {
                if st.width * px < 0.35 {
                    p.stroke = None;
                } else {
                    st.dash.clear();
                    st.width_profile.clear();
                }
            }
            if level >= 2 {
                if let Some(fill) = &p.fill {
                    p.fill = Some(flatten(fill));
                }
                if let Some(st) = &mut p.stroke {
                    st.paint = flatten(&st.paint);
                }
            }
        }
        if level >= 2 {
            node.blend = crate::style::Blend::Normal;
            if let Some(img) = &mut node.image {
                img.mesh = None;
            }
        }
        if level >= 3 {
            node.mask = None;
        }
    }
}

/// Straight RGBA of a proxy frame: `(width, height, pixels)`. Level 0 is the full-quality raster.
pub fn render_proxy_rgba(engine: &Engine, frame: f32, level: u8) -> Result<(u32, u32, Vec<u8>), String> {
    check_level(level)?;
    let opts = RasterOptions { scale: proxy_scale(level), show_bones: false, transparent: false };
    let mut f = engine.evaluate(frame);
    simplify_frame(&mut f, level, opts.scale);
    let pm = pixmap_from_frame(engine, &f, frame, &opts);
    Ok((pm.width(), pm.height(), to_straight_rgba(&pm)))
}

/// A proxy frame as PNG bytes. `scale_mul` multiplies the level's resolution (CLI `--scale`).
pub fn render_proxy_png(engine: &Engine, frame: f32, level: u8, scale_mul: f32, bones: bool) -> Result<Vec<u8>, String> {
    check_level(level)?;
    if !(scale_mul > 0.0 && scale_mul <= 16.0) {
        return Err("scale must be in (0, 16]".into());
    }
    let opts = RasterOptions { scale: proxy_scale(level) * scale_mul, show_bones: bones, transparent: false };
    let mut f = engine.evaluate(frame);
    simplify_frame(&mut f, level, opts.scale);
    pixmap_from_frame(engine, &f, frame, &opts).encode_png().map_err(|e| e.to_string())
}

/// The proxy as a *vector* document: the simplified frame through the normal SVG writer. Much cheaper for a
/// browser than the full SVG when the picture has masks, blends, gradients or image meshes (no per-triangle clip
/// paths), and cheap for the server too. Level 0 is the ordinary full SVG.
pub fn proxy_svg_vector(engine: &Engine, frame: f32, level: u8, bones: bool) -> Result<String, String> {
    check_level(level)?;
    let mut f = engine.evaluate(frame);
    simplify_frame(&mut f, level, proxy_scale(level));
    Ok(crate::svg::render_evaluated(engine, &f, frame, &crate::svg::RenderOptions { show_bones: bones }))
}

/// The proxy as an SVG document of the project's size with the low-resolution bitmap embedded, so a
/// browser shows it at the same size as the full SVG render and just scales it up.
pub fn proxy_svg(engine: &Engine, frame: f32, level: u8, bones: bool) -> Result<String, String> {
    let png = render_proxy_png(engine, frame, level, 1.0, bones)?;
    let p = &engine.project;
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\"><image href=\"data:image/png;base64,{}\" width=\"{w}\" height=\"{h}\" preserveAspectRatio=\"none\" style=\"image-rendering:auto\"/></svg>\n",
        crate::base64::encode(&png),
        w = p.width,
        h = p.height
    ))
}

// ------------------------------------------------------------------------------------------------
// LRU frame cache
// ------------------------------------------------------------------------------------------------

/// What a cached picture is: frame, quality level, whether bones were drawn and which encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CacheKey {
    frame_bits: u32,
    pub level: u8,
    pub bones: bool,
    /// 0 = SVG vectors, 1 = bitmap wrapped in SVG, 2 = PNG
    pub kind: u8,
    /// Reduced-detail copy (`LodProxy`) the picture was drawn from; 0 = the project itself.
    pub lod: u8,
}

impl CacheKey {
    pub fn new(frame: f32, level: u8, bones: bool, kind: u8) -> CacheKey {
        CacheKey { frame_bits: frame.to_bits(), level, bones, kind, lod: 0 }
    }
    pub fn with_lod(mut self, lod: u8) -> CacheKey {
        self.lod = lod;
        self
    }
}

// ------------------------------------------------------------------------------------------------
// Reduced-detail copies for scrubbing (I-11 wired into the studio)
// ------------------------------------------------------------------------------------------------

/// A finished background level build and the seconds it took.
type BuildResult = (Result<Vec<crate::lod::Lod>, String>, f64);

/// Levels of detail the scrub path can draw from. `lod::build` runs once per project revision, the first time
/// a proxy picture is asked for with LOD on, at the reference scale of 1 pixel per stage unit, so the declared
/// bounds are in stage units (`LodRecord::bound_stage`) and any screen size is answered by `lod::pick_level`
/// without rebuilding. An engine is made for a level only when that level is first used. An edit (a new
/// revision) throws everything away; nothing stale can be drawn.
///
/// The build is not instant (the certified simplifier is quadratic in the anchors of a path), so
/// [`LodProxy::background`] runs it on a worker thread: until it is ready `pick` answers level 0 (the project
/// itself, always correct) and the page keeps scrubbing. [`LodProxy::new`] builds synchronously (tests, CLI).
pub struct LodProxy {
    revision: Option<u64>,
    records: Vec<crate::lod::LodRecord>,
    projects: Vec<Option<crate::project::Project>>,
    engines: Vec<Option<Engine>>,
    /// Levels exist for `revision` (or building failed for it, see `unavailable`).
    ready: bool,
    background: bool,
    building: Option<(u64, std::sync::mpsc::Receiver<BuildResult>)>,
    /// Why no levels exist for this revision (the project could not be reduced); the full project is drawn.
    pub unavailable: Option<String>,
    pub builds: u64,
    pub invalidations: u64,
    pub engines_made: u64,
    pub last_build_ms: f64,
}

/// Levels built (L1..Ln), and the tolerance of L1 in stage units; each level doubles it.
pub const LOD_PROXY_LEVELS: usize = 6;
pub const LOD_PROXY_TOLERANCE: f64 = 0.25;

impl Default for LodProxy {
    fn default() -> Self {
        LodProxy::new()
    }
}

impl LodProxy {
    /// Builds on the calling thread when a revision is first asked for.
    pub fn new() -> LodProxy {
        LodProxy { revision: None, records: vec![], projects: vec![], engines: vec![], ready: false, background: false, building: None, unavailable: None, builds: 0, invalidations: 0, engines_made: 0, last_build_ms: 0.0 }
    }

    /// Builds on a worker thread; `pick` answers level 0 until the levels are ready.
    pub fn background() -> LodProxy {
        LodProxy { background: true, ..LodProxy::new() }
    }

    /// Records of the levels for the revision last used (empty before the build is ready or when unavailable).
    pub fn records(&self) -> &[crate::lod::LodRecord] {
        &self.records
    }

    /// True while a worker thread is still building the levels of the current revision.
    pub fn is_building(&self) -> bool {
        self.building.is_some()
    }

    /// True once the levels (or the reason there are none) are known for the revision last used.
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    fn install(&mut self, res: Result<Vec<crate::lod::Lod>, String>, ms: f64) {
        match res {
            Ok(levels) => {
                // `build` returns the original as level 0 followed by L1..Ln: keep only the derived levels
                self.records = levels.iter().skip(1).map(|l| l.record.clone()).collect();
                self.projects = levels.into_iter().skip(1).map(|l| Some(l.project)).collect();
                self.engines = (0..self.projects.len()).map(|_| None).collect();
            }
            Err(e) => self.unavailable = Some(e),
        }
        self.builds += 1;
        self.last_build_ms = ms;
        self.ready = true;
    }

    fn build_now(project: &crate::project::Project) -> (Result<Vec<crate::lod::Lod>, String>, f64) {
        let t0 = crate::clock::Stamp::now();
        let o = crate::lod::LodOptions::new(LOD_PROXY_TOLERANCE, 1.0, LOD_PROXY_LEVELS);
        let r = crate::lod::build(project, &o);
        (r, t0.elapsed().as_secs_f64() * 1000.0)
    }

    /// Take a finished background build if there is one. A result for an older revision is dropped.
    /// True when the levels (or the reason there are none) for `revision` are now known.
    fn collect(&mut self, revision: u64) -> bool {
        let Some((rev, rx)) = &self.building else { return self.ready };
        match rx.try_recv() {
            Ok((res, ms)) => {
                let rev = *rev;
                self.building = None;
                if rev == revision && self.revision == Some(revision) {
                    self.install(res, ms);
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.building = None;
                self.unavailable = Some("the level-of-detail worker stopped".into());
                self.ready = true;
            }
        }
        self.ready
    }

    /// For status reports: pick up a finished build without asking for a picture.
    pub fn poll(&mut self, revision: u64) {
        if self.revision == Some(revision) {
            self.collect(revision);
        }
    }

    fn sync(&mut self, project: &crate::project::Project, revision: u64) {
        if self.revision != Some(revision) {
            if self.revision.is_some() {
                self.invalidations += 1;
            }
            self.revision = Some(revision);
            self.records.clear();
            self.projects.clear();
            self.engines.clear();
            self.unavailable = None;
            self.ready = false;
        }
        if self.ready {
            return;
        }
        if !self.background {
            let (r, ms) = LodProxy::build_now(project);
            self.install(r, ms);
            return;
        }
        if self.collect(revision) {
            return;
        }
        if self.building.is_some() {
            return;
        }
        let p = project.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let tx2 = tx.clone();
        let p2 = p.clone();
        // `thread::spawn` panics where threads do not exist (wasm32-unknown-unknown); the builder reports an error
        // instead and the levels are then built on the calling thread
        let spawned = std::thread::Builder::new().name("osatura-lod".into()).spawn(move || {
            let _ = tx2.send(LodProxy::build_now(&p2));
        });
        if spawned.is_err() {
            let _ = tx.send(LodProxy::build_now(&p));
        }
        self.building = Some((revision, rx));
    }

    /// Wait (up to `timeout_ms`) for a background build of the current revision; true when the levels are ready.
    pub fn wait_ready(&mut self, project: &crate::project::Project, revision: u64, timeout_ms: u64) -> bool {
        let t0 = crate::clock::Stamp::now();
        loop {
            self.sync(project, revision);
            if self.ready {
                return true;
            }
            if t0.elapsed().as_millis() as u64 > timeout_ms {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    /// The level to draw for a screen showing the project at `screen_scale` pixels per stage unit when
    /// the picture may differ by at most `max_px` pixels: `(level, declared bound in screen pixels)`.
    /// Level 0 (the project itself, bound 0) when nothing coarser is within budget, no levels exist or
    /// they are still being built.
    pub fn pick(&mut self, project: &crate::project::Project, revision: u64, screen_scale: f64, max_px: f64) -> (u8, f64) {
        self.sync(project, revision);
        let k = crate::lod::pick_level(&self.records, screen_scale, max_px);
        if k == 0 {
            return (0, 0.0);
        }
        (k.min(255) as u8, self.records[k - 1].bound_stage * screen_scale)
    }

    /// The engine of level `k >= 1` for the revision last used, made on first use.
    pub fn engine(&mut self, k: u8, asset_dir: &std::path::Path) -> Result<&Engine, String> {
        let i = (k as usize).checked_sub(1).filter(|i| *i < self.engines.len()).ok_or_else(|| format!("no level of detail {k}"))?;
        if self.engines[i].is_none() {
            let p = self.projects[i].clone().ok_or("level was released")?;
            self.engines[i] = Some(Engine::with_asset_dir(p, asset_dir).map_err(|e| format!("level {k} does not validate: {}", e.join("; ")))?);
            self.engines_made += 1;
        }
        Ok(self.engines[i].as_ref().expect("just made"))
    }
}

/// Least-recently-used cache of rendered frames, tied to a project revision: the first access made with a
/// different revision empties it, so an edit can never be answered with a stale picture.
pub struct FrameCache {
    max_entries: usize,
    max_bytes: usize,
    revision: u64,
    tick: u64,
    bytes: usize,
    map: HashMap<CacheKey, (Arc<Vec<u8>>, u64)>,
    pub hits: u64,
    pub misses: u64,
    pub invalidations: u64,
}

impl FrameCache {
    pub fn new(max_entries: usize, max_bytes: usize) -> FrameCache {
        FrameCache { max_entries: max_entries.max(1), max_bytes, revision: 0, tick: 0, bytes: 0, map: HashMap::new(), hits: 0, misses: 0, invalidations: 0 }
    }

    fn sync(&mut self, revision: u64) {
        if revision != self.revision {
            if !self.map.is_empty() {
                self.invalidations += 1;
            }
            self.map.clear();
            self.bytes = 0;
            self.revision = revision;
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn get(&mut self, revision: u64, key: CacheKey) -> Option<Arc<Vec<u8>>> {
        self.sync(revision);
        self.tick += 1;
        let t = self.tick;
        match self.map.get_mut(&key) {
            Some(e) => {
                e.1 = t;
                self.hits += 1;
                Some(e.0.clone())
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    pub fn put(&mut self, revision: u64, key: CacheKey, data: Vec<u8>) -> Arc<Vec<u8>> {
        self.sync(revision);
        self.tick += 1;
        let a = Arc::new(data);
        if let Some(old) = self.map.insert(key, (a.clone(), self.tick)) {
            self.bytes -= old.0.len();
        }
        self.bytes += a.len();
        while self.map.len() > self.max_entries || (self.bytes > self.max_bytes && self.map.len() > 1) {
            let Some(oldest) = self.map.iter().min_by_key(|(_, v)| v.1).map(|(k, _)| *k) else { break };
            if let Some(old) = self.map.remove(&oldest) {
                self.bytes -= old.0.len();
            }
        }
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    fn eng() -> Engine {
        Engine::new(demo::project()).unwrap()
    }

    #[test]
    fn levels_scale_the_resolution() {
        assert_eq!(proxy_scale(0), 1.0);
        assert_eq!(proxy_scale(1), 0.5);
        assert_eq!(proxy_scale(3), 0.125);
        let e = eng();
        let (w0, h0, p0) = render_proxy_rgba(&e, 5.0, 0).unwrap();
        let (w2, h2, p2) = render_proxy_rgba(&e, 5.0, 2).unwrap();
        assert_eq!((w0, h0), (640, 480));
        assert_eq!((w2, h2), (160, 120));
        assert_eq!(p0.len(), 640 * 480 * 4);
        assert_eq!(p2.len(), 160 * 120 * 4);
        assert!(render_proxy_rgba(&e, 5.0, 5).is_err());
        assert!(render_proxy_png(&e, 5.0, 1, 0.0, false).is_err());
    }

    #[test]
    fn level_zero_is_the_exact_full_render() {
        let e = eng();
        let (_, _, full) = crate::raster::render_rgba(&e, 7.0, &RasterOptions::default());
        let (_, _, p0) = render_proxy_rgba(&e, 7.0, 0).unwrap();
        assert_eq!(full, p0);
    }

    #[test]
    fn proxies_are_deterministic_and_resemble_the_full_frame() {
        let e = eng();
        for level in 1..=MAX_LEVEL {
            let a = render_proxy_png(&e, 11.0, level, 1.0, false).unwrap();
            let b = render_proxy_png(&e, 11.0, level, 1.0, false).unwrap();
            assert_eq!(a, b, "level {level} must be bit-identical on repeat");
        }
        // mean colour stays close to the full frame: tightly at level 1 (nothing is flattened), loosely at 2
        // (gradients become flat colours, which shifts the sky and sun)
        let (_, _, full) = render_proxy_rgba(&e, 11.0, 0).unwrap();
        let mean = |v: &[u8], c: usize| v.chunks_exact(4).map(|p| p[c] as f64).sum::<f64>() / (v.len() / 4) as f64;
        for (level, tol) in [(1u8, 4.0), (2u8, 40.0)] {
            let (_, _, small) = render_proxy_rgba(&e, 11.0, level).unwrap();
            for c in 0..3 {
                assert!((mean(&full, c) - mean(&small, c)).abs() < tol, "level {level} channel {c}: {} vs {}", mean(&full, c), mean(&small, c));
            }
        }
    }

    #[test]
    fn simplification_drops_fine_detail_by_level() {
        let e = eng();
        let count = |f: &Frame| f.nodes.iter().flat_map(|n| &n.paths).filter(|p| p.stroke.is_some()).count();
        let grads = |f: &Frame| f.nodes.iter().flat_map(|n| &n.paths).filter(|p| matches!(p.fill, Some(Paint::Linear { .. }) | Some(Paint::Radial { .. }))).count();
        let full = e.evaluate(3.0);
        let mut l1 = e.evaluate(3.0);
        simplify_frame(&mut l1, 1, 0.5);
        let mut l4 = e.evaluate(3.0);
        simplify_frame(&mut l4, 4, proxy_scale(4));
        assert!(grads(&full) > 0, "the demo has gradient fills");
        assert_eq!(grads(&l1), grads(&full), "gradients survive level 1");
        assert_eq!(grads(&l4), 0, "gradients are flat at level 4");
        assert!(count(&l4) < count(&full), "hairlines vanish at 1/16 size");
        assert!(l4.nodes.iter().all(|n| n.mask.is_none() && n.blend == crate::style::Blend::Normal));
        assert!(l1.nodes.iter().flat_map(|n| &n.paths).all(|p| p.stroke.as_ref().map(|s| s.dash.is_empty() && s.width_profile.is_empty()).unwrap_or(true)));
    }

    #[test]
    fn vector_proxy_is_valid_smaller_svg_and_level_zero_is_exact() {
        let e = eng();
        let full = crate::svg::render_frame(&e, 4.0, &crate::svg::RenderOptions::default());
        assert_eq!(proxy_svg_vector(&e, 4.0, 0, false).unwrap(), full);
        let v = proxy_svg_vector(&e, 4.0, 3, false).unwrap();
        assert!(v.starts_with("<svg") && v.trim_end().ends_with("</svg>"));
        assert!(v.len() < full.len(), "{} vs {}", v.len(), full.len());
        assert!(!v.contains("<linearGradient") && !v.contains("<radialGradient") && !v.contains("mix-blend-mode"));
        assert_eq!(v, proxy_svg_vector(&e, 4.0, 3, false).unwrap());
        assert!(proxy_svg_vector(&e, 4.0, 7, false).is_err());
    }

    #[test]
    fn proxy_svg_wraps_a_png_at_project_size() {
        let e = eng();
        let s = proxy_svg(&e, 2.0, 2, false).unwrap();
        assert!(s.contains("viewBox=\"0 0 640 480\"") && s.contains("data:image/png;base64,"));
        assert!(proxy_svg(&e, 2.0, 9, false).is_err());
    }

    #[test]
    fn cache_hits_lru_eviction_and_revision_invalidation() {
        let mut c = FrameCache::new(2, 1 << 20);
        let k = |f: f32| CacheKey::new(f, 1, false, 0);
        assert!(c.get(1, k(0.0)).is_none());
        c.put(1, k(0.0), vec![0; 10]);
        c.put(1, k(1.0), vec![1; 10]);
        assert_eq!(c.get(1, k(0.0)).unwrap()[0], 0); // touch 0 -> 1 is now the oldest
        c.put(1, k(2.0), vec![2; 10]);
        assert_eq!(c.len(), 2);
        assert!(c.get(1, k(1.0)).is_none(), "least recently used entry evicted");
        assert!(c.get(1, k(0.0)).is_some() && c.get(1, k(2.0)).is_some());
        // a different level or bones flag is a different picture
        assert!(c.get(1, CacheKey::new(0.0, 2, false, 0)).is_none());
        assert!(c.get(1, CacheKey::new(0.0, 1, true, 0)).is_none());
        assert!(c.get(1, CacheKey::new(0.0, 1, false, 2)).is_none());
        // any edit bumps the revision: everything goes
        assert!(c.get(2, k(0.0)).is_none());
        assert!(c.is_empty() && c.bytes() == 0 && c.invalidations == 1);
        c.put(2, k(0.0), vec![9; 4]);
        assert_eq!(c.get(2, k(0.0)).unwrap().as_slice(), &[9, 9, 9, 9]);
        assert!(c.hits >= 4 && c.misses >= 4);
    }

    #[test]
    fn cache_respects_the_byte_budget() {
        let mut c = FrameCache::new(100, 25);
        for i in 0..5 {
            c.put(1, CacheKey::new(i as f32, 0, false, 0), vec![0; 10]);
        }
        assert!(c.bytes() <= 25 && c.len() == 2, "{} bytes, {} entries", c.bytes(), c.len());
        // one oversize entry is still kept (the newest picture is always servable)
        c.put(1, CacheKey::new(9.0, 0, false, 0), vec![0; 100]);
        assert_eq!(c.len(), 1);
    }

    // ---- LodProxy ----

    fn heavy(layers: usize, anchors: usize) -> crate::project::Project {
        use crate::math::Vec2;
        use crate::path::{Anchor, VPath};
        use crate::project::{Layer, Project};
        let mut p = Project::new("heavy", 640.0, 480.0, 24.0, 24);
        for l in 0..layers {
            // a smooth wavy closed outline sampled at many corner points: lots of anchors a certified simplifier may drop
            let (cx, cy, r) = (60.0 + (l % 8) as f32 * 70.0, 60.0 + (l / 8) as f32 * 70.0, 28.0);
            let pts: Vec<Anchor> = (0..anchors)
                .map(|i| {
                    let a = i as f32 / anchors as f32 * std::f32::consts::TAU;
                    Anchor::corner(Vec2::new(cx + (r + 3.0 * (3.0 * a).sin()) * a.cos(), cy + (r + 3.0 * (3.0 * a).sin()) * a.sin()))
                })
                .collect();
            p.layers.push(Layer::vector(&format!("blob{l}"), vec![VPath::new("p", true, pts).with_fill("#3a7bd5").with_stroke("#123", 1.5)]));
        }
        p
    }

    #[test]
    fn the_picked_level_never_exceeds_the_requested_pixels_and_levels_are_built_lazily_per_revision() {
        let p = heavy(3, 30);
        let eng0 = Engine::new(p.clone()).unwrap();
        let mut lp = LodProxy::new();
        assert!(lp.records().is_empty() && lp.builds == 0, "nothing is built before the first request");
        let mut picked = std::collections::BTreeSet::new();
        for screen in [0.05f64, 0.1, 0.25, 0.5, 1.0, 2.0, 4.0] {
            for max_px in [0.0f64, 0.1, 0.5, 1.0, 2.0, 5.0] {
                let (k, bound_px) = lp.pick(&p, 1, screen, max_px);
                assert!(bound_px <= max_px + 1e-12, "level {k} at scale {screen}: declared {bound_px} px > requested {max_px}");
                if k > 0 {
                    // the declared bound of the record, scaled to the screen, is what pick reports
                    let rec = &lp.records()[k as usize - 1];
                    assert_eq!(rec.level, k as usize);
                    assert!(rec.bound_stage * screen <= max_px + 1e-12);
                    // and the next coarser level (if any) would not fit
                    if let Some(next) = lp.records().get(k as usize) {
                        assert!(next.bound_stage * screen > max_px, "level {} would also have fit", k + 1);
                    }
                } else {
                    assert_eq!(bound_px, 0.0);
                }
                picked.insert(k);
            }
        }
        assert_eq!(lp.builds, 1, "one build serves every screen size");
        assert_eq!(lp.engines_made, 0, "engines are made only when a level is drawn");
        assert!(picked.len() >= 3 && picked.contains(&0) && picked.iter().any(|k| *k >= 2), "different budgets give different levels: {picked:?}");
        // the coarsest picked level really is smaller, and drawing from it makes an engine once
        let k = *picked.iter().max().unwrap();
        let e = lp.engine(k, std::path::Path::new(".")).unwrap();
        assert!(crate::lod::count_stats(&e.project).path_anchors < crate::lod::count_stats(&eng0.project).path_anchors / 2, "the coarsest level keeps far fewer anchors");
        lp.engine(k, std::path::Path::new(".")).unwrap();
        assert_eq!(lp.engines_made, 1);
        assert!(lp.engine(0, std::path::Path::new(".")).is_err() && lp.engine(99, std::path::Path::new(".")).is_err());
        // an edit (new revision) throws the levels away and rebuilds on demand
        lp.pick(&p, 2, 0.5, 1.0);
        assert_eq!((lp.builds, lp.invalidations, lp.engines_made), (2, 1, 1));
        lp.pick(&p, 2, 0.25, 3.0);
        assert_eq!(lp.builds, 2, "same revision, no rebuild");
        // bad numbers are level 0, not a panic
        assert_eq!(lp.pick(&p, 2, f64::NAN, 1.0).0, 0);
        assert_eq!(lp.pick(&p, 2, 0.5, f64::NAN).0, 0);
        assert_eq!(lp.pick(&p, 2, -1.0, 1.0).0, 0);
    }

    #[test]
    fn a_picked_level_draws_close_to_the_full_picture_and_cheaper_and_a_hopeless_project_falls_back() {
        let p = heavy(4, 40);
        let full = Engine::new(p.clone()).unwrap();
        let mut lp = LodProxy::new();
        let screen = 0.5; // shown at half size
        let (k, bound) = lp.pick(&p, 7, screen, 1.0);
        assert!(k >= 1 && bound <= 1.0, "{k} {bound}");
        let lo = lp.engine(k, std::path::Path::new(".")).unwrap();
        // same proxy level for both: the pictures differ only by the level of detail
        let (_, _, a) = render_proxy_rgba(&full, 5.0, 1).unwrap();
        let (_, _, b) = render_proxy_rgba(lo, 5.0, 1).unwrap();
        let diff = a.iter().zip(&b).map(|(x, y)| (*x as i32 - *y as i32).unsigned_abs() as f64).sum::<f64>() / a.len() as f64;
        assert!(diff < 2.0, "mean absolute channel difference {diff}");
        // speed: frame evaluation is cheaper with fewer anchors (reported, only loosely asserted: timing varies)
        let t = |e: &Engine| {
            let t0 = crate::clock::Stamp::now();
            for f in 0..6 {
                let _ = render_proxy_rgba(e, f as f32, 1).unwrap();
            }
            t0.elapsed().as_secs_f64() * 1000.0 / 6.0
        };
        let (tf, tl) = (t(&full), t(lo));
        println!("LOD scrub proxy: full {tf:.2} ms/frame, level {k} {tl:.2} ms/frame ({:.2}x)", tf / tl.max(1e-9));
        assert!(tl < tf * 1.5, "the reduced copy must not be slower: {tf} vs {tl}");
        // a project that cannot be reduced (here: one that does not validate for build) is simply drawn in full
        let mut bad = heavy(1, 8);
        bad.fps = 0.0;
        let mut lp2 = LodProxy::new();
        assert_eq!(lp2.pick(&bad, 1, 0.5, 1.0), (0, 0.0));
        assert!(lp2.unavailable.is_some());
    }

    #[test]
    fn the_background_build_answers_level_zero_until_ready_and_drops_results_of_old_revisions() {
        let p = heavy(2, 30);
        let mut lp = LodProxy::background();
        let first = lp.pick(&p, 1, 0.25, 4.0);
        assert_eq!(first, (0, 0.0), "nothing is ready yet: the full project is drawn");
        assert!(lp.is_building() || lp.is_ready());
        // an edit arrives while the first build runs: its result is not used for revision 2
        let first_ready = lp.pick(&p, 2, 0.25, 4.0);
        assert_eq!(first_ready.0, 0);
        assert!(lp.wait_ready(&p, 2, 120_000), "the build for revision 2 finishes");
        assert!(lp.builds >= 1 && lp.invalidations >= 1);
        let (k, b) = lp.pick(&p, 2, 0.25, 4.0);
        assert!(k >= 1 && b <= 4.0, "{k} {b}");
        assert!(!lp.is_building());
        // a failing project is "ready" with a reason and keeps answering level 0
        let mut bad = heavy(1, 8);
        bad.fps = 0.0;
        let mut lq = LodProxy::background();
        assert!(lq.wait_ready(&bad, 5, 60_000));
        assert!(lq.unavailable.is_some() && lq.pick(&bad, 5, 0.5, 1.0) == (0, 0.0));
    }

    #[test]
    fn cache_keys_separate_lod_levels() {
        let mut c = FrameCache::new(8, 1 << 20);
        let a = CacheKey::new(1.0, 2, false, 0);
        c.put(1, a, vec![1]);
        assert!(c.get(1, a).is_some() && c.get(1, a.with_lod(3)).is_none());
        c.put(1, a.with_lod(3), vec![2]);
        assert_eq!(c.get(1, a).unwrap()[0], 1);
        assert_eq!(c.get(1, a.with_lod(3)).unwrap()[0], 2);
    }
}
