"""The pointing budget (tools/pointing_budget.py; since S7.14 pnt's methods, generated into the engine and asked through
`adcs results budget`): terms add in quadrature, the control part is inferred from the flown loop, and a budget with a
term nobody states never closes. Each check runs a stored run's manifest through the engine, on a data folder whose
case and product state what the check needs. Copyright (c) 2026 Agastya.
"""
import csv
import io
import json
import math
import os
import pathlib
import shutil
import tempfile
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import pointing_budget as B

_ = _path
ROOT = pathlib.Path(__file__).resolve().parents[1]
HAVE_ENGINE = B.ENGINE.is_file() and os.access(B.ENGINE, os.X_OK)


@unittest.skipUnless(HAVE_ENGINE, "the engine is not built (engine/target/release/adcs)")
class Budget(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="pointing_budget_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)

    def _row(self, case, alignment, metrics=None):
        """The budget of a run of case ais_img_3u (its req.ape and pointing.et as `case` states them) on a product that
        states `alignment` [rad] (None: none), with the flown metrics given."""
        shutil.rmtree(self.tmp, ignore_errors=True)
        root = self.tmp / "root"
        (root / "cases").mkdir(parents=True)
        (root / "data" / "products").mkdir(parents=True)
        rows = list(csv.reader(io.StringIO((ROOT / "matlab_sils" / "cases" / "ais_img_3u.csv").read_text(encoding="utf-8"))))
        for r in rows:
            if len(r) > 4 and r[1] in ("req.ape", "pointing.et"):
                r[4] = "" if case.get(r[1]) is None else repr(case[r[1]])
        out = io.StringIO()
        csv.writer(out, lineterminator="\n").writerows(rows)
        (root / "cases" / "ais_img_3u.csv").write_text(out.getvalue(), encoding="utf-8")
        p = json.loads((ROOT / "matlab_sils" / "data" / "products" / "TRN-P-3U-IMG.json").read_text(encoding="utf-8"))
        if alignment is not None:
            p["payload_alignment_rad"] = alignment
        (root / "data" / "products" / "P.json").write_text(json.dumps(p), encoding="utf-8")
        run = self.tmp / "run"
        run.mkdir()
        m = metrics or {"ape_los_p9973": 0.006, "ake_los_p9973": 0.002, "budget_jitter": 3.6}    # jitter 3.6 arcsec = 0.001 deg
        (run / "manifest.json").write_text(json.dumps({"scenario": "x", "case": "ais_img_3u", "product": "P",
                                                       "metrics": [{"id": k, "value": v} for k, v in m.items()]}), encoding="utf-8")
        env = {"ADCS_ROOT": str(root), "TRINETRA_DESIGN": ""}
        with mock.patch.dict(os.environ, env):
            return B.row(run)

    def test_a_budget_with_every_term_stated_closes_or_not_against_req_ape(self):
        r = self._row({"req.ape": 0.01, "pointing.et": 0.002}, math.radians(0.003))
        self.assertAlmostEqual(r["terms_deg"]["gp_2"], 0.003)
        self.assertAlmostEqual(r["terms_deg"]["gp_4"], 0.001)
        self.assertAlmostEqual(r["total_deg"], math.sqrt(0.006**2 + 0.003**2 + 0.002**2 + 0.001**2))
        self.assertEqual(r["verdict"], "closes")
        self.assertEqual(self._row({"req.ape": 0.006, "pointing.et": 0.002}, math.radians(0.003))["verdict"], "does not close")

    def test_the_control_part_is_the_flown_ape_less_the_knowledge(self):
        self.assertAlmostEqual(self._row({"req.ape": 0.01}, None, {"ape_los_p9973": 0.005, "ake_los_p9973": 0.003})["terms_deg"]["gp_1"], 0.004)
        self.assertEqual(self._row({"req.ape": 0.01}, None, {"ape_los_p9973": 0.002, "ake_los_p9973": 0.003})["terms_deg"]["gp_1"], 0.0,
                         "never the root of a negative")

    def test_an_unstated_term_keeps_the_budget_incomplete_and_names_it(self):
        r = self._row({"req.ape": 0.01}, None)
        self.assertEqual(list(r), ["scenario", "case", "product", "flown_ape_deg", "terms_deg", "total_deg", "req_ape_deg", "room_gp2_gp3_deg", "verdict"])
        self.assertEqual(r["verdict"], "incomplete: gp_2, gp_3 not stated")
        self.assertIsNone(r["total_deg"])
        self.assertAlmostEqual(r["room_gp2_gp3_deg"], math.sqrt(0.01**2 - 0.006**2 - 0.001**2))
        self.assertEqual(self._row({"req.ape": 0.01}, None, {"ape_los_p9973": 0.006})["terms_deg"]["gp_4"], None, "no jitter metric")

    def test_the_tool_computes_no_term_itself(self):
        text = (ROOT / "tools" / "pointing_budget.py").read_text(encoding="utf-8")
        for gone in ("math.", "/3600", "def rss", "def control_part", "def product_alignment"):
            self.assertNotIn(gone, text, gone)


if __name__ == "__main__":
    unittest.main()
