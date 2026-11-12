"""The epoch span the design's data is valid for (docs/S12_INVENTORY.md S12.4, E14; DR-env-002): the stated env_epoch_span is
2025.0 to 2030.0 intersected with the data's expiry, and the two places that refuse an epoch outside it read it by name:
  - the stated list equals the intersection computed here from the other stated nodes: the first day of the EOP snapshot and
    of IGRF-14 (2025.0), the end of its span (2030.0) and env_eop_finals_expiry_mjd; the file that limits it is
    finals2000A.all's (its last day of values, 5 November 2027). Leap_Second.dat expires earlier (28 June 2027): it does not
    limit the span, a run past it warns and records, and the design's season survey (the case's epoch plus up to 273.9 days)
    is inside the span;
  - the flight software's rule for jd0 (fsw/params/params.toml, span = "env_epoch_span") is written into the C and the Rust
    checks with those two numbers, and nothing else holds them (the parameter table names the node, not the dates);
  - the engine and the twin refuse a mission epoch outside it by name (cargo test -p adcs-sim --test inputs; the twin's
    t_epoch_span).

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
    def test_the_span_is_2025_to_2030_intersected_with_the_expiry(self):
        finals, leap = STATED["env_eop_finals_expiry_mjd"], STATED["env_leap_second_expiry_mjd"]
        last = min(MJD_2030_0, finals)
        self.assertEqual(STATED["env_epoch_span"], [MJD_2025_0 + MJD_TO_JD, last + MJD_TO_JD])
        self.assertEqual(last, finals, "finals2000A.all's last day of values limits the span, before 2030.0")
        self.assertLess(leap, finals, "Leap_Second.dat expires inside the span: past it a run warns and records")

    def test_the_sizings_seasons_are_inside_the_span(self):
        """adcs-design's demand survey flies the case's epoch plus 0, 91.3, 182.6 and 273.9 days (sizedemand): a span that
        refused them would refuse `adcs design size`."""
        text = (ROOT / "engine" / "crates" / "adcs-design" / "src" / "gen" / "sizedemand.rs").read_text()
        days = [float(x) for x in re.search(r"let mut days: \[f64; 4\] = \[([^\]]+)\]", text).group(1).split(",")]
        lo, hi = STATED["env_epoch_span"]
        epoch = 2451545.0 + 27.0 * 365.25                      # the shipped cases: mission.epoch 27.0, years after J2000.0
        for d in days:
            self.assertTrue(lo <= epoch + d <= hi, f"season +{d} days: JD {epoch + d} is outside [{lo}, {hi}]")

    def test_the_flight_rule_names_the_node_and_the_checks_hold_its_numbers(self):
        f = next(x for x in tomllib.loads((ROOT / "fsw" / "params" / "params.toml").read_text())["field"] if x["name"] == "jd0")
        self.assertEqual(f.get("span"), "env_epoch_span")
        self.assertNotIn("min", f)
        self.assertNotIn("max", f)
        lo, hi = STATED["env_epoch_span"]
        for path, line in (("fsw/src/adcs_params.c", r"p->jd0 < {lo} \|\| p->jd0 > {hi}"), ("fsw-rs/src/params.rs", r"self\.jd0 < {lo} \|\| self\.jd0 > {hi}")):
            text = (ROOT / path).read_text()
            self.assertRegex(text, line.format(lo=re.escape(repr(lo)), hi=re.escape(repr(hi))), path)

    def test_the_parameter_table_holds_no_date(self):
        text = (ROOT / "fsw" / "params" / "params.toml").read_text()
        self.assertNotIn("2460676", text)
        self.assertNotIn("2461584", text)


if __name__ == "__main__":
    unittest.main()
