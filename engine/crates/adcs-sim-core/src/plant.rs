//! Rigid body with momentum-exchange rotors (wheels, fluid rings, CMG/VSCMG),
//! RK4 (asils.plant.{geometry,deriv,step}). Total angular momentum is conserved:
//! rotor torques are internal.
use crate::la::*;
use crate::pm::*;
use crate::{NG, NR};

/// Rotor geometry: spin axes A0 (3 x nr), gimbal axes G (3 x ng), gimbal index gi (0 = fixed).
#[derive(Clone, Copy, Debug, Default)]
pub struct Geometry { pub nr: usize, pub ng: usize, pub a0: [V3; NR], pub t0: [V3; NR], pub g: [V3; NG], pub gi: [usize; NR] }

impl Geometry {
    pub fn new(a0: &[V3], g: &[V3], gi: &[usize]) -> Geometry {
        let mut m = Geometry { nr: a0.len(), ng: g.len(), ..Default::default() };
        for i in 0..m.nr {
            m.a0[i] = a0[i];
            m.gi[i] = gi[i];
            if gi[i] > 0 { m.t0[i] = cross(&g[gi[i] - 1], &a0[i]); }
        }
        for j in 0..m.ng { m.g[j] = g[j]; }
        m
    }
    /// Current spin axes at gimbal angles d.
    pub fn axes(&self, d: &[f64; NG]) -> [V3; NR] {
        let mut a = self.a0;
        for i in 0..self.nr {
            let j = self.gi[i];
            if j > 0 { let dj = d[j - 1]; for k in 0..3 { a[i][k] = cos(dj)*self.a0[i][k] + sin(dj)*self.t0[i][k]; } }
        }
        a
    }
}

/// State: attitude (ECI -> body), body rate, rotor momenta, gimbal angles.
#[derive(Clone, Copy, Debug, Default)]
pub struct State { pub q: Q, pub w: V3, pub h: [f64; NR], pub d: [f64; NG] }

#[derive(Clone, Copy, Debug)]
pub struct Body { pub i: M3, pub iinv: M3, pub m: Geometry }

fn deriv(x: &State, b: &Body, tau_ext: &V3, tau_r: &[f64; NR], gdot: &[f64; NG]) -> State {
    let (w, m) = (x.w, &b.m);
    let mut hr = [0.0; 3];
    let mut hd = [0.0; 3];
    for i in 0..m.nr {
        let (mut a, j) = (m.a0[i], m.gi[i]);
        if j > 0 {
            let dj = x.d[j - 1];
            let (c, s) = (cos(dj), sin(dj));
            for k in 0..3 {
                a[k] = c*m.a0[i][k] + s*m.t0[i][k];
                let ta = -s*m.a0[i][k] + c*m.t0[i][k];
                hd[k] += x.h[i]*gdot[j - 1]*ta;
            }
        }
        for k in 0..3 { hr[k] += a[k]*x.h[i]; hd[k] += a[k]*tau_r[i]; }
    }
    let ht = add(&mv(&b.i, &w), &hr);
    let g = cross(&w, &ht);
    let wd = mv(&b.iinv, &[tau_ext[0] - hd[0] - g[0], tau_ext[1] - hd[1] - g[1], tau_ext[2] - hd[2] - g[2]]);
    let mut xd = State { q: kin(&x.q, &w), w: wd, ..Default::default() };
    for i in 0..m.nr { xd.h[i] = tau_r[i]; }
    for j in 0..m.ng { xd.d[j] = gdot[j]; }
    xd
}

fn axpy(x: &State, h: f64, k: &State, m: &Geometry) -> State {
    let mut y = *x;
    for i in 0..4 { y.q[i] += h*k.q[i]; }
    for i in 0..3 { y.w[i] += h*k.w[i]; }
    for i in 0..m.nr { y.h[i] += h*k.h[i]; }
    for j in 0..m.ng { y.d[j] += h*k.d[j]; }
    y
}

/// One RK4 step with inputs held (external torque, rotor momentum rates, gimbal rates).
pub fn step(x: &State, dt: f64, b: &Body, tau_ext: &V3, tau_r: &[f64; NR], gdot: &[f64; NG]) -> State {
    let m = &b.m;
    let k1 = deriv(x, b, tau_ext, tau_r, gdot);
    let k2 = deriv(&axpy(x, 0.5*dt, &k1, m), b, tau_ext, tau_r, gdot);
    let k3 = deriv(&axpy(x, 0.5*dt, &k2, m), b, tau_ext, tau_r, gdot);
    let k4 = deriv(&axpy(x, dt, &k3, m), b, tau_ext, tau_r, gdot);
    let mut y = *x;
    for i in 0..4 { y.q[i] += dt/6.0*(k1.q[i] + 2.0*k2.q[i] + 2.0*k3.q[i] + k4.q[i]); }
    for i in 0..3 { y.w[i] += dt/6.0*(k1.w[i] + 2.0*k2.w[i] + 2.0*k3.w[i] + k4.w[i]); }
    for i in 0..m.nr { y.h[i] += dt/6.0*(k1.h[i] + 2.0*k2.h[i] + 2.0*k3.h[i] + k4.h[i]); }
    for j in 0..m.ng { y.d[j] += dt/6.0*(k1.d[j] + 2.0*k2.d[j] + 2.0*k3.d[j] + k4.d[j]); }
    y.q = qnorm(&y.q);
    y
}

/// Total angular momentum in the body frame (conservation diagnostic).
pub fn momentum(x: &State, b: &Body) -> V3 {
    let a = b.m.axes(&x.d);
    let mut h = mv(&b.i, &x.w);
    for i in 0..b.m.nr { for k in 0..3 { h[k] += a[i][k]*x.h[i]; } }
    h
}
