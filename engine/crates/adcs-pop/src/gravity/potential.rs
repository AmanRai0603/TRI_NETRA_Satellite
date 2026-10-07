//! Port of `grav.potential` (geopotential from normalised coefficients) and its
//! local `normLegendre` (forward column recursion): env's method env_gravity_field, generated from the design into
//! `gen::gravity` (`potential`, `norm_legendre_pot`).
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::field::Field;
use crate::la::V3;

/// Port of the local `normLegendre(sp, cp, N)` of `grav.potential`: fully
/// normalised `Pbar(n,m)` at `sin(phi) = sp`, row-major `(N+1)^2` into `p` (the design's).
pub fn norm_legendre_pot(sp: f64, cp: f64, nn: usize, p: &mut [f64]) {
    assert!(nn <= super::sphharm::NCAP, "normLegendre: degree {nn} > {}", super::sphharm::NCAP);
    let st = nn + 1;
    let q = crate::gen::gravity::norm_legendre_pot(sp, cp, nn as i64);
    p[..st * st].copy_from_slice(&q[..st * st]);
}

/// Port of `grav.potential(r, mu, Re, Cbar, Sbar, nmax)`: `U` [m^2/s^2] with
/// `a = +grad U` (not pole-safe, as the MATLAB).
pub fn potential(r: &V3, f: &Field, nmax: usize) -> f64 {
    potential_coeffs(r, f.gm, f.re, &f.cbar, &f.sbar, f.stride(), nmax)
}

/// [`potential`] on raw row-major normalised coefficient arrays with `stride` (the design's evaluation).
pub fn potential_coeffs(r: &V3, mu: f64, re: f64, cbar: &[f64], sbar: &[f64], stride: usize, nmax: usize) -> f64 {
    let (mut c, mut s) = super::sphharm::packed(nmax, cbar, sbar, stride);
    crate::gen::gravity::potential(*r, mu, re, &mut c, &mut s, (nmax + 1) as i64, nmax as i64)
}
