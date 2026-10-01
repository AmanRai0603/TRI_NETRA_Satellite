#!/usr/bin/env python3
"""Mutation testing of the flight software's control and estimation: how many deliberate small
faults (a + made -, a > made >=, a function made to return a default) the Rust flight software's
own tests catch.

cargo-mutants makes each mutant of fsw-rs/src/ctl.rs and fsw-rs/src/est.rs in turn and runs
`cargo test --release` in fsw-rs; a mutant the tests still pass is "missed". The kill rate is
caught / (caught + missed) (a mutant that hangs the tests counts as caught). The C build is the
same algorithms; C = Rust bit for bit is checked elsewhere (the differential fuzzer, the 40
scenarios), so a fault the Rust tests miss is one neither build's unit tests would see.

    python3 tools/mutation.py              run cargo-mutants (about 30 min), write results/MUTATION.md
    python3 tools/mutation.py --from DIR   summarise an existing cargo-mutants output folder
    python3 tools/mutation.py --check      run it and fail when the kill rate is under FLOOR

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import collections
import json
import pathlib
import subprocess
import sys
import tempfile

from common import ROOT, write_json, write_text

FILES = ["src/ctl.rs", "src/est.rs"]
FLOOR = 0.55     # the kill rate this repository holds itself to; raised as tests are added


def run(out, jobs):
    cmd = ["cargo", "mutants", "--jobs", str(jobs), "--timeout", "120", "--output", str(out)]
    for f in FILES:
        cmd += ["--file", f]
    print("$", " ".join(cmd), "-- --release", flush=True)
    subprocess.run(cmd + ["--", "--release"], cwd=ROOT / "fsw-rs", env={**__import__("os").environ, "ADCS_FUZZ_N": "300"})


def summarise(out):
    """Per file and per function: caught, missed, timeout, unviable; and the missed mutants' names."""
    d = json.loads((pathlib.Path(out) / "mutants.out" / "outcomes.json").read_text())
    rows = collections.defaultdict(collections.Counter)
    missed = []
    for o in d["outcomes"]:
        m = o["scenario"].get("Mutant") if isinstance(o["scenario"], dict) else None
        if not m:
            continue
        kind = {"CaughtMutant": "caught", "MissedMutant": "missed", "Timeout": "timeout", "Unviable": "unviable"}.get(o["summary"], o["summary"])
        key = (m["file"], (m.get("function") or {}).get("function_name", "?"))
        rows[key][kind] += 1
        if kind == "missed":
            missed.append(m["name"])
    tot = collections.Counter()
    for c in rows.values():
        tot.update(c)
    killed = tot["caught"] + tot["timeout"]
    rate = killed / max(1, killed + tot["missed"])
    return {"version": d.get("cargo_mutants_version"), "files": FILES, "total": tot, "rate": rate,
            "functions": [{"file": f, "function": fn, **c} for (f, fn), c in sorted(rows.items())], "missed": sorted(missed)}


def markdown(s):
    t = s["total"]
    L = ["# Mutation testing: the flight software's control and estimation", "",
         f"> **Answer first.** {t['caught'] + t['timeout']} of {t['caught'] + t['timeout'] + t['missed']} mutants caught: "
         f"a kill rate of **{100 * s['rate']:.1f} %** (floor {100 * FLOOR:.0f} %). {t['unviable']} did not compile.", "",
         f"> **Kind:** generated (`python3 tools/mutation.py`, cargo-mutants {s['version']}) · **Files:** {', '.join('fsw-rs/' + f for f in s['files'])}", "",
         "A mutant is the code with one small fault put in on purpose; it is *caught* when a test fails.",
         "A missed mutant is a fault the unit tests would let through: either a test is owed, or the",
         "change makes no observable difference (an equivalent mutant).", "",
         "| file | function | caught | missed | kill rate |", "|---|---|---:|---:|---:|"]
    for r in sorted(s["functions"], key=lambda r: (-r.get("missed", 0), r["file"], r["function"])):
        k, m = r.get("caught", 0) + r.get("timeout", 0), r.get("missed", 0)
        L.append(f"| {r['file']} | `{r['function']}` | {k} | {m} | {100 * k / max(1, k + m):.0f} % |")
    L += ["", "## Missed mutants", ""] + [f"- `{m}`" for m in s["missed"]]
    return "\n".join(L) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--from", dest="src", metavar="DIR", help="an existing cargo-mutants --output folder")
    ap.add_argument("--check", action="store_true", help=f"fail when the kill rate is under {FLOOR:.0%}")
    ap.add_argument("--jobs", type=int, default=3)
    a = ap.parse_args(argv)
    if a.src:
        s = summarise(a.src)
    else:
        with tempfile.TemporaryDirectory() as tmp:
            run(tmp, a.jobs)
            s = summarise(tmp)
    write_text(ROOT / "results" / "MUTATION.md", markdown(s))
    write_json(ROOT / "results" / "mutation.json", {k: v for k, v in s.items()}, indent=1)
    print(f"mutation: kill rate {100 * s['rate']:.1f} % ({s['total']['missed']} missed), floor {100 * FLOOR:.0f} %")
    return 1 if a.check and s["rate"] < FLOOR else 0


if __name__ == "__main__":
    sys.exit(main())
