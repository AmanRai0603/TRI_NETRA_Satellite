//! Ports of `grav.twoBody` and `grav.j2accel` (fast zonal J2..J6).
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::octave_norm;
use super::sphharm::SphHarm;
use crate::la::V3;

/// Port of `grav.twoBody(r, mu)`: `-mu * r / |r|^3` (`|r|` is Octave's `norm`).
pub fn two_body(r: &V3, mu: f64) -> V3 {
    let rn3 = octave_norm(r).powf(3.0);
    [-mu * r[0] / rn3, -mu * r[1] / rn3, -mu * r[2] / rn3]
}

/// Preallocated `grav.j2accel` for a fixed `J = [J2 .. Jn]` (the J3..J6 part runs
/// through a zonal-only [`SphHarm`] built once, exactly as the MATLAB builds its
/// zonal `C` matrix on every call).
#[derive(Clone, Debug)]
pub struct J2Accel {
    j: Vec<f64>,
    ws: Option<SphHarm>,
}

impl J2Accel {
    /// Workspace for `grav.j2accel(., ., ., J)`.
    pub fn new(j: &[f64]) -> J2Accel {
        let nj = j.len();
        let ws = if nj >= 2 && j[1..].iter().any(|&x| x != 0.0) {
            let maxn = nj + 1;
            let st = maxn + 1;
            let mut c = vec![0.0; st * st];
            let s = vec![0.0; st * st];
            c[0] = 1.0;
            for kk in 2..=nj {
                let n = kk + 1;
                c[n * st] = -j[kk - 1] / ((2 * n + 1) as f64).sqrt();
            }
            Some(SphHarm::from_coeffs(maxn, &c, &s, st))
        } else {
            None
        };
        J2Accel { j: j.to_vec(), ws }
    }

    /// `grav.j2accel(r_ecef, mu, Re, J)`: central + zonal acceleration in ECEF [m/s^2].
    pub fn accel(&mut self, r_ecef: &V3, mu: f64, re: f64) -> V3 {
        let r = r_ecef;
        let rn = octave_norm(r);
        let rn3 = rn.powf(3.0);
        let mut a = [-mu * r[0] / rn3, -mu * r[1] / rn3, -mu * r[2] / rn3];
        let nj = self.j.len();
        if nj == 0 {
            return a;
        }
        if nj >= 1 && self.j[0] != 0.0 {
            let j2 = self.j[0];
            let u = r[2] / rn;
            let k = 1.5 * j2 * mu * re.powf(2.0) / rn.powf(4.0);
            let u2 = u.powf(2.0);
            a[0] += k * (5.0 * u2 - 1.0) * r[0] / rn;
            a[1] += k * (5.0 * u2 - 1.0) * r[1] / rn;
            a[2] += k * (5.0 * u2 - 3.0) * r[2] / rn;
        }
        if let Some(ws) = self.ws.as_mut() {
            let maxn = nj + 1;
            let sh = ws.accel(r, mu, re, maxn, 0);
            let tb = two_body(r, mu);
            for i in 0..3 {
                a[i] = a[i] + sh[i] - tb[i];
            }
        }
        a
    }
}

/// One-shot port of `grav.j2accel(r_ecef, mu, Re, J)` (allocates; see [`J2Accel`]).
pub fn j2accel(r_ecef: &V3, mu: f64, re: f64, j: &[f64]) -> V3 {
    J2Accel::new(j).accel(r_ecef, mu, re)
}
