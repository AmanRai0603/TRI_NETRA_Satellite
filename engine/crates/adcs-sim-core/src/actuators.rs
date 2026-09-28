//! Actuator models (asils.devices.{mtq,mex,rcs}, asils.comp.fluid_loop.drive).
use crate::la::*;
use crate::pm::*;
use crate::rng::Rng;
use crate::{NC, NG, NR, NS};

// ---------------- magnetorquers ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct MtqDesc { pub fitted: bool, pub n: usize, pub axes: [V3; NS], pub m_max: f64, pub p_max: f64, pub scale_sigma: f64, pub misalign: f64 }
#[derive(Clone, Copy, Debug, Default)]
pub struct Mtq { pub d: MtqDesc, pub a: [V3; NS], pub scale: [f64; NS], pub dead: [bool; NS], pinv: [[f64; 3]; NS] }
impl Mtq {
    pub fn new(d: MtqDesc, disp: &mut Rng) -> Mtq {
        let mut s = Mtq { d, ..Default::default() };
        for j in 0..d.n { s.scale[j] = 1.0 + d.scale_sigma*disp.normal(); }
        for j in 0..d.n { let m = small_rot(&scale(&disp.normal3(), d.misalign)); s.a[j] = mv(&m, &d.axes[j]); }
        // nominal per-coil allocation pinv(axes) = At (A At)^-1 (the driver board's table)
        let mut aat = [[0.0; 3]; 3];
        for j in 0..d.n { for r in 0..3 { for c in 0..3 { aat[r][c] += d.axes[j][r]*d.axes[j][c]; } } }
        let ai = inv(&aat);
        for j in 0..d.n { s.pinv[j] = mtv(&ai, &d.axes[j]); }
        s
    }
    /// Commanded body dipole -> (true body dipole, power).
    pub fn apply(&self, m_body_cmd: &V3) -> (V3, f64) {
        let mut m = [0.0; 3];
        let mut p = 0.0;
        for j in 0..self.d.n {
            let mut mc = clamp(dot(&self.pinv[j], m_body_cmd), -self.d.m_max, self.d.m_max);
            if self.dead[j] { mc = 0.0; }
            p += abs(mc)/self.d.m_max*self.d.p_max;
            m = add(&m, &scale(&self.a[j], mc*self.scale[j]));
        }
        (m, p)
    }
}

// ---------------- momentum-exchange devices ----------------
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Kind { #[default] Rw, Fmr, Cmg, Vscmg }

#[derive(Clone, Copy, Debug, Default)]
pub struct MexDesc {
    pub n: usize, pub ng: usize, pub kind: [Kind; NR], pub a0: [V3; NR], pub gi: [usize; NR], pub g: [V3; NG],
    pub h_max: [f64; NR], pub torque_max: [f64; NR], pub jrot: [f64; NR], pub coulomb: [f64; NR], pub viscous: [f64; NR],
    pub p_steady: [f64; NR], pub tsig: [f64; NR], pub flo: [f64; NR], pub fhi: [f64; NR], pub misalign: [f64; NR],
    pub t_sd: [f64; NR], pub k_hv: [f64; NR], pub ac: [f64; NR], pub s: [f64; NR], pub l: [f64; NR], pub flow_noise_h: [f64; NR],
    pub field_power: [f64; NR], pub eta_lo: [f64; NR], pub eta_hi: [f64; NR], pub h0: [f64; NR],
    pub torque_noise: f64, pub friction_comp: f64, pub eta: f64, pub k_speed: f64, pub k_flow: f64, pub flow_tau: f64,
    pub gimbal_rate_max: f64, pub gimbal_power: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Mex {
    pub d: MexDesc, pub a0: [V3; NR], pub tscale: [f64; NR], pub fscale: [f64; NR], pub eta: [f64; NR],
    pub failed: [bool; NR], pub gfailed: [bool; NG], htgt: [f64; NR], hf: [f64; NR], pub rng: Rng,
}
impl Mex {
    pub fn new(d: MexDesc, disp: &mut Rng, noise: Rng) -> Mex {
        let mut s = Mex { d, rng: noise, ..Default::default() };
        for i in 0..d.n { s.tscale[i] = 1.0 + d.tsig[i]*disp.normal(); }
        for i in 0..d.n { s.fscale[i] = d.flo[i] + (d.fhi[i] - d.flo[i])*disp.uniform(); }
        for i in 0..d.n { s.eta[i] = d.eta_lo[i] + (d.eta_hi[i] - d.eta_lo[i])*disp.uniform(); }
        for i in 0..d.n { s.a0[i] = mv(&small_rot(&scale(&disp.normal3(), d.misalign[i])), &d.a0[i]); }
        s
    }
    /// Commands -> (rotor momentum rates, gimbal rates, power).
    pub fn apply(&mut self, cmd_r: &[f64; NR], cmd_g: &[f64; NG], h: &[f64; NR], dt: f64) -> ([f64; NR], [f64; NG], f64) {
        let m = self.d;
        let mut hd = [0.0; NR];
        let mut p = 0.0;
        for i in 0..m.n {
            if self.failed[i] { hd[i] = -m.viscous[i]*h[i]/m.jrot[i] - m.coulomb[i]*sign(h[i]); continue; }
            match m.kind[i] {
                Kind::Rw | Kind::Vscmg => {
                    let tc = clamp(cmd_r[i], -m.torque_max[i], m.torque_max[i])*self.tscale[i];
                    let om = h[i]/m.jrot[i];
                    let fr = (m.coulomb[i]*sign(om) + m.viscous[i]*om)*self.fscale[i];
                    hd[i] = tc - (1.0 - m.friction_comp)*fr + m.torque_noise*m.torque_max[i]*self.rng.normal();
                    if abs(h[i]) >= m.h_max[i] && sign(hd[i]) == sign(h[i]) { hd[i] = -(1.0 - m.friction_comp)*fr; }
                    p += m.p_steady[i] + abs(tc*om)/m.eta;
                }
                Kind::Fmr => {
                    let tsd = m.t_sd[i];
                    self.htgt[i] = clamp(self.htgt[i] + cmd_r[i]*dt, -m.h_max[i], m.h_max[i]);
                    let nz = self.rng.normal();
                    self.hf[i] += dt/(m.flow_tau + dt)*(h[i] + m.flow_noise_h[i]*nz - self.hf[i]);
                    let pump = clamp(cmd_r[i] + self.hf[i]/tsd + m.k_flow*(self.htgt[i] - self.hf[i]), -m.torque_max[i], m.torque_max[i]);
                    hd[i] = pump - h[i]/tsd*self.fscale[i];
                    if abs(h[i]) >= m.h_max[i] && sign(hd[i]) == sign(h[i]) { hd[i] = 0.0; }
                    let v = h[i]/m.k_hv[i];
                    let dp = pump*m.l[i]/(2.0*m.s[i]*m.ac[i]);
                    p += abs(dp*m.ac[i]*v)/self.eta[i];
                    if abs(self.htgt[i]) > 0.02*m.h_max[i] || abs(cmd_r[i]) > 0.02*m.torque_max[i] { p += m.field_power[i]; }
                }
                Kind::Cmg => {
                    hd[i] = clamp(-m.k_speed*(h[i] - m.h0[i]), -m.torque_max[i], m.torque_max[i]);
                    p += m.p_steady[i];
                }
            }
        }
        let mut gd = [0.0; NG];
        for j in 0..m.ng {
            if self.gfailed[j] { continue; }
            gd[j] = clamp(cmd_g[j], -m.gimbal_rate_max, m.gimbal_rate_max);
            p += m.gimbal_power*abs(gd[j])/m.gimbal_rate_max;
        }
        (hd, gd, p)
    }
}

// ---------------- cold-gas thrusters ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct RcsDesc {
    pub fitted: bool, pub nc: usize, pub tau: [V3; NC], pub thrust: f64, pub isp: f64, pub mib: f64, pub res: f64,
    pub prop_kg: f64, pub valve_power: f64, pub isp_lo: f64, pub isp_hi: f64, pub thrust_sigma: f64, pub misalign: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Rcs { pub d: RcsDesc, pub tau: [V3; NC], pub tscale: [f64; NC], pub failed: [bool; NC], pub isp: f64 }
impl Rcs {
    pub fn new(d: RcsDesc, disp: &mut Rng) -> Rcs {
        let mut s = Rcs { d, ..Default::default() };
        for j in 0..d.nc { s.tscale[j] = 1.0 + d.thrust_sigma*disp.normal(); }
        for j in 0..d.nc { s.tau[j] = mv(&small_rot(&scale(&disp.normal3(), d.misalign)), &d.tau[j]); }
        s.isp = d.isp_lo + (d.isp_hi - d.isp_lo)*disp.uniform();
        s
    }
    /// Duty per couple over T -> (torque, mass flow, power).
    pub fn apply(&self, duty: &[f64; NC], t: f64) -> (V3, f64, f64) {
        let mut tau = [0.0; 3];
        let (mut fs, mut p) = (0.0, 0.0);
        for j in 0..self.d.nc {
            let mut on = clamp(duty[j], 0.0, 1.0)*t;
            if on < self.d.mib { on = 0.0; }
            on = round(on/self.d.res)*self.d.res;
            if self.failed[j] { on = 0.0; }
            let f = on/t;
            tau = add(&tau, &scale(&self.tau[j], f*self.tscale[j]));
            fs += f*self.tscale[j];
            if on > 0.0 { p += self.d.valve_power; }
        }
        (tau, fs*2.0*self.d.thrust/(self.isp*9.80665), p)
    }
}
