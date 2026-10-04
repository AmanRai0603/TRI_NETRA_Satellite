//! The mode manager (08_mode_manager.md): which controller state the spacecraft is in, what it may
//! fly, and when it moves on (commanded changes, the detumble exit, the spin guards, the Sun
//! acquisition's end); twin of adcs_modes.c. Group gdn (design/groups.toml); MATLAB twin:
//! asils.fsw.mode_manager. A child of fsw: it works on the flight software's own state.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::*;

/// Can the fitted hardware fly this state? (= feasible in adcs_fsw.c)
pub(super) fn feasible(p: &Params, m: u8) -> bool {
    match m {
        NADIR_FINE | TARGET_FINE | SLEW_FINE | SUN_FINE => p.nr > 0 || p.nc > 0,
        SUN_ACQ_ROTOR => p.nr > 0,
        DETUMBLE_RCS => p.nc > 0,
        _ => m < MODE_COUNT,
    }
}

impl Fsw {
    pub(super) fn enter(&mut self, mode: u8) {
        if mode == SPINUP { self.sz_sum = 0.0; self.sz_n = 0; self.sz_t0 = self.t; }
        self.mode = mode;
        self.t_mode = self.t;
        self.hold = 0.0;
        self.ho = false;
        self.ho_t = 0.0;
        // the states that skip estimation freeze the attitude: the next pointing state re-initialises it
        if mode == DETUMBLE || mode == DETUMBLE_RCS || mode == SPINUP || mode == SUN_SPIN { self.ad_ok = false; self.n_rej = 0.0; }
        self.i_q = [0.0; 3];
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

    /// Commanded changes: the schedule's entries whose time has come.
    pub(super) fn modes_schedule(&mut self) {
        let p = self.p;
        while self.sched_i < p.n_sched as usize && self.t >= p.sched_t[self.sched_i] {
            self.enter(p.sched_mode[self.sched_i]);
            self.sched_i += 1;
        }
    }

    /// The transitions of one tick (after safe mode, fdir_safe).
    pub(super) fn modes_step(&mut self, z: &Meas, dt: f64) {
        let p = self.p;
        if (self.mode == DETUMBLE || self.mode == DETUMBLE_RCS) && p.auto_next != MODE_NONE
            && self.faults & (FAULT_MAG_STALE | FAULT_GYRO_STALE) == 0 {
            if norm3(&z.w) < p.detumble_exit { self.hold += dt; } else { self.hold = 0.0; }
            if self.hold >= p.detumble_hold_s { self.enter(p.auto_next); }
        }
        if (self.mode == SPINUP || self.mode == SUN_SPIN) && p.ss_law != 2 { self.spin_guards(dt); }
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
    }
}
