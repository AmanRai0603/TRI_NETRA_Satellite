//! Fault detection, isolation and recovery: sensor health (a magnetometer reading outside the model
//! field, a silent sensor), safe mode, and the rotor FDIR (instantaneous and windowed,
//! 07_allocation.md); twin of adcs_fdir.c. Group fdir (design/groups.toml); MATLAB twin:
//! asils.fsw.fdir. A child of fsw: it works on the flight software's own state.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::*;
use crate::alg::modes::{self as alg, RotorFdir, RotorParams};

impl Fsw {
    /// Sensor health, after the drivers read: a field outside [0.25, 4] x the model is no
    /// reading; the last good field is held; the age of each sensor's last good sample.
    /// Written from the design: modes::fdir_sensors (src/alg).
    pub(super) fn fdir_sensors(&mut self, z: &mut Meas, dt: f64) {
        (z.mag_ok, z.b, z.w, self.b_good, self.mag_age, self.mag_seen, self.gyro_age) =
            alg::fdir_sensors(z.mag_ok, z.b, self.bref.is_some(), self.bref.unwrap_or([0.0; 3]), self.b_good, self.mag_age,
                              self.mag_seen, self.p.has_gyro != 0, z.gyro_ok, z.w, self.gyro_age, dt);
    }

    /// Safe mode: a sensor silent too long holds the spacecraft in magnetorquer detumble.
    /// Written from the design: modes::fdir_safe (src/alg).
    pub(super) fn fdir_safe(&mut self) { self.set_modes_state(alg::fdir_safe(self.modes_state())); }

    /// Rotor FDIR in the momentum-device states: a fixed rotor that does not follow its command is
    /// isolated (instantaneous: the momentum rate against the command; windowed: the momentum change
    /// over fdir_win_s against the change commanded, fluid loops only).
    /// Written from the design: modes::fdir_rotors (src/alg).
    pub(super) fn fdir_rotors(&mut self, z: &Meas, dt: f64) {
        let p = &self.p;
        let mut fw_bad = [0i64; NR];
        for i in 0..NR { fw_bad[i] = self.fw_bad[i] as i64; }
        let st = RotorFdir {
            h_prev: self.h_prev.unwrap_or([0.0; NR]), h_prev_ok: self.h_prev.is_some(), cmd_r_prev: self.cmd_r_prev,
            fd_count: self.fd_count, rot_failed: self.rot_failed, faults: self.faults as i64, fw_e: self.fw_e, fw_h0: self.fw_h0,
            fw_t0: self.fw_t0, fw_last: self.fw_last, fw_on: self.fw_on, fw_bad,
        };
        let rp = RotorParams {
            nr: p.nr as i64, rot_tmax: p.rot_tmax, rot_hmax: p.rot_hmax, rot_gi: ints(&p.rot_gi), rot_kind: ints(&p.rot_kind),
            fdir_s: p.fdir_s, fdir_win_s: p.fdir_win_s, fdir_h_frac: p.fdir_h_frac,
        };
        let s = alg::fdir_rotors(st, rp, self.t, dt, z.h);
        self.h_prev = s.h_prev_ok.then_some(s.h_prev);
        self.cmd_r_prev = s.cmd_r_prev; self.fd_count = s.fd_count; self.rot_failed = s.rot_failed; self.faults = s.faults as u16;
        self.fw_e = s.fw_e; self.fw_h0 = s.fw_h0; self.fw_t0 = s.fw_t0; self.fw_last = s.fw_last; self.fw_on = s.fw_on;
        for i in 0..NR { self.fw_bad[i] = s.fw_bad[i] as u32; }
    }
}
