#!/usr/bin/env python3
"""Export catalogue/{parts,products}/*.toml, scenarios/, campaigns/ and trades/ to matlab_sils/data/*/<id>.json.

MATLAB has no TOML reader, so the MATLAB SILS reads JSON (SPEC.md 10.8.2).
The export is generated, never edited: run this after changing a TOML file.

    python3 tools/export_catalogue.py            write every JSON from its TOML
    python3 tools/export_catalogue.py --check    say which JSON differs from its TOML (or has
                                                 no TOML), change nothing, exit 1 if any does
Copyright (c) 2026 Agastya. All rights reserved.
"""
import json, pathlib, sys, tomllib
from common import write_text

ROOT = pathlib.Path(__file__).resolve().parents[1]
KINDS = [("parts", None), ("products", None), ("algorithms", None), ("scenarios", ROOT / "scenarios"),
         ("campaigns", ROOT / "campaigns"), ("trades", ROOT / "trades"), ("modes", None), ("components", None)]


def outputs():
    """Every generated JSON file and the text its TOML gives it."""
    for kind, src in KINDS:
        out = ROOT / "matlab_sils" / "data" / kind
        for f in sorted((src or ROOT / "catalogue" / kind).glob("*.toml")):
            d = tomllib.loads(f.read_text())
            key = d.get("part_number") or d.get("id") or f.stem
            yield f, out / f"{key}.json", json.dumps(d, indent=1, sort_keys=True) + "\n"
    fam = ROOT / "catalogue" / "families.toml"
    yield fam, ROOT / "matlab_sils" / "data" / "families.json", json.dumps(tomllib.loads(fam.read_text()), indent=1, sort_keys=True) + "\n"


def check():
    """The JSON files that differ from their TOML, and the JSON files no TOML gives."""
    bad, made = [], set()
    for src, dst, text in outputs():
        made.add(dst)
        if not dst.exists() or dst.read_text() != text:
            bad.append(f"{dst.relative_to(ROOT)} differs from {src.relative_to(ROOT)}")
    for kind, _ in KINDS:
        for j in sorted((ROOT / "matlab_sils" / "data" / kind).glob("*.json")):
            if j not in made:
                bad.append(f"{j.relative_to(ROOT)} has no TOML source")
    return bad


if __name__ == "__main__":
    if sys.argv[1:] == ["--check"]:
        bad = check()
        for b in bad:
            print(b)
        print(f"export_catalogue --check: {len(bad)} file(s) drifted" if bad else "export_catalogue --check: every JSON is its TOML")
        sys.exit(1 if bad else 0)
    if sys.argv[1:]:
        sys.exit("usage: python3 tools/export_catalogue.py [--check]")
    for src, dst, text in outputs():
        dst.parent.mkdir(parents=True, exist_ok=True)
        write_text(dst, text)
        print("wrote", dst.relative_to(ROOT))
