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
import argparse, os, sys
from engine_base import (
    BIN, DATA, ENG, OUT, ROOT, TWIN,
)
from engine_runs import (
    VOBC_PAIRS, build, fsw_parity, mc, one, parity_line,
    run, scenario_list, scenarios, vobc,
)
from engine_twin import (
    MC_PAIRS, NOTES, twin_parity,
)
from engine_solutions import (
    MODES_DIR, SOL, case_value, dispatch, mode_scenario, sol_job,
    solutions,
)
from engine_campaigns import (
    AP_NODES, CAMP, KP_NODES, camp_job, campaign, campaign_ledger,
    case_values, draw, kp2ap, summarise,
)
from engine_oils import (
    oils, oils_job, oils_ledger,
)

# re-exported: other tools and the tests use engine.<name>
__all__ = [
    'AP_NODES', 'BIN', 'CAMP', 'DATA', 'ENG', 'KP_NODES',
    'MC_PAIRS', 'MODES_DIR', 'NOTES', 'OUT', 'ROOT', 'SOL',
    'TWIN', 'VOBC_PAIRS', 'build', 'camp_job', 'campaign', 'campaign_ledger',
    'case_value', 'case_values', 'dispatch', 'draw', 'fsw_parity', 'kp2ap',
    'mc', 'mode_scenario', 'oils', 'oils_job', 'oils_ledger', 'one',
    'parity_line', 'run', 'scenario_list', 'scenarios', 'sol_job', 'solutions',
    'summarise', 'twin_parity', 'vobc',
]


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
    sp.add_parser("campaign-ledger").set_defaults(f=lambda a: campaign_ledger(announce=True))
    p = sp.add_parser("oils"); p.add_argument("scenarios", nargs="*"); p.add_argument("--fsw", default="qemu")
    p.add_argument("--cpi", type=float, nargs="+", default=None, help="also fly each scenario at each of these CPIs (the sweep)")
    p.add_argument("--duration", type=float, default=None); p.add_argument("--jobs", type=int, default=os.cpu_count()); p.set_defaults(f=oils)
    sp.add_parser("oils-ledger").set_defaults(f=lambda a: oils_ledger(announce=True))
    p = sp.add_parser("solutions"); p.add_argument("cases", nargs="*"); p.add_argument("--seeds", default="1,2"); p.add_argument("--fsw", default="c")
    p.add_argument("--jobs", type=int, default=os.cpu_count()); p.set_defaults(f=solutions)
    for p in sp.choices.values():
        p.add_argument("--dry-run", action="store_true", help="say what this command would do, and do nothing")
    a = ap.parse_args()
    if a.dry_run:
        import trinetra
        trinetra.dry_run("engine.py", a.cmd)
    if a.cmd not in ("build",) and not BIN.exists():
        sys.exit("engine not built: python3 tools/engine.py build")
    failed = a.f(a) or 0
    if failed:
        sys.exit(f"engine.py {a.cmd}: {failed} failure(s), each marked FAIL above; nothing failed was counted as passing")


if __name__ == "__main__":
    main()
