// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Pin warp: drag a few "pins" and the surrounding artwork follows smoothly.
//!
//! Clean-room design note (replaces any as-rigid-as-possible style solver): this is plain
//! scattered-data interpolation with a compactly supported radial basis function
//! (Wendland's C² function, 1995). For pins `p_i` with displacements `o_i` we solve
//! `Φ c = o` once per frame (`Φ_ij = w(|p_i − p_j| / R)`) and move a point by
//! `Σ_j c_j · w(|p − p_j| / R)`. The field interpolates every pin exactly, is C²-smooth,
//! and is exactly zero farther than `R` from every pin, so it is local, cheap, and
//! needs no mesh. `Φ` is positive definite for distinct pins, so its inverse exists.

use crate::math::Vec2;

/// Wendland C² basis `(1−t)⁴ (4t+1)` for `t < 1`, else 0.
pub fn wendland(t: f32) -> f32 {
    if t >= 1.0 {
        0.0
    } else {
        let u = 1.0 - t.max(0.0);
        u * u * u * u * (4.0 * t.max(0.0) + 1.0)
    }
}

/// Precomputed solver for a fixed set of rest-pose pins.
#[derive(Clone, Debug)]
pub struct PinSolver {
    pins: Vec<Vec2>,
    radius: f32,
    /// Row-major inverse of Φ (f64 for conditioning).
    inv: Vec<f64>,
}

impl PinSolver {
    pub const MAX_PINS: usize = 64;

    pub fn new(pins: &[Vec2], radius: f32) -> Result<PinSolver, String> {
        let n = pins.len();
        if n == 0 {
            return Err("needs at least one pin".into());
        }
        if n > Self::MAX_PINS {
            return Err(format!("at most {} pins", Self::MAX_PINS));
        }
        if (radius.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)) || !radius.is_finite() {
            return Err("radius must be positive".into());
        }
        let mut phi = vec![0.0f64; n * n];
        for i in 0..n {
            for j in 0..n {
                phi[i * n + j] = wendland(pins[i].distance(pins[j]) / radius) as f64;
            }
        }
        let inv = invert(&phi, n).ok_or("pins coincide or are too close together")?;
        Ok(PinSolver {
            pins: pins.to_vec(),
            radius,
            inv,
        })
    }

    pub fn len(&self) -> usize {
        self.pins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pins.is_empty()
    }

    /// Displacement of rest-space point `p` for pin displacements `offs`.
    pub fn displacement(&self, offs: &[Vec2], p: Vec2) -> Vec2 {
        let n = self.pins.len();
        let mut phi = [0.0f64; Self::MAX_PINS];
        let mut any = false;
        for (j, ph) in phi.iter_mut().enumerate().take(n) {
            let w = wendland(p.distance(self.pins[j]) / self.radius);
            if w > 0.0 {
                any = true;
            }
            *ph = w as f64;
        }
        if !any {
            return Vec2::ZERO;
        }
        // h = Φ⁻¹ φ(p); displacement = Σ h_i o_i
        let (mut dx, mut dy) = (0.0f64, 0.0f64);
        // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            let mut h = 0.0f64;
            // the index is a coordinate/identifier used beyond one slice (parallel arrays, offsets, neighbours); an iterator would obscure it
            #[allow(clippy::needless_range_loop)]
            for j in 0..n {
                h += self.inv[i * n + j] * phi[j];
            }
            dx += h * offs[i].x as f64;
            dy += h * offs[i].y as f64;
        }
        Vec2::new(dx as f32, dy as f32)
    }
}

/// Gauss–Jordan inverse with partial pivoting; `None` if singular.
pub fn invert(a: &[f64], n: usize) -> Option<Vec<f64>> {
    let mut m = a.to_vec();
    let mut inv = vec![0.0f64; n * n];
    for i in 0..n {
        inv[i * n + i] = 1.0;
    }
    for col in 0..n {
        let mut piv = col;
        for r in col + 1..n {
            if m[r * n + col].abs() > m[piv * n + col].abs() {
                piv = r;
            }
        }
        if m[piv * n + col].abs() < 1e-12 {
            return None;
        }
        if piv != col {
            for k in 0..n {
                m.swap(col * n + k, piv * n + k);
                inv.swap(col * n + k, piv * n + k);
            }
        }
        let d = m[col * n + col];
        for k in 0..n {
            m[col * n + k] /= d;
            inv[col * n + k] /= d;
        }
        for r in 0..n {
            if r != col {
                let f = m[r * n + col];
                if f != 0.0 {
                    for k in 0..n {
                        m[r * n + k] -= f * m[col * n + k];
                        inv[r * n + k] -= f * inv[col * n + k];
                    }
                }
            }
        }
    }
    Some(inv)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    #[test]
    fn interpolates_pins_exactly() {
        let pins = [v(0.0, 0.0), v(40.0, 0.0), v(20.0, 30.0)];
        let offs = [v(5.0, -3.0), v(-2.0, 8.0), v(0.0, 4.0)];
        let s = PinSolver::new(&pins, 100.0).unwrap();
        for i in 0..3 {
            let d = s.displacement(&offs, pins[i]);
            assert!(
                d.distance(offs[i]) < 1e-3,
                "pin {i}: {d:?} vs {:?}",
                offs[i]
            );
        }
    }

    #[test]
    fn is_local_and_smooth() {
        let s = PinSolver::new(&[v(0.0, 0.0)], 50.0).unwrap();
        let offs = [v(10.0, 0.0)];
        assert_eq!(
            s.displacement(&offs, v(60.0, 0.0)),
            Vec2::ZERO,
            "beyond the radius nothing moves"
        );
        let a = s.displacement(&offs, v(10.0, 0.0)).x;
        let b = s.displacement(&offs, v(25.0, 0.0)).x;
        let c = s.displacement(&offs, v(49.0, 0.0)).x;
        assert!(
            a > b && b > c && (0.0..0.01).contains(&c),
            "monotone fall-off to zero: {a} {b} {c}"
        );
    }

    #[test]
    fn zero_offsets_are_identity_and_linear_in_offsets() {
        let pins = [v(0.0, 0.0), v(30.0, 10.0)];
        let s = PinSolver::new(&pins, 80.0).unwrap();
        assert_eq!(s.displacement(&[Vec2::ZERO; 2], v(12.0, 4.0)), Vec2::ZERO);
        let o1 = [v(1.0, 2.0), v(-3.0, 1.0)];
        let o2 = [v(2.0, -1.0), v(0.5, 0.5)];
        let sum = [o1[0] + o2[0], o1[1] + o2[1]];
        let p = v(11.0, 7.0);
        let lhs = s.displacement(&sum, p);
        let rhs = s.displacement(&o1, p) + s.displacement(&o2, p);
        assert!(lhs.distance(rhs) < 1e-4);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(PinSolver::new(&[], 10.0).is_err());
        assert!(
            PinSolver::new(&[v(0.0, 0.0), v(0.0, 0.0)], 10.0).is_err(),
            "coincident pins"
        );
        assert!(PinSolver::new(&[v(0.0, 0.0)], 0.0).is_err());
        assert!(PinSolver::new(&vec![v(0.0, 0.0); 65], 10.0).is_err());
    }

    #[test]
    fn inverse_is_correct() {
        let a = [4.0, 3.0, 6.0, 3.0];
        let inv = invert(&a, 2).unwrap();
        let id = [
            a[0] * inv[0] + a[1] * inv[2],
            a[0] * inv[1] + a[1] * inv[3],
            a[2] * inv[0] + a[3] * inv[2],
            a[2] * inv[1] + a[3] * inv[3],
        ];
        assert!(
            (id[0] - 1.0).abs() < 1e-9
                && id[1].abs() < 1e-9
                && id[2].abs() < 1e-9
                && (id[3] - 1.0).abs() < 1e-9
        );
        assert!(invert(&[1.0, 2.0, 2.0, 4.0], 2).is_none());
    }
}
