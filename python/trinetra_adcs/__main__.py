"""`trinetra-adcs ...` and `python -m trinetra_adcs ...`: the engine's command line;
`trinetra-adcs-app`: the desktop app. Copyright (c) 2026 Agastya. All rights reserved."""
import subprocess
import sys

import trinetra_adcs as t


def main(argv=None):
    """The engine's command line, with this package's data."""
    argv = sys.argv[1:] if argv is None else argv
    if argv[:1] in (["--version"], ["version"]):
        print(f"trinetra-adcs {t.__version__} ({t.system()})")
        return 0
    return t.run(argv)


def app():
    """The desktop app. Windows runs this without a console window (a GUI script)."""
    return subprocess.call([str(t.program("trinetra-app"))], env=t.env())


if __name__ == "__main__":
    sys.exit(main())
