/*
 * adcs_alg_glue.h -- private to the flight software: the copies between the hand-written C's
 * arrays (adcs_real[n], row-major matrices) and the value types of the algorithms written from the
 * design (fsw/alg, adcs_alg.h: pc_a3f = struct { double v[3]; }, nested for matrices). Element by
 * element, so a copy is exact. Not part of the ABI.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#ifndef ADCS_ALG_GLUE_H
#define ADCS_ALG_GLUE_H
#include <stdint.h>
#include "adcs_math.h"
#include "adcs_alg.h"

/* vectors in */
static inline pc_a3f gl_v3(const adcs_real *a) { pc_a3f r; int i; for (i = 0; i < 3; i++) r.v[i] = a[i]; return r; }
static inline pc_a4f gl_v4(const adcs_real *a) { pc_a4f r; int i; for (i = 0; i < 4; i++) r.v[i] = a[i]; return r; }
static inline pc_a8f gl_v8(const adcs_real *a) { pc_a8f r; int i; for (i = 0; i < 8; i++) r.v[i] = a[i]; return r; }
static inline pc_a8i gl_u8(const uint8_t *a) { pc_a8i r; int i; for (i = 0; i < 8; i++) r.v[i] = a[i]; return r; }
static inline pc_a8b gl_b8(const int *a) { pc_a8b r; int i; for (i = 0; i < 8; i++) r.v[i] = a[i] != 0; return r; }

/* vectors out */
static inline void gl_o3(pc_a3f v, adcs_real *o) { int i; for (i = 0; i < 3; i++) o[i] = v.v[i]; }
static inline void gl_o4(pc_a4f v, adcs_real *o) { int i; for (i = 0; i < 4; i++) o[i] = v.v[i]; }
static inline void gl_o8(pc_a8f v, adcs_real *o) { int i; for (i = 0; i < 8; i++) o[i] = v.v[i]; }

/* matrices in, from the first element of a row-major array (&M[0][0]) */
static inline pc_a3a3f gl_m33(const adcs_real *m) { pc_a3a3f r; int i, j; for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) r.v[i].v[j] = m[3*i + j]; return r; }
static inline pc_a3a8f gl_m38(const adcs_real *m) { pc_a3a8f r; int i, j; for (i = 0; i < 3; i++) for (j = 0; j < 8; j++) r.v[i].v[j] = m[8*i + j]; return r; }
static inline pc_a4a3f gl_m43(const adcs_real *m) { pc_a4a3f r; int i, j; for (i = 0; i < 4; i++) for (j = 0; j < 3; j++) r.v[i].v[j] = m[3*i + j]; return r; }
static inline pc_a8a3f gl_m83(const adcs_real *m) { pc_a8a3f r; int i, j; for (i = 0; i < 8; i++) for (j = 0; j < 3; j++) r.v[i].v[j] = m[3*i + j]; return r; }
static inline pc_a6a6f gl_m66(const adcs_real *m) { pc_a6a6f r; int i, j; for (i = 0; i < 6; i++) for (j = 0; j < 6; j++) r.v[i].v[j] = m[6*i + j]; return r; }

/* matrices out */
static inline void gl_om33(pc_a3a3f v, adcs_real *o) { int i, j; for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) o[3*i + j] = v.v[i].v[j]; }
static inline void gl_om66(pc_a6a6f v, adcs_real *o) { int i, j; for (i = 0; i < 6; i++) for (j = 0; j < 6; j++) o[6*i + j] = v.v[i].v[j]; }

#ifdef ADCS_FSW_INT_H
/* the records of 08_mode_manager.pc onto the flight software's state (fsw_t), field by field */
static inline Modes gl_modes(const fsw_t *s)
{
    Modes m;
    m.mode = s->mode; m.t = s->t; m.t_mode = s->t_mode; m.hold = s->hold; m.ho = s->ho != 0; m.ho_t = s->ho_t;
    m.ad_ok = s->ad_ok != 0; m.n_rej = s->n_rej; m.i_q = gl_v3(s->I_q); m.sz_sum = s->sz_sum; m.sz_n = s->sz_n;
    m.sz_t0 = s->sz_t0; m.sigma = s->sigma; m.s_prop = gl_v3(s->s_prop); m.s_prop_ok = s->s_prop_ok != 0;
    m.acq_hold = s->acq_hold; m.sched_i = s->sched_i; m.faults = s->faults; m.w_est = gl_v3(s->w_est);
    m.mag_seen = s->mag_seen != 0; m.mag_age = s->mag_age; m.gyro_age = s->gyro_age;
    return m;
}
static inline void gl_put_modes(Modes m, fsw_t *s)
{
    s->mode = (uint8_t)m.mode; s->t = m.t; s->t_mode = m.t_mode; s->hold = m.hold; s->ho = m.ho; s->ho_t = m.ho_t;
    s->ad_ok = m.ad_ok; s->n_rej = (int)m.n_rej; gl_o3(m.i_q, s->I_q); s->sz_sum = m.sz_sum; s->sz_n = (int)m.sz_n;
    s->sz_t0 = m.sz_t0; s->sigma = m.sigma; gl_o3(m.s_prop, s->s_prop); s->s_prop_ok = m.s_prop_ok;
    s->acq_hold = m.acq_hold; s->sched_i = (int)m.sched_i; s->faults = (uint16_t)m.faults; gl_o3(m.w_est, s->w_est);
    s->mag_seen = m.mag_seen; s->mag_age = m.mag_age; s->gyro_age = m.gyro_age;
}
static inline ModeParams gl_mode_params(const adcs_params_t *p)
{
    ModeParams mp;
    int i;
    mp.auto_next = p->auto_next; mp.n_sched = p->n_sched; mp.sched_t = gl_v8(p->sched_t);
    for (i = 0; i < 8; i++) mp.sched_mode.v[i] = p->sched_mode[i];
    mp.detumble_exit = p->detumble_exit; mp.detumble_hold_s = p->detumble_hold_s; mp.ss_law = p->ss_law;
    mp.ss_omega_max_dps = p->ss_omega_max_dps; mp.ss_spin_dps = p->ss_spin_dps; mp.ss_z_in_dps = p->ss_z_in_dps;
    mp.ss_perp_in_dps = p->ss_perp_in_dps; mp.ss_t_check_s = p->ss_t_check_s; mp.ss_sun_min = p->ss_sun_min;
    mp.ss_dwell_in_s = p->ss_dwell_in_s; mp.ss_omega_exit_dps = p->ss_omega_exit_dps; mp.ss_perp_out_dps = p->ss_perp_out_dps;
    mp.ss_dwell_out_s = p->ss_dwell_out_s; mp.sun_axis = gl_v3(p->sun_axis); mp.sa_done_deg = p->sa_done_deg;
    mp.sa_done_hold_s = p->sa_done_hold_s;
    return mp;
}
#endif

#endif
