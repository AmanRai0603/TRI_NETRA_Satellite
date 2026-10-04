#!/usr/bin/env python3
"""Both cases through everything, from the design database (docs/RELEASE_PLAN.md P13,
docs/END_TO_END.md): the case's rows from design.tndb, the design loop (sizing, the mode matrix,
convergence, selection, the selected family's Monte Carlo, dispatch, soft OILS), the case's
campaigns, every row evaluated or shown as not computed, every KPI closure answered or blocked by
name, and the traceability, every engine run reading its inputs from the database alone
(TRINETRA_DESIGN), and every number held to the one flown from the files.

    python3 tools/end_to_end.py [CASE ...] [--design DIR] [--no-loop] [--no-campaigns]

DIR is a design folder (default build/end_to_end/Design, seeded and carried over when it does not
exist). The run is refused when the database's engine inputs are not the data folder's own bytes
(tools/design_inputs.py differences): then the two could give different numbers.

What it writes: results/END_TO_END.md and end_to_end.json (what ran, from where, and every number
that differs from the one stored before, by name: none, when the database is the files), and,
through the tools it runs, the store, results/EVALUATION.md and results/TRACEABILITY.md.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import math
import os
import pathlib
import subprocess
import sys

import design_inputs
from common import ROOT, write_text

STORE = ROOT / "matlab_sils" / "store"
PY = sys.executable


def campaigns_of(case):
    out = []
    for f in sorted((ROOT / "matlab_sils" / "data" / "campaigns").glob("*.json")):
        c = json.loads(f.read_text())
        if c.get("case") == case:
            out.append(f.stem)
    return out


def numbers(case):
    """{name: value} of every number the case's stored results give: the design loop's selection and
    budgets, its Monte Carlo and the case's campaigns, statistic by statistic."""
    out = {}

    def flat(prefix, v):
        if isinstance(v, bool) or v is None:
            return
        if isinstance(v, (int, float)):
            if math.isfinite(v):
                out[prefix] = v
        elif isinstance(v, dict):
            for k, x in v.items():
                if k not in ("wall_s", "created_utc", "at", "values"):
                    flat(f"{prefix}.{k}", x)
    sel = STORE / "pipeline" / case / "selection.json"
    if sel.is_file():
        s = json.loads(sel.read_text())
        out[f"{case}.selected"] = s.get("selected")
        flat(f"{case}.selection", {k: v for k, v in s.items() if k != "selected"})
    for f in [STORE / "pipeline" / case / "mc" / "summary.json"] + [STORE / "results_engine" / "campaigns" / c / "summary.json" for c in campaigns_of(case)]:
        if f.is_file():
            s = json.loads(f.read_text())
            for st in s.get("stats", []):
                flat(f"{f.parent.name}:{st['id']}", {k: st.get(k) for k in ("mean", "std", "min", "max", "pass_rate")})
    return out


def run(cmd, env, what):
    print(f"end_to_end: {what}: {' '.join(cmd)}", flush=True)
    r = subprocess.run(cmd, cwd=ROOT, env=env)
    return {"what": what, "command": " ".join(cmd), "exit": r.returncode}


def manifests_from(db, case):
    """(runs naming the database, runs not naming it) among the case's runs written by this run."""
    import sqlite3
    with sqlite3.connect(f"file:{db}?mode=ro", uri=True) as c:
        fp = dict(c.execute('SELECT "key", "value" FROM meta'))["inputs_fingerprint"]
    named, other = 0, []
    roots = [STORE / "pipeline" / case] + [STORE / "results_engine" / "campaigns" / c for c in campaigns_of(case)]
    for r in roots:
        for m in r.rglob("manifest.json"):
            try:
                x = json.loads(m.read_text())
            except ValueError:
                continue
            if x.get("case") != case or "engine_source" not in x:
                continue
            if ((x.get("inputs") or {}).get("design") or {}).get("fingerprint") == fp:
                named += 1
            else:
                other.append(str(m.relative_to(ROOT)))
    return named, other


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cases", nargs="*")
    ap.add_argument("--design", default=str(ROOT / "build" / "end_to_end" / "Design"))
    ap.add_argument("--no-loop", action="store_true", help="leave the design loop out")
    ap.add_argument("--no-campaigns", action="store_true", help="leave the case's campaigns out")
    ap.add_argument("--jobs", default="4")
    a = ap.parse_args(argv)
    cases = a.cases or ["ais_3u", "ais_img_3u"]
    d = pathlib.Path(a.design)
    if not (d / "design.tndb").exists():
        import carry_over
        import seed_design
        seed_design.seed(d, sync=False)
        carry_over.carry(d)
    db = d / "design.tndb"
    diff = design_inputs.differences(db)
    if diff:
        for x in diff[:20]:
            print("end_to_end: " + x, file=sys.stderr)
        print(f"end_to_end: {db} does not hold the data folder's inputs; rebuild it (python3 tools/seed_design.py) so both give the same numbers", file=sys.stderr)
        return 2
    env = {**os.environ, "TRINETRA_DESIGN": str(db)}
    rec = {"design": str(db), "cases": {}}
    for case in cases:
        before = numbers(case)
        steps = []
        if not a.no_loop:
            steps.append(run([PY, "tools/pipeline.py", case, "--jobs", a.jobs], env, f"{case}: the design loop"))
        else:
            steps.append({"what": f"{case}: the design loop", "exit": None, "skipped": True})
        if not a.no_campaigns and campaigns_of(case):
            steps.append(run([PY, "tools/engine.py", "campaign", *campaigns_of(case), "--jobs", a.jobs], env, f"{case}: its campaigns"))
        elif campaigns_of(case):
            steps.append({"what": f"{case}: its campaigns", "exit": None, "skipped": True})
        after = numbers(case)
        changed = sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))
        named, other = manifests_from(db, case)
        rec["cases"][case] = {"steps": steps, "numbers": len(after), "changed": [{"name": k, "before": before.get(k), "after": after.get(k)} for k in changed],
                              "runs_from_the_database": named, "runs_not": other[:50], "runs_not_count": len(other)}
    rec["evaluate"] = run([PY, "tools/evaluate.py", str(d), *cases], env, "every row and every closure")
    rec["trace"] = run([PY, "tools/trace.py"], env, "the traceability")
    ev = json.loads((ROOT / "results" / "evaluation.json").read_text())
    L = ["# Both cases end to end, from the design database", "",
         f"**In one line:** {', '.join(cases)} through the design loop and their campaigns with every engine run reading its inputs from "
         f"`{db.relative_to(ROOT) if db.is_relative_to(ROOT) else db}` alone; every number is held to the one flown from the files, and every "
         "row and KPI closure is evaluated (`tools/end_to_end.py`, `docs/END_TO_END.md`).", "",
         "| Case | Steps | Runs from the database | Numbers compared | Numbers that differ | Rows with a value | Closures answered | Closures blocked |",
         "|---|---|---|---|---|---|---|---|"]
    for case, r in rec["cases"].items():
        e = next(x for x in ev if x["case"] == case)
        valued = sum(1 for x in e["rows"] if x["state"] != "not computed")
        ans = sum(1 for c in e["closures"] if c["answer"] != "blocked")
        steps = ", ".join(f"{s['what'].split(': ', 1)[1]} ({'not run this time: its stored results are read' if s.get('skipped') else 'ok' if not s['exit'] else 'exit ' + str(s['exit'])})" for s in r["steps"]) or "none"
        L.append(f"| `{case}` | {steps} | {r['runs_from_the_database']} (not: {r['runs_not_count']}) | {r['numbers']} | {len(r['changed'])} | "
                 f"{valued} of {len(e['rows'])} | {ans} | {len(e['closures']) - ans} |")
    for case, r in rec["cases"].items():
        if r["changed"]:
            L += ["", f"**{case}: numbers that differ from the ones stored before** (by name):", ""]
            L += [f"- `{c['name']}`: {c['before']} → {c['after']}" for c in r["changed"][:40]]
    L += ["", "Each closure's answer, or the row that blocks it: `results/EVALUATION.md`. Each requirement's check: `results/TRACEABILITY.md`.", ""]
    write_text(ROOT / "results" / "end_to_end.json", json.dumps(rec, indent=1, default=str) + "\n")
    write_text(ROOT / "results" / "END_TO_END.md", "\n".join(L))
    print("\n".join(L))
    bad = any(s["exit"] for r in rec["cases"].values() for s in r["steps"]) or rec["evaluate"]["exit"] or rec["trace"]["exit"]
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
