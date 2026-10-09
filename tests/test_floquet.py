"""The Floquet certificate (node certify; docs/S7_INVENTORY.md S7.15b): ctl's method (ctl_floquet_certificate, module
floquet, generated into the engine), which tools/floquet.py asks through `adcs design call`.

- The tool's words and the method's inputs are the design's: the flight software's law numbers are fswchoice's MtqAlg,
  the gains the record FqGains, field by field.
- The laws, through the engine: none asks a torque on the reference, the PD law restores an error and damps a rate, q and
  -q ask the same torque; the small rotation's quaternion is a unit quaternion of that angle.
- The certificate against the committed results (matlab_sils/store/pipeline/<case>/floquet.json, written by the tool's
  numpy version): the method's tolerance, as the design states it beside fq_certify (`## held to numpy:`), on every
  multiplier and on the reference frame; every verdict, free direction, gain and blob the same. It needs the stored
  runs' channels (git keeps none) and the built engine; without them it is skipped, saying so.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import math
import pathlib
import re
import shutil
import tempfile
import unittest

import _path  # puts tools/ on the import path
import floquet as F
import from_design
from design_call import ENGINE, call

_ = _path  # imported for its effect: tools/ on sys.path

CASES = ("ais_3u", "ais_img_3u")
# nominal parameters (SI): the inertia, the coil window and cycle, the roll axis, the feed-forward bits, the PD gains, the
# averaging laws', Avanzini's, Celani 2026's and TANGO's
P = {"J": [[0.01, 0, 0], [0, 0.04, 0], [0, 0, 0.04]], "mtq_meas": 0.1, "mtq_period": 1.0, "roll_axis": [0, 0, 1.0], "mtq_gg_ff": 0,
     "mtq_Kp": [1e-4] * 3, "mtq_Kd": [2e-3] * 3, "mtq_eps": 0.1, "mtq_k1": 1.0, "mtq_k2": 1.0, "mtq_k16": 1e-3, "mtq_lam16": 0.5,
     "sb_kp": 1e-4, "sb_kd": 1e-3, "sb_kroll": 0.0, "sb_kdroll": 0.0, "sb_roll_gate": 0.9, "sun_axis": [0, 0, 1.0],
     "mtq_Pth": [[1e-4, 0, 0], [0, 1e-4, 0], [0, 0, 1e-4]], "mtq_Pw": [[1e-3, 0, 0], [0, 1e-3, 0], [0, 0, 1e-3]]}


def method():
    return from_design.text("ctl/floquet.pc")


def torque(law, qe, w, wref, e3):
    return call("floquet::fq_torque", F.MTQ_LAW.index(law), [P[k] for k in F.GAINS], qe, w, wref, e3)


@unittest.skipUnless(ENGINE.is_file(), "the engine is not built (cargo build --release -p adcs-cli)")
class Inputs(unittest.TestCase):
    def test_the_law_numbers_are_the_designs(self):
        m = re.search(r"^choice MtqAlg = ((?:[^\n]*\\\n)*[^\n]*)", from_design.text("fsw/params/fswchoice.pc"), re.M)
        opts = [o.strip() for o in m.group(1).replace("\\\n", " ").split(",")]
        self.assertEqual(F.MTQ_LAW + ["none"], opts)

    def test_the_gains_are_the_records_fields(self):
        m = re.search(r"^record FqGains\n(.*?)^end", method(), re.M | re.S)
        self.assertEqual(F.GAINS, [ln.split(":")[0].strip() for ln in m.group(1).splitlines() if ln.strip()])

    def test_the_laws_judged_are_the_registrys_candidates(self):
        import pipeline_base
        laws = F.laws()
        self.assertEqual(len(laws), 7)
        self.assertLessEqual(set(laws), set(pipeline_base.CANDIDATES["mtq_pointing"]))


@unittest.skipUnless(ENGINE.is_file(), "the engine is not built (cargo build --release -p adcs-cli)")
class Laws(unittest.TestCase):
    def test_every_law_asks_no_torque_on_the_reference(self):
        n = 1.1e-3
        wref = [0, -n, 0]                              # about the orbit normal, a principal axis
        for law in F.laws():
            for v in torque(law, [0, 0, 0, 1.0], wref, wref, [0, 0, 1.0]):
                self.assertAlmostEqual(v, 0.0, delta=1e-15, msg=law)

    def test_the_pd_law_restores_an_error_and_damps_a_rate(self):
        qe = call("floquet::fq_rv2q", [0.1, 0, 0])
        self.assertLess(torque("mtq_pd", qe, [0, 0, 0], [0, 0, 0], [0, 0, 1.0])[0], 0)
        tau = torque("mtq_rate_damp", [0, 0, 0, 1.0], [0, 0.01, 0], [0, 0, 0], [0, 0, 1.0])
        for v, want in zip(tau, [0, -2e-5, 0]):
            self.assertAlmostEqual(v, want, delta=1e-20)
        # the short way round: q and -q ask the same torque
        self.assertEqual(torque("mtq_pd", [-x for x in qe], [0, 0, 0], [0, 0, 0], [0, 0, 1.0]), torque("mtq_pd", qe, [0, 0, 0], [0, 0, 0], [0, 0, 1.0]))

    def test_the_small_rotations_quaternion_is_a_unit_one_of_that_angle(self):
        for v in ([0.3, -0.2, 0.1], [math.pi, 0, 0], [0, 0, 1e-17], [0, 0, 0]):
            q = call("floquet::fq_rv2q", v)
            self.assertAlmostEqual(math.sqrt(sum(x * x for x in q)), 1.0, places=12)
            self.assertAlmostEqual(2 * math.acos(min(1.0, q[3])), math.sqrt(sum(x * x for x in v)), places=7)
        for got, want in zip(call("floquet::fq_rv2q", [math.pi, 0, 0]), [1, 0, 0, 0]):
            self.assertAlmostEqual(got, want, delta=1e-12)


@unittest.skipUnless(ENGINE.is_file(), "the engine is not built (cargo build --release -p adcs-cli)")
class AgainstTheCommittedResults(unittest.TestCase):
    """The new path on a copy of the pipeline's families and selection, against the committed floquet.json."""

    def test_every_multiplier_within_the_methods_tolerance_and_every_verdict_the_same(self):
        tol = float(re.search(r"^## held to numpy:\s*([0-9.eE+-]+)\s*$", method(), re.M).group(1))
        self.assertLessEqual(tol, 1e-12, "the stated tolerance")
        for case in CASES:
            fm = json.loads((F.PIPE / case / "families.json").read_text())
            sel = json.loads((F.PIPE / case / "selection.json").read_text())
            if not (F.ROOT / fm[sel["selected"]]["check_dir"] / "c" / "channels.csv").is_file():
                self.skipTest(f"{case}: the stored run's channels are not here (git keeps none)")
        tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, tmp, True)
        for case in CASES:
            with self.subTest(case=case):
                (tmp / case).mkdir()
                for f in ("families.json", "selection.json"):
                    shutil.copy(F.PIPE / case / f, tmp / case / f)
                new = F.certify(case, tmp)
                old = json.loads((F.PIPE / case / "floquet.json").read_text())
                self.assertEqual({k: old[k] for k in ("case", "family", "dispatched_law")}, {k: new[k] for k in ("case", "family", "dispatched_law")})
                for a, b in zip(old["C_body_orbit"], new["C_body_orbit"]):
                    for x, y in zip(a, b):
                        self.assertLessEqual(abs(x - y), tol)
                self.assertEqual([x["law"] for x in old["laws"]], [x["law"] for x in new["laws"]])
                for x, y in zip(old["laws"], new["laws"]):
                    for k in ("dispatched", "gains", "free_directions", "certified"):
                        self.assertEqual(x[k], y[k], f"{case} {x['law']}: {k}")
                    worst = max(abs(p - q) for p, q in zip(x["mu_abs"], y["mu_abs"]))
                    self.assertLessEqual(worst, tol, f"{case} {x['law']}: max |d mu| {worst:.3e}")
                    self.assertLessEqual(abs(x["max_mu"] - y["max_mu"]), tol)
                # the scenarios and blobs each law was certified with are the committed ones, byte for byte
                for f in sorted((F.PIPE / case / "floquet").iterdir()):
                    self.assertEqual(f.read_bytes(), (tmp / case / "floquet" / f.name).read_bytes(), f"{case}: floquet/{f.name}")


if __name__ == "__main__":
    unittest.main()
