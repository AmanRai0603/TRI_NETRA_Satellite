# Dispatch: mtq for case ais_3u

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

Magnetorquers only (solution). Modes passed: 1 of 4. Recommended for this case: **mtq**. no solution family passes every mode; best mtq (1/4 modes): sun_acquisition: mtq fails sun_acquisition_time, sun_angle_p95; sun_referencing: mtq fails sun_ape_p9973; nadir_pointing: mtq fails ape_los_p9973

Budget (sized): 0.236 kg, 2.06 W nominal, 0.018 L.

| mode | method | FSW state | feasible |
|---|---|---|---|
| Detumbling | mtq | detumble | yes |
| Sun acquisition | mtq | spinup | no |
| Sun referencing | mtq | sun_mtq | no |
| Nadir pointing | mtq | nadir_mtq | no |

## Files

- `product.json`: the sized hardware
- `modes.json`: method, algorithms and SILS evidence per mode
- `fsw_params.json`, `adcs_fsw_params.h`: flight parameters
- `hal_map.json`: components on the HAL and their chains

## Rungs

1. **SILS** (this evidence): `asils.solution.run('ais_3u')`, collected by `asils.solution.collect`.
2. **Algorithm hardening**: the component chains (`+asils/+comp`) ported to the units and the OBC, node by node.
3. **OILS**: the flight OBC runs the C flight software behind `adcs_hal.h`; `hal__backend = udp`, `hal__realtime = 1`.
4. **HILS**: each component replaces its model one at a time (docs/OILS_HILS.md).
