/* adcs_guid.c -- guidance (04_guidance.md): the reference attitude and rate of each pointing
 * state, the nadir yaw flip, the boresight offset, and which guidance a controller state flies.
 * Group gdn (design/groups.toml); twin: asils.fsw.guidance, yaw_flip, boresight_offset; Rust: guid.rs.
 * Every function delegates to its translation of 04_guidance.pc (fsw/alg/src/guidance.c); the
 * guidance record adcs_guid_t is passed field by field.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_fsw_int.h"
#include "adcs_alg_glue.h"

/* the guidance a controller state flies: 0 nadir, 1 target, 2 slew, 4 Sun */
/* written from the design: guidance::guid_kind (fsw/alg) */
int adcs_guid_kind(uint8_t mode) { return (int)guidance_guid_kind(mode); }

/* written from the design: guidance::guidance (fsw/alg) */
void adcs_guidance(int kind, const adcs_real r[3], const adcs_real v[3], adcs_real t, const adcs_guid_t *g,
                   adcs_real q_ref[4], adcs_real w_ref[3], adcs_real wd_ref[3])
{
    guidance_guidance_out o = guidance_guidance(kind, gl_v3(r), gl_v3(v), t, gl_v4(g->q_off), g->roll_deg, g->t0, g->T,
                                                gl_v3(g->axis), gl_v4(g->q_inertial), gl_v3(g->sun_axis), gl_v3(g->roll_axis),
                                                gl_v3(g->sun_eci), g->flip != 0);
    gl_o4(o.q_ref, q_ref); gl_o3(o.w_ref, w_ref); gl_o3(o.wd_ref, wd_ref);
}

/* Nadir-family yaw flip with hysteresis: turn 180 deg about the boresight when the power face would look
 * away from the Sun (04_guidance.md) */
/* written from the design: guidance::yaw_flip (fsw/alg); the flag is set or cleared only past the hysteresis */
void adcs_yaw_flip(adcs_guid_t *g, const adcs_real r[3], const adcs_real v[3], adcs_real hyst)
{
    int flip = guidance_yaw_flip(gl_v3(r), gl_v3(v), gl_v4(g->q_off), gl_v3(g->sun_axis), gl_v3(g->roll_axis),
                                 gl_v3(g->sun_eci), g->flip != 0, hyst);
    if (flip != (g->flip != 0)) g->flip = flip;
}

/* written from the design: guidance::boresight_offset (fsw/alg) */
void adcs_boresight_offset(const adcs_real bs_in[3], adcs_real q[4]) { gl_o4(guidance_boresight_offset(gl_v3(bs_in)), q); }
