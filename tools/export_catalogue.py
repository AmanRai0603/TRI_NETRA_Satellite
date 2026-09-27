#!/usr/bin/env python3
"""Export catalogue/{parts,products}/*.toml, scenarios/ and campaigns/ to matlab_sils/data/*/<id>.json.

MATLAB has no TOML reader, so the MATLAB SILS reads JSON (SPEC.md 10.8.2).
The export is generated, never edited: run this after changing a TOML file.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import json, pathlib, tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]

def export(kind, src=None):
    out = ROOT / "matlab_sils" / "data" / kind
    out.mkdir(parents=True, exist_ok=True)
    for f in sorted((src or ROOT / "catalogue" / kind).glob("*.toml")):
        d = tomllib.loads(f.read_text())
        key = d.get("part_number") or d.get("id") or f.stem
        (out / f"{key}.json").write_text(json.dumps(d, indent=1, sort_keys=True) + "\n")
        print("wrote", (out / f"{key}.json").relative_to(ROOT))

if __name__ == "__main__":
    export("parts")
    export("products")
    export("scenarios", ROOT / "scenarios")
    export("campaigns", ROOT / "campaigns")
