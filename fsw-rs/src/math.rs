//! Fixed-size vector, matrix and quaternion kernel (fsw/pseudocode/01_math.md), twin of adcs_math.c.
//! Quaternions are [x y z w] (scalar last), Hamilton; dcm(q) is the passive ECI -> body matrix.
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
pub fn maxabs3(a: &V3) -> f64 {
    let mut m = fabs(a[0]);
    if fabs(a[1]) > m { m = fabs(a[1]); }
    if fabs(a[2]) > m { m = fabs(a[2]); }
    m
}
#[inline] pub fn is_zero3(a: &V3) -> bool { a[0] == 0.0 && a[1] == 0.0 && a[2] == 0.0 }

pub fn mat3_vec(m: &M3, v: &V3) -> V3 {
    let mut r = [0.0; 3];
    for i in 0..3 { r[i] = m[i][0]*v[0] + m[i][1]*v[1] + m[i][2]*v[2]; }
    r
}
pub fn mat3t_vec(m: &M3, v: &V3) -> V3 {
    let mut r = [0.0; 3];
    for i in 0..3 { r[i] = m[0][i]*v[0] + m[1][i]*v[1] + m[2][i]*v[2]; }
    r
}
pub fn mat3_mul(a: &M3, b: &M3) -> M3 {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { r[i][j] = a[i][0]*b[0][j] + a[i][1]*b[1][j] + a[i][2]*b[2][j]; } }
    r
}
pub fn skew(a: &V3) -> M3 { [[0.0, -a[2], a[1]], [a[2], 0.0, -a[0]], [-a[1], a[0], 0.0]] }
pub fn det3(m: &M3) -> f64 {
    m[0][0]*(m[1][1]*m[2][2] - m[1][2]*m[2][1]) - m[0][1]*(m[1][0]*m[2][2] - m[1][2]*m[2][0])
        + m[0][2]*(m[1][0]*m[2][1] - m[1][1]*m[2][0])
}
/// Inverse by cofactors; zero matrix (and false) when singular.
pub fn inv3(m: &M3) -> (M3, bool) {
    let d = det3(m);
    if fabs(d) < 1e-300 { return ([[0.0; 3]; 3], false); }
    ([[(m[1][1]*m[2][2] - m[1][2]*m[2][1])/d, (m[0][2]*m[2][1] - m[0][1]*m[2][2])/d, (m[0][1]*m[1][2] - m[0][2]*m[1][1])/d],
      [(m[1][2]*m[2][0] - m[1][0]*m[2][2])/d, (m[0][0]*m[2][2] - m[0][2]*m[2][0])/d, (m[0][2]*m[1][0] - m[0][0]*m[1][2])/d],
      [(m[1][0]*m[2][1] - m[1][1]*m[2][0])/d, (m[0][1]*m[2][0] - m[0][0]*m[2][1])/d, (m[0][0]*m[1][1] - m[0][1]*m[1][0])/d]], true)
}
/// Minimum-norm right inverse of a 3 x n matrix (n <= 8), rank safe: At (A At + eps I)^-1.
pub fn pinv_rows(a: &[[f64; 8]; 3], n: usize) -> [[f64; 3]; 8] {
    let mut s = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { for k in 0..n { s[i][j] += a[i][k]*a[j][k]; } } }
    let tr = (s[0][0] + s[1][1] + s[2][2])/3.0;
    let eps = 1e-12*(if tr > 1e-30 { tr } else { 1e-30 });
    for i in 0..3 { s[i][i] += eps; }
    let (si, _) = inv3(&s);
    let mut out = [[0.0; 3]; 8];
    for k in 0..n { for j in 0..3 { out[k][j] = a[0][k]*si[0][j] + a[1][k]*si[1][j] + a[2][k]*si[2][j]; } }
    out
}
/// Symmetric 4x4 eigen decomposition by cyclic Jacobi (12 sweeps); columns of V are eigenvectors.
pub fn jacobi_eig4(k: &[[f64; 4]; 4]) -> ([f64; 4], [[f64; 4]; 4]) {
    const PQ: [[usize; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];
    let mut a = *k;
    let mut v = [[0.0; 4]; 4];
    for i in 0..4 { v[i][i] = 1.0; }
    for _ in 0..12 {
        for pq in PQ.iter() {
            let (p, q) = (pq[0], pq[1]);
            let apq = a[p][q];
            if fabs(apq) < 1e-300 { continue; }
            let th = (a[q][q] - a[p][p])/(2.0*apq);
            let mut t = sign(th)/(fabs(th) + sqrt(th*th + 1.0));
            if th == 0.0 { t = 1.0; }
            let c = 1.0/sqrt(t*t + 1.0);
            let sn = t*c;
            for i in 0..4 { let (aip, aiq) = (a[i][p], a[i][q]); a[i][p] = c*aip - sn*aiq; a[i][q] = sn*aip + c*aiq; }
            for i in 0..4 { let (api, aqi) = (a[p][i], a[q][i]); a[p][i] = c*api - sn*aqi; a[q][i] = sn*api + c*aqi; }
            for i in 0..4 { let (vip, viq) = (v[i][p], v[i][q]); v[i][p] = c*vip - sn*viq; v[i][q] = sn*vip + c*viq; }
        }
    }
    ([a[0][0], a[1][1], a[2][2], a[3][3]], v)
}

pub fn qmult(a: &Q, b: &Q) -> Q {
    [a[3]*b[0] + b[3]*a[0] + (a[1]*b[2] - a[2]*b[1]),
     a[3]*b[1] + b[3]*a[1] + (a[2]*b[0] - a[0]*b[2]),
     a[3]*b[2] + b[3]*a[2] + (a[0]*b[1] - a[1]*b[0]),
     a[3]*b[3] - (a[0]*b[0] + a[1]*b[1] + a[2]*b[2])]
}
#[inline] pub fn qconj(q: &Q) -> Q { [-q[0], -q[1], -q[2], q[3]] }
pub fn qnorm(q: &Q) -> Q {
    let n = sqrt(q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]);
    if n < 1e-30 { return [0.0, 0.0, 0.0, 1.0]; }
    [q[0]/n, q[1]/n, q[2]/n, q[3]/n]
}
pub fn dcm(q: &Q) -> M3 {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    [[1.0 - 2.0*(y*y + z*z), 2.0*(x*y + z*w), 2.0*(x*z - y*w)],
     [2.0*(y*x - z*w), 1.0 - 2.0*(x*x + z*z), 2.0*(y*z + x*w)],
     [2.0*(z*x + y*w), 2.0*(z*y - x*w), 1.0 - 2.0*(x*x + y*y)]]
}
/// The largest of (trace, R11, R22, R33) selects the stable form (as asils.quat.fromdcm).
pub fn fromdcm(r: &M3) -> Q {
    let tr = r[0][0] + r[1][1] + r[2][2];
    let (mut idx, mut best) = (0, tr);
    if r[0][0] > best { best = r[0][0]; idx = 1; }
    if r[1][1] > best { best = r[1][1]; idx = 2; }
    if r[2][2] > best { idx = 3; }
    let q = match idx {
        0 => [r[1][2] - r[2][1], r[2][0] - r[0][2], r[0][1] - r[1][0], 1.0 + tr],
        1 => [1.0 + 2.0*r[0][0] - tr, r[0][1] + r[1][0], r[0][2] + r[2][0], r[1][2] - r[2][1]],
        2 => [r[1][0] + r[0][1], 1.0 + 2.0*r[1][1] - tr, r[1][2] + r[2][1], r[2][0] - r[0][2]],
        _ => [r[2][0] + r[0][2], r[2][1] + r[1][2], 1.0 + 2.0*r[2][2] - tr, r[0][1] - r[1][0]],
    };
    qnorm(&q)
}
pub fn fromrotvec(th: &V3) -> Q {
    let a = norm3(th);
    if a < 1e-12 { return qnorm(&[th[0]/2.0, th[1]/2.0, th[2]/2.0, 1.0]); }
    let s = sin(a/2.0);
    [s*th[0]/a, s*th[1]/a, s*th[2]/a, cos(a/2.0)]
}
pub fn qangle(a: &Q, b: &Q) -> f64 {
    let d = fabs(a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3]);
    2.0*acos(if d > 1.0 { 1.0 } else { d })
}
/// Error quaternion conj(q_ref) * q with the scalar made non-negative.
pub fn qerr(q_ref: &Q, q: &Q) -> Q {
    let e = qmult(&qconj(q_ref), q);
    if e[3] < 0.0 { [-e[0], -e[1], -e[2], -e[3]] } else { e }
}
