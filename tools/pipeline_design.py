"""The loop's design nodes: size, the mode matrix (every mode x option x seed flown), assess, converge.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "tools"))
import concurrent.futures as cf, json, math, os, pathlib, shutil, subprocess, time
import engine as E
from common import write_text
from design_call import Buf, call, take
from pipeline_base import BIN, CACHE, CANDIDATES, DOWN, FAMILIES, FLOW_SIGMA_MIN, GYRO_MIN, IMPROVE, LAMBDA_MAX, LAMBDA_MIN, MS, PIPE, SCALE_MAX, SCALE_MIN, SLOT, TUNE, UP, case_bytes, cls, sha, split_alg, tune_grid, usable, write


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


# the design loop's rules are design_loop_converge's, design_loop_robustness's and design_loop_redundancy's methods
# (design/looprules.pc), which the engine carries; these nodes hand them the iteration and say their decisions in words
PARTS = ["cmg", "fmr", "mtq", "mtqp", "rcs", "rw", "vscmg"]                 # looprules' Part, in its order
ACTUATORS = ["mtq", "rw", "cmg", "vscmg", "fmr", "rcs"]                     # looprules' Actuator
LEVERS = ["gyro_grade", "st_heads", "fmr_lambda", "scale.fmr"]             # looprules' MassLever
EVENTS = ["gyro_back", "authority_back", "thin_fly", "thin_tune", "fly_every", "tune", "perf_and_power", "gyro_down", "flow_sensor",
          "star_tracker", "knowledge_blocked", "rcs_power", "lambda_up", "lambda_blocked", "power_floor", "mass_undo", "mass_gyro",
          "mass_heads", "mass_lambda", "mass_fmr", "mass_none", "frozen", "bound_up", "conflict_up", "bound_down", "conflict_down",
          "authority", "robust_lambda", "robust_power", "robust_authority", "robust_performance", "robust_star_tracker",
          "robust_knowledge"]                                                # looprules' LoopEvent
EVENT_BUFS = 7                                                               # kind, option, part, three numbers, a flag


def num(x):
    """A number the loop's method takes: a value that is not one (None) as nan, with whether there is one."""
    return (True, float(x)) if isinstance(x, (int, float)) else (False, float("nan"))


def events(flat, n_events):
    """The method's events, each (kind, option, part, a, b, c, flag), from its flattened buffers."""
    cols = flat[:EVENT_BUFS]
    return [(EVENTS[int(cols[0][i])], int(cols[1][i]), int(cols[2][i]), cols[3][i], cols[4][i], cols[5][i], int(cols[6][i])) for i in range(n_events)]


def node_converge(case, res, knobs, variants_on, history, fine, modes, sel=None):
    """The knob changes the failures call for (design_loop_converge, design/looprules.pc, asked through the engine).
    Returns (new knobs, new variants, changes, blocked)."""
    k = json.loads(json.dumps(knobs))
    v = set(variants_on)
    changes, blocked = [], []
    Mby = {M["id"]: M for M in modes}
    opt = lambda key: next(x for x in Mby[key[0]]["options"] if x["id"] == key[1])
    keys = list(res)                                   # the options in the order they were flown
    ix = {key: i for i, key in enumerate(keys)}
    mix = {M["id"]: i for i, M in enumerate(modes)}
    tuned_h = history.get("_tuned", [])
    lg = history.pop("_last_gyro", None)
    frozen = history.setdefault("_frozen", {})
    last_up = history.pop("_last_up", {})
    lm = history.pop("_last_mass", None)
    # the options: what each is, how it did, its violations in order
    O = {c: [] for c in ("mode", "act", "dump", "feas", "slot", "tun", "hobj", "obj", "hreq", "req", "perf", "know", "power", "prop", "rate", "inv", "tuned")}
    vstart, vval, vperf, vrate = [0], [], [], []
    for key in keys:
        r, o = res[key], opt(key)
        kinds = set(r["failing"].values())
        (ho, ob), (hq, q) = (r["objective"] is not None, float(r["objective"]) if r["objective"] is not None else float("nan")), num(r.get("objective_req"))
        for c, x in (("mode", mix.get(key[0], -1)), ("act", ACTUATORS.index(o["actuator"])), ("dump", o.get("dump") == "rcs"), ("feas", bool(r["feasible"])),
                     ("slot", bool(r["slot"])), ("tun", r["slot"] in TUNE["grids"] and o["actuator"] in TUNE["actuators"]), ("hobj", ho), ("obj", ob),
                     ("hreq", hq), ("req", q), ("perf", "performance" in kinds), ("know", "knowledge" in kinds), ("power", "power" in kinds),
                     ("prop", "propellant" in kinds), ("rate", any(m.startswith("rate_stability") for m in r["failing"])), ("inv", key in v),
                     ("tuned", [key[0], key[1]] in tuned_h)):
            O[c].append(x)
        for m, val in r.get("violation", {}).items():
            vval.append(val); vperf.append(cls(m) == "performance"); vrate.append(m.startswith("rate_stability"))
        vstart.append(len(vval))
    order = sorted(range(len(keys)), key=lambda i: keys[i])
    # the iteration the authority moves were made in (every part moved up in it holds its results), and those moves
    prev = next(iter(last_up.values()))[1] if last_up else {}
    P = {c: [] for c in ("act", "feas", "cur")}
    pstart, pval, pperf = [0], [], []
    for key, r0 in prev.items():
        P["act"].append(ACTUATORS.index(opt(key)["actuator"])); P["feas"].append(bool(r0["feasible"])); P["cur"].append(ix.get(key, -1))
        for m, val in r0.get("violation", {}).items():
            pval.append(val); pperf.append(cls(m) == "performance")
        pstart.append(len(pval))
    bits = lambda parts: sum(1 << PARTS.index(p) for p in parts if p in PARTS)
    lm_key, lm_old, lm_before, lm_acts, lm_vb = lm if lm else ("gyro_grade", 0.0, [], [], 0.0)
    has_scale = [p in k.get("scale", {}) for p in PARTS]
    sol = [f for f, x in (sel or {}).get("families", {}).items() if x["role"] == "solution"]
    acts_of = lambda f: next((fa["actuators"] for fa in FAMILIES if fa["id"] == f), None)
    F = (sel or {}).get("families", {})
    gyro, sigma, lam = num(k.get("gyro_grade")), num(k.get("fmr_flow_sigma")), num(k.get("fmr_lambda"))
    k0g, k0s, k0l = num(knobs.get("gyro_grade")), num(knobs.get("fmr_flow_sigma")), num(knobs.get("fmr_lambda"))
    heads = num(k.get("st_heads"))
    ne = 4*len(keys) + 4*len(PARTS) + 16
    no = len(keys)
    args = (
        Buf(O["mode"] + [0]), Buf(O["act"] + [0]), Buf(O["dump"] + [0]), Buf(O["feas"] + [0]), Buf(O["slot"] + [0]), Buf(O["tun"] + [0]),
        Buf(O["hobj"] + [0]), Buf(O["obj"] + [0.0]), Buf(O["hreq"] + [0]), Buf(O["req"] + [0.0]), Buf(O["perf"] + [0]), Buf(O["know"] + [0]),
        Buf(O["power"] + [0]), Buf(O["prop"] + [0]), Buf(O["rate"] + [0]), Buf(O["inv"] + [0]), Buf(O["tuned"] + [0]), Buf(vstart),
        Buf(vval + [0.0]), Buf(vperf + [0]), Buf(vrate + [0]), Buf(order + [0]), no, len(modes),
        Buf(P["act"] + [0]), Buf(P["feas"] + [0]), Buf(pstart), Buf(pval + [0.0]), Buf(pperf + [0]), Buf(P["cur"] + [0]), len(prev),
        Buf([PARTS.index(p) for p in last_up]), Buf([float(s0) for s0, _r in last_up.values()]), len(last_up),
        bool(lg), float(lg[0]) if lg else 0.0, float(lg[1]) if lg else 0.0, bool(history.get("_closed_gyro_grade")),
        bool(history.get("_closed_st_heads")), bool(history.get("_closed_fmr_lambda")), bool(history.get("_closed_scale.fmr")),
        bool(history.get("_lam_up")), bool(history.get("_lam_down")), bool(history.get("_st2")),
        bits(frozen), bits(p for p in PARTS if "up" in history.get(p, [])), bits(p for p in PARTS if "down" in history.get(p, [])),
        bool(lm), LEVERS.index(lm_key), float(lm_old), float(lm_vb), Buf([key in lm_before for key in keys] + [0]),
        Buf([usable(opt(key), lm_acts) for key in keys] + [0]),
        gyro[0], gyro[1], sigma[0], sigma[1], lam[0], lam[1], bool(k.get("star_tracker")), heads[0], heads[1],
        Buf(has_scale), Buf([float(k["scale"][p]) if has else 1.0 for p, has in zip(PARTS, has_scale)]),
        k0g[0], k0g[1], k0s[0], k0s[1], k0l[0], k0l[1], bool(fine), bool(sel), len(sol),
        Buf([len(F[f]["gaps"]) for f in sol]), Buf([float(F[f]["simplicity"]) for f in sol]),
        Buf(["mass_kg" in " ".join(F[f]["gaps"]) for f in sol]), Buf(["power" in " ".join(F[f]["gaps"]) for f in sol]),
        Buf(["fmr" in f for f in sol]),
        Buf([sum(1 << j for j, f in enumerate(sol) if acts_of(f) is not None and usable(opt(key), acts_of(f))) for key in keys] + [0]),
        1.0 - IMPROVE, SCALE_MIN, SCALE_MAX, UP, DOWN, LAMBDA_MIN, LAMBDA_MAX, FLOW_SIGMA_MIN, GYRO_MIN, TUNE["margin"],
        *[Buf([0] * ne) for _ in range(EVENT_BUFS)], Buf([0] * (no + 1)), Buf([0] * (no + 1)))
    # the method's outputs: the events' count, then every buffer it was handed, as it left them
    out = take(call("looprules::loop_converge", *args), 1, *["buf"] * sum(isinstance(a, Buf) for a in args))
    n_events, bufs = int(out[0]), out[1:]
    if n_events >= ne:
        raise SystemExit(f"converge: {case}: the loop's rules made {n_events} decisions, the most the lists hold")
    o_lost, o_feas = bufs[-2], bufs[-1]
    new_last_up = {}
    for kind, i, p, a, b, c, f in events(bufs[-EVENT_BUFS - 2:-2], n_events):
        key = keys[i] if i >= 0 else None
        name = f"{key[0]}/{key[1]}" if key else ""
        r = res[key] if key else None
        part = PARTS[p] if p >= 0 else None
        fam = sol[p] if kind.startswith("mass_") and kind != "mass_undo" else None
        if kind == "gyro_back":
            k["gyro_grade"] = lg[0]; history["_closed_gyro_grade"] = True
            changes.append(f"gyro grade back to x{lg[0]:g}: rate-stability violation {lg[1]:.3g} -> {c:.3g}")
        elif kind == "authority_back":
            k.setdefault("scale", {})[part] = last_up[part][0]
            frozen[part] = f"more authority did not reduce the performance violation ({b:.3g} -> {c:.3g}); kept at x{last_up[part][0]:g}"
            changes.append(f"{part}: authority back to x{last_up[part][0]:g} (no improvement)")
        elif kind in ("thin_fly", "thin_tune"):
            what = f"fly every {r['slot']} algorithm" if kind == "thin_fly" else f"tune every {r['slot']} law's gains (Bruni & Celani 2017)"
            if kind == "thin_fly":
                v.add(key)
            else:
                history.setdefault("_tuned", []).append([key[0], key[1]])
            changes.append(f"{name}: thin margin ({r['objective']:.3g} of {r.get('objective_req'):g}): {what}")
        elif kind == "fly_every":
            v.add(key); changes.append(f"{name}: fly every {r['slot']} algorithm")
        elif kind == "tune":
            history.setdefault("_tuned", []).append([key[0], key[1]])
            changes.append(f"{name}: tune every {r['slot']} law's gains, min over the gains of the worst seed (Bruni & Celani 2017)")
        elif kind == "perf_and_power":
            blocked.append(f"{name}: performance and power both fail — no authority change helps")
        elif kind == "gyro_down":
            g0 = k.get("gyro_grade", 1.0); k["gyro_grade"] = b
            history["_last_gyro"] = (g0, c)
            changes.append(f"rate stability: gyro noise x{g0:g} -> x{k['gyro_grade']:g} (fibre-optic class)")
        elif kind == "flow_sensor":
            sg = k.get("fmr_flow_sigma", 0.002); k["fmr_flow_sigma"] = b
            changes.append(f"{name}: fluid-loop flow sensor {sg * 1e3:g} -> {k['fmr_flow_sigma'] * 1e3:g} mm/s (1 sigma)")
        elif kind == "star_tracker":
            k["star_tracker"] = True; changes.append(f"{name}: knowledge -> fit the star tracker")
        elif kind == "knowledge_blocked":
            blocked.append(f"{name}: knowledge fails with the star tracker fitted")
        elif kind == "rcs_power":
            blocked.append(f"{name}: power is the thrusters' valve power (RCS dumping), not the pump")
        elif kind == "lambda_up":
            lam0 = k.get("fmr_lambda", 0.1); k["fmr_lambda"] = b; history["_lam_up"] = True
            changes.append(f"{name}: power -> electromagnetic pump with more copper (lambda {lam0:g} -> {k['fmr_lambda']:g} kg/W)")
        elif kind == "lambda_blocked":
            blocked.append(f"{name}: power fails with the pump at lambda {k.get('fmr_lambda', 0.1):g} kg/W" + (" (mass needs it lower: conflict)" if f else " (bound)"))
        elif kind == "power_floor":
            blocked.append(f"{name}: power fails at the sized authority ({part}); the part's standby power is the floor")
        elif kind == "mass_undo":
            lost = sorted(f"{keys[j][0]}/{keys[j][1]}" for j in range(no) if o_lost[j])
            if f:
                lost.append(f"requirement violation {lm_vb:.3g} -> {b:.3g}")
            if lm_key == "scale.fmr":
                k.setdefault("scale", {})["fmr"] = lm_old
            else:
                k[lm_key] = lm_old
            history[f"_closed_{lm_key}"] = True
            changes.append(f"mass lever {lm_key} undone: it broke {', '.join(lost)}")
        elif kind in ("mass_gyro", "mass_heads", "mass_lambda", "mass_fmr"):
            feas_now = [keys[j] for j in range(no) if o_feas[j]]
            acts = acts_of(fam)
            if kind == "mass_gyro":
                g0 = k["gyro_grade"]; history["_last_mass"] = ("gyro_grade", g0, feas_now, acts, c)
                k["gyro_grade"] = b
                changes.append(f"mass ({fam}): gyro noise x{g0:g} -> x{k['gyro_grade']:g} (a lighter gyro)")
            elif kind == "mass_heads":
                history["_last_mass"] = ("st_heads", 2, feas_now, acts, c)
                k["st_heads"] = 1; changes.append(f"mass ({fam}): one star-tracker head instead of two")
            elif kind == "mass_lambda":
                lam0 = k.get("fmr_lambda", 0.1); history["_last_mass"] = ("fmr_lambda", lam0, feas_now, acts, c)
                k["fmr_lambda"] = b; history["_lam_down"] = True
                changes.append(f"mass ({fam}): lighter pump, less copper (lambda {lam0:g} -> {k['fmr_lambda']:g} kg/W)")
            else:
                s0 = k.setdefault("scale", {}).get("fmr", 1.0); history["_last_mass"] = ("scale.fmr", s0, feas_now, acts, c)
                k["scale"]["fmr"] = b
                history.setdefault("fmr", []).append("down")
                changes.append(f"mass ({fam}): fluid-loop momentum x{s0:g} -> x{k['scale']['fmr']:g}")
        elif kind == "mass_none":
            blocked.append(f"mass ({fam}): no lever left ({' '.join(F[fam]['gaps'])})")
        else:                                          # the authority asked of a part
            s0 = k.setdefault("scale", {}).get(part, 1.0)
            if kind == "frozen":
                blocked.append(f"{part}: {frozen[part]}"); continue
            tried = history.setdefault(part, [])
            if kind == "bound_up":
                blocked.append(f"{part}: authority at its bound (x{s0:g})")
            elif kind == "conflict_up":
                blocked.append(f"{part}: performance needs more authority, power needs less — conflict")
            elif kind == "bound_down":
                blocked.append(f"{part}: authority at its lower bound (x{s0:g})")
            elif kind == "conflict_down":
                blocked.append(f"{part}: power needs less authority, performance needs more — conflict")
            else:
                tried.append("up" if f else "down")
                if f:
                    new_last_up[part] = (s0, res)
                k["scale"][part] = b
                changes.append(f"{part}: authority x{s0:g} -> x{b:g} ({'performance' if f else 'power'})")
    history["_last_up"] = new_last_up
    return k, v, changes, blocked


def node_key(disp, sized, build, extra=""):
    scen = json.loads(pathlib.Path(disp["scenario"]).read_text())
    prod, parts = product_blob(sized, scen["product"])
    return sha(scen, prod, parts, build, extra, case_bytes(scen["case"]))


def node_redundancy(sel, knobs):
    """The fault campaign's lever (node faults, select fault_policy gap): when the selected or closest family carries
    fluid rings and loses fine pointing to a single ring failure, fit the spare ring (adcs-design knob fmr_spare: a
    fourth, skewed ring that stands in for any one), once (design_loop_redundancy, design/looprules.pc)."""
    fam = sel["families"].get(sel["selected"], {})
    ring_gap = [g for g in fam.get("fault_gaps", []) if g.startswith("fault: rotor_fail")]
    if not call("looprules::loop_redundancy", "fmr" in sel["selected"].split("_"), bool(ring_gap), bool(knobs.get("fmr_spare")))[0]:
        return knobs, []
    k = json.loads(json.dumps(knobs))
    k["fmr_spare"] = True
    return k, [f"redundancy ({sel['selected']}): {ring_gap[0]} -> a spare fluid ring, skewed, that stands in for any one ring"]
