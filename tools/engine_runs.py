"""Build, fly scenarios, seed sweeps, and flight-software parity (C, Rust, the virtual OBC).

Copyright (c) 2026 Agastya. All rights reserved.
"""
import concurrent.futures as cf, json, math, re, shutil, statistics, subprocess, time
from common import Steps, sh, write_text
from engine_base import BIN, DATA, ENG, OUT, ROOT


def build(_):
    S = Steps("engine.py", "build")
    S(1)
    sh(["make", "-s", "clean"], cwd=ROOT / "fsw")
    sh(["make", "-s", "test"], cwd=ROOT / "fsw")
    sh(["make", "-s", "check"], cwd=ROOT / "fsw")
    S(2)
    sh(["cargo", "test", "--release", "-q"], cwd=ROOT / "fsw-rs")
    S(3)
    # the OBC links the C-ABI static library: build it (host and Cortex-M) before make obc, since a plain
    # cargo build leaves a newer libadcs_fsw.a without the exports that make would not rebuild
    sh(["cargo", "build", "--release", "-q", "--no-default-features", "--features", "cabi"], cwd=ROOT / "fsw-rs")
    sh(["cargo", "build", "--release", "-q", "--no-default-features", "--features", "cabi", "--target", "thumbv7em-none-eabihf"], cwd=ROOT / "fsw-rs")
    S(4)
    sh(["make", "-s", "obc"], cwd=ROOT / "fsw")            # virtual OBC firmware (process, QEMU) and the insn plugin
    S(5)
    sh(["cargo", "test", "--release", "-q"], cwd=ROOT / "engine")
    sh(["cargo", "build", "--release", "-q"], cwd=ROOT / "engine")


def scenarios(names):
    return names or sorted(p.stem for p in DATA.glob("*.json"))


def one(args):
    scen, fsw, seed, out, extra = args
    cmd = [str(BIN), "run", scen, "--fsw", fsw, "--seed", str(seed), "--quiet"] + extra
    if out:
        cmd += ["--out", str(out)]
    # the run's folder starts empty, so a run that fails leaves nothing to be read as its result
    shutil.rmtree(out or ENG / scen, ignore_errors=True)
    t0 = time.time()
    p = subprocess.run(cmd, capture_output=True, text=True)
    return scen, seed, p.returncode, time.time() - t0, (p.stdout + p.stderr).strip()


def run(a):
    S = Steps("engine.py", "run")
    jobs = [(s, a.fsw, a.seed, None, []) for s in scenarios(a.scenarios)]
    S(1, f"{len(jobs)} scenario(s) on {a.jobs} processes")
    S(2, "as each finishes")
    failed = 0
    with cf.ProcessPoolExecutor(a.jobs) as ex:
        for scen, _, rc, dt, txt in ex.map(one, jobs):
            print(f"[{'ok' if rc == 0 else 'FAIL'}] {scen:24s} {dt:6.1f} s wall")
            if rc:
                failed += 1
                print(txt)
    return failed


def mc(a):
    base = ENG / f"mc_{a.scenario}"
    jobs = [(a.scenario, a.fsw, s, base / f"seed_{s:03d}", []) for s in range(1, a.seeds + 1)]
    S = Steps("engine.py", "mc")
    S(1, f"{a.scenario}, {a.seeds} seeds")
    if base.exists():                       # no seed of an earlier sweep is ever read as this one's
        shutil.rmtree(base)
    rows, failed = [], []
    with cf.ProcessPoolExecutor(a.jobs) as ex:
        for scen, seed, rc, dt, txt in ex.map(one, jobs):
            if rc:
                print(f"[FAIL] seed {seed}\n{txt}")
                failed.append(seed)
                continue
            m = json.loads((base / f"seed_{seed:03d}" / "manifest.json").read_text())
            rows.append({x["id"]: x["value"] for x in m["metrics"]} | {"seed": seed})
    S(2)
    ids = [k for k in rows[0] if k != "seed"] if rows else []
    summ = {}
    for k in ids:
        v = [r[k] for r in rows if isinstance(r.get(k), (int, float)) and math.isfinite(r[k])]
        if v:
            summ[k] = {"n": len(v), "mean": statistics.fmean(v), "std": statistics.pstdev(v), "min": min(v), "max": max(v)}
    write_text(base / "summary.json", json.dumps({"scenario": a.scenario, "fsw": a.fsw, "runs": rows, "failed_seeds": failed, "stats": summ}, indent=1))
    for k, s in summ.items():
        print(f"  {k:28s} mean {s['mean']:.4g}  std {s['std']:.3g}  [{s['min']:.4g}, {s['max']:.4g}]  n={s['n']}")
    if failed:
        print(f"  {len(failed)} seed(s) failed to fly: {failed}; the statistics above leave them out and say so here")
    return len(failed)


def fsw_parity(a):
    S = Steps("engine.py", "fsw-parity")
    S(1, f"{a.duration:.0f} s each")
    lines, failed = [], 0
    for s in scenarios(a.scenarios):
        p = subprocess.run([str(BIN), "parity", s, "--set", f"engine.duration_s={a.duration}"], capture_output=True, text=True)
        line = re.sub(r"\(trinetra[^)]*\)\)", "", parity_line(p))
        if p.returncode:                      # adcs parity exits non-zero when the targets differ or a run fails
            failed += 1
            line = "[FAIL] " + line
        lines.append(line)
    S(2)
    for line in lines:
        print(line)
    return failed


VOBC_PAIRS = [("c", "obc-posix"), ("rust", "obc-posix-rs"), ("c", "qemu"), ("rust", "qemu-rs"), ("c", "rust")]


def vobc(a):
    S = Steps("engine.py", "vobc")
    S(1)
    sh(["make", "-s", "-C", "fsw", "obc"])
    scen = a.scenarios or ["detumble_ais", "mission_ais", "fine_hold_img", "slew_img", "fine_hold_cmg", "fine_hold_fmr_rcs", "sun_spin_ais"]
    L = ["# Virtual OBC loop", "", "Owner: Agastya. `tools/engine.py vobc` -- the Rust engine drives the flight software over",
         "adcs-link/1 (fsw/targets/link) running as a separate host process and as bare-metal firmware on an",
         "emulated Cortex-M4F (QEMU mps2-an386, arm-none-eabi-gcc + newlib for C, rustc thumbv7em-none-eabihf for",
         f"Rust), and compares every recorded sample with the in-process build ({a.duration:.0f} s per scenario).", "",
         "| scenario | reference | virtual OBC | result | wall [s] |", "|---|---|---|---|---:|"]
    S(2, f"{len(scen)} scenario(s) x {len(VOBC_PAIRS)} pairs")
    failed = 0
    for s in scenario_list(scen):
        for ref, tgt in VOBC_PAIRS:
            t0 = time.time()
            p = subprocess.run([str(BIN), "parity", s, "--fsw", ref, "--against", tgt, "--set", f"engine.duration_s={a.duration}"],
                               capture_output=True, text=True)
            line = parity_line(p)
            ok = p.returncode == 0 and "bit-identical" in line
            res = "bit-identical" if ok else re.sub(r".*: max", "max", line)
            failed += not ok
            print(f"{'' if ok else '[FAIL] '}{s:20s} {ref:5s} vs {tgt:13s} {res}")
            L.append(f"| {s} | {ref} (in-process) | {tgt} | {res} | {time.time() - t0:.1f} |")
    S(3)
    write_text(OUT / "VIRTUAL_OBC.md", "\n".join(L) + "\n")
    print("wrote results/VIRTUAL_OBC.md")
    return failed


def parity_line(p):
    """The `[parity]` line of an `adcs parity` run (it exits 1 when the targets differ, and
    says so after it), else its last line."""
    lines = (p.stdout + p.stderr).strip().splitlines()
    return next((l for l in lines if l.startswith("[parity]")), lines[-1] if lines else "no output")


def scenario_list(x):
    return x
