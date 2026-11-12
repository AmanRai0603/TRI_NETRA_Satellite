"""IGRF-14 in the design (docs/S12_INVENTORY.md S12.2; DR-env-001): the committed IAGA file matlab_sils/data/igrf14coeffs.txt
(pyIGRF14 1.0.4) is checked against a second, independently produced copy of the same model, the .shc file of ppigrf 2.1.0
(tests/data/IGRF14.shc), each parsed here by its own parser, not by tools/readers.py. Every coefficient of every epoch
must be the same decimal; the 2030.0 epoch (the .shc holds it as a column) must be 2025.0 + 5 x the 2025-30 secular
variation of the text file; and the reader (tools/readers.py igrf14) must give the same numbers as the .shc.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import pathlib
import tempfile
import unittest
from decimal import Decimal

import _path  # puts tools/ on the import path
import readers

_ = _path
ROOT = pathlib.Path(__file__).resolve().parents[1]
TXT = ROOT / "matlab_sils" / "data" / "igrf14coeffs.txt"
SHC = ROOT / "tests" / "data" / "IGRF14.shc"


def parse_shc(path):
    """ppigrf's .shc: '#' comments, a line `min_degree max_degree n_epochs spline_order n_knots first last`, the epochs,
    then a row per coefficient `n m v1 ... vE` (m < 0: h, m >= 0: g). -> (epochs, {(n, m): [Decimal per epoch]})."""
    lines = [ln for ln in pathlib.Path(path).read_text().splitlines() if ln.strip() and not ln.startswith("#")]
    head = lines[0].split()
    n_epochs = int(head[2])
    epochs = [Decimal(x) for x in lines[1].split()]
    table = {}
    for ln in lines[2:]:
        t = ln.split()
        table[(int(t[0]), int(t[1]))] = [Decimal(x) for x in t[2:]]
    assert len(epochs) == n_epochs
    return epochs, table


def parse_txt(path):
    """IAGA's text file: `g/h n m` rows, one value per epoch column, then the 2025-30 secular variation (the last column).
    -> (epochs incl. the SV column's label dropped, {(n, m): [Decimal per column]}) with m < 0 for h, as the .shc."""
    epochs, table = None, {}
    for ln in pathlib.Path(path).read_text().splitlines():
        t = ln.split()
        if ln.startswith("g/h"):
            epochs = [Decimal(x) for x in t[3:-1]]
        elif t and t[0] in ("g", "h"):
            n, m = int(t[1]), int(t[2])
            table[(n, m if t[0] == "g" else -m)] = [Decimal(x) for x in t[3:]]
    return epochs, table


def differences(txt, shc):
    """Every disagreement between the two copies: [(what, ...)]; empty when every coefficient of every epoch is equal."""
    e_t, t_t = parse_txt(txt)
    e_s, t_s = parse_shc(shc)
    bad = []
    if e_s != e_t + [Decimal("2030.0")]:
        bad.append(("epochs", e_t, e_s))
    if sorted(t_t) != sorted(t_s):
        bad.append(("coefficient set", len(t_t), len(t_s)))
        return bad
    for k in sorted(t_t):
        row_t, row_s = t_t[k], t_s[k]
        if len(row_t) != len(e_t) + 1 or len(row_s) != len(e_s):
            bad.append(("row length", k, len(row_t), len(row_s)))
            continue
        for j, y in enumerate(e_t):
            if row_t[j] != row_s[j]:
                bad.append((k, str(y), row_t[j], row_s[j]))
        # the 2030.0 epoch of the .shc is the text file's 2025.0 plus five years of its 2025-30 secular variation
        if row_t[-2] + 5 * row_t[-1] != row_s[-1]:
            bad.append((k, "2030.0", row_t[-2] + 5 * row_t[-1], row_s[-1]))
    return bad


class Igrf14(unittest.TestCase):
    def test_the_two_copies_agree_on_every_coefficient_of_every_epoch(self):
        e_t, t_t = parse_txt(TXT)
        self.assertEqual(len(e_t), 26)
        self.assertEqual(len(t_t), 195)
        self.assertEqual(differences(TXT, SHC), [])
        # the 2025.0 epoch's first coefficients (IGRF-14 g10, g11, h11), as the file states them
        self.assertEqual(t_t[(1, 0)][-2], Decimal("-29350.0"))
        self.assertEqual((t_t[(1, 1)][-2], t_t[(1, -1)][-2]), (Decimal("-1410.3"), Decimal("4545.5")))

    def test_the_comparison_fails_when_one_coefficient_changes(self):
        """The check can fail: one coefficient of the text file changed in the last printed digit, at any epoch (a DGRF
        one, the 2025.0 one, the secular variation), is reported."""
        text = TXT.read_text()
        lines = text.splitlines(keepends=True)
        i = next(k for k, ln in enumerate(lines) if ln.startswith("g  5  3"))
        for col in (3, 10, 28, 29):           # 1900.0, 1945.0, 2025.0, the 2025-30 SV
            t = lines[i].split()
            v = t[col]
            t[col] = v[:-1] + str((int(v[-1]) + 1) % 10)
            bad = lines[:]
            bad[i] = "  ".join(t) + "\n"
            with tempfile.TemporaryDirectory() as tmp:
                f = pathlib.Path(tmp) / "igrf14coeffs.txt"
                f.write_text("".join(bad))
                d = differences(f, SHC)
            self.assertTrue(d, f"column {col}: a changed coefficient was not found")

    def test_the_reader_gives_the_shc_numbers(self):
        got = readers.igrf14(TXT)
        epochs, shc = parse_shc(SHC)
        self.assertEqual(got["IGRF14_YEAR"][1], [float(y) for y in epochs])
        # the file's order: g(n,0), then g(n,m), h(n,m) for m = 1..n
        order = [(n, s * m) for n in range(1, 14) for m in range(0, n + 1) for s in ((1,) if m == 0 else (1, -1))]
        gh = got["IGRF14_GH"][1]
        self.assertEqual(len(gh), 27)
        for j in range(27):
            self.assertEqual(gh[j], [float(shc[k][j]) for k in order], f"epoch {epochs[j]}")
        self.assertEqual(len(got["IGRF14_SV"][1]), 195)
        self.assertEqual(gh[25][0], -29350.0)       # g10 at 2025.0, the number the S12 inventory names
        self.assertEqual(gh[26][0], -29287.0)       # 2025.0 + 5 x 12.6

    def test_the_design_carries_the_file_and_writes_it_back(self):
        """matlab_sils/data is generated from the design (tools/from_design.py): the committed file is the node's own table."""
        import sqlite3
        with sqlite3.connect(f"file:{ROOT / 'tests' / 'regression' / 'design.tndb'}?mode=ro", uri=True) as c:
            body = c.execute('SELECT "body" FROM engine_input WHERE "path" = ?', ("data/igrf14coeffs.txt",)).fetchone()[0]
        self.assertEqual(bytes(body), TXT.read_bytes())

    def test_the_revision_cites_its_files_and_version(self):
        import convert_2_0
        d = next(d for r in convert_2_0.revisions() if r["id"] == "S12.2" for d in r["data"] if d["node"] == "env_igrf14_coefficients")
        self.assertEqual(d["reader"], "igrf14")
        self.assertEqual(d["file"], "matlab_sils/data/igrf14coeffs.txt")
        for s in ("IAGA", "IGRF-14", "pyIGRF14 1.0.4", "ppigrf 2.1.0", "10.5281/zenodo.14012302"):
            self.assertIn(s, d["publication"])


if __name__ == "__main__":
    unittest.main()
