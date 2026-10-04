"""What every part of tools/report.py shares: the stores it reads, the palette, reading a filed run,
and drawing a figure. Every figure is drawn by the engine's plotting module (adcs-plot, through
`adcs figures` and `adcs plot`), the one plotting module for engine and twin runs alike: a run's
figures come from its folder, any other figure from a JSON description (the schema is in
engine/crates/adcs-plot/src/lib.rs). Figures are SVG.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import csv, json, math, os, pathlib, subprocess, tempfile
from common import ROOT, trace
from engine_base import BIN
import numpy as np


STORE = ROOT / "matlab_sils" / "store" / "results"


TRADES = ROOT / "matlab_sils" / "store" / "trades"


OUT = ROOT / "results"


FIG = OUT / "figures"


# Reference palette (dataviz skill, light mode), fixed slot order; adcs-plot draws with the same.
S1, S2, S3, S4 = "#2a78d6", "#eb6834", "#1baf7a", "#eda100"


INK, INK2, GRID, SURF = "#0b0b0b", "#52514e", "#e4e3df", "#fcfcfb"


MUTED = "#b9bec4"                          # a candidate that fails a requirement


MODES = ["detumble", "nadir_mtq", "nadir_fine", "target_fine", "slew_fine", "spinup", "sun_spin"]


def load_manifest(d):
    """A filed run's manifest, its one-element struct arrays read as lists."""
    man = json.loads((pathlib.Path(d) / "manifest.json").read_text())
    for k in ("metrics", "mode_log"):          # a one-element struct array is written as an object
        if isinstance(man.get(k), dict): man[k] = [man[k]]
    return man


def load_run(d):
    man = load_manifest(d)
    with open(d / "channels.csv") as f:
        r = csv.reader(f)
        hdr = next(r)
        data = np.array([[float(x) for x in row] for row in r])
    ch = {h: data[:, i] for i, h in enumerate(hdr)}
    return man, ch


def adcs():
    """The engine's command line, which draws the figures ($ADCS_BIN, else engine/target/release/adcs)."""
    b = pathlib.Path(os.environ.get("ADCS_BIN") or BIN)
    if not b.is_file():
        raise SystemExit(f"{b}: no engine build, and the engine draws every figure; build it: python3 tools/engine.py build")
    return str(b)


def _adcs(args, what):
    r = subprocess.run([adcs(), *map(str, args)], capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f"adcs {what}: {(r.stderr or r.stdout).strip()}")
    return r.stdout


def jsonable(o):
    """`o` as strict JSON takes it: NaN and infinities as null, numpy arrays and numbers as lists and floats."""
    if isinstance(o, dict):
        return {k: jsonable(v) for k, v in o.items()}
    if isinstance(o, (list, tuple)):
        return [jsonable(v) for v in o]
    if hasattr(o, "tolist"):
        return jsonable(o.tolist())
    if isinstance(o, float):
        return o if math.isfinite(o) else None
    return o


def plot(spec, out):
    """Draw a figure described as JSON (or a list of them) into `out` (.svg or .pdf) with `adcs plot`."""
    out = pathlib.Path(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(suffix=".json")
    try:
        with os.fdopen(fd, "w") as f:
            json.dump(jsonable(spec), f)
        _adcs(["plot", tmp, "--out", out], f"plot {out.name}")
    finally:
        os.unlink(tmp)
    trace(f"wrote {out}")
    return out


def plot_svg(spec):
    """The SVG text of a figure described as JSON (for a page that inlines its figures)."""
    with tempfile.TemporaryDirectory() as d:
        return plot(spec, pathlib.Path(d) / "figure.svg").read_text()


def save(spec, name):
    """Draw the figure into results/figures/<name>.svg; its file name."""
    return plot(spec, FIG / f"{name}.svg").name


def run_svgs(run_dir, prefix, full=True):
    """A filed run's figures (adcs figures: the same for an engine and a twin run) in results/figures;
    their file names, in order."""
    args = ["figures", run_dir, "--out", FIG, "--prefix", prefix] + (["--full"] if full else [])
    names = [pathlib.Path(l).name for l in _adcs(args, f"figures {run_dir}").splitlines() if l.strip()]
    trace(f"wrote {len(names)} figures of {prefix}")
    return names


def req_line(panel, y, label):
    """A requirement as a dashed line with its label on a panel description."""
    if y is not None and np.isfinite(y):
        panel.setdefault("refs", []).append({"axis": "y", "at": y, "label": f"{label} {y:g}"})
    return panel


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
