# Monte Carlo and edge-case campaigns: Rust engine vs MATLAB twin

Owner: Agastya. `tools/engine.py campaign` flies every campaign of `campaigns/*.toml` on the Rust engine
(POP in the loop, C flight software behind the byte HAL) with the draws of `asils.campaign.draw`:
the same dispersions, bounds, run count and per-run seeds (seed + 7919 k). Edge campaigns put each
dispersion at its low and high bound one at a time, then all at the adverse end.

Two differences are by design and are named, not hidden: the random streams (Mersenne twister vs
SplitMix64 / Python) so MC realisations differ and only distributions compare; and the flight software
on the engine keeps the NOMINAL (ground-calibrated) inertia while the plant is dispersed, whereas the
MATLAB twin's control laws read the dispersed inertia. The engine is therefore the more conservative.

## edge_fine_img — fine_hold_img on ais_img_3u (edge, 15 runs)

every dispersion of the imaging fine hold at its low and high bound, one at a time, then all at the adverse end

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_p9973 (deg) | 0.01 | 0.01236 ± 0.00475 [0.006594, 0.02038] | 40 % | 0.01625 ± 0.00351 [0.008576, 0.02168] | 7 % |
| ake_los_p9973 (deg) | 0.005 | 0.003307 ± 0.000418 [0.002792, 0.004166] | 100 % | 0.002739 ± 0.000362 [0.002327, 0.003714] | 100 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.003689 ± 0.000116 [0.003522, 0.003893] | 100 % | 0.003788 ± 0.00013 [0.003565, 0.003954] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.003344 ± 0.00013 [0.003255, 0.003651] | — | 0.003262 ± 7.91e-05 [0.003002, 0.003313] | — |
| power_mean (W) | 2 | 1.558 ± 0.00216 [1.556, 1.564] | 100 % | 1.557 ± 0.00112 [1.554, 1.56] | 100 % |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_p9973 MATLAB / engine | ake_los_p9973 MATLAB / engine | rate_stability_p9973 MATLAB / engine | power_mean MATLAB / engine |
|---|---|---|---|---|---|
| 1 | inertia low | 0.01549 / 0.01626 | 0.003219 / 0.002365 | 0.003808 / 0.003792 | 1.557 / 1.557 |
| 2 | inertia high | 0.0199 / 0.0154 | 0.002792 / 0.002602 | 0.003698 / 0.003634 | 1.557 / 1.557 |
| 3 | residual_dipole low | 0.02038 / 0.01962 | 0.002816 / 0.002554 | 0.003842 / 0.003797 | 1.557 / 1.556 |
| 4 | residual_dipole high | 0.009263 / 0.01314 | 0.002845 / 0.00317 | 0.003569 / 0.003775 | 1.564 / 1.56 |
| 5 | cm_offset low | 0.01034 / 0.01831 | 0.004003 / 0.002497 | 0.003609 / 0.00363 | 1.557 / 1.557 |
| 6 | cm_offset high | 0.01273 / 0.01911 | 0.003535 / 0.00294 | 0.003713 / 0.003954 | 1.557 / 1.556 |
| 7 | solar_flux low | 0.007503 / 0.01632 | 0.003246 / 0.002891 | 0.003593 / 0.003811 | 1.557 / 1.557 |
| 8 | solar_flux high | 0.006594 / 0.01661 | 0.003397 / 0.002489 | 0.003705 / 0.003952 | 1.556 / 1.557 |
| 9 | accommodation low | 0.00839 / 0.01801 | 0.003579 / 0.002566 | 0.003688 / 0.003759 | 1.557 / 1.558 |
| 10 | accommodation high | 0.02035 / 0.02001 | 0.003194 / 0.002327 | 0.003893 / 0.003943 | 1.557 / 1.557 |
| 11 | reflectivity low | 0.00834 / 0.01241 | 0.002814 / 0.002603 | 0.003677 / 0.003565 | 1.557 / 1.557 |
| 12 | reflectivity high | 0.01215 / 0.01632 | 0.003416 / 0.002546 | 0.003566 / 0.003898 | 1.557 / 1.556 |
| 13 | initial_error_deg low | 0.01287 / 0.02168 | 0.004166 / 0.003714 | 0.003852 / 0.003906 | 1.556 / 1.557 |
| 14 | initial_error_deg high | 0.01286 / 0.008576 | 0.003502 / 0.002845 | 0.003596 / 0.003799 | 1.557 / 1.557 |
| 15 | all adverse | 0.00818 / 0.01202 | 0.003073 / 0.002977 | 0.003522 / 0.003611 | 1.561 / 1.554 |

## edge_nadir_ais — nadir_hold_ais on ais_3u (edge, 17 runs)

every dispersion of the AIS nadir hold at its low and high bound, one at a time, then all at the adverse end

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 24.91 ± 40.4 [7.045, 155.1] | 59 % | 21.49 ± 29 [5.769, 125.5] | 29 % |
| ake_los_last_orbit (deg) | 5 | 3.712 ± 2.58 [0.9593, 11.79] | 82 % | 1.899 ± 0.905 [0.4541, 3.879] | 100 % |
| power_mean (W) | 0.5 | 0.01057 ± 0.00121 [0.006771, 0.01289] | 100 % | 0.01096 ± 0.00194 [0.006811, 0.01726] | 100 % |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_last_orbit MATLAB / engine | ake_los_last_orbit MATLAB / engine | power_mean MATLAB / engine |
|---|---|---|---|---|
| 1 | inertia low | 9.2 / 16.18 | 1.507 / 2.602 | 0.0105 / 0.01058 |
| 2 | inertia high | 7.232 / 9.084 | 5.853 / 2.927 | 0.01077 / 0.01089 |
| 3 | residual_dipole low | 34.87 / 28.82 | 1.122 / 2.018 | 0.006771 / 0.006811 |
| 4 | residual_dipole high | 155.1 / 125.5 | 4.176 / 1.874 | 0.01289 / 0.01726 |
| 5 | cm_offset low | 7.045 / 6.557 | 3.717 / 0.8748 | 0.01074 / 0.01057 |
| 6 | cm_offset high | 9.07 / 14.24 | 3.213 / 2.318 | 0.01065 / 0.01089 |
| 7 | solar_flux low | 10.32 / 10.08 | 4.274 / 1.448 | 0.01032 / 0.01054 |
| 8 | solar_flux high | 9.788 / 20.52 | 0.9593 / 3.879 | 0.0105 / 0.01067 |
| 9 | kp low | 10.44 / 9.7 | 2.501 / 1.785 | 0.01063 / 0.01072 |
| 10 | kp high | 15.34 / 13.94 | 2.388 / 0.4541 | 0.01066 / 0.01072 |
| 11 | accommodation low | 9.212 / 5.769 | 11.79 / 0.7401 | 0.01033 / 0.01057 |
| 12 | accommodation high | 8.96 / 12.13 | 3.238 / 3.007 | 0.01028 / 0.0107 |
| 13 | reflectivity low | 10.27 / 11.7 | 3.409 / 0.7029 | 0.01073 / 0.01056 |
| 14 | reflectivity high | 8.867 / 10.07 | 1.72 / 1.818 | 0.01059 / 0.01085 |
| 15 | initial_error_deg low | 8.153 / 10.46 | 2.389 / 1.956 | 0.01071 / 0.01079 |
| 16 | initial_error_deg high | 9.194 / 8.581 | 6.28 / 1.766 | 0.0103 / 0.01076 |
| 17 | all adverse | 100.3 / 51.96 | 4.568 / 2.114 | 0.01236 / 0.01239 |

## mc_agile_rw_rcs — agile_slew_rw_rcs on ais_img_3u (montecarlo, 20 runs)

agile 90 deg pitch slew in 15 s with wheels + cold-gas RCS across dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_on_target_p9973 (deg) | 0.01 | 0.008337 ± 0.000712 [0.00725, 0.00977] | 100 % | 0.008427 ± 0.00124 [0.006534, 0.0107] | 90 % |
| wheel_momentum_peak (N m s) | — | 0.003391 ± 4.68e-05 [0.003327, 0.003471] | — | 0.003373 ± 3.99e-05 [0.003304, 0.003482] | — |

## mc_detumble_ais — detumble_ais on ais_3u (montecarlo, 24 runs)

B-dot detumble of the AIS 3U across inertia, dipole, CM, flux and initial-rate dispersions

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| detumble_time (min) | 284 | 48.5 ± 12 [23.54, 75.24] | 100 % | 56.29 ± 11.4 [34.71, 80.47] | 100 % |
| power_mean (W) | 0.5 | 0.02267 ± 0.00701 [0.01143, 0.0355] | 100 % | 0.02764 ± 0.00684 [0.01561, 0.03891] | 100 % |
| power_peak (W) | 1.5 | 0.3902 ± 0.169 [0.1265, 0.638] | 100 % | 0.402 ± 0.16 [0.1762, 0.648] | 100 % |

## mc_fine_img — fine_hold_img on ais_img_3u (montecarlo, 24 runs)

Fine nadir hold of the imaging 3U (0.01 deg 3-sigma) across mass-property, magnetic, aero, SRP, space-weather dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_p9973 (deg) | 0.01 | 0.01315 ± 0.00434 [0.007152, 0.02007] | 29 % | 0.01456 ± 0.00486 [0.00657, 0.0214] | 25 % |
| ake_los_p9973 (deg) | 0.005 | 0.002894 ± 0.000605 [0.002, 0.00408] | 100 % | 0.002523 ± 0.000383 [0.001768, 0.003593] | 100 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.003738 ± 9.57e-05 [0.003559, 0.003962] | 100 % | 0.003755 ± 0.00013 [0.003507, 0.003953] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.003075 ± 0.000218 [0.002768, 0.003568] | — | 0.003114 ± 0.0002 [0.00273, 0.003456] | — |
| power_mean (W) | 2 | 1.558 ± 0.0021 [1.554, 1.563] | 100 % | 1.558 ± 0.00149 [1.556, 1.561] | 100 % |

## mc_nadir_ais — nadir_hold_ais on ais_3u (montecarlo, 24 runs)

Magnetic-only nadir hold of the AIS 3U (10 deg) across mass-property, magnetic, aero, SRP and space-weather dispersions and sensor/actuator part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 89.69 ± 38.5 [16.21, 165.2] | 0 % | 88.47 ± 43.7 [27.49, 165.3] | 0 % |
| ake_los_last_orbit (deg) | 5 | 3.3 ± 1.73 [1.096, 6.253] | 75 % | 1.651 ± 0.729 [0.4513, 2.876] | 100 % |
| power_mean (W) | 0.5 | 0.01145 ± 0.0025 [0.006323, 0.01589] | 100 % | 0.01013 ± 0.00302 [0.005447, 0.01585] | 100 % |

## mc_slew_cmg — slew_cmg on ais_img_3u (montecarlo, 20 runs)

30 deg slew with the 4-SGCMG pyramid across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 13.14 ± 0.46 [12.4, 14.3] | 100 % | 13.03 ± 0.785 [11.9, 14.7] | 100 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.003931 ± 0.000635 [0.002886, 0.005957] | 100 % | 0.003761 ± 0.000349 [0.003036, 0.004525] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.004 ± 0 [0.004, 0.004] | — | 0.004 ± 0 [0.004, 0.004] | — |

## mc_slew_img — slew_img on ais_img_3u (montecarlo, 40 runs)

30 deg target slew in 60 s and settle, imaging 3U, across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 13.03 ± 0.959 [11.1, 14.9] | 100 % | 13.84 ± 4.76 [9.8, 42.1] | 98 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.00815 ± 0.00115 [0.00601, 0.01114] | 92 % | 0.007886 ± 0.001 [0.006011, 0.0106] | 95 % |
| wheel_momentum_peak (N m s) | — | 0.003475 ± 0.000273 [0.002952, 0.003868] | — | 0.003428 ± 0.000281 [0.002998, 0.003886] | — |

