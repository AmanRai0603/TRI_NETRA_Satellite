#!/usr/bin/env python3
"""Sealed group releases (docs/RELEASE_PLAN.md P6), checked from Python: a second, separate judge
of what the group app (design/js/release.js) writes.

    python3 tools/release.py check PATH...   each release file (or every one under a folder's
                                             releases/): exit 1 on any problem
    python3 tools/release.py list DIR        every release of the folder: group, version, who,
                                             nodes, how many confirmed

A release file, releases/<group>-<version>.tnrel, is frozen. The rules:
  - it passes tools/tndb.py check, and is named for its group and version;
  - every node's fingerprint is the SHA-256 of its content, and the release's fingerprint the
    SHA-256 of the lines "<id> <fingerprint>", sorted, joined by a newline;
  - every node's body fingerprint is the SHA-256 of its body;
  - its nodes are the group's nodes as sealed (every one not archived), each sealed as
    "confirmed" or "unconfirmed", an unconfirmed one with why;
  - a confirmed node was checked by someone other than its author, and a computing node has a
    test vector whose answer comes from outside the code;
  - the lead's seal is a signature naming the version and the release's fingerprint.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import hashlib
import json
import pathlib
import re
import sqlite3
import sys

import node_catalog
import tndb

FIXED_KINDS = ("interface", "closure_interface", "required", "achieved")


def _sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _rows(conn, sql):
    conn.row_factory = sqlite3.Row
    return [dict(r) for r in conn.execute(sql)]


def computing(body, rows):
    """Whether a node computes (the node app's kindOf, in Python): the spec's catalogue says so,
    or, for a row it has not named, its author chose it."""
    node = body.get("node") or {}
    kind = node.get("kind") or ""
    if kind in FIXED_KINDS or kind.startswith("closure"):
        return False
    spec = rows.get(node.get("id"))
    if spec:
        return spec[1] == "computed"
    chosen = {f"{s}.{f}": v for s, f, v, _o in body.get("content", [])}.get("identity.form_kind")
    return chosen == "computed"


def check(path, rows=None):
    """The problems of one release file."""
    path = pathlib.Path(path)
    problems = [str(p) for p in tndb.check(path)]
    if problems:
        return problems
    rows = rows if rows is not None else node_catalog.catalogue()["rows"]
    with sqlite3.connect(path) as c:
        rel = _rows(c, "SELECT * FROM release")
        if len(rel) != 1:
            return [f"{path}: {len(rel)} release rows, not one"]
        rel = rel[0]
        if path.name != f"{rel['group_id']}-{rel['version']}.tnrel":
            problems.append(f"{path}: holds {rel['group_id']} {rel['version']}; the file is named for another")
        if not re.fullmatch(r"\d+\.\d+", rel["version"] or ""):
            problems.append(f"{path}: version {rel['version']!r} is not MAJOR.MINOR")
        lines, ids = [], set()
        for rn in _rows(c, "SELECT id, fingerprint, content FROM release_node ORDER BY id"):
            nid, where = rn["id"], f"{path}: {rn['id']}"
            ids.add(nid)
            lines.append(f"{nid} {rn['fingerprint']}")
            if _sha(rn["content"]) != rn["fingerprint"]:
                problems.append(f"{where}: its fingerprint is not the SHA-256 of its content")
            try:
                x = json.loads(rn["content"])
                body = json.loads(x["body"])
            except (ValueError, KeyError, TypeError) as e:
                problems.append(f"{where}: its content is not a sealed node ({e})")
                continue
            if _sha(x["body"]) != x.get("body_fingerprint"):
                problems.append(f"{where}: its body fingerprint is not the SHA-256 of its body")
            if (body.get("node") or {}).get("id") != nid:
                problems.append(f"{where}: its body is about {(body.get('node') or {}).get('id')!r}")
            sealed_as = x.get("sealed_as")
            if sealed_as not in ("confirmed", "unconfirmed"):
                problems.append(f"{where}: sealed as {sealed_as!r}, not confirmed or unconfirmed")
            elif sealed_as == "unconfirmed" and not x.get("why"):
                problems.append(f"{where}: sealed unconfirmed with no reason given")
            elif sealed_as == "confirmed":
                author = (body.get("node") or {}).get("author")
                checkers = [s[1] for s in body.get("signature", []) if s[0] == "checked by"]
                if not checkers or checkers[-1] == author:
                    problems.append(f"{where}: sealed as confirmed, but nobody other than its author ({author}) checked it")
                if computing(body, rows) and not any(f[5] for f in body.get("fixture", [])):
                    problems.append(f"{where}: a computing node sealed as confirmed with no test vector from outside the code")
        if _sha("\n".join(sorted(lines))) != rel["fingerprint"]:
            problems.append(f"{path}: the release fingerprint is not the SHA-256 of its nodes' fingerprints")
        live = {r["id"] for r in _rows(c, "SELECT id, state FROM group_node") if r["state"] != "archived"}
        if live != ids:
            problems.append(f"{path}: its nodes are not the group's: missing {sorted(live - ids)[:5]}, extra {sorted(ids - live)[:5]}")
        seals = [s for s in _rows(c, "SELECT * FROM signature WHERE role = 'sealed'")]
        st = {}
        try:
            st = json.loads(seals[-1]["statement"]) if seals else {}
        except ValueError:
            st = {}
        if not seals or st.get("version") != rel["version"] or st.get("fingerprint") != rel["fingerprint"] or seals[-1]["name"] != rel["sealed_by"]:
            problems.append(f"{path}: no seal by {rel['sealed_by']} naming {rel['version']} and its fingerprint")
    return problems


def files(paths):
    out = []
    for p in map(pathlib.Path, paths):
        if p.is_dir():
            out += sorted((p / "releases").glob("*.tnrel")) if (p / "releases").is_dir() else sorted(p.glob("*.tnrel"))
        else:
            out.append(p)
    return out


def listing(root):
    out = []
    for f in files([root]):
        with sqlite3.connect(f) as c:
            r = _rows(c, "SELECT * FROM release")[0]
            sealed = [json.loads(x)["sealed_as"] for (x,) in c.execute("SELECT content FROM release_node")]
        out.append((r["group_id"], r["version"], r["sealed_at"], r["sealed_by"], len(sealed), sealed.count("confirmed")))
    return sorted(out, key=lambda r: (r[0], [int(x) for x in r[1].split(".")]))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0], formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("check", help="check release files (or every one under a design folder)")
    c.add_argument("paths", nargs="+")
    li = sub.add_parser("list", help="every release of a design folder")
    li.add_argument("dir")
    a = ap.parse_args(argv)
    if a.cmd == "check":
        fs, rows, problems = files(a.paths), node_catalog.catalogue()["rows"], []
        for f in fs:
            problems += check(f, rows)
        for p in problems:
            print(p)
        print(f"release: {len(fs)} file(s), {len(problems)} problem(s)")
        return 1 if problems else 0
    for g, v, at, by, n, conf in listing(a.dir):
        print(f"{g:12} {v:6} {at}  {by:24} {conf}/{n} confirmed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
