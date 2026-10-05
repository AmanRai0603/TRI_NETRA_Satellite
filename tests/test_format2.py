"""Format 2 of the design files (docs/PLAN_2_0.md S2): every node file and every group file of 1.0.0
upgrades with nothing dropped, field by field, both by tools/tndb.py and by the one library
(`tndb upgrade`, engine/crates/trinetra-design), and the upgraded files check clean in both. A
format-1 file is today's file less the tables format 2 added, as 1.0.0 wrote it.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import os
import pathlib
import shutil
import sqlite3
import subprocess
import tempfile
import unittest

import _path  # puts tools/ on the import path
import carry_over
import seed_design
import tndb

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
TNDB = ROOT / "engine" / "target" / "release" / "tndb"
ADDED = {"node": ["block", "port", "loop", "closure", "key_signature"],
         "group": ["group_frame", "mount", "member_key", "key_signature"]}


def downgrade(path, kind):
    """The file as 1.0.0 wrote it: without format 2's tables, at version 1."""
    c = sqlite3.connect(path)
    for t in ADDED[kind]:
        c.execute(f'DROP TABLE "{t}"')
    c.execute('UPDATE meta SET "value" = ? WHERE "key" = ?', ("1", "format_version"))
    c.commit()
    c.execute("VACUUM")
    c.close()


def old_tables(dump, kind):
    return {t: v for t, v in dump["tables"].items() if t not in ADDED[kind]}


class Format2(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.d = pathlib.Path(cls.tmp.name) / "Design"
        seed_design.seed(cls.d, sync=False)
        carry_over.carry(cls.d)
        cls.files = [(f, "node") for f in sorted((cls.d / "nodes").glob("*.node.tndb"))] + \
                    [(f, "group") for f in sorted((cls.d / "structure").glob("*.group.tndb"))]
        cls.before = {}
        for f, kind in cls.files:
            cls.before[f.name] = old_tables(tndb.dump(f), kind)
            downgrade(f, kind)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_whole_design_is_there_to_upgrade(self):
        self.assertEqual(sum(k == "node" for _, k in self.files), 765)
        self.assertEqual(sum(k == "group" for _, k in self.files), 20)

    def test_every_file_upgrades_in_python_with_nothing_dropped(self):
        work = pathlib.Path(self.tmp.name) / "py"
        shutil.copytree(self.d, work)
        for f, kind in self.files:
            g = work / f.relative_to(self.d)
            tndb.open_file(g, kind).close()           # upgrades, keeping the original beside it
            self.assertTrue(g.with_name(g.name + ".v1.bak").is_file(), g.name)
            self.assertEqual(tndb.check(g), [], g.name)
            after = tndb.dump(g)
            self.assertEqual(old_tables(after, kind), self.before[f.name], f"{g.name}: a field changed in the upgrade")
            self.assertTrue(all(not after["tables"][t]["rows"] for t in ADDED[kind]), g.name)

    @unittest.skipUnless(TNDB.is_file() and os.access(TNDB, os.X_OK), "the library's command is not built (engine/target/release/tndb)")
    def test_every_file_upgrades_in_the_library_with_nothing_dropped(self):
        work = pathlib.Path(self.tmp.name) / "rs"
        shutil.copytree(self.d, work)
        paths = [str(work / f.relative_to(self.d)) for f, _ in self.files]
        r = subprocess.run([str(TNDB), "upgrade", *paths], capture_output=True, text=True, timeout=600)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn(f"{len(paths)} upgraded, 0 refused", r.stdout)
        r = subprocess.run([str(TNDB), "check", *paths], capture_output=True, text=True, timeout=600)
        self.assertEqual(r.returncode, 0, r.stderr)
        for f, kind in self.files:
            g = work / f.relative_to(self.d)
            self.assertEqual(old_tables(tndb.dump(g), kind), self.before[f.name], f"{g.name}: a field changed in the upgrade")
        # upgrading again changes nothing
        r = subprocess.run([str(TNDB), "upgrade", *paths], capture_output=True, text=True, timeout=600)
        self.assertIn(f"{len(paths)} file(s), 0 upgraded, 0 refused", r.stdout)


if __name__ == "__main__":
    unittest.main()
