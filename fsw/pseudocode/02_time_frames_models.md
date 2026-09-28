# 02 — Time, frames and onboard environment models (`adcs_env`, `env`)

## Julian date and GMST

```
jd(t) = jd0 + t / 86400                       # jd0 from the config (epoch of tick 0)
gmst_rot(jd):
    T = (jd − 2451545) / 36525
    g = mod(67310.54841 + (876600·3600 + 8640184.812866) T + 0.093104 T² − 6.2e-6 T³, 86400) / 240 · π/180
    return [[cos g, sin g, 0], [−sin g, cos g, 0], [0, 0, 1]]      # ECI -> ECEF
decyear(jd) = 2000 + (jd − 2451544.5) / 365.25
```

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
    C = gmst_rot(jd); r_e = C r_eci; (lat, lon, h) = geodetic(r_e)
    B_ned = igrf_ned(gh, lat, lon, h/1000, nmax) · 1e-9
    R_n2e = [[−sl co, −so, −cl co], [−sl so, co, −cl so], [cl, 0, −sl]]
    return Cᵀ R_n2e B_ned                   # T, ECI
```

Coefficient table: `matlab_sils/data/igrf13.json` generates `fsw/include/adcs_igrf13.h` and
`fsw-rs/src/igrf13.rs` (tools/gen_fsw_tables.py). The field is recomputed at most once per
second, at the start of a coil cycle.

## Onboard orbit

```
orbit_step(r, v, dt):
    if gnss fix this tick: r, v = fix
    elif r known: a = −μ r/|r|³;  r += v dt + a dt²/2;  v += a dt        # μ = 3.986004418e14
```
