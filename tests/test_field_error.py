"""The truth field's error (docs/S12_INVENTORY.md S12.7, E5; DR-env-005): a truth-only node, env_field_error, adds the ring current's
uniform external field (Dst, a stated scenario quantity with a quiet default) and the crust (random Gauss coefficients of the degrees
14 to 20 of a stated RMS at the orbit, drawn from the run's seed) to the truth field. The magnitudes are the decision's, the developer's,
unsigned. The engine's and the twin's tests (adcs-sim tests/field_error.rs, the twin's t_field_error) hold the numbers; this holds the
design's statements and the independent reference they read:
  - the stated values (quiet Dst, the crust's RMS, the error on) are inside the decision's bands: the quiet error (the crust and Dst
    together) 20-50 nT, 100-500 nT at Dst -200 nT;
  - DR-env-005's `sets` names the node and the three values; the nodes say they are the developer's and unsigned; the relation the ring
    current takes (Dessler-Parker-Sckopke) is cited with the words that the paper was not checked;
  - tests/data/field_error_reference.txt is what its generator (the Python standard library alone) writes.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import math
import pathlib
import subprocess
import sys
import tempfile
import tomllib
import unittest

import _path  # puts tools/ on the import path

_ = _path
ROOT = pathlib.Path(__file__).resolve().parents[1]
STATED = json.loads((ROOT / "matlab_sils" / "data" / "stated.json").read_text())["value"]
REVISIONS = (ROOT / "design" / "revisions_2_0.toml").read_text()
NODES = ("env_field_error", "env_dst_default", "env_field_crustal_rms", "env_field_error_on")


class FieldError(unittest.TestCase):
    def test_the_quiet_values_are_inside_the_decisions_bands(self):
        dst, rms = STATED["env_dst_default"], STATED["env_field_crustal_rms"]
        self.assertEqual((dst, rms, STATED["env_field_error_on"]), (-10, 30, 1))
        # the crust and the ring current's uniform field are orthogonal in the mean: the quiet error is their root-sum-square
        self.assertTrue(20 <= math.hypot(rms, dst) <= 50, math.hypot(rms, dst))
        self.assertTrue(100 <= math.hypot(rms, -200) <= 500, math.hypot(rms, -200))

    def test_the_decision_names_the_node_and_its_values(self):
        d = tomllib.loads((ROOT / "design" / "decisions" / "DR-env-005.toml").read_text())
        sets = " ".join(d["sets"])
        for n in NODES:
            self.assertIn(n, sets)
        self.assertEqual(d["status"], "proposed")
        self.assertEqual(d["signature"], {})

    def test_the_nodes_are_the_developers_and_unsigned_and_the_citations_say_what_was_not_checked(self):
        rev = tomllib.loads(REVISIONS)["revision"]
        r = next(x for x in rev if x["id"] == "S12.7b")
        self.assertIn("unsigned", r["about"])
        self.assertIn("developer", r["about"])
        ids = [n["id"] for n in r["node"]]
        self.assertEqual(ids, ["env_field_error", "env_dst_default", "env_field_crustal_rms", "env_field_error_on"])
        m = next(x for x in r["method"] if x["node"] == "env_field_error")
        for word in ("Dessler and Parker 1959", "Sckopke 1966", "Lowes (1966)", "not checked"):
            self.assertIn(word, m["source"])
        self.assertIn("generate", m)
        self.assertEqual(m["generate"], ["adcs-sim-core"])

    def test_the_reference_is_what_its_generator_writes(self):
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "ref.txt"
            subprocess.run([sys.executable, "-I", str(ROOT / "tests" / "data" / "field_error_reference_make.py"), str(out)], check=True)
            self.assertEqual(out.read_text(), (ROOT / "tests" / "data" / "field_error_reference.txt").read_text())


if __name__ == "__main__":
    unittest.main()
