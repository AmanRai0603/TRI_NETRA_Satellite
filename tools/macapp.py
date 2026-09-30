#!/usr/bin/env python3
"""The macOS desktop app, TRI-NETRA ADCS.app, from a macOS kit.

    python3 tools/macapp.py --kit dist/kit/trinetra-adcs-<v>-aarch64-apple-darwin --out dist/app

The bundle holds the app program in Contents/MacOS, the kit's data and documents in
Contents/Resources (where the program looks for them), the icon, Info.plist and PkgInfo.
The release workflow signs it (ad hoc, or with a Developer ID when one is configured) and
zips it with `ditto -c -k --keepParent`, which keeps the bundle whole.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import pathlib
import shutil
import sys

from common import ROOT, write_text

NAME = "TRI-NETRA ADCS"
BUNDLE_ID = "space.agastya.trinetra-adcs"


def info_plist(version):
    return f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>{NAME}</string>
  <key>CFBundleDisplayName</key><string>{NAME}</string>
  <key>CFBundleIdentifier</key><string>{BUNDLE_ID}</string>
  <key>CFBundleVersion</key><string>{version}</string>
  <key>CFBundleShortVersionString</key><string>{version}</string>
  <key>CFBundleExecutable</key><string>trinetra-app</string>
  <key>CFBundleIconFile</key><string>trinetra</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHumanReadableCopyright</key><string>Copyright (c) 2026 Agastya. All rights reserved.</string>
</dict>
</plist>
"""


def build(kit, out):
    kit, out = pathlib.Path(kit), pathlib.Path(out)
    prog = kit / "trinetra-app"
    if not prog.is_file():
        sys.exit(f"{kit} has no trinetra-app program: build the kit on a Mac from a release build")
    version = (kit / "VERSION").read_text().splitlines()[0].replace("TRI-NETRA ADCS", "").strip()
    app = out / f"{NAME}.app"
    if app.exists():
        shutil.rmtree(app)
    res, macos = app / "Contents" / "Resources", app / "Contents" / "MacOS"
    macos.mkdir(parents=True)
    shutil.copytree(kit, res, ignore=shutil.ignore_patterns("trinetra-app", "adcs"))
    shutil.copy2(prog, macos / "trinetra-app")
    (macos / "trinetra-app").chmod(0o755)
    shutil.copy2(ROOT / "engine" / "crates" / "trinetra-app" / "icon" / "trinetra.icns", res / "trinetra.icns")
    write_text(app / "Contents" / "Info.plist", info_plist(version))
    write_text(app / "Contents" / "PkgInfo", "APPL????")
    return app, version


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--kit", required=True, type=pathlib.Path, help="a macOS kit (tools/kit.py) with the trinetra-app program")
    ap.add_argument("--out", type=pathlib.Path, default=ROOT / "dist" / "app")
    a = ap.parse_args(argv)
    app, v = build(a.kit, a.out)
    print(f"app: {app} -- TRI-NETRA ADCS {v}. Sign it, then zip it with `ditto -c -k --keepParent`.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
