"""Figures of each scenario run and campaign, and the engine-versus-twin comparison figures.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import math
import numpy as np
import matplotlib.pyplot as plt
from report_base import INK, INK2, MODES, S1, S2, S3, S4, SURF, case_req, mval, req_line, save


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
