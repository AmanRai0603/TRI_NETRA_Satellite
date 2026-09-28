# From SILS to OILS and HILS — the hardware-abstraction boundary

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

The spec's rule (`spec/fsw/include/adcs_hal.h`) is *one header, three
implementations*: the flight software only ever talks to the HAL, and what
answers the HAL is the only thing that changes between the rungs. The MATLAB
SILS carries the same boundary as `+asils/+hal`, so everything the flight
software reads or writes already crosses it as **integer register values**
(quantised exactly as the drivers will see them) — in every SILS run.

```
            ┌──────────── SILS (this repo, today) ────────────┐
 plant ─► devices ─► hal.sensors ─► FSW ─► hal.actuators ─► devices ─► plant
                         │  register frames (LSBs below)  │
            └─ backend 'sils' (memory) / 'loopback' (bytes) ─┘

 OILS:  plant ─► devices ─► hal.sensors ═══UDP═══► flight OBC (C FSW)
                                                    │
        plant ◄─ devices ◄─ hal.actuators ◄══UDP════┘          backend 'udp', realtime = 1

 HILS:  plant ─► hal.stimulus ─► Helmholtz cage (B_body), Sun simulator (sun_body),
                                 air bearing (w_body reference), lamp (sunlit)
        real sensors ─► OBC ─► real actuators;   the SILS keeps the orbit + environment
```

## 1. Backends (`P.hal.backend`)

| backend | what crosses | used for |
|---|---|---|
| `sils` (default) | quantised values in memory | every SILS run and campaign |
| `loopback` | every frame is packed to bytes and unpacked again each tick | proving the byte layout the OILS/HILS link carries (`tests/run_all_tests` → `t_hal_loopback`) |
| `udp` | sensor frames out to the OBC, actuator frames back | OILS (flight OBC in the loop), HILS (rig interface unit) |

Set it per run: `asils.run(id, case, 'set', struct('hal__backend', 'loopback'))`.
Real-time pacing: `'hal__realtime', 1` (wall clock = simulated time; required for OILS/HILS).

## 2. Frame layouts (little-endian)

**Sensor frame** (SILS → OBC), `asils.hal.pack_sensors`:

| field | type | LSB |
|---|---|---|
| gyro rate x,y,z | int32 ×3 | 5/2²³ rad/s (24-bit, ±5 rad/s) |
| magnetometer x,y,z | int16 ×3 | 1e-4/2¹⁵ T (3.05 nT, ±100 µT) |
| Sun unit vector | int16 ×3 | Q15 |
| flags | uint8 | bit0 Sun valid, bit1 star tracker valid, bit2 GNSS fix, bit3 coils off (clean field) |
| star-tracker quaternion (head 1) | int32 ×4 | Q30, scalar last |
| GNSS position | int32 ×3 | 1 cm |
| GNSS velocity | int32 ×3 | 1 mm/s |
| rotor momentum (wheels, rings, CMG rotors) | int32 × nr | 1e-9 N m s |
| gimbal angle | int32 × ng | 1e-7 rad |

**Actuator frame** (OBC → SILS), `asils.hal.unpack_actuators`:

| field | type | scaling |
|---|---|---|
| coil PWM duty x,y,z | int16 ×3 | Q15 of the part's full dipole (`adcs_hal_pwm_set`) |
| rotor torque command | int16 × nr | Q15 of the part's maximum torque |
| gimbal rate command | int16 × ng | Q15 of the maximum gimbal rate |
| thruster valve on-time | uint8 × nc | 1 % of the 100 ms control period (1 ms) |

## 3. HILS stimulus

`asils.hal.stimulus(rec)` returns, at the recording rate, what the rig must
reproduce: the field in the body frame (Helmholtz-cage command), the Sun
direction in the body frame (Sun-simulator pointing), the body rate (air-bearing
reference) and the sunlit fraction (lamp on/off). Every recorded run already
carries these channels (`B_*_T`, `sun_body_*`, `w_*_degps`, `shadow_nu` in
`channels.csv`), so a HILS test can replay any SILS scenario.

## 4. Environment models the rigs need

The environment stays in the SILS at every rung, driven by the in-loop precision
orbit: IGRF-13 (degree 13), DE440 Sun and Moon, conical shadow with penumbra,
DTM2020 density, co-rotating atmosphere, Earth albedo for the cosine sun
sensors. At HILS these are *commands* to the stimulators instead of inputs to
sensor models; nothing else changes.

## 5. Component levels and the dispatch package

Every unit runs at one of three levels. At `model`, its statistics only. At `chain`, its own
processing is in the loop: `+asils/+comp`, for example the star tracker's image → centroid →
identify → QUEST. At `hil`, the real unit is on the HAL link. Its HAL frame is the same at every
level, so one unit can move up a level at a time.

`asils.solution.dispatch(case)` writes what the OBC build needs for the recommended solution into
`dist/dispatch/<case>/<family>/`:
- `adcs_fsw_params.h`: the flight parameters the SILS flew;
- `modes.json`: per-mode methods and algorithms;
- `hal_map.json`: each fitted component, its HAL output and its chain.

## 6. Bring-up order

1. `loopback` on every scenario — the byte frames are right.
2. Soft OILS (`docs/SOFT_OILS.md`): the same flight software as Cortex-M4F firmware in QEMU on
   adcs-link/1, with exact per-step instruction counts turned into command latency; every scenario
   next to its SILS run (`tools/engine.py oils`) and the dispatched mission (`tools/pipeline.py`).
3. OILS with the flight OBC running the C flight software behind `adcs_hal.h`,
   `udp`, `realtime = 1`, detumble first (`detumble_ais`), then pointing.
4. HILS per device: replace one device model at a time by the real part
   (magnetometer in the cage first, then coils, then wheels on the air bearing),
   keeping the rest emulated.
5. Compare every OILS/HILS run with its SILS twin (`asils.viz.compare`-style
   overlays of `channels.csv`): a difference is a parity-ledger line with a cause.

The `udp` backend needs `udpport` (MATLAB R2020b+) or the Octave
`instrument-control` package; SILS needs neither.
