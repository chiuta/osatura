// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Small affine helpers (f64) (part of `lod`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// Small affine helpers (f64)

/// `[a, b, c, d, tx, ty]` as in [`Affine2`]: `x' = a x + c y + tx`, `y' = b x + d y + ty`.
pub(super) type A = [f64; 6];

pub(super) fn a_of(m: &Affine2) -> A {
    [m.a as f64, m.b as f64, m.c as f64, m.d as f64, m.tx as f64, m.ty as f64]
}
pub(super) fn a_apply(m: &A, p: P) -> P {
    [m[0] * p[0] + m[2] * p[1] + m[4], m[1] * p[0] + m[3] * p[1] + m[5]]
}
pub(super) fn a_mul(l: &A, r: &A) -> A {
    [
        l[0] * r[0] + l[2] * r[1],
        l[1] * r[0] + l[3] * r[1],
        l[0] * r[2] + l[2] * r[3],
        l[1] * r[2] + l[3] * r[3],
        l[0] * r[4] + l[2] * r[5] + l[4],
        l[1] * r[4] + l[3] * r[5] + l[5],
    ]
}
pub(super) fn a_inv(m: &A) -> Option<A> {
    let det = m[0] * m[3] - m[1] * m[2];
    if (det.abs().partial_cmp(&(1e-12)) != Some(std::cmp::Ordering::Greater)) || !det.is_finite() {
        return None;
    }
    let (ia, ib, ic, id) = (m[3] / det, -m[1] / det, -m[2] / det, m[0] / det);
    Some([ia, ib, ic, id, -(ia * m[4] + ic * m[5]), -(ib * m[4] + id * m[5])])
}
/// Largest singular value of the linear part.
pub(super) fn lin_norm(m: &A) -> f64 {
    let (p, q, r) = (m[0] * m[0] + m[1] * m[1], m[0] * m[2] + m[1] * m[3], m[2] * m[2] + m[3] * m[3]);
    let h = (p + r) * 0.5;
    (h + (((p - r) * 0.5).powi(2) + q * q).sqrt()).max(0.0).sqrt()
}
pub(super) fn local_matrix(t: &crate::project::Transform2D) -> A {
    a_of(&t.matrix())
}
pub(super) const A_ID: A = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
