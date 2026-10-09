"""What every node of the design loop shares: where the stores are, the node registry's parameters, the knobs' bounds, hashing and writing.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "tools"))
import hashlib, json, pathlib
import engine as E
from common import write_text, ROOT




MS = ROOT / "matlab_sils"


BIN = E.BIN


PIPE = MS / "store" / "pipeline"


CACHE = PIPE / "cache"


OUT = ROOT / "results"


# which algorithm slot decides an option's performance, and the registry's candidates for it
SLOT = {("detumble", "mtq"): "detumble", ("sun_acquisition", "mtq"): "sun_acquisition"}


for m in ("sun_referencing", "nadir_pointing"):
    SLOT[(m, "mtq")] = "mtq_pointing"
    for o in ("rw", "cmg", "vscmg", "fmr"):
        for d in ("mtq", "rcs"):
            SLOT[(m, f"{o}+{d}")] = "pointing"


# every node's parameters come from the node registry (matlab_sils/data/pipeline/nodes.json, docs/NODES.md)
NODES = {n["id"]: n for n in json.loads((MS / "data" / "pipeline" / "nodes.json").read_text())["nodes"]}


P = lambda node: NODES[node]["parameters"]


CANDIDATES = P("matrix")["algorithm_candidates"]


TUNE = P("tune")


# a candidate "law@bwX" is the law with its pointing bandwidth tuned to X rad/s (fsw.rw_bandwidth);
# "law@key=value,key=value" is the law with those fsw gain scales (node tune)
def split_alg(a):
    if a and "@bw" in a:
        law, bw = a.split("@bw")
        return law, {"rw_bandwidth": float(bw)}
    if a and "@" in a:
        law, kv = a.split("@", 1)
        return law, {k: float(v) for k, v in (x.split("=") for x in kv.split(","))}
    return a, {}


def tune_grid(slot):
    """Node tune (Bruni & Celani 2017): every law of the slot at every point of its gain grid."""
    g = TUNE["grids"][slot]
    keys = list(g)
    pts = [[]]
    for k in keys:
        pts = [x + [v] for x in pts for v in g[k]]
    return [f"{law}@" + ",".join(f"{k}={v:g}" for k, v in zip(keys, pt)) for law in CANDIDATES[slot] for pt in pts]


(SCALE_MIN, SCALE_MAX), UP, DOWN = P("converge")["scale_bounds"], P("converge")["scale_up"], P("converge")["scale_down"]


LAMBDA_MIN, LAMBDA_MAX = P("converge")["fmr_lambda_bounds_kg_W"]


FLOW_SIGMA_MIN = P("converge")["fmr_flow_sigma_min_m_s"]


GYRO_MIN = P("converge")["gyro_grade_min"]


IMPROVE = 1.0 - P("converge")["improvement_needed"]


FAMILIES = []


def cls(metric):
    if metric.startswith("power"):
        return "power"
    if metric.startswith("ake"):
        return "knowledge"
    if metric.startswith("propellant"):
        return "propellant"
    return "performance"


def sha(*parts):
    h = hashlib.sha1()
    for p in parts:
        h.update(p if isinstance(p, bytes) else json.dumps(p, sort_keys=True).encode())
    return h.hexdigest()[:16]


def case_bytes(case):
    # the runs score against the case's requirements, so a changed case must not hit the cache
    return (MS / "cases" / f"{case}.csv").read_bytes()


def write(p, obj):
    p.parent.mkdir(parents=True, exist_ok=True)
    write_text(p, json.dumps(obj, indent=1))


def usable(o, fam_acts):
    need = {o["actuator"]} | ({o["dump"]} if o.get("dump") else set())
    return need <= set(fam_acts)


def jl_(p):
    return json.loads(p.read_text()) if p.exists() else None
