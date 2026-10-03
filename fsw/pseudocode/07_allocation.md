# 07 — Allocation, momentum management, thrusters, FDIR (`adcs_alloc`, `alloc`)

Rotors: `nr ≤ 8` momentum devices. Each has a nominal axis `a0_i`, and a gimbal index `gi_i`
(0 for a fixed rotor). Gimbal `j` has axis `g_j`. For a gimballed rotor, `t0_i = cross(g_j, a0_i)`
and its axis at gimbal angle δ is `a_i = cos δ a0_i + sin δ t0_i`. The measured rotor momentum is
`h_i`; the device momentum in body axes is `H_dev = Σ a_i h_i`.

## Fixed rotors (wheels, fluid loops), slot `allocation = rotor_pinv`

```
F = { i : gi_i = 0 and not failed_i }
cmd_r[F] = −pinv_rows(A[:, F]) τ_rot             # commanded ḣ; the body gets −A ḣ
```

## IDMAS split (`idmas_split`)

The coils take the part of τ across the field. At the coil update:
`m += torque2dipole(τ_req, B)`. The rotors get `τ_req − cross(m + m_res_est, B)` (the coil
torque is fed forward).

## Singularity-robust CMG / VSCMG steering (Wie 2008), `cmg_sr` / `vscmg_sr`

```
J_g[:, j] = −h_i cross(g_j, a_i)       for each gimballed rotor i on gimbal j
m_s = det(J_g J_gᵀ)/max(h0, 1e-12)⁶    # singularity measure
VSCMG: J = [J_g, −A], W = diag(I_ng, w_w I_nr), w_w = 0.01 + 2 exp(−10 m_s);   CMG: J = J_g, W = I
λ = λ0 exp(−μ m_s)
u = W Jᵀ (J W Jᵀ + λ I)⁻¹ τ
δ̇ = u[0:ng];  s = max(1, max|δ̇|/δ̇max);  δ̇ /= s
VSCMG wheels: ḣ = u[ng:]/s − k_null (h − h0)
```

## Momentum dumping with the coils (`dump`, sign as corrected in IDMAS v2 §12.3)

```
ΔH = H_dev − H_t;  m = k_dump cross(ΔH, B)/|B|²;  sat(m)
```

## Thrusters (N2O couples, `rcs_pwm`)

```
rcs_duty(req, T):                # one couple per axis sign; torque of couple k = τ_k
    for each axis a with req_a ≠ 0:
        k = couple of the sign of req_a
        on = min(1, |req_a|/|τ_k[a]|) T
        if on < MIB: skip (not fired, not fed forward)
        on = round(on/res) res;  duty_k = on/T;  τ_fed += τ_k duty_k
assist:   req_a = sign(τ_a) max(0, |τ_a| − f cap_a)        # beyond the rotors' torque
          axis near saturation (|H_dev,a| > f hcap_a) and τ would push it further: req_a = τ_a
dumping:  hysteresis on |ΔH| (dump_hi / dump_lo, scaled to the capacity): req −= k ΔH
detumble_rcs: 1 s PWM cycle: at the cycle start τ = −I ω/T_damp (off below the deadband),
          on-times from rcs_duty(τ, 1 s), then played out tick by tick (duty = min(1, left/dt))
```

## FDIR: a fixed rotor that does not follow its command is isolated

```
meas = (h − h_prev)/dt;  expect = clamp(cmd_prev, ±0.8 τmax)
bad_i = |meas − expect| > 0.5 τmax and fixed and |expect| > 0.2 τmax and |h_i| < 0.9 h_max
count_i = (count_i + dt) · bad_i;  count_i > fdir_s -> failed_i = true (event logged)

windowed, fluid loops only (fine pointing commands are too small for the test above; a fluid loop's
driver closes a momentum loop so a healthy one tracks the commanded change, while a wheel's
uncompensated friction drifts it off over a window: a friction-aware wheel test is owed):
  on entry (or after a gap > 1.5 dt): E_i = 0, h0_i = h_i, t0 = t
  each tick:  E_i += clamp(cmd_prev_i, ±0.8 τmax_i) · dt
  when t − t0 ≥ fdir_win_s, for each fluid loop (kind 1), fixed, not failed, with |E_i| > fdir_h_frac · h_max_i
  and |h_i|, |h0_i| < 0.9 h_max_i (judged):
      bad_i = |(h_i − h0_i) − E_i| > 0.5 |E_i|;  nbad_i = bad_i ? nbad_i + 1 : 0
      nbad_i ≥ FDIR_WIN_BAD (2) -> failed_i = true (event logged)
  then E = 0, h0 = h, t0 = t (a window not judged leaves nbad_i as it is)
lost axis flown with the coils: un = τ_req − A_F pinv_rows(A_F) τ_req;  m += torque2dipole(un, B)
```
