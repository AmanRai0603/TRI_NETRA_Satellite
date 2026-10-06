#!/usr/bin/env python3
"""Today's design, built from the drive's folder (docs/PLAN_2_0.md S4; docs/OPERATING_2_0.md §5): the one
database every program reads, made from every group's latest sealed release that passes its checks.

    python3 tools/design_build.py DRIVE --out design.tndb [--export DIR] [--at ISO-TIME]

DRIVE is the shared drive's folder (or a converted design, tools/convert_2_0.py): groups/<group>/
with its group file and releases/, and cases/.

What it writes:
  - design.tndb (format 3): every group's release it used (design_group), every node as released
    (design_node, as tools/group.py merge writes it), the wires, the catalogue of outputs, which
    design this is (design_release: kind "today", with the releases it used) and what it needs to run
    (toolbox, application); and the engine's inputs: every case line by line (design_case) and every
    file under data/ (engine_input), each GENERATED from the design's own blocks and case files.
  - with --export DIR, the same inputs as a data folder (DIR/data/..., DIR/cases/*.csv): what the
    MATLAB twin and the Python tools read, generated from the design, never edited.

Which release of a group: its newest under releases/ that tools/release.py finds sound. A group whose
newest is refused keeps its last good one, and the refusal is named; a group with none is left out and
named. The engine's inputs come only from the releases used: a refused release changes nothing.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import hashlib
import json
import pathlib
import re
import sqlite3
import sys
import tomllib

import design_inputs
import gen_fsw_params
import release
import tndb

# The engine's data folder, made from the design: the folder a file goes to, from the block's table
# name or the case file's kind. Every catalogue TOML gives JSON as tools/export_catalogue.py writes
# it; the datasheets, the node registry and the scenario format are JSON already.
TOML_KINDS = ("parts", "products", "algorithms", "modes", "components")
CASE_KINDS = {"scenario": "scenarios", "campaign": "campaigns", "trade": "trades"}
TOP_LEVEL = ("families", "classes")       # the catalogue files the engine reads from the top of data/


def _rows(conn, sql, args=()):
    conn.row_factory = sqlite3.Row
    return [dict(r) for r in conn.execute(sql, args)]


def _version_key(path):
    m = re.fullmatch(r".+-(\d+)\.(\d+)\.tnrel", path.name)
    return (int(m.group(1)), int(m.group(2))) if m else (-1, -1)


def releases_used(drive):
    """{group: (path or None, [what was refused and why])}: each group's newest sound release."""
    out = {}
    for gd in sorted((pathlib.Path(drive) / "groups").iterdir()):
        if not gd.is_dir():
            continue
        refused, chosen = [], None
        for r in sorted((gd / "releases").glob("*.tnrel"), key=_version_key, reverse=True):
            p = release.check(r, rows={})
            if p:
                refused.append(f"{r.name}: refused ({p[0]}{'; …' if len(p) > 1 else ''}); today's design keeps the last good release")
                continue
            chosen = r
            break
        out[gd.name] = (chosen, refused if chosen or refused else [f"{gd.name}: no sealed release yet"])
    return out


def _bodies(rel):
    with sqlite3.connect(f"file:{rel}?mode=ro", uri=True) as c:
        info = _rows(c, "SELECT * FROM release")[0]
        nodes = {r["id"]: json.loads(r["content"]) for r in _rows(c, "SELECT id, content FROM release_node")}
        gnodes = {r["id"]: r for r in _rows(c, "SELECT * FROM group_node")}
        edges = _rows(c, "SELECT * FROM edge")
        contracts = _rows(c, "SELECT * FROM contract")
    return info, nodes, gnodes, edges, contracts


def engine_inputs(bodies, cases):
    """{path in the data folder: bytes} generated from the design's lookup blocks and case files."""
    files = {}
    for nid, body in bodies.items():
        for sec, field, value, origin in body.get("content", []):
            if sec != "table" or not origin:
                continue
            src = origin.split(" ")[0]
            if src.startswith("catalogue/") and src.endswith(".toml"):
                d = tomllib.loads(value)
                parts = src.split("/")
                text = json.dumps(d, indent=1, sort_keys=True) + "\n"
                if len(parts) == 3 and parts[1] in TOML_KINDS:
                    key = d.get("part_number") or d.get("id") or pathlib.Path(src).stem
                    files[f"data/{parts[1]}/{key}.json"] = text.encode()
                elif len(parts) == 2 and pathlib.Path(src).stem in TOP_LEVEL:
                    files[f"data/{pathlib.Path(src).stem}.json"] = text.encode()
            elif src.startswith("matlab_sils/data/"):
                files[src[len("matlab_sils/"):]] = value.encode()
                if src.endswith("igrf13coeffs.txt"):
                    files["data/igrf13.json"] = gen_fsw_params.igrf_json(gen_fsw_params.igrf_table(value.encode())).encode()
    for cid, (kind, text) in cases.items():
        if kind in CASE_KINDS:
            d = tomllib.loads(text)
            key = d.get("id") or cid.split("_", 1)[1]
            files[f"data/{CASE_KINDS[kind]}/{key}.json"] = (json.dumps(d, indent=1, sort_keys=True) + "\n").encode()
    return files


def read_cases(drive):
    """{case id: (kind, source text)} and the case rows of every case (kind "case") the engine flies
    (meta `flown`; a case file without it is flown)."""
    cases, rows = {}, []
    for f in sorted((pathlib.Path(drive) / "cases").glob("*.tncase")):
        with sqlite3.connect(f"file:{f}?mode=ro", uri=True) as c:
            cid = c.execute("SELECT id FROM case_info").fetchone()[0]
            src = c.execute("SELECT format, text FROM case_source").fetchone()
            flown = dict(c.execute('SELECT "key", "value" FROM meta')).get("flown", "yes")
            if flown != "yes":                # kept on the drive, not an engine input (the plan's own copies)
                continue
            cases[cid] = src
            if src[0] == "case":
                for r in c.execute('SELECT * FROM case_line ORDER BY "ord"'):
                    rows.append((cid, *r))
    return cases, rows


def build(drive, out, *, export=None, at="2026-10-06T00:00:00Z"):
    drive, out = pathlib.Path(drive), pathlib.Path(out)
    used = releases_used(drive)
    dgroups, dnodes, edges, catalogue, notes, bodies = [], [], [], [], [], {}
    readers = {}
    loaded = {g: _bodies(p) for g, (p, _) in used.items() if p}
    for g, (_info, _n, _gn, gedges, _c) in loaded.items():
        for e in gedges:
            readers.setdefault(e["from_node"], set()).add(g)
    for gid, (path, why) in used.items():
        notes += why
        if not path:
            continue
        info, nodes, gnodes, gedges, contracts = loaded[gid]
        dgroups.append((gid, info["version"], info["fingerprint"], info["sealed_at"]))
        cver = {(c["node"], c["output"]): c for c in contracts}
        for nid, x in sorted(nodes.items()):
            body = json.loads(x["body"])
            bodies[nid] = body
            r = gnodes[nid]
            dnodes.append((nid, gid, r["stage"], r["layer"], r["kind"], r["label"],
                           json.dumps({"body": body, "sealed_as": x["sealed_as"], "why": x.get("why", [])}, sort_keys=True), f"{gid} {info['version']}"))
            for o in body.get("output", []):
                c = cver.get((nid, o[0])) or cver.get((nid, "*"))
                catalogue.append((nid, o[0], o[1], c["version"] if c else 0, ",".join(sorted(readers.get(nid, set()) - {gid}))))
        edges += [(e["from_node"], e["to_node"], e["kind"], e["label"]) for e in gedges]
    cases, case_rows = read_cases(drive)
    files = engine_inputs(bodies, cases)
    file_rows = [(p, design_inputs.fnv_hex(b), b) for p, b in sorted(files.items())]
    h = hashlib.sha256()
    for r in case_rows:
        h.update(f"case {r[0]} {r[1]} {r[-1]}\n".encode())
    for p, fp, _ in file_rows:
        h.update(f"file {p} {fp}\n".encode())
    fp_all = h.hexdigest()
    if out.exists():
        out.unlink()

    def fill(c):
        c.executemany("INSERT INTO design_group VALUES (?, ?, ?, ?)", dgroups)
        c.executemany("INSERT INTO design_node VALUES (?, ?, ?, ?, ?, ?, ?, ?)", dnodes)
        c.executemany("INSERT INTO edge VALUES (?, ?, ?, ?)", edges)
        c.executemany("INSERT INTO catalogue_output VALUES (?, ?, ?, ?, ?)", catalogue)
        c.executemany("INSERT INTO design_case VALUES (" + ", ".join("?" * 13) + ")", case_rows)
        c.executemany("INSERT INTO engine_input VALUES (?, ?, ?)", file_rows)
        c.execute("INSERT INTO design_release VALUES (?, ?, ?, ?, ?, ?)",
                  (f"today {at[:10]}", "today", at, "tools/design_build.py", design_inputs.needs_application(), design_inputs.TOOLBOX))
        for k, v in (("inputs_fingerprint", fp_all), ("toolbox", design_inputs.TOOLBOX), ("needs_application", design_inputs.needs_application()),
                     ("design_kind", "today"), ("releases_used", json.dumps({g: v for g, v, _f, _a in dgroups}, sort_keys=True)),
                     ("refused", json.dumps(notes))):
            c.execute('INSERT OR REPLACE INTO meta VALUES (?, ?)', (k, v))
    tndb.create(out, "design", "today", written_by="tools/design_build.py", fill=fill, sync=False)
    if export:
        write_export(pathlib.Path(export), files, case_rows)
    return {"groups": len(dgroups), "nodes": len(dnodes), "edges": len(edges), "cases": len({r[0] for r in case_rows}),
            "files": len(file_rows), "refused": notes}


def write_export(root, files, case_rows):
    """The engine's data folder, generated from the design: data/... and cases/<id>.csv."""
    for p, b in files.items():
        f = root / p
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_bytes(b)
    by = {}
    for r in case_rows:
        by.setdefault(r[0], []).append(r[-1])
    (root / "cases").mkdir(parents=True, exist_ok=True)
    for cid, lines in by.items():
        (root / "cases" / f"{cid}.csv").write_text(design_inputs.HEADER + "\n" + "".join(x + "\n" for x in lines), encoding="utf-8")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("drive")
    ap.add_argument("--out", required=True)
    ap.add_argument("--export")
    ap.add_argument("--at", default="2026-10-06T00:00:00Z")
    a = ap.parse_args(argv)
    s = build(a.drive, a.out, export=a.export, at=a.at)
    for n in s.pop("refused"):
        print("design_build: " + n, file=sys.stderr)
    print("design_build: " + ", ".join(f"{k} {v}" for k, v in s.items()))
    return 0


if __name__ == "__main__":
    sys.exit(main())
