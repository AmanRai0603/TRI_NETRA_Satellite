#!/usr/bin/env python3
"""The actuator catalogue node: bought reaction wheels and CMGs as their datasheets state them.

    python3 tools/catalogue.py            # (re)derive every file's modelling block, then docs/CATALOGUE.md
    python3 tools/catalogue.py --check    # say which file's block is not what its datasheet gives; write nothing

Each matlab_sils/data/catalogue/*.json (adcs-datasheet/1) holds the vendor's numbers in `datasheet`
(null = not stated, never guessed), where they came from (`source_url`, `verification`, `quote`),
and a `derived` block: the part parameters the engine flies, each computed by the catalogue's rule
and listed in `assumptions` when the datasheet does not give it. `selectable` is false when the
datasheet lacks a number the selection needs (momentum, torque, mass, steady power); such models
stay listed so the gap is visible. The select_rotor node (adcs-design, docs/NODES.md) reads only
selectable models.

The rule is the design's (docs/S7_INVENTORY.md S7.15): catalogue_datasheet_derive's method
(catalogue/catderive.pc), generated into the engine, which this tool asks (`adcs design derive`); the
tool keeps no relation of its own: it writes the engine's block and says, in words, what was assumed.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import json, subprocess, sys, tempfile, pathlib
from common import write_text, ROOT

CAT = ROOT / "matlab_sils" / "data" / "catalogue"
ENGINE = ROOT / "engine" / "target" / "release" / "adcs"


def engine_derive(cs):
    """Each datasheet's derivation by the design's method, through the engine (one call for all)."""
    with tempfile.TemporaryDirectory(prefix="catalogue_") as d:
        files = []
        for i, c in enumerate(cs):
            f = pathlib.Path(d) / f"{i:04d}.json"
            f.write_text(json.dumps(c))
            files.append(str(f))
        r = subprocess.run([str(ENGINE), "design", "derive", *files], capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f"catalogue: the engine's derive failed ({ENGINE}): {r.stderr.strip()[-2000:]}")
    return [json.loads(line) for line in r.stdout.splitlines()]


def derive(c, e=None):
    """The file c with its derived block and its assumptions, from the engine's derivation e (asked for when not given)."""
    e = e or engine_derive([c])[0]
    c["selectable"] = e["selectable"]
    c["missing"] = e["missing"]
    if not e["selectable"]:
        c["derived"], c["assumptions"] = {}, [f"not selectable: the datasheet does not state {', '.join(e['missing'])}"]
        return c
    R, A, a = e["rules"], e["assumed"], []
    if A["rpm"]:
        a.append(f"speed at the nominal momentum: {R['assumed_rpm']:g} rpm (not stated)")
    if A["tuna"]:
        a.append("volume: 0.2 L per unit (datasheet: '0.2U+ tuna can')")
    a += [f"rotor mass {R['rotor_share']:g} x unit mass, balance ISO 1940 G2.5 (imbalance model)",
          "friction: Coulomb 1e-5 sqrt(h / 0.01 Nms) N m, viscous 1e-8 N m s (not on datasheets)"]
    if A["peak"]:
        a.append("peak power: steady + torque x speed / 0.5 (not stated)")
    if c["type"] == "reaction_wheel":
        if A["supply"]:
            a.append("motor supply 5 V (not stated)")
        a += [f"motor: no-load speed {R['no_load']:g} x the top stated speed, the line through the stated torque there "
              f"(k_t = V / w_nl, R = k_t V / stall torque, V = the lowest stated supply)",
              f"breakaway friction {R['static_fraction']:g} x Coulomb, Stribeck speed {R['stribeck']:g} rad/s (Bialke 1998; not on datasheets)"]
    if c["type"] in ("cmg", "cmg_cluster"):
        a += ["gimbal rate: output torque / rotor momentum", f"rotor torque: {R['cmg_rotor_torque']:g} x output torque",
              "gimbal power inside the steady power (the datasheet's figure is the whole unit)",
              "as a VSCMG the rotor runs at half its momentum so the speed can move both ways (h_max = the datasheet's)"]
        if c["datasheet"].get("variable_speed") is not True:
            a.append("variable rotor speed not stated: the VSCMG use is an assumption")
    c["derived"], c["assumptions"] = e["derived"], a
    return c


def main():
    check = "--check" in sys.argv[1:]
    paths = sorted(CAT.glob("*.json"))
    cs = [json.loads(p.read_text()) for p in paths]
    bad = 0
    for p, c, e in zip(paths, cs, engine_derive(cs)):
        text = json.dumps(derive(c, e), indent=1) + "\n"
        if check:
            if p.read_text() != text:
                bad += 1
                print(f"  {p.relative_to(ROOT)}: not what its datasheet gives")
            continue
        write_text(p, text)
        print(f"  {c['part_number']:<42} {'selectable' if c['selectable'] else 'not selectable (' + ', '.join(c['missing']) + ')'}")
    if check:
        print(f"catalogue --check: {len(paths)} models, {bad} differ")
        return 1 if bad else 0
    print(f"catalogue: {len(paths)} models")
    subprocess.run([sys.executable, str(ROOT / "tools" / "nodes_doc.py")], check=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
