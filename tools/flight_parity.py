#!/usr/bin/env python3
"""Flight parity (docs/PLAN_2_0.md S6): the flight software built from the design gives the same answers
as the software it replaces. It compares stored results, flown before and after, and flies nothing:

    python3 tools/flight_parity.py BASELINE [--label NAME] [--pair LABEL BASE CURRENT ...] [--why TEXT ...] [--no-write]

BASELINE and every BASE or CURRENT folder is laid out as the repository is (the tree itself is the default
CURRENT): matlab_sils/store/results_engine/campaigns/<id>/summary.json, as `engine.py campaign` writes it, and
matlab_sils/store/results_engine/soft_oils/<scenario>/{sils,oils}/manifest.json, as `engine.py oils` writes
them. Copy them aside before flying again; `--pair` adds a comparison of two other folders (the Rust flight
software's campaigns, say, or a firmware flown before and after).

What it holds:
  campaigns   every run's draws, metric values and verdicts, every statistic and every ECSS interpretation:
              identical (none further apart than 1e-12 relative), the same verdicts, the same runs and failures
  soft OILS   SILS identical as above; on the firmware, no overrun, a worst-case deadline margin that is not
              negative, the same verdicts as before; the instruction counts (mean, max), the minimum deadline
              margin and the metric differences are tabulated, not judged (an algorithm written another way
              runs another number of instructions, so the command lands at another time)
What it writes: results/FLIGHT_PARITY.md and results/flight_parity.json. Exit 1 when anything regressed,
each named.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import math
import pathlib
import sys

from common import ROOT, write_json, write_text

ENG = pathlib.Path("matlab_sils") / "store" / "results_engine"
TOL = 1e-12          # relative: what "identical" allows (none expected: the same bits)
SKIP = {"wall_s", "created_utc", "fsw", "result_id", "engine_source", "engine_version", "inputs", "product_files"}


def rel(a, b):
    """Relative difference of two numbers (0 when equal, inf when one side is missing)."""
    if a == b or (isinstance(a, float) and isinstance(b, float) and math.isnan(a) and math.isnan(b)):
        return 0.0
    if not isinstance(a, (int, float)) or not isinstance(b, (int, float)) or isinstance(a, bool) or isinstance(b, bool):
        return math.inf
    return abs(a - b) / max(abs(a), abs(b))


def leaves(x, path=""):
    """Every scalar of a JSON value by its path; a list of records is keyed by each record's id (or k)."""
    if isinstance(x, dict):
        for k, v in x.items():
            if k not in SKIP:
                yield from leaves(v, f"{path}.{k}" if path else k)
    elif isinstance(x, list):
        for i, v in enumerate(x):
            key = v.get("id", v.get("k", i)) if isinstance(v, dict) else i
            yield from leaves(v, f"{path}[{key}]")
    else:
        yield path, x


def diff(a, b):
    """(compared, differing, max relative difference, worst path) between two JSON values."""
    A, B = dict(leaves(a)), dict(leaves(b))
    n, bad, worst, where = 0, 0, 0.0, None
    for p in sorted(set(A) | set(B)):
        r = rel(A.get(p), B.get(p)) if p in A and p in B else math.inf
        n += 1
        if r > TOL:
            bad += 1
        if r > worst or (where is None and r > 0):
            worst, where = r, p
    return n, bad, (None if math.isinf(worst) else worst), where


def f3(x):
    return "a missing value" if x is None else f"{x:.3g}"


def verdicts(metrics):
    return {m["id"]: m.get("pass") for m in metrics if m.get("pass") is not None}


def load(p):
    return json.loads(p.read_text()) if p.exists() else None


def campaigns(label, base, cur):
    rows, faults = [], []
    if not any((cur / ENG / "campaigns").glob("*/summary.json")):
        return rows, faults           # this set holds no campaigns: nothing to compare
    ids = sorted({p.parent.name for d in (base, cur) for p in (d / ENG / "campaigns").glob("*/summary.json")})
    for cid in ids:
        A, B = load(base / ENG / "campaigns" / cid / "summary.json"), load(cur / ENG / "campaigns" / cid / "summary.json")
        if A is None or B is None:
            faults.append(f"{label} campaign {cid}: only in the {'current' if A is None else 'baseline'} results")
            continue
        n, bad, worst, where = diff(A, B)
        va = {(r["k"], i): p for r in A["per_run"] for i, p in verdicts(r["metrics"]).items()}
        vb = {(r["k"], i): p for r in B["per_run"] for i, p in verdicts(r["metrics"]).items()}
        changed = sorted(k for k in set(va) | set(vb) if va.get(k) != vb.get(k))
        row = {"set": label, "campaign": cid, "scenario": B["scenario"], "fsw": [A.get("fsw"), B.get("fsw")],
               "runs": [A["runs"], B["runs"]], "failed_runs": [A["failed_runs"], B["failed_runs"]],
               "values": n, "differ": bad, "max_rel": worst, "worst": where,
               "verdicts": len(vb), "verdicts_changed": [f"run {k} {i}" for k, i in changed]}
        rows.append(row)
        if bad:
            faults.append(f"{label} campaign {cid}: {bad} of {n} values differ, worst {where} ({f3(worst)} relative)")
        if changed:
            faults.append(f"{label} campaign {cid}: {len(changed)} verdict(s) changed: {row['verdicts_changed'][:5]}")
        if B["failed_runs"] or B["runs"] != A["runs"]:
            faults.append(f"{label} campaign {cid}: {B['runs']} runs, {B['failed_runs']} failed (before {A['runs']}, {A['failed_runs']})")
    return rows, faults


def word(p):
    return {1: "pass", 0: "FAIL"}.get(p, "—")


def soft_oils(label, base, cur, notes):
    rows, faults = [], []
    if not any((cur / ENG / "soft_oils").glob("*/*/manifest.json")):
        return rows, faults           # this set holds no soft-OILS runs: nothing to compare
    names = sorted({p.parent.parent.name for d in (base, cur) for p in (d / ENG / "soft_oils").glob("*/*/manifest.json")})
    for s in names:
        row = {"set": label, "scenario": s}
        for mode in ("sils", "oils"):
            A, B = (load(d / ENG / "soft_oils" / s / mode / "manifest.json") for d in (base, cur))
            if B is None:
                row[mode] = None
                if A is not None:
                    faults.append(f"{label} soft OILS {s} {mode}: only in the baseline results")
                continue
            if A is None:      # a run the baseline does not hold (it was not flown, or failed, before): shown, judged alone
                notes.append(f"{label} soft OILS {s} {mode}: not in the baseline; judged on its own deadlines only")
                r = {"impl": [None, B["fsw"].get("impl")], "metrics": len(B["metrics"]), "differ": None, "max_rel": None,
                     "worst": None, "verdicts": len(verdicts(B["metrics"])), "verdicts_changed": [], "new": True}
            else:
                n, bad, worst, where = diff(A["metrics"], B["metrics"])
                va, vb = verdicts(A["metrics"]), verdicts(B["metrics"])
                changed = sorted(i for i in set(va) | set(vb) if va.get(i) != vb.get(i))
                r = {"impl": [A["fsw"].get("impl"), B["fsw"].get("impl")], "metrics": n, "differ": bad, "max_rel": worst, "worst": where,
                     "verdicts": len(vb), "verdicts_changed": [f"{i} ({word(va.get(i))} -> {word(vb.get(i))})" for i in changed]}
                if changed:
                    faults.append(f"{label} soft OILS {s} {mode}: verdict(s) changed: {r['verdicts_changed']}")
                if mode == "sils" and bad:
                    faults.append(f"{label} soft OILS {s} sils: {bad} of {n} metrics differ, worst {where} ({f3(worst)} relative)")
            if mode == "oils":
                oa, ob = (A or {}).get("oils") or {}, B.get("oils") or {}
                ia, ib = oa.get("instructions") or {}, ob.get("instructions") or {}
                pct = lambda k: None if not ia.get(k) or ib.get(k) is None else 100.0 * (ib[k] - ia[k]) / ia[k]
                r |= {"ticks": [oa.get("ticks"), ob.get("ticks")], "overruns": [oa.get("overruns"), ob.get("overruns")],
                      "insn_mean": [ia.get("mean"), ib.get("mean")], "insn_max": [ia.get("max"), ib.get("max")],
                      "insn_mean_pct": pct("mean"), "insn_max_pct": pct("max"),
                      "cpu_load_max": [oa.get("cpu_load_max"), ob.get("cpu_load_max")],
                      "deadline_margin_min_s": [oa.get("deadline_margin_min_s"), ob.get("deadline_margin_min_s")],
                      "worst_case_margin_s": [oa.get("worst_case_margin_s"), ob.get("worst_case_margin_s")]}
                if ob.get("overruns"):
                    faults.append(f"{label} soft OILS {s}: {ob['overruns']} overrun(s)")
                if ob.get("worst_case_margin_s") is not None and ob["worst_case_margin_s"] < 0:
                    faults.append(f"{label} soft OILS {s}: worst-case deadline margin {1e3 * ob['worst_case_margin_s']:.3g} ms")
            row[mode] = r
        rows.append(row)
    return rows, faults


def shown(d):
    """A folder as the ledger names it: in the repository, its path there; elsewhere (a copy kept aside), its last two parts."""
    p = pathlib.Path(d).resolve()
    if p == ROOT:
        return "the tree"
    if ROOT in p.parents:
        return p.relative_to(ROOT).as_posix()
    return ".../" + "/".join(p.parts[-2:])


def ledger(pairs, camp, oils, faults, notes=(), why=()):
    f = lambda x: "—" if x is None else (f"{x:.4g}" if isinstance(x, float) else str(x))
    pc = lambda x: "—" if x is None else f"{x:+.2f} %"
    ms = lambda x: "—" if x is None else f"{1e3 * x:.4g}"
    L = ["# Flight parity: the flight software from the design against the hand-written software", "",
         "Owner: Agastya. `tools/flight_parity.py` compares the stored results of the flight software built from the design",
         "(docs/PLAN_2_0.md S6: the algorithms written by `tools/flight_build.py`) with the results of the hand-written",
         "software it replaces, flown with the same commands (`engine.py campaign`, `engine.py oils`). It flies nothing.",
         f"Identical means no value further apart than {TOL:g} relative; on the soft-OILS firmware the deadlines and verdicts",
         "are judged, and the instruction counts and metric differences are shown.", "",
         "| set | baseline | current |", "|---|---|---|"]
    L += [f"| {lab} | `{shown(b)}` | `{shown(c)}` |" for lab, b, c in pairs]
    L += ["", f"**{'No regression.' if not faults else f'{len(faults)} regression(s):'}**", ""]
    L += [f"- {x}" for x in faults]
    if notes:
        L += ["", "Not judged against a baseline:", ""] + [f"- {x}" for x in notes]
    if why:
        L += ["", "## What the run found", ""] + [f"- {x}" for x in why]
    if camp:
        L += ["", "## SILS campaigns", "", "| set | campaign | scenario | fsw before / now | runs | values compared | differ | max relative | verdicts | changed |",
              "|---|---|---|---|---:|---:|---:|---:|---:|---:|"]
        for r in camp:
            L.append(f"| {r['set']} | {r['campaign']} | {r['scenario']} | {r['fsw'][0]} / {r['fsw'][1]} | {r['runs'][1]} | {r['values']} | {r['differ']} | "
                     f"{f(r['max_rel'])} | {r['verdicts']} | {len(r['verdicts_changed'])} |")
    if oils:
        L += ["", "## Soft OILS: the deadline and the timing", "",
              "| set | scenario | ticks | overruns | instructions mean before / now | change | max before / now | change | "
              "deadline margin min [ms] before / now | worst-case margin [ms] before / now |", "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
        for r in oils:
            o = r.get("oils")
            if not o:
                L.append(f"| {r['set']} | {r['scenario']} | — | — | — | — | — | — | — | — |")
                continue
            L.append(f"| {r['set']} | {r['scenario']} | {o['ticks'][1]} | {o['overruns'][1]} | {f(o['insn_mean'][0])} / {f(o['insn_mean'][1])} | {pc(o['insn_mean_pct'])} | "
                     f"{f(o['insn_max'][0])} / {f(o['insn_max'][1])} | {pc(o['insn_max_pct'])} | "
                     f"{ms(o['deadline_margin_min_s'][0])} / {ms(o['deadline_margin_min_s'][1])} | {ms(o['worst_case_margin_s'][0])} / {ms(o['worst_case_margin_s'][1])} |")
        L += ["", "## Soft OILS: the metrics", "", "| set | scenario | SILS differ / compared | SILS max relative | OILS differ / compared | OILS max relative | OILS worst metric | verdicts changed |",
              "|---|---|---:|---:|---:|---:|---|---|"]
        for r in oils:
            s, o = r.get("sils") or {}, r.get("oils") or {}
            ch = [f"sils {i}" for i in s.get("verdicts_changed", [])] + [f"oils {i}" for i in o.get("verdicts_changed", [])]
            L.append(f"| {r['set']} | {r['scenario']} | {f(s.get('differ'))} / {f(s.get('metrics'))} | {f(s.get('max_rel'))} | "
                     f"{f(o.get('differ'))} / {f(o.get('metrics'))} | {f(o.get('max_rel'))} | {o.get('worst') or '—'} | {', '.join(ch) or 'none'} |")
        for lab in dict.fromkeys(r["set"] for r in oils):
            tim = [r["oils"] for r in oils if r.get("oils") and r["set"] == lab]
            if not tim:
                continue
            vals = lambda k: [t[k] for t in tim if t[k] is not None]
            span = lambda k: f"{min(vals(k)):+.2f} % to {max(vals(k)):+.2f} %" if vals(k) else "—"
            wcm = [t["worst_case_margin_s"][1] for t in tim if t["worst_case_margin_s"][1] is not None]
            L += ["", f"Set {lab}, over its {len(tim)} scenarios: instructions mean change {span('insn_mean_pct')}, max change {span('insn_max_pct')}; "
                  f"overruns {sum(t['overruns'][1] or 0 for t in tim)}; minimum worst-case margin {ms(min(wcm)) if wcm else '—'} ms."]
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("baseline", nargs="?", help="the results before, laid out as the repository (compared with the tree)")
    ap.add_argument("--label", default="tree", help="the name of the BASELINE-to-tree comparison in the ledger")
    ap.add_argument("--pair", nargs=3, action="append", default=[], metavar=("LABEL", "BASE", "CURRENT"),
                    help="also compare CURRENT with BASE, both laid out as the repository")
    ap.add_argument("--why", action="append", default=[], metavar="TEXT",
                    help="a finding of the run to keep in the ledger (the cause of a difference, say); repeatable")
    ap.add_argument("--no-write", action="store_true", help="print the regressions and write no ledger")
    a = ap.parse_args()
    pairs = ([(a.label, a.baseline, str(ROOT))] if a.baseline else []) + [tuple(p) for p in a.pair]
    if not pairs:
        ap.error("give a baseline folder or a --pair")
    camp, oils, faults, notes = [], [], [], []
    for lab, b, c in pairs:
        base, cur = pathlib.Path(b).resolve(), (ROOT if c == "." else pathlib.Path(c).resolve())
        for d in (base, cur):
            if not (d / ENG).is_dir():
                raise SystemExit(f"{d}: no {ENG} in it (a results folder is laid out as the repository)")
        r, x = campaigns(lab, base, cur)
        camp += r
        faults += x
        r, x = soft_oils(lab, base, cur, notes)
        oils += r
        faults += x
    for x in faults:
        print("REGRESSION:", x)
    for x in notes:
        print("note:", x)
    print(f"flight parity: {len(camp)} campaign(s), {len(oils)} soft-OILS scenario(s), {len(faults)} regression(s)")
    if not a.no_write:
        write_text(ROOT / "results" / "FLIGHT_PARITY.md", ledger(pairs, camp, oils, faults, notes, a.why))
        write_json(ROOT / "results" / "flight_parity.json", {"schema": "adcs-flight-parity/1", "tolerance": TOL,
                   "pairs": [{"set": lab, "baseline": shown(b), "current": shown(c)} for lab, b, c in pairs],
                   "regressions": faults, "notes": notes, "why": a.why, "campaigns": camp, "soft_oils": oils}, indent=1)
        print("wrote results/FLIGHT_PARITY.md and results/flight_parity.json")
    return 1 if faults else 0


if __name__ == "__main__":
    sys.exit(main())
