"""A5 (DR-vv-002): a campaign's inertia dispersion is always a body that can exist. 1e4 Monte Carlo draws each satisfy the triangle
inequality of the principal moments for every ordering, every edge corner of the shipped edge campaigns does, and a draw that was
a body is the draw the seed always gave.
Copyright (c) 2026 Agastya. All rights reserved."""
import json
import random
import unittest

from _path import ROOT
import common
import engine

CAMP = ROOT / "matlab_sils" / "data" / "campaigns"


def moments(cv, f):
    return [cv["mass.imin"] * f[0], cv["mass.iint"] * f[1], cv["mass.imax"] * f[2]]


def is_body(m):
    a, b, c = m
    return min(m) > 0 and a + b >= c and b + c >= a and a + c >= b


def scales(sets):
    return next(json.loads(s.split("=", 1)[1]) for s in sets if s.startswith("engine.inertia_scale="))


class InertiaDispersion(unittest.TestCase):
    def test_ten_thousand_draws_are_all_bodies(self):
        cv = common.case_values("ais_3u")
        C = {"case": "ais_3u", "seed": 1, "type": "mc", "dispersions": [{"kind": "inertia", "frac": 0.1}]}
        for k in range(1, 10001):
            f = scales(engine.draw(C, k)[0])
            self.assertTrue(is_body(moments(cv, f)), (k, f))

    def test_a_draw_that_was_a_body_is_the_draw_the_seed_gave(self):
        cv = common.case_values("ais_3u")
        C = {"case": "ais_3u", "seed": 5, "type": "mc", "dispersions": [{"kind": "inertia", "frac": 0.1}]}
        kept = moved = 0
        for k in range(1, 301):
            r = random.Random(5 + 7919 * k)
            raw = [1 + 0.1 * (2 * r.random() - 1) for _ in range(3)]
            got = scales(engine.draw(C, k)[0])
            if is_body(moments(cv, raw)):
                kept += 1
                self.assertEqual(got, raw, k)
            else:
                moved += 1
                self.assertNotEqual(got, raw, k)
        self.assertGreater(kept, 0)
        self.assertGreater(moved, 0)

    def test_every_edge_corner_is_a_body(self):
        n = 0
        for p in sorted(CAMP.glob("edge_*.json")):
            C = json.loads(p.read_text())
            cv = common.case_values(C["case"])
            for k in range(1, 2 * len(C["dispersions"]) + 2):
                sets, _d = engine.draw(C, k)
                if any(s.startswith("engine.inertia_scale=") for s in sets):
                    n += 1
                    self.assertTrue(is_body(moments(cv, scales(sets))), (p.name, k))
        self.assertGreater(n, 0)


if __name__ == "__main__":
    unittest.main()
