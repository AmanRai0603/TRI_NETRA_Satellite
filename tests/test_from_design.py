"""The design in the repository (docs/PLAN_2_0.md S4): every file the code reads that is design is generated from
the regression copy (tests/regression/design.tndb) and checked against it; tools read the rest of 1.0.0's plan from
the design by its 1.0.0 path.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import tomllib
import unittest

import _path  # puts tools/ on the import path
import design_inputs
import from_design

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]


class FromDesign(unittest.TestCase):
    def test_every_generated_file_is_the_designs(self):
        self.assertEqual(from_design.stale(), [])

    def test_it_gives_the_engines_inputs_cases_parameters_and_algorithms(self):
        outs = from_design.outputs()
        rows, files, _fp = design_inputs.gather()
        self.assertEqual({f"matlab_sils/{p}" for p, _f, _b in files}, {p for p in outs if p.startswith("matlab_sils/data/")})
        self.assertEqual({f"matlab_sils/cases/{c}.csv" for c in {r[0] for r in rows}}, {p for p in outs if p.startswith("matlab_sils/cases/")})
        self.assertIn("fsw/params/params.toml", outs)
        self.assertEqual(len([p for p in outs if p.startswith("fsw/pseudocode/") and p.endswith(".pc")]), 7)

    def test_the_plan_is_read_from_the_design(self):
        kpis = tomllib.loads(from_design.text("spec/plan/kpis.toml"))
        self.assertEqual(len(kpis["kpi"]), 22)
        self.assertEqual(len(from_design.paths("spec/physics/")), 11)
        with self.assertRaises(SystemExit):
            from_design.text("spec/plan/no_such_file.toml")

    def test_a_hand_edit_is_found(self):
        p = ROOT / "matlab_sils" / "cases" / "ais_3u.csv"
        before = p.read_bytes()
        try:
            p.write_bytes(before.replace(b"\n", b"\n", 1) + b"extra,line\n")
            self.assertTrue(any("ais_3u.csv" in x for x in from_design.stale()))
        finally:
            p.write_bytes(before)


if __name__ == "__main__":
    unittest.main()
