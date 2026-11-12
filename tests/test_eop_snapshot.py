"""The EOP and leap-second snapshot in the design (docs/S12_INVENTORY.md S12.2; DR-env-002): env_eop_snapshot holds the
tables tools/readers.py reads from the committed IERS files (finals2000A.all and Leap_Second.dat of astropy-iers-data
0.2026.11.9.0.35.9), and its stated nodes (the snapshot date, the two expiries) agree with the files.
  - the engine's own reader of those files (adcs-pop eop.rs parse_finals and parse_leap, run by `cargo test -p adcs-pop --test
    eop_snapshot`) reads, for every day of the span, what the design's tables hold (Bulletin B where the day has it, else A;
    dX and dY in arcsec, 0 when the file has none), bit for bit, and the leap-second rows;
  - the row counts and the span equal what a plain scan of the file gives; the stated expiries are the files' last day of data
    and the header's date.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import datetime
import json
import pathlib
import re
import sqlite3
import struct
import subprocess
import unittest

import _path  # puts tools/ on the import path
import readers

_ = _path
ROOT = pathlib.Path(__file__).resolve().parents[1]
REG = ROOT / "tests" / "regression" / "design.tndb"
DATA = ROOT / "engine" / "crates" / "adcs-pop" / "data" / "time_frames"
FINALS = DATA / "finals2000A.all"
LEAP = DATA / "Leap_Second.dat"
MJD0, MJD1 = 60676.0, 62502.0          # 2025.0 and 2030.0


def mjd_of(y, m, d):
    return float((datetime.date(y, m, d) - datetime.date(1858, 11, 17)).days)


def bits(x):
    return struct.pack("<d", x)


def design_tables(nid):
    """{name: [rows]} of the data blocks of a node of the regression copy, each row a list of its columns."""
    with sqlite3.connect(f"file:{REG}?mode=ro", uri=True) as c:
        (x,) = [json.loads(v) for (v,) in c.execute("SELECT content FROM design_node WHERE id = ?", (nid,))]
    body = json.loads(x["body"]) if isinstance(x["body"], str) else x["body"]
    (text,) = [v for sec, field, v, _o in body["content"] if sec == "data"]
    out = {}
    for name, cols, rows in re.findall(r"^data (\w+): real\[1\]\[(\d+)\](?:\[\d+\])?\n(.*?)^end$", text, re.S | re.M):
        flat = [float(t) for t in re.findall(r"[-+0-9.eE]+", rows)]
        out[name] = [flat[i:i + int(cols)] for i in range(0, len(flat), int(cols))]
    return out


def stated(nid):
    with sqlite3.connect(f"file:{REG}?mode=ro", uri=True) as c:
        (x,) = [json.loads(v) for (v,) in c.execute("SELECT content FROM design_node WHERE id = ?", (nid,))]
    body = json.loads(x["body"]) if isinstance(x["body"], str) else x["body"]
    return {f"{sec}.{field}": v for sec, field, v, _o in body["content"]}


def engine_reads():
    r = subprocess.run(["cargo", "test", "-p", "adcs-pop", "--test", "eop_snapshot", "--", "--nocapture"], cwd=ROOT / "engine",
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stdout[-2000:] + r.stderr[-2000:]
    (line,) = [ln for ln in r.stdout.splitlines() if ln.startswith("ENGINE_EOP_JSON ")]
    return json.loads(line[len("ENGINE_EOP_JSON "):])


def engine_rows_from_design(t):
    """The rows the engine's parse_finals makes from the design's tables, by its rule (eop.rs): a day with Bulletin A values
    takes Bulletin B's x, y, UT1 - UTC, dX, dY when it has them, else A's; dX and dY in arcsec, 0 where the file has none."""
    nut = {r[0]: r for r in t["FINALS_A_NUT"]}
    b = {r[0]: r for r in t["FINALS_B"]}
    out = []
    for m, _pf, xa, ya, _uf, ua in t["FINALS_A_PM_UT1"]:
        if m in b:
            _, x, y, u, dx, dy = b[m]
        else:
            x, y, u = xa, ya, ua
            dx, dy = (nut[m][2], nut[m][3]) if m in nut else (0.0, 0.0)
        out.append((m, x, y, u, dx / 1000.0, dy / 1000.0))
    return out


def differences(eng, rows, leap_design):
    """Every disagreement between what the engine read (over the span) and the design's rows; empty when they are the same."""
    sel = [i for i, m in enumerate(eng["mjd"]) if MJD0 <= m <= MJD1]
    bad = []
    if len(sel) != len(rows):
        bad.append(("rows", len(sel), len(rows)))
        return bad
    for i, row in zip(sel, rows):
        got = (eng["mjd"][i], eng["xp"][i], eng["yp"][i], eng["dut1"][i], eng["dx"][i], eng["dy"][i])
        if [bits(a) for a in got] != [bits(a) for a in row]:
            bad.append((row[0], got, row))
    if eng["leap_mjd"] != [r[0] for r in leap_design] or eng["leap_dat"] != [r[4] for r in leap_design]:
        bad.append(("leap rows",))
    return bad


class EopSnapshot(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.t = design_tables("env_eop_snapshot")
        cls.eng = engine_reads()

    def test_the_engines_reader_reads_what_the_design_holds(self):
        rows = engine_rows_from_design(self.t)
        self.assertEqual(len(rows), 1039)
        self.assertEqual(differences(self.eng, rows, self.t["LEAP_DAT"]), [])

    def test_the_comparison_fails_when_one_value_changes(self):
        rows = engine_rows_from_design(self.t)
        for col in range(1, 6):
            i = 500 + col
            bad = [list(r) for r in rows]
            bad[i][col] = bad[i][col] + 1e-7
            d = differences(self.eng, [tuple(r) for r in bad], self.t["LEAP_DAT"])
            self.assertEqual([x[0] for x in d], [rows[i][0]], f"column {col}")
        leap = [list(r) for r in self.t["LEAP_DAT"]]
        leap[27][4] += 1.0
        self.assertTrue(differences(self.eng, rows, leap))
        self.assertTrue(differences(self.eng, rows[:-1], self.t["LEAP_DAT"]))

    def test_the_design_tables_are_the_files_rows_in_the_span(self):
        """Counts and span from a plain scan of the file (not the reader): a day is in the table when its columns hold a value."""
        lines = [ln.ljust(187) for ln in FINALS.read_text().splitlines()]
        span = [ln for ln in lines if MJD0 <= float(ln[7:15]) <= MJD1]
        self.assertEqual(len(self.t["FINALS_A_PM_UT1"]), sum(1 for ln in span if ln[18:27].strip()))
        self.assertEqual(len(self.t["FINALS_A_NUT"]), sum(1 for ln in span if ln[97:106].strip()))
        self.assertEqual(len(self.t["FINALS_B"]), sum(1 for ln in span if ln[134:144].strip()))
        self.assertEqual((len(self.t["FINALS_A_PM_UT1"]), len(self.t["FINALS_A_NUT"]), len(self.t["FINALS_B"])), (1039, 744, 609))
        pm = self.t["FINALS_A_PM_UT1"]
        self.assertEqual((pm[0][0], pm[-1][0]), (60676.0, 61714.0))
        self.assertEqual([r[0] for r in pm], [60676.0 + i for i in range(1039)])      # a row every day, none missing
        self.assertTrue(all(MJD0 <= r[0] <= MJD1 for t in ("FINALS_A_NUT", "FINALS_B") for r in self.t[t]))
        self.assertTrue(set(r[0] for r in self.t["FINALS_B"]) <= set(r[0] for r in pm))
        # the file goes past the last day with values (blank rows to MJD 61764), and ends before the span's end
        self.assertEqual(float(lines[-1][7:15]), 61764.0)
        self.assertTrue(all(not ln[18:27].strip() for ln in lines if float(ln[7:15]) > 61714.0))
        # the first row of the span, as the file prints it
        self.assertEqual(pm[0], [60676.0, 0.0, 0.144063, 0.305108, 0.0, 0.0462673])

    def test_the_stated_dates_are_the_files(self):
        self.assertEqual(float(stated("env_eop_finals_expiry_mjd")["value.number"]), self.t["FINALS_A_PM_UT1"][-1][0])
        hdr = next(ln for ln in LEAP.read_text().splitlines() if "File expires on" in ln)
        self.assertEqual(hdr.strip(), "#  File expires on 28 June 2027")
        self.assertEqual(self.t["LEAP_DAT_EXPIRY"], [[2027.0, 6.0, 28.0, mjd_of(2027, 6, 28)]])
        self.assertEqual(float(stated("env_leap_second_expiry_mjd")["value.number"]), mjd_of(2027, 6, 28))
        # the snapshot is dated by its package's version, 0.2026.11.9.0.35.9
        self.assertEqual(float(stated("env_eop_snapshot_mjd")["value.number"]), mjd_of(2026, 11, 9))
        self.assertIn("0.2026.11.9.0.35.9", stated("env_eop_snapshot_mjd")["value.source"])

    def test_the_leap_table_is_bulletin_c_and_agrees_with_the_older_one(self):
        leap = self.t["LEAP_DAT"]
        self.assertEqual(len(leap), 28)
        self.assertEqual((leap[0], leap[-1]), ([41317.0, 1.0, 1.0, 1972.0, 10.0], [57754.0, 1.0, 1.0, 2017.0, 37.0]))
        for mjd, d, m, y, _t in leap:
            self.assertEqual(mjd, mjd_of(int(y), int(m), int(d)))
        old = design_tables("env_leap_seconds")["LEAP_SECONDS"]       # year, month, TAI - UTC (the Octave POP's table)
        self.assertEqual([(r[3], r[2], r[4]) for r in leap], [tuple(r) for r in old])

    def test_the_reader_is_the_one_the_revision_names(self):
        import convert_2_0
        d = next(d for r in convert_2_0.revisions() if r["id"] == "S12.2" for d in r["data"] if d["node"] == "env_eop_snapshot")
        got = {}
        for one in d["files"]:
            got.update(readers.read({"reader": d["reader"], **one}))
        self.assertEqual(sorted(got), sorted(self.t))
        for name, (_ty, rows) in got.items():
            flat = [x for r in rows for x in (r if isinstance(r, list) else [r])]
            self.assertEqual([bits(x) for x in flat], [bits(x) for r in self.t[name] for x in r], name)
        for s in ("IERS", "astropy-iers-data 0.2026.11.9.0.35.9", "Bulletin 72", "BSD 3-Clause", "not stated in the file"):
            self.assertIn(s, d["publication"])


if __name__ == "__main__":
    unittest.main()
