"""The outside-reference ledger (docs/references.toml) says what it can show: a checked row names
the tests that check it, and they exist; an owed row says why. Copyright (c) 2026 Agastya."""
import tomllib
import unittest

import _path
from common import ROOT

_ = _path


class Ledger(unittest.TestCase):
    def test_a_checked_reference_names_tests_that_exist_and_an_owed_one_says_why(self):
        refs = tomllib.loads((ROOT / "docs" / "references.toml").read_text())["ref"]
        self.assertEqual(len({r["id"] for r in refs}), len(refs), "ids are unique")
        for r in refs:
            self.assertIn(r["status"], ("checked", "owed"), r["id"])
            self.assertTrue(r["what"] and r["where"] and r["reference"], r["id"])
            tests = [t.strip() for t in r["test"].split(",") if t.strip()]
            if r["status"] == "checked":
                self.assertTrue(tests, f"{r['id']}: checked by what?")
                for t in tests:
                    self.assertTrue((ROOT / t).is_file(), f"{r['id']}: {t} does not exist")
            else:
                self.assertTrue(r.get("note"), f"{r['id']}: owed, but not why")


if __name__ == "__main__":
    unittest.main()
