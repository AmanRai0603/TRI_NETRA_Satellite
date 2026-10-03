"""Figures of each scenario run and campaign, and the engine-versus-twin comparison figures.
A run's figures are `adcs figures` on its folder (adcs-plot draws engine and twin runs alike);
the campaign and comparison figures are described here as JSON and drawn by `adcs plot`.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import math
import numpy as np
from report_base import S1, S2, mval, run_svgs, save


def run_figures(sid, man, run_dir, full=True):
    """The run's figures (1 attitude, 2 disturbances, 3 actuators, 7 Sun spin when it spun up; with
    `full` also 4 environment, 5 ground track, 6 modes) as results/figures/<sid>_<n>_<name>.svg."""
    return run_svgs(run_dir, sid, full)


def _req_x(req):
    return [{"axis": "x", "at": req, "label": f"req {req:g}"}] if req is not None and np.isfinite(req) else []


def campaign_figures(cid, summ, rows):
    files = []
    for st in summ["stats"]:
        vals = np.array([r[st["id"]] for r in rows if np.isfinite(r[st["id"]])])
        if vals.size == 0:
            continue
        vs = np.sort(vals)
        refs = _req_x(st["req"]) + [{"axis": "x", "at": st["pct"], "color": S2, "dashed": False, "width": 1.4, "label": f"p{st['level']:g}"}]
        rq = "" if st["req"] is None or not np.isfinite(st["req"]) else f"  ·  requirement {st['req']:g} (dashed)"
        files.append(save({"layout": "grid", "rows": 1, "cols": 2, "width": 720, "height": 259, "panels": [
            {"title": f"{cid}: {st['id']}  ({vals.size} runs)", "xlabel": f"{st['id']} [{st['unit']}]", "ylabel": "runs", "refs": refs,
             "series": [{"kind": "hist", "values": vals, "bins": max(6, int(math.sqrt(vals.size)) + 2), "color": S1}]},
            {"title": f"p{st['level']:g} = {st['pct']:.4g} (orange){rq}", "xlabel": st["unit"], "ylabel": "empirical CDF", "refs": refs,
             "series": [{"kind": "step", "x": vs, "y": np.arange(1, vs.size + 1) / vs.size, "color": S1}]}]}, f"{cid}_{st['id']}"))
    # metric vs each dispersion (first bound metric)
    bound = [s for s in summ["stats"] if s["req"] is not None and np.isfinite(s["req"])]
    if bound:
        st = bound[0]
        keys = [k for k in rows[0] if k not in ("run",) and k not in [s["id"] for s in summ["stats"]]]
        if keys:
            cols = 3; nr = math.ceil(len(keys) / cols)
            y = [r[st["id"]] for r in rows]
            panels = [{"xlabel": k, "ylabel": st["unit"], "refs": [{"axis": "y", "at": st["req"], "width": 1.2}],
                       "series": [{"kind": "scatter", "x": [r[k] for r in rows], "y": y, "color": S1, "size": 2.4}]} for k in keys]
            files.append(save({"title": f"{cid}: {st['id']} against every dispersed parameter", "layout": "grid", "rows": nr, "cols": cols,
                               "width": 720, "height": 202 * nr + 24, "panels": panels}, f"{cid}_scatter"))
    return files


GROUPS = [
    ("AIS 3U, magnetorquers only (10°, SSO dawn–dusk)", ["detumble_ais", "nadir_hold_ais", "nadir_hold_ais_css", "sun_spin_ais", "mission_ais", "fault_coil_ais", "fault_gyro_ais"]),
    ("Imaging 3U, magnetorquers + reaction wheels (0.01°, SSO 10:00)", ["detumble_img", "fine_hold_img", "slew_img", "agile_slew_img", "mission_img", "fault_wheel_img", "fault_st_img"]),
    ("Imaging 3U, magnetorquers + fluid momentum rings (IDMAS)", ["fine_hold_fmr", "slew_fmr", "mission_fmr"]),
    ("Imaging 3U, magnetorquers + fluid rings + cold-gas RCS", ["fine_hold_fmr_rcs", "slew_fmr_rcs", "agile_slew_fmr_rcs", "mission_fmr_rcs"]),
    ("Imaging 3U, magnetorquers + reaction wheels + cold-gas RCS", ["fine_hold_rw_rcs", "slew_rw_rcs", "agile_slew_rw_rcs", "mission_rw_rcs"]),
    ("Imaging 3U, magnetorquers + 4 SGCMG", ["fine_hold_cmg", "slew_cmg", "agile_slew_cmg", "mission_cmg", "fault_gimbal_cmg"]),
    ("Imaging 3U, magnetorquers + 4 VSCMG", ["fine_hold_vscmg", "slew_vscmg", "agile_slew_vscmg", "mission_vscmg"]),
]


PRIMARY = {"detumble_ais", "sun_spin_ais", "nadir_hold_ais", "mission_ais", "detumble_img", "fine_hold_img", "slew_img", "mission_img",
           "mission_fmr", "mission_fmr_rcs", "mission_rw_rcs", "mission_cmg", "mission_vscmg"}


CAMPAIGNS = ["mc_detumble_ais", "mc_nadir_ais", "edge_nadir_ais", "mc_fine_img", "edge_fine_img", "mc_slew_img", "mc_slew_cmg", "mc_agile_rw_rcs"]


def hbar_panel(labels, v, xlabel, req=None, logx=False, nan_text="no value", color=S1, colors=None, title=""):
    """One horizontal-bar panel: a bar per category (the first at the top), its value after it,
    the requirement dashed, a log x axis when asked and something is positive."""
    v = [float("nan") if x is None else float(x) for x in v]
    p = {"title": title, "xlabel": xlabel, "series": [{"kind": "hbar", "labels": labels, "values": v, "color": color,
         "texts": [f"{x:.3g}" if np.isfinite(x) else nan_text for x in v], **({"colors": colors} if colors else {})}]}
    if req is not None:
        p["refs"] = [{"axis": "x", "at": req, "width": 1.2}]
    if logx and any(np.isfinite(x) and x > 0 for x in v):
        p["xscale"] = "log"
    return p


def comparison_figures(runs):
    files = []
    fams = [("RW", "img"), ("fluid rings", "fmr"), ("rings + RCS", "fmr_rcs"), ("RW + RCS", "rw_rcs"), ("SGCMG", "cmg"), ("VSCMG", "vscmg")]
    rows = []
    for name, k in fams:
        fh, sl, ag = runs.get(f"fine_hold_{k}"), runs.get(f"slew_{k}"), runs.get(f"agile_slew_{k}")
        rows.append((name,
                     mval(fh[0], "ape_los_p9973") if fh else np.nan,
                     mval(sl[0], "settle_time_after_slew") if sl else np.nan,
                     mval(ag[0], "ape_los_on_target_p9973") if ag else np.nan,
                     mval(fh[0], "power_mean") if fh else np.nan))
    if any(np.isfinite(r[1]) for r in rows):
        names = [r[0] for r in rows]
        panels = [hbar_panel(names if i == 0 else [], [r[col] for r in rows], lab, req, logx, nan_text="not settled")
                  for i, (col, lab, req, logx) in enumerate([(1, "fine-hold APE, p99.73 [deg]", 0.01, True),
                                                             (2, "30° slew settling [s]", 20, False),
                                                             (3, "after 90°/15 s slew, APE p99.73 [deg]", 0.01, True)])]
        files.append(save({"title": "Imaging 3U: actuator families on the same case (dashed = requirement)", "layout": "grid", "rows": 1, "cols": 3,
                           "width": 792, "height": 274, "panels": panels}, "compare_families"))
    return files

