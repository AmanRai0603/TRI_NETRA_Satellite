#!/usr/bin/env python3
"""Fly the MATLAB twin's stored runs again with the twin of today (docs/S7_INVENTORY.md S7.17): every scenario
(matlab_sils/data/scenarios, seed 1, its own case) and every campaign the store holds (matlab_sils/store/results/<id>
with a summary.json), each run staged, then swapped in for the old one (never two full copies kept).

    python3 tools/twin_refly.py status                     what is flown, staged, running and left
    python3 tools/twin_refly.py run [--workers N] [--only ID ...] [--singles | --campaigns]
                                                            fly what is left, N GNU Octave processes at a time (3);
                                                            resumable: a run already swapped in, staged, or a campaign's
                                                            run already kept is skipped, and a run cut short goes on from
                                                            its checkpoint (asils.run 'checkpoint')

Staging: matlab_sils/store/refly/<id>/ (a scenario's run, or a campaign's run_NNNN.mat files), its checkpoints and logs
under matlab_sils/store/refly/logs/; progress in matlab_sils/store/refly/progress.log. A scenario's staged run replaces
matlab_sils/store/results/<id> when it is whole; a campaign's runs are collected (summary.json, runs.csv, its figures)
and replace matlab_sils/store/results/<id> when every run is in. A run the twin of today flew carries its version in
its manifest (asils.version: "... generated from the design"); a campaign the file refly.json.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import datetime
import json
import os
import pathlib
import shutil
import signal
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
TW = ROOT / "matlab_sils"
RES = TW / "store" / "results"
STAGE = TW / "store" / "refly"
LOGS = STAGE / "logs"
PROGRESS = STAGE / "progress.log"
PIDFILE = STAGE / "refly.pid"
MARK = "generated from the design"          # in asils.version(): the twin that flies the design's code


def log(msg):
    STAGE.mkdir(parents=True, exist_ok=True)
    line = f"{datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}  {msg}"
    with open(PROGRESS, "a") as f:
        f.write(line + "\n")
    print(line, flush=True)


def scenarios():
    out = {}
    for p in sorted((TW / "data" / "scenarios").glob("*.json")):
        s = json.loads(p.read_text())
        t = s.get("time", {})
        out[s["id"]] = {"case": s["case"], "ticks": t.get("duration_s", 600) / t.get("dt_s", 0.1),
                        "star": "img" in s["case"]}
    return out


def campaigns():
    """The campaigns the store holds (a results folder with a summary.json and a campaign definition)."""
    out = {}
    for d in sorted(RES.iterdir()):
        f = TW / "data" / "campaigns" / f"{d.name}.json"
        if (d / "summary.json").exists() and f.exists():
            c = json.loads(f.read_text())
            out[d.name] = {"runs": int(c["runs"]), "scenario": c["scenario"]}
    return out


def swapped(sid):
    m = RES / sid / "manifest.json"
    try:
        return MARK in json.loads(m.read_text()).get("engine", "")
    except (OSError, ValueError):
        return False


def camp_swapped(cid):
    return (RES / cid / "refly.json").exists()


def cost(info):
    # wall seconds, roughly: ticks at the twin's speed (a star tracker's chain is the dearest part)
    return info["ticks"] * (0.09 if info["star"] else 0.035)


def octave(code):
    return ["octave-cli", "--no-gui", "-q", "--eval", f"startup_asils; {code}"]


def swap(src, dst):
    """Put the staged folder in place of the old one: the old one goes first (one full copy at a time)."""
    if dst.exists():
        shutil.rmtree(dst)
    os.replace(src, dst)


def jobs(want_singles, want_camps, only):
    S, C = scenarios(), campaigns()
    out = []
    if want_singles:
        for sid, info in sorted(S.items(), key=lambda kv: cost(kv[1])):
            if only and sid not in only:
                continue
            if swapped(sid):
                continue
            if (STAGE / sid / "manifest.json").exists() and (STAGE / sid / "rec.mat").exists():
                out.append(("swap", sid, None))
                continue
            ck = LOGS / f"{sid}.ckpt"
            code = (f"rec = asils.run('{sid}', 'cases/{info['case']}.csv', 'quiet', true, 'checkpoint', '{ck}'); "
                    f"asils.result.save(rec, '{STAGE / sid}');")
            out.append(("single", sid, code))
    if want_camps:
        for cid, info in sorted(C.items(), key=lambda kv: kv[1]["runs"] * cost(S[kv[1]["scenario"]])):
            if only and cid not in only:
                continue
            if camp_swapped(cid):
                continue
            for k in range(1, info["runs"] + 1):
                if (STAGE / cid / f"run_{k:04d}.mat").exists():
                    continue
                code = f"asils.campaign.run('{cid}', 'runs', {k}, 'out', '{STAGE / cid}');"
                out.append(("camp", f"{cid}#{k}", code))
            out.append(("collect", cid, None))
    return out


def collect(cid):
    """Every run of the campaign in: its summary, its runs table and its figures, then in place of the old one."""
    n = json.loads((TW / "data" / "campaigns" / f"{cid}.json").read_text())["runs"]
    have = len(list((STAGE / cid).glob("run_*.mat"))) if (STAGE / cid).is_dir() else 0
    if have < n:
        log(f"campaign {cid}: {have} of {n} runs staged, not collected (a run failed: see logs/{cid}_*.log; run again)")
        return False
    code = (f"graphics_toolkit('gnuplot'); res = asils.campaign.collect('{cid}', '{STAGE / cid}'); "
            f"asils.campaign.write(res, '{STAGE / cid}'); try, asils.viz.campaign(res, '{STAGE / cid}'); catch e, disp(e.message); end")
    r = subprocess.run(octave(code), cwd=TW, capture_output=True, text=True)
    (LOGS / f"{cid}.collect.log").write_text(r.stdout + r.stderr)
    if r.returncode or not (STAGE / cid / "summary.json").exists():
        log(f"collect {cid} FAILED (see logs/{cid}.collect.log)")
        return False
    (STAGE / cid / "refly.json").write_text(json.dumps({"runs": n, "twin": MARK, "flown": datetime.datetime.now().isoformat(timespec="seconds")}) + "\n")
    swap(STAGE / cid, RES / cid)
    log(f"campaign {cid}: {n} runs collected and swapped in")
    return True


def run(a):
    STAGE.mkdir(parents=True, exist_ok=True)
    LOGS.mkdir(parents=True, exist_ok=True)
    if PIDFILE.exists():
        try:
            pid = int(PIDFILE.read_text())
            os.kill(pid, 0)
            sys.exit(f"twin_refly: already running (pid {pid}); `status` shows it")
        except (ValueError, ProcessLookupError, PermissionError):
            pass
    PIDFILE.write_text(str(os.getpid()))
    want_s = not a.campaigns
    want_c = not a.singles
    todo = jobs(want_s, want_c, set(a.only or []))
    log(f"start: {len(todo)} job(s), {a.workers} worker(s)")
    running = {}
    stop = {"now": False}
    signal.signal(signal.SIGTERM, lambda *_: stop.update(now=True))
    try:
        while (todo or running) and not stop["now"]:
            # finished ones
            for name, (p, kind, t0, fh) in list(running.items()):
                if p.poll() is None:
                    continue
                fh.close()
                del running[name]
                dt = time.time() - t0
                if kind == "single":
                    if p.returncode == 0 and (STAGE / name / "manifest.json").exists():
                        swap(STAGE / name, RES / name)
                        log(f"{name}: flown in {dt:.0f} s, swapped in")
                    else:
                        log(f"{name}: FAILED rc {p.returncode} after {dt:.0f} s (logs/{name}.log)")
                else:
                    log(f"{name}: {'flown' if p.returncode == 0 else 'FAILED rc %d' % p.returncode} in {dt:.0f} s")
            # start more
            while todo and len(running) < a.workers:
                kind, name, code = todo[0]
                if kind == "swap":
                    todo.pop(0)
                    swap(STAGE / name, RES / name)
                    log(f"{name}: staged run swapped in")
                    continue
                if kind == "collect":
                    cid = name
                    if any(n.startswith(cid + "#") for n in running) or any(t[0] == "camp" and t[1].startswith(cid + "#") for t in todo[1:]):
                        # wait for the campaign's runs; move it behind the next job
                        if len(todo) > 1:
                            todo.append(todo.pop(0))
                            continue
                        break
                    todo.pop(0)
                    collect(cid)
                    continue
                todo.pop(0)
                fh = open(LOGS / f"{name.replace('#', '_')}.log", "a")
                p = subprocess.Popen(octave(code), cwd=TW, stdout=fh, stderr=subprocess.STDOUT)
                running[name] = (p, kind, time.time(), fh)
                log(f"{name}: started (pid {p.pid})")
            time.sleep(5)
    finally:
        for name, (p, _kind, _t0, fh) in running.items():
            p.terminate()
            fh.close()
            log(f"{name}: stopped (it goes on from its checkpoint next time)")
        PIDFILE.unlink(missing_ok=True)
    log("done" if not todo and not running else "stopped")


def status(_a):
    S, C = scenarios(), campaigns()
    done = [s for s in S if swapped(s)]
    staged = [s for s in S if not swapped(s) and (STAGE / s / "manifest.json").exists()]
    left = [s for s in S if s not in done and s not in staged]
    print(f"scenarios: {len(done)} of {len(S)} flown again; staged {staged or '-'}")
    print(f"  left ({len(left)}): {' '.join(left) or '-'}")
    for cid, info in C.items():
        if camp_swapped(cid):
            print(f"campaign {cid}: flown again ({info['runs']} runs)")
        else:
            have = len(list((STAGE / cid).glob("run_*.mat"))) if (STAGE / cid).exists() else 0
            print(f"campaign {cid}: {have} of {info['runs']} runs staged")
    cks = sorted(p.name for p in LOGS.glob("*.ckpt")) if LOGS.exists() else []
    print(f"checkpoints: {' '.join(cks) or '-'}")
    if PIDFILE.exists():
        print(f"running: pid {PIDFILE.read_text().strip()}")
    if PROGRESS.exists():
        print("last:", *PROGRESS.read_text().splitlines()[-5:], sep="\n  ")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    r = sp.add_parser("run")
    r.add_argument("--workers", type=int, default=3)
    r.add_argument("--only", nargs="+")
    g = r.add_mutually_exclusive_group()
    g.add_argument("--singles", action="store_true")
    g.add_argument("--campaigns", action="store_true")
    r.set_defaults(f=run)
    sp.add_parser("status").set_defaults(f=status)
    a = ap.parse_args(argv)
    a.f(a)


if __name__ == "__main__":
    main()
