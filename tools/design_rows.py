#!/usr/bin/env python3
"""Every row of the ADCS tree, from the spec package, with the id the design files give it.

The spec (`spec/plan/tree.json`, `kpis.toml`, `expected_node_ids.json`; SPEC.md §5) names the
rows of layers 1 and 2 in full, gives each subsystem layer its shape only, and lets the seeder
write the closures. This module turns that into one list of 734 rows, each with:

    id        the short id (`gf_7`, `l3_fmr_interface`, `kpi_<slug>_verified`)
    sheet     the long id a node file carries (`sys_..._ring_spin_down_time` for layers 1–2;
              the short id itself for layer 3 and the closures, which have no long form)
    layer     1, 2, 3 or "closure"
    kind      "leaf" (layers 1–2), "interface", "required", "achieved", "internal" (layer 3),
              "closure_interface", "closure_analysis", "closure_verified"
    branch    the layer-1/2 group it sits under (`sb2`, `fa2`, …) or the subsystem layer (`fmr`)
    target    for a layer-3 required or achieved row, the layer-2 target it answers (`gf_7`)
    label     the spec's label, or "to be named" for an internal subsystem row

Layer-3 ids (the spec fixes only the interface's): `l3_<sid>_interface`, `l3_<sid>_<target>_required`,
`l3_<sid>_<target>_achieved`, and `l3_<sid>_row_<nn>` for the internal rows "to be named" (SPEC.md
§5.4: rows − 1 − 2·targets of them), which are renamed when their content is carried over (P8).

    python3 tools/design_rows.py              the counts by layer and kind
    python3 tools/design_rows.py --list       one line per row

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import collections
import json
import sys
import tomllib

from common import ROOT

PLAN = ROOT / "spec" / "plan"
# SPEC.md §5.1: the seeded sheet counts the list must reproduce
COUNTS = {1: 133, 2: 194, 3: 368, "closure": 39}


def _tree():
    return json.loads((PLAN / "tree.json").read_text())


def rows():
    """All rows of the tree, in tree order, as dicts (see the module's docstring)."""
    t = _tree()
    sheet = json.loads((PLAN / "expected_node_ids.json").read_text())
    out = []
    for layer, key in ((1, "HN_MGT"), (2, "HN_SYS")):
        hn = t[key]
        kids = collections.defaultdict(list)
        for r in hn:
            kids[r[2]].append(r[0])
        parent = {r[0]: r[2] for r in hn}
        for r in hn:
            if kids[r[0]]:
                continue                          # a group, not a sheet
            out.append({"id": r[0], "sheet": sheet[r[0]], "layer": layer, "kind": "leaf",
                        "branch": parent[r[0]], "target": None, "label": r[1]})
    leaves_of = collections.defaultdict(list)
    for r in t["HN_SYS"]:
        leaves_of[r[2]].append(r[0])
    for s in t["layer3_shape"]:
        sid, targets = s["id"], leaves_of[s["group"]]
        if len(targets) != s["targets"]:
            raise SystemExit(f"design_rows: layer {sid} names {s['targets']} targets, its group {s['group']} holds {len(targets)}")
        internal = s["nodes"] - 1 - 2 * len(targets)
        if internal < 0:
            raise SystemExit(f"design_rows: layer {sid} has {s['nodes']} rows, too few for its interface and target pairs")
        add = lambda i, kind, target, label: out.append(
            {"id": i, "sheet": i, "layer": 3, "kind": kind, "branch": sid, "target": target, "label": label})
        add(f"l3_{sid}_interface", "interface", None, f"{s['label']}: interface")
        for g in targets:
            add(f"l3_{sid}_{g}_required", "required", g, f"{s['label']}: required for {g}")
            add(f"l3_{sid}_{g}_achieved", "achieved", g, f"{s['label']}: achieved for {g}")
        for n in range(internal):
            add(f"l3_{sid}_row_{n + 1:02d}", "internal", None, "to be named")
    kpis = tomllib.loads((PLAN / "kpis.toml").read_text())["kpi"]
    out.append({"id": "l3_x_closure_interface", "sheet": "l3_x_closure_interface", "layer": "closure",
                "kind": "closure_interface", "branch": "x_closure", "target": None, "label": "KPI closures: interface"})
    for k in kpis:
        if k["analysis"] != "none":
            i = f"kpi_{k['slug']}_analysis"
            out.append({"id": i, "sheet": i, "layer": "closure", "kind": "closure_analysis", "branch": "x_closure",
                        "target": k["requirement"], "label": f"{k['label']}: analysis closure"})
        i = f"kpi_{k['slug']}_verified"
        out.append({"id": i, "sheet": i, "layer": "closure", "kind": "closure_verified", "branch": "x_closure",
                    "target": k["requirement"], "label": f"{k['label']}: evidence closure"})
    return out


def check(rs):
    """The counts SPEC.md §5.1 states, and every id and sheet id unique."""
    errs = []
    by = collections.Counter(r["layer"] for r in rs)
    for layer, n in COUNTS.items():
        if by[layer] != n:
            errs.append(f"layer {layer}: {by[layer]} rows, the spec says {n}")
    for field in ("id", "sheet"):
        dup = [k for k, n in collections.Counter(r[field] for r in rs).items() if n > 1]
        if dup:
            errs.append(f"{field} not unique: {', '.join(dup[:5])}")
    return errs


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--list", action="store_true", help="one line per row")
    a = ap.parse_args(argv)
    rs = rows()
    if a.list:
        for r in rs:
            print(f"{r['id']:48s} {str(r['layer']):8s} {r['kind']:18s} {r['branch']:10s} {r['label']}")
    errs = check(rs)
    print(f"design_rows: {len(rs)} rows; " + ", ".join(f"layer {k}: {v}" for k, v in collections.Counter(r['layer'] for r in rs).items()),
          file=sys.stderr if a.list else sys.stdout)
    for e in errs:
        print("design_rows: " + e, file=sys.stderr)
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
