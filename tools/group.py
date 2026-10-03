#!/usr/bin/env python3
"""The structure of a design folder (docs/RELEASE_PLAN.md P4), checked from Python: the same rules
the group app holds every change to (design/js/structure.js integrity()), written a second time so
neither implementation is the only judge of the other.

    python3 tools/group.py check DIR       every rule over the folder; exit 1 on any problem
    python3 tools/group.py list DIR        the groups: nodes, stages, edges, open change requests

A design folder (tools/seed_design.py writes one): structure/<group>.group.tndb,
structure/actions/<id>.json, nodes/<id>.node.tndb. The rules:
  - every group file and node file passes tools/tndb.py check;
  - every node in exactly one group; its node file there, saying that group, stage, label, state;
  - every node file in a group;
  - a node's stage one of its group's stages (none when the group has none);
  - an edge kept by the group of the node that reads, from a node that exists, is not itself, is
    not archived (unless the reader is), and is not there twice;
  - every author, contract and stage owner about the group's own nodes and members;
  - no structure action left unfinished (structure/actions/*.json with done false).

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import pathlib
import sqlite3
import sys

import tndb


def _rows(conn, sql):
    conn.row_factory = sqlite3.Row
    return [dict(r) for r in conn.execute(sql)]


def load(root):
    """{group id: {...}} and problems, from the folder's group files."""
    root = pathlib.Path(root)
    groups, problems = {}, []
    for f in sorted((root / "structure").glob("*.group.tndb")):
        errs = tndb.check(f)
        if errs:
            problems += errs
            continue
        conn = sqlite3.connect(f"file:{f}?mode=ro", uri=True)
        try:
            info = (_rows(conn, "SELECT * FROM group_info") or [{}])[0]
            gid = info.get("id") or f.name[: -len(".group.tndb")]
            if f.name != f"{gid}.group.tndb":
                problems.append(f"structure/{f.name}: its group is {gid}; the file is named for another")
            groups[gid] = {
                "stages": _rows(conn, "SELECT * FROM stage"),
                "nodes": {r["id"]: r for r in _rows(conn, "SELECT * FROM group_node")},
                "edges": _rows(conn, "SELECT * FROM edge"),
                "contracts": _rows(conn, "SELECT * FROM contract"),
                "members": {r["name"] for r in _rows(conn, "SELECT * FROM member")},
                "member_nodes": _rows(conn, "SELECT * FROM member_node"),
                "crs": _rows(conn, "SELECT * FROM change_request"),
            }
        finally:
            conn.close()
    return groups, problems


def check(root):
    root = pathlib.Path(root)
    groups, problems = load(root)
    owner = {}
    for gid, g in groups.items():
        for nid in g["nodes"]:
            if nid in owner:
                problems.append(f"node {nid} is in two groups: {owner[nid]} and {gid}")
            owner.setdefault(nid, gid)
    for gid, g in groups.items():
        stages = {s["id"] for s in g["stages"]}
        for n in g["nodes"].values():
            if n["stage"] and n["stage"] not in stages:
                problems.append(f"{gid}: node {n['id']} is in stage {n['stage']}, which the group does not have")
            if not n["stage"] and stages:
                problems.append(f"{gid}: node {n['id']} is in no stage; the group has stages")
        seen = set()
        for e in g["edges"]:
            k = (e["from_node"], e["to_node"], e["kind"])
            if k in seen:
                problems.append(f"{gid}: edge {e['from_node']} -> {e['to_node']} ({e['kind']}) twice")
            seen.add(k)
            if e["to_node"] not in g["nodes"]:
                problems.append(f"{gid}: keeps the edge {e['from_node']} -> {e['to_node']}, but {e['to_node']} is not its node")
            fg = owner.get(e["from_node"])
            if fg is None:
                problems.append(f"{gid}: {e['to_node']} reads {e['from_node']}, which is in no group")
            elif groups[fg]["nodes"][e["from_node"]]["state"] == "archived" and g["nodes"].get(e["to_node"], {}).get("state") != "archived":
                problems.append(f"{gid}: {e['to_node']} reads {e['from_node']}, which {fg} archived")
            if e["from_node"] == e["to_node"]:
                problems.append(f"{gid}: {e['to_node']} reads itself")
        for m in g["member_nodes"]:
            if m["node"] not in g["nodes"]:
                problems.append(f"{gid}: {m['author']} authors {m['node']}, which is not its node")
        for c in g["contracts"]:
            if c["node"] not in g["nodes"]:
                problems.append(f"{gid}: a contract on {c['node']}, which is not its node")
        for s in g["stages"]:
            if s["owner"] and s["owner"] not in g["members"]:
                problems.append(f"{gid}: stage {s['id']} is owned by {s['owner']}, who is not a member")
        for n in g["nodes"].values():
            f = root / "nodes" / f"{n['id']}.node.tndb"
            if not f.is_file():
                problems.append(f"{gid}: node {n['id']} has no file nodes/{f.name}")
                continue
            errs = tndb.check(f)
            if errs:
                problems += errs
                continue
            conn = sqlite3.connect(f"file:{f}?mode=ro", uri=True)
            try:
                row = (_rows(conn, "SELECT * FROM node") or [None])[0]
            finally:
                conn.close()
            if row is None:
                problems.append(f"nodes/{f.name}: no node row")
                continue
            for k, want in (("id", n["id"]), ("group_id", gid), ("stage", n["stage"]), ("label", n["label"]), ("state", n["state"])):
                if row[k] != want:
                    problems.append(f"nodes/{f.name}: {k} is {row[k]!r}, its group {gid} says {want!r}")
    for f in sorted((root / "nodes").glob("*.node.tndb")):
        if f.name[: -len(".node.tndb")] not in owner:
            problems.append(f"nodes/{f.name}: in no group")
    for f in sorted((root / "structure" / "actions").glob("*.json")):
        try:
            if not json.loads(f.read_text()).get("done"):
                problems.append(f"structure/actions/{f.name}: a structure action not finished (the group app finishes it)")
        except ValueError:
            problems.append(f"structure/actions/{f.name}: not readable")
    return problems


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name, help_ in (("check", "every rule over the folder"), ("list", "the groups at a glance")):
        p = sub.add_parser(name, help=help_)
        p.add_argument("dir")
    a = ap.parse_args(argv)
    if a.cmd == "list":
        groups, problems = load(a.dir)
        for gid, g in groups.items():
            live = sum(1 for n in g["nodes"].values() if n["state"] != "archived")
            opened = sum(1 for c in g["crs"] if c["state"] == "open")
            print(f"{gid:10} {live:4} nodes ({len(g['nodes']) - live} archived)  {len(g['stages'])} stages  {len(g['edges'])} edges  {opened} open requests")
        return 1 if problems else 0
    problems = check(a.dir)
    for p in problems:
        print("group: " + p, file=sys.stderr)
    print(f"group: {a.dir}: {len(problems)} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
