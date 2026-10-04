#!/usr/bin/env python3
"""The design files (`design/schema.toml`): make one, open one (checking, or upgrading, its format),
check its tables against the schema, and dump or load its whole content as canonical JSON.

    python3 tools/tndb.py check FILE...     the format, its version and every table against the schema
    python3 tools/tndb.py dump FILE         the content as canonical JSON (what a round trip compares)
    python3 tools/tndb.py ddl KIND          the SQL that makes a file of that kind
    python3 tools/tndb.py gen [--check]     write (or check) the files made from the schema:
                                            design/ddl.sql (the Rust reader checks itself against it)
                                            and design/js/tndb_schema.js (the browser's reader)

A file of this program's format version opens as it is. An older one is upgraded in place by the
steps in UPGRADES, after a copy of the original is kept beside it (`<file>.v<N>.bak`). A newer one
is refused by name: this program cannot know what it holds.

Writes are whole-file: a new file is built under a temporary name and renamed into place, so a
crash never leaves half a file where the old one was.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import hashlib
import json
import os
import pathlib
import shutil
import sqlite3
import sys
import tomllib

from common import ROOT, write_text

SCHEMA = ROOT / "design" / "schema.toml"
PROGRAM = "trinetra-tndb"


class FormatError(Exception):
    """A file this program cannot take: another format, a newer version, a table that is wrong."""


def schema(path=SCHEMA):
    s = tomllib.loads(pathlib.Path(path).read_text())
    if s.get("schema") != "trinetra-design-schema/1":
        raise FormatError(f"{path} is not trinetra-design-schema/1")
    for kind, f in s["formats"].items():
        missing = [t for t in f["tables"] if t not in s["tables"]]
        if missing:
            raise FormatError(f"format {kind} names tables the schema does not define: {', '.join(missing)}")
    return s


def columns(table, s):
    """[(name, sqlite type, is key)] of one table."""
    out = []
    for name, decl in s["tables"][table]["columns"]:
        parts = decl.split()
        out.append((name, parts[0].upper(), "key" in parts[1:]))
    return out


def ddl(kind, s=None):
    s = s or schema()
    stmts = []
    for t in s["formats"][kind]["tables"]:
        cols = columns(t, s)
        keys = [c for c, _, k in cols if k]
        body = ", ".join(f'"{c}" {ty}' for c, ty, _ in cols)
        if keys:
            body += ", PRIMARY KEY (" + ", ".join(f'"{k}"' for k in keys) + ")"
        stmts.append(f'CREATE TABLE "{t}" ({body})')
    return stmts


# Upgrade steps: {kind: {from_version: function(connection)}}; each takes a file one version up.
UPGRADES = {kind: {} for kind in ("node", "group", "release", "design")}


def _design_1_to_2(conn):
    """design 1 -> 2: the engine's inputs (design_case, engine_input), empty until the design is
    seeded or merged again, which fills them (tools/design_inputs.py)."""
    s = schema()
    for st in ddl("design", s):
        if '"design_case"' in st or '"engine_input"' in st:
            conn.execute(st)


UPGRADES["design"][1] = _design_1_to_2


def _meta(conn):
    try:
        return dict(conn.execute('SELECT "key", "value" FROM meta').fetchall())
    except sqlite3.Error as e:
        raise FormatError(f"no meta table: not a design file ({e})") from None


def create(path, kind, file_id, s=None, written_by=PROGRAM, fill=None, sync=True):
    """A new file of this kind, written whole: its tables, its meta, and what fill(connection) puts
    in, built in one transaction under a temporary name, synced once and renamed into place.
    Refuses to overwrite. sync=False skips the sync, for a file in a scratch folder (a check, a test)."""
    s = s or schema()
    path = pathlib.Path(path)
    if kind not in s["formats"]:
        raise FormatError(f"no format {kind!r}; the formats are {', '.join(s['formats'])}")
    if path.exists():
        raise FormatError(f"{path} exists; a design file is never overwritten by create")
    tmp = path.with_name(path.name + ".tmp")
    tmp.unlink(missing_ok=True)
    # one transaction for the whole file, no journal and no sync while it is built: the temporary
    # file is nobody's until the rename, and it is synced once before it
    conn = sqlite3.connect(tmp, isolation_level=None)
    try:
        conn.execute("PRAGMA journal_mode = OFF")
        conn.execute("PRAGMA synchronous = OFF")
        conn.execute("BEGIN")
        for st in ddl(kind, s):
            conn.execute(st)
        meta = {"format": kind, "format_version": str(s["formats"][kind]["version"]), "id": file_id, "written_by": written_by}
        conn.executemany('INSERT INTO meta VALUES (?, ?)', sorted(meta.items()))
        if fill:
            fill(conn)
        conn.execute("COMMIT")
    except BaseException:
        conn.close()
        tmp.unlink(missing_ok=True)
        raise
    conn.close()
    if sync:
        fd = os.open(tmp, os.O_RDONLY)
        try:
            os.fsync(fd)
        finally:
            os.close(fd)
    os.replace(tmp, path)
    return path


def open_file(path, kind=None, s=None):
    """A connection to a design file of the current version: upgraded when older (a copy of the
    original kept beside it), refused when newer, refused when it is another kind."""
    s = s or schema()
    path = pathlib.Path(path)
    if not path.is_file():
        raise FormatError(f"{path}: no such file")
    conn = sqlite3.connect(path)
    try:
        m = _meta(conn)
        got = m.get("format")
        if got not in s["formats"]:
            raise FormatError(f"{path}: format {got!r} is not a design-file format")
        if kind and got != kind:
            raise FormatError(f"{path}: a {got} file, not a {kind} file")
        have, want = int(m.get("format_version", "0")), s["formats"][got]["version"]
        if have > want:
            raise FormatError(f"{path}: {got} format version {have} is newer than this program's {want}; "
                              "open it with a newer TRI-NETRA")
        if have < want:
            conn.close()
            shutil.copy2(path, path.with_name(f"{path.name}.v{have}.bak"))
            conn = sqlite3.connect(path)
            for v in range(have, want):
                step = UPGRADES[got].get(v)
                if step is None:
                    raise FormatError(f"{path}: no upgrade from {got} version {v} to {v + 1}")
                step(conn)
                conn.execute('UPDATE meta SET "value" = ? WHERE "key" = ?', (str(v + 1), "format_version"))
                conn.commit()
        return conn
    except BaseException:
        conn.close()
        raise


def check(path, s=None):
    """The problems with one file: its format and every table and column against the schema."""
    s = s or schema()
    try:
        conn = open_file(path, s=s)
    except FormatError as e:
        return [str(e)]
    try:
        kind = _meta(conn)["format"]
        errs = []
        have = {r[0] for r in conn.execute("SELECT name FROM sqlite_master WHERE type = 'table'")}
        want = set(s["formats"][kind]["tables"])
        errs += [f"{path}: table {t} missing" for t in sorted(want - have)]
        errs += [f"{path}: table {t} is not in the {kind} format" for t in sorted(have - want)]
        for t in sorted(want & have):
            got = [(r[1], r[2].upper()) for r in conn.execute(f'PRAGMA table_info("{t}")')]
            exp = [(c, ty) for c, ty, _ in columns(t, s)]
            if got != exp:
                errs.append(f"{path}: table {t} has columns {got}, the schema says {exp}")
        f = s["formats"][kind]
        if "max_bytes" in f and pathlib.Path(path).stat().st_size > f["max_bytes"]:
            errs.append(f"{path}: {pathlib.Path(path).stat().st_size} bytes, over the {f['max_bytes']} a {kind} file may hold")
        if "max_attachment_bytes" in f and "attachment" in have:
            big = conn.execute('SELECT name, size FROM attachment WHERE size > ?', (f["max_attachment_bytes"],)).fetchall()
            errs += [f"{path}: attachment {n} is {z} bytes, over {f['max_attachment_bytes']}" for n, z in big]
        return errs
    finally:
        conn.close()


def _jsonable(v):
    return {"blob": v.hex()} if isinstance(v, bytes) else v


def dump(path, s=None):
    """The whole file as canonical JSON-able data: meta, then every table's rows sorted."""
    s = s or schema()
    conn = open_file(path, s=s)
    try:
        kind = _meta(conn)["format"]
        out = {"meta": _meta(conn), "tables": {}}
        for t in s["formats"][kind]["tables"]:
            if t == "meta":
                continue
            names = [c for c, _, _ in columns(t, s)]
            rows = [[_jsonable(v) for v in r] for r in conn.execute(f'SELECT {", ".join(chr(34) + n + chr(34) for n in names)} FROM "{t}"')]
            out["tables"][t] = {"columns": names, "rows": sorted(rows, key=lambda r: json.dumps(r, sort_keys=True))}
        return out
    finally:
        conn.close()


def load(data, path, s=None):
    """Write a dump back to a new file, whole. dump(load(dump(f))) == dump(f)."""
    s = s or schema()
    m = data["meta"]
    path = pathlib.Path(path)
    if path.exists():
        raise FormatError(f"{path} exists; load writes a new file")

    def fill(conn):
        conn.execute('DELETE FROM meta')
        conn.executemany('INSERT INTO meta VALUES (?, ?)', sorted(m.items()))
        for t, d in data["tables"].items():
            ph = ", ".join("?" for _ in d["columns"])
            cols = ", ".join(f'"{c}"' for c in d["columns"])
            conn.executemany(f'INSERT INTO "{t}" ({cols}) VALUES ({ph})',
                             [[bytes.fromhex(v["blob"]) if isinstance(v, dict) and "blob" in v else v for v in r] for r in d["rows"]])

    return create(path, m["format"], m["id"], s=s, written_by=m.get("written_by", PROGRAM), fill=fill)


def fingerprint(obj):
    """sha256 of the canonical JSON of obj: what a release seals each node with."""
    return hashlib.sha256(json.dumps(obj, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


GEN_SQL = ROOT / "design" / "ddl.sql"
GEN_JS = ROOT / "design" / "js" / "tndb_schema.js"
_HEAD = "Generated by tools/tndb.py gen from design/schema.toml; do not edit."


def gen_sql(s=None):
    s = s or schema()
    out = [f"-- {_HEAD}"]
    for kind in s["formats"]:
        out.append(f"-- {kind} version {s['formats'][kind]['version']}")
        out += [st + ";" for st in ddl(kind, s)]
    return "\n".join(out) + "\n"


def gen_js(s=None):
    s = s or schema()
    body = {"schema": s["schema"],
            "tables": {t: [[c, ty.lower(), k] for c, ty, k in columns(t, s)] for t in s["tables"]},
            "formats": {k: {kk: v for kk, v in f.items()} for k, f in s["formats"].items()},
            "ddl": {k: ddl(k, s) for k in s["formats"]}}
    return (f"// {_HEAD}\n"
            "// The browser's reader opens a design file with SQLite compiled to WebAssembly (P3) and checks\n"
            "// it against this: the same tables, columns and format versions tools/tndb.py checks.\n"
            f"export const TNDB_SCHEMA = {json.dumps(body, indent=1, sort_keys=True)};\n")


def gen(check_only=False):
    """[(path, problem)] of the generated files; writes them unless check_only."""
    errs = []
    for path, text in ((GEN_SQL, gen_sql()), (GEN_JS, gen_js())):
        have = path.read_text() if path.is_file() else None
        if have == text:
            continue
        if check_only:
            errs.append(f"{path.relative_to(ROOT)} is not what design/schema.toml makes (run tools/tndb.py gen)")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            write_text(path, text)
    return errs


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("check", help="check files against the schema")
    c.add_argument("files", nargs="+")
    d = sub.add_parser("dump", help="the content as canonical JSON")
    d.add_argument("file")
    q = sub.add_parser("ddl", help="the SQL that makes a file of a kind")
    q.add_argument("kind")
    g = sub.add_parser("gen", help="write (or check) design/ddl.sql and design/js/tndb_schema.js")
    g.add_argument("--check", action="store_true", help="exit 1 if either is not what the schema makes")
    a = ap.parse_args(argv)
    if a.cmd == "gen":
        errs = gen(a.check)
        for e in errs:
            print("tndb: " + e, file=sys.stderr)
        print(f"tndb: generated files {'checked' if a.check else 'written'}, {len(errs)} problem(s)")
        return 1 if errs else 0
    if a.cmd == "ddl":
        print(";\n".join(ddl(a.kind)) + ";")
        return 0
    if a.cmd == "dump":
        print(json.dumps(dump(a.file), indent=1, sort_keys=True))
        return 0
    errs = [e for f in a.files for e in check(f)]
    for e in errs:
        print("tndb: " + e, file=sys.stderr)
    print(f"tndb: {len(a.files)} file(s), {len(errs)} problem(s)")
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
