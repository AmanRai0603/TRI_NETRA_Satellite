/* adcs_ctl.c -- control laws (05), detumble and Sun spin (06); guidance (04) is adcs_guid.c.
 * Every law delegates to its translation of 05_control.pc (fsw/alg/src/control.c); the gains record
 * adcs_gains_t is passed field by field (the error the magnetorquer laws share, control::mtq_err, is
 * reached through them).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_gnc.h"
#include "adcs_alg_glue.h"

/* ---------------- control laws ---------------- */
/* written from the design: control::control_law (fsw/alg); no wd_ref (NULL) is a zero one */
void adcs_control_law(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                      adcs_real I_q[3], adcs_real dt, const adcs_gains_t *g, adcs_real J[3][3],
                      const adcs_real Hs[3], const adcs_real wd_ref[3], adcs_real tau[3])
{
    static const adcs_real zero[3] = {0, 0, 0};
    control_control_law_out o = control_control_law(gl_v4(q), gl_v3(w), gl_v4(q_ref), gl_v3(w_ref), gl_v3(I_q), dt, g->law,
                                                    gl_v3(g->Kp), gl_v3(g->Kd), gl_v3(g->Ki), gl_m33(&g->Klqr[0][0]), g->lambda,
                                                    g->phi, gl_v3(g->Gs), g->err_max, g->int_max, gl_m33(&J[0][0]), gl_v3(Hs),
                                                    gl_v3(wd_ref ? wd_ref : zero));
    gl_o3(o.tau, tau); gl_o3(o.i_q, I_q);
}

/* written from the design: control::mtq_pd (fsw/alg) */
void adcs_mtq_pd(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                 const adcs_gains_t *g, adcs_real tau[3])
{
    gl_o3(control_mtq_pd(gl_v4(q), gl_v3(w), gl_v4(q_ref), gl_v3(w_ref), gl_v3(g->Kp), gl_v3(g->Kd)), tau);
}

/* written from the design: control::sat_dipole (fsw/alg) */
void adcs_sat_dipole(adcs_real m[3], adcs_real m_max) { gl_o3(control_sat_dipole(gl_v3(m), m_max), m); }

/* written from the design: control::torque2dipole (fsw/alg) */
void adcs_torque2dipole(const adcs_real tau[3], const adcs_real B[3], adcs_real m_max, adcs_real m[3])
{
    gl_o3(control_torque2dipole(gl_v3(tau), gl_v3(B), m_max), m);
}

/* ---------------- detumble and Sun spin ---------------- */
/* written from the design: control::bdot (fsw/alg) */
void adcs_bdot(const adcs_real b1[3], const adcs_real b2[3], adcs_real dt, adcs_real Bn, adcs_real k, adcs_real m_max, adcs_real m[3])
{
    gl_o3(control_bdot(gl_v3(b1), gl_v3(b2), dt, Bn, k, m_max), m);
}

/* written from the design: control::gen_bdot (fsw/alg) */
void adcs_gen_bdot(const adcs_real B[3], const adcs_real Bd[3], const adcs_real wd[3], adcs_real k, adcs_real m0[3])
{
    gl_o3(control_gen_bdot(gl_v3(B), gl_v3(Bd), gl_v3(wd), k), m0);
}

/* written from the design: control::sun_spin (fsw/alg) */
void adcs_sun_spin(const adcs_real B[3], const adcs_real w[3], const adcs_real s[3], int eclipse, adcs_real J[3][3],
                   adcs_real spin_dps, adcs_real k1, adcs_real k2, adcs_real rz_floor, adcs_real m0[3])
{
    gl_o3(control_sun_spin(gl_v3(B), gl_v3(w), gl_v3(s), eclipse != 0, gl_m33(&J[0][0]), spin_dps, k1, k2, rz_floor), m0);
}

/* ---- magnetorquer-only literature laws (05_control.md, docs/MTQ_LITERATURE.md) ----
 * Every law returns a torque request; torque2dipole projects it on the plane normal to B
 * (m = B x tau / |B|^2), the Gamma(t) = I - b b^T reformulation of Lovera & Astolfi. */

/* P1 Lovera & Astolfi 2004, Prop. 1: u = -(eps^2 k_p q_v + eps k_v J w) */
/* written from the design: control::mtq_lovera (fsw/alg) */
void adcs_mtq_lovera(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                     adcs_real J[3][3], adcs_real eps, adcs_real kp, adcs_real kv, adcs_real tau[3])
{
    gl_o3(control_mtq_lovera(gl_v4(q), gl_v3(w), gl_v4(q_ref), gl_v3(w_ref), gl_m33(&J[0][0]), eps, kp, kv), tau);
}

/* P4 Celani 2015, Thm 2: u = -(eps^2 k1 q_v + eps k2 w), no inertia in the law */
/* written from the design: control::mtq_celani (fsw/alg) */
void adcs_mtq_celani(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                     adcs_real eps, adcs_real k1, adcs_real k2, adcs_real tau[3])
{
    gl_o3(control_mtq_celani(gl_v4(q), gl_v3(w), gl_v4(q_ref), gl_v3(w_ref), eps, k1, k2), tau);
}

/* P16 Avanzini, de Angelis & Giulietti 2021: pitch axis spun at the reference rate and aligned
 * with the orbit normal (fast), pitch error removed through eta(theta) (slow).
 * Returns 0 when the reference does not rotate (tau is then left as it is). */
/* written from the design: control::mtq_avanzini (fsw/alg) */
int adcs_mtq_avanzini(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                      adcs_real J[3][3], adcs_real k, adcs_real lam, adcs_real tau[3])
{
    control_mtq_avanzini_out o = control_mtq_avanzini(gl_v4(q), gl_v3(w), gl_v4(q_ref), gl_v3(w_ref), gl_m33(&J[0][0]), k, lam);
    if (!o.ok) return 0;
    gl_o3(o.tau, tau);
    return 1;
}

/* P8 Celani 2026: boresight e3 onto the target a (body), rotation about e3 left free:
 * m = b x (k_p (e3 x a) - k_d w); the rate is taken relative to the reference rate. */
/* written from the design: control::mtq_boresight (fsw/alg) */
void adcs_mtq_boresight(const adcs_real e3[3], const adcs_real a[3], const adcs_real we[3], adcs_real kp, adcs_real kd, adcs_real tau[3])
{
    gl_o3(control_mtq_boresight(gl_v3(e3), gl_v3(a), gl_v3(we), kp, kd), tau);
}

/* P3 TANGO (Chasset et al. 2013): LQR with P frozen at the averaged-Riccati P_ss; Pth = P21/r,
 * Pw = P22/r are computed on the ground. */
/* written from the design: control::mtq_tango (fsw/alg) */
void adcs_mtq_tango(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                    adcs_real Pth[3][3], adcs_real Pw[3][3], adcs_real tau[3])
{
    gl_o3(control_mtq_tango(gl_v4(q), gl_v3(w), gl_v4(q_ref), gl_v3(w_ref), gl_m33(&Pth[0][0]), gl_m33(&Pw[0][0])), tau);
}

/* P2 de Ruiter 2011 on the Sun line (06_detumble_sunspin.md): body z spins at spin_dps with
 * -z on the Sun, one Lyapunov function for spin axis, spin rate and nutation. */
/* written from the design: control::sun_spin_deruiter (fsw/alg) */
void adcs_sun_spin_deruiter(const adcs_real B[3], const adcs_real w[3], const adcs_real s[3], int eclipse, adcs_real J[3][3],
                            adcs_real spin_dps, adcs_real k, adcs_real k1, adcs_real k2, adcs_real m0[3])
{
    gl_o3(control_sun_spin_deruiter(gl_v3(B), gl_v3(w), gl_v3(s), eclipse != 0, gl_m33(&J[0][0]), spin_dps, k, k1, k2), m0);
}
