//! Port of `grav.sphericalHarmonic` (Cunningham V/W recursion, Montenbruck & Gill
//! 3.2.4-3.2.5, unnormalised, Cartesian and pole-safe): env's method env_gravity_field, generated from the design
//! into `gen::gravity` (`sph_setup`, `sph_accel`), here with its preallocated workspace.
//!
//! The MATLAB denormalises the coefficients on every call; here that is done once
//! per field in [`SphHarm::new`] with the identical arithmetic (`denormFactor`), so
//! the per-call values are bit-identical and the hot path allocates nothing.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::field::Field;
use crate::gen::gravity as g;
use crate::la::V3;

/// The largest degree the design's evaluation holds (`gen::gravity::GRAV_NCAP`).
pub const NCAP: usize = g::GRAV_NCAP as usize;
const CS: usize = g::GRAV_CS as usize;
const VS: usize = g::GRAV_VS as usize;

/// Port of the local `denormFactor(n, m)` of `grav.sphericalHarmonic`:
/// `Pi = sqrt((2-d0m)(2n+1) / prod(n-m+1 .. n+m))` (the design's).
pub fn denorm_factor(n: usize, m: usize) -> f64 {
    g::denorm_factor(n as i64, m as i64)
}

/// Normalised coefficients re-laid row-major with stride `nmax + 1`, as the design's arrays hold them.
pub(crate) fn packed(nmax: usize, cbar: &[f64], sbar: &[f64], stride: usize) -> (Box<[f64; CS]>, Box<[f64; CS]>) {
    assert!(nmax <= NCAP, "sphericalHarmonic: degree {nmax} > {NCAP}, the most the design's evaluation holds");
    let mut c = Box::new([0.0; CS]);
    let mut s = Box::new([0.0; CS]);
    let st = nmax + 1;
    for n in 0..=nmax {
        for m in 0..=n {
            c[n * st + m] = cbar[n * stride + m];
            s[n * st + m] = sbar[n * stride + m];
        }
    }
    (c, s)
}

/// Preallocated workspace for `grav.sphericalHarmonic` on one field: the
/// unnormalised `C, S` (denormalised once) and the `V, W` recursion arrays.
#[derive(Clone, Debug)]
pub struct SphHarm {
    nmax: usize,
    c: Box<[f64; CS]>,
    s: Box<[f64; CS]>,
    f1: Box<[f64; VS]>,
    f2: Box<[f64; VS]>,
    v: Box<[f64; VS]>,
    w: Box<[f64; VS]>,
}

impl SphHarm {
    /// Workspace for `field` up to its full `nmax`, or the design's most ([`NCAP`]) for a larger field
    /// (denormalises every coefficient it holds).
    pub fn new(field: &Field) -> SphHarm {
        Self::from_coeffs(field.nmax.min(NCAP), &field.cbar, &field.sbar, field.stride())
    }

    /// Workspace from normalised coefficient arrays (row-major with `stride`),
    /// holding degrees `0..=nmax` (at most [`NCAP`]).
    pub fn from_coeffs(nmax: usize, cbar: &[f64], sbar: &[f64], stride: usize) -> SphHarm {
        let (mut cb, mut sb) = packed(nmax, cbar, sbar, stride);
        let mut h = SphHarm {
            nmax,
            c: Box::new([0.0; CS]),
            s: Box::new([0.0; CS]),
            f1: Box::new([0.0; VS]),
            f2: Box::new([0.0; VS]),
            v: Box::new([0.0; VS]),
            w: Box::new([0.0; VS]),
        };
        g::sph_setup(nmax as i64, &mut cb, &mut sb, (nmax + 1) as i64, &mut h.c, &mut h.s, &mut h.f1, &mut h.f2);
        h
    }

    /// Largest degree this workspace can evaluate.
    pub fn nmax(&self) -> usize {
        self.nmax
    }

    /// `grav.sphericalHarmonic(r_ecef, mu, Re, Cbar, Sbar, nmax, mmax)`: ECEF
    /// acceleration [m/s^2] (central term included), `mmax = min(mmax, nmax)`.
    /// Panics if `nmax` exceeds the workspace (MATLAB: index out of bounds).
    pub fn accel(&mut self, r_ecef: &V3, mu: f64, re: f64, nmax: usize, mmax: usize) -> V3 {
        assert!(nmax <= self.nmax, "sphericalHarmonic: degree {nmax} > field nmax {}", self.nmax);
        g::sph_accel(*r_ecef, mu, re, self.nmax as i64, nmax as i64, mmax as i64, &mut self.c, &mut self.s, &mut self.f1, &mut self.f2, &mut self.v, &mut self.w)
    }

    /// `grav.j2accel`'s evaluation on this (zonal) workspace: see [`super::J2Accel`].
    pub(crate) fn j2accel(&mut self, r: &V3, mu: f64, re: f64, j: [f64; 5], nj: usize, use_sh: bool) -> V3 {
        g::j2accel(*r, mu, re, j, nj as i64, use_sh, &mut self.c, &mut self.s, &mut self.f1, &mut self.f2, &mut self.v, &mut self.w)
    }
}

/// One-shot `grav.sphericalHarmonic(r_ecef, f.mu, f.Re, f.Cbar, f.Sbar, degree, order)`
/// on a [`Field`] (allocates a workspace; use [`SphHarm`] in loops).
/// `degree` is clamped to the field (as `forces.gravity` does).
pub fn accel_ecef(f: &Field, r_ecef: &V3, degree: usize, order: usize) -> V3 {
    let mut ws = SphHarm::new(f);
    let n = degree.min(f.nmax);
    ws.accel(r_ecef, f.gm, f.re, n, order.min(n))
}
