"""The parity gate's campaigns part (tools/parity_2_0.py, DR-vv-003): bit-exact with the record's fingerprint, within the
per-metric tolerance of results/golden.json with another, a verdict always equal.
Copyright (c) 2026 Agastya. All rights reserved."""
import copy
import unittest

import _path  # puts tools/ on the import path
import parity_2_0 as p

_ = _path

FP = {"rustc": "r", "engine": "e", "fsw": "f", "machine": "x86_64"}
WANT = [{"id": "a", "kind": "ape_los", "value": 0.0064, "pass": 1},
        {"id": "t", "kind": "time_to_threshold", "value": 81.6, "pass": None}]
TOL = 0.05


def moved(rel=1e-12, ok=1):
    g = copy.deepcopy(WANT)
    g[0]["value"] *= 1 + rel
    g[0]["pass"] = ok
    return g


class Judge(unittest.TestCase):
    def test_other_fingerprint_small_change_is_within_tolerance(self):
        for rec in ("unrecorded", dict(FP, machine="aarch64")):
            st, why, rel = p.judge(moved(), WANT, FP, rec, TOL)
            self.assertEqual((st, why), ("tolerance", None))
            self.assertGreater(rel, 0)

    def test_verdict_flip_fails_and_is_named(self):
        st, why, _ = p.judge(moved(0, 0), WANT, FP, "unrecorded", TOL)
        self.assertIsNone(st)
        self.assertIn("a: verdict", why)

    def test_three_percent_on_another_fingerprint_passes(self):
        st, why, rel = p.judge(moved(0.03), WANT, FP, "unrecorded", TOL)
        self.assertEqual((st, why), ("tolerance", None))
        self.assertAlmostEqual(rel, 0.03 / 1.03)

    def test_the_tolerance_is_the_golden_table_field(self):
        self.assertEqual(p.campaign_tolerance(), 0.05)

    def test_beyond_tolerance_fails_and_is_named(self):
        st, why, _ = p.judge(moved(0.06), WANT, FP, "unrecorded", TOL)
        self.assertIsNone(st)
        self.assertIn("a: outside relative tolerance", why)

    def test_same_fingerprint_any_change_fails(self):
        st, why, _ = p.judge(moved(), WANT, FP, FP, TOL)
        self.assertIsNone(st)
        self.assertEqual(why, "its metrics differ")

    def test_equal_is_bit_exact(self):
        self.assertEqual(p.judge(copy.deepcopy(WANT), WANT, FP, "unrecorded", TOL)[0], "exact")

    def test_existing_records_are_unrecorded(self):
        self.assertEqual(p.campaign_fingerprint(), "unrecorded")


if __name__ == "__main__":
    unittest.main()
