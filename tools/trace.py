#!/usr/bin/env python3
"""The requirements traceability matrix: every requirement a case states, what checks it, and
what the latest stored result says.

A stated requirement (a `req.*` row of a case with a value) is checked in one of four ways:
  flight   a scenario flown on that case judges a metric against it (requirement = "req.x");
           the latest stored run of that scenario gives the value and the verdict
  design   the design loop's selection keeps the selected family's budget within it
           (req.mass, req.vol: store/pipeline/<case>/selection.json)
  modes    the design loop flies each mission mode with the selected family and judges the
           mode's metrics (catalogue/modes) against the case: store/pipeline/<case>/selection.json
  profile  the reference slew: a slew scenario commands mission.sangle in no more than req.slew
           (its success is then judged by req.settle and req.ape in the same scenario)
A stated requirement with none of these is owed, and named. Every shipped metric is decided:
it judges (a requirement or a scenario limit) or says why it only reports (diagnostic).

    python3 tools/trace.py           write results/TRACEABILITY.md and results/traceability.json
    python3 tools/trace.py --check   refuse (exit 1) an undecided metric, a requirement key the case
                                     does not have, or a stated requirement nothing checks

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import math
import sys

from common import ROOT, case_rows, write_json, write_text

DATA = ROOT / "matlab_sils" / "data"
STORE = ROOT / "matlab_sils" / "store"
DESIGN = {"req.mass": "mass_kg", "req.vol": "volume_L"}


def cases():
    return sorted(p.stem for p in (ROOT / "matlab_sils" / "cases").glob("*.csv") if p.stem != "case_template")


def scenarios():
    return [json.loads(p.read_text()) for p in sorted((DATA / "scenarios").glob("*.json"))]


def stated(case):
    """Every req.* row of the case: key -> (label, unit, value or None, note)."""
    out = {}
    for r in case_rows(case):
        if r.get("section") == "req":
            v = r.get("value", "").strip()
            out[r["key"]] = (r.get("label", ""), r.get("unit", ""), float(v) if v else None, r.get("note", ""))
    return out


def latest(scenario):
    """The stored run of a scenario (store/results_engine/<id>), its metrics by id and its provenance."""
    p = STORE / "results_engine" / scenario / "manifest.json"
    if not p.exists():
        return None
    m = json.loads(p.read_text())
    ms = m.get("metrics") or []
    return {"metrics": {x["id"]: x for x in (ms if isinstance(ms, list) else [ms])}, "created": m.get("created_utc", ""),
            "engine_source": m.get("engine_source")}


def modes():
    return {p.stem: json.loads(p.read_text()) for p in sorted((DATA / "modes").glob("*.json"))}


_sel = {}


def selected(case):
    """The design loop's selected family for the case (its budget and each mode's flown metrics), or None."""
    if case not in _sel:
        p = STORE / "pipeline" / case / "selection.json"
        sel = json.loads(p.read_text()) if p.exists() else None
        _sel[case] = dict(sel["families"][sel["selected"]], id=sel["selected"]) if sel and sel.get("selected") in sel.get("families", {}) else None
    return _sel[case]


def mission(case, key):
    for r in case_rows(case):
        if r.get("key") == key and r.get("value", "").strip():
            return float(r["value"])
    return None


def build():
    scen = scenarios()
    undecided, unknown_key = [], []
    for s in scen:
        for m in s.get("metrics", []):
            judges = "requirement" in m or "limit" in m
            if not judges and not str(m.get("diagnostic", "")).strip():
                undecided.append(f"{s['id']}/{m['id']}")
    rows = []
    for case in cases():
        reqs = stated(case)
        on = [s for s in scen if s.get("case") == case]
        for s in on:
            for m in s.get("metrics", []):
                if "requirement" in m and m["requirement"] not in reqs:
                    unknown_key.append(f"{s['id']}/{m['id']}: {m['requirement']}")
        for key, (label, unit, value, note) in reqs.items():
            row = {"case": case, "key": key, "label": label, "unit": unit, "value": value,
                   "unconfirmed": "UNCONFIRMED" in note, "flight": [], "design": None, "modes": [], "profile": []}
            if value is None:
                row["status"] = "not stated"
                rows.append(row)
                continue
            for s in on:
                for m in s.get("metrics", []):
                    if m.get("requirement") != key:
                        continue
                    run = latest(s["id"])
                    got = run["metrics"].get(m["id"]) if run else None
                    row["flight"].append({"scenario": s["id"], "metric": m["id"], "value": got.get("value") if got else None,
                                          "pass": got.get("pass") if got else None, "run": run["created"] if run else None})
            if key in DESIGN and selected(case):
                v = selected(case).get("budget", {}).get(DESIGN[key])
                row["design"] = {"family": selected(case)["id"], "value": v, "pass": None if v is None else int(v <= value)}
            for mode, mdef in modes().items():
                for m in mdef.get("metrics", []):
                    if m.get("requirement") != key:
                        continue
                    got = selected(case).get("modes", {}).get(mode, {}).get("metrics", {}).get(m["id"]) if selected(case) else None
                    ok = None if got is None else int(got >= value if m.get("sense") == "min" else got <= value)
                    row["modes"].append({"mode": mode, "metric": m["id"], "value": got, "pass": ok})
            if key == "req.slew":
                ang = mission(case, "mission.sangle")
                for s in on:
                    g = s.get("fsw", {}).get("guidance", {})
                    if g.get("kind") == "slew" and ang is not None and g.get("roll_deg") == ang:
                        row["profile"].append({"scenario": s["id"], "angle_deg": ang, "T_s": g.get("T_s"), "pass": int(g.get("T_s", math.inf) <= value)})
            verdicts = [f["pass"] for f in row["flight"]] + ([row["design"]["pass"]] if row["design"] else []) + [m["pass"] for m in row["modes"]] + [p["pass"] for p in row["profile"]]
            if not verdicts:
                row["status"] = "owed: nothing checks it"
            elif any(v is None for v in verdicts):
                row["status"] = "not yet flown"
            elif all(v == 1 for v in verdicts):
                row["status"] = "met"
            else:
                row["status"] = f"not met ({sum(1 for v in verdicts if v != 1)} of {len(verdicts)})"
            rows.append(row)
    diag = [{"scenario": s["id"], "metric": m["id"], "why": m["diagnostic"]} for s in scen for m in s.get("metrics", []) if "diagnostic" in m]
    return {"rows": rows, "undecided": undecided, "unknown_key": unknown_key, "diagnostics": diag,
            "counts": {"metrics": sum(len(s.get("metrics", [])) for s in scen), "judged": sum(1 for s in scen for m in s.get("metrics", []) if "requirement" in m or "limit" in m),
                       "diagnostic": len(diag)}}


def fmt(v):
    return "—" if v is None else (f"{v:.4g}" if isinstance(v, float) else str(v))


def markdown(t):
    c = t["counts"]
    stated_rows = [r for r in t["rows"] if r["status"] != "not stated"]
    owed = [r for r in stated_rows if r["status"].startswith("owed")]
    L = ["# Requirements traceability", "",
         f"> **Answer first.** {len(stated_rows)} stated requirements across {len(cases())} cases; "
         f"{sum(1 for r in stated_rows if r['status'] == 'met')} met, "
         f"{sum(1 for r in stated_rows if r['status'].startswith('not met'))} not met, "
         f"{sum(1 for r in stated_rows if r['status'] == 'not yet flown')} not yet flown, {len(owed)} with nothing checking them. "
         f"Of {c['metrics']} shipped metrics, {c['judged']} judge and {c['diagnostic']} report with a stated reason.", "",
         "> **Kind:** generated (`python3 tools/trace.py`) · **Source:** the cases, the scenarios, the latest stored runs, the design loop's selection", "",
         "A requirement marked † is UNCONFIRMED in its case (a plan value a person has not confirmed).", ""]
    for case in cases():
        L += [f"## {case}", "", "| requirement | value | checked by | latest | status |", "|---|---|---|---|---|"]
        for r in [r for r in t["rows"] if r["case"] == case]:
            if r["status"] == "not stated":
                continue
            by, got = [], []
            for f in r["flight"]:
                by.append(f"flight `{f['scenario']}` / {f['metric']}")
                got.append(f"{fmt(f['value'])} {'✅' if f['pass'] == 1 else '❌' if f['pass'] == 0 else '·'}")
            if r["design"]:
                d = r["design"]
                by.append(f"design: selected `{d['family']}` budget")
                got.append(f"{fmt(d['value'])} {'✅' if d['pass'] == 1 else '❌' if d['pass'] == 0 else '·'}")
            for m in r["modes"]:
                by.append(f"design loop: {m['mode']} / {m['metric']}")
                got.append(f"{fmt(m['value'])} {'✅' if m['pass'] == 1 else '❌' if m['pass'] == 0 else '·'}")
            for p in r["profile"]:
                by.append(f"profile `{p['scenario']}`: {fmt(p['angle_deg'])}° in {fmt(p['T_s'])} s")
                got.append("✅" if p["pass"] == 1 else "❌")
            L.append(f"| {r['label']} (`{r['key']}`){' †' if r['unconfirmed'] else ''} | {fmt(r['value'])} {r['unit']} | "
                     f"{'<br>'.join(by) or '—'} | {'<br>'.join(got) or '—'} | {r['status']} |")
        blank = [r["key"] for r in t["rows"] if r["case"] == case and r["status"] == "not stated"]
        L += ["", f"Not stated by the case (nothing to check): {', '.join(f'`{k}`' for k in blank) or 'none'}.", ""]
    L += ["## Metrics that report without judging", "", "| scenario | metric | why |", "|---|---|---|"]
    L += [f"| `{d['scenario']}` | {d['metric']} | {d['why']} |" for d in t["diagnostics"]]
    return "\n".join(L) + "\n"


def problems(t):
    bad = [f"metric decides nothing (give a requirement, a limit, or diagnostic = \"why\"): {x}" for x in t["undecided"]]
    bad += [f"requirement the case does not have: {x}" for x in t["unknown_key"]]
    bad += [f"stated requirement nothing checks: {r['case']} {r['key']}" for r in t["rows"] if r["status"].startswith("owed")]
    return bad


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--check", action="store_true", help="refuse undecided metrics and unchecked requirements; write nothing")
    a = ap.parse_args(argv)
    t = build()
    bad = problems(t)
    if a.check:
        for b in bad:
            print("REFUSED", b)
        print(f"trace: {len(t['rows'])} requirement rows, {t['counts']['metrics']} metrics, {len(bad)} problem(s)")
        return 1 if bad else 0
    write_text(ROOT / "results" / "TRACEABILITY.md", markdown(t))
    write_json(ROOT / "results" / "traceability.json", t, indent=1)
    for b in bad:
        print("owed:", b)
    return 0


if __name__ == "__main__":
    sys.exit(main())
