#!/usr/bin/env python3
"""The structure of a design folder (docs/RELEASE_PLAN.md P4), checked from Python: the same rules
the group app holds every change to (design/js/structure.js integrity()), written a second time so
neither implementation is the only judge of the other.

    python3 tools/group.py check DIR       every rule over the folder; exit 1 on any problem
    python3 tools/group.py list DIR        the groups: nodes, stages, edges, open change requests
    python3 tools/group.py verify DIR      every group's latest release (releases/) against tools/release.py
                                           and against the folder: its nodes are the group's, what they read
                                           exists, its contracts' readers are groups (docs/RELEASE_PLAN.md P9)
    python3 tools/group.py merge DIR       every group's latest release into DIR/design.tndb, with the
                                           catalogue of every output (the contract at every group boundary);
                                           the previous design.tndb is kept as design.tndb.prev
    python3 tools/group.py impact DIR NODE who reads NODE, in its group and across groups (transitively),
                                           and the contracts on it

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
import re
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


# ------------------------------------------------------------------ releases into the design

def latest_releases(root):
    """{group: path} of each group's newest release in DIR/releases/."""
    out = {}
    for f in sorted((pathlib.Path(root) / "releases").glob("*.tnrel")):
        m = re.fullmatch(r"(.+)-(\d+)\.(\d+)\.tnrel", f.name)
        if m:
            key = (int(m.group(2)), int(m.group(3)))
            if m.group(1) not in out or key > out[m.group(1)][0]:
                out[m.group(1)] = (key, f)
    return {g: f for g, (_, f) in out.items()}


def _release(path):
    with sqlite3.connect(f"file:{path}?mode=ro", uri=True) as c:
        rel = _rows(c, "SELECT * FROM release")[0]
        nodes = {r["id"]: json.loads(r["content"]) for r in _rows(c, "SELECT id, content FROM release_node")}
        return rel, nodes, _rows(c, "SELECT * FROM group_node"), _rows(c, "SELECT * FROM edge"), _rows(c, "SELECT * FROM contract")


def verify(root):
    """Every group's latest release, checked: {group: [problems]} (a group with no release says so)."""
    import release as R
    root = pathlib.Path(root)
    groups, problems = load(root)
    owner = {nid: gid for gid, g in groups.items() for nid in g["nodes"]}
    rels, out = latest_releases(root), {}
    for gid in sorted(groups):
        if gid not in rels:
            out[gid] = [f"{gid}: no release yet (the group app seals one)"]
            continue
        p = list(R.check(rels[gid]))
        rel, nodes, gnodes, edges, contracts = _release(rels[gid])
        if rel["group_id"] != gid:
            p.append(f"{rels[gid].name}: a release of {rel['group_id']}, not {gid}")
        mine = {n for n, g in groups[gid]["nodes"].items() if g["state"] != "archived"}
        for n in sorted(set(nodes) - mine):
            p.append(f"{gid} {rel['version']}: {n} is no longer a node of {gid} (moved or archived since): seal again")
        for n in sorted(mine - set(nodes)):
            p.append(f"{gid} {rel['version']}: {n} is a node of {gid} the release does not hold (added since): seal again")
        for e in edges:
            if e["from_node"] not in owner:
                p.append(f"{gid} {rel['version']}: {e['to_node']} reads {e['from_node']}, which is in no group")
        for c in contracts:
            for r in [x for x in (c["readers"] or "").split(",") if x]:
                if r not in groups:
                    p.append(f"{gid} {rel['version']}: the contract on {c['node']} names {r}, which is not a group")
        out[gid] = p
    return out


def merge(root, *, require_all=False):
    """Every group's latest release into DIR/design.tndb; returns (summary, problems)."""
    root = pathlib.Path(root)
    v = verify(root)
    problems = [x for gid, ps in v.items() for x in ps if not (x.endswith("no release yet (the group app seals one)") and not require_all)]
    if problems:
        return None, problems
    groups, _ = load(root)
    rels = latest_releases(root)
    readers = {}      # node -> {groups that read it}
    for gid, g in groups.items():
        for e in g["edges"]:
            readers.setdefault(e["from_node"], set()).add(gid)
    design_rows, edges, catalogue, dgroups = [], [], [], []
    for gid in sorted(rels):
        rel, nodes, gnodes, gedges, contracts = _release(rels[gid])
        dgroups.append((gid, rel["version"], rel["fingerprint"], rel["sealed_at"]))
        row = {r["id"]: r for r in gnodes}
        cver = {(c["node"], c["output"]): c for c in contracts}
        for nid, x in sorted(nodes.items()):
            r = row[nid]
            design_rows.append((nid, gid, r["stage"], r["layer"], r["kind"], r["label"], json.dumps({"body": json.loads(x["body"]), "sealed_as": x["sealed_as"], "why": x.get("why", [])}, sort_keys=True), f"{gid} {rel['version']}"))
            outs = json.loads(x["body"]).get("output", [])
            for o in outs:
                c = cver.get((nid, o[0])) or cver.get((nid, "*"))
                rd = sorted(readers.get(nid, set()) - {gid})
                catalogue.append((nid, o[0], o[1], c["version"] if c else 0, ",".join(rd)))
        edges += [(e["from_node"], e["to_node"], e["kind"], e["label"]) for e in gedges]
    db = root / "design.tndb"
    if db.exists():
        db.replace(root / "design.tndb.prev")

    def fill(c):
        c.executemany("INSERT INTO design_group VALUES (?, ?, ?, ?)", dgroups)
        c.executemany("INSERT INTO design_node VALUES (?, ?, ?, ?, ?, ?, ?, ?)", design_rows)
        c.executemany("INSERT INTO edge VALUES (?, ?, ?, ?)", edges)
        c.executemany("INSERT INTO catalogue_output VALUES (?, ?, ?, ?, ?)", catalogue)
    tndb.create(db, "design", "design", fill=fill, sync=False)
    return {"groups": len(dgroups), "nodes": len(design_rows), "edges": len(edges), "outputs": len(catalogue),
            "not_released": sorted(set(groups) - set(rels))}, []


def impact(root, node):
    """Who reads `node`: [(depth, group, reader)], and the contracts on it."""
    groups, _ = load(root)
    owner = {nid: gid for gid, g in groups.items() for nid in g["nodes"]}
    reads = {}
    for g in groups.values():
        for e in g["edges"]:
            reads.setdefault(e["from_node"], []).append(e["to_node"])
    out, seen, frontier, depth = [], {node}, [node], 0
    while frontier:
        depth += 1
        nxt = []
        for n in frontier:
            for r in sorted(reads.get(n, [])):
                if r not in seen:
                    seen.add(r)
                    out.append((depth, owner.get(r, "?"), r))
                    nxt.append(r)
        frontier = nxt
    contracts = [c for g in groups.values() for c in g["contracts"] if c["node"] == node]
    return owner.get(node), out, contracts


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name, help_ in (("check", "every rule over the folder"), ("list", "the groups at a glance"),
                        ("verify", "every group's latest release against the folder"), ("merge", "every latest release into design.tndb")):
        p = sub.add_parser(name, help=help_)
        p.add_argument("dir")
        if name == "merge":
            p.add_argument("--require-all", action="store_true", help="refuse unless every group has a release")
    p = sub.add_parser("impact", help="who reads a node, across groups")
    p.add_argument("dir")
    p.add_argument("node")
    a = ap.parse_args(argv)
    if a.cmd == "verify":
        bad = 0
        for gid, ps in verify(a.dir).items():
            print(f"{gid:10} " + ("ok" if not ps else "; ".join(ps)))
            bad += len(ps)
        return 1 if bad else 0
    if a.cmd == "merge":
        summary, problems = merge(a.dir, require_all=a.require_all)
        for p_ in problems:
            print("group: " + p_, file=sys.stderr)
        if problems:
            return 1
        print(f"group: merged {summary['groups']} group release(s), {summary['nodes']} nodes, {summary['edges']} edges, {summary['outputs']} outputs in the catalogue into {a.dir}/design.tndb"
              + (f"; not released yet: {', '.join(summary['not_released'])}" if summary["not_released"] else ""))
        return 0
    if a.cmd == "impact":
        g, rs, cs = impact(a.dir, a.node)
        print(f"{a.node} (group {g}): read by {len(rs)} node(s), {len({x[1] for x in rs} - {g})} other group(s)")
        for d, gid, r in rs:
            print(f"  {'  ' * (d - 1)}{r} ({gid}){'  [other group]' if gid != g else ''}")
        for c in cs:
            print(f"  contract {c['node']}.{c['output']} v{c['version']} read by {c['readers'] or 'nobody yet'}")
        return 0
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
