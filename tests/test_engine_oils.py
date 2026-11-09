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


class LedgerNamesWhatItRejects(unittest.TestCase):
    """S11.16: the tool's exit said "each marked FAIL above" with no FAIL line, because a rejected pair was only
    in the page. Each rejected pair now prints a [FAIL] line, as many as the count it returns."""

    def test_every_rejected_pair_prints_a_fail_line_and_is_counted(self):
        import contextlib
        import io
        import json
        import pathlib
        import tempfile
        with tempfile.TemporaryDirectory() as t:
            t = pathlib.Path(t)
            m = lambda v: {"metrics": [{"id": "ape", "unit": "deg", "req": 10.0, "value": v, "pass": int(v <= 10.0)},
                                       {"id": "ake", "unit": "deg", "req": 5.0, "value": 1.0, "pass": 1}],
                           "dt_s": 0.1, "oils": {"overruns": 0}}
            for way, v in (("sils", 10.0), ("oils", 12.0)):
                d = t / "soft_oils" / "sc" / way
                d.mkdir(parents=True)
                (d / "manifest.json").write_text(json.dumps(m(v)))
            saved = O.ENG, O.OUT
            O.ENG, O.OUT = t, t / "results"
            try:
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    failed = O.oils_ledger()
            finally:
                O.ENG, O.OUT = saved
        fails = [x for x in out.getvalue().splitlines() if x.startswith("[FAIL]")]
        self.assertEqual(failed, 1)
        self.assertEqual(len(fails), 1)
        self.assertIn("sc ape", fails[0])
        self.assertIn("differs by", fails[0])


if __name__ == "__main__":
    unittest.main()
