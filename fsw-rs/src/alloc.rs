//! Rotor geometry, SR steering, dumping, thruster duty (fsw/pseudocode/07); twin of adcs_alloc.c.
use crate::alg::allocation as alg;
use crate::math::*;
use crate::params::{Params, MAX_COUPLES};

pub type A38 = [[f64; 8]; 3];

/// Written from the design: allocation::rotor_axes (src/alg).
pub fn rotor_axes(p: &Params, delta: &[f64; 4]) -> A38 { alg::rotor_axes(p.nr as i64, p.rot_a0, ints(&p.rot_gi), p.gim_axis, *delta) }

/// Singularity-robust steering; returns (gimbal rates, wheel momentum rates).
/// Written from the design: allocation::steer_sr (src/alg).
pub fn steer_sr(tau: &V3, a: &A38, h: &[f64; 8], p: &Params, wheels: bool) -> ([f64; 4], [f64; 8]) {
    alg::steer_sr(*tau, *a, *h, p.nr as i64, p.ng as i64, ints(&p.rot_gi), p.gim_axis, p.gim_rate_max, p.cmg_lam0, p.cmg_mu, wheels)
}

/// Written from the design: allocation::dump (src/alg).
pub fn dump(hdev: &V3, ht: &V3, b: &V3, k: f64, m_max: f64) -> V3 { alg::dump(*hdev, *ht, *b, k, m_max) }

/// Thruster couples: on-time fraction per couple over T and the torque fed forward.
/// Written from the design: allocation::rcs_duty (src/alg).
pub fn rcs_duty(req: &V3, p: &Params, t: f64) -> ([f64; MAX_COUPLES], V3) {
    alg::rcs_duty(*req, p.nc as i64, p.rcs_tau, p.rcs_mib, p.rcs_res, t)
}
