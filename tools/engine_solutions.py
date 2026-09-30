"""The customer-case solution matrix and the dispatch of the recommended solution.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import concurrent.futures as cf, json, math, subprocess, sys, time
from common import sh, write_text
from engine_base import BIN, OUT, ROOT


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
                write_text(sp, json.dumps(mode_scenario(c, M, o), indent=1))
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
    write_text(OUT / "engine_solutions.json", json.dumps(res, indent=1))
    write_text(OUT / "ENGINE_SOLUTIONS.md", "\n".join(L) + "\n")
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
        write_text(sp, json.dumps(scen, indent=1))
        case_csv = str(ROOT / "matlab_sils" / "cases" / f"{c}.csv")
        blob = dd / "adcs_fswcfg.bin"
        sh([str(BIN), "params", str(sp), "--case", case_csv, "--out", str(blob)])
        write_text(dd / "adcs_fswcfg.json", json.dumps(fswcfg.decode(blob.read_bytes()), indent=1))
        res = {}
        for impl in ("c", "rust"):
            out = SOL / c / "dispatch" / impl
            p = subprocess.run([str(BIN), "run", str(sp), "--case", case_csv, "--fsw", impl, "--out", str(out), "--quiet"], capture_output=True, text=True)
            man = json.loads((out / "manifest.json").read_text()) if p.returncode == 0 else {}
            res[impl] = {"rc": p.returncode, "build_id": man.get("fsw", {}).get("build_id"), "metrics": man.get("metrics"), "mode_log": man.get("mode_log")}
        write_text(dd / "engine_check.json", json.dumps(res, indent=1))
        write_text(dd / "BUILD.md", f"""# Flight software for {c} / {fam}

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
