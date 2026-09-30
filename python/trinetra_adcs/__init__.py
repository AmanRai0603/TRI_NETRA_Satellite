"""TRI-NETRA ADCS: the attitude-determination-and-control design and test engine, for any LEO
satellite, as a Python package that carries its own programs and data.

    trinetra-adcs run nadir_hold_ais            the engine's command line (adcs), every command
    trinetra-adcs-app                           the desktop app: pick a case and a scenario, fly it
    python -m trinetra_adcs help                the same as `trinetra-adcs help`

From Python:

    import trinetra_adcs as t
    t.run(["run", "nadir_hold_ais", "--quiet"])      # the exit status of the engine
    t.data_root()                                    # the scenarios, cases and ephemeris it carries

The engine and the app are the same programs as the release kits (Windows, macOS, Linux);
this package picks the one for this computer. Runs are kept in ~/.trinetra/store
(or $TRINETRA_STORE), outside the package, so an upgrade keeps them.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import os
import pathlib
import platform
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
try:
    from ._build import VERSION as __version__
except ImportError:  # a checkout, not an installed package
    __version__ = "dev"


def system():
    """This computer as the package names its programs: linux-x86_64, macos-arm64, ..."""
    osname = {"linux": "linux", "darwin": "macos", "win32": "windows"}.get(sys.platform, sys.platform)
    arch = platform.machine().lower()
    arch = {"amd64": "x86_64", "x64": "x86_64", "aarch64": "arm64"}.get(arch, arch)
    return f"{osname}-{arch}"


def data_root():
    """The data the programs read: scenarios, cases, products, the ephemeris."""
    return HERE / "_kit"


def program(name):
    """The path of `adcs` or `trinetra-app` for this computer, or a clear refusal."""
    exe = ".exe" if sys.platform == "win32" else ""
    p = HERE / "_bin" / system() / f"{name}{exe}"
    if not p.is_file():
        have = sorted(d.name for d in (HERE / "_bin").iterdir()) if (HERE / "_bin").is_dir() else []
        raise SystemExit(f"trinetra-adcs {__version__} has no engine for {system()}; it carries {', '.join(have) or 'none'}. "
                         "Use the release kit for your system, or build from source (docs/CHANGING.md).")
    if exe == "" and not os.access(p, os.X_OK):
        try:
            p.chmod(p.stat().st_mode | 0o111)  # a wheel does not always keep the executable bit
        except OSError:
            pass
    return p


def env():
    """The environment the programs run in: they read this package's data."""
    e = dict(os.environ)
    e["ADCS_ROOT"] = str(data_root())
    return e


def run(args, **kw):
    """Run the engine's command line with `args` and return its exit status."""
    return subprocess.call([str(program("adcs")), *map(str, args)], env=env(), **kw)


__all__ = ["run", "program", "data_root", "system", "__version__"]
