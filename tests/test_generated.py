"""Generated files are what their definitions give: a hand edit to one is caught here.
Copyright (c) 2026 Agastya. All rights reserved."""
import unittest

from _path import ROOT
import export_catalogue
import gen_fsw_params


class Generated(unittest.TestCase):
    def test_the_catalogue_json_is_its_toml(self):
        self.assertEqual(export_catalogue.check(), [])

    def test_the_flight_software_parameters_are_their_definition(self):
        stale = [str(p.relative_to(ROOT)) for p, text in gen_fsw_params.outputs() if p.read_text() != text]
        self.assertEqual(stale, [])


if __name__ == "__main__":
    unittest.main()
