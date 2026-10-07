//! In-loop orbit propagator: RK4 at a fixed node step with cubic Hermite
//! interpolation between nodes (the structure of asils.orbit.{advance,state}).
//! Forces: zonal gravity J2..J6, Sun and Moon point masses, cannonball drag
//! (co-rotating atmosphere) and SRP with a conical shadow. The MATLAB twin runs
//! the POP (degree-6 field with tesserals, DE440, DTM2020); the differences are
//! parity-ledger lines, not failures.
use crate::la::*;
use crate::pm::*;
use crate::{atmos, ephem, field, time};

pub const MU: f64 = 3.986004418e14;
pub const RE: f64 = 6378137.0;
pub const OMEGA_E: f64 = 7.292115e-5;
pub const MU_SUN: f64 = 1.32712440018e20;
pub const MU_MOON: f64 = 4.9028e12;
const J: [f64; 7] = [0.0, 0.0, 1.08262668e-3, -2.53265649e-6, -1.61962159e-6, -2.27296083e-7, 5.40681239e-7];

#[derive(Clone, Copy, Debug)]
pub struct OrbitCfg {
    pub jd0_utc: f64, pub step_s: f64, pub zonal_max: usize,
    pub third_body: bool, pub drag: bool, pub srp: bool,
    pub mass_kg: f64, pub area_m2: f64, pub cd: f64, pub cr: f64, pub density_scale: f64,
}

/// What the environment needs at a node (interpolated in between).
#[derive(Clone, Copy, Debug, Default)]
pub struct Ctx { pub sun: V3, pub moon: V3, pub p_srp: f64, pub rho: f64 }

#[derive(Clone, Copy, Debug)]
struct Node { t: f64, r: V3, v: V3, a: V3, x: Ctx }

#[derive(Clone, Copy, Debug)]
pub struct Orbit { pub cfg: OrbitCfg, n0: Node, n1: Node }

fn ctx(cfg: &OrbitCfg, t: f64, r: &V3) -> Ctx {
    let jd_tt = cfg.jd0_utc + (t + time::TT_MINUS_UTC_S)/86400.0;
    let sun = ephem::sun(jd_tt);
    let moon = ephem::moon(jd_tt);
    let c = time::eci2ecef(cfg.jd0_utc + t/86400.0);
    let (_, _, h) = field::geodetic(&mv(&c, r));
    Ctx { sun, moon, p_srp: ephem::p_srp(norm(&sub(&sun, r))), rho: atmos::density(h, cfg.density_scale) }
}

/// Acceleration [m/s^2] at (t, r, v) with the node context x.
pub fn accel(cfg: &OrbitCfg, r: &V3, v: &V3, x: &Ctx) -> V3 {
    let rn = norm(r);
    let mut a = scale(r, -MU/(rn*rn*rn));
    // zonal harmonics: U_n = -mu J_n Re^n P_n(s) / r^(n+1), s = z/r
    let s = r[2]/rn;
    let rh = scale(r, 1.0/rn);
    let (mut pm1, mut p, mut dpm1, mut dp) = (1.0, s, 0.0, 1.0);
    for n in 1..cfg.zonal_max {
        let nf = n as f64;
        let pn1 = ((2.0*nf + 1.0)*s*p - nf*pm1)/(nf + 1.0);
        let dpn1 = dpm1 + (2.0*nf + 1.0)*p;
        pm1 = p; p = pn1; dpm1 = dp; dp = dpn1;
        let deg = n + 1;
        if deg >= 2 && deg < J.len() {
            let df = deg as f64;
            let f = -MU*J[deg]*pow(RE/rn, deg)/rn;
            for i in 0..3 {
                let zi = if i == 2 { 1.0 } else { 0.0 };
                a[i] += f/rn*(-(df + 1.0)*p*rh[i] + dp*(zi - s*rh[i]));
            }
        }
    }
    if cfg.third_body {
        for (rb, mu) in [(&x.sun, MU_SUN), (&x.moon, MU_MOON)] {
            let d = sub(rb, r);
            let (dn, bn) = (norm(&d), norm(rb));
            for i in 0..3 { a[i] += mu*(d[i]/(dn*dn*dn) - rb[i]/(bn*bn*bn)); }
        }
    }
    let am = cfg.area_m2/cfg.mass_kg;
    if cfg.drag && x.rho > 0.0 {
        let vr = [v[0] + OMEGA_E*r[1], v[1] - OMEGA_E*r[0], v[2]];
        let vn = norm(&vr);
        for i in 0..3 { a[i] -= 0.5*cfg.cd*am*x.rho*vn*vr[i]; }
    }
    if cfg.srp {
        let nu = ephem::shadow(r, &x.sun);
        if nu > 0.0 {
            let d = sub(r, &x.sun);
            let dn = norm(&d);
            for i in 0..3 { a[i] += nu*x.p_srp*cfg.cr*am*d[i]/dn; }
        }
    }
    a
}

fn pow(x: f64, n: usize) -> f64 { let mut y = 1.0; for _ in 0..n { y *= x; } y }

impl Orbit {
    pub fn new(cfg: OrbitCfg, r0: V3, v0: V3) -> Orbit {
        let x0 = ctx(&cfg, 0.0, &r0);
        let n0 = Node { t: 0.0, r: r0, v: v0, a: accel(&cfg, &r0, &v0, &x0), x: x0 };
        let n1 = Self::advance(&cfg, &n0);
        Orbit { cfg, n0, n1 }
    }

    fn advance(cfg: &OrbitCfg, n: &Node) -> Node {
        let h = cfg.step_s;
        let (r, v) = (n.r, n.v);
        let f = |t: f64, r: &V3, v: &V3| { let x = ctx(cfg, t, r); accel(cfg, r, v, &x) };
        let k1v = n.a; let k1r = v;
        let r2 = add(&r, &scale(&k1r, 0.5*h)); let v2 = add(&v, &scale(&k1v, 0.5*h));
        let k2v = f(n.t + 0.5*h, &r2, &v2); let k2r = v2;
        let r3 = add(&r, &scale(&k2r, 0.5*h)); let v3 = add(&v, &scale(&k2v, 0.5*h));
        let k3v = f(n.t + 0.5*h, &r3, &v3); let k3r = v3;
        let r4 = add(&r, &scale(&k3r, h)); let v4 = add(&v, &scale(&k3v, h));
        let k4v = f(n.t + h, &r4, &v4); let k4r = v4;
        let mut r1 = [0.0; 3];
        let mut v1 = [0.0; 3];
        for i in 0..3 {
            r1[i] = r[i] + h/6.0*(k1r[i] + 2.0*k2r[i] + 2.0*k3r[i] + k4r[i]);
            v1[i] = v[i] + h/6.0*(k1v[i] + 2.0*k2v[i] + 2.0*k3v[i] + k4v[i]);
        }
        let x1 = ctx(cfg, n.t + h, &r1);
        Node { t: n.t + h, r: r1, v: v1, a: accel(cfg, &r1, &v1, &x1), x: x1 }
    }

    /// Position and velocity at t (t non-decreasing between calls).
    pub fn state(&mut self, t: f64) -> (V3, V3) {
        while t > self.n1.t + 1e-9 {
            self.n0 = self.n1;
            self.n1 = Self::advance(&self.cfg, &self.n0);
        }
        let h = self.n1.t - self.n0.t;
        let s = (t - self.n0.t)/h;
        let (s2, s3) = (s*s, s*s*s);
        let (h00, h10, h01, h11) = (2.0*s3 - 3.0*s2 + 1.0, s3 - 2.0*s2 + s, -2.0*s3 + 3.0*s2, s3 - s2);
        let mut r = [0.0; 3];
        let mut v = [0.0; 3];
        for i in 0..3 {
            r[i] = h00*self.n0.r[i] + h10*h*self.n0.v[i] + h01*self.n1.r[i] + h11*h*self.n1.v[i];
            v[i] = h00*self.n0.v[i] + h10*h*self.n0.a[i] + h01*self.n1.v[i] + h11*h*self.n1.a[i];
        }
        (r, v)
    }

    /// Environment context at t (linear; density log-linear), as asils.orbit.context.
    pub fn context(&self, t: f64) -> Ctx {
        let s = (t - self.n0.t)/(self.n1.t - self.n0.t);
        let (a, b) = (&self.n0.x, &self.n1.x);
        let lerp = |x: &V3, y: &V3| [x[0] + s*(y[0] - x[0]), x[1] + s*(y[1] - x[1]), x[2] + s*(y[2] - x[2])];
        let rho = if a.rho > 0.0 && b.rho > 0.0 { exp(ln(a.rho) + s*(ln(b.rho) - ln(a.rho))) } else { a.rho + s*(b.rho - a.rho) };
        Ctx { sun: lerp(&a.sun, &b.sun), moon: lerp(&a.moon, &b.moon), p_srp: a.p_srp + s*(b.p_srp - a.p_srp), rho }
    }
}

/// Classical elements -> ECI state (argument of latitude u for a circular orbit): env's method
/// (env_two_body_elements, `gen::elements::coe2rv`).
pub fn coe2rv(a: f64, e: f64, inc: f64, raan: f64, argp: f64, nu: f64) -> (V3, V3) { crate::gen::elements::coe2rv(a, e, inc, raan, argp, nu, MU) }
