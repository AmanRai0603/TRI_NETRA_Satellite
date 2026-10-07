"""The engine's inputs, as the design database holds them (docs/RELEASE_PLAN.md P11): every case
row by row (`design_case`) and every other file the engine reads under data/ (`engine_input`).
tools/seed_design.py and tools/group.py merge put them into design.tndb; the engine and the app
read them from there (engine/crates/adcs-sim/src/source.rs) with the same bytes, so a run flown
from the database has the same result id as one flown from the files.

    rows, files, fp = design_inputs.gather()       # what to write
    design_inputs.fill(conn)                        # write them into an open design database

A case row keeps its line exactly as written (the quoting included), its fields, and the node
that declares its value (spec/plan/case_inputs.toml). The test vectors under data/ are not
inputs (the generators' own checks) and stay out.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import csv
import hashlib
import io
import tomllib

from common import ROOT

DATA = ROOT / "matlab_sils"
HEADER = "section,key,label,unit,value,lo,hi,level,note"
NOT_INPUTS = ("_vectors.json",)


def fnv_hex(b):
    """FNV-1a 64 as hex: the fingerprint a run gives every input (engine store.rs)."""
    h = 0xcbf29ce484222325
    for x in b:
        h = ((h ^ x) * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def _nodes_of_keys():
    import from_design
    ci = tomllib.loads(from_design.text("spec/plan/case_inputs.toml"))
    return {x["key"]: x.get("tree_id") for x in ci.get("input", []) if x.get("key")}


def case_rows(path, node_of=None):
    """[(case_id, ord, section, key, label, unit, value, lo, hi, level, note, node, line)] of one case
    file; refused when its lines do not give back the file's bytes exactly."""
    node_of = _nodes_of_keys() if node_of is None else node_of
    text = path.read_bytes().decode("utf-8")
    lines = text.split("\n")
    if lines[0].rstrip("\r") != HEADER:
        raise SystemExit(f"design_inputs: {path} does not start with the adcs-case/1 header")
    body = [ln for ln in lines[1:] if ln.strip()]
    if HEADER + "\n" + "".join(ln + "\n" for ln in body) != text:
        raise SystemExit(f"design_inputs: {path} has blank lines, carriage returns or no final newline; "
                         "its lines would not give its bytes back")
    cid = path.stem
    out = []
    for i, ln in enumerate(body, 1):
        f = (next(csv.reader(io.StringIO(ln))) + [""] * 9)[:9]
        out.append((cid, i, *f, node_of.get(f[1].strip()), ln))
    return out


def gather():
    """(case rows, [(path, fingerprint, bytes)], the inputs' own fingerprint)."""
    node_of = _nodes_of_keys()
    rows = [r for f in sorted((DATA / "cases").glob("*.csv")) for r in case_rows(f, node_of)]
    files = []
    for f in sorted((DATA / "data").rglob("*")):
        if f.is_file() and not f.name.endswith(NOT_INPUTS) and f.name != ".gitkeep":
            b = f.read_bytes()
            files.append((f.relative_to(DATA).as_posix(), fnv_hex(b), b))
    h = hashlib.sha256()
    for r in rows:
        h.update(f"case {r[0]} {r[1]} {r[-1]}\n".encode())
    for p, fp, _ in files:
        h.update(f"file {p} {fp}\n".encode())
    return rows, files, h.hexdigest()


# The toolbox this design is built for, and the oldest application that can run it: the engine,
# the app and the Python package refuse a design that names another toolbox or a newer application
# (engine/crates/adcs-sim/src/source.rs, `cannot_run`). Raise TOOLBOX when the functions a design may
# call change; NEEDS_APPLICATION follows the program version that wrote the design.
# /2 (S7.2): data tables, inf and nan, choices, named capacities, inputs by reference, the sort, random streams.
TOOLBOX = "trinetra-toolbox/2"


def needs_application():
    return (ROOT / "VERSION").read_text().strip()


def fill(conn, got=None):
    """Write the engine's inputs into an open design database (its meta gains `inputs_fingerprint`,
    `toolbox` and `needs_application`)."""
    rows, files, fp = got or gather()
    conn.executemany("INSERT INTO design_case VALUES (" + ", ".join("?" * 13) + ")", rows)
    conn.executemany("INSERT INTO engine_input VALUES (?, ?, ?)", files)
    conn.execute('INSERT OR REPLACE INTO meta VALUES (?, ?)', ("inputs_fingerprint", fp))
    conn.execute('INSERT OR REPLACE INTO meta VALUES (?, ?)', ("toolbox", TOOLBOX))
    conn.execute('INSERT OR REPLACE INTO meta VALUES (?, ?)', ("needs_application", needs_application()))
    return {"cases": len({r[0] for r in rows}), "case_rows": len(rows), "files": len(files), "fingerprint": fp}


def inputs_fingerprint(conn):
    """The inputs' own fingerprint of an open design database, from the rows it holds (the same hash gather()
    and tools/design_build.py give: every case line, every input file's fingerprint)."""
    h = hashlib.sha256()
    for cid, ord_, line in conn.execute('SELECT "case_id", "ord", "line" FROM design_case ORDER BY "case_id", "ord"'):
        h.update(f"case {cid} {ord_} {line}\n".encode())
    for p, fp in conn.execute('SELECT "path", "fingerprint" FROM engine_input ORDER BY "path"'):
        h.update(f"file {p} {fp}\n".encode())
    return h.hexdigest()


def set_input(conn, path, body):
    """Change one engine input a design database holds (its bytes, its fingerprint) and the database's inputs
    fingerprint with it, as merging a changed release would: what the engine then reads from the design."""
    if conn.execute('UPDATE engine_input SET "fingerprint" = ?, "body" = ? WHERE "path" = ?', (fnv_hex(body), body, path)).rowcount != 1:
        raise SystemExit(f"design_inputs: the design holds no input {path}")
    conn.execute('INSERT OR REPLACE INTO meta VALUES (?, ?)', ("inputs_fingerprint", inputs_fingerprint(conn)))


def differences(db):
    """What differs between a design database's engine inputs and the data folder's files: [text]
    (empty: the engine reads the same bytes from either)."""
    import sqlite3
    rows, files, fp = gather()
    with sqlite3.connect(f"file:{db}?mode=ro", uri=True) as c:
        meta = dict(c.execute('SELECT "key", "value" FROM meta'))
        held = dict(c.execute('SELECT "path", "fingerprint" FROM engine_input'))
        lines = {(r[0], r[1]): r[2] for r in c.execute('SELECT "case_id", "ord", "line" FROM design_case')}
    out = []
    now = {p: f for p, f, _ in files}
    out += [f"{p}: in the data folder, not in the database" for p in sorted(set(now) - set(held))]
    out += [f"{p}: in the database, not in the data folder" for p in sorted(set(held) - set(now))]
    out += [f"{p}: changed since the database was built" for p in sorted(set(now) & set(held)) if now[p] != held[p]]
    mine = {(r[0], r[1]): r[-1] for r in rows}
    for case in sorted({k[0] for k in set(mine) | set(lines)}):
        if {k: v for k, v in mine.items() if k[0] == case} != {k: v for k, v in lines.items() if k[0] == case}:
            out.append(f"cases/{case}.csv: differs from the database's rows")
    if not out and meta.get("inputs_fingerprint") != fp:
        out.append("the database's inputs fingerprint is not its inputs'")
    return out
