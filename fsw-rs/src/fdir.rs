//! Fault detection, isolation and recovery: sensor health (a magnetometer reading outside the model
//! field, a silent sensor), safe mode, and the rotor FDIR (instantaneous and windowed,
//! 07_allocation.md); twin of adcs_fdir.c. Group fdir (design/groups.toml); MATLAB twin:
//! asils.fsw.fdir. A child of fsw: it works on the flight software's own state.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use super::*;

/// Judged momentum windows in a row a rotor misses before it is isolated.
const FDIR_WIN_BAD: u32 = 2;
/// Sensor health (= adcs_fdir.c): a field reading outside [MAG_LO, MAG_HI] x the model field is not a
/// reading; the held field drives the coils for at most MAG_HOLD_CYCLES coil cycles (fsw.rs); a magnetometer
/// or gyro silent for SAFE_STALE_S holds the spacecraft in magnetorquer detumble.
const MAG_LO: f64 = 0.25;
const MAG_HI: f64 = 4.0;
const SAFE_STALE_S: f64 = 60.0;

impl Fsw {
    /// Sensor health, after the drivers read: a field outside [MAG_LO, MAG_HI] x the model is no
    /// reading; the last good field is held; the age of each sensor's last good sample.
    pub(super) fn fdir_sensors(&mut self, z: &mut Meas, dt: f64) {
        let p = self.p;
        if z.mag_ok {
            if let Some(br) = self.bref {
                let (bn, rn) = (norm3(&z.b), norm3(&br));
                if !(bn > MAG_LO*rn && bn < MAG_HI*rn) { z.mag_ok = false; }
            }
        }
        if z.mag_ok { self.b_good = z.b; self.mag_age = 0.0; self.mag_seen = true; }
        else { z.b = self.b_good; self.mag_age += dt; }
        if p.has_gyro == 0 || z.gyro_ok { self.gyro_age = 0.0; } else { self.gyro_age += dt; }
        if !z.gyro_ok && p.has_gyro == 0 { z.w = [0.0; 3]; }
    }

    /// Safe mode: a sensor silent too long holds the spacecraft in magnetorquer detumble.
    pub(super) fn fdir_safe(&mut self) {
        let mut f = 0u16;
        if self.mag_seen && self.mag_age > SAFE_STALE_S { f |= FAULT_MAG_STALE; }
        if self.gyro_age > SAFE_STALE_S { f |= FAULT_GYRO_STALE; }
        self.faults = (self.faults & !(FAULT_MAG_STALE | FAULT_GYRO_STALE)) | f;
        if f != 0 && self.mode != DETUMBLE { self.enter(DETUMBLE); }
    }

    /// Rotor FDIR in the momentum-device states: a fixed rotor that does not follow its command is
    /// isolated (instantaneous: the momentum rate against the command; windowed: the momentum change
    /// over fdir_win_s against the change commanded, fluid loops only).
    pub(super) fn fdir_rotors(&mut self, z: &Meas, dt: f64) {
        let p = self.p;
        let nr = p.nr as usize;
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
                // windowed: the momentum each fluid loop was commanded to change over fdir_win_s against the
                // change measured; catches a loop that does not follow the small commands of fine pointing.
                // Fluid loops only: their driver closes a momentum loop, so a healthy one tracks the
                // commanded change; a wheel's uncompensated friction drifts it off over a window
                if !self.fw_on || self.t - self.fw_last > 1.5*dt {
                    for i in 0..nr { self.fw_e[i] = 0.0; self.fw_h0[i] = z.h[i]; }
                    self.fw_t0 = self.t; self.fw_on = true;
                } else {
                    for i in 0..nr { self.fw_e[i] += clamp(self.cmd_r_prev[i], -0.8*p.rot_tmax[i], 0.8*p.rot_tmax[i])*dt; }
                    if self.t - self.fw_t0 >= p.fdir_win_s - 1e-9 {
                        for i in 0..nr {
                            let (hmax, e, m) = (p.rot_hmax[i], self.fw_e[i], z.h[i] - self.fw_h0[i]);
                            if p.rot_kind[i] != 1 || p.rot_gi[i] != 0 || self.rot_failed[i] || fabs(e) <= p.fdir_h_frac*hmax
                                || fabs(z.h[i]) >= 0.9*hmax || fabs(self.fw_h0[i]) >= 0.9*hmax { continue; }
                            self.fw_bad[i] = if fabs(m - e) > 0.5*fabs(e) { self.fw_bad[i] + 1 } else { 0 };
                            if self.fw_bad[i] >= FDIR_WIN_BAD { self.rot_failed[i] = true; self.faults |= 1u16 << i; }
                        }
                        for i in 0..nr { self.fw_e[i] = 0.0; self.fw_h0[i] = z.h[i]; }
                        self.fw_t0 = self.t;
                    }
                }
                self.fw_last = self.t;
            }
        }
    }
}
