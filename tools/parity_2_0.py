#!/usr/bin/env python3
"""The parity gate (docs/PLAN_2_0.md S4): the new path, today's design built from the drive, gives today's
answers, and refuses where today refuses. Every engine run here reads the design alone: an empty data folder
(ADCS_ROOT) and the design database (TRINETRA_DESIGN).

    python3 tools/parity_2_0.py DESIGN.tndb [--quick] [--twin SCENARIO ...] [--oils SCENARIO ...] [--jobs N]

What it holds, each against the repository's own files or its stored results:
  inputs     every engine input file and case line the design gives is the data folder's, byte for byte
  layout     the flight software's parameter table read from its nodes is fsw/params/params.toml, and writes
             the same C and Rust
  blobs      every scenario's parameter blob (adcs-fswcfg/1) from the design is the files' byte for byte
  runs       every stored scenario run (matlab_sils/store/results_engine/<scenario>) flown again from the
             design: the same inputs fingerprint and the same metrics
  campaigns  every run of every stored campaign, its dispersions drawn again: the same draws and metrics as
             its summary (left out with --quick)
  evaluate   every row and closure of results/evaluation.json, from the design: the same state, value and why
  oils       soft OILS on QEMU (--oils): each scenario flown on the Cortex-M4F firmware from the files and from
             the design, the same metrics and timing verdicts
  twin       the MATLAB twin (--twin, Octave) flown from a data folder the design exports: the same metrics
             as its stored run (matlab_sils/store/results/<scenario>)
The design loop and the case's campaigns flown end to end from the design are tools/end_to_end.py
(results/END_TO_END.md), which this page links.

What it writes: results/PARITY_2_0.md and parity_2_0.json. Exit 1 when anything differs.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import concurrent.futures as cf
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import tomllib

import design_build
import design_inputs
import evaluate
import gen_fsw_params
from common import ROOT, write_text

ADCS = ROOT / "engine" / "target" / "release" / "adcs"
STORE = ROOT / "matlab_sils" / "store"


class Gate:
    def __init__(self, db, work, jobs):
        self.db, self.work, self.jobs = pathlib.Path(db).resolve(), pathlib.Path(work), jobs
        self.bare = self.work / "bare"
        self.bare.mkdir(parents=True, exist_ok=True)
        self.items = []

    def record(self, name, what, total, differ):
        self.items.append({"check": name, "what": what, "total": total, "differ": differ})
        print(f"parity: {name}: {total - len(differ)} of {total} equal" + (f"; differ: {differ[:5]}" if differ else ""), flush=True)

    def env(self, from_design, store):
        e = dict(os.environ, TRINETRA_STORE=str(store))
        e.pop("TRINETRA_DESIGN", None)
        e.pop("ADCS_ROOT", None)
        if from_design:
            e.update(ADCS_ROOT=str(self.bare), TRINETRA_DESIGN=str(self.db))
        return e

    def adcs(self, args, from_design, tag):
        store = self.work / f"store_{tag}"
        r = subprocess.run([str(ADCS), *args], env=self.env(from_design, store), capture_output=True, text=True, cwd=ROOT)
        shutil.rmtree(store, ignore_errors=True)
        return r

    def fly(self, scenario, args, from_design, tag):
        """(manifest or None, error) of one run, its folder removed after."""
        out = self.work / f"run_{tag}"
        r = self.adcs(["run", scenario, *args, "--out", str(out), "-q"], from_design, tag)
        if r.returncode:
            shutil.rmtree(out, ignore_errors=True)
            return None, (r.stderr or r.stdout).strip().splitlines()[-1:] or ["exit status"]
        m = json.loads((out / "manifest.json").read_text())
        shutil.rmtree(out, ignore_errors=True)
        return m, None

    # -------------------------------------------------------------------------------------------------
    def inputs(self):
        d = design_inputs.differences(self.db)
        rows, files, _fp = design_inputs.gather()
        self.record("inputs", "every engine input file and every case line, byte for byte", len(files) + len({r[0] for r in rows}), d)

    def layout(self):
        import sqlite3
        with sqlite3.connect(f"file:{self.db}?mode=ro", uri=True) as c:
            got = json.loads(dict(c.execute('SELECT "key", "value" FROM meta'))["flight_layout"])
        want = tomllib.loads((ROOT / "fsw" / "params" / "params.toml").read_text())
        differ = [f["name"] for f, g in zip(want["field"], got["field"]) if f != g]
        if {k: v for k, v in got.items() if k != "field"} != {k: v for k, v in want.items() if k != "field"} or len(got["field"]) != len(want["field"]):
            differ.append("the table's limits or its length")
        base = gen_fsw_params.outputs()
        try:
            gen_fsw_params.use(got)
            gen = gen_fsw_params.outputs()
        finally:
            gen_fsw_params.use(want)
        differ += [str(p.relative_to(ROOT)) for (p, a), (_q, b) in zip(base, gen) if a != b]
        self.record("layout", "the parameter table from its nodes, and the C and Rust it writes", len(want["field"]) + len(base), differ)

    def blobs(self):
        scen = sorted(p.stem for p in (design_inputs.DATA / "data" / "scenarios").glob("*.json"))

        def one(sc):
            a, b = self.work / f"{sc}.a.bin", self.work / f"{sc}.b.bin"
            ra = self.adcs(["params", sc, "--out", str(a)], False, f"pa_{sc}")
            rb = self.adcs(["params", sc, "--out", str(b)], True, f"pb_{sc}")
            ok = ra.returncode == rb.returncode == 0 and a.read_bytes() == b.read_bytes()
            for f in (a, b):
                f.unlink(missing_ok=True)
            return sc, ok
        with cf.ThreadPoolExecutor(self.jobs) as ex:
            res = list(ex.map(one, scen))
        self.record("blobs", "every scenario's parameter blob (adcs-fswcfg/1)", len(scen), [s for s, ok in res if not ok])

    def runs(self):
        ms = sorted((STORE / "results_engine").glob("*/manifest.json"))

        def one(m):
            x = json.loads(m.read_text())
            i = x["inputs"]
            y, err = self.fly(x["scenario"], ["--fsw", i["fsw_id"], "--seed", str(x["seed"]), *sum((["--set", o] for o in i["overrides"]), [])],
                              True, f"r_{m.parent.name}")
            if err:
                return m.parent.name, f"refused: {err[0]}"
            if y["inputs"]["input_hash"] != i["input_hash"]:
                return m.parent.name, "its inputs differ"
            return m.parent.name, None if y["metrics"] == x["metrics"] else "its metrics differ"
        with cf.ThreadPoolExecutor(self.jobs) as ex:
            res = list(ex.map(one, ms))
        self.record("runs", "every stored scenario run, flown again from the design: inputs and metrics", len(ms), [f"{a}: {b}" for a, b in res if b])

    def campaigns(self):
        import engine_campaigns as ec
        jobs = []
        for f in sorted((design_inputs.DATA / "data" / "campaigns").glob("*.json")):
            C = json.loads(f.read_text())
            s = STORE / "results_engine" / "campaigns" / C["id"] / "summary.json"
            if not s.is_file():
                continue
            summ = json.loads(s.read_text())
            for r in summ["per_run"]:
                sets, d = ec.draw(C, r["k"])
                jobs.append((C, r, sets, d, summ.get("fsw") if summ.get("fsw") in ("c", "rust") else "c"))

        def one(j):
            C, want, sets, d, fsw = j
            tag = f"{C['id']}_{want['k']}"
            if d != want["draws"]:
                return tag, "its draws differ"
            y, err = self.fly(C["scenario"], ["--case", C["case"], "--seed", str(C["seed"] + 7919 * want["k"]), "--fsw", fsw,
                                              *sum((["--set", s] for s in sets), [])], True, f"c_{tag}")
            if err:
                return tag, f"refused: {err[0]}" if not want.get("failed") else None
            got = y["metrics"] if isinstance(y["metrics"], list) else [y["metrics"]]
            return tag, None if got == want["metrics"] else "its metrics differ"
        with cf.ThreadPoolExecutor(self.jobs) as ex:
            res = list(ex.map(one, jobs))
        n = len({j[0]["id"] for j in jobs})
        self.record("campaigns", f"every run of the {n} stored campaigns, its dispersions drawn again: draws and metrics", len(jobs),
                    [f"{a}: {b}" for a, b in res if b])

    def evaluate(self):
        d = self.work / "design"
        d.mkdir(exist_ok=True)
        shutil.copy(self.db, d / "design.tndb")
        want = json.loads((ROOT / "results" / "evaluation.json").read_text())
        differ, total = [], 0
        for w in want:
            g = evaluate.evaluate(d, w["case"])
            rows = {r["id"]: r for r in g["rows"]}
            for r in w["rows"]:
                total += 1
                h = rows.get(r["id"])
                # the dissolved group `case` (decision S1-6): its nodes now name programme or systems
                if h is None or {k: v for k, v in r.items() if k != "group"} != {k: v for k, v in h.items() if k != "group"}:
                    differ.append(f"{w['case']}: {r['id']}")
            cl = {c["id"]: c for c in g["closures"]}
            for c in w["closures"]:
                total += 1
                if cl.get(c["id"]) != c:
                    differ.append(f"{w['case']}: {c['id']}")
        self.record("evaluate", "every row and closure of results/evaluation.json (a node of the dissolved group `case` names its new group)", total, differ)

    def oils(self, scenarios):
        def one(sc):
            got = []
            for way in (False, True):
                m, err = self.fly(sc, ["--fsw", "qemu", "--oils"], way, f"o_{sc}_{way}")
                if err:
                    return sc, f"refused ({'design' if way else 'files'}): {err[0]}"
                got.append((m["metrics"], m.get("oils"), m["inputs"]["input_hash"]))
            return sc, None if got[0] == got[1] else "differ"
        with cf.ThreadPoolExecutor(max(1, self.jobs // 2)) as ex:
            res = list(ex.map(one, scenarios))
        self.record("oils", "soft OILS on QEMU (Cortex-M4F firmware), from the files and from the design: metrics and timing", len(scenarios),
                    [f"{a}: {b}" for a, b in res if b])

    def twin(self, scenarios):
        t = self.work / "twin"
        shutil.rmtree(t, ignore_errors=True)
        for x in ("+asils", "pop", "tools", "startup_asils.m"):
            src = ROOT / "matlab_sils" / x
            (shutil.copytree if src.is_dir() else shutil.copy)(src, t / x)
        design_build.write_export(t, *self._export())
        (t / "store").mkdir()
        differ = []
        for sc in scenarios:
            r = subprocess.run(["octave-cli", "--no-gui", "-q", "--eval", f"startup_asils; addpath tools; run_scenarios({{'{sc}'}})"],
                               cwd=t, capture_output=True, text=True, timeout=7200)
            got = t / "store" / "results" / sc / "manifest.json"
            want = STORE / "results" / sc / "manifest.json"
            if r.returncode or not got.is_file():
                differ.append(f"{sc}: the twin stopped ({(r.stderr or r.stdout).strip().splitlines()[-1:] or ''})")
            elif json.loads(got.read_text())["metrics"] != json.loads(want.read_text())["metrics"]:
                differ.append(f"{sc}: metrics differ")
        self.record("twin", "the MATLAB twin flown from the data folder the design exports, against its stored run", len(scenarios), differ)

    def _export(self):
        import sqlite3
        with sqlite3.connect(f"file:{self.db}?mode=ro", uri=True) as c:
            files = {p: b for p, b in c.execute('SELECT "path", "body" FROM engine_input')}
            rows = c.execute('SELECT * FROM design_case ORDER BY case_id, "ord"').fetchall()
        return files, rows


def page(g, db):
    ok = all(not x["differ"] for x in g.items)
    L = ["# The parity gate: today's design gives today's answers", "",
         f"**In one line:** {'every check holds' if ok else 'something differs'}: today's design, built from the drive "
         f"(`tools/design_build.py`), read alone by the engine (an empty data folder), gives the repository's own inputs, "
         "parameter blobs, runs, campaigns and evaluation (`tools/parity_2_0.py`, `docs/PLAN_2_0.md` S4).", "",
         f"Design: `{db.resolve().relative_to(ROOT) if db.resolve().is_relative_to(ROOT) else db.name}`.", "", "| Check | What is held | Equal | Differ |", "|---|---|---|---|"]
    for x in g.items:
        L.append(f"| {x['check']} | {x['what']} | {x['total'] - len(x['differ'])} of {x['total']} | {len(x['differ'])} |")
    for x in g.items:
        if x["differ"]:
            L += ["", f"**{x['check']}, what differs:**", ""] + [f"- {d}" for d in x["differ"][:40]]
    L += ["", "The design loop and each case's campaigns flown end to end from the design: `results/END_TO_END.md`. "
          "The health map, range verdicts and tornadoes of the same design: `results/HEALTH.md`.", ""]
    return "\n".join(L)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("design")
    ap.add_argument("--quick", action="store_true", help="leave out the campaigns")
    ap.add_argument("--oils", nargs="*", default=[], metavar="SCENARIO")
    ap.add_argument("--twin", nargs="*", default=[], metavar="SCENARIO")
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 2)
    ap.add_argument("--out", default=str(ROOT / "results"))
    a = ap.parse_args(argv)
    if not ADCS.is_file():
        sys.exit("parity: the engine is not built (cargo build --release in engine/)")
    with tempfile.TemporaryDirectory() as t:
        g = Gate(a.design, t, a.jobs)
        g.inputs()
        g.layout()
        g.blobs()
        g.runs()
        if not a.quick:
            g.campaigns()
        g.evaluate()
        if a.oils:
            g.oils(a.oils)
        if a.twin:
            g.twin(a.twin)
    out = pathlib.Path(a.out)
    write_text(out / "parity_2_0.json", json.dumps({"design": str(pathlib.Path(a.design)), "checks": g.items}, indent=1) + "\n")
    write_text(out / "PARITY_2_0.md", page(g, pathlib.Path(a.design)))
    return 1 if any(x["differ"] for x in g.items) else 0


if __name__ == "__main__":
    sys.exit(main())
