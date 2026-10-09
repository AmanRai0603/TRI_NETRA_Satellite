#!/usr/bin/env python3
"""The health map of a design for a case (docs/OPERATING_2_0.md §6; docs/SYSTEM_MODEL.md §4; docs/PLAN_2_0.md S4):
every node's health with why, rolled up group by group to the ADCS, each closure's range verdict and tornado,
and trace to cause for every closure that does not close.

    python3 tools/health.py DIR [CASE ...] [--out DIR]     results/HEALTH.md and health.json
    python3 tools/health.py DIR --built-in                 the built-in nodes by group (relations still in code)

DIR holds the design database (DIR/design.tndb: today's design, tools/design_build.py, or a released one).
Every row's value comes from tools/evaluate.py on that design; nothing here computes a value of its own.

A node's health, worst first (a node shows the worst that applies, and lists every one):
  fails     a closure it decides does not hold
  refused   it refused: its value is outside its own range, or its method stopped
  blocked   it cannot run: a node it reads has no value; it names that node
  open      not decided yet: an open block, a value nobody states yet, or evidence no run gives yet
  unproven  its release is not signed by a person (a converted baseline), or its cases are not reproduced
  tight     it closes, but only for part of the range its inputs may still take
  closes    answered, and every closure it feeds holds

The built-in count (docs/PLAN_2_0.md S7): every node whose behaviour is built-in, its relation still in compiled
code, by group; S7 lowers it to zero. Beside it, the nodes that describe code by the boundary (the flight
software's runtime and its tests: content code.boundary), which stay code and are not counted.

A closure's range verdict (docs/SYSTEM_MODEL.md §4) says one of three things: it closes for the whole range,
for part of it (and where it crosses), or fails for all of it. Today's ranges are the case's dispersions:
  - by evidence: every run of the case's edge campaigns that judges the closure's requirement (each dispersion at
    its low and high bound, one at a time, then all at the adverse end), and the design loop's Monte Carlo
    (the spread of the runs);
  - by analysis: the design graph; a closure that is blocked says which row blocks it.
Its tornado ranks the dispersions by how far each moves the margin, from the edge campaign's one-at-a-time runs:
the widest bar is the decision that matters most.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import pathlib
import re
import sqlite3
import sys

import evaluate
from common import ROOT, design_folder, write_text

ORDER = ["fails", "refused", "blocked", "open", "unproven", "tight", "closes"]
# units a node's range is written in that tools/evaluate.py's case table does not need, to SI
RANGE_UNITS = {"Microtesla": 1e-6, "KmPerSecond": 1e3, "Millitesla": 1e-3, "Milliwatt": 1e-3, "Kilowatt": 1e3, "Milligram": 1e-6,
               "Microradian": 1e-6, "Milliradian": 1e-3, "Arcsecond": evaluate.DEG / 3600, "Arcminute": evaluate.DEG / 60}


def to_si(unit):
    """The factor from a range's unit to SI, or None when it is not known (the range is then not judged)."""
    f = evaluate.UNITS.get(unit or "")
    return f if f is not None else RANGE_UNITS.get(unit or "")
STORE = ROOT / "matlab_sils" / "store"
CAMPAIGNS = STORE / "results_engine" / "campaigns"


def worst(states):
    return min(states, key=ORDER.index) if states else "closes"


def design_facts(d):
    """{node: {group, label, behaviour, sealed_as, why_sealed, outputs{name: (unit, lower, upper)}, port_states[...], closure}}."""
    out = {}
    with sqlite3.connect(f"file:{pathlib.Path(d) / 'design.tndb'}?mode=ro", uri=True) as c:
        for nid, gid, label, content in c.execute("SELECT id, group_id, label, content FROM design_node"):
            x = json.loads(content)
            b = x["body"]
            blk = (b.get("block") or [[None] * 6])[0]
            out[nid] = {"group": gid, "label": label, "behaviour": blk[3], "sealed_as": x.get("sealed_as"), "why_sealed": x.get("why") or [],
                        "outputs": {o[0]: (o[1], o[2], o[3]) for o in b.get("output", [])},
                        "port_states": [p[3] for p in b.get("port", []) if p[1] == "out"],
                        "closure": (b.get("closure") or [None])[0]}
    return out


def built_in(d):
    """The built-in count of a design (DIR/design.tndb): {"count", "by_group" {group: [node]}, "boundary" {node: kind}}.
    A node is built-in when its behaviour is; a node with content code.boundary describes runtime or test code."""
    out, boundary = {}, {}
    with sqlite3.connect(f"file:{pathlib.Path(d) / 'design.tndb'}?mode=ro", uri=True) as c:
        for nid, gid, content in c.execute("SELECT id, group_id, content FROM design_node ORDER BY id"):
            b = json.loads(content)["body"]
            if any(blk[3] == "built-in" for blk in b.get("block") or []):
                out.setdefault(gid, []).append(nid)
            kind = next((v for s, f, v, _o in b.get("content", []) if (s, f) == ("code", "boundary")), None)
            if kind:
                boundary[nid] = kind
    return {"count": sum(len(v) for v in out.values()), "by_group": dict(sorted(out.items())), "boundary": boundary}


def built_in_lines(bi):
    """The built-in count as the page and the command show it."""
    L = [f"**{bi['count']} built-in nodes**, relations still in compiled code (docs/PLAN_2_0.md S7 lowers this to zero): "
         + (", ".join(f"{g} {len(v)}" for g, v in bi["by_group"].items()) or "none") + ".", ""]
    if bi["by_group"]:
        L += ["| Group | Built-in nodes |", "|---|---|"]
        L += [f"| {g} | " + ", ".join(f"`{n}`" for n in v) + " |" for g, v in bi["by_group"].items()]
        L.append("")
    if bi["boundary"]:
        L += [f"Not counted: {len(bi['boundary'])} nodes describe code by the boundary (docs/SYSTEM_MODEL.md §7): "
              + ", ".join(f"`{n}` ({k})" for n, k in sorted(bi["boundary"].items())) + ".", ""]
    return L


def _sense_margin(value, req, sense):
    """The margin in the requirement's own sense: positive holds, negative fails."""
    return (req - value) if sense == "<=" else (value - req)


def evidence_ranges(case, req_key, req_si, sense):
    """The closure's range from the edge campaigns and the design loop's Monte Carlo that judge req_key:
    (verdict, text, tornado[(dispersion, margin at low, margin at high, width)], sources)."""
    points, tornado, sources = [], [], []
    for f in sorted(CAMPAIGNS.glob("*/summary.json")):
        s = json.loads(f.read_text())
        if s.get("case") != case or s.get("type") != "edge":
            continue
        C = json.loads((ROOT / "matlab_sils" / "data" / "campaigns" / f"{s['id']}.json").read_text())
        ds = C["dispersions"] if isinstance(C["dispersions"], list) else [C["dispersions"]]
        by_k = {}
        for r in s["per_run"]:
            for m in r.get("metrics", []):
                if m.get("req_key") == req_key and m.get("value") is not None:
                    fac = evaluate.UNITS.get(m.get("unit") or next((st["unit"] for st in s["stats"] if st["id"] == m["id"]), ""))
                    if fac is None:
                        continue
                    by_k[r["k"]] = (m["id"], m["value"] * fac)
        if not by_k:
            continue
        sources.append(f"{s['id']} ({len(by_k)} runs)")
        points += [(f"{s['id']} run {k}", v) for k, (_m, v) in sorted(by_k.items())]
        for j, d in enumerate(ds, 1):
            lo, hi = by_k.get(2 * j - 1), by_k.get(2 * j)
            if lo and hi:
                a, b = _sense_margin(lo[1], req_si, sense), _sense_margin(hi[1], req_si, sense)
                tornado.append({"dispersion": d.get("kind"), "campaign": s["id"], "margin_low": a, "margin_high": b, "width": abs(b - a),
                                "range": [d.get("lo"), d.get("hi")] if "lo" in d else [f"-{d.get('frac')}", f"+{d.get('frac')}"]})
    mc = STORE / "pipeline" / case / "mc" / "summary.json"
    if mc.is_file():
        s = json.loads(mc.read_text())
        n = 0
        for r in s.get("per_run", []):
            for m in r.get("metrics", []):
                if m.get("req_key") == req_key and m.get("value") is not None:
                    st = next((x for x in s["stats"] if x["id"] == m["id"]), None)
                    fac = evaluate.UNITS.get((st or {}).get("unit", ""))
                    if fac is not None:
                        points.append((f"{s['id']} run {r['k']}", m["value"] * fac))
                        n += 1
        if n:
            sources.append(f"the design loop's Monte Carlo {s['id']} ({n} runs)")
    tornado.sort(key=lambda t: -t["width"])
    if not points:
        return None, "no run of this case judges it", [], []
    margins = [(w, _sense_margin(v, req_si, sense)) for w, v in points]
    held = [w for w, m in margins if m >= 0]
    if len(held) == len(margins):
        return "whole", f"closes for the whole range: all {len(margins)} runs hold (least margin {min(m for _, m in margins):.4g})", tornado, sources
    if not held:
        return "fails", f"fails for all of it: none of {len(margins)} runs holds", tornado, sources
    bad = [w for w, m in margins if m < 0]
    return "part", (f"closes for part of it: {len(held)} of {len(margins)} runs hold; it fails at {', '.join(bad[:6])}"
                    f"{', …' if len(bad) > 6 else ''}"), tornado, sources


def health(d, case):
    d = design_folder(d) if not (pathlib.Path(d) / "design.tndb").is_file() else pathlib.Path(d)
    facts = design_facts(d)
    ev = evaluate.evaluate(d, case)
    rows = {r["id"]: r for r in ev["rows"]}
    stated, by_key = evaluate.case_values(d, case)
    key_of = {r["node"]: k for k, r in by_key.items() if r.get("node")}
    reasons = {nid: [] for nid in facts}

    def add(nid, state, why):
        if nid in reasons:
            reasons[nid].append((state, why))

    # each row: what evaluate found, held to its own range
    for nid, f in facts.items():
        r = rows.get(nid)
        if r is None:
            continue
        if r["state"] in ("computed", "stated", "evidence") and r["si"] is not None:
            if isinstance(r["si"], list):
                continue                  # a vector or a matrix: its outputs' ranges are each element's, not stated per element
            for oname, (unit, lo, hi) in f["outputs"].items():
                k = to_si(unit)
                if r["state"] != "computed" or k is None or (lo is None and hi is None):
                    continue
                if (lo is not None and r["si"] < lo * k * (1 - 1e-12)) or (hi is not None and r["si"] > hi * k * (1 + 1e-12)):
                    add(nid, "refused", f"{oname} = {r['si'] / k:.6g} {unit} is outside its own range [{lo}, {hi}] {unit}")
        elif r["state"] == "not computed":
            w = r["why"]
            if w.startswith("computed during a run"):
                # a model the engine or the flight software runs at each step: its inputs are the run's (code.run_inputs),
                # so it has no one value to wait for; its method is there, which is all a design row can be
                continue
            if w.startswith("needs"):
                add(nid, "blocked", w)
            elif re.match(r"^\w+: ", w) and "interpreter" in w or w.startswith(("its pseudocode defines no function",)):
                add(nid, "refused", w)
            elif f["outputs"] and f["behaviour"] not in ("lookup", "children"):
                # a value to have and none yet; a lookup's answer is its table, a branch's its children, and a
                # block with no output (a flight algorithm module, a table) has no row value to wait for
                add(nid, "open", w)
    for nid, f in facts.items():
        if f["behaviour"] == "open":
            add(nid, "open", "an open block: not decided yet")
        if f["behaviour"] == "evidence" and (rows.get(nid) or {}).get("state") != "evidence":
            add(nid, "open", "evidence that no run gives yet" + (f" ({rows[nid]['why']})" if rows.get(nid) and rows[nid].get("why") else ""))
        if f["sealed_as"] not in (None, "sealed", "confirmed", "signed"):
            add(nid, "unproven", "; ".join(f["why_sealed"]) or f"its release is {f['sealed_as']}")
    # closures: the answer, the range verdict, the tornado
    closures = []
    for c in ev["closures"]:
        cl = (facts.get(c["id"]) or {}).get("closure")
        req, ach, sense, by = (cl[1], cl[2], cl[3], cl[4]) if cl else (None, None, None, None)
        item = {"id": c["id"], "kpi": c["kpi"], "answer": c["answer"], "why": c["why"], "requirement": req, "achieved": ach, "by": by}
        if c["answer"] == "blocked":
            item["verdict"], item["range"] = "blocked", c["why"]
            add(c["id"], "blocked", c["why"])
        else:
            r_si = (rows.get(req) or {}).get("si")
            verdict, text, torn, src = (None, "by analysis: the design graph gives one value; its inputs carry no range yet", [], [])
            if by == "evidence" and req in key_of:
                verdict, text, torn, src = evidence_ranges(case, key_of[req], r_si, sense)
            item.update({"verdict": verdict or ("whole" if c["answer"] == "pass" else "fails"), "range": text, "tornado": torn, "range_from": src})
            if c["answer"] == "fail" or verdict == "fails":
                add(c["id"], "fails", c["why"])
                if ach:
                    add(ach, "fails", f"decides {c['id']}, which fails")
            elif verdict == "part":
                add(c["id"], "tight", text)
                if ach:
                    add(ach, "tight", f"decides {c['id']}: {text}")
        closures.append(item)
    # trace to cause: down through what a closure reads to the nodes that cause it

    def causes(nid, seen=None):
        seen = seen or set()
        if nid in seen:
            return []
        seen.add(nid)
        st = [s for s, _w in reasons.get(nid, []) if s in ("blocked",)]
        if not st:
            return [nid]
        why = next(w for s, w in reasons[nid] if s == "blocked")
        ups = re.findall(r"from (\w+)", why)
        out = []
        for u in ups:
            out += causes(u, seen)
        return out or [nid]
    for item in closures:
        if item["verdict"] == "blocked":
            m = re.search(r"(?:requirement )?(\w+) has no value", item["why"])
            root = m.group(1) if m else None
            cs = sorted(set(causes(root))) if root else []
            item["cause"] = [{"node": x, "group": (facts.get(x) or {}).get("group"), "label": (facts.get(x) or {}).get("label"),
                              "why": "; ".join(w for _s, w in reasons.get(x, []) if _s != "unproven") or (rows.get(x) or {}).get("why", "")} for x in cs]
        elif item["verdict"] in ("part", "fails") and item.get("tornado"):
            t = item["tornado"][0]
            item["cause"] = [{"dispersion": t["dispersion"], "campaign": t["campaign"],
                              "why": f"the widest bar: margin {t['margin_low']:.4g} at its low end, {t['margin_high']:.4g} at its high end"}]
    node_rows = []
    for nid, f in sorted(facts.items()):
        rs = reasons[nid]
        st = worst([s for s, _w in rs])
        node_rows.append({"id": nid, "group": f["group"], "label": f["label"], "health": st, "why": [f"{s}: {w}" for s, w in rs],
                          "value": (rows.get(nid) or {}).get("value", ""),
                          "cause": sorted(set(causes(nid))) if any(s == "blocked" for s, _w in rs) else []})
    groups = {}
    for n in node_rows:
        g = groups.setdefault(n["group"], {"group": n["group"], "health": "closes", "count": {}})
        g["count"][n["health"]] = g["count"].get(n["health"], 0) + 1
        g["health"] = worst([g["health"], n["health"]])
    adcs = worst([g["health"] for g in groups.values()])
    return {"case": case, "design": str(d / "design.tndb"), "adcs": adcs, "groups": sorted(groups.values(), key=lambda g: g["group"]),
            "closures": closures, "nodes": node_rows}


def page(results, bi=None):
    L = ["# The health map", "",
         "**In one line:** every node's health for each case, worst first, rolled up group by group to the ADCS; each closure's "
         "answer, its range verdict and tornado; and, for every closure that does not close, the nodes that cause it "
         "(`tools/health.py`, `docs/OPERATING_2_0.md` §6).", "",
         "Health, worst first: " + ", ".join(f"**{s}**" for s in ORDER) + ". A node shows the worst that applies and lists every one.", ""]
    if bi is not None:
        L += ["## Built-in: relations still in code", ""] + built_in_lines(bi)
    for r in results:
        L += [f"## {r['case']}: the ADCS is **{r['adcs']}**", "", f"From `{r['design']}`.", "",
              "| Group | Health | Nodes by health |", "|---|---|---|"]
        L += [f"| {g['group']} | **{g['health']}** | " + ", ".join(f"{g['count'][s]} {s}" for s in ORDER if s in g["count"]) + " |" for g in r["groups"]]
        L += ["", "| Closure | Answer | Range verdict | Tornado: widest bars (margin at low → at high) | Cause |", "|---|---|---|---|---|"]
        for c in r["closures"]:
            torn = "; ".join(f"{t['dispersion']} ({t['margin_low']:.3g} → {t['margin_high']:.3g})" for t in (c.get("tornado") or [])[:3]) or "—"
            cause = "; ".join((f"`{x['node']}` ({x['group']}): {x['why']}" if "node" in x else f"{x['dispersion']} in {x['campaign']}: {x['why']}")
                              for x in c.get("cause", [])[:3]) or "—"
            L.append(f"| `{c['id']}` | {c['answer']} | {c['range']} | {torn} | {cause} |")
        bad = [n for n in r["nodes"] if n["health"] in ("fails", "refused", "tight")]
        if bad:
            L += ["", "**Nodes that fail, refuse or are tight:**", ""]
            L += [f"- `{n['id']}` ({n['group']}): {n['health']}: {'; '.join(n['why'][:2])}" for n in bad]
        L.append("")
    return "\n".join(L) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("dir")
    ap.add_argument("cases", nargs="*")
    ap.add_argument("--out", default=str(ROOT / "results"))
    ap.add_argument("--built-in", action="store_true", help="only print the built-in nodes by group (relations still in code)")
    a = ap.parse_args(argv)
    d = pathlib.Path(a.dir)
    if not (d / "design.tndb").is_file():
        sys.exit(f"health: {d} holds no design.tndb (build today's design: python3 tools/design_build.py DRIVE --out {d}/design.tndb)")
    bi = built_in(d)
    if a.built_in:
        print("\n".join(built_in_lines(bi)).rstrip())
        return 0
    with sqlite3.connect(f"file:{d / 'design.tndb'}?mode=ro", uri=True) as c:
        cases = a.cases or [x for (x,) in c.execute("SELECT DISTINCT case_id FROM design_case ORDER BY case_id") if x != "case_template"]
    results = [health(d, case) for case in cases]
    out = pathlib.Path(a.out)
    write_text(out / "health.json", json.dumps(results, indent=1) + "\n")
    write_text(out / "HEALTH.md", page(results, bi))
    print(f"health: built-in {bi['count']} (" + ", ".join(f"{g} {len(v)}" for g, v in bi["by_group"].items()) + ")")
    for r in results:
        cnt = {}
        for n in r["nodes"]:
            cnt[n["health"]] = cnt.get(n["health"], 0) + 1
        print(f"health: {r['case']}: ADCS {r['adcs']}; nodes " + ", ".join(f"{cnt[s]} {s}" for s in ORDER if s in cnt)
              + "; closures " + ", ".join(f"{c['id'].removeprefix('kpi_')} {c['verdict']}" for c in r["closures"] if c["verdict"] != "blocked"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
