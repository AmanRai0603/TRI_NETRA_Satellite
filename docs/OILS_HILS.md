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
| `sils` (default) | the bus's bytes in memory | every SILS run and campaign |
| `loopback` | every frame is packed to bytes and unpacked again each tick | proving the byte layout the OILS/HILS link carries (`tests/run_all_tests` → `t_hal_link`) |
| `udp` | sensor frames out to the OBC, actuator frames back | OILS (flight OBC in the loop), HILS (rig interface unit) |

Set it per run: `asils.run(id, case, 'set', struct('hal__backend', 'loopback'))`.
Real-time pacing: `'hal__realtime', 1` (wall clock = simulated time; required for OILS/HILS).

## 2. Frame layouts (little-endian)

Every sensor reaches the flight software, and every command leaves it, as the bytes of its part on the bus
(`asils.hal.bus`, the engine's bus): the I2C register blocks (magnetometer, Sun sensor, Earth sensor), the gyro's
SPI response, the star tracker's and the GNSS receiver's UART frames, the rotors' CAN telemetry, the coils' PWM
words and the CAN command frames. The counts are the design's emulator scaling (`asils.models.emucodec`, the
inverse of the flight drivers). The link carries that bus in two frames (`asils.hal.link`):

**Sensor frame** (SILS → OBC), `asils.hal.link('pack_sensors', bus)`:

| field | type |
|---|---|
| time | u64, ns |
| present | u8: bit0 magnetometer, bit1 gyro, bit2 Sun sensor, bit3 Earth sensor |
| magnetometer registers | 7 bytes |
| gyro SPI response | 13 bytes |
| Sun sensor registers | 7 bytes |
| Earth sensor registers | 7 bytes |
| UART port 1 (star tracker) | u16 n, n bytes |
| UART port 2 (GNSS) | u16 n, n bytes |
| CAN frames received (rotor telemetry) | u8 count, each u32 id, u8 dlc, 8 data |

**Command frame** (OBC → SILS), `asils.hal.link('pack_commands', bus)`:

| field | type |
|---|---|
| PWM words, channels 1–8 (coils on 1–3) | i16 ×8 |
| CAN frames sent (rotor commands 0x100+i, gimbal 0x140+j, valves 0x300) | u8 count, each u32 id, u8 dlc, 8 data |

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
