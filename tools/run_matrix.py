#!/usr/bin/env python3
"""Run the whole SILS test matrix with GNU Octave on N workers.

Every scenario in matlab_sils/data/scenarios and every run of every campaign
in matlab_sils/data/campaigns is one job; jobs run longest-first on N worker
processes; campaigns are collected and plotted at the end.
    python3 tools/run_matrix.py --workers 4 [--only scenarios|campaigns]
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

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--only", default="all"); a = ap.parse_args()
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
    (LOG / "matrix.done").write_text("done\n")

if __name__ == "__main__":
    main()
