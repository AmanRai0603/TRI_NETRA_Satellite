//! Small fixed-size linear algebra and quaternions ([x y z w], scalar last, Hamilton;
//! dcm(q) is the passive ECI -> body matrix), as asils.quat.
use crate::pm::*;

pub type V3 = [f64; 3];
pub type M3 = [[f64; 3]; 3];
pub type Q = [f64; 4];

#[inline] pub fn dot(a: &V3, b: &V3) -> f64 { a[0]*b[0] + a[1]*b[1] + a[2]*b[2] }
#[inline] pub fn cross(a: &V3, b: &V3) -> V3 { [a[1]*b[2] - a[2]*b[1], a[2]*b[0] - a[0]*b[2], a[0]*b[1] - a[1]*b[0]] }
#[inline] pub fn norm(a: &V3) -> f64 { sqrt(dot(a, a)) }
#[inline] pub fn add(a: &V3, b: &V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
#[inline] pub fn sub(a: &V3, b: &V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
#[inline] pub fn scale(a: &V3, s: f64) -> V3 { [a[0]*s, a[1]*s, a[2]*s] }
pub fn unit(a: &V3) -> V3 { let n = norm(a); if n < 1e-300 { [0.0; 3] } else { scale(a, 1.0/n) } }
pub fn mv(m: &M3, v: &V3) -> V3 { [dot(&m[0], v), dot(&m[1], v), dot(&m[2], v)] }
pub fn mtv(m: &M3, v: &V3) -> V3 {
    [m[0][0]*v[0] + m[1][0]*v[1] + m[2][0]*v[2], m[0][1]*v[0] + m[1][1]*v[1] + m[2][1]*v[2], m[0][2]*v[0] + m[1][2]*v[1] + m[2][2]*v[2]]
}
pub fn mm(a: &M3, b: &M3) -> M3 {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { r[i][j] = a[i][0]*b[0][j] + a[i][1]*b[1][j] + a[i][2]*b[2][j]; } }
    r
}
pub fn transpose(a: &M3) -> M3 { [[a[0][0], a[1][0], a[2][0]], [a[0][1], a[1][1], a[2][1]], [a[0][2], a[1][2], a[2][2]]] }
pub fn diag(d: &V3) -> M3 { [[d[0], 0.0, 0.0], [0.0, d[1], 0.0], [0.0, 0.0, d[2]]] }
pub fn det(m: &M3) -> f64 {
    m[0][0]*(m[1][1]*m[2][2] - m[1][2]*m[2][1]) - m[0][1]*(m[1][0]*m[2][2] - m[1][2]*m[2][0]) + m[0][2]*(m[1][0]*m[2][1] - m[1][1]*m[2][0])
}
pub fn inv(m: &M3) -> M3 {
    let d = det(m);
    [[(m[1][1]*m[2][2] - m[1][2]*m[2][1])/d, (m[0][2]*m[2][1] - m[0][1]*m[2][2])/d, (m[0][1]*m[1][2] - m[0][2]*m[1][1])/d],
     [(m[1][2]*m[2][0] - m[1][0]*m[2][2])/d, (m[0][0]*m[2][2] - m[0][2]*m[2][0])/d, (m[0][2]*m[1][0] - m[0][0]*m[1][2])/d],
     [(m[1][0]*m[2][1] - m[1][1]*m[2][0])/d, (m[0][1]*m[2][0] - m[0][0]*m[2][1])/d, (m[0][0]*m[1][1] - m[0][1]*m[1][0])/d]]
}

pub fn qmult(a: &Q, b: &Q) -> Q {
    [a[3]*b[0] + b[3]*a[0] + (a[1]*b[2] - a[2]*b[1]),
     a[3]*b[1] + b[3]*a[1] + (a[2]*b[0] - a[0]*b[2]),
     a[3]*b[2] + b[3]*a[2] + (a[0]*b[1] - a[1]*b[0]),
     a[3]*b[3] - (a[0]*b[0] + a[1]*b[1] + a[2]*b[2])]
}
pub fn qconj(q: &Q) -> Q { [-q[0], -q[1], -q[2], q[3]] }
pub fn qnorm(q: &Q) -> Q {
    let n = sqrt(q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]);
    if n < 1e-300 { [0.0, 0.0, 0.0, 1.0] } else { [q[0]/n, q[1]/n, q[2]/n, q[3]/n] }
}
pub fn dcm(q: &Q) -> M3 {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    [[1.0 - 2.0*(y*y + z*z), 2.0*(x*y + z*w), 2.0*(x*z - y*w)],
     [2.0*(y*x - z*w), 1.0 - 2.0*(x*x + z*z), 2.0*(y*z + x*w)],
     [2.0*(z*x + y*w), 2.0*(z*y - x*w), 1.0 - 2.0*(x*x + y*y)]]
}
pub fn fromrotvec(th: &V3) -> Q {
    let a = norm(th);
    if a < 1e-12 { return qnorm(&[th[0]/2.0, th[1]/2.0, th[2]/2.0, 1.0]); }
    let s = sin(a/2.0);
    [s*th[0]/a, s*th[1]/a, s*th[2]/a, cos(a/2.0)]
}
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
pub fn qangle(a: &Q, b: &Q) -> f64 {
    let d = abs(a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3]);
    2.0*acos(if d > 1.0 { 1.0 } else { d })
}
/// q-dot = 1/2 q (x) [w; 0]  (asils.quat.kin)
pub fn kin(q: &Q, w: &V3) -> Q {
    [0.5*(q[3]*w[0] + q[1]*w[2] - q[2]*w[1]),
     0.5*(q[3]*w[1] - q[0]*w[2] + q[2]*w[0]),
     0.5*(q[3]*w[2] + q[0]*w[1] - q[1]*w[0]),
     0.5*(-q[0]*w[0] - q[1]*w[1] - q[2]*w[2])]
}
/// Spherical linear interpolation, shortest arc.
pub fn slerp(a: &Q, b: &Q, s: f64) -> Q {
    let mut d = a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3];
    let mut bb = *b;
    if d < 0.0 { d = -d; bb = [-b[0], -b[1], -b[2], -b[3]]; }
    if d > 0.9995 {
        return qnorm(&[a[0] + s*(bb[0] - a[0]), a[1] + s*(bb[1] - a[1]), a[2] + s*(bb[2] - a[2]), a[3] + s*(bb[3] - a[3])]);
    }
    let th = acos(d);
    let (ka, kb) = (sin((1.0 - s)*th)/sin(th), sin(s*th)/sin(th));
    [ka*a[0] + kb*bb[0], ka*a[1] + kb*bb[1], ka*a[2] + kb*bb[2], ka*a[3] + kb*bb[3]]
}
/// Symmetric 4x4 eigen decomposition by cyclic Jacobi; returns (lambda, V columns).
pub fn jacobi4(k: &[[f64; 4]; 4]) -> ([f64; 4], [[f64; 4]; 4]) {
    const PQ: [[usize; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];
    let mut a = *k;
    let mut v = [[0.0; 4]; 4];
    for i in 0..4 { v[i][i] = 1.0; }
    for _ in 0..12 {
        for pq in PQ.iter() {
            let (p, q) = (pq[0], pq[1]);
            let apq = a[p][q];
            if abs(apq) < 1e-300 { continue; }
            let th = (a[q][q] - a[p][p])/(2.0*apq);
            let t = if th == 0.0 { 1.0 } else { sign(th)/(abs(th) + sqrt(th*th + 1.0)) };
            let c = 1.0/sqrt(t*t + 1.0);
            let sn = t*c;
            for i in 0..4 { let (x, y) = (a[i][p], a[i][q]); a[i][p] = c*x - sn*y; a[i][q] = sn*x + c*y; }
            for i in 0..4 { let (x, y) = (a[p][i], a[q][i]); a[p][i] = c*x - sn*y; a[q][i] = sn*x + c*y; }
            for i in 0..4 { let (x, y) = (v[i][p], v[i][q]); v[i][p] = c*x - sn*y; v[i][q] = sn*x + c*y; }
        }
    }
    ([a[0][0], a[1][1], a[2][2], a[3][3]], v)
}
/// Davenport q-method on unit vector pairs b_k = A r_k (the star tracker's onboard QUEST).
pub fn qmethod(b: &[V3], r: &[V3]) -> Q {
    let mut bm = [[0.0; 3]; 3];
    for l in 0..b.len() { for i in 0..3 { for j in 0..3 { bm[i][j] += b[l][i]*r[l][j]; } } }
    let sg = bm[0][0] + bm[1][1] + bm[2][2];
    let z = [bm[1][2] - bm[2][1], bm[2][0] - bm[0][2], bm[0][1] - bm[1][0]];
    let mut k = [[0.0; 4]; 4];
    for i in 0..3 {
        for j in 0..3 { k[i][j] = bm[i][j] + bm[j][i] - if i == j { sg } else { 0.0 }; }
        k[i][3] = z[i]; k[3][i] = z[i];
    }
    k[3][3] = sg;
    let (lam, v) = jacobi4(&k);
    let mut im = 0;
    for i in 1..4 { if lam[i] > lam[im] { im = i; } }
    qnorm(&[v[0][im], v[1][im], v[2][im], v[3][im]])
}
/// Small random rotation matrix (misalignment).
pub fn small_rot(th: &V3) -> M3 { dcm(&fromrotvec(th)) }
