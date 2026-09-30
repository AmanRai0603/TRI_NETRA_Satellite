#!/usr/bin/env python3
"""Node certify: Floquet multipliers of the coils-only nadir loop (Celani 2026's certificate method).

    python3 tools/floquet.py <case>          -> matlab_sils/store/pipeline/<case>/floquet.json

For each magnetic pointing law the closed loop, as flown (the law's nonlinear form, rigid-body
gyroscopic coupling, gravity gradient, the coil duty d = 1 - mtq_meas/mtq_period), is linearised
numerically about the nadir reference, where the body turns at the orbit rate about the orbit normal (the reference is the flight software's own nadir frame, with its yaw flip):

    x = [error rotation, body rate],   J w_dot = -w x J w + 3 n^2 c x J c + d Gamma(t) tau(q_e, w),   Gamma = I - b b^T b(t) is the unit field in the reference body frame along one circular orbit,
from an aligned dipole in the orbit frame (x along track, y = -orbit normal, z = nadir). The body-to-orbit
rotation is the guidance frame itself (boresight offset, and the yaw flip if the mission flew it). The monodromy
matrix over one orbit gives the multipliers; all |mu| < 1 certifies local exponential stability of that
periodic linear loop. A boresight law leaves the rotation about its boresight free, so one multiplier
stays at 1 by design. The gains are the ones the flight software boots with: each law's config blob is
built with `adcs params` and decoded, so they are the dispatched values and not recomputed here.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import csv, json, math, pathlib, subprocess, sys
import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[1]
MS = ROOT / "matlab_sils"
PIPE = MS / "store" / "pipeline"
sys.path.insert(0, str(ROOT / "tools"))
import engine as E                                     # noqa: E402
import fswcfg                                          # noqa: E402

LAWS = {0: "mtq_pd", 3: "mtq_rate_damp", 4: "mtq_lovera2004", 5: "mtq_celani2015", 6: "mtq_avanzini2021", 7: "mtq_celani2026", 8: "mtq_tango2013"}


def body_from_orbit(channels, q_off, roll_axis):
    """The flight software's nadir reference, body <- orbit (x along track, y = -orbit normal, z = nadir).

    Guidance builds the reference rows [-ram, -r, -n] (04_guidance.md), i.e. R0 below in orbit axes, then
    applies the boresight offset q_off and, when the yaw flip is on, 180 deg about the boresight. Which of the
    two the mission flew is read from its nadir phase: the candidate nearest the mean truth attitude there."""
    R0 = np.array([[-1.0, 0, 0], [0, 0, 1], [0, 1, 0]])
    u = np.array(roll_axis) / np.linalg.norm(roll_axis)
    A0 = dcm(np.array(q_off)) @ R0
    cands = [A0, dcm(np.array([u[0], u[1], u[2], 0.0])) @ A0]
    rows = [r for r in csv.DictReader(open(channels)) if int(float(r["mode"])) - 1 in (1, 2)]   # nadir_mtq, nadir_fine
    rows = rows[-len(rows) // 2:]
    acc = np.zeros((3, 3))
    for k in range(1, len(rows)):
        r0 = np.array([float(rows[k - 1][f"r_{a}_m"]) for a in "xyz"])
        r1 = np.array([float(rows[k][f"r_{a}_m"]) for a in "xyz"])
        v = (r1 - r0) / (float(rows[k]["t_s"]) - float(rows[k - 1]["t_s"]))
        z = -r1 / np.linalg.norm(r1)
        h = np.cross(r1, v); y = -h / np.linalg.norm(h)
        x = np.cross(y, z)
        Co = np.vstack([x, y, z])                       # rows: orbit axes in ECI
        q = np.array([float(rows[k][c]) for c in ("q_x", "q_y", "q_z", "q_w")])
        acc += dcm(q) @ Co.T                             # body <- orbit (the same passive dcm as the flight software)
    return max(cands, key=lambda C: np.trace(C.T @ acc))


def qmul(a, b):
    ax, ay, az, aw = a; bx, by, bz, bw = b
    return np.array([aw * bx + bw * ax + ay * bz - az * by, aw * by + bw * ay + az * bx - ax * bz,
                     aw * bz + bw * az + ax * by - ay * bx, aw * bw - ax * bx - ay * by - az * bz])


def dcm(q):
    """The flight software's dcm(q_e): reference-frame vectors into the body (fsw/src/adcs_math.c)."""
    x, y, z, w = q
    return np.array([[1 - 2 * (y * y + z * z), 2 * (x * y + z * w), 2 * (x * z - y * w)],
                     [2 * (x * y - z * w), 1 - 2 * (x * x + z * z), 2 * (y * z + x * w)],
                     [2 * (x * z + y * w), 2 * (y * z - x * w), 1 - 2 * (x * x + y * y)]])


def rv2q(v):
    a = np.linalg.norm(v)
    if a < 1e-15:
        return np.array([v[0] / 2, v[1] / 2, v[2] / 2, 1.0])
    s = math.sin(a / 2) / a
    return np.array([v[0] * s, v[1] * s, v[2] * s, math.cos(a / 2)])


def law_torque(law, P, qe, w, wref_r, e3, s_body):
    """The flight software's law (05_control.md), nonlinear, with q_e = q_ref^-1 q and w_ref in the reference frame."""
    J = np.array(P["J"]); A = dcm(qe); wr = A @ wref_r; we = w - wr
    sg = 1.0 if qe[3] >= 0 else -1.0
    qv = qe[:3]
    if law == 0:
        return -np.array(P["mtq_Kp"]) * (sg * qv) - np.array(P["mtq_Kd"]) * we
    if law == 3:
        return -np.array(P["mtq_Kd"]) * we
    if law == 4:
        e = P["mtq_eps"]; return -(e * e * P["mtq_k1"] * (sg * qv) + e * P["mtq_k2"] * (J @ we))
    if law == 5:
        e = P["mtq_eps"]; return -(e * e * P["mtq_k1"] * (sg * qv) + e * P["mtq_k2"] * we)
    if law == 6:
        n = np.linalg.norm(wref_r); ep = wref_r / n; sgm = A @ ep; Jp = ep @ J @ ep
        th = 2 * sg * (qv @ ep); eta = Jp * n * (1 - P["mtq_lam16"] * th); Jw = J @ w; k = P["mtq_k16"]
        return k * (eta * sgm - Jw) + k * (eta * ep - Jw)
    if law == 7:
        a = A @ e3
        return P["sb_kp"] * np.cross(e3, a) - P["sb_kd"] * we
    if law == 8:
        return -(np.array(P["mtq_Pth"]) @ (2 * sg * qv) + np.array(P["mtq_Pw"]) @ we)
    raise ValueError(law)


def monodromy(law, P, C_bo, inc, n, steps=1500):
    """Numerical linearisation of the full loop (gyroscopic, gravity gradient, the law as flown, coil duty)
    about the reference, integrated over one orbit."""
    J = np.array(P["J"]); Ji = np.linalg.inv(J)
    duty = 1 - P["mtq_meas"] / P["mtq_period"]
    ep = -C_bo[:, 1]; nad = C_bo[:, 2]                  # orbit normal and nadir in the reference body frame
    wref_r = n * ep
    e3 = np.array(P["roll_axis"])
    T = 2 * math.pi / n

    def f(t, x):
        qe = rv2q(x[:3]); w = x[3:]
        A = dcm(qe)
        u = n * t
        bo = np.array([math.cos(u) * math.sin(inc), -math.cos(inc), 2 * math.sin(u) * math.sin(inc)])
        b = A @ (C_bo @ (bo / np.linalg.norm(bo)))
        G = np.eye(3) - np.outer(b, b)
        tau = law_torque(law, P, qe, w, wref_r, e3, None)
        c = A @ nad
        if int(P["mtq_gg_ff"]) & 2:                     # nadir-state gravity-gradient feed-forward (05_control.md)
            tau = tau - 3 * n * n * np.cross(c, J @ c)
        wdot = Ji @ (-np.cross(w, J @ w) + 3 * n * n * np.cross(c, J @ c) + duty * (G @ tau))
        we = w - A @ wref_r
        return np.concatenate([we, wdot])                 # small-rotation kinematics of the error

    x0 = np.concatenate([np.zeros(3), wref_r])

    def jac(t):
        Jm = np.zeros((6, 6)); h = 1e-7
        for j in range(6):
            d = np.zeros(6); d[j] = h
            Jm[:, j] = (f(t, x0 + d) - f(t, x0 - d)) / (2 * h)
        return Jm

    h = T / steps; Phi = np.eye(6); t = 0.0
    for _ in range(steps):
        A1 = jac(t); A2 = jac(t + h / 2); A3 = jac(t + h)
        k1 = A1 @ Phi; k2 = A2 @ (Phi + h / 2 * k1); k3 = A2 @ (Phi + h / 2 * k2); k4 = A3 @ (Phi + h * k3)
        Phi = Phi + h / 6 * (k1 + 2 * k2 + 2 * k3 + k4); t += h
    return np.linalg.eigvals(Phi)


def certify(case):
    fm = json.loads((PIPE / case / "families.json").read_text())
    sel = json.loads((PIPE / case / "selection.json").read_text())
    if "mtq" not in fm:
        return None
    fam = fm["mtq"]
    pkg = ROOT / fam["package"]
    scen = json.loads((pkg / "mission_scenario.json").read_text())
    # the reference frame from a mission that holds nadir: the selected family's
    ref = ROOT / fm[sel["selected"]]["check_dir"] / "c" / "channels.csv"
    sp0 = PIPE / case / "floquet" / "_boot.json"; sp0.parent.mkdir(parents=True, exist_ok=True); sp0.write_text(json.dumps(scen))
    subprocess.run([str(E.BIN), "params", str(sp0), "--case", str(MS / "cases" / f"{case}.csv"), "--out", str(sp0.with_suffix(".bin"))], cwd=MS,
                   check=True, capture_output=True, env={**__import__("os").environ, "ADCS_SIZED_DIR": str(pkg / "sized")})
    P0 = fswcfg.decode(sp0.with_suffix(".bin").read_bytes())
    C_bo = body_from_orbit(ref, P0["gd_q_off"], P0["roll_axis"])
    inc = math.radians(E.case_value(case, "orbit.inc"))
    a = 6378137 + E.case_value(case, "orbit.alt") * 1e3
    n = math.sqrt(3.986004418e14 / a ** 3)
    out = {"case": case, "family": "mtq", "dispatched_law": (fam["algorithms"] or {}).get("mtq_pointing"), "C_body_orbit": C_bo.tolist(), "laws": []}
    tune = {k: v for k, v in scen["fsw"].items() if k in ("mtq_gain_p", "mtq_gain_d")}
    for law_id, law in LAWS.items():
        s = json.loads(json.dumps(scen))
        s["fsw"].setdefault("algorithms", {})["mtq_pointing"] = law
        for k in ("mtq_gain_p", "mtq_gain_d"):
            s["fsw"].pop(k, None)
        if law == out["dispatched_law"]:
            s["fsw"].update(tune)
        sp = PIPE / case / "floquet" / f"{law}.json"; sp.parent.mkdir(parents=True, exist_ok=True)
        sp.write_text(json.dumps(s))
        blob = sp.with_suffix(".bin")
        subprocess.run([str(E.BIN), "params", str(sp), "--case", str(MS / "cases" / f"{case}.csv"), "--out", str(blob)], cwd=MS, check=True,
                       capture_output=True, env={**__import__("os").environ, "ADCS_SIZED_DIR": str(pkg / "sized")})
        P = fswcfg.decode(blob.read_bytes())
        mu = monodromy(law_id, P, C_bo, inc, n)
        am = sorted(abs(mu), reverse=True)
        free = 1 if law_id == 7 else 0
        cert = max(am[free:]) < 1.0
        out["laws"].append({"law": law, "dispatched": law == out["dispatched_law"], "gains": {k: v for k, v in s["fsw"].items() if k.startswith("mtq_gain")} or "nominal",
                            "mu_abs": [float(x) for x in am], "max_mu": float(am[free]), "free_directions": free, "certified": bool(cert)})
    (PIPE / case / "floquet.json").write_text(json.dumps(out, indent=1))
    return out


if __name__ == "__main__":
    for c in sys.argv[1:] or ["ais_3u", "ais_img_3u"]:
        r = certify(c)
        if r:
            print(c, ", ".join(f"{x['law']} max|mu| {x['max_mu']:.4f}{' (dispatched)' if x['dispatched'] else ''}{' certified' if x['certified'] else ''}" for x in r["laws"]))
