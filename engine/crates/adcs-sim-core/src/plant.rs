//! Rigid body with momentum-exchange rotors (wheels, fluid rings, CMG/VSCMG) and one flexible mode: dyn's methods,
//! generated from the design (tools/engine_build.py): the rotors' geometry and axes (`gen::rotors`,
//! dyn_rotor_coupling), the flexible mode (`gen::flexmode`, dyn_flexible_mode), the state and its rate (`gen::rigidbody`,
//! dyn_rigid_body) and the total momentum (`gen::momentum`, dyn_total_momentum). What is left here is the engine's
//! integrator (RK4 with the inputs held, the flexible mode's sub-steps: core), the body it carries (the inertia's
//! inverses by the toolbox's `la::inv`) and the calls, by the names the engine has always used. Total angular momentum
//! is conserved: rotor torques are internal.
use crate::gen::{flexmode, momentum as mo, rigidbody, rotors};
use crate::la::*;
use crate::pm::*;
use crate::{NG, NR};

/// Rotor geometry: spin axes A0 (3 x nr), their transverse axes T0, gimbal axes G (3 x ng), gimbal index gi (0 = fixed).
pub use crate::gen::rotors::RotorGeometry as Geometry;

impl Geometry {
    pub fn new(a0: &[V3], g: &[V3], gi: &[usize]) -> Geometry {
        let (mut a, mut gg, mut gix) = ([[0.0; 3]; NR], [[0.0; 3]; NG], [0i64; NR]);
        for i in 0..a0.len() { a[i] = a0[i]; gix[i] = gi[i] as i64; }
        for j in 0..g.len() { gg[j] = g[j]; }
        rotors::rotor_geometry(a, gg, gix, a0.len() as i64, g.len() as i64)
    }
    /// Current spin axes at gimbal angles d.
    pub fn axes(&self, d: &[f64; NG]) -> [V3; NR] { rotors::rotor_axes(*self, *d) }
    /// How many rotors and gimbals.
    pub fn nr(&self) -> usize { self.nr as usize }
    pub fn ng(&self) -> usize { self.ng as usize }
}

/// State: attitude (ECI -> body), body rate, rotor momenta, gimbal angles, and the flexible
/// mode's coordinate and rate (zero on a rigid body).
pub use crate::gen::rigidbody::PlantState as State;

/// One flexible mode in hybrid coordinates (Hughes, Spacecraft Attitude Dynamics, ch. 12): delta [sqrt(kg) m] couples
/// the mode to the body, omega [rad/s] and zeta its frequency and damping.
pub use crate::gen::flexmode::Flex;

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
        Body { i, iinv: inv(&i), m, flex, minv: inv(&flexmode::flex_reduced_inertia(i, flex.delta)) }
    }
}

fn deriv(x: &State, b: &Body, tau_ext: &V3, tau_r: &[f64; NR], gdot: &[f64; NG]) -> State {
    rigidbody::plant_deriv(*x, b.i, b.iinv, b.minv, b.m, b.flex, *tau_ext, *tau_r, *gdot)
}

fn axpy(x: &State, h: f64, k: &State, m: &Geometry) -> State {
    let mut y = *x;
    for i in 0..4 { y.q[i] += h*k.q[i]; }
    for i in 0..3 { y.w[i] += h*k.w[i]; }
    for i in 0..m.nr() { y.h[i] += h*k.h[i]; }
    for j in 0..m.ng() { y.d[j] += h*k.d[j]; }
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
    for i in 0..m.nr() { y.h[i] += dt/6.0*(k1.h[i] + 2.0*k2.h[i] + 2.0*k3.h[i] + k4.h[i]); }
    for j in 0..m.ng() { y.d[j] += dt/6.0*(k1.d[j] + 2.0*k2.d[j] + 2.0*k3.d[j] + k4.d[j]); }
    y.eta += dt/6.0*(k1.eta + 2.0*k2.eta + 2.0*k3.eta + k4.eta);
    y.etad += dt/6.0*(k1.etad + 2.0*k2.etad + 2.0*k3.etad + k4.etad);
    y.q = qnorm(&y.q);
    y
}

/// Total angular momentum in the body frame (conservation diagnostic).
pub fn momentum(x: &State, b: &Body) -> V3 { mo::total_momentum(*x, b.i, b.m, b.flex) }
