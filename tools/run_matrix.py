#!/usr/bin/env python3
"""Run the whole SILS test matrix with GNU Octave on N workers.

Every scenario in matlab_sils/data/scenarios, every run of every campaign
in matlab_sils/data/campaigns and every (candidate, seed) of every trade in
matlab_sils/data/trades is one job; jobs run longest-first on N worker
processes; campaigns are collected and plotted at the end.
    python3 tools/run_matrix.py --workers 4 [--only scenarios|campaigns|trades]
MATLAB users: run_scenarios and run_campaign (parfor) do the same.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse, json, pathlib, subprocess, time, concurrent.futures as cf

ROOT = pathlib.Path(__file__).resolve().parents[1] / "matlab_sils"
LOG = ROOT / "store" / "logs"

def octave(code, log):
    cmd = ["octave-cli", "--no-gui", "-q", "--eval", "startup_asils; addpath tools; " + code]
    with open(log, "w") as f:
        return subprocess.run(cmd, cwd=ROOT, stdout=f, stderr=subprocess.STDOUT).returncode

def orbit_period(case):
    """Orbit period [s] from the case's altitude (circular)."""
    import csv, math
    with open(ROOT / "cases" / f"{case}.csv") as f:
        alt = next(float(r["value"]) for r in csv.DictReader(f) if r["key"] == "orbit.alt")
    return 2 * math.pi * math.sqrt((6378137 + alt * 1e3) ** 3 / 3.986004418e14)


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--only", default="all")
    ap.add_argument("--cases", default="ais_3u,ais_img_3u", help="cases for --only solutions")
    ap.add_argument("--seeds", default="1,2"); a = ap.parse_args()
    LOG.mkdir(parents=True, exist_ok=True)
    jobs = []
    if a.only in ("all", "scenarios"):
        for f in sorted((ROOT / "data" / "scenarios").glob("*.json")):
            s = json.loads(f.read_text()); cost = s["time"]["duration_s"] / s["time"]["dt_s"]
            jobs.append((cost, f"scen:{s['id']}", f"run_scenarios({{'{s['id']}'}});"))
    if a.only in ("all", "campaigns"):
        for f in sorted((ROOT / "data" / "campaigns").glob("*.json")):
            c = json.loads(f.read_text())
            s = json.loads((ROOT / "data" / "scenarios" / f"{c['scenario']}.json").read_text())
            dur = c.get("duration_s", s["time"]["duration_s"]); cost = dur / s["time"]["dt_s"]
            for k in range(1, c["runs"] + 1):
                jobs.append((cost, f"{c['id']}:{k}", f"asils.campaign.run('{c['id']}', 'runs', {k});"))
    if a.only in ("all", "trades"):
        for f in sorted((ROOT / "data" / "trades").glob("*.json")):
            d = json.loads(f.read_text()); k = 0
            for c in d["candidates"]:
                s = json.loads((ROOT / "data" / "scenarios" / f"{c.get('scenario', d.get('scenario'))}.json").read_text())
                cost = d.get("duration_s", s["time"]["duration_s"]) / s["time"]["dt_s"]
                for seed in d.get("seeds", [1]):
                    k += 1
                    jobs.append((cost, f"{d['id']}:{k}", f"asils.trade.run('{d['id']}', 'jobs', {k});"))
    if a.only == "solutions":
        # the solution matrix: every mode x option x seed of each case (sized first)
        seeds = [int(x) for x in a.seeds.split(",")]
        for c in a.cases.split(","):
            rc = octave(f"asils.sizing.size_all('{c}', struct('quiet', true));", LOG / f"size_{c}.log")
            print(f"sized {c} rc={rc}", flush=True)
            T = orbit_period(c)
            k = 0
            for mf in sorted((ROOT / "data" / "modes").glob("*.json"), key=lambda f: json.loads(f.read_text())["order"]):
                M = json.loads(mf.read_text())
                for o in M["options"]:
                    cost = o.get("duration_orbits", M["test"]["duration_orbits"]) * T / o["dt_s"]
                    for s in seeds:
                        k += 1
                        jobs.append((cost, f"sol:{c}:{k}", f"asils.solution.run('{c}', 'jobs', {k}, 'seeds', [{' '.join(map(str, seeds))}]);"))
    jobs.sort(key=lambda j: -j[0])
    t0 = time.time(); done = 0
    print(f"{len(jobs)} jobs on {a.workers} workers", flush=True)
    with cf.ThreadPoolExecutor(a.workers) as ex:
        futs = {ex.submit(octave, code, LOG / (name.replace(':', '_') + ".log")): name for _, name, code in jobs}
        for fu in cf.as_completed(futs):
            done += 1
            print(f"[{time.time()-t0:7.0f} s] {done}/{len(jobs)} {futs[fu]} rc={fu.result()}", flush=True)
    if a.only in ("all", "campaigns"):
        for f in sorted((ROOT / "data" / "campaigns").glob("*.json")):
            cid = f.stem
            rc = octave(f"run_campaign('{cid}');", LOG / f"collect_{cid}.log")
            print(f"collected {cid} rc={rc}", flush=True)
    if a.only in ("all", "trades"):
        for f in sorted((ROOT / "data" / "trades").glob("*.json")):
            rc = octave(f"asils.trade.collect('{f.stem}');", LOG / f"collect_{f.stem}.log")
            print(f"collected {f.stem} rc={rc}", flush=True)
    if a.only == "solutions":
        for c in a.cases.split(","):
            rc = octave(f"asils.solution.collect('{c}');", LOG / f"collect_solution_{c}.log")
            print(f"collected solution {c} rc={rc}", flush=True)
    (LOG / "matrix.done").write_text("done\n")

if __name__ == "__main__":
    main()
