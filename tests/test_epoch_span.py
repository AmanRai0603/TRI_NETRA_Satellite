"""The epoch spans the design's data is valid for (docs/S12_INVENTORY.md S12.4, E14; DR-env-002), two stated nodes:
  - env_epoch_span (truth) is 2025.0 to the earliest of 2030.0, Leap_Second.dat's expiry and finals2000A.all's last day,
    computed here from the other stated nodes; the engine and the twin refuse the case's STATED mission.epoch outside it by
    name (cargo test -p adcs-sim --test inputs; the twin's t_epoch_span). A season shift or a dispersed epoch past it is not
    refused: the run warns and records (frame_record.rs); the sizing's survey and the campaigns' seasons pass the expiry;
  - env_flight_epoch_span (flight) is 2025.0 to 2030.0, the span of the flight's own data; the flight software's rule for jd0
    (fsw/params/params.toml, span = "env_flight_epoch_span") is written into the C and the Rust checks with those two numbers,
    and nothing else holds them (the parameter table names the node, not the dates).

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import json
import pathlib
import re
import tomllib
import unittest

import _path  # puts tools/ on the import path

_ = _path
ROOT = pathlib.Path(__file__).resolve().parents[1]
STATED = json.loads((ROOT / "matlab_sils" / "data" / "stated.json").read_text())["value"]
MJD_TO_JD = 2400000.5
MJD_2025_0, MJD_2030_0 = 60676.0, 62502.0         # 2025-01-01 and 2030-01-01 0 h UTC


class EpochSpan(unittest.TestCase):
    def test_the_truth_span_is_2025_to_the_earliest_expiry(self):
        finals, leap = STATED["env_eop_finals_expiry_mjd"], STATED["env_leap_second_expiry_mjd"]
        last = min(MJD_2030_0, finals, leap)
        self.assertEqual(STATED["env_epoch_span"], [MJD_2025_0 + MJD_TO_JD, last + MJD_TO_JD])
        self.assertEqual(last, leap, "Leap_Second.dat's expiry limits the truth span today")

    def test_the_flight_span_is_the_flights_own_data(self):
        self.assertEqual(STATED["env_flight_epoch_span"], [MJD_2025_0 + MJD_TO_JD, MJD_2030_0 + MJD_TO_JD])

    def test_the_flight_rule_names_the_node_and_the_checks_hold_its_numbers(self):
        f = next(x for x in tomllib.loads((ROOT / "fsw" / "params" / "params.toml").read_text())["field"] if x["name"] == "jd0")
        self.assertEqual(f.get("span"), "env_flight_epoch_span")
        self.assertNotIn("min", f)
        self.assertNotIn("max", f)
        lo, hi = STATED["env_flight_epoch_span"]
        for path, line in (("fsw/src/adcs_params.c", r"p->jd0 < {lo} \|\| p->jd0 > {hi}"), ("fsw-rs/src/params.rs", r"self\.jd0 < {lo} \|\| self\.jd0 > {hi}")):
            text = (ROOT / path).read_text()
            self.assertRegex(text, line.format(lo=re.escape(repr(lo)), hi=re.escape(repr(hi))), path)

    def test_the_parameter_table_holds_no_date(self):
        text = (ROOT / "fsw" / "params" / "params.toml").read_text()
        self.assertNotIn("2460676", text)
        self.assertNotIn("2461584", text)
        self.assertNotIn("2462502", text)


if __name__ == "__main__":
    unittest.main()
