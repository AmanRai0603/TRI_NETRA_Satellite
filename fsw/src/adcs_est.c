/* adcs_est.c -- MEKF, TRIAD, q-method (fsw/pseudocode/03_estimation.md).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_gnc.h"

static void mat6_zero(adcs_real M[6][6]) { int i, j; for (i = 0; i < 6; i++) for (j = 0; j < 6; j++) M[i][j] = 0; }

void adcs_mekf_init(adcs_mekf_t *k, const adcs_real q0[4], adcs_real sa, adcs_real sb, adcs_real arw, adcs_real rrw)
{
    int i;
    for (i = 0; i < 4; i++) k->q[i] = q0[i];
    adcs_zero3(k->b);
    mat6_zero(k->P);
    for (i = 0; i < 3; i++) { k->P[i][i] = sa*sa; k->P[i + 3][i + 3] = sb*sb; }
    k->arw = arw; k->rrw = rrw;
}

void adcs_mekf_predict(adcs_mekf_t *k, const adcs_real wm[3], adcs_real dt)
{
    adcs_real w[3], th[3], dq[4], W[3][3], W2[3][3], Phi[6][6], T[6][6], Pn[6][6];
    adcs_real sv2 = k->arw*k->arw, su2 = k->rrw*k->rrw;
    int i, j, l;
    adcs_sub3(wm, k->b, w);
    adcs_scale3(w, dt, th);
    adcs_fromrotvec(th, dq);
    adcs_qmult(k->q, dq, k->q);
    adcs_qnorm(k->q);
    adcs_skew(w, W);
    adcs_mat3_mul(W, W, W2);
    mat6_zero(Phi);
    for (i = 0; i < 3; i++) {
        for (j = 0; j < 3; j++) Phi[i][j] = (i == j ? 1.0 : 0.0) - W[i][j]*dt + 0.5*W2[i][j]*dt*dt;
        Phi[i][i + 3] = -dt;
        Phi[i + 3][i + 3] = 1.0;
    }
    for (i = 0; i < 6; i++)                          /* T = Phi P */
        for (j = 0; j < 6; j++) { T[i][j] = 0; for (l = 0; l < 6; l++) T[i][j] += Phi[i][l]*k->P[l][j]; }
    for (i = 0; i < 6; i++)                          /* Pn = T Phi' + Q */
        for (j = 0; j < 6; j++) { Pn[i][j] = 0; for (l = 0; l < 6; l++) Pn[i][j] += T[i][l]*Phi[j][l]; }
    for (i = 0; i < 3; i++) {
        Pn[i][i] += sv2*dt + su2*dt*dt*dt/3.0;
        Pn[i][i + 3] += -su2*dt*dt/2.0;
        Pn[i + 3][i] += -su2*dt*dt/2.0;
        Pn[i + 3][i + 3] += su2*dt;
    }
    for (i = 0; i < 6; i++) for (j = 0; j < 6; j++) k->P[i][j] = Pn[i][j];
}

/* Joseph-form update with a 3-row measurement: H (3x6), R (3x3), innovation y. With gate > 0 an
 * innovation whose y' S^-1 y exceeds the gate is rejected (returns 0) and the state is left as is.
 * A singular innovation covariance, or an innovation that is not a number, is rejected the same way. */
static int update3(adcs_mekf_t *k, adcs_real H[3][6], adcs_real R[3][3], const adcs_real y[3], adcs_real gate)
{
    adcs_real PHt[6][3], S[3][3], Si[3][3], G[6][3], dx[6], IKH[6][6], T[6][6], Pn[6][6], dq[4], chi = 0;
    int i, j, l;
    for (i = 0; i < 6; i++) for (j = 0; j < 3; j++) { PHt[i][j] = 0; for (l = 0; l < 6; l++) PHt[i][j] += k->P[i][l]*H[j][l]; }
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) { S[i][j] = R[i][j]; for (l = 0; l < 6; l++) S[i][j] += H[i][l]*PHt[l][j]; }
    if (adcs_inv3(S, Si) != 0) return 0;
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) chi += y[i]*Si[i][j]*y[j];
    if (!(chi == chi) || (gate > 0 && chi > gate)) return 0;
    for (i = 0; i < 6; i++) for (j = 0; j < 3; j++) { G[i][j] = 0; for (l = 0; l < 3; l++) G[i][j] += PHt[i][l]*Si[l][j]; }
    for (i = 0; i < 6; i++) { dx[i] = 0; for (l = 0; l < 3; l++) dx[i] += G[i][l]*y[l]; }
    dq[0] = 0.5*dx[0]; dq[1] = 0.5*dx[1]; dq[2] = 0.5*dx[2]; dq[3] = 1.0;
    adcs_qmult(k->q, dq, k->q);
    adcs_qnorm(k->q);
    for (i = 0; i < 3; i++) k->b[i] += dx[i + 3];
    for (i = 0; i < 6; i++)
        for (j = 0; j < 6; j++) {
            IKH[i][j] = (i == j) ? 1.0 : 0.0;
            for (l = 0; l < 3; l++) IKH[i][j] -= G[i][l]*H[l][j];
        }
    for (i = 0; i < 6; i++) for (j = 0; j < 6; j++) { T[i][j] = 0; for (l = 0; l < 6; l++) T[i][j] += IKH[i][l]*k->P[l][j]; }
    for (i = 0; i < 6; i++) for (j = 0; j < 6; j++) { Pn[i][j] = 0; for (l = 0; l < 6; l++) Pn[i][j] += T[i][l]*IKH[j][l]; }
    for (i = 0; i < 6; i++)                          /* + G R G' */
        for (j = 0; j < 6; j++) {
            int a, c;
            for (a = 0; a < 3; a++) for (c = 0; c < 3; c++) Pn[i][j] += G[i][a]*R[a][c]*G[j][c];
        }
    for (i = 0; i < 6; i++) for (j = 0; j < 6; j++) k->P[i][j] = Pn[i][j];
    return 1;
}

int adcs_mekf_vector(adcs_mekf_t *k, const adcs_real bm[3], const adcs_real rr[3], adcs_real sigma, adcs_real gate)
{
    adcs_real b[3], r[3], bh[3], A[3][3], Sk[3][3], H[3][6], R[3][3], y[3];
    int i, j;
    adcs_unit(bm, b); adcs_unit(rr, r);
    adcs_dcm(k->q, A); adcs_mat3_vec(A, r, bh);
    adcs_skew(bh, Sk);
    for (i = 0; i < 3; i++) for (j = 0; j < 6; j++) H[i][j] = (j < 3) ? Sk[i][j] : 0.0;
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) R[i][j] = (i == j) ? sigma*sigma : 0.0;
    adcs_sub3(b, bh, y);
    return update3(k, H, R, y, gate);
}

int adcs_mekf_quat(adcs_mekf_t *k, const adcs_real qm[4], adcs_real sc, adcs_real sr, const adcs_real bs[3], adcs_real gate)
{
    adcs_real dq[4], y[3], H[3][6], R[3][3];
    int i, j;
    adcs_qerr(k->q, qm, dq);
    for (i = 0; i < 3; i++) y[i] = 2.0*dq[i];
    for (i = 0; i < 3; i++) for (j = 0; j < 6; j++) H[i][j] = (i == j) ? 1.0 : 0.0;
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) R[i][j] = (i == j ? sc*sc : 0.0) + (sr*sr - sc*sc)*bs[i]*bs[j];
    return update3(k, H, R, y, gate);
}

#define TRIAD_MIN_SIN 1e-3                 /* 0.06 deg: closer to parallel fixes no attitude */

int adcs_triad(const adcs_real b1[3], const adcs_real b2[3], const adcs_real r1[3], const adcs_real r2[3], adcs_real q[4])
{
    adcs_real tb[3][3], tr[3][3], c[3], A[3][3];
    int i, j;
    adcs_cross(b1, b2, c); if (!(adcs_norm3(c) > TRIAD_MIN_SIN*adcs_norm3(b1)*adcs_norm3(b2))) return -1;
    adcs_cross(r1, r2, c); if (!(adcs_norm3(c) > TRIAD_MIN_SIN*adcs_norm3(r1)*adcs_norm3(r2))) return -1;
    adcs_unit(b1, tb[0]); adcs_cross(b1, b2, c); adcs_unit(c, tb[1]); adcs_cross(tb[0], tb[1], tb[2]);
    adcs_unit(r1, tr[0]); adcs_cross(r1, r2, c); adcs_unit(c, tr[1]); adcs_cross(tr[0], tr[1], tr[2]);
    /* A = [t1b t2b t3b] [t1r t2r t3r]' : columns are the triad vectors */
    for (i = 0; i < 3; i++)
        for (j = 0; j < 3; j++) A[i][j] = tb[0][i]*tr[0][j] + tb[1][i]*tr[1][j] + tb[2][i]*tr[2][j];
    adcs_fromdcm(A, q);
    return 0;
}

adcs_real adcs_quest(adcs_real b[][3], adcs_real r[][3], const adcs_real *w, int n, adcs_real q[4])
{
    adcs_real B[3][3], K[4][4], lam[4], V[4][4], sg, ws = 0, z[3];
    int i, j, l, im = 0;
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) B[i][j] = 0;
    for (l = 0; l < n; l++) {
        adcs_real wl = w ? w[l] : 1.0;
        ws += wl;
        for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) B[i][j] += wl*b[l][i]*r[l][j];
    }
    sg = B[0][0] + B[1][1] + B[2][2];
    z[0] = B[1][2] - B[2][1]; z[1] = B[2][0] - B[0][2]; z[2] = B[0][1] - B[1][0];
    for (i = 0; i < 3; i++) {
        for (j = 0; j < 3; j++) K[i][j] = B[i][j] + B[j][i] - (i == j ? sg : 0.0);
        K[i][3] = z[i]; K[3][i] = z[i];
    }
    K[3][3] = sg;
    adcs_jacobi_eig4(K, lam, V);
    for (i = 1; i < 4; i++) if (lam[i] > lam[im]) im = i;
    for (i = 0; i < 4; i++) q[i] = V[i][im];
    adcs_qnorm(q);
    return ws - lam[im];
}

void adcs_latency(const adcs_real q_st[4], const adcs_real w[3], adcs_real lat, adcs_real q[4])
{
    adcs_real th[3], dq[4];
    adcs_scale3(w, lat, th);
    adcs_fromrotvec(th, dq);
    adcs_qmult(q_st, dq, q);
    adcs_qnorm(q);
}
