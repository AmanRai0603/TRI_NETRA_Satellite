#!/usr/bin/env python3
"""The pseudocode v2 tools (docs/PSEUDOCODE_V2.md): check a file, run a function, and make (or check)
everything generated from the physics relations (spec/physics/*.pc).

    python3 tools/pcode.py check FILE...                  units, types, every output set, no recursion
    python3 tools/pcode.py run FILE... --fn NAME --args JSON   the interpreter's outputs (SI)
    python3 tools/pcode.py gen [--check]                  write (or check) everything the pseudocode makes:
        engine/crates/adcs-physics/                       the physics (spec/physics/*.pc) in Rust: a crate,
                                                          with test vectors drawn from the interpreter
        matlab_sils/+asils/+physics/                      the physics in MATLAB, and the same vectors in
                                                          matlab_sils/data/physics_vectors.json
        engine/crates/pcode-selftest/, +asils/+pcselftest the language's own test (design/pcode_selftest)
        fsw/tests/pcode_vectors.txt                       the flight algorithms' vectors (fsw/pseudocode/*.pc),
                                                          which the C and Rust tests both read
        matlab_sils/+asils/+pc/*.m                        the MATLAB runtime they share
        design/pcode_checker.html                         the checker in the browser: one offline page
    python3 tools/pcode.py fixtures                       every seeded test vector (spec/plan/seed_content.toml)
                                                          of a physics row, run in the interpreter

`gen --check` also holds the relations to the registry: spec/plan/physics.toml names each function
with its module and arguments, and the pseudocode defines exactly those, in that order.

The language is implemented once, in JavaScript (design/js/pcode.js), so the browser's checker and
these tools are the same code; this script runs it with Node.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import pathlib
import shutil
import subprocess
import sys
import tomllib

from common import ROOT, write_text

CLI = ROOT / "design" / "js" / "pcode_cli.mjs"
REGISTRY = ROOT / "spec" / "plan" / "physics.toml"
SEED = ROOT / "spec" / "plan" / "seed_content.toml"
MATLAB_RT = ROOT / "matlab_sils" / "+asils" / "+pc"
CHECKER_TEMPLATE = ROOT / "design" / "js" / "pcode_check.template.html"
CHECKER = ROOT / "design" / "pcode_checker.html"
N_VECTORS, SEED_VECTORS = 12, 20261003
N_FSW_VECTORS, FSW_BUDGET = 32, 2000   # the flight algorithms branch more; a function of many values gets fewer

# What the pseudocode makes: each package's sources, its Rust crate, its MATLAB package, its vectors.
PACKAGES = [
    {"name": "physics", "src": ROOT / "spec" / "physics", "crate": ROOT / "engine" / "crates" / "adcs-physics", "crate_name": "adcs-physics",
     "about": "TRI-NETRA physics: the relations of spec/plan/physics.toml, translated from the pseudocode (spec/physics/*.pc) by tools/pcode.py",
     "mpkg": "asils.physics", "vectors": ROOT / "matlab_sils" / "data" / "physics_vectors.json", "registry": True},
    {"name": "selftest", "src": ROOT / "design" / "pcode_selftest", "crate": ROOT / "engine" / "crates" / "pcode-selftest", "crate_name": "pcode-selftest",
     "about": "The pseudocode's own test: every feature of the language, translated by tools/pcode.py and held to the interpreter",
     "mpkg": "asils.pcselftest", "vectors": ROOT / "matlab_sils" / "data" / "pcselftest_vectors.json", "registry": False},
]
PHYSICS = PACKAGES[0]["src"]

# The flight software's algorithms (fsw/pseudocode/*.pc) are not translated: the hand-written C
# (fsw/src) and Rust (fsw-rs/src) are held to the interpreter on its vectors, written here as
# a C header (hex floats) and a Rust table (bits), so neither test parses decimals.
FSW_SRC = ROOT / "fsw" / "pseudocode"
FSW_VECTORS = ROOT / "fsw" / "tests" / "pcode_vectors.txt"


class PcodeFailed(Exception):
    pass


def node():
    exe = shutil.which("node")
    if not exe:
        raise SystemExit("pcode: Node.js is needed (the language runs as JavaScript, the same code as the browser's checker)")
    return exe


def cli(*args, parse=True):
    r = subprocess.run([node(), str(CLI), *map(str, args)], capture_output=True, text=True)
    if r.returncode != 0:
        raise PcodeFailed((r.stderr or r.stdout).strip())
    return json.loads(r.stdout) if parse else r.stdout


def sources(pkg=None):
    return sorted((pkg["src"] if pkg else PHYSICS).glob("*.pc"))


def mdir(pkg):
    return ROOT / "matlab_sils" / pathlib.Path(*["+" + x for x in pkg["mpkg"].split(".")])


def registry_problems(sigs):
    """spec/plan/physics.toml against the pseudocode's functions: the same set, the same arguments."""
    reg = {f"{f['module']}::{f['name']}": f["args"] for f in tomllib.loads(REGISTRY.read_text())["function"]}
    have = {f"{s['module']}::{s['name']}": [i["name"] for i in s["inputs"]] for s in sigs}
    errs = [f"physics.toml names {k}, which spec/physics does not define" for k in sorted(set(reg) - set(have))]
    for k in sorted(set(reg) & set(have)):
        if reg[k] != have[k]:
            errs.append(f"{k}: physics.toml says ({', '.join(reg[k])}), the pseudocode ({', '.join(have[k])})")
    # helpers the relations use (env::dipole, orbit::sun_ra_dec, the atmosphere table) need not be registered
    return errs


def generated():
    """{path: text} of every generated file, every package."""
    out = {}
    for rel, text in cli("matlab-rt").items():
        out[MATLAB_RT / rel] = text
    for pkg in PACKAGES:
        files = [str(f) for f in sources(pkg)]
        for rel, text in cli("rust", *files, "--title", pkg["about"]).items():
            out[pkg["crate"] / rel] = text
        out[pkg["crate"] / "Cargo.toml"] = CARGO.format(name=pkg["crate_name"], about=pkg["about"])
        out[pkg["crate"] / "tests" / "vectors.rs"] = VECTORS_RS.replace("CRATE", pkg["crate_name"].replace("-", "_"))
        for rel, text in cli("matlab", *files, "--pkg", pkg["mpkg"]).items():
            out[mdir(pkg) / rel] = text
        vec = cli("vectors", *files, "--n", N_VECTORS, "--seed", SEED_VECTORS)
        text = json.dumps({"generated_by": "tools/pcode.py gen (the interpreter, design/js/pcode.js)", "vectors": vec}, indent=1, sort_keys=True) + "\n"
        out[pkg["crate"] / "tests" / "vectors.json"] = text
        out[pkg["vectors"]] = text
    fsw = cli("vectors", *map(str, sorted(FSW_SRC.glob("*.pc"))), "--n", N_FSW_VECTORS, "--budget", FSW_BUDGET, "--seed", SEED_VECTORS)
    out[FSW_VECTORS] = fsw_vectors(fsw)
    out[CHECKER] = checker_page()
    return out


def _double(hi_lo):
    """A double from its [high, low] 32-bit words, as the interpreter wrote it."""
    import struct
    return struct.unpack(">d", struct.pack(">II", *hi_lo))[0]


def _flat(vectors):
    """Every call as (name, exact, set number, inputs, outputs): a fn's set is one call, a proc's a run."""
    rows, k = [], 0
    for name in sorted(vectors):
        e = vectors[name]
        for s in e["sets"]:
            for c in (s["calls"] if e.get("proc") else [s]):
                rows.append((name, e["exact"], k, [_double(b) for b in c["in_bits"]], [_double(b) for b in c["out_bits"]]))
            k += 1
    return rows


def fsw_vectors(vectors):
    """The flight algorithms' vectors as one text file the C and Rust tests both read, every value
    its exact 64-bit pattern in hex: a header line, then a line per call,
    `name exact set nx ny` and the nx + ny words."""
    import struct
    word = lambda v: "%016x" % struct.unpack("<Q", struct.pack("<d", v))[0]
    lines = ["# Generated by tools/pcode.py gen from the .pc files of fsw/pseudocode (the interpreter, design/js/pcode.js); "
             "do not edit. A line per call: name exact set nx ny, then the inputs and outputs as IEEE-754 bits."]
    for (n, ex, k, x, y) in _flat(vectors):
        lines.append(" ".join([n, str(int(ex)), str(k), str(len(x)), str(len(y))] + [word(v) for v in x] + [word(v) for v in y]))
    return "\n".join(lines) + "\n"


CARGO = """# Generated by tools/pcode.py gen; do not edit.
[package]
name = "{name}"
description = "{about}"
version.workspace = true
edition.workspace = true
authors.workspace = true
publish.workspace = true

[dev-dependencies]
# read every decimal to the double it was written from
serde_json = {{ version = "1", features = ["float_roundtrip"] }}
"""

VECTORS_RS = """//! Translator = interpreter: every vector the interpreter (design/js/pcode.js) drew, through the Rust
//! translation. Generated by tools/pcode.py gen; do not edit.
//!
//! The arithmetic is the same on both sides (x^k by repeated multiplication, sums from the left,
//! min/max written out), so a function that uses no transcendental (directly or through what it
//! calls) must agree bit for bit. A sin, exp, log or atan2 may differ in its last bits between the
//! platform's maths library and JavaScript's (more after a large argument is reduced), so those are
//! held to 1e-12 relative, and the test reports how many values are bit for bit.
//! A proc's vector is a run of calls, its state carried from each call to the next.

fn nums(v: &serde_json::Value) -> Vec<f64> {
    v.as_array().unwrap().iter().map(|a| a.as_f64().unwrap()).collect()
}

#[test]
fn every_vector_of_the_interpreter_is_reproduced() {
    let v: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let (mut n, mut values, mut exact, mut worst) = (0, 0, 0, 0.0f64);
    for (name, entry) in v["vectors"].as_object().unwrap() {
        let must_be_exact = entry["exact"].as_bool().unwrap();
        let sets = entry["sets"].as_array().unwrap();
        assert!(!sets.is_empty(), "{name}: no vector drawn");
        for s in sets {
            let (got, want): (Vec<Vec<f64>>, Vec<Vec<f64>>) = if entry["proc"].as_bool().unwrap_or(false) {
                let calls = s["calls"].as_array().unwrap();
                let xs: Vec<Vec<f64>> = calls.iter().map(|c| nums(&c["in"])).collect();
                let got = CRATE::dispatch::call_seq(name, &xs).unwrap_or_else(|| panic!("{name}: no such proc"));
                (got, calls.iter().map(|c| nums(&c["out"])).collect())
            } else {
                let got = CRATE::dispatch::call(name, &nums(&s["in"])).unwrap_or_else(|| panic!("{name}: no such function"));
                (vec![got], vec![nums(&s["out"])])
            };
            for (g, w) in got.iter().zip(&want) {
                assert_eq!(g.len(), w.len(), "{name}: output count");
                for (g, w) in g.iter().zip(w) {
                    let err = (g - w).abs() / w.abs().max(1e-300);
                    worst = worst.max(err);
                    values += 1;
                    if g.to_bits() == w.to_bits() { exact += 1; }
                    assert!(g.to_bits() == w.to_bits() || (!must_be_exact && err < 1e-12),
                            "{name}: Rust {g:e}, interpreter {w:e}{}", if must_be_exact { " (no transcendental: must agree bit for bit)" } else { "" });
                }
            }
            n += 1;
        }
    }
    println!("{n} vectors, {values} values, {exact} bit for bit, worst relative difference {worst:e}");
}
"""


def checker_page():
    """The browser's checker: the template with design/js/pcode.js inlined (a page opened from disk
    cannot import a module), so it is the same code the tools run."""
    js = (ROOT / "design" / "js" / "pcode.js").read_text()
    js = "\n".join(line[len("export "):] if line.startswith("export ") else line for line in js.splitlines())
    page = CHECKER_TEMPLATE.read_text()
    if "/*PCODE*/" not in page:
        raise PcodeFailed(f"{CHECKER_TEMPLATE.relative_to(ROOT)} has no /*PCODE*/ marker")
    return page.replace("/*PCODE*/", js.replace("</script", "<\\/script"))


def gen(check_only):
    errs = []
    try:
        for pkg in PACKAGES:
            cli("check", *map(str, sources(pkg)), parse=False)
        cli("check", *map(str, sorted(FSW_SRC.glob("*.pc"))), parse=False)
        sigs = cli("signatures", *map(str, sources()))
        files = generated()
    except PcodeFailed as e:
        return [str(e)]
    errs += registry_problems(sigs)
    owned = [MATLAB_RT] + [d for pkg in PACKAGES for d in (pkg["crate"] / "src", mdir(pkg))]
    stale = [p for d in owned if d.exists() for p in d.rglob("*") if p.is_file() and p not in files]
    for path, text in sorted(files.items()):
        have = path.read_text() if path.is_file() else None
        if have == text:
            continue
        if check_only:
            errs.append(f"{path.relative_to(ROOT)} is not what the pseudocode makes (run tools/pcode.py gen)")
        else:
            write_text(path, text)
    for p in stale:
        if check_only:
            errs.append(f"{p.relative_to(ROOT)} is generated by nothing now (run tools/pcode.py gen)")
        else:
            p.unlink()
    return errs


def fixtures():
    """Every seeded fixture of a physics row, through the interpreter: (label, ok, line)."""
    rows = tomllib.loads(SEED.read_text())["row"]
    sigs = {f"{s['module']}::{s['name']}": s for s in cli("signatures", *map(str, sources()))}
    out = []
    for r in rows:
        if "physics" not in r:
            continue
        for fx in r.get("fixture", []):
            s = sigs.get(r["physics"])
            if s is None:
                out.append((fx["label"], False, f"{r['tree_id']}: {r['physics']} is not defined"))
                continue
            names = [i["name"] for i in s["inputs"]]
            if set(names) != set(fx["inputs"]):
                out.append((fx["label"], False, f"{r['tree_id']}: the fixture's inputs {sorted(fx['inputs'])} are not {names}"))
                continue
            args = [round(fx["inputs"][n]) if s["inputs"][k]["type"] == "int" else fx["inputs"][n] for k, n in enumerate(names)]
            got = cli("run", *map(str, sources()), "--fn", s["name"], "--args", json.dumps(args))[0]
            rel = abs(got - fx["expect"]) / abs(fx["expect"])
            ok = rel <= fx["tolerance"]
            out.append((fx["label"], ok, f"{r['tree_id']} {r['physics']}: {got:.6g} against {fx['expect']:.6g} "
                        f"({rel*100:.2f} % of {fx['tolerance']*100:.0f} %), {fx['source']}"))
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("check", help="check files")
    c.add_argument("files", nargs="+")
    r = sub.add_parser("run", help="run a function in the interpreter")
    r.add_argument("files", nargs="+")
    r.add_argument("--fn", required=True)
    r.add_argument("--args", default="[]", help="the inputs, SI, as a JSON list")
    g = sub.add_parser("gen", help="write (or check) everything the physics makes")
    g.add_argument("--check", action="store_true")
    sub.add_parser("fixtures", help="the seeded test vectors of the physics rows, in the interpreter")
    a = ap.parse_args(argv)
    try:
        if a.cmd == "check":
            print(cli("check", *a.files, parse=False).strip())
            return 0
        if a.cmd == "run":
            print(json.dumps(cli("run", *a.files, "--fn", a.fn, "--args", a.args)))
            return 0
    except PcodeFailed as e:
        print(str(e), file=sys.stderr)
        return 1
    if a.cmd == "fixtures":
        res = fixtures()
        for _, ok, line in res:
            print(("ok    " if ok else "FAIL  ") + line)
        bad = sum(1 for _, ok, _ in res if not ok)
        print(f"pcode: {len(res)} fixture(s), {bad} outside their tolerance")
        return 1 if bad else 0
    errs = gen(a.check)
    for e in errs:
        print("pcode: " + e, file=sys.stderr)
    print(f"pcode: {len(PACKAGES)} packages ({', '.join(p['name'] for p in PACKAGES)}) {'checked' if a.check else 'generated'}, {len(errs)} problem(s)")
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
