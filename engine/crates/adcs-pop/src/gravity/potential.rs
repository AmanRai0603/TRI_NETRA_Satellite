//! Port of `grav.potential` (geopotential from normalised coefficients) and its
//! local `normLegendre` (forward column recursion).
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::field::Field;
use super::octave_norm;
use crate::la::V3;

/// Port of the local `normLegendre(sp, cp, N)` of `grav.potential`: fully
/// normalised `Pbar(n,m)` at `sin(phi) = sp`, row-major `(N+1)^2` into `p`.
pub fn norm_legendre_pot(sp: f64, cp: f64, nn: usize, p: &mut [f64]) {
    let st = nn + 1;
    for x in p[..st * st].iter_mut() {
        *x = 0.0;
    }
    p[0] = 1.0;
    if nn >= 1 {
        p[st] = 3f64.sqrt() * sp;
        p[st + 1] = 3f64.sqrt() * cp;
    }
    for n in 2..=nn {
        let nf = n as f64;
        p[n * st + n] = ((2.0 * nf + 1.0) / (2.0 * nf)).sqrt() * cp * p[(n - 1) * st + n - 1];
        for m in 0..n {
            let mf = m as f64;
            let a = ((2.0 * nf + 1.0) * (2.0 * nf - 1.0) / ((nf - mf) * (nf + mf))).sqrt();
            let b = ((2.0 * nf + 1.0) * (nf + mf - 1.0) * (nf - mf - 1.0) / ((2.0 * nf - 3.0) * (nf - mf) * (nf + mf))).sqrt();
            p[n * st + m] = if m == n - 1 {
                a * sp * p[(n - 1) * st + m]
            } else {
                a * sp * p[(n - 1) * st + m] - b * p[(n - 2) * st + m]
            };
        }
    }
}

/// Port of `grav.potential(r, mu, Re, Cbar, Sbar, nmax)`: `U` [m^2/s^2] with
/// `a = +grad U` (not pole-safe, as the MATLAB).
pub fn potential(r: &V3, f: &Field, nmax: usize) -> f64 {
    potential_coeffs(r, f.gm, f.re, &f.cbar, &f.sbar, f.stride(), nmax)
}

/// [`potential`] on raw row-major normalised coefficient arrays with `stride`.
pub fn potential_coeffs(r: &V3, mu: f64, re: f64, cbar: &[f64], sbar: &[f64], stride: usize, nmax: usize) -> f64 {
    let (x, y, z) = (r[0], r[1], r[2]);
    let rn = octave_norm(r);
    let phi = (z / rn).asin();
    let lam = y.atan2(x);
    let (sp, cp) = (phi.sin(), phi.cos());
    let st = nmax + 1;
    let mut p = vec![0.0; st * st];
    norm_legendre_pot(sp, cp, nmax, &mut p);
    let cml: Vec<f64> = (0..=nmax).map(|m| (m as f64 * lam).cos()).collect();
    let sml: Vec<f64> = (0..=nmax).map(|m| (m as f64 * lam).sin()).collect();
    let mut u = 0.0;
    let rr = re / rn;
    for n in 0..=nmax {
        let rn_pow = rr.powf(n as f64);
        let mut inner = 0.0;
        for m in 0..=n {
            inner += p[n * st + m] * (cbar[n * stride + m] * cml[m] + sbar[n * stride + m] * sml[m]);
        }
        u += rn_pow * inner;
    }
    mu / rn * u
}
