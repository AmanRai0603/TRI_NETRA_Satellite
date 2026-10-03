"""The carry-over (docs/RELEASE_PLAN.md P8): everything the repository already says, carried into the
seeded node files by tools/carry_over.py, each item marked with its origin, every gap listed with
its owner team; the folder afterwards holds every rule; the carried pseudocode compiles and runs in
the node app's own code; and carrying again changes nothing an author wrote.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import shutil
import sqlite3
import subprocess
import tempfile
import unittest

import _path  # puts tools/ on the import path
import carry_over
import group
import seed_design
import tndb

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
NODE = shutil.which("node")


def content(d, nid):
    with sqlite3.connect(pathlib.Path(d) / "nodes" / f"{nid}.node.tndb") as c:
        return {(f"{s}.{f}" if f else s): (v, o) for s, f, v, o in c.execute("SELECT section, field, value, origin FROM content")}


class Carry(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.d = pathlib.Path(cls.tmp.name) / "design"
        seed_design.seed(cls.d, sync=False)
        # an author wrote one field before the carry: it must stay theirs
        with sqlite3.connect(cls.d / "nodes" / "m2_4.node.tndb") as c:
            c.execute("INSERT INTO content VALUES ('relation', 'why', 'Asha''s own words.', 'typed by Asha')")
        cls.report = carry_over.carry(cls.d)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_map_names_only_what_exists(self):
        self.assertEqual(carry_over.problems(), [])

    def test_every_file_and_rule_holds_afterwards(self):
        self.assertEqual([e for f in sorted(self.d.rglob("*.tndb")) for e in tndb.check(f)], [])
        self.assertEqual(group.check(self.d), [])
        n = sum(r["nodes"] for r in self.report.values())
        self.assertEqual(n, 734 + len(carry_over.Plan().carry["add"]))

    def test_items_are_marked_with_their_origin(self):
        c = content(self.d, "m2_4")
        self.assertEqual(c["identity.question"][1], "carried:spec/plan/seed_content.toml")
        self.assertTrue(c["code.pseudocode"][0].startswith("## Earth's equatorial radius"), c["code.pseudocode"][0][:80])
        self.assertIn("adcs_physics::orbit::radius", c["code.rust"][0])
        self.assertEqual(c["relation.why"], ("Asha's own words.", "typed by Asha"))
        g = content(self.d, "gd_0")
        self.assertIn("fn gd_0(", g["code.pseudocode"][0])      # the node's answer, named as the node names it
        p = carry_over.Plan()
        req = next(k["requirement"] for k in p.kpis if k["requirement"] not in p.seed)       # a requirement the spec does not seed
        self.assertEqual(content(self.d, req)["requirement.sense"][1], "carried:spec/plan/kpis.toml")

    def test_internal_rows_are_named_and_rows_added_from_the_code(self):
        c = content(self.d, "l3_sens_row_01")
        self.assertEqual(c["identity.label"][0], "Gyro angle random walk")
        groups, _ = group.load(self.d)
        self.assertEqual(groups["sens"]["nodes"]["l3_sens_row_01"]["label"], "Gyro angle random walk")
        a = content(self.d, "gdn_guidance")
        self.assertIn("fn guidance(", a["code.pseudocode"][0])
        self.assertEqual(a["identity.form_kind"][0], "computed")
        self.assertIn("act_cmg_steering", groups["act"]["nodes"])
        self.assertEqual(groups["act"]["nodes"]["act_cmg_steering"]["stage"], "cmg")

    def test_every_gap_is_listed_with_its_owner_team(self):
        for f in sorted((self.d / "nodes").glob("*.node.tndb")):
            c = content(self.d, f.name.split(".")[0])
            self.assertIn("status.gaps", c, f.name)
            self.assertTrue(c["status.owner_team"][0], f.name)
        gaps = json.loads(content(self.d, "l3_dist_row_01")["status.gaps"][0])
        self.assertIn("its kind is not chosen (declared or computed)", gaps)
        self.assertNotIn("the row is not named yet", gaps)

    def test_carrying_again_adds_nothing(self):
        before = content(self.d, "gd_0")
        rep = carry_over.carry(self.d)
        self.assertEqual(sum(r["items"] for r in rep.values()), 0)
        after = content(self.d, "gd_0")
        self.assertEqual({k: v for k, v in before.items() if not k.startswith("status.")}, {k: v for k, v in after.items() if not k.startswith("status.")})
        self.assertEqual(group.check(self.d), [])

    @unittest.skipUnless(NODE, "Node.js is needed to run the node app's code")
    def test_the_node_app_reads_every_node_and_runs_its_pseudocode(self):
        r = subprocess.run([NODE, str(ROOT / "tests" / "js" / "carry.test.mjs"), str(self.d)], capture_output=True, text=True, timeout=600)
        self.assertEqual(r.returncode, 0, r.stdout[-3000:] + r.stderr[-2000:])
        s = json.loads(r.stdout)
        self.assertEqual(s["checked"], s["nodes"])
        self.assertEqual(s["compiled"], s["pseudocode"])
        self.assertGreaterEqual(s["pseudocode"], 48)
        self.assertEqual(s["reproduced"], s["tried"])
        self.assertGreaterEqual(s["tried"], 4)


if __name__ == "__main__":
    unittest.main()
