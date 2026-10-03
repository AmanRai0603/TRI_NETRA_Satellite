#!/usr/bin/env python3
"""The Drive pack: what goes into the shared Drive folder of the design, ready to drop in.

    python3 tools/drive_pack.py [--out dist/trinetra-drive-pack]

It holds:
  Apps/TRI-NETRA Files.html     open, change and save one design file (docs/FILES_IN_THE_BROWSER.md)
  Apps/TRI-NETRA Group.html     a group lead's app: structure (docs/GROUP_APP.md)
  Design/structure/             the 20 group files, seeded from the spec (tools/seed_design.py)
  Design/nodes/                 the 734 node files
  Design/design.tndb            the design database as seeded
  README.txt                    what to do with it

CI builds it on every push (the trinetra-drive-pack artifact of the CI run). Copy its contents into
the Drive folder (with Drive for desktop the folder is on your computer), open an app from Apps/ in
Chrome or Edge, and pick the Design folder.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import pathlib
import shutil
import sys

import pages
import seed_design
from common import ROOT, write_text

README = """TRI-NETRA design on Drive
=========================

Apps/     the two apps. Each is one file that runs offline from disk: open it in Chrome or Edge.
          TRI-NETRA Files.html  open one design file, change it, undo, save, see its history
          TRI-NETRA Group.html  a group lead's app: map, nodes, stages, people, contracts,
                                change requests, every structure change with its impact check
Design/   the design itself, as the apps read and write it
          structure/  one file per group (20)
          nodes/      one file per node (734)

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
    names = {"files": "TRI-NETRA Files.html", "group": "TRI-NETRA Group.html"}
    for k, (p, _n) in built.items():
        shutil.move(str(p), out / "Apps" / names.get(k, f"{k}.html"))
    shutil.rmtree(out / "_pages")
    seed_design.seed(out / "Design")
    files, errs = seed_design.check_tree(out / "Design")
    if errs:
        raise SystemExit("drive_pack: the seeded design does not check: " + "; ".join(errs[:5]))
    write_text(out / "README.txt", README.format(rev=rev))
    return out, len(files)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", default=str(ROOT / "dist" / "trinetra-drive-pack"))
    ap.add_argument("--rev", default="this checkout", help="what it was built from (CI passes the commit)")
    a = ap.parse_args(argv)
    out, n = build(a.out, a.rev)
    print(f"drive_pack: {out}: 2 apps, {n} design files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
