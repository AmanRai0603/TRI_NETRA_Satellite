"""The algorithm and hardware trades: loading, figures and their page section.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import json, html
import numpy as np
import matplotlib.pyplot as plt
from report_base import INK, INK2, S1, S3, TRADES, fnum, save


TRADE_ORDER = ["trade_detumble_ais", "trade_mtq_pointing_ais", "trade_sun_spin_ais", "trade_sun_sensor_ais",
               "trade_pointing_img", "trade_slew_img", "trade_pointing_cmg", "trade_pointing_vscmg", "trade_pointing_fmr",
               "trade_allocation_fmr", "trade_hardware_fine", "trade_hardware_agile"]


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
