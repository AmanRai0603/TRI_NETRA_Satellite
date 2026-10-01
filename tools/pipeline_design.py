"""The loop's design nodes: size, the mode matrix (every mode x option x seed flown), assess, converge.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "tools"))
import concurrent.futures as cf, json, math, os, pathlib, shutil, subprocess, time
import engine as E
from common import write_text
from pipeline_base import BIN, CACHE, CANDIDATES, DOWN, FAMILIES, FLOW_SIGMA_MIN, GYRO_MIN, IMPROVE, LAMBDA_MAX, LAMBDA_MIN, MS, PIPE, SCALE_MAX, SCALE_MIN, SLOT, TUNE, UP, auth_part, case_bytes, cls, fam_violation, rate_violation, sha, split_alg, tune_grid, usable, write


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


def node_matrix(case, it, sized, modes, variants_on, seeds, jobs, build, tuned=()):
    tests, runs = [], []
    seeds0 = seeds
    for M in modes:
        for o in M["options"]:
            base = E.mode_scenario(case, M, o)
            slot = SLOT.get((M["id"], o["id"]))
            algs, seeds = [None], seeds0
            if slot and (M["id"], o["id"]) in variants_on:
                algs = CANDIDATES[slot]
            if slot and [M["id"], o["id"]] in [list(x) for x in tuned]:
                # min over the gains of the worst case over the seeds: extra seeds widen the worst case
                algs = CANDIDATES[slot] + tune_grid(slot)
                seeds = list(seeds0) + [x for x in TUNE["extra_seeds"] if x not in seeds0]
            for alg in algs:
                s = json.loads(json.dumps(base))
                if alg:
                    law, tune = split_alg(alg)
                    s["fsw"].setdefault("algorithms", {})[slot] = law
                    s["fsw"].update(tune)
                    s["id"] = f"{s['id']}__{alg.replace('@', '_').replace('=', '').replace(',', '_')}"
                prod, parts = product_blob(sized, s["product"])
                creq = case_bytes(case)
                d = PIPE / case / f"iter_{it}" / "scenarios"
                d.mkdir(parents=True, exist_ok=True)
                sp = d / f"{s['id']}.json"
                write_text(sp, json.dumps(s, indent=1))
                keys = []
                for sd in seeds:
                    k = sha(s, prod, parts, sd, "c", build, creq)
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
    res, allv = {}, {}
    for t in tests:
        M = Mby[t["mode"]]
        mans = [json.loads((CACHE / k / "manifest.json").read_text()) for k in t["keys"] if (CACHE / k / "manifest.json").exists()]
        fails, obj, metrics, viol, oreq = {}, None, {}, {}, None
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
                    oreq = x.get("req")
        feas = not fails and len(mans) == len(t["keys"])
        worst = {k: (max((v for v in vs if v is not None), default=None)) for k, vs in metrics.items()}
        r = {**t, "feasible": feas, "failing": fails, "violation": viol, "objective": obj, "objective_id": M["objective"], "objective_req": oreq, "metrics": worst,
             "algorithms": (mans[0].get("algorithms") if mans else {})}
        key = (t["mode"], t["option"])
        cur = res.get(key)
        rank = lambda z: (not z["feasible"], len(z["failing"]), z["objective"] if z["objective"] is not None else math.inf)
        allv.setdefault(key, []).append({"alg": t["alg"], "feasible": feas, "failing": fails, "objective": obj, "violation": sum(viol.values())})
        if cur is None or rank(r) < rank(cur):
            res[key] = r
    # every variant's worst case, kept for the options that flew more than one (the literature table)
    for key, vs in allv.items():
        if len(vs) > 1 and key in res:
            res[key]["variants"] = vs
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
        if vn > IMPROVE * vb:
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
        if before > 0 and now > IMPROVE * before:
            k.setdefault("scale", {})[part] = prev_scale
            frozen[part] = f"more authority did not reduce the performance violation ({before:.3g} -> {now:.3g}); kept at x{prev_scale:g}"
            changes.append(f"{part}: authority back to x{prev_scale:g} (no improvement)")
    want = {}                                  # part -> "up" | "down"
    for (mode, oid), r in sorted(res.items()):
        o = next(x for x in Mby[mode]["options"] if x["id"] == oid)
        if r["feasible"]:
            # a coils-only option that passes with less than the tune margin left (worst seed above margin x req) is
            # tuned too: it flies from two seeds here, and the Monte Carlo would find the thin margin
            q = r.get("objective_req")
            if r["slot"] in TUNE["grids"] and o["actuator"] in TUNE["actuators"] and isinstance(q, (int, float)) and q > 0 \
                    and r["objective"] is not None and r["objective"] > TUNE["margin"] * q:
                if (mode, oid) not in v:
                    v.add((mode, oid)); changes.append(f"{mode}/{oid}: thin margin ({r['objective']:.3g} of {q:g}): fly every {r['slot']} algorithm")
                elif [mode, oid] not in history.setdefault("_tuned", []):
                    history["_tuned"].append([mode, oid])
                    changes.append(f"{mode}/{oid}: thin margin ({r['objective']:.3g} of {q:g}): tune every {r['slot']} law's gains (Bruni & Celani 2017)")
            continue
        kinds = set(r["failing"].values())
        part = auth_part(mode, o)
        if "performance" in kinds:
            if r["slot"] and (mode, oid) not in v:
                v.add((mode, oid)); changes.append(f"{mode}/{oid}: fly every {r['slot']} algorithm")
            elif r["slot"] in TUNE["grids"] and o["actuator"] in TUNE["actuators"] and [mode, oid] not in history.setdefault("_tuned", []):
                history["_tuned"].append([mode, oid])
                changes.append(f"{mode}/{oid}: tune every {r['slot']} law's gains, min over the gains of the worst seed (Bruni & Celani 2017)")
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
        if v_now > (2.0 - IMPROVE) * v_before + 1e-9:
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


def node_key(disp, sized, build, extra=""):
    scen = json.loads(pathlib.Path(disp["scenario"]).read_text())
    prod, parts = product_blob(sized, scen["product"])
    return sha(scen, prod, parts, build, extra, case_bytes(scen["case"]))


def node_redundancy(sel, knobs):
    """The fault campaign's lever (node faults, select fault_policy gap): when the selected or closest
    family carries fluid rings and loses fine pointing to a single ring failure, fit the spare ring
    (adcs-design knob fmr_spare: a fourth, skewed ring that stands in for any one). Once only."""
    fam = sel["families"].get(sel["selected"], {})
    ring_gap = [g for g in fam.get("fault_gaps", []) if g.startswith("fault: rotor_fail")]
    if "fmr" not in sel["selected"].split("_") or not ring_gap or knobs.get("fmr_spare"):
        return knobs, []
    k = json.loads(json.dumps(knobs))
    k["fmr_spare"] = True
    return k, [f"redundancy ({sel['selected']}): {ring_gap[0]} -> a spare fluid ring, skewed, that stands in for any one ring"]
