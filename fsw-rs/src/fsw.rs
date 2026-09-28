//! The reference flight software over the Hal trait; twin of fsw/src/adcs_fsw.c.
//! Tick order and mode manager: fsw/pseudocode/08_mode_manager.md, one branch per
//! controller state as asils.fsw.step (MATLAB) and adcs_fsw.c (C).
use crate::alloc::{self, A38};
use crate::ctl::{self, Gains, Guid};
use crate::drv::{self, Drv, Meas};
use crate::env;
use crate::est::{self, Mekf};
use crate::hal::Hal;
use crate::m::*;
use crate::math::*;
use crate::params::{Mode, Params, MAX_COUPLES as NC, MAX_GIMBALS as NG, MAX_ROTORS as NR, MODE_NONE};

pub const ABI_VERSION: u32 = 1;
pub const BUILD_ID: &str = "trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1)";
const MODE_COUNT: u8 = 11;

#[derive(Debug, PartialEq, Eq)]
pub enum InitError { Abi, Config }

/// What adcs_fsw_peek reports (floats, quaternion scalar first).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct State {
    pub t_ns: u64, pub q_est: [f32; 4], pub w_est: [f32; 3], pub b_est: [f32; 3], pub h_int: [f32; 3],
    pub m_cmd: [f32; 3], pub u_cmd: [f32; 8], pub mode: u16, pub faults: u16,
}

#[derive(Clone, Copy, Default)]
pub struct Fsw {
    pub p: Params,
    ready: bool,
    start_ns: u64,
    t: f64, jd: f64,
    mode: u8,
    t_mode: f64, hold: f64,
    z: Meas,
    drv: Drv,
    clean: bool,
    have_r: bool, r: V3, v: V3,
    k: Mekf, ad_ok: bool, t_st: f64,
    w_est: V3,
    gh: Option<[f64; 195]>, bref: Option<V3>, t_bref: f64,
    bsum: V3, bsum_raw: V3, bn: u32,
    b1: Option<V3>, b1raw: Option<V3>,
    m_hold: V3, b_dump: V3,
    sigma: f64, sz_sum: f64, sz_n: u32, sz_t0: f64,
    s_prop: Option<V3>, acq_hold: f64,
    gd: Guid, g_rw: Gains, g_mtq: Gains,
    q_ref: Q, w_ref: V3, tau_req: V3, i_q: V3, last_ctrl: f64, capturing: bool,
    h_prev: Option<[f64; NR]>, cmd_r_prev: [f64; NR], rot_failed: [bool; NR], fd_count: [f64; NR],
    h_t_rot: [f64; NR], h_t: V3, cap: V3, hcap: V3, dump_hi: f64, dump_lo: f64,
    has_rcs_dump: bool, rcs_dumping: bool, rcs_left: Option<[f64; NC]>,
    sched_i: usize,
    out_m: V3, out_r: [f64; NR], out_g: [f64; NG], out_duty: [f64; NC],
    faults: u16,
}

fn guid_kind_of(mode: u8) -> i32 {
    match Mode::from_u8(mode) {
        Some(Mode::TargetFine) => 1,
        Some(Mode::SlewFine) => 2,
        Some(Mode::SunMtq) | Some(Mode::SunFine) => 4,
        _ => 0,
    }
}

#[allow(clippy::too_many_arguments)]
fn gains(law: u8, kp: &V3, kd: &V3, ki: &V3, klqr: &M3, lambda: f64, phi: f64, gs: &V3, err_max: f64, int_max: f64) -> Gains {
    Gains { law, kp: *kp, kd: *kd, ki: *ki, klqr: *klqr, lambda, phi, gs: *gs, err_max, int_max }
}

const DETUMBLE: u8 = Mode::Detumble as u8;
const NADIR_MTQ: u8 = Mode::NadirMtq as u8;
const NADIR_FINE: u8 = Mode::NadirFine as u8;
const TARGET_FINE: u8 = Mode::TargetFine as u8;
const SLEW_FINE: u8 = Mode::SlewFine as u8;
const SPINUP: u8 = Mode::Spinup as u8;
const SUN_SPIN: u8 = Mode::SunSpin as u8;
const DETUMBLE_RCS: u8 = Mode::DetumbleRcs as u8;
const SUN_ACQ_ROTOR: u8 = Mode::SunAcqRotor as u8;
const SUN_MTQ: u8 = Mode::SunMtq as u8;
const SUN_FINE: u8 = Mode::SunFine as u8;

impl Fsw {
    pub fn new() -> Fsw { Fsw::default() }

    pub fn init(&mut self, abi_version: u32, blob: &[u8], start_ns: u64) -> Result<(), InitError> {
        *self = Fsw::default();
        if abi_version != ABI_VERSION { return Err(InitError::Abi); }
        let p = Params::decode(blob).map_err(|_| InitError::Config)?;
        self.p = p;
        self.start_ns = start_ns;
        self.mode = p.start_mode;
        self.t_st = -1e9; self.t_bref = -1e9; self.last_ctrl = -1e9;
        self.sigma = p.ss_sigma0;
        self.clean = true;
        self.gd = Guid {
            q_off: p.gd_q_off, roll_deg: p.gd_roll_deg, t0: p.gd_t0, t_slew: p.gd_T, axis: p.gd_axis,
            q_inertial: p.gd_q_inertial, sun_axis: p.sun_axis, roll_axis: p.roll_axis, sun_eci: env::sun_model(p.jd0),
        };
        self.g_rw = gains(p.rw_law, &p.rw_Kp, &p.rw_Kd, &p.rw_Ki, &p.rw_Klqr, p.rw_lambda, p.rw_phi, &p.rw_Gs, p.rw_err_max, p.rw_int_max);
        let ml = if p.mtq_law == 1 { 1 } else if p.mtq_law == 2 { 2 } else { 0 };
        self.g_mtq = gains(ml, &p.mtq_Kp, &p.mtq_Kd, &p.mtq_Ki, &p.mtq_Klqr, p.mtq_lambda, p.mtq_phi, &p.mtq_Gs, p.mtq_err_max, p.mtq_int_max);
        let nr = p.nr as usize;
        for i in 0..nr {
            if p.rot_kind[i] == 0 {
                self.h_t_rot[i] = if p.h_bias < 0.25*p.rot_hmax[i] { p.h_bias } else { 0.25*p.rot_hmax[i] };
            } else if p.rot_kind[i] >= 2 {
                self.h_t_rot[i] = p.rot_h0[i];
            }
            if p.rot_gi[i] == 0 {
                for k in 0..3 {
                    self.cap[k] += fabs(p.rot_a0[i][k])*p.rot_tmax[i];
                    self.hcap[k] += fabs(p.rot_a0[i][k])*p.rot_hmax[i];
                    self.h_t[k] += p.rot_a0[i][k]*self.h_t_rot[i];
                }
            }
        }
        if p.ng > 0 {
            let mut h0: f64 = 0.0;
            for i in 0..nr { if p.rot_h0[i] > h0 { h0 = p.rot_h0[i]; } }
            for k in 0..3 { self.cap[k] += 2.0*h0*p.gim_rate_max; self.hcap[k] += 2.5*h0; }
        }
        self.dump_hi = p.rcs_dump_hi;
        self.dump_lo = p.rcs_dump_lo;
        if nr > 0 {
            let mut hmin = 1e300;
            for k in 0..3 { if self.hcap[k] < hmin { hmin = self.hcap[k]; } }
            if 0.5*hmin < self.dump_hi { self.dump_hi = 0.5*hmin; }
            if 0.15*hmin < self.dump_lo { self.dump_lo = 0.15*hmin; }
        }
        self.has_rcs_dump = p.nc > 0 && p.rcs_dump != 0;
        self.ready = true;
        Ok(())
    }

    fn enter(&mut self, mode: u8) {
        if mode == SPINUP { self.sz_sum = 0.0; self.sz_n = 0; self.sz_t0 = self.t; }
        self.mode = mode;
        self.t_mode = self.t;
        self.hold = 0.0;
        self.i_q = [0.0; 3];
    }

    fn mtq_law(&mut self) {
        let p = &self.p;
        self.tau_req = if p.mtq_law == 0 {
            ctl::mtq_pd(&self.k.q, &self.w_est, &self.q_ref, &self.w_ref, &self.g_mtq)
        } else if p.mtq_law == 3 {
            let qe = qmult(&qconj(&self.q_ref), &self.k.q);
            let wr = mat3_vec(&dcm(&qe), &self.w_ref);
            let mut t = [0.0; 3];
            for i in 0..3 { t[i] = -self.g_mtq.kd[i]*(self.w_est[i] - wr[i]); }
            t
        } else {
            ctl::control_law(&self.k.q, &self.w_est, &self.q_ref, &self.w_ref, &mut self.i_q, p.mtq_period, &self.g_mtq, &p.J, &[0.0; 3], &[0.0; 3])
        };
    }

    fn capture_law(&mut self, hdev: &V3) -> bool {
        let p = &self.p;
        if self.mode == SLEW_FINE || p.capture_deg <= 0.0 { return false; }
        let qe = qerr(&self.q_ref, &self.k.q);
        let th = 2.0*acos(if qe[3] > 1.0 { 1.0 } else { qe[3] });
        if th < p.capture_deg*D2R { return false; }
        let mut n = norm3(&[qe[0], qe[1], qe[2]]);
        if n < 1e-12 { n = 1e-12; }
        let e = [qe[0]/n, qe[1]/n, qe[2]/n];
        let mut jm = p.J[0][0];
        if p.J[1][1] > jm { jm = p.J[1][1]; }
        if p.J[2][2] > jm { jm = p.J[2][2]; }
        let (mut mincap, mut minh) = (self.cap[0], self.hcap[0]);
        for i in 1..3 {
            if self.cap[i] < mincap { mincap = self.cap[i]; }
            if self.hcap[i] < minh { minh = self.hcap[i]; }
        }
        let alpha = 0.5*mincap/jm;
        let mut wmax = p.capture_rate_deg_s*D2R;
        if 0.5*minh/jm < wmax { wmax = 0.5*minh/jm; }
        let wref = mat3_vec(&dcm(&qe), &self.w_ref);
        let mut sp = sqrt(2.0*alpha*th);
        if wmax < sp { sp = wmax; }
        let mut wc = [0.0; 3];
        for i in 0..3 { wc[i] = wref[i] - e[i]*sp; }
        let mut kr = 4.0*alpha/(if wmax > 1e-6 { wmax } else { 1e-6 });
        if kr > 0.5 { kr = 0.5; }
        let gy = cross(&self.w_est, &add3(&mat3_vec(&p.J, &self.w_est), hdev));
        let mut x = [0.0; 3];
        for i in 0..3 { x[i] = kr*(wc[i] - self.w_est[i]); }
        self.tau_req = add3(&mat3_vec(&p.J, &x), &gy);
        true
    }

    fn sun_acq_law(&mut self, hdev: &V3) {
        let p = &self.p;
        let wmax = p.sa_w_max_deg_s*D2R;
        let a = p.sun_axis;
        let mut wc = [0.0; 3];
        if let Some(sp) = self.s_prop {
            let s = unit(&sp);
            let mut c = cross(&a, &s);
            if dot(&s, &a) < -0.95 {
                c = cross(&a, &[1.0, 0.0, 0.0]);
                if norm3(&c) < 0.1 { c = cross(&a, &[0.0, 1.0, 0.0]); }
                c = unit(&c);
            }
            wc = scale3(&c, wmax/0.5);
            let n = norm3(&wc);
            if n > wmax { wc = scale3(&wc, wmax/n); }
        }
        let gy = cross(&self.w_est, &add3(&mat3_vec(&p.J, &self.w_est), hdev));
        let mut x = [0.0; 3];
        for i in 0..3 { x[i] = p.sa_kd*(wc[i] - self.w_est[i]); }
        self.tau_req = add3(&mat3_vec(&p.J, &x), &gy);
    }

    fn spin_guards(&mut self, dt: f64) {
        let p = self.p;
        let d = D2R;
        let wz = self.w_est[2];
        let wp = sqrt(self.w_est[0]*self.w_est[0] + self.w_est[1]*self.w_est[1]);
        if norm3(&self.w_est) > p.ss_omega_max_dps*d { self.enter(DETUMBLE); return; }
        if self.mode == SPINUP {
            let conv = fabs(wz - self.sigma*p.ss_spin_dps*d) < p.ss_z_in_dps*d && wp < p.ss_perp_in_dps*d;
            if conv && self.z.sun_ok { self.sz_sum += self.z.sun[2]; self.sz_n += 1; }
            if self.t - self.sz_t0 >= p.ss_t_check_s && self.sz_n > 0 && self.sz_sum/self.sz_n as f64 > p.ss_sun_min {
                self.sigma = -self.sigma; self.sz_sum = 0.0; self.sz_n = 0; self.sz_t0 = self.t;
            }
            let ok = conv && self.z.sun_ok && self.z.sun[2] < 0.0;
            if ok { self.hold += dt; } else { self.hold = 0.0; }
            if self.hold >= p.ss_dwell_in_s { self.enter(SUN_SPIN); }
        } else {
            let bad = fabs(wz) < p.ss_omega_exit_dps*d || wp > p.ss_perp_out_dps*d;
            if bad { self.hold += dt; } else { self.hold = 0.0; }
            if self.hold >= p.ss_dwell_out_s { self.enter(SPINUP); }
        }
    }

    fn allocate(&self, tau_rot: &V3, a: &A38, cmd_r: &mut [f64; NR], cmd_g: &mut [f64; NG]) {
        let p = &self.p;
        let mut fixed = [0usize; NR];
        let mut nf = 0;
        for i in 0..p.nr as usize { if p.rot_gi[i] == 0 && !self.rot_failed[i] { fixed[nf] = i; nf += 1; } }
        if nf > 0 {
            let mut af = [[0.0; 8]; 3];
            for i in 0..nf { for k in 0..3 { af[k][i] = a[k][fixed[i]]; } }
            let pi = pinv_rows(&af, nf);
            for i in 0..nf { cmd_r[fixed[i]] = -(pi[i][0]*tau_rot[0] + pi[i][1]*tau_rot[1] + pi[i][2]*tau_rot[2]); }
        }
        if p.ng > 0 {
            let wheels = (0..p.nr as usize).any(|i| p.rot_kind[i] == 3);
            let (gd, hdot) = alloc::steer_sr(tau_rot, a, &self.z.h, p, wheels);
            cmd_g[..4].copy_from_slice(&gd);
            if wheels {
                for i in 0..p.nr as usize {
                    if p.rot_gi[i] > 0 { cmd_r[i] = hdot[i] - p.cmg_k_null*(self.z.h[i] - p.rot_h0[i]); }
                }
            }
        }
    }

    fn idle_rotors(&self, cmd_r: &mut [f64; NR], zero_cmg: bool) {
        for i in 0..self.p.nr as usize {
            cmd_r[i] = -0.2*(self.z.h[i] - self.h_t_rot[i]);
            if zero_cmg && self.p.rot_gi[i] > 0 && self.p.rot_kind[i] == 2 { cmd_r[i] = 0.0; }
        }
    }

    /// One flight-software tick at now_ns (adcs_fsw_step).
    pub fn step<H: Hal>(&mut self, hal: &mut H, now_ns: u64) -> i32 {
        if !self.ready { return -1; }
        let p = self.p;
        let dt = p.dt;
        let nr = p.nr as usize;
        self.t = now_ns.wrapping_sub(self.start_ns) as f64*1e-9;
        self.jd = p.jd0 + self.t/86400.0;
        let mut z = self.z;
        self.drv.read(hal, &p, &mut z);
        if !z.gyro_ok && p.has_gyro == 0 { z.w = [0.0; 3]; }
        self.z = z;

        // 1 onboard orbit
        if z.gps_ok { self.r = z.r; self.v = z.v; self.have_r = true; }
        else if self.have_r {
            let rn = norm3(&self.r);
            let a = scale3(&self.r, -p.mu/(rn*rn*rn));
            for i in 0..3 { self.r[i] += self.v[i]*dt + 0.5*a[i]*dt*dt; self.v[i] += a[i]*dt; }
        }

        // 2 estimation
        self.gd.sun_eci = env::sun_model(self.jd);
        let phase = fmod(self.t + 1e-9, p.mtq_period);
        let first = phase < dt - 1e-9;
        if first && self.have_r && (self.t - self.t_bref) >= 0.999 {
            if self.gh.is_none() { self.gh = Some(env::igrf_gh(env::decyear(self.jd))); }
            let gh = self.gh.as_ref().unwrap();
            self.bref = Some(env::field_eci(&self.r, self.jd, gh, p.igrf_nmax as i32));
            self.t_bref = self.t;
        }
        let m = self.mode;
        if !(m == DETUMBLE || m == DETUMBLE_RCS || m == SPINUP || m == SUN_SPIN) {
            if !self.ad_ok {
                if p.has_st != 0 && z.st_ok {
                    let mut h = 0;
                    while h < p.n_heads as usize && !z.st_valid[h] { h += 1; }
                    let q0 = est::latency(&z.q_st[h], &z.w, p.st_latency);
                    self.k = Mekf::new(&q0, 1e-3, 2e-4, p.gyro_arw, p.gyro_rrw);
                    self.ad_ok = true;
                    self.t_st = self.t;
                } else if z.sun_ok && self.clean && self.bref.is_some() {
                    let q0 = est::triad(&z.sun, &z.b, &self.gd.sun_eci, self.bref.as_ref().unwrap());
                    self.k = Mekf::new(&q0, 0.05, 2e-4, p.gyro_arw, p.gyro_rrw);
                    self.ad_ok = true;
                }
            } else {
                self.k.predict(&z.w, dt);
                if p.has_st != 0 && z.st_ok {
                    for h in 0..p.n_heads as usize {
                        if z.st_valid[h] {
                            let wb = sub3(&z.w, &self.k.b);
                            let ql = est::latency(&z.q_st[h], &wb, p.st_latency);
                            self.k.quat(&ql, p.st_noise_cross*p.mekf_meas_scale, p.st_noise_roll*p.mekf_meas_scale, &p.st_bs[h]);
                        }
                    }
                    self.t_st = self.t;
                } else if p.has_st != 0 && self.t - self.t_st < p.st_coast_s {
                    // short star-tracker outage: coast on the gyro
                } else {
                    if z.sun_ok && first { self.k.vector(&z.sun, &self.gd.sun_eci, p.mekf_sig_sun); }
                    if self.clean && first {
                        if let Some(br) = self.bref { self.k.vector(&z.b, &br, p.mekf_sig_mag); }
                    }
                    if z.es_ok && first && self.have_r {
                        let sg = if p.es_noise > 1e-3 { p.es_noise } else { 1e-3 };
                        self.k.vector(&z.nadir, &scale3(&self.r, -1.0), 2.0*sg);
                    }
                }
            }
        }
        {
            let a = dt/(p.rate_lpf_s + dt);
            let wr = if self.ad_ok { sub3(&z.w, &self.k.b) } else { z.w };
            if p.rate_lpf_s <= 0.0 { self.w_est = wr; }
            else { for i in 0..3 { self.w_est[i] += a*(wr[i] - self.w_est[i]); } }
        }

        // 3 mode manager
        while self.sched_i < p.n_sched as usize && self.t >= p.sched_t[self.sched_i] {
            self.enter(p.sched_mode[self.sched_i]);
            self.sched_i += 1;
        }
        if (self.mode == DETUMBLE || self.mode == DETUMBLE_RCS) && p.auto_next != MODE_NONE {
            if norm3(&z.w) < p.detumble_exit { self.hold += dt; } else { self.hold = 0.0; }
            if self.hold >= p.detumble_hold_s { self.enter(p.auto_next); }
        }
        if self.mode == SPINUP || self.mode == SUN_SPIN { self.spin_guards(dt); }
        if self.mode == SPINUP || self.mode == SUN_SPIN || self.mode == SUN_ACQ_ROTOR {
            if z.sun_ok { self.s_prop = Some(z.sun); }
            else if let Some(sp) = self.s_prop {
                let a = dcm(&fromrotvec(&scale3(&self.w_est, dt)));
                self.s_prop = Some(unit(&mat3_vec(&a, &sp)));
            }
        }
        if self.mode == SUN_ACQ_ROTOR && p.auto_next != MODE_NONE {
            let ok = z.sun_ok && acos(clamp(dot(&z.sun, &p.sun_axis), -1.0, 1.0)) < p.sa_done_deg*D2R && self.ad_ok;
            if ok { self.acq_hold += dt; } else { self.acq_hold = 0.0; }
            if self.acq_hold >= p.sa_done_hold_s { self.enter(p.auto_next); }
        }

        // 4 guidance / control / commands
        let mut m_body = self.m_hold;
        let mut cmd_r = [0.0; NR];
        let mut cmd_g = [0.0; NG];
        let mut duty = [0.0; NC];
        match self.mode {
            DETUMBLE => {
                if phase < p.mtq_meas + dt/2.0 {
                    m_body = [0.0; 3];
                    if first { self.bsum = [0.0; 3]; self.bsum_raw = [0.0; 3]; self.bn = 0; }
                    self.bsum = add3(&self.bsum, &unit(&z.b));
                    self.bsum_raw = add3(&self.bsum_raw, &z.b);
                    self.bn += 1;
                    if fabs(phase - p.mtq_meas) < dt/2.0 {
                        let b = unit(&scale3(&self.bsum, 1.0/self.bn as f64));
                        let mut law = p.bdot_law;
                        if law == 0 && p.has_gyro == 0 { law = 1; }
                        if law == 0 {
                            let bd = scale3(&cross(&z.w, &b), -1.0);
                            m_body = ctl::bdot(&sub3(&b, &bd), &b, 1.0, norm3(&z.b), p.bdot_k, p.m_max);
                        } else if law == 1 {
                            if let Some(b1) = self.b1 { m_body = ctl::bdot(&b1, &b, p.mtq_period, norm3(&z.b), p.bdot_k, p.m_max); }
                        } else if law == 2 {
                            if let Some(b1) = self.b1 {
                                for i in 0..3 {
                                    let bd = (b[i] - b1[i])/p.mtq_period;
                                    m_body[i] = if fabs(bd) > 1e-4 { -p.m_max*sign(bd) } else { 0.0 };
                                }
                            }
                        } else {
                            let bav = scale3(&self.bsum_raw, 1.0/self.bn as f64);
                            if let Some(b1r) = self.b1raw {
                                let mut bd = [0.0; 3];
                                for i in 0..3 { bd[i] = (bav[i] - b1r[i])/p.mtq_period; }
                                m_body = ctl::gen_bdot(&bav, &bd, &[0.0; 3], p.ss_k_l1);
                                let mut mx = maxabs3(&m_body);
                                if mx < 1e-30 { mx = 1e-30; }
                                m_body = scale3(&m_body, if p.m_max/mx < 1.0 { p.m_max/mx } else { 1.0 });
                            }
                            self.b1raw = Some(bav);
                        }
                        if !is_zero3(&m_body) { m_body = ctl::sat_dipole(&sub3(&m_body, &p.m_res_est), p.m_max); }
                        self.b1 = Some(b);
                    }
                }
                if nr > 0 { self.idle_rotors(&mut cmd_r, true); }
            }
            DETUMBLE_RCS => {
                let tc = p.rcsd_period_s;
                if p.nc > 0 && (self.rcs_left.is_none() || fmod(self.t + 1e-9, tc) < dt - 1e-9) {
                    let mut left = [0.0; NC];
                    if norm3(&self.w_est) > p.rcsd_deadband_deg_s*D2R {
                        self.tau_req = mat3_vec(&p.J, &scale3(&self.w_est, -1.0/p.rcsd_T_damp_s));
                        let (dc, _) = alloc::rcs_duty(&self.tau_req, &p, tc);
                        for i in 0..NC { left[i] = dc[i]*tc; }
                    }
                    self.rcs_left = Some(left);
                }
                if p.nc > 0 {
                    let left = self.rcs_left.as_mut().unwrap();
                    for i in 0..NC {
                        duty[i] = left[i]/dt;
                        if duty[i] > 1.0 { duty[i] = 1.0; }
                        left[i] -= dt;
                        if left[i] < 0.0 { left[i] = 0.0; }
                    }
                }
                m_body = [0.0; 3];
                if nr > 0 { self.idle_rotors(&mut cmd_r, true); }
            }
            NADIR_MTQ | SUN_MTQ => {
                if first { m_body = [0.0; 3]; }
                else if fabs(phase - p.mtq_meas) < dt/2.0 && self.ad_ok && self.have_r {
                    let rf = ctl::guidance(guid_kind_of(self.mode), &self.r, &self.v, self.t, &self.gd);
                    self.q_ref = rf.q; self.w_ref = rf.w;
                    self.mtq_law();
                    let md = ctl::torque2dipole(&self.tau_req, &z.b, p.m_max);
                    m_body = ctl::sat_dipole(&sub3(&md, &p.m_res_est), p.m_max);
                } else if phase < p.mtq_meas { m_body = [0.0; 3]; }
            }
            NADIR_FINE | TARGET_FINE | SLEW_FINE | SUN_FINE | SUN_ACQ_ROTOR => {
                let acq = self.mode == SUN_ACQ_ROTOR;
                let ctl_ok = self.ad_ok || acq;
                let a = alloc::rotor_axes(&p, &z.delta);
                let mut hdev = [0.0; 3];
                for i in 0..nr { hdev[0] += a[0][i]*z.h[i]; hdev[1] += a[1][i]*z.h[i]; hdev[2] += a[2][i]*z.h[i]; }
                // FDIR on fixed rotors
                if nr > 0 {
                    if let Some(hp) = self.h_prev {
                        for i in 0..nr {
                            let tmax = p.rot_tmax[i];
                            let meas = (z.h[i] - hp[i])/dt;
                            let expect = clamp(self.cmd_r_prev[i], -0.8*tmax, 0.8*tmax);
                            let bad = fabs(meas - expect) > 0.5*tmax && p.rot_gi[i] == 0 && fabs(expect) > 0.2*tmax && fabs(z.h[i]) < 0.9*p.rot_hmax[i];
                            self.fd_count[i] = if bad { self.fd_count[i] + dt } else { 0.0 };
                            if self.fd_count[i] > p.fdir_s && !self.rot_failed[i] { self.rot_failed[i] = true; self.faults |= 1u16 << i; }
                        }
                    }
                }
                // control law at the control rate
                if acq && self.t - self.last_ctrl >= p.rw_dt - 1e-9 { self.sun_acq_law(&hdev); self.last_ctrl = self.t; }
                else if !acq && self.ad_ok && self.have_r && self.t - self.last_ctrl >= p.rw_dt - 1e-9 {
                    let rf = ctl::guidance(guid_kind_of(self.mode), &self.r, &self.v, self.t, &self.gd);
                    self.q_ref = rf.q; self.w_ref = rf.w;
                    self.capturing = self.capture_law(&hdev);
                    if self.capturing { self.i_q = [0.0; 3]; }
                    else {
                        self.tau_req = ctl::control_law(&self.k.q, &self.w_est, &self.q_ref, &self.w_ref, &mut self.i_q, p.rw_dt, &self.g_rw, &p.J, &hdev, &rf.wd);
                    }
                    self.last_ctrl = self.t;
                }
                // momentum management
                let dh = sub3(&hdev, &self.h_t);
                if self.has_rcs_dump {
                    if norm3(&dh) > self.dump_hi { self.rcs_dumping = true; }
                    else if norm3(&dh) < self.dump_lo { self.rcs_dumping = false; }
                }
                if first { m_body = [0.0; 3]; }
                else if fabs(phase - p.mtq_meas) < dt/2.0 {
                    let mut mm = [0.0; 3];
                    if !self.has_rcs_dump { mm = alloc::dump(&hdev, &self.h_t, &z.b, p.dump_k, p.m_max); }
                    if p.alloc == 1 && ctl_ok { mm = add3(&mm, &ctl::torque2dipole(&self.tau_req, &z.b, p.m_max)); }
                    if self.ad_ok {
                        let mut anyf = false;
                        let mut fx = [0usize; NR];
                        let mut nf = 0;
                        for i in 0..nr { if self.rot_failed[i] { anyf = true; } else if p.rot_gi[i] == 0 { fx[nf] = i; nf += 1; } }
                        if anyf {
                            let mut un = self.tau_req;
                            if nf > 0 {
                                let mut af = [[0.0; 8]; 3];
                                for i in 0..nf { for k in 0..3 { af[k][i] = a[k][fx[i]]; } }
                                let pi = pinv_rows(&af, nf);
                                let mut y = [0.0; 8];
                                for i in 0..nf { y[i] = pi[i][0]*self.tau_req[0] + pi[i][1]*self.tau_req[1] + pi[i][2]*self.tau_req[2]; }
                                for k in 0..3 { for i in 0..nf { un[k] -= af[k][i]*y[i]; } }
                            }
                            mm = add3(&mm, &ctl::torque2dipole(&un, &z.b, p.m_max));
                        }
                    }
                    m_body = ctl::sat_dipole(&sub3(&mm, &p.m_res_est), p.m_max);
                    self.b_dump = z.b;
                } else if phase < p.mtq_meas { m_body = [0.0; 3]; }
                // thrusters: assist and dumping
                let mut tau_rcs = [0.0; 3];
                if p.nc > 0 && ctl_ok {
                    let mut req = [0.0; 3];
                    if p.rcs_assist != 0 {
                        for i in 0..3 {
                            let ex = fabs(self.tau_req[i]) - p.rcs_assist_frac*self.cap[i];
                            req[i] = sign(self.tau_req[i])*(if ex > 0.0 { ex } else { 0.0 });
                            if fabs(hdev[i]) > p.rcs_assist_frac*self.hcap[i] && sign(-self.tau_req[i]) == sign(hdev[i]) { req[i] = self.tau_req[i]; }
                        }
                    }
                    if self.rcs_dumping { for i in 0..3 { req[i] -= p.rcs_dump_k*dh[i]; } }
                    let (d, t) = alloc::rcs_duty(&req, &p, dt);
                    duty = d;
                    tau_rcs = t;
                }
                // momentum devices deliver the rest (coil and thruster torques fed forward)
                let mut tau_coil = [0.0; 3];
                if !is_zero3(&m_body) { tau_coil = cross(&add3(&m_body, &p.m_res_est), &self.b_dump); }
                if ctl_ok && (self.have_r || acq) && nr > 0 {
                    let mut tr = [0.0; 3];
                    for i in 0..3 { tr[i] = self.tau_req[i] - tau_coil[i] - tau_rcs[i]; }
                    self.allocate(&tr, &a, &mut cmd_r, &mut cmd_g);
                }
                let mut hp = [0.0; NR];
                hp[..nr].copy_from_slice(&z.h[..nr]);
                self.cmd_r_prev[..nr].copy_from_slice(&cmd_r[..nr]);
                if let Some(old) = self.h_prev { hp[nr..].copy_from_slice(&old[nr..]); }
                self.h_prev = Some(hp);
            }
            SPINUP | SUN_SPIN => {
                if phase < p.mtq_meas + dt/2.0 {
                    m_body = [0.0; 3];
                    if first { self.bsum_raw = [0.0; 3]; self.bn = 0; }
                    self.bsum_raw = add3(&self.bsum_raw, &z.b);
                    self.bn += 1;
                    if fabs(phase - p.mtq_meas) < dt/2.0 {
                        let bav = scale3(&self.bsum_raw, 1.0/self.bn as f64);
                        let m0 = if self.mode == SPINUP {
                            let mut bd = [0.0; 3];
                            if let Some(b1r) = self.b1raw { for i in 0..3 { bd[i] = (bav[i] - b1r[i])/p.mtq_period; } }
                            ctl::gen_bdot(&bav, &bd, &[0.0, 0.0, self.sigma*p.ss_spin_dps*D2R], p.ss_k_l1)
                        } else {
                            let ecl = (!z.sun_ok && p.ss_eclipse == 1) || self.s_prop.is_none();
                            let s = self.s_prop.unwrap_or([0.0; 3]);
                            ctl::sun_spin(&bav, &self.w_est, &s, ecl, &p.J, p.ss_spin_dps, p.ss_k1, p.ss_k2, p.ss_rz_floor)
                        };
                        self.b1raw = Some(bav);
                        if !is_zero3(&m0) { m_body = ctl::sat_dipole(&sub3(&m0, &p.m_res_est), p.m_max); }
                    }
                }
                if nr > 0 { self.idle_rotors(&mut cmd_r, false); }
            }
            _ => {}
        }
        self.m_hold = m_body;
        self.clean = is_zero3(&m_body);
        drv::write(hal, &p, &m_body, &cmd_r, &cmd_g, &duty);
        self.out_m = m_body;
        self.out_r = cmd_r;
        self.out_g = cmd_g;
        self.out_duty = duty;
        0
    }

    /// TC 0x01: set controller state (tc[1]).
    pub fn command(&mut self, tc: &[u8]) -> i32 {
        if !self.ready || tc.len() < 2 { return -1; }
        if tc[0] == 0x01 && tc[1] < MODE_COUNT { self.enter(tc[1]); return 0; }
        -2
    }

    pub fn peek(&self) -> Option<State> {
        if !self.ready { return None; }
        let q = self.k.q;
        let a = alloc::rotor_axes(&self.p, &self.z.delta);
        let mut h = [0.0f64; 3];
        for i in 0..self.p.nr as usize { h[0] += a[0][i]*self.z.h[i]; h[1] += a[1][i]*self.z.h[i]; h[2] += a[2][i]*self.z.h[i]; }
        let mut o = State { t_ns: (self.t*1e9 + 0.5) as u64, q_est: [q[3] as f32, q[0] as f32, q[1] as f32, q[2] as f32], ..State::default() };
        for i in 0..3 {
            o.w_est[i] = self.w_est[i] as f32; o.b_est[i] = self.k.b[i] as f32;
            o.m_cmd[i] = self.out_m[i] as f32; o.h_int[i] = h[i] as f32;
        }
        for i in 0..8 { o.u_cmd[i] = self.out_r[i] as f32; }
        o.mode = self.mode as u16;
        o.faults = self.faults;
        Some(o)
    }

    /// t, mode, ad_ok, q[4], b[3], w_est[3], q_ref[4], tau_req[3], m[3], cmd_r[8], cmd_g[4], duty[6] (= adcs_fsw_debug).
    pub fn debug(&self, out: &mut [f64]) -> usize {
        let mut v = [0.0f64; 48];
        let mut k = 0;
        let mut put = |x: f64| { v[k] = x; k += 1; };
        put(self.t); put(self.mode as f64); put(if self.ad_ok { 1.0 } else { 0.0 });
        for &x in &self.k.q { put(x); }
        for &x in &self.k.b { put(x); }
        for &x in &self.w_est { put(x); }
        for &x in &self.q_ref { put(x); }
        for &x in &self.tau_req { put(x); }
        for &x in &self.out_m { put(x); }
        for &x in &self.out_r { put(x); }
        for &x in &self.out_g { put(x); }
        for &x in &self.out_duty { put(x); }
        let n = out.len().min(k);
        out[..n].copy_from_slice(&v[..n]);
        k
    }

    pub fn mode(&self) -> u8 { self.mode }
    pub fn time(&self) -> f64 { self.t }
}
