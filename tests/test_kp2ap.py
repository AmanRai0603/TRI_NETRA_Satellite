"""The Kp -> ap table is one table in three places -- the propagator (Rust), the MATLAB twin's
campaign draws and the Python campaign draws -- with one rule: interp1 "nearest" by midpoints.
Copyright (c) 2026 Agastya. All rights reserved."""
import re
import unittest

from _path import ROOT
import engine


def floats(text):
    return [float(x) for x in re.findall(r"-?(?:\d+(?:\.\d*)?|\.\d+)", text)]


class KpToAp(unittest.TestCase):
    def test_the_three_tables_are_one(self):
        rs = (ROOT / "engine/crates/adcs-pop/src/spaceweather.rs").read_text()
        kpv = floats(re.search(r"const KPV: \[f64; 28\] = \[(.*?)\];", rs, re.S).group(1))
        apv = floats(re.search(r"const APV: \[f64; 28\] = \[(.*?)\];", rs, re.S).group(1))
        m = (ROOT / "matlab_sils/+asils/+campaign/draw.m").read_text()
        mk = floats(re.search(r"kpv = \[(.*?)\];", m).group(1))
        ma = floats(re.search(r"apv = \[(.*?)\];", m).group(1))
        self.assertEqual(kpv, mk)
        self.assertEqual(apv, ma)
        self.assertEqual(kpv, [float(x) for x in engine.KP_NODES])
        self.assertEqual(apv, [float(x) for x in engine.AP_NODES])

    def test_nearest_node_by_midpoints(self):
        self.assertEqual([engine.kp2ap(k) for k in (0, 2, 5, 6, 9, 12, -1)], [0, 7, 48, 80, 400, 400, 0])
        self.assertEqual(engine.kp2ap(0.165), 2)   # the midpoint of 0 and 0.33 goes up, as interp1 does
        self.assertEqual(engine.kp2ap(0.5), 3)     # 0.5 is past (0.33 + 0.67)/2 in floating point

    def test_ap_never_leaves_its_scale(self):
        self.assertTrue(all(0 <= engine.kp2ap(k / 10) <= 400 for k in range(-20, 120)))


if __name__ == "__main__":
    unittest.main()
