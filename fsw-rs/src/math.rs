//! Fixed-size vector, matrix and quaternion kernel (fsw/pseudocode/01_math.md), twin of adcs_math.c.
//! Quaternions are [x y z w] (scalar last), Hamilton; dcm(q) is the passive ECI -> body matrix.
use crate::alg::math as alg;
use crate::m::*;

pub type V3 = [f64; 3];
pub type M3 = [[f64; 3]; 3];
pub type Q = [f64; 4];

pub const PI: f64 = core::f64::consts::PI;
pub const D2R: f64 = PI / 180.0;
/// Earth rotation rate [rad/s], J2 and equatorial radius [m] (adcs_math.h).
pub const OMEGA_E: f64 = 7.2921158553e-5;
pub const J2: f64 = 1.08262668e-3;
pub const RE: f64 = 6378137.0;

#[inline] pub fn dot(a: &V3, b: &V3) -> f64 { a[0]*b[0] + a[1]*b[1] + a[2]*b[2] }
#[inline] pub fn cross(a: &V3, b: &V3) -> V3 { [a[1]*b[2] - a[2]*b[1], a[2]*b[0] - a[0]*b[2], a[0]*b[1] - a[1]*b[0]] }
#[inline] pub fn norm3(a: &V3) -> f64 { sqrt(dot(a, a)) }
pub fn unit(a: &V3) -> V3 { let mut n = norm3(a); if n < 1e-30 { n = 1e-30; } [a[0]/n, a[1]/n, a[2]/n] }
#[inline] pub fn scale3(a: &V3, s: f64) -> V3 { [a[0]*s, a[1]*s, a[2]*s] }
#[inline] pub fn add3(a: &V3, b: &V3) -> V3 { [a[0]+b[0], a[1]+b[1], a[2]+b[2]] }
#[inline] pub fn sub3(a: &V3, b: &V3) -> V3 { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
#[inline] pub fn clamp(x: f64, lo: f64, hi: f64) -> f64 { if x < lo { lo } else if x > hi { hi } else { x } }
#[inline] pub fn sign(x: f64) -> f64 { if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 } }
/// Written from the design: math::maxabs3 (src/alg).
pub fn maxabs3(a: &V3) -> f64 { alg::maxabs3(*a) }
#[inline] pub fn is_zero3(a: &V3) -> bool { a[0] == 0.0 && a[1] == 0.0 && a[2] == 0.0 }

pub fn mat3_vec(m: &M3, v: &V3) -> V3 {
    let mut r = [0.0; 3];
    for i in 0..3 { r[i] = m[i][0]*v[0] + m[i][1]*v[1] + m[i][2]*v[2]; }
    r
}
/// Written from the design: math::mat3t_vec (src/alg).
pub fn mat3t_vec(m: &M3, v: &V3) -> V3 { alg::mat3t_vec(*m, *v) }
pub fn mat3_mul(a: &M3, b: &M3) -> M3 {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { r[i][j] = a[i][0]*b[0][j] + a[i][1]*b[1][j] + a[i][2]*b[2][j]; } }
    r
}
/// Written from the design: math::skew (src/alg).
pub fn skew(a: &V3) -> M3 { alg::skew(*a) }
/// Written from the design: math::det3 (src/alg).
pub fn det3(m: &M3) -> f64 { alg::det3(*m) }
/// Inverse by cofactors; zero matrix (and false) when singular.
/// Written from the design: math::inv3 (src/alg).
pub fn inv3(m: &M3) -> (M3, bool) { alg::inv3(*m) }
/// Minimum-norm right inverse of a 3 x n matrix (n <= 8), rank safe: At (A At + eps I)^-1.
/// Written from the design: math::pinv_rows (src/alg).
pub fn pinv_rows(a: &[[f64; 8]; 3], n: usize) -> [[f64; 3]; 8] { alg::pinv_rows(*a, n as i64) }
/// Symmetric 4x4 eigen decomposition by cyclic Jacobi (12 sweeps); columns of V are eigenvectors.
/// Written from the design: math::jacobi_eig4 (src/alg).
pub fn jacobi_eig4(k: &[[f64; 4]; 4]) -> ([f64; 4], [[f64; 4]; 4]) { alg::jacobi_eig4(*k) }

/// Written from the design: math::qmult (src/alg).
pub fn qmult(a: &Q, b: &Q) -> Q { alg::qmult(*a, *b) }
/// Written from the design: math::qconj (src/alg).
#[inline] pub fn qconj(q: &Q) -> Q { alg::qconj(*q) }
/// Written from the design: math::qnorm (src/alg).
pub fn qnorm(q: &Q) -> Q { alg::qnorm(*q) }
/// Written from the design: math::dcm (src/alg).
pub fn dcm(q: &Q) -> M3 { alg::dcm(*q) }
/// The largest of (trace, R11, R22, R33) selects the stable form (as asils.quat.fromdcm).
/// Written from the design: math::fromdcm (src/alg).
pub fn fromdcm(r: &M3) -> Q { alg::fromdcm(*r) }
/// Written from the design: math::fromrotvec (src/alg).
pub fn fromrotvec(th: &V3) -> Q { alg::fromrotvec(*th) }
/// Written from the design: math::qangle (src/alg).
pub fn qangle(a: &Q, b: &Q) -> f64 { alg::qangle(*a, *b) }
/// Error quaternion conj(q_ref) * q with the scalar made non-negative.
/// Written from the design: math::qerr (src/alg).
pub fn qerr(q_ref: &Q, q: &Q) -> Q { alg::qerr(*q_ref, *q) }

/// A parameter-blob table of small whole numbers (u8) as the design's ints.
pub(crate) fn ints<const N: usize>(a: &[u8; N]) -> [i64; N] {
    let mut r = [0i64; N];
    for i in 0..N { r[i] = a[i] as i64; }
    r
}
