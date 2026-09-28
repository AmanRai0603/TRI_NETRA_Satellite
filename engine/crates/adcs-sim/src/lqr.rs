//! Continuous LQR for the single-axis chains of asils.config (asils.fsw.lqr_gain),
//! solved by Kleinman's Newton iteration on the Riccati equation
//! A'X + XA - X B R^-1 B'X + Q = 0, started from a stabilising pole placement.

fn solve(a: &mut [[f64; 9]; 9], b: &mut [f64; 9]) {
    for c in 0..9 {
        let mut p = c;
        for r in c + 1..9 { if a[r][c].abs() > a[p][c].abs() { p = r; } }
        a.swap(c, p); b.swap(c, p);
        for r in c + 1..9 {
            let f = a[r][c]/a[c][c];
            for k in c..9 { a[r][k] -= f*a[c][k]; }
            b[r] -= f*b[c];
        }
    }
    for c in (0..9).rev() {
        let mut s = b[c];
        for k in c + 1..9 { s -= a[c][k]*b[k]; }
        b[c] = s/a[c][c];
    }
}

/// Ac' X + X Ac = -Qk for symmetric X (3x3).
fn lyap(ac: &[[f64; 3]; 3], qk: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut m = [[0.0; 9]; 9];
    let mut rhs = [0.0; 9];
    for i in 0..3 { for j in 0..3 {
        let row = 3*i + j;
        rhs[row] = -qk[i][j];
        for k in 0..3 {
            m[row][3*k + j] += ac[k][i];   // (Ac' X)_ij = sum_k Ac_ki X_kj
            m[row][3*i + k] += ac[k][j];   // (X Ac)_ij  = sum_k X_ik Ac_kj
        }
    } }
    solve(&mut m, &mut rhs);
    let mut x = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { x[i][j] = 0.5*(rhs[3*i + j] + rhs[3*j + i]); } }
    x
}

/// Gain K (1x3) for A = [0 1 0; 0 0 1; 0 0 0], B = [0; 0; b], weights Q (diag) and scalar R; p0 seeds the start.
pub fn chain3(b: f64, q: [f64; 3], r: f64, p0: f64) -> [f64; 3] {
    let a = [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 0.0]];
    let mut k = [p0*p0*p0/b, 3.0*p0*p0/b, 3.0*p0/b];
    for _ in 0..200 {
        let mut ac = a;
        for j in 0..3 { ac[2][j] -= b*k[j]; }
        let mut qk = [[0.0; 3]; 3];
        for i in 0..3 { qk[i][i] = q[i]; for j in 0..3 { qk[i][j] += k[i]*r*k[j]; } }
        let x = lyap(&ac, &qk);
        let kn = [b*x[2][0]/r, b*x[2][1]/r, b*x[2][2]/r];
        let d = (0..3).map(|i| ((kn[i] - k[i])/kn[i].abs().max(1e-300)).abs()).fold(0.0, f64::max);
        k = kn;
        if d < 1e-13 { break; }
    }
    k
}

#[cfg(test)]
mod t {
    #[test]
    fn double_integrator_limit() {
        // with a negligible integral weight the chain is a PD with the classic sqrt(2) damping
        let (b, q1, q2, r): (f64, f64, f64, f64) = (1.0, 1e-12, 1.0, 1.0);
        let k = super::chain3(b, [q1, q2, 0.0], r, 0.5);
        assert!((k[1] - 1.0).abs() < 1e-3 && (k[2] - 2f64.sqrt()).abs() < 1e-3, "{k:?}");
    }
}
