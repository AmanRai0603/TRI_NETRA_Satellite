//! Actuator models (asils.devices.{mtq,mex,rcs}, asils.comp.fluid_loop.drive): act's methods, generated from the
//! design (tools/engine_build.py): the coil set (`gen::coilset`, l3_mtq_row_13, over the rows' coillag, coilsat,
//! coilpower, coilaxes, coildisp), the momentum devices (`gen::rotorset`, act_rotor_set, over the wheels', rings',
//! CMG's, VSCMG's and gimbals' rows) and the thrusters (`gen::thrusters`, l3_rcs_row_11, over the thruster rows); their
//! random draws are the language's streams (rng.rs's, value for value). What is left here is the engine's descriptors
//! as product.rs fills them from the parts (code: the reading), handed to the generated models, and the units in flight
//! as the generated state, by the names the engine has always used.
use crate::gen::{coillag, coilset, rotorset, thrusters, wheelmotor, wheelspeed};
use crate::la::*;
use crate::rng::Rng;
use crate::{NC, NG, NR, NS};

// ---------------- magnetorquers ----------------
/// A coil is a series RL circuit behind a current-limited driver: its dipole follows the
/// saturated command as a first-order lag with the coil's L/R time constant `tau` [s]
/// (part `time_constant_s`). Air-core coils: no core, so no hysteresis.
#[derive(Clone, Copy, Debug, Default)]
pub struct MtqDesc { pub fitted: bool, pub n: usize, pub axes: [V3; NS], pub m_max: f64, pub p_max: f64, pub scale_sigma: f64, pub misalign: f64, pub tau: f64 }
impl MtqDesc {
    /// The design's record of the coils.
    pub fn rec(&self) -> coilset::CoilDesc {
        coilset::CoilDesc { fitted: self.fitted, n: self.n as i64, axes: self.axes, m_max: self.m_max, p_max: self.p_max, scale_sigma: self.scale_sigma,
                            misalign: self.misalign, tau: self.tau }
    }
}
/// The coil set in flight: the generated state (each coil's dispersed axis and scale, whether it has failed, its row of
/// the allocation, its dipole and last command), read and written through `Deref`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mtq { pub d: MtqDesc, dg: coilset::CoilDesc, s: coilset::CoilSet }
impl core::ops::Deref for Mtq { type Target = coilset::CoilSet; fn deref(&self) -> &coilset::CoilSet { &self.s } }
impl core::ops::DerefMut for Mtq { fn deref_mut(&mut self) -> &mut coilset::CoilSet { &mut self.s } }

/// One step `dt` of the lag x' = (u - x)/tau with u held: (x at the end of the step, x
/// averaged over it). The average is what a field constant over the step turns into torque.
pub fn lag(x0: f64, u: f64, tau: f64, dt: f64) -> (f64, f64) { coillag::coil_lag(x0, u, tau, dt) }

impl Mtq {
    pub fn new(d: MtqDesc, disp: &mut Rng) -> Mtq {
        let dg = d.rec();
        let mut g = disp.stream();
        let s = coilset::coilset_new(dg, &mut g);
        *disp = Rng::from_stream(&g);
        Mtq { d, dg, s }
    }
    /// Commanded body dipole held for `dt` -> (true body dipole averaged over the step, true body
    /// dipole at its end, power). Each coil saturates at m_max, then lags with tau; a failed
    /// coil is open (its current stops at once). Power is linear in the mean drive.
    pub fn apply(&mut self, m_body_cmd: &V3, dt: f64) -> (V3, V3, f64) { coilset::coilset_apply(&mut self.s, self.dg, *m_body_cmd, dt) }
}

/// The torque [N m] of the body dipole m in the body field b (l3_mtq_row_06's `coil_torque`, m x B).
pub fn coil_torque(m: &V3, b: &V3) -> V3 { crate::gen::coiltorque::coil_torque(*m, *b) }

// ---------------- momentum-exchange devices ----------------
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Kind { #[default] Rw, Fmr, Cmg, Vscmg }
impl Kind {
    /// The design's choice RotorKind (`gen::rotorset::ROTORKIND_*`).
    pub fn choice(self) -> i64 {
        match self { Kind::Rw => rotorset::ROTORKIND_RW, Kind::Fmr => rotorset::ROTORKIND_FMR, Kind::Cmg => rotorset::ROTORKIND_CMG, Kind::Vscmg => rotorset::ROTORKIND_VSCMG }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MexDesc {
    pub n: usize, pub ng: usize, pub kind: [Kind; NR], pub a0: [V3; NR], pub gi: [usize; NR], pub g: [V3; NG],
    pub h_max: [f64; NR], pub torque_max: [f64; NR], pub jrot: [f64; NR], pub coulomb: [f64; NR], pub viscous: [f64; NR],
    pub p_steady: [f64; NR], pub tsig: [f64; NR], pub flo: [f64; NR], pub fhi: [f64; NR], pub misalign: [f64; NR],
    pub t_sd: [f64; NR], pub k_hv: [f64; NR], pub ac: [f64; NR], pub s: [f64; NR], pub l: [f64; NR], pub flow_noise_h: [f64; NR],
    pub field_power: [f64; NR], pub eta_lo: [f64; NR], pub eta_hi: [f64; NR], pub h0: [f64; NR],
    /// reaction wheel (Kind::Rw) motor and bearing: the drive's speed limit [rad/s]; the motor's
    /// stall torque k_t V / R [N m] and no-load speed V / k_t [rad/s] (its torque-speed line);
    /// the breakaway (static) friction [N m] and the Stribeck speed [rad/s]
    pub speed_max: [f64; NR], pub t_stall: [f64; NR], pub w_nl: [f64; NR], pub f_static: [f64; NR], pub w_stribeck: [f64; NR],
    pub torque_noise: f64, pub friction_comp: f64, pub eta: f64, pub k_speed: f64, pub k_flow: f64, pub flow_tau: f64,
    pub gimbal_rate_max: f64, pub gimbal_power: f64,
}
impl MexDesc {
    /// The design's record of the momentum devices.
    pub fn rec(&self) -> rotorset::RotorDesc {
        let mut kind = [0i64; NR];
        let mut gi = [0i64; NR];
        for i in 0..NR { kind[i] = self.kind[i].choice(); gi[i] = self.gi[i] as i64; }
        rotorset::RotorDesc {
            n: self.n as i64, ng: self.ng as i64, kind, a0: self.a0, gi, h_max: self.h_max, torque_max: self.torque_max, jrot: self.jrot,
            coulomb: self.coulomb, viscous: self.viscous, p_steady: self.p_steady, tsig: self.tsig, flo: self.flo, fhi: self.fhi,
            misalign: self.misalign, t_sd: self.t_sd, k_hv: self.k_hv, ac: self.ac, s: self.s, l: self.l, flow_noise_h: self.flow_noise_h,
            field_power: self.field_power, eta_lo: self.eta_lo, eta_hi: self.eta_hi, h0: self.h0, speed_max: self.speed_max,
            t_stall: self.t_stall, w_nl: self.w_nl, f_static: self.f_static, w_stribeck: self.w_stribeck, g: self.g,
            torque_noise: self.torque_noise, friction_comp: self.friction_comp, eta: self.eta, k_speed: self.k_speed, k_flow: self.k_flow,
            flow_tau: self.flow_tau, gimbal_rate_max: self.gimbal_rate_max, gimbal_power: self.gimbal_power,
        }
    }
}
/// The torque a wheel's motor delivers for the driver's demand `tc` at speed `om`: inside its
/// torque-speed line (back-EMF: k_t (+-V - k_e om)/R, i.e. -T_s (1 + om/w_nl) .. T_s (1 - om/w_nl)
/// with k_e = k_t in SI), and none that would speed it past the drive's speed limit (l3_rw_row_01 and 02).
pub fn wheel_motor(m: &MexDesc, i: usize, tc: f64, om: f64) -> f64 {
    wheelspeed::speed_limited(wheelmotor::motor_line(tc, om, m.t_stall[i], m.w_nl[i]), om, m.speed_max[i])
}

/// The momentum devices in flight: the generated state (each rotor's dispersed axis, torque and friction factors and
/// pump efficiency, whether it or a gimbal has failed, a ring's target, filtered flow and field, the noise stream),
/// read and written through `Deref`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mex { pub d: MexDesc, dg: rotorset::RotorDesc, s: rotorset::RotorSet }
impl core::ops::Deref for Mex { type Target = rotorset::RotorSet; fn deref(&self) -> &rotorset::RotorSet { &self.s } }
impl core::ops::DerefMut for Mex { fn deref_mut(&mut self) -> &mut rotorset::RotorSet { &mut self.s } }
impl Mex {
    pub fn new(d: MexDesc, disp: &mut Rng, noise: Rng) -> Mex {
        let dg = d.rec();
        let mut g = disp.stream();
        let s = rotorset::rotorset_new(dg, &mut g, noise.stream());
        *disp = Rng::from_stream(&g);
        Mex { d, dg, s }
    }
    /// Commands -> (rotor momentum rates, gimbal rates, power).
    pub fn apply(&mut self, cmd_r: &[f64; NR], cmd_g: &[f64; NG], h: &[f64; NR], dt: f64) -> ([f64; NR], [f64; NG], f64) {
        rotorset::rotorset_apply(&mut self.s, self.dg, *cmd_r, *cmd_g, *h, dt)
    }
}

// ---------------- cold-gas thrusters ----------------
#[derive(Clone, Copy, Debug, Default)]
pub struct RcsDesc {
    pub fitted: bool, pub nc: usize, pub tau: [V3; NC], pub thrust: f64, pub isp: f64, pub mib: f64, pub res: f64,
    pub prop_kg: f64, pub valve_power: f64, pub isp_lo: f64, pub isp_hi: f64, pub thrust_sigma: f64, pub misalign: f64,
}
impl RcsDesc {
    /// The design's record of the thrusters.
    pub fn rec(&self) -> thrusters::ThrusterDesc {
        thrusters::ThrusterDesc { fitted: self.fitted, nc: self.nc as i64, tau: self.tau, thrust: self.thrust, isp: self.isp, mib: self.mib, res: self.res,
            prop_kg: self.prop_kg, valve_power: self.valve_power, isp_lo: self.isp_lo, isp_hi: self.isp_hi, thrust_sigma: self.thrust_sigma, misalign: self.misalign }
    }
}
/// The thrusters in flight: the generated state (each couple's dispersed torque and thrust factor, whether it has
/// failed, the unit's specific impulse), read and written through `Deref`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rcs { pub d: RcsDesc, dg: thrusters::ThrusterDesc, s: thrusters::ThrusterSet }
impl core::ops::Deref for Rcs { type Target = thrusters::ThrusterSet; fn deref(&self) -> &thrusters::ThrusterSet { &self.s } }
impl core::ops::DerefMut for Rcs { fn deref_mut(&mut self) -> &mut thrusters::ThrusterSet { &mut self.s } }
impl Rcs {
    pub fn new(d: RcsDesc, disp: &mut Rng) -> Rcs {
        let dg = d.rec();
        let mut g = disp.stream();
        let s = thrusters::thrusters_new(dg, &mut g);
        *disp = Rng::from_stream(&g);
        Rcs { d, dg, s }
    }
    /// Duty per couple over T -> (torque, mass flow, power).
    pub fn apply(&self, duty: &[f64; NC], t: f64) -> (V3, f64, f64) { thrusters::thrusters_apply(self.s, self.dg, *duty, t) }
    /// The propellant used after a step `dt` at flow `mdot`; an empty tank fails every valve (l3_rcs_row_08 and 11).
    pub fn spend(&mut self, used: f64, mdot: f64, dt: f64) -> f64 {
        let (u, empty) = crate::gen::rcsprop::tank_update(used, mdot, dt, self.d.prop_kg);
        if empty { self.s = thrusters::thrusters_empty(self.s); }
        u
    }
}
