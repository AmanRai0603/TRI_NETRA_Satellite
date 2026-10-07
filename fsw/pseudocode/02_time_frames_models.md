# 02 — Time, frames and onboard environment models (`adcs_env`, `env`)

## Julian date and GMST

```
jd(t) = jd0 + t / 86400                       # jd0 from the config (epoch of tick 0)
gmst_rot(jd):
    T = (jd − 2451545) / 36525
    g = mod(67310.54841 + (876600·3600 + 8640184.812866) T + 0.093104 T² − 6.2e-6 T³, 86400) / 240 · π/180
    return [[cos g, sin g, 0], [−sin g, cos g, 0], [0, 0, 1]]      # mean of date -> ECEF
prec_rot(jd):                                  # J2000 -> mean of date, IAU-76
    ζ = 2306.2181 T + 0.30188 T² + 0.017998 T³, z = 2306.2181 T + 1.09468 T² + 0.018203 T³,
    θ = 2004.3109 T − 0.42665 T² − 0.041833 T³   (arcsec)
    return R3(−z) R2(θ) R3(−ζ)
eci2ecef(jd) = gmst_rot(jd) prec_rot(jd)       # J2000 -> ECEF (nutation ≤ 0.005°, polar motion omitted)
decyear(jd) = 2000 + (jd − 2451544.5) / 365.25
```

Every inertial vector onboard is J2000: the Sun model, the star-tracker quaternion, the GNSS fix after
conversion, and the field reference (through `eci2ecef`). A GMST-only rotation would put the field and the
GNSS position in the mean equator of date, 0.37° off J2000 in 2027. The truth model uses the same precession
for the field and the GNSS emulator; its gravity field keeps POP's GMST orientation (a 0.37° turn of the
tesseral terms, negligible for the orbit).

## Onboard Sun model (Vallado alg. 29, precessed to J2000)

```
sun_model(jd):
    T = (jd − 2451545)/36525
    L = mod(280.460 + 36000.771 T, 360)
    M = mod(357.5291092 + 35999.05034 T, 360)·π/180
    λ = (L + 1.914666471 sin M + 0.019994643 sin 2M)·π/180
    ε = (23.439291 − 0.0130042 T)·π/180
    s_mod = [cos λ, cos ε sin λ, sin ε sin λ]
    ζ = (2306.2181 T + 0.30188 T²)″, z = (2306.2181 T + 1.09468 T²)″, θ = (2004.3109 T − 0.42665 T²)″
    s = R3(ζ) R2(−θ) R3(z) s_mod           # R3(a) = [[c, s, 0], [−s, c, 0], [0, 0, 1]]
    return unit(s)                          #  R2(a) = [[c, 0, −s], [0, 1, 0], [s, 0, c]]
```

The MATLAB twin found the missing precession as a 0.37° floor in Sun referencing. It matches
DE440 to 0.004° in 2027.

## Geodetic from ECEF (WGS-84, Bowring, 3 iterations)

```
geodetic(r):   a = 6378137, f = 1/298.257223563, e² = f(2 − f)
    lon = atan2(y, x);  p = hypot(x, y);  lat = atan2(z, p (1 − e²))
    repeat 3: N = a / sqrt(1 − e² sin² lat); h = p / cos lat − N; lat = atan2(z, p (1 − e² N/(N + h)))
    return lat, lon, h
```

## Onboard field (IGRF-13, degree `igrf_nmax`, default 10)

```
igrf_gh(decyear):     # linear between the two bracketing 5-year epochs; the last interval extrapolates
igrf_ned(gh, lat, lon, alt_km, nmax) -> B_NED [nT]:
    identical to asils.env.igrf_ned (Standard Code env.igrf, geodetic -> geocentric rotation,
    Schmidt semi-normalised recursion); coefficient order g10 g11 h11 g20 g21 h21 ...
field_eci(r_eci, jd, gh, nmax):
    C = eci2ecef(jd); r_e = C r_eci; (lat, lon, h) = geodetic(r_e)
    B_ned = igrf_ned(gh, lat, lon, h/1000, nmax) · 1e-9
    R_n2e = [[−sl co, −so, −cl co], [−sl so, co, −cl so], [cl, 0, −sl]]
    return Cᵀ R_n2e B_ned                   # T, ECI
```

Coefficient table: `matlab_sils/data/igrf13coeffs.txt` generates `02_igrf13.pc` (tools/gen_fsw_params.py), which the
flight build writes into the flight software with the rest (tools/flight_build.py). The field is recomputed at most once per
second, at the start of a coil cycle; the coefficients `gh` are re-interpolated once a day. IGRF-13's last
interval (2020–2025) is extrapolated for 2027; IGRF-14 is the table to load for flight.

## Onboard orbit

```
orbit_step(r, v, dt):
    if gnss fix this tick:
        L = gps_latency                                                # the fix is the state L seconds ago
        if gnss_ecef: C = eci2ecef(jd − L/86400); r = Cᵀ r_e; v = Cᵀ (v_e + ω_E ẑ × r_e)    # receivers fix in ECEF
        else: r, v = fix
        if L > 0: a0 = acc(r); r += v L + a0 L²/2; v += (a0 + acc(r)) L/2  # carried forward to now
    elif r known (velocity Verlet, two-body + J2):
        a0 = acc(r); r += v dt + a0 dt²/2; v += (a0 + acc(r)) dt/2
acc(r) = −μ r/|r|³ − 1.5 J2 μ R_E²/|r|⁵ [x(1 − 5z²/r²), y(1 − 5z²/r²), z(3 − 5z²/r²)]
         μ = 3.986004418e14, J2 = 1.08262668e-3, R_E = 6378137, ω_E = 7.2921158553e-5
```

Between fixes the J2 term matters: two-body alone drifts 32 km per orbit at 550 km, and the old first-order
velocity update another 23 km. Without any fix the onboard orbit is not valid (`have_r`), so neither the field
reference nor the pointing states run.
