"""A2: the ledger's comparison sentence is built from the data's run counts and pass rates, and the
two-sample command (identical draws on the engine and the twin, a rank test) has its plumbing right.
Copyright (c) 2026 Agastya. All rights reserved."""
import json
import math
import pathlib
import tempfile
import unittest

import _path  # puts tools/ on the import path
import engine_campaigns as EC
import engine_twosample as T

_ = _path

MC = {"id": "t", "seed": 7, "runs": 3, "scenario": "nadir_hold_ais", "case": "ais_3u", "type": "montecarlo", "duration_s": 600.0,
      "dispersions": [{"kind": "solar_flux", "lo": 70.0, "hi": 220.0}, {"kind": "kp", "lo": 0.0, "hi": 6.0}]}


def row(cid, metric, mr, er, mp, ep):
    return {"campaign": cid, "metric": metric, "matlab_runs": mr, "engine_runs": er, "matlab_pass_rate": mp, "engine_pass_rate": ep}


class LedgerSentence(unittest.TestCase):
    def text(self, rows):
        return " ".join(EC.comparison_sentence(rows))

    def test_unequal_run_counts_cannot_say_the_same(self):
        t = self.text([row("a", "ape", 24, 1109, 1.0, 0.9)])
        self.assertNotIn("the same run count", t)
        self.assertIn("24", t)
        self.assertIn("1109", t)

    def test_equal_run_counts_say_the_same(self):
        self.assertIn("the same run count", self.text([row("a", "ape", 20, 20, 1.0, 1.0)]))

    def test_the_more_conservative_side_comes_from_the_pass_rates(self):
        self.assertIn("engine", self.text([row("a", "ape", 20, 20, 1.0, 0.5)]).split("lower")[0])
        less = self.text([row("a", "ape", 20, 20, 0.5, 1.0)])
        self.assertNotIn("the engine is therefore the more conservative", less.lower())
        self.assertIn("MATLAB twin", less)

    def test_no_judged_metric_claims_nothing_about_conservatism(self):
        t = self.text([row("a", "ape", 20, 20, None, None)])
        self.assertNotIn("conservative", t)

    def test_a_tie_claims_no_side(self):
        t = self.text([row("a", "ape", 20, 20, 1.0, 1.0)])
        self.assertNotIn("more conservative", t)


class MannWhitney(unittest.TestCase):
    def test_separated_samples(self):
        r = T.mann_whitney([1, 2, 3, 4, 5], [6, 7, 8, 9, 10])
        self.assertEqual(r["u"], 0.0)
        self.assertAlmostEqual(r["p"], math.erfc((12.5 - 0.5) / math.sqrt(25 * 11 / 12) / math.sqrt(2)), places=9)

    def test_identical_samples_do_not_reject(self):
        r = T.mann_whitney([1, 2, 3, 4], [1, 2, 3, 4])
        self.assertAlmostEqual(r["p"], 1.0, places=9)

    def test_ties_and_nonfinite_values_are_handled(self):
        r = T.mann_whitney([1, 1, 1, float("nan"), None], [1, 1, 1])
        self.assertEqual((r["n1"], r["n2"]), (3, 3))
        self.assertEqual(r["p"], 1.0)

    def test_an_empty_side_gives_no_test(self):
        self.assertIsNone(T.mann_whitney([], [1.0])["p"])


class Plumbing(unittest.TestCase):
    def test_the_draws_are_the_campaigns_draws_run_by_run(self):
        d = T.identical_draws(MC, 3)
        self.assertEqual([x["k"] for x in d], [1, 2, 3])
        for x in d:
            sets, _ = EC.draw(MC, x["k"])
            self.assertEqual([f"{s['name']}={json.dumps(s['value'])}" for s in x["set"]], sets)
            self.assertEqual(x["seed"], MC["seed"] + 7919 * x["k"])

    def test_the_two_sides_get_the_same_draw_file(self):
        with tempfile.TemporaryDirectory() as d:
            f = T.write_draws(MC, 3, pathlib.Path(d))
            self.assertEqual(json.loads(f.read_text())["draws"], T.identical_draws(MC, 3))

    def test_fewer_than_100_draws_is_a_smoke_test_not_a_result(self):
        res = T.compare(MC, [{"k": 1, "metrics": [{"id": "m", "value": 1.0}]}], [{"k": 1, "metrics": [{"id": "m", "value": 2.0}]}])
        self.assertFalse(res["complete"])
        self.assertIn("not a result", " ".join(T.section(res)))

    def test_a_complete_result_names_the_test_and_every_metric(self):
        eng = [{"k": k, "metrics": [{"id": "m", "value": float(k)}]} for k in range(1, 101)]
        twin = [{"k": k, "metrics": [{"id": "m", "value": float(k) + 0.5}]} for k in range(1, 101)]
        res = T.compare(MC, eng, twin)
        self.assertTrue(res["complete"])
        txt = "\n".join(T.section(res))
        self.assertIn("Mann-Whitney", txt)
        self.assertIn("| m |", txt)

    def test_the_ledger_carries_the_two_sample_section_when_the_result_is_filed(self):
        res = T.compare(MC, [], [])
        self.assertTrue(any("Two-sample" in x for x in T.section(res)))


if __name__ == "__main__":
    unittest.main()
