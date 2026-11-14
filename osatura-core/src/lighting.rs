// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Per-pixel light contribution with distance attenuation, a cone-and-exponent spot light and a Blinn specular term.
//!
//! Basis (public): the light model of the classic fixed-function graphics API as described in its manual page for the light-parameter
//! call (attenuation `1 / (kc + kl d + kq d^2)`, spot cutoff angle with allowed values 0..90 or 180, spot exponent 0..128, intensity scaled
//! by the cosine of the angle to the spot axis raised to the exponent); Lambert's cosine law for the diffuse term; the half-vector
//! specular model of J. F. Blinn (1977). Only the manual page was read; the other two are textbook knowledge. Spec:
//! docs/clean-room/specs/09-iluminare-spot-si-speculara.md.
//!
//! The viewer is orthographic above the image plane, looking down: the vector to the viewer is `(0, 0, 1)` (a project decision). This
//! module is separate from the default model of the `relight` post effect in `postfx` (which has its own soft-edged spot); a `relight`
//! light uses this one only when it has the optional `shading` field (see docs/clean-room/specs/INTREBARI.md). Shadows are out of scope.

/// A 3D vector.
pub type V3 = [f64; 3];

/// A light source above the image plane.
#[derive(Clone, Debug, PartialEq)]
pub struct LightSpec {
    /// Position; `z` must be above the plane (> 0).
    pub pos: V3,
    /// Linear RGB colour of the light.
    pub color: V3,
    pub intensity: f64,
    /// Attenuation factors `(kc, kl, kq)`, each >= 0 and not all zero. `(1, 0, 0)` = none.
    pub attenuation: (f64, f64, f64),
    /// Spot axis (non-zero); used only when `cutoff_deg` is not 180.
    pub direction: V3,
    /// Cone half-angle in degrees: 0..=90, or exactly 180 for "not a spot" (the default).
    pub cutoff_deg: f64,
    /// Spot exponent 0..=128 (0 = uniform inside the cone).
    pub exponent: f64,
}

impl LightSpec {
    /// A white point light of intensity 1 without attenuation.
    pub fn point(pos: V3) -> LightSpec {
        LightSpec { pos, color: [1.0; 3], intensity: 1.0, attenuation: (1.0, 0.0, 0.0), direction: [0.0, 0.0, -1.0], cutoff_deg: 180.0, exponent: 0.0 }
    }

    /// Checks every parameter against its range.
    pub fn validate(&self) -> Result<(), String> {
        let finite = self.pos.iter().chain(&self.color).chain(&self.direction).all(|v| v.is_finite())
            && self.intensity.is_finite()
            && self.cutoff_deg.is_finite()
            && self.exponent.is_finite()
            && [self.attenuation.0, self.attenuation.1, self.attenuation.2].iter().all(|v| v.is_finite());
        if !finite {
            return Err("light parameters must be finite".into());
        }
        if self.pos[2] <= 0.0 {
            return Err(format!("the light must be above the plane (z > 0), got z = {}", self.pos[2]));
        }
        let (kc, kl, kq) = self.attenuation;
        if kc < 0.0 || kl < 0.0 || kq < 0.0 {
            return Err("attenuation factors must be >= 0".into());
        }
        if kc == 0.0 && kl == 0.0 && kq == 0.0 {
            return Err("attenuation factors cannot all be zero".into());
        }
        if !((0.0..=90.0).contains(&self.cutoff_deg) || self.cutoff_deg == 180.0) {
            return Err(format!("the cutoff angle must be in [0, 90] or 180, got {}", self.cutoff_deg));
        }
        if !(0.0..=128.0).contains(&self.exponent) {
            return Err(format!("the spot exponent must be in [0, 128], got {}", self.exponent));
        }
        if self.cutoff_deg != 180.0 && dot(self.direction, self.direction) == 0.0 {
            return Err("the spot direction must not be the zero vector".into());
        }
        if self.color.iter().any(|c| *c < 0.0) || self.intensity < 0.0 {
            return Err("colour and intensity must be >= 0".into());
        }
        Ok(())
    }
}

/// Surface material for the specular term.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    /// Specular coefficient, >= 0.
    pub ks: f64,
    /// Shininess exponent, >= 0.
    pub shininess: f64,
}

/// Contribution of one or more lights to a pixel.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Contribution {
    pub diffuse: V3,
    pub specular: V3,
}

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn norm(a: V3) -> Option<V3> {
    let l = dot(a, a).sqrt();
    if l > 0.0 && l.is_finite() {
        Some([a[0] / l, a[1] / l, a[2] / l])
    } else {
        None
    }
}

/// Attenuation `1 / (kc + kl d + kq d^2)` at distance `d`.
pub fn attenuation(l: &LightSpec, d: f64) -> f64 {
    let (kc, kl, kq) = l.attenuation;
    1.0 / (kc + kl * d + kq * d * d)
}

/// Spot factor at `point`: 1 for a light without cone; otherwise 0 when the angle between the axis and the direction from the light to
/// the point is above the cutoff (compared with a tolerance of 1e-6 degrees, so a point exactly on the limit is inside), else
/// `cos(angle)^exponent`.
pub fn spot_factor(l: &LightSpec, point: V3) -> Result<f64, String> {
    if l.cutoff_deg == 180.0 {
        return Ok(1.0);
    }
    let axis = norm(l.direction).ok_or("the spot direction must not be the zero vector")?;
    let to_point = norm(sub(point, l.pos)).ok_or("the point coincides with the light")?;
    let c = dot(axis, to_point).clamp(-1.0, 1.0);
    let angle = c.acos().to_degrees();
    if angle > l.cutoff_deg + 1e-6 {
        return Ok(0.0);
    }
    Ok(if l.exponent == 0.0 { 1.0 } else { c.max(0.0).powf(l.exponent) })
}

/// Checks the material coefficients.
pub fn validate_material(m: &Material) -> Result<(), String> {
    if !(m.ks.is_finite() && m.shininess.is_finite()) || m.ks < 0.0 || m.shininess < 0.0 {
        return Err("material coefficients must be finite and >= 0".into());
    }
    Ok(())
}

/// Diffuse and specular contribution of one light at `point` with surface `normal` (normalised here; a zero normal is an error).
pub fn shade(point: V3, normal: V3, l: &LightSpec, m: &Material) -> Result<Contribution, String> {
    l.validate()?;
    validate_material(m)?;
    shade_prevalidated(point, normal, l, m)
}

/// [`shade`] for callers that checked the light (`LightSpec::validate`) and the material (`validate_material`) once and shade many points
/// (the `relight` effect): the per-point work is only the finiteness check of `point` and `normal`.
pub fn shade_prevalidated(point: V3, normal: V3, l: &LightSpec, m: &Material) -> Result<Contribution, String> {
    if !point.iter().chain(&normal).all(|v| v.is_finite()) {
        return Err("point and normal must be finite".into());
    }
    let n = norm(normal).ok_or("the surface normal must not be the zero vector")?;
    let to_light = sub(l.pos, point);
    let dist = dot(to_light, to_light).sqrt();
    let lv = norm(to_light).ok_or("the point coincides with the light")?;
    let nl = dot(n, lv);
    if nl <= 0.0 {
        return Ok(Contribution::default());
    }
    let k = l.intensity * attenuation(l, dist) * spot_factor(l, point)?;
    let diff = nl * k;
    let h = norm([lv[0], lv[1], lv[2] + 1.0]).ok_or("degenerate half vector")?;
    let spec = if m.ks > 0.0 { m.ks * dot(n, h).max(0.0).powf(m.shininess) * k } else { 0.0 };
    Ok(Contribution { diffuse: [l.color[0] * diff, l.color[1] * diff, l.color[2] * diff], specular: [l.color[0] * spec, l.color[1] * spec, l.color[2] * spec] })
}

/// Sum over several lights; the result does not depend on their order (up to floating-point rounding of the sum).
pub fn shade_all(point: V3, normal: V3, lights: &[LightSpec], m: &Material) -> Result<Contribution, String> {
    let mut total = Contribution::default();
    for l in lights {
        let c = shade(point, normal, l, m)?;
        for i in 0..3 {
            total.diffuse[i] += c.diffuse[i];
            total.specular[i] += c.specular[i];
        }
    }
    Ok(total)
}

/// Limits a colour to the valid range `[0, 1]` (done once, after summing).
pub fn clamp_color(c: V3) -> V3 {
    [c[0].clamp(0.0, 1.0), c[1].clamp(0.0, 1.0), c[2].clamp(0.0, 1.0)]
}

#[cfg(test)]
mod tests {
    use super::*;

    const O: V3 = [0.0, 0.0, 0.0];
    const UP: V3 = [0.0, 0.0, 1.0];

    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-4, "{a} vs {b}");
    }
    fn mat(s: f64) -> Material {
        Material { ks: 1.0, shininess: s }
    }

    #[test]
    fn light_straight_above() {
        let c = shade(O, UP, &LightSpec::point([0.0, 0.0, 10.0]), &mat(32.0)).unwrap();
        near(c.diffuse[0], 1.0);
        near(c.specular[0], 1.0);
        assert_eq!(c.diffuse[0], c.diffuse[2]);
    }

    #[test]
    fn oblique_light_specular_exponents() {
        let l = LightSpec::point([10.0, 0.0, 10.0]);
        near(shade(O, UP, &l, &mat(2.0)).unwrap().diffuse[0], std::f64::consts::FRAC_1_SQRT_2);
        near(shade(O, UP, &l, &mat(2.0)).unwrap().specular[0], 0.853553);
        near(shade(O, UP, &l, &mat(32.0)).unwrap().specular[0], 0.079376);
        near(shade(O, UP, &l, &mat(8.0)).unwrap().specular[0], 0.530790);
    }

    #[test]
    fn distance_attenuation() {
        let mut l = LightSpec::point([0.0, 0.0, 10.0]);
        l.attenuation = (1.0, 0.1, 0.0);
        near(attenuation(&l, 10.0), 0.5);
        near(shade(O, UP, &l, &mat(1.0)).unwrap().diffuse[0], 0.5);
    }

    fn spot() -> LightSpec {
        let mut l = LightSpec::point([0.0, 0.0, 10.0]);
        l.direction = [0.0, 0.0, -1.0];
        l.cutoff_deg = 30.0;
        l.exponent = 2.0;
        l
    }

    #[test]
    fn spot_cone() {
        near(spot_factor(&spot(), O).unwrap(), 1.0);
        near(spot_factor(&spot(), [5.7735, 0.0, 0.0]).unwrap(), 0.75);
        assert_eq!(spot_factor(&spot(), [10.0, 0.0, 0.0]).unwrap(), 0.0);
        let c = shade([10.0, 0.0, 0.0], UP, &spot(), &mat(8.0)).unwrap();
        assert_eq!(c.diffuse, [0.0; 3]);
        assert_eq!(c.specular, [0.0; 3]);
        // exponent 0: uniform inside the cone
        let mut u = spot();
        u.exponent = 0.0;
        near(spot_factor(&u, [5.0, 0.0, 0.0]).unwrap(), 1.0);
    }

    #[test]
    fn cutoff_180_is_not_a_spot() {
        let mut l = spot();
        l.cutoff_deg = 180.0;
        near(spot_factor(&l, [10.0, 0.0, 0.0]).unwrap(), 1.0);
        assert!(shade([10.0, 0.0, 0.0], UP, &l, &mat(1.0)).unwrap().diffuse[0] > 0.0);
    }

    #[test]
    fn back_facing_surface_gets_nothing() {
        let l = LightSpec::point([-10.0, 0.0, 10.0]);
        let c = shade(O, [1.0, 0.0, 0.0], &l, &mat(8.0)).unwrap();
        assert_eq!(c, Contribution::default());
    }

    #[test]
    fn parameter_errors() {
        let mut l = spot();
        l.cutoff_deg = 100.0;
        assert!(l.validate().is_err());
        let mut l = spot();
        l.exponent = 200.0;
        assert!(l.validate().is_err());
        let mut l = spot();
        l.attenuation = (1.0, -0.1, 0.0);
        assert!(l.validate().is_err());
        let mut l = spot();
        l.direction = [0.0; 3];
        assert!(l.validate().is_err());
        let mut l = spot();
        l.pos[2] = 0.0;
        assert!(shade(O, UP, &l, &mat(1.0)).is_err());
        let mut l = LightSpec::point([0.0, 0.0, 1.0]);
        l.attenuation = (0.0, 0.0, 0.0);
        assert!(l.validate().is_err());
        assert!(shade(O, [0.0; 3], &LightSpec::point([0.0, 0.0, 1.0]), &mat(1.0)).is_err());
        assert!(shade(O, UP, &LightSpec::point([0.0, 0.0, 1.0]), &Material { ks: -1.0, shininess: 1.0 }).is_err());
        assert!(shade([f64::NAN, 0.0, 0.0], UP, &LightSpec::point([0.0, 0.0, 1.0]), &mat(1.0)).is_err());
    }

    #[test]
    fn normal_is_normalised_and_lights_add_in_any_order() {
        let a = LightSpec::point([10.0, 0.0, 10.0]);
        let mut b = LightSpec::point([0.0, 5.0, 10.0]);
        b.color = [1.0, 0.5, 0.0];
        let s1 = shade_all(O, [0.0, 0.0, 3.0], &[a.clone(), b.clone()], &mat(8.0)).unwrap();
        let s2 = shade_all(O, UP, &[b, a], &mat(8.0)).unwrap();
        for i in 0..3 {
            assert!((s1.diffuse[i] - s2.diffuse[i]).abs() < 1e-12 && (s1.specular[i] - s2.specular[i]).abs() < 1e-12);
        }
        assert_eq!(clamp_color([1.5, -0.2, 0.3]), [1.0, 0.0, 0.3]);
    }

    #[test]
    fn zero_specular_coefficient_gives_no_highlight() {
        let c = shade(O, UP, &LightSpec::point([0.0, 0.0, 10.0]), &Material { ks: 0.0, shininess: 8.0 }).unwrap();
        assert_eq!(c.specular, [0.0; 3]);
        assert!(c.diffuse[0] > 0.0);
    }
}
