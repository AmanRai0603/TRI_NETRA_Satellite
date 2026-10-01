"""Monte Carlo and edge campaigns: the twin's draws (the Kp -> ap table included), the runs, the ledger.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import concurrent.futures as cf, json, math, shutil, statistics, subprocess, time
import common
from common import Steps, write_text
from engine_base import BIN, DATA, ENG, OUT, ROOT, TWIN


# ---------------- campaigns: Monte Carlo and edge cases on the engine (asils.campaign) ----------------
CAMP = ROOT / "matlab_sils" / "data" / "campaigns"


def case_values(case):
    return common.case_values(case)


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
        elif kind == "epoch_days":          # the season: the mission starts this many days later
            v = U(s["lo"], s["hi"]); sets.append(("engine.epoch_days", v)); d["epoch_days"] = v
        elif kind == "ltan_h":              # the beta angle: the truth orbit's local time of the ascending node
            v = U(s["lo"], s["hi"]); sets.append(("engine.ltan_h", v)); d["ltan_h"] = v
        elif kind == "alt_km":              # the truth orbit's altitude (the flight software keeps the nominal)
            v = U(s["lo"], s["hi"]); sets.append(("engine.alt_km", v)); d["alt_km"] = v
        elif kind == "inertia_products":    # Ixy, Ixz, Iyz, each a fraction of sqrt(I_ii I_jj)
            f = [U(s["lo"], s["hi"]) for _ in range(3)]
            sets.append(("engine.inertia_products", f)); d.update(product_xy=f[0], product_xz=f[1], product_yz=f[2])
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


def summarise(runs, levels=None, claim=(None, None)):
    """adcs-campaign-result/1 stats (asils.campaign.collect) from per-run manifests. With a claim
    (probability, confidence), each judged metric also carries the reliability its runs show
    (the Clopper-Pearson lower bound) and whether that meets the claim."""
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
        # a run that failed to fly is a failed run for every judged requirement, never left out
        judged_metric = meta[i].get("req") is not None
        verd = [0 if (r.get("failed") and judged_metric) else v for r, v in zip(runs, verd)]
        judged = [v for v in verd if v is not None]
        req = meta[i].get("req")
        stats.append({"id": i, "unit": meta[i].get("unit", ""), "req": req, "values": vals,
                      "mean": statistics.fmean(fin) if fin else None, "std": statistics.stdev(fin) if len(fin) > 1 else 0.0,
                      "min": min(fin) if fin else None, "max": max(fin) if fin else None, "n_valid": len(fin),
                      "n_failed_runs": sum(1 for r in runs if r.get("failed")),
                      "pass_rate": (sum(judged) / len(judged)) if judged else None,
                      "pass": (all(judged) if judged else None)})
        p, conf = claim
        if p and judged:
            shown = reliability_lower(len(judged), len(judged) - sum(judged), conf)
            stats[-1].update(reliability_shown=shown, claim_met=shown >= p)
    return stats


def success_runs(p, confidence):
    """Runs with no failure that show a probability of at least p at the confidence:
    n = ceil(ln(1 - confidence) / ln(p)) (the success-run theorem; 1109 for 99.73 % at 95 %)."""
    return math.ceil(math.log(1 - confidence) / math.log(p))


def reliability_lower(n, failures, confidence):
    """The one-sided Clopper-Pearson lower bound on the probability of passing, from `failures`
    of `n` runs: the p at which `failures` or fewer failures would be seen only 1 - confidence
    of the time. None for no runs."""
    if n <= 0:
        return None
    if failures == 0:
        return (1 - confidence) ** (1 / n)

    def tail(p):        # P(failures or fewer | p), with q = 1 - p the failure probability
        q = 1 - p
        return sum(math.exp(math.lgamma(n + 1) - math.lgamma(k + 1) - math.lgamma(n - k + 1) + k * math.log(q) + (n - k) * math.log(p))
                   for k in range(failures + 1)) if 0 < p < 1 else (1.0 if p >= 1 else 0.0)
    lo, hi = 0.0, 1.0
    for _ in range(100):
        mid = (lo + hi) / 2
        if tail(mid) < 1 - confidence:
            lo = mid
        else:
            hi = mid
    return lo


def claim(C):
    """The probability the campaign must show (the strictest confidence level the case states for a
    requirement the scenario judges) and the runs that needs at the campaign's confidence; (None,
    runs) when the campaign states no confidence or no judged requirement has a level."""
    conf = C.get("confidence")
    if conf is None:
        return None, C["runs"]
    levels = {r["key"]: r.get("level", "") for r in common.case_rows(C["case"])}
    scen = json.loads((DATA / f"{C['scenario']}.json").read_text())
    ps = [round(float(levels[m["requirement"]]) / 100, 12) for m in scen.get("metrics", []) if m.get("requirement") in levels and levels[m["requirement"]].strip()]
    if not ps:
        return None, C["runs"]
    p = max(ps)
    return p, max(C["runs"], success_runs(p, conf))


# the error channels the ECSS interpretations read, by metric kind, and the statistics they apply to
INTERP_CHANNEL = {"ape": "ape_3ax_deg", "ape_los": "ape_los_deg", "ake": "ake_3ax_deg", "ake_los": "ake_los_deg"}
INTERP_Q = {"p99.73": 0.9973, "p95": 0.95, "max": 1.0}


def quantile(v, q):
    """The q-quantile as the engine takes it: the ceil(q n)-th smallest finite value (None if none)."""
    v = sorted(x for x in v if math.isfinite(x))
    return v[max(1, math.ceil(q * len(v))) - 1] if v else None


def window_index(t, spec, period):
    te = t[-1] if t else 0.0
    if spec == "last_orbit":
        return [j for j, x in enumerate(t) if x >= te - period]
    if spec == "last_half_orbit":
        return [j for j, x in enumerate(t) if x >= te - period / 2]
    if spec.startswith("after_s:"):
        return [j for j, x in enumerate(t) if x >= float(spec[8:])]
    return list(range(len(t)))      # all (and 'pointing', which needs the mode: taken as all)


def interpretations(run_dirs, metrics):
    """ECSS-E-ST-60-10C statistical interpretations of each error metric over a campaign
    (one value each, in the metric's unit):
      temporal  the statistic within each run, then the worst run
      ensemble  the quantile across runs at each instant, then the worst instant
      mixed     the quantile over every sample of every run, pooled
    For ape/ape_los/ake/ake_los metrics with a percentile or max statistic; the runs share one time grid."""
    out = []
    series = {}
    for d in run_dirs:
        f = d / "channels.csv"
        man = d / "manifest.json"
        if f.exists() and man.exists():
            series[d] = (f.read_text().splitlines(), json.loads(man.read_text())["orbit"]["period_s"])
    if not series:
        return out
    for m in metrics:
        col, q = INTERP_CHANNEL.get(m.get("kind")), INTERP_Q.get(m.get("statistic", "max"))
        if col is None or q is None:
            continue
        per_run = []
        for lines, period in series.values():
            head = lines[0].split(",")
            if col not in head:
                continue
            k, kt = head.index(col), head.index("t_s")
            rows = [ln.split(",") for ln in lines[1:]]
            t = [float(r[kt]) for r in rows]
            idx = window_index(t, m.get("window", "all"), period)
            per_run.append([float(rows[j][k]) for j in idx])
        if not per_run:
            continue
        n = min(len(x) for x in per_run)
        temporal = [quantile(x, q) for x in per_run]
        temporal = max((v for v in temporal if v is not None), default=None)
        ensemble = [quantile([x[i] for x in per_run], q) for i in range(n)]
        ensemble = max((v for v in ensemble if v is not None), default=None)
        mixed = quantile([v for x in per_run for v in x], q)
        out.append({"id": m["id"], "statistic": m.get("statistic", "max"), "runs": len(per_run),
                    "temporal": temporal, "ensemble": ensemble, "mixed": mixed})
    return out


def campaign(a):
    ids = a.ids or sorted(p.stem for p in CAMP.glob("*.json"))
    S = Steps("engine.py", "campaign")
    rows, failed_total = {}, 0
    for cid in ids:
        C = json.loads((CAMP / f"{cid}.json").read_text())
        base = ENG / "campaigns" / cid
        # a campaign starts from an empty folder: no run of an earlier campaign is ever read as this one's
        if base.exists():
            shutil.rmtree(base)
        jobs = []
        draws = {}
        p_claim, n_runs = claim(C)
        S(1, f"{cid}, {n_runs} runs" + (f" (to show {100 * p_claim:g} % at {100 * C['confidence']:g} % confidence)" if p_claim else ""))
        for k in range(1, n_runs + 1):
            sets, d = draw(C, k)
            draws[k] = d
            jobs.append((cid, C["scenario"], C["case"], C["seed"] + 7919 * k, k, sets, base / f"run_{k:04d}", a.fsw))
        t0 = time.time()
        S(2, cid)
        errors = {}
        with cf.ProcessPoolExecutor(a.jobs) as ex:
            for k, rc, txt in ex.map(camp_job, jobs):
                if rc:
                    errors[k] = txt.splitlines()[-1] if txt else f"exit status {rc}"
                    print(f"[FAIL] {cid} run {k}: {errors[k]}")
        S(3, cid)
        runs = []
        for k in range(1, n_runs + 1):
            f = base / f"run_{k:04d}" / "manifest.json"
            if k in errors or not f.exists():
                runs.append({"k": k, "failed": True, "error": errors.get(k, "no manifest written"), "metrics": [], "draws": draws[k]})
                continue
            m = json.loads(f.read_text())
            runs.append({"k": k, "metrics": m["metrics"] if isinstance(m["metrics"], list) else [m["metrics"]], "draws": draws[k], "wall_s": m.get("wall_s")})
        nfail = sum(1 for r in runs if r.get("failed"))
        failed_total += nfail
        res = {"schema": "adcs-campaign-result/1", "owner": "Agastya", "id": cid, "scenario": C["scenario"], "case": C["case"],
               "type": C.get("type", "montecarlo"), "runs": len(runs) - nfail, "failed_runs": nfail, "engine": True, "fsw": a.fsw,
               "stats": summarise(runs, claim=(p_claim, C.get("confidence"))), "per_run": runs, "wall_s": time.time() - t0,
               "claim": {"probability": p_claim, "confidence": C.get("confidence"), "runs_needed": n_runs} if p_claim else None,
               "interpretations": interpretations([base / f"run_{r['k']:04d}" for r in runs if not r.get("failed")],
                                                  json.loads((DATA / f"{C['scenario']}.json").read_text()).get("metrics", []))}
        write_text(base / "summary.json", json.dumps(res, indent=1))
        rows[cid] = res
        print(f"{cid}: {len(runs) - nfail}/{n_runs} runs flown, {nfail} failed, in {time.time() - t0:.0f} s wall")
    S(4)
    campaign_ledger()
    return failed_total


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
        if E.get("interpretations"):
            L += ["", "ECSS-E-ST-60-10C interpretations of the error metrics (engine): temporal = the statistic in each run, worst run;",
                  "ensemble = the quantile across runs at each instant, worst instant; mixed = the quantile of every sample pooled.", "",
                  "| metric | statistic | runs | temporal | ensemble | mixed |", "|---|---|---:|---:|---:|---:|"]
            for x in E["interpretations"]:
                L.append(f"| {x['id']} | {x['statistic']} | {x['runs']} | {fmt(x['temporal'])} | {fmt(x['ensemble'])} | {fmt(x['mixed'])} |")
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
