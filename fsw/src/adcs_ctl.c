/* adcs_ctl.c -- guidance (04), control laws (05), detumble and Sun spin (06).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_gnc.h"

/* ---------------- guidance ---------------- */
void adcs_guidance(int kind, const adcs_real r[3], const adcs_real v[3], adcs_real t, const adcs_guid_t *g,
                   adcs_real q_ref[4], adcs_real w_ref[3], adcs_real wd_ref[3])
{
    adcs_real rh[3], vh[3], nrm[3], ram[3], mr[3], R[3][3], q_nad[4], w_orb[3], A[3][3], ax[3], th[3], dq[4], c[3];
    int i;
    adcs_unit(r, rh); adcs_unit(v, vh);
    adcs_scale3(rh, -1.0, mr);
    adcs_cross(vh, mr, c); adcs_unit(c, nrm);
    adcs_cross(mr, nrm, c); adcs_unit(c, ram);
    for (i = 0; i < 3; i++) { R[0][i] = -ram[i]; R[1][i] = -rh[i]; R[2][i] = -nrm[i]; }
    adcs_fromdcm(R, q_nad);
    if (g->q_off[0] != 0 || g->q_off[1] != 0 || g->q_off[2] != 0 || g->q_off[3] != 0) adcs_qmult(q_nad, g->q_off, q_nad);
    if (g->flip) {           /* 180 deg about the boresight: q_f = [u, 0] */
        adcs_real u[3], qf[4];
        u[0] = 1; u[1] = 0; u[2] = 0;
        if (adcs_norm3(g->roll_axis) > 0) adcs_unit(g->roll_axis, u);
        qf[0] = u[0]; qf[1] = u[1]; qf[2] = u[2]; qf[3] = 0;
        adcs_qmult(q_nad, qf, q_nad);
    }
    adcs_cross(r, v, w_orb); adcs_scale3(w_orb, 1.0/adcs_dot(r, r), w_orb);
    adcs_zero3(wd_ref);
    ax[0] = 1; ax[1] = 0; ax[2] = 0;
    if (adcs_norm3(g->axis) > 0) adcs_unit(g->axis, ax);
    switch (kind) {
    case 1:          /* target */
        adcs_scale3(ax, g->roll_deg*ADCS_D2R, th); adcs_fromrotvec(th, dq); adcs_qmult(q_nad, dq, q_ref);
        adcs_dcm(q_ref, A); adcs_mat3_vec(A, w_orb, w_ref);
        break;
    case 2: {        /* cycloidal slew in the rotating nadir frame */
        adcs_real tau = adcs_clamp((t - g->t0)/g->T, 0.0, 1.0), sd = 0, sdd = 0, s, ph = g->roll_deg*ADCS_D2R, wo[3];
        if (t > g->t0 && t < g->t0 + g->T) { sd = (1 - cos(2*ADCS_PI*tau))/g->T; sdd = 2*ADCS_PI*sin(2*ADCS_PI*tau)/(g->T*g->T); }
        s = tau - sin(2*ADCS_PI*tau)/(2*ADCS_PI);
        adcs_scale3(ax, ph*s, th); adcs_fromrotvec(th, dq); adcs_qmult(q_nad, dq, q_ref);
        adcs_dcm(q_ref, A); adcs_mat3_vec(A, w_orb, wo);
        {   /* the slew frame turns at ax ph s_dot relative to the orbiting frame: transport term on w_orb */
            adcs_real wr[3], tr[3];
            adcs_scale3(ax, ph*sd, wr); adcs_cross(wr, wo, tr);
            for (i = 0; i < 3; i++) { w_ref[i] = wo[i] + ax[i]*ph*sd; wd_ref[i] = ax[i]*ph*sdd - tr[i]; }
        }
        break; }
    case 3:          /* inertial */
        for (i = 0; i < 4; i++) q_ref[i] = g->q_inertial[i];
        adcs_zero3(w_ref);
        break;
    case 4: {        /* Sun referencing */
        adcs_real a[3], b[3], s[3], e2[3], e3[3], b3[3], Bm[3][3], Em[3][3], M[3][3];
        int j;
        a[0] = 0; a[1] = 0; a[2] = -1;
        if (adcs_norm3(g->sun_axis) > 0) adcs_unit(g->sun_axis, a);
        b[0] = 1; b[1] = 0; b[2] = 0;
        if (adcs_norm3(g->roll_axis) > 0) adcs_copy3(g->roll_axis, b);
        {   /* b made normal to the power face */
            adcs_real ab = adcs_dot(a, b);
            for (i = 0; i < 3; i++) b[i] -= ab*a[i];
        }
        if (adcs_norm3(b) < 1e-6) { b[0] = -a[1]*a[0]; b[1] = 1 - a[1]*a[1]; b[2] = -a[1]*a[2]; }
        adcs_unit(b, b);
        adcs_unit(g->sun_eci, s);
        for (i = 0; i < 3; i++) e2[i] = nrm[i] - adcs_dot(nrm, s)*s[i];
        if (adcs_norm3(e2) < 1e-6) { e2[0] = -s[2]*s[0]; e2[1] = -s[2]*s[1]; e2[2] = 1 - s[2]*s[2]; }
        adcs_unit(e2, e2);
        adcs_cross(a, b, b3); adcs_cross(s, e2, e3);
        for (i = 0; i < 3; i++) { Bm[i][0] = a[i]; Bm[i][1] = b[i]; Bm[i][2] = b3[i]; Em[i][0] = s[i]; Em[i][1] = e2[i]; Em[i][2] = e3[i]; }
        for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) M[i][j] = Bm[i][0]*Em[j][0] + Bm[i][1]*Em[j][1] + Bm[i][2]*Em[j][2];
        adcs_fromdcm(M, q_ref);
        adcs_zero3(w_ref);
        break; }
    default:         /* nadir */
        for (i = 0; i < 4; i++) q_ref[i] = q_nad[i];
        adcs_dcm(q_nad, A); adcs_mat3_vec(A, w_orb, w_ref);
        break;
    }
}

/* Nadir-family yaw flip with hysteresis: turn 180 deg about the boresight when the power face would look
 * away from the Sun (04_guidance.md) */
void adcs_yaw_flip(adcs_guid_t *g, const adcs_real r[3], const adcs_real v[3], adcs_real hyst)
{
    adcs_guid_t g0 = *g;
    adcs_real q[4], w[3], wd[3], A[3][3], s[3], sb[3], a[3], d;
    g0.flip = 0;
    adcs_guidance(0, r, v, 0.0, &g0, q, w, wd);
    adcs_unit(g->sun_eci, s); adcs_dcm(q, A); adcs_mat3_vec(A, s, sb);
    a[0] = 0; a[1] = 0; a[2] = -1;
    if (adcs_norm3(g->sun_axis) > 0) adcs_unit(g->sun_axis, a);
    d = adcs_dot(a, sb);
    if (d < -hyst) g->flip = 1;
    else if (d > hyst) g->flip = 0;
}

void adcs_boresight_offset(const adcs_real bs_in[3], adcs_real q[4])
{
    adcs_real ey[3] = {0, 1, 0}, bs[3], ax[3], K[3][3], K2[3][3], A[3][3], c, s, k[3];
    int i, j;
    adcs_unit(bs_in, bs);
    adcs_cross(ey, bs, ax);
    c = adcs_dot(ey, bs); s = adcs_norm3(ax);
    if (s < 1e-12) {
        q[0] = 0; q[1] = 0; q[2] = 0; q[3] = 1;
        if (c <= 0) { q[0] = 1; q[3] = 0; }
        return;
    }
    adcs_scale3(ax, 1.0/s, k);
    adcs_skew(k, K); adcs_mat3_mul(K, K, K2);
    for (i = 0; i < 3; i++) for (j = 0; j < 3; j++) A[i][j] = (i == j ? 1.0 : 0.0) + s*K[i][j] + (1 - c)*K2[i][j];
    adcs_fromdcm(A, q);
}

/* ---------------- control laws ---------------- */
void adcs_control_law(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                      adcs_real I_q[3], adcs_real dt, const adcs_gains_t *g, adcs_real J[3][3],
                      const adcs_real Hs[3], const adcs_real wd_ref[3], adcs_real tau[3])
{
    adcs_real qe[4], e[3], we[3], A[3][3], wr[3], H[3], gyro[3], ff[3], Jw[3];
    int i;
    adcs_qerr(q_ref, q, qe);
    for (i = 0; i < 3; i++) e[i] = adcs_clamp(qe[i], -g->err_max, g->err_max);
    adcs_dcm(qe, A); adcs_mat3_vec(A, w_ref, wr); adcs_sub3(w, wr, we);
    adcs_mat3_vec(J, w, Jw); adcs_add3(Jw, Hs, H); adcs_cross(w, H, gyro);
    if (wd_ref) adcs_mat3_vec(J, wd_ref, ff); else adcs_zero3(ff);
    switch (g->law) {
    case 1:          /* lqr */
        for (i = 0; i < 3; i++) {
            I_q[i] = adcs_clamp(I_q[i] + 2*e[i]*dt, -g->int_max, g->int_max);
            tau[i] = -(g->Klqr[i][0]*I_q[i] + g->Klqr[i][1]*(2*e[i]) + g->Klqr[i][2]*we[i]);
        }
        break;
    case 2: {        /* sliding mode */
        adcs_real s[3], sat[3], ed[3], c[3], x[3];
        adcs_cross(e, we, c);
        for (i = 0; i < 3; i++) {
            s[i] = we[i] + g->lambda*e[i];
            sat[i] = adcs_clamp(s[i]/g->phi, -1.0, 1.0);
            ed[i] = 0.5*(qe[3]*we[i] + c[i]);
            x[i] = g->lambda*ed[i] + g->Gs[i]*sat[i];
        }
        adcs_mat3_vec(J, x, tau);
        adcs_scale3(tau, -1.0, tau);
        break; }
    default:         /* pid */
        for (i = 0; i < 3; i++) {
            I_q[i] = adcs_clamp(I_q[i] + e[i]*dt, -g->int_max, g->int_max);
            tau[i] = -g->Kp[i]*e[i] - g->Kd[i]*we[i] - g->Ki[i]*I_q[i];
        }
        break;
    }
    for (i = 0; i < 3; i++) tau[i] += gyro[i] + ff[i];
}

void adcs_mtq_pd(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                 const adcs_gains_t *g, adcs_real tau[3])
{
    adcs_real qc[4], qe[4], A[3][3], wr[3], s;
    int i;
    adcs_qconj(q_ref, qc); adcs_qmult(qc, q, qe);          /* no sign fix here: sign(qe.w) below */
    s = adcs_sign(qe[3]); if (s == 0) s = 1;
    adcs_dcm(qe, A); adcs_mat3_vec(A, w_ref, wr);
    for (i = 0; i < 3; i++) tau[i] = -g->Kp[i]*(s*qe[i]) - g->Kd[i]*(w[i] - wr[i]);
}

void adcs_sat_dipole(adcs_real m[3], adcs_real m_max)
{
    adcs_real a = 1.0; int i;
    for (i = 0; i < 3; i++) {
        adcs_real d = fabs(m[i]); if (d < 1e-30) d = 1e-30;
        if (m_max/d < a) a = m_max/d;
    }
    adcs_scale3(m, a, m);
}

void adcs_torque2dipole(const adcs_real tau[3], const adcs_real B[3], adcs_real m_max, adcs_real m[3])
{
    adcs_real Bs = adcs_dot(B, B);
    if (Bs < 1e-18) { adcs_zero3(m); return; }
    adcs_cross(B, tau, m); adcs_scale3(m, 1.0/Bs, m);
    adcs_sat_dipole(m, m_max);
}

/* ---------------- detumble and Sun spin ---------------- */
void adcs_bdot(const adcs_real b1[3], const adcs_real b2[3], adcs_real dt, adcs_real Bn, adcs_real k, adcs_real m_max, adcs_real m[3])
{
    adcs_real u1[3], u2[3]; int i;
    adcs_unit(b1, u1); adcs_unit(b2, u2);
    for (i = 0; i < 3; i++) m[i] = -(k/Bn)*(u2[i] - u1[i])/dt;
    adcs_sat_dipole(m, m_max);
}

void adcs_gen_bdot(const adcs_real B[3], const adcs_real Bd[3], const adcs_real wd[3], adcs_real k, adcs_real m0[3])
{
    adcs_real c[3]; int i;
    adcs_cross(wd, B, c);
    for (i = 0; i < 3; i++) m0[i] = -k*(Bd[i] + c[i]);
}

void adcs_sun_spin(const adcs_real B[3], const adcs_real w[3], const adcs_real s[3], int eclipse, adcs_real J[3][3],
                   adcs_real spin_dps, adcs_real k1, adcs_real k2, adcs_real rz_floor, adcs_real m0[3])
{
    adcs_real ws, sg, h[3], hd, ht[3], Rz[3], x[3], A[3], Bs, fl, su[3];
    int i;
    if (eclipse) { adcs_zero3(m0); return; }
    fl = rz_floor*J[2][2];
    ws = -fabs(spin_dps*ADCS_D2R);
    sg = adcs_sign(w[2]); if (sg == 0) sg = 1;
    adcs_mat3_vec(J, w, h);
    adcs_copy3(s, su);
    for (i = 0; i < 3; i++) { hd = sg*J[2][2]*ws*su[i]; ht[i] = h[i] - hd; }
    Rz[0] = J[2][2] - J[0][0]; if (Rz[0] < fl) Rz[0] = fl;
    Rz[1] = J[2][2] - J[1][1]; if (Rz[1] < fl) Rz[1] = fl;
    Rz[2] = 0;
    for (i = 0; i < 3; i++) x[i] = k1*ht[i] + k2*Rz[i]*w[i];
    adcs_cross(B, x, A);
    Bs = adcs_dot(B, B);
    if (Bs < 1e-18) { adcs_zero3(m0); return; }
    for (i = 0; i < 3; i++) m0[i] = -A[i]/Bs;
}

/* ---- magnetorquer-only literature laws (05_control.md, docs/MTQ_LITERATURE.md) ----
 * Every law returns a torque request; torque2dipole projects it on the plane normal to B
 * (m = B x tau / |B|^2), the Gamma(t) = I - b b^T reformulation of Lovera & Astolfi. */
static void mtq_err(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                    adcs_real qe[4], adcs_real *s, adcs_real wr[3], adcs_real we[3])
{
    adcs_real qc[4], A[3][3];
    int i;
    adcs_qconj(q_ref, qc); adcs_qmult(qc, q, qe);
    *s = adcs_sign(qe[3]); if (*s == 0) *s = 1;
    adcs_dcm(qe, A); adcs_mat3_vec(A, w_ref, wr);
    for (i = 0; i < 3; i++) we[i] = w[i] - wr[i];
}

/* P1 Lovera & Astolfi 2004, Prop. 1: u = -(eps^2 k_p q_v + eps k_v J w) */
void adcs_mtq_lovera(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                     adcs_real J[3][3], adcs_real eps, adcs_real kp, adcs_real kv, adcs_real tau[3])
{
    adcs_real qe[4], s, wr[3], we[3], Jw[3];
    int i;
    mtq_err(q, w, q_ref, w_ref, qe, &s, wr, we);
    adcs_mat3_vec(J, we, Jw);
    for (i = 0; i < 3; i++) tau[i] = -(eps*eps*kp*(s*qe[i]) + eps*kv*Jw[i]);
}

/* P4 Celani 2015, Thm 2: u = -(eps^2 k1 q_v + eps k2 w), no inertia in the law */
void adcs_mtq_celani(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                     adcs_real eps, adcs_real k1, adcs_real k2, adcs_real tau[3])
{
    adcs_real qe[4], s, wr[3], we[3];
    int i;
    mtq_err(q, w, q_ref, w_ref, qe, &s, wr, we);
    for (i = 0; i < 3; i++) tau[i] = -(eps*eps*k1*(s*qe[i]) + eps*k2*we[i]);
}

/* P16 Avanzini, de Angelis & Giulietti 2021: pitch axis spun at the reference rate and aligned
 * with the orbit normal (fast), pitch error removed through eta(theta) (slow).
 * e_p = w_ref/|w_ref| (body axis meant to spin), sigma = orbit normal seen in body now,
 * theta = pitch error to first order, eta = J_p n (1 - lambda theta),
 * tau = k (eta sigma - J w) + k (eta e_p - J w).  Returns 0 when the reference does not rotate. */
int adcs_mtq_avanzini(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                      adcs_real J[3][3], adcs_real k, adcs_real lam, adcs_real tau[3])
{
    adcs_real qe[4], s, wr[3], we[3], n, ep[3], sg[3], A[3][3], Jep[3], Jp, th, eta, Jw[3];
    int i;
    n = adcs_norm3(w_ref);
    if (n < 1e-9) return 0;
    mtq_err(q, w, q_ref, w_ref, qe, &s, wr, we);
    for (i = 0; i < 3; i++) ep[i] = w_ref[i]/n;
    adcs_dcm(qe, A); adcs_mat3_vec(A, ep, sg);
    adcs_mat3_vec(J, ep, Jep); Jp = adcs_dot(ep, Jep);
    th = 2*s*adcs_dot(qe, ep);
    eta = Jp*n*(1 - lam*th);
    adcs_mat3_vec(J, w, Jw);
    for (i = 0; i < 3; i++) tau[i] = k*(eta*sg[i] - Jw[i]) + k*(eta*ep[i] - Jw[i]);
    return 1;
}

/* P8 Celani 2026: boresight e3 onto the target a (body), rotation about e3 left free:
 * m = b x (k_p (e3 x a) - k_d w); the rate is taken relative to the reference rate. */
void adcs_mtq_boresight(const adcs_real e3[3], const adcs_real a[3], const adcs_real we[3], adcs_real kp, adcs_real kd, adcs_real tau[3])
{
    adcs_real c[3];
    int i;
    adcs_cross(e3, a, c);
    for (i = 0; i < 3; i++) tau[i] = kp*c[i] - kd*we[i];
}

/* P3 TANGO (Chasset et al. 2013): LQR with P frozen at the averaged-Riccati P_ss; with the
 * command multiplied by J the torque is -(1/r) Gamma(t) (P21 theta + P22 w), Gamma applied by the
 * projection, theta = 2 q_v. Pth = P21/r, Pw = P22/r are computed on the ground. */
void adcs_mtq_tango(const adcs_real q[4], const adcs_real w[3], const adcs_real q_ref[4], const adcs_real w_ref[3],
                    adcs_real Pth[3][3], adcs_real Pw[3][3], adcs_real tau[3])
{
    adcs_real qe[4], s, wr[3], we[3], th[3], a[3], b[3];
    int i;
    mtq_err(q, w, q_ref, w_ref, qe, &s, wr, we);
    for (i = 0; i < 3; i++) th[i] = 2*s*qe[i];
    adcs_mat3_vec(Pth, th, a); adcs_mat3_vec(Pw, we, b);
    for (i = 0; i < 3; i++) tau[i] = -(a[i] + b[i]);
}

/* P2 de Ruiter 2011 on the Sun line (06_detumble_sunspin.md): body z spins at spin_dps with
 * -z on the Sun, one Lyapunov function for spin axis, spin rate and nutation.
 * h_d = -sg J_zz ws s, e_hz = h_z - sg J_zz ws, P = diag(1, 1, 0) (the spin axis is z here),
 * A = B x (h - h_d + k1 e_hz z + k2 P w),  m0 = -(k/|B|^2) A. */
void adcs_sun_spin_deruiter(const adcs_real B[3], const adcs_real w[3], const adcs_real s[3], int eclipse, adcs_real J[3][3],
                            adcs_real spin_dps, adcs_real k, adcs_real k1, adcs_real k2, adcs_real m0[3])
{
    adcs_real ws, sg, h[3], x[3], A[3], Bs, ehz;
    int i;
    if (eclipse) { adcs_zero3(m0); return; }
    ws = fabs(spin_dps*ADCS_D2R);
    sg = adcs_sign(w[2]); if (sg == 0) sg = 1;
    adcs_mat3_vec(J, w, h);
    ehz = h[2] - sg*J[2][2]*ws;
    for (i = 0; i < 3; i++) x[i] = h[i] + sg*J[2][2]*ws*s[i];
    x[2] += k1*ehz;
    x[0] += k2*w[0]; x[1] += k2*w[1];
    adcs_cross(B, x, A);
    Bs = adcs_dot(B, B);
    if (Bs < 1e-18) { adcs_zero3(m0); return; }
    for (i = 0; i < 3; i++) m0[i] = -k*A[i]/Bs;
}
