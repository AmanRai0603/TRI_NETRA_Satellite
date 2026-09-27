#!/usr/bin/env python3
"""Runs VLEO_SIMULATOR's own seeding code over plan/tree.json and records the
node id each tree row receives.

    python3 tools/check_seed_with_vleo.py /path/to/VLEO_SIMULATOR          # check
    python3 tools/check_seed_with_vleo.py /path/to/VLEO_SIMULATOR --write  # rewrite plan/expected_node_ids.json

Why: the ADCS tree must be consumable by the same seeder VLEO uses. The
strongest check of that is to run the seeder's code, not a re-implementation
of it. This script copies tools/cd06_rows.py and tools/seed_helpers.py from a
VLEO_SIMULATOR checkout (read-only) into a temporary directory, patches only
the maps SPEC.md §5.7 names, and runs install(), LAYER3_SOURCE() and
install_edges() over this package's tree.

Without --write it fails if any recorded id differs, so a tree edit that
moves an id cannot pass unnoticed.
"""

import json
import os
import re
import shutil
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from plan_model import OWNER, LAYER3_OWNER  # noqa: E402  (one copy of the owner maps)

TONE = {"svc": "amber", "cpt": "slate", "msn": "violet", "sat": "slate", "sub": "green", "ver": "teal",
        "cas": "teal", "cat": "slate", "cmr": "amber", "ord": "green", "fac": "green",
        "std": "violet", "sup": "violet", "hrt": "violet", "rsk": "amber"}
SYS_ROOT = "sys_satellite_adcs"


def main():
    if len(sys.argv) < 2 or sys.argv[1].startswith("--"):
        print(__doc__)
        print("This tool has no --selftest: it checks the tree against VLEO_SIMULATOR's own code,"
              " so it needs a checkout path as its first argument.")
        return 2
    vleo = sys.argv[1]
    write = "--write" in sys.argv
    tree_path = os.path.join(ROOT, "plan", "tree.json")
    tree = json.load(open(tree_path, encoding="utf-8"))
    tmp = tempfile.mkdtemp(prefix="adcs-seedcheck-")
    try:
        os.makedirs(os.path.join(tmp, "tools"))
        for f in ("cd06_rows.py", "seed_helpers.py"):
            shutil.copy(os.path.join(vleo, "tools", f), os.path.join(tmp, "tools", f))
        src = open(os.path.join(tmp, "tools", "cd06_rows.py"), encoding="utf-8").read()
        l3g = {s["id"]: s["group"] for s in tree["layer3_shape"]}
        for name, val in (("OWNER", OWNER), ("TONE", TONE), ("LAYER3_GROUP", l3g), ("LAYER3_OWNER", LAYER3_OWNER)):
            src, n = re.subn(r"(?m)^%s = \{.*?\n\}\n" % name, "%s = %r\n" % (name, val), src, count=1, flags=re.S)
            if n != 1:
                raise SystemExit("could not find the %s map in cd06_rows.py — has VLEO changed?" % name)
        src, n = re.subn(r'(?m)^SYS_ROOT = ".*"$', 'SYS_ROOT = "%s"' % SYS_ROOT, src)
        if n != 1:
            raise SystemExit("could not find SYS_ROOT in cd06_rows.py")
        open(os.path.join(tmp, "tools", "cd06_rows.py"), "w", encoding="utf-8").write(src)
        sys.path.insert(0, os.path.join(tmp, "tools"))
        import seed_helpers as H
        import cd06_rows as R
        cd = R.Cd06(tree_path)
        R.install(cd, H.layer, H.S)
        if cd.nid["prg"] != R.SYS_ROOT:
            raise SystemExit("system root became %s, SYS_ROOT is %s" % (cd.nid["prg"], R.SYS_ROOT))
        shape = R.LAYER3_SOURCE(cd)          # asserts targets == leaf count, per layer
        by_id = {n["id"]: n for n in H.NODES}
        stat = R.install_edges(cd, H.LAYERS, by_id)
        ids = [n["id"] for n in H.NODES]
        if len(ids) != len(set(ids)):
            raise SystemExit("duplicate node ids")
        print("seeded %d rows in %d groups; %d crossings; %d subsystem layers; edges %s"
              % (len(H.NODES), len(H.LAYERS), len(cd.crossings), len(shape), stat))
        got = dict(sorted(cd.nid.items()))
        path = os.path.join(ROOT, "plan", "expected_node_ids.json")
        if write:
            with open(path, "w", encoding="utf-8") as f:
                json.dump(got, f, indent=0, sort_keys=True)
                f.write("\n")
            print("wrote %s (%d ids)" % (os.path.relpath(path, ROOT), len(got)))
            return 0
        want = json.load(open(path, encoding="utf-8")) if os.path.exists(path) else {}
        diff = {k: (want.get(k), got.get(k)) for k in set(want) | set(got) if want.get(k) != got.get(k)}
        for k, (a, b) in sorted(diff.items()):
            print("DIFF %s: recorded %s, seeder gives %s" % (k, a, b))
        print("%d id difference(s)" % len(diff))
        return 1 if diff else 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
