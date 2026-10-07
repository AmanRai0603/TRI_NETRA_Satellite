#!/usr/bin/env python3
"""The readers of published data (docs/PLAN_2_0.md, the boundary: code keeps the readers of files' formats, which load
published data into the database, never models of their own). Each reader reads one file format and gives the
tables it holds, as the pseudocode's `data` items (docs/PSEUDOCODE_V2.md, *Data tables*) that a node of the design
holds; the developer's revisions (design/revisions_2_0.toml, `[[revision.data]]`) name the reader, the file and the
publication, and tools/convert_2_0.py runs it. Nothing here is a value: every number comes from the file.

    python3 tools/readers.py READER FILE        print the tables READER reads from FILE (the module text)
    python3 tools/readers.py de440_slice FILE JD0 JD1      the same for the DE440 records of a span of TDB dates

Readers (each returns {table name: (type, rows)}: a 1-D table a list of numbers, a 2-D one a list of rows):
  igrf13        IAGA's igrf13coeffs.txt: the IGRF-13 epochs and Gauss coefficients [nT] (gen_fsw_params.igrf_table,
                as the flight software's and the engine's tables have always been made)
  matlab_matrix the numeric matrices a MATLAB file assigns (`NAME = [...]`), e.g. the Octave POP's leapTable.m and
                its IERS tidal-EOP tables, and the numbers it assigns (`F.mu = 3.986004415e14;`)
  matlab_cell   the rows of a cell array a MATLAB file assigns (`L = {...};`), each row's numbers in order (the Octave
                POP's ocean-tide main lines)
  xys06         the IAU 2006/2000A X, Y and s series, as refgen exported xys06_tables.mat (xys06.bin: little-endian
                doubles, a header of counts, then the polynomials and the terms)
  text_table    a whitespace-separated table of numbers, `#` lines comments (refgen's DTM2020 coefficient exports:
                96 rows, the terms, of 9 columns, tt h he o az2 o2 az t0 tp)
  solfsmy       Space Environment Technologies' SOLFSMY.TXT (JB2008's daily solar indices): a row a day, year, day of
                year, F10, F81c, S10, S81c, M10, M81c, Y10, Y81c [sfu] (the Julian day and the source flag dropped)
  dtcfile       Space Environment Technologies' DTCFILE.TXT (JB2008's Dst-driven temperature change): a row a day,
                year, day of year, then dTc [K] for each of the 24 hours
  fes_bin       refgen's export of the Octave POP's force_data/fes2004_deg10.mat (POPFES01: the note, then the
                Doodson multipliers, degree, order and the four Stokes coefficients of each line)
  de440_slice   a JPL DAF/SPK kernel (de440s.bsp): the Chebyshev records (type 2) of the Sun, the Earth-Moon
                barycentre, the Earth and the Moon over a span of TDB Julian dates (the runs' span), with each
                segment's footer and where the slice sits in it (the parser of adcs-pop spk.rs, de440.open)

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import hashlib
import math
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
    or a line end, values at blanks or commas, comments (`%` to the line end) dropped, a line continued by `...` joined
    to the next. Python's float() reads each decimal to the nearest double, as MATLAB and Rust do."""
    text = pathlib.Path(path).read_text(encoding="utf-8")
    text = "\n".join(line.split("%", 1)[0] for line in text.splitlines())
    text = re.sub(r"\.\.\.[^\n]*\n", " ", text)
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
    for name, value in _SCALAR.findall(text):
        if names is not None and name in names and name not in out:
            out[name] = [[float(value)]]
    if names is not None and set(names) - set(out):
        raise SystemExit(f"readers: {path}: no numeric matrix {sorted(set(names) - set(out))}")
    return out


_SCALAR = re.compile(r"(?m)^\s*([A-Za-z_][\w.]*)\s*=\s*([-+]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][-+]?[0-9]+)?)\s*;")
_CELL = re.compile(r"\b([A-Za-z_]\w*)\s*=\s*\{([^}]*)\}", re.S)


def matlab_cell(path, names=None):
    """The rows of each cell array a MATLAB file assigns as a literal (`L = { [2 0 0], 2, 2, -3.10; ... };`), by name:
    rows split at a line end or `;`, each row the numbers it holds in order (brackets and commas dropped), comments (`%`
    to the line end) dropped."""
    text = pathlib.Path(path).read_text(encoding="utf-8")
    text = "\n".join(line.split("%", 1)[0] for line in text.splitlines())
    out = {}
    for name, body in _CELL.findall(text):
        if names is not None and name not in names:
            continue
        rows = [[float(v) for v in _NUM.findall(r)] for r in re.split(r"[;\n]", body)]
        rows = [r for r in rows if r]
        if len({len(r) for r in rows}) != 1:
            raise SystemExit(f"readers: {path}: {name} has rows of different lengths")
        out[name] = rows
    if names is not None and set(names) - set(out):
        raise SystemExit(f"readers: {path}: no cell array {sorted(set(names) - set(out))}")
    return out


def fes_bin(path):
    """The FES2004 table refgen/gravity_tides_fes_export.m wrote from force_data/fes2004_deg10.mat: `POPFES01`, the
    number of lines and the note's length (uint32), the note, then by column the six Doodson multipliers (int8), the
    degree and order (uint8), and Cp, Sp, Cm, Sm (doubles), little-endian. A row of FES2004: the multipliers of tau, s,
    h, p, N', ps, the degree, the order, then Cp, Sp, Cm, Sm (normalised, SI)."""
    b = pathlib.Path(path).read_bytes()
    if len(b) < 16 or b[:8] != b"POPFES01":
        raise SystemExit(f"readers: {path}: not a POPFES01 table")
    n, ln = struct.unpack_from("<II", b, 8)
    o = 16 + ln
    if len(b) != o + 6 * n + 2 * n + 32 * n:
        raise SystemExit(f"readers: {path}: {len(b)} bytes, the header accounts for {o + 40 * n}")
    dood = struct.unpack_from(f"<{6 * n}b", b, o)
    o += 6 * n
    deg = struct.unpack_from(f"<{n}B", b, o)
    order = struct.unpack_from(f"<{n}B", b, o + n)
    o += 2 * n
    cols = [struct.unpack_from(f"<{n}d", b, o + 8 * n * k) for k in range(4)]
    rows = [[float(x) for x in dood[6 * i:6 * i + 6]] + [float(deg[i]), float(order[i])] + [c[i] for c in cols] for i in range(n)]
    return {"FES2004": ("real[1][12][{n}]", rows)}


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


_NUM = re.compile(r"[+-]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?")


def _numbers(line):
    """The leading numbers of a line, as MATLAB's sscanf(L, '%f') reads them: token by token, a token that is not a
    number giving its numeric prefix, if it has one, and ending the scan (the SET files' source flag, `1B11`, gives 1)."""
    out = []
    for tok in line.split():
        m = _NUM.match(tok)
        if m and m.end() == len(tok):
            out.append(float(tok))
            continue
        if m:
            out.append(float(m.group(0)))
        break
    return out


def text_table(path):
    """A table of numbers, a row a line, `#` lines and blank lines skipped (refgen's DTM2020 coefficient exports,
    written with %.17g, so each decimal reads back as the double the MATLAB held)."""
    rows = []
    for line in pathlib.Path(path).read_text(encoding="utf-8").splitlines():
        t = line.strip()
        if not t or t.startswith("#"):
            continue
        rows.append([float(x) for x in t.split()])
    if not rows or len({len(r) for r in rows}) != 1:
        raise SystemExit(f"readers: {path}: not a table of rows of one length")
    return {"TABLE": ("real[1][{cols}][{n}]".replace("{cols}", str(len(rows[0]))), rows)}


def _ascending(path, rows):
    keys = [(r[0], r[1]) for r in rows]
    if any(b <= a for a, b in zip(keys, keys[1:])):
        raise SystemExit(f"readers: {path}: the days are not in ascending order")


def solfsmy(path):
    """SOLFSMY.TXT as get_jb2008_indices's parse_solfsmy reads it: a data line has at least 11 numbers and a year
    after 1900 and before 2100; comment lines (`#`, `%`) and blank ones are skipped. A row: YYYY, DDD, F10, F81c, S10,
    S81c, M10, M81c, Y10, Y81c (the Julian day, column 3, is not read by the model). The file's days must ascend."""
    rows = []
    for line in pathlib.Path(path).read_text(encoding="utf-8").splitlines():
        t = line.strip()
        if not t or t[0] in "#%":
            continue
        v = _numbers(t.replace("\t", " "))
        if len(v) >= 11 and 1900.0 < v[0] < 2100.0:
            rows.append([v[0], v[1]] + v[3:11])
    if not rows:
        raise SystemExit(f"readers: {path}: no SOLFSMY data line")
    _ascending(path, rows)
    return {"SOLFSMY": ("real[1][10][{n}]", rows)}


def dtcfile(path):
    """DTCFILE.TXT as get_jb2008_indices's parse_dtcfile reads it: a line starting `DTC` with at least 26 numbers
    after it, YYYY, DDD and the 24 hourly dTc [K]. The file's days must ascend."""
    rows = []
    for line in pathlib.Path(path).read_text(encoding="utf-8").splitlines():
        t = line.strip()
        if len(t) < 3 or t[:3].upper() != "DTC":
            continue
        v = _numbers(t[3:].replace("\t", " "))
        if len(v) >= 26:
            rows.append(v[:26])
    if not rows:
        raise SystemExit(f"readers: {path}: no DTC line")
    _ascending(path, rows)
    return {"DTCFILE": ("real[1][26][{n}]", rows)}


# the segments the precision orbit evaluates (de440.sun, de440.moon, de440.earth): (center, target, table)
DE440_SEGMENTS = [(0, 10, "DE440_SUN"), (0, 3, "DE440_EMB"), (3, 399, "DE440_EARTH"), (3, 301, "DE440_MOON")]


def daf_spk(path):
    """A DAF/SPK file as de440.open reads it: (the file's doubles, [(center, target, frame, type, sa, ea, init,
    intlen, rsize, n)] in file order). Both byte orders (LTL-IEEE, BIG-IEEE)."""
    b = pathlib.Path(path).read_bytes()
    if len(b) < 1024 or not b.startswith(b"DAF/SPK"):
        raise SystemExit(f"readers: {path}: not a DAF/SPK file")
    big = b[88:96].decode("ascii", "replace").strip() == "BIG-IEEE"
    e = ">" if big else "<"
    i32 = lambda off: struct.unpack_from(e + "i", b, off)[0]
    nd, ni, fward = i32(8), i32(12), i32(76)
    if not (2 <= nd <= 124 and 6 <= ni <= 250):
        raise SystemExit(f"readers: {path}: bad ND/NI {nd}/{ni}")
    nw = len(b) // 8
    words = struct.unpack_from(f"{e}{nw}d", b, 0)
    ss = nd + (ni + 1) // 2
    segs, recno, guard = [], fward, 0
    while recno != 0:
        guard += 1
        if recno < 0 or guard > nw // 128 + 1:
            raise SystemExit(f"readers: {path}: corrupt summary record chain")
        base = (recno - 1) * 128
        for k in range(int(words[base + 2])):
            off = base + 3 + k * ss
            iw = off + nd + 1
            ints = [i32((iw - 1) * 8 + 4 * j) for j in range(ni)]
            tgt, ctr, frame, typ, sa, ea = ints[:6]
            init, intlen, rsize, n = words[ea - 4:ea]
            segs.append((ctr, tgt, frame, typ, sa, ea, init, intlen, int(rsize), int(n)))
        recno = int(words[base])
    return words, segs


def de440_slice(path, span_jd_tdb):
    """The records of the Sun (SSB -> 10), the Earth-Moon barycentre (SSB -> 3), the Earth (3 -> 399) and the Moon
    (3 -> 301) that cover a span of TDB Julian dates, each a row as the kernel holds it: the interval's mid-point and
    half-length [s past J2000 TDB], then the X, Y and Z Chebyshev coefficients [km]. DE440_SEGMENTS, a row a segment in
    that order: center, target, INIT and INTLEN [s], RSIZE, the coefficients a component, N (the segment's records),
    the index in the segment of the slice's first record and the records in the slice. A record is chosen as
    de440.state chooses it: floor((et - INIT)/INTLEN), held to the segment."""
    words, segs = daf_spk(path)
    et0, et1 = ((jd - 2451545.0) * 86400.0 for jd in span_jd_tdb)
    out, rows = {}, []
    for ctr, tgt, name in DE440_SEGMENTS:
        seg = next((s for s in segs if s[0] == ctr and s[1] == tgt), None)
        if seg is None:
            raise SystemExit(f"readers: {path}: segment {ctr} -> {tgt} not in the kernel")
        _c, _t, frame, typ, sa, _ea, init, intlen, rsize, n = seg
        if typ != 2 or frame != 1:
            raise SystemExit(f"readers: {path}: segment {ctr} -> {tgt} is type {typ}, frame {frame}, not type 2 in J2000")
        rec = lambda et: min(max(math.floor((et - init) / intlen), 0), n - 1)
        k0, k1 = rec(et0), rec(et1)
        recs = [list(words[sa - 1 + k * rsize:sa - 1 + (k + 1) * rsize]) for k in range(k0, k1 + 1)]
        out[name] = (f"real[1][{rsize}][{{n}}]", recs)
        rows.append([float(ctr), float(tgt), init, intlen, float(rsize), float((rsize - 2) // 3), float(n), float(k0), float(len(recs))])
    return {"DE440_SEGMENTS": ("real[1][9][{n}]", rows), **out}


READERS = {"igrf13": igrf13, "matlab_matrix": matlab_matrix, "xys06": xys06, "text_table": text_table,
           "solfsmy": solfsmy, "dtcfile": dtcfile, "de440_slice": de440_slice, "matlab_cell": matlab_cell, "fes_bin": fes_bin}


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
    """Run the reader a revision names ({reader, file, tables?, types?, args?}) on its file: {name: (type, rows)}.
    tables: the tables to keep, or {the reader's name: the table's}; args: what the reader takes besides the file
    (de440_slice: the span)."""
    path = ROOT / spec["file"]
    if not path.is_file():
        raise SystemExit(f"readers: {spec['file']} is not in the repository (the revision's file)")
    reader = READERS.get(spec["reader"])
    if reader is None:
        raise SystemExit(f"readers: no reader {spec['reader']!r} (readers: {', '.join(sorted(READERS))})")
    if spec["reader"] in ("matlab_matrix", "matlab_cell"):
        got = reader(path, names=list(spec["tables"]))
        out = {}
        for src, dst in spec["tables"].items():
            rows = got[src]
            ty = spec.get("types", {}).get(dst, "real[1][{cols}][{n}]").replace("{cols}", str(len(rows[0])))
            if ty.count("][") == 1:       # a 1-D table: the matrix's one row, or its one column
                if len(rows) == 1:
                    rows = rows[0]
                elif all(len(r) == 1 for r in rows):
                    rows = [r[0] for r in rows]
                else:
                    raise SystemExit(f"readers: {spec['file']}: {src} is {len(rows)} x {len(rows[0])}, not a row or a column")
            out[dst] = (ty, rows)
        return out
    got = reader(path, **spec.get("args", {}))
    want = spec.get("tables")
    if isinstance(want, dict):            # renamed: {the reader's name: the table's}
        return {dst: got[src] for src, dst in want.items()}
    return {k: got[k] for k in (want or got)}


def sha256(path):
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    span = argv[:1] == ["de440_slice"] and len(argv) == 4
    if not (len(argv) == 2 or span) or argv[0] not in READERS:
        sys.exit(f"usage: python3 tools/readers.py {{{'|'.join(sorted(READERS))}}} FILE (de440_slice: FILE JD0 JD1, TDB)")
    got = READERS[argv[0]](argv[1], [float(argv[2]), float(argv[3])]) if span else READERS[argv[0]](argv[1])
    if argv[0] in ("matlab_matrix", "matlab_cell"):
        got = {k: (f"real[1][{len(v[0])}][{{n}}]", v) for k, v in got.items()}
    print(module_text("read", [f"read by tools/readers.py {argv[0]} from {argv[1]}"], got, {}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
