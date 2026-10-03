"""What every part of tools/engine.py shares: where the engine, the data and the stores are.

Copyright (c) 2026 Agastya. All rights reserved.
"""
from common import ROOT




BIN = ROOT / "engine" / "target" / "release" / "adcs"


DATA = ROOT / "matlab_sils" / "data" / "scenarios"


TWIN = ROOT / "matlab_sils" / "store" / "results"


ENG = ROOT / "matlab_sils" / "store" / "results_engine"


OUT = ROOT / "results"
