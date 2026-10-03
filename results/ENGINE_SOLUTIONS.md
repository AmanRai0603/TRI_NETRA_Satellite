# Solution matrix on the Rust engine

Owner: Agastya. `tools/engine.py solutions` -- every mission mode x option of
each case, flown on the case's sized products (matlab_sils/store/sized) with the C flight software,
seeds 1,2; the MATLAB column is `matlab_sils/store/solutions/<case>/solution.json` when collected.

## ais_3u

| mode | option | engine feasible | objective (worst seed) | failing on the engine | MATLAB feasible | MATLAB objective |
|---|---|---|---:|---|---|---:|
| Detumbling | mtq | yes | 49.06 detumble_time | — | yes | 70.09 |
| Detumbling | rcs | no | 1.092 detumble_time | power_peak | no | 1.042 |
| Sun acquisition | mtq | no | 110.4 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 | no | — |
| Sun acquisition | rw | no | 2.292 sun_acquisition_time | power_mean | no | 2.142 |
| Sun acquisition | cmg | no | 2.275 sun_acquisition_time | power_mean | no | 1.158 |
| Sun acquisition | vscmg | no | 2.958 sun_acquisition_time | power_mean | no | 2.358 |
| Sun acquisition | fmr | no | 2.292 sun_acquisition_time | power_mean | no | 2.142 |
| Sun referencing | mtq | no | 150.4 sun_ape_p9973 | sun_ape_p9973 | no | 131 |
| Sun referencing | rw+mtq | no | 0.3656 sun_ape_p9973 | power_mean | no | 0.2466 |
| Sun referencing | rw+rcs | no | 0.3656 sun_ape_p9973 | power_mean | no | 0.2253 |
| Sun referencing | cmg+mtq | no | 0.3654 sun_ape_p9973 | power_mean | no | 0.4133 |
| Sun referencing | cmg+rcs | no | 0.3655 sun_ape_p9973 | power_mean | no | 0.2239 |
| Sun referencing | vscmg+mtq | no | 0.3726 sun_ape_p9973 | power_mean | no | 29.16 |
| Sun referencing | vscmg+rcs | no | 0.3671 sun_ape_p9973 | power_mean | no | 24.89 |
| Sun referencing | fmr+mtq | no | 0.3673 sun_ape_p9973 | power_mean | no | 0.2451 |
| Sun referencing | fmr+rcs | no | 0.3675 sun_ape_p9973 | power_mean | no | 0.2261 |
| Nadir pointing | mtq | yes | 9.9 ape_los_p9973 | — | no | 15.32 |
| Nadir pointing | rw+mtq | no | 0.1466 ape_los_p9973 | power_mean | no | 0.2226 |
| Nadir pointing | rw+rcs | no | 0.1465 ape_los_p9973 | power_mean | no | 0.2101 |
| Nadir pointing | cmg+mtq | no | 0.1465 ape_los_p9973 | power_mean | no | 0.1866 |
| Nadir pointing | cmg+rcs | no | 0.1465 ape_los_p9973 | power_mean | no | 0.2011 |
| Nadir pointing | vscmg+mtq | no | 0.1501 ape_los_p9973 | power_mean | no | 0.1992 |
| Nadir pointing | vscmg+rcs | no | 0.1448 ape_los_p9973 | power_mean | no | 0.2041 |
| Nadir pointing | fmr+mtq | no | 0.1468 ape_los_p9973 | power_mean | no | 0.2225 |
| Nadir pointing | fmr+rcs | no | 0.1468 ape_los_p9973 | power_mean | no | 0.2091 |

## ais_img_3u

| mode | option | engine feasible | objective (worst seed) | failing on the engine | MATLAB feasible | MATLAB objective |
|---|---|---|---:|---|---|---:|
| Detumbling | mtq | yes | 64.12 detumble_time | — | yes | 58.87 |
| Detumbling | rcs | yes | 1.092 detumble_time | — | yes | 1.075 |
| Sun acquisition | mtq | no | 81.54 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 | no | — |
| Sun acquisition | rw | yes | 2.275 sun_acquisition_time | — | yes | 1.325 |
| Sun acquisition | cmg | yes | 2.275 sun_acquisition_time | — | yes | 1.792 |
| Sun acquisition | vscmg | yes | 2.975 sun_acquisition_time | — | yes | 3.042 |
| Sun acquisition | fmr | yes | 2.275 sun_acquisition_time | — | no | 1.325 |
| Sun referencing | mtq | no | 178.4 sun_ape_p9973 | sun_ape_p9973 | no | 162.3 |
| Sun referencing | rw+mtq | yes | 0.008654 sun_ape_p9973 | — | yes | 0.009302 |
| Sun referencing | rw+rcs | yes | 0.008616 sun_ape_p9973 | — | yes | 0.009094 |
| Sun referencing | cmg+mtq | yes | 0.00868 sun_ape_p9973 | — | yes | 0.009306 |
| Sun referencing | cmg+rcs | yes | 0.008648 sun_ape_p9973 | — | yes | 0.008412 |
| Sun referencing | vscmg+mtq | yes | 0.008541 sun_ape_p9973 | — | yes | 0.00935 |
| Sun referencing | vscmg+rcs | yes | 1.532 sun_ape_p9973 | — | yes | 0.01053 |
| Sun referencing | fmr+mtq | yes | 0.02556 sun_ape_p9973 | — | yes | 0.02511 |
| Sun referencing | fmr+rcs | no | 0.0256 sun_ape_p9973 | power_mean | no | 0.02434 |
| Nadir pointing | mtq | no | 164.6 ape_los_p9973 | ake_los_p9973, ape_los_p9973, rate_stability_p9973 | no | 168.9 |
| Nadir pointing | rw+mtq | yes | 0.005158 ape_los_p9973 | — | no | 0.007805 |
| Nadir pointing | rw+rcs | yes | 0.00521 ape_los_p9973 | — | no | 0.005535 |
| Nadir pointing | cmg+mtq | yes | 0.005284 ape_los_p9973 | — | no | 0.005729 |
| Nadir pointing | cmg+rcs | yes | 0.00528 ape_los_p9973 | — | no | 0.005468 |
| Nadir pointing | vscmg+mtq | yes | 0.006325 ape_los_p9973 | — | no | 0.01069 |
| Nadir pointing | vscmg+rcs | yes | 0.008358 ape_los_p9973 | — | no | 0.006233 |
| Nadir pointing | fmr+mtq | no | 0.02316 ape_los_p9973 | ape_los_p9973, rate_stability_p9973 | no | 0.02281 |
| Nadir pointing | fmr+rcs | no | 0.02321 ape_los_p9973 | ape_los_p9973, power_mean, rate_stability_p9973 | no | 0.02271 |

