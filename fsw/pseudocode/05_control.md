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

### Magnetorquer-only literature laws (docs/MTQ_LITERATURE.md)

Common: `q_e = qconj(q_ref) ⊗ q`, `s = sign(q_e.w)` (1 if 0), `ω_r = dcm(q_e) ω_ref`, `ω_e = ω − ω_r`.
Every law returns a torque; `torque2dipole` applies the projection Γ(t) = I − b bᵀ (m = B × τ/|B|²),
which is the Lovera–Astolfi input reformulation. Gains are torque-level (normalised Γ), computed on the ground.

```
mtq_lovera2004 (P1, Prop. 1):  τ = −(ε² k_p s q_e.v + ε k_v J ω_e)
mtq_celani2015 (P4, Thm 2):    τ = −(ε² k_1 s q_e.v + ε k_2 ω_e)            # no inertia in the law
mtq_avanzini2021 (P16):        n = |ω_ref|;  if n < 1e-9: use mtq_celani2015
                               e_p = ω_ref/n;  σ = dcm(q_e) e_p;  J_p = e_pᵀ J e_p
                               θ = 2 s (q_e.v · e_p)                          # pitch error, first order
                               η = J_p n (1 − λ θ)
                               τ = k (η σ − J ω) + k (η e_p − J ω)            # k_ζ = k_ε = k
mtq_celani2026 (P8):           e3 = sun_axis (Sun state) or payload boresight (nadir state)
                               a = unit(s_prop) in the Sun state with a Sun, else dcm(q_e) e3
                               τ = k_p (e3 × a) − k_d ω_e                    # rotation about e3 free
mtq_tango2013 (P3):            θ = 2 s q_e.v;  τ = −(P_θ θ + P_ω ω_e)        # P_θ = P21/r, P_ω = P22/r
```

Ground gains (engine config, fsw.mtq_gain_p / mtq_gain_d scale them, node `tune`):
ε = 10⁻³, k_1 = g_p J̄ ω_n²/ε², k_2 = g_d 2ζ J̄ ω_n/ε (Lovera: g_d 2ζ ω_n/ε, the J is in the law);
k = g_d 0.84 n, λ = g_p 0.08 (Avanzini's case); k_p = g_p J̄ ω_n², k_d = g_d 2ζ J̄ ω_n (Celani 2026);
TANGO: per-axis double-integrator CARE with the orbit-averaged B_u R⁻¹ B_uᵀ, isotropic field average
E[Γ J⁻² Γ]_ii = (7/15) J_i⁻² + tr(J⁻²)/15, Q chosen so the average axis has ω_n, ζ (the paper's Q, R are
unpublished); P12 = √(q_θ/m_i), P22 = √((q_ω + 2 P12)/m_i).

## Magnetic pointing states (`NADIR_MTQ`, `SUN_MTQ`): hand-over and gravity-gradient feed-forward

Once per coil cycle, at the end of the measure window, with an estimate and an orbit:

```
guidance -> q_ref, ω_ref
q_e = qconj(q_ref) ⊗ q;  ω_r = dcm(q_e) ω_ref;  ω_e = ω − ω_r
# hand-over: a body arriving from the Sun spin (4 °/s) is despun before any pointing law runs
if not ho and |ω_e| > ho_in:  ho = 1; ho_t = 0
if ho: ho_t = ho_t + T_coil if |ω_e| < ho_out else 0;  if ho_t ≥ ho_hold: ho = 0
if ho:
    m = (k_bdot/|B|) (ω_e × unit(B))           # torque −k_bdot ω_e⊥B: the detumble gain on the rate error
    τ_req = 0
else:
    τ_req = the selected magnetic law (above)
    if mtq_gg_ff bit (0: SUN_MTQ, 1: NADIR_MTQ):
        r_b = dcm(q) r;  τ_req −= 3 μ/|r_b|⁵ (r_b × J r_b)    # cancel the modelled gravity gradient
    m = torque2dipole(τ_req, B)
m = sat(m − m_res_est)
```

Why the despin uses the detumble gain k_bdot = 3·2n(1 + sin i)J_min (Avanzini & Giulietti 2012) and not the
spin-up gain: the coils only act across B. With a very high gain the rate across B is removed at once and the
body keeps a rate along the field line, which the turning field only rotates, so the rate hardly decays
(3.9 → 0.3 °/s in one orbit). The optimal B-dot gain lets that component decay (3.9 → 0.5 °/s in 0.4
orbit). The pointing law then takes over at ho_out = 0.5 °/s; at 0.2 °/s it hands over late or not at all, because the
field's own rotation keeps a rate of that order.

Why the feed-forward is on only in the Sun state: at nadir the long (minimum-inertia) axis sits at the
gravity-gradient equilibrium, so the gradient is a restoring stiffness that helps the loop. Cancelling it would
remove that help. Under Sun referencing the attitude is inertial, so the gradient is a periodic forcing at twice
the orbit rate, of the order of 3n²ΔJ, and the coils cannot reject its component along B.

## Torque to dipole (`torque2dipole`, Standard Code act.torque2dipole)

```
if |B|² < 1e-18: m = 0
m = cross(B, τ)/|B|²
m *= min(1, min_i(m_max / max(|m_i|, 1e-30)))        # direction-preserving saturation
```
