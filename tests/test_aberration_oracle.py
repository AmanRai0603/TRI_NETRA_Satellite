"""An external oracle for the aberration of starlight (docs/S12_INVENTORY.md S12.5, E2; DR-nav-001): the design's st_aberrate
(sens_star_catalogue, run by the pseudocode interpreter, which the engine's Rust and the twin's MATLAB are held to) against
ERFA's `ab` (pyerfa, the BSD release of the IAU SOFA library's iauAb: the special-relativistic aberration, the exact form of
the Explanatory Supplement to the Astronomical Almanac), for velocities from a low orbit alone to the Earth's orbital speed
plus a low orbit's (7.6 + 29.8 km/s):

 * the design's first-order form differs from erfa.ab by less than (v/c)^2, 2e-8 rad at 37.4 km/s;
 * the shift from the geometric direction is (v/c) sin(angle) to 1e-8 rad, at most 1.2475e-4 rad at 37.4 km/s;
 * the geometric direction (no aberration) is more than 0.7 of v/c from erfa.ab for a star not along the velocity, so the test can fail.

The edition and section of the Explanatory Supplement were not to hand: the exact form is ERFA's. Skipped, with its reason, only
when pyerfa is not importable. Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import math
import pathlib
import subprocess
import sys
import tempfile
import unittest

import _path  # puts tools/ on the import path

_ = _path
try:
    import erfa
    import numpy as np
    SKIP = None
except ImportError as e:        # pragma: no cover
    erfa = np = None
    SKIP = f"pyerfa is not importable ({e}); pip install pyerfa (tools/requirements-ci.txt)"

import from_design

ROOT = pathlib.Path(__file__).resolve().parents[1]
C = 299792458.0
FILES = ["sens/starcat.pc", "spec/physics/constants.pc", "env/facets.pc"]


def run(files, s, v):
    out = subprocess.run([sys.executable, str(ROOT / "tools" / "pcode.py"), "run", *files, "--fn", "st_aberrate", "--args", json.dumps([list(s), list(v)])],
                         capture_output=True, text=True, check=True).stdout
    return np.array(json.loads(out)[0])


@unittest.skipIf(SKIP, SKIP)
class Aberration(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.files = []
        for i, p in enumerate(FILES):
            f = pathlib.Path(self.tmp.name) / f"{i}_{pathlib.Path(p).name}"
            f.write_text(from_design.text(p), encoding="utf-8")
            self.files.append(str(f))

    def tearDown(self):
        self.tmp.cleanup()

    def test_the_first_order_form_is_erfas_to_v_over_c_squared(self):
        worst, shift_worst, none_best = 0.0, 0.0, 1.0
        stars = [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.6, 0.0, 0.8], [-0.48, 0.64, 0.6], [0.0, -1.0, 0.0]]
        for v in ([7.6e3, 0.0, 0.0], [0.0, 29.8e3, 0.0], [7.6e3, 29.8e3, 0.0], [-20e3, 5e3, 28e3], [37.4e3, 0.0, 0.0]):
            vc = np.array(v) / C
            for s in stars:
                s = np.array(s)
                p = run(self.files, s, v)
                e = erfa.ab(s, vc, 1.0, math.sqrt(1 - vc @ vc))
                worst = max(worst, np.linalg.norm(p - e))
                shift_worst = max(shift_worst, abs(np.linalg.norm(p - s) - np.linalg.norm(vc) * np.linalg.norm(np.cross(s, vc / np.linalg.norm(vc)))))
                if abs(s @ (vc / np.linalg.norm(vc))) < 0.7:
                    none_best = min(none_best, np.linalg.norm(s - e) / np.linalg.norm(vc))
        print(f"\nthe design's aberration against erfa.ab: {worst:.2e} rad (limit 2e-8); its shift against (v/c) sin a: {shift_worst:.1e}")
        self.assertLess(worst, 2e-8)
        self.assertLess(shift_worst, 1e-8)
        # the geometric direction (no aberration) is, relative to v/c, a large fraction of it from erfa.ab for stars not along v
        self.assertGreater(none_best, 0.7)


if __name__ == "__main__":
    unittest.main()
