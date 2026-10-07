"""Published data in the design (docs/S7_INVENTORY.md S7.2b, S7.3b, S7.3c, S7.3d; the owner's ruling of 7 Oct 2026): each published model's
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
        cls.rev_of = {d["node"]: r["id"] for r in convert_2_0.revisions() for d in r.get("data", [])}

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

    def test_the_atmosphere_flies_the_designs_tables(self):
        """S7.3b: DTM2020's coefficients, JB2008's SET indices and the Kp-ap tables are env's data, read by their readers
        (text_table, solfsmy, dtcfile, matlab_matrix) from the files adcs-pop embedded and the MATLAB it was ported from;
        the precision orbit flies them generated (gen::dtm2020coeffs, gen::jbset, gen::kpap), the same 64 bits. Before
        S7.3b adcs-pop parsed the same files at run time (the coefficient exports, SOLFSMY.TXT and DTCFILE.TXT) and typed
        the Kp-ap tables; the switch-over held every model equal to that code, bit for bit."""
        gen_pop = POP / "src" / "gen"
        dtm = self.module("env_dtm2020_coefficients")
        self.assertEqual({k: len(v) for k, v in dtm.items()}, {"DTM2020_OPER": 96 * 9, "DTM2020_RESEARCH": 96 * 9})
        for name in dtm:
            self.assertEqual(bits(dtm[name]), bits(x for r in rust_table(gen_pop / "dtm2020coeffs.rs", f"DATA_{name}") for x in r), name)
        # the refgen export is the MATLAB transcription of the publication's coefficients: its first row, tt = 1052.93 K
        self.assertEqual(dtm["DTM2020_OPER"][0], 1052.93)
        jb = self.module("env_jb2008_indices")
        self.assertEqual({k: len(v) for k, v in jb.items()}, {"SOLFSMY": 10729 * 10, "DTCFILE": 10729 * 26})
        for name in jb:
            self.assertEqual(bits(jb[name]), bits(x for r in rust_table(gen_pop / "jbset.rs", f"DATA_{name}") for x in r), name)
        self.assertEqual(jb["SOLFSMY"][:10], [1997.0, 1.0, 72.4, 78.0, 74.0, 79.2, 65.4, 73.8, 61.9, 70.7])
        self.assertEqual(jb["DTCFILE"][-26:-22], [2026.0, 137.0, 146.0, 115.0])
        kp = self.module("env_kp_ap_table")
        self.assertEqual({k: len(v) for k, v in kp.items()}, {"KP_AP_KP": 28, "KP_AP_AP": 28, "DTM_AAP": 38, "DTM_KP": 38})
        for name in kp:
            self.assertEqual(bits(kp[name]), bits(rust_table(gen_pop / "kpap.rs", f"DATA_{name}")), name)
        self.assertEqual((kp["KP_AP_KP"][-1], kp["KP_AP_AP"][-1], kp["DTM_AAP"][-1], kp["DTM_KP"][-1]), (9.0, 400.0, 657.0, 12.33))
        # the files the readers read are the MATLAB toolbox's own copies: the same bytes
        for f in ("SOLFSMY.TXT", "DTCFILE.TXT"):
            self.assertEqual((POP / "data" / "atmos_drag" / f).read_bytes(),
                             (ROOT / "matlab_sils" / "pop" / "04_atmosphere" / "density_models" / "jb2008" / f).read_bytes(), f)

    def test_the_ephemeris_flies_the_designs_slice(self):
        """S7.3c: DE440's records of the Sun, the Earth-Moon barycentre, the Earth and the Moon over the runs' span are
        env's data (env_de440_slice), read from de440s.bsp by the DAF/SPK reader; the precision orbit flies them
        generated (gen::de440slice), the same 64 bits; each record is the kernel's, word for word, and the slice covers
        every epoch a run or a campaign takes (2027-01-01 06:00 UTC plus up to 365 days and a scenario's 28,700 s)."""
        d = self.module("env_de440_slice")
        gen_pop = POP / "src" / "gen"
        for name in d:
            rows = rust_table(gen_pop / "de440slice.rs", f"DATA_{name}")
            self.assertEqual(bits(d[name]), bits(x for r in rows for x in r), name)
        segs = [d["DE440_SEGMENTS"][9 * i:9 * i + 9] for i in range(4)]
        self.assertEqual([(s[0], s[1]) for s in segs], [(0.0, 10.0), (0.0, 3.0), (3.0, 399.0), (3.0, 301.0)])
        words, _segs = readers.daf_spk(ROOT / "matlab_sils/pop/03_frames_time/ephemeris/data/de440s.bsp")
        for (ctr, tgt, name), s in zip(readers.DE440_SEGMENTS, segs):
            init, intlen, rsize, nk, k0 = s[2], s[3], int(s[4]), int(s[8]), int(s[7])
            self.assertEqual(len(d[name]), nk * rsize, name)
            sa = next(x[4] for x in _segs if (x[0], x[1]) == (ctr, tgt))
            self.assertEqual(bits(d[name]), bits(words[sa - 1 + k0 * rsize:sa - 1 + (k0 + nk) * rsize]), name)
            # the runs' span, in TDB seconds past J2000 (TDB - UTC is 69.18 s in 2027: a minute's margin is ample)
            first, last = (2461406.75 - 2451545.0) * 86400.0 - 60.0, (2461406.75 + 365.0 - 2451545.0) * 86400.0 + 28700.0 + 70.0
            self.assertLessEqual(init + k0 * intlen, first, name)
            self.assertGreaterEqual(init + (k0 + nk) * intlen, last, name)
        self.assertEqual(sum(len(v) for v in d.values()), 11020)

    def test_gravity_and_tides_fly_the_designs_tables(self):
        """S7.3d: the default field (EGM's GM, radius and zonals J2..J6), the ocean tides' eight main lines and FES2004
        to degree 10 are env's data, read by their readers (matlab_matrix, matlab_cell, fes_bin) from the MATLAB the
        Rust tables were ported from and from refgen's export of fes2004_deg10.mat; the precision orbit flies them
        generated (gen::gravfield, gen::tidelines), the same 64 bits."""
        gen_pop = POP / "src" / "gen"
        gf = self.module("env_gravity_default_field")
        self.assertEqual(gf, {"GRAV_DEFAULT_GM": [3.986004415e14], "GRAV_DEFAULT_RE": [6378136.3],
                              "GRAV_DEFAULT_J": [1.08262668355e-3, -2.53265648533e-6, -1.61962159137e-6, -2.27296082869e-7, 5.40681239107e-7]})
        for name in gf:
            self.assertEqual(bits(gf[name]), bits(rust_table(gen_pop / "gravfield.rs", f"DATA_{name}")), name)
        td = self.module("env_ocean_tide_tables")
        self.assertEqual({k: len(v) for k, v in td.items()}, {"OCEAN_MAIN_LINES": 8 * 10, "FES2004": 1052 * 12})
        for name in td:
            self.assertEqual(bits(td[name]), bits(x for r in rust_table(gen_pop / "tidelines.rs", f"DATA_{name}") for x in r), name)
        # M2: Doodson 2 0 0 0 0 0, degree 2, order 2, C+ -3.10e-11, S+ 0.40e-11
        self.assertEqual(td["OCEAN_MAIN_LINES"][:10], [2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 2.0, -3.1, 0.4])
        # FES2004's columns hold signed zeros: they come through as the export wrote them
        raw = (POP / "data" / "gravity_tides" / "fes2004_deg10.bin").read_bytes()
        n, ln = struct.unpack_from("<II", raw, 8)
        cp = struct.unpack_from(f"<{n}d", raw, 16 + ln + 8 * n)
        self.assertEqual(bits(td["FES2004"][8::12]), bits(cp))

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
            rid = self.rev_of[d["node"]]
            self.assertTrue(any(w.startswith(f"the developer's revision {rid}") and "not yet signed by a person" in w for w in x["why"]), d["node"])
        self.assertEqual(tomllib.loads((ROOT / "design" / "revisions_2_0.toml").read_text())["schema"], "trinetra-revisions/1")


if __name__ == "__main__":
    unittest.main()
