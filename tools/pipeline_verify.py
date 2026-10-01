"""The loop's verification nodes: select, dispatch, Monte Carlo, robustness, the Floquet certificate, every solution family's mission, soft OILS.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "tools"))
import json, math, os, shutil, subprocess, time
import engine as E
from common import write_bytes, write_text
from pipeline_base import BIN, LAMBDA_MAX, MS, P, PIPE, ROOT, SCALE_MAX, SLOT, UP, cls, jl_, split_alg, usable, write
from pipeline_design import node_key


def node_select(case, res, sizing, modes, families):
    req = sizing["demand"]["req"]
    out = {}
    for fa in families:
        acts = fa["actuators"] if isinstance(fa["actuators"], list) else [fa["actuators"]]
        bud = sizing["families"][fa["id"]]
        per_mode, gaps = {}, []
        for M in modes:
            cands = [r for (m, oid), r in res.items() if m == M["id"] and usable(next(x for x in M["options"] if x["id"] == oid), acts)]
            cands.sort(key=lambda z: (not z["feasible"], len(z["failing"]), z["objective"] if z["objective"] is not None else math.inf))
            best = cands[0] if cands else None
            per_mode[M["id"]] = best and {"option": best["option"], "alg": best["alg"], "feasible": best["feasible"], "failing": best["failing"],
                                          "objective": best["objective"], "objective_id": best["objective_id"], "algorithms": best["algorithms"],
                                          "metrics": best["metrics"]}
            if not best or not best["feasible"]:
                gaps.append(f"{M['id']}: " + (", ".join(f"{a} ({b})" for a, b in best["failing"].items()) if best else "no option"))
        # the case's requirements, then what the platform allocates (resources.*), each when stated
        for key, name in (("mass", "mass_kg"), ("vol", "volume_L"), ("malloc", "mass_kg"), ("valloc", "volume_L"), ("palloc", "power_W")):
            lim = req.get(key)
            if lim is not None and bud[name] > lim:
                gaps.append(f"budget: {name} {bud[name]:.3g} > {lim:g}" + (f" (resources.{key})" if key.endswith("alloc") else ""))
        out[fa["id"]] = {"role": fa["role"], "simplicity": fa.get("simplicity", 9), "label": fa["label"], "feasible": not gaps, "gaps": gaps,
                         "modes": per_mode, "budget": {k: bud[k] for k in ("mass_kg", "power_W", "volume_L")}, "product": bud["product"]}
    sp = P("select")
    rank_f = lambda f: tuple(out[f]["simplicity"] if k == "simplicity" else out[f]["budget"][k] for k in sp["rank_feasible"])
    rank_i = lambda f: tuple(len(out[f]["gaps"]) if k == "gap_count" else out[f]["budget"][k] for k in sp["rank_infeasible"])

    def pick_of(role):
        fs = [f for f in out if out[f]["role"] == role]
        feas = sorted((f for f in fs if out[f]["feasible"]), key=rank_f)
        if feas:
            return feas[0], "feasible", feas
        return (min(fs, key=rank_i) if fs else None), "closest (not feasible)", []

    pick, status, ranked = pick_of(sp["select_role"])
    bench, bstatus, branked = pick_of(sp["compare_role"])
    for i, f in enumerate(ranked):
        out[f]["rank"] = i + 1
    for i, f in enumerate(branked):
        out[f]["rank"] = i + 1
    return {"case": case, "selected": pick, "status": status, "benchmark": bench, "benchmark_status": bstatus,
            "rule": f"least {', then '.join(sp['rank_feasible'])} among feasible {sp['select_role']} families", "families": out}


def node_dispatch(case, sel, sized, modes, build, fam=None):
    """The selected family goes to dist/dispatch/<case>/<family>/converged; any other family (node
    family_missions) is packaged the same way under matlab_sils/store/pipeline/<case>/families/<family>/."""
    fam = fam or sel["selected"]
    chosen = fam == sel["selected"]
    F = sel["families"][fam]
    meth = {m: F["modes"][m]["option"] for m in F["modes"] if F["modes"][m]}
    Mby = {M["id"]: M for M in modes}
    opt = lambda m: next(o for o in Mby[m]["options"] if o["id"] == meth[m])
    algs, dt = {}, 0.2
    tune = {}
    for m in meth:
        algs.update({k: v for k, v in (F["modes"][m].get("algorithms") or {}).items() if k in (SLOT.get((m, meth[m])),)})
        if m in ("sun_acquisition", "nadir_pointing"):          # the mission flies these two: their tuned gains
            tune.update(split_alg(F["modes"][m].get("alg"))[1])
        dt = min(dt, opt(m)["dt_s"])
    det, acq, fine = opt("detumble"), opt("sun_acquisition"), opt("nadir_pointing")
    a_ = 6378137 + E.case_value(case, "orbit.alt") * 1e3
    T = 2 * math.pi * math.sqrt(a_ ** 3 / 3.986004418e14)
    # a coils-only nadir starts from the Sun spin: the despin and the magnetic capture take about two orbits
    DP = P("dispatch"); coils = "coils_only" if fine["fsw_mode"] == "nadir_mtq" else "rotors"
    scen = {"schema": "adcs-scenario/1", "id": f"dispatch_{case}_{fam}", "case": case, "product": f"SZ-{case}-{fam}",
            "label": f"{case} — dispatched {fam}: {det['fsw_mode']} -> {acq['fsw_mode']} (auto) -> {fine['fsw_mode']} (schedule)",
            "time": {"duration_s": round((DP["nadir_at_orbits"] + DP["nadir_orbits"][coils]) * T), "dt_s": dt, "record_dt_s": 1.0},
            "initial": {"attitude": {"kind": "random"}, "rate": {"kind": "random_direction", "magnitude_deg_s": "case:mission.w0"}},
            "fsw": {"start_mode": det["fsw_mode"], "auto_next": acq["fsw_mode"], "guidance": {"kind": "nadir"}, "algorithms": algs,
                    "schedule": [{"t_s": round(DP["nadir_at_orbits"] * T), "mode": fine["fsw_mode"]}],
                    "rcs_dump": 1.0 if fine.get("dump") == "rcs" else 0.0, **({"dump_gain": 0.03} if fine["actuator"] == "fmr" else {}), **tune},
            "metrics": [{"id": "detumble_time", "kind": "time_to_rate", "rate_threshold_deg_s": 0.5, "hold_s": 600.0, "end_at_mode_exit": True, "requirement": "req.detumble"},
                        {"id": "ape_los_p9973", "kind": "ape_los", "window": "last_half_orbit", "statistic": "p99.73", "requirement": "req.ape"},
                        {"id": "ake_los_p9973", "kind": "ake_los", "window": "last_half_orbit", "statistic": "p99.73", "requirement": "req.ake"},
                        {"id": "power_mean", "kind": "power_mean", "requirement": "req.pavg", "window": "all"},
                        {"id": "power_peak", "kind": "power_peak", "window": "all"},
                        {"id": "propellant", "kind": "propellant", "window": "all"}]}
    dd = ROOT / "dist" / "dispatch" / case / fam / "converged" if chosen else PIPE / case / "families" / fam / "package"
    chk = PIPE / case / "dispatch" if chosen else PIPE / case / "families" / fam / "dispatch"
    if dd.exists():
        shutil.rmtree(dd)
    (dd / "fsw").mkdir(parents=True)
    sp = dd / "mission_scenario.json"
    write_text(sp, json.dumps(scen, indent=1))
    for f in ("products", "parts"):
        shutil.copytree(sized / f, dd / "sized" / f)
    write_bytes(dd / "sized" / "sizing.json", (sized / "sizing.json").read_bytes())
    env = dict(os.environ, ADCS_SIZED_DIR=str(sized))
    case_csv = str(MS / "cases" / f"{case}.csv")
    blob = dd / "fsw" / "adcs_fswcfg.bin"
    subprocess.run([str(BIN), "params", str(sp), "--case", case_csv, "--out", str(blob)], cwd=MS, env=env, check=True, capture_output=True)
    import fswcfg
    write_text(dd / "fsw" / "adcs_fswcfg.json", json.dumps(fswcfg.decode(blob.read_bytes()), indent=1))
    res = {}
    for impl in ("c", "rust"):
        out = chk / impl
        p = subprocess.run([str(BIN), "run", str(sp), "--case", case_csv, "--fsw", impl, "--out", str(out), "--quiet"], cwd=MS, env=env, capture_output=True, text=True)
        man = json.loads((out / "manifest.json").read_text()) if p.returncode == 0 else {}
        res[impl] = {"rc": p.returncode, "build_id": man.get("fsw", {}).get("build_id"), "metrics": man.get("metrics"), "mode_log": man.get("mode_log")}
    same = False
    a, b = chk / "c" / "channels.csv", chk / "rust" / "channels.csv"
    if a.exists() and b.exists():
        same = a.read_bytes() == b.read_bytes()
    res["c_equals_rust_bitwise"] = same
    write_text(dd / "engine_check.json", json.dumps(res, indent=1))
    ms = ", ".join(f"{m} = {meth[m]}" for m in meth)
    sens = [f["slot"] for f in json.loads((sized / "products" / f"SZ-{case}-{fam}.json").read_text())["fill"]]
    write_text(dd / "BUILD.md", f"""# Flight configuration: {case} / {fam}

Owner: Agastya. Generated by `tools/pipeline.py` (node `dispatch`) from the converged design loop
(`matlab_sils/store/pipeline/{case}/selection.json`; this family: **{'selected, ' if chosen else 'not selected, '}{'feasible' if F['feasible'] else 'not feasible'}**).
The MATLAB solution pipeline's package for this family is the parent folder.

- `sized/`: the converged sized parts and products (adcs-design), `sizing.json` with the demand and the knobs.
- `fsw/adcs_fswcfg.bin`: the adcs-fswcfg/1 blob the flight software boots from; `fsw/adcs_fswcfg.json` decodes it.
- `mission_scenario.json`: {det['fsw_mode']} at boot, {acq['fsw_mode']} when detumble completes, {fine['fsw_mode']} by schedule at 2 orbits.
- Methods per mode: {ms}. Algorithms chosen by the loop: {json.dumps(algs) if algs else 'registry defaults'}.
- Units fitted: {', '.join(sens)}.

Build: `make -C fsw` (C, libadcs_fsw.a) or `cd fsw-rs && cargo build --release --no-default-features --features cabi
--target thumbv7em-none-eabihf` (Rust, same ABI). C and Rust on the engine, same blob: bit-identical = {same}.
Soft OILS of this configuration: `results/DESIGN_{case}.md`. Real OILS: `adcs run mission_scenario.json --fsw tcp:<obc>:<port> --realtime`.
""")
    print(f"  dispatch: {fam} -> {dd.relative_to(ROOT)} (C = Rust bitwise: {same})")
    return {"family": fam, "selected": chosen, "dir": str(dd.relative_to(ROOT)), "check_dir": str(chk.relative_to(ROOT)), "scenario": str(sp),
            "check": res, "methods": meth, "algorithms": algs}


def node_mc(case, disp, sized, runs, jobs, base=None):
    """Monte Carlo of the dispatched mission: the case's Monte Carlo dispersions (the MC campaign
    of the case), per-run seeds, on the converged product."""
    camp = {"ais_3u": "mc_nadir_ais", "ais_img_3u": "mc_fine_img"}.get(case)
    C = json.loads((E.CAMP / f"{camp}.json").read_text()) if camp else {"dispersions": []}
    C = {"id": f"mc_dispatch_{case}", "case": case, "seed": C.get("seed", 1), "runs": runs, "type": "montecarlo",
         "dispersions": [d for d in (C["dispersions"] if isinstance(C["dispersions"], list) else [C["dispersions"]]) if d["kind"] != "initial_error_deg"]}
    base = base or PIPE / case / "mc"
    if base.exists():                   # a Monte Carlo starts empty: no earlier run is read as this one's
        shutil.rmtree(base)
    env_dir = str(sized)
    jobs_ = []
    draws = {}
    for k in range(1, runs + 1):
        sets, d = E.draw(C, k)
        draws[k] = d
        jobs_.append((k, sets, base / f"run_{k:04d}"))

    def one(j):
        k, sets, out = j
        cmd = [str(BIN), "run", disp["scenario"], "--case", str(MS / "cases" / f"{case}.csv"), "--seed", str(C["seed"] + 7919 * k), "--out", str(out), "--quiet"]
        for s in sets:
            cmd += ["--set", s]
        return k, subprocess.run(cmd, cwd=MS, env=dict(os.environ, ADCS_SIZED_DIR=env_dir), capture_output=True, text=True).returncode

    import concurrent.futures as tf
    bad = set()
    with tf.ThreadPoolExecutor(jobs) as ex:
        for k, rc in ex.map(one, jobs_):
            if rc:
                bad.add(k)
                print(f"  [FAIL] mc run {k}")
    rr = []
    for k in range(1, runs + 1):
        f = base / f"run_{k:04d}" / "manifest.json"
        if k in bad or not f.exists():      # a run that failed to fly counts as failed, never dropped
            rr.append({"k": k, "failed": True, "metrics": [], "draws": draws[k]})
        else:
            rr.append({"k": k, "metrics": json.loads(f.read_text())["metrics"], "draws": draws[k]})
    nfail = sum(1 for r in rr if r.get("failed"))
    res = {"schema": "adcs-campaign-result/1", "id": C["id"], "case": case, "runs": len(rr) - nfail, "failed_runs": nfail,
           "dispersions": [d["kind"] for d in C["dispersions"]],
           "stats": E.summarise(rr), "per_run": rr}
    write(base / "summary.json", res)
    print(f"  mc: {len(rr) - nfail}/{runs} runs of the dispatched mission flown, {nfail} failed" + ("" if disp.get("selected", True) else f" ({disp['family']})"))
    return res


def node_robust(sel, fails, knobs, history):
    """Node mc's feedback: the Monte Carlo of the dispatched mission fails a requirement in some
    dispersed run. The selected family's lever for that failure class is moved and the lever that
    works against it is closed, then the loop runs on from these knobs."""
    k = json.loads(json.dumps(knobs))
    f = sel["selected"]
    changes, blocked = [], []
    kinds = {cls(x["id"]) for x in fails}
    if "power" in kinds:
        lam = k.get("fmr_lambda", 0.1)
        if "fmr" in f and lam < LAMBDA_MAX:
            k["fmr_lambda"] = min(LAMBDA_MAX, lam * 3)
            history["_lam_up"] = True; history["_closed_fmr_lambda"] = True
            changes.append(f"robustness ({f}): mean power fails in dispersed runs -> pump with more copper (lambda {lam:g} -> {k['fmr_lambda']:g} kg/W); lighter-pump lever closed")
        else:
            blocked.append(f"robustness ({f}): power fails in dispersed runs and no power lever is left")
    if "performance" in kinds:
        part = "fmr" if "fmr" in f else "mtqp" if f == "mtq" else None
        s0 = k.setdefault("scale", {}).get(part, 1.0) if part else None
        if part and s0 < SCALE_MAX:
            k["scale"][part] = min(SCALE_MAX, s0 * UP); history[f"_closed_scale.{part}"] = True
            changes.append(f"robustness ({f}): performance fails in dispersed runs -> {part} authority x{s0:g} -> x{k['scale'][part]:g}")
        else:
            blocked.append(f"robustness ({f}): performance fails in dispersed runs and no authority is left")
    if "knowledge" in kinds:
        if not k.get("star_tracker") and sel["class"] != "fine":
            k["star_tracker"] = True; changes.append(f"robustness ({f}): knowledge fails in dispersed runs -> fit the star tracker")
        else:
            blocked.append(f"robustness ({f}): knowledge fails in dispersed runs with the star tracker fitted")
    return k, changes, blocked


def node_certify(case):
    """Node certify (Celani 2026's method): Floquet multipliers of the coils-only nadir loop, every law."""
    import floquet
    r = floquet.certify(case)
    if r:
        print("  certify: " + ", ".join(f"{x['law']} |mu| {x['max_mu']:.3g}{'*' if x['dispatched'] else ''}" for x in r["laws"]))
    return r


def node_family_missions(case, sel, sized, modes, build, runs, jobs, disp, mc):
    """Every solution family, selected or not, flown as the dispatched mission (C and Rust) and by
    Monte Carlo, so each one's behaviour is on record: the selected one reuses its dispatch and mc."""
    out = {}
    for f, F in sel["families"].items():
        if F["role"] != P("family_missions")["role"]:
            continue
        if f == sel["selected"]:
            d, m = disp, mc
        else:
            d = node_dispatch(case, sel, sized, modes, build, fam=f)
            m = node_mc(case, d, sized, runs, jobs, base=PIPE / case / "families" / f / "mc") if runs else None
        out[f] = {"selected": f == sel["selected"], "feasible": F["feasible"], "gaps": F["gaps"], "budget": F["budget"],
                  "methods": d["methods"], "algorithms": d["algorithms"], "package": d["dir"], "check_dir": d["check_dir"],
                  "c_equals_rust_bitwise": d["check"]["c_equals_rust_bitwise"],
                  "mission": {impl: d["check"][impl]["metrics"] for impl in ("c", "rust")},
                  "mode_log": d["check"]["c"].get("mode_log"),
                  "mc": m and {"runs": m["runs"], "stats": [{k: s_[k] for k in ("id", "req", "mean", "std", "min", "max", "pass_rate", "pass")} for s_ in m["stats"]]}}
    write(PIPE / case / "families.json", out)
    return out


def node_soft_oils(case, disp, sized, build):
    key = node_key(disp, sized, build, "soft_oils")
    prev = jl_(PIPE / case / "soft_oils.json")
    if prev and prev.get("_key") == key:
        print("  soft_oils: unchanged configuration, cached")
        return prev
    out = {"_key": key}
    env = dict(os.environ, ADCS_SIZED_DIR=str(sized))

    def one(job):
        mode, fsw, extra = job
        d = PIPE / case / "soft_oils" / mode
        t0 = time.time()
        p = subprocess.run([str(BIN), "run", disp["scenario"], "--case", str(MS / "cases" / f"{case}.csv"), "--fsw", fsw, "--out", str(d), "--quiet"] + extra,
                           cwd=MS, env=env, capture_output=True, text=True)
        man = json.loads((d / "manifest.json").read_text()) if p.returncode == 0 else {}
        print(f"  soft_oils {mode}: rc {p.returncode} in {time.time() - t0:.0f} s", flush=True)
        return mode, {"rc": p.returncode, "wall_s": time.time() - t0, "metrics": man.get("metrics"), "oils": man.get("oils"), "fsw": man.get("fsw")}

    import concurrent.futures as tf
    with tf.ThreadPoolExecutor(3) as ex:
        for mode, r in ex.map(one, (("sils", "c", []), ("oils", "qemu", ["--oils"]), ("oils_rs", "qemu-rs", ["--oils"]))):
            out[mode] = r
    write(PIPE / case / "soft_oils.json", out)
    return out
