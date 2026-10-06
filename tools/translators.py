#!/usr/bin/env python3
"""The translators held to the interpreter (docs/PLAN_2_0.md S5): every package of pseudocode translated to Rust, C and
MATLAB, built, and run on every vector the interpreter (design/js/pcode.js) drew for it.

    python3 tools/translators.py [--lang rust c matlab] [--package physics selftest fsw groups] [--out DIR] [--keep DIR]

For each function and proc of each package: the interpreter's vectors (inputs and outputs as exact IEEE-754 bits; a
proc's as runs of calls, its state carried); each translation called on the same inputs. An exact function (one that
calls no transcendental: sin, cos, tan, asin, acos, atan, atan2, exp, log, log10, pow) must give the same bits; any
other within 1e-12 relative (the maths library's last bit may differ from the browser's). A function a translation
cannot build, or a value outside its tolerance, is named.

The packages:
  physics    the relations (the design's spec/physics, tools/from_design.py)
  selftest   the language's own test (design/pcode_selftest): every construct
  fsw        the flight software's algorithms (fsw/pseudocode 01-09)
  groups     each group's generated code (design/groups), when there is one

What it writes: results/TRANSLATORS.md and translators.json (per package and language: functions, values, how many bit
for bit, the worst relative error, every failure).

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import pathlib
import shutil
import struct
import subprocess
import sys
import tempfile

import from_design
from common import ROOT, write_text

CLI = ROOT / "design" / "js" / "pcode_cli.mjs"
SEED = 20261003
TOL = 1e-12


def packages():
    fsw = ROOT / "fsw" / "pseudocode"
    return {
        "physics": {"files": sorted(from_design.folder("spec/physics/").glob("*.pc")), "n": 12, "budget": 1e9},
        "selftest": {"files": sorted((ROOT / "design" / "pcode_selftest").glob("*.pc")), "n": 12, "budget": 1e9},
        "fsw": {"files": sorted(fsw.glob("*.pc")), "n": 32, "budget": 2000},
        "groups": {"files": sorted((ROOT / "design" / "groups").rglob("*.pc")), "n": 12, "budget": 1e9},
    }


def node(*args):
    r = subprocess.run(["node", str(CLI), *map(str, args)], capture_output=True, text=True)
    if r.returncode:
        raise RuntimeError((r.stderr or r.stdout).strip()[-2000:])
    return json.loads(r.stdout)


def double(hi_lo):
    return struct.unpack(">d", struct.pack(">II", *hi_lo))[0]


def bits64(x):
    return struct.unpack(">Q", struct.pack(">d", x))[0]


def calls_file(vec, path):
    """The vectors as a text the harnesses read: per entry `name proc nsets`, per set `ncalls`, per call `nx x... ny`
    with every value as its 64 bits in hex (no decimal parsing anywhere)."""
    L = []
    for name, e in vec.items():
        sets = e["sets"]
        L.append(f"{name} {1 if e.get('proc') else 0} {len(sets)}")
        for s in sets:
            calls = s["calls"] if e.get("proc") else [s]
            L.append(str(len(calls)))
            for c in calls:
                xs = [f"{bits64(double(b)):016x}" for b in c["in_bits"]]
                L.append(" ".join([str(len(xs)), *xs, str(len(c["out_bits"]))]))
    write_text(path, "\n".join(L) + "\n")


def compare(vec, got_text, whole=None):
    """(functions, values, bit for bit, worst relative error, [failures]) of a harness's answers against the vectors.
    The harness prints, per entry, `name` then per call one line: `ok y...` (hex bits) or `none` (it has no such
    function, or refused the inputs)."""
    lines = iter(got_text.splitlines())
    fns = vals = exact = 0
    worst, fails = 0.0, []
    for name, e in vec.items():
        head = next(lines, None)
        if head != name:
            fails.append(f"{name}: the harness answered {head!r}")
            break
        bad = None
        for s in e["sets"]:
            for c in (s["calls"] if e.get("proc") else [s]):
                ln = next(lines, "").split()
                if not ln or ln[0] != "ok":
                    bad = bad or "no answer (not translated, or refused the inputs)"
                    continue
                got = [struct.unpack(">d", bytes.fromhex(h))[0] for h in ln[1:]]
                want = [double(b) for b in c["out_bits"]]
                if len(got) != len(want):
                    bad = bad or f"{len(got)} outputs, the interpreter {len(want)}"
                    continue
                ints = (whole or {}).get(name) or [False] * len(want)
                for g, w, is_int in zip(got, want, ints):
                    vals += 1
                    if bits64(g) == bits64(w) or (is_int and g == w):   # a whole number has no sign of zero
                        exact += 1
                        continue
                    if e["exact"]:
                        bad = bad or f"not bit for bit: {g!r} against {w!r}"
                        continue
                    err = abs(g - w) / max(abs(w), 1e-300)
                    worst = max(worst, err)
                    if not err <= TOL:
                        bad = bad or f"{g!r} against {w!r} (relative {err:.3g})"
        fns += 1
        if bad:
            fails.append(f"{name}: {bad}")
    return fns, vals, exact, worst, fails


# ------------------------------------------------------------------------------------------------ Rust
RUST_MAIN = r"""// The vector harness (tools/translators.py): reads the calls, answers each with the translation.
#![allow(clippy::all)]
mod rt;
mod dispatch;
MODS
use std::io::{self, BufRead, Write};
fn main() {
    let stdin = io::stdin();
    let mut it = stdin.lock().lines().map(|l| l.unwrap());
    let out = io::stdout();
    let mut o = io::BufWriter::new(out.lock());
    while let Some(head) = it.next() {
        let h: Vec<&str> = head.split_whitespace().collect();
        if h.is_empty() { continue; }
        let (name, proc, nsets) = (h[0].to_string(), h[1] == "1", h[2].parse::<usize>().unwrap());
        writeln!(o, "{}", name).unwrap();
        for _ in 0..nsets {
            let ncalls: usize = it.next().unwrap().trim().parse().unwrap();
            let mut calls = Vec::new();
            for _ in 0..ncalls {
                let l = it.next().unwrap();
                let w: Vec<&str> = l.split_whitespace().collect();
                let nx: usize = w[0].parse().unwrap();
                calls.push(w[1..1 + nx].iter().map(|s| f64::from_bits(u64::from_str_radix(s, 16).unwrap())).collect::<Vec<f64>>());
            }
            let answers: Vec<Option<Vec<f64>>> = if proc {
                match dispatch::call_seq(&name, &calls) { Some(v) => v.into_iter().map(Some).collect(), None => vec![None; ncalls] }
            } else {
                calls.iter().map(|x| dispatch::call(&name, x)).collect()
            };
            for a in answers {
                match a {
                    Some(y) => { let s: Vec<String> = y.iter().map(|v| format!("{:016x}", v.to_bits())).collect(); writeln!(o, "ok {}", s.join(" ")).unwrap(); }
                    None => writeln!(o, "none").unwrap(),
                }
            }
        }
    }
}
"""


def run_rust(files, work, calls):
    src = work / "rust"
    gen = node("rust", *files, "--title", "translator check")
    for rel, text in gen.items():
        write_text(src / rel, text)
    mods = [p for p in gen if p.startswith("src/") and p not in ("src/rt.rs", "src/dispatch.rs", "src/lib.rs")]
    main = RUST_MAIN.replace("MODS", "".join(f"mod {pathlib.Path(p).stem};\n" for p in mods))
    write_text(src / "src" / "main.rs", main)
    exe = work / "rust_harness"
    r = subprocess.run(["rustc", "--edition", "2021", "-O", "-C", "debuginfo=0", "-A", "warnings", "-o", str(exe), str(src / "src" / "main.rs")],
                       capture_output=True, text=True)
    if r.returncode:
        return None, "the Rust does not build: " + r.stderr.strip()[-1500:]
    r = subprocess.run([str(exe)], stdin=open(calls), capture_output=True, text=True)
    return r.stdout, None if r.returncode == 0 else f"the harness stopped: {r.stderr.strip()[-500:]}"


C_MAIN = r"""/* The vector harness (tools/translators.py): reads the calls, answers each with the translation. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "pcode.h"

static double from_hex(const char *s) { uint64_t u = strtoull(s, NULL, 16); double d; memcpy(&d, &u, 8); return d; }
static void put(double d) { uint64_t u; memcpy(&u, &d, 8); printf(" %016llx", (unsigned long long)u); }

int main(void) {
    static char line[1 << 20];
    static double xs[64 * 512], outs[64 * 256];
    char name[256];
    while (fgets(line, sizeof line, stdin)) {
        int proc, nsets, s;
        if (sscanf(line, "%255s %d %d", name, &proc, &nsets) != 3) continue;
        printf("%s\n", name);
        for (s = 0; s < nsets; s++) {
            int ncalls, c, nx = 0;
            if (!fgets(line, sizeof line, stdin)) return 1;
            ncalls = atoi(line);
            for (c = 0; c < ncalls; c++) {
                char *tok;
                int i;
                if (!fgets(line, sizeof line, stdin)) return 1;
                tok = strtok(line, " \n");
                nx = atoi(tok);
                for (i = 0; i < nx; i++) { tok = strtok(NULL, " \n"); xs[c * nx + i] = from_hex(tok); }
            }
            if (proc) {
                int ny = 0;
                if (pc_call_seq(name, xs, ncalls, nx, outs, &ny) != 0) { for (c = 0; c < ncalls; c++) printf("none\n"); continue; }
                for (c = 0; c < ncalls; c++) { int j; printf("ok"); for (j = 0; j < ny; j++) put(outs[c * 256 + j]); printf("\n"); }
            } else {
                for (c = 0; c < ncalls; c++) {
                    int ny = 0, j;
                    if (pc_call(name, xs + c * nx, nx, outs, &ny) != 0) { printf("none\n"); continue; }
                    printf("ok"); for (j = 0; j < ny; j++) put(outs[j]); printf("\n");
                }
            }
        }
    }
    return 0;
}
"""
# the flight software's flags (fsw/Makefile): C99, no fused multiply-add, no fast maths; every warning an error but
# an unused local or parameter (a translation keeps every name the pseudocode declares)
CFLAGS = ["-std=c99", "-O2", "-Wall", "-Wextra", "-pedantic", "-Werror", "-ffp-contract=off", "-fno-fast-math",
          "-Wno-unused-variable", "-Wno-unused-but-set-variable", "-Wno-unused-parameter"]


def run_c(files, work, calls):
    src = work / "c"
    gen = node("c", *files, "--title", "translator check", "--lib", "pcode")
    for rel, text in gen.items():
        write_text(src / rel, text)
    write_text(src / "main.c", C_MAIN)
    exe = work / "c_harness"
    cs = sorted(str(p) for p in (src / "src").glob("*.c")) + [str(src / "main.c")]
    r = subprocess.run(["gcc", *CFLAGS, "-I", str(src / "include"), "-o", str(exe), *cs, "-lm"], capture_output=True, text=True)
    if r.returncode:
        return None, "the C does not build: " + r.stderr.strip()[-2500:]
    r = subprocess.run([str(exe)], stdin=open(calls), capture_output=True, text=True)
    return r.stdout, None if r.returncode == 0 else f"the harness stopped: {r.stderr.strip()[-500:]}"


MATLAB_MAIN = r"""function harness(infile, outfile)
%HARNESS  The vector harness (tools/translators.py): reads the calls, answers each with the translation.
    fin = fopen(infile, 'r'); fout = fopen(outfile, 'w');
    while true
        head = fgetl(fin);
        if ~ischar(head), break; end
        h = strsplit(strtrim(head));
        if numel(h) < 3, continue; end
        name = h{1}; proc = strcmp(h{2}, '1'); nsets = str2double(h{3});
        fprintf(fout, '%s\n', name);
        for s = 1:nsets
            ncalls = str2double(fgetl(fin));
            X = {};
            for c = 1:ncalls
                w = strsplit(strtrim(fgetl(fin)));
                nx = str2double(w{1});
                x = zeros(nx, 1);
                for i = 1:nx, x(i) = hex2num(w{1 + i}); end
                X{c} = x;
            end
            try
                if proc
                    if ncalls > 0 && ~isempty(X{1}), Y = tc.call_seq(name, [X{:}]); else, Y = tc.call_seq(name, zeros(0, ncalls)); end
                    for c = 1:ncalls, wr(fout, Y(:, c)); end
                else
                    for c = 1:ncalls, wr(fout, tc.call(name, X{c})); end
                end
            catch err
                for c = 1:ncalls, fprintf(fout, 'none\n'); end
            end
        end
    end
    fclose(fin); fclose(fout);
end

function wr(fout, y)
    fprintf(fout, 'ok');
    for j = 1:numel(y), fprintf(fout, ' %s', num2hex(double(y(j)))); end
    fprintf(fout, '\n');
end
"""


def run_matlab(files, work, calls):
    src = work / "matlab"
    shutil.rmtree(src, ignore_errors=True)
    gen = node("matlab", *files, "--pkg", "tc")
    for rel, text in gen.items():
        write_text(src / "+tc" / rel, text)
    for rel, text in node("matlab-rt").items():
        write_text(src / "+asils" / "+pc" / rel, text)
    write_text(src / "harness.m", MATLAB_MAIN)
    out = work / "matlab_answers.txt"
    r = subprocess.run(["octave-cli", "--no-gui", "-q", "--eval", f"harness('{calls}', '{out}')"], cwd=src, capture_output=True, text=True, timeout=7200)
    if r.returncode or not out.is_file():
        return None, "Octave stopped: " + (r.stderr or r.stdout).strip()[-1500:]
    return out.read_text(), None


LANGS = {"rust": run_rust, "c": run_c, "matlab": run_matlab}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--lang", nargs="+", default=list(LANGS))
    ap.add_argument("--package", nargs="+", default=None)
    ap.add_argument("--out", default=str(ROOT / "results"))
    ap.add_argument("--keep", default=None, help="keep the generated code and harnesses here")
    a = ap.parse_args(argv)
    pk = packages()
    names = a.package or [p for p in pk if pk[p]["files"]]
    rows, bad = [], 0
    with tempfile.TemporaryDirectory() as t:
        work_root = pathlib.Path(a.keep) if a.keep else pathlib.Path(t)
        for p in names:
            spec = pk[p]
            if not spec["files"]:
                continue
            work = work_root / p
            shutil.rmtree(work, ignore_errors=True)
            work.mkdir(parents=True)
            vec = node("vectors", *spec["files"], "--n", spec["n"], "--budget", int(spec["budget"]), "--seed", SEED)
            calls_file(vec, work / "calls.txt")
            whole = node("outkinds", *spec["files"])
            for lang in a.lang:
                text, err = LANGS[lang](spec["files"], work, work / "calls.txt")
                if err:
                    rows.append({"package": p, "lang": lang, "functions": len(vec), "values": 0, "exact": 0, "worst": None, "failures": [err]})
                    bad += 1
                    print(f"translators: {p} {lang}: {err[:300]}", flush=True)
                    continue
                fns, vals, ex, worst, fails = compare(vec, text, whole)
                rows.append({"package": p, "lang": lang, "functions": fns, "values": vals, "exact": ex, "worst": worst, "failures": fails})
                bad += bool(fails)
                print(f"translators: {p} {lang}: {fns} functions, {vals} values, {ex} bit for bit, worst {worst:.3g}"
                      + (f"; {len(fails)} failing: {fails[:3]}" if fails else ""), flush=True)
    out = pathlib.Path(a.out)
    write_text(out / "translators.json", json.dumps(rows, indent=1) + "\n")
    L = ["# The translators, held to the interpreter", "",
         "**In one line:** every package of pseudocode translated to each language, built, and run on every vector the "
         "interpreter drew; an exact function bit for bit, any other within 1e-12 relative (`tools/translators.py`, "
         "`docs/PLAN_2_0.md` S5).", "",
         "| Package | Language | Functions | Values | Bit for bit | Worst relative error | Failing |", "|---|---|---|---|---|---|---|"]
    for r in rows:
        L.append(f"| {r['package']} | {r['lang']} | {r['functions']} | {r['values']} | {r['exact']} | "
                 f"{'—' if r['worst'] is None else format(r['worst'], '.3g')} | {len(r['failures'])} |")
    for r in rows:
        if r["failures"]:
            L += ["", f"**{r['package']}, {r['lang']}:**", ""] + [f"- {x}" for x in r["failures"][:40]]
    write_text(out / "TRANSLATORS.md", "\n".join(L) + "\n")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
