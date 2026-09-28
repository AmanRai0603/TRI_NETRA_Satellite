# 03 — Attitude estimation (`adcs_est`, `est`)

## MEKF (Markley & Crassidis), state q, gyro bias b, covariance P (6×6)

```
mekf_init(q0, σ_att0, σ_bias0):  q = q0; b = 0; P = diag(σ_att0² I3, σ_bias0² I3)

mekf_predict(ω_meas, dt):
    ω = ω_meas − b
    q = qnorm(qmult(q, fromrotvec(ω dt)))
    W = skew(ω)
    Φ = [[I − W dt + ½ W² dt², −I dt], [0, I]]
    Q = [[(σv² dt + σu² dt³/3) I, −(σu² dt²/2) I], [−(σu² dt²/2) I, σu² dt I]]   # σv = ARW, σu = RRW
    P = Φ P Φᵀ + Q

mekf_vector(b_meas, r_ref, σ):          # Sun, field, nadir
    b = unit(b_meas); r = unit(r_ref); b̂ = dcm(q) r
    H = [skew(b̂), 0]; R = σ² I3
    S = H P Hᵀ + R; G = P Hᵀ S⁻¹; δx = G (b − b̂)
    apply(δx, G, H, R)

mekf_quat(q_meas, σ_cross, σ_roll, boresight bs):   # star tracker head
    δq = qmult(qconj(q), q_meas); if δq.w < 0: δq = −δq
    y = 2 δq.v; H = [I, 0]
    R = σ_cross² I + (σ_roll² − σ_cross²) bs bsᵀ
    S = H P Hᵀ + R; G = P Hᵀ S⁻¹; δx = G y
    apply(δx, G, H, R)

apply(δx, G, H, R):
    q = qnorm(qmult(q, [½ δx[0:3], 1])); b += δx[3:6]
    P = (I − G H) P (I − G H)ᵀ + G R Gᵀ          # Joseph form
```

## TRIAD (initialisation from Sun + field)

```
triad(b1, b2, r1, r2):
    t1b = unit(b1); t2b = unit(cross(b1, b2)); t3b = cross(t1b, t2b)
    t1r = unit(r1); t2r = unit(cross(r1, r2)); t3r = cross(t1r, t2r)
    A = [t1b t2b t3b] [t1r t2r t3r]ᵀ;  return fromdcm(A)
```

## q-method / QUEST (star tracker head, component chain)

```
quest(b[3×n], r[3×n], w[n]):
    B = Σ w_i b_i r_iᵀ; S = B + Bᵀ; σ = trace B; z = [B23 − B32, B31 − B13, B12 − B21]
    K = [[S − σ I, z], [zᵀ, σ]]
    (λ, V) = jacobi_eig4(K); q = qnorm(V[:, argmax λ])    # scalar last by construction of K
```

## Star-tracker latency compensation and rate filter

```
latency(q_st, ω, lat) = qnorm(qmult(q_st, fromrotvec(ω lat)))
rate_lpf:  a = dt/(τ + dt);  ω_f += a (ω_raw − ω_f)          # τ = rate_lpf_s; ω_raw = ω_meas − b
```

## Which updates run (per tick, after predict)

```
if star tracker fitted and a head is valid: mekf_quat for every valid head (latency-compensated); t_st = t
elif star tracker fitted and t − t_st < st_coast_s: nothing (gyro coast)
else, at the start of a coil cycle only (clean field):
    Sun valid      -> mekf_vector(sun, sun_model(jd), sig_sun)
    field clean    -> mekf_vector(B, field_eci, sig_mag)
    Earth sensor   -> mekf_vector(nadir, −r/|r|, 2 σ_es)
Initialisation (while no estimate): the first valid star-tracker head (σ 1e-3, 2e-4),
else TRIAD(Sun, field) (σ 0.05, 2e-4).
```
