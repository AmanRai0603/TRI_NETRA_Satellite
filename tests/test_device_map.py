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


class BusFaultCases(unittest.TestCase):
    """O11: every fault point of the map (a device and a bus fault it takes) has a design case that injects it (design/revisions/S11.14/)."""

    def cases(self):
        import tomllib
        out = {}
        for f in sorted((common.ROOT / "design" / "revisions" / "S11.14").glob("*.toml")):
            d = tomllib.loads(f.read_text())
            for x in d.get("faults", []):
                out[(x["device"], x["kind"].removeprefix("bus_"))] = d
        return out

    def test_every_fault_point_has_a_case(self):
        import tomllib
        cases = self.cases()
        devs = tomllib.loads(device_map.DEFAULT_MAP.read_text())["device"]
        missing = []
        for d in devs:
            for k in d["bus_faults"]:
                if d["name"] == "earth_sensor":
                    continue    # no product carries an Earth sensor, so no scenario can fly it: the HAL tests (fsw-rs/tests/bus_faults.rs, test_fsw.c) hold it
                if k == "delay" and d["name"] != "gyro":
                    continue    # a delay is the bus's, the same at every device: one case, on the gyro, flies it
                if (d["name"], k) not in cases:
                    missing.append((d["name"], k))
        self.assertEqual(missing, [])

    def test_every_case_names_a_device_of_the_map_and_judges_a_flag(self):
        by_name = dict(device_map.bus_faults_by_name())
        for (dev, kind), d in self.cases().items():
            self.assertIn(kind, by_name.get(dev, []), (dev, kind))
            m = [x for x in d["metrics"] if x["kind"] == "fault_raised"]
            self.assertEqual(len(m), 1, (dev, kind))
            self.assertEqual((m[0]["limit"], m[0]["sense"]), (1.0, "min"))

    def test_the_engine_knows_the_same_devices(self):
        import re
        import tomllib
        src = (common.ROOT / "engine" / "crates" / "adcs-sim" / "src" / "busfault.rs").read_text()
        engine = re.search(r"pub const DEVICES: \[&str; \d+\] = \[(.*?)\];", src).group(1)
        engine = [x.strip().strip('"') for x in engine.split(",")]
        self.assertEqual(engine, [d["name"] for d in tomllib.loads(device_map.DEFAULT_MAP.read_text())["device"]])


if __name__ == "__main__":
    unittest.main()
