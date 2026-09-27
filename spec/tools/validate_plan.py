#!/usr/bin/env python3
"""Checks everything under plan/, catalogue/, scenarios/, campaigns/ and
designs/ for internal consistency, before Claude Code or a person builds on it.

    python3 tools/validate_plan.py          # exit 0 = clean
    python3 tools/validate_plan.py --selftest

It checks shape, not physics. Whether a relation is right is H1b's question,
and no script answers it.
"""

import csv
import glob
import io
import json
import math
import re
import os
import sys

try:
    import tomllib  # Python 3.11+
except ImportError:  # pragma: no cover
    tomllib = None

ROOT = os.environ.get("ADCS_PLAN_ROOT") or os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
KIND = {
    "set here": "declared",
    "computed": "computed",
    "target — required": "required",
    "achieved — what the design delivers": "achieved",
}
FAMILIES = {"mtq", "rw", "fmr", "rcs"}
KEYS = ["_source", "_note", "layer3_shape", "HN_MGT", "ED_MGT", "HN_SYS", "ED_SYS", "VE", "KE"]


def check_tree(t):
    errs = []
    for k in KEYS:
        if k not in t:
            errs.append("tree.json: missing key %s" % k)
    if errs:
        return errs, {}
    rows = {}
    table_of = {}
    for tab, root in (("HN_MGT", "mgm"), ("HN_SYS", "prg")):
        roots = [r for r in t[tab] if r[2] is None]
        if [r[0] for r in roots] != [root]:
            errs.append("%s: exactly one root %r expected, got %r" % (tab, root, [r[0] for r in roots]))
        for r in t[tab]:
            if len(r) != 5:
                errs.append("%s: row %r is not a 5-tuple" % (tab, r[0]))
            if r[0] in rows:
                errs.append("duplicate id %s" % r[0])
            rows[r[0]] = r
            table_of[r[0]] = tab
    kids = {}
    for r in rows.values():
        if r[2] is not None:
            if r[2] not in rows:
                errs.append("%s: parent %s does not exist" % (r[0], r[2]))
            elif table_of[r[2]] != table_of[r[0]]:
                errs.append("%s: parent is in the other layer" % r[0])
            kids.setdefault(r[2], []).append(r[0])
    leaf = {i for i in rows if i not in kids}
    kind = {}
    doors = 0
    for i in leaf:
        note = rows[i][3]
        if "door into" in note:
            doors += 1
            kind[i] = "required"
        else:
            kind[i] = KIND.get(note, "declared")
        extra = rows[i][4]
        if extra:
            bad = set(extra.split(",")) - FAMILIES
            if bad:
                errs.append("%s: unknown family tag(s) %s" % (i, sorted(bad)))
    if doors != 1:
        errs.append("%d door-into leaves; exactly one is allowed: every case goes through it, and no customer is a branch of the tree" % doors)
    # labels unique within a group (V15)
    for g, ks in kids.items():
        labs = [rows[k][1] for k in ks]
        dup = {x for x in labs if labs.count(x) > 1}
        if dup:
            errs.append("%s: labels repeated within the group: %s" % (g, sorted(dup)))
    # layer 3 shape
    for s in t["layer3_shape"]:
        g = s.get("group")
        if g not in rows:
            errs.append("layer3 %s: group %s not in HN_SYS" % (s["id"], g))
            continue
        n = len([k for k in kids.get(g, []) if k in leaf])
        if n != s["targets"]:
            errs.append("layer3 %s: targets %d but group %s holds %d leaves" % (s["id"], s["targets"], g, n))
        if s["nodes"] < 1 + 2 * s["targets"]:
            errs.append("layer3 %s: %d rows cannot hold the interface and target pairs" % (s["id"], s["nodes"]))
    # edges
    for tab in ("ED_MGT", "ED_SYS"):
        want = "HN_MGT" if tab == "ED_MGT" else "HN_SYS"
        for a, b, why in t[tab]:
            for x in (a, b):
                if x not in rows:
                    errs.append("%s: endpoint %s missing" % (tab, x))
                elif table_of[x] != want:
                    errs.append("%s: endpoint %s is in the other layer" % (tab, x))
            if not why:
                errs.append("%s: %s->%s has no reason" % (tab, a, b))
    inputs = {}
    for a, b, why in t["VE"]:
        if a not in rows or b not in rows:
            errs.append("VE: %s->%s endpoint missing" % (a, b))
            continue
        if table_of[a] != table_of[b]:
            errs.append("VE: %s->%s crosses a layer; only the door may" % (a, b))
        if a not in leaf or b not in leaf:
            errs.append("VE: %s->%s must join two leaves" % (a, b))
        if kind.get(b) != "computed":
            errs.append("VE: %s->%s feeds a %s row; only computed rows take inputs" % (a, b, kind.get(b)))
        inputs.setdefault(b, []).append(a)
    for i in leaf:
        if kind[i] == "computed" and i not in inputs:
            errs.append("%s (%s): computed with no input (V5)" % (i, rows[i][1]))
    # acyclic VE
    state = {}

    def dfs(n, stack):
        state[n] = 1
        for m in [b for a, b, _ in t["VE"] if a == n]:
            if state.get(m) == 1:
                errs.append("VE cycle through %s" % " -> ".join(stack + [m]))
            elif not state.get(m):
                dfs(m, stack + [m])
        state[n] = 2

    for n in list(inputs):
        if not state.get(n):
            dfs(n, [n])
    for a, b, why in t["KE"]:
        if a not in leaf:
            errs.append("KE: %s is not a leaf" % a)
        if kind.get(b) != "achieved":
            errs.append("KE: %s->%s must point at an achieved KPI" % (a, b))
    # every required KPI has its achieved twin, same label
    for g in [x for x in rows if x.endswith("k") and x[:-1] + "a" in rows]:
        req = [rows[k][1] for k in kids.get(g, [])]
        ach = [rows[k][1] for k in kids.get(g[:-1] + "a", [])]
        if req != ach:
            errs.append("%s and %s do not list the same targets" % (g, g[:-1] + "a"))
    achieved = [i for i in leaf if kind[i] == "achieved"]
    fed = {b for _, b, _ in t["KE"]}
    for i in achieved:
        if i not in fed:
            errs.append("%s (%s): an achieved KPI nothing contributes to" % (i, rows[i][1]))
    return errs, {"rows": rows, "leaf": leaf, "kind": kind, "kids": kids}


def load_toml(path):
    with open(path, "rb") as f:
        return tomllib.load(f)


def check_seed_content(ctx):
    errs = []
    p = os.path.join(ROOT, "plan", "seed_content.toml")
    if not os.path.exists(p) or tomllib is None:
        return errs
    d = load_toml(p)
    rows, leaf, kind = ctx["rows"], ctx["leaf"], ctx["kind"]
    l3 = {s["id"] for s in json.load(open(os.path.join(ROOT, "plan", "tree.json")))["layer3_shape"]}
    sources = {s["id"] for s in d.get("source", [])}
    cp = os.path.join(ROOT, "plan", "case_inputs.toml")
    ledger = {x["tree_id"] for x in load_toml(cp).get("supplier", []) if x.get("by") == "derisk"} if os.path.exists(cp) else set()
    ids = set()
    for r in d.get("row", []):
        rid = r.get("tree_id") or r.get("id")
        if not rid:
            errs.append("seed: a row with neither tree_id nor id")
            continue
        if rid in ids:
            errs.append("seed: %s written twice" % rid)
        ids.add(rid)
        if "tree_id" in r:
            if r["tree_id"] not in leaf:
                errs.append("seed: tree_id %s is not a leaf of the tree" % r["tree_id"])
            elif kind[r["tree_id"]] == "required" and r.get("kind") == "declared" and r.get("sense") in ("<=", ">="):
                pass  # a written requirement: declared with a sense, the VLEO convention
            elif kind[r["tree_id"]] != r.get("kind"):
                errs.append("seed: %s is %s on the tree, %s in the seed content" % (rid, kind[r["tree_id"]], r.get("kind")))
        else:
            if r.get("layer3") not in l3:
                errs.append("seed: %s names layer3 %r, not a subsystem layer" % (rid, r.get("layer3")))
        for f in ("question", "expression", "source", "symbol", "type", "unit", "reason_lower", "reason_upper"):
            if not r.get(f):
                errs.append("seed: %s has no %s" % (rid, f))
        if r.get("source") and r["source"] not in sources:
            errs.append("seed: %s cites %s, which is not a [[source]]" % (rid, r["source"]))
        if not (r.get("lower", 0) < r.get("upper", 0)):
            errs.append("seed: %s lower must be below upper" % rid)
        if r.get("sense") not in (None, "<=", ">="):
            errs.append("seed: %s sense must be exactly <= or >=" % rid)
        if r.get("kind") == "declared" and r.get("tree_id") in ledger:
            if "value" in r:
                errs.append("seed: %s is counted from the de-risking ledger; the seed content must not give it a value" % rid)
        elif r.get("kind") == "declared":
            if "value" not in r:
                errs.append("seed: declared %s has no reference value" % rid)
            elif not (r["lower"] <= r["value"] <= r["upper"]):
                errs.append("seed: %s reference value outside its own domain" % rid)
        if r.get("kind") == "computed":
            if not r.get("inputs"):
                errs.append("seed: computed %s has no inputs" % rid)
            if not r.get("physics"):
                errs.append("seed: computed %s names no adcs-core::physics function" % rid)
            if not r.get("why"):
                errs.append("seed: computed %s has no [theory] why — it would return Undefined" % rid)
        for fx in r.get("fixture", []):
            if fx.get("provenance") in ("self-snapshot", "agent-generated"):
                errs.append("seed: %s fixture provenance %s is refused" % (rid, fx["provenance"]))
            if fx.get("provenance") not in ("independent-derivation", "published-source", "independent-tool", "physical-bound"):
                errs.append("seed: %s fixture provenance %r unknown" % (rid, fx.get("provenance")))
            if fx.get("source") not in sources:
                errs.append("seed: %s fixture cites unknown source %r" % (rid, fx.get("source")))
            if not fx.get("tolerance", 0) > 0:
                errs.append("seed: %s fixture tolerance must be > 0" % rid)
    known = ids | set(leaf)
    tree = json.load(open(os.path.join(ROOT, "plan", "tree.json"), encoding="utf-8"))
    ve_in = {}
    for a, b, _ in tree["VE"]:
        ve_in.setdefault(b, set()).add(a)
    for r in d.get("row", []):
        rid = r.get("tree_id") or r.get("id")
        for b in r.get("inputs", []):
            if b[1] not in known:
                errs.append("seed: %s reads %s, which is neither in the seed content nor on the tree" % (rid, b[1]))
        if r.get("tier") not in ("A+", "A", "B", "C"):
            errs.append("seed: %s tier %r is not A+, A, B or C" % (rid, r.get("tier")))
        counts_read = sorted(b[1] for b in r.get("inputs", []) if b[1].startswith("cf_"))
        if r.get("kind") == "computed" and sorted(r.get("zero_when_absent", [])) != counts_read:
            errs.append("seed: %s reads counts %s but declares zero_when_absent %s; they must match (SPEC.md §5.6)"
                        % (rid, counts_read, sorted(r.get("zero_when_absent", []))))
        if r.get("call") not in (None, "slice"):
            errs.append("seed: %s call %r is not 'slice'" % (rid, r.get("call")))
        if "tree_id" in r and r.get("kind") == "computed":
            got = {b[1] for b in r.get("inputs", [])}
            want = ve_in.get(r["tree_id"], set())
            if got != want:
                errs.append("seed: %s inputs %s differ from its VE edges %s" % (rid, sorted(got), sorted(want)))
        for fx in r.get("fixture", []):
            bind = {b[0] for b in r.get("inputs", [])}
            if set(fx.get("inputs", {})) != bind:
                errs.append("seed: %s fixture %r inputs %s are not exactly the bindings %s"
                            % (rid, fx.get("label"), sorted(fx.get("inputs", {})), sorted(bind)))
    return errs


METRIC_KINDS = {"ape", "rpe", "pde", "ake", "rke", "rate_stability", "pointing_rms", "time_to_threshold",
                "time_to_rate", "sun_acquisition", "slew_time", "settling_time", "recovery_time",
                "slews_per_orbit", "tracking_rate", "max_rate", "momentum_margin", "dump_interval",
                "faults_survived", "actuator_peak", "actuator_power", "power", "power_peak", "consumable",
                "consumable_per_year", "inspection_mass", "inspection_volume"}


def check_kpis(ctx):
    errs = []
    p = os.path.join(ROOT, "plan", "kpis.toml")
    if not os.path.exists(p):
        return ["plan/kpis.toml missing"], {}
    rows, kind = ctx["rows"], ctx["kind"]
    kpis = load_toml(p).get("kpi", [])
    slugs = set()
    by_ev = {}
    for k in kpis:
        r, e, a = k.get("requirement"), k.get("evidence"), k.get("analysis")
        if kind.get(r) != "required":
            errs.append("kpis: requirement %s is not a required row" % r)
        if kind.get(e) != "achieved":
            errs.append("kpis: evidence %s is not an achieved row" % e)
        if r in rows and e in rows and rows[r][1] != rows[e][1]:
            errs.append("kpis: %s and %s name different targets" % (r, e))
        if a != "none" and kind.get(a) not in ("computed", "declared"):
            errs.append("kpis: analysis %s is not a system-layer computed or declared row" % a)
        if k.get("metric") not in METRIC_KINDS:
            errs.append("kpis: metric %r is not a metric kind (SPEC.md §10.3)" % k.get("metric"))
        if k.get("sense") not in ("<=", ">="):
            errs.append("kpis: %s sense %r" % (r, k.get("sense")))
        if k.get("slug") in slugs:
            errs.append("kpis: slug %s used twice" % k.get("slug"))
        slugs.add(k.get("slug"))
        by_ev[e] = k
    achieved = {i for i in kind if kind[i] == "achieved"}
    if set(by_ev) != achieved:
        errs.append("kpis: covers %d achieved rows, the tree has %d" % (len(by_ev), len(achieved)))
    return errs, by_ev


SOURCES = set()
KPIS = {}
FAMILY_PARTS = {}
FAMILY_SLOTS = {}
SCENARIOS = {}


def check_units_physics():
    """plan/units.toml and plan/physics.toml against each other, the seed content,
    the case format and SPEC.md §6.2 (when the spec is beside the plan)."""
    errs = []
    up, pp = os.path.join(ROOT, "plan", "units.toml"), os.path.join(ROOT, "plan", "physics.toml")
    if tomllib is None or not os.path.exists(up) or not os.path.exists(pp):
        return errs
    u = load_toml(up)
    quantities = {q["name"]: q for q in u.get("quantity", [])}
    units = {x["name"]: x for x in u.get("unit", [])}
    covered = set()
    for n, x in units.items():
        for q in x.get("quantity", []):
            if q not in quantities:
                errs.append("units: %s states quantity %s, which is not a [[quantity]]" % (n, q))
            covered.add(q)
        if not (isinstance(x.get("si"), float) and x["si"] > 0):
            errs.append("units: %s has no positive SI factor" % n)
    for q, x in quantities.items():
        if x.get("origin") == "adcs" and q not in covered:
            errs.append("units: quantity %s (SPEC.md §6.1) has no unit" % q)
    fns = {"%s::%s" % (f["module"], f["name"]): f for f in load_toml(pp).get("function", [])}
    sp = os.path.join(ROOT, "plan", "seed_content.toml")
    seed = load_toml(sp).get("row", []) if os.path.exists(sp) else []
    for r in seed:
        rid = r.get("tree_id")
        if r.get("unit") not in units:
            errs.append("seed: %s unit %r is not in plan/units.toml" % (rid, r.get("unit")))
        elif r.get("type") not in units[r["unit"]].get("quantity", []):
            errs.append("seed: %s states %s in %s, which states %s" % (rid, r.get("type"), r["unit"], units[r["unit"]].get("quantity")))
        if r.get("physics"):
            f = fns.get(r["physics"])
            if not f:
                errs.append("seed: %s calls %s, which plan/physics.toml does not have" % (rid, r["physics"]))
            elif r.get("call") != "slice" and len(f["args"]) != len(r.get("inputs", [])):
                errs.append("seed: %s calls %s with %d binding(s); it takes %d" % (rid, r["physics"], len(r.get("inputs", [])), len(f["args"])))
            elif rid not in f.get("used_by", []):
                errs.append("physics: %s does not list %s, which calls it" % (r["physics"], rid))
    cp = os.path.join(ROOT, "plan", "case_inputs.toml")
    if os.path.exists(cp):
        for i in load_toml(cp).get("input", []):
            if i.get("unit") not in units:
                errs.append("case_inputs: %s unit %r is not in plan/units.toml" % (i.get("key"), i.get("unit")))
    spec = os.path.join(ROOT, "spec", "06_physics.md")
    if os.path.exists(spec):
        text = open(spec, encoding="utf-8").read()
        a = text.find("| Module | Functions")
        table = text[a:text.find("\n\n", a)] if a >= 0 else ""
        named = set()
        for line in table.splitlines()[2:]:
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if len(cells) < 2:
                continue
            mod = cells[0].strip("`").split("`")[0].split()[0]
            for m in re.finditer(r"`(\w+)\(", cells[1]):
                named.add("%s::%s" % (mod, m.group(1)))
        if named and named != set(fns):
            errs.append("physics: plan/physics.toml and SPEC.md §6.2 differ: only in the file %s, only in the spec %s"
                        % (sorted(set(fns) - named), sorted(named - set(fns))))
    return errs


def check_catalogue():
    errs = []
    dp = os.path.join(ROOT, "plan", "seed_content.toml")
    if os.path.exists(dp):
        SOURCES.update(s["id"] for s in load_toml(dp).get("source", []))
    if tomllib is None:
        return ["tomllib unavailable: use Python 3.11+"]
    schema = os.path.join(ROOT, "catalogue", "schema.toml")
    if not os.path.exists(schema):
        return errs
    sch = load_toml(schema)
    kinds = {k["kind"]: k for k in sch.get("module_kind", [])}
    parts = {}
    for p in sorted(glob.glob(os.path.join(ROOT, "catalogue", "parts", "*.toml"))):
        d = load_toml(p)
        pn = d.get("part_number")
        if pn in parts:
            errs.append("catalogue: part %s defined twice" % pn)
        parts[pn] = d
        k = d.get("kind")
        if k not in kinds:
            errs.append("catalogue %s: kind %r not in schema" % (pn, k))
            continue
        for f in kinds[k].get("required", []):
            if f not in d.get("nominal", {}):
                errs.append("catalogue %s: nominal.%s missing" % (pn, f))
        for f in ("status", "source", "families"):
            if f not in d:
                errs.append("catalogue %s: %s missing" % (pn, f))
        if d.get("status") not in ("synthetic", "placeholder", "reference", "qualified", "flight-proven"):
            errs.append("catalogue %s: status %r unknown" % (pn, d.get("status")))
        if d.get("source") not in SOURCES:
            errs.append("catalogue %s: source %r is not a [[source]] in plan/seed_content.toml" % (pn, d.get("source")))
        if d.get("status") == "synthetic" and not str(pn).startswith("SYN-"):
            errs.append("catalogue %s: a synthetic part must be named SYN-*, so it is visible everywhere it appears" % pn)
    fams = os.path.join(ROOT, "catalogue", "families.toml")
    if os.path.exists(fams):
        for fam in load_toml(fams).get("family", []):
            fid = fam["id"]
            FAMILY[fid] = fam
            FAMILY_PARTS[fid] = set()
            if not set(fam.get("tags", [])) <= FAMILIES or "mtq" not in fam.get("tags", []):
                errs.append("families %s: tags %r must be drawn from %s and include mtq (coils are the safety floor)"
                            % (fid, fam.get("tags"), sorted(FAMILIES)))
            slots = fam.get("slot", [])
            FAMILY_SLOTS[fid] = slots
            if not any(s.get("name") == "coils" and s.get("required") for s in slots):
                errs.append("families %s: no required coils slot (coils are the safety floor, SPEC.md §2)" % fid)
            names = [s.get("name") for s in slots]
            if len(names) != len(set(names)):
                errs.append("families %s: a slot name repeats" % fid)
            counted = [s.get("count") for s in slots if s.get("count")]
            live = [k for k, (lo, hi) in fam.get("counts", {}).items() if hi > 0]
            if sorted(counted) != sorted(live):
                errs.append("families %s: every count row that can be above zero must belong to exactly one slot; slots count %s, the family has %s"
                            % (fid, sorted(counted), sorted(live)))
            for s in slots:
                if s.get("count") not in ("",) and s.get("count") not in fam.get("counts", {}):
                    errs.append("families %s slot %s: count %r is not one of the family's counts" % (fid, s.get("name"), s.get("count")))
                if s.get("count_from"):
                    for pn in s.get("parts", []):
                        k = kinds.get(parts.get(pn, {}).get("kind"), {})
                        if pn in parts and s["count_from"] not in k.get("required", []):
                            errs.append("families %s slot %s: count_from %s is not a required field of %s's kind"
                                        % (fid, s.get("name"), s["count_from"], pn))
                for pn in s.get("parts", []):
                    FAMILY_PARTS[fid].add(pn)
                    if pn not in parts:
                        errs.append("families %s slot %s: part %s not in catalogue/parts" % (fid, s.get("name"), pn))
                        continue
                    if parts[pn].get("kind") not in s.get("kinds", []):
                        errs.append("families %s slot %s: %s is a %s, not one of %s"
                                    % (fid, s.get("name"), pn, parts[pn].get("kind"), s.get("kinds")))
                    if fid not in parts[pn].get("families", []):
                        errs.append("families %s slot %s: %s does not list the family" % (fid, s.get("name"), pn))
    return errs, parts


CASE = {}          # plan/case_inputs.toml, loaded
CASE_KEYS = {}     # key -> input or meta entry
CLASSES = {}
ALGORITHMS = {}
PRODUCTS = {}
FAMILY = {}
NUMBERS = {}
FIELD_UNIT = {"_km": "Kilometre", "_deg": "Degree", "_deg_s": "DegreePerSecond", "_rad_s": "RadianPerSecond",
              "_m_s": "MetrePerSecond", "_h": "Hour", "_s": "Second"}


def check_case_registry(ctx):
    """plan/case_inputs.toml against the tree and the seed content: every declared
    layer-2 leaf has exactly one supplier, every requirement row is a case
    input, and a seed row's unit is the unit the case CSV carries."""
    errs = []
    p = os.path.join(ROOT, "plan", "case_inputs.toml")
    if not os.path.exists(p):
        return ["plan/case_inputs.toml missing: run python3 tools/build_tree.py"]
    reg = load_toml(p)
    CASE.update(reg)
    rows, kind, leaf = ctx["rows"], ctx["kind"], ctx["leaf"]
    sys_leaf = {i for i in leaf if i in rows and _in_sys(i, rows)}
    seen = {}
    for m in reg.get("meta", []):
        CASE_KEYS[m["key"]] = dict(m, section="meta")
    for i in reg.get("input", []):
        if i["key"] in CASE_KEYS:
            errs.append("case_inputs: key %s twice" % i["key"])
        CASE_KEYS[i["key"]] = i
        t = i.get("tree_id")
        if t not in sys_leaf:
            errs.append("case_inputs: %s -> %s, not a layer-2 leaf" % (i["key"], t))
            continue
        if kind[t] not in ("declared", "required"):
            errs.append("case_inputs: %s -> %s, a %s row; only declared and requirement rows take a case value" % (i["key"], t, kind[t]))
        if i.get("blank") not in ("stated", "default"):
            errs.append("case_inputs: %s blank policy %r" % (i["key"], i.get("blank")))
        if i.get("level") and kind[t] != "required":
            errs.append("case_inputs: %s carries a level but is not a requirement" % i["key"])
        seen.setdefault(t, []).append("case")
    for sp in reg.get("supplier", []):
        t = sp.get("tree_id")
        if sp.get("by") == "lab":
            # layer 1 only: a facility capability, read from every lab file the rig runs from (SPEC.md §12.10)
            if t in sys_leaf or t not in leaf or not _under(t, "fac", rows) or kind.get(t) != "declared":
                errs.append("case_inputs: lab supplies %s, which is not a declared leaf of the Test facility branch" % t)
            for lp in sorted(glob.glob(os.path.join(ROOT, "rig", "labs", "*.toml"))):
                v = load_toml(lp)
                for part in str(sp.get("field", "")).split("."):
                    v = v.get(part) if isinstance(v, dict) else None
                    if v is None:
                        break
                ok = (isinstance(v, (int, float)) and not isinstance(v, bool)) or (isinstance(v, list) and sp.get("reduce") in ("max", "len"))
                if not ok:
                    errs.append("rig/labs/%s: %s (%s) needs the field %s as a number (nan until measured)%s"
                                % (os.path.basename(lp), t, rows[t][1], sp.get("field"), " or a list" if sp.get("reduce") else ""))
            seen.setdefault(t, []).append("lab")
            continue
        if sp.get("by") == "derisk":
            # layer 1 only: the risk rows, counted from the de-risking ledger (SPEC.md §5.13)
            if t in sys_leaf or t not in leaf or not _under(t, "rsk", rows) or kind.get(t) != "declared":
                errs.append("case_inputs: derisk supplies %s, which is not a declared leaf of the rsk branch" % t)
            seen.setdefault(t, []).append("derisk")
            continue
        if sp.get("by") not in ("product", "tuned", "evidence"):
            errs.append("case_inputs: supplier of %s is %r" % (t, sp.get("by")))
        seen.setdefault(t, []).append(sp.get("by"))
    for t in leaf:
        if t in rows and _under(t, "rsk", rows) and kind[t] == "declared" and seen.get(t) != ["derisk"]:
            errs.append("case_inputs: %s (%s) is a declared risk row; the de-risking ledger must be its one supplier" % (t, rows[t][1]))
    for t in sys_leaf:
        if kind[t] in ("declared", "required") and len(seen.get(t, [])) != 1:
            errs.append("case_inputs: %s (%s) has %d suppliers %s; exactly one is required"
                        % (t, rows[t][1], len(seen.get(t, [])), seen.get(t, [])))
    dp = os.path.join(ROOT, "plan", "seed_content.toml")
    if os.path.exists(dp):
        by_tree = {i["tree_id"]: i for i in reg.get("input", [])}
        for r in load_toml(dp).get("row", []):
            i = by_tree.get(r.get("tree_id"))
            if i and r.get("unit") != i.get("unit"):
                errs.append("case_inputs: %s is in %s in the seed content but the case CSV carries %s" % (i["key"], r.get("unit"), i.get("unit")))
    return errs


def _under(t, gid, rows):
    while t is not None:
        if t == gid:
            return True
        t = rows[t][2] if t in rows else None
    return False


def _in_sys(i, rows):
    while rows[i][2] is not None:
        i = rows[i][2]
    return i == "prg"


def _num(x):
    try:
        v = float(x)
    except (TypeError, ValueError):
        return None
    return v


def check_case_csv(name, text, reg=None, classes=None, families=None):
    """One case CSV against the fixed format. Returns (errors, meta)."""
    reg = reg if reg is not None else CASE
    classes = CLASSES if classes is None else classes
    families = set(FAMILY) if families is None else families
    errs = []
    cols = reg.get("columns", [])
    rows = list(csv.reader(io.StringIO(text)))
    if not rows or rows[0] != cols:
        return ["%s: header must be exactly %s" % (name, ",".join(cols))], {}
    want = [("meta", m["key"], m["label"], "") for m in reg.get("meta", [])] + \
           [(i["section"], i["key"], i["label"], i["unit"]) for i in reg.get("input", [])]
    body = rows[1:]
    if len(body) != len(want):
        errs.append("%s: %d rows, the format has %d; every key appears once, in the template's order" % (name, len(body), len(want)))
    by_key = {i["key"]: i for i in reg.get("input", [])}
    meta = {}
    for n, (r, w) in enumerate(zip(body, want), start=2):
        if len(r) != len(cols):
            errs.append("%s line %d: %d fields, the format has %d" % (name, n, len(r), len(cols)))
            continue
        sec, key, lab, unit, val, lo, hi, lvl, note = r
        if (sec, key, lab, unit) != w:
            errs.append("%s line %d: %s — section, key, label and unit must read %s" % (name, n, key, ",".join(w)))
            continue
        if sec == "meta":
            meta[key[5:]] = val
            if lo or hi or lvl:
                errs.append("%s: %s takes a value only" % (name, key))
            continue
        i = by_key[key]
        v = _num(val) if val else None
        if val and (v is None or math.isnan(v) or math.isinf(v)):
            errs.append("%s: %s value %r is not a finite number" % (name, key, val))
        if (lo or hi) and not i.get("range"):
            errs.append("%s: %s takes no lo/hi" % (name, key))
        elif bool(lo) != bool(hi):
            errs.append("%s: %s needs both lo and hi, or neither" % (name, key))
        elif lo and hi:
            a, b = _num(lo), _num(hi)
            if a is None or b is None or not a < b:
                errs.append("%s: %s lo %r must be a number below hi %r" % (name, key, lo, hi))
            elif v is not None and not a <= v <= b:
                errs.append("%s: %s value %s lies outside its own lo/hi" % (name, key, val))
        if lvl:
            if not i.get("level"):
                errs.append("%s: %s takes no level; only requirements do" % (name, key))
            elif not (_num(lvl) is not None and 0.0 < _num(lvl) < 100.0):
                errs.append("%s: %s level %r must be a percentage between 0 and 100" % (name, key, lvl))
            elif not val:
                errs.append("%s: %s has a level but no value" % (name, key))
    if meta.get("schema") != reg.get("schema"):
        errs.append("%s: meta.schema must be %s" % (name, reg.get("schema")))
    if not re.fullmatch(r"[a-z0-9_]+", meta.get("case_id", "")):
        errs.append("%s: meta.case_id %r must be lower-case letters, digits and _" % (name, meta.get("case_id")))
    if meta.get("class") and classes and meta["class"] not in classes:
        errs.append("%s: meta.class %r is not in catalogue/classes.toml" % (name, meta["class"]))
    if meta.get("families"):
        bad = set(meta["families"].split(";")) - families
        if bad:
            errs.append("%s: meta.families names unknown families %s" % (name, sorted(bad)))
    return errs, meta


def check_cases():
    errs = []
    ids = {}
    for p in sorted(glob.glob(os.path.join(ROOT, "plan", "cases", "*.csv"))) + \
            sorted(glob.glob(os.path.join(ROOT, "catalogue", "classes", "*.csv"))):
        name = os.path.relpath(p, ROOT)
        e, meta = check_case_csv(name, open(p, encoding="utf-8").read())
        errs += e
        cid = meta.get("case_id")
        if cid in ids:
            errs.append("%s: case_id %s already used by %s" % (name, cid, ids[cid]))
        ids[cid] = name
        if name.startswith("plan") and cid != os.path.basename(p)[:-4]:
            errs.append("%s: case_id %r must match the file name" % (name, cid))
    return errs


def check_classes():
    errs = []
    p = os.path.join(ROOT, "catalogue", "classes.toml")
    if not os.path.exists(p):
        return [] if not os.path.isdir(os.path.join(ROOT, "catalogue")) else ["catalogue/classes.toml missing"]
    for c in load_toml(p).get("class", []):
        if c["id"] in CLASSES:
            errs.append("classes: %s twice" % c["id"])
        CLASSES[c["id"]] = c
        if c.get("source") and c["source"] not in SOURCES:
            errs.append("classes %s: source %r is not a [[source]]" % (c["id"], c["source"]))
        m = c.get("mass_kg", [])
        if len(m) != 2 or (not any(math.isnan(x) for x in m) and not m[0] < m[1]):
            errs.append("classes %s: mass_kg must be [lo, hi] with lo < hi, or nan until D21" % c["id"])
        for st in c.get("standards", []):
            if not os.path.exists(os.path.join(ROOT, st)):
                errs.append("classes %s: standard %s does not exist" % (c["id"], st))
    return errs


def check_class_standards():
    errs = []
    for cid, c in CLASSES.items():
        for st in c.get("standards", []):
            if not os.path.exists(os.path.join(ROOT, st)):
                continue
            _, meta = check_case_csv(st, open(os.path.join(ROOT, st), encoding="utf-8").read())
            if meta.get("class") != cid:
                errs.append("classes %s: its standard names class %r" % (cid, meta.get("class")))
    return errs


def check_algorithms(tree_rows):
    errs = []
    tuned = {s["tree_id"] for s in CASE.get("supplier", []) if s.get("by") == "tuned"}
    for p in sorted(glob.glob(os.path.join(ROOT, "catalogue", "algorithms", "*.toml"))):
        d = load_toml(p)
        aid = d.get("id")
        name = os.path.basename(p)
        if d.get("schema") != "adcs-algorithm/1":
            errs.append("%s: schema must be adcs-algorithm/1" % name)
        if aid != name[:-5]:
            errs.append("%s: id %r must match the file name" % (name, aid))
        ALGORITHMS[aid] = d
        fams = set(d.get("families", []))
        if not fams or not fams <= set(FAMILY):
            errs.append("%s: families %s must be known families" % (name, sorted(fams)))
        if d.get("source") not in SOURCES:
            errs.append("%s: source %r is not a [[source]]" % (name, d.get("source")))
        if not isinstance(d.get("prototype"), bool):
            errs.append("%s: prototype must be true or false" % name)
        num = d.get("number")
        if not isinstance(num, int) or num < 1:
            errs.append("%s: number must be a positive integer, assigned once" % name)
        elif num in NUMBERS:
            errs.append("%s: number %d is already %s's; numbers are never reused" % (name, num, NUMBERS[num]))
        else:
            NUMBERS[num] = aid
        pnums = [pr.get("number") for pr in d.get("param", [])]
        if len(set(pnums)) != len(pnums) or not all(isinstance(n, int) and n >= 1 for n in pnums):
            errs.append("%s: every parameter needs its own positive integer number" % name)
        names = set()
        for pr in d.get("param", []):
            pn = pr.get("name")
            if pn in names:
                errs.append("%s: parameter %s twice" % (name, pn))
            names.add(pn)
            pf = set(pr.get("families", d.get("families", [])))
            if not pf <= fams:
                errs.append("%s %s: families %s outside the algorithm's" % (name, pn, sorted(pf - fams)))
            if pr.get("tree_id") and pr["tree_id"] not in tuned:
                errs.append("%s %s: tree_id %s is not a row supplied by tuning (plan/case_inputs.toml)" % (name, pn, pr["tree_id"]))
            for side in ("lo", "hi"):
                b = pr.get(side)
                if isinstance(b, str):
                    m = re.fullmatch(r"part:([a-z_]+)\.([A-Za-z0-9_]+)", b)
                    if not m:
                        errs.append("%s %s: %s %r is neither a number nor part:<slot>.<field>" % (name, pn, side, b))
                        continue
                    for f in pf:
                        if m.group(1) not in {sl["name"] for sl in FAMILY[f].get("slot", [])}:
                            errs.append("%s %s: %s reads slot %s, which family %s has not" % (name, pn, side, m.group(1), f))
                elif not isinstance(b, (int, float)) or math.isnan(b):
                    errs.append("%s %s: %s must be a number or part:<slot>.<field>" % (name, pn, side))
            if isinstance(pr.get("lo"), (int, float)) and isinstance(pr.get("hi"), (int, float)) and not pr["lo"] < pr["hi"]:
                errs.append("%s %s: lo must be below hi" % (name, pn))
            if pr.get("scale") not in ("lin", "log"):
                errs.append("%s %s: scale %r must be lin or log" % (name, pn, pr.get("scale")))
            if pr.get("scale") == "log" and isinstance(pr.get("lo"), (int, float)) and pr["lo"] <= 0:
                errs.append("%s %s: a log scale needs lo above zero" % (name, pn))
    # every family: its algorithms exist, serve it, and tune every tuned row in play for it
    for fid, fam in FAMILY.items():
        al = fam.get("algorithms", [])
        if "bdot" not in al:
            errs.append("families %s: must carry bdot — the coils own detumble and safe mode (SPEC.md §2)" % fid)
        covered = set()
        for a in al:
            if a not in ALGORITHMS:
                errs.append("families %s: algorithm %s is not in catalogue/algorithms" % (fid, a))
                continue
            if fid not in ALGORITHMS[a].get("families", []):
                errs.append("families %s: algorithm %s does not serve it" % (fid, a))
            for pr in ALGORITHMS[a].get("param", []):
                if pr.get("tree_id") and fid in pr.get("families", ALGORITHMS[a].get("families", [])):
                    covered.add(pr["tree_id"])
        tags = set(fam.get("tags", []))
        for t in tuned:
            extra = tree_rows[t][4]
            if (not extra or set(extra.split(",")) & tags) and t not in covered:
                errs.append("families %s: tuned row %s (%s) is in play but no algorithm of the family tunes it"
                            % (fid, t, tree_rows[t][1]))
    return errs


def check_product(name, d, parts):
    errs = []
    pid = d.get("id")
    if d.get("schema") != "adcs-product/1":
        errs.append("%s: schema must be adcs-product/1" % name)
    fam = FAMILY.get(d.get("family"))
    if fam is None:
        return errs + ["%s: family %r unknown" % (name, d.get("family"))]
    for c in d.get("classes", []):
        if c not in CLASSES:
            errs.append("%s: class %s is not in catalogue/classes.toml" % (name, c))
    st, org = d.get("status"), d.get("origin")
    if st not in ("candidate", "offered", "retired"):
        errs.append("%s: status %r" % (name, st))
    if org not in ("seeded", "designed"):
        errs.append("%s: origin %r" % (name, org))
    al = d.get("algorithms", [])
    proto = [a for a in al if ALGORITHMS.get(a, {}).get("prototype")]
    if proto:
        errs.append("%s: carries prototype algorithm(s) %s, which only the MATLAB twin implements" % (name, proto))
    if not set(al) <= set(fam.get("algorithms", [])) or "bdot" not in al:
        errs.append("%s: algorithms %s must be drawn from the family's %s and include bdot" % (name, al, fam.get("algorithms")))
    counts = dict(d.get("counts", {}))
    from_part = {sl["count"]: sl for sl in fam.get("slot", []) if sl.get("count_from")}
    want = set(fam.get("counts", {})) - set(from_part)
    if set(counts) != want:
        errs.append("%s: counts must set exactly %s (a count_from count comes from the part)" % (name, sorted(want)))
    filled_slots = {f.get("slot") for f in d.get("fill", [])}
    for k, sl in from_part.items():
        counts[k] = 1 if sl["name"] in filled_slots else 0   # stands for "supplied by the part" in the checks below
    for k, (lo, hi) in fam.get("counts", {}).items():
        if k in from_part:
            continue
        v = counts.get(k)
        if v is not None and not (lo <= v <= hi or v == 0):
            errs.append("%s: %s = %s outside the family's [%s, %s]" % (name, k, v, lo, hi))
    slots = {sl["name"]: sl for sl in fam.get("slot", [])}
    filled = {}
    for f in d.get("fill", []):
        sl = slots.get(f.get("slot"))
        if sl is None:
            errs.append("%s: slot %r is not a slot of family %s" % (name, f.get("slot"), d.get("family")))
            continue
        if f.get("slot") in filled:
            errs.append("%s: slot %s filled twice; alternatives are separate products" % (name, f["slot"]))
        filled[f["slot"]] = f.get("part")
        if f.get("part") not in sl.get("parts", []):
            errs.append("%s: %s may not fill slot %s" % (name, f.get("part"), f["slot"]))
    for sn, sl in slots.items():
        if sl.get("required") and sn not in filled:
            errs.append("%s: required slot %s is empty" % (name, sn))
        if sl.get("count"):
            n = counts.get(sl["count"], 0)
            if sn in filled and n < 1:
                errs.append("%s: slot %s is filled but %s is 0" % (name, sn, sl["count"]))
            if sn not in filled and n != 0:
                errs.append("%s: slot %s is empty but %s is %s; absence is a count of zero" % (name, sn, sl["count"], n))
    flown = set(filled.values())
    for m in d.get("mount", []):
        if m.get("part") not in flown:
            errs.append("%s: a mount names %s, which fills no slot" % (name, m.get("part")))
    mounted = {m.get("part") for m in d.get("mount", [])}
    unmounted = sorted(pn for sn, pn in filled.items() if sn != "controller" and pn not in mounted)
    if unmounted and not d.get("mounts_pending"):
        errs.append("%s: %s have no mount; mount them, or say why in mounts_pending" % (name, unmounted))
    if d.get("mounts_pending") and not unmounted:
        errs.append("%s: mounts_pending, but every part is mounted" % name)
    status_of = {pn: parts.get(pn, {}).get("status") for pn in flown}
    if st == "offered":
        weak = sorted(pn for pn, s in status_of.items() if s in ("synthetic", "placeholder"))
        if weak:
            errs.append("%s: offered, but holds %s parts %s; never offered to a client" % (name, "synthetic or placeholder", weak))
        if not d.get("design", {}).get("campaign_hash"):
            errs.append("%s: offered with no design campaign behind it" % name)
        if not d.get("promotion", {}).get("by"):
            errs.append("%s: offered, but no person promoted it (H14)" % name)
        for a in al:
            if "UNCONFIRMED" in ALGORITHMS.get(a, {}).get("confirmed_by", "UNCONFIRMED"):
                errs.append("%s: offered, but algorithm %s's tuning bounds are unconfirmed" % (name, a))
    if org == "designed" and not d.get("design", {}).get("case_hash"):
        errs.append("%s: designed, but its [design] names no case it was designed against" % name)
    return errs


def check_products(parts):
    errs = []
    for p in sorted(glob.glob(os.path.join(ROOT, "catalogue", "products", "*.toml"))):
        d = load_toml(p)
        name = os.path.basename(p)
        if d.get("id") != name[:-5]:
            errs.append("%s: id %r must match the file name" % (name, d.get("id")))
        if d.get("id") in PRODUCTS:
            errs.append("%s: product id twice" % name)
        PRODUCTS[d.get("id")] = d
        errs += check_product(name, d, parts)
    return errs


def _case_refs(x, path=""):
    if isinstance(x, dict):
        for k, v in x.items():
            yield from _case_refs(v, path + "." + k if path else k)
    elif isinstance(x, list):
        for n, v in enumerate(x):
            yield from _case_refs(v, "%s[%d]" % (path, n))
    elif isinstance(x, str) and x.startswith("case:"):
        yield path, x[5:]


def check_scenarios(parts):
    errs = []
    tree = json.load(open(os.path.join(ROOT, "plan", "tree.json"), encoding="utf-8"))
    rows = {r[0]: r for r in tree["HN_SYS"]}
    case_files = {os.path.basename(p)[:-4] for p in glob.glob(os.path.join(ROOT, "plan", "cases", "*.csv"))}
    # Phases copy the package in stages (SPEC.md §19): a directory not copied
    # yet is not checked, and its absence is not a finding.
    for p in sorted(glob.glob(os.path.join(ROOT, "scenarios", "*.toml"))):
        d = load_toml(p)
        name = os.path.basename(p)
        SCENARIOS[d.get("id")] = d
        if d.get("schema") != "adcs-scenario/1":
            errs.append("%s: schema must be adcs-scenario/1" % name)
        if d.get("case") not in case_files:
            errs.append("%s: case %r is not a file in plan/cases/" % (name, d.get("case")))
        fams = d.get("families", [])
        if not fams or not set(fams) <= set(FAMILY):
            errs.append("%s: families %s must be known families" % (name, fams))
        conf = d.get("configuration", {})
        if set(conf) - {"product"}:
            errs.append("%s: [configuration] holds only `product`; parts, counts and mounts live in the product" % name)
        prod = PRODUCTS.get(conf.get("product"))
        if prod is None:
            errs.append("%s: product %r is not in catalogue/products" % (name, conf.get("product")))
        elif prod.get("family") not in fams:
            errs.append("%s: its default product is a %s, not one of its families %s" % (name, prod.get("family"), fams))
        for f in ("altitude_km", "inclination_deg", "eccentricity", "ltan_h"):
            v = d.get("orbit", {}).get(f)
            if v is not None and not (isinstance(v, str) and v.startswith("case:")):
                errs.append("%s: orbit.%s restates a number; the orbit is the case's (case:orbit.*)" % (name, f))
        if not (isinstance(d.get("time", {}).get("epoch"), str) and d["time"]["epoch"] == "case:mission.epoch"):
            errs.append("%s: time.epoch must be case:mission.epoch" % name)
        for path, key in _case_refs(d):
            i = CASE_KEYS.get(key)
            if i is None or i.get("section") == "meta":
                errs.append("%s: %s reads case key %s, which the case format has not" % (name, path, key))
                continue
            for suf, unit in sorted(FIELD_UNIT.items(), key=lambda x: -len(x[0])):
                if path.split(".")[-1].endswith(suf):
                    if i["unit"] != unit:
                        errs.append("%s: %s is in %s but case key %s is in %s" % (name, path, unit, key, i["unit"]))
                    break
        for m in d.get("metric", []):
            for key, want in (("requirement", "target"), ("evidence", "achieved"), ("analysis", None)):
                rid = m.get(key)
                if rid is None:
                    continue
                if rid not in rows:
                    errs.append("%s metric %s: %s %s is not a system-layer row" % (name, m["id"], key, rid))
                elif want and not rows[rid][3].startswith(want):
                    errs.append("%s metric %s: %s %s is a %r row" % (name, m["id"], key, rid, rows[rid][3]))
            if m.get("kind") not in METRIC_KINDS:
                errs.append("%s metric %s: kind %r is not a metric kind" % (name, m["id"], m.get("kind")))
            if m.get("evidence") and KPIS.get(m["evidence"]) and KPIS[m["evidence"]]["metric"] != m.get("kind"):
                errs.append("%s metric %s: kind %s, but plan/kpis.toml says %s supplies %s"
                            % (name, m["id"], m.get("kind"), m["evidence"], KPIS[m["evidence"]]["metric"]))
            if m.get("requirement") and m.get("evidence"):
                if rows.get(m["requirement"], [0, 1])[1] != rows.get(m["evidence"], [0, 2])[1]:
                    errs.append("%s metric %s: requirement and evidence rows name different targets" % (name, m["id"]))
        pr = d.get("parity_reference")
        if pr is not None and pr.get("source") not in SOURCES:
            errs.append("%s: parity_reference cites %r, which is not a [[source]]" % (name, pr.get("source")))
    for p in sorted(glob.glob(os.path.join(ROOT, "campaigns", "*.toml"))):
        d = load_toml(p)
        name = os.path.basename(p)
        sc = SCENARIOS.get(d.get("scenario"))
        if sc is None:
            errs.append("%s: scenario %r does not exist" % (name, d.get("scenario")))
            continue
        if d.get("type") not in ("nominal", "edge", "montecarlo", "sweep", "fault", "labtwin"):
            errs.append("%s: unknown campaign type %r" % (name, d.get("type")))
        if d.get("type") == "sweep" and str(d.get("parameter", "")).startswith("case:"):
            i = CASE_KEYS.get(d["parameter"][5:])
            if i is None or not i.get("range"):
                errs.append("%s: sweeps %s, which is not a case input that takes a range" % (name, d["parameter"]))
        st = d.get("statistic", {}).get("metric")
        if st and st not in {m.get("id") for m in sc.get("metric", [])}:
            errs.append("%s: statistic.metric %s is not a metric of scenario %s" % (name, st, sc.get("id")))
        if d.get("type") == "fault":
            prod = PRODUCTS.get(sc.get("configuration", {}).get("product"), {})
            flown = {f.get("part") for f in prod.get("fill", [])}
            places = {(m.get("part"), m.get("place")) for m in prod.get("mount", [])}
            known = set()
            for dm in glob.glob(os.path.join(ROOT, "rig", "device_maps", "*.toml")):
                known |= {f.get("kind") for f in load_toml(dm).get("fault_point", [])}
            for f in d.get("faults", []):
                part, _, place = f.get("target", "").partition("@")
                if part not in flown:
                    errs.append("%s: fault target %s is not flown by product %s" % (name, part, prod.get("id")))
                elif place and place != "*" and (part, place) not in places:
                    errs.append("%s: fault target %s@%s is not a mount of product %s" % (name, part, place, prod.get("id")))
                if known and f.get("kind") not in known:
                    errs.append("%s: fault kind %s is not a fault_point in any rig device map" % (name, f.get("kind")))
        if d.get("type") == "labtwin" and not os.path.exists(os.path.join(ROOT, d.get("lab", ""))):
            errs.append("%s: lab file %r does not exist" % (name, d.get("lab")))
    for p in sorted(glob.glob(os.path.join(ROOT, "designs", "*.toml"))):
        d = load_toml(p)
        name = os.path.basename(p)
        if d.get("schema") != "adcs-design/1":
            errs.append("%s: schema must be adcs-design/1" % name)
        c = CLASSES.get(d.get("class"))
        if c is None:
            errs.append("%s: class %r unknown" % (name, d.get("class")))
        elif d.get("standard") not in c.get("standards", []):
            errs.append("%s: standard %r is not one of class %s's standards %s" % (name, d.get("standard"), d.get("class"), c.get("standards", [])))
        if not set(d.get("families", [])) <= set(FAMILY) or not d.get("families"):
            errs.append("%s: families %s must be known families" % (name, d.get("families")))
        if d.get("parts") not in ("internal", "offerable"):
            errs.append("%s: parts %r must be internal or offerable" % (name, d.get("parts")))
    return errs


# Each mutation breaks the package in one way the checks exist to catch. The
# selftest applies it to a copy and requires the named finding.
MUTATIONS = [
    ("plan/cases/ais_3u.csv", "orbit.alt,Altitude,Kilometre,", "orbit.alt,Altitude,Metre,", "unit must read"),
    ("plan/cases/ais_3u.csv", "orbit.alt,Altitude,Kilometre,500.0,,", "orbit.alt,Altitude,Kilometre,500.0,400.0,", "needs both lo and hi"),
    ("plan/cases/ais_3u.csv", "orbit.inc,Inclination,Degree,97.4,,,", "orbit.inc,Inclination,Degree,97.4,,,99.0", "takes no level"),
    ("plan/cases/ais_3u.csv", "meta.schema,Case format,,adcs-case/1", "meta.schema,Case format,,adcs-case/2", "meta.schema must be"),
    ("catalogue/classes/cubesat_3u_img.csv", "meta.class,Satellite class,,cubesat_3u", "meta.class,Satellite class,,cubesat_6u", "its standard names class"),
    ("plan/cases/ais_img_3u.csv", "req.ape,Absolute pointing error (APE),Degree,0.01,,,99.73", "req.ape,Absolute pointing error (APE),Degree,0.01,,,100.0", "must be a percentage between 0 and 100"),
    ("catalogue/products/SYN-P-3U-FMR.toml", 'status = "candidate"', 'status = "offered"', "never offered to a client"),
    ("catalogue/products/SYN-P-3U-MTQ.toml", "n_st = 0", "n_st = 1", "absence is a count of zero"),
    ("catalogue/products/SYN-P-3U-FMR.toml", 'slot = "rings"\npart = "SYN-MFP-1"', 'slot = "rings"\npart = "SYN-RW-10"', "may not fill slot"),
    ("scenarios/inertial_hold_3u.toml", 'altitude_km = "case:orbit.alt"', "altitude_km = 500.0", "restates a number"),
    ("scenarios/inertial_hold_3u.toml", 'inclination_deg = "case:orbit.inc"', 'inclination_deg = "case:orbit.incl"', "which the case format has not"),
    ("scenarios/detumble_3u.toml", '"case:mission.w0"', '"case:mission.sangle"', "is in DegreePerSecond but case key"),
    ("catalogue/algorithms/bdot.toml", 'tree_id = ""', 'tree_id = "gc_2"', "not a row supplied by tuning"),
    ("catalogue/algorithms/idmas_split.toml", 'tree_id = "gx_0"', 'tree_id = ""', "no algorithm of the family tunes it"),
    ("plan/case_inputs.toml", '[[supplier]]\ntree_id = "gc_0"', '[[supplier]]\ntree_id = "gc_2"', "suppliers"),
    ("campaigns/detumble_rate_sweep.toml", '"case:mission.w0"', '"case:mission.epoch"', "takes a range"),
    ("catalogue/products/SYN-P-3U-FMR.toml", '[[mount]]\npart = "SYN-GYRO-1"', '[[mount]]\npart = "SYN-GYRO-1-X"', "have no mount"),
    ("catalogue/families.toml", 'count = "n_rcs"\n  count_from', 'count = ""\n  count_from', "every count row that can be above zero must belong to exactly one slot"),
    ("plan/seed_content.toml", 'zero_when_absent = ["cf_3"]', 'zero_when_absent = []', "declares zero_when_absent"),
    ("plan/seed_content.toml", 'physics = "fmr::spin_down_time"', 'physics = "fmr::spin_down"', "which plan/physics.toml does not have"),
    ("plan/units.toml", 'name = "MillinewtonMetreSecond"\nsymbol = "mN.m.s"\nsi = 0.001\nquantity = ["AngularMomentum"]',
     'name = "MillinewtonMetreSecond"\nsymbol = "mN.m.s"\nsi = 0.001\nquantity = ["Torque"]', "which states ['Torque']"),
    ("catalogue/algorithms/mekf.toml", "prototype = false", "prototype = true", "carries prototype algorithm"),
    ("catalogue/algorithms/mekf.toml", "number = 2                         # the algorithm id", "number = 1                         # the algorithm id", "numbers are never reused"),
    ("campaigns/inertial_hold_mc500.toml", 'metric = "ape_last_half_orbit"', 'metric = "ape_whole_run"', "is not a metric of scenario"),
    ("campaigns/ring_freeze_fault.toml", "SYN-MFP-1@+Y", "SYN-MFP-1@+W", "is not a mount of product"),
    ("rig/labs/syn_lab.toml", "faintest_magnitude = 6.0\n", "", "needs the field star_stimulator.faintest_magnitude"),
    ("plan/case_inputs.toml", 'tree_id = "rk1_0"\nlabel = "Risks open"\nby = "derisk"', 'tree_id = "rk1_0"\nlabel = "Risks open"\nby = "product"', "the de-risking ledger must be its one supplier"),
    ("plan/seed_content.toml", 'question = "How many risks in the register are open?"', 'question = "How many risks in the register are open?"\nvalue = 3.0', "must not give it a value"),
]


def selftest():
    import shutil
    import subprocess
    import tempfile
    bad = {"_source": "", "_note": "", "layer3_shape": [], "HN_MGT": [["mgm", "x", None, "", ""]],
           "ED_MGT": [], "HN_SYS": [["prg", "x", None, "", ""], ["g", "G", "prg", "", ""],
                                    ["g_0", "a", "g", "computed", ""]], "ED_SYS": [], "VE": [], "KE": []}
    errs, _ = check_tree(bad)
    assert any("computed with no input" in e for e in errs), errs
    assert any("exactly one is allowed" in e for e in errs), errs
    bad["VE"] = [["g_0", "g_0", "self"]]
    errs, _ = check_tree(bad)
    assert any("cycle" in e for e in errs), errs
    here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    failed, skipped = [], []
    for path, old, new, want in MUTATIONS:
        with tempfile.TemporaryDirectory() as tmp:
            for d in ("plan", "catalogue", "scenarios", "campaigns", "designs", "rig", "tools"):
                if os.path.isdir(os.path.join(here, d)):
                    shutil.copytree(os.path.join(here, d), os.path.join(tmp, d))
            f = os.path.join(tmp, path)
            if not os.path.exists(f):
                skipped.append(path)
                continue
            text = open(f, encoding="utf-8").read()
            if text.count(old) != 1:
                failed.append("%s: the mutation's text is not there exactly once: %r" % (path, old))
                continue
            open(f, "w", encoding="utf-8").write(text.replace(old, new))
            out = subprocess.run([sys.executable, os.path.abspath(__file__)], capture_output=True, text=True,
                                 env=dict(os.environ, ADCS_PLAN_ROOT=tmp)).stdout
            if want not in out:
                failed.append("%s: %r -> %r was not caught (wanted %r)\n%s" % (path, old, new, want, out))
    if failed:
        print("\n".join(failed))
        raise SystemExit("selftest FAILED: %d of %d mutations slipped through" % (len(failed), len(MUTATIONS)))
    print("selftest ok: the checks refuse what they exist to refuse (%d mutations caught%s)"
          % (len(MUTATIONS) - len(skipped), ", %d skipped: their files are not copied yet" % len(skipped) if skipped else ""))


def main():
    if "--selftest" in sys.argv:
        selftest()
        return 0
    t = json.load(open(os.path.join(ROOT, "plan", "tree.json"), encoding="utf-8"))
    errs, ctx = check_tree(t)
    errs += check_seed_content(ctx)
    errs += check_units_physics()
    kerrs, kp = check_kpis(ctx)
    errs += kerrs
    KPIS.update(kp)
    cat = check_catalogue()
    parts = {}
    if isinstance(cat, tuple):
        errs += cat[0]
        parts = cat[1]
    else:
        errs += cat
    errs += check_case_registry(ctx)
    errs += check_classes()
    errs += check_cases()
    errs += check_class_standards()
    errs += check_algorithms({r[0]: r for r in t["HN_SYS"]})
    errs += check_products(parts)
    errs += check_scenarios(parts)
    for e in errs:
        print("FAIL", e)
    leaf = ctx.get("leaf", set())
    kinds = {}
    for i in leaf:
        kinds[ctx["kind"][i]] = kinds.get(ctx["kind"][i], 0) + 1
    print("%d finding(s) · %d rows · leaves by kind %s · %d layer-3 layers"
          % (len(errs), len(ctx.get("rows", {})), dict(sorted(kinds.items())), len(t["layer3_shape"])))
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
