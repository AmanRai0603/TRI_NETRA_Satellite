#!/usr/bin/env python3
"""Every row of the design evaluated for one case, or shown as not computed and why, and every
KPI closure answered or blocked by name (docs/RELEASE_PLAN.md P13, docs/END_TO_END.md).

    python3 tools/evaluate.py DIR [CASE ...] [--out DIR]   results/EVALUATION.md and evaluation.json
    python3 tools/evaluate.py DIR --check                 exit 1 when a closure that answered before
                                                          is blocked now, or a stated value is lost

DIR is a design folder (or the Drive pack's root, whose Design/ is used): its merged releases
(DIR/design.tndb) when there are, else its node files. Anything else is refused by name.
The case is read from DIR/design.tndb (its `design_case` rows), never from the data folder.

How a row gets its value, in order:
  stated     a value the case states, on the row that declares it (spec/plan/case_inputs.toml,
             carried into design_case.node), converted to SI from the case's unit
  computed   the row's pseudocode (code.pseudocode) run in the interpreter (design/js/pcode.js),
             each of its function's inputs taken from the row its `input` names, once that row
             has a value; the answer in SI
  evidence   an achieved row a campaign supplies (spec/plan/kpis.toml): what the selected design's
             Monte Carlo (the design loop's) shows for a metric judged against the KPI's requirement
             key, at the campaign's claimed probability, else the worst run
  not computed, with why: the input missing, the row that has none, the unit not known, no
             pseudocode, a declared value the case does not state, a supplier's value not kept

A KPI's two closures (kpi_<slug>_verified, kpi_<slug>_analysis) answer pass or fail when the
requirement and its evidence (or its analysis row) both have a value, in the requirement's
sense; otherwise they are blocked, naming the row that has no value.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import math
import pathlib
import sqlite3
import subprocess
import sys
import tempfile
import tomllib

from common import ROOT, design_folder, write_text

PLAN = "spec/plan"                           # 1.0.0's plan, as the design holds it (tools/from_design.py)
STORE = ROOT / "matlab_sils" / "store"
CLI = ROOT / "design" / "js" / "pcode_cli.mjs"
DEG = math.pi / 180
# the case's and the metrics' units, to SI
UNITS = {"": 1, "One": 1, "Count": 1, "unit": 1, "Percent": 0.01, "%": 0.01, "Degree": DEG, "deg": DEG, "DegreePerSecond": DEG, "deg/s": DEG,
         "Radian": 1, "RadianPerSecond": 1, "Kilogram": 1, "Gram": 1e-3, "Litre": 1e-3, "CubicMetre": 1, "SquareMetre": 1, "Metre": 1, "Kilometre": 1e3,
         "Millimetre": 1e-3, "Second": 1, "s": 1, "Minute": 60, "min": 60, "Hour": 3600, "Day": 86400, "Year": 365.25 * 86400, "Watt": 1, "W": 1,
         "WattHour": 3600, "Volt": 1, "Ampere": 1, "AmpereSquareMetre": 1, "KilogramSquareMetre": 1, "NewtonMetre": 1, "N m": 1,
         "MillinewtonMetre": 1e-3, "MicronewtonMetre": 1e-6, "NewtonMetreSecond": 1, "N m s": 1, "MillinewtonMetreSecond": 1e-3, "Newton": 1,
         "Tesla": 1, "Nanotesla": 1e-9, "Hertz": 1, "KilogramPerCubicMetre": 1}


def _toml(p):
    import from_design
    return tomllib.loads(from_design.text(p) if isinstance(p, str) else p.read_text(encoding="utf-8"))


def load_design(d):
    """{node: {group, kind, label, content{sec.field: value}, inputs[(name, from)], release}} of the design."""
    d = pathlib.Path(d)
    out = {}
    db = d / "design.tndb"
    if db.exists():
        with sqlite3.connect(f"file:{db}?mode=ro", uri=True) as c:
            for nid, gid, kind, label, content, rel in c.execute("SELECT id, group_id, kind, label, content, release FROM design_node"):
                try:
                    x = json.loads(content)
                except (TypeError, ValueError):
                    continue
                if isinstance(x, dict) and "body" in x:
                    b = x["body"]
                    out[nid] = {"group": gid, "kind": kind, "label": label, "release": rel,
                                "content": {(f"{s}.{f}" if f else s): v for s, f, v, _o in b.get("content", [])},
                                "inputs": [(i[0], i[1]) for i in b.get("input", [])]}
    if out:
        return out, "the merged releases (design.tndb)"
    for f in sorted((d / "nodes").glob("*.node.tndb")):
        with sqlite3.connect(f"file:{f}?mode=ro", uri=True) as c:
            nid, gid, kind, label = c.execute("SELECT id, group_id, kind, label FROM node").fetchone()
            out[nid] = {"group": gid, "kind": kind, "label": label, "release": None,
                        "content": {(f"{s}.{fl}" if fl else s): v for s, fl, v in c.execute("SELECT section, field, value FROM content")},
                        "inputs": [(n, fr) for n, fr in c.execute("SELECT name, from_node FROM input")]}
    return out, "the node files (no group has released yet)"


def case_values(d, case):
    """{node: (si, stated text)} and {key: row} of a case, from DIR/design.tndb."""
    with sqlite3.connect(f"file:{pathlib.Path(d) / 'design.tndb'}?mode=ro", uri=True) as c:
        rows = [dict(zip(("key", "label", "unit", "value", "node"), r)) for r in
                c.execute('SELECT "key", "label", "unit", "value", "node" FROM design_case WHERE case_id = ? ORDER BY "ord"', (case,))]
    if not rows:
        raise SystemExit(f"evaluate: no case {case} in {d}/design.tndb (its engine inputs: tools/design_inputs.py)")
    vals = {}
    for r in rows:
        if not r["node"] or not (r["value"] or "").strip():
            continue
        try:
            x = float(r["value"])
        except ValueError:
            continue
        f = UNITS.get(r["unit"] or "")
        vals[r["node"]] = (x * f if f is not None else None, f"{r['value']} {r['unit'] or ''}".strip(), r["key"], r["unit"])
    return vals, {r["key"]: r for r in rows}


def _fn_of(nid, text, symbol):
    import carry_over
    import groupcode
    with tempfile.TemporaryDirectory() as t:
        p = pathlib.Path(t) / "n.pc"
        p.write_text(text, encoding="utf-8")
        bs = carry_over.pc_blocks([p])
    fn = nid if nid in bs else next((k for k, v in bs.items() if v[0] == "fn" and carry_over.fn_outputs(v[2]) == symbol), None)
    if fn is None:
        fn = next((k for k, v in reversed(list(bs.items())) if v[0] == "fn"), None)
    params, outs = groupcode._header(bs[fn][2], fn) if fn else (None, None)
    return fn, params or [], outs or []


def _run(text, fn, args):
    with tempfile.TemporaryDirectory() as t:
        p = pathlib.Path(t) / "n.pc"
        p.write_text(text, encoding="utf-8")
        r = subprocess.run(["node", str(CLI), "run", str(p), "--fn", fn, "--args", json.dumps(args)], capture_output=True, text=True, timeout=60)
    if r.returncode:
        return None, (r.stderr or r.stdout).strip().splitlines()[-1][:200] if (r.stderr or r.stdout).strip() else "the interpreter stopped"
    v = json.loads(r.stdout)
    v = v.get("outputs", v) if isinstance(v, dict) else v
    flat = v if isinstance(v, list) else [v]
    while flat and isinstance(flat[0], list):
        flat = flat[0]
    try:
        return float(flat[0]), None
    except (TypeError, ValueError, IndexError):
        return None, f"the answer is not one number ({str(v)[:60]})"


def evidence(case):
    """{requirement key: (si, text, source)}: what the selected design's Monte Carlo (the design loop's,
    store/pipeline/<case>/mc) shows for each requirement a metric is judged against (the metric's
    req_key): the value at the campaign's claimed probability (99.73 % of runs), else the worst run.
    Before the design loop has run, the case's reference-product campaigns, labelled so."""
    best = {}
    loop = STORE / "pipeline" / case / "mc" / "summary.json"
    sel = STORE / "pipeline" / case / "selection.json"
    if loop.is_file():
        family = json.loads(sel.read_text()).get("selected") if sel.is_file() else "?"
        srcs, label = [loop], f"the design loop's Monte Carlo of the selected family {family}"
    else:
        srcs, label = sorted((STORE / "results_engine" / "campaigns").glob("*/summary.json")), "a campaign of a reference product (the design loop has not run)"
    for f in srcs:
        s = json.loads(f.read_text())
        if s.get("case") != case:
            continue
        keys = {}
        for r in s.get("per_run", []):
            for m in r.get("metrics", []):
                if m.get("req_key"):
                    keys.setdefault(m["id"], m["req_key"])
        claim = (s.get("claim") or {}).get("probability")
        for st in s["stats"]:
            key, fac = keys.get(st["id"]), UNITS.get(st["unit"])
            vals = sorted(v for v in st.get("values", []) if v is not None and math.isfinite(v))
            if not key or fac is None or not vals:
                continue
            if claim:
                v = vals[min(len(vals) - 1, max(0, math.ceil(claim * len(vals)) - 1))]
                how = f"{100 * claim:g} % of {len(vals)} runs"
            else:
                v, how = vals[-1], f"the worst of {len(vals)} runs"
            cur = best.get(key)
            if cur is None or v * fac > cur[0]:
                best[key] = (v * fac, f"{v:.4g} {st['unit']}", f"{label} ({s['id']}): {st['id']}, {how}")
    return best


def evaluate(d, case):
    nodes, source = load_design(d)
    stated, _ = case_values(d, case)
    kpis = _toml(f"{PLAN}/kpis.toml")["kpi"]
    ci = {x["tree_id"]: x for x in _toml(f"{PLAN}/case_inputs.toml").get("input", []) if x.get("tree_id")}
    ev = evidence(case)
    out = {}

    def put(nid, state, si=None, text="", why=""):
        out[nid] = {"id": nid, "group": nodes.get(nid, {}).get("group"), "label": nodes.get(nid, {}).get("label"), "state": state,
                    "si": si, "value": text, "why": why}

    for nid, (si, text, key, unit) in stated.items():
        if nid in nodes:
            if si is None:
                put(nid, "not computed", why=f"the case states {text} ({key}), in a unit this tool has no SI factor for")
            else:
                put(nid, "stated", si, text, f"the case, {key}")
    evid = {k["evidence"]: k for k in kpis}
    for nid, k in evid.items():
        key = (ci.get(k["requirement"]) or {}).get("key")
        if nid in nodes and nid not in out and key:
            e = ev.get(key)
            if e:
                put(nid, "evidence", e[0], e[1], e[2])
    code = {nid: n for nid, n in nodes.items() if n["content"].get("code.pseudocode") and nid not in out}
    plans = {nid: _fn_of(nid, n["content"]["code.pseudocode"], n["content"].get("output.symbol")) for nid, n in code.items()}
    progress = True
    while progress:
        progress = False
        for nid, n in code.items():
            if nid in out:
                continue
            fn, params, _ = plans[nid]
            if not fn:
                put(nid, "not computed", why="its pseudocode defines no function")
                progress = True
                continue
            src = dict(n["inputs"])
            args, missing = [], []
            for pname, ptype in params:
                fr = src.get(pname)
                if fr is None:
                    missing.append(f"{pname} (no input names it)")
                elif fr not in out:
                    missing.append(f"{pname} from {fr}")
                elif out[fr]["si"] is None:
                    missing.append(f"{pname} from {fr} (not computed)")
                else:
                    args.append(out[fr]["si"])
            if missing:
                if all(x.endswith("(no input names it)") or x.endswith("(not computed)") for x in missing):
                    put(nid, "not computed", why="needs " + ", ".join(missing))
                    progress = True
                continue
            v, err = _run(n["content"]["code.pseudocode"], fn, args)
            put(nid, "computed" if v is not None else "not computed", v, f"{v:.6g} (SI)" if v is not None else "",
                f"{fn}({', '.join(p for p, _ in params)})" if v is not None else f"{fn}: {err}")
            progress = True
    for nid, n in code.items():
        if nid not in out:
            fn, params, _ = plans[nid]
            src = dict(n["inputs"])
            need = [f"{p} from {src.get(p)}" for p, _ in params if src.get(p) not in out or out[src.get(p)]["si"] is None]
            put(nid, "not computed", why="needs " + ", ".join(need) + " (a loop, or rows not computed)")
    for nid, n in nodes.items():
        if nid in out or n["kind"] in ("closure_analysis", "closure_verified", "closure_interface", "interface"):
            continue
        c = n["content"]
        if nid in ci and ci[nid].get("key"):
            why = f"the case does not state {ci[nid]['key']}"
        elif nid in ci and ci[nid].get("by"):
            why = f"supplied by {ci[nid]['by']}: not kept in the design yet"
        elif nid in evid:
            why = f"evidence: no engine campaign of this case judges a metric against {(ci.get(evid[nid]['requirement']) or {}).get('key') or evid[nid]['requirement']}"
        elif c.get("relation.expression") or c.get("spec.expression"):
            why = "a relation, but no pseudocode yet: its author writes it"
        else:
            why = "no value, relation or pseudocode yet"
        put(nid, "not computed", why=why)
    closures = []
    for k in kpis:
        req = out.get(k["requirement"])
        for kind, other in (("verified", k["evidence"]), ("analysis", k["analysis"])):
            cid = f"kpi_{k['slug']}_{kind}"
            if kind == "analysis" and other == "none":
                continue
            o = out.get(other)
            if not req or req["si"] is None:
                closures.append({"id": cid, "kpi": k["label"], "answer": "blocked", "why": f"the requirement {k['requirement']} has no value ({(req or {}).get('why') or 'not in the design'})"})
            elif not o or o["si"] is None:
                closures.append({"id": cid, "kpi": k["label"], "answer": "blocked", "why": f"{other} has no value ({(o or {}).get('why') or 'not in the design'})"})
            else:
                ok = o["si"] <= req["si"] if k["sense"] == "<=" else o["si"] >= req["si"]
                closures.append({"id": cid, "kpi": k["label"], "answer": "pass" if ok else "fail",
                                 "why": f"{other} = {o['value']} {k['sense']} {req['value']} ({k['requirement']})"})
    return {"case": case, "source": source, "rows": sorted(out.values(), key=lambda r: r["id"]), "closures": closures}


def page(results):
    L = ["# Every row, every closure, from the design database", "",
         "**In one line:** for each case, every row of the design with its value (stated by the case, computed by its pseudocode, or "
         "supplied by a campaign) or why it has none, and every KPI closure answered or blocked by name (`tools/evaluate.py`, "
         "`docs/END_TO_END.md`).", ""]
    for r in results:
        st = {}
        for x in r["rows"]:
            st[x["state"]] = st.get(x["state"], 0) + 1
        ans = {}
        for c in r["closures"]:
            ans[c["answer"]] = ans.get(c["answer"], 0) + 1
        L += [f"## {r['case']}", "", f"From {r['source']}. Rows: " + ", ".join(f"{v} {k}" for k, v in sorted(st.items())) + ". "
              "Closures: " + ", ".join(f"{v} {k}" for k, v in sorted(ans.items())) + ".", "",
              "| Closure | KPI | Answer | Why |", "|---|---|---|---|"]
        L += [f"| `{c['id']}` | {c['kpi']} | {'**' + c['answer'] + '**' if c['answer'] != 'blocked' else 'blocked'} | {c['why']} |" for c in r["closures"]]
        L += ["", "| Row | Group | State | Value | Where from / why not |", "|---|---|---|---|---|"]
        L += [f"| `{x['id']}` | {x['group']} | {x['state']} | {x['value']} | {x['why']} |" for x in r["rows"] if x["state"] != "not computed"]
        nc = [x for x in r["rows"] if x["state"] == "not computed"]
        why = {}
        for x in nc:
            k = x["why"].split(":")[0] if x["why"].startswith(("supplied by", "evidence of kind")) else x["why"] if not x["why"].startswith("needs") else "needs an input that has no value"
            why.setdefault(k, []).append(x["id"])
        L += ["", f"**Not computed ({len(nc)}), by why:**", ""]
        L += [f"- {k}: {len(v)} ({', '.join(v[:12])}{', …' if len(v) > 12 else ''})" for k, v in sorted(why.items(), key=lambda kv: -len(kv[1]))]
        L.append("")
    return "\n".join(L) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("dir")
    ap.add_argument("cases", nargs="*")
    ap.add_argument("--out", default=str(ROOT / "results"))
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    cases = a.cases or ["ais_3u", "ais_img_3u"]
    d = design_folder(a.dir)
    if not (d / "design.tndb").is_file():
        raise SystemExit(f"evaluate: {d} has node files but no design.tndb, which holds the cases "
                         "(build it: python3 tools/seed_design.py, or tools/group.py merge)")
    res = [evaluate(d, c) for c in cases]
    out = pathlib.Path(a.out)
    prev = json.loads((out / "evaluation.json").read_text()) if (out / "evaluation.json").exists() else None
    if a.check:
        bad = []
        if prev:
            for r in res:
                p = next((x for x in prev if x["case"] == r["case"]), None)
                if not p:
                    continue
                was = {c["id"]: c["answer"] for c in p["closures"]}
                bad += [f"{r['case']}: {c['id']} answered before ({was[c['id']]}), blocked now: {c['why']}" for c in r["closures"]
                        if c["answer"] == "blocked" and was.get(c["id"]) in ("pass", "fail")]
                had = {x["id"] for x in p["rows"] if x["state"] == "stated"}
                bad += [f"{r['case']}: {x['id']} was stated, now: {x['why']}" for x in r["rows"] if x["id"] in had and x["state"] != "stated"]
        for b in bad:
            print("evaluate: " + b, file=sys.stderr)
        print(f"evaluate: {len(res)} case(s) checked, {len(bad)} problem(s)")
        return 1 if bad else 0
    write_text(out / "evaluation.json", json.dumps(res, indent=1) + "\n")
    write_text(out / "EVALUATION.md", page(res))
    for r in res:
        st = {}
        for x in r["rows"]:
            st[x["state"]] = st.get(x["state"], 0) + 1
        ans = {}
        for c in r["closures"]:
            ans[c["answer"]] = ans.get(c["answer"], 0) + 1
        print(f"evaluate: {r['case']}: rows {st}; closures {ans}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
