"""The soft-OILS deadline gate: fly scenarios on the firmware in QEMU and fail when a deadline margin falls.

Each scenario is flown with `adcs run --fsw qemu --oils` for a fixed duration; the run's worst-case deadline
margin (manifest oils.worst_case_margin_s: execution at the worst CPI, interrupts and bus time against the
share of the control period the command must land in) is compared with the minimum stored in
results/oils_deadline_min.json. A margin below its minimum, an overrun, or a run not timed by the
instruction-count plugin fails. The runs are deterministic (exact instruction counts), so a fall is a change in
the flight software or the model, not noise.

Usage: python3 tools/oils_gate.py              check against the stored minima
       python3 tools/oils_gate.py --set        measure and store the minima (the file records how)
Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import datetime
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ADCS = ROOT / "engine" / "target" / "release" / "adcs"
STORE = ROOT / "results" / "oils_deadline_min.json"
SLACK = 0.10      # a stored minimum is (1 - SLACK) of the margin measured when it was set


def judge(measured, stored):
    """The failures of one set of measurements against the stored file: a list of lines, empty when it holds."""
    out = []
    for scen, floor in stored["minimum_margin_s"].items():
        m = measured.get(scen)
        if m is None:
            out.append(f"{scen}: not flown")
            continue
        if not m["time_base"].startswith("qemu insn_count plugin"):
            out.append(f"{scen}: timed by '{m['time_base']}', not by the instruction-count plugin")
        if m["overruns"]:
            out.append(f"{scen}: {m['overruns']} overrun(s)")
        if m["margin_s"] < floor:
            out.append(f"{scen}: deadline margin {1e3 * m['margin_s']:.3f} ms is below the stored minimum {1e3 * floor:.3f} ms")
    return out


def fly(scen, duration_s, out):
    cmd = [str(ADCS), "run", scen, "--fsw", "qemu", "--oils", "--quiet", "--set", f"engine.duration_s={duration_s}", "--out", str(out)]
    p = subprocess.run(cmd, capture_output=True, text=True)
    mf = out / "manifest.json"
    if not mf.exists():
        sys.exit(f"{scen}: the run left no manifest (exit {p.returncode}): {(p.stdout + p.stderr).strip()[-600:]}")
    o = json.loads(mf.read_text())["oils"]
    return {"margin_s": o["worst_case_margin_s"], "overruns": o["overruns"], "time_base": o.get("time_base", "not recorded")}


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--set", action="store_true", help="measure the scenarios and store their minima")
    a = ap.parse_args()
    stored = json.loads(STORE.read_text())
    if not ADCS.exists():
        sys.exit("engine not built: cargo build --release -p adcs-cli (in engine/)")
    with tempfile.TemporaryDirectory() as d:
        measured = {s: fly(s, stored["duration_s"], Path(d) / s) for s in stored["minimum_margin_s"]}
    for s, m in measured.items():
        print(f"{s:22s} worst-case margin {1e3 * m['margin_s']:8.3f} ms  overruns {m['overruns']}  [{m['time_base']}]")
    if a.set:
        stored["minimum_margin_s"] = {s: round((1 - SLACK) * m["margin_s"], 6) for s, m in measured.items()}
        stored["set"] = {"date": datetime.date.today().isoformat(), "slack": SLACK,
                         "how": f"each scenario flown {stored['duration_s']:g} s with `adcs run --fsw qemu --oils` (tools/oils_gate.py --set); "
                                f"the minimum is {1 - SLACK:.0%} of the worst-case margin measured then (the runs are deterministic, the slack is for a toolchain change)"}
        STORE.write_text(json.dumps(stored, indent=1) + "\n")
        print(f"stored the minima in {STORE.relative_to(ROOT)}")
        return 0
    bad = judge(measured, stored)
    for line in bad:
        print("FAIL", line)
    print(f"oils gate: {len(measured)} scenario(s), {len(bad)} failure(s)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
