//! The mode manager (08_mode_manager.md): which controller state the spacecraft is in, what it may
//! fly, and when it moves on (commanded changes, the detumble exit, the spin guards, the Sun
//! acquisition's end); twin of adcs_modes.c. Group gdn (design/groups.toml); MATLAB twin:
//! asils.fsw.mode_manager. A child of fsw: it works on the flight software's own state.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::*;
use crate::alg::modes::{self as alg, ModeParams, Modes};

/// Can the fitted hardware fly this state? (= feasible in adcs_fsw.c)
/// Written from the design: modes::modes_feasible (src/alg).
pub(super) fn feasible(p: &Params, m: u8) -> bool { alg::modes_feasible(p.nr as i64, p.nc as i64, m as i64) }

impl Fsw {
    /// The part of the state the mode manager reads and writes, as the design's record.
    pub(super) fn modes_state(&self) -> Modes {
        Modes {
            mode: self.mode as i64, t: self.t, t_mode: self.t_mode, hold: self.hold, ho: self.ho, ho_t: self.ho_t, ad_ok: self.ad_ok,
            n_rej: self.n_rej as i64, i_q: self.i_q, sz_sum: self.sz_sum, sz_n: self.sz_n as i64, sz_t0: self.sz_t0, sigma: self.sigma,
            s_prop: self.s_prop.unwrap_or([0.0; 3]), s_prop_ok: self.s_prop.is_some(), acq_hold: self.acq_hold,
            sched_i: self.sched_i as i64, faults: self.faults as i64, w_est: self.w_est, mag_seen: self.mag_seen,
            mag_age: self.mag_age, gyro_age: self.gyro_age,
        }
    }

    pub(super) fn set_modes_state(&mut self, s: Modes) {
        self.mode = s.mode as u8; self.t = s.t; self.t_mode = s.t_mode; self.hold = s.hold; self.ho = s.ho; self.ho_t = s.ho_t;
        self.ad_ok = s.ad_ok; self.n_rej = s.n_rej as f64; self.i_q = s.i_q; self.sz_sum = s.sz_sum; self.sz_n = s.sz_n as u32;
        self.sz_t0 = s.sz_t0; self.sigma = s.sigma; self.s_prop = s.s_prop_ok.then_some(s.s_prop); self.acq_hold = s.acq_hold;
        self.sched_i = s.sched_i as usize; self.faults = s.faults as u16; self.w_est = s.w_est; self.mag_seen = s.mag_seen;
        self.mag_age = s.mag_age; self.gyro_age = s.gyro_age;
    }

    fn mode_params(&self) -> ModeParams {
        let p = &self.p;
        ModeParams {
            auto_next: p.auto_next as i64, n_sched: p.n_sched as i64, sched_t: p.sched_t, sched_mode: ints(&p.sched_mode),
            detumble_exit: p.detumble_exit, detumble_hold_s: p.detumble_hold_s, ss_law: p.ss_law as i64,
            ss_omega_max_dps: p.ss_omega_max_dps, ss_spin_dps: p.ss_spin_dps, ss_z_in_dps: p.ss_z_in_dps,
            ss_perp_in_dps: p.ss_perp_in_dps, ss_t_check_s: p.ss_t_check_s, ss_sun_min: p.ss_sun_min, ss_dwell_in_s: p.ss_dwell_in_s,
            ss_omega_exit_dps: p.ss_omega_exit_dps, ss_perp_out_dps: p.ss_perp_out_dps, ss_dwell_out_s: p.ss_dwell_out_s,
            sun_axis: p.sun_axis, sa_done_deg: p.sa_done_deg, sa_done_hold_s: p.sa_done_hold_s,
        }
    }

    /// Written from the design: modes::modes_enter (src/alg).
    pub(super) fn enter(&mut self, mode: u8) { self.set_modes_state(alg::modes_enter(self.modes_state(), mode as i64)); }

    /// Commanded changes: the schedule's entries whose time has come.
    /// Written from the design: modes::modes_schedule (src/alg).
    pub(super) fn modes_schedule(&mut self) { self.set_modes_state(alg::modes_schedule(self.modes_state(), self.mode_params())); }

    /// The transitions of one tick (after safe mode, fdir_safe), the spin guards among them.
    /// Written from the design: modes::modes_step (src/alg).
    pub(super) fn modes_step(&mut self, z: &Meas, dt: f64) {
        self.set_modes_state(alg::modes_step(self.modes_state(), self.mode_params(), z.w, z.sun_ok, z.sun, dt));
    }
}
