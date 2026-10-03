/* adcs_modes.c -- the mode manager (08_mode_manager.md): which controller state the spacecraft is
 * in, what it may fly, and when it moves on (commanded changes, the detumble exit, the spin guards,
 * the Sun acquisition's end). Group gdn (design/groups.toml); twin: asils.fsw.mode_manager; Rust: modes.rs.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_fsw_int.h"

void adcs_modes_enter(fsw_t *s, uint8_t mode)
{
    if (mode == ADCS_MODE_SPINUP) { s->sz_sum = 0; s->sz_n = 0; s->sz_t0 = s->t; }
    s->mode = mode; s->t_mode = s->t; s->hold = 0; s->ho = 0; s->ho_t = 0;
    /* the states that skip estimation freeze the attitude: the next pointing state re-initialises it */
    if (mode == ADCS_MODE_DETUMBLE || mode == ADCS_MODE_DETUMBLE_RCS || mode == ADCS_MODE_SPINUP || mode == ADCS_MODE_SUN_SPIN) {
        s->ad_ok = 0; s->n_rej = 0;
    }
    adcs_zero3(s->I_q);
}

/* Can the fitted hardware fly this state? Fine pointing needs momentum devices or thrusters, the
 * rotor Sun acquisition needs rotors, the thruster detumble thrusters; the coil states need only
 * the coils every configuration has. A state it cannot fly is refused at init and by telecommand. */
int adcs_modes_feasible(const adcs_params_t *p, unsigned m)
{
    switch (m) {
    case ADCS_MODE_NADIR_FINE: case ADCS_MODE_TARGET_FINE: case ADCS_MODE_SLEW_FINE: case ADCS_MODE_SUN_FINE:
        return p->nr > 0 || p->nc > 0;
    case ADCS_MODE_SUN_ACQ_ROTOR: return p->nr > 0;
    case ADCS_MODE_DETUMBLE_RCS: return p->nc > 0;
    default: return m < ADCS_MODE_COUNT;
    }
}

static void spin_guards(fsw_t *s, double dt)
{
    const adcs_params_t *p = &s->p;
    double d = ADCS_D2R, wz = s->w_est[2], wp = sqrt(s->w_est[0]*s->w_est[0] + s->w_est[1]*s->w_est[1]);
    if (adcs_norm3(s->w_est) > p->ss_omega_max_dps*d) { adcs_modes_enter(s, ADCS_MODE_DETUMBLE); return; }
    if (s->mode == ADCS_MODE_SPINUP) {
        int conv = fabs(wz - s->sigma*p->ss_spin_dps*d) < p->ss_z_in_dps*d && wp < p->ss_perp_in_dps*d, ok;
        if (conv && s->z.sun_ok) { s->sz_sum += s->z.sun[2]; s->sz_n += 1; }
        if (s->t - s->sz_t0 >= p->ss_t_check_s && s->sz_n > 0 && s->sz_sum/s->sz_n > p->ss_sun_min) {
            s->sigma = -s->sigma; s->sz_sum = 0; s->sz_n = 0; s->sz_t0 = s->t;
        }
        ok = conv && s->z.sun_ok && s->z.sun[2] < 0;
        if (ok) s->hold += dt; else s->hold = 0;
        if (s->hold >= p->ss_dwell_in_s) adcs_modes_enter(s, ADCS_MODE_SUN_SPIN);
    } else {
        int bad = fabs(wz) < p->ss_omega_exit_dps*d || wp > p->ss_perp_out_dps*d;
        if (bad) s->hold += dt; else s->hold = 0;
        if (s->hold >= p->ss_dwell_out_s) adcs_modes_enter(s, ADCS_MODE_SPINUP);
    }
}

/* commanded changes: the schedule's entries whose time has come */
void adcs_modes_schedule(fsw_t *s)
{
    const adcs_params_t *p = &s->p;
    while (s->sched_i < p->n_sched && s->t >= p->sched_t[s->sched_i]) { adcs_modes_enter(s, p->sched_mode[s->sched_i]); s->sched_i++; }
}

/* the transitions of one tick (after safe mode, adcs_fdir_safe) */
void adcs_modes_step(fsw_t *s, double dt)
{
    const adcs_params_t *p = &s->p;
    if ((s->mode == ADCS_MODE_DETUMBLE || s->mode == ADCS_MODE_DETUMBLE_RCS) && p->auto_next != ADCS_MODE_NONE
        && !(s->faults & (FAULT_MAG_STALE | FAULT_GYRO_STALE))) {
        if (adcs_norm3(s->z.w) < p->detumble_exit) s->hold += dt; else s->hold = 0;
        if (s->hold >= p->detumble_hold_s) adcs_modes_enter(s, p->auto_next);
    }
    if ((s->mode == ADCS_MODE_SPINUP || s->mode == ADCS_MODE_SUN_SPIN) && p->ss_law != 2) spin_guards(s, dt);
    if (s->mode == ADCS_MODE_SPINUP || s->mode == ADCS_MODE_SUN_SPIN || s->mode == ADCS_MODE_SUN_ACQ_ROTOR) {
        if (s->z.sun_ok) { adcs_copy3(s->z.sun, s->s_prop); s->s_prop_ok = 1; }
        else if (s->s_prop_ok) {
            adcs_real th[3], dq[4], A[3][3];
            adcs_scale3(s->w_est, dt, th); adcs_fromrotvec(th, dq); adcs_dcm(dq, A);
            adcs_mat3_vec(A, s->s_prop, s->s_prop); adcs_unit(s->s_prop, s->s_prop);
        }
    }
    if (s->mode == ADCS_MODE_SUN_ACQ_ROTOR && p->auto_next != ADCS_MODE_NONE) {
        int ok = s->z.sun_ok && acos(adcs_clamp(adcs_dot(s->z.sun, p->sun_axis), -1, 1)) < p->sa_done_deg*ADCS_D2R && s->ad_ok;
        if (ok) s->acq_hold += dt; else s->acq_hold = 0;
        if (s->acq_hold >= p->sa_done_hold_s) adcs_modes_enter(s, p->auto_next);
    }
}
