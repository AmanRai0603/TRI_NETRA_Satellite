# Monte Carlo and edge-case campaigns: Rust engine vs MATLAB twin

Owner: Agastya. `tools/engine.py campaign` flies every campaign of `campaigns/*.toml` on the Rust engine
(POP in the loop, C flight software behind the byte HAL) with the draws of `asils.campaign.draw`:
the same dispersions, bounds, run count and per-run seeds (seed + 7919 k). Edge campaigns put each
dispersion at its low and high bound one at a time, then all at the adverse end.

Two differences are by design and are named, not hidden: the random streams (Mersenne twister vs
SplitMix64 / Python) so MC realisations differ and only distributions compare; and the flight software
on the engine keeps the NOMINAL (ground-calibrated) inertia while the plant is dispersed, whereas the
MATLAB twin's control laws read the dispersed inertia.

The run counts are not the same on both sides (mc_agile_rw_rcs: MATLAB twin 20, engine 1109; mc_fine_img: MATLAB twin 24, engine 1109; mc_slew_cmg: MATLAB twin 20, engine 1109; mc_slew_img: MATLAB twin 40, engine 1109), so pass rates and spreads compare samples of different size.
Of the 22 judged metrics that both sides report, the engine's pass rate is lower than the twin's on 13 (the engine the more conservative there), higher on 4 (the MATLAB twin the more conservative there), equal on 5.

## edge_fine_img — fine_hold_img on ais_img_3u (edge, 15 runs)

every dispersion of the imaging fine hold at its low and high bound, one at a time, then all at the adverse end

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_p9973 (deg) | 0.01 | 0.01236 ± 0.00475 [0.006594, 0.02038] | 40 % | 0.00502 ± 0.000219 [0.004747, 0.005485] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.003307 ± 0.000418 [0.002792, 0.004166] | 100 % | 0.002652 ± 0.000441 [0.002287, 0.004083] | 100 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.003689 ± 0.000116 [0.003522, 0.003893] | 100 % | 0.00372 ± 5.73e-05 [0.003597, 0.003795] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.003344 ± 0.00013 [0.003255, 0.003651] | — | 0.002694 ± 9.18e-05 [0.002598, 0.003001] | — |
| power_mean (W) | 2 | 1.558 ± 0.00216 [1.556, 1.564] | 100 % | 5.066 ± 0.000898 [5.065, 5.068] | 0 % |
| power_margin_last_orbit (W) | 0 | — | — | -1.866 ± 0.00098 [-1.869, -1.865] | 0 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_p9973 | p99.73 | 15 | 0.005485 | 0.007548 | 0.005072 |
| ape_los_max | max | 15 | 0.007548 | 0.007548 | 0.007548 |
| ape_3ax_p9973 | p99.73 | 15 | 0.00691 | 0.008566 | 0.006165 |
| ake_los_p9973 | p99.73 | 15 | 0.004083 | 0.005347 | 0.002923 |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_p9973 MATLAB / engine | ake_los_p9973 MATLAB / engine | rate_stability_p9973 MATLAB / engine | power_mean MATLAB / engine | power_margin_last_orbit MATLAB / engine |
|---|---|---|---|---|---|---|
| 1 | inertia low | 0.01549 / 0.00476 | 0.003219 / 0.002346 | 0.003808 / 0.003746 | 1.557 / 5.065 | — / -1.866 |
| 2 | inertia high | 0.0199 / 0.004862 | 0.002792 / 0.002689 | 0.003698 / 0.003693 | 1.557 / 5.065 | — / -1.866 |
| 3 | residual_dipole low | 0.02038 / 0.004939 | 0.002816 / 0.002741 | 0.003842 / 0.003704 | 1.557 / 5.067 | — / -1.868 |
| 4 | residual_dipole high | 0.009263 / 0.0054 | 0.002845 / 0.00267 | 0.003569 / 0.003784 | 1.564 / 5.068 | — / -1.869 |
| 5 | cm_offset low | 0.01034 / 0.004747 | 0.004003 / 0.002358 | 0.003609 / 0.003763 | 1.557 / 5.066 | — / -1.866 |
| 6 | cm_offset high | 0.01273 / 0.005054 | 0.003535 / 0.002528 | 0.003713 / 0.003742 | 1.557 / 5.065 | — / -1.866 |
| 7 | solar_flux low | 0.007503 / 0.005108 | 0.003246 / 0.002815 | 0.003593 / 0.003755 | 1.557 / 5.065 | — / -1.866 |
| 8 | solar_flux high | 0.006594 / 0.005485 | 0.003397 / 0.004083 | 0.003705 / 0.003778 | 1.556 / 5.066 | — / -1.867 |
| 9 | accommodation low | 0.00839 / 0.004849 | 0.003579 / 0.002371 | 0.003688 / 0.003597 | 1.557 / 5.065 | — / -1.866 |
| 10 | accommodation high | 0.02035 / 0.004966 | 0.003194 / 0.00255 | 0.003893 / 0.003717 | 1.557 / 5.065 | — / -1.866 |
| 11 | reflectivity low | 0.00834 / 0.005064 | 0.002814 / 0.002914 | 0.003677 / 0.003722 | 1.557 / 5.066 | — / -1.866 |
| 12 | reflectivity high | 0.01215 / 0.004803 | 0.003416 / 0.002657 | 0.003566 / 0.003726 | 1.557 / 5.065 | — / -1.867 |
| 13 | initial_error_deg low | 0.01287 / 0.00498 | 0.004166 / 0.002458 | 0.003852 / 0.003795 | 1.556 / 5.065 | — / -1.866 |
| 14 | initial_error_deg high | 0.01286 / 0.005061 | 0.003502 / 0.002319 | 0.003596 / 0.00366 | 1.557 / 5.065 | — / -1.865 |
| 15 | all adverse | 0.00818 / 0.005226 | 0.003073 / 0.002287 | 0.003522 / 0.003625 | 1.561 / 5.067 | — / -1.868 |

## edge_nadir_ais — nadir_hold_ais on ais_3u (edge, 17 runs)

every dispersion of the AIS nadir hold at its low and high bound, one at a time, then all at the adverse end

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 22.16 ± 28.2 [5.018, 97.4] | 41 % | 20.89 ± 28.4 [5.719, 121.6] | 35 % |
| ake_los_last_orbit (deg) | 5 | 2.577 ± 1.37 [0.9207, 5.153] | 94 % | 1.885 ± 0.909 [0.4543, 3.836] | 100 % |
| power_mean (W) | 0.5 | 0.01074 ± 0.00143 [0.006316, 0.01328] | 100 % | 1.511 ± 0.00197 [1.507, 1.517] | 0 % |
| power_margin_last_orbit (W) | 0 | — | — | 4.963 ± 0.45 [3.637, 5.471] | 100 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_3axis_last_orbit | max | 17 | 180 | 180 | 180 |
| ape_los_last_orbit | max | 17 | 121.6 | 121.6 | 121.6 |
| ake_los_last_orbit | max | 17 | 3.836 | 3.836 | 3.836 |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_last_orbit MATLAB / engine | ake_los_last_orbit MATLAB / engine | power_mean MATLAB / engine | power_margin_last_orbit MATLAB / engine |
|---|---|---|---|---|---|
| 1 | inertia low | 10.69 / 11.93 | 1.569 / 2.556 | 0.01077 / 1.511 | — / 5.011 |
| 2 | inertia high | 6.16 / 5.739 | 4.687 / 3.122 | 0.01075 / 1.511 | — / 5.228 |
| 3 | residual_dipole low | 59.15 / 27.04 | 1.319 / 1.947 | 0.006316 / 1.507 | — / 4.863 |
| 4 | residual_dipole high | 97.4 / 121.6 | 0.9207 / 1.766 | 0.01328 / 1.517 | — / 3.637 |
| 5 | cm_offset low | 9.227 / 6.17 | 2.307 / 0.8678 | 0.01052 / 1.511 | — / 4.694 |
| 6 | cm_offset high | 10.63 / 14.24 | 3.529 / 2.305 | 0.01064 / 1.511 | — / 4.732 |
| 7 | solar_flux low | 5.018 / 9.984 | 3.587 / 1.45 | 0.01058 / 1.511 | — / 4.234 |
| 8 | solar_flux high | 22.79 / 20.7 | 1.449 / 3.836 | 0.01066 / 1.511 | — / 5.083 |
| 9 | kp low | 10.79 / 9.854 | 1.041 / 1.788 | 0.01064 / 1.511 | — / 5.052 |
| 10 | kp high | 11.02 / 13.98 | 4.166 / 0.4543 | 0.01065 / 1.511 | — / 5.239 |
| 11 | accommodation low | 7.048 / 5.719 | 4.066 / 0.7351 | 0.0107 / 1.511 | — / 5.471 |
| 12 | accommodation high | 9.974 / 12.3 | 1.112 / 2.977 | 0.01073 / 1.511 | — / 5.26 |
| 13 | reflectivity low | 12.03 / 11.79 | 2.384 / 0.6963 | 0.01074 / 1.511 | — / 5.19 |
| 14 | reflectivity high | 10.36 / 10.25 | 1.654 / 1.821 | 0.01087 / 1.511 | — / 5.129 |
| 15 | initial_error_deg low | 6.384 / 10.59 | 5.153 / 1.998 | 0.01093 / 1.511 | — / 5.073 |
| 16 | initial_error_deg high | 7.453 / 8.684 | 2.349 / 1.766 | 0.0105 / 1.511 | — / 5.069 |
| 17 | all adverse | 80.56 / 54.52 | 2.52 / 1.954 | 0.01327 / 1.512 | — / 5.405 |

## mc_agile_rw_rcs — agile_slew_rw_rcs on ais_img_3u (montecarlo, 1109 runs)

agile 90 deg pitch slew in 15 s with wheels + cold-gas RCS across dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_on_target_p9973 (deg) | 0.01 | 0.008337 ± 0.000712 [0.00725, 0.00977] | 100 % | 0.007945 ± 0.0132 [0.004846, 0.2514] | 98 % |
| wheel_momentum_peak (N m s) | — | 0.003391 ± 4.68e-05 [0.003327, 0.003471] | — | 0.00358 ± 8.89e-05 [0.003393, 0.003956] | — |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_on_target_p9973 | p99.73 | 1109 | 0.2514 | 0.1688 | 0.1067 |

## mc_detumble_ais — detumble_ais on ais_3u (montecarlo, 24 runs)

B-dot detumble of the AIS 3U across inertia, dipole, CM, flux and initial-rate dispersions

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| detumble_time (min) | 284 | 48.5 ± 12 [23.54, 75.24] | 100 % | 55.61 ± 13.4 [26.14, 77.89] | 100 % |
| power_mean (W) | 0.5 | 0.02267 ± 0.00701 [0.01143, 0.0355] | 100 % | 1.528 ± 0.00823 [1.516, 1.544] | 0 % |
| power_peak (W) | 1.5 | 0.3902 ± 0.169 [0.1265, 0.638] | 100 % | 1.864 ± 0.158 [1.652, 2.163] | 0 % |

## mc_fine_img — fine_hold_img on ais_img_3u (montecarlo, 1109 runs)

Fine nadir hold of the imaging 3U (0.01 deg 3-sigma) across mass-property, magnetic, aero, SRP, space-weather dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_p9973 (deg) | 0.01 | 0.01315 ± 0.00434 [0.007152, 0.02007] | 29 % | 0.007036 ± 0.00788 [0.004578, 0.08277] | 92 % |
| ake_los_p9973 (deg) | 0.005 | 0.002894 ± 0.000605 [0.002, 0.00408] | 100 % | 0.004817 ± 0.00838 [0.002141, 0.08248] | 91 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.003738 ± 9.57e-05 [0.003559, 0.003962] | 100 % | 0.003747 ± 8.68e-05 [0.003517, 0.004146] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.003075 ± 0.000218 [0.002768, 0.003568] | — | 0.002837 ± 0.000243 [0.002341, 0.003514] | — |
| power_mean (W) | 2 | 1.558 ± 0.0021 [1.554, 1.563] | 100 % | 5.066 ± 0.0046 [5.056, 5.078] | 0 % |
| power_margin_last_orbit (W) | 0 | — | — | -1.489 ± 0.394 [-2.439, -0.6998] | 0 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_p9973 | p99.73 | 1109 | 0.08277 | 0.0446 | 0.01281 |
| ape_los_max | max | 1109 | 0.08471 | 0.08471 | 0.08471 |
| ape_3ax_p9973 | p99.73 | 1109 | 0.08295 | 0.05853 | 0.01659 |
| ake_los_p9973 | p99.73 | 1109 | 0.08248 | 0.04356 | 0.01258 |

## mc_nadir_ais — nadir_hold_ais on ais_3u (montecarlo, 24 runs)

Magnetic-only nadir hold of the AIS 3U (10 deg) across mass-property, magnetic, aero, SRP and space-weather dispersions and sensor/actuator part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 91.95 ± 46.4 [17.14, 169.3] | 0 % | 78.26 ± 38.7 [28.9, 174.9] | 0 % |
| ake_los_last_orbit (deg) | 5 | 2.086 ± 1.02 [0.5506, 5.18] | 96 % | 1.589 ± 0.748 [0.4068, 3.205] | 100 % |
| power_mean (W) | 0.5 | 0.01165 ± 0.00288 [0.005911, 0.01693] | 100 % | 1.51 ± 0.00292 [1.506, 1.516] | 0 % |
| power_margin_last_orbit (W) | 0 | — | — | 5.755 ± 1.66 [2.896, 8.133] | 100 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_3axis_last_orbit | max | 24 | 180 | 180 | 180 |
| ape_los_last_orbit | max | 24 | 174.9 | 174.9 | 174.9 |
| ake_los_last_orbit | max | 24 | 3.205 | 3.205 | 3.205 |

## mc_slew_cmg — slew_cmg on ais_img_3u (montecarlo, 1109 runs)

30 deg slew with the 4-SGCMG pyramid across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 13.14 ± 0.46 [12.4, 14.3] | 100 % | 11.96 ± 32.8 [0, 304.5] | 95 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.003931 ± 0.000635 [0.002886, 0.005957] | 100 % | 0.008808 ± 0.0223 [0.002537, 0.2133] | 91 % |
| wheel_momentum_peak (N m s) | — | 0.004 ± 0 [0.004, 0.004] | — | 0.004 ± 0 [0.004, 0.004] | — |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_on_target_p9973 | p99.73 | 1109 | 0.2133 | 0.1972 | 0.1291 |

## mc_slew_img — slew_img on ais_img_3u (montecarlo, 1109 runs)

30 deg target slew in 60 s and settle, imaging 3U, across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 13.03 ± 0.959 [11.1, 14.9] | 100 % | 12.16 ± 31.7 [1, 301.6] | 95 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.00815 ± 0.00115 [0.00601, 0.01114] | 92 % | 0.01139 ± 0.0219 [0.004485, 0.2139] | 91 % |
| wheel_momentum_peak (N m s) | — | 0.003475 ± 0.000273 [0.002952, 0.003868] | — | 0.002586 ± 0.000135 [0.002224, 0.002894] | — |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_on_target_p9973 | p99.73 | 1109 | 0.2139 | 0.1962 | 0.129 |

