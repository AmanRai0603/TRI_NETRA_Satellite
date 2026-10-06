//! MEKF, TRIAD, q-method (fsw/pseudocode/03_estimation.md), twin of adcs_est.c.
use crate::alg::estimation as alg;
use crate::math::*;

type M6 = [[f64; 6]; 6];

#[derive(Clone, Copy, Debug, Default)]
pub struct Mekf { pub q: Q, pub b: V3, pub p: M6, pub arw: f64, pub rrw: f64 }

impl Mekf {
    /// Written from the design: estimation::mekf_init (src/alg).
    pub fn new(q0: &Q, sa: f64, sb: f64, arw: f64, rrw: f64) -> Mekf {
        let (q, b, p) = alg::mekf_init(*q0, sa, sb);
        Mekf { q, b, p, arw, rrw }
    }

    /// Written from the design: estimation::mekf_predict (src/alg).
    pub fn predict(&mut self, wm: &V3, dt: f64) {
        (self.q, self.p) = alg::mekf_predict(self.q, self.b, self.p, self.arw, self.rrw, *wm, dt);
    }

    /// Joseph-form update on a vector measurement; with gate > 0 an innovation whose y' S^-1 y exceeds
    /// the gate is rejected (false) and the state is left as is.
    /// Written from the design: estimation::mekf_vector (src/alg).
    pub fn vector(&mut self, bm: &V3, rr: &V3, sigma: f64, gate: f64) -> bool {
        let took;
        (self.q, self.b, self.p, took) = alg::mekf_vector(self.q, self.b, self.p, *bm, *rr, sigma, gate);
        took
    }

    /// Written from the design: estimation::mekf_quat (src/alg).
    pub fn quat(&mut self, qm: &Q, sc: f64, sr: f64, bs: &V3, gate: f64) -> bool {
        let took;
        (self.q, self.b, self.p, took) = alg::mekf_quat(self.q, self.b, self.p, *qm, sc, sr, *bs, gate);
        took
    }
}

/// None when either pair is (nearly) parallel.
/// Written from the design: estimation::triad (src/alg).
pub fn triad(b1: &V3, b2: &V3, r1: &V3, r2: &V3) -> Option<Q> {
    let (q, ok) = alg::triad(*b1, *b2, *r1, *r2);
    if ok { Some(q) } else { None }
}

/// q-method on vector pairs (weights default to 1); returns (q, loss = sum(w) - lambda_max).
/// At most 16 pairs, as the design takes them.
/// Written from the design: estimation::quest (src/alg).
pub fn quest(b: &[V3], r: &[V3], w: Option<&[f64]>) -> (Q, f64) {
    let n = b.len();
    assert!(n <= 16, "quest takes at most 16 vector pairs");
    let (mut bv, mut rv, mut wv) = ([[0.0; 3]; 16], [[0.0; 3]; 16], [0.0; 16]);
    for l in 0..n {
        bv[l] = b[l];
        rv[l] = r[l];
        wv[l] = match w { Some(w) => w[l], None => 1.0 };
    }
    alg::quest(bv, rv, wv, n as i64)
}

/// Star-tracker quaternion propagated over its latency with the body rate.
/// Written from the design: estimation::latency (src/alg).
pub fn latency(q_st: &Q, w: &V3, lat: f64) -> Q { alg::latency(*q_st, *w, lat) }
