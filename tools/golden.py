#!/usr/bin/env python3
"""The golden table: every scenario re-flown on the engine, compared with the stored results.

CI never re-flies the scenarios, so a gain change could merge green with every stored verdict stale. The
table (results/golden.json) holds, for each of the scenarios of matlab_sils/data/scenarios flown with the
C flight software and the stored seed, every metric's value and verdict, the algorithms the run names, and
a relative tolerance per metric (DR-vv-003).

  python3 tools/golden.py write [--jobs N] [--only scen ...]   re-fly and write results/golden.json
  python3 tools/golden.py check [--jobs N] [--only scen ...]   re-fly and compare; exit 1 on any difference

The engine is deterministic, so with the table's toolchain fingerprint (the rustc, the engine and flight
software build ids, the machine) a re-fly must match bit for bit. With another fingerprint (another compiler
or libm moves the last bits) each metric must agree within its relative tolerance. A verdict or the algorithms
named never get a tolerance. A difference names the scenario, the metric, the stored value and the new one.
`write --only` replaces just those scenarios' rows and keeps the tolerance column of every metric already in
the table, so a tolerance changed by hand in the table survives a re-write.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse, concurrent.futures as cf, json, math, os, platform, subprocess, sys, tempfile
from common import write_text
from engine_base import BIN, OUT
from engine_runs import one, scenarios

TABLE = OUT / "golden.json"
SCHEMA = "trinetra-golden/1"
CAMPAIGN_TOL = 0.05
CAMPAIGN_NOTE = ("DR-vv-003: a campaign run flown on another toolchain fingerprint passes when every verdict is equal and every "
                 "metric is within this relative tolerance; a Monte Carlo run of fine pointing is noise-dominated, so another "
                 "libm moves its metrics by up to percent; 5 % is the tolerance O19 uses for SILS against soft OILS "
                 "(docs/TEST_BENCH_PLAN.md section 5)")
SEED = 1
# the tolerance a metric starts with when the table is first written (afterwards the table's column rules):
# a time to a threshold moves by whole samples, a count by whole events, anything else by rounding
START_TOL = {"time_to_threshold": 1e-2, "count": 1e-2}
DEFAULT_TOL = 1e-6


def fingerprint(manifest):
    """What makes two runs of the same scenario bit-identical: the compiler, the engine and flight-software
    builds the run names, the machine. The engine's source hash is not in it: a changed source must be
    compared, not excused."""
    rustc = subprocess.run(["rustc", "-V"], capture_output=True, text=True).stdout.strip()
    return {"rustc": rustc, "engine": manifest["engine"], "fsw": manifest["fsw"]["build_id"], "machine": platform.machine()}


def fly(scen, folder):
    out = os.path.join(folder, scen)
    _, _, rc, _, txt = one((scen, "c", SEED, out, []))
    if rc:
        raise RuntimeError(f"{scen} did not fly:\n{txt}")
    with open(os.path.join(out, "manifest.json")) as f:
        m = json.load(f)
    return m


def fly_row(scen):
    """One scenario's row, from a run in a folder of its own."""
    with tempfile.TemporaryDirectory() as d:
        try:
            m = fly(scen, d)
        except RuntimeError as e:
            return scen, None, str(e)
    row = {"algorithms": m["algorithms"],
           "metrics": {x["id"]: {"kind": x["kind"], "value": x["value"], "pass": x["pass"]} for x in m["metrics"]}}
    return scen, row, {"engine": m["engine"], "fsw": m["fsw"]["build_id"]}


def same(a, b):
    """Bit-exact equality of two metric values (NaN equals NaN; -0.0 and 0.0 differ only if their bits do)."""
    if isinstance(a, float) and isinstance(b, float):
        return (math.isnan(a) and math.isnan(b)) or (a == b and math.copysign(1, a) == math.copysign(1, b))
    return a == b


def within(a, b, tol):
    """|a - b| <= tol * max(|a|, |b|), for numbers; anything else must be equal."""
    if not all(isinstance(x, (int, float)) and not isinstance(x, bool) for x in (a, b)):
        return same(a, b)
    if math.isnan(a) or math.isnan(b):
        return math.isnan(a) and math.isnan(b)
    return abs(a - b) <= tol * max(abs(a), abs(b))


def compare(stored, new, exact):
    """The differences between a stored scenario row and a re-flown one: (metric, stored, new, why) tuples.
    exact: bit for bit; else each metric within its own relative tolerance. A verdict and the algorithms
    named are always exact; a metric the table or the run lacks is a difference."""
    diffs = []
    if stored["algorithms"] != new["algorithms"]:
        diffs.append(("algorithms", stored["algorithms"], new["algorithms"], "the algorithms the run names"))
    for mid in sorted(set(stored["metrics"]) | set(new["metrics"])):
        s, n = stored["metrics"].get(mid), new["metrics"].get(mid)
        if s is None or n is None:
            diffs.append((mid, s and s["value"], n and n["value"], "metric missing from the " + ("table" if s is None else "run")))
            continue
        if s["pass"] != n["pass"]:
            diffs.append((mid, s["pass"], n["pass"], "verdict"))
        if exact:
            if not same(s["value"], n["value"]):
                diffs.append((mid, s["value"], n["value"], "not bit-identical"))
        elif not within(s["value"], n["value"], s["rel_tol"]):
            diffs.append((mid, s["value"], n["value"], f"outside relative tolerance {s['rel_tol']:g}"))
    return diffs


def fly_all(names, jobs):
    rows, ids, errs = {}, {}, []
    with cf.ProcessPoolExecutor(jobs) as ex:
        for scen, row, info in ex.map(fly_row, names):
            if row is None:
                errs.append(info)
                continue
            rows[scen] = row
            ids = info
    return rows, ids, errs


def load():
    with open(TABLE) as f:
        return json.load(f)


def pick(a):
    every = scenarios(None)
    unknown = set(a.only or []) - set(every)
    if unknown:
        sys.exit("golden.py: no scenario " + ", ".join(sorted(unknown)))
    return a.only or every


def write(a):
    names = pick(a)
    old = load() if TABLE.exists() else None
    rows, ids, errs = fly_all(names, a.jobs)
    for e in errs:
        print("FAIL", e)
    if errs:
        return len(errs)
    fp = fingerprint({"engine": ids["engine"], "fsw": {"build_id": ids["fsw"]}})
    keep = old["scenarios"] if old and a.only else {}
    if old and a.only and old["fingerprint"] != fp:
        sys.exit("golden.py write --only: the table was written with another fingerprint; write all the scenarios")
    for scen, row in rows.items():
        for mid, m in row["metrics"].items():
            was = (old or {}).get("scenarios", {}).get(scen, {}).get("metrics", {}).get(mid)
            m["rel_tol"] = was["rel_tol"] if was else START_TOL.get(m["kind"], DEFAULT_TOL)
    table = {"schema": SCHEMA, "decision": "DR-vv-003", "seed": SEED, "fsw": "c", "fingerprint": fp,
             "campaign_relative_tolerance": old["campaign_relative_tolerance"] if old else CAMPAIGN_TOL,
             "campaign_relative_tolerance_note": old["campaign_relative_tolerance_note"] if old else CAMPAIGN_NOTE,
             "scenarios": {k: v for k, v in sorted((keep | rows).items())}}
    write_text(TABLE, json.dumps(table, indent=1, allow_nan=True) + "\n")
    print(f"wrote {TABLE.relative_to(TABLE.parent.parent)}: {len(table['scenarios'])} scenario(s)")
    return 0


def check(a):
    names = pick(a)
    table = load()
    missing = [s for s in names if s not in table["scenarios"]]
    for s in missing:
        print(f"DIFF {s}: not in the table")
    rows, ids, errs = fly_all([s for s in names if s not in missing], a.jobs)
    for e in errs:
        print("FAIL", e)
    fp = fingerprint({"engine": ids["engine"], "fsw": {"build_id": ids["fsw"]}}) if ids else table["fingerprint"]
    exact = fp == table["fingerprint"]
    print("fingerprint " + ("equals the table's: bit-exact" if exact else
          "differs from the table's (" + ", ".join(k for k in fp if fp[k] != table["fingerprint"].get(k)) + "): per-metric relative tolerance"))
    bad = len(missing) + len(errs)
    for scen in sorted(rows):
        for mid, s, n, why in compare(table["scenarios"][scen], rows[scen], exact):
            bad += 1
            print(f"DIFF {scen}: {mid}: stored {s!r}, new {n!r} ({why})")
    print(f"golden check: {len(names)} scenario(s), " + (f"{bad} difference(s)" if bad else "no difference"))
    return 1 if bad else 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    for name, f in (("write", write), ("check", check)):
        p = sp.add_parser(name)
        p.add_argument("--jobs", type=int, default=os.cpu_count())
        p.add_argument("--only", nargs="+", metavar="scen")
        p.set_defaults(f=f)
    a = ap.parse_args(argv)
    if not BIN.exists():
        sys.exit("engine not built: cd engine && cargo build --release -p adcs-cli")
    import engine_fresh
    engine_fresh.exit_if_stale()
    return a.f(a)


if __name__ == "__main__":
    sys.exit(main())
