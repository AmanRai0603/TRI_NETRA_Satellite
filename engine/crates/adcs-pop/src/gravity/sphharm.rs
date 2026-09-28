//! Port of `grav.sphericalHarmonic` (Cunningham V/W recursion, Montenbruck & Gill
//! 3.2.4-3.2.5, unnormalised, Cartesian and pole-safe) with a preallocated workspace.
//!
//! The MATLAB denormalises the coefficients on every call; here that is done once
//! per field in [`SphHarm::new`] with the identical arithmetic (`denormFactor`), so
//! the per-call values are bit-identical and the hot path allocates nothing.
//!
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::field::Field;
use crate::la::V3;

/// Port of the local `denormFactor(n, m)` of `grav.sphericalHarmonic`:
/// `Pi = sqrt((2-d0m)(2n+1) / prod(n-m+1 .. n+m))`.
pub fn denorm_factor(n: usize, m: usize) -> f64 {
    let d = if m == 0 { 1.0 } else { 2.0 };
    let mut p = 1.0f64;
    for k in (n - m + 1)..=(n + m) {
        p *= k as f64;
    }
    (d * (2 * n + 1) as f64 / p).sqrt()
}

/// Preallocated workspace for `grav.sphericalHarmonic` on one field: the
/// unnormalised `C, S` (denormalised once) and the `V, W` recursion arrays.
#[derive(Clone, Debug)]
pub struct SphHarm {
    nmax: usize,
    cs: usize,  // stride of c/s = nmax + 1
    c: Vec<f64>,
    s: Vec<f64>,
    vs: usize,  // stride of v/w/f1/f2 = nmax + 3
    v: Vec<f64>,
    w: Vec<f64>,
    // recursion factors (2n-1)/(n-m) and (n+m-1)/(n-m), same division as the
    // MATLAB performs per call, tabulated once (bit-identical, no divides per call)
    f1: Vec<f64>,
    f2: Vec<f64>,
}

impl SphHarm {
    /// Workspace for `field` up to its full `nmax` (denormalises every coefficient).
    pub fn new(field: &Field) -> SphHarm {
        Self::from_coeffs(field.nmax, &field.cbar, &field.sbar, field.stride())
    }

    /// Workspace from normalised coefficient arrays (row-major with `stride`),
    /// holding degrees `0..=nmax`.
    pub fn from_coeffs(nmax: usize, cbar: &[f64], sbar: &[f64], stride: usize) -> SphHarm {
        let cs = nmax + 1;
        let mut c = vec![0.0; cs * cs];
        let mut s = vec![0.0; cs * cs];
        for n in 0..=nmax {
            for m in 0..=n {
                let pi = denorm_factor(n, m);
                c[n * cs + m] = cbar[n * stride + m] * pi;
                if m > 0 {
                    s[n * cs + m] = sbar[n * stride + m] * pi;
                }
            }
        }
        let vs = nmax + 3;
        let mut f1 = vec![0.0; vs * vs];
        let mut f2 = vec![0.0; vs * vs];
        for m in 1..=nmax + 1 {
            for n in m + 2..=nmax + 1 {
                let (nf, mf) = (n as f64, m as f64);
                f1[n * vs + m] = (2.0 * nf - 1.0) / (nf - mf);
                f2[n * vs + m] = (nf + mf - 1.0) / (nf - mf);
            }
        }
        SphHarm { nmax, cs, c, s, vs, v: vec![0.0; vs * vs], w: vec![0.0; vs * vs], f1, f2 }
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
        let mmax = mmax.min(nmax);
        let (x, y, z) = (r_ecef[0], r_ecef[1], r_ecef[2]);
        let r2 = x * x + y * y + z * z;
        let rn = r2.sqrt();
        let xf = re * x / r2;
        let yf = re * y / r2;
        let zf = re * z / r2;
        let rr2 = re * re / r2;
        let vs = self.vs;
        let v = &mut self.v[..];
        let w = &mut self.w[..];

        // The MATLAB fills V/W column by column (m outer, n inner). Every entry
        // depends only on the two entries above it in its own column (or, for the
        // sectorial/sub-diagonal ones, on the previous row's diagonal), so filling
        // row by row performs the identical operations on identical operands (same
        // bits) while letting the general-term loop over m run without a serial
        // dependency chain.
        v[0] = re / rn;
        w[0] = 0.0;
        let mcol = mmax + 1; // highest order the MATLAB recursion fills
        for n in 1..=nmax + 1 {
            let nf = n as f64;
            let row = n * vs;
            let (head, tail) = v.split_at_mut(row);
            let (whead, wtail) = w.split_at_mut(row);
            let vr = &mut tail[..vs];
            let wr = &mut wtail[..vs];
            let vr1 = &head[row - vs..row];
            let wr1 = &whead[row - vs..row];
            // m = 0 (zonal column; W(:,1) stays 0)
            if n == 1 {
                vr[0] = zf * 1.0 * vr1[0];
            } else {
                let vr2 = &head[row - 2 * vs..row - vs];
                vr[0] = ((2.0 * nf - 1.0) * zf * vr1[0] - (nf - 1.0) * rr2 * vr2[0]) / nf;
            }
            wr[0] = 0.0;
            // general terms 1 <= m <= min(n-2, mmax+1)
            if n >= 3 {
                let mg = (n - 2).min(mcol);
                let vr2 = &head[row - 2 * vs..row - vs];
                let wr2 = &whead[row - 2 * vs..row - vs];
                let f1 = &self.f1[row..row + vs];
                let f2 = &self.f2[row..row + vs];
                for m in 1..=mg {
                    vr[m] = f1[m] * zf * vr1[m] - f2[m] * rr2 * vr2[m];
                    wr[m] = f1[m] * zf * wr1[m] - f2[m] * rr2 * wr2[m];
                }
            }
            // first sub-diagonal m = n-1 (from the previous row's sectorial term)
            if n >= 2 && n - 1 <= mcol {
                let m = n - 1;
                let mf = m as f64;
                vr[m] = (2.0 * mf + 1.0) * zf * vr1[m];
                wr[m] = (2.0 * mf + 1.0) * zf * wr1[m];
            }
            // sectorial m = n
            if n <= mcol {
                let (vp, wp) = (vr1[n - 1], wr1[n - 1]);
                vr[n] = (2.0 * nf - 1.0) * (xf * vp - yf * wp);
                wr[n] = (2.0 * nf - 1.0) * (xf * wp + yf * vp);
            }
        }

        // accumulate (Montenbruck & Gill Eq. 3.33)
        let (mut ax, mut ay, mut az) = (0.0f64, 0.0f64, 0.0f64);
        let cs = self.cs;
        for n in 0..=nmax {
            let mt = n.min(mmax);
            let row = (n + 1) * vs;
            let vr = &v[row..row + mt + 2];
            let wr = &w[row..row + mt + 2];
            let cr = &self.c[n * cs..n * cs + mt + 1];
            let sr = &self.s[n * cs..n * cs + mt + 1];
            // m = 0
            let (c0, s0) = (cr[0], sr[0]);
            if !(c0 == 0.0 && s0 == 0.0) {
                ax -= c0 * vr[1];
                ay -= c0 * wr[1];
                az += (n + 1) as f64 * (-c0 * vr[0] - s0 * wr[0]);
            }
            for m in 1..=mt {
                let cnm = cr[m];
                let snm = sr[m];
                if cnm == 0.0 && snm == 0.0 {
                    continue;
                }
                let fac = ((n - m + 1) * (n - m + 2)) as f64;
                let (vp1, wp1, vm1, wm1) = (vr[m + 1], wr[m + 1], vr[m - 1], wr[m - 1]);
                ax += 0.5 * (-cnm * vp1 - snm * wp1 + fac * (cnm * vm1 + snm * wm1));
                ay += 0.5 * (-cnm * wp1 + snm * vp1 + fac * (-cnm * wm1 + snm * vm1));
                az += (n - m + 1) as f64 * (-cnm * vr[m] - snm * wr[m]);
            }
        }
        let scale = mu / (re * re);
        [scale * ax, scale * ay, scale * az]
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
