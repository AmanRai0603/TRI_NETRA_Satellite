#!/usr/bin/env python3
"""The design loop, node by node: from a customer case to a selected, dispatched, verified ADCS.

    case -> [size] -> [matrix] -> [assess] -> [converge] --not converged: resize / upgrade--> [size] ...
                                                  | converged
                                               [select] -> [dispatch] -> [mc] -> [soft_oils] -> report

  size       adcs size (Rust, adcs-design): demand survey on the POP orbit, every actuator option
             sized with the current knobs (authority scales, margins, pump type, star tracker)
  matrix     every mission mode x option x seed flown on the Rust engine with the sized products
             (ADCS_SIZED_DIR); failing options also fly every algorithm of their slot
  assess     per option: feasible on every seed? failing requirements -> cause class
             (performance, knowledge, power, propellant)
  converge   knob changes the failures call for: more authority for a performance failure,
             the star tracker for a knowledge failure, more pump copper (lambda) or less
             authority for a power failure, one star-tracker head / a lighter pump / less
             fluid-loop momentum for a mass gap (undone if it breaks a mode); blocked when a part is at its bound or a
             performance/power conflict is found. Converged when nothing is left to change.
  select     the simplest SOLUTION family (mtq -> mtq_fmr -> mtq_fmr_rcs) whose best option
             passes every mode and whose budget meets req.mass / req.vol; benchmarks scored alike
  dispatch   the selected family's flight configuration (adcs-fswcfg/1 blob) + C and Rust engine check
  mc         Monte Carlo of the dispatched mission (case dispersions, per-run seeds)
  soft_oils  the dispatched mission with the flight software as Cortex-M4F firmware (QEMU),
             exact instruction timing, next to its SILS run

Every node writes matlab_sils/store/pipeline/<case>/<node>.json; mode tests are cached by the
hash of (scenario, product, parts, seed, engine build), so an iteration flies only what changed.

  python3 tools/pipeline.py [case ...] [--seeds 1,2] [--max-iter 5] [--jobs N] [--mc-runs 12] [--no-oils]

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse, concurrent.futures as cf, hashlib, json, math, os, pathlib, shutil, subprocess, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import engine as E  # noqa: E402

MS = ROOT / "matlab_sils"
BIN = E.BIN
PIPE = MS / "store" / "pipeline"
CACHE = PIPE / "cache"
OUT = ROOT / "results"

# which algorithm slot decides an option's performance, and the registry's candidates for it
SLOT = {("detumble", "mtq"): "detumble", ("sun_acquisition", "mtq"): "sun_acquisition"}
for m in ("sun_referencing", "nadir_pointing"):
    SLOT[(m, "mtq")] = "mtq_pointing"
    for o in ("rw", "cmg", "vscmg", "fmr"):
        for d in ("mtq", "rcs"):
            SLOT[(m, f"{o}+{d}")] = "pointing"
CANDIDATES = {"detumble": ["bdot_gyro", "bdot_mag", "bdot_bangbang", "genbdot_l1"],
              "sun_acquisition": ["sunspin_damped", "sunspin_l1l2", "sunspin_l1l2_e2"],
              "mtq_pointing": ["mtq_pd", "mtq_lqr", "mtq_smc", "mtq_rate_damp"],
              "pointing": ["pid", "lqr", "smc", "pid@bw2.5", "pid@bw4"]}
# a candidate "law@bwX" is the law with its pointing bandwidth tuned to X rad/s (fsw.rw_bandwidth)
def split_alg(a):
    if a and "@bw" in a:
        law, bw = a.split("@bw")
        return law, {"rw_bandwidth": float(bw)}
    return a, {}
# the sized part that gives an option its authority
def auth_part(mode, o):
    a = o["actuator"]
    return {"mtq": "mtqp", "rw": "rw", "cmg": "cmg", "vscmg": "vscmg", "fmr": "fmr", "rcs": "rcs"}[a]
SCALE_MIN, SCALE_MAX, UP, DOWN = 0.5, 4.0, 1.5, 0.75
LAMBDA_MIN, LAMBDA_MAX = 0.01, 3.0
FLOW_SIGMA_MIN = 0.00005
GYRO_MIN = 0.1
FAMILIES = []


def cls(metric):
    if metric.startswith("power"):
        return "power"
    if metric.startswith("ake"):
        return "knowledge"
    if metric.startswith("propellant"):
        return "propellant"
    return "performance"


def sha(*parts):
    h = hashlib.sha1()
    for p in parts:
        h.update(p if isinstance(p, bytes) else json.dumps(p, sort_keys=True).encode())
    return h.hexdigest()[:16]


def write(p, obj):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(obj, indent=1))


# ---------------------------------------------------------------- nodes
def node_size(case, it, knobs):
    d = PIPE / case / f"iter_{it}"
    kf = d / "knobs.json"
    write(kf, knobs)
    sized = d / "sized"
    if sized.exists():
        shutil.rmtree(sized)
    p = subprocess.run([str(BIN), "size", case, "--knobs", str(kf), "--out", str(sized)], cwd=MS, capture_output=True, text=True)
    if p.returncode:
        raise SystemExit(f"size {case}: {p.stderr}")
    print("  " + p.stdout.strip().splitlines()[0])
    return sized, json.loads((sized / "sizing.json").read_text())


def product_blob(sized, pid):
    pr = json.loads((sized / "products" / f"{pid}.json").read_text())
    parts = {}
    for f in pr["fill"]:
        q = sized / "parts" / f"{f['part']}.json"
        if q.exists():
            parts[f["part"]] = json.loads(q.read_text())
    return {k: v for k, v in pr.items() if k != "knobs"}, parts


def run_job(args):
    key, scen_path, case, seed, fsw, sized = args
    out = CACHE / key
    if (out / "manifest.json").exists():
        return key, 0, "cached"
    env = dict(os.environ, ADCS_SIZED_DIR=str(sized))
    p = subprocess.run([str(BIN), "run", str(scen_path), "--case", str(MS / "cases" / f"{case}.csv"), "--seed", str(seed),
                        "--fsw", fsw, "--out", str(out), "--quiet"], cwd=MS, capture_output=True, text=True, env=env)
    (out / "channels.csv").unlink(missing_ok=True)      # the loop reads the manifest; the channels cost GBs over a run
    return key, p.returncode, (p.stdout + p.stderr).strip()[-500:]


def node_matrix(case, it, sized, modes, variants_on, seeds, jobs, build):
    tests, runs = [], []
    for M in modes:
        for o in M["options"]:
            base = E.mode_scenario(case, M, o)
            slot = SLOT.get((M["id"], o["id"]))
            algs = [None]
            if slot and (M["id"], o["id"]) in variants_on:
                algs = CANDIDATES[slot]
            for alg in algs:
                s = json.loads(json.dumps(base))
                if alg:
                    law, tune = split_alg(alg)
                    s["fsw"].setdefault("algorithms", {})[slot] = law
                    s["fsw"].update(tune)
                    s["id"] = f"{s['id']}__{alg.replace('@', '_')}"
                prod, parts = product_blob(sized, s["product"])
                d = PIPE / case / f"iter_{it}" / "scenarios"
                d.mkdir(parents=True, exist_ok=True)
                sp = d / f"{s['id']}.json"
                sp.write_text(json.dumps(s, indent=1))
                keys = []
                for sd in seeds:
                    k = sha(s, prod, parts, sd, "c", build)
                    keys.append(k)
                    runs.append((k, sp, case, sd, "c", sized))
                tests.append({"mode": M["id"], "option": o["id"], "alg": alg, "slot": slot, "keys": keys, "product": s["product"]})
    todo = [r for r in runs if not (CACHE / r[0] / "manifest.json").exists()]
    t0 = time.time()
    with cf.ProcessPoolExecutor(jobs) as ex:
        for k, rc, txt in ex.map(run_job, todo):
            if rc:
                print(f"  [FAIL] {k}: {txt.splitlines()[-1] if txt else ''}")
    print(f"  matrix: {len(runs)} mode tests ({len(todo)} flown, {len(runs) - len(todo)} cached) in {time.time() - t0:.0f} s")
    return tests


def node_assess(tests, modes):
    Mby = {M["id"]: M for M in modes}
    res = {}
    for t in tests:
        M = Mby[t["mode"]]
        mans = [json.loads((CACHE / k / "manifest.json").read_text()) for k in t["keys"] if (CACHE / k / "manifest.json").exists()]
        fails, obj, metrics, viol = {}, None, {}, {}
        for m in mans:
            for x in m["metrics"]:
                metrics.setdefault(x["id"], []).append(x.get("value"))
                if x.get("pass") == 0:
                    fails[x["id"]] = cls(x["id"])
                    v, q = x.get("value"), x.get("req")
                    over = (v / q - 1) if isinstance(v, (int, float)) and isinstance(q, (int, float)) and q > 0 and math.isfinite(v) else 10.0
                    viol[x["id"]] = max(viol.get(x["id"], 0.0), over)
                if x["id"] == M["objective"] and x.get("value") is not None:
                    v = x["value"]
                    obj = v if obj is None else max(obj, v)
        feas = not fails and len(mans) == len(t["keys"])
        worst = {k: (max((v for v in vs if v is not None), default=None)) for k, vs in metrics.items()}
        r = {**t, "feasible": feas, "failing": fails, "violation": viol, "objective": obj, "objective_id": M["objective"], "metrics": worst,
             "algorithms": (mans[0].get("algorithms") if mans else {})}
        key = (t["mode"], t["option"])
        cur = res.get(key)
        rank = lambda z: (not z["feasible"], len(z["failing"]), z["objective"] if z["objective"] is not None else math.inf)
        if cur is None or rank(r) < rank(cur):
            res[key] = r
    return res


def node_converge(case, res, knobs, variants_on, history, fine, modes, sel=None):
    """The knob changes the failures call for. Returns (new knobs, new variants, changes, blocked)."""
    k = json.loads(json.dumps(knobs))
    v = set(variants_on)
    changes, blocked = [], []
    Mby = {M["id"]: M for M in modes}
    # did the last authority increase help? performance violation of the options each part serves, before vs now
    lg = history.pop("_last_gyro", None)
    if lg:
        g0, vb = lg
        vn = rate_violation(res)
        if vn > 0.95 * vb:
            k["gyro_grade"] = g0; history["_closed_gyro_grade"] = True
            changes.append(f"gyro grade back to x{g0:g}: rate-stability violation {vb:.3g} -> {vn:.3g}")
    frozen = history.setdefault("_frozen", {})
    for part, (prev_scale, prev_res) in list(history.pop("_last_up", {}).items()):
        before = now = 0.0
        for key, r0 in prev_res.items():
            o = next(x for x in Mby[key[0]]["options"] if x["id"] == key[1])
            if auth_part(key[0], o) != part or r0["feasible"]:
                continue
            perf = lambda z: sum(val for mid, val in z.get("violation", {}).items() if cls(mid) == "performance")
            before += perf(r0); now += perf(res.get(key, r0))
        if before > 0 and now > 0.95 * before:
            k.setdefault("scale", {})[part] = prev_scale
            frozen[part] = f"more authority did not reduce the performance violation ({before:.3g} -> {now:.3g}); kept at x{prev_scale:g}"
            changes.append(f"{part}: authority back to x{prev_scale:g} (no improvement)")
    want = {}                                  # part -> "up" | "down"
    for (mode, oid), r in sorted(res.items()):
        if r["feasible"]:
            continue
        o = next(x for x in Mby[mode]["options"] if x["id"] == oid)
        kinds = set(r["failing"].values())
        part = auth_part(mode, o)
        if "performance" in kinds:
            if r["slot"] and (mode, oid) not in v:
                v.add((mode, oid)); changes.append(f"{mode}/{oid}: fly every {r['slot']} algorithm")
            elif "power" in kinds:
                blocked.append(f"{mode}/{oid}: performance and power both fail — no authority change helps")
            elif fine and any(m.startswith("rate_stability") for m in r["failing"]) and k.get("gyro_grade", 1.0) > GYRO_MIN * 1.01 \
                    and not history.get("_closed_gyro_grade"):
                if k.get("gyro_grade", 1.0) == knobs.get("gyro_grade", 1.0):
                    g0 = k.get("gyro_grade", 1.0); k["gyro_grade"] = max(GYRO_MIN, round(g0 * 0.3, 3))
                    history["_last_gyro"] = (g0, rate_violation(res))
                    changes.append(f"rate stability: gyro noise x{g0:g} -> x{k['gyro_grade']:g} (fibre-optic class)")
            elif o["actuator"] == "fmr" and k.get("fmr_flow_sigma", 0.002) > FLOW_SIGMA_MIN * 1.01:
                sg = k.get("fmr_flow_sigma", 0.002)
                if sg == knobs.get("fmr_flow_sigma", 0.002):
                    k["fmr_flow_sigma"] = max(FLOW_SIGMA_MIN, sg / 4)
                    changes.append(f"{mode}/{oid}: fluid-loop flow sensor {sg * 1e3:g} -> {k['fmr_flow_sigma'] * 1e3:g} mm/s (1 sigma)")
            else:
                want.setdefault(part, "up")
        if "knowledge" in kinds:
            if not fine and not k.get("star_tracker"):
                k["star_tracker"] = True; changes.append(f"{mode}/{oid}: knowledge -> fit the star tracker")
            else:
                blocked.append(f"{mode}/{oid}: knowledge fails with the star tracker fitted")
        if "power" in kinds:
            lam = k.get("fmr_lambda", 0.1)
            if o["actuator"] == "fmr" and k.get("fmr_lambda", 0.1) != knobs.get("fmr_lambda", 0.1):
                pass                                   # already raised this iteration
            elif o["actuator"] == "fmr" and o.get("dump") == "rcs":
                blocked.append(f"{mode}/{oid}: power is the thrusters' valve power (RCS dumping), not the pump")
            elif o["actuator"] == "fmr" and lam < LAMBDA_MAX and not history.get("_lam_down"):
                k["fmr_lambda"] = min(LAMBDA_MAX, lam * 3); history["_lam_up"] = True
                changes.append(f"{mode}/{oid}: power -> electromagnetic pump with more copper (lambda {lam:g} -> {k['fmr_lambda']:g} kg/W)")
            elif o["actuator"] == "fmr":
                blocked.append(f"{mode}/{oid}: power fails with the pump at lambda {lam:g} kg/W" + (" (mass needs it lower: conflict)" if history.get("_lam_down") else " (bound)"))
            elif "performance" not in kinds and part in ("mtqp",):
                want.setdefault(part, "down")
            elif "performance" not in kinds:
                blocked.append(f"{mode}/{oid}: power fails at the sized authority ({part}); the part's standby power is the floor")
        if "propellant" in kinds:
            want["rcs"] = "up"
    # a mass move that broke an option which passed before is undone, and that lever is closed
    lm = history.pop("_last_mass", None)
    if lm:
        key, old, before, acts, v_before = lm
        fam_opts = {kk: r for kk, r in res.items() if usable(next(x for x in Mby[kk[0]]["options"] if x["id"] == kk[1]), acts)}
        lost = sorted(f"{m}/{o}" for (m, o) in before if (m, o) in fam_opts and not fam_opts[(m, o)]["feasible"])
        v_now = fam_violation(fam_opts, modes)
        if v_now > 1.05 * v_before + 1e-9:
            lost.append(f"requirement violation {v_before:.3g} -> {v_now:.3g}")
        if lost:
            if key == "scale.fmr":
                k.setdefault("scale", {})["fmr"] = old
            else:
                k[key] = old
            history[f"_closed_{key}"] = True
            changes.append(f"mass lever {key} undone: it broke {', '.join(lost)}")
    closed = lambda key: history.get(f"_closed_{key}")
    # mass budget of the solution families (the closest one first): one star-tracker head, a
    # lighter pump (lower lambda) when power allows, less fluid-loop authority when performance allows
    if sel and not changes:
        order = sorted((f for f, x in sel["families"].items() if x["role"] == "solution"), key=lambda f: (len(sel["families"][f]["gaps"]), sel["families"][f]["simplicity"]))
        for f in order[:1]:
            g = " ".join(sel["families"][f]["gaps"])
            if "mass_kg" not in g:
                continue
            fam_has_fmr = "fmr" in f
            acts = next(fa for fa in FAMILIES if fa["id"] == f)["actuators"]
            fam_opts = {kk: r for kk, r in res.items() if usable(next(x for x in Mby[kk[0]]["options"] if x["id"] == kk[1]), acts)}
            feas_now = [key for key, r in fam_opts.items() if r["feasible"]]
            vb = fam_violation(fam_opts, modes)
            if fine and k.get("gyro_grade", 1.0) < 0.999 and not closed("gyro_grade"):
                g0 = k["gyro_grade"]; history["_last_mass"] = ("gyro_grade", g0, feas_now, acts, vb)
                k["gyro_grade"] = min(1.0, round(g0 * 3, 3))
                changes.append(f"mass ({f}): gyro noise x{g0:g} -> x{k['gyro_grade']:g} (a lighter gyro)")
            elif fine and k.get("st_heads", 2) == 2 and not history.get("_st2") and not closed("st_heads"):
                history["_last_mass"] = ("st_heads", 2, feas_now, acts, vb)
                k["st_heads"] = 1; changes.append(f"mass ({f}): one star-tracker head instead of two")
            elif fam_has_fmr and "power" not in g and k.get("fmr_lambda", 0.1) > LAMBDA_MIN and not history.get("_lam_up") and not closed("fmr_lambda"):
                lam = k.get("fmr_lambda", 0.1); history["_last_mass"] = ("fmr_lambda", lam, feas_now, acts, vb)
                k["fmr_lambda"] = max(LAMBDA_MIN, lam / 3); history["_lam_down"] = True
                changes.append(f"mass ({f}): lighter pump, less copper (lambda {lam:g} -> {k['fmr_lambda']:g} kg/W)")
            elif fam_has_fmr and not closed("scale.fmr") and k.get("scale", {}).get("fmr", 1.0) > SCALE_MIN:
                s0 = k.setdefault("scale", {}).get("fmr", 1.0); history["_last_mass"] = ("scale.fmr", s0, feas_now, acts, vb)
                k["scale"]["fmr"] = max(SCALE_MIN, s0 * DOWN)
                history.setdefault("fmr", []).append("down")
                changes.append(f"mass ({f}): fluid-loop momentum x{s0:g} -> x{k['scale']['fmr']:g}")
            else:
                blocked.append(f"mass ({f}): no lever left ({g})")
    last_up = {}
    for part, d in sorted(want.items()):
        s = k.setdefault("scale", {}).get(part, 1.0)
        if part in frozen:
            blocked.append(f"{part}: {frozen[part]}"); continue
        tried = history.setdefault(part, [])
        if d == "up":
            if s >= SCALE_MAX:
                blocked.append(f"{part}: authority at its bound (x{s:g})"); continue
            if "down" in tried:
                blocked.append(f"{part}: performance needs more authority, power needs less — conflict"); continue
            ns = min(SCALE_MAX, s * UP)
        else:
            if s <= SCALE_MIN:
                blocked.append(f"{part}: authority at its lower bound (x{s:g})"); continue
            if "up" in tried:
                blocked.append(f"{part}: power needs less authority, performance needs more — conflict"); continue
            ns = max(SCALE_MIN, s * DOWN)
        tried.append(d)
        if d == "up":
            last_up[part] = (s, res)
        k["scale"][part] = ns
        changes.append(f"{part}: authority x{s:g} -> x{ns:g} ({'performance' if d == 'up' else 'power'})")
    history["_last_up"] = last_up
    return k, v, changes, blocked


def rate_violation(res):
    return sum(min(v, 10.0) for r in res.values() for m, v in r.get("violation", {}).items() if m.startswith("rate_stability"))


def fam_violation(fam_opts, modes):
    """Sum over the modes of the best usable option's requirement violation (0 when a mode passes)."""
    tot = 0.0
    for M in modes:
        rs = [r for (m, _), r in fam_opts.items() if m == M["id"]]
        if rs:
            tot += min(sum(r.get("violation", {}).values()) if not r["feasible"] else 0.0 for r in rs)
    return tot


def usable(o, fam_acts):
    need = {o["actuator"]} | ({o["dump"]} if o.get("dump") else set())
    return need <= set(fam_acts)


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
        for key, name in (("mass", "mass_kg"), ("vol", "volume_L")):
            lim = req.get(key)
            if lim is not None and bud[name] > lim:
                gaps.append(f"budget: {name} {bud[name]:.3g} > {lim:g}")
        out[fa["id"]] = {"role": fa["role"], "simplicity": fa.get("simplicity", 9), "label": fa["label"], "feasible": not gaps, "gaps": gaps,
                         "modes": per_mode, "budget": {k: bud[k] for k in ("mass_kg", "power_W", "volume_L")}, "product": bud["product"]}
    sol = sorted([f for f in out if out[f]["role"] == "solution"], key=lambda f: out[f]["simplicity"])
    feas = [f for f in sol if out[f]["feasible"]]
    if feas:
        pick, status = feas[0], "feasible"
    else:
        pick = min(sol, key=lambda f: (len(out[f]["gaps"]), out[f]["simplicity"]))
        status = "closest (not feasible)"
    return {"case": case, "selected": pick, "status": status, "families": out}


def node_dispatch(case, sel, sized, modes, build):
    fam = sel["selected"]
    F = sel["families"][fam]
    meth = {m: F["modes"][m]["option"] for m in F["modes"] if F["modes"][m]}
    Mby = {M["id"]: M for M in modes}
    opt = lambda m: next(o for o in Mby[m]["options"] if o["id"] == meth[m])
    algs, dt = {}, 0.2
    tune = {}
    for m in meth:
        algs.update({k: v for k, v in (F["modes"][m].get("algorithms") or {}).items() if k in (SLOT.get((m, meth[m])),)})
        if m == "nadir_pointing":
            tune = split_alg(F["modes"][m].get("alg"))[1]
        dt = min(dt, opt(m)["dt_s"])
    det, acq, fine = opt("detumble"), opt("sun_acquisition"), opt("nadir_pointing")
    a_ = 6378137 + E.case_value(case, "orbit.alt") * 1e3
    T = 2 * math.pi * math.sqrt(a_ ** 3 / 3.986004418e14)
    scen = {"schema": "adcs-scenario/1", "id": f"dispatch_{case}_{fam}", "case": case, "product": f"SZ-{case}-{fam}",
            "label": f"{case} — dispatched {fam}: {det['fsw_mode']} -> {acq['fsw_mode']} (auto) -> {fine['fsw_mode']} (schedule)",
            "time": {"duration_s": round(3 * T), "dt_s": dt, "record_dt_s": 1.0},
            "initial": {"attitude": {"kind": "random"}, "rate": {"kind": "random_direction", "magnitude_deg_s": "case:mission.w0"}},
            "fsw": {"start_mode": det["fsw_mode"], "auto_next": acq["fsw_mode"], "guidance": {"kind": "nadir"}, "algorithms": algs,
                    "schedule": [{"t_s": round(2 * T), "mode": fine["fsw_mode"]}],
                    "rcs_dump": 1.0 if fine.get("dump") == "rcs" else 0.0, **({"dump_gain": 0.03} if fine["actuator"] == "fmr" else {}), **tune},
            "metrics": [{"id": "detumble_time", "kind": "time_to_rate", "rate_threshold_deg_s": 0.5, "hold_s": 600.0, "requirement": "req.detumble"},
                        {"id": "ape_los_p9973", "kind": "ape_los", "window": "last_half_orbit", "statistic": "p99.73", "requirement": "req.ape"},
                        {"id": "ake_los_p9973", "kind": "ake_los", "window": "last_half_orbit", "statistic": "p99.73", "requirement": "req.ake"},
                        {"id": "power_mean", "kind": "power_mean", "requirement": "req.pavg", "window": "all"},
                        {"id": "power_peak", "kind": "power_peak", "window": "all"},
                        {"id": "propellant", "kind": "propellant", "window": "all"}]}
    dd = ROOT / "dist" / "dispatch" / case / fam / "converged"      # beside the MATLAB solution's package, never over it
    if dd.exists():
        shutil.rmtree(dd)
    (dd / "fsw").mkdir(parents=True)
    sp = dd / "mission_scenario.json"
    sp.write_text(json.dumps(scen, indent=1))
    for f in ("products", "parts"):
        shutil.copytree(sized / f, dd / "sized" / f)
    shutil.copy(sized / "sizing.json", dd / "sized" / "sizing.json")
    env = dict(os.environ, ADCS_SIZED_DIR=str(sized))
    case_csv = str(MS / "cases" / f"{case}.csv")
    blob = dd / "fsw" / "adcs_fswcfg.bin"
    subprocess.run([str(BIN), "params", str(sp), "--case", case_csv, "--out", str(blob)], cwd=MS, env=env, check=True, capture_output=True)
    import fswcfg
    (dd / "fsw" / "adcs_fswcfg.json").write_text(json.dumps(fswcfg.decode(blob.read_bytes()), indent=1))
    res = {}
    for impl in ("c", "rust"):
        out = PIPE / case / "dispatch" / impl
        p = subprocess.run([str(BIN), "run", str(sp), "--case", case_csv, "--fsw", impl, "--out", str(out), "--quiet"], cwd=MS, env=env, capture_output=True, text=True)
        man = json.loads((out / "manifest.json").read_text()) if p.returncode == 0 else {}
        res[impl] = {"rc": p.returncode, "build_id": man.get("fsw", {}).get("build_id"), "metrics": man.get("metrics"), "mode_log": man.get("mode_log")}
    same = False
    a, b = PIPE / case / "dispatch" / "c" / "channels.csv", PIPE / case / "dispatch" / "rust" / "channels.csv"
    if a.exists() and b.exists():
        same = a.read_bytes() == b.read_bytes()
    res["c_equals_rust_bitwise"] = same
    (dd / "engine_check.json").write_text(json.dumps(res, indent=1))
    ms = ", ".join(f"{m} = {meth[m]}" for m in meth)
    sens = [f["slot"] for f in json.loads((sized / "products" / f"SZ-{case}-{fam}.json").read_text())["fill"]]
    (dd / "BUILD.md").write_text(f"""# Flight configuration: {case} / {fam}

Owner: Agastya. Generated by `tools/pipeline.py` (node `dispatch`) from the converged design loop
(`matlab_sils/store/pipeline/{case}/selection.json`, status: **{sel['status']}**).
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
    return {"family": fam, "dir": str(dd.relative_to(ROOT)), "scenario": str(sp), "check": res, "methods": meth, "algorithms": algs}


def node_mc(case, disp, sized, runs, jobs):
    """Monte Carlo of the dispatched mission: the case's Monte Carlo dispersions (the MC campaign
    of the case), per-run seeds, on the converged product."""
    camp = {"ais_3u": "mc_nadir_ais", "ais_img_3u": "mc_fine_img"}.get(case)
    C = json.loads((E.CAMP / f"{camp}.json").read_text()) if camp else {"dispersions": []}
    C = {"id": f"mc_dispatch_{case}", "case": case, "seed": C.get("seed", 1), "runs": runs, "type": "montecarlo",
         "dispersions": [d for d in (C["dispersions"] if isinstance(C["dispersions"], list) else [C["dispersions"]]) if d["kind"] != "initial_error_deg"]}
    base = PIPE / case / "mc"
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
    with tf.ThreadPoolExecutor(jobs) as ex:
        for k, rc in ex.map(one, jobs_):
            if rc:
                print(f"  [FAIL] mc run {k}")
    rr = []
    for k in range(1, runs + 1):
        f = base / f"run_{k:04d}" / "manifest.json"
        if f.exists():
            rr.append({"k": k, "metrics": json.loads(f.read_text())["metrics"], "draws": draws[k]})
    res = {"schema": "adcs-campaign-result/1", "id": C["id"], "case": case, "runs": len(rr), "dispersions": [d["kind"] for d in C["dispersions"]],
           "stats": E.summarise(rr), "per_run": rr}
    write(base / "summary.json", res)
    print(f"  mc: {len(rr)}/{runs} runs of the dispatched mission")
    return res


def node_key(disp, sized, build, extra=""):
    pid = json.loads(pathlib.Path(disp["scenario"]).read_text())["product"]
    prod, parts = product_blob(sized, pid)
    return sha(json.loads(pathlib.Path(disp["scenario"]).read_text()), prod, parts, build, extra)


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


def jl_(p):
    return json.loads(p.read_text()) if p.exists() else None


# ---------------------------------------------------------------- the loop
def run_case(case, a, modes, families, build):
    print(f"== {case}")
    state = PIPE / case
    state.mkdir(parents=True, exist_ok=True)
    for old in state.glob("iter_*"):                    # a new loop starts from the laws as written
        shutil.rmtree(old)
    knobs, variants_on, history, log = {"scale": {}}, set(), {}, []
    it = 0
    for it in range(1, a.max_iter + 1):
        print(f" iteration {it}: knobs {json.dumps(knobs)}")
        sized, sizing = node_size(case, it, knobs)
        tests = node_matrix(case, it, sized, modes, variants_on, [int(s) for s in a.seeds.split(",")], a.jobs, build)
        res = node_assess(tests, modes)
        sel = node_select(case, res, sizing, modes, families)
        knobs2, variants2, changes, blocked = node_converge(case, res, knobs, variants_on, history, sizing["class"] == "fine", modes, sel)
        entry = {"iteration": it, "knobs": knobs, "class": sizing["class"], "selected": sel["selected"], "status": sel["status"],
                 "feasible_options": sum(r["feasible"] for r in res.values()), "options": len(res), "changes": changes, "blocked": blocked,
                 "families": {f: {"feasible": v["feasible"], "gaps": v["gaps"], "budget": v["budget"]} for f, v in sel["families"].items()},
                 "matrix": [{k: r[k] for k in ("mode", "option", "alg", "feasible", "failing", "objective", "objective_id")} for r in res.values()]}
        log.append(entry)
        write(state / f"iter_{it}" / "assess.json", entry)
        print(f"  -> {sel['selected']} ({sel['status']}), {entry['feasible_options']}/{entry['options']} options feasible; "
              f"{len(changes)} change(s), {len(blocked)} blocked")
        for c in changes:
            print(f"     change: {c}")
        if not changes:
            break
        knobs, variants_on = knobs2, variants2
    converged = not log[-1]["changes"]
    sel["converged"], sel["iterations"], sel["knobs"] = converged, it, knobs
    sel["class"] = sizing["class"]
    sel["sensors"] = [f["slot"] for f in json.loads((sized / "products" / f"SZ-{case}-{sel['selected']}.json").read_text())["fill"]
                      if f["slot"] not in ("coils", "wheels", "rings", "cmg", "vscmg", "rcs")]
    sel["demand"] = sizing["demand"]
    write(state / "selection.json", sel)
    write(state / "loop.json", log)
    disp = node_dispatch(case, sel, sized, modes, build)
    write(state / "dispatch.json", disp)
    mc = node_mc(case, disp, sized, a.mc_runs, a.jobs) if a.mc_runs else None
    so = node_soft_oils(case, disp, sized, build) if not a.no_oils else None
    ledger(case, sel, log, disp, mc, so, sizing)


def ledger(case, sel, log, disp, mc, so, sizing):
    fmt = lambda x: "—" if x is None else (f"{x:.4g}" if isinstance(x, (int, float)) else str(x))
    F = sel["families"][sel["selected"]]
    L = [f"# Design loop: {case}", "",
         f"Owner: Agastya. `tools/pipeline.py {case}` — sizing (Rust, adcs-design) -> SILS mode matrix (Rust engine, POP in the loop,",
         "C flight software) -> assess -> converge, repeated until nothing is left to change; then select, dispatch, Monte Carlo and",
         "soft OILS of the selected configuration. Flow: `docs/figures/flow_design_to_hils.svg`, method: `docs/DESIGN_LOOP.md`.", "",
         f"**Selected: `{sel['selected']}` — {F['label']} — {sel['status']}**, {'converged' if sel['converged'] else 'NOT converged'} after "
         f"{sel['iterations']} iteration(s). Knowledge class: {sel['class']}. Sensors: {', '.join(sel['sensors'])}.", ""]
    if F["gaps"]:
        L += ["Open requirement gaps of the selected family (what the case must relax, or the next design lever):", ""] + [f"- {g}" for g in F["gaps"]] + [""]
    L += ["## Iterations", "", "| iteration | knobs | options feasible | selected | changes | blocked |", "|---:|---|---:|---|---|---|"]
    for e in log:
        kn = ", ".join([f"{k} x{v:.3g}" for k, v in e["knobs"].get("scale", {}).items()] + [k for k in ("star_tracker",) if e["knobs"].get(k)] +
                      [f"pump lambda {e['knobs']['fmr_lambda']:g} kg/W" for _ in [0] if "fmr_lambda" in e["knobs"]] +
                      [f"{e['knobs']['st_heads']} ST head" for _ in [0] if e["knobs"].get("st_heads") == 1] +
                      [f"flow sensor {e['knobs']['fmr_flow_sigma'] * 1e3:g} mm/s" for _ in [0] if e["knobs"].get("fmr_flow_sigma")] +
                      [f"gyro noise x{e['knobs']['gyro_grade']:g}" for _ in [0] if e["knobs"].get("gyro_grade", 1) < 1]) or "laws as written"
        L.append(f"| {e['iteration']} | {kn} | {e['feasible_options']}/{e['options']} | {e['selected']} ({e['status']}) | "
                 f"{'<br>'.join(e['changes']) or '—'} | {len(e['blocked'])} |")
    if log[-1]["blocked"]:
        L += ["", "Why the loop stopped (nothing left that a knob can change):", ""] + [f"- {b}" for b in sorted(set(log[-1]["blocked"]))]
    L += ["", "## Families (last iteration)", "", "| family | role | feasible | mass [kg] | power [W] | volume [L] | gaps |", "|---|---|---|---:|---:|---:|---|"]
    for f, v in sel["families"].items():
        b = v["budget"]
        L.append(f"| {f} | {v['role']} | {'yes' if v['feasible'] else 'no'} | {b['mass_kg']:.3f} | {b['power_W']:.2f} | {b['volume_L']:.3f} | {'; '.join(v['gaps']) or '—'} |")
    L += ["", f"## Selected methods ({sel['selected']})", "", "| mode | option | feasible | objective | algorithms | failing |", "|---|---|---|---:|---|---|"]
    for m, r in F["modes"].items():
        if r:
            L.append(f"| {m} | {r['option']} | {'yes' if r['feasible'] else 'no'} | {fmt(r['objective'])} {r['objective_id']} | "
                     f"{', '.join(f'{k}={v}' for k, v in (r['algorithms'] or {}).items() if v)} | {', '.join(r['failing']) or '—'} |")
    last = log[-1]
    L += ["", "## Mode matrix (last iteration, best algorithm per option)", "", "| mode | option | algorithm | feasible | objective | failing (cause) |", "|---|---|---|---|---:|---|"]
    for r in sorted(last["matrix"], key=lambda z: (z["mode"], z["option"])):
        L.append(f"| {r['mode']} | {r['option']} | {r['alg'] or 'default'} | {'yes' if r['feasible'] else 'no'} | {fmt(r['objective'])} | "
                 f"{', '.join(f'{k} ({v})' for k, v in r['failing'].items()) or '—'} |")
    if mc:
        L += ["", f"## Monte Carlo of the dispatched mission ({mc['runs']} runs; dispersions: {', '.join(mc['dispersions'])})", "",
              "| metric | req | mean ± std | [min, max] | pass rate |", "|---|---:|---|---|---:|"]
        for s in mc["stats"]:
            if s["mean"] is None:
                continue
            rate = "—" if s["pass_rate"] is None else f"{100 * s['pass_rate']:.0f} %"
            L.append(f"| {s['id']} ({s['unit']}) | {fmt(s['req'])} | {s['mean']:.4g} ± {s['std']:.3g} | [{s['min']:.4g}, {s['max']:.4g}] | {rate} |")
    if so:
        L += ["", "## SILS and soft OILS of the dispatched mission", "",
              "The same scenario, product and blob: in-process C (SILS) and the flight software as Cortex-M4F firmware in QEMU,",
              "C and Rust builds, with exact per-step instruction counts and the command latency applied (docs/SOFT_OILS.md).", "",
              "| metric | req | SILS | soft OILS (C on M4F) | soft OILS (Rust on M4F) |", "|---|---:|---:|---:|---:|"]
        get = lambda k, i: next((x for x in (so[k].get("metrics") or []) if x["id"] == i), {})
        for m in so["sils"].get("metrics") or []:
            vv = lambda k: (lambda x: f"{fmt(x.get('value'))} {'✓' if x.get('pass') == 1 else ('✗' if x.get('pass') == 0 else '')}")(get(k, m["id"]))
            L.append(f"| {m['id']} | {fmt(m.get('req'))} | {vv('sils')} | {vv('oils')} | {vv('oils_rs')} |")
        L += ["", "| OBC build | instructions/step mean / max | exec max [ms] | latency mean / max [ms] | CPU load max | overruns |", "|---|---:|---:|---:|---:|---:|"]
        for k, lab in (("oils", "C (arm-none-eabi-gcc -O2)"), ("oils_rs", "Rust (thumbv7em-none-eabihf)")):
            o = so[k].get("oils") or {}
            if o:
                L.append(f"| {lab} | {fmt(o['instructions']['mean'])} / {fmt(o['instructions']['max'])} | {1e3 * o['exec_s']['max']:.3f} | "
                         f"{1e3 * o['latency_s']['mean']:.3f} / {1e3 * o['latency_s']['max']:.3f} | {100 * o['cpu_load_max']:.1f} % | {o['overruns']} |")
    L += ["", f"Dispatch: `{disp['dir']}` (blob, sized products, BUILD.md). C = Rust flight software bitwise on the engine: "
          f"{disp['check'].get('c_equals_rust_bitwise')}.", ""]
    OUT.mkdir(exist_ok=True)
    (OUT / f"DESIGN_{case}.md").write_text("\n".join(L) + "\n")
    print(f"  wrote results/DESIGN_{case}.md")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cases", nargs="*")
    ap.add_argument("--seeds", default="1,2")
    ap.add_argument("--max-iter", type=int, default=16)
    ap.add_argument("--jobs", type=int, default=os.cpu_count())
    ap.add_argument("--mc-runs", type=int, default=12)
    ap.add_argument("--no-oils", action="store_true")
    a = ap.parse_args()
    if not BIN.exists():
        sys.exit("engine not built: python3 tools/engine.py build")
    modes = sorted((json.loads(f.read_text()) for f in (MS / "data" / "modes").glob("*.json")), key=lambda M: M["order"])
    fams = json.loads((MS / "data" / "families.json").read_text())["family"]
    FAMILIES[:] = fams
    build = sha(BIN.read_bytes())
    for c in a.cases or ["ais_3u", "ais_img_3u"]:
        run_case(c, a, modes, fams, build)


if __name__ == "__main__":
    main()
