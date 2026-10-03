/* adcs_guid.c -- guidance (04_guidance.md): the reference attitude and rate of each pointing
 * state, the nadir yaw flip, the boresight offset, and which guidance a controller state flies.
 * Group gdn (design/groups.toml); twin: asils.fsw.guidance, yaw_flip, boresight_offset; Rust: guid.rs.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_fsw_int.h"

/* the guidance a controller state flies: 0 nadir, 1 target, 2 slew, 4 Sun */
int adcs_guid_kind(uint8_t mode)
{
    switch (mode) {
    case ADCS_MODE_TARGET_FINE: return 1;
    case ADCS_MODE_SLEW_FINE: return 2;
    case ADCS_MODE_SUN_MTQ: case ADCS_MODE_SUN_FINE: return 4;
    default: return 0;
    }
}

void adcs_guidance(int kind, const adcs_real r[3], const adcs_real v[3], adcs_real t, const adcs_guid_t *g,
                   adcs_real q_ref[4], adcs_real w_ref[3], adcs_real wd_ref[3])
{
    adcs_real rh[3], vh[3], nrm[3], ram[3], mr[3], R[3][3], q_nad[4], w_orb[3], A[3][3], ax[3], th[3], dq[4], c[3];
    int i;
    adcs_unit(r, rh); adcs_unit(v, vh);
    adcs_scale3(rh, -1.0, mr);
    adcs_cross(vh, mr, c); adcs_unit(c, nrm);
    adcs_cross(mr, nrm, c); adcs_unit(c, ram);
    for (i = 0; i < 3; i++) { R[0][i] = -ram[i]; R[1][i] = -rh[i]; R[2][i] = -nrm[i]; }
    adcs_fromdcm(R, q_nad);
    if (g->q_off[0] != 0 || g->q_off[1] != 0 || g->q_off[2] != 0 || g->q_off[3] != 0) adcs_qmult(q_nad, g->q_off, q_nad);
    if (g->flip) {           /* 180 deg about the boresight: q_f = [u, 0] */
        adcs_real u[3], qf[4];
        u[0] = 1; u[1] = 0; u[2] = 0;
        if (adcs_norm3(g->roll_axis) > 0) adcs_unit(g->roll_axis, u);
        qf[0] = u[0]; qf[1] = u[1]; qf[2] = u[2]; qf[3] = 0;
        adcs_qmult(q_nad, qf, q_nad);
    }
    adcs_cross(r, v, w_orb); adcs_scale3(w_orb, 1.0/adcs_dot(r, r), w_orb);
    adcs_zero3(wd_ref);
    ax[0] = 1; ax[1] = 0; ax[2] = 0;
    if (adcs_norm3(g->axis) > 0) adcs_unit(g->axis, ax);
    switch (kind) {
    case 1:          /* target */
        adcs_scale3(ax, g->roll_deg*ADCS_D2R, th); adcs_fromrotvec(th, dq); adcs_qmult(q_nad, dq, q_ref);
        adcs_dcm(q_ref, A); adcs_mat3_vec(A, w_orb, w_ref);
        break;
    case 2: {        /* cycloidal slew in the rotating nadir frame */
        adcs_real tau = adcs_clamp((t - g->t0)/g->T, 0.0, 1.0), sd = 0, sdd = 0, s, ph = g->roll_deg*ADCS_D2R, wo[3];
        if (t > g->t0 && t < g->t0 + g->T) { sd = (1 - cos(2*ADCS_PI*tau))/g->T; sdd = 2*ADCS_PI*sin(2*ADCS_PI*tau)/(g->T*g->T); }
        s = tau - sin(2*ADCS_PI*tau)/(2*ADCS_PI);
        adcs_scale3(ax, ph*s, th); adcs_fromrotvec(th, dq); adcs_qmult(q_nad, dq, q_ref);
        adcs_dcm(q_ref, A); adcs_mat3_vec(A, w_orb, wo);
        {   /* the slew frame turns at ax ph s_dot relative to the orbiting frame: transport term on w_orb */
            adcs_real wr[3], tr[3];
            adcs_scale3(ax, ph*sd, wr); adcs_cross(wr, wo, tr);
            for (i = 0; i < 3; i++) { w_ref[i] = wo[i] + ax[i]*ph*sd; wd_ref[i] = ax[i]*ph*sdd - tr[i]; }
        }
        break; }
    case 3:          /* inertial */
        for (i = 0; i < 4; i++) q_ref[i] = g->q_inertial[i];
        adcs_zero3(w_ref);
        break;
    case 4: {        /* Sun referencing */
        adcs_real a[3], b[3], s[3], e2[3], e3[3], b3[3], Bm[3][3], Em[3][3], M[3][3];
        int j;
        a[0] = 0; a[1] = 0; a[2] = -1;
        if (adcs_norm3(g->sun_axis) > 0) adcs_unit(g->sun_axis, a);
        b[0] = 1; b[1] = 0; b[2] = 0;
        if (adcs_norm3(g->roll_axis) > 0) adcs_copy3(g->roll_axis, b);
        {   /* b made normal to the power face */
            adcs_real ab = adcs_dot(a, b);
            for (i = 0; i < 3; i++) b[i] -= ab*a[i];
        }
        if (adcs_norm3(b) < 1e-6) { b[0] = -a[1]*a[0]; b[1] = 1 - a[1]*a[1]; b[2] = -a[1]*a[2]; }
        adcs_unit(b, b);
        adcs_unit(g->sun_eci, s);
        for (i = 0; i < 3; i++) e2[i] = nrm[i] - adcs_dot(nrm, s)*s[i];
        if (adcs_norm3(e2) < 1e-6) { e2[0] = -s[2]*s[0]; e2[1] = -s[2]*s[1]; e2[2] = 1 - s[2]*s[2]; }
        adcs_unit(e2, e2);
        adcs_cross(a, b, b3); adcs_cross(s, e2, e3);
        for (i = 0; i < 3; i++) { Bm[i][0] = a[i]; Bm[i][1] = b[i]; Bm[i][2] = b3[i]; Em[i][0] = s[i]; Em[i][1] = e2[i]; Em[i][2] = e3[i]; }
        for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) M[i][j] = Bm[i][0]*Em[j][0] + Bm[i][1]*Em[j][1] + Bm[i][2]*Em[j][2];
        adcs_fromdcm(M, q_ref);
        adcs_zero3(w_ref);
        break; }
    default:         /* nadir */
        for (i = 0; i < 4; i++) q_ref[i] = q_nad[i];
        adcs_dcm(q_nad, A); adcs_mat3_vec(A, w_orb, w_ref);
        break;
    }
}

/* Nadir-family yaw flip with hysteresis: turn 180 deg about the boresight when the power face would look
 * away from the Sun (04_guidance.md) */
void adcs_yaw_flip(adcs_guid_t *g, const adcs_real r[3], const adcs_real v[3], adcs_real hyst)
{
    adcs_guid_t g0 = *g;
    adcs_real q[4], w[3], wd[3], A[3][3], s[3], sb[3], a[3], d;
    g0.flip = 0;
    adcs_guidance(0, r, v, 0.0, &g0, q, w, wd);
    adcs_unit(g->sun_eci, s); adcs_dcm(q, A); adcs_mat3_vec(A, s, sb);
    a[0] = 0; a[1] = 0; a[2] = -1;
    if (adcs_norm3(g->sun_axis) > 0) adcs_unit(g->sun_axis, a);
    d = adcs_dot(a, sb);
    if (d < -hyst) g->flip = 1;
    else if (d > hyst) g->flip = 0;
}

void adcs_boresight_offset(const adcs_real bs_in[3], adcs_real q[4])
{
    adcs_real ey[3] = {0, 1, 0}, bs[3], ax[3], K[3][3], K2[3][3], A[3][3], c, s, k[3];
    int i, j;
    adcs_unit(bs_in, bs);
    adcs_cross(ey, bs, ax);
    c = adcs_dot(ey, bs); s = adcs_norm3(ax);
    if (s < 1e-12) {
        q[0] = 0; q[1] = 0; q[2] = 0; q[3] = 1;
        if (c <= 0) { q[0] = 1; q[3] = 0; }
        return;
    }
    adcs_scale3(ax, 1.0/s, k);
    adcs_skew(k, K); adcs_mat3_mul(K, K, K2);
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) A[i][j] = (i == j ? 1.0 : 0.0) + s*K[i][j] + (1 - c)*K2[i][j];
    adcs_fromdcm(A, q);
}

