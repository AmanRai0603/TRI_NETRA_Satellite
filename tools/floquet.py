#!/usr/bin/env python3
"""Node certify: the Floquet certificate of the coils-only nadir loop (Celani 2026's certificate method), ctl's method.

    python3 tools/floquet.py <case>          -> matlab_sils/store/pipeline/<case>/floquet.json

The certificate is the design's (docs/S7_INVENTORY.md S7.15b; the owner's decision 3: an analysis method of ctl): the
node ctl_floquet_certificate, module floquet, generated into the engine (adcs-design). Its method finds the nadir
reference frame the mission flew, linearises the closed loop of each magnetic pointing law about it, integrates the
monodromy matrix over one orbit, takes its multipliers (the toolbox's eig) and gives the verdict: all |mu| < 1 (a
boresight law's free direction aside) certifies local exponential stability of that periodic linear loop. This tool keeps
no relation of its own: it reads the inputs, asks the engine (`adcs design call floquet::...`, tools/design_call.py)
and writes what it answers.

The inputs: the case's orbit (inclination, altitude); the selected family's mission channels (the reference frame is read
from a mission that holds nadir); for each law the method judges (floquet::fq_laws), the flight software's parameters as
it boots with them: each law's configuration blob built with `adcs params` and decoded, the dispatched law with the
mission's tuned gains, any other with its nominal ones.

The committed results (matlab_sils/store/pipeline/<case>/floquet.json) were written by this tool's numpy version (1.0.0):
the method gives every multiplier within 1e-12 of them and every verdict the same, not their last bits (numpy's matrix
products ran on OpenBLAS's fused multiply-add kernels, its eigenvalues on LAPACK's dgeev); tests/test_floquet.py holds
that. They stand until the owner agrees to write them again by this path.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import csv
import json
import os
import subprocess
import sys

from common import write_text, ROOT
from design_call import Buf, call, take

MS = ROOT / "matlab_sils"
PIPE = MS / "store" / "pipeline"
sys.path.insert(0, str(ROOT / "tools"))
import engine as E                                     # noqa: E402
import fswcfg                                          # noqa: E402

# The flight software's magnetic pointing laws by its number (the configuration's mtq_law): the scenario's words for
# them, in the order of the design's choice (fswchoice's MtqAlg; tests/test_floquet.py holds them equal).
MTQ_LAW = ["mtq_pd", "mtq_lqr", "mtq_smc", "mtq_rate_damp", "mtq_lovera2004", "mtq_celani2015", "mtq_avanzini2021",
           "mtq_celani2026", "mtq_tango2013"]
# The method's record FqGains, field by field in its order: the blob's fields of the same names (the test holds them).
GAINS = ["J", "mtq_meas", "mtq_period", "roll_axis", "mtq_gg_ff", "mtq_Kp", "mtq_Kd", "mtq_eps", "mtq_k1", "mtq_k2", "mtq_k16",
         "mtq_lam16", "sb_kp", "sb_kd", "sb_kroll", "sb_kdroll", "sb_roll_gate", "sun_axis", "mtq_Pth", "mtq_Pw"]
# The channels the method reads of a run, by sample: time, mode, position (ECI), attitude.
CHANNELS = ["t_s", "mode", "r_x_m", "r_y_m", "r_z_m", "q_x", "q_y", "q_z", "q_w"]


def laws():
    """The laws the certificate judges, by name, in the method's order."""
    return [MTQ_LAW[int(k)] for k in call("floquet::fq_laws")]


def channels(path):
    """The method's channels of a run's channels.csv, each a column of numbers."""
    cols = [[] for _ in CHANNELS]
    with open(path, newline="") as f:
        for r in csv.DictReader(f):
            for c, k in zip(cols, CHANNELS):
                c.append(float(r[k]))
    return cols


def frame(path, q_off, roll_axis):
    """body <- orbit, the nadir reference the mission flew (floquet::fq_frame), as rows."""
    cols = channels(path)
    out = call("floquet::fq_frame", *[Buf(c) for c in cols], q_off, roll_axis)
    c_bo, *_ = take(out, 9, *["buf"] * len(cols))
    return [c_bo[0:3], c_bo[3:6], c_bo[6:9]]


def blob(scen, path, case, pkg):
    """The flight software's parameters a scenario boots with: its blob built by `adcs params`, decoded."""
    write_text(path, json.dumps(scen))
    subprocess.run([str(E.BIN), "params", str(path), "--case", str(MS / "cases" / f"{case}.csv"), "--out", str(path.with_suffix(".bin"))], cwd=MS,
                   check=True, capture_output=True, env={**os.environ, "ADCS_SIZED_DIR": str(pkg / "sized")})
    return fswcfg.decode(path.with_suffix(".bin").read_bytes())


def certify(case, pipe=None):
    pipe = pipe or PIPE
    fm = json.loads((pipe / case / "families.json").read_text())
    sel = json.loads((pipe / case / "selection.json").read_text())
    if "mtq" not in fm:
        return None
    fam = fm["mtq"]
    pkg = ROOT / fam["package"]
    scen = json.loads((pkg / "mission_scenario.json").read_text())
    # the reference frame from a mission that holds nadir: the selected family's
    ref = ROOT / fm[sel["selected"]]["check_dir"] / "c" / "channels.csv"
    (pipe / case / "floquet").mkdir(parents=True, exist_ok=True)
    P0 = blob(scen, pipe / case / "floquet" / "_boot.json", case, pkg)
    c_bo = frame(ref, P0["gd_q_off"], P0["roll_axis"])
    inc, alt = E.case_value(case, "orbit.inc"), E.case_value(case, "orbit.alt")
    out = {"case": case, "family": "mtq", "dispatched_law": (fam["algorithms"] or {}).get("mtq_pointing"), "C_body_orbit": c_bo, "laws": []}
    tune = {k: v for k, v in scen["fsw"].items() if k in ("mtq_gain_p", "mtq_gain_d")}
    for law in laws():
        s = json.loads(json.dumps(scen))
        s["fsw"].setdefault("algorithms", {})["mtq_pointing"] = law
        for k in ("mtq_gain_p", "mtq_gain_d"):
            s["fsw"].pop(k, None)
        if law == out["dispatched_law"]:
            s["fsw"].update(tune)
        P = blob(s, pipe / case / "floquet" / f"{law}.json", case, pkg)
        mu, max_mu, free, cert = take(call("floquet::fq_certify", MTQ_LAW.index(law), [P[k] for k in GAINS], c_bo, inc, alt), 6, 1, 1, 1)
        out["laws"].append({"law": law, "dispatched": law == out["dispatched_law"], "gains": {k: v for k, v in s["fsw"].items() if k.startswith("mtq_gain")} or "nominal",
                            "mu_abs": mu, "max_mu": max_mu, "free_directions": int(free), "certified": cert == 1.0})
    write_text(pipe / case / "floquet.json", json.dumps(out, indent=1))
    return out


if __name__ == "__main__":
    for c in sys.argv[1:] or ["ais_3u", "ais_img_3u"]:
        r = certify(c)
        if r:
            print(c, ", ".join(f"{x['law']} max|mu| {x['max_mu']:.4f}{' (dispatched)' if x['dispatched'] else ''}{' certified' if x['certified'] else ''}" for x in r["laws"]))
