# S7.19: what moved in the evaluation and the health map, and every new value against the engine

**In one line:** the baseline of the parity gate's evaluate check moved (results/evaluation.json, results/EVALUATION.md, results/health.json, results/HEALTH.md written again from the wired regression copy): every row, closure and node health that differs from the committed files is below, old -> new, with why, and every value evaluate now gives where it gave none is held to what the engine flies or computes for the case.

How it was made: the committed files (the baseline the gate held, as last written: 712 rows a case, from before the design gained the S7 nodes), d6aee43's evaluate and health on d6aee43's regression copy (what the earlier S7 steps changed, reason text only), and S7.19's on the wired copy (`python3 tools/evaluate.py tests/regression`, `python3 tools/health.py tests/regression`). The engine's values: `adcs params` from the design alone (an empty data folder) for a scenario of each case that overrides no tuning (ais_3u sun_mtq_ais, ais_img_3u detumble_rcs), their stored run manifests, `adcs size` for each case, and the design's data/stated.json.

## The evaluation

### ais_3u

Of the baseline's 712 rows, 144 changed (not computed -> computed: 11, not computed -> not computed: 119, not computed -> stated: 14); 510 rows are new to the baseline (nodes the design gained since it was made).

| Row | Old | Old why | New | New why | Why it moved |
|---|---|---|---|---|---|
| `act_cmg_axes` | not computed | needs nr (no input names it), rot_a0 (no input names it), rot_gi (no input names it), gim_axis (no input names it), delta (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs nr, rot_a0, rot_gi, gim_axis, delta are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `act_cmg_model` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs h, h0, k_speed, torque_max are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `act_cmg_steering` | not computed | needs tau (no input names it), a (no input names it), h (no input names it), nr (no input names it), ng (no input names it), rot_gi (no input names it), gim_axis (no input names it), gim_rate_max (no input names it), cmg_lam0 (no input names it), cmg_mu (no input names it), wheels (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs tau, a, h, nr, ng, rot_gi, gim_axis, gim_rate_max, cmg_lam0, cmg_mu, wheels are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `act_vscmg_gimbal_limits` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `act_vscmg_model` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs tm, fr, nz, k_c are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `cf_0` | not computed | a relation, but no pseudocode yet: its author writes it | stated 3 Count | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `cf_2` | not computed | a relation, but no pseudocode yet: its author writes it | stated 3 Count | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `cf_3` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0 Count | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `design_sizing_cmg` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_fmr` | not computed | no value, relation or pseudocode yet | not computed | its module has 8 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_mtq` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_rcs` | not computed | no value, relation or pseudocode yet | not computed | its module has 3 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_rw` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_sensors` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_vscmg` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `dyn_flexible_mode` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `dyn_kinematics` | not computed | needs a (no input names it), b (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs a, b are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `dyn_rigid_body` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `dyn_rotor_coupling` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `dyn_total_momentum` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs x, inertia, m, f are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `fdir_rotor_health` | not computed | needs st (no input names it), rp (no input names it), t (no input names it), dt (no input names it), zh (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs st, rp, t, dt, zh are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `fdir_safe_mode` | not computed | needs st (no input names it), mode (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs st, mode are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `fdir_sensor_health` | not computed | needs mag_ok0 (no input names it), b0 (no input names it), bref_ok (no input names it), bref (no input names it), b_good0 (no input names it), mag_age0 (no input names it), mag_seen0 (no input names it), has_gyro (no input names it), gyro_ok (no input names it), w0 (no input names it), gyro_age0 (no input names it), dt (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs mag_ok0, b0, bref_ok, bref, b_good0, mag_age0, mag_seen0, has_gyro, gyro_ok, w0, gyro_age0, dt are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gb_0` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gb_1` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gb_2` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gb_3` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gc_0` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.1 RadianPerSecond | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gc_1` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.707 One | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gc_2` | not computed | needs w_n from gc_0, zeta from gc_1 (a loop, or rows not computed) | computed 56.5771 (SI) | settling_time_2pct(w_n, zeta) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gd_3` | not computed | needs residual_dipole (no input names it), field (no input names it) | computed 4.6512e-07 (SI) | gd_3(residual_dipole, field) | S7.19: wired (revision S7.19), computed from its rows |
| `gd_4` | not computed | needs aerodynamic (no input names it), gravity_gradient (no input names it), solar (no input names it), magnetic (no input names it) | computed 5.47392e-07 (SI) | gd_4(aerodynamic, gravity_gradient, solar, magnetic) | S7.19: wired (revision S7.19), computed from its rows |
| `gd_5` | not computed | needs tau_d from gd_4 (not computed) | computed 0.00314148 (SI) | gd_5(tau_d, t_orb) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gdn_boresight_offset` | not computed | needs r (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs r are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gdn_guidance` | not computed | needs kind (no input names it), r (no input names it), v (no input names it), t (no input names it), q_off (no input names it), roll_deg (no input names it), t0 (no input names it), t_slew (no input names it), axis (no input names it), q_inertial (no input names it), sun_axis (no input names it), roll_axis (no input names it), sun_eci (no input names it), flip (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs kind, r, v, t, q_off, roll_deg, t0, t_slew, axis, q_inertial, sun_axis, roll_axis, sun_eci, flip are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gdn_mode_manager` | not computed | needs st (no input names it), mode (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs st, mode are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gdn_mode_schedule` | not computed | needs st (no input names it), mode (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs st, mode are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gdn_yaw_flip` | not computed | needs r (no input names it), v (no input names it), q_off (no input names it), sun_axis (no input names it), roll_axis (no input names it), sun_eci (no input names it), flip (no input names it), hyst (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs r, v, q_off, sun_axis, roll_axis, sun_eci, flip, hyst are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gf_0` | not computed | a relation, but no pseudocode yet: its author writes it | stated 3 Millimetre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_1` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.022 SquareMetre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_2` | not computed | a relation, but no pseudocode yet: its author writes it | stated 6440 KgPerCubicMetre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_3` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.0024 PascalSecond | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_4` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.2 MetrePerSecond | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_5` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.72 Metre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_6` | not computed | needs d from gf_0, s from gf_1, rho from gf_2, v from gf_4 (a loop, or rows not computed) | computed 0.000400591 (SI) | gf_6(d, s, rho, v) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gf_7` | not computed | needs d from gf_0, rho from gf_2, mu from gf_3 (a loop, or rows not computed) | computed 0.754688 (SI) | gf_7(d, rho, mu) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gf_8` | not computed | needs h from gf_6, mu from gf_3, l from gf_5, rho from gf_2, d from gf_0, s from gf_1 (a loop, or rows not computed) | computed 0.00173718 (SI) | gf_8(h, mu, l, rho, d, s) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gf_9` | not computed | needs tau from gw_4, l from gf_5, s from gf_1, d from gf_0 (a loop, or rows not computed) | not computed | needs tau from gw_4 (not computed) | S7.19: evaluate resolves each input in turn and names the one with no value (no longer 'a loop, or rows not computed') |
| `gm_0` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.45 AmpereSquareMetre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gm_1` | not computed | needs tau_d from gd_4 (not computed) | computed 0.0235377 (SI) | gm_1(tau_d, b_min) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gm_2` | not computed | needs momentum (no input names it), field (no input names it), dump_time (no input names it) | computed 0.0235377 (SI) | gm_2(momentum, field, dump_time) | S7.19: wired (revision S7.19), computed from its rows |
| `gm_3` | not computed | needs m_av from gm_0, n_mtq from cf_0 (a loop, or rows not computed) | computed 1.04652e-05 (SI) | gm_3(m_av, b_min, n_mtq) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gm_4` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gm_5` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gp_0` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim): its inputs ake are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gp_1` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim): its inputs ape, ake are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gp_2` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim): its inputs a are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gp_4` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gp_5` | not computed | needs terms (no input names it) | not computed | needs terms[1] from gp_0 (not computed), terms[2] from gp_1 (not computed), terms[3] from gp_2 (not computed), terms[4] from gp_3 (not computed), terms[5] from gp_4 (not computed) | S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `gr_1` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.8 Metre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gr_2` | not computed | a relation, but no pseudocode yet: its author writes it | stated 220 Second | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gr_3` | not computed | needs h from gw_3, r from gr_1, isp from gr_2 (a loop, or rows not computed) | not computed | needs h from gw_3 (not computed) | S7.19: evaluate resolves each input in turn and names the one with no value (no longer 'a loop, or rows not computed') |
| `gr_4` | not computed | needs m_p from gr_3, n_rcs from cf_3 (a loop, or rows not computed) | not computed | needs m_p from gr_3 (not computed) | S7.19: evaluate resolves each input in turn and names the one with no value (no longer 'a loop, or rows not computed') |
| `gw_2` | not computed | no value, relation or pseudocode yet | not computed | its module has 13 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gw_3` | not computed | needs t_slew from p2k_0 (a loop, or rows not computed) | not computed | needs t_slew from p2k_0 (not computed) | S7.19: evaluate resolves each input in turn and names the one with no value (no longer 'a loop, or rows not computed') |
| `gw_4` | not computed | needs t_slew from p2k_0 (a loop, or rows not computed) | not computed | needs t_slew from p2k_0 (not computed) | S7.19: evaluate resolves each input in turn and names the one with no value (no longer 'a loop, or rows not computed') |
| `gw_5` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gw_6` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_01` | not computed | no value, relation or pseudocode yet | not computed | its module has 13 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_02` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_03` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_04` | not computed | no value, relation or pseudocode yet | not computed | its module has 8 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_05` | not computed | no value, relation or pseudocode yet | not computed | its module has 3 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_06` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_07` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_08` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_dist_row_01` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs rm, r, j, mu are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs r, sun_rel, p_sun are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_05` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs r are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_06` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs rm, m_res, b_eci are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_07` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs h_m, scale are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_08` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_09` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_10` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs r_sat, r_sun are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_07` | not computed | no value, relation or pseudocode yet | not computed | its module has 4 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_fmr_row_08` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_09` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs hf, hm, tau, dt are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_10` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_11` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_12` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 5 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_13` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs a, sigma, disp are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_14` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_fmr_row_15` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_mtq_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs x0, u, tau, dt are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs a, m_max, p_max are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_05` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_06` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs m, b are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_12` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_13` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_oils_row_07` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 11 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_pnt_row_09` | not computed | no value, relation or pseudocode yet | not computed | needs ape (the run's), e_a from gp_2 (not computed), e_t from gp_3 (not computed), e_j from gp_4 (not computed) | earlier S7 steps: the reason's text; S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `l3_rcs_row_01` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs sigma, disp are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs duty, t, mib, res are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_08` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs used, mdot, dt, prop_kg are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_09` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs on, power are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_10` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs tau, sigma, disp are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_11` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_01` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 4 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_05` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs share, torque_max, g are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_06` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs a, sigma, disp are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_07` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs p_steady, tm, om, eta are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_01` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_05` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_06` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_07` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_08` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs i, p are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_09` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_10` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 6 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_11` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 6 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_12` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_13` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_14` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 4 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `m2_7` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs nu, n are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_gnss_fix` | not computed | needs th (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs th are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_gyro_filter` | not computed | needs th (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs th are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_onboard_orbit` | not computed | needs r (no input names it), mu (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs r, mu are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_quest` | not computed | needs bv (no input names it), rv (no input names it), w (no input names it), n (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs bv, rv, w, n are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_time_frames` | not computed | needs jd (no input names it) | not computed | stated, with no value: its method is env_time_frames's since S7.3 (fsw/pseudocode/02_time_frames_models.pc, one copy); the pseudocode it carried from 1.0.0 stays as provenance | S7.19: a stated block keeps 1.0.0's pseudocode as provenance; evaluate runs methods only |
| `nav_triad` | not computed | needs r (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs r are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `rk4_0` | not computed | needs levels (no input names it) | not computed | needs levels[1] from rk3_0 (not computed), levels[2] from rk3_1 (not computed), levels[3] from rk3_2 (not computed), levels[4] from rk3_3 (not computed), levels[5] from rk3_4 (not computed), levels[6] from rk3_5 (not computed), levels[7] from rk3_6 (not computed) | S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `rk4_1` | not computed | needs closed (no input names it), opened (no input names it) | not computed | needs closed from rk1_4 (not computed), opened from rk1_3 (not computed) | S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `rk4_2` | not computed | needs untested (no input names it), total (no input names it) | not computed | needs untested from rk2_2 (not computed), total from rk2_3 (not computed) | S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `s1_4` | not computed | the case does not state mass.cm | computed [0.00599251, 0.0139825, -0.0129838] (SI) | cm_offset(cpa, dir) | earlier S7 steps: the reason's text; S7.19: wired (revision S7.19), computed from its rows |

Rows new to the baseline, by state (each with its value or why in results/EVALUATION.md):

| State | Rows |
|---|---|
| computed | 32: `dyn_truth_plant`, `env_case_orbit`, `fsw_param_J`, `fsw_param_bdot_k`, `fsw_param_detumble_exit`, `fsw_param_gd_q_off`, `fsw_param_jd0`, `fsw_param_m_res_est`, `fsw_param_mtq_Gs`, `fsw_param_mtq_Kd`, `fsw_param_mtq_Ki`, `fsw_param_mtq_Kp`, `fsw_param_mtq_Pth`, `fsw_param_mtq_Pw`, `fsw_param_mtq_gg_ff`, `fsw_param_mtq_lambda`, `fsw_param_mu`, `fsw_param_roll_axis`, `fsw_param_rw_Gs`, `fsw_param_rw_Kd`, `fsw_param_rw_Ki`, `fsw_param_rw_Kp`, `fsw_param_rw_dt`, `fsw_param_rw_lambda`, `fsw_param_sb_kdroll`, `fsw_param_sb_kroll`, `fsw_param_sb_roll_gate`, `fsw_param_ss_dr_k`, `fsw_param_ss_dr_k2`, `fsw_param_ss_k1`, `fsw_param_ss_k2`, `fsw_param_sun_axis` |
| not computed: computed during a run | 86: `act_rotor_set`, `act_rotor_telemetry`, `dyn_initial_state`, `env_calendar_time`, `env_de440`, `env_density_model`, `env_drag_force`, `env_dtm2020_operational`, `env_dtm2020_research`, `env_earth_frames`, `env_erp_force`, `env_exponential_atmosphere`, `env_force_model`, `env_gas_surface`, `env_geodetic`, `env_gravity_field`, `env_iau2006`, `env_jb2008`, `env_moon_fast`, `env_ocean_tides`, `env_orbit_fast`, `env_orbit_start`, `env_relativity`, `env_solid_tides`, `env_space_weather`, `env_srp_force`, `env_third_body`, `env_tidal_eop`, `env_time_frames`, `env_time_scales`, `env_two_body_elements`, `fsw_allocation`, `fsw_control`, `fsw_drivers`, `fsw_estimation`, `fsw_guidance`, `fsw_modes`, `fsw_param_alloc`, `fsw_param_bdot_law`, `fsw_param_es_noise`, `fsw_param_gim_axis`, `fsw_param_gim_rate_max`, `fsw_param_gps_latency`, `fsw_param_gyro_arw`, `fsw_param_gyro_rrw`, `fsw_param_has_es`, `fsw_param_has_gps`, `fsw_param_has_gyro`, `fsw_param_has_st`, `fsw_param_has_sun`, `fsw_param_m_max`, `fsw_param_mekf_mag_err_T`, `fsw_param_mekf_sig_mag`, `fsw_param_mekf_sig_sun`, `fsw_param_mtq_law`, `fsw_param_n_heads`, `fsw_param_nc`, `fsw_param_ng`, `fsw_param_nr`, `fsw_param_rcs_mib`, `fsw_param_rcs_res`, `fsw_param_rcs_tau`, `fsw_param_rot_a0`, `fsw_param_rot_gi`, `fsw_param_rot_h0`, `fsw_param_rot_hmax`, `fsw_param_rot_kind`, `fsw_param_rot_tmax`, `fsw_param_rw_law`, `fsw_param_ss_eclipse`, `fsw_param_ss_law`, `fsw_param_ss_rz_floor`, `fsw_param_st_bs`, `fsw_param_st_latency`, `fsw_param_st_noise_cross`, `fsw_param_st_noise_roll`, `fsw_steplaws`, `kpi_metric_channels`, `kpi_metric_ecss`, `kpi_metric_evaluate`, `kpi_metric_statistics`, `sens_fine_sun`, `sens_sky_view`, `sens_star_catalogue`, `sens_star_image`, `sens_star_tracker` |
| not computed: its module has 10 functions and names none for this node (by its id or its output's symbol) | 1: `fsw_param_mtq_Klqr` |
| not computed: its module has 4 functions and names none for this node (by its id or its output's symbol) | 1: `fsw_param_rw_Klqr` |
| not computed: its module has 6 functions and names none for this node (by its id or its output's symbol) | 2: `catalogue_datasheet_derive`, `design_power_system` |
| not computed: its module has 8 functions and names none for this node (by its id or its output's symbol) | 1: `ctl_floquet_certificate` |
| not computed: its module has 9 functions and names none for this node (by its id or its output's symbol) | 3: `design_loop_converge`, `design_loop_redundancy`, `design_loop_robustness` |
| not computed: needs a row with no value | 6: `fsw_param_mtq_k1`, `fsw_param_mtq_k16`, `fsw_param_mtq_k2`, `fsw_param_mtq_lam16`, `fsw_param_sb_kd`, `fsw_param_sb_kp` |
| not computed: no value, relation or pseudocode yet | 248: `act__cmg`, `act__vscmg`, `cas`, `cat`, `cf`, `ci1`, `cm1`, `cm2`, `cm3`, `cmr`, `cpt`, `ct1`, `ct2`, `ct3`, `design__sizing_cmg`, `design__sizing_fmr`, `design__sizing_mtq`, `design__sizing_rcs`, `design__sizing_rw`, `design__sizing_sens`, `design__sizing_vscmg`, `dyn__carried`, `env_de440_slice`, `env_dtm2020_coefficients`, `env_gravity_default_field`, `env_igrf13_coefficients`, `env_jb2008_indices`, `env_kp_ap_table`, `env_leap_seconds`, `env_ocean_tide_tables`, `env_tidal_eop_terms`, `env_xys06_series`, `fa1`, `fa2`, `fa3`, `fa4`, `fa5`, `fa6`, `fa7`, `fa8`, `fac`, `fdir__carried`, `fsw_param_n_sched`, `fsw_param_sched_mode`, `fsw_param_sched_t`, `gb`, `gc`, `gd`, `gdn__carried`, `ge`, `gf`, `gm`, `gp`, `gq`, `gr`, `gs`, `gw`, `gx`, `hr1`, `hrt`, `l3_budget`, `l3_ctl`, `l3_dist`, `l3_est`, `l3_fmr`, `l3_fsw`, `l3_hils`, `l3_modes`, `l3_mtq`, `l3_oils`, `l3_pnt`, `l3_rcs`, `l3_rw`, `l3_sens`, `lib_catalogue_algorithms_bdot`, `lib_catalogue_algorithms_bdot_bangbang`, `lib_catalogue_algorithms_bdot_gyro`, `lib_catalogue_algorithms_bdot_mag`, `lib_catalogue_algorithms_cmg_sr`, `lib_catalogue_algorithms_genbdot_l1`, `lib_catalogue_algorithms_idmas_split`, `lib_catalogue_algorithms_lqr`, `lib_catalogue_algorithms_mekf`, `lib_catalogue_algorithms_mtq_avanzini2021`, `lib_catalogue_algorithms_mtq_celani2015`, `lib_catalogue_algorithms_mtq_celani2026`, `lib_catalogue_algorithms_mtq_lovera2004`, `lib_catalogue_algorithms_mtq_lqr`, `lib_catalogue_algorithms_mtq_pd`, `lib_catalogue_algorithms_mtq_rate_damp`, `lib_catalogue_algorithms_mtq_smc`, `lib_catalogue_algorithms_mtq_tango2013`, `lib_catalogue_algorithms_pd_alloc`, `lib_catalogue_algorithms_pid`, `lib_catalogue_algorithms_rcs_pwm`, `lib_catalogue_algorithms_rcs_rate`, `lib_catalogue_algorithms_rotor_pinv`, `lib_catalogue_algorithms_smc`, `lib_catalogue_algorithms_sun_acq_rotor`, `lib_catalogue_algorithms_sun_boresight_celani2026`, `lib_catalogue_algorithms_sunspin_damped`, `lib_catalogue_algorithms_sunspin_deruiter2011`, `lib_catalogue_algorithms_sunspin_l1l2`, `lib_catalogue_algorithms_sunspin_l1l2_e2`, `lib_catalogue_algorithms_vscmg_sr`, `lib_catalogue_classes`, `lib_catalogue_components_cmg`, `lib_catalogue_components_coarse_sun_sensor`, `lib_catalogue_components_earth_sensor`, `lib_catalogue_components_fluid_loop`, `lib_catalogue_components_gnss`, `lib_catalogue_components_gyro`, `lib_catalogue_components_magnetometer`, `lib_catalogue_components_magnetorquer`, `lib_catalogue_components_rcs`, `lib_catalogue_components_reaction_wheel`, `lib_catalogue_components_star_tracker`, `lib_catalogue_components_sun_sensor`, `lib_catalogue_components_vscmg`, `lib_catalogue_dispersions`, `lib_catalogue_families`, `lib_catalogue_modes_detumble`, `lib_catalogue_modes_nadir_pointing`, `lib_catalogue_modes_sun_acquisition`, `lib_catalogue_modes_sun_referencing`, `lib_catalogue_parts_syn_ct_1`, `lib_catalogue_parts_syn_es_1`, `lib_catalogue_parts_syn_gyro_1`, `lib_catalogue_parts_syn_mag_1`, `lib_catalogue_parts_syn_mfp_1`, `lib_catalogue_parts_syn_rw_10`, `lib_catalogue_parts_syn_st_1`, `lib_catalogue_parts_syn_sun_1`, `lib_catalogue_parts_trn_cmg_1`, `lib_catalogue_parts_trn_css_1`, `lib_catalogue_parts_trn_gps_1`, `lib_catalogue_parts_trn_gyro_p1`, `lib_catalogue_parts_trn_rcs_3u`, `lib_catalogue_parts_trn_vscmg_1`, `lib_catalogue_products_trn_p_3u_ais`, `lib_catalogue_products_trn_p_3u_ais_css`, `lib_catalogue_products_trn_p_3u_cmg`, `lib_catalogue_products_trn_p_3u_fmr`, `lib_catalogue_products_trn_p_3u_fmr_rcs`, `lib_catalogue_products_trn_p_3u_img`, `lib_catalogue_products_trn_p_3u_rw_rcs`, `lib_catalogue_products_trn_p_3u_vscmg`, `lib_fsw_params_params`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw222_3`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw222_6`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw400_15`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw400_30`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw400_50`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw0017`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw0057`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw0162`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw0500`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw1200`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw2500`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw5000`, `lib_matlab_sils_data_catalogue_rocket_lab_rw3_0_06`, `lib_matlab_sils_data_catalogue_rocket_lab_rw3_1_0`, `lib_matlab_sils_data_catalogue_rocket_lab_rw_0_003`, `lib_matlab_sils_data_catalogue_rocket_lab_rw_0_01`, `lib_matlab_sils_data_catalogue_rocket_lab_rw_0_03`, `lib_matlab_sils_data_catalogue_tensor_tech_adcs400`, `lib_matlab_sils_data_catalogue_tensor_tech_tensorcmg_10m`, `lib_matlab_sils_data_pipeline_nodes`, `lib_matlab_sils_data_scenario_schema`, `lib_spec_physics_constants`, `lib_spec_physics_ctl`, `lib_spec_physics_env`, `lib_spec_physics_fmr`, `lib_spec_physics_gnc`, `lib_spec_physics_mission`, `lib_spec_physics_mtq`, `lib_spec_physics_orbit`, `lib_spec_physics_rcs`, `lib_spec_physics_risk`, `lib_spec_physics_rw`, `lib_spec_plan_case_inputs`, `lib_spec_plan_case_template`, `lib_spec_plan_expected_node_ids`, `lib_spec_plan_kpis`, `lib_spec_plan_physics`, `lib_spec_plan_seed_content`, `lib_spec_plan_tree`, `lib_spec_plan_twin_map`, `lib_spec_plan_units`, `m1`, `m2`, `m3`, `mgm`, `msn`, `nav__carried`, `od1`, `od2`, `ord`, `p1`, `p1a`, `p1k`, `p2`, `p2a`, `p2k`, `p3`, `p3a`, `p3k`, `p4`, `p4a`, `p4k`, `prg`, `programme_delivery_waves`, `rk1`, `rk2`, `rk3`, `rk4`, `rsk`, `s1`, `s2`, `s3`, `s4`, `s5`, `sat`, `sb0`, `sb1`, `sb2`, `sb3`, `sb4`, `st1`, `st2`, `st3`, `st4`, `st5`, `st6`, `st7`, `std`, `su1`, `su2`, `su3`, `sub`, `sup`, `svc`, `v1`, `v2`, `v3`, `v4`, `ver`, `x_closure` |
| stated | 130: `act_cmg_speed_gain`, `act_fmr_flow_gain`, `act_fmr_flow_tau`, `act_gimbal_tlm_noise`, `act_rotor_tlm_noise`, `act_rw_drive_efficiency`, `act_rw_friction_comp`, `act_rw_torque_noise`, `dyn_cm_direction_x`, `dyn_cm_direction_y`, `dyn_cm_direction_z`, `dyn_initial_attitude_kind_default`, `dyn_initial_error_angle`, `dyn_initial_error_axis`, `dyn_initial_rate_extra`, `dyn_initial_rate_kind_default`, `dyn_initial_rate_magnitude`, `dyn_initial_rate_value`, `dyn_surface_accommodation`, `dyn_surface_specular_share`, `dyn_surface_vb_ratio`, `env_ap_default`, `env_density_scale_default`, `env_f107_default`, `env_f107a_default`, `env_fast_zonal_degree`, `env_field_degree`, `env_force_drag`, `env_force_srp`, `env_force_third_body`, `env_kp_default`, `env_orbit_precision_default`, `env_orbit_step`, `env_refresh_step`, `env_start_arg_lat_default`, `env_torque_aero`, `env_torque_gravity_gradient`, `env_torque_magnetic`, `env_torque_radiation`, `fsw_param_auto_next`, `fsw_param_capture_deg`, `fsw_param_capture_rate_deg_s`, `fsw_param_cmg_k_null`, `fsw_param_cmg_lam0`, `fsw_param_cmg_mu`, `fsw_param_detumble_hold_s`, `fsw_param_dt`, `fsw_param_dump_k`, `fsw_param_fdir_h_frac`, `fsw_param_fdir_s`, `fsw_param_fdir_win_s`, `fsw_param_gd_T`, `fsw_param_gd_axis`, `fsw_param_gd_flip_hyst`, `fsw_param_gd_kind`, `fsw_param_gd_q_inertial`, `fsw_param_gd_roll_deg`, `fsw_param_gd_t0`, `fsw_param_gd_yaw_flip`, `fsw_param_gnss_ecef`, `fsw_param_h_bias`, `fsw_param_ho_hold_s`, `fsw_param_ho_in_dps`, `fsw_param_ho_out_dps`, `fsw_param_igrf_nmax`, `fsw_param_mekf_gate`, `fsw_param_mekf_meas_scale`, `fsw_param_mekf_rej_max`, `fsw_param_mtq_eps`, `fsw_param_mtq_err_max`, `fsw_param_mtq_int_max`, `fsw_param_mtq_meas`, `fsw_param_mtq_period`, `fsw_param_mtq_phi`, `fsw_param_rate_lpf_s`, `fsw_param_rcs_assist`, `fsw_param_rcs_assist_frac`, `fsw_param_rcs_dump`, `fsw_param_rcs_dump_hi`, `fsw_param_rcs_dump_k`, `fsw_param_rcs_dump_lo`, `fsw_param_rcsd_T_damp_s`, `fsw_param_rcsd_deadband_deg_s`, `fsw_param_rcsd_period_s`, `fsw_param_rw_err_max`, `fsw_param_rw_int_max`, `fsw_param_rw_phi`, `fsw_param_sa_done_deg`, `fsw_param_sa_done_hold_s`, `fsw_param_sa_kd`, `fsw_param_sa_w_max_deg_s`, `fsw_param_ss_dr_k1`, `fsw_param_ss_dwell_in_s`, `fsw_param_ss_dwell_out_s`, `fsw_param_ss_k_l1`, `fsw_param_ss_omega_exit_dps`, `fsw_param_ss_omega_max_dps`, `fsw_param_ss_perp_in_dps`, `fsw_param_ss_perp_out_dps`, `fsw_param_ss_sigma0`, `fsw_param_ss_spin_dps`, `fsw_param_ss_sun_min`, `fsw_param_ss_t_check_s`, `fsw_param_ss_z_in_dps`, `fsw_param_st_coast_s`, `fsw_param_start_mode`, `fsw_tune_avanzini_k_over_n`, `fsw_tune_avanzini_lambda`, `fsw_tune_bdot_gain_scale`, `fsw_tune_detumble_exit_deg_s`, `fsw_tune_mtq_gain_d`, `fsw_tune_mtq_gain_p`, `fsw_tune_mtq_wn`, `fsw_tune_mtq_zeta`, `fsw_tune_roll_gain`, `fsw_tune_roll_gate_deg`, `fsw_tune_roll_wn_orbits`, `fsw_tune_roll_zeta`, `fsw_tune_rw_bandwidth`, `fsw_tune_rw_damping`, `fsw_tune_rw_rate_hz`, `fsw_tune_ss_gain`, `gdn_payload_boresight_x`, `gdn_payload_boresight_y`, `gdn_payload_boresight_z`, `gdn_sun_axis_x`, `gdn_sun_axis_y`, `gdn_sun_axis_z`, `vv_record_step_default`, `vv_run_duration_default` |

Closures that moved:

| Closure | Old | Old why | New | New why |
|---|---|---|---|---|
| `kpi_absolute_pointing_error_ape_analysis` | blocked | gp_5 has no value (needs terms (no input names it)) | **blocked** | gp_5 has no value (needs terms[1] from gp_0 (not computed), terms[2] from gp_1 (not computed), terms[3] from gp_2 (not computed), terms[4] from gp_3 (not computed), terms[5] from gp_4 (not computed)) |
| `kpi_adcs_mass_analysis` | blocked | gb_0 has no value (no value, relation or pseudocode yet) | **blocked** | gb_0 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_orbit_average_power_analysis` | blocked | gb_1 has no value (no value, relation or pseudocode yet) | **blocked** | gb_1 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_peak_power_analysis` | blocked | gb_2 has no value (no value, relation or pseudocode yet) | **blocked** | gb_2 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_volume_analysis` | blocked | gb_3 has no value (no value, relation or pseudocode yet) | **blocked** | gb_3 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |

### ais_img_3u

Of the baseline's 712 rows, 142 changed (not computed -> computed: 14, not computed -> not computed: 114, not computed -> stated: 14); 510 rows are new to the baseline (nodes the design gained since it was made).

| Row | Old | Old why | New | New why | Why it moved |
|---|---|---|---|---|---|
| `act_cmg_axes` | not computed | needs nr (no input names it), rot_a0 (no input names it), rot_gi (no input names it), gim_axis (no input names it), delta (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs nr, rot_a0, rot_gi, gim_axis, delta are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `act_cmg_model` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs h, h0, k_speed, torque_max are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `act_cmg_steering` | not computed | needs tau (no input names it), a (no input names it), h (no input names it), nr (no input names it), ng (no input names it), rot_gi (no input names it), gim_axis (no input names it), gim_rate_max (no input names it), cmg_lam0 (no input names it), cmg_mu (no input names it), wheels (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs tau, a, h, nr, ng, rot_gi, gim_axis, gim_rate_max, cmg_lam0, cmg_mu, wheels are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `act_vscmg_gimbal_limits` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `act_vscmg_model` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs tm, fr, nz, k_c are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `cf_0` | not computed | a relation, but no pseudocode yet: its author writes it | stated 3 Count | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `cf_2` | not computed | a relation, but no pseudocode yet: its author writes it | stated 3 Count | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `cf_3` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0 Count | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `design_sizing_cmg` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_fmr` | not computed | no value, relation or pseudocode yet | not computed | its module has 8 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_mtq` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_rcs` | not computed | no value, relation or pseudocode yet | not computed | its module has 3 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_rw` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_sensors` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `design_sizing_vscmg` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `dyn_flexible_mode` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `dyn_kinematics` | not computed | needs a (no input names it), b (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs a, b are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `dyn_rigid_body` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `dyn_rotor_coupling` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `dyn_total_momentum` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs x, inertia, m, f are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `fdir_rotor_health` | not computed | needs st (no input names it), rp (no input names it), t (no input names it), dt (no input names it), zh (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs st, rp, t, dt, zh are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `fdir_safe_mode` | not computed | needs st (no input names it), mode (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs st, mode are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `fdir_sensor_health` | not computed | needs mag_ok0 (no input names it), b0 (no input names it), bref_ok (no input names it), bref (no input names it), b_good0 (no input names it), mag_age0 (no input names it), mag_seen0 (no input names it), has_gyro (no input names it), gyro_ok (no input names it), w0 (no input names it), gyro_age0 (no input names it), dt (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs mag_ok0, b0, bref_ok, bref, b_good0, mag_age0, mag_seen0, has_gyro, gyro_ok, w0, gyro_age0, dt are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gb_0` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gb_1` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gb_2` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gb_3` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gc_0` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.1 RadianPerSecond | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gc_1` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.707 One | the design, adcs_ref_c1 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gc_2` | not computed | needs w_n from gc_0, zeta from gc_1 (a loop, or rows not computed) | computed 56.5771 (SI) | settling_time_2pct(w_n, zeta) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gd_3` | not computed | needs residual_dipole (no input names it), field (no input names it) | computed 4.6512e-07 (SI) | gd_3(residual_dipole, field) | S7.19: wired (revision S7.19), computed from its rows |
| `gd_4` | not computed | needs aerodynamic (no input names it), gravity_gradient (no input names it), solar (no input names it), magnetic (no input names it) | computed 5.47392e-07 (SI) | gd_4(aerodynamic, gravity_gradient, solar, magnetic) | S7.19: wired (revision S7.19), computed from its rows |
| `gd_5` | not computed | needs tau_d from gd_4 (not computed) | computed 0.00314148 (SI) | gd_5(tau_d, t_orb) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gdn_boresight_offset` | not computed | needs r (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs r are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gdn_guidance` | not computed | needs kind (no input names it), r (no input names it), v (no input names it), t (no input names it), q_off (no input names it), roll_deg (no input names it), t0 (no input names it), t_slew (no input names it), axis (no input names it), q_inertial (no input names it), sun_axis (no input names it), roll_axis (no input names it), sun_eci (no input names it), flip (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs kind, r, v, t, q_off, roll_deg, t0, t_slew, axis, q_inertial, sun_axis, roll_axis, sun_eci, flip are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gdn_mode_manager` | not computed | needs st (no input names it), mode (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs st, mode are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gdn_mode_schedule` | not computed | needs st (no input names it), mode (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs st, mode are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gdn_yaw_flip` | not computed | needs r (no input names it), v (no input names it), q_off (no input names it), sun_axis (no input names it), roll_axis (no input names it), sun_eci (no input names it), flip (no input names it), hyst (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs r, v, q_off, sun_axis, roll_axis, sun_eci, flip, hyst are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gf_0` | not computed | a relation, but no pseudocode yet: its author writes it | stated 3 Millimetre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_1` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.022 SquareMetre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_2` | not computed | a relation, but no pseudocode yet: its author writes it | stated 6440 KgPerCubicMetre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_3` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.0024 PascalSecond | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_4` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.2 MetrePerSecond | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_5` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.72 Metre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gf_6` | not computed | needs d from gf_0, s from gf_1, rho from gf_2, v from gf_4 (a loop, or rows not computed) | computed 0.000400591 (SI) | gf_6(d, s, rho, v) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gf_7` | not computed | needs d from gf_0, rho from gf_2, mu from gf_3 (a loop, or rows not computed) | computed 0.754688 (SI) | gf_7(d, rho, mu) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gf_8` | not computed | needs h from gf_6, mu from gf_3, l from gf_5, rho from gf_2, d from gf_0, s from gf_1 (a loop, or rows not computed) | computed 0.00173718 (SI) | gf_8(h, mu, l, rho, d, s) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gf_9` | not computed | needs l from gf_5, s from gf_1, d from gf_0 (a loop, or rows not computed) | computed 56.5657 (SI) | pump_pressure_for_torque(tau, l, s, d) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gm_0` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.45 AmpereSquareMetre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gm_1` | not computed | needs tau_d from gd_4 (not computed) | computed 0.0235377 (SI) | gm_1(tau_d, b_min) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gm_2` | not computed | needs momentum (no input names it), field (no input names it), dump_time (no input names it) | computed 0.0235377 (SI) | gm_2(momentum, field, dump_time) | S7.19: wired (revision S7.19), computed from its rows |
| `gm_3` | not computed | needs m_av from gm_0, n_mtq from cf_0 (a loop, or rows not computed) | computed 1.04652e-05 (SI) | gm_3(m_av, b_min, n_mtq) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gm_4` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gm_5` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gp_0` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim): its inputs ake are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gp_1` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim): its inputs ape, ake are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gp_2` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim): its inputs a are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `gp_4` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gp_5` | not computed | needs terms (no input names it) | not computed | needs terms[1] from gp_0 (not computed), terms[2] from gp_1 (not computed), terms[3] from gp_2 (not computed), terms[4] from gp_3 (not computed), terms[5] from gp_4 (not computed) | S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `gr_1` | not computed | a relation, but no pseudocode yet: its author writes it | stated 0.8 Metre | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gr_2` | not computed | a relation, but no pseudocode yet: its author writes it | stated 220 Second | the design, idmas_v2 | S7.19: evaluate reads the value the design states (value.number in its output's unit, value.list), S7.13's finding |
| `gr_3` | not computed | needs r from gr_1, isp from gr_2 (a loop, or rows not computed) | computed 8.49422e-07 (SI) | propellant_per_slew(h, r, isp) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gr_4` | not computed | needs m_p from gr_3, n_rcs from cf_3 (a loop, or rows not computed) | computed 0 (SI) | propellant_per_year(m_p, n_day, n_rcs) | S7.19: its inputs now have values (the design's stated values, or rows the wiring computes) |
| `gw_2` | not computed | no value, relation or pseudocode yet | not computed | its module has 13 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gw_5` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `gw_6` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_01` | not computed | no value, relation or pseudocode yet | not computed | its module has 13 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_02` | not computed | no value, relation or pseudocode yet | not computed | its module has 5 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_03` | not computed | no value, relation or pseudocode yet | not computed | its module has 7 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_04` | not computed | no value, relation or pseudocode yet | not computed | its module has 8 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_05` | not computed | no value, relation or pseudocode yet | not computed | its module has 3 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_06` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_07` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_budget_row_08` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_dist_row_01` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs rm, r, j, mu are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs r, sun_rel, p_sun are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_05` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs r are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_06` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs rm, m_res, b_eci are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_07` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs h_m, scale are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_08` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_09` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_dist_row_10` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs r_sat, r_sun are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_07` | not computed | no value, relation or pseudocode yet | not computed | its module has 4 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_fmr_row_08` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_09` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs hf, hm, tau, dt are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_10` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_11` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_12` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 5 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_13` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs a, sigma, disp are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_fmr_row_14` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_fmr_row_15` | not computed | no value, relation or pseudocode yet | not computed | its module has 2 functions and names none for this node (by its id or its output's symbol) | earlier S7 steps (S7.3 to S7.15b): the reason's text only, as each step recorded, the baseline kept |
| `l3_mtq_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs x0, u, tau, dt are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs a, m_max, p_max are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_05` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_06` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs m, b are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_12` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_mtq_row_13` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_oils_row_07` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 11 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_pnt_row_09` | not computed | no value, relation or pseudocode yet | not computed | needs ape (the run's), e_a from gp_2 (not computed), e_t from gp_3 (not computed), e_j from gp_4 (not computed) | earlier S7 steps: the reason's text; S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `l3_rcs_row_01` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs sigma, disp are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs duty, t, mib, res are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_08` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs used, mdot, dt, prop_kg are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_09` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs on, power are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_10` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs tau, sigma, disp are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rcs_row_11` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_01` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 4 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_05` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs share, torque_max, g are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_06` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs a, sigma, disp are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_rw_row_07` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs p_steady, tm, om, eta are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_01` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_02` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_03` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_04` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_05` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_06` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_07` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_08` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs i, p are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_09` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_10` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 6 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_11` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 6 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_12` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 2 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_13` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 3 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `l3_sens_row_14` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its module's 4 functions take the run's state and the product it flies, not rows of the design, so it has no one value | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `m2_7` | not computed | no value, relation or pseudocode yet | not computed | computed during a run (generated into adcs-sim-core): its inputs nu, n are the run's state and the product it flies, not rows of the design | earlier S7 steps: the reason's text; S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_gnss_fix` | not computed | needs th (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs th are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_gyro_filter` | not computed | needs th (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs th are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_onboard_orbit` | not computed | needs r (no input names it), mu (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs r, mu are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_quest` | not computed | needs bv (no input names it), rv (no input names it), w (no input names it), n (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs bv, rv, w, n are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `nav_time_frames` | not computed | needs jd (no input names it) | not computed | stated, with no value: its method is env_time_frames's since S7.3 (fsw/pseudocode/02_time_frames_models.pc, one copy); the pseudocode it carried from 1.0.0 stays as provenance | S7.19: a stated block keeps 1.0.0's pseudocode as provenance; evaluate runs methods only |
| `nav_triad` | not computed | needs r (no input names it) | not computed | computed during a run (by the flight software or the engine, at each step): its inputs r are the run's state and the product it flies, not rows of the design | S7.19: its inputs declared the run's (code.run_inputs, revision S7.19) |
| `rk4_0` | not computed | needs levels (no input names it) | not computed | needs levels[1] from rk3_0 (not computed), levels[2] from rk3_1 (not computed), levels[3] from rk3_2 (not computed), levels[4] from rk3_3 (not computed), levels[5] from rk3_4 (not computed), levels[6] from rk3_5 (not computed), levels[7] from rk3_6 (not computed) | S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `rk4_1` | not computed | needs closed (no input names it), opened (no input names it) | not computed | needs closed from rk1_4 (not computed), opened from rk1_3 (not computed) | S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `rk4_2` | not computed | needs untested (no input names it), total (no input names it) | not computed | needs untested from rk2_2 (not computed), total from rk2_3 (not computed) | S7.19: wired (revision S7.19); it still needs a row with no value, which it names |
| `s1_4` | not computed | the case does not state mass.cm | computed [0.00599251, 0.0139825, -0.0129838] (SI) | cm_offset(cpa, dir) | earlier S7 steps: the reason's text; S7.19: wired (revision S7.19), computed from its rows |

Rows new to the baseline, by state (each with its value or why in results/EVALUATION.md):

| State | Rows |
|---|---|
| computed | 32: `dyn_truth_plant`, `env_case_orbit`, `fsw_param_J`, `fsw_param_bdot_k`, `fsw_param_detumble_exit`, `fsw_param_gd_q_off`, `fsw_param_jd0`, `fsw_param_m_res_est`, `fsw_param_mtq_Gs`, `fsw_param_mtq_Kd`, `fsw_param_mtq_Ki`, `fsw_param_mtq_Kp`, `fsw_param_mtq_Pth`, `fsw_param_mtq_Pw`, `fsw_param_mtq_gg_ff`, `fsw_param_mtq_lambda`, `fsw_param_mu`, `fsw_param_roll_axis`, `fsw_param_rw_Gs`, `fsw_param_rw_Kd`, `fsw_param_rw_Ki`, `fsw_param_rw_Kp`, `fsw_param_rw_dt`, `fsw_param_rw_lambda`, `fsw_param_sb_kdroll`, `fsw_param_sb_kroll`, `fsw_param_sb_roll_gate`, `fsw_param_ss_dr_k`, `fsw_param_ss_dr_k2`, `fsw_param_ss_k1`, `fsw_param_ss_k2`, `fsw_param_sun_axis` |
| not computed: computed during a run | 86: `act_rotor_set`, `act_rotor_telemetry`, `dyn_initial_state`, `env_calendar_time`, `env_de440`, `env_density_model`, `env_drag_force`, `env_dtm2020_operational`, `env_dtm2020_research`, `env_earth_frames`, `env_erp_force`, `env_exponential_atmosphere`, `env_force_model`, `env_gas_surface`, `env_geodetic`, `env_gravity_field`, `env_iau2006`, `env_jb2008`, `env_moon_fast`, `env_ocean_tides`, `env_orbit_fast`, `env_orbit_start`, `env_relativity`, `env_solid_tides`, `env_space_weather`, `env_srp_force`, `env_third_body`, `env_tidal_eop`, `env_time_frames`, `env_time_scales`, `env_two_body_elements`, `fsw_allocation`, `fsw_control`, `fsw_drivers`, `fsw_estimation`, `fsw_guidance`, `fsw_modes`, `fsw_param_alloc`, `fsw_param_bdot_law`, `fsw_param_es_noise`, `fsw_param_gim_axis`, `fsw_param_gim_rate_max`, `fsw_param_gps_latency`, `fsw_param_gyro_arw`, `fsw_param_gyro_rrw`, `fsw_param_has_es`, `fsw_param_has_gps`, `fsw_param_has_gyro`, `fsw_param_has_st`, `fsw_param_has_sun`, `fsw_param_m_max`, `fsw_param_mekf_mag_err_T`, `fsw_param_mekf_sig_mag`, `fsw_param_mekf_sig_sun`, `fsw_param_mtq_law`, `fsw_param_n_heads`, `fsw_param_nc`, `fsw_param_ng`, `fsw_param_nr`, `fsw_param_rcs_mib`, `fsw_param_rcs_res`, `fsw_param_rcs_tau`, `fsw_param_rot_a0`, `fsw_param_rot_gi`, `fsw_param_rot_h0`, `fsw_param_rot_hmax`, `fsw_param_rot_kind`, `fsw_param_rot_tmax`, `fsw_param_rw_law`, `fsw_param_ss_eclipse`, `fsw_param_ss_law`, `fsw_param_ss_rz_floor`, `fsw_param_st_bs`, `fsw_param_st_latency`, `fsw_param_st_noise_cross`, `fsw_param_st_noise_roll`, `fsw_steplaws`, `kpi_metric_channels`, `kpi_metric_ecss`, `kpi_metric_evaluate`, `kpi_metric_statistics`, `sens_fine_sun`, `sens_sky_view`, `sens_star_catalogue`, `sens_star_image`, `sens_star_tracker` |
| not computed: its module has 10 functions and names none for this node (by its id or its output's symbol) | 1: `fsw_param_mtq_Klqr` |
| not computed: its module has 4 functions and names none for this node (by its id or its output's symbol) | 1: `fsw_param_rw_Klqr` |
| not computed: its module has 6 functions and names none for this node (by its id or its output's symbol) | 2: `catalogue_datasheet_derive`, `design_power_system` |
| not computed: its module has 8 functions and names none for this node (by its id or its output's symbol) | 1: `ctl_floquet_certificate` |
| not computed: its module has 9 functions and names none for this node (by its id or its output's symbol) | 3: `design_loop_converge`, `design_loop_redundancy`, `design_loop_robustness` |
| not computed: needs a row with no value | 6: `fsw_param_mtq_k1`, `fsw_param_mtq_k16`, `fsw_param_mtq_k2`, `fsw_param_mtq_lam16`, `fsw_param_sb_kd`, `fsw_param_sb_kp` |
| not computed: no value, relation or pseudocode yet | 248: `act__cmg`, `act__vscmg`, `cas`, `cat`, `cf`, `ci1`, `cm1`, `cm2`, `cm3`, `cmr`, `cpt`, `ct1`, `ct2`, `ct3`, `design__sizing_cmg`, `design__sizing_fmr`, `design__sizing_mtq`, `design__sizing_rcs`, `design__sizing_rw`, `design__sizing_sens`, `design__sizing_vscmg`, `dyn__carried`, `env_de440_slice`, `env_dtm2020_coefficients`, `env_gravity_default_field`, `env_igrf13_coefficients`, `env_jb2008_indices`, `env_kp_ap_table`, `env_leap_seconds`, `env_ocean_tide_tables`, `env_tidal_eop_terms`, `env_xys06_series`, `fa1`, `fa2`, `fa3`, `fa4`, `fa5`, `fa6`, `fa7`, `fa8`, `fac`, `fdir__carried`, `fsw_param_n_sched`, `fsw_param_sched_mode`, `fsw_param_sched_t`, `gb`, `gc`, `gd`, `gdn__carried`, `ge`, `gf`, `gm`, `gp`, `gq`, `gr`, `gs`, `gw`, `gx`, `hr1`, `hrt`, `l3_budget`, `l3_ctl`, `l3_dist`, `l3_est`, `l3_fmr`, `l3_fsw`, `l3_hils`, `l3_modes`, `l3_mtq`, `l3_oils`, `l3_pnt`, `l3_rcs`, `l3_rw`, `l3_sens`, `lib_catalogue_algorithms_bdot`, `lib_catalogue_algorithms_bdot_bangbang`, `lib_catalogue_algorithms_bdot_gyro`, `lib_catalogue_algorithms_bdot_mag`, `lib_catalogue_algorithms_cmg_sr`, `lib_catalogue_algorithms_genbdot_l1`, `lib_catalogue_algorithms_idmas_split`, `lib_catalogue_algorithms_lqr`, `lib_catalogue_algorithms_mekf`, `lib_catalogue_algorithms_mtq_avanzini2021`, `lib_catalogue_algorithms_mtq_celani2015`, `lib_catalogue_algorithms_mtq_celani2026`, `lib_catalogue_algorithms_mtq_lovera2004`, `lib_catalogue_algorithms_mtq_lqr`, `lib_catalogue_algorithms_mtq_pd`, `lib_catalogue_algorithms_mtq_rate_damp`, `lib_catalogue_algorithms_mtq_smc`, `lib_catalogue_algorithms_mtq_tango2013`, `lib_catalogue_algorithms_pd_alloc`, `lib_catalogue_algorithms_pid`, `lib_catalogue_algorithms_rcs_pwm`, `lib_catalogue_algorithms_rcs_rate`, `lib_catalogue_algorithms_rotor_pinv`, `lib_catalogue_algorithms_smc`, `lib_catalogue_algorithms_sun_acq_rotor`, `lib_catalogue_algorithms_sun_boresight_celani2026`, `lib_catalogue_algorithms_sunspin_damped`, `lib_catalogue_algorithms_sunspin_deruiter2011`, `lib_catalogue_algorithms_sunspin_l1l2`, `lib_catalogue_algorithms_sunspin_l1l2_e2`, `lib_catalogue_algorithms_vscmg_sr`, `lib_catalogue_classes`, `lib_catalogue_components_cmg`, `lib_catalogue_components_coarse_sun_sensor`, `lib_catalogue_components_earth_sensor`, `lib_catalogue_components_fluid_loop`, `lib_catalogue_components_gnss`, `lib_catalogue_components_gyro`, `lib_catalogue_components_magnetometer`, `lib_catalogue_components_magnetorquer`, `lib_catalogue_components_rcs`, `lib_catalogue_components_reaction_wheel`, `lib_catalogue_components_star_tracker`, `lib_catalogue_components_sun_sensor`, `lib_catalogue_components_vscmg`, `lib_catalogue_dispersions`, `lib_catalogue_families`, `lib_catalogue_modes_detumble`, `lib_catalogue_modes_nadir_pointing`, `lib_catalogue_modes_sun_acquisition`, `lib_catalogue_modes_sun_referencing`, `lib_catalogue_parts_syn_ct_1`, `lib_catalogue_parts_syn_es_1`, `lib_catalogue_parts_syn_gyro_1`, `lib_catalogue_parts_syn_mag_1`, `lib_catalogue_parts_syn_mfp_1`, `lib_catalogue_parts_syn_rw_10`, `lib_catalogue_parts_syn_st_1`, `lib_catalogue_parts_syn_sun_1`, `lib_catalogue_parts_trn_cmg_1`, `lib_catalogue_parts_trn_css_1`, `lib_catalogue_parts_trn_gps_1`, `lib_catalogue_parts_trn_gyro_p1`, `lib_catalogue_parts_trn_rcs_3u`, `lib_catalogue_parts_trn_vscmg_1`, `lib_catalogue_products_trn_p_3u_ais`, `lib_catalogue_products_trn_p_3u_ais_css`, `lib_catalogue_products_trn_p_3u_cmg`, `lib_catalogue_products_trn_p_3u_fmr`, `lib_catalogue_products_trn_p_3u_fmr_rcs`, `lib_catalogue_products_trn_p_3u_img`, `lib_catalogue_products_trn_p_3u_rw_rcs`, `lib_catalogue_products_trn_p_3u_vscmg`, `lib_fsw_params_params`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw222_3`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw222_6`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw400_15`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw400_30`, `lib_matlab_sils_data_catalogue_aac_clyde_space_rw400_50`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw0017`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw0057`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw0162`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw0500`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw1200`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw2500`, `lib_matlab_sils_data_catalogue_cubespace_cubewheel_cw5000`, `lib_matlab_sils_data_catalogue_rocket_lab_rw3_0_06`, `lib_matlab_sils_data_catalogue_rocket_lab_rw3_1_0`, `lib_matlab_sils_data_catalogue_rocket_lab_rw_0_003`, `lib_matlab_sils_data_catalogue_rocket_lab_rw_0_01`, `lib_matlab_sils_data_catalogue_rocket_lab_rw_0_03`, `lib_matlab_sils_data_catalogue_tensor_tech_adcs400`, `lib_matlab_sils_data_catalogue_tensor_tech_tensorcmg_10m`, `lib_matlab_sils_data_pipeline_nodes`, `lib_matlab_sils_data_scenario_schema`, `lib_spec_physics_constants`, `lib_spec_physics_ctl`, `lib_spec_physics_env`, `lib_spec_physics_fmr`, `lib_spec_physics_gnc`, `lib_spec_physics_mission`, `lib_spec_physics_mtq`, `lib_spec_physics_orbit`, `lib_spec_physics_rcs`, `lib_spec_physics_risk`, `lib_spec_physics_rw`, `lib_spec_plan_case_inputs`, `lib_spec_plan_case_template`, `lib_spec_plan_expected_node_ids`, `lib_spec_plan_kpis`, `lib_spec_plan_physics`, `lib_spec_plan_seed_content`, `lib_spec_plan_tree`, `lib_spec_plan_twin_map`, `lib_spec_plan_units`, `m1`, `m2`, `m3`, `mgm`, `msn`, `nav__carried`, `od1`, `od2`, `ord`, `p1`, `p1a`, `p1k`, `p2`, `p2a`, `p2k`, `p3`, `p3a`, `p3k`, `p4`, `p4a`, `p4k`, `prg`, `programme_delivery_waves`, `rk1`, `rk2`, `rk3`, `rk4`, `rsk`, `s1`, `s2`, `s3`, `s4`, `s5`, `sat`, `sb0`, `sb1`, `sb2`, `sb3`, `sb4`, `st1`, `st2`, `st3`, `st4`, `st5`, `st6`, `st7`, `std`, `su1`, `su2`, `su3`, `sub`, `sup`, `svc`, `v1`, `v2`, `v3`, `v4`, `ver`, `x_closure` |
| stated | 130: `act_cmg_speed_gain`, `act_fmr_flow_gain`, `act_fmr_flow_tau`, `act_gimbal_tlm_noise`, `act_rotor_tlm_noise`, `act_rw_drive_efficiency`, `act_rw_friction_comp`, `act_rw_torque_noise`, `dyn_cm_direction_x`, `dyn_cm_direction_y`, `dyn_cm_direction_z`, `dyn_initial_attitude_kind_default`, `dyn_initial_error_angle`, `dyn_initial_error_axis`, `dyn_initial_rate_extra`, `dyn_initial_rate_kind_default`, `dyn_initial_rate_magnitude`, `dyn_initial_rate_value`, `dyn_surface_accommodation`, `dyn_surface_specular_share`, `dyn_surface_vb_ratio`, `env_ap_default`, `env_density_scale_default`, `env_f107_default`, `env_f107a_default`, `env_fast_zonal_degree`, `env_field_degree`, `env_force_drag`, `env_force_srp`, `env_force_third_body`, `env_kp_default`, `env_orbit_precision_default`, `env_orbit_step`, `env_refresh_step`, `env_start_arg_lat_default`, `env_torque_aero`, `env_torque_gravity_gradient`, `env_torque_magnetic`, `env_torque_radiation`, `fsw_param_auto_next`, `fsw_param_capture_deg`, `fsw_param_capture_rate_deg_s`, `fsw_param_cmg_k_null`, `fsw_param_cmg_lam0`, `fsw_param_cmg_mu`, `fsw_param_detumble_hold_s`, `fsw_param_dt`, `fsw_param_dump_k`, `fsw_param_fdir_h_frac`, `fsw_param_fdir_s`, `fsw_param_fdir_win_s`, `fsw_param_gd_T`, `fsw_param_gd_axis`, `fsw_param_gd_flip_hyst`, `fsw_param_gd_kind`, `fsw_param_gd_q_inertial`, `fsw_param_gd_roll_deg`, `fsw_param_gd_t0`, `fsw_param_gd_yaw_flip`, `fsw_param_gnss_ecef`, `fsw_param_h_bias`, `fsw_param_ho_hold_s`, `fsw_param_ho_in_dps`, `fsw_param_ho_out_dps`, `fsw_param_igrf_nmax`, `fsw_param_mekf_gate`, `fsw_param_mekf_meas_scale`, `fsw_param_mekf_rej_max`, `fsw_param_mtq_eps`, `fsw_param_mtq_err_max`, `fsw_param_mtq_int_max`, `fsw_param_mtq_meas`, `fsw_param_mtq_period`, `fsw_param_mtq_phi`, `fsw_param_rate_lpf_s`, `fsw_param_rcs_assist`, `fsw_param_rcs_assist_frac`, `fsw_param_rcs_dump`, `fsw_param_rcs_dump_hi`, `fsw_param_rcs_dump_k`, `fsw_param_rcs_dump_lo`, `fsw_param_rcsd_T_damp_s`, `fsw_param_rcsd_deadband_deg_s`, `fsw_param_rcsd_period_s`, `fsw_param_rw_err_max`, `fsw_param_rw_int_max`, `fsw_param_rw_phi`, `fsw_param_sa_done_deg`, `fsw_param_sa_done_hold_s`, `fsw_param_sa_kd`, `fsw_param_sa_w_max_deg_s`, `fsw_param_ss_dr_k1`, `fsw_param_ss_dwell_in_s`, `fsw_param_ss_dwell_out_s`, `fsw_param_ss_k_l1`, `fsw_param_ss_omega_exit_dps`, `fsw_param_ss_omega_max_dps`, `fsw_param_ss_perp_in_dps`, `fsw_param_ss_perp_out_dps`, `fsw_param_ss_sigma0`, `fsw_param_ss_spin_dps`, `fsw_param_ss_sun_min`, `fsw_param_ss_t_check_s`, `fsw_param_ss_z_in_dps`, `fsw_param_st_coast_s`, `fsw_param_start_mode`, `fsw_tune_avanzini_k_over_n`, `fsw_tune_avanzini_lambda`, `fsw_tune_bdot_gain_scale`, `fsw_tune_detumble_exit_deg_s`, `fsw_tune_mtq_gain_d`, `fsw_tune_mtq_gain_p`, `fsw_tune_mtq_wn`, `fsw_tune_mtq_zeta`, `fsw_tune_roll_gain`, `fsw_tune_roll_gate_deg`, `fsw_tune_roll_wn_orbits`, `fsw_tune_roll_zeta`, `fsw_tune_rw_bandwidth`, `fsw_tune_rw_damping`, `fsw_tune_rw_rate_hz`, `fsw_tune_ss_gain`, `gdn_payload_boresight_x`, `gdn_payload_boresight_y`, `gdn_payload_boresight_z`, `gdn_sun_axis_x`, `gdn_sun_axis_y`, `gdn_sun_axis_z`, `vv_record_step_default`, `vv_run_duration_default` |

Closures that moved:

| Closure | Old | Old why | New | New why |
|---|---|---|---|---|
| `kpi_absolute_pointing_error_ape_analysis` | blocked | gp_5 has no value (needs terms (no input names it)) | **blocked** | gp_5 has no value (needs terms[1] from gp_0 (not computed), terms[2] from gp_1 (not computed), terms[3] from gp_2 (not computed), terms[4] from gp_3 (not computed), terms[5] from gp_4 (not computed)) |
| `kpi_settling_time_after_a_slew_analysis` | blocked | gc_2 has no value (needs w_n from gc_0, zeta from gc_1 (a loop, or rows not computed)) | **fail** | gc_2 = 56.5771 (SI) <= 20.0 Second (p2k_1) |
| `kpi_adcs_mass_analysis` | blocked | gb_0 has no value (no value, relation or pseudocode yet) | **blocked** | gb_0 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_orbit_average_power_analysis` | blocked | gb_1 has no value (no value, relation or pseudocode yet) | **blocked** | gb_1 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_peak_power_analysis` | blocked | gb_2 has no value (no value, relation or pseudocode yet) | **blocked** | gb_2 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |
| `kpi_adcs_volume_analysis` | blocked | gb_3 has no value (no value, relation or pseudocode yet) | **blocked** | gb_3 has no value (its module has 2 functions and names none for this node (by its id or its output's symbol)) |

## The health map

### ais_3u: the ADCS blocked -> **blocked**

Groups that moved: ctl blocked -> open, dyn blocked -> open, env blocked -> open, fdir blocked -> open, gdn blocked -> open, nav blocked -> open, sens blocked -> open.

| Closure | Old answer, verdict | New answer, verdict |
|---|---|---|
| none | | |

280 nodes' health moved (blocked -> open: 1, blocked -> unproven: 121, open -> blocked: 7, open -> unproven: 151); 59 nodes are new to the map. A node shows the worst that applies; *unproven* (converted, not yet signed by a person) is what is left once nothing worse does.

| Node | Old | New | New reasons (besides unproven) | Why it moved |
|---|---|---|---|---|
| `act_cmg_axes` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `act_cmg_model` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `act_cmg_steering` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `act_rotor_set` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `act_rotor_telemetry` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `act_vscmg_gimbal_limits` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `act_vscmg_model` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `cf_0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `cf_2` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `cf_3` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `dyn_flexible_mode` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `dyn_kinematics` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `dyn_rigid_body` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `dyn_rotor_coupling` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `dyn_total_momentum` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `env_calendar_time` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_de440` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_density_model` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_drag_force` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_dtm2020_operational` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_dtm2020_research` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_earth_frames` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_erp_force` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_exponential_atmosphere` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_force_model` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_gas_surface` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_geodetic` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `env_gravity_field` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_iau2006` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_jb2008` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_moon_fast` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_ocean_tides` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_orbit_fast` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_orbit_start` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_relativity` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_solid_tides` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_space_weather` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_srp_force` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_third_body` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_tidal_eop` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_time_frames` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_time_scales` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_two_body_elements` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fdir_rotor_health` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fdir_safe_mode` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fdir_sensor_health` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_allocation` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_control` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_drivers` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_estimation` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_guidance` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_modes` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_param_J` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_alloc` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_auto_next` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_bdot_k` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_bdot_law` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_capture_deg` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_capture_rate_deg_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_cmg_k_null` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_cmg_lam0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_cmg_mu` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_detumble_exit` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_detumble_hold_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_dt` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_dump_k` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_es_noise` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_fdir_h_frac` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_fdir_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_fdir_win_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_T` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_axis` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_flip_hyst` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_kind` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_q_inertial` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_q_off` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_roll_deg` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_t0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_yaw_flip` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gim_axis` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_gim_rate_max` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_gnss_ecef` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gps_latency` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_gyro_arw` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_gyro_rrw` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_h_bias` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_has_es` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_has_gps` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_has_gyro` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_has_st` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_has_sun` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ho_hold_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ho_in_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ho_out_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_igrf_nmax` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_jd0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_m_max` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_m_res_est` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mekf_gate` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mekf_mag_err_T` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_mekf_meas_scale` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mekf_rej_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mekf_sig_mag` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_mekf_sig_sun` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_mtq_Gs` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Kd` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Ki` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Kp` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Pth` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Pw` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_eps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_err_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_gg_ff` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_int_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_k1` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_mtq_k16` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_mtq_k2` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_mtq_lam16` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_mtq_lambda` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_law` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_mtq_meas` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_period` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_phi` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mu` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_n_heads` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_nc` | open | unproven | — | both: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ng` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_nr` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rate_lpf_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_assist` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_assist_frac` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_dump` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_dump_hi` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_dump_k` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_dump_lo` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_mib` | open | unproven | — | both: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rcs_res` | open | unproven | — | both: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rcs_tau` | open | unproven | — | both: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rcsd_T_damp_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcsd_deadband_deg_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcsd_period_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_roll_axis` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rot_a0` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_gi` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_h0` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_hmax` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_kind` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_tmax` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rw_Gs` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_Kd` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_Ki` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_Kp` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_dt` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_err_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_int_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_lambda` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_law` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rw_phi` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sa_done_deg` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sa_done_hold_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sa_kd` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sa_w_max_deg_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sb_kd` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_sb_kdroll` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sb_kp` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_sb_kroll` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sb_roll_gate` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dr_k` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dr_k1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dr_k2` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dwell_in_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dwell_out_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_eclipse` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ss_k1` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_k2` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_k_l1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_law` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ss_omega_exit_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_omega_max_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_perp_in_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_perp_out_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_rz_floor` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ss_sigma0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_spin_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_sun_min` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_t_check_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_z_in_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_st_bs` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_st_coast_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_st_latency` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_st_noise_cross` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_st_noise_roll` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_start_mode` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sun_axis` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_steplaws` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `gc_0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gc_1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gc_2` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gd_3` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gd_4` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gd_5` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gdn_boresight_offset` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gdn_guidance` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gdn_mode_manager` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gdn_mode_schedule` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gdn_yaw_flip` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gf_0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_2` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_3` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_4` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_5` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_6` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_7` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_8` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gm_0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gm_1` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gm_2` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gm_3` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gr_1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gr_2` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `l3_dist_row_01` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_02` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_dist_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_dist_row_04` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_05` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_06` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_07` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_08` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_dist_row_09` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_dist_row_10` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_fmr_row_08` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_fmr_row_09` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_fmr_row_10` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_fmr_row_11` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_fmr_row_12` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_fmr_row_13` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_mtq_row_02` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_mtq_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_mtq_row_04` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_mtq_row_05` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_mtq_row_06` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_mtq_row_12` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_mtq_row_13` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_pnt_row_09` | open | blocked | blocked: needs ape (the run's), e_a from gp_2 (not computed), e_t from gp_3 (not computed), e_j from gp_4 (not computed) | both: blocked |
| `l3_rcs_row_01` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rcs_row_02` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rcs_row_04` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_08` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_09` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_10` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_11` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_01` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_02` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_04` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_05` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rw_row_06` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rw_row_07` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_sens_row_01` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_02` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_04` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_05` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_06` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_07` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_08` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_sens_row_13` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_14` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `m2_7` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_gnss_fix` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_gyro_filter` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_onboard_orbit` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_quest` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_time_frames` | blocked | open | open: stated, with no value: its method is env_time_frames's since S7.3 (fsw/pseudocode/02_time_frames_models.pc, one copy); the pseudocode it carried from 1.0.0 stays as provenance | S7.19: open |
| `nav_triad` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `s1_4` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `sens_fine_sun` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `sens_sky_view` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |

### ais_img_3u: the ADCS blocked -> **fails**

Groups that moved: act blocked -> open, ctl blocked -> fails, dyn blocked -> open, env blocked -> open, fdir blocked -> open, gdn blocked -> open, kpi blocked -> fails, nav blocked -> open, sens blocked -> open.

| Closure | Old answer, verdict | New answer, verdict |
|---|---|---|
| `kpi_settling_time_after_a_slew_analysis` | blocked, blocked | **fail, fails** |

284 nodes' health moved (blocked -> fails: 2, blocked -> open: 1, blocked -> unproven: 123, open -> blocked: 7, open -> unproven: 151); 59 nodes are new to the map. A node shows the worst that applies; *unproven* (converted, not yet signed by a person) is what is left once nothing worse does.

| Node | Old | New | New reasons (besides unproven) | Why it moved |
|---|---|---|---|---|
| `act_cmg_axes` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `act_cmg_model` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `act_cmg_steering` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `act_rotor_set` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `act_rotor_telemetry` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `act_vscmg_gimbal_limits` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `act_vscmg_model` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `cf_0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `cf_2` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `cf_3` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `dyn_flexible_mode` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `dyn_kinematics` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `dyn_rigid_body` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `dyn_rotor_coupling` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `dyn_total_momentum` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `env_calendar_time` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_de440` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_density_model` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_drag_force` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_dtm2020_operational` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_dtm2020_research` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_earth_frames` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_erp_force` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_exponential_atmosphere` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_force_model` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_gas_surface` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_geodetic` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `env_gravity_field` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_iau2006` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_jb2008` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_moon_fast` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_ocean_tides` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_orbit_fast` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_orbit_start` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_relativity` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_solid_tides` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_space_weather` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_srp_force` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_third_body` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_tidal_eop` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_time_frames` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_time_scales` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `env_two_body_elements` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fdir_rotor_health` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fdir_safe_mode` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fdir_sensor_health` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_allocation` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_control` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_drivers` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_estimation` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_guidance` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_modes` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `fsw_param_J` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_alloc` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_auto_next` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_bdot_k` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_bdot_law` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_capture_deg` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_capture_rate_deg_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_cmg_k_null` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_cmg_lam0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_cmg_mu` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_detumble_exit` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_detumble_hold_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_dt` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_dump_k` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_es_noise` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_fdir_h_frac` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_fdir_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_fdir_win_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_T` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_axis` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_flip_hyst` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_kind` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_q_inertial` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_q_off` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_roll_deg` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_t0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gd_yaw_flip` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gim_axis` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_gim_rate_max` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_gnss_ecef` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_gps_latency` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_gyro_arw` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_gyro_rrw` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_h_bias` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_has_es` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_has_gps` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_has_gyro` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_has_st` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_has_sun` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ho_hold_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ho_in_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ho_out_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_igrf_nmax` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_jd0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_m_max` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_m_res_est` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mekf_gate` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mekf_mag_err_T` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_mekf_meas_scale` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mekf_rej_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mekf_sig_mag` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_mekf_sig_sun` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_mtq_Gs` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Kd` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Ki` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Kp` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Pth` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_Pw` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_eps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_err_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_gg_ff` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_int_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_k1` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_mtq_k16` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_mtq_k2` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_mtq_lam16` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_mtq_lambda` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_law` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_mtq_meas` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_period` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mtq_phi` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_mu` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_n_heads` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_nc` | open | unproven | — | both: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ng` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_nr` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rate_lpf_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_assist` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_assist_frac` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_dump` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_dump_hi` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_dump_k` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_dump_lo` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcs_mib` | open | unproven | — | both: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rcs_res` | open | unproven | — | both: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rcs_tau` | open | unproven | — | both: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rcsd_T_damp_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcsd_deadband_deg_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rcsd_period_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_roll_axis` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rot_a0` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_gi` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_h0` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_hmax` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_kind` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rot_tmax` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rw_Gs` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_Kd` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_Ki` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_Kp` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_dt` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_err_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_int_max` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_lambda` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_rw_law` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_rw_phi` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sa_done_deg` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sa_done_hold_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sa_kd` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sa_w_max_deg_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sb_kd` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_sb_kdroll` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sb_kp` | open | blocked | blocked: needs law from fsw_param_mtq_law (not computed) | S7.19: blocked |
| `fsw_param_sb_kroll` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sb_roll_gate` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dr_k` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dr_k1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dr_k2` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dwell_in_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_dwell_out_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_eclipse` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ss_k1` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_k2` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_k_l1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_law` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ss_omega_exit_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_omega_max_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_perp_in_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_perp_out_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_rz_floor` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_ss_sigma0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_spin_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_sun_min` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_t_check_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_ss_z_in_dps` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_st_bs` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_st_coast_s` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_st_latency` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_st_noise_cross` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_st_noise_roll` | open | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `fsw_param_start_mode` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_param_sun_axis` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `fsw_steplaws` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `gc_0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gc_1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gc_2` | blocked | fails | fails: decides kpi_settling_time_after_a_slew_analysis, which fails | S7.19: fails |
| `gd_3` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gd_4` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gd_5` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gdn_boresight_offset` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gdn_guidance` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gdn_mode_manager` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gdn_mode_schedule` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gdn_yaw_flip` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `gf_0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_2` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_3` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_4` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_5` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_6` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_7` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_8` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gf_9` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gm_0` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gm_1` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gm_2` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gm_3` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gr_1` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gr_2` | open | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gr_3` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `gr_4` | blocked | unproven | — | S7.19: nothing left but unproven: its value now stated or computed |
| `kpi_settling_time_after_a_slew_analysis` | blocked | fails | fails: gc_2 = 56.5771 (SI) <= 20.0 Second (p2k_1) | S7.19: fails |
| `l3_dist_row_01` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_02` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_dist_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_dist_row_04` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_05` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_06` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_07` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_dist_row_08` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_dist_row_09` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_dist_row_10` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_fmr_row_08` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_fmr_row_09` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_fmr_row_10` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_fmr_row_11` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_fmr_row_12` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_fmr_row_13` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_mtq_row_02` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_mtq_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_mtq_row_04` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_mtq_row_05` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_mtq_row_06` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_mtq_row_12` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_mtq_row_13` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_pnt_row_09` | open | blocked | blocked: needs ape (the run's), e_a from gp_2 (not computed), e_t from gp_3 (not computed), e_j from gp_4 (not computed) | both: blocked |
| `l3_rcs_row_01` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rcs_row_02` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rcs_row_04` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_08` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_09` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_10` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rcs_row_11` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_01` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_02` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_04` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_rw_row_05` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rw_row_06` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_rw_row_07` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_sens_row_01` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_02` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_03` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_04` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_05` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_06` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_07` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_08` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `l3_sens_row_13` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `l3_sens_row_14` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `m2_7` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_gnss_fix` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_gyro_filter` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_onboard_orbit` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_quest` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `nav_time_frames` | blocked | open | open: stated, with no value: its method is env_time_frames's since S7.3 (fsw/pseudocode/02_time_frames_models.pc, one copy); the pseudocode it carried from 1.0.0 stays as provenance | S7.19: open |
| `nav_triad` | blocked | unproven | — | S7.19: nothing left but unproven: computed during a run (its inputs the run's) |
| `s1_4` | open | unproven | — | both: nothing left but unproven: its value now stated or computed |
| `sens_fine_sun` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |
| `sens_sky_view` | blocked | unproven | — | earlier S7 steps: the method or value an S7 step gave it |

## Every new value against the engine

Each row evaluate gives a value for where d6aee43's gave none (a stated value it now reads, or a value it now computes), against the engine's for the case. *agrees* is bit for bit unless the note says otherwise; *differs* names why; *—* where the engine computes no such number.

| Case | Row | State | Evaluate | Engine | Engine source | Verdict | Note | Why they differ |
|---|---|---|---|---|---|---|---|---|
| ais_3u | `act_cmg_speed_gain` | stated | 1.0 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `act_fmr_flow_gain` | stated | 2.0 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `act_fmr_flow_tau` | stated | 0.3 | 0.3 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `act_gimbal_tlm_noise` | stated | 1e-5 | 1e-05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `act_rotor_tlm_noise` | stated | 1e-7 | 1e-07 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `act_rw_drive_efficiency` | stated | 0.8 | 0.8 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `act_rw_friction_comp` | stated | 0.95 | 0.95 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `act_rw_torque_noise` | stated | 0.001 | 0.001 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `cf_0` | stated | 3 Count | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `cf_2` | stated | 3 Count | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `cf_3` | stated | 0 Count | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_cm_direction_x` | stated | 0.30 | 0.3 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_cm_direction_y` | stated | 0.70 | 0.7 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_cm_direction_z` | stated | -0.65 | -0.65 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_initial_attitude_kind_default` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_initial_error_angle` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_initial_error_axis` | stated | [1, 0, 0] | [1.0, 0.0, 0.0] | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_initial_rate_extra` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_initial_rate_kind_default` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_initial_rate_magnitude` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_initial_rate_value` | stated | [0, 0, 0] | [0.0, 0.0, 0.0] | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_surface_accommodation` | stated | 0.8 | 0.8 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_surface_specular_share` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_surface_vb_ratio` | stated | 0.05 | 0.05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `dyn_truth_plant` | computed | [[0.0067, 0, 0], [0, 0.042, 0], [0, 0, 0.042]] (SI) | [[0.0067, 0.0, 0.0], [0.0, 0.042, 0.0], [0.0, 0.0, 0.042]] | the blob of sun_mtq_ais: J (the truth inertia the flight software is loaded with) | agrees | bit for bit |  |
| ais_3u | `env_ap_default` | stated | 7 | 7.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_case_orbit` | computed | 5738.99 (SI) | 5738.992815014798 | sun_mtq_ais's manifest orbit.period_s | agrees | bit for bit |  |
| ais_3u | `env_density_scale_default` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_f107_default` | stated | 130 | 130.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_f107a_default` | stated | 130 | 130.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_fast_zonal_degree` | stated | 6 | 6.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_field_degree` | stated | 13 | 13.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_force_drag` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_force_srp` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_force_third_body` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_kp_default` | stated | 2 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_orbit_precision_default` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_orbit_step` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_refresh_step` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_start_arg_lat_default` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_torque_aero` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_torque_gravity_gradient` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_torque_magnetic` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `env_torque_radiation` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_J` | computed | [[0.0067, 0, 0], [0, 0.042, 0], [0, 0, 0.042]] (SI) | [[0.0067, 0.0, 0.0], [0.0, 0.042, 0.0], [0.0, 0.0, 0.042]] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_auto_next` | stated | 255 | 255.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_bdot_k` | computed | 8.76379e-05 (SI) | 8.763791486082992e-05 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_capture_deg` | stated | 3 | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_capture_rate_deg_s` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_cmg_k_null` | stated | 0.002 | 0.002 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_cmg_lam0` | stated | 1e-09 | 1e-09 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_cmg_mu` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_detumble_exit` | computed | 0.00872665 (SI) | 0.008726646259971648 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_detumble_hold_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_dt` | stated | 0.1 | 0.1 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_dump_k` | stated | 0.002 | 0.002 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_fdir_h_frac` | stated | 0.005 | 0.005 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_fdir_s` | stated | 3 | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_fdir_win_s` | stated | 120 | 120.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_gd_T` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_gd_axis` | stated | [0, 0, 0] | [0.0, 0.0, 0.0] | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_gd_flip_hyst` | stated | 0.1 | 0.1 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_gd_kind` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_gd_q_inertial` | stated | [0, 0, 0, 1] | [0.0, 0.0, 0.0, 1.0] | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_gd_q_off` | computed | [0, 0, 0, 1] (SI) | [0.0, 0.0, 0.7071067811865475, 0.7071067811865475] | the blob of sun_mtq_ais (adcs params, from the design) | differs | largest relative difference 1 | the boresight: TRN-P-3U-AIS states payload_boresight_body [1.0, 0.0, 0.0]; evaluate has a case and no product, so its wire takes gdn's default (+Y, gdn_payload_boresight_*), the axis a product that states none flies (the IMG family states +Y, which agrees). A case names no product in the design: the flight parameters that hang on the product cannot be evaluated for a case |
| ais_3u | `fsw_param_gd_roll_deg` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_gd_t0` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_gd_yaw_flip` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_gnss_ecef` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_h_bias` | stated | 0.002 | 0.002 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ho_hold_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ho_in_dps` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ho_out_dps` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_igrf_nmax` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_jd0` | computed | 2.46141e+06 (SI) | 2461406.75 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_m_res_est` | computed | [0.0057735, 0.0057735, 0.0057735] (SI) | [0.005773502691896258, 0.005773502691896258, 0.005773502691896258] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mekf_gate` | stated | 16.27 | 16.27 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mekf_meas_scale` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mekf_rej_max` | stated | 30 | 30.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_Gs` | computed | [1e-05, 1e-05, 1e-05] (SI) | [1e-05, 1e-05, 1e-05] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_Kd` | computed | [0.000134, 0.00084, 0.00084] (SI) | [0.000134, 0.00084, 0.00084] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_Ki` | computed | [0, 0, 0] (SI) | [0.0, 0.0, 0.0] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_Kp` | computed | [1.675e-07, 1.05e-06, 1.05e-06] (SI) | [1.675e-07, 1.0500000000000001e-06, 1.0500000000000001e-06] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_Pth` | computed | [[4.98567e-07, 0, 0], [0, 1.27604e-06, 0], [0, 0, 1.27604e-06]] (SI) | [[4.985665295262127e-07, 0.0, 0.0], [0.0, 1.276038768529423e-06, 0.0], [0.0, 0.0, 1.276... | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_Pw` | computed | [[0.000398799, 0, 0], [0, 0.00102111, 0], [0, 0, 0.00102111]] (SI) | [[0.00039879927297085255, 0.0, 0.0], [0.0, 0.0010211101662281358, 0.0], [0.0, 0.0, 0.00... | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_eps` | stated | 0.001 | 0.001 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_err_max` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_gg_ff` | computed | 3 (SI) | 1 | the blob of sun_mtq_ais (adcs params, from the design) | differs | largest relative difference 0.667 | the boresight: TRN-P-3U-AIS states payload_boresight_body [1.0, 0.0, 0.0]; evaluate has a case and no product, so its wire takes gdn's default (+Y, gdn_payload_boresight_*), the axis a product that states none flies (the IMG family states +Y, which agrees). A case names no product in the design: the flight parameters that hang on the product cannot be evaluated for a case |
| ais_3u | `fsw_param_mtq_int_max` | stated | 0.05 | 0.05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_lambda` | computed | 0.0025 (SI) | 0.0025 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_meas` | stated | 0.2 | 0.2 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_period` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mtq_phi` | stated | 0.0005 | 0.0005 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_mu` | computed | 3.986e+14 (SI) | 398600441800000.0 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rate_lpf_s` | stated | 0.3 | 0.3 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rcs_assist` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rcs_assist_frac` | stated | 0.8 | 0.8 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rcs_dump` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rcs_dump_hi` | stated | 0.004 | 0.004 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rcs_dump_k` | stated | 0.05 | 0.05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rcs_dump_lo` | stated | 0.001 | 0.001 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rcsd_T_damp_s` | stated | 20 | 20.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rcsd_deadband_deg_s` | stated | 0.2 | 0.2 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rcsd_period_s` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_roll_axis` | computed | [0, 1, 0] (SI) | [1.0, 0.0, 0.0] | the blob of sun_mtq_ais (adcs params, from the design) | differs | largest relative difference 1 | the boresight: TRN-P-3U-AIS states payload_boresight_body [1.0, 0.0, 0.0]; evaluate has a case and no product, so its wire takes gdn's default (+Y, gdn_payload_boresight_*), the axis a product that states none flies (the IMG family states +Y, which agrees). A case names no product in the design: the flight parameters that hang on the product cannot be evaluated for a case |
| ais_3u | `fsw_param_rw_Gs` | computed | [0.00072, 0.00072, 0.00072] (SI) | [0.00072, 0.00072, 0.00072] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rw_Kd` | computed | [0.02412, 0.1512, 0.1512] (SI) | [0.024120000000000003, 0.1512, 0.1512] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rw_Ki` | computed | [0.000732645, 0.0045927, 0.0045927] (SI) | [0.0007326450000000001, 0.004592700000000001, 0.004592700000000001] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rw_Kp` | computed | [0.005427, 0.03402, 0.03402] (SI) | [0.005427, 0.03402, 0.03402] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rw_dt` | computed | 0.1 (SI) | 0.1 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rw_err_max` | stated | 0.2 | 0.2 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rw_int_max` | stated | 0.02 | 0.02 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rw_lambda` | computed | 0.225 (SI) | 0.225 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_rw_phi` | stated | 0.0002 | 0.0002 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_sa_done_deg` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_sa_done_hold_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_sa_kd` | stated | 0.1 | 0.1 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_sa_w_max_deg_s` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_sb_kdroll` | computed | 0.000275896 (SI) | 4.4011912454009244e-05 | the blob of sun_mtq_ais (adcs params, from the design) | differs | largest relative difference 0.84 | the boresight: TRN-P-3U-AIS states payload_boresight_body [1.0, 0.0, 0.0]; evaluate has a case and no product, so its wire takes gdn's default (+Y, gdn_payload_boresight_*), the axis a product that states none flies (the IMG family states +Y, which agrees). A case names no product in the design: the flight parameters that hang on the product cannot be evaluated for a case |
| ais_3u | `fsw_param_sb_kroll` | computed | 4.53086e-07 (SI) | 7.227792678579754e-08 | the blob of sun_mtq_ais (adcs params, from the design) | differs | largest relative difference 0.84 | the boresight: TRN-P-3U-AIS states payload_boresight_body [1.0, 0.0, 0.0]; evaluate has a case and no product, so its wire takes gdn's default (+Y, gdn_payload_boresight_*), the axis a product that states none flies (the IMG family states +Y, which agrees). A case names no product in the design: the flight parameters that hang on the product cannot be evaluated for a case |
| ais_3u | `fsw_param_sb_roll_gate` | computed | 0.965926 (SI) | 0.9659258262890683 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_dr_k` | computed | 0.01 (SI) | 0.01 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_dr_k1` | stated | 1.5 | 1.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_dr_k2` | computed | 0.021 (SI) | 0.021 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_dwell_in_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_dwell_out_s` | stated | 30 | 30.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_k1` | computed | 0.01 (SI) | 0.01 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_k2` | computed | 0.05 (SI) | 0.05 | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_k_l1` | stated | 1000000 | 1000000.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_omega_exit_dps` | stated | 2 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_omega_max_dps` | stated | 100 | 100.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_perp_in_dps` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_perp_out_dps` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_sigma0` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_spin_dps` | stated | 6 | 6.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_sun_min` | stated | 0.05 | 0.05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_t_check_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_ss_z_in_dps` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_st_coast_s` | stated | 900 | 900.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_start_mode` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_param_sun_axis` | computed | [0, 0, -1] (SI) | [0.0, 0.0, -1.0] | the blob of sun_mtq_ais (adcs params, from the design) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_avanzini_k_over_n` | stated | 0.84 | 0.84 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_avanzini_lambda` | stated | 0.08 | 0.08 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_bdot_gain_scale` | stated | 3 | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_detumble_exit_deg_s` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_mtq_gain_d` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_mtq_gain_p` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_mtq_wn` | stated | 0.005 | 0.005 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_mtq_zeta` | stated | 2 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_roll_gain` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_roll_gate_deg` | stated | 15 | 15.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_roll_wn_orbits` | stated | 3 | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_roll_zeta` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_rw_bandwidth` | stated | 0.9 | 0.9 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_rw_damping` | stated | 2 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_rw_rate_hz` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `fsw_tune_ss_gain` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gc_0` | stated | 0.1 RadianPerSecond | 0.1 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gc_1` | stated | 0.707 One | 0.707 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gc_2` | computed | 56.5771 (SI) |  | no engine counterpart: the engine computes no settling time from gc_0 and gc_1 (1.0.0's stated 0.1 rad/s and 0.707); the fine pointing it flies is tuned at fsw_tune_rw_bandwidth 0.9 rad/s and fsw_tune_rw_damping 2 (or the scenario's), whose 4/(zeta wn) is 2.2 s | — | no engine counterpart: the engine computes no settling time from gc_0 and gc_1 (1.0.0's stated 0.1 rad/s and 0.707); the fine pointing it flies is tuned at fsw_tune_rw_bandwidth 0.9 rad/s and fsw_tune_rw_damping 2 (or the scenario's), whose 4/(zeta wn) is 2.2 s |  |
| ais_3u | `gd_3` | computed | 4.6512e-07 (SI) |  | no engine counterpart: the survey gives the total torque only (demand.tau_dist), not the residual dipole's worst case; the sized coil's authority is not 1.0.0's 0.45 A m^2 a side | — | no engine counterpart: the survey gives the total torque only (demand.tau_dist), not the residual dipole's worst case; the sized coil's authority is not 1.0.0's 0.45 A m^2 a side |  |
| ais_3u | `gd_4` | computed | 5.47392e-07 (SI) | 5.658232382243399e-07 | adcs size: demand.tau_dist (the survey's peak over four attitudes) | differs | largest relative difference 0.0326 | the design's analysis row sums each torque's worst case (gravity gradient at the inertia's spread, aerodynamic and solar at full area and offset, magnetic at the field's maximum); the engine's survey flies the orbit and takes the largest total at any time |
| ais_3u | `gd_5` | computed | 0.00314148 (SI) | 0.0007630557317062136 | adcs size: demand.h_secular (the survey's worst secular momentum an orbit) | differs | largest relative difference 0.757 | the analysis row takes the worst total torque held for a whole orbit (tau_d T); the survey integrates the torque the orbit actually gives, which largely cancels |
| ais_3u | `gdn_payload_boresight_x` | stated | 0.0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gdn_payload_boresight_y` | stated | 1.0 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gdn_payload_boresight_z` | stated | 0.0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gdn_sun_axis_x` | stated | 0.0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gdn_sun_axis_y` | stated | 0.0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gdn_sun_axis_z` | stated | -1.0 | -1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gf_0` | stated | 3 Millimetre | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gf_1` | stated | 0.022 SquareMetre | 0.022 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gf_2` | stated | 6440 KgPerCubicMetre | 6440.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gf_3` | stated | 0.0024 PascalSecond | 0.0024 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gf_4` | stated | 0.2 MetrePerSecond | 0.2 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gf_5` | stated | 0.72 Metre | 0.72 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gf_6` | computed | 0.000400591 (SI) |  | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) | — | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) |  |
| ais_3u | `gf_7` | computed | 0.754688 (SI) |  | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) | — | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) |  |
| ais_3u | `gf_8` | computed | 0.00173718 (SI) |  | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) | — | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) |  |
| ais_3u | `gm_0` | stated | 0.45 AmpereSquareMetre | 0.45 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gm_1` | computed | 0.0235377 (SI) | 0.025667527060151837 | adcs size: the sized coil's sizing.m_momentum_Am2 | differs | largest relative difference 0.083 | two different laws: the analysis row rejects the total torque at the minimum field (tau_d/B_min); the sizing's momentum dipole stores the survey's momentum |
| ais_3u | `gm_2` | computed | 0.0235377 (SI) | 0.12471712848439347 | adcs size: the sized coil's sizing.m_dump_Am2 | differs | largest relative difference 0.811 | the analysis row dumps gd_5's bound in one orbit at the dipole's minimum field; the sizing dumps the survey's secular momentum (mtq_dipoles) |
| ais_3u | `gm_3` | computed | 1.04652e-05 (SI) |  | no engine counterpart: the survey gives the total torque only (demand.tau_dist), not the residual dipole's worst case; the sized coil's authority is not 1.0.0's 0.45 A m^2 a side | — | no engine counterpart: the survey gives the total torque only (demand.tau_dist), not the residual dipole's worst case; the sized coil's authority is not 1.0.0's 0.45 A m^2 a side |  |
| ais_3u | `gr_1` | stated | 0.8 Metre | 0.8 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `gr_2` | stated | 220 Second | 220.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `s1_4` | computed | [0.00599251, 0.0139825, -0.0129838] (SI) | [0.005992514033267068, 0.013982532744289823, -0.012983780405411982] | sun_mtq_ais's manifest assumptions.cm_offset_m | agrees | bit for bit |  |
| ais_3u | `vv_record_step_default` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_3u | `vv_run_duration_default` | stated | 600 | 600.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `act_cmg_speed_gain` | stated | 1.0 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `act_fmr_flow_gain` | stated | 2.0 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `act_fmr_flow_tau` | stated | 0.3 | 0.3 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `act_gimbal_tlm_noise` | stated | 1e-5 | 1e-05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `act_rotor_tlm_noise` | stated | 1e-7 | 1e-07 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `act_rw_drive_efficiency` | stated | 0.8 | 0.8 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `act_rw_friction_comp` | stated | 0.95 | 0.95 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `act_rw_torque_noise` | stated | 0.001 | 0.001 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `cf_0` | stated | 3 Count | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `cf_2` | stated | 3 Count | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `cf_3` | stated | 0 Count | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_cm_direction_x` | stated | 0.30 | 0.3 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_cm_direction_y` | stated | 0.70 | 0.7 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_cm_direction_z` | stated | -0.65 | -0.65 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_initial_attitude_kind_default` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_initial_error_angle` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_initial_error_axis` | stated | [1, 0, 0] | [1.0, 0.0, 0.0] | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_initial_rate_extra` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_initial_rate_kind_default` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_initial_rate_magnitude` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_initial_rate_value` | stated | [0, 0, 0] | [0.0, 0.0, 0.0] | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_surface_accommodation` | stated | 0.8 | 0.8 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_surface_specular_share` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_surface_vb_ratio` | stated | 0.05 | 0.05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `dyn_truth_plant` | computed | [[0.0067, 0, 0], [0, 0.042, 0], [0, 0, 0.042]] (SI) | [[0.0067, 0.0, 0.0], [0.0, 0.042, 0.0], [0.0, 0.0, 0.042]] | the blob of detumble_rcs: J (the truth inertia the flight software is loaded with) | agrees | bit for bit |  |
| ais_img_3u | `env_ap_default` | stated | 7 | 7.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_case_orbit` | computed | 5738.99 (SI) | 5738.992815014798 | detumble_rcs's manifest orbit.period_s | agrees | bit for bit |  |
| ais_img_3u | `env_density_scale_default` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_f107_default` | stated | 130 | 130.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_f107a_default` | stated | 130 | 130.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_fast_zonal_degree` | stated | 6 | 6.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_field_degree` | stated | 13 | 13.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_force_drag` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_force_srp` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_force_third_body` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_kp_default` | stated | 2 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_orbit_precision_default` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_orbit_step` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_refresh_step` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_start_arg_lat_default` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_torque_aero` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_torque_gravity_gradient` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_torque_magnetic` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `env_torque_radiation` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_J` | computed | [[0.0067, 0, 0], [0, 0.042, 0], [0, 0, 0.042]] (SI) | [[0.0067, 0.0, 0.0], [0.0, 0.042, 0.0], [0.0, 0.0, 0.042]] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_auto_next` | stated | 255 | 255.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_bdot_k` | computed | 8.76379e-05 (SI) | 8.763791486082992e-05 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_capture_deg` | stated | 3 | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_capture_rate_deg_s` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_cmg_k_null` | stated | 0.002 | 0.002 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_cmg_lam0` | stated | 1e-09 | 1e-09 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_cmg_mu` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_detumble_exit` | computed | 0.00872665 (SI) | 0.008726646259971648 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_detumble_hold_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_dt` | stated | 0.1 | 0.1 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_dump_k` | stated | 0.002 | 0.002 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_fdir_h_frac` | stated | 0.005 | 0.005 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_fdir_s` | stated | 3 | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_fdir_win_s` | stated | 120 | 120.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gd_T` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gd_axis` | stated | [0, 0, 0] | [0.0, 0.0, 0.0] | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gd_flip_hyst` | stated | 0.1 | 0.1 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gd_kind` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gd_q_inertial` | stated | [0, 0, 0, 1] | [0.0, 0.0, 0.0, 1.0] | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gd_q_off` | computed | [0, 0, 0, 1] (SI) | [0.0, 0.0, 0.0, 1.0] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gd_roll_deg` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gd_t0` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gd_yaw_flip` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_gnss_ecef` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_h_bias` | stated | 0.002 | 0.002 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ho_hold_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ho_in_dps` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ho_out_dps` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_igrf_nmax` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_jd0` | computed | 2.46141e+06 (SI) | 2461406.75 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_m_res_est` | computed | [0.0057735, 0.0057735, 0.0057735] (SI) | [0.005773502691896258, 0.005773502691896258, 0.005773502691896258] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mekf_gate` | stated | 16.27 | 16.27 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mekf_meas_scale` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mekf_rej_max` | stated | 30 | 30.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_Gs` | computed | [1e-05, 1e-05, 1e-05] (SI) | [1e-05, 1e-05, 1e-05] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_Kd` | computed | [0.000134, 0.00084, 0.00084] (SI) | [0.000134, 0.00084, 0.00084] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_Ki` | computed | [0, 0, 0] (SI) | [0.0, 0.0, 0.0] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_Kp` | computed | [1.675e-07, 1.05e-06, 1.05e-06] (SI) | [1.675e-07, 1.0500000000000001e-06, 1.0500000000000001e-06] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_Pth` | computed | [[4.98567e-07, 0, 0], [0, 1.27604e-06, 0], [0, 0, 1.27604e-06]] (SI) | [[4.985665295262127e-07, 0.0, 0.0], [0.0, 1.276038768529423e-06, 0.0], [0.0, 0.0, 1.276... | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_Pw` | computed | [[0.000398799, 0, 0], [0, 0.00102111, 0], [0, 0, 0.00102111]] (SI) | [[0.00039879927297085255, 0.0, 0.0], [0.0, 0.0010211101662281358, 0.0], [0.0, 0.0, 0.00... | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_eps` | stated | 0.001 | 0.001 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_err_max` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_gg_ff` | computed | 3 (SI) | 3 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_int_max` | stated | 0.05 | 0.05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_lambda` | computed | 0.0025 (SI) | 0.0025 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_meas` | stated | 0.2 | 0.2 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_period` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mtq_phi` | stated | 0.0005 | 0.0005 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_mu` | computed | 3.986e+14 (SI) | 398600441800000.0 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rate_lpf_s` | stated | 0.3 | 0.3 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rcs_assist` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rcs_assist_frac` | stated | 0.8 | 0.8 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rcs_dump` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rcs_dump_hi` | stated | 0.004 | 0.004 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rcs_dump_k` | stated | 0.05 | 0.05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rcs_dump_lo` | stated | 0.001 | 0.001 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rcsd_T_damp_s` | stated | 20 | 20.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rcsd_deadband_deg_s` | stated | 0.2 | 0.2 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rcsd_period_s` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_roll_axis` | computed | [0, 1, 0] (SI) | [0.0, 1.0, 0.0] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rw_Gs` | computed | [0.00072, 0.00072, 0.00072] (SI) | [0.00072, 0.00072, 0.00072] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rw_Kd` | computed | [0.02412, 0.1512, 0.1512] (SI) | [0.024120000000000003, 0.1512, 0.1512] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rw_Ki` | computed | [0.000732645, 0.0045927, 0.0045927] (SI) | [0.0007326450000000001, 0.004592700000000001, 0.004592700000000001] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rw_Kp` | computed | [0.005427, 0.03402, 0.03402] (SI) | [0.005427, 0.03402, 0.03402] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rw_dt` | computed | 0.1 (SI) | 0.1 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rw_err_max` | stated | 0.2 | 0.2 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rw_int_max` | stated | 0.02 | 0.02 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rw_lambda` | computed | 0.225 (SI) | 0.225 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_rw_phi` | stated | 0.0002 | 0.0002 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_sa_done_deg` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_sa_done_hold_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_sa_kd` | stated | 0.1 | 0.1 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_sa_w_max_deg_s` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_sb_kdroll` | computed | 0.000275896 (SI) | 0.00027589557060722213 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_sb_kroll` | computed | 4.53086e-07 (SI) | 4.530855111945517e-07 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_sb_roll_gate` | computed | 0.965926 (SI) | 0.9659258262890683 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_dr_k` | computed | 0.01 (SI) | 0.01 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_dr_k1` | stated | 1.5 | 1.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_dr_k2` | computed | 0.021 (SI) | 0.021 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_dwell_in_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_dwell_out_s` | stated | 30 | 30.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_k1` | computed | 0.01 (SI) | 0.01 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_k2` | computed | 0.05 (SI) | 0.05 | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_k_l1` | stated | 1000000 | 1000000.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_omega_exit_dps` | stated | 2 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_omega_max_dps` | stated | 100 | 100.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_perp_in_dps` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_perp_out_dps` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_sigma0` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_spin_dps` | stated | 6 | 6.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_sun_min` | stated | 0.05 | 0.05 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_t_check_s` | stated | 60 | 60.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_ss_z_in_dps` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_st_coast_s` | stated | 900 | 900.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_start_mode` | stated | 0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_param_sun_axis` | computed | [0, 0, -1] (SI) | [0.0, 0.0, -1.0] | the blob of detumble_rcs (adcs params, from the design) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_avanzini_k_over_n` | stated | 0.84 | 0.84 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_avanzini_lambda` | stated | 0.08 | 0.08 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_bdot_gain_scale` | stated | 3 | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_detumble_exit_deg_s` | stated | 0.5 | 0.5 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_mtq_gain_d` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_mtq_gain_p` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_mtq_wn` | stated | 0.005 | 0.005 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_mtq_zeta` | stated | 2 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_roll_gain` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_roll_gate_deg` | stated | 15 | 15.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_roll_wn_orbits` | stated | 3 | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_roll_zeta` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_rw_bandwidth` | stated | 0.9 | 0.9 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_rw_damping` | stated | 2 | 2.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_rw_rate_hz` | stated | 10 | 10.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `fsw_tune_ss_gain` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gc_0` | stated | 0.1 RadianPerSecond | 0.1 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gc_1` | stated | 0.707 One | 0.707 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gc_2` | computed | 56.5771 (SI) |  | no engine counterpart: the engine computes no settling time from gc_0 and gc_1 (1.0.0's stated 0.1 rad/s and 0.707); the fine pointing it flies is tuned at fsw_tune_rw_bandwidth 0.9 rad/s and fsw_tune_rw_damping 2 (or the scenario's), whose 4/(zeta wn) is 2.2 s | — | no engine counterpart: the engine computes no settling time from gc_0 and gc_1 (1.0.0's stated 0.1 rad/s and 0.707); the fine pointing it flies is tuned at fsw_tune_rw_bandwidth 0.9 rad/s and fsw_tune_rw_damping 2 (or the scenario's), whose 4/(zeta wn) is 2.2 s |  |
| ais_img_3u | `gd_3` | computed | 4.6512e-07 (SI) |  | no engine counterpart: the survey gives the total torque only (demand.tau_dist), not the residual dipole's worst case; the sized coil's authority is not 1.0.0's 0.45 A m^2 a side | — | no engine counterpart: the survey gives the total torque only (demand.tau_dist), not the residual dipole's worst case; the sized coil's authority is not 1.0.0's 0.45 A m^2 a side |  |
| ais_img_3u | `gd_4` | computed | 5.47392e-07 (SI) | 4.6888221818267933e-07 | adcs size: demand.tau_dist (the survey's peak over four attitudes) | differs | largest relative difference 0.143 | the design's analysis row sums each torque's worst case (gravity gradient at the inertia's spread, aerodynamic and solar at full area and offset, magnetic at the field's maximum); the engine's survey flies the orbit and takes the largest total at any time |
| ais_img_3u | `gd_5` | computed | 0.00314148 (SI) | 0.0006236740179952858 | adcs size: demand.h_secular (the survey's worst secular momentum an orbit) | differs | largest relative difference 0.801 | the analysis row takes the worst total torque held for a whole orbit (tau_d T); the survey integrates the torque the orbit actually gives, which largely cancels |
| ais_img_3u | `gdn_payload_boresight_x` | stated | 0.0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gdn_payload_boresight_y` | stated | 1.0 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gdn_payload_boresight_z` | stated | 0.0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gdn_sun_axis_x` | stated | 0.0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gdn_sun_axis_y` | stated | 0.0 | 0.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gdn_sun_axis_z` | stated | -1.0 | -1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gf_0` | stated | 3 Millimetre | 3.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gf_1` | stated | 0.022 SquareMetre | 0.022 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gf_2` | stated | 6440 KgPerCubicMetre | 6440.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gf_3` | stated | 0.0024 PascalSecond | 0.0024 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gf_4` | stated | 0.2 MetrePerSecond | 0.2 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gf_5` | stated | 0.72 Metre | 0.72 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gf_6` | computed | 0.000400591 (SI) |  | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) | — | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) |  |
| ais_img_3u | `gf_7` | computed | 0.754688 (SI) |  | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) | — | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) |  |
| ais_img_3u | `gf_8` | computed | 0.00173718 (SI) |  | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) | — | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) |  |
| ais_img_3u | `gf_9` | computed | 56.5657 (SI) |  | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) | — | no engine counterpart: the sizing (adcs size, sizefmr) designs its own ring from the demand and the knobs, not 1.0.0's stated bore, area, fluid and speed (gf_0 to gf_5) |  |
| ais_img_3u | `gm_0` | stated | 0.45 AmpereSquareMetre | 0.45 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gm_1` | computed | 0.0235377 (SI) | 0.02021692116638005 | adcs size: the sized coil's sizing.m_momentum_Am2 | differs | largest relative difference 0.141 | two different laws: the analysis row rejects the total torque at the minimum field (tau_d/B_min); the sizing's momentum dipole stores the survey's momentum |
| ais_img_3u | `gm_2` | computed | 0.0235377 (SI) | 0.10391054352889326 | adcs size: the sized coil's sizing.m_dump_Am2 | differs | largest relative difference 0.773 | the analysis row dumps gd_5's bound in one orbit at the dipole's minimum field; the sizing dumps the survey's secular momentum (mtq_dipoles) |
| ais_img_3u | `gm_3` | computed | 1.04652e-05 (SI) |  | no engine counterpart: the survey gives the total torque only (demand.tau_dist), not the residual dipole's worst case; the sized coil's authority is not 1.0.0's 0.45 A m^2 a side | — | no engine counterpart: the survey gives the total torque only (demand.tau_dist), not the residual dipole's worst case; the sized coil's authority is not 1.0.0's 0.45 A m^2 a side |  |
| ais_img_3u | `gr_1` | stated | 0.8 Metre | 0.8 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gr_2` | stated | 220 Second | 220.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `gr_3` | computed | 8.49422e-07 (SI) |  | no engine counterpart: the sizing (sizercs) sizes thrusters from the demand; 1.0.0's rows take its stated arm and Isp (gr_1, gr_2) and cf_3's 0 thrusters | — | no engine counterpart: the sizing (sizercs) sizes thrusters from the demand; 1.0.0's rows take its stated arm and Isp (gr_1, gr_2) and cf_3's 0 thrusters |  |
| ais_img_3u | `gr_4` | computed | 0 (SI) |  | no engine counterpart: the sizing (sizercs) sizes thrusters from the demand; 1.0.0's rows take its stated arm and Isp (gr_1, gr_2) and cf_3's 0 thrusters | — | no engine counterpart: the sizing (sizercs) sizes thrusters from the demand; 1.0.0's rows take its stated arm and Isp (gr_1, gr_2) and cf_3's 0 thrusters |  |
| ais_img_3u | `s1_4` | computed | [0.00599251, 0.0139825, -0.0129838] (SI) | [0.005992514033267068, 0.013982532744289823, -0.012983780405411982] | detumble_rcs's manifest assumptions.cm_offset_m | agrees | bit for bit |  |
| ais_img_3u | `vv_record_step_default` | stated | 1 | 1.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |
| ais_img_3u | `vv_run_duration_default` | stated | 600 | 600.0 | data/stated.json (the value the engine reads by node) | agrees | bit for bit |  |

