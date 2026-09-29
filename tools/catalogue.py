#!/usr/bin/env python3
"""The actuator catalogue node: bought reaction wheels and CMGs as their datasheets state them.

    python3 tools/catalogue.py            # (re)derive every file's modelling block, then docs/CATALOGUE.md

Each matlab_sils/data/catalogue/*.json (adcs-datasheet/1) holds the vendor's numbers in `datasheet`
(null = not stated, never guessed), where they came from (`source_url`, `verification`, `quote`),
and a `derived` block: the part parameters the engine flies, each computed by a rule below and
listed in `assumptions` when the datasheet does not give it. `selectable` is false when the
datasheet lacks a number the selection needs (momentum, torque, mass, steady power); such models
stay listed so the gap is visible. The select_rotor node (adcs-design, docs/NODES.md) reads only
selectable models.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import json, math, pathlib, re, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CAT = ROOT / "matlab_sils" / "data" / "catalogue"
RPM = 2 * math.pi / 60
ASSUMED_RPM = 6000.0          # speed at the nominal momentum when a datasheet does not state it
ROTOR_FRACTION = 0.4          # rotor share of the unit mass (for the imbalance model)
CMG_ROTOR_TORQUE = 0.2        # rotor spin-up torque as a share of the output torque
NEED = ("h_mNms", "torque_mNm", "mass_g", "power_steady_W")


def volume_L(dims):
    """Box volume from 'a x b x c' [mm]; a 0.2U tuna can when that is all the datasheet says."""
    if not dims:
        return None, None
    n = [float(x) for x in re.findall(r"\d+(?:\.\d+)?", dims.split("(")[0])]
    if "x" in dims and len(n) >= 3:
        return n[0] * n[1] * n[2] * 1e-6, None
    if "0.2U" in dims:
        return 0.2, "volume: 0.2 L per unit (datasheet: '0.2U+ tuna can')"
    return None, None


def derive(c):
    d, a = c["datasheet"], []
    missing = [k for k in NEED if d.get(k) is None]
    c["selectable"] = not missing
    c["missing"] = missing
    if missing:
        c["derived"], c["assumptions"] = {}, [f"not selectable: the datasheet does not state {', '.join(missing)}"]
        return c
    h, tau = d["h_mNms"] * 1e-3, d["torque_mNm"] * 1e-3
    rpm = d.get("h_speed_rpm")
    if rpm is None:
        rpm = ASSUMED_RPM; a.append(f"speed at the nominal momentum: {ASSUMED_RPM:g} rpm (not stated)")
    w = rpm * RPM
    j = h / w
    m = d["mass_g"] * 1e-3
    vol, va = volume_L(d.get("dims_mm"))
    if va:
        a.append(va)
    mr = ROTOR_FRACTION * m
    r = math.sqrt(j / (0.9 * mr))
    us = mr * 2.5e-3 / w
    a += [f"rotor mass {ROTOR_FRACTION:g} x unit mass, balance ISO 1940 G2.5 (imbalance model)",
          "friction: Coulomb 1e-5 sqrt(h / 0.01 Nms) N m, viscous 1e-8 N m s (not on datasheets)"]
    pk = d.get("power_peak_W")
    if pk is None:
        pk = d["power_steady_W"] + tau * w / 0.5; a.append("peak power: steady + torque x speed / 0.5 (not stated)")
    x = {"h_max_Nms": h, "torque_max_Nm": tau, "speed_max_rad_s": w, "rotor_inertia_kgm2": j, "rotor_radius_m": r, "rotor_mass_kg": mr,
         "friction_coulomb_Nm": 1e-5 * math.sqrt(h / 0.01), "friction_viscous_Nms": 1e-8, "static_imbalance_kgm": us, "dynamic_imbalance_kgm2": us * r / 2,
         "power_steady_W": d["power_steady_W"], "power_peak_W": pk, "mass_kg": m, "volume_L": vol}
    if c["type"] in ("cmg", "cmg_cluster"):
        # the datasheet's torque is the unit's output (gyroscopic) torque: h x gimbal rate
        x.update({"rotor_momentum_Nms": h, "rotor_speed_rad_s": w, "rotor_torque_max_Nm": CMG_ROTOR_TORQUE * tau,
                  "gimbal_rate_max_rad_s": tau / h, "gimbal_power_W": 0.0,
                  "vscmg_rotor_momentum_Nms": h / 2})
        a += ["gimbal rate: output torque / rotor momentum", f"rotor torque: {CMG_ROTOR_TORQUE:g} x output torque",
              "gimbal power inside the steady power (the datasheet's figure is the whole unit)",
              "as a VSCMG the rotor runs at half its momentum so the speed can move both ways (h_max = the datasheet's)"]
        if d.get("variable_speed") is not True:
            a.append("variable rotor speed not stated: the VSCMG use is an assumption")
    c["derived"], c["assumptions"] = x, a
    return c


def main():
    n = 0
    for p in sorted(CAT.glob("*.json")):
        c = derive(json.loads(p.read_text()))
        p.write_text(json.dumps(c, indent=1) + "\n")
        n += 1
        print(f"  {c['part_number']:<42} {'selectable' if c['selectable'] else 'not selectable (' + ', '.join(c['missing']) + ')'}")
    print(f"catalogue: {n} models")
    subprocess.run([sys.executable, str(ROOT / "tools" / "nodes_doc.py")], check=True)


if __name__ == "__main__":
    main()
