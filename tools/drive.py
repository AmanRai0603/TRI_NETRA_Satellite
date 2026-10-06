#!/usr/bin/env python3
"""The shared drive, Trinetra Database (docs/OPERATING_2_0.md §15; docs/PLAN_2_0.md S4): pack what goes on it, and
check what is on it against what was packed.

    python3 tools/drive.py pack OUT [--zip]                 zip 1: the design converted, guides, START HERE, MANIFEST.json
    python3 tools/drive.py verify FOLDER [--first-upload]   the drive's folder (Drive for desktop) against its MANIFEST.json
    python3 tools/drive.py listing LISTING.json [--manifest MANIFEST.json]
                                                            a listing of the drive (name, size, MD5 for each file, as the
                                                            Drive connector reports them) against the manifest

What zip 1 holds (S4):
  groups/<group>/            the group file, its nodes and its baseline release 0.1 (tools/convert_2_0.py)
  cases/                     every case, scenario, campaign and trade as a case file
  readable/                  Groups, Nodes, Ports, Wires, Closures, BuiltIn, Conversion (CSV)
  design/ daily/ integration/ issues/ results/
                             empty but for a NOTE.txt that says what will be there (Drive may drop an empty folder)
  guides/                    the model, the operating model, the plan, the code's architecture; guides/engineering/
  START HERE.txt             what the drive is and how the upload is checked
  MANIFEST.json              every other file with its size, SHA-256 and MD5 (the MD5 is what Drive reports)

`verify` names every file that is missing, has another size or MD5, or is not in the manifest (old-1.0/, and what
Drive or the application add, are left out); with --first-upload it also refuses the 1.0.0 layout (Apps/,
Design/, Guides/ at the top). Exit 1 when anything is named.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import fnmatch
import hashlib
import json
import pathlib
import shutil
import sys
import tempfile
import zipfile

from common import ROOT, write_text

GUIDES = ["SYSTEM_MODEL.md", "OPERATING_2_0.md", "PLAN_2_0.md", "CODE_ARCHITECTURE.md"]
ENGINEERING = ["TECHNICAL_ROADMAP.md", "DESIGN_DECISIONS.md", "TEST_BENCH_PLAN.md", "TEST_STRATEGY.md", "ORBIT_PROPAGATOR_AUDIT.md",
               "TOOL_COMPARISON.md", "ADCS_GAPS.md", "RELEASE_NOTES.md", "RELEASE_NOTES_1_0_0_CORRECTION.md"]
NOTES = {
    "design": "The released design will be here: design.tnrel, the same under its version, NOTES.md, its flight images and every earlier "
              "release (docs/OPERATING_2_0.md §15). The first one is released at the switch-over (S10).",
    "daily": "A snapshot of today's design, one per day, kept by the system engineer's application.",
    "integration": "Previews of a group's release inside today's design, and the answers of the engineers whose values it moves.",
    "issues": "Every issue raised, one file each (<number>-<group>.tnissue), closed by the release that resolves it.",
    "results": "Results people chose to keep, each naming the design it was flown on.",
}
START_HERE = """TRI-NETRA · Trinetra Database
==============================

This drive is the design of the ADCS. The design lives here, in files, not in the code.

  groups/<group>/   each group's design: <group>.group.tndb, nodes/ (one file per node), releases/ (sealed releases)
  cases/            the shared cases, scenarios, campaigns and trades
  readable/         the design as spreadsheets (CSV), to read without the application
  guides/           how the design is modelled, how we work, the plan to 2.0.0, the engineering documents
  design/ daily/ integration/ issues/ results/   used from the switch-over on (each has a NOTE.txt)

What is here today (zip 1, {date}): 1.0.0's design converted into the 2.0.0 layout. Every group has a baseline
release 0.1, sealed by the conversion and marked "converted, not yet signed by a person": every node shows as
unproven until its group signs it. Today's design built from these releases gives 1.0.0's answers (results/PARITY_2_0.md
in the repository).

Nobody edits these files by hand. The application (zip 2, S9) opens them; until then they are read, not changed.

How the upload is checked: MANIFEST.json lists every file with its size and MD5. On Drive for desktop run
    python3 tools/drive.py verify "<this folder>" --first-upload
or the developer compares the drive with the manifest through the Drive connector, read-only.
Keep "convert uploads" off: a converted file is no longer the design file.

The programme manager's key fingerprint is written here when keys are registered (S9).
"""


def digests(p):
    h, m = hashlib.sha256(), hashlib.md5()
    with open(p, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
            m.update(chunk)
    return h.hexdigest(), m.hexdigest()


def manifest(root, what):
    files = []
    for f in sorted(p for p in root.rglob("*") if p.is_file() and p.name != "MANIFEST.json"):
        sha, md5 = digests(f)
        files.append({"path": f.relative_to(root).as_posix(), "size": f.stat().st_size, "sha256": sha, "md5": md5})
    return {"schema": "trinetra-drive-manifest/1", "what": what, "files": files,
            "folders": sorted({p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_dir()})}


def pack(out, zip_it=False, date="6 Oct 2026"):
    """Zip 1 into OUT/Trinetra Database (and OUT/Trinetra_Database_zip1.zip with --zip): (folder, manifest)."""
    import convert_2_0
    out = pathlib.Path(out)
    db = out / "Trinetra Database"
    if db.exists():
        shutil.rmtree(db)
    db.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as t:
        conv = pathlib.Path(t) / "conv"
        convert_2_0.convert(conv)
        bad = convert_2_0.check_folder(conv)
        if bad:
            raise SystemExit("drive: the conversion does not check: " + "; ".join(bad[:5]))
        shutil.copytree(conv, db)
    for d, note in NOTES.items():
        (db / d).mkdir(exist_ok=True)
        write_text(db / d / "NOTE.txt", note + "\n")
    (db / "guides" / "engineering").mkdir(parents=True)
    for name in GUIDES:
        shutil.copy2(ROOT / "docs" / name, db / "guides" / name)
    for name in ENGINEERING:
        if (ROOT / "docs" / name).is_file():
            shutil.copy2(ROOT / "docs" / name, db / "guides" / "engineering" / name)
    write_text(db / "START HERE.txt", START_HERE.format(date=date))
    m = manifest(db, f"zip 1 ({date}): 1.0.0's design converted into the 2.0.0 layout, baseline releases 0.1 unconfirmed")
    write_text(db / "MANIFEST.json", json.dumps(m, indent=1) + "\n")
    if zip_it:
        z = out / "Trinetra_Database_zip1.zip"
        with zipfile.ZipFile(z, "w", zipfile.ZIP_DEFLATED) as zf:
            for p in sorted(db.rglob("*")):
                arc = p.relative_to(db).as_posix() + ("/" if p.is_dir() else "")
                info = zipfile.ZipInfo(arc, date_time=(2026, 10, 6, 0, 0, 0))
                info.external_attr = (0o40755 << 16) | 0x10 if p.is_dir() else 0o644 << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                zf.writestr(info, b"" if p.is_dir() else p.read_bytes())
    return db, m


# What the drive may hold beside the manifest's files: made by Drive or the operating system, kept by people
# (old-1.0/ with 1.0.0's guides and sheets), or written by the application later.
IGNORED = ["old-1.0/*", "desktop.ini", "*/desktop.ini", ".DS_Store", "*/.DS_Store", "Icon\r", "*.gdoc", "*.gsheet", ".tmp.drive*"]


def compare(want, have, first_upload=False):
    """[why] for every difference between the manifest (want) and what is there (have: {path: (size, md5)})."""
    out = []
    for f in want["files"]:
        h = have.get(f["path"])
        if h is None:
            out.append(f"{f['path']}: missing")
        elif h[0] != f["size"]:
            out.append(f"{f['path']}: {h[0]} bytes, the manifest says {f['size']}")
        elif h[1] is not None and h[1] != f["md5"]:
            out.append(f"{f['path']}: its MD5 is not the manifest's (changed, or converted by Drive)")
    known = {f["path"] for f in want["files"]} | {"MANIFEST.json"}
    for p in sorted(set(have) - known):
        if any(fnmatch.fnmatch(p, g) for g in IGNORED):
            continue
        if first_upload or not p.startswith(("daily/", "integration/", "issues/", "results/", "design/")):
            out.append(f"{p}: not in the manifest" + (" (a second copy?)" if "(1)" in p else ""))
    if first_upload:
        tops = {p.split("/")[0] for p in have}
        out += [f"{t}/: 1.0.0's layout; delete it, or move it into old-1.0/" for t in ("Apps", "Design", "Guides") if t in tops]
    return out


def verify(folder, first_upload=False):
    folder = pathlib.Path(folder)
    mf = folder / "MANIFEST.json"
    if not mf.is_file():
        return [f"{folder}: no MANIFEST.json (upload zip 1's content, the manifest included)"]
    want = json.loads(mf.read_text())
    have = {}
    for p in folder.rglob("*"):
        if p.is_file():
            have[p.relative_to(folder).as_posix()] = (p.stat().st_size, digests(p)[1])
    return compare(want, have, first_upload)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    p = sp.add_parser("pack")
    p.add_argument("out")
    p.add_argument("--zip", action="store_true")
    p = sp.add_parser("verify")
    p.add_argument("folder")
    p.add_argument("--first-upload", action="store_true")
    p = sp.add_parser("listing")
    p.add_argument("listing")
    p.add_argument("--manifest", default=None)
    a = ap.parse_args(argv)
    if a.cmd == "pack":
        db, m = pack(a.out, a.zip)
        size = sum(f["size"] for f in m["files"])
        print(f"drive: packed {db}: {len(m['files'])} files, {size / 1e6:.1f} MB" + (f"; {pathlib.Path(a.out) / 'Trinetra_Database_zip1.zip'}" if a.zip else ""))
        return 0
    if a.cmd == "verify":
        bad = verify(a.folder, a.first_upload)
    else:
        want = json.loads(pathlib.Path(a.manifest or pathlib.Path(a.listing).with_name("MANIFEST.json")).read_text())
        have = {x["path"]: (int(x["size"]), x.get("md5")) for x in json.loads(pathlib.Path(a.listing).read_text())}
        bad = compare(want, have, first_upload=True)
    for x in bad:
        print("drive: " + x)
    print(f"drive: {'the drive is what was packed' if not bad else f'{len(bad)} difference(s)'}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
