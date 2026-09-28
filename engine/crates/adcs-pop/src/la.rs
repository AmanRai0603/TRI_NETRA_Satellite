//! Shared fixed-size linear algebra. V3 = [x, y, z]; M3 row-major (m[row][col]).
pub type V3 = [f64; 3];
pub type M3 = [[f64; 3]; 3];
#[inline] pub fn dot(a: &V3, b: &V3) -> f64 { a[0]*b[0] + a[1]*b[1] + a[2]*b[2] }
#[inline] pub fn cross(a: &V3, b: &V3) -> V3 { [a[1]*b[2] - a[2]*b[1], a[2]*b[0] - a[0]*b[2], a[0]*b[1] - a[1]*b[0]] }
#[inline] pub fn norm(a: &V3) -> f64 { dot(a, a).sqrt() }
#[inline] pub fn add(a: &V3, b: &V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
#[inline] pub fn sub(a: &V3, b: &V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
#[inline] pub fn scale(a: &V3, s: f64) -> V3 { [a[0]*s, a[1]*s, a[2]*s] }
pub fn unit(a: &V3) -> V3 { let n = norm(a); scale(a, 1.0/n) }
pub fn mv(m: &M3, v: &V3) -> V3 { [dot(&m[0], v), dot(&m[1], v), dot(&m[2], v)] }
pub fn mtv(m: &M3, v: &V3) -> V3 { let t = transpose(m); mv(&t, v) }
pub fn mm(a: &M3, b: &M3) -> M3 {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { r[i][j] = a[i][0]*b[0][j] + a[i][1]*b[1][j] + a[i][2]*b[2][j]; } }
    r
}
pub fn transpose(a: &M3) -> M3 { [[a[0][0], a[1][0], a[2][0]], [a[0][1], a[1][1], a[2][1]], [a[0][2], a[1][2], a[2][2]]] }
pub const I3: M3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
/// Elementary rotations (passive, as MATLAB R1/R2/R3 in POP): R3(a) = [c s 0; -s c 0; 0 0 1].
pub fn r1(a: f64) -> M3 { let (s, c) = a.sin_cos(); [[1.0, 0.0, 0.0], [0.0, c, s], [0.0, -s, c]] }
pub fn r2(a: f64) -> M3 { let (s, c) = a.sin_cos(); [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]] }
pub fn r3(a: f64) -> M3 { let (s, c) = a.sin_cos(); [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]] }
