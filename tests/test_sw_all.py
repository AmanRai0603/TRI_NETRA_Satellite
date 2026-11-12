"""The dated space-weather table in the design (docs/S12_INVENTORY.md S12.2; DR-env-004): env_sw_all_table holds what
tools/readers.py sw_all reads from the committed CelesTrak file (engine/crates/adcs-pop/data/atmos_drag/SW-All.txt, the copy bundled in
the PyPI package spaceweather 0.4.2), from 1 January 2020 to the file's end. The file is parsed here a second way (by its
whitespace-separated tokens, where the reader uses the fixed columns of its FORMAT line); the two must agree on every number of
every row, the row counts must equal the file's, and the storm of 10-11 May 2024 must be the file's.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import datetime
import json
import pathlib
import re
import shutil
import sqlite3
import struct
import tempfile
import unittest

import _path  # puts tools/ on the import path
import readers

_ = _path
ROOT = pathlib.Path(__file__).resolve().parents[1]
REG = ROOT / "tests" / "regression" / "design.tndb"
FILE = ROOT / "engine" / "crates" / "adcs-pop" / "data" / "atmos_drag" / "SW-All.txt"
BLOCKS = {"OBSERVED": 0.0, "DAILY_PREDICTED": 1.0, "MONTHLY_PREDICTED": 2.0}


def bits(x):
    return struct.pack("<d", x)


def file_rows(path, first_year=2020):
    """The file by its tokens: ({0: observed, 1: daily predicted, 2: monthly predicted} -> [(date, row of numbers)], header counts)."""
    out = {0.0: [], 1.0: [], 2.0: []}
    counts, block = {}, None
    for line in pathlib.Path(path).read_text().splitlines():
        m = re.match(r"NUM_(\w+)_POINTS\s+(\d+)", line)
        if m:
            counts[m.group(1)] = int(m.group(2))
        elif line.startswith("BEGIN "):
            block = line.split()[1]
        elif line.startswith("END "):
            block = None
        elif block and line.strip():
            t = line.split()
            y, mo, d = int(t[0]), int(t[1]), int(t[2])
            if y < first_year:
                continue
            if block == "MONTHLY_PREDICTED":
                # year month day BSRN ND, then (no Kp, Ap, Cp, C9) ISN, F10.7 adj, ctr81, lst81, F10.7 obs, ctr81, lst81
                assert len(t) == 5 + 7, line
                out[2.0].append(((y, mo, d), [float(x) for x in (t[:3] + [2] + t[6:])]))
                continue
            # year month day BSRN ND Kp x8, sum, Ap x8, avg, Cp, C9, ISN, F10.7 adj, [Q (observed only)], ctr81, lst81, obs, ctr81, lst81
            q = 1 if block == "OBSERVED" else 0
            assert len(t) == 5 + 8 + 1 + 8 + 1 + 3 + 1 + q + 5, (block, line)
            kp, ap, avg = t[5:13], t[14:22], t[22]
            f = t[26:27] + t[27 + q:32 + q]
            out[BLOCKS[block]].append(((y, mo, d), [float(x) for x in t[:3] + [BLOCKS[block]] + kp + ap + [avg] + f]))
    return out, counts


def design_rows():
    with sqlite3.connect(f"file:{REG}?mode=ro", uri=True) as c:
        (x,) = [json.loads(v) for (v,) in c.execute("SELECT content FROM design_node WHERE id = ?", ("env_sw_all_table",))]
    body = json.loads(x["body"]) if isinstance(x["body"], str) else x["body"]
    (text,) = [v for sec, field, v, _o in body["content"] if sec == "data"]
    out = {}
    for name, cols, rows in re.findall(r"^data (\w+): real\[1\]\[(\d+)\]\[\d+\]\n(.*?)^end$", text, re.S | re.M):
        flat = [float(t) for t in re.findall(r"[-+0-9.eE]+", rows)]
        out[name] = [flat[i:i + int(cols)] for i in range(0, len(flat), int(cols))]
    return out


def differences(path, daily, monthly):
    """Every disagreement between the file (by tokens) and the two tables; empty when they are the same."""
    got, _c = file_rows(path)
    want_d = [r for _d, r in got[0.0] + got[1.0]]
    want_m = [r for _d, r in got[2.0]]
    bad = []
    for name, want, have in (("SW_DAILY", want_d, daily), ("SW_MONTHLY", want_m, monthly)):
        if len(want) != len(have):
            bad.append((name, "rows", len(want), len(have)))
            continue
        for k, (a, b) in enumerate(zip(want, have)):
            if [bits(x) for x in a] != [bits(x) for x in b]:
                bad.append((name, k, a, b))
    return bad


class SwAll(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.t = design_rows()
        cls.file, cls.counts = file_rows(FILE)

    def test_the_design_table_is_the_file_row_for_row(self):
        self.assertEqual(sorted(self.t), ["SW_DAILY", "SW_MONTHLY"])
        self.assertEqual(differences(FILE, self.t["SW_DAILY"], self.t["SW_MONTHLY"]), [])

    def test_the_row_counts_are_the_files(self):
        # by a scan of the lines in the file's blocks from 2020, and by the file's own header counts less its rows before 2020
        text = FILE.read_text().splitlines()
        n = {k: 0 for k in BLOCKS}
        pre = {k: 0 for k in BLOCKS}
        block = None
        for ln in text:
            if ln.startswith("BEGIN "):
                block = ln.split()[1]
            elif ln.startswith("END "):
                block = None
            elif block and ln.strip():
                if int(ln[:4]) >= 2020:
                    n[block] += 1
                else:
                    pre[block] += 1
        self.assertEqual({k: self.counts.get(k) for k in BLOCKS}, {k: n[k] + pre[k] for k in BLOCKS})
        self.assertEqual((n["OBSERVED"], n["DAILY_PREDICTED"], n["MONTHLY_PREDICTED"]), (2028, 39, 194))
        self.assertEqual(len(self.t["SW_DAILY"]), n["OBSERVED"] + n["DAILY_PREDICTED"])
        self.assertEqual(len(self.t["SW_MONTHLY"]), n["MONTHLY_PREDICTED"])
        self.assertEqual(sum(1 for r in self.t["SW_DAILY"] if r[3] == 0.0), n["OBSERVED"])
        self.assertEqual(sum(1 for r in self.t["SW_DAILY"] if r[3] == 1.0), n["DAILY_PREDICTED"])
        self.assertEqual(sum(1 for r in self.t["SW_MONTHLY"] if r[3] == 2.0), n["MONTHLY_PREDICTED"])

    def test_the_span_is_2020_to_the_files_end_without_a_gap(self):
        days = [datetime.date(int(r[0]), int(r[1]), int(r[2])) for r in self.t["SW_DAILY"]]
        self.assertEqual((days[0], days[-1]), (datetime.date(2020, 1, 1), datetime.date(2025, 8, 28)))
        self.assertEqual([(b - a).days for a, b in zip(days, days[1:])], [1] * (len(days) - 1))
        obs = [d for d, r in zip(days, self.t["SW_DAILY"]) if r[3] == 0.0]
        self.assertEqual(obs[-1], datetime.date(2025, 7, 20))
        mon = [(int(r[0]), int(r[1]), int(r[2])) for r in self.t["SW_MONTHLY"]]
        self.assertEqual((mon[0], mon[-1]), ((2025, 9, 1), (2041, 10, 1)))
        self.assertTrue(all(d == 1 for _y, _m, d in mon))
        # the file's last monthly line, by its tokens: year month day BSRN ND ISN, then the six F10.7 values
        last = [ln for ln in FILE.read_text().splitlines() if ln.startswith("2041 10 01")][0].split()
        self.assertEqual(self.t["SW_MONTHLY"][-1][4:], [float(x) for x in last[6:]])

    def test_the_storm_of_10_and_11_may_2024_is_the_files(self):
        rows = {(int(r[0]), int(r[1]), int(r[2])): r for r in self.t["SW_DAILY"]}
        # what the file says, from its own lines
        raw = {ln.split()[2]: ln.split() for ln in FILE.read_text().splitlines() if ln.startswith("2024 05 1") and ln[8:10] in ("10", "11")}
        for day in ("10", "11"):
            t = raw[day]
            r = rows[(2024, 5, int(day))]
            self.assertEqual(r[3], 0.0)                                   # observed
            self.assertEqual(r[4:12], [float(x) for x in t[5:13]])          # Kp x 10
            self.assertEqual(r[12:20], [float(x) for x in t[14:22]])        # the eight 3-hour Ap
            self.assertEqual(r[20], float(t[22]))                          # the daily Ap
        # the numbers the file holds: on 11 May the eight 3-hour Ap are 400, 236, 236, 400, 300, 236, 179 and 179 and the daily
        # Ap is 271; on 10 May the daily Ap is 105 and the last three 3-hour values are 179, 300, 300
        self.assertEqual(rows[(2024, 5, 11)][12:21], [400.0, 236.0, 236.0, 400.0, 300.0, 236.0, 179.0, 179.0, 271.0])
        self.assertEqual(rows[(2024, 5, 10)][12:21], [12.0, 12.0, 9.0, 7.0, 22.0, 179.0, 300.0, 300.0, 105.0])
        self.assertEqual(rows[(2024, 5, 11)][4:12], [90.0, 83.0, 83.0, 90.0, 87.0, 83.0, 77.0, 77.0])
        # F10.7 adjusted, its centred and last-81-day means, then the observed ones
        self.assertEqual(rows[(2024, 5, 11)][21:], [218.0, 180.5, 163.6, 213.7, 177.1, 163.7])

    def test_the_comparison_fails_when_one_number_changes(self):
        """The check can fail: a digit changed in one row of the file (a Kp, an Ap, an F10.7, a monthly F10.7) is reported."""
        for prefix, col in (("2024 05 11", 20), ("2024 05 11", 9), ("2025 07 21", 100), ("2033 04 01", 97)):
            lines = FILE.read_text().splitlines(keepends=True)
            i = next(k for k, ln in enumerate(lines) if ln.startswith(prefix))
            ln = lines[i]
            j = max(k for k in range(min(col, len(ln.rstrip()) - 1) + 1) if ln[k].isdigit())
            lines[i] = ln[:j] + str((int(ln[j]) + 1) % 10) + ln[j + 1:]
            with tempfile.TemporaryDirectory() as tmp:
                f = pathlib.Path(tmp) / "SW-All.txt"
                f.write_text("".join(lines))
                d = differences(f, self.t["SW_DAILY"], self.t["SW_MONTHLY"])
            self.assertEqual(len(d), 1, (prefix, col, d))
        # a row dropped from the file is a count that no longer agrees, and the reader refuses a header that disagrees with its blocks
        with tempfile.TemporaryDirectory() as tmp:
            lines = FILE.read_text().splitlines(keepends=True)
            i = next(k for k, ln in enumerate(lines) if ln.startswith("2024 05 11"))
            f = pathlib.Path(tmp) / "SW-All.txt"
            f.write_text("".join(lines[:i] + lines[i + 1:]))
            self.assertTrue(any(x[1] == "rows" for x in differences(f, self.t["SW_DAILY"], self.t["SW_MONTHLY"])))
            with self.assertRaises(SystemExit):
                readers.sw_all(f, 2020)

    def test_the_reader_is_the_one_the_revision_names(self):
        import convert_2_0
        d = next(d for r in convert_2_0.revisions() if r["id"] == "S12.2" for d in r["data"] if d["node"] == "env_sw_all_table")
        got = readers.read({"reader": d["reader"], **d})
        self.assertEqual(sorted(got), ["SW_DAILY", "SW_MONTHLY"])
        for name, (_ty, rows) in got.items():
            self.assertEqual([bits(x) for r in rows for x in r], [bits(x) for r in self.t[name] for x in r], name)
        for s in ("CelesTrak", "T.S. Kelso", "spaceweather 0.4.2", "UPDATED 2025 Jul 21 10:37:15 UTC", "VERSION 1.2"):
            self.assertIn(s, d["publication"])


if __name__ == "__main__":
    unittest.main()
