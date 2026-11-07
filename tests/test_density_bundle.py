"""E6 (S11.11): the density row m3_3 answers with the operational DTM2020 at the bundle's low, mean and high solar
activity, its answer the high member, and gd_1's worst aerodynamic torque is taken at it. The relations are the design's
(env's atmosbundle, asked of the engine); this file holds the cases they are held to.
Copyright (c) 2026 Agastya. All rights reserved."""
import json
import math
import pathlib
import sqlite3
import unittest

import _path  # puts tools/ on the import path
from design_call import call

_ = _path
ROOT = pathlib.Path(__file__).resolve().parent.parent
YR = 31557600.0                      # SI: the design's values are metres and seconds
POP = ROOT / "engine" / "crates" / "adcs-pop" / "tests" / "data" / "atmos_drag.json"
H550, T_EPOCH = 550e3, 27.5 * YR
LOW, MEAN, HIGH = 0, 1, 2            # atmosbundle's SolarActivity, in its order


def member(level, h=H550):
    return call("atmosbundle::density_member", h, T_EPOCH, level)[0]


def vallado_550():
    # Vallado (2013) table 8-4, the static table m3_3 answered with: 6.967e-13 kg/m^3 at 500 km, scale height 63.822 km
    return 6.967e-13 * math.exp(-50.0 / 63.822)


class Bundle(unittest.TestCase):
    def test_a_member_is_the_dtm2020_value_at_its_drivers(self):
        # m3_3 at 550 km equals the operational DTM2020 at the member's F10.7 (250 high, 70 low) to the evaluation's own precision
        # (the same model, held to the Octave POP's reference vectors at 1e-12, adcs-pop/tests/atmos_drag.rs; 1e-9 is the tolerance here)
        place = (0.0, 0.0, 14.0)
        doy = 1.0 + 365.0 * 0.5          # the epoch 27.5 years: half a year
        for level, f107 in ((LOW, 70.0), (MEAN, 130.0), (HIGH, 250.0)):
            want = call("atmosbundle::dtm_density", 550.0, *place, doy, f107, f107, 2.0)[0]
            self.assertAlmostEqual(member(level) / want, 1.0, delta=1e-9, msg=f"F10.7 = {f107}")

    def test_the_dtm_wrapper_reproduces_the_pops_reference_rows(self):
        # the Octave POP's published-model reference (the rows adcs-pop's tests hold DTM2020 to): the wrapper the bundle calls gives them
        rows = json.loads(POP.read_text())["dtm_oper"]
        n = 0
        for c in rows:
            alt = float(c["alt"])
            if alt <= 120.5 or len(c["kp"]) != 1:
                continue
            f, fb = float(c["f"]), float(c["fbar"])
            if not (60 <= f <= 250 and 60 <= fb <= 250):
                continue
            got = call("atmosbundle::dtm_density", alt, float(c["lat"]), float(c["lon"]), float(c["lst"]), float(c["doy"]), f, fb, float(c["kp"][0]))[0]
            self.assertAlmostEqual(got / float(c["rho"]), 1.0, delta=1e-9)
            n += 1
        self.assertGreater(n, 10)

    def test_the_bundle_is_ordered_and_its_answer_is_the_high_member(self):
        lo, me, hi = call("atmosbundle::density_bundle", H550, T_EPOCH)
        self.assertLess(lo, me)
        self.assertLess(me, hi)
        self.assertEqual((lo, me, hi), (member(LOW), member(MEAN), member(HIGH)))
        self.assertEqual(call("atmosbundle::density_worst", H550, T_EPOCH)[0], hi)

    def test_the_static_table_was_far_from_both_ends(self):
        # before: the static table at 550 km was 10 times the low member and 4.8 times below the high one
        v = vallado_550()
        self.assertAlmostEqual(v / member(LOW), 10.4, delta=0.1)
        self.assertAlmostEqual(member(HIGH) / v, 4.75, delta=0.05)

    def test_the_density_falls_with_altitude(self):
        for level in (LOW, MEAN, HIGH):
            self.assertGreater(member(level, 400e3), member(level, 550e3))
            self.assertGreater(member(level, 550e3), member(level, 800e3))


class InTheDesign(unittest.TestCase):
    def body(self, nid):
        from_design = __import__("from_design")
        with sqlite3.connect(f"file:{from_design.design()}?mode=ro", uri=True) as c:
            (x,) = c.execute("SELECT content FROM design_node WHERE id = ?", (nid,)).fetchone()
        b = json.loads(x)["body"]
        return {(s, f): v for s, f, v, _o in b["content"]}, b

    def test_the_spec_text_no_longer_names_nrlmsis(self):
        for nid in ("m3_3", "gd_1"):
            c, _ = self.body(nid)
            for (s, f), v in c.items():
                if s in ("relation", "spec", "sources") and f in ("why", "source", "cited", "steps", "derivation", "expression"):
                    self.assertNotIn("nrlmsis", str(v).lower(), f"{nid} {s}.{f}")

    def test_gd_1_reads_the_density_row_which_answers_with_the_high_member(self):
        c, b = self.body("m3_3")
        self.assertEqual(c[("code", "function")], "density_worst")
        _, g = self.body("gd_1")
        self.assertTrue(any(i[0] == "rho" and i[1] == "m3_3" for i in g["input"]))
        self.assertIn("high member", self.body("gd_1")[0][("relation", "why")])


if __name__ == "__main__":
    unittest.main()
