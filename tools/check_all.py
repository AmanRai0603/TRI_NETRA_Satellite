#!/usr/bin/env python3
"""Every check the repository has, in one command, with one line per check and a verdict.

    python3 tools/check_all.py                 the fast checks (a few minutes)
    python3 tools/check_all.py --octave        and the MATLAB twin's suites in GNU Octave
    python3 tools/check_all.py --only NAME...  just those checks (names as printed)

The checks: the Python tests (tests/), the generated files against their definitions, the
C flight software's checks, the Rust flight software's and the engine's tests, the platform
specification's package checks, and the node-by-node verification of the stored design loop.
A check that cannot run here (a missing compiler) is reported as not run, never as passed.
Exit status 1 when any check fails.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import shutil
import subprocess
import sys
import time

from common import ROOT

PY = sys.executable
CHECKS = [
    # name, what it proves, command, working folder, programs it needs
    ("python-tests", "the tools' own tests: the registry, generated files, the Kp -> ap table, atomic writes",
     [PY, "-m", "unittest", "discover", "-s", "tests", "-t", "tests"], ".", []),
    ("wheel", "the Python package builder: RECORD, entry points, executable bits, a changed byte caught",
     [PY, "tools/build_wheel.py", "--selftest"], ".", []),
    ("catalogue", "every JSON the engine and the twin read is its TOML",
     [PY, "tools/export_catalogue.py", "--check"], ".", []),
    ("fsw-params", "the C and Rust parameter tables are their definition",
     [PY, "tools/gen_fsw_params.py", "--check"], ".", []),
    ("commands-doc", "docs/COMMANDS.md is the registry",
     [PY, "tools/trinetra.py", "docs", "--check"], ".", []),
    ("fsw-c", "the C flight software builds clean and passes its checks",
     ["make", "-s", "test"], "fsw", ["make", "gcc"]),
    ("fsw-rs", "the Rust flight software's tests",
     ["cargo", "test", "--release", "-q"], "fsw-rs", ["cargo"]),
    ("engine", "the engine's tests: inputs refused by name, results store, determinism, C = Rust",
     ["cargo", "test", "--release", "-q"], "engine", ["cargo"]),
    ("spec", "the platform specification package is consistent",
     ["bash", "-c", "python3 tools/validate_plan.py && python3 tools/build_tree.py --check && python3 tools/intake.py selftest"
      " && python3 tools/derisk.py check && python3 tools/twin_check.py && bash tools/assemble_spec.sh --check"], "spec", ["bash"]),
    ("design-loop", "every stored design-loop decision recomputed from its inputs",
     [PY, "tools/verify_nodes.py"], ".", []),
]
OCTAVE = [
    ("twin", "the MATLAB twin's test suite (GNU Octave)",
     ["octave-cli", "--no-gui", "-q", "--eval", "startup_asils; addpath tests; run_all_tests"], "matlab_sils", ["octave-cli"]),
    ("propagator", "the propagator's regression suite (GNU Octave)",
     ["octave-cli", "--no-gui", "-q", "--eval", "setup_paths; addpath('08_test'); run_all_tests"], "matlab_sils/pop", ["octave-cli"]),
]


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--octave", action="store_true", help="also run the MATLAB twin's suites in GNU Octave")
    ap.add_argument("--only", nargs="+", metavar="NAME", help="run only these checks")
    a = ap.parse_args(argv)
    todo = CHECKS + (OCTAVE if a.octave or (a.only and set(a.only) & {n for n, *_ in OCTAVE}) else [])
    if a.only:
        unknown = set(a.only) - {c[0] for c in todo}
        if unknown:
            ap.error("no check " + ", ".join(sorted(unknown)) + "; the checks are " + ", ".join(c[0] for c in CHECKS + OCTAVE))
        todo = [c for c in todo if c[0] in a.only]
    rows, failed = [], []
    for name, what, cmd, cwd, needs in todo:
        missing = [n for n in needs if not shutil.which(n)]
        if missing:
            rows.append((name, "NOT RUN", f"needs {', '.join(missing)}", 0.0))
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
