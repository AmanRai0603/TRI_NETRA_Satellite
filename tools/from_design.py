#!/usr/bin/env python3
"""The design in the repository (docs/PLAN_2_0.md S4; docs/CODE_ARCHITECTURE.md §1): the repository holds no design
data of its own. It keeps one copy of a design for its tests and its build, the regression copy
(tests/regression/design.tndb), and every file the code still reads that is design is GENERATED from it and
never edited:

    python3 tools/from_design.py            write every generated file from the regression copy
    python3 tools/from_design.py --check    exit 1 when a generated file is not what the design gives
    python3 tools/from_design.py --list     every generated file and where in the design it comes from

The generated files (until the flight build and the engine build generate code from the nodes, S6 and S7):
  matlab_sils/data/...        the engine's input files (engine_input), what the engine and the twin read
  matlab_sils/cases/<id>.csv  every case the engine flies (design_case), line for line
  fsw/params/params.toml      the flight software's parameter table (its library block, kept whole)
  fsw/pseudocode/02           the published models the flight software uses: the IGRF table (env's data), from S7.3
                              time, frames and the field (env's methods)
  fsw/pseudocode/03-09        the flight software's algorithms and their notes (the fsw_* method blocks)
Files under matlab_sils/data that are the generators' own test vectors (*_vectors.json) are code, not design.

Tools that read the rest of 1.0.0's plan (spec/plan, spec/physics: archived in archive/design-1.0/) read it from
the design by its 1.0.0 path:

    import from_design
    from_design.text("spec/plan/kpis.toml")       # the text the design holds whole, byte for byte
    from_design.paths("spec/physics/")             # every such path under a folder

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import functools
import json
import sqlite3
import sys

from common import ROOT, write_bytes

REGRESSION = ROOT / "tests" / "regression" / "design.tndb"
NOT_DESIGN = ("_vectors.json", ".gitkeep")       # under matlab_sils/data: the generators' own checks


def design():
    if not REGRESSION.is_file():
        raise SystemExit(f"from_design: no regression copy at {REGRESSION.relative_to(ROOT)} (tests/regression/README.md says how it is made)")
    return REGRESSION


@functools.lru_cache(maxsize=None)
def _bodies(db=None):
    with sqlite3.connect(f"file:{db or design()}?mode=ro", uri=True) as c:
        return {nid: json.loads(x)["body"] for nid, x in c.execute("SELECT id, content FROM design_node")}


@functools.lru_cache(maxsize=None)
def _held(db=None):
    """{1.0.0 path: text} of every file the design keeps whole (a lookup block's table, a method's pseudocode
    and its notes, a data block's module), by the path its origin names."""
    out = {}
    for _nid, b in _bodies(db).items():
        for sec, field, value, origin in b.get("content", []):
            if not origin:
                continue
            src = origin.split(" ")[0]
            if (sec == "table" and "/" in src) or sec == "data" or (sec, field) in (("code", "pseudocode"), ("explain", "theory")):
                if src.startswith(("spec/", "catalogue/", "fsw/", "matlab_sils/", "env/", "dyn/", "act/", "sens/", "oils/", "pnt/", "design/", "kpi/")) and value:
                    out.setdefault(src, value)
    return out


def text(path, db=None):
    """The text of a 1.0.0 file as the design holds it, byte for byte; refused by name when it holds none."""
    held = _held(db)
    if path not in held:
        raise SystemExit(f"from_design: the design holds no {path} (tools/convert_2_0.py LIBRARY says what it keeps)")
    return held[path]


def paths(prefix, db=None):
    return sorted(p for p in _held(db) if p.startswith(prefix))


def folder(prefix, db=None):
    """The design's texts under a 1.0.0 folder (spec/physics/), written as files for a tool that takes paths
    (the pseudocode's translators), under build/from_design/ (not kept in git); its path."""
    root = ROOT / "build" / "from_design"
    ps = paths(prefix, db)
    if not ps:
        raise SystemExit(f"from_design: the design holds nothing under {prefix}")
    for p in ps:
        f = root / p
        b = text(p, db).encode()
        if not f.is_file() or f.read_bytes() != b:
            write_bytes(f, b)
    out = root / prefix.rstrip("/")
    for f in out.glob("*"):
        if f.is_file() and f.relative_to(root).as_posix() not in ps:
            f.unlink()
    return out


def version(db=None):
    """Which design it is: its released version (meta design_version) when it has one, else the
    version its design_release row gives (today's design: `today <date>`)."""
    with sqlite3.connect(f"file:{db or design()}?mode=ro", uri=True) as c:
        meta = dict(c.execute('SELECT "key", "value" FROM meta'))
        row = c.execute('SELECT "version" FROM design_release ORDER BY "built_at" DESC LIMIT 1').fetchone()
    return meta.get("design_version") or (row[0] if row else meta.get("id", "?"))


def fingerprint(db=None):
    """The design's content hash: sha256 over its inputs' fingerprint (meta inputs_fingerprint: every case
    and engine input) and every node's content, by id. Any change to a node, a case or an input changes it;
    the file's layout on disk does not."""
    import hashlib
    with sqlite3.connect(f"file:{db or design()}?mode=ro", uri=True) as c:
        meta = dict(c.execute('SELECT "key", "value" FROM meta'))
        nodes = c.execute('SELECT "id", "content" FROM design_node ORDER BY "id"').fetchall()
    h = hashlib.sha256(f"inputs {meta.get('inputs_fingerprint', '')}\n".encode())
    for nid, content in nodes:
        h.update(f"node {nid} {hashlib.sha256((content or '').encode()).hexdigest()}\n".encode())
    return h.hexdigest()


def outputs(db=None):
    """{repository path: (bytes, where it comes from)} of every file generated from the design."""
    db = db or design()
    out = {}
    with sqlite3.connect(f"file:{db}?mode=ro", uri=True) as c:
        for p, b in c.execute('SELECT "path", "body" FROM engine_input'):
            out[f"matlab_sils/{p}"] = (bytes(b), "engine_input")
        rows = c.execute('SELECT * FROM design_case ORDER BY case_id, "ord"').fetchall()
    by = {}
    for r in rows:
        by.setdefault(r[0], []).append(r[-1])
    import design_inputs
    for cid, lines in by.items():
        out[f"matlab_sils/cases/{cid}.csv"] = ((design_inputs.HEADER + "\n" + "".join(x + "\n" for x in lines)).encode(), "design_case")
    held = _held(db)
    out["fsw/params/params.toml"] = (held["fsw/params/params.toml"].encode(), "the parameter table's lookup block")
    for p in sorted(held):
        if p.startswith("fsw/pseudocode/") and p.split("/")[-1][:2] in ("03", "04", "05", "06", "07", "08", "09"):
            out[p] = (held[p].encode(), "a flight algorithm block (fsw_*)")
        elif p.startswith("fsw/pseudocode/02"):
            out[p] = (held[p].encode(), "a published model's method or data the flight software uses (env)")
    return out


def stale(db=None):
    """[why] for every generated file that is not what the design gives, and every file under the generated
    folders that the design does not give."""
    outs = outputs(db)
    bad = [f"{p}: not what the design gives ({how})" for p, (b, how) in sorted(outs.items())
           if not (ROOT / p).is_file() or (ROOT / p).read_bytes() != b]
    for folder in ("matlab_sils/data", "matlab_sils/cases"):
        for f in sorted((ROOT / folder).rglob("*")):
            rel = f.relative_to(ROOT).as_posix()
            if f.is_file() and rel not in outs and not f.name.endswith(NOT_DESIGN):
                bad.append(f"{rel}: under a generated folder, but the design does not give it")
    return bad


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    if argv == ["--check"]:
        bad = stale()
        for x in bad:
            print("from_design: " + x)
        print(f"from_design --check: {'every generated file is what the design gives' if not bad else f'{len(bad)} stale'} "
              f"(design: {design().relative_to(ROOT)})")
        return 1 if bad else 0
    if argv == ["--list"]:
        for p, (b, how) in sorted(outputs().items()):
            print(f"{p}\t{len(b)} bytes\t{how}")
        return 0
    if argv:
        sys.exit("usage: python3 tools/from_design.py [--check | --list]")
    outs = outputs()
    for p, (b, _how) in outs.items():
        if not (ROOT / p).is_file() or (ROOT / p).read_bytes() != b:
            write_bytes(ROOT / p, b)
    print(f"from_design: {len(outs)} files generated from {design().relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
