"""Monte Carlo and edge campaigns: the twin's draws (the Kp -> ap table included), the runs, the ledger.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import concurrent.futures as cf, json, math, statistics, subprocess, time
from common import Steps, write_text
from engine_base import BIN, ENG, OUT, ROOT, TWIN


# ---------------- campaigns: Monte Carlo and edge cases on the engine (asils.campaign) ----------------
CAMP = ROOT / "matlab_sils" / "data" / "campaigns"


def case_values(case):
    import csv
    out = {}
    for row in csv.DictReader(open(ROOT / "matlab_sils" / "cases" / f"{case}.csv")):
        try:
            out[row["key"]] = float(row["value"])
        except (TypeError, ValueError):
            pass
    return out


def draw(C, k):
    """asils.campaign.draw for the engine: the dispersed overrides of run k (1-based).
    MC: every dispersion drawn from its own stream (seed + 7919 k). Edge (type = "edge"): run
    2j-1 / 2j puts dispersion j at its low / high bound, every other one nominal; run 2n+1 puts
    every dispersion at its adverse (upper) bound. Random directions (CM offset, dipole, initial
    error axis) come from Python's generator, the magnitudes follow the MATLAB rule exactly."""
    import random
    rng = random.Random(C["seed"] + 7919 * k)
    cv = case_values(C["case"])
    ds = C["dispersions"] if isinstance(C["dispersions"], list) else [C["dispersions"]]
    edge = C.get("type", "mc") == "edge"
    which, hi = 0, True
    if edge and k <= 2 * len(ds):
        which, hi = (k + 1) // 2, k % 2 == 0
    sets, d = [], {}
    if "duration_s" in C:
        sets.append(("engine.duration_s", C["duration_s"]))
    unit = lambda: (lambda u: [x / math.sqrt(sum(y * y for y in u)) for x in u])([rng.gauss(0, 1) for _ in range(3)])
    for i, s in enumerate(ds, 1):
        U = lambda a, b: a + (b - a) * rng.random()
        if edge:
            if which > 0 and i != which:
                continue
            U = (lambda a, b: b if hi else a) if which > 0 else (lambda a, b: b)
            d["edge_case"], d["edge_high"] = which, int(hi)
        kind = s["kind"]
        if kind == "inertia":
            f = [1 + s["frac"] * (2 * rng.random() - 1) for _ in range(3)]
            if edge:
                g = 1 + s["frac"] * (2 * U(0, 1) - 1)
                f = [g, 2 - g, g]
            sets.append(("engine.inertia_scale", f)); d.update(inertia_scale_x=f[0], inertia_scale_y=f[1], inertia_scale_z=f[2])
        elif kind == "mass":
            v = cv["mass.m"] * (1 + s["frac"] * rng.gauss(0, 1)); sets.append(("engine.mass_kg", v)); d["mass_kg"] = v
        elif kind == "cm_offset":
            u = unit(); m = cv["surface.cpa"] * U(s["lo"], s["hi"])
            sets.append(("engine.cm_offset_m", [m * x for x in u])); d["cm_offset_mm"] = m * 1000
        elif kind == "residual_dipole":
            u = unit(); m = cv["magnetic.dres"] * U(s["lo"], s["hi"])
            sets.append(("engine.m_res", [m * x for x in u])); d["residual_dipole_Am2"] = m
        elif kind == "solar_flux":
            v = U(s["lo"], s["hi"]); sets += [("engine.f107", v), ("engine.f107a", v)]; d["F107"] = v
        elif kind == "kp":
            v = U(s["lo"], s["hi"]); sets += [("engine.kp", v), ("engine.ap", kp2ap(v))]; d["Kp"] = v
        elif kind == "accommodation":
            v = U(s["lo"], s["hi"]); sets.append(("engine.accommodation", v)); d["sigma_accom"] = v
        elif kind == "reflectivity":
            v = U(s["lo"], s["hi"]); sets.append(("engine.refl", v)); d["reflectivity"] = v
        elif kind == "initial_error_deg":
            u = unit(); v = U(s["lo"], s["hi"])
            sets += [("initial.attitude.axis_body", u), ("initial.attitude.angle_deg", v)]; d["initial_error_deg"] = v
        elif kind == "initial_rate_deg_s":
            v = U(s["lo"], s["hi"]); sets.append(("initial.rate.magnitude_deg_s", v)); d["initial_rate_deg_s"] = v
        elif kind == "arg_lat_deg":
            v = U(0, 360); sets.append(("initial.arg_lat_deg", v)); d["arg_lat_deg"] = v
        else:
            raise ValueError(f"unknown dispersion {kind}")
    return [f"{a}={json.dumps(b)}" for a, b in sets], d


# The standard Kp -> ap table, nearest node: kp2ap of the propagator (atmos.spaceweather,
# adcs_pop::spaceweather::kp2ap). ap ends at 400 (Kp 9); an exponential fit does not.
KP_NODES = [0, .33, .67, 1, 1.33, 1.67, 2, 2.33, 2.67, 3, 3.33, 3.67, 4, 4.33, 4.67, 5, 5.33, 5.67, 6, 6.33, 6.67, 7, 7.33, 7.67, 8, 8.33, 8.67, 9]


AP_NODES = [0, 2, 3, 4, 5, 6, 7, 9, 12, 15, 18, 22, 27, 32, 39, 48, 56, 67, 80, 94, 111, 132, 154, 179, 207, 236, 300, 400]


def kp2ap(kp):
    kp = max(0.0, min(9.0, kp))
    # interp1 "nearest" as Octave and adcs_pop::atmos::octave::interp1_nearest decide it:
    # past the midpoint (x[i] + x[i+1])/2 of two nodes, the upper one
    k = 0
    for i in range(len(KP_NODES) - 1):
        if (KP_NODES[i] + KP_NODES[i + 1]) / 2.0 <= kp:
            k = i + 1
        else:
            break
    return AP_NODES[k]


def camp_job(args):
    cid, scen, case, seed, k, sets, out, fsw = args
    cmd = [str(BIN), "run", scen, "--case", str(ROOT / "matlab_sils" / "cases" / f"{case}.csv"), "--seed", str(seed),
           "--fsw", fsw, "--out", str(out), "--quiet"]
    for s in sets:
        cmd += ["--set", s]
    p = subprocess.run(cmd, capture_output=True, text=True)
    return k, p.returncode, (p.stdout + p.stderr).strip()


def summarise(runs, levels=None):
    """adcs-campaign-result/1 stats (asils.campaign.collect) from per-run manifests."""
    ids, meta = [], {}
    for r in runs:
        for m in r["metrics"]:
            if m["id"] not in meta:
                ids.append(m["id"]); meta[m["id"]] = m
    stats = []
    for i in ids:
        vals = [next((m.get("value") for m in r["metrics"] if m["id"] == i), None) for r in runs]
        fin = [v for v in vals if isinstance(v, (int, float)) and math.isfinite(v)]
        verd = [next((m.get("pass") for m in r["metrics"] if m["id"] == i), None) for r in runs]
        judged = [v for v in verd if v is not None]
        req = meta[i].get("req")
        stats.append({"id": i, "unit": meta[i].get("unit", ""), "req": req, "values": vals,
                      "mean": statistics.fmean(fin) if fin else None, "std": statistics.stdev(fin) if len(fin) > 1 else 0.0,
                      "min": min(fin) if fin else None, "max": max(fin) if fin else None, "n_valid": len(fin),
                      "pass_rate": (sum(judged) / len(judged)) if judged else None,
                      "pass": (all(judged) if judged else None)})
    return stats


def campaign(a):
    ids = a.ids or sorted(p.stem for p in CAMP.glob("*.json"))
    S = Steps("engine.py", "campaign")
    rows = {}
    for cid in ids:
        C = json.loads((CAMP / f"{cid}.json").read_text())
        base = ENG / "campaigns" / cid
        jobs = []
        draws = {}
        S(1, f"{cid}, {C['runs']} runs")
        for k in range(1, C["runs"] + 1):
            sets, d = draw(C, k)
            draws[k] = d
            jobs.append((cid, C["scenario"], C["case"], C["seed"] + 7919 * k, k, sets, base / f"run_{k:04d}", a.fsw))
        t0 = time.time()
        S(2, cid)
        with cf.ProcessPoolExecutor(a.jobs) as ex:
            for k, rc, txt in ex.map(camp_job, jobs):
                if rc:
                    print(f"[FAIL] {cid} run {k}: {txt.splitlines()[-1] if txt else ''}")
        S(3, cid)
        runs = []
        for k in range(1, C["runs"] + 1):
            f = base / f"run_{k:04d}" / "manifest.json"
            if f.exists():
                m = json.loads(f.read_text())
                runs.append({"k": k, "metrics": m["metrics"] if isinstance(m["metrics"], list) else [m["metrics"]], "draws": draws[k], "wall_s": m.get("wall_s")})
        res = {"schema": "adcs-campaign-result/1", "owner": "Agastya", "id": cid, "scenario": C["scenario"], "case": C["case"],
               "type": C.get("type", "montecarlo"), "runs": len(runs), "engine": True, "fsw": a.fsw,
               "stats": summarise(runs), "per_run": runs, "wall_s": time.time() - t0}
        write_text(base / "summary.json", json.dumps(res, indent=1))
        rows[cid] = res
        print(f"{cid}: {len(runs)}/{C['runs']} runs in {time.time() - t0:.0f} s wall")
    S(4)
    campaign_ledger()


def campaign_ledger(announce=False):
    """results/ENGINE_CAMPAIGNS.md: every campaign, engine vs MATLAB twin, requirement metrics.
    `announce`: say its own steps (when it is the command, not the last step of `campaign`)."""
    S = Steps("engine.py", "campaign-ledger", show=announce)
    S(1)
    fmt = lambda x: "—" if x is None else (f"{x:.4g}" if isinstance(x, (int, float)) else str(x))
    pr = lambda p: "—" if p is None else f"{100 * p:.0f} %"
    L = ["# Monte Carlo and edge-case campaigns: Rust engine vs MATLAB twin", "",
         "Owner: Agastya. `tools/engine.py campaign` flies every campaign of `campaigns/*.toml` on the Rust engine",
         "(POP in the loop, C flight software behind the byte HAL) with the draws of `asils.campaign.draw`:",
         "the same dispersions, bounds, run count and per-run seeds (seed + 7919 k). Edge campaigns put each",
         "dispersion at its low and high bound one at a time, then all at the adverse end.", "",
         "Two differences are by design and are named, not hidden: the random streams (Mersenne twister vs",
         "SplitMix64 / Python) so MC realisations differ and only distributions compare; and the flight software",
         "on the engine keeps the NOMINAL (ground-calibrated) inertia while the plant is dispersed, whereas the",
         "MATLAB twin's control laws read the dispersed inertia. The engine is therefore the more conservative.", ""]
    allrows = []
    for cid in sorted(p.stem for p in CAMP.glob("*.json")):
        e, m = ENG / "campaigns" / cid / "summary.json", TWIN / cid / "summary.json"
        if not e.exists():
            continue
        E = json.loads(e.read_text())
        M = json.loads(m.read_text()) if m.exists() else {"stats": []}
        ms = {s["id"]: s for s in (M["stats"] if isinstance(M["stats"], list) else [M["stats"]])}
        C = json.loads((CAMP / f"{cid}.json").read_text())
        L += [f"## {cid} — {C['scenario']} on {C['case']} ({C.get('type', 'montecarlo')}, {E['runs']} runs)", "", C.get("what", ""), "",
              "| metric | req | MATLAB mean ± std [min, max] | MATLAB pass | engine mean ± std [min, max] | engine pass |", "|---|---:|---|---:|---|---:|"]
        for s in E["stats"]:
            t = ms.get(s["id"], {})
            if s.get("req") is None and s["id"] not in ("power_mean", "detumble_time", "wheel_momentum_peak"):
                continue
            mm = lambda z: "—" if z.get("mean") is None else f"{z['mean']:.4g} ± {z.get('std', 0):.3g} [{z['min']:.4g}, {z['max']:.4g}]"
            L.append(f"| {s['id']} ({s['unit']}) | {fmt(s.get('req'))} | {mm(t) if t else '—'} | {pr(t.get('pass_rate')) if t else '—'} | {mm(s)} | {pr(s.get('pass_rate'))} |")
            allrows.append({"campaign": cid, "metric": s["id"], "req": s.get("req"), "matlab_pass_rate": t.get("pass_rate"), "engine_pass_rate": s.get("pass_rate"),
                            "matlab_mean": t.get("mean"), "engine_mean": s.get("mean"), "matlab_std": t.get("std"), "engine_std": s.get("std")})
        if E.get("type") == "edge":
            req_ids = [s["id"] for s in E["stats"] if s.get("req") is not None]
            L += ["", "Edge runs (requirement metrics; run 2j-1 low / 2j high bound of dispersion j, last run all adverse):", "",
                  "| run | case | " + " | ".join(f"{i} MATLAB / engine" for i in req_ids) + " |", "|---|---|" + "---|" * len(req_ids)]
            kinds = [x["kind"] for x in (C["dispersions"] if isinstance(C["dispersions"], list) else [C["dispersions"]])]
            for r in E["per_run"]:
                k = r["k"]
                j = (k + 1) // 2
                lab = f"{kinds[j - 1]} {'high' if k % 2 == 0 else 'low'}" if j <= len(kinds) else "all adverse"
                cells = []
                for i in req_ids:
                    ev = next((x.get("value") for x in r["metrics"] if x["id"] == i), None)
                    mv = ms.get(i, {}).get("values", [None] * k)
                    mv = mv[k - 1] if len(mv) >= k else None
                    cells.append(f"{fmt(mv)} / {fmt(ev)}")
                L.append(f"| {k} | {lab} | " + " | ".join(cells) + " |")
        L.append("")
    OUT.mkdir(exist_ok=True)
    S(2)
    write_text(OUT / "engine_campaigns.json", json.dumps(allrows, indent=1))
    write_text(OUT / "ENGINE_CAMPAIGNS.md", "\n".join(L) + "\n")
    print("wrote results/ENGINE_CAMPAIGNS.md")
