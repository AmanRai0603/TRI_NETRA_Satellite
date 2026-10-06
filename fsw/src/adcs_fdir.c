/* adcs_fdir.c -- fault detection, isolation and recovery: sensor health (a magnetometer reading
 * outside the model field, a silent sensor), safe mode, and the rotor FDIR (instantaneous and
 * windowed, 07_allocation.md). Group fdir (design/groups.toml); twin: asils.fsw.fdir; Rust: fdir.rs.
 * Every function delegates to its translation of 08_mode_manager.pc (fsw/alg/src/modes.c): a field
 * outside [0.25, 4] x the model is no reading, the last good field is held (the coils act on it for at
 * most MAG_HOLD_CYCLES coil cycles, adcs_fsw_int.h), a magnetometer or a gyro silent 60 s holds the
 * spacecraft in magnetorquer detumble, and a fixed rotor that misses its command 2 windows in a row is
 * isolated.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_fsw_int.h"
#include "adcs_alg_glue.h"

/* sensor health, after the drivers read: a field outside [MAG_LO, MAG_HI] x the model is no
 * reading; the last good field is held; the age of each sensor's last good sample */
/* written from the design: modes::fdir_sensors (fsw/alg) */
void adcs_fdir_sensors(fsw_t *s, double dt)
{
    adcs_meas_t *z = &s->z;
    modes_fdir_sensors_out o = modes_fdir_sensors(z->mag_ok != 0, gl_v3(z->B), s->bref_ok != 0, gl_v3(s->Bref), gl_v3(s->B_good),
                                                  s->mag_age, s->mag_seen != 0, s->p.has_gyro != 0, z->gyro_ok != 0, gl_v3(z->w),
                                                  s->gyro_age, dt);
    z->mag_ok = o.mag_ok; gl_o3(o.b, z->B); gl_o3(o.w, z->w); gl_o3(o.b_good, s->B_good);
    s->mag_age = o.mag_age; s->mag_seen = o.mag_seen; s->gyro_age = o.gyro_age;
}

/* safe mode: a sensor silent too long holds the spacecraft in magnetorquer detumble */
/* written from the design: modes::fdir_safe (fsw/alg) */
void adcs_fdir_safe(fsw_t *s) { gl_put_modes(modes_fdir_safe(gl_modes(s)), s); }

/* rotor FDIR in the momentum-device states: a fixed rotor that does not follow its command is
 * isolated (instantaneous: the momentum rate against the command; windowed: the momentum change
 * over fdir_win_s against the change commanded, fluid loops only) */
/* written from the design: modes::fdir_rotors (fsw/alg) */
void adcs_fdir_rotors(fsw_t *s, double dt)
{
    const adcs_params_t *p = &s->p;
    RotorFdir f;
    RotorParams rp;
    int i;
    f.h_prev = gl_v8(s->h_prev); f.h_prev_ok = s->h_prev_ok != 0; f.cmd_r_prev = gl_v8(s->cmd_r_prev); f.fd_count = gl_v8(s->fd_count);
    f.rot_failed = gl_b8(s->rot_failed); f.faults = s->faults; f.fw_e = gl_v8(s->fw_E); f.fw_h0 = gl_v8(s->fw_h0);
    f.fw_t0 = s->fw_t0; f.fw_last = s->fw_last; f.fw_on = s->fw_on != 0;
    for (i = 0; i < NR; i++) f.fw_bad.v[i] = s->fw_bad[i];
    rp.nr = p->nr; rp.rot_tmax = gl_v8(p->rot_tmax); rp.rot_hmax = gl_v8(p->rot_hmax); rp.rot_gi = gl_u8(p->rot_gi);
    rp.rot_kind = gl_u8(p->rot_kind); rp.fdir_s = p->fdir_s; rp.fdir_win_s = p->fdir_win_s; rp.fdir_h_frac = p->fdir_h_frac;
    f = modes_fdir_rotors(f, rp, s->t, dt, gl_v8(s->z.h));
    gl_o8(f.h_prev, s->h_prev); s->h_prev_ok = f.h_prev_ok; gl_o8(f.cmd_r_prev, s->cmd_r_prev); gl_o8(f.fd_count, s->fd_count);
    for (i = 0; i < NR; i++) { s->rot_failed[i] = f.rot_failed.v[i]; s->fw_bad[i] = (int)f.fw_bad.v[i]; }
    s->faults = (uint16_t)f.faults; gl_o8(f.fw_e, s->fw_E); gl_o8(f.fw_h0, s->fw_h0);
    s->fw_t0 = f.fw_t0; s->fw_last = f.fw_last; s->fw_on = f.fw_on;
}
