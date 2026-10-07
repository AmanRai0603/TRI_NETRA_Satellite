//! trinetra-pcode: the pseudocode v2 language (docs/PSEUDOCODE_V2.md) in Rust: units and dimension
//! algebra, lexer, parser, checker and interpreter, a port of design/js/pcode.js held equal to it
//! on every test vector the JavaScript drew (tests/vectors.rs) and on what its checker accepts
//! and refuses (tests/checker.rs). Pure Rust, no I/O: it builds for wasm32-unknown-unknown.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
//!
//! It also holds the translators (src/gen/): a checked program to Rust, C and MATLAB
//! (`Program::to_rust`, `to_c`, `to_matlab`, and `matlab_runtime`), ports of design/js/pcode_gen.js
//! and pcode_c.js that write byte for byte the files the JavaScript writes
//! (tests/test_translators_lib.py holds them to it; `tndb translate` drives them).
//!
//! ```
//! let src = "module m\nfn f(x: real[km]) -> y: real[m]\n    y = 2*x\nend\n";
//! let p = trinetra_pcode::compile(&[("m.pc", src)]).unwrap();
//! let y = p.call("f", &[trinetra_pcode::Value::Num(1500.0)]).unwrap();
//! assert_eq!(y, vec![trinetra_pcode::Value::Num(3000.0)]);
//! ```
//!
//! # The same numbers as the JavaScript
//!
//! Every value is an f64 (an int too, as in JavaScript), and every operation is the JavaScript's,
//! in its order: `x^k` by repeated multiplication, sums from the left, min/max/abs/clamp/sign
//! written out, `round` half away from zero, `fmod` and `rem` C's, no fused multiply-add. The maths
//! library is JavaScript's own, ported (src/vmath.rs): `pow`, `log`, `log10` and `atan2` line for
//! line from the fdlibm code of V8 (`src/base/ieee754.cc`), and `tan`, `asin`, `acos`, `atan`
//! and `exp` from the `libm` crate, whose fdlibm port gives V8's bits (tests/math.rs holds both to
//! Node's Math on 40 000 arguments a function, no bit different). So every vector is reproduced
//! bit for bit, but for one exception:
//!
//! **`sin` and `cos`.** Node 22's V8 is built with `v8_use_libm_trig_functions`: its `Math.sin`
//! and `Math.cos` are glibc's (`sysdeps/ieee754/dbl-64/s_sin.c`, forked into V8's
//! `third_party/glibc`), not fdlibm. That code is LGPL, so it is not ported here; this crate uses
//! the fdlibm `sin` and `cos` (the `libm` crate), which are within 1 ulp of glibc's and differ
//! from them in the last bit on about 1 % of arguments. A vector whose function uses `sin` or
//! `cos` (directly or through what it calls) can therefore differ in its last bits; the tests
//! hold those to 1e-12 relative, the tolerance the generated crates use for every transcendental
//! (docs/PSEUDOCODE_V2.md), and count how many values are bit for bit anyway.
//!
//! A NaN's own bits (its sign and payload) are not held: JavaScript cannot tell one NaN from another.
//!
//! # Where the JavaScript itself fails
//!
//! A few malformed sources make the JavaScript checker throw a TypeError rather than name a
//! problem (a table column that is not real, `min()` with no input, `clamp` with fewer than three,
//! an int matrix in a matrix product). The Rust names these `ErrorKind::Internal` and stops. Names
//! that every JavaScript object has (`toString`, `constructor`, ...) are refused as declarations as
//! the JavaScript refuses them; used undeclared, or as a unit, they are simply unknown here. An
//! index that is not a whole number (reachable only by passing a fraction for an int input) stops
//! the run here, where JavaScript reads `undefined`.

// `!(a < b)` is the JavaScript's test, NaN and all, and is kept as it is written there
#![allow(clippy::neg_cmp_op_on_partial_ord)]

mod ast;
mod check;
mod error;
mod gen;
mod interp;
mod jsfmt;
mod parse;
mod units;
mod vmath;

pub use ast::{FnKind, TableMode, Ty};
pub use error::{ErrorKind, PcodeError, Pos, RunError};
pub use gen::{files_json, lit, matlab_runtime, matlab_runtime_files, Files, RustOptions};
pub use interp::{State, Value};
pub use units::{dim_text, Dim, UNITS};

/// The language and its version, as design/js/pcode.js names it.
pub const VERSION: &str = "trinetra-pcode/2";

/// JavaScript's `Math` functions as the interpreter computes them (see the crate's header), and
/// `Math.round`, for whoever draws inputs as the JavaScript does.
pub mod math {
    pub use crate::vmath::{acos, asin, atan, atan2, cos, exp, log, log10, pow, sin, tan};

    /// `Math.round`: the nearest whole number, a half towards +infinity.
    pub fn round(x: f64) -> f64 {
        if !x.is_finite() || x == 0.0 {
            return x;
        }
        let f = x.floor();
        let r = if x - f >= 0.5 { f + 1.0 } else { f };
        if r == 0.0 && x < 0.0 {
            -0.0
        } else {
            r
        }
    }
}

/// Text as JavaScript writes it, for whoever must say what a JavaScript page says.
pub mod js {
    /// `String(x)` of a number: `1e+21`, `1e-7`, `0.1`, `-0` as `0`.
    pub fn number_text(x: f64) -> String {
        crate::jsfmt::num(x)
    }
    /// `JSON.stringify(s)` of a string.
    pub fn json_string(s: &str) -> String {
        crate::jsfmt::json_str(s)
    }
}

/// Parse and check the files together (they may call one another), as the JavaScript's `compile`:
/// a program when there is no problem, otherwise every problem found. A file that does not parse
/// gives its first problem, and the files are then not checked.
pub fn compile(sources: &[(&str, &str)]) -> Result<Program, Vec<PcodeError>> {
    let mut exprs = Vec::new();
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for (file, text) in sources {
        match parse::parse(text, file, &mut exprs) {
            Ok(f) => files.push(f),
            Err(e) => errors.push(e),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let checked = check::check(files, exprs)?;
    Ok(Program { i: interp::Interp::new(checked) })
}

/// The problems in the files, as `pcode.py check` lists them (empty when there are none).
pub fn check(sources: &[(&str, &str)]) -> Vec<PcodeError> {
    compile(sources).err().unwrap_or_default()
}

/// An input or output of a function, as declared.
#[derive(Clone, Debug)]
pub struct ParamInfo<'a> {
    pub name: &'a str,
    /// an input handed by reference (`inout`): its value after the call follows the outputs
    pub inout: bool,
    pub ty: &'a Ty,
    /// the range `in lo .. hi`, in SI (a bound that is not constant is None)
    pub range: Option<(Option<f64>, Option<f64>)>,
}

/// A function or proc: where it is, what it takes and gives.
#[derive(Clone, Debug)]
pub struct FnInfo<'a> {
    pub module: &'a str,
    pub name: &'a str,
    pub kind: FnKind,
    pub inputs: Vec<ParamInfo<'a>>,
    pub outputs: Vec<ParamInfo<'a>>,
    /// its `##` lines
    pub doc: &'a [String],
}

/// A checked program, ready to run.
pub struct Program {
    i: interp::Interp,
}

impl Program {
    fn param_info<'a>(&'a self, p: &'a ast::Param) -> ParamInfo<'a> {
        let range = p.range.map(|(lo, hi)| (self.i.c.ann[lo].bound, self.i.c.ann[hi].bound));
        ParamInfo { name: &p.name, inout: p.inout, ty: &p.ty, range }
    }
    fn info<'a>(&'a self, f: &'a ast::Func) -> FnInfo<'a> {
        FnInfo {
            module: &f.module,
            name: &f.name,
            kind: f.kind,
            inputs: f.params.iter().map(|p| self.param_info(p)).collect(),
            outputs: f.outs.iter().map(|p| self.param_info(p)).collect(),
            doc: &f.doc,
        }
    }
    fn fn_ix(&self, name: &str) -> Result<usize, RunError> {
        self.i.c.fns.iter().position(|f| f.name == name).ok_or_else(|| RunError(format!("no fn {name}")))
    }

    /// Every fn and proc, in the order they are declared.
    pub fn functions(&self) -> Vec<FnInfo<'_>> {
        self.i.c.fns.iter().map(|f| self.info(f)).collect()
    }
    /// A fn or proc by its name.
    pub fn function(&self, name: &str) -> Option<FnInfo<'_>> {
        self.i.c.fns.iter().find(|f| f.name == name).map(|f| self.info(f))
    }
    /// The fields of a record, in order, with their ranges.
    pub fn record_fields(&self, name: &str) -> Option<Vec<ParamInfo<'_>>> {
        self.i.c.records.iter().find(|r| r.name == name).map(|r| r.fields.iter().map(|f| self.param_info(f)).collect())
    }
    /// The options of a choice, in order (the first is numbered 0).
    pub fn choice_options(&self, name: &str) -> Option<Vec<&str>> {
        self.i.c.choices.iter().find(|c| c.name == name).map(|c| c.options.iter().map(|o| o.0.as_str()).collect())
    }
    /// The modules, in the order their files were given, with each file's `##` lines.
    pub fn modules(&self) -> Vec<(&str, &[String])> {
        self.i.c.modules.iter().map(|(m, d)| (m.as_str(), d.as_slice())).collect()
    }

    /// Call a fn with SI inputs; its outputs, in order, then each inout input's value after the call. A proc
    /// called so starts from fresh state.
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Vec<Value>, RunError> {
        let fi = self.fn_ix(name)?;
        self.check_args(fi, args)?;
        self.i.run_fn(fi, args.to_vec(), None)
    }
    /// Call a proc with its state, which the call carries on.
    pub fn call_with_state(&self, name: &str, args: &[Value], state: &mut State) -> Result<Vec<Value>, RunError> {
        let fi = self.fn_ix(name)?;
        self.check_args(fi, args)?;
        if state.func != fi {
            return Err(RunError(format!("this state is {}'s, not {name}'s", self.i.c.fns[state.func].name)));
        }
        self.i.run_fn(fi, args.to_vec(), Some(state))
    }
    fn check_args(&self, fi: usize, args: &[Value]) -> Result<(), RunError> {
        let f = &self.i.c.fns[fi];
        if args.len() != f.params.len() {
            return Err(RunError(format!("{} takes {} inputs", f.name, f.params.len())));
        }
        Ok(())
    }
    /// A proc's state as it starts.
    pub fn new_state(&self, name: &str) -> Result<State, RunError> {
        let fi = self.fn_ix(name)?;
        self.i.new_state(fi)
    }
    /// A table's row at a key (SI), every column, as the table interpolates or steps.
    pub fn lookup(&self, table: &str, x: f64) -> Result<Vec<f64>, RunError> {
        let t = self.i.c.tables.iter().find(|t| t.name == table).ok_or_else(|| RunError(format!("no table {table}")))?;
        Ok(interp::lookup(t, x))
    }

    /// Does a fn call any of these builtins, itself or through the fns it calls? (The vectors call a
    /// fn `exact` when it uses none of `sin cos tan asin acos atan atan2 exp log log10 pow`.)
    pub fn uses(&self, name: &str, builtins: &[&str]) -> bool {
        fn walk(c: &check::Checked, fi: usize, builtins: &[&str], seen: &mut Vec<bool>) -> bool {
            if seen[fi] {
                return false;
            }
            seen[fi] = true;
            let mut calls = Vec::new();
            let mut found = false;
            // a normal draw from a stream is Box-Muller's sqrt, log, sin and cos
            let box_muller = builtins.iter().any(|b| matches!(*b, "sin" | "cos" | "log"));
            ast::visit_stmts(&c.fns[fi].body, &c.exprs, &mut |e| match &c.ann[e].target {
                check::Target::Builtin(b) if builtins.contains(b) => found = true,
                check::Target::Builtin("normal" | "normal3") if box_muller => found = true,
                check::Target::Fn(g) => calls.push(*g),
                _ => {}
            });
            found || calls.into_iter().any(|g| walk(c, g, builtins, seen))
        }
        match self.fn_ix(name) {
            Ok(fi) => walk(&self.i.c, fi, builtins, &mut vec![false; self.i.c.fns.len()]),
            Err(_) => false,
        }
    }

    /// The zero of a type: every number 0, every bool false.
    pub fn zero(&self, ty: &Ty) -> Value {
        interp::zero_of(&self.i.c, ty)
    }
    /// A value of a type from its numbers, as `Value::flatten` writes them (a bool from 0 or not).
    pub fn value_from_flat(&self, ty: &Ty, nums: &mut dyn Iterator<Item = f64>) -> Option<Value> {
        Some(match ty {
            Ty::Int | Ty::Real(_) | Ty::Choice(_) => Value::Num(nums.next()?),
            Ty::Bool => Value::Bool(nums.next()? != 0.0),
            Ty::Stream => Value::Arr((0..6).map(|_| nums.next().map(Value::Num)).collect::<Option<Vec<_>>>()?),
            Ty::Str => return None,
            Ty::Arr(n, of) => Value::Arr((0..*n).map(|_| self.value_from_flat(of, nums)).collect::<Option<Vec<_>>>()?),
            Ty::Rec(name) => {
                let r = self.i.c.records.iter().find(|r| &r.name == name)?;
                Value::Rec(r.fields.iter().map(|f| self.value_from_flat(&f.ty, nums)).collect::<Option<Vec<_>>>()?)
            }
            Ty::Tuple(_) => return None,
        })
    }
}

/// The JavaScript's generator for test vectors (xorshift64*): numbers in [0, 1).
#[derive(Clone, Debug)]
pub struct Prng(u64);

impl Prng {
    pub fn new(seed: i64) -> Self {
        Prng((seed as u64).wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(1))
    }
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f64 {
        let mut s = self.0;
        s ^= s >> 12;
        s ^= s << 25;
        s ^= s >> 27;
        self.0 = s;
        (s.wrapping_mul(0x2545F4914F6CDD1D) >> 11) as f64 / 9007199254740992.0
    }
}
