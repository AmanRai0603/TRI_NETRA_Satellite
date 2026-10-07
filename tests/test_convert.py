"""The design leaves the repository (docs/PLAN_2_0.md S3): tools/convert_2_0.py writes the 2.0.0 layout,
and nothing 1.0.0 holds is dropped:
  - every 1.0.0 node's every 1.0.0 table is in its converted file, field by field (only a dissolved
    group's node names its new group); a row added is the developer's revision (design/revisions_2_0.toml), named
    in its origin or its history;
  - every library file (catalogue, KPIs, units, flight parameters), every case, scenario, campaign and
    trade is held with its text whole, a case's lines one by one;
  - every flight algorithm module, every one of its vectors, every flight parameter and the IGRF table;
  - Conversion.csv places everything and drops nothing;
  - every file checks, in tools/tndb.py and in the library; every baseline release passes
    tools/release.py and the library; the tree has one root and every parent and mount is a block;
  - converting twice gives the same files, byte for byte.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import csv
import hashlib
import io
import os
import pathlib
import sqlite3
import subprocess
import tempfile
import tomllib
import unittest

import _path  # puts tools/ on the import path
import carry_over
import convert_2_0 as conv
import seed_design

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
TNDB = ROOT / "engine" / "target" / "release" / "tndb"
OLD = ["node", "content", "input", "output", "fixture", "attachment", "revision", "comment", "change_request", "signature"]


def table(path, t):
    with sqlite3.connect(path) as c:
        return sorted(c.execute(f'SELECT * FROM "{t}"').fetchall(), key=repr)


class Convert(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        d = pathlib.Path(cls.tmp.name)
        cls.src = d / "src"
        seed_design.seed(cls.src, sync=False)
        carry_over.carry(cls.src)
        cls.out = d / "out"
        cls.summary = conv.convert(cls.out, cls.src)
        cls.where = {f.name[:-len(".node.tndb")]: f for f in (cls.out / "groups").glob("*/nodes/*.node.tndb")}

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_folder_checks_clean(self):
        self.assertEqual(conv.check_folder(self.out), [])
        self.assertEqual(self.summary["groups"], 21)
        self.assertEqual(self.summary["converted_1_0_nodes"], 765)

    def test_every_1_0_node_is_there_field_by_field(self):
        for f in sorted((self.src / "nodes").glob("*.node.tndb")):
            nid = f.name[:-len(".node.tndb")]
            g = self.where[nid]
            for t in OLD:
                a, b = table(f, t), table(g, t)
                if t == "node" and a != b:
                    self.assertEqual(a[0][2], "case", f"{nid}: only a dissolved group's node changes its group")
                    self.assertEqual(a[0][:2] + a[0][3:], b[0][:2] + b[0][3:], nid)
                    continue
                if t in ("content", "revision") and a != b:
                    self.assertEqual([r for r in a if r not in b], [], f"{nid}: table {t}: a 1.0.0 row is not kept")
                    added = [r for r in b if r not in a]
                    mark = (lambda r: r[3].startswith("the developer's revision ")) if t == "content" else (lambda r: r[2] == "the developer")
                    self.assertTrue(all(mark(r) for r in added), f"{nid}: table {t}: a row added is not the developer's revision: {added}")
                    continue
                self.assertEqual(a, b, f"{nid}: table {t}")

    def test_every_node_has_one_block_and_its_ports(self):
        for nid, f in self.where.items():
            blk = table(f, "block")
            self.assertEqual(len(blk), 1, nid)
            outs = {r[0] for r in table(f, "output")}
            ports = {r[0] for r in table(f, "port") if r[1] == "out"}
            self.assertEqual(outs, ports, f"{nid}: every output is a port")

    def test_the_library_data_and_the_cases_are_held_whole(self):
        for folder, pat, gid, _what in conv.LIBRARY:
            for src in sorted((ROOT / folder).glob(pat)):
                rel = src.relative_to(ROOT).as_posix()
                nid = "lib_" + "".join(ch if ch.isalnum() else "_" for ch in rel.lower().rsplit(".", 1)[0])
                nid = "_".join(x for x in nid.split("_") if x) if "__" in nid else nid
                f = next((p for k, p in self.where.items() if k.startswith("lib_") and table(p, "content") and
                          any(r[3] == rel for r in table(p, "content"))), None)
                self.assertIsNotNone(f, rel)
                text = next(r[2] for r in table(f, "content") if r[3] == rel)
                self.assertEqual(text.encode("utf-8"), src.read_bytes(), rel)
        texts = {}
        for f in (self.out / "cases").glob("*.tncase"):
            for name, fmt, text in table(f, "case_source"):
                texts.setdefault(text, []).append(f)
                if fmt == "case":
                    n_lines = len(list(csv.reader(io.StringIO(text)))) - 1
                    self.assertEqual(len(table(f, "case_line")), n_lines, f.name)
        for folder, pat, _kind in conv.CASES:
            for src in sorted((ROOT / folder).glob(pat)):
                self.assertIn(src.read_bytes().decode("utf-8"), texts, src)

    def test_the_flight_software_design_is_there(self):
        vec = [x for x in (ROOT / "fsw" / "tests" / "pcode_vectors.txt").read_text().splitlines() if x.strip() and not x.startswith("#")]
        held = 0
        for fname in conv.FSW_MODULES:
            text = (ROOT / "fsw" / "pseudocode" / fname).read_bytes().decode("utf-8")
            f = next(p for k, p in self.where.items() if k.startswith("fsw_") and any(r[2] == text for r in table(p, "content")))
            held += len(table(f, "fixture"))
        mods = {fname: (ROOT / "fsw" / "pseudocode" / fname).read_text().split("module ", 1)[1].split()[0] for fname in conv.FSW_MODULES}
        self.assertEqual(held, sum(1 for x in vec if x.split("::")[0] in mods.values()))
        params = tomllib.loads((ROOT / "fsw" / "params" / "params.toml").read_text())["field"]
        for p in params:
            self.assertIn(f"fsw_param_{p['name']}", self.where, p["name"])
        igrf = table(self.where["env_igrf13_coefficients"], "content")
        self.assertIn((ROOT / "matlab_sils" / "data" / "igrf13coeffs.txt").read_bytes().decode("utf-8"), [r[2] for r in igrf])

    def test_the_report_places_everything_and_drops_nothing(self):
        with open(self.out / "readable" / "Conversion.csv", encoding="utf-8") as fh:
            rep = list(csv.DictReader(fh))
        self.assertFalse([r for r in rep if "dropped" in (r["is now"] + r["how"]).lower()])
        self.assertEqual(sum(r["what"] == "node" for r in rep), 765)
        for r in rep:
            if r["what"] in ("node", "block", "flight algorithms", "flight parameter", "library", "table"):
                self.assertTrue((self.out / r["is now"]).is_file(), r)

    def test_case_lines_name_the_node_that_declares_them(self):
        import design_inputs
        f = self.out / "cases" / "ais_3u.tncase"
        lines = table(f, "case_line")
        want = design_inputs.case_rows(ROOT / "matlab_sils" / "cases" / "ais_3u.csv")
        self.assertEqual(sorted(lines), sorted(tuple(r[1:]) for r in want))
        self.assertGreater(sum(1 for r in lines if r[10]), 0)

    def test_every_built_in_relation_is_listed_with_its_owner(self):
        with open(self.out / "readable" / "BuiltIn.csv", encoding="utf-8") as fh:
            rows_ = list(csv.DictReader(fh))
        n = sum(1 for p in self.where.values() if table(p, "block")[0][3] == "built-in")
        self.assertEqual(len(rows_), n)
        self.assertTrue(all(r["group"] for r in rows_))

    @unittest.skipUnless(TNDB.is_file() and os.access(TNDB, os.X_OK), "the library's command is not built (engine/target/release/tndb)")
    def test_the_library_finds_every_file_and_release_sound(self):
        files = [str(p) for p in sorted(self.out.rglob("*")) if p.suffix in (".tndb", ".tnrel", ".tncase")]
        r = subprocess.run([str(TNDB), "check", *files], capture_output=True, text=True, timeout=600)
        self.assertEqual(r.returncode, 0, r.stderr[-2000:])
        for rel in sorted(self.out.glob("groups/*/releases/*.tnrel")):
            r = subprocess.run([str(TNDB), "check-release", str(rel)], capture_output=True, text=True, timeout=600)
            self.assertEqual(r.stdout.strip(), "", rel.name)

    def test_converting_again_gives_the_same_files(self):
        again = pathlib.Path(self.tmp.name) / "again"
        conv.convert(again, self.src)

        def digest(root):
            return {p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(root.rglob("*")) if p.is_file()}
        self.assertEqual(digest(again), digest(self.out))


if __name__ == "__main__":
    unittest.main()
