//! Doubly-averaged (Lidov-Kozai) secular third-body dynamics -- port of
//! `matlab_sils/pop/02_forces/+thirdbody/+secular/*` (`elem2vectors`,
//! `vectors2elem`, `kozaiRates`, `phiQuad`, `propagate`).

use crate::ephem::constants;
use crate::la::{cross, dot, norm, V3};

/// `[j, e] = thirdbody.secular.elem2vectors(a, ecc, inc, Om, w)`: classical
/// elements -> (j, e) vectors (`a` is unused, as in MATLAB).
pub fn elem2vectors(_a: f64, ecc: f64, inc: f64, om: f64, w: f64) -> (V3, V3) {
    let (ci, si, co, so) = (inc.cos(), inc.sin(), om.cos(), om.sin());
    let jhat = [so * si, -co * si, ci];
    let node = [co, so, 0.0];
    let ip = cross(&jhat, &node);
    let (cw, sw) = (w.cos(), w.sin());
    let ehat = [cw * node[0] + sw * ip[0], cw * node[1] + sw * ip[1], cw * node[2] + sw * ip[2]];
    let sj = (1.0 - ecc * ecc).sqrt();
    ([sj * jhat[0], sj * jhat[1], sj * jhat[2]], [ecc * ehat[0], ecc * ehat[1], ecc * ehat[2]])
}

/// `[ecc, inc, Om, w] = thirdbody.secular.vectors2elem(j, e)` [rad].
pub fn vectors2elem(j: &V3, e: &V3) -> (f64, f64, f64, f64) {
    let jn = norm(j);
    let ecc = norm(e);
    #[allow(clippy::manual_clamp)] // MATLAB max(min(x,1),-1) semantics (NaN -> 1)
    let inc = (j[2] / jn).min(1.0).max(-1.0).acos();
    let node = cross(&[0.0, 0.0, 1.0], j);
    let nn = norm(&node);
    let (om, nodeu) = if nn < 1e-12 {
        (0.0, [1.0, 0.0, 0.0])
    } else {
        (node[1].atan2(node[0]), [node[0] / nn, node[1] / nn, node[2] / nn])
    };
    let w = if ecc < 1e-12 {
        0.0
    } else {
        let ehat = [e[0] / ecc, e[1] / ecc, e[2] / ecc];
        let ip = cross(&[j[0] / jn, j[1] / jn, j[2] / jn], &nodeu);
        dot(&ip, &ehat).atan2(dot(&nodeu, &ehat))
    };
    (ecc, inc, om, w)
}

/// `[dj, de] = thirdbody.secular.kozaiRates(j, e, nhat, phiQ)`: vectorial
/// doubly-averaged quadrupole rates for one perturber.
pub fn kozai_rates(j: &V3, e: &V3, nhat: &V3, phi_q: f64) -> (V3, V3) {
    let nn = norm(nhat);
    let n = [nhat[0] / nn, nhat[1] / nn, nhat[2] / nn];
    let jn = dot(j, &n);
    let en = dot(e, &n);
    let (jxn, exn, jxe) = (cross(j, &n), cross(e, &n), cross(j, e));
    let mut dj = [0.0; 3];
    let mut de = [0.0; 3];
    for i in 0..3 {
        dj[i] = phi_q * (jn * jxn[i] - 5.0 * en * exn[i]);
        de[i] = phi_q * (jn * exn[i] + 2.0 * jxe[i] - 5.0 * en * jxn[i]);
    }
    (dj, de)
}

/// `thirdbody.secular.phiQuad(a_sat, GM_body, r_body, e_body, mu)` [rad/s];
/// `e_body` default 0 and `mu` default `de440.constants().mu_earth` via `None`.
pub fn phi_quad(a_sat: f64, gm_body: f64, r_body: f64, e_body: Option<f64>, mu: Option<f64>) -> f64 {
    let e_body = e_body.unwrap_or(0.0);
    let mu = mu.unwrap_or_else(|| constants().mu_earth);
    let n = (mu / a_sat.powf(3.0)).sqrt();
    0.75 * (gm_body / r_body.powf(3.0)) / n / (1.0 - e_body * e_body).powf(1.5)
}

/// One perturber of `thirdbody.secular.propagate` (`P(m).nhat`, `P(m).phiQ`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Perturber {
    /// Perturber orbit pole (unit vector in the element frame).
    pub nhat: V3,
    /// Secular quadrupole frequency [rad/s] ([`phi_quad`]).
    pub phi_q: f64,
}

/// Histories returned by `thirdbody.secular.propagate` (`out` struct).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SecularOut {
    /// `out.t` [s].
    pub t: Vec<f64>,
    /// `out.e` eccentricity.
    pub e: Vec<f64>,
    /// `out.inc` [rad].
    pub inc: Vec<f64>,
    /// `out.Om` [rad].
    pub om: Vec<f64>,
    /// `out.w` [rad].
    pub w: Vec<f64>,
    /// `out.kozai = sqrt(1-e^2) cos(i)`.
    pub kozai: Vec<f64>,
    /// `out.a` (constant semi-major axis).
    pub a: f64,
}

fn sum_rates(j: &V3, e: &V3, p: &[Perturber]) -> (V3, V3) {
    let mut dj = [0.0; 3];
    let mut de = [0.0; 3];
    for m in p {
        let (d1, d2) = kozai_rates(j, e, &m.nhat, m.phi_q);
        for i in 0..3 { dj[i] += d1[i]; de[i] += d2[i]; }
    }
    (dj, de)
}

fn axpy(x: &V3, h: f64, y: &V3) -> V3 { [x[0] + h * y[0], x[1] + h * y[1], x[2] + h * y[2]] }

/// `thirdbody.secular.propagate(a, ecc0, inc0, Om0, w0, perturbers, tspan, nsteps)`:
/// RK4 integration of the (j, e) vectors under the vectorial DA quadrupole.
pub fn propagate(a: f64, ecc0: f64, inc0: f64, om0: f64, w0: f64, perturbers: &[Perturber], tspan: f64, nsteps: usize) -> SecularOut {
    let (mut j, mut e) = elem2vectors(a, ecc0, inc0, om0, w0);
    let dt = tspan / nsteps as f64;
    let n = nsteps + 1;
    let mut out = SecularOut { a, ..Default::default() };
    for k in 0..n {
        let (ec, ic, om, w) = vectors2elem(&j, &e);
        out.t.push(k as f64 * dt);
        out.e.push(ec);
        out.inc.push(ic);
        out.om.push(om);
        out.w.push(w);
        out.kozai.push((1.0 - ec * ec).sqrt() * ic.cos());
        let (a1, b1) = sum_rates(&j, &e, perturbers);
        let (a2, b2) = sum_rates(&axpy(&j, 0.5 * dt, &a1), &axpy(&e, 0.5 * dt, &b1), perturbers);
        let (a3, b3) = sum_rates(&axpy(&j, 0.5 * dt, &a2), &axpy(&e, 0.5 * dt, &b2), perturbers);
        let (a4, b4) = sum_rates(&axpy(&j, dt, &a3), &axpy(&e, dt, &b3), perturbers);
        for i in 0..3 {
            j[i] += dt / 6.0 * (a1[i] + 2.0 * a2[i] + 2.0 * a3[i] + a4[i]);
            e[i] += dt / 6.0 * (b1[i] + 2.0 * b2[i] + 2.0 * b3[i] + b4[i]);
        }
    }
    out
}
