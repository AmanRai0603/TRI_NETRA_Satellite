#!/usr/bin/env python3
"""The flight build (docs/PLAN_2_0.md S6): the flight software's algorithms written from the design.

    python3 tools/flight_build.py gen [--check]     write (or check) the generated algorithm sources

The algorithms are the design's: the flight algorithm blocks of nav, gdn, ctl and fsw (fsw/pseudocode/03-09, written
from the design by tools/from_design.py) with the toolbox they call (fsw/pseudocode/01-02, code). The library's
translators (trinetra-pcode, `tndb translate`; the JavaScript ones, byte for byte the same, when the library's command
is not built) write them as:
  fsw/alg/include/adcs_alg.h, adcs_alg_rt.h, fsw/alg/src/<module>.c     C99, no dynamic memory, the flight flags
  fsw-rs/src/alg/<module>.rs, rt.rs, mod.rs                            Rust, a module of the no_std flight crate, its
                                                                       scalar maths from crate::m
The runtime stays code (fsw/src/adcs_fsw.c and fsw-rs/src/fsw.rs: the tick and its schedule; the HAL, the C interface,
the parameter blob, the targets); it calls the algorithms through the hand-written signatures the tick has always
called, which now only hand the values to the generated functions.

Every file it writes says so in its first line and is never edited. --check exits 1 when one is not what the design
gives.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import pathlib
import subprocess
import sys

from common import ROOT, write_text

SOURCES = ROOT / "fsw" / "pseudocode"
C_OUT = ROOT / "fsw" / "alg"
RS_OUT = ROOT / "fsw-rs" / "src" / "alg"
TNDB = ROOT / "engine" / "target" / "release" / "tndb"
TITLE = "TRI-NETRA flight algorithms, written from the design by tools/flight_build.py"
LIB = "adcs_alg"


def translate(lang, *opts):
    files = [str(p) for p in sorted(SOURCES.glob("*.pc"))]
    if TNDB.is_file():
        cmd = [str(TNDB), "translate", lang, *files, *opts]
    else:
        cmd = ["node", str(ROOT / "design" / "js" / "pcode_cli.mjs"), lang, *files, *opts]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f"flight_build: the translator refused the flight algorithms:\n{r.stderr.strip()[-3000:]}")
    return json.loads(r.stdout)


def outputs():
    """{path: text} of every generated algorithm source."""
    out = {}
    for rel, text in translate("c", "--lib", LIB, "--no-dispatch", "--title", TITLE).items():
        out[C_OUT / rel] = text
    for rel, text in translate("rust", "--root", "crate::alg", "--math", "crate::m", "--no-dispatch", "--title", TITLE).items():
        out[RS_OUT / pathlib.PurePosixPath(rel).name] = text
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    g = sp.add_parser("gen")
    g.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    outs = outputs()
    have = {p for d in (C_OUT, RS_OUT) if d.is_dir() for p in d.rglob("*") if p.is_file()}
    stale = sorted(str(p.relative_to(ROOT)) for p, t in outs.items() if not p.is_file() or p.read_text() != t)
    extra = sorted(str(p.relative_to(ROOT)) for p in have - set(outs))
    if a.check:
        for p in stale:
            print(f"flight_build: {p} is not what the design gives (run python3 tools/flight_build.py gen)")
        for p in extra:
            print(f"flight_build: {p} is in a generated folder but the design does not give it")
        print(f"flight_build --check: {len(outs)} generated file(s), {len(stale) + len(extra)} problem(s)")
        return 1 if stale or extra else 0
    for p in extra:
        (ROOT / p).unlink()
    for p, t in outs.items():
        if not p.is_file() or p.read_text() != t:
            write_text(p, t)
    print(f"flight_build: {len(outs)} generated file(s), {len(stale)} written, {len(extra)} removed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
