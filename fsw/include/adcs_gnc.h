/*
 * adcs_gnc.h -- estimation, guidance, control, magnetic laws and allocation of
 * the flight software (fsw/pseudocode/03..07). Pure functions over plain
 * structs; the mode manager (adcs_fsw.c) owns the state.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#ifndef ADCS_GNC_H
#define ADCS_GNC_H
#include "adcs_math.h"
#include "adcs_params.h"

/* ---- estimation (03) ---- */
typedef struct {
    adcs_real q[4], b[3], P[6][6];
    adcs_real arw, rrw;
} adcs_mekf_t;

void adcs_mekf_init(adcs_mekf_t *k, const adcs_real q0[4], adcs_real sig_att0, adcs_real sig_bias0, adcs_real arw, adcs_real rrw);
void adcs_mekf_predict(adcs_mekf_t *k, const adcs_real w_meas[3], adcs_real dt);
void adcs_mekf_vector(adcs_mekf_t *k, const adcs_real b_meas[3], const adcs_real r_ref[3], adcs_real sigma);
void adcs_mekf_quat(adcs_mekf_t *k, const adcs_real q_meas[4], adcs_real sig_cross, adcs_real sig_roll, const adcs_real bs[3]);
void adcs_triad(const adcs_real b1[3], const adcs_real b2[3], const adcs_real r1[3], const adcs_real r2[3], adcs_real q[4]);
/* q-method on n <= 16 vector pairs; returns the loss sum(w) - lambda_max */
adcs_real adcs_quest(adcs_real b[][3], adcs_real r[][3], const adcs_real *w, int n, adcs_real q[4]);
void adcs_latency(const adcs_real q_st[4], const adcs_real w[3], adcs_real lat, adcs_real q[4]);

/* ---- guidance (04) ---- */
typedef struct {
    int kind;                 /* 0 nadir, 1 target, 2 slew, 3 inertial, 4 sun */
    adcs_real q_off[4], roll_deg, t0, T, axis[3], q_inertial[4], sun_axis[3], roll_axis[3], sun_eci[3];
} adcs_guid_t;

void adcs_guidance(int kind, const adcs_real r[3], const adcs_real v[3], adcs_real t, const adcs_guid_t *g,
                   adcs_real q_ref[4], adcs_real w_ref[3], adcs_real wd_ref[3]);
void adcs_boresight_offset(const adcs_real bs[3], adcs_real q[4]);

/* ---- control (05) ---- */
typedef struct {
    int law;                  /* 0 pid, 1 lqr, 2 smc */
    adcs_real Kp[3], Kd[3], Ki[3], Klqr[3][3], lambda, phi, Gs[3], err_max, int_max;
} adcs_gains_t;

void adcs_control_law(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                      adcs_real I_q[3], adcs_real dt, const adcs_gains_t *g, adcs_real J[3][3],
                      const adcs_real Hs[3], const adcs_real wd_ref[3], adcs_real tau[3]);
void adcs_mtq_pd(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                 const adcs_gains_t *g, adcs_real tau[3]);
void adcs_torque2dipole(const adcs_real tau[3], const adcs_real B[3], adcs_real m_max, adcs_real m[3]);
/* magnetorquer-only literature laws (05_control.md) */
void adcs_mtq_lovera(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                     adcs_real J[3][3], adcs_real eps, adcs_real kp, adcs_real kv, adcs_real tau[3]);
void adcs_mtq_celani(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                     adcs_real eps, adcs_real k1, adcs_real k2, adcs_real tau[3]);
int adcs_mtq_avanzini(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                      adcs_real J[3][3], adcs_real k, adcs_real lam, adcs_real tau[3]);
void adcs_mtq_boresight(const adcs_real e3[3], const adcs_real a[3], const adcs_real we[3], adcs_real kp, adcs_real kd, adcs_real tau[3]);
void adcs_mtq_tango(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                    adcs_real Pth[3][3], adcs_real Pw[3][3], adcs_real tau[3]);
void adcs_sat_dipole(adcs_real m[3], adcs_real m_max);

/* ---- detumble and Sun spin (06) ---- */
void adcs_bdot(const adcs_real b1[3], const adcs_real b2[3], adcs_real dt, adcs_real Bnorm, adcs_real k, adcs_real m_max, adcs_real m[3]);
void adcs_gen_bdot(const adcs_real B[3], const adcs_real Bdot[3], const adcs_real wd[3], adcs_real k, adcs_real m0[3]);
void adcs_sun_spin(const adcs_real B[3], const adcs_real w[3], const adcs_real s[3], int eclipse, adcs_real J[3][3],
                   adcs_real spin_dps, adcs_real k1, adcs_real k2, adcs_real rz_floor, adcs_real m0[3]);
void adcs_sun_spin_deruiter(const adcs_real B[3], const adcs_real w[3], const adcs_real s[3], int eclipse, adcs_real J[3][3],
                            adcs_real spin_dps, adcs_real k, adcs_real k1, adcs_real k2, adcs_real m0[3]);

/* ---- allocation (07) ---- */
void adcs_rotor_axes(const adcs_params_t *p, const adcs_real delta[4], adcs_real A[3][8]);
void adcs_steer_sr(const adcs_real tau[3], adcs_real A[3][8], const adcs_real h[8], const adcs_params_t *p,
                   int wheels, adcs_real gdot[4], adcs_real hdot[8]);
void adcs_dump(const adcs_real Hdev[3], const adcs_real Ht[3], const adcs_real B[3], adcs_real k, adcs_real m_max, adcs_real m[3]);
/* thruster couples: on-time fraction per couple over T and the torque fed forward */
void adcs_rcs_duty(const adcs_real req[3], const adcs_params_t *p, adcs_real T, adcs_real duty[6], adcs_real tau[3]);

#endif
