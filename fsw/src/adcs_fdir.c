/* adcs_fdir.c -- fault detection, isolation and recovery: sensor health (a magnetometer reading
 * outside the model field, a silent sensor), safe mode, and the rotor FDIR (instantaneous and
 * windowed, 07_allocation.md). Group fdir (design/groups.toml); twin: asils.fsw.fdir; Rust: fdir.rs.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_fsw_int.h"

#define FDIR_WIN_BAD 2      /* judged momentum windows in a row a rotor misses before it is isolated */

/* Sensor health (fsw/pseudocode: safe mode). A field reading outside [MAG_LO, MAG_HI] x the model
 * field is a stuck or dead magnetometer, not a reading. The last good field is held through a
 * short dropout; the coils act on it for at most MAG_HOLD_CYCLES coil cycles (adcs_fsw_int.h). A magnetometer or a
 * gyro silent for SAFE_STALE_S puts the spacecraft in magnetorquer detumble until it answers. */
#define MAG_LO 0.25
#define MAG_HI 4.0
#define SAFE_STALE_S 60.0

/* sensor health, after the drivers read: a field outside [MAG_LO, MAG_HI] x the model is no
 * reading; the last good field is held; the age of each sensor's last good sample */
void adcs_fdir_sensors(fsw_t *s, double dt)
{
    const adcs_params_t *p = &s->p;
    adcs_meas_t *z = &s->z;
    if (z->mag_ok && s->bref_ok) {
        adcs_real bn = adcs_norm3(z->B), rn = adcs_norm3(s->Bref);
        if (!(bn > MAG_LO*rn && bn < MAG_HI*rn)) z->mag_ok = 0;
    }
    if (z->mag_ok) { adcs_copy3(z->B, s->B_good); s->mag_age = 0; s->mag_seen = 1; }
    else { adcs_copy3(s->B_good, z->B); s->mag_age += dt; }
    if (!p->has_gyro || z->gyro_ok) s->gyro_age = 0; else s->gyro_age += dt;
    if (!z->gyro_ok && !p->has_gyro) adcs_zero3(z->w);
}

/* safe mode: a sensor silent too long holds the spacecraft in magnetorquer detumble */
void adcs_fdir_safe(fsw_t *s)
{
    uint16_t f = 0;
    if (s->mag_seen && s->mag_age > SAFE_STALE_S) f |= FAULT_MAG_STALE;
    if (s->gyro_age > SAFE_STALE_S) f |= FAULT_GYRO_STALE;
    s->faults = (uint16_t)((s->faults & ~(FAULT_MAG_STALE | FAULT_GYRO_STALE)) | f);
    if (f && s->mode != ADCS_MODE_DETUMBLE) adcs_modes_enter(s, ADCS_MODE_DETUMBLE);
}

/* rotor FDIR in the momentum-device states: a fixed rotor that does not follow its command is
 * isolated (instantaneous: the momentum rate against the command; windowed: the momentum change
 * over fdir_win_s against the change commanded, fluid loops only) */
void adcs_fdir_rotors(fsw_t *s, double dt)
{
    const adcs_params_t *p = &s->p;
    int i, nr = p->nr;
    if (nr > 0 && s->h_prev_ok) {
        for (i = 0; i < nr; i++) {
            adcs_real tmax = p->rot_tmax[i], meas = (s->z.h[i] - s->h_prev[i])/dt;
            adcs_real expect = adcs_clamp(s->cmd_r_prev[i], -0.8*tmax, 0.8*tmax);
            int bad = fabs(meas - expect) > 0.5*tmax && p->rot_gi[i] == 0 && fabs(expect) > 0.2*tmax && fabs(s->z.h[i]) < 0.9*p->rot_hmax[i];
            s->fd_count[i] = bad ? s->fd_count[i] + dt : 0;
            if (s->fd_count[i] > p->fdir_s && !s->rot_failed[i]) { s->rot_failed[i] = 1; s->faults |= (uint16_t)(1u << i); }
        }
        /* windowed: the momentum each fluid loop was commanded to change over fdir_win_s against the
           change measured; catches a loop that does not follow the small commands of fine pointing.
           Fluid loops only: their driver closes a momentum loop, so a healthy one tracks the
           commanded change; a wheel's uncompensated friction drifts it off over a window */
        if (!s->fw_on || s->t - s->fw_last > 1.5*dt) {
            for (i = 0; i < nr; i++) { s->fw_E[i] = 0; s->fw_h0[i] = s->z.h[i]; }
            s->fw_t0 = s->t; s->fw_on = 1;
        } else {
            for (i = 0; i < nr; i++) s->fw_E[i] += adcs_clamp(s->cmd_r_prev[i], -0.8*p->rot_tmax[i], 0.8*p->rot_tmax[i])*dt;
            if (s->t - s->fw_t0 >= p->fdir_win_s - 1e-9) {
                for (i = 0; i < nr; i++) {
                    adcs_real hmax = p->rot_hmax[i], E = s->fw_E[i], M = s->z.h[i] - s->fw_h0[i];
                    if (p->rot_kind[i] != 1 || p->rot_gi[i] != 0 || s->rot_failed[i] || fabs(E) <= p->fdir_h_frac*hmax
                        || fabs(s->z.h[i]) >= 0.9*hmax || fabs(s->fw_h0[i]) >= 0.9*hmax) continue;
                    s->fw_bad[i] = fabs(M - E) > 0.5*fabs(E) ? s->fw_bad[i] + 1 : 0;
                    if (s->fw_bad[i] >= FDIR_WIN_BAD) { s->rot_failed[i] = 1; s->faults |= (uint16_t)(1u << i); }
                }
                for (i = 0; i < nr; i++) { s->fw_E[i] = 0; s->fw_h0[i] = s->z.h[i]; }
                s->fw_t0 = s->t;
            }
        }
        s->fw_last = s->t;
    }
}
