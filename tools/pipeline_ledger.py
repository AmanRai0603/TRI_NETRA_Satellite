"""The design ledger (results/DESIGN_<case>.md) and its literature table.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import math
from common import write_text
from pipeline_base import OUT, PIPE, jl_, split_alg


PAPER = {"mtq_lovera2004": "P1 Lovera & Astolfi 2004", "mtq_celani2015": "P4 Celani 2015", "mtq_avanzini2021": "P16 Avanzini et al. 2021",
         "mtq_celani2026": "P8 Celani 2026", "mtq_tango2013": "P3 TANGO 2013 (flown)", "sunspin_l1l2": "P11 UPMSat-2 -> P5 He et al. 2023",
         "sunspin_deruiter2011": "P11 -> P2 de Ruiter 2011", "sun_boresight_celani2026": "P8 Celani 2026", "sunspin_l1l2_e2": "P11 -> P5, eclipse E2",
         "sunspin_damped": "P11 -> P5, R_z floor", "mtq_pd": "baseline PD", "mtq_lqr": "baseline LQR", "mtq_smc": "baseline SMC", "mtq_rate_damp": "baseline rate damping"}


def literature_table(log):
    """Best variant per law (over its tuned gains) for the coils-only option of each mode, last iteration."""
    rows = []
    rank = lambda z: (not z["feasible"], len(z["failing"]), z["objective"] if z["objective"] is not None else math.inf)
    for r in log[-1]["matrix"]:
        if r["option"] != "mtq" or not r.get("variants"):
            continue
        best = {}
        for v in r["variants"]:
            law, tune = split_alg(v["alg"]) if v["alg"] else ("default", {})
            if law not in best or rank(v) < rank(best[law][0]):
                best[law] = (v, tune)
        for law, (v, tune) in sorted(best.items(), key=lambda x: rank(x[1][0])):
            rows.append({"mode": r["mode"], "law": law, "paper": PAPER.get(law, "—"), "gains": ", ".join(f"{k} {x:g}" for k, x in tune.items()) or "nominal",
                         "feasible": v["feasible"], "objective": v["objective"], "objective_id": r["objective_id"], "failing": list(v["failing"])})
    return rows


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
    for r in sel.get("robustness", []):
        L += ["", f"**Robustness (node `mc`) after iteration {r['after_iteration']}:** the Monte Carlo of `{r['family']}` failed "
              + ", ".join(f"{k} ({100 * v:.0f} % of runs pass)" for k, v in r["mc_failing"].items()) + ". "
              + " ".join(r["changes"] + r["blocked"])]
    if log[-1]["blocked"]:
        L += ["", "Why the loop stopped (nothing left that a knob can change):", ""] + [f"- {b}" for b in sorted(set(log[-1]["blocked"]))]
    L += ["", "## Every configuration compared (last iteration)", "",
          f"Selection rule (node `select`, docs/NODES.md): {sel.get('rule', '')}. The benchmarks are ranked by the same rule; "
          f"best benchmark: **`{sel.get('benchmark')}`** ({sel.get('benchmark_status', '')}).", "",
          "| family | role | rank | feasible | mass [kg] | power [W] | volume [L] | momentum actuator | gaps |", "|---|---|---:|---|---:|---:|---:|---|---|"]
    order = sorted(sel["families"], key=lambda f: (sel["families"][f]["role"] != "solution", not sel["families"][f]["feasible"],
                                                   sel["families"][f]["budget"]["mass_kg"]))
    for f in order:
        v = sel["families"][f]
        b = v["budget"]
        its = [x for x in sizing["families"][f]["items"] if x["slot"] in ("wheels", "cmg", "vscmg", "rings")]
        act = ", ".join(sorted({f"{x['part']} x{x['n']:g}" if x["slot"] != "rings" else "fluid loop (3 rings)" for x in its})) or "coils only"
        L.append(f"| {f} | {v['role']} | {v.get('rank', '—')} | {'yes' if v['feasible'] else 'no'} | {b['mass_kg']:.3f} | {b['power_W']:.2f} | "
                 f"{b['volume_L']:.3f} | {act} | {'; '.join(v['gaps']) or '—'} |")
    L += ["", f"## Selected methods ({sel['selected']})", "", "| mode | option | feasible | objective | algorithms | failing |", "|---|---|---|---:|---|---|"]
    for m, r in F["modes"].items():
        if r:
            L.append(f"| {m} | {r['option']} | {'yes' if r['feasible'] else 'no'} | {fmt(r['objective'])} {r['objective_id']} | "
                     f"{', '.join(f'{k}={v}' for k, v in (r['algorithms'] or {}).items() if v)} | {', '.join(r['failing']) or '—'} |")
    fm = jl_(PIPE / case / "families.json") or {}
    if fm:
        L += ["", "## Every solution family flown as the mission (node `family_missions`)", "",
              "Detumble -> Sun acquisition -> nadir with each family's best methods from the loop, C flight software (C = Rust bit for bit), "
              "and the Monte Carlo pass rate of each requirement metric.", "",
              "| family | selected | feasible | mass [kg] | methods | " + " | ".join(m for m in ("detumble_time", "ape_los_p9973", "ake_los_p9973", "power_mean")) + " | C = Rust | MC pass rates |",
              "|---|---|---|---:|---|" + "---:|" * 4 + "---|---|"]
        for f, v in fm.items():
            met = {x["id"]: x for x in (v["mission"]["c"] or [])}
            cell = lambda i: "—" if i not in met or met[i]["value"] is None else f"{met[i]['value']:.4g} {'✓' if met[i]['pass'] == 1 else '✗' if met[i]['pass'] == 0 else ''}"
            mcs = "; ".join(f"{s['id']} {100 * s['pass_rate']:.0f} %" for s in (v["mc"] or {}).get("stats", []) if s["pass_rate"] is not None) or "—"
            L.append(f"| {f} | {'yes' if v['selected'] else 'no'} | {'yes' if v['feasible'] else 'no'} | {v['budget']['mass_kg']:.3f} | "
                     f"{', '.join(f'{m}={o}' for m, o in v['methods'].items())} | " + " | ".join(cell(i) for i in ("detumble_time", "ape_los_p9973", "ake_los_p9973", "power_mean")) +
                     f" | {v['c_equals_rust_bitwise']} | {mcs} |")
    lit = literature_table(log)
    if lit:
        L += ["", "## Coils only: every law of the literature, each at its best gains (nodes `matrix` + `tune`)", "",
              "Each law's result is its worst seed at the gains that make that worst seed best (Bruni & Celani's min-max); "
              "the laws are listed by mode, best first. Sources: docs/MTQ_LITERATURE.md.", "",
              "| mode | law | paper | best gains | feasible | objective (worst seed) | failing |", "|---|---|---|---|---|---:|---|"]
        for row in lit:
            L.append(f"| {row['mode']} | {row['law']} | {row['paper']} | {row['gains']} | {'yes' if row['feasible'] else 'no'} | "
                     f"{fmt(row['objective'])} {row['objective_id']} | {', '.join(row['failing']) or '—'} |")
    fq = jl_(PIPE / case / "floquet.json")
    if fq:
        L += ["", "## Coils-only nadir loop certificate (node `certify`, Floquet multipliers, Celani 2026's method)", "",
              "The loop linearised about nadir with the gyroscopic and gravity-gradient terms, the coil duty and the field along one orbit; "
              "all |mu| < 1 certifies the periodic loop (a boresight law keeps one multiplier at 1 by design).", "",
              "| law | gains | max abs(mu) | certified |", "|---|---|---:|---|"]
        for x in fq["laws"]:
            L.append(f"| {x['law']}{' (dispatched)' if x['dispatched'] else ''} | {x['gains']} | {x['max_mu']:.4f} | {'yes' if x['certified'] else 'no'} |")
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
    write_text(OUT / f"DESIGN_{case}.md", "\n".join(L) + "\n")
    print(f"  wrote results/DESIGN_{case}.md")
