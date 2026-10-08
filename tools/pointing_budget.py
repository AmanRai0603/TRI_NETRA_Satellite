#!/usr/bin/env python3
"""The absolute pointing error budget of each fine-pointing scenario (SPEC rows gp_0 to gp_5,
ECSS-E-ST-60-10C): what the flown loop shows, what the parts and the case state, and whether the
budget closes against the case's req.ape.

    gp_0 knowledge   the flown AKE (line of sight, p99.73)
    gp_1 control     the flown APE with the knowledge removed in quadrature (inferred, see below)
    gp_2 alignment   payload to star-tracker alignment: the product's `payload_alignment_rad`
    gp_3 thermal     thermal distortion between payload and sensors: the case's `pointing.et`
    gp_4 jitter      the engine's rotor-imbalance jitter (gp_4's method, last orbit)
    gp_5 total       sqrt(flown APE^2 + gp_2^2 + gp_3^2 + gp_4^2)

The flown APE already contains knowledge and control (and the star tracker's calibrated mount
residual, which the engine flies), so gp_0 and gp_1 are not added again. gp_5 adds the terms in
quadrature, which holds only for independent random terms (SPEC risk R-15): a bias among them adds
linearly, and the page says so. A term nobody states is "not stated", and a budget with one is
"incomplete": it never closes on a guess.

The terms, the total, the room and the verdict are pnt's methods (gp_0, gp_1, gp_2, gp_4, l3_pnt_row_09:
design/revisions/S7.14/pntbudget.pc), generated into the engine: this tool flies each scenario, names
the run's metrics the budget takes, and asks the engine for the budget (`adcs results budget`).

    python3 tools/pointing_budget.py            fly each scenario once, write results/POINTING_BUDGET.md

Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import subprocess
import sys
import tempfile

from common import ROOT, write_json, write_text

ENGINE = ROOT / "engine" / "target" / "release" / "adcs"
SCEN = ROOT / "matlab_sils" / "data" / "scenarios"
# the scenarios that judge req.ape on a star-tracker product
SCENARIOS = ["fine_hold_img", "fine_hold_rw_rcs", "fine_hold_cmg", "fine_hold_fmr", "fine_hold_fmr_rcs", "fine_hold_vscmg", "target_img"]
JITTER = {"id": "budget_jitter", "kind": "jitter", "window": "last_orbit",
          "diagnostic": "the jitter term of the pointing budget (gp_4)"}


def fly(sid, out):
    """One run of the scenario with the jitter term added to its metrics; its folder."""
    s = json.loads((SCEN / f"{sid}.json").read_text())
    s["metrics"] = list(s.get("metrics", [])) + [JITTER]
    f = out / f"{sid}.json"
    f.write_text(json.dumps(s))
    r = subprocess.run([str(ENGINE), "run", str(f), "--case", s["case"], "-q", "--out", str(out / sid)], capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"pointing_budget: {sid} did not fly: {r.stderr.strip() or r.stdout.strip()}")
    return out / sid


def row(run):
    """The run's budget as the engine gives it (`adcs results budget`): the flown APE and AKE are the run's first
    computed metrics across the boresight at p99.73, the jitter the term added above."""
    m = {x["id"]: x.get("value") for x in json.loads((run / "manifest.json").read_text())["metrics"]}
    pick = lambda pre: next((k for k in m if k.startswith(pre) and "p9973" in k and isinstance(m[k], (int, float))), None)
    names = [("--flown", pick("ape_los")), ("--knowledge", pick("ake_los")), ("--jitter", "budget_jitter")]
    r = subprocess.run([str(ENGINE), "results", "budget", str(run), *[a for k, v in names if v for a in (k, v)]], capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"pointing_budget: the engine gave no budget for {run.name}: {r.stderr.strip() or r.stdout.strip()}")
    b = json.loads(r.stdout)
    if b["verdict"] == "incomplete":
        b["verdict"] = "incomplete: " + ", ".join(k for k in ("gp_2", "gp_3", "gp_4") if b["terms_deg"][k] is None) + " not stated"
    return {k: b[k] for k in ("scenario", "case", "product", "flown_ape_deg", "terms_deg", "total_deg", "req_ape_deg", "room_gp2_gp3_deg", "verdict")}


def markdown(rows):
    f = lambda x: "not stated" if x is None else f"{x:.5f}"
    L = ["# Pointing error budget (gp_0 to gp_5)", "",
         "> **Kind:** generated (`python3 tools/pointing_budget.py`): each scenario flown once on today's engine.", "",
         "The flown APE holds knowledge and control together (and the star tracker's calibrated mount residual), so the "
         "total adds to it only what the loop does not fly: payload alignment (gp_2, the product's `payload_alignment_rad`), "
         "thermal distortion (gp_3, the case's `pointing.et`) and rotor jitter (gp_4). Terms add in quadrature, which holds "
         "for independent random terms only (SPEC risk R-15); a bias adds linearly. A term nobody states keeps the budget "
         "incomplete. The room column is what req.ape leaves for alignment and thermal together, "
         "sqrt(req² − APE² − gp_4²): the allocation those two terms must fit. Degrees, p99.73.", "",
         "| scenario | product | gp_0 knowledge | gp_1 control (inferred) | flown APE | gp_2 alignment | gp_3 thermal | gp_4 jitter | gp_5 total | req.ape | room for gp_2 + gp_3 | verdict |",
         "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"]
    for r in rows:
        t = r["terms_deg"]
        L.append(f"| {r['scenario']} | {r['product']} | {f(t['gp_0'])} | {f(t['gp_1'])} | {f(r['flown_ape_deg'])} | {f(t['gp_2'])} | "
                 f"{f(t['gp_3'])} | {f(t['gp_4'])} | {f(r['total_deg'])} | {f(r['req_ape_deg'])} | {f(r['room_gp2_gp3_deg'])} | {r['verdict']} |")
    return "\n".join(L) + "\n"


def main(argv=None):
    if not ENGINE.exists():
        raise SystemExit(f"pointing_budget: no engine at {ENGINE} (cd engine && cargo build --release)")
    with tempfile.TemporaryDirectory() as tmp:
        rows = [row(fly(sid, pathlib.Path(tmp))) for sid in SCENARIOS]
    write_text(ROOT / "results" / "POINTING_BUDGET.md", markdown(rows))
    write_json(ROOT / "results" / "pointing_budget.json", rows, indent=1)
    for r in rows:
        print(f"{r['scenario']:18s} {r['verdict']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
