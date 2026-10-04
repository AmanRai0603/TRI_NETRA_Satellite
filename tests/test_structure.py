"""The structure of the design (docs/RELEASE_PLAN.md P4): the group app's structure actions
(design/js/structure.js) on the whole seeded design, and the two checkers of its rules
(structure.js integrity() and tools/group.py) held to each other on folders broken on purpose.

Needs Node.js (the structure code is the browser's, run under Node on a folder on disk).
Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import unittest

import _path  # puts tools/ on the import path
import group
import seed_design
import tndb

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
NODE = shutil.which("node")


def js_integrity(d):
    r = subprocess.run([NODE, str(ROOT / "tests" / "js" / "integrity.mjs"), str(d)], capture_output=True, text=True, timeout=300)
    if r.returncode:
        raise AssertionError(r.stderr)
    return json.loads(r.stdout)


@unittest.skipUnless(NODE, "Node.js is needed")
class Structure(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.seed = pathlib.Path(cls.tmp.name) / "seed"
        seed_design.seed(cls.seed, sync=False)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def copy(self):
        d = pathlib.Path(tempfile.mkdtemp(dir=self.tmp.name))
        shutil.copytree(self.seed, d / "design")
        return d / "design"

    def test_every_structure_action_on_the_whole_design_and_no_node_file_broken(self):
        d = self.copy()
        r = subprocess.run([NODE, str(ROOT / "tests" / "js" / "structure.test.mjs"), str(d)], capture_output=True, text=True, timeout=600)
        sys.stdout.write(r.stdout[-3000:])
        self.assertEqual(r.returncode, 0, r.stdout[-4000:] + r.stderr[-2000:])
        files = sorted(d.rglob("*.tndb"))
        self.assertEqual([e for f in files for e in tndb.check(f)], [])
        self.assertEqual(group.check(d), [], "tools/group.py finds nothing broken")
        groups, _ = group.load(d)
        self.assertIn("act_new_wheel_model", groups["ctl"]["nodes"])
        self.assertNotIn("act_new_wheel_model", groups["act"]["nodes"])
        with sqlite3.connect(d / "nodes" / "act_new_wheel_model.node.tndb") as c:
            self.assertEqual(c.execute("SELECT group_id FROM node").fetchone()[0], "ctl")
        self.assertFalse(list(d.rglob("*.editing")) + list(d.rglob("*.crswap")), "no marker or swap file left")

    def test_the_seeded_design_holds_every_rule_in_both_checkers(self):
        self.assertEqual(group.check(self.seed), [])
        self.assertEqual(js_integrity(self.seed), [])

    def test_both_checkers_find_the_same_breakage(self):
        def node_says_other_group(d):
            with sqlite3.connect(d / "nodes" / "ct1_0.node.tndb") as c:
                c.execute("UPDATE node SET group_id = 'act'")

        def node_file_gone(d):
            (d / "nodes" / "ct1_0.node.tndb").unlink()

        def edge_from_nowhere(d):
            with sqlite3.connect(d / "structure" / "catalogue.group.tndb") as c:
                c.execute("INSERT INTO edge VALUES ('no_such_node', 'ct1_0', 'derivation', 'x')")

        def edge_kept_by_the_wrong_group(d):
            with sqlite3.connect(d / "structure" / "catalogue.group.tndb") as c:
                to = sqlite3.connect(d / "structure" / "act.group.tndb").execute("SELECT id FROM group_node LIMIT 1").fetchone()[0]
                c.execute("INSERT INTO edge VALUES ('ct1_0', ?, 'derivation', 'x')", (to,))

        def node_in_two_groups(d):
            with sqlite3.connect(d / "structure" / "act.group.tndb") as c:
                c.execute("INSERT INTO group_node SELECT * FROM group_node LIMIT 0")
                c.execute("INSERT INTO group_node VALUES ('ct1_0', 's', (SELECT id FROM stage LIMIT 1), '1', 'leaf', 'x', 'shell')")

        def stray_node_file(d):
            shutil.copy(d / "nodes" / "ct1_0.node.tndb", d / "nodes" / "zz_stray.node.tndb")

        for name, breakit in [(f.__name__, f) for f in (node_says_other_group, node_file_gone, edge_from_nowhere, edge_kept_by_the_wrong_group, node_in_two_groups, stray_node_file)]:
            with self.subTest(name):
                d = self.copy()
                breakit(d)
                py, js = group.check(d), js_integrity(d)
                self.assertTrue(py, f"{name}: tools/group.py finds it")
                self.assertTrue(js, f"{name}: structure.js finds it")

    def test_an_unfinished_action_is_reported(self):
        d = self.copy()
        (d / "structure" / "actions").mkdir()
        (d / "structure" / "actions" / "x.json").write_text(json.dumps({"id": "x", "done": False, "files": []}))
        self.assertTrue(any("not finished" in p for p in group.check(d)))


if __name__ == "__main__":
    unittest.main()
