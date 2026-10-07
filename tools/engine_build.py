#!/usr/bin/env python3
"""The engine build (docs/PLAN_2_0.md S7; docs/S7_INVENTORY.md S7.3): the time engine's published models written from
the design. Only the engine's core stays hand-written (step order, recorder, the toolbox); a model of the world is a
method of an env node, and its Rust is generated here, never edited.

    python3 tools/engine_build.py gen [--check] [--design FILE]
    python3 tools/engine_build.py modules [--design FILE]      which modules each target takes, and from where

What a target takes: every method block of the design whose `code.generate` names the target, and every module its
`code.uses` names, in turn (a published model's data, env's onboard frames and field, the physics' tables, the
toolbox). A module is found by the path its node's origin names (tools/from_design.py `text`), or, for the toolbox
(fsw/pseudocode/01_math.pc), in the repository. The library's translator (trinetra-pcode, `tndb translate`; the
JavaScript one, byte for byte the same, when the library's command is not built) writes each target as a module of
its crate:

  engine/crates/adcs-sim-core/src/gen/   the engine's truth environment: calendar and sidereal time, the field along the
                                         orbit, the fast orbit's atmosphere, elements to state; its scalar maths from
                                         crate::pm (the pure-Rust libm: the same trajectory on every target), no_std
  engine/crates/adcs-pop/src/gen/        the precision orbit's time scales, geodetic coordinates, Earth frames, the
                                         IAU 2006/2000A kernel and the tidal EOP models; std maths (as the Octave POP
                                         they are held to)
  matlab_sils/+asils/+models/            the same models for the MATLAB twin (asils.models.<module>.<function>), one
                                         package of every module the engine's targets take, over the twin's shared
                                         runtime +asils/+pc

Every file it writes says so in its first line. --check exits 1 when one is not what the design gives, or a file in a
generated folder is not one the design gives.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import pathlib
import sqlite3
import subprocess
import sys
import tempfile

import from_design
from common import ROOT, write_text

TNDB = ROOT / "engine" / "target" / "release" / "tndb"
TITLE = "TRI-NETRA engine models, written from the design by tools/engine_build.py"
# where each target goes, and how: the crate's module path and the module its scalar maths come from (None: std)
TARGETS = {
    "adcs-sim-core": {"dir": "engine/crates/adcs-sim-core/src/gen", "root": "crate::gen", "math": "crate::pm"},
    "adcs-pop": {"dir": "engine/crates/adcs-pop/src/gen", "root": "crate::gen", "math": None},
}
# the MATLAB twin's package: every module the engine's targets take, one copy, beside the twin's own code
TWIN = {"dir": "matlab_sils/+asils/+models", "pkg": "asils.models", "takes": list(TARGETS)}
TOOLBOX = ("fsw/pseudocode/01",)       # the toolbox's pseudocode: code, read from the repository


def blocks(db=None):
    """[(node, path of its module, [targets], [paths it uses])] of every method block that names a target."""
    out = []
    with sqlite3.connect(f"file:{db or from_design.design()}?mode=ro", uri=True) as c:
        for nid, x in c.execute("SELECT id, content FROM design_node ORDER BY id"):
            body = json.loads(x)["body"]
            rows = {(s, f): (v, o) for s, f, v, o in body["content"]}
            if ("code", "generate") not in rows or ("code", "pseudocode") not in rows:
                continue
            path = rows[("code", "pseudocode")][1].split(" ")[0]
            gen = [t.strip() for t in rows[("code", "generate")][0].split(",") if t.strip()]
            uses = json.loads(rows[("code", "uses")][0]) if ("code", "uses") in rows else []
            out.append((nid, path, gen, uses))
    return out


def module_text(path, db=None):
    if path.startswith(TOOLBOX):
        f = ROOT / path
        if not f.is_file():
            raise SystemExit(f"engine_build: the toolbox module {path} is not in the repository")
        return f.read_text(encoding="utf-8")
    return from_design.text(path, db)


def modules(target, db=None):
    """{path: text} of every module a target takes: the blocks that name it and, in turn, what they use."""
    bs = blocks(db)
    uses = {p: u for _n, p, _g, u in bs}
    todo = [p for _n, p, g, _u in bs if target in g]
    if not todo:
        raise SystemExit(f"engine_build: the design names no method for {target}")
    got = {}
    while todo:
        p = todo.pop(0)
        if p in got:
            continue
        got[p] = module_text(p, db)
        todo += [q for q in uses.get(p, []) if q not in got]
    return dict(sorted(got.items()))


def translate(texts, root=None, math=None, lang="rust", pkg=None):
    with tempfile.TemporaryDirectory(prefix="engine_build_") as tmp:
        files = []
        for p, t in texts.items():
            f = pathlib.Path(tmp) / pathlib.PurePosixPath(p).name
            if f.exists():
                raise SystemExit(f"engine_build: two modules named {f.name}")
            f.write_text(t, encoding="utf-8")
            files.append(str(f))
        if lang == "rust":
            opts = ["--root", root, "--no-dispatch", "--title", TITLE] + (["--math", math] if math else [])
        else:
            opts = ["--pkg", pkg]
        cmd = [str(TNDB), "translate", lang, *files, *opts] if TNDB.is_file() else \
            ["node", str(ROOT / "design" / "js" / "pcode_cli.mjs"), lang, *files, *opts]
        r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f"engine_build: the translator refused the engine's models:\n{r.stderr.strip()[-3000:]}")
    return json.loads(r.stdout)


def outputs(db=None):
    """{repository path: text} of every generated file."""
    out = {}
    twin = {}
    for target, t in TARGETS.items():
        mods = modules(target, db)
        for rel, text in translate(mods, t["root"], t["math"]).items():
            out[f"{t['dir']}/{pathlib.PurePosixPath(rel).name}"] = text
        if target in TWIN["takes"]:
            twin.update(mods)
    for rel, text in translate(dict(sorted(twin.items())), lang="matlab", pkg=TWIN["pkg"]).items():
        out[f"{TWIN['dir']}/{rel}"] = text
    return out


def gen(a):
    db = pathlib.Path(a.design).resolve() if a.design else None
    outs = outputs(db)
    have = {p.relative_to(ROOT).as_posix() for t in [*TARGETS.values(), TWIN] if (ROOT / t["dir"]).is_dir()
            for p in (ROOT / t["dir"]).rglob("*") if p.is_file()}
    stale = sorted(rel for rel, t in outs.items() if not (ROOT / rel).is_file() or (ROOT / rel).read_text() != t)
    extra = sorted(have - set(outs))
    if a.check:
        for p in stale:
            print(f"engine_build: {p} is not what the design gives (run python3 tools/engine_build.py gen)")
        for p in extra:
            print(f"engine_build: {p} is in a generated folder but the design does not give it")
        print(f"engine_build --check: {len(outs)} generated file(s), {len(stale) + len(extra)} problem(s)")
        return 1 if stale or extra else 0
    for p in extra:
        (ROOT / p).unlink()
    for rel in stale:
        write_text(ROOT / rel, outs[rel])
    print(f"engine_build: {len(outs)} generated file(s), {len(stale)} written, {len(extra)} removed")
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    p = sp.add_parser("gen")
    p.add_argument("--check", action="store_true")
    p.add_argument("--design")
    p = sp.add_parser("modules")
    p.add_argument("--design")
    a = ap.parse_args(argv)
    if a.cmd == "gen":
        return gen(a)
    db = pathlib.Path(a.design).resolve() if a.design else None
    for target in TARGETS:
        print(f"{target}: " + ", ".join(modules(target, db)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
