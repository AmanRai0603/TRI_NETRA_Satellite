"""E10 (docs/S12_INVENTORY.md, S12.6): the dipole rows scale by IGRF's reference radius a = 6371.2 km, the radius the Gauss coefficients are
defined at, not by the WGS-84 equatorial radius R_E = 6378.137 km (1.00327 times too strong at 6928.137 km).
  - dipole_field_equator(6928.137 km) = B0 (6371.2/6928.137)^3 to 1e-12 relative, B0 = sqrt(g10^2 + g11^2 + h11^2) of the IGRF-14 file at
    2025.0, in the design's interpreter, in the relations' generated Rust (through the vectors it reproduces bit for bit) and in the twin's
    generated MATLAB;
  - dipole_field_max at the same radius and several latitudes, the nodes' answers m3_0 and m3_1, and the tilt (unchanged);
  - the test discriminates: with R_E the same assertion fails by 3.3e-3.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import math
import pathlib
import shutil
import subprocess
import unittest

import _path  # puts tools/ on the import path
import from_design
import pcode
import test_igrf14

_ = _path
ROOT = pathlib.Path(__file__).resolve().parents[1]
A_IGRF, R_E = 6371.2, 6378.137          # km: IGRF's reference radius; WGS-84's equatorial radius
FILES = None


def b0_tesla():
    """B0 from the file itself (igrf14coeffs.txt at 2025.0), not from the design's table."""
    epochs, t = test_igrf14.parse_txt(test_igrf14.TXT)
    j = epochs.index(test_igrf14.Decimal("2025.0"))
    g10, g11, h11 = (float(t[k][j]) for k in ((1, 0), (1, 1), (1, -1)))
    return math.sqrt(g10 ** 2 + g11 ** 2 + h11 ** 2) * 1e-9, g10


def run(fn, *args):
    global FILES
    if FILES is None:
        FILES = [str(f) for f in sorted(from_design.folder("spec/physics/").glob("*.pc"))]
    return pcode.cli("run", *FILES, "--fn", fn, "--args", json.dumps(list(args)))


class DipoleRadius(unittest.TestCase):
    def test_the_equatorial_field_at_6928_km_is_b0_times_the_radius_ratio_cubed(self):
        b0, _g10 = b0_tesla()
        r = 6928.137e3
        got = run("dipole_field_equator", r)[0]
        want = b0 * (A_IGRF / 6928.137) ** 3
        self.assertAlmostEqual(got / want, 1.0, delta=1e-12, msg=f"{got} vs {want}")
        self.assertAlmostEqual(run("m3_0", r)[0] / want, 1.0, delta=1e-12)

    def test_the_old_radius_is_what_the_test_rejects(self):
        """The check discriminates: B0 (R_E/r)^3, what the relation computed before E10, is 1.00327 times the relation's value at 6928.137 km (0.33 %
        of the cube of R_E/a), far outside the 1e-12 the first test holds it to."""
        b0, _g10 = b0_tesla()
        got = run("dipole_field_equator", 6928.137e3)[0]
        with_re = b0 * (R_E / 6928.137) ** 3
        self.assertGreater(abs(got / with_re - 1.0), 3e-3)
        self.assertAlmostEqual(with_re / (b0 * (A_IGRF / 6928.137) ** 3), 1.00327, places=5)

    def test_the_maximum_and_the_tilt(self):
        b0, g10 = b0_tesla()
        for r_km in (6500.0, 6928.137, 7400.0, 8500.0):
            for lam in (0.0, 0.3, 0.9, 1.5):
                got = run("dipole_field_max", r_km * 1e3, lam)[0]
                want = b0 * (A_IGRF / r_km) ** 3 * math.sqrt(1 + 3 * math.sin(lam) ** 2)
                self.assertAlmostEqual(got / want, 1.0, delta=1e-12, msg=f"{r_km} {lam}")
                self.assertAlmostEqual(run("m3_1", r_km * 1e3, lam)[0] / want, 1.0, delta=1e-12)
        epochs, t = test_igrf14.parse_txt(test_igrf14.TXT)
        j = epochs.index(test_igrf14.Decimal("2025.0"))
        g11, h11 = float(t[(1, 1)][j]), float(t[(1, -1)][j])
        tilt = math.acos(-g10 / math.sqrt(g10 ** 2 + g11 ** 2 + h11 ** 2))
        self.assertAlmostEqual(run("dipole")[1], tilt, delta=1e-14)

    def test_the_generated_rust_and_matlab_are_the_interpreters(self):
        """The relations crate's vectors (drawn by the interpreter, reproduced bit for bit by the Rust in `cargo test -p adcs-relations`, and by
        the twin in t_physics_vectors) are B0 (a/r)^3 (sqrt(1 + 3 sin^2 lambda)); the twin's package is called here as well."""
        b0, _g10 = b0_tesla()
        vec = json.loads((ROOT / "engine/crates/adcs-relations/tests/physics_vectors.json").read_text())["vectors"]
        n = 0
        for s in vec["env::dipole_field_equator"]["sets"]:
            self.assertAlmostEqual(s["out"][0] / (b0 * (A_IGRF * 1e3 / s["in"][0]) ** 3), 1.0, delta=1e-12)
            n += 1
        for s in vec["env::dipole_field_max"]["sets"]:
            self.assertAlmostEqual(s["out"][0] / (b0 * (A_IGRF * 1e3 / s["in"][0]) ** 3 * math.sqrt(1 + 3 * math.sin(s["in"][1]) ** 2)), 1.0, delta=1e-12)
            n += 1
        self.assertGreaterEqual(n, 20)
        octave = shutil.which("octave-cli")
        if octave:
            out = subprocess.run([octave, "--no-gui", "-q", "--eval",
                                  "addpath('matlab_sils'); startup_asils; printf('%.17g %.17g\\n', asils.relations.env.dipole_field_equator(6928137), "
                                  "asils.relations.env.dipole_field_max(6928137, 0.9))"],
                                 cwd=ROOT / "matlab_sils", capture_output=True, text=True, check=True).stdout.split("\n")[-2].split()
            self.assertAlmostEqual(float(out[0]) / (b0 * (A_IGRF / 6928.137) ** 3), 1.0, delta=1e-12)
            self.assertAlmostEqual(float(out[1]) / (b0 * (A_IGRF / 6928.137) ** 3 * math.sqrt(1 + 3 * math.sin(0.9) ** 2)), 1.0, delta=1e-12)


if __name__ == "__main__":
    unittest.main()
