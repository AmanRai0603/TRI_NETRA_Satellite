#!/usr/bin/env python3
"""The tool as a team member gets it: the programs beside exactly the files they read.

    python3 tools/kit.py --bin engine/target/release --out dist/kit/trinetra-adcs-<v>-<target>
    python3 tools/kit.py --files-only --out <dir>       the data alone (what the Python package carries)

A kit holds:
  adcs[.exe]                      the engine's command line
  TRI-NETRA ADCS[.exe]            the desktop app (when it was built)
  data/, cases/                   the scenarios, products, parts, algorithms, campaigns and cases
  design.tndb                     the design database, which holds the same cases and inputs: the
                                  app reads them from it (tools/design_inputs.py)
  pop/03_frames_time/ephemeris/data/de440s.bsp   the ephemeris the propagator reads
  VERSION                         the release and the version of each part; its presence is
                                  what tells the programs they run from a kit, so results go
                                  to ~/.trinetra/store and never into the kit
  pages/files.html, group.html, node.html  the offline pages (tools/pages.py): open them from disk in Chrome or Edge
  START_HERE.md, FIRST_RUN.md, COMMANDS.md, ENVIRONMENT.md
No git history, no generators, no build files. Zip the folder and share it.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import pathlib
import shutil
import sys
import tomllib

from common import ROOT, write_text

MS = ROOT / "matlab_sils"
PROGRAMS = ["adcs", "trinetra-app"]
# on Windows the desktop app carries its name, so a double-click on it is the whole of starting it
WINDOWS_NAME = {"trinetra-app": "TRI-NETRA ADCS"}


def version():
    return (ROOT / "VERSION").read_text().strip()


def components():
    eng = tomllib.loads((ROOT / "engine" / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    fsw = tomllib.loads((ROOT / "fsw-rs" / "Cargo.toml").read_text())["package"]["version"]
    return {"engine": eng, "flight software (Rust)": fsw, "flight software (C)": fsw_c(), "data": "adcs-case/1, adcs-scenario/1"}


def fsw_c():
    import re
    m = re.search(r'adcs_fsw_build_id\(void\) \{ return "trinetra-fsw-c/([^ "]+)', (ROOT / "fsw" / "src" / "adcs_fsw.c").read_text())
    return m.group(1) if m else "?"


def copy_data(out):
    """The files the engine reads at run time, and nothing else."""
    n = 0
    for sub in ("data", "cases"):
        for f in sorted((MS / sub).rglob("*")):
            if f.is_file() and not f.name.startswith("."):
                dst = out / sub / f.relative_to(MS / sub)
                dst.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(f, dst)
                n += 1
    bsp = pathlib.Path("pop/03_frames_time/ephemeris/data/de440s.bsp")
    (out / bsp).parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(MS / bsp, out / bsp)
    return n + 1


def design_db(out):
    """The design the repository is held to (tests/regression/design.tndb, today's design), beside the data:
    the app flies from it."""
    import from_design
    shutil.copy2(from_design.design(), out / "design.tndb")


def build(out, bin_dir=None, files_only=False):
    out = pathlib.Path(out)
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)
    progs = []
    if not files_only:
        exe = ".exe" if (bin_dir / "adcs.exe").exists() else ""
        for p in PROGRAMS:
            src = bin_dir / f"{p}{exe}"
            if not src.exists():   # a kit without one of its programs is not a kit
                sys.exit(f"{src} does not exist: build the engine and the app first (cargo build --release in engine/)")
            name = WINDOWS_NAME.get(p, p) if exe else p
            shutil.copy2(src, out / f"{name}{exe}")
            progs.append(f"{name}{exe}")
    n = copy_data(out)
    design_db(out)
    if not files_only:
        import pages
        built = pages.build(out / "pages")
        for name in ("testapp", "app"):     # the test apps' template (groupcode deliver fills it); the app's page is inside the app
            if name in built:
                built[name][0].unlink()
    v = version()
    write_text(out / "VERSION", f"TRI-NETRA ADCS {v}\n" + "".join(f"{k}: {x}\n" for k, x in components().items()))
    for doc, name in (("docs/START_HERE.md", "START_HERE.md"), ("docs/FIRST_RUN.md", "FIRST_RUN.md"), ("docs/COMMANDS.md", "COMMANDS.md"),
                      ("docs/ENVIRONMENT.md", "ENVIRONMENT.md")):
        if not (ROOT / doc).exists():
            sys.exit(f"{doc} does not exist, and every kit carries it")
        shutil.copy2(ROOT / doc, out / name)
    return v, progs, n


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bin", type=pathlib.Path, default=ROOT / "engine" / "target" / "release", help="where the release-built programs are")
    ap.add_argument("--out", type=pathlib.Path, help="the kit folder (default dist/kit/trinetra-adcs-<version>)")
    ap.add_argument("--files-only", action="store_true", help="the data without the programs (for the Python package)")
    a = ap.parse_args(argv)
    out = a.out or ROOT / "dist" / "kit" / f"trinetra-adcs-{version()}"
    v, progs, n = build(out, a.bin, a.files_only)
    print(f"kit: {out} -- TRI-NETRA ADCS {v}, {len(progs)} program(s) {', '.join(progs)}, {n} data files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
