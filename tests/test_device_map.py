"""The rig's device map equals the HAL header (design/decisions/DR-oils-001.toml: the header is the truth).

The archived 1.0 map contradicted the header on most ports; the check must fail on it and pass on the map the
rig reads (fsw/devices/example_3u.toml). Copyright (c) 2026 Agastya. All rights reserved.
"""
import unittest

import _path  # puts tools/ on the import path
import common
import device_map

_ = _path

ARCHIVED = common.ROOT / "archive" / "design-1.0" / "spec" / "rig" / "device_maps" / "example_3u.toml"


class DeviceMap(unittest.TestCase):
    def test_the_rig_map_equals_the_header(self):
        self.assertEqual(device_map.problems(), [])

    def test_the_archived_map_fails_on_the_magnetometer_bus(self):
        if not ARCHIVED.exists():
            self.skipTest("the archived 1.0 map is not in this checkout")
        p = device_map.problems(ARCHIVED)
        self.assertTrue(any("magnetometer: bus = 1" in x for x in p), p)
        self.assertGreaterEqual(len(p), 4, p)

    def test_a_changed_port_is_found(self):
        import tempfile
        from pathlib import Path
        txt = device_map.DEFAULT_MAP.read_text().replace("chip_select = 0", "chip_select = 1")
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / "m.toml"
            f.write_text(txt)
            self.assertTrue(any(x.startswith("gyro: chip_select") for x in device_map.problems(f)))


if __name__ == "__main__":
    unittest.main()
