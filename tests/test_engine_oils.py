"""Soft OILS acceptance: SILS against OILS at the numeric tolerance of docs/TEST_BENCH_PLAN.md section 5,
not on verdict agreement alone. Nothing here runs the engine. Copyright (c) 2026 Agastya. All rights reserved."""
import unittest

import _path  # puts tools/ on the import path
import engine_oils as O

_ = _path


class PairVerdict(unittest.TestCase):
    def test_a_flipped_verdict_on_noise_at_the_edge_is_accepted(self):
        ok, why = O.pair_verdict(0.010, 0.0100001, 0.010, 1, 0)
        self.assertTrue(ok, why)
        self.assertIn("knife-edge", why)

    def test_a_30_percent_difference_fails_even_with_the_same_verdict(self):
        ok, _ = O.pair_verdict(0.010, 0.013, 0.010, 0, 0)
        self.assertFalse(ok)

    def test_a_flip_away_from_the_requirement_fails(self):
        ok, _ = O.pair_verdict(0.5, 0.52, 1.0, 1, 0)
        self.assertFalse(ok)

    def test_equal_values_and_verdicts_are_accepted(self):
        self.assertTrue(O.pair_verdict(0.2, 0.2, 1.0, 1, 1)[0])

    def test_a_metric_only_one_side_has_is_judged_on_the_verdict(self):
        self.assertTrue(O.pair_verdict(None, 3.0, 0.0, 1, 1)[0])
        self.assertFalse(O.pair_verdict(None, 3.0, 0.0, 1, 0)[0])


if __name__ == "__main__":
    unittest.main()
