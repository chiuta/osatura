// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Options and records (part of `lod`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Options, records

/// Parameters of [`build`].
#[derive(Clone, Debug, PartialEq)]
pub struct LodOptions {
    /// Declared bound of level 1 in output pixels at `scale`.
    pub tolerance_px: f64,
    /// Output pixels per stage unit (render scale), e.g. `1920 / project.width`.
    pub scale: f64,
    /// Number of derived levels (L1..Ln).
    pub levels: usize,
    /// Tolerance multiplier from one level to the next.
    pub ratio: f64,
    /// Share of the tolerance given to path / mesh geometry.
    pub geometry_share: f64,
    /// Share of the tolerance given to animation keys (`geometry_share + keys_share <= 1`).
    pub keys_share: f64,
    /// Layers whose screen extent stays below `cull_share * tolerance` are removed.
    pub cull_share: f64,
}

impl LodOptions {
    pub fn new(tolerance_px: f64, scale: f64, levels: usize) -> LodOptions {
        LodOptions { tolerance_px, scale, levels, ratio: 2.0, geometry_share: 0.5, keys_share: 0.4, cull_share: 0.5 }
    }
    pub fn check(&self) -> Result<(), String> {
        let ok = |x: f64| x.is_finite() && x > 0.0;
        if !ok(self.tolerance_px) {
            return Err("tolerance must be a positive number of pixels".into());
        }
        if !ok(self.scale) || self.scale > 64.0 {
            return Err("scale must be in (0, 64]".into());
        }
        if self.levels == 0 || self.levels > 12 {
            return Err("levels must be between 1 and 12".into());
        }
        if !ok(self.ratio) || self.ratio < 1.0 || self.ratio > 16.0 {
            return Err("ratio must be in [1, 16]".into());
        }
        if !ok(self.geometry_share) || !ok(self.keys_share) || self.geometry_share + self.keys_share > 1.0 + 1e-9 {
            return Err("geometry and keys shares must be positive and sum to at most 1".into());
        }
        if !ok(self.cull_share) || self.cull_share > 1.0 {
            return Err("cull share must be in (0, 1]".into());
        }
        Ok(())
    }
    /// Declared tolerance (px) of level `k >= 1`.
    pub fn level_tolerance(&self, k: usize) -> f64 {
        self.tolerance_px * self.ratio.powi(k as i32 - 1)
    }
}

/// Counts that describe the size of a project (what a level is supposed to shrink).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LodStats {
    /// Path anchors of all vector layers.
    pub path_anchors: usize,
    /// Vertices of all image meshes.
    pub mesh_vertices: usize,
    /// Keys of all keyed channels.
    pub keys: usize,
    /// Layers (all nesting levels).
    pub layers: usize,
    /// Size of the compact JSON in bytes.
    pub bytes: usize,
}

/// Metadata stored in a derived project (`Project::lods`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LodRecord {
    pub level: usize,
    /// Requested tolerance of this level (px at `scale`).
    pub tolerance_px: f64,
    /// Render scale (output pixels per stage unit) the pixel numbers refer to.
    pub scale: f64,
    /// Declared total bound in px at `scale`: max(cull, max(path, mesh) + keys).
    pub declared_px: f64,
    /// The same bound in stage units (multiply by a display scale to get pixels there).
    pub bound_stage: f64,
    pub path_px: f64,
    pub mesh_px: f64,
    pub keys_px: f64,
    pub cull_px: f64,
    pub stats: LodStats,
    /// Removed or reduced things, for the report.
    #[serde(default)]
    pub notes: Vec<String>,
}

/// Level of detail: project plus record.
#[derive(Clone, Debug)]
pub struct Lod {
    pub project: Project,
    pub record: LodRecord,
}

/// Round up to six decimals: short decimal text that survives a JSON round trip and never lowers a bound.
pub(super) fn ceil6(x: f64) -> f64 {
    (x * 1e6).ceil() / 1e6
}

pub fn count_stats(p: &Project) -> LodStats {
    fn walk(ls: &[Layer], s: &mut LodStats) {
        for l in ls {
            s.layers += 1;
            match &l.kind {
                LayerKind::Vector { paths } | LayerKind::Boolean { paths, .. } => s.path_anchors += paths.iter().map(|x| x.anchors.len()).sum::<usize>(),
                LayerKind::Image { mesh: Some(m), .. } => s.mesh_vertices += m.vertex_count(),
                _ => {}
            }
            walk(l.children(), s);
        }
    }
    let mut s = LodStats::default();
    walk(&p.layers, &mut s);
    for c in &p.channels {
        if let Source::Keys { track } = &c.source {
            s.keys += track.keys.len();
        }
    }
    s.bytes = serde_json::to_string(p).map(|t| t.len()).unwrap_or(0);
    s
}
