#!/usr/bin/env python3
"""Writes tests/data/igrf14_reference.txt: 50 IGRF-14 points computed by two independent implementations (S12.6, TS1(f)).

    python3 -m venv v && v/bin/pip install ppigrf==2.1.0 pyIGRF14==1.0.4
    v/bin/python tests/data/igrf14_reference_make.py > tests/data/igrf14_reference.txt

It is run once and its output committed; nothing in the repository runs it (the tests read the file). It imports nothing of this
repository, so the reference shares no code with the design, the engine, the flight software or the twin.

The two packages (both PyPI, MIT):
  ppigrf 2.1.0      reads IGRF14.shc (the coefficients with the 2030.0 epoch as a column), its own Legendre recursion
                    (get_legendre), its own WGS-84 geodetic conversion (geod2geoc, geoc2geod); the coefficients between epochs are
                    interpolated linearly in CALENDAR time (pandas `interpolate(method='time')`) between the epochs' dates.
  pyIGRF14 1.0.4    the IAGA synthesis routine igrf12syn ported to Python (calculate.py), its own coefficient file
                    (src/igrf14coeffs.txt) and WGS-84 constants (a^2 = 40680631.6, b^2 = 40408296.0, rounded as IAGA's Fortran has
                    them); the coefficients between 2025.0 and 2030.0 are 2025.0 + (date - 2025.0) x the 2025-30 secular variation,
                    linear in the DECIMAL YEAR, which is IGRF's own definition.
The two differ in the date convention. A decimal year d is turned into the calendar date for ppigrf that makes its calendar-linear
interpolation the decimal-year-linear one: the fraction f = (d - e0)/(e1 - e0) of the way between the two epochs e0 <= d < e1 (of the
file's 5-year epochs, the last interval being 2025.0 to 2030.0) is the same fraction of the way between 1 January 0 h of e0 and of e1.
Without it ppigrf is up to 0.07 nT from pyIGRF14 inside a 5-year interval at degree 13 (the epochs' own values agree to 0.005 nT).

Degree 10: ppigrf's max_degree=10; pyIGRF14 has no such argument, so its coefficient lists (g, h, as it builds them for the date) are cut
to degree 10 before its own synthesis routine runs (its nmx = len(g) - 1 then follows).

The grid (documented here, nothing random): 44 points i = 0..43 of
    decimal year  D[(3 i) mod 11]   D = 2025.0 2025.5 2026.0 2026.37 2027.0 2027.5 2028.0 2028.62 2029.0 2029.5 2030.0  (both ends of 2025.0-2030.0)
    latitude      L[(5 i) mod 12]   L = -80 -65 -50 -35 -20 -5 10 25 40 55 70 85  [deg, geodetic]
    longitude     G[(7 i) mod 13]   G = -180 -150 ... 180 (13 values, 30 deg apart)  [deg east]
    altitude      H[(5 i) mod 6]    H = 300 400 500 600 700 800  [km above the WGS-84 ellipsoid]
then the point the S12 inventory names (2027.0, 45 N, 105 E, 500 km), then five edge cases: two points a tenth of a degree from the poles, two
a hundredth of a degree from them (at the altitude limits 300 and 800 km, and at longitudes 0, 90, -120, 179.9) and the equator at the
anti-meridian at the end of the span.
"""
import datetime
import importlib.metadata as md
import platform
import sys

import numpy as np
import pandas
import ppigrf
import pyIGRF14.calculate as calc

EPOCHS = [1900.0 + 5 * i for i in range(27)]           # 1900.0 ... 2030.0
D = [2025.0, 2025.5, 2026.0, 2026.37, 2027.0, 2027.5, 2028.0, 2028.62, 2029.0, 2029.5, 2030.0]
L = [-80, -65, -50, -35, -20, -5, 10, 25, 40, 55, 70, 85]
G = [-180 + 30 * k for k in range(13)]
H = [300, 400, 500, 600, 700, 800]
EDGES = [(2027.0, 89.9, 0.0, 500.0), (2027.0, -89.9, 90.0, 500.0), (2028.5, 89.99, -120.0, 800.0), (2025.0, -89.99, 179.9, 300.0),
         (2030.0, 0.0, 180.0, 800.0)]


def points():
    out = [(D[(3 * i) % 11], float(L[(5 * i) % 12]), float(G[(7 * i) % 13]), float(H[(5 * i) % 6])) for i in range(44)]
    return out + [(2027.0, 45.0, 105.0, 500.0)] + EDGES


def calendar(d):
    """The calendar date for ppigrf that gives the decimal-year-linear interpolation (see the module's text)."""
    i = min(max(k for k in range(26) if EPOCHS[k] <= d), 25)
    f = (d - EPOCHS[i]) / (EPOCHS[i + 1] - EPOCHS[i])
    day = lambda y: datetime.datetime(int(y), 1, 1)  # noqa: E731
    return day(EPOCHS[i]) + (day(EPOCHS[i + 1]) - day(EPOCHS[i])) * f


def pp(d, lat, lon, alt, deg):
    be, bn, bu = ppigrf.igrf(lon, lat, alt, calendar(d), max_degree=deg)
    return [float(bn.ravel()[0]), float(be.ravel()[0]), float(-bu.ravel()[0])]


def py(d, lat, lon, alt, deg):
    orig = calc.get_coeffs
    calc.get_coeffs = lambda date: tuple(x[:deg + 1] for x in orig(date))
    try:
        x, y, z, _f = calc.igrf12syn(d, 1, alt, lat, lon)
    finally:
        calc.get_coeffs = orig
    return [x, y, z]


def main():
    rows = []
    for n, p in enumerate(points(), 1):
        vals = [pp(*p, 13), py(*p, 13), pp(*p, 10), py(*p, 10)]
        rows.append((n, p, vals))
    d13 = [max(abs(a - b) for a, b in zip(v[0], v[1])) for _n, _p, v in rows]
    d10 = [max(abs(a - b) for a, b in zip(v[2], v[3])) for _n, _p, v in rows]
    print("# IGRF-14 reference: 50 points computed by two independent implementations (docs/S12_INVENTORY.md TS1(f), S12.6).")
    print("# Written once by tests/data/igrf14_reference_make.py and committed; its docstring states the grid, the date convention and the degree-10 cut.")
    print(f"# ppigrf {md.version('ppigrf')} (PyPI, MIT; numpy {np.__version__}, pandas {pandas.__version__}); "
          f"pyIGRF14 {md.version('pyIGRF14')} (PyPI, MIT); python {platform.python_version()}.")
    print("# Each row: n, decimal year, geodetic latitude [deg], east longitude [deg], altitude above the WGS-84 ellipsoid [km], then north, east and")
    print("# down [nT] of: ppigrf degree 13, pyIGRF14 degree 13, ppigrf degree 10, pyIGRF14 degree 10.")
    print(f"# The two packages agree to {max(d13):.6f} nT at degree 13 and {max(d10):.6f} nT at degree 10 over the 50 points (largest difference of a component).")
    print("# columns: n year lat lon alt  pp13_N pp13_E pp13_D  py13_N py13_E py13_D  pp10_N pp10_E pp10_D  py10_N py10_E py10_D")
    for n, (d, lat, lon, alt), vals in rows:
        print(f"{n} {d!r} {lat!r} {lon!r} {alt!r} " + " ".join(" ".join(f"{x:.6f}" for x in v) for v in vals))
    sys.stderr.write(f"agreement: degree 13 {max(d13):.6f} nT (point {d13.index(max(d13)) + 1}), degree 10 {max(d10):.6f} nT (point {d10.index(max(d10)) + 1})\n")


if __name__ == "__main__":
    main()
