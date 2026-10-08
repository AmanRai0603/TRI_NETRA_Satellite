//! Rust = JavaScript on every vector: each vector the JavaScript interpreter (design/js/pcode.js)
//! drew for the physics, the language's self-test, the flight software's algorithms and the
//! groups' relations, run through this crate's interpreter from the same sources.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
//!
//! A function that uses neither `sin` nor `cos`, itself or through what it calls, must agree bit
//! for bit: its arithmetic and its maths library are the JavaScript's own (see src/lib.rs). One
//! that does may differ in its last bits (Node's sin and cos are glibc's, not ported here), and is
//! held to 1e-12 relative; the test says how many of its values are bit for bit anyway.

mod common;

use common::*;
use trinetra_pcode::{FnKind, Value};

const TOLERANCE: f64 = 1e-12;

#[derive(Default)]
struct Tally {
    functions: usize,
    vectors: usize,
    calls: usize,
    values: usize,
    bit_for_bit: usize,
    within_tolerance: usize,
    worst: f64,
    /// the functions with a value not bit for bit, and how many
    inexact: Vec<(String, usize)>,
}

fn run_package(pkg: &Package) -> Tally {
    let srcs = sources(pkg.src);
    let prog = compile(&srcs);
    let entries = vectors(pkg.vectors);
    assert!(!entries.is_empty(), "{}: no vectors in {}", pkg.name, pkg.vectors);
    let mut t = Tally::default();
    let mut failures = Vec::new();
    for e in &entries {
        let name = bare(&e.name);
        let info = prog.function(name).unwrap_or_else(|| panic!("{}: {} is not in {}", pkg.name, e.name, pkg.src));
        assert_eq!(format!("{}::{}", info.module, info.name), e.name, "{}: the module of {}", pkg.name, e.name);
        let trig = prog.uses(name, &["sin", "cos"]);
        let tys: Vec<_> = info.inputs.iter().map(|p| p.ty.clone()).collect();
        // the outputs' types, then each inout input's (its value after the call follows the outputs; a buffer its length first)
        let otys: Vec<_> = info.outputs.iter().chain(info.inputs.iter().filter(|p| p.inout)).map(|p| p.ty.clone()).collect();
        t.functions += 1;
        let mut inexact = 0;
        for s in &e.sets {
            t.vectors += 1;
            let mut state = (info.kind == FnKind::Proc).then(|| prog.new_state(name).unwrap());
            for (ci, c) in s.calls.iter().enumerate() {
                let mut it = c.input.iter().copied();
                let args: Vec<Value> = tys.iter().map(|ty| prog.value_from_flat(ty, &mut it).expect("too few inputs")).collect();
                assert!(it.next().is_none(), "{}: more inputs than {name} takes", e.name);
                let r = match &mut state {
                    Some(st) => prog.call_with_state(name, &args, st),
                    None => prog.call(name, &args),
                };
                let got: Vec<f64> = match r {
                    Ok(v) => v.iter().zip(&otys).flat_map(|(v, ty)| prog.flatten_as(ty, v)).collect(),
                    Err(er) => {
                        failures.push(format!("{} vector {} call {ci}: the run stops: {er}", e.name, t.vectors));
                        continue;
                    }
                };
                t.calls += 1;
                if got.len() != c.output.len() {
                    failures.push(format!("{}: {} outputs, the JavaScript {}", e.name, got.len(), c.output.len()));
                    continue;
                }
                for (k, (g, w)) in got.iter().zip(&c.output).enumerate() {
                    t.values += 1;
                    if g.to_bits() == w.to_bits() {
                        t.bit_for_bit += 1;
                        continue;
                    }
                    inexact += 1;
                    let rel = (g - w).abs() / w.abs().max(1e-300);
                    t.worst = t.worst.max(rel);
                    if trig && !e.exact && rel <= TOLERANCE {
                        t.within_tolerance += 1;
                    } else {
                        failures.push(format!(
                            "{} vector {} call {ci} output {k}: Rust {g:e}, JavaScript {w:e} (relative {rel:e}){}",
                            e.name,
                            t.vectors,
                            if trig { "" } else { " -- no sin or cos: must be bit for bit" }
                        ));
                    }
                }
            }
        }
        if inexact > 0 {
            t.inexact.push((e.name.clone(), inexact));
        }
    }
    println!(
        "{}: {} functions, {} vectors ({} calls), {} values: {} bit for bit, {} within {TOLERANCE:e} (worst {:e})",
        pkg.name, t.functions, t.vectors, t.calls, t.values, t.bit_for_bit, t.within_tolerance, t.worst
    );
    for (n, k) in &t.inexact {
        println!("    {n}: {k} value(s) not bit for bit (uses sin or cos)");
    }
    assert!(failures.is_empty(), "{}: {} difference(s):\n{}", pkg.name, failures.len(), failures[..failures.len().min(40)].join("\n"));
    t
}

#[test]
fn physics_vectors() {
    run_package(&PACKAGES[0]);
}

#[test]
fn selftest_vectors() {
    let t = run_package(&PACKAGES[1]);
    // its only sin and cos are a stream's normal draws (Box-Muller): every other value bit for bit
    let normal = ["selftest::gauss", "selftest::jitter", "selftest::noisy"];
    assert!(t.inexact.iter().all(|(n, _)| normal.contains(&n.as_str())), "only the normal draws may differ in their last bit: {:?}", t.inexact);
    assert_eq!(t.bit_for_bit + t.within_tolerance, t.values, "the self-test: every value bit for bit or, a normal draw, within 1e-12");
}

#[test]
fn flight_software_vectors() {
    run_package(&PACKAGES[2]);
}

#[test]
fn groups_vectors() {
    run_package(&PACKAGES[3]);
}
