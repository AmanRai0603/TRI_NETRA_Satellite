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
             carried into design_case.node), converted to SI from the case's unit; else, for a
             stated block, the value the design states (value.number in its output's unit, or a
             list, value.list), as the engine reads it (data/stated.json)
  evidence   an achieved row a campaign supplies (spec/plan/kpis.toml): what the selected design's
             Monte Carlo (the design loop's) shows for a metric judged against the KPI's requirement
             key, at the campaign's claimed probability, else the worst run
  computed   the row's pseudocode (code.pseudocode, with the modules it uses, code.uses) run in the
             interpreter (design/js/pcode.js): the node's function (code.function, else the one named
             as the node or its output's symbol, else the module's only one) on its inputs, each
             taken from the row its `input` names (the input's name is the parameter; `p[i]` an
             element of an array parameter), in the input's unit when it states one, else in SI. An
             input may name one output of the row it reads (`from_output`: `function.output`, then
             `[i]`/`[i][j]` for an element, from 1), which runs that function of the row's module on
             the row's own inputs. The row's value is the function's output named by code.output,
             else its output's symbol, else its first; a list where it is a vector or a matrix
  not computed, with why: the input missing, the row that has none, the unit not known, no
             pseudocode, a declared value the case does not state, a supplier's value not kept,
             or a row computed during a run, whose inputs are the run's (code.run_inputs)

A KPI's two closures (kpi_<slug>_verified, kpi_<slug>_analysis) answer pass or fail when the
requirement and its evidence (or its analysis row) both have a value, in the requirement's
sense; otherwise they are blocked, naming the row that has no value.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import math
import pathlib
import posixpath
import re
import sqlite3
import subprocess
import sys
import tempfile
import tomllib

from common import ROOT, design_folder, write_text

PLAN = "spec/plan"                           # 1.0.0's plan, as the design holds it (tools/from_design.py)
STORE = ROOT / "matlab_sils" / "store"
CLI = ROOT / "design" / "js" / "pcode_cli.mjs"
TOOLBOX = ("fsw/pseudocode/01",)             # the toolbox's pseudocode: code, read from the repository
DEG = math.pi / 180
# the case's and the metrics' units, to SI
UNITS = {"": 1, "One": 1, "Count": 1, "unit": 1, "Percent": 0.01, "%": 0.01, "Degree": DEG, "deg": DEG, "DegreePerSecond": DEG, "deg/s": DEG,
         "Radian": 1, "RadianPerSecond": 1, "Kilogram": 1, "Gram": 1e-3, "Litre": 1e-3, "CubicMetre": 1, "SquareMetre": 1, "Metre": 1, "Kilometre": 1e3,
         "Millimetre": 1e-3, "Second": 1, "s": 1, "Minute": 60, "min": 60, "Hour": 3600, "Day": 86400, "Year": 365.25 * 86400, "Watt": 1, "W": 1,
         "WattHour": 3600, "Volt": 1, "Ampere": 1, "AmpereSquareMetre": 1, "KilogramSquareMetre": 1, "NewtonMetre": 1, "N m": 1,
         "MillinewtonMetre": 1e-3, "MicronewtonMetre": 1e-6, "NewtonMetreSecond": 1, "N m s": 1, "MillinewtonMetreSecond": 1e-3, "Newton": 1,
         "Tesla": 1, "Nanotesla": 1e-9, "Hertz": 1, "KilogramPerCubicMetre": 1,
         # the units the design's own rows state their values and outputs in (S7.19: evaluate reads stated blocks)
         "KgPerCubicMetre": 1, "MetrePerSecond": 1, "KmPerSecond": 1e3, "Pascal": 1, "Kilopascal": 1e3, "PascalSecond": 1,
         "Microtesla": 1e-6, "Milliwatt": 1e-3, "RiskLevel": 1, "UsDollar": 1}
SELECT = re.compile(r"^(?:(?P<fn>[A-Za-z_]\w*)\.)?(?P<out>[A-Za-z_]\w*)?(?P<idx>(?:\[\d+\])*)$")
ELEMENT = re.compile(r"^(?P<name>[A-Za-z_]\w*)(?P<idx>(?:\[\d+\])+)$")


def _toml(p):
    import from_design
    return tomllib.loads(from_design.text(p) if isinstance(p, str) else p.read_text(encoding="utf-8"))


def load_design(d):
    """{node: {group, kind, label, behaviour, content{sec.field: value}, origin{sec.field: origin}, inputs[(name, from,
    from_output, unit)], release}} of the design."""
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
                                "behaviour": ((b.get("block") or [[None] * 6])[0])[3],
                                "content": {(f"{s}.{f}" if f else s): v for s, f, v, _o in b.get("content", [])},
                                "origin": {(f"{s}.{f}" if f else s): o for s, f, _v, o in b.get("content", [])},
                                "inputs": [tuple((list(i) + [None, None])[:4]) for i in b.get("input", [])]}
    if out:
        return out, "the merged releases (design.tndb)"
    for f in sorted((d / "nodes").glob("*.node.tndb")):
        with sqlite3.connect(f"file:{f}?mode=ro", uri=True) as c:
            nid, gid, kind, label = c.execute("SELECT id, group_id, kind, label FROM node").fetchone()
            blk = c.execute("SELECT behaviour FROM block").fetchone() if c.execute(
                "SELECT 1 FROM sqlite_master WHERE name = 'block'").fetchone() else None
            rows = list(c.execute("SELECT section, field, value, origin FROM content"))
            out[nid] = {"group": gid, "kind": kind, "label": label, "release": None, "behaviour": blk[0] if blk else None,
                        "content": {(f"{s}.{fl}" if fl else s): v for s, fl, v, _o in rows},
                        "origin": {(f"{s}.{fl}" if fl else s): o for s, fl, _v, o in rows},
                        "inputs": [tuple(r) for r in c.execute("SELECT name, from_node, from_output, unit FROM input")]}
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


def _module(text):
    """(functions {name: (params [(name, type)], outputs [(name, type)])}, the module's name) of a pseudocode text."""
    import carry_over
    import groupcode
    with tempfile.TemporaryDirectory() as t:
        p = pathlib.Path(t) / "n.pc"
        p.write_text(text, encoding="utf-8")
        bs = carry_over.pc_blocks([p])
    fns = {}
    for k, v in bs.items():
        if v[0] == "fn":
            params, outs = groupcode._header(v[2], k)
            fns[k] = (params or [], outs or [])
    m = re.search(r"^module\s+(\w+)", text, re.M)
    return fns, (m.group(1) if m else None)


def _fn_of(nid, content, fns):
    """The node's function: code.function, else the one named as the node or by its output's symbol, else the module's only
    one; None when its module has several and names none for it (none is guessed)."""
    if content.get("code.function"):
        return content["code.function"] if content["code.function"] in fns else None
    if nid in fns:
        return nid
    symbol = content.get("output.symbol")
    fn = next((k for k, (_p, o) in fns.items() if o and o[0][0] == symbol), None) if symbol else None
    return fn if fn else (next(iter(fns)) if len(fns) == 1 else None)


def _files(n, nodes_by_path, d):
    """{path: text} of the node's module and, in turn, the modules it uses (code.uses): the design's, by their path, and
    the toolbox's from the repository."""
    import from_design
    path = (n["origin"].get("code.pseudocode") or "").split(" ")[0] or "n.pc"
    got, todo = {}, [(path, n["content"]["code.pseudocode"], n["content"].get("code.uses"))]
    while todo:
        p, text, uses = todo.pop(0)
        if p in got:
            continue
        got[p] = text
        for q in json.loads(uses) if uses else []:
            q = q if "/" in q else posixpath.join(posixpath.dirname(p), q)
            if q in got:
                continue
            if q.startswith(TOOLBOX):
                todo.append((q, (ROOT / q).read_text(encoding="utf-8"), None))
            elif q in nodes_by_path:
                m = nodes_by_path[q]
                todo.append((q, m["content"]["code.pseudocode"], m["content"].get("code.uses")))
            else:
                todo.append((q, from_design.text(q, pathlib.Path(d) / "design.tndb"), None))
    return got


def _run(files, fn, args):
    """The function's outputs (a list, one entry an output) run in the interpreter, or (None, why)."""
    with tempfile.TemporaryDirectory() as t:
        paths = []
        for p, text in files.items():
            f = pathlib.Path(t) / pathlib.PurePosixPath(p.lstrip("/"))
            f.parent.mkdir(parents=True, exist_ok=True)
            f.write_text(text, encoding="utf-8")
            paths.append(str(f))
        r = subprocess.run(["node", str(CLI), "run", *paths, "--fn", fn, "--args", json.dumps(args)], capture_output=True, text=True, timeout=60)
    if r.returncode:
        return None, (r.stderr or r.stdout).strip().splitlines()[-1][:200] if (r.stderr or r.stdout).strip() else "the interpreter stopped"
    v = json.loads(r.stdout)
    v = v.get("outputs", v) if isinstance(v, dict) else v
    return (v if isinstance(v, list) else [v]), None


def _pick(v, idx):
    """An element of a value by its 1-based indices `[i][j]`."""
    for i in re.findall(r"\[(\d+)\]", idx or ""):
        if not isinstance(v, list) or not 1 <= int(i) <= len(v):
            return None
        v = v[int(i) - 1]
    return v


def _scale(v, k):
    """A value in SI to a unit of k SI."""
    return [_scale(x, k) for x in v] if isinstance(v, list) else v / k


def _times(v, k):
    """A value in a unit of k SI to SI."""
    return [_times(x, k) for x in v] if isinstance(v, list) else v * k


def _where(gen):
    return f"generated into {gen}" if gen else "by the flight software or the engine, at each step"


def _text(v):
    return "[" + ", ".join(_text(x) for x in v) + "]" if isinstance(v, list) else f"{v:.6g}"


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
    raw = {}          # a case's value as it states it, with its unit's factor: a row read in that unit takes it whole

    def put(nid, state, si=None, text="", why=""):
        out[nid] = {"id": nid, "group": nodes.get(nid, {}).get("group"), "label": nodes.get(nid, {}).get("label"), "state": state,
                    "si": si, "value": text, "why": why}

    for nid, (si, text, key, unit) in stated.items():
        if nid in nodes:
            if si is None:
                put(nid, "not computed", why=f"the case states {text} ({key}), in a unit this tool has no SI factor for")
            else:
                put(nid, "stated", si, text, f"the case, {key}")
                raw[nid] = (float(text.split()[0]), UNITS[unit or ""])
    evid = {k["evidence"]: k for k in kpis}
    for nid, k in evid.items():
        key = (ci.get(k["requirement"]) or {}).get("key")
        if nid in nodes and nid not in out and key:
            e = ev.get(key)
            if e:
                put(nid, "evidence", e[0], e[1], e[2])
    # what the design states itself, on a stated block the case does not state (S7.13's finding: the engine reads these
    # values by node, data/stated.json; evaluate read none of them): a number in its output's unit, or a list as stated
    for nid, n in nodes.items():
        c = n["content"]
        if nid in out or n["behaviour"] != "stated" or ("value.number" not in c and "value.list" not in c):
            continue
        src = c.get("value.source") or c.get("spec.source") or "no source given"
        if "value.list" in c:
            try:
                v = json.loads(c["value.list"])
            except ValueError:
                put(nid, "not computed", why=f"the design states {c['value.list']!r}, not a list of numbers")
                continue
            put(nid, "stated", v, _text(v), f"the design, {src}")
            continue
        unit = c.get("output.unit") or ""
        f = UNITS.get(unit)
        try:
            x = float(c["value.number"])
        except ValueError:
            put(nid, "not computed", why=f"the design states {c['value.number']!r}, not a number")
            continue
        if f is None:
            put(nid, "not computed", why=f"the design states {c['value.number']} {unit}, in a unit this tool has no SI factor for")
            continue
        put(nid, "stated", x * f, f"{c['value.number']} {unit}".strip(), f"the design, {src}")
        raw[nid] = (x, f)

    # a method (or a node file's row, which has no block yet); a stated block that keeps 1.0.0's pseudocode as provenance is not one
    code = {nid: n for nid, n in nodes.items() if n["content"].get("code.pseudocode") and nid not in out and n["behaviour"] in (None, "method")}
    by_path = {(n["origin"].get("code.pseudocode") or "").split(" ")[0]: n for n in nodes.values() if n["content"].get("code.pseudocode")}
    mods, files, memo = {}, {}, {}

    def module(nid):
        if nid not in mods:
            mods[nid] = _module(nodes[nid]["content"]["code.pseudocode"])[0]
        return mods[nid]

    def run_inputs(nid):
        r = nodes[nid]["content"].get("code.run_inputs")
        return set(json.loads(r)) if r else set()

    def value_of(src, sel, unit, stack):
        """(value, None) of what an input reads, or (None, why it has none)."""
        m = SELECT.match(sel or "")
        if src not in code and src in out and out[src]["state"] != "not computed":
            v = out[src]["si"]
            if m and m.group("idx"):
                v = _pick(v, m.group("idx"))
        elif src in code:
            fns = module(src)
            fn = m.group("fn") if m and m.group("fn") in fns else _fn_of(src, nodes[src]["content"], fns)
            if fn is None:
                return None, "not computed"
            got, why = call(src, fn, stack)
            if got is None:
                return None, "a loop" if why == "a loop" else "not computed"
            name = m.group("out") if m and m.group("out") in got else _out_of(src, fn)
            v = got.get(name)
            if fn == _fn_of(src, nodes[src]["content"], fns) and name == _out_of(src, fn):
                v = _si(src, v)          # the row's own output, in SI as its row holds it
            v = _pick(v, m.group("idx") if m else "")
        else:
            return None, "not computed"
        if v is None:
            return None, "not computed"
        if unit:
            k = UNITS.get(unit)
            if k is None:
                return None, f"in {unit}, a unit this tool has no SI factor for"
            # a value stated in the very unit the input asks for is taken as stated, not through SI and back
            v = raw[src][0] if src in raw and raw[src][1] == k and not (m and m.group("idx")) else _scale(v, k)
        return v, None

    def _si(nid, v):
        """A row's output in SI: its function gives it in code.output_unit when it states one."""
        u = nodes[nid]["content"].get("code.output_unit")
        return v if not u or v is None else _times(v, UNITS[u])

    def _out_of(nid, fn):
        outs = [o for o, _t in module(nid)[fn][1]]
        want = nodes[nid]["content"].get("code.output") or nodes[nid]["content"].get("output.symbol")
        return want if want in outs else outs[0]

    def call(nid, fn, stack):
        """({output: value}, None) of one function of a node's module on the node's inputs, or (None, why)."""
        key = (nid, fn)
        if key in memo:
            return memo[key]
        if key in stack:
            return None, "a loop"
        params, outs = module(nid)[fn]
        direct, elems = {}, {}
        for name, fr, fo, unit in nodes[nid]["inputs"]:
            e = ELEMENT.match(name or "")
            if e:
                elems.setdefault(e.group("name"), []).append((e.group("idx"), fr, fo, unit))
            else:
                direct[name] = (fr, fo, unit)
        run = run_inputs(nid)
        args, missing, gone = [], [], []
        for pname, ptype in params:
            if pname in direct:
                fr, fo, unit = direct[pname]
                v, why = value_of(fr, fo, unit, stack + [key])
                if why:
                    (gone if why == "a loop" else missing).append(f"{pname} from {fr} ({why})")
                    continue
            elif pname in elems:
                v, bad = None, []
                for idx, fr, fo, unit in sorted(elems[pname], key=lambda x: [int(i) for i in re.findall(r"\d+", x[0])]):
                    x, why = value_of(fr, fo, unit, stack + [key])
                    if why:
                        bad.append(f"{pname}{idx} from {fr} ({why})")
                        continue
                    ij = [int(i) for i in re.findall(r"\d+", idx)]
                    v = v if v is not None else []
                    if len(ij) == 1:
                        v += [None] * (ij[0] - len(v))
                        v[ij[0] - 1] = x
                    else:
                        v += [[] for _ in range(ij[0] - len(v))]
                        v[ij[0] - 1] += [None] * (ij[1] - len(v[ij[0] - 1]))
                        v[ij[0] - 1][ij[1] - 1] = x
                if bad:
                    missing += bad
                    continue
            elif pname in run or "*" in run:
                missing.append(f"{pname} (the run's)")
                continue
            else:
                missing.append(f"{pname} (no input names it)")
                continue
            t = ptype.split(" in ")[0].strip()
            if t == "bool":
                v = bool(v)
            elif t == "int":
                if v != int(v):
                    missing.append(f"{pname} = {v}, not a whole number")
                    continue
                v = int(v)
            elif t[:1].isupper() or t.startswith(("rec", "stream")):
                missing.append(f"{pname} ({t}, a record or choice this tool does not pass)")
                continue
            args.append(v)
        if gone:
            res = (None, "a loop")
        elif missing:
            res = (None, "needs " + ", ".join(missing))
        else:
            if nid not in files:
                files[nid] = _files(nodes[nid], by_path, d)
            v, err = _run(files[nid], fn, args)
            res = ({o: x for (o, _t), x in zip(outs, v)}, None) if v is not None else (None, f"{fn}: {err}")
        memo[key] = res
        return res

    for nid, n in sorted(code.items()):
        fns = module(nid)
        fn = _fn_of(nid, n["content"], fns)
        run = run_inputs(nid)
        if not fn:
            gen = n["content"].get("code.generate")
            if n["content"].get("code.function"):
                why = f"its function {n['content']['code.function']} is not in its module"
            elif not fns:
                why = "its pseudocode defines no function"
            elif run:
                why = (f"computed during a run ({_where(gen)}): its module's {len(fns)} functions take the run's state and the product it "
                       "flies, not rows of the design, so it has no one value")
            else:
                why = f"its module has {len(fns)} functions and names none for this node (by its id or its output's symbol)"
            put(nid, "not computed", why=why)
            continue
        got, why = call(nid, fn, [])
        params = module(nid)[fn][0]
        if got is None:
            if why == "a loop":
                why = "needs " + ", ".join(f"{p} from {fr}" for p, fr, *_r in n["inputs"]) + " (a loop)"
            elif run and why.startswith("needs") and all(x.endswith("(the run's)") for x in why[6:].split(", ")):
                why = (f"computed during a run ({_where(n['content'].get('code.generate'))}): its inputs "
                       + ", ".join(p for p, _t in params) + " are the run's state and the product it flies, not rows of the design")
            put(nid, "not computed", why=why)
            continue
        o = _out_of(nid, fn)
        if n["content"].get("code.output_unit") and n["content"]["code.output_unit"] not in UNITS:
            put(nid, "not computed", why=f"its function gives {o} in {n['content']['code.output_unit']}, a unit this tool has no SI factor for")
            continue
        v = _si(nid, got[o])
        put(nid, "computed", v, f"{_text(v)} (SI)", f"{fn}({', '.join(p for p, _ in params)})" + (f" -> {o}" if len(got) > 1 else ""))
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
        elif c.get("code.moved"):
            why = f"stated, with no value: {c['code.moved']}"
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
            elif isinstance(o["si"], list) or isinstance(req["si"], list):
                closures.append({"id": cid, "kpi": k["label"], "answer": "blocked", "why": f"{other} or {k['requirement']} is not one number"})
            else:
                ok = o["si"] <= req["si"] if k["sense"] == "<=" else o["si"] >= req["si"]
                closures.append({"id": cid, "kpi": k["label"], "answer": "pass" if ok else "fail",
                                 "why": f"{other} = {o['value']} {k['sense']} {req['value']} ({k['requirement']})"})
    return {"case": case, "source": source, "rows": sorted(out.values(), key=lambda r: r["id"]), "closures": closures}


def page(results):
    L = ["# Every row, every closure, from the design database", "",
         "**In one line:** for each case, every row of the design with its value (stated by the case or the design, computed by its "
         "pseudocode, or supplied by a campaign) or why it has none, and every KPI closure answered or blocked by name "
         "(`tools/evaluate.py`, `docs/END_TO_END.md`).", ""]
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
            k = x["why"].split(":")[0] if x["why"].startswith(("supplied by", "evidence of kind", "computed during a run")) else \
                x["why"] if not x["why"].startswith("needs") else "needs an input that has no value"
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
