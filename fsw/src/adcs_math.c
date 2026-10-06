/* adcs_math.c -- see adcs_math.h and fsw/pseudocode/01_math.md. The functions of 01_math.pc delegate to
 * their translation (fsw/alg/src/math.c); the vector utilities the pseudocode has as builtins stay here.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_math.h"
#include "adcs_alg_glue.h"

adcs_real adcs_dot(const adcs_real a[3], const adcs_real b[3]) { return a[0]*b[0] + a[1]*b[1] + a[2]*b[2]; }

void adcs_cross(const adcs_real a[3], const adcs_real b[3], adcs_real out[3])
{
    adcs_real x = a[1]*b[2] - a[2]*b[1], y = a[2]*b[0] - a[0]*b[2], z = a[0]*b[1] - a[1]*b[0];
    out[0] = x; out[1] = y; out[2] = z;
}

adcs_real adcs_norm3(const adcs_real a[3]) { return sqrt(adcs_dot(a, a)); }

void adcs_unit(const adcs_real a[3], adcs_real out[3])
{
    adcs_real n = adcs_norm3(a);
    if (n < 1e-30) n = 1e-30;
    out[0] = a[0]/n; out[1] = a[1]/n; out[2] = a[2]/n;
}

void adcs_scale3(const adcs_real a[3], adcs_real s, adcs_real out[3]) { out[0] = a[0]*s; out[1] = a[1]*s; out[2] = a[2]*s; }
void adcs_add3(const adcs_real a[3], const adcs_real b[3], adcs_real out[3]) { out[0] = a[0]+b[0]; out[1] = a[1]+b[1]; out[2] = a[2]+b[2]; }
void adcs_sub3(const adcs_real a[3], const adcs_real b[3], adcs_real out[3]) { out[0] = a[0]-b[0]; out[1] = a[1]-b[1]; out[2] = a[2]-b[2]; }
void adcs_copy3(const adcs_real a[3], adcs_real out[3]) { out[0] = a[0]; out[1] = a[1]; out[2] = a[2]; }
void adcs_zero3(adcs_real out[3]) { out[0] = 0; out[1] = 0; out[2] = 0; }
adcs_real adcs_clamp(adcs_real x, adcs_real lo, adcs_real hi) { return x < lo ? lo : (x > hi ? hi : x); }
adcs_real adcs_sign(adcs_real x) { return (x > 0) ? 1.0 : ((x < 0) ? -1.0 : 0.0); }

/* written from the design: math::maxabs3 (fsw/alg) */
adcs_real adcs_maxabs3(const adcs_real a[3]) { return math_maxabs3(gl_v3(a)); }

void adcs_mat3_vec(adcs_real M[3][3], const adcs_real v[3], adcs_real out[3])
{
    adcs_real r[3]; int i;
    for (i = 0; i < 3; i++) r[i] = M[i][0]*v[0] + M[i][1]*v[1] + M[i][2]*v[2];
    adcs_copy3(r, out);
}

/* written from the design: math::mat3t_vec (fsw/alg) */
void adcs_mat3t_vec(adcs_real M[3][3], const adcs_real v[3], adcs_real out[3])
{
    gl_o3(math_mat3t_vec(gl_m33(&M[0][0]), gl_v3(v)), out);
}

void adcs_mat3_mul(adcs_real A[3][3], adcs_real B[3][3], adcs_real out[3][3])
{
    adcs_real r[3][3]; int i, j;
    for (i = 0; i < 3; i++)
        for (j = 0; j < 3; j++) r[i][j] = A[i][0]*B[0][j] + A[i][1]*B[1][j] + A[i][2]*B[2][j];
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) out[i][j] = r[i][j];
}

void adcs_mat3_transpose(adcs_real A[3][3], adcs_real out[3][3])
{
    adcs_real r[3][3]; int i, j;
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) r[i][j] = A[j][i];
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) out[i][j] = r[i][j];
}

/* written from the design: math::skew (fsw/alg) */
void adcs_skew(const adcs_real a[3], adcs_real out[3][3]) { gl_om33(math_skew(gl_v3(a)), &out[0][0]); }

/* written from the design: math::det3 (fsw/alg) */
adcs_real adcs_det3(adcs_real M[3][3]) { return math_det3(gl_m33(&M[0][0])); }

/* written from the design: math::inv3 (fsw/alg); 0, or -1 (and zeros) for a singular M */
int adcs_inv3(adcs_real M[3][3], adcs_real out[3][3])
{
    math_inv3_out r = math_inv3(gl_m33(&M[0][0]));
    gl_om33(r.r, &out[0][0]);
    return r.ok ? 0 : -1;
}

/* written from the design: math::pinv_rows (fsw/alg); the rows of out past n are left as they are */
void adcs_pinv_rows(adcs_real A[3][8], int n, adcs_real out[8][3])
{
    pc_a3a8f a = {{{{0}}}};
    pc_a8a3f r;
    int i, k;
    for (i = 0; i < 3; i++) for (k = 0; k < n && k < 8; k++) a.v[i].v[k] = A[i][k];
    r = math_pinv_rows(a, n);
    for (k = 0; k < n && k < 8; k++) for (i = 0; i < 3; i++) out[k][i] = r.v[k].v[i];
}

/* written from the design: math::jacobi_eig4 (fsw/alg) */
void adcs_jacobi_eig4(adcs_real K[4][4], adcs_real lam[4], adcs_real V[4][4])
{
    pc_a4a4f k;
    math_jacobi_eig4_out r;
    int i, j;
    for (i = 0; i < 4; i++) k.v[i] = gl_v4(K[i]);
    r = math_jacobi_eig4(k);
    gl_o4(r.lam, lam);
    for (i = 0; i < 4; i++) for (j = 0; j < 4; j++) V[i][j] = r.v.v[i].v[j];
}

/* written from the design: math::qmult (fsw/alg) */
void adcs_qmult(const adcs_real a[4], const adcs_real b[4], adcs_real out[4]) { gl_o4(math_qmult(gl_v4(a), gl_v4(b)), out); }

/* written from the design: math::qconj (fsw/alg) */
void adcs_qconj(const adcs_real q[4], adcs_real out[4]) { gl_o4(math_qconj(gl_v4(q)), out); }

/* written from the design: math::qnorm (fsw/alg) */
void adcs_qnorm(adcs_real q[4]) { gl_o4(math_qnorm(gl_v4(q)), q); }

/* written from the design: math::dcm (fsw/alg) */
void adcs_dcm(const adcs_real q[4], adcs_real A[3][3]) { gl_om33(math_dcm(gl_v4(q)), &A[0][0]); }

/* written from the design: math::fromdcm (fsw/alg) */
void adcs_fromdcm(adcs_real R[3][3], adcs_real q[4]) { gl_o4(math_fromdcm(gl_m33(&R[0][0])), q); }

/* written from the design: math::fromrotvec (fsw/alg) */
void adcs_fromrotvec(const adcs_real th[3], adcs_real q[4]) { gl_o4(math_fromrotvec(gl_v3(th)), q); }

/* written from the design: math::qangle (fsw/alg) */
adcs_real adcs_qangle(const adcs_real a[4], const adcs_real b[4]) { return math_qangle(gl_v4(a), gl_v4(b)); }

/* written from the design: math::qerr (fsw/alg) */
void adcs_qerr(const adcs_real q_ref[4], const adcs_real q[4], adcs_real qe[4]) { gl_o4(math_qerr(gl_v4(q_ref), gl_v4(q)), qe); }
