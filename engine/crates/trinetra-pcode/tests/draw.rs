//! The vectors drawn again, in Rust: design/js/pcode_cli.mjs's `vectors` (as tools/pcode.py and
//! tools/groupcode.py run it) ported over this crate, from the same seed, must draw the same inputs,
//! keep the same sets and leave out the same ones (a run that stops, an output not finite). That
//! holds the checker's range bounds, the generator, the JavaScript's Math.exp, Math.log and
//! Math.round, and the interpreter's run errors to the JavaScript's, bit for bit.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

mod common;

use common::*;
use std::collections::BTreeMap;
use trinetra_pcode::{math, FnKind, ParamInfo, Prng, Program, Ty, Value};

const SEED: i64 = 20261003;

/// per function (`module::name`), its sets, each a run of (inputs, outputs) calls
type Drawn = BTreeMap<String, Vec<Vec<(Vec<f64>, Vec<f64>)>>>;

struct Drawer<'a> {
    prog: &'a Program,
    rand: Prng,
    /// the length the function at hand draws its buffers at (`## length: 64`, else 8)
    blen: usize,
}

impl Drawer<'_> {
    fn scalar(&mut self, lo: f64, hi: f64, int: bool) -> f64 {
        let r = self.rand.next();
        let v = if lo > 0.0 && hi / lo > 20.0 { math::exp(math::log(lo) + r * (math::log(hi) - math::log(lo))) } else { lo + r * (hi - lo) };
        if int {
            // an int has no sign of zero: the JavaScript adds 0 (Math.round of a draw in (-0.5, 0) is -0)
            math::round(v) + 0.0
        } else {
            v
        }
    }
    fn draw(&mut self, p: &ParamInfo) -> Value {
        // a bound that is not constant is JavaScript's null: 0 in arithmetic
        let range = p.range.map(|(lo, hi)| (lo.unwrap_or(0.0), hi.unwrap_or(0.0)));
        self.one(p.ty, range)
    }
    fn one(&mut self, t: &Ty, range: Option<(f64, f64)>) -> Value {
        match t {
            Ty::Rec(name) => {
                let fields = self.prog.record_fields(name).unwrap();
                Value::Rec(fields.iter().map(|f| self.draw(f)).collect())
            }
            Ty::Arr(n, of) => Value::Arr((0..*n).map(|_| self.one(of, range)).collect()),
            Ty::Buf(of) => Value::Arr((0..self.blen).map(|_| self.one(of, range)).collect()),
            Ty::Bool => Value::Bool(self.rand.next() < 0.5),
            // a stream: a random key, a counter below 1000, a spare in -1 .. 1 that is there or not
            Ty::Stream => {
                let a = (self.rand.next() * 4294967296.0).floor();
                let b = (self.rand.next() * 4294967296.0).floor();
                let c = (self.rand.next() * 1000.0).floor();
                let d = self.rand.next() * 2.0 - 1.0;
                let e = if self.rand.next() < 0.5 { 1.0 } else { 0.0 };
                Value::Arr([a, b, 0.0, c, d, e].into_iter().map(Value::Num).collect())
            }
            Ty::Choice(name) => {
                let n = self.prog.choice_options(name).unwrap().len() as f64;
                Value::Num((self.rand.next() * n).floor().min(n - 1.0))
            }
            Ty::Int => {
                let (lo, hi) = range.unwrap_or((0.0, 10.0));
                Value::Num(self.scalar(lo, hi, true))
            }
            _ => {
                let (lo, hi) = range.unwrap_or((0.1, 10.0));
                Value::Num(self.scalar(lo, hi, false))
            }
        }
    }
}

fn width(prog: &Program, t: &Ty, blen: usize) -> usize {
    match t {
        Ty::Arr(n, of) => n * width(prog, of, blen),
        Ty::Buf(of) => 1 + blen * width(prog, of, blen),
        Ty::Rec(name) => prog.record_fields(name).unwrap().iter().map(|f| width(prog, f.ty, blen)).sum(),
        Ty::Stream => 6,
        _ => 1,
    }
}

/// values of these types as numbers (a buffer its length first)
fn flat(prog: &Program, tys: &[&Ty], vs: &[Value]) -> Vec<f64> {
    vs.iter().zip(tys).flat_map(|(v, ty)| prog.flatten_as(ty, v)).collect()
}

/// `vectors --n N --budget B --seed S`: per function, its sets of (inputs, outputs) calls.
fn draw_all(prog: &Program, n_want: usize, budget: f64) -> Drawn {
    let mut d = Drawer { prog, rand: Prng::new(SEED), blen: 8 };
    let mut res = BTreeMap::new();
    for f in prog.functions() {
        let asked = f.doc.iter().find_map(|l| l.strip_prefix("vectors:").map(str::trim).filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())));
        d.blen = f.doc.iter().find_map(|l| l.strip_prefix("length:").map(str::trim).filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))).map_or(8, |s| s.parse().unwrap());
        let itys: Vec<&Ty> = f.inputs.iter().map(|p| p.ty).collect();
        let otys: Vec<&Ty> = f.outputs.iter().chain(f.inputs.iter().filter(|p| p.inout)).map(|p| p.ty).collect();
        let n = match asked {
            Some(a) => a.parse::<usize>().unwrap(),
            None => {
                let w: usize = f.inputs.iter().chain(&f.outputs).map(|x| width(prog, x.ty, d.blen)).sum();
                (budget / w as f64).floor().min(n_want as f64).max(4.0) as usize
            }
        };
        let mut sets = Vec::new();
        if f.kind == FnKind::Proc {
            let want = (n as f64 / 4.0).max(2.0);
            let mut k = 0;
            while k < n * 3 && (sets.len() as f64) < want {
                k += 1;
                let mut st = prog.new_state(f.name).unwrap();
                let mut calls = Vec::new();
                let mut ok = true;
                for _ in 0..8 {
                    let ins: Vec<Value> = f.inputs.iter().map(|p| d.draw(p)).collect();
                    match prog.call_with_state(f.name, &ins, &mut st) {
                        Ok(o) if flat(prog, &otys, &o).iter().all(|x| x.is_finite()) => calls.push((flat(prog, &itys, &ins), flat(prog, &otys, &o))),
                        _ => {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    sets.push(calls);
                }
            }
            res.insert(format!("{}::{}", f.module, f.name), sets);
            continue;
        }
        // `## inputs from: g`: g's outputs give the inputs of the same name
        let from = f.doc.iter().find_map(|l| l.strip_prefix("inputs from:").map(str::trim).filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')));
        let g = from.map(|g| prog.function(g).unwrap_or_else(|| panic!("{}: inputs from {g}, which is no fn", f.name)));
        let mut k = 0;
        while k < n * 3 && sets.len() < n {
            k += 1;
            let mut ins: Vec<Value> = f.inputs.iter().map(|p| d.draw(p)).collect();
            if let Some(g) = &g {
                let gins: Vec<Value> = g.inputs.iter().map(|p| d.draw(p)).collect();
                let Ok(go) = prog.call(g.name, &gins) else { continue };
                for (o, v) in g.outputs.iter().zip(go) {
                    if let Some(i) = f.inputs.iter().position(|p| p.name == o.name) {
                        ins[i] = v;
                    }
                }
            }
            if let Ok(o) = prog.call(f.name, &ins) {
                let fo = flat(prog, &otys, &o);
                if fo.iter().all(|x| x.is_finite()) {
                    sets.push(vec![(flat(prog, &itys, &ins), fo)]);
                }
            }
        }
        res.insert(format!("{}::{}", f.module, f.name), sets);
    }
    res
}

fn same(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
}

/// The drawn inputs against the JavaScript's, set by set; returns (functions, sets, calls) compared.
fn redraw(pkg: &Package, n: usize, budget: f64, all_functions: bool) -> (usize, usize, usize) {
    let prog = compile(&sources(pkg.src));
    let mine = draw_all(&prog, n, budget);
    let theirs = vectors(pkg.vectors);
    if all_functions {
        let names: Vec<&String> = mine.iter().filter(|(_, s)| !s.is_empty()).map(|(k, _)| k).collect();
        let js: Vec<&String> = {
            let mut v: Vec<&String> = theirs.iter().map(|e| &e.name).collect();
            v.sort();
            v
        };
        assert_eq!(names, js, "{}: the functions with vectors", pkg.name);
    }
    let (mut sets, mut calls, mut bad) = (0, 0, Vec::new());
    for e in &theirs {
        let m = mine.get(&e.name).unwrap_or_else(|| panic!("{}: no draw for {}", pkg.name, e.name));
        if m.len() != e.sets.len() {
            bad.push(format!("{}: {} sets drawn, the JavaScript {}", e.name, m.len(), e.sets.len()));
            continue;
        }
        for (si, (ms, js)) in m.iter().zip(&e.sets).enumerate() {
            sets += 1;
            for (ci, ((mi, mo), jc)) in ms.iter().zip(&js.calls).enumerate() {
                calls += 1;
                if !same(mi, &jc.input) {
                    bad.push(format!("{} set {si} call {ci}: the inputs differ", e.name));
                }
                // the outputs are the replay test's (tests/vectors.rs): here, that the same sets are kept
                assert_eq!(mo.len(), jc.output.len(), "{}: outputs", e.name);
            }
        }
    }
    println!("{}: {} functions, {sets} sets, {calls} calls drawn as the JavaScript drew them", pkg.name, theirs.len());
    assert!(bad.is_empty(), "{}:\n{}", pkg.name, bad[..bad.len().min(30)].join("\n"));
    (theirs.len(), sets, calls)
}

#[test]
fn physics_drawn_again() {
    redraw(&PACKAGES[0], 12, 1e9, true);
}

#[test]
fn selftest_drawn_again() {
    redraw(&PACKAGES[1], 12, 1e9, true);
}

#[test]
fn flight_software_drawn_again() {
    redraw(&PACKAGES[2], 32, 2000.0, true);
}

#[test]
fn groups_drawn_again() {
    // tools/groupcode.py keeps the functions its generated dispatcher can call
    redraw(&PACKAGES[3], 8, 1e9, false);
}
