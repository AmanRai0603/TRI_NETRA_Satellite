"""The customer-case solution matrix: every family and option, compared, and its documents.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import json, html
import numpy as np
from common import write_text
from report_base import MUTED, ROOT, S1, S2, aslist, fnum, save


SOLS = ROOT / "matlab_sils" / "store" / "solutions"


SIZED = ROOT / "matlab_sils" / "store" / "sized"


OURS = {"mtq", "fmr", "rcs"}


MODE_ORDER = ["detumble", "sun_acquisition", "sun_referencing", "nadir_pointing"]


def load_solutions():
    out = []
    for c in ["ais_3u", "ais_img_3u"]:
        f = SOLS / c / "solution.json"
        if not f.exists():
            continue
        S = json.loads(f.read_text())
        S["sizing"] = json.loads((SIZED / c / "sizing.json").read_text())
        for m in S["modes"].values():
            m["options"] = aslist(m["options"])
        out.append(S)
    return out


def option_figure(S, mid):
    M = S["modes"][mid]; O = M["options"]
    ob = M["objective"]; unit = ""; req = None
    for o in O:
        mm = o.get("metrics") or {}
        if ob in mm:
            unit = mm[ob].get("unit", ""); r = mm[ob].get("req")
            if r is not None and np.isfinite(fnum(r)): req = fnum(r)
            break
    v = [fnum(o["obj"]) for o in O]
    vals = [x for x in v if np.isfinite(x) and x > 0]
    p = {"title": f"{S['case']} · {M['label']}\nblue: our actuators (MTQ, fluid loop, RCS) · orange: benchmark · grey: fails a requirement",
         "xlabel": f"{ob} [{unit}], worst over seeds" + (" — dashed: requirement" if req is not None else ""),
         "xscale": "log" if vals and max(vals) / max(min(vals), 1e-12) > 50 else "linear",
         "refs": [{"axis": "x", "at": req, "width": 1.2}] if req is not None else [],
         "series": [{"kind": "hbar", "labels": [o["id"] for o in O], "values": v,
                     "colors": [((S1 if set(aslist(o["uses"])) <= OURS else S2) if o["feasible"] else MUTED) for o in O],
                     "texts": [(f"{x:.3g}" if np.isfinite(x) else "no result") + ("" if o["feasible"] else "  fails") for x, o in zip(v, O)]}]}
    return save({"width": 720, "height": 30 * len(O) + 101, "panels": [p]}, f"solution_{S['case']}_{mid}")


def family_rows(S):
    """Per family: sizing budget + the measured figures of its chosen method per mode."""
    rows = []
    rcs = S["sizing"]["parts"].get("rcs", {}).get("nominal", {})
    for fid, E in S["families"].items():
        r = {"id": fid, "role": E["role"], "label": E["label"], "passes": E["passes"], "pass": E["pass"],
             "mass": fnum(E["mass_kg"]), "pnom": fnum(E["power_nominal_W"]), "vol": fnum(E["volume_L"]),
             "prop": fnum(rcs.get("propellant_kg")) if "rcs" in fid else 0.0, "gaps": aslist(E.get("gaps"))}
        for mid in MODE_ORDER:
            me = E["methods"].get(mid, {})
            r[mid] = me.get("option", ""); r[mid + "_obj"] = fnum(me.get("obj")); r[mid + "_ok"] = bool(me.get("feasible"))
            if mid == "nadir_pointing":
                for o in S["modes"][mid]["options"]:
                    if o["id"] == me.get("option"):
                        mm = o.get("metrics") or {}
                        for k in ("ake_los_p9973", "rate_stability_p9973", "jitter", "power_mean"):
                            r[k] = fnum(mm.get(k, {}).get("worst")) if k != "power_mean" else fnum(mm.get(k, {}).get("mean"))
        rows.append(r)
    return rows


def compare_figure(S, rows):
    """Our three solutions vs the benchmarks on the parameters a customer weighs."""
    params = [("mass", "ADCS mass [kg]"), ("power_mean", "power in nadir pointing [W]"),
              ("nadir_pointing_obj", "nadir APE p99.73 [deg]"), ("sun_acquisition_obj", "Sun acquisition [min]"),
              ("detumble_obj", "detumble [min]"), ("jitter", "jitter [arcsec]")]
    names = [r["id"] for r in rows]
    cols = [S1 if r["role"] == "solution" else S2 for r in rows]
    panels = []
    for i, (k, lab) in enumerate(params):
        v = [fnum(r.get(k)) for r in rows]
        pos = [x for x in v if np.isfinite(x) and x > 0]
        panels.append({"title": lab, "xscale": "log" if pos and max(pos) / max(min(pos), 1e-12) > 50 else "linear",
                       "series": [{"kind": "hbar", "labels": names if i % 3 == 0 else [], "values": v, "colors": cols,
                                   "texts": [f"{x:.3g}" if np.isfinite(x) else "—" for x in v]}]})
    return save({"title": f"{S['case']}: our solutions (blue) against sized benchmarks (orange)", "layout": "grid", "rows": 2, "cols": 3,
                 "width": 792, "height": 446, "panels": panels}, f"solution_{S['case']}_compare")


def solution_html(S, rows, figs):
    D = S["sizing"]["demand"]
    o = [f"<h3 id='sol_{S['case']}'>Case {S['case']} ({S['class']} class)</h3>"]
    rec = S["recommended"]; E = S["families"][rec]
    o.append(f"<p><b>Recommended: {html.escape(E['label'])}</b> (<code>{rec}</code>). {html.escape(S['verdict'])}</p>")
    o.append(f"<p class='muted'>Demand from the case: peak disturbance {fnum(D['tau_dist']):.2e} N m ({D['worst_attitude']}); "
             f"stored momentum {fnum(D['h_dist'])*1e3:.2f} mN m s per orbit; slew {D['slew_deg']:g}° in {D['slew_s']:g} s; "
             f"detumble {fnum(D['h_detumble'])*1e3:.1f} mN m s; weakest field {fnum(D['B_min'])*1e6:.1f} µT. "
             f"Required, with margins ×{fnum(D['k_h']):g} momentum and ×{fnum(D['k_tau']):g} torque: "
             f"{fnum(D['h_req'])*1e3:.2f} mN m s, {fnum(D['tau_req'])*1e6:.0f} µN m.</p>")
    o.append("<div class='scroll'><table><tr><th>family</th><th>role</th><th>modes passed</th>"
             + "".join(f"<th>{m.replace('_', ' ')}</th>" for m in MODE_ORDER)
             + "<th>mass kg</th><th>power (nadir) W</th><th>vol L</th><th>N2O kg / life</th><th>AKE °</th><th>RKS °/s</th><th>jitter ″</th></tr>")
    for r in rows:
        cls = "pass" if r["pass"] else "fail"
        role = "<b>ours</b>" if r["role"] == "solution" else "benchmark"
        cells = "".join(f"<td><span class='{'pass' if r[m + '_ok'] else 'fail'}'>{html.escape(r[m] or '—')}</span><br>{r[m + '_obj']:.3g}</td>" for m in MODE_ORDER)
        o.append(f"<tr><td><b>{r['id']}</b></td><td>{role}</td><td class='{cls}'>{r['passes']}/4</td>{cells}"
                 f"<td>{r['mass']:.3f}</td><td>{fnum(r.get('power_mean')):.2f}</td><td>{r['vol']:.3f}</td><td>{r['prop']:.3f}</td>"
                 f"<td>{fnum(r.get('ake_los_p9973')):.3g}</td><td>{fnum(r.get('rate_stability_p9973')):.3g}</td><td>{fnum(r.get('jitter')):.2g}</td></tr>")
    o.append("</table></div>")
    o.append("<p class='muted'>Each mode cell: the method the family flies (the best option its actuators allow) and its objective, worst over the seeds: detumble and Sun acquisition in minutes, Sun referencing and nadir in degrees p99.73. Mass, volume and N2O come from the sizing. Power is the measured mean in nadir pointing.</p>")
    for f in figs:
        o.append(f"<figure><img src='figures/{f}' alt='{f[:-4]}' loading='lazy'></figure>")
    return "\n".join(o)


def components_html():
    import tomllib
    C = [tomllib.loads(f.read_text()) for f in sorted((ROOT / "catalogue" / "components").glob("*.toml"))]
    C.sort(key=lambda c: (c["kind"] != "sensor", c["id"]))
    o = ["<h2 id='components'>Components: SILS model and in-house chain</h2>",
         "<p>The SILS flies a <b>model</b> of each unit. The unit's own processing, its <b>chain</b>, is where in-house development plugs in node by node, and it is packaged for the unit or the OBC. The HAL frame is the only interface either way (docs/COMPONENTS.md).</p>",
         "<div class='scroll'><table><tr><th>component</th><th>kind</th><th>in-house</th><th>chain (status)</th><th>HAL output</th></tr>"]
    for c in C:
        ch = ", ".join(f"{n['node']} <span class='muted'>({n['status']})</span>" for n in c.get("chain", [])) or "—"
        o.append(f"<tr><td><b>{c['id']}</b></td><td>{c['kind']}</td><td>{c['in_house']}</td><td>{ch}</td><td class='muted'>{html.escape(c['hal_output'])}</td></tr>")
    o.append("</table></div>")
    return "\n".join(o)


def write_solutions_md(sols):
    L = ["# Solutions per customer case", "", "**Owner: Agastya.** Copyright (c) 2026 Agastya. All rights reserved.", "",
         "Generated by `tools/report.py` from `matlab_sils/store/solutions/<case>/solution.json`.",
         "See `docs/SOLUTION_PIPELINE.md` for how a case becomes a solution.", ""]
    for S in sols:
        rows = family_rows(S)
        L += [f"## {S['case']} ({S['class']} class)", "", f"**Recommended: `{S['recommended']}`.** {S['verdict']}", "",
              "| family | role | modes | " + " | ".join(m.replace('_', ' ') for m in MODE_ORDER) + " | mass kg | power (nadir) W | N2O kg/life |",
              "|---|---|---|" + "---|" * len(MODE_ORDER) + "---|---|---|"]
        for r in rows:
            cells = " | ".join(f"{r[m] or '—'} ({r[m + '_obj']:.3g}){'' if r[m + '_ok'] else ' ✖'}" for m in MODE_ORDER)
            L.append(f"| {r['id']} | {'ours' if r['role'] == 'solution' else 'benchmark'} | {r['passes']}/4 | {cells} | {r['mass']:.3f} | {fnum(r.get('power_mean')):.2f} | {r['prop']:.3f} |")
        for mid in MODE_ORDER:
            M = S["modes"][mid]
            L += ["", f"### {M['label']} — objective `{M['objective']}` (worst over seeds)", "", "| option | actuators | feasible | objective | power W | fails |", "|---|---|---|---|---|---|"]
            for o in M["options"]:
                L.append(f"| {o['id']} | {', '.join(aslist(o['uses']))} | {'yes' if o['feasible'] else 'no'} | {fnum(o['obj']):.4g} | {fnum(o['power']):.2f} | {', '.join(aslist(o.get('failed'))) or '—'} |")
        L.append("")
    write_text(ROOT / "docs" / "SOLUTIONS.md", "\n".join(L) + "\n")
