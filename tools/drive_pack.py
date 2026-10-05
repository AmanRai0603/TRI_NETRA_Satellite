#!/usr/bin/env python3
"""The Drive pack: what goes into the shared Drive folder of the design, ready to drop in.

    python3 tools/drive_pack.py [--out dist/trinetra-drive-pack]
    python3 tools/drive_pack.py --verify DIR [--pack MANIFEST.json] [--first-upload]

It holds:
  Apps/TRI-NETRA Files.html     open, change and save one design file (docs/FILES_IN_THE_BROWSER.md)
  Apps/TRI-NETRA Group.html     a group lead's app: structure and releases (docs/GROUP_APP.md)
  Apps/TRI-NETRA Node.html      a node author's app: every step, checks, preview, sign (docs/NODE_APP.md)
  Design/structure/             the 20 group files, seeded from the spec (tools/seed_design.py) and
                                carried over from the repository (tools/carry_over.py)
  Design/nodes/                 the node files (734 from the spec, and the rows added from the code)
  Design/design.tndb            the design database as seeded
  Guides/Engineering/           the technical audit, roadmap, decisions, test plans and gap register
  README.txt                    what to do with it
  MANIFEST.json                 every file above with its size and SHA-256, and the rules for what may
                                and may not be in the folder (--verify checks a folder against it)

--verify DIR checks a Drive folder (the folder Drive for desktop shows, or a downloaded copy) against the
manifest: every file present; apps and README unchanged; every design file still a valid design file
(people's edits are expected, so a changed design file is reported, not refused, unless --first-upload);
no file converted to a Google format; anything that should not be there named. Exit 1 on a problem.

CI builds it on every push (the trinetra-drive-pack artifact of the CI run). Copy its contents into
the Drive folder (with Drive for desktop the folder is on your computer), open an app from Apps/ in
Chrome or Edge, and pick the Design folder.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import fnmatch
import hashlib
import json
import pathlib
import shutil
import sys

import pages
import carry_over
import seed_design
import tndb
from common import ROOT, write_text

# the engineering documents a reader of the design needs beside the guides
ENGINEERING = ["TECHNICAL_ROADMAP.md", "DESIGN_DECISIONS.md", "TEST_BENCH_PLAN.md", "TEST_STRATEGY.md",
               "ORBIT_PROPAGATOR_AUDIT.md", "TOOL_COMPARISON.md", "ADCS_GAPS.md", "RELEASE_NOTES.md"]
# what may be in the folder besides the manifest's files (made by the apps, by Drive, or added on Drive)
ALLOWED_EXTRA = ["Design/releases/*", "Design/deliveries/*", "*.editing", "Guides/*.gdoc", "Guides/*.gsheet",
                 "*.gdoc", "*.gsheet", "desktop.ini", ".DS_Store", "Icon\r"]
# what must never be in it: Google conversions of design files, code, build outputs, results
FORBIDDEN = ["Design/*.gdoc", "Design/*.gsheet", "Apps/*.gdoc", "*.py", "*.rs", "*.c", "*.h", "*.m",
             "*.exe", "*.dll", "*.so", "*.whl", "store/*", "dist/*", "target/*"]

README = """TRI-NETRA design on Drive
=========================

Apps/     the three apps. Each is one file that runs offline from disk: open it in Chrome or Edge.
          TRI-NETRA Files.html  open one design file, change it, undo, save, see its history
          TRI-NETRA Group.html  a group lead's app: map, nodes, stages, people, contracts,
                                change requests, every structure change with its impact check;
                                progress, assemble, sign stages, seal releases, re-issue, import
          TRI-NETRA Node.html   a node author's app: fill a node step by step, live checks,
                                preview, mark ready; a second person signs it as checked
Design/   the design itself, as the apps read and write it
          structure/  one file per group (20)
          nodes/      one file per node (734 from the spec, and the rows added from the code);
                      each carries what the repository already says about it, and lists
                      what is still missing (the node app's Home, the group app's Progress)
          releases/   each group's sealed releases (made by the group app when a lead seals)

Every app has Help at the top: the guide for your role, the journey of a node, the glossary, and
the tour of the screen you are on.

To start: Drive for desktop shows this folder on your computer. Open Apps/TRI-NETRA Group.html,
choose "Open design folder", and pick the Design folder. Type your name the first time.

Built from {rev}. Nothing in it is sent anywhere; the apps only read and write these files.
"""


def build(out, rev="this checkout"):
    out = pathlib.Path(out)
    if out.exists():
        shutil.rmtree(out)
    (out / "Apps").mkdir(parents=True)
    built = pages.build(out / "_pages")
    names = {"files": "TRI-NETRA Files.html", "group": "TRI-NETRA Group.html", "node": "TRI-NETRA Node.html"}
    for k, (p, _n) in built.items():
        if k in names:                      # the app page and the test-app template run elsewhere, not from Drive
            shutil.move(str(p), out / "Apps" / names[k])
    shutil.rmtree(out / "_pages")
    seed_design.seed(out / "Design")
    carry_over.carry(out / "Design")
    files, errs = seed_design.check_tree(out / "Design")
    if errs:
        raise SystemExit("drive_pack: the seeded design does not check: " + "; ".join(errs[:5]))
    (out / "Guides" / "Engineering").mkdir(parents=True)
    for name in ENGINEERING:
        shutil.copy2(ROOT / "docs" / name, out / "Guides" / "Engineering" / name)
    write_text(out / "README.txt", README.format(rev=rev))
    write_text(out / "MANIFEST.json", json.dumps(manifest(out, rev), indent=1) + "\n")
    return out, len(files)


def _sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def manifest(out, rev):
    out = pathlib.Path(out)
    files = sorted(p for p in out.rglob("*") if p.is_file() and p.name != "MANIFEST.json")
    role = lambda r: "app" if r.startswith("Apps/") else "design" if r.startswith("Design/") else "guide" if r.startswith("Guides/") else "readme"
    return {"schema": "trinetra-drive-manifest/1", "built_from": rev,
            "files": [{"path": p.relative_to(out).as_posix(), "bytes": p.stat().st_size, "sha256": _sha(p),
                       "role": role(p.relative_to(out).as_posix())} for p in files],
            "allowed_extra": ALLOWED_EXTRA, "forbidden": FORBIDDEN}


def verify(folder, man, first_upload=False):
    """(problems, notes) of a Drive folder against a pack manifest."""
    folder = pathlib.Path(folder)
    problems, notes = [], []
    if not folder.is_dir():
        return [f"{folder} is not a folder"], notes
    want = {f["path"]: f for f in man["files"]}
    have = {p.relative_to(folder).as_posix(): p for p in folder.rglob("*") if p.is_file()}
    for path, f in want.items():
        p = have.get(path)
        if p is None:
            problems.append(f"missing: {path}")
        elif p.stat().st_size != f["bytes"] or _sha(p) != f["sha256"]:
            if f["role"] == "design" and not first_upload:
                notes.append(f"changed since the pack (people's edits): {path}")
            else:
                problems.append(f"changed: {path} (expected the pack's file, {f['bytes']} bytes)")
    match = lambda r, pats: any(fnmatch.fnmatch(r, pat) for pat in pats)
    for path in sorted(set(have) - set(want)):
        if path == "MANIFEST.json":
            continue
        if match(path, man["forbidden"]):
            problems.append(f"must not be here: {path}")
        elif " (1)." in path or "conflict" in path.lower():
            problems.append(f"Drive conflict copy, to be resolved: {path}")
        elif match(path, man["allowed_extra"]):
            notes.append(f"made on Drive or by the apps: {path}")
        else:
            problems.append(f"should not be here: {path}")
    for path, p in sorted(have.items()):
        if path.endswith(".tndb"):
            problems += [f"{path}: {e}" for e in tndb.check(p)]
    return problems, notes


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", default=str(ROOT / "dist" / "trinetra-drive-pack"))
    ap.add_argument("--rev", default="this checkout", help="what it was built from (CI passes the commit)")
    ap.add_argument("--verify", metavar="DIR", help="check a Drive folder against a pack's manifest")
    ap.add_argument("--pack", help="the manifest to check against (default: DIR/MANIFEST.json)")
    ap.add_argument("--first-upload", action="store_true", help="design files must equal the pack's (no edits yet)")
    a = ap.parse_args(argv)
    if a.verify:
        mf = pathlib.Path(a.pack) if a.pack else pathlib.Path(a.verify) / "MANIFEST.json"
        if not mf.is_file():
            raise SystemExit(f"drive_pack: no manifest at {mf} (upload MANIFEST.json with the pack, or pass --pack)")
        man = json.loads(mf.read_text(encoding="utf-8"))
        problems, notes = verify(a.verify, man, a.first_upload)
        for n in notes:
            print("  note: " + n)
        for p in problems:
            print("  PROBLEM: " + p)
        print(f"drive_pack verify: {len(man['files'])} files expected, {len(problems)} problem(s), {len(notes)} note(s)")
        return 1 if problems else 0
    out, n = build(a.out, a.rev)
    print(f"drive_pack: {out}: {len(list((out / 'Apps').iterdir()))} apps, {n} design files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
