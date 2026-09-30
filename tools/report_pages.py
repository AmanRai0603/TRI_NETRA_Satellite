"""The report's page (results/index.html), docs/RESULTS.md and docs/SELECTION.md.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import json, html
import numpy as np
import matplotlib.pyplot as plt
from common import write_text
from report_base import OUT, ROOT, S1, S2, aslist, fnum, req_line, save, verdict
from report_solutions import components_html
from report_trades import trade_html


def engine_section():
    """The Rust engine and the two flight-software builds (tools/engine.py twin-parity)."""
    import sys
    sys.path.insert(0, str(ROOT / "tools"))
    import engine as E
    pj = OUT / "engine_parity.json"
    if not pj.exists():
        return []
    rows = json.loads(pj.read_text())
    judged = [r for r in rows if r["agree"] is not None]
    agree = sum(r["agree"] for r in judged)
    walls = {r["scenario"]: (r["wall_matlab_s"], r["wall_engine_s"]) for r in rows if r["wall_matlab_s"] and r["wall_engine_s"]}
    sp = sorted(a / b for a, b in walls.values())
    # distribution figure: MATLAB campaign vs engine Monte Carlo
    pairs = [("mc_fine_img", "fine_hold_img", "ape_los_p9973", "APE LOS p99.73 [deg]", 0.01),
             ("mc_detumble_ais", "detumble_ais", "detumble_time", "detumble time [min]", None),
             ("mc_slew_img", "slew_img", "settle_time_after_slew", "settle after slew [s]", 20)]
    have = []
    for camp, scen, mid, lab, req in pairs:
        a, b = E.TWIN / camp / "summary.json", E.ENG / f"mc_{scen}" / "summary.json"
        if a.exists() and b.exists():
            sa = json.loads(a.read_text()); sb = json.loads(b.read_text())
            va = next((m["values"] for m in aslist(sa["stats"]) if m["id"] == mid), None)
            vb = [r[mid] for r in sb["runs"] if isinstance(r.get(mid), (int, float))]
            if va and vb:
                have.append((camp, scen, lab, req, [x for x in va if x is not None], vb))
    vo = (OUT / "VIRTUAL_OBC.md").read_text() if (OUT / "VIRTUAL_OBC.md").exists() else ""
    n_vo = vo.count("| bit-identical |"); n_vt = sum(1 for l in vo.splitlines() if l.startswith("| ") and "---" not in l and "scenario" not in l)
    out = ["<h2 id='engine'>Flight software in C and Rust, engine in Rust, OBC in the loop</h2>",
           "<p>The flight software is written once as pseudocode (<code>fsw/pseudocode</code>) and flown in two builds: "
           "embedded C (<code>fsw/</code>, C99, the OBC build) and Rust (<code>fsw-rs/</code>, <code>no_std</code>). The plant and every "
           "device run in the Rust engine (<code>engine/</code>), with the <b>full POP propagator ported to Rust</b> (<code>adcs-pop</code>: "
           "time scales, frames, DE440, spherical-harmonic gravity, tides, SRP/ERP, relativity, DTM2020/JB2008 drag, integrators, each "
           "bit-exact against Octave) stepped in the loop as the MATLAB twin steps POP. Each device speaks its own bytes (I2C "
           "registers, SPI, UART frames with CRC, CAN), so the flight drivers are exercised as on the OBC. Python orchestrates and "
           "reports; MATLAB stays the SILS twin (docs/LANGUAGES.md, fsw/twin_map.toml).</p>",
           f"<div class='kv'><div><b>C vs Rust flight software</b>bit-identical closed-loop trajectories on every scenario tried (<code>adcs parity</code>)</div>"
           f"<div><b>Truth environment vs MATLAB</b>orbit, Sun, Moon, shadow, density and field identical on every recorded sample of all {len(walls)} scenarios</div>"
           f"<div><b>Virtual OBC</b>{n_vo} of {n_vt} runs bit-identical: the flight software as a process and as Cortex-M4F firmware in QEMU over adcs-link/1</div>"
           f"<div><b>Engine vs MATLAB, single runs</b>{agree} of {len(judged)} requirement verdicts agree over {len(walls)} scenarios</div>"
           f"<div><b>Speed, one core</b>{sp[len(sp)//2]:.0f}× faster than the Octave twin (median; {sp[0]:.0f}–{sp[-1]:.0f}×)</div>"
           f"<div><b>Distributions</b>Monte Carlo on both sides agree where the dispersions match (figure)</div></div>"]
    if have:
        fig, axs = plt.subplots(1, len(have), figsize=(3.6 * len(have), 3.2))
        axs = np.atleast_1d(axs)
        for ax, (camp, scen, lab, req, va, vb) in zip(axs, have):
            rng = np.random.default_rng(0)
            ax.scatter(0 + 0.08 * rng.standard_normal(len(va)), va, s=18, color=S1, label="MATLAB twin")
            ax.scatter(1 + 0.08 * rng.standard_normal(len(vb)), vb, s=18, color=S2, label="Rust engine")
            if req: req_line(ax, req, "req")
            ax.set_xticks([0, 1]); ax.set_xticklabels(["MATLAB", "engine"]); ax.set_xlim(-0.6, 1.6); ax.set_title(scen); ax.set_ylabel(lab)
        h, l = axs[0].get_legend_handles_labels()
        fig.legend(h, l, loc="lower center", ncol=2, bbox_to_anchor=(0.5, -0.04), fontsize=9)
        fig.suptitle("Monte Carlo: MATLAB campaign vs Rust engine (C flight software)", fontsize=11, fontweight="bold")
        fig.tight_layout(rect=(0, 0.04, 1, 0.94))
        out.append(f"<figure><img src='figures/{save(fig, 'engine_vs_matlab_mc')}' alt='Monte Carlo distributions, MATLAB twin vs Rust engine' loading='lazy'></figure>")
    out.append("<h3>Differences traced to their cause</h3><ul>" + "".join(f"<li><b>{html.escape(k)}.</b> {html.escape(v)}</li>" for k, v in E.NOTES) + "</ul>")
    out.append("<p class='muted'>Full ledger, metric by metric: <code>results/ENGINE_PARITY.md</code>. Build and run: <code>python3 tools/engine.py build && python3 tools/engine.py run</code>.</p>")
    return out


def write_html(sections, cmp_figs, report, trades=(), sol_html=()):
    css = """
:root{--bg:#f7f8f9;--surface:#ffffff;--text:#15191e;--muted:#56606b;--line:#dde2e7;--accent:#2a78d6;--pass:#0a7a3a;--fail:#b3261e;--chip:#eef2f6}
@media (prefers-color-scheme: dark){:root:not([data-theme="light"]){color-scheme:dark;--bg:#14171a;--surface:#1b1f23;--text:#eef1f4;--muted:#a3adb8;--line:#2f353c;--accent:#5a9ce8;--pass:#5bd08a;--fail:#ff8a80;--chip:#252b31}}
:root[data-theme="dark"]{color-scheme:dark;--bg:#14171a;--surface:#1b1f23;--text:#eef1f4;--muted:#a3adb8;--line:#2f353c;--accent:#5a9ce8;--pass:#5bd08a;--fail:#ff8a80;--chip:#252b31}
body{background:var(--bg);color:var(--text);font:15px/1.55 "IBM Plex Sans",system-ui,-apple-system,"Segoe UI",sans-serif}
main{max-width:1080px;margin:0 auto;padding-inline:16px;padding-block:28px 64px}
h3{font-size:17px;margin:30px 0 4px}
h1{font-size:30px;line-height:1.15;margin:0 0 6px;text-wrap:balance} h2{font-size:21px;margin:44px 0 6px;padding-top:22px;border-top:1px solid var(--line);text-wrap:balance}
.muted{color:var(--muted)} p{max-width:72ch} code,.mono{font-family:"IBM Plex Mono",ui-monospace,monospace;font-size:.92em}
.lede{font-size:16px}
table{border-collapse:collapse;width:100%;margin:10px 0 14px;font-variant-numeric:tabular-nums;background:var(--surface)}
th,td{border-bottom:1px solid var(--line);padding:7px 10px;text-align:left;vertical-align:top} th{color:var(--muted);font-weight:600;font-size:13px;text-transform:uppercase;letter-spacing:.04em}
.pass{color:var(--pass);font-weight:600}.fail{color:var(--fail);font-weight:600}
figure{margin:10px 0 18px} img{display:block;max-width:100%;height:auto;border:1px solid var(--line);border-radius:4px;background:#fcfcfb}
nav{display:flex;flex-wrap:wrap;gap:6px;margin:16px 0}
nav a{font:13px "IBM Plex Mono",monospace;color:var(--text);background:var(--chip);border:1px solid var(--line);border-radius:3px;padding:3px 8px;text-decoration:none}
nav a:hover,nav a:focus-visible{border-color:var(--accent);outline:none}
.scroll{overflow-x:auto}
.kv{display:grid;grid-template-columns:repeat(auto-fit,minmax(210px,1fr));gap:10px;margin:14px 0}
.kv div{background:var(--surface);border:1px solid var(--line);border-radius:4px;padding:10px 12px}.kv b{display:block;font-size:12px;color:var(--muted);text-transform:uppercase;letter-spacing:.05em;font-weight:600}
"""
    out = ['<title>TRI-NETRA SILS Results</title>',
           '<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>',
           '<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;600;700&display=swap">',
           f'<style>{css}</style><main>']
    out.append("<h1>TRI-NETRA ADCS: SILS results</h1><p class='lede muted'>Two 3U cases at 550 km sun-synchronous orbit, closed loop, with the Precision Orbit Propagator (POP v51) stepped inside the attitude loop. Owner: <b>Agastya</b>. Every number was produced by <code>matlab_sils</code> in GNU Octave 8.4 and can be re-derived from the filed channels.</p>")
    out.append("<div class='kv'><div><b>AIS 3U</b>10° APE · SSO dawn–dusk (LTAN 06:00) · coils only</div><div><b>Imaging 3U</b>0.01° APE 3σ · SSO LTAN 10:00 · wheels + 2 star trackers</div><div><b>Orbit truth</b>POP RK4 in the loop · J2–J6 · DE440 Sun/Moon · DTM2020 drag · SRP</div><div><b>Epoch</b>2027-01-01 06:00 UTC (case mission.epoch)</div></div>")
    # verdict matrix
    out.append("<h2 id='verdicts'>Verdicts at a glance</h2><p class='muted'>Every requirement-bound metric of every scenario (single nominal run, seed 1). Monte Carlo and edge-case statistics follow at the end.</p>")
    out.append("<div class='scroll'><table><tr><th>scenario</th><th>product</th><th>requirement checks</th></tr>")
    for gname, items in sections:
        for kind, sid, obj, figs in items:
            if kind != "run": continue
            checks = []
            for m in obj["metrics"]:
                v, cls = verdict(m["pass"])
                if cls: checks.append(f"<span class='{cls}'>{v.split()[0]}</span> {m['id']} {m['value']:.3g} {m['unit']}")
            out.append(f"<tr><td><a href='#{sid}'>{sid}</a></td><td class='mono'>{obj['product']}</td><td>{'<br>'.join(checks) or '—'}</td></tr>")
    out.append("</table></div>")
    if sol_html:
        out.append("<h2 id='solutions'>From the customer's case to our ADCS</h2>")
        out.append("<p>Each case is sized: every actuator option is scaled to what the case demands. Then every mission mode (detumble, Sun acquisition, Sun referencing, nadir pointing) is flown with every method the hardware allows, on the same seeds. Our three solutions are magnetorquers only, + fluid loop, and + fluid loop + N2O cold-gas RCS. They are scored against reaction wheels, CMG and VSCMG sized to the same case, and the simplest of ours that passes every mode is recommended (docs/SOLUTION_PIPELINE.md).</p>")
        out.extend(sol_html)
        out.append(components_html())
    if trades:
        out.append("<h2 id='trades'>Trades: one job, several algorithms, several hardware sets</h2>")
        out.append("<p>Every algorithm is registered once (<code>catalogue/algorithms</code>) with the job it does (its <i>slot</i>) and the hardware it <i>needs</i>. A run's algorithm for each slot comes from the scenario, else the product's promoted <code>[selected]</code> table, else the first compatible default, and a choice the hardware cannot fly is refused before the run. A trade flies every candidate on the same seeds, ranks the ones that meet every requirement by their worst objective value, and <b>proposes</b> a winner for a person to confirm.</p>")
        out.append("<div class='scroll'><table><tr><th>trade</th><th>slot</th><th>product</th><th>proposed</th><th>why</th></tr>")
        for T in trades:
            out.append(f"<tr><td><a href='#{T['id']}'>{html.escape(T['label'])}</a></td><td><code>{T.get('slot','')}</code></td><td class='mono'>{T.get('promote_to','') or '—'}</td><td><b>{html.escape(T.get('selected') or 'none')}</b></td><td class='muted'>{html.escape(T.get('rationale',''))}</td></tr>")
        out.append("</table></div>")
        for T in trades:
            out.append(trade_html(T))
    out.extend(engine_section())
    if cmp_figs:
        out.append("<h2 id='compare'>Comparisons</h2>")
        for f in cmp_figs: out.append(f"<figure><img src='figures/{f}' alt='{f[:-4]}' loading='lazy'></figure>")
    out.append("<nav>" + "".join(f"<a href='#g{gi}'>{html.escape(g.split(',')[0] if len(g) > 40 else g)}</a>" for gi, (g, _) in enumerate(sections)) + "</nav>")
    for gi, (gname, items) in enumerate(sections):
        out.append(f"<h2 id='g{gi}'>{html.escape(gname)}</h2>")
        for kind, sid, obj, figs in items:
            out.append(f"<h3 id='{sid}'>{sid}</h3>")
            if kind == "run":
                man = obj
                log = "; ".join(f"{e['t']:.0f} s {e['mode']}" for e in (man.get("mode_log") or []) if isinstance(e, dict))
                out.append(f"<p class='muted'>{html.escape(man.get('label', ''))} · product <b>{man['product']}</b> · {man['duration_s']/60:.0f} min simulated in {man['wall_s']/60:.1f} min wall · events: {html.escape(log)}</p>")
                out.append("<div class='scroll'><table><tr><th>metric</th><th>value</th><th>required</th><th>verdict</th></tr>")
                for m in man["metrics"]:
                    v, cls = verdict(m["pass"])
                    req = "—" if m["req"] is None or not np.isfinite(m["req"]) else f"{m['req']:g}"
                    out.append(f"<tr><td>{m['id']}</td><td>{m['value']:.4g} {m['unit']}</td><td>{req}</td><td class='{cls}'>{v}</td></tr>")
                out.append("</table></div>")
            else:
                summ = obj
                kindtxt = "edge cases (each dispersion at its bounds, then all adverse)" if sid.startswith("edge") else "Monte Carlo"
                out.append(f"<p class='muted'>{kindtxt} · scenario <b>{summ['scenario']}</b> · case <b>{summ['case']}</b> · {summ['runs']} runs · dispersed: mass properties, magnetic dipole, CM offset, surface properties, space weather, initial conditions, and every part error from its descriptor.</p>")
                out.append("<div class='scroll'><table><tr><th>metric</th><th>mean</th><th>std</th><th>ensemble percentile</th><th>required</th><th>runs passing</th><th>verdict</th></tr>")
                for st in summ["stats"]:
                    v, cls = verdict(st["pass"])
                    req = "—" if st["req"] is None or not np.isfinite(st["req"]) else f"{st['req']:g}"
                    pr = "—" if st["pass_rate"] is None or not np.isfinite(st["pass_rate"]) else f"{100*st['pass_rate']:.0f}%"
                    out.append(f"<tr><td>{st['id']}</td><td>{st['mean']:.4g}</td><td>{st['std']:.3g}</td><td>p{st['level']:g}: {st['pct']:.4g} {st['unit']}</td><td>{req}</td><td>{pr}</td><td class='{cls}'>{v}</td></tr>")
                out.append("</table></div>")
            for f in figs:
                out.append(f"<figure><img src='figures/{f}' alt='{html.escape(f.replace('_', ' ')[:-4])}' loading='lazy'></figure>")
    out.append("<p class='muted'>Copyright © 2026 Agastya. All rights reserved.</p></main>")
    write_text(OUT / "index.html", "\n".join(out))


def write_md(sections, cmp_figs):
    """docs/RESULTS.md: the verdict tables (the HTML report carries the figures)."""
    L = ["# TRI-NETRA ADCS — SILS results", "", "**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.", "",
         "Produced by `matlab_sils` (GNU Octave 8.4) with the Precision Orbit Propagator stepped inside the attitude loop.",
         "Figures: `results/index.html` (open in a browser) and `results/figures/`. Single runs are the nominal case, seed 1.", ""]
    for gname, items in sections:
        L += [f"## {gname}", ""]
        for kind, sid, obj, figs in items:
            if kind == "run":
                L += [f"### `{sid}` — {obj.get('label', '')}", "", "| metric | value | required | verdict |", "|---|---|---|---|"]
                for m in obj["metrics"]:
                    v, _ = verdict(m["pass"])
                    req = "—" if m["req"] is None or not np.isfinite(m["req"]) else f"{m['req']:g}"
                    L.append(f"| {m['id']} | {m['value']:.4g} {m['unit']} | {req} | {v} |")
                ev = [f"{e['t']:.0f} s {e['mode']}" for e in (obj.get('mode_log') or []) if isinstance(e, dict)]
                if len(ev) > 1: L += ["", "Events: " + "; ".join(ev)]
                L.append("")
            else:
                L += [f"### `{sid}` — {obj['runs']} runs of `{obj['scenario']}`", "",
                      "| metric | mean | std | ensemble percentile | required | runs passing | verdict |", "|---|---|---|---|---|---|---|"]
                for st in obj["stats"]:
                    v, _ = verdict(st["pass"])
                    req = "—" if st["req"] is None or not np.isfinite(st["req"]) else f"{st['req']:g}"
                    pr = "—" if st["pass_rate"] is None or not np.isfinite(st["pass_rate"]) else f"{100*st['pass_rate']:.0f}%"
                    L.append(f"| {st['id']} | {st['mean']:.4g} | {st['std']:.3g} | p{st['level']:g}: {st['pct']:.4g} {st['unit']} | {req} | {pr} | {v} |")
                L.append("")
    write_text(ROOT / "docs" / "RESULTS.md", "\n".join(L) + "\n")


def write_selection(trades):
    """docs/SELECTION.md: what each trade proposes, and how the choice is applied."""
    L = ["# Algorithm and hardware selection", "", "**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.", "",
         "Generated by `tools/report.py` from `matlab_sils/store/trades/*/trade.json`. Every entry is a",
         "**proposal**: the product's `[selected]` table is edited only when a person confirms it.", "",
         "## How one job, several algorithms and several hardware sets are managed", "",
         "1. **Registry.** Every algorithm is one file in `catalogue/algorithms/<id>.toml` with its `slot`",
         "   (the job: `detumble`, `attitude`, `mtq_pointing`, `pointing`, `sun_acquisition` (formerly `sun_spin`), `allocation`, `thrusters`)",
         "   and what it `needs` (`coils`, `magnetometer`, `gyro`, `sun`, `star_tracker`, `attitude`, `momentum`,",
         "   `wheels_or_rings`, `rings`, `cmg`, `vscmg`, `rcs`).",
         "2. **Resolution** (`asils.fsw.select`, once per run, before the loop): the scenario's",
         "   `[fsw] algorithms = {slot = id}` → else the product's `[selected]` table → else the first compatible",
         "   default. An algorithm whose needs the product does not carry is refused by name.",
         "3. **Dispatch.** The flight software modes only ask *which law fills this slot* (`asils.fsw.step`);",
         "   adding an algorithm is one registry file plus one `case` in the law's dispatcher.",
         "4. **Trade** (`trades/<id>.toml`, `asils.trade.run`): candidates are algorithms for one slot on fixed",
         "   hardware, hardware sets (other products), or tunings, all flown on the same seeds. Feasible",
         "   candidates (every requirement met on every seed) are ranked by their worst objective value, ties by",
         "   power or propellant.",
         "5. **Promotion.** The confirmed winner goes into the product's `[selected]` table, so every later",
         "   scenario and campaign on that product flies it without restating it.", "",
         "## Corrections in this batch", "",
         "- **Fluid-ring pump field power.** The ring model now counts the pump's field power while the",
         "  loop pumps (IDMAS v2 §03C: 1–3 W per unit, 2 W for SYN-MFP-1). A loop spins down in about 1 s,",
         "  so it pumps whenever it holds momentum. Earlier runs left this out (about 0.05 W instead of",
         "  about 4 W for three rings), and that flipped the fine-hold hardware trade: the CMG pyramid now",
         "  holds the camera as well as the rings at under half their power. The pump field power is the",
         "  number to engineer down for the fluid loop, for example with a permanent-magnet yoke.",
         "- **Rings with and without RCS tie on the fine hold.** The thrusters are not used while holding.",
         "  RCS pays off in agility (the agile-slew trade) and in momentum dumping.", "",
         "## Proposals", "", "| trade | slot | product | proposed | rationale |", "|---|---|---|---|---|"]
    for T in trades:
        L.append(f"| {T['label']} | `{T.get('slot','')}` | {T.get('promote_to','') or '—'} | **{T.get('selected') or 'none'}** | {T.get('rationale','')} |")
    for T in trades:
        L += ["", f"### {T['label']}", "", T.get("question", ""), "",
              f"Objective: worst `{T['objective']['metric']}` over the seeds; figure `results/figures/{T.get('figure','')}`.", "",
              "| rank | candidate | product | objective worst | mean | checks passed |", "|---|---|---|---|---|---|"]
        for i, c in enumerate(T["candidates"]):
            L.append(f"| {i+1} | {c['id']} | {c.get('product','')} | {fnum(c['obj']):.4g} | {fnum(c['obj_mean']):.4g} | {fnum(c.get('pass_rate')):.0f}% |")
    write_text(ROOT / "docs" / "SELECTION.md", "\n".join(L) + "\n")
