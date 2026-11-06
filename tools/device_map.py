"""The rig's device map held equal to the HAL header: every port in the map is the number in adcs_devices.h.

The header (fsw/include/adcs_devices.h) is the truth (design/decisions/DR-oils-001.toml); a map entry that
names a part the header does not know, or whose port differs from it, is a problem.
Usage: python3 tools/device_map.py [map.toml ...]   (default: fsw/devices/example_3u.toml)
Copyright (c) 2026 Agastya. All rights reserved.
"""
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HEADER = ROOT / "fsw" / "include" / "adcs_devices.h"
DEFAULT_MAP = ROOT / "fsw" / "devices" / "example_3u.toml"

# map device name -> {hal field: the header macro it must equal}. The coils are PWM with no number in the header.
BINDING = {
    "magnetometer": {"bus": "ADCS_MAG_BUS", "addr7": "ADCS_MAG_ADDR"},
    "sun_sensor_plus_x": {"bus": "ADCS_SUN_BUS", "addr7": "ADCS_SUN_ADDR"},
    "earth_sensor": {"bus": "ADCS_ES_BUS", "addr7": "ADCS_ES_ADDR"},
    "star_tracker": {"port": "ADCS_ST_UART"},
    "gyro": {"bus": "ADCS_GYRO_BUS", "chip_select": "ADCS_GYRO_CS"},
    "gnss": {"port": "ADCS_GPS_UART"},
    "coils": {},
    "momentum_devices": {"port": "ADCS_CAN_PORT", "rotor_cmd": "ADCS_CAN_ROTOR_CMD", "gimbal_cmd": "ADCS_CAN_GIMBAL_CMD",
                         "rotor_tm": "ADCS_CAN_ROTOR_TM", "valves": "ADCS_CAN_VALVES"},
}
KIND = {"magnetometer": "i2c", "sun_sensor_plus_x": "i2c", "earth_sensor": "i2c", "star_tracker": "uart", "gyro": "spi",
        "gnss": "uart", "coils": "pwm", "momentum_devices": "can"}


def header_numbers(path=HEADER):
    """Every integer `#define NAME 123u` / `0x1Eu` of the header."""
    out = {}
    for m in re.finditer(r"^#define\s+(ADCS_\w+)\s+(0x[0-9A-Fa-f]+|\d+)u?\b", Path(path).read_text(), re.M):
        out[m.group(1)] = int(m.group(2), 0)
    return out


def problems(map_path=DEFAULT_MAP, header=HEADER):
    """The differences between a map and the header, one line each; empty when they agree."""
    nums = header_numbers(header)
    devs = tomllib.loads(Path(map_path).read_text()).get("device", [])
    out = []
    for d in devs:
        name, hal = d.get("name"), d.get("hal", {})
        if name not in BINDING:
            out.append(f"{name}: no such part in adcs_devices.h (the header has none by that name)")
            continue
        if hal.get("kind") != KIND[name]:
            out.append(f"{name}: kind {hal.get('kind')!r}, the header has it on {KIND[name]!r}")
        for field, macro in BINDING[name].items():
            if macro not in nums:
                out.append(f"{name}: {macro} is not defined in adcs_devices.h")
            elif hal.get(field) != nums[macro]:
                out.append(f"{name}: {field} = {hal.get(field)!r}, adcs_devices.h {macro} = {nums[macro]:#x}")
    present = {d.get("name") for d in devs}
    out += [f"{n}: in the header but not in the map" for n in BINDING if n not in present]
    return out


def main(argv):
    bad = 0
    for p in argv or [str(DEFAULT_MAP)]:
        for line in problems(p):
            print(f"{p}: {line}")
            bad += 1
    print(f"device map: {bad} problem(s)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
