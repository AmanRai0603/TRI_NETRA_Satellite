"""The kit a team member gets: the version, and the files-only kit is exactly the data the engine
reads, the ephemeris, the documents and a VERSION file, with nothing left from before.
Copyright (c) 2026 Agastya. All rights reserved."""
import pathlib
import re
import shutil
import tempfile
import unittest

import _path  # puts tools/ on the import path
import kit
from _path import ROOT

_ = _path  # imported for its effect: tools/ on sys.path


class Version(unittest.TestCase):
    def test_the_version_is_the_version_file_stripped(self):
        v = kit.version()
        self.assertEqual(v, (ROOT / "VERSION").read_text().strip())
        self.assertRegex(v, r"^\d+\.\d+\.\d+")

    def test_every_component_names_a_version(self):
        c = kit.components()
        self.assertEqual(set(c), {"engine", "flight software (Rust)", "flight software (C)", "data"})
        for k in ("engine", "flight software (Rust)", "flight software (C)"):
            self.assertRegex(c[k], r"^\d+\.\d+\.\d+", k)


class FilesOnly(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = pathlib.Path(tempfile.mkdtemp())
        cls.out = cls.tmp / "kit"
        cls.out.mkdir()
        (cls.out / "stale.txt").write_text("from an earlier kit")
        cls.v, cls.progs, cls.n = kit.build(cls.out, files_only=True)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, ignore_errors=True)

    def test_it_holds_exactly_the_data_the_ephemeris_the_documents_and_version(self):
        self.assertEqual(sorted(p.name for p in self.out.iterdir()),
                         ["COMMANDS.md", "ENVIRONMENT.md", "FIRST_RUN.md", "START_HERE.md", "VERSION", "cases", "data", "pop"])
        self.assertEqual(self.progs, [], "no programs in a files-only kit")

    def test_the_data_is_every_visible_file_of_data_and_cases_byte_for_byte(self):
        ms = ROOT / "matlab_sils"
        want = sorted(f.relative_to(ms) for sub in ("data", "cases") for f in (ms / sub).rglob("*") if f.is_file() and not f.name.startswith("."))
        got = sorted(f.relative_to(self.out) for sub in ("data", "cases") for f in (self.out / sub).rglob("*") if f.is_file())
        self.assertEqual(got, want)
        self.assertEqual(self.n, len(want) + 1, "the count includes the ephemeris")
        for rel in want[:: max(1, len(want) // 20)]:
            self.assertEqual((self.out / rel).read_bytes(), (ms / rel).read_bytes(), rel)

    def test_the_ephemeris_is_the_one_the_propagator_reads(self):
        rel = pathlib.Path("pop/03_frames_time/ephemeris/data/de440s.bsp")
        self.assertEqual((self.out / rel).stat().st_size, (ROOT / "matlab_sils" / rel).stat().st_size)
        self.assertEqual([p for p in (self.out / "pop").rglob("*") if p.is_file()], [self.out / rel])

    def test_version_names_the_release_and_each_part(self):
        text = (self.out / "VERSION").read_text()
        self.assertEqual(text.splitlines()[0], f"TRI-NETRA ADCS {kit.version()}")
        self.assertEqual(self.v, kit.version())
        for k, x in kit.components().items():
            self.assertIn(f"{k}: {x}\n", text)

    def test_the_documents_are_the_repositorys(self):
        for name in ("START_HERE.md", "FIRST_RUN.md", "COMMANDS.md", "ENVIRONMENT.md"):
            self.assertEqual((self.out / name).read_bytes(), (ROOT / "docs" / name).read_bytes())

    def test_nothing_from_before_is_left(self):
        self.assertFalse((self.out / "stale.txt").exists())


class Programs(unittest.TestCase):
    def test_a_kit_without_its_programs_is_refused(self):
        tmp = pathlib.Path(tempfile.mkdtemp())
        try:
            (tmp / "bin").mkdir()
            (tmp / "bin" / "adcs").write_text("#!/bin/sh\n")         # the app is missing
            with self.assertRaises(SystemExit) as e:
                kit.build(tmp / "kit", bin_dir=tmp / "bin")
            self.assertIn("trinetra-app does not exist", str(e.exception))
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_on_windows_the_app_carries_its_name(self):
        tmp = pathlib.Path(tempfile.mkdtemp())
        try:
            (tmp / "bin").mkdir()
            for p in ("adcs.exe", "trinetra-app.exe"):
                (tmp / "bin" / p).write_bytes(b"MZ")
            v, progs, n = kit.build(tmp / "kit", bin_dir=tmp / "bin")
            self.assertEqual(progs, ["adcs.exe", "TRI-NETRA ADCS.exe"])
            self.assertTrue((tmp / "kit" / "TRI-NETRA ADCS.exe").is_file())
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_the_c_build_id_is_read_from_the_source(self):
        self.assertTrue(re.match(r"^\d+\.\d+\.\d+", kit.fsw_c()))


if __name__ == "__main__":
    unittest.main()
