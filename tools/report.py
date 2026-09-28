#!/usr/bin/env python3
"""Build the TRI-NETRA ADCS SILS results report from the filed runs.

Reads, never re-runs:
  matlab_sils/store/results/<scenario>/{manifest.json, channels.csv}
  matlab_sils/store/results/<campaign>/{summary.json, runs.csv, run_*.mat (not needed)}
  matlab_sils/store/trades/<trade>/trade.json
Writes:
  results/figures/<id>_*.png      one figure set per test
  results/index.html               the report page (figures + verdict tables)
  results/summary.json             every metric, every verdict
  docs/RESULTS.md, docs/SELECTION.md

Copyright (c) 2026 Agastya. All rights reserved.
"""
import csv, json, math, pathlib, html, shutil
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = pathlib.Path(__file__).resolve().parents[1]
STORE = ROOT / "matlab_sils" / "store" / "results"
TRADES = ROOT / "matlab_sils" / "store" / "trades"
OUT = ROOT / "results"
FIG = OUT / "figures"

# Reference palette (dataviz skill, light mode), fixed slot order.
S1, S2, S3, S4 = "#2a78d6", "#eb6834", "#1baf7a", "#eda100"
INK, INK2, GRID, SURF = "#0b0b0b", "#52514e", "#e4e3df", "#fcfcfb"
plt.rcParams.update({
    "figure.facecolor": SURF, "axes.facecolor": SURF, "savefig.facecolor": SURF,
    "axes.edgecolor": INK2, "axes.labelcolor": INK, "xtick.color": INK2, "ytick.color": INK2,
    "axes.grid": True, "grid.color": GRID, "grid.linewidth": 0.8, "axes.spines.top": False,
    "axes.spines.right": False, "font.size": 10, "axes.titlesize": 11, "axes.titleweight": "bold",
    "lines.linewidth": 1.6, "legend.frameon": False,
})
MODES = ["detumble", "nadir_mtq", "nadir_fine", "target_fine", "slew_fine", "spinup", "sun_spin"]


def load_run(d):
    man = json.loads((d / "manifest.json").read_text())
    for k in ("metrics", "mode_log"):          # a one-element struct array is written as an object
        if isinstance(man.get(k), dict): man[k] = [man[k]]
    with open(d / "channels.csv") as f:
        r = csv.reader(f)
        hdr = next(r)
        data = np.array([[float(x) for x in row] for row in r])
    ch = {h: data[:, i] for i, h in enumerate(hdr)}
    return man, ch


def req_line(ax, y, label):
    if y is not None and np.isfinite(y):
        ax.axhline(y, color=INK, ls="--", lw=1.4)
        ax.annotate(f"{label} {y:g}", xy=(1, y), xycoords=("axes fraction", "data"),
                    xytext=(-4, 4), textcoords="offset points", ha="right", va="bottom", color=INK, fontsize=9)


def metric(man, kind_prefix):
    for m in man["metrics"]:
        if m["id"].startswith(kind_prefix):
            return m
    return None


def case_req(man, key):
    for m in man["metrics"]:
        if m.get("req_key") == key and m.get("req") is not None:
            return m["req"]
    return None


def save(fig, name):
    p = FIG / f"{name}.png"
    fig.savefig(p, dpi=130, bbox_inches="tight")
    plt.close(fig)
    return p.name


def run_figures(sid, man, ch, full=True):
    t = ch["t_s"] / 60.0
    files = []
    title = f"{sid}  ·  {man['case']}  ·  {man['product']}"
    pointing = np.isfinite(ch["ape_los_deg"]).any()

    # 1 pointing & knowledge
    fig, axs = plt.subplots(2, 1, figsize=(10, 6.2), sharex=True)
    if pointing:
        axs[0].semilogy(t, ch["ape_los_deg"], color=S1, label="payload line of sight")
        axs[0].semilogy(t, ch["ape_3ax_deg"], color=S2, lw=1.1, label="3-axis")
        req_line(axs[0], case_req(man, "req.ape"), "APE req")
        axs[0].set_ylabel("APE [deg]"); axs[0].legend(loc="upper right")
        axs[0].set_title(f"{title}\nabsolute pointing error vs the true target (precision orbit)", loc="left")
        axs[1].semilogy(t, ch["ake_los_deg"], color=S1, label="payload line of sight")
        axs[1].semilogy(t, ch["ake_3ax_deg"], color=S2, lw=1.1, label="3-axis")
        req_line(axs[1], case_req(man, "req.ake"), "AKE req")
        axs[1].set_ylabel("AKE [deg]"); axs[1].legend(loc="upper right")
        axs[1].set_title("absolute knowledge error (MEKF vs truth)", loc="left")
    else:
        axs[0].semilogy(t, ch["rate_degps"], color=S1)
        axs[0].axhline(0.5, color=INK, ls="--", lw=1.4)
        axs[0].annotate("detumble threshold 0.5 deg/s", xy=(1, 0.5), xycoords=("axes fraction", "data"),
                        xytext=(-4, 4), textcoords="offset points", ha="right", color=INK, fontsize=9)
        axs[0].set_ylabel("|ω| [deg/s]"); axs[0].set_title(f"{title}\nbody rate magnitude", loc="left")
        for k, (c, n) in enumerate(zip([S1, S2, S3], "xyz")):
            axs[1].plot(t, ch[f"w_{n}_degps"], color=c, lw=1.0, label=f"ω{n}")
        axs[1].set_ylabel("ω [deg/s]"); axs[1].legend(loc="upper right", ncol=3)
        axs[1].set_title("body rate components", loc="left")
    axs[1].set_xlabel("time [min]")
    files.append(save(fig, f"{sid}_1_attitude"))

    # 2 disturbance torques by source (one unit, one axis)
    fig, ax = plt.subplots(figsize=(10, 3.8))
    for c, n, lab in [(S1, "gg", "gravity gradient"), (S2, "aero", "aerodynamic (facets)"),
                      (S3, "srp", "solar radiation (facets)"), (S4, "mag", "residual dipole")]:
        mag = np.sqrt(sum(ch[f"tau_{n}_{a}_Nm"] ** 2 for a in "xyz"))
        ax.semilogy(t, np.maximum(mag, 1e-14), color=c, lw=1.2, label=lab)
    ax.set_ylabel("|τ| [N m]"); ax.set_xlabel("time [min]"); ax.legend(loc="lower right", ncol=2)
    ax.set_title(f"{title}\ndisturbance torques driven by the in-loop precision orbit", loc="left")
    files.append(save(fig, f"{sid}_2_disturbances"))

    # 3 actuators & power (small multiples: different units)
    has_rw = "h_w1_Nms" in ch and np.isfinite(ch["h_w1_Nms"]).any()
    has_g = "gimbal1_rad" in ch
    has_p = "prop_kg" in ch and np.nanmax(ch["prop_kg"]) > 0
    panels = ["dipole"] + (["h"] if has_rw else []) + (["gimbal"] if has_g else []) + (["prop"] if has_p else []) + ["power"]
    fig, axs = plt.subplots(len(panels), 1, figsize=(10, 2.3 * len(panels) + 0.6), sharex=True)
    for ax_, pn in zip(axs, panels):
        if pn == "dipole":
            for c, a in zip([S1, S2, S3], "xyz"):
                ax_.plot(t, ch[f"m_{a}_Am2"], color=c, lw=0.9, label=f"m{a}")
            ax_.set_ylabel("dipole [A m²]"); ax_.legend(loc="upper right", ncol=3)
            ax_.set_title(f"{title}\nactuators: magnetorquer dipole", loc="left")
        elif pn == "h":
            cols = [S1, S2, S3, S4]
            i = 1
            while f"h_w{i}_Nms" in ch:
                ax_.plot(t, ch[f"h_w{i}_Nms"] * 1e3, color=cols[(i - 1) % 4], label=f"rotor {i}"); i += 1
            ax_.set_ylabel("h [mN m s]"); ax_.legend(loc="upper right", ncol=4)
            ax_.set_title("momentum-exchange devices: stored momentum", loc="left")
        elif pn == "gimbal":
            cols = [S1, S2, S3, S4]; i = 1
            while f"gimbal{i}_rad" in ch:
                ax_.plot(t, np.degrees(ch[f"gimbal{i}_rad"]), color=cols[(i - 1) % 4], label=f"gimbal {i}"); i += 1
            ax_.set_ylabel("gimbal [deg]"); ax_.legend(loc="upper right", ncol=4); ax_.set_title("CMG gimbal angles", loc="left")
        elif pn == "prop":
            ax_.plot(t, ch["prop_kg"] * 1e3, color=S1); ax_.set_ylabel("propellant [g]"); ax_.set_title("cold-gas propellant used", loc="left")
        else:
            tot = ch["P_mtq_W"] + ch["P_rw_W"] + (ch["P_rcs_W"] if "P_rcs_W" in ch else 0)
            ax_.plot(t, tot, color=S1)
            req_line(ax_, case_req(man, "req.pavg"), "orbit-average req")
            ax_.set_ylabel("ADCS power [W]"); ax_.set_title("actuator power (cycle average)", loc="left")
    axs[-1].set_xlabel("time [min]")
    files.append(save(fig, f"{sid}_3_actuators"))

    if "sun_body_z" in ch and (ch["mode"] >= 6).any():
        # Sun-spin chain: -Z_B to Sun angle, spin rate, modes
        lit = ch["shadow_nu"] > 0.5
        ang = np.degrees(np.arccos(np.clip(-ch["sun_body_z"], -1, 1))); ang[~lit] = np.nan
        fig, axs = plt.subplots(3, 1, figsize=(10, 7), sharex=True, gridspec_kw={"height_ratios": [2, 2, 1]})
        axs[0].plot(t, ang, color=S1, lw=1.0); axs[0].axhline(20, color=INK, ls="--", lw=1.2)
        axs[0].set_ylabel("−Z_B to Sun [deg]"); axs[0].set_ylim(0, 180)
        axs[0].set_title(f"{title}\nSun acquisition with coils only: Sun angle (sunlit samples; dashed θ_ok 20°)", loc="left")
        axs[1].plot(t, ch["w_z_degps"], color=S1, label="ω_z (spin)")
        axs[1].plot(t, np.hypot(ch["w_x_degps"], ch["w_y_degps"]), color=S2, lw=1.0, label="|ω_xy| (nutation)")
        axs[1].set_ylabel("rate [deg/s]"); axs[1].legend(loc="upper right")
        axs[2].step(t, ch["mode"], color=S1, where="post"); axs[2].set_yticks([1, 6, 7]); axs[2].set_yticklabels(["detumble", "spinup", "sun_spin"])
        axs[2].set_xlabel("time [min]")
        files.append(save(fig, f"{sid}_7_sunspin"))
    if not full:
        return files
    # 4 environment from the precision orbit
    fig, axs = plt.subplots(3, 1, figsize=(10, 6.5), sharex=True)
    axs[0].semilogy(t, ch["rho_kgm3"], color=S1); axs[0].set_ylabel("ρ [kg/m³]")
    axs[0].set_title(f"{title}\nenvironment seen along the POP orbit (DTM2020 density, DE440 Sun, IGRF)", loc="left")
    axs[1].plot(t, ch["shadow_nu"], color=S1); axs[1].set_ylabel("sunlit fraction ν"); axs[1].set_ylim(-0.05, 1.05)
    axs[2].plot(t, np.sqrt(ch["B_x_T"] ** 2 + ch["B_y_T"] ** 2 + ch["B_z_T"] ** 2) * 1e9, color=S1)
    axs[2].set_ylabel("|B| [nT]"); axs[2].set_xlabel("time [min]")
    files.append(save(fig, f"{sid}_4_environment"))

    # 5 ground track (lat/lon from ECI with GMST)
    r = np.vstack([ch["r_x_m"], ch["r_y_m"], ch["r_z_m"]])
    jd0 = 2451545.0 + 27 * 365.25 if man["epoch_utc"][0] == 2027 else None
    ep = man["epoch_utc"]
    jd = 367 * ep[0] - int(7 * (ep[0] + int((ep[1] + 9) / 12)) / 4) + int(275 * ep[1] / 9) + ep[2] + 1721013.5 + ((ep[5] / 60 + ep[4]) / 60 + ep[3]) / 24
    gm = np.mod(280.46061837 + 360.98564736629 * (jd + ch["t_s"] / 86400 - 2451545), 360)
    lon = np.mod(np.degrees(np.arctan2(r[1], r[0])) - gm + 180, 360) - 180
    lat = np.degrees(np.arcsin(r[2] / np.linalg.norm(r, axis=0)))
    fig, ax = plt.subplots(figsize=(10, 4.6))
    lit = ch["shadow_nu"] > 0.5
    ax.scatter(lon[lit], lat[lit], s=2, color=S1, label="sunlit")
    ax.scatter(lon[~lit], lat[~lit], s=2, color=INK2, label="eclipse")
    ax.set_xlim(-180, 180); ax.set_ylim(-90, 90); ax.set_xlabel("longitude [deg]"); ax.set_ylabel("latitude [deg]")
    o = man["orbit"]
    ax.set_title(f"{title}\nground track — {o['alt_km']:.0f} km SSO, i {o['inc_deg']:.3f}°, LTAN {o['ltan_h']:05.2f} h", loc="left")
    ax.legend(loc="lower left", markerscale=4)
    files.append(save(fig, f"{sid}_5_groundtrack"))

    # 6 mode timeline + rate
    fig, axs = plt.subplots(2, 1, figsize=(10, 4.8), sharex=True, gridspec_kw={"height_ratios": [1, 2]})
    axs[0].step(t, ch["mode"], color=S1, where="post"); axs[0].set_yticks(range(1, len(MODES) + 1)); axs[0].set_yticklabels(MODES)
    axs[0].set_title(f"{title}\nmode timeline and body rate", loc="left")
    axs[1].semilogy(t, ch["rate_degps"], color=S1); axs[1].set_ylabel("|ω| [deg/s]"); axs[1].set_xlabel("time [min]")
    files.append(save(fig, f"{sid}_6_modes"))
    return files


def campaign_figures(cid, summ, rows):
    files = []
    for st in summ["stats"]:
        vals = np.array([r[st["id"]] for r in rows if np.isfinite(r[st["id"]])])
        if vals.size == 0:
            continue
        fig, axs = plt.subplots(1, 2, figsize=(10, 3.6))
        axs[0].hist(vals, bins=max(6, int(math.sqrt(vals.size)) + 2), color=S1, edgecolor=SURF, linewidth=2)
        axs[0].set_xlabel(f"{st['id']} [{st['unit']}]"); axs[0].set_ylabel("runs")
        vs = np.sort(vals)
        axs[1].step(vs, np.arange(1, vs.size + 1) / vs.size, color=S1, where="post")
        axs[1].set_xlabel(st["unit"]); axs[1].set_ylabel("empirical CDF")
        for a in axs:
            if st["req"] is not None and np.isfinite(st["req"]):
                a.axvline(st["req"], color=INK, ls="--", lw=1.4)
            a.axvline(st["pct"], color=S2, lw=1.4)
        axs[0].set_title(f"{cid}: {st['id']}  ({vals.size} runs)", loc="left")
        rq = "" if st["req"] is None or not np.isfinite(st["req"]) else f"  ·  requirement {st['req']:g} (dashed)"
        axs[1].set_title(f"p{st['level']:g} = {st['pct']:.4g} (orange){rq}", loc="left", fontsize=9, fontweight="normal")
        files.append(save(fig, f"{cid}_{st['id']}"))
    # metric vs each dispersion (first bound metric)
    bound = [s for s in summ["stats"] if s["req"] is not None and np.isfinite(s["req"])]
    if bound:
        st = bound[0]
        keys = [k for k in rows[0] if k not in ("run",) and k not in [s["id"] for s in summ["stats"]]]
        n = len(keys)
        if n:
            cols = 3; nr = math.ceil(n / cols)
            fig, axs = plt.subplots(nr, cols, figsize=(10, 2.8 * nr), squeeze=False)
            for i, k in enumerate(keys):
                a = axs[i // cols][i % cols]
                x = np.array([r[k] for r in rows]); y = np.array([r[st["id"]] for r in rows])
                a.scatter(x, y, s=18, color=S1, edgecolor=SURF, linewidth=1)
                a.axhline(st["req"], color=INK, ls="--", lw=1.2)
                a.set_xlabel(k, fontsize=9); a.set_ylabel(st["unit"], fontsize=9)
            for j in range(n, nr * cols):
                axs[j // cols][j % cols].axis("off")
            fig.suptitle(f"{cid}: {st['id']} against every dispersed parameter", x=0.01, ha="left", fontweight="bold")
            fig.tight_layout()
            files.append(save(fig, f"{cid}_scatter"))
    return files


def verdict(p):
    if p is None or (isinstance(p, float) and not np.isfinite(p)):
        return "—", ""
    return ("✔ PASS", "pass") if p else ("✖ FAIL", "fail")


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


def mval(man, mid):
    for m in man["metrics"]:
        if m["id"] == mid:
            return m["value"]
    return float("nan")


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
        fig, axs = plt.subplots(1, 3, figsize=(11, 3.8))
        names = [r[0] for r in rows]; y = np.arange(len(rows))
        for ax_, col, lab, req, logx in [(axs[0], 1, "fine-hold APE, p99.73 [deg]", 0.01, True),
                                          (axs[1], 2, "30° slew settling [s]", 20, False),
                                          (axs[2], 3, "after 90°/15 s slew, APE p99.73 [deg]", 0.01, True)]:
            v = np.array([r[col] for r in rows], dtype=float)
            vv = np.where(np.isfinite(v), v, 0)
            ax_.barh(y, vv, color=S1, height=0.6)
            for yi, val in zip(y, v):
                ax_.text(vv[yi] if np.isfinite(val) else 0, yi, f" {val:.3g}" if np.isfinite(val) else " not settled", va="center", fontsize=8, color=INK)
            ax_.axvline(req, color=INK, ls="--", lw=1.2)
            if logx and np.nanmax(vv) > 0: ax_.set_xscale("log")
            ax_.set_yticks(y); ax_.set_yticklabels(names if ax_ is axs[0] else []); ax_.set_xlabel(lab); ax_.grid(axis="y", visible=False)
            ax_.invert_yaxis()
        fig.suptitle("Imaging 3U: actuator families on the same case (dashed = requirement)", x=0.01, ha="left", fontweight="bold")
        fig.tight_layout(); files.append(save(fig, "compare_families"))
    return files


TRADE_ORDER = ["trade_detumble_ais", "trade_mtq_pointing_ais", "trade_sun_spin_ais", "trade_sun_sensor_ais",
               "trade_pointing_img", "trade_slew_img", "trade_pointing_cmg", "trade_pointing_vscmg", "trade_pointing_fmr",
               "trade_allocation_fmr", "trade_hardware_fine", "trade_hardware_agile"]


def fnum(x):
    return float("nan") if x is None else float(x)


def seeds_of(c):
    v = c.get("obj_seeds")
    if v is None: return []
    if not isinstance(v, list): v = [v]
    return [fnum(x) for x in v]


def load_trades():
    out = []
    ids = [t for t in TRADE_ORDER if (TRADES / t / "trade.json").exists()]
    ids += sorted(d.name for d in TRADES.glob("*") if (d / "trade.json").exists() and d.name not in ids) if TRADES.exists() else []
    for tid in ids:
        T = json.loads((TRADES / tid / "trade.json").read_text())
        c = T["candidates"]
        T["candidates"] = c if isinstance(c, list) else [c]
        out.append(T)
    return out


def trade_figure(T):
    C = T["candidates"]; n = len(C)
    unit = ""
    m0 = C[0].get("metrics") or {}
    if isinstance(m0, dict) and T["objective"]["metric"] in m0:
        unit = m0[T["objective"]["metric"]].get("unit", "")
    fig, ax = plt.subplots(figsize=(10, 0.55 * n + 1.6))
    y = np.arange(n)
    for i, c in enumerate(C):
        col = S3 if (i == 0 and T.get("selected")) else (S1 if c["feasible"] else "#b9bec4")
        w = fnum(c["obj"])
        if np.isfinite(w):
            ax.barh(i, w, color=col, height=0.55)
        for v in seeds_of(c):
            if np.isfinite(v): ax.plot(v, i, "o", color=INK, ms=4)
        lab = f" {w:.3g}" if np.isfinite(w) else " no value"
        ax.text(w if np.isfinite(w) else 0, i, lab + ("" if c["feasible"] else "  (fails a requirement)"), va="center", fontsize=8, color=INK2)
    ax.set_yticks(y); ax.set_yticklabels([c["id"] for c in C]); ax.invert_yaxis(); ax.grid(axis="y", visible=False)
    vals = [fnum(c["obj"]) for c in C if np.isfinite(fnum(c["obj"])) and fnum(c["obj"]) > 0]
    if vals and max(vals) / max(min(vals), 1e-12) > 50: ax.set_xscale("log")
    ax.set_xlabel(f"{T['objective']['metric']} [{unit}] — bar: worst over seeds, dots: each seed")
    ax.set_title(f"{T['label']}\ngreen: proposed · blue: meets every requirement · grey: fails one", loc="left")
    return save(fig, T["id"])


SOLS = ROOT / "matlab_sils" / "store" / "solutions"
SIZED = ROOT / "matlab_sils" / "store" / "sized"
OURS = {"mtq", "fmr", "rcs"}
MODE_ORDER = ["detumble", "sun_acquisition", "sun_referencing", "nadir_pointing"]


def aslist(x):
    return x if isinstance(x, list) else ([] if x is None else [x])


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
    fig, ax = plt.subplots(figsize=(10, 0.42 * len(O) + 1.4))
    for i, o in enumerate(O):
        ours = set(aslist(o["uses"])) <= OURS
        v = fnum(o["obj"])
        col = (S1 if ours else S2) if o["feasible"] else "#b9bec4"
        if np.isfinite(v): ax.barh(i, v, color=col, height=0.6)
        lab = f" {v:.3g}" if np.isfinite(v) else " no result"
        ax.text(v if np.isfinite(v) else 0, i, lab + ("" if o["feasible"] else "  fails"), va="center", fontsize=8, color=INK2)
    ax.set_yticks(range(len(O))); ax.set_yticklabels([o["id"] for o in O]); ax.invert_yaxis(); ax.grid(axis="y", visible=False)
    if req is not None: ax.axvline(req, color=INK, ls="--", lw=1.2)
    vals = [fnum(o["obj"]) for o in O if np.isfinite(fnum(o["obj"])) and fnum(o["obj"]) > 0]
    if vals and max(vals) / max(min(vals), 1e-12) > 50: ax.set_xscale("log")
    ax.set_xlabel(f"{ob} [{unit}], worst over seeds" + (" — dashed: requirement" if req is not None else ""))
    ax.set_title(f"{S['case']} · {M['label']}\nblue: our actuators (MTQ, fluid loop, RCS) · orange: benchmark · grey: fails a requirement", loc="left")
    return save(fig, f"solution_{S['case']}_{mid}")


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
    fig, axs = plt.subplots(2, 3, figsize=(11, 6.2))
    names = [r["id"] for r in rows]; y = np.arange(len(rows))
    for ax, (k, lab) in zip(axs.flat, params):
        v = np.array([fnum(r.get(k)) for r in rows])
        cols = [S1 if r["role"] == "solution" else S2 for r in rows]
        ax.barh(y, np.nan_to_num(v), color=cols, height=0.6)
        ax.set_yticks(y); ax.set_yticklabels(names if ax in axs[:, 0] else [], fontsize=8); ax.invert_yaxis()
        ax.set_title(lab, loc="left", fontsize=10); ax.grid(axis="y", visible=False)
        pos = v[np.isfinite(v) & (v > 0)]
        if pos.size and pos.max() / max(pos.min(), 1e-12) > 50: ax.set_xscale("log")
    fig.suptitle(f"{S['case']}: our solutions (blue) against sized benchmarks (orange)", x=0.01, ha="left", fontweight="bold")
    fig.tight_layout()
    return save(fig, f"solution_{S['case']}_compare")


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
    (ROOT / "docs" / "SOLUTIONS.md").write_text("\n".join(L) + "\n")


def main():
    FIG.mkdir(parents=True, exist_ok=True)
    report = {"owner": "Agastya", "scenarios": {}, "campaigns": {}}
    runs = {}
    for _, ids in GROUPS:
        for sid in ids:
            d = STORE / sid
            if (d / "manifest.json").exists():
                runs[sid] = load_run(d)
    cmp_figs = comparison_figures(runs)
    sections = []
    for gname, ids in GROUPS:
        items = []
        for sid in ids:
            if sid not in runs: continue
            man, ch = runs[sid]
            figs = run_figures(sid, man, ch, full=sid in PRIMARY)
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
    (OUT / "summary.json").write_text(json.dumps(report, indent=1, default=float))
    write_html(sections, cmp_figs, report, trades, sol_html)
    write_md(sections, cmp_figs)
    write_selection(trades)
    print(f"report: {sum(len(i) for _, i in sections)} items, {len(list(FIG.glob('*.png')))} figures")


def trade_html(T):
    o = [f"<h3 id='{T['id']}'>{html.escape(T['label'])}</h3>",
         f"<p class='muted'>{html.escape(T.get('question', ''))} · {T['kind']} trade"
         + (f" for slot <code>{T['slot']}</code>" if T.get('slot') else "")
         + (f" on <b>{T['promote_to']}</b>" if T.get('promote_to') else "")
         + f" · seeds {T['seeds'] if isinstance(T['seeds'], list) else [T['seeds']]} · ranked by the worst <code>{T['objective']['metric']}</code> over the seeds"
         + (f", ties by <code>{T['tie_break']['metric']}</code>" if T.get('tie_break', {}).get('metric') else "") + "</p>"]
    o.append("<div class='scroll'><table><tr><th>rank</th><th>candidate</th><th>product / algorithms</th><th>objective worst</th><th>mean</th><th>requirement checks passed</th><th>tie-break</th></tr>")
    for i, c in enumerate(T["candidates"]):
        al = c.get("algorithms") or {}
        alt = ", ".join(f"{k}={v}" for k, v in al.items() if v) if isinstance(al, dict) else ""
        pr = fnum(c.get("pass_rate")); cls = "pass" if c["feasible"] else "fail"
        o.append(f"<tr><td>{i+1}</td><td><b>{html.escape(c['id'])}</b><br><span class='muted'>{html.escape(c.get('label',''))}</span></td>"
                 f"<td class='mono'>{html.escape(c.get('product',''))}<br><span class='muted'>{html.escape(alt)}</span></td>"
                 f"<td>{fnum(c['obj']):.4g}</td><td>{fnum(c['obj_mean']):.4g}</td><td class='{cls}'>{pr:.0f}%</td><td>{fnum(c.get('tie')):.3g}</td></tr>")
    o.append("</table></div>")
    sel = T.get("selected") or "none"
    o.append(f"<p><b>Proposed: {html.escape(sel)}</b> — {html.escape(T.get('rationale',''))} <span class='muted'>({html.escape(T.get('status',''))})</span></p>")
    o.append(f"<figure><img src='figures/{T['figure']}' alt='{html.escape(T['id'])}' loading='lazy'></figure>")
    return "\n".join(o)


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
    (OUT / "index.html").write_text("\n".join(out))


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
    (ROOT / "docs" / "RESULTS.md").write_text("\n".join(L) + "\n")


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
    (ROOT / "docs" / "SELECTION.md").write_text("\n".join(L) + "\n")


if __name__ == "__main__":
    main()
