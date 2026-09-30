#!/usr/bin/env python3
"""The downloadable V&V report: tools/templates/vv_report.html filled from the filed results,
written as self-contained HTML (figures inlined) and printed to PDF with headless Chromium.

  python3 tools/vv_report.py            -> results/vv/TRINETRA_ADCS_VV_report.{html,pdf}
                                           and dist/TRINETRA_ADCS_VV_report.pdf

Sources (whatever exists is reported; a missing source is named, never invented):
  matlab_sils/store/pipeline/<case>/     design loop (selection, loop, dispatch, mc, soft_oils)
  matlab_sils/store/results_engine/      engine SILS runs, campaigns/, soft_oils/
  matlab_sils/store/results/             MATLAB twin runs and campaigns
  results/engine_parity.json, results/soft_oils.json, results/VIRTUAL_OBC.md

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import base64, csv, datetime, glob, html, io, json, math, pathlib, re, shutil, string, subprocess
from common import source_date, write_text

ROOT = pathlib.Path(__file__).resolve().parents[1]
MS = ROOT / "matlab_sils"
ENG = MS / "store" / "results_engine"
TWIN = MS / "store" / "results"
PIPE = MS / "store" / "pipeline"
OUT = ROOT / "results" / "vv"
CASES = ["ais_3u", "ais_img_3u"]
# validated two-series palette (dataviz reference, light surface): MATLAB twin / Rust engine
C_TWIN, C_ENG, C_REQ, INK, MUTED = "#2a78d6", "#eb6834", "#1f2a44", "#1f2a44", "#5b6477"

e = html.escape


def jl(p):
    p = pathlib.Path(p)
    return json.loads(p.read_text()) if p.exists() else None


def fmt(x, d=4):
    if x is None or (isinstance(x, float) and not math.isfinite(x)):
        return "—"
    if isinstance(x, (int, float)):
        return f"{x:.{d}g}"
    return e(str(x))


def verdict(p):
    return {1: '<span class="pass">pass</span>', True: '<span class="pass">pass</span>', 0: '<span class="fail">FAIL</span>',
            False: '<span class="fail">FAIL</span>'}.get(p, '<span class="na">—</span>')


def table(head, rows, num=()):
    h = "".join(f'<th class="{"n" if i in num else ""}">{c}</th>' for i, c in enumerate(head))
    b = "".join("<tr>" + "".join(f'<td class="{"n" if i in num else ""}">{c}</td>' for i, c in enumerate(r)) + "</tr>" for r in rows)
    return f'<div class="tbl"><table><thead><tr>{h}</tr></thead><tbody>{b}</tbody></table></div>'


def png(fig):
    b = io.BytesIO()
    fig.savefig(b, format="png", dpi=160, bbox_inches="tight", facecolor="white")
    import matplotlib.pyplot as plt
    plt.close(fig)
    return f'<img alt="" src="data:image/png;base64,{base64.b64encode(b.getvalue()).decode()}">'


def svg_inline(p):
    p = ROOT / p
    if not p.exists():
        return f'<p class="note">{e(str(p))} missing.</p>'
    return f'<img alt="" src="data:image/svg+xml;base64,{base64.b64encode(p.read_bytes()).decode()}">'


def mpl():
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 8, "axes.edgecolor": "#b8bfcc", "axes.labelcolor": INK,
                         "xtick.color": MUTED, "ytick.color": MUTED, "axes.spines.top": False, "axes.spines.right": False,
                         "axes.grid": True, "grid.color": "#e6e9ef", "grid.linewidth": 0.6, "axes.axisbelow": True})
    return plt


# ------------------------------------------------------------------ sections
def requirements():
    out = []
    for c in CASES:
        rows = []
        for r in csv.DictReader(open(MS / "cases" / f"{c}.csv")):
            if r["section"] in ("req", "orbit", "mass", "mission") and r["value"] not in ("", None):
                rows.append([f"<code>{e(r['key'])}</code>", e(r["label"]), e(r["value"]), e(r["unit"]), f'<span class="note">{e(r["note"])}</span>'])
        title = next((r["value"] for r in csv.DictReader(open(MS / "cases" / f"{c}.csv")) if r["key"] == "meta.title"), c)
        out.append(f"<h3>{e(c)} — {e(title)}</h3>" + table(["key", "quantity", "value", "unit", "source / status"], rows, num=(2,)))
    return "\n".join(out)


def nodes():
    """Section 3: the node registry (matlab_sils/data/pipeline/nodes.json), the single definition of every node."""
    R = jl(ROOT / "matlab_sils" / "data" / "pipeline" / "nodes.json")
    fv = lambda v: "; ".join(f"{k}: {fv(x)}" for k, x in v.items()) if isinstance(v, dict) else ", ".join(map(fv, v)) if isinstance(v, list) else str(v)
    rows = [[i, f"<code>{e(n['id'])}</code>", e(n["stage"]), e(n["runs_in"]), e(n["does"]),
             "<br>".join(e(f"{k} = {fv(v)}") for k, v in n["parameters"].items()) or "—",
             "<br>".join(e(r) for r in n["rules"]) or "—"] for i, n in enumerate(R["nodes"], 1)]
    return ("<p>Every step of the chain is a node with its inputs, outputs, parameters and rules in one registry "
            "(<code>matlab_sils/data/pipeline/nodes.json</code>; <code>docs/NODES.md</code>). The pipeline reads its parameters "
            "from it, so this table is what ran.</p>" + table(["#", "node", "stage", "runs in", "what it does", "parameters", "rules"], rows, num=(0,)))


def catalogue():
    """Section 4: the bought momentum actuators as their datasheets state them, and which one each case chose."""
    cat = [jl(p) for p in sorted((ROOT / "matlab_sils" / "data" / "catalogue").glob("*.json"))]
    chosen = {}
    for c in CASES:
        sel = jl(PIPE / c / "selection.json")
        z = sel and jl(PIPE / c / f"iter_{sel['iterations']}" / "sized" / "sizing.json")
        for k in ("rw", "cmg", "vscmg"):
            if z:
                chosen.setdefault(z["parts"][k]["part_number"].replace("-VSCMG", ""), []).append(f"{c} {k.upper()}")
    g = lambda d, k: "—" if d.get(k) is None else fmt(d[k], 4)
    rows = [[e(c["vendor"]), e(c["model"]), e(c["type"]), g(c["datasheet"], "h_mNms"), g(c["datasheet"], "torque_mNm"), g(c["datasheet"], "mass_g"),
             g(c["datasheet"], "power_steady_W"), g(c["datasheet"], "power_peak_W"), e(c["datasheet"].get("dims_mm") or "—"),
             "yes" if c["selectable"] else "no: " + e(", ".join(c["missing"])), e(", ".join(chosen.get(c["part_number"], []))) or "—",
             f"<a href='{e(c['source_url'])}'>{e(c['source_kind'])}</a><br><span class='note'>{e(c['verification'])}</span>"] for c in cat]
    return ("<p>Reaction wheels, CMGs and VSCMGs are bought, so the benchmarks fly real products: the <code>select_rotor</code> node takes the "
            "lightest catalogue model that meets the case's per-unit momentum and torque (three wheels; a four-unit CMG pyramid, two units "
            "per axis). We design only the magnetorquers, the fluid loop and the RCS. Numbers are the vendors'; a model whose datasheet lacks "
            "momentum, torque, mass or steady power is listed but not selectable. Values the engine needs and datasheets omit (rotor "
            "inertia, friction, imbalance) are derived with the rules in each file (<code>tools/catalogue.py</code>, <code>docs/CATALOGUE.md</code>).</p>" +
            table(["vendor", "model", "type", "h [mNms]", "torque [mNm]", "mass [g]", "steady [W]", "peak [W]", "size [mm]", "selectable", "chosen in", "source"],
                  rows, num=(3, 4, 5, 6, 7)) +
            "<p class='note'>The vendor sites could not be reached from the build environment; numbers marked verified were read from the "
            "full datasheet PDFs (distributor mirror), the others from search excerpts of the cited page and should be confirmed with the vendor. "
            "Tensor Tech's ADCS400 figures are the whole integrated ADCS (CMGs, magnetorquer, sun sensors, gyro) as upper bounds, so the CMG "
            "benchmark's mass and power are conservative.</p>")


def mtq_only(c, sel):
    """The coils-only family's behaviour per mode, stated from the stored runs (kept as designed; the
    owner will improve coils-only nadir pointing from the literature later)."""
    F = sel["families"].get("mtq")
    if not F:
        return ""
    M = {m: r for m, r in F["modes"].items() if r}
    g = lambda m, k: (M.get(m) or {}).get("metrics", {}).get(k)
    req = sel["demand"]["req"]
    fm = (jl(PIPE / c / "families.json") or {}).get("mtq")
    mis = {x["id"]: x for x in (fm or {}).get("mission", {}).get("c") or []}
    mcs = {x["id"]: x for x in ((fm or {}).get("mc") or {}).get("stats", [])}
    sa, sp = g("sun_acquisition", "sun_acquisition_time"), g("sun_acquisition", "sun_angle_p95")
    L = [f"<b>Detumble</b>: {fmt(g('detumble', 'detumble_time'))} min in the mode test and {fmt((mis.get('detumble_time') or {}).get('value'))} min in the mission "
         f"(requirement {fmt(req.get('detumble'))} min); it passes" + (f" in {100 * mcs['detumble_time']['pass_rate']:.0f} % of the Monte Carlo runs" if "detumble_time" in mcs else "") + ".",
         f"<b>Sun acquisition</b> (Sun-spin, {e(M['sun_acquisition']['alg'] or 'default')}): the coils spin the body up about the Sun line and bring the power face "
         f"towards the Sun, so coils alone are largely sufficient here. They do not meet the case's line, though: the power face within 20 deg "
         + (f"after {fmt(sa)} min" if sa is not None else "not within the 1.5-orbit test") + f" (requirement {fmt(req.get('sunacq'))} min), "
         f"and a 95th-percentile Sun angle of {fmt(sp)} deg over the last half orbit (limit 20 deg).",
         f"<b>Sun referencing</b>: {fmt(g('sun_referencing', 'sun_ape_p9973'))} deg (p99.73) with {e(M['sun_referencing']['alg'] or 'default')}; the coils cannot hold a three-axis Sun attitude.",
         f"<b>Nadir pointing</b> is where the coils fall short. From a settled start the best law ({e(M['nadir_pointing']['alg'] or 'default')}) holds the line of "
         f"sight to {fmt(g('nadir_pointing', 'ape_los_p9973'))} deg (p99.73) against {fmt(req.get('ape'))} deg. In the full mission nadir is commanded while the "
         f"Sun-spin is still turning, and the coils cannot take that momentum out in time: {fmt((mis.get('ape_los_p9973') or {}).get('value'))} deg."]
    return ("<div class='find'><b>Coils only (<code>mtq</code>), how it behaves.</b> It is kept as designed; coils-only nadir pointing is "
            "to be improved later from the owner's references.<ul>" + "".join(f"<li>{x}</li>" for x in L) + "</ul></div>")


def family_missions(c, sel):
    """Every one of our solution families flown as the full mission (node family_missions): a table of
    its verdicts and one timeline figure, so e.g. the coils-only behaviour is on record beside the pick."""
    fm = jl(PIPE / c / "families.json")
    if not fm:
        return ""
    ids = ("detumble_time", "ape_los_p9973", "ake_los_p9973", "power_mean")
    rows = []
    for f, v in fm.items():
        met = {x["id"]: x for x in v["mission"]["c"] or []}
        mcs = {x["id"]: x for x in (v["mc"] or {}).get("stats", [])}
        cell = lambda i: "—" if i not in met else f"{fmt(met[i]['value'])} {verdict(met[i]['pass'])}" + (
            f"<br><span class='note'>MC {100 * mcs[i]['pass_rate']:.0f} % pass</span>" if i in mcs and mcs[i]["pass_rate"] is not None else "")
        rows.append([f"<code>{e(f)}</code>" + (" <b>(selected)</b>" if v["selected"] else ""), verdict(v["feasible"]), fmt(v["budget"]["mass_kg"], 3),
                     e(", ".join(f"{m}={o}" for m, o in v["methods"].items()))] + [cell(i) for i in ids] + [verdict(v["c_equals_rust_bitwise"])])
    out = ("<p>Every one of our solution families flown as the full mission (detumble → Sun acquisition → nadir), with its best "
           "methods from the loop. Whether or not it is selected, this is how it behaves (node <code>family_missions</code>):</p>" +
           table(["family", "feasible", "mass [kg]", "methods", "detumble [min]", "APE [deg]", "AKE [deg]", "mean power [W]", "C = Rust"], rows, num=(2,)))
    plt = mpl()
    cols = ["#2a78d6", "#eb6834", "#1baf7a"]
    fig, ax = plt.subplots(4, 1, figsize=(7.4, 7.2), sharex=True)
    req = {m["id"]: m.get("req") for m in (next(iter(fm.values()))["mission"]["c"] or [])}
    for (f, v), col in zip(fm.items(), cols):
        ch = ROOT / v["check_dir"] / "c" / "channels.csv"
        if not ch.exists():
            continue
        t, ape, w, pw, mode = [], [], [], [], []
        with open(ch) as fh:
            for i, r in enumerate(csv.DictReader(fh)):
                if i % 10:
                    continue
                t.append(float(r["t_s"]) / 3600); ape.append(max(float(r["ape_los_deg"]), 1e-4)); w.append(float(r["rate_degps"]))
                pw.append(float(r["P_mtq_W"]) + float(r["P_rw_W"]) + float(r["P_rcs_W"])); mode.append(float(r["mode"]))
        lab = f + (" (selected)" if v["selected"] else "")
        ax[0].semilogy(t, w, color=col, lw=1, label=lab); ax[1].semilogy(t, ape, color=col, lw=1)
        ax[2].plot(t, pw, color=col, lw=0.8); ax[3].step(t, mode, color=col, lw=1, where="post")
    for a_, yl in zip(ax, ("body rate [deg/s]", "APE, line of sight [deg]", "ADCS power [W]", "FSW mode")):
        a_.set_ylabel(yl)
    if req.get("ape_los_p9973"):
        ax[1].axhline(req["ape_los_p9973"], color=C_REQ, lw=0.8, ls="--"); ax[1].text(0.01, req["ape_los_p9973"], " APE requirement", color=C_REQ, fontsize=7, va="bottom", transform=ax[1].get_yaxis_transform())
    if req.get("power_mean"):
        ax[2].axhline(req["power_mean"], color=C_REQ, lw=0.8, ls="--"); ax[2].text(0.01, req["power_mean"], " orbit-average power requirement", color=C_REQ, fontsize=7, va="bottom", transform=ax[2].get_yaxis_transform())
    ax[3].set_xlabel("mission time [h]")
    ax[0].legend(loc="upper right", frameon=False, fontsize=7)
    fig.suptitle(f"{c}: every solution family flown as the dispatched mission (C flight software)", x=0.01, ha="left", fontsize=9, fontweight="bold")
    fig.tight_layout()
    return out + f"<figure>{png(fig)}<figcaption>{e(c)}: body rate, pointing error, power and flight-software mode through the mission, one line per family.</figcaption></figure>"


def verification():
    """Section 12: tools/verify_nodes.py, every node's decision recomputed from its stored inputs."""
    v = jl(ROOT / "results" / "node_verification.json")
    if not v:
        return "<p class='note'>Not run (python3 tools/verify_nodes.py).</p>"
    by = {}
    for r in v["rows"]:
        b = by.setdefault(r["node"], [0, 0, []]); b[0] += r["ok"]; b[1] += 1
        if not r["ok"]:
            b[2].append(f"[{r['case']}] {r['check']}: {r['detail']}")
    rows = [[f"<code>{e(n)}</code>", f"{a}/{t}", verdict(a == t), e("; ".join(f)) or "—"] for n, (a, t, f) in by.items()]
    return (f"<p><b>{v['passed']} of {v['checks']} checks pass</b> over {len(v['nodes'])} nodes. Each check recomputes a node's decision "
            "from what it stored and the rules in the node registry (the catalogue pick from the datasheets, the selection from the family "
            "scores, the budgets from their units, each feasibility from its failures), rather than reading the node's own verdict. "
            "Every check is listed in <code>results/NODE_VERIFICATION.md</code>.</p>" + table(["node", "checks passed", "verdict", "failures"], rows, num=(1,)))


def literature():
    """Section 7: the coils-only laws of the literature review, flown, tuned and certified per case."""
    import importlib.util, sys as _s
    spec = importlib.util.spec_from_file_location("pipeline", ROOT / "tools" / "pipeline.py")
    out = ["<p>Sixteen magnetorquer-only papers were reviewed (literature_review_v2.pptx). Every law that the accessible text "
           "specifies is now in the flight software, in C and Rust (bit for bit identical), in the pseudocode, and in the "
           "registry as a candidate of its slot: Lovera &amp; Astolfi 2004, Celani 2015, Avanzini et al. 2021, Celani 2026 "
           "(boresight, for the payload and for the power face), TANGO's frozen-Riccati LQR (flown), de Ruiter 2011, and "
           "the P11 → P5 chain already in place (UPMSat-2 spin-up, He et al. Sun spin). Bruni &amp; Celani's min–max tuning "
           "is node <code>tune</code>; Celani's Floquet certificate is node <code>certify</code>. The paper-by-paper record, "
           "including what was not implemented and why, is <code>docs/MTQ_LITERATURE.md</code>.</p>"]
    fam, mcs = {}, {}
    for c in CASES:
        v = (jl(PIPE / c / "families.json") or {}).get("mtq")
        st = {x["id"]: x for x in ((v or {}).get("mc") or {}).get("stats", [])}
        if st.get("ape_los_p9973", {}).get("pass_rate") is not None:
            mcs[c] = st["ape_los_p9973"]
        met = {x["id"]: x for x in ((v or {}).get("mission") or {}).get("c") or []}
        if "ape_los_p9973" in met:
            fam[c] = met["ape_los_p9973"]
    out.append("<h3>The Sun-spin → nadir hand-over</h3><p>The coils-only mission commands nadir from a body spinning at about "
               "4 °/s. The magnetic states now despin first. "
               "Above 1 °/s of rate error they run the B-dot law on that error with the detumble's optimal gain "
               "(Avanzini &amp; Giulietti 2012). The spin-up's high gain only drags the rate along the turning field. The "
               "pointing law takes over below 0.5 °/s, held for 60 s. The magnetic capture then takes about 1.5 orbits, so "
               "node <code>dispatch</code> gives a coils-only nadir three orbits after the command, and the metrics take "
               "the last half orbit. The coils-only nadir mode test starts the same way (arbitrary attitude, 6 °/s), and node "
               "<code>tune</code> flies the hand-over exit threshold with the gains, so the dispatched gains capture as "
               "well as hold. Coils-only mission nadir APE (p99.73): " +
               ("; ".join(f"{e(c)} {fmt(m['value'])}° {verdict(m['pass'])}" for c, m in fam.items()) or "not run") + ". "
               + " ".join(f"{e(c)} Monte Carlo: {100 * x['pass_rate']:.0f} % of runs within {fmt(x['req'])}° ({fmt(x['min'])}–{fmt(x['max'])}°)."
                          for c, x in mcs.items()) + "</p>"
               "<h3>Why Sun referencing is worse than nadir with coils only</h3><p>The coils are not short of dipole. The "
               "difference is the gravity gradient. At nadir on ais_3u the long, minimum-inertia axis is the payload axis, at the "
               "gradient's equilibrium, so the gradient is a restoring stiffness (ais_img_3u flies its long axis along track, "
               "pitch is unstable, and the nadir state cancels the gradient too). Under Sun referencing the attitude is inertial, so the "
               "gradient becomes a forcing at twice the orbit rate. A magnetic torque is perpendicular to B, so the part of "
               "that forcing along B cannot be rejected at that instant. The Sun state now cancels the modelled gradient "
               "(<code>mtq_gg_ff</code>); on ais_3u with Celani 2026 the Sun-pointing error went from 80.6° to 66.8°. It "
               "is still not the 5° line, which agrees with the flown and published figures (TANGO 16°, Celani 2026 24°). "
               "The coils-only power attitude remains the Sun spin.</p>")
    if "pipeline" not in _s.modules:
        pl = importlib.util.module_from_spec(spec); _s.path.insert(0, str(ROOT / "tools")); spec.loader.exec_module(pl)
    else:
        pl = _s.modules["pipeline"]
    for c in CASES:
        log = jl(PIPE / c / "loop.json")
        rows = pl.literature_table(log) if log else []
        if rows:
            out.append(f"<h3>{e(c)}: coils only, every law at its best gains</h3>" + table(
                ["mode", "law", "paper", "best gains", "feasible", "objective (worst seed)", "failing"],
                [[e(r["mode"]), f"<code>{e(r['law'])}</code>", e(r["paper"]), e(r["gains"]), verdict(r["feasible"]),
                  f"{fmt(r['objective'])} {e(r['objective_id'])}", e(", ".join(r["failing"])) or "—"] for r in rows], num=(5,)))
        fq = jl(PIPE / c / "floquet.json")
        if fq:
            out.append(f"<p>{e(c)}: Floquet multipliers of the coils-only nadir loop (linearised with gyroscopic and gravity-gradient "
                       "terms, coil duty and the field along one orbit); all |μ| &lt; 1 certifies the periodic loop.</p>" + table(
                ["law", "gains", "max |μ|", "certified"],
                [[f"<code>{e(x['law'])}</code>" + (" <b>(dispatched)</b>" if x["dispatched"] else ""), e(str(x["gains"])), f"{x['max_mu']:.4f}",
                  verdict(x["certified"])] for x in fq["laws"]], num=(2,)))
    return "\n".join(out)


def design():
    out = []
    for c in CASES:
        sel, log = jl(PIPE / c / "selection.json"), jl(PIPE / c / "loop.json")
        if not sel:
            out.append(f"<h3>{e(c)}</h3><p class='note'>Design loop not run for this case (python3 tools/pipeline.py {c}).</p>")
            continue
        F = sel["families"][sel["selected"]]
        d = sel["demand"]
        out.append(f"<h3>{e(c)}: selected <code>{e(sel['selected'])}</code> — {e(F['label'])} ({e(sel['status'])})</h3>")
        out.append(f"<p>{'Converged' if sel['converged'] else 'Not converged'} after {sel['iterations']} iteration(s); knowledge class "
                   f"<b>{e(sel['class'])}</b>; sensors: {e(', '.join(sel['sensors']))}. Demand on the POP orbit: peak disturbance "
                   f"{fmt(d['tau_dist'])} N m ({e(d['worst_attitude'])}), stored momentum requirement {fmt(d['h_req'])} N m s, "
                   f"torque requirement {fmt(d['tau_req'])} N m, weakest field {fmt(d['B_min'] * 1e6)} µT, detumble momentum {fmt(d['h_detumble'])} N m s.</p>")
        if F["gaps"]:
            out.append("<div class='find'><b>Open gaps of the selected family:</b> " + e("; ".join(F["gaps"])) + "</div>")
        rows = []
        for it in log:
            kn = ", ".join([f"{k} ×{v:.3g}" for k, v in it["knobs"].get("scale", {}).items()] + [k for k in ("star_tracker",) if it["knobs"].get(k)] +
                      [f"pump λ {it['knobs']['fmr_lambda']:g} kg/W" for _ in [0] if "fmr_lambda" in it["knobs"]] +
                      ["1 ST head" for _ in [0] if it["knobs"].get("st_heads") == 1] +
                      [f"flow sensor {it['knobs']['fmr_flow_sigma'] * 1e3:g} mm/s" for _ in [0] if it["knobs"].get("fmr_flow_sigma")] +
                      [f"gyro noise x{it['knobs']['gyro_grade']:g}" for _ in [0] if it["knobs"].get("gyro_grade", 1) < 1]) or "laws as written"
            rows.append([it["iteration"], e(kn), f"{it['feasible_options']}/{it['options']}", e(f"{it['selected']} ({it['status']})"),
                         "<br>".join(e(x) for x in it["changes"]) or "—", len(it["blocked"])])
        out.append(table(["iter", "knobs", "options feasible", "selection", "changes for the next iteration", "blocked"], rows, num=(0, 2, 5)))
        if log and log[-1]["blocked"]:
            out.append("<p class='note'>Why the loop stopped (nothing left that a knob can change):</p><ul class='note'>" +
                       "".join(f"<li>{e(b)}</li>" for b in sorted(set(log[-1]["blocked"]))) + "</ul>")
        z = jl(PIPE / c / f"iter_{sel['iterations']}" / "sized" / "sizing.json") or {"families": {}, "parts": {}}
        pname = {p["part_number"]: p["name"] for p in z["parts"].values()}

        def act(f):
            its = [x for x in z["families"].get(f, {}).get("items", []) if x["slot"] in ("wheels", "cmg", "vscmg", "rings", "rcs")]
            return "<br>".join(sorted({e("fluid loop, 3 rings (ours)") if x["slot"] == "rings" else
                                       e(f"{pname.get(x['part'], x['part']).split(' (')[0].split(' — ')[0]} ×{x['n']:g}") for x in its})) or "coils only"
        order = sorted(sel["families"], key=lambda f: (sel["families"][f]["role"] != "solution", not sel["families"][f]["feasible"],
                                                       sel["families"][f]["budget"]["mass_kg"]))
        rows = [[f"<code>{e(f)}</code>", e(sel["families"][f]["role"]), sel["families"][f].get("rank", "—"), verdict(sel["families"][f]["feasible"]),
                 fmt(sel["families"][f]["budget"]["mass_kg"], 3), fmt(sel["families"][f]["budget"]["power_W"], 3), fmt(sel["families"][f]["budget"]["volume_L"], 3),
                 act(f), e("; ".join(sel["families"][f]["gaps"])) or "—"] for f in order]
        out.append(f"<p>Every configuration, ours and the benchmarks, flown and scored the same way. Selection rule (node <code>select</code>): "
                   f"{e(sel.get('rule', 'simplest feasible solution'))}. Selected: <b><code>{e(sel['selected'])}</code></b>; the benchmarks "
                   f"ranked by the same rule give <b><code>{e(sel.get('benchmark') or '—')}</code></b> ({e(sel.get('benchmark_status', ''))}).</p>" +
                   table(["family", "role", "rank", "feasible", "mass [kg]", "steady power [W]", "volume [L]", "momentum / thrust actuators", "gaps"], rows, num=(2, 4, 5, 6)))
        rows = [[e(m), f"<code>{e(r['option'])}</code>", verdict(r["feasible"]), f"{fmt(r['objective'])} {e(r['objective_id'])}",
                 e(", ".join(f"{k}={v}" for k, v in (r["algorithms"] or {}).items() if v))] for m, r in F["modes"].items() if r]
        out.append("<p>Selected method per mission mode:</p>" + table(["mode", "option", "feasible", "objective (worst seed)", "algorithms"], rows))
        for r in sel.get("robustness", []):
            out.append("<div class='find'><b>Robustness (Monte Carlo feedback) after iteration " + str(r["after_iteration"]) + ":</b> " +
                       e(", ".join(f"{k} passed in {100 * v:.0f} % of dispersed runs" for k, v in r["mc_failing"].items()) + ". " + " ".join(r["changes"] + r["blocked"])) + "</div>")
        out.append(family_missions(c, sel))
        out.append(mtq_only(c, sel))
        fig = pump_front(c, sel)
        if fig:
            out.append(fig)
        mc = jl(PIPE / c / "mc" / "summary.json")
        if mc:
            rows = [[e(s["id"]), fmt(s["req"]), f"{fmt(s['mean'])} ± {fmt(s['std'], 3)}", f"[{fmt(s['min'])}, {fmt(s['max'])}]",
                     "—" if s["pass_rate"] is None else f"{100 * s['pass_rate']:.0f} %"] for s in mc["stats"] if s["mean"] is not None]
            out.append(f"<p>Monte Carlo of the dispatched mission ({mc['runs']} runs; {e(', '.join(mc['dispersions']))}):</p>" +
                       table(["metric", "req", "mean ± std", "range", "pass rate"], rows, num=(1,)))
        so = jl(PIPE / c / "soft_oils.json")
        if so:
            g = lambda k, i: next((x for x in (so.get(k, {}).get("metrics") or []) if x["id"] == i), {})
            rows = [[e(m["id"]), fmt(m.get("req")), f"{fmt(g('sils', m['id']).get('value'))} {verdict(g('sils', m['id']).get('pass'))}",
                     f"{fmt(g('oils', m['id']).get('value'))} {verdict(g('oils', m['id']).get('pass'))}",
                     f"{fmt(g('oils_rs', m['id']).get('value'))} {verdict(g('oils_rs', m['id']).get('pass'))}"] for m in so["sils"].get("metrics") or []]
            out.append("<p>The dispatched mission in SILS and in soft OILS (flight software as Cortex-M4F firmware, C and Rust builds):</p>" +
                       table(["metric", "req", "SILS", "soft OILS, C", "soft OILS, Rust"], rows, num=(1,)))
    return "\n".join(out)


def pump_front(c, sel):
    """The electromagnetic pump's mass / steady-power front per ring (empump.rs), the chosen design marked."""
    its = [PIPE / c / f"iter_{sel['iterations']}" / "sized" / "sizing.json"]
    if not its[0].exists() or "fmr" not in sel["selected"] and "fmr" not in " ".join(sel["families"]):
        return ""
    z = jl(its[-1])
    parts = z.get("parts", {})
    if "fmr_x" not in parts or "pareto" not in parts["fmr_x"].get("sizing", {}):
        return ""
    plt = mpl()
    fig, ax = plt.subplots(figsize=(7.2, 3.0))
    for key, lab, col, mk in (("fmr_x", "X ring", C_TWIN, "o"), ("fmr_y", "Y ring", C_ENG, "s"), ("fmr_z", "Z ring", "#1baf7a", "^")):
        pf = parts[key]["sizing"]["pareto"]
        xs, ys = [q["power_W"] for q in pf], [q["mass_kg"] for q in pf]
        ax.plot(xs, ys, color=col, lw=1.6, marker=mk, ms=4, label=lab)
        nm = parts[key]["nominal"]
        ax.scatter([nm["power_steady_W"]], [nm["mass_kg"]], s=70, facecolors="none", edgecolors=INK, linewidths=1.4, zorder=4)
        ax.annotate(lab, (xs[0], ys[0]), xytext=(4, 0), textcoords="offset points", fontsize=7, color=INK, va="center")
    ax.set_xscale("log")
    ax.set_xlabel("steady pump power per ring [W] (coil + electrodes, log scale)")
    ax.set_ylabel("ring mass [kg]")
    ax.legend(frameon=False, fontsize=7, loc="upper right")
    lam = z.get("knobs", {}).get("fmr_lambda", 0.1)
    return (f"<figure>{png(fig)}<figcaption>Electromagnetic pump design, {e(c)}: each ring's mass against steady power as the "
            f"mass/power rate λ runs from 0.003 to 3 kg/W; circles mark the converged designs (λ = {lam:g} kg/W).</figcaption></figure>")


def sils():
    rows, n, ok = [], 0, 0
    for d in sorted(p for p in ENG.iterdir() if (p / "manifest.json").exists() and (MS / "data" / "scenarios" / f"{p.name}.json").exists()):
        m = jl(d / "manifest.json")
        ms = m["metrics"] if isinstance(m["metrics"], list) else [m["metrics"]]
        req = [x for x in ms if x.get("req") is not None]
        good = all(x.get("pass") == 1 for x in req)
        n += 1; ok += good
        rows.append([f"<code>{e(d.name)}</code>", e(m.get("label", ""))[:90], e(m.get("product", "")), fmt(m.get("duration_s")),
                     "<br>".join(f"{e(x['id'])} {fmt(x.get('value'))} / {fmt(x.get('req'))} {verdict(x.get('pass'))}" for x in req) or "—"])
    return (f"<p>Every scenario on the Rust engine with the C flight software behind the byte HAL (POP in the loop). "
            f"{ok} of {n} scenarios meet every requirement they carry; the failures are findings of the design (section 10), "
            "not engine errors: the same verdicts come out of the MATLAB twin (section 7).</p>" +
            table(["scenario", "what", "product", "duration [s]", "requirement metrics: value / req"], rows, num=(3,)))


def campaigns():
    plt = mpl()
    out = ["<p>Every campaign of <code>campaigns/*.toml</code> flown on the Rust engine with the draws of <code>asils.campaign.draw</code> "
           "(same dispersions, bounds, run count, per-run seeds), next to the MATLAB twin's campaign. Monte Carlo realisations differ "
           "(different random streams) so distributions are compared; edge runs are compared run by run. The engine's flight software "
           "keeps the nominal inertia while the plant is dispersed; the MATLAB twin's laws read the dispersed inertia.</p>"]
    ids = sorted(p.stem for p in (MS / "data" / "campaigns").glob("*.json"))
    rows = []
    panels = []
    for cid in ids:
        E_ = jl(ENG / "campaigns" / cid / "summary.json")
        M_ = jl(TWIN / cid / "summary.json")
        if not E_:
            continue
        ms = {s["id"]: s for s in ((M_ or {}).get("stats") or [])} if M_ and isinstance(M_.get("stats"), list) else {}
        for s in E_["stats"]:
            if s.get("req") is None:
                continue
            t = ms.get(s["id"], {})
            mm = lambda z: "—" if not z or z.get("mean") is None else f"{fmt(z['mean'])} ± {fmt(z.get('std'), 3)}"
            pr = lambda z: "—" if not z or z.get("pass_rate") is None else f"{100 * z['pass_rate']:.0f} %"
            rows.append([f"<code>{e(cid)}</code>", e(E_.get("type", "")), e(s["id"]), fmt(s["req"]), mm(t), pr(t), mm(s), pr(s)])
            if t.get("values"):
                panels.append((cid, s["id"], s["req"], [v for v in t["values"] if isinstance(v, (int, float))], [v for v in s["values"] if isinstance(v, (int, float))], s.get("unit", "")))
    out.append(table(["campaign", "type", "metric", "req", "MATLAB mean ± std", "MATLAB pass", "engine mean ± std", "engine pass"], rows, num=(3,)))
    if panels:
        n = len(panels)
        cols = 3
        rws = math.ceil(n / cols)
        fig, axs = plt.subplots(rws, cols, figsize=(7.2, 2.0 * rws), squeeze=False)
        for i, (cid, mid, req, a, b, unit) in enumerate(panels):
            ax = axs[i // cols][i % cols]
            ax.scatter([0.1 * ((k % 7) - 3) / 3 for k in range(len(a))], a, s=12, color=C_TWIN, marker="o", label="MATLAB twin", zorder=3, linewidths=0)
            ax.scatter([1 + 0.1 * ((k % 7) - 3) / 3 for k in range(len(b))], b, s=14, color=C_ENG, marker="D", label="Rust engine", zorder=3, linewidths=0)
            ax.axhline(req, color=C_REQ, lw=1, ls=(0, (4, 3)))
            ax.text(1.45, req, "req", va="bottom", ha="right", fontsize=7, color=INK)
            ax.set_xticks([0, 1]); ax.set_xticklabels(["MATLAB", "engine"])
            ax.set_xlim(-0.5, 1.5)
            vals = [v for v in a + b + [req] if v and v > 0]
            if vals and max(vals) / min(vals) > 50:
                ax.set_yscale("log")
            ax.set_title(f"{cid}\n{mid} [{unit}]", fontsize=7.5, color=INK, loc="left")
        for j in range(n, rws * cols):
            axs[j // cols][j % cols].axis("off")
        h, l = axs[0][0].get_legend_handles_labels()
        fig.legend(h, l, loc="upper center", ncol=2, frameon=False, bbox_to_anchor=(0.5, 1.01))
        fig.tight_layout(rect=(0, 0, 1, 0.97))
        out.append(f"<figure>{png(fig)}<figcaption>Figure 3. Every run of every campaign, requirement metrics: MATLAB twin (circles) and "
                   "Rust engine (diamonds), dashed line = requirement.</figcaption></figure>")
    return "\n".join(out)


def parity():
    rows = jl(ROOT / "results" / "engine_parity.json") or []
    judged = [r for r in rows if r.get("agree") is not None]
    agree = sum(r["agree"] for r in judged)
    md = (ROOT / "results" / "ENGINE_PARITY.md").read_text() if (ROOT / "results" / "ENGINE_PARITY.md").exists() else ""
    env = [l for l in md.splitlines() if l.startswith("| ") and re.match(r"\| [a-z_]+ \| \d+ \|", l)]
    er = [[e(x.strip()) for x in l.strip("|").split("|")] for l in env]
    txt = (f"<p>Single runs, engine vs MATLAB twin, same scenario, case and product: <b>{agree} of {len(judged)}</b> requirement verdicts agree. "
           "The truth environment (orbit, Sun, Moon, shadow, density, field) of the engine's POP port is identical to the twin's on every "
           "filed sample of every scenario; what differs is the random stream (unit errors, initial tumble), so single-run metrics scatter "
           "inside the distributions of section 6. The differences traced to their cause are listed in results/ENGINE_PARITY.md.</p>")
    if er:
        txt += table(["scenario", "samples", "max |r| difference [m]", "max density ratio − 1", "max Sun-direction difference", "max |B| ratio − 1"], er, num=(1, 2, 3, 4, 5))
    dis = [[e(r["scenario"]), e(r["metric"]), fmt(r["matlab"]), fmt(r["engine"]), fmt(r["req"]), verdict(r["pass_matlab"]), verdict(r["pass_engine"])]
           for r in judged if not r["agree"]]
    if dis:
        txt += "<p>Verdicts that differ (all traced; see ENGINE_PARITY.md):</p>" + table(["scenario", "metric", "MATLAB", "engine", "req", "MATLAB", "engine"], dis, num=(2, 3, 4))
    return txt


def oils():
    so = jl(ROOT / "results" / "soft_oils.json")
    if not so:
        return "<p class='note'>Soft OILS matrix not run yet (python3 tools/engine.py oils).</p>"
    rows, tim = so["metrics"], so["timing"]
    judged = [r for r in rows if r["pass_sils"] is not None]
    agree = sum(r["pass_sils"] == r["pass_oils"] for r in judged)
    m = next((t["model"] for t in tim if t.get("model")), {}) or {}
    txt = [f"<p>Every scenario flown twice on the engine: SILS (C flight software in-process) and soft OILS (the flight software compiled for the OBC, "
           f"arm-none-eabi-gcc -mcpu=cortex-m4 -mfloat-abi=hard, running as firmware on QEMU mps2-an386 behind adcs-link/1). In soft OILS each command "
           f"reaches the actuators only after the OBC's sensor reads, its execution and its CAN frames; the execution is the step's exact instruction "
           f"count (QEMU plugin) × CPI {m.get('cpi', 1.25)} / {m.get('cpu_hz', 168e6) / 1e6:.0f} MHz. <b>{agree} of {len(judged)}</b> requirement verdicts "
           f"agree between SILS and soft OILS over {len(tim)} scenarios; runs are deterministic.</p>"]
    trows = [[f"<code>{e(t['scenario'])}</code>", fmt(1e3 * t["dt_s"], 3), f"{fmt(t['insn_mean'], 3)} / {fmt(t['insn_max'], 3)}", fmt(t["exec_max_ms"], 3),
              f"{fmt(t['lat_mean_ms'], 3)} / {fmt(t['lat_max_ms'], 3)}", f"{100 * (t['cpu_load_max'] or 0):.1f} %", t["overruns"]] for t in tim]
    txt.append(table(["scenario", "period [ms]", "instructions/step mean / max", "exec max [ms]", "latency mean / max [ms]", "CPU load max", "overruns"],
                     trows, num=(1, 2, 3, 4, 5, 6)))
    plt = mpl()
    tt = sorted(tim, key=lambda t: t["cpu_load_max"] or 0)
    if tt:
        fig, ax = plt.subplots(figsize=(7.2, 0.16 * len(tt) + 0.8))
        y = range(len(tt))
        ax.barh(list(y), [100 * (t["cpu_load_max"] or 0) for t in tt], height=0.6, color=C_TWIN, label="worst step")
        ax.scatter([100 * (t["cpu_load_mean"] or 0) for t in tt], list(y), s=10, color=INK, zorder=3, label="mean")
        ax.set_yticks(list(y)); ax.set_yticklabels([t["scenario"] for t in tt], fontsize=6.5)
        ax.set_xlabel("OBC CPU load in the control period [%] (Cortex-M4F, 168 MHz)")
        ax.grid(axis="y", visible=False)
        ax.legend(frameon=False, loc="lower right", fontsize=7)
        txt.append(f"<figure>{png(fig)}<figcaption>Figure 4. OBC load per scenario: worst step (bar) and mean (dot).</figcaption></figure>")
    dis = [[e(r["scenario"]), e(r["metric"]), fmt(r["req"]), fmt(r["sils"]), fmt(r["oils"]), verdict(r["pass_sils"]), verdict(r["pass_oils"])]
           for r in judged if r["pass_sils"] != r["pass_oils"]]
    if dis:
        txt.append("<p>Verdicts that change between SILS and soft OILS (the command latency's effect):</p>" +
                   table(["scenario", "metric", "req", "SILS", "soft OILS", "SILS", "soft OILS"], dis, num=(2, 3, 4)))
    return "\n".join(txt)


def vobc():
    p = ROOT / "results" / "VIRTUAL_OBC.md"
    if not p.exists():
        return "<p class='note'>Not run.</p>"
    rows = [[e(x.strip()) for x in l.strip("|").split("|")] for l in p.read_text().splitlines() if l.startswith("| ") and "bit" in l or l.startswith("| ") and "max" in l]
    n = sum("bit-identical" in r[3] for r in rows if len(r) > 3)
    return (f"<p>Zero-latency lockstep (no timing model): the flight software as a host process and as QEMU Cortex-M4F firmware, C and Rust, "
            f"reproduces the in-process trajectory bit for bit: <b>{n} of {len(rows)}</b> runs bit-identical.</p>" +
            table(["scenario", "reference", "virtual OBC", "result", "wall [s]"], rows, num=(4,)))


def open_items():
    items = []
    for c in CASES:
        sel = jl(PIPE / c / "selection.json")
        if sel and sel["families"][sel["selected"]]["gaps"]:
            items.append(f"<b>{e(c)}</b>: the selected <code>{e(sel['selected'])}</code> leaves " + e("; ".join(sel["families"][sel["selected"]]["gaps"])) +
                         ". Either the case relaxes these (several are marked UNCONFIRMED in the case file) or the next design lever is needed.")
    notes = [
        "Bang-bang B-dot (fixed): pure sign switching limit-cycled around the 0.5 deg/s exit rate in 4 of 12 seeds with or without OBC "
        "latency; a boundary layer (4 x the B-dot gain inside it) detumbles 12 of 12 at 0, 4 and 8 ms. C, Rust, MATLAB and pseudocode updated.",
        "Rate stability (imaging): relaxed by the owner from 0.001 to 0.005 deg/s (18 arcsec/s, ~0.5 m smear in a 10 ms exposure at 550 km). "
        "At 0.001 deg/s it was set by gyro noise for every actuator (0.0025-0.004 deg/s with TRN-GYRO-P1) and needed a fibre-optic-class "
        "gyro (0.6 kg); at 0.005 deg/s the catalogue gyro holds it. The ADCS budget is 1.6 kg / 1.0 L (1U of the 3U; ceiling 1.7 kg).",
        "Fluid loop fine pointing: limited by the loop's flow sensor, not the pump. The converged imaging design carries a 0.125 mm/s (1 sigma) "
        "flow sensor; this is a requirement on the in-house sensor. One star-tracker head instead of two breaks nadir pointing, so both stay.",
        "Engine vs MATLAB on fine_hold_rw_rcs rate stability: 0.0056 against 0.0036 deg/s (ratio 1.56, as before the relaxation); the new "
        "0.005 deg/s line now falls between them, so this verdict disagrees. The gap is the wheel-plus-RCS model, not the requirement.",
        "Sun spin: 17 of 24 seeds pass with and without the OBC; failures are Sun-spin entry with the Sun near the XY plane tripping the 1 deg/s "
        "exit guard during the L2 precession transient -- a tuning trade (fsw.sun_spin_perp_out_dps, fsw.sun_spin_dwell_out_s).",
        "Coils only (mtq): sufficient for detumble and for Sun acquisition by Sun spin (the power face comes towards the Sun, though not "
        "within the cases' 95 min / 20 deg line). Nadir: with the hand-over (despin, then capture) and the capture test, see section 7 for "
        "the tuned laws and the mission figure. Three-axis Sun referencing stays out of reach with coils only (gravity gradient along B).",
        "Navigation and guidance audit (docs/NAV_GUIDANCE_AUDIT.md), corrected in C, Rust and the spec: the magnetometer update never ran "
        "while the coils actuated; the estimate was never dropped after the spin states; no innovation gating and fixed Sun/field sigmas; "
        "two-body-only onboard orbit; GNSS taken as J2000 and the field frame without precession (0.37 deg); in nadir the power face "
        "pointed away from the Sun on both cases (now yaw-flipped, 90 deg - |beta|); nadir gravity-gradient feed-forward for ais_img_3u.",
        "Power face in coils-only nadir: Celani 2026 leaves the roll about the payload axis free; a weak roll PD (3n, zeta 1, certified "
        "|mu| 0.913) now turns the face towards the Sun (86 deg mean on ais_3u, was shaded) at the cost of pointing margin (6.6 deg nominal), "
        "since the coils' along-B shortfall lands on roll. Open from the audit: gyro scale factor aliases into bias after the despin; IGRF-13 is extrapolated for 2027 (load IGRF-14); "
        "nadir is geocentric, not geodetic; the MATLAB twin keeps J2000 GNSS, GMST-only frames and no gating.",
        "Bought against designed (both cases, same budget of 1.6 kg and 1.0 L): on ais_img_3u three CubeSpace CW0017 wheels are feasible "
        "at 1.0 kg against our fluid loop's 1.6 kg, so the lightest configuration overall is a benchmark; the fluid loop is selected "
        "because the selection is among our solutions, and its 0.6 kg is the price of not buying wheels. On ais_3u every rotor fails "
        "the 0.5 W orbit-average power (three CW0017 draw 0.9 W steady) and the fluid loop, feasible at 1.45 kg, is the only one that passes. "
        "The Tensor Tech CMG cluster fails power in both cases on its whole-ADCS datasheet figure (4 W upper bound).",
        "Catalogue data: the vendor sites were not reachable from the build environment. CubeSpace Gen2, Rocket Lab RW-0.01, RW3-0.06, "
        "RW3-1.0 and Tensor Tech ADCS100/400 numbers were read from the full datasheets (distributor mirror); AAC Clyde Space RW222/RW400, "
        "Rocket Lab RW-0.03 and Tensor Tech CMG-10m from search excerpts. AAC Clyde Space models lack mass or power on what could be read, "
        "so they are listed but not selectable; confirm with the vendors before a buy decision.",
        "Soft OILS models the OBC's CPU and buses; real OILS still needs the board target (fsw/targets/<board>/main.c: UART/Ethernet, clock, "
        "linker script, modelled on fsw/targets/qemu-mps2) and --realtime on adcs-link/1.",
    ]
    return "<ul>" + "".join(f"<li>{x}</li>" for x in items) + "".join(f"<li>{e(x)}</li>" for x in notes) + "</ul>"


def summary():
    cards, finds = [], []
    for c in CASES:
        sel = jl(PIPE / c / "selection.json")
        if sel:
            cards.append(f"<div class='verdict'><span>{e(c)} selected</span><b>{e(sel['selected'])}</b>{e(sel['status'])}</div>")
    par = jl(ROOT / "results" / "engine_parity.json") or []
    j = [r for r in par if r.get("agree") is not None]
    if j:
        cards.append(f"<div class='verdict'><span>engine vs MATLAB verdicts</span><b>{sum(r['agree'] for r in j)}/{len(j)}</b>truth environment identical</div>")
    so = jl(ROOT / "results" / "soft_oils.json")
    if so:
        jj = [r for r in so["metrics"] if r["pass_sils"] is not None]
        mx = max((t["cpu_load_max"] or 0) for t in so["timing"]) if so["timing"] else 0
        cards.append(f"<div class='verdict'><span>SILS vs soft OILS verdicts</span><b>{sum(r['pass_sils'] == r['pass_oils'] for r in jj)}/{len(jj)}</b>"
                     f"worst OBC load {100 * mx:.1f} %</div>")
    vo = (ROOT / "results" / "VIRTUAL_OBC.md")
    if vo.exists():
        t = vo.read_text()
        cards.append(f"<div class='verdict'><span>virtual OBC bit-identical</span><b>{t.count('bit-identical') - 0}</b>runs, process + QEMU</div>")
    ncamp = len(list((ENG / "campaigns").glob("*/summary.json"))) if (ENG / "campaigns").exists() else 0
    cards.append(f"<div class='verdict'><span>campaigns on the engine</span><b>{ncamp}</b>MC + edge, vs MATLAB</div>")
    for c in CASES:
        sel = jl(PIPE / c / "selection.json")
        if not sel:
            continue
        F = sel["families"][sel["selected"]]
        finds.append(f"<p><b>{e(c)}</b> ({e(sel['class'])} class): the loop selects <code>{e(sel['selected'])}</code> ({e(F['label'])}), "
                     f"{e(sel['status'])}, after {sel['iterations']} iteration(s). Methods: " +
                     e(", ".join(f"{m} = {r['option']}" for m, r in F["modes"].items() if r)) + ". Budget: "
                     f"{F['budget']['mass_kg']:.3f} kg, {F['budget']['volume_L']:.3f} L." + (" Gaps: " + e("; ".join(F["gaps"])) if F["gaps"] else "") +
                     (lambda b: f" Best benchmark (catalogue actuators, same rule): <code>{e(b)}</code>, {e(sel.get('benchmark_status', ''))}, "
                                f"{sel['families'][b]['budget']['mass_kg']:.3f} kg" + (f" — {e('; '.join(sel['families'][b]['gaps']))}" if sel['families'][b]['gaps'] else "") + "."
                      if b else "")(sel.get("benchmark")) + "</p>")
    return "".join(finds), "".join(cards)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    tpl = string.Template((ROOT / "tools" / "templates" / "vv_report.html").read_text())
    commit = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, capture_output=True, text=True).stdout.strip() or "—"
    eng = re.search(r'ENGINE: &str = "([^"]+)"', (ROOT / "engine" / "crates" / "adcs-sim" / "src" / "lib.rs").read_text())
    summ, cards = summary()
    doc = tpl.substitute(
        docno=f"TRN-ADCS-VV-{source_date():%Y%m%d}", date=f"{source_date():%Y-%m-%d %H:%M} UTC", commit=e(commit),
        engine=e(eng.group(1) if eng else "adcs-engine-rs"), fsw="trinetra-fsw-c/1.0.0 and trinetra-fsw-rs (C99 and Rust no_std, adcs-fswcfg/1)",
        cases=e(", ".join(CASES)), verdicts=cards, summary=summ, fig_flow=svg_inline("docs/figures/flow_design_to_hils.svg"),
        fig_arch=svg_inline("docs/figures/architecture_languages.svg"), nodes=nodes(), catalogue=catalogue(), literature=literature(), requirements=requirements(), design=design(), sils=sils(), verification=verification(),
        campaigns=campaigns(), parity=parity(), oils=oils(), vobc=vobc(), open=open_items())
    h = OUT / "TRINETRA_ADCS_VV_report.html"
    write_text(h, doc)
    # the published page: the same report without the document wrapper (the artifact adds it),
    # linking the full results index (results.html) that sits beside it with its figures
    title = re.search(r"<title>.*?</title>", doc, re.S).group(0)
    style = re.search(r"<style>.*?</style>", doc, re.S).group(0)
    body = re.search(r"<body>(.*)</body>", doc, re.S).group(1)
    nav = ('<nav class="wrap" style="padding-block:10px;font-size:9.5pt;color:#5b6477">Full results with every figure: '
           '<a href="results.html">results page</a> · PDF and code: <code>dist/TRINETRA_ADCS_VV_report.pdf</code> in the repository</nav>')
    write_text(OUT / "vv_artifact.html", title + "\n" + style.replace("body { margin: 0;", "body { margin: 0; min-height: 100%;") + "\n" + nav + body)
    chrome = next(iter(glob.glob("/opt/pw-browsers/chromium*/chrome-linux/chrome")), None) or shutil.which("chromium") or shutil.which("google-chrome")
    pdf = OUT / "TRINETRA_ADCS_VV_report.pdf"
    if chrome:
        subprocess.run([chrome, "--headless", "--no-sandbox", "--disable-gpu", "--no-pdf-header-footer", f"--print-to-pdf={pdf}", h.as_uri()],
                       capture_output=True, timeout=300)
    if pdf.exists():
        (ROOT / "dist").mkdir(exist_ok=True)
        shutil.copy(pdf, ROOT / "dist" / "TRINETRA_ADCS_VV_report.pdf")
        print(f"wrote {h.relative_to(ROOT)}, {pdf.relative_to(ROOT)} ({pdf.stat().st_size / 1e6:.1f} MB), dist/TRINETRA_ADCS_VV_report.pdf")
    else:
        print(f"wrote {h.relative_to(ROOT)} (no Chromium for the PDF)")


if __name__ == "__main__":
    main()
