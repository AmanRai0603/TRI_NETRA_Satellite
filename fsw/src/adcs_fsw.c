/*
 * adcs_fsw.c -- the reference flight software: adcs_fsw.h over adcs_hal.h.
 * Tick order and mode manager: fsw/pseudocode/08_mode_manager.md; one branch per
 * controller state, as asils.fsw.step (the MATLAB twin) and fsw-rs (Rust).
 * All state is in one static struct; adcs_fsw_init resets every field.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#include <string.h>
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
    /* orbit */
    int have_r; adcs_real r[3], v[3];
    /* estimation */
    adcs_mekf_t K; int ad_ok; double t_st;
    adcs_real w_est[3];
    adcs_real gh[195]; int gh_ok; adcs_real Bref[3]; int bref_ok; double t_Bref;
    /* coil cycle */
    adcs_real bsum[3], bsum_raw[3]; int bn;
    adcs_real b1[3]; int b1_ok; adcs_real B1raw[3]; int B1raw_ok;
    adcs_real m_hold[3], B_dump[3];
    /* Sun spin / acquisition */
    adcs_real sigma, sz_sum; int sz_n; double sz_t0;
    adcs_real s_prop[3]; int s_prop_ok; double acq_hold;
    /* fine modes */
    adcs_guid_t gd;
    adcs_gains_t g_rw, g_mtq;
    adcs_real q_ref[4], w_ref[3], tau_req[3], I_q[3]; double last_ctrl; int capturing;
    adcs_real h_prev[NR]; int h_prev_ok; adcs_real cmd_r_prev[NR]; int rot_failed[NR]; adcs_real fd_count[NR];
    adcs_real h_t_rot[NR], H_t[3], cap[3], hcap[3], dump_hi, dump_lo;
    int has_rcs_dump, rcs_dumping; adcs_real rcs_left[NC]; int rcs_left_ok;
    int sched_i;
    /* last commands (peek) */
    adcs_real out_m[3], out_r[NR], out_g[NG], out_duty[NC];
    uint16_t faults;
} fsw_t;

static fsw_t S;

static int guid_kind_of(uint8_t mode)
{
    switch (mode) {
    case ADCS_MODE_TARGET_FINE: return 1;
    case ADCS_MODE_SLEW_FINE: return 2;
    case ADCS_MODE_SUN_MTQ: case ADCS_MODE_SUN_FINE: return 4;
    default: return 0;
    }
}

static void enter(uint8_t mode)
{
    if (mode == ADCS_MODE_SPINUP) { S.sz_sum = 0; S.sz_n = 0; S.sz_t0 = S.t; }
    S.mode = mode; S.t_mode = S.t; S.hold = 0;
    adcs_zero3(S.I_q);
}

static void set_gains(adcs_gains_t *g, int law, const double Kp[3], const double Kd[3], const double Ki[3], double Klqr[3][3],
                      double lambda, double phi, const double Gs[3], double err_max, double int_max)
{
    int i, j;
    g->law = law;
    for (i = 0; i < 3; i++) {
        g->Kp[i] = Kp[i]; g->Kd[i] = Kd[i]; g->Ki[i] = Ki[i]; g->Gs[i] = Gs[i];
        for (j = 0; j < 3; j++) g->Klqr[i][j] = Klqr[i][j];
    }
    g->lambda = lambda; g->phi = phi; g->err_max = err_max; g->int_max = int_max;
}

int32_t adcs_fsw_init(const adcs_fsw_init_t *init)
{
    adcs_params_t *p;
    int i, k;
    double hmin = 1e300;
    memset(&S, 0, sizeof S);
    adcs_drv_reset();
    if (!init || init->abi_version != ADCS_FSW_ABI_VERSION) return -10;
    if (adcs_params_decode(init->config_blob, init->config_len, &S.p) != 0) return -11;
    p = &S.p;
    S.start_ns = init->start_ns;
    S.mode = p->start_mode; S.t_st = -1e9; S.t_Bref = -1e9; S.last_ctrl = -1e9;
    S.sigma = p->ss_sigma0; S.clean = 1;
    for (i = 0; i < 4; i++) { S.q_ref[i] = 0; S.gd.q_off[i] = p->gd_q_off[i]; S.gd.q_inertial[i] = p->gd_q_inertial[i]; }
    for (i = 0; i < 3; i++) {
        S.gd.axis[i] = p->gd_axis[i]; S.gd.sun_axis[i] = p->sun_axis[i]; S.gd.roll_axis[i] = p->roll_axis[i];
    }
    S.gd.roll_deg = p->gd_roll_deg; S.gd.t0 = p->gd_t0; S.gd.T = p->gd_T;
    adcs_sun_model(p->jd0, S.gd.sun_eci);
    set_gains(&S.g_rw, p->rw_law, p->rw_Kp, p->rw_Kd, p->rw_Ki, p->rw_Klqr, p->rw_lambda, p->rw_phi, p->rw_Gs, p->rw_err_max, p->rw_int_max);
    set_gains(&S.g_mtq, p->mtq_law == 1 ? 1 : (p->mtq_law == 2 ? 2 : 0), p->mtq_Kp, p->mtq_Kd, p->mtq_Ki, p->mtq_Klqr,
              p->mtq_lambda, p->mtq_phi, p->mtq_Gs, p->mtq_err_max, p->mtq_int_max);
    /* momentum targets and per-axis authority (asils.fsw.init) */
    for (i = 0; i < p->nr; i++) {
        if (p->rot_kind[i] == 0) S.h_t_rot[i] = (p->h_bias < 0.25*p->rot_hmax[i]) ? p->h_bias : 0.25*p->rot_hmax[i];
        else if (p->rot_kind[i] >= 2) S.h_t_rot[i] = p->rot_h0[i];
        if (p->rot_gi[i] == 0) {
            for (k = 0; k < 3; k++) {
                S.cap[k] += fabs(p->rot_a0[i][k])*p->rot_tmax[i];
                S.hcap[k] += fabs(p->rot_a0[i][k])*p->rot_hmax[i];
                S.H_t[k] += p->rot_a0[i][k]*S.h_t_rot[i];
            }
        }
    }
    if (p->ng > 0) {
        double h0 = 0;
        for (i = 0; i < p->nr; i++) if (p->rot_h0[i] > h0) h0 = p->rot_h0[i];
        for (k = 0; k < 3; k++) { S.cap[k] += 2*h0*p->gim_rate_max; S.hcap[k] += 2.5*h0; }
    }
    S.dump_hi = p->rcs_dump_hi; S.dump_lo = p->rcs_dump_lo;
    if (p->nr > 0) {
        for (k = 0; k < 3; k++) if (S.hcap[k] < hmin) hmin = S.hcap[k];
        if (0.5*hmin < S.dump_hi) S.dump_hi = 0.5*hmin;
        if (0.15*hmin < S.dump_lo) S.dump_lo = 0.15*hmin;
    }
    S.has_rcs_dump = p->nc > 0 && p->rcs_dump;
    S.ready = 1;
    return 0;
}

/* ---------------- helpers of the step ---------------- */
static void mtq_law(void)
{
    const adcs_params_t *p = &S.p;
    adcs_real zero[3] = {0, 0, 0};
    if (p->mtq_law == 0) {
        adcs_mtq_pd(S.K.q, S.w_est, S.q_ref, S.w_ref, &S.g_mtq, S.tau_req);
    } else if (p->mtq_law == 3) {             /* rate damping relative to the reference */
        adcs_real qc[4], qe[4], A[3][3], wr[3]; int i;
        adcs_qconj(S.q_ref, qc); adcs_qmult(qc, S.K.q, qe);
        adcs_dcm(qe, A); adcs_mat3_vec(A, S.w_ref, wr);
        for (i = 0; i < 3; i++) S.tau_req[i] = -S.g_mtq.Kd[i]*(S.w_est[i] - wr[i]);
    } else {
        adcs_control_law(S.K.q, S.w_est, S.q_ref, S.w_ref, S.I_q, p->mtq_period, &S.g_mtq, S.p.J, zero, zero, S.tau_req);
    }
}

static int capture_law(const adcs_real Hdev[3])
{
    adcs_params_t *p = &S.p;
    adcs_real qe[4], th, e[3], Jm, alpha, wmax, wref[3], wc[3], A[3][3], Jw[3], H[3], gy[3], x[3], kr, mincap, minh, sp;
    int i;
    if (S.mode == ADCS_MODE_SLEW_FINE || p->capture_deg <= 0) return 0;
    adcs_qerr(S.q_ref, S.K.q, qe);
    th = 2*acos(qe[3] > 1 ? 1 : qe[3]);
    if (th < p->capture_deg*ADCS_D2R) return 0;
    { adcs_real n = adcs_norm3(qe); if (n < 1e-12) n = 1e-12; adcs_scale3(qe, 1.0/n, e); }
    Jm = p->J[0][0]; if (p->J[1][1] > Jm) Jm = p->J[1][1]; if (p->J[2][2] > Jm) Jm = p->J[2][2];
    mincap = S.cap[0]; minh = S.hcap[0];
    for (i = 1; i < 3; i++) { if (S.cap[i] < mincap) mincap = S.cap[i]; if (S.hcap[i] < minh) minh = S.hcap[i]; }
    alpha = 0.5*mincap/Jm;
    wmax = p->capture_rate_deg_s*ADCS_D2R; if (0.5*minh/Jm < wmax) wmax = 0.5*minh/Jm;
    adcs_dcm(qe, A); adcs_mat3_vec(A, S.w_ref, wref);
    sp = sqrt(2*alpha*th); if (wmax < sp) sp = wmax;
    for (i = 0; i < 3; i++) wc[i] = wref[i] - e[i]*sp;
    kr = 4*alpha/(wmax > 1e-6 ? wmax : 1e-6); if (kr > 0.5) kr = 0.5;
    adcs_mat3_vec(p->J, S.w_est, Jw); adcs_add3(Jw, Hdev, H); adcs_cross(S.w_est, H, gy);
    for (i = 0; i < 3; i++) x[i] = kr*(wc[i] - S.w_est[i]);
    adcs_mat3_vec(p->J, x, S.tau_req);
    adcs_add3(S.tau_req, gy, S.tau_req);
    return 1;
}

static void sun_acq_law(const adcs_real Hdev[3])
{
    adcs_params_t *p = &S.p;
    adcs_real wmax = p->sa_w_max_deg_s*ADCS_D2R, wc[3] = {0, 0, 0}, s[3], c[3], a[3], Jw[3], H[3], gy[3], x[3], n;
    int i;
    adcs_copy3(p->sun_axis, a);
    if (S.s_prop_ok) {
        adcs_unit(S.s_prop, s);
        adcs_cross(a, s, c);
        if (adcs_dot(s, a) < -0.95) {
            adcs_real ex[3] = {1, 0, 0}, ey[3] = {0, 1, 0};
            adcs_cross(a, ex, c); if (adcs_norm3(c) < 0.1) adcs_cross(a, ey, c);
            adcs_unit(c, c);
        }
        adcs_scale3(c, wmax/0.5, wc);
        n = adcs_norm3(wc); if (n > wmax) adcs_scale3(wc, wmax/n, wc);
    }
    adcs_mat3_vec(p->J, S.w_est, Jw); adcs_add3(Jw, Hdev, H); adcs_cross(S.w_est, H, gy);
    for (i = 0; i < 3; i++) x[i] = p->sa_kd*(wc[i] - S.w_est[i]);
    adcs_mat3_vec(p->J, x, S.tau_req);
    adcs_add3(S.tau_req, gy, S.tau_req);
}

static void spin_guards(double dt)
{
    const adcs_params_t *p = &S.p;
    double d = ADCS_D2R, wz = S.w_est[2], wp = sqrt(S.w_est[0]*S.w_est[0] + S.w_est[1]*S.w_est[1]);
    if (adcs_norm3(S.w_est) > p->ss_omega_max_dps*d) { enter(ADCS_MODE_DETUMBLE); return; }
    if (S.mode == ADCS_MODE_SPINUP) {
        int conv = fabs(wz - S.sigma*p->ss_spin_dps*d) < p->ss_z_in_dps*d && wp < p->ss_perp_in_dps*d, ok;
        if (conv && S.z.sun_ok) { S.sz_sum += S.z.sun[2]; S.sz_n += 1; }
        if (S.t - S.sz_t0 >= p->ss_t_check_s && S.sz_n > 0 && S.sz_sum/S.sz_n > p->ss_sun_min) {
            S.sigma = -S.sigma; S.sz_sum = 0; S.sz_n = 0; S.sz_t0 = S.t;
        }
        ok = conv && S.z.sun_ok && S.z.sun[2] < 0;
        if (ok) S.hold += dt; else S.hold = 0;
        if (S.hold >= p->ss_dwell_in_s) enter(ADCS_MODE_SUN_SPIN);
    } else {
        int bad = fabs(wz) < p->ss_omega_exit_dps*d || wp > p->ss_perp_out_dps*d;
        if (bad) S.hold += dt; else S.hold = 0;
        if (S.hold >= p->ss_dwell_out_s) enter(ADCS_MODE_SPINUP);
    }
}

static void allocate(const adcs_real tau_rot[3], adcs_real A[3][8], adcs_real cmd_r[NR], adcs_real cmd_g[NG])
{
    const adcs_params_t *p = &S.p;
    adcs_real Af[3][8], Pi[8][3];
    int fixed[NR], nf = 0, i, k;
    for (i = 0; i < p->nr; i++) if (p->rot_gi[i] == 0 && !S.rot_failed[i]) fixed[nf++] = i;
    if (nf > 0) {
        for (i = 0; i < nf; i++) for (k = 0; k < 3; k++) Af[k][i] = A[k][fixed[i]];
        adcs_pinv_rows(Af, nf, Pi);
        for (i = 0; i < nf; i++) cmd_r[fixed[i]] = -(Pi[i][0]*tau_rot[0] + Pi[i][1]*tau_rot[1] + Pi[i][2]*tau_rot[2]);
    }
    if (p->ng > 0) {
        adcs_real hdot[NR];
        int wheels = 0;
        for (i = 0; i < p->nr; i++) if (p->rot_kind[i] == 3) wheels = 1;
        adcs_steer_sr(tau_rot, A, S.z.h, p, wheels, cmd_g, hdot);
        if (wheels)
            for (i = 0; i < p->nr; i++) if (p->rot_gi[i] > 0) cmd_r[i] = hdot[i] - p->cmg_k_null*(S.z.h[i] - p->rot_h0[i]);
    }
}

static void idle_rotors(adcs_real cmd_r[NR], int zero_cmg)
{
    int i;
    for (i = 0; i < S.p.nr; i++) {
        cmd_r[i] = -0.2*(S.z.h[i] - S.h_t_rot[i]);
        if (zero_cmg && S.p.rot_gi[i] > 0 && S.p.rot_kind[i] == 2) cmd_r[i] = 0;
    }
}

/* ---------------- one tick ---------------- */
int32_t adcs_fsw_step(uint64_t now_ns)
{
    adcs_params_t *p = &S.p;
    adcs_meas_t *z = &S.z;
    double dt, phase;
    int first, i, nr, ng;
    adcs_real m_body[3], cmd_r[NR], cmd_g[NG], duty[NC];
    if (!S.ready) return -1;
    dt = p->dt; nr = p->nr; ng = p->ng;
    S.t = (double)(now_ns - S.start_ns)*1e-9;
    S.jd = p->jd0 + S.t/86400.0;
    adcs_drv_read(p, z);
    if (!z->gyro_ok && !p->has_gyro) adcs_zero3(z->w);

    /* 1 onboard orbit */
    if (z->gps_ok) { adcs_copy3(z->r, S.r); adcs_copy3(z->v, S.v); S.have_r = 1; }
    else if (S.have_r) {
        adcs_real rn = adcs_norm3(S.r), a[3];
        adcs_scale3(S.r, -p->mu/(rn*rn*rn), a);
        for (i = 0; i < 3; i++) { S.r[i] += S.v[i]*dt + 0.5*a[i]*dt*dt; S.v[i] += a[i]*dt; }
    }

    /* 2 estimation */
    adcs_sun_model(S.jd, S.gd.sun_eci);
    phase = fmod(S.t + 1e-9, p->mtq_period);
    first = phase < dt - 1e-9;
    if (first && S.have_r && (S.t - S.t_Bref) >= 0.999) {
        if (!S.gh_ok) { adcs_igrf_gh(adcs_decyear(S.jd), S.gh); S.gh_ok = 1; }
        adcs_field_eci(S.r, S.jd, S.gh, p->igrf_nmax, S.Bref); S.bref_ok = 1; S.t_Bref = S.t;
    }
    if (!(S.mode == ADCS_MODE_DETUMBLE || S.mode == ADCS_MODE_DETUMBLE_RCS || S.mode == ADCS_MODE_SPINUP || S.mode == ADCS_MODE_SUN_SPIN)) {
        if (!S.ad_ok) {
            if (p->has_st && z->st_ok) {
                adcs_real q0[4]; int h = 0;
                while (h < p->n_heads && !z->st_valid[h]) h++;
                adcs_latency(z->q_st[h], z->w, p->st_latency, q0);
                adcs_mekf_init(&S.K, q0, 1e-3, 2e-4, p->gyro_arw, p->gyro_rrw);
                S.ad_ok = 1; S.t_st = S.t;
            } else if (z->sun_ok && S.clean && S.bref_ok) {
                adcs_real q0[4];
                adcs_triad(z->sun, z->B, S.gd.sun_eci, S.Bref, q0);
                adcs_mekf_init(&S.K, q0, 0.05, 2e-4, p->gyro_arw, p->gyro_rrw);
                S.ad_ok = 1;
            }
        } else {
            adcs_mekf_predict(&S.K, z->w, dt);
            if (p->has_st && z->st_ok) {
                int h;
                for (h = 0; h < p->n_heads; h++) {
                    if (z->st_valid[h]) {
                        adcs_real wb[3], ql[4], bs[3];
                        adcs_sub3(z->w, S.K.b, wb);
                        adcs_latency(z->q_st[h], wb, p->st_latency, ql);
                        for (i = 0; i < 3; i++) bs[i] = p->st_bs[h][i];
                        adcs_mekf_quat(&S.K, ql, p->st_noise_cross*p->mekf_meas_scale, p->st_noise_roll*p->mekf_meas_scale, bs);
                    }
                }
                S.t_st = S.t;
            } else if (p->has_st && S.t - S.t_st < p->st_coast_s) {
                /* short star-tracker outage: coast on the gyro */
            } else {
                if (z->sun_ok && first) adcs_mekf_vector(&S.K, z->sun, S.gd.sun_eci, p->mekf_sig_sun);
                if (S.clean && first && S.bref_ok) adcs_mekf_vector(&S.K, z->B, S.Bref, p->mekf_sig_mag);
                if (z->es_ok && first && S.have_r) {
                    adcs_real nr_[3], sg = p->es_noise > 1e-3 ? p->es_noise : 1e-3;
                    adcs_scale3(S.r, -1.0, nr_);
                    adcs_mekf_vector(&S.K, z->nadir, nr_, 2*sg);
                }
            }
        }
    }
    {   /* rate for the controllers: bias removed, first-order low-pass */
        adcs_real wr[3], a = dt/(p->rate_lpf_s + dt);
        if (S.ad_ok) adcs_sub3(z->w, S.K.b, wr); else adcs_copy3(z->w, wr);
        if (p->rate_lpf_s <= 0) adcs_copy3(wr, S.w_est);
        else for (i = 0; i < 3; i++) S.w_est[i] += a*(wr[i] - S.w_est[i]);
    }

    /* 3 mode manager */
    while (S.sched_i < p->n_sched && S.t >= p->sched_t[S.sched_i]) { enter(p->sched_mode[S.sched_i]); S.sched_i++; }
    if ((S.mode == ADCS_MODE_DETUMBLE || S.mode == ADCS_MODE_DETUMBLE_RCS) && p->auto_next != ADCS_MODE_NONE) {
        if (adcs_norm3(z->w) < p->detumble_exit) S.hold += dt; else S.hold = 0;
        if (S.hold >= p->detumble_hold_s) enter(p->auto_next);
    }
    if (S.mode == ADCS_MODE_SPINUP || S.mode == ADCS_MODE_SUN_SPIN) spin_guards(dt);
    if (S.mode == ADCS_MODE_SPINUP || S.mode == ADCS_MODE_SUN_SPIN || S.mode == ADCS_MODE_SUN_ACQ_ROTOR) {
        if (z->sun_ok) { adcs_copy3(z->sun, S.s_prop); S.s_prop_ok = 1; }
        else if (S.s_prop_ok) {
            adcs_real th[3], dq[4], A[3][3];
            adcs_scale3(S.w_est, dt, th); adcs_fromrotvec(th, dq); adcs_dcm(dq, A);
            adcs_mat3_vec(A, S.s_prop, S.s_prop); adcs_unit(S.s_prop, S.s_prop);
        }
    }
    if (S.mode == ADCS_MODE_SUN_ACQ_ROTOR && p->auto_next != ADCS_MODE_NONE) {
        int ok = z->sun_ok && acos(adcs_clamp(adcs_dot(z->sun, p->sun_axis), -1, 1)) < p->sa_done_deg*ADCS_D2R && S.ad_ok;
        if (ok) S.acq_hold += dt; else S.acq_hold = 0;
        if (S.acq_hold >= p->sa_done_hold_s) enter(p->auto_next);
    }

    /* 4 guidance / control / commands */
    adcs_copy3(S.m_hold, m_body);
    for (i = 0; i < NR; i++) cmd_r[i] = 0;
    for (i = 0; i < NG; i++) cmd_g[i] = 0;
    for (i = 0; i < NC; i++) duty[i] = 0;
    switch (S.mode) {
    case ADCS_MODE_DETUMBLE:
        if (phase < p->mtq_meas + dt/2) {
            adcs_zero3(m_body);
            if (first) { adcs_zero3(S.bsum); adcs_zero3(S.bsum_raw); S.bn = 0; }
            {   adcs_real u[3]; adcs_unit(z->B, u); adcs_add3(S.bsum, u, S.bsum); adcs_add3(S.bsum_raw, z->B, S.bsum_raw); S.bn++; }
            if (fabs(phase - p->mtq_meas) < dt/2) {
                adcs_real b[3];
                int law = p->bdot_law;
                adcs_scale3(S.bsum, 1.0/S.bn, b); adcs_unit(b, b);
                if (law == 0 && !p->has_gyro) law = 1;
                if (law == 0) {
                    adcs_real bd[3], bm[3];
                    adcs_cross(z->w, b, bd); adcs_scale3(bd, -1.0, bd);
                    adcs_sub3(b, bd, bm);
                    adcs_bdot(bm, b, 1.0, adcs_norm3(z->B), p->bdot_k, p->m_max, m_body);
                } else if (law == 1) {
                    if (S.b1_ok) adcs_bdot(S.b1, b, p->mtq_period, adcs_norm3(z->B), p->bdot_k, p->m_max, m_body);
                } else if (law == 2) {
                    if (S.b1_ok) for (i = 0; i < 3; i++) {
                        adcs_real bd = (b[i] - S.b1[i])/p->mtq_period;
                        m_body[i] = (fabs(bd) > 1e-4) ? -p->m_max*adcs_sign(bd) : 0.0;
                    }
                } else {
                    adcs_real Bav[3], Bd[3], zero[3] = {0, 0, 0}, mx;
                    adcs_scale3(S.bsum_raw, 1.0/S.bn, Bav);
                    if (S.B1raw_ok) {
                        for (i = 0; i < 3; i++) Bd[i] = (Bav[i] - S.B1raw[i])/p->mtq_period;
                        adcs_gen_bdot(Bav, Bd, zero, p->ss_k_l1, m_body);
                        mx = adcs_maxabs3(m_body); if (mx < 1e-30) mx = 1e-30;
                        adcs_scale3(m_body, (p->m_max/mx < 1) ? p->m_max/mx : 1.0, m_body);
                    }
                    adcs_copy3(Bav, S.B1raw); S.B1raw_ok = 1;
                }
                if (m_body[0] != 0 || m_body[1] != 0 || m_body[2] != 0) {
                    adcs_sub3(m_body, p->m_res_est, m_body); adcs_sat_dipole(m_body, p->m_max);
                }
                adcs_copy3(b, S.b1); S.b1_ok = 1;
            }
        }
        if (nr > 0) idle_rotors(cmd_r, 1);
        break;

    case ADCS_MODE_DETUMBLE_RCS: {
        double Tc = p->rcsd_period_s;
        if (p->nc > 0 && (!S.rcs_left_ok || fmod(S.t + 1e-9, Tc) < dt - 1e-9)) {
            for (i = 0; i < NC; i++) S.rcs_left[i] = 0;
            S.rcs_left_ok = 1;
            if (adcs_norm3(S.w_est) > p->rcsd_deadband_deg_s*ADCS_D2R) {
                adcs_real x[3], dc[NC], tf[3];
                adcs_scale3(S.w_est, -1.0/p->rcsd_T_damp_s, x);
                adcs_mat3_vec(p->J, x, S.tau_req);
                adcs_rcs_duty(S.tau_req, p, Tc, dc, tf);
                for (i = 0; i < NC; i++) S.rcs_left[i] = dc[i]*Tc;
            }
        }
        if (p->nc > 0) for (i = 0; i < NC; i++) {
            duty[i] = S.rcs_left[i]/dt; if (duty[i] > 1) duty[i] = 1;
            S.rcs_left[i] -= dt; if (S.rcs_left[i] < 0) S.rcs_left[i] = 0;
        }
        adcs_zero3(m_body);
        if (nr > 0) idle_rotors(cmd_r, 1);
        break; }

    case ADCS_MODE_NADIR_MTQ:
    case ADCS_MODE_SUN_MTQ:
        if (first) adcs_zero3(m_body);
        else if (fabs(phase - p->mtq_meas) < dt/2 && S.ad_ok && S.have_r) {
            adcs_real wd[3];
            adcs_guidance(guid_kind_of(S.mode), S.r, S.v, S.t, &S.gd, S.q_ref, S.w_ref, wd);
            mtq_law();
            adcs_torque2dipole(S.tau_req, z->B, p->m_max, m_body);
            adcs_sub3(m_body, p->m_res_est, m_body); adcs_sat_dipole(m_body, p->m_max);
        } else if (phase < p->mtq_meas) adcs_zero3(m_body);
        break;

    case ADCS_MODE_NADIR_FINE: case ADCS_MODE_TARGET_FINE: case ADCS_MODE_SLEW_FINE:
    case ADCS_MODE_SUN_FINE: case ADCS_MODE_SUN_ACQ_ROTOR: {
        int acq = S.mode == ADCS_MODE_SUN_ACQ_ROTOR, ctl_ok = S.ad_ok || acq;
        adcs_real A[3][8], Hdev[3] = {0, 0, 0}, dH[3], tau_rcs[3] = {0, 0, 0}, tau_coil[3] = {0, 0, 0}, tr[3];
        adcs_rotor_axes(p, z->delta, A);
        for (i = 0; i < nr; i++) { Hdev[0] += A[0][i]*z->h[i]; Hdev[1] += A[1][i]*z->h[i]; Hdev[2] += A[2][i]*z->h[i]; }
        /* FDIR on fixed rotors */
        if (nr > 0 && S.h_prev_ok) {
            for (i = 0; i < nr; i++) {
                adcs_real tmax = p->rot_tmax[i], meas = (z->h[i] - S.h_prev[i])/dt;
                adcs_real expect = adcs_clamp(S.cmd_r_prev[i], -0.8*tmax, 0.8*tmax);
                int bad = fabs(meas - expect) > 0.5*tmax && p->rot_gi[i] == 0 && fabs(expect) > 0.2*tmax && fabs(z->h[i]) < 0.9*p->rot_hmax[i];
                S.fd_count[i] = bad ? S.fd_count[i] + dt : 0;
                if (S.fd_count[i] > p->fdir_s && !S.rot_failed[i]) { S.rot_failed[i] = 1; S.faults |= (uint16_t)(1u << i); }
            }
        }
        /* control law at the control rate */
        if (acq && S.t - S.last_ctrl >= p->rw_dt - 1e-9) { sun_acq_law(Hdev); S.last_ctrl = S.t; }
        else if (!acq && S.ad_ok && S.have_r && S.t - S.last_ctrl >= p->rw_dt - 1e-9) {
            adcs_real wd[3];
            adcs_guidance(guid_kind_of(S.mode), S.r, S.v, S.t, &S.gd, S.q_ref, S.w_ref, wd);
            S.capturing = capture_law(Hdev);
            if (S.capturing) adcs_zero3(S.I_q);
            else adcs_control_law(S.K.q, S.w_est, S.q_ref, S.w_ref, S.I_q, p->rw_dt, &S.g_rw, p->J, Hdev, wd, S.tau_req);
            S.last_ctrl = S.t;
        }
        /* momentum management */
        adcs_sub3(Hdev, S.H_t, dH);
        if (S.has_rcs_dump) {
            if (adcs_norm3(dH) > S.dump_hi) S.rcs_dumping = 1;
            else if (adcs_norm3(dH) < S.dump_lo) S.rcs_dumping = 0;
        }
        if (first) adcs_zero3(m_body);
        else if (fabs(phase - p->mtq_meas) < dt/2) {
            adcs_real m[3] = {0, 0, 0}, mi[3];
            if (!S.has_rcs_dump) adcs_dump(Hdev, S.H_t, z->B, p->dump_k, p->m_max, m);
            if (p->alloc == 1 && ctl_ok) { adcs_torque2dipole(S.tau_req, z->B, p->m_max, mi); adcs_add3(m, mi, m); }
            if (S.ad_ok) {
                int anyf = 0, nf = 0, fx[NR];
                for (i = 0; i < nr; i++) { if (S.rot_failed[i]) anyf = 1; else if (p->rot_gi[i] == 0) fx[nf++] = i; }
                if (anyf) {
                    adcs_real un[3], Af[3][8], Pi[8][3], y[8];
                    int k;
                    if (nf == 0) adcs_copy3(S.tau_req, un);
                    else {
                        for (i = 0; i < nf; i++) for (k = 0; k < 3; k++) Af[k][i] = A[k][fx[i]];
                        adcs_pinv_rows(Af, nf, Pi);
                        for (i = 0; i < nf; i++) y[i] = Pi[i][0]*S.tau_req[0] + Pi[i][1]*S.tau_req[1] + Pi[i][2]*S.tau_req[2];
                        for (k = 0; k < 3; k++) { un[k] = S.tau_req[k]; for (i = 0; i < nf; i++) un[k] -= Af[k][i]*y[i]; }
                    }
                    adcs_torque2dipole(un, z->B, p->m_max, mi); adcs_add3(m, mi, m);
                }
            }
            adcs_sub3(m, p->m_res_est, m_body); adcs_sat_dipole(m_body, p->m_max);
            adcs_copy3(z->B, S.B_dump);
        } else if (phase < p->mtq_meas) adcs_zero3(m_body);
        /* thrusters: assist and dumping */
        if (p->nc > 0 && ctl_ok) {
            adcs_real req[3] = {0, 0, 0};
            if (p->rcs_assist) {
                for (i = 0; i < 3; i++) {
                    adcs_real ex = fabs(S.tau_req[i]) - p->rcs_assist_frac*S.cap[i];
                    req[i] = adcs_sign(S.tau_req[i])*(ex > 0 ? ex : 0);
                    if (fabs(Hdev[i]) > p->rcs_assist_frac*S.hcap[i] && adcs_sign(-S.tau_req[i]) == adcs_sign(Hdev[i])) req[i] = S.tau_req[i];
                }
            }
            if (S.rcs_dumping) for (i = 0; i < 3; i++) req[i] -= p->rcs_dump_k*dH[i];
            adcs_rcs_duty(req, p, dt, duty, tau_rcs);
        }
        /* momentum devices deliver the rest (coil and thruster torques fed forward) */
        if (m_body[0] != 0 || m_body[1] != 0 || m_body[2] != 0) {
            adcs_real mm[3]; adcs_add3(m_body, p->m_res_est, mm); adcs_cross(mm, S.B_dump, tau_coil);
        }
        if (ctl_ok && (S.have_r || acq) && nr > 0) {
            for (i = 0; i < 3; i++) tr[i] = S.tau_req[i] - tau_coil[i] - tau_rcs[i];
            allocate(tr, A, cmd_r, cmd_g);
        }
        for (i = 0; i < nr; i++) { S.h_prev[i] = z->h[i]; S.cmd_r_prev[i] = cmd_r[i]; }
        S.h_prev_ok = 1;
        break; }

    case ADCS_MODE_SPINUP:
    case ADCS_MODE_SUN_SPIN:
        if (phase < p->mtq_meas + dt/2) {
            adcs_zero3(m_body);
            if (first) { adcs_zero3(S.bsum_raw); S.bn = 0; }
            adcs_add3(S.bsum_raw, z->B, S.bsum_raw); S.bn++;
            if (fabs(phase - p->mtq_meas) < dt/2) {
                adcs_real Bav[3], m0[3];
                adcs_scale3(S.bsum_raw, 1.0/S.bn, Bav);
                if (S.mode == ADCS_MODE_SPINUP) {
                    adcs_real bd[3] = {0, 0, 0}, wd[3] = {0, 0, 0};
                    if (S.B1raw_ok) for (i = 0; i < 3; i++) bd[i] = (Bav[i] - S.B1raw[i])/p->mtq_period;
                    wd[2] = S.sigma*p->ss_spin_dps*ADCS_D2R;
                    adcs_gen_bdot(Bav, bd, wd, p->ss_k_l1, m0);
                } else {
                    int ecl = (!z->sun_ok && p->ss_eclipse == 1) || !S.s_prop_ok;
                    adcs_sun_spin(Bav, S.w_est, S.s_prop, ecl, p->J, p->ss_spin_dps, p->ss_k1, p->ss_k2, p->ss_rz_floor, m0);
                }
                adcs_copy3(Bav, S.B1raw); S.B1raw_ok = 1;
                if (m0[0] != 0 || m0[1] != 0 || m0[2] != 0) { adcs_sub3(m0, p->m_res_est, m_body); adcs_sat_dipole(m_body, p->m_max); }
            }
        }
        if (nr > 0) idle_rotors(cmd_r, 0);
        break;
    default:
        break;
    }
    adcs_copy3(m_body, S.m_hold);
    S.clean = (m_body[0] == 0 && m_body[1] == 0 && m_body[2] == 0);
    adcs_drv_write(p, m_body, cmd_r, cmd_g, duty);
    adcs_copy3(m_body, S.out_m);
    for (i = 0; i < NR; i++) S.out_r[i] = cmd_r[i];
    for (i = 0; i < NG; i++) S.out_g[i] = cmd_g[i];
    for (i = 0; i < NC; i++) S.out_duty[i] = duty[i];
    (void)ng;
    return 0;
}

int32_t adcs_fsw_command(const uint8_t *tc, size_t len)
{
    /* TC 0x01: set controller state (tc[1]) */
    if (!S.ready || !tc || len < 2) return -1;
    if (tc[0] == 0x01 && tc[1] < ADCS_MODE_COUNT) { enter(tc[1]); return 0; }
    return -2;
}

int32_t adcs_fsw_peek(adcs_fsw_state_t *o)
{
    int i;
    adcs_real H[3] = {0, 0, 0}, A[3][8];
    if (!S.ready || !o) return -1;
    o->t_ns = (uint64_t)(S.t*1e9 + 0.5);
    o->q_est[0] = (float)S.K.q[3]; o->q_est[1] = (float)S.K.q[0]; o->q_est[2] = (float)S.K.q[1]; o->q_est[3] = (float)S.K.q[2];
    for (i = 0; i < 3; i++) { o->w_est[i] = (float)S.w_est[i]; o->b_est[i] = (float)S.K.b[i]; o->m_cmd[i] = (float)S.out_m[i]; }
    adcs_rotor_axes(&S.p, S.z.delta, A);
    for (i = 0; i < S.p.nr; i++) { H[0] += A[0][i]*S.z.h[i]; H[1] += A[1][i]*S.z.h[i]; H[2] += A[2][i]*S.z.h[i]; }
    for (i = 0; i < 3; i++) o->h_int[i] = (float)H[i];
    for (i = 0; i < 8; i++) o->u_cmd[i] = (float)(i < ADCS_MAX_ROTORS ? S.out_r[i] : 0);
    o->mode = S.mode; o->faults = S.faults;
    return 0;
}

const char *adcs_fsw_build_id(void) { return "trinetra-fsw-c/1.0.0 (adcs-fswcfg/1)"; }

/* ---- extension for the SILS and the parity ledger (not part of adcs_fsw.h) ---- */
int32_t adcs_fsw_debug(double *out, int n)
{
    /* t, mode, ad_ok, q[4], b[3], w_est[3], q_ref[4], tau_req[3], m[3], cmd_r[8], cmd_g[4], duty[6] */
    double v[48]; int i, k = 0;
    v[k++] = S.t; v[k++] = S.mode; v[k++] = S.ad_ok;
    for (i = 0; i < 4; i++) v[k++] = S.K.q[i];
    for (i = 0; i < 3; i++) v[k++] = S.K.b[i];
    for (i = 0; i < 3; i++) v[k++] = S.w_est[i];
    for (i = 0; i < 4; i++) v[k++] = S.q_ref[i];
    for (i = 0; i < 3; i++) v[k++] = S.tau_req[i];
    for (i = 0; i < 3; i++) v[k++] = S.out_m[i];
    for (i = 0; i < 8; i++) v[k++] = S.out_r[i];
    for (i = 0; i < 4; i++) v[k++] = S.out_g[i];
    for (i = 0; i < 6; i++) v[k++] = S.out_duty[i];
    for (i = 0; i < n && i < k; i++) out[i] = v[i];
    return k;
}
