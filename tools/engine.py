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
    sh(["cargo", "build", "--release", "-q", "--no-default-features", "--features", "cabi"], cwd=ROOT / "fsw-rs")
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
     "Engine Monte Carlo (20 seeds): mean 0.034 deg, range 0.010-0.082 deg; the MATLAB run sits at the lucky end. "
     "Finding: RCS dumping during imaging breaks the 0.01 deg APE for most thrust-scale draws -> calibrate thrust "
     "on orbit or inhibit RCS dumping in the fine modes."),
    ("sun_spin_ais · sun_angle, spin_rate_error, share",
     "Traced to the flight logic (identical in MATLAB, C and Rust): when spin-up converges with the Sun on +Z, the "
     "sigma flip reverses the spin every 60 s check, which drives the body through zero spin and never moves the Sun "
     "to -Z. Engine seed 1 is that case (1 of 20 seeds); the 20-seed mean sun angle is 23 deg (range 2-174 deg), the "
     "MATLAB run (6.5 deg) is inside. Finding: the sign-flip guard needs a hemisphere manoeuvre, not a spin reversal."),
    ("nadir_hold_ais, mission_ais · ake_los / ape_los",
     "Realisation spread of the coils-only MEKF (magnetometer + Sun): engine 24-seed AKE mean 4.3 deg (2.0-7.4 deg); "
     "the MATLAB runs (3.9-4.0 deg) are inside the distribution."),
    ("detumble_* and mission_* · detumble_time",
     "Random initial tumble direction; at distribution level the MATLAB campaign (48.5 +/- 12 min) and the engine "
     "Monte Carlo (53 +/- 7.6 min) agree (table above)."),
    ("mc_nadir_ais vs engine nadir_hold_ais · ape_los",
     "Not a like-for-like pair: the MATLAB campaign also disperses the residual dipole (0.5-2x), inertia and CM "
     "offset, which a coils-only nadir hold cannot absorb (MATLAB mean 90 deg); the engine Monte Carlo disperses the "
     "units only (10.5 deg), matching the nominal MATLAB run (7.2 deg). Environment dispersions in the engine's "
     "Monte Carlo are the next step for this pair."),
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
         "One realisation per side; the random streams differ by design (MATLAB Mersenne twister vs the",
         "engine's counter-based SplitMix64), and the engine's orbit/ephemeris/density models are the",
         "analytic ones listed in `fsw/twin_map.toml`, not POP/DE440/DTM2020. So values are compared as",
         "a ratio and the requirement verdicts for agreement; Monte Carlo on both sides is the",
         "distribution-level check (`tools/engine.py mc`).", "",
         f"**Verdict agreement: {agree} of {len(judged)} judged metrics** "
         f"({len(rows)} metrics over {len(walls)} scenarios).", "",
         "| scenario | metric | MATLAB | engine | engine/MATLAB | req | MATLAB | engine |",
         "|---|---|---:|---:|---:|---:|---|---|"]
    for r in rows:
        flag = "" if r["agree"] in (None, True) else " ⚠"
        L.append(f"| {r['scenario']} | {r['metric']} ({r['unit']}) | {fmt(r['matlab'])} | {fmt(r['engine'])} | {fmt(r['ratio'])} | "
                 f"{fmt(r['req'])} | {verdict(r['pass_matlab'])} | {verdict(r['pass_engine'])}{flag} |")
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
            "initial": {"attitude": M["test"]["attitude"], "rate": M["test"]["rate"]}, "fsw": fsw, "metrics": ms}


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
                tobj = tw.get("objective", tw.get("worst"))
                fmt = lambda x: "—" if x is None else (f"{x:.4g}" if isinstance(x, (int, float)) else str(x))
                L.append(f"| {M['label']} | {o['id']} | {'yes' if feas else 'no'} | {fmt(obj)} {M['objective']} | {', '.join(sorted(fails)) or '—'} | "
                         f"{'—' if tf is None else ('yes' if tf else 'no')} | {fmt(tobj)} |")
        L.append("")
    OUT.mkdir(exist_ok=True)
    (OUT / "engine_solutions.json").write_text(json.dumps(res, indent=1))
    (OUT / "ENGINE_SOLUTIONS.md").write_text("\n".join(L) + "\n")
    print("wrote results/ENGINE_SOLUTIONS.md")


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
    p = sp.add_parser("solutions"); p.add_argument("cases", nargs="*"); p.add_argument("--seeds", default="1,2"); p.add_argument("--fsw", default="c")
    p.add_argument("--jobs", type=int, default=os.cpu_count()); p.set_defaults(f=solutions)
    a = ap.parse_args()
    if a.cmd not in ("build",) and not BIN.exists():
        sys.exit("engine not built: python3 tools/engine.py build")
    a.f(a)


if __name__ == "__main__":
    main()
