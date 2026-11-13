/*
 * test_fsw.c -- host unit tests of the reference flight software.
 * Reference numbers marked "twin" come from the MATLAB twin (asils.*) and are
 * the same vectors fsw-rs checks, so the three implementations agree by test.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "adcs_fsw.h"
#include "adcs_params.h"
#include "adcs_gnc.h"
#include "adcs_env.h"
#include "adcs_devices.h"
#include "adcs_drv.h"
#include "hal_stub.h"
#include "adcs_alg.h"

static int n_pass, n_fail;
#define CHECK(cond, ...) do { if (cond) n_pass++; else { n_fail++; printf("FAIL %s:%d ", __FILE__, __LINE__); printf(__VA_ARGS__); printf("\n"); } } while (0)

static void t_math(void)
{
    adcs_real a[4] = {0.1, -0.3, 0.2, 0.9}, b[4] = {-0.4, 0.2, 0.5, 0.7}, ab[4], A[3][3], B[3][3], AB[3][3], BA[3][3], q[4];
    int i, j; adcs_real e = 0, e2;
    adcs_qnorm(a); adcs_qnorm(b); adcs_qmult(a, b, ab);
    adcs_dcm(ab, AB); adcs_dcm(a, A); adcs_dcm(b, B); adcs_mat3_mul(B, A, BA);
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) e += fabs(AB[i][j] - BA[i][j]);
    CHECK(e < 1e-14, "dcm(a*b) = dcm(b) dcm(a): %g", e);
    adcs_fromdcm(A, q);
    e2 = fabs(fabs(q[0]*a[0] + q[1]*a[1] + q[2]*a[2] + q[3]*a[3]) - 1);
    CHECK(e2 < 1e-14, "fromdcm round trip %g", e2);
    {   /* pinv_rows: A A+ = I for 4 pyramid axes */
        adcs_real P[3][8] = {{0.8165, 0, -0.8165, 0}, {0, 0.8165, 0, -0.8165}, {0.5774, 0.5774, 0.5774, 0.5774}}, Pi[8][3], I3 = 0;
        adcs_pinv_rows(P, 4, Pi);
        for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) {
            int k; adcs_real s = 0;
            for (k = 0; k < 4; k++) s += P[i][k]*Pi[k][j];
            I3 += fabs(s - (i == j));
        }
        CHECK(I3 < 1e-9, "A pinv(A) = I: %g", I3);
    }
    {   /* Jacobi on a symmetric 4x4 with known eigenvalues */
        adcs_real K[4][4] = {{4, 1, 0, 0}, {1, 3, 0, 0}, {0, 0, 2, 0.5}, {0, 0, 0.5, 1}}, lam[4], V[4][4], mx = -1e9;
        adcs_jacobi_eig4(K, lam, V);
        for (i = 0; i < 4; i++) if (lam[i] > mx) mx = lam[i];
        CHECK(fabs(mx - (3.5 + sqrt(1.25))) < 1e-12, "jacobi lambda_max %.15g", mx);
    }
}

static void t_env(void)
{
    /* twin: asils.env.igrf_ned(asils.env.igrf_gh(2027.1), 0.3, 1.2, 550, 13) and asils.fsw.sun_model */
    adcs_real gh[195], B[3], s[3];
    adcs_igrf_gh(2027.1, gh);
    adcs_igrf_ned(gh, 0.3, 1.2, 550, 13, B);
    CHECK(fabs(B[0] - 29155.707342) < 1e-5 && fabs(B[1] + 187.025096) < 1e-5 && fabs(B[2] - 13232.587581) < 1e-5,
          "IGRF vs twin: %.6f %.6f %.6f", B[0], B[1], B[2]);
    adcs_sun_model(2461407.25, s);
    CHECK(fabs(s[0] - 0.185796268321) < 1e-11 && fabs(s[1] + 0.901530663479) < 1e-11 && fabs(s[2] + 0.390796890321) < 1e-11,
          "Sun model vs twin: %.12f %.12f %.12f", s[0], s[1], s[2]);
}

static void t_estimation(void)
{
    adcs_real q[4] = {0.2, -0.1, 0.4, 0.8}, A[3][3], b[8][3], r[8][3], qm[4], e;
    int i, k;
    adcs_qnorm(q); adcs_dcm(q, A);
    for (i = 0; i < 8; i++) {
        adcs_real v[3] = {sin(1.3*i + 0.2), cos(0.7*i + 1.1), sin(2.1*i - 0.4)};
        adcs_unit(v, r[i]); adcs_mat3_vec(A, r[i], b[i]);
    }
    adcs_quest(b, r, 0, 8, qm);
    e = adcs_qangle(q, qm);
    CHECK(e < 1e-9, "QUEST on exact stars %g rad", e);
    adcs_triad(b[0], b[1], r[0], r[1], 0.2, qm);
    CHECK(adcs_qangle(q, qm) < 1e-9, "TRIAD %g rad", adcs_qangle(q, qm));
    {   adcs_real bb[3] = {2*b[0][0], 2*b[0][1], 2*b[0][2]}, q1[4] = {9, 9, 9, 9};
        CHECK(adcs_triad(b[0], bb, r[0], r[1], 0.2, q1) == -1 && q1[0] == 9, "TRIAD on parallel vectors fixes no attitude"); }
    {   /* MEKF converges from a 10 deg error with two exact vectors, gyro at rest */
        adcs_mekf_t K; adcs_real q0[4], th[3] = {0.1, -0.12, 0.08}, dq[4], w[3] = {0, 0, 0};
        adcs_fromrotvec(th, dq); adcs_qmult(q, dq, q0);
        adcs_mekf_init(&K, q0, 0.2, 1e-3, 1e-6, 1e-8);
        for (k = 0; k < 200; k++) {
            adcs_mekf_predict(&K, w, 0.1, 0);
            adcs_mekf_vector(&K, b[0], r[0], 1e-3, 0); adcs_mekf_vector(&K, b[1], r[1], 1e-3, 0);
        }
        e = adcs_qangle(q, K.q)*180/ADCS_PI;
        CHECK(e < 0.01, "MEKF two-vector %g deg", e);
    }
}

static void t_laws(void)
{
    adcs_real B[3] = {2e-5, -1e-5, 3e-5}, w[3] = {0.05, -0.02, 0.1}, bd[3], m[3], tq[3], wd[3] = {0, 0, 0.1}, d[3];
    adcs_real J[3][3] = {{0.0067, 0, 0}, {0, 0.042, 0}, {0, 0, 0.042}}, s[3] = {0.3, -0.5, -0.8}, x[3], A[3];
    int k, worst_ok = 1;
    adcs_cross(w, B, bd); adcs_scale3(bd, -1, bd);                      /* dB/dt = -w x B */
    adcs_gen_bdot(B, bd, wd, 1e6, m); adcs_cross(m, B, tq); adcs_sub3(w, wd, d);
    CHECK(adcs_dot(tq, d) < 0, "L1 drives w toward w_d");
    adcs_torque2dipole(w, B, 10, m); adcs_cross(m, B, tq);
    CHECK(fabs(adcs_dot(m, B)) < 1e-18, "torque2dipole m.B = 0");
    adcs_unit(s, s);
    for (k = 0; k < 50; k++) {        /* L2: A.m0 <= 0 (the torque never raises V) */
        adcs_real Bk[3] = {3e-5*sin(k), 3e-5*cos(1.7*k), 2e-5*sin(0.3*k + 1)}, wk[3] = {0.1*sin(2*k), 0.1*cos(k), 0.1*sin(k + 2)};
        adcs_real h[3], ht[3], Rz[3] = {0.035, 0, 0}, hd; int i;
        adcs_sun_spin(Bk, wk, s, 0, J, 6, 0.01, 0.05, 0, m);
        adcs_mat3_vec(J, wk, h);
        for (i = 0; i < 3; i++) { hd = (wk[2] >= 0 ? 1 : -1)*0.042*(-6*ADCS_D2R)*s[i]; ht[i] = h[i] - hd; x[i] = 0.01*ht[i] + 0.05*Rz[i]*wk[i]; }
        adcs_cross(Bk, x, A);
        if (adcs_dot(A, m) > 1e-20) worst_ok = 0;
    }
    CHECK(worst_ok, "Sun spin L2 A.m0 <= 0");
}

/* G2 (S11.6): the rotor commands keep the torque's direction, and a rotor at its momentum capacity is dropped. Three orthogonal rotors
 * and, with four, one along (1, 1, 1)/sqrt(3); the design's alloc_rotors called as the translator wrote it */
static void g2_alloc(const adcs_real tau[3], int four, const adcs_real zh_in[8], adcs_real cmd[8])
{
    pc_a3a8f a = {{{{0}}}};
    pc_a8f cmd_r0 = {{0}}, zh = {{0}}, tmax, hmax, h0 = {{0}};
    pc_a4f cmd_g0 = {{0}};
    pc_a8i gi = {{0}}, kind = {{0}};
    pc_a8b failed = {{0}};
    pc_a4a3f gax = {{{{0}}}};
    steplaws_alloc_rotors_out o;
    int i, k;
    for (i = 0; i < 3; i++) a.v[i].v[i] = 1.0;
    if (four) for (k = 0; k < 3; k++) a.v[k].v[3] = 1.0/sqrt(3.0);
    for (i = 0; i < 8; i++) { tmax.v[i] = 0.002; hmax.v[i] = 0.02; zh.v[i] = zh_in[i]; }
    o = steplaws_alloc_rotors((pc_a3f){{tau[0], tau[1], tau[2]}}, a, cmd_r0, cmd_g0, four ? 4 : 3, 0, gi, kind, failed, zh, gax, 1.0, 1e-3, 10.0, 0.0, h0, tmax, hmax);
    for (i = 0; i < 8; i++) cmd[i] = o.cmd_r.v[i];
}

static void t_alloc(void)
{
    adcs_real zh[8] = {0, 0, 0, 0, 0, 0, 0, 0}, cmd[8], tau[3] = {0.004, 0.0004, -0.0008};
    int i, ok = 1;
    g2_alloc(tau, 0, zh, cmd);
    for (i = 0; i < 3; i++) if (fabs(cmd[i]) > 0.002 + 1e-13) ok = 0;
    CHECK(ok, "a command passes the rotor's torque limit of 0.002: %g %g %g", cmd[0], cmd[1], cmd[2]);
    CHECK(fabs(cmd[1]/cmd[0] - tau[1]/tau[0]) < 1e-12, "the direction bent: y/x is %g for a torque with %g", cmd[1]/cmd[0], tau[1]/tau[0]);
    CHECK(fabs(cmd[2]/cmd[0] - tau[2]/tau[0]) < 1e-12, "the direction bent: z/x is %g for a torque with %g", cmd[2]/cmd[0], tau[2]/tau[0]);
    {   adcs_real small[3] = {0.001, -0.0005, 0.0002};
        g2_alloc(small, 0, zh, cmd);
        CHECK(fabs(cmd[0] + 0.001) < 1e-13 && fabs(cmd[1] - 0.0005) < 1e-13 && fabs(cmd[2] + 0.0002) < 1e-13, "a torque inside the limits is not touched: %.17g %.17g %.17g", cmd[0], cmd[1], cmd[2]); }
    {   adcs_real t2[3] = {-0.001, 0.0005, 0.0};
        zh[0] = 0.02;                                   /* the x rotor full: the command that adds to it is dropped */
        g2_alloc(t2, 0, zh, cmd);
        CHECK(cmd[0] == 0.0, "a rotor at its capacity was commanded to take more: %g", cmd[0]);
        CHECK(fabs(cmd[1] + 0.0005) < 1e-13, "the rotors still working were not solved for: %g", cmd[1]);
        zh[0] = -0.02;                                  /* full the other way: the command unloads it */
        g2_alloc(t2, 0, zh, cmd);
        CHECK(fabs(cmd[0] - 0.001) < 1e-13, "a rotor at its capacity was refused a command that unloads it: %.17g", cmd[0]); }
    {   adcs_real t3[3] = {-0.001, 0.0, 0.0}, s = 1.0/sqrt(3.0);
        int k;
        zh[0] = 0.02;
        g2_alloc(t3, 1, zh, cmd);
        CHECK(cmd[0] == 0.0, "the full rotor was commanded: %g", cmd[0]);
        for (k = 0; k < 3; k++) {
            adcs_real got = cmd[k] + s*cmd[3], want = k == 0 ? 0.001 : 0.0;
            CHECK(fabs(got - want) < 1e-12, "the torque on axis %d is %g with the full rotor dropped", k, got);
        } }
}

static void t_rcs(void)
{
    adcs_params_t p; adcs_real duty[6], tau[3], req[3] = {1e-4, 0, -2e-6};
    memset(&p, 0, sizeof p);
    p.nc = 6; p.rcs_mib = 0.005; p.rcs_res = 0.001;
    p.rcs_tau[0][0] = 4.5e-4; p.rcs_tau[1][0] = -4.5e-4; p.rcs_tau[2][1] = 1.5e-3; p.rcs_tau[3][1] = -1.5e-3; p.rcs_tau[4][2] = 1.5e-3; p.rcs_tau[5][2] = -1.5e-3;
    adcs_rcs_duty(req, &p, 1.0, duty, tau);
    CHECK(fabs(duty[0] - 0.222) < 1e-12 && duty[5] == 0, "RCS duty with MIB: %.4f %.4f", duty[0], duty[5]);
}

/* ---- the ABI end to end: detumble through the byte HAL ---- */
static void put16(uint8_t *b, double v) { int16_t x = (int16_t)(v < 0 ? v - 0.5 : v + 0.5); b[0] = (uint8_t)(x & 0xFF); b[1] = (uint8_t)((x >> 8) & 0xFF); }
static void put32(uint8_t *b, double v) { int32_t x = (int32_t)(v < 0 ? v - 0.5 : v + 0.5); int k; for (k = 0; k < 4; k++) b[k] = (uint8_t)((uint32_t)x >> (8*k)); }

static int run_detumble(int16_t pwm_log[30][3])
{
    static adcs_params_t p; static uint8_t blob[ADCS_PARAMS_BLOB_SIZE];
    adcs_fsw_init_t in; int k, i;
    memset(&p, 0, sizeof p);
    p.jd0 = 2461407.25; p.dt = 0.1; p.mu = 3.986004418e14; p.start_mode = ADCS_MODE_DETUMBLE; p.auto_next = ADCS_MODE_NONE;
    p.bdot_law = 0; p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.has_gyro = 1;
    p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042; p.igrf_nmax = 10; p.rate_lpf_s = 0.3;
    /* the values params.toml requires to be positive (the flight software refuses a blob without them) */
    p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005; p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0;
    p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0; p.clock_step_max_s = 60.0; p.gyro_stale_ticks = 3; p.st_norm_tol = 1e-3; p.triad_sin_min = 0.2;
    adcs_params_encode(&p, blob, sizeof blob);
    hal_stub_reset();
    in.abi_version = ADCS_FSW_ABI_VERSION; in.config_blob = blob; in.config_len = sizeof blob; in.start_ns = 0;
    if (adcs_fsw_init(&in) != 0) return -1;
    for (k = 0; k < 30; k++) {
        double B[3] = {2e-5, -1e-5, 3e-5}, w[3] = {0.05, -0.02, 0.1};
        HS.now_ns = (uint64_t)k*100000000ull;
        HS.mag[0] = 1; for (i = 0; i < 3; i++) put16(HS.mag + 1 + 2*i, B[i]/ADCS_MAG_LSB_T);
        HS.gyro[0] = 1; for (i = 0; i < 3; i++) put32(HS.gyro + 1 + 4*i, w[i]/ADCS_GYRO_LSB);
        adcs_fsw_step(HS.now_ns);
        for (i = 0; i < 3; i++) pwm_log[k][i] = HS.pwm[i];
    }
    return 0;
}

static void t_abi(void)
{
    int16_t a[30][3], b[30][3];
    int k, on = 0;
    CHECK(run_detumble(a) == 0, "init with a generated blob");
    CHECK(run_detumble(b) == 0 && memcmp(a, b, sizeof a) == 0, "two runs in one process give identical bytes");
    for (k = 0; k < 30; k++) {
        double t = k*0.1, ph = fmod(t + 1e-9, 1.0);
        if (ph < 0.2 - 1e-9) CHECK(a[k][0] == 0 && a[k][1] == 0 && a[k][2] == 0, "coils off in the measurement window (t=%.1f)", t);
        if (ph > 0.25 && (a[k][0] || a[k][1] || a[k][2])) on = 1;
    }
    {   /* B-dot on a pure rotation: m ~ -(w x b) x ... ; the dipole opposes db/dt = -w x b */
        double B[3] = {2e-5, -1e-5, 3e-5}, w[3] = {0.05, -0.02, 0.1}, bd[3], m[3] = {a[5][0], a[5][1], a[5][2]};
        bd[0] = -(w[1]*B[2] - w[2]*B[1]); bd[1] = -(w[2]*B[0] - w[0]*B[2]); bd[2] = -(w[0]*B[1] - w[1]*B[0]);
        CHECK(on && m[0]*bd[0] + m[1]*bd[1] + m[2]*bd[2] < 0, "B-dot dipole opposes dB/dt");
        CHECK(a[2][0] == 25936 && a[2][1] == 32767 && a[2][2] == -6826, "PWM words equal the Rust build (fsw-rs/tests/fsw.rs)");
    }
    {   /* a corrupted blob is refused */
        uint8_t blob[ADCS_PARAMS_BLOB_SIZE]; adcs_params_t p; adcs_fsw_init_t in;
        memset(&p, 0, sizeof p); adcs_params_encode(&p, blob, sizeof blob); blob[100] ^= 1;
        in.abi_version = ADCS_FSW_ABI_VERSION; in.config_blob = blob; in.config_len = sizeof blob; in.start_ns = 0;
        CHECK(adcs_fsw_init(&in) != 0, "bad CRC refused");
    }
    {   /* a blob with a good CRC but a value outside its rule (params.toml) is refused at init */
        static adcs_params_t p; static uint8_t blob[ADCS_PARAMS_BLOB_SIZE]; adcs_fsw_init_t in; int k;
        for (k = 0; k < 7; k++) {
            memset(&p, 0, sizeof p);
            p.jd0 = 2461407.25; p.dt = 0.1; p.mu = 3.986004418e14; p.start_mode = ADCS_MODE_DETUMBLE; p.auto_next = ADCS_MODE_NONE;
            p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042;
            p.igrf_nmax = 10; p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005;
            p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0; p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0; p.clock_step_max_s = 60.0; p.gyro_stale_ticks = 3; p.st_norm_tol = 1e-3; p.triad_sin_min = 0.2;
            if (k == 1) p.nr = 9;                      /* more rotors than the arrays hold */
            if (k == 2) p.m_max = 0.0/0.0;             /* NaN */
            if (k == 3) p.start_mode = 11;             /* no such controller state */
            if (k == 4) p.jd0 = 2466000.0;             /* 2039: outside env_epoch_span (E14), no EOP, leap-second or field data */
            if (k == 5) p.jd0 = 2460000.0;             /* 2023: before it */
            if (k == 6) p.jd0 = 2461750.25;            /* 2027-12-11: past the truth's data expiry, inside the flight's own span */
            adcs_params_encode(&p, blob, sizeof blob);
            in.abi_version = ADCS_FSW_ABI_VERSION; in.config_blob = blob; in.config_len = sizeof blob; in.start_ns = 0;
            if (k == 0 || k == 6) CHECK(adcs_fsw_init(&in) == 0 && adcs_params_validate(&p) == 0, "a blob within every rule is accepted (jd0 on 2027-12-11 too)");
            else CHECK(adcs_fsw_init(&in) == -12, "a value outside its rule is refused at init");
        }
        p.start_mode = ADCS_MODE_DETUMBLE; p.nr = 9; p.jd0 = 2461407.25;
        CHECK(strcmp(adcs_params_field(adcs_params_validate(&p)), "nr") == 0, "the refused field is named");
        p.nr = 0; p.jd0 = 2466000.0;
        CHECK(strcmp(adcs_params_field(adcs_params_validate(&p)), "jd0") == 0, "a date outside env_epoch_span names jd0");
    }
}

static void t_st_frame(void)
{
    /* a star-tracker frame through the UART parser */
    uint8_t f[64], pl[40]; int k; uint16_t c;
    adcs_params_t p; adcs_meas_t z; double q[4] = {0.1, 0.2, -0.3, 0.927};
    memset(&p, 0, sizeof p); memset(&z, 0, sizeof z);
    p.has_st = 1; p.n_heads = 1;
    pl[0] = 1; pl[1] = 1;
    for (k = 0; k < 4; k++) put32(pl + 2 + 4*k, q[k]*ADCS_Q30);
    f[0] = ADCS_ST_SYNC1; f[1] = ADCS_ST_SYNC2; f[2] = 18;
    memcpy(f + 3, pl, 18); c = adcs_crc16(pl, 18); f[21] = (uint8_t)(c & 0xFF); f[22] = (uint8_t)(c >> 8);
    hal_stub_reset(); adcs_drv_reset();
    hal_stub_uart_push(ADCS_ST_UART, (const uint8_t *)"\x00\x13", 2);      /* noise before the frame */
    hal_stub_uart_push(ADCS_ST_UART, f, 23);
    adcs_drv_read(&p, &z);
    CHECK(z.st_ok && z.st_valid[0] && fabs(z.q_st[0][2] + 0.3) < 1e-8, "ST frame parsed: ok %d q3 %.9f", z.st_ok, z.q_st[0][2]);
}

static void t_safe(void)
{
    /* the magnetometer stops answering: in detumble the coils act on the field, then stop; in a
     * pointing state the state holds for 60 s, then it is magnetorquer detumble with the fault
     * raised; the fault clears when the magnetometer answers again */
    static adcs_params_t p; static uint8_t blob[ADCS_PARAMS_BLOB_SIZE];
    adcs_fsw_init_t in; adcs_fsw_state_t st; int k, i, run, on_before = 0, on_after = 0, mode59 = -1;
    for (run = 0; run < 2; run++) {
        memset(&p, 0, sizeof p);
        p.jd0 = 2461407.25; p.dt = 0.1; p.mu = 3.986004418e14; p.auto_next = ADCS_MODE_NONE;
        p.start_mode = run == 0 ? ADCS_MODE_DETUMBLE : ADCS_MODE_NADIR_MTQ;
        p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.has_gyro = 1;
        p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042; p.igrf_nmax = 10; p.rate_lpf_s = 0.3;
        p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005; p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0;
        p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0; p.clock_step_max_s = 60.0; p.gyro_stale_ticks = 3; p.st_norm_tol = 1e-3; p.triad_sin_min = 0.2;
        adcs_params_encode(&p, blob, sizeof blob);
        hal_stub_reset();
        in.abi_version = ADCS_FSW_ABI_VERSION; in.config_blob = blob; in.config_len = sizeof blob; in.start_ns = 0;
        CHECK(adcs_fsw_init(&in) == 0, "init (safe-mode test)");
        for (k = 0; k < 720; k++) {
            double B[3] = {2e-5, -1e-5, 3e-5}, w[3] = {0.05, -0.02, 0.1};
            HS.now_ns = (uint64_t)k*100000000ull;
            HS.mag[0] = (uint8_t)(k < 30 || k == 719); for (i = 0; i < 3; i++) put16(HS.mag + 1 + 2*i, B[i]/ADCS_MAG_LSB_T);
            HS.gyro[0] = 1; for (i = 0; i < 3; i++) put32(HS.gyro + 1 + 4*i, w[i]/ADCS_GYRO_LSB + (double)((k*7 + i*3) % 5));   /* counts that change, as a gyro's do (a repeated sample is a quiet gyro, F2) */
            adcs_fsw_step(HS.now_ns);
            if (run == 0 && k < 30 && (HS.pwm[0] || HS.pwm[1] || HS.pwm[2])) on_before = 1;
            if (run == 0 && k >= 30 && k < 719 && (HS.pwm[0] || HS.pwm[1] || HS.pwm[2])) on_after = 1;
            if (run == 1 && k == 620) { adcs_fsw_peek(&st); mode59 = st.mode; }
            if (k == 718) { adcs_fsw_peek(&st); CHECK(st.mode == ADCS_MODE_DETUMBLE && (st.faults & 0x100), "magnetometer silent 60 s: detumble, fault raised (mode %d faults %x)", st.mode, st.faults); }
        }
        adcs_fsw_peek(&st);
        CHECK(!(st.faults & 0x100), "the fault clears when the magnetometer answers");
    }
    CHECK(on_before && !on_after, "coils act on the field, then stop in the dropout (%d %d)", on_before, on_after);
    {   /* no rotors, no thrusters: fine pointing is refused by telecommand and at init */
        uint8_t tc[2] = {0x01, ADCS_MODE_NADIR_FINE};
        CHECK(adcs_fsw_command(tc, 2) == -3, "a state the hardware cannot fly is refused by telecommand");
        tc[1] = ADCS_MODE_NADIR_MTQ;
        CHECK(adcs_fsw_command(tc, 2) == 0, "a state it can fly is accepted");
        p.start_mode = ADCS_MODE_NADIR_FINE; adcs_params_encode(&p, blob, sizeof blob);
        CHECK(adcs_fsw_init(&in) == -13, "and at init");
    }
    CHECK(mode59 == ADCS_MODE_NADIR_MTQ, "a pointing state holds through the first 59 s of the dropout (%d)", mode59);
}

static void t_eop_tc(void)
{
    /* TC 0x02 sets the Earth orientation (UT1 - UTC, xp, yp) of the onboard frame: three little-endian doubles after the id, each inside
     * the rule of its parameter in params.toml, or the whole command is refused and nothing changes (T5/E3, DR-env-002) */
    static adcs_params_t p; static uint8_t blob[ADCS_PARAMS_BLOB_SIZE];
    adcs_fsw_init_t in; uint8_t tc[25]; double e[3] = {-0.0123, 0.12, 0.34};
    memset(&p, 0, sizeof p);
    p.jd0 = 2461407.25; p.dt = 0.1; p.mu = 3.986004418e14; p.auto_next = ADCS_MODE_NONE; p.start_mode = ADCS_MODE_NADIR_MTQ;
    p.mtq_period = 1.0; p.mtq_meas = 0.2; p.m_max = 0.2; p.bdot_k = 1e-3; p.has_gyro = 1;
    p.J[0][0] = 0.0067; p.J[1][1] = 0.042; p.J[2][2] = 0.042; p.igrf_nmax = 10; p.rate_lpf_s = 0.3;
    p.ss_eclipse = 1; p.gd_T = 1.0; p.mtq_phi = 0.01; p.rw_phi = 0.01; p.rw_dt = 0.1; p.fdir_s = 3.0; p.fdir_win_s = 120.0; p.fdir_h_frac = 0.005; p.rcsd_T_damp_s = 20.0; p.rcsd_period_s = 1.0;
    p.st_coast_s = 900.0; p.mekf_sig_mag = 0.01; p.mekf_sig_sun = 0.005; p.mekf_meas_scale = 1.0; p.clock_step_max_s = 60.0; p.gyro_stale_ticks = 3; p.st_norm_tol = 1e-3; p.triad_sin_min = 0.2;
    adcs_params_encode(&p, blob, sizeof blob);
    hal_stub_reset();
    in.abi_version = ADCS_FSW_ABI_VERSION; in.config_blob = blob; in.config_len = sizeof blob; in.start_ns = 0;
    CHECK(adcs_fsw_init(&in) == 0, "init (Earth orientation test)");
    tc[0] = 0x02; memcpy(tc + 1, e, sizeof e);
    CHECK(adcs_fsw_command(tc, 25) == 0, "an Earth orientation inside the rules is accepted");
    CHECK(adcs_fsw_command(tc, 24) == -1, "a short Earth orientation command is refused");
    e[0] = 0.95; memcpy(tc + 1, e, sizeof e);
    CHECK(adcs_fsw_command(tc, 25) == -4, "|UT1 - UTC| above 0.9 s is refused");
    e[0] = 0.5; e[1] = 1.5; memcpy(tc + 1, e, sizeof e);
    CHECK(adcs_fsw_command(tc, 25) == -4, "a pole above 1 arcsecond is refused");
    e[1] = 0.0/0.0; memcpy(tc + 1, e, sizeof e);
    CHECK(adcs_fsw_command(tc, 25) == -4, "a NaN is refused");
}

static void t_nonfinite(void)
{
    /* a NaN or an infinite command drives nothing: zero PWM, zero rotor word, closed valve */
    static adcs_params_t p; double m[3] = {0.0/0.0, 1.0/0.0, 0.1}, r[ADCS_MAX_ROTORS] = {0}, g[ADCS_MAX_GIMBALS] = {0}, d[ADCS_MAX_COUPLES] = {0};
    memset(&p, 0, sizeof p);
    p.m_max = 0.2; p.nr = 1; p.rot_tmax[0] = 1e-3; p.nc = 2; p.dt = 0.1;
    r[0] = 0.0/0.0; d[0] = 0.0/0.0; d[1] = -1.0/0.0;
    hal_stub_reset();
    adcs_drv_write(&p, m, r, g, d);
    CHECK(HS.pwm[0] == 0 && HS.pwm[1] == 0 && HS.pwm[2] == 16384, "non-finite dipole -> 0, finite one kept (%d %d %d)", HS.pwm[0], HS.pwm[1], HS.pwm[2]);
    CHECK(HS.n_can_tx == 2 && HS.can_tx[0].data[0] == 0 && HS.can_tx[0].data[1] == 0, "non-finite rotor command -> 0");
    CHECK(HS.can_tx[1].data[0] == 0 && HS.can_tx[1].data[1] == 0, "non-finite valve duty -> closed");
}


/* ---- the tick tests (docs/S11_INVENTORY.md TS4, S11.4; fsw-rs/tests/tick.rs holds the same) ----
 * What the tick does with a warm reset, a step or a jitter in the time it is handed, and a gyro that has gone quiet. Each
 * states the BEHAVIOUR THE FIXES OWE (docs/TECHNICAL_ROADMAP.md F1 monotonic time kept through a reset and a step; F2 a silent gyro
 * is flagged and its rate dropped; F3 the rate groups on an integer tick counter). F1 and F3 are fixed (S11.4b) and run in main;
 * F2's fail today and are in the skipped list below. The step that fixes one moves it from the list to the run;
 * ADCS_RUN_SKIPPED=1 runs the list and says what each shows today. The sensors are exact and the attitude held: the test is of the tick, not of the estimate. */
int32_t adcs_fsw_debug(double *out, int n);
#define TK_JD0 2461407.25
#define TK_DAY 86400.0
#define TK_STALE_LIMIT_S 1.0        /* F2: DR-fdir-002 sets 3 ticks (0.3 s); the test looks once a second, so one second is its limit; far shorter than the 60 s fault */

static void tk_params(adcs_params_t *p, int start, int sun_gps)
{
    memset(p, 0, sizeof *p);
    p->jd0 = TK_JD0; p->dt = 0.1; p->mu = 3.986004418e14; p->start_mode = (uint8_t)start; p->auto_next = ADCS_MODE_NONE;
    p->bdot_law = 0; p->mtq_period = 1.0; p->mtq_meas = 0.2; p->m_max = 0.2; p->bdot_k = 1e-3; p->has_gyro = 1;
    p->J[0][0] = 0.0067; p->J[1][1] = 0.042; p->J[2][2] = 0.042; p->igrf_nmax = 10; p->rate_lpf_s = 0.3;
    p->ss_eclipse = 1; p->gd_T = 1.0; p->mtq_phi = 0.01; p->rw_phi = 0.01; p->rw_dt = 0.1; p->fdir_s = 3.0; p->fdir_win_s = 120.0; p->fdir_h_frac = 0.005;
    p->rcsd_T_damp_s = 20.0; p->rcsd_period_s = 1.0; p->st_coast_s = 900.0; p->mekf_sig_mag = 0.01; p->mekf_sig_sun = 0.005; p->mekf_meas_scale = 1.0; p->clock_step_max_s = 60.0; p->gyro_stale_ticks = 3; p->st_norm_tol = 1e-3; p->triad_sin_min = 0.2;
    if (sun_gps) {
        int i;
        p->has_sun = 1; p->has_gps = 1; p->gnss_ecef = 0; p->gyro_arw = 1e-6; p->gyro_rrw = 1e-8;
        for (i = 0; i < 3; i++) { p->mtq_Kp[i] = 1e-3; p->mtq_Kd[i] = 1e-2; }
    }
}

static int tk_init(int start, int sun_gps, uint64_t start_ns)
{
    static adcs_params_t p; static uint8_t blob[ADCS_PARAMS_BLOB_SIZE];
    adcs_fsw_init_t in;
    tk_params(&p, start, sun_gps);
    adcs_params_encode(&p, blob, sizeof blob);
    in.abi_version = ADCS_FSW_ABI_VERSION; in.config_blob = blob; in.config_len = sizeof blob; in.start_ns = start_ns;
    return adcs_fsw_init(&in);
}

static double tk_jit(uint64_t *seed)       /* a splitmix64 uniform in (-1, 1), the test's own */
{
    uint64_t z = (*seed += 0x9E3779B97F4A7C15ull);
    z = (z ^ (z >> 30))*0xBF58476D1CE4E5B9ull; z = (z ^ (z >> 27))*0x94D049BB133111EBull; z ^= z >> 31;
    return 2.0*((double)(z >> 11) + 0.5)/9007199254740992.0 - 1.0;
}

/* F3: 30 s of detumble with timestamps off by +/- amp (uniform, seeded) against the exact ones: the same command words */
static void tk_detumble_pwm(double amp, int16_t out[300][3])
{
    uint64_t seed = 7; int k, i;
    hal_stub_reset();
    tk_init(ADCS_MODE_DETUMBLE, 0, 0);
    for (k = 0; k < 300; k++) {
        double B[3] = {2e-5, -1e-5, 3e-5}, w[3] = {0.05, -0.02, 0.1}, off = k == 0 ? 0.0 : amp*tk_jit(&seed);
        HS.now_ns = (uint64_t)((k*0.1 + off)*1e9 + 0.5);
        HS.mag[0] = 1; for (i = 0; i < 3; i++) put16(HS.mag + 1 + 2*i, B[i]/ADCS_MAG_LSB_T);
        HS.gyro[0] = 1; for (i = 0; i < 3; i++) put32(HS.gyro + 1 + 4*i, w[i]/ADCS_GYRO_LSB);
        adcs_fsw_step(HS.now_ns);
        for (i = 0; i < 3; i++) out[k][i] = HS.pwm[i];
    }
}

static void tk_jitter_changes_no_command(double amp)
{
    static int16_t a[300][3], b[300][3];
    int k, differ = 0;
    tk_detumble_pwm(0.0, a); tk_detumble_pwm(amp, b);
    for (k = 0; k < 300; k++) if (memcmp(a[k], b[k], sizeof a[k])) differ++;
    CHECK(differ == 0, "timestamps off by +/- %g s moved the commands of %d of 300 ticks", amp, differ);
}
static void tk_jitter_1us(void) { tk_jitter_changes_no_command(1e-6); }
static void tk_jitter_5ms(void) { tk_jitter_changes_no_command(5e-3); }

/* F1: the scene a sun sensor, a magnetometer, a gyro at rest and a GNSS receiver give, at truth date jd and time t, for the held
 * attitude TK_Q (ECI to body): a circular orbit, the field and the Sun from the models */
static adcs_real TK_Q[4];
static adcs_real TK_GH[195];
static const adcs_real TK_EOP[3] = {0, 0, 0};       /* the blobs of these scenes hold no Earth orientation: dUT1, xp, yp = 0 */

static void tk_scene_init(void)
{
    adcs_real q[4] = {0.2, -0.1, 0.4, 0.8};
    adcs_qnorm(q); memcpy(TK_Q, q, sizeof q);
    adcs_igrf_gh(adcs_decyear(TK_JD0), TK_GH);
}

static void tk_feed_at(double t, double jd, const adcs_real r[3], const adcs_real v[3]);
static void tk_feed(double t, double jd)
{
    double n = 0.0011, rr = 6.9e6, inc = 97.0*ADCS_PI/180.0, u = n*t;
    adcs_real r[3] = {rr*cos(u), rr*sin(u)*cos(inc), rr*sin(u)*sin(inc)};
    adcs_real v[3] = {-rr*n*sin(u), rr*n*cos(u)*cos(inc), rr*n*cos(u)*sin(inc)};
    tk_feed_at(t, jd, r, v);
}

/* the same scene at a given position and velocity (a test that wants the Sun and the field at an angle of its choosing) */
static void tk_feed_at(double t, double jd, const adcs_real r[3], const adcs_real v[3])
{
    adcs_real A[3][3], B[3], Bb[3], S[3], Sb[3];
    uint8_t pl[33], fr[48], i; uint16_t c;
    double ds = (jd - 2451545.0)*86400.0, ws = floor(ds);
    (void)t;        /* the fix's date after its velocity: seconds and microseconds from J2000.0 */
    adcs_dcm(TK_Q, A);
    adcs_field_eci(r, jd, TK_EOP, TK_GH, 10, B); adcs_mat3_vec(A, B, Bb);
    adcs_sun_model(jd, S); adcs_mat3_vec(A, S, Sb);
    HS.mag[0] = 1; for (i = 0; i < 3; i++) put16(HS.mag + 1 + 2*i, Bb[i]/ADCS_MAG_LSB_T);
    HS.sun[0] = 1; for (i = 0; i < 3; i++) put16(HS.sun + 1 + 2*i, Sb[i]*ADCS_Q15);
    HS.gyro[0] = 1;       /* a rate of zero with counts that change, as a gyro's do (a repeated sample is a quiet gyro, F2) */
    { static unsigned tick_n; for (i = 0; i < 3; i++) put32(HS.gyro + 1 + 4*i, (double)((tick_n*7 + i*3) % 5)); tick_n++; }
    memset(pl, 0, sizeof pl); pl[0] = 1;
    for (i = 0; i < 3; i++) { put32(pl + 1 + 4*i, r[i]*100.0); put32(pl + 13 + 4*i, v[i]*1000.0); }
    put32(pl + 25, ws); put32(pl + 29, (ds - ws)*1e6);
    fr[0] = ADCS_ST_SYNC1; fr[1] = ADCS_GPS_SYNC2; fr[2] = 33; memcpy(fr + 3, pl, 33);
    c = adcs_crc16(pl, 33); fr[36] = (uint8_t)(c & 0xFF); fr[37] = (uint8_t)(c >> 8);
    hal_stub_uart_push(ADCS_GPS_UART, fr, 38);
}

/* the angle [deg] between the flight software's attitude estimate and the held attitude (180: none) */
static double tk_err_deg(void)
{
    double d[8]; adcs_real q[4];
    adcs_fsw_debug(d, 8);
    if (d[2] == 0.0) return 180.0;
    q[0] = d[3]; q[1] = d[4]; q[2] = d[5]; q[3] = d[6];
    return adcs_qangle(TK_Q, q)*180.0/ADCS_PI;
}

/* run the scene for secs of truth time from t0, the clock being clock(t); the worst attitude error over the last 20 s */
static double tk_fly(double age, double t0, double secs, double (*clock)(double, double), double param)
{
    double worst = 0; int k, n = (int)(secs/0.1 + 0.5);
    for (k = 0; k < n; k++) {
        double t = t0 + k*0.1;
        tk_feed(t, TK_JD0 + (age + t)/TK_DAY);
        HS.now_ns = (uint64_t)(clock(t, param)*1e9 + 0.5);
        adcs_fsw_step(HS.now_ns);
        if (k >= n - 200) { double e = tk_err_deg(); if (e > worst) worst = e; }
    }
    return worst;
}
static double tk_clock_age(double t, double age) { return age + t; }
static double tk_clock_step(double t, double step) { return t + step; }

static void tk_reset_day30(void)
{
    double age = 30.0*TK_DAY, before, after;
    tk_scene_init(); hal_stub_reset();
    tk_init(ADCS_MODE_NADIR_MTQ, 1, 0);
    before = tk_fly(age, 0.0, 120.0, tk_clock_age, age);
    CHECK(before < 1.0, "before the reset the attitude is known to %g deg (the rig itself)", before);
    tk_init(ADCS_MODE_NADIR_MTQ, 1, (uint64_t)((age + 120.0)*1e9));       /* the warm reset: the clock goes on */
    after = tk_fly(age, 120.0, 120.0, tk_clock_age, age);
    CHECK(after < 1.0, "120 s after the reset the attitude is known to %g deg: the date went back by 30 days", after);
}

static void tk_step_plus_1h(void)
{
    double before, after;
    tk_scene_init(); hal_stub_reset();
    tk_init(ADCS_MODE_NADIR_MTQ, 1, 0);
    before = tk_fly(0.0, 0.0, 120.0, tk_clock_step, 0.0);
    CHECK(before < 1.0, "before the step the attitude is known to %g deg (the rig itself)", before);
    after = tk_fly(0.0, 120.0, 240.0, tk_clock_step, 3600.0);
    CHECK(after < 1.0, "240 s after a +1 h step the attitude is known to %g deg: the date moved by an hour", after);
}

static void tk_step_minus_1s(void)
{
    int k; double last = 0, d[1];
    hal_stub_reset();
    tk_init(ADCS_MODE_DETUMBLE, 0, 0);
    for (k = 0; k < 300; k++) {
        HS.now_ns = (uint64_t)((k*0.1 - (k >= 150 ? 1.0 : 0.0))*1e9 + 0.5);
        HS.mag[0] = 1; HS.gyro[0] = 1;
        adcs_fsw_step(HS.now_ns);
        adcs_fsw_debug(d, 1);
        CHECK(d[0] >= last, "the time ran back from %g s to %g s at tick %d", last, d[0], k);
        if (d[0] < last) break;
        last = d[0];
    }
}

static void tk_step_back_past_start(void)
{
    int k; double d[1];
    hal_stub_reset();
    tk_init(ADCS_MODE_DETUMBLE, 0, 100000000000ull);
    for (k = 0; k < 50; k++) {
        HS.now_ns = (uint64_t)((100.0 + k*0.1 - (k >= 5 ? 1.0 : 0.0))*1e9 + 0.5);
        HS.mag[0] = 1; HS.gyro[0] = 1;
        adcs_fsw_step(HS.now_ns);
        adcs_fsw_debug(d, 1);
        CHECK(d[0] < 3600.0, "the time reads %g s at tick %d (a wrap of the unsigned difference)", d[0], k);
        if (d[0] >= 3600.0) break;
    }
}

/* F2: 60 s of detumble at a steady rate; the gyro goes quiet at 10 s (silent: it stops answering; else it answers with its last
 * sample, flagged ok): the stale fault must be raised within the limit and the rate no longer fed */
static void tk_quiet_gyro(int silent)
{
    int k, i, flagged = -1; double w10 = -1;
    adcs_fsw_state_t st; double d[13];
    hal_stub_reset();
    tk_init(ADCS_MODE_DETUMBLE, 0, 0);
    for (k = 0; k < 600; k++) {
        double B[3] = {2e-5, -1e-5, 3e-5}, w[3] = {0.05, -0.02, 0.1};
        HS.now_ns = (uint64_t)k*100000000ull;
        HS.mag[0] = 1; for (i = 0; i < 3; i++) put16(HS.mag + 1 + 2*i, B[i]/ADCS_MAG_LSB_T);
        if (k < 100) { HS.gyro[0] = 1; for (i = 0; i < 3; i++) put32(HS.gyro + 1 + 4*i, w[i]/ADCS_GYRO_LSB + (double)((k*7 + i*3) % 5)); }
        else HS.gyro_silent = silent;
        adcs_fsw_step(HS.now_ns);
        if (k >= 100 && k % 10 == 9) {
            int s = (k - 99)/10;
            adcs_fsw_peek(&st); adcs_fsw_debug(d, 13);
            if (flagged < 0 && (st.faults & 0x400)) flagged = s;     /* bit 10: the quiet gyro (the 60 s fault is bit 9) */
            if (s == (int)TK_STALE_LIMIT_S) w10 = sqrt(d[10]*d[10] + d[11]*d[11] + d[12]*d[12]);
        }
    }
    CHECK(flagged > 0 && flagged <= (int)TK_STALE_LIMIT_S, "the gyro stale fault is raised %d s after the gyro went quiet (-1: never within 50; the limit is %g s)", flagged, TK_STALE_LIMIT_S);
    CHECK(w10 >= 0 && w10 < 0.1*0.114, "%g s after, the controllers are still fed a rate of %g rad/s from the quiet gyro", TK_STALE_LIMIT_S, w10);
}
static void tk_gyro_stops_answering(void) { tk_quiet_gyro(1); }
static void tk_gyro_frozen(void) { tk_quiet_gyro(0); }

/* F5: the detumble exit's rate source (DR-gdn-003). 40 s of detumble with the exit at 0.01 rad/s held for 5 s, then the next state.
 * The field turns in the body at spin rad/s about z (a tumbling body); the gyro reads w_gyro with counts that change, as a gyro's do;
 * gyro says whether the product has one, fail_at the second it stops answering (-1: never). Returns the second the mode left detumble, -1: it did not. */
static int tk_exit_run(int gyro, double w_gyro, double spin, int fail_at)
{
    static adcs_params_t p; static uint8_t blob[ADCS_PARAMS_BLOB_SIZE];
    adcs_fsw_init_t in; adcs_fsw_state_t st; int k, i;
    tk_params(&p, ADCS_MODE_DETUMBLE, 0);
    p.has_gyro = (uint8_t)gyro; p.auto_next = ADCS_MODE_NADIR_MTQ; p.detumble_exit = 0.01; p.detumble_hold_s = 5.0;
    adcs_params_encode(&p, blob, sizeof blob);
    hal_stub_reset();
    in.abi_version = ADCS_FSW_ABI_VERSION; in.config_blob = blob; in.config_len = sizeof blob; in.start_ns = 0;
    if (adcs_fsw_init(&in) != 0) return -2;
    for (k = 0; k < 400; k++) {
        double t = k*0.1, c = cos(spin*t), s = sin(spin*t), b0[3] = {2e-5, -1e-5, 3e-5};
        double b[3] = {b0[0]*c + b0[1]*s, -b0[0]*s + b0[1]*c, b0[2]};
        HS.now_ns = (uint64_t)k*100000000ull;
        HS.mag[0] = 1; for (i = 0; i < 3; i++) put16(HS.mag + 1 + 2*i, b[i]/ADCS_MAG_LSB_T);
        if (gyro) {
            HS.gyro[0] = 1; for (i = 0; i < 3; i++) put32(HS.gyro + 1 + 4*i, w_gyro/ADCS_GYRO_LSB + (double)((k*7 + i*3) % 5));
            HS.gyro_silent = fail_at >= 0 && k >= fail_at*10;
        }
        adcs_fsw_step(HS.now_ns);
        adcs_fsw_peek(&st);
        if (st.mode != ADCS_MODE_DETUMBLE) return k/10;
    }
    return -1;
}
static void tk_exit_healthy_gyro(void)
{
    CHECK(tk_exit_run(1, 0.0, 0.0, -1) >= 0, "a still body with a healthy gyro never left detumble");
    CHECK(tk_exit_run(1, 0.1, 0.0, -1) < 0, "a gyro reading 0.1 rad/s let the body leave detumble");
    CHECK(tk_exit_run(1, 0.0, 0.1, -1) >= 0, "the field's turn kept a body the healthy gyro reads still in detumble: the gyro judges");
}
static void tk_exit_no_gyro(void)
{
    CHECK(tk_exit_run(0, 0.0, 0.0, -1) >= 0, "a still field with no gyro never let the body go");
    CHECK(tk_exit_run(0, 0.0, 0.1, -1) < 0, "a body whose field turns at 0.1 rad/s left detumble after the hold, with no gyro");
}
static void tk_exit_gyro_fails(void)
{
    CHECK(tk_exit_run(1, 0.0, 0.1, 3) < 0, "a gyro that failed at 3 s let the tumbling body leave detumble");
}

/* F4, G1 (S11.6): the estimator's start. A star-tracker frame of one head, valid, carrying the quaternion q as it is (its norm is the caller's) */
void adcs_fsw_est_counts(uint32_t out[2]);

static void tk_st_frame(const adcs_real q[4])
{
    uint8_t f[64], pl[18]; int k; uint16_t c;
    pl[0] = 1; pl[1] = 1;
    for (k = 0; k < 4; k++) put32(pl + 2 + 4*k, q[k]*ADCS_Q30);
    f[0] = ADCS_ST_SYNC1; f[1] = ADCS_ST_SYNC2; f[2] = 18;
    memcpy(f + 3, pl, 18); c = adcs_crc16(pl, 18); f[21] = (uint8_t)(c & 0xFF); f[22] = (uint8_t)(c >> 8);
    hal_stub_uart_push(ADCS_ST_UART, f, 23);
}

/* `secs` of the scene at the fixed position r (no velocity), the tracker's frame the held attitude times st_scale (0: no tracker,
 * has_st and heads as given); returns the attitude error [deg] at the end (180: no attitude); wrong_mag_from: the second from which
 * the field is read turned a quarter turn about x (negative: never); rej_max, gate for the filter's divergence test */
static double tk_start_run(int sun_gps, int has_st, int heads, double st_scale, const adcs_real r[3], double secs,
                           double wrong_mag_from, double rej_max, double gate, uint32_t counts_at_60[2])
{
    static adcs_params_t p; static uint8_t blob[ADCS_PARAMS_BLOB_SIZE];
    adcs_fsw_init_t in; int k, n = (int)(secs/0.1 + 0.5);
    adcs_real v[3] = {0, 0, 0};
    tk_scene_init(); hal_stub_reset();
    tk_params(&p, ADCS_MODE_NADIR_MTQ, sun_gps);
    if (has_st) {
        p.has_st = 1; p.n_heads = (uint8_t)heads; p.st_bs[0][2] = 1; p.st_bs[1][2] = 1; p.st_noise_cross = 1e-4; p.st_noise_roll = 1e-3;
    }
    p.mekf_gate = gate; p.mekf_rej_max = rej_max;
    adcs_params_encode(&p, blob, sizeof blob);
    in.abi_version = ADCS_FSW_ABI_VERSION; in.config_blob = blob; in.config_len = sizeof blob; in.start_ns = 0;
    CHECK(adcs_fsw_init(&in) == 0, "init (estimator start test)");
    for (k = 0; k < n; k++) {
        double t = k*0.1;
        tk_feed_at(t, TK_JD0 + t/TK_DAY, r, v);
        if (wrong_mag_from >= 0 && t >= wrong_mag_from) {       /* the field turned a quarter turn about the body's x axis: a magnetometer wrong for good */
            int16_t y = (int16_t)(HS.mag[3] | (HS.mag[4] << 8)), z = (int16_t)(HS.mag[5] | (HS.mag[6] << 8)), ny = (int16_t)-z;
            HS.mag[3] = (uint8_t)(ny & 0xFF); HS.mag[4] = (uint8_t)((ny >> 8) & 0xFF); HS.mag[5] = (uint8_t)(y & 0xFF); HS.mag[6] = (uint8_t)((y >> 8) & 0xFF);
        }
        if (has_st && st_scale > 0) { adcs_real q[4]; int i; for (i = 0; i < 4; i++) q[i] = st_scale*TK_Q[i]; tk_st_frame(q); }
        HS.now_ns = (uint64_t)(t*1e9 + 0.5);
        adcs_fsw_step(HS.now_ns);
        if (counts_at_60 && k == 599) adcs_fsw_est_counts(counts_at_60);
    }
    return tk_err_deg();
}

static const adcs_real TK_R0[3] = {6.9e6, 0, 0};

static void tk_st_norm_half(void)
{
    uint32_t c[2]; double e = tk_start_run(1, 1, 1, 0.5, TK_R0, 60.0, -1, 0, 0, NULL);
    adcs_fsw_est_counts(c);
    CHECK(c[0] > 0, "no refusal counted for a frame of norm 0.5");
    CHECK(e < 1.0, "the filter started from the frame of norm 0.5 (or from nothing): its attitude is %g deg off", e);
}

static void tk_st_unit_starts(void)
{
    uint32_t c[2]; double e = tk_start_run(1, 1, 1, 1.0, TK_R0, 5.0, -1, 0, 0, NULL);
    adcs_fsw_est_counts(c);
    CHECK(e < 0.01, "the filter started from a unit frame is %g deg off", e);
    CHECK(c[0] == 0, "a unit frame was refused");
}

static void tk_no_head_none_selected(void)
{
    double e = tk_start_run(0, 1, 0, 1.0, TK_R0, 30.0, -1, 0, 0, NULL);
    CHECK(e == 180.0, "a head the product does not carry started the filter (%g deg off)", e);
}

/* the angle [deg] between the Sun and the field at the position r, at the date of the scene's start */
static double tk_sun_field_deg(const adcs_real r[3])
{
    adcs_real s[3], b[3], c;
    adcs_sun_model(TK_JD0, s); adcs_field_eci(r, TK_JD0, TK_EOP, TK_GH, 10, b);
    c = adcs_dot(s, b)/(adcs_norm3(s)*adcs_norm3(b));
    if (c > 1) c = 1;
    if (c < -1) c = -1;
    return acos(c)*180.0/ADCS_PI;
}

/* the position on the orbit's sphere where the Sun and the field are closest to deg apart (a grid of 1 deg) */
static double tk_position_at_angle(double deg, adcs_real best[3])
{
    double bd = 1e9; int la, lo;
    for (la = -89; la < 90; la++) for (lo = 0; lo < 360; lo++) {
        double a = la*ADCS_PI/180.0, l = lo*ADCS_PI/180.0, d;
        adcs_real r[3] = {6.9e6*cos(a)*cos(l), 6.9e6*cos(a)*sin(l), 6.9e6*sin(a)};
        d = fabs(tk_sun_field_deg(r) - deg);
        if (d < bd) { bd = d; best[0] = r[0]; best[1] = r[1]; best[2] = r[2]; }
    }
    return bd;
}

static void tk_triad_refused_5deg(void)
{
    adcs_real near[3], far[3]; double e;
    tk_scene_init();
    CHECK(tk_position_at_angle(5.0, near) < 0.5, "no position with the Sun and the field 5 deg apart");
    CHECK(tk_position_at_angle(60.0, far) < 0.5, "no position with the Sun and the field 60 deg apart");
    e = tk_start_run(1, 0, 0, 0, near, 30.0, -1, 0, 0, NULL);
    CHECK(e == 180.0, "TRIAD started the filter with the Sun and the field %.1f deg apart (%g deg off)", tk_sun_field_deg(near), e);
    e = tk_start_run(1, 0, 0, 0, far, 30.0, -1, 0, 0, NULL);
    CHECK(e < 1.0, "TRIAD with the Sun and the field 60 deg apart left the filter %g deg off", e);
}

static void tk_sensor_rejected_for_good(void)
{
    uint32_t c60[2] = {0, 0}, c[2]; double e = tk_start_run(1, 0, 0, 0, TK_R0, 120.0, 60.0, 20.0, 11.34, c60);
    adcs_fsw_est_counts(c);
    CHECK(e < 180.0, "the filter did not start at all");
    CHECK(c60[1] == 0, "the filter restarted with every sensor right");
    CHECK(c[1] > c60[1], "a magnetometer wrong for 60 s and a Sun sensor right never restarted the filter");
}

/* ---- bus faults at the HAL (docs/S11_INVENTORY.md O11, S11.14; fsw-rs/tests/bus_faults.rs holds the same) ----
 * One test per fault point of the device map (fsw/devices/example_3u.toml) and per bus fault (a NACK, a CRC wrong, a register stuck, a
 * supply drop, a CAN error frame, a write that fails): does the flight driver raise its fault flag? Where one does (the magnetometer's
 * and the gyro's) the test runs in main; where the drivers raise none it is in the skipped list as a FINDING of S11.14 (ADCS_RUN_SKIPPED=1
 * shows what each says today). The scene is the tick tests': exact sensors, a held attitude, 150 s with the fault in force from 10 s. */
enum { BF_MAG = 1, BF_SUN, BF_ES, BF_ST, BF_GYRO, BF_GNSS, BF_COILS, BF_CAN };
enum { BF_NACK = 1, BF_CRC, BF_STUCK, BF_BROWNOUT, BF_CANERR };

/* the flight software's fault word after the run (HS.writes_failed counts the output writes the bus refused) */
static unsigned bf_fly(int dev, int kind)
{
    static adcs_params_t p; static uint8_t blob[ADCS_PARAMS_BLOB_SIZE];
    adcs_fsw_init_t in; adcs_fsw_state_t st; int k, i;
    tk_scene_init(); hal_stub_reset();
    tk_params(&p, ADCS_MODE_NADIR_MTQ, 1); p.has_es = 1; p.has_st = 1;
    adcs_params_encode(&p, blob, sizeof blob);
    in.abi_version = ADCS_FSW_ABI_VERSION; in.config_blob = blob; in.config_len = sizeof blob; in.start_ns = 0;
    adcs_fsw_init(&in);
    HS.bf_dev = dev; HS.bf_kind = kind;
    for (k = 0; k < 1500; k++) {
        double t = k*0.1;
        HS.bf_on = t >= 10.0;
        tk_feed(t, TK_JD0 + t/TK_DAY);
        HS.es[0] = 1;
        if (k % 10 == 0) tk_st_frame(TK_Q);
        for (i = 0; i < 3; i++) {
            adcs_can_frame_t f; memset(&f, 0, sizeof f); f.id = ADCS_CAN_ROTOR_TM + (uint32_t)i; f.dlc = 8;
            if (HS.n_can_rx < 32) HS.can_rx[HS.n_can_rx++] = f;
        }
        HS.now_ns = (uint64_t)k*100000000ull;
        adcs_fsw_step(HS.now_ns);
        HS.n_can_tx = 0;
    }
    adcs_fsw_peek(&st);
    return st.faults;
}

static void bf_expect(int dev, int kind, unsigned mask, const char *what)
{
    unsigned clean = bf_fly(0, 0), got;
    CHECK((clean & mask) == 0, "%s: the scene raises a flag of %x by itself (word %x)", what, mask, clean);
    got = bf_fly(dev, kind);
    CHECK((got & mask) != 0, "%s: the flight software raised no fault flag of %x (word %x)", what, mask, got);
}
#define BF_ANY 0xFFFFu
#define BF_MAG_STALE 0x100u
#define BF_GYRO_FLAGS 0x600u
static void bf_mag_nack(void) { bf_expect(BF_MAG, BF_NACK, BF_MAG_STALE, "magnetometer NACK"); }
static void bf_mag_brownout(void) { bf_expect(BF_MAG, BF_BROWNOUT, BF_MAG_STALE, "magnetometer supply drop"); }
static void bf_mag_stuck(void) { bf_expect(BF_MAG, BF_STUCK, BF_MAG_STALE, "magnetometer register stuck"); }
static void bf_gyro_nack(void) { bf_expect(BF_GYRO, BF_NACK, BF_GYRO_FLAGS, "gyro NACK"); }
static void bf_gyro_brownout(void) { bf_expect(BF_GYRO, BF_BROWNOUT, BF_GYRO_FLAGS, "gyro supply drop"); }
static void bf_gyro_stuck(void) { bf_expect(BF_GYRO, BF_STUCK, BF_GYRO_FLAGS, "gyro register stuck"); }
static void bf_sun_nack(void) { bf_expect(BF_SUN, BF_NACK, BF_ANY, "Sun sensor NACK"); }
static void bf_sun_stuck(void) { bf_expect(BF_SUN, BF_STUCK, BF_ANY, "Sun sensor register stuck"); }
static void bf_sun_brownout(void) { bf_expect(BF_SUN, BF_BROWNOUT, BF_ANY, "Sun sensor supply drop"); }
static void bf_es_nack(void) { bf_expect(BF_ES, BF_NACK, BF_ANY, "Earth sensor NACK"); }
static void bf_es_stuck(void) { bf_expect(BF_ES, BF_STUCK, BF_ANY, "Earth sensor register stuck"); }
static void bf_es_brownout(void) { bf_expect(BF_ES, BF_BROWNOUT, BF_ANY, "Earth sensor supply drop"); }
static void bf_st_nack(void) { bf_expect(BF_ST, BF_NACK, BF_ANY, "star tracker silent"); }
static void bf_st_crc(void) { bf_expect(BF_ST, BF_CRC, BF_ANY, "star tracker CRC wrong"); }
static void bf_st_stuck(void) { bf_expect(BF_ST, BF_STUCK, BF_ANY, "star tracker frame stuck"); }
static void bf_st_brownout(void) { bf_expect(BF_ST, BF_BROWNOUT, BF_ANY, "star tracker supply drop"); }
static void bf_gnss_nack(void) { bf_expect(BF_GNSS, BF_NACK, BF_ANY, "GNSS receiver silent"); }
static void bf_gnss_crc(void) { bf_expect(BF_GNSS, BF_CRC, BF_ANY, "GNSS receiver CRC wrong"); }
static void bf_gnss_stuck(void) { bf_expect(BF_GNSS, BF_STUCK, BF_ANY, "GNSS receiver frame stuck"); }
static void bf_gnss_brownout(void) { bf_expect(BF_GNSS, BF_BROWNOUT, BF_ANY, "GNSS receiver supply drop"); }
static void bf_can_nack(void) { bf_expect(BF_CAN, BF_NACK, 0x7u, "momentum devices silent"); }
static void bf_can_error(void) { bf_expect(BF_CAN, BF_CANERR, 0x7u, "CAN error frame"); }
static void bf_pwm_write_error(void) { bf_expect(BF_COILS, BF_NACK, BF_ANY, "PWM write error"); }
static void bf_can_write_error(void) { bf_expect(BF_CAN, BF_NACK, BF_ANY, "CAN send error"); }

/* the rig itself: no fault, no flag; a failing write is a status the driver gets */
static void bf_rig(void)
{
    CHECK(bf_fly(0, 0) == 0, "the scene raises a flag with no fault");
    bf_fly(BF_COILS, BF_NACK);
    CHECK(HS.writes_failed > 1000, "the coils' writes failed %u times: the fault does not reach the HAL", (unsigned)HS.writes_failed);
}

/* The tick tests a fix has made pass: in main's run. A fix moves its tests here from the skipped list below. */
static const struct { const char *name; void (*fn)(void); } ticks[] = {
    { "timestamps off by 1 us change no command (F3)", tk_jitter_1us },
    { "timestamps off by 5 ms change no command (F3)", tk_jitter_5ms },
    { "a warm reset 30 days in keeps the date (F1)", tk_reset_day30 },
    { "a clock step of +1 h keeps the date (F1)", tk_step_plus_1h },
    { "a clock step of -1 s never runs the time back (F1)", tk_step_minus_1s },
    { "a clock step back past the start does not wrap (F1)", tk_step_back_past_start },
    { "a gyro that stops answering is flagged, its rate dropped (F2)", tk_gyro_stops_answering },
    { "a gyro frozen on its last sample is flagged, rate dropped (F2)", tk_gyro_frozen },
    { "a healthy gyro leaves detumble on its rate (F5)", tk_exit_healthy_gyro },
    { "with no gyro a tumbling body does not leave detumble (F5)", tk_exit_no_gyro },
    { "a gyro failing in detumble does not let a tumbling body go (F5)", tk_exit_gyro_fails },
    { "a tracker frame of norm 0.5 is refused at the start and counted (F4)", tk_st_norm_half },
    { "a unit tracker frame starts the filter (F4)", tk_st_unit_starts },
    { "with no head configured none is ever selected (F4)", tk_no_head_none_selected },
    { "TRIAD is refused when the Sun and the field are 5 deg apart (G1)", tk_triad_refused_5deg },
    { "a sensor rejected for good restarts the filter while another is accepted (G1)", tk_sensor_rejected_for_good },
    { "bus fault rig: no flag with no fault, a failing write reaches the HAL (O11)", bf_rig },
    { "magnetometer NACK: fault raised (O11)", bf_mag_nack },
    { "magnetometer supply drop: fault raised (O11)", bf_mag_brownout },
    { "gyro NACK: fault raised (O11)", bf_gyro_nack },
    { "gyro supply drop: fault raised (O11)", bf_gyro_brownout },
    { "gyro register stuck: fault raised (O11)", bf_gyro_stuck },
};

static void t_ticks(void)
{
    size_t i;
    for (i = 0; i < sizeof ticks/sizeof ticks[0]; i++) {
        int f0 = n_fail;
        ticks[i].fn();
        if (n_fail > f0) printf("FAIL %s\n", ticks[i].name);
    }
}

/* The skipped list: the tick tests no fix has made pass yet. The step that fixes one (named with it) moves it into main's run. */
static const struct { const char *name; void (*fn)(void); const char *fixed_by; } skipped[] = {
    { NULL, NULL, NULL },       /* none: F2 (S11.5) was the last */
    /* S11.14 (O11) findings: the bus faults no driver flags. The step that gives the driver its flag moves its test into ticks[] */
    { "magnetometer register stuck: fault raised (O11)", bf_mag_stuck, "a driver flag for a repeated field (S11.14 finding)" },
    { "Sun sensor NACK: fault raised (O11)", bf_sun_nack, "a driver flag for the Sun sensor (S11.14 finding)" },
    { "Sun sensor register stuck: fault raised (O11)", bf_sun_stuck, "a driver flag for the Sun sensor (S11.14 finding)" },
    { "Sun sensor supply drop: fault raised (O11)", bf_sun_brownout, "a driver flag for the Sun sensor (S11.14 finding)" },
    { "Earth sensor NACK: fault raised (O11)", bf_es_nack, "a driver flag for the Earth sensor (S11.14 finding)" },
    { "Earth sensor register stuck: fault raised (O11)", bf_es_stuck, "a driver flag for the Earth sensor (S11.14 finding)" },
    { "Earth sensor supply drop: fault raised (O11)", bf_es_brownout, "a driver flag for the Earth sensor (S11.14 finding)" },
    { "star tracker silent: fault raised (O11)", bf_st_nack, "a driver flag for the star tracker (S11.14 finding)" },
    { "star tracker CRC wrong: fault raised (O11)", bf_st_crc, "a driver flag for the star tracker (S11.14 finding)" },
    { "star tracker frame stuck: fault raised (O11)", bf_st_stuck, "a driver flag for the star tracker (S11.14 finding)" },
    { "star tracker supply drop: fault raised (O11)", bf_st_brownout, "a driver flag for the star tracker (S11.14 finding)" },
    { "GNSS receiver silent: fault raised (O11)", bf_gnss_nack, "a driver flag for the GNSS receiver (S11.14 finding)" },
    { "GNSS receiver CRC wrong: fault raised (O11)", bf_gnss_crc, "a driver flag for the GNSS receiver (S11.14 finding)" },
    { "GNSS receiver frame stuck: fault raised (O11)", bf_gnss_stuck, "a driver flag for the GNSS receiver (S11.14 finding)" },
    { "GNSS receiver supply drop: fault raised (O11)", bf_gnss_brownout, "a driver flag for the GNSS receiver (S11.14 finding)" },
    { "momentum devices silent: fault raised (O11)", bf_can_nack, "a rotor flag outside the momentum states (S11.14 finding)" },
    { "CAN error frame: fault raised (O11)", bf_can_error, "a rotor flag outside the momentum states (S11.14 finding)" },
    { "PWM write error: fault raised (TS4, O11)", bf_pwm_write_error, "a check of the write's status in the driver (S11.14 finding)" },
    { "CAN send error: fault raised (O11)", bf_can_write_error, "a check of the send's status in the driver (S11.14 finding)" },
};

/* without ADCS_RUN_SKIPPED the list is named and not run; with it every test runs and says what it shows today, outside the count */
static void t_skipped(void)
{
    size_t i;
    int run = getenv("ADCS_RUN_SKIPPED") != NULL;
    for (i = 0; i < sizeof skipped/sizeof skipped[0]; i++) {
        int p0 = n_pass, f0 = n_fail;
        if (!skipped[i].name) continue;
        if (!run) { printf("SKIP %s: %s fixes it\n", skipped[i].name, skipped[i].fixed_by); continue; }
        skipped[i].fn();
        printf("%s %s (%d checks failed)\n", n_fail > f0 ? "FAILS TODAY" : "PASSES", skipped[i].name, n_fail - f0);
        n_pass = p0; n_fail = f0;
    }
}

int main(void)
{
    t_math(); t_env(); t_estimation(); t_laws(); t_alloc(); t_rcs(); t_abi(); t_st_frame(); t_eop_tc(); t_nonfinite(); t_safe(); t_ticks(); t_skipped();
    printf("%d checks passed, %d failed (%s)\n", n_pass, n_fail, adcs_fsw_build_id());
    return n_fail ? 1 : 0;
}
