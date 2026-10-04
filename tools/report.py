"""Build the TRI-NETRA ADCS SILS results report from the filed runs.

Reads, never re-runs:
  matlab_sils/store/results/<scenario>/{manifest.json, channels.csv}
  matlab_sils/store/results/<campaign>/{summary.json, runs.csv, run_*.mat (not needed)}
  matlab_sils/store/trades/<trade>/trade.json
Writes:
  results/figures/<id>_*.svg      one figure set per test (drawn by adcs-plot: adcs figures, adcs plot)
  results/index.html               the report page (figures + verdict tables)
  results/summary.json             every metric, every verdict
  docs/RESULTS.md, docs/SELECTION.md

Copyright (c) 2026 Agastya. All rights reserved.
"""
import csv, json
from common import write_text
from report_base import (
    FIG, GRID, INK, INK2, MODES, OUT,
    ROOT, S1, S2, S3, S4, STORE,
    SURF, TRADES, aslist, case_req, fnum, load_manifest, load_run,
    metric, mval, plot, req_line, save, verdict,
)
from report_runs import (
    CAMPAIGNS, GROUPS, PRIMARY, campaign_figures, comparison_figures, run_figures,
)
from report_trades import (
    TRADE_ORDER, load_trades, seeds_of, trade_figure, trade_html,
)
from report_solutions import (
    MODE_ORDER, OURS, SIZED, SOLS, compare_figure, components_html,
    family_rows, load_solutions, option_figure, solution_html, write_solutions_md,
)
from report_pages import (
    engine_section, write_html, write_md, write_selection,
)

# re-exported: other tools and the tests use report.<name>
__all__ = [
    'CAMPAIGNS', 'FIG', 'GRID', 'GROUPS', 'INK', 'INK2',
    'MODES', 'MODE_ORDER', 'OURS', 'OUT', 'PRIMARY', 'ROOT',
    'S1', 'S2', 'S3', 'S4', 'SIZED', 'SOLS',
    'STORE', 'SURF', 'TRADES', 'TRADE_ORDER', 'aslist', 'campaign_figures',
    'case_req', 'compare_figure', 'comparison_figures', 'components_html', 'engine_section', 'family_rows',
    'fnum', 'load_manifest', 'load_run', 'load_solutions', 'load_trades', 'metric', 'mval',
    'option_figure', 'plot', 'req_line', 'run_figures', 'save', 'seeds_of', 'solution_html',
    'trade_figure', 'trade_html', 'verdict', 'write_html', 'write_md', 'write_selection',
    'write_solutions_md',
]


def main():
    FIG.mkdir(parents=True, exist_ok=True)
    report = {"owner": "Agastya", "scenarios": {}, "campaigns": {}}
    runs = {}
    for _, ids in GROUPS:
        for sid in ids:
            d = STORE / sid
            if (d / "manifest.json").exists():
                runs[sid] = (load_manifest(d), d)
    cmp_figs = comparison_figures(runs)
    sections = []
    for gname, ids in GROUPS:
        items = []
        for sid in ids:
            if sid not in runs: continue
            man, d = runs[sid]
            figs = run_figures(sid, man, d, full=sid in PRIMARY)
            report["scenarios"][sid] = {"case": man["case"], "product": man["product"], "label": man.get("label", sid),
                                        "metrics": man["metrics"], "wall_s": man["wall_s"], "duration_s": man["duration_s"],
                                        "mode_log": man.get("mode_log", []), "figures": figs}
            items.append(("run", sid, man, figs))
        sections.append((gname, items))
    mc_items = []
    for cid in CAMPAIGNS:
        d = STORE / cid
        if not (d / "summary.json").exists():
            continue
        summ = json.loads((d / "summary.json").read_text())
        if isinstance(summ["stats"], dict):
            summ["stats"] = [summ["stats"]]
        with open(d / "runs.csv") as f:
            rows = [{k: float(v) for k, v in r.items()} for r in csv.DictReader(f)]
        figs = campaign_figures(cid, summ, rows)
        report["campaigns"][cid] = {"summary": summ, "figures": figs}
        mc_items.append(("mc", cid, summ, figs))
    sections.append(("Monte Carlo and edge-case campaigns", mc_items))
    trades = load_trades()
    for T in trades:
        T["figure"] = trade_figure(T)
    report["trades"] = trades
    sols = load_solutions(); sol_html = []
    for S in sols:
        rows = family_rows(S)
        figs = [compare_figure(S, rows)] + [option_figure(S, m) for m in MODE_ORDER if m in S["modes"]]
        sol_html.append(solution_html(S, rows, figs))
        report.setdefault("solutions", {})[S["case"]] = {"recommended": S["recommended"], "verdict": S["verdict"], "families": rows}
    if sols: write_solutions_md(sols)
    write_text(OUT / "summary.json", json.dumps(report, indent=1, default=float))
    write_html(sections, cmp_figs, report, trades, sol_html)
    write_md(sections, cmp_figs)
    write_selection(trades)
    print(f"report: {sum(len(i) for _, i in sections)} items, {len(list(FIG.glob('*.svg')))} figures")


if __name__ == "__main__":
    main()
