#!/usr/bin/env python3
"""Build dist/TRINETRA_ADCS_flight_engine_<version>.zip -- the flight software and the Rust engine.

Contents (repository layout kept, so it builds as unpacked):
  fsw/            pseudocode, embedded C flight software, params.toml, tests, twin_map.toml
  fsw-rs/         the Rust flight software (no_std; C ABI feature)
  engine/         the Rust SILS engine (adcs-pop = the POP port, adcs-sim-core, adcs-fsw-abi, adcs-sim, adcs-cli)
  fsw/targets/    the virtual OBC (adcs-link/1; POSIX process, QEMU Cortex-M4F firmware)
  matlab_sils/pop/.../de440s.bsp   the DE440 kernel adcs-pop reads
  tools/          pipeline.py (design loop), engine.py (orchestration), rescore.py, vv_report.py + templates/, gen_fsw_params.py
  matlab_sils/data, matlab_sils/cases   the cases, scenarios, products and parts the engine reads
  docs/ (LANGUAGES, DESIGN_LOOP, SOFT_OILS, VIRTUAL_OBC, OILS_HILS, figures/), results ledgers, the V&V report PDF,
  NOTICE.md, MANIFEST.sha256
Build: python3 tools/engine.py build (needs gcc, make, cargo). Deterministic zip.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import hashlib, pathlib, re, zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
ver = re.search(r'^version = "([0-9.]+)"', (ROOT / "engine" / "Cargo.toml").read_text(), re.M).group(1)
name = f"TRINETRA_ADCS_flight_engine_{ver}"
out = ROOT / "dist" / f"{name}.zip"
out.parent.mkdir(exist_ok=True)

trees = ["fsw", "fsw-rs", "engine", "matlab_sils/data", "matlab_sils/cases", "docs/figures", "tools/figures", "tools/templates"]
singles = ["tools/engine.py", "tools/pipeline.py", "tools/rescore.py", "tools/catalogue.py", "tools/nodes_doc.py", "docs/NODES.md", "docs/CATALOGUE.md", "tools/vv_report.py", "tools/gen_fsw_params.py", "tools/fswcfg.py", "docs/LANGUAGES.md",
           "docs/VIRTUAL_OBC.md", "docs/DESIGN_LOOP.md", "docs/SOFT_OILS.md", "docs/OILS_HILS.md", "results/ENGINE_PARITY.md",
           "results/ENGINE_CAMPAIGNS.md", "results/SOFT_OILS.md", "results/DESIGN_ais_3u.md", "results/DESIGN_ais_img_3u.md",
           "dist/TRINETRA_ADCS_VV_report.pdf",
           "results/VIRTUAL_OBC.md", "results/ENGINE_SOLUTIONS.md", "NOTICE.md", "README.md",
           "matlab_sils/pop/03_frames_time/ephemeris/data/de440s.bsp"]
skip = {"target", "build", "__pycache__"}
files = []
for t in trees:
    for p in sorted((ROOT / t).rglob("*")):
        rel = p.relative_to(ROOT)
        if p.is_dir() or skip & set(rel.parts) or p.suffix in (".log", ".asv", ".o", ".a"):
            continue
        files.append(rel)
files += [pathlib.Path(s) for s in singles if (ROOT / s).exists()]
FIXED = (2026, 9, 28, 0, 0, 0)
man = []
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for rel in files:
        data = (ROOT / rel).read_bytes()
        arc = f"{name}/{rel.as_posix()}"
        zi = zipfile.ZipInfo(arc, FIXED); zi.compress_type = zipfile.ZIP_DEFLATED; zi.external_attr = 0o644 << 16
        z.writestr(zi, data); man.append(f"{hashlib.sha256(data).hexdigest()}  {arc}")
    zi = zipfile.ZipInfo(f"{name}/MANIFEST.sha256", FIXED); zi.compress_type = zipfile.ZIP_DEFLATED
    z.writestr(zi, "\n".join(man) + "\n")
print(out, f"{out.stat().st_size/1e6:.1f} MB, {len(man)} files")
