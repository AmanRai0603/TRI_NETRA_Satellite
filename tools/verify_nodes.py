#!/usr/bin/env python3
"""Node-by-node verification of the design loop and verification chain.

    python3 tools/verify_nodes.py [case ...]

For every node of matlab_sils/data/pipeline/nodes.json this checks, per case, that the node's
outputs exist and that they follow the node's rules. It does this by recomputing each decision
from the node's stored inputs, not by trusting the node's own verdict. For example, the catalogue
pick is recomputed from the datasheets, the selection from the family scores, the budgets from
their items, and every feasibility from its failing list. It writes results/NODE_VERIFICATION.md and
results/node_verification.json, and exits non-zero when a check fails.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import json, math, pathlib, sys
from common import write_text

ROOT = pathlib.Path(__file__).resolve().parents[1]
MS = ROOT / "matlab_sils"
PIPE = MS / "store" / "pipeline"
NODES = json.loads((MS / "data" / "pipeline" / "nodes.json").read_text())["nodes"]
P = {n["id"]: n["parameters"] for n in NODES}
jl = lambda p: json.loads(p.read_text()) if p.exists() else None


class Check:
    def __init__(self):
        self.rows = []

    def __call__(self, node, case, what, ok, detail=""):
        self.rows.append({"node": node, "case": case, "check": what, "ok": bool(ok), "detail": str(detail)})
        return ok


def case_req(case):
    import csv
    out = {}
    with open(MS / "cases" / f"{case}.csv", newline="") as f:
        for r in csv.DictReader(f):
            try:
                out[r["key"]] = float(r["value"])
            except (TypeError, ValueError):
                pass
    return out


def verify_catalogue(C):
    need = ("h_mNms", "torque_mNm", "mass_g", "power_steady_W")
    cat = [jl(p) for p in sorted((MS / "data" / "catalogue").glob("*.json"))]
    C("catalogue", "—", "catalogue files present", len(cat) > 0, f"{len(cat)} models")
    for c in cat:
        d = c["datasheet"]
        C("catalogue", "—", f"{c['part_number']}: source and schema", c.get("schema") == "adcs-datasheet/1" and c.get("source_url", "").startswith("http"), c.get("source_kind"))
        sel = all(d.get(k) is not None for k in need)
        C("catalogue", "—", f"{c['part_number']}: selectable iff momentum, torque, mass and steady power are stated", c["selectable"] == sel,
          "selectable" if sel else "missing " + ", ".join(k for k in need if d.get(k) is None))
        if sel:
            x = c["derived"]
            C("catalogue", "—", f"{c['part_number']}: derived values follow the datasheet",
              math.isclose(x["h_max_Nms"], d["h_mNms"] * 1e-3) and math.isclose(x["torque_max_Nm"], d["torque_mNm"] * 1e-3)
              and math.isclose(x["mass_kg"], d["mass_g"] * 1e-3) and x["power_steady_W"] == d["power_steady_W"]
              and math.isclose(x["rotor_inertia_kgm2"] * x["speed_max_rad_s"], x["h_max_Nms"]), "h, torque, mass, power, J w = h")
    return {c["part_number"]: c for c in cat}


def verify_case(C, case, cat):
    req = case_req(case)
    C("case", case, "case file with mass and volume budgets", "req.mass" in req and "req.vol" in req, f"{req.get('req.mass')} kg, {req.get('req.vol')} L")
    sel, log = jl(PIPE / case / "selection.json"), jl(PIPE / case / "loop.json")
    if not C("select", case, "selection.json and loop.json present", sel and log):
        return
    it = sel["iterations"]
    z = jl(PIPE / case / f"iter_{it}" / "sized" / "sizing.json")
    if not C("demand", case, f"sizing of the final iteration (iter_{it}) present", z):
        return
    d = z["demand"]
    C("demand", case, "momentum and torque demand positive and finite", d["h_req"] > 0 and d["tau_req"] > 0 and math.isfinite(d["h_req"]),
      f"h_req {d['h_req']:.3e} N m s, tau_req {d['tau_req']:.3e} N m")
    C("demand", case, "class from req.ake (fine when <= 0.05 deg)", (z["class"] == "fine") == (req.get("req.ake", math.inf) <= 0.05), z["class"])
    parts = z["parts"]
    for key, node in (("mtq", "size_mtq"), ("mtqp", "size_mtq"), ("fmr_x", "size_fmr"), ("fmr_y", "size_fmr"), ("fmr_z", "size_fmr"), ("rcs", "size_rcs")):
        nm = parts[key]["nominal"]
        C(node, case, f"{parts[key]['part_number']}: designed part with positive mass", nm.get("mass_kg", 0) > 0 and parts[key]["made"] == "in-house", f"{nm['mass_kg']:.4g} kg")
    fx = parts["fmr_x"]["nominal"]
    C("size_fmr", case, "electromagnetic pump (no permanent magnet), electrode current within the bound",
      "electromagnet" in fx.get("pump_type", "") and "electromagnet" in fx["pump"]["type"] and
      0 < fx["pump"]["electrode_current_design_A"] <= P["size_fmr"]["electrode_current_max_A"] + 1e-9,
      f"{fx['pump']['type']}, electrode current {fx['pump']['electrode_current_design_A']:.2f} A, lambda {fx['pump']['lambda_kg_per_W']:g} kg/W")
    # select_rotor: recompute the pick from the catalogue and the need
    for k in ("rw", "cmg", "vscmg"):
        p = parts[k]
        s = p["sizing"]
        share = 1.0 if k == "rw" else 0.5
        scale = z["knobs"]["scale"].get(k, 1.0)
        hn, tn = share * d["h_req"] * scale, share * d["tau_req"] * scale
        types = ("reaction_wheel",) if k == "rw" else ("cmg", "cmg_cluster")
        cands = [c for c in cat.values() if c["type"] in types and c["selectable"]]
        ok = sorted((c for c in cands if c["derived"]["h_max_Nms"] >= hn and c["derived"]["torque_max_Nm"] >= tn),
                    key=lambda c: (c["derived"]["mass_kg"], c["derived"]["power_steady_W"], c["derived"]["volume_L"] or 0))
        want = ok[0]["part_number"] if ok else max(cands, key=lambda c: c["derived"]["h_max_Nms"])["part_number"]
        got = p["part_number"].replace("-VSCMG", "")
        C("select_rotor", case, f"{k.upper()}: lightest selectable catalogue model meeting the need", got == want,
          f"need h {hn:.3e} N m s, tau {tn:.3e} N m per unit -> {got}" + ("" if ok else " (none meets; largest fitted)"))
        C("select_rotor", case, f"{k.upper()}: fitted part carries the datasheet's mass and power", p["made"] == "bought" and
          math.isclose(p["nominal"]["mass_kg"], cat[got]["derived"]["mass_kg"]) and p["nominal"]["power_steady_W"] == cat[got]["derived"]["power_steady_W"],
          f"{p['nominal']['mass_kg']} kg, {p['nominal']['power_steady_W']} W ({cat[got]['verification']})")
    # size_sensors and budget
    for f, B in z["families"].items():
        pr = jl(PIPE / case / f"iter_{it}" / "sized" / "products" / f"SZ-{case}-{f}.json")
        slots = [x["slot"] for x in pr["fill"]]
        C("size_sensors", case, f"{f}: sensor set (star tracker on the fine class)", all(s_ in slots for s_ in ("magnetometer", "sun_sensors", "gyro")) and
          (("star_tracker" in slots) == (z["class"] == "fine" or z["knobs"].get("star_tracker", False))), ", ".join(s_ for s_ in slots if s_ not in ("coils",)))
        C("budget", case, f"{f}: mass, power and volume are the sums of the fitted units",
          all(math.isclose(B[q], sum(x[q] for x in B["items"]), rel_tol=1e-9, abs_tol=1e-12) for q in ("mass_kg", "power_W", "volume_L")),
          f"{B['mass_kg']:.3f} kg, {B['power_W']:.2f} W, {B['volume_L']:.3f} L")
    # matrix and assess
    last = log[-1]
    modes = [jl(p) for p in sorted((MS / "data" / "modes").glob("*.json"))]
    want_tests = {(M["id"], o["id"]) for M in modes for o in M["options"]}
    got_tests = {(r["mode"], r["option"]) for r in last["matrix"]}
    C("matrix", case, "every mode x option flown in the last iteration", want_tests == got_tests, f"{len(got_tests)} of {len(want_tests)} mode tests")
    C("assess", case, "an option is feasible exactly when nothing fails", all(r["feasible"] == (not r["failing"]) for r in last["matrix"]),
      f"{sum(r['feasible'] for r in last['matrix'])} feasible")
    C("assess", case, "every failing requirement has a cause class", all(v in ("performance", "knowledge", "power", "propellant") for r in last["matrix"] for v in r["failing"].values()))
    # converge
    C("converge", case, "converged: the last iteration proposes no change", sel["converged"] and not last["changes"], f"{it} iteration(s)")
    C("converge", case, "within the iteration cap", it <= P["converge"]["max_iterations"], f"cap {P['converge']['max_iterations']}")
    sb = P["converge"]["scale_bounds"]
    C("converge", case, "every authority scale inside its bounds", all(sb[0] - 1e-9 <= v <= sb[1] + 1e-9 for v in sel["knobs"].get("scale", {}).values()),
      json.dumps(sel["knobs"].get("scale", {})))
    C("converge", case, "every family was flown and scored in every iteration", all(set(e["families"]) == set(sel["families"]) for e in log),
      f"{len(sel['families'])} families x {len(log)} iterations")
    # tune: every tuned option flew every law of its slot at every grid point, and assess kept the min-max best
    grids = P["tune"]["grids"]
    cands = P["matrix"]["algorithm_candidates"]
    rank = lambda z: (not z["feasible"], len(z["failing"]), z["objective"] if z["objective"] is not None else math.inf)
    for m_, o_ in last.get("tuned", []):
        r = next((x for x in last["matrix"] if x["mode"] == m_ and x["option"] == o_), None)
        slot = {"sun_acquisition": "sun_acquisition"}.get(m_, "mtq_pointing")
        npts = 1
        for v in grids[slot].values():
            npts *= len(v)
        want = len(cands[slot]) * (npts + 1)
        vs = (r or {}).get("variants", [])
        C("tune", case, f"{m_}/{o_}: every {slot} law flown at every grid point", len(vs) == want, f"{len(vs)} of {want} variants")
        if vs:
            best = min(vs, key=rank)
            C("tune", case, f"{m_}/{o_}: the kept variant is the best worst seed", rank(best) == rank(r), f"{r['alg']} ({r['objective_id']} {r['objective']})")
    tuned = [list(x) for x in last.get("tuned", [])]
    thin = [f"{x['mode']}/{x['option']}" for x in last["matrix"] if x["option"] == "mtq" and x["mode"] in ("nadir_pointing", "sun_referencing", "sun_acquisition")
            and x["feasible"] and isinstance(x.get("objective_req"), (int, float)) and x["objective"] is not None
            and x["objective"] > P["tune"]["margin"] * x["objective_req"] and [x["mode"], x["option"]] not in tuned]
    C("tune", case, f"every coils-only option passing with less than the {P['tune']['margin']:g} margin was tuned", not thin, ", ".join(thin) or "none left")
    # certify: every law's verdict follows from its multipliers
    fq = jl(PIPE / case / "floquet.json")
    if C("certify", case, "Floquet multipliers for every magnetic law of the coils-only family", fq and len(fq["laws"]) >= 5,
         fq and f"{len(fq['laws'])} laws; dispatched {fq['dispatched_law']}"):
        for x in fq["laws"]:
            C("certify", case, f"{x['law']}: certified exactly when every multiplier outside the free directions is inside the unit circle",
              x["certified"] == (max(x["mu_abs"][x["free_directions"]:]) < 1.0), f"max |mu| {x['max_mu']:.4g}")
        C("certify", case, "the dispatched law is among them", any(x["dispatched"] for x in fq["laws"]) or not fq["dispatched_law"])
    # select: recompute from the family scores
    F = sel["families"]
    for f, v in F.items():
        g = list(v["gaps"])
        mass_gap = v["budget"]["mass_kg"] > req["req.mass"]
        vol_gap = v["budget"]["volume_L"] > req["req.vol"]
        C("select", case, f"{f}: feasible exactly when no mode and no budget gap", v["feasible"] == (not g) and
          (mass_gap == any("mass_kg" in x for x in g)) and (vol_gap == any("volume_L" in x for x in g)), "; ".join(g) or "no gap")
    rank = lambda f: tuple(F[f]["simplicity"] if k == "simplicity" else F[f]["budget"][k] for k in P["select"]["rank_feasible"])
    for role, key in ((P["select"]["select_role"], "selected"), (P["select"]["compare_role"], "benchmark")):
        fs = [f for f in F if F[f]["role"] == role]
        feas = sorted((f for f in fs if F[f]["feasible"]), key=rank)
        want = feas[0] if feas else min(fs, key=lambda f: (len(F[f]["gaps"]), F[f]["budget"]["mass_kg"]))
        C("select", case, f"{key}: {role} family by the rule ({', '.join(P['select']['rank_feasible'])})", sel[key] == want, f"{sel[key]}")
    # dispatch
    dd = ROOT / "dist" / "dispatch" / case / sel["selected"] / "converged"
    ec = jl(dd / "engine_check.json")
    C("dispatch", case, "package: mission scenario, config blob, products", (dd / "mission_scenario.json").exists() and (dd / "fsw" / "adcs_fswcfg.bin").exists() and (dd / "sized" / "sizing.json").exists(),
      str(dd.relative_to(ROOT)))
    C("dispatch", case, "C and Rust flight software bit-identical on the mission", ec and ec["c"]["rc"] == 0 and ec["rust"]["rc"] == 0 and ec["c_equals_rust_bitwise"])
    # mc
    mc = jl(PIPE / case / "mc" / "summary.json")
    C("mc", case, f"Monte Carlo ran every run ({P['mc']['runs']})", mc and mc["runs"] == P["mc"]["runs"], mc and f"{mc['runs']} runs")
    if mc:
        bad = [s["id"] for s in mc["stats"] if s["pass"] is False]
        C("mc", case, "every requirement metric passes in every run", not bad, ", ".join(bad) or "all pass")
    # family_missions
    fm = jl(PIPE / case / "families.json")
    sol = [f for f in F if F[f]["role"] == "solution"]
    if C("family_missions", case, "every solution family flown as the mission", fm and set(fm) == set(sol), ", ".join(sorted(fm or []))):
        for f, v in fm.items():
            C("family_missions", case, f"{f}: C = Rust bit for bit", v["c_equals_rust_bitwise"])
            C("family_missions", case, f"{f}: Monte Carlo ran", v["mc"] and v["mc"]["runs"] == P["mc"]["runs"])
            met = {x["id"]: x for x in v["mission"]["c"] or []}
            fails = [i for i, x in met.items() if x.get("pass") == 0]
            C("family_missions", case, f"{f}: mission verdict matches the loop's feasibility (informational for families that are not feasible)",
              (not fails) if v["feasible"] else True, ("fails " + ", ".join(fails)) if fails else "mission passes")
    # soft_oils
    so = jl(PIPE / case / "soft_oils.json")
    if C("soft_oils", case, "SILS and soft OILS (C and Rust firmware) ran", so and all(so[k]["rc"] == 0 for k in ("sils", "oils", "oils_rs"))):
        C("soft_oils", case, "no overrun on either firmware", all(so[k]["oils"]["overruns"] == 0 for k in ("oils", "oils_rs")),
          ", ".join(f"{k}: CPU max {100 * so[k]['oils']['cpu_load_max']:.1f} %" for k in ("oils", "oils_rs")))
        sv = {m["id"]: m.get("pass") for m in so["sils"]["metrics"]}
        for k in ("oils", "oils_rs"):
            ov = {m["id"]: m.get("pass") for m in so[k]["metrics"]}
            C("soft_oils", case, f"{k}: every requirement verdict equals SILS", all(ov.get(i) == p for i, p in sv.items() if p is not None))
    # report
    C("report", case, "design ledger names the selection", (ROOT / "results" / f"DESIGN_{case}.md").exists() and
      f"`{sel['selected']}`" in (ROOT / "results" / f"DESIGN_{case}.md").read_text())


def main():
    cases = sys.argv[1:] or ["ais_3u", "ais_img_3u"]
    C = Check()
    ids = [n["id"] for n in NODES]
    C("nodes", "—", "registry: unique node ids with inputs, outputs and rules", len(ids) == len(set(ids)) and all(n["outputs"] for n in NODES), f"{len(ids)} nodes")
    cat = verify_catalogue(C)
    for c in cases:
        verify_case(C, c, cat)
    C("report", "—", "V&V report (HTML and PDF)", (ROOT / "results" / "vv" / "TRINETRA_ADCS_VV_report.html").exists() and (ROOT / "dist" / "TRINETRA_ADCS_VV_report.pdf").exists())
    rows = C.rows
    order = {n: i for i, n in enumerate(["nodes"] + ids)}
    rows.sort(key=lambda r: (order.get(r["node"], 99), r["case"]))
    npass = sum(r["ok"] for r in rows)
    covered = sorted({r["node"] for r in rows} & set(ids), key=order.get)
    L = ["# Node-by-node verification", "", "**Owner: Agastya.** Generated by `tools/verify_nodes.py`. Each check recomputes a node's decision "
         "from its stored inputs and the rules in `matlab_sils/data/pipeline/nodes.json`, rather than reading the node's own verdict.", "",
         f"**{npass} of {len(rows)} checks pass** over {len(covered)} of {len(ids)} nodes ({', '.join(cases)}).", "",
         "| node | case | check | result | detail |", "|---|---|---|---|---|"]
    L += [f"| `{r['node']}` | {r['case']} | {r['check']} | {'pass' if r['ok'] else '**FAIL**'} | {r['detail']} |" for r in rows]
    write_text(ROOT / "results" / "NODE_VERIFICATION.md", "\n".join(L) + "\n")
    write_text(ROOT / "results" / "node_verification.json", json.dumps({"passed": npass, "checks": len(rows), "nodes": covered, "rows": rows}, indent=1))
    print(f"verify_nodes: {npass}/{len(rows)} checks pass over {len(covered)}/{len(ids)} nodes; wrote results/NODE_VERIFICATION.md")
    for r in rows:
        if not r["ok"]:
            print(f"  FAIL {r['node']} [{r['case']}] {r['check']}: {r['detail']}")
    sys.exit(0 if npass == len(rows) else 1)


if __name__ == "__main__":
    main()
