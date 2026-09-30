"""A file a tool writes is whole or absent. Copyright (c) 2026 Agastya. All rights reserved."""
import os
import pathlib
import tempfile
import unittest

from _path import ROOT
import common


class AtomicWrites(unittest.TestCase):
    def setUp(self):
        self.d = pathlib.Path(tempfile.mkdtemp())

    def test_a_write_replaces_whole_and_leaves_no_temporary_file(self):
        f = self.d / "sub" / "ledger.json"
        common.write_text(f, "one")
        common.write_json(f, {"two": 2})
        self.assertEqual(f.read_text(), '{"two": 2}\n')
        self.assertEqual([p.name for p in f.parent.iterdir()], ["ledger.json"])

    def test_a_failed_write_leaves_the_old_file(self):
        f = self.d / "ledger.json"
        common.write_text(f, "old")
        with self.assertRaises(TypeError):
            common.write_bytes(f, "not bytes")
        self.assertEqual(f.read_text(), "old")
        self.assertEqual(os.listdir(self.d), ["ledger.json"])

    def test_a_write_keeps_the_mode_a_file_has_or_gets(self):
        f = self.d / "new.txt"
        common.write_text(f, "x")
        umask = os.umask(0); os.umask(umask)
        self.assertEqual(f.stat().st_mode & 0o777, 0o666 & ~umask)
        os.chmod(f, 0o755)
        common.write_text(f, "y")
        self.assertEqual(f.stat().st_mode & 0o777, 0o755)

    def test_atomic_path_renames_only_on_success(self):
        f = self.d / "out.zip"
        with self.assertRaises(RuntimeError):
            with common.atomic_path(f) as t:
                t.write_bytes(b"half")
                raise RuntimeError("interrupted")
        self.assertFalse(f.exists())
        self.assertEqual(os.listdir(self.d), [])
        with common.atomic_path(f) as t:
            t.write_bytes(b"whole")
        self.assertEqual(f.read_bytes(), b"whole")


if __name__ == "__main__":
    unittest.main()
