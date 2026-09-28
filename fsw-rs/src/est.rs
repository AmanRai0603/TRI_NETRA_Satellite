//! MEKF, TRIAD, q-method (fsw/pseudocode/03_estimation.md), twin of adcs_est.c.
use crate::math::*;

type M6 = [[f64; 6]; 6];

#[derive(Clone, Copy, Debug, Default)]
pub struct Mekf { pub q: Q, pub b: V3, pub p: M6, pub arw: f64, pub rrw: f64 }

impl Mekf {
    pub fn new(q0: &Q, sa: f64, sb: f64, arw: f64, rrw: f64) -> Mekf {
        let mut p = [[0.0; 6]; 6];
        for i in 0..3 { p[i][i] = sa*sa; p[i + 3][i + 3] = sb*sb; }
        Mekf { q: *q0, b: [0.0; 3], p, arw, rrw }
    }

    pub fn predict(&mut self, wm: &V3, dt: f64) {
        let sv2 = self.arw*self.arw;
        let su2 = self.rrw*self.rrw;
        let w = sub3(wm, &self.b);
        let dq = fromrotvec(&scale3(&w, dt));
        self.q = qnorm(&qmult(&self.q, &dq));
        let ww = skew(&w);
        let w2 = mat3_mul(&ww, &ww);
        let mut phi = [[0.0; 6]; 6];
        for i in 0..3 {
            for j in 0..3 { phi[i][j] = (if i == j { 1.0 } else { 0.0 }) - ww[i][j]*dt + 0.5*w2[i][j]*dt*dt; }
            phi[i][i + 3] = -dt;
            phi[i + 3][i + 3] = 1.0;
        }
        let mut t = [[0.0; 6]; 6];
        for i in 0..6 { for j in 0..6 { for l in 0..6 { t[i][j] += phi[i][l]*self.p[l][j]; } } }
        let mut pn = [[0.0; 6]; 6];
        for i in 0..6 { for j in 0..6 { for l in 0..6 { pn[i][j] += t[i][l]*phi[j][l]; } } }
        for i in 0..3 {
            pn[i][i] += sv2*dt + su2*dt*dt*dt/3.0;
            pn[i][i + 3] += -su2*dt*dt/2.0;
            pn[i + 3][i] += -su2*dt*dt/2.0;
            pn[i + 3][i + 3] += su2*dt;
        }
        self.p = pn;
    }

    /// Joseph-form update with a 3-row measurement.
    fn update3(&mut self, h: &[[f64; 6]; 3], r: &M3, y: &V3) {
        let mut pht = [[0.0; 3]; 6];
        for i in 0..6 { for j in 0..3 { for l in 0..6 { pht[i][j] += self.p[i][l]*h[j][l]; } } }
        let mut s = *r;
        for i in 0..3 { for j in 0..3 { for l in 0..6 { s[i][j] += h[i][l]*pht[l][j]; } } }
        let (si, _) = inv3(&s);
        let mut g = [[0.0; 3]; 6];
        for i in 0..6 { for j in 0..3 { for l in 0..3 { g[i][j] += pht[i][l]*si[l][j]; } } }
        let mut dx = [0.0; 6];
        for i in 0..6 { for l in 0..3 { dx[i] += g[i][l]*y[l]; } }
        let dq = [0.5*dx[0], 0.5*dx[1], 0.5*dx[2], 1.0];
        self.q = qnorm(&qmult(&self.q, &dq));
        for i in 0..3 { self.b[i] += dx[i + 3]; }
        let mut ikh = [[0.0; 6]; 6];
        for i in 0..6 {
            for j in 0..6 {
                ikh[i][j] = if i == j { 1.0 } else { 0.0 };
                for l in 0..3 { ikh[i][j] -= g[i][l]*h[l][j]; }
            }
        }
        let mut t = [[0.0; 6]; 6];
        for i in 0..6 { for j in 0..6 { for l in 0..6 { t[i][j] += ikh[i][l]*self.p[l][j]; } } }
        let mut pn = [[0.0; 6]; 6];
        for i in 0..6 { for j in 0..6 { for l in 0..6 { pn[i][j] += t[i][l]*ikh[j][l]; } } }
        for i in 0..6 { for j in 0..6 { for a in 0..3 { for c in 0..3 { pn[i][j] += g[i][a]*r[a][c]*g[j][c]; } } } }
        self.p = pn;
    }

    pub fn vector(&mut self, bm: &V3, rr: &V3, sigma: f64) {
        let b = unit(bm);
        let r = unit(rr);
        let bh = mat3_vec(&dcm(&self.q), &r);
        let sk = skew(&bh);
        let mut h = [[0.0; 6]; 3];
        for i in 0..3 { for j in 0..3 { h[i][j] = sk[i][j]; } }
        let mut rm = [[0.0; 3]; 3];
        for i in 0..3 { rm[i][i] = sigma*sigma; }
        self.update3(&h, &rm, &sub3(&b, &bh));
    }

    pub fn quat(&mut self, qm: &Q, sc: f64, sr: f64, bs: &V3) {
        let dq = qerr(&self.q, qm);
        let y = [2.0*dq[0], 2.0*dq[1], 2.0*dq[2]];
        let mut h = [[0.0; 6]; 3];
        let mut r = [[0.0; 3]; 3];
        for i in 0..3 {
            h[i][i] = 1.0;
            for j in 0..3 { r[i][j] = (if i == j { sc*sc } else { 0.0 }) + (sr*sr - sc*sc)*bs[i]*bs[j]; }
        }
        self.update3(&h, &r, &y);
    }
}

pub fn triad(b1: &V3, b2: &V3, r1: &V3, r2: &V3) -> Q {
    let tb0 = unit(b1);
    let tb1 = unit(&cross(b1, b2));
    let tb2 = cross(&tb0, &tb1);
    let tr0 = unit(r1);
    let tr1 = unit(&cross(r1, r2));
    let tr2 = cross(&tr0, &tr1);
    let mut a = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { a[i][j] = tb0[i]*tr0[j] + tb1[i]*tr1[j] + tb2[i]*tr2[j]; } }
    fromdcm(&a)
}

/// q-method on vector pairs (weights default to 1); returns (q, loss = sum(w) - lambda_max).
pub fn quest(b: &[V3], r: &[V3], w: Option<&[f64]>) -> (Q, f64) {
    let mut bm = [[0.0; 3]; 3];
    let mut ws = 0.0;
    for l in 0..b.len() {
        let wl = match w { Some(w) => w[l], None => 1.0 };
        ws += wl;
        for i in 0..3 { for j in 0..3 { bm[i][j] += wl*b[l][i]*r[l][j]; } }
    }
    let sg = bm[0][0] + bm[1][1] + bm[2][2];
    let z = [bm[1][2] - bm[2][1], bm[2][0] - bm[0][2], bm[0][1] - bm[1][0]];
    let mut k = [[0.0; 4]; 4];
    for i in 0..3 {
        for j in 0..3 { k[i][j] = bm[i][j] + bm[j][i] - if i == j { sg } else { 0.0 }; }
        k[i][3] = z[i];
        k[3][i] = z[i];
    }
    k[3][3] = sg;
    let (lam, v) = jacobi_eig4(&k);
    let mut im = 0;
    for i in 1..4 { if lam[i] > lam[im] { im = i; } }
    (qnorm(&[v[0][im], v[1][im], v[2][im], v[3][im]]), ws - lam[im])
}

/// Star-tracker quaternion propagated over its latency with the body rate.
pub fn latency(q_st: &Q, w: &V3, lat: f64) -> Q {
    qnorm(&qmult(q_st, &fromrotvec(&scale3(w, lat))))
}
