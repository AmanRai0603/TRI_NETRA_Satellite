/*
 * adcs_math.h -- fixed-size vector, matrix and quaternion kernel of the flight
 * software (fsw/pseudocode/01_math.md). C99, no allocation.
 * Quaternions are [x y z w] (scalar last), Hamilton; dcm(q) is the passive
 * ECI -> body matrix (00_conventions.md).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#ifndef ADCS_MATH_H
#define ADCS_MATH_H

#include <math.h>
#include <stdint.h>

#ifdef ADCS_REAL_FLOAT
typedef float adcs_real;
#else
typedef double adcs_real;
#endif

#define ADCS_PI 3.14159265358979323846
#define ADCS_D2R (ADCS_PI / 180.0)

adcs_real adcs_dot(const adcs_real a[3], const adcs_real b[3]);
void adcs_cross(const adcs_real a[3], const adcs_real b[3], adcs_real out[3]);
adcs_real adcs_norm3(const adcs_real a[3]);
void adcs_unit(const adcs_real a[3], adcs_real out[3]);
void adcs_scale3(const adcs_real a[3], adcs_real s, adcs_real out[3]);
void adcs_add3(const adcs_real a[3], const adcs_real b[3], adcs_real out[3]);
void adcs_sub3(const adcs_real a[3], const adcs_real b[3], adcs_real out[3]);
void adcs_copy3(const adcs_real a[3], adcs_real out[3]);
void adcs_zero3(adcs_real out[3]);
adcs_real adcs_clamp(adcs_real x, adcs_real lo, adcs_real hi);
adcs_real adcs_maxabs3(const adcs_real a[3]);
adcs_real adcs_sign(adcs_real x);

void adcs_mat3_vec(adcs_real M[3][3], const adcs_real v[3], adcs_real out[3]);
void adcs_mat3t_vec(adcs_real M[3][3], const adcs_real v[3], adcs_real out[3]);
void adcs_mat3_mul(adcs_real A[3][3], adcs_real B[3][3], adcs_real out[3][3]);
void adcs_mat3_transpose(adcs_real A[3][3], adcs_real out[3][3]);
void adcs_skew(const adcs_real a[3], adcs_real out[3][3]);
adcs_real adcs_det3(adcs_real M[3][3]);
int adcs_inv3(adcs_real M[3][3], adcs_real out[3][3]);
/* minimum-norm right inverse of a 3 x n matrix (n <= 8), rank safe: At (A At + eps I)^-1 */
void adcs_pinv_rows(adcs_real A[3][8], int n, adcs_real out[8][3]);
/* symmetric 4x4 eigen decomposition by cyclic Jacobi (12 sweeps); columns of V are eigenvectors */
void adcs_jacobi_eig4(adcs_real K[4][4], adcs_real lam[4], adcs_real V[4][4]);

void adcs_qmult(const adcs_real a[4], const adcs_real b[4], adcs_real out[4]);
void adcs_qconj(const adcs_real q[4], adcs_real out[4]);
void adcs_qnorm(adcs_real q[4]);
void adcs_dcm(const adcs_real q[4], adcs_real out[3][3]);
void adcs_fromdcm(adcs_real A[3][3], adcs_real q[4]);
void adcs_fromrotvec(const adcs_real th[3], adcs_real q[4]);
adcs_real adcs_qangle(const adcs_real a[4], const adcs_real b[4]);
void adcs_qerr(const adcs_real q_ref[4], const adcs_real q[4], adcs_real qe[4]);

#endif
