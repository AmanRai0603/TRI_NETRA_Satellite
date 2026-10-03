"""The design loop, node by node: from a customer case to a selected, dispatched, verified ADCS.

    case -> [size] -> [matrix] -> [assess] -> [converge] --not converged: resize / upgrade--> [size] ...
                                                  | converged
                                     [faults] -> [select] -> [dispatch] -> [mc] -> [soft_oils] -> report

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
  faults     once converged: each solution family's mission flown fault-free and once per single
             fault its product can carry (one coil, one rotor, the second star-tracker head, a gyro
             bias step, a GNSS outage, one RCS valve: nodes.json faults.set); a fault that breaks a
             requirement the fault-free run meets is a gap "fault: <kind>: <metrics>"
  select     the lightest SOLUTION family (least mass, then power, then volume: nodes.json
             select.rank_feasible) whose best option passes every mode and whose budget meets
             req.mass / req.vol; benchmarks ranked by the same rule. It runs in every iteration
             without the fault campaign and again after node faults, counting it under
             nodes.json select.fault_policy ("gap": not feasible; "rank": fewest failed faults first)
  dispatch   the selected family's flight configuration (adcs-fswcfg/1 blob) + C and Rust engine check
  mc         Monte Carlo of the dispatched mission (case dispersions, per-run seeds)
  soft_oils  the dispatched mission with the flight software as Cortex-M4F firmware (QEMU),
             exact instruction timing, next to its SILS run

Every node writes matlab_sils/store/pipeline/<case>/<node>.json; mode tests are cached by the
hash of (scenario, product, parts, seed, engine build), so an iteration flies only what changed.

  python3 tools/pipeline.py [case ...] [--seeds 1,2] [--max-iter 5] [--jobs N] [--mc-runs 12] [--no-oils]

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "tools"))
import argparse, json, os, shutil, sys
from common import Steps
from pipeline_base import (
    BIN, CACHE, CANDIDATES, DOWN, FAMILIES, FLOW_SIGMA_MIN,
    GYRO_MIN, IMPROVE, LAMBDA_MAX, LAMBDA_MIN, MS, NODES,
    OUT, P, PIPE, ROOT, SCALE_MAX, SCALE_MIN,
    SLOT, TUNE, UP, auth_part, case_bytes, cls,
    fam_violation, jl_, rate_violation, sha, split_alg, tune_grid,
    usable, write,
)
from pipeline_design import (
    node_assess, node_converge, node_key, node_matrix, node_redundancy, node_size, product_blob,
    run_job,
)
from pipeline_verify import (
    node_certify, node_dispatch, node_family_missions, node_faults, node_mc, node_robust, node_select,
    node_soft_oils, select_pick,
)
from pipeline_ledger import (
    PAPER, ledger, literature_table,
)

# re-exported: other tools and the tests use pipeline.<name>
__all__ = [
    'BIN', 'CACHE', 'CANDIDATES', 'DOWN', 'FAMILIES', 'FLOW_SIGMA_MIN',
    'GYRO_MIN', 'IMPROVE', 'LAMBDA_MAX', 'LAMBDA_MIN', 'MS', 'NODES',
    'OUT', 'P', 'PAPER', 'PIPE', 'ROOT', 'SCALE_MAX',
    'SCALE_MIN', 'SLOT', 'TUNE', 'UP', 'auth_part', 'case_bytes',
    'cls', 'fam_violation', 'jl_', 'ledger', 'literature_table', 'node_assess',
    'node_certify', 'node_converge', 'node_dispatch', 'node_family_missions', 'node_faults', 'node_key', 'node_matrix',
    'node_mc', 'node_robust', 'node_select', 'node_size', 'node_soft_oils', 'product_blob',
    'rate_violation', 'run_job', 'select_pick', 'sha', 'split_alg', 'tune_grid', 'usable',
    'write',
]


# ---------------------------------------------------------------- the loop
def run_case(case, a, modes, families, build):
    print(f"== {case}")
    S = Steps("pipeline.py", "pipeline")
    state = PIPE / case
    state.mkdir(parents=True, exist_ok=True)
    for old in state.glob("iter_*"):                    # a new loop starts from the laws as written
        shutil.rmtree(old)
    knobs, variants_on, history, log = {"scale": {}}, set(), {}, []
    it, robust, first = 0, [], 1
    for rpass in range(P("mc")["robustness_passes"] + 1):
      for it in range(first, a.max_iter + 1):
          print(f" iteration {it}: knobs {json.dumps(knobs)}")
          S(1, f"{case}, iteration {it}")
          sized, sizing = node_size(case, it, knobs)
          S(2, f"{case}, iteration {it}")
          tests = node_matrix(case, it, sized, modes, variants_on, [int(s) for s in a.seeds.split(",")], a.jobs, build, history.get("_tuned", []))
          S(3, f"{case}, iteration {it}")
          res = node_assess(tests, modes)
          S(4, f"{case}, iteration {it}")
          sel = node_select(case, res, sizing, modes, families)
          S(5, f"{case}, iteration {it}")
          knobs2, variants2, changes, blocked = node_converge(case, res, knobs, variants_on, history, sizing["class"] == "fine", modes, sel)
          entry = {"iteration": it, "knobs": knobs, "class": sizing["class"], "selected": sel["selected"], "status": sel["status"],
                   "feasible_options": sum(r["feasible"] for r in res.values()), "options": len(res), "changes": changes, "blocked": blocked,
                   "families": {f: {"feasible": v["feasible"], "gaps": v["gaps"], "budget": v["budget"]} for f, v in sel["families"].items()},
                   "matrix": [{**{k: r[k] for k in ("mode", "option", "alg", "feasible", "failing", "objective", "objective_id", "objective_req")},
                             **({"variants": r["variants"]} if r["option"] == "mtq" and "variants" in r else {})} for r in res.values()],
                 "tuned": history.get("_tuned", [])}
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
      S(6, case)
      faults = node_faults(case, sel, sized, modes, build, a.jobs)
      sel.update(select_pick(case, sel["families"], faults))
      print(f"  -> {sel['selected']} ({sel['status']}) with the fault campaign counted ({sel['fault_policy']})")
      # redundancy: a fluid-ring family that does not survive losing a ring gets the spare ring, and the
      # loop sizes, flies and converges again (its mass levers then work against the case budget)
      knobs2, changes = node_redundancy(sel, knobs)
      if changes:
          robust.append({"after_iteration": it, "family": sel["selected"], "fault_gaps": sel["families"][sel["selected"]].get("fault_gaps", []),
                         "changes": changes, "blocked": []})
          print("  redundancy: " + "; ".join(changes))
          knobs, first = knobs2, it + 1
          continue
      S(7, f"{case}: {sel['selected']}")
      disp = node_dispatch(case, sel, sized, modes, build)
      S(8, f"{case}: {a.mc_runs} runs" if a.mc_runs else f"{case}: skipped (--mc-runs 0)")
      mc = node_mc(case, disp, sized, a.mc_runs, a.jobs) if a.mc_runs else None
      # robustness: a requirement the Monte Carlo breaks is a failure the loop must fix (node mc)
      fails = [x for x in (mc or {}).get("stats", []) if x["pass"] is False]
      if not fails or not converged:
          break
      S(9, case)
      knobs2, changes, blocked = node_robust(sel, fails, knobs, history)
      robust.append({"after_iteration": it, "family": sel["selected"], "mc_failing": {x["id"]: x["pass_rate"] for x in fails},
                     "changes": changes, "blocked": blocked})
      fl = ", ".join(f"{x['id']} ({100 * x['pass_rate']:.0f} % pass)" for x in fails)
      print(f"  robustness: Monte Carlo fails {fl}" + "".join(f"; change: {c}" for c in changes) + "".join(f"; blocked: {x}" for x in blocked))
      if not changes:
          break
      knobs, first = knobs2, it + 1
    sel["robustness"] = robust
    write(state / "selection.json", sel)
    write(state / "loop.json", log)
    write(state / "dispatch.json", disp)
    S(10, case)
    node_family_missions(case, sel, sized, modes, build, a.mc_runs, a.jobs, disp, mc)
    S(11, case)
    node_certify(case)
    S(12, case if not a.no_oils else f"{case}: skipped (--no-oils)")
    so = node_soft_oils(case, disp, sized, build) if not a.no_oils else None
    S(13, case)
    ledger(case, sel, log, disp, mc, so, sizing)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cases", nargs="*")
    ap.add_argument("--seeds", default=",".join(map(str, P("matrix")["seeds"])))
    ap.add_argument("--max-iter", type=int, default=P("converge")["max_iterations"])
    ap.add_argument("--jobs", type=int, default=os.cpu_count())
    ap.add_argument("--mc-runs", type=int, default=P("mc")["runs"])
    ap.add_argument("--no-oils", action="store_true")
    ap.add_argument("--dry-run", action="store_true", help="say what the design loop would do, and do nothing")
    a = ap.parse_args()
    if a.dry_run:
        import trinetra
        trinetra.dry_run("pipeline.py", "pipeline")
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
