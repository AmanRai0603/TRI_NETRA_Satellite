"""The pointing budget (tools/pointing_budget.py): terms add in quadrature, the control part is
inferred from the flown loop, and a budget with a term nobody states never closes. Copyright (c) 2026 Agastya.
"""
import math
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import pointing_budget as B

_ = _path


class Budget(unittest.TestCase):
    def test_terms_add_in_quadrature_and_an_unstated_term_gives_no_total(self):
        self.assertAlmostEqual(B.rss([0.003, 0.004]), 0.005)
        self.assertIsNone(B.rss([0.003, None]))

    def test_the_control_part_is_the_flown_ape_less_the_knowledge(self):
        self.assertAlmostEqual(B.control_part(0.005, 0.003), 0.004)
        self.assertEqual(B.control_part(0.002, 0.003), 0.0, "never the root of a negative")

    def _row(self, case, alignment):
        s = {"case": "c", "product": "P"}
        m = {"ape_los_p9973": 0.006, "ake_los_p9973": 0.002, "budget_jitter": 3.6}   # jitter 3.6 arcsec = 0.001 deg
        with mock.patch.object(B, "case_values", return_value=case), mock.patch.object(B, "product_alignment", return_value=alignment):
            return B.row("x", s, m)

    def test_a_budget_with_every_term_stated_closes_or_not_against_req_ape(self):
        r = self._row({"req.ape": 0.01, "pointing.et": 0.002}, 0.003)
        self.assertAlmostEqual(r["total_deg"], math.sqrt(0.006**2 + 0.003**2 + 0.002**2 + 0.001**2))
        self.assertEqual(r["verdict"], "closes")
        self.assertEqual(self._row({"req.ape": 0.006, "pointing.et": 0.002}, 0.003)["verdict"], "does not close")

    def test_an_unstated_term_keeps_the_budget_incomplete_and_names_it(self):
        r = self._row({"req.ape": 0.01}, None)
        self.assertEqual(r["verdict"], "incomplete: gp_2, gp_3 not stated")
        self.assertIsNone(r["total_deg"])
        self.assertAlmostEqual(r["room_gp2_gp3_deg"], math.sqrt(0.01**2 - 0.006**2 - 0.001**2))


if __name__ == "__main__":
    unittest.main()
