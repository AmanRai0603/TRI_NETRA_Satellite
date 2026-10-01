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
| ape_los_p9973 (deg) | 0.01 | 0.01236 ± 0.00475 [0.006594, 0.02038] | 40 % | 0.006695 ± 0.000423 [0.00586, 0.00731] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.003307 ± 0.000418 [0.002792, 0.004166] | 100 % | 0.00272 ± 0.000428 [0.002212, 0.00413] | 100 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.003689 ± 0.000116 [0.003522, 0.003893] | 100 % | 0.003673 ± 0.0001 [0.003517, 0.003801] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.003344 ± 0.00013 [0.003255, 0.003651] | — | 0.002693 ± 9.17e-05 [0.002597, 0.003] | — |
| power_mean (W) | 2 | 1.558 ± 0.00216 [1.556, 1.564] | 100 % | 1.56 ± 0.000857 [1.559, 1.562] | 100 % |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_p9973 MATLAB / engine | ake_los_p9973 MATLAB / engine | rate_stability_p9973 MATLAB / engine | power_mean MATLAB / engine |
|---|---|---|---|---|---|
| 1 | inertia low | 0.01549 / 0.006435 | 0.003219 / 0.002566 | 0.003808 / 0.003776 | 1.557 / 1.559 |
| 2 | inertia high | 0.0199 / 0.006428 | 0.002792 / 0.002769 | 0.003698 / 0.003567 | 1.557 / 1.56 |
| 3 | residual_dipole low | 0.02038 / 0.006974 | 0.002816 / 0.002789 | 0.003842 / 0.003714 | 1.557 / 1.561 |
| 4 | residual_dipole high | 0.009263 / 0.007214 | 0.002845 / 0.002676 | 0.003569 / 0.00374 | 1.564 / 1.562 |
| 5 | cm_offset low | 0.01034 / 0.006557 | 0.004003 / 0.002623 | 0.003609 / 0.003567 | 1.557 / 1.56 |
| 6 | cm_offset high | 0.01273 / 0.006047 | 0.003535 / 0.002626 | 0.003713 / 0.003669 | 1.557 / 1.559 |
| 7 | solar_flux low | 0.007503 / 0.006738 | 0.003246 / 0.002583 | 0.003593 / 0.00376 | 1.557 / 1.559 |
| 8 | solar_flux high | 0.006594 / 0.00731 | 0.003397 / 0.00413 | 0.003705 / 0.003731 | 1.556 / 1.56 |
| 9 | accommodation low | 0.00839 / 0.006637 | 0.003579 / 0.002572 | 0.003688 / 0.003714 | 1.557 / 1.56 |
| 10 | accommodation high | 0.02035 / 0.007143 | 0.003194 / 0.002776 | 0.003893 / 0.003523 | 1.557 / 1.559 |
| 11 | reflectivity low | 0.00834 / 0.006375 | 0.002814 / 0.002973 | 0.003677 / 0.00358 | 1.557 / 1.56 |
| 12 | reflectivity high | 0.01215 / 0.006754 | 0.003416 / 0.002486 | 0.003566 / 0.003798 | 1.557 / 1.56 |
| 13 | initial_error_deg low | 0.01287 / 0.00586 | 0.004166 / 0.00259 | 0.003852 / 0.003638 | 1.556 / 1.559 |
| 14 | initial_error_deg high | 0.01286 / 0.007105 | 0.003502 / 0.002212 | 0.003596 / 0.003801 | 1.557 / 1.559 |
| 15 | all adverse | 0.00818 / 0.006844 | 0.003073 / 0.002432 | 0.003522 / 0.003517 | 1.561 / 1.56 |

## edge_nadir_ais — nadir_hold_ais on ais_3u (edge, 17 runs)

every dispersion of the AIS nadir hold at its low and high bound, one at a time, then all at the adverse end

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 22.16 ± 28.2 [5.018, 97.4] | 41 % | 21.42 ± 28.9 [5.791, 125.2] | 35 % |
| ake_los_last_orbit (deg) | 5 | 2.577 ± 1.37 [0.9207, 5.153] | 94 % | 1.9 ± 0.908 [0.4544, 3.863] | 100 % |
| power_mean (W) | 0.5 | 0.01074 ± 0.00143 [0.006316, 0.01328] | 100 % | 0.01096 ± 0.00196 [0.006811, 0.01738] | 100 % |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_last_orbit MATLAB / engine | ake_los_last_orbit MATLAB / engine | power_mean MATLAB / engine |
|---|---|---|---|---|
| 1 | inertia low | 10.69 / 16.12 | 1.569 / 2.607 | 0.01077 / 0.01059 |
| 2 | inertia high | 6.16 / 9.131 | 4.687 / 2.934 | 0.01075 / 0.01089 |
| 3 | residual_dipole low | 59.15 / 28.44 | 1.319 / 2.001 | 0.006316 / 0.006811 |
| 4 | residual_dipole high | 97.4 / 125.2 | 0.9207 / 1.801 | 0.01328 / 0.01738 |
| 5 | cm_offset low | 9.227 / 6.588 | 2.307 / 0.8682 | 0.01052 / 0.01057 |
| 6 | cm_offset high | 10.63 / 14.28 | 3.529 / 2.308 | 0.01064 / 0.01089 |
| 7 | solar_flux low | 5.018 / 9.876 | 3.587 / 1.515 | 0.01058 / 0.01054 |
| 8 | solar_flux high | 22.79 / 20.57 | 1.449 / 3.863 | 0.01066 / 0.01067 |
| 9 | kp low | 10.79 / 9.759 | 1.041 / 1.789 | 0.01064 / 0.01072 |
| 10 | kp high | 11.02 / 13.93 | 4.166 / 0.4544 | 0.01065 / 0.01072 |
| 11 | accommodation low | 7.048 / 5.791 | 4.066 / 0.7375 | 0.0107 / 0.01057 |
| 12 | accommodation high | 9.974 / 12.06 | 1.112 / 3.085 | 0.01073 / 0.01069 |
| 13 | reflectivity low | 12.03 / 11.74 | 2.384 / 0.6964 | 0.01074 / 0.01055 |
| 14 | reflectivity high | 10.36 / 10.14 | 1.654 / 1.822 | 0.01087 / 0.01085 |
| 15 | initial_error_deg low | 6.384 / 10.35 | 5.153 / 1.928 | 0.01093 / 0.01079 |
| 16 | initial_error_deg high | 7.453 / 8.566 | 2.349 / 1.766 | 0.0105 / 0.01076 |
| 17 | all adverse | 80.56 / 51.61 | 2.52 / 2.121 | 0.01327 / 0.0124 |

## mc_agile_rw_rcs — agile_slew_rw_rcs on ais_img_3u (montecarlo, 20 runs)

agile 90 deg pitch slew in 15 s with wheels + cold-gas RCS across dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_on_target_p9973 (deg) | 0.01 | 0.008337 ± 0.000712 [0.00725, 0.00977] | 100 % | 0.008203 ± 0.00124 [0.006562, 0.01165] | 95 % |
| wheel_momentum_peak (N m s) | — | 0.003391 ± 4.68e-05 [0.003327, 0.003471] | — | 0.00342 ± 7.47e-05 [0.003334, 0.003587] | — |

## mc_detumble_ais — detumble_ais on ais_3u (montecarlo, 24 runs)

B-dot detumble of the AIS 3U across inertia, dipole, CM, flux and initial-rate dispersions

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| detumble_time (min) | 284 | 48.5 ± 12 [23.54, 75.24] | 100 % | 56.3 ± 11.5 [34.74, 80.52] | 100 % |
| power_mean (W) | 0.5 | 0.02267 ± 0.00701 [0.01143, 0.0355] | 100 % | 0.02772 ± 0.00692 [0.0156, 0.03885] | 100 % |
| power_peak (W) | 1.5 | 0.3902 ± 0.169 [0.1265, 0.638] | 100 % | 0.4014 ± 0.159 [0.1753, 0.6448] | 100 % |

## mc_fine_img — fine_hold_img on ais_img_3u (montecarlo, 24 runs)

Fine nadir hold of the imaging 3U (0.01 deg 3-sigma) across mass-property, magnetic, aero, SRP, space-weather dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_p9973 (deg) | 0.01 | 0.01315 ± 0.00434 [0.007152, 0.02007] | 29 % | 0.006716 ± 0.000362 [0.00588, 0.007339] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.002894 ± 0.000605 [0.002, 0.00408] | 100 % | 0.002531 ± 0.000391 [0.001751, 0.003408] | 100 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.003738 ± 9.57e-05 [0.003559, 0.003962] | 100 % | 0.003658 ± 0.000114 [0.003478, 0.003885] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.003075 ± 0.000218 [0.002768, 0.003568] | — | 0.00291 ± 0.000191 [0.002603, 0.003266] | — |
| power_mean (W) | 2 | 1.558 ± 0.0021 [1.554, 1.563] | 100 % | 1.56 ± 0.00397 [1.553, 1.568] | 100 % |

## mc_nadir_ais — nadir_hold_ais on ais_3u (montecarlo, 24 runs)

Magnetic-only nadir hold of the AIS 3U (10 deg) across mass-property, magnetic, aero, SRP and space-weather dispersions and sensor/actuator part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 91.95 ± 46.4 [17.14, 169.3] | 0 % | 88.63 ± 43.9 [27.5, 164.5] | 0 % |
| ake_los_last_orbit (deg) | 5 | 2.086 ± 1.02 [0.5506, 5.18] | 96 % | 1.683 ± 0.752 [0.4519, 2.837] | 100 % |
| power_mean (W) | 0.5 | 0.01165 ± 0.00288 [0.005911, 0.01693] | 100 % | 0.01015 ± 0.00305 [0.005449, 0.01583] | 100 % |

## mc_slew_cmg — slew_cmg on ais_img_3u (montecarlo, 20 runs)

30 deg slew with the 4-SGCMG pyramid across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 13.14 ± 0.46 [12.4, 14.3] | 100 % | 13.03 ± 0.773 [12, 14.7] | 100 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.003931 ± 0.000635 [0.002886, 0.005957] | 100 % | 0.003789 ± 0.000457 [0.002744, 0.004728] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.004 ± 0 [0.004, 0.004] | — | 0.004 ± 0 [0.004, 0.004] | — |

## mc_slew_img — slew_img on ais_img_3u (montecarlo, 40 runs)

30 deg target slew in 60 s and settle, imaging 3U, across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 13.03 ± 0.959 [11.1, 14.9] | 100 % | 13.58 ± 3 [9.8, 30.1] | 98 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.00815 ± 0.00115 [0.00601, 0.01114] | 92 % | 0.007854 ± 0.00102 [0.005974, 0.01068] | 98 % |
| wheel_momentum_peak (N m s) | — | 0.003475 ± 0.000273 [0.002952, 0.003868] | — | 0.002595 ± 0.000169 [0.002268, 0.00284] | — |

