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
    ci = tomllib.loads((ROOT / "spec" / "plan" / "case_inputs.toml").read_text())
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


def fill(conn, got=None):
    """Write the engine's inputs into an open design database (its meta gains `inputs_fingerprint`)."""
    rows, files, fp = got or gather()
    conn.executemany("INSERT INTO design_case VALUES (" + ", ".join("?" * 13) + ")", rows)
    conn.executemany("INSERT INTO engine_input VALUES (?, ?, ?)", files)
    conn.execute('INSERT OR REPLACE INTO meta VALUES (?, ?)', ("inputs_fingerprint", fp))
    return {"cases": len({r[0] for r in rows}), "case_rows": len(rows), "files": len(files), "fingerprint": fp}
