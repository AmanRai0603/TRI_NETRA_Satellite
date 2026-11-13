"""IGRF-14 is the one table the code flies (docs/S12_INVENTORY.md IGRF-14, S12.6; DR-env-001): the engine's truth field, both flight
softwares (C and Rust) and the twin take env_igrf14_coefficients through the generators, and the dipole's constants are that
table's degree-1 row at a stated epoch, not a second copy.
  - every generated table (the engine's, the flight software's in Rust and in C, the twin's in its two packages, the relations'
    degree-1 slice in Rust and MATLAB) hashes equal to the design's: the same 64 bits, table by table;
  - g(1,0) at 2025.0 is the file's, in the design, in the pseudocode's igrf_gh at 2025.0 and in the dipole's B0;
  - IGRF-13 is flown nowhere: no generated file, no table, no name of it in the generated code;
  - igrf_gh interpolates 2025.0 to 2030.0 as IGRF-14 defines it (the model's own secular variation, not an extrapolation).

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import hashlib
import json
import math
import pathlib
import re
import sqlite3
import struct
import unittest

import _path  # puts tools/ on the import path
import from_design
import pcode
import readers
import test_igrf14

_ = _path
ROOT = pathlib.Path(__file__).resolve().parents[1]
REG = ROOT / "tests" / "regression" / "design.tndb"
TABLES = ("IGRF14_YEAR", "IGRF14_GH", "IGRF14_SV")
RUST = [ROOT / "engine/crates/adcs-sim-core/src/gen/igrf14coeffs.rs", ROOT / "fsw-rs/src/alg/igrf14coeffs.rs"]
C = ROOT / "fsw/alg/src/igrf14coeffs.c"
M = [ROOT / "matlab_sils/+asils/+alg/+igrf14coeffs", ROOT / "matlab_sils/+asils/+models/+igrf14coeffs"]
DIPOLE_RUST = ROOT / "engine/crates/adcs-relations/src/gen/igrf14dipole.rs"
DIPOLE_M = ROOT / "matlab_sils/+asils/+relations/+igrf14dipole/IGRF14_DEGREE1.m"


def sha(xs):
    """sha256 of a table's 64-bit values in order (big endian)."""
    return hashlib.sha256(b"".join(struct.pack(">d", float(x)) for x in xs)).hexdigest()


def numbers(text):
    return [float(x) for x in re.findall(r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?", text)]


def rust_table(path, name):
    m = re.search(rf"static DATA_{name}: [^=]+= \[(.*?)\n\];", pathlib.Path(path).read_text(), re.S)
    return numbers(m.group(1))


def c_table(path, name):
    m = re.search("igrf14coeffs_DATA_" + name + r" = (.*?)\n\} \};", pathlib.Path(path).read_text(), re.S)
    return numbers(m.group(1))


def m_table(path):
    """The values of a generated MATLAB table in the order the design lists them (a 2-D table row by row)."""
    body = re.search(r"T = \[(.*?)\n\s*\];", pathlib.Path(path).read_text(), re.S).group(1)
    return numbers(body)


def design_module(nid):
    with sqlite3.connect(f"file:{REG}?mode=ro", uri=True) as c:
        body = json.loads(c.execute("SELECT content FROM design_node WHERE id = ?", (nid,)).fetchone()[0])["body"]
    (text,) = [v for s, f, v, _o in body["content"] if s == "data"]
    return {name: numbers(b) for name, b in re.findall(r"^data (\w+): [^\n]+\n(.*?)^end$", text, re.S | re.M)}


class Flown(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.table = design_module("env_igrf14_coefficients")
        cls.dipole = design_module("env_igrf14_dipole")
        cls.epochs, cls.file = test_igrf14.parse_txt(test_igrf14.TXT)

    def file_g10(self):
        return float(self.file[(1, 0)][self.epochs.index(test_igrf14.Decimal("2025.0"))])

    def test_the_design_holds_the_file_to_the_bit(self):
        got = readers.igrf14(test_igrf14.TXT)
        for name in TABLES:
            flat = [x for r in got[name][1] for x in (r if isinstance(r, list) else [r])]
            self.assertEqual(sha(self.table[name]), sha(flat), name)
        self.assertEqual(len(self.table["IGRF14_YEAR"]), 27)
        self.assertEqual(len(self.table["IGRF14_GH"]), 27 * 195)

    def test_every_generated_table_hashes_equal_to_the_designs(self):
        for name in TABLES:
            want = sha(self.table[name])
            for where in RUST:
                self.assertEqual(sha(rust_table(where, name)), want, f"{where.relative_to(ROOT)} {name}")
            self.assertEqual(sha(c_table(C, name)), want, f"{C.relative_to(ROOT)} {name}")
            for pkg in M:
                self.assertEqual(sha(m_table(pkg / f"{name}.m")), want, f"{(pkg / name).relative_to(ROOT)}")
        want = sha(self.dipole["IGRF14_DEGREE1"])
        self.assertEqual(sha(rust_table(DIPOLE_RUST, "IGRF14_DEGREE1")), want)
        self.assertEqual(sha(m_table(DIPOLE_M)), want)

    def test_the_hash_check_can_fail(self):
        """One coefficient changed in the last bit is a different hash."""
        xs = list(self.table["IGRF14_GH"])
        ys = list(xs)
        ys[7] = math.nextafter(ys[7], math.inf)
        self.assertNotEqual(sha(xs), sha(ys))

    def test_igrf13_is_flown_nowhere(self):
        gen = [ROOT / "engine/crates/adcs-sim-core/src/gen", ROOT / "fsw-rs/src/alg", ROOT / "fsw/alg/src", ROOT / "fsw/alg/include",
               ROOT / "matlab_sils/+asils/+alg", ROOT / "matlab_sils/+asils/+models", ROOT / "matlab_sils/+asils/+relations",
               ROOT / "engine/crates/adcs-relations/src/gen", ROOT / "engine/crates/adcs-pop/src/gen", ROOT / "engine/crates/adcs-sim/src/gen"]
        for d in gen:
            for f in d.rglob("*"):
                if f.is_file():
                    self.assertNotIn("igrf13", f.name.lower(), f)
                    text = f.read_text(encoding="utf-8", errors="replace")
                    for word in ("DATA_IGRF_GH", "DATA_IGRF_YEAR", "D__IGRF_GH", "D__IGRF_YEAR", "igrf13"):
                        self.assertNotIn(word, text.replace("igrf-13", ""), f)
        self.assertFalse((ROOT / "fsw/pseudocode/02_igrf13.pc").exists())
        self.assertTrue((ROOT / "fsw/pseudocode/02_igrf14.pc").is_file())

    def test_g10_at_2025_is_the_files_in_the_design_the_pseudocode_and_the_dipole(self):
        g10 = self.file_g10()
        self.assertEqual(g10, -29350.0)          # the number the file states (the inventory's check; IGRF-13's 2025.0 was -29376.3)
        self.assertEqual(self.table["IGRF14_GH"][25 * 195], g10)
        self.assertEqual(self.dipole["IGRF14_DEGREE1"], [2025.0, g10, float(self.file[(1, 1)][self.epochs.index(test_igrf14.Decimal('2025.0'))]), float(self.file[(1, -1)][self.epochs.index(test_igrf14.Decimal('2025.0'))])])
        # the interpreter on the design's own pseudocode: the flight software's and the engine's coefficients at 2025.0
        frames = [ROOT / "fsw/pseudocode/02_time_frames_models.pc", ROOT / "fsw/pseudocode/02_igrf14.pc", ROOT / "fsw/pseudocode/01_math.pc"]
        gh = pcode.cli("run", *map(str, frames), "--fn", "igrf_gh", "--args", json.dumps([2025.0]))[0]
        self.assertEqual(gh[0], g10)
        # the dipole's strength is the file's degree-1 row: B0 = sqrt(g10^2 + g11^2 + h11^2), in tesla
        g11, h11 = float(self.file[(1, 1)][self.epochs.index(test_igrf14.Decimal('2025.0'))]), float(self.file[(1, -1)][self.epochs.index(test_igrf14.Decimal('2025.0'))])
        files = [str(f) for f in sorted(from_design.folder("spec/physics/").glob("*.pc"))]
        b0, tilt = pcode.cli("run", *files, "--fn", "dipole", "--args", "[]")
        self.assertAlmostEqual(b0 / (math.sqrt(g10 ** 2 + g11 ** 2 + h11 ** 2) * 1e-9), 1.0, delta=1e-14)
        self.assertAlmostEqual(tilt, math.acos(-g10 / math.sqrt(g10 ** 2 + g11 ** 2 + h11 ** 2)), delta=1e-14)

    def test_the_dipole_constants_are_kept_once(self):
        """No degree-1 coefficient typed in the relations: the constants library has no IGRF_* and the dipole file reads the table."""
        self.assertNotIn("IGRF_G10", from_design.text("spec/physics/constants.pc"))
        dip = from_design.text("spec/physics/dipole.pc")
        code = "\n".join(ln for ln in dip.splitlines() if not ln.startswith("##"))
        self.assertNotRegex(code, r"\d{3,}\.\d")      # no coefficient typed
        self.assertIn("IGRF14_DEGREE1", dip)

    def test_the_2025_to_2030_interval_is_the_models_own_interpolation(self):
        frames = [ROOT / "fsw/pseudocode/02_time_frames_models.pc", ROOT / "fsw/pseudocode/02_igrf14.pc", ROOT / "fsw/pseudocode/01_math.pc"]
        sv = self.table["IGRF14_SV"]
        for dy in (2025.0, 2026.5, 2027.0, 2029.999, 2030.0):
            gh = pcode.cli("run", *map(str, frames), "--fn", "igrf_gh", "--args", json.dumps([dy]))[0]
            for k in (0, 1, 2, 3, 100):
                want = self.table["IGRF14_GH"][25 * 195 + k] + sv[k] * (dy - 2025.0)
                self.assertAlmostEqual(gh[k], want, delta=1e-9, msg=f"{dy} {k}")


if __name__ == "__main__":
    unittest.main()
