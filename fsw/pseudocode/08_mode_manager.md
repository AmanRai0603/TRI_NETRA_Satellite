# 08 — Tick order and mode manager (`adcs_fsw`, `fsw`)

## Controller states and the mission mode each serves

| state | mission mode | guidance | actuators |
|---|---|---|---|
| `DETUMBLE` | detumble | — | coils (B-dot family) |
| `DETUMBLE_RCS` | detumble | — | thrusters |
| `SPINUP`, `SUN_SPIN` | sun acquisition | — | coils (L1 / L2) |
| `SUN_ACQ_ROTOR` | sun acquisition | Sun vector | momentum devices |
| `SUN_MTQ` / `SUN_FINE` | sun referencing | `sun` | coils / momentum devices |
| `NADIR_MTQ` / `NADIR_FINE` | nadir pointing | `nadir` | coils / momentum devices |
| `TARGET_FINE`, `SLEW_FINE` | target pointing, slew | `target`, `slew` | momentum devices |

## One tick

```
step(now):
    t = (now − start)/1e9;  jd = jd0 + t/86400
    read sensors through the drivers (09_drivers.md)
    1 onboard orbit (02)
    2 estimation (03): field reference at the coil-cycle start, MEKF init / predict / updates
       (skipped in DETUMBLE, DETUMBLE_RCS, SPINUP, SUN_SPIN), rate filter ω_f
    3 mode manager:
         commanded schedule / telecommands: enter(mode)
         DETUMBLE*, auto_next set: |ω_meas| < detumble_exit held detumble_hold_s -> enter(auto_next)
         SPINUP / SUN_SPIN: spin guards (06)
         SPINUP / SUN_SPIN / SUN_ACQ_ROTOR: propagate the Sun (06)
         SUN_ACQ_ROTOR, auto_next set: Sun valid, angle(sun, a) < 10° and estimate ready, held 60 s -> enter
    4 control for the state (05, 06, 07); coil duty cycle; thrusters; allocation
    write actuator commands through the drivers; hold the dipole between cycles
enter(mode): mode = mode; t_mode = t; hold = 0; I_q = 0; (SPINUP: reset the G_σ window); log the event
```

## Fine states in one pass (`SUN_FINE`, `NADIR_FINE`, `TARGET_FINE`, `SLEW_FINE`, `SUN_ACQ_ROTOR`)

```
A = axes(δ); H_dev = A h
FDIR (07)
at the control rate (1/rw_rate_hz):
    SUN_ACQ_ROTOR: τ_req = sun_acq law (05)
    else if estimate and orbit: guidance; capture law or control_law (05)
ΔH = H_dev − H_t; RCS dump hysteresis
coil cycle: first tick: coils off
            phase = meas: m = dump (unless RCS dumps) + IDMAS share + lost-axis share; m = sat(m − m_res_est)
thrusters: assist + dumping -> rcs_duty(req, dt)
rotors: allocate(τ_req − cross(m + m_res_est, B_at_cycle) − τ_rcs)
```

## Rotor idle command (every non-fine state)

`cmd_r = −0.2 (h − h_bias)`, and a CMG rotor's torque command is 0: the rotor's own speed loop
holds it.
