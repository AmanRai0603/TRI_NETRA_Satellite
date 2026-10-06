#!/usr/bin/env python3
"""Carry what already exists into the node files (docs/RELEASE_PLAN.md P8): a seeded design folder
(tools/seed_design.py) gets, node by node, every item the repository already states about it,
each marked with where it came from, and a list of what is still missing, with the team that owns it.

    python3 tools/carry_over.py DIR            carry into the design folder DIR (seeded, not yet edited)
    python3 tools/carry_over.py --check        seed and carry into a temporary folder, and check it all
    python3 tools/carry_over.py --report DIR   per group: nodes, items carried, gaps

What is carried, and from where (the origin of each item says it, "carried:<file>"):
  - the spec's seed content (spec/plan/seed_content.toml, 82 rows): the question, the answer's
    symbol, quantity, unit and bounds, the relation, its source and derivation, the value, the
    requirement's sense, assumptions, explanation, inputs and test vectors, into the node's own fields;
  - the physics (spec/plan/physics.toml, spec/physics/*.pc): each row's relation as pseudocode,
    with the constants and functions it calls, and where it is computed in Rust and in the twin;
  - the case (spec/plan/case_inputs.toml): which case key or supplier gives a declared value;
  - the KPIs (spec/plan/kpis.toml): a requirement's sense, an evidence row's metric;
  - the catalogue's algorithms (catalogue/algorithms/*.toml): their source and where they run;
  - the tree's own note on a row (spec/plan/tree.json);
  - design/carry.toml: names for the internal layer-3 rows "to be named", from the code that
    computes them, and the rows a discipline had no node for (guidance, onboard navigation,
    dynamics, FDIR, CMG and VSCMG, sizing), each with its flight pseudocode where there is one.

Nothing is invented: a field nothing states is left empty and listed as a gap (status.gaps), with
the group's lead team as its owner (status.owner_team). A field already in a node file is never
replaced. The node's signatures are untouched: carried content is a starting point, not a check.
Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import pathlib
import re
import sqlite3
import sys
import tempfile
import tomllib

import node_catalog
import tndb
from common import ROOT, V1, v1_root

PLAN = V1 / "spec" / "plan"
SPEC_SENSE = {"<=": "at_most", ">=": "at_least", "at_most": "at_most", "at_least": "at_least"}
OUTSIDE = {"independent-derivation", "published-source", "independent-tool", "physical-bound"}
FIXED_KINDS = ("interface", "closure_interface", "required", "achieved")
CODE_EXT = {".rs": "code.rust", ".c": "code.c", ".h": "code.c", ".m": "code.twin"}


def _toml(p):
    return tomllib.loads(pathlib.Path(p).read_text(encoding="utf-8"))


def _num(v):
    return None if v in (None, "") else str(v)[:-2] if str(v).endswith(".0") else str(v)


def _j(v):
    return json.dumps(v, ensure_ascii=False)


# ------------------------------------------------------------------ pseudocode, with what it calls

DECL = re.compile(r"^(fn|record|table|const)\s+([A-Za-z_][A-Za-z0-9_]*)")


def pc_blocks(files):
    """Every top-level declaration of the .pc files: name -> (kind, file, text with its comments)."""
    out = {}
    for f in files:
        lines = pathlib.Path(f).read_text(encoding="utf-8").split("\n")
        notes, i = [], 0
        while i < len(lines):
            ln = lines[i]
            m = DECL.match(ln)
            if ln.startswith("##"):
                notes.append(ln)
            elif m:
                kind, name = m.groups()
                body = [ln]
                if kind != "const":
                    i += 1
                    while i < len(lines) and lines[i] != "end":
                        body.append(lines[i])
                        i += 1
                    body.append("end")
                else:
                    while body[-1].rstrip().endswith("\\") and i + 1 < len(lines):
                        i += 1
                        body.append(lines[i])
                out.setdefault(name, (kind, str(f), "\n".join([n for n in notes if not n.startswith("## inputs from:")] + body)))
                notes = []
            elif not ln.strip() or ln.startswith("module"):
                notes = [] if not ln.strip() else notes
            i += 1
    return out


def pc_closure(names, blocks):
    """The text of the declarations `names` and everything they use, in dependency order."""
    order, seen = [], set()

    def visit(n):
        if n in seen or n not in blocks:
            return
        seen.add(n)
        body = "\n".join(x for x in blocks[n][2].split("\n") if not x.startswith("##"))
        for tok in dict.fromkeys(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", body)):     # in order: the same text every run
            if tok != n:
                visit(tok)
        order.append(n)
    for n in names:
        visit(n)
    return "\n\n".join(blocks[n][2] for n in order) + "\n"


def answer_wrapper(fn_text, node_id, symbol):
    """When the node calls its answer `symbol` and the function names its single output otherwise,
    a one-line function with the node's name for the answer: the same inputs, the same type."""
    lines, head = [x for x in fn_text.split("\n") if not x.startswith("##")], []
    for x in lines:
        head.append(x.rstrip().rstrip("\\").strip())
        if not x.rstrip().endswith("\\"):
            break
    head = " ".join(head)
    m = re.match(r"fn\s+(\w+)\((.*)\)\s*->\s*(\w+)\s*:\s*(.+)$", head)
    if not m or m.group(3) == symbol or "(" in head.split("->", 1)[1].split(":")[0]:
        return ""
    name, params, _out, typ = m.groups()
    args = [x.split(":")[0].strip() for x in params.split(",") if x.strip()]
    return f"\n## The node's answer, {symbol}, as {name} gives it.\nfn {node_id}({params}) -> {symbol}: {typ.strip()}\n    {symbol} = {name}({', '.join(args)})\nend\n"


def fn_outputs(text):
    m = re.search(r"->\s*\(?\s*([A-Za-z_][A-Za-z0-9_]*)\s*:", text)
    return m.group(1) if m else None


# ------------------------------------------------------------------ what each node gets

class Plan:
    """Everything the repository states, read once."""

    def __init__(self):
        sc = _toml(PLAN / "seed_content.toml")
        self.seed = {r["tree_id"]: r for r in sc["row"]}
        self.sources = {s["id"] for s in sc["source"]}
        self.cat = node_catalog.catalogue()
        self.rows = self.cat["rows"]
        self.physics = _toml(PLAN / "physics.toml")["function"]
        self.phys_blocks = pc_blocks(sorted((V1 / "spec" / "physics").glob("*.pc")))
        self.fsw_blocks = pc_blocks(sorted((ROOT / "fsw" / "pseudocode").glob("*.pc")))
        ci = _toml(PLAN / "case_inputs.toml")
        self.case_in = {x["tree_id"]: x for x in ci.get("input", []) if x.get("tree_id")}
        self.supplier = {x["tree_id"]: x for x in ci.get("supplier", []) if x.get("tree_id")}
        self.kpis = _toml(PLAN / "kpis.toml")["kpi"]
        self.algos = {}
        for f in sorted((V1 / "catalogue" / "algorithms").glob("*.toml")):
            a = _toml(f)
            for prm in a.get("param", []):
                if prm.get("tree_id"):
                    self.algos.setdefault(prm["tree_id"], []).append((f.relative_to(V1).as_posix(), a, prm))
        tree = json.loads((PLAN / "tree.json").read_text(encoding="utf-8"))
        self.note = {r[0]: r[3] for k in ("HN_MGT", "HN_SYS") for r in tree[k] if r[3] and r[3] != "computed"}
        self.carry = _toml(ROOT / "design" / "carry.toml")
        self.groups = _toml(ROOT / "design" / "groups.toml")["group"]
        self.fsw_map = _toml(ROOT / "fsw" / "twin_map.toml").get("node", [])

    def physics_for(self, tid):
        return [f for f in self.physics if tid in f.get("used_by", [])]

    def fsw_code(self, fn):
        """Where a flight pseudocode function is implemented (fsw/twin_map.toml), by its file."""
        if fn not in self.fsw_blocks:
            return {}
        src = pathlib.Path(self.fsw_blocks[fn][1]).name
        stem = src.split("_", 1)[0]
        for n in self.fsw_map:
            if str(n.get("pseudocode", "")).startswith(stem):
                return {"code.c": n.get("c"), "code.rust": n.get("rust"), "code.twin": "matlab_sils/" + n["matlab"] if n.get("matlab") else None}
        return {}


def seed_items(p, tid, kind):
    """The spec's seed content of one row, in the node app's fields (as the node app's
    specFields reads it): [(field, value, source)], inputs, fixtures."""
    s = p.seed.get(tid)
    if not s:
        return [], [], []
    src = "spec/plan/seed_content.toml"
    f = []
    put = lambda k, v: f.append((k, v, src)) if v not in (None, "", [], {}) else None  # noqa: E731
    put("identity.question", s.get("question"))
    put("identity.note", s.get("note"))
    put("output.symbol", s.get("symbol"))
    put("output.quantity", s.get("type"))
    put("output.unit", s.get("unit"))
    for x in ("lower", "upper"):
        put(f"output.{x}", _num(s.get(x)))
    put("output.reason_lower", s.get("reason_lower"))
    put("output.reason_upper", s.get("reason_upper"))
    if kind == "computed":
        put("relation.expression", s.get("expression"))
        put("relation.source", s.get("source"))
        put("relation.why", s.get("why"))
        steps = [x.get("text") if isinstance(x, dict) else str(x) for x in s.get("steps", [])]
        put("relation.derivation", _j([x for x in steps if x]) if any(steps) else None)
    elif kind == "declared":
        put("value.number", _num(s.get("value")))
        put("value.source", s.get("source"))
    elif kind == "kpi":
        put("requirement.sense", SPEC_SENSE.get(s.get("sense", "")))
        put("requirement.value", _num(s.get("value")))
    if s.get("source") in p.sources:
        put("sources.cited", _j([s["source"]]))
    a = s.get("assumptions") or []
    if a:
        put("assumptions", _j([{"assumes": x[0], "until": x[1]} if isinstance(x, list) else {"assumes": x.get("text", ""), "until": x.get("fails_when", "")} for x in a]))
    for k, v in (s.get("explain") or {}).items():
        put(f"explain.{k}", v if isinstance(v, str) else _j(v))
    inputs = [{"name": b, "from_node": t, "quantity": (p.rows.get(t) or [None, None, None])[2]} for b, t in s.get("inputs", [])] if kind == "computed" else []
    fixtures = [{"name": x.get("label") or f"v{i + 1}", "inputs": x.get("inputs", {}), "expected": x.get("expect"), "tolerance": x.get("tolerance"),
                 "provenance": x.get("provenance", ""), "source": x.get("source", ""), "where": x.get("where") or x.get("note", "")} for i, x in enumerate(s.get("fixture", []))]
    return f, inputs, fixtures


def kind_of(p, node, form_kind=None):
    k = node["kind"] or ""
    if k in FIXED_KINDS or k.startswith("closure"):
        return "fixed"
    spec = p.rows.get(node["id"])
    if spec:
        return {"declared": "declared", "computed": "computed", "required": "kpi", "achieved": "evidence", "door": "fixed"}.get(spec[1], "declared")
    return form_kind


def items_for(p, node, group, own):
    """Every item the repository states about one node: [(field, value, source)], inputs, fixtures."""
    tid = node["id"]
    form = own.get("identity.form_kind")
    kind = kind_of(p, node, form)
    f, inputs, fixtures = seed_items(p, tid, kind)
    put = lambda k, v, src: f.append((k, v, src)) if v not in (None, "", [], {}) else None  # noqa: E731
    # the physics: the relation as pseudocode, and where it runs
    ph = p.physics_for(tid)
    if ph:
        text = pc_closure([x["name"] for x in ph], p.phys_blocks)
        sym = own.get("output.symbol") or (p.seed.get(tid) or {}).get("symbol")
        if sym and len(ph) == 1:
            text += answer_wrapper(p.phys_blocks[ph[0]["name"]][2], tid, sym)
        put("code.pseudocode", text, "spec/physics/" + ph[0]["module"] + ".pc")
        put("code.rust", "; ".join(f"adcs_physics::{x['module']}::{x['name']} (engine/crates/adcs-physics)" for x in ph), "spec/plan/twin_map.toml")
        put("code.twin", "; ".join(f"asils.physics.{x['module']}.{x['name']} (matlab_sils/+asils/+physics/+{x['module']}/{x['name']}.m)" for x in ph), "spec/plan/twin_map.toml")
        if ph[0].get("source") in p.sources:
            put("relation.source", ph[0]["source"], "spec/plan/physics.toml")
    # the case, and who supplies a value
    if tid in p.case_in:
        x = p.case_in[tid]
        put("value.source", f"the case: key {x['key']} ({x.get('section', '')}), {x.get('label', '')}".strip(), "spec/plan/case_inputs.toml")
        put("identity.note", x.get("help"), "spec/plan/case_inputs.toml")
    elif tid in p.supplier:
        x = p.supplier[tid]
        put("value.source", f"supplied by {x['by']}: {x.get('label', '')}" + (f" (field {x['field']})" if x.get("field") else ""), "spec/plan/case_inputs.toml")
    # the KPIs
    for k in p.kpis:
        if k.get("requirement") == tid:
            put("requirement.sense", SPEC_SENSE.get(k.get("sense", "")), "spec/plan/kpis.toml")
            put("identity.note", f"KPI: {k.get('label', '')}, judged by {k.get('metric', '')}", "spec/plan/kpis.toml")
        if k.get("evidence") == tid:
            if k.get("metric") in p.cat.get("metrics", []):
                put("evidence.metric", k["metric"], "spec/plan/kpis.toml")
            put("identity.note", f"Evidence for the KPI {k.get('label', '')}: what the campaigns show for {k.get('metric', '')}", "spec/plan/kpis.toml")
    # the catalogue's algorithms
    for path, a, prm in p.algos.get(tid, []):
        if a.get("source") in p.sources:
            put("sources.cited", _j([a["source"]]), path)
        put("identity.note", f"A parameter of the algorithm {a.get('label', a.get('id'))} ({prm.get('name')}): {prm.get('meaning', '')}; it trades {prm.get('trades', '')}; "
            f"from {prm.get('lo')} to {prm.get('hi')} {prm.get('unit', '')}", path)
    # the tree's own note
    if tid in p.note:
        put("identity.note", p.note[tid], "spec/plan/tree.json")
    # where a computing row of the group runs, when nothing more precise says it
    if kind == "computed" and not any(k.startswith("code.") and k != "code.pseudocode" for k, _, _ in f):
        mods = group.get("modules", {})
        put("code.c", mods.get("c"), "design/groups.toml")
        put("code.rust", mods.get("rust"), "design/groups.toml")
        put("code.twin", mods.get("twin"), "design/groups.toml")
    return kind, f, inputs, fixtures


def gaps(kind, content, inputs, fixtures, label):
    """What a node still lacks, in words its author and lead read."""
    g, has = [], lambda k: str(content.get(k) or "").strip() not in ("", "[]")  # noqa: E731
    if "to be named" in (label or ""):
        g.append("the row is not named yet")
    if kind is None:
        g.append("its kind is not chosen (declared or computed)")
    if kind != "fixed" and not has("identity.question"):
        g.append("no question")
    if kind == "computed":
        if not has("relation.expression") and not has("code.pseudocode"):
            g.append("no relation and no pseudocode")
        if not any(x.get("provenance") in OUTSIDE for x in fixtures):
            g.append("no test vector with an answer from outside the code")
    if kind == "declared":
        if not has("value.number") and not has("value.source"):
            g.append("no value and no source for it")
    if kind == "kpi" and not has("requirement.sense"):
        g.append("no sense for the requirement")
    if kind == "evidence" and not has("evidence.metric"):
        g.append("no evidence metric")
    if not has("explain.one_line") and not has("explain.simply"):
        g.append("no explanation (the one line, said simply)")
    if kind not in ("fixed", None) and not has("belief.believed"):
        g.append("no belief record")
    return g


# ------------------------------------------------------------------ writing

def _content(c):
    return {f"{s}.{f}" if f else s: v for s, f, v in c.execute("SELECT section, field, value FROM content")}


def _put(c, key, value, origin):
    sec, _, fld = key.partition(".")      # "assumptions" is a section with one, unnamed field
    c.execute("INSERT INTO content (section, field, value, origin) VALUES (?, ?, ?, ?)", (sec, fld, value if isinstance(value, str) else _j(value), origin))


def _write_node(c, p, node, group, team, extra=()):
    own = _content(c)
    for k, v, src in extra:
        if k not in own:
            _put(c, k, v, f"carried:{src}")
            own[k] = v
    kind, items, inputs, fixtures = items_for(p, node, group, own)
    sources, n = set(), 0
    for k, v, src in items:
        if k in own:
            continue
        _put(c, k, v, f"carried:{src}")
        own[k] = v
        sources.add(src)
        n += 1
    if inputs and not c.execute("SELECT count(*) FROM input").fetchone()[0]:
        c.executemany("INSERT OR REPLACE INTO input VALUES (?, ?, ?, ?)", [(i["name"], i["from_node"], i["quantity"], None) for i in inputs])
        sources.add("spec/plan/seed_content.toml")
    have = {r[0] for r in c.execute("SELECT name FROM fixture")}
    for x in fixtures:
        if x["name"] in have:
            continue
        c.execute("INSERT INTO fixture VALUES (?, ?, ?, ?, ?, ?)", (x["name"], _j(x["inputs"]), _num(x["expected"]),
                  None if x["tolerance"] in (None, "") else float(x["tolerance"]), _j({"provenance": x["provenance"], "source": x["source"], "where": x["where"]}),
                  1 if x["provenance"] in OUTSIDE else 0))
        sources.add("spec/plan/seed_content.toml")
    if own.get("output.symbol") and not c.execute("SELECT count(*) FROM output").fetchone()[0]:
        num = lambda x: None if x in (None, "") else float(x)  # noqa: E731
        c.execute("INSERT INTO output VALUES (?, ?, ?, ?, ?, ?)", (own["output.symbol"], own.get("output.unit"), num(own.get("output.lower")), num(own.get("output.upper")),
                                                                  own.get("output.reason_lower"), own.get("output.reason_upper")))
    fx = [{"provenance": json.loads(s or "{}").get("provenance")} for (s,) in c.execute("SELECT source FROM fixture")]
    g = gaps(kind, own, inputs, fx, node["label"])
    c.execute("DELETE FROM content WHERE section = 'status' AND field IN ('gaps', 'owner_team', 'carried')")
    _put(c, "status.gaps", _j(g), "carried:tools/carry_over.py")
    _put(c, "status.owner_team", team or "", "carried:design/groups.toml")
    _put(c, "status.carried", _j(sorted(sources)), "carried:tools/carry_over.py")
    n_rev = (c.execute("SELECT coalesce(max(n), 0) FROM revision").fetchone()[0] or 0) + 1
    c.execute("INSERT INTO revision VALUES (?, ?, ?, ?)", (n_rev, "2026-10-03T00:00:00Z", "tools/carry_over.py", f"carried over: {n} item(s) from {len(sources)} source(s); {len(g)} gap(s)"))
    return n, g


def carry(root):
    """Carry everything into the design folder `root`; returns a report {group: {...}}."""
    root = pathlib.Path(root)
    p = Plan()
    gdef = {g["id"]: g for g in p.groups}
    report = {}
    # 1. the internal rows get their names, in both the group file and the node file
    names = {}
    for layer, spec in p.carry.get("internal", {}).items():
        for k, r in enumerate(spec["rows"], 1):
            names[f"l3_{layer}_row_{k:02d}"] = r
    # 2. the rows added
    added = {a["id"]: a for a in p.carry.get("add", [])}
    for gf in sorted((root / "structure").glob("*.group.tndb")):
        gid = gf.name.split(".")[0]
        with sqlite3.connect(gf) as c:
            have = {r[0] for r in c.execute("SELECT id FROM group_node")}
            for nid, r in names.items():
                if nid in have:
                    c.execute("UPDATE group_node SET label = ? WHERE id = ?", (r["label"], nid))
            for a in added.values():
                if a["group"] == gid and a["id"] not in have:
                    c.execute("INSERT INTO group_node VALUES (?, ?, ?, ?, ?, ?, ?)", (a["id"], a["id"], a.get("stage"), "added", "leaf", a["label"], "shell"))
            n_rev = (c.execute("SELECT coalesce(max(n), 0) FROM revision").fetchone()[0] or 0) + 1
            c.execute("INSERT INTO revision VALUES (?, ?, ?, ?)", (n_rev, "2026-10-03T00:00:00Z", "tools/carry_over.py", "carried over: internal rows named, rows added from the code"))
    s = tndb.schema()
    for a in added.values():
        path = root / "nodes" / f"{a['id']}.node.tndb"
        if not path.exists():
            tndb.create(path, "node", a["id"], s=s, sync=False, fill=lambda c, a=a: (
                c.execute("INSERT INTO node VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", (a["id"], a["id"], a["group"], a.get("stage"), "added", "leaf", a["label"], "shell", None, 0)),
                c.executemany("INSERT INTO content VALUES (?, ?, ?, ?)", [("identity", "label", a["label"], "carried:design/carry.toml"), ("identity", "sheet", a["id"], "carried:design/carry.toml")])))
    # 3. every node file
    for gf in sorted((root / "structure").glob("*.group.tndb")):
        gid = gf.name.split(".")[0]
        with sqlite3.connect(gf) as c:
            team = (c.execute("SELECT lead_team FROM group_info").fetchone() or [None])[0]
            nodes = [dict(zip(("id", "kind", "label", "stage"), r)) for r in c.execute("SELECT id, kind, label, stage FROM group_node WHERE state <> 'archived'")]
        rep = report.setdefault(gid, {"nodes": 0, "items": 0, "gaps": 0, "with_gaps": 0, "team": team})
        for node in nodes:
            extra = []
            r = names.get(node["id"]) or added.get(node["id"])
            if r:
                extra.append(("identity.question", r["question"], "design/carry.toml"))
                code = r.get("code", "")
                key = CODE_EXT.get(pathlib.Path(code).suffix)
                if key:
                    extra.append((key, code, "design/carry.toml"))
                extra.append(("identity.note", f"Computed in {code}", "design/carry.toml"))
                if node["id"] in names:
                    extra.append(("identity.label", r["label"], "design/carry.toml"))
            a = added.get(node["id"])
            if a:
                extra.append(("identity.form_kind", "computed", "design/carry.toml"))
                if a.get("pcode"):
                    text = pc_closure([a["pcode"]], p.fsw_blocks)
                    extra.append(("code.pseudocode", text, "fsw/pseudocode/" + pathlib.Path(p.fsw_blocks[a["pcode"]][1]).name))
                    out = fn_outputs(p.fsw_blocks[a["pcode"]][2])
                    if out:
                        extra.append(("output.symbol", out, "fsw/pseudocode/" + pathlib.Path(p.fsw_blocks[a["pcode"]][1]).name))
                    for k, v in p.fsw_code(a["pcode"]).items():
                        if v:
                            extra.append((k, v, "fsw/twin_map.toml"))
            with sqlite3.connect(root / "nodes" / f"{node['id']}.node.tndb") as c:
                if node["id"] in names:
                    c.execute("UPDATE node SET label = ?", (names[node["id"]]["label"],))
                    c.execute("DELETE FROM content WHERE section = 'identity' AND field = 'label'")
                    node["label"] = names[node["id"]]["label"]
                n, g = _write_node(c, p, node, gdef.get(gid, {}), team, extra)
            rep["nodes"] += 1
            rep["items"] += n
            rep["gaps"] += len(g)
            rep["with_gaps"] += 1 if g else 0
    return report


def problems():
    """carry.toml held to the repository: every code path exists, every pcode is a flight function,
    every layer names no more rows than it has, every added row's group and stage exist."""
    out, p = [], Plan()
    import design_rows
    counts = {}
    for r in design_rows.rows():
        m = re.fullmatch(r"l3_([a-z]+)_row_\d+", r["id"])
        if m:
            counts[m.group(1)] = counts.get(m.group(1), 0) + 1
    for layer, spec in p.carry.get("internal", {}).items():
        if layer not in counts:
            out.append(f"design/carry.toml: internal.{layer}: no such layer")
        elif len(spec["rows"]) > counts[layer]:
            out.append(f"design/carry.toml: internal.{layer}: {len(spec['rows'])} names for {counts[layer]} rows")
        for r in spec["rows"]:
            if not (v1_root(r["code"]) / r["code"]).exists():
                out.append(f"design/carry.toml: internal.{layer}: {r['label']}: no {r['code']}")
    gdef = {g["id"]: g for g in p.groups}
    ids = set()
    for a in p.carry.get("add", []):
        if a["id"] in ids:
            out.append(f"design/carry.toml: {a['id']} twice")
        ids.add(a["id"])
        if not re.fullmatch(r"[a-z][a-z0-9_]{1,63}", a["id"]):
            out.append(f"design/carry.toml: {a['id']} is not a node id")
        g = gdef.get(a["group"])
        if not g:
            out.append(f"design/carry.toml: {a['id']}: no group {a['group']}")
        elif g.get("stages") and a.get("stage") not in {s["id"] for s in g["stages"]}:
            out.append(f"design/carry.toml: {a['id']}: {a['group']} has stages; {a.get('stage')!r} is not one")
        elif not g.get("stages") and a.get("stage"):
            out.append(f"design/carry.toml: {a['id']}: {a['group']} has no stages")
        if not (v1_root(a["code"]) / a["code"]).exists():
            out.append(f"design/carry.toml: {a['id']}: no {a['code']}")
        if a.get("pcode") and a["pcode"] not in p.fsw_blocks:
            out.append(f"design/carry.toml: {a['id']}: no flight pseudocode function {a['pcode']}")
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("dir", nargs="?")
    ap.add_argument("--check", action="store_true", help="seed and carry into a temporary folder and check it all")
    ap.add_argument("--report", action="store_true", help="only report what DIR holds (after carrying)")
    a = ap.parse_args(argv)
    if a.check:
        import group
        import seed_design
        bad = problems()
        with tempfile.TemporaryDirectory() as d:
            seed_design.seed(d, sync=False)
            rep = carry(d)
            bad += [e for f in sorted(pathlib.Path(d).rglob("*.tndb")) for e in tndb.check(f)]
            bad += group.check(d)
        for b in bad:
            print(b)
        n = sum(r["nodes"] for r in rep.values())
        print(f"carry_over: {n} nodes in {len(rep)} groups, {sum(r['items'] for r in rep.values())} items carried, "
              f"{sum(r['with_gaps'] for r in rep.values())} nodes with gaps; {len(bad)} problem(s)")
        return 1 if bad else 0
    if not a.dir:
        ap.error("give the design folder, or --check")
    if a.report:
        rep = report(a.dir)
    else:
        rep = carry(a.dir)
    for gid, r in sorted(rep.items()):
        print(f"{gid:10} {r['nodes']:4} nodes  {r['items']:5} items  {r['with_gaps']:4} with gaps  owner: {r['team']}")
    return 0


def report(root):
    """Per group, from the node files as they are: nodes, carried items, gaps."""
    out = {}
    for gf in sorted(pathlib.Path(root, "structure").glob("*.group.tndb")):
        gid = gf.name.split(".")[0]
        with sqlite3.connect(gf) as c:
            team = (c.execute("SELECT lead_team FROM group_info").fetchone() or [None])[0]
            ids = [r[0] for r in c.execute("SELECT id FROM group_node WHERE state <> 'archived'")]
        r = out.setdefault(gid, {"nodes": 0, "items": 0, "gaps": 0, "with_gaps": 0, "team": team})
        for nid in ids:
            with sqlite3.connect(pathlib.Path(root, "nodes", f"{nid}.node.tndb")) as c:
                r["nodes"] += 1
                r["items"] += c.execute("SELECT count(*) FROM content WHERE origin LIKE 'carried:%' AND section <> 'status'").fetchone()[0]
                g = json.loads((c.execute("SELECT value FROM content WHERE section = 'status' AND field = 'gaps'").fetchone() or ["[]"])[0])
                r["gaps"] += len(g)
                r["with_gaps"] += 1 if g else 0
    return out


if __name__ == "__main__":
    sys.exit(main())
