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

/// State: attitude (ECI -> body), body rate, rotor momenta, gimbal angles, and the flexible
/// mode's coordinate and rate (zero on a rigid body).
#[derive(Clone, Copy, Debug, Default)]
pub struct State { pub q: Q, pub w: V3, pub h: [f64; NR], pub d: [f64; NG], pub eta: f64, pub etad: f64 }

/// One flexible mode in hybrid coordinates (Hughes, Spacecraft Attitude Dynamics, ch. 12):
///   J wdot + delta etadd = torques - w x (J w + h_rotors + delta etad)
///   etadd + 2 zeta Omega etad + Omega^2 eta + delta . wdot = 0
/// delta [sqrt(kg) m] couples the mode to the body; |delta|^2 is the inertia that takes part.
#[derive(Clone, Copy, Debug, Default)]
pub struct Flex { pub on: bool, pub delta: V3, pub omega: f64, pub zeta: f64 }

#[derive(Clone, Copy, Debug)]
pub struct Body { pub i: M3, pub iinv: M3, pub m: Geometry,
    /// the flexible mode, and the inverse of the reduced inertia J - delta delta' the body
    /// equation divides by when it is on (iinv when off)
    pub flex: Flex, pub minv: M3 }

impl Body {
    /// A rigid body.
    pub fn rigid(i: M3, m: Geometry) -> Body { let iinv = inv(&i); Body { i, iinv, m, flex: Flex::default(), minv: iinv } }
    /// A body with one flexible mode.
    pub fn flexible(i: M3, m: Geometry, flex: Flex) -> Body {
        let mut r = i;
        for a in 0..3 { for c in 0..3 { r[a][c] -= flex.delta[a]*flex.delta[c]; } }
        Body { i, iinv: inv(&i), m, flex, minv: inv(&r) }
    }
}

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
    let f = &b.flex;
    // the mode's own momentum delta etad turns with the body too
    let ht = if f.on { add(&add(&mv(&b.i, &w), &hr), &scale(&f.delta, x.etad)) } else { add(&mv(&b.i, &w), &hr) };
    let g = cross(&w, &ht);
    let mut xd = if f.on {
        // the mode's restoring and damping forces act on the body through delta
        let pull = 2.0*f.zeta*f.omega*x.etad + f.omega*f.omega*x.eta;
        let rhs = [tau_ext[0] - hd[0] - g[0] + f.delta[0]*pull, tau_ext[1] - hd[1] - g[1] + f.delta[1]*pull, tau_ext[2] - hd[2] - g[2] + f.delta[2]*pull];
        let wd = mv(&b.minv, &rhs);
        State { q: kin(&x.q, &w), w: wd, eta: x.etad, etad: -pull - dot(&f.delta, &wd), ..Default::default() }
    } else {
        let wd = mv(&b.iinv, &[tau_ext[0] - hd[0] - g[0], tau_ext[1] - hd[1] - g[1], tau_ext[2] - hd[2] - g[2]]);
        State { q: kin(&x.q, &w), w: wd, ..Default::default() }
    };
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
    y.eta += h*k.eta; y.etad += h*k.etad;
    y
}

/// One RK4 step with inputs held (external torque, rotor momentum rates, gimbal rates). With a
/// flexible mode the step is cut so that Omega h stays at most 0.5 (RK4 is stable to 2.8, accurate
/// well below it); a rigid body takes the one step, as before.
pub fn step(x: &State, dt: f64, b: &Body, tau_ext: &V3, tau_r: &[f64; NR], gdot: &[f64; NG]) -> State {
    if b.flex.on {
        let n = (-floor(-b.flex.omega*dt/0.5) as usize).max(1);
        let mut y = *x;
        for _ in 0..n { y = step_rk4(&y, dt/n as f64, b, tau_ext, tau_r, gdot); }
        return y;
    }
    step_rk4(x, dt, b, tau_ext, tau_r, gdot)
}

fn step_rk4(x: &State, dt: f64, b: &Body, tau_ext: &V3, tau_r: &[f64; NR], gdot: &[f64; NG]) -> State {
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
    y.eta += dt/6.0*(k1.eta + 2.0*k2.eta + 2.0*k3.eta + k4.eta);
    y.etad += dt/6.0*(k1.etad + 2.0*k2.etad + 2.0*k3.etad + k4.etad);
    y.q = qnorm(&y.q);
    y
}

/// Total angular momentum in the body frame (conservation diagnostic).
pub fn momentum(x: &State, b: &Body) -> V3 {
    let a = b.m.axes(&x.d);
    let mut h = mv(&b.i, &x.w);
    for i in 0..b.m.nr { for k in 0..3 { h[k] += a[i][k]*x.h[i]; } }
    if b.flex.on { for k in 0..3 { h[k] += b.flex.delta[k]*x.etad; } }
    h
}
