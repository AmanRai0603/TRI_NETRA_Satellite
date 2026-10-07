"""Published data in the design (docs/S7_INVENTORY.md S7.2b; the owner's ruling of 7 Oct 2026): each published model's
tables are data tables of an env node, loaded by a reader (tools/readers.py, code) through the developer's revision
(design/revisions_2_0.toml), never typed by hand.
  - every data block of the regression copy is what its reader gives from its file today, and names the file's sha256;
  - each table is, value for value (the same 64 bits), the table the code flies: generated from the design into the
    engine (adcs-sim-core, adcs-pop) and the flight software;
  - every module checks as pseudocode (the interpreter reads it), and each node cites the publication.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import re
import sqlite3
import struct
import subprocess
import tempfile
import tomllib
import unittest

import _path  # puts tools/ on the import path
import convert_2_0
import readers

_ = _path  # imported for its effect: tools/ on sys.path
ROOT = pathlib.Path(__file__).resolve().parents[1]
REG = ROOT / "tests" / "regression" / "design.tndb"
POP = ROOT / "engine" / "crates" / "adcs-pop"


def rust_table(path, name):
    """The rows of a Rust `const NAME: [[f64; C]; R] = [ ... ];` (or 1-D), each decimal read to its double."""
    text = pathlib.Path(path).read_text()
    m = re.search(rf"(?:const|static) {name}: [^=]+= \[(.*?)\];", text, re.S)
    body = m.group(1)
    rows = re.findall(r"\[([^\[\]]*)\]", body)
    num = lambda s: [float(x.rstrip(".")) for x in re.findall(r"-?[0-9][0-9_.eE+-]*", s)]
    return [num(r) for r in rows] if rows else num(body)


def data_tables(text):
    """{name: values in order} of the data blocks of a module text."""
    out = {}
    for name, body in re.findall(r"^data (\w+): [^\n]+\n(.*?)^end$", text, re.S | re.M):
        out[name] = [float(x) for x in re.findall(r"[-+0-9.eE]+", body)]
    return out


def bits(xs):
    return [struct.pack("<d", float(x)) for x in xs]


class PublishedData(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with sqlite3.connect(f"file:{REG}?mode=ro", uri=True) as c:
            cls.nodes = {nid: (g, json.loads(x)) for nid, g, x in c.execute("SELECT id, group_id, content FROM design_node")}
        cls.data = {}
        for nid, (_g, x) in cls.nodes.items():
            for sec, field, value, origin in json.loads(x["body"])["content"] if isinstance(x["body"], str) else x["body"]["content"]:
                if sec == "data":
                    cls.data[(nid, field)] = (value, origin)
        cls.revs = [d for r in convert_2_0.revisions() for d in r.get("data", [])]

    def module(self, nid):
        (text, _o), = [v for (n, _f), v in self.data.items() if n == nid]
        return data_tables(text)

    def test_every_data_block_is_its_readers_and_names_its_file(self):
        self.assertEqual({(d["node"], f"{d['module']}.pc") for d in self.revs}, set(self.data))
        for d in self.revs:
            text, origin = self.data[(d["node"], f"{d['module']}.pc")]
            got = {}
            for one in d.get("files") or [d]:
                got.update(readers.read({"reader": d["reader"], **one}))
                self.assertIn(readers.sha256(one["file"]), origin)
            have = data_tables(text)
            self.assertEqual(sorted(have), sorted(got))
            for name, (_ty, rows) in got.items():
                flat = [x for r in rows for x in (r if isinstance(r, list) else [r])]
                self.assertEqual(bits(have[name]), bits(flat), f"{d['node']} {name}")
                self.assertIn(d["publication"][:40], text)

    def test_the_engine_flies_the_designs_tables(self):
        """The tables the code flies are the design's, generated (tools/engine_build.py, tools/flight_build.py): the same
        64 bits as the data blocks. (Before S7.3 the engine flew hand-typed copies; at S7.2b each was this table, value
        for value: the leap seconds of adcs-pop time.rs, the six tidal tables of tidal.rs, the IGRF table of the engine's
        field model, the xys06 series the IAU 2006 kernel embedded. S7.3 deleted the copies.)"""
        gen_core = ROOT / "engine" / "crates" / "adcs-sim-core" / "src" / "gen"
        gen_pop = POP / "src" / "gen"
        leap = self.module("env_leap_seconds")["LEAP_SECONDS"]
        self.assertEqual(len(leap), 28 * 3)
        self.assertEqual(bits(leap), bits(x for r in rust_table(gen_pop / "leapsec.rs", "DATA_LEAP_SECONDS") for x in r))
        igrf = self.module("env_igrf13_coefficients")
        for where in (gen_core / "igrf13.rs", ROOT / "fsw-rs" / "src" / "alg" / "igrf13.rs"):
            self.assertEqual(bits(igrf["IGRF_YEAR"]), bits(rust_table(where, "DATA_IGRF_YEAR")), where)
            self.assertEqual(bits(igrf["IGRF_GH"]), bits(x for r in rust_table(where, "DATA_IGRF_GH") for x in r), where)
        tid = self.module("env_tidal_eop_terms")
        for name in tid:
            self.assertEqual(bits(tid[name]), bits(x for r in rust_table(gen_pop / "tidaleopterms.rs", f"DATA_{name}") for x in r), name)
        self.assertEqual(sum(len(v) for v in tid.values()), 1378)
        xys = self.module("env_xys06_series")
        self.assertEqual(len(xys["XYS06_XY"]), 3082 * 18)
        self.assertEqual(len(xys["XYS06_S"]), 66 * 11)
        for name in xys:
            rows = rust_table(gen_pop / "xys06.rs", f"DATA_{name}")
            flat = [x for r in rows for x in r] if rows and isinstance(rows[0], list) else rows
            self.assertEqual(bits(xys[name]), bits(flat), name)
        raw = (POP / "data" / "time_frames" / "xys06.bin").read_bytes()
        v = struct.unpack(f"<{len(raw) // 8}d", raw)
        self.assertEqual(bits(xys["XYS06_XYP"] + xys["XYS06_SPOLY"] + xys["XYS06_XY"]), bits(v[8:8 + 18 + 3082 * 18]))
        self.assertEqual(bits(x for i, x in enumerate(xys["XYS06_S"]) if i % 11), bits(v[8 + 18 + 3082 * 18:]))

    def test_every_module_checks_as_pseudocode_and_cites(self):
        with tempfile.TemporaryDirectory() as tmp:
            files = []
            for (nid, field), (text, _o) in sorted(self.data.items()):
                f = pathlib.Path(tmp) / field
                f.write_text(text)
                files.append(str(f))
            r = subprocess.run(["python3", str(ROOT / "tools" / "pcode.py"), "check", *files], capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
            self.assertIn("0 problems", r.stdout)
        for d in self.revs:
            body = self.nodes[d["node"]][1]["body"]
            body = json.loads(body) if isinstance(body, str) else body
            cited = {(s, f): v for s, f, v, _o in body["content"]}
            self.assertEqual(cited[("sources", f"published.{d['module']}")], d["publication"])
            self.assertEqual(self.nodes[d["node"]][0], "env")
            self.assertEqual(body["block"][0][3], "lookup", d["node"])

    def test_the_revision_is_the_developers_unsigned(self):
        for d in self.revs:
            x = self.nodes[d["node"]][1]
            self.assertEqual(x["sealed_as"], "unconfirmed")
            self.assertTrue(any(w.startswith("the developer's revision S7.2b") and "not yet signed by a person" in w for w in x["why"]), d["node"])
        self.assertEqual(tomllib.loads((ROOT / "design" / "revisions_2_0.toml").read_text())["schema"], "trinetra-revisions/1")


if __name__ == "__main__":
    unittest.main()
