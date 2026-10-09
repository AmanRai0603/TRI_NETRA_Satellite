"""Every row evaluated or shown as not computed, every KPI closure answered or blocked by name
(docs/RELEASE_PLAN.md P13, tools/evaluate.py), on a design folder seeded and carried over: the case
from its design database, a stated value in SI, a value computed by its pseudocode from another
row's, each not computed row with why, each closure with its answer or the row that blocks it.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import math
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
ROOT = pathlib.Path(__file__).resolve().parents[1]


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


@unittest.skipUnless(shutil.which("node"), "Node.js runs the pseudocode")
class WiredDesign(unittest.TestCase):
    """The regression copy, wired (S7.19, design/revisions_2_0.toml): the design's own stated values read, a method's
    inputs taken from the rows its wires name (an element of an array, one output of a module's function, a unit), the
    rows computed during a run said so, and the closure the wiring answers. Each expected value is worked out here from
    the stated inputs, not taken from the code under test."""

    @classmethod
    def setUpClass(cls):
        cls.r = {c: evaluate.evaluate(ROOT / "tests" / "regression", c) for c in ("ais_3u", "ais_img_3u")}
        cls.rows = {c: {x["id"]: x for x in r["rows"]} for c, r in cls.r.items()}

    def test_the_designs_stated_values_are_read(self):
        rows = self.rows["ais_3u"]
        self.assertEqual((rows["fsw_tune_mtq_wn"]["state"], rows["fsw_tune_mtq_wn"]["si"]), ("stated", 0.005))
        self.assertEqual(rows["dyn_initial_error_axis"]["si"], [1.0, 0.0, 0.0])                # a list, as stated
        self.assertEqual(rows["gf_0"]["state"], "stated")
        self.assertAlmostEqual(rows["gf_0"]["si"], 3e-3, delta=1e-18)                          # 3 mm, in its output's unit
        self.assertEqual(rows["m2_0"]["why"], "the case, orbit.alt")                           # the case's value wins

    def test_a_wired_method_takes_its_inputs_from_their_rows(self):
        rows = self.rows["ais_3u"]
        r = 6378137.0 + 550e3
        self.assertEqual(rows["env_case_orbit"]["state"], "computed")
        self.assertAlmostEqual(rows["env_case_orbit"]["si"], 2 * math.pi / math.sqrt(3.986004418e14 / r ** 3), delta=1e-9)
        self.assertEqual(rows["dyn_truth_plant"]["si"], [[0.0067, 0, 0], [0, 0.042, 0], [0, 0, 0.042]])
        self.assertAlmostEqual(rows["fsw_param_ss_dr_k2"]["si"], 0.5 * 0.042, delta=1e-18)  # J_zz, an element of the plant's inertia
        self.assertAlmostEqual(rows["fsw_param_detumble_exit"]["si"], 0.5 * math.pi / 180, delta=1e-18)
        self.assertEqual(rows["fsw_param_mtq_Kp"]["si"], [0.0067 * 0.005 ** 2, 0.042 * 0.005 ** 2, 0.042 * 0.005 ** 2])
        self.assertAlmostEqual(rows["fsw_param_jd0"]["si"], 2461406.75, delta=1e-6)           # 2027-01-01 06:00 UTC
        self.assertEqual(rows["fsw_param_mu"]["si"], 3.986004418e14)

    def test_a_row_computed_during_a_run_says_so(self):
        rows = self.rows["ais_3u"]
        for nid in ("l3_dist_row_01", "env_jb2008", "fsw_estimation", "fsw_param_m_max"):
            self.assertEqual(rows[nid]["state"], "not computed", nid)
            self.assertTrue(rows[nid]["why"].startswith("computed during a run"), (nid, rows[nid]["why"]))

    def test_the_settling_time_closure_is_answered(self):
        # gc_2 from the stated bandwidth 0.1 rad/s and damping 0.707: 4/(zeta wn) = 56.6 s, against the case's 20 s
        cl = {c["id"]: c for c in self.r["ais_img_3u"]["closures"]}["kpi_settling_time_after_a_slew_analysis"]
        self.assertEqual(cl["answer"], "fail")
        self.assertAlmostEqual(self.rows["ais_img_3u"]["gc_2"]["si"], 4 / (0.707 * 0.1), delta=1e-2)


if __name__ == "__main__":
    unittest.main()
