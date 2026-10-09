#!/usr/bin/env python3
"""The engine build (docs/PLAN_2_0.md S7; docs/S7_INVENTORY.md S7.3-S7.16): the time engine's published models and relations
written from the design. Only the engine's core stays hand-written (step order, recorder, integrators, the toolbox); a
model of the world or of the spacecraft is a method of an env, dyn, act or sens node, and its Rust is generated here, never
edited.

    python3 tools/engine_build.py gen [--check] [--design FILE]
    python3 tools/engine_build.py modules [--design FILE]      which modules each target takes, and from where

What a target takes: every method block of the design whose `code.generate` names the target, and every module its
`code.uses` names, in turn (a published model's data, env's onboard frames and field, the physics' tables, the
toolbox; a bare name, as the flight algorithms name theirs, is its user's neighbour). A module is found by the path its
node's origin names (tools/from_design.py `text`), or, for the toolbox (fsw/pseudocode/01_math.pc), in the repository. The library's translator (trinetra-pcode, `tndb translate`; the
JavaScript one, byte for byte the same, when the library's command is not built) writes each target as a module of
its crate:

  engine/crates/adcs-sim-core/src/gen/   the engine's truth environment: calendar and sidereal time, the field along the
                                         orbit, the fast orbit's atmosphere, elements to state (S7.3); the box's faces
                                         and the disturbance torques, the fast Sun, its pressure and the Moon, the
                                         shadow and the eclipse fraction (S7.4); the plant's rate, the rotors' geometry,
                                         the flexible mode and the total momentum (dyn, S7.5); the fast orbit's forces
                                         and node context (S7.6); the actuators: the coils, the momentum devices and the
                                         thrusters, their draws the language's streams (act, S7.7); the sensors: the
                                         gyro, the magnetometer, the coarse and fine Sun sensors, the Earth sensor, the
                                         GNSS receiver, and the rotors' telemetry (sens and act, S7.8); the star
                                         tracker's unit, its onboard table and its attitude (sens, S7.9), its image
                                         chain: the frame, the spots, their identification and the image model (sens,
                                         S7.10, over buffers whose length is the caller's); the plant's state at the
                                         start of a run (dyn, S7.11); the device emulators' scaling, the inverse of the
                                         flight software's drivers, with the drivers' module it takes it from (oils,
                                         S7.12); the three-axis angle of a metric's channels, with the same
                                         portable maths the toolbox's qangle flew (kpi, S7.14b); its scalar maths
                                         from crate::pm (the pure-Rust libm: the same trajectory on every target), no_std
  engine/crates/adcs-pop/src/gen/        the precision orbit's time scales, geodetic coordinates, Earth frames, the
                                         IAU 2006/2000A kernel and the tidal EOP models (S7.3); the atmosphere
                                         (DTM2020, JB2008, the exponential, the switch) and the space-weather indices
                                         (S7.3b); DE440 (S7.3c); gravity and the tides (S7.3d); relativity (S7.3e); the
                                         spacecraft force models (third body, gas-surface interaction, drag, solar and
                                         Earth radiation pressure), the force set, its sum and the sun-synchronous start
                                         (S7.6); the EOP splice and interpolation of the frame builds A, B and C
                                         (S7.19b); std maths (as the Octave POP they are held to)
  engine/crates/adcs-sim/src/gen/        the engine's relations that fly with the platform's maths: the fast orbit's
                                         start from the LTAN (S7.6), what the sensors see of the sky (S7.8), the
                                         set-up from the case: the centre of mass's offset, the truth plant, the
                                         case's orbit and epoch (S7.11); the flight software's parameters: its laws
                                         of gain, the LQR's weights, its choices of law and the copies of the
                                         product's, the case's and the constants' values (fsw, S7.13; the guidance
                                         module it takes the payload offset from); the power system (design), the
                                         rotors' jitter and the pointing budget (pnt, S7.14); how each metric is
                                         measured from a run: its channels, windows, statistics, the ECSS indices,
                                         each kind's value and unit and its verdict (kpi, S7.14b); std maths, and the
                                         translator's dispatcher, by which `adcs design call` serves the case's orbit
                                         period (caseorbit) to the tools that size a run by it (S7.19b)
  engine/crates/adcs-design/src/gen/     the sizing (design and act, S7.15): the demand survey's reductions and what the
                                         case asks of an actuator, the magnetorquer coil, the momentum actuators chosen
                                         from the catalogue, the fluid rings and their electromagnetic pump, where a ring
                                         lies in the box, the thrusters, the sensor suite, the budget; the catalogue's
                                         derive rule (catalogue); the design loop's rules (design); the Floquet
                                         certificate of the coils-only loop (ctl, S7.15b); std maths, and the
                                         translator's dispatcher, by which `adcs design call` serves a method by name to
                                         the tools (tools/design_call.py)
  matlab_sils/+asils/+models/            the same models for the MATLAB twin (asils.models.<module>.<function>), one
                                         package of every module the engine's targets take, over the twin's shared
                                         runtime +asils/+pc
  engine/crates/adcs-relations/          the relations (S7.16), a crate generated whole: the design's relations library
                                         (1.0.0's spec/physics, its lib_spec_physics_* nodes) and every group's computing
                                         rows as tools/groupcode.py wires them (design/groups), each relation once: an item
                                         of a group's module the library holds word for word is the library's, and the
                                         group's rows call it there (modules of one name, env, ctl and risk, are one module);
                                         under src/gen with the translator's dispatcher (std maths), src/wasm.rs the groups'
                                         test apps' WebAssembly face (tools/groupcode.py deliver), and its tests: each
                                         package's vectors as the interpreter draws them from the package's own files
                                         (physics_vectors.json, groups_vectors.json, the twin's copies in matlab_sils/data;
                                         the groups' `library` names where the crate holds a group's copy), and the nodes'
                                         own test vectors (fixtures.json)
  matlab_sils/+asils/+relations/         the same relations for the twin, one package

Every file it writes says so in its first line. --check exits 1 when one is not what the design gives, or a file in a
generated folder is not one the design gives.

Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import json
import pathlib
import posixpath
import re
import sqlite3
import subprocess
import sys
import tempfile

import from_design
from common import ROOT, write_text

TNDB = ROOT / "engine" / "target" / "release" / "tndb"
TITLE = "TRI-NETRA engine models, written from the design by tools/engine_build.py"
# where each target goes, and how: the crate's module path and the module its scalar maths come from (None: std)
TARGETS = {
    "adcs-sim-core": {"dir": "engine/crates/adcs-sim-core/src/gen", "root": "crate::gen", "math": "crate::pm"},
    "adcs-pop": {"dir": "engine/crates/adcs-pop/src/gen", "root": "crate::gen", "math": None},
    "adcs-sim": {"dir": "engine/crates/adcs-sim/src/gen", "root": "crate::gen", "math": None, "dispatch": True},
    "adcs-design": {"dir": "engine/crates/adcs-design/src/gen", "root": "crate::gen", "math": None, "dispatch": True},
}
# the MATLAB twin's package: every module the engine's targets take, one copy, beside the twin's own code
TWIN = {"dir": "matlab_sils/+asils/+models", "pkg": "asils.models", "takes": list(TARGETS)}
TOOLBOX = ("fsw/pseudocode/01",)       # the toolbox's pseudocode: code, read from the repository
# The relations (S7.16): the design's relations library (1.0.0's spec/physics, its lib_spec_physics_* nodes) and every
# group's computing rows as tools/groupcode.py wires them (design/groups), one crate and one MATLAB package, each relation
# once: a group's item the library holds word for word is the library's (the group's rows call it there). The crate is
# generated whole (its Cargo.toml, its lib.rs, its WebAssembly face for the groups' test apps, its tests), with the
# translator's dispatcher; each package's vectors are drawn by the interpreter as the package's own files give them.
RELATIONS = {
    "crate": "engine/crates/adcs-relations", "name": "adcs-relations", "pkg": "asils.relations", "mdir": "matlab_sils/+asils/+relations",
    "about": "TRI-NETRA relations: the design's relations (spec/physics) and every group's computing rows (design/groups), "
             "each once, written from the design by tools/engine_build.py",
    "library": "spec/physics/", "groups": ROOT / "design" / "groups",
    # the vectors: the package, how many a function, the twin's copy (the crate's own beside its tests)
    "vectors": {"physics": (12, "matlab_sils/data/physics_vectors.json"), "groups": (8, "matlab_sils/data/groups_vectors.json")},
}
SEED_VECTORS = 20261003
SCALAR = re.compile(r"^real(\[[^\]]*\])?(\s+in\s.*)?$")     # a plain number with its unit, not an array
PLAIN = re.compile(r"^(real|int|bool)(\[[^\]]*\])*$")     # a value the dispatcher takes and gives as numbers
DECL = re.compile(r"^(fn|proc|record|table|const|data|choice)\s+([A-Za-z_][A-Za-z0-9_]*)")


def blocks(db=None):
    """[(node, path of its module, [targets], [paths it uses])] of every method block that names a target."""
    out = []
    with sqlite3.connect(f"file:{db or from_design.design()}?mode=ro", uri=True) as c:
        for nid, x in c.execute("SELECT id, content FROM design_node ORDER BY id"):
            body = json.loads(x)["body"]
            rows = {(s, f): (v, o) for s, f, v, o in body["content"]}
            if ("code", "generate") not in rows or ("code", "pseudocode") not in rows:
                continue
            path = rows[("code", "pseudocode")][1].split(" ")[0]
            gen = [t.strip() for t in rows[("code", "generate")][0].split(",") if t.strip()]
            uses = json.loads(rows[("code", "uses")][0]) if ("code", "uses") in rows else []
            out.append((nid, path, gen, uses))
    return out


def module_text(path, db=None):
    if path.startswith(TOOLBOX):
        f = ROOT / path
        if not f.is_file():
            raise SystemExit(f"engine_build: the toolbox module {path} is not in the repository")
        return f.read_text(encoding="utf-8")
    return from_design.text(path, db)


def modules(target, db=None):
    """{path: text} of every module a target takes: the blocks that name it and, in turn, what they use."""
    bs = blocks(db)
    uses = {p: u for _n, p, _g, u in bs}
    todo = [p for _n, p, g, _u in bs if target in g]
    if not todo:
        raise SystemExit(f"engine_build: the design names no method for {target}")
    got = {}
    while todo:
        p = todo.pop(0)
        if p in got:
            continue
        got[p] = module_text(p, db)
        # a module named bare (the flight algorithms' own uses, "01_math.pc") is its user's neighbour
        todo += [q if "/" in q else posixpath.join(posixpath.dirname(p), q) for q in uses.get(p, [])]
    return dict(sorted(got.items()))


def translate(texts, root=None, math=None, lang="rust", pkg=None, dispatch=False, title=TITLE):
    with tempfile.TemporaryDirectory(prefix="engine_build_") as tmp:
        files = []
        for p, t in texts.items():
            # a file by its design path: the relations' env.pc and the env group's are two files of one module
            f = pathlib.Path(tmp) / pathlib.PurePosixPath(p)
            if f.exists():
                raise SystemExit(f"engine_build: two modules at {p}")
            f.parent.mkdir(parents=True, exist_ok=True)
            f.write_text(t, encoding="utf-8")
            files.append(str(f))
        if lang == "rust":
            opts = ["--root", root, "--title", title] + ([] if dispatch else ["--no-dispatch"]) + (["--math", math] if math else [])
        else:
            opts = ["--pkg", pkg]
        cmd = [str(TNDB), "translate", lang, *files, *opts] if TNDB.is_file() else \
            ["node", str(ROOT / "design" / "js" / "pcode_cli.mjs"), lang, *files, *opts]
        r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f"engine_build: the translator refused the engine's models:\n{r.stderr.strip()[-3000:]}")
    return json.loads(r.stdout)


def outputs(db=None):
    """{repository path: text} of every generated file."""
    out = {}
    twin = {}
    for target, t in TARGETS.items():
        mods = modules(target, db)
        for rel, text in translate(mods, t["root"], t["math"], dispatch=t.get("dispatch", False)).items():
            out[f"{t['dir']}/{pathlib.PurePosixPath(rel).name}"] = text
        if target in TWIN["takes"]:
            twin.update(mods)
    for rel, text in translate(dict(sorted(twin.items())), lang="matlab", pkg=TWIN["pkg"]).items():
        out[f"{TWIN['dir']}/{rel}"] = text
    out.update(relations(db))
    return out


# ------------------------------------------------------------------ the relations (S7.16)

def _items(text):
    """(the module's head, [(name, the item's text with the comments above it)]) of a module's top-level items."""
    lines = text.rstrip("\n").split("\n")
    head, items, notes, i = [], [], [], 0
    while i < len(lines):
        ln = lines[i]
        m = DECL.match(ln)
        if ln.startswith("##"):
            notes.append(ln)
        elif m:
            body = [ln]
            if m.group(1) in ("const", "choice"):
                while body[-1].rstrip().endswith("\\") and i + 1 < len(lines):
                    i += 1
                    body.append(lines[i])
            else:
                while i + 1 < len(lines) and lines[i] != "end":
                    i += 1
                    body.append(lines[i])
            items.append((m.group(2), "\n".join(notes + body)))
            notes = []
        elif ln.strip():
            if items:
                raise SystemExit(f"engine_build: a top-level line the relations cannot place: {ln!r}")
            head += notes + [ln]
            notes = []
        elif notes and not items:
            head += notes
            notes = []
        i += 1
    return head, items


def _same(a, b):
    # how an item is drawn (`## inputs from:`) is its package's vectors', not the relation
    strip = lambda t: "\n".join(x for x in t.split("\n") if not x.startswith("## inputs from:"))  # noqa: E731
    return strip(a) == strip(b)


def relations_sources(db=None):
    """({design path: text} the relations crate is translated from, {a group's name: the library's name}): the library's
    modules whole, then each group's module without the items the library holds word for word (refused when a group writes
    one of the library's names another way: two relations of one name in one package)."""
    lib = {p: from_design.text(p, db) for p in from_design.paths(RELATIONS["library"], db)}
    if not lib:
        raise SystemExit(f"engine_build: the design holds no relations under {RELATIONS['library']}")
    held = {}
    for p, t in lib.items():
        head, items = _items(t)
        mod = next(x.split()[1] for x in head if x.startswith("module "))
        for name, text in items:
            held[name] = (mod, text)
    out, moved = dict(lib), {}
    for f in sorted(RELATIONS["groups"].glob("*.pc")):
        head, items = _items(f.read_text(encoding="utf-8"))
        mod = next(x.split()[1] for x in head if x.startswith("module "))
        kept = []
        for name, text in items:
            if name not in held:
                kept.append(text)
            elif _same(held[name][1], text):
                moved[f"{mod}::{name}"] = f"{held[name][0]}::{name}"
            else:
                raise SystemExit(f"engine_build: {name} is written one way in {RELATIONS['library']}{held[name][0]}.pc and another "
                                 f"in design/groups/{f.name}")
        out[f"design/groups/{f.name}"] = "\n".join(head) + "\n\n" + "\n\n".join(kept) + "\n"
    return out, moved


RELATIONS_CARGO = """# Generated by tools/engine_build.py; do not edit.
[package]
name = "{name}"
description = "{about}"
version.workspace = true
edition.workspace = true
authors.workspace = true
publish.workspace = true

[lib]
# the Rust library (its tests), and WebAssembly for the groups' test apps (tools/groupcode.py deliver)
crate-type = ["cdylib", "rlib"]

[dev-dependencies]
# read every decimal to the double it was written from
serde_json = {{ version = "1", features = ["float_roundtrip"] }}
"""

RELATIONS_LIB = """//! {about}; do not edit.
//! The translator's modules are under `gen`, re-exported here; `wasm` is the groups' test apps' WebAssembly face.
#![allow(clippy::all)]
pub mod gen;
pub use gen::*;
pub mod wasm;
"""

RELATIONS_WASM = """//! The relations as WebAssembly, for the groups' test apps (tools/groupcode.py deliver): one entry, `call`, that runs a
//! function by its index in NAMES on the doubles in the input buffer. Generated by tools/engine_build.py; do not edit.
#![allow(clippy::missing_safety_doc)]

/// Each function as a group's test app names it (its group's module, or shared), in the order the apps index them.
pub const NAMES: &[&str] = &[NAMES_HERE];
/// Where each is in this crate: an item the relations library holds word for word is the library's.
pub const CALLS: &[&str] = &[CALLS_HERE];
static mut BUF_IN: [f64; 256] = [0.0; 256];
static mut BUF_OUT: [f64; 64] = [0.0; 64];

#[cfg_attr(target_arch = "wasm32", no_mangle)]
pub extern "C" fn input() -> *mut f64 { core::ptr::addr_of_mut!(BUF_IN) as *mut f64 }
#[cfg_attr(target_arch = "wasm32", no_mangle)]
pub extern "C" fn output() -> *const f64 { core::ptr::addr_of!(BUF_OUT) as *const f64 }

/// Run NAMES[which] on the first `n` inputs; the number of outputs written, or -1.
#[cfg_attr(target_arch = "wasm32", no_mangle)]
pub extern "C" fn call(which: u32, n: u32) -> i32 {
    let Some(name) = CALLS.get(which as usize) else { return -1 };
    let x = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(BUF_IN) as *const f64, n.min(256) as usize) };
    match crate::dispatch::call(name, x) {
        Some(y) => {
            let out = unsafe { core::slice::from_raw_parts_mut(core::ptr::addr_of_mut!(BUF_OUT) as *mut f64, 64) };
            for (o, v) in out.iter_mut().zip(&y) { *o = *v; }
            y.len().min(64) as i32
        }
        None => -1,
    }
}
"""

RELATIONS_FIXTURES = """//! Every node's own test vectors (answers from outside the code), through the generated Rust.
//! Generated by tools/engine_build.py from design/groups/*.wire.json; do not edit.

#[test]
fn every_node_reproduces_its_own_test_vectors() {
    let v: serde_json::Value = serde_json::from_str(include_str!("fixtures.json")).unwrap();
    let mut n = 0;
    for f in v.as_array().unwrap() {
        let name = f["call"].as_str().unwrap();
        let x: Vec<f64> = f["inputs"].as_array().unwrap().iter().map(|a| a.as_f64().unwrap()).collect();
        let got = adcs_relations::dispatch::call(name, &x).unwrap_or_else(|| panic!("{name}: no such function"))[0];
        let (want, tol) = (f["expected"].as_f64().unwrap(), f["tolerance"].as_f64().unwrap());
        let err = (got - want).abs();
        assert!(err <= tol * want.abs().max(if want == 0.0 { 1.0 } else { 0.0 }),
                "{} {}: Rust gives {got:e}, the node expects {want:e} within {tol:e}", f["node"].as_str().unwrap(), f["vector"].as_str().unwrap());
        n += 1;
    }
    assert!(n > 0, "no test vector at all");
}

#[test]
fn every_function_of_the_test_apps_is_called_where_the_crate_holds_it() {
    use adcs_relations::wasm::{CALLS, NAMES};
    assert_eq!(NAMES.len(), CALLS.len());
    let groups: serde_json::Value = serde_json::from_str(include_str!("groups_vectors.json")).unwrap();
    for (n, c) in NAMES.iter().zip(CALLS) {
        let e = groups["vectors"].get(*n).unwrap_or_else(|| panic!("{n}: the groups' package drew no vector"));
        assert_eq!(groups["library"].get(*n).map(|x| x.as_str().unwrap()).unwrap_or(n), *c, "{n}: called where the crate does not hold it");
        if let Some(s) = e["sets"].as_array().unwrap().first().filter(|_| !e["proc"].as_bool().unwrap_or(false)) {
            let x: Vec<f64> = s["in"].as_array().unwrap().iter().map(|a| a.as_f64().unwrap()).collect();
            assert!(adcs_relations::dispatch::call(c, &x).is_some(), "{n}: the crate cannot call {c}");
        }
    }
}
"""


RELATIONS_VECTORS = """//! Translator = interpreter: every vector the interpreter (design/js/pcode.js) drew for each package the crate holds (the
//! relations library, physics_vectors.json; the groups' computing rows, groups_vectors.json), through the Rust translation.
//! A group's function the library holds word for word is called where the crate holds it (the file's `library`).
//! Generated by tools/engine_build.py; do not edit.
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

fn every_vector(file: &str, text: &str) {
    let v: serde_json::Value = serde_json::from_str(text).unwrap();
    let (mut n, mut values, mut exact, mut worst) = (0, 0, 0, 0.0f64);
    for (key, entry) in v["vectors"].as_object().unwrap() {
        let name = v["library"].get(key).map(|x| x.as_str().unwrap()).unwrap_or(key);
        let must_be_exact = entry["exact"].as_bool().unwrap();
        let sets = entry["sets"].as_array().unwrap();
        assert!(!sets.is_empty(), "{key}: no vector drawn");
        for s in sets {
            let (got, want): (Vec<Vec<f64>>, Vec<Vec<f64>>) = if entry["proc"].as_bool().unwrap_or(false) {
                let calls = s["calls"].as_array().unwrap();
                let xs: Vec<Vec<f64>> = calls.iter().map(|c| nums(&c["in"])).collect();
                let got = adcs_relations::dispatch::call_seq(name, &xs).unwrap_or_else(|| panic!("{key}: no such proc {name}"));
                (got, calls.iter().map(|c| nums(&c["out"])).collect())
            } else {
                let got = adcs_relations::dispatch::call(name, &nums(&s["in"])).unwrap_or_else(|| panic!("{key}: no such function {name}"));
                (vec![got], vec![nums(&s["out"])])
            };
            for (g, w) in got.iter().zip(&want) {
                assert_eq!(g.len(), w.len(), "{key}: output count");
                for (g, w) in g.iter().zip(w) {
                    let err = (g - w).abs() / w.abs().max(1e-300);
                    worst = worst.max(err);
                    values += 1;
                    if g.to_bits() == w.to_bits() { exact += 1; }
                    assert!(g.to_bits() == w.to_bits() || (!must_be_exact && err < 1e-12),
                            "{key}: Rust {g:e}, interpreter {w:e}{}", if must_be_exact { " (no transcendental: must agree bit for bit)" } else { "" });
                }
            }
            n += 1;
        }
    }
    println!("{file}: {n} vectors, {values} values, {exact} bit for bit, worst relative difference {worst:e}");
}

#[test]
fn every_vector_of_the_relations_library_is_reproduced() {
    every_vector("physics_vectors.json", include_str!("physics_vectors.json"));
}

#[test]
fn every_vector_of_the_groups_rows_is_reproduced() {
    every_vector("groups_vectors.json", include_str!("groups_vectors.json"));
}
"""


def relations(db=None):
    """{repository path: text} of the relations crate, its MATLAB package and its vectors."""
    import pcode
    R = RELATIONS
    crate = R["crate"]
    srcs, moved = relations_sources(db)
    out = {f"{crate}/Cargo.toml": RELATIONS_CARGO.format(name=R["name"], about=R["about"]),
           f"{crate}/src/lib.rs": RELATIONS_LIB.format(about=R["about"])}
    for rel, text in translate(srcs, "crate::gen", None, dispatch=True, title=R["about"]).items():
        out[f"{crate}/src/gen/{pathlib.PurePosixPath(rel).name}"] = text
    for rel, text in translate(srcs, lang="matlab", pkg=R["pkg"]).items():
        out[f"{R['mdir']}/{rel}"] = text
    # each package's vectors as its own files give them (the interpreter draws them in order: the same files, the same
    # draws); the groups' only for what the dispatcher calls with numbers, and not for what is the library's
    lib_files = sorted(from_design.folder(R["library"], db).glob("*.pc"))
    group_files = sorted(R["groups"].glob("*.pc"))
    plain = {f"{f['module']}::{f['name']}" for f in pcode.cli("signatures", *map(str, group_files))
             if all(PLAIN.match(x["type"]) for x in f["inputs"] + f["outputs"])}
    for pkg, files, keep in (("physics", lib_files, None), ("groups", group_files, plain)):
        n, twin = R["vectors"][pkg]
        vec = pcode.cli("vectors", *map(str, files), "--n", n, "--seed", SEED_VECTORS)
        if keep is not None:
            vec = {k: v for k, v in vec.items() if k in keep}
        body = {"generated_by": f"tools/engine_build.py (the interpreter, design/js/pcode.js, on the {pkg} package's own files)",
                "vectors": vec}
        lib = {k: moved[k] for k in vec if pkg == "groups" and moved.get(k, k) != k}
        if lib:     # a group's function the library holds: where the crate (and the twin's package) has it
            body["library"] = lib
        text = json.dumps(body, indent=1, sort_keys=True) + "\n"
        out[f"{crate}/tests/{pkg}_vectors.json"] = text
        out[twin] = text
    out[f"{crate}/tests/vectors.rs"] = RELATIONS_VECTORS
    # every node's own test vectors and the test apps' names, each called where the crate holds it
    wires = [json.loads(f.read_text(encoding="utf-8")) for f in sorted(R["groups"].glob("*.wire.json"))]
    page = lambda w, r: f"{'shared' if r['fn'] in w.get('shared', []) else w['group']}::{r['fn']}"  # noqa: E731
    fixtures = [{"node": r["id"], "vector": v["name"], "call": moved.get(page(w, r), page(w, r)), "inputs": v["inputs"],
                 "expected": v["expected"], "tolerance": v["tolerance"]}
                for w in wires for r in w["rows"] for v in r["vectors"]
                if r["fn"] and len(r["outputs"]) == 1 and all(SCALAR.match(t) for _, t in r["params"] + r["outputs"])]
    out[f"{crate}/tests/fixtures.json"] = json.dumps(fixtures, indent=1, sort_keys=True) + "\n"
    out[f"{crate}/tests/fixtures.rs"] = RELATIONS_FIXTURES
    names = sorted({page(w, r) for w in wires for r in w["rows"] if r["fn"]} & plain)
    out[f"{crate}/src/wasm.rs"] = RELATIONS_WASM.replace("NAMES_HERE", ", ".join(json.dumps(x) for x in names)) \
        .replace("CALLS_HERE", ", ".join(json.dumps(moved.get(x, x)) for x in names))
    return out


def gen(a):
    db = pathlib.Path(a.design).resolve() if a.design else None
    outs = outputs(db)
    dirs = [t["dir"] for t in [*TARGETS.values(), TWIN]] + [RELATIONS["crate"], RELATIONS["mdir"]]
    have = {p.relative_to(ROOT).as_posix() for d in dirs if (ROOT / d).is_dir() for p in (ROOT / d).rglob("*") if p.is_file()}
    stale = sorted(rel for rel, t in outs.items() if not (ROOT / rel).is_file() or (ROOT / rel).read_text() != t)
    extra = sorted(have - set(outs))
    if a.check:
        for p in stale:
            print(f"engine_build: {p} is not what the design gives (run python3 tools/engine_build.py gen)")
        for p in extra:
            print(f"engine_build: {p} is in a generated folder but the design does not give it")
        print(f"engine_build --check: {len(outs)} generated file(s), {len(stale) + len(extra)} problem(s)")
        return 1 if stale or extra else 0
    for p in extra:
        (ROOT / p).unlink()
    for rel in stale:
        write_text(ROOT / rel, outs[rel])
    print(f"engine_build: {len(outs)} generated file(s), {len(stale)} written, {len(extra)} removed")
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    p = sp.add_parser("gen")
    p.add_argument("--check", action="store_true")
    p.add_argument("--design")
    p = sp.add_parser("modules")
    p.add_argument("--design")
    a = ap.parse_args(argv)
    if a.cmd == "gen":
        return gen(a)
    db = pathlib.Path(a.design).resolve() if a.design else None
    for target in TARGETS:
        print(f"{target}: " + ", ".join(modules(target, db)))
    srcs, moved = relations_sources(db)
    print(f"{RELATIONS['name']}: " + ", ".join(srcs) + f" ({len(moved)} item(s) of the groups' the library's)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
