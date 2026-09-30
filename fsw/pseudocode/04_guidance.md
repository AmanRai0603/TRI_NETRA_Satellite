# 04 — Guidance (`adcs_guid`, `guid`)

`guidance(kind, r, v, t, gd) -> (q_ref, ω_ref, ω̇_ref)`, with `ω_ref` expressed in the
reference frame.

```
common:
    rh = unit(r); vh = unit(v); n = unit(cross(vh, −rh)); ram = unit(cross(−rh, n))
    R_nad = rows[−ram, −rh, −n]                 # X = −ram, Y = nadir, Z = −orbit normal (Standard Code guid.nadir)
    q_nad = fromdcm(R_nad)
    if gd.q_off set: q_nad = qmult(q_nad, gd.q_off)   # payload boresight -> nadir
    if gd.flip: q_nad = qmult(q_nad, [u, 0])           # 180° about the boresight u = unit(gd.roll_axis)
    ω_orb = cross(r, v)/|r|²
    ax = unit(gd.axis) or [1,0,0]

kind nadir:    q_ref = q_nad;  ω_ref = dcm(q_nad) ω_orb;  ω̇_ref = 0
kind target:   q_ref = qmult(q_nad, fromrotvec(ax · roll));  ω_ref = dcm(q_ref) ω_orb
kind slew:     τ = clamp((t − t0)/T, 0, 1)
               s = τ − sin(2πτ)/(2π); ṡ = (1 − cos 2πτ)/T, s̈ = 2π sin(2πτ)/T² inside (t0, t0+T), else 0
               q_ref = qmult(q_nad, fromrotvec(ax · φ s));  ω_o = dcm(q_ref) ω_orb
               ω_ref = ω_o + ax φ ṡ;  ω̇_ref = ax φ s̈ − (ax φ ṡ) × ω_o        # transport term of the turning frame
kind inertial: q_ref = gd.q_inertial; ω_ref = 0
kind sun:      a = unit(gd.sun_axis) (default −Z);  b = gd.roll_axis made normal to a (default +X)
               s = unit(gd.sun_eci);  e2 = unit(n − (n·s) s)  (Sun on the orbit normal: e2 = unit(ẑ − s_z s))
               B = [a, b, a×b];  E = [s, e2, s×e2];  q_ref = fromdcm(B Eᵀ);  ω_ref = 0
```

## Yaw flip (nadir, target, slew; `gd_yaw_flip`)

```
yaw_flip(gd, r, v, h):                      # every tick with an orbit, before guidance
    q = guidance(nadir, r, v) with flip = 0;  d = gd.sun_axis · dcm(q) unit(gd.sun_eci)
    if d < −h: flip = 1;  elif d > h: flip = 0     # h = 0.1
```

The base nadir frame puts body −Z on the orbit normal whatever the sign of β. With a −Z power face that faced
away from the Sun for every nadir orbit on both cases (β = −59° and −24°): 147° and 112° from the Sun. The
flip turns the body 180° about the payload axis, which keeps the payload on nadir and gives the geometric
best, 90° − |β| (31° and 66°). In a Sun-synchronous orbit β keeps its sign, so the flip is set once; the
hysteresis stops chatter should β cross zero. The metrics apply the same flip to the truth reference.

## Boresight offset (config time)

```
boresight_offset(bs):      # rotation taking +Y_B (the nadir axis of the base frame) to the payload axis
    ey = [0,1,0]; bs = unit(bs); ax = cross(ey, bs); c = ey·bs; s = |ax|
    s < 1e-12:  return c > 0 ? [0,0,0,1] : [1,0,0,0]
    K = skew(ax/s);  A = I + s K + (1 − c) K²;  return fromdcm(A)
```
