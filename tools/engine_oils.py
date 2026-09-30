"""SILS and soft OILS side by side: the flight software as Cortex-M4F firmware with exact timing.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import concurrent.futures as cf, json, shutil, subprocess, time
from common import Steps, sh, write_text
from engine_base import BIN, DATA, ENG, OUT
from engine_runs import scenarios


# ---------------- soft OILS: SILS and the flight software on the virtual Cortex-M4F, side by side ----------------
def oils_job(args):
    scen, mode, fsw, out, extra = args
    cmd = [str(BIN), "run", scen, "--fsw", fsw, "--out", str(out), "--quiet"] + extra
    if mode == "oils":
        cmd.append("--oils")
    shutil.rmtree(out, ignore_errors=True)   # a failed run leaves nothing to be read as its result
    t0 = time.time()
    p = subprocess.run(cmd, capture_output=True, text=True)
    return scen, mode, p.returncode, time.time() - t0, (p.stdout + p.stderr).strip()


def oils(a):
    S = Steps("engine.py", "oils")
    S(1)
    sh(["make", "-s", "-C", "fsw", "obc"])
    scen = scenarios(a.scenarios)
    base = ENG / "soft_oils"
    extra = ["--set", f"engine.duration_s={a.duration}"] if a.duration else []
    jobs = []
    for s in scen:
        jobs.append((s, "sils", "c", base / s / "sils", extra))
        jobs.append((s, "oils", a.fsw, base / s / "oils", extra))
    # longest first so the pool stays busy
    dur = {s: json.loads((DATA / f"{s}.json").read_text())["time"]["duration_s"] / json.loads((DATA / f"{s}.json").read_text())["time"]["dt_s"] for s in scen}
    jobs.sort(key=lambda j: -dur[j[0]] * (50 if j[1] == "oils" else 1))
    S(2, f"{len(scen)} scenario(s)")
    failed = 0
    with cf.ProcessPoolExecutor(a.jobs) as ex:
        for s, mode, rc, dt, txt in ex.map(oils_job, jobs):
            print(f"[{'ok' if rc == 0 else 'FAIL'}] {s:22s} {mode:4s} {dt:7.1f} s wall", flush=True)
            if rc:
                failed += 1
                print(txt[-2000:])
    S(3)
    oils_ledger(scen, a.fsw)
    return failed


def oils_ledger(scen=None, fsw="qemu", announce=False):
    S = Steps("engine.py", "oils-ledger", show=announce)
    S(1)
    base = ENG / "soft_oils"
    scen = scen or sorted(p.name for p in base.iterdir() if (p / "oils" / "manifest.json").exists())
    fmt = lambda x: "—" if x is None else (f"{x:.4g}" if isinstance(x, (int, float)) else str(x))
    rows, tim = [], []
    for s in scen:
        a, b = base / s / "sils" / "manifest.json", base / s / "oils" / "manifest.json"
        if not (a.exists() and b.exists()):
            continue
        A, B = json.loads(a.read_text()), json.loads(b.read_text())
        mb = {m["id"]: m for m in B["metrics"]}
        for m in A["metrics"]:
            e = mb.get(m["id"], {})
            rows.append({"scenario": s, "metric": m["id"], "unit": m.get("unit", ""), "req": m.get("req"), "sils": m.get("value"), "oils": e.get("value"),
                         "pass_sils": m.get("pass"), "pass_oils": e.get("pass")})
        o = B.get("oils") or {}
        tim.append({"scenario": s, "dt_s": B.get("dt_s"), "wall_s": B.get("wall_s"), "duration_s": B.get("duration_s"), **{k: o.get(k) for k in
                    ("ticks", "overruns", "cpu_load_mean", "cpu_load_max", "deadline_margin_min_s")},
                    "insn_mean": (o.get("instructions") or {}).get("mean"), "insn_max": (o.get("instructions") or {}).get("max"),
                    "lat_mean_ms": 1e3 * o["latency_s"]["mean"] if o.get("latency_s") else None, "lat_max_ms": 1e3 * o["latency_s"]["max"] if o.get("latency_s") else None,
                    "exec_max_ms": 1e3 * o["exec_s"]["max"] if o.get("exec_s") else None, "model": o.get("model")})
    judged = [r for r in rows if r["pass_sils"] is not None]
    agree = sum(r["pass_sils"] == r["pass_oils"] for r in judged)
    model = next((t["model"] for t in tim if t.get("model")), {}) or {}
    L = ["# Soft OILS: SILS and the flight software on the virtual OBC, side by side", "",
         "Owner: Agastya. `tools/engine.py oils` flies every scenario twice on the Rust engine (POP in the loop):",
         "**SILS** with the C flight software in-process, and **soft OILS** with the flight software built for the",
         f"OBC (arm-none-eabi-gcc, Cortex-M4F hard-float) running as firmware in QEMU mps2-an386 (`--fsw {fsw}`)",
         "behind adcs-link/1. In soft OILS every command reaches the actuators only after the OBC has read its",
         "sensors on the buses, executed the step and sent its CAN frames; the previous command holds until then",
         "(docs/SOFT_OILS.md). The step's execution is its **exact** instruction count (QEMU plugin",
         f"fsw/targets/qemu-mps2/insn_count.c) x CPI {model.get('cpi', 1.25)} / {model.get('cpu_hz', 168e6) / 1e6:.0f} MHz;",
         f"buses: I2C {model.get('i2c_hz', 4e5) / 1e3:.0f} kHz, SPI {model.get('spi_hz', 1e6) / 1e6:.0f} MHz, CAN {model.get('can_bps', 1e6) / 1e6:.0f} Mbit/s.",
         "Runs are deterministic: the same scenario gives the same instruction counts and trajectory every time.", "",
         f"**Verdict agreement SILS vs soft OILS: {agree} of {len(judged)} judged metrics** over {len(tim)} scenarios.", "",
         "## OBC timing budget", "",
         "| scenario | period [ms] | instructions mean / max | exec max [ms] | latency mean / max [ms] | CPU load mean / max | overruns | deadline margin min [ms] |",
         "|---|---:|---:|---:|---:|---:|---:|---:|"]
    for t in tim:
        L.append(f"| {t['scenario']} | {1e3 * t['dt_s']:.0f} | {fmt(t['insn_mean'])} / {fmt(t['insn_max'])} | {fmt(t['exec_max_ms'])} | "
                 f"{fmt(t['lat_mean_ms'])} / {fmt(t['lat_max_ms'])} | {100 * (t['cpu_load_mean'] or 0):.1f} % / {100 * (t['cpu_load_max'] or 0):.1f} % | "
                 f"{t['overruns']} | {fmt(1e3 * t['deadline_margin_min_s'] if t['deadline_margin_min_s'] is not None else None)} |")
    L += ["", "## Metrics: SILS vs soft OILS", "", "| scenario | metric | req | SILS | soft OILS | SILS | soft OILS |", "|---|---|---:|---:|---:|---|---|"]
    v = lambda p: {1: "pass", 0: "FAIL"}.get(p, "—")
    for r in rows:
        if r["req"] is None:
            continue
        flag = "" if r["pass_sils"] == r["pass_oils"] else " ⚠"
        L.append(f"| {r['scenario']} | {r['metric']} ({r['unit']}) | {fmt(r['req'])} | {fmt(r['sils'])} | {fmt(r['oils'])} | {v(r['pass_sils'])} | {v(r['pass_oils'])}{flag} |")
    OUT.mkdir(exist_ok=True)
    S(2)
    write_text(OUT / "soft_oils.json", json.dumps({"metrics": rows, "timing": tim}, indent=1))
    write_text(OUT / "SOFT_OILS.md", "\n".join(L) + "\n")
    print(f"soft OILS verdict agreement {agree}/{len(judged)}; wrote results/SOFT_OILS.md")
