"""The two-sample campaign: identical draws flown on the engine and on the MATLAB twin, and a rank test of
whether each metric's values come from the same distribution (A2).

The engine's own campaigns and the twin's draw from different random streams, so their run counts and
realisations differ. Here the draws are the engine's (engine_campaigns.draw, run k of the campaign's
seed), written once to a file; the engine reads them as `--set` strings and the twin through
matlab_sils/tools/run_draws.m, so a pair of runs differs only in the simulator. The test is the design's Mann-Whitney
U test (normal approximation with tie correction and continuity correction), a published two-sample test of
a location shift; it judges no requirement, it says whether the two simulators agree on a metric.

  python3 tools/engine.py twosample <campaign> [--draws 100] [--jobs N] [--out FILE]

Fewer than 100 draws is a smoke test of the plumbing and the section says it is not a result.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import concurrent.futures as cf, json, math, pathlib, statistics, subprocess
from common import write_text
from design_call import Buf, call
from engine_base import ENG, OUT, ROOT, TWIN
import engine_campaigns as EC

MIN_DRAWS = 100          # fewer is a smoke test, not a result


def mann_whitney(a, b):
    """The design's Mann-Whitney U test (kpistats::kpi_mann_whitney, asked of the engine): n1, n2, u, z and the
    two-sided p; u, z and p are None when a side has no finite value. This file keeps no test of its own."""
    num = lambda v: [float(x) if isinstance(x, (int, float)) else math.nan for x in v]
    out = call("kpistats::kpi_mann_whitney", Buf(num(a)), Buf(num(b)))
    n1, n2, u, z, p = out[:5]
    nn = lambda x: None if math.isnan(x) else x
    return {"n1": int(n1), "n2": int(n2), "u": nn(u), "z": nn(z), "p": nn(p)}


def identical_draws(C, n):
    """The n draws both sides fly: run k of the campaign's seed, as the engine draws it, each as named values."""
    out = []
    for k in range(1, n + 1):
        sets, d = EC.draw(C, k)
        out.append({"k": k, "seed": C["seed"] + 7919 * k, "draws": d,
                    "set": [{"name": s.split("=", 1)[0], "value": json.loads(s.split("=", 1)[1])} for s in sets]})
    return out


def write_draws(C, n, folder):
    """The file the twin reads (matlab_sils/tools/run_draws.m): the campaign and its n draws."""
    f = folder / "draws.json"
    folder.mkdir(parents=True, exist_ok=True)
    write_text(f, json.dumps({"campaign": C["id"], "scenario": C["scenario"], "case_id": C["case"], "draws": identical_draws(C, n)}, indent=1))
    return f


def _values(runs):
    v = {}
    for r in runs:
        ms = r["metrics"] if isinstance(r["metrics"], list) else [r["metrics"]]
        for m in ms:
            v.setdefault(m["id"], {})[r["k"]] = m.get("value")
    return v


def compare(C, engine_runs, twin_runs):
    """The test result for a campaign: each metric over the draws both sides flew (paired by run k)."""
    ev, tv = _values(engine_runs), _values(twin_runs)
    ks = sorted({r["k"] for r in engine_runs} & {r["k"] for r in twin_runs})
    metrics = []
    for mid in ev:
        if mid not in tv:
            continue
        a, b = [ev[mid].get(k) for k in ks], [tv[mid].get(k) for k in ks]
        t = mann_whitney(a, b)
        med = lambda v: statistics.median([x for x in v if isinstance(x, (int, float)) and math.isfinite(x)] or [math.nan])
        metrics.append({"id": mid, **t, "median_engine": med(a), "median_twin": med(b)})
    n = len(ks)
    return {"schema": "adcs-twosample/1", "owner": "Agastya", "campaign": C["id"], "scenario": C["scenario"], "case": C["case"], "test": "Mann-Whitney U",
            "draws": n, "complete": n >= MIN_DRAWS, "alpha": 0.05, "metrics": metrics}


def section(res):
    """The ledger's section (results/ENGINE_CAMPAIGNS.md) for a two-sample result."""
    g = lambda x: "—" if x is None or (isinstance(x, float) and math.isnan(x)) else f"{x:.4g}"
    L = ["", f"## Two-sample test — {res['campaign']} ({res['scenario']} on {res['case']})", "",
         f"The same {res['draws']} draws (the engine's, run k of the campaign's seed) were flown on the Rust engine and on the MATLAB twin; "
         f"{res['test']} test per metric, two-sided, normal approximation with tie and continuity corrections."]
    if not res["complete"]:
        L += ["", f"**Fewer than {MIN_DRAWS} draws: a smoke test of the plumbing, not a result.** No statement about the two sides follows from it."]
    else:
        L += [f"A p-value below {res['alpha']:g} marks a metric on which the two simulators give different distributions; "
              "the metrics are tested one by one, with no correction for their number."]
    L += ["", "| metric | n | median MATLAB | median engine | U | z | p | differs at 5 % |", "|---|---:|---:|---:|---:|---:|---:|---|"]
    for m in res["metrics"]:
        d = "—" if m["p"] is None else ("yes" if m["p"] < res["alpha"] else "no")
        L.append(f"| {m['id']} | {m['n1']} / {m['n2']} | {g(m['median_twin'])} | {g(m['median_engine'])} | {g(m['u'])} | {g(m['z'])} | {g(m['p'])} | {d if res['complete'] else '—'} |")
    return L + [""]


def _twin_runs(folder, ks):
    out = []
    for k in ks:
        f = folder / f"run_{k:04d}.json"
        if f.exists():
            out.append(json.loads(f.read_text()))
    return out


def twosample(a):
    """Fly the draws on both sides and file the result (a.out)."""
    C = json.loads((EC.CAMP / f"{a.id}.json").read_text())
    if a.duration_s:
        C["duration_s"] = a.duration_s
    n = a.draws
    base = ENG / "twosample" / a.id
    twin_dir = TWIN.parent / "twosample" / a.id
    f = write_draws(C, n, base)
    draws = json.loads(f.read_text())["draws"]
    jobs = [(a.id, C["scenario"], C["case"], d["seed"], d["k"], [f"{s['name']}={json.dumps(s['value'])}" for s in d["set"]],
             base / f"run_{d['k']:04d}", a.fsw) for d in draws]
    with cf.ProcessPoolExecutor(a.jobs) as ex:
        for k, rc, txt in ex.map(EC.camp_job, jobs):
            if rc:
                print(f"[FAIL] engine run {k}: {txt.splitlines()[-1] if txt else rc}")
    twin_dir.mkdir(parents=True, exist_ok=True)
    ks = [d["k"] for d in draws]
    procs = [subprocess.Popen(["octave-cli", "--no-gui", "-q", "--eval",
                               f"startup_asils; addpath tools; run_draws('{f}', '{twin_dir}', {w + 1}:{a.jobs}:{n});"],
                              cwd=ROOT / "matlab_sils", stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT) for w in range(min(a.jobs, n))]
    for p in procs:
        p.wait()
    eng = []
    for k in ks:
        m = base / f"run_{k:04d}" / "manifest.json"
        if m.exists():
            eng.append({"k": k, "metrics": json.loads(m.read_text())["metrics"]})
    res = compare(C, eng, _twin_runs(twin_dir, ks))
    out = pathlib.Path(a.out) if a.out else OUT / "engine_twosample.json"
    write_text(out, json.dumps(res, indent=1))
    print(f"{a.id}: {res['draws']} paired draws, {'a result' if res['complete'] else 'a smoke test (fewer than ' + str(MIN_DRAWS) + ' draws)'}; wrote {out}")
    for m in res["metrics"]:
        print(f"  {m['id']}: p = {m['p']}")
    return 0
