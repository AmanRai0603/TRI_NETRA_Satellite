"""Every row evaluated or shown as not computed, every KPI closure answered or blocked by name
(docs/RELEASE_PLAN.md P13, tools/evaluate.py), on a design folder seeded and carried over: the case
from its design database, a stated value in SI, a value computed by its pseudocode from another
row's, each not computed row with why, each closure with its answer or the row that blocks it.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import shutil
import tempfile
import unittest

import _path  # puts tools/ on the import path
import carry_over
import design_inputs
import evaluate
import seed_design

_ = _path  # imported for its effect: tools/ on sys.path


@unittest.skipUnless(shutil.which("node"), "Node.js runs the pseudocode")
class Evaluate(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.d = pathlib.Path(cls.tmp.name) / "Design"
        seed_design.seed(cls.d, sync=False)
        carry_over.carry(cls.d)
        cls.r = {c: evaluate.evaluate(cls.d, c) for c in ("ais_3u", "ais_img_3u")}

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_every_row_has_a_value_or_a_reason(self):
        for case, r in self.r.items():
            ids = {x["id"] for x in r["rows"]}
            self.assertGreaterEqual(len(ids), 650, case)
            for x in r["rows"]:
                if x["state"] == "not computed":
                    self.assertTrue(x["why"], f"{case}: {x['id']} not computed with no reason")
                else:
                    self.assertIsNotNone(x["si"], f"{case}: {x['id']}")

    def test_a_stated_value_in_si_and_a_value_computed_from_it(self):
        rows = {x["id"]: x for x in self.r["ais_3u"]["rows"]}
        self.assertEqual(rows["p1k_0"]["state"], "stated")          # req.ape, 10 deg
        self.assertAlmostEqual(rows["p1k_0"]["si"], 10 * evaluate.DEG)
        self.assertEqual(rows["m2_4"]["state"], "computed")        # the orbit radius from the altitude
        self.assertAlmostEqual(rows["m2_4"]["si"], 6378137 + 550e3, delta=1.0)

    def test_every_closure_answers_or_names_what_blocks_it(self):
        for case, r in self.r.items():
            self.assertEqual(len(r["closures"]), 38, case)          # 22 verified + 16 analysis
            for c in r["closures"]:
                self.assertIn(c["answer"], ("pass", "fail", "blocked"))
                self.assertTrue(c["why"], f"{case}: {c['id']}")
                if c["answer"] == "blocked":
                    self.assertRegex(c["why"], r"\b[a-z0-9]+_[a-z0-9_]+\b|\bp\d", f"{case}: {c['id']} names a row")

    def test_the_database_holds_the_data_folders_inputs(self):
        self.assertEqual(design_inputs.differences(self.d / "design.tndb"), [])


if __name__ == "__main__":
    unittest.main()
