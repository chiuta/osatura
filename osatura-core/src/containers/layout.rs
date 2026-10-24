// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `containers` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

pub(super) const MAX_DEPTH: usize = 64;

pub(super) const MAX_MEMBERS: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dir {
    #[default]
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Wrap {
    #[default]
    None,
    Wrap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AspectMode {
    /// The rectangle fits inside the available area.
    #[default]
    Fit,
    /// The rectangle covers the available area (and overflows on one axis).
    Fill,
}

pub(super) fn half() -> f32 {
    0.5
}

/// Where a root container's rectangle comes from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RectRule {
    /// Absolute rectangle.
    Fixed { x: f32, y: f32, w: f32, h: f32 },
    /// Pinned to canvas edges. Per axis: both margins give the size (canvas minus margins); one
    /// margin plus `width`/`height` pins that edge; a size alone centres; one margin alone makes
    /// the rectangle extend to the opposite edge; nothing set covers the whole canvas.
    Pin {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        left: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        top: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        right: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bottom: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<f32>,
    },
    /// A rectangle of width / height `ratio` placed in the canvas inset by `margin`; `ax`/`ay`
    /// (0..1, default centred) place it in the leftover space.
    Aspect {
        ratio: f32,
        #[serde(default)]
        mode: AspectMode,
        #[serde(default)]
        margin: f32,
        #[serde(default = "half")]
        ax: f32,
        #[serde(default = "half")]
        ay: f32,
    },
    /// Top-left corner given, size from the content.
    Hug {
        #[serde(default)]
        x: f32,
        #[serde(default)]
        y: f32,
    },
}

impl Default for RectRule {
    /// The whole canvas.
    fn default() -> Self {
        RectRule::Pin { left: None, top: None, right: None, bottom: None, width: None, height: None }
    }
}

pub(super) fn one() -> f32 {
    1.0
}

/// One entry of a container: exactly one of `layer`, `container` or `spacer`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Member {
    /// Layer path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,
    /// Id of a nested container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    /// An empty flexible gap (size from `width` / `height` / `grow`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub spacer: bool,
    /// Share of free main-axis space (0 = none).
    #[serde(default)]
    pub grow: f32,
    /// Share of a main-axis shortfall (weighted by size).
    #[serde(default = "one")]
    pub shrink: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_w: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_w: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_h: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_h: Option<f32>,
    /// Cross-axis alignment override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    /// Scale a layer to its slot (growth, shrinkage and cross-axis stretch change its size).
    /// Without it the layer keeps its size and sits at the start of its slot.
    #[serde(default, skip_serializing_if = "is_false")]
    pub resize: bool,
}

pub(super) fn is_false(b: &bool) -> bool {
    !*b
}

impl Member {
    pub(super) fn blank() -> Member {
        Member {
            layer: None,
            container: None,
            spacer: false,
            grow: 0.0,
            shrink: 1.0,
            width: None,
            height: None,
            min_w: None,
            max_w: None,
            min_h: None,
            max_h: None,
            align: None,
            resize: false,
        }
    }
    pub fn layer(path: &str) -> Member {
        Member { layer: Some(path.into()), ..Member::blank() }
    }
    pub fn nested(id: &str) -> Member {
        Member { container: Some(id.into()), ..Member::blank() }
    }
    pub fn spacer(grow: f32) -> Member {
        Member { spacer: true, grow, ..Member::blank() }
    }
    pub(super) fn label(&self) -> String {
        match (&self.layer, &self.container) {
            (Some(l), _) => format!("layer '{l}'"),
            (_, Some(c)) => format!("container '{c}'"),
            _ => "spacer".into(),
        }
    }
    pub(super) fn fixed(&self, axis: usize) -> Option<f32> {
        if axis == 0 { self.width } else { self.height }
    }
    pub(super) fn limits(&self, axis: usize) -> (f32, f32) {
        let (lo, hi) = if axis == 0 { (self.min_w, self.max_w) } else { (self.min_h, self.max_h) };
        (lo.unwrap_or(0.0), hi.unwrap_or(f32::INFINITY))
    }
    pub(super) fn clamp(&self, axis: usize, v: f32) -> f32 {
        let (lo, hi) = self.limits(axis);
        v.max(lo).min(hi)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub members: Vec<Member>,
    #[serde(default)]
    pub axis: Dir,
    /// Space between neighbours on a line.
    #[serde(default)]
    pub gap: f32,
    /// Space between wrapped lines (default: `gap`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cross_gap: Option<f32>,
    /// Left, top, right, bottom.
    #[serde(default)]
    pub padding: [f32; 4],
    #[serde(default)]
    pub justify: Justify,
    #[serde(default)]
    pub align: Align,
    #[serde(default)]
    pub wrap: Wrap,
    /// Rectangle of a root container (ignored for nested containers and scroll content).
    #[serde(default)]
    pub rect: RectRule,
}

impl Container {
    pub fn new(id: &str, axis: Dir, members: Vec<Member>) -> Container {
        Container {
            id: id.into(),
            members,
            axis,
            gap: 0.0,
            cross_gap: None,
            padding: [0.0; 4],
            justify: Justify::Start,
            align: Align::Start,
            wrap: Wrap::None,
            rect: RectRule::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollAxis {
    #[default]
    Y,
    X,
    Both,
}

pub(super) fn def_stiffness() -> f32 {
    180.0
}

pub(super) fn def_zeta() -> f32 {
    0.9
}

/// Rubber-band overscroll.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Elastic {
    /// Largest displayed overscroll in pixels (the pull saturates towards it).
    pub limit: f32,
    #[serde(default = "def_stiffness")]
    pub stiffness: f32,
    #[serde(default = "def_zeta")]
    pub damping_ratio: f32,
}

pub(super) fn def_fling() -> f32 {
    0.15
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scroll {
    pub id: String,
    /// Viewport rectangle (also the clip rectangle).
    #[serde(default)]
    pub viewport: RectRule,
    /// Id of the content container (must not be a member of another container).
    pub content: String,
    #[serde(default)]
    pub axis: ScrollAxis,
    /// Requested offset in pixels: how far the content is moved up / left.
    #[serde(default)]
    pub offset: [f32; 2],
    /// Overscroll with a rubber band instead of clamping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elastic: Option<Elastic>,
    /// Resting offsets (x axis / y axis) a release settles on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub snap_x: Vec<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub snap_y: Vec<f32>,
    /// Seconds of release velocity projected to find the landing offset.
    #[serde(default = "def_fling")]
    pub fling: f32,
    /// Rectangle layer (normally hidden) fitted to the viewport and used as a mask for every
    /// layer of the content, which clips the content to the viewport.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip_layer: Option<String>,
    /// External value path that drives the offset (pixels along the main axis); see [`apply_inputs`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
}

impl Scroll {
    pub fn new(id: &str, content: &str) -> Scroll {
        Scroll {
            id: id.into(),
            viewport: RectRule::default(),
            content: content.into(),
            axis: ScrollAxis::Y,
            offset: [0.0; 2],
            elastic: None,
            snap_x: vec![],
            snap_y: vec![],
            fling: def_fling(),
            clip_layer: None,
            input: None,
        }
    }
    pub(super) fn scrolls(&self, axis: usize) -> bool {
        match self.axis {
            ScrollAxis::Both => true,
            ScrollAxis::X => axis == 0,
            ScrollAxis::Y => axis == 1,
        }
    }
}
