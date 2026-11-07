"""The shadow's ill-conditioned area states its tolerance in the design (S11.8c); the global limit is not loosened.

Copyright (c) 2026 Agastya. All rights reserved."""
import unittest

import _path  # puts tools/ on the import path
import translators as T

_ = _path


class SrpTolerance(unittest.TestCase):
    def test_the_functions_through_the_lens_area_state_the_tolerance_and_the_global_limit_stands(self):
        self.assertEqual(T.TOL, 1e-12)
        files = T.env_files()
        sigs = {f"{s['module']}::{s['name']}": s for s in T.node("signatures", *files)}
        stated = {k for k, s in sigs.items() if k.startswith("srp::") and "tolerance: 1e-8" in s["doc"]}
        self.assertEqual(stated, {"srp::frac_conical", "srp::srp_eclipse", "srp::srp_force"})
        self.assertTrue(all("last-bit difference" in " ".join(sigs[k]["doc"]) for k in stated), "the reason stands beside it")


if __name__ == "__main__":
    unittest.main()
