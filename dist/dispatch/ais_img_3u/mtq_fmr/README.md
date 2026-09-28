# Dispatch: mtq_fmr for case ais_img_3u

**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.

Magnetorquers + fluid momentum loop (solution). Modes passed: 2 of 4. Recommended for this case: **mtq_fmr**. no solution family passes every mode; best mtq_fmr (2/4 modes): sun_acquisition: fmr fails power_mean; nadir_pointing: fmr+mtq fails ape_los_p9973, rate_stability_p9973

Budget (sized): 1.357 kg, 12.31 W nominal, 0.474 L.

| mode | method | FSW state | feasible |
|---|---|---|---|
| Detumbling | mtq | detumble | yes |
| Sun acquisition | fmr | sun_acq_rotor | no |
| Sun referencing | fmr+mtq | sun_fine | yes |
| Nadir pointing | fmr+mtq | nadir_fine | no |

## Files

- `product.json`: the sized hardware
- `modes.json`: method, algorithms and SILS evidence per mode
- `fsw_params.json`, `adcs_fsw_params.h`: flight parameters
- `hal_map.json`: components on the HAL and their chains

## Rungs

1. **SILS** (this evidence): `asils.solution.run('ais_img_3u')`, collected by `asils.solution.collect`.
2. **Algorithm hardening**: the component chains (`+asils/+comp`) ported to the units and the OBC, node by node.
3. **OILS**: the flight OBC runs the C flight software behind `adcs_hal.h`; `hal__backend = udp`, `hal__realtime = 1`.
4. **HILS**: each component replaces its model one at a time (docs/OILS_HILS.md).
