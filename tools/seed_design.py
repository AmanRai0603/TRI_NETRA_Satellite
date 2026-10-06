#!/usr/bin/env python3
"""Seed the design files from the spec: one group file per group, one node file per row of the
tree, and the starting design database, all from what the spec states today (P1).

    python3 tools/seed_design.py [--out build/design]
    python3 tools/seed_design.py --check        seed into a temporary folder and check every file

What each file gets:
  <group>.group.tndb   the group, its stages, its nodes (stage, layer, kind, label, state "shell"),
                       every edge into or inside it (the spec's derivation and KPI-contribution
                       edges, VE and KE)
  <id>.node.tndb       the node row (group and stage from design/groups.toml), its identity, and,
                       for the 82 rows the spec seeds, every field the spec states, each marked
                       with where it came from (`spec:seed_content.toml`, `spec:tree.json`)
  design.tndb          every node as seeded, release "seed", and every edge; and the engine's inputs,
                       every case row by row and every file it reads under data/ (tools/design_inputs.py)

Nothing is invented: a field the spec does not state is left out, and a node with no content is
a shell. Carrying the content over from the code, the twin and the pseudocode is P8.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import pathlib
import sys
import tempfile
import tomllib

import design_inputs
import design_rows
import groups as G
import tndb
from common import ROOT, V1, Steps

PLAN = V1 / "spec" / "plan"
SEED_RELEASE = "seed"


def _seed_content():
    rows = tomllib.loads((PLAN / "seed_content.toml").read_text())["row"]
    return {r["tree_id"]: r for r in rows}


def _edges():
    t = json.loads((PLAN / "tree.json").read_text())
    return [(a, b, "derivation", lab) for a, b, lab in t["VE"]] + [(a, b, "contribution", lab) for a, b, lab in t["KE"]]


def _text(v):
    return v if isinstance(v, str) else json.dumps(v, sort_keys=True)


def node_content(r, seed):
    """[(section, field, value, origin)] the spec states for one row."""
    out = [("identity", "label", r["label"], "spec:tree.json"), ("identity", "sheet", r["sheet"], "spec:tree.json")]
    s = seed.get(r["id"])
    if s:
        for k in sorted(s):
            if k != "tree_id":
                out.append(("spec", k, _text(s[k]), "spec:seed_content.toml"))
    return out


def seed(out, sync=True):
    out = pathlib.Path(out)
    (out / "structure").mkdir(parents=True, exist_ok=True)
    (out / "nodes").mkdir(parents=True, exist_ok=True)
    s = tndb.schema()
    rows, g = design_rows.rows(), G.load()
    errs = G.check(rows, g)
    if errs:
        raise SystemExit("seed_design: the group map does not hold: " + "; ".join(errs))
    placed = G.place(rows, g)
    grp_of = {r["id"]: placed[r["id"]][0] for r in rows}
    gdef = {x["id"]: x for x in g["group"]}
    stage = {r["id"]: G.stage_of(r, gdef[grp_of[r["id"]]]) for r in rows}
    sc, edges = _seed_content(), _edges()

    def group_file(x, conn):
        conn.execute("INSERT INTO group_info VALUES (?, ?, ?, ?)", (x["id"], x["label"], x["lead"], None))
        conn.executemany("INSERT INTO stage VALUES (?, ?, ?)", [(st["id"], st["label"], None) for st in x.get("stages", [])])
        mine = [r for r in rows if grp_of[r["id"]] == x["id"]]
        conn.executemany("INSERT INTO group_node VALUES (?, ?, ?, ?, ?, ?, ?)",
                         [(r["id"], r["sheet"], stage[r["id"]], str(r["layer"]), r["kind"], r["label"], "shell") for r in mine])
        ids = {r["id"] for r in mine}
        conn.executemany("INSERT INTO edge VALUES (?, ?, ?, ?)", [e for e in edges if e[1] in ids])

    def node_file(r, conn):
        conn.execute("INSERT INTO node VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                     (r["id"], r["sheet"], grp_of[r["id"]], stage[r["id"]], str(r["layer"]), r["kind"], r["label"], "shell", None, 0))
        conn.executemany("INSERT INTO content VALUES (?, ?, ?, ?)", node_content(r, sc))

    def design_file(conn):
        conn.executemany("INSERT INTO design_group VALUES (?, ?, ?, ?)", [(x["id"], SEED_RELEASE, None, None) for x in g["group"]])
        conn.executemany("INSERT INTO design_node VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                         [(r["id"], grp_of[r["id"]], stage[r["id"]], str(r["layer"]), r["kind"], r["label"],
                           json.dumps([list(c) for c in node_content(r, sc)], sort_keys=True), SEED_RELEASE) for r in rows])
        conn.executemany("INSERT INTO edge VALUES (?, ?, ?, ?)", edges)
        design_inputs.fill(conn)

    for x in g["group"]:
        tndb.create(out / "structure" / f"{x['id']}.group.tndb", "group", x["id"], s=s, fill=lambda c, x=x: group_file(x, c), sync=sync)
    for r in rows:
        tndb.create(out / "nodes" / f"{r['id']}.node.tndb", "node", r["id"], s=s, fill=lambda c, r=r: node_file(r, c), sync=sync)
    tndb.create(out / "design.tndb", "design", "design", s=s, fill=design_file, sync=sync)
    return out


def check_tree(out):
    files = sorted(pathlib.Path(out).rglob("*.tndb"))
    errs = [e for f in files for e in tndb.check(f)]
    return files, errs


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", default=str(ROOT / "build" / "design"), help="where to write (default build/design)")
    ap.add_argument("--check", action="store_true", help="seed into a temporary folder and check every file")
    a = ap.parse_args(argv)
    S = Steps("seed_design.py", "seed-design")
    if a.check:
        with tempfile.TemporaryDirectory() as tmp:
            S(1, "20 group files, 734 node files, design.tndb")
            seed(tmp, sync=False)
            S(2)
            files, errs = check_tree(tmp)
    else:
        out = pathlib.Path(a.out)
        if out.exists() and any(out.iterdir()):
            raise SystemExit(f"seed_design: {out} is not empty; seeding never overwrites design files (remove it, or pass --out)")
        S(1, "20 group files, 734 node files, design.tndb")
        seed(out)
        S(2)
        files, errs = check_tree(out)
    for e in errs:
        print("seed_design: " + e, file=sys.stderr)
    print(f"seed_design: {len(files)} files, {len(errs)} problem(s)")
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
