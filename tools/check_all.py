#!/usr/bin/env python3
"""Every check the repository has, in one command, with one line per check and a verdict.

    python3 tools/check_all.py                 the fast checks (a few minutes)
    python3 tools/check_all.py --octave        and the MATLAB twin's suites in GNU Octave
    python3 tools/check_all.py --pages         and rebuild the rendered pages (needs the runs' time series)
    python3 tools/check_all.py --mutation      and mutation testing of the flight software (about 30 min)
    python3 tools/check_all.py --only NAME...  just those checks (names as printed)
    python3 tools/check_all.py --strict        a check that cannot run here fails (the release runs this)

The checks: the Python tests (tests/), the generated files against their definitions, the
C flight software's checks, the Rust flight software's and the engine's tests, the platform
specification's package checks, and the node-by-node verification of the stored design loop.
A check that cannot run here (a missing compiler) is reported as not run, never as passed; with
--strict it fails.
Exit status 1 when any check fails.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import glob
import importlib.util
import shutil
import subprocess
import sys
import time

from common import ROOT

PY = sys.executable
CHECKS = [
    # name, what it proves, command, working folder, programs it needs (py:<module> for a Python module)
    ("lint", "no unused name, undefined name or dead import in the tools and tests (pyflakes)",
     [PY, "-m", "pyflakes", "tools", "tests"], ".", ["py:pyflakes"]),
    ("version", "VERSION is the version of every part: the engine, both flight softwares, their build ids",
     [PY, "tools/version.py", "--check"], ".", []),
    ("python-tests", "the tools' own tests: the registry, generated files, the Kp -> ap table, atomic writes",
     [PY, "-m", "unittest", "discover", "-s", "tests", "-t", "tests"], ".", []),
    ("wheel", "the Python package builder: RECORD, entry points, executable bits, a changed byte caught",
     [PY, "tools/build_wheel.py", "--selftest"], ".", []),
    ("from-design", "every file the code reads that is design (the engine's inputs and cases, the flight parameter table, the flight algorithms) is what the regression copy gives, and nothing else sits in the generated folders",
     [PY, "tools/from_design.py", "--check"], ".", []),
    ("boundary", "no relation or published model in code outside the generated files and the toolbox: every source file generated (and saying so) or classified in tools/boundary.toml with no class for design, every classified file searched for published constants, gravity and drag laws and coefficient tables, each find allowed by its reason, every relation still in code a declared exception printed with what removes it (docs/PLAN_2_0.md S7)",
     [PY, "tools/boundary.py"], ".", []),
    ("fsw-params", "the C and Rust parameter tables are their definition",
     [PY, "tools/gen_fsw_params.py", "--check"], ".", []),
    ("commands-doc", "docs/COMMANDS.md is the registry",
     [PY, "tools/trinetra.py", "docs", "--check"], ".", []),
    ("flight-build", "the flight software's generated algorithm sources (C and Rust) are what the design gives",
     [PY, "tools/flight_build.py", "gen", "--check"], ".", ["node"]),
    ("engine-build", "the time engine's published models and relations (adcs-sim-core, adcs-pop, adcs-sim, adcs-design) and the twin's copy (+asils/+models), and the relations crate (adcs-relations: the design's relations and every group's computing rows, each once, with its WebAssembly face, its MATLAB package +asils/+relations and its vectors) are what the design gives (the groups' methods and tables, tools/engine_build.py), and nothing else sits in their generated folders",
     [PY, "tools/engine_build.py", "gen", "--check"], ".", ["node"]),
    ("flight-image", "the C flight software's build seals as a flight image from the design (its sources the design's, its flight flags, its vectors, its build id naming the algorithms; every scenario's blob) and the sealed file verifies; a change in the design reaching the sources, the library and the metrics is tests/test_flight_build.py",
     ["bash", "-c", "rm -rf build/flight_images && python3 tools/flight_build.py seal --target posix --out build/flight_images "
      "&& python3 tools/flight_build.py verify build/flight_images/design-*.posix.tnfsw"], ".", ["make", "gcc", "bash", "engine"]),
    ("fsw-c", "the C flight software builds clean with every warning an error, calls nothing forbidden (malloc, time, rand), and passes its checks",
     ["make", "-s", "check", "test"], "fsw", ["make", "gcc"]),
    ("trace", "every shipped metric judges or says why it only reports; every stated requirement is checked",
     [PY, "tools/trace.py", "--check"], ".", []),
    ("fsw-stack", "the flight software's deepest stack fits the stack the OBC firmware reserves",
     [PY, "tools/fsw_stack.py"], ".", ["arm-none-eabi-gcc"]),
    ("fsw-rs", "the Rust flight software's tests",
     ["cargo", "test", "--locked", "--release", "-q"], "fsw-rs", ["cargo"]),
    ("engine", "the engine's tests: inputs refused by name, results store, determinism, C = Rust",
     ["cargo", "test", "--locked", "--release", "-q"], "engine", ["cargo"]),
    ("design-files", "every row of the tree in exactly one group; the seeded group, node and design files check against design/schema.toml; the generated SQL and browser schema are the schema; the apps' manual is current, every page in the explanation standard's shape, every tour target in its app, every field with help",
     [PY, "-c", "import sys; sys.path.insert(0, 'tools'); import groups, manual, node_catalog, seed_design, tndb; "
      "sys.exit(groups.main(['--check']) or seed_design.main(['--check']) or node_catalog.main(['--check']) or manual.main(['--check']) or tndb.main(['gen', '--check']))"], ".", []),
    ("pseudocode", "everything the language makes of itself is current (its self-test in Rust and MATLAB and its vectors, the flight algorithms' vectors, the MATLAB runtime, the checker page), the physics is its registry, and every sourced fixture of a physics row holds",
     ["bash", "-c", "python3 tools/pcode.py gen --check && python3 tools/pcode.py fixtures"], ".", ["node", "bash"]),
    ("offline-pages", "the offline pages build with their vendored files pinned, one component set and no outside hosts; TRI-NETRA Files passes its browser tests (save, reopen, crash, second editor, conflict copies, caps), and TRI-NETRA Group and TRI-NETRA Node theirs (the node app: every node kind filled and previewed), when a browser is here; the node app's checks rule by rule; a newcomer's first open (tours, Help, printing, help on every field) and a walkthrough from a node to a sealed release by what the screens say",
     [PY, "-m", "unittest", "discover", "-s", "tests", "-p", "test_pages.py"], ".", []),
    ("structure", "the group app's structure actions on the whole seeded design (all 20 groups; act and catalogue restructured; a node moved between groups) leave every rule kept, by structure.js and by tools/group.py; both checkers find the same breakage",
     [PY, "-m", "unittest", "discover", "-s", "tests", "-p", "test_structure.py"], ".", ["node"]),
    ("release", "the group app's release side on the whole seeded design (all 20 groups sealed 1.0, then a node of each re-issued and changed and each sealed 1.1; a damaged node file made again from a release; a node form imported) and through the page; every release held by tools/release.py, which finds each one broken on purpose",
     [PY, "-m", "unittest", "discover", "-s", "tests", "-p", "test_release.py"], ".", ["node"]),
    ("carry", "what the repository already says carried into the node files (seed content, physics as pseudocode, case keys, KPIs, algorithms, the tree's notes; the internal rows named and the rows added from the code), each item with its origin and every gap with its owner team; every rule holds afterwards, the carried pseudocode compiles and reproduces its test vectors in the node app's own code, and carrying again changes nothing",
     [PY, "-m", "unittest", "discover", "-s", "tests", "-p", "test_carry.py"], ".", []),
    ("groupcode", "each group's code generated from its nodes: the wiring is the design's, the relations crate's Rust, WebAssembly and MATLAB are current with each relation once, every computing row with pseudocode is in the generated code, the Rust reproduces the interpreter on both packages' vectors and every node's own test vectors, and every group's test app passes in the browser (interpreter and WebAssembly agreeing)",
     [PY, "-m", "unittest", "discover", "-s", "tests", "-p", "test_groupcode.py"], ".", ["node", "cargo"]),
    ("translators", "the pseudocode's Rust and C translations of every package (the relations, the language's self-test, the flight software, each group's code) equal the interpreter on every vector: bit for bit where exact, within 1e-12 otherwise",
     [PY, "tools/translators.py", "--lang", "rust", "c", "--out", "build/translators"], ".", ["node", "rustc", "gcc"]),
    ("design-loop", "every stored design-loop decision recomputed from its inputs",
     [PY, "tools/verify_nodes.py"], ".", []),
]
# The rendered pages (figures, the results page, the V&V report HTML and PDF), rebuilt from the
# runs' time series (channels.csv), which git does not keep, so this runs where the runs were flown
# and says NOT RUN elsewhere (CI, a fresh clone). Only the V&V report is committed; the results page
# and its figures stay local (a run's figures and report: adcs figures, adcs report, the app).
PAGES = [
    ("pages", "the figures, results/index.html and the V&V report (HTML and PDF) rebuild from the stored runs",
     ["bash", "-c", "cargo build --release -q --manifest-path engine/Cargo.toml -p adcs-cli && python3 tools/report.py && python3 tools/vv_report.py"], ".",
     ["py:numpy", "cargo", "browser", "time series"]),
]

# The two suites report a failure in what they return (ok) or leave (nfail); Octave's own exit
# status is 0 either way, so each command turns that into its exit status.
OCTAVE = [
    ("twin", "the MATLAB twin's test suite (GNU Octave)",
     ["octave-cli", "--no-gui", "-q", "--eval", "startup_asils; addpath tests; ok = run_all_tests(); exit(double(~ok))"], "matlab_sils", ["octave-cli"]),
    ("translators-matlab", "the pseudocode's MATLAB translation of every package equals the interpreter on every vector (GNU Octave)",
     [PY, "tools/translators.py", "--lang", "matlab", "--out", "build/translators"], ".", ["node", "octave-cli"]),
    ("propagator", "the propagator's regression suite (GNU Octave)",
     ["octave-cli", "--no-gui", "-q", "--eval", "setup_paths; addpath('08_test'); run_all_tests; exit(double(nfail > 0))"], "matlab_sils/pop", ["octave-cli", "gnuplot"]),
]

# Mutation testing takes about half an hour, so it runs when asked (and weekly in CI).
MUTATION = [
    ("mutation", "the flight software's unit tests catch deliberate faults in control and estimation (cargo-mutants)",
     [PY, "tools/mutation.py", "--check"], ".", ["cargo-mutants"]),
]


def have(need):
    """Is a check's need here: a program on PATH, a Python module (py:<name>), the built engine (engine), or a
    browser that prints PDFs (the one vv_report.py looks for)?"""
    if need.startswith("py:"):
        return importlib.util.find_spec(need[3:]) is not None
    if need == "time series":
        return any((ROOT / "matlab_sils" / "store" / "results").glob("*/channels.csv"))
    if need == "engine":
        return (ROOT / "engine" / "target" / "release" / "adcs").is_file()
    if need == "browser":
        return bool(glob.glob("/opt/pw-browsers/chromium*/chrome-linux/chrome") or shutil.which("chromium") or shutil.which("google-chrome"))
    return shutil.which(need) is not None


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--octave", action="store_true", help="also run the MATLAB twin's suites in GNU Octave")
    ap.add_argument("--pages", action="store_true", help="also build the rendered pages (figures, results page, V&V report)")
    ap.add_argument("--mutation", action="store_true", help="also run mutation testing of the flight software (about 30 min)")
    ap.add_argument("--only", nargs="+", metavar="NAME", help="run only these checks")
    ap.add_argument("--strict", action="store_true",
                    help="a check that cannot run here (NOT RUN) fails: what the release runs, so it proves what CI proves")
    a = ap.parse_args(argv)
    wants = lambda group, flag: flag or bool(a.only and set(a.only) & {n for n, *_ in group})
    todo = CHECKS + (OCTAVE if wants(OCTAVE, a.octave) else []) + (PAGES if wants(PAGES, a.pages) else []) \
        + (MUTATION if wants(MUTATION, a.mutation) else [])
    if a.only:
        unknown = set(a.only) - {c[0] for c in todo}
        if unknown:
            ap.error("no check " + ", ".join(sorted(unknown)) + "; the checks are " + ", ".join(c[0] for c in CHECKS + OCTAVE + PAGES + MUTATION))
        todo = [c for c in todo if c[0] in a.only]
    rows, failed = [], []
    for name, what, cmd, cwd, needs in todo:
        missing = [n for n in needs if not have(n)]
        if missing:
            rows.append((name, "NOT RUN", f"needs {', '.join(missing)}", 0.0))
            if a.strict:
                failed.append(name)
            continue
        print(f"== {name}: {what}", flush=True)
        t0 = time.time()
        p = subprocess.run(cmd, cwd=ROOT / cwd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        dt = time.time() - t0
        tail = [l for l in p.stdout.strip().splitlines() if l.strip()][-1:] or [""]
        if p.returncode:
            failed.append(name)
            print(p.stdout[-4000:], flush=True)
        rows.append((name, "ok" if p.returncode == 0 else "FAILED", tail[0][:90], dt))
    print("\n" + "\n".join(f"{n:<13} {v:<8} {t:6.0f} s  {last}" for n, v, last, t in rows))
    print(f"\ncheck_all: {len(rows) - len(failed)}/{len(rows)} ok" + (f"; FAILED: {', '.join(failed)}" if failed else ""))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
