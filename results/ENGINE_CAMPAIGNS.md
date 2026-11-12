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
Of the 26 judged metrics that both sides report, the engine's pass rate is lower than the twin's on 9 (the engine the more conservative there), higher on 3 (the MATLAB twin the more conservative there), equal on 14.

## edge_fine_img — fine_hold_img on ais_img_3u (edge, 15 runs)

every dispersion of the imaging fine hold at its low and high bound, one at a time, then all at the adverse end

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_p9973 (deg) | 0.01 | 0.00502 ± 0.000217 [0.004744, 0.005484] | 100 % | 0.00502 ± 0.000217 [0.004746, 0.005487] | 100 % |
| ake_los_p9973 (deg) | 0.005 | 0.002652 ± 0.000441 [0.002288, 0.004083] | 100 % | 0.002653 ± 0.000435 [0.002291, 0.004064] | 100 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.00372 ± 5.97e-05 [0.00359, 0.003804] | 100 % | 0.003721 ± 5.84e-05 [0.003595, 0.003796] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.002747 ± 0.000175 [0.002609, 0.003189] | — | 0.002694 ± 9.18e-05 [0.002598, 0.003001] | — |
| power_mean (W) | 2 | 5.065 ± 0.0011 [5.064, 5.069] | 0 % | 5.066 ± 0.000898 [5.065, 5.068] | 0 % |
| power_margin_last_orbit (W) | 0 | -1.866 ± 0.000753 [-1.869, -1.865] | 100 % | -1.867 ± 0.000981 [-1.869, -1.865] | 0 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_p9973 | p99.73 | 15 | 0.005487 | 0.007534 | 0.00507 |
| ape_los_max | max | 15 | 0.007534 | 0.007534 | 0.007534 |
| ape_3ax_p9973 | p99.73 | 15 | 0.006882 | 0.008726 | 0.006162 |
| ake_los_p9973 | p99.73 | 15 | 0.004064 | 0.005345 | 0.002916 |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_p9973 MATLAB / engine | ake_los_p9973 MATLAB / engine | rate_stability_p9973 MATLAB / engine | power_mean MATLAB / engine | power_margin_last_orbit MATLAB / engine |
|---|---|---|---|---|---|---|
| 1 | inertia low | 0.00476 / 0.00476 | 0.002348 / 0.002356 | 0.003745 / 0.003753 | 5.065 / 5.065 | -1.866 / -1.866 |
| 2 | inertia high | 0.004861 / 0.004861 | 0.002689 / 0.00268 | 0.003696 / 0.003699 | 5.065 / 5.065 | -1.866 / -1.866 |
| 3 | residual_dipole low | 0.004946 / 0.00495 | 0.002742 / 0.002746 | 0.003697 / 0.003702 | 5.066 / 5.067 | -1.866 / -1.868 |
| 4 | residual_dipole high | 0.005388 / 0.005398 | 0.002667 / 0.002668 | 0.003804 / 0.003782 | 5.069 / 5.068 | -1.869 / -1.869 |
| 5 | cm_offset low | 0.004744 / 0.004746 | 0.002358 / 0.002362 | 0.003762 / 0.003767 | 5.066 / 5.066 | -1.867 / -1.867 |
| 6 | cm_offset high | 0.00505 / 0.005048 | 0.002531 / 0.002531 | 0.003743 / 0.003742 | 5.065 / 5.065 | -1.866 / -1.866 |
| 7 | solar_flux low | 0.0051 / 0.005105 | 0.002813 / 0.002816 | 0.003753 / 0.003758 | 5.065 / 5.065 | -1.866 / -1.866 |
| 8 | solar_flux high | 0.005484 / 0.005487 | 0.004083 / 0.004064 | 0.003777 / 0.003774 | 5.066 / 5.066 | -1.867 / -1.867 |
| 9 | accommodation low | 0.004853 / 0.00485 | 0.00237 / 0.002372 | 0.00359 / 0.003595 | 5.065 / 5.065 | -1.866 / -1.866 |
| 10 | accommodation high | 0.004966 / 0.004964 | 0.00255 / 0.002554 | 0.003719 / 0.003724 | 5.065 / 5.065 | -1.866 / -1.866 |
| 11 | reflectivity low | 0.005065 / 0.005054 | 0.002917 / 0.00291 | 0.003712 / 0.003719 | 5.066 / 5.066 | -1.866 / -1.867 |
| 12 | reflectivity high | 0.004811 / 0.004813 | 0.002658 / 0.00266 | 0.003726 / 0.003729 | 5.065 / 5.065 | -1.867 / -1.867 |
| 13 | initial_error_deg low | 0.004979 / 0.004974 | 0.002456 / 0.002472 | 0.003794 / 0.003796 | 5.065 / 5.065 | -1.866 / -1.866 |
| 14 | initial_error_deg high | 0.005064 / 0.005069 | 0.002315 / 0.002319 | 0.003664 / 0.003661 | 5.064 / 5.065 | -1.865 / -1.865 |
| 15 | all adverse | 0.005228 / 0.005215 | 0.002288 / 0.002291 | 0.003625 / 0.00362 | 5.067 / 5.067 | -1.866 / -1.869 |

## edge_nadir_ais — nadir_hold_ais on ais_3u (edge, 17 runs)

every dispersion of the AIS nadir hold at its low and high bound, one at a time, then all at the adverse end

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 21.47 ± 23.7 [5.719, 80.15] | 35 % | 20.89 ± 28.4 [5.701, 121.4] | 29 % |
| ake_los_last_orbit (deg) | 5 | 1.929 ± 0.975 [0.4543, 3.836] | 100 % | 1.884 ± 0.907 [0.4556, 3.827] | 100 % |
| power_mean (W) | 0.5 | 1.511 ± 0.00136 [1.506, 1.513] | 0 % | 1.511 ± 0.00197 [1.507, 1.517] | 0 % |
| power_margin_last_orbit (W) | 0 | 5.056 ± 0.427 [4.109, 5.651] | 0 % | 4.965 ± 0.456 [3.617, 5.47] | 100 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_3axis_last_orbit | max | 17 | 180 | 180 | 180 |
| ape_los_last_orbit | max | 17 | 121.4 | 121.4 | 121.4 |
| ake_los_last_orbit | max | 17 | 3.827 | 3.827 | 3.827 |

Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):

| run | case | ape_los_last_orbit MATLAB / engine | ake_los_last_orbit MATLAB / engine | power_mean MATLAB / engine | power_margin_last_orbit MATLAB / engine |
|---|---|---|---|---|---|
| 1 | inertia low | 11.93 / 12.01 | 2.556 / 2.558 | 1.511 / 1.511 | 5.011 / 5.048 |
| 2 | inertia high | 5.739 / 5.82 | 3.122 / 3.114 | 1.511 / 1.511 | 5.228 / 5.231 |
| 3 | residual_dipole low | 59.57 / 27.08 | 1.69 / 1.945 | 1.506 / 1.507 | 4.394 / 4.864 |
| 4 | residual_dipole high | 70.47 / 121.4 | 1.627 / 1.761 | 1.513 / 1.517 | 4.109 / 3.617 |
| 5 | cm_offset low | 8.489 / 6.141 | 0.8134 / 0.8645 | 1.511 / 1.511 | 5.424 / 4.682 |
| 6 | cm_offset high | 10.68 / 14.24 | 2.59 / 2.304 | 1.511 / 1.511 | 5.651 / 4.721 |
| 7 | solar_flux low | 9.984 / 10.01 | 1.45 / 1.438 | 1.511 / 1.511 | 4.234 / 4.237 |
| 8 | solar_flux high | 20.7 / 20.75 | 3.836 / 3.827 | 1.511 / 1.511 | 5.083 / 5.086 |
| 9 | kp low | 9.854 / 9.851 | 1.788 / 1.789 | 1.511 / 1.511 | 5.052 / 5.056 |
| 10 | kp high | 13.98 / 13.96 | 0.4543 / 0.4556 | 1.511 / 1.511 | 5.239 / 5.259 |
| 11 | accommodation low | 5.719 / 5.701 | 0.7351 / 0.7323 | 1.511 / 1.511 | 5.471 / 5.47 |
| 12 | accommodation high | 12.3 / 12.32 | 2.977 / 2.966 | 1.511 / 1.511 | 5.26 / 5.264 |
| 13 | reflectivity low | 11.79 / 11.78 | 0.6963 / 0.6983 | 1.511 / 1.511 | 5.19 / 5.19 |
| 14 | reflectivity high | 10.25 / 10.26 | 1.821 / 1.822 | 1.511 / 1.511 | 5.129 / 5.132 |
| 15 | initial_error_deg low | 8.651 / 10.66 | 1.579 / 2.017 | 1.511 / 1.511 | 4.933 / 5.086 |
| 16 | initial_error_deg high | 14.74 / 8.655 | 3.086 / 1.769 | 1.51 / 1.511 | 5.351 / 5.057 |
| 17 | all adverse | 80.15 / 54.46 | 1.963 / 1.96 | 1.513 / 1.512 | 5.196 / 5.406 |

## mc_agile_rw_rcs — agile_slew_rw_rcs on ais_img_3u (montecarlo, 1109 runs)

agile 90 deg pitch slew in 15 s with wheels + cold-gas RCS across dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_on_target_p9973 (deg) | 0.01 | 0.01159 ± 0.0235 [0.005566, 0.1114] | 95 % | 0.007945 ± 0.0132 [0.004837, 0.2513] | 98 % |
| wheel_momentum_peak (N m s) | — | 0.003548 ± 5.21e-05 [0.00345, 0.003647] | — | 0.00358 ± 8.89e-05 [0.003393, 0.003956] | — |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_on_target_p9973 | p99.73 | 1109 | 0.2513 | 0.1689 | 0.1068 |

## mc_detumble_ais — detumble_ais on ais_3u (montecarlo, 24 runs)

B-dot detumble of the AIS 3U across inertia, dipole, CM, flux and initial-rate dispersions

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| detumble_time (min) | 284 | 56.64 ± 12.5 [31.17, 74.86] | 100 % | 55.61 ± 13.4 [26.14, 77.89] | 100 % |
| power_mean (W) | 0.5 | 1.527 ± 0.00791 [1.516, 1.547] | 0 % | 1.528 ± 0.00823 [1.516, 1.544] | 0 % |
| power_peak (W) | 1.5 | 1.861 ± 0.139 [1.669, 2.124] | 0 % | 1.864 ± 0.158 [1.652, 2.163] | 0 % |

## mc_fine_img — fine_hold_img on ais_img_3u (montecarlo, 1109 runs)

Fine nadir hold of the imaging 3U (0.01 deg 3-sigma) across mass-property, magnetic, aero, SRP, space-weather dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_p9973 (deg) | 0.01 | 0.006315 ± 0.00593 [0.004784, 0.03413] | 96 % | 0.007038 ± 0.00789 [0.004579, 0.08284] | 92 % |
| ake_los_p9973 (deg) | 0.005 | 0.004011 ± 0.00642 [0.002236, 0.03409] | 96 % | 0.004818 ± 0.00839 [0.002138, 0.08254] | 91 % |
| rate_stability_p9973 (deg/s) | 0.005 | 0.003774 ± 0.000103 [0.003585, 0.004059] | 100 % | 0.003747 ± 8.68e-05 [0.003518, 0.004159] | 100 % |
| wheel_momentum_peak (N m s) | — | 0.002856 ± 0.000266 [0.002455, 0.003359] | — | 0.002837 ± 0.000243 [0.002341, 0.003514] | — |
| power_mean (W) | 2 | 5.066 ± 0.00529 [5.057, 5.073] | 0 % | 5.066 ± 0.0046 [5.056, 5.078] | 0 % |
| power_margin_last_orbit (W) | 0 | -1.586 ± 0.405 [-2.39, -0.894] | 100 % | -1.489 ± 0.394 [-2.439, -0.6996] | 0 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_p9973 | p99.73 | 1109 | 0.08284 | 0.04472 | 0.01281 |
| ape_los_max | max | 1109 | 0.08478 | 0.08478 | 0.08478 |
| ape_3ax_p9973 | p99.73 | 1109 | 0.08301 | 0.05913 | 0.01658 |
| ake_los_p9973 | p99.73 | 1109 | 0.08254 | 0.04364 | 0.01255 |

## mc_nadir_ais — nadir_hold_ais on ais_3u (montecarlo, 24 runs)

Magnetic-only nadir hold of the AIS 3U (10 deg) across mass-property, magnetic, aero, SRP and space-weather dispersions and sensor/actuator part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| ape_los_last_orbit (deg) | 10 | 101.7 ± 47 [16.59, 163.8] | 0 % | 78.45 ± 38.5 [29.04, 174.8] | 0 % |
| ake_los_last_orbit (deg) | 5 | 1.449 ± 0.675 [0.4562, 3.076] | 100 % | 1.591 ± 0.748 [0.4063, 3.209] | 100 % |
| power_mean (W) | 0.5 | 1.512 ± 0.00285 [1.506, 1.516] | 0 % | 1.51 ± 0.00291 [1.506, 1.516] | 0 % |
| power_margin_last_orbit (W) | 0 | 6.825 ± 1.64 [4.261, 8.951] | 0 % | 5.741 ± 1.67 [2.891, 8.135] | 100 % |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_3axis_last_orbit | max | 24 | 180 | 180 | 180 |
| ape_los_last_orbit | max | 24 | 174.8 | 174.8 | 174.8 |
| ake_los_last_orbit | max | 24 | 3.209 | 3.209 | 3.209 |

## mc_slew_cmg — slew_cmg on ais_img_3u (montecarlo, 1109 runs)

30 deg slew with the 4-SGCMG pyramid across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 6.42 ± 0.984 [5, 8.7] | 100 % | 11.97 ± 32.8 [0, 304.5] | 95 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.003431 ± 0.000386 [0.002746, 0.004268] | 100 % | 0.008809 ± 0.0223 [0.002563, 0.2132] | 91 % |
| wheel_momentum_peak (N m s) | — | 0.004 ± 0 [0.004, 0.004] | — | 0.004 ± 0 [0.004, 0.004] | — |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_on_target_p9973 | p99.73 | 1109 | 0.2132 | 0.1971 | 0.129 |

## mc_slew_img — slew_img on ais_img_3u (montecarlo, 1109 runs)

30 deg target slew in 60 s and settle, imaging 3U, across inertia and disturbance dispersions and part errors

| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |
|---|---:|---|---:|---|---:|
| settle_time_after_slew (s) | 20 | 6.88 ± 1.54 [3.9, 9.9] | 100 % | 12.17 ± 31.7 [1, 301.6] | 95 % |
| ape_los_on_target_p9973 (deg) | 0.01 | 0.006286 ± 0.00116 [0.005216, 0.01145] | 98 % | 0.01139 ± 0.0219 [0.004473, 0.2137] | 91 % |
| wheel_momentum_peak (N m s) | — | 0.002567 ± 0.000142 [0.002285, 0.002847] | — | 0.002586 ± 0.000135 [0.002224, 0.002894] | — |

ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;
ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.

| metric | statistic | runs | temporal | ensemble | mixed |
|---|---|---:|---:|---:|---:|
| ape_los_on_target_p9973 | p99.73 | 1109 | 0.2137 | 0.1961 | 0.129 |


## Two-sample test — mc_nadir_ais (nadir_hold_ais on ais_3u)

The same 100 draws (the engine's, run k of the campaign's seed) were flown on the Rust engine and on the MATLAB twin; Mann-Whitney U test per metric, two-sided, normal approximation with tie and continuity corrections.
A p-value below 0.05 marks a metric on which the two simulators give different distributions; the metrics are tested one by one, with no correction for their number.

| metric | n | median MATLAB | median engine | U | z | p | differs at 5 % |
|---|---:|---:|---:|---:|---:|---:|---|
| ape_3axis_last_orbit | 100 / 100 | 180 | 180 | 4997 | 0.006108 | 0.9951 | no |
| ape_los_last_orbit | 100 / 100 | 72.37 | 72.37 | 4996 | 0.008552 | 0.9932 | no |
| ake_los_last_orbit | 100 / 100 | 1.637 | 1.637 | 5000 | 0 | 1 | no |
| power_mean | 100 / 100 | 1.511 | 1.511 | 4998 | 0.002443 | 0.9981 | no |
| power_margin_last_orbit | 100 / 100 | 6.032 | 6.032 | 4996 | 0.009774 | 0.9922 | no |
| battery_dod_last_orbit | 100 / 100 | 0.04534 | 0.04534 | 5000 | 0 | 1 | no |

