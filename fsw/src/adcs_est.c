/* adcs_est.c -- MEKF, TRIAD, q-method (fsw/pseudocode/03_estimation.md). Every function delegates to
 * its translation of 03_estimation.pc (fsw/alg/src/estimation.c); the filter record adcs_mekf_t is
 * passed as its parts q, b, P (the update shared by the vector and quaternion measurements,
 * estimation::update3, is reached through them).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_gnc.h"
#include "adcs_alg_glue.h"

/* written from the design: estimation::mekf_init (fsw/alg); arw and rrw are kept for the prediction */
void adcs_mekf_init(adcs_mekf_t *k, const adcs_real q0[4], adcs_real sa, adcs_real sb, adcs_real arw, adcs_real rrw)
{
    estimation_mekf_init_out r = estimation_mekf_init(gl_v4(q0), sa, sb);
    gl_o4(r.q, k->q); gl_o3(r.b, k->b); gl_om66(r.p, &k->P[0][0]);
    k->arw = arw; k->rrw = rrw;
}

/* written from the design: estimation::mekf_predict (fsw/alg) */
void adcs_mekf_predict(adcs_mekf_t *k, const adcs_real wm[3], adcs_real dt)
{
    estimation_mekf_predict_out r = estimation_mekf_predict(gl_v4(k->q), gl_v3(k->b), gl_m66(&k->P[0][0]), k->arw, k->rrw, gl_v3(wm), dt);
    gl_o4(r.q, k->q); gl_om66(r.p, &k->P[0][0]);
}

/* A measurement whose innovation fails the gate (gate > 0), whose innovation covariance is singular,
 * or whose innovation is not a number is refused (0) and the filter left as it is. */
/* written from the design: estimation::mekf_vector (fsw/alg) */
int adcs_mekf_vector(adcs_mekf_t *k, const adcs_real bm[3], const adcs_real rr[3], adcs_real sigma, adcs_real gate)
{
    estimation_mekf_vector_out r = estimation_mekf_vector(gl_v4(k->q), gl_v3(k->b), gl_m66(&k->P[0][0]), gl_v3(bm), gl_v3(rr), sigma, gate);
    gl_o4(r.q, k->q); gl_o3(r.b, k->b); gl_om66(r.p, &k->P[0][0]);
    return r.took ? 1 : 0;
}

/* written from the design: estimation::mekf_quat (fsw/alg) */
int adcs_mekf_quat(adcs_mekf_t *k, const adcs_real qm[4], adcs_real sc, adcs_real sr, const adcs_real bs[3], adcs_real gate)
{
    estimation_mekf_quat_out r = estimation_mekf_quat(gl_v4(k->q), gl_v3(k->b), gl_m66(&k->P[0][0]), gl_v4(qm), sc, sr, gl_v3(bs), gate);
    gl_o4(r.q, k->q); gl_o3(r.b, k->b); gl_om66(r.p, &k->P[0][0]);
    return r.took ? 1 : 0;
}

/* written from the design: estimation::triad (fsw/alg); -1 (q left as it is) when either pair is
 * within 1e-3 (0.06 deg) of parallel and fixes no attitude */
int adcs_triad(const adcs_real b1[3], const adcs_real b2[3], const adcs_real r1[3], const adcs_real r2[3], adcs_real q[4])
{
    estimation_triad_out r = estimation_triad(gl_v3(b1), gl_v3(b2), gl_v3(r1), gl_v3(r2));
    if (!r.ok) return -1;
    gl_o4(r.q, q);
    return 0;
}

/* written from the design: estimation::quest (fsw/alg); no weights (w NULL) is weight 1 on each pair */
adcs_real adcs_quest(adcs_real b[][3], adcs_real r[][3], const adcs_real *w, int n, adcs_real q[4])
{
    pc_a16a3f bv = {{{{0}}}}, rv = {{{{0}}}};
    pc_a16f wv = {{0}};
    estimation_quest_out o;
    int l;
    for (l = 0; l < n && l < 16; l++) { bv.v[l] = gl_v3(b[l]); rv.v[l] = gl_v3(r[l]); wv.v[l] = w ? w[l] : 1.0; }
    o = estimation_quest(bv, rv, wv, n);
    gl_o4(o.q, q);
    return o.loss;
}

/* written from the design: estimation::latency (fsw/alg) */
void adcs_latency(const adcs_real q_st[4], const adcs_real w[3], adcs_real lat, adcs_real q[4])
{
    gl_o4(estimation_latency(gl_v4(q_st), gl_v3(w), lat), q);
}
