"""Put tools/ on the import path, as running a tool does. Copyright (c) 2026 Agastya."""
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
if str(ROOT / "tools") not in sys.path:
    sys.path.insert(0, str(ROOT / "tools"))
