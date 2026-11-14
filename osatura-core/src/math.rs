// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Mul, Neg, Sub};

/// A 2D vector / point. Serialized as `[x, y]` for hand-editable JSON.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f32; 2]", into = "[f32; 2]")]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl From<[f32; 2]> for Vec2 {
    fn from(a: [f32; 2]) -> Self {
        Vec2 { x: a[0], y: a[1] }
    }
}
impl From<Vec2> for [f32; 2] {
    fn from(v: Vec2) -> Self {
        [v.x, v.y]
    }
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }
    pub fn dot(self, o: Vec2) -> f32 {
        self.x * o.x + self.y * o.y
    }
    pub fn distance(self, o: Vec2) -> f32 {
        (self - o).length()
    }
    pub fn normalized(self) -> Vec2 {
        let l = self.length();
        if l < 1e-12 {
            Vec2::ZERO
        } else {
            self * (1.0 / l)
        }
    }
    pub fn lerp(self, o: Vec2, t: f32) -> Vec2 {
        self + (o - self) * t
    }
    pub fn from_angle(angle: f32) -> Vec2 {
        Vec2::new(angle.cos(), angle.sin())
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}
impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        self.x += o.x;
        self.y += o.y;
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f32) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}
impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

/// 2×3 affine transform. Maps `(x, y)` to
/// `(a·x + c·y + tx, b·x + d·y + ty)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine2 {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

/// Affine map taking triangle `from` onto triangle `to` (None if `from` is degenerate).
pub fn tri_affine(from: [Vec2; 3], to: [Vec2; 3]) -> Option<Affine2> {
    let src = Affine2 {
        a: from[1].x - from[0].x,
        b: from[1].y - from[0].y,
        c: from[2].x - from[0].x,
        d: from[2].y - from[0].y,
        tx: from[0].x,
        ty: from[0].y,
    };
    let dst = Affine2 {
        a: to[1].x - to[0].x,
        b: to[1].y - to[0].y,
        c: to[2].x - to[0].x,
        d: to[2].y - to[0].y,
        tx: to[0].x,
        ty: to[0].y,
    };
    Some(dst.mul(src.inverse()?))
}

impl Affine2 {
    pub const IDENTITY: Affine2 = Affine2 {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub fn translate(v: Vec2) -> Self {
        Affine2 {
            tx: v.x,
            ty: v.y,
            ..Self::IDENTITY
        }
    }
    pub fn rotate(radians: f32) -> Self {
        let (s, c) = radians.sin_cos();
        Affine2 {
            a: c,
            b: s,
            c: -s,
            d: c,
            tx: 0.0,
            ty: 0.0,
        }
    }
    pub fn scale(s: f32) -> Self {
        Affine2 {
            a: s,
            d: s,
            ..Self::IDENTITY
        }
    }
    pub fn scale_xy(sx: f32, sy: f32) -> Self {
        Affine2 {
            a: sx,
            d: sy,
            ..Self::IDENTITY
        }
    }
    /// Horizontal shear: `x' = x + tan(radians)·y`.
    pub fn shear_x(radians: f32) -> Self {
        Affine2 {
            c: radians.tan(),
            ..Self::IDENTITY
        }
    }

    /// Composition: `self ∘ rhs` — `rhs` is applied first, then `self`.
    // Matrix composition with documented argument order; `std::ops::Mul` would hide that and `mul` is public API.
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, rhs: Affine2) -> Affine2 {
        Affine2 {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            tx: self.a * rhs.tx + self.c * rhs.ty + self.tx,
            ty: self.b * rhs.tx + self.d * rhs.ty + self.ty,
        }
    }

    pub fn apply(&self, p: Vec2) -> Vec2 {
        Vec2::new(
            self.a * p.x + self.c * p.y + self.tx,
            self.b * p.x + self.d * p.y + self.ty,
        )
    }

    /// Rotation of the transformed x axis, in radians (exact for uniform scale).
    pub fn rotation(&self) -> f32 {
        self.b.atan2(self.a)
    }
    /// Length of the transformed x axis.
    pub fn x_scale(&self) -> f32 {
        self.a.hypot(self.b)
    }

    pub fn inverse(&self) -> Option<Affine2> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-12 {
            return None;
        }
        let ia = self.d / det;
        let ib = -self.b / det;
        let ic = -self.c / det;
        let id = self.a / det;
        Some(Affine2 {
            a: ia,
            b: ib,
            c: ic,
            d: id,
            tx: -(ia * self.tx + ic * self.ty),
            ty: -(ib * self.tx + id * self.ty),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec2, b: Vec2) -> bool {
        a.distance(b) < 1e-3
    }

    #[test]
    fn inverse_round_trip() {
        let m = Affine2::translate(Vec2::new(10.0, -4.0))
            .mul(Affine2::rotate(0.7))
            .mul(Affine2::scale(1.8));
        let p = Vec2::new(3.0, 5.0);
        let q = m.inverse().unwrap().apply(m.apply(p));
        assert!(close(p, q));
    }

    #[test]
    fn composition_order() {
        // rotate 90° first, then translate: (1,0) -> (0,1) -> (5,1)
        let m = Affine2::translate(Vec2::new(5.0, 0.0))
            .mul(Affine2::rotate(std::f32::consts::FRAC_PI_2));
        assert!(close(m.apply(Vec2::new(1.0, 0.0)), Vec2::new(5.0, 1.0)));
    }

    #[test]
    fn vec2_json_is_array() {
        let s = serde_json::to_string(&Vec2::new(1.0, 2.0)).unwrap();
        assert_eq!(s, "[1.0,2.0]");
    }
}
