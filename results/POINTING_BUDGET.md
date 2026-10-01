# Pointing error budget (gp_0 to gp_5)

> **Kind:** generated (`python3 tools/pointing_budget.py`): each scenario flown once on today's engine.

The flown APE holds knowledge and control together (and the star tracker's calibrated mount residual), so the total adds to it only what the loop does not fly: payload alignment (gp_2, the product's `payload_alignment_rad`), thermal distortion (gp_3, the case's `pointing.et`) and rotor jitter (gp_4). Terms add in quadrature, which holds for independent random terms only (SPEC risk R-15); a bias adds linearly. A term nobody states keeps the budget incomplete. The room column is what req.ape leaves for alignment and thermal together, sqrt(req² − APE² − gp_4²): the allocation those two terms must fit. Degrees, p99.73.

| scenario | product | gp_0 knowledge | gp_1 control (inferred) | flown APE | gp_2 alignment | gp_3 thermal | gp_4 jitter | gp_5 total | req.ape | room for gp_2 + gp_3 | verdict |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| fine_hold_img | TRN-P-3U-IMG | 0.00284 | 0.00626 | 0.00687 | not stated | not stated | 0.00009 | not stated | 0.01000 | 0.00726 | incomplete: gp_2, gp_3 not stated |
| fine_hold_rw_rcs | TRN-P-3U-RW-RCS | 0.00283 | 0.00619 | 0.00681 | not stated | not stated | 0.00009 | not stated | 0.01000 | 0.00732 | incomplete: gp_2, gp_3 not stated |
| fine_hold_cmg | TRN-P-3U-CMG | 0.00285 | 0.00437 | 0.00522 | not stated | not stated | not stated | not stated | 0.01000 | not stated | incomplete: gp_2, gp_3, gp_4 not stated |
| fine_hold_fmr | TRN-P-3U-FMR | 0.00287 | 0.00523 | 0.00596 | not stated | not stated | 0.00000 | not stated | 0.01000 | 0.00803 | incomplete: gp_2, gp_3 not stated |
| fine_hold_fmr_rcs | TRN-P-3U-FMR-RCS | 0.00287 | 0.00523 | 0.00596 | not stated | not stated | 0.00000 | not stated | 0.01000 | 0.00803 | incomplete: gp_2, gp_3 not stated |
| fine_hold_vscmg | TRN-P-3U-VSCMG | 0.00285 | 0.00787 | 0.00837 | not stated | not stated | not stated | not stated | 0.01000 | not stated | incomplete: gp_2, gp_3, gp_4 not stated |
| target_img | TRN-P-3U-IMG | 0.01149 | 0.00370 | 0.01207 | not stated | not stated | 0.00009 | not stated | 0.01000 | 0.00000 | incomplete: gp_2, gp_3 not stated |
