//! Rotor geometry, SR steering, dumping, thruster duty (fsw/pseudocode/07); twin of adcs_alloc.c.
use crate::m::*;
use crate::math::*;
use crate::params::{Params, MAX_COUPLES};

pub type A38 = [[f64; 8]; 3];

pub fn rotor_axes(p: &Params, delta: &[f64; 4]) -> A38 {
    let mut a = [[0.0; 8]; 3];
    for i in 0..p.nr as usize {
        let a0 = p.rot_a0[i];
        if p.rot_gi[i] > 0 {
            let j = p.rot_gi[i] as usize - 1;
            let t0 = cross(&p.gim_axis[j], &a0);
            for k in 0..3 { a[k][i] = cos(delta[j])*a0[k] + sin(delta[j])*t0[k]; }
        } else {
            for k in 0..3 { a[k][i] = a0[k]; }
        }
    }
    a
}

/// Singularity-robust steering; returns (gimbal rates, wheel momentum rates).
pub fn steer_sr(tau: &V3, a: &A38, h: &[f64; 8], p: &Params, wheels: bool) -> ([f64; 4], [f64; 8]) {
    let (ng, nr) = (p.ng as usize, p.nr as usize);
    let mut jg = [[0.0; 4]; 3];
    let mut h0: f64 = 0.0;
    for i in 0..nr {
        if p.rot_gi[i] > 0 {
            let j = p.rot_gi[i] as usize - 1;
            let c = cross(&p.gim_axis[j], &[a[0][i], a[1][i], a[2][i]]);
            for k in 0..3 { jg[k][j] = -h[i]*c[k]; }
            if fabs(h[i]) > h0 { h0 = fabs(h[i]); }
        }
    }
    let mut m = [[0.0; 3]; 3];
    for i in 0..3 { for j in 0..3 { for k in 0..ng { m[i][j] += jg[i][k]*jg[j][k]; } } }
    if h0 < 1e-12 { h0 = 1e-12; }
    let ms = det3(&m)/(h0*h0*h0*h0*h0*h0);
    let n = if wheels { ng + nr } else { ng };
    let mut jj = [[0.0; 12]; 3];
    for i in 0..3 {
        for j in 0..ng { jj[i][j] = jg[i][j]; }
        if wheels { for j in 0..nr { jj[i][ng + j] = -a[i][j]; } }
    }
    let mut w = [0.0; 12];
    for j in 0..n { w[j] = if j < ng { 1.0 } else { 0.01 + 2.0*exp(-10.0*ms) }; }
    let lam = p.cmg_lam0*exp(-p.cmg_mu*ms);
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = if i == j { lam } else { 0.0 };
            for k in 0..n { m[i][j] += jj[i][k]*w[k]*jj[j][k]; }
        }
    }
    let (mi, _) = inv3(&m);
    let x = mat3_vec(&mi, tau);
    let mut u = [0.0; 12];
    for k in 0..n { u[k] = w[k]*(jj[0][k]*x[0] + jj[1][k]*x[1] + jj[2][k]*x[2]); }
    let mut s: f64 = 1.0;
    let mut gdot = [0.0; 4];
    for j in 0..ng { gdot[j] = u[j]; if fabs(u[j])/p.gim_rate_max > s { s = fabs(u[j])/p.gim_rate_max; } }
    for j in 0..ng { gdot[j] /= s; }
    let mut hdot = [0.0; 8];
    for i in 0..nr { hdot[i] = if wheels { u[ng + i]/s } else { 0.0 }; }
    (gdot, hdot)
}

pub fn dump(hdev: &V3, ht: &V3, b: &V3, k: f64, m_max: f64) -> V3 {
    let bs = dot(b, b);
    if bs < 1e-18 { return [0.0; 3]; }
    sat_dipole(&scale3(&cross(&sub3(hdev, ht), b), k/bs), m_max)
}

fn sat_dipole(m: &V3, m_max: f64) -> V3 { crate::ctl::sat_dipole(m, m_max) }

/// Thruster couples: on-time fraction per couple over T and the torque fed forward.
pub fn rcs_duty(req: &V3, p: &Params, t: f64) -> ([f64; MAX_COUPLES], V3) {
    let mut duty = [0.0; MAX_COUPLES];
    let mut tau = [0.0; 3];
    for ax in 0..3 {
        let u = req[ax];
        if u == 0.0 { continue; }
        let k = if u > 0.0 { 2*ax } else { 2*ax + 1 };
        if k >= p.nc as usize { continue; }
        let mut on = fabs(u)/fabs(p.rcs_tau[k][ax]);
        if on > 1.0 { on = 1.0; }
        on *= t;
        if on < p.rcs_mib { continue; }
        on = round(on/p.rcs_res)*p.rcs_res;
        duty[k] = on/t;
        for i in 0..3 { tau[i] += p.rcs_tau[k][i]*duty[k]; }
    }
    (duty, tau)
}
