"""The Drive folder check (tools/drive_pack.py --verify, docs/TECHNICAL_ROADMAP.md D0): a folder equal to its
manifest passes; a missing file, a changed app, a conflict copy, a file that must not be there, a stray file and a
damaged design file are each named; a design file changed by people is a note, not a problem, unless the check
is of a first upload.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import shutil
import tempfile
import unittest

import _path  # puts tools/ on the import path
import drive_pack
import tndb

_ = _path  # imported for its effect: tools/ on sys.path


class Verify(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = pathlib.Path(self.tmp.name) / "Trinetra Database"
        (self.d / "Apps").mkdir(parents=True)
        (self.d / "Design" / "nodes").mkdir(parents=True)
        (self.d / "Apps" / "TRI-NETRA Node.html").write_text("<!doctype html>app")
        (self.d / "README.txt").write_text("readme")
        tndb.create(self.d / "Design" / "nodes" / "x.node.tndb", "node", "x")
        self.man = drive_pack.manifest(self.d, "test")

    def tearDown(self):
        self.tmp.cleanup()

    def problems(self, first=False):
        return drive_pack.verify(self.d, self.man, first)

    def test_a_folder_equal_to_its_manifest_passes(self):
        self.assertEqual(self.problems(first=True), ([], []))

    def test_each_kind_of_damage_is_named(self):
        (self.d / "README.txt").unlink()
        (self.d / "Apps" / "TRI-NETRA Node.html").write_text("changed")
        (self.d / "Design" / "x (1).tndb").write_bytes(b"")
        (self.d / "Design" / "tool.py").write_text("")
        (self.d / "stray.txt").write_text("")
        p, _ = self.problems()
        text = "\n".join(p)
        for want in ("missing: README.txt", "changed: Apps/TRI-NETRA Node.html", "conflict copy",
                     "must not be here: Design/tool.py", "should not be here: stray.txt"):
            self.assertIn(want, text)

    def test_a_design_file_edited_by_people_is_a_note_unless_first_upload(self):
        f = self.d / "Design" / "nodes" / "x.node.tndb"
        shutil.copy(f, f.with_suffix(".bak"))
        f.unlink()
        tndb.create(f, "node", "x")
        import sqlite3
        c = sqlite3.connect(f)
        c.execute("insert or replace into meta(key, value) values ('edited', 'yes')")
        c.commit()
        c.close()
        f.with_suffix(".bak").unlink()
        p, notes = self.problems()
        self.assertEqual(p, [])
        self.assertTrue(any("people's edits" in n for n in notes))
        p, _ = self.problems(first=True)
        self.assertTrue(any(x.startswith("changed: Design/nodes/x.node.tndb") for x in p))

    def test_a_damaged_design_file_is_a_problem(self):
        (self.d / "Design" / "nodes" / "x.node.tndb").write_bytes(b"not a database")
        p, _ = self.problems()
        self.assertTrue(any("x.node.tndb:" in x for x in p))


if __name__ == "__main__":
    unittest.main()
