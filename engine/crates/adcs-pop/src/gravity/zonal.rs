//! Ports of `grav.twoBody` and `grav.j2accel` (fast zonal J2..J6): env's method env_gravity_field, generated from
//! the design into `gen::gravity` (`two_body`, `zonal_c`, `zonal_uses_sh`, `j2accel`).
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::sphharm::SphHarm;
use crate::la::V3;

/// Port of `grav.twoBody(r, mu)`: `-mu * r / |r|^3` (`|r|` is Octave's `norm`).
pub fn two_body(r: &V3, mu: f64) -> V3 {
    crate::gen::gravity::two_body(*r, mu)
}

/// Preallocated `grav.j2accel` for a fixed `J = [J2 .. Jn]` (the J3..J6 part runs
/// through a zonal-only [`SphHarm`] built once, exactly as the MATLAB builds its
/// zonal `C` matrix on every call).
#[derive(Clone, Debug)]
pub struct J2Accel {
    j: [f64; 5],
    nj: usize,
    use_sh: bool,
    ws: SphHarm,
}

impl J2Accel {
    /// Workspace for `grav.j2accel(., ., ., J)` (J2..J6 at most).
    pub fn new(j: &[f64]) -> J2Accel {
        assert!(j.len() <= 5, "j2accel: J2..J6 at most, {} given", j.len());
        let nj = j.len();
        let mut j5 = [0.0; 5];
        j5[..nj].copy_from_slice(j);
        let use_sh = crate::gen::gravity::zonal_uses_sh(j5, nj as i64);
        let maxn = nj + 1;
        let c = crate::gen::gravity::zonal_c(j5, nj as i64);
        let ws = SphHarm::from_coeffs(maxn, &c[..], &vec![0.0; c.len()][..], maxn + 1);
        J2Accel { j: j5, nj, use_sh, ws }
    }

    /// `grav.j2accel(r_ecef, mu, Re, J)`: central + zonal acceleration in ECEF [m/s^2].
    pub fn accel(&mut self, r_ecef: &V3, mu: f64, re: f64) -> V3 {
        self.ws.j2accel(r_ecef, mu, re, self.j, self.nj, self.use_sh)
    }
}

/// One-shot port of `grav.j2accel(r_ecef, mu, Re, J)` (allocates; see [`J2Accel`]).
pub fn j2accel(r_ecef: &V3, mu: f64, re: f64, j: &[f64]) -> V3 {
    J2Accel::new(j).accel(r_ecef, mu, re)
}
