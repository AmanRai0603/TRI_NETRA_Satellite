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
| ape_los_p9973 (deg) | 0.01 | 0.01236 ± 0.00475 [0.006594, 0.02038] | 40 % | 0.006706 ± 0.00043 [0.005852, 0.007363] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.003307 ± 0.000418 [0.002792, 0.004166] | 100 % | 0.002835 ± 0.000435 [0.002314, 0.004277] | 100 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.003689 ± 0.000116 [0.003522, 0.003893] | 100 % | 0.003673 ± 0.000103 [0.003514, 0.003807] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.003344 ± 0.00013 [0.003255, 0.003651] | — | 0.002693 ± 9.19e-05 [0.002598, 0.003001] | — |
| power_mean (W) | 2 | 1.558 ± 0.00216 [1.556, 1.564] | 100 % | 1.56 ± 0.000857 [1.559, 1.562] | 100 % |
| power_margin_last_orbit (W) | 0 | — | — | 1.644 ± 0.000864 [1.642, 1.645] | 100 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_p9973 | p99.73 | 15 | 0.007363 | 0.0106 | 0.006746 |
| ape_los_max | max | 15 | 0.0106 | 0.0106 | 0.0106 |
| ape_3ax_p9973 | p99.73 | 15 | 0.008411 | 0.0111 | 0.007508 |
| ake_los_p9973 | p99.73 | 15 | 0.004277 | 0.005331 | 0.003307 |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_p9973 MATLAB / engine | ake_los_p9973 MATLAB / engine | rate_stability_p9973 MATLAB / engine | power_mean MATLAB / engine | power_margin_last_orbit MATLAB / engine |
|---|---|---|---|---|---|---|
| 1 | inertia low | 0.01549 / 0.00642 | 0.003219 / 0.002682 | 0.003808 / 0.003773 | 1.557 / 1.559 | — / 1.644 |
| 2 | inertia high | 0.0199 / 0.006444 | 0.002792 / 0.002885 | 0.003698 / 0.003543 | 1.557 / 1.56 | — / 1.644 |
| 3 | residual_dipole low | 0.02038 / 0.006942 | 0.002816 / 0.002891 | 0.003842 / 0.003713 | 1.557 / 1.561 | — / 1.643 |
| 4 | residual_dipole high | 0.009263 / 0.007209 | 0.002845 / 0.002786 | 0.003569 / 0.003755 | 1.564 / 1.562 | — / 1.642 |
| 5 | cm_offset low | 0.01034 / 0.006575 | 0.004003 / 0.002755 | 0.003609 / 0.003586 | 1.557 / 1.56 | — / 1.644 |
| 6 | cm_offset high | 0.01273 / 0.006057 | 0.003535 / 0.002733 | 0.003713 / 0.003676 | 1.557 / 1.559 | — / 1.644 |
| 7 | solar_flux low | 0.007503 / 0.006722 | 0.003246 / 0.002697 | 0.003593 / 0.003766 | 1.557 / 1.559 | — / 1.645 |
| 8 | solar_flux high | 0.006594 / 0.007363 | 0.003397 / 0.004277 | 0.003705 / 0.00373 | 1.556 / 1.56 | — / 1.644 |
| 9 | accommodation low | 0.00839 / 0.006687 | 0.003579 / 0.002684 | 0.003688 / 0.003709 | 1.557 / 1.56 | — / 1.644 |
| 10 | accommodation high | 0.02035 / 0.007167 | 0.003194 / 0.002906 | 0.003893 / 0.003517 | 1.557 / 1.559 | — / 1.644 |
| 11 | reflectivity low | 0.00834 / 0.006375 | 0.002814 / 0.003064 | 0.003677 / 0.003576 | 1.557 / 1.56 | — / 1.644 |
| 12 | reflectivity high | 0.01215 / 0.006762 | 0.003416 / 0.002616 | 0.003566 / 0.003807 | 1.557 / 1.56 | — / 1.644 |
| 13 | initial_error_deg low | 0.01287 / 0.005852 | 0.004166 / 0.002683 | 0.003852 / 0.003637 | 1.556 / 1.559 | — / 1.645 |
| 14 | initial_error_deg high | 0.01286 / 0.007115 | 0.003502 / 0.002314 | 0.003596 / 0.003794 | 1.557 / 1.559 | — / 1.645 |
| 15 | all adverse | 0.00818 / 0.006902 | 0.003073 / 0.002558 | 0.003522 / 0.003514 | 1.561 / 1.56 | — / 1.644 |

## edge_nadir_ais — nadir_hold_ais on ais_3u (edge, 17 runs)

every dispersion of the AIS nadir hold at its low and high bound, one at a time, then all at the adverse end

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 22.16 ± 28.2 [5.018, 97.4] | 41 % | 21.29 ± 28.1 [5.721, 121.2] | 29 % |
| ake_los_last_orbit (deg) | 5 | 2.577 ± 1.37 [0.9207, 5.153] | 94 % | 1.876 ± 0.905 [0.4547, 3.858] | 100 % |
| power_mean (W) | 0.5 | 0.01074 ± 0.00143 [0.006316, 0.01328] | 100 % | 0.01096 ± 0.00198 [0.006737, 0.01742] | 100 % |
| power_margin_last_orbit (W) | 0 | — | — | 6.498 ± 0.485 [5.064, 7.277] | 100 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_3axis_last_orbit | max | 17 | 180 | 180 | 180 |
| ape_los_last_orbit | max | 17 | 121.2 | 121.2 | 121.2 |
| ake_los_last_orbit | max | 17 | 3.858 | 3.858 | 3.858 |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_last_orbit MATLAB / engine | ake_los_last_orbit MATLAB / engine | power_mean MATLAB / engine | power_margin_last_orbit MATLAB / engine |
|---|---|---|---|---|---|
| 1 | inertia low | 10.69 / 16.06 | 1.569 / 2.61 | 0.01077 / 0.01059 | — / 7.277 |
| 2 | inertia high | 6.16 / 8.962 | 4.687 / 2.875 | 0.01075 / 0.01089 | — / 6.35 |
| 3 | residual_dipole low | 59.15 / 26.98 | 1.319 / 1.944 | 0.006316 / 0.006737 | — / 6.361 |
| 4 | residual_dipole high | 97.4 / 121.2 | 0.9207 / 1.745 | 0.01328 / 0.01742 | — / 5.064 |
| 5 | cm_offset low | 9.227 / 6.597 | 2.307 / 0.8599 | 0.01052 / 0.01056 | — / 6.311 |
| 6 | cm_offset high | 10.63 / 14.24 | 3.529 / 2.298 | 0.01064 / 0.01089 | — / 6.273 |
| 7 | solar_flux low | 5.018 / 10.66 | 3.587 / 1.316 | 0.01058 / 0.01055 | — / 5.869 |
| 8 | solar_flux high | 22.79 / 20.64 | 1.449 / 3.858 | 0.01066 / 0.01068 | — / 6.577 |
| 9 | kp low | 10.79 / 9.863 | 1.041 / 1.788 | 0.01064 / 0.01071 | — / 6.555 |
| 10 | kp high | 11.02 / 13.98 | 4.166 / 0.4547 | 0.01065 / 0.01073 | — / 6.768 |
| 11 | accommodation low | 7.048 / 5.721 | 4.066 / 0.7352 | 0.0107 / 0.01057 | — / 6.968 |
| 12 | accommodation high | 9.974 / 12.22 | 1.112 / 3.021 | 0.01073 / 0.0107 | — / 6.773 |
| 13 | reflectivity low | 12.03 / 11.79 | 2.384 / 0.696 | 0.01074 / 0.01056 | — / 6.714 |
| 14 | reflectivity high | 10.36 / 10.26 | 1.654 / 1.82 | 0.01087 / 0.01085 | — / 6.63 |
| 15 | initial_error_deg low | 6.384 / 10.7 | 5.153 / 2.022 | 0.01093 / 0.01078 | — / 6.588 |
| 16 | initial_error_deg high | 7.453 / 8.63 | 2.349 / 1.766 | 0.0105 / 0.01076 | — / 6.542 |
| 17 | all adverse | 80.56 / 53.47 | 2.52 / 2.076 | 0.01327 / 0.01234 | — / 6.846 |

## mc_agile_rw_rcs — agile_slew_rw_rcs on ais_img_3u (montecarlo, 1109 runs)

agile 90 deg pitch slew in 15 s with wheels + cold-gas RCS across dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_on_target_p9973 (deg) | 0.01 | 0.008337 ± 0.000712 [0.00725, 0.00977] | 100 % | 0.009573 ± 0.0134 [0.005805, 0.222] | 92 % |
| wheel_momentum_peak (N m s) | — | 0.003391 ± 4.68e-05 [0.003327, 0.003471] | — | 0.003441 ± 8.98e-05 [0.00325, 0.003818] | — |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_on_target_p9973 | p99.73 | 1109 | 0.222 | 0.1711 | 0.1122 |

## mc_detumble_ais — detumble_ais on ais_3u (montecarlo, 24 runs)

B-dot detumble of the AIS 3U across inertia, dipole, CM, flux and initial-rate dispersions

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| detumble_time (min) | 284 | 48.5 ± 12 [23.54, 75.24] | 100 % | 55.14 ± 13.8 [26.14, 77.89] | 100 % |
| power_mean (W) | 0.5 | 0.02267 ± 0.00701 [0.01143, 0.0355] | 100 % | 0.02831 ± 0.00826 [0.01557, 0.04437] | 100 % |
| power_peak (W) | 1.5 | 0.3902 ± 0.169 [0.1265, 0.638] | 100 % | 0.363 ± 0.158 [0.1522, 0.6635] | 100 % |

## mc_fine_img — fine_hold_img on ais_img_3u (montecarlo, 1109 runs)

Fine nadir hold of the imaging 3U (0.01 deg 3-sigma) across mass-property, magnetic, aero, SRP, space-weather dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_p9973 (deg) | 0.01 | 0.01315 ± 0.00434 [0.007152, 0.02007] | 29 % | 0.008521 ± 0.0144 [0.005565, 0.2763] | 95 % |
| ake_los_p9973 (deg) | 0.005 | 0.002894 ± 0.000605 [0.002, 0.00408] | 100 % | 0.003942 ± 0.00769 [0.001697, 0.09422] | 95 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.003738 ± 9.57e-05 [0.003559, 0.003962] | 100 % | 0.003787 ± 0.00185 [0.003331, 0.04177] | 98 % |
| wheel_momentum_peak (N m s) | — | 0.003075 ± 0.000218 [0.002768, 0.003568] | — | 0.002827 ± 0.000249 [0.002341, 0.003514] | — |
| power_mean (W) | 2 | 1.558 ± 0.0021 [1.554, 1.563] | 100 % | 1.562 ± 0.00564 [1.548, 1.577] | 100 % |
| power_margin_last_orbit (W) | 0 | — | — | 2.019 ± 0.394 [1.069, 2.812] | 100 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_p9973 | p99.73 | 1109 | 0.2763 | 0.06021 | 0.01556 |
| ape_los_max | max | 1109 | 0.2947 | 0.2947 | 0.2947 |
| ape_3ax_p9973 | p99.73 | 1109 | 0.2763 | 0.0624 | 0.02099 |
| ake_los_p9973 | p99.73 | 1109 | 0.09422 | 0.05724 | 0.01392 |

## mc_nadir_ais — nadir_hold_ais on ais_3u (montecarlo, 24 runs)

Magnetic-only nadir hold of the AIS 3U (10 deg) across mass-property, magnetic, aero, SRP and space-weather dispersions and sensor/actuator part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 91.95 ± 46.4 [17.14, 169.3] | 0 % | 74.69 ± 33.8 [28.88, 150.3] | 0 % |
| ake_los_last_orbit (deg) | 5 | 2.086 ± 1.02 [0.5506, 5.18] | 96 % | 1.6 ± 0.752 [0.4434, 3.198] | 100 % |
| power_mean (W) | 0.5 | 0.01165 ± 0.00288 [0.005911, 0.01693] | 100 % | 0.009984 ± 0.00307 [0.005894, 0.01589] | 100 % |
| power_margin_last_orbit (W) | 0 | — | — | 7.338 ± 1.61 [4.45, 9.645] | 100 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_3axis_last_orbit | max | 24 | 180 | 180 | 180 |
| ape_los_last_orbit | max | 24 | 150.3 | 150.3 | 150.3 |
| ake_los_last_orbit | max | 24 | 3.198 | 3.198 | 3.198 |

## mc_slew_cmg — slew_cmg on ais_img_3u (montecarlo, 1109 runs)

30 deg slew with the 4-SGCMG pyramid across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 13.14 ± 0.46 [12.4, 14.3] | 100 % | 18.16 ± 30.6 [7.9, 300] | 95 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.003931 ± 0.000635 [0.002886, 0.005957] | 100 % | 0.008919 ± 0.0221 [0.002695, 0.218] | 91 % |
| wheel_momentum_peak (N m s) | — | 0.004 ± 0 [0.004, 0.004] | — | 0.004 ± 0 [0.004, 0.004] | — |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_on_target_p9973 | p99.73 | 1109 | 0.218 | 0.1965 | 0.1332 |

## mc_slew_img — slew_img on ais_img_3u (montecarlo, 1109 runs)

30 deg target slew in 60 s and settle, imaging 3U, across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 13.03 ± 0.959 [11.1, 14.9] | 100 % | 18.81 ± 32.1 [7.9, 309.1] | 94 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.00815 ± 0.00115 [0.00601, 0.01114] | 92 % | 0.01289 ± 0.0216 [0.005454, 0.219] | 86 % |
| wheel_momentum_peak (N m s) | — | 0.003475 ± 0.000273 [0.002952, 0.003868] | — | 0.002586 ± 0.000134 [0.002223, 0.002894] | — |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_on_target_p9973 | p99.73 | 1109 | 0.219 | 0.1955 | 0.133 |

