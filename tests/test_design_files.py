"""The design files (P1): the tree's rows, the group map, and the four file kinds made, checked,
dumped, loaded and upgraded from design/schema.toml. Every file is written in a temporary folder.
Copyright (c) 2026 Agastya.
"""
import copy
import pathlib
import shutil
import sqlite3
import tempfile
import unittest
from unittest import mock

import _path  # puts tools/ on the import path
import design_rows
import groups as G
import seed_design
import tndb

_ = _path  # imported for its effect: tools/ on sys.path


class Rows(unittest.TestCase):
    def test_the_tree_has_the_rows_the_spec_states(self):
        rows = design_rows.rows()
        self.assertEqual(len(rows), 734)
        self.assertEqual(design_rows.check(rows), [])

    def test_a_subsystem_layer_has_its_interface_its_target_pairs_and_its_unnamed_rows(self):
        fmr = [r for r in design_rows.rows() if r["branch"] == "fmr"]
        kinds = [r["kind"] for r in fmr]
        self.assertEqual((len(fmr), kinds.count("interface"), kinds.count("required"), kinds.count("internal")), (36, 1, 10, 15))
        self.assertIn("l3_fmr_gf_7_achieved", {r["id"] for r in fmr})


class Groups(unittest.TestCase):
    def test_every_row_is_in_exactly_one_group(self):
        self.assertEqual(G.check(design_rows.rows(), G.load()), [])

    def test_a_target_split_between_groups_takes_its_layer3_rows_with_it(self):
        p = G.place(design_rows.rows(), G.load())
        self.assertEqual(p["gq_2"][0], "fdir")
        self.assertEqual(p["l3_modes_gq_2_achieved"][0], "fdir")
        self.assertEqual(p["l3_modes_gq_0_required"][0], "gdn")

    def test_a_row_no_rule_reaches_and_a_wrong_count_are_named(self):
        g = copy.deepcopy(G.load())
        lab = next(x for x in g["group"] if x["id"] == "lab")
        lab["branches"].remove("fa4")
        errs = G.check(design_rows.rows(), g)
        self.assertTrue(any("fa4_0 (branch fa4) is in no group" in e for e in errs))
        self.assertTrue(any("group lab holds 35 rows" in e for e in errs))

    def test_a_stage_outside_its_group_is_refused(self):
        g = copy.deepcopy(G.load())
        act = next(x for x in g["group"] if x["id"] == "act")
        act["stages"][0]["branches"].append("gd")
        self.assertTrue(any("stage mtq: branches gd" in e for e in G.check(design_rows.rows(), g)))


class Files(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = pathlib.Path(tempfile.mkdtemp())
        seed_design.seed(cls.tmp / "seed", sync=False)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp)

    def test_the_seed_writes_every_group_every_node_and_the_design(self):
        files, errs = seed_design.check_tree(self.tmp / "seed")
        self.assertEqual(errs, [])
        self.assertEqual(len(files), 20 + 734 + 1)

    def test_a_seeded_node_carries_what_the_spec_states_and_says_where_from(self):
        d = tndb.dump(self.tmp / "seed" / "nodes" / "m1_1.node.tndb")
        content = d["tables"]["content"]["rows"]
        self.assertIn(["spec", "symbol", "t_epoch", "spec:seed_content.toml"], content)
        node = d["tables"]["node"]["rows"][0]
        self.assertEqual((node[2], node[7]), ("case", "shell"))

    def round_trip(self, f):
        a = tndb.dump(f)
        out = self.tmp / f"copy_{f.name}"
        out.unlink(missing_ok=True)
        tndb.load(a, out)
        self.assertEqual(tndb.dump(out), a)
        self.assertEqual(tndb.check(out), [])

    def test_every_kind_round_trips_unchanged(self):
        self.round_trip(self.tmp / "seed" / "nodes" / "gq_2.node.tndb")
        self.round_trip(self.tmp / "seed" / "structure" / "act.group.tndb")
        self.round_trip(self.tmp / "seed" / "design.tndb")
        rel = tndb.create(self.tmp / "fdir-1.0.tnrel", "release", "fdir-1.0")
        conn = sqlite3.connect(rel)
        node = tndb.dump(self.tmp / "seed" / "nodes" / "gq_2.node.tndb")
        conn.execute("INSERT INTO release VALUES (?, ?, ?, ?, ?)", ("fdir", "1.0", "2026-10-03", "test", tndb.fingerprint(node)))
        conn.execute("INSERT INTO release_node VALUES (?, ?, ?)", ("gq_2", tndb.fingerprint(node), "{}"))
        conn.commit()
        conn.close()
        self.round_trip(rel)

    def test_a_picture_round_trips_byte_for_byte_and_a_big_one_is_named(self):
        f = tndb.create(self.tmp / "pic.node.tndb", "node", "pic")
        conn = sqlite3.connect(f)
        conn.execute("INSERT INTO attachment VALUES (?, ?, ?, ?)", ("small.png", "image/png", 4, b"\x89PNG"))
        conn.execute("INSERT INTO attachment VALUES (?, ?, ?, ?)", ("big.png", "image/png", 600000, b"x"))
        conn.commit()
        conn.close()
        self.assertTrue(any("attachment big.png is 600000 bytes" in e for e in tndb.check(f)))
        a = tndb.dump(f)
        out = self.tmp / "pic_copy.node.tndb"
        tndb.load(a, out)
        self.assertEqual(tndb.dump(out), a)

    def test_create_never_overwrites_and_a_wrong_kind_is_refused(self):
        f = self.tmp / "seed" / "design.tndb"
        with self.assertRaises(tndb.FormatError):
            tndb.create(f, "design", "again")
        with self.assertRaises(tndb.FormatError):
            tndb.open_file(f, kind="node")

    def test_a_missing_table_is_named(self):
        f = tndb.create(self.tmp / "torn.node.tndb", "node", "torn")
        conn = sqlite3.connect(f)
        conn.execute("DROP TABLE fixture")
        conn.commit()
        conn.close()
        self.assertTrue(any("table fixture missing" in e for e in tndb.check(f)))

    def test_an_older_file_is_upgraded_with_its_original_kept_and_a_newer_one_refused(self):
        s1 = tndb.schema()
        old = tndb.create(self.tmp / "old.node.tndb", "node", "old", s=s1)
        s2 = copy.deepcopy(s1)
        s2["formats"]["node"]["version"] = 2
        ran = []
        with mock.patch.dict(tndb.UPGRADES["node"], {1: lambda conn: ran.append(1)}):
            conn = tndb.open_file(old, s=s2)
            self.assertEqual(dict(conn.execute("SELECT key, value FROM meta").fetchall())["format_version"], "2")
            conn.close()
        self.assertEqual(ran, [1])
        self.assertTrue((self.tmp / "old.node.tndb.v1.bak").exists())
        with self.assertRaises(tndb.FormatError) as e:
            tndb.open_file(old, s=s1)
        self.assertIn("newer than this program", str(e.exception))

    def test_an_upgrade_with_no_step_is_refused(self):
        old = tndb.create(self.tmp / "nostep.node.tndb", "node", "nostep")
        s2 = copy.deepcopy(tndb.schema())
        s2["formats"]["node"]["version"] = 2
        with self.assertRaises(tndb.FormatError) as e:
            tndb.open_file(old, s=s2)
        self.assertIn("no upgrade from node version 1", str(e.exception))


if __name__ == "__main__":
    unittest.main()
