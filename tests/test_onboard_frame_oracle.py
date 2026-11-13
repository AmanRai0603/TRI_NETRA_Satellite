"""An external oracle for the flight's Earth frame (docs/S12_INVENTORY.md S12.5, T5/E3 onboard side; DR-env-002): the design's
env_time_frames eci2ecef (fsw/pseudocode/02_time_frames_models.pc, run by the pseudocode interpreter, the one the C, Rust and
MATLAB translations are held to bit for bit) at epochs from 2025.0 to 2030.0 with an arbitrary Earth orientation
eop = [UT1 - UTC, xp, yp], against ERFA (pyerfa, the BSD release of the IAU SOFA library):

 * nut00b equals erfa.nut00b to the rounding of the sum, and fw06's angles erfa.pfw06 and obl06 (phi_bar as dphi + epsilon_A);
 * the matrix equals the chain built from ERFA's own pieces (pn06 with erfa.nut00b, gmst06, the equation of the equinoxes
   dpsi cos epsilon_A, pom00) to 1e-12 rad: the design's arithmetic is the algebra of the pieces;
 * the matrix is within 4 milliarcseconds of erfa.c2t06a (2.6 measured: the 1 mas of the abridged nutation and the 2.7 mas at most of the equation of the equinoxes' complementary terms), the IAU 2006/2000A chain with the same EOP: what the abridged
   nutation and the dropped complementary terms cost;
 * the frame the flight had before (gmst_rot x prec_rot, the IAU 1982 GMST after the IAU 1976 precession, kept in the file) is
   more than 5 arcseconds from c2t06a at the same epochs, so the test can fail.

The nutation table NUT00B is the nut00b table of the same ERFA, so the first comparison also holds the 77 rows. Skipped, with its
reason, only when pyerfa is not importable. Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import math
import pathlib
import subprocess
import sys
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

ROOT = pathlib.Path(__file__).resolve().parents[1]
FRAMES = ROOT / "fsw" / "pseudocode" / "02_time_frames_models.pc"
LIBS = [ROOT / "fsw" / "pseudocode" / "01_math.pc", ROOT / "fsw" / "pseudocode" / "02_igrf13.pc"]
AS2R = math.pi / 180 / 3600
MAS = AS2R / 1000
TT_UTC = 69.184 / 86400          # TAI - UTC 37 s since 2017-01-01, plus 32.184 s
EPOCHS = [2460676.5 + 0.37, 2460900.81, 2461041.75, 2461300.2, 2461584.5, 2461800.65, 2462100.05, 2462502.5 - 0.1]
EOPS = [[-0.0123, 0.12, 0.34], [0.5, -0.3, 0.2], [0.0, 0.0, 0.0], [-0.21, 0.05, 0.47], [0.08, 0.13, 0.31], [-0.4, 0.6, -0.6], [0.3, -0.1, 0.4], [-0.119, 0.079, 0.359]]


def run(fn, args):
    out = subprocess.run([sys.executable, str(ROOT / "tools" / "pcode.py"), "run", str(FRAMES), *map(str, LIBS), "--fn", fn, "--args", json.dumps(args)],
                         capture_output=True, text=True, check=True).stdout
    return json.loads(out)


def angle(a, b):
    m = np.array(a) @ np.array(b).T
    v = np.array([m[2, 1] - m[1, 2], m[0, 2] - m[2, 0], m[1, 0] - m[0, 1]])
    return math.atan2(0.5 * np.linalg.norm(v), 0.5 * (np.trace(m) - 1))


def rz(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, s, 0], [-s, c, 0], [0, 0, 1]])


@unittest.skipIf(SKIP, SKIP)
class OnboardFrameOracle(unittest.TestCase):
    def test_the_nutation_and_the_precession_are_erfas(self):
        for jd in EPOCHS:
            t = (jd + TT_UTC - 2451545.0) / 36525
            dpsi, deps = run("nut00b", [t])
            e = erfa.nut00b(jd + TT_UTC, 0.0)
            self.assertLess(abs(dpsi - e[0]), 1e-15, (jd, dpsi, e[0]))
            self.assertLess(abs(deps - e[1]), 1e-15, (jd, deps, e[1]))
            gam, dphi, psi, eps = run("fw06", [t])
            g, phi, ps, ep = erfa.pfw06(jd + TT_UTC, 0.0)
            for got, want, name in ((gam, g, "gamma_bar"), (psi, ps, "psi_bar"), (eps, ep, "epsilon_A"), (dphi + eps, phi, "phi_bar")):
                self.assertLess(abs(got - want), 2e-16, (jd, name, got, want))
            self.assertAlmostEqual(eps, erfa.obl06(jd + TT_UTC, 0.0), delta=2e-16)

    def test_the_matrix_is_the_chain_of_erfas_pieces(self):
        worst = 0.0
        for jd, eop in zip(EPOCHS, EOPS):
            c = run("eci2ecef", [jd, eop])[0]
            tt = jd + TT_UTC
            dpsi, deps = erfa.nut00b(tt, 0.0)
            epsa, _rb, _rp, _rbp, _rn, rbpn = erfa.pn06(tt, 0.0, dpsi, deps)
            gast = erfa.gmst06(jd, eop[0] / 86400, tt, 0.0) + dpsi * math.cos(epsa)
            want = np.array(erfa.pom00(eop[1] * AS2R, eop[2] * AS2R, 0.0)) @ rz(gast) @ np.array(rbpn)
            worst = max(worst, angle(c, want))
        print(f"\nthe design's frame against the chain of ERFA's pieces: {worst:.2e} rad")
        self.assertLess(worst, 1e-12)

    def test_the_matrix_is_within_milliarcseconds_of_the_2006_2000a_chain(self):
        worst = 0.0
        for jd, eop in zip(EPOCHS, EOPS):
            c = run("eci2ecef", [jd, eop])[0]
            want = erfa.c2t06a(jd, TT_UTC, jd, eop[0] / 86400, eop[1] * AS2R, eop[2] * AS2R)
            worst = max(worst, angle(c, want))
        print(f"\nthe design's frame against erfa.c2t06a: {worst / MAS:.3f} mas")
        self.assertLess(worst, 4.0 * MAS)

    def test_the_old_frame_is_far_from_the_same_oracle(self):
        worst, best = 0.0, 1e9
        for jd, eop in zip(EPOCHS, EOPS):
            old = np.array(run("gmst_rot", [jd])[0]) @ np.array(run("prec_rot", [jd])[0])
            want = erfa.c2t06a(jd, TT_UTC, jd, eop[0] / 86400, eop[1] * AS2R, eop[2] * AS2R)
            a = angle(old, want) / AS2R
            worst, best = max(worst, a), min(best, a)
        print(f"\nthe flight's former frame against erfa.c2t06a: {best:.2f} to {worst:.2f} arcsec")
        self.assertGreater(best, 5.0)


if __name__ == "__main__":
    unittest.main()
