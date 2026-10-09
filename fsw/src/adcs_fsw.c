/*
 * adcs_fsw.c -- the reference flight software: adcs_fsw.h over adcs_hal.h, the shell.
 * Tick order: fsw/pseudocode/08_mode_manager.md; one branch per controller state, as
 * asils.fsw.step (the MATLAB twin) and fsw-rs (Rust). The shell owns the tick, the state and
 * the HAL; guidance is adcs_guid.c, the mode manager adcs_modes.c, FDIR adcs_fdir.c.
 * All state is in one static struct (adcs_fsw_int.h); adcs_fsw_init resets every field.
 * The helpers of the step (adcs_ctl_*, adcs_alloc_*, adcs_orbit_acc) delegate to their translation of
 * 06_step_laws.pc (fsw/alg/src/steplaws.c) on the records CtlState and CtlParams.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#include <string.h>
#include "adcs_fsw.h"
#include "adcs_fsw_int.h"
#include "adcs_alg_glue.h"
#include "adcs_alg_id.h"          /* the algorithms' identity (tools/flight_build.py): the build id ends with it */

/* bang-bang B-dot boundary layer: proportional gain inside it = BDOT_BL_GAIN x the B-dot gain */
#define BDOT_BL_GAIN 4.0

static fsw_t S;

/* A star-tracker quaternion further than ST_NORM_TOL from unit length is not a reading. (Its
 * innovation is not gated: after a slew or a coast the filter's covariance is too small, and a
 * gate rejects the very updates that correct it; gating waits on a consistent filter.) */
#define ST_NORM_TOL 1e-3

/* two-body + J2 acceleration in J2000 (the pole of date is 0.4 deg off: second order on J2); no orbit to
 * speak of, no acceleration (not a division by zero) */
/* written from the design: steplaws::orbit_acc (fsw/alg) */
void adcs_orbit_acc(const adcs_real r[3], adcs_real mu, adcs_real a[3]) { gl_o3(steplaws_orbit_acc(gl_v3(r), mu), a); }

/* nav's onboard orbit after the tick's step 1: the receiver's fix taken in (its gate, its frame, its age), else the orbit
 * held carried one tick dt (two-body + J2 by velocity Verlet), else none */
/* written from the design: navorbit::onboard_orbit (fsw/alg) */
void adcs_onboard_orbit(const adcs_real r0[3], const adcs_real v0[3], int have, int ok, const adcs_real r_fix[3], const adcs_real v_fix[3],
                        int ecef, double lat, double jd, double dt, double mu, adcs_real r[3], adcs_real v[3], int *have_r)
{
    navorbit_onboard_orbit_out o = navorbit_onboard_orbit(gl_v3(r0), gl_v3(v0), have != 0, ok != 0, gl_v3(r_fix), gl_v3(v_fix), ecef != 0,
                                                          lat, jd, dt, mu);
    gl_o3(o.r, r); gl_o3(o.v, v); *have_r = o.have_r ? 1 : 0;
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
    if (adcs_params_validate(&S.p) != 0) { S.ready = 0; return -12; }   /* a value outside its rule (params.toml) */
    if (!adcs_modes_feasible(&S.p, S.p.start_mode) || (S.p.auto_next != ADCS_MODE_NONE && !adcs_modes_feasible(&S.p, S.p.auto_next))) return -13;
    for (i = 0; i < S.p.n_sched; i++) if (!adcs_modes_feasible(&S.p, S.p.sched_mode[i])) return -13;
    p = &S.p;
    S.start_ns = init->start_ns;
    S.mode = p->start_mode; S.t_st = -1e9; S.t_Bref = -1e9; S.last_ctrl = -1e9; S.es_t = -1e9;
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
/* the records of 06_step_laws.pc onto the flight software's state, field by field */
static CtlState ctl_state(const fsw_t *st)
{
    CtlState c;
    c.q = gl_v4(st->K.q); c.w_est = gl_v3(st->w_est); c.q_ref = gl_v4(st->q_ref); c.w_ref = gl_v3(st->w_ref);
    c.i_q = gl_v3(st->I_q); c.tau_req = gl_v3(st->tau_req); c.mode = st->mode; c.s_prop = gl_v3(st->s_prop);
    c.s_prop_ok = st->s_prop_ok != 0; c.cap = gl_v3(st->cap); c.hcap = gl_v3(st->hcap);
    return c;
}

static void put_ctl_state(CtlState c, fsw_t *st)
{
    gl_o4(c.q, st->K.q); gl_o3(c.w_est, st->w_est); gl_o4(c.q_ref, st->q_ref); gl_o3(c.w_ref, st->w_ref);
    gl_o3(c.i_q, st->I_q); gl_o3(c.tau_req, st->tau_req); st->mode = (uint8_t)c.mode; gl_o3(c.s_prop, st->s_prop);
    st->s_prop_ok = c.s_prop_ok; gl_o3(c.cap, st->cap); gl_o3(c.hcap, st->hcap);
}

/* the parameters the laws read, the magnetic gains g_mtq among them */
static CtlParams ctl_params(const fsw_t *st)
{
    const adcs_params_t *p = &st->p;
    const adcs_gains_t *g = &st->g_mtq;
    CtlParams c;
    c.mtq_law = p->mtq_law; c.mtq_eps = p->mtq_eps; c.mtq_k1 = p->mtq_k1; c.mtq_k2 = p->mtq_k2; c.mtq_k16 = p->mtq_k16;
    c.mtq_lam16 = p->mtq_lam16; c.sb_kp = p->sb_kp; c.sb_kd = p->sb_kd; c.sb_kroll = p->sb_kroll; c.sb_kdroll = p->sb_kdroll;
    c.sb_roll_gate = p->sb_roll_gate; c.sun_axis = gl_v3(p->sun_axis); c.roll_axis = gl_v3(p->roll_axis);
    c.mtq_pth = gl_m33(&p->mtq_Pth[0][0]); c.mtq_pw = gl_m33(&p->mtq_Pw[0][0]); c.mtq_period = p->mtq_period;
    c.j = gl_m33(&p->J[0][0]); c.capture_deg = p->capture_deg; c.capture_rate_deg_s = p->capture_rate_deg_s;
    c.sa_w_max_deg_s = p->sa_w_max_deg_s; c.sa_kd = p->sa_kd;
    c.g_law = g->law; c.g_kp = gl_v3(g->Kp); c.g_kd = gl_v3(g->Kd); c.g_ki = gl_v3(g->Ki); c.g_klqr = gl_m33(&g->Klqr[0][0]);
    c.g_lambda = g->lambda; c.g_phi = g->phi; c.g_gs = gl_v3(g->Gs); c.g_err_max = g->err_max; c.g_int_max = g->int_max;
    return c;
}

/* the magnetic pointing law the registry selected (mtq_law) */
/* written from the design: steplaws::ctl_mtq (fsw/alg) */
void adcs_ctl_mtq(fsw_t *st) { put_ctl_state(steplaws_ctl_mtq(ctl_state(st), ctl_params(st)), st); }

/* the large-error capture of the fine states; 1 when it ran */
/* written from the design: steplaws::ctl_capture (fsw/alg) */
int adcs_ctl_capture(fsw_t *st, const adcs_real Hdev[3])
{
    steplaws_ctl_capture_out o = steplaws_ctl_capture(ctl_state(st), ctl_params(st), gl_v3(Hdev));
    put_ctl_state(o.s, st);
    return o.on ? 1 : 0;
}

/* the rotor Sun acquisition */
/* written from the design: steplaws::ctl_sun_acq (fsw/alg) */
void adcs_ctl_sun_acq(fsw_t *st, const adcs_real Hdev[3]) { put_ctl_state(steplaws_ctl_sun_acq(ctl_state(st), ctl_params(st), gl_v3(Hdev)), st); }

/* the rotor commands for a torque (commands of rotors not set keep their value) */
/* written from the design: steplaws::alloc_rotors (fsw/alg) */
void adcs_alloc_rotors(fsw_t *st, const adcs_real tau_rot[3], adcs_real A[3][8], adcs_real cmd_r[NR], adcs_real cmd_g[NG])
{
    const adcs_params_t *p = &st->p;
    pc_a3a8f a = {{{{0}}}};
    steplaws_alloc_rotors_out o;
    int i, k;
    for (k = 0; k < 3; k++) for (i = 0; i < p->nr && i < NR; i++) a.v[k].v[i] = A[k][i];
    o = steplaws_alloc_rotors(gl_v3(tau_rot), a, gl_v8(cmd_r), gl_v4(cmd_g), p->nr, p->ng, gl_u8(p->rot_gi), gl_u8(p->rot_kind),
                              gl_b8(st->rot_failed), gl_v8(st->z.h), gl_m43(&p->gim_axis[0][0]), p->gim_rate_max, p->cmg_lam0,
                              p->cmg_mu, p->cmg_k_null, gl_v8(p->rot_h0));
    gl_o8(o.cmd_r, cmd_r); gl_o4(o.cmd_g, cmd_g);
}

/* the rotor command outside the fine states */
/* written from the design: steplaws::alloc_idle (fsw/alg) */
void adcs_alloc_idle(fsw_t *st, adcs_real cmd_r[NR], int zero_cmg)
{
    gl_o8(steplaws_alloc_idle(gl_v8(cmd_r), st->p.nr, gl_v8(st->z.h), gl_v8(st->h_t_rot), gl_u8(st->p.rot_gi), gl_u8(st->p.rot_kind),
                              zero_cmg != 0), cmd_r);
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
    adcs_fdir_sensors(&S, dt);

    /* 1 onboard orbit (02): nav's (navorbit::onboard_orbit): the GNSS fix (a fix inside the Earth is none; one in ECEF
     * taken to J2000 at its epoch; a late one carried forward by its age), else two-body + J2 by velocity Verlet */
    adcs_onboard_orbit(S.r, S.v, S.have_r, z->gps_ok, z->r, z->v, p->gnss_ecef, p->gps_latency, S.jd, dt, p->mu, S.r, S.v, &S.have_r);

    /* 2 estimation */
    adcs_sun_model(S.jd, S.gd.sun_eci);
    if (p->gd_yaw_flip && S.have_r) adcs_yaw_flip(&S.gd, S.r, S.v, p->gd_flip_hyst);
    phase = fmod(S.t + 1e-9, p->mtq_period);
    first = phase < dt - 1e-9;
    if (first) S.mag_done = 0;
    if (z->es_ok) { adcs_copy3(z->nadir, S.es_n); S.es_t = S.t; }
    if (first && S.have_r && (S.t - S.t_Bref) >= 0.999) {
        if (!S.gh_ok || fabs(S.jd - S.gh_jd) > 1.0) { adcs_igrf_gh(adcs_decyear(S.jd), S.gh); S.gh_ok = 1; S.gh_jd = S.jd; }
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
            } else if (z->sun_ok && z->mag_ok && S.clean && S.bref_ok) {
                adcs_real q0[4];
                if (adcs_triad(z->sun, z->B, S.gd.sun_eci, S.Bref, q0) == 0) {       /* Sun and field not parallel */
                    adcs_mekf_init(&S.K, q0, 0.05, 2e-4, p->gyro_arw, p->gyro_rrw);
                    S.ad_ok = 1;
                }
            }
        } else {
            adcs_mekf_predict(&S.K, z->w, dt);
            if (p->has_st && z->st_ok) {
                int h;
                for (h = 0; h < p->n_heads; h++) {
                    const adcs_real *qs = z->q_st[h];
                    adcs_real qn = sqrt(qs[0]*qs[0] + qs[1]*qs[1] + qs[2]*qs[2] + qs[3]*qs[3]);
                    if (z->st_valid[h] && fabs(qn - 1.0) < ST_NORM_TOL) {         /* a unit quaternion, or no reading */
                        adcs_real wb[3], ql[4], bs[3];
                        adcs_sub3(z->w, S.K.b, wb);
                        adcs_latency(z->q_st[h], wb, p->st_latency, ql);
                        for (i = 0; i < 3; i++) bs[i] = p->st_bs[h][i];
                        (void)adcs_mekf_quat(&S.K, ql, p->st_noise_cross*p->mekf_meas_scale, p->st_noise_roll*p->mekf_meas_scale, bs, 0.0);
                    }
                }
                S.t_st = S.t;
            } else if (p->has_st && S.t - S.t_st < p->st_coast_s) {
                /* short star-tracker outage: coast on the gyro */
            } else {
                /* vector updates once per coil cycle; the field on the first clean tick of the cycle (the
                 * coils are off from the cycle start, and the magnetometer sees the previous tick's dipole) */
                int tried = 0, took = 0;
                if (z->sun_ok && first) { tried++; took += adcs_mekf_vector(&S.K, z->sun, S.gd.sun_eci, p->mekf_sig_sun, p->mekf_gate); }
                if (S.clean && !S.mag_done && S.bref_ok && z->mag_ok) {
                    adcs_real Bn = adcs_norm3(z->B), e = p->mekf_mag_err_T/(Bn > 1e-9 ? Bn : 1e-9);
                    S.mag_done = 1; tried++;
                    took += adcs_mekf_vector(&S.K, z->B, S.Bref, sqrt(p->mekf_sig_mag*p->mekf_sig_mag + e*e), p->mekf_gate);
                }
                if (first && S.have_r && S.t - S.es_t < p->mtq_period) {    /* latest Earth-sensor sample of the cycle */
                    adcs_real nr_[3], sg = p->es_noise > 1e-3 ? p->es_noise : 1e-3;
                    adcs_scale3(S.r, -1.0, nr_);
                    tried++; took += adcs_mekf_vector(&S.K, S.es_n, nr_, 2*sg, p->mekf_gate);
                    S.es_t = -1e9;
                }
                /* every update gated for mekf_rej_max in a row: the estimate has diverged, re-initialise */
                if (tried) { if (took) S.n_rej = 0; else S.n_rej += tried; }
                if (p->mekf_rej_max > 0 && S.n_rej >= p->mekf_rej_max) { S.ad_ok = 0; S.n_rej = 0; }
            }
        }
    }
    {   /* rate for the controllers: bias removed, first-order low-pass */
        adcs_real wr[3], a = dt/(p->rate_lpf_s + dt);
        if (S.ad_ok) adcs_sub3(z->w, S.K.b, wr); else adcs_copy3(z->w, wr);
        if (p->rate_lpf_s <= 0) adcs_copy3(wr, S.w_est);
        else for (i = 0; i < 3; i++) S.w_est[i] += a*(wr[i] - S.w_est[i]);
    }

    /* 3 mode manager: commanded changes, safe mode, the transitions */
    adcs_modes_schedule(&S);
    adcs_fdir_safe(&S);
    adcs_modes_step(&S, dt);

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
            if (z->mag_ok) { adcs_real u[3]; adcs_unit(z->B, u); adcs_add3(S.bsum, u, S.bsum); adcs_add3(S.bsum_raw, z->B, S.bsum_raw); S.bn++; }
            if (fabs(phase - p->mtq_meas) < dt/2 && S.bn == 0) { S.b1_ok = 0; S.B1raw_ok = 0; }   /* no field this cycle: no dipole, no rate across the gap */
            else if (fabs(phase - p->mtq_meas) < dt/2) {
                adcs_real b[3];
                int law = p->bdot_law;
                adcs_scale3(S.bsum, 1.0/S.bn, b); adcs_unit(b, b);
                if (law == 0 && (!p->has_gyro || S.gyro_age > 0)) law = 1;     /* no fresh rate: the field-derivative law */
                if (law == 0) {
                    adcs_real bd[3], bm[3];
                    adcs_cross(z->w, b, bd); adcs_scale3(bd, -1.0, bd);
                    adcs_sub3(b, bd, bm);
                    adcs_bdot(bm, b, 1.0, adcs_norm3(z->B), p->bdot_k, p->m_max, m_body);
                } else if (law == 1) {
                    if (S.b1_ok) adcs_bdot(S.b1, b, p->mtq_period, adcs_norm3(z->B), p->bdot_k, p->m_max, m_body);
                } else if (law == 2) {
                    /* bang-bang with a boundary layer: full dipole against d(b)/dt outside it, a
                     * proportional law BDOT_BL_GAIN x the B-dot gain inside it (pure sign switching
                     * limit-cycles around the detumble exit rate: 4 of 12 seeds never settle) */
                    adcs_real bl = p->m_max*adcs_norm3(z->B)/(BDOT_BL_GAIN*p->bdot_k), r;
                    if (S.b1_ok) for (i = 0; i < 3; i++) {
                        r = ((b[i] - S.b1[i])/p->mtq_period)/bl;
                        m_body[i] = -p->m_max*(r > 1.0 ? 1.0 : (r < -1.0 ? -1.0 : r));
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
        if (nr > 0) adcs_alloc_idle(&S, cmd_r, 1);
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
        if (nr > 0) adcs_alloc_idle(&S, cmd_r, 1);
        break; }

    case ADCS_MODE_NADIR_MTQ:
    case ADCS_MODE_SUN_MTQ:
        if (first) adcs_zero3(m_body);
        else if (fabs(phase - p->mtq_meas) < dt/2 && S.ad_ok && S.have_r) {
            adcs_real wd[3], qc[4], qe[4], A[3][3], wr[3], we[3], wen;
            adcs_guidance(adcs_guid_kind(S.mode), S.r, S.v, S.t, &S.gd, S.q_ref, S.w_ref, wd);
            /* hand-over (05_control.md): from a spinning body the rate error is damped first with the
             * detumble gain (Avanzini & Giulietti 2012), m = (k/|B|) (w_e x b), torque -k w_e perp b;
             * the spin-up gain drags the rate along the turning field instead. The pointing law takes
             * over once the rate error has stayed below ho_out for ho_hold_s */
            adcs_qconj(S.q_ref, qc); adcs_qmult(qc, S.K.q, qe);
            adcs_dcm(qe, A); adcs_mat3_vec(A, S.w_ref, wr);
            adcs_sub3(S.w_est, wr, we); wen = adcs_norm3(we);
            if (!S.ho && wen > p->ho_in_dps*ADCS_D2R) { S.ho = 1; S.ho_t = 0; }
            if (S.ho) {
                if (wen < p->ho_out_dps*ADCS_D2R) S.ho_t += p->mtq_period; else S.ho_t = 0;
                if (S.ho_t >= p->ho_hold_s) S.ho = 0;
            }
            if (S.ho) {
                adcs_real bu[3], Bn = adcs_norm3(z->B);
                adcs_unit(z->B, bu); adcs_cross(we, bu, m_body);
                adcs_scale3(m_body, Bn > 1e-9 ? p->bdot_k/Bn : 0.0, m_body);     /* no field: no dipole */
                adcs_zero3(S.tau_req);
            } else {
                adcs_ctl_mtq(&S);
                if (p->mtq_gg_ff & (S.mode == ADCS_MODE_SUN_MTQ ? 1 : 2)) {
                    /* cancel the modelled gravity-gradient torque, 3 mu/|r|^5 (r_b x J r_b) */
                    adcs_real Ab[3][3], rb[3], Jr[3], c[3], rn, f;
                    adcs_dcm(S.K.q, Ab); adcs_mat3_vec(Ab, S.r, rb);
                    rn = adcs_norm3(rb); f = 3*p->mu/(rn*rn*rn*rn*rn);
                    adcs_mat3_vec(p->J, rb, Jr); adcs_cross(rb, Jr, c);
                    for (i = 0; i < 3; i++) S.tau_req[i] -= f*c[i];
                }
                adcs_torque2dipole(S.tau_req, z->B, p->m_max, m_body);
            }
            adcs_sub3(m_body, p->m_res_est, m_body); adcs_sat_dipole(m_body, p->m_max);
        } else if (phase < p->mtq_meas) adcs_zero3(m_body);
        break;

    case ADCS_MODE_NADIR_FINE: case ADCS_MODE_TARGET_FINE: case ADCS_MODE_SLEW_FINE:
    case ADCS_MODE_SUN_FINE: case ADCS_MODE_SUN_ACQ_ROTOR: {
        int acq = S.mode == ADCS_MODE_SUN_ACQ_ROTOR, ctl_ok = S.ad_ok || acq;
        adcs_real A[3][8], Hdev[3] = {0, 0, 0}, dH[3], tau_rcs[3] = {0, 0, 0}, tau_coil[3] = {0, 0, 0}, tr[3];
        adcs_rotor_axes(p, z->delta, A);
        for (i = 0; i < nr; i++) { Hdev[0] += A[0][i]*z->h[i]; Hdev[1] += A[1][i]*z->h[i]; Hdev[2] += A[2][i]*z->h[i]; }
        adcs_fdir_rotors(&S, dt);
        /* control law at the control rate */
        if (acq && S.t - S.last_ctrl >= p->rw_dt - 1e-9) { adcs_ctl_sun_acq(&S, Hdev); S.last_ctrl = S.t; }
        else if (!acq && S.ad_ok && S.have_r && S.t - S.last_ctrl >= p->rw_dt - 1e-9) {
            adcs_real wd[3];
            adcs_guidance(adcs_guid_kind(S.mode), S.r, S.v, S.t, &S.gd, S.q_ref, S.w_ref, wd);
            S.capturing = adcs_ctl_capture(&S, Hdev);
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
            adcs_alloc_rotors(&S, tr, A, cmd_r, cmd_g);
        }
        for (i = 0; i < nr; i++) { S.h_prev[i] = z->h[i]; S.cmd_r_prev[i] = cmd_r[i]; }
        S.h_prev_ok = 1;
        break; }

    case ADCS_MODE_SPINUP:
    case ADCS_MODE_SUN_SPIN:
        if (phase < p->mtq_meas + dt/2) {
            adcs_zero3(m_body);
            if (first) { adcs_zero3(S.bsum_raw); S.bn = 0; }
            if (z->mag_ok) { adcs_add3(S.bsum_raw, z->B, S.bsum_raw); S.bn++; }
            if (fabs(phase - p->mtq_meas) < dt/2 && S.bn == 0) S.B1raw_ok = 0;
            else if (fabs(phase - p->mtq_meas) < dt/2) {
                adcs_real Bav[3], m0[3];
                adcs_scale3(S.bsum_raw, 1.0/S.bn, Bav);
                if (S.mode == ADCS_MODE_SPINUP) {
                    adcs_real bd[3] = {0, 0, 0}, wd[3] = {0, 0, 0};
                    if (S.B1raw_ok) for (i = 0; i < 3; i++) bd[i] = (Bav[i] - S.B1raw[i])/p->mtq_period;
                    wd[2] = S.sigma*p->ss_spin_dps*ADCS_D2R;
                    adcs_gen_bdot(Bav, bd, wd, p->ss_k_l1, m0);
                } else {
                    int ecl = (!z->sun_ok && p->ss_eclipse == 1) || !S.s_prop_ok;
                    if (p->ss_law == 1)
                        adcs_sun_spin_deruiter(Bav, S.w_est, S.s_prop, ecl, p->J, p->ss_spin_dps, p->ss_dr_k, p->ss_dr_k1, p->ss_dr_k2, m0);
                    else
                        adcs_sun_spin(Bav, S.w_est, S.s_prop, ecl, p->J, p->ss_spin_dps, p->ss_k1, p->ss_k2, p->ss_rz_floor, m0);
                }
                if (p->ss_law == 2) {                 /* P8 Celani 2026: power face onto the Sun, no spin */
                    adcs_real a[3], tau[3], Bs = adcs_dot(Bav, Bav);
                    adcs_zero3(m0);
                    if (S.s_prop_ok && Bs >= 1e-18) {
                        adcs_unit(S.s_prop, a);
                        adcs_mtq_boresight(p->sun_axis, a, S.w_est, p->sb_kp, p->sb_kd, tau);
                        adcs_cross(Bav, tau, m0);
                        for (i = 0; i < 3; i++) m0[i] = m0[i]/Bs;
                    }
                }
                adcs_copy3(Bav, S.B1raw); S.B1raw_ok = 1;
                if (m0[0] != 0 || m0[1] != 0 || m0[2] != 0) { adcs_sub3(m0, p->m_res_est, m_body); adcs_sat_dipole(m_body, p->m_max); }
            }
        }
        if (nr > 0) adcs_alloc_idle(&S, cmd_r, 0);
        break;
    default:                                           /* no such state (refused at init and by command): coils off */
        adcs_zero3(m_body);
        break;
    }
    if (S.mag_seen && S.mag_age > MAG_HOLD_CYCLES*p->mtq_period) adcs_zero3(m_body);   /* no field to act on */
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
    if (tc[0] == 0x01 && tc[1] < ADCS_MODE_COUNT) { if (!adcs_modes_feasible(&S.p, tc[1])) return -3; adcs_modes_enter(&S, tc[1]); return 0; }
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

const char *adcs_fsw_build_id(void) { return "trinetra-fsw-c/1.0.0 (adcs-fswcfg/1) alg " ADCS_ALG_ID; }

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
