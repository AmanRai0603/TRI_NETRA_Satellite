#!/usr/bin/env python3
"""Build the TRI-NETRA ADCS SILS results report from the filed runs.

Reads, never re-runs:
  matlab_sils/store/results/<scenario>/{manifest.json, channels.csv}
  matlab_sils/store/results/<campaign>/{summary.json, runs.csv, run_*.mat (not needed)}
Writes:
  results/figures/<id>_*.png      one figure set per test
  results/index.html               the report page (figures + verdict tables)
  results/summary.json             every metric, every verdict

Copyright (c) 2026 Agastya. All rights reserved.
"""
import csv, json, math, pathlib, html, shutil
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = pathlib.Path(__file__).resolve().parents[1]
STORE = ROOT / "matlab_sils" / "store" / "results"
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
MODES = ["detumble", "nadir_mtq", "nadir_rw", "target_rw", "slew_rw"]


def load_run(d):
    man = json.loads((d / "manifest.json").read_text())
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


def run_figures(sid, man, ch):
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
    fig, axs = plt.subplots(3 if has_rw else 2, 1, figsize=(10, 7 if has_rw else 5), sharex=True)
    for c, a in zip([S1, S2, S3], "xyz"):
        axs[0].plot(t, ch[f"m_{a}_Am2"], color=c, lw=0.9, label=f"m{a}")
    axs[0].set_ylabel("dipole [A m²]"); axs[0].legend(loc="upper right", ncol=3)
    axs[0].set_title(f"{title}\nmagnetorquer dipole", loc="left")
    k = 1
    if has_rw:
        for c, i in zip([S1, S2, S3], [1, 2, 3]):
            axs[1].plot(t, ch[f"h_w{i}_Nms"] * 1e3, color=c, label=f"wheel {i}")
        axs[1].set_ylabel("h [mN m s]"); axs[1].legend(loc="upper right", ncol=3); axs[1].set_title("wheel momentum", loc="left")
        k = 2
    axs[k].plot(t, ch["P_mtq_W"] + ch["P_rw_W"], color=S1)
    req_line(axs[k], case_req(man, "req.pavg"), "orbit-average req")
    axs[k].set_ylabel("ADCS power [W]"); axs[k].set_xlabel("time [min]"); axs[k].set_title("actuator power (cycle average)", loc="left")
    files.append(save(fig, f"{sid}_3_actuators"))

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
    axs[0].step(t, ch["mode"], color=S1, where="post"); axs[0].set_yticks(range(1, 6)); axs[0].set_yticklabels(MODES)
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


def main():
    FIG.mkdir(parents=True, exist_ok=True)
    report = {"owner": "Agastya", "scenarios": {}, "campaigns": {}}
    sections = []
    order = ["detumble_ais", "nadir_hold_ais", "mission_ais", "detumble_img", "fine_hold_img", "slew_img", "mission_img"]
    for sid in order:
        d = STORE / sid
        if not (d / "manifest.json").exists():
            continue
        man, ch = load_run(d)
        figs = run_figures(sid, man, ch)
        report["scenarios"][sid] = {"case": man["case"], "product": man["product"], "metrics": man["metrics"],
                                    "wall_s": man["wall_s"], "duration_s": man["duration_s"], "figures": figs}
        sections.append(("run", sid, man, figs))
    for cid in ["mc_detumble_ais", "mc_nadir_ais", "mc_fine_img", "mc_slew_img"]:
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
        sections.append(("mc", cid, summ, figs))
    (OUT / "summary.json").write_text(json.dumps(report, indent=1, default=float))
    write_html(sections)
    print(f"report: {len(sections)} sections, {sum(len(s[3]) for s in sections)} figures")


def write_html(sections):
    css = """
:root{--bg:#f7f8f9;--surface:#ffffff;--text:#15191e;--muted:#56606b;--line:#dde2e7;--accent:#2a78d6;--pass:#0a7a3a;--fail:#b3261e;--chip:#eef2f6}
@media (prefers-color-scheme: dark){:root:not([data-theme="light"]){color-scheme:dark;--bg:#14171a;--surface:#1b1f23;--text:#eef1f4;--muted:#a3adb8;--line:#2f353c;--accent:#5a9ce8;--pass:#5bd08a;--fail:#ff8a80;--chip:#252b31}}
:root[data-theme="dark"]{color-scheme:dark;--bg:#14171a;--surface:#1b1f23;--text:#eef1f4;--muted:#a3adb8;--line:#2f353c;--accent:#5a9ce8;--pass:#5bd08a;--fail:#ff8a80;--chip:#252b31}
body{background:var(--bg);color:var(--text);font:15px/1.55 "IBM Plex Sans",system-ui,-apple-system,"Segoe UI",sans-serif}
main{max-width:1080px;margin:0 auto;padding-inline:16px;padding-block:28px 64px}
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
    out.append("<nav>" + "".join(f"<a href='#{html.escape(s[1])}'>{html.escape(s[1])}</a>" for s in sections) + "</nav>")
    for kind, sid, obj, figs in sections:
        out.append(f"<h2 id='{sid}'>{sid}</h2>")
        if kind == "run":
            man = obj
            out.append(f"<p class='muted'>case <b>{html.escape(man['case'])}</b> — {html.escape(man['case_title'])} · product <b>{man['product']}</b> · {man['duration_s']/60:.0f} min simulated in {man['wall_s']/60:.1f} min wall · seed {man['seed']}</p>")
            out.append("<div class='scroll'><table><tr><th>metric</th><th>value</th><th>required</th><th>verdict</th></tr>")
            for m in man["metrics"]:
                v, cls = verdict(m["pass"])
                req = "—" if m["req"] is None or not np.isfinite(m["req"]) else f"{m['req']:g}"
                out.append(f"<tr><td>{m['id']}</td><td>{m['value']:.4g} {m['unit']}</td><td>{req}</td><td class='{cls}'>{v}</td></tr>")
            out.append("</table></div>")
        else:
            summ = obj
            out.append(f"<p class='muted'>Monte Carlo · scenario <b>{summ['scenario']}</b> · case <b>{summ['case']}</b> · {summ['runs']} runs · dispersed: mass properties, magnetic dipole, CM offset, aero/SRP surface properties, space weather, initial conditions, and every sensor/actuator part error from its descriptor.</p>")
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


if __name__ == "__main__":
    main()
