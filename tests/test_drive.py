"""The shared drive (docs/PLAN_2_0.md S4, zip 1): tools/drive.py packs the converted design with its guides, START HERE
and a manifest of every file's size, SHA-256 and MD5, and finds every difference between a drive and that manifest.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import shutil
import tempfile
import unittest

import _path  # puts tools/ on the import path
import drive

_ = _path  # imported for its effect: tools/ on sys.path


class Drive(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.db, cls.m = drive.pack(pathlib.Path(cls.tmp.name) / "out", zip_it=True)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def copy(self):
        d = pathlib.Path(self.tmp.name) / "copy"
        if d.exists():
            shutil.rmtree(d)
        shutil.copytree(self.db, d)
        return d

    def test_it_holds_the_layout_and_checks_clean(self):
        top = sorted(p.name for p in self.db.iterdir())
        self.assertEqual(top, ["MANIFEST.json", "START HERE.txt", "cases", "daily", "design", "groups", "guides", "integration",
                               "issues", "readable", "results"])
        self.assertEqual(len(list((self.db / "groups").iterdir())), 21)
        self.assertTrue(all(len(list((g / "releases").glob("*.tnrel"))) == 1 for g in (self.db / "groups").iterdir()))
        self.assertEqual(drive.verify(self.db, first_upload=True), [])
        for f in self.m["files"]:
            self.assertEqual(set(f), {"path", "size", "sha256", "md5"})

    def test_every_difference_is_named(self):
        d = self.copy()
        node = next((d / "groups" / "env" / "nodes").glob("*.node.tndb"))
        b = bytearray(node.read_bytes())
        b[-1] ^= 1
        node.write_bytes(bytes(b))
        (d / "readable" / "Nodes.csv").unlink()
        (d / "groups" / "env" / "notes(1).txt").write_text("x")
        (d / "Design").mkdir()
        (d / "Design" / "old.tndb").write_text("x")
        (d / "old-1.0").mkdir()
        (d / "old-1.0" / "Guides.txt").write_text("kept by hand")
        bad = drive.verify(d, first_upload=True)
        self.assertTrue(any(node.name in x and "MD5" in x for x in bad), bad)
        self.assertIn("readable/Nodes.csv: missing", bad)
        self.assertTrue(any("notes(1).txt" in x and "second copy" in x for x in bad), bad)
        self.assertTrue(any(x.startswith("Design/: 1.0.0's layout") for x in bad), bad)
        self.assertFalse(any(x.startswith("old-1.0/") for x in bad), "old-1.0/ is left out")

    def test_a_listing_from_the_drive_is_checked_by_name_size_and_md5(self):
        have = [{"path": f["path"], "size": f["size"], "md5": f["md5"]} for f in self.m["files"]]
        self.assertEqual(drive.compare(self.m, {x["path"]: (x["size"], x["md5"]) for x in have}, first_upload=True), [])
        have[0]["size"] += 1
        self.assertEqual(len(drive.compare(self.m, {x["path"]: (x["size"], x["md5"]) for x in have}, first_upload=True)), 1)

    def test_packing_again_gives_the_same_zip(self):
        again = pathlib.Path(self.tmp.name) / "again"
        drive.pack(again, zip_it=True)
        z = "Trinetra_Database_zip1.zip"
        self.assertEqual((again / z).read_bytes(), (pathlib.Path(self.tmp.name) / "out" / z).read_bytes())
        self.assertEqual(json.loads((again / "Trinetra Database" / "MANIFEST.json").read_text()), self.m)


if __name__ == "__main__":
    unittest.main()
