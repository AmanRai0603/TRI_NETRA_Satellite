#!/usr/bin/env python3
"""Orchestrate the Rust SILS engine and the two flight-software builds.

Python is the upper layer: it builds, fans runs out over processes, gathers
results and writes the ledgers. The physics runs in Rust (engine/), the flight
software in C (fsw/) or Rust (fsw-rs/), all from one pseudocode (fsw/pseudocode).

  python3 tools/engine.py build                     make fsw (C tests) + cargo test/build fsw-rs and engine
  python3 tools/engine.py run [scen ...] [--fsw c|rust] [--jobs N] [--seed S]
  python3 tools/engine.py mc <scen> --seeds N [--fsw c|rust] [--jobs N]
  python3 tools/engine.py fsw-parity [scen ...] [--duration S]     C vs Rust flight software, same loop, same bytes
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
    a = ap.parse_args()
    if a.cmd not in ("build",) and not BIN.exists():
        sys.exit("engine not built: python3 tools/engine.py build")
    a.f(a)


if __name__ == "__main__":
    main()
