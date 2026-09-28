/* adcs_math.c -- see adcs_math.h and fsw/pseudocode/01_math.md.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_math.h"

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

adcs_real adcs_maxabs3(const adcs_real a[3])
{
    adcs_real m = fabs(a[0]);
    if (fabs(a[1]) > m) m = fabs(a[1]);
    if (fabs(a[2]) > m) m = fabs(a[2]);
    return m;
}

void adcs_mat3_vec(adcs_real M[3][3], const adcs_real v[3], adcs_real out[3])
{
    adcs_real r[3]; int i;
    for (i = 0; i < 3; i++) r[i] = M[i][0]*v[0] + M[i][1]*v[1] + M[i][2]*v[2];
    adcs_copy3(r, out);
}

void adcs_mat3t_vec(adcs_real M[3][3], const adcs_real v[3], adcs_real out[3])
{
    adcs_real r[3]; int i;
    for (i = 0; i < 3; i++) r[i] = M[0][i]*v[0] + M[1][i]*v[1] + M[2][i]*v[2];
    adcs_copy3(r, out);
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

void adcs_skew(const adcs_real a[3], adcs_real out[3][3])
{
    out[0][0] = 0;     out[0][1] = -a[2]; out[0][2] = a[1];
    out[1][0] = a[2];  out[1][1] = 0;     out[1][2] = -a[0];
    out[2][0] = -a[1]; out[2][1] = a[0];  out[2][2] = 0;
}

adcs_real adcs_det3(adcs_real M[3][3])
{
    return M[0][0]*(M[1][1]*M[2][2] - M[1][2]*M[2][1]) - M[0][1]*(M[1][0]*M[2][2] - M[1][2]*M[2][0])
         + M[0][2]*(M[1][0]*M[2][1] - M[1][1]*M[2][0]);
}

int adcs_inv3(adcs_real M[3][3], adcs_real out[3][3])
{
    adcs_real d = adcs_det3(M), r[3][3]; int i, j;
    if (fabs(d) < 1e-300) {
        for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) out[i][j] = 0;
        return -1;
    }
    r[0][0] = (M[1][1]*M[2][2] - M[1][2]*M[2][1])/d; r[0][1] = (M[0][2]*M[2][1] - M[0][1]*M[2][2])/d; r[0][2] = (M[0][1]*M[1][2] - M[0][2]*M[1][1])/d;
    r[1][0] = (M[1][2]*M[2][0] - M[1][0]*M[2][2])/d; r[1][1] = (M[0][0]*M[2][2] - M[0][2]*M[2][0])/d; r[1][2] = (M[0][2]*M[1][0] - M[0][0]*M[1][2])/d;
    r[2][0] = (M[1][0]*M[2][1] - M[1][1]*M[2][0])/d; r[2][1] = (M[0][1]*M[2][0] - M[0][0]*M[2][1])/d; r[2][2] = (M[0][0]*M[1][1] - M[0][1]*M[1][0])/d;
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) out[i][j] = r[i][j];
    return 0;
}

void adcs_pinv_rows(adcs_real A[3][8], int n, adcs_real out[8][3])
{
    adcs_real S[3][3], Si[3][3], tr, eps; int i, j, k;
    for (i = 0; i < 3; i++)
        for (j = 0; j < 3; j++) {
            S[i][j] = 0;
            for (k = 0; k < n; k++) S[i][j] += A[i][k]*A[j][k];
        }
    tr = (S[0][0] + S[1][1] + S[2][2])/3.0;
    eps = 1e-12*(tr > 1e-30 ? tr : 1e-30);
    for (i = 0; i < 3; i++) S[i][i] += eps;
    adcs_inv3(S, Si);
    for (k = 0; k < n; k++)
        for (j = 0; j < 3; j++) out[k][j] = A[0][k]*Si[0][j] + A[1][k]*Si[1][j] + A[2][k]*Si[2][j];
}

void adcs_jacobi_eig4(adcs_real K[4][4], adcs_real lam[4], adcs_real V[4][4])
{
    adcs_real a[4][4]; int i, j, s, pi_, p, q;
    static const int PQ[6][2] = {{0, 1}, {0, 2}, {0, 3}, {1, 2}, {1, 3}, {2, 3}};
    for (i = 0; i < 4; i++) for (j = 0; j < 4; j++) { a[i][j] = K[i][j]; V[i][j] = (i == j) ? 1.0 : 0.0; }
    for (s = 0; s < 12; s++) {
        for (pi_ = 0; pi_ < 6; pi_++) {
            adcs_real th, t, c, sn, apq;
            p = PQ[pi_][0]; q = PQ[pi_][1]; apq = a[p][q];
            if (fabs(apq) < 1e-300) continue;
            th = (a[q][q] - a[p][p])/(2.0*apq);
            t = adcs_sign(th)/(fabs(th) + sqrt(th*th + 1.0));
            if (th == 0) t = 1.0;
            c = 1.0/sqrt(t*t + 1.0); sn = t*c;
            for (i = 0; i < 4; i++) {        /* a = J^T a J, columns p,q then rows p,q */
                adcs_real aip = a[i][p], aiq = a[i][q];
                a[i][p] = c*aip - sn*aiq; a[i][q] = sn*aip + c*aiq;
            }
            for (i = 0; i < 4; i++) {
                adcs_real api = a[p][i], aqi = a[q][i];
                a[p][i] = c*api - sn*aqi; a[q][i] = sn*api + c*aqi;
            }
            for (i = 0; i < 4; i++) {
                adcs_real vip = V[i][p], viq = V[i][q];
                V[i][p] = c*vip - sn*viq; V[i][q] = sn*vip + c*viq;
            }
        }
    }
    for (i = 0; i < 4; i++) lam[i] = a[i][i];
}

void adcs_qmult(const adcs_real a[4], const adcs_real b[4], adcs_real out[4])
{
    adcs_real r[4];
    r[0] = a[3]*b[0] + b[3]*a[0] + (a[1]*b[2] - a[2]*b[1]);
    r[1] = a[3]*b[1] + b[3]*a[1] + (a[2]*b[0] - a[0]*b[2]);
    r[2] = a[3]*b[2] + b[3]*a[2] + (a[0]*b[1] - a[1]*b[0]);
    r[3] = a[3]*b[3] - (a[0]*b[0] + a[1]*b[1] + a[2]*b[2]);
    out[0] = r[0]; out[1] = r[1]; out[2] = r[2]; out[3] = r[3];
}

void adcs_qconj(const adcs_real q[4], adcs_real out[4]) { out[0] = -q[0]; out[1] = -q[1]; out[2] = -q[2]; out[3] = q[3]; }

void adcs_qnorm(adcs_real q[4])
{
    adcs_real n = sqrt(q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]);
    if (n < 1e-30) { q[0] = 0; q[1] = 0; q[2] = 0; q[3] = 1; return; }
    q[0] /= n; q[1] /= n; q[2] /= n; q[3] /= n;
}

void adcs_dcm(const adcs_real q[4], adcs_real A[3][3])
{
    adcs_real x = q[0], y = q[1], z = q[2], w = q[3];
    A[0][0] = 1 - 2*(y*y + z*z); A[0][1] = 2*(x*y + z*w);     A[0][2] = 2*(x*z - y*w);
    A[1][0] = 2*(y*x - z*w);     A[1][1] = 1 - 2*(x*x + z*z); A[1][2] = 2*(y*z + x*w);
    A[2][0] = 2*(z*x + y*w);     A[2][1] = 2*(z*y - x*w);     A[2][2] = 1 - 2*(x*x + y*y);
}

void adcs_fromdcm(adcs_real R[3][3], adcs_real q[4])
{
    /* the largest of (trace, R11, R22, R33) selects the stable form (as asils.quat.fromdcm) */
    adcs_real tr = R[0][0] + R[1][1] + R[2][2];
    int idx = 0; adcs_real best = tr;
    if (R[0][0] > best) { best = R[0][0]; idx = 1; }
    if (R[1][1] > best) { best = R[1][1]; idx = 2; }
    if (R[2][2] > best) { idx = 3; }
    switch (idx) {
    case 0:  q[0] = R[1][2] - R[2][1]; q[1] = R[2][0] - R[0][2]; q[2] = R[0][1] - R[1][0]; q[3] = 1 + tr; break;
    case 1:  q[0] = 1 + 2*R[0][0] - tr; q[1] = R[0][1] + R[1][0]; q[2] = R[0][2] + R[2][0]; q[3] = R[1][2] - R[2][1]; break;
    case 2:  q[0] = R[1][0] + R[0][1]; q[1] = 1 + 2*R[1][1] - tr; q[2] = R[1][2] + R[2][1]; q[3] = R[2][0] - R[0][2]; break;
    default: q[0] = R[2][0] + R[0][2]; q[1] = R[2][1] + R[1][2]; q[2] = 1 + 2*R[2][2] - tr; q[3] = R[0][1] - R[1][0]; break;
    }
    adcs_qnorm(q);
}

void adcs_fromrotvec(const adcs_real th[3], adcs_real q[4])
{
    adcs_real a = adcs_norm3(th);
    if (a < 1e-12) { q[0] = th[0]/2; q[1] = th[1]/2; q[2] = th[2]/2; q[3] = 1.0; adcs_qnorm(q); return; }
    q[0] = sin(a/2)*th[0]/a; q[1] = sin(a/2)*th[1]/a; q[2] = sin(a/2)*th[2]/a; q[3] = cos(a/2);
}

adcs_real adcs_qangle(const adcs_real a[4], const adcs_real b[4])
{
    adcs_real d = fabs(a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3]);
    return 2.0*acos(d > 1.0 ? 1.0 : d);
}

void adcs_qerr(const adcs_real q_ref[4], const adcs_real q[4], adcs_real qe[4])
{
    adcs_real c[4];
    adcs_qconj(q_ref, c);
    adcs_qmult(c, q, qe);
    if (qe[3] < 0) { qe[0] = -qe[0]; qe[1] = -qe[1]; qe[2] = -qe[2]; qe[3] = -qe[3]; }
}
