# Solution matrix on the Rust engine

Owner: Agastya. `tools/engine.py solutions` -- every mission mode x option of
each case, flown on the case's sized products (matlab_sils/store/sized) with the C flight software,
seeds 1,2; the MATLAB column is `matlab_sils/store/solutions/<case>/solution.json` when collected.

## ais_3u

| mode | option | engine feasible | objective (worst seed) | failing on the engine | MATLAB feasible | MATLAB objective |
|---|---|---|---:|---|---|---:|
| Detumbling | mtq | yes | 48.74 detumble_time | — | yes | 70.09 |
| Detumbling | rcs | no | 1.125 detumble_time | power_peak | no | 1.042 |
| Sun acquisition | mtq | no | — sun_acquisition_time | sun_acquisition_time, sun_angle_p95 | no | — |
| Sun acquisition | rw | no | 2.292 sun_acquisition_time | power_mean | no | 2.142 |
| Sun acquisition | cmg | no | 2.275 sun_acquisition_time | power_mean | no | 1.158 |
| Sun acquisition | vscmg | no | 2.958 sun_acquisition_time | power_mean | no | 2.358 |
| Sun acquisition | fmr | no | 2.292 sun_acquisition_time | power_mean | no | 2.142 |
| Sun referencing | mtq | no | 159.2 sun_ape_p9973 | sun_ape_p9973 | no | 131 |
| Sun referencing | rw+mtq | no | 0.2468 sun_ape_p9973 | power_mean | no | 0.2466 |
| Sun referencing | rw+rcs | no | 0.4947 sun_ape_p9973 | power_mean | no | 0.2253 |
| Sun referencing | cmg+mtq | no | 0.2467 sun_ape_p9973 | power_mean | no | 0.4133 |
| Sun referencing | cmg+rcs | no | 0.2467 sun_ape_p9973 | power_mean | no | 0.2239 |
| Sun referencing | vscmg+mtq | no | 11.1 sun_ape_p9973 | power_mean, sun_ape_p9973 | no | 29.16 |
| Sun referencing | vscmg+rcs | no | 6.76 sun_ape_p9973 | power_mean, sun_ape_p9973 | no | 24.89 |
| Sun referencing | fmr+mtq | no | 0.2469 sun_ape_p9973 | power_mean | no | 0.2451 |
| Sun referencing | fmr+rcs | no | 0.247 sun_ape_p9973 | power_mean | no | 0.2261 |
| Nadir pointing | mtq | no | 15.19 ape_los_p9973 | ape_los_p9973 | no | 15.32 |
| Nadir pointing | rw+mtq | no | 0.1481 ape_los_p9973 | power_mean | no | 0.2226 |
| Nadir pointing | rw+rcs | no | 0.1481 ape_los_p9973 | power_mean | no | 0.2101 |
| Nadir pointing | cmg+mtq | no | 0.148 ape_los_p9973 | power_mean | no | 0.1866 |
| Nadir pointing | cmg+rcs | no | 0.1481 ape_los_p9973 | power_mean | no | 0.2011 |
| Nadir pointing | vscmg+mtq | no | 0.1556 ape_los_p9973 | power_mean | no | 0.1992 |
| Nadir pointing | vscmg+rcs | no | 0.1518 ape_los_p9973 | power_mean | no | 0.2041 |
| Nadir pointing | fmr+mtq | no | 0.1478 ape_los_p9973 | power_mean | no | 0.2225 |
| Nadir pointing | fmr+rcs | no | 0.1478 ape_los_p9973 | power_mean | no | 0.2091 |

## ais_img_3u

| mode | option | engine feasible | objective (worst seed) | failing on the engine | MATLAB feasible | MATLAB objective |
|---|---|---|---:|---|---|---:|
| Detumbling | mtq | yes | 63.54 detumble_time | — | yes | 58.87 |
| Detumbling | rcs | yes | 1.092 detumble_time | — | yes | 1.075 |
| Sun acquisition | mtq | no | 98.47 sun_acquisition_time | sun_acquisition_time, sun_angle_p95 | no | — |
| Sun acquisition | rw | yes | 2.275 sun_acquisition_time | — | yes | 1.325 |
| Sun acquisition | cmg | yes | 2.275 sun_acquisition_time | — | yes | 1.792 |
| Sun acquisition | vscmg | yes | 2.975 sun_acquisition_time | — | yes | 3.042 |
| Sun acquisition | fmr | yes | 2.275 sun_acquisition_time | — | no | 1.325 |
| Sun referencing | mtq | no | 157.8 sun_ape_p9973 | sun_ape_p9973 | no | 162.3 |
| Sun referencing | rw+mtq | yes | 0.00865 sun_ape_p9973 | — | yes | 0.009302 |
| Sun referencing | rw+rcs | yes | 0.008624 sun_ape_p9973 | — | yes | 0.009094 |
| Sun referencing | cmg+mtq | yes | 0.00867 sun_ape_p9973 | — | yes | 0.009306 |
| Sun referencing | cmg+rcs | yes | 0.008654 sun_ape_p9973 | — | yes | 0.008412 |
| Sun referencing | vscmg+mtq | yes | 0.0085 sun_ape_p9973 | — | yes | 0.00935 |
| Sun referencing | vscmg+rcs | yes | 1.888 sun_ape_p9973 | — | yes | 0.01053 |
| Sun referencing | fmr+mtq | yes | 0.02559 sun_ape_p9973 | — | yes | 0.02511 |
| Sun referencing | fmr+rcs | no | 0.02553 sun_ape_p9973 | power_mean | no | 0.02434 |
| Nadir pointing | mtq | no | 172.8 ape_los_p9973 | ake_los_p9973, ape_los_p9973, rate_stability_p9973 | no | 168.9 |
| Nadir pointing | rw+mtq | no | 0.004933 ape_los_p9973 | rate_stability_p9973 | no | 0.007805 |
| Nadir pointing | rw+rcs | no | 0.004866 ape_los_p9973 | rate_stability_p9973 | no | 0.005535 |
| Nadir pointing | cmg+mtq | no | 0.004989 ape_los_p9973 | rate_stability_p9973 | no | 0.005729 |
| Nadir pointing | cmg+rcs | no | 0.005003 ape_los_p9973 | rate_stability_p9973 | no | 0.005468 |
| Nadir pointing | vscmg+mtq | no | 0.007184 ape_los_p9973 | rate_stability_p9973 | no | 0.01069 |
| Nadir pointing | vscmg+rcs | no | 0.005099 ape_los_p9973 | rate_stability_p9973 | no | 0.006233 |
| Nadir pointing | fmr+mtq | no | 0.02281 ape_los_p9973 | ape_los_p9973, rate_stability_p9973 | no | 0.02281 |
| Nadir pointing | fmr+rcs | no | 0.02278 ape_los_p9973 | ape_los_p9973, power_mean, rate_stability_p9973 | no | 0.02271 |

