#!/usr/bin/env python3
"""The flight build (docs/PLAN_2_0.md S6): the flight software's algorithms written from the design, built for a
target and sealed as a flight image.

    python3 tools/flight_build.py gen [--check] [--design FILE] [--out DIR]
                                                write (or check) the generated algorithm sources
    python3 tools/flight_build.py seal --target posix|posix-rs|qemu|qemu-rs [--out DIR]
                                                build the target, check it, seal it as design-<version>.<target>.tnfsw
    python3 tools/flight_build.py verify FILE.tnfsw
                                                recheck a sealed image's hashes; say whether today's build is it
    python3 tools/flight_build.py which RUN [--dir DIR]
                                                the sealed image a result (its manifest.json, or its folder) flew

The algorithms are the design's: the flight algorithm blocks of nav, gdn, ctl and fsw (fsw/pseudocode/03-09, written
from the design by tools/from_design.py) with the toolbox they call (fsw/pseudocode/01-02, code). The library's
translators (trinetra-pcode, `tndb translate`; the JavaScript ones, byte for byte the same, when the library's command
is not built) write them as:
  fsw/alg/include/adcs_alg.h, adcs_alg_rt.h, fsw/alg/src/<module>.c     C99, no dynamic memory, the flight flags
  fsw-rs/src/alg/<module>.rs, rt.rs, mod.rs                            Rust, a module of the no_std flight crate, its
                                                                       scalar maths from crate::m
  fsw/alg/include/adcs_alg_id.h, fsw-rs/src/alg/alg_id.rs              the algorithms' identity: sha256 (first 16 hex)
                                                                       over the C and Rust sources above; both build
                                                                       ids end with it (`... alg <id>`), so every result
                                                                       names the algorithms it flew
  fsw/tests/alg_dispatch.c, fsw-rs/tests/alg/dispatch.rs               the translators' vector dispatcher (a function by
                                                                       its name, inputs flattened): test code only, never
                                                                       in an image; the vector tests call through it the
                                                                       functions the runtime does not call by itself
The runtime stays code (fsw/src/adcs_fsw.c and fsw-rs/src/fsw.rs: the tick and its schedule; the HAL, the C interface,
the parameter blob, the targets); it calls the algorithms through the hand-written signatures the tick has always
called, which now only hand the values to the generated functions.

Every file it writes says so in its first line and is never edited. --check exits 1 when one is not what the design
gives. --design FILE writes from another design's flight algorithm blocks (the toolbox stays the repository's), and
--out DIR writes under DIR (its fsw/ and fsw-rs/ folders) instead of the repository: the two together let a test
show a change made in a design reaching the sources without touching the tree.

The flight image (design/schema.toml, formats.flight_image) is what one target's build is: the design it came from
(version and content hash), the runtime version (the build id the binary carries), the toolchain, every generated
source, the binary and the configuration blobs the engine boots each scenario with, each with its sha256, and the
checks made on it (the sources are the design's, the vectors on the build, the stack on the Cortex-M firmware). It is
the developer's and unsigned (key_signature empty). A result names its image by its manifest's fsw.build_id, which is
the image's runtime_version (`which`).

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import datetime
import getpass
import hashlib
import json
import os
import pathlib
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile

from common import ROOT, write_text

SOURCES = ROOT / "fsw" / "pseudocode"
C_OUT = ROOT / "fsw" / "alg"
RS_OUT = ROOT / "fsw-rs" / "src" / "alg"
TNDB = ROOT / "engine" / "target" / "release" / "tndb"
ADCS = ROOT / "engine" / "target" / "release" / "adcs"
TITLE = "TRI-NETRA flight algorithms, written from the design by tools/flight_build.py"
LIB = "adcs_alg"
# the generated files, by their path in the repository
C_ID = "fsw/alg/include/adcs_alg_id.h"
RS_ID = "fsw-rs/src/alg/alg_id.rs"
C_DISPATCH = "fsw/tests/alg_dispatch.c"
RS_DISPATCH = "fsw-rs/tests/alg/dispatch.rs"
GEN_DIRS = ("fsw/alg", "fsw-rs/src/alg")
DESIGN_BLOCKS = ("03", "04", "05", "06", "07", "08", "09")     # fsw/pseudocode/NN_*: the flight algorithm blocks
IMAGES = ROOT / "results" / "flight_images"
INDEX_MD = ROOT / "results" / "FLIGHT_IMAGES.md"
ID_RE = re.compile(rb"trinetra-fsw-(?:c|rs)/[0-9]+\.[0-9]+\.[0-9]+ \(adcs-fswcfg/1\) alg [0-9a-f]{16}")


# ------------------------------------------------------------------ the sources

def translate(lang, *opts, sources=SOURCES):
    files = [str(p) for p in sorted(pathlib.Path(sources).glob("*.pc"))]
    if TNDB.is_file():
        cmd = [str(TNDB), "translate", lang, *files, *opts]
    else:
        cmd = ["node", str(ROOT / "design" / "js" / "pcode_cli.mjs"), lang, *files, *opts]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f"flight_build: the translator refused the flight algorithms:\n{r.stderr.strip()[-3000:]}")
    return json.loads(r.stdout)


def alg_id(sources):
    """sha256 (first 16 hex) over the generated algorithm sources {repository path: text}: deterministic, so the
    same design gives the same id on every machine."""
    h = hashlib.sha256()
    for p in sorted(sources):
        h.update(p.encode() + b"\0" + sources[p].encode() + b"\0")
    return h.hexdigest()[:16]


def id_files(aid):
    how = ("sha256 (first 16 hex) over the generated algorithm sources, C and Rust (fsw/alg, fsw-rs/src/alg), "
           "this file and its twin excepted")
    c = (f"/* The flight algorithms' identity, written by tools/flight_build.py; do not edit. */\n"
         f"/* {how}. adcs_fsw_build_id() ends with it. */\n"
         "#ifndef ADCS_ALG_ID_H\n#define ADCS_ALG_ID_H\n"
         f"#define ADCS_ALG_ID \"{aid}\"\n"
         "#endif\n")
    rs = (f"//! The flight algorithms' identity, written by tools/flight_build.py; do not edit.\n"
          f"//! {how}. Both build ids end with it.\n"
          "/// The identity as a literal (for `concat!`, which takes literals only).\n"
          f"macro_rules! adcs_alg_id {{ () => {{ \"{aid}\" }} }}\n"
          "/// The identity of the flight algorithms this crate carries.\n"
          "pub const ALG_ID: &str = adcs_alg_id!();\n")
    return {C_ID: c, RS_ID: rs}


def sources_from(design):
    """A folder of pseudocode holding the design's flight algorithm blocks and the repository's toolbox (01-02):
    what gen --design translates. Its path (a temporary folder the caller removes)."""
    import from_design
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="flight_build_"))
    for p in sorted(SOURCES.glob("*.pc")):
        if p.name[:2] not in DESIGN_BLOCKS:
            shutil.copy2(p, tmp / p.name)
    got = 0
    for rel, (b, _how) in from_design.outputs(design).items():
        if rel.startswith("fsw/pseudocode/") and rel.endswith(".pc"):
            (tmp / pathlib.PurePosixPath(rel).name).write_bytes(b)
            got += 1
    if not got:
        shutil.rmtree(tmp, ignore_errors=True)
        raise SystemExit(f"flight_build: {design} holds no flight algorithm block")
    return tmp


def outputs(sources=SOURCES):
    """{repository path: text} of every generated file, and the algorithms' identity."""
    alg = {}
    for rel, text in translate("c", "--lib", LIB, "--no-dispatch", "--title", TITLE, sources=sources).items():
        alg[f"fsw/alg/{rel}"] = text
    for rel, text in translate("rust", "--root", "crate::alg", "--math", "crate::m", "--no-dispatch", "--title", TITLE,
                               sources=sources).items():
        alg[f"fsw-rs/src/alg/{pathlib.PurePosixPath(rel).name}"] = text
    aid = alg_id(alg)
    out = dict(alg)
    out.update(id_files(aid))
    # the vector dispatchers (test code): the same translation with its dispatcher, of which only the dispatcher
    out[C_DISPATCH] = translate("c", "--lib", LIB, "--title", TITLE, sources=sources)["src/dispatch.c"]
    out[RS_DISPATCH] = translate("rust", "--root", "crate::alg", "--math", "crate::m", "--title", TITLE,
                                 sources=sources)["src/dispatch.rs"]
    return out, aid


def current_id(root=ROOT):
    """The algorithms' identity the tree's generated files carry (None when there is none)."""
    f = pathlib.Path(root) / C_ID
    m = re.search(r'#define ADCS_ALG_ID "([0-9a-f]{16})"', f.read_text()) if f.is_file() else None
    return m.group(1) if m else None


def gen(a):
    root = pathlib.Path(a.out).resolve() if a.out else ROOT
    tmp = sources_from(pathlib.Path(a.design)) if a.design else None
    try:
        outs, aid = outputs(tmp or SOURCES)
    finally:
        if tmp:
            shutil.rmtree(tmp, ignore_errors=True)
    path = {rel: root / rel for rel in outs}
    have = {p for d in GEN_DIRS if (root / d).is_dir() for p in (root / d).rglob("*") if p.is_file()}
    stale = sorted(rel for rel, t in outs.items() if not path[rel].is_file() or path[rel].read_text() != t)
    extra = sorted(p.relative_to(root).as_posix() for p in have - set(path.values()))
    if a.check:
        for p in stale:
            print(f"flight_build: {p} is not what the design gives (run python3 tools/flight_build.py gen)")
        for p in extra:
            print(f"flight_build: {p} is in a generated folder but the design does not give it")
        print(f"flight_build --check: {len(outs)} generated file(s), algorithms {aid}, {len(stale) + len(extra)} problem(s)")
        return 1 if stale or extra else 0
    for p in extra:
        (root / p).unlink()
    for rel, t in outs.items():
        if rel in stale:
            write_text(path[rel], t)
    print(f"flight_build: {len(outs)} generated file(s), algorithms {aid}, {len(stale)} written, {len(extra)} removed"
          + (f" (under {root})" if a.out else ""))
    return 0


# ------------------------------------------------------------------ the targets and the seal

def _rs_cabi(*target):
    return (["cargo", "build", "--locked", "--release", "-q", "--no-default-features", "--features", "cabi", *target], "fsw-rs")


# target: what builds it (commands, each with its folder), the image it makes, the toolchain that makes it, its language
TARGETS = {
    "posix": {"build": [(["make", "-s", "build/libadcs_fsw.a"], "fsw")], "image": "fsw/build/libadcs_fsw.a",
              "tools": [["gcc", "--version"], ["ar", "--version"]], "lang": "c",
              "about": "the C flight software as a host static library (the in-process SILS links the same sources)"},
    "posix-rs": {"build": [_rs_cabi()], "image": "fsw-rs/target/release/libadcs_fsw.a",
                 "tools": [["rustc", "--version"], ["cargo", "--version"]], "lang": "rs",
                 "about": "the Rust flight software as a host static library with the C interface"},
    "qemu": {"build": [(["make", "-s", "build/obc_qemu.elf"], "fsw")], "image": "fsw/build/obc_qemu.elf",
             "tools": [["arm-none-eabi-gcc", "--version"]], "lang": "c",
             "about": "the C flight software as Cortex-M4F firmware (QEMU mps2-an386, soft OILS)"},
    "qemu-rs": {"build": [_rs_cabi("--target", "thumbv7em-none-eabihf"), (["make", "-s", "build/obc_qemu_rs.elf"], "fsw")],
                "image": "fsw/build/obc_qemu_rs.elf",
                "tools": [["rustc", "--version"], ["arm-none-eabi-gcc", "--version"]], "lang": "rs",
                "about": "the Rust flight software as Cortex-M4F firmware (QEMU mps2-an386, soft OILS)"},
}
# a result's fsw.impl -> the target whose sources it flew
IMPL_TARGET = [("c (in-process)", "posix"), ("rust (in-process)", "posix-rs"), ("obc_qemu_rs.elf", "qemu-rs"),
               ("obc_qemu.elf", "qemu"), ("obc_posix_rs", "posix-rs"), ("obc_posix", "posix")]


def sha256(b):
    return hashlib.sha256(b).hexdigest()


def run(cmd, cwd="."):
    return subprocess.run(cmd, cwd=ROOT / cwd, capture_output=True, text=True)


def toolchain(target):
    out = []
    for cmd in TARGETS[target]["tools"]:
        try:
            r = subprocess.run(cmd, capture_output=True, text=True)
            first = (r.stdout or r.stderr).strip().splitlines()
            out.append(first[0] if first else f"{cmd[0]}: no version")
        except FileNotFoundError:
            out.append(f"{cmd[0]}: not found")
    return "; ".join(out)


def slug(version):
    return re.sub(r"[^A-Za-z0-9._-]+", "-", version).strip("-")


def runtime_version(binary):
    """The build id the binary carries (the one adcs_fsw_build_id returns), or None."""
    ids = sorted(set(ID_RE.findall(binary)))
    return ids[0].decode() if len(ids) == 1 else None


def expected_id(lang, aid):
    ver = (ROOT / "VERSION").read_text().strip()
    return f"trinetra-fsw-{lang}/{ver} (adcs-fswcfg/1) alg {aid}"


def blobs(design):
    """{scenario: the adcs-fswcfg/1 blob the engine boots it with}, read from the design itself."""
    if not ADCS.is_file():
        raise SystemExit("flight_build: the engine is not built (python3 tools/engine.py build): it makes the configuration blobs")
    with sqlite3.connect(f"file:{design}?mode=ro", uri=True) as c:
        scen = [pathlib.PurePosixPath(p).stem for (p,) in c.execute(
            "SELECT \"path\" FROM engine_input WHERE \"path\" LIKE 'data/scenarios/%.json' ORDER BY \"path\"")]
    out = {}
    env = dict(os.environ, TRINETRA_DESIGN=str(design))
    with tempfile.TemporaryDirectory() as tmp:
        for s in scen:
            f = pathlib.Path(tmp) / f"{s}.fswcfg"
            r = subprocess.run([str(ADCS), "params", s, "--out", str(f)], capture_output=True, text=True, env=env, cwd=tmp)
            if r.returncode:
                raise SystemExit(f"flight_build: adcs params {s} failed: {(r.stderr or r.stdout).strip()[-500:]}")
            out[s] = f.read_bytes()
    return out


def checks_for(target, aid, outs):
    """[(name, passed, note)]: the checks made on the target before its image is built."""
    import from_design
    t = TARGETS[target]
    rows = []
    stale = [rel for rel, text in outs.items() if not (ROOT / rel).is_file() or (ROOT / rel).read_text() != text]
    rows.append(("flight_build gen --check", int(not stale),
                 f"every generated source is what the design gives (algorithms {aid})" if not stale else f"{len(stale)} stale: {', '.join(stale[:5])}"))
    bad = [x for x in from_design.stale() if x.startswith(("fsw/pseudocode/", "fsw/params/"))]
    rows.append(("from_design: the flight algorithm blocks", int(not bad),
                 "fsw/pseudocode/03-09 and fsw/params/params.toml are what the design gives" if not bad else "; ".join(bad[:3])))
    if t["lang"] == "c":
        r = run(["make", "-s", "check"], "fsw")
        rows.append(("C flight flags", int(r.returncode == 0),
                     "every warning an error, no malloc/time/rand (make check)" if r.returncode == 0 else (r.stdout + r.stderr).strip()[-300:]))
        r = run(["make", "-s", "build/test_pcode"], "fsw")
        if r.returncode == 0:
            r = run(["./build/test_pcode"], "fsw")
        last = ((r.stdout or r.stderr).strip().splitlines() or ["no output"])[-1]
        where = "on the host build of these sources (fsw/build/test_pcode, the library posix seals)"
        rows.append(("vectors", int(r.returncode == 0), (f"{last}; " if r.returncode == 0 else f"FAILED: {last}; ") +
                     (where if target == "posix" else where + "; the Cortex-M image is not run on its vectors here")))
    else:
        r = run(["cargo", "test", "--locked", "--release", "-q", "--test", "pcode", "--", "--nocapture"], "fsw-rs")
        last = [x for x in (r.stdout + r.stderr).splitlines() if "functions" in x and "values" in x]
        note = (last[-1].strip() if last else (r.stdout + r.stderr).strip()[-200:])
        where = "on the host build of these sources (cargo test --test pcode, std)"
        rows.append(("vectors", int(r.returncode == 0), (note if r.returncode == 0 else "FAILED: " + note) + "; " +
                     (where if target == "posix-rs" else where + "; the Cortex-M image is not run on its vectors here")))
    if target == "qemu":
        r = run([sys.executable, "tools/fsw_stack.py"])
        last = ((r.stdout or r.stderr).strip().splitlines() or ["no output"])[-1]
        rows.append(("stack", int(r.returncode == 0), last + " (tools/fsw_stack.py)"))
    elif target == "qemu-rs":
        rows.append(("stack", 0, "NOT RUN: tools/fsw_stack.py reads GCC's call graph of the C firmware; the Rust firmware has no stack analysis here"))
    return rows


def seal(a):
    import from_design
    import tndb
    target = a.target
    t = TARGETS[target]
    design = from_design.design()
    outs, aid = outputs()
    rows = checks_for(target, aid, outs)
    for cmd, cwd in t["build"]:          # the image last: a test build (std) can overwrite the host static library
        r = run(cmd, cwd)
        if r.returncode:
            raise SystemExit(f"flight_build: {' '.join(cmd)} failed:\n{(r.stdout + r.stderr)[-2000:]}")
    binary = (ROOT / t["image"]).read_bytes()
    rv = runtime_version(binary)
    want = expected_id(t["lang"], aid)
    rows.append(("build id", int(rv == want), f"the image carries {rv!r}" + ("" if rv == want else f"; the sources give {want!r}")))
    cfg = blobs(design)
    version = from_design.version(design)
    out = pathlib.Path(a.out).resolve() if a.out else IMAGES
    out.mkdir(parents=True, exist_ok=True)
    path = out / f"design-{slug(version)}.{target}.tnfsw"
    if path.exists():
        path.unlink()                    # a seal of today's build replaces the last seal of the same design and target
    files = [(rel, sha256(text.encode()), text.encode()) for rel, text in sorted(outs.items())
             if not rel.startswith(("fsw/tests/", "fsw-rs/tests/"))]
    files.append((t["image"], sha256(binary), binary))
    files += [(f"blob/{s}.fswcfg", sha256(b), b) for s, b in sorted(cfg.items())]
    image = (target, version, from_design.fingerprint(design), rv or "?", toolchain(target),
             datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), getpass.getuser(), sha256(binary))

    def fill(c):
        c.execute("INSERT INTO flight_image VALUES (?, ?, ?, ?, ?, ?, ?, ?)", image)
        c.executemany("INSERT INTO flight_file VALUES (?, ?, ?)", files)
        c.executemany("INSERT INTO flight_check VALUES (?, ?, ?)", rows)
        c.execute('INSERT INTO meta VALUES (?, ?)', ("signed", "no: the developer's build, not signed (key_signature empty)"))

    tndb.create(path, "flight_image", f"{slug(version)}.{target}", written_by="tools/flight_build.py", fill=fill, sync=False)
    print(f"flight_build: sealed {path.relative_to(ROOT) if path.is_relative_to(ROOT) else path}: {target}, design {version}, "
          f"{rv}, image {sha256(binary)[:16]}, {len(files)} file(s), {path.stat().st_size} bytes")
    for name, ok, note in rows:
        print(f"  [{'ok' if ok else 'NOT PASSED'}] {name}: {note}")
    if out == IMAGES:
        write_index()
    # a check that could not run on this target is recorded as not passed with its reason; it does not fail the seal
    return 0 if all(ok or note.startswith("NOT RUN") for _name, ok, note in rows) else 1


def read_image(path):
    import tndb
    c = tndb.open_file(path, "flight_image")
    try:
        cols = [r[1] for r in c.execute('PRAGMA table_info("flight_image")')]
        img = dict(zip(cols, c.execute("SELECT * FROM flight_image").fetchone()))
        files = c.execute("SELECT path, sha256, bytes FROM flight_file ORDER BY path").fetchall()
        checks = c.execute("SELECT name, passed, note FROM flight_check ORDER BY rowid").fetchall()
        sigs = c.execute("SELECT COUNT(*) FROM key_signature").fetchone()[0]
    finally:
        c.close()
    return img, files, checks, sigs


def write_index():
    """results/flight_images/index.json and results/FLIGHT_IMAGES.md: every sealed image's target, design, build id,
    image hash and checks (the .tnfsw files themselves are not kept in git: the Rust static library alone is 8 MB)."""
    rows = []
    order = list(TARGETS)
    for p in sorted(IMAGES.glob("*.tnfsw")):
        img, files, checks, sigs = read_image(p)
        rows.append({"file": p.name, "bytes": p.stat().st_size, **{k: img[k] for k in ("target", "design_version", "design_fingerprint",
                     "runtime_version", "toolchain", "built_at", "built_by", "image_sha256")},
                     "files": len(files), "signed": bool(sigs), "checks": [{"name": n, "passed": bool(ok), "note": note} for n, ok, note in checks]})
    rows.sort(key=lambda r: (r["design_version"], order.index(r["target"]) if r["target"] in order else len(order)))
    write_text(IMAGES / "index.json", json.dumps({"schema": "trinetra-flight-images/1", "images": rows}, indent=1) + "\n")
    L = ["# The sealed flight images", "",
         "Owner: Agastya. Written by `python3 tools/flight_build.py seal` (docs/PLAN_2_0.md S6). Each image is one target's",
         "build of the flight software written from the design: the generated sources, the binary and the configuration",
         "blobs of every scenario, each with its sha256, and the checks made on it. The `.tnfsw` files sit in",
         "`results/flight_images/` and are not kept in git (the Rust host library alone is 8 MB); `index.json` is.",
         "`python3 tools/flight_build.py verify FILE` rechecks one; `which RUN` names the image a result flew (its manifest's",
         "fsw.build_id is the image's runtime version). They are the developer's builds, not signed.", "",
         "| target | design | runtime version (build id) | image sha256 | files | checks |", "|---|---|---|---|---:|---|"]
    for r in rows:
        ok = [c["name"] for c in r["checks"] if c["passed"]]
        no = [c["name"] for c in r["checks"] if not c["passed"] and not c["note"].startswith("NOT RUN")]
        nr = [c["name"] for c in r["checks"] if not c["passed"] and c["note"].startswith("NOT RUN")]
        L.append(f"| {r['target']} | {r['design_version']} (`{r['design_fingerprint'][:12]}`) | `{r['runtime_version']}` | "
                 f"`{r['image_sha256'][:16]}` | {r['files']} | {len(ok)} passed" + (f"; NOT PASSED: {', '.join(no)}" if no else "")
                 + (f"; not run: {', '.join(nr)}" if nr else "") + " |")
    L += ["", "## Checks", ""]
    for r in rows:
        L.append(f"**{r['target']}** ({r['file']}, built {r['built_at']} with {r['toolchain']}):")
        L += [f"- {'passed' if c['passed'] else 'not run' if c['note'].startswith('NOT RUN') else 'NOT PASSED'}: {c['name']}: {c['note']}"
              for c in r["checks"]]
        L.append("")
    write_text(INDEX_MD, "\n".join(L).rstrip() + "\n")


def verify(a):
    import tndb
    path = pathlib.Path(a.file)
    errs = tndb.check(path)
    if errs:
        for e in errs:
            print("flight_build verify: " + e)
        return 1
    img, files, checks, sigs = read_image(path)
    damaged = [p for p, h, b in files if sha256(b) != h]
    binrow = [x for x in files if x[0] == TARGETS.get(img["target"], {}).get("image")]
    if not binrow or binrow[0][1] != img["image_sha256"]:
        damaged.append("the image's own hash is not its binary's")
    print(f"{path.name}: {img['target']}, design {img['design_version']} ({img['design_fingerprint'][:16]}), {img['runtime_version']}")
    print(f"  built {img['built_at']} by {img['built_by']} with {img['toolchain']}; {'signed' if sigs else 'not signed: the developer build'}")
    print(f"  {len(files)} file(s): " + ("every hash holds" if not damaged else f"{len(damaged)} DAMAGED: {', '.join(damaged[:5])}"))
    print("  checks: " + ", ".join(f"{n} {'passed' if ok else 'not run' if note.startswith('NOT RUN') else 'NOT PASSED'}"
                                  for n, ok, note in checks))
    differ, missing, same = [], [], 0
    for p, h, _b in files:
        if p.startswith("blob/"):
            continue
        f = ROOT / p
        if not f.is_file():
            missing.append(p)
        elif sha256(f.read_bytes()) != h:
            differ.append(p)
        else:
            same += 1
    blobs_note = ""
    if ADCS.is_file() and not a.no_blobs:
        import from_design
        now = blobs(from_design.design())
        bdiff = [p for p, h, _b in files if p.startswith("blob/") and sha256(now.get(p[5:-7], b"")) != h]
        blobs_note = f"; blobs: {'every one equal' if not bdiff else f'{len(bdiff)} differ'}"
        differ += bdiff
    aid = current_id()
    print(f"  today's tree: {same} file(s) equal, {len(differ)} differ, {len(missing)} missing{blobs_note}; algorithms now {aid}")
    for p in (differ + missing)[:10]:
        print(f"    {'differs' if p in differ else 'missing'}: {p}")
    image_now = TARGETS.get(img["target"], {}).get("image")
    same_bin = image_now and (ROOT / image_now).is_file() and sha256((ROOT / image_now).read_bytes()) == img["image_sha256"]
    print(f"  the current build's binary {'IS' if same_bin else 'is NOT'} the sealed image; "
          f"its sources {'ARE' if not [p for p in differ + missing if p != image_now and not p.startswith('blob/')] else 'are NOT'} the sealed ones")
    return 1 if damaged else 0


def which(a):
    run_path = pathlib.Path(a.run)
    man = run_path / "manifest.json" if run_path.is_dir() else run_path
    m = json.loads(man.read_text())
    impl, bid = m.get("fsw", {}).get("impl", ""), m.get("fsw", {}).get("build_id", "")
    rv = re.sub(r" via adcs-link/1$", "", bid or "")
    target = next((t for key, t in IMPL_TARGET if key in impl), None)
    print(f"{man}: flown on {impl or '?'}, build id {bid or '(none)'}")
    if not ID_RE.fullmatch(rv.encode()):
        print("  its build id names no algorithms (flown before the image named them, docs/PLAN_2_0.md S6 step 6): no sealed image can be named")
        return 1
    found = []
    for p in sorted(pathlib.Path(a.dir).glob("*.tnfsw")):
        img, _f, _c, _s = read_image(p)
        if img["runtime_version"] == rv:
            found.append((p, img))
    exact = [(p, i) for p, i in found if i["target"] == target]
    for p, img in exact or found:
        if img["target"] != target:
            how = f"the same algorithms and runtime version, sealed for {img['target']}"
        elif target in ("qemu", "qemu-rs"):
            elf = ROOT / TARGETS[target]["image"]
            now = elf.is_file() and sha256(elf.read_bytes()) == img["image_sha256"]
            how = (f"the same runtime version; the firmware the engine loads ({TARGETS[target]['image']}) "
                   f"{'IS' if now else 'is NOT'} this image today (the run itself records its build id, not the firmware's hash)")
        else:
            how = "the same sources and runtime version (the engine compiles them into itself, in-process)"
        print(f"  {p}: {img['target']}, design {img['design_version']}, image {img['image_sha256'][:16]}: {how}")
    if not found:
        print(f"  no sealed image in {a.dir} has the runtime version {rv!r}")
        return 1
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    g = sp.add_parser("gen")
    g.add_argument("--check", action="store_true")
    g.add_argument("--design", help="write from this design's flight algorithm blocks (default: fsw/pseudocode as it is)")
    g.add_argument("--out", help="write under this folder (its fsw/ and fsw-rs/) instead of the repository")
    s = sp.add_parser("seal")
    s.add_argument("--target", required=True, choices=sorted(TARGETS))
    s.add_argument("--out", help=f"where the .tnfsw goes (default {IMAGES.relative_to(ROOT)})")
    v = sp.add_parser("verify")
    v.add_argument("file")
    v.add_argument("--no-blobs", action="store_true", help="do not make the blobs again to compare them")
    w = sp.add_parser("which")
    w.add_argument("run", help="a result's manifest.json, or its folder")
    w.add_argument("--dir", default=str(IMAGES), help="where the sealed images are")
    a = ap.parse_args(argv)
    return {"gen": gen, "seal": seal, "verify": verify, "which": which}[a.cmd](a)


if __name__ == "__main__":
    sys.exit(main())
