"""Put tools/ on the import path, as running a tool does. Copyright (c) 2026 Agastya."""
import os
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
os.environ["TRINETRA_TRACE"] = "0"   # tests write temporary files; they are not a record of real work
if str(ROOT / "tools") not in sys.path:
    sys.path.insert(0, str(ROOT / "tools"))
