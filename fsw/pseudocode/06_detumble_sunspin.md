# 06 — Detumble and coils-only Sun acquisition (`adcs_mag`, `mag`)

## Coil duty cycle (Standard Code ctrl.bdotScheduler)

A coil cycle lasts `mtq_period` (1 s). The coils are off for the first `mtq_meas` (0.2 s): the
magnetometer samples of that window are averaged as a clean field. At `phase = mtq_meas` the law
runs once, and the dipole is held for the rest of the cycle. `first` is the first tick of a cycle.

## B-dot family (slot `detumble`)

```
b = unit(mean(B_i/|B_i|)) over the window;  B_av = mean(B_i)
bdot_gyro:     ḃ = −cross(ω_meas, b);  m = −(k/|B|) (b − (b − ḃ))           # = −(k/|B|) ḃ
bdot_mag:      m = −(k/|B|) (b − b_prev)/T_c                                  # once b_prev exists
bdot_bangbang: ḃ = (b − b_prev)/T_c;  ḃ_bl = m_max |B| / (4 k);  m_i = −m_max sat(ḃ_i / ḃ_bl)
               # boundary layer: full dipole outside, 4 x the B-dot gain inside. Pure sign switching
               # (the Standard Code law) limit-cycles around the 0.5 deg/s exit rate: 4 of 12 seeds
               # never settle, with or without OBC latency (docs/SOFT_OILS.md).
genbdot_l1:    m = −k_l1 ((B_av − B_prev)/T_c + cross(0, B_av))            # ω_d = 0
k = gain_scale · 2 n (1 + sin i) J_min        (Avanzini & Giulietti 2012), computed at config
m *= min(1, min_i(m_max/|m_i|))   # direction-preserving (act.saturateDipole)
if m ≠ 0: m = sat(m − m_res_est)  # residual-dipole compensation
```

## Spin-up (L1, ctrl.spinupTick) and Sun spin (L2, ctrl.sunSpin)

```
spinup:   ḃ = (B_av − B_prev)/T_c (0 on the first cycle)
          ω_d = σ ω_s ẑ;  m0 = −k_l1 (ḃ + cross(ω_d, B_av))
sun_spin: E1 in eclipse (Sun invalid): m0 = 0
          s = s_prop (measured, or gyro-propagated for E2)
          ω* = −|ω_s|;  σ = sign(ω_z) (1 if 0)
          h = J ω;  h_d = σ J_zz ω* s;  h̃ = h − h_d
          f = rz_floor · J_zz;  R_z = diag(max(J_zz − J_xx, f), max(J_zz − J_yy, f), 0)
          A = cross(B, k1 h̃ + k2 R_z ω);  m0 = −A/|B|²  (0 when |B|² < 1e-18)
m = sat(m0 − m_res_est)
```

The chain spin-up → Sun spin above is P11 (UPMSat-2, flown: m = −k(Ḃ + ω_d × B)) → P5 (He et al. 2023),
with the rate band of P9 (Roldugin et al.) as the switch (`ss_law = 0`). Two more Sun laws from the
literature review share the modes (docs/MTQ_LITERATURE.md):

```
ss_law = 1, P2 de Ruiter 2011 on the Sun line (sun_spin state):
          ws = |ω_s|;  σ = sign(ω_z) (1 if 0);  h = J ω
          e_hz = h_z − σ J_zz ws
          x = h + σ J_zz ws s;  x_z += k1 e_hz;  x_x += k2 ω_x;  x_y += k2 ω_y   # P = diag(1, 1, 0)
          A = cross(B, x);  m0 = −k A/|B|²                                        # k1 > 1, k2 > 0
ss_law = 2, P8 Celani 2026 boresight on the Sun (both spin states, no spin guards):
          if s_prop known and |B|² ≥ 1e-18: τ = k_p (sun_axis × unit(s_prop)) − k_d ω;  m0 = cross(B, τ)/|B|²
          else m0 = 0
```

## Spin guards (modes.transitions rows, applied each tick)

```
|ω| > ω_max                      -> detumble        (dwell 0)
spinup:
    conv = |ω_z − σ ω_s| < z_in and |ω_xy| < perp_in
    G_σ: if conv and Sun valid: sum += s_z, n += 1
         if t − t0 ≥ t_check and n > 0 and sum/n > s_min: σ = −σ; sum = n = 0; t0 = t
    conv and Sun valid and s_z < 0, held dwell_in      -> sun_spin
sun_spin:
    |ω_z| < ω_exit or |ω_xy| > perp_out, held dwell_out -> spinup
```

## Sun propagation in eclipse (ctrl.propagateSun, E2)

```
Sun valid: s_prop = s_meas;  else if s_prop known: s_prop = unit(dcm(fromrotvec(ω dt)) s_prop)
```
