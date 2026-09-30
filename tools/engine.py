#!/usr/bin/env python3
"""Orchestrate the Rust SILS engine and the two flight-software builds.

Python is the upper layer: it builds, fans runs out over processes, gathers
results and writes the ledgers. The physics runs in Rust (engine/), the flight
software in C (fsw/) or Rust (fsw-rs/), all from one pseudocode (fsw/pseudocode).

  python3 tools/engine.py build                     make fsw (C tests) + cargo test/build fsw-rs and engine
  python3 tools/engine.py run [scen ...] [--fsw c|rust] [--jobs N] [--seed S]
  python3 tools/engine.py mc <scen> --seeds N [--fsw c|rust] [--jobs N]
  python3 tools/engine.py fsw-parity [scen ...] [--duration S]     C vs Rust flight software, same loop, same bytes
  python3 tools/engine.py solutions [case ...] [--seeds 1,2]       the customer-case solution matrix on the engine
                                                                    (every mode x option x seed, sized products)
                                                                    -> results/ENGINE_SOLUTIONS.md
  python3 tools/engine.py dispatch [case ...]                       the recommended solution's flight configuration:
                                                                    dist/dispatch/<case>/<family>/fsw/ (adcs-fswcfg/1 blob,
                                                                    decoded JSON, build notes) + an engine mission check
  python3 tools/engine.py vobc [scen ...] [--duration S]           virtual-OBC loop: the flight software as a separate
                                                                    process and on QEMU Cortex-M4 (C and Rust) over
                                                                    adcs-link/1, compared with the in-process builds
                                                                    -> results/VIRTUAL_OBC.md
  python3 tools/engine.py campaign [id ...]                         every Monte Carlo / edge campaign on the engine
                                                                    (asils.campaign.draw semantics) vs the MATLAB twin
                                                                    -> results/ENGINE_CAMPAIGNS.md
  python3 tools/engine.py oils [scen ...] [--fsw qemu|qemu-rs]      SILS and soft OILS (flight software as Cortex-M4F
                                                                    firmware, exact instruction timing, command latency)
                                                                    side by side -> results/SOFT_OILS.md
  python3 tools/engine.py twin-parity                               engine vs MATLAB twin, metric by metric
                                                                    -> results/ENGINE_PARITY.md, results/engine_parity.json

Results land in matlab_sils/store/results_engine/<scenario>/ (adcs-rec/1, the
format tools/report.py reads). Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse, concurrent.futures as cf, json, math, os, pathlib, re, statistics, subprocess, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[1]
BIN = ROOT / "engine" / "target" / "release" / "adcs"
DATA = ROOT / "matlab_sils" / "data" / "scenarios"
TWIN = ROOT / "matlab_sils" / "store" / "results"
ENG = ROOT / "matlab_sils" / "store" / "results_engine"
OUT = ROOT / "results"


def sh(cmd, cwd=ROOT, check=True):
    print("$", " ".join(map(str, cmd)), flush=True)
    return subprocess.run(cmd, cwd=cwd, check=check)


def build(_):
    sh(["make", "-s", "clean"], cwd=ROOT / "fsw")
    sh(["make", "-s", "test"], cwd=ROOT / "fsw")
    sh(["make", "-s", "check"], cwd=ROOT / "fsw")
    sh(["cargo", "test", "--release", "-q"], cwd=ROOT / "fsw-rs")
    # the OBC links the C-ABI static library: build it (host and Cortex-M) before make obc, since a plain
    # cargo build leaves a newer libadcs_fsw.a without the exports that make would not rebuild
    sh(["cargo", "build", "--release", "-q", "--no-default-features", "--features", "cabi"], cwd=ROOT / "fsw-rs")
    sh(["cargo", "build", "--release", "-q", "--no-default-features", "--features", "cabi", "--target", "thumbv7em-none-eabihf"], cwd=ROOT / "fsw-rs")
    sh(["make", "-s", "obc"], cwd=ROOT / "fsw")            # virtual OBC firmware (process, QEMU) and the insn plugin
    sh(["cargo", "test", "--release", "-q"], cwd=ROOT / "engine")
    sh(["cargo", "build", "--release", "-q"], cwd=ROOT / "engine")


def scenarios(names):
    return names or sorted(p.stem for p in DATA.glob("*.json"))


def one(args):
    scen, fsw, seed, out, extra = args
    cmd = [str(BIN), "run", scen, "--fsw", fsw, "--seed", str(seed), "--quiet"] + extra
    if out:
        cmd += ["--out", str(out)]
    t0 = time.time()
    p = subprocess.run(cmd, capture_output=True, text=True)
    return scen, seed, p.returncode, time.time() - t0, (p.stdout + p.stderr).strip()


def run(a):
    jobs = [(s, a.fsw, a.seed, None, []) for s in scenarios(a.scenarios)]
    with cf.ProcessPoolExecutor(a.jobs) as ex:
        for scen, _, rc, dt, txt in ex.map(one, jobs):
            print(f"[{'ok' if rc == 0 else 'FAIL'}] {scen:24s} {dt:6.1f} s wall")
            if rc:
                print(txt)


def mc(a):
    base = ENG / f"mc_{a.scenario}"
    jobs = [(a.scenario, a.fsw, s, base / f"seed_{s:03d}", []) for s in range(1, a.seeds + 1)]
    rows = []
    with cf.ProcessPoolExecutor(a.jobs) as ex:
        for scen, seed, rc, dt, txt in ex.map(one, jobs):
            if rc:
                print(f"seed {seed} FAILED\n{txt}")
                continue
            m = json.loads((base / f"seed_{seed:03d}" / "manifest.json").read_text())
            rows.append({x["id"]: x["value"] for x in m["metrics"]} | {"seed": seed})
    ids = [k for k in rows[0] if k != "seed"] if rows else []
    summ = {}
    for k in ids:
        v = [r[k] for r in rows if isinstance(r.get(k), (int, float)) and math.isfinite(r[k])]
        if v:
            summ[k] = {"n": len(v), "mean": statistics.fmean(v), "std": statistics.pstdev(v), "min": min(v), "max": max(v)}
    (base / "summary.json").write_text(json.dumps({"scenario": a.scenario, "fsw": a.fsw, "runs": rows, "stats": summ}, indent=1))
    for k, s in summ.items():
        print(f"  {k:28s} mean {s['mean']:.4g}  std {s['std']:.3g}  [{s['min']:.4g}, {s['max']:.4g}]  n={s['n']}")


def fsw_parity(a):
    for s in scenarios(a.scenarios):
        p = subprocess.run([str(BIN), "parity", s, "--set", f"engine.duration_s={a.duration}"], capture_output=True, text=True)
        print(re.sub(r"\(trinetra[^)]*\)\)", "", (p.stdout + p.stderr).strip().splitlines()[-1]))


# MATLAB campaign <-> engine Monte Carlo of the same scenario (tools/engine.py mc <scenario>)
MC_PAIRS = {"mc_detumble_ais": "detumble_ais", "mc_nadir_ais": "nadir_hold_ais", "mc_fine_img": "fine_hold_img", "mc_slew_img": "slew_img"}

# Differences that were traced to their cause (kept with the ledger).
NOTES = [
    ("fine_hold_fmr, fine_hold_fmr_rcs · power_mean",
     "Knife-edge in the fluid-ring power model, not an engine difference: the 2 W field power per ring is on while the "
     "driver's momentum target exceeds 2 % of h_max (2e-5 N m s). Both runs start with all three rings on (6 W); the "
     "ring momenta then hover at 2.3-2.6e-5 N m s, so which side of the threshold each run settles on decides a 2 W "
     "requirement. Action: a hysteresis band on the field-power switch (asils.devices.mex and the engine's Mex)."),
    ("fine_hold_rw_rcs · ape_los_p9973",
     "Traced: the momentum-dump pulses (same times, same counts, same propellant on both sides) are fed forward with "
     "the nominal thruster torque; the unit's thrust-scale draw (sigma 3 %) leaves an uncompensated torque for the "
     "~30 s dump. Engine draw 0.985 -> ~3e-6 N m -> 0.03 deg offset against Kp = 5.4e-3; MATLAB draw 0.995 -> 0.01 deg. "
     "Engine Monte Carlo (24 seeds): mean 0.034 deg, range 0.009-0.079 deg; the MATLAB run sits at the lucky end. "
     "Finding: RCS dumping during imaging breaks the 0.01 deg APE for most thrust-scale draws -> calibrate thrust "
     "on orbit or inhibit RCS dumping in the fine modes."),
    ("sun_spin_ais · sun_angle, spin_rate_error, share",
     "Traced to the flight logic (identical in MATLAB, C and Rust): when spin-up converges with the Sun on +Z, the "
     "sigma flip reverses the spin every 60 s check, which drives the body through zero spin and never moves the Sun "
     "to -Z. Engine seed 1 is that case; 3 of 24 Monte Carlo seeds lock up (mean sun angle 26 deg, range 2.7-173 deg), "
     "the MATLAB run (6.5 deg) is inside. Finding: the sign-flip guard needs a hemisphere manoeuvre, not a spin reversal."),
    ("nadir_hold_ais, mission_ais · ake_los / ape_los",
     "Realisation spread of the coils-only MEKF (magnetometer + Sun): engine 24-seed AKE mean 4.0 deg (0.8-8.5 deg); "
     "the MATLAB runs (3.9-4.0 deg) are inside the distribution."),
    ("detumble_* and mission_* · detumble_time",
     "Random initial tumble direction; at distribution level the MATLAB campaign (48.5 +/- 12 min) and the engine "
     "campaign mc_detumble_ais with the full dispersions (56.3 +/- 11.4 min) overlap; results/ENGINE_CAMPAIGNS.md."),
    ("mc_nadir_ais · ape_los",
     "Like-for-like since the engine campaigns (tools/engine.py campaign, results/ENGINE_CAMPAIGNS.md) disperse the "
     "residual dipole, inertia, CM offset, flux, Kp, accommodation and reflectivity exactly as asils.campaign.draw: "
     "MATLAB mean 89.7 deg vs engine 93.0 deg; the coils-only nadir hold cannot absorb the dispersed dipole."),
]


def twin_parity(_):
    """Engine (Rust) vs the MATLAB twin, same scenario, same case, same product.
    One realisation each: the random streams differ (MATLAB twister vs counter-based
    SplitMix64), so a value-by-value match is not expected; the ledger records the
    verdict agreement and the ratio, and the model differences named in fsw/twin_map.toml."""
    rows = []
    for s in scenarios([]):
        a, b = TWIN / s / "manifest.json", ENG / s / "manifest.json"
        if not (a.exists() and b.exists()):
            continue
        ma, mb = json.loads(a.read_text()), json.loads(b.read_text())
        lst = lambda x: x if isinstance(x, list) else [x]
        mb_by = {m["id"]: m for m in lst(mb["metrics"])}
        for m in lst(ma["metrics"]):
            e = mb_by.get(m["id"])
            if not e:
                continue
            va, vb = m.get("value"), e.get("value")
            fin = lambda x: isinstance(x, (int, float)) and math.isfinite(x)
            ratio = vb / va if fin(va) and fin(vb) and va != 0 else None
            rows.append({"scenario": s, "metric": m["id"], "unit": m.get("unit", ""), "matlab": va if fin(va) else None,
                         "engine": vb if fin(vb) else None, "ratio": ratio, "req": m.get("req"),
                         "pass_matlab": m.get("pass"), "pass_engine": e.get("pass"),
                         "agree": (m.get("pass") == e.get("pass")) if m.get("pass") is not None else None,
                         "wall_matlab_s": ma.get("wall_s"), "wall_engine_s": mb.get("wall_s")})
    OUT.mkdir(exist_ok=True)
    (OUT / "engine_parity.json").write_text(json.dumps(rows, indent=1))
    judged = [r for r in rows if r["agree"] is not None]
    agree = sum(r["agree"] for r in judged)
    walls = {}
    for r in rows:
        walls[r["scenario"]] = (r["wall_matlab_s"], r["wall_engine_s"])
    fmt = lambda x: "—" if x is None else (f"{x:.4g}" if isinstance(x, float) else str(x))
    verdict = lambda p: {1: "pass", 0: "FAIL"}.get(p, "—")
    L = ["# Engine parity: Rust engine vs MATLAB twin", "",
         "Owner: Agastya. Generated by `tools/engine.py twin-parity` from the filed runs:",
         "`matlab_sils/store/results/<scenario>` (MATLAB twin, POP in the loop) and",
         "`matlab_sils/store/results_engine/<scenario>` (Rust engine, C flight software behind the byte HAL).", "",
         "The engine steps the Rust port of POP inside the loop exactly as the twin steps the MATLAB POP",
         "(adcs-pop: time scales, frames, DE440, degree-6 field, Battin third body, DTM2020 drag, conical",
         "SRP, RK4 10 s + Hermite): orbit, Sun, Moon, shadow, density and field are bit-identical to the",
         "twin's filed channels (see 'Truth environment' below). What still differs is the random stream",
         "(MATLAB Mersenne twister vs the engine's counter-based SplitMix64), so single runs are",
         "compared by metric ratio and verdict agreement, and Monte Carlo on both sides compares",
         "distributions (`tools/engine.py mc`).", "",
         f"**Verdict agreement: {agree} of {len(judged)} judged metrics** "
         f"({len(rows)} metrics over {len(walls)} scenarios).", "",
         "| scenario | metric | MATLAB | engine | engine/MATLAB | req | MATLAB | engine |",
         "|---|---|---:|---:|---:|---:|---|---|"]
    for r in rows:
        flag = "" if r["agree"] in (None, True) else " ⚠"
        L.append(f"| {r['scenario']} | {r['metric']} ({r['unit']}) | {fmt(r['matlab'])} | {fmt(r['engine'])} | {fmt(r['ratio'])} | "
                 f"{fmt(r['req'])} | {verdict(r['pass_matlab'])} | {verdict(r['pass_engine'])}{flag} |")
    L += ["", "## Truth environment: engine vs MATLAB twin, every recorded sample", "",
          "| scenario | samples | max |r| difference [m] | max density ratio - 1 | max Sun-direction difference | max |B| ratio - 1 |", "|---|---:|---:|---:|---:|---:|"]
    import csv
    for s in scenarios([]):
        a, b = TWIN / s / "channels.csv", ENG / s / "channels.csv"
        if not (a.exists() and b.exists()):
            continue
        A, B = list(csv.DictReader(open(a))), list(csv.DictReader(open(b)))
        wr = wrho = wsun = wb = 0.0
        for x, y in zip(A, B):
            f = lambda d, k: float(d[k])
            wr = max(wr, sum((f(x, f"r_{k}_m") - f(y, f"r_{k}_m")) ** 2 for k in "xyz") ** 0.5)
            wrho = max(wrho, abs(f(x, "rho_kgm3") / f(y, "rho_kgm3") - 1) if f(y, "rho_kgm3") else 0)
            wsun = max(wsun, sum((f(x, f"sun_{k}") - f(y, f"sun_{k}")) ** 2 for k in "xyz") ** 0.5)
            na = sum(f(x, f"B_{k}_T") ** 2 for k in "xyz") ** 0.5; nb = sum(f(y, f"B_{k}_T") ** 2 for k in "xyz") ** 0.5
            wb = max(wb, abs(na / nb - 1))
        L.append(f"| {s} | {min(len(A), len(B))} | {wr:.3g} | {wrho:.3g} | {wsun:.3g} | {wb:.3g} |")
    L += ["", "Channels are filed with 9 significant digits, so differences below ~1e-8 relative are the",
          "file's rounding, not the models.", ""]
    L += ["", "## Distributions: MATLAB campaigns vs engine Monte Carlo", "",
          "MATLAB campaigns also disperse inertia, residual dipole, CM offset, solar flux, accommodation and",
          "reflectivity; the engine Monte Carlo disperses the units and the initial state only.", "",
          "| MATLAB campaign | engine scenario | metric | MATLAB mean ± std [min, max] (n) | engine mean ± std [min, max] (n) |",
          "|---|---|---|---|---|"]
    for camp, scen in MC_PAIRS.items():
        a, b = TWIN / camp / "summary.json", ENG / f"mc_{scen}" / "summary.json"
        if not (a.exists() and b.exists()):
            continue
        sa, sb = json.loads(a.read_text()), json.loads(b.read_text())
        for m in (sa["stats"] if isinstance(sa["stats"], list) else [sa["stats"]]):
            e = sb["stats"].get(m["id"])
            if not e or m.get("mean") is None:
                continue
            L.append(f"| {camp} | {scen} | {m['id']} | {m['mean']:.4g} ± {m['std']:.3g} [{m['min']:.4g}, {m['max']:.4g}] ({m.get('n_valid', '')}) | "
                     f"{e['mean']:.4g} ± {e['std']:.3g} [{e['min']:.4g}, {e['max']:.4g}] ({e['n']}) |")
    L += ["", "## Differences traced to their cause", ""]
    L += [f"- **{k}.** {v}" for k, v in NOTES]
    L += ["", "## Wall time (one core each)", "", "| scenario | MATLAB/Octave [s] | engine [s] | speed-up |", "|---|---:|---:|---:|"]
    for s, (wa, wb) in walls.items():
        if wa and wb:
            L.append(f"| {s} | {wa:.0f} | {wb:.2f} | {wa / wb:.0f}x |")
    (OUT / "ENGINE_PARITY.md").write_text("\n".join(L) + "\n")
    print(f"verdict agreement {agree}/{len(judged)}; wrote results/ENGINE_PARITY.md")


MODES_DIR = ROOT / "matlab_sils" / "data" / "modes"
SOL = ROOT / "matlab_sils" / "store" / "solutions_engine"


def case_value(case, key):
    for line in (ROOT / "matlab_sils" / "cases" / f"{case}.csv").read_text().splitlines():
        f = line.split(",")
        if len(f) > 4 and f[1] == key:
            return float(f[4])
    raise KeyError(key)


def mode_scenario(case, M, o):
    """asils.solution.scenario: one mode test of one option on the case's sized product."""
    a = 6378137 + case_value(case, "orbit.alt") * 1e3
    T = 2 * math.pi * math.sqrt(a ** 3 / 3.986004418e14)
    orbits = o.get("duration_orbits", M["test"]["duration_orbits"])
    win = o.get("window", M["test"]["window"])
    g = M["guidance"] if M["guidance"] not in ("none", "sun_vector") else "nadir"
    fsw = {"start_mode": o["fsw_mode"], "guidance": {"kind": g}, "rcs_dump": 1.0 if o.get("dump") == "rcs" else 0.0}
    if "algorithms" in o:
        fsw["algorithms"] = o["algorithms"]
    if o["actuator"] == "fmr":
        fsw["dump_gain"] = 0.03
    ms = [dict(m, window=win) if m.get("window") == M["test"]["window"] else m for m in M["metrics"]]
    return {"schema": "adcs-scenario/1", "id": f"{case}__{M['id']}__{o['id'].replace('+', '_')}", "case": case,
            "label": f"{case} — {M['label']} with {o['id']}", "product": f"SZ-{case}-{o['family']}",
            "time": {"duration_s": round(orbits * T), "dt_s": o["dt_s"], "record_dt_s": 1.0},
            "initial": o.get("test_initial") or {"attitude": M["test"]["attitude"], "rate": M["test"]["rate"]}, "fsw": fsw, "metrics": ms}


def sol_job(args):
    case, scen_path, seed, out, fsw = args
    p = subprocess.run([str(BIN), "run", str(scen_path), "--case", str(ROOT / "matlab_sils" / "cases" / f"{case}.csv"),
                        "--seed", str(seed), "--fsw", fsw, "--out", str(out), "--quiet"], capture_output=True, text=True)
    return args, p.returncode, (p.stdout + p.stderr).strip()


def solutions(a):
    cases = a.cases or ["ais_3u", "ais_img_3u"]
    seeds = [int(x) for x in a.seeds.split(",")]
    modes = sorted((json.loads(f.read_text()) for f in MODES_DIR.glob("*.json")), key=lambda M: M["order"])
    jobs = []
    for c in cases:
        for M in modes:
            for o in M["options"]:
                d = SOL / c / M["id"] / o["id"].replace("+", "_")
                d.mkdir(parents=True, exist_ok=True)
                sp = d / "scenario.json"
                sp.write_text(json.dumps(mode_scenario(c, M, o), indent=1))
                jobs += [(c, sp, s, d / f"seed_{s}", a.fsw) for s in seeds]
    t0 = time.time()
    with cf.ProcessPoolExecutor(a.jobs) as ex:
        for (c, sp, s, out, _), rc, txt in ex.map(sol_job, jobs):
            if rc:
                print(f"[FAIL] {sp.parent.relative_to(SOL)} seed {s}: {txt.splitlines()[-1] if txt else ''}")
    print(f"{len(jobs)} mode tests in {time.time() - t0:.0f} s wall")
    # score: an option is feasible when every requirement-bound metric passes on every seed
    res = {}
    L = ["# Solution matrix on the Rust engine", "", "Owner: Agastya. `tools/engine.py solutions` -- every mission mode x option of",
         "each case, flown on the case's sized products (matlab_sils/store/sized) with the C flight software,",
         f"seeds {a.seeds}; the MATLAB column is `matlab_sils/store/solutions/<case>/solution.json` when collected.", ""]
    for c in cases:
        twin = {}
        tj = ROOT / "matlab_sils" / "store" / "solutions" / c / "solution.json"
        if tj.exists():
            T = json.loads(tj.read_text())
            for mid, mv in T.get("modes", {}).items():
                opts = mv.get("options", [])
                for o in (opts if isinstance(opts, list) else [opts]):
                    twin[(mid, o.get("id"))] = o
        L += [f"## {c}", "", "| mode | option | engine feasible | objective (worst seed) | failing on the engine | MATLAB feasible | MATLAB objective |", "|---|---|---|---:|---|---|---:|"]
        for M in modes:
            for o in M["options"]:
                d = SOL / c / M["id"] / o["id"].replace("+", "_")
                mans = [json.loads((d / f"seed_{s}" / "manifest.json").read_text()) for s in seeds if (d / f"seed_{s}" / "manifest.json").exists()]
                if not mans:
                    continue
                fails, obj = set(), None
                for m in mans:
                    for x in m["metrics"]:
                        if x.get("pass") == 0:
                            fails.add(x["id"])
                        if x["id"] == M["objective"] and x.get("value") is not None:
                            v = x["value"]
                            obj = v if obj is None else (min(obj, v) if M["objective"].endswith("share_last_orbit") else max(obj, v))
                feas = not fails and len(mans) == len(seeds)
                res.setdefault(c, {}).setdefault(M["id"], {})[o["id"]] = {"feasible": feas, "objective": obj, "failing": sorted(fails)}
                tw = twin.get((M["id"], o["id"]), {})
                tf = tw.get("feasible")
                tobj = tw.get("obj", tw.get("objective"))
                fmt = lambda x: "—" if x is None else (f"{x:.4g}" if isinstance(x, (int, float)) else str(x))
                L.append(f"| {M['label']} | {o['id']} | {'yes' if feas else 'no'} | {fmt(obj)} {M['objective']} | {', '.join(sorted(fails)) or '—'} | "
                         f"{'—' if tf is None else ('yes' if tf else 'no')} | {fmt(tobj)} |")
        L.append("")
    OUT.mkdir(exist_ok=True)
    (OUT / "engine_solutions.json").write_text(json.dumps(res, indent=1))
    (OUT / "ENGINE_SOLUTIONS.md").write_text("\n".join(L) + "\n")
    print("wrote results/ENGINE_SOLUTIONS.md")


def dispatch(a):
    sys.path.insert(0, str(ROOT / "tools"))
    import fswcfg
    for c in a.cases or ["ais_3u", "ais_img_3u"]:
        sj = ROOT / "matlab_sils" / "store" / "solutions" / c / "solution.json"
        if not sj.exists():
            print(f"{c}: no solution.json yet (asils.solution.collect) -- skipped"); continue
        S = json.loads(sj.read_text())
        fam = S["recommended"]
        E = S["families"][fam]
        dd = ROOT / "dist" / "dispatch" / c / fam / "fsw"
        dd.mkdir(parents=True, exist_ok=True)
        modes = {json.loads(f.read_text())["id"]: json.loads(f.read_text()) for f in MODES_DIR.glob("*.json")}
        meth = {m: E["methods"][m]["option"] for m in E["methods"]}
        opt = lambda m: next(o for o in modes[m]["options"] if o["id"] == meth[m])
        algs, dt = {}, 0.2
        for m in meth:
            o = opt(m)
            algs.update(o.get("algorithms", {}))
            dt = min(dt, o["dt_s"])
        det, acq = opt("detumble"), opt("sun_acquisition")
        fine = opt("nadir_pointing")
        a_ = 6378137 + case_value(c, "orbit.alt") * 1e3
        T = 2 * math.pi * math.sqrt(a_ ** 3 / 3.986004418e14)
        scen = {"schema": "adcs-scenario/1", "id": f"dispatch_{c}_{fam}", "case": c, "product": f"SZ-{c}-{fam}",
                "label": f"{c} — dispatched {fam}: {det['fsw_mode']} -> {acq['fsw_mode']} (auto), then {fine['fsw_mode']} by command",
                "time": {"duration_s": round(2 * T), "dt_s": dt, "record_dt_s": 1.0},
                "initial": {"attitude": {"kind": "random"}, "rate": {"kind": "random_direction", "magnitude_deg_s": "case:mission.w0"}},
                "fsw": {"start_mode": det["fsw_mode"], "auto_next": acq["fsw_mode"], "guidance": {"kind": "nadir"}, "algorithms": algs,
                        "rcs_dump": 1.0 if fine.get("dump") == "rcs" else 0.0, **({"dump_gain": 0.03} if fine["actuator"] == "fmr" else {})},
                "metrics": [{"id": "detumble_time", "kind": "time_to_rate", "rate_threshold_deg_s": 0.5, "hold_s": 600.0, "requirement": "req.detumble"},
                            {"id": "power_mean", "kind": "power_mean", "requirement": "req.pavg", "window": "all"}]}
        sp = dd / "mission_scenario.json"
        sp.write_text(json.dumps(scen, indent=1))
        case_csv = str(ROOT / "matlab_sils" / "cases" / f"{c}.csv")
        blob = dd / "adcs_fswcfg.bin"
        sh([str(BIN), "params", str(sp), "--case", case_csv, "--out", str(blob)])
        (dd / "adcs_fswcfg.json").write_text(json.dumps(fswcfg.decode(blob.read_bytes()), indent=1))
        res = {}
        for impl in ("c", "rust"):
            out = SOL / c / "dispatch" / impl
            p = subprocess.run([str(BIN), "run", str(sp), "--case", case_csv, "--fsw", impl, "--out", str(out), "--quiet"], capture_output=True, text=True)
            man = json.loads((out / "manifest.json").read_text()) if p.returncode == 0 else {}
            res[impl] = {"rc": p.returncode, "build_id": man.get("fsw", {}).get("build_id"), "metrics": man.get("metrics"), "mode_log": man.get("mode_log")}
        (dd / "engine_check.json").write_text(json.dumps(res, indent=1))
        (dd / "BUILD.md").write_text(f"""# Flight software for {c} / {fam}

Owner: Agastya. Generated by `tools/engine.py dispatch` from the recommended solution
(`matlab_sils/store/solutions/{c}/solution.json`).

- `adcs_fswcfg.bin` -- the adcs-fswcfg/1 blob the flight software boots from (CRC-32 protected);
  `adcs_fswcfg.json` is the same blob decoded (`python3 tools/fswcfg.py adcs_fswcfg.bin`).
- Mission sequence in the blob: `{det['fsw_mode']}` at boot, `{acq['fsw_mode']}` when detumble completes
  (auto), then `{fine['fsw_mode']}` by telecommand `01 {['detumble','nadir_mtq','nadir_fine','target_fine','slew_fine','spinup','sun_spin','detumble_rcs','sun_acq_rotor','sun_mtq','sun_fine'].index(fine['fsw_mode']):02x}`
  (set controller state). Methods per mode: {', '.join(f'{m} = {meth[m]}' for m in meth)}.
- Algorithms: {json.dumps(algs) if algs else 'registry defaults for the fitted hardware'}.

## Build for the OBC

C (the OBC reference build):

    make -C fsw                       # libadcs_fsw.a; link with your adcs_hal.h implementation
    adcs_fsw_init(&(adcs_fsw_init_t){{ADCS_FSW_ABI_VERSION, blob, blob_len, adcs_hal_time_ns()}});
    every tick: adcs_fsw_step(adcs_hal_time_ns());

Rust (same ABI, `no_std`):

    cd fsw-rs && cargo build --release --no-default-features --features cabi --target thumbv7em-none-eabihf

Both builds give bit-identical outputs for the same bytes (`adcs parity`).

## Engine check (SILS, 2 orbits, this blob)

`engine_check.json`: C and Rust flight software on the Rust engine, same scenario (`mission_scenario.json`).
Then OILS: the same blob on the OBC with the engine's device emulators on the wire (docs/OILS_HILS.md).
""")
        print(f"{c}: {fam} -> {dd.relative_to(ROOT)}  (engine check: " + ", ".join(f"{k} rc={v['rc']}" for k, v in res.items()) + ")")


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
    rows = {}
    for cid in ids:
        C = json.loads((CAMP / f"{cid}.json").read_text())
        base = ENG / "campaigns" / cid
        jobs = []
        draws = {}
        for k in range(1, C["runs"] + 1):
            sets, d = draw(C, k)
            draws[k] = d
            jobs.append((cid, C["scenario"], C["case"], C["seed"] + 7919 * k, k, sets, base / f"run_{k:04d}", a.fsw))
        t0 = time.time()
        with cf.ProcessPoolExecutor(a.jobs) as ex:
            for k, rc, txt in ex.map(camp_job, jobs):
                if rc:
                    print(f"[FAIL] {cid} run {k}: {txt.splitlines()[-1] if txt else ''}")
        runs = []
        for k in range(1, C["runs"] + 1):
            f = base / f"run_{k:04d}" / "manifest.json"
            if f.exists():
                m = json.loads(f.read_text())
                runs.append({"k": k, "metrics": m["metrics"] if isinstance(m["metrics"], list) else [m["metrics"]], "draws": draws[k], "wall_s": m.get("wall_s")})
        res = {"schema": "adcs-campaign-result/1", "owner": "Agastya", "id": cid, "scenario": C["scenario"], "case": C["case"],
               "type": C.get("type", "montecarlo"), "runs": len(runs), "engine": True, "fsw": a.fsw,
               "stats": summarise(runs), "per_run": runs, "wall_s": time.time() - t0}
        (base / "summary.json").write_text(json.dumps(res, indent=1))
        rows[cid] = res
        print(f"{cid}: {len(runs)}/{C['runs']} runs in {time.time() - t0:.0f} s wall")
    campaign_ledger()


def campaign_ledger():
    """results/ENGINE_CAMPAIGNS.md: every campaign, engine vs MATLAB twin, requirement metrics."""
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
    (OUT / "engine_campaigns.json").write_text(json.dumps(allrows, indent=1))
    (OUT / "ENGINE_CAMPAIGNS.md").write_text("\n".join(L) + "\n")
    print("wrote results/ENGINE_CAMPAIGNS.md")


# ---------------- soft OILS: SILS and the flight software on the virtual Cortex-M4F, side by side ----------------
def oils_job(args):
    scen, mode, fsw, out, extra = args
    cmd = [str(BIN), "run", scen, "--fsw", fsw, "--out", str(out), "--quiet"] + extra
    if mode == "oils":
        cmd.append("--oils")
    t0 = time.time()
    p = subprocess.run(cmd, capture_output=True, text=True)
    return scen, mode, p.returncode, time.time() - t0, (p.stdout + p.stderr).strip()


def oils(a):
    sh(["make", "-s", "-C", "fsw", "obc"])
    scen = scenarios(a.scenarios)
    base = ENG / "soft_oils"
    extra = ["--set", f"engine.duration_s={a.duration}"] if a.duration else []
    jobs = []
    for s in scen:
        jobs.append((s, "sils", "c", base / s / "sils", extra))
        jobs.append((s, "oils", a.fsw, base / s / "oils", extra))
    # longest first so the pool stays busy
    dur = {s: json.loads((DATA / f"{s}.json").read_text())["time"]["duration_s"] / json.loads((DATA / f"{s}.json").read_text())["time"]["dt_s"] for s in scen}
    jobs.sort(key=lambda j: -dur[j[0]] * (50 if j[1] == "oils" else 1))
    with cf.ProcessPoolExecutor(a.jobs) as ex:
        for s, mode, rc, dt, txt in ex.map(oils_job, jobs):
            print(f"[{'ok' if rc == 0 else 'FAIL'}] {s:22s} {mode:4s} {dt:7.1f} s wall", flush=True)
            if rc:
                print(txt[-2000:])
    oils_ledger(scen, a.fsw)


def oils_ledger(scen=None, fsw="qemu"):
    base = ENG / "soft_oils"
    scen = scen or sorted(p.name for p in base.iterdir() if (p / "oils" / "manifest.json").exists())
    fmt = lambda x: "—" if x is None else (f"{x:.4g}" if isinstance(x, (int, float)) else str(x))
    rows, tim = [], []
    for s in scen:
        a, b = base / s / "sils" / "manifest.json", base / s / "oils" / "manifest.json"
        if not (a.exists() and b.exists()):
            continue
        A, B = json.loads(a.read_text()), json.loads(b.read_text())
        mb = {m["id"]: m for m in B["metrics"]}
        for m in A["metrics"]:
            e = mb.get(m["id"], {})
            rows.append({"scenario": s, "metric": m["id"], "unit": m.get("unit", ""), "req": m.get("req"), "sils": m.get("value"), "oils": e.get("value"),
                         "pass_sils": m.get("pass"), "pass_oils": e.get("pass")})
        o = B.get("oils") or {}
        tim.append({"scenario": s, "dt_s": B.get("dt_s"), "wall_s": B.get("wall_s"), "duration_s": B.get("duration_s"), **{k: o.get(k) for k in
                    ("ticks", "overruns", "cpu_load_mean", "cpu_load_max", "deadline_margin_min_s")},
                    "insn_mean": (o.get("instructions") or {}).get("mean"), "insn_max": (o.get("instructions") or {}).get("max"),
                    "lat_mean_ms": 1e3 * o["latency_s"]["mean"] if o.get("latency_s") else None, "lat_max_ms": 1e3 * o["latency_s"]["max"] if o.get("latency_s") else None,
                    "exec_max_ms": 1e3 * o["exec_s"]["max"] if o.get("exec_s") else None, "model": o.get("model")})
    judged = [r for r in rows if r["pass_sils"] is not None]
    agree = sum(r["pass_sils"] == r["pass_oils"] for r in judged)
    model = next((t["model"] for t in tim if t.get("model")), {}) or {}
    L = ["# Soft OILS: SILS and the flight software on the virtual OBC, side by side", "",
         "Owner: Agastya. `tools/engine.py oils` flies every scenario twice on the Rust engine (POP in the loop):",
         "**SILS** with the C flight software in-process, and **soft OILS** with the flight software built for the",
         f"OBC (arm-none-eabi-gcc, Cortex-M4F hard-float) running as firmware in QEMU mps2-an386 (`--fsw {fsw}`)",
         "behind adcs-link/1. In soft OILS every command reaches the actuators only after the OBC has read its",
         "sensors on the buses, executed the step and sent its CAN frames; the previous command holds until then",
         "(docs/SOFT_OILS.md). The step's execution is its **exact** instruction count (QEMU plugin",
         f"fsw/targets/qemu-mps2/insn_count.c) x CPI {model.get('cpi', 1.25)} / {model.get('cpu_hz', 168e6) / 1e6:.0f} MHz;",
         f"buses: I2C {model.get('i2c_hz', 4e5) / 1e3:.0f} kHz, SPI {model.get('spi_hz', 1e6) / 1e6:.0f} MHz, CAN {model.get('can_bps', 1e6) / 1e6:.0f} Mbit/s.",
         "Runs are deterministic: the same scenario gives the same instruction counts and trajectory every time.", "",
         f"**Verdict agreement SILS vs soft OILS: {agree} of {len(judged)} judged metrics** over {len(tim)} scenarios.", "",
         "## OBC timing budget", "",
         "| scenario | period [ms] | instructions mean / max | exec max [ms] | latency mean / max [ms] | CPU load mean / max | overruns | deadline margin min [ms] |",
         "|---|---:|---:|---:|---:|---:|---:|---:|"]
    for t in tim:
        L.append(f"| {t['scenario']} | {1e3 * t['dt_s']:.0f} | {fmt(t['insn_mean'])} / {fmt(t['insn_max'])} | {fmt(t['exec_max_ms'])} | "
                 f"{fmt(t['lat_mean_ms'])} / {fmt(t['lat_max_ms'])} | {100 * (t['cpu_load_mean'] or 0):.1f} % / {100 * (t['cpu_load_max'] or 0):.1f} % | "
                 f"{t['overruns']} | {fmt(1e3 * t['deadline_margin_min_s'] if t['deadline_margin_min_s'] is not None else None)} |")
    L += ["", "## Metrics: SILS vs soft OILS", "", "| scenario | metric | req | SILS | soft OILS | SILS | soft OILS |", "|---|---|---:|---:|---:|---|---|"]
    v = lambda p: {1: "pass", 0: "FAIL"}.get(p, "—")
    for r in rows:
        if r["req"] is None:
            continue
        flag = "" if r["pass_sils"] == r["pass_oils"] else " ⚠"
        L.append(f"| {r['scenario']} | {r['metric']} ({r['unit']}) | {fmt(r['req'])} | {fmt(r['sils'])} | {fmt(r['oils'])} | {v(r['pass_sils'])} | {v(r['pass_oils'])}{flag} |")
    OUT.mkdir(exist_ok=True)
    (OUT / "soft_oils.json").write_text(json.dumps({"metrics": rows, "timing": tim}, indent=1))
    (OUT / "SOFT_OILS.md").write_text("\n".join(L) + "\n")
    print(f"soft OILS verdict agreement {agree}/{len(judged)}; wrote results/SOFT_OILS.md")


VOBC_PAIRS = [("c", "obc-posix"), ("rust", "obc-posix-rs"), ("c", "qemu"), ("rust", "qemu-rs"), ("c", "rust")]


def vobc(a):
    sh(["make", "-s", "-C", "fsw", "obc"])
    scen = a.scenarios or ["detumble_ais", "mission_ais", "fine_hold_img", "slew_img", "fine_hold_cmg", "fine_hold_fmr_rcs", "sun_spin_ais"]
    L = ["# Virtual OBC loop", "", "Owner: Agastya. `tools/engine.py vobc` -- the Rust engine drives the flight software over",
         "adcs-link/1 (fsw/targets/link) running as a separate host process and as bare-metal firmware on an",
         "emulated Cortex-M4F (QEMU mps2-an386, arm-none-eabi-gcc + newlib for C, rustc thumbv7em-none-eabihf for",
         f"Rust), and compares every recorded sample with the in-process build ({a.duration:.0f} s per scenario).", "",
         "| scenario | reference | virtual OBC | result | wall [s] |", "|---|---|---|---|---:|"]
    for s in scenario_list(scen):
        for ref, tgt in VOBC_PAIRS:
            t0 = time.time()
            p = subprocess.run([str(BIN), "parity", s, "--fsw", ref, "--against", tgt, "--set", f"engine.duration_s={a.duration}"],
                               capture_output=True, text=True)
            line = (p.stdout + p.stderr).strip().splitlines()[-1] if (p.stdout + p.stderr).strip() else "no output"
            res = "bit-identical" if "bit-identical" in line else re.sub(r".*: max", "max", line)
            print(f"{s:20s} {ref:5s} vs {tgt:13s} {res}")
            L.append(f"| {s} | {ref} (in-process) | {tgt} | {res} | {time.time() - t0:.1f} |")
    (OUT / "VIRTUAL_OBC.md").write_text("\n".join(L) + "\n")
    print("wrote results/VIRTUAL_OBC.md")


def scenario_list(x):
    return x


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    sp.add_parser("build").set_defaults(f=build)
    p = sp.add_parser("run"); p.add_argument("scenarios", nargs="*"); p.add_argument("--fsw", default="c")
    p.add_argument("--jobs", type=int, default=os.cpu_count()); p.add_argument("--seed", type=int, default=1); p.set_defaults(f=run)
    p = sp.add_parser("mc"); p.add_argument("scenario"); p.add_argument("--seeds", type=int, default=20); p.add_argument("--fsw", default="c")
    p.add_argument("--jobs", type=int, default=os.cpu_count()); p.set_defaults(f=mc)
    p = sp.add_parser("fsw-parity"); p.add_argument("scenarios", nargs="*"); p.add_argument("--duration", type=float, default=1800); p.set_defaults(f=fsw_parity)
    sp.add_parser("twin-parity").set_defaults(f=twin_parity)
    p = sp.add_parser("vobc"); p.add_argument("scenarios", nargs="*"); p.add_argument("--duration", type=float, default=600); p.set_defaults(f=vobc)
    p = sp.add_parser("dispatch"); p.add_argument("cases", nargs="*"); p.set_defaults(f=dispatch)
    p = sp.add_parser("campaign"); p.add_argument("ids", nargs="*"); p.add_argument("--fsw", default="c")
    p.add_argument("--jobs", type=int, default=os.cpu_count()); p.set_defaults(f=campaign)
    sp.add_parser("campaign-ledger").set_defaults(f=lambda a: campaign_ledger())
    p = sp.add_parser("oils"); p.add_argument("scenarios", nargs="*"); p.add_argument("--fsw", default="qemu")
    p.add_argument("--duration", type=float, default=None); p.add_argument("--jobs", type=int, default=os.cpu_count()); p.set_defaults(f=oils)
    sp.add_parser("oils-ledger").set_defaults(f=lambda a: oils_ledger())
    p = sp.add_parser("solutions"); p.add_argument("cases", nargs="*"); p.add_argument("--seeds", default="1,2"); p.add_argument("--fsw", default="c")
    p.add_argument("--jobs", type=int, default=os.cpu_count()); p.set_defaults(f=solutions)
    a = ap.parse_args()
    if a.cmd not in ("build",) and not BIN.exists():
        sys.exit("engine not built: python3 tools/engine.py build")
    a.f(a)


if __name__ == "__main__":
    main()
