#!/usr/bin/env python3
"""One Python package for every computer: trinetra_adcs-<version>-py3-none-any.whl.

    python3 tools/build_wheel.py --kit <files-only kit> --bin linux-x86_64=<dir> [--bin windows-x86_64=<dir> ...] --out dist/
    python3 tools/build_wheel.py --selftest

The wheel carries the Python front end (python/trinetra_adcs), the engine and the desktop
app for each system given (each <dir> holds adcs[.exe] and trinetra-app[.exe]), and the data
of a files-only kit (tools/kit.py --files-only). Installing it compiles nothing: the front
end runs the program for the computer it is on. It installs two commands:

    trinetra-adcs        the engine's command line (console)
    trinetra-adcs-app    the desktop app (a GUI script: no console window on Windows)

Every file is listed in RECORD with its SHA-256; `--selftest` builds a small wheel and checks
the RECORD, the entry points, the executable bits and that a changed byte is caught.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import base64
import hashlib
import pathlib
import sys
import tempfile
import zipfile

from common import ROOT, atomic_path

PKG = ROOT / "python" / "trinetra_adcs"
SYSTEMS = ["linux-x86_64", "macos-arm64", "macos-x86_64", "windows-x86_64"]
PROGRAMS = ["adcs", "trinetra-app"]
FIXED = (2026, 1, 1, 0, 0, 0)

METADATA = """Metadata-Version: 2.1
Name: trinetra-adcs
Version: {version}
Summary: TRI-NETRA ADCS: the attitude determination and control design and test engine, for any LEO satellite
Author: Agastya
License: Proprietary. Copyright (c) 2026 Agastya. All rights reserved.
Requires-Python: >=3.8
Classifier: Operating System :: Microsoft :: Windows
Classifier: Operating System :: MacOS
Classifier: Operating System :: POSIX :: Linux
Description-Content-Type: text/plain

Install: python -m pip install trinetra_adcs-{version}-py3-none-any.whl
Then:    trinetra-adcs help        (the engine)
         trinetra-adcs-app         (the desktop app)
It carries its engine for {systems}, and the scenarios, cases and ephemeris it reads.
"""
WHEEL = "Wheel-Version: 1.0\nGenerator: trinetra tools/build_wheel.py\nRoot-Is-Purelib: true\nTag: py3-none-any\n"
ENTRY_POINTS = "[console_scripts]\ntrinetra-adcs = trinetra_adcs.__main__:main\n\n[gui_scripts]\ntrinetra-adcs-app = trinetra_adcs.__main__:app\n"


def digest(data):
    return "sha256=" + base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()


def contents(version, kit, bins):
    """(name in the wheel, bytes, executable) for every file, in a fixed order."""
    out = [(f"trinetra_adcs/{f.name}", f.read_bytes(), False) for f in sorted(PKG.glob("*.py"))]
    out.append(("trinetra_adcs/_build.py", f'VERSION = "{version}"\n'.encode(), False))
    for system, d in sorted(bins.items()):
        exe = ".exe" if system.startswith("windows") else ""
        for p in PROGRAMS:
            src = pathlib.Path(d) / f"{p}{exe}"
            if not src.is_file():
                sys.exit(f"--bin {system}={d}: {src.name} is not there")
            out.append((f"trinetra_adcs/_bin/{system}/{p}{exe}", src.read_bytes(), True))
    kit = pathlib.Path(kit)
    for f in sorted(p for p in kit.rglob("*") if p.is_file()):
        out.append((f"trinetra_adcs/_kit/{f.relative_to(kit).as_posix()}", f.read_bytes(), False))
    info = f"trinetra_adcs-{version}.dist-info"
    out += [(f"{info}/METADATA", METADATA.format(version=version, systems=", ".join(sorted(bins))).encode(), False),
            (f"{info}/WHEEL", WHEEL.encode(), False), (f"{info}/entry_points.txt", ENTRY_POINTS.encode(), False)]
    return out, info


def build(version, kit, bins, out_dir):
    if not (pathlib.Path(kit) / "data" / "scenarios").is_dir() or not (pathlib.Path(kit) / "VERSION").is_file():
        sys.exit(f"{kit} is not a kit: python3 tools/kit.py --files-only --out <dir>")
    if not bins:
        sys.exit("no --bin system=<dir>: the package would run nowhere")
    for s in bins:
        if s not in SYSTEMS:
            sys.exit(f"unknown system {s!r}; one of {', '.join(SYSTEMS)}")
    files, info = contents(version, kit, bins)
    wheel = pathlib.Path(out_dir) / f"trinetra_adcs-{version}-py3-none-any.whl"
    record = []
    with atomic_path(wheel) as part, zipfile.ZipFile(part, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for name, data, exe in files:
            zi = zipfile.ZipInfo(name, FIXED)
            zi.compress_type = zipfile.ZIP_DEFLATED
            zi.external_attr = (0o755 if exe else 0o644) << 16
            z.writestr(zi, data)
            record.append(f"{name},{digest(data)},{len(data)}")
        record.append(f"{info}/RECORD,,")
        zi = zipfile.ZipInfo(f"{info}/RECORD", FIXED)
        zi.compress_type = zipfile.ZIP_DEFLATED
        z.writestr(zi, "\n".join(record) + "\n")
    return wheel


def verify(wheel):
    """Everything the RECORD lists is in the wheel with that hash, and nothing else is."""
    bad = []
    with zipfile.ZipFile(wheel) as z:
        names = set(z.namelist())
        rec = [n for n in names if n.endswith(".dist-info/RECORD")]
        if len(rec) != 1:
            return [f"expected one RECORD, found {len(rec)}"]
        listed = set()
        for line in z.read(rec[0]).decode().splitlines():
            name, h, _ = line.rsplit(",", 2)
            listed.add(name)
            if name == rec[0]:
                continue
            if name not in names:
                bad.append(f"{name} is in the RECORD and not in the wheel")
            elif digest(z.read(name)) != h:
                bad.append(f"{name} does not match its RECORD hash")
        bad += [f"{n} is in the wheel and not in the RECORD" for n in sorted(names - listed)]
    return bad


def selftest():
    fails = []
    with tempfile.TemporaryDirectory() as t:
        t = pathlib.Path(t)
        kit = t / "kit"
        (kit / "data" / "scenarios").mkdir(parents=True)
        (kit / "data" / "scenarios" / "x.json").write_text("{}")
        (kit / "VERSION").write_text("TRI-NETRA ADCS 9.9.9\n")
        for s in ("linux-x86_64", "windows-x86_64"):
            exe = ".exe" if s.startswith("windows") else ""
            (t / s).mkdir()
            for p in PROGRAMS:
                (t / s / f"{p}{exe}").write_bytes(b"program " + p.encode())
        w = build("9.9.9", kit, {"linux-x86_64": t / "linux-x86_64", "windows-x86_64": t / "windows-x86_64"}, t / "out")
        if w.name != "trinetra_adcs-9.9.9-py3-none-any.whl":
            fails.append(f"named {w.name}")
        fails += verify(w)
        with zipfile.ZipFile(w) as z:
            names = set(z.namelist())
            for need in ("trinetra_adcs/__init__.py", "trinetra_adcs/__main__.py", "trinetra_adcs/_build.py",
                         "trinetra_adcs/_bin/linux-x86_64/adcs", "trinetra_adcs/_bin/windows-x86_64/trinetra-app.exe",
                         "trinetra_adcs/_kit/VERSION", "trinetra_adcs-9.9.9.dist-info/entry_points.txt"):
                if need not in names:
                    fails.append(f"missing {need}")
            if (z.getinfo("trinetra_adcs/_bin/linux-x86_64/adcs").external_attr >> 16) & 0o111 == 0:
                fails.append("a program is not executable")
            ep = z.read("trinetra_adcs-9.9.9.dist-info/entry_points.txt").decode()
            if "trinetra-adcs-app = trinetra_adcs.__main__:app" not in ep or "[gui_scripts]" not in ep:
                fails.append("the app is not a GUI script")
            items = {n: z.read(n) for n in z.namelist()}
        # one changed byte must be caught
        bad = t / "bad.whl"
        with zipfile.ZipFile(bad, "w") as z:
            for n, d in items.items():
                z.writestr(n, d[:-1] + bytes([d[-1] ^ 1]) if n.endswith("/adcs") else d)
        if not any("does not match" in b for b in verify(bad)):
            fails.append("a changed program passed the RECORD check")
    for f in fails:
        print("selftest FAIL:", f)
    print("build_wheel selftest:", "ok" if not fails else f"{len(fails)} failure(s)")
    return 1 if fails else 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--version", help="the release version (default: VERSION)")
    ap.add_argument("--kit", help="a files-only kit: python3 tools/kit.py --files-only --out <dir>")
    ap.add_argument("--bin", action="append", default=[], metavar="SYSTEM=DIR", help=f"a system's programs; SYSTEM is one of {', '.join(SYSTEMS)}")
    ap.add_argument("--out", default=str(ROOT / "dist"))
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args(argv)
    if a.selftest:
        return selftest()
    if not a.kit:
        ap.error("--kit is needed")
    version = a.version or (ROOT / "VERSION").read_text().strip()
    bins = dict(b.split("=", 1) for b in a.bin)
    w = build(version, a.kit, bins, a.out)
    bad = verify(w)
    if bad:
        sys.exit("\n".join(bad))
    print(f"{w} -- {w.stat().st_size / 1e6:.1f} MB, engines for {', '.join(sorted(bins))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
