//! Doubly-averaged (Lidov-Kozai) secular third-body dynamics -- port of
//! `matlab_sils/pop/02_forces/+thirdbody/+secular/*` (`elem2vectors`,
//! `vectors2elem`, `kozaiRates`, `phiQuad`, `propagate`). The rates and the conversions are
//! env's method env_third_body (`gen::thirdbody`); the RK4 of the histories is the code's.

use crate::ephem::constants;
use crate::gen::thirdbody as tb;
use crate::la::V3;

/// `[j, e] = thirdbody.secular.elem2vectors(a, ecc, inc, Om, w)`: classical
/// elements -> (j, e) vectors (`a` is unused, as in MATLAB).
pub fn elem2vectors(_a: f64, ecc: f64, inc: f64, om: f64, w: f64) -> (V3, V3) {
    tb::elem2vectors(ecc, inc, om, w)
}

/// `[ecc, inc, Om, w] = thirdbody.secular.vectors2elem(j, e)` [rad].
pub fn vectors2elem(j: &V3, e: &V3) -> (f64, f64, f64, f64) {
    tb::vectors2elem(*j, *e)
}

/// `[dj, de] = thirdbody.secular.kozaiRates(j, e, nhat, phiQ)`: vectorial
/// doubly-averaged quadrupole rates for one perturber.
pub fn kozai_rates(j: &V3, e: &V3, nhat: &V3, phi_q: f64) -> (V3, V3) {
    tb::kozai_rates(*j, *e, *nhat, phi_q)
}

/// `thirdbody.secular.phiQuad(a_sat, GM_body, r_body, e_body, mu)` [rad/s];
/// `e_body` default 0 and `mu` default `de440.constants().mu_earth` via `None`.
pub fn phi_quad(a_sat: f64, gm_body: f64, r_body: f64, e_body: Option<f64>, mu: Option<f64>) -> f64 {
    tb::phi_quad(a_sat, gm_body, r_body, e_body.unwrap_or(0.0), mu.unwrap_or_else(|| constants().mu_earth))
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
    assert!(p.len() <= 8, "thirdbody.secular.propagate: {} perturbers, at most 8", p.len());
    let mut nh = [[0.0; 3]; 8];
    let mut ph = [0.0; 8];
    for (k, m) in p.iter().enumerate() {
        nh[k] = m.nhat;
        ph[k] = m.phi_q;
    }
    tb::secular_rates(*j, *e, nh, ph, p.len() as i64)
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
        out.kozai.push(tb::kozai_constant(ec, ic));
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
