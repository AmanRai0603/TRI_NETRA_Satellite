#!/usr/bin/env python3
"""The readers of published data (docs/PLAN_2_0.md, the boundary: code keeps the readers of files' formats, which load
published data into the database, never models of their own). Each reader reads one file format and gives the
tables it holds, as the pseudocode's `data` items (docs/PSEUDOCODE_V2.md, *Data tables*) that a node of the design
holds; the developer's revisions (design/revisions_2_0.toml, `[[revision.data]]`) name the reader, the file and the
publication, and tools/convert_2_0.py runs it. Nothing here is a value: every number comes from the file.

    python3 tools/readers.py READER FILE        print the tables READER reads from FILE (the module text)

Readers (each returns {table name: (type, rows)}: a 1-D table a list of numbers, a 2-D one a list of rows):
  igrf13        IAGA's igrf13coeffs.txt: the IGRF-13 epochs and Gauss coefficients [nT] (gen_fsw_params.igrf_table,
                as the flight software's and the engine's tables have always been made)
  matlab_matrix the numeric matrices a MATLAB file assigns (`NAME = [...]`), e.g. the Octave POP's leapTable.m and
                its IERS tidal-EOP tables
  xys06         the IAU 2006/2000A X, Y and s series, as refgen exported xys06_tables.mat (xys06.bin: little-endian
                doubles, a header of counts, then the polynomials and the terms)

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import hashlib
import pathlib
import re
import struct
import sys

from common import ROOT


def shortest(x):
    """The shortest decimal that reads back as the same double (Python's repr), as the pseudocode writes a literal."""
    x = float(x)
    if x != x or x in (float("inf"), float("-inf")):
        raise SystemExit(f"readers: {x} is not a finite number; a data table holds finite numbers")
    return repr(x)


# ------------------------------------------------------------------ the readers

def igrf13(path):
    """IAGA igrf13coeffs.txt -> IGRF_YEAR (26 epochs) and IGRF_GH (26 x 195, nT): every DGRF/IGRF epoch, then 2025.0 =
    2020.0 + 5 x the secular variation (the model's own definition of its last epoch)."""
    import gen_fsw_params
    d = gen_fsw_params.igrf_table(pathlib.Path(path).read_bytes())
    bad = gen_fsw_params.check_igrf(d)
    if bad:
        raise SystemExit("readers: igrf13: " + "; ".join(bad))
    return {"IGRF_YEAR": ("real[1][{n}]", [e["year"] for e in d["epochs"]]),
            "IGRF_GH": ("real[1][195][{n}]", [e["gh"] for e in d["epochs"]])}


_MAT = re.compile(r"\b([A-Za-z_]\w*)\s*=\s*\[([^\]]*)\]", re.S)


def matlab_matrix(path, names=None):
    """Every numeric matrix a MATLAB file assigns as a literal (`NAME = [a b c; d e f];`), by name: rows split at `;`
    or a line end, values at blanks or commas, comments (`%` to the line end) dropped. Python's float() reads each
    decimal to the nearest double, as MATLAB and Rust do."""
    text = pathlib.Path(path).read_text(encoding="utf-8")
    text = "\n".join(line.split("%", 1)[0] for line in text.splitlines())
    out = {}
    for name, body in _MAT.findall(text):
        rows = []
        for r in re.split(r"[;\n]", body):
            vals = [v for v in re.split(r"[\s,]+", r.strip()) if v]
            if vals:
                try:
                    rows.append([float(v) for v in vals])
                except ValueError:
                    break
        else:
            if rows and (names is None or name in names):
                if len({len(r) for r in rows}) != 1:
                    raise SystemExit(f"readers: {path}: {name} has rows of different lengths")
                out[name] = rows
    if names is not None and set(names) - set(out):
        raise SystemExit(f"readers: {path}: no numeric matrix {sorted(set(names) - set(out))}")
    return out


def xys06(path):
    """xys06.bin (refgen/time_frames_xys06_export.m of xys06_tables.mat): doubles, little-endian. Header (8): the number
    of X/Y terms, then of s terms of each power 0..4, then two unused. Then the X and Y polynomials (2 x 6, arcsec) and
    the s polynomial (6, arcsec); the X/Y terms (each: 0 for X or 1 for Y, the power of t, 14 multipliers of the
    fundamental arguments, the sine and cosine amplitudes in micro-arcsec); the s terms of each power in turn (each:
    8 multipliers of fa[1 2 3 4 5 7 8 14], sine and cosine amplitudes)."""
    b = pathlib.Path(path).read_bytes()
    if len(b) % 8:
        raise SystemExit(f"readers: {path}: {len(b)} bytes is not a whole number of doubles")
    v = list(struct.unpack(f"<{len(b) // 8}d", b))
    n_xy, ns = int(v[0]), [int(x) for x in v[1:6]]
    k = 8
    xyp = [v[k:k + 6], v[k + 6:k + 12]]
    k += 12
    spoly = v[k:k + 6]
    k += 6
    xy = [v[k + 18 * i:k + 18 * (i + 1)] for i in range(n_xy)]
    k += 18 * n_xy
    s = []
    for p, n in enumerate(ns):
        for i in range(n):
            s.append([float(p)] + v[k + 10 * i:k + 10 * (i + 1)])
        k += 10 * n
    if k != len(v):
        raise SystemExit(f"readers: {path}: {len(v)} doubles, the header accounts for {k}")
    return {"XYS06_XYP": ("real[1][6][{n}]", xyp), "XYS06_SPOLY": ("real[1][{n}]", spoly),
            "XYS06_XY": ("real[1][18][{n}]", xy), "XYS06_S": ("real[1][11][{n}]", s)}


READERS = {"igrf13": igrf13, "matlab_matrix": matlab_matrix, "xys06": xys06}


# ------------------------------------------------------------------ the module text a node holds

def data_block(name, ty, rows, doc):
    """One `data` item: its documentation, its type (the outer length filled in), its values a row a line (a 1-D
    table ten a line)."""
    n = len(rows)
    out = [f"## {line}" if line else "##" for line in doc]
    out.append(f"data {name}: {ty.format(n=n)}")
    if rows and isinstance(rows[0], list):
        out += ["    " + ", ".join(shortest(x) for x in r) for r in rows]
    else:
        out += ["    " + ", ".join(shortest(x) for x in rows[i:i + 10]) for i in range(0, n, 10)]
    out.append("end")
    return "\n".join(out)


def module_text(module, about, tables, docs):
    """The module a data node holds: a heading, `module NAME`, and its tables in the order given."""
    head = [f"## {line}" for line in about]
    body = "\n\n".join(data_block(name, ty, rows, docs.get(name, [])) for name, (ty, rows) in tables.items())
    return "\n".join(head) + f"\nmodule {module}\n\n" + body + "\n"


def read(spec):
    """Run the reader a revision names ({reader, file, tables, rename?, types?}) on its file: {name: (type, rows)}."""
    path = ROOT / spec["file"]
    if not path.is_file():
        raise SystemExit(f"readers: {spec['file']} is not in the repository (the revision's file)")
    reader = READERS.get(spec["reader"])
    if reader is None:
        raise SystemExit(f"readers: no reader {spec['reader']!r} (readers: {', '.join(sorted(READERS))})")
    if spec["reader"] == "matlab_matrix":
        got = reader(path, names=list(spec["tables"]))
        out = {}
        for src, dst in spec["tables"].items():
            rows = got[src]
            ty = spec.get("types", {}).get(dst, "real[1][{cols}][{n}]").replace("{cols}", str(len(rows[0])))
            out[dst] = (ty, rows)
        return out
    got = reader(path)
    want = spec.get("tables")
    return {k: got[k] for k in (want or got)}


def sha256(path):
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    if len(argv) != 2 or argv[0] not in READERS:
        sys.exit(f"usage: python3 tools/readers.py {{{'|'.join(sorted(READERS))}}} FILE")
    got = READERS[argv[0]](argv[1])
    if argv[0] == "matlab_matrix":
        got = {k: (f"real[1][{len(v[0])}][{{n}}]", v) for k, v in got.items()}
    print(module_text("read", [f"read by tools/readers.py {argv[0]} from {argv[1]}"], got, {}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
