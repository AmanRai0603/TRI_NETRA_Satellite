#!/usr/bin/env python3
"""One version for the whole repository: `VERSION` is the source, and every place a version is
written follows it.

    python3 tools/version.py              print VERSION and what each part says
    python3 tools/version.py --check      exit 1 when any part differs from VERSION
    python3 tools/version.py --set 1.0.1  write a new version into VERSION and every part

The parts: the engine's Cargo workspace (every engine crate, the CLI and the desktop app inherit it),
the Rust flight software's Cargo package, the C flight software's build id. The Rust build ids
(`adcs_sim::ENGINE`, `fsw::BUILD_ID`, the Rust C interface's `adcs_fsw_build_id`) are built from their
Cargo version at compile time, so they cannot drift; the check makes sure they still are. Both flight
softwares' build ids end with the algorithms' identity (` alg <id>`, tools/flight_build.py), after the
version the patterns read. The wheel, the kits and the release tag read
VERSION directly.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import re
import sys

from common import ROOT

SEMVER = re.compile(r"^\d+\.\d+\.\d+$")

# (part, file, pattern whose group 1 is the version)
PARTS = [
    ("engine (Cargo workspace)", "engine/Cargo.toml", r'(?ms)^\[workspace\.package\].*?^version = "([^"]+)"'),
    ("flight software, Rust (Cargo)", "fsw-rs/Cargo.toml", r'(?ms)^\[package\].*?^version = "([^"]+)"'),
    ("flight software, C (build id)", "fsw/src/adcs_fsw.c", r'adcs_fsw_build_id\(void\) \{ return "trinetra-fsw-c/([^ "]+)'),
]
# build ids that must take their version from Cargo, never a literal
DERIVED = [
    ("adcs_sim::ENGINE", "engine/crates/adcs-sim/src/lib.rs", r'pub const ENGINE: &str = concat!\("adcs-engine-rs/", env!\("CARGO_PKG_VERSION"\)'),
    ("fsw::BUILD_ID", "fsw-rs/src/fsw.rs", r'pub const BUILD_ID: &str = concat!\("trinetra-fsw-rs/", env!\("CARGO_PKG_VERSION"\)'),
    ("adcs_fsw_build_id (Rust C interface)", "fsw-rs/src/cabi.rs", r'concat!\("trinetra-fsw-rs/", env!\("CARGO_PKG_VERSION"\)'),
]


def source():
    v = (ROOT / "VERSION").read_text().strip()
    if not SEMVER.match(v):
        raise SystemExit(f"version: VERSION holds {v!r}, not X.Y.Z")
    return v


def read_parts():
    out = []
    for name, rel, pat in PARTS:
        m = re.search(pat, (ROOT / rel).read_text())
        out.append((name, rel, m.group(1) if m else None))
    return out


def problems(v):
    bad = [f"{name} ({rel}) says {got or 'nothing'}, VERSION says {v}" for name, rel, got in read_parts() if got != v]
    bad += [f"{name} ({rel}) is not built from its Cargo version" for name, rel, pat in DERIVED
            if not re.search(pat, (ROOT / rel).read_text())]
    return bad


def set_version(new):
    if not SEMVER.match(new):
        raise SystemExit(f"version: {new!r} is not X.Y.Z")
    for name, rel, pat in PARTS:
        f = ROOT / rel
        text = f.read_text()
        m = re.search(pat, text)
        if not m:
            raise SystemExit(f"version: no version found in {rel} ({name}); nothing written")
        f.write_text(text[:m.start(1)] + new + text[m.end(1):])
    (ROOT / "VERSION").write_text(new + "\n")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    g = ap.add_mutually_exclusive_group()
    g.add_argument("--check", action="store_true", help="exit 1 when any part differs from VERSION")
    g.add_argument("--set", metavar="X.Y.Z", help="write this version into VERSION and every part")
    a = ap.parse_args(argv)
    if a.set:
        set_version(a.set)
        print(f"version: {a.set} written to VERSION and {len(PARTS)} parts")
        return 0
    v = source()
    bad = problems(v)
    if not a.check:
        print(f"VERSION {v}")
        for name, rel, got in read_parts():
            print(f"  {name:32s} {got or '?':10s} {rel}")
    for b in bad:
        print("version: " + b, file=sys.stderr)
    if a.check and not bad:
        print(f"version: every part is {v}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
