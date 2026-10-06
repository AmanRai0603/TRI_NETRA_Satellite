/* adcs_modes.c -- the mode manager (08_mode_manager.md): which controller state the spacecraft is
 * in, what it may fly, and when it moves on (commanded changes, the detumble exit, the spin guards,
 * the Sun acquisition's end). Group gdn (design/groups.toml); twin: asils.fsw.mode_manager; Rust: modes.rs.
 * Every function delegates to its translation of 08_mode_manager.pc (fsw/alg/src/modes.c) on the
 * records Modes and ModeParams, copied from and back to the flight software's state (the spin guards,
 * modes::spin_guards, are reached through modes_step).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include "adcs_fsw_int.h"
#include "adcs_alg_glue.h"

/* the states that skip estimation freeze the attitude: the next pointing state re-initialises it */
/* written from the design: modes::modes_enter (fsw/alg) */
void adcs_modes_enter(fsw_t *s, uint8_t mode) { gl_put_modes(modes_modes_enter(gl_modes(s), mode), s); }

/* Can the fitted hardware fly this state? Fine pointing needs momentum devices or thrusters, the
 * rotor Sun acquisition needs rotors, the thruster detumble thrusters; the coil states need only
 * the coils every configuration has. A state it cannot fly is refused at init and by telecommand. */
/* written from the design: modes::modes_feasible (fsw/alg) */
int adcs_modes_feasible(const adcs_params_t *p, unsigned m) { return modes_modes_feasible(p->nr, p->nc, m) ? 1 : 0; }

/* commanded changes: the schedule's entries whose time has come */
/* written from the design: modes::modes_schedule (fsw/alg) */
void adcs_modes_schedule(fsw_t *s) { gl_put_modes(modes_modes_schedule(gl_modes(s), gl_mode_params(&s->p)), s); }

/* the transitions of one tick (after safe mode, adcs_fdir_safe) */
/* written from the design: modes::modes_step (fsw/alg) */
void adcs_modes_step(fsw_t *s, double dt)
{
    gl_put_modes(modes_modes_step(gl_modes(s), gl_mode_params(&s->p), gl_v3(s->z.w), s->z.sun_ok != 0, gl_v3(s->z.sun), dt), s);
}
