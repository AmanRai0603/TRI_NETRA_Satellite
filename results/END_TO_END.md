# Both cases end to end, from the design database

**In one line:** ais_3u, ais_img_3u through the design loop and their campaigns with every engine run reading its inputs from `build/end_to_end/Design/design.tndb` alone; every number is held to the one flown from the files, and every row and KPI closure is evaluated (`tools/end_to_end.py`, `docs/END_TO_END.md`).

| Case | Steps | Runs from the database | Numbers compared | Numbers that differ | Rows with a value | Closures answered | Closures blocked |
|---|---|---|---|---|---|---|---|
| `ais_3u` | the design loop (ok), its campaigns (ok) | 45 (not: 3) | 484 | 0 | 43 of 712 | 4 | 34 |
| `ais_img_3u` | the design loop (ok), its campaigns (ok) | 50 (not: 0) | 562 | 51 | 48 of 712 | 4 | 34 |

**ais_img_3u: numbers that differ from the ones stored before** (by name):

- `ais_img_3u.selection.families.mtq_fmr.budget.mass_kg`: 8.015074185008947 → 11.99557328644847
- `ais_img_3u.selection.families.mtq_fmr.budget.power_W`: 4.979764408616867 → 5.763282280603447
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_acquisition.metrics.power_mean`: 1.6395243760205616 → 1.6409696633142599
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_acquisition.metrics.power_peak`: 2.52337591525296 → 2.2637794018528057
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_acquisition.metrics.sun_angle_p95`: 0.4186339791031823 → 0.5879705618417329
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_referencing.metrics.ake_3ax_p9973`: 0.008806474484843237 → 0.012506681254206255
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_referencing.metrics.ape_3ax_p9973`: 0.020274859179592458 → 0.042860175442351704
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_referencing.metrics.momentum_peak`: 0.001068560795147859 → 0.00045187089728070573
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_referencing.metrics.power_mean`: 4.160364732880716 → 4.009894807188986
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_referencing.metrics.power_peak`: 4.870113418613782 → 5.602434349144837
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_referencing.metrics.rate_stability_p9973`: 0.03349010644246053 → 0.04421994730262022
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_referencing.metrics.sun_ape_p9973`: 0.020180313702076826 → 0.04279106332169133
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_referencing.objective`: 0.020180313702076826 → 0.04279106332169133
- `ais_img_3u.selection.families.mtq_fmr.modes.sun_referencing.violation.power_mean`: 1.0801823664403578 → 1.004947403594493
- `ais_img_3u.selection.families.mtq_fmr.shortfall`: 3127.813541 → 3130.08753
- `ais_img_3u.selection.families.mtq_fmr_rcs.budget.mass_kg`: 8.481181954025041 → 12.461681055464563
- `ais_img_3u.selection.families.mtq_fmr_rcs.budget.power_W`: 5.029764408616867 → 5.813282280603446
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_acquisition.metrics.power_mean`: 1.6395243760205616 → 1.6409696633142599
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_acquisition.metrics.power_peak`: 2.52337591525296 → 2.2637794018528057
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_acquisition.metrics.sun_angle_p95`: 0.4186339791031823 → 0.5879705618417329
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_referencing.metrics.ake_3ax_p9973`: 0.008806474484843237 → 0.012506681254206255
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_referencing.metrics.ape_3ax_p9973`: 0.020274859179592458 → 0.042860175442351704
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_referencing.metrics.momentum_peak`: 0.001068560795147859 → 0.00045187089728070573
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_referencing.metrics.power_mean`: 4.160364732880716 → 4.009894807188986
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_referencing.metrics.power_peak`: 4.870113418613782 → 5.602434349144837
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_referencing.metrics.rate_stability_p9973`: 0.03349010644246053 → 0.04421994730262022
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_referencing.metrics.sun_ape_p9973`: 0.020180313702076826 → 0.04279106332169133
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_referencing.objective`: 0.020180313702076826 → 0.04279106332169133
- `ais_img_3u.selection.families.mtq_fmr_rcs.modes.sun_referencing.violation.power_mean`: 1.0801823664403578 → 1.004947403594493
- `ais_img_3u.selection.families.mtq_fmr_rcs.shortfall`: 3128.456333 → 3130.730321
- `ais_img_3u.selection.iterations`: 8 → 9
- `mc:ake_los_p9973.max`: 0.7987380601425521 → 0.7668625784322405
- `mc:ake_los_p9973.mean`: 0.08325573643772624 → 0.10151455708046825
- `mc:ake_los_p9973.min`: 0.008668178458561627 → 0.011932864110640441
- `mc:ake_los_p9973.std`: 0.22577380464199592 → 0.2220673454586704
- `mc:ape_los_p9973.max`: 154.35563072130466 → 153.71954778836167
- `mc:ape_los_p9973.mean`: 84.61252844422118 → 73.83982559994429
- `mc:ape_los_p9973.min`: 24.79371695724766 → 31.013303970935183
- `mc:ape_los_p9973.std`: 43.409034139390265 → 38.60042342551431
- `mc:detumble_time.max`: 79.60833333333333 → 81.19166666666666

Each closure's answer, or the row that blocks it: `results/EVALUATION.md`. Each requirement's check: `results/TRACEABILITY.md`.
