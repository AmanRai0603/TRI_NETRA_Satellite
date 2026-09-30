/* adcs_alloc.c -- rotor geometry, SR steering, dumping, thruster duty (fsw/pseudocode/07).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_gnc.h"

void adcs_rotor_axes(const adcs_params_t *p, const adcs_real delta[4], adcs_real A[3][8])
{
    int i, k;
    for (i = 0; i < p->nr; i++) {
        adcs_real a0[3], g[3], t0[3];
        for (k = 0; k < 3; k++) a0[k] = p->rot_a0[i][k];
        if (p->rot_gi[i] > 0) {
            int j = p->rot_gi[i] - 1;
            for (k = 0; k < 3; k++) g[k] = p->gim_axis[j][k];
            adcs_cross(g, a0, t0);
            for (k = 0; k < 3; k++) A[k][i] = cos(delta[j])*a0[k] + sin(delta[j])*t0[k];
        } else {
            for (k = 0; k < 3; k++) A[k][i] = a0[k];
        }
    }
}

void adcs_steer_sr(const adcs_real tau[3], adcs_real A[3][8], const adcs_real h[8], const adcs_params_t *p,
                   int wheels, adcs_real gdot[4], adcs_real hdot[8])
{
    adcs_real Jg[3][4], J[3][12], W[12], M[3][3], Mi[3][3], x[3], u[12], h0 = 0, ms, lam, s = 1.0;
    int ng = p->ng, nr = p->nr, i, j, k, n;
    for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) Jg[i][j] = 0;
    for (i = 0; i < nr; i++) {
        if (p->rot_gi[i] > 0) {
            adcs_real a[3], g[3], c[3];
            j = p->rot_gi[i] - 1;
            for (k = 0; k < 3; k++) { a[k] = A[k][i]; g[k] = p->gim_axis[j][k]; }
            adcs_cross(g, a, c);
            for (k = 0; k < 3; k++) Jg[k][j] = -h[i]*c[k];
            if (fabs(h[i]) > h0) h0 = fabs(h[i]);
        }
    }
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) { M[i][j] = 0; for (k = 0; k < ng; k++) M[i][j] += Jg[i][k]*Jg[j][k]; }
    if (h0 < 1e-12) h0 = 1e-12;
    ms = adcs_det3(M)/(h0*h0*h0*h0*h0*h0);
    n = wheels ? ng + nr : ng;
    for (i = 0; i < 3; i++) {
        for (j = 0; j < ng; j++) J[i][j] = Jg[i][j];
        for (j = 0; wheels && j < nr; j++) J[i][ng + j] = -A[i][j];
    }
    for (j = 0; j < n; j++) W[j] = (j < ng) ? 1.0 : (0.01 + 2.0*exp(-10.0*ms));
    lam = p->cmg_lam0*exp(-p->cmg_mu*ms);
    for (i = 0; i < 3; i++)
        for (j = 0; j < 3; j++) {
            M[i][j] = (i == j) ? lam : 0.0;
            for (k = 0; k < n; k++) M[i][j] += J[i][k]*W[k]*J[j][k];
        }
    adcs_inv3(M, Mi);
    adcs_mat3_vec(Mi, tau, x);
    for (k = 0; k < n; k++) u[k] = W[k]*(J[0][k]*x[0] + J[1][k]*x[1] + J[2][k]*x[2]);
    for (j = 0; j < ng; j++) { gdot[j] = u[j]; if (fabs(u[j])/p->gim_rate_max > s) s = fabs(u[j])/p->gim_rate_max; }
    for (j = 0; j < ng; j++) gdot[j] /= s;
    for (i = 0; i < nr; i++) hdot[i] = wheels ? u[ng + i]/s : 0.0;
}

void adcs_dump(const adcs_real Hdev[3], const adcs_real Ht[3], const adcs_real B[3], adcs_real k, adcs_real m_max, adcs_real m[3])
{
    adcs_real dh[3], Bs = adcs_dot(B, B);
    adcs_sub3(Hdev, Ht, dh);
    if (Bs < 1e-18) { adcs_zero3(m); return; }
    adcs_cross(dh, B, m); adcs_scale3(m, k/Bs, m);
    adcs_sat_dipole(m, m_max);
}

void adcs_rcs_duty(const adcs_real req[3], const adcs_params_t *p, adcs_real T, adcs_real duty[6], adcs_real tau[3])
{
    int ax, k, i;
    for (k = 0; k < ADCS_MAX_COUPLES; k++) duty[k] = 0;
    adcs_zero3(tau);
    for (ax = 0; ax < 3; ax++) {
        adcs_real u = req[ax], on;
        if (u == 0) continue;
        k = (u > 0) ? 2*ax : 2*ax + 1;
        if (k >= p->nc || !(fabs(p->rcs_tau[k][ax]) > 0)) continue;     /* no couple for this axis and sense */
        on = fabs(u)/fabs(p->rcs_tau[k][ax]); if (on > 1) on = 1;
        on *= T;
        if (on < p->rcs_mib) continue;
        on = round(on/p->rcs_res)*p->rcs_res;
        duty[k] = on/T;
        for (i = 0; i < 3; i++) tau[i] += p->rcs_tau[k][i]*duty[k];
    }
}
