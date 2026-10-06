/* adcs_alloc.c -- rotor geometry, SR steering, dumping, thruster duty (fsw/pseudocode/07).
 * Every function delegates to its translation of 07_allocation.pc (fsw/alg/src/allocation.c); the
 * parameters (adcs_params_t) are passed as the fields each one reads.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_gnc.h"
#include "adcs_alg_glue.h"

/* written from the design: allocation::rotor_axes (fsw/alg); the columns of A past nr are left as they are */
void adcs_rotor_axes(const adcs_params_t *p, const adcs_real delta[4], adcs_real A[3][8])
{
    pc_a3a8f a = allocation_rotor_axes(p->nr, gl_m83(&p->rot_a0[0][0]), gl_u8(p->rot_gi), gl_m43(&p->gim_axis[0][0]), gl_v4(delta));
    int i, k;
    for (i = 0; i < p->nr && i < 8; i++) for (k = 0; k < 3; k++) A[k][i] = a.v[k].v[i];
}

/* written from the design: allocation::steer_sr (fsw/alg); gdot past ng and hdot past nr are left as they are */
void adcs_steer_sr(const adcs_real tau[3], adcs_real A[3][8], const adcs_real h[8], const adcs_params_t *p,
                   int wheels, adcs_real gdot[4], adcs_real hdot[8])
{
    pc_a3a8f a = {{{{0}}}};
    allocation_steer_sr_out o;
    int i, k;
    for (k = 0; k < 3; k++) for (i = 0; i < p->nr && i < 8; i++) a.v[k].v[i] = A[k][i];
    o = allocation_steer_sr(gl_v3(tau), a, gl_v8(h), p->nr, p->ng, gl_u8(p->rot_gi), gl_m43(&p->gim_axis[0][0]),
                            p->gim_rate_max, p->cmg_lam0, p->cmg_mu, wheels != 0);
    for (i = 0; i < p->ng && i < 4; i++) gdot[i] = o.gdot.v[i];
    for (i = 0; i < p->nr && i < 8; i++) hdot[i] = o.hdot.v[i];
}

/* written from the design: allocation::dump (fsw/alg) */
void adcs_dump(const adcs_real Hdev[3], const adcs_real Ht[3], const adcs_real B[3], adcs_real k, adcs_real m_max, adcs_real m[3])
{
    gl_o3(allocation_dump(gl_v3(Hdev), gl_v3(Ht), gl_v3(B), k, m_max), m);
}

/* written from the design: allocation::rcs_duty (fsw/alg) */
void adcs_rcs_duty(const adcs_real req[3], const adcs_params_t *p, adcs_real T, adcs_real duty[6], adcs_real tau[3])
{
    pc_a6a3f rt;
    allocation_rcs_duty_out o;
    int k;
    for (k = 0; k < ADCS_MAX_COUPLES; k++) rt.v[k] = gl_v3(p->rcs_tau[k]);
    o = allocation_rcs_duty(gl_v3(req), p->nc, rt, p->rcs_mib, p->rcs_res, T);
    for (k = 0; k < ADCS_MAX_COUPLES; k++) duty[k] = o.duty.v[k];
    gl_o3(o.tau, tau);
}
