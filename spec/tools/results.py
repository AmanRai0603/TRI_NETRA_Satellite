#!/usr/bin/env python3
"""Result documents, adcs-result/1 (SPEC.md §13.5): what a finished run or
campaign is saved as, and what any copy of the software opens again without
re-running it.

    python3 tools/results.py demo [--out <dir>]        # two demonstration results (see below)
    python3 tools/results.py read <file.result.html>   # summary: case, product, verdicts, runs
    python3 tools/results.py import <file.result.html>... --store <dir>   # put results into a local store
    python3 tools/results.py index <dir>               # (re)build <dir>/index.csv from the result files in it
    python3 tools/results.py channels <file> <run> <out.csv>   # one kept run's channels as CSV

THE LOCAL STORE (SPEC.md §13.5.4). A result is never separated from the case it
was run on. `import` writes each result's own case CSV (carried inside the result)
to <store>/cases/<case id>/<sha12>.csv, the result beside it under
<store>/results/<case id>/<sha12>/, and rebuilds <store>/index.csv. Nothing runs:
importing a result is how a result made elsewhere is seen again. The files are the
record; the index is rebuilt from them and may be deleted at any time.

A result document is results/template.html with one JSON block filled in: the
case as uploaded, the product, scenario and campaign, every requirement with its
verdict, every metric per run and for the ensemble, and the recorded channels of
the runs kept, gzip-compressed and base64-encoded as little-endian float32,
column-major. It opens offline in any browser, and the web app, the workbench
and the MATLAB SILS tool all open the same file.

THE DEMO IS NOT THE PLATFORM. `demo` runs a deliberately simple Python model — a
rigid 3U with ais_3u's inertia, a circular 500 km orbit at 97.4°, a tilted
dipole field fixed in inertial space, ideal sensors, B-dot on three ideal coils
of SYN-CT-1's 0.45 A.m2 — only so the result document has real trajectories to
show. Its numbers are not evidence and are marked so in every file it writes:
the engine is named "demo", and the page says it.
"""

import base64
import csv
import datetime
import glob
import gzip
import hashlib
import io
import json
import math
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TEMPLATE = os.path.join(ROOT, "results", "template.html")
OPEN = '<script type="application/json" id="adcs-result">'
SCHEMA = "adcs-result/1"

try:
    import tomllib
except ImportError:  # pragma: no cover
    sys.exit("results.py needs Python 3.11+")


# ------------------------------------------------------------------ file format
def pack_channels(columns):
    """columns: list of equal-length float lists -> base64(gzip(float32 LE, column-major))."""
    import struct
    raw = b"".join(struct.pack("<%df" % len(c), *c) for c in columns)
    return base64.b64encode(gzip.compress(raw, compresslevel=9, mtime=0)).decode()


def unpack_channels(blob, n_channels, n_samples):
    import struct
    raw = gzip.decompress(base64.b64decode(blob))
    vals = struct.unpack("<%df" % (n_channels * n_samples), raw)
    return [list(vals[i * n_samples:(i + 1) * n_samples]) for i in range(n_channels)]


def fill(res):
    tpl = open(TEMPLATE, encoding="utf-8").read()
    body = json.dumps(res, ensure_ascii=False, separators=(",", ":")).replace("<", "\\u003c")
    title = res["title"].replace("&", "&amp;").replace("<", "&lt;")
    return tpl.replace("__TITLE__", title, 1).replace("__RESULT_JSON__", body, 1)


def read_result(path):
    text = open(path, encoding="utf-8").read()
    i = text.find(OPEN)
    if i < 0:
        raise SystemExit("%s: no adcs-result block; not a result document" % path)
    j = text.find("</script>", i)
    res = json.loads(text[i + len(OPEN):j])
    if res.get("schema") != SCHEMA:
        raise SystemExit("%s: schema %r, this reads %s" % (path, res.get("schema"), SCHEMA))
    return res


# ------------------------------------------------------------------ the demonstration model
def case_values(case_id):
    rows = list(csv.reader(open(os.path.join(ROOT, "plan", "cases", case_id + ".csv"), encoding="utf-8")))
    return rows, {r[1]: r for r in rows[1:]}


def quat_mul(a, b):
    w1, x1, y1, z1 = a
    w2, x2, y2, z2 = b
    return (w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2, w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
            w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2, w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2)


def rot_to_body(q, v):
    # v_body = q^* v q  for q = body->inertial
    w, x, y, z = q
    qc = (w, -x, -y, -z)
    r = quat_mul(quat_mul(qc, (0.0, v[0], v[1], v[2])), q)
    return (r[1], r[2], r[3])


def demo_run(k, seed, w0_deg, J, alt_km, inc_deg, t_end, dt, rec_dt, m_max):
    import random
    rng = random.Random(hash((seed, k)) & 0xFFFFFFFF)
    mu, Re = 3.986004418e14, 6.3781363e6
    r = Re + alt_km * 1e3
    n = math.sqrt(mu / r ** 3)
    inc = math.radians(inc_deg)
    # Tilted dipole, fixed in inertial space (a demonstration simplification).
    m_e = 7.94e22 * 1e-7   # mu0/(4 pi) * dipole moment
    tilt = math.radians(11.0)
    mhat = (math.sin(tilt), 0.0, -math.cos(tilt))

    def field(t):
        u = n * t
        rv = (r * math.cos(u), r * math.sin(u) * math.cos(inc), r * math.sin(u) * math.sin(inc))
        rh = tuple(c / r for c in rv)
        d = sum(a * b for a, b in zip(mhat, rh))
        return tuple(m_e / r ** 3 * (3 * d * rh[i] - mhat[i]) for i in range(3))

    # random initial rate direction and attitude
    v = [rng.gauss(0, 1) for _ in range(3)]
    s = math.sqrt(sum(c * c for c in v))
    w = [math.radians(w0_deg) * c / s for c in v]
    qv = [rng.gauss(0, 1) for _ in range(4)]
    s = math.sqrt(sum(c * c for c in qv))
    q = tuple(c / s for c in qv)
    xi = inc   # the Avanzini-Giulietti gain uses the orbit's magnetic inclination; the orbit inclination stands in for it here
    kgain = 2 * n * (1 + math.sin(xi)) * min(J)
    b_prev = rot_to_body(q, field(0.0))
    rec = {c: [] for c in ("w_x", "w_y", "w_z", "w_norm", "b_x", "b_y", "b_z", "m_x", "m_y", "m_z", "q_w", "q_x", "q_y", "q_z")}
    t, next_rec, m = 0.0, 0.0, (0.0, 0.0, 0.0)

    def deriv(qq, ww, tt, mm):
        bb = rot_to_body(qq, field(tt))
        tau = (mm[1] * bb[2] - mm[2] * bb[1], mm[2] * bb[0] - mm[0] * bb[2], mm[0] * bb[1] - mm[1] * bb[0])
        h = (J[0] * ww[0], J[1] * ww[1], J[2] * ww[2])
        cross = (ww[1] * h[2] - ww[2] * h[1], ww[2] * h[0] - ww[0] * h[2], ww[0] * h[1] - ww[1] * h[0])
        wd = tuple((tau[i] - cross[i]) / J[i] for i in range(3))
        qd = tuple(0.5 * c for c in quat_mul(qq, (0.0, ww[0], ww[1], ww[2])))
        return qd, wd

    steps = int(round(t_end / dt))
    for _ in range(steps + 1):
        b = rot_to_body(q, field(t))
        if t >= next_rec - 1e-9:
            wn = math.sqrt(sum(c * c for c in w))
            for name, val in zip(("w_x", "w_y", "w_z", "w_norm"), (w[0], w[1], w[2], wn)):
                rec[name].append(math.degrees(val))
            for name, val in zip(("b_x", "b_y", "b_z"), b):
                rec[name].append(val * 1e6)
            for name, val in zip(("m_x", "m_y", "m_z"), m):
                rec[name].append(val)
            for name, val in zip(("q_w", "q_x", "q_y", "q_z"), q):
                rec[name].append(val)
            next_rec += rec_dt
        bdot = tuple((b[i] - b_prev[i]) / dt for i in range(3))
        b_prev = b
        m = tuple(max(-m_max, min(m_max, -kgain * bdot[i] / max(1e-12, sum(c * c for c in b)))) for i in range(3))
        # RK4
        k1q, k1w = deriv(q, w, t, m)
        q2 = tuple(q[i] + 0.5 * dt * k1q[i] for i in range(4)); w2 = [w[i] + 0.5 * dt * k1w[i] for i in range(3)]
        k2q, k2w = deriv(q2, w2, t + dt / 2, m)
        q3 = tuple(q[i] + 0.5 * dt * k2q[i] for i in range(4)); w3 = [w[i] + 0.5 * dt * k2w[i] for i in range(3)]
        k3q, k3w = deriv(q3, w3, t + dt / 2, m)
        q4 = tuple(q[i] + dt * k3q[i] for i in range(4)); w4 = [w[i] + dt * k3w[i] for i in range(3)]
        k4q, k4w = deriv(q4, w4, t + dt, m)
        q = tuple(q[i] + dt / 6 * (k1q[i] + 2 * k2q[i] + 2 * k3q[i] + k4q[i]) for i in range(4))
        w = [w[i] + dt / 6 * (k1w[i] + 2 * k2w[i] + 2 * k3w[i] + k4w[i]) for i in range(3)]
        s = math.sqrt(sum(c * c for c in q)); q = tuple(c / s for c in q)
        t += dt
    # time_to_rate: first time |w| < 0.5 deg/s and stays there for 600 s
    thr, hold = 0.5, 600.0
    ts = [i * rec_dt for i in range(len(rec["w_norm"]))]
    ttr = None
    for i, wn in enumerate(rec["w_norm"]):
        if wn < thr and all(x < thr for x, tt in zip(rec["w_norm"][i:], ts[i:]) if tt <= ts[i] + hold) and ts[i] + hold <= ts[-1]:
            ttr = ts[i]
            break
    return rec, {"detumble_time": None if ttr is None else ttr / 60.0, "max_rate": max(rec["w_norm"])}, \
        {"w0_direction": [round(c / math.radians(w0_deg), 4) for c in w]}


def demo(out):
    rows, cv = case_values("ais_3u")
    val = lambda k: float(cv[k][4])
    J = (val("mass.imax"), val("mass.iint"), val("mass.imin"))
    common = dict(J=J, alt_km=val("orbit.alt"), inc_deg=val("orbit.inc"), t_end=22800.0, dt=1.0, rec_dt=2.0, m_max=0.45)
    csv_text = open(os.path.join(ROOT, "plan", "cases", "ais_3u.csv"), encoding="utf-8").read()
    reqs = [{"key": "req.detumble", "tree_id": "p3k_0", "label": "Detumble time", "unit": "Minute", "sense": "<=",
             "value": val("req.detumble"), "level": 99.73, "level_assumed": True}]
    names = ["w_x", "w_y", "w_z", "w_norm", "b_x", "b_y", "b_z", "m_x", "m_y", "m_z", "q_w", "q_x", "q_y", "q_z"]
    units = ["deg/s"] * 4 + ["µT"] * 3 + ["A·m²"] * 3 + ["1"] * 4
    engine = {"name": "demo", "version": "tools/results.py", "commit": "package",
              "note": "A deliberately simple Python B-dot model written to exercise the result document: rigid 3U, circular orbit, "
                      "tilted dipole fixed in inertial space, no disturbances, ideal sensors and coils. It is not the platform "
                      "engine. Its numbers are not evidence."}
    base = {"schema": SCHEMA, "created": datetime.date.today().isoformat(), "engine": engine,
            "case": {"id": "ais_3u", "hash": hashlib.sha256(csv_text.encode()).hexdigest()[:16], "csv": csv_text},
            "product": {"id": "SYN-P-3U-MTQ", "family": "mtq", "synthetic": True, "parts": ["SYN-CT-1", "SYN-MAG-1", "IDM-CTRL-1"]},
            "tuned": {"hash": None, "note": "B-dot at the reference gain, not tuned"},
            "scenario": {"id": "detumble_3u", "duration_s": 22800.0}, "requirements": reqs,
            "channels": {"rate_hz": 0.5, "names": names, "units": units, "runs": {}, "groups": [
                {"title": "Body rate magnitude", "unit": "deg/s", "channels": ["w_norm"], "labels": ["|ω|"],
                 "hline": {"y": 0.5, "label": "threshold 0.5 °/s"}, "time_marks": True},
                {"title": "Body rate per axis", "unit": "deg/s", "channels": ["w_x", "w_y", "w_z"], "labels": ["ω x", "ω y", "ω z"], "zero": False},
                {"title": "Measured field, body frame", "unit": "µT", "channels": ["b_x", "b_y", "b_z"], "labels": ["B x", "B y", "B z"], "zero": False, "half": True},
                {"title": "Commanded dipole", "unit": "A·m²", "channels": ["m_x", "m_y", "m_z"], "labels": ["m x", "m y", "m z"], "zero": False, "half": True}]},
            "restricted_excluded": [], "notes": [],
            "credibility": {"level": "none", "why": "a demonstration engine: its numbers are not evidence"},
            "limits": ["The engine is a demonstration model, not the platform: no disturbance torques, ideal sensors and coils, "
                       "and a dipole field fixed in inertial space.",
                       "The product SYN-P-3U-MTQ is synthetic: its parts hold test values, and it can never be offered."]}
    written = []
    # 1) a nominal run
    rec, met, disp = demo_run(0, 20260927, 10.0, **common)
    n = len(rec["w_norm"])
    res = json.loads(json.dumps(base))
    res.update({"kind": "run", "title": "Demo — detumble, ais_3u on SYN-P-3U-MTQ (nominal)",
                "campaign": {"id": None, "type": "nominal", "runs": 1, "seed": 20260927},
                "runs": [{"k": 0, "kept": True, "dispersions": disp, "metrics": met}]})
    res["channels"]["n"] = n
    res["channels"]["runs"]["0"] = pack_channels([rec[c] for c in names])
    res["metrics"] = [metric_summary("detumble_time", "time_to_rate", "Minute", "req.detumble", [met["detumble_time"]], reqs[0]),
                      metric_summary("max_rate", "max_rate", "deg/s", None, [met["max_rate"]], None)]
    written.append(write(res, out, "demo_detumble_ais_3u_nominal"))
    # 2) a 20-run Monte Carlo, keeping the best, the worst and run 0
    runs, recs = [], {}
    for k in range(20):
        rec_k, met_k, disp_k = demo_run(k, 20260928, 10.0, **common)
        runs.append({"k": k, "kept": False, "dispersions": disp_k, "metrics": met_k})
        recs[k] = rec_k
    vals = [r["metrics"]["detumble_time"] for r in runs]
    finite = [(v, r["k"]) for v, r in zip(vals, runs) if v is not None]
    keep = {0}
    if finite:
        keep |= {min(finite)[1], max(finite)[1]}
    res = json.loads(json.dumps(base))
    res.update({"kind": "campaign", "title": "Demo — detumble, ais_3u on SYN-P-3U-MTQ (20-run Monte Carlo, random rate direction)",
                "campaign": {"id": "demo_detumble_mc20", "type": "montecarlo", "runs": 20, "seed": 20260928,
                             "dispersed": ["initial rate direction", "initial attitude"]},
                "runs": runs})
    res["channels"]["n"] = len(recs[0]["w_norm"])
    for k in sorted(keep):
        runs[k]["kept"] = True
        res["channels"]["runs"][str(k)] = pack_channels([recs[k][c] for c in names])
    res["metrics"] = [metric_summary("detumble_time", "time_to_rate", "Minute", "req.detumble", vals, reqs[0]),
                      metric_summary("max_rate", "max_rate", "deg/s", None, [r["metrics"]["max_rate"] for r in runs], None)]
    res["notes"].append("20 runs cannot support a 99.73 % ensemble percentile: the statistic reported is the largest of the 20, "
                        "and the page says so. A real campaign states its run count and confidence (SPEC.md §5.5).")
    written.append(write(res, out, "demo_detumble_ais_3u_mc20"))
    return written


def metric_summary(mid, kind, unit, req_key, values, req):
    finite = sorted(v for v in values if v is not None)
    missing = sum(1 for v in values if v is None)
    stat = None
    level = req["level"] if req else None
    note = ""
    if finite:
        if level and len(values) > 1:
            k = min(len(values), max(1, math.ceil(level / 100.0 * len(values))))
            stat = (finite + [float("inf")] * missing)[k - 1]
            if k == len(values):
                note = "the largest of %d runs; too few runs for a %.2f %% percentile" % (len(values), level)
        else:
            stat = finite[-1] if len(finite) == 1 else max(finite)
    if missing and (stat is None or stat == float("inf")):
        stat, note = None, "%d run(s) never met the threshold" % missing
    verdict, margin = "no requirement", None
    if req:
        if stat is None:
            verdict = "fail"
        else:
            margin = (req["value"] - stat) / abs(req["value"]) if req["sense"] == "<=" else (stat - req["value"]) / abs(req["value"])
            verdict = "pass" if margin >= 0 else "fail"
    return {"id": mid, "kind": kind, "unit": unit, "requirement": req_key, "values": values,
            "ensemble": {"statistic": "p%.2f" % level if level else "value", "value": stat, "note": note},
            "margin": margin, "verdict": verdict}


def write(res, out, stem):
    os.makedirs(out, exist_ok=True)
    path = os.path.join(out, stem + ".result.html")
    open(path, "w", encoding="utf-8").write(fill(res))
    return path


# ------------------------------------------------------------------ read, index, channels
def summary(res):
    lines = ["%s · %s" % (res["title"], res["schema"]),
             "engine %s (%s) · case %s (%s) · product %s · scenario %s" % (
                 res["engine"]["name"], res["engine"].get("version"), res["case"]["id"], res["case"]["hash"],
                 res["product"]["id"], res["scenario"]["id"]),
             "%s: %d run(s), kept channels for %s" % (res["kind"], len(res["runs"]), sorted(int(k) for k in res["channels"]["runs"]))]
    for m in res["metrics"]:
        e = m["ensemble"]
        lines.append("  %-16s %-8s %s = %s %s  margin %s  %s" % (
            m["id"], m["verdict"], e["statistic"], "—" if e["value"] is None else "%.4g" % e["value"], m["unit"],
            "—" if m["margin"] is None else "%+.1f %%" % (100 * m["margin"]), e.get("note", "")))
    return "\n".join(lines)


def csv_sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:12]


def store_import(paths, store):
    """Each result into the store, with its case beside it. Nothing is run."""
    done = []
    for p in paths:
        r = read_result(p)
        cid = r["case"]["id"]
        if not cid or any(c in cid for c in "/\\.") :
            raise SystemExit("%s: case id %r cannot name a folder" % (p, cid))
        text = r["case"].get("csv") or ""
        if not text:
            raise SystemExit("%s: the result carries no case CSV; it cannot be stored beside its case" % p)
        sha = csv_sha(text)
        cdir = os.path.join(store, "cases", cid)
        os.makedirs(cdir, exist_ok=True)
        cpath = os.path.join(cdir, sha + ".csv")
        if not os.path.exists(cpath):
            open(cpath, "w", encoding="utf-8", newline="").write(text)
        rdir = os.path.join(store, "results", cid, sha)
        os.makedirs(rdir, exist_ok=True)
        dest = os.path.join(rdir, os.path.basename(p))
        raw = open(p, "rb").read()
        if os.path.exists(dest) and open(dest, "rb").read() != raw:
            stem = os.path.basename(p)[:-len(".result.html")]
            dest = os.path.join(rdir, "%s_%s.result.html" % (stem, hashlib.sha256(raw).hexdigest()[:6]))
        if not os.path.exists(dest):
            open(dest, "wb").write(raw)
        done.append((dest, cpath))
    index(store)
    return done


def index(d):
    rows = []
    for p in sorted(glob.glob(os.path.join(d, "**", "*.result.html"), recursive=True)):
        r = read_result(p)
        worst = [m["margin"] for m in r["metrics"] if m["margin"] is not None]
        text = r["case"].get("csv") or ""
        cfile = os.path.join("cases", r["case"]["id"], csv_sha(text) + ".csv") if text else ""
        rows.append([os.path.relpath(p, d), cfile if os.path.exists(os.path.join(d, cfile)) else "", r["kind"], r["created"],
                     r["engine"]["name"], r["case"]["id"], r["case"]["hash"],
                     r["product"]["id"], r["scenario"]["id"], (r.get("campaign") or {}).get("type"), len(r["runs"]),
                     "fail" if any(m["verdict"] == "fail" for m in r["metrics"]) else "pass",
                     "" if not worst else "%.4f" % min(worst),
                     hashlib.sha256(open(p, "rb").read()).hexdigest()[:16]])
    out = os.path.join(d, "index.csv")
    with open(out, "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["file", "case_file", "kind", "created", "engine", "case", "case_hash", "product", "scenario", "campaign_type", "runs",
                    "verdict", "worst_margin", "sha256"])
        w.writerows(rows)
    return out, len(rows)


def main():
    a = sys.argv[1:]
    out = os.path.join(ROOT, "results", "examples")
    if "--out" in a:
        i = a.index("--out"); out = a[i + 1]; del a[i:i + 2]
    store = None
    if "--store" in a:
        i = a.index("--store"); store = a[i + 1]; del a[i:i + 2]
    if a == ["demo"]:
        for p in demo(out):
            print("wrote", os.path.relpath(p, ROOT))
        return 0
    if len(a) == 2 and a[0] == "read":
        print(summary(read_result(a[1])))
        return 0
    if len(a) >= 2 and a[0] == "import" and store:
        for dest, cpath in store_import(a[1:], store):
            print("stored %s  (case %s)" % (os.path.relpath(dest, store), os.path.relpath(cpath, store)))
        print("nothing was run; open any stored result to see it, or its index: %s" % os.path.join(store, "index.csv"))
        return 0
    if len(a) == 2 and a[0] == "index":
        p, n = index(a[1])
        print("wrote %s: %d result(s)" % (p, n))
        return 0
    if len(a) == 4 and a[0] == "channels":
        r = read_result(a[1])
        ch = r["channels"]
        if a[2] not in ch["runs"]:
            raise SystemExit("run %s was not kept; kept: %s. Re-run it from the manifest to get its channels." % (a[2], sorted(ch["runs"])))
        cols = unpack_channels(ch["runs"][a[2]], len(ch["names"]), ch["n"])
        with open(a[3], "w", newline="") as f:
            w = csv.writer(f)
            w.writerow(["t_s"] + ["%s [%s]" % (n, u) for n, u in zip(ch["names"], ch["units"])])
            for i in range(ch["n"]):
                w.writerow(["%g" % (i / ch["rate_hz"])] + ["%.7g" % c[i] for c in cols])
        print("wrote", a[3])
        return 0
    print(__doc__.split("\n\n")[1])
    return 2


if __name__ == "__main__":
    sys.exit(main())
