"""What every part of tools/report.py shares: the stores it reads, the palette, and reading a filed run.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import csv, json, pathlib
import numpy as np
import matplotlib
import matplotlib.pyplot as plt


matplotlib.use("Agg")


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


def verdict(p):
    if p is None or (isinstance(p, float) and not np.isfinite(p)):
        return "—", ""
    return ("✔ PASS", "pass") if p else ("✖ FAIL", "fail")


def mval(man, mid):
    for m in man["metrics"]:
        if m["id"] == mid:
            return m["value"]
    return float("nan")


def fnum(x):
    return float("nan") if x is None else float(x)


def aslist(x):
    return x if isinstance(x, list) else ([] if x is None else [x])
