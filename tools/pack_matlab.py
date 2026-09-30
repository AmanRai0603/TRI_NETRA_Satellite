#!/usr/bin/env python3
"""Build dist/TRINETRA_ADCS_SILS_matlab_<version>.zip — the downloadable MATLAB SILS.

Contents: matlab_sils/ (engine, POP, cases, data, examples, tests, tools),
an empty store/, the architecture plan, NOTICE and MANIFEST.sha256.
Deterministic: files sorted, fixed timestamps. Run it from the repository.
Copyright (c) 2026 Agastya. All rights reserved.
"""
import hashlib, pathlib, re, zipfile
from common import atomic_path

ROOT = pathlib.Path(__file__).resolve().parents[1]
SRC = ROOT / "matlab_sils"
ver = re.search(r"'(asils-[0-9.]+)", (SRC / "+asils" / "version.m").read_text()).group(1)
name = f"TRINETRA_ADCS_SILS_matlab_{ver}"
out = ROOT / "dist" / f"{name}.zip"
out.parent.mkdir(exist_ok=True)

skip_dirs = {"store"}
files = []
for p in sorted(SRC.rglob("*")):
    rel = p.relative_to(SRC)
    if p.is_dir() or rel.parts[0] in skip_dirs or p.name.endswith((".log", ".asv")) or "__pycache__" in rel.parts:
        continue
    files.append((p, pathlib.PurePosixPath(name) / rel.as_posix()))
extra = {
    "docs/ARCHITECTURE_PLAN.md": ROOT / "docs" / "ARCHITECTURE_PLAN.md",
    "docs/RESULTS.md": ROOT / "docs" / "RESULTS.md",
    "docs/SELECTION.md": ROOT / "docs" / "SELECTION.md",
    "docs/OILS_HILS.md": ROOT / "docs" / "OILS_HILS.md",
    "docs/SOLUTION_PIPELINE.md": ROOT / "docs" / "SOLUTION_PIPELINE.md",
    "docs/SOLUTIONS.md": ROOT / "docs" / "SOLUTIONS.md",
    "docs/COMPONENTS.md": ROOT / "docs" / "COMPONENTS.md",
    "store/README.md": None,
}
man = []
FIXED = (2026, 9, 27, 0, 0, 0)
# the zip is written beside its name and renamed into place when whole
with atomic_path(out) as part, zipfile.ZipFile(part, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for src, arc in files:
        data = src.read_bytes()
        zi = zipfile.ZipInfo(str(arc), FIXED); zi.compress_type = zipfile.ZIP_DEFLATED; zi.external_attr = 0o644 << 16
        z.writestr(zi, data); man.append(f"{hashlib.sha256(data).hexdigest()}  {arc}")
    for arc, src in extra.items():
        data = src.read_bytes() if src and src.exists() else (
            b"Your results are filed here: results/<scenario or campaign>/, trades/<trade>/:\n"
            b"channels.csv, manifest.json, figures, rec.mat, result.html; trade.json, trade.png.\n")
        a = f"{name}/{arc}"
        zi = zipfile.ZipInfo(a, FIXED); zi.compress_type = zipfile.ZIP_DEFLATED; zi.external_attr = 0o644 << 16
        z.writestr(zi, data); man.append(f"{hashlib.sha256(data).hexdigest()}  {a}")
    zi = zipfile.ZipInfo(f"{name}/MANIFEST.sha256", FIXED); zi.compress_type = zipfile.ZIP_DEFLATED
    z.writestr(zi, "\n".join(man) + "\n")
print(out, f"{out.stat().st_size/1e6:.1f} MB, {len(man)} files")
