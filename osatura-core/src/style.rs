// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Colours, paints (solid / gradients), stroke styles and blend modes.

use crate::math::Vec2;
use serde::{Deserialize, Serialize};

/// Straight (non-premultiplied) RGBA, each channel 0..1.
/// Serialised as a CSS-like hex string (`#rgb`, `#rrggbb`, `#rrggbbaa`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const BLACK: Rgba = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Rgba = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Rgba = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Rgba { r, g, b, a }
    }

    pub fn parse(s: &str) -> Option<Rgba> {
        let s = s.trim();
        if let Some(hex) = s.strip_prefix('#') {
            let nib = |i: usize| {
                u8::from_str_radix(&hex[i..i + 1], 16)
                    .ok()
                    .map(|v| (v * 17) as f32 / 255.0)
            };
            let byte = |i: usize| {
                u8::from_str_radix(&hex[i..i + 2], 16)
                    .ok()
                    .map(|v| v as f32 / 255.0)
            };
            if !hex.is_ascii() {
                return None;
            }
            return match hex.len() {
                3 => Some(Rgba::new(nib(0)?, nib(1)?, nib(2)?, 1.0)),
                4 => Some(Rgba::new(nib(0)?, nib(1)?, nib(2)?, nib(3)?)),
                6 => Some(Rgba::new(byte(0)?, byte(2)?, byte(4)?, 1.0)),
                8 => Some(Rgba::new(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
                _ => None,
            };
        }
        match s.to_ascii_lowercase().as_str() {
            "black" => Some(Rgba::BLACK),
            "white" => Some(Rgba::WHITE),
            "red" => Some(Rgba::new(1.0, 0.0, 0.0, 1.0)),
            "green" => Some(Rgba::new(0.0, 0.5, 0.0, 1.0)),
            "blue" => Some(Rgba::new(0.0, 0.0, 1.0, 1.0)),
            "yellow" => Some(Rgba::new(1.0, 1.0, 0.0, 1.0)),
            "transparent" => Some(Rgba::TRANSPARENT),
            _ => None,
        }
    }

    pub fn hex(s: &str) -> Rgba {
        Rgba::parse(s).unwrap_or_else(|| panic!("bad colour literal '{s}'"))
    }

    pub fn to_u8(self) -> [u8; 4] {
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        [q(self.r), q(self.g), q(self.b), q(self.a)]
    }

    pub fn to_hex(self) -> String {
        let [r, g, b, a] = self.to_u8();
        if a == 255 {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
        }
    }

    pub fn lerp(self, o: Rgba, t: f32) -> Rgba {
        let l = |a: f32, b: f32| a + (b - a) * t;
        Rgba::new(
            l(self.r, o.r),
            l(self.g, o.g),
            l(self.b, o.b),
            l(self.a, o.a),
        )
    }

    pub fn with_alpha(self, a: f32) -> Rgba {
        Rgba { a, ..self }
    }
}

impl TryFrom<String> for Rgba {
    type Error = String;
    fn try_from(s: String) -> Result<Self, String> {
        Rgba::parse(&s).ok_or_else(|| format!("invalid colour '{s}'"))
    }
}
impl From<Rgba> for String {
    fn from(c: Rgba) -> String {
        c.to_hex()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stop {
    pub offset: f32,
    pub color: Rgba,
}

/// The stops ordered by offset (stable, so equal offsets keep their order). Every renderer and exporter
/// uses this order, as CSS and SVG do for the offsets they get; a document with stops out of order
/// therefore looks the same everywhere (`check` warns about it).
pub fn sorted_stops(stops: &[Stop]) -> Vec<Stop> {
    let mut v = stops.to_vec();
    v.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    v
}

/// Fill source. Gradient geometry is in the layer's local space.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Paint {
    Solid(Rgba),
    Linear {
        from: Vec2,
        to: Vec2,
        stops: Vec<Stop>,
    },
    Radial {
        center: Vec2,
        radius: f32,
        stops: Vec<Stop>,
    },
}

impl Paint {
    pub fn solid(hex: &str) -> Paint {
        Paint::Solid(Rgba::hex(hex))
    }
    pub fn linear(from: Vec2, to: Vec2, stops: &[(f32, &str)]) -> Paint {
        Paint::Linear {
            from,
            to,
            stops: stops
                .iter()
                .map(|&(o, c)| Stop {
                    offset: o,
                    color: Rgba::hex(c),
                })
                .collect(),
        }
    }
    pub fn radial(center: Vec2, radius: f32, stops: &[(f32, &str)]) -> Paint {
        Paint::Radial {
            center,
            radius,
            stops: stops
                .iter()
                .map(|&(o, c)| Stop {
                    offset: o,
                    color: Rgba::hex(c),
                })
                .collect(),
        }
    }
    /// A gradient that has no direction or no extent (a linear one whose end points coincide, a radial
    /// one of radius 0) paints the LAST stop everywhere, as CSS and SVG define it. `Some(colour)` for
    /// those; `None` for solid paints, ordinary gradients and gradients without stops. Every renderer
    /// and exporter uses this one rule.
    pub fn degenerate_color(&self) -> Option<Rgba> {
        let (flat, stops) = match self {
            Paint::Solid(_) => return None,
            Paint::Linear { from, to, stops } => (from.distance(*to) < 1e-6, stops),
            Paint::Radial { radius, stops, .. } => (
                ((*radius).partial_cmp(&(1e-6)) != Some(std::cmp::Ordering::Greater)),
                stops,
            ),
        };
        if flat {
            sorted_stops(stops).last().map(|s| s.color)
        } else {
            None
        }
    }

    /// Flat colour of this paint, or the first stop of a gradient.
    pub fn base_color(&self) -> Rgba {
        match self {
            Paint::Solid(c) => *c,
            Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
                stops.first().map(|s| s.color).unwrap_or(Rgba::BLACK)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cap {
    Butt,
    #[default]
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Join {
    Miter,
    #[default]
    Round,
    Bevel,
}

fn one() -> f32 {
    1.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub paint: Paint,
    #[serde(default = "one")]
    pub width: f32,
    #[serde(default)]
    pub cap: Cap,
    #[serde(default)]
    pub join: Join,
    /// Dash pattern (on, off, on, off …); empty = solid.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dash: Vec<f32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub dash_offset: f32,
    /// Variable-width profile: `[t, factor]` pairs, `t` = fraction of the path
    /// length (0..1), factor multiplies `width`. Empty = constant width.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub width_profile: Vec<[f32; 2]>,
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

impl StrokeStyle {
    pub fn new(paint: Paint, width: f32) -> Self {
        StrokeStyle {
            paint,
            width,
            cap: Cap::Round,
            join: Join::Round,
            dash: Vec::new(),
            dash_offset: 0.0,
            width_profile: Vec::new(),
        }
    }
    pub fn solid(hex: &str, width: f32) -> Self {
        StrokeStyle::new(Paint::solid(hex), width)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Blend {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Add,
}

impl Blend {
    /// CSS `mix-blend-mode` keyword.
    pub fn css(self) -> &'static str {
        match self {
            Blend::Normal => "normal",
            Blend::Multiply => "multiply",
            Blend::Screen => "screen",
            Blend::Overlay => "overlay",
            Blend::Darken => "darken",
            Blend::Lighten => "lighten",
            Blend::ColorDodge => "color-dodge",
            Blend::ColorBurn => "color-burn",
            Blend::HardLight => "hard-light",
            Blend::SoftLight => "soft-light",
            Blend::Difference => "difference",
            Blend::Exclusion => "exclusion",
            Blend::Add => "plus-lighter",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_parsing_and_round_trip() {
        assert_eq!(Rgba::parse("#fff"), Some(Rgba::WHITE));
        assert_eq!(Rgba::parse("#ff000080").unwrap().to_u8(), [255, 0, 0, 128]);
        assert!(Rgba::parse("#12345").is_none());
        assert_eq!(Rgba::hex("#3d2c2e").to_hex(), "#3d2c2e");
        assert_eq!(Rgba::parse("#ééé"), None);
    }

    #[test]
    fn paint_json_shapes() {
        let s: Paint = serde_json::from_str("\"#ff0000\"").unwrap();
        assert!(matches!(s, Paint::Solid(_)));
        let l: Paint = serde_json::from_str(
            r##"{"from":[0,0],"to":[10,0],"stops":[{"offset":0,"color":"#000"},{"offset":1,"color":"#fff"}]}"##,
        )
        .unwrap();
        assert!(matches!(l, Paint::Linear { .. }));
        let r: Paint = serde_json::from_str(
            r##"{"center":[0,0],"radius":5,"stops":[{"offset":0,"color":"#000"}]}"##,
        )
        .unwrap();
        assert!(matches!(r, Paint::Radial { .. }));
        let back = serde_json::to_string(&l).unwrap();
        assert_eq!(serde_json::from_str::<Paint>(&back).unwrap(), l);
    }
}
