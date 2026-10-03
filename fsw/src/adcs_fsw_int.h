/*
 * adcs_fsw_int.h -- private to the flight software: its state, shared by the shell (adcs_fsw.c)
 * and the parts it calls each tick (adcs_guid.c, adcs_modes.c, adcs_fdir.c). Not part of the ABI.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#ifndef ADCS_FSW_INT_H
#define ADCS_FSW_INT_H
#include <math.h>
#include "adcs_fsw.h"
#include "adcs_params.h"
#include "adcs_gnc.h"
#include "adcs_env.h"
#include "adcs_drv.h"

#define NR ADCS_MAX_ROTORS
#define NG ADCS_MAX_GIMBALS
#define NC ADCS_MAX_COUPLES

typedef struct {
    adcs_params_t p;
    int ready;
    uint64_t start_ns;
    double t, jd;
    uint8_t mode;
    double t_mode, hold;
    /* sensors (latest) */
    adcs_meas_t z;
    int clean;                           /* coils were off during the last tick */
    /* sensor health: seconds since the last good sample, the last good field */
    double mag_age, gyro_age; int mag_seen; adcs_real B_good[3];
    /* orbit */
    int have_r; adcs_real r[3], v[3];
    /* estimation */
    adcs_mekf_t K; int ad_ok; double t_st; int mag_done, n_rej; double gh_jd;
    adcs_real es_n[3]; double es_t;           /* latest valid Earth-sensor sample and its time */
    adcs_real w_est[3];
    adcs_real gh[195]; int gh_ok; adcs_real Bref[3]; int bref_ok; double t_Bref;
    /* coil cycle */
    adcs_real bsum[3], bsum_raw[3]; int bn;
    adcs_real b1[3]; int b1_ok; adcs_real B1raw[3]; int B1raw_ok;
    adcs_real m_hold[3], B_dump[3];
    /* Sun spin / acquisition */
    adcs_real sigma, sz_sum; int sz_n; double sz_t0;
    adcs_real s_prop[3]; int s_prop_ok; double acq_hold;
    int ho; double ho_t;                      /* magnetic pointing hand-over (P11 despin) and its dwell */
    /* fine modes */
    adcs_guid_t gd;
    adcs_gains_t g_rw, g_mtq;
    adcs_real q_ref[4], w_ref[3], tau_req[3], I_q[3]; double last_ctrl; int capturing;
    adcs_real h_prev[NR]; int h_prev_ok; adcs_real cmd_r_prev[NR]; int rot_failed[NR]; adcs_real fd_count[NR];
    adcs_real fw_E[NR], fw_h0[NR]; double fw_t0, fw_last; int fw_on, fw_bad[NR];   /* windowed rotor FDIR */
    adcs_real h_t_rot[NR], H_t[3], cap[3], hcap[3], dump_hi, dump_lo;
    int has_rcs_dump, rcs_dumping; adcs_real rcs_left[NC]; int rcs_left_ok;
    int sched_i;
    /* last commands (peek) */
    adcs_real out_m[3], out_r[NR], out_g[NG], out_duty[NC];
    uint16_t faults;
} fsw_t;

/* fault bits a sensor silent too long sets (adcs_fdir.c); the mode manager reads them */
#define FAULT_MAG_STALE  (1u << 8)
#define FAULT_GYRO_STALE (1u << 9)
/* the coils act on a held field for at most this many coil cycles (adcs_fdir.c holds it) */
#define MAG_HOLD_CYCLES 2.0

/* guidance (adcs_guid.c) */
int adcs_guid_kind(uint8_t mode);

/* mode manager (adcs_modes.c) */
int adcs_modes_feasible(const adcs_params_t *p, unsigned m);
void adcs_modes_enter(fsw_t *s, uint8_t mode);
void adcs_modes_schedule(fsw_t *s);
void adcs_modes_step(fsw_t *s, double dt);

/* FDIR (adcs_fdir.c) */
void adcs_fdir_sensors(fsw_t *s, double dt);
void adcs_fdir_safe(fsw_t *s);
void adcs_fdir_rotors(fsw_t *s, double dt);

#endif
