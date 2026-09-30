"""The engine against the MATLAB twin, scenario by scenario and metric by metric.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import json, math
from common import Steps, write_text
from engine_base import ENG, OUT, TWIN
from engine_runs import scenarios


# MATLAB campaign <-> engine Monte Carlo of the same scenario (tools/engine.py mc <scenario>)
MC_PAIRS = {"mc_detumble_ais": "detumble_ais", "mc_nadir_ais": "nadir_hold_ais", "mc_fine_img": "fine_hold_img", "mc_slew_img": "slew_img"}


# Differences that were traced to their cause (kept with the ledger).
NOTES = [
    ("fine_hold_fmr, fine_hold_fmr_rcs · power_mean",
     "Knife-edge in the fluid-ring power model, not an engine difference: the 2 W field power per ring is on while the "
     "driver's momentum target exceeds 2 % of h_max (2e-5 N m s). Both runs start with all three rings on (6 W); the "
     "ring momenta then hover at 2.3-2.6e-5 N m s, so which side of the threshold each run settles on decides a 2 W "
     "requirement. Action: a hysteresis band on the field-power switch (asils.devices.mex and the engine's Mex)."),
    ("fine_hold_rw_rcs · ape_los_p9973",
     "Traced: the momentum-dump pulses (same times, same counts, same propellant on both sides) are fed forward with "
     "the nominal thruster torque; the unit's thrust-scale draw (sigma 3 %) leaves an uncompensated torque for the "
     "~30 s dump. Engine draw 0.985 -> ~3e-6 N m -> 0.03 deg offset against Kp = 5.4e-3; MATLAB draw 0.995 -> 0.01 deg. "
     "Engine Monte Carlo (24 seeds): mean 0.034 deg, range 0.009-0.079 deg; the MATLAB run sits at the lucky end. "
     "Finding: RCS dumping during imaging breaks the 0.01 deg APE for most thrust-scale draws -> calibrate thrust "
     "on orbit or inhibit RCS dumping in the fine modes."),
    ("sun_spin_ais · sun_angle, spin_rate_error, share",
     "Traced to the flight logic (identical in MATLAB, C and Rust): when spin-up converges with the Sun on +Z, the "
     "sigma flip reverses the spin every 60 s check, which drives the body through zero spin and never moves the Sun "
     "to -Z. Engine seed 1 is that case; 3 of 24 Monte Carlo seeds lock up (mean sun angle 26 deg, range 2.7-173 deg), "
     "the MATLAB run (6.5 deg) is inside. Finding: the sign-flip guard needs a hemisphere manoeuvre, not a spin reversal."),
    ("nadir_hold_ais, mission_ais · ake_los / ape_los",
     "Realisation spread of the coils-only MEKF (magnetometer + Sun): engine 24-seed AKE mean 4.0 deg (0.8-8.5 deg); "
     "the MATLAB runs (3.9-4.0 deg) are inside the distribution."),
    ("detumble_* and mission_* · detumble_time",
     "Random initial tumble direction; at distribution level the MATLAB campaign (48.5 +/- 12 min) and the engine "
     "campaign mc_detumble_ais with the full dispersions (56.3 +/- 11.4 min) overlap; results/ENGINE_CAMPAIGNS.md."),
    ("mc_nadir_ais · ape_los",
     "Like-for-like since the engine campaigns (tools/engine.py campaign, results/ENGINE_CAMPAIGNS.md) disperse the "
     "residual dipole, inertia, CM offset, flux, Kp, accommodation and reflectivity exactly as asils.campaign.draw: "
     "MATLAB mean 89.7 deg vs engine 93.0 deg; the coils-only nadir hold cannot absorb the dispersed dipole."),
]


def twin_parity(_):
    """Engine (Rust) vs the MATLAB twin, same scenario, same case, same product.
    One realisation each: the random streams differ (MATLAB twister vs counter-based
    SplitMix64), so a value-by-value match is not expected; the ledger records the
    verdict agreement and the ratio, and the model differences named in fsw/twin_map.toml."""
    S = Steps("engine.py", "twin-parity")
    S(1)
    rows = []
    for s in scenarios([]):
        a, b = TWIN / s / "manifest.json", ENG / s / "manifest.json"
        if not (a.exists() and b.exists()):
            continue
        ma, mb = json.loads(a.read_text()), json.loads(b.read_text())
        lst = lambda x: x if isinstance(x, list) else [x]
        mb_by = {m["id"]: m for m in lst(mb["metrics"])}
        for m in lst(ma["metrics"]):
            e = mb_by.get(m["id"])
            if not e:
                continue
            va, vb = m.get("value"), e.get("value")
            fin = lambda x: isinstance(x, (int, float)) and math.isfinite(x)
            ratio = vb / va if fin(va) and fin(vb) and va != 0 else None
            rows.append({"scenario": s, "metric": m["id"], "unit": m.get("unit", ""), "matlab": va if fin(va) else None,
                         "engine": vb if fin(vb) else None, "ratio": ratio, "req": m.get("req"),
                         "pass_matlab": m.get("pass"), "pass_engine": e.get("pass"),
                         "agree": (m.get("pass") == e.get("pass")) if m.get("pass") is not None else None,
                         "wall_matlab_s": ma.get("wall_s"), "wall_engine_s": mb.get("wall_s")})
    S(2, f"{len(rows)} metric pairs")
    OUT.mkdir(exist_ok=True)
    write_text(OUT / "engine_parity.json", json.dumps(rows, indent=1))
    judged = [r for r in rows if r["agree"] is not None]
    agree = sum(r["agree"] for r in judged)
    walls = {}
    for r in rows:
        walls[r["scenario"]] = (r["wall_matlab_s"], r["wall_engine_s"])
    fmt = lambda x: "—" if x is None else (f"{x:.4g}" if isinstance(x, float) else str(x))
    verdict = lambda p: {1: "pass", 0: "FAIL"}.get(p, "—")
    L = ["# Engine parity: Rust engine vs MATLAB twin", "",
         "Owner: Agastya. Generated by `tools/engine.py twin-parity` from the filed runs:",
         "`matlab_sils/store/results/<scenario>` (MATLAB twin, POP in the loop) and",
         "`matlab_sils/store/results_engine/<scenario>` (Rust engine, C flight software behind the byte HAL).", "",
         "The engine steps the Rust port of POP inside the loop exactly as the twin steps the MATLAB POP",
         "(adcs-pop: time scales, frames, DE440, degree-6 field, Battin third body, DTM2020 drag, conical",
         "SRP, RK4 10 s + Hermite): orbit, Sun, Moon, shadow, density and field are bit-identical to the",
         "twin's filed channels (see 'Truth environment' below). What still differs is the random stream",
         "(MATLAB Mersenne twister vs the engine's counter-based SplitMix64), so single runs are",
         "compared by metric ratio and verdict agreement, and Monte Carlo on both sides compares",
         "distributions (`tools/engine.py mc`).", "",
         f"**Verdict agreement: {agree} of {len(judged)} judged metrics** "
         f"({len(rows)} metrics over {len(walls)} scenarios).", "",
         "| scenario | metric | MATLAB | engine | engine/MATLAB | req | MATLAB | engine |",
         "|---|---|---:|---:|---:|---:|---|---|"]
    for r in rows:
        flag = "" if r["agree"] in (None, True) else " ⚠"
        L.append(f"| {r['scenario']} | {r['metric']} ({r['unit']}) | {fmt(r['matlab'])} | {fmt(r['engine'])} | {fmt(r['ratio'])} | "
                 f"{fmt(r['req'])} | {verdict(r['pass_matlab'])} | {verdict(r['pass_engine'])}{flag} |")
    L += ["", "## Truth environment: engine vs MATLAB twin, every recorded sample", "",
          "| scenario | samples | max |r| difference [m] | max density ratio - 1 | max Sun-direction difference | max |B| ratio - 1 |", "|---|---:|---:|---:|---:|---:|"]
    import csv
    for s in scenarios([]):
        a, b = TWIN / s / "channels.csv", ENG / s / "channels.csv"
        if not (a.exists() and b.exists()):
            continue
        A, B = list(csv.DictReader(open(a))), list(csv.DictReader(open(b)))
        wr = wrho = wsun = wb = 0.0
        for x, y in zip(A, B):
            f = lambda d, k: float(d[k])
            wr = max(wr, sum((f(x, f"r_{k}_m") - f(y, f"r_{k}_m")) ** 2 for k in "xyz") ** 0.5)
            wrho = max(wrho, abs(f(x, "rho_kgm3") / f(y, "rho_kgm3") - 1) if f(y, "rho_kgm3") else 0)
            wsun = max(wsun, sum((f(x, f"sun_{k}") - f(y, f"sun_{k}")) ** 2 for k in "xyz") ** 0.5)
            na = sum(f(x, f"B_{k}_T") ** 2 for k in "xyz") ** 0.5; nb = sum(f(y, f"B_{k}_T") ** 2 for k in "xyz") ** 0.5
            wb = max(wb, abs(na / nb - 1))
        L.append(f"| {s} | {min(len(A), len(B))} | {wr:.3g} | {wrho:.3g} | {wsun:.3g} | {wb:.3g} |")
    L += ["", "Channels are filed with 9 significant digits, so differences below ~1e-8 relative are the",
          "file's rounding, not the models.", ""]
    L += ["", "## Distributions: MATLAB campaigns vs engine Monte Carlo", "",
          "MATLAB campaigns also disperse inertia, residual dipole, CM offset, solar flux, accommodation and",
          "reflectivity; the engine Monte Carlo disperses the units and the initial state only.", "",
          "| MATLAB campaign | engine scenario | metric | MATLAB mean ± std [min, max] (n) | engine mean ± std [min, max] (n) |",
          "|---|---|---|---|---|"]
    for camp, scen in MC_PAIRS.items():
        a, b = TWIN / camp / "summary.json", ENG / f"mc_{scen}" / "summary.json"
        if not (a.exists() and b.exists()):
            continue
        sa, sb = json.loads(a.read_text()), json.loads(b.read_text())
        for m in (sa["stats"] if isinstance(sa["stats"], list) else [sa["stats"]]):
            e = sb["stats"].get(m["id"])
            if not e or m.get("mean") is None:
                continue
            L.append(f"| {camp} | {scen} | {m['id']} | {m['mean']:.4g} ± {m['std']:.3g} [{m['min']:.4g}, {m['max']:.4g}] ({m.get('n_valid', '')}) | "
                     f"{e['mean']:.4g} ± {e['std']:.3g} [{e['min']:.4g}, {e['max']:.4g}] ({e['n']}) |")
    L += ["", "## Differences traced to their cause", ""]
    L += [f"- **{k}.** {v}" for k, v in NOTES]
    L += ["", "## Wall time (one core each)", "", "| scenario | MATLAB/Octave [s] | engine [s] | speed-up |", "|---|---:|---:|---:|"]
    for s, (wa, wb) in walls.items():
        if wa and wb:
            L.append(f"| {s} | {wa:.0f} | {wb:.2f} | {wa / wb:.0f}x |")
    write_text(OUT / "ENGINE_PARITY.md", "\n".join(L) + "\n")
    print(f"verdict agreement {agree}/{len(judged)}; wrote results/ENGINE_PARITY.md")
