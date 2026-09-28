/*
 * test_fsw.c -- host unit tests of the reference flight software.
 * Reference numbers marked "twin" come from the MATLAB twin (asils.*) and are
 * the same vectors fsw-rs checks, so the three implementations agree by test.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
 */
#include <stdio.h>
#include <string.h>
#include "adcs_fsw.h"
#include "adcs_params.h"
#include "adcs_gnc.h"
#include "adcs_env.h"
#include "adcs_devices.h"
#include "adcs_drv.h"
#include "hal_stub.h"

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
    CHECK(fabs(B[0] - 29130.375576) < 1e-5 && fabs(B[1] + 193.674477) < 1e-5 && fabs(B[2] - 12967.840908) < 1e-5,
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
    adcs_triad(b[0], b[1], r[0], r[1], qm);
    CHECK(adcs_qangle(q, qm) < 1e-9, "TRIAD %g rad", adcs_qangle(q, qm));
    {   /* MEKF converges from a 10 deg error with two exact vectors, gyro at rest */
        adcs_mekf_t K; adcs_real q0[4], th[3] = {0.1, -0.12, 0.08}, dq[4], w[3] = {0, 0, 0};
        adcs_fromrotvec(th, dq); adcs_qmult(q, dq, q0);
        adcs_mekf_init(&K, q0, 0.2, 1e-3, 1e-6, 1e-8);
        for (k = 0; k < 200; k++) {
            adcs_mekf_predict(&K, w, 0.1);
            adcs_mekf_vector(&K, b[0], r[0], 1e-3); adcs_mekf_vector(&K, b[1], r[1], 1e-3);
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

int main(void)
{
    t_math(); t_env(); t_estimation(); t_laws(); t_rcs(); t_abi(); t_st_frame();
    printf("%d checks passed, %d failed (%s)\n", n_pass, n_fail, adcs_fsw_build_id());
    return n_fail ? 1 : 0;
}
