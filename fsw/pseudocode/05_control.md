# 05 — Control laws (`adcs_ctl`, `ctl`)

Every law returns a body torque request `τ` [N m]. Gains arrive in the parameter table
(computed on the ground: LQR gains by the Hamiltonian ARE, `matlab_sils/+asils/+fsw/lqr_gain.m`).

## Three-axis law for momentum devices (`control_law`, slot `pointing`)

```
q_e = qmult(qconj(q_ref), q); if q_e.w < 0: q_e = −q_e
e   = clamp(q_e.v, −err_max, err_max)                     # per component
ω_e = ω − dcm(q_e) ω_ref
H   = I ω + H_dev;  gyro = cross(ω, H);  ff = I ω̇_ref
law pid:  I_q = clamp(I_q + e dt, ±int_max);  τ = −Kp∘e − Kd∘ω_e − Ki∘I_q
law lqr:  I_q = clamp(I_q + 2 e dt, ±int_max); τ = −(K1∘I_q + K2∘(2e) + K3∘ω_e)
law smc:  s = ω_e + λ e;  sat = clamp(s/φ, −1, 1)
          ė = ½ (q_e.w ω_e + cross(e, ω_e));  τ = −I (λ ė + Gs∘sat)
τ += gyro + ff
```

## Capture manoeuvre (fine modes, error beyond `capture_deg`)

```
if mode = slew_fine or capture_deg ≤ 0: off
θ = 2 acos(min(1, q_e.w)); if θ < capture_deg: off
e = q_e.v/|q_e.v|;  Jm = max diag(I)
α = ½ min(cap)/Jm;  ω_max = min(capture_rate, ½ min(hcap)/Jm)
ω_c = dcm(q_e) ω_ref − e min(ω_max, sqrt(2 α θ))
k_r = min(0.5, 4 α/max(ω_max, 1e-6))
τ = I k_r (ω_c − ω) + cross(ω, I ω + H_dev);   I_q = 0
```

`cap` and `hcap` are per axis: the sum over fixed rotors of `|axis|·torque_max` and
`|axis|·h_max`. Gimballed rotors add `2 h0 δ̇max` and `2.5 h0`.

## Sun-vector acquisition with momentum devices (`sun_acq_rotor`)

```
s = unit(s_prop)       # measured Sun, or propagated on the gyro in eclipse
if no Sun ever seen: ω_c = 0
else: c = cross(a, s)  # a = power face
      if s·a < −0.95: c = unit(cross(a, x̂)) (or ŷ when parallel)
      ω_c = (ω_max/0.5) c, limited to |ω_c| ≤ ω_max
τ = I kd (ω_c − ω) + cross(ω, I ω + H_dev)           # d(s·a)/dt = k (1 − (s·a)²) ≥ 0
```

## Magnetic three-axis laws (slot `mtq_pointing`)

```
mtq_pd:        τ = −Kp∘(sign(q_e.w) q_e.v) − Kd∘(ω − dcm(q_e) ω_ref)
mtq_lqr/smc:   control_law with the magnetic gains, dt = coil period, H_dev = 0
mtq_rate_damp: τ = −Kd∘(ω − dcm(q_e) ω_ref)
```

## Torque to dipole (`torque2dipole`, Standard Code act.torque2dipole)

```
if |B|² < 1e-18: m = 0
m = cross(B, τ)/|B|²
m *= min(1, min_i(m_max / max(|m_i|, 1e-30)))        # direction-preserving saturation
```
