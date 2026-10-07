"""tools/version.py: VERSION is the one version; a part that drifts, or a build id written as a
literal, is caught; --set writes every part. Run on a copy of the files, never the checkout.
Copyright (c) 2026 Agastya.
"""
import pathlib
import shutil
import tempfile
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import version as V
from common import ROOT

_ = _path  # imported for its effect: tools/ on sys.path

FILES = ["VERSION"] + [rel for _, rel, _ in V.PARTS] + [rel for _, rel, _ in V.DERIVED]


class Version(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp())
        for rel in FILES:
            (self.tmp / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(ROOT / rel, self.tmp / rel)
        self.root = mock.patch.object(V, "ROOT", self.tmp)
        self.root.start()

    def tearDown(self):
        self.root.stop()
        shutil.rmtree(self.tmp)

    def edit(self, rel, a, b):
        f = self.tmp / rel
        text = f.read_text()
        self.assertIn(a, text)
        f.write_text(text.replace(a, b, 1))

    def test_the_repository_is_one_version(self):
        self.assertEqual(V.problems(V.source()), [])

    def test_a_part_that_drifts_is_named(self):
        v = V.source()
        self.edit("engine/Cargo.toml", f'version = "{v}"', 'version = "9.9.9"')
        bad = V.problems(v)
        self.assertEqual(len(bad), 1)
        self.assertIn("engine (Cargo workspace)", bad[0])
        self.assertEqual(V.main(["--check"]), 1)

    def test_a_literal_build_id_is_caught(self):
        self.edit("fsw-rs/src/fsw.rs", 'concat!("trinetra-fsw-rs/", env!("CARGO_PKG_VERSION"), " (adcs-fswcfg/1) alg ", adcs_alg_id!())',
                  'concat!("trinetra-fsw-rs/1.0.0 (adcs-fswcfg/1) alg ", adcs_alg_id!())')
        self.assertTrue(any("fsw::BUILD_ID" in b for b in V.problems(V.source())))

    def test_set_writes_every_part(self):
        self.assertEqual(V.main(["--set", "2.3.4"]), 0)
        self.assertEqual(V.source(), "2.3.4")
        self.assertEqual({got for _, _, got in V.read_parts()}, {"2.3.4"})
        self.assertEqual(V.problems("2.3.4"), [])

    def test_a_version_that_is_not_x_y_z_is_refused(self):
        with self.assertRaises(SystemExit):
            V.set_version("1.0")
        (self.tmp / "VERSION").write_text("one\n")
        with self.assertRaises(SystemExit):
            V.source()


if __name__ == "__main__":
    unittest.main()
