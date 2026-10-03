#!/usr/bin/env python3
"""The group map (`design/groups.toml`) against every row of the tree (`tools/design_rows.py`).

    python3 tools/groups.py              each group: its lead, its rows, its stages
    python3 tools/groups.py --check      exit 1 unless every row is in exactly one group, every
                                         group holds the rows it states, every stage sits inside
                                         its group, and every override and boundary names real
                                         rows and groups
    python3 tools/groups.py --row ID     where one row goes, and why

How a row is placed (the file's header): an override that names it; else, for a layer-3 required
or achieved row, the group of the target it answers; else the group whose branches hold its branch.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import collections
import sys
import tomllib

import design_rows
from common import ROOT

GROUPS = ROOT / "design" / "groups.toml"


def load(path=GROUPS):
    g = tomllib.loads(path.read_text())
    if g.get("schema") != "trinetra-groups/1":
        raise SystemExit(f"groups: {path} is not trinetra-groups/1")
    return g


def place(rows, g):
    """{row id: (group id, how)}; a row no rule reaches is left out (check() names it)."""
    by_branch, override = {}, {}
    for grp in g["group"]:
        for b in grp["branches"]:
            if b in by_branch:
                raise SystemExit(f"groups: branch {b} is in both {by_branch[b]} and {grp['id']}")
            by_branch[b] = grp["id"]
    for o in g.get("override", []):
        for i in o["ids"]:
            override[i] = o["group"]
    out = {}

    def one(r):
        if r["id"] in override:
            return override[r["id"]], "override"
        if r["kind"] in ("required", "achieved") and r["target"]:
            t = next((x for x in rows if x["id"] == r["target"]), None)
            if t:
                got = one(t)
                if got:
                    return got[0], f"follows its target {r['target']}"
        if r["branch"] in by_branch:
            return by_branch[r["branch"]], f"branch {r['branch']}"
        return None

    for r in rows:
        got = one(r)
        if got:
            out[r["id"]] = got
    return out


def stage_of(r, grp):
    for s in grp.get("stages", []):
        if r["branch"] in s["branches"]:
            return s["id"]
    return None


def check(rows, g):
    errs = []
    ids = {r["id"] for r in rows}
    gids = [grp["id"] for grp in g["group"]]
    if len(gids) != len(set(gids)):
        errs.append("a group id is used twice")
    placed = place(rows, g)
    for r in rows:
        if r["id"] not in placed:
            errs.append(f"row {r['id']} (branch {r['branch']}) is in no group")
    count = collections.Counter(grp for grp, _ in placed.values())
    for grp in g["group"]:
        if count[grp["id"]] != grp["nodes"]:
            errs.append(f"group {grp['id']} holds {count[grp['id']]} rows, the file says {grp['nodes']}")
        for s in grp.get("stages", []):
            stray = [b for b in s["branches"] if b not in grp["branches"]]
            if stray:
                errs.append(f"group {grp['id']} stage {s['id']}: branches {', '.join(stray)} are not the group's")
        for side in ("c", "rust", "twin"):
            if side not in grp.get("modules", {}):
                errs.append(f"group {grp['id']}: no {side} modules listed (an empty list says it has none)")
    for o in g.get("override", []):
        if o["group"] not in gids:
            errs.append(f"override to unknown group {o['group']}")
        errs += [f"override names unknown row {i}" for i in o["ids"] if i not in ids]
    for b in g.get("boundary", []):
        errs += [f"boundary names unknown group {x}" for x in b["between"] if x != "*" and x not in gids]
    if sum(grp["nodes"] for grp in g["group"]) != len(rows):
        errs.append(f"the groups state {sum(grp['nodes'] for grp in g['group'])} rows, the tree has {len(rows)}")
    return errs


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--check", action="store_true", help="exit 1 on any problem")
    ap.add_argument("--row", metavar="ID", help="where one row goes, and why")
    a = ap.parse_args(argv)
    rows, g = design_rows.rows(), load()
    placed = place(rows, g)
    if a.row:
        if a.row not in placed:
            print(f"groups: {a.row} is in no group" if a.row in {r['id'] for r in rows} else f"groups: no row {a.row}")
            return 1
        grp, how = placed[a.row]
        print(f"{a.row}: {grp} ({how})")
        return 0
    errs = check(rows, g)
    if not a.check:
        count = collections.Counter(grp for grp, _ in placed.values())
        for grp in g["group"]:
            st = ", ".join(f"{s['id']} {sum(1 for r in rows if placed.get(r['id'], ('',))[0] == grp['id'] and stage_of(r, grp) == s['id'])}"
                           for s in grp.get("stages", []))
            print(f"{grp['id']:10s} {grp['lead']:13s} {count[grp['id']]:4d}  {grp['label']}" + (f"  [{st}]" if st else ""))
    for e in errs:
        print("groups: " + e, file=sys.stderr)
    if a.check and not errs:
        print(f"groups: {len(rows)} rows in {len(g['group'])} groups, each in exactly one")
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
