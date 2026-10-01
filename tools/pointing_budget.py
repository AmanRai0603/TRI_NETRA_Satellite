#!/usr/bin/env python3
"""The absolute pointing error budget of each fine-pointing scenario (SPEC rows gp_0 to gp_5,
ECSS-E-ST-60-10C): what the flown loop shows, what the parts and the case state, and whether the
budget closes against the case's req.ape.

    gp_0 knowledge   the flown AKE (line of sight, p99.73)
    gp_1 control     the flown APE with the knowledge removed in quadrature (inferred, see below)
    gp_2 alignment   payload to star-tracker alignment: the product's `payload_alignment_rad`
    gp_3 thermal     thermal distortion between payload and sensors: the case's `pointing.et`
    gp_4 jitter      the engine's rotor-imbalance jitter (metrics::jitter, last orbit)
    gp_5 total       sqrt(flown APE^2 + gp_2^2 + gp_3^2 + gp_4^2)

The flown APE already contains knowledge and control (and the star tracker's calibrated mount
residual, which the engine flies), so gp_0 and gp_1 are not added again. gp_5 adds the terms in
quadrature, which holds only for independent random terms (SPEC risk R-15): a bias among them adds
linearly, and the page says so. A term nobody states is "not stated", and a budget with one is
"incomplete": it never closes on a guess.

    python3 tools/pointing_budget.py            fly each scenario once, write results/POINTING_BUDGET.md

Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import math
import pathlib
import subprocess
import sys
import tempfile

from common import ROOT, case_values, write_json, write_text

ENGINE = ROOT / "engine" / "target" / "release" / "adcs"
SCEN = ROOT / "matlab_sils" / "data" / "scenarios"
PRODUCTS = ROOT / "catalogue" / "products"
# the scenarios that judge req.ape on a star-tracker product
SCENARIOS = ["fine_hold_img", "fine_hold_rw_rcs", "fine_hold_cmg", "fine_hold_fmr", "fine_hold_fmr_rcs", "fine_hold_vscmg", "target_img"]
JITTER = {"id": "budget_jitter", "kind": "jitter", "window": "last_orbit",
          "diagnostic": "the jitter term of the pointing budget (gp_4)"}


def rss(terms):
    """gp_5: the root-sum-square of the terms (degrees); None when any term is not stated."""
    return None if any(t is None for t in terms) else math.sqrt(sum(t*t for t in terms))


def control_part(ape, ake):
    """gp_1 inferred from the flown loop: APE with the knowledge removed in quadrature."""
    return math.sqrt(max(ape*ape - ake*ake, 0.0))


def product_alignment(product):
    """gp_2 from the product file, in degrees; None when the product does not state it."""
    import tomllib
    f = PRODUCTS / f"{product}.toml"
    if not f.exists():
        return None
    x = tomllib.loads(f.read_text()).get("payload_alignment_rad")
    return None if x is None else math.degrees(float(x))


def fly(sid, out):
    """One run of the scenario with the jitter term added to its metrics; {metric id: value}."""
    s = json.loads((SCEN / f"{sid}.json").read_text())
    s["metrics"] = list(s.get("metrics", [])) + [JITTER]
    f = out / f"{sid}.json"
    f.write_text(json.dumps(s))
    r = subprocess.run([str(ENGINE), "run", str(f), "--case", s["case"], "-q", "--out", str(out / sid)], capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"pointing_budget: {sid} did not fly: {r.stderr.strip() or r.stdout.strip()}")
    man = json.loads((out / sid / "manifest.json").read_text())
    return s, {m["id"]: m.get("value") for m in man["metrics"]}


def row(sid, s, m):
    pick = lambda kind_ids: next((m[k] for k in kind_ids if isinstance(m.get(k), (int, float))), None)
    ape = pick([k for k in m if k.startswith("ape_los") and "p9973" in k])
    ake = pick([k for k in m if k.startswith("ake_los") and "p9973" in k])
    jit = m.get("budget_jitter")
    cv = case_values(s["case"])
    req = cv.get("req.ape")
    terms = {"gp_0": ake, "gp_1": None if ape is None or ake is None else control_part(ape, ake),
             "gp_2": product_alignment(s["product"]), "gp_3": cv.get("pointing.et"),
             "gp_4": None if not isinstance(jit, (int, float)) or math.isnan(jit) else jit/3600.0}
    total = rss([ape, terms["gp_2"], terms["gp_3"], terms["gp_4"]])
    # what req.ape leaves for alignment and thermal together, once the flown loop and the jitter are in
    room = None if req is None or ape is None or terms["gp_4"] is None else math.sqrt(max(req*req - ape*ape - terms["gp_4"]**2, 0.0))
    missing = [k for k in ("gp_2", "gp_3", "gp_4") if terms[k] is None]
    verdict = ("incomplete: " + ", ".join(missing) + " not stated") if missing else \
              ("closes" if req is not None and total <= req else "does not close")
    return {"scenario": sid, "case": s["case"], "product": s["product"], "flown_ape_deg": ape, "terms_deg": terms,
            "total_deg": total, "req_ape_deg": req, "room_gp2_gp3_deg": room, "verdict": verdict}


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
        rows = [row(sid, *fly(sid, pathlib.Path(tmp))) for sid in SCENARIOS]
    write_text(ROOT / "results" / "POINTING_BUDGET.md", markdown(rows))
    write_json(ROOT / "results" / "pointing_budget.json", rows, indent=1)
    for r in rows:
        print(f"{r['scenario']:18s} {r['verdict']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
