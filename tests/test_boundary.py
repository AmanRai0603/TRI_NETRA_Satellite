"""The database/code boundary (docs/PLAN_2_0.md S7 done-when, tools/boundary.py): the repository's code holds no relation
or published model outside its generated files and the toolbox but the exceptions it declares, and a relation planted in
a copy of the code is found by name: a published constant and a gravity law in the engine's core, a coefficient table in
a tool, a hand-written file hiding in a generated folder, a file nobody classified, and a registry that tries to give
design a class of its own.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import shutil
import tempfile
import tomllib
import unittest

import _path  # puts tools/ on the import path
import boundary
from common import ROOT

_ = _path  # imported for its effect: tools/ on sys.path


class Boundary(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.copy = pathlib.Path(cls.tmp.name) / "repo"
        reg = tomllib.loads((ROOT / boundary.REGISTRY).read_text(encoding="utf-8"))
        for f in boundary.files_in(ROOT, reg):
            (cls.copy / f).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / f, cls.copy / f)
        (cls.copy / "tools").mkdir(exist_ok=True)
        shutil.copyfile(ROOT / boundary.REGISTRY, cls.copy / boundary.REGISTRY)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def planted(self, path, text, registry=None):
        """The problems of the copy with `text` added to `path` (or the registry replaced), the copy put back after."""
        p = self.copy / path
        old = p.read_text(encoding="utf-8") if p.exists() else None
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text((old or "") + text, encoding="utf-8")
        reg = None
        if registry is not None:
            reg = pathlib.Path(self.tmp.name) / "registry.toml"
            reg.write_text(registry, encoding="utf-8")
        try:
            return boundary.check(self.copy, reg)[0]
        finally:
            if old is None:
                p.unlink()
            else:
                p.write_text(old, encoding="utf-8")

    def test_the_repository_holds_the_boundary(self):
        problems, report = boundary.check(ROOT)
        self.assertEqual(problems, [])
        self.assertTrue(any("declared exceptions" in r for r in report))

    def test_the_copy_holds_it_too(self):
        self.assertEqual(boundary.check(self.copy)[0], [])

    def test_a_published_constant_and_a_gravity_law_in_the_engine_are_found(self):
        got = self.planted("engine/crates/adcs-sim/src/run.rs",
                           "\nfn accel(r: [f64; 3]) -> f64 {\n    let mu = 3.986004418e14;\n    let rn = norm(r);\n    -mu / rn.powi(3)\n}\n")
        self.assertTrue(got, "a relation planted in the engine's core was not found")
        self.assertTrue(all(g.startswith("engine/crates/adcs-sim/src/run.rs:") for g in got), got)
        self.assertTrue(any("earth GM" in g for g in got), got)
        self.assertTrue(any("inverse-cube gravity" in g for g in got), got)

    def test_a_relation_in_a_test_module_of_a_source_file_is_a_fixture(self):
        self.assertEqual(self.planted("engine/crates/adcs-sim/src/run.rs",
                                      "\n#[cfg(test)]\nmod t {\n    const MU: f64 = 3.986004418e14;\n}\n"), [])

    def test_a_coefficient_table_in_a_tool_is_found(self):
        rows = "\n".join(f"    {1.2345678 + i:.7f}, {2.3456789 + i:.7f}, {3.4567891 + i:.7f}," for i in range(5))
        got = self.planted("tools/report_base.py", f"\nCOEFFS = [\n{rows}\n]\n")
        self.assertTrue(any(g.startswith("tools/report_base.py:") and "table" in g for g in got), got)

    def test_a_hand_written_file_cannot_hide_in_a_generated_folder(self):
        got = self.planted("engine/crates/adcs-sim/src/gen/handmade.rs", "pub fn srp() -> f64 { 4.56e-6 }\n")
        self.assertTrue(any("handmade.rs: in a generated folder" in g for g in got), got)

    def test_an_unclassified_file_is_refused(self):
        got = self.planted("fsw-rs/orbit_helper.rs", "pub fn period(a: f64) -> f64 { a }\n")
        self.assertTrue(any("fsw-rs/orbit_helper.rs: not classified" in g for g in got), got)

    def test_design_has_no_class_in_the_code(self):
        text = (ROOT / boundary.REGISTRY).read_text(encoding="utf-8").replace(
            '[classes]\n', '[classes]\nmodel = "a relation kept in code"\n', 1)
        got = self.planted("tools/report_base.py", "", registry=text)
        self.assertTrue(any("a class for design in code (model)" in g for g in got), got)

    def test_a_stale_allowance_is_a_problem(self):
        text = (ROOT / boundary.REGISTRY).read_text(encoding="utf-8") + (
            '\n[[allow]]\nfile = "tools/report_base.py"\nfind = "earth GM"\nwhy = "nothing there"\n')
        got = self.planted("tools/report_base.py", "", registry=text)
        self.assertTrue(any("tools/report_base.py (earth GM) matches nothing" in g for g in got), got)


if __name__ == "__main__":
    unittest.main()
