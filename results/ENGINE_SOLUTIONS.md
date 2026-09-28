# Solution matrix on the Rust engine

Owner: Agastya. `tools/engine.py solutions` -- every mission mode x option of
each case, flown on the case's sized products (matlab_sils/store/sized) with the C flight software,
seeds 1,2; the MATLAB column is `matlab_sils/store/solutions/<case>/solution.json` when collected.

## ais_3u

| mode | option | engine feasible | objective (worst seed) | failing on the engine | MATLAB feasible | MATLAB objective |
|---|---|---|---:|---|---|---:|
| Detumbling | mtq | yes | 48.86 detumble_time | — | — | — |
| Detumbling | rcs | no | 1.125 detumble_time | power_peak | — | — |
| Sun acquisition | mtq | no | 92.32 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 | — | — |
| Sun acquisition | rw | no | 2.292 sun_acquisition_time | power_mean | — | — |
| Sun acquisition | cmg | no | 2.292 sun_acquisition_time | power_mean | — | — |
| Sun acquisition | vscmg | no | 2.958 sun_acquisition_time | power_mean | — | — |
| Sun acquisition | fmr | no | 2.292 sun_acquisition_time | power_mean | — | — |
| Sun referencing | mtq | no | 157.9 sun_ape_p9973 | sun_ape_p9973 | — | — |
| Sun referencing | rw+mtq | no | 0.248 sun_ape_p9973 | power_mean | — | — |
| Sun referencing | rw+rcs | no | 0.5009 sun_ape_p9973 | power_mean | — | — |
| Sun referencing | cmg+mtq | no | 0.248 sun_ape_p9973 | power_mean | — | — |
| Sun referencing | cmg+rcs | no | 0.248 sun_ape_p9973 | power_mean | — | — |
| Sun referencing | vscmg+mtq | no | 13.42 sun_ape_p9973 | power_mean, sun_ape_p9973 | — | — |
| Sun referencing | vscmg+rcs | no | 3.167 sun_ape_p9973 | power_mean | — | — |
| Sun referencing | fmr+mtq | no | 0.2484 sun_ape_p9973 | power_mean | — | — |
| Sun referencing | fmr+rcs | no | 0.2485 sun_ape_p9973 | power_mean | — | — |
| Nadir pointing | mtq | yes | 9.009 ape_los_p9973 | — | — | — |
| Nadir pointing | rw+mtq | no | 0.1454 ape_los_p9973 | power_mean | — | — |
| Nadir pointing | rw+rcs | no | 0.1455 ape_los_p9973 | power_mean | — | — |
| Nadir pointing | cmg+mtq | no | 0.1454 ape_los_p9973 | power_mean | — | — |
| Nadir pointing | cmg+rcs | no | 0.1454 ape_los_p9973 | power_mean | — | — |
| Nadir pointing | vscmg+mtq | no | 0.1449 ape_los_p9973 | power_mean | — | — |
| Nadir pointing | vscmg+rcs | no | 0.1446 ape_los_p9973 | power_mean | — | — |
| Nadir pointing | fmr+mtq | no | 0.1443 ape_los_p9973 | power_mean | — | — |
| Nadir pointing | fmr+rcs | no | 0.1443 ape_los_p9973 | power_mean | — | — |

## ais_img_3u

| mode | option | engine feasible | objective (worst seed) | failing on the engine | MATLAB feasible | MATLAB objective |
|---|---|---|---:|---|---|---:|
| Detumbling | mtq | yes | 63.69 detumble_time | — | — | — |
| Detumbling | rcs | yes | 1.092 detumble_time | — | — | — |
| Sun acquisition | mtq | no | 94.01 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 | — | — |
| Sun acquisition | rw | yes | 2.275 sun_acquisition_time | — | — | — |
| Sun acquisition | cmg | yes | 2.275 sun_acquisition_time | — | — | — |
| Sun acquisition | vscmg | yes | 2.992 sun_acquisition_time | — | — | — |
| Sun acquisition | fmr | yes | 2.275 sun_acquisition_time | — | — | — |
| Sun referencing | mtq | no | 167.2 sun_ape_p9973 | sun_ape_p9973 | — | — |
| Sun referencing | rw+mtq | yes | 0.0863 sun_ape_p9973 | — | — | — |
| Sun referencing | rw+rcs | yes | 0.08632 sun_ape_p9973 | — | — | — |
| Sun referencing | cmg+mtq | yes | 0.08628 sun_ape_p9973 | — | — | — |
| Sun referencing | cmg+rcs | yes | 0.08627 sun_ape_p9973 | — | — | — |
| Sun referencing | vscmg+mtq | yes | 0.08658 sun_ape_p9973 | — | — | — |
| Sun referencing | vscmg+rcs | yes | 1.834 sun_ape_p9973 | — | — | — |
| Sun referencing | fmr+mtq | yes | 0.102 sun_ape_p9973 | — | — | — |
| Sun referencing | fmr+rcs | no | 0.1019 sun_ape_p9973 | power_mean | — | — |
| Nadir pointing | mtq | no | 177.1 ape_los_p9973 | ake_los_p9973, ape_los_p9973, rate_stability_p9973 | — | — |
| Nadir pointing | rw+mtq | no | 0.005261 ape_los_p9973 | rate_stability_p9973 | — | — |
| Nadir pointing | rw+rcs | no | 0.005074 ape_los_p9973 | rate_stability_p9973 | — | — |
| Nadir pointing | cmg+mtq | no | 0.004984 ape_los_p9973 | rate_stability_p9973 | — | — |
| Nadir pointing | cmg+rcs | no | 0.00499 ape_los_p9973 | rate_stability_p9973 | — | — |
| Nadir pointing | vscmg+mtq | no | 0.00677 ape_los_p9973 | rate_stability_p9973 | — | — |
| Nadir pointing | vscmg+rcs | no | 0.005109 ape_los_p9973 | rate_stability_p9973 | — | — |
| Nadir pointing | fmr+mtq | no | 0.02274 ape_los_p9973 | ape_los_p9973, rate_stability_p9973 | — | — |
| Nadir pointing | fmr+rcs | no | 0.02279 ape_los_p9973 | ape_los_p9973, power_mean, rate_stability_p9973 | — | — |

